//! The native side: pages in French and English, docs, static files, the static export
//! and a small development server. Both serve the same responses ([`Site::respond`]).

pub mod assets;
pub mod compare;
pub mod dev;
pub mod docs;
pub mod export;
pub mod i18n;
pub mod markdown;
pub mod pages;

use std::path::{Path, PathBuf};

use crate::routes::{Lang, Page, Route, base};
use assets::Assets;
use docs::Docs;

/// The browser bundle written by wasm-bindgen into `site/pkg`.
pub const BUNDLE_JS: &str = "/pkg/ferroswift_site.js";
pub const BUNDLE_WASM: &str = "/pkg/ferroswift_site_bg.wasm";

pub struct Site {
    pub docs: Docs,
    pub assets: Assets,
    /// Absolute address of the site, base path included, without trailing slash
    /// (`https://maxlestage.github.io/FerroSwift`): canonical URLs, Open Graph, sitemap.
    pub origin: String,
}

/// An HTTP response.
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
    /// `Location` of a redirection.
    pub location: Option<String>,
}

impl Response {
    fn ok(content_type: &'static str, body: impl Into<Vec<u8>>) -> Response {
        Response {
            status: 200,
            content_type,
            body: body.into(),
            location: None,
        }
    }

    fn html(status: u16, html: String) -> Response {
        Response {
            status,
            content_type: "text/html; charset=utf-8",
            body: html.into_bytes(),
            location: None,
        }
    }
}

/// The `site/` folder: `SITE_DIR`, or the crate folder when run with cargo.
pub fn site_dir() -> PathBuf {
    std::env::var_os("SITE_DIR")
        .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from)
}

impl Site {
    /// Reads docs, static files and the browser bundle from `dir` (the `site/` folder).
    /// `site_url` is the public address (`SITE_URL`); the base path (see
    /// [`set_base`](crate::routes::set_base)) is appended to it when missing.
    pub fn load(dir: &Path, site_url: Option<String>) -> Site {
        let mut assets = Assets::default();
        assets.load(&dir.join("static"), "/static");
        assets.load(&dir.join("pkg"), "/pkg");
        let origin = site_url
            .map(|u| u.trim().trim_end_matches('/').to_string())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| "http://localhost:3000".into());
        let origin = if origin.ends_with(base()) {
            origin
        } else {
            format!("{origin}{}", base())
        };
        Site {
            docs: Docs::load(&dir.join("docs")),
            assets,
            origin,
        }
    }

    /// Whether the browser bundle has been built (see site/README.md).
    pub fn has_bundle(&self) -> bool {
        self.assets.get(BUNDLE_JS).is_some() && self.assets.get(BUNDLE_WASM).is_some()
    }

    /// Every page of the site, in every language.
    pub fn routes(&self) -> Vec<Route> {
        let mut routes = Vec::new();
        for lang in Lang::ALL {
            routes.push(Route::new(lang, Page::Home));
            routes.push(Route::new(lang, Page::Playground));
            routes.push(Route::new(lang, Page::Docs));
            routes.extend(
                self.docs
                    .slugs()
                    .iter()
                    .map(|s| Route::new(lang, Page::Doc(s.clone()))),
            );
        }
        routes
    }

    /// The HTML of `route`.
    pub fn page(&self, route: &Route) -> String {
        pages::render(self, route)
    }

    /// The answer to `GET path`, `path` without the base path.
    pub fn respond(&self, path: &str) -> Response {
        let path = path.split(['?', '#']).next().unwrap_or("/");
        if path.contains("..") {
            return self.not_found(Lang::Fr);
        }
        match path {
            "/" | "/index.html" => return Response::html(200, pages::root(self)),
            "/sitemap.xml" => return Response::ok(assets::content_type(path), self.sitemap()),
            "/robots.txt" => return Response::ok(assets::content_type(path), self.robots()),
            "/favicon.svg" | "/og.png" => return self.file(&format!("/static{path}")),
            p if p.starts_with("/static/") || p.starts_with("/pkg/") => return self.file(p),
            _ => {}
        }
        match Route::parse(path, self.docs.slugs()) {
            Some(route) if route.page == Page::NotFound => self.not_found(route.lang),
            Some(route) if !path.ends_with('/') => Response {
                status: 301,
                content_type: "text/plain; charset=utf-8",
                body: Vec::new(),
                location: Some(route.href()),
            },
            Some(route) => Response::html(200, self.page(&route)),
            None => self.not_found(Lang::Fr),
        }
    }

    fn not_found(&self, lang: Lang) -> Response {
        Response::html(404, self.page(&Route::new(lang, Page::NotFound)))
    }

    fn file(&self, url: &str) -> Response {
        match self
            .assets
            .get(url)
            .and_then(|a| std::fs::read(&a.path).ok().map(|bytes| (a, bytes)))
        {
            Some((asset, bytes)) => Response::ok(asset.content_type, bytes),
            None => Response {
                status: 404,
                content_type: "text/plain; charset=utf-8",
                body: b"not found".to_vec(),
                location: None,
            },
        }
    }

    /// Every page with its alternate languages, absolute URLs.
    pub fn sitemap(&self) -> String {
        let origin = &self.origin;
        let mut xml = String::from(concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" ",
            "xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n"
        ));
        for route in self.routes() {
            xml.push_str(&format!("<url><loc>{origin}{}</loc>", route.path()));
            for lang in Lang::ALL {
                xml.push_str(&format!(
                    r#"<xhtml:link rel="alternate" hreflang="{}" href="{origin}{}"/>"#,
                    lang.code(),
                    route.with_lang(lang).path()
                ));
            }
            xml.push_str(&format!(
                r#"<xhtml:link rel="alternate" hreflang="x-default" href="{origin}{}"/></url>"#,
                route.with_lang(Lang::Fr).path()
            ));
            xml.push('\n');
        }
        xml.push_str("</urlset>\n");
        xml
    }

    pub fn robots(&self) -> String {
        format!(
            "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
            self.origin
        )
    }
}
