//! Every page of the site, written with active.

use active::prelude::*;

use super::compare::{self, Comparison};
use super::i18n::{Item, Section, t};
use super::{BUNDLE_JS, BUNDLE_WASM, Site};
use crate::highlight::{Syntax, escape};
use crate::islands::{Props, Variant};
use crate::labels::labels;
use crate::routes::{Lang, Page, Route, base, link};

pub const GITHUB: &str = "https://github.com/maxlestage/FerroSwift";
pub const ACTIVE: &str = "https://github.com/maxlestage/Active";

/// How a page loads the browser bundle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Bundle {
    /// No island: no WebAssembly.
    None,
    /// Loaded when the island comes near the viewport.
    Lazy,
    /// Preloaded and started at once.
    Eager,
}

/// Adds `js` to `<html>` (copy buttons are shown), and makes `[data-copy]` and `.code`
/// buttons copy their text.
const INLINE_SCRIPT: &str = concat!(
    "<script>document.documentElement.classList.add('js');",
    "document.addEventListener('click',function(e){",
    "var b=e.target.closest&&e.target.closest('button.copy');",
    "if(!b||!navigator.clipboard)return;",
    "var c=b.closest('.code,.cmd');var p=c&&c.querySelector('pre,code');",
    "var s=b.getAttribute('data-copy')||(p?p.innerText:'');",
    "navigator.clipboard.writeText(s).then(function(){",
    "var o=b.textContent;b.textContent=b.getAttribute('data-done')||'✓';b.classList.add('done');",
    "setTimeout(function(){b.textContent=o;b.classList.remove('done')},1600)})});</script>"
);

/// A JavaScript string literal that cannot close the `<script>` element.
fn js_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Starts the bundle once the first island is less than a screen away (or touched).
fn lazy_loader(js: &str, wasm: &str) -> String {
    format!(
        concat!(
            "<script type=\"module\">",
            "const i=document.querySelector('active-island');let s=false;",
            "const go=()=>{{if(s)return;s=true;import({js}).then(m=>m.default({{module_or_path:{wasm}}}))}};",
            "if(i){{const t=i.firstElementChild||i;",
            "['pointerdown','focusin'].forEach(e=>i.addEventListener(e,go,{{once:true}}));",
            "if('IntersectionObserver' in window){{const o=new IntersectionObserver(es=>{{",
            "if(es.some(e=>e.isIntersecting)){{o.disconnect();go()}}}},{{rootMargin:'600px 0px'}});o.observe(t)}}",
            "else go()}}</script>"
        ),
        js = js_string(js),
        wasm = js_string(wasm),
    )
}

// ----- Pieces -----

/// The glyph: an iron tile, "Fe" (ferro), in the colours of Rust and Swift.
pub fn glyph(id: &str) -> Element {
    svg()
        .class("glyph")
        .attr("viewBox", "0 0 32 32")
        .attr("aria-hidden", "true")
        .child(
            defs().child(
                linear_gradient()
                    .id(id.to_string())
                    .attr("x1", "0")
                    .attr("y1", "0")
                    .attr("x2", "1")
                    .attr("y2", "1")
                    .child(stop().attr("offset", "0").attr("stop-color", "#CE412B"))
                    .child(stop().attr("offset", "0.55").attr("stop-color", "#F05138"))
                    .child(stop().attr("offset", "1").attr("stop-color", "#FF9152")),
            ),
        )
        .child(
            rect()
                .attr("width", "32")
                .attr("height", "32")
                .attr("rx", "8")
                .attr("fill", format!("url(#{id})")),
        )
        .child(
            path()
                .attr(
                    "d",
                    "M9.5 23V9.5h7.5M9.5 16h6M18.6 18.6h8.6a4.3 4.3 0 1 0-1.26 3.04",
                )
                .attr("fill", "none")
                .attr("stroke", "#fff")
                .attr("stroke-width", "2.6")
                .attr("stroke-linecap", "round")
                .attr("stroke-linejoin", "round"),
        )
}

