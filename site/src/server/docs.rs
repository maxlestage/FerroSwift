//! The documentation: `site/docs/{fr,en}/{slug}.md`, ordered by their `order:` field.

use std::collections::HashMap;
use std::path::Path;

use super::i18n::t;
use super::markdown::{Doc, render};
use crate::routes::Lang;

pub struct Docs {
    pages: HashMap<(Lang, String), Doc>,
    slugs: Vec<String>,
}

impl Docs {
    /// Reads every `dir/{lang}/*.md`. A page missing in one language shows the other one.
    pub fn load(dir: &Path) -> Docs {
        let mut pages = HashMap::new();
        for lang in Lang::ALL {
            let Ok(entries) = std::fs::read_dir(dir.join(lang.code())) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let Some(slug) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if let Ok(src) = std::fs::read_to_string(&path) {
                    let copied = crate::labels::labels(lang).copied;
                    pages.insert(
                        (lang, slug.to_string()),
                        render(&src, t(lang).docs.copy, copied),
                    );
                }
            }
        }
        let mut slugs: Vec<String> = pages.keys().map(|(_, s)| s.clone()).collect();
        slugs.sort();
        slugs.dedup();
        let order = |slug: &String| {
            Lang::ALL
                .iter()
                .find_map(|l| pages.get(&(*l, slug.clone())))
                .map_or(100, |d: &Doc| d.order)
        };
        slugs.sort_by_key(|s| (order(s), s.clone()));
        Docs { pages, slugs }
    }

    /// Pages that exist, in reading order.
    pub fn slugs(&self) -> &[String] {
        &self.slugs
    }

    pub fn get(&self, lang: Lang, slug: &str) -> Option<&Doc> {
        self.pages
            .get(&(lang, slug.to_string()))
            .or_else(|| self.pages.get(&(lang.other(), slug.to_string())))
    }

    pub fn title<'a>(&'a self, lang: Lang, slug: &'a str) -> &'a str {
        self.get(lang, slug).map_or(slug, |d| d.title.as_str())
    }

    /// The pages before and after `slug`.
    pub fn neighbours(&self, slug: &str) -> (Option<&str>, Option<&str>) {
        let i = self.slugs.iter().position(|s| s == slug);
        let prev = i
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| self.slugs.get(i))
            .map(String::as_str);
        let next = i.and_then(|i| self.slugs.get(i + 1)).map(String::as_str);
        (prev, next)
    }
}
