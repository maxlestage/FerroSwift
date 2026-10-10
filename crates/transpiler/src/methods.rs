//! Calls: functions, constructors, methods of the crate's types, and the methods of the
//! standard library (`Vec`, `String`, `HashMap`, `Option`, iterators…) mapped to Swift's.

use syn::spanned::Spanned;
use syn::{Expr, ExprCall, ExprMethodCall};

use crate::expr::{Ex, P_ADD, P_CMP, P_MUL, P_NIL, P_PRIMARY, P_TRY, wrap};
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
        let turbofish = generic_args(&p.path);
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
                    if matches!(expected, Ty::Option(_)) {
                        return Ex { ty, ..v };
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
                    let args: Vec<String> = c
                        .args
                        .iter()
                        .enumerate()
                        .map(|(i, a)| {
                            let t = st.fields.get(i).map_or(Ty::Unknown, |f| f.ty.clone());
                            let v = self.expr_with(a, &t);
                            self.settle(v).code
                        })
                        .collect();
                    return Ex::primary(
                        format!("{}({})", st.swift, args.join(", ")),
                        Ty::Named(st.swift.clone(), vec![]),
                    );
                }
            }
            if let Some(sig) = self.syms.fns.get(&last).cloned() {
                return self.sig_call(names::lower_camel(&last), &sig, &c.args);
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
            if inout {
                let inner = match a {
                    Expr::Reference(r) => &*r.expr,
                    e => e,
                };
                let ty = self.place_ty(inner);
                let place = self.place(inner);
                out.push(format!("&{place}"));
                tys.push(ty);
                continue;
            }
            let v = self.expr_with(a, &expected);
            let v = self.settle(v);
            out.push(v.code);
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
                Some(Ex::primary(format!("{last}({}, {})", a.code, b.code), a.ty))
            }
            "drop" if args.len() == 1 => Some(Ex::primary("", Ty::Unit)),
            "swap" | "take" | "replace" => {
                self.std_path_call(&["mem".into(), last.into()], args, expected, &[])
            }
            _ => None,
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
                let v = arg(self, 0, &Ty::Unknown);
                let mut ex = Ex::primary(
                    format!("String(bytes: {}, encoding: .utf8)", v.code),
                    Ty::Option(Box::new(Ty::Str)),
                );
                ex.parsed = true;
                Some(ex)
            }
            ("String", "from_utf8_lossy") if args.len() == 1 => {
                let v = arg(self, 0, &Ty::Unknown);
                Some(Ex::primary(
                    format!("String(decoding: {}, as: UTF8.self)", v.code),
                    Ty::Str,
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
                    format!("Set({})", v.code),
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
                        "Dictionary({}, uniquingKeysWith: {{ _, last in last }})",
                        v.code
                    ),
                    Ty::Map(Box::new(k), Box::new(val), owner == "BTreeMap"),
                ))
            }
            (
                "Box" | "Rc" | "Arc" | "RefCell" | "Cell" | "Mutex" | "RwLock" | "Cow",
                "new" | "from",
            ) if args.len() == 1 => Some(arg(self, 0, expected)),
            ("cmp", "max" | "min") | ("std", "max" | "min") if args.len() == 2 => {
                let a = arg(self, 0, expected);
                let b = arg(self, 1, &a.ty);
                Some(Ex::primary(format!("{last}({}, {})", a.code, b.code), a.ty))
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
            (o, "from_str_radix") if args.len() == 2 && IntKind::parse(o).is_some() => {
                let t = crate::ty::from_name(o, vec![]);
                let s = arg(self, 0, &Ty::Str);
                let r = arg(self, 1, &Ty::Int(IntKind::U32));
                let mut ex = Ex::primary(
                    format!("{}({}, radix: Int({}))", t.swift_lossy(), s.code, r.code),
                    Ty::Option(Box::new(t)),
                );
                ex.parsed = true;
                Some(ex)
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
        // `"42".parse().unwrap()` with an `i32` expected: the receiver is an `Option<i32>`.
        let recv_expected = match name.as_str() {
            "unwrap" | "expect" | "unwrap_or" | "unwrap_or_default" | "unwrap_or_else"
                if !expected.is_unknown() =>
            {
                Ty::Option(Box::new(expected.clone()))
            }
            // `text.parse().map_err(…)?` with an `f64` expected: the same target for `parse`.
            "map_err" | "ok" | "or" | "or_else" | "ok_or" | "ok_or_else" => expected.clone(),
            _ => Ty::Unknown,
        };
        // `map.entry(k).or_insert_with(Vec::new).push(x)`: the receiver is a place.
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
        // A method of a type of the crate.
        if let Some((mut sig, owner)) = self.user_method(&recv.ty, &name) {
            let recv = self.settle(recv);
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
            let mut ex = self.sig_result(format!("{callee}({})", codes.join(", ")), &sig);
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
        let turbofish: Vec<Ty> = m
            .turbofish
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
            .unwrap_or_default();
        // Results of throwing calls.
        if recv.throws.is_some() {
            if let Some(ex) = self.throwing_receiver(recv.clone(), &name, m, expected) {
                return ex;
            }
        }
        let recv = self.settle(recv);
        // `self.dismiss.call()` → `dismiss()`.
        if name == "call"
            && m.args.is_empty()
            && recv.ty == Ty::Named("DismissAction".into(), vec![])
        {
            return Ex::primary(format!("{}()", wrap(&recv, P_PRIMARY)), Ty::Unit);
        }
        let args = self.method_args(&recv, &name, m);
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
    fn user_method(&self, ty: &Ty, name: &str) -> Option<(FnSig, String)> {
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
    fn method_args(&mut self, recv: &Ex, name: &str, m: &ExprMethodCall) -> Vec<Ex> {
        let item = recv.ty.item();
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
                (
                    "map" | "filter" | "filter_map" | "flat_map" | "any" | "all" | "find"
                    | "position" | "for_each" | "take_while" | "skip_while" | "max_by_key"
                    | "min_by_key" | "sort_by_key" | "sort_by_cached_key" | "retain" | "inspect"
                    | "find_map" | "partition" | "is_some_and" | "and_then" | "map_or"
                    | "map_or_else",
                    _,
                    _,
                ) => {
                    let param = match (&recv.ty, name) {
                        (Ty::Map(..), "retain") => item.clone(),
                        _ => item.clone(),
                    };
                    if name == "map_or" && i == 0 {
                        Ty::Unknown
                    } else {
                        Ty::Fn(vec![param], Box::new(Ty::Unknown))
                    }
                }
                ("sort_by" | "max_by" | "min_by" | "sort_unstable_by", 0, _) => Ty::Fn(
                    vec![item.clone(), item.clone()],
                    Box::new(Ty::Named("Ordering".into(), vec![])),
                ),
                ("fold", 0, _) => Ty::Unknown,
                ("unwrap_or" | "or", 0, Ty::Option(t)) => (**t).clone(),
                ("unwrap_or_else" | "or_else", 0, Ty::Option(t)) => Ty::Fn(vec![], t.clone()),
                (
                    "min" | "max" | "pow" | "powi" | "powf" | "abs_diff" | "atan2" | "hypot"
                    | "checked_add" | "checked_sub" | "checked_mul" | "checked_div"
                    | "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add"
                    | "saturating_sub" | "saturating_mul" | "rem_euclid" | "div_euclid" | "clamp",
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
                (
                    "push_str" | "contains" | "starts_with" | "ends_with" | "rfind" | "split"
                    | "replace" | "strip_prefix" | "strip_suffix" | "split_once"
                    | "trim_start_matches" | "trim_end_matches" | "matches",
                    _,
                    Ty::Str,
                ) => Ty::Str,
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

        // Option values (and parsed values, which are Swift optionals).
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
                "ok_or" | "ok_or_else" => {
                    let e = if name == "ok_or" {
                        a(0)
                    } else {
                        closure_body(&a(0))
                    };
                    self.helper("option");
                    return prim(
                        format!("{r}.okOr({e})"),
                        Ty::Result(Box::new(inner), Box::new(ty_a(0))),
                    );
                }
                // `s.parse::<T>().map_err(|_| E)`: the optional becomes a `Result`.
                "map_err" if recv.parsed => {
                    self.helper("option");
                    let e = closure_body(&a(0));
                    return prim(
                        format!("{r}.okOr({e})"),
                        Ty::Result(Box::new(inner), Box::new(fn_ret(0))),
                    );
                }
                "ok" | "cloned" | "copied" | "as_ref" | "as_mut" | "as_deref" | "as_deref_mut"
                | "clone" | "map_err" => return recv,
                "flatten" => return prim(format!("{r}.flatMap {{ $0 }}"), inner),
                "iter" | "into_iter" => {
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
                    return prim(
                        format!("{r}.mapError({})", a(0)),
                        Ty::Result(Box::new(ok), Box::new(fn_ret(0))),
                    );
                }
                "and_then" => return prim(format!("{r}.flatMap({})", a(0)), fn_ret(0)),
                "is_ok_and" => {
                    return Ex::new(format!("{r}.ok().map({}) ?? false", a(0)), P_NIL, Ty::Bool);
                }
                _ => {}
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
                let ty = match (&rty, name) {
                    (Ty::Map(..), "iter" | "into_iter") => Ty::Iter(Box::new(item)),
                    (Ty::Range(_), "iter" | "into_iter") => Ty::Iter(Box::new(item)),
                    _ => rty,
                };
                if matches!(recv.ty, Ty::Map(_, _, true)) && matches!(name, "iter" | "into_iter") {
                    return prim(format!("{r}.sorted(by: {{ $0.key < $1.key }})"), ty);
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
                "len" => return prim(format!("{r}.utf8.count"), usize_ty),
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
                "chars" => {
                    return Ex {
                        ty: Ty::Iter(Box::new(Ty::Char)),
                        ..recv
                    };
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
                "to_uppercase" | "to_ascii_uppercase" => {
                    return prim(format!("{r}.uppercased()"), Ty::Str);
                }
                "to_lowercase" | "to_ascii_lowercase" => {
                    return prim(format!("{r}.lowercased()"), Ty::Str);
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
                "trim_start_matches" | "trim_end_matches" => {
                    self.helper("string");
                    let f = if name.starts_with("trim_start") {
                        "trimStartMatches"
                    } else {
                        "trimEndMatches"
                    };
                    return prim(format!("{r}.{f}({})", str_arg(0)), Ty::Str);
                }
                "starts_with" => return prim(format!("{r}.hasPrefix({})", str_arg(0)), Ty::Bool),
                "ends_with" => return prim(format!("{r}.hasSuffix({})", str_arg(0)), Ty::Bool),
                "contains" => return prim(format!("{r}.contains({})", a(0)), Ty::Bool),
                "find" | "rfind" => {
                    self.helper("string");
                    let f = if name == "find" { "find" } else { "rfind" };
                    return prim(
                        format!("{r}.{f}({})", str_arg(0)),
                        Ty::Option(Box::new(usize_ty)),
                    );
                }
                "replace" | "replacen" => {
                    return prim(
                        format!(
                            "{r}.replacingOccurrences(of: {}, with: {})",
                            str_arg(0),
                            a(1)
                        ),
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
                    let code = match &target {
                        Ty::Int(_) | Ty::Float(_) | Ty::Bool => {
                            format!("{}({})", target.swift_lossy(), recv.code)
                        }
                        Ty::Char => format!("({}.count == 1 ? {}.first : nil)", r, r),
                        Ty::Str => format!("Optional({})", recv.code),
                        Ty::Named(n, _) if self.generics.contains(n) => {
                            format!("{n}({})", recv.code)
                        }
                        _ => {
                            self.error(
                                span,
                                "parse(): write the target type, e.g. `s.parse::<i32>()`",
                            );
                            format!("Int({})", recv.code)
                        }
                    };
                    let mut ex = prim(code, Ty::Option(Box::new(target)));
                    ex.parsed = true;
                    return ex;
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
                "is_char_boundary" => return prim("true".into(), Ty::Bool),
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
                "abs" => return prim(format!("abs({})", recv.code), t),
                "min" | "max" if args.len() == 1 => {
                    return prim(format!("{name}({}, {})", recv.code, a(0)), t);
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
                "signum" if float => return prim(format!("({} < 0 ? -1.0 : 1.0)", recv.code), t),
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
                "count_ones" => return prim(format!("{r}.nonzeroBitCount"), Ty::Int(IntKind::U32)),
                "leading_zeros" => {
                    return prim(format!("{r}.leadingZeroBitCount"), Ty::Int(IntKind::U32));
                }
                "trailing_zeros" => {
                    return prim(format!("{r}.trailingZeroBitCount"), Ty::Int(IntKind::U32));
                }
                "is_power_of_two" => {
                    return Ex::new(
                        format!("{} > 0 && {r}.nonzeroBitCount == 1", wrap(&recv, P_CMP + 1)),
                        crate::expr::P_AND,
                        Ty::Bool,
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
                | "div_euclid" | "abs_diff" | "isqrt" | "checked_pow" => {
                    self.helper("int");
                    let ret = if name.starts_with("checked") {
                        Ty::Option(Box::new(t.clone()))
                    } else {
                        t
                    };
                    return prim(format!("{r}.{}({})", names::lower_camel(name), a(0)), ret);
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
        let r = wrap(recv, P_PRIMARY);
        let rty = recv.ty.clone();
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
            | "split_off" | "drain" => {
                self.helper("vec");
                let codes: Vec<String> = args.iter().map(|a| a.code.clone()).collect();
                let ret = match name {
                    "split_off" | "drain" => Ty::Vec(Box::new(item)),
                    _ => Ty::Unit,
                };
                let call = if name == "drain" {
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
            "sort_by_key" | "sort_unstable_by_key" | "sort_by_cached_key" => {
                self.helper("vec");
                prim(format!("{r}.sortByKey({})", a(0)), Ty::Unit)
            }
            "extend" | "extend_from_slice" | "append" => {
                self.pending_refine = Some(Ty::Vec(Box::new(ty_a(0).item())));
                prim(format!("{r}.append(contentsOf: {})", a(0)), Ty::Unit)
            }
            "retain" => {
                self.helper("vec");
                prim(format!("{r}.retain({})", a(0)), Ty::Unit)
            }
            "first" => prim(format!("{r}.first"), Ty::Option(Box::new(item))),
            "last" => prim(format!("{r}.last"), Ty::Option(Box::new(item))),
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
                prim(format!("{r}.{f}({})", a(0)), iter(Ty::Vec(Box::new(item))))
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
                self.helper("vec");
                prim(
                    format!("{r}.binarySearch({})", a(0)),
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
                format!("zip({}, {})", recv.code, a(0)),
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
            "chain" => Ex::new(
                format!("Array({}) + Array({})", recv.code, a(0)),
                P_ADD,
                iter(item),
            ),
            "cycle" | "peekable" | "fuse" => recv.clone(),
            "inspect" => recv.clone(),
            "sum" => {
                let t = turbofish
                    .first()
                    .cloned()
                    .filter(|t| !t.is_unknown())
                    .unwrap_or(item);
                prim(format!("{r}.reduce(0, +)"), t)
            }
            "product" => {
                let t = turbofish
                    .first()
                    .cloned()
                    .filter(|t| !t.is_unknown())
                    .unwrap_or(item);
                prim(format!("{r}.reduce(1, *)"), t)
            }
            "count" => prim(format!("{r}.count"), usize_ty),
            "min" | "max" if args.is_empty() && matches!(item, Ty::Tuple(_)) => {
                prim(format!("{r}.{name}(by: <)"), Ty::Option(Box::new(item)))
            }
            "min" | "max" if args.is_empty() => {
                prim(format!("{r}.{name}()"), Ty::Option(Box::new(item)))
            }
            "min" | "max" if args.len() == 1 => {
                prim(format!("{name}({}, {})", recv.code, a(0)), recv.ty.clone())
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
            "dedup_by_key" | "group_by" | "chunk_by" => return None,
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
        let is_array_chain = code.starts_with('[')
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
            );
        match &target {
            Ty::Str => Ex::primary(format!("String({code})"), Ty::Str),
            Ty::Map(..) => {
                let (k, v) = match &item {
                    Ty::Tuple(ts) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
                    _ => (Ty::Unknown, Ty::Unknown),
                };
                let mut ty = target.clone();
                ty.merge(&Ty::Map(Box::new(k), Box::new(v), false));
                Ex::primary(
                    format!("Dictionary({code}, uniquingKeysWith: {{ _, last in last }})"),
                    ty,
                )
            }
            Ty::Set(_, ordered) => {
                Ex::primary(format!("Set({code})"), Ty::Set(Box::new(item), *ordered))
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

fn generic_args(path: &syn::Path) -> Vec<Ty> {
    let _ = path;
    Vec::new()
}
