//! `format!` strings: `"{} {name:?} {:.2} {:>8}"` → Swift string interpolation.

use syn::Expr;

use crate::expr::{Ex, P_PRIMARY, wrap};
use crate::translate::Translator;
use crate::ty::Ty;

#[derive(Debug, Clone, PartialEq)]
enum ArgRef {
    Next,
    Index(usize),
    Name(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spec {
    fill: Option<char>,
    align: Option<char>,
    plus: bool,
    alternate: bool,
    zero: bool,
    width: Option<Count>,
    precision: Option<Count>,
    kind: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Count {
    Value(usize),
    Arg(ArgRef),
}

#[derive(Debug, Clone, PartialEq)]
enum Piece {
    Text(String),
    Arg(ArgRef, Spec),
}

fn parse(fmt: &str) -> Result<Vec<Piece>, String> {
    let mut pieces = Vec::new();
    let mut text = String::new();
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                text.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                text.push('}');
            }
            '{' => {
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(c) => inner.push(c),
                        None => return Err("unclosed `{` in format string".into()),
                    }
                }
                if !text.is_empty() {
                    pieces.push(Piece::Text(std::mem::take(&mut text)));
                }
                let (arg, spec) = match inner.split_once(':') {
                    Some((a, s)) => (a.trim().to_string(), s.to_string()),
                    None => (inner.trim().to_string(), String::new()),
                };
                pieces.push(Piece::Arg(arg_ref(&arg), parse_spec(&spec)?));
            }
            '}' => return Err("unmatched `}` in format string".into()),
            c => text.push(c),
        }
    }
    if !text.is_empty() {
        pieces.push(Piece::Text(text));
    }
    Ok(pieces)
}

fn arg_ref(s: &str) -> ArgRef {
    if s.is_empty() {
        ArgRef::Next
    } else if let Ok(i) = s.parse() {
        ArgRef::Index(i)
    } else {
        ArgRef::Name(s.to_string())
    }
}

fn parse_spec(s: &str) -> Result<Spec, String> {
    let mut spec = Spec::default();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    // [[fill]align]
    if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '^') {
        spec.fill = Some(chars[0]);
        spec.align = Some(chars[1]);
        i = 2;
    } else if !chars.is_empty() && matches!(chars[0], '<' | '>' | '^') {
        spec.align = Some(chars[0]);
        i = 1;
    }
    if chars.get(i) == Some(&'+') {
        spec.plus = true;
        i += 1;
    } else if chars.get(i) == Some(&'-') {
        i += 1;
    }
    if chars.get(i) == Some(&'#') {
        spec.alternate = true;
        i += 1;
    }
    if chars.get(i) == Some(&'0') {
        spec.zero = true;
        i += 1;
    }
    let count = |i: &mut usize| -> Option<Count> {
        let start = *i;
        while *i < chars.len() && (chars[*i].is_alphanumeric() || chars[*i] == '_') {
            *i += 1;
        }
        if *i == start {
            return None;
        }
        let word: String = chars[start..*i].iter().collect();
        if chars.get(*i) == Some(&'$') {
            *i += 1;
            return Some(Count::Arg(arg_ref(&word)));
        }
        match word.parse() {
            Ok(n) => Some(Count::Value(n)),
            Err(_) => {
                // `08b`: the width, then the kind.
                let digits = word.chars().take_while(char::is_ascii_digit).count();
                *i = start + digits;
                (digits > 0).then(|| Count::Value(word[..digits].parse().unwrap_or(0)))
            }
        }
    };
    spec.width = count(&mut i);
    if chars.get(i) == Some(&'.') {
        i += 1;
        if chars.get(i) == Some(&'*') {
            i += 1;
            spec.precision = Some(Count::Arg(ArgRef::Next));
        } else {
            spec.precision = count(&mut i);
        }
    }
    spec.kind = chars[i..].iter().collect();
    if !matches!(
        spec.kind.as_str(),
        "" | "?" | "#?" | "x" | "X" | "b" | "o" | "e" | "E" | "x?" | "X?"
    ) {
        return Err(format!("unsupported format `{{:{s}}}`"));
    }
    if spec.kind == "#?" {
        spec.kind = "?".into();
        spec.alternate = true;
    }
    Ok(spec)
}

