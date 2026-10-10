//! The live playground: an island rendered on the server with the first example already
//! translated (it works without JavaScript and is indexed), then hydrated in the browser
//! where the transpiler, compiled to WebAssembly, translates on every keystroke.

use active::prelude::*;
use active::{Dom, State};

use crate::examples::{EXAMPLES, index_of};
use crate::highlight::{Syntax, highlight, line_count};
use crate::labels::{Labels, fixed2, labels};
use crate::routes::{Lang, link};

/// Every island, registered under the name used by [`island`].
pub const ISLANDS: [(&str, active::IslandFn); 1] = [("playground", playground)];

/// The comment the transpiler writes before the support code it appends.
const SUPPORT_MARK: &str = "// MARK: - FerroSwift support";

/// Where the playground is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// Embedded in the home page, with a link to the full page.
    Home,
    /// The playground page, full width.
    Page,
}

/// The island's props, written in the HTML: `"fr counter page"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Props {
    pub lang: Lang,
    pub example: usize,
    pub variant: Variant,
}

impl Props {
    pub fn encode(&self) -> String {
        let variant = match self.variant {
            Variant::Home => "home",
            Variant::Page => "page",
        };
        format!(
            "{} {} {variant}",
            self.lang.code(),
            EXAMPLES[self.example].id
        )
    }

    pub fn parse(props: &str) -> Props {
        let mut parts = props.split_whitespace();
        let lang = parts.next().and_then(Lang::parse).unwrap_or(Lang::Fr);
        let example = parts.next().map_or(0, index_of);
        let variant = match parts.next() {
            Some("page") => Variant::Page,
            _ => Variant::Home,
        };
        Props {
            lang,
            example,
            variant,
        }
    }
}

/// A diagnostic of the transpiler, as shown under the editor.
#[derive(Clone, Debug, PartialEq)]
pub struct Diag {
    pub error: bool,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

/// The result of translating the editor's content.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Translation {
    /// The whole Swift output (what "Copy" copies).
    pub swift: String,
    pub diagnostics: Vec<Diag>,
    /// Milliseconds taken in the browser; `None` when rendered on the server.
    pub ms: Option<f64>,
    /// The code does not parse: `swift` is the last valid translation.
    pub stale: bool,
    /// The transpiler panicked (a bug): its message.
    pub crash: Option<String>,
}

impl Translation {
    /// The translated code, without the support code.
    pub fn main_code(&self) -> &str {
        match self.swift.find(SUPPORT_MARK) {
            Some(at) => self.swift[..at].trim_end(),
            None => self.swift.trim_end(),
        }
    }

    /// The support code appended by the transpiler (empty when none is needed).
    pub fn support_code(&self) -> &str {
        match self.swift.find(SUPPORT_MARK) {
            Some(at) => self.swift[at + SUPPORT_MARK.len()..].trim(),
            None => "",
        }
    }

    pub fn errors(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.error).count()
    }

    pub fn warnings(&self) -> usize {
        self.diagnostics.iter().filter(|d| !d.error).count()
    }
}

/// Translates `source`. When it cannot be translated at all (a syntax error while typing),
/// the `previous` Swift stays on screen, marked stale.
pub fn translate(source: &str, previous: Option<&Translation>) -> Translation {
    let start = now();
    let result = transpile_guarded(source);
    let ms = start.map(|s| now().unwrap_or(s) - s);
    let previous_swift = || previous.map(|p| p.swift.clone()).unwrap_or_default();
    match result {
        Ok(out) => {
            let diagnostics = out
                .diagnostics
                .iter()
                .map(|d| Diag {
                    error: d.is_error(),
                    line: d.line,
                    column: d.column,
                    message: d.message.clone(),
                })
                .collect();
            let failed = out.swift.trim().is_empty() && out.has_errors();
            let stale = failed && previous.is_some_and(|p| !p.swift.is_empty());
            Translation {
                swift: if stale { previous_swift() } else { out.swift },
                diagnostics,
                ms,
                stale,
                crash: None,
            }
        }
        Err(message) => Translation {
            swift: previous_swift(),
            diagnostics: Vec::new(),
            ms,
            stale: previous.is_some(),
            crash: Some(message),
        },
    }
}

/// The transpiler, protected against its own panics: in the browser a panic would
/// otherwise stop the WebAssembly module for good.
fn transpile_guarded(source: &str) -> Result<ferroswift_transpiler::Output, String> {
    #[cfg(target_arch = "wasm32")]
    return crate::client::guard::transpile(source);
    #[cfg(not(target_arch = "wasm32"))]
    std::panic::catch_unwind(|| ferroswift_transpiler::transpile(source)).map_err(|panic| {
        panic
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic".into())
    })
}

