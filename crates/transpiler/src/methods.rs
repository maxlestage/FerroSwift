//! Calls: functions, constructors, methods of the crate's types, and the methods of the
//! standard library (`Vec`, `String`, `HashMap`, `Option`, iterators…) mapped to Swift's.

use syn::spanned::Spanned;
use syn::{Expr, ExprCall, ExprMethodCall};

use crate::expr::{Ex, P_ADD, P_CMP, P_MUL, P_NIL, P_PRIMARY, P_TERNARY, P_TRY, wrap};
use crate::names;
use crate::symbols::{FnSig, ParamInfo, SelfKind, TypeKind, VariantShape};
use crate::translate::Translator;
use crate::ty::{FloatKind, IntKind, Ty};

impl Translator<'_> {
    // ----- Function calls -----

    pub fn call(&mut self, c: &ExprCall, expected: &Ty) -> Ex {
        let Expr::Path(p) = &*c.func else {
            let f = self.expr(&c.func);
            let args = self.call_args(&c.args, None);
            let ret = match &f.ty {
                Ty::Fn(_, r) => (**r).clone(),
                _ => Ty::Unknown,
            };
            return Ex::primary(format!("{}({})", wrap(&f, P_PRIMARY), args.join(", ")), ret);
        };
        let segs: Vec<String> = p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let last = segs.last().cloned().unwrap_or_default();
        let turbofish = self.generic_args(&p.path);
        if last == "Reverse"
            && c.args.len() == 1
            && (segs.len() == 1 || segs[0] == "std" || segs[0] == "cmp")
        {
            self.helper("reverse");
            let v = self.expr(&c.args[0]);
            let v = self.settle(v);
            // `Reverse((d, node))`: the tuple made comparable.
            let code = match crate::ty::ord_tuple_arity(&v.ty) {
                Some(n) => format!("RsReverse(RsOrd{n}({}))", v.code),
                None => format!("RsReverse({})", v.code),
            };
            return Ex::primary(code, Ty::Named("RsReverse".into(), vec![v.ty]));
        }
        if segs.len() == 1 {
            match last.as_str() {
                "Some" if c.args.len() == 1 => {
                    let inner = match expected {
                        Ty::Option(t) => (**t).clone(),
                        _ => Ty::Unknown,
                    };
                    let v = self.expr_with(&c.args[0], &inner);
                    let v = self.settle(v);
                    let ty = Ty::Option(Box::new(v.ty.clone()));
                    // `Some(None)` as an `Option<Option<T>>`: `.some(nil)`, not `nil`.
                    let nested = v.code == "nil" || matches!(v.ty, Ty::Option(_));
                    if matches!(expected, Ty::Option(_)) && !nested {
                        return Ex { ty, ..v };
                    }
                    if matches!(expected, Ty::Option(_)) && nested {
                        return Ex::primary(format!(".some({})", v.code), ty);
                    }
                    return Ex::primary(format!("Optional({})", v.code), ty);
                }
                "Ok" | "Err" if c.args.len() == 1 => {
                    let (ok, err) = match expected {
                        Ty::Result(o, e) => ((**o).clone(), (**e).clone()),
                        _ => (Ty::Unknown, Ty::Unknown),
                    };
                    let (case, arg_ty) = if last == "Ok" {
                        ("success", &ok)
                    } else {
                        ("failure", &err)
                    };
                    let v = self.expr_with(&c.args[0], arg_ty);
                    let v = self.settle(v);
                    let ty = if last == "Ok" {
                        Ty::Result(Box::new(v.ty.clone()), Box::new(err.clone()))
                    } else {
                        Ty::Result(Box::new(ok.clone()), Box::new(v.ty.clone()))
                    };
                    let code = if v.code == "()" && last == "Ok" {
                        format!(".{case}(())")
                    } else {
                        format!(".{case}({})", v.code)
                    };
                    return Ex::primary(code, ty);
                }
                _ => {}
            }
            if let Some(local) = self.lookup(&last).cloned() {
                // A closure in a local variable.
                let ty = self.local_ty(&local);
                let params: Option<Vec<ParamInfo>> = match &ty {
                    Ty::Fn(ps, _) => Some(
                        ps.iter()
                            .map(|t| ParamInfo {
                                ty: t.clone(),
                                inout: false,
                            })
                            .collect(),
                    ),
                    _ => None,
                };
                let args = self.call_args(&c.args, params.as_deref());
                let ret = match &ty {
                    Ty::Fn(_, r) => (**r).clone(),
                    _ => Ty::Unknown,
                };
                return Ex::primary(format!("{}({})", local.swift, args.join(", ")), ret);
            }
            if let Some(st) = self.syms.structs.get(&last).cloned() {
                if st.tuple {
                    let mut bound = std::collections::HashMap::new();
                    let args: Vec<String> = c
                        .args
                        .iter()
                        .enumerate()
                        .map(|(i, a)| {
                            let t = st.fields.get(i).map_or(Ty::Unknown, |f| f.ty.clone());
                            let v = self.expr_with(a, &t);
                            let v = self.settle(v);
                            unify(&t, &v.ty, &st.generics, &mut bound);
                            v.code
                        })
                        .collect();
                    // `Pair("x", 1.5)` is a `Pair<String, Double>`.
                    let type_args: Vec<Ty> = st
                        .generics
                        .iter()
                        .map(|g| bound.get(g).cloned().unwrap_or(Ty::Unknown))
                        .collect();
                    return Ex::primary(
                        format!("{}({})", st.swift, args.join(", ")),
                        Ty::Named(st.swift.clone(), type_args),
                    );
                }
            }
            if let Some(mut sig) = self.syms.fns.get(&last).cloned() {
                // `generic_max::<u8>(&[])`: the turbofish fixes the type parameters.
                if !turbofish.is_empty() && turbofish.len() == sig.generics.len() {
                    let map: std::collections::HashMap<String, Ty> = sig
                        .generics
                        .iter()
                        .cloned()
                        .zip(turbofish.iter().cloned())
                        .collect();
                    for p in &mut sig.params {
                        p.ty = crate::symbols::substitute(&p.ty, &map);
                    }
                    sig.ret = crate::symbols::substitute(&sig.ret, &map);
                }
                if !sig.generics.is_empty() {
                    self.generic_call += 1;
                }
                let ex = self.sig_call(names::lower_camel(&last), &sig, &c.args);
                if !sig.generics.is_empty() {
                    self.generic_call -= 1;
                }
                return ex;
            }
            // A tuple variant imported with `use Enum::*`.
            let found: Vec<(String, String)> = self
                .syms
                .enums
                .values()
                .filter_map(|e| e.variant(&last).map(|v| (e.swift.clone(), v.swift.clone())))
                .collect();
            if found.len() == 1 {
                let (en, case) = &found[0];
                let args = self.variant_args(en, &last, &c.args);
                let ty = Ty::Named(en.clone(), vec![]);
                let prefix = if expected.named() == Some(en.as_str()) {
                    String::new()
                } else {
                    en.clone()
                };
                return Ex::primary(format!("{prefix}.{case}({})", args.join(", ")), ty);
            }
            if let Some(ex) = self.ui_call(&last, &c.args, expected) {
                return ex;
            }
            if let Some(ex) = self.std_free_call(&segs, &c.args, expected) {
                return ex;
            }
            let args = self.call_args(&c.args, None);
            return Ex::primary(
                format!("{}({})", names::lower_camel(&last), args.join(", ")),
                Ty::Unknown,
            );
        }
        // `Type::function(…)`, `Enum::Variant(…)`, `module::function(…)`.
        let owner = segs[segs.len() - 2].clone();
        let owner_rust = self.rust_type_name(&owner);
        if let Some(en) = self.syms.enums.get(&owner_rust).cloned() {
            if let Some(v) = en.variant(&last) {
                let args = self.variant_args(&owner_rust, &last, &c.args);
                let ty = Ty::Named(en.swift.clone(), vec![]);
                let prefix = if expected.named() == Some(en.swift.as_str()) {
                    String::new()
                } else {
                    self.type_path_name(&owner)
                };
                return Ex::primary(format!("{prefix}.{}({})", v.swift, args.join(", ")), ty);
            }
        }
        if let Some(sig) = self
            .syms
            .methods
            .get(&owner_rust)
            .and_then(|m| m.get(&last))
            .cloned()
        {
            let tname = self.type_path_name(&owner);
            if sig.is_init {
                let mut ex = self.sig_call(tname.clone(), &sig, &c.args);
                if let Ty::Result(_, e) = &sig.ret {
                    let self_ty = Ty::Named(self.struct_or_enum_swift(&owner_rust), vec![]);
                    ex.throws = Some(Box::new((self_ty, (**e).clone())));
                    ex.ty = Ty::Unknown;
                } else if let Ty::Option(_) = &sig.ret {
                    ex.ty = Ty::Option(Box::new(Ty::Named(
                        self.struct_or_enum_swift(&owner_rust),
                        vec![],
                    )));
                } else {
                    let n = self
                        .syms
                        .structs
                        .get(&owner_rust)
                        .map_or(0, |s| s.generics.len());
                    ex.ty = match expected {
                        Ty::Named(e, args)
                            if *e == self.struct_or_enum_swift(&owner_rust) && args.len() == n =>
                        {
                            expected.clone()
                        }
                        _ => {
                            Ty::Named(self.struct_or_enum_swift(&owner_rust), vec![Ty::Unknown; n])
                        }
                    };
                }
                return ex;
            }
            if sig.self_kind != SelfKind::None && !c.args.is_empty() {
                // `Type::method(&x, …)` → `x.method(…)`.
                let recv = self.expr(&c.args[0]);
                let recv = self.settle(recv);
                let rest: Vec<Expr> = c.args.iter().skip(1).cloned().collect();
                let rest: syn::punctuated::Punctuated<Expr, syn::token::Comma> =
                    rest.into_iter().collect();
                return self.sig_call(
                    format!("{}.{}", wrap(&recv, P_PRIMARY), names::lower_camel(&last)),
                    &sig,
                    &rest,
                );
            }
            let mut ex = self.sig_call(
                format!("{tname}.{}", names::lower_camel(&last)),
                &sig,
                &c.args,
            );
            if ex.ty == Ty::Named("Self".into(), vec![]) {
                ex.ty = Ty::Named(self.struct_or_enum_swift(&owner_rust), vec![]);
            }
            return ex;
        }
        if let Some(st) = self.syms.structs.get(&owner_rust).cloned() {
            if last == "default" {
                return Ex::primary(
                    format!("{}()", self.type_path_name(&owner)),
                    Ty::Named(st.swift.clone(), vec![]),
                );
            }
        }
        if self.syms.enums.contains_key(&owner_rust) && last == "default" {
            return Ex::primary(
                format!("{}()", self.type_path_name(&owner)),
                Ty::Named(self.struct_or_enum_swift(&owner_rust), vec![]),
            );
        }
        if let Some(ex) = self.std_path_call(&segs, &c.args, expected, &turbofish) {
            return ex;
        }
        if let Some(ex) = self.ui_call(&last, &c.args, expected) {
            return ex;
        }
        // A function of another module: modules disappear.
        if let Some(sig) = self.syms.fns.get(&last).cloned() {
            return self.sig_call(names::lower_camel(&last), &sig, &c.args);
        }
        if self.syms.is_user_type(&owner_rust) || owner == "Self" || self.generics.contains(&owner)
        {
            let args = self.call_args(&c.args, None);
            return Ex::primary(
                format!(
                    "{}.{}({})",
                    self.type_path_name(&owner),
                    names::lower_camel(&last),
                    args.join(", ")
                ),
                Ty::Unknown,
            );
        }
        let args = self.call_args(&c.args, None);
        self.warn(
            c.span(),
            format!("unknown function `{}`: translated as is", segs.join("::")),
        );
        Ex::primary(
            format!("{}({})", names::lower_camel(&last), args.join(", ")),
            Ty::Unknown,
        )
    }

    fn struct_or_enum_swift(&self, rust: &str) -> String {
        self.syms
            .structs
            .get(rust)
            .map(|s| s.swift.clone())
            .or_else(|| self.syms.enums.get(rust).map(|e| e.swift.clone()))
            .unwrap_or_else(|| names::upper_camel(rust))
    }

    fn variant_args(
        &mut self,
        enum_rust: &str,
        variant: &str,
        args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
    ) -> Vec<String> {
        let tys: Vec<Ty> = self
            .syms
            .enums
            .get(enum_rust)
            .or_else(|| self.syms.enums.values().find(|e| e.swift == enum_rust))
            .and_then(|e| e.variant(variant))
            .map(|v| match &v.shape {
                VariantShape::Tuple(ts) => ts.clone(),
                _ => vec![],
            })
            .unwrap_or_default();
        args.iter()
            .enumerate()
            .map(|(i, a)| {
                let v = self.expr_with(a, tys.get(i).unwrap_or(&Ty::Unknown));
                self.settle(v).code
            })
            .collect()
    }

    /// A call to a function of the crate, whose signature is known.
    pub fn sig_call(
        &mut self,
        callee: String,
        sig: &FnSig,
        args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
    ) -> Ex {
        let (codes, _) = self.call_args_typed(args, Some(&sig.params));
        let mut ex = Ex::primary(format!("{callee}({})", codes.join(", ")), sig.ret.clone());
        if let Ty::Result(ok, err) = &sig.ret {
            if !sig.is_init {
                ex.throws = Some(Box::new(((**ok).clone(), (**err).clone())));
                ex.ty = (**ok).clone();
            }
        }
        if matches!(sig.ret, Ty::Iter(_)) {
            ex.ty = Ty::Vec(Box::new(sig.ret.item()));
        }
        ex
    }

    /// The value of a call to a function of the crate: a throwing call for a `Result`.
    fn sig_result(&self, code: String, sig: &FnSig) -> Ex {
        let mut ex = Ex::primary(code, sig.ret.clone());
        if let Ty::Result(ok, err) = &sig.ret {
            if !sig.is_init {
                ex.throws = Some(Box::new(((**ok).clone(), (**err).clone())));
                ex.ty = (**ok).clone();
            }
        }
        if matches!(sig.ret, Ty::Iter(_)) {
            ex.ty = Ty::Vec(Box::new(sig.ret.item()));
        }
        ex
    }

    /// Call arguments: `&mut x` for an `inout` parameter becomes `&x`, other references
    /// disappear.
    pub fn call_args(
        &mut self,
        args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        params: Option<&[ParamInfo]>,
    ) -> Vec<String> {
        self.call_args_typed(args, params).0
    }

    /// Like [`call_args`](Self::call_args), with the types of the arguments.
    pub fn call_args_typed(
        &mut self,
        args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        params: Option<&[ParamInfo]>,
    ) -> (Vec<String>, Vec<Ty>) {
        let mut out = Vec::new();
        let mut tys = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let param = params.and_then(|p| p.get(i));
            let inout = param.is_some_and(|p| p.inout);
            let expected = param.map_or(Ty::Unknown, |p| p.ty.clone());
            // `generic_max::<u8>(&[])`: Swift cannot infer `T` from an empty literal.
            let cast_empty = self.generic_call > 0
                && matches!(expected, Ty::Vec(_))
                && expected.is_complete()
                && matches!(a, Expr::Reference(_) | Expr::Array(_) | Expr::Macro(_));
            if inout {
                let inner = match a {
                    Expr::Reference(r) => &*r.expr,
                    e => e,
                };
                let ty = self.place_ty(inner);
                let place = self.place(inner);
                // `let mut out = Vec::new(); fill(&mut out)`: the parameter tells the type.
                if let (Some(p), Expr::Path(path)) = (param, inner) {
                    if let Some(id) = path.path.get_ident() {
                        if p.ty.is_complete() {
                            self.refine(&id.to_string(), &p.ty);
                        }
                    }
                }
                out.push(format!("&{place}"));
                tys.push(ty);
                continue;
            }
            let v = self.expr_with(a, &expected);
            let v = self.settle(v);
            if cast_empty && v.code == "[]" {
                out.push(format!("[] as {}", expected.swift_lossy()));
            } else {
                out.push(v.code);
            }
            tys.push(v.ty);
        }
        (out, tys)
    }

    /// `max(a, b)`, `drop(x)`… called without a path.
    fn std_free_call(
        &mut self,
        segs: &[String],
        args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        expected: &Ty,
    ) -> Option<Ex> {
        let last = segs.last()?.as_str();
        match last {
            "max" | "min" if args.len() == 2 => {
                let a = self.expr_with(&args[0], expected);
                let a = self.settle(a);
                let b = self.expr_with(&args[1], &a.ty);
                let b = self.settle(b);
                let f = self.global_fn(last);
                Some(Ex::primary(format!("{f}({}, {})", a.code, b.code), a.ty))
            }
            "drop" if args.len() == 1 => Some(Ex::primary("", Ty::Unit)),
            "swap" | "take" | "replace" => {
                self.std_path_call(&["mem".into(), last.into()], args, expected, &[])
            }
            _ => {
                // `use std::iter::once;` then `once(x)`.
                let full = self.syms.imports.get(last).cloned()?;
                if full.len() < 2 || full[0] != "std" {
                    return None;
                }
                self.std_path_call(&full, args, expected, &[])
            }
        }
    }

    /// `String::new()`, `Vec::with_capacity(n)`, `HashMap::from(…)`, `i64::from(x)`…
    fn std_path_call(
        &mut self,
        segs: &[String],
        args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        expected: &Ty,
        turbofish: &[Ty],
    ) -> Option<Ex> {
        let last = segs.last()?.as_str();
        let owner = segs[segs.len() - 2].as_str();
        let arg = |t: &mut Self, i: usize, exp: &Ty| -> Ex {
            let v = t.expr_with(&args[i], exp);
            t.settle(v)
        };
        match (owner, last) {
            ("String", "new" | "with_capacity") => Some(Ex::primary("\"\"", Ty::Str)),
            ("String", "from" | "from_str") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Str);
                if v.ty == Ty::Char {
                    return Some(Ex::primary(format!("String({})", v.code), Ty::Str));
                }
                Some(Ex { ty: Ty::Str, ..v })
            }
            ("String", "from_utf8") if args.len() == 1 => {
                self.helper("utf8");
                let v = arg(self, 0, &Ty::Vec(Box::new(Ty::Int(IntKind::U8))));
                Some(Ex::primary(
                    format!("rsFromUtf8({})", v.code),
                    Ty::Result(
                        Box::new(Ty::Str),
                        Box::new(Ty::Named("RsFromUtf8Error".into(), vec![])),
                    ),
                ))
            }
            ("String", "from_utf8_lossy") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Unknown);
                Some(Ex::primary(
                    format!("String(decoding: {}, as: UTF8.self)", v.code),
                    Ty::Str,
                ))
            }
            ("BinaryHeap", "new" | "with_capacity") => {
                self.helper("heap");
                let item = turbofish
                    .first()
                    .cloned()
                    .unwrap_or_else(|| match expected {
                        Ty::Named(n, args) if n == "RsBinaryHeap" => {
                            args.first().cloned().unwrap_or(Ty::Unknown)
                        }
                        _ => Ty::Unknown,
                    });
                Some(Ex::primary(
                    "RsBinaryHeap()",
                    Ty::Named("RsBinaryHeap".into(), vec![item]),
                ))
            }
            ("BinaryHeap", "from" | "from_iter") if args.len() == 1 => {
                self.helper("heap");
                let v = arg(self, 0, &Ty::Unknown);
                let item = v.ty.item();
                let values = match crate::ty::ord_tuple_arity(&item) {
                    Some(n) => format!("{}.map {{ RsOrd{n}($0) }}", v.code),
                    None if v.code.starts_with('[') || matches!(v.ty, Ty::Vec(_)) => v.code,
                    None => format!("Array({})", v.code),
                };
                let code = format!("RsBinaryHeap({values})");
                Some(Ex::primary(
                    code,
                    Ty::Named("RsBinaryHeap".into(), vec![item]),
                ))
            }
            ("Vec" | "VecDeque", "new" | "with_capacity") => {
                let item = turbofish
                    .first()
                    .cloned()
                    .unwrap_or_else(|| expected.item());
                Some(Ex::primary("[]", Ty::Vec(Box::new(item))))
            }
            ("Vec" | "VecDeque", "from") if args.len() == 1 => {
                let v = arg(self, 0, expected);
                Some(Ex {
                    ty: Ty::Vec(Box::new(v.ty.item())),
                    ..v
                })
            }
            ("HashMap" | "BTreeMap", "new" | "with_capacity") => {
                let ordered = owner == "BTreeMap";
                let ty = match expected {
                    Ty::Map(k, v, _) => Ty::Map(k.clone(), v.clone(), ordered),
                    _ => Ty::Map(Box::new(Ty::Unknown), Box::new(Ty::Unknown), ordered),
                };
                Some(Ex::primary("[:]", ty))
            }
            ("HashSet" | "BTreeSet", "new" | "with_capacity") => {
                let ordered = owner == "BTreeSet";
                let ty = match expected {
                    Ty::Set(t, _) => Ty::Set(t.clone(), ordered),
                    _ => Ty::Set(Box::new(Ty::Unknown), ordered),
                };
                Some(Ex::primary("[]", ty))
            }
            ("HashSet" | "BTreeSet", "from" | "from_iter") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Unknown);
                let item = v.ty.item();
                Some(Ex::primary(
                    format!("{}({})", crate::ty::set_type_name(&item), v.code),
                    Ty::Set(Box::new(item), owner == "BTreeSet"),
                ))
            }
            ("HashMap" | "BTreeMap", "from" | "from_iter") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Unknown);
                let (k, val) = match v.ty.item() {
                    Ty::Tuple(ts) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
                    _ => (Ty::Unknown, Ty::Unknown),
                };
                Some(Ex::primary(
                    format!(
                        "{}({}, uniquingKeysWith: {{ _, last in last }})",
                        crate::ty::map_type_name(&k),
                        v.code
                    ),
                    Ty::Map(Box::new(k), Box::new(val), owner == "BTreeMap"),
                ))
            }
            // A cell is a class holding the value: the clones of an `Rc` share it.
            ("RefCell" | "Cell" | "Mutex" | "RwLock", "new") if args.len() == 1 => {
                self.helper("refcell");
                let inner_expected = match expected {
                    Ty::Named(n, a) if n == "RsRefCell" => {
                        a.first().cloned().unwrap_or(Ty::Unknown)
                    }
                    _ => Ty::Unknown,
                };
                let v = arg(self, 0, &inner_expected);
                let kind = if owner == "RefCell" {
                    String::new()
                } else {
                    format!(", kind: \"{owner}\"")
                };
                // `RefCell::new(Vec::new())`: an empty literal says nothing of the type.
                let generic = match v.ty.swift() {
                    Some(t) if matches!(v.code.as_str(), "[]" | "[:]" | "nil") => format!("<{t}>"),
                    _ => String::new(),
                };
                Some(Ex::primary(
                    format!("RsRefCell{generic}({}{kind})", v.code),
                    Ty::Named("RsRefCell".into(), vec![v.ty]),
                ))
            }
            ("Box" | "Rc" | "Arc" | "Cow", "new" | "from") if args.len() == 1 => {
                Some(arg(self, 0, expected))
            }
            // `Rc::clone(&x)`: the same cell, or a copy of a plain value.
            ("Rc" | "Arc", "clone") if args.len() == 1 => Some(arg(self, 0, expected)),
            ("Rc" | "Arc", "ptr_eq") if args.len() == 2 => {
                let a = arg(self, 0, &Ty::Unknown);
                let b = arg(self, 1, &a.ty);
                let shared = matches!(&a.ty, Ty::Named(n, _) if n == "RsRefCell")
                    || self.is_recursive_class(&a.ty);
                if !shared {
                    self.warn(
                        args.span(),
                        "`Rc::ptr_eq` compares the values here: Swift copies them",
                    );
                }
                let op = if shared { "===" } else { "==" };
                Some(Ex::new(
                    format!("{} {op} {}", a.code, b.code),
                    P_CMP,
                    Ty::Bool,
                ))
            }
            ("cmp", "max" | "min") | ("std", "max" | "min") if args.len() == 2 => {
                let a = arg(self, 0, expected);
                let b = arg(self, 1, &a.ty);
                let f = self.global_fn(last);
                Some(Ex::primary(format!("{f}({}, {})", a.code, b.code), a.ty))
            }
            ("mem", "swap") if args.len() == 2 => {
                let a = self.place(strip_ref(&args[0]));
                let b = self.place(strip_ref(&args[1]));
                Some(Ex::primary(format!("swap(&{a}, &{b})"), Ty::Unit))
            }
            ("mem", "take") if args.len() == 1 => {
                self.helper("mem");
                let ty = self.place_ty(strip_ref(&args[0]));
                let a = self.place(strip_ref(&args[0]));
                let default = ty.default_value().unwrap_or_else(|| "fatalError()".into());
                Some(Ex::primary(format!("rsReplace(&{a}, {default})"), ty))
            }
            ("mem", "replace") if args.len() == 2 => {
                self.helper("mem");
                let ty = self.place_ty(strip_ref(&args[0]));
                let a = self.place(strip_ref(&args[0]));
                let b = arg(self, 1, &ty);
                Some(Ex::primary(format!("rsReplace(&{a}, {})", b.code), ty))
            }
            ("mem", "drop") => Some(Ex::primary("", Ty::Unit)),
            ("process", "exit") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Int(IntKind::I32));
                Some(Ex::primary(format!("exit(Int32({}))", v.code), Ty::Never))
            }
            ("env", "args") => Some(Ex::primary(
                "CommandLine.arguments",
                Ty::Vec(Box::new(Ty::Str)),
            )),
            ("char", "from") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Int(IntKind::U8));
                Some(Ex::primary(
                    format!("Character(UnicodeScalar({}))", v.code),
                    Ty::Char,
                ))
            }
            ("char", "from_u32") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Int(IntKind::U32));
                Some(Ex::primary(
                    format!("UnicodeScalar({}).map(Character.init)", v.code),
                    Ty::Option(Box::new(Ty::Char)),
                ))
            }
            ("char", "from_digit") if args.len() == 2 => {
                self.helper("char");
                let d = arg(self, 0, &Ty::Int(IntKind::U32));
                let r = arg(self, 1, &Ty::Int(IntKind::U32));
                Some(Ex::primary(
                    format!("Character.fromDigit({}, {})", d.code, r.code),
                    Ty::Option(Box::new(Ty::Char)),
                ))
            }
            (o, "from")
                if args.len() == 1 && (IntKind::parse(o).is_some() || o == "f64" || o == "f32") =>
            {
                let t = crate::ty::from_name(o, vec![]);
                let v = arg(self, 0, &Ty::Unknown);
                if v.ty.swift() == t.swift() {
                    return Some(v);
                }
                let ts = t.swift_lossy();
                if v.ty == Ty::Char {
                    self.helper("char");
                    return Some(Ex::primary(
                        format!("{ts}({}.rsCode)", wrap(&v, P_PRIMARY)),
                        t,
                    ));
                }
                if v.ty == Ty::Bool {
                    return Some(Ex::primary(format!("({} ? 1 : 0)", v.code), t));
                }
                Some(Ex::primary(format!("{ts}({})", v.code), t))
            }
            ("char", "try_from") if args.len() == 1 => {
                self.helper("parse");
                let v = arg(self, 0, &Ty::Int(IntKind::U32));
                Some(Ex::primary(
                    format!("rsCharTryFrom({})", v.code),
                    Ty::Result(
                        Box::new(Ty::Char),
                        Box::new(Ty::Named("RsCharTryFromError".into(), vec![])),
                    ),
                ))
            }
            (o, "try_from") if args.len() == 1 && IntKind::parse(o).is_some() => {
                self.helper("try_from");
                let t = crate::ty::from_name(o, vec![]);
                let v = arg(self, 0, &Ty::Unknown);
                let bits = IntKind::parse(o)
                    .and_then(IntKind::narrow_bits)
                    .map(|b| format!(", bits: {b}"))
                    .unwrap_or_default();
                Some(Ex::primary(
                    format!("rsTryFrom({}.self, {}{bits})", t.swift_lossy(), v.code),
                    Ty::Result(
                        Box::new(t),
                        Box::new(Ty::Named("RsTryFromIntError".into(), vec![])),
                    ),
                ))
            }
            (o, "from_str_radix") if args.len() == 2 && IntKind::parse(o).is_some() => {
                self.helper("parse");
                let t = crate::ty::from_name(o, vec![]);
                let s = arg(self, 0, &Ty::Str);
                let r = arg(self, 1, &Ty::Int(IntKind::U32));
                let radix = if r.weak {
                    r.code
                } else {
                    format!("Int({})", r.code)
                };
                let bits = IntKind::parse(o)
                    .and_then(IntKind::narrow_bits)
                    .map(|b| format!(", bits: {b}"))
                    .unwrap_or_default();
                Some(Ex::primary(
                    format!(
                        "rsParseInt({}, {}.self, radix: {radix}{bits})",
                        s.code,
                        t.swift_lossy()
                    ),
                    Ty::Result(
                        Box::new(t),
                        Box::new(Ty::Named("RsParseIntError".into(), vec![])),
                    ),
                ))
            }
            (o, "pow" | "abs" | "max" | "min" | "sqrt")
                if IntKind::parse(o).is_some() || o == "f64" || o == "f32" =>
            {
                // `i32::pow(x, 2)` → `x.pow(2)`.
                if args.is_empty() {
                    return None;
                }
                let recv = arg(self, 0, &crate::ty::from_name(o, vec![]));
                let rest: Vec<Ex> = (1..args.len())
                    .map(|i| arg(self, i, &Ty::Unknown))
                    .collect();
                Some(self.std_method(recv, last, rest, &[], expected, args.span()))
            }
            (_, "default") => {
                let t = if owner == "Default" {
                    expected.clone()
                } else {
                    crate::ty::from_name(owner, vec![])
                };
                match t.default_value() {
                    Some(v) => Some(Ex::primary(v, t)),
                    None if self.generics.iter().any(|g| g == owner) => {
                        Some(Ex::primary(format!("{owner}()"), t))
                    }
                    None => {
                        self.error(args.span(), "Default::default(): the type is unknown here, write it (`T::default()`)");
                        Some(Ex::primary("fatalError()", Ty::Unknown))
                    }
                }
            }
            ("Instant", "now") => Some(Ex::primary("Date()", Ty::Named("Date".into(), vec![]))),
            ("iter", "successors") if args.len() == 2 => {
                self.helper("iter");
                let first = arg(self, 0, &Ty::Unknown);
                let item = first.ty.item();
                let f = arg(
                    self,
                    1,
                    &Ty::Fn(
                        vec![item.clone()],
                        Box::new(Ty::Option(Box::new(item.clone()))),
                    ),
                );
                Some(Ex::primary(
                    format!("rsSuccessors({}, {})", first.code, f.code),
                    Ty::Iter(Box::new(item)),
                ))
            }
            ("iter", "repeat") if args.len() == 1 => {
                self.helper("iter");
                let v = arg(self, 0, &expected.item());
                Some(Ex::primary(
                    format!("rsRepeat({})", v.code),
                    Ty::Iter(Box::new(v.ty)),
                ))
            }
            ("iter", "repeat_n") if args.len() == 2 => {
                let v = arg(self, 0, &expected.item());
                let n = arg(self, 1, &Ty::Int(IntKind::Usize));
                Some(Ex::primary(
                    format!("repeatElement({}, count: {})", v.code, n.code),
                    Ty::Iter(Box::new(v.ty)),
                ))
            }
            ("iter", "once") if args.len() == 1 => {
                let v = arg(self, 0, &expected.item());
                Some(Ex::primary(
                    format!("[{}]", v.code),
                    Ty::Vec(Box::new(v.ty)),
                ))
            }
            ("iter", "empty") if args.is_empty() => {
                Some(Ex::primary("[]", Ty::Vec(Box::new(expected.item()))))
            }
            _ => None,
        }
    }

    // ----- Method calls -----

    pub fn method_call(&mut self, m: &ExprMethodCall, expected: &Ty) -> Ex {
        let name = m.method.to_string();
        if let Some(ex) = self.ui_method_chain(m, expected) {
            return ex;
        }
        if name == "into" && m.args.is_empty() {
            return self.convert_into(m, expected);
        }
        if name == "try_into" && m.args.is_empty() {
            return self.convert_try_into(m, expected);
        }
        // `v.chunks_exact(n).remainder()`: what the chunks leave out.
        if name == "remainder" && m.args.is_empty() {
            if let Expr::MethodCall(inner) = &*m.receiver {
                if inner.method == "chunks_exact" && inner.args.len() == 1 {
                    let v = self.expr(&inner.receiver);
                    let v = self.settle(v);
                    let n = self.expr_with(&inner.args[0], &Ty::Int(IntKind::Usize));
                    let n = self.settle(n);
                    let c = wrap(&v, P_PRIMARY);
                    return Ex::primary(
                        format!("Array({c}.dropFirst({c}.count - {c}.count % {}))", n.code),
                        v.ty.clone(),
                    );
                }
            }
        }
        if matches!(name.as_str(), "unwrap" | "expect") {
            // `v.last_mut().unwrap().done = true`: the element itself is the place.
            if let Expr::MethodCall(inner) = &*m.receiver {
                if let Some((place, ty)) = self.mutable_place_forced(inner) {
                    return Ex::primary(place, ty);
                }
            }
            // `m.lock().unwrap()`, `c.try_borrow_mut().unwrap()`: the value of the cell.
            if let Expr::MethodCall(inner) = &*m.receiver {
                if inner.args.is_empty() && is_cell_access(&inner.method.to_string()) {
                    if let Ty::Named(n, targs) = self.place_ty(&inner.receiver) {
                        if n == "RsRefCell" {
                            let value = targs.first().cloned().unwrap_or(Ty::Unknown);
                            let cell = self.expr(&inner.receiver);
                            let cell = self.settle(cell);
                            return Ex::primary(format!("{}.value", wrap(&cell, P_PRIMARY)), value);
                        }
                    }
                }
            }
            // `write!(out, …).unwrap()`: writing to a `String` cannot fail.
            if let Expr::Macro(mac) = &*m.receiver {
                if matches!(
                    crate::macros::macro_name(&mac.mac).as_str(),
                    "write" | "writeln"
                ) {
                    return self.macro_expr(&mac.mac, &Ty::Unit);
                }
            }
        }
        let collect_target = if name == "collect" {
            // `collect::<Option<Vec<_>>>()` with a `Vec<u64>` expected: both say a part.
            let mut target = self
                .method_turbofish(m)
                .first()
                .cloned()
                .filter(|t| !t.is_unknown())
                .unwrap_or_else(|| expected.clone());
            target.merge(expected);
            target
        } else {
            Ty::Unknown
        };
        // `"42".parse().unwrap()` with an `i32` expected: the receiver is an `Option<i32>`.
        let recv_expected = match name.as_str() {
            // `.map(|s| parse(s)).collect::<Result<Vec<_>, _>>()`: the items are Results.
            "collect" => match &collect_target {
                Ty::Result(inner, err) => {
                    Ty::Iter(Box::new(Ty::Result(Box::new(inner.item()), err.clone())))
                }
                Ty::Option(inner) if matches!(**inner, Ty::Vec(_)) => {
                    Ty::Iter(Box::new(Ty::Option(Box::new(inner.item()))))
                }
                Ty::Vec(t) | Ty::Set(t, _) if !t.is_unknown() => Ty::Iter(t.clone()),
                Ty::Map(k, v, _) => {
                    Ty::Iter(Box::new(Ty::Tuple(vec![(**k).clone(), (**v).clone()])))
                }
                _ => Ty::Unknown,
            },
            "unwrap" | "expect" | "unwrap_or" | "unwrap_or_default" | "unwrap_or_else"
                if !expected.is_unknown() =>
            {
                Ty::Option(Box::new(expected.clone()))
            }
            // `text.parse().map_err(…)?` with an `f64` expected: the same target for `parse`.
            "map_err" | "ok" | "or" | "or_else" | "ok_or" | "ok_or_else" => expected.clone(),
            _ => Ty::Unknown,
        };
        if let Some(ex) = self.entry_modify(m) {
            return ex;
        }
        // `map.entry(k).or_default()` anywhere: the place in the map.
        if let Some(place) = self.entry_place(m) {
            let ty = self.entry_value_ty(m);
            return Ex::primary(place, ty);
        }
        // `map.entry(k).or_insert_with(Vec::new).push(x)`: the receiver is a place.
        let value_results = name == "collect" && !matches!(collect_target, Ty::Result(..));
        if value_results {
            self.result_values += 1;
        }
        let recv = if let Expr::MethodCall(inner) = &*m.receiver {
            match self.entry_place(inner) {
                Some(place) => {
                    let ty = self.entry_value_ty(inner);
                    Ex::primary(place, ty)
                }
                None => self.expr_with(&m.receiver, &recv_expected),
            }
        } else {
            self.expr_with(&m.receiver, &recv_expected)
        };
        if value_results {
            self.result_values -= 1;
        }
        // A method of a type of the crate.
        if let Some((mut sig, owner)) = self.user_method(&recv.ty, &name) {
            let mut recv = self.settle(recv);
            // `Lexer::new(src).tokens()` with `tokens(&mut self)`: Swift cannot mutate a
            // temporary, so it gets a name first.
            if sig.self_kind == SelfKind::RefMut
                && !crate::expr::is_place(&m.receiver)
                && self.preludes.last().is_some()
                && self
                    .struct_by_swift(&owner)
                    .is_some_and(|s| s.kind == TypeKind::Plain && !s.recursive)
            {
                let tmp = self.fresh(&names::lower_camel(&owner));
                self.prelude(format!("var {tmp} = {}", recv.code));
                recv = Ex::primary(tmp, recv.ty.clone());
            }
            // The type's generic parameters, as far as the receiver's type knows them.
            let generics = self
                .struct_by_swift(&owner)
                .map(|s| s.generics.clone())
                .unwrap_or_default();
            if let Ty::Named(_, args) = &recv.ty {
                let known: std::collections::HashMap<String, Ty> = generics
                    .iter()
                    .zip(args)
                    .filter(|(_, t)| t.is_complete())
                    .map(|(g, t)| (g.clone(), t.clone()))
                    .collect();
                if !known.is_empty() {
                    for p in &mut sig.params {
                        p.ty = crate::symbols::substitute(&p.ty, &known);
                    }
                    sig.ret = crate::symbols::substitute(&sig.ret, &known);
                }
            }
            let callee = if crate::expr::is_self(&m.receiver)
                && recv.code == "self"
                && !self.swift_local_visible(&names::lower_camel(&name))
                && self
                    .imp
                    .as_ref()
                    .is_some_and(|i| i.kind != TypeKind::Observable)
            {
                names::lower_camel(&name)
            } else {
                format!("{}.{}", wrap(&recv, P_PRIMARY), names::lower_camel(&name))
            };
            let (codes, arg_tys) = self.call_args_typed(&m.args, Some(&sig.params));
            if sig.property {
                return Ex::primary(callee, sig.ret.clone());
            }
            let mut ex = self.sig_result(format!("{callee}({})", codes.join(", ")), &sig);
            if sig.returns_self_mut {
                ex.ty = Ty::Unit;
            }
            if ex.ty == Ty::Named("Self".into(), vec![]) {
                ex.ty = recv.ty.clone();
            }
            // `stack.push("a".to_string())` tells that `stack` is a `Stack<String>`.
            if let (Expr::Path(p), Ty::Named(n, args)) = (&*m.receiver, &recv.ty) {
                if let Some(id) = p.path.get_ident() {
                    let mut bound = std::collections::HashMap::new();
                    for (param, arg) in sig.params.iter().zip(&arg_tys) {
                        unify(&param.ty, arg, &generics, &mut bound);
                    }
                    if !bound.is_empty() {
                        let args: Vec<Ty> = generics
                            .iter()
                            .enumerate()
                            .map(|(i, g)| {
                                bound
                                    .get(g)
                                    .cloned()
                                    .unwrap_or_else(|| args.get(i).cloned().unwrap_or(Ty::Unknown))
                            })
                            .collect();
                        let learned = Ty::Named(n.clone(), args);
                        self.refine(&id.to_string(), &learned);
                    }
                }
            }
            return ex;
        }
        let turbofish = self.method_turbofish(m);
        // Results of throwing calls.
        if recv.throws.is_some() {
            if let Some(ex) = self.throwing_receiver(recv.clone(), &name, m, expected) {
                return ex;
            }
        }
        let mut recv = self.settle(recv);
        // `v[..3].fill(0)`, `v[1..].sort()`: the slice itself is changed, not a copy of it.
        if matches!(
            name.as_str(),
            "fill"
                | "sort"
                | "sort_unstable"
                | "reverse"
                | "swap"
                | "copy_from_slice"
                | "clone_from_slice"
        ) && matches!(&*m.receiver, Expr::Index(i) if matches!(*i.index, Expr::Range(_)))
        {
            if let Some(inner) = recv
                .code
                .strip_prefix("Array(")
                .and_then(|s| s.strip_suffix(')'))
            {
                recv.code = inner.to_string();
            }
        }
        // `Some(4).take_if(…)`: Swift cannot mutate a temporary, so it gets a name first.
        if matches!(
            name.as_str(),
            "take" | "take_if" | "replace" | "insert" | "get_or_insert" | "get_or_insert_with"
        ) && matches!(recv.ty, Ty::Option(_))
            && !crate::expr::is_place(&m.receiver)
            && self.preludes.last().is_some()
        {
            let tmp = self.fresh("value");
            self.prelude(format!("var {tmp} = {}", recv.code));
            recv = Ex::primary(tmp, recv.ty.clone());
        }
        // `self.dismiss.call()` → `dismiss()`.
        if name == "call"
            && m.args.is_empty()
            && recv.ty == Ty::Named("DismissAction".into(), vec![])
        {
            return Ex::primary(format!("{}()", wrap(&recv, P_PRIMARY)), Ty::Unit);
        }
        let args = self.method_args(&recv, &name, m, expected);
        let mut ex = self.std_method(recv, &name, args, &turbofish, expected, m.span());
        if self.no_trailing == 0 {
            ex.code = trailing_closure(ex.code);
        }
        // Learn the types of collections from their uses.
        if let Expr::Path(p) = &*m.receiver {
            if let Some(id) = p.path.get_ident() {
                let id = id.to_string();
                if let Some(t) = self.pending_refine.take() {
                    self.refine(&id, &t);
                }
            }
        }
        self.pending_refine = None;
        ex
    }

    /// The types of `collect::<Vec<i32>>()`.
    fn method_turbofish(&mut self, m: &ExprMethodCall) -> Vec<Ty> {
        m.turbofish
            .as_ref()
            .map(|t| {
                t.args
                    .iter()
                    .filter_map(|a| match a {
                        syn::GenericArgument::Type(t) => Some(self.ty(t)),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `map.entry(k).and_modify(|v| …).or_insert(x)`: an `if` on the presence of the key.
    fn entry_modify(&mut self, m: &ExprMethodCall) -> Option<Ex> {
        let last = m.method.to_string();
        if !matches!(last.as_str(), "or_insert" | "or_insert_with" | "or_default") {
            return None;
        }
        let Expr::MethodCall(modify) = &*m.receiver else {
            return None;
        };
        if modify.method != "and_modify" || modify.args.len() != 1 {
            return None;
        }
        let Expr::MethodCall(entry) = &*modify.receiver else {
            return None;
        };
        if entry.method != "entry" || entry.args.len() != 1 {
            return None;
        }
        let Expr::Closure(c) = &modify.args[0] else {
            return None;
        };
        let map = self.expr(&entry.receiver);
        let map = self.settle(map);
        let (key_ty, value_ty) = match &map.ty {
            Ty::Map(k, v, _) => ((**k).clone(), (**v).clone()),
            _ => (Ty::Unknown, Ty::Unknown),
        };
        let key = self.expr_with(&entry.args[0], &key_ty);
        let key = self.settle(key);
        let place = format!("{}[{}]", wrap(&map, P_PRIMARY), key.code);
        // The closure's parameter stands for the value in the map.
        self.push_scope();
        if let Some(p) = c.inputs.first() {
            let pat = match p {
                syn::Pat::Type(pt) => &*pt.pat,
                p => p,
            };
            if let syn::Pat::Ident(id) = pat {
                self.declare_alias(&id.ident.to_string(), format!("{place}!"), value_ty.clone());
            }
        }
        let body = match &*c.body {
            Expr::Block(b) => self.block_lines(&b.block, crate::stmt::Sink::Discard),
            e => self.expr_stmt(e),
        };
        self.pop_scope();
        let default = match last.as_str() {
            "or_insert" => {
                let v = self.expr_with(&m.args[0], &value_ty);
                self.settle(v).code
            }
            "or_insert_with" => match &m.args[0] {
                Expr::Closure(f) if f.inputs.is_empty() => {
                    let v = self.expr_with(&f.body, &value_ty);
                    self.settle(v).code
                }
                f => {
                    let v = self.expr(f);
                    format!("{}()", wrap(&v, P_PRIMARY))
                }
            },
            _ => value_ty.default_value().unwrap_or_else(|| {
                self.error(
                    m.span(),
                    "or_default(): the value type of this map is unknown",
                );
                "fatalError()".into()
            }),
        };
        Some(Ex::new(
            format!(
                "if {place} != nil {{\n{}\n}} else {{\n    {place} = {default}\n}}",
                crate::translate::indent_lines(&body.join("\n"))
            ),
            crate::expr::P_ASSIGN,
            Ty::Unit,
        ))
    }

    pub fn entry_value_ty(&mut self, m: &ExprMethodCall) -> Ty {
        if let Expr::MethodCall(entry) = &*m.receiver {
            if let Expr::Path(p) = &*entry.receiver {
                if let Some(l) = p
                    .path
                    .get_ident()
                    .and_then(|i| self.lookup(&i.to_string()))
                    .cloned()
                {
                    if let Ty::Map(_, v, _) = self.local_ty(&l) {
                        return *v;
                    }
                }
            }
        }
        Ty::Unknown
    }

    /// `x.into()`: a `From` conversion when the target is known, otherwise the value.
    fn convert_into(&mut self, m: &ExprMethodCall, expected: &Ty) -> Ex {
        let v = self.expr(&m.receiver);
        let v = self.settle(v);
        if let Ty::Named(n, _) = expected {
            let rust = self
                .syms
                .structs
                .iter()
                .find(|(_, s)| &s.swift == n)
                .map(|(k, _)| k.clone())
                .or_else(|| {
                    self.syms
                        .enums
                        .iter()
                        .find(|(_, e)| &e.swift == n)
                        .map(|(k, _)| k.clone())
                });
            if let Some(rust) = rust {
                if self.syms.from_impls.contains_key(&rust) {
                    return Ex::primary(format!("{n}({})", v.code), expected.clone());
                }
            }
        }
        match (expected, &v.ty) {
            (Ty::Str, Ty::Char) => Ex::primary(format!("String({})", v.code), Ty::Str),
            // `impl From<Version> for String`: `String(version)`.
            (Ty::Str, Ty::Named(..)) if self.syms.from_impls.contains_key("String") => {
                Ex::primary(format!("String({})", v.code), Ty::Str)
            }
            (Ty::Int(_) | Ty::Float(_), t) if t.swift() != expected.swift() && t.is_numeric() => {
                Ex::primary(
                    format!("{}({})", expected.swift_lossy(), v.code),
                    expected.clone(),
                )
            }
            _ => {
                let mut ty = v.ty.clone();
                ty.merge(expected);
                Ex { ty, ..v }
            }
        }
    }

    /// The signature of a method of the crate's types, for a receiver of type `ty`.
    pub fn user_method(&self, ty: &Ty, name: &str) -> Option<(FnSig, String)> {
        match ty {
            Ty::Named(n, _) => {
                let rust = if n == "Self" {
                    self.imp.as_ref().map(|i| i.rust.clone())?
                } else {
                    self.syms
                        .structs
                        .iter()
                        .find(|(_, s)| &s.swift == n)
                        .map(|(k, _)| k.clone())
                        .or_else(|| {
                            self.syms
                                .enums
                                .iter()
                                .find(|(_, e)| &e.swift == n)
                                .map(|(k, _)| k.clone())
                        })
                        .unwrap_or_else(|| n.clone())
                };
                let sig = self.syms.methods.get(&rust)?.get(name)?;
                Some((sig.clone(), n.clone()))
            }
            Ty::Unknown | Ty::Dyn(_) | Ty::Opaque(_) => {
                if std_method_name(name) {
                    return None;
                }
                let mut found = self
                    .syms
                    .methods
                    .iter()
                    .filter(|(_, ms)| ms.contains_key(name));
                let (owner, ms) = found.next()?;
                Some((ms[name].clone(), names::upper_camel(owner)))
            }
            _ => None,
        }
    }

    /// Arguments of a standard method, with the expected types its receiver implies.
    fn method_args(
        &mut self,
        recv: &Ex,
        name: &str,
        m: &ExprMethodCall,
        call_expected: &Ty,
    ) -> Vec<Ex> {
        // `result.map(|ok| …)`, `result.map_err(|e| …)`.
        let item = match (&recv.ty, name) {
            (
                Ty::Result(_, e),
                "map_err" | "or_else" | "unwrap_or_else" | "is_err_and" | "inspect_err",
            ) => (**e).clone(),
            (Ty::Result(ok, _), _) => (**ok).clone(),
            _ => recv.ty.item(),
        };
        let (key, value) = match &recv.ty {
            Ty::Map(k, v, _) => ((**k).clone(), (**v).clone()),
            _ => (Ty::Unknown, Ty::Unknown),
        };
        let mut out = Vec::new();
        for (i, a) in m.args.iter().enumerate() {
            let expected = match (name, i, &recv.ty) {
                ("push" | "contains" | "starts_with" | "ends_with" | "fill", 0, Ty::Vec(_)) => {
                    item.clone()
                }
                ("push", 0, Ty::Str) => Ty::Char,
                ("insert", 0, Ty::Map(..))
                | ("get" | "contains_key" | "remove" | "get_mut" | "entry", 0, Ty::Map(..)) => {
                    key.clone()
                }
                ("insert", 1, Ty::Map(..)) => value.clone(),
                ("insert", 1, Ty::Vec(_)) => item.clone(),
                ("insert" | "contains" | "remove", 0, Ty::Set(..)) => item.clone(),
                // `result.and_then(|v| …)` and `result.or_else(|e| …)` give a `Result` back.
                ("and_then", 0, Ty::Result(ok, err)) => Ty::Fn(
                    vec![(**ok).clone()],
                    Box::new(Ty::Result(Box::new(Ty::Unknown), err.clone())),
                ),
                ("or_else", 0, Ty::Result(ok, err)) => Ty::Fn(
                    vec![(**err).clone()],
                    Box::new(Ty::Result(ok.clone(), Box::new(Ty::Unknown))),
                ),
                (
                    "map" | "filter" | "filter_map" | "flat_map" | "any" | "all" | "find"
                    | "position" | "for_each" | "take_while" | "skip_while" | "max_by_key"
                    | "min_by_key" | "sort_by_key" | "sort_by_cached_key" | "retain" | "inspect"
                    | "find_map" | "partition" | "is_some_and" | "and_then" | "map_or"
                    | "map_or_else",
                    _,
                    _,
                ) => {
                    let param = item.clone();
                    // `.map(|s| parse(s)).collect::<Result<Vec<_>, _>>()`: the closure throws.
                    let ret = match (name, call_expected) {
                        ("map", Ty::Iter(t) | Ty::Vec(t)) if !t.is_unknown() => (**t).clone(),
                        ("filter_map", Ty::Iter(t) | Ty::Vec(t)) if !t.is_unknown() => {
                            Ty::Option(t.clone())
                        }
                        _ => Ty::Unknown,
                    };
                    if name == "map_or" && i == 0 {
                        Ty::Unknown
                    } else {
                        Ty::Fn(vec![param], Box::new(ret))
                    }
                }

                ("dedup_by", 0, _) => Ty::Fn(vec![item.clone(), item.clone()], Box::new(Ty::Bool)),
                ("retain_mut" | "dedup_by_key", 0, _) => {
                    Ty::Fn(vec![item.clone()], Box::new(Ty::Unknown))
                }
                ("binary_search_by", 0, _) => Ty::Fn(
                    vec![item.clone()],
                    Box::new(Ty::Named("Ordering".into(), vec![])),
                ),
                ("binary_search_by_key", 1, _) => Ty::Fn(vec![item.clone()], Box::new(Ty::Unknown)),
                ("sort_by" | "max_by" | "min_by" | "sort_unstable_by", 0, _) => Ty::Fn(
                    vec![item.clone(), item.clone()],
                    Box::new(Ty::Named("Ordering".into(), vec![])),
                ),
                // `fold(BTreeMap::new(), …)` with the result's type known: the same type.
                ("fold", 0, _) => call_expected.clone(),
                ("unwrap_or" | "or", 0, Ty::Option(t)) => (**t).clone(),
                ("set" | "replace", 0, Ty::Named(n, a)) if n == "RsRefCell" => {
                    a.first().cloned().unwrap_or(Ty::Unknown)
                }
                ("unwrap_or_else" | "or_else", 0, Ty::Option(t)) => Ty::Fn(vec![], t.clone()),
                ("unwrap_or", 0, Ty::Result(t, _)) => (**t).clone(),
                (
                    "map_err" | "unwrap_or_else" | "is_err_and" | "inspect_err" | "is_ok_and",
                    0,
                    Ty::Result(..),
                ) => Ty::Fn(vec![item.clone()], Box::new(Ty::Unknown)),
                (
                    "min" | "max" | "pow" | "powi" | "powf" | "abs_diff" | "atan2" | "hypot"
                    | "checked_add" | "checked_sub" | "checked_mul" | "checked_div"
                    | "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add"
                    | "saturating_sub" | "saturating_mul" | "rem_euclid" | "div_euclid" | "clamp"
                    | "overflowing_add" | "overflowing_sub" | "overflowing_mul",
                    _,
                    t,
                ) if t.is_numeric() => {
                    if matches!(name, "pow" | "powi") {
                        Ty::Int(IntKind::U32)
                    } else {
                        t.clone()
                    }
                }
                (
                    "cmp" | "partial_cmp" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "total_cmp",
                    0,
                    t,
                ) => t.clone(),
                // `s.starts_with(char::is_uppercase)`, `s.find(|c| c == ' ')`: a predicate.
                (
                    "contains" | "starts_with" | "ends_with" | "rfind" | "split" | "strip_prefix"
                    | "strip_suffix" | "trim_start_matches" | "trim_end_matches" | "trim_matches"
                    | "rsplit" | "split_terminator",
                    0,
                    Ty::Str,
                ) if matches!(&m.args[i], Expr::Closure(_))
                    || matches!(&m.args[i], Expr::Path(p) if p.path.segments.len() >= 2) =>
                {
                    Ty::Fn(vec![Ty::Char], Box::new(Ty::Bool))
                }
                (
                    "push_str" | "contains" | "starts_with" | "ends_with" | "rfind" | "split"
                    | "replace" | "strip_prefix" | "strip_suffix" | "split_once"
                    | "trim_start_matches" | "trim_end_matches" | "trim_matches" | "matches"
                    | "rsplit" | "split_terminator",
                    _,
                    Ty::Str,
                ) => Ty::Str,
                ("splitn" | "rsplitn", 1, Ty::Str) => Ty::Str,
                ("splitn" | "rsplitn", 0, Ty::Str) => Ty::Int(IntKind::Usize),
                // `scan(state, |state, x| …)`: the closure changes the state in place.
                ("scan", 1, _) => {
                    let state = out.first().map_or(Ty::Unknown, |e: &Ex| e.ty.clone());
                    Ty::Fn(vec![state, item.clone()], Box::new(Ty::Unknown))
                }
                _ => Ty::Unknown,
            };
            // `fold(init, |acc, x| …)`: the accumulator has the type of `init`.
            let expected = if name == "fold" && i == 1 {
                let init_ty = out.first().map_or(Ty::Unknown, |e: &Ex| e.ty.clone());
                Ty::Fn(vec![init_ty.clone(), item.clone()], Box::new(init_ty))
            } else {
                expected
            };
            let v = self.expr_with(a, &expected);
            out.push(v);
        }
        out
    }

    /// Methods of `Result`s returned by throwing calls: `f()?` is handled by `?`, this covers
    /// `f().unwrap()`, `f().ok()`, `f().is_ok()`…
    fn throwing_receiver(
        &mut self,
        recv: Ex,
        name: &str,
        m: &ExprMethodCall,
        expected: &Ty,
    ) -> Option<Ex> {
        let (ok, _err) = recv.throws.as_ref().map(|t| (t.0.clone(), t.1.clone()))?;
        let code = recv.code.clone();
        Some(match name {
            "unwrap" | "expect" => Ex::new(format!("try! {code}"), P_TRY, ok),
            "ok" => Ex::new(format!("try? {code}"), P_TRY, Ty::Option(Box::new(ok))),
            "is_ok" => Ex::new(format!("(try? {code}) != nil"), P_CMP, Ty::Bool),
            "is_err" => Ex::new(format!("(try? {code}) == nil"), P_CMP, Ty::Bool),
            "unwrap_or" => {
                let d = self.expr_with(&m.args[0], &ok);
                let d = self.settle(d);
                Ex::new(format!("(try? {code}) ?? {}", wrap(&d, P_NIL)), P_NIL, ok)
            }
            "unwrap_or_default" => {
                let d = ok.default_value().unwrap_or_else(|| "fatalError()".into());
                Ex::new(format!("(try? {code}) ?? {d}"), P_NIL, ok)
            }
            "unwrap_or_else" => {
                if let Expr::Closure(c) = &m.args[0] {
                    if matches!(c.inputs.first(), Some(syn::Pat::Wild(_))) || c.inputs.is_empty() {
                        let v = self.expr_with(&c.body, &ok);
                        let v = self.settle(v);
                        return Some(Ex::new(
                            format!("(try? {code}) ?? {}", wrap(&v, P_NIL)),
                            P_NIL,
                            ok,
                        ));
                    }
                }
                let settled = self.settle(recv);
                let f = self.expr(&m.args[0]);
                self.helper("result");
                Ex::primary(
                    format!("{}.unwrapOrElse({})", wrap(&settled, P_PRIMARY), f.code),
                    ok,
                )
            }
            _ => {
                let _ = expected;
                return None;
            }
        })
    }

    /// `s.parse::<T>()`: the parse functions of the support code for the primitive types,
    /// the type's own `from_str` otherwise.
    fn parse_call(&mut self, recv: Ex, target: Ty, span: proc_macro2::Span) -> Ex {
        let s = recv.code;
        let result = |ok: &Ty, err: &str| {
            Ty::Result(
                Box::new(ok.clone()),
                Box::new(Ty::Named(err.into(), vec![])),
            )
        };
        match &target {
            Ty::Int(k) => {
                self.helper("parse");
                let bits = k
                    .narrow_bits()
                    .map(|b| format!(", bits: {b}"))
                    .unwrap_or_default();
                Ex::primary(
                    format!("rsParseInt({s}, {}.self{bits})", k.swift()),
                    result(&target, "RsParseIntError"),
                )
            }
            Ty::Float(_) => {
                self.helper("parse");
                Ex::primary(
                    format!("rsParseFloat({s}, {}.self)", target.swift_lossy()),
                    result(&target, "RsParseFloatError"),
                )
            }
            Ty::Bool => {
                self.helper("parse");
                Ex::primary(
                    format!("rsParseBool({s})"),
                    result(&target, "RsParseBoolError"),
                )
            }
            Ty::Char => {
                self.helper("parse");
                Ex::primary(
                    format!("rsParseChar({s})"),
                    result(&target, "RsParseCharError"),
                )
            }
            Ty::Str => Ex::primary(
                format!("Result<String, Never>.success({s})"),
                result(&target, "Never"),
            ),
            // `T: FromStr` in generic code: Swift's `LosslessStringConvertible`.
            Ty::Named(n, _) if self.generics.contains(n) => {
                self.helper("option");
                self.helper("ParseError");
                Ex::primary(
                    format!("{n}({s}).okOr(RsParseError())"),
                    result(&target, "RsParseError"),
                )
            }
            Ty::Named(n, _) => match self.user_method(&target, "from_str") {
                Some((sig, _)) => {
                    let mut ex = self.sig_result(format!("{n}.fromStr({s})"), &sig);
                    self.self_result(&mut ex, &target);
                    ex
                }
                None => {
                    self.error(
                        span,
                        format!("parse(): `{n}` has no `FromStr` implementation"),
                    );
                    Ex::primary(s, Ty::Unknown)
                }
            },
            _ => {
                self.error(
                    span,
                    "parse(): write the target type, e.g. `s.parse::<i32>()`",
                );
                Ex::primary(s, Ty::Unknown)
            }
        }
    }

    /// `Self` in the result of a static function: the type itself.
    fn self_result(&self, ex: &mut Ex, ty: &Ty) {
        let this = Ty::Named("Self".into(), vec![]);
        if ex.ty == this {
            ex.ty = ty.clone();
        }
        if let Some(t) = &mut ex.throws {
            if t.0 == this {
                t.0 = ty.clone();
            }
        }
    }

    /// `x.try_into()`: the target type comes from the context (`let n: u8 = x.try_into()?`).
    fn convert_try_into(&mut self, m: &ExprMethodCall, expected: &Ty) -> Ex {
        let target = match expected {
            Ty::Result(t, _) | Ty::Option(t) => (**t).clone(),
            t => t.clone(),
        };
        let result = |ok: &Ty, err: &str| {
            Ty::Result(
                Box::new(ok.clone()),
                Box::new(Ty::Named(err.into(), vec![])),
            )
        };
        match &target {
            Ty::Int(k) => {
                let v = self.expr(&m.receiver);
                let v = self.settle(v);
                self.helper("try_from");
                let bits = k
                    .narrow_bits()
                    .map(|b| format!(", bits: {b}"))
                    .unwrap_or_default();
                Ex::primary(
                    format!("rsTryFrom({}.self, {}{bits})", k.swift(), v.code),
                    result(&target, "RsTryFromIntError"),
                )
            }
            Ty::Char => {
                let v = self.expr(&m.receiver);
                let v = self.settle(v);
                self.helper("parse");
                Ex::primary(
                    format!("rsCharTryFrom({})", v.code),
                    result(&Ty::Char, "RsCharTryFromError"),
                )
            }
            Ty::Named(n, _) => match self.user_method(&target, "try_from") {
                Some((sig, _)) => {
                    let param = sig.params.first().map_or(Ty::Unknown, |p| p.ty.clone());
                    let v = self.expr_with(&m.receiver, &param);
                    let v = self.settle(v);
                    let mut ex = self.sig_result(format!("{n}.tryFrom({})", v.code), &sig);
                    self.self_result(&mut ex, &target);
                    ex
                }
                None => {
                    self.error(
                        m.span(),
                        format!("try_into(): `{n}` has no `TryFrom` implementation"),
                    );
                    let v = self.expr(&m.receiver);
                    self.settle(v)
                }
            },
            _ => {
                self.error(
                    m.span(),
                    "try_into(): write the target type, e.g. `let n: u8 = x.try_into()?`",
                );
                let v = self.expr(&m.receiver);
                self.settle(v)
            }
        }
    }

    /// The standard library methods.
    #[allow(clippy::too_many_lines)]
    pub fn std_method(
        &mut self,
        recv: Ex,
        name: &str,
        args: Vec<Ex>,
        turbofish: &[Ty],
        expected: &Ty,
        span: proc_macro2::Span,
    ) -> Ex {
        let args: Vec<Ex> = args.into_iter().map(|a| self.settle(a)).collect();
        // `s.starts_with(['a', 'b'])`, `s.split([',', ';'])`: an array of characters is a
        // predicate, like a closure.
        let args: Vec<Ex> = if recv.ty == Ty::Str
            && matches!(
                name,
                "contains"
                    | "starts_with"
                    | "ends_with"
                    | "find"
                    | "rfind"
                    | "split"
                    | "rsplit"
                    | "trim_start_matches"
                    | "trim_end_matches"
                    | "trim_matches"
                    | "matches"
                    | "match_indices"
                    | "split_terminator"
                    | "strip_prefix"
                    | "strip_suffix"
            ) {
            args.into_iter()
                .enumerate()
                .map(|(i, e)| {
                    if i == 0 && matches!(&e.ty, Ty::Vec(t) | Ty::Set(t, _) if **t == Ty::Char) {
                        Ex::primary(
                            format!("{{ (c: Character) in {}.contains(c) }}", e.code),
                            Ty::Fn(vec![Ty::Char], Box::new(Ty::Bool)),
                        )
                    } else {
                        e
                    }
                })
                .collect()
        } else {
            args
        };
        // A type of the crate implementing `Iterator` is a sequence.
        let recv = match &recv.ty {
            Ty::Named(n, _) => match self.syms.iterators.get(n) {
                Some(item) => Ex {
                    ty: Ty::Iter(Box::new(item.clone())),
                    ..recv
                },
                None => recv,
            },
            _ => recv,
        };
        let r = wrap(&recv, P_PRIMARY);
        let rty = recv.ty.clone();
        let item = rty.item();
        let a = |i: usize| {
            args.get(i)
                .map_or_else(String::new, |e: &Ex| e.code.clone())
        };
        let ty_a = |i: usize| args.get(i).map_or(Ty::Unknown, |e: &Ex| e.ty.clone());
        let fn_ret = |i: usize| match args.get(i).map(|e| &e.ty) {
            Some(Ty::Fn(_, r)) => (**r).clone(),
            _ => Ty::Unknown,
        };
        let prim = |code: String, ty: Ty| Ex::primary(code, ty);
        // `f.precision()`, `f.width()`, `f.alternate()` in a `Display` impl: Swift's
        // `description` knows nothing of the format spec.
        if rty == Ty::Named("Formatter".into(), vec![]) {
            match name {
                "precision" | "width" => {
                    self.warn(span, format!("`{name}()` of the formatter is always `None` in Swift: `description` does not know the format spec"));
                    return Ex::primary(
                        "(nil as Int?)",
                        Ty::Option(Box::new(Ty::Int(IntKind::Usize))),
                    );
                }
                "alternate" | "sign_plus" | "sign_minus" | "sign_aware_zero_pad" => {
                    return Ex::primary("false", Ty::Bool);
                }
                // `f.write_str(s)`, `f.write_char(c)`, `f.pad(s)`: appended to the text.
                "write_str" | "write_char" | "pad" | "write_fmt" => {
                    let target = recv
                        .code
                        .trim_start_matches('\u{2}')
                        .trim_start_matches("fmt:")
                        .to_string();
                    let value = if name == "write_char" {
                        format!("String({})", a(0))
                    } else {
                        a(0)
                    };
                    return Ex::new(
                        format!("{target} += {value}"),
                        crate::expr::P_ASSIGN,
                        Ty::Unit,
                    );
                }
                _ => {}
            }
        }
        // An iterator kept in a variable (`RsIter`).
        if let Ty::Named(n, targs) = &rty {
            if n == "RsIter" {
                let t = targs.first().cloned().unwrap_or(Ty::Unknown);
                match name {
                    "next" | "peek" => {
                        return prim(format!("{r}.{name}()"), Ty::Option(Box::new(t)));
                    }
                    "next_if" => {
                        return prim(format!("{r}.nextIf({})", a(0)), Ty::Option(Box::new(t)));
                    }
                    "nth" => return prim(format!("{r}.nth({})", a(0)), Ty::Option(Box::new(t))),
                    "by_ref" | "peekable" => return recv,
                    _ => {
                        let rest = Ex::primary(format!("{r}.rest()"), Ty::Iter(Box::new(t)));
                        return self.std_method(rest, name, args, turbofish, expected, span);
                    }
                }
            }
        }
        let is_seq = matches!(
            rty,
            Ty::Vec(_) | Ty::Iter(_) | Ty::Range(_) | Ty::Set(..) | Ty::Map(..)
        );
        let is_str = rty == Ty::Str;
        let usize_ty = Ty::Int(IntKind::Usize);

        // Option values.
        if let Ty::Option(inner) = &rty {
            let inner = (**inner).clone();
            match name {
                "is_some" | "is_ok" => {
                    return Ex::new(
                        format!("{} != nil", wrap(&recv, P_CMP + 1)),
                        P_CMP,
                        Ty::Bool,
                    );
                }
                "is_none" | "is_err" => {
                    return Ex::new(
                        format!("{} == nil", wrap(&recv, P_CMP + 1)),
                        P_CMP,
                        Ty::Bool,
                    );
                }
                "unwrap" | "expect" | "unwrap_unchecked" => return prim(format!("{r}!"), inner),
                "unwrap_or" | "or" => {
                    let mut t = inner.clone();
                    t.merge(&ty_a(0));
                    let d = &args[0];
                    return Ex::new(
                        format!("{} ?? {}", wrap(&recv, P_NIL + 1), wrap(d, P_NIL)),
                        P_NIL,
                        if name == "or" { rty.clone() } else { t },
                    );
                }
                "unwrap_or_else" | "or_else" => {
                    let body = closure_body(&a(0));
                    return Ex::new(
                        format!("{} ?? {body}", wrap(&recv, P_NIL + 1)),
                        P_NIL,
                        if name == "or_else" {
                            rty.clone()
                        } else {
                            inner
                        },
                    );
                }
                "unwrap_or_default" => {
                    let d = inner.default_value().unwrap_or_else(|| {
                        self.error(span, "unwrap_or_default(): unknown type here");
                        "fatalError()".into()
                    });
                    return Ex::new(format!("{} ?? {d}", wrap(&recv, P_NIL + 1)), P_NIL, inner);
                }
                "map" => {
                    return prim(
                        format!("{r}.map({})", a(0)),
                        Ty::Option(Box::new(fn_ret(0))),
                    );
                }
                "and_then" => {
                    let ret = match fn_ret(0) {
                        Ty::Option(t) => *t,
                        t => t,
                    };
                    return prim(format!("{r}.flatMap({})", a(0)), Ty::Option(Box::new(ret)));
                }
                "map_or" => {
                    return Ex::new(
                        format!("{r}.map({}) ?? {}", a(1), wrap(&args[0], P_NIL)),
                        P_NIL,
                        ty_a(0),
                    );
                }
                "map_or_else" => {
                    return Ex::new(
                        format!("{r}.map({}) ?? {}", a(1), closure_body(&a(0))),
                        P_NIL,
                        fn_ret(1),
                    );
                }
                "is_some_and" => {
                    return Ex::new(format!("{r}.map({}) ?? false", a(0)), P_NIL, Ty::Bool);
                }
                "is_none_or" => {
                    return Ex::new(format!("{r}.map({}) ?? true", a(0)), P_NIL, Ty::Bool);
                }
                "filter" => {
                    self.helper("option");
                    return prim(format!("{r}.filter({})", a(0)), rty);
                }
                "take" => {
                    self.helper("option");
                    return prim(format!("{r}.take()"), rty);
                }
                "replace" => {
                    self.helper("option");
                    return prim(format!("{r}.replace({})", a(0)), rty);
                }
                "xor" => {
                    self.helper("option");
                    return prim(format!("{r}.xor({})", a(0)), rty);
                }
                "zip" => {
                    self.helper("option");
                    return prim(
                        format!("{r}.zip({})", a(0)),
                        Ty::Option(Box::new(Ty::Tuple(vec![inner, ty_a(0).item()]))),
                    );
                }
                // As a statement; as a `let` initializer, `let_ident` makes an alias.
                "get_or_insert" | "get_or_insert_with" => {
                    let v = if name == "get_or_insert" {
                        a(0)
                    } else {
                        closure_body(&a(0))
                    };
                    return Ex::new(
                        format!("if {r} == nil {{ {r} = {v} }}"),
                        crate::expr::P_ASSIGN,
                        Ty::Unit,
                    );
                }
                "ok_or" | "ok_or_else" => {
                    let e = if name == "ok_or" {
                        a(0)
                    } else {
                        closure_body(&a(0))
                    };
                    self.helper("option");
                    let err = ty_a(0);
                    self.note_types(&err);
                    return prim(
                        format!("{r}.okOr({e})"),
                        Ty::Result(Box::new(inner), Box::new(err)),
                    );
                }
                "ok" | "cloned" | "copied" | "as_ref" | "as_mut" | "as_deref" | "as_deref_mut"
                | "clone" | "map_err" => return recv,
                "flatten" => return prim(format!("{r}.flatMap {{ $0 }}"), inner),
                // `Option<Result<T, E>>` → `Result<Option<T>, E>`.
                "transpose" => {
                    self.helper("result");
                    let (ok, err) = match &inner {
                        Ty::Result(o, e) => ((**o).clone(), (**e).clone()),
                        _ => (Ty::Unknown, Ty::Unknown),
                    };
                    return prim(
                        format!("{r}.transpose()"),
                        Ty::Result(Box::new(Ty::Option(Box::new(ok))), Box::new(err)),
                    );
                }
                "iter" | "into_iter" => {
                    // A bare `None.iter()` has nothing to map: an empty array, typed when
                    // the element type is known.
                    if r == "nil" {
                        let code = if inner.is_complete() {
                            format!("[{}]()", inner.swift_lossy())
                        } else {
                            "[]".into()
                        };
                        return prim(code, Ty::Vec(Box::new(inner)));
                    }
                    return prim(
                        format!("({r}.map {{ [$0] }} ?? [])"),
                        Ty::Vec(Box::new(inner)),
                    );
                }
                "unwrap_or_else_default" => {}
                _ => {}
            }
        }
        // Result values.
        if let Ty::Result(ok, err) = &rty {
            let (ok, err) = ((**ok).clone(), (**err).clone());
            self.helper("result");
            match name {
                "is_ok" => return prim(format!("{r}.isOk"), Ty::Bool),
                "is_err" => return prim(format!("{r}.isErr"), Ty::Bool),
                "unwrap" | "expect" => return Ex::new(format!("try! {r}.get()"), P_TRY, ok),
                "unwrap_err" | "expect_err" => return prim(format!("{r}.err()!"), err),
                "ok" => return prim(format!("{r}.ok()"), Ty::Option(Box::new(ok))),
                "err" => return prim(format!("{r}.err()"), Ty::Option(Box::new(err))),
                "unwrap_or" => {
                    return Ex::new(format!("{r}.ok() ?? {}", wrap(&args[0], P_NIL)), P_NIL, ok);
                }
                "unwrap_or_default" => {
                    let d = ok.default_value().unwrap_or_else(|| "fatalError()".into());
                    return Ex::new(format!("{r}.ok() ?? {d}"), P_NIL, ok);
                }
                "unwrap_or_else" => return prim(format!("{r}.unwrapOrElse({})", a(0)), ok),
                "map" => {
                    return prim(
                        format!("{r}.map({})", a(0)),
                        Ty::Result(Box::new(fn_ret(0)), Box::new(err)),
                    );
                }
                "map_err" => {
                    let ty = Ty::Result(Box::new(ok), Box::new(fn_ret(0)));
                    self.note_types(&ty);
                    return prim(format!("{r}.mapError({})", a(0)), ty);
                }
                "and_then" => {
                    let ret = match fn_ret(0) {
                        Ty::Result(o, _) => Ty::Result(o, Box::new(err)),
                        t => Ty::Result(Box::new(t), Box::new(err)),
                    };
                    return prim(format!("{r}.andThen({})", a(0)), ret);
                }
                "or_else" => {
                    let ret = match fn_ret(0) {
                        Ty::Result(_, e) => Ty::Result(Box::new(ok), e),
                        _ => Ty::Result(Box::new(ok), Box::new(Ty::Unknown)),
                    };
                    return prim(format!("{r}.orElse({})", a(0)), ret);
                }
                "is_ok_and" => {
                    return Ex::new(format!("{r}.ok().map({}) ?? false", a(0)), P_NIL, Ty::Bool);
                }
                // `Result<Option<T>, E>` → `Option<Result<T, E>>`.
                "transpose" => {
                    let inner = match &ok {
                        Ty::Option(t) => (**t).clone(),
                        t => t.clone(),
                    };
                    return prim(
                        format!("{r}.transpose()"),
                        Ty::Option(Box::new(Ty::Result(Box::new(inner), Box::new(err)))),
                    );
                }
                "is_err_and" => {
                    return Ex::new(format!("{r}.err().map({}) ?? false", a(0)), P_NIL, Ty::Bool);
                }
                _ => {}
            }
        }
        // Cells (`RefCell`, `Cell`, `Mutex`…) are `RsRefCell`, a class holding the `value`.
        if let Ty::Named(n, targs) = &rty {
            if n == "RsRefCell" {
                self.helper("refcell");
                let inner = targs.first().cloned().unwrap_or(Ty::Unknown);
                let value = format!("{r}.value");
                match name {
                    "borrow" | "borrow_mut" | "get" | "get_mut" | "into_inner"
                        if args.is_empty() =>
                    {
                        return prim(value, inner);
                    }
                    n if is_cell_access(n) && args.is_empty() => return prim(value, inner),
                    "set" if args.len() == 1 => {
                        return Ex::new(
                            format!("{value} = {}", a(0)),
                            crate::expr::P_ASSIGN,
                            Ty::Unit,
                        );
                    }
                    "replace" if args.len() == 1 => {
                        return prim(format!("{r}.replace({})", a(0)), inner);
                    }
                    "take" if args.is_empty() => {
                        let d = inner
                            .default_value()
                            .unwrap_or_else(|| "fatalError()".into());
                        return prim(format!("{r}.replace({d})"), inner);
                    }
                    "swap" if args.len() == 1 => {
                        return prim(format!("swap(&{value}, &{}.value)", a(0)), Ty::Unit);
                    }
                    "update" if args.len() == 1 => {
                        return Ex::new(
                            format!("{value} = ({})({value})", a(0)),
                            crate::expr::P_ASSIGN,
                            Ty::Unit,
                        );
                    }
                    // `rc.clone()`: the same cell.
                    "clone" if args.is_empty() => return recv,
                    _ => {}
                }
            }
        }
        // Universal conversions.
        match name {
            "clone" | "to_owned" | "cloned" | "copied" | "as_ref" | "as_mut" | "borrow"
            | "borrow_mut" | "as_slice" | "as_mut_slice" | "by_ref" | "iter" | "into_iter"
            | "as_str" | "into_boxed_slice" | "into_owned" | "lock" | "read" | "write"
            | "get_ref"
                if args.is_empty() =>
            {
                if matches!(name, "lock" | "read" | "write") {
                    return recv;
                }
                // A recursive struct is a class: `clone()` copies it for real.
                if matches!(name, "clone" | "to_owned") && self.is_recursive_class(&rty) {
                    return prim(format!("{r}.clone()"), rty);
                }
                let ty = match (&rty, name) {
                    (Ty::Map(..), "iter" | "into_iter") => Ty::Iter(Box::new(item.clone())),
                    (Ty::Range(_), "iter" | "into_iter") => Ty::Iter(Box::new(item.clone())),
                    _ => rty.clone(),
                };
                if matches!(recv.ty, Ty::Map(_, _, true)) && matches!(name, "iter" | "into_iter") {
                    return prim(format!("{r}.sorted(by: {{ $0.key < $1.key }})"), ty);
                }
                // A `BTreeSet` iterates in order.
                if matches!(recv.ty, Ty::Set(_, true)) && matches!(name, "iter" | "into_iter") {
                    return prim(format!("{r}.sorted()"), Ty::Iter(Box::new(item)));
                }
                return Ex { ty, ..recv };
            }
            "to_vec" => {
                return Ex {
                    ty: Ty::Vec(Box::new(item)),
                    ..recv
                };
            }
            "to_string" if args.is_empty() => {
                return match &rty {
                    Ty::Str => recv,
                    Ty::Char => prim(format!("String({})", recv.code), Ty::Str),
                    Ty::Int(_) | Ty::Bool => prim(format!("String({})", recv.code), Ty::Str),
                    Ty::Float(_) => {
                        self.helper("display");
                        prim(format!("rsDisplay({})", recv.code), Ty::Str)
                    }
                    _ => prim(format!("String(describing: {})", recv.code), Ty::Str),
                };
            }
            "unwrap" | "expect" if rty.is_unknown() => return prim(format!("{r}!"), Ty::Unknown),
            _ => {}
        }
        // Strings.
        if is_str {
            let str_arg = |i: usize| -> String {
                let e = &args[i];
                if e.ty == Ty::Char {
                    match e
                        .code
                        .strip_prefix("Character(")
                        .and_then(|s| s.strip_suffix(')'))
                    {
                        Some(lit) if lit.starts_with('"') => lit.to_string(),
                        _ => format!("String({})", e.code),
                    }
                } else {
                    e.code.clone()
                }
            };
            match name {
                "len" | "capacity" => return prim(format!("{r}.utf8.count"), usize_ty),
                "is_empty" => return prim(format!("{r}.isEmpty"), Ty::Bool),
                "push_str" => {
                    return Ex::new(
                        format!("{} += {}", recv.code, a(0)),
                        crate::expr::P_ASSIGN,
                        Ty::Unit,
                    );
                }
                "push" => return prim(format!("{r}.append({})", a(0)), Ty::Unit),
                "pop" => return prim(format!("{r}.popLast()"), Ty::Option(Box::new(Ty::Char))),
                "clear" => return prim(format!("{r}.removeAll()"), Ty::Unit),
                // Rust's `char`s are Unicode scalars; Swift's characters are grapheme clusters.
                "chars" => {
                    self.helper("chars");
                    return prim(format!("{r}.rsChars"), Ty::Iter(Box::new(Ty::Char)));
                }
                "bytes" => {
                    return prim(
                        format!("{r}.utf8"),
                        Ty::Iter(Box::new(Ty::Int(IntKind::U8))),
                    );
                }
                "as_bytes" | "into_bytes" => {
                    return prim(
                        format!("Array({r}.utf8)"),
                        Ty::Vec(Box::new(Ty::Int(IntKind::U8))),
                    );
                }
                "to_uppercase" => return prim(format!("{r}.uppercased()"), Ty::Str),
                "to_lowercase" => return prim(format!("{r}.lowercased()"), Ty::Str),
                // Only the ASCII letters change: `straße` stays as it is.
                "to_ascii_uppercase" | "to_ascii_lowercase" => {
                    self.helper("string");
                    return prim(format!("{r}.{}()", names::lower_camel(name)), Ty::Str);
                }
                "trim" => {
                    return prim(
                        format!("{r}.trimmingCharacters(in: .whitespacesAndNewlines)"),
                        Ty::Str,
                    );
                }
                "trim_start" | "trim_end" | "trim_left" | "trim_right" => {
                    self.helper("string");
                    let f = if name.ends_with("start") || name.ends_with("left") {
                        "trimStart"
                    } else {
                        "trimEnd"
                    };
                    return prim(format!("{r}.{f}()"), Ty::Str);
                }
                "trim_start_matches" if matches!(ty_a(0), Ty::Fn(..)) => {
                    return prim(format!("String({r}.drop(while: {}))", a(0)), Ty::Str);
                }
                "trim_end_matches" | "trim_matches" if matches!(ty_a(0), Ty::Fn(..)) => {
                    self.helper("string");
                    let f = names::lower_camel(name);
                    return prim(format!("{r}.{f}(where: {})", a(0)), Ty::Str);
                }
                "trim_start_matches" | "trim_end_matches" | "trim_matches" => {
                    self.helper("string");
                    let f = match name {
                        "trim_start_matches" => "trimStartMatches",
                        "trim_end_matches" => "trimEndMatches",
                        _ => "trimMatches",
                    };
                    return prim(format!("{r}.{f}({})", str_arg(0)), Ty::Str);
                }
                "splitn" | "rsplitn" if args.len() == 2 => {
                    self.helper("string");
                    return prim(
                        format!("{r}.{name}({}, {})", a(0), str_arg(1)),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "rsplit" if matches!(ty_a(0), Ty::Fn(..)) => {
                    return prim(
                        format!(
                            "Array({r}.split(omittingEmptySubsequences: false, whereSeparator: {}).map(String.init).reversed())",
                            a(0)
                        ),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "rsplit" => {
                    return prim(
                        format!("{r}.components(separatedBy: {}).reversed()", str_arg(0)),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "match_indices" => {
                    self.helper("string");
                    let arg = if matches!(ty_a(0), Ty::Fn(..)) {
                        format!("where: {}", a(0))
                    } else {
                        str_arg(0)
                    };
                    return prim(
                        format!("{r}.matchIndices({arg})"),
                        Ty::Iter(Box::new(Ty::Tuple(vec![Ty::Int(IntKind::Usize), Ty::Str]))),
                    );
                }
                // `s.retain(|c| …)`, `s.drain(..n)`, `s.split_off(n)`: the text changed in place.
                "retain" if args.len() == 1 => {
                    self.helper("string");
                    return prim(format!("{r}.retain({})", a(0)), Ty::Unit);
                }
                "drain" => {
                    self.helper("string");
                    return prim(format!("{r}.drain({})", a(0)), Ty::Str);
                }
                "split_off" if args.len() == 1 => {
                    self.helper("string");
                    return prim(format!("{r}.splitOff({})", a(0)), Ty::Str);
                }
                "make_ascii_uppercase" | "make_ascii_lowercase" => {
                    self.helper("string");
                    return prim(format!("{r}.{}()", names::lower_camel(name)), Ty::Unit);
                }
                "split_terminator" => {
                    self.helper("string");
                    return prim(
                        format!("{r}.splitTerminator({})", str_arg(0)),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "starts_with" if matches!(ty_a(0), Ty::Fn(..)) => {
                    return Ex::new(format!("{r}.first.map({}) ?? false", a(0)), P_NIL, Ty::Bool);
                }
                "ends_with" if matches!(ty_a(0), Ty::Fn(..)) => {
                    return Ex::new(format!("{r}.last.map({}) ?? false", a(0)), P_NIL, Ty::Bool);
                }
                "contains" if matches!(ty_a(0), Ty::Fn(..)) => {
                    return prim(format!("{r}.contains(where: {})", a(0)), Ty::Bool);
                }
                "find" | "rfind" if matches!(ty_a(0), Ty::Fn(..)) => {
                    self.helper("string");
                    let f = if name == "find" {
                        "findWhere"
                    } else {
                        "rfindWhere"
                    };
                    return prim(format!("{r}.{f}({})", a(0)), Ty::Option(Box::new(usize_ty)));
                }
                "strip_prefix" if matches!(ty_a(0), Ty::Fn(..)) => {
                    return Ex::new(
                        format!(
                            "{r}.first.map({}) == true ? String({r}.dropFirst()) : nil",
                            a(0)
                        ),
                        P_TERNARY,
                        Ty::Option(Box::new(Ty::Str)),
                    );
                }
                "strip_suffix" if matches!(ty_a(0), Ty::Fn(..)) => {
                    return Ex::new(
                        format!(
                            "{r}.last.map({}) == true ? String({r}.dropLast()) : nil",
                            a(0)
                        ),
                        P_TERNARY,
                        Ty::Option(Box::new(Ty::Str)),
                    );
                }
                "starts_with" => return prim(format!("{r}.hasPrefix({})", str_arg(0)), Ty::Bool),
                "ends_with" => return prim(format!("{r}.hasSuffix({})", str_arg(0)), Ty::Bool),
                "contains" => return prim(format!("{r}.contains({})", a(0)), Ty::Bool),
                // `s.get(a..b)`: `None` when a bound is not a character boundary.
                "get" if args.len() == 1 && matches!(ty_a(0), Ty::Range(_)) => {
                    self.helper("string");
                    return prim(
                        format!("{r}.rsGet({})", a(0)),
                        Ty::Option(Box::new(Ty::Str)),
                    );
                }
                "find" | "rfind" => {
                    self.helper("string");
                    let f = if name == "find" { "find" } else { "rfind" };
                    return prim(
                        format!("{r}.{f}({})", str_arg(0)),
                        Ty::Option(Box::new(usize_ty)),
                    );
                }
                "replace" => {
                    return prim(
                        format!(
                            "{r}.replacingOccurrences(of: {}, with: {})",
                            str_arg(0),
                            a(1)
                        ),
                        Ty::Str,
                    );
                }
                "replacen" if args.len() == 3 => {
                    self.helper("string");
                    return prim(
                        format!("{r}.replacen({}, {}, {})", str_arg(0), a(1), a(2)),
                        Ty::Str,
                    );
                }
                "split" => {
                    if matches!(ty_a(0), Ty::Fn(..)) {
                        return prim(
                            format!(
                                "{r}.split(omittingEmptySubsequences: false, whereSeparator: {}).map(String.init)",
                                a(0)
                            ),
                            Ty::Iter(Box::new(Ty::Str)),
                        );
                    }
                    return prim(
                        format!("{r}.components(separatedBy: {})", str_arg(0)),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "split_whitespace" | "split_ascii_whitespace" => {
                    return prim(
                        format!("{r}.split(whereSeparator: \\.isWhitespace).map(String.init)"),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "lines" => {
                    self.helper("string");
                    return prim(format!("{r}.lines()"), Ty::Iter(Box::new(Ty::Str)));
                }
                "split_once" | "rsplit_once" => {
                    self.helper("string");
                    let f = if name == "split_once" {
                        "splitOnce"
                    } else {
                        "rsplitOnce"
                    };
                    return prim(
                        format!("{r}.{f}({})", str_arg(0)),
                        Ty::Option(Box::new(Ty::Tuple(vec![Ty::Str, Ty::Str]))),
                    );
                }
                "strip_prefix" | "strip_suffix" => {
                    self.helper("string");
                    let f = if name == "strip_prefix" {
                        "stripPrefix"
                    } else {
                        "stripSuffix"
                    };
                    return prim(
                        format!("{r}.{f}({})", str_arg(0)),
                        Ty::Option(Box::new(Ty::Str)),
                    );
                }
                "char_indices" => {
                    self.helper("string");
                    return prim(
                        format!("{r}.charIndices()"),
                        Ty::Iter(Box::new(Ty::Tuple(vec![usize_ty, Ty::Char]))),
                    );
                }
                "repeat" => {
                    return prim(
                        format!("String(repeating: {}, count: {})", recv.code, a(0)),
                        Ty::Str,
                    );
                }
                "parse" => {
                    let target = turbofish
                        .first()
                        .cloned()
                        .unwrap_or_else(|| match expected {
                            Ty::Result(t, _) | Ty::Option(t) => (**t).clone(),
                            t => t.clone(),
                        });
                    return self.parse_call(recv, target, span);
                }
                "eq_ignore_ascii_case" => {
                    return Ex::new(
                        format!(
                            "{r}.lowercased() == {}.lowercased()",
                            wrap(&args[0], P_PRIMARY)
                        ),
                        P_CMP,
                        Ty::Bool,
                    );
                }
                "get" => {
                    self.helper("string");
                    return prim(
                        format!("{r}.rsSlice({})", a(0).replace("..<", ", ")),
                        Ty::Option(Box::new(Ty::Str)),
                    );
                }
                "insert" => {
                    self.helper("string");
                    return prim(format!("{r}.insertAt({}, {})", a(0), a(1)), Ty::Unit);
                }
                "insert_str" => {
                    self.helper("string");
                    return prim(format!("{r}.insertAt({}, {})", a(0), a(1)), Ty::Unit);
                }
                "remove" => {
                    self.helper("string");
                    return prim(format!("{r}.removeAt({})", a(0)), Ty::Char);
                }
                "truncate" => {
                    self.helper("string");
                    return prim(format!("{r}.truncate({})", a(0)), Ty::Unit);
                }
                "matches" if matches!(ty_a(0), Ty::Fn(..)) => {
                    self.helper("chars");
                    return prim(
                        format!("{r}.rsChars.filter({}).map {{ String($0) }}", a(0)),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "matches" => {
                    return prim(
                        format!("{r}.components(separatedBy: {}).dropFirst()", str_arg(0)),
                        Ty::Iter(Box::new(Ty::Str)),
                    );
                }
                "next" => return prim(format!("{r}.first"), Ty::Option(Box::new(Ty::Char))),
                "char_count" => return prim(format!("{r}.count"), usize_ty),
                "cmp" => {
                    self.helper("Ordering");
                    return prim(
                        format!("rsCmp({}, {})", recv.code, a(0)),
                        Ty::Named("Ordering".into(), vec![]),
                    );
                }
                "is_char_boundary" => {
                    self.helper("string");
                    return prim(format!("{r}.isCharBoundary({})", a(0)), Ty::Bool);
                }
                _ => {}
            }
        }
        // Characters.
        if rty == Ty::Char {
            let p = |s: &str, ty: Ty| prim(format!("{r}.{s}"), ty);
            match name {
                "is_alphabetic" => return p("isLetter", Ty::Bool),
                "is_numeric" => return p("isNumber", Ty::Bool),
                "is_alphanumeric" => {
                    return Ex::new(
                        format!("({r}.isLetter || {r}.isNumber)"),
                        P_PRIMARY,
                        Ty::Bool,
                    );
                }
                "is_whitespace" | "is_ascii_whitespace" => return p("isWhitespace", Ty::Bool),
                "is_uppercase" => return p("isUppercase", Ty::Bool),
                "is_lowercase" => return p("isLowercase", Ty::Bool),
                "is_ascii" => return p("isASCII", Ty::Bool),
                "is_control" | "is_ascii_control" => {
                    return Ex::new(
                        format!(
                            "{r}.unicodeScalars.allSatisfy {{ $0.properties.generalCategory == .control }}"
                        ),
                        P_PRIMARY,
                        Ty::Bool,
                    );
                }
                "is_ascii_digit"
                | "is_ascii_alphabetic"
                | "is_ascii_alphanumeric"
                | "is_ascii_uppercase"
                | "is_ascii_lowercase"
                | "is_ascii_punctuation"
                | "is_ascii_hexdigit"
                | "is_digit" => {
                    self.helper("char");
                    if name == "is_digit" {
                        return prim(format!("({r}.toDigit({}) != nil)", a(0)), Ty::Bool);
                    }
                    return p(&names::lower_camel(name), Ty::Bool);
                }
                "to_ascii_uppercase" => {
                    return prim(format!("Character({r}.uppercased())"), Ty::Char);
                }
                "to_ascii_lowercase" => {
                    return prim(format!("Character({r}.lowercased())"), Ty::Char);
                }
                "to_uppercase" => return prim(format!("{r}.uppercased()"), Ty::Str),
                "to_lowercase" => return prim(format!("{r}.lowercased()"), Ty::Str),
                "to_digit" => {
                    self.helper("char");
                    return prim(
                        format!("{r}.toDigit({})", a(0)),
                        Ty::Option(Box::new(Ty::Int(IntKind::U32))),
                    );
                }
                "len_utf8" => return prim(format!("String({}).utf8.count", recv.code), usize_ty),
                "eq_ignore_ascii_case" => {
                    return Ex::new(
                        format!(
                            "{r}.lowercased() == {}.lowercased()",
                            wrap(&args[0], P_PRIMARY)
                        ),
                        P_CMP,
                        Ty::Bool,
                    );
                }
                "cmp" => {
                    self.helper("Ordering");
                    return prim(
                        format!("rsCmp({}, {})", recv.code, a(0)),
                        Ty::Named("Ordering".into(), vec![]),
                    );
                }
                _ => {}
            }
        }
        // Numbers.
        if rty.is_numeric()
            || (rty.is_unknown()
                && matches!(
                    name,
                    "abs" | "pow" | "powi" | "powf" | "sqrt" | "floor" | "ceil" | "round" | "trunc"
                ))
        {
            let float = rty.is_float()
                || matches!(
                    name,
                    "powi" | "powf" | "sqrt" | "floor" | "ceil" | "round" | "trunc"
                );
            let fty = if let Ty::Float(_) = rty {
                rty.clone()
            } else {
                Ty::Float(FloatKind::F64)
            };
            let t = if float { fty.clone() } else { rty.clone() };
            let ts = t.swift().unwrap_or_else(|| "Double".into());
            match name {
                "abs" => return prim(format!("{}({})", self.global_fn("abs"), recv.code), t),
                // `f64::max` ignores a NaN operand: IEEE `maximum`/`minimum`.
                "min" | "max" if args.len() == 1 && float => {
                    let f = if name == "max" { "maximum" } else { "minimum" };
                    return prim(format!("{ts}.{f}({}, {})", recv.code, a(0)), t);
                }
                "min" | "max" if args.len() == 1 => {
                    return prim(
                        format!("{}({}, {})", self.global_fn(name), recv.code, a(0)),
                        t,
                    );
                }
                "clamp" => return prim(format!("min(max({}, {}), {})", recv.code, a(0), a(1)), t),
                "pow" if !float => {
                    self.helper("int");
                    return prim(format!("{r}.pow({})", a(0)), t);
                }
                "powi" => return prim(format!("pow({}, {ts}({}))", recv.code, a(0)), t),
                "powf" | "pow" => return prim(format!("pow({}, {})", recv.code, a(0)), t),
                "sqrt" => return prim(format!("{r}.squareRoot()"), t),
                "cbrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sinh" | "cosh"
                | "tanh" | "exp" | "exp2" | "log10" | "log2" => {
                    return prim(format!("{name}({})", recv.code), t);
                }
                "ln" => return prim(format!("log({})", recv.code), t),
                "log" => return prim(format!("(log({}) / log({}))", recv.code, a(0)), t),
                "atan2" => return prim(format!("atan2({}, {})", recv.code, a(0)), t),
                "hypot" => return prim(format!("hypot({}, {})", recv.code, a(0)), t),
                "floor" => return prim(format!("{r}.rounded(.down)"), t),
                "ceil" => return prim(format!("{r}.rounded(.up)"), t),
                "round" => return prim(format!("{r}.rounded()"), t),
                "trunc" => return prim(format!("{r}.rounded(.towardZero)"), t),
                "fract" => {
                    return Ex::new(
                        format!("{} - {r}.rounded(.towardZero)", recv.code),
                        P_ADD,
                        t,
                    );
                }
                "signum" if float => {
                    self.helper("int");
                    return prim(format!("{r}.rsSignum()"), t);
                }
                "signum" => return prim(format!("{r}.signum()"), t),
                "is_nan" => return prim(format!("{r}.isNaN"), Ty::Bool),
                "is_finite" => return prim(format!("{r}.isFinite"), Ty::Bool),
                "is_infinite" => return prim(format!("{r}.isInfinite"), Ty::Bool),
                "is_sign_negative" | "is_negative" => {
                    return Ex::new(format!("{} < 0", wrap(&recv, P_CMP + 1)), P_CMP, Ty::Bool);
                }
                "is_sign_positive" | "is_positive" => {
                    return Ex::new(format!("{} > 0", wrap(&recv, P_CMP + 1)), P_CMP, Ty::Bool);
                }
                "to_degrees" => {
                    return Ex::new(format!("{} * 180 / .pi", wrap(&recv, P_MUL)), P_MUL, t);
                }
                "to_radians" => {
                    return Ex::new(format!("{} * .pi / 180", wrap(&recv, P_MUL)), P_MUL, t);
                }
                "mul_add" => {
                    return Ex::new(
                        format!(
                            "{} * {} + {}",
                            wrap(&recv, P_MUL),
                            wrap(&args[0], P_MUL),
                            wrap(&args[1], P_ADD + 1)
                        ),
                        P_ADD,
                        t,
                    );
                }
                // Bits of an `i32`, which Swift holds in 64: the 32-bit value.
                "count_ones" | "leading_zeros" | "trailing_zeros" | "rotate_left"
                | "rotate_right" => {
                    let narrow = if t == Ty::Int(IntKind::I32) {
                        format!("Int32(truncatingIfNeeded: {r})")
                    } else {
                        r.clone()
                    };
                    let u32 = Ty::Int(IntKind::U32);
                    return match name {
                        "count_ones" => prim(format!("{narrow}.nonzeroBitCount"), u32),
                        "leading_zeros" => prim(format!("{narrow}.leadingZeroBitCount"), u32),
                        "trailing_zeros" => prim(format!("{narrow}.trailingZeroBitCount"), u32),
                        _ => {
                            self.helper("int");
                            let f = names::lower_camel(name);
                            let rotated = format!("{narrow}.{f}({})", a(0));
                            if t == Ty::Int(IntKind::I32) {
                                prim(format!("Int({rotated})"), t)
                            } else {
                                prim(rotated, t)
                            }
                        }
                    };
                }
                "is_power_of_two" => {
                    return Ex::new(
                        format!("{} > 0 && {r}.nonzeroBitCount == 1", wrap(&recv, P_CMP + 1)),
                        crate::expr::P_AND,
                        Ty::Bool,
                    );
                }
                // `i32` is Swift's 64-bit `Int`: these operations keep the 32-bit limits.
                "wrapping_add" | "wrapping_sub" | "wrapping_mul" if t == Ty::Int(IntKind::I32) => {
                    let op = match name {
                        "wrapping_add" => "&+",
                        "wrapping_sub" => "&-",
                        _ => "&*",
                    };
                    return prim(
                        format!(
                            "Int(Int32(truncatingIfNeeded: {}) {op} Int32(truncatingIfNeeded: {}))",
                            recv.code,
                            a(0)
                        ),
                        t,
                    );
                }
                "wrapping_add" => {
                    return Ex::new(
                        format!("{} &+ {}", wrap(&recv, P_ADD), wrap(&args[0], P_ADD + 1)),
                        P_ADD,
                        t,
                    );
                }
                "wrapping_sub" => {
                    return Ex::new(
                        format!("{} &- {}", wrap(&recv, P_ADD), wrap(&args[0], P_ADD + 1)),
                        P_ADD,
                        t,
                    );
                }
                "wrapping_mul" => {
                    return Ex::new(
                        format!("{} &* {}", wrap(&recv, P_MUL), wrap(&args[0], P_MUL + 1)),
                        P_MUL,
                        t,
                    );
                }
                "checked_add" | "checked_sub" | "checked_mul" | "checked_div" | "checked_rem"
                | "saturating_add" | "saturating_sub" | "saturating_mul" | "rem_euclid"
                | "div_euclid" | "abs_diff" | "isqrt" | "checked_pow" | "overflowing_add"
                | "overflowing_sub" | "overflowing_mul" | "div_ceil" => {
                    self.helper("int");
                    let ret = if name.starts_with("checked") {
                        Ty::Option(Box::new(t.clone()))
                    } else if name.starts_with("overflowing") {
                        Ty::Tuple(vec![t.clone(), Ty::Bool])
                    } else {
                        t.clone()
                    };
                    let narrow = t == Ty::Int(IntKind::I32)
                        && (name.starts_with("checked")
                            || name.starts_with("saturating")
                            || name.starts_with("overflowing"));
                    if narrow {
                        let call = format!(
                            "Int32(truncatingIfNeeded: {}).{}(Int32(truncatingIfNeeded: {}))",
                            recv.code,
                            names::lower_camel(name),
                            a(0)
                        );
                        let code = if name.starts_with("checked") {
                            format!("{call}.map {{ Int($0) }}")
                        } else if name.starts_with("overflowing") {
                            format!("rsWiden({call})")
                        } else {
                            format!("Int({call})")
                        };
                        return prim(code, ret);
                    }
                    return prim(format!("{r}.{}({})", names::lower_camel(name), a(0)), ret);
                }
                "next_power_of_two" | "ilog2" | "ilog10" if t.is_int() => {
                    self.helper("int");
                    let f = names::lower_camel(name);
                    let ret = if name == "next_power_of_two" {
                        t.clone()
                    } else {
                        Ty::Int(IntKind::U32)
                    };
                    return prim(format!("{r}.{f}()"), ret);
                }
                // `x.unsigned_abs()`: the magnitude, in the unsigned type of the same width.
                "unsigned_abs" => {
                    let Ty::Int(k) = t else {
                        return prim(format!("{r}.magnitude"), t);
                    };
                    let unsigned = match k {
                        IntKind::I8 => IntKind::U8,
                        IntKind::I16 => IntKind::U16,
                        IntKind::I32 => IntKind::U32,
                        IntKind::I64 => IntKind::U64,
                        IntKind::I128 => IntKind::U128,
                        IntKind::Isize => IntKind::Usize,
                        k => k,
                    };
                    let code = if k == IntKind::I32 {
                        format!("UInt32(truncatingIfNeeded: {r}.magnitude)")
                    } else {
                        format!("{r}.magnitude")
                    };
                    return prim(code, Ty::Int(unsigned));
                }
                // `x.checked_abs()`, `x.wrapping_neg()`: within the 32-bit limits of `i32`.
                "checked_abs" | "checked_neg" | "wrapping_neg" | "wrapping_abs"
                | "saturating_neg" | "saturating_abs"
                    if t.is_int() =>
                {
                    self.helper("int");
                    let f = names::lower_camel(name);
                    let ret = if name.starts_with("checked") {
                        Ty::Option(Box::new(t.clone()))
                    } else {
                        t.clone()
                    };
                    if t == Ty::Int(IntKind::I32) {
                        let call = format!("Int32(truncatingIfNeeded: {}).{f}()", recv.code);
                        let code = if name.starts_with("checked") {
                            format!("{call}.map {{ Int($0) }}")
                        } else {
                            format!("Int({call})")
                        };
                        return prim(code, ret);
                    }
                    return prim(format!("{r}.{f}()"), ret);
                }
                "cmp" => {
                    self.helper("Ordering");
                    return prim(
                        format!("rsCmp({}, {})", recv.code, a(0)),
                        Ty::Named("Ordering".into(), vec![]),
                    );
                }
                "partial_cmp" => {
                    self.helper("Ordering");
                    return prim(
                        format!("rsPartialCmp({}, {})", recv.code, a(0)),
                        Ty::Option(Box::new(Ty::Named("Ordering".into(), vec![]))),
                    );
                }
                "total_cmp" => {
                    self.helper("Ordering");
                    return prim(
                        format!("rsCmp({}, {})", recv.code, a(0)),
                        Ty::Named("Ordering".into(), vec![]),
                    );
                }
                "to_le_bytes" | "to_be_bytes" | "swap_bytes" => {}
                _ => {}
            }
        }
        // Booleans.
        if rty == Ty::Bool {
            match name {
                "then" => {
                    return Ex::new(
                        format!(
                            "{} ? {} : nil",
                            wrap(&recv, crate::expr::P_TERNARY + 1),
                            closure_body(&a(0))
                        ),
                        crate::expr::P_TERNARY,
                        Ty::Option(Box::new(fn_ret(0))),
                    );
                }
                "then_some" => {
                    return Ex::new(
                        format!(
                            "{} ? {} : nil",
                            wrap(&recv, crate::expr::P_TERNARY + 1),
                            a(0)
                        ),
                        crate::expr::P_TERNARY,
                        Ty::Option(Box::new(ty_a(0))),
                    );
                }
                "cmp" => {
                    self.helper("Ordering");
                    return prim(
                        format!("rsCmp({}, {})", recv.code, a(0)),
                        Ty::Named("Ordering".into(), vec![]),
                    );
                }
                _ => {}
            }
        }
        // Bytes: `b.is_ascii_digit()`, `b.to_ascii_uppercase()`.
        if rty == Ty::Int(IntKind::U8) {
            match name {
                "is_ascii_digit"
                | "is_ascii_alphabetic"
                | "is_ascii_alphanumeric"
                | "is_ascii_uppercase"
                | "is_ascii_lowercase"
                | "is_ascii_punctuation"
                | "is_ascii_hexdigit"
                | "is_ascii_whitespace"
                | "is_ascii_control"
                | "is_ascii_graphic" => {
                    let ch = Ex::primary(format!("Character(UnicodeScalar({r}))"), Ty::Char);
                    return self.std_method(ch, name, args, turbofish, expected, span);
                }
                "is_ascii" => return Ex::new(format!("{r} < 128"), P_CMP, Ty::Bool),
                "to_ascii_uppercase" => {
                    return prim(format!("({r} >= 97 && {r} <= 122 ? {r} - 32 : {r})"), rty);
                }
                "to_ascii_lowercase" => {
                    return prim(format!("({r} >= 65 && {r} <= 90 ? {r} + 32 : {r})"), rty);
                }
                "eq_ignore_ascii_case" => {
                    return Ex::new(
                        format!(
                            "({r} >= 65 && {r} <= 90 ? {r} + 32 : {r}) == ({} >= 65 && {} <= 90 ? {} + 32 : {})",
                            a(0),
                            a(0),
                            a(0),
                            a(0)
                        ),
                        P_CMP,
                        Ty::Bool,
                    );
                }
                _ => {}
            }
        }
        // Maps.
        if let Ty::Map(k, v, ordered) = &rty {
            let (k, v, ordered) = ((**k).clone(), (**v).clone(), *ordered);
            let sorted = |s: &str| {
                if ordered {
                    format!("{s}.sorted()")
                } else {
                    s.to_string()
                }
            };
            match name {
                "len" => return prim(format!("{r}.count"), usize_ty),
                "is_empty" => return prim(format!("{r}.isEmpty"), Ty::Bool),
                "insert" => {
                    self.pending_refine =
                        Some(Ty::Map(Box::new(ty_a(0)), Box::new(ty_a(1)), ordered));
                    return prim(
                        format!("{r}.updateValue({}, forKey: {})", a(1), a(0)),
                        Ty::Option(Box::new(v)),
                    );
                }
                "get" => return prim(format!("{r}[{}]", a(0)), Ty::Option(Box::new(v))),
                "get_mut" => return prim(format!("{r}[{}]", a(0)), Ty::Option(Box::new(v))),
                "contains_key" => return Ex::new(format!("{r}[{}] != nil", a(0)), P_CMP, Ty::Bool),
                "remove" => {
                    return prim(
                        format!("{r}.removeValue(forKey: {})", a(0)),
                        Ty::Option(Box::new(v)),
                    );
                }
                "keys" | "into_keys" => {
                    return prim(sorted(&format!("{r}.keys")), Ty::Iter(Box::new(k)));
                }
                "values" | "into_values" | "values_mut" => {
                    if ordered {
                        return prim(
                            format!("{r}.sorted(by: {{ $0.key < $1.key }}).map(\\.value)"),
                            Ty::Iter(Box::new(v)),
                        );
                    }
                    return prim(format!("{r}.values"), Ty::Iter(Box::new(v)));
                }
                "clear" => return prim(format!("{r}.removeAll()"), Ty::Unit),
                "extend" => {
                    return prim(
                        format!("{r}.merge({}, uniquingKeysWith: {{ _, new in new }})", a(0)),
                        Ty::Unit,
                    );
                }
                "retain" => {
                    self.helper("map");
                    return prim(format!("{r}.retain({})", a(0)), Ty::Unit);
                }
                "iter_mut" => return recv,
                // `map.range(a..b)` of a `BTreeMap`: the entries in order, within the bounds.
                "range" if ordered && args.len() == 1 => {
                    return prim(
                        format!(
                            "{r}.sorted(by: {{ $0.key < $1.key }}).filter {{ ({}).contains($0.key) }}",
                            a(0)
                        ),
                        Ty::Iter(Box::new(Ty::Tuple(vec![k, v]))),
                    );
                }
                "first_key_value" => {
                    return prim(
                        format!("{r}.min(by: {{ $0.key < $1.key }}).map {{ ($0.key, $0.value) }}"),
                        Ty::Option(Box::new(Ty::Tuple(vec![k, v]))),
                    );
                }
                "last_key_value" => {
                    return prim(
                        format!("{r}.max(by: {{ $0.key < $1.key }}).map {{ ($0.key, $0.value) }}"),
                        Ty::Option(Box::new(Ty::Tuple(vec![k, v]))),
                    );
                }
                _ => {}
            }
        }
        // Sets.
        if let Ty::Set(t, ordered) = &rty {
            let (t, ordered) = ((**t).clone(), *ordered);
            match name {
                "insert" => {
                    self.pending_refine = Some(Ty::Set(Box::new(ty_a(0)), ordered));
                    return prim(format!("{r}.insert({}).inserted", a(0)), Ty::Bool);
                }
                "remove" => {
                    return Ex::new(format!("{r}.remove({}) != nil", a(0)), P_CMP, Ty::Bool);
                }
                "contains" => return prim(format!("{r}.contains({})", a(0)), Ty::Bool),
                "len" => return prim(format!("{r}.count"), usize_ty),
                "is_empty" => return prim(format!("{r}.isEmpty"), Ty::Bool),
                // `set.range(a..b)` of a `BTreeSet`: the elements in order, within the bounds.
                "range" if ordered && args.len() == 1 => {
                    return prim(
                        format!("{r}.sorted().filter {{ ({}).contains($0) }}", a(0)),
                        Ty::Iter(Box::new(t)),
                    );
                }
                "union" => return prim(format!("{r}.union({})", a(0)), rty.clone()),
                "intersection" => return prim(format!("{r}.intersection({})", a(0)), rty.clone()),
                "difference" => return prim(format!("{r}.subtracting({})", a(0)), rty.clone()),
                "symmetric_difference" => {
                    return prim(format!("{r}.symmetricDifference({})", a(0)), rty.clone());
                }
                "is_subset" => return prim(format!("{r}.isSubset(of: {})", a(0)), Ty::Bool),
                "is_superset" => return prim(format!("{r}.isSuperset(of: {})", a(0)), Ty::Bool),
                "is_disjoint" => return prim(format!("{r}.isDisjoint(with: {})", a(0)), Ty::Bool),
                "extend" => return prim(format!("{r}.formUnion({})", a(0)), Ty::Unit),
                "clear" => return prim(format!("{r}.removeAll()"), Ty::Unit),
                "retain" => {
                    return prim(
                        format!("{r}.formIntersection({r}.filter({}))", a(0)),
                        Ty::Unit,
                    );
                }
                _ if ordered && matches!(name, "first" | "last") => {
                    let f = if name == "first" { "min" } else { "max" };
                    return prim(format!("{r}.{f}()"), Ty::Option(Box::new(t)));
                }
                _ => {}
            }
        }
        // Binary heaps.
        if let Ty::Named(n, targs) = &rty {
            if n == "RsBinaryHeap" {
                self.helper("heap");
                let t = targs.first().cloned().unwrap_or(Ty::Unknown);
                let usize_ty = Ty::Int(IntKind::Usize);
                // Tuples are stored made comparable (`RsOrd2`), and given back as tuples.
                let back = if crate::ty::ord_tuple_arity(&t).is_some() {
                    ".map { $0.tuple }"
                } else {
                    ""
                };
                match name {
                    "push" => {
                        self.pending_refine = Some(Ty::Named("RsBinaryHeap".into(), vec![ty_a(0)]));
                        let x = match crate::ty::ord_tuple_arity(&ty_a(0)) {
                            Some(n) => format!("RsOrd{n}({})", a(0)),
                            None => a(0),
                        };
                        return prim(format!("{r}.push({x})"), Ty::Unit);
                    }
                    "pop" => return prim(format!("{r}.pop(){back}"), Ty::Option(Box::new(t))),
                    "peek" => return prim(format!("{r}.peek{back}"), Ty::Option(Box::new(t))),
                    "len" => return prim(format!("{r}.count"), usize_ty),
                    "is_empty" => return prim(format!("{r}.isEmpty"), Ty::Bool),
                    "into_sorted_vec" => {
                        return prim(format!("{r}.intoSortedVec(){back}"), Ty::Vec(Box::new(t)));
                    }
                    "into_vec" | "as_slice" => {
                        return prim(format!("{r}.items{back}"), Ty::Vec(Box::new(t)));
                    }
                    "drain" => return prim(format!("{r}.drain(){back}"), Ty::Vec(Box::new(t))),
                    "iter" | "into_iter" => {
                        return prim(format!("{r}.items{back}"), Ty::Iter(Box::new(t)));
                    }
                    "clear" => return prim(format!("{r}.clear()"), Ty::Unit),
                    "extend" => return prim(format!("{r}.extend({})", a(0)), Ty::Unit),
                    "append" => return prim(format!("{r}.append(&{})", a(0)), Ty::Unit),
                    "retain" => return prim(format!("{r}.retain({})", a(0)), Ty::Unit),
                    _ => {}
                }
            }
        }
        // Vectors, ranges and iterators.
        if is_seq || is_str || rty.is_unknown() || matches!(rty, Ty::Iter(_)) {
            if let Some(ex) = self.seq_method(&recv, name, &args, turbofish, expected) {
                return ex;
            }
        }
        // Comparisons on any type.
        match name {
            "cmp" | "partial_cmp" | "total_cmp" => {
                self.helper("Ordering");
                let f = if name == "partial_cmp" {
                    "rsPartialCmp"
                } else {
                    "rsCmp"
                };
                let ty = if name == "partial_cmp" {
                    Ty::Option(Box::new(Ty::Named("Ordering".into(), vec![])))
                } else {
                    Ty::Named("Ordering".into(), vec![])
                };
                return prim(format!("{f}({}, {})", recv.code, a(0)), ty);
            }
            "eq" => {
                return Ex::new(
                    format!(
                        "{} == {}",
                        wrap(&recv, P_CMP + 1),
                        wrap(&args[0], P_CMP + 1)
                    ),
                    P_CMP,
                    Ty::Bool,
                );
            }
            "ne" => {
                return Ex::new(
                    format!(
                        "{} != {}",
                        wrap(&recv, P_CMP + 1),
                        wrap(&args[0], P_CMP + 1)
                    ),
                    P_CMP,
                    Ty::Bool,
                );
            }
            "lt" | "le" | "gt" | "ge" => {
                let op = match name {
                    "lt" => "<",
                    "le" => "<=",
                    "gt" => ">",
                    _ => ">=",
                };
                return Ex::new(
                    format!(
                        "{} {op} {}",
                        wrap(&recv, P_CMP + 1),
                        wrap(&args[0], P_CMP + 1)
                    ),
                    P_CMP,
                    Ty::Bool,
                );
            }
            "reverse" if rty == Ty::Named("Ordering".into(), vec![]) => {
                return prim(format!("{r}.reverse()"), rty);
            }
            "then" if rty == Ty::Named("Ordering".into(), vec![]) => {
                return prim(format!("{r}.then({})", a(0)), rty);
            }
            "then_with" if rty == Ty::Named("Ordering".into(), vec![]) => {
                return prim(format!("{r}.thenWith({})", a(0)), rty);
            }
            "is_lt" | "is_le" | "is_gt" | "is_ge" | "is_eq" | "is_ne" => {
                self.helper("Ordering");
                return prim(format!("{r}.{}", names::lower_camel(name)), Ty::Bool);
            }
            "hash" => return prim("()".into(), Ty::Unit),
            "len" => return prim(format!("{r}.count"), usize_ty),
            "is_empty" => return prim(format!("{r}.isEmpty"), Ty::Bool),
            _ => {}
        }
        // Anything else: same name, in lowerCamelCase.
        if !rty.is_unknown() && !matches!(rty, Ty::Named(..) | Ty::Dyn(_) | Ty::Opaque(_)) {
            self.warn(
                span,
                format!(
                    "no Swift equivalent known for `.{name}()` on `{}`: translated as is",
                    rty.swift_lossy()
                ),
            );
        }
        let codes: Vec<String> = args.iter().map(|a| a.code.clone()).collect();
        // A method of `self` in a protocol extension: no `self.` needed.
        if recv.code == "self"
            && self
                .imp
                .as_ref()
                .is_some_and(|i| i.kind != TypeKind::Observable)
        {
            return prim(
                format!("{}({})", names::lower_camel(name), codes.join(", ")),
                Ty::Unknown,
            );
        }
        prim(
            format!("{r}.{}({})", names::lower_camel(name), codes.join(", ")),
            Ty::Unknown,
        )
    }

    /// Methods shared by vectors, slices, ranges and iterators.
    #[allow(clippy::too_many_lines)]
    fn seq_method(
        &mut self,
        recv: &Ex,
        name: &str,
        args: &[Ex],
        turbofish: &[Ty],
        expected: &Ty,
    ) -> Option<Ex> {
        let mut r = wrap(recv, P_PRIMARY);
        let rty = recv.ty.clone();
        // A sequence without an end (an `impl Iterator` of the crate, `(1..)`): its adapters
        // must be lazy, as in Rust.
        let endless = match &rty {
            Ty::Named(n, _) => self.syms.iterators.contains_key(n),
            Ty::Range(_) => r.ends_with("...") || r.ends_with("...)"),
            _ => false,
        };
        if endless
            && matches!(
                name,
                "map"
                    | "filter"
                    | "filter_map"
                    | "flat_map"
                    | "take_while"
                    | "skip_while"
                    | "enumerate"
                    | "skip"
                    | "take"
                    | "step_by"
                    | "inspect"
                    | "scan"
                    | "zip"
                    | "chain"
                    | "map_while"
                    | "flatten"
                    | "peekable"
            )
        {
            r = format!("{r}.lazy");
        }
        let item = rty.item();
        let a = |i: usize| args.get(i).map_or_else(String::new, |e| e.code.clone());
        let ty_a = |i: usize| args.get(i).map_or(Ty::Unknown, |e| e.ty.clone());
        let fn_ret = |i: usize| match args.get(i).map(|e| &e.ty) {
            Some(Ty::Fn(_, r)) => (**r).clone(),
            _ => Ty::Unknown,
        };
        let prim = |code: String, ty: Ty| Ex::primary(code, ty);
        let iter = |t: Ty| Ty::Iter(Box::new(t));
        let usize_ty = Ty::Int(IntKind::Usize);
        let is_range = matches!(rty, Ty::Range(_));
        Some(match name {
            "len" => prim(format!("{r}.count"), usize_ty),
            "is_empty" => prim(format!("{r}.isEmpty"), Ty::Bool),
            "capacity" => prim(format!("{r}.capacity"), usize_ty),
            "next_back" if args.is_empty() => prim(format!("{r}.last"), Ty::Option(Box::new(item))),
            "push" | "push_back" => {
                self.pending_refine = Some(Ty::Vec(Box::new(ty_a(0))));
                prim(format!("{r}.append({})", a(0)), Ty::Unit)
            }
            "push_front" => prim(format!("{r}.insert({}, at: 0)", a(0)), Ty::Unit),
            "pop" | "pop_back" => prim(format!("{r}.popLast()"), Ty::Option(Box::new(item))),
            "pop_front" => Ex::new(
                format!("{r}.isEmpty ? nil : {r}.removeFirst()"),
                crate::expr::P_TERNARY,
                Ty::Option(Box::new(item)),
            ),
            "insert" if args.len() == 2 => {
                if matches!(rty, Ty::Vec(_)) {
                    prim(format!("{r}.insert({}, at: {})", a(1), a(0)), Ty::Unit)
                } else {
                    self.pending_refine =
                        Some(Ty::Map(Box::new(ty_a(0)), Box::new(ty_a(1)), false));
                    prim(
                        format!("{r}.updateValue({}, forKey: {})", a(1), a(0)),
                        Ty::Option(Box::new(ty_a(1))),
                    )
                }
            }
            "insert" if args.len() == 1 => prim(format!("{r}.insert({}).inserted", a(0)), Ty::Bool),
            "remove" if matches!(rty, Ty::Vec(_)) || rty.is_unknown() => {
                prim(format!("{r}.remove(at: {})", a(0)), item)
            }
            "swap_remove" => {
                self.helper("vec");
                prim(format!("{r}.swapRemove({})", a(0)), item)
            }
            "truncate" | "dedup" | "resize" | "fill" | "rotate_left" | "rotate_right"
            | "split_off" | "drain" | "retain_mut" | "dedup_by_key" | "dedup_by" => {
                self.helper("vec");
                let codes: Vec<String> = args.iter().map(|a| a.code.clone()).collect();
                let ret = match name {
                    "split_off" | "drain" => Ty::Vec(Box::new(item)),
                    _ => Ty::Unit,
                };
                let call = if name == "drain" && codes.first().is_none_or(|c| c == "...") {
                    "drain()".to_string()
                } else {
                    format!("{}({})", names::lower_camel(name), codes.join(", "))
                };
                prim(format!("{r}.{call}"), ret)
            }
            "clear" => prim(format!("{r}.removeAll()"), Ty::Unit),
            "contains" => prim(format!("{r}.contains({})", a(0)), Ty::Bool),
            "swap" if args.len() == 2 => prim(format!("{r}.swapAt({}, {})", a(0), a(1)), Ty::Unit),
            "reverse" => prim(format!("{r}.reverse()"), Ty::Unit),
            "sort" | "sort_unstable" if matches!(item, Ty::Tuple(_)) => {
                prim(format!("{r}.sort(by: <)"), Ty::Unit)
            }
            "sort" | "sort_unstable" => prim(format!("{r}.sort()"), Ty::Unit),
            "sort_by" | "sort_unstable_by" => {
                self.helper("vec");
                self.helper("Ordering");
                prim(format!("{r}.sortBy({})", a(0)), Ty::Unit)
            }
            // A tuple key (`|w| (w.len(), w)`): Swift compares tuples, but no helper can take them.
            "sort_by_key" | "sort_unstable_by_key" | "sort_by_cached_key"
                if matches!(fn_ret(0), Ty::Tuple(_)) =>
            {
                let k = a(0);
                prim(format!("{r}.sort {{ ({k})($0) < ({k})($1) }}"), Ty::Unit)
            }
            "sort_by_key" | "sort_unstable_by_key" | "sort_by_cached_key" => {
                self.helper("vec");
                prim(format!("{r}.sortByKey({})", a(0)), Ty::Unit)
            }
            "split_first" | "split_last" => {
                let (end, rest) = if name == "split_first" {
                    ("first", "dropFirst")
                } else {
                    ("last", "dropLast")
                };
                prim(
                    format!("{r}.{end}.map {{ ($0, Array({r}.{rest}())) }}"),
                    Ty::Option(Box::new(Ty::Tuple(vec![
                        item.clone(),
                        Ty::Vec(Box::new(item)),
                    ]))),
                )
            }
            "extend" | "extend_from_slice" | "append" => {
                self.pending_refine = Some(Ty::Vec(Box::new(ty_a(0).item())));
                prim(format!("{r}.append(contentsOf: {})", a(0)), Ty::Unit)
            }
            "retain" => {
                self.helper("vec");
                prim(format!("{r}.retain({})", a(0)), Ty::Unit)
            }
            "first" | "front" => prim(format!("{r}.first"), Ty::Option(Box::new(item))),
            "last" | "back" => prim(format!("{r}.last"), Ty::Option(Box::new(item))),
            "next" => prim(format!("{r}.first"), Ty::Option(Box::new(item))),
            "get" if !matches!(rty, Ty::Map(..)) => {
                self.helper("vec");
                prim(format!("{r}.get({})", a(0)), Ty::Option(Box::new(item)))
            }
            "nth" => {
                self.helper("vec");
                prim(format!("{r}.nth({})", a(0)), Ty::Option(Box::new(item)))
            }
            "split_at" => {
                self.helper("vec");
                prim(
                    format!("{r}.splitAt({})", a(0)),
                    Ty::Tuple(vec![rty.clone(), rty.clone()]),
                )
            }
            "windows" | "chunks" | "chunks_exact" => {
                self.helper("vec");
                let f = if name == "windows" {
                    "windows"
                } else {
                    "chunks"
                };
                let code = if name == "chunks_exact" {
                    // Only the full chunks; `remainder()` gives the rest.
                    format!("{r}.chunks({}).filter {{ $0.count == {} }}", a(0), a(0))
                } else {
                    format!("{r}.{f}({})", a(0))
                };
                prim(code, iter(Ty::Vec(Box::new(item))))
            }
            "concat" => {
                if item == Ty::Str {
                    prim(format!("{r}.joined()"), Ty::Str)
                } else {
                    prim(
                        format!("{r}.flatMap {{ $0 }}"),
                        Ty::Vec(Box::new(item.item())),
                    )
                }
            }
            // `[[1, 2], [3]].join(&0)`: a vector of vectors joined by an element or a slice.
            "join" if matches!(item, Ty::Vec(_)) => {
                let sep = if matches!(ty_a(0), Ty::Vec(_)) {
                    a(0)
                } else {
                    format!("[{}]", a(0))
                };
                prim(format!("Array({r}.joined(separator: {sep}))"), item)
            }
            "join" => prim(format!("{r}.joined(separator: {})", a(0)), Ty::Str),
            "starts_with" => prim(format!("{r}.starts(with: {})", a(0)), Ty::Bool),
            "ends_with" => prim(
                format!("{r}.reversed().starts(with: {}.reversed())", a(0)),
                Ty::Bool,
            ),
            "remove_offsets" => prim(format!("{r}.remove(atOffsets: {})", a(0)), Ty::Unit),
            "move_offsets" => prim(
                format!("{r}.move(fromOffsets: {}, toOffset: {})", a(0), a(1)),
                Ty::Unit,
            ),
            "binary_search" => {
                self.helper("binary_search");
                prim(
                    format!("{r}.binarySearch({})", a(0)),
                    Ty::Result(Box::new(usize_ty.clone()), Box::new(usize_ty)),
                )
            }
            "binary_search_by" | "binary_search_by_key" => {
                self.helper("binary_search");
                let call = if name == "binary_search_by" {
                    format!("{r}.binarySearchBy({})", a(0))
                } else {
                    format!("{r}.binarySearchByKey({}, {})", a(0), a(1))
                };
                prim(
                    call,
                    Ty::Result(Box::new(usize_ty.clone()), Box::new(usize_ty)),
                )
            }
            "iter_mut" | "chars" => Ex {
                ty: iter(item),
                ..recv.clone()
            },
            // Iterator adapters.
            "map" => prim(format!("{r}.map({})", a(0)), iter(fn_ret(0))),
            "filter" => prim(format!("{r}.filter({})", a(0)), iter(item)),
            "filter_map" => {
                let ret = match fn_ret(0) {
                    Ty::Option(t) => *t,
                    t => t,
                };
                prim(format!("{r}.compactMap({})", a(0)), iter(ret))
            }
            "flat_map" => prim(format!("{r}.flatMap({})", a(0)), iter(fn_ret(0).item())),
            "flatten" => {
                if matches!(item, Ty::Option(_)) {
                    prim(format!("{r}.compactMap {{ $0 }}"), iter(item.item()))
                } else {
                    prim(format!("{r}.flatMap {{ $0 }}"), iter(item.item()))
                }
            }
            "enumerate" => prim(
                format!("{r}.enumerated()"),
                iter(Ty::Tuple(vec![usize_ty, item])),
            ),
            "zip" => prim(
                format!("{}({}, {})", self.global_fn("zip"), recv.code, a(0)),
                iter(Ty::Tuple(vec![item, ty_a(0).item()])),
            ),
            "rev" => prim(format!("{r}.reversed()"), iter(item)),
            "skip" => prim(format!("{r}.dropFirst({})", a(0)), iter(item)),
            "take" => prim(format!("{r}.prefix({})", a(0)), iter(item)),
            "skip_while" => prim(format!("{r}.drop(while: {})", a(0)), iter(item)),
            "take_while" => prim(format!("{r}.prefix(while: {})", a(0)), iter(item)),
            "step_by" => {
                if is_range {
                    if let Some((lo, hi, closed)) = split_range(&recv.code) {
                        let to = if closed { "through" } else { "to" };
                        return Some(prim(
                            format!("stride(from: {lo}, {to}: {hi}, by: {})", a(0)),
                            iter(item),
                        ));
                    }
                }
                self.helper("vec");
                prim(format!("{r}.stepBy({})", a(0)), iter(item))
            }
            "copy_from_slice" | "clone_from_slice" => {
                self.helper("vec");
                prim(format!("{r}.copyFromSlice({})", a(0)), Ty::Unit)
            }
            "rchunks" => {
                self.helper("vec");
                prim(
                    format!("{r}.rchunks({})", a(0)),
                    iter(Ty::Vec(Box::new(item.clone()))),
                )
            }
            "partition_point" => {
                self.helper("vec");
                prim(
                    format!("{r}.partitionPoint({})", a(0)),
                    Ty::Int(IntKind::Usize),
                )
            }
            "chain" => {
                let other = a(0);
                // Chaining an untyped empty array (`None.iter()`) adds nothing.
                if other == "[]" {
                    return Some(prim(format!("Array({})", recv.code), iter(item)));
                }
                Ex::new(
                    format!("Array({}) + Array({other})", recv.code),
                    P_ADD,
                    iter(item),
                )
            }
            "cycle" => {
                self.helper("iter");
                prim(format!("rsCycle({})", recv.code), iter(item))
            }
            "peekable" | "fuse" => recv.clone(),
            "inspect" => recv.clone(),
            "sum" | "product" => {
                let mut t = turbofish
                    .first()
                    .cloned()
                    .filter(|t| !t.is_unknown())
                    .or_else(|| (!item.is_unknown()).then(|| item.clone()))
                    .unwrap_or_else(|| expected.clone());
                t.merge(expected);
                let scalar = match &t {
                    Ty::Option(inner) | Ty::Result(inner, _) => (**inner).clone(),
                    t => t.clone(),
                };
                let (zero, op) = match (name, &scalar) {
                    ("sum", Ty::Float(_)) => ("0.0", "+"),
                    ("sum", _) => ("0", "+"),
                    (_, Ty::Float(_)) => ("1.0", "*"),
                    _ => ("1", "*"),
                };
                // `Option`s and `Result`s add up to the first `None` or `Err`.
                match (&t, t.swift()) {
                    (Ty::Option(_) | Ty::Result(..), Some(wrapper)) => {
                        self.note_types(&t);
                        // `vec![Ok(2), Ok(3)]`: the literal learns its type here.
                        let r = if recv.code.starts_with('[') && !item.is_complete() {
                            format!("({} as [{wrapper}])", recv.code)
                        } else {
                            r.clone()
                        };
                        let start = if t.is_option() {
                            format!("{wrapper}({zero})")
                        } else {
                            format!("{wrapper}.success({zero})")
                        };
                        prim(
                            format!(
                                "{r}.reduce({start}) {{ acc, x in acc.flatMap {{ a in x.map {{ a {op} $0 }} }} }}"
                            ),
                            t,
                        )
                    }
                    _ => prim(format!("{r}.reduce({zero}, {op})"), t),
                }
            }
            "count" => prim(format!("{r}.count"), usize_ty),
            "min" | "max" if args.is_empty() && matches!(item, Ty::Tuple(_)) => {
                prim(format!("{r}.{name}(by: <)"), Ty::Option(Box::new(item)))
            }
            "min" | "max" if args.is_empty() => {
                prim(format!("{r}.{name}()"), Ty::Option(Box::new(item)))
            }
            "min" | "max" if args.len() == 1 => prim(
                format!("{}({}, {})", self.global_fn(name), recv.code, a(0)),
                recv.ty.clone(),
            ),
            "min_by_key" | "max_by_key" if matches!(fn_ret(0), Ty::Tuple(_)) => {
                self.helper("vec");
                self.helper("Ordering");
                let k = a(0);
                let f = if name == "min_by_key" {
                    "minBy"
                } else {
                    "maxBy"
                };
                prim(
                    format!(
                        "{r}.{f} {{ ({k})($0) < ({k})($1) ? .less : (({k})($1) < ({k})($0) ? .greater : .equal) }}"
                    ),
                    Ty::Option(Box::new(item)),
                )
            }
            "min_by_key" | "max_by_key" | "min_by" | "max_by" => {
                self.helper("vec");
                if name.ends_with("by") {
                    self.helper("Ordering");
                }
                prim(
                    format!("{r}.{}({})", names::lower_camel(name), a(0)),
                    Ty::Option(Box::new(item)),
                )
            }
            "fold" => {
                let t = ty_a(0);
                prim(format!("{r}.reduce({}, {})", a(0), a(1)), t)
            }
            "scan" => {
                self.helper("vec");
                let ret = match fn_ret(1) {
                    Ty::Option(t) => *t,
                    t => t,
                };
                prim(format!("{r}.rsScan({}, {})", a(0), a(1)), iter(ret))
            }
            "reduce" => {
                self.helper("vec");
                prim(
                    format!("{r}.reduceFirst({})", a(0)),
                    Ty::Option(Box::new(item)),
                )
            }
            "any" => prim(format!("{r}.contains(where: {})", a(0)), Ty::Bool),
            "all" => prim(format!("{r}.allSatisfy({})", a(0)), Ty::Bool),
            "find" => prim(
                format!("{r}.first(where: {})", a(0)),
                Ty::Option(Box::new(item)),
            ),
            "find_map" => {
                let ret = match fn_ret(0) {
                    Ty::Option(t) => *t,
                    t => t,
                };
                prim(
                    format!("{r}.lazy.compactMap({}).first", a(0)),
                    Ty::Option(Box::new(ret)),
                )
            }
            // Characters and lazy sequences have no integer index: count instead.
            "position" | "rposition"
                if recv.code.contains(".rsChars") || recv.code.contains(".lazy") =>
            {
                self.helper("vec");
                let f = if name == "position" {
                    "rsPosition"
                } else {
                    "rsRposition"
                };
                prim(
                    format!("{r}.{f}(where: {})", a(0)),
                    Ty::Option(Box::new(usize_ty)),
                )
            }
            "position" => prim(
                format!("{r}.firstIndex(where: {})", a(0)),
                Ty::Option(Box::new(usize_ty)),
            ),
            "rposition" => prim(
                format!("{r}.lastIndex(where: {})", a(0)),
                Ty::Option(Box::new(usize_ty)),
            ),
            "for_each" => prim(format!("{r}.forEach({})", a(0)), Ty::Unit),
            "partition" => {
                self.helper("vec");
                prim(
                    format!("{r}.partitioned({})", a(0)),
                    Ty::Tuple(vec![
                        Ty::Vec(Box::new(item.clone())),
                        Ty::Vec(Box::new(item)),
                    ]),
                )
            }
            "unzip" => {
                let (x, y) = match &item {
                    Ty::Tuple(ts) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
                    _ => (Ty::Unknown, Ty::Unknown),
                };
                prim(
                    format!("({r}.map {{ $0.0 }}, {r}.map {{ $0.1 }})"),
                    Ty::Tuple(vec![Ty::Vec(Box::new(x)), Ty::Vec(Box::new(y))]),
                )
            }
            "collect" => return Some(self.collect(recv, turbofish, expected)),
            "is_sorted" => prim(
                format!(
                    "zip({}, {}.dropFirst()).allSatisfy {{ $0 <= $1 }}",
                    recv.code, r
                ),
                Ty::Bool,
            ),
            "group_by" | "chunk_by" => return None,
            _ => return None,
        })
    }

    /// `collect()` into the type given by the turbofish or the context.
    fn collect(&mut self, recv: &Ex, turbofish: &[Ty], expected: &Ty) -> Ex {
        let target = turbofish
            .first()
            .cloned()
            .filter(|t| !t.is_unknown())
            .unwrap_or_else(|| expected.clone());
        let item = recv.ty.item();
        let code = recv.code.clone();
        // These methods already return an `Array`.
        // A lazy sequence (`s.rsChars`, `.lazy`) is not an array, whatever its last call.
        let lazy = code.contains(".rsChars") || code.contains(".lazy");
        let is_array_chain = !lazy
            && ((code.starts_with('[') && code.ends_with(']'))
                || matches!(
                    last_member(&code).as_deref(),
                    Some(
                        "map"
                            | "filter"
                            | "compactMap"
                            | "flatMap"
                            | "sorted"
                            | "components"
                            | "shuffled"
                    )
                ));
        match &target {
            // Pieces of text are joined; characters make a string.
            Ty::Str if item == Ty::Str => Ex::primary(format!("{code}.joined()"), Ty::Str),
            Ty::Str => Ex::primary(format!("String({code})"), Ty::Str),
            // `collect::<Option<Vec<_>>>()`: all or nothing.
            Ty::Option(inner) if matches!(**inner, Ty::Vec(_)) => {
                self.helper("option");
                let mut ty = Ty::Vec(Box::new(item.item()));
                ty.merge(inner);
                Ex::primary(format!("rsAllSome({code})"), Ty::Option(Box::new(ty)))
            }
            Ty::Map(..) => {
                let (k, v) = match &item {
                    Ty::Tuple(ts) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
                    _ => (Ty::Unknown, Ty::Unknown),
                };
                let mut ty = target.clone();
                ty.merge(&Ty::Map(Box::new(k.clone()), Box::new(v), false));
                let key = match &ty {
                    Ty::Map(key, _, _) => (**key).clone(),
                    _ => k,
                };
                Ex::primary(
                    format!(
                        "{}({code}, uniquingKeysWith: {{ _, last in last }})",
                        crate::ty::map_type_name(&key)
                    ),
                    ty,
                )
            }
            Ty::Set(t, ordered) => {
                let mut elem = item;
                elem.merge(t);
                Ex::primary(
                    format!("{}({code})", crate::ty::set_type_name(&elem)),
                    Ty::Set(Box::new(elem), *ordered),
                )
            }
            // `collect::<BinaryHeap<_>>()`.
            Ty::Named(n, args) if n == "RsBinaryHeap" => {
                self.helper("heap");
                let mut t = item;
                t.merge(args.first().unwrap_or(&Ty::Unknown));
                let inner = match crate::ty::ord_tuple_arity(&t) {
                    Some(k) => format!("{code}.map {{ RsOrd{k}($0) }}"),
                    None if is_array_chain => code,
                    None => format!("Array({code})"),
                };
                Ex::primary(
                    format!("RsBinaryHeap({inner})"),
                    Ty::Named("RsBinaryHeap".into(), vec![t]),
                )
            }
            Ty::Result(inner, err) => {
                // The closures throw; the whole chain is a throwing call.
                let mut ex = Ex::primary(code, (**inner).clone());
                ex.throws = Some(Box::new(((**inner).clone(), (**err).clone())));
                ex
            }
            _ => {
                let mut ty = Ty::Vec(Box::new(item));
                ty.merge(&target);
                if is_array_chain {
                    Ex::primary(code, ty)
                } else {
                    Ex::primary(format!("Array({code})"), ty)
                }
            }
        }
    }
}

/// `xs.map({ x in x + 1 })` → `xs.map { x in x + 1 }`, `reduce(0, { … })` → `reduce(0) { … }`,
/// `first(where: { … })` → `first { … }`. Only when the last argument is a closure.
pub fn trailing_closure(code: String) -> String {
    if !code.ends_with("})") {
        return code;
    }
    let chars: Vec<char> = code.chars().collect();
    let last = chars.len() - 1;
    // Find the parenthesis that the final `)` closes, skipping strings and interpolations.
    enum Frame {
        Paren(usize),
        Brace(usize),
        Interp,
        Str,
    }
    let mut stack: Vec<Frame> = Vec::new();
    let mut open_paren = None;
    let mut closure_start = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let in_str = matches!(stack.last(), Some(Frame::Str));
        if in_str {
            match c {
                '\\' if chars.get(i + 1) == Some(&'(') => {
                    stack.push(Frame::Interp);
                    i += 2;
                    continue;
                }
                '\\' => {
                    i += 2;
                    continue;
                }
                '"' => {
                    stack.pop();
                }
                _ => {}
            }
            i += 1;
            continue;
        }
        match c {
            '"' => stack.push(Frame::Str),
            '(' => stack.push(Frame::Paren(i)),
            '{' => stack.push(Frame::Brace(i)),
            ')' => match stack.pop() {
                Some(Frame::Paren(j)) if i == last => open_paren = Some(j),
                Some(Frame::Paren(_)) | Some(Frame::Interp) => {}
                _ => return code,
            },
            '}' => match stack.pop() {
                Some(Frame::Brace(j)) if i == last - 1 => closure_start = Some(j),
                Some(Frame::Brace(_)) => {}
                _ => return code,
            },
            _ => {}
        }
        i += 1;
    }
    let (Some(p), Some(b)) = (open_paren, closure_start) else {
        return code;
    };
    if b <= p {
        return code;
    }
    let callee: String = chars[..p].iter().collect();
    let before: String = chars[p + 1..b].iter().collect();
    let closure: String = chars[b..last].iter().collect();
    let before = before.trim();
    if before.is_empty() || matches!(before, "where:" | "by:" | "while:" | "catching:") {
        return format!("{callee} {closure}");
    }
    if let Some(args) = before.strip_suffix(',') {
        // Only plain arguments before the closure (no label of the closure itself).
        if !args.trim().is_empty() {
            return format!("{callee}({}) {closure}", args.trim());
        }
    }
    code
}

/// The last member called at the top level of `code`: `map` for `xs.filter { … }.map(f)`.
fn last_member(code: &str) -> Option<String> {
    let chars: Vec<char> = code.chars().collect();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut last = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            '.' if depth == 0 => {
                let name: String = chars[i + 1..]
                    .iter()
                    .take_while(|c| c.is_alphanumeric() || **c == '_')
                    .collect();
                if !name.is_empty() {
                    last = Some(name);
                }
            }
            _ => {}
        }
        i += 1;
    }
    last
}

/// Binds the generic parameters of `param` to the parts of `arg`.
fn unify(
    param: &Ty,
    arg: &Ty,
    generics: &[String],
    out: &mut std::collections::HashMap<String, Ty>,
) {
    match (param, arg) {
        (Ty::Named(g, a), _) if a.is_empty() && generics.contains(g) => {
            if arg.is_complete() && !out.contains_key(g) {
                out.insert(g.clone(), arg.clone());
            }
        }
        (Ty::Vec(p), Ty::Vec(a))
        | (Ty::Option(p), Ty::Option(a))
        | (Ty::Set(p, _), Ty::Set(a, _)) => unify(p, a, generics, out),
        (Ty::Map(pk, pv, _), Ty::Map(ak, av, _)) | (Ty::Result(pk, pv), Ty::Result(ak, av)) => {
            unify(pk, ak, generics, out);
            unify(pv, av, generics, out);
        }
        (Ty::Tuple(ps), Ty::Tuple(as_)) => {
            for (p, a) in ps.iter().zip(as_) {
                unify(p, a, generics, out);
            }
        }
        (Ty::Named(n, ps), Ty::Named(m, as_)) if n == m => {
            for (p, a) in ps.iter().zip(as_) {
                unify(p, a, generics, out);
            }
        }
        _ => {}
    }
}

/// Methods every standard type might have: an unknown receiver with such a method is not
/// assumed to be one of the crate's types.
fn std_method_name(name: &str) -> bool {
    matches!(
        name,
        "len"
            | "is_empty"
            | "push"
            | "pop"
            | "iter"
            | "map"
            | "filter"
            | "collect"
            | "clone"
            | "to_string"
            | "unwrap"
            | "insert"
            | "remove"
            | "get"
            | "contains"
            | "next"
    )
}

/// `$0`-free body of a closure with no parameters: `{ 0 }` → `0`.
fn closure_body(code: &str) -> String {
    let t = code.trim();
    if let Some(inner) = t.strip_prefix("{ ").and_then(|s| s.strip_suffix(" }")) {
        if !inner.contains(" in ") && !inner.contains('\n') {
            return inner.to_string();
        }
        if let Some(rest) = inner.strip_prefix("_ in ") {
            return rest.to_string();
        }
    }
    format!("{}()", wrap(&Ex::primary(code, Ty::Unknown), P_PRIMARY + 1))
}

/// `0..<10` → (`0`, `10`, closed).
fn split_range(code: &str) -> Option<(String, String, bool)> {
    let code = code.trim_start_matches('(').trim_end_matches(')');
    if let Some((a, b)) = code.split_once("..<") {
        return Some((a.to_string(), b.to_string(), false));
    }
    if let Some((a, b)) = code.split_once("...") {
        return Some((a.to_string(), b.to_string(), true));
    }
    None
}

fn strip_ref(e: &Expr) -> &Expr {
    match e {
        Expr::Reference(r) => &r.expr,
        e => e,
    }
}

impl Translator<'_> {
    /// The turbofish of a call: `generic_max::<u8>(…)`, `Vec::<i32>::new()`.
    fn generic_args(&self, path: &syn::Path) -> Vec<Ty> {
        let Some(last) = path.segments.last() else {
            return Vec::new();
        };
        let seg = if matches!(last.arguments, syn::PathArguments::None) && path.segments.len() >= 2
        {
            &path.segments[path.segments.len() - 2]
        } else {
            last
        };
        match &seg.arguments {
            syn::PathArguments::AngleBracketed(a) => a
                .args
                .iter()
                .filter_map(|g| match g {
                    syn::GenericArgument::Type(t) => Some(self.ty(t)),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

/// `lock()`, `read()`, `try_borrow_mut()`…: an access to a cell's value that Rust wraps in
/// a `Result` or an `Option`, unwrapped right after.
fn is_cell_access(name: &str) -> bool {
    matches!(
        name,
        "lock"
            | "read"
            | "write"
            | "try_lock"
            | "try_read"
            | "try_write"
            | "try_borrow"
            | "try_borrow_mut"
    )
}