fn logo(id: &str) -> Element {
    span()
        .class("logo")
        .child(glyph(id))
        .child(span().class("word").text("Ferro").child(em().text("Swift")))
}

fn arrow() -> Element {
    svg()
        .class("arrow")
        .attr("viewBox", "0 0 24 24")
        .attr("aria-hidden", "true")
        .child(
            path()
                .attr("d", "M5 12h14M13 6l6 6-6 6")
                .attr("fill", "none")
                .attr("stroke", "currentColor")
                .attr("stroke-width", "2")
                .attr("stroke-linecap", "round")
                .attr("stroke-linejoin", "round"),
        )
}

fn github_icon() -> Element {
    svg().attr("viewBox", "0 0 24 24").attr("aria-hidden", "true").attr("width", "18").attr("height", "18").child(path().attr("fill", "currentColor").attr(
        "d",
        "M12 .5a11.5 11.5 0 0 0-3.64 22.41c.58.1.79-.25.79-.56v-2c-3.2.7-3.88-1.37-3.88-1.37-.53-1.33-1.28-1.69-1.28-1.69-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.71 1.26 3.37.96.1-.75.4-1.26.73-1.55-2.56-.29-5.25-1.28-5.25-5.7 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.47.11-3.06 0 0 .97-.31 3.17 1.18a11 11 0 0 1 5.77 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.77.11 3.06.74.81 1.19 1.84 1.19 3.1 0 4.43-2.7 5.4-5.27 5.69.41.36.78 1.06.78 2.14v3.17c0 .31.21.67.8.56A11.5 11.5 0 0 0 12 .5Z",
    ))
}

fn external(link: Element) -> Element {
    link.attr("target", "_blank")
        .attr("rel", "noopener noreferrer")
}

fn lang_switch(route: &Route) -> Element {
    let d = t(route.lang);
    div()
        .class("langs")
        .attr("role", "group")
        .attr("aria-label", d.nav.language)
        .children(Lang::ALL.map(|l| {
            let a = a()
                .href(route.with_lang(l).href())
                .attr("hreflang", l.code())
                .attr("lang", l.code())
                .attr("title", t(l).lang_name)
                .text(l.code().to_uppercase());
            if l == route.lang {
                a.attr("aria-current", "true")
            } else {
                a.attr("aria-label", d.nav.switch_to)
            }
        }))
}

fn header(route: &Route) -> Element {
    let d = t(route.lang);
    let l = route.lang;
    let current = |page: &Page| match (&route.page, page) {
        (Page::Doc(_), Page::Docs) => true,
        (a, b) => a == b,
    };
    let nav_link = |page: Page, label: &'static str| {
        let link = a().href(Route::new(l, page.clone()).href()).text(label);
        if current(&page) {
            link.attr("aria-current", "page")
        } else {
            link
        }
    };
    el("header")
        .class("top")
        .id("top")
        .child(a().class("skip").href("#main").text(d.skip))
        .child(
            div()
                .class("wrap top-in")
                .child(
                    a().class("brand")
                        .href(Route::new(l, Page::Home).href())
                        .attr("aria-label", "FerroSwift")
                        .child(logo("g-top")),
                )
                .child(
                    nav()
                        .class("nav")
                        .attr("aria-label", d.nav.main)
                        .child(nav_link(Page::Playground, d.nav.playground))
                        .child(nav_link(Page::Docs, d.nav.docs)),
                )
                .child(
                    div().class("top-actions").child(lang_switch(route)).child(
                        external(
                            a().class("icon-btn")
                                .href(GITHUB)
                                .attr("aria-label", "GitHub"),
                        )
                        .child(github_icon()),
                    ),
                ),
        )
}