/// Milliseconds since the page loaded, in the browser; `None` on the server.
fn now() -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    return crate::client::now();
    #[cfg(not(target_arch = "wasm32"))]
    None
}

/// The line numbers of the editor, those with a diagnostic marked.
fn gutter(lines: usize, diagnostics: &[Diag]) -> String {
    let mut out = String::with_capacity(lines * 4);
    for n in 1..=lines {
        if n > 1 {
            out.push('\n');
        }
        let mark = diagnostics
            .iter()
            .filter(|d| d.line == n)
            .map(|d| if d.error { "e" } else { "w" })
            .min();
        match mark {
            Some(class) => {
                out.push_str("<span class=\"");
                out.push_str(class);
                out.push_str("\">");
                out.push_str(&n.to_string());
                out.push_str("</span>");
            }
            None => out.push_str(&n.to_string()),
        }
    }
    out
}

/// The highlighted Rust under the textarea. A trailing newline needs a character after it to
/// take its line, as in the textarea.
fn editor_html(source: &str) -> String {
    let mut html = highlight(source, Syntax::Rust);
    if source.is_empty() || source.ends_with('\n') {
        html.push(' ');
    }
    html
}

fn count_label(l: &Labels, r: &Translation) -> String {
    let (e, w) = (r.errors(), r.warnings());
    let errors = match e {
        1 => l.one_error.to_string(),
        n => format!("{n} {}", l.errors),
    };
    let warnings = match w {
        1 => l.one_warning.to_string(),
        n => format!("{n} {}", l.warnings),
    };
    match (e, w) {
        (0, 0) if r.crash.is_none() => "✓".into(),
        (_, 0) => errors,
        (0, _) => warnings,
        _ => format!("{errors}, {warnings}"),
    }
}

