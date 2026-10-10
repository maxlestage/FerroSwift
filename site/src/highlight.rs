//! A small syntax highlighter for Rust and Swift (plus shell and TOML for the docs), shared
//! by the server (pages, docs, comparisons) and the browser (the playground). It writes
//! escaped HTML with `<span class="h-…">` tokens, coloured by app.css:
//!
//! | class  | token                                                     |
//! |--------|-----------------------------------------------------------|
//! | `h-k`  | keyword                                                   |
//! | `h-l`  | literal keyword: `true`, `self`, `nil`…                   |
//! | `h-t`  | type: primitive or `Capitalized` identifier               |
//! | `h-f`  | function or method name before `(`                        |
//! | `h-m`  | Rust macro: `println!`, `format!`                         |
//! | `h-a`  | attribute, property wrapper, directive: `#[state]`, `@State`, `#Preview` |
//! | `h-s`  | string or character                                       |
//! | `h-e`  | escape, interpolation `\(…)` or placeholder `{…}` in a string |
//! | `h-n`  | number                                                    |
//! | `h-c`  | comment (`h-cd` for doc comments)                         |
//! | `h-lt` | Rust lifetime or label                                    |

/// The languages the highlighter knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syntax {
    Rust,
    Swift,
    Shell,
    Toml,
    Plain,
}

impl Syntax {
    /// The syntax of a Markdown code fence (`rust`, `swift`, `sh`…).
    pub fn from_fence(info: &str) -> Syntax {
        match info.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => Syntax::Rust,
            "swift" => Syntax::Swift,
            "sh" | "bash" | "shell" | "console" | "terminal" | "zsh" => Syntax::Shell,
            "toml" => Syntax::Toml,
            _ => Syntax::Plain,
        }
    }

    /// The label shown above a code block ("" for plain text).
    pub fn label(self) -> &'static str {
        match self {
            Syntax::Rust => "rust",
            Syntax::Swift => "swift",
            Syntax::Shell => "terminal",
            Syntax::Toml => "toml",
            Syntax::Plain => "",
        }
    }
}

/// Escapes `&`, `<`, `>`, `"` and `'` into `out`.
pub fn escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
}

/// `s` with `&`, `<`, `>`, `"` and `'` escaped.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    escape_into(&mut out, s);
    out
}

/// `code` as escaped HTML with highlighted tokens.
pub fn highlight(code: &str, syntax: Syntax) -> String {
    let mut out = String::with_capacity(code.len() * 2);
    match syntax {
        Syntax::Rust | Syntax::Swift => code_into(&mut out, code, syntax),
        Syntax::Shell => shell_into(&mut out, code),
        Syntax::Toml => toml_into(&mut out, code),
        Syntax::Plain => escape_into(&mut out, code),
    }
    out
}

/// The number of lines of `code` (an empty text has one line).
pub fn line_count(code: &str) -> usize {
    code.split('\n').count()
}

fn span(out: &mut String, class: &str, text: &str) {
    if text.is_empty() {
        return;
    }
    out.push_str("<span class=\"h-");
    out.push_str(class);
    out.push_str("\">");
    escape_into(out, text);
    out.push_str("</span>");
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "static", "struct", "super", "trait", "type", "unsafe", "use", "where", "while",
    "yield",
];
const RUST_LITERALS: &[&str] = &["true", "false", "self", "Self"];
const RUST_PRIMITIVES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str",
];
const SWIFT_KEYWORDS: &[&str] = &[
    "actor",
    "any",
    "as",
    "associatedtype",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "continue",
    "convenience",
    "default",
    "defer",
    "deinit",
    "didSet",
    "do",
    "else",
    "enum",
    "extension",
    "fallthrough",
    "fileprivate",
    "final",
    "for",
    "func",
    "get",
    "guard",
    "if",
    "import",
    "in",
    "indirect",
    "init",
    "inout",
    "internal",
    "is",
    "lazy",
    "let",
    "mutating",
    "nonmutating",
    "open",
    "operator",
    "override",
    "private",
    "protocol",
    "public",
    "repeat",
    "required",
    "rethrows",
    "return",
    "set",
    "some",
    "static",
    "struct",
    "subscript",
    "switch",
    "throw",
    "throws",
    "try",
    "typealias",
    "unowned",
    "var",
    "weak",
    "where",
    "while",
    "willSet",
];
const SWIFT_LITERALS: &[&str] = &["true", "false", "nil", "self", "Self", "super"];

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// The end of the identifier starting at `i`.
fn ident_end(src: &str, i: usize) -> usize {
    src[i..]
        .char_indices()
        .find(|&(_, c)| !is_ident_char(c))
        .map_or(src.len(), |(n, _)| i + n)
}