fn footer(route: &Route, site: &Site) -> Element {
    let d = t(route.lang);
    let f = &d.footer;
    let l = route.lang;
    let doc = |slug: &str| Route::new(l, Page::Doc(slug.to_string())).href();
    let doc_title = |slug: &'static str| site.docs.title(l, slug).to_string();
    let column = |title: &'static str, links: Vec<Element>| {
        nav()
            .class("foot-col")
            .attr("aria-label", title)
            .child(h2().text(title))
            .child(ul().children(links.into_iter().map(|link| li().child(link))))
    };
    el("footer")
        .class("foot")
        .child(
            div()
                .class("wrap foot-grid")
                .child(
                    div()
                        .class("foot-brand")
                        .child(logo("g-foot"))
                        .child(p().class("tagline").text(f.tagline)),
                )
                .child(column(
                    f.product,
                    vec![
                        a().href(Route::new(l, Page::Playground).href())
                            .text(d.nav.playground),
                        a().href(Route::new(l, Page::Docs).href()).text(d.nav.docs),
                        a().href(doc("getting-started"))
                            .text(doc_title("getting-started")),
                        a().href(doc("cli")).text(doc_title("cli")),
                    ],
                ))
                .child(column(
                    f.project,
                    vec![
                        external(a().href(GITHUB).text("GitHub")),
                        external(a().href(format!("{GITHUB}/issues")).text("Issues")),
                        external(a().href(ACTIVE).text(f.made_with)),
                    ],
                ))
                .child(column(
                    d.nav.language,
                    Lang::ALL
                        .iter()
                        .map(|lang| {
                            let a = a()
                                .href(route.with_lang(*lang).href())
                                .attr("hreflang", lang.code())
                                .attr("lang", lang.code())
                                .text(t(*lang).lang_name);
                            if *lang == l {
                                a.attr("aria-current", "true")
                            } else {
                                a
                            }
                        })
                        .collect(),
                )),
        )
        .child(
            div()
                .class("wrap foot-bottom")
                .child(span().text(format!("© 2026 FerroSwift · {}", f.license)))
                .child(
                    external(a().class("made").href(ACTIVE))
                        .text(f.made_with)
                        .text(" ↗"),
                )
                .child(a().href("#top").text(format!("{} ↑", f.top))),
        )
}

/// The page's own SEO: title, description, canonical URL and the other language.
fn seo(route: &Route, title: &str, description: &str) -> Seo {
    let seo = Seo::new()
        .title(title)
        .description(description)
        .canonical(route.path());
    Lang::ALL
        .into_iter()
        .fold(seo, |seo, l| {
            seo.alternate(l.code(), route.with_lang(l).path())
        })
        .alternate("x-default", route.with_lang(Lang::Fr).path())
}

fn json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// A complete HTML page around `content`.
fn document(site: &Site, route: &Route, kind: &str, content: Node, bundle: Bundle) -> String {
    let assets = &site.assets;
    let body = fragment([
        Node::from(header(route)),
        el("main")
            .id("main")
            .class(format!("page page-{kind}"))
            .attr("tabindex", "-1")
            .child(content)
            .into(),
        footer(route, site).into(),
    ]);
    let defaults = Seo::new()
        .site_name("FerroSwift")
        .title_template("%s · FerroSwift")
        .base_url(site.origin.clone())
        .image("/og.png")
        .locale(t(route.lang).locale);
    let mut doc = Document::new(body)
        .lang(route.lang.code())
        .seo(defaults)
        .stylesheet(assets.url("/static/app.css"))
        .head(format!(
            concat!(
                r#"<meta name="color-scheme" content="dark"/>"#,
                r##"<meta name="theme-color" content="#0d0b0a"/>"##,
                r#"<link rel="icon" href="{base}/favicon.svg" type="image/svg+xml"/>"#,
                // Same URL as in app.css (no version query), so the preload is reused.
                r#"<link rel="preload" href="{base}/static/fonts/geist-latin-wght-normal.woff2" as="font" type="font/woff2" crossorigin=""/>"#,
                "{script}"
            ),
            base = base(),
            script = INLINE_SCRIPT,
        ));
    if site.has_bundle() {
        let (js, wasm) = (assets.url(BUNDLE_JS), assets.url(BUNDLE_WASM));
        doc = match bundle {
            Bundle::None => doc,
            Bundle::Eager => doc.client(js, wasm),
            Bundle::Lazy => doc.head(lazy_loader(&js, &wasm)),
        };
    }
    if !base().is_empty() {
        // Read by the browser bundle to build its URLs.
        doc = doc.html_attr("data-base", base());
    }
    doc.render()
}

