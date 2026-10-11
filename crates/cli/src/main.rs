//! `ferroswift`: translates a Rust crate into Swift and SwiftUI.

use std::collections::{BTreeMap, HashSet};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime};

use ferroswift_transpiler::{Diagnostic, Options, Severity, SourceFile};

mod scaffold;

const HELP: &str = "\
ferroswift — write Rust, ship Swift and SwiftUI

Usage:
  ferroswift new <name>                 Create an app: a Rust crate and its Swift package
  ferroswift build [crate] [-o <dir>]   Translate src/**/*.rs into Swift files
  ferroswift watch [crate] [-o <dir>]   Translate again on every change
  ferroswift check [crate]              Only report what cannot be translated
  ferroswift file <file.rs> [-o <out>]  Translate one file (to stdout by default)

Options:
  -o, --out <dir>     Where to write the Swift files
                      (default: <crate>/swift/Sources/<Name> when <crate>/swift/Package.swift
                      exists, <crate>/swift otherwise)
  --no-previews       Leave out #[preview] functions (#Preview needs Xcode)
  -h, --help          Show this help
  -V, --version       Show the version
";

struct Args {
    command: String,
    path: Option<PathBuf>,
    out: Option<PathBuf>,
    previews: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let mut command = None;
    let mut path = None;
    let mut out = None;
    let mut previews = true;
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("ferroswift {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-o" | "--out" => out = Some(PathBuf::from(args.next().ok_or("-o needs a directory")?)),
            "--no-previews" => previews = false,
            s if s.starts_with('-') => return Err(format!("unknown option {s}")),
            s if command.is_none() => command = Some(s.to_string()),
            s if path.is_none() => path = Some(PathBuf::from(s)),
            s => return Err(format!("unexpected argument {s}")),
        }
    }
    let command = command.ok_or("missing command (try --help)")?;
    // `ferroswift app.rs` is `ferroswift file app.rs`.
    if command.ends_with(".rs") && path.is_none() {
        return Ok(Args {
            command: "file".into(),
            path: Some(PathBuf::from(command)),
            out,
            previews,
        });
    }
    Ok(Args {
        command,
        path,
        out,
        previews,
    })
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let options = Options {
        previews: args.previews,
        ..Options::default()
    };
    let result = match args.command.as_str() {
        "new" => match &args.path {
            Some(p) => scaffold::new_app(p).and_then(|()| {
                let created = Args {
                    command: "build".into(),
                    path: Some(p.clone()),
                    out: None,
                    previews: args.previews,
                };
                build(&created, &options, false).and_then(succeeded)?;
                eprintln!("\nNext:\n  cd {0}\n  ferroswift watch        # translate on every save\n  open swift/Package.swift  # or: cd swift && swift run (macOS)", p.display());
                Ok(())
            }),
            None => Err("ferroswift new needs a name".into()),
        },
        "build" => build(&args, &options, false).and_then(succeeded),
        "check" => build(&args, &options, true).and_then(succeeded),
        "watch" => watch(&args, &options),
        "file" => file(&args, &options),
        other => Err(format!("unknown command {other} (try --help)")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.is_empty() => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("{}: {e}", paint("error", Color::Red));
            ExitCode::FAILURE
        }
    }
}

/// A failed build was already reported: exit with an error, without a message.
fn succeeded(ok: bool) -> Result<(), String> {
    if ok { Ok(()) } else { Err(String::new()) }
}

/// `ferroswift file app.rs`.
fn file(args: &Args, options: &Options) -> Result<(), String> {
    let path = args
        .path
        .as_ref()
        .ok_or("ferroswift file needs a .rs file")?;
    let source = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let options = Options {
        source_path: Some(path.display().to_string()),
        ..options.clone()
    };
    let out = ferroswift_transpiler::transpile_with(&source, &options);
    for d in &out.diagnostics {
        eprint!("{}", render(d, &path.display().to_string(), &source));
    }
    match &args.out {
        Some(o) => std::fs::write(o, &out.swift).map_err(|e| format!("{}: {e}", o.display()))?,
        None => print!("{}", out.swift),
    }
    if out.has_errors() {
        Err(String::new())
    } else {
        Ok(())
    }
}