fn char_at(src: &str, i: usize) -> Option<char> {
    src.get(i..).and_then(|s| s.chars().next())
}

/// Rust or Swift source.
fn code_into(out: &mut String, src: &str, syntax: Syntax) {
    let rust = syntax == Syntax::Rust;
    let mut i = 0;
    let mut plain_start = 0;
    // Plain text is copied in runs, between tokens.
    let flush = |out: &mut String, from: usize, to: usize| {
        if to > from {
            escape_into(out, &src[from..to]);
        }
    };
    while i < src.len() {
        let rest = &src[i..];
        let c = rest.chars().next().unwrap_or(' ');
        let token: Option<(usize, &str)> = if rest.starts_with("//") {
            let end = rest.find('\n').map_or(src.len(), |n| i + n);
            let doc =
                (rest.starts_with("///") && !rest.starts_with("////")) || rest.starts_with("//!");
            Some((end, if doc { "cd" } else { "c" }))
        } else if rest.starts_with("/*") {
            Some((block_comment_end(src, i), "c"))
        } else if c == '"' || (!rust && c == '#' && rest[1..].starts_with('"')) {
            let end = if rust {
                rust_string_end(src, i)
            } else {
                swift_string_end(src, i)
            };
            flush(out, plain_start, i);
            if rust {
                rust_string_body(out, &src[i..end]);
            } else {
                swift_string_body(out, &src[i..end]);
            }
            i = end;
            plain_start = i;
            continue;
        } else if rust && c == '\'' {
            rust_quote(src, i)
        } else if rust && (rest.starts_with("#[") || rest.starts_with("#![")) {
            Some((bracket_end(src, i), "a"))
        } else if !rust && (c == '@' || c == '#') && char_at(src, i + 1).is_some_and(is_ident_start)
        {
            Some((ident_end(src, i + 1), "a"))
        } else if c.is_ascii_digit() {
            Some((number_end(src, i, rust), "n"))
        } else if let Some((end, raw)) = rust.then(|| rust_prefixed_string(src, i)).flatten() {
            flush(out, plain_start, i);
            if raw {
                span(out, "s", &src[i..end]);
            } else {
                rust_string_body(out, &src[i..end]);
            }
            i = end;
            plain_start = i;
            continue;
        } else if is_ident_start(c) {
            let end = ident_end(src, i);
            let word = &src[i..end];
            let next = char_at(src, end);
            let class = if rust
                && next == Some('!')
                && char_at(src, end + 1) != Some('=')
                && !RUST_KEYWORDS.contains(&word)
            {
                flush(out, plain_start, i);
                span(out, "m", &src[i..end + 1]);
                i = end + 1;
                plain_start = i;
                continue;
            } else if !rust
                && next == Some(':')
                && char_at(src, end + 1) != Some(':')
                && word != "default"
                && SWIFT_KEYWORDS.contains(&word)
            {
                // A Swift argument label, even a keyword: `in:`, `for:`.
                None
            } else if (rust && RUST_KEYWORDS.contains(&word))
                || (!rust && SWIFT_KEYWORDS.contains(&word))
            {
                Some("k")
            } else if (rust && RUST_LITERALS.contains(&word))
                || (!rust && SWIFT_LITERALS.contains(&word))
            {
                Some("l")
            } else if (rust && RUST_PRIMITIVES.contains(&word))
                || word.chars().next().is_some_and(char::is_uppercase)
            {
                Some("t")
            } else if next == Some('(') || (rust && src[end..].starts_with("::<")) {
                Some("f")
            } else {
                None
            };
            match class {
                Some(class) => Some((end, class)),
                None => {
                    i = end;
                    continue;
                }
            }
        } else {
            None
        };
        match token {
            Some((end, class)) => {
                flush(out, plain_start, i);
                span(out, class, &src[i..end]);
                i = end;
                plain_start = i;
            }
            None => i += c.len_utf8(),
        }
    }
    flush(out, plain_start, src.len());
}