fn section_head(id: &str, number: usize, s: &Section) -> Element {
    div()
        .class("section-head")
        .child(
            p().class("kicker")
                .child(span().class("num").text(format!("{number:02}")))
                .text(s.kicker),
        )
        .child(h2().id(format!("{id}-title")).text(s.title))
        .child(p().class("sub").text(s.text))
}

/// A code window: a bar with dots and the file name, then the code.
fn window(file: &str, lang: &str, html: &str) -> Element {
    div()
        .class(format!("win win-{lang}"))
        .child(
            div()
                .class("win-bar")
                .child(
                    span()
                        .class("dots")
                        .attr("aria-hidden", "true")
                        .child(i())
                        .child(i())
                        .child(i()),
                )
                .child(span().class("win-file").text(file.to_string()))
                .child(
                    span()
                        .class(format!("win-lang {lang}"))
                        .text(if lang == "rust" { "Rust" } else { "Swift" }),
                ),
        )
        .child(pre().child(code().html(html.to_string())))
}

// ----- Home -----

fn hero(lang: Lang) -> Element {
    let d = &t(lang).home;
    let cmp = compare::hero();
    section()
        .class("hero")
        .attr("aria-labelledby", "hero-title")
        .child(div().class("hero-bg").attr("aria-hidden", "true"))
        .child(
            div()
                .class("wrap hero-grid")
                .child(
                    div()
                        .class("hero-copy")
                        .child(
                            p().class("eyebrow")
                                .child(span().class("pulse").attr("aria-hidden", "true"))
                                .text(d.eyebrow),
                        )
                        .child(
                            h1().id("hero-title")
                                .class("hero-title")
                                .child(span().class("l1").text(d.hero[0]))
                                .child(" ")
                                .child(span().class("l2").text(d.hero[1])),
                        )
                        .child(p().class("lead").text(d.lead))
                        .child(
                            div()
                                .class("hero-cta")
                                .child(
                                    a().class("btn primary")
                                        .href(Route::new(lang, Page::Playground).href())
                                        .text(d.cta_play)
                                        .child(arrow()),
                                )
                                .child(
                                    a().class("btn")
                                        .href(
                                            Route::new(lang, Page::Doc("getting-started".into()))
                                                .href(),
                                        )
                                        .text(d.cta_start),
                                )
                                .child(
                                    external(a().class("btn ghost").href(GITHUB))
                                        .child(github_icon())
                                        .text("GitHub"),
                                ),
                        ),
                )
                .child(
                    figure()
                        .class("hero-demo")
                        .attr(
                            "aria-label",
                            format!("{} → {}", cmp.rust_file, cmp.swift_file),
                        )
                        .child(window(d.hero_rust, "rust", &cmp.rust_html))
                        .child(
                            div()
                                .class("hero-arrow")
                                .attr("aria-hidden", "true")
                                .child(span().text("ferroswift build"))
                                .child(arrow()),
                        )
                        .child(window(d.hero_swift, "swift", &cmp.swift_html)),
                ),
        )
}

fn feature(item: &Item) -> Element {
    article()
        .class("feature")
        .child(code().class("chip").text(item.code))
        .child(h3().text(item.title))
        .child(p().text(item.text))
}

