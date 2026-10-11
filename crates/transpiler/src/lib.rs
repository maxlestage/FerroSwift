//! # FerroSwift transpiler
//!
//! Translates Rust source code into Swift. Ordinary Rust (structs, enums, traits, functions,
//! `match`, `Option`, `Result`, iterators, `format!`…) becomes idiomatic Swift; views written
//! with the [`ferroswift`](https://docs.rs/ferroswift) crate become SwiftUI.
//!
//! ```
//! let out = ferroswift_transpiler::transpile(r#"
//!     fn area(width: f64, height: f64) -> f64 {
//!         width * height
//!     }
//! "#);
//! assert!(out.swift.contains("func area(_ width: Double, _ height: Double) -> Double {"));
//! assert!(out.diagnostics.is_empty());
//! ```

mod comments;
mod diag;
mod expr;
mod format;
mod macros;
mod methods;
mod names;
mod pat;
mod stmt;
mod support;
mod swiftui;
mod symbols;
mod translate;
mod ty;

/// The version of this crate, e.g. `"0.1.0"` (the FerroSwift crates share it).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use diag::{Diagnostic, Severity};

use std::collections::BTreeSet;

use symbols::Symbols;
use translate::Translator;

/// Translation settings.
#[derive(Clone, Debug)]
pub struct Options {
    /// Call `main()` at the end of the file when it defines `fn main` (a `main.swift`).
    pub entry_point: bool,
    /// Translate `#[preview]` functions into `#Preview` blocks.
    pub previews: bool,
    /// Write the header comment naming the source file.
    pub header: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            entry_point: true,
            previews: true,
            header: true,
        }
    }
}

/// The translation of one file.
#[derive(Clone, Debug, Default)]
pub struct Output {
    /// The Swift source, support code included at the end when needed.
    pub swift: String,
    pub diagnostics: Vec<Diagnostic>,
}

impl Output {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }
}

/// A Rust source file of a crate.
#[derive(Clone, Debug)]
pub struct SourceFile {
    /// Path relative to the crate root, e.g. `src/views/row.rs`.
    pub path: String,
    pub source: String,
}

/// The translation of a crate.
#[derive(Clone, Debug, Default)]
pub struct CrateOutput {
    /// One Swift file per Rust file: (Swift file name, source).
    pub files: Vec<(String, String)>,
    /// The Rust file each of `files` comes from.
    pub file_sources: Vec<String>,
    /// The support code shared by the files (`FerroSwiftSupport.swift`), if any is needed.
    pub support: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    /// Files that are not valid Rust: they have no translation (keep the previous one).
    pub unparsed: Vec<String>,
}

/// Translates one Rust file into one Swift file, with default options.
pub fn transpile(source: &str) -> Output {
    transpile_with(source, &Options::default())
}

/// Translates one Rust file into one Swift file.
pub fn transpile_with(source: &str, options: &Options) -> Output {
    let file = match syn::parse_file(source) {
        Ok(file) => file,
        Err(error) => {
            return Output {
                swift: String::new(),
                diagnostics: vec![parse_error(&error, None)],
            };
        }
    };
    let symbols = Symbols::collect(&[&file]);
    let mut helpers = BTreeSet::new();
    let mut t = Translator::new(&symbols, options, source, None);
    let mut swift = t.file(&file, options.entry_point);
    helpers.extend(t.helpers.iter().copied());
    let mut diagnostics = std::mem::take(&mut t.diags);
    if let Some(code) = support::render(&helpers, symbols.ui || t.used_ui) {
        swift.push_str("\n// MARK: - FerroSwift support\n\n");
        swift.push_str(&code);
    }
    diagnostics.sort_by_key(|d| (d.line, d.column));
    // One message per position: the first, most precise one.
    diagnostics.dedup_by(|a, b| a.line == b.line && a.column == b.column);
    Output { swift, diagnostics }
}

/// Translates the files of a crate together: types, functions and methods declared in one
/// file are known in the others. `entry` is the file whose `fn main` starts the program
/// (`src/main.rs`), if any.
pub fn transpile_crate(
    files: &[SourceFile],
    entry: Option<&str>,
    options: &Options,
) -> CrateOutput {
    let mut out = CrateOutput::default();
    let mut parsed = Vec::new();
    for f in files {
        match syn::parse_file(&f.source) {
            Ok(file) => parsed.push((f, file)),
            Err(error) => {
                out.diagnostics.push(parse_error(&error, Some(&f.path)));
                out.unparsed.push(f.path.clone());
            }
        }
    }
    let refs: Vec<&syn::File> = parsed.iter().map(|(_, f)| f).collect();
    let symbols = Symbols::collect(&refs);
    let mut helpers = BTreeSet::new();
    let mut ui = symbols.ui;
    for (src, file) in &parsed {
        let mut t = Translator::new(&symbols, options, &src.source, Some(src.path.clone()));
        let is_entry = entry == Some(src.path.as_str());
        let swift = t.file(file, is_entry);
        helpers.extend(t.helpers.iter().copied());
        ui |= t.used_ui;
        out.diagnostics.append(&mut t.diags);
        out.files
            .push((swift_file_name(&src.path, is_entry), swift));
        out.file_sources.push(src.path.clone());
    }
    out.support = support::render(&helpers, ui).map(|code| {
        format!(
            "// Generated by FerroSwift: support code for the translated Rust. Do not edit.\n\nimport {}\n\n{code}",
            if ui { "SwiftUI" } else { "Foundation" }
        )
    });
    out.diagnostics
        .sort_by(|a, b| (&a.file, a.line, a.column).cmp(&(&b.file, b.line, b.column)));
    out.diagnostics
        .dedup_by(|a, b| a.file == b.file && a.line == b.line && a.column == b.column);
    out
}

/// `src/views/todo_row.rs` → `TodoRow.swift`; the entry file is `main.swift`.
pub fn swift_file_name(path: &str, entry: bool) -> String {
    if entry {
        return "main.swift".into();
    }
    let stem = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".rs");
    let stem = match stem {
        "mod" | "lib" | "main" => {
            let parts: Vec<&str> = path.split('/').collect();
            if stem == "mod" && parts.len() >= 2 {
                parts[parts.len() - 2]
            } else if stem == "lib" {
                "App"
            } else {
                "Main"
            }
        }
        s => s,
    };
    format!("{}.swift", names::upper_camel(stem))
}

fn parse_error(error: &syn::Error, file: Option<&str>) -> Diagnostic {
    let (line, column) = diag::position(error.span());
    Diagnostic {
        severity: Severity::Error,
        message: format!("Rust syntax error: {error}"),
        file: file.map(str::to_string),
        line,
        column,
    }
}
