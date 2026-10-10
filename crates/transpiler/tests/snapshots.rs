//! Snapshot tests: each `snapshots/NAME.rs` is translated and compared with
//! `snapshots/NAME.swift`. `FERROSWIFT_BLESS=1 cargo test --test snapshots` rewrites the
//! expected files after a deliberate change (review the diff!).

use std::path::Path;

use ferroswift_transpiler::{Options, transpile_with};

#[test]
fn translations_match_their_snapshots() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
    let bless = std::env::var_os("FERROSWIFT_BLESS").is_some();
    let options = Options {
        header: false,
        entry_point: false,
        previews: true,
    };
    let mut inputs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    inputs.sort();
    assert!(!inputs.is_empty());
    let mut failures = Vec::new();
    for input in inputs {
        let source = std::fs::read_to_string(&input).unwrap();
        let out = transpile_with(&source, &options);
        let mut actual = out.swift.clone();
        if !out.diagnostics.is_empty() {
            actual.push_str("\n// Diagnostics:\n");
            for d in &out.diagnostics {
                actual.push_str(&format!("// {d}\n"));
            }
        }
        let expected_path = input.with_extension("swift");
        if bless {
            std::fs::write(&expected_path, &actual).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&expected_path).unwrap_or_default();
        if expected != actual {
            let first = expected
                .lines()
                .zip(actual.lines())
                .position(|(e, a)| e != a)
                .unwrap_or_else(|| expected.lines().count().min(actual.lines().count()));
            failures.push(format!(
                "{} differs from line {}:\n  expected: {:?}\n  actual:   {:?}",
                expected_path.display(),
                first + 1,
                expected.lines().nth(first).unwrap_or(""),
                actual.lines().nth(first).unwrap_or("")
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{}\n(FERROSWIFT_BLESS=1 to accept the new output)",
        failures.join("\n\n")
    );
}