fn compare_tabs(lang: Lang) -> Element {
    let d = &t(lang).home;
    let tabs = compare::tabs();
    let mut root = div()
        .class("tabs")
        .attr("role", "group")
        .attr("aria-labelledby", "compare-title");
    for i in 0..tabs.len() {
        let input = input()
            .class("tab-radio")
            .attr("type", "radio")
            .attr("name", "compare")
            .id(format!("cmp-{i}"));
        root = root.child(if i == 0 {
            input.attr("checked", "")
        } else {
            input
        });
    }
    root.child(
        div().class("tab-labels").children(
            d.tabs
                .iter()
                .enumerate()
                .map(|(i, title)| label().attr("for", format!("cmp-{i}")).text(*title)),
        ),
    )
    .child(
        div()
            .class("tab-panels")
            .children(tabs.iter().enumerate().map(|(i, c): (usize, &Comparison)| {
                div()
                    .class(format!("tab-panel p{i}"))
                    .child(p().class("tab-note").text(d.tab_notes[i]))
                    .child(
                        div()
                            .class("pair")
                            .child(window(c.rust_file, "rust", &c.rust_html))
                            .child(window(c.swift_file, "swift", &c.swift_html)),
                    )
                    .child(c.output.map(|out| {
                        div()
                            .class("output")
                            .child(span().class("output-label").text(match lang {
                                Lang::Fr => "Sortie",
                                Lang::En => "Output",
                            }))
                            .child(pre().text(out))
                    }))
            })),
    )
}

fn home(site: &Site, route: &Route) -> Node {
    let lang = route.lang;
    let d = &t(lang).home;
    let l = labels(lang);
    let seo = seo(route, d.title, d.description)
        .title_template("%s")
        .json_ld(format!(
            r#"{{"@context":"https://schema.org","@type":"SoftwareSourceCode","name":"FerroSwift","description":"{}","codeRepository":"{GITHUB}","programmingLanguage":["Rust","Swift"],"license":"https://opensource.org/licenses/MIT","url":"{}/{}/"}}"#,
            json(d.description),
            site.origin,
            lang.code()
        ));
    let props = Props {
        lang,
        example: 0,
        variant: Variant::Home,
    };
    fragment([
        Node::from(seo),
        hero(lang).into(),
        section()
            .id("playground")
            .class("section play-section")
            .attr("aria-labelledby", "playground-title")
            .child(
                div()
                    .class("wrap")
                    .child(section_head("playground", 1, &d.play)),
            )
            .child(div().class("wrap wide").child(island(
                "playground",
                props.encode(),
                crate::islands::playground,
            )))
            .into(),
        section()
            .id("features")
            .class("section")
            .attr("aria-labelledby", "features-title")
            .child(
                div()
                    .class("wrap")
                    .child(section_head("features", 2, &d.features))
                    .child(
                        div()
                            .class("features")
                            .children(d.items.iter().map(feature)),
                    ),
            )
            .into(),
        section()
            .id("how")
            .class("section")
            .attr("aria-labelledby", "how-title")
            .child(
                div()
                    .class("wrap")
                    .child(section_head("how", 3, &d.how))
                    .child(
                        ol().class("pipeline")
                            .children(d.steps.iter().enumerate().map(|(i, s)| {
                                li().class("step")
                                    .child(span().class("step-num").text(format!("{}", i + 1)))
                                    .child(h3().text(s.title))
                                    .child(p().text(s.text))
                                    .child(code().class("chip").text(s.code))
                            })),
                    ),
            )
            .into(),
        section()
            .id("compare")
            .class("section")
            .attr("aria-labelledby", "compare-title")
            .child(
                div()
                    .class("wrap")
                    .child(section_head("compare", 4, &d.compare))
                    .child(compare_tabs(lang)),
            )
            .into(),
        section()
            .id("start")
            .class("section")
            .attr("aria-labelledby", "start-title")
            .child(
                div()
                    .class("wrap")
                    .child(section_head("start", 5, &d.install))
                    .child(
                        ol().class("commands")
                            .children(d.commands.iter().enumerate().map(|(i, c)| {
                                li().child(span().class("step-num").text(format!("{}", i + 1)))
                                    .child(
                                        div()
                                            .class("cmd-text")
                                            .child(h3().text(c.title))
                                            .child(p().text(c.text)),
                                    )
                                    .child(
                                        div()
                                            .class("cmd")
                                            .child(
                                                span()
                                                    .class("prompt")
                                                    .attr("aria-hidden", "true")
                                                    .text("$"),
                                            )
                                            .child(code().html(crate::highlight::highlight(
                                                c.code,
                                                Syntax::Shell,
                                            )))
                                            .child(
                                                button()
                                                    .class("copy")
                                                    .attr("type", "button")
                                                    .attr("data-copy", c.code)
                                                    .attr("data-done", l.copied)
                                                    .attr(
                                                        "aria-label",
                                                        format!(
                                                            "{} : {}",
                                                            t(lang).docs.copy,
                                                            c.code
                                                        ),
                                                    )
                                                    .text(t(lang).docs.copy),
                                            ),
                                    )
                            })),
                    )
                    .child(
                        a().class("more")
                            .href(Route::new(lang, Page::Doc("getting-started".into())).href())
                            .text(d.install_more)
                            .child(arrow()),
                    ),
            )
            .into(),
    ])
}

