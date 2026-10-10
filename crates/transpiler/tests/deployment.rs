//! The generated Swift compiles in the Swift 6 language mode (strict concurrency) and, on macOS
//! with `FERROSWIFT_SDK=iphonesimulator` or `macosx`, for the oldest systems FerroSwift targets:
//! iOS 17 and macOS 14. Every program of `tests/programs`, every snapshot and every example app
//! is type-checked in both language modes.
//!
//! On other systems, the SwiftUI parts use the stand-in module of `tests/swiftui-shim`. Needs
//! `swiftc` (or `FERROSWIFT_SWIFTC`); skipped without it.

use std::path::{Path, PathBuf};
use std::process::Command;

use ferroswift_transpiler::{Options, SourceFile, transpile_crate, transpile_with};

fn swiftc() -> Option<PathBuf> {
    let candidate = std::env::var_os("FERROSWIFT_SWIFTC")
        .map_or_else(|| PathBuf::from("swiftc"), PathBuf::from);
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

/// A translation to type-check: its Swift files, and whether it has top-level code.
struct Unit {
    name: String,
    files: Vec<PathBuf>,
    library: bool,
}

/// One file (a program or a snapshot) translated into `dir`; `None` when it has errors (the
/// diagnostics snapshot).
fn single(rs: &Path, dir: &Path, library: bool, options: &Options) -> Option<Unit> {
    let name = rs.file_stem().unwrap().to_string_lossy().into_owned();
    let out = transpile_with(&std::fs::read_to_string(rs).unwrap(), options);
    if out.has_errors() {
        return None;
    }
    std::fs::create_dir_all(dir).unwrap();
    let p = dir.join(format!("{name}.swift"));
    std::fs::write(&p, &out.swift).unwrap();
    Some(Unit {
        name,
        files: vec![p],
        library,
    })
}

/// An example crate translated into `dir`.
fn example(root: &Path, dir: &Path, options: &Options) -> Unit {
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
    let out = transpile_crate(&sources, None, options);
    assert!(
        !out.diagnostics.iter().any(|d| d.is_error()),
        "{}: translation errors",
        root.display()
    );
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
    Unit {
        name: root.file_name().unwrap().to_string_lossy().into_owned(),
        files: written,
        library: true,
    }
}

#[test]
fn generated_swift_compiles_in_swift_6_and_for_the_oldest_targets() {
    let Some(swiftc) = swiftc() else {
        eprintln!("swiftc not found (set FERROSWIFT_SWIFTC): skipped");
        return;
    };
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.join("../..");
    let work = Path::new(env!("CARGO_TARGET_TMPDIR")).join("deployment");
    // `#Preview` is a macro of Apple's SwiftUI: the stand-in module has none.
    let sdk = std::env::var("FERROSWIFT_SDK").ok();
    let options = Options {
        previews: sdk.is_some(),
        ..Options::default()
    };

    let mut units = Vec::new();
    for (sub, library) in [("tests/programs", false), ("tests/snapshots", true)] {
        let mut files = Vec::new();
        rust_files(&manifest.join(sub), &mut files);
        files.sort();
        for rs in files {
            let dir = work.join(sub.rsplit('/').next().unwrap());
            units.extend(single(&rs, &dir, library, &options));
        }
    }
    let mut examples: Vec<PathBuf> = std::fs::read_dir(workspace.join("examples"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("src").is_dir())
        .collect();
    examples.sort();
    for e in &examples {
        let name = e.file_name().unwrap().to_string_lossy().into_owned();
        units.push(example(e, &work.join("examples").join(name), &options));
    }
    assert!(units.len() > 20, "{} units", units.len());

    // The SDK and its oldest supported system on macOS, the shim elsewhere.
    let mut base: Vec<String> = Vec::new();
    let target = match sdk {
        Some(sdk) => {
            let path = Command::new("xcrun")
                .args(["--sdk", &sdk, "--show-sdk-path"])
                .output()
                .expect("xcrun");
            let path = String::from_utf8_lossy(&path.stdout).trim().to_string();
            base.extend(["-sdk".into(), path]);
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "x86_64"
            };
            let target = if sdk.contains("iphone") {
                format!("{arch}-apple-ios17.0-simulator")
            } else {
                format!("{arch}-apple-macos14.0")
            };
            base.extend(["-target".into(), target.clone()]);
            target
        }
        None => {
            let shim = manifest.join("tests/swiftui-shim/SwiftUI.swift");
            let module_dir = work.join("shim");
            std::fs::create_dir_all(&module_dir).unwrap();
            let status = Command::new(&swiftc)
                .args([
                    "-emit-module",
                    "-module-name",
                    "SwiftUI",
                    "-parse-as-library",
                    "-o",
                ])
                .arg(module_dir.join("SwiftUI.swiftmodule"))
                .arg(&shim)
                .status()
                .expect("swiftc");
            assert!(status.success(), "the SwiftUI shim does not compile");
            base.extend(["-I".into(), module_dir.to_string_lossy().into_owned()]);
            "host".to_string()
        }
    };

    let mut failures = Vec::new();
    for mode in [&[][..], &["-swift-version", "6"][..]] {
        for unit in &units {
            let mut cmd = Command::new(&swiftc);
            cmd.arg("-typecheck");
            if unit.library {
                cmd.arg("-parse-as-library");
            }
            cmd.args(&base).args(mode).args(&unit.files);
            let out = cmd.output().expect("swiftc");
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let errors: Vec<&str> = stderr.lines().filter(|l| l.contains("error:")).collect();
                failures.push(format!(
                    "== {} ({target}{}) ==\n{}",
                    unit.name,
                    if mode.is_empty() { "" } else { ", Swift 6" },
                    errors.join("\n")
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
