//! Statements and blocks. An `if`, a `match` or a block used as a value is lowered to Swift
//! statements that deliver the value through a [`Sink`]: `return`, an assignment, a view
//! of a result builder…

use syn::spanned::Spanned;
use syn::{Block, Expr, Local, Pat, Stmt};

use crate::expr::{Ex, P_PRIMARY};
use crate::names;
use crate::translate::{Translator, indent_lines};
use crate::ty::Ty;

/// Where the value of a block goes.
#[derive(Clone, Debug, PartialEq)]
pub enum Sink {
    /// Returned from the function.
    Return,
    /// Assigned to a variable.
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
        self.push_scope();
        let lines = self.stmts(&b.stmts, sink, b.span().start().line, b.span().end().line);
        self.pop_scope();
        lines
    }

    pub fn stmts(&mut self, stmts: &[Stmt], sink: Sink, open: usize, close: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut prev = open;
        for (i, s) in stmts.iter().enumerate() {
            let last = i + 1 == stmts.len();
            let start = s.span().start().line;
            let end = s.span().end().line;
            out.extend(self.comments.take_between(prev, start));
            prev = end;
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
        }
        out.extend(self.comments.take_between(prev, close));
        out
    }

    /// Runs `f` with a fresh prelude and puts the prelude's statements first.
    pub fn with_prelude(&mut self, f: impl FnOnce(&mut Self) -> Vec<String>) -> Vec<String> {
        self.preludes.push(Vec::new());
        let lines = f(self);
        let mut out = self.preludes.pop().unwrap_or_default();
        out.extend(lines);
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
        // `RefCell::new(…)` is mutated through a shared reference: a `var` in Swift.
        let mutable = mutable || interior_mutable(value);
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
            let lines = self.tail(value, &Sink::Assign(rust.to_string()));
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
        // `let v = map.entry(k).or_insert(0);`: `v` stands for `map[k, default: 0]`.
        if let Expr::MethodCall(m) = value {
            let ty = self.entry_value_ty(m);
            if let Some(place) = self.entry_place(m) {
                self.declare_alias(rust, place, ty);
                return vec![];
            }
        }
        // `let x = opt?;` → `guard let x = opt else { return nil }`.
        if let Expr::Try(t) = value {
            if let Some(lines) = self.let_try(rust, mutable, &declared_ty, &t.expr) {
                return lines;
            }
        }
        let v = self.expr_with(value, &declared_ty);
        let mut v = self.settle_for_let(v);
        // `let mut it = s.chars();` and then `it.next()`: a Rust iterator with its position.
        if mutable && declared.is_none() {
            if let Ty::Iter(item) = &v.ty {
                self.helper("iter");
                v = Ex::primary(
                    format!("RsIter({})", v.code),
                    Ty::Named("RsIter".into(), vec![(**item).clone()]),
                );
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
        if v.throws.is_some() || v.ty.is_result() && !v.parsed || v.ty == Ty::Unit {
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
            v.code
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
                    return (
                        vec![format!("{kw} ({}) = {subject}", names.join(", "))],
                        true,
                    );
                }
                let mut lines = Vec::new();
                let mut ok = true;
                for (i, p) in t.elems.iter().enumerate() {
                    let (l, o) = self.let_pattern(
                        p,
                        &format!("{subject}.{i}"),
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
                Sink::Assign(name) => {
                    let expected = self
                        .lookup(name)
                        .map(|l| self.local_ty(l))
                        .unwrap_or(Ty::Unknown);
                    let v = self.expr_with(e, &expected);
                    let v = self.settle(v);
                    self.refine(name, &v.ty);
                    let swift = self
                        .lookup(name)
                        .map(|l| l.swift.clone())
                        .unwrap_or_else(|| names::lower_camel(name));
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
    pub fn return_stmt(&mut self, e: Option<&Expr>) -> Vec<String> {
        let Some(cx) = self.fn_cx().cloned() else {
            return vec!["return".into()];
        };
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
            return vec![format!("return {}", v.code)];
        }
        let v = self.expr_with(e, &cx.ret);
        let v = self.settle(v);
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
        self.no_trailing += 1;
        let subject = self.expr(&m.expr);
        self.no_trailing -= 1;
        let subject = self.settle(subject);
        let subject_code = subject.code.clone();
        let subject_ty = subject.ty.clone();
        let mut cases = Vec::new();
        let n = m.arms.len();
        let mut prev = m.expr.span().end().line;
        for (i, arm) in m.arms.iter().enumerate() {
            let leading = self.comments.take_between(prev, arm.span().start().line);
            prev = arm.span().end().line;
            self.push_scope();
            let pats = self.case_patterns(&arm.pat, &subject_ty, &subject_code);
            let mut guards = pats.guards.clone();
            if let Some((_, g)) = &arm.guard {
                let g = self.expr_with(g, &Ty::Bool);
                let g = self.settle(g);
                guards.push(g.code);
            }
            let mut body = pats.lets.clone();
            // An arm is a scope already: its block needs no `do { }`.
            match &*arm.body {
                Expr::Block(b) if b.label.is_none() => {
                    body.extend(self.block_lines(&b.block, sink.clone()))
                }
                other => body.extend(self.tail(other, sink)),
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
        vec![format!(
            "switch {subject_code} {{\n{}\n}}",
            cases.join("\n")
        )]
    }

    // ----- Loops -----

    fn for_loop(&mut self, f: &syn::ExprForLoop) -> Vec<String> {
        let label = f.label.as_ref().map(|l| format!("{}: ", l.name.ident));
        self.loops
            .push(f.label.as_ref().map(|l| l.name.ident.to_string()));
        self.push_scope();
        self.no_trailing += 1;
        let iter = self.iterable(&f.expr);
        self.no_trailing -= 1;
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
                        self.declare_alias(&id.ident.to_string(), element, iter.item.clone());
                    }
                    Pat::Tuple(t) if t.elems.len() == 2 && iter.keyed => {
                        if let Pat::Ident(k) = &t.elems[0] {
                            let kn = self.declare(&k.ident.to_string(), Ty::Unknown);
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
                            return vec![format!(
                                "{}for {kn} in {collection}.keys {{\n{}\n}}",
                                label.unwrap_or_default(),
                                indent_lines(&body.join("\n"))
                            )];
                        }
                    }
                    _ => {}
                }
                if iter.keyed {
                    format!("for {i} in Array({collection}.keys)")
                } else {
                    format!("for {i} in {collection}.indices")
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
        vec![format!(
            "{}{head} {{\n{}\n}}",
            label.unwrap_or_default(),
            indent_lines(&body.join("\n"))
        )]
    }

    fn while_loop(&mut self, w: &syn::ExprWhile) -> Vec<String> {
        let label = w
            .label
            .as_ref()
            .map(|l| format!("{}: ", l.name.ident))
            .unwrap_or_default();
        self.loops
            .push(w.label.as_ref().map(|l| l.name.ident.to_string()));
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
        let label = l
            .label
            .as_ref()
            .map(|l| format!("{}: ", l.name.ident))
            .unwrap_or_default();
        self.loops
            .push(l.label.as_ref().map(|l| l.name.ident.to_string()));
        self.loop_values.resize(self.loops.len() - 1, None);
        self.loop_values.push(value);
        let body = self.block_lines(&l.body, Sink::Discard);
        self.loops.pop();
        self.loop_values.truncate(self.loops.len());
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
            Expr::Reference(r) if r.mutability.is_some() => Some((&*r.expr, false)),
            Expr::MethodCall(m) if m.method == "iter_mut" && m.args.is_empty() => {
                Some((&*m.receiver, false))
            }
            Expr::MethodCall(m) if m.method == "values_mut" && m.args.is_empty() => {
                Some((&*m.receiver, true))
            }
            _ => None,
        };
        if let Some((collection, values)) = mutable {
            let v = self.expr(collection);
            let v = self.settle(v);
            let keyed = matches!(v.ty, Ty::Map(..));
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
                enumerated: false,
            };
        }
        let inner = match e {
            Expr::Reference(r) => &*r.expr,
            e => e,
        };
        let v = self.expr(inner);
        let v = self.settle(v);
        let item = v.ty.item();
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
            enumerated: false,
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
        let name = id.ident.to_string();
        match m.method.to_string().as_str() {
            "get_mut" if m.args.len() == 1 => {
                let coll = self.expr(&m.receiver);
                let coll = self.settle(coll);
                let c = crate::expr::wrap(&coll, P_PRIMARY);
                match &coll.ty {
                    Ty::Map(k, v, _) => {
                        let key = self.expr_with(&m.args[0], k);
                        let key = self.settle(key);
                        self.declare_alias(&name, format!("{c}[{}]!", key.code), (**v).clone());
                        Some(format!("{c}[{}] != nil", key.code))
                    }
                    Ty::Vec(t) => {
                        let i = self.expr_with(&m.args[0], &Ty::Int(crate::ty::IntKind::Usize));
                        let i = self.settle(i);
                        self.declare_alias(&name, format!("{c}[{}]", i.code), (**t).clone());
                        Some(format!("{c}.indices.contains({})", i.code))
                    }
                    _ => None,
                }
            }
            "last_mut" | "first_mut" if m.args.is_empty() => {
                let coll = self.expr(&m.receiver);
                let coll = self.settle(coll);
                let c = crate::expr::wrap(&coll, P_PRIMARY);
                let index = if m.method == "last_mut" {
                    format!("{c}.count - 1")
                } else {
                    "0".into()
                };
                self.declare_alias(&name, format!("{c}[{index}]"), coll.ty.item());
                Some(format!("!{c}.isEmpty"))
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
                self.declare_alias(&name, format!("{c}[{i}]"), item);
                Some(format!("let {i} = {c}.firstIndex(where: {})", f.code))
            }
            _ => None,
        }
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
                            return format!("{kw} {n} = {}", v.code);
                        }
                        if ctor == "Ok" && v.throws.is_some() {
                            let ok = v
                                .throws
                                .as_ref()
                                .map(|t| t.0.clone())
                                .unwrap_or(Ty::Unknown);
                            let n = self.declare(&id.ident.to_string(), ok);
                            return format!("{kw} {n} = try? {}", v.code);
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

/// What a `for` loop iterates on.
struct Iterable {
    code: String,
    item: Ty,
    /// `for x in v.iter_mut()`: the collection, whose elements are changed through indices.
    mutable_items: Option<String>,
    /// The collection is a dictionary (indexed by keys).
    keyed: bool,
    /// `.iter_mut().enumerate()`: the pattern is `(index, element)`.
    enumerated: bool,
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

/// `RefCell::new(x)`, `Rc::new(Cell::new(x))`, `Arc::new(Mutex::new(x))`…
pub fn interior_mutable(e: &Expr) -> bool {
    let Expr::Call(c) = e else { return false };
    let Expr::Path(p) = &*c.func else {
        return false;
    };
    let segs: Vec<String> = p
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    match segs.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [.., "RefCell" | "Cell" | "Mutex" | "RwLock", "new"] => true,
        [.., "Rc" | "Arc" | "Box", "new"] => c.args.first().is_some_and(interior_mutable),
        _ => false,
    }
}