// ----- Playground -----

fn playground(route: &Route) -> Node {
    let lang = route.lang;
    let d = &t(lang).playground;
    let props = Props {
        lang,
        example: 0,
        variant: Variant::Page,
    };
    fragment([
        Node::from(seo(route, d.title, d.description)),
        section()
            .class("wrap page-head")
            .child(p().class("kicker").text("Rust → Swift · WebAssembly"))
            .child(h1().text(d.title))
            .child(p().class("lead").text(d.lead))
            .into(),
        div()
            .class("wrap wide")
            .child(island(
                "playground",
                props.encode(),
                crate::islands::playground,
            ))
            .into(),
        section()
            .class("wrap tips")
            .attr("aria-labelledby", "tips-title")
            .child(h2().id("tips-title").text(d.tips_title))
            .child(ul().children(d.tips.iter().map(|tip| li().text(*tip))))
            .into(),
    ])
}

// ----- Docs -----

fn doc_link(site: &Site, lang: Lang, slug: &str, current: Option<&str>) -> Element {
    let link = a()
        .href(Route::new(lang, Page::Doc(slug.to_string())).href())
        .text(site.docs.title(lang, slug).to_string());
    if current == Some(slug) {
        link.attr("aria-current", "page")
    } else {
        link
    }
}

fn docs_nav(site: &Site, lang: Lang, current: Option<&str>) -> Element {
    let d = &t(lang).docs;
    ul().class("docs-list")
        .child(li().child({
            let index = a().href(Route::new(lang, Page::Docs).href()).text(d.title);
            if current.is_none() {
                index.attr("aria-current", "page")
            } else {
                index
            }
        }))
        .children(
            site.docs
                .slugs()
                .iter()
                .map(|slug| li().child(doc_link(site, lang, slug, current))),
        )
}

/// The docs layout: menu, content and (on doc pages) the table of contents.
fn docs_layout(
    site: &Site,
    lang: Lang,
    current: Option<&str>,
    article: Element,
    toc: Option<Element>,
) -> Element {
    let d = &t(lang).docs;
    div()
        .class("wrap docs")
        .child(
            aside().class("docs-side").child(
                nav()
                    .attr("aria-label", d.menu)
                    .child(p().class("docs-side-title").text(d.title))
                    .child(docs_nav(site, lang, current)),
            ),
        )
        .child(
            div()
                .class("docs-main")
                .child(
                    details()
                        .class("docs-menu")
                        .child(summary().text(d.menu))
                        .child(
                            nav()
                                .attr("aria-label", d.menu)
                                .child(docs_nav(site, lang, current)),
                        ),
                )
                .child(article),
        )
        .child(toc)
}