/// The end of a (nested) block comment starting at `i`.
fn block_comment_end(src: &str, i: usize) -> usize {
    let bytes = src.as_bytes();
    let mut depth = 0;
    let mut j = i;
    while j + 1 < bytes.len() {
        match (bytes[j], bytes[j + 1]) {
            (b'/', b'*') => {
                depth += 1;
                j += 2;
            }
            (b'*', b'/') => {
                depth -= 1;
                j += 2;
                if depth == 0 {
                    return j;
                }
            }
            _ => j += 1,
        }
    }
    src.len()
}

/// The end of a Rust attribute `#[…]` starting at `i` (strings inside are skipped).
fn bracket_end(src: &str, i: usize) -> usize {
    let mut depth = 0;
    let mut j = i;
    while j < src.len() {
        let c = char_at(src, j).unwrap_or(' ');
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            '"' => {
                j = rust_string_end(src, j);
                continue;
            }
            '\n' if depth == 0 => return j,
            _ => {}
        }
        j += c.len_utf8();
    }
    src.len()
}

/// The end of a number starting at `i`: `42`, `1_000`, `0xFF`, `16.0`, `1e-9`, `2u32`.
fn number_end(src: &str, i: usize, rust: bool) -> usize {
    let bytes = src.as_bytes();
    let mut j = i;
    let digit = |b: u8| b.is_ascii_digit() || b == b'_';
    if bytes[j] == b'0' && matches!(bytes.get(j + 1), Some(b'x' | b'o' | b'b')) {
        j += 2;
        while j < bytes.len() && (bytes[j].is_ascii_hexdigit() || bytes[j] == b'_') {
            j += 1;
        }
    } else {
        while j < bytes.len() && digit(bytes[j]) {
            j += 1;
        }
        if j + 1 < bytes.len() && bytes[j] == b'.' && bytes[j + 1].is_ascii_digit() {
            j += 1;
            while j < bytes.len() && digit(bytes[j]) {
                j += 1;
            }
        }
        if j < bytes.len() && (bytes[j] == b'e' || bytes[j] == b'E') {
            let sign = matches!(bytes.get(j + 1), Some(b'+' | b'-'));
            let k = if sign { j + 2 } else { j + 1 };
            if bytes.get(k).is_some_and(u8::is_ascii_digit) {
                j = k;
                while j < bytes.len() && digit(bytes[j]) {
                    j += 1;
                }
            }
        }
    }
    if rust {
        // Suffix: `2u32`, `1.5f64`.
        while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
            j += 1;
        }
    }
    j
}

/// Rust `'a'`, `'\n'` (characters) or `'a` (lifetimes and labels).
fn rust_quote(src: &str, i: usize) -> Option<(usize, &'static str)> {
    let after = i + 1;
    let first = char_at(src, after)?;
    if first == '\\' {
        // An escape: up to the closing quote, close by.
        let close = src[after + 1..]
            .char_indices()
            .take(12)
            .skip(1)
            .find(|&(_, c)| c == '\'')
            .map(|(n, _)| after + 1 + n + 1)?;
        return Some((close, "s"));
    }
    let next = after + first.len_utf8();
    if char_at(src, next) == Some('\'') {
        return Some((next + 1, "s"));
    }
    if is_ident_start(first) {
        return Some((ident_end(src, after), "lt"));
    }
    None
}

/// `r"…"`, `r#"…"#`, `b"…"`, `br"…"`, `c"…"` starting at `i`: their end, and whether the
/// string is raw (no escapes).
fn rust_prefixed_string(src: &str, i: usize) -> Option<(usize, bool)> {
    let rest = &src[i..];
    // Only at the start of a word.
    if i > 0 && char_at(src, i - 1).is_some_and(is_ident_char) {
        return None;
    }
    let prefix = ["br", "cr", "r", "b", "c"]
        .into_iter()
        .find(|p| rest.starts_with(p))?;
    let after = &rest[prefix.len()..];
    if prefix.ends_with('r') {
        let hashes = after.chars().take_while(|&c| c == '#').count();
        if !after[hashes..].starts_with('"') {
            return None;
        }
        let close = format!("\"{}", "#".repeat(hashes));
        let body = i + prefix.len() + hashes + 1;
        let end = src[body..]
            .find(&close)
            .map_or(src.len(), |n| body + n + close.len());
        return Some((end, true));
    }
    if after.starts_with('"') {
        return Some((rust_string_end(src, i + prefix.len()), false));
    }
    None
}

