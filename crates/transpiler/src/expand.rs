//! Declarative macros (`macro_rules!`): their uses are expanded before translation, as rustc
//! expands them, so that the code they produce is translated like the rest.

use std::collections::HashMap;

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::ext::IdentExt;
use syn::parse::discouraged::Speculative;
use syn::parse::{ParseStream, Parser};
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, Item, Stmt};

/// One element of a rule's pattern.
#[derive(Clone, Debug)]
enum Matcher {
    /// A token to find as is (`=>`, `,`, an identifier).
    Token(TokenTree),
    Group(Delimiter, Vec<Matcher>),
    /// `$name:kind`.
    Fragment(String, String),
    /// `$( … ) sep op`.
    Repeat(Vec<Matcher>, Option<TokenTree>, char),
}

/// One element of a rule's body.
#[derive(Clone, Debug)]
enum Transcriber {
    Token(TokenTree),
    Group(Delimiter, Vec<Transcriber>),
    Var(String),
    Repeat(Vec<Transcriber>, Option<TokenTree>),
}

#[derive(Clone, Debug)]
struct Rule {
    matcher: Vec<Matcher>,
    body: Vec<Transcriber>,
}

/// What a fragment matched: tokens, or one binding per repetition.
#[derive(Clone, Debug)]
enum Bound {
    Leaf(TokenStream),
    Seq(Vec<Bound>),
}

type Env = HashMap<String, Bound>;

/// The `macro_rules!` of a crate, by name.
#[derive(Default)]
pub struct Macros {
    rules: HashMap<String, Vec<Rule>>,
    /// Where expansions were placed (line, column): their literals may be folded.
    spans: std::cell::RefCell<std::collections::HashSet<(usize, usize)>>,
}

impl Macros {
    /// Collects the definitions of the files.
    pub fn collect(files: &[&syn::File]) -> Macros {
        struct Collector<'a>(&'a mut Macros);
        impl<'ast> syn::visit::Visit<'ast> for Collector<'_> {
            fn visit_item_macro(&mut self, m: &'ast syn::ItemMacro) {
                if m.mac.path.is_ident("macro_rules") {
                    if let Some(name) = &m.ident {
                        let rules = parse_rules(m.mac.tokens.clone());
                        if !rules.is_empty() {
                            self.0.rules.insert(name.to_string(), rules);
                        }
                    }
                }
            }
        }
        let mut out = Macros::default();
        for f in files {
            syn::visit::Visit::visit_file(&mut Collector(&mut out), f);
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    fn name_of(&self, path: &syn::Path) -> Option<String> {
        let last = path.segments.last()?.ident.to_string();
        self.rules.contains_key(&last).then_some(last)
    }

    /// The tokens a use of the macro expands to, if a rule matches.
    fn expand(&self, name: &str, input: TokenStream, span: Span) -> Option<TokenStream> {
        for rule in self.rules.get(name)? {
            let matcher = rule.matcher.clone();
            let parsed = (move |s: ParseStream| -> syn::Result<Env> {
                let mut env = Env::new();
                match_seq(s, &matcher, &mut env)?;
                if !s.is_empty() {
                    return Err(s.error("unexpected tokens"));
                }
                Ok(env)
            })
            .parse2(input.clone());
            if let Ok(env) = parsed {
                let out = transcribe(&rule.body, &env);
                let at = span.start();
                self.spans.borrow_mut().insert((at.line, at.column));
                return Some(respan(out, span));
            }
        }
        None
    }

    /// Expands the uses of the macros in a token stream (the arguments of `println!`,
    /// `vec![…]`): each becomes a parenthesized expression.
    fn expand_tokens(&self, ts: TokenStream, changed: &mut bool) -> TokenStream {
        let toks: Vec<TokenTree> = ts.into_iter().collect();
        let mut out = Vec::new();
        let mut i = 0;
        while i < toks.len() {
            if let (TokenTree::Ident(id), Some(TokenTree::Punct(p)), Some(TokenTree::Group(g))) =
                (&toks[i], toks.get(i + 1), toks.get(i + 2))
            {
                if p.as_char() == '!' && self.rules.contains_key(&id.to_string()) {
                    if let Some(exp) = self.expand(&id.to_string(), g.stream(), id.span()) {
                        let mut group = Group::new(Delimiter::Parenthesis, exp);
                        group.set_span(id.span());
                        out.push(TokenTree::Group(group));
                        *changed = true;
                        i += 3;
                        continue;
                    }
                }
            }
            match &toks[i] {
                TokenTree::Group(g) => {
                    let mut ng = Group::new(g.delimiter(), self.expand_tokens(g.stream(), changed));
                    ng.set_span(g.span());
                    out.push(TokenTree::Group(ng));
                }
                t => out.push(t.clone()),
            }
            i += 1;
        }
        out.into_iter().collect()
    }
}

