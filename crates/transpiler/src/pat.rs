//! Patterns of `match`, `if let`, `while let` and `for`.

use syn::Pat;
use syn::spanned::Spanned;

use crate::expr::swift_string;
use crate::names;
use crate::symbols::VariantShape;
use crate::translate::Translator;
use crate::ty::Ty;

/// The Swift form of a `match` arm's pattern.
#[derive(Default, Debug)]
pub struct CasePats {
    /// After `case` (alternatives separated by commas).
    pub code: String,
    /// Conditions added to `where`.
    pub guards: Vec<String>,
    /// Statements at the start of the arm (bindings Swift patterns cannot express).
    pub lets: Vec<String>,
    /// The pattern matches everything (`_`): the arm can be `default:`.
    pub wildcard: bool,
    /// Variant patterns with bindings inside a slice (`[Token::Word(cmd), ..]`), with the
    /// element they match: a guard using the bindings is evaluated after them.
    pub binding_cases: Vec<(String, String)>,
}

impl Translator<'_> {
    /// Translates an arm's pattern; its bindings are declared in the current scope.
    pub fn case_patterns(&mut self, pat: &Pat, ty: &Ty, subject: &str) -> CasePats {
        let mut out = CasePats::default();
        let alternatives: Vec<&Pat> = match pat {
            Pat::Or(o) => o.cases.iter().collect(),
            p => vec![p],
        };
        let mut codes = Vec::new();
        let mut declared = false;
        for alt in alternatives {
            let mut ctx = PatCx {
                subject: subject.to_string(),
                guards: Vec::new(),
                lets: Vec::new(),
                declare: !declared,
                binding_cases: Vec::new(),
            };
            let code = self.pat(alt, ty, &mut ctx, true);
            declared = true;
            if code == "_" && ctx.guards.is_empty() {
                out.wildcard = true;
            }
            // Nested alternatives (`Some(1 | 2)`) expand into several patterns.
            codes.extend(expand_alternatives(&code));
            out.guards.extend(ctx.guards);
            out.lets.extend(ctx.lets);
            out.binding_cases.extend(ctx.binding_cases);
        }
        out.code = codes.join(", ");
        out
    }

    /// The pattern of a `for` loop or closure parameter: a name, a tuple of names, `_`.
    pub fn binding_pattern(&mut self, pat: &Pat, ty: &Ty) -> String {
        match pat {
            Pat::Ident(id) => {
                let n = self.declare(&id.ident.to_string(), ty.clone());
                if id.mutability.is_some() {
                    self.prelude_body.push(format!("var {n} = {n}"));
                }
                n
            }
            Pat::Wild(_) => "_".into(),
            Pat::Reference(r) => self.binding_pattern(&r.pat, ty),
            Pat::Paren(p) => self.binding_pattern(&p.pat, ty),
            // `|(x,)|`: the tuple of one element is bound whole, `x` stands for its field.
            Pat::Tuple(t) if t.elems.len() == 1 => {
                let elem_ty = match ty {
                    Ty::Tuple(ts) if ts.len() == 1 => ts[0].clone(),
                    _ => Ty::Unknown,
                };
                let whole = self.fresh("tuple");
                if let Pat::Ident(id) = &t.elems[0] {
                    self.declare_alias(&id.ident.to_string(), format!("{whole}._0"), elem_ty);
                }
                whole
            }
            Pat::Tuple(t) => {
                let tys = match ty {
                    Ty::Tuple(ts) => ts.clone(),
                    _ => vec![Ty::Unknown; t.elems.len()],
                };
                let parts: Vec<String> = t
                    .elems
                    .iter()
                    .enumerate()
                    .map(|(i, p)| self.binding_pattern(p, tys.get(i).unwrap_or(&Ty::Unknown)))
                    .collect();
                format!("({})", parts.join(", "))
            }
            Pat::Type(pt) => {
                let t = self.ty(&pt.ty);
                self.binding_pattern(&pt.pat, &t)
            }
            other => {
                self.error(other.span(), "this loop pattern is not supported");
                "_".into()
            }
        }
    }

    fn pat(&mut self, pat: &Pat, ty: &Ty, cx: &mut PatCx, top: bool) -> String {
        match pat {
            Pat::Wild(_) => "_".into(),
            Pat::Rest(_) => "_".into(),
            Pat::Ident(id) => {
                let name = id.ident.to_string();
                // A unit variant brought into scope (`Red =>`), or a constant.
                if id.subpat.is_none() && id.mutability.is_none() && id.by_ref.is_none() {
                    // `None =>` is an identifier pattern to syn.
                    if name == "None" {
                        return ".none".into();
                    }
                    if let Some(case) = self.unit_variant_named(&name, ty) {
                        return case;
                    }
                    if self.syms.consts.contains_key(&name) {
                        return self.const_name(&name);
                    }
                }
                let kw = if id.mutability.is_some() {
                    "var"
                } else {
                    "let"
                };
                match &id.subpat {
                    None => {
                        let n = self.bind(&name, ty, cx);
                        format!("{kw} {n}")
                    }
                    Some((_, sub)) => {
                        // `x @ 1..=5`: bind the value, test the sub-pattern in `where`.
                        let n = self.bind(&name, ty, cx);
                        let mut inner = PatCx {
                            subject: n.clone(),
                            guards: Vec::new(),
                            lets: Vec::new(),
                            declare: cx.declare,
                            binding_cases: Vec::new(),
                        };
                        let sub_code = self.pat(sub, ty, &mut inner, false);
                        if sub_code != "_" {
                            if is_value_pattern(&sub_code) {
                                cx.guards.push(format!("{sub_code} ~= {n}"));
                            } else {
                                cx.guards.push(format!("{{ if case {sub_code} = {n} {{ return true }}; return false }}()"));
                            }
                        }
                        cx.guards.extend(inner.guards);
                        cx.lets.extend(inner.lets);
                        format!("{kw} {n}")
                    }
                }
            }
            Pat::Lit(l) => {
                let v = self.expr_with(
                    &syn::Expr::Lit(l.clone()),
                    &if *ty == Ty::Char {
                        Ty::Char
                    } else {
                        ty.clone()
                    },
                );
                if let syn::Lit::Char(c) = &l.lit {
                    return swift_string(&c.value().to_string());
                }
                v.code
            }
            Pat::Range(r) => {
                let item = match ty {
                    Ty::Char => Ty::Char,
                    t => t.clone(),
                };
                let lo = r.start.as_ref().map(|e| self.expr_with(e, &item).code);
                let hi = r.end.as_ref().map(|e| self.expr_with(e, &item).code);
                let lo = lo.map(|s| char_literal(&s));
                let hi = hi.map(|s| char_literal(&s));
                let op = match r.limits {
                    syn::RangeLimits::HalfOpen(_) => "..<",
                    syn::RangeLimits::Closed(_) => "...",
                };
                let spaced = lo
                    .as_deref()
                    .is_some_and(|a| a.contains('(') || a.starts_with('-'))
                    || hi
                        .as_deref()
                        .is_some_and(|b| b.starts_with('-') || b.contains('('));
                let op = if spaced {
                    format!(" {op} ")
                } else {
                    op.to_string()
                };
                match (lo, hi) {
                    (Some(a), Some(b)) => format!("{a}{op}{b}"),
                    (Some(a), None) => format!("{a}..."),
                    (None, Some(b)) => format!("{}{b}", op.trim()),
                    (None, None) => "_".into(),
                }
            }
            Pat::Reference(r) => self.pat(&r.pat, ty, cx, top),
            Pat::Paren(p) => self.pat(&p.pat, ty, cx, top),
            Pat::Or(o) => {
                let parts: Vec<String> = o
                    .cases
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let saved = cx.declare;
                        cx.declare = saved && i == 0;
                        let s = self.pat(p, ty, cx, false);
                        cx.declare = saved;
                        s
                    })
                    .collect();
                format!("\u{3}{}\u{4}", parts.join("\u{5}"))
            }
            // `(x,)`: the field of the support struct.
            Pat::Tuple(t) if t.elems.len() == 1 && !matches!(t.elems[0], Pat::Rest(_)) => {
                let elem_ty = match ty {
                    Ty::Tuple(ts) if ts.len() == 1 => ts[0].clone(),
                    _ => Ty::Unknown,
                };
                let mut inner = PatCx {
                    binding_cases: Vec::new(),
                    subject: format!("{}._0", cx.subject),
                    guards: Vec::new(),
                    lets: Vec::new(),
                    declare: cx.declare,
                };
                self.field_subpattern(&t.elems[0], &elem_ty, &mut inner, cx);
                "_".into()
            }
            Pat::Tuple(t) => {
                let tys = match ty {
                    Ty::Tuple(ts) => ts.clone(),
                    _ => vec![],
                };
                let rest = t.elems.iter().position(|p| matches!(p, Pat::Rest(_)));
                let arity = tys.len().max(t.elems.len() - usize::from(rest.is_some()));
                let mut parts = Vec::new();
                let mut index = 0;
                for p in &t.elems {
                    if matches!(p, Pat::Rest(_)) {
                        let skip = arity.saturating_sub(t.elems.len() - 1);
                        for _ in 0..skip {
                            parts.push("_".to_string());
                            index += 1;
                        }
                        continue;
                    }
                    let elem_ty = tys.get(index).cloned().unwrap_or(Ty::Unknown);
                    let element = format!("{}.{index}", cx.subject);
                    let saved = std::mem::replace(&mut cx.subject, element);
                    parts.push(self.pat(p, &elem_ty, cx, false));
                    cx.subject = saved;
                    index += 1;
                }
                format!("({})", parts.join(", "))
            }
            Pat::TupleStruct(ts) => self.tuple_struct_pat(ts, ty, cx),
            Pat::Path(p) => {
                let segs: Vec<String> = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                let last = segs.last().cloned().unwrap_or_default();
                if last == "None" && segs.len() == 1 {
                    return ".none".into();
                }
                if segs.len() >= 2 {
                    let owner = self.rust_type_name(&segs[segs.len() - 2]);
                    if let Some(en) = self.syms.enums.get(&owner) {
                        if let Some(v) = en.variant(&last) {
                            return format!(".{}", v.swift);
                        }
                    }
                    if segs[segs.len() - 2] == "Ordering" {
                        self.helper("Ordering");
                        return format!(".{}", names::case_name(&last));
                    }
                    if let Some(c) = self.ui_case(&segs[segs.len() - 2], &last) {
                        return c;
                    }
                }
                if let Some(case) = self.unit_variant_named(&last, ty) {
                    return case;
                }
                let v = self.expr(&syn::Expr::Path(syn::ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: p.path.clone(),
                }));
                v.code
            }
            Pat::Struct(s) => self.struct_pat(s, ty, cx, top),
            Pat::Slice(s) => self.slice_pat(s, ty, cx),
            Pat::Type(pt) => {
                let t = self.ty(&pt.ty);
                self.pat(&pt.pat, &t, cx, top)
            }
            Pat::Const(_) | Pat::Macro(_) | Pat::Verbatim(_) | _ => {
                self.error(pat.span(), "this pattern is not supported");
                "_".into()
            }
        }
    }

    fn bind(&mut self, name: &str, ty: &Ty, cx: &PatCx) -> String {
        if cx.declare {
            self.declare(name, ty.clone())
        } else {
            self.lookup(name)
                .map_or_else(|| names::lower_camel(name), |l| l.swift.clone())
        }
    }

    fn unit_variant_named(&self, name: &str, ty: &Ty) -> Option<String> {
        let Ty::Named(n, _) = ty else { return None };
        let en = self.syms.enums.values().find(|e| &e.swift == n)?;
        let v = en.variant(name)?;
        matches!(v.shape, VariantShape::Unit).then(|| format!(".{}", v.swift))
    }

    fn tuple_struct_pat(&mut self, ts: &syn::PatTupleStruct, ty: &Ty, cx: &mut PatCx) -> String {
        let segs: Vec<String> = ts
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let last = segs.last().cloned().unwrap_or_default();
        let one = |t: &mut Self, cx: &mut PatCx, inner_ty: &Ty, case: &str| -> String {
            let parts: Vec<String> = ts
                .elems
                .iter()
                .map(|p| t.payload_pat(p, inner_ty, "value", cx))
                .collect();
            format!(".{case}({})", parts.join(", "))
        };
        match (segs.len(), last.as_str()) {
            (1, "Some") => {
                let inner = ty.item();
                return one(self, cx, &inner, "some");
            }
            (1, "Ok") => {
                if matches!(ty, Ty::Option(_)) {
                    return one(self, cx, &ty.item(), "some");
                }
                let inner = match ty {
                    Ty::Result(o, _) => (**o).clone(),
                    _ => Ty::Unknown,
                };
                return one(self, cx, &inner, "success");
            }
            (1, "Err") => {
                if matches!(ty, Ty::Option(_)) {
                    return ".none".into();
                }
                let inner = match ty {
                    Ty::Result(_, e) => (**e).clone(),
                    _ => Ty::Unknown,
                };
                return one(self, cx, &inner, "failure");
            }
            _ => {}
        }
        // An enum variant: `Shape::Circle(r)`, or `Circle(r)` after `use Shape::*`.
        let owner = if segs.len() >= 2 {
            Some(self.rust_type_name(&segs[segs.len() - 2]))
        } else {
            match ty {
                Ty::Named(n, _) => self
                    .syms
                    .enums
                    .iter()
                    .find(|(_, e)| &e.swift == n)
                    .map(|(k, _)| k.clone()),
                _ => None,
            }
        };
        if let Some(en) = owner.as_ref().and_then(|o| self.syms.enums.get(o)).cloned() {
            if let Some(v) = en.variant(&last) {
                let field_tys: Vec<Ty> = match &v.shape {
                    VariantShape::Tuple(ts) => ts.clone(),
                    _ => vec![],
                };
                let rest = ts.elems.iter().position(|p| matches!(p, Pat::Rest(_)));
                let mut parts = Vec::new();
                let mut index = 0;
                for p in &ts.elems {
                    if matches!(p, Pat::Rest(_)) {
                        let skip = field_tys.len().saturating_sub(ts.elems.len() - 1);
                        for _ in 0..skip {
                            parts.push("_".to_string());
                            index += 1;
                        }
                        continue;
                    }
                    let fty = field_tys.get(index).cloned().unwrap_or(Ty::Unknown);
                    let code = self.payload_pat(p, &fty, &format!("field{index}"), cx);
                    parts.push(code);
                    index += 1;
                }
                let _ = rest;
                if parts.iter().all(|p| p == "_") {
                    return format!(".{}", v.swift);
                }
                return format!(".{}({})", v.swift, parts.join(", "));
            }
        }
        // A tuple struct: `Meters(x)` binds its fields.
        if let Some(st) = self.syms.structs.get(&self.rust_type_name(&last)).cloned() {
            let subject = cx.subject.clone();
            for (i, p) in ts.elems.iter().enumerate() {
                let Some(f) = st.fields.get(i) else { break };
                let mut inner = PatCx {
                    binding_cases: Vec::new(),
                    subject: format!("{subject}.{}", f.swift),
                    guards: Vec::new(),
                    lets: Vec::new(),
                    declare: cx.declare,
                };
                self.field_subpattern(p, &f.ty, &mut inner, cx);
            }
            return "_".into();
        }
        // `Reverse(x)`: the wrapped value.
        if last == "Reverse" && ts.elems.len() == 1 {
            let inner_ty = match ty {
                Ty::Named(n, args) if n == "RsReverse" => {
                    args.first().cloned().unwrap_or(Ty::Unknown)
                }
                _ => Ty::Unknown,
            };
            let field = if crate::ty::ord_tuple_arity(&inner_ty).is_some() {
                "value.tuple"
            } else {
                "value"
            };
            let mut inner = PatCx {
                binding_cases: Vec::new(),
                subject: format!("{}.{field}", cx.subject),
                guards: Vec::new(),
                lets: Vec::new(),
                declare: cx.declare,
            };
            self.field_subpattern(&ts.elems[0], &inner_ty, &mut inner, cx);
            return "_".into();
        }
        self.error(
            ts.span(),
            format!("unknown variant `{last}` in this pattern"),
        );
        "_".into()
    }

    /// A pattern on the payload of an enum variant. A struct or slice pattern there needs
    /// the payload bound to a name first: `case .click(let at, 1) where at.x == 0`.
    fn payload_pat(&mut self, p: &Pat, ty: &Ty, hint: &str, cx: &mut PatCx) -> String {
        let nested = match p {
            Pat::Struct(st) => {
                let last = st.path.segments.last().map(|s| s.ident.to_string());
                last.is_some_and(|l| self.syms.structs.contains_key(&self.rust_type_name(&l)))
            }
            Pat::TupleStruct(ts) => {
                let last = ts.path.segments.last().map(|s| s.ident.to_string());
                last.is_some_and(|l| {
                    l == "Reverse" || self.syms.structs.contains_key(&self.rust_type_name(&l))
                })
            }
            Pat::Slice(_) => true,
            _ => false,
        };
        if !nested {
            return self.pat(p, ty, cx, false);
        }
        let name = self.fresh(&names::lower_camel(hint));
        let mut inner = PatCx {
            subject: name.clone(),
            guards: Vec::new(),
            lets: Vec::new(),
            declare: cx.declare,
            binding_cases: Vec::new(),
        };
        let code = self.pat(p, ty, &mut inner, false);
        cx.guards.append(&mut inner.guards);
        cx.lets.append(&mut inner.lets);
        if code != "_" {
            cx.guards.push(format!("{code} ~= {name}"));
        }
        format!("let {name}")
    }

    /// `Point { x: 0, y }`: Swift has no struct patterns. Literal fields become `where`
    /// conditions, bound fields `let` statements.
    fn struct_pat(&mut self, s: &syn::PatStruct, ty: &Ty, cx: &mut PatCx, _top: bool) -> String {
        let segs: Vec<String> = s
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let last = segs.last().cloned().unwrap_or_default();
        // An enum variant with named fields: `Shape::Rect { w, h }`.
        let owner = if segs.len() >= 2 {
            Some(self.rust_type_name(&segs[segs.len() - 2]))
        } else {
            match ty {
                Ty::Named(n, _) => self
                    .syms
                    .enums
                    .iter()
                    .find(|(_, e)| &e.swift == n)
                    .map(|(k, _)| k.clone()),
                _ => None,
            }
        };
        if let Some(en) = owner.as_ref().and_then(|o| self.syms.enums.get(o)).cloned() {
            if let Some(v) = en.variant(&last) {
                let VariantShape::Struct(fields) = &v.shape else {
                    return format!(".{}", v.swift);
                };
                let mut parts = Vec::new();
                for (fname, fty) in fields {
                    match s
                        .fields
                        .iter()
                        .find(|f| crate::expr::member_name(&f.member) == *fname)
                    {
                        Some(fp) => {
                            let code = self.payload_pat(&fp.pat, fty, fname, cx);
                            parts.push(code);
                        }
                        None => parts.push("_".into()),
                    }
                }
                if parts.iter().all(|p| p == "_") {
                    return format!(".{}", v.swift);
                }
                return format!(".{}({})", v.swift, parts.join(", "));
            }
        }
        let Some(st) = self.syms.structs.get(&self.rust_type_name(&last)).cloned() else {
            self.error(s.span(), format!("unknown struct `{last}` in this pattern"));
            return "_".into();
        };
        let subject = cx.subject.clone();
        for fp in &s.fields {
            let name = crate::expr::member_name(&fp.member);
            let Some(f) = st.field(&name) else { continue };
            let mut inner = PatCx {
                binding_cases: Vec::new(),
                subject: format!("{subject}.{}", f.swift),
                guards: Vec::new(),
                lets: Vec::new(),
                declare: cx.declare,
            };
            self.field_subpattern(&fp.pat, &f.ty, &mut inner, cx);
        }
        "_".into()
    }

    /// A sub-pattern applied to a field: a binding becomes `let`, anything else a condition.
    fn field_subpattern(&mut self, p: &Pat, ty: &Ty, inner: &mut PatCx, cx: &mut PatCx) {
        match p {
            // `None` on a field: a condition.
            Pat::Ident(id) if id.subpat.is_none() && id.ident == "None" => {
                cx.guards.push(format!("{} == nil", inner.subject));
            }
            Pat::Ident(id)
                if id.subpat.is_none()
                    && self.unit_variant_named(&id.ident.to_string(), ty).is_none() =>
            {
                // The name stands for the field (`p.x`), so that a guard can use it.
                if id.mutability.is_none() {
                    if cx.declare {
                        self.declare_alias(
                            &id.ident.to_string(),
                            inner.subject.clone(),
                            ty.clone(),
                        );
                    }
                    return;
                }
                let n = self.bind(&id.ident.to_string(), ty, inner);
                cx.lets.push(format!("var {n} = {}", inner.subject));
            }
            Pat::Wild(_) | Pat::Rest(_) => {}
            // `(d, node)` on a field: each element in turn.
            Pat::Tuple(t) if !t.elems.iter().any(|p| matches!(p, Pat::Rest(_))) => {
                let tys = match ty {
                    Ty::Tuple(ts) => ts.clone(),
                    _ => vec![Ty::Unknown; t.elems.len()],
                };
                for (i, p) in t.elems.iter().enumerate() {
                    let member = if t.elems.len() == 1 {
                        "_0".to_string()
                    } else {
                        i.to_string()
                    };
                    let mut sub = PatCx {
                        binding_cases: Vec::new(),
                        subject: format!("{}.{member}", inner.subject),
                        guards: Vec::new(),
                        lets: Vec::new(),
                        declare: cx.declare,
                    };
                    self.field_subpattern(p, tys.get(i).unwrap_or(&Ty::Unknown), &mut sub, cx);
                }
            }
            // `Some(x)` on a field: a condition, and `x` stands for the unwrapped field.
            Pat::TupleStruct(ts)
                if ts.path.is_ident("Some")
                    && ts.elems.len() == 1
                    && matches!(&ts.elems[0], Pat::Ident(id) if id.subpat.is_none() && id.mutability.is_none()) =>
            {
                let Pat::Ident(id) = &ts.elems[0] else { return };
                cx.guards.push(format!("{} != nil", inner.subject));
                if cx.declare {
                    self.declare_alias(
                        &id.ident.to_string(),
                        format!("{}!", inner.subject),
                        ty.item(),
                    );
                }
            }
            other => {
                let code = self.pat(other, ty, inner, false);
                if code != "_" {
                    if is_value_pattern(&code) {
                        cx.guards.push(format!("{code} ~= {}", inner.subject));
                    } else if code.contains("let ") || code.contains("var ") {
                        // `[Token::Word(cmd), ..]`: the shape is checked in the guard, the
                        // bindings taken at the start of the arm.
                        cx.guards.push(format!(
                            "{{ if case {} = {} {{ return true }}; return false }}()",
                            wildcard_bindings(&code),
                            inner.subject
                        ));
                        cx.lets.push(format!(
                            "guard case {code} = {} else {{ fatalError() }}",
                            inner.subject
                        ));
                        cx.binding_cases.push((code.clone(), inner.subject.clone()));
                    } else {
                        cx.guards.push(format!(
                            "{{ if case {code} = {} {{ return true }}; return false }}()",
                            inner.subject
                        ));
                    }
                }
                cx.guards.append(&mut inner.guards);
                cx.lets.append(&mut inner.lets);
            }
        }
    }

    /// `[first, .., last]`: conditions on the count, bindings by index.
    fn slice_pat(&mut self, s: &syn::PatSlice, ty: &Ty, cx: &mut PatCx) -> String {
        let item = ty.item();
        let subject = cx.subject.clone();
        let n = s.elems.len();
        let rest_at = s.elems.iter().position(is_rest);
        match rest_at {
            Some(_) => cx.guards.push(format!("{subject}.count >= {}", n - 1)),
            None => cx.guards.push(format!("{subject}.count == {n}")),
        }
        for (i, p) in s.elems.iter().enumerate() {
            if Some(i) == rest_at {
                // `[first, rest @ ..] if rest.is_empty()`: the name stands for the slice, so
                // that the guard can use it before any `let`.
                if let Pat::Ident(id) = p {
                    let rest_ty = Ty::Vec(Box::new(item.clone()));
                    let after = n - 1 - i;
                    if cx.declare {
                        self.declare_alias(
                            &id.ident.to_string(),
                            format!("Array({subject}[{i}..<({subject}.count - {after})])"),
                            rest_ty,
                        );
                    }
                }
                continue;
            }
            let index = match rest_at {
                Some(r) if i > r => format!("{subject}[{subject}.count - {}]", n - i),
                _ => format!("{subject}[{i}]"),
            };
            // `[first, .., last] if first == last`: the names stand for the elements, so
            // that the guard can use them before any `let`.
            if let Pat::Ident(id) = p {
                if id.subpat.is_none()
                    && id.mutability.is_none()
                    && self
                        .unit_variant_named(&id.ident.to_string(), &item)
                        .is_none()
                {
                    if cx.declare {
                        self.declare_alias(&id.ident.to_string(), index, item.clone());
                    }
                    continue;
                }
            }
            let mut inner = PatCx {
                subject: index,
                guards: Vec::new(),
                lets: Vec::new(),
                declare: cx.declare,
                binding_cases: Vec::new(),
            };
            self.field_subpattern(p, &item, &mut inner, cx);
        }
        "_".into()
    }
}

