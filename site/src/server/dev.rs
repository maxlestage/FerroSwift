//! `ferroswift-site serve`: a small development server with no dependencies. It answers
//! with the same [`Site::respond`] as the static export, reloading the docs and the list of
//! files on every page, so edits show up without a restart.

use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use super::{Response, Site, site_dir};
use crate::routes::{base, set_base};

struct Config {
    site_url: String,
}

/// Serves the site on `host:port` until the process is stopped.
pub fn serve(host: &str, port: u16) -> io::Result<()> {
    set_base(&std::env::var("BASE_PATH").unwrap_or_default());
    let site_url = std::env::var("SITE_URL").unwrap_or_else(|_| format!("http://localhost:{port}"));
    let config = Arc::new(Config { site_url });
    let listener = TcpListener::bind((host, port))?;
    if !Site::load(&site_dir(), Some(config.site_url.clone())).has_bundle() {
        eprintln!(
            "warning: site/pkg is missing: the playground will not be interactive (see site/README.md)"
        );
    }
    println!("FerroSwift site on http://localhost:{port}{}/", base());
    for stream in listener.incoming().flatten() {
        let config = config.clone();
        std::thread::spawn(move || {
            let _ = handle(&config, stream);
        });
    }
    Ok(())
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        301 => "Moved Permanently",
        302 => "Found",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Unknown",
    }
}

fn handle(config: &Config, mut stream: TcpStream) -> io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut request = String::new();
    reader.read_line(&mut request)?;
    // Headers are not used: read them so the client sees a complete exchange.
    let mut header = String::new();
    while reader.read_line(&mut header)? > 2 {
        header.clear();
    }
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/").to_string();
    let response = match method {
        "GET" | "HEAD" => respond(config, &target),
        _ => Response {
            status: 405,
            content_type: "text/plain; charset=utf-8",
            body: b"method not allowed".to_vec(),
            location: None,
        },
    };
    let mut head = format!(
        "HTTP/1.1 {} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\ncache-control: no-cache\r\nx-content-type-options: nosniff\r\nconnection: close\r\n",
        response.status,
        status_text(response.status),
        response.content_type,
        response.body.len()
    );
    if let Some(location) = &response.location {
        head.push_str(&format!("location: {location}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    if method != "HEAD" {
        stream.write_all(&response.body)?;
    }
    stream.flush()
}

/// The response to `target` (a path with its query string), behind the base path.
fn respond(config: &Config, target: &str) -> Response {
    let redirect = |to: String| Response {
        status: 302,
        content_type: "text/plain; charset=utf-8",
        body: Vec::new(),
        location: Some(to),
    };
    let path = match target.strip_prefix(base()) {
        _ if base().is_empty() => target,
        Some("") => return redirect(format!("{}/", base())),
        Some(rest) if rest.starts_with('/') || rest.starts_with('?') => rest,
        _ if target == "/" => return redirect(format!("{}/", base())),
        _ => "/404/",
    };
    let site = Site::load(&site_dir(), Some(config.site_url.clone()));
    site.respond(path)
}
