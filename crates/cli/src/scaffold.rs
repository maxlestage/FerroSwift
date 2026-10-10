//! `ferroswift new`: a Rust crate for the app and the Swift package it is translated into.

use std::path::Path;

pub fn new_app(dir: &Path) -> Result<(), String> {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or("invalid name")?;
    let valid = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        return Err(format!(
            "`{name}` is not a valid crate name: use letters, digits, - and _"
        ));
    }
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    let crate_name = name.replace('-', "_").to_lowercase();
    let swift_name: String = name
        .split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect();
    let files: [(&str, String); 5] = [
        ("Cargo.toml", CARGO.replace("{crate}", &crate_name)),
        ("src/lib.rs", LIB.replace("{App}", &swift_name)),
        ("swift/Package.swift", PACKAGE.replace("{App}", &swift_name)),
        (".gitignore", GITIGNORE.to_string()),
        (
            "README.md",
            README
                .replace("{App}", &swift_name)
                .replace("{name}", &name),
        ),
    ];
    for (path, content) in files {
        let p = dir.join(path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&p, content).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    eprintln!("created {}", dir.display());
    Ok(())
}

const CARGO: &str = r#"[package]
name = "{crate}"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
ferroswift = { git = "https://github.com/maxlestage/FerroSwift" }
"#;

const LIB: &str = r#"//! The app, in Rust. `ferroswift build` (or `ferroswift watch`) writes its Swift
//! translation into swift/Sources/{App}.

use ferroswift::prelude::*;

/// The first screen.
#[derive(View, Default)]
pub struct ContentView {
    #[state]
    taps: u32,
}

impl View for ContentView {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(16.0)
            .child(system_image("hand.tap").font(Font::LargeTitle).foreground(Color::Accent))
            .child(text(format!("Tapped {} times", self.taps)).font(Font::Title2))
            .child(button("Tap me", || self.taps += 1).button_style(ButtonStyle::BorderedProminent))
            .padding()
    }
}

/// The app: one window.
pub struct {App}App;

impl App for {App}App {
    fn body(&mut self) -> impl Scene {
        window_group(ContentView::default())
    }
}
"#;

const PACKAGE: &str = r#"// swift-tools-version: 5.9
// The Swift package of the app. Its sources are generated from the Rust crate by FerroSwift.
import PackageDescription

let package = Package(
    name: "{App}",
    platforms: [.macOS(.v14), .iOS(.v17)],
    targets: [
        .executableTarget(name: "{App}", path: "Sources/{App}")
    ]
)
"#;

const GITIGNORE: &str = "target\nswift/.build\nswift/.swiftpm\n";

const README: &str = r#"# {name}

An app written in Rust and translated to SwiftUI by [FerroSwift](https://github.com/maxlestage/FerroSwift).

- `src/` — the app, in Rust. `cargo check` type-checks it, `cargo test` runs its tests.
- `swift/` — the Swift package. `swift/Sources/{App}` is generated: do not edit it.

```sh
ferroswift watch          # translate src/ into swift/Sources/{App} on every save
cd swift && swift run     # run it on macOS
```

For iOS, create an app project in Xcode and add the `swift/Sources/{App}` folder to it
(a synchronized folder: files added by FerroSwift appear by themselves).
"#;
