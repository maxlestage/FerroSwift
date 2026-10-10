//! Documentation pages: Markdown with a small front matter, rendered once at startup.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};

use crate::highlight::{Syntax, escape, highlight};

pub struct Doc {
    pub title: String,
    pub description: String,
    /// Position in the menu (front matter `order:`), smallest first.
    pub order: i32,
    pub html: String,
    pub toc: Vec<TocItem>,
}

pub struct TocItem {
    pub depth: u8,
    pub id: String,
    pub text: String,
}

/// The fields between `---` lines at the top of the file, and the rest.
fn front_matter(src: &str) -> (Vec<(&str, &str)>, &str) {
    let Some(rest) = src
        .strip_prefix("---\n")
        .or_else(|| src.strip_prefix("---\r\n"))
    else {
        return (Vec::new(), src);
    };
    let Some(end) = rest.find("\n---") else {
        return (Vec::new(), src);
    };
    let (head, body) = (&rest[..end], &rest[end + 4..]);
    let body = body.trim_start_matches(['\r', '\n']);
    let fields = head
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            Some((
                key.trim(),
                value.trim().trim_matches(|c| c == '"' || c == '\''),
            ))
        })
        .collect();
    (fields, body)
}

/// Heading ids: lowercase ASCII, accents removed, other characters collapsed to `-`.
pub fn slugify(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in text.chars().flat_map(fold_accent) {
        if c.is_ascii_alphanumeric() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(c.to_ascii_lowercase());
        } else {
            dash = true;
        }
    }
    out
}

/// `é` → `e`, `œ` → `oe`: enough for French headings.
fn fold_accent(c: char) -> Vec<char> {
    let plain = match c {
        'à' | 'â' | 'ä' | 'á' | 'À' | 'Â' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' => 'e',
        'î' | 'ï' | 'í' | 'Î' => 'i',
        'ô' | 'ö' | 'ó' | 'Ô' => 'o',
        'ù' | 'û' | 'ü' | 'ú' | 'Û' => 'u',
        'ç' | 'Ç' => 'c',
        'ñ' => 'n',
        'œ' | 'Œ' => return vec!['o', 'e'],
        'æ' | 'Æ' => return vec!['a', 'e'],
        c => c,
    };
    vec![plain]
}

