//! Static files (`site/static`) and the browser bundle (`site/pkg`): listed once with a
//! content hash for cache busting, read from disk when served or exported.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct Asset {
    /// The file on disk.
    pub path: PathBuf,
    pub content_type: &'static str,
    /// Short content hash, used in `?v=` so files can be cached forever.
    pub version: String,
}

#[derive(Default)]
pub struct Assets {
    files: BTreeMap<String, Asset>,
}

/// The `Content-Type` of a file, by extension.
pub fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

/// FNV-1a, enough to tell versions apart.
pub fn hash(bytes: &[u8]) -> String {
    let h = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{h:016x}")[..10].to_string()
}

impl Assets {
    /// Lists every file of `dir` (recursively) under the URL prefix `prefix`.
    pub fn load(&mut self, dir: &Path, prefix: &str) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                self.load(&path, &format!("{prefix}/{name}"));
            } else if let Ok(bytes) = std::fs::read(&path) {
                self.files.insert(
                    format!("{prefix}/{name}"),
                    Asset {
                        path,
                        content_type: content_type(&name),
                        version: hash(&bytes),
                    },
                );
            }
        }
    }

    pub fn get(&self, url: &str) -> Option<&Asset> {
        self.files.get(url)
    }

    /// The link to `url`, behind the base path: `url?v=<hash>` for files that exist.
    pub fn url(&self, url: &str) -> String {
        let link = crate::routes::link(url);
        match self.files.get(url) {
            Some(asset) => format!("{link}?v={}", asset.version),
            None => link,
        }
    }

    /// Every file, with its URL.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Asset)> {
        self.files.iter().map(|(url, asset)| (url.as_str(), asset))
    }
}
