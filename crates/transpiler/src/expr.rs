//! Expressions. Each translates to an [`Ex`]: Swift code, its precedence (to add only the
//! parentheses Swift needs) and its type as far as it is known.

use syn::spanned::Spanned;
use syn::{BinOp, Expr, Lit, Pat, UnOp};

use crate::names;
use crate::stmt::{Sink, has_control_flow, needs_statements};
use crate::symbols::{FieldRole, TypeKind, VariantShape};
use crate::translate::{FnCx, Translator, indent_lines};
use crate::ty::{FloatKind, IntKind, Ty};

// Swift precedence, highest first.
pub const P_PRIMARY: u8 = 100;
pub const P_PREFIX: u8 = 90;
pub const P_SHIFT: u8 = 80;
pub const P_MUL: u8 = 70;
pub const P_ADD: u8 = 60;
pub const P_RANGE: u8 = 50;
pub const P_NIL: u8 = 40;
pub const P_CMP: u8 = 30;
pub const P_AND: u8 = 20;
pub const P_OR: u8 = 10;
pub const P_TERNARY: u8 = 5;
pub const P_TRY: u8 = 4;
pub const P_ASSIGN: u8 = 1;

/// A translated expression.
#[derive(Clone, Debug)]
pub struct Ex {
    pub code: String,
    pub prec: u8,
    pub ty: Ty,
    /// The code calls a function that throws (a Rust function returning `Result`), without
    /// `try` yet: (success type, error type).
    pub throws: Option<Box<(Ty, Ty)>>,
    /// A Swift optional standing for a Rust `Result` whose error does not matter
    /// (`s.parse::<i32>()` is `Int(s)`).
    pub parsed: bool,
    /// A number literal without suffix: its type comes from the context.
    pub weak: bool,
}

impl Ex {
    pub fn new(code: impl Into<String>, prec: u8, ty: Ty) -> Ex {
        Ex {
            code: code.into(),
            prec,
            ty,
            throws: None,
            parsed: false,
            weak: false,
        }
    }

    pub fn primary(code: impl Into<String>, ty: Ty) -> Ex {
        Ex::new(code, P_PRIMARY, ty)
    }
}

/// The code of `ex`, in parentheses if its precedence is below `min`.
pub fn wrap(ex: &Ex, min: u8) -> String {
    if ex.prec < min {
        format!("({})", ex.code)
    } else {
        ex.code.clone()
    }
}

