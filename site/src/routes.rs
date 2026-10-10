//! URLs of the site: `/{lang}/`, `/{lang}/playground/`, `/{lang}/docs/` and
//! `/{lang}/docs/{slug}/`, all behind an optional base path (`/FerroSwift` when the site is
//! published on GitHub Pages).

use std::sync::OnceLock;

static BASE: OnceLock<String> = OnceLock::new();

/// Sets the base path once, before anything is rendered: `""` (the default) or
/// `/FerroSwift`. The generator takes it from `BASE_PATH`, the browser from
/// `<html data-base>`.
pub fn set_base(base: &str) {
    let _ = BASE.set(normalize_base(base));
}

/// `"FerroSwift/"` → `"/FerroSwift"`, `"/"` → `""`.
pub fn normalize_base(base: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    match base {
        "" => String::new(),
        b if b.starts_with('/') => b.to_string(),
        b => format!("/{b}"),
    }
}

/// The base path, without trailing slash (empty by default).
pub fn base() -> &'static str {
    BASE.get().map_or("", String::as_str)
}

/// A link to `path` (which starts with `/`), behind the base path.
pub fn link(path: &str) -> String {
    format!("{}{path}", base())
}

/// The languages of the site. French first: it is the default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    Fr,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Fr, Lang::En];

    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
        }
    }

    pub fn parse(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    /// The other language (for the language switcher).
    pub fn other(self) -> Lang {
        match self {
            Lang::Fr => Lang::En,
            Lang::En => Lang::Fr,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Playground,
    Docs,
    Doc(String),
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub lang: Lang,
    pub page: Page,
}

impl Route {
    pub fn new(lang: Lang, page: Page) -> Route {
        Route { lang, page }
    }

    /// The route of `path` (without the base path). `slugs` lists the documentation pages
    /// that exist. `None` when the path does not start with a language.
    pub fn parse(path: &str, slugs: &[String]) -> Option<Route> {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let lang = Lang::parse(parts.first()?)?;
        let page = match parts[1..] {
            [] => Page::Home,
            ["playground"] => Page::Playground,
            ["docs"] => Page::Docs,
            ["docs", slug] if slugs.iter().any(|s| s == slug) => Page::Doc(slug.to_string()),
            _ => Page::NotFound,
        };
        Some(Route { lang, page })
    }

    /// The path of the page, without the base path.
    pub fn path(&self) -> String {
        let l = self.lang.code();
        match &self.page {
            Page::Home => format!("/{l}/"),
            Page::Playground => format!("/{l}/playground/"),
            Page::Docs => format!("/{l}/docs/"),
            Page::Doc(slug) => format!("/{l}/docs/{slug}/"),
            Page::NotFound => format!("/{l}/404/"),
        }
    }

    /// A link to the page, behind the base path.
    pub fn href(&self) -> String {
        link(&self.path())
    }

    /// The same page in another language.
    pub fn with_lang(&self, lang: Lang) -> Route {
        Route {
            lang,
            page: self.page.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_builds_urls() {
        let slugs = vec!["cli".to_string(), "swiftui".to_string()];
        let doc = Route::parse("/fr/docs/cli/", &slugs).unwrap();
        assert_eq!(doc, Route::new(Lang::Fr, Page::Doc("cli".into())));
        assert_eq!(doc.path(), "/fr/docs/cli/");
        assert_eq!(doc.with_lang(Lang::En).path(), "/en/docs/cli/");
        assert_eq!(Route::parse("/en/", &slugs).unwrap().page, Page::Home);
        assert_eq!(
            Route::parse("/en/playground", &slugs).unwrap().page,
            Page::Playground
        );
        assert_eq!(Route::parse("/fr/docs", &slugs).unwrap().page, Page::Docs);
        assert_eq!(
            Route::parse("/en/docs/nope/", &slugs).unwrap().page,
            Page::NotFound
        );
        assert_eq!(Route::parse("/de/", &slugs), None);
        assert_eq!(Route::parse("/", &slugs), None);
    }

    #[test]
    fn normalizes_base_paths() {
        assert_eq!(normalize_base(""), "");
        assert_eq!(normalize_base("/"), "");
        assert_eq!(normalize_base("FerroSwift/"), "/FerroSwift");
        assert_eq!(normalize_base(" /FerroSwift "), "/FerroSwift");
    }
}