/// The playground. Props: see [`Props`].
pub fn playground(props: &str) -> Node {
    let props = Props::parse(props);
    let lang = props.lang;
    let l = labels(lang);
    let page = props.variant == Variant::Page;
    let first = &EXAMPLES[props.example];

    let example = use_state(props.example);
    let source = use_state(first.source.to_string());
    let result = use_state(translate(first.source, None));
    let copied = use_state(false);
    // Escape was just pressed in the editor: the next Tab leaves it.
    let escape = use_state(false);
    // The editor's elements, once in the page (browser).
    let editor: State<Option<Dom>> = use_state(None);
    let scroller: State<Option<Dom>> = use_state(None);
    let picker: State<Option<Dom>> = use_state(None);

    // New Rust code: translated at once.
    let apply = move |code: String| {
        let next = result.with(|previous| translate(&code, Some(previous)));
        source.set(code);
        result.set(next);
    };

    // Loads example `i` into the editor.
    let load = move |i: usize| {
        let i = i.min(EXAMPLES.len() - 1);
        example.set(i);
        let code = EXAMPLES[i].source.to_string();
        #[cfg(target_arch = "wasm32")]
        crate::client::editor::show(
            editor.get().as_ref(),
            scroller.get().as_ref(),
            picker.get().as_ref(),
            &code,
            EXAMPLES[i].id,
            page,
        );
        let _ = (editor, scroller, picker);
        apply(code);
    };

    // A click on a diagnostic: select its line in the editor.
    let go_to = move |line: usize, column: usize| {
        #[cfg(target_arch = "wasm32")]
        if let (Some(area), Some(scroll)) = (editor.get(), scroller.get()) {
            source.with(|s| crate::client::editor::select(&area, &scroll, s, line, column));
        }
        let _ = (line, column);
    };

    let toolbar = div()
        .class("pg-bar")
        .child(
            label()
                .class("pg-pick")
                .child(span().class("pg-pick-label").text(l.example))
                .child(
                    select()
                        .attr("autocomplete", "off")
                        .on("change", move |e| load(index_of(&e.value())))
                        .on_mount(move |dom| picker.set(Some(dom.to_owned())))
                        .children(EXAMPLES.iter().enumerate().map(|(i, ex)| {
                            let option = option().attr("value", ex.id).text(ex.title(lang));
                            if i == props.example {
                                option.attr("selected", "")
                            } else {
                                option
                            }
                        })),
                ),
        )
        .child(
            span()
                .class("pg-time")
                .class_if("live", move || result.with(|r| r.ms.is_some()))
                .child(span().class("pg-dot").attr("aria-hidden", "true"))
                .text_dyn(move || {
                    result.with(|r| match r.ms {
                        None => l.server.to_string(),
                        Some(ms) => format!("{} · {} ms", l.wasm, fixed2(ms, l.decimal)),
                    })
                }),
        )
        .child(
            div()
                .class("pg-actions")
                .child(if page {
                    Node::from(
                        button()
                            .class("pg-btn ghost")
                            .attr("type", "button")
                            .on_click(move |_| load(example.get()))
                            .text(l.reset),
                    )
                } else {
                    Node::from(
                        a().class("pg-btn ghost")
                            .attr_dyn("href", move || {
                                link(&format!(
                                    "/{}/playground/?example={}",
                                    lang.code(),
                                    EXAMPLES[example.get()].id
                                ))
                            })
                            .text(l.open),
                    )
                })
                .child(
                    button()
                        .class("pg-btn")
                        .attr("type", "button")
                        .on_click(move |_| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                result.with(|r| crate::client::copy_text(&r.swift));
                                copied.set(true);
                                crate::client::after(1600, move || copied.set(false));
                            }
                        })
                        .child(copy_icon())
                        .text_dyn(move || {
                            (if copied.get() { l.copied } else { l.copy }).to_string()
                        }),
                ),
        );

    let rust_pane = div()
        .class("pg-pane pg-rust")
        .child(
            div()
                .class("pg-head")
                .child(span().class("pg-lang rust").text("Rust"))
                .child(
                    span()
                        .class("pg-file")
                        .text_dyn(move || EXAMPLES[example.get()].file.to_string()),
                )
                .child(span().class("pg-hint").id("pg-hint").text(l.hint)),
        )
        .child(
            div()
                .class("pg-scroll")
                .on_mount(move |dom| scroller.set(Some(dom.to_owned())))
                .child(
                    div()
                        .class("pg-code")
                        .child(
                            div()
                                .class("pg-gutter")
                                .attr("aria-hidden", "true")
                                .html_dyn(move || {
                                    let lines = source.with(|s| line_count(s));
                                    result.with(|r| gutter(lines, &r.diagnostics))
                                }),
                        )
                        .child(
                            div()
                                .class("pg-stack")
                                .child(pre().class("pg-hl").attr("aria-hidden", "true").child(
                                    code().html_dyn(move || source.with(|s| editor_html(s))),
                                ))
                                .child(
                                    textarea()
                                        .class("pg-input")
                                        .attr("aria-label", l.editor)
                                        .attr("aria-describedby", "pg-hint")
                                        .attr("spellcheck", "false")
                                        .attr("autocapitalize", "off")
                                        .attr("autocomplete", "off")
                                        .attr("autocorrect", "off")
                                        .attr("wrap", "off")
                                        .attr("translate", "no")
                                        .on_input(move |e| {
                                            escape.set(false);
                                            apply(e.value());
                                        })
                                        .on("keydown", move |_e| {
                                            #[cfg(target_arch = "wasm32")]
                                            crate::client::editor::keydown(&_e, escape);
                                        })
                                        .on_mount(move |dom| {
                                            editor.set(Some(dom.to_owned()));
                                            // Text typed before the bundle loaded, or restored
                                            // by the browser.
                                            #[cfg(target_arch = "wasm32")]
                                            if let Some(typed) = crate::client::editor::value(dom) {
                                                if source.with(|s| *s != typed) {
                                                    apply(typed);
                                                }
                                            }
                                        })
                                        .text(first.source),
                                ),
                        ),
                ),
        );

    let swift_pane = div()
        .class("pg-pane pg-swift")
        .class_if("stale", move || result.with(|r| r.stale))
        .child(
            div()
                .class("pg-head")
                .child(span().class("pg-lang swift").text("Swift"))
                .child(
                    span()
                        .class("pg-file")
                        .text_dyn(move || EXAMPLES[example.get()].swift_file.to_string()),
                ),
        )
        .child(
            div()
                .class("pg-scroll pg-out")
                .attr("tabindex", "0")
                .attr("role", "region")
                .attr("aria-label", l.output)
                .child(
                    div()
                        .class("pg-code")
                        .child(
                            div()
                                .class("pg-gutter")
                                .attr("aria-hidden", "true")
                                .html_dyn(move || {
                                    result.with(|r| gutter(line_count(r.main_code()), &[]))
                                }),
                        )
                        .child(pre().class("pg-swift-code").child(code().html_dyn(move || {
                            result.with(|r| highlight(r.main_code(), Syntax::Swift))
                        }))),
                )
                .child(
                    details()
                        .class("pg-support")
                        .bool_attr("hidden", move || {
                            result.with(|r| r.support_code().is_empty())
                        })
                        .child(summary().text(l.support).text_dyn(move || {
                            result.with(|r| {
                                format!(" · {} {}", line_count(r.support_code()), l.lines)
                            })
                        }))
                        .child(pre().child(code().html_dyn(move || {
                            result.with(|r| highlight(r.support_code(), Syntax::Swift))
                        }))),
                ),
        );

    let diagnostics = div()
        .class("pg-diags")
        .child(
            div()
                .class("pg-diags-head")
                .child(span().text(l.problems))
                .child(
                    span()
                        .class("pg-count")
                        .class_if("bad", move || {
                            result.with(|r| r.errors() > 0 || r.crash.is_some())
                        })
                        .attr("aria-live", "polite")
                        .text_dyn(move || result.with(|r| count_label(l, r))),
                ),
        )
        .child(ul().class("pg-diag-list").children_dyn(move || {
            result.with(|r| {
                let mut items: Vec<Node> = Vec::new();
                if let Some(crash) = &r.crash {
                    items.push(
                        li().class("crash")
                            .child(strong().text(l.crash))
                            .text(" — ")
                            .text(crash.clone())
                            .into(),
                    );
                }
                if r.stale && r.crash.is_none() {
                    items.push(li().class("info").text(l.stale).into());
                }
                for d in &r.diagnostics {
                    let (line, column) = (d.line, d.column);
                    items.push(
                        li().child(
                            button()
                                .attr("type", "button")
                                .class(if d.error {
                                    "diag error"
                                } else {
                                    "diag warning"
                                })
                                .attr("title", format!("{} {line}", l.go_to))
                                .on_click(move |_| go_to(line, column))
                                .child(span().class("sev").text(if d.error {
                                    l.error
                                } else {
                                    l.warning
                                }))
                                .child(span().class("loc").text(format!("{line}:{column}")))
                                .child(span().class("msg").text(d.message.clone())),
                        )
                        .into(),
                    );
                }
                if items.is_empty() {
                    items.push(li().class("ok").text(l.no_problems).into());
                }
                items
            })
        }));

    div()
        .class(if page { "pg pg-page" } else { "pg pg-home" })
        .on_mount(move |_| {
            // `?example=todo` on the playground page.
            #[cfg(target_arch = "wasm32")]
            if page {
                if let Some(id) = crate::client::query_param("example") {
                    let i = index_of(&id);
                    if EXAMPLES[i].id == id && i != example.get() {
                        load(i);
                    }
                }
            }
        })
        .child(toolbar)
        .child(div().class("pg-panes").child(rust_pane).child(swift_pane))
        .child(diagnostics)
        .into()
}

