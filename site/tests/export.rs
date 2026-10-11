//! The static export for GitHub Pages: every page as a file, links behind the base path,
//! absolute URLs on the public address.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use ferroswift_site::server::export::export;

fn read(dir: &Path, path: &str) -> String {
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every `href="/…"` and `src="/…"` of `html` starts with the base path.
fn links_behind(html: &str, base: &str, page: &str) {
    for attr in ["href=\"/", "src=\"/", "content=\"0; url=/"] {
        for (i, _) in html.match_indices(attr) {
            let rest = &html[i + attr.len() - 1..];
            assert!(
                rest.starts_with(base),
                "{page}: a link without the base path: {}",
                &rest[..rest.len().min(60)]
            );
        }
    }
}

#[test]
fn exports_every_page_behind_the_base_path() {
    let dir = std::env::temp_dir().join(format!("ferroswift-export-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let files = export(
        &dir,
        "/FerroSwift/",
        Some("https://maxlestage.github.io".into()),
    )
    .expect("export");
    // Pages, static files and, once built, the browser bundle (`site/pkg`).
    assert!(files >= 30, "{files} files");

    let pages = [
        "fr/index.html",
        "en/index.html",
        "fr/playground/index.html",
        "en/playground/index.html",
        "fr/docs/index.html",
        "en/docs/index.html",
        "fr/docs/getting-started/index.html",
        "en/docs/swiftui/index.html",
        "fr/docs/language/index.html",
        "en/docs/cli/index.html",
        "fr/docs/limitations/index.html",
        "fr/404/index.html",
        "en/404/index.html",
        "offline/index.html",
        "index.html",
        "404.html",
    ];
    for page in pages {
        let html = read(&dir, page);
        links_behind(&html, "/FerroSwift/", page);
        assert!(
            html.contains(r#"href="/FerroSwift/static/app.css?v="#),
            "{page}"
        );
    }

    let doc = read(&dir, "fr/docs/cli/index.html");
    assert!(doc.contains(r#"<html lang="fr" data-base="/FerroSwift">"#));
    assert!(doc.contains(
        r#"<link rel="canonical" href="https://maxlestage.github.io/FerroSwift/fr/docs/cli/"/>"#
    ));
    assert!(
        doc.contains(
            r#"hreflang="en" href="https://maxlestage.github.io/FerroSwift/en/docs/cli/""#
        )
    );
    assert!(
        doc.contains(r#"href="/FerroSwift/en/docs/cli/""#),
        "language switcher"
    );
    assert!(doc.contains(
        r#"<meta property="og:image" content="https://maxlestage.github.io/FerroSwift/og.png"/>"#
    ));
    assert!(doc.contains(r#"<link rel="icon" href="/FerroSwift/favicon.svg""#));

    let playground = read(&dir, "en/playground/index.html");
    if dir.join("pkg/ferroswift_site_bg.wasm").exists() {
        assert!(playground.contains(r#"import init from "/FerroSwift/pkg/ferroswift_site.js?v="#));
        assert!(
            read(&dir, "fr/index.html")
                .contains(r#"import("/FerroSwift/pkg/ferroswift_site.js?v="#)
        );
        assert!(dir.join("pkg/ferroswift_site.js").is_file());
    }
    // The island's link to the full playground, rendered on the server.
    assert!(
        read(&dir, "fr/index.html")
            .contains(r#"href="/FerroSwift/fr/playground/?example=counter""#)
    );

    // What a static host cannot compute.
    let root = read(&dir, "index.html");
    assert!(root.contains("navigator.languages"), "{root}");
    assert!(root.contains(r#"content="0; url=/FerroSwift/fr/""#));
    let not_found = read(&dir, "404.html");
    assert!(not_found.contains(r#"<meta name="robots" content="noindex,follow"/>"#));
    assert!(not_found.contains("navigator.languages") && not_found.contains(r#""404/""#));
    assert!(not_found.contains(r#"href="/FerroSwift/en/404/""#));
    assert!(read(&dir, "en/404/index.html").contains("Page not found"));
    // The PWA, behind the base path.
    let manifest = read(&dir, "en/manifest.webmanifest");
    assert!(
        manifest.contains(r#""start_url":"/FerroSwift/en/""#),
        "{manifest}"
    );
    assert!(
        manifest.contains(r#""scope":"/FerroSwift/""#)
            && manifest.contains(r#""id":"/FerroSwift/""#)
    );
    assert!(manifest.contains(r#""src":"/FerroSwift/static/icon-512.png""#));
    assert!(read(&dir, "manifest.webmanifest").contains(r#""start_url":"/FerroSwift/fr/""#));
    let worker = read(&dir, "sw.js");
    assert!(
        worker.contains(r#""/FerroSwift/fr/docs/cli/""#)
            && worker.contains(r#""/FerroSwift/offline/""#)
    );
    assert!(!worker.contains("\"/fr/\""), "a page outside the base path");
    assert!(worker.contains(r#"const OFFLINE = "/FerroSwift/offline/";"#));
    let sitemap = read(&dir, "sitemap.xml");
    assert!(
        sitemap.contains("<loc>https://maxlestage.github.io/FerroSwift/fr/docs/swiftui/</loc>")
    );
    assert!(sitemap.contains("<loc>https://maxlestage.github.io/FerroSwift/en/playground/</loc>"));
    assert!(
        read(&dir, "robots.txt")
            .contains("Sitemap: https://maxlestage.github.io/FerroSwift/sitemap.xml")
    );

    for file in [
        "static/app.css",
        "static/favicon.svg",
        "static/og.png",
        "static/fonts/geist-latin-wght-normal.woff2",
        "static/fonts/Geist-OFL.txt",
        "favicon.svg",
        "og.png",
        ".nojekyll",
    ] {
        assert!(dir.join(file).is_file(), "{file}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