fn docs_index(site: &Site, route: &Route) -> Node {
    let lang = route.lang;
    let d = &t(lang).docs;
    let article = article()
        .class("doc")
        .child(p().class("kicker").text("FerroSwift · Rust → Swift"))
        .child(h1().class("doc-title").text(d.title))
        .child(p().class("doc-lead").text(d.intro))
        .child(
            ol().class("doc-cards")
                .children(site.docs.slugs().iter().enumerate().map(|(i, slug)| {
                    let doc = site.docs.get(lang, slug);
                    li().child(
                        a().href(Route::new(lang, Page::Doc(slug.clone())).href())
                            .child(span().class("step-num").text(format!("{}", i + 1)))
                            .child(strong().text(site.docs.title(lang, slug).to_string()))
                            .child(
                                span().text(doc.map_or(String::new(), |d| d.description.clone())),
                            )
                            .child(arrow()),
                    )
                })),
        );
    fragment([
        Node::from(seo(route, d.title, d.description)),
        docs_layout(site, lang, None, article, None).into(),
    ])
}

fn doc_page(site: &Site, route: &Route, slug: &str) -> Node {
    let lang = route.lang;
    let d = &t(lang).docs;
    let Some(doc) = site.docs.get(lang, slug) else {
        return not_found(lang);
    };
    let (prev, next) = site.docs.neighbours(slug);
    let page_seo = seo(route, &doc.title, &doc.description)
        .og_type("article")
        .json_ld(format!(
            r#"{{"@context":"https://schema.org","@type":"TechArticle","headline":"{}","description":"{}","inLanguage":"{}"}}"#,
            json(&doc.title),
            json(&doc.description),
            lang.code()
        ));
    let pager_link = |slug: Option<&str>, label: String, class: &'static str| -> Node {
        match slug {
            Some(s) => a()
                .href(Route::new(lang, Page::Doc(s.to_string())).href())
                .attr("rel", class)
                .class(class)
                .child(small().text(label))
                .child(span().text(site.docs.title(lang, s).to_string()))
                .into(),
            None => span().into(),
        }
    };
    let article = article()
        .class("doc")
        .child(
            nav()
                .class("crumbs")
                .attr("aria-label", "Breadcrumb")
                .child(a().href(Route::new(lang, Page::Docs).href()).text(d.title))
                .child(span().attr("aria-hidden", "true").text("/"))
                .child(span().attr("aria-current", "page").text(doc.title.clone())),
        )
        .child(h1().class("doc-title").text(doc.title.clone()))
        .child(p().class("doc-lead").text(doc.description.clone()))
        .child(div().class("prose").html(doc.html.clone()))
        .child(
            external(a().class("edit").href(format!(
                "{GITHUB}/edit/master/site/docs/{}/{slug}.md",
                lang.code()
            )))
            .text(d.edit),
        )
        .child(
            nav()
                .class("pager")
                .attr("aria-label", "Pagination")
                .child(pager_link(prev, format!("← {}", d.prev), "prev"))
                .child(pager_link(next, format!("{} →", d.next), "next")),
        );
    let toc = (!doc.toc.is_empty()).then(|| {
        nav()
            .class("toc")
            .attr("aria-label", d.on_this_page)
            .child(p().class("toc-title").text(d.on_this_page))
            .child(ul().children(doc.toc.iter().map(|h| {
                li().class(format!("d{}", h.depth))
                    .child(a().href(format!("#{}", h.id)).text(h.text.clone()))
            })))
    });
    fragment([
        Node::from(page_seo),
        docs_layout(site, lang, Some(slug), article, toc).into(),
    ])
}

