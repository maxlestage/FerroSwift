//! `impl Drop`: the values of a scope are dropped when it ends, in reverse order, as Rust
//! drops them; a value moved out of its variable (passed by value, returned, stored) is
//! dropped where it went instead.

use std::collections::HashSet;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, Pat, Stmt};

use crate::symbols::{SelfKind, VariantShape};
use crate::translate::Translator;
use crate::ty::Ty;

impl Translator<'_> {
    /// A value of this type runs an `impl Drop` when dropped: its own, or one of a field,
    /// an item or a payload.
    pub fn needs_drop(&self, ty: &Ty) -> bool {
        if self.syms.drop_types.is_empty() {
            return false;
        }
        self.needs_drop_in(ty, &mut HashSet::new())
    }

    fn needs_drop_in(&self, ty: &Ty, seen: &mut HashSet<String>) -> bool {
        match ty {
            Ty::Named(n, args) => {
                if self.syms.drop_types.contains(n) {
                    return true;
                }
                if !seen.insert(n.clone()) {
                    return false;
                }
                if args.iter().any(|a| self.needs_drop_in(a, seen)) {
                    return true;
                }
                if let Some(st) = self.syms.structs.values().find(|s| &s.swift == n) {
                    let fields: Vec<Ty> = st.fields.iter().map(|f| f.ty.clone()).collect();
                    return fields.iter().any(|t| self.needs_drop_in(t, seen));
                }
                if let Some(en) = self.syms.enums.values().find(|e| &e.swift == n) {
                    let payloads: Vec<Ty> = en
                        .variants
                        .iter()
                        .flat_map(|v| match &v.shape {
                            VariantShape::Unit => vec![],
                            VariantShape::Tuple(ts) => ts.clone(),
                            VariantShape::Struct(fs) => fs.iter().map(|(_, t)| t.clone()).collect(),
                        })
                        .collect();
                    return payloads.iter().any(|t| self.needs_drop_in(t, seen));
                }
                false
            }
            Ty::Vec(t) | Ty::Option(t) | Ty::Set(t, _) => self.needs_drop_in(t, seen),
            Ty::Map(k, v, _) | Ty::Result(k, v) => {
                self.needs_drop_in(k, seen) || self.needs_drop_in(v, seen)
            }
            Ty::Tuple(ts) => ts.iter().any(|t| self.needs_drop_in(t, seen)),
            // `Box<dyn Shape>`: the value inside may have an `impl Drop`.
            Ty::Dyn(..) => true,
            _ => false,
        }
    }

    /// The lines that have the local `rust` (Swift `swift`) dropped when the scope being
    /// translated ends. `rest` are the statements that may move it: each move is written
    /// `rsMoved(&flag, x)`, so that the scope no longer drops it.
    pub fn register_drop(&mut self, rust: &str, swift: &str, rest: &[Stmt]) -> Vec<String> {
        self.helper("drop");
        if let Some(frame) = self.drop_frames.last_mut() {
            if frame.is_none() {
                self.tmp += 1;
                *frame = Some(format!("drops{}", self.tmp));
            }
        }
        let by_value: HashSet<String> = self
            .syms
            .methods
            .values()
            .flat_map(|ms| {
                ms.iter()
                    .filter(|(_, s)| s.self_kind == SelfKind::Value)
                    .map(|(n, _)| n.clone())
            })
            .collect();
        let sites = move_sites(rust, rest, &by_value);
        if sites.is_empty() {
            return vec![format!("rsOnDrop {{ rsDropValue({swift}) }}")];
        }
        let flag = format!("{swift}Alive");
        for site in sites {
            self.move_sites.insert(site, flag.clone());
        }
        self.drop_flags.insert(swift.to_string(), flag.clone());
        vec![
            format!("var {flag} = true"),
            format!("rsOnDrop {{ if {flag} {{ rsDropValue({swift}) }} }}"),
        ]
    }
}