/// The Rust files of a crate (`src/**/*.rs`, or the file itself).
fn sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    let src = root.join("src");
    if !src.is_dir() {
        return Err(format!(
            "{} has no src/ directory: is it a Rust crate?",
            root.display()
        ));
    }
    let mut files = Vec::new();
    let mut dirs = vec![src];
    while let Some(dir) = dirs.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                dirs.push(p);
            } else if p.extension().is_some_and(|e| e == "rs") {
                files.push(p);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// The name of the crate (`[package] name`), as a Swift type name.
fn crate_name(root: &Path) -> String {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    let name = manifest
        .lines()
        .skip_while(|l| l.trim() != "[package]")
        .find_map(|l| {
            let l = l.trim();
            l.strip_prefix("name")
                .map(|r| r.trim_start())
                .and_then(|r| r.strip_prefix('='))
                .map(|r| r.trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| {
            root.canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "App".into())
        });
    name.split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn out_dir(root: &Path, args: &Args) -> PathBuf {
    if let Some(o) = &args.out {
        return o.clone();
    }
    let base = if root.is_file() {
        root.parent().unwrap_or(Path::new("."))
    } else {
        root
    };
    let swift = base.join("swift");
    if swift.join("Package.swift").is_file() {
        swift.join("Sources").join(crate_name(base))
    } else {
        swift
    }
}

/// Translates the crate; returns whether it had no errors.
fn build(args: &Args, options: &Options, check_only: bool) -> Result<bool, String> {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let started = Instant::now();
    let files = sources(&root)?;
    if files.is_empty() {
        return Err(format!("no Rust file in {}", root.display()));
    }
    let base = if root.is_file() {
        root.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        root.clone()
    };
    let mut inputs = Vec::new();
    for f in &files {
        let source = std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()))?;
        let rel = f
            .strip_prefix(&base)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        inputs.push(SourceFile { path: rel, source });
    }
    let entry = inputs
        .iter()
        .find(|f| f.path == "src/main.rs")
        .map(|f| f.path.clone());
    let out = ferroswift_transpiler::transpile_crate(&inputs, entry.as_deref(), options);
    let sources: BTreeMap<&str, &str> = inputs
        .iter()
        .map(|f| (f.path.as_str(), f.source.as_str()))
        .collect();
    for d in &out.diagnostics {
        let file = d.file.as_deref().unwrap_or("");
        let src = sources.get(file).copied().unwrap_or("");
        eprint!("{}", render(d, &base.join(file).display().to_string(), src));
    }
    let errors = out.diagnostics.iter().filter(|d| d.is_error()).count();
    let warnings = out.diagnostics.len() - errors;
    if check_only {
        report(errors, warnings, inputs.len(), None, started);
        return Ok(errors == 0);
    }
    let dir = out_dir(&root, args);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut written = HashSet::new();
    let mut names: BTreeMap<String, usize> = BTreeMap::new();
    for (name, _) in &out.files {
        *names.entry(name.clone()).or_default() += 1;
    }
    for ((name, swift), source) in out.files.iter().zip(&out.file_sources) {
        // Two `mod.rs`-like files with the same name: prefix with the folder.
        let name = if names[name] > 1 {
            let folder = Path::new(source)
                .parent()
                .and_then(|p| p.file_name())
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_default();
            format!("{}{name}", capitalize(&folder))
        } else {
            name.clone()
        };
        write_if_changed(&dir.join(&name), swift)?;
        written.insert(name);
    }
    if let Some(support) = &out.support {
        write_if_changed(&dir.join("FerroSwiftSupport.swift"), support)?;
        written.insert("FerroSwiftSupport.swift".into());
    }
    // Remove the files a previous build generated for Rust files that are gone (a file with
    // a syntax error keeps its last translation).
    for path in &out.unparsed {
        written.insert(ferroswift_transpiler::swift_file_name(
            path,
            entry.as_deref() == Some(path.as_str()),
        ));
    }
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".swift") && !written.contains(&name) {
                let generated = std::fs::read_to_string(e.path())
                    .is_ok_and(|s| s.starts_with("// Generated by FerroSwift"));
                if generated {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
    }
    report(errors, warnings, out.files.len(), Some(&dir), started);
    Ok(errors == 0)
}

fn write_if_changed(path: &Path, content: &str) -> Result<(), String> {
    if std::fs::read_to_string(path).is_ok_and(|old| old == content) {
        return Ok(());
    }
    std::fs::write(path, content).map_err(|e| format!("{}: {e}", path.display()))
}

fn report(errors: usize, warnings: usize, files: usize, dir: Option<&Path>, started: Instant) {
    let ms = started.elapsed().as_millis();
    let plural = |n: usize, w: &str| {
        if n == 1 {
            format!("{n} {w}")
        } else {
            format!("{n} {w}s")
        }
    };
    let status = if errors > 0 {
        paint(&plural(errors, "error"), Color::Red)
    } else {
        paint("ok", Color::Green)
    };
    let warnings = if warnings > 0 {
        format!(", {}", paint(&plural(warnings, "warning"), Color::Yellow))
    } else {
        String::new()
    };
    match dir {
        Some(d) => eprintln!(
            "{status}{warnings}: {} translated into {} in {ms} ms",
            plural(files, "file"),
            d.display()
        ),
        None => eprintln!(
            "{status}{warnings}: {} checked in {ms} ms",
            plural(files, "file")
        ),
    }
}

/// `ferroswift watch`: translates, then again after every change of a Rust file.
fn watch(args: &Args, options: &Options) -> Result<(), String> {
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let snapshot = |root: &Path| -> Vec<(PathBuf, Option<SystemTime>)> {
        sources(root)
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                let t = std::fs::metadata(&p).and_then(|m| m.modified()).ok();
                (p, t)
            })
            .collect()
    };
    let mut last = snapshot(&root);
    let _ = build(args, options, false)?;
    eprintln!(
        "watching {} for changes (Ctrl-C to stop)",
        root.join("src").display()
    );
    loop {
        std::thread::sleep(Duration::from_millis(300));
        let now = snapshot(&root);
        if now != last {
            last = now;
            eprintln!();
            if let Err(e) = build(args, options, false) {
                eprintln!("{}: {e}", paint("error", Color::Red));
            }
        }
    }
}

