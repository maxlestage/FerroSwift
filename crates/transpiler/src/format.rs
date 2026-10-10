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
                *i = start;
                None
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
    if !matches!(spec.kind.as_str(), "" | "?" | "#?" | "x" | "X" | "b" | "o" | "e" | "E" | "x?" | "X?") {
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
    pub fn format_args(&mut self, args: &[Expr], span: proc_macro2::Span) -> Ex {
        let Some(Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(fmt), .. })) = args.first() else {
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
                    let value = self.format_value(r, &mut next, &positional, &named, &mut translated, span);
                    let precision = spec.precision.as_ref().map(|c| match c {
                        Count::Value(n) => n.to_string(),
                        Count::Arg(r) => self.format_value(r, &mut next, &positional, &named, &mut translated, span).code,
                    });
                    let width = spec.width.as_ref().map(|c| match c {
                        Count::Value(n) => n.to_string(),
                        Count::Arg(r) => self.format_value(r, &mut next, &positional, &named, &mut translated, span).code,
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
            let code = self.unsupported(span, "a format string with more placeholders than arguments");
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
    fn format_piece(&mut self, v: &Ex, spec: &Spec, precision: Option<String>, width: Option<String>) -> (bool, String) {
        let x = &v.code;
        let mut plain = false;
        let mut s = match spec.kind.as_str() {
            "?" | "x?" | "X?" => match &v.ty {
                Ty::Int(_) | Ty::Bool => x.clone(),
                Ty::Float(_) if precision.is_some() => {
                    self.helper("fixed");
                    format!("rsFixed({x}, {})", precision.clone().unwrap_or_default())
                }
                _ => {
                    self.helper("debug");
                    if spec.alternate {
                        format!("rsDebug({x}, pretty: true)")
                    } else {
                        format!("rsDebug({x})")
                    }
                }
            },
            "x" | "X" | "b" | "o" => {
                let radix = match spec.kind.as_str() {
                    "x" | "X" => 16,
                    "b" => 2,
                    _ => 8,
                };
                let upper = if spec.kind == "X" { ", uppercase: true" } else { "" };
                let base = format!("String({x}, radix: {radix}{upper})");
                if spec.alternate {
                    let prefix = match radix {
                        16 => "0x",
                        2 => "0b",
                        _ => "0o",
                    };
                    format!("\"{prefix}\" + {base}")
                } else {
                    base
                }
            }
            "e" | "E" => format!("String(format: \"%{}\", {x})", spec.kind),
            _ => match (&v.ty, &precision) {
                (Ty::Float(_), Some(p)) => {
                    self.helper("fixed");
                    format!("rsFixed({x}, {p})")
                }
                (Ty::Unknown, Some(p)) => {
                    self.helper("fixed");
                    format!("rsFixed(Double({x}), {p})")
                }
                (Ty::Str, Some(p)) => format!("String({}.prefix({p}))", wrap(v, P_PRIMARY)),
                (Ty::Float(_), None) => {
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
            let as_string = if s == *x && v.ty != Ty::Str { format!("\"\\({s})\"") } else { s.clone() };
            if spec.zero && spec.align.is_none() {
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
        assert!(matches!(&p[5], Piece::Arg(ArgRef::Index(0), s) if s.align == Some('>') && s.width == Some(Count::Value(8)) && s.precision == Some(Count::Value(2))));
        assert_eq!(p[6], Piece::Text(" {}".into()));
        let z = parse("{:08.3}").unwrap();
        assert!(matches!(&z[0], Piece::Arg(_, s) if s.zero && s.width == Some(Count::Value(8))));
        let f = parse("{:*^10}").unwrap();
        assert!(matches!(&f[0], Piece::Arg(_, s) if s.fill == Some('*') && s.align == Some('^')));
        assert!(parse("{").is_err());
    }
}