/// Expands every use of the crate's macros in the files, until none is left, and removes
/// the definitions.
pub fn expand_files(files: &mut [&mut syn::File], macros: &Macros) {
    if macros.is_empty() {
        return;
    }
    for f in files.iter_mut() {
        for _ in 0..64 {
            let mut e = Expander {
                macros,
                changed: false,
            };
            e.visit_file_mut(f);
            if !e.changed {
                break;
            }
        }
        let spans = macros.spans.borrow().clone();
        Fold { spans }.visit_file_mut(f);
    }
}

struct Expander<'a> {
    macros: &'a Macros,
    changed: bool,
}

impl Expander<'_> {
    fn expand_items(&mut self, items: &mut Vec<Item>) {
        let mut out = Vec::new();
        for item in items.drain(..) {
            if let Item::Macro(m) = &item {
                if m.mac.path.is_ident("macro_rules") {
                    continue;
                }
                if let Some(name) = self.macros.name_of(&m.mac.path) {
                    let span = m.mac.path.segments[0].ident.span();
                    if let Some(ts) = self.macros.expand(&name, m.mac.tokens.clone(), span) {
                        if let Ok(file) = syn::parse2::<syn::File>(ts) {
                            out.extend(file.items);
                            self.changed = true;
                            continue;
                        }
                    }
                }
            }
            out.push(item);
        }
        *items = out;
    }
}

