//! `ferroswift-site export <dir>`: the whole site as static files, for GitHub Pages or any
//! static host. Pages come from the same [`Site::respond`] as the development server.
//!
//! What a static host cannot do is replaced: the root page and `404.html` pick the language
//! with a tiny script (the language of the address, then the browser's, French by default)
//! and link both languages; every language has its own "not found" page. The manifests,
//! the service worker and the offline page of the PWA are written too.

use std::io;
use std::path::Path;

use super::{Site, pages, site_dir};
use crate::routes::{Lang, Page, Route, set_base};

/// Writes the site into `out`, links behind `base` (`""` or `/FerroSwift`), absolute URLs
/// (canonical, sitemap, Open Graph) on `site_url`. Returns the number of files written.
pub fn export(out: &Path, base: &str, site_url: Option<String>) -> io::Result<usize> {
    set_base(base);
    let site = Site::load(&site_dir(), site_url);
    if !site.has_bundle() {
        eprintln!(
            "warning: site/pkg is missing: the playground will not be interactive (see site/README.md)"
        );
    }
    let mut written = 0;
    let mut write = |path: &str, bytes: &[u8]| -> io::Result<()> {
        let file = out.join(path.trim_start_matches('/'));
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(file, bytes)?;
        written += 1;
        Ok(())
    };

    for route in site.routes() {
        let response = site.respond(&route.path());
        if response.status != 200 {
            return Err(io::Error::other(format!(
                "{}: status {}",
                route.path(),
                response.status
            )));
        }
        write(&format!("{}index.html", route.path()), &response.body)?;
    }
    write("index.html", &site.respond("/").body)?;
    write("404.html", pages::root(&site, "404/").as_bytes())?;
    for lang in Lang::ALL {
        write(
            &format!("{}/404/index.html", lang.code()),
            site.page(&Route::new(lang, Page::NotFound)).as_bytes(),
        )?;
    }
    // The PWA: a manifest per language (the French one at the root too), the service
    // worker and the page it shows without network.
    let mut files = vec!["/manifest.webmanifest".to_string(), "/sw.js".to_string()];
    files.extend(
        Lang::ALL
            .iter()
            .map(|l| format!("/{}/manifest.webmanifest", l.code())),
    );
    for file in files {
        let response = site.respond(&file);
        if response.status != 200 {
            return Err(io::Error::other(format!(
                "{file}: status {}",
                response.status
            )));
        }
        write(&file, &response.body)?;
    }
    write("offline/index.html", &site.respond("/offline/").body)?;
    write("sitemap.xml", site.sitemap().as_bytes())?;
    write("robots.txt", site.robots().as_bytes())?;
    for (url, asset) in site.assets.iter() {
        write(url, &std::fs::read(&asset.path)?)?;
    }
    // Served at the root too.
    for file in ["favicon.svg", "og.png"] {
        if let Some(asset) = site.assets.get(&format!("/static/{file}")) {
            write(file, &std::fs::read(&asset.path)?)?;
        }
    }
    // Files are served as they are: no Jekyll processing on GitHub Pages.
    write(".nojekyll", b"")?;
    Ok(written)
}