fn copy_icon() -> Element {
    svg()
        .attr("viewBox", "0 0 24 24")
        .attr("width", "15")
        .attr("height", "15")
        .attr("aria-hidden", "true")
        .child(
            rect()
                .attr("x", "8")
                .attr("y", "8")
                .attr("width", "12")
                .attr("height", "12")
                .attr("rx", "2.5")
                .attr("fill", "none")
                .attr("stroke", "currentColor")
                .attr("stroke-width", "2"),
        )
        .child(
            path()
                .attr("d", "M16 8V6.5A2.5 2.5 0 0 0 13.5 4h-7A2.5 2.5 0 0 0 4 6.5v7A2.5 2.5 0 0 0 6.5 16H8")
                .attr("fill", "none")
                .attr("stroke", "currentColor")
                .attr("stroke-width", "2"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn props_round_trip() {
        let props = Props {
            lang: Lang::En,
            example: 3,
            variant: Variant::Page,
        };
        assert_eq!(props.encode(), "en errors page");
        assert_eq!(Props::parse(&props.encode()), props);
        assert_eq!(
            Props::parse("nonsense"),
            Props {
                lang: Lang::Fr,
                example: 0,
                variant: Variant::Home
            }
        );
    }

    #[test]
    fn keeps_the_last_valid_swift_on_syntax_errors() {
        let good = translate("fn a() -> i32 { 1 }", None);
        assert!(good.swift.contains("func a() -> Int"), "{}", good.swift);
        assert!(!good.stale);
        let bad = translate("fn a( -> {", Some(&good));
        assert!(bad.stale);
        assert_eq!(bad.swift, good.swift);
        assert!(bad.errors() > 0);
        assert!(bad.diagnostics.iter().all(|d| d.line >= 1 && d.column >= 1));
    }

    #[test]
    fn marks_diagnostic_lines_in_the_gutter() {
        let d = |error, line| Diag {
            error,
            line,
            column: 1,
            message: String::new(),
        };
        assert_eq!(
            gutter(3, &[d(false, 2), d(true, 3), d(false, 3)]),
            "1\n<span class=\"w\">2</span>\n<span class=\"e\">3</span>"
        );
    }
}
