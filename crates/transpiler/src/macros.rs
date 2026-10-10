//! Macros of the standard library: `println!`, `format!`, `vec!`, `assert!`, `matches!`…

use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, Macro, Token};

use crate::expr::{Ex, P_PRIMARY};
use crate::translate::Translator;
use crate::ty::Ty;

fn macro_name(mac: &Macro) -> String {
    mac.path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default()
}

/// Macros that are statements (or never return): written on their own line.
pub fn is_statement_macro(mac: &Macro) -> bool {
    matches!(
        macro_name(mac).as_str(),
        "println"
            | "print"
            | "eprintln"
            | "eprint"
            | "assert"
            | "assert_eq"
            | "assert_ne"
            | "debug_assert"
            | "debug_assert_eq"
            | "debug_assert_ne"
            | "panic"
            | "unreachable"
            | "todo"
            | "unimplemented"
            | "write"
            | "writeln"
    )
}

fn comma_args(mac: &Macro) -> syn::Result<Vec<Expr>> {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    Ok(parser.parse2(mac.tokens.clone())?.into_iter().collect())
}

impl Translator<'_> {
    /// A macro on its own line.
    pub fn macro_stmt(&mut self, mac: &Macro) -> Vec<String> {
        let name = macro_name(mac);
        let span = mac.span();
        let args = match comma_args(mac) {
            Ok(a) => a,
            Err(_) if name == "vec" || name == "matches" => vec![],
            Err(e) => {
                return vec![self.unsupported(span, &format!("the arguments of {name}!: {e}"))];
            }
        };
        match name.as_str() {
            "println" | "print" | "eprintln" | "eprint" => {
                let newline = name.ends_with("ln");
                let stderr = name.starts_with('e');
                if stderr {
                    self.helper("eprint");
                }
                if args.is_empty() {
                    return vec![if stderr {
                        "rsEprint(\"\")".into()
                    } else {
                        "print()".into()
                    }];
                }
                let text = self.format_args(&args, span);
                let f = if stderr { "rsEprint" } else { "print" };
                if newline {
                    vec![format!("{f}({})", text.code)]
                } else {
                    vec![format!("{f}({}, terminator: \"\")", text.code)]
                }
            }
            "assert" | "debug_assert" => {
                let f = if name == "assert" {
                    "precondition"
                } else {
                    "assert"
                };
                let Some(cond) = args.first() else {
                    return vec![];
                };
                let c = self.expr_with(cond, &Ty::Bool);
                let c = self.settle(c);
                if args.len() > 1 {
                    let msg = self.format_args(&args[1..], span);
                    vec![format!("{f}({}, {})", c.code, msg.code)]
                } else {
                    vec![format!("{f}({})", c.code)]
                }
            }
            "assert_eq" | "assert_ne" | "debug_assert_eq" | "debug_assert_ne" => {
                let f = if name.starts_with("debug") {
                    "assert"
                } else {
                    "precondition"
                };
                let op = if name.ends_with("eq") { "==" } else { "!=" };
                if args.len() < 2 {
                    return vec![
                        self.unsupported(span, &format!("{name}! with fewer than two arguments")),
                    ];
                }
                let a = self.expr(&args[0]);
                let a = self.settle(a);
                let b = self.expr_with(&args[1], &a.ty);
                let b = self.settle(b);
                let msg = if args.len() > 2 {
                    format!(", {}", self.format_args(&args[2..], span).code)
                } else {
                    String::new()
                };
                vec![format!(
                    "{f}({} {op} {}{msg})",
                    crate::expr::wrap(&a, crate::expr::P_CMP + 1),
                    crate::expr::wrap(&b, crate::expr::P_CMP + 1)
                )]
            }
            "write" | "writeln" => {
                let newline = name == "writeln";
                if args.is_empty() {
                    return vec![];
                }
                let target = self.expr(&args[0]);
                let target_code = target
                    .code
                    .strip_prefix("fmt:")
                    .map(str::to_string)
                    .unwrap_or(target.code.clone());
                let text = if args.len() > 1 {
                    let t = self.format_args(&args[1..], span);
                    if newline {
                        match t.code.strip_suffix('"') {
                            Some(s) if t.code.starts_with('"') => format!("{s}\\n\""),
                            _ => format!("{} + \"\\n\"", t.code),
                        }
                    } else {
                        t.code
                    }
                } else {
                    "\"\\n\"".into()
                };
                vec![format!("{target_code} += {text}")]
            }
            "panic" | "unreachable" | "todo" | "unimplemented" => {
                let e = self.macro_expr(mac, &Ty::Never);
                vec![e.code]
            }
            _ => {
                let e = self.macro_expr(mac, &Ty::Unknown);
                if e.code.is_empty() {
                    vec![]
                } else if matches!(e.ty, Ty::Unit | Ty::Never) {
                    vec![e.code]
                } else {
                    vec![format!("_ = {}", e.code)]
                }
            }
        }
    }

    /// A macro used as a value.
    pub fn macro_expr(&mut self, mac: &Macro, expected: &Ty) -> Ex {
        let name = macro_name(mac);
        let span = mac.span();
        match name.as_str() {
            "format" | "format_args" => {
                let args = match comma_args(mac) {
                    Ok(a) => a,
                    Err(e) => {
                        let code =
                            self.unsupported(span, &format!("the arguments of format!: {e}"));
                        return Ex::primary(code, Ty::Str);
                    }
                };
                self.format_args(&args, span)
            }
            "vec" => self.vec_macro(mac, expected),
            "panic" | "unreachable" | "todo" | "unimplemented" => {
                let args = comma_args(mac).unwrap_or_default();
                let default = match name.as_str() {
                    "unreachable" => "internal error: entered unreachable code",
                    "todo" => "not yet implemented",
                    "unimplemented" => "not implemented",
                    _ => "explicit panic",
                };
                let msg = if args.is_empty() {
                    crate::expr::swift_string(default)
                } else {
                    self.format_args(&args, span).code
                };
                Ex::primary(format!("fatalError({msg})"), Ty::Never)
            }
            "matches" => self.matches_macro(mac),
            "dbg" => {
                let args = comma_args(mac).unwrap_or_default();
                match args.first() {
                    Some(a) => self.expr_with(a, expected),
                    None => Ex::primary("()", Ty::Unit),
                }
            }
            "line" => Ex::primary("#line", Ty::Int(crate::ty::IntKind::U32)),
            "column" => Ex::primary("#column", Ty::Int(crate::ty::IntKind::U32)),
            "file" => Ex::primary("#file", Ty::Str),
            "stringify" => Ex::primary(crate::expr::swift_string(&mac.tokens.to_string()), Ty::Str),
            "concat" => {
                let args = comma_args(mac).unwrap_or_default();
                let mut s = String::new();
                for a in &args {
                    if let Expr::Lit(l) = a {
                        match &l.lit {
                            syn::Lit::Str(v) => s.push_str(&v.value()),
                            syn::Lit::Int(v) => s.push_str(v.base10_digits()),
                            syn::Lit::Bool(v) => s.push_str(&v.value.to_string()),
                            syn::Lit::Char(v) => s.push(v.value()),
                            _ => {}
                        }
                    }
                }
                Ex::primary(crate::expr::swift_string(&s), Ty::Str)
            }
            "write" | "writeln" | "println" | "print" | "eprintln" | "eprint" | "assert"
            | "assert_eq" | "assert_ne" | "debug_assert" | "debug_assert_eq"
            | "debug_assert_ne" => {
                let lines = self.macro_stmt(mac);
                Ex::new(lines.join("\n"), crate::expr::P_ASSIGN, Ty::Unit)
            }
            _ => {
                let code = self.unsupported(span, &format!("the macro {name}!"));
                Ex::primary(code, Ty::Unknown)
            }
        }
    }

    fn vec_macro(&mut self, mac: &Macro, expected: &Ty) -> Ex {
        let item = expected.item();
        // vec![x; n]
        let repeat = (|input: syn::parse::ParseStream<'_>| {
            let value: Expr = input.parse()?;
            input.parse::<Token![;]>()?;
            let count: Expr = input.parse()?;
            Ok((value, count))
        })
        .parse2(mac.tokens.clone());
        if let Ok((value, count)) = repeat {
            let v = self.expr_with(&value, &item);
            let v = self.settle(v);
            let n = self.expr_with(&count, &Ty::Int(crate::ty::IntKind::Usize));
            let n = self.settle(n);
            return Ex::primary(
                format!("Array(repeating: {}, count: {})", v.code, n.code),
                Ty::Vec(Box::new(v.ty)),
            );
        }
        let args = match comma_args(mac) {
            Ok(a) => a,
            Err(e) => {
                let code = self.unsupported(mac.span(), &format!("the arguments of vec!: {e}"));
                return Ex::primary(code, Ty::Vec(Box::new(Ty::Unknown)));
            }
        };
        let mut it = item.clone();
        let mut codes = Vec::new();
        for a in &args {
            let v = self.expr_with(a, &item);
            let v = self.settle(v);
            it.merge(&v.ty);
            codes.push(v.code);
        }
        let joined = codes.join(", ");
        let code = if joined.len() > 80 || joined.contains('\n') {
            format!(
                "[\n{}\n]",
                crate::translate::indent_lines(&codes.join(",\n"))
            )
        } else {
            format!("[{joined}]")
        };
        Ex::primary(code, Ty::Vec(Box::new(it)))
    }

    /// `matches!(x, Pattern if guard)` → a closure testing `if case`.
    fn matches_macro(&mut self, mac: &Macro) -> Ex {
        let parsed = (|input: syn::parse::ParseStream<'_>| {
            let value: Expr = input.parse()?;
            input.parse::<Token![,]>()?;
            let pat = syn::Pat::parse_multi_with_leading_vert(input)?;
            let guard = if input.peek(Token![if]) {
                input.parse::<Token![if]>()?;
                Some(input.parse::<Expr>()?)
            } else {
                None
            };
            let _ = input.parse::<Option<Token![,]>>();
            Ok((value, pat, guard))
        })
        .parse2(mac.tokens.clone());
        let Ok((value, pat, guard)) = parsed else {
            let code = self.unsupported(mac.span(), "the arguments of matches!");
            return Ex::primary(code, Ty::Bool);
        };
        let v = self.expr(&value);
        let v = self.settle(v);
        // `matches!(o, Some(_))` / `None`: a comparison.
        if guard.is_none() {
            if let syn::Pat::TupleStruct(ts) = &pat {
                let ctor = ts
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                if ctor == "Some" && ts.elems.len() == 1 && matches!(ts.elems[0], syn::Pat::Wild(_))
                {
                    return Ex::new(
                        format!("{} != nil", crate::expr::wrap(&v, crate::expr::P_CMP + 1)),
                        crate::expr::P_CMP,
                        Ty::Bool,
                    );
                }
            }
            if let syn::Pat::Ident(id) = &pat {
                if id.ident == "None" {
                    return Ex::new(
                        format!("{} == nil", crate::expr::wrap(&v, crate::expr::P_CMP + 1)),
                        crate::expr::P_CMP,
                        Ty::Bool,
                    );
                }
            }
        }
        self.push_scope();
        let pats = self.case_patterns(&pat, &v.ty, &v.code);
        let mut guards = pats.guards.clone();
        if let Some(g) = &guard {
            let g = self.expr_with(g, &Ty::Bool);
            let g = self.settle(g);
            guards.push(g.code);
        }
        self.pop_scope();
        let w = if guards.is_empty() {
            String::new()
        } else {
            format!(" where {}", guards.join(" && "))
        };
        let lets = if pats.lets.is_empty() {
            String::new()
        } else {
            format!("{}; ", pats.lets.join("; "))
        };
        Ex::primary(
            format!(
                "{{ switch {} {{ case {}{w}: {lets}return true; default: return false }} }}()",
                crate::expr::wrap(&v, P_PRIMARY),
                pats.code
            ),
            Ty::Bool,
        )
    }
}
