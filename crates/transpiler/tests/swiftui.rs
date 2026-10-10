//! The example apps (`examples/*`, Rust crates written with the ferroswift builders) are
//! translated and type-checked as Swift.
//!
//! On Linux, against `tests/swiftui-shim/SwiftUI.swift`, a stand-in with the signatures of the
//! real SwiftUI. On macOS with `FERROSWIFT_SDK=iphonesimulator`, against Apple's SwiftUI.
//! Needs `swiftc` (or `FERROSWIFT_SWIFTC`); skipped without it.

use std::path::{Path, PathBuf};
use std::process::Command;

use ferroswift_transpiler::{Options, SourceFile, transpile_crate};

fn swiftc() -> Option<PathBuf> {
    let candidate = std::env::var_os("FERROSWIFT_SWIFTC").map_or_else(|| PathBuf::from("swiftc"), PathBuf::from);
    Command::new(&candidate)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
        .then_some(candidate)
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read dir").flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// Translates the crate at `root` into `dir`; returns the Swift files.
fn translate(root: &Path, dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    files.sort();
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|f| SourceFile {
            path: f.strip_prefix(root).unwrap().to_string_lossy().into_owned(),
            source: std::fs::read_to_string(f).unwrap(),
        })
        .collect();
    let options = Options {
        previews: false,
        ..Options::default()
    };
    let out = transpile_crate(&sources, None, &options);
    let errors: Vec<String> = out.diagnostics.iter().filter(|d| d.is_error()).map(ToString::to_string).collect();
    assert!(errors.is_empty(), "{}: translation errors:\n{}", root.display(), errors.join("\n"));
    std::fs::create_dir_all(dir).unwrap();
    let mut written = Vec::new();
    for (name, swift) in &out.files {
        let p = dir.join(name);
        std::fs::write(&p, swift).unwrap();
        written.push(p);
    }
    if let Some(support) = &out.support {
        let p = dir.join("FerroSwiftSupport.swift");
        std::fs::write(&p, support).unwrap();
        written.push(p);
    }
    written
}

#[test]
fn example_apps_type_check_as_swiftui() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let work = Path::new(env!("CARGO_TARGET_TMPDIR")).join("swiftui");
    let mut examples: Vec<PathBuf> = std::fs::read_dir(workspace.join("examples"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("src").is_dir())
        .collect();
    examples.sort();
    assert!(!examples.is_empty());
    let translated: Vec<(PathBuf, Vec<PathBuf>)> = examples
        .iter()
        .map(|e| {
            let name = e.file_name().unwrap().to_string_lossy().into_owned();
            let files = translate(e, &work.join(&name));
            (e.clone(), files)
        })
        .collect();
    let Some(swiftc) = swiftc() else {
        eprintln!("swiftc not found (set FERROSWIFT_SWIFTC): only checking the translations");
        return;
    };
    // The SDK's SwiftUI on macOS, the shim elsewhere.
    let sdk = std::env::var("FERROSWIFT_SDK").ok();
    let mut base: Vec<String> = Vec::new();
    match &sdk {
        Some(sdk) => {
            let path = Command::new("xcrun").args(["--sdk", sdk, "--show-sdk-path"]).output().expect("xcrun");
            let path = String::from_utf8_lossy(&path.stdout).trim().to_string();
            base.extend(["-sdk".into(), path]);
            if sdk.contains("iphone") {
                let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };
                base.extend(["-target".into(), format!("{arch}-apple-ios17.0-simulator")]);
            } else {
                base.extend(["-target".into(), "arm64-apple-macos14.0".into()]);
            }
        }
        None => {
            let shim = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/swiftui-shim/SwiftUI.swift");
            let module_dir = work.join("shim");
            std::fs::create_dir_all(&module_dir).unwrap();
            let status = Command::new(&swiftc)
                .args(["-emit-module", "-module-name", "SwiftUI", "-parse-as-library", "-o"])
                .arg(module_dir.join("SwiftUI.swiftmodule"))
                .arg(&shim)
                .status()
                .expect("swiftc");
            assert!(status.success(), "the SwiftUI shim does not compile");
            base.extend(["-I".into(), module_dir.to_string_lossy().into_owned()]);
        }
    }
    let mut failures = Vec::new();
    for (example, files) in &translated {
        let out = Command::new(&swiftc)
            .args(["-typecheck", "-parse-as-library"])
            .args(&base)
            .args(files)
            .output()
            .expect("swiftc");
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let errors: Vec<&str> = stderr.lines().filter(|l| l.contains("error:")).collect();
            failures.push(format!("== {} ==\n{}", example.display(), errors.join("\n")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