impl Translator<'_> {
    /// A `format!`-like macro: the format string literal and its arguments.
    /// Whether a method of this name changes its receiver (`pop`, `push`, a `&mut self`
    /// method of the crate).
    pub fn is_mutating_method(&self, name: &str) -> bool {
        let std = matches!(
            name,
            "pop"
                | "push"
                | "push_str"
                | "insert"
                | "remove"
                | "truncate"
                | "clear"
                | "drain"
                | "take"
                | "replace"
                | "swap"
                | "sort"
                | "sort_by"
                | "sort_by_key"
                | "sort_unstable"
                | "reverse"
                | "retain"
                | "dedup"
                | "append"
                | "extend"
                | "pop_front"
                | "pop_back"
                | "push_front"
                | "push_back"
                | "next"
                | "split_off"
                | "swap_remove"
                | "resize"
                | "fill"
                | "take_if"
                | "get_or_insert"
                | "get_or_insert_with"
                | "or_insert"
                | "or_default"
                | "or_insert_with"
                | "entry"
                | "iter_mut"
                | "as_mut"
                | "get_mut"
                | "last_mut"
                | "first_mut"
                | "back_mut"
                | "front_mut"
        );
        std || self.syms.methods.values().any(|ms| {
            ms.get(name)
                .is_some_and(|s| s.self_kind == crate::symbols::SelfKind::RefMut)
        })
    }

    /// Whether `e` calls a method that changes its receiver (`s.pop()`, `v.push(x)`, a
    /// `&mut self` method of the crate).
    fn has_mutating_call(&self, e: &Expr) -> bool {
        use syn::visit::Visit;
        struct V<'a, 'b> {
            t: &'a Translator<'b>,
            found: bool,
        }
        impl<'ast> Visit<'ast> for V<'_, '_> {
            fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
                if self.t.is_mutating_method(&m.method.to_string()) {
                    self.found = true;
                }
                syn::visit::visit_expr_method_call(self, m);
            }
        }
        let mut v = V {
            t: self,
            found: false,
        };
        v.visit_expr(e);
        v.found
    }

    pub fn format_args(&mut self, args: &[Expr], span: proc_macro2::Span) -> Ex {
        let Some(Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(fmt),
            ..
        })) = args.first()
        else {
            let code = self.unsupported(span, "a format string that is not a literal");
            return Ex::primary(code, Ty::Str);
        };
        let pieces = match parse(&fmt.value()) {
            Ok(p) => p,
            Err(e) => {
                let code = self.unsupported(span, &e);
                return Ex::primary(code, Ty::Str);
            }
        };
        // Positional and named arguments.
        let mut positional = Vec::new();
        let mut named = Vec::new();
        for a in &args[1..] {
            match a {
                Expr::Assign(asg) => {
                    if let Expr::Path(p) = &*asg.left {
                        if let Some(id) = p.path.get_ident() {
                            named.push((id.to_string(), (*asg.right).clone()));
                            continue;
                        }
                    }
                    positional.push(a.clone());
                }
                a => positional.push(a.clone()),
            }
        }
        // `println!("{s:?} {:?}", s.pop())`: Rust evaluates the arguments first, the captured
        // names after. Arguments that change something are computed before the text.
        if self.preludes.last().is_some()
            && positional
                .iter()
                .chain(named.iter().map(|(_, e)| e))
                .any(|e| self.has_mutating_call(e))
        {
            for (i, e) in positional
                .iter_mut()
                .chain(named.iter_mut().map(|(_, e)| e))
                .enumerate()
            {
                let v = self.expr(e);
                let v = self.settle(v);
                let tmp = self.fresh("arg");
                self.prelude(format!("let {tmp} = {}", v.code));
                let rust = format!("__format_arg{i}");
                self.declare_alias(&rust, tmp, v.ty);
                let ident = syn::Ident::new(&rust, span);
                *e = syn::parse_quote!(#ident);
            }
        }
        let mut translated: Vec<Option<Ex>> = vec![None; positional.len()];
        let mut next = 0;
        let mut out = String::from("\"");
        let mut single: Option<Ex> = None;
        let only_one = pieces.len() == 1;
        for piece in &pieces {
            match piece {
                Piece::Text(t) => {
                    let lit = crate::expr::swift_string(t);
                    out.push_str(&lit[1..lit.len() - 1]);
                }
                Piece::Arg(r, spec) => {
                    // `{:.*}` takes the precision from the next argument, then the value.
                    let star = matches!(spec.precision, Some(Count::Arg(ArgRef::Next)));
                    let mut precision = None;
                    if star {
                        precision = Some(
                            self.format_value(
                                &ArgRef::Next,
                                &mut next,
                                &positional,
                                &named,
                                &mut translated,
                                span,
                            )
                            .code,
                        );
                    }
                    let value =
                        self.format_value(r, &mut next, &positional, &named, &mut translated, span);
                    if !star {
                        precision = spec.precision.as_ref().map(|c| match c {
                            Count::Value(n) => n.to_string(),
                            Count::Arg(r) => {
                                self.format_value(
                                    r,
                                    &mut next,
                                    &positional,
                                    &named,
                                    &mut translated,
                                    span,
                                )
                                .code
                            }
                        });
                    }
                    let width = spec.width.as_ref().map(|c| match c {
                        Count::Value(n) => n.to_string(),
                        Count::Arg(r) => {
                            self.format_value(
                                r,
                                &mut next,
                                &positional,
                                &named,
                                &mut translated,
                                span,
                            )
                            .code
                        }
                    });
                    let text = self.format_piece(&value, spec, precision, width);
                    if only_one && text.0 && value.ty == Ty::Str {
                        single = Some(value.clone());
                    }
                    out.push_str("\\(");
                    out.push_str(&text.1);
                    out.push(')');
                }
            }
        }
        out.push('"');
        if let Some(v) = single {
            return Ex { ty: Ty::Str, ..v };
        }
        Ex::primary(out, Ty::Str)
    }

    fn format_value(
        &mut self,
        r: &ArgRef,
        next: &mut usize,
        positional: &[Expr],
        named: &[(String, Expr)],
        translated: &mut [Option<Ex>],
        span: proc_macro2::Span,
    ) -> Ex {
        let index = match r {
            ArgRef::Next => {
                let i = *next;
                *next += 1;
                Some(i)
            }
            ArgRef::Index(i) => Some(*i),
            ArgRef::Name(n) => {
                if let Some((_, e)) = named.iter().find(|(k, _)| k == n) {
                    let v = self.expr(e);
                    return self.interpolable(v);
                }
                // `{name}` captures a variable.
                let e: Expr = syn::parse_str(n).unwrap_or_else(|_| syn::parse_quote!(missing));
                let v = self.expr(&e);
                return self.interpolable(v);
            }
        };
        let Some(i) = index.filter(|i| *i < positional.len()) else {
            let code = self.unsupported(
                span,
                "a format string with more placeholders than arguments",
            );
            return Ex::primary(code, Ty::Unknown);
        };
        if let Some(v) = &translated[i] {
            return v.clone();
        }
        let v = self.expr(&positional[i]);
        let v = self.interpolable(v);
        translated[i] = Some(v.clone());
        v
    }

    /// Code that can sit inside `\( … )`: one line, no throwing call left unmarked.
    fn interpolable(&mut self, v: Ex) -> Ex {
        let v = self.settle(v);
        if v.code.contains('\n') {
            let tmp = self.fresh("text");
            self.prelude(format!("let {tmp} = {}", v.code));
            return Ex::primary(tmp, v.ty);
        }
        v
    }

    /// The interpolated expression of one placeholder; `true` when it is the plain value.
    fn format_piece(
        &mut self,
        v: &Ex,
        spec: &Spec,
        precision: Option<String>,
        width: Option<String>,
    ) -> (bool, String) {
        let x = &v.code;
        let mut plain = false;
        // A `Display` written with `write!` ignores the spec it is given; one asking its
        // formatter (`f.pad`, `f.precision()`) is given the spec.
        let user_display = match &v.ty {
            Ty::Named(n, _) => self
                .rust_name_of_swift(n)
                .filter(|rust| self.syms.display.contains(rust))
                .map(|rust| self.syms.fmt_queries.contains(&rust)),
            _ => None,
        };
        if spec.kind.is_empty() {
            match user_display {
                Some(true) => {
                    let mut parts = Vec::new();
                    if let Some(w) = &width {
                        parts.push(format!("width: {w}"));
                    }
                    if let Some(p) = &precision {
                        parts.push(format!("precision: {p}"));
                    }
                    if let Some(a) = spec.align {
                        parts.push(format!("align: \"{a}\""));
                    }
                    if let Some(f) = spec.fill.filter(|f| *f != ' ') {
                        parts.push(format!(
                            "fill: {}",
                            crate::expr::swift_string(&f.to_string())
                        ));
                    }
                    if spec.zero {
                        parts.push("zero: true".into());
                    }
                    if spec.alternate {
                        parts.push("alternate: true".into());
                    }
                    if spec.plus {
                        parts.push("signPlus: true".into());
                    }
                    if parts.is_empty() {
                        return (true, x.clone());
                    }
                    return (
                        false,
                        format!(
                            "{}.rsFormatted(RsFormatter({}))",
                            wrap(v, P_PRIMARY),
                            parts.join(", ")
                        ),
                    );
                }
                Some(false) => return (true, x.clone()),
                None => {}
            }
        }
        let is_debug = matches!(spec.kind.as_str(), "?" | "x?" | "X?");
        let hex = match spec.kind.as_str() {
            "x?" => Some('x'),
            "X?" => Some('X'),
            _ => None,
        };
        // Rust's derived `Debug` passes the width and precision down to the numbers inside.
        let debug_style = if is_debug && (width.is_some() || precision.is_some() || hex.is_some()) {
            let mut parts = Vec::new();
            if let Some(w) = &width {
                parts.push(format!("width: {w}"));
                if let Some(a) = spec.align {
                    parts.push(format!("align: \"{a}\""));
                }
                if let Some(f) = spec.fill {
                    if f != ' ' {
                        parts.push(format!(
                            "fill: {}",
                            crate::expr::swift_string(&f.to_string())
                        ));
                    }
                }
                if spec.zero {
                    parts.push("zero: true".into());
                }
            }
            if let Some(p) = &precision {
                parts.push(format!("precision: {p}"));
            }
            if let Some(h) = hex {
                parts.push(format!("hex: \"{h}\""));
            }
            Some(format!(", style: RsDebugStyle({})", parts.join(", ")))
        } else {
            None
        };
        let width = if debug_style.is_some() { None } else { width };
        let mut s = match spec.kind.as_str() {
            "?" | "x?" | "X?" => match (&v.ty, &debug_style) {
                (Ty::Int(_) | Ty::Bool, None) => x.clone(),
                // A `BTreeSet` prints in order; Swift's `Set` has none.
                (Ty::Set(_, true), _) => {
                    self.helper("debug");
                    format!(
                        "rsDebugSet({x}.sorted(){})",
                        if spec.alternate { ", pretty: true" } else { "" }
                    )
                }
                _ => {
                    self.helper("debug");
                    let style = debug_style.clone().unwrap_or_default();
                    if spec.alternate {
                        format!("rsDebug({x}, pretty: true{style})")
                    } else {
                        format!("rsDebug({x}{style})")
                    }
                }
            },
            "x" | "X" | "b" | "o" => {
                let radix = match spec.kind.as_str() {
                    "x" | "X" => 16,
                    "b" => 2,
                    _ => 8,
                };
                let upper = if spec.kind == "X" {
                    ", uppercase: true"
                } else {
                    ""
                };
                let trait_name = match spec.kind.as_str() {
                    "x" => "LowerHex",
                    "X" => "UpperHex",
                    "b" => "Binary",
                    _ => "Octal",
                };
                // `impl fmt::Binary for Flags`: the type's own text.
                let own = match &v.ty {
                    Ty::Named(n, _) => self.rust_name_of_swift(n).filter(|rust| {
                        self.syms
                            .radix_fmt
                            .contains(&(rust.clone(), trait_name.to_string()))
                    }),
                    _ => None,
                };
                // A negative number prints its two's complement, in the width of its type.
                let bits = match &v.ty {
                    Ty::Int(k) if k.signed() => {
                        let b = k.rust_bits();
                        let prefix = if b == 128 { "Rs" } else { "" };
                        format!(
                            "{prefix}UInt{b}(bitPattern: {prefix}Int{b}(truncatingIfNeeded: {x}))"
                        )
                    }
                    _ => x.clone(),
                };
                let base = if own.is_some() {
                    format!("{}.rs{trait_name}", wrap(v, P_PRIMARY))
                } else {
                    format!("String({bits}, radix: {radix}{upper})")
                };
                if spec.alternate {
                    let prefix = match radix {
                        16 => "0x",
                        2 => "0b",
                        _ => "0o",
                    };
                    // `{:#010x}`: the zeros come after the prefix, which counts in the width.
                    if let (true, Some(w)) = (spec.zero, &width) {
                        self.helper("pad");
                        return (false, format!("\"{prefix}\" + rsPadZero({base}, {w} - 2)"));
                    }
                    format!("\"{prefix}\" + {base}")
                } else {
                    base
                }
            }
            "e" | "E" => {
                self.helper("exp");
                let upper = if spec.kind == "E" {
                    ", upper: true"
                } else {
                    ""
                };
                match &precision {
                    Some(p) => format!("rsExp({x}, {p}{upper})"),
                    None => format!("rsExp({x}{upper})"),
                }
            }
            _ => match (&v.ty, &precision) {
                (Ty::Float(_), Some(p)) => {
                    self.helper("fixed");
                    format!("rsFixed({x}, {p})")
                }
                (Ty::Unknown, Some(p)) => {
                    self.helper("fixed");
                    format!("rsFixed(Double({x}), {p})")
                }
                // `{:.2}` of a text: its first scalars, as Rust counts `char`s.
                (Ty::Str, Some(p)) => {
                    format!("String({}.unicodeScalars.prefix({p}))", wrap(v, P_PRIMARY))
                }
                (Ty::Float(_), None) => {
                    self.helper("display");
                    format!("rsDisplay({x})")
                }
                // A type parameter: a float as Rust writes it, if that is what it holds.
                (Ty::Named(n, _), None) if self.generics.contains(n) => {
                    self.helper("display");
                    format!("rsDisplay({x})")
                }
                _ => {
                    plain = true;
                    x.clone()
                }
            },
        };
        if spec.plus {
            self.helper("pad");
            s = format!("rsSign({s})");
            plain = false;
        }
        if let Some(w) = width {
            self.helper("pad");
            let numeric = v.ty.is_numeric();
            let as_string = if s == *x && v.ty != Ty::Str {
                format!("\"\\({s})\"")
            } else {
                s.clone()
            };
            // The `0` flag pads with zeros whatever the alignment asks.
            if spec.zero {
                s = format!("rsPadZero({as_string}, {w})");
            } else {
                let align = spec.align.unwrap_or(if numeric { '>' } else { '<' });
                let fill = spec.fill.unwrap_or(' ');
                let fill = crate::expr::swift_string(&fill.to_string());
                s = format!("rsPad({as_string}, {w}, \"{align}\", {fill})");
            }
            plain = false;
        }
        (plain, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_format_strings() {
        let p = parse("a {} {x:?} {0:>8.2} {{}}").unwrap();
        assert_eq!(p.len(), 7);
        assert_eq!(p[0], Piece::Text("a ".into()));
        assert!(matches!(&p[3], Piece::Arg(ArgRef::Name(n), s) if n == "x" && s.kind == "?"));
        assert!(
            matches!(&p[5], Piece::Arg(ArgRef::Index(0), s) if s.align == Some('>') && s.width == Some(Count::Value(8)) && s.precision == Some(Count::Value(2)))
        );
        assert_eq!(p[6], Piece::Text(" {}".into()));
        let z = parse("{:08.3}").unwrap();
        assert!(matches!(&z[0], Piece::Arg(_, s) if s.zero && s.width == Some(Count::Value(8))));
        let f = parse("{:*^10}").unwrap();
        assert!(matches!(&f[0], Piece::Arg(_, s) if s.fill == Some('*') && s.align == Some('^')));
        assert!(parse("{").is_err());
    }
}
