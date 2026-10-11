//! `ferroswift new` from a user's point of view: the Rust crate type-checks with `cargo`, its
//! translation type-checks as Swift and, on macOS (`FERROSWIFT_SDK=macosx`), the Swift package
//! builds with `swift build`.
//!
//! The crate depends on ferroswift from crates.io; the test points it at the checkout instead.
//! Needs `swiftc` (or `FERROSWIFT_SWIFTC`) for the Swift parts; skipped without it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn swiftc() -> Option<PathBuf> {
    let candidate = std::env::var_os("FERROSWIFT_SWIFTC")
        .map_or_else(|| PathBuf::from("swiftc"), PathBuf::from);
    Command::new(&candidate)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
        .then_some(candidate)
}

fn run(cmd: &mut Command) -> String {
    let out = cmd.output().unwrap_or_else(|e| panic!("{cmd:?}: {e}"));
    assert!(
        out.status.success(),
        "{cmd:?} failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_new_app_checks_builds_and_translates() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let work = Path::new(env!("CARGO_TARGET_TMPDIR")).join("scaffold");
    // Outside this workspace: cargo would otherwise take the new crate for a member of it.
    let app = std::env::temp_dir().join(format!("ferroswift-new-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&app);
    std::fs::create_dir_all(&work).unwrap();
    let ferroswift = env!("CARGO_BIN_EXE_ferroswift");

    run(Command::new(ferroswift).arg("new").arg(&app));
    let name = app.file_name().unwrap().to_string_lossy().into_owned();
    // `ferroswift-new-123` → `FerroswiftNew123`.
    let swift_name: String = name
        .split('-')
        .map(|w| w[..1].to_uppercase() + &w[1..])
        .collect();
    let sources = app.join("swift/Sources").join(&swift_name);
    for f in [
        "Cargo.toml",
        "src/lib.rs",
        "swift/Package.swift",
        ".gitignore",
        "README.md",
    ] {
        assert!(app.join(f).is_file(), "{f} was not created");
    }
    assert!(
        sources.join("App.swift").is_file(),
        "App.swift was not created"
    );

    // The crate: `cargo check`, against this checkout's ferroswift instead of crates.io's.
    let manifest = app.join("Cargo.toml");
    let toml = std::fs::read_to_string(&manifest).unwrap();
    let dep = toml
        .lines()
        .find(|l| l.starts_with("ferroswift = "))
        .expect("the ferroswift dependency");
    assert_eq!(
        dep,
        "ferroswift = { git = \"https://github.com/maxlestage/FerroSwift\" }"
    );
    let local = format!(
        "ferroswift = {{ path = {:?} }}",
        workspace.join("crates/ferroswift").canonicalize().unwrap()
    );
    std::fs::write(&manifest, toml.replace(dep, &local)).unwrap();
    run(Command::new("cargo")
        .args(["check", "-q", "--manifest-path"])
        .arg(&manifest)
        .env("CARGO_TARGET_DIR", work.join("target")));

    // The translation, again, and the diagnostics.
    run(Command::new(ferroswift).arg("check").arg(&app));
    let out = run(Command::new(ferroswift).arg("build").arg(&app));
    assert!(out.is_empty(), "build writes nothing on stdout: {out}");
    let swift = std::fs::read_to_string(sources.join("App.swift")).unwrap();
    assert!(
        swift.contains(&format!("@main\nstruct {swift_name}App: App {{")),
        "{swift}"
    );
    assert!(swift.contains("struct ContentView: View {"), "{swift}");

    let Some(swiftc) = swiftc() else {
        eprintln!("swiftc not found (set FERROSWIFT_SWIFTC): the Swift package is not checked");
        return;
    };
    let files: Vec<PathBuf> = std::fs::read_dir(&sources)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    match std::env::var("FERROSWIFT_SDK").ok() {
        // macOS: the package builds as the README says (`cd swift && swift run`).
        Some(sdk) if sdk == "macosx" => {
            run(Command::new("swift")
                .args(["build", "--package-path"])
                .arg(app.join("swift")));
        }
        Some(sdk) => {
            let path = run(Command::new("xcrun").args(["--sdk", &sdk, "--show-sdk-path"]));
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "x86_64"
            };
            run(Command::new(&swiftc)
                .args(["-typecheck", "-parse-as-library", "-sdk", path.trim()])
                .args(["-target", &format!("{arch}-apple-ios17.0-simulator")])
                .args(&files));
        }
        // Elsewhere: against the stand-in SwiftUI module of the transpiler's tests.
        None => {
            let shim = workspace.join("crates/transpiler/tests/swiftui-shim/SwiftUI.swift");
            let module_dir = work.join("shim");
            std::fs::create_dir_all(&module_dir).unwrap();
            run(Command::new(&swiftc)
                .args([
                    "-emit-module",
                    "-module-name",
                    "SwiftUI",
                    "-parse-as-library",
                    "-o",
                ])
                .arg(module_dir.join("SwiftUI.swiftmodule"))
                .arg(&shim));
            run(Command::new(&swiftc)
                .args(["-typecheck", "-parse-as-library", "-I"])
                .arg(&module_dir)
                .args(&files));
        }
    }
    let _ = std::fs::remove_dir_all(&app);
}
