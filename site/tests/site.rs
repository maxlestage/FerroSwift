//! Every page renders, in both languages, with its SEO tags; the playground island is
//! rendered on the server with the example already translated. No base path here (see
//! export.rs for `/FerroSwift`).
#![cfg(not(target_arch = "wasm32"))]

use ferroswift_site::examples::EXAMPLES;
use ferroswift_site::islands::{Props, Variant, playground, translate};
use ferroswift_site::routes::{Lang, Page, Route};
use ferroswift_site::server::{Site, site_dir};

fn site() -> Site {
    Site::load(&site_dir(), Some("https://example.com/".into()))
}

fn body(site: &Site, path: &str) -> String {
    let response = site.respond(path);
    assert_eq!(response.status, 200, "GET {path}");
    String::from_utf8(response.body).expect("UTF-8")
}

#[test]
fn every_page_ends_with_the_signature_footer() {
    let site = site();
    for (lang, rights, here) in [
        ("en", "All rights reserved", "You are here"),
        ("fr", "Tous droits réservés", "Vous êtes ici"),
    ] {
        let html = body(&site, &format!("/{lang}/docs/"));
        let footer = html.split("<footer").nth(1).expect("a footer");
        // The author's signature, the legal line and the license.
        assert!(footer.contains(r#"class="credits-name""#), "{lang}");
        assert!(footer.contains("Maxime Nathan Lestage"), "{lang}");
        assert!(footer.contains(rights), "{lang}");
        assert!(footer.contains(r#"href="/static/LICENSE.txt""#), "{lang}");
        assert!(!footer.contains("MIT"), "{lang}");
        // The install command and the version of the crates.
        assert!(
            footer.contains(
                "cargo install --git https://github.com/maxlestage/FerroSwift ferroswift-cli"
            ),
            "{lang}"
        );
        assert!(
            footer.contains(&format!("v{}", ferroswift_transpiler::VERSION)),
            "{lang}"
        );
        // The seven projects of the ecosystem, this one marked as the current page.
        for project in [
            "Active",
            "Vitesse",
            "Securite",
            "Memoire",
            "Paquets",
            "FerroSwift",
            "Solidify",
        ] {
            assert!(
                footer.contains(&format!(
                    r#"href="https://maxlestage.github.io/{project}/""#
                )),
                "{lang} {project}"
            );
        }
        assert!(footer.contains(r#"aria-current="page""#), "{lang}");
        assert!(footer.contains(here), "{lang}");
    }
    // The author is named in the metadata too.
    let html = body(&site, "/en/");
    assert!(html.contains(r#"<meta name="author" content="Maxime Nathan Lestage"/>"#));
    assert!(html.contains(r#""author":{"@type":"Person","name":"Maxime Nathan Lestage""#));
    // The license is served by the site, identical to the repository's.
    let response = site.respond("/static/LICENSE.txt");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, include_bytes!("../../LICENSE"));
}

#[test]
fn every_route_renders_with_its_seo_tags() {
    let site = site();
    let routes = site.routes();
    assert!(routes.len() >= 2 * (3 + 5), "{} routes", routes.len());
    for route in routes {
        let path = route.path();
        let html = body(&site, &path);
        let lang = route.lang.code();
        assert!(
            html.starts_with(&format!(r#"<!DOCTYPE html><html lang="{lang}">"#)),
            "{path}"
        );
        assert_eq!(html.matches("<title>").count(), 1, "{path}");
        assert!(
            html.contains(" · FerroSwift</title>") || route.page == Page::Home,
            "{path}"
        );
        assert!(
            html.contains(r#"<meta name="description" content=""#),
            "{path}"
        );
        assert!(
            html.contains(&format!(
                r#"<link rel="canonical" href="https://example.com{path}"/>"#
            )),
            "{path}"
        );
        for other in Lang::ALL {
            let alternate = format!(
                r#"<link rel="alternate" hreflang="{}" href="https://example.com{}"/>"#,
                other.code(),
                route.with_lang(other).path()
            );
            assert!(html.contains(&alternate), "{path}: {alternate}");
            // The language switcher links the same page in every language.
            assert!(
                html.contains(&format!(r#"href="{}""#, route.with_lang(other).path())),
                "{path}"
            );
        }
        assert!(html.contains(r#"hreflang="x-default""#), "{path}");
        assert!(
            html.contains(r#"<meta property="og:image" content="https://example.com/og.png"/>"#)
        );
        assert!(html.contains(r#"href="/static/app.css?v="#), "{path}");
        assert!(
            html.contains("https://maxlestage.github.io/Active/"),
            "{path}: made with active"
        );
        assert!(!html.contains("data-base"), "{path}");
        // The source is on GitHub, the only way to get the crate until it is on crates.io.
        assert!(
            html.contains("https://github.com/maxlestage/FerroSwift"),
            "{path}"
        );
    }
}

#[test]
fn home_pages_have_the_pitch_the_playground_and_build_time_comparisons() {
    let site = site();
    let fr = body(&site, "/fr/");
    assert!(fr.contains("Écrivez du Rust."));
    assert!(fr.contains("Livrez du SwiftUI."));
    let en = body(&site, "/en/");
    assert!(en.contains("Write Rust.") && en.contains("Ship SwiftUI."));
    for html in [&fr, &en] {
        assert!(html.contains(r#"<active-island data-island="playground""#));
        // Translated on the server: the Swift of the counter is in the page.
        assert!(html.contains(r#"<span class="h-k">struct</span> <span class="h-t">Counter</span>: <span class="h-t">View</span>"#));
        // Comparisons produced by the transpiler while rendering.
        assert!(
            html.contains(
                r#"<span class="h-k">throws</span>(<span class="h-t">LoginError</span>)"#
            )
        );
        assert!(html.contains(
            "cargo install --git https://github.com/maxlestage/FerroSwift ferroswift-cli"
        ));
        assert!(html.contains("ferroswift watch"));
        assert!(html.contains("SoftwareSourceCode"));
    }
    assert!(fr.contains("rendu sur le serveur"));
    assert!(en.contains("rendered on the server"));
}

#[test]
fn the_bundle_loads_only_where_there_is_an_island() {
    let site = site();
    let doc = body(&site, "/fr/docs/cli/");
    assert!(!doc.contains("/pkg/"), "docs pages have no island");
    let playground = body(&site, "/en/playground/");
    let home = body(&site, "/fr/");
    if site.has_bundle() {
        assert!(
            playground
                .contains(r#"<script type="module">import init from "/pkg/ferroswift_site.js?v="#)
        );
        assert!(
            playground.contains(r#"<link rel="preload" href="/pkg/ferroswift_site_bg.wasm?v="#)
        );
        assert!(
            home.contains("IntersectionObserver") && home.contains("/pkg/ferroswift_site.js?v=")
        );
    } else {
        assert!(!playground.contains("/pkg/") && !home.contains("/pkg/"));
    }
}

#[test]
fn docs_render_in_both_languages_with_a_sidebar_and_a_table_of_contents() {
    let site = site();
    let slugs = site.docs.slugs().to_vec();
    for slug in [
        "getting-started",
        "swiftui",
        "language",
        "cli",
        "limitations",
    ] {
        assert!(slugs.iter().any(|s| s == slug), "missing doc {slug}");
    }
    assert_eq!(slugs[0], "getting-started", "ordered by front matter");
    for lang in Lang::ALL {
        let index = body(&site, &format!("/{}/docs/", lang.code()));
        for slug in &slugs {
            assert!(index.contains(&format!(r#"href="/{}/docs/{slug}/""#, lang.code())));
            let html = body(&site, &format!("/{}/docs/{slug}/", lang.code()));
            let doc = site.docs.get(lang, slug).unwrap();
            assert!(
                !doc.title.is_empty() && !doc.description.is_empty(),
                "{slug}"
            );
            assert!(html.contains(&format!(r#"<h1 class="doc-title">{}</h1>"#, doc.title)));
            assert!(html.contains(r#"class="docs-side""#));
            assert!(html.contains(r#"aria-current="page""#));
            assert!(
                html.contains(r#"<nav class="toc""#),
                "{slug}: table of contents"
            );
        }
    }
    let start = body(&site, "/en/docs/getting-started/");
    assert!(start.contains(r#"<div class="code" data-lang="terminal">"#));
    assert!(start.contains(r#"<div class="code" data-lang="rust">"#));
    let swiftui = body(&site, "/fr/docs/swiftui/");
    assert!(swiftui.contains(r#"<span class="h-a">@State</span>"#) || swiftui.contains("@State"));
    assert!(swiftui.contains("SwiftUI en Rust"));
}

#[test]
fn unknown_pages_and_redirects() {
    let site = site();
    for path in [
        "/fr/nope/",
        "/de/",
        "/fr/docs/nope/",
        "/static/../Cargo.toml",
    ] {
        let response = site.respond(path);
        assert_eq!(response.status, 404, "{path}");
    }
    let html = String::from_utf8(site.respond("/en/nope/").body).unwrap();
    assert!(html.contains(r#"<meta name="robots" content="noindex,follow"/>"#));
    assert!(html.contains("Page not found") && html.contains("Page introuvable"));
    let redirect = site.respond("/fr/playground");
    assert_eq!(redirect.status, 301);
    assert_eq!(redirect.location.as_deref(), Some("/fr/playground/"));
}

#[test]
fn the_root_page_sends_visitors_to_their_language() {
    let site = site();
    let root = body(&site, "/");
    assert!(root.contains("location.replace"));
    assert!(
        root.contains(r#"<noscript><meta http-equiv="refresh" content="0; url=/fr/"/></noscript>"#)
    );
    assert!(root.contains(r#"href="/fr/""#) && root.contains(r#"href="/en/""#));
    assert!(root.contains(r#"<meta name="robots" content="noindex,follow"/>"#));
}

#[test]
fn sitemap_and_robots_list_every_page() {
    let site = site();
    let sitemap = body(&site, "/sitemap.xml");
    for route in site.routes() {
        assert!(sitemap.contains(&format!("<loc>https://example.com{}</loc>", route.path())));
    }
    assert!(sitemap.contains(r#"hreflang="en" href="https://example.com/en/playground/""#));
    assert_eq!(
        body(&site, "/robots.txt"),
        "User-agent: *\nAllow: /\n\nSitemap: https://example.com/sitemap.xml\n"
    );
    assert_eq!(site.respond("/favicon.svg").content_type, "image/svg+xml");
    assert_eq!(site.respond("/static/app.css").status, 200);
}

#[test]
fn every_example_translates_without_errors() {
    for example in &EXAMPLES {
        let out = ferroswift_transpiler::transpile(example.source);
        let errors: Vec<_> = out.diagnostics.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "{}: {errors:?}", example.file);
        assert!(!out.swift.trim().is_empty(), "{}", example.file);
        let translation = translate(example.source, None);
        assert_eq!(translation.errors(), 0, "{}", example.file);
        assert!(!translation.stale && translation.crash.is_none());
    }
}

#[test]
fn the_playground_island_renders_every_example_on_the_server() {
    for (i, example) in EXAMPLES.iter().enumerate() {
        for lang in Lang::ALL {
            let props = Props {
                lang,
                example: i,
                variant: Variant::Page,
            };
            let html = active::island("playground", props.encode(), playground).render();
            assert!(html.starts_with(&format!(
                r#"<active-island data-island="playground" data-props="{} {} page">"#,
                lang.code(),
                example.id
            )));
            // The Rust source is in the textarea, escaped.
            let area = html
                .split_once(r#"<textarea class="pg-input""#)
                .and_then(|(_, rest)| rest.split_once('>'))
                .and_then(|(_, rest)| rest.split_once("</textarea>"))
                .map(|(content, _)| content)
                .expect("textarea");
            assert!(area.contains("-&gt;"), "{}: escaped arrows", example.id);
            assert!(
                !area.contains(['<', '>']),
                "{}: no raw HTML from the code",
                example.id
            );
            assert!(area.starts_with("use ") || area.starts_with("///") || area.starts_with("#"));
            // The selected example and the Swift translation.
            assert!(html.contains(&format!(r#"<option value="{}" selected="">"#, example.id)));
            assert!(
                html.contains(r#"<pre class="pg-swift-code"><code><!--[-->"#)
                    || html.contains(r#"<pre class="pg-swift-code"><code>"#)
            );
            assert!(
                html.contains(r#"<span class="h-k">import</span>"#),
                "{}",
                example.id
            );
            assert!(
                html.contains(r#"class="ok""#),
                "{}: no problems",
                example.id
            );
        }
    }
}

#[test]
fn routes_round_trip() {
    let site = site();
    for route in site.routes() {
        assert_eq!(
            Route::parse(&route.path(), site.docs.slugs()),
            Some(route.clone())
        );
    }
}

#[test]
fn the_site_is_an_installable_app_that_works_offline() {
    let site = site();
    // A manifest per language, the French one at the root too.
    for (path, lang) in [
        ("/manifest.webmanifest", "fr"),
        ("/fr/manifest.webmanifest", "fr"),
        ("/en/manifest.webmanifest", "en"),
    ] {
        let response = site.respond(path);
        assert_eq!(response.status, 200, "{path}");
        assert_eq!(response.content_type, "application/manifest+json", "{path}");
        let json = String::from_utf8(response.body).unwrap();
        assert!(
            json.starts_with(r#"{"name":"FerroSwift","short_name":"FerroSwift""#),
            "{path}"
        );
        assert!(
            json.contains(&format!(r#""start_url":"/{lang}/""#)),
            "{path}"
        );
        assert!(json.contains(&format!(r#""lang":"{lang}""#)), "{path}");
        assert!(
            json.contains(r#""id":"/""#) && json.contains(r#""scope":"/""#),
            "{path}"
        );
        assert!(json.contains(r#""display":"standalone""#), "{path}");
        assert!(json.contains(r##""theme_color":"#0d0b0a""##), "{path}");
        assert!(json.contains(r#""src":"/static/icon-512.png""#), "{path}");
        assert!(json.contains(r#""purpose":"maskable""#), "{path}");
    }
    // The icons are real PNG files.
    for icon in [
        "/static/icon-192.png",
        "/static/icon-512.png",
        "/static/icon-maskable-512.png",
        "/static/apple-touch-icon.png",
    ] {
        let response = site.respond(icon);
        assert_eq!(response.status, 200, "{icon}");
        assert_eq!(response.content_type, "image/png", "{icon}");
        assert!(response.body.starts_with(b"\x89PNG"), "{icon}");
    }
    // Every page is installable and registers the service worker.
    for (path, lang) in [
        ("/fr/", "fr"),
        ("/en/docs/cli/", "en"),
        ("/en/playground/", "en"),
    ] {
        let html = body(&site, path);
        assert!(
            html.contains(&format!(
                r#"<link rel="manifest" href="/{lang}/manifest.webmanifest"/>"#
            )),
            "{path}"
        );
        assert!(
            html.contains(r##"<meta name="theme-color" content="#0d0b0a"/>"##),
            "{path}"
        );
        assert!(
            html.contains(r#"navigator.serviceWorker.register("/sw.js")"#),
            "{path}"
        );
        assert!(
            html.contains(
                r#"<link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png"/>"#
            ),
            "{path}"
        );
    }
    // The service worker caches every page, the offline page, the stylesheet, the icons
    // and the fonts when it installs.
    let response = site.respond("/sw.js");
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/javascript; charset=utf-8");
    let js = String::from_utf8(response.body).unwrap();
    for url in [
        "\"/fr/\"",
        "\"/en/\"",
        "\"/fr/docs/\"",
        "\"/en/docs/cli/\"",
        "\"/en/playground/\"",
        "\"/offline/\"",
        "\"/static/app.css?v=",
        "\"/favicon.svg\"",
        "\"/static/icon-512.png\"",
        "\"/static/fonts/geist-latin-wght-normal.woff2\"",
    ] {
        assert!(js.contains(url), "{url} is not precached");
    }
    assert!(js.contains(r#"const OFFLINE = "/offline/";"#));
    assert!(js.contains(&format!(
        "const CACHE = \"ferroswift-site-{}\";",
        site.version()
    )));
    // The offline page, in both languages, leads to what is cached.
    let html = body(&site, "/offline/");
    assert!(html.contains("Vous êtes hors ligne") && html.contains("You are offline"));
    assert!(html.contains(r#"<meta name="robots" content="noindex,follow"/>"#));
    assert!(html.contains(r#"href="/fr/""#) && html.contains(r#"href="/en/docs/""#));
    assert!(html.contains(r#"href="/en/playground/""#));
    let redirect = site.respond("/offline");
    assert_eq!(redirect.status, 301);
    assert_eq!(redirect.location.as_deref(), Some("/offline/"));
}