/// A Swift string literal.
pub fn swift_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl Translator<'_> {
    pub fn expr(&mut self, e: &Expr) -> Ex {
        self.expr_with(e, &Ty::Unknown)
    }

    /// Translates `e`, knowing the type the context expects (for `None`, `vec![]`, literals…).
    /// Translates `e`, knowing the type the context expects (for `None`, `vec![]`, literals…).
    /// A number of another numeric type is converted, as Rust's inference would have typed it.
    pub fn expr_with(&mut self, e: &Expr, expected: &Ty) -> Ex {
        // Deeply nested code (or a translation bug looping) must not overflow the stack: in
        // the browser playground that would kill the page.
        if self.depth > 120 {
            let code = self.unsupported(e.span(), "an expression nested this deeply");
            return Ex::primary(code, Ty::Unknown);
        }
        self.depth += 1;
        let v = self.expr_raw(e, expected);
        self.depth -= 1;
        self.coerce(v, expected)
    }

    /// Converts a number to the numeric type the context expects.
    pub fn coerce(&mut self, mut v: Ex, expected: &Ty) -> Ex {
        if v.throws.is_some() || !v.ty.is_numeric() || !expected.is_numeric() {
            return v;
        }
        let (Some(have), Some(want)) = (v.ty.swift(), expected.swift()) else {
            return v;
        };
        if have == want {
            return v;
        }
        if v.weak {
            let int_literal = v.ty.is_int() && !v.code.contains('.') && !v.code.contains('e');
            if expected.is_int() && v.ty.is_int()
                || expected.is_float() && (v.ty.is_float() || int_literal)
            {
                v.ty = expected.clone();
                return v;
            }
        }
        if v.ty.is_int() == expected.is_int() {
            return Ex::primary(format!("{want}({})", v.code), expected.clone());
        }
        v
    }

    fn expr_raw(&mut self, e: &Expr, expected: &Ty) -> Ex {
        match e {
            Expr::Lit(l) => self.lit(&l.lit, expected),
            Expr::Path(p) => self.path_expr(&p.path, expected, e.span()),
            Expr::Binary(b) => self.binary(b, expected),
            Expr::Unary(u) => self.unary(u, expected),
            Expr::Paren(p) => {
                let mut inner = self.expr_with(&p.expr, expected);
                if inner.prec < P_PRIMARY && matches!(&*p.expr, Expr::Cast(_)) {
                    inner.code = format!("({})", inner.code);
                    inner.prec = P_PRIMARY;
                }
                inner
            }
            Expr::Group(g) => self.expr_with(&g.expr, expected),
            Expr::Reference(r) => self.expr_with(&r.expr, expected),
            Expr::Call(c) => self.call(c, expected),
            Expr::MethodCall(m) => self.method_call(m, expected),
            Expr::Field(f) => self.field(f),
            Expr::Index(i) => self.index(i),
            Expr::Tuple(t) => {
                if t.elems.is_empty() {
                    return Ex::primary("()", Ty::Unit);
                }
                let expected_elems = match expected {
                    Ty::Tuple(ts) => ts.clone(),
                    _ => vec![],
                };
                let mut tys = Vec::new();
                let mut codes = Vec::new();
                for (i, el) in t.elems.iter().enumerate() {
                    let v = self.expr_with(el, expected_elems.get(i).unwrap_or(&Ty::Unknown));
                    let v = self.settle(v);
                    tys.push(v.ty.clone());
                    codes.push(v.code);
                }
                Ex::primary(format!("({})", codes.join(", ")), Ty::Tuple(tys))
            }
            Expr::Array(a) => {
                let item = expected.item();
                let mut it = item.clone();
                let mut codes = Vec::new();
                for el in &a.elems {
                    let v = self.expr_with(el, &item);
                    let v = self.settle(v);
                    it.merge(&v.ty);
                    codes.push(v.code);
                }
                Ex::primary(format!("[{}]", codes.join(", ")), Ty::Vec(Box::new(it)))
            }
            Expr::Repeat(r) => {
                let item = expected.item();
                let v = self.expr_with(&r.expr, &item);
                let v = self.settle(v);
                let n = self.expr_with(&r.len, &Ty::Int(IntKind::Usize));
                Ex::primary(
                    format!("Array(repeating: {}, count: {})", v.code, n.code),
                    Ty::Vec(Box::new(v.ty)),
                )
            }
            Expr::Struct(s) => self.struct_lit(s, expected),
            Expr::Closure(c) => self.closure(c, expected),
            Expr::Range(r) => self.range(r, expected),
            Expr::Cast(c) => self.cast(c),
            Expr::Try(t) => self.try_expr(t, expected),
            Expr::Macro(m) => self.macro_expr(&m.mac, expected),
            Expr::Assign(a) => {
                let rhs_expected = self.place_ty(&a.left);
                let lhs = self.place(&a.left);
                let rhs = self.expr_with(&a.right, &rhs_expected);
                let rhs = self.settle(rhs);
                if let Expr::Path(p) = &*a.left {
                    if let Some(id) = p.path.get_ident() {
                        let id = id.to_string();
                        self.refine(&id, &rhs.ty);
                    }
                }
                Ex::new(format!("{lhs} = {}", rhs.code), P_ASSIGN, Ty::Unit)
            }
            Expr::Await(a) => {
                let v = self.expr_with(&a.base, expected);
                let mut ex = Ex::new(format!("await {}", wrap(&v, P_PREFIX)), P_TRY, v.ty);
                // `f().await` of an async function returning a `Result` still throws.
                ex.throws = v.throws;
                ex
            }
            Expr::Block(b) if b.label.is_some() => {
                let code = self.unsupported(
                    e.span(),
                    "a labeled block used as a value: bind it with `let x: T = 'a: { … };`",
                );
                Ex::primary(code, Ty::Unknown)
            }
            Expr::If(_) | Expr::Match(_) | Expr::Block(_) | Expr::Unsafe(_) | Expr::Loop(_) => {
                self.statements_as_value(e, expected)
            }
            Expr::Return(_) | Expr::Break(_) | Expr::Continue(_) => {
                let code = self.unsupported(e.span(), "`return`, `break` or `continue` inside an expression (use a `let` with a type annotation, or a `match` statement)");
                Ex::primary(code, Ty::Never)
            }
            Expr::Let(_) => {
                let code = self.unsupported(e.span(), "`let` in this position");
                Ex::primary(code, Ty::Bool)
            }
            Expr::ForLoop(_) | Expr::While(_) => {
                let code = self.unsupported(e.span(), "a loop used as a value");
                Ex::primary(code, Ty::Unit)
            }
            _ => {
                let code = self.unsupported(e.span(), "this expression");
                Ex::primary(code, Ty::Unknown)
            }
        }
    }

    /// A value usable anywhere: a throwing call becomes a Swift `Result` value.
    pub fn settle(&mut self, mut v: Ex) -> Ex {
        if let Some(t) = v.throws.take() {
            let (ok, err) = *t;
            if v.code.starts_with("await ") {
                // `Result(catching:)` takes no async closure.
                self.helper("async_result");
                let effects = match err.swift() {
                    Some(e) if !matches!(err, Ty::Dyn(_)) => format!("async throws({e})"),
                    _ => "async throws".into(),
                };
                let ret = ok.swift().map(|o| format!(" -> {o}")).unwrap_or_default();
                let code = format!(
                    "await rsAsyncResult {{ () {effects}{ret} in try {} }}",
                    v.code
                );
                return Ex::new(code, P_TRY, Ty::Result(Box::new(ok), Box::new(err)));
            }
            let code = match (ok.swift(), err.swift()) {
                (Some(o), Some(e)) if !matches!(err, Ty::Dyn(_)) => {
                    format!(
                        "Result<{o}, {e}> {{ () throws({e}) -> {o} in try {} }}",
                        v.code
                    )
                }
                _ => format!("Result {{ try {} }}", v.code),
            };
            return Ex::primary(code, Ty::Result(Box::new(ok), Box::new(err)));
        }
        v
    }

    pub fn settle_for_let(&mut self, v: Ex) -> Ex {
        self.settle(v)
    }

    /// `try? call`: the result of a throwing call as an optional.
    pub fn try_optional(&mut self, mut v: Ex) -> Ex {
        if let Some(t) = v.throws.take() {
            return Ex::new(format!("try? {}", v.code), P_TRY, Ty::Option(Box::new(t.0)));
        }
        if v.ty.is_result() {
            let ok = match &v.ty {
                Ty::Result(o, _) => (**o).clone(),
                _ => Ty::Unknown,
            };
            return Ex::new(
                format!("try? {}.get()", wrap(&v, P_PRIMARY)),
                P_TRY,
                Ty::Option(Box::new(ok)),
            );
        }
        v
    }

    // ----- Literals -----

    fn lit(&mut self, lit: &Lit, expected: &Ty) -> Ex {
        match lit {
            Lit::Int(i) => {
                let suffix = i.suffix();
                let text = i.to_string();
                let digits = text[..text.len() - suffix.len()]
                    .trim_end_matches('_')
                    .to_string();
                let ty = match suffix {
                    "f32" => Ty::Float(FloatKind::F32),
                    "f64" => Ty::Float(FloatKind::F64),
                    s if !s.is_empty() => IntKind::parse(s).map_or(Ty::Int(IntKind::I32), Ty::Int),
                    _ => match expected {
                        Ty::Int(_) | Ty::Float(_) => expected.clone(),
                        _ => Ty::Int(IntKind::I32),
                    },
                };
                let code = if ty.is_float() && suffix.starts_with('f') {
                    format!("{digits}.0")
                } else {
                    digits
                };
                let mut ex = Ex::primary(code, ty.clone());
                if suffix.is_empty() {
                    ex.weak = true;
                } else if !matches!(ty.swift().as_deref(), Some("Int" | "Double"))
                    && expected.swift() != ty.swift()
                {
                    // `5u8` → `UInt8(5)`: Swift would make a bare literal an `Int`.
                    ex.code = format!("{}({})", ty.swift_lossy(), ex.code);
                }
                ex
            }
            Lit::Float(f) => {
                let suffix = f.suffix();
                let text = f.to_string();
                let mut digits = text[..text.len() - suffix.len()]
                    .trim_end_matches('_')
                    .to_string();
                if digits.ends_with('.') {
                    digits.push('0');
                }
                let ty = match suffix {
                    "f32" => Ty::Float(FloatKind::F32),
                    "f64" => Ty::Float(FloatKind::F64),
                    _ => match expected {
                        Ty::Float(_) => expected.clone(),
                        _ => Ty::Float(FloatKind::F64),
                    },
                };
                if self.implicit_member {
                    // `16.0` → `16` in SwiftUI arguments.
                    if let Some(int) = digits.strip_suffix(".0") {
                        if int.chars().all(|c| c.is_ascii_digit()) {
                            digits = int.to_string();
                        }
                    }
                }
                let mut ex = Ex::primary(digits, ty.clone());
                if suffix.is_empty() {
                    ex.weak = true;
                } else if ty.swift().as_deref() == Some("Float") && expected.swift() != ty.swift() {
                    ex.code = format!("Float({})", ex.code);
                }
                ex
            }
            Lit::Str(s) => Ex::primary(swift_string(&s.value()), Ty::Str),
            Lit::Char(c) => {
                let s = swift_string(&c.value().to_string());
                if *expected == Ty::Char {
                    Ex::primary(s, Ty::Char)
                } else {
                    Ex::primary(format!("Character({s})"), Ty::Char)
                }
            }
            Lit::Bool(b) => Ex::primary(b.value.to_string(), Ty::Bool),
            Lit::Byte(b) => {
                let v = b.value();
                if v.is_ascii() && !v.is_ascii_control()
                    || v == b'\n'
                    || v == b'\t'
                    || v == b'\r'
                    || v == 0
                {
                    Ex::primary(
                        format!("UInt8(ascii: {})", swift_string(&(v as char).to_string())),
                        Ty::Int(IntKind::U8),
                    )
                } else {
                    Ex::primary(format!("UInt8({v})"), Ty::Int(IntKind::U8))
                }
            }
            Lit::ByteStr(b) => {
                let s = String::from_utf8_lossy(&b.value()).to_string();
                Ex::primary(
                    format!("Array({}.utf8)", swift_string(&s)),
                    Ty::Vec(Box::new(Ty::Int(IntKind::U8))),
                )
            }
            other => {
                let code = self.unsupported(other.span(), "this literal");
                Ex::primary(code, Ty::Unknown)
            }
        }
    }

    // ----- Paths -----

    pub fn self_code(&self) -> String {
        if let Some(n) = self.fn_cx().and_then(|f| f.self_name.clone()) {
            return n;
        }
        if let Some(r) = self.imp.as_ref().and_then(|i| i.self_rename.clone()) {
            return r;
        }
        "self".into()
    }

    fn self_ty(&self) -> Ty {
        self.imp.as_ref().map_or(Ty::Unknown, |i| i.self_ty.clone())
    }

    /// The Swift name of a type named in a path (`Self` → the impl's type).
    pub fn type_path_name(&self, name: &str) -> String {
        if name == "Self" {
            if let Some(i) = &self.imp {
                return if i.kind == TypeKind::Observable {
                    i.swift.clone()
                } else {
                    "Self".into()
                };
            }
        }
        match name {
            "String" | "str" => "String".into(),
            "Vec" => "Array".into(),
            "HashMap" | "BTreeMap" => "Dictionary".into(),
            "HashSet" | "BTreeSet" => "Set".into(),
            n => match crate::ty::from_name(n, vec![]) {
                Ty::Named(s, _) => s,
                t => t.swift_lossy(),
            },
        }
    }

    /// Resolves `Self` and aliases to the Rust name of a type.
    pub fn rust_type_name(&self, name: &str) -> String {
        if name == "Self" {
            if let Some(i) = &self.imp {
                return i.rust.clone();
            }
        }
        name.to_string()
    }

    fn path_expr(&mut self, path: &syn::Path, expected: &Ty, span: proc_macro2::Span) -> Ex {
        let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        if segs.len() == 1 {
            let n = segs[0].as_str();
            if n == "self" {
                return Ex::primary(self.self_code(), self.self_ty());
            }
            if n == "None" {
                let ty = match expected {
                    Ty::Option(_) => expected.clone(),
                    _ => Ty::Option(Box::new(Ty::Unknown)),
                };
                return Ex::primary("nil", ty);
            }
            if let Some(local) = self.lookup(n).cloned() {
                let ty = self.local_ty(&local);
                if let Some(rest) = local.swift.strip_prefix('\u{2}') {
                    return Ex::primary(rest.to_string(), ty);
                }
                return Ex::primary(local.swift, ty);
            }
            if let Some(ty) = self.syms.consts.get(n) {
                return Ex::primary(names::constant(n), ty.clone());
            }
            // `use std::f64::consts::PI;` then `PI`.
            if let Some(full) = self.syms.imports.get(n).cloned() {
                if full.len() >= 2 {
                    if let Some(ex) = self.std_constant(&full) {
                        return ex;
                    }
                }
            }
            if let Some(st) = self.syms.structs.get(n) {
                if st.fields.is_empty() {
                    return Ex::primary(
                        format!("{}()", st.swift),
                        Ty::Named(st.swift.clone(), vec![]),
                    );
                }
            }
            if let Some(sig) = self.syms.fns.get(n) {
                let ty = Ty::Fn(
                    sig.params.iter().map(|p| p.ty.clone()).collect(),
                    Box::new(sig.ret.clone()),
                );
                return Ex::primary(names::lower_camel(n), ty);
            }
            // A variant imported with `use Enum::*`.
            let found: Vec<(String, String)> = self
                .syms
                .enums
                .values()
                .filter_map(|e| e.variant(n).map(|v| (e.swift.clone(), v.swift.clone())))
                .collect();
            if found.len() == 1 {
                let (e, v) = &found[0];
                return self.variant_value(e, v, expected);
            }
            if crate::ty::is_primitive_name(n) || n == "String" {
                return Ex::primary(self.type_path_name(n), Ty::Unknown);
            }
            return Ex::primary(names::lower_camel(n), Ty::Unknown);
        }
        let last = segs[segs.len() - 1].as_str();
        let owner = segs[segs.len() - 2].as_str();
        let owner_rust = self.rust_type_name(owner);
        if let Some(ex) = self.std_constant(&segs) {
            return ex;
        }
        // `use std::f64::consts;` then `consts::E`.
        if let Some(full) = self.syms.imports.get(&segs[0]) {
            let mut full = full.clone();
            full.extend(segs[1..].iter().cloned());
            if let Some(ex) = self.std_constant(&full) {
                return ex;
            }
        }
        if let Some(en) = self.syms.enums.get(&owner_rust).cloned() {
            if let Some(v) = en.variant(last) {
                if matches!(v.shape, VariantShape::Unit) {
                    return self.variant_value(&en.swift, &v.swift, expected);
                }
                let ty = Ty::Named(en.swift.clone(), vec![]);
                return Ex::primary(format!("{}.{}", self.type_path_name(owner), v.swift), ty);
            }
        }
        if let Some(methods) = self.syms.methods.get(&owner_rust) {
            if let Some(sig) = methods.get(last) {
                let tname = self.type_path_name(owner);
                if sig.is_init {
                    return Ex::primary(format!("{tname}.init"), Ty::Unknown);
                }
                return Ex::primary(format!("{tname}.{}", names::lower_camel(last)), Ty::Unknown);
            }
        }
        if self.syms.is_user_type(&owner_rust) || owner == "Self" {
            // An associated constant.
            return Ex::primary(
                format!("{}.{}", self.type_path_name(owner), names::constant(last)),
                Ty::Unknown,
            );
        }
        if let Some(ex) = self.ui_constant(owner, last, expected) {
            return ex;
        }
        if owner == "Ordering" {
            self.helper("Ordering");
            return Ex::primary(
                format!("Ordering.{}", names::case_name(last)),
                Ty::Named("Ordering".into(), vec![]),
            );
        }
        // `module::item`: modules disappear.
        let single = syn::Path::from(syn::Ident::new(&segs[segs.len() - 1], span));
        self.path_expr(&single, expected, span)
    }

    /// A unit enum case: `.red` when the context knows the type, `Color.red` otherwise.
    fn variant_value(&self, enum_swift: &str, case: &str, expected: &Ty) -> Ex {
        let ty = Ty::Named(enum_swift.to_string(), vec![]);
        if expected.named() == Some(enum_swift) {
            Ex::primary(format!(".{case}"), ty)
        } else {
            Ex::primary(format!("{enum_swift}.{case}"), ty)
        }
    }

    /// `i32::MAX`, `f64::consts::PI`, `f64::INFINITY`…
    fn std_constant(&mut self, segs: &[String]) -> Option<Ex> {
        let last = segs.last()?.as_str();
        let owner = segs[segs.len() - 2].as_str();
        let owner = if owner == "consts" && segs.len() >= 3 {
            segs[segs.len() - 3].as_str()
        } else {
            owner
        };
        if let Some(k) = IntKind::parse(owner) {
            let t = k.swift();
            // `i32` is Swift's `Int`, but keeps the limits of 32 bits.
            let exact = match k {
                IntKind::I32 => "Int32",
                _ => t,
            };
            let code = match last {
                "MAX" if exact != t => format!("Int({exact}.max)"),
                "MIN" if exact != t => format!("Int({exact}.min)"),
                "MAX" => format!("{t}.max"),
                "MIN" => format!("{t}.min"),
                "BITS" => format!("{t}.bitWidth"),
                _ => return None,
            };
            return Some(Ex::primary(code, Ty::Int(k)));
        }
        if owner == "f64" || owner == "f32" {
            let (t, ty) = if owner == "f64" {
                ("Double", Ty::Float(FloatKind::F64))
            } else {
                ("Float", Ty::Float(FloatKind::F32))
            };
            let member = match last {
                "PI" => "pi",
                "INFINITY" => "infinity",
                "NEG_INFINITY" => return Some(Ex::new(format!("-{t}.infinity"), P_PREFIX, ty)),
                "NAN" => "nan",
                "EPSILON" => "ulpOfOne",
                "MAX" => "greatestFiniteMagnitude",
                "MIN" => {
                    return Some(Ex::new(
                        format!("-{t}.greatestFiniteMagnitude"),
                        P_PREFIX,
                        ty,
                    ));
                }
                "MIN_POSITIVE" => "leastNormalMagnitude",
                "E" => return Some(Ex::primary(format!("{t}(M_E)"), ty)),
                "TAU" => return Some(Ex::new(format!("2 * {t}.pi"), P_MUL, ty)),
                "SQRT_2" => return Some(Ex::primary(format!("{t}(2).squareRoot()"), ty)),
                "FRAC_PI_2" => return Some(Ex::new(format!("{t}.pi / 2"), P_MUL, ty)),
                "LN_2" => return Some(Ex::primary(format!("{t}(M_LN2)"), ty)),
                _ => return None,
            };
            if self.implicit_member && member == "infinity" {
                return Some(Ex::primary(".infinity", ty));
            }
            return Some(Ex::primary(format!("{t}.{member}"), ty));
        }
        if owner == "char" && last == "MAX" {
            return Some(Ex::primary("Character(\"\\u{10FFFF}\")", Ty::Char));
        }
        None
    }

    // ----- Operators -----

    fn binary(&mut self, b: &syn::ExprBinary, expected: &Ty) -> Ex {
        let (op, prec, assign) = match b.op {
            BinOp::Add(_) => ("+", P_ADD, false),
            BinOp::Sub(_) => ("-", P_ADD, false),
            BinOp::Mul(_) => ("*", P_MUL, false),
            BinOp::Div(_) => ("/", P_MUL, false),
            BinOp::Rem(_) => ("%", P_MUL, false),
            BinOp::And(_) => ("&&", P_AND, false),
            BinOp::Or(_) => ("||", P_OR, false),
            BinOp::BitXor(_) => ("^", P_ADD, false),
            BinOp::BitAnd(_) => ("&", P_MUL, false),
            BinOp::BitOr(_) => ("|", P_ADD, false),
            BinOp::Shl(_) => ("<<", P_SHIFT, false),
            BinOp::Shr(_) => (">>", P_SHIFT, false),
            BinOp::Eq(_) => ("==", P_CMP, false),
            BinOp::Lt(_) => ("<", P_CMP, false),
            BinOp::Le(_) => ("<=", P_CMP, false),
            BinOp::Ne(_) => ("!=", P_CMP, false),
            BinOp::Ge(_) => (">=", P_CMP, false),
            BinOp::Gt(_) => (">", P_CMP, false),
            BinOp::AddAssign(_) => ("+=", P_ASSIGN, true),
            BinOp::SubAssign(_) => ("-=", P_ASSIGN, true),
            BinOp::MulAssign(_) => ("*=", P_ASSIGN, true),
            BinOp::DivAssign(_) => ("/=", P_ASSIGN, true),
            BinOp::RemAssign(_) => ("%=", P_ASSIGN, true),
            BinOp::BitXorAssign(_) => ("^=", P_ASSIGN, true),
            BinOp::BitAndAssign(_) => ("&=", P_ASSIGN, true),
            BinOp::BitOrAssign(_) => ("|=", P_ASSIGN, true),
            BinOp::ShlAssign(_) => ("<<=", P_ASSIGN, true),
            BinOp::ShrAssign(_) => (">>=", P_ASSIGN, true),
            _ => {
                let code = self.unsupported(b.span(), "this operator");
                return Ex::primary(code, Ty::Unknown);
            }
        };
        if assign {
            let lhs_ty = self.place_ty(&b.left);
            let lhs = self.place(&b.left);
            let rhs = self.expr_with(&b.right, &lhs_ty);
            let rhs = self.settle(rhs);
            if op == "%=" && lhs_ty.is_float() {
                return Ex::new(
                    format!(
                        "{lhs} = {lhs}.truncatingRemainder(dividingBy: {})",
                        rhs.code
                    ),
                    P_ASSIGN,
                    Ty::Unit,
                );
            }
            if op == "+=" && lhs_ty == Ty::Str && rhs.ty == Ty::Char {
                return Ex::new(format!("{lhs}.append({})", rhs.code), P_PRIMARY, Ty::Unit);
            }
            return Ex::new(format!("{lhs} {op} {}", rhs.code), P_ASSIGN, Ty::Unit);
        }
        let comparison = prec == P_CMP;
        let left_expected = if comparison || matches!(op, "&&" | "||") {
            if matches!(op, "&&" | "||") {
                Ty::Bool
            } else {
                Ty::Unknown
            }
        } else {
            expected.clone()
        };
        let left = self.expr_with(&b.left, &left_expected);
        let left = self.settle(left);
        let right_expected = if matches!(op, "&&" | "||") {
            Ty::Bool
        } else if matches!(op, "<<" | ">>") {
            Ty::Unknown
        } else {
            left.ty.clone()
        };
        let right = self.expr_with(&b.right, &right_expected);
        let right_tries = right.throws.is_some() || right.code.contains("try ");
        let right = self.settle(right);
        // When the left side's type was unknown, the right side may tell it.
        let mut operand_ty = left.ty.clone();
        operand_ty.merge(&right.ty);
        let result_ty = if comparison || matches!(op, "&&" | "||") {
            Ty::Bool
        } else {
            operand_ty.clone()
        };
        // `a && try f()`: Swift's autoclosures cannot throw a typed error.
        if right_tries && matches!(op, "&&" | "||") {
            let l = wrap(&left, P_TERNARY + 1);
            let code = if op == "&&" {
                format!("{l} ? {} : false", right.code)
            } else {
                format!("{l} ? true : {}", right.code)
            };
            return Ex::new(code, P_TERNARY, Ty::Bool);
        }
        if op == "%" && operand_ty.is_float() {
            return Ex::primary(
                format!(
                    "{}.truncatingRemainder(dividingBy: {})",
                    wrap(&left, P_PRIMARY),
                    right.code
                ),
                operand_ty,
            );
        }
        let op = match (op, &operand_ty) {
            ("&", Ty::Bool) => "&&",
            ("|", Ty::Bool) => "||",
            ("^", Ty::Bool) => "!=",
            (op, _) => op,
        };
        let (lmin, rmin) = if comparison {
            (P_CMP + 1, P_CMP + 1)
        } else {
            (prec, prec + 1)
        };
        let mut ex = Ex::new(
            format!("{} {op} {}", wrap(&left, lmin), wrap(&right, rmin)),
            prec,
            result_ty,
        );
        ex.weak = left.weak && right.weak && !comparison;
        ex
    }

    fn unary(&mut self, u: &syn::ExprUnary, expected: &Ty) -> Ex {
        match u.op {
            UnOp::Deref(_) => self.expr_with(&u.expr, expected),
            UnOp::Neg(_) => {
                let v = self.expr_with(&u.expr, expected);
                let v = self.settle(v);
                let weak = v.weak;
                let mut ex = Ex::new(format!("-{}", wrap(&v, P_PREFIX)), P_PREFIX, v.ty);
                ex.weak = weak;
                ex
            }
            UnOp::Not(_) => {
                let v = self.expr_with(&u.expr, expected);
                let v = self.settle(v);
                let op = if v.ty.is_int() { "~" } else { "!" };
                Ex::new(format!("{op}{}", wrap(&v, P_PREFIX)), P_PREFIX, v.ty)
            }
            _ => {
                let code = self.unsupported(u.span(), "this operator");
                Ex::primary(code, Ty::Unknown)
            }
        }
    }

    fn cast(&mut self, c: &syn::ExprCast) -> Ex {
        let target = self.ty(&c.ty);
        let v = self.expr(&c.expr);
        let v = self.settle(v);
        let Some(t) = target.swift() else {
            return v;
        };
        // Same Swift type: nothing to convert (`i32` and `usize` are both `Int`).
        if v.ty.swift().as_deref() == Some(t.as_str()) {
            return v;
        }
        match (&v.ty, &target) {
            (Ty::Bool, Ty::Int(_)) => Ex::new(format!("({} ? 1 : 0)", v.code), P_PRIMARY, target),
            (Ty::Char, Ty::Int(_)) => {
                self.helper("char");
                if t == "UInt32" {
                    Ex::primary(format!("{}.rsCode", wrap(&v, P_PRIMARY)), target)
                } else {
                    Ex::primary(
                        format!("{t}(truncatingIfNeeded: {}.rsCode)", wrap(&v, P_PRIMARY)),
                        target,
                    )
                }
            }
            (Ty::Int(IntKind::U8), Ty::Char) => {
                Ex::primary(format!("Character(UnicodeScalar({}))", v.code), Ty::Char)
            }
            (Ty::Int(_), Ty::Char) => {
                self.helper("char");
                Ex::primary(format!("Character(rsScalar: {})", v.code), Ty::Char)
            }
            (Ty::Named(n, _), Ty::Int(_))
                if self.syms.enums.contains_key(n)
                    || self.syms.enums.values().any(|e| &e.swift == n) =>
            {
                let raw = format!("{}.rawValue", wrap(&v, P_PRIMARY));
                if t == "Int" {
                    Ex::primary(raw, target)
                } else {
                    Ex::primary(format!("{t}({raw})"), target)
                }
            }
            // Narrowing, or a negative value made unsigned: Rust's `as` truncates.
            (Ty::Int(from), Ty::Int(to))
                if to.bits() < from.bits()
                    || from.signed() && !to.signed()
                    || to.bits() == from.bits() && from.signed() != to.signed() =>
            {
                Ex::primary(format!("{t}(truncatingIfNeeded: {})", v.code), target)
            }
            _ => Ex::primary(format!("{t}({})", v.code), target),
        }
    }

    fn range(&mut self, r: &syn::ExprRange, expected: &Ty) -> Ex {
        let item = match expected {
            Ty::Range(t) => (**t).clone(),
            _ => Ty::Unknown,
        };
        let start = r.start.as_ref().map(|s| {
            let v = self.expr_with(s, &item);
            self.settle(v)
        });
        let start_weak = start.as_ref().is_some_and(|s| s.weak);
        let mut item_ty = start.as_ref().map_or(Ty::Unknown, |s| s.ty.clone());
        let end_expected = if start_weak {
            item.clone()
        } else {
            item_ty.clone()
        };
        let end = r.end.as_ref().map(|e| {
            let v = self.expr_with(e, &end_expected);
            self.settle(v)
        });
        if let Some(e) = &end {
            if start_weak && !e.weak {
                item_ty = e.ty.clone();
            } else {
                item_ty.merge(&e.ty);
            }
        }
        let op = match r.limits {
            syn::RangeLimits::HalfOpen(_) => "..<",
            syn::RangeLimits::Closed(_) => "...",
        };
        let code = match (&start, &end) {
            (Some(s), Some(e)) if e.code.starts_with('-') => {
                format!("{} {op} {}", wrap(s, P_RANGE + 1), wrap(e, P_RANGE + 1))
            }
            (Some(s), Some(e)) => format!("{}{op}{}", wrap(s, P_RANGE + 1), wrap(e, P_RANGE + 1)),
            (Some(s), None) => format!("{}...", wrap(s, P_RANGE + 1)),
            (None, Some(e)) => format!("{op}{}", wrap(e, P_RANGE + 1)),
            (None, None) => "...".into(),
        };
        Ex::new(code, P_RANGE, Ty::Range(Box::new(item_ty)))
    }

    // ----- ? -----

    fn try_expr(&mut self, t: &syn::ExprTry, expected: &Ty) -> Ex {
        let inner_expected = if expected.is_unknown() {
            Ty::Unknown
        } else {
            Ty::Option(Box::new(expected.clone()))
        };
        // The value may end up in a `guard let … else`: no trailing closure.
        self.no_trailing += 1;
        let v = self.expr_with(&t.expr, &inner_expected);
        self.no_trailing -= 1;
        if v.ty == Ty::Unit {
            // `write!(f, …)?`
            return v;
        }
        if let Some(th) = &v.throws {
            let ok = th.0.clone();
            return Ex::new(format!("try {}", v.code), P_TRY, ok);
        }
        if v.ty.is_result() && !v.parsed {
            let ok = match &v.ty {
                Ty::Result(o, _) => (**o).clone(),
                _ => Ty::Unknown,
            };
            return Ex::new(format!("try {}.get()", wrap(&v, P_PRIMARY)), P_TRY, ok);
        }
        // `?` on an Option (or a parsed value): leave the function early.
        let item = v.ty.item();
        let tmp = self.fresh("value");
        let throws = self.fn_cx().and_then(|f| f.throws.clone());
        let exit = match (&throws, v.parsed) {
            (Some(_), _) => {
                self.helper("ParseError");
                "throw RsParseError()".to_string()
            }
            (None, _) => {
                let cx = self.fn_cx();
                let unit = cx.is_some_and(|f| f.ret == Ty::Unit);
                if unit {
                    "return".into()
                } else {
                    "return nil".into()
                }
            }
        };
        self.prelude(format!("guard let {tmp} = {} else {{ {exit} }}", v.code));
        Ex::primary(tmp, item)
    }

    // ----- Places (assignment targets) -----

    /// The type of an assignment target.
    pub fn place_ty(&mut self, e: &Expr) -> Ty {
        match e {
            Expr::Path(p) => p
                .path
                .get_ident()
                .and_then(|i| self.lookup(&i.to_string()).cloned())
                .map(|l| self.local_ty(&l))
                .unwrap_or(Ty::Unknown),
            Expr::Unary(u) => self.place_ty(&u.expr),
            Expr::Field(f) => {
                let base = self.place_ty(&f.base);
                let base = if base.is_unknown() && is_self(&f.base) {
                    self.self_ty()
                } else {
                    base
                };
                self.field_ty(&base, &f.member)
            }
            Expr::Index(i) => {
                let base = self.place_ty(&i.expr);
                match base {
                    Ty::Map(_, v, _) => *v,
                    t => t.item(),
                }
            }
            _ => Ty::Unknown,
        }
    }

    /// An assignment target: `*x` is `x`, `*map.entry(k).or_insert(0)` is `map[k, default: 0]`.
    pub fn place(&mut self, e: &Expr) -> String {
        match e {
            Expr::Unary(u) if matches!(u.op, UnOp::Deref(_)) => self.place(&u.expr),
            Expr::Paren(p) => self.place(&p.expr),
            Expr::MethodCall(m) => {
                if let Some(code) = self.entry_place(m) {
                    return code;
                }
                let v = self.expr(e);
                v.code
            }
            _ => {
                let v = self.expr(e);
                v.code
            }
        }
    }

    /// `map.entry(k).or_insert(v)` → `map[k, default: v]`.
    pub fn entry_place(&mut self, m: &syn::ExprMethodCall) -> Option<String> {
        let method = m.method.to_string();
        if !matches!(
            method.as_str(),
            "or_insert" | "or_insert_with" | "or_default"
        ) {
            return None;
        }
        let Expr::MethodCall(entry) = &*m.receiver else {
            return None;
        };
        if entry.method != "entry" || entry.args.len() != 1 {
            return None;
        }
        let map = self.expr(&entry.receiver);
        let (key_ty, value_ty) = match &map.ty {
            Ty::Map(k, v, _) => ((**k).clone(), (**v).clone()),
            _ => (Ty::Unknown, Ty::Unknown),
        };
        let key = self.expr_with(&entry.args[0], &key_ty);
        let key = self.settle(key);
        let default = match method.as_str() {
            "or_insert" => {
                let v = self.expr_with(&m.args[0], &value_ty);
                let v = self.settle(v);
                if let Expr::Path(p) = &*entry.receiver {
                    if let Some(id) = p.path.get_ident() {
                        self.refine(
                            &id.to_string(),
                            &Ty::Map(Box::new(key.ty.clone()), Box::new(v.ty.clone()), false),
                        );
                    }
                }
                v.code
            }
            "or_insert_with" => {
                let f = &m.args[0];
                match f {
                    Expr::Closure(c) => {
                        let v = self.expr_with(&c.body, &value_ty);
                        self.settle(v).code
                    }
                    Expr::Path(p) => {
                        let s = quote::quote!(#p).to_string().replace(' ', "");
                        match s.as_str() {
                            "Vec::new" | "HashSet::new" | "BTreeSet::new" => "[]".into(),
                            "HashMap::new" | "BTreeMap::new" => "[:]".into(),
                            "String::new" => "\"\"".into(),
                            _ => {
                                let v = self.expr(f);
                                format!("{}()", v.code)
                            }
                        }
                    }
                    other => {
                        let v = self.expr(other);
                        format!("{}()", v.code)
                    }
                }
            }
            _ => value_ty.default_value().unwrap_or_else(|| {
                self.error(
                    m.span(),
                    "or_default(): the value type of this map is unknown",
                );
                "fatalError()".into()
            }),
        };
        Some(format!(
            "{}[{}, default: {default}]",
            wrap(&map, P_PRIMARY),
            key.code
        ))
    }

    // ----- Fields and indexes -----

    pub fn field_ty(&self, base: &Ty, member: &syn::Member) -> Ty {
        match (base, member) {
            (Ty::Tuple(ts), syn::Member::Unnamed(i)) => {
                ts.get(i.index as usize).cloned().unwrap_or(Ty::Unknown)
            }
            (Ty::Named(n, _), m) => {
                let rust = match m {
                    syn::Member::Named(id) => id.to_string(),
                    syn::Member::Unnamed(i) => i.index.to_string(),
                };
                self.struct_by_swift(n)
                    .and_then(|s| s.field(&rust).map(|f| f.ty.clone()))
                    .unwrap_or(Ty::Unknown)
            }
            _ => Ty::Unknown,
        }
    }

    pub fn struct_by_swift(&self, swift: &str) -> Option<&crate::symbols::StructInfo> {
        self.syms
            .structs
            .get(swift)
            .or_else(|| self.syms.structs.values().find(|s| s.swift == swift))
    }

    fn field(&mut self, f: &syn::ExprField) -> Ex {
        let base_is_self = is_self(&f.base);
        let base = self.expr(&f.base);
        let base = self.settle(base);
        let base_ty = if base.ty.is_unknown() && base_is_self {
            self.self_ty()
        } else {
            base.ty.clone()
        };
        let ty = self.field_ty(&base_ty, &f.member);
        let swift_member = match (&f.member, &base_ty) {
            (syn::Member::Named(id), Ty::Named(n, _)) => self
                .struct_by_swift(n)
                .and_then(|s| s.field(&id.to_string()).map(|f| f.swift.clone()))
                .unwrap_or_else(|| names::lower_camel(&id.to_string())),
            (syn::Member::Named(id), _) => names::lower_camel(&id.to_string()),
            (syn::Member::Unnamed(i), Ty::Named(n, _))
                if self.struct_by_swift(n).is_some_and(|s| s.tuple) =>
            {
                format!("_{}", i.index)
            }
            (syn::Member::Unnamed(i), _) => i.index.to_string(),
        };
        // `self.count` → `count`, unless a local of that name hides the field.
        if base_is_self && base.code == "self" {
            let in_class = self
                .imp
                .as_ref()
                .is_some_and(|i| i.kind == TypeKind::Observable);
            if !in_class && !self.swift_local_visible(&swift_member) {
                return Ex::primary(swift_member, ty);
            }
        }
        Ex::primary(format!("{}.{swift_member}", wrap(&base, P_PRIMARY)), ty)
    }

    fn index(&mut self, i: &syn::ExprIndex) -> Ex {
        let base = self.expr(&i.expr);
        let base = self.settle(base);
        if let Expr::Range(r) = &*i.index {
            let range = self.range(r, &Ty::Range(Box::new(Ty::Int(IntKind::Usize))));
            if base.ty == Ty::Str {
                self.helper("string");
                let a = r
                    .start
                    .as_ref()
                    .map(|s| {
                        let v = self.expr(s);
                        self.settle(v).code
                    })
                    .unwrap_or_else(|| "0".into());
                let b = r.end.as_ref().map(|e| {
                    let v = self.expr(e);
                    self.settle(v).code
                });
                let closed = matches!(r.limits, syn::RangeLimits::Closed(_));
                let end = match (b, closed) {
                    (Some(b), false) => b,
                    (Some(b), true) => format!("{b} + 1"),
                    (None, _) => format!("{}.utf8.count", wrap(&base, P_PRIMARY)),
                };
                return Ex::primary(
                    format!("{}.rsSlice({a}, {end})", wrap(&base, P_PRIMARY)),
                    Ty::Str,
                );
            }
            if r.start.is_none() && r.end.is_none() {
                return base;
            }
            return Ex::primary(
                format!("Array({}[{}])", wrap(&base, P_PRIMARY), range.code),
                base.ty,
            );
        }
        let key_ty = match &base.ty {
            Ty::Map(k, _, _) => (**k).clone(),
            _ => Ty::Int(IntKind::Usize),
        };
        let idx = self.expr_with(&i.index, &key_ty);
        let idx = self.settle(idx);
        match &base.ty {
            Ty::Map(_, v, _) => Ex::primary(
                format!("{}[{}]!", wrap(&base, P_PRIMARY), idx.code),
                (**v).clone(),
            ),
            t => Ex::primary(
                format!("{}[{}]", wrap(&base, P_PRIMARY), idx.code),
                t.item(),
            ),
        }
    }

    // ----- Struct literals -----

    fn struct_lit(&mut self, s: &syn::ExprStruct, expected: &Ty) -> Ex {
        let segs: Vec<String> = s
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let last = segs.last().cloned().unwrap_or_default();
        // An enum variant with named fields: `Shape::Rect { w, h }`.
        if segs.len() >= 2 {
            let owner = self.rust_type_name(&segs[segs.len() - 2]);
            if let Some(en) = self.syms.enums.get(&owner).cloned() {
                if let Some(v) = en.variant(&last) {
                    let VariantShape::Struct(fields) = &v.shape else {
                        let code = self.unsupported(s.span(), "this enum literal");
                        return Ex::primary(code, Ty::Unknown);
                    };
                    let mut args = Vec::new();
                    for (fname, fty) in fields {
                        let value = s.fields.iter().find(|f| member_name(&f.member) == *fname);
                        let code = match value {
                            Some(fv) => {
                                let v = self.expr_with(&fv.expr, fty);
                                self.settle(v).code
                            }
                            None => "fatalError()".into(),
                        };
                        args.push(format!("{}: {code}", names::lower_camel(fname)));
                    }
                    let ty = Ty::Named(en.swift.clone(), vec![]);
                    let prefix = if expected.named() == Some(en.swift.as_str()) {
                        String::new()
                    } else {
                        self.type_path_name(&segs[segs.len() - 2])
                    };
                    return Ex::primary(format!("{prefix}.{}({})", v.swift, args.join(", ")), ty);
                }
            }
        }
        let rust_name = self.rust_type_name(&last);
        let Some(info) = self.syms.structs.get(&rust_name).cloned() else {
            let code = self.unsupported(s.span(), &format!("the unknown struct `{last}`"));
            return Ex::primary(code, Ty::Unknown);
        };
        let type_code = match expected {
            // `Pair<B, A>(…)`: inside `Pair`'s own methods, a bare `Pair` would mean `Pair<A, B>`.
            Ty::Named(n, args)
                if *n == info.swift && !args.is_empty() && expected.is_complete() =>
            {
                expected.swift_lossy()
            }
            _ if last == "Self" && info.kind != TypeKind::Observable => "Self".to_string(),
            _ => info.swift.clone(),
        };
        let ty = match expected {
            Ty::Named(n, _) if *n == info.swift => expected.clone(),
            _ => Ty::Named(info.swift.clone(), vec![]),
        };
        let mut args = Vec::new();
        for field in &info.fields {
            // `@Environment` values are provided by SwiftUI, not by the caller.
            if matches!(field.role, FieldRole::Environment(_)) {
                continue;
            }
            let Some(fv) = s
                .fields
                .iter()
                .find(|f| member_name(&f.member) == field.rust)
            else {
                continue;
            };
            let code = if field.role == FieldRole::Binding {
                self.binding_arg(&fv.expr)
            } else {
                let v = self.expr_with(&fv.expr, &field.ty);
                self.settle(v).code
            };
            if info.tuple {
                args.push(code);
            } else {
                args.push(format!("{}: {code}", field.swift));
            }
        }
        if let Some(rest) = &s.rest {
            // `Point { x: 1, ..base }`: a modified copy.
            let is_default = matches!(&**rest, Expr::Call(c) if quote::quote!(#c).to_string().replace(' ', "").ends_with("default()"));
            let base = if is_default {
                format!("{type_code}()")
            } else {
                let v = self.expr_with(rest, &ty);
                self.settle(v).code
            };
            let tmp = "copy";
            let mut body = vec![format!("var {tmp} = {base}")];
            for f in &s.fields {
                let name = member_name(&f.member);
                let field = info.field(&name);
                if field.is_some_and(|f| matches!(f.role, FieldRole::Environment(_))) {
                    continue;
                }
                let swift = field.map_or_else(|| names::lower_camel(&name), |f| f.swift.clone());
                let fty = field.map_or(Ty::Unknown, |f| f.ty.clone());
                let v = self.expr_with(&f.expr, &fty);
                let v = self.settle(v);
                body.push(format!("{tmp}.{swift} = {}", v.code));
            }
            body.push(format!("return {tmp}"));
            return Ex::primary(format!("{{\n{}\n}}()", indent_lines(&body.join("\n"))), ty);
        }
        Ex::primary(format!("{type_code}({})", args.join(", ")), ty)
    }

    /// A value given to a `#[binding]` field: `self.flag` → `$flag`.
    pub fn binding_arg(&mut self, e: &Expr) -> String {
        let inner = match e {
            Expr::Reference(r) => &*r.expr,
            Expr::MethodCall(m) if m.method == "clone" && m.args.is_empty() => &*m.receiver,
            e => e,
        };
        if let Expr::Field(f) = inner {
            if is_self(&f.base) {
                if let syn::Member::Named(id) = &f.member {
                    let swift = self
                        .imp
                        .as_ref()
                        .and_then(|i| self.syms.structs.get(&i.rust))
                        .and_then(|s| s.field(&id.to_string()).map(|f| f.swift.clone()))
                        .unwrap_or_else(|| names::lower_camel(&id.to_string()));
                    return format!("${swift}");
                }
            }
            // `&mut item.done` inside `for_each(&mut self.items, …)`.
            let base = self.expr(&f.base);
            if self.binding_items.contains(&base.code) {
                let member = match &f.member {
                    syn::Member::Named(id) => {
                        let rust = id.to_string();
                        match &base.ty {
                            Ty::Named(n, _) => self
                                .struct_by_swift(n)
                                .and_then(|s| s.field(&rust).map(|f| f.swift.clone()))
                                .unwrap_or_else(|| names::lower_camel(&rust)),
                            _ => names::lower_camel(&rust),
                        }
                    }
                    syn::Member::Unnamed(i) => i.index.to_string(),
                };
                return format!("${}.{member}", base.code);
            }
            let whole = self.expr(inner);
            if let Some(rest) = whole.code.strip_prefix("self.") {
                return format!("${rest}");
            }
            if !whole.code.contains('(') {
                return format!("${}", whole.code);
            }
        }
        if let Expr::Path(p) = inner {
            if let Some(id) = p.path.get_ident() {
                if let Some(l) = self.lookup(&id.to_string()) {
                    if self.binding_items.contains(&l.swift) {
                        return format!("${}", l.swift);
                    }
                }
            }
        }
        let v = self.expr(inner);
        if v.code.starts_with('$') || v.code.starts_with(".constant(") {
            return v.code;
        }
        format!(".constant({})", v.code)
    }

    // ----- Closures -----

    pub fn closure(&mut self, c: &syn::ExprClosure, expected: &Ty) -> Ex {
        let (param_tys, ret_ty) = match expected {
            Ty::Fn(p, r) => (p.clone(), (**r).clone()),
            _ => (vec![], Ty::Unknown),
        };
        self.push_scope();
        let mut params = Vec::new();
        let mut typed = false;
        let mut tys = Vec::new();
        let mut prologue = Vec::new();
        for (i, p) in c.inputs.iter().enumerate() {
            let hint = param_tys.get(i).cloned().unwrap_or(Ty::Unknown);
            let (pat, annot) = match p {
                Pat::Type(pt) => (&*pt.pat, Some(self.ty(&pt.ty))),
                p => (p, None),
            };
            let ty = annot.clone().unwrap_or(hint);
            tys.push(ty.clone());
            let name = self.closure_param(pat, &ty, &mut prologue, i);
            match annot {
                Some(t) => {
                    typed = true;
                    params.push(format!("{name}: {}", t.swift_lossy()));
                }
                None => params.push(name),
            }
        }
        let declared_ret = match &c.output {
            syn::ReturnType::Type(_, t) => Some(self.ty(t)),
            syn::ReturnType::Default => None,
        };
        let mut ret = declared_ret.clone().unwrap_or(ret_ty);
        let returns_result = closure_returns_result(&c.body) || ret.is_result();
        let throws = if returns_result {
            Some(match &ret {
                Ty::Result(_, e) => (**e).clone(),
                _ => self
                    .fn_cx()
                    .and_then(|f| f.throws.clone())
                    .unwrap_or(Ty::Unknown),
            })
        } else {
            None
        };
        if returns_result && !ret.is_result() {
            ret = Ty::Result(
                Box::new(Ty::Unknown),
                Box::new(throws.clone().unwrap_or(Ty::Unknown)),
            );
        }
        self.fns.push(FnCx {
            ret: ret.clone(),
            throws: throws.clone(),
            init: false,
            closure: true,
            self_name: None,
        });
        let body_expr = &*c.body;
        let mut body_ty = Ty::Unknown;
        let single: Option<String> = match body_expr {
            Expr::Block(b) if b.block.stmts.len() == 1 && throws.is_none() => {
                match &b.block.stmts[0] {
                    syn::Stmt::Expr(e, None) if !needs_statements(e) && !has_control_flow(e) => {
                        let v = self.closure_value(e, &ret);
                        body_ty = v.ty.clone();
                        Some(v.code)
                    }
                    _ => None,
                }
            }
            Expr::Block(_) => None,
            e if !needs_statements(e) && !has_control_flow(e) && throws.is_none() => {
                let v = self.closure_value(e, &ret);
                body_ty = v.ty.clone();
                Some(v.code)
            }
            _ => None,
        };
        let lines = match single {
            Some(code) => {
                let mut l = prologue.clone();
                if !l.is_empty() {
                    l.push(format!("return {code}"));
                    l
                } else {
                    vec![code]
                }
            }
            None => {
                let sink = if ret == Ty::Unit {
                    Sink::Discard
                } else {
                    Sink::Return
                };
                let mut l = prologue.clone();
                match body_expr {
                    Expr::Block(b) => l.extend(self.block_lines(&b.block, sink)),
                    e => {
                        let lines = self.with_prelude(|t| t.tail(e, &sink));
                        l.extend(lines);
                    }
                }
                l
            }
        };
        self.fns.pop();
        self.pop_scope();
        let throws_clause = match &throws {
            Some(Ty::Unknown) | Some(Ty::Dyn(_)) => " throws".to_string(),
            Some(e) => format!(" throws({})", e.swift_lossy()),
            None => String::new(),
        };
        let ret_clause = match &declared_ret {
            Some(Ty::Result(ok, _)) if throws.is_some() => format!(" -> {}", ok.swift_lossy()),
            Some(t) if *t != Ty::Unit && throws.is_none() => format!(" -> {}", t.swift_lossy()),
            _ => String::new(),
        };
        let head = if params.is_empty() {
            if throws_clause.is_empty() && ret_clause.is_empty() {
                String::new()
            } else {
                format!("(){throws_clause}{ret_clause} in")
            }
        } else if typed || !ret_clause.is_empty() {
            format!("({}){throws_clause}{ret_clause} in", params.join(", "))
        } else {
            format!("{}{throws_clause} in", params.join(", "))
        };
        let fn_ret = match (&declared_ret, &body_ty) {
            (Some(t), _) => t.clone(),
            (None, t) => t.clone(),
        };
        let ty = Ty::Fn(tys, Box::new(fn_ret));
        let code = if lines.len() == 1 && !lines[0].contains('\n') && lines[0].len() < 70 {
            if head.is_empty() {
                format!("{{ {} }}", lines[0])
            } else {
                format!("{{ {head} {} }}", lines[0])
            }
        } else if head.is_empty() {
            format!("{{\n{}\n}}", indent_lines(&lines.join("\n")))
        } else {
            format!("{{ {head}\n{}\n}}", indent_lines(&lines.join("\n")))
        };
        let mut ex = Ex::primary(code, ty);
        if throws.is_some() {
            ex.parsed = false;
        }
        ex
    }

    fn closure_value(&mut self, e: &Expr, ret: &Ty) -> Ex {
        let v = self.expr_with(e, ret);
        let v = self.settle(v);
        let prelude_empty = self.preludes.last().is_none_or(|p| p.is_empty());
        let _ = prelude_empty;
        v
    }

    /// A closure parameter: a name, `(a, b)`, or `_`.
    fn closure_param(
        &mut self,
        pat: &Pat,
        ty: &Ty,
        prologue: &mut Vec<String>,
        index: usize,
    ) -> String {
        match pat {
            Pat::Ident(id) => {
                let n = self.declare(&id.ident.to_string(), ty.clone());
                if id.mutability.is_some() {
                    prologue.push(format!("var {n} = {n}"));
                }
                n
            }
            Pat::Wild(_) => "_".into(),
            Pat::Reference(r) => self.closure_param(&r.pat, ty, prologue, index),
            Pat::Tuple(t) => {
                let tys = match ty {
                    Ty::Tuple(ts) => ts.clone(),
                    _ => vec![Ty::Unknown; t.elems.len()],
                };
                let parts: Vec<String> = t
                    .elems
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        self.closure_param(p, tys.get(i).unwrap_or(&Ty::Unknown), prologue, i)
                    })
                    .collect();
                format!("({})", parts.join(", "))
            }
            other => {
                let n = format!("arg{index}");
                let (lines, ok) = self.let_pattern(other, &n, ty, false);
                if !ok {
                    self.error(
                        other.span(),
                        "this closure parameter pattern is not supported",
                    );
                }
                prologue.extend(lines);
                n
            }
        }
    }

    /// An `if`, `match` or block used as a value inside an expression.
    fn statements_as_value(&mut self, e: &Expr, expected: &Ty) -> Ex {
        // `if c { a } else { b }` → `c ? a : b`.
        if let Expr::If(i) = e {
            if let Some(ex) = self.ternary(i, expected) {
                return ex;
            }
        }
        if has_control_flow(e) {
            self.error(e.span(), "control flow inside an expression used as a value is not supported: bind it with `let x: T = …;` first");
        }
        let ret = expected.clone();
        self.fns.push(FnCx {
            ret: if ret.is_unknown() {
                Ty::Unknown
            } else {
                ret.clone()
            },
            throws: None,
            init: false,
            closure: true,
            self_name: None,
        });
        let lines = self.with_prelude(|t| t.tail(e, &Sink::Return));
        self.fns.pop();
        let head = match ret.swift() {
            Some(t) if ret != Ty::Unit => format!(" () -> {t} in"),
            _ => String::new(),
        };
        Ex::primary(
            format!("{{{head}\n{}\n}}()", indent_lines(&lines.join("\n"))),
            ret,
        )
    }

    pub fn ternary(&mut self, i: &syn::ExprIf, expected: &Ty) -> Option<Ex> {
        if crate::stmt::contains_let(&i.cond) {
            return None;
        }
        let then = single_expr(&i.then_branch)?;
        let (_, else_e) = i.else_branch.as_ref()?;
        let else_expr: &Expr = match &**else_e {
            Expr::Block(b) => single_expr(&b.block)?,
            e @ Expr::If(_) => e,
            _ => return None,
        };
        if has_control_flow(then) || has_control_flow(else_expr) || needs_statements(then) {
            return None;
        }
        if let Expr::If(inner) = else_expr {
            self.ternary_possible(inner)?;
        }
        let c = self.expr_with(&i.cond, &Ty::Bool);
        let c = self.settle(c);
        // `cond ? .red : .primary` would mix two style types: name them.
        let saved_implicit = std::mem::replace(&mut self.implicit_member, false);
        let a = self.expr_with(then, expected);
        let a = self.settle(a);
        let mut ty = expected.clone();
        ty.merge(&a.ty);
        let b = self.expr_with(else_expr, &ty);
        let b = self.settle(b);
        self.implicit_member = saved_implicit;
        ty.merge(&b.ty);
        Some(Ex::new(
            format!(
                "{} ? {} : {}",
                wrap(&c, P_TERNARY + 1),
                wrap(&a, P_TERNARY + 1),
                wrap(&b, P_TERNARY)
            ),
            P_TERNARY,
            ty,
        ))
    }

    fn ternary_possible(&self, i: &syn::ExprIf) -> Option<()> {
        if crate::stmt::contains_let(&i.cond) {
            return None;
        }
        let then = single_expr(&i.then_branch)?;
        if has_control_flow(then) || needs_statements(then) {
            return None;
        }
        let (_, e) = i.else_branch.as_ref()?;
        match &**e {
            Expr::Block(b) => {
                let e = single_expr(&b.block)?;
                if has_control_flow(e) || needs_statements(e) {
                    None
                } else {
                    Some(())
                }
            }
            Expr::If(inner) => self.ternary_possible(inner),
            _ => None,
        }
    }
}