struct PatCx {
    /// The Swift expression matched by the pattern being translated.
    subject: String,
    guards: Vec<String>,
    lets: Vec<String>,
    /// Declare the bindings (false for the second alternative of `A(x) | B(x)`).
    declare: bool,
    /// Variant patterns with bindings on an element, with the element.
    binding_cases: Vec<(String, String)>,
}

fn is_rest(p: &Pat) -> bool {
    match p {
        Pat::Rest(_) => true,
        Pat::Ident(id) => matches!(id.subpat.as_ref().map(|(_, p)| &**p), Some(Pat::Rest(_))),
        _ => false,
    }
}

/// A literal or range pattern, usable with `~=`.
fn is_value_pattern(code: &str) -> bool {
    !code.starts_with('.')
        && !code.contains("let ")
        && !code.contains("var ")
        && !code.contains('(')
        || code.contains("...")
        || code.contains("..<")
}

/// `Character("a")` → `"a"` inside patterns.
fn char_literal(s: &str) -> String {
    s.strip_prefix("Character(")
        .and_then(|r| r.strip_suffix(')'))
        .map_or_else(|| s.to_string(), str::to_string)
}

/// Expands nested alternatives (marked `\u{3}a\u{5}b\u{4}`) into separate patterns.
fn expand_alternatives(code: &str) -> Vec<String> {
    let Some(start) = code.find('\u{3}') else {
        return vec![code.to_string()];
    };
    // Find the matching end.
    let mut depth = 0;
    let mut end = start;
    for (i, c) in code[start..].char_indices() {
        match c {
            '\u{3}' => depth += 1,
            '\u{4}' => {
                depth -= 1;
                if depth == 0 {
                    end = start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let inner = &code[start + '\u{3}'.len_utf8()..end];
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut last = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '\u{3}' => depth += 1,
            '\u{4}' => depth -= 1,
            '\u{5}' if depth == 0 => {
                parts.push(&inner[last..i]);
                last = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&inner[last..]);
    let before = &code[..start];
    let after = &code[end + '\u{4}'.len_utf8()..];
    let mut out = Vec::new();
    for p in parts {
        for expanded in expand_alternatives(&format!("{before}{p}{after}")) {
            out.push(expanded);
        }
    }
    out
}

/// `.word(let cmd)` → `.word(_)`: the pattern with its bindings replaced by wildcards.
fn wildcard_bindings(code: &str) -> String {
    let mut out = String::new();
    let mut rest = code;
    while let Some(pos) = rest.find("let ").or_else(|| rest.find("var ")) {
        let (before, after) = rest.split_at(pos);
        out.push_str(before);
        let name_end = after[4..]
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '`'))
            .map_or(after.len(), |i| 4 + i);
        out.push('_');
        rest = &after[name_end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_nested_alternatives() {
        assert_eq!(
            expand_alternatives(".some(\u{3}1\u{5}2\u{4})"),
            vec![".some(1)", ".some(2)"]
        );
        assert_eq!(
            expand_alternatives("(\u{3}1\u{5}2\u{4}, \u{3}a\u{5}b\u{4})").len(),
            4
        );
        assert_eq!(expand_alternatives("_"), vec!["_"]);
    }
}