impl Translator<'_> {
    /// `slot = Guard::new("new");` on a variable to drop: the old value is dropped once the
    /// new one is made.
    pub fn drop_assign(&mut self, a: &syn::ExprAssign) -> Option<Vec<String>> {
        // `w.current = Job(70)`, `w.queue[0] = Job(80)`: a place of a value to drop.
        if matches!(&*a.left, Expr::Field(_) | Expr::Index(_)) {
            let ty = self.place_ty(&a.left);
            if !self.needs_drop(&ty) {
                return None;
            }
            let v = self.expr_with(&a.right, &ty);
            let v = self.settle(v);
            let new = self.fresh("new");
            let place = self.place(&a.left);
            return Some(vec![
                format!("let {new} = {}", v.code),
                format!("rsDropValue({place})"),
                format!("{place} = {new}"),
            ]);
        }
        let Expr::Path(p) = &*a.left else { return None };
        let rust = p.path.get_ident()?.to_string();
        let local = self.lookup(&rust).cloned()?;
        let ty = self.local_ty(&local);
        if local.alias || !self.needs_drop(&ty) {
            return None;
        }
        let v = self.expr_with(&a.right, &ty);
        let v = self.settle(v);
        let new = self.fresh("new");
        let swift = local.swift.clone();
        let mut lines = vec![format!("let {new} = {}", v.code)];
        match self.drop_flags.get(&swift).cloned() {
            Some(flag) => {
                lines.push(format!("if {flag} {{ rsDropValue({swift}) }}"));
                lines.push(format!("{swift} = {new}"));
                lines.push(format!("{flag} = true"));
            }
            None => {
                lines.push(format!("rsDropValue({swift})"));
                lines.push(format!("{swift} = {new}"));
            }
        }
        Some(lines)
    }
}

impl Translator<'_> {
    /// The bindings of `pat` that own a value taken out of `subject`, a temporary
    /// (`while let Some(job) = queue.pop()`): dropped when the block they are bound for ends.
    pub fn owned_bindings(&self, pat: &Pat, subject: &Expr) -> Vec<(String, String)> {
        if self.syms.drop_types.is_empty()
            || crate::expr::is_place(subject)
            || matches!(subject, Expr::Reference(_))
        {
            return vec![];
        }
        let mut names = Vec::new();
        pattern_names(pat, &mut names);
        names
            .into_iter()
            .filter_map(|rust| {
                let local = self.lookup(&rust)?;
                let ty = self.local_ty(local);
                (self.needs_drop(&ty) && !local.alias).then(|| (rust, local.swift.clone()))
            })
            .collect()
    }

    /// The statements of a body that drops `pending` when it ends: a block of its own when
    /// it is a single expression.
    pub fn body_with_drops(
        &mut self,
        body: &Expr,
        pending: Vec<(String, String)>,
        sink: &crate::stmt::Sink,
    ) -> Option<Vec<String>> {
        if pending.is_empty() {
            return None;
        }
        self.pending_drops.extend(pending);
        let block = match body {
            Expr::Block(b) if b.label.is_none() => b.block.clone(),
            e => syn::Block {
                brace_token: Default::default(),
                stmts: vec![Stmt::Expr(e.clone(), None)],
            },
        };
        Some(self.block_lines(&block, sink.clone()))
    }
}

/// The names a pattern binds by value.
fn pattern_names(p: &Pat, out: &mut Vec<String>) {
    match p {
        Pat::Ident(i) if i.by_ref.is_none() => {
            out.push(i.ident.to_string());
            if let Some((_, sub)) = &i.subpat {
                pattern_names(sub, out);
            }
        }
        Pat::Type(t) => pattern_names(&t.pat, out),
        Pat::Tuple(t) => t.elems.iter().for_each(|p| pattern_names(p, out)),
        Pat::TupleStruct(t) => t.elems.iter().for_each(|p| pattern_names(p, out)),
        Pat::Struct(s) => s.fields.iter().for_each(|f| pattern_names(&f.pat, out)),
        _ => {}
    }
}

/// The positions of the uses of `name` that move its value out, in `stmts` (up to a `let`
/// declaring the name again).
pub fn move_sites(name: &str, stmts: &[Stmt], by_value: &HashSet<String>) -> Vec<(usize, usize)> {
    let mut finder = Moves {
        name,
        by_value,
        sites: Vec::new(),
    };
    finder.statements(stmts);
    finder.sites
}