/// The end of the Rust string starting with `"` at `i`.
fn rust_string_end(src: &str, i: usize) -> usize {
    let mut chars = src[i + 1..].char_indices();
    while let Some((n, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '"' => return i + 1 + n + 1,
            _ => {}
        }
    }
    src.len()
}

/// A Rust string with its escapes and `format!` placeholders (`{}`, `{name:?}`) marked.
fn rust_string_body(out: &mut String, s: &str) {
    out.push_str("<span class=\"h-s\">");
    let mut i = 0;
    let mut start = 0;
    while i < s.len() {
        let c = char_at(s, i).unwrap_or(' ');
        let end = match c {
            '\\' if i + 1 < s.len() => {
                let next = char_at(s, i + 1).unwrap_or(' ');
                if next == 'u' && s[i + 2..].starts_with('{') {
                    s[i..].find('}').map(|n| i + n + 1)
                } else {
                    Some(i + 1 + next.len_utf8())
                }
            }
            '{' if s[i..].starts_with("{{") => {
                i += 2;
                continue;
            }
            '{' => s[i + 1..]
                .char_indices()
                .find(|&(_, c)| !(is_ident_char(c) || ".:?#<>^+-$".contains(c)))
                .filter(|&(_, c)| c == '}')
                .map(|(n, _)| i + 1 + n + 1),
            _ => None,
        };
        match end {
            Some(end) => {
                escape_into(out, &s[start..i]);
                span(out, "e", &s[i..end]);
                i = end;
                start = i;
            }
            None => i += c.len_utf8(),
        }
    }
    escape_into(out, &s[start..]);
    out.push_str("</span>");
}

/// The end of the Swift string starting at `i`: `"…"`, `"""…"""` or `#"…"#`, interpolations
/// `\(…)` included.
fn swift_string_end(src: &str, i: usize) -> usize {
    let hashes = src[i..].chars().take_while(|&c| c == '#').count();
    let open = i + hashes;
    let multi = src[open..].starts_with("\"\"\"");
    let close = format!(
        "{}{}",
        if multi { "\"\"\"" } else { "\"" },
        "#".repeat(hashes)
    );
    let mut j = open + if multi { 3 } else { 1 };
    while j < src.len() {
        let rest = &src[j..];
        if rest.starts_with(&close) {
            return j + close.len();
        }
        let c = rest.chars().next().unwrap_or(' ');
        if c == '\n' && !multi {
            return j;
        }
        if hashes == 0 && c == '\\' {
            if rest[1..].starts_with('(') {
                j = interpolation_end(src, j + 2);
                continue;
            }
            j += 1 + char_at(src, j + 1).map_or(0, char::len_utf8);
            continue;
        }
        j += c.len_utf8();
    }
    src.len()
}

/// The end (after the `)`) of the Swift interpolation whose content starts at `i`.
fn interpolation_end(src: &str, i: usize) -> usize {
    let mut depth = 1;
    let mut j = i;
    while j < src.len() {
        let c = char_at(src, j).unwrap_or(' ');
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            '"' => {
                j = swift_string_end(src, j);
                continue;
            }
            '\n' => return j,
            _ => {}
        }
        j += c.len_utf8();
    }
    src.len()
}

/// A Swift string: escapes marked, interpolated code highlighted.
fn swift_string_body(out: &mut String, s: &str) {
    if s.starts_with('#') {
        span(out, "s", s);
        return;
    }
    out.push_str("<span class=\"h-s\">");
    let mut i = 0;
    let mut start = 0;
    while i < s.len() {
        let c = char_at(s, i).unwrap_or(' ');
        if c == '\\' && i + 1 < s.len() {
            escape_into(out, &s[start..i]);
            if s[i + 1..].starts_with('(') {
                let end = interpolation_end(s, i + 2);
                let closed = s[..end].ends_with(')') && end > i + 2;
                let inner_end = if closed { end - 1 } else { end };
                span(out, "e", "\\(");
                out.push_str("</span>");
                code_into(out, &s[i + 2..inner_end], Syntax::Swift);
                out.push_str("<span class=\"h-s\">");
                if closed {
                    span(out, "e", ")");
                }
                i = end;
            } else {
                let next = char_at(s, i + 1).map_or(0, char::len_utf8);
                span(out, "e", &s[i..i + 1 + next]);
                i += 1 + next;
            }
            start = i;
            continue;
        }
        i += c.len_utf8();
    }
    escape_into(out, &s[start..]);
    out.push_str("</span>");
}