impl VisitMut for Expander<'_> {
    fn visit_file_mut(&mut self, f: &mut syn::File) {
        self.expand_items(&mut f.items);
        visit_mut::visit_file_mut(self, f);
    }

    fn visit_item_impl_mut(&mut self, imp: &mut syn::ItemImpl) {
        let mut out = Vec::new();
        for item in imp.items.drain(..) {
            if let syn::ImplItem::Macro(m) = &item {
                if let Some(name) = self.macros.name_of(&m.mac.path) {
                    let span = m.mac.path.segments[0].ident.span();
                    if let Some(ts) = self.macros.expand(&name, m.mac.tokens.clone(), span) {
                        let parsed = (|s: ParseStream| -> syn::Result<Vec<syn::ImplItem>> {
                            let mut items = Vec::new();
                            while !s.is_empty() {
                                items.push(s.parse()?);
                            }
                            Ok(items)
                        })
                        .parse2(ts);
                        if let Ok(items) = parsed {
                            out.extend(items);
                            self.changed = true;
                            continue;
                        }
                    }
                }
            }
            out.push(item);
        }
        imp.items = out;
        visit_mut::visit_item_impl_mut(self, imp);
    }

    fn visit_item_mod_mut(&mut self, m: &mut syn::ItemMod) {
        if let Some((_, items)) = &mut m.content {
            self.expand_items(items);
        }
        visit_mut::visit_item_mod_mut(self, m);
    }

    fn visit_block_mut(&mut self, b: &mut syn::Block) {
        let mut out = Vec::new();
        for s in b.stmts.drain(..) {
            match s {
                Stmt::Item(Item::Macro(m)) if m.mac.path.is_ident("macro_rules") => {}
                Stmt::Macro(sm) => {
                    let expanded = self.macros.name_of(&sm.mac.path).and_then(|name| {
                        let span = sm.mac.path.segments[0].ident.span();
                        self.macros.expand(&name, sm.mac.tokens.clone(), span)
                    });
                    let parsed = expanded.and_then(|ts| syn::Block::parse_within.parse2(ts).ok());
                    match parsed {
                        Some(mut stmts) => {
                            // `m!(…);`: a value the expansion ends with is discarded.
                            if sm.semi_token.is_some() {
                                if let Some(Stmt::Expr(_, semi @ None)) = stmts.last_mut() {
                                    *semi = Some(Default::default());
                                }
                            }
                            out.extend(stmts);
                            self.changed = true;
                        }
                        None => {
                            let mut sm = sm;
                            sm.mac.tokens = self
                                .macros
                                .expand_tokens(sm.mac.tokens.clone(), &mut self.changed);
                            out.push(Stmt::Macro(sm));
                        }
                    }
                }
                s => out.push(s),
            }
        }
        b.stmts = out;
        visit_mut::visit_block_mut(self, b);
    }

    fn visit_expr_mut(&mut self, e: &mut Expr) {
        if let Expr::Macro(m) = e {
            if let Some(name) = self.macros.name_of(&m.mac.path) {
                let span = m.mac.path.segments[0].ident.span();
                if let Some(ts) = self.macros.expand(&name, m.mac.tokens.clone(), span) {
                    let parsed = syn::parse2::<Expr>(ts.clone()).ok().or_else(|| {
                        // Statements in expression position: a block.
                        syn::parse2::<Expr>(quote::quote!({ #ts })).ok()
                    });
                    if let Some(new) = parsed {
                        *e = new;
                        self.changed = true;
                    }
                }
            } else {
                m.mac.tokens = self
                    .macros
                    .expand_tokens(m.mac.tokens.clone(), &mut self.changed);
            }
        }
        visit_mut::visit_expr_mut(self, e);
    }
}

/// `1usize + (1usize + 0usize)`, as a recursive macro writes it: the integer it is, as rustc
/// computes it (Swift's type checker gives up on long chains of literals).
struct Fold {
    spans: std::collections::HashSet<(usize, usize)>,
}

impl Fold {
    /// An integer literal placed by an expansion.
    fn int_lit(&self, e: &Expr) -> Option<(i128, String)> {
        match e {
            Expr::Lit(l) => match &l.lit {
                syn::Lit::Int(i) => {
                    let at = i.span().start();
                    if !self.spans.contains(&(at.line, at.column)) {
                        return None;
                    }
                    Some((i.base10_parse().ok()?, i.suffix().to_string()))
                }
                _ => None,
            },
            Expr::Paren(p) => self.int_lit(&p.expr),
            Expr::Group(g) => self.int_lit(&g.expr),
            _ => None,
        }
    }
}

impl VisitMut for Fold {
    /// `println!("{}", count!(a b))`: the arguments of the standard macros too.
    fn visit_macro_mut(&mut self, m: &mut syn::Macro) {
        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        if let Ok(mut args) = parser.parse2(m.tokens.clone()) {
            for a in args.iter_mut() {
                self.visit_expr_mut(a);
            }
            m.tokens = args.into_token_stream();
        }
    }

    fn visit_expr_mut(&mut self, e: &mut Expr) {
        visit_mut::visit_expr_mut(self, e);
        let span = match e {
            Expr::Binary(b) => self.int_lit(&b.left).map(|_| {
                let mut l = &*b.left;
                while let Expr::Paren(p) = l {
                    l = &p.expr;
                }
                match l {
                    Expr::Lit(x) => x.lit.span(),
                    _ => Span::call_site(),
                }
            }),
            Expr::Paren(p) => self.int_lit(&p.expr).map(|_| p.paren_token.span.join()),
            _ => None,
        }
        .unwrap_or_else(Span::call_site);
        let folded = match e {
            Expr::Binary(b) => match (self.int_lit(&b.left), self.int_lit(&b.right)) {
                (Some((x, sx)), Some((y, sy))) if sx.is_empty() || sy.is_empty() || sx == sy => {
                    let value = match b.op {
                        syn::BinOp::Add(_) => x.checked_add(y),
                        syn::BinOp::Sub(_) => x.checked_sub(y),
                        syn::BinOp::Mul(_) => x.checked_mul(y),
                        _ => None,
                    };
                    value.filter(|v| *v >= 0).map(|v| {
                        let suffix = if sx.is_empty() { sy } else { sx };
                        syn::LitInt::new(&format!("{v}{suffix}"), span)
                    })
                }
                _ => None,
            },
            Expr::Paren(p) => match self.int_lit(&p.expr) {
                Some((v, s)) if v >= 0 => Some(syn::LitInt::new(&format!("{v}{s}"), span)),
                _ => None,
            },
            _ => None,
        };
        if let Some(lit) = folded {
            *e = Expr::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Int(lit),
            });
        }
    }
}

