//! Ordinary comments (`//`, `/* */`) are not part of the syntax tree: they are read from the
//! source text and put back before the items, fields, statements and match arms they precede.

#[derive(Clone, Debug)]
pub struct Comment {
    pub line: usize,
    /// The comment as written, `//` or `/* */` included.
    pub text: String,
    /// Code precedes it on its line.
    pub trailing: bool,
    taken: bool,
}

#[derive(Default)]
pub struct Comments {
    list: Vec<Comment>,
}

impl Comments {
    pub fn parse(src: &str) -> Comments {
        let mut list = Vec::new();
        let bytes: Vec<char> = src.chars().collect();
        let mut i = 0;
        let mut line = 1;
        let mut code_on_line = false;
        while i < bytes.len() {
            let c = bytes[i];
            let next = bytes.get(i + 1).copied();
            match c {
                '\n' => {
                    line += 1;
                    code_on_line = false;
                    i += 1;
                }
                '/' if next == Some('/') => {
                    let start = i;
                    while i < bytes.len() && bytes[i] != '\n' {
                        i += 1;
                    }
                    let text: String = bytes[start..i].iter().collect();
                    let doc = (text.starts_with("///") && !text.starts_with("////"))
                        || text.starts_with("//!");
                    if !doc {
                        list.push(Comment {
                            line,
                            text: text.trim_end().to_string(),
                            trailing: code_on_line,
                            taken: false,
                        });
                    }
                }
                '/' if next == Some('*') => {
                    let start = i;
                    let start_line = line;
                    let mut depth = 0;
                    while i < bytes.len() {
                        if bytes[i] == '/' && bytes.get(i + 1) == Some(&'*') {
                            depth += 1;
                            i += 2;
                        } else if bytes[i] == '*' && bytes.get(i + 1) == Some(&'/') {
                            depth -= 1;
                            i += 2;
                            if depth == 0 {
                                break;
                            }
                        } else {
                            if bytes[i] == '\n' {
                                line += 1;
                            }
                            i += 1;
                        }
                    }
                    let text: String = bytes[start..i.min(bytes.len())].iter().collect();
                    let doc = (text.starts_with("/**") && !text.starts_with("/***"))
                        || text.starts_with("/*!");
                    if !doc && text != "/**/" {
                        list.push(Comment {
                            line: start_line,
                            text,
                            trailing: code_on_line,
                            taken: false,
                        });
                    }
                    code_on_line = true;
                }
                'r' if matches!(next, Some('"') | Some('#'))
                    && (i == 0 || !is_ident(bytes[i - 1])) =>
                {
                    // Raw string r"…" / r#"…"#.
                    let mut j = i + 1;
                    let mut hashes = 0;
                    while bytes.get(j) == Some(&'#') {
                        hashes += 1;
                        j += 1;
                    }
                    if bytes.get(j) != Some(&'"') {
                        i += 1;
                        code_on_line = true;
                        continue;
                    }
                    j += 1;
                    loop {
                        match bytes.get(j) {
                            None => break,
                            Some('\n') => {
                                line += 1;
                                j += 1;
                            }
                            Some('"') => {
                                let closing = (1..=hashes).all(|k| bytes.get(j + k) == Some(&'#'));
                                j += 1;
                                if closing {
                                    j += hashes;
                                    break;
                                }
                            }
                            Some(_) => j += 1,
                        }
                    }
                    i = j;
                    code_on_line = true;
                }
                '"' => {
                    i += 1;
                    while i < bytes.len() {
                        match bytes[i] {
                            '\\' => i += 2,
                            '"' => {
                                i += 1;
                                break;
                            }
                            '\n' => {
                                line += 1;
                                i += 1;
                            }
                            _ => i += 1,
                        }
                    }
                    code_on_line = true;
                }
                '\'' => {
                    // A char literal, or a lifetime ('a).
                    if next == Some('\\') {
                        i += 2;
                        while i < bytes.len() && bytes[i] != '\'' {
                            i += 1;
                        }
                        i += 1;
                    } else if bytes.get(i + 2) == Some(&'\'') {
                        i += 3;
                    } else {
                        i += 1;
                    }
                    code_on_line = true;
                }
                c => {
                    if !c.is_whitespace() {
                        code_on_line = true;
                    }
                    i += 1;
                }
            }
        }
        Comments { list }
    }

    /// The comments on their own lines strictly between lines `after` and `before`.
    pub fn take_between(&mut self, after: usize, before: usize) -> Vec<String> {
        self.take_between_lines(after, before)
            .into_iter()
            .map(|(_, text)| text)
            .collect()
    }

    /// Like [`take_between`](Self::take_between), with the line of each comment.
    pub fn take_between_lines(&mut self, after: usize, before: usize) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        for c in &mut self.list {
            if !c.taken && c.line > after && c.line < before && !(c.trailing && c.line == after) {
                c.taken = true;
                out.push((c.line, c.text.clone()));
            }
        }
        out
    }

    /// The comment at the end of line `line`, after code.
    pub fn take_trailing(&mut self, line: usize) -> Option<String> {
        let c = self
            .list
            .iter_mut()
            .find(|c| !c.taken && c.trailing && c.line == line)?;
        c.taken = true;
        Some(c.text.clone())
    }
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_comments_outside_strings() {
        let src = "// one\nfn a() { let s = \"// no\"; // two\n    let c = '/'; /* three */\n}\n/// doc\nfn b<'a>() {}";
        let mut c = Comments::parse(src);
        assert_eq!(c.take_between(0, 2), vec!["// one"]);
        assert_eq!(c.take_trailing(2).as_deref(), Some("// two"));
        assert_eq!(c.take_trailing(3).as_deref(), Some("/* three */"));
        assert!(c.take_between(0, 100).is_empty());
    }
}
