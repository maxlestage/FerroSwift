//! `cargo run -p ferroswift-site -- export dist`: the whole site as static files in `dist/`,
//! for GitHub Pages or any static host. `BASE_PATH` (e.g. `/FerroSwift`) prefixes every link,
//! `SITE_URL` (e.g. `https://maxlestage.github.io`) makes the absolute URLs.
//!
//! `cargo run -p ferroswift-site` (or `-- serve`): the development server on `PORT` (3000 by
//! default). Build the browser bundle first for a live playground: see site/README.md.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use ferroswift_site::server;

    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("export") => {
            let dir = args.next().unwrap_or_else(|| "dist".into());
            let base = std::env::var("BASE_PATH").unwrap_or_default();
            let site_url = std::env::var("SITE_URL").ok();
            match server::export::export(std::path::Path::new(&dir), &base, site_url) {
                Ok(files) => println!("{files} files written to {dir}"),
                Err(error) => {
                    eprintln!("export failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        Some("serve") | None => {
            let port = std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000);
            let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
            if let Err(error) = server::dev::serve(&host, port) {
                eprintln!("server failed: {error}");
                std::process::exit(1);
            }
        }
        Some(other) => {
            eprintln!(
                "unknown command {other:?}\nusage: ferroswift-site [serve]\n       ferroswift-site export <dir>"
            );
            std::process::exit(2);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