struct Moves<'a> {
    name: &'a str,
    by_value: &'a HashSet<String>,
    sites: Vec<(usize, usize)>,
}

impl Moves<'_> {
    fn moved(&mut self, e: &Expr) {
        match e {
            Expr::Path(p) if p.path.is_ident(self.name) => {
                let start = e.span().start();
                self.sites.push((start.line, start.column));
            }
            Expr::Paren(p) => self.moved(&p.expr),
            _ => {}
        }
    }

    fn statements(&mut self, stmts: &[Stmt]) {
        for (i, s) in stmts.iter().enumerate() {
            match s {
                Stmt::Local(l) => {
                    if let Some(init) = &l.init {
                        if !matches!(l.pat, Pat::Wild(_)) {
                            self.moved(&init.expr);
                        }
                        self.visit_expr(&init.expr);
                        if let Some((_, e)) = &init.diverge {
                            self.visit_expr(e);
                        }
                    }
                    if binds(&l.pat, self.name) {
                        return;
                    }
                }
                Stmt::Expr(e, semi) => {
                    if semi.is_none() && i + 1 == stmts.len() {
                        self.moved(e);
                    }
                    self.visit_expr(e);
                }
                Stmt::Macro(m) => self.visit_macro(&m.mac),
                Stmt::Item(_) => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Moves<'_> {
    fn visit_block(&mut self, b: &'ast syn::Block) {
        self.statements(&b.stmts);
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        for a in &c.args {
            self.moved(a);
        }
        syn::visit::visit_expr_call(self, c);
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let method = m.method.to_string();
        if self.by_value.contains(&method) || method.starts_with("into_") {
            self.moved(&m.receiver);
        }
        for a in &m.args {
            self.moved(a);
        }
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        for f in &s.fields {
            self.moved(&f.expr);
        }
        syn::visit::visit_expr_struct(self, s);
    }

    fn visit_expr_tuple(&mut self, t: &'ast syn::ExprTuple) {
        for e in &t.elems {
            self.moved(e);
        }
        syn::visit::visit_expr_tuple(self, t);
    }

    fn visit_expr_array(&mut self, a: &'ast syn::ExprArray) {
        for e in &a.elems {
            self.moved(e);
        }
        syn::visit::visit_expr_array(self, a);
    }

    fn visit_expr_return(&mut self, r: &'ast syn::ExprReturn) {
        if let Some(e) = &r.expr {
            self.moved(e);
        }
        syn::visit::visit_expr_return(self, r);
    }

    fn visit_expr_assign(&mut self, a: &'ast syn::ExprAssign) {
        self.moved(&a.right);
        syn::visit::visit_expr_assign(self, a);
    }

    fn visit_expr_for_loop(&mut self, f: &'ast syn::ExprForLoop) {
        self.moved(&f.expr);
        syn::visit::visit_expr_for_loop(self, f);
    }

    fn visit_arm(&mut self, a: &'ast syn::Arm) {
        self.moved(&a.body);
        syn::visit::visit_arm(self, a);
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        // `vec![a, b]` moves its items; the formatting macros take references.
        if m.path.is_ident("vec") {
            if let Ok(items) = m.parse_body_with(
                syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated,
            ) {
                for e in &items {
                    self.moved(e);
                    self.visit_expr(e);
                }
            }
        }
    }
}

/// The pattern declares `name`.
fn binds(p: &Pat, name: &str) -> bool {
    match p {
        Pat::Ident(i) => i.ident == name,
        Pat::Type(t) => binds(&t.pat, name),
        Pat::Tuple(t) => t.elems.iter().any(|p| binds(p, name)),
        Pat::TupleStruct(t) => t.elems.iter().any(|p| binds(p, name)),
        Pat::Struct(s) => s.fields.iter().any(|f| binds(&f.pat, name)),
        Pat::Reference(r) => binds(&r.pat, name),
        _ => false,
    }
}