fn single_expr(b: &syn::Block) -> Option<&Expr> {
    match b.stmts.as_slice() {
        [syn::Stmt::Expr(e, None)] => Some(e),
        _ => None,
    }
}

pub fn is_self(e: &Expr) -> bool {
    matches!(e, Expr::Path(p) if p.path.is_ident("self"))
}

pub fn member_name(m: &syn::Member) -> String {
    match m {
        syn::Member::Named(id) => id.to_string(),
        syn::Member::Unnamed(i) => i.index.to_string(),
    }
}

/// A closure whose body returns `Ok(…)`/`Err(…)` or uses `?` on a `Result`.
fn closure_returns_result(body: &Expr) -> bool {
    use syn::visit::Visit;
    struct V(bool);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr(&mut self, e: &'ast Expr) {
            match e {
                Expr::Closure(_) => {}
                Expr::Call(c) => {
                    if let Expr::Path(p) = &*c.func {
                        if p.path.is_ident("Ok") || p.path.is_ident("Err") {
                            self.0 = true;
                        }
                    }
                    syn::visit::visit_expr(self, e);
                }
                _ => syn::visit::visit_expr(self, e),
            }
        }
    }
    let mut v = V(false);
    v.visit_expr(body);
    // Only the tail or returns matter, but any Ok/Err is a strong hint.
    v.0
}