enum Buffer<'a> {
    Heading(u8, Vec<Event<'a>>),
    Quote(Vec<Event<'a>>),
}

/// A highlighted code block with its label and a copy button.
pub fn code_block(code: &str, syntax: Syntax, copy: &str, copied: &str) -> String {
    let label = syntax.label();
    format!(
        concat!(
            r#"<div class="code" data-lang="{label}"><div class="code-bar"><span>{label}</span>"#,
            r#"<button class="copy" type="button" data-done="{copied}">{copy}</button></div>"#,
            r#"<pre><code>{code}</code></pre></div>"#
        ),
        label = escape(label),
        copy = escape(copy),
        copied = escape(copied),
        code = highlight(code.trim_end_matches('\n'), syntax),
    )
}

/// Renders a page: heading anchors and table of contents, highlighted code with a copy
/// button, scrollable tables, callouts (`> **Note:**`, `> **Astuce :**`, `> **Attention :**`…)
/// and external links opening in a new tab.
pub fn render(src: &str, copy: &str, copied: &str) -> Doc {
    let (fields, body) = front_matter(src);
    let field = |name: &str| {
        fields
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
    };
    let mut out: Vec<Event<'_>> = Vec::new();
    let mut stack: Vec<Buffer<'_>> = Vec::new();
    let mut toc = Vec::new();
    let mut seen: Vec<(String, usize)> = Vec::new();
    let mut code: Option<(Syntax, String)> = None;
    let mut external = Vec::new();

    fn emit<'a>(stack: &mut [Buffer<'a>], out: &mut Vec<Event<'a>>, event: Event<'a>) {
        match stack.last_mut() {
            Some(Buffer::Heading(_, events) | Buffer::Quote(events)) => events.push(event),
            None => out.push(event),
        }
    }

    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    for event in Parser::new_ext(body, options) {
        if let Some((syntax, text)) = code.as_mut() {
            match event {
                Event::Text(t) => text.push_str(&t),
                Event::End(TagEnd::CodeBlock) => {
                    let html = code_block(text, *syntax, copy, copied);
                    code = None;
                    emit(&mut stack, &mut out, Event::Html(html.into()));
                }
                _ => {}
            }
            continue;
        }
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let syntax = match kind {
                    CodeBlockKind::Fenced(info) => {
                        Syntax::from_fence(info.split_whitespace().next().unwrap_or(""))
                    }
                    CodeBlockKind::Indented => Syntax::Plain,
                };
                code = Some((syntax, String::new()));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                stack.push(Buffer::Heading(level as u8, Vec::new()))
            }
            Event::End(TagEnd::Heading(_)) => {
                let Some(Buffer::Heading(depth, events)) = stack.pop() else {
                    continue;
                };
                let text: String = events
                    .iter()
                    .filter_map(|e| match e {
                        Event::Text(t) | Event::Code(t) => Some(t.as_ref()),
                        _ => None,
                    })
                    .collect();
                let mut id = slugify(&text);
                if id.is_empty() {
                    id = "section".into();
                }
                match seen.iter_mut().find(|(s, _)| *s == id) {
                    Some((_, n)) => {
                        *n += 1;
                        id = format!("{id}-{n}");
                    }
                    None => seen.push((id.clone(), 0)),
                }
                let mut inner = String::new();
                html::push_html(&mut inner, events.into_iter());
                if depth == 2 || depth == 3 {
                    toc.push(TocItem {
                        depth,
                        id: id.clone(),
                        text,
                    });
                }
                let heading = format!(
                    r##"<h{depth} id="{id}"><a class="anchor" href="#{id}" aria-hidden="true" tabindex="-1">#</a>{inner}</h{depth}>"##
                );
                emit(&mut stack, &mut out, Event::Html(heading.into()));
            }
            Event::Start(Tag::BlockQuote(_)) => stack.push(Buffer::Quote(Vec::new())),
            Event::End(TagEnd::BlockQuote(_)) => {
                let Some(Buffer::Quote(events)) = stack.pop() else {
                    continue;
                };
                let mut inner = String::new();
                html::push_html(&mut inner, events.into_iter());
                let word = inner
                    .strip_prefix("<p><strong>")
                    .map(|rest| {
                        rest.split(|c: char| !c.is_alphabetic())
                            .next()
                            .unwrap_or("")
                            .to_lowercase()
                    })
                    .unwrap_or_default();
                let class = match word.as_str() {
                    "tip" | "astuce" => "tip",
                    "warning" | "attention" => "warning",
                    _ => "note",
                };
                emit(
                    &mut stack,
                    &mut out,
                    Event::Html(
                        format!(r#"<blockquote class="callout {class}">{inner}</blockquote>"#)
                            .into(),
                    ),
                );
            }
            Event::Start(Tag::Table(alignments)) => {
                emit(
                    &mut stack,
                    &mut out,
                    Event::Html(r#"<div class="table">"#.into()),
                );
                emit(&mut stack, &mut out, Event::Start(Tag::Table(alignments)));
            }
            Event::End(TagEnd::Table) => {
                emit(&mut stack, &mut out, Event::End(TagEnd::Table));
                emit(&mut stack, &mut out, Event::Html("</div>".into()));
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let is_external =
                    dest_url.starts_with("http://") || dest_url.starts_with("https://");
                external.push(is_external);
                if is_external {
                    let title = if title.is_empty() {
                        String::new()
                    } else {
                        format!(r#" title="{}""#, escape(&title))
                    };
                    let open = format!(
                        r#"<a href="{}"{title} target="_blank" rel="noopener noreferrer">"#,
                        escape(&dest_url)
                    );
                    emit(&mut stack, &mut out, Event::Html(open.into()));
                } else {
                    emit(
                        &mut stack,
                        &mut out,
                        Event::Start(Tag::Link {
                            link_type,
                            dest_url,
                            title,
                            id,
                        }),
                    );
                }
            }
            Event::End(TagEnd::Link) => {
                let event = if external.pop() == Some(true) {
                    Event::Html(CowStr::from("</a>"))
                } else {
                    Event::End(TagEnd::Link)
                };
                emit(&mut stack, &mut out, event);
            }
            other => emit(&mut stack, &mut out, other),
        }
    }
    let mut html_out = String::with_capacity(body.len() * 2);
    html::push_html(&mut html_out, out.into_iter());
    Doc {
        title: field("title").unwrap_or_default(),
        description: field("description").unwrap_or_default(),
        order: field("order").and_then(|o| o.parse().ok()).unwrap_or(100),
        html: html_out,
        toc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_fold_french_accents() {
        assert_eq!(slugify("Ce que génère la CLI"), "ce-que-genere-la-cli");
        assert_eq!(slugify("Les cœurs : `#[state]`"), "les-coeurs-state");
        assert_eq!(slugify("  --a__b--  "), "a-b");
    }

    #[test]
    fn renders_headings_code_tables_callouts_and_links() {
        let doc = render(
            concat!(
                "---\ntitle: Hello\ndescription: \"World\"\norder: 3\n---\n",
                "Intro with [a link](https://x.dev) and [docs](../cli/).\n\n",
                "## Install it\n\n```sh\ncargo install ferroswift-cli\n```\n\n",
                "### Install it\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n",
                "```rust\nlet x = \"<b>\";\n```\n\n",
                "> **Astuce :** read [this](https://y.dev).\n"
            ),
            "Copy",
            "Copied",
        );
        assert_eq!(
            (doc.title.as_str(), doc.description.as_str(), doc.order),
            ("Hello", "World", 3)
        );
        assert!(doc.html.contains(
            r#"<a href="https://x.dev" target="_blank" rel="noopener noreferrer">a link</a>"#
        ));
        assert!(doc.html.contains(r#"<a href="../cli/">docs</a>"#));
        assert!(
            doc.html
                .contains(r##"<h2 id="install-it"><a class="anchor" href="#install-it""##)
        );
        assert!(doc.html.contains(r#"<h3 id="install-it-1">"#));
        assert!(
            doc.html
                .contains(r#"<div class="code" data-lang="terminal">"#)
        );
        assert!(doc.html.contains(r#"<span class="h-f">cargo</span>"#));
        assert!(doc.html.contains("&quot;&lt;b&gt;&quot;"), "{}", doc.html);
        assert!(doc.html.contains(r#"<div class="table"><table>"#));
        assert!(doc.html.contains(r#"<blockquote class="callout tip">"#));
        assert_eq!(doc.toc.len(), 2);
    }
}
