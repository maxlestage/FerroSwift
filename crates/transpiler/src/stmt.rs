//! Statements and blocks. An `if`, a `match` or a block used as a value is lowered to Swift
//! statements that deliver the value through a [`Sink`]: `return`, an assignment, a view
//! of a result builder…

use syn::spanned::Spanned;
use syn::{Block, Expr, Local, Pat, Stmt};

use crate::expr::{Ex, P_PRIMARY};
use crate::names;
use crate::translate::{INDENT, Translator, indent_lines};
use crate::ty::Ty;

/// Where the value of a block goes.
#[derive(Clone, Debug, PartialEq)]
pub enum Sink {
    /// Returned from the function.
    Return,
    /// Assigned to a variable (its Swift name).
    Assign(String),
    /// Unused.
    Discard,
    /// Written as a bare expression: a view in a result builder, or a branch of a Swift
    /// `if`/`switch` expression.
    Yield,
    /// The body of an initializer: `self = value`.
    InitSelf,
}

impl Translator<'_> {
    /// The statements of a block, in a new scope.
    pub fn block_lines(&mut self, b: &Block, sink: Sink) -> Vec<String> {
        self.reassigned_lets.extend(assigned_names(&b.stmts));
        self.push_scope();
        let lines = self.stmts(&b.stmts, sink, b.span().start().line, b.span().end().line);
        self.pop_scope();
        lines
    }

    pub fn stmts(&mut self, stmts: &[Stmt], sink: Sink, open: usize, close: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut prev = open;
        let mut writebacks: Vec<(usize, String)> = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let last = i + 1 == stmts.len();
            let start = s.span().start().line;
            let end = s.span().end().line;
            // A blank line between two groups of statements stays one, comments included.
            let leading = self.comments.take_between_lines(prev, start);
            let mut at = prev;
            for (line, text) in leading {
                if i > 0 && line > at + 1 && !out.last().is_some_and(String::is_empty) {
                    out.push(String::new());
                }
                out.push(text);
                at = line;
            }
            if i > 0 && start > at + 1 && !out.last().is_some_and(String::is_empty) {
                out.push(String::new());
            }
            prev = end;
            // `let (a, b) = v.split_at_mut(k)`: `a` and `b` are copies of the two parts of
            // `v`, written back after the last statement that uses them.
            if let Stmt::Local(l) = s {
                if let Some((lines, parts, writeback)) = self.split_at_mut_local(l) {
                    let last_use = stmts
                        .iter()
                        .enumerate()
                        .skip(i + 1)
                        .filter(|(_, s)| parts.iter().any(|p| mentions(s, p)))
                        .map(|(j, _)| j)
                        .next_back()
                        .unwrap_or(i);
                    writebacks.push((last_use, writeback));
                    out.extend(lines);
                    while let Some(pos) = writebacks.iter().position(|(j, _)| *j == i) {
                        out.push(writebacks.remove(pos).1);
                    }
                    continue;
                }
            }
            let mut lines = match s {
                Stmt::Local(l) => self.with_prelude(|t| t.local(l)),
                Stmt::Item(item) => self.local_item(item),
                Stmt::Expr(e, None) if last => self.with_prelude(|t| t.tail(e, &sink)),
                Stmt::Expr(e, _) => self.with_prelude(|t| t.expr_stmt(e)),
                Stmt::Macro(m) => {
                    if last && m.semi_token.is_none() && sink != Sink::Discard {
                        let e = Expr::Macro(syn::ExprMacro {
                            attrs: m.attrs.clone(),
                            mac: m.mac.clone(),
                        });
                        self.with_prelude(|t| t.tail(&e, &sink))
                    } else {
                        self.with_prelude(|t| t.macro_stmt(&m.mac))
                    }
                }
            };
            if let Some(c) = self.comments.take_trailing(end) {
                if let Some(l) = lines.last_mut() {
                    l.push(' ');
                    l.push_str(&c);
                }
            }
            out.append(&mut lines);
            while let Some(pos) = writebacks.iter().position(|(j, _)| *j == i) {
                out.push(writebacks.remove(pos).1);
            }
        }
        out.extend(self.comments.take_between(prev, close));
        out
    }

    /// `let (a, b) = v.split_at_mut(k)`: the lines declaring the two parts, their names, and
    /// the statement writing them back into `v`.
    fn split_at_mut_local(&mut self, l: &Local) -> Option<(Vec<String>, Vec<String>, String)> {
        let Pat::Tuple(t) = &l.pat else { return None };
        let names: Vec<String> = t
            .elems
            .iter()
            .filter_map(|p| match p {
                Pat::Ident(i) => Some(i.ident.to_string()),
                _ => None,
            })
            .collect();
        if names.len() != 2 || t.elems.len() != 2 {
            return None;
        }
        let Expr::MethodCall(m) = &*l.init.as_ref()?.expr else {
            return None;
        };
        if m.method != "split_at_mut" || m.args.len() != 1 || !simple_place(&m.receiver) {
            return None;
        }
        let ty = self.place_ty(&m.receiver);
        let base = self.place(&m.receiver);
        let k = self.expr_with(&m.args[0], &Ty::Int(crate::ty::IntKind::Usize));
        let k = self.settle(k);
        let left = self.declare(&names[0], ty.clone());
        let right = self.declare(&names[1], ty);
        let lines = vec![
            format!("var {left} = Array({base}[..<{}])", k.code),
            format!("var {right} = Array({base}[{}...])", k.code),
        ];
        Some((lines, names, format!("{base} = {left} + {right}")))
    }

    /// Runs `f` with a fresh prelude and puts the prelude's statements first.
    pub fn with_prelude(&mut self, f: impl FnOnce(&mut Self) -> Vec<String>) -> Vec<String> {
        self.preludes.push(Vec::new());
        self.postludes.push(Vec::new());
        let lines = f(self);
        let post = self.postludes.pop().unwrap_or_default();
        let mut out = self.preludes.pop().unwrap_or_default();
        out.extend(lines);
        out.extend(post);
        out
    }

    /// A function, struct or enum declared inside a function body.
    fn local_item(&mut self, item: &syn::Item) -> Vec<String> {
        match item {
            syn::Item::Fn(f) => {
                let code = self.function(&f.attrs, &f.sig, &f.block, crate::translate::Place::Free);
                vec![code]
            }
            syn::Item::Use(_) => vec![],
            syn::Item::Const(c) => {
                let ty = self.ty(&c.ty);
                let v = self.expr_with(&c.expr, &ty);
                let v = self.settle(v);
                let name = self.declare(&c.ident.to_string(), ty);
                vec![format!("let {name} = {}", v.code)]
            }
            // `static EMPTY: Cell = Cell::Empty;` in a function: a local too.
            syn::Item::Static(c) => {
                let ty = self.ty(&c.ty);
                let v = self.expr_with(&c.expr, &ty);
                let v = self.settle(v);
                let name = self.declare(&c.ident.to_string(), ty.clone());
                let annot = if v.code.starts_with('.') {
                    format!(": {}", ty.swift_lossy())
                } else {
                    String::new()
                };
                let kw = if matches!(c.mutability, syn::StaticMutability::Mut(_)) {
                    "var"
                } else {
                    "let"
                };
                vec![format!("{kw} {name}{annot} = {}", v.code)]
            }
            other => {
                let s = self.unsupported(other.span(), "an item declared inside a function");
                vec![s]
            }
        }
    }

    // ----- let -----

    fn local(&mut self, l: &Local) -> Vec<String> {
        let (pat, declared) = match &l.pat {
            Pat::Type(pt) => (&*pt.pat, Some(self.ty(&pt.ty))),
            p => (p, None),
        };
        let declared_ty = declared.clone().unwrap_or(Ty::Unknown);
        let Some(init) = &l.init else {
            // `let x;` initialized later.
            if let Pat::Ident(id) = pat {
                let rust = id.ident.to_string();
                let name = self.declare(&rust, declared_ty.clone());
                let (slot, placeholder) = self.new_slot(declared_ty);
                self.attach_slot(&rust, slot);
                let kw = if id.mutability.is_some() {
                    "var"
                } else {
                    "let"
                };
                return vec![format!("{kw} {name}{placeholder}")];
            }
            return vec![self.unsupported(l.span(), "a destructuring `let` without a value")];
        };
        if let Some((_, diverge)) = &init.diverge {
            return self.let_else(pat, &init.expr, diverge, &declared_ty);
        }
        let value_expr = &*init.expr;
        // `let mut q: VecDeque<_> = …`, `let mut q = VecDeque::new();`: a deque.
        if let Pat::Ident(id) = pat {
            let typed = matches!(&l.pat, Pat::Type(pt) if crate::symbols::mentions_deque(&pt.ty));
            let init_text = quote::quote!(#value_expr).to_string();
            if typed || init_text.contains("VecDeque") {
                self.deque_locals.insert(id.ident.to_string());
            }
        }
        match pat {
            Pat::Ident(id) if id.subpat.is_none() => {
                let rust = id.ident.to_string();
                let mutable = id.mutability.is_some();
                self.let_ident(&rust, mutable, declared, value_expr)
            }
            Pat::Wild(_) => {
                let v = self.expr(value_expr);
                let v = if v.throws.is_some() {
                    self.try_optional(v)
                } else {
                    v
                };
                vec![format!("_ = {}", v.code)]
            }
            // `let (mut count, mut first) = (0, None);`: one `let` per element, so that each
            // gets its own type (`first` from a later use).
            Pat::Tuple(t)
                if declared.is_none()
                    && matches!(value_expr, Expr::Tuple(v) if v.elems.len() == t.elems.len())
                    && t.elems
                        .iter()
                        .all(|p| matches!(p, Pat::Ident(i) if i.subpat.is_none())) =>
            {
                let Expr::Tuple(values) = value_expr else {
                    unreachable!()
                };
                let mut lines = Vec::new();
                for (p, v) in t.elems.iter().zip(&values.elems) {
                    let Pat::Ident(id) = p else { continue };
                    lines.extend(self.let_ident(
                        &id.ident.to_string(),
                        id.mutability.is_some(),
                        None,
                        v,
                    ));
                }
                lines
            }
            _ => {
                let v = self.expr_with(value_expr, &declared_ty);
                let v = self.settle(v);
                let subject = if is_simple_path(&v.code) {
                    v.code.clone()
                } else {
                    let tmp = self.fresh("value");
                    self.prelude(format!("let {tmp} = {}", v.code));
                    tmp
                };
                let ty = if declared_ty.is_unknown() {
                    v.ty.clone()
                } else {
                    declared_ty
                };
                let (lines, ok) = self.let_pattern(pat, &subject, &ty, false);
                if !ok {
                    return vec![self.unsupported(pat.span(), "this destructuring pattern")];
                }
                lines
            }
        }
    }

    fn let_ident(
        &mut self,
        rust: &str,
        mutable: bool,
        declared: Option<Ty>,
        value: &Expr,
    ) -> Vec<String> {
        // `let max = …`: inside the initializer, `max(a, b)` must already say `Swift.max`;
        // `let data = self.data…`: the field must keep its `self.`.
        let swift_name = names::lower_camel(rust);
        self.pending_locals.push(swift_name);
        self.last_closure_inouts.clear();
        let lines = self.let_ident_inner(rust, mutable, declared, value);
        self.pending_locals.pop();
        // `let flush = |word: &mut String, …| …`: calls pass those arguments `inout`.
        if matches!(value, Expr::Closure(_)) && self.last_closure_inouts.iter().any(|b| *b) {
            if let Some(local) = self.lookup(rust) {
                let flags = self.last_closure_inouts.clone();
                self.closure_inouts.insert(local.swift.clone(), flags);
            }
        }
        lines
    }

    fn let_ident_inner(
        &mut self,
        rust: &str,
        mutable: bool,
        declared: Option<Ty>,
        value: &Expr,
    ) -> Vec<String> {
        let kw = if mutable { "var" } else { "let" };
        let declared_ty = declared.clone().unwrap_or(Ty::Unknown);
        if needs_statements(value) {
            // `let x = match … { … }`: declare, then assign in every branch.
            let (slot, placeholder) = self.new_slot(declared_ty.clone());
            let simple = self.simple_branches(value);
            // A short `if c { a } else { b }`: `c ? a : b`.
            if let (true, Expr::If(i)) = (simple, value) {
                let saved = self.diags.len();
                self.preludes.push(Vec::new());
                let t = self.ternary(i, &declared_ty);
                let pre = self.preludes.pop().unwrap_or_default();
                match t {
                    Some(t) if pre.is_empty() && !t.code.contains('\n') && t.code.len() <= 80 => {
                        self.slots[slot].merge(&t.ty);
                        let name = self.declare(rust, t.ty.clone());
                        self.attach_slot(rust, slot);
                        let annot = if declared.is_some() {
                            placeholder
                        } else {
                            String::new()
                        };
                        return vec![format!("{kw} {name}{annot} = {}", t.code)];
                    }
                    _ => self.diags.truncate(saved),
                }
            }
            if simple {
                let code = self.branch_expression(value, &declared_ty);
                if let Some((code, ty)) = code {
                    self.slots[slot].merge(&ty);
                    let name = self.declare(rust, ty);
                    self.attach_slot(rust, slot);
                    let annot = if declared.is_some() {
                        placeholder
                    } else {
                        String::new()
                    };
                    return vec![format!("{kw} {name}{annot} = {code}")];
                }
            }
            let name = self.declare(rust, declared_ty.clone());
            self.attach_slot(rust, slot);
            self.assign_slots.insert(name.clone(), slot);
            let lines = self.tail(value, &Sink::Assign(name.clone()));
            self.assign_slots.remove(&name);
            let mut out = vec![format!("{kw} {name}{placeholder}")];
            out.extend(lines);
            if self.slots[slot].swift().is_none() {
                self.error(
                    value.span(),
                    "add a type annotation to this `let`: its value is computed by statements",
                );
            }
            return out;
        }
        // `let mut cur = &mut self.head;` re-pointed later (`cur = &mut node.next`): a
        // cursor on the links between class nodes, holding the node and the field it is
        // in, the root link being the place it started from.
        if let Expr::Reference(re) = value {
            if re.mutability.is_some()
                && mutable
                && simple_place(&re.expr)
                && self.reassigned_lets.contains(rust)
            {
                let ty = self.place_ty(&re.expr);
                if let Ty::Option(inner) = &ty {
                    if self.is_recursive_class(inner) {
                        let root = self.place(&re.expr);
                        let node = inner.swift_lossy();
                        let name = self.declare_link(rust, root, ty.clone());
                        self.helper("link");
                        return vec![format!("var {name} = RsLink<{node}>()")];
                    }
                }
            }
        }
        // `let r = &mut inv;`: `r` stands for `inv`.
        if let Expr::Reference(re) = value {
            if re.mutability.is_some() && simple_place(&re.expr) {
                let ty = self.place_ty(&re.expr);
                let place = self.place(&re.expr);
                self.declare_alias(rust, place, ty);
                return vec![];
            }
        }
        // `let item = inv.add("x", 1);` with a method returning `&mut T`: `item` stands for
        // the element, through the key path.
        if let Expr::MethodCall(m) = value {
            if let Some(sig) = self.key_path_sig(m) {
                if !matches!(sig.ret, Ty::Option(_) | Ty::Result(..)) {
                    if let Some((call, root, _)) = self.raw_key_path_call(m) {
                        let p = self.fresh("path");
                        self.declare_alias(rust, format!("{root}[keyPath: {p}]"), sig.ret.clone());
                        return vec![format!("let {p} = {call}")];
                    }
                }
            }
        }
        // `let mut n = cell.borrow_mut();`: `n` stands for `cell.value`.
        if let Expr::MethodCall(m) = value {
            if let Some((place, ty)) = self.cell_place(m) {
                self.declare_alias(rust, place, ty);
                return vec![];
            }
        }
        // `let v = map.entry(k).or_insert(0);`: `v` stands for `map[k, default: 0]`.
        if let Expr::MethodCall(m) = value {
            let ty = self.entry_value_ty(m);
            if let Some(place) = self.entry_place(m) {
                self.declare_alias(rust, place, ty);
                return vec![];
            }
            // `let v = opt.get_or_insert(0);`: `v` stands for `opt!`.
            let method = m.method.to_string();
            if matches!(method.as_str(), "get_or_insert" | "get_or_insert_with")
                && m.args.len() == 1
            {
                if let Ty::Option(inner) = self.place_ty(&m.receiver) {
                    let place = self.place(&m.receiver);
                    let v = match (&m.args[0], method.as_str()) {
                        (Expr::Closure(c), "get_or_insert_with") if c.inputs.is_empty() => {
                            self.expr_with(&c.body, &inner)
                        }
                        (e, "get_or_insert") => self.expr_with(e, &inner),
                        (e, _) => {
                            let f = self.expr(e);
                            Ex::primary(format!("({})()", f.code), (*inner).clone())
                        }
                    };
                    let v = self.settle(v);
                    self.declare_alias(rust, format!("{place}!"), *inner);
                    return vec![format!("if {place} == nil {{ {place} = {} }}", v.code)];
                }
            }
        }
        // `let t = v.last_mut().unwrap();`: `t` stands for the element itself.
        if let Expr::MethodCall(m) = value {
            if matches!(m.method.to_string().as_str(), "unwrap" | "expect") {
                if let Expr::MethodCall(inner) = &*m.receiver {
                    if let Some((place, ty)) = self.mutable_place_forced(inner) {
                        self.declare_alias(rust, place, ty);
                        return vec![];
                    }
                }
            }
        }
        // `let x = opt?;` → `guard let x = opt else { return nil }`.
        if let Expr::Try(t) = value {
            if let Some(lines) = self.let_mutable_try(rust, &t.expr) {
                return lines;
            }
            if let Some(lines) = self.let_try(rust, mutable, &declared_ty, &t.expr) {
                return lines;
            }
        }
        let v = self.expr_with(value, &declared_ty);
        let mut v = self.settle_for_let(v);
        // `let mut it = s.chars();` and then `it.next()`: a Rust iterator with its position.
        if mutable && declared.is_none() {
            let iterator_chain = matches!(value, Expr::MethodCall(m) if matches!(
                m.method.to_string().as_str(),
                "iter" | "into_iter" | "chars" | "bytes" | "char_indices" | "lines" | "keys"
                    | "values" | "peekable" | "enumerate" | "rev" | "map" | "filter"
                    | "filter_map" | "flat_map" | "zip" | "chain" | "skip" | "take" | "step_by"
                    | "cycle" | "split" | "split_whitespace" | "windows" | "chunks" | "drain"
            ));
            match &v.ty {
                Ty::Iter(item) => {
                    self.helper("iter");
                    v = Ex::primary(
                        format!("RsIter({})", v.code),
                        Ty::Named("RsIter".into(), vec![(**item).clone()]),
                    );
                }
                Ty::Vec(item) | Ty::Set(item, _) if iterator_chain => {
                    self.helper("iter");
                    v = Ex::primary(
                        format!("RsIter({})", v.code),
                        Ty::Named("RsIter".into(), vec![(**item).clone()]),
                    );
                }
                _ => {}
            }
        }
        let mut ty = declared_ty.clone();
        ty.merge(&v.ty);
        // Empty collections and `nil` need their type written.
        let needs_annotation = declared
            .as_ref()
            .is_some_and(|d| !crate::translate::natural_type_matches(&v.ty, d))
            || matches!(v.code.as_str(), "[]" | "[:]" | "nil")
            || v.code.starts_with("Set()");
        let name = self.declare(rust, ty.clone());
        if needs_annotation || !ty.is_complete() {
            // Completed by later uses (`v.push(1)`), or left out if still unknown.
            let (slot, placeholder) = self.new_slot(ty);
            self.attach_slot(rust, slot);
            return vec![format!("{kw} {name}{placeholder} = {}", v.code)];
        }
        vec![format!("{kw} {name} = {}", v.code)]
    }

    /// `let x = e?;` when `e` is an optional: a `guard let`.
    fn let_try(
        &mut self,
        rust: &str,
        mutable: bool,
        declared: &Ty,
        inner: &Expr,
    ) -> Option<Vec<String>> {
        let expected = if declared.is_unknown() {
            Ty::Unknown
        } else {
            Ty::Option(Box::new(declared.clone()))
        };
        // Translate in a scratch prelude: a throwing call or a Result goes the usual way.
        self.preludes.push(Vec::new());
        self.no_trailing += 1;
        let saved_diags = self.diags.len();
        let v = self.expr_with(inner, &expected);
        self.no_trailing -= 1;
        let pre = self.preludes.pop().unwrap_or_default();
        if v.throws.is_some() || v.ty.is_result() || v.ty == Ty::Unit {
            self.diags.truncate(saved_diags);
            return None;
        }
        for p in pre {
            self.prelude(p);
        }
        let item = v.ty.item();
        let throws = self.fn_cx().and_then(|f| f.throws.clone());
        let exit = if throws.is_some() {
            self.helper("ParseError");
            "throw RsParseError()".to_string()
        } else if self.fn_cx().is_some_and(|f| f.ret == Ty::Unit) {
            "return".to_string()
        } else {
            "return nil".to_string()
        };
        let mut ty = declared.clone();
        ty.merge(&item);
        let name = self.declare(rust, ty);
        let kw = if mutable { "var" } else { "let" };
        Some(vec![format!(
            "guard {kw} {name} = {} else {{ {exit} }}",
            crate::expr::condition_value(v.code)
        )])
    }

    /// Whether every branch of an `if`/`match` value is a single expression without control
    /// flow: then Swift's `if`/`switch` expressions can hold it.
    fn simple_branches(&self, e: &Expr) -> bool {
        match e {
            Expr::If(i) => {
                if contains_let(&i.cond) {
                    return false;
                }
                let then_ok = block_is_single_expr(&i.then_branch);
                let else_ok = match &i.else_branch {
                    Some((_, e)) => match &**e {
                        Expr::Block(b) => block_is_single_expr(&b.block),
                        e @ Expr::If(_) => self.simple_branches(e),
                        _ => false,
                    },
                    None => false,
                };
                then_ok && else_ok
            }
            Expr::Match(m) => m.arms.iter().all(|a| match &*a.body {
                Expr::Block(b) => block_is_single_expr(&b.block),
                body => !has_control_flow(body) && !needs_statements(body),
            }),
            _ => false,
        }
    }

    /// An `if`/`switch` expression for a value whose branches are simple.
    fn branch_expression(&mut self, e: &Expr, expected: &Ty) -> Option<(String, Ty)> {
        let lines = self.tail(e, &Sink::Yield);
        let ty = self.last_branch_ty.take().unwrap_or(Ty::Unknown);
        let mut ty2 = expected.clone();
        ty2.merge(&ty);
        Some((lines.join("\n"), ty2))
    }

    /// `let Some(x) = opt else { return };` → `guard let x = opt else { return }`.
    fn let_else(&mut self, pat: &Pat, value: &Expr, diverge: &Expr, declared: &Ty) -> Vec<String> {
        // `let Some(t) = v.get_mut(i) else { … }`: `t` stands for the element itself.
        if let Some(cond) = self.mutable_binding(pat, value) {
            let else_lines = match diverge {
                Expr::Block(b) => self.block_lines(&b.block, Sink::Discard),
                other => self.expr_stmt(other),
            };
            return vec![format!(
                "guard {cond} else {{\n{}\n}}",
                indent_lines(&else_lines.join("\n"))
            )];
        }
        self.no_trailing += 1;
        let v = self.expr_with(value, declared);
        self.no_trailing -= 1;
        let else_lines = match diverge {
            Expr::Block(b) => self.block_lines(&b.block, Sink::Discard),
            other => self.expr_stmt(other),
        };
        let cond = self.pattern_condition(pat, v);
        vec![format!(
            "guard {cond} else {{\n{}\n}}",
            indent_lines(&else_lines.join("\n"))
        )]
    }

    /// Destructures `subject` (a Swift expression) with a `let` pattern.
    pub fn let_pattern(
        &mut self,
        pat: &Pat,
        subject: &str,
        ty: &Ty,
        mutable: bool,
    ) -> (Vec<String>, bool) {
        let kw = if mutable { "var" } else { "let" };
        match pat {
            Pat::Ident(id) => {
                let n = self.declare(&id.ident.to_string(), ty.clone());
                let kw = if id.mutability.is_some() { "var" } else { kw };
                (vec![format!("{kw} {n} = {subject}")], true)
            }
            Pat::Wild(_) => (vec![], true),
            Pat::Type(pt) => {
                let t = self.ty(&pt.ty);
                self.let_pattern(&pt.pat, subject, &t, mutable)
            }
            Pat::Reference(r) => self.let_pattern(&r.pat, subject, ty, mutable),
            Pat::Paren(p) => self.let_pattern(&p.pat, subject, ty, mutable),
            Pat::Tuple(t) => {
                let elems: Vec<Ty> = match ty {
                    Ty::Tuple(ts) => ts.clone(),
                    _ => vec![Ty::Unknown; t.elems.len()],
                };
                // Simple tuples of names: `let (a, b) = t`.
                let all_idents = t.elems.iter().all(|p| {
                    matches!(p, Pat::Ident(i) if i.subpat.is_none()) || matches!(p, Pat::Wild(_))
                });
                // `let (ref mut a, ref mut b) = pair`: `a` and `b` are the fields of `pair`.
                let ref_mut = |p: &Pat| matches!(p, Pat::Ident(i) if i.by_ref.is_some() && i.mutability.is_some());
                let place = subject
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.');
                if all_idents && place && t.elems.iter().any(ref_mut) {
                    let mut lines = Vec::new();
                    for (i, p) in t.elems.iter().enumerate() {
                        let Pat::Ident(id) = p else { continue };
                        let elem = elems.get(i).cloned().unwrap_or(Ty::Unknown);
                        let member = format!("{subject}.{i}");
                        if ref_mut(p) {
                            self.declare_alias(&id.ident.to_string(), member, elem);
                        } else {
                            let n = self.declare(&id.ident.to_string(), elem);
                            lines.push(format!("let {n} = {member}"));
                        }
                    }
                    return (lines, true);
                }
                if all_idents {
                    let any_mut = t
                        .elems
                        .iter()
                        .any(|p| matches!(p, Pat::Ident(i) if i.mutability.is_some()));
                    let names: Vec<String> = t
                        .elems
                        .iter()
                        .enumerate()
                        .map(|(i, p)| match p {
                            Pat::Ident(id) => self.declare(
                                &id.ident.to_string(),
                                elems.get(i).cloned().unwrap_or(Ty::Unknown),
                            ),
                            _ => "_".into(),
                        })
                        .collect();
                    let kw = if any_mut || mutable { "var" } else { "let" };
                    // `let (x,) = t`: the field of the support struct.
                    if names.len() == 1 {
                        return (vec![format!("{kw} {} = {subject}._0", names[0])], true);
                    }
                    return (
                        vec![format!("{kw} ({}) = {subject}", names.join(", "))],
                        true,
                    );
                }
                let mut lines = Vec::new();
                let mut ok = true;
                for (i, p) in t.elems.iter().enumerate() {
                    let member = if t.elems.len() == 1 {
                        "_0".to_string()
                    } else {
                        i.to_string()
                    };
                    let (l, o) = self.let_pattern(
                        p,
                        &format!("{subject}.{member}"),
                        elems.get(i).unwrap_or(&Ty::Unknown),
                        mutable,
                    );
                    lines.extend(l);
                    ok &= o;
                }
                (lines, ok)
            }
            Pat::Struct(s) => {
                let name = s
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                let info = self.syms.structs.get(&name).cloned();
                let mut lines = Vec::new();
                let mut ok = true;
                for f in &s.fields {
                    let member = match &f.member {
                        syn::Member::Named(n) => n.to_string(),
                        syn::Member::Unnamed(i) => i.index.to_string(),
                    };
                    let field = info.as_ref().and_then(|i| i.field(&member));
                    let swift =
                        field.map_or_else(|| names::lower_camel(&member), |f| f.swift.clone());
                    let fty = field.map_or(Ty::Unknown, |f| f.ty.clone());
                    let (l, o) =
                        self.let_pattern(&f.pat, &format!("{subject}.{swift}"), &fty, mutable);
                    lines.extend(l);
                    ok &= o;
                }
                (lines, ok)
            }
            Pat::TupleStruct(ts) => {
                let name = ts
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                // `let Reverse(x) = heap.pop().unwrap();`
                if name == "Reverse" && ts.elems.len() == 1 {
                    let inner = match ty {
                        Ty::Named(n, args) if n == "RsReverse" => {
                            args.first().cloned().unwrap_or(Ty::Unknown)
                        }
                        _ => Ty::Unknown,
                    };
                    let field = if crate::ty::ord_tuple_arity(&inner).is_some() {
                        "value.tuple"
                    } else {
                        "value"
                    };
                    return self.let_pattern(
                        &ts.elems[0],
                        &format!("{subject}.{field}"),
                        &inner,
                        mutable,
                    );
                }
                let Some(info) = self.syms.structs.get(&name).cloned() else {
                    return (vec![], false);
                };
                let mut lines = Vec::new();
                let mut ok = true;
                for (i, p) in ts.elems.iter().enumerate() {
                    let Some(field) = info.fields.get(i) else {
                        break;
                    };
                    let (l, o) = self.let_pattern(
                        p,
                        &format!("{subject}.{}", field.swift),
                        &field.ty,
                        mutable,
                    );
                    lines.extend(l);
                    ok &= o;
                }
                (lines, ok)
            }
            Pat::Slice(s) => {
                let item = ty.item();
                let mut lines = Vec::new();
                let mut ok = true;
                let n = s.elems.len();
                let rest_at = s.elems.iter().position(|p| matches!(p, Pat::Rest(_)) || matches!(p, Pat::Ident(i) if matches!(i.subpat.as_ref().map(|(_, p)| &**p), Some(Pat::Rest(_)))));
                for (i, p) in s.elems.iter().enumerate() {
                    if Some(i) == rest_at {
                        continue;
                    }
                    let index = match rest_at {
                        Some(r) if i > r => format!("{subject}[{subject}.count - {}]", n - i),
                        _ => format!("{subject}[{i}]"),
                    };
                    let (l, o) = self.let_pattern(p, &index, &item, mutable);
                    lines.extend(l);
                    ok &= o;
                }
                (lines, ok)
            }
            _ => (vec![], false),
        }
    }

    // ----- Expression statements and tails -----

    /// An expression evaluated for its effect.
    pub fn expr_stmt(&mut self, e: &Expr) -> Vec<String> {
        // `Ok(())` on its own line: nothing happens.
        if let Expr::Call(c) = e {
            if let Expr::Path(p) = &*c.func {
                if p.path.is_ident("Ok")
                    && c.args.len() == 1
                    && matches!(c.args[0], Expr::Tuple(ref t) if t.elems.is_empty())
                {
                    return vec![];
                }
            }
        }
        // `acct.deposit(100).deposit(-30)`: builder methods, one statement each.
        if let Some(lines) = self.builder_chain(e) {
            return lines;
        }
        // `map.entry(k).or_insert(v);` on its own: the entry is inserted when missing.
        if let Expr::MethodCall(m) = e {
            if let Some(code) = self.entry_place(m) {
                if let Some((map, key, default)) = crate::keypath::entry_parts(&code) {
                    return vec![format!(
                        "if {map}[{key}] == nil {{ {map}[{key}] = {default} }}"
                    )];
                }
            }
        }
        // `self.cache.as_mut().map(|c| c.push(x));`: the element changed in place, when it
        // exists.
        if let Some(lines) = self.option_map_stmt(e) {
            return lines;
        }
        // `cur = &mut node.next;`, `*cur = value;` on a cursor local.
        if let Expr::Assign(a) = e {
            if let Some(lines) = self.link_assign(a) {
                return lines;
            }
        }
        // `opt.get_or_insert_with(f);` on its own: only the insertion, written as a prelude.
        if let Expr::MethodCall(m) = e {
            if matches!(
                m.method.to_string().as_str(),
                "get_or_insert" | "get_or_insert_with"
            ) && m.args.len() == 1
                && matches!(self.place_ty(&m.receiver), Ty::Option(_))
            {
                let v = self.expr(e);
                if v.code.ends_with('!') {
                    return vec![];
                }
                return vec![v.code];
            }
        }
        // `bank.open("ann", 1);`: the key path returned is not needed.
        if let Expr::MethodCall(m) = e {
            if let Some((call, _, sig)) = self.raw_key_path_call(m) {
                let t = if matches!(sig.ret, Ty::Result(..)) {
                    "try "
                } else {
                    ""
                };
                return vec![format!("_ = {t}{call}")];
            }
        }
        match e {
            Expr::If(_) | Expr::Match(_) | Expr::Block(_) | Expr::Unsafe(_) => {
                self.tail(e, &Sink::Discard)
            }
            Expr::ForLoop(f) => self.for_loop(f),
            Expr::While(w) => self.while_loop(w),
            Expr::Loop(l) => self.loop_stmt(l),
            Expr::Return(r) => self.return_stmt(r.expr.as_deref()),
            Expr::Break(b) => {
                let jump = match &b.label {
                    Some(l) => format!("break {}", l.ident),
                    // Inside a `switch`, a plain `break` would leave the switch, not the loop.
                    None if self.switch_depth > 0 => match self.loops.last().cloned().flatten() {
                        Some(l) => {
                            self.used_labels.insert(l.clone());
                            format!("break {l}")
                        }
                        None => "break".into(),
                    },
                    None => "break".into(),
                };
                let Some(value) = &b.expr else {
                    return vec![jump];
                };
                // `break value` out of `let x = loop { … }`: assign, then leave.
                let target = match &b.label {
                    Some(l) => self
                        .loops
                        .iter()
                        .rposition(|n| n.as_deref() == Some(&l.ident.to_string())),
                    None => self.loops.len().checked_sub(1),
                };
                let var = target.and_then(|i| self.loop_values.get(i).cloned().flatten());
                match var {
                    Some(rust) => {
                        let mut lines = self.tail(value, &Sink::Assign(rust));
                        lines.push(jump);
                        lines
                    }
                    None => vec![self.unsupported(
                        b.span(),
                        "`break` with a value outside `let x = loop { … }`",
                    )],
                }
            }
            Expr::Continue(c) => vec![match &c.label {
                Some(l) => format!("continue {}", l.ident),
                None => "continue".into(),
            }],
            Expr::Macro(m) => self.macro_stmt(&m.mac),
            Expr::Paren(p) => self.expr_stmt(&p.expr),
            _ => {
                if let Some(lines) = self.for_each_loop(e) {
                    return lines;
                }
                if let Some(lines) = self.special_stmt(e) {
                    return lines;
                }
                let v = self.expr(e);
                if v.throws.is_some() {
                    let v = self.try_optional(v);
                    return vec![format!("_ = {}", v.code)];
                }
                if v.code.is_empty() {
                    return vec![];
                }
                let discard = matches!(e, Expr::Call(_) | Expr::MethodCall(_))
                    && !matches!(v.ty, Ty::Unit | Ty::Never | Ty::Unknown | Ty::View)
                    && !v.code.starts_with("try ")
                    && user_call_returns_value(self, e);
                if discard {
                    vec![format!("_ = {}", v.code)]
                } else {
                    vec![v.code]
                }
            }
        }
    }

    /// `cur = &mut node.next;` (`cur = if c { &mut n.left } else { &mut n.right };`) and
    /// `*cur = value;` on a cursor local: the cursor moved, or the link written.
    fn link_assign(&mut self, a: &syn::ExprAssign) -> Option<Vec<String>> {
        let link_of = |t: &Self, e: &Expr| -> Option<(String, String, Ty)> {
            let Expr::Path(p) = e else { return None };
            let id = p.path.get_ident()?.to_string();
            let local = t.lookup(&id)?;
            let root = local.link_root.clone()?;
            Some((local.swift.clone(), root, t.local_ty(local)))
        };
        if let Expr::Unary(u) = &*a.left {
            if matches!(u.op, syn::UnOp::Deref(_)) {
                let (swift, root, ty) = link_of(self, &u.expr)?;
                let v = self.expr_with(&a.right, &ty);
                let v = self.settle(v);
                return Some(vec![format!("{swift}.set(&{root}, {})", v.code)]);
            }
        }
        let (swift, _, _) = link_of(self, &a.left)?;
        self.link_move(&swift, &a.right)
    }

    /// `cur.move(node, \.next)` for `&mut node.next`, through the branches of an `if`.
    fn link_move(&mut self, swift: &str, target: &Expr) -> Option<Vec<String>> {
        match target {
            Expr::Reference(r) if r.mutability.is_some() => {
                let Expr::Field(f) = &*r.expr else {
                    return None;
                };
                let base = self.expr(&f.base);
                let base = self.settle(base);
                let field = match &f.member {
                    syn::Member::Named(id) => {
                        let base_ty = base.ty.clone();
                        match &base_ty {
                            Ty::Named(n, _) => self
                                .struct_by_swift(n)
                                .and_then(|s| s.field(&id.to_string()).map(|f| f.swift.clone()))
                                .unwrap_or_else(|| names::lower_camel(&id.to_string())),
                            _ => names::lower_camel(&id.to_string()),
                        }
                    }
                    syn::Member::Unnamed(i) => format!("_{}", i.index),
                };
                Some(vec![format!(
                    "{swift}.move({}, \\.{field})",
                    crate::expr::wrap(&base, P_PRIMARY)
                )])
            }
            Expr::Paren(p) => self.link_move(swift, &p.expr),
            Expr::Block(b) if b.block.stmts.len() == 1 => match &b.block.stmts[0] {
                Stmt::Expr(e, None) => self.link_move(swift, e),
                _ => None,
            },
            Expr::If(i) => {
                let cond = self.expr_with(&i.cond, &Ty::Bool);
                let cond = self.settle(cond);
                let then_e = match i.then_branch.stmts.as_slice() {
                    [Stmt::Expr(e, None)] => e,
                    _ => return None,
                };
                let then_lines = self.link_move(swift, then_e)?;
                let (_, else_e) = i.else_branch.as_ref()?;
                let else_lines = self.link_move(swift, else_e)?;
                let mut out = vec![format!("if {} {{", cond.code)];
                out.extend(then_lines.iter().map(|l| format!("{INDENT}{l}")));
                out.push("} else {".into());
                out.extend(else_lines.iter().map(|l| format!("{INDENT}{l}")));
                out.push("}".into());
                Some(out)
            }
            _ => None,
        }
    }

    /// The last expression of a block, delivered to `sink`.
    pub fn tail(&mut self, e: &Expr, sink: &Sink) -> Vec<String> {
        match e {
            Expr::If(i) => self.if_stmt(i, sink),
            Expr::Match(m) => self.match_stmt(m, sink),
            Expr::Block(b) if b.label.is_none() => {
                let lines = self.block_lines(&b.block, sink.clone());
                if *sink == Sink::Yield
                    || *sink == Sink::Discard
                        && lines
                            .iter()
                            .all(|l| !l.starts_with("let ") && !l.starts_with("var "))
                {
                    lines
                } else {
                    vec![format!("do {{\n{}\n}}", indent_lines(&lines.join("\n")))]
                }
            }
            // `'a: { … break 'a value; … }` → `a: do { … }`.
            Expr::Block(b) => {
                let label = b
                    .label
                    .as_ref()
                    .map(|l| l.name.ident.to_string())
                    .unwrap_or_default();
                let value = match sink {
                    Sink::Assign(name) => Some(name.clone()),
                    _ => None,
                };
                self.loops.push(Some(label.clone()));
                self.loop_values.resize(self.loops.len() - 1, None);
                self.loop_values.push(value);
                let lines = self.block_lines(&b.block, sink.clone());
                self.loops.pop();
                self.loop_values.truncate(self.loops.len());
                vec![format!(
                    "{label}: do {{\n{}\n}}",
                    indent_lines(&lines.join("\n"))
                )]
            }
            Expr::Unsafe(u) => {
                self.warn(
                    u.unsafe_token.span,
                    "unsafe code is translated as ordinary code: check it",
                );
                self.block_lines(&u.block, sink.clone())
            }
            Expr::Paren(p) => self.tail(&p.expr, sink),
            Expr::Loop(l) => {
                let value = match sink {
                    Sink::Assign(name) => Some(name.clone()),
                    _ => None,
                };
                self.loop_with(l, value)
            }
            Expr::ForLoop(_)
            | Expr::While(_)
            | Expr::Return(_)
            | Expr::Break(_)
            | Expr::Continue(_) => self.expr_stmt(e),
            Expr::Macro(m) if crate::macros::is_statement_macro(&m.mac) => self.macro_stmt(&m.mac),
            _ => match sink {
                Sink::Discard => self.expr_stmt(e),
                Sink::Return => self.return_stmt(Some(e)),
                Sink::InitSelf => self.init_value(e),
                // The target is a Swift name, with the slot of its type: a binding of the
                // branch may have the same Rust name.
                Sink::Assign(swift) => {
                    let slot = self.assign_slots.get(swift).copied();
                    let expected = slot.map_or(Ty::Unknown, |s| self.slots[s].clone());
                    let v = self.expr_with(e, &expected);
                    let v = self.settle(v);
                    if let Some(slot) = slot {
                        self.slots[slot].merge(&v.ty);
                    }
                    vec![format!("{swift} = {}", v.code)]
                }
                Sink::Yield => {
                    if self.in_view_builder() {
                        return self.view_lines(e);
                    }
                    let expected = Ty::Unknown;
                    let v = self.expr_with(e, &expected);
                    let v = self.settle(v);
                    let mut t = self.last_branch_ty.take().unwrap_or(Ty::Unknown);
                    t.merge(&v.ty);
                    self.last_branch_ty = Some(t);
                    vec![v.code]
                }
            },
        }
    }

    /// Whether `Sink::Yield` means a view of a result builder.
    pub fn in_view_builder(&self) -> bool {
        self.fn_cx()
            .is_some_and(|f| f.ret == Ty::View && !f.closure)
            || self.view_closure_depth > 0
    }

    /// `return e`, with `Ok`/`Err`/`Some`/`None` mapped to the Swift function's conventions.
    /// `a.b(…).c(…)` where `b` returns `&mut Self`: `a.b(…)` then `a.c(…)`.
    fn builder_chain(&mut self, e: &Expr) -> Option<Vec<String>> {
        let Expr::MethodCall(outer) = e else {
            return None;
        };
        let Expr::MethodCall(inner) = &*outer.receiver else {
            return None;
        };
        let root = crate::expr::root_receiver(e);
        let root_ty = self.place_ty(root);
        let is_builder = |t: &mut Self, name: &str| {
            t.user_method(&root_ty, name)
                .is_some_and(|(sig, _)| sig.returns_self_mut)
        };
        if !is_builder(self, &inner.method.to_string()) {
            return None;
        }
        // The calls, from the root outwards.
        let mut calls: Vec<&syn::ExprMethodCall> = vec![outer];
        let mut cur = inner;
        loop {
            calls.push(cur);
            match &*cur.receiver {
                Expr::MethodCall(m) if is_builder(self, &m.method.to_string()) => cur = m,
                _ => break,
            }
        }
        calls.reverse();
        let mut lines = Vec::new();
        for call in calls {
            let mut one = call.clone();
            *one.receiver = root.clone();
            let v = self.method_call(&one, &Ty::Unknown);
            let v = self.settle(v);
            lines.push(v.code);
        }
        Some(lines)
    }

    pub fn return_stmt(&mut self, e: Option<&Expr>) -> Vec<String> {
        let Some(cx) = self.fn_cx().cloned() else {
            return vec!["return".into()];
        };
        // `return self` of a builder method returning `&mut Self`: nothing to return.
        if cx.ret == Ty::Unit && e.is_some_and(crate::expr::is_self) {
            return vec!["return".into()];
        }
        if cx.init {
            return match e {
                Some(e) => {
                    let mut lines = self.init_value(e);
                    if !lines
                        .last()
                        .is_some_and(|l| l.starts_with("throw ") || l.starts_with("return"))
                    {
                        lines.push("return".into());
                    }
                    lines
                }
                None => vec!["return".into()],
            };
        }
        let Some(e) = e else {
            return vec!["return".into()];
        };
        // `fn get(&mut self, …) -> &mut T`: the key path to the value.
        if let Some(root) = cx.key_path.clone() {
            return self.key_path_return(e, &root, &cx);
        }
        // In a function returning a `Result`.
        if let Some(err) = &cx.throws {
            let ok_ty = match &cx.ret {
                Ty::Result(ok, _) => (**ok).clone(),
                _ => Ty::Unknown,
            };
            if let Some((ctor, arg)) = result_ctor(e) {
                if ctor == "Err" {
                    let v = self.expr_with(arg, err);
                    let v = self.settle(v);
                    return vec![format!("throw {}", v.code)];
                }
                if matches!(arg, Expr::Tuple(t) if t.elems.is_empty()) || ok_ty == Ty::Unit {
                    let v = self.expr_with(arg, &ok_ty);
                    if v.code == "()" || v.code.is_empty() {
                        return vec!["return".into()];
                    }
                    return vec![v.code, "return".into()];
                }
                let v = self.expr_with(arg, &ok_ty);
                let v = self.settle(v);
                return vec![format!("return {}", v.code)];
            }
            let v = self.expr_with(e, &cx.ret);
            if v.throws.is_some() {
                // A closure collected into a `Result<_, _>`: the call it returns says the
                // error type (`throws(VmError)`, not `throws`).
                if cx.closure && matches!(cx.throws, Some(Ty::Unknown)) {
                    if let (Some(t), Some(f)) = (&v.throws, self.fns.last_mut()) {
                        if t.1.is_complete() {
                            f.throws = Some(t.1.clone());
                        }
                    }
                }
                let code = format!("try {}", v.code);
                return if ok_ty == Ty::Unit {
                    vec![code, "return".into()]
                } else {
                    vec![format!("return {code}")]
                };
            }
            if v.ty.is_result() {
                let v = self.settle(v);
                let code = format!("try {}.get()", crate::expr::wrap(&v, P_PRIMARY));
                return if ok_ty == Ty::Unit {
                    vec![code, "return".into()]
                } else {
                    vec![format!("return {code}")]
                };
            }
            let v = self.settle(v);
            if v.ty == Ty::Never {
                return vec![v.code];
            }
            return vec![format!("return {}", v.code)];
        }
        let v = self.expr_with(e, &cx.ret);
        let v = self.settle(v);
        // `std::process::exit(3)` as the value: nothing comes back to return.
        if v.ty == Ty::Never {
            return vec![v.code];
        }
        if cx.closure && !cx.ret.is_complete() {
            if let Some(f) = self.fns.last_mut() {
                f.ret.merge(&v.ty);
            }
        }
        if cx.ret == Ty::Unit && cx.closure {
            return vec![v.code];
        }
        if cx.ret == Ty::Unit || cx.ret == Ty::Never {
            if v.code.is_empty() || v.code == "()" {
                return vec!["return".into()];
            }
            return vec![v.code, "return".into()];
        }
        if matches!(cx.ret, Ty::Iter(_)) && !v.code.starts_with('[') {
            return vec![format!("return Array({})", v.code)];
        }
        vec![format!("return {}", v.code)]
    }

    /// The value of an initializer: `self = value`, `throw e`, `return nil`.
    fn init_value(&mut self, e: &Expr) -> Vec<String> {
        let ret = self.fn_cx().map(|f| f.ret.clone()).unwrap_or(Ty::Unknown);
        if let Some((ctor, arg)) = result_ctor(e) {
            match ctor {
                "Err" => {
                    let v = self.expr(arg);
                    return vec![format!("throw {}", v.code)];
                }
                "Ok" | "Some" => {
                    let inner = match &ret {
                        Ty::Option(t) | Ty::Result(t, _) => (**t).clone(),
                        t => t.clone(),
                    };
                    let v = self.expr_with(arg, &inner);
                    let v = self.settle(v);
                    return vec![format!("self = {}", v.code)];
                }
                _ => {}
            }
        }
        if let Expr::Path(p) = e {
            if p.path.is_ident("None") {
                return vec!["return nil".into()];
            }
        }
        let inner = match &ret {
            Ty::Option(t) | Ty::Result(t, _) => (**t).clone(),
            t => t.clone(),
        };
        let v = self.expr_with(e, &inner);
        if v.throws.is_some() {
            return vec![format!("self = try {}", v.code)];
        }
        let v = self.settle(v);
        vec![format!("self = {}", v.code)]
    }

    // ----- if / match -----

    fn if_stmt(&mut self, i: &syn::ExprIf, sink: &Sink) -> Vec<String> {
        let mut out = String::new();
        self.push_scope();
        let cond = self.condition(&i.cond);
        let then = self.block_lines(&i.then_branch, sink.clone());
        self.pop_scope();
        out.push_str(&format!(
            "if {cond} {{\n{}\n}}",
            indent_lines(&join_or_break(then, sink, self.in_view_builder()))
        ));
        if let Some((_, e)) = &i.else_branch {
            match &**e {
                Expr::If(inner) => {
                    let rest = self.if_stmt(inner, sink);
                    out.push_str(" else ");
                    out.push_str(&rest.join("\n"));
                }
                Expr::Block(b) => {
                    let lines = self.block_lines(&b.block, sink.clone());
                    // `else { None }` in a view: nothing to show.
                    let skip = lines.is_empty() && *sink == Sink::Yield && self.in_view_builder();
                    if !skip {
                        out.push_str(&format!(
                            " else {{\n{}\n}}",
                            indent_lines(&join_or_break(lines, sink, self.in_view_builder()))
                        ));
                    }
                }
                other => {
                    let lines = self.tail(other, sink);
                    out.push_str(&format!(
                        " else {{\n{}\n}}",
                        indent_lines(&lines.join("\n"))
                    ));
                }
            }
        }
        vec![out]
    }

    fn match_stmt(&mut self, m: &syn::ExprMatch, sink: &Sink) -> Vec<String> {
        if let Some(lines) = self.match_mutable(m, sink) {
            return lines;
        }
        self.no_trailing += 1;
        let subject = self.expr(&m.expr);
        self.no_trailing -= 1;
        let subject = self.settle(subject);
        let subject_code = subject.code.clone();
        let subject_ty = subject.ty.clone();
        let mut cases = Vec::new();
        let n = m.arms.len();
        let mut prev = m.expr.span().end().line;
        self.switch_depth += 1;
        for (i, arm) in m.arms.iter().enumerate() {
            let leading = self.comments.take_between(prev, arm.span().start().line);
            prev = arm.span().end().line;
            self.push_scope();
            // `Shape::Circle { r } => *r *= k` on a place: the variant is built again from
            // its changed fields and stored back. Not into a `self` taken by value: the
            // changes are the new value's own.
            let consumed_self = subject_code == "self" && !self.self_mutable;
            // `match map.get_mut(k)`: the subject is the place in the map.
            let lookup = matches!(&*m.expr, Expr::MethodCall(c)
                if matches!(c.method.to_string().as_str(), "get_mut" | "as_mut" | "last_mut" | "first_mut" | "back_mut" | "front_mut")
                    && simple_place(&c.receiver));
            let write_back = if (crate::expr::is_place(&m.expr) || lookup)
                && arm.guard.is_none()
                && !consumed_self
            {
                self.variant_write_back(&arm.pat, &arm.body, &subject_ty, &subject_code)
            } else {
                None
            };
            let pats = match &write_back {
                Some(wb) => crate::pat::CasePats {
                    code: wb.0.clone(),
                    ..Default::default()
                },
                None => self.case_patterns(&arm.pat, &subject_ty, &subject_code),
            };
            let mut guards = pats.guards.clone();
            if let Some((_, g)) = &arm.guard {
                let g = self.expr_with(g, &Ty::Bool);
                let g = self.settle(g);
                if pats.binding_cases.is_empty() {
                    guards.push(g.code);
                } else {
                    // The guard uses bindings Swift's pattern cannot take: bound first.
                    let chain: Vec<String> = pats
                        .binding_cases
                        .iter()
                        .map(|(code, subject)| format!("case {code} = {subject}"))
                        .collect();
                    guards.push(format!(
                        "{{ guard {} else {{ return false }}; return {} }}()",
                        chain.join(", "),
                        g.code
                    ));
                }
            }
            let mut body = pats.lets.clone();
            // An arm is a scope already: its block needs no `do { }`.
            match &*arm.body {
                Expr::Block(b) if b.label.is_none() => {
                    body.extend(self.block_lines(&b.block, sink.clone()))
                }
                other => body.extend(self.with_prelude(|t| t.tail(other, sink))),
            }
            if let Some(wb) = &write_back {
                // A `return` in the arm (or the arm's value returned) would skip a store at
                // its end: `defer` stores first.
                if has_control_flow(&arm.body) || *sink == Sink::Return {
                    body.insert(0, format!("defer {{ {} }}", wb.1));
                } else {
                    body.push(wb.1.clone());
                }
            }
            self.pop_scope();
            let body = join_or_break(body, sink, self.in_view_builder());
            let last = i + 1 == n;
            // Rust's match is exhaustive: its last arm (without `if`) takes whatever is left,
            // which Swift cannot always prove (slices, struct fields, integer ranges).
            let catch_all = last
                && arm.guard.is_none()
                && !pats.code.contains("let ")
                && !pats.code.contains("var ")
                && (!pats.guards.is_empty()
                    || matches!(subject_ty, Ty::Int(_) | Ty::Char | Ty::Str | Ty::Float(_)));
            let label = if (pats.wildcard && guards.is_empty() || catch_all) && last {
                "default:".to_string()
            } else {
                let w = if guards.is_empty() {
                    String::new()
                } else {
                    format!(" where {}", guards.join(" && "))
                };
                format!("case {}{w}:", pats.code)
            };
            let mut case = String::new();
            for c in leading {
                case.push_str(&c);
                case.push('\n');
            }
            case.push_str(&format!("{label}\n{}", indent_lines(&body)));
            cases.push(case);
        }
        self.switch_depth -= 1;
        vec![format!(
            "switch {subject_code} {{\n{}\n}}",
            cases.join("\n")
        )]
    }

    /// `match v.get_mut(i) { Some(t) => …, None => … }`: `t` stands for the element itself,
    /// so the arms become an `if`.
    fn match_mutable(&mut self, m: &syn::ExprMatch, sink: &Sink) -> Option<Vec<String>> {
        let Expr::MethodCall(call) = &*m.expr else {
            return None;
        };
        if !matches!(
            call.method.to_string().as_str(),
            "get_mut" | "as_mut" | "last_mut" | "first_mut" | "back_mut" | "front_mut" | "find"
        ) && !self.is_key_path_call(call)
            || m.arms.len() != 2
        {
            return None;
        }
        let some_arm = m.arms.iter().find(|a| {
            matches!(&a.pat, Pat::TupleStruct(ts) if ts.path.is_ident("Some")
                && ts.elems.len() == 1
                && matches!(ts.elems[0], Pat::Ident(_)))
        })?;
        let none_arm = m.arms.iter().find(|a| {
            matches!(&a.pat, Pat::Wild(_))
                || matches!(&a.pat, Pat::Ident(id) if id.ident == "None" && id.subpat.is_none())
        })?;
        if some_arm.guard.is_some() || none_arm.guard.is_some() {
            return None;
        }
        self.push_scope();
        let Some(cond) = self.mutable_binding(&some_arm.pat, &m.expr) else {
            self.pop_scope();
            return None;
        };
        let then = self.arm_lines(some_arm, sink);
        self.pop_scope();
        self.push_scope();
        let otherwise = self.arm_lines(none_arm, sink);
        self.pop_scope();
        let then = join_or_break(then, sink, self.in_view_builder());
        let otherwise = join_or_break(otherwise, sink, self.in_view_builder());
        Some(vec![format!(
            "if {cond} {{\n{}\n}} else {{\n{}\n}}",
            indent_lines(&then),
            indent_lines(&otherwise)
        )])
    }

    /// The statements of a `match` arm.
    fn arm_lines(&mut self, arm: &syn::Arm, sink: &Sink) -> Vec<String> {
        match &*arm.body {
            Expr::Block(b) if b.label.is_none() => self.block_lines(&b.block, sink.clone()),
            other => self.with_prelude(|t| t.tail(other, sink)),
        }
    }

    /// The fields of an enum variant changed through a `match` on a place: the Swift pattern
    /// binding every field (`var` for the changed ones) and the assignment that stores the
    /// variant back (`self = .rect(w: w, h: h)`).
    fn variant_write_back(
        &mut self,
        pat: &Pat,
        body: &Expr,
        subject_ty: &Ty,
        subject: &str,
    ) -> Option<(String, String)> {
        let mutated = self.mutated_idents(body);
        if mutated.is_empty() {
            return None;
        }
        // `Some(Value::Int(n)) => *n += 1` on `map.get_mut(k)`: the variant inside the option.
        if let (Pat::TupleStruct(ts), Ty::Option(inner)) = (pat, subject_ty) {
            if ts.path.is_ident("Some") && ts.elems.len() == 1 {
                let (code, back) =
                    self.variant_write_back(&ts.elems[0], body, inner, &format!("{subject}!"))?;
                return Some((format!(".some({code})"), back));
            }
        }
        let (segs, fields_of): (Vec<String>, FieldPatterns) = match pat {
            Pat::Struct(s) => {
                let segs: Vec<String> = s
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                let s = s.clone();
                (
                    segs,
                    Box::new(move |names: &[String], _: &[Ty]| {
                        names
                            .iter()
                            .map(|n| {
                                s.fields
                                    .iter()
                                    .find(|f| crate::expr::member_name(&f.member) == *n)
                                    .map(|f| (*f.pat).clone())
                            })
                            .collect()
                    }),
                )
            }
            Pat::TupleStruct(ts) => {
                let segs: Vec<String> = ts
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                let elems: Vec<Pat> = ts.elems.iter().cloned().collect();
                (
                    segs,
                    Box::new(move |_: &[String], tys: &[Ty]| {
                        let rest = elems.iter().position(|p| matches!(p, Pat::Rest(_)));
                        (0..tys.len())
                            .map(|i| match rest {
                                Some(r) if i < r => elems.get(i).cloned(),
                                Some(r) => {
                                    let from_end = tys.len() - i;
                                    let after = elems.len() - r - 1;
                                    (from_end <= after)
                                        .then(|| elems[elems.len() - from_end].clone())
                                }
                                None => elems.get(i).cloned(),
                            })
                            .collect()
                    }),
                )
            }
            _ => return None,
        };
        let last = segs.last()?.clone();
        let owner = if segs.len() >= 2 {
            self.rust_type_name(&segs[segs.len() - 2])
        } else {
            let Ty::Named(n, _) = subject_ty else {
                return None;
            };
            self.rust_name_of_swift(n)?
        };
        let en = self.syms.enums.get(&owner)?.clone();
        let v = en.variant(&last)?.clone();
        let (names, tys, labeled): (Vec<String>, Vec<Ty>, bool) = match &v.shape {
            crate::symbols::VariantShape::Struct(fields) => (
                fields.iter().map(|(n, _)| n.clone()).collect(),
                fields.iter().map(|(_, t)| t.clone()).collect(),
                true,
            ),
            crate::symbols::VariantShape::Tuple(tys) => (
                (0..tys.len()).map(|i| i.to_string()).collect(),
                tys.clone(),
                false,
            ),
            crate::symbols::VariantShape::Unit => return None,
        };
        let subpats = fields_of(&names, &tys);
        // Every field gets a name: the bound ones keep theirs, the others a fresh one.
        let mut rust_names = Vec::new();
        for sp in &subpats {
            match sp {
                None | Some(Pat::Wild(_)) | Some(Pat::Rest(_)) => rust_names.push(None),
                Some(Pat::Ident(id)) if id.subpat.is_none() => {
                    rust_names.push(Some(id.ident.to_string()));
                }
                _ => return None,
            }
        }
        if !rust_names.iter().flatten().any(|n| mutated.contains(n)) {
            return None;
        }
        let mut parts = Vec::new();
        let mut args = Vec::new();
        for (i, rust) in rust_names.iter().enumerate() {
            let (rust, kw) = match rust {
                Some(r) => (r.clone(), if mutated.contains(r) { "var" } else { "let" }),
                None => (format!("__field{i}"), "let"),
            };
            let swift = self.declare(&rust, tys[i].clone());
            parts.push(format!("{kw} {swift}"));
            if labeled {
                args.push(format!("{}: {swift}", names::lower_camel(&names[i])));
            } else {
                args.push(swift);
            }
        }
        Some((
            format!(".{}({})", v.swift, parts.join(", ")),
            format!("{subject} = .{}({})", v.swift, args.join(", ")),
        ))
    }

    /// The variables that `e` changes: assigned, compound-assigned, or the receiver of a
    /// method that changes it.
    fn mutated_idents(&self, e: &Expr) -> std::collections::HashSet<String> {
        use syn::visit::Visit;
        struct V<'a, 'b> {
            t: &'a Translator<'b>,
            out: std::collections::HashSet<String>,
        }
        fn root(e: &Expr) -> Option<String> {
            match e {
                Expr::Path(p) => p.path.get_ident().map(ToString::to_string),
                Expr::Unary(u) => root(&u.expr),
                Expr::Field(f) => root(&f.base),
                Expr::Index(i) => root(&i.expr),
                Expr::Paren(p) => root(&p.expr),
                Expr::Reference(r) => root(&r.expr),
                Expr::MethodCall(m) => root(&m.receiver),
                _ => None,
            }
        }
        impl<'ast> Visit<'ast> for V<'_, '_> {
            fn visit_expr(&mut self, e: &'ast Expr) {
                match e {
                    Expr::Assign(a) => self.out.extend(root(&a.left)),
                    Expr::Binary(b)
                        if matches!(
                            b.op,
                            syn::BinOp::AddAssign(_)
                                | syn::BinOp::SubAssign(_)
                                | syn::BinOp::MulAssign(_)
                                | syn::BinOp::DivAssign(_)
                                | syn::BinOp::RemAssign(_)
                                | syn::BinOp::BitAndAssign(_)
                                | syn::BinOp::BitOrAssign(_)
                                | syn::BinOp::BitXorAssign(_)
                                | syn::BinOp::ShlAssign(_)
                                | syn::BinOp::ShrAssign(_)
                        ) =>
                    {
                        self.out.extend(root(&b.left));
                    }
                    Expr::MethodCall(m) if self.t.is_mutating_method(&m.method.to_string()) => {
                        self.out.extend(root(&m.receiver));
                    }
                    Expr::Reference(r) if r.mutability.is_some() => self.out.extend(root(&r.expr)),
                    _ => {}
                }
                syn::visit::visit_expr(self, e);
            }
        }
        let mut v = V {
            t: self,
            out: std::collections::HashSet::new(),
        };
        v.visit_expr(e);
        v.out
    }

    // ----- Loops -----

    fn for_loop(&mut self, f: &syn::ExprForLoop) -> Vec<String> {
        let name = self.loop_label(f.label.as_ref());
        self.loops.push(Some(name.clone()));
        let saved_switches = std::mem::replace(&mut self.switch_depth, 0);
        self.push_scope();
        if let Some(lines) = self.chunks_mut_loop(f, &name) {
            self.pop_scope();
            self.loops.pop();
            self.switch_depth = saved_switches;
            return lines;
        }
        self.no_trailing += 1;
        let iter = self.iterable(&f.expr);
        self.no_trailing -= 1;
        let mut filter_where = String::new();
        let head = match &iter.mutable_items {
            Some(collection) => {
                // `for x in v.iter_mut()`: loop on the indices, `x` stands for `v[i]`.
                let mut i = self.fresh("i");
                match &*f.pat {
                    // `for (i, x) in v.iter_mut().enumerate()`: the loop index is `i`.
                    Pat::Tuple(t) if t.elems.len() == 2 && iter.enumerated && !iter.keyed => {
                        if let Pat::Ident(index) = &t.elems[0] {
                            i = self.declare(
                                &index.ident.to_string(),
                                Ty::Int(crate::ty::IntKind::Usize),
                            );
                        }
                        if let Pat::Ident(id) = &t.elems[1] {
                            self.declare_alias(
                                &id.ident.to_string(),
                                format!("{collection}[{i}]"),
                                iter.item.clone(),
                            );
                        }
                    }
                    Pat::Ident(id) => {
                        let element = if iter.keyed {
                            format!("{collection}[{i}]!")
                        } else {
                            format!("{collection}[{i}]")
                        };
                        self.declare_alias(
                            &id.ident.to_string(),
                            element.clone(),
                            iter.item.clone(),
                        );
                        // `.filter(|c| c.alive)`: a `where` on the loop, `c` the element too.
                        if let Some(c) = &iter.filter {
                            if let Some(p) = c.inputs.first() {
                                let pat = match p {
                                    Pat::Type(pt) => &*pt.pat,
                                    p => p,
                                };
                                let pat = match pat {
                                    Pat::Reference(r) => &*r.pat,
                                    p => p,
                                };
                                if let Pat::Ident(pid) = pat {
                                    self.declare_alias(
                                        &pid.ident.to_string(),
                                        element,
                                        iter.item.clone(),
                                    );
                                }
                            }
                            let lines = self.with_prelude(|t| {
                                let v = t.expr_with(&c.body, &Ty::Bool);
                                let v = t.settle(v);
                                vec![v.code]
                            });
                            match lines.split_last() {
                                Some((cond, [])) => filter_where = format!(" where {cond}"),
                                Some((cond, pre)) => {
                                    self.prelude_body.extend(pre.iter().cloned());
                                    self.prelude_body
                                        .push(format!("if !({cond}) {{ continue }}"));
                                }
                                None => {}
                            }
                        }
                    }
                    Pat::Tuple(t) if t.elems.len() == 2 && iter.keyed => {
                        if let Pat::Ident(k) = &t.elems[0] {
                            let kn = self.declare(&k.ident.to_string(), iter.key.clone());
                            if let Pat::Ident(v) = &t.elems[1] {
                                self.declare_alias(
                                    &v.ident.to_string(),
                                    format!("{collection}[{kn}]!"),
                                    iter.item.clone(),
                                );
                            }
                            let body = self.block_lines(&f.body, Sink::Discard);
                            self.pop_scope();
                            self.loops.pop();
                            self.switch_depth = saved_switches;
                            let label = self.label_prefix(f.label.is_some(), &name);
                            return vec![format!(
                                "{label}for {kn} in {collection}.keys {{\n{}\n}}",
                                indent_lines(&body.join("\n"))
                            )];
                        }
                    }
                    _ => {}
                }
                if iter.keyed {
                    format!("for {i} in Array({collection}.keys)")
                } else {
                    format!("for {i} in {collection}.indices{filter_where}")
                }
            }
            None => {
                let pat = self.binding_pattern(&f.pat, &iter.item);
                format!("for {pat} in {}", iter.code)
            }
        };
        let mut body = std::mem::take(&mut self.prelude_body);
        body.extend(self.block_lines(&f.body, Sink::Discard));
        self.pop_scope();
        self.loops.pop();
        self.switch_depth = saved_switches;
        let label = self.label_prefix(f.label.is_some(), &name);
        vec![format!(
            "{label}{head} {{\n{}\n}}",
            indent_lines(&body.join("\n"))
        )]
    }

    /// `for chunk in v.chunks_mut(n) { … }`: each chunk is a copy of a part of `v`, written
    /// back when its iteration ends.
    fn chunks_mut_loop(&mut self, f: &syn::ExprForLoop, name: &str) -> Option<Vec<String>> {
        let Expr::MethodCall(m) = &*f.expr else {
            return None;
        };
        let exact = match m.method.to_string().as_str() {
            "chunks_mut" => false,
            "chunks_exact_mut" => true,
            _ => return None,
        };
        if m.args.len() != 1 || !simple_place(&m.receiver) {
            return None;
        }
        let Pat::Ident(id) = &*f.pat else { return None };
        let ty = self.place_ty(&m.receiver);
        let base = self.place(&m.receiver);
        let n = self.expr_with(&m.args[0], &Ty::Int(crate::ty::IntKind::Usize));
        let n = self.settle(n);
        let n = n.code;
        let start = self.fresh("start");
        let chunk = self.declare(&id.ident.to_string(), ty);
        let end = if exact {
            format!("{start} + {n}")
        } else {
            format!("min({start} + {n}, {base}.count)")
        };
        let to = if exact {
            format!("{base}.count - {base}.count % {n}")
        } else {
            format!("{base}.count")
        };
        let mut body = vec![
            format!("var {chunk} = Array({base}[{start}..<{end}])"),
            format!(
                "defer {{ {base}.replaceSubrange({start}..<{start} + {chunk}.count, with: {chunk}) }}"
            ),
        ];
        body.extend(self.block_lines(&f.body, Sink::Discard));
        let label = self.label_prefix(f.label.is_some(), name);
        Some(vec![format!(
            "{label}for {start} in stride(from: 0, to: {to}, by: {n}) {{\n{}\n}}",
            indent_lines(&body.join("\n"))
        )])
    }

    /// The Swift label of a loop: Rust's, or a fresh one in case a `break` inside a `switch`
    /// needs to name the loop.
    fn loop_label(&mut self, label: Option<&syn::Label>) -> String {
        match label {
            Some(l) => l.name.ident.to_string(),
            None => self.fresh("loop"),
        }
    }

    /// `label: ` before a loop labeled in Rust, or whose generated label a `break` uses.
    fn label_prefix(&self, explicit: bool, name: &str) -> String {
        if explicit || self.used_labels.contains(name) {
            format!("{name}: ")
        } else {
            String::new()
        }
    }

    fn while_loop(&mut self, w: &syn::ExprWhile) -> Vec<String> {
        let name = self.loop_label(w.label.as_ref());
        self.loops.push(Some(name.clone()));
        let saved_switches = std::mem::replace(&mut self.switch_depth, 0);
        self.push_scope();
        let cond_lines = self.with_prelude(|t| {
            let c = t.condition(&w.cond);
            vec![c]
        });
        let (cond, pre) = cond_lines
            .split_last()
            .map(|(c, p)| (c.clone(), p.to_vec()))
            .unwrap_or_default();
        let body = self.block_lines(&w.body, Sink::Discard);
        self.pop_scope();
        self.loops.pop();
        self.switch_depth = saved_switches;
        let label = self.label_prefix(w.label.is_some(), &name);
        if !pre.is_empty() {
            // The condition needs statements: evaluate it at the top of an endless loop.
            let mut inner = pre;
            inner.push(format!("if !({cond}) {{ break }}"));
            inner.extend(body);
            return vec![format!(
                "{label}while true {{\n{}\n}}",
                indent_lines(&inner.join("\n"))
            )];
        }
        vec![format!(
            "{label}while {cond} {{\n{}\n}}",
            indent_lines(&body.join("\n"))
        )]
    }

    fn loop_stmt(&mut self, l: &syn::ExprLoop) -> Vec<String> {
        self.loop_with(l, None)
    }

    /// `loop { … }`; `value`: the variable its `break value` assigns.
    fn loop_with(&mut self, l: &syn::ExprLoop, value: Option<String>) -> Vec<String> {
        let name = self.loop_label(l.label.as_ref());
        self.loops.push(Some(name.clone()));
        let saved_switches = std::mem::replace(&mut self.switch_depth, 0);
        self.loop_values.resize(self.loops.len() - 1, None);
        self.loop_values.push(value);
        let body = self.block_lines(&l.body, Sink::Discard);
        self.loops.pop();
        self.loop_values.truncate(self.loops.len());
        self.switch_depth = saved_switches;
        let label = self.label_prefix(l.label.is_some(), &name);
        vec![format!(
            "{label}while true {{\n{}\n}}",
            indent_lines(&body.join("\n"))
        )]
    }

    /// What a `for` loop iterates on.
    fn iterable(&mut self, e: &Expr) -> Iterable {
        // `for x in &mut v` / `v.iter_mut()` / `map.values_mut()`: elements changed in place.
        // `.iter_mut().enumerate()`: the same, with the index.
        if let Expr::MethodCall(m) = e {
            if m.method == "enumerate" && m.args.is_empty() {
                let inner_mutable = matches!(&*m.receiver, Expr::MethodCall(i) if i.method == "iter_mut")
                    || matches!(&*m.receiver, Expr::Reference(r) if r.mutability.is_some());
                if inner_mutable {
                    let mut it = self.iterable(&m.receiver);
                    it.enumerated = true;
                    return it;
                }
            }
        }
        let mutable = match e {
            Expr::Reference(r) if r.mutability.is_some() => Some((&*r.expr, false, None)),
            Expr::MethodCall(m) if m.method == "iter_mut" && m.args.is_empty() => {
                Some((&*m.receiver, false, None))
            }
            Expr::MethodCall(m) if m.method == "values_mut" && m.args.is_empty() => {
                Some((&*m.receiver, true, None))
            }
            // `v.iter_mut().filter(|x| …)`: the elements kept, changed in place.
            Expr::MethodCall(m) if m.method == "filter" && m.args.len() == 1 => {
                match (&*m.receiver, &m.args[0]) {
                    (Expr::MethodCall(inner), Expr::Closure(c))
                        if inner.method == "iter_mut" && inner.args.is_empty() =>
                    {
                        Some((&*inner.receiver, false, Some(c.clone())))
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some((collection, values, filter)) = mutable {
            let v = self.expr(collection);
            let v = self.settle(v);
            let keyed = matches!(v.ty, Ty::Map(..));
            let key_ty = match &v.ty {
                Ty::Map(k, _, _) => (**k).clone(),
                _ => Ty::Unknown,
            };
            let item = match &v.ty {
                Ty::Map(k, val, _) if !values => Ty::Tuple(vec![(**k).clone(), (**val).clone()]),
                Ty::Map(_, val, _) => (**val).clone(),
                t => t.item(),
            };
            let item = if keyed && !values {
                match item {
                    Ty::Tuple(ts) => ts[1].clone(),
                    t => t,
                }
            } else {
                item
            };
            return Iterable {
                code: v.code.clone(),
                item,
                mutable_items: Some(v.code),
                keyed,
                key: key_ty,
                enumerated: false,
                filter,
            };
        }
        let inner = match e {
            Expr::Reference(r) => &*r.expr,
            e => e,
        };
        let v = self.expr(inner);
        let v = self.settle(v);
        let item = match &v.ty {
            Ty::Named(n, _) => self
                .syms
                .iterators
                .get(n)
                .cloned()
                .unwrap_or_else(|| v.ty.item()),
            t => t.item(),
        };
        let code = match &v.ty {
            Ty::Map(_, _, true) if !v.code.contains(".sorted(") => format!(
                "{}.sorted(by: {{ $0.key < $1.key }})",
                crate::expr::wrap(&v, P_PRIMARY)
            ),
            Ty::Set(_, true) => format!("{}.sorted()", crate::expr::wrap(&v, P_PRIMARY)),
            _ => v.code.clone(),
        };
        Iterable {
            code,
            item,
            mutable_items: None,
            keyed: false,
            key: Ty::Unknown,
            enumerated: false,
            filter: None,
        }
    }

    /// Statements written differently from the same expression used as a value.
    fn special_stmt(&mut self, e: &Expr) -> Option<Vec<String>> {
        let Expr::MethodCall(m) = e else { return None };
        if m.method != "insert" {
            return None;
        }
        let recv = self.expr(&m.receiver);
        let recv = self.settle(recv);
        match (&recv.ty, m.args.len()) {
            (Ty::Map(k, v, ordered), 2) => {
                let key = self.expr_with(&m.args[0], k);
                let key = self.settle(key);
                let value = self.expr_with(&m.args[1], v);
                let value = self.settle(value);
                let learned = Ty::Map(
                    Box::new(key.ty.clone()),
                    Box::new(value.ty.clone()),
                    *ordered,
                );
                if let Expr::Path(p) = &*m.receiver {
                    if let Some(id) = p.path.get_ident() {
                        self.refine(&id.to_string(), &learned);
                    }
                }
                Some(vec![format!(
                    "{}[{}] = {}",
                    crate::expr::wrap(&recv, P_PRIMARY),
                    key.code,
                    value.code
                )])
            }
            (Ty::Set(t, ordered), 1) => {
                let x = self.expr_with(&m.args[0], t);
                let x = self.settle(x);
                let learned = Ty::Set(Box::new(x.ty.clone()), *ordered);
                if let Expr::Path(p) = &*m.receiver {
                    if let Some(id) = p.path.get_ident() {
                        self.refine(&id.to_string(), &learned);
                    }
                }
                Some(vec![format!(
                    "{}.insert({})",
                    crate::expr::wrap(&recv, P_PRIMARY),
                    x.code
                )])
            }
            _ => None,
        }
    }

    // ----- Conditions -----

    /// The condition of an `if`/`while`: a boolean, `let` patterns (`if let Some(x) = o`), or
    /// a chain of both with `&&`. Bindings are declared in the current scope.
    pub fn condition(&mut self, cond: &Expr) -> String {
        self.no_trailing += 1;
        let c = self.condition_inner(cond);
        self.no_trailing -= 1;
        c
    }

    fn condition_inner(&mut self, cond: &Expr) -> String {
        if !contains_let(cond) {
            let v = self.expr_with(cond, &Ty::Bool);
            let v = self.settle(v);
            // `while matches!(…)`: a closure applied on the spot cannot open a condition.
            if v.code.starts_with('{') {
                return format!("({})", v.code);
            }
            return v.code;
        }
        let mut parts = Vec::new();
        self.condition_parts(cond, &mut parts);
        parts.join(", ")
    }

    fn condition_parts(&mut self, cond: &Expr, out: &mut Vec<String>) {
        match cond {
            Expr::Binary(b) if matches!(b.op, syn::BinOp::And(_)) && contains_let(cond) => {
                self.condition_parts(&b.left, out);
                self.condition_parts(&b.right, out);
            }
            Expr::Let(l) => {
                if let Some(c) = self.mutable_binding(&l.pat, &l.expr) {
                    out.push(c);
                    return;
                }
                let v = self.expr(&l.expr);
                out.push(self.pattern_condition(&l.pat, v));
            }
            Expr::Paren(p) => self.condition_parts(&p.expr, out),
            other => {
                let v = self.expr_with(other, &Ty::Bool);
                let v = self.settle(v);
                out.push(v.code);
            }
        }
    }

    /// `if let Some(x) = map.get_mut(&k)` and friends: Swift has no mutable references, so
    /// `x` stands for the element itself (`map[k]!`, `items[i]`).
    fn mutable_binding(&mut self, pat: &Pat, e: &Expr) -> Option<String> {
        let Pat::TupleStruct(ts) = pat else {
            return None;
        };
        if !ts.path.is_ident("Some") || ts.elems.len() != 1 {
            return None;
        }
        let Pat::Ident(id) = &ts.elems[0] else {
            return None;
        };
        let Expr::MethodCall(m) = e else { return None };
        let (place, ty, condition) = self.mutable_place(m)?;
        self.declare_alias(&id.ident.to_string(), place, ty);
        Some(condition)
    }

    /// `v.last_mut()`, `v.get_mut(i)`, `map.get_mut(&k)`, `opt.as_mut()`,
    /// `v.iter_mut().find(…)`: the element itself as a place, with the condition under
    /// which it exists (a `let` for `find`, which looks the index up).
    fn mutable_place(&mut self, m: &syn::ExprMethodCall) -> Option<(String, Ty, String)> {
        match m.method.to_string().as_str() {
            "get_mut" if m.args.len() == 1 => {
                let coll = self.expr(&m.receiver);
                let coll = self.settle(coll);
                let c = crate::expr::wrap(&coll, P_PRIMARY);
                match &coll.ty {
                    Ty::Map(k, v, _) => {
                        let key = self.expr_with(&m.args[0], k);
                        let key = self.settle(key);
                        Some((
                            format!("{c}[{}]!", key.code),
                            (**v).clone(),
                            format!("{c}[{}] != nil", key.code),
                        ))
                    }
                    Ty::Vec(t) => {
                        let i = self.expr_with(&m.args[0], &Ty::Int(crate::ty::IntKind::Usize));
                        let i = self.settle(i);
                        Some((
                            format!("{c}[{}]", i.code),
                            (**t).clone(),
                            format!("{c}.indices.contains({})", i.code),
                        ))
                    }
                    _ => None,
                }
            }
            // `opt.as_mut()`: `opt!`.
            "as_mut" if m.args.is_empty() => {
                let Ty::Option(inner) = self.place_ty(&m.receiver) else {
                    return None;
                };
                let place = self.place(&m.receiver);
                Some((format!("{place}!"), *inner, format!("{place} != nil")))
            }
            "last_mut" | "first_mut" | "back_mut" | "front_mut" if m.args.is_empty() => {
                let coll = self.expr(&m.receiver);
                let coll = self.settle(coll);
                let c = crate::expr::wrap(&coll, P_PRIMARY);
                let index = if m.method == "last_mut" || m.method == "back_mut" {
                    format!("{c}.count - 1")
                } else {
                    "0".into()
                };
                Some((
                    format!("{c}[{index}]"),
                    coll.ty.item(),
                    format!("!{c}.isEmpty"),
                ))
            }
            "find" if m.args.len() == 1 => {
                // `v.iter_mut().find(|x| …)` → the index of the first match.
                let Expr::MethodCall(inner) = &*m.receiver else {
                    return None;
                };
                if inner.method != "iter_mut" {
                    return None;
                }
                let coll = self.expr(&inner.receiver);
                let coll = self.settle(coll);
                let c = crate::expr::wrap(&coll, P_PRIMARY);
                let item = coll.ty.item();
                let f = self.expr_with(&m.args[0], &Ty::Fn(vec![item.clone()], Box::new(Ty::Bool)));
                let f = self.settle(f);
                let i = self.fresh("index");
                Some((
                    format!("{c}[{i}]"),
                    item,
                    format!("let {i} = {c}.firstIndex(where: {})", f.code),
                ))
            }
            // `self.find_mut(k)`: a method of the crate returning `Option<&mut T>` as a key path.
            _ => {
                let (call, root, sig) = self.raw_key_path_call(m)?;
                if !matches!(sig.ret, Ty::Option(_)) {
                    return None;
                }
                let p = self.fresh("path");
                Some((
                    format!("{root}[keyPath: {p}]"),
                    sig.ret.item(),
                    format!("let {p} = {call}"),
                ))
            }
        }
    }

    /// `let x = map.get_mut(&k).ok_or(e)?;`, `let x = v.last_mut()?;`: the element itself,
    /// after a `guard` that it exists.
    fn let_mutable_try(&mut self, rust: &str, inner: &Expr) -> Option<Vec<String>> {
        let (accessor, error) = match inner {
            Expr::MethodCall(m)
                if matches!(m.method.to_string().as_str(), "ok_or" | "ok_or_else")
                    && m.args.len() == 1 =>
            {
                (&*m.receiver, Some((&m.args[0], m.method == "ok_or_else")))
            }
            e => (e, None),
        };
        let Expr::MethodCall(accessor) = accessor else {
            return None;
        };
        // `let acct = self.get(name)?;` with a method returning `Result<&mut T, E>`.
        if error.is_none() {
            if let Some((call, root, sig)) = self.raw_key_path_call(accessor) {
                if let Ty::Result(ok, _) = &sig.ret {
                    let p = self.fresh("path");
                    self.declare_alias(rust, format!("{root}[keyPath: {p}]"), (**ok).clone());
                    return Some(vec![format!("let {p} = try {call}")]);
                }
            }
        }
        if !matches!(
            accessor.method.to_string().as_str(),
            "get_mut" | "as_mut" | "last_mut" | "first_mut" | "back_mut" | "front_mut" | "find"
        ) && !self.is_key_path_call(accessor)
        {
            return None;
        }
        self.preludes.push(Vec::new());
        let saved = self.diags.len();
        let found = self.mutable_place(accessor);
        let pre = self.preludes.pop().unwrap_or_default();
        let Some((place, ty, condition)) = found else {
            self.diags.truncate(saved);
            return None;
        };
        let throws = self.fn_cx().and_then(|f| f.throws.clone());
        let exit = match (error, &throws) {
            (Some((e, lazy)), Some(err_ty)) => {
                let code = match (lazy, e) {
                    (true, Expr::Closure(c)) => {
                        let v = self.expr_with(&c.body, err_ty);
                        self.settle(v).code
                    }
                    (true, e) => {
                        let f = self.expr(e);
                        format!("({})()", f.code)
                    }
                    (false, e) => {
                        let v = self.expr_with(e, err_ty);
                        self.settle(v).code
                    }
                };
                format!("throw {code}")
            }
            (None, Some(_)) => {
                self.helper("ParseError");
                "throw RsParseError()".to_string()
            }
            (_, None) if self.fn_cx().is_some_and(|f| f.ret == Ty::Unit) => "return".into(),
            (_, None) => "return nil".into(),
        };
        let mut lines = pre;
        lines.push(format!("guard {condition} else {{ {exit} }}"));
        self.declare_alias(rust, place, ty);
        Some(lines)
    }

    /// The place of `v.last_mut().unwrap()` and friends: the element itself, which must
    /// exist. Nothing is translated when the receiver is not one of the mutable accessors.
    /// `cell.borrow_mut()`, `m.lock().unwrap()` on a simple place: the value of the cell,
    /// `cell.value`, with its type.
    pub fn cell_place(&mut self, m: &syn::ExprMethodCall) -> Option<(String, Ty)> {
        let method = m.method.to_string();
        let cell: &Expr = match method.as_str() {
            "borrow" | "borrow_mut" | "get_mut" if m.args.is_empty() => &m.receiver,
            "unwrap" | "expect" => match &*m.receiver {
                Expr::MethodCall(inner)
                    if inner.args.is_empty()
                        && matches!(
                            inner.method.to_string().as_str(),
                            "lock"
                                | "read"
                                | "write"
                                | "try_lock"
                                | "try_read"
                                | "try_write"
                                | "try_borrow"
                                | "try_borrow_mut"
                        ) =>
                {
                    &inner.receiver
                }
                _ => return None,
            },
            _ => return None,
        };
        if !simple_place(cell) {
            return None;
        }
        let Ty::Named(n, args) = self.place_ty(cell) else {
            return None;
        };
        if n != "RsRefCell" {
            return None;
        }
        let inner = args.first().cloned().unwrap_or(Ty::Unknown);
        let place = self.place(cell);
        Some((format!("{place}.value"), inner))
    }

    pub fn mutable_place_forced(&mut self, m: &syn::ExprMethodCall) -> Option<(String, Ty)> {
        if !matches!(
            m.method.to_string().as_str(),
            "get_mut" | "as_mut" | "last_mut" | "first_mut" | "back_mut" | "front_mut" | "find"
        ) && !self.is_key_path_call(m)
        {
            return None;
        }
        // `self.get(name).unwrap()` with a method returning `Result<&mut T, E>`.
        if let Some((call, root, sig)) = self.raw_key_path_call(m) {
            if let Ty::Result(ok, _) = &sig.ret {
                let p = self.fresh("path");
                self.prelude(format!("let {p} = try! {call}"));
                return Some((format!("{root}[keyPath: {p}]"), (**ok).clone()));
            }
        }
        self.preludes.push(Vec::new());
        let saved = self.diags.len();
        let found = self.mutable_place(m);
        let pre = self.preludes.pop().unwrap_or_default();
        let Some((place, ty, condition)) = found else {
            self.diags.truncate(saved);
            return None;
        };
        for p in pre {
            self.prelude(p);
        }
        // `v.iter_mut().find(…).unwrap()`: the index first.
        if let Some(binding) = condition.strip_prefix("let ") {
            self.prelude(format!("let {binding}!"));
        }
        Some((place, ty))
    }

    /// `opt.as_mut().map(|c| …)`, `v.last_mut().map(|x| …)` as a statement: an `if` on the
    /// place, whose closure parameter stands for the element itself.
    fn option_map_stmt(&mut self, e: &Expr) -> Option<Vec<String>> {
        let Expr::MethodCall(m) = e else { return None };
        if m.method != "map" || m.args.len() != 1 {
            return None;
        }
        let Expr::Closure(c) = &m.args[0] else {
            return None;
        };
        let Expr::MethodCall(accessor) = &*m.receiver else {
            return None;
        };
        if !matches!(
            accessor.method.to_string().as_str(),
            "as_mut" | "get_mut" | "last_mut" | "first_mut" | "back_mut" | "front_mut"
        ) || c.inputs.len() != 1
        {
            return None;
        }
        let Pat::Ident(id) = &c.inputs[0] else {
            return None;
        };
        self.push_scope();
        let Some((place, ty, condition)) = self.mutable_place(accessor) else {
            self.pop_scope();
            return None;
        };
        self.declare_alias(&id.ident.to_string(), place, ty);
        let body = match &*c.body {
            Expr::Block(b) if b.label.is_none() => self.block_lines(&b.block, Sink::Discard),
            other => self.expr_stmt(other),
        };
        self.pop_scope();
        Some(vec![format!(
            "if {condition} {{\n{}\n}}",
            indent_lines(&body.join("\n"))
        )])
    }

    /// `v.iter_mut().for_each(|x| …)`: a `for` loop, whose elements are changed in place.
    fn for_each_loop(&mut self, e: &Expr) -> Option<Vec<String>> {
        let Expr::MethodCall(m) = e else { return None };
        if m.method != "for_each" || m.args.len() != 1 {
            return None;
        }
        let mutable = match &*m.receiver {
            Expr::MethodCall(r) => {
                matches!(r.method.to_string().as_str(), "iter_mut" | "values_mut")
                    && r.args.is_empty()
            }
            Expr::Reference(r) => r.mutability.is_some(),
            _ => false,
        };
        if !mutable {
            return None;
        }
        let Expr::Closure(c) = &m.args[0] else {
            return None;
        };
        // `return` in the closure would mean `continue`: not a loop then.
        if c.inputs.len() != 1 || has_control_flow(&c.body) {
            return None;
        }
        let mut pat = c.inputs[0].clone();
        while let Pat::Type(t) = pat {
            pat = *t.pat;
        }
        let body: Block = match &*c.body {
            Expr::Block(b) => b.block.clone(),
            other => syn::parse_quote!({ #other }),
        };
        let iter = &m.receiver;
        let f: syn::ExprForLoop = syn::parse_quote!(for #pat in #iter #body);
        Some(self.for_loop(&f))
    }

    /// `let x = opt`, `case .circle(let r) = shape`… for a pattern matched against a value.
    pub fn pattern_condition(&mut self, pat: &Pat, v: Ex) -> String {
        // `Some(x)` / `Ok(x)` with a plain name: Swift's optional binding.
        if let Pat::TupleStruct(ts) = pat {
            let ctor = ts
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            if ts.elems.len() == 1 {
                // `Some(&x)` binds like `Some(x)`.
                let inner = match &ts.elems[0] {
                    Pat::Reference(r) => &*r.pat,
                    p => p,
                };
                if let Pat::Ident(id) = inner {
                    if id.subpat.is_none() {
                        let kw = if id.mutability.is_some() {
                            "var"
                        } else {
                            "let"
                        };
                        if ctor == "Some"
                            && (v.ty.is_option() || v.ty.is_unknown())
                            && v.throws.is_none()
                        {
                            let item = v.ty.item();
                            let n = self.declare(&id.ident.to_string(), item);
                            return format!("{kw} {n} = {}", crate::expr::condition_value(v.code));
                        }
                        if ctor == "Ok" && v.throws.is_some() {
                            let ok = v
                                .throws
                                .as_ref()
                                .map(|t| t.0.clone())
                                .unwrap_or(Ty::Unknown);
                            let n = self.declare(&id.ident.to_string(), ok);
                            return format!(
                                "{kw} {n} = try? {}",
                                crate::expr::condition_value(v.code)
                            );
                        }
                    }
                }
            }
        }
        let v = self.settle(v);
        let subject = v.code.clone();
        let pats = self.case_patterns(pat, &v.ty, &subject);
        let mut s = format!(
            "case {} = {}",
            pats.code,
            crate::expr::wrap(&v, crate::expr::P_TERNARY + 1)
        );
        for g in pats.guards {
            s.push_str(", ");
            s.push_str(&g);
        }
        if !pats.lets.is_empty() {
            for l in &pats.lets {
                if let Some(rest) = l.strip_prefix("let ") {
                    s.push_str(&format!(", case let {rest}"));
                }
            }
        }
        s
    }
}

/// The sub-pattern of each field of a variant, given the field names and types.
type FieldPatterns = Box<dyn Fn(&[String], &[Ty]) -> Vec<Option<Pat>>>;

/// What a `for` loop iterates on.
struct Iterable {
    code: String,
    item: Ty,
    /// `for x in v.iter_mut()`: the collection, whose elements are changed through indices.
    mutable_items: Option<String>,
    /// The collection is a dictionary (indexed by keys).
    keyed: bool,
    /// The type of its keys.
    key: Ty,
    /// `.iter_mut().enumerate()`: the pattern is `(index, element)`.
    enumerated: bool,
    /// `.iter_mut().filter(|x| …)`: only the elements the closure keeps.
    filter: Option<syn::ExprClosure>,
}

/// Whether a statement names an identifier.
/// The locals the statements assign (`cur = …`), at any depth.
fn assigned_names(stmts: &[Stmt]) -> std::collections::HashSet<String> {
    use syn::visit::Visit;
    struct V(std::collections::HashSet<String>);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr_assign(&mut self, a: &'ast syn::ExprAssign) {
            if let Expr::Path(p) = &*a.left {
                if let Some(id) = p.path.get_ident() {
                    self.0.insert(id.to_string());
                }
            }
            syn::visit::visit_expr_assign(self, a);
        }
    }
    let mut v = V(std::collections::HashSet::new());
    for s in stmts {
        v.visit_stmt(s);
    }
    v.0
}

fn mentions(s: &Stmt, name: &str) -> bool {
    quote::quote!(#s)
        .into_iter()
        .any(|t| tree_mentions(t, name))
}

fn tree_mentions(t: proc_macro2::TokenTree, name: &str) -> bool {
    match t {
        proc_macro2::TokenTree::Ident(i) => i == name,
        proc_macro2::TokenTree::Group(g) => g.stream().into_iter().any(|t| tree_mentions(t, name)),
        _ => false,
    }
}

/// Lines of a branch: Swift needs at least one statement in a `case`.
fn join_or_break(lines: Vec<String>, sink: &Sink, view: bool) -> String {
    if lines.is_empty() {
        if *sink == Sink::Yield && view {
            return "EmptyView()".into();
        }
        return "break".into();
    }
    lines.join("\n")
}

fn block_is_single_expr(b: &Block) -> bool {
    match b.stmts.as_slice() {
        [Stmt::Expr(e, None)] => !has_control_flow(e) && !needs_statements(e),
        [Stmt::Macro(m)] => m.semi_token.is_none() && !crate::macros::is_statement_macro(&m.mac),
        _ => false,
    }
}

/// Expressions that Swift can only write as statements.
pub fn needs_statements(e: &Expr) -> bool {
    match e {
        Expr::If(_)
        | Expr::Match(_)
        | Expr::Loop(_)
        | Expr::While(_)
        | Expr::ForLoop(_)
        | Expr::Unsafe(_) => true,
        Expr::Block(_) => true,
        Expr::Paren(p) => needs_statements(&p.expr),
        _ => false,
    }
}

/// `return`, `break`, `continue` or `?` outside closures.
pub fn has_control_flow(e: &Expr) -> bool {
    use syn::visit::Visit;
    struct V(bool);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr(&mut self, e: &'ast Expr) {
            match e {
                Expr::Return(_) | Expr::Break(_) | Expr::Continue(_) | Expr::Try(_) => {
                    self.0 = true
                }
                Expr::Closure(_) => {}
                Expr::Macro(m) if crate::macros::is_statement_macro(&m.mac) => self.0 = true,
                _ => syn::visit::visit_expr(self, e),
            }
        }
        fn visit_item(&mut self, _: &'ast syn::Item) {}
    }
    let mut v = V(false);
    v.visit_expr(e);
    v.0
}

/// `return`, `break`, `continue` or `?` inside the expression: a jump that a closure
/// standing for the expression could not make.
pub fn has_jump(e: &Expr) -> bool {
    use syn::visit::Visit;
    struct V(bool);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr(&mut self, e: &'ast Expr) {
            match e {
                Expr::Return(_) | Expr::Break(_) | Expr::Continue(_) | Expr::Try(_) => {
                    self.0 = true
                }
                Expr::Closure(_) => {}
                _ => syn::visit::visit_expr(self, e),
            }
        }
        fn visit_item(&mut self, _: &'ast syn::Item) {}
    }
    let mut v = V(false);
    v.visit_expr(e);
    v.0
}

pub fn contains_let(e: &Expr) -> bool {
    match e {
        Expr::Let(_) => true,
        Expr::Binary(b) if matches!(b.op, syn::BinOp::And(_)) => {
            contains_let(&b.left) || contains_let(&b.right)
        }
        Expr::Paren(p) => contains_let(&p.expr),
        _ => false,
    }
}

/// `Ok(x)`, `Err(e)`, `Some(x)` → (constructor, argument).
pub fn result_ctor(e: &Expr) -> Option<(&'static str, &Expr)> {
    let Expr::Call(c) = e else { return None };
    let Expr::Path(p) = &*c.func else { return None };
    if c.args.len() != 1 {
        return None;
    }
    let last = p.path.segments.last()?.ident.to_string();
    let ctor = match last.as_str() {
        "Ok" => "Ok",
        "Err" => "Err",
        "Some" => "Some",
        _ => return None,
    };
    if p.path.segments.len() > 1 {
        let first = p.path.segments[0].ident.to_string();
        if !matches!(first.as_str(), "Result" | "Option" | "std" | "core") {
            return None;
        }
    }
    Some((ctor, &c.args[0]))
}

pub fn is_simple_path(code: &str) -> bool {
    !code.is_empty()
        && code
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '`')
}

fn user_call_returns_value(t: &Translator<'_>, e: &Expr) -> bool {
    match e {
        Expr::Call(c) => match &*c.func {
            Expr::Path(p) => p.path.segments.last().is_some_and(|s| {
                t.syms.fns.contains_key(&s.ident.to_string()) && p.path.segments.len() == 1
            }),
            _ => false,
        },
        Expr::MethodCall(m) => {
            let name = m.method.to_string();
            t.syms.methods.values().any(|ms| ms.contains_key(&name))
        }
        _ => false,
    }
}

/// A place that can be written again without a side effect: a variable, a field, `*x`.
pub fn simple_place(e: &Expr) -> bool {
    match e {
        Expr::Path(_) => true,
        Expr::Field(f) => simple_place(&f.base),
        Expr::Paren(p) => simple_place(&p.expr),
        Expr::Reference(r) => simple_place(&r.expr),
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Deref(_)) => simple_place(&u.expr),
        Expr::Index(i) => {
            simple_place(&i.expr) && matches!(&*i.index, Expr::Lit(_) | Expr::Path(_))
        }
        _ => false,
    }
}