/// A diagnostic like rustc's: position, message, the source line and a caret.
fn render(d: &Diagnostic, path: &str, source: &str) -> String {
    let (label, color) = match d.severity {
        Severity::Error => ("error", Color::Red),
        Severity::Warning => ("warning", Color::Yellow),
    };
    let mut out = format!(
        "{}: {}\n  --> {path}:{}:{}\n",
        paint(label, color),
        d.message,
        d.line,
        d.column
    );
    if let Some(line) = source.lines().nth(d.line.saturating_sub(1)) {
        let n = d.line.to_string();
        let pad = " ".repeat(n.len());
        let caret_at: String = line
            .chars()
            .take(d.column.saturating_sub(1))
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        out.push_str(&format!(
            "{pad} |\n{n} | {line}\n{pad} | {caret_at}{}\n",
            paint("^", color)
        ));
    }
    out
}

#[derive(Clone, Copy)]
enum Color {
    Red,
    Yellow,
    Green,
}

fn paint(s: &str, color: Color) -> String {
    if !std::io::stderr().is_terminal() || std::env::var_os("NO_COLOR").is_some() {
        return s.to_string();
    }
    let code = match color {
        Color::Red => "1;31",
        Color::Yellow => "1;33",
        Color::Green => "1;32",
    };
    format!("\x1b[{code}m{s}\x1b[0m")
}

fn capitalize(s: &str) -> String {
    s.split(['_', '-'])
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}