// ----- Rules -----

fn parse_rules(ts: TokenStream) -> Vec<Rule> {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut rules = Vec::new();
    let mut i = 0;
    while i + 3 < toks.len() + 1 {
        let (
            Some(TokenTree::Group(pat)),
            Some(TokenTree::Punct(eq)),
            Some(TokenTree::Punct(gt)),
            Some(TokenTree::Group(body)),
        ) = (
            toks.get(i),
            toks.get(i + 1),
            toks.get(i + 2),
            toks.get(i + 3),
        )
        else {
            break;
        };
        if eq.as_char() != '=' || gt.as_char() != '>' {
            break;
        }
        rules.push(Rule {
            matcher: parse_matcher(pat.stream()),
            body: parse_transcriber(body.stream()),
        });
        i += 4;
        if matches!(toks.get(i), Some(TokenTree::Punct(p)) if p.as_char() == ';') {
            i += 1;
        }
    }
    rules
}

fn is_op(t: &TokenTree) -> Option<char> {
    match t {
        TokenTree::Punct(p) if matches!(p.as_char(), '*' | '+' | '?') => Some(p.as_char()),
        _ => None,
    }
}

/// The separator and operator after `$( … )`, and how many tokens they take.
fn repetition_tail(toks: &[TokenTree], at: usize) -> (Option<TokenTree>, char, usize) {
    match toks.get(at).and_then(is_op) {
        Some(op) => (None, op, 1),
        None => match (toks.get(at), toks.get(at + 1).and_then(is_op)) {
            (Some(sep), Some(op)) => (Some(sep.clone()), op, 2),
            _ => (None, '*', 0),
        },
    }
}

