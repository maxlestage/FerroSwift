//! Methods returning `&mut T`. Swift cannot return a mutable reference, so such a method
//! returns a `WritableKeyPath` from its type to the value, and the caller reads and writes
//! `receiver[keyPath: path]`.

use syn::{Expr, Pat, spanned::Spanned};

use crate::expr::{Ex, is_self};
use crate::stmt::result_ctor;
use crate::symbols::FnSig;
use crate::translate::{FnCx, Translator};
use crate::ty::{IntKind, Ty};

/// `(e)`, `&mut e`, `&e` → `e`.
fn strip(e: &Expr) -> &Expr {
    match e {
        Expr::Paren(p) => strip(&p.expr),
        Expr::Reference(r) => strip(&r.expr),
        _ => e,
    }
}

/// `self.items` is written `items` in a method: the key path starts after the root.
fn strip_self(place: &str) -> &str {
    place.strip_prefix("self.").unwrap_or(place)
}

impl Translator<'_> {
    /// The lines ending a method that returns a key path: the checks that the value exists,
    /// then `return \Bank.accounts[name]!`.
    pub fn key_path_return(&mut self, e: &Expr, root: &str, cx: &FnCx) -> Vec<String> {
        let e = strip(e);
        if let Expr::Path(p) = e {
            if p.path.is_ident("None") {
                return vec!["return nil".into()];
            }
        }
        if let Some((ctor, arg)) = result_ctor(e) {
            if ctor == "Err" {
                let err = cx.throws.clone().unwrap_or(Ty::Unknown);
                let v = self.expr_with(arg, &err);
                let v = self.settle(v);
                return vec![format!("throw {}", v.code)];
            }
            return self.definite_key_path_return(arg, root);
        }
        if let Expr::MethodCall(m) = e {
            let method = m.method.to_string();
            // `self.map.get_mut(&k).ok_or(e)`: a guard, then the path.
            if matches!(method.as_str(), "ok_or" | "ok_or_else") && m.args.len() == 1 {
                if let Some((pre, condition, path)) = self.optional_key_path(&m.receiver, root) {
                    let err_ty = cx.throws.clone().unwrap_or(Ty::Unknown);
                    let err = match (&m.args[0], method == "ok_or_else") {
                        (Expr::Closure(c), true) => {
                            let v = self.expr_with(&c.body, &err_ty);
                            self.settle(v).code
                        }
                        (f, true) => {
                            let v = self.expr(f);
                            format!("({})()", v.code)
                        }
                        (v, false) => {
                            let v = self.expr_with(v, &err_ty);
                            self.settle(v).code
                        }
                    };
                    let mut lines = pre;
                    lines.push(format!("guard {condition} else {{ throw {err} }}"));
                    lines.push(format!("return {path}"));
                    return lines;
                }
            }
        }
        if matches!(cx.ret, Ty::Option(_)) {
            if let Some((pre, condition, path)) = self.optional_key_path(e, root) {
                // Not a `?:` with `nil`: Swift 6 finds the key path ambiguous there.
                let mut lines = pre;
                lines.push(format!("guard {condition} else {{ return nil }}"));
                lines.push(format!("return {path}"));
                return lines;
            }
        }
        self.definite_key_path_return(e, root)
    }

    fn definite_key_path_return(&mut self, e: &Expr, root: &str) -> Vec<String> {
        match self.definite_key_path(e, root) {
            Some((mut lines, path)) => {
                lines.push(format!("return {path}"));
                lines
            }
            None => {
                let code =
                    self.unsupported(e.span(), "this value as a mutable reference to return");
                vec![format!("return {code}")]
            }
        }
    }

    /// `&mut self.items[i]`, `self.map.entry(k).or_insert(v)`, `v.last_mut().unwrap()` or a
    /// local standing for such a place: the statements making sure the value exists, and
    /// its key path.
    fn definite_key_path(&mut self, e: &Expr, root: &str) -> Option<(Vec<String>, String)> {
        let e = strip(e);
        match e {
            _ if is_self(e) => Some((vec![], format!("\\{root}.self"))),
            Expr::Path(p) => {
                let rust = p.path.get_ident()?.to_string();
                let (alias, swift) = self.lookup(&rust).map(|l| (l.alias, l.swift.clone()))?;
                if !alias {
                    return None;
                }
                Some(self.key_path_of_alias(&swift, root))
            }
            Expr::Field(_) | Expr::Index(_) => {
                let place = self.place(e);
                Some((vec![], key_path_of_place(&place, root)))
            }
            Expr::MethodCall(m) => {
                let method = m.method.to_string();
                if matches!(
                    method.as_str(),
                    "or_insert" | "or_insert_with" | "or_insert_with_key" | "or_default"
                ) {
                    let code = self.entry_place(m)?;
                    return Some(self.key_path_of_alias(&code, root));
                }
                if matches!(method.as_str(), "unwrap" | "expect") {
                    let (mut pre, condition, path) = self.optional_key_path(&m.receiver, root)?;
                    pre.push(format!("precondition({condition})"));
                    return Some((pre, path));
                }
                None
            }
            _ => None,
        }
    }

    /// A place written as Swift code: `accounts[name, default: Account()]` (an entry, which
    /// is inserted first) or `items[i]`.
    fn key_path_of_alias(&self, code: &str, root: &str) -> (Vec<String>, String) {
        if let Some((map, key, default)) = entry_parts(code) {
            let pre = vec![format!(
                "if {map}[{key}] == nil {{ {map}[{key}] = {default} }}"
            )];
            return (pre, format!("\\{root}.{}[{key}]!", strip_self(map)));
        }
        (vec![], key_path_of_place(code, root))
    }

    /// `v.get_mut(i)`, `map.get_mut(&k)`, `v.last_mut()`, `opt.as_mut()`,
    /// `v.iter_mut().find(…)`: the statements looking the element up, the condition under
    /// which it exists, and its key path.
    fn optional_key_path(&mut self, e: &Expr, root: &str) -> Option<(Vec<String>, String, String)> {
        let e = strip(e);
        let Expr::MethodCall(m) = e else { return None };
        let method = m.method.to_string();
        match method.as_str() {
            "get_mut" if m.args.len() == 1 => {
                let coll_ty = self.place_ty(&m.receiver);
                let coll = self.place(&m.receiver);
                let kp = key_path_of_place(&coll, root);
                match coll_ty {
                    Ty::Map(k, _, _) => {
                        let key = self.expr_with(&m.args[0], &k);
                        let key = self.settle(key).code;
                        Some((
                            vec![],
                            format!("{coll}[{key}] != nil"),
                            format!("{kp}[{key}]!"),
                        ))
                    }
                    Ty::Vec(_) => {
                        let i = self.expr_with(&m.args[0], &Ty::Int(IntKind::Usize));
                        let i = self.settle(i).code;
                        Some((
                            vec![],
                            format!("{coll}.indices.contains({i})"),
                            format!("{kp}[{i}]"),
                        ))
                    }
                    _ => None,
                }
            }
            "last_mut" | "first_mut" if m.args.is_empty() => {
                let coll = self.place(&m.receiver);
                let kp = key_path_of_place(&coll, root);
                let index = if method == "last_mut" {
                    format!("{coll}.count - 1")
                } else {
                    "0".into()
                };
                Some((vec![], format!("!{coll}.isEmpty"), format!("{kp}[{index}]")))
            }
            "as_mut" if m.args.is_empty() => {
                let coll = self.place(&m.receiver);
                let kp = key_path_of_place(&coll, root);
                Some((vec![], format!("{coll} != nil"), format!("{kp}!")))
            }
            // `self.rows.get_mut(y).and_then(|row| row.get_mut(x))`: the second lookup in
            // the place the first one found.
            "and_then" if m.args.len() == 1 => {
                let (pre, cond, kp) = self.optional_key_path(&m.receiver, root)?;
                let Expr::Closure(c) = &m.args[0] else {
                    return None;
                };
                let param = match c.inputs.first()? {
                    Pat::Ident(id) => id.ident.to_string(),
                    Pat::Type(pt) => match &*pt.pat {
                        Pat::Ident(id) => id.ident.to_string(),
                        _ => return None,
                    },
                    _ => return None,
                };
                let found = kp.strip_prefix(&format!("\\{root}."))?.to_string();
                let found_ty = match strip(&m.receiver) {
                    Expr::MethodCall(inner) => self.place_ty(&inner.receiver).item(),
                    _ => Ty::Unknown,
                };
                self.push_scope();
                self.declare_alias(&param, found, found_ty);
                let inner = self.optional_key_path(&c.body, root);
                self.pop_scope();
                let (mut inner_pre, inner_cond, inner_kp) = inner?;
                let mut all = pre;
                all.append(&mut inner_pre);
                Some((all, format!("{cond} && {inner_cond}"), inner_kp))
            }
            "find" if m.args.len() == 1 => {
                let Expr::MethodCall(inner) = &*m.receiver else {
                    return None;
                };
                if inner.method != "iter_mut" || !inner.args.is_empty() {
                    return None;
                }
                let coll_ty = self.place_ty(&inner.receiver);
                let coll = self.place(&inner.receiver);
                let kp = key_path_of_place(&coll, root);
                let item = coll_ty.item();
                let f = self.expr_with(&m.args[0], &Ty::Fn(vec![item], Box::new(Ty::Bool)));
                let f = self.settle(f);
                let i = self.fresh("index");
                Some((
                    vec![format!("let {i} = {coll}.firstIndex(where: {})", f.code)],
                    format!("{i} != nil"),
                    format!("{kp}[{i}!]"),
                ))
            }
            _ => None,
        }
    }

    /// A call of a method returning a key path (`self.get(name)`, `bank.open("ann", 1)`):
    /// the path is taken first, then `receiver[keyPath: path]` is the place. In raw mode the
    /// call itself is kept for the caller to bind (`if let path = …`).
    pub fn key_path_call(&mut self, call: String, sig: &FnSig, receiver: &str) -> Ex {
        if self.raw_key_path {
            self.key_path_raw = Some((call.clone(), receiver.to_string(), sig.clone()));
            return Ex::primary(call, Ty::Unknown);
        }
        let path = self.fresh("path");
        match &sig.ret {
            // As a value (`bank.get(k).map(…)`): a `Result` holding a copy of the element;
            // `?`, `unwrap()` and a `let` binding take the path itself.
            Ty::Result(ok, err) => Ex::primary(
                format!("Result {{ try {call} }}.map {{ {receiver}[keyPath: $0] }}"),
                Ty::Result(ok.clone(), err.clone()),
            ),
            Ty::Option(t) => {
                self.prelude(format!("let {path} = {call}"));
                Ex::primary(
                    format!("({path} == nil ? nil : {receiver}[keyPath: {path}!])"),
                    Ty::Option(t.clone()),
                )
            }
            t => {
                self.prelude(format!("let {path} = {call}"));
                Ex::primary(format!("{receiver}[keyPath: {path}]"), t.clone())
            }
        }
    }

    /// Whether `m` calls a method of the crate that returns a key path.
    pub fn is_key_path_call(&mut self, m: &syn::ExprMethodCall) -> bool {
        self.key_path_sig(m).is_some()
    }

    /// The signature of the key-path method that `m` calls, if any.
    pub fn key_path_sig(&mut self, m: &syn::ExprMethodCall) -> Option<FnSig> {
        let recv_ty = if is_self(&m.receiver) {
            self.self_ty()
        } else {
            self.place_ty(&m.receiver)
        };
        self.user_method(&recv_ty, &m.method.to_string())
            .map(|(sig, _)| sig)
            .filter(|sig| sig.key_path)
    }

    /// The call of a key-path method as plain Swift code, with the receiver it is applied
    /// to and the signature: (`get(name)`, `self`, sig).
    pub fn raw_key_path_call(
        &mut self,
        m: &syn::ExprMethodCall,
    ) -> Option<(String, String, FnSig)> {
        if !self.is_key_path_call(m) {
            return None;
        }
        self.raw_key_path = true;
        self.key_path_raw = None;
        let _ = self.method_call(m, &Ty::Unknown);
        self.raw_key_path = false;
        self.key_path_raw.take()
    }
}

/// The parts of an entry place `map[key, default: value]`: (`map`, `key`, `value`).
pub fn entry_parts(code: &str) -> Option<(&str, &str, &str)> {
    let (head, default) = code.rsplit_once(", default: ")?;
    let default = default.strip_suffix(']')?;
    let (map, key) = head.rsplit_once('[')?;
    Some((map, key, default))
}

/// `items[i]` → `\Bank.items[i]`, `self` → `\Bank.self`.
fn key_path_of_place(place: &str, root: &str) -> String {
    if place == "self" {
        return format!("\\{root}.self");
    }
    format!("\\{root}.{}", strip_self(place))
}