// ----- 404 and root -----

/// The "not found" page, with a line in the other language: static hosts show it for
/// every unknown URL.
fn not_found(lang: Lang) -> Node {
    let d = &t(lang).not_found;
    let other = lang.other();
    let o = &t(other).not_found;
    section()
        .class("wrap notfound")
        .child(Seo::new().title(d.title).description(d.text).noindex(true))
        .child(
            span()
                .class("code404")
                .attr("aria-hidden", "true")
                .text("404"),
        )
        .child(h1().text(d.title))
        .child(p().class("lead").text(d.text))
        .child(
            div()
                .class("hero-cta")
                .child(
                    a().class("btn primary")
                        .href(Route::new(lang, Page::Home).href())
                        .text(d.back),
                )
                .child(
                    a().class("btn")
                        .href(Route::new(lang, Page::Playground).href())
                        .text(t(lang).nav.playground),
                ),
        )
        .child(
            p().class("other-lang").attr("lang", other.code()).child(
                a().href(Route::new(other, Page::Home).href())
                    .attr("hreflang", other.code())
                    .text(format!("{} — {}", o.title, o.back)),
            ),
        )
        .into()
}

/// The root page: sends the visitor to their language (French unless the browser prefers
/// English), with links to both for those without JavaScript.
pub fn root(site: &Site) -> String {
    let script = format!(
        concat!(
            "<script>(function(){{var l=navigator.languages||[navigator.language||''],c='fr';",
            "for(var i=0;i<l.length;i++){{var x=String(l[i]).slice(0,2).toLowerCase();",
            "if(x==='fr'||x==='en'){{c=x;break}}}}",
            "location.replace({base}+'/'+c+'/'+location.search+location.hash)}})()</script>",
            r#"<noscript><meta http-equiv="refresh" content="0; url={href}"/></noscript>"#,
        ),
        base = js_string(base()),
        href = escape(&link("/fr/")),
    );
    let fr = &t(Lang::Fr).home;
    let seo = Seo::new()
        .site_name("FerroSwift")
        .title("FerroSwift")
        .description(fr.description)
        .base_url(site.origin.clone())
        .canonical("/fr/")
        .alternate("fr", "/fr/")
        .alternate("en", "/en/")
        .alternate("x-default", "/fr/")
        .image("/og.png")
        .noindex(true);
    let body = main().class("root").child(
        div()
            .class("root-card")
            .child(logo("g-root"))
            .child(p().text("Écrivez du Rust. Livrez du SwiftUI."))
            .child(p().attr("lang", "en").text("Write Rust. Ship SwiftUI."))
            .child(ul().children(Lang::ALL.map(|l| {
                li().child(
                    a().class("btn")
                        .href(link(&format!("/{}/", l.code())))
                        .attr("hreflang", l.code())
                        .attr("lang", l.code())
                        .text(t(l).lang_name),
                )
            }))),
    );
    Document::new(body)
        .lang("fr")
        .seo(seo)
        .stylesheet(site.assets.url("/static/app.css"))
        .head(format!(
            r#"<meta name="color-scheme" content="dark"/><link rel="icon" href="{}/favicon.svg" type="image/svg+xml"/>{script}"#,
            base()
        ))
        .render()
}

/// Renders the page of `route`.
pub fn render(site: &Site, route: &Route) -> String {
    let (kind, content, bundle) = match &route.page {
        Page::Home => ("home", home(site, route), Bundle::Lazy),
        Page::Playground => ("playground", playground(route), Bundle::Eager),
        Page::Docs => ("docs", docs_index(site, route), Bundle::None),
        Page::Doc(slug) => ("doc", doc_page(site, route, slug), Bundle::None),
        Page::NotFound => ("404", not_found(route.lang), Bundle::None),
    };
    document(site, route, kind, content, bundle)
}