/// Shell commands: comments, the command name, flags and strings.
fn shell_into(out: &mut String, src: &str) {
    for (n, line) in src.split('\n').enumerate() {
        if n > 0 {
            out.push('\n');
        }
        let mut command = true;
        let mut i = 0;
        while i < line.len() {
            let rest = &line[i..];
            let c = rest.chars().next().unwrap_or(' ');
            if c == '#' && (i == 0 || line[..i].ends_with(' ')) {
                span(out, "c", rest);
                break;
            }
            if c == '"' || c == '\'' {
                let end = rest[1..].find(c).map_or(line.len(), |e| i + e + 2);
                span(out, "s", &line[i..end]);
                i = end;
                command = false;
                continue;
            }
            if c.is_whitespace() || "|&;".contains(c) {
                if "|&;".contains(c) {
                    command = true;
                }
                escape_into(out, &line[i..i + c.len_utf8()]);
                i += c.len_utf8();
                continue;
            }
            let end = rest
                .find(|c: char| c.is_whitespace() || "|&;".contains(c))
                .map_or(line.len(), |e| i + e);
            let word = &line[i..end];
            if word == "$" && command {
                span(out, "c", word);
            } else if command {
                span(out, "f", word);
                command = false;
            } else if word.starts_with('-') {
                span(out, "a", word);
            } else {
                escape_into(out, word);
            }
            i = end;
        }
    }
}

/// TOML: sections, keys, strings, numbers and booleans.
fn toml_into(out: &mut String, src: &str) {
    for (n, line) in src.split('\n').enumerate() {
        if n > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            span(out, "c", line);
        } else if trimmed.starts_with('[') {
            span(out, "t", line);
        } else if let Some((key, value)) = line.split_once('=') {
            span(out, "f", key);
            out.push('=');
            toml_value(out, value);
        } else {
            escape_into(out, line);
        }
    }
}

