//! Identifiers: Rust `snake_case` becomes Swift `lowerCamelCase`, keywords are escaped.

/// Swift keywords that cannot be used as plain identifiers.
const KEYWORDS: &[&str] = &[
    "associatedtype",
    "class",
    "deinit",
    "enum",
    "extension",
    "fileprivate",
    "func",
    "import",
    "init",
    "inout",
    "internal",
    "let",
    "open",
    "operator",
    "private",
    "precedencegroup",
    "protocol",
    "public",
    "rethrows",
    "static",
    "struct",
    "subscript",
    "typealias",
    "var",
    "break",
    "case",
    "catch",
    "continue",
    "default",
    "defer",
    "do",
    "else",
    "fallthrough",
    "for",
    "guard",
    "if",
    "in",
    "repeat",
    "return",
    "throw",
    "switch",
    "where",
    "while",
    "Any",
    "as",
    "false",
    "is",
    "nil",
    "super",
    "throws",
    "true",
    "try",
    "await",
    "async",
    "some",
    "any",
];

/// Global functions of the Swift standard library a constant must not shadow.
const GLOBALS: &[&str] = &[
    "abs",
    "max",
    "min",
    "print",
    "zip",
    "stride",
    "sequence",
    "repeatElement",
    "swap",
    "type",
    "assert",
    "precondition",
    "fatalError",
    "debugPrint",
    "readLine",
    "dump",
];

/// Whether a local of this Swift name hides a global function (`max`, `abs`, `zip`…).
pub fn shadows_global(swift: &str) -> bool {
    GLOBALS.contains(&swift)
}

/// `my_value` → `myValue`, `MAX_SIZE` → `maxSize`, `r#type` → `` `type` ``.
pub fn lower_camel(name: &str) -> String {
    let raw = name.strip_prefix("r#").unwrap_or(name);
    let converted = camel(raw, false);
    escape(&converted)
}

/// Like [`lower_camel`] for a global constant: keeps the original name when the converted one
/// would shadow a Swift global function (`MAX` stays `MAX` instead of `max`).
pub fn constant(name: &str) -> String {
    let raw = name.strip_prefix("r#").unwrap_or(name);
    let converted = camel(raw, false);
    if GLOBALS.contains(&converted.as_str()) {
        raw.to_string()
    } else {
        escape(&converted)
    }
}

/// `my_type` or `MyType` → `MyType`.
pub fn upper_camel(name: &str) -> String {
    let raw = name.strip_prefix("r#").unwrap_or(name);
    if raw.contains('_') || raw.chars().all(|c| !c.is_ascii_uppercase()) {
        camel(raw, true)
    } else {
        raw.to_string()
    }
}

/// An enum variant (`LargeTitle`, `Red`) → its case name (`largeTitle`, `red`).
pub fn case_name(name: &str) -> String {
    lower_camel(name)
}

/// Escapes a Swift keyword with backticks.
pub fn escape(name: &str) -> String {
    if KEYWORDS.contains(&name) {
        format!("`{name}`")
    } else {
        name.to_string()
    }
}

fn camel(raw: &str, upper_first: bool) -> String {
    // Leading underscores are kept (`_unused`).
    let trimmed = raw.trim_start_matches('_');
    let prefix = &raw[..raw.len() - trimmed.len()];
    if trimmed.is_empty() {
        return raw.to_string();
    }
    let all_caps = trimmed.chars().all(|c| !c.is_ascii_lowercase());
    let mut out = String::with_capacity(raw.len());
    out.push_str(prefix);
    let mut first_word = true;
    for word in trimmed.split('_').filter(|w| !w.is_empty()) {
        let word = if all_caps {
            word.to_ascii_lowercase()
        } else {
            word.to_string()
        };
        let mut chars = word.chars();
        let Some(first) = chars.next() else { continue };
        if first_word && !upper_first {
            // `LargeTitle` → `largeTitle`, `URLPath` → `urlPath`, `X` → `x`.
            let upper_run = word.chars().take_while(|c| c.is_ascii_uppercase()).count();
            if upper_run > 1 && upper_run < word.len() {
                let (head, tail) = word.split_at(upper_run - 1);
                out.push_str(&head.to_ascii_lowercase());
                out.push_str(tail);
            } else if upper_run == word.len() {
                out.push_str(&word.to_ascii_lowercase());
            } else {
                out.extend(first.to_lowercase());
                out.push_str(chars.as_str());
            }
        } else {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
        first_word = false;
    }
    if trimmed.ends_with('_') {
        out.push('_');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_names() {
        assert_eq!(lower_camel("my_value"), "myValue");
        assert_eq!(lower_camel("x"), "x");
        assert_eq!(lower_camel("MAX_SIZE"), "maxSize");
        assert_eq!(lower_camel("LargeTitle"), "largeTitle");
        assert_eq!(lower_camel("URLPath"), "urlPath");
        assert_eq!(lower_camel("IO"), "io");
        assert_eq!(lower_camel("_unused"), "_unused");
        assert_eq!(lower_camel("default"), "`default`");
        assert_eq!(lower_camel("r#type"), "type");
        assert_eq!(lower_camel("is_ok2"), "isOk2");
        assert_eq!(upper_camel("my_type"), "MyType");
        assert_eq!(upper_camel("MyType"), "MyType");
        assert_eq!(constant("MAX"), "MAX");
        assert_eq!(constant("MAX_LEN"), "maxLen");
        assert_eq!(case_name("BorderedProminent"), "borderedProminent");
    }
}