fn parse_matcher(ts: TokenStream) -> Vec<Matcher> {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if let TokenTree::Punct(p) = &toks[i] {
            if p.as_char() == '$' {
                match toks.get(i + 1) {
                    Some(TokenTree::Ident(name)) => {
                        if let (Some(TokenTree::Punct(c)), Some(TokenTree::Ident(kind))) =
                            (toks.get(i + 2), toks.get(i + 3))
                        {
                            if c.as_char() == ':' {
                                out.push(Matcher::Fragment(
                                    name.unraw().to_string(),
                                    kind.to_string(),
                                ));
                                i += 4;
                                continue;
                            }
                        }
                    }
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                        let (sep, op, n) = repetition_tail(&toks, i + 2);
                        out.push(Matcher::Repeat(parse_matcher(g.stream()), sep, op));
                        i += 2 + n;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        match &toks[i] {
            TokenTree::Group(g) => {
                out.push(Matcher::Group(g.delimiter(), parse_matcher(g.stream())))
            }
            t => out.push(Matcher::Token(t.clone())),
        }
        i += 1;
    }
    out
}

fn parse_transcriber(ts: TokenStream) -> Vec<Transcriber> {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if let TokenTree::Punct(p) = &toks[i] {
            if p.as_char() == '$' {
                match toks.get(i + 1) {
                    // `$crate::f()`: the crate is this one.
                    Some(TokenTree::Ident(id)) if id == "crate" => {
                        i += 2;
                        while matches!(toks.get(i), Some(TokenTree::Punct(c)) if c.as_char() == ':')
                        {
                            i += 1;
                        }
                        continue;
                    }
                    Some(TokenTree::Ident(id)) => {
                        out.push(Transcriber::Var(id.unraw().to_string()));
                        i += 2;
                        continue;
                    }
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                        let (sep, _, n) = repetition_tail(&toks, i + 2);
                        out.push(Transcriber::Repeat(parse_transcriber(g.stream()), sep));
                        i += 2 + n;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        match &toks[i] {
            TokenTree::Group(g) => out.push(Transcriber::Group(
                g.delimiter(),
                parse_transcriber(g.stream()),
            )),
            t => out.push(Transcriber::Token(t.clone())),
        }
        i += 1;
    }
    out
}

// ----- Matching -----

fn same_token(a: &TokenTree, b: &TokenTree) -> bool {
    match (a, b) {
        (TokenTree::Ident(x), TokenTree::Ident(y)) => x == y,
        (TokenTree::Punct(x), TokenTree::Punct(y)) => x.as_char() == y.as_char(),
        (TokenTree::Literal(x), TokenTree::Literal(y)) => x.to_string() == y.to_string(),
        _ => false,
    }
}

fn match_seq(input: ParseStream, ms: &[Matcher], env: &mut Env) -> syn::Result<()> {
    for m in ms {
        match_one(input, m, env)?;
    }
    Ok(())
}

fn match_one(input: ParseStream, m: &Matcher, env: &mut Env) -> syn::Result<()> {
    match m {
        Matcher::Token(t) => {
            let got: TokenTree = input.parse()?;
            if !same_token(&got, t) {
                return Err(syn::Error::new(got.span(), "unexpected token"));
            }
        }
        Matcher::Group(d, inner) => {
            let got: TokenTree = input.parse()?;
            let TokenTree::Group(g) = got else {
                return Err(input.error("expected a group"));
            };
            if g.delimiter() != *d {
                return Err(input.error("unexpected delimiter"));
            }
            let inner = inner.clone();
            let found = (move |s: ParseStream| -> syn::Result<Env> {
                let mut e = Env::new();
                match_seq(s, &inner, &mut e)?;
                if !s.is_empty() {
                    return Err(s.error("unexpected tokens"));
                }
                Ok(e)
            })
            .parse2(g.stream())?;
            env.extend(found);
        }
        Matcher::Fragment(name, kind) => {
            let ts = parse_fragment(input, kind)?;
            env.insert(name.clone(), Bound::Leaf(ts));
        }
        Matcher::Repeat(inner, sep, op) => {
            let mut iterations: Vec<Env> = Vec::new();
            loop {
                if *op == '?' && iterations.len() == 1 {
                    break;
                }
                let fork = input.fork();
                if !iterations.is_empty() {
                    if let Some(sep) = sep {
                        match fork.parse::<TokenTree>() {
                            Ok(t) if same_token(&t, sep) => {}
                            _ => break,
                        }
                    }
                }
                let mut e = Env::new();
                if match_seq(&fork, inner, &mut e).is_err() || fork.cursor() == input.cursor() {
                    break;
                }
                input.advance_to(&fork);
                iterations.push(e);
            }
            if *op == '+' && iterations.is_empty() {
                return Err(input.error("expected at least one repetition"));
            }
            for var in matcher_vars(inner) {
                let seq = iterations
                    .iter()
                    .map(|e| e.get(&var).cloned().unwrap_or(Bound::Seq(vec![])))
                    .collect();
                env.insert(var, Bound::Seq(seq));
            }
        }
    }
    Ok(())
}

fn matcher_vars(ms: &[Matcher]) -> Vec<String> {
    let mut out = Vec::new();
    for m in ms {
        match m {
            Matcher::Fragment(n, _) => out.push(n.clone()),
            Matcher::Group(_, inner) | Matcher::Repeat(inner, _, _) => {
                out.extend(matcher_vars(inner))
            }
            Matcher::Token(_) => {}
        }
    }
    out
}

fn parse_fragment(input: ParseStream, kind: &str) -> syn::Result<TokenStream> {
    Ok(match kind {
        "expr" => {
            let e: Expr = input.parse()?;
            // An expression stays one operand where it is substituted (`$x * 2`).
            let atomic = matches!(
                e,
                Expr::Lit(_)
                    | Expr::Path(_)
                    | Expr::Call(_)
                    | Expr::MethodCall(_)
                    | Expr::Field(_)
                    | Expr::Index(_)
                    | Expr::Paren(_)
                    | Expr::Macro(_)
                    | Expr::Tuple(_)
                    | Expr::Array(_)
                    | Expr::Block(_)
            );
            if atomic {
                e.into_token_stream()
            } else {
                let ts = e.into_token_stream();
                quote::quote!((#ts))
            }
        }
        "ident" => input.call(syn::Ident::parse_any)?.into_token_stream(),
        "ty" => input.parse::<syn::Type>()?.into_token_stream(),
        "pat" => syn::Pat::parse_multi_with_leading_vert(input)?.into_token_stream(),
        "pat_param" => syn::Pat::parse_single(input)?.into_token_stream(),
        "literal" => {
            let minus: Option<syn::Token![-]> = input.parse()?;
            let lit: syn::Lit = input.parse()?;
            quote::quote!(#minus #lit)
        }
        "block" => input.parse::<syn::Block>()?.into_token_stream(),
        "stmt" => input.parse::<Stmt>()?.into_token_stream(),
        "path" => input.parse::<syn::Path>()?.into_token_stream(),
        "lifetime" => input.parse::<syn::Lifetime>()?.into_token_stream(),
        "vis" => input.parse::<syn::Visibility>()?.into_token_stream(),
        "item" => input.parse::<Item>()?.into_token_stream(),
        "meta" => input.parse::<syn::Meta>()?.into_token_stream(),
        _ => input.parse::<TokenTree>()?.into_token_stream(),
    })
}

// ----- Transcription -----

fn transcribe(body: &[Transcriber], env: &Env) -> TokenStream {
    let mut out = TokenStream::new();
    for t in body {
        match t {
            Transcriber::Token(tok) => out.extend([tok.clone()]),
            Transcriber::Group(d, inner) => {
                out.extend([TokenTree::Group(Group::new(*d, transcribe(inner, env)))]);
            }
            Transcriber::Var(name) => match env.get(name) {
                Some(Bound::Leaf(ts)) => out.extend(ts.clone()),
                // Not a variable of the rule (`$x` in a nested `macro_rules!`): as written.
                _ => {
                    out.extend(quote::quote!($));
                    out.extend([TokenTree::Ident(proc_macro2::Ident::new(
                        name,
                        Span::call_site(),
                    ))]);
                }
            },
            Transcriber::Repeat(inner, sep) => {
                let vars: Vec<String> = transcriber_vars(inner)
                    .into_iter()
                    .filter(|v| matches!(env.get(v), Some(Bound::Seq(_))))
                    .collect();
                let count = vars
                    .iter()
                    .filter_map(|v| match env.get(v) {
                        Some(Bound::Seq(s)) => Some(s.len()),
                        _ => None,
                    })
                    .max()
                    .unwrap_or(0);
                for i in 0..count {
                    if i > 0 {
                        if let Some(sep) = sep {
                            out.extend([sep.clone()]);
                        }
                    }
                    let mut inner_env = env.clone();
                    for v in &vars {
                        if let Some(Bound::Seq(s)) = env.get(v) {
                            if let Some(b) = s.get(i) {
                                inner_env.insert(v.clone(), b.clone());
                            }
                        }
                    }
                    out.extend(transcribe(inner, &inner_env));
                }
            }
        }
    }
    out
}

fn transcriber_vars(ts: &[Transcriber]) -> Vec<String> {
    let mut out = Vec::new();
    for t in ts {
        match t {
            Transcriber::Var(n) => out.push(n.clone()),
            Transcriber::Group(_, inner) | Transcriber::Repeat(inner, _) => {
                out.extend(transcriber_vars(inner));
            }
            Transcriber::Token(_) => {}
        }
    }
    out
}

/// The expansion placed where the macro is used, for diagnostics and comments.
fn respan(ts: TokenStream, span: Span) -> TokenStream {
    ts.into_iter()
        .map(|t| match t {
            TokenTree::Group(g) => {
                let mut ng = Group::new(g.delimiter(), respan(g.stream(), span));
                ng.set_span(span);
                TokenTree::Group(ng)
            }
            mut t => {
                t.set_span(span);
                t
            }
        })
        .collect()
}