fn toml_value(out: &mut String, value: &str) {
    let mut i = 0;
    while i < value.len() {
        let rest = &value[i..];
        let c = rest.chars().next().unwrap_or(' ');
        if c == '"' {
            let end = rest[1..].find('"').map_or(value.len(), |e| i + e + 2);
            span(out, "s", &value[i..end]);
            i = end;
        } else if c == '#' {
            span(out, "c", rest);
            break;
        } else if c.is_ascii_digit() {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_'))
                .map_or(value.len(), |e| i + e);
            span(out, "n", &value[i..end]);
            i = end;
        } else if rest.starts_with("true") || rest.starts_with("false") {
            let len = if rest.starts_with("true") { 4 } else { 5 };
            span(out, "l", &rest[..len]);
            i += len;
        } else {
            escape_into(out, &value[i..i + c.len_utf8()]);
            i += c.len_utf8();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html_everywhere() {
        let html = highlight("let s = \"<b>&'\"; // <i>\nlet c = '<';", Syntax::Rust);
        assert!(!html.contains("<b>") && !html.contains("<i>"));
        assert!(html.contains("&lt;b&gt;&amp;&#39;"), "{html}");
        assert!(
            html.contains("<span class=\"h-c\">// &lt;i&gt;</span>"),
            "{html}"
        );
        assert!(
            html.contains("<span class=\"h-s\">&#39;&lt;&#39;</span>"),
            "{html}"
        );
        for syntax in [Syntax::Swift, Syntax::Shell, Syntax::Toml, Syntax::Plain] {
            let html = highlight("a < b && \"c\" > 'd' # <script>", syntax);
            assert!(!html.contains("<script>"), "{syntax:?}: {html}");
            assert!(!html.contains(" < "), "{syntax:?}: {html}");
        }
        assert_eq!(
            escape(r#"<a href="x">'&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }

    #[test]
    fn highlights_rust_tokens() {
        let html = highlight(
            "/// Doc\n#[derive(View)]\npub struct A<'a> { n: i64 }\nfn f() { println!(\"{} {x:?}\\n\", 1.5e3, r#\"raw\"#); let c = 'x'; }",
            Syntax::Rust,
        );
        assert!(
            html.contains("<span class=\"h-cd\">/// Doc</span>"),
            "{html}"
        );
        assert!(html.contains("<span class=\"h-a\">#[derive(View)]</span>"));
        assert!(html.contains("<span class=\"h-k\">pub</span> <span class=\"h-k\">struct</span> <span class=\"h-t\">A</span>"));
        assert!(html.contains("<span class=\"h-lt\">&#39;a</span>"));
        assert!(html.contains("<span class=\"h-t\">i64</span>"));
        assert!(html.contains("<span class=\"h-m\">println!</span>"));
        assert!(html.contains("<span class=\"h-e\">{}</span> <span class=\"h-e\">{x:?}</span><span class=\"h-e\">\\n</span>"), "{html}");
        assert!(html.contains("<span class=\"h-n\">1.5e3</span>"));
        assert!(html.contains("<span class=\"h-s\">r#&quot;raw&quot;#</span>"));
        assert!(html.contains("<span class=\"h-s\">&#39;x&#39;</span>"));
        assert!(html.contains("<span class=\"h-f\">f</span>()"));
        // `!=` is not a macro.
        let html = highlight("if a != b {}", Syntax::Rust);
        assert!(!html.contains("h-m"), "{html}");
    }

    #[test]
    fn highlights_swift_tokens_and_interpolations() {
        let html = highlight(
            "@State var count = 0\nText(\"Count: \\(count + items.joined(separator: \", \"))\")\n#Preview { }",
            Syntax::Swift,
        );
        assert!(
            html.contains("<span class=\"h-a\">@State</span> <span class=\"h-k\">var</span>"),
            "{html}"
        );
        assert!(html.contains("<span class=\"h-t\">Text</span>("));
        assert!(html.contains("<span class=\"h-e\">\\(</span></span>count + items.<span class=\"h-f\">joined</span>(separator: <span class=\"h-s\">&quot;, &quot;</span>)<span class=\"h-s\"><span class=\"h-e\">)</span>&quot;</span>"), "{html}");
        assert!(html.contains("<span class=\"h-a\">#Preview</span>"));
        // Argument labels are not keywords, `default:` is.
        let html = highlight(
            "Slider(value: $v, in: 0...1)\ndefault: break\nstruct A: View",
            Syntax::Swift,
        );
        assert!(html.contains("$v, in: <span"), "{html}");
        assert!(
            html.contains("<span class=\"h-t\">A</span>: <span class=\"h-t\">View</span>"),
            "{html}"
        );
        assert!(
            html.contains("<span class=\"h-k\">default</span>:"),
            "{html}"
        );
        // Unterminated strings stop at the end of the line.
        let html = highlight("let s = \"abc\nlet t = 1", Syntax::Swift);
        assert!(html.contains("<span class=\"h-k\">let</span> t"), "{html}");
    }

    #[test]
    fn highlights_shell_and_toml() {
        let sh = highlight(
            "$ cargo install --git https://x ferroswift-cli # comment",
            Syntax::Shell,
        );
        assert!(sh.contains("<span class=\"h-f\">cargo</span>"), "{sh}");
        assert!(sh.contains("<span class=\"h-a\">--git</span>"));
        assert!(sh.contains("<span class=\"h-c\"># comment</span>"));
        let toml = highlight("[dependencies]\nferroswift = \"0.1\" # x", Syntax::Toml);
        assert!(toml.contains("<span class=\"h-t\">[dependencies]</span>"));
        assert!(toml.contains("<span class=\"h-s\">&quot;0.1&quot;</span>"));
    }

    #[test]
    fn never_loses_text() {
        // Whatever the input, removing the tags and unescaping gives the input back.
        let inputs = [
            "fn main() { let s = \"unterminated",
            "/* nested /* comment */ still */ x",
            "'a 'b' '\\u{1F600}' b\"bytes\" br#\"x\"# c\"y\"",
            "Text(\"a \\(b(\"c\\(d)\")) e\") \"\"\"\nmulti \\(x)\n\"\"\"",
            "0x1F 1_000u32 2.5f64 1e-9 3..4 x.0",
            "é ñ 你好 🦀 @ # ' \\",
        ];
        for input in inputs {
            for syntax in [Syntax::Rust, Syntax::Swift, Syntax::Shell, Syntax::Toml] {
                let html = highlight(input, syntax);
                let mut text = String::new();
                let mut in_tag = false;
                for c in html.chars() {
                    match c {
                        '<' => in_tag = true,
                        '>' if in_tag => in_tag = false,
                        c if !in_tag => text.push(c),
                        _ => {}
                    }
                }
                let text = text
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&quot;", "\"")
                    .replace("&#39;", "'")
                    .replace("&amp;", "&");
                assert_eq!(text, input, "{syntax:?}");
            }
        }
    }
}
