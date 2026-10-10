//! Differential tests: every program of `tests/programs` is compiled and run twice, as Rust
//! with `rustc` and as Swift (after translation) with `swiftc`. Their outputs must be equal.
//!
//! Needs a Swift compiler: `swiftc` on the PATH, or its path in `FERROSWIFT_SWIFTC`.
//! Without one, the test only checks that every program translates without errors.
//! `FERROSWIFT_PROGRAM=name` runs a single program.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

fn swiftc() -> Option<PathBuf> {
    let candidate = std::env::var_os("FERROSWIFT_SWIFTC")
        .map_or_else(|| PathBuf::from("swiftc"), PathBuf::from);
    let ok = Command::new(&candidate)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    ok.then_some(candidate)
}

fn programs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let only = std::env::var("FERROSWIFT_PROGRAM").ok();
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("tests/programs")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .filter(|p| {
            only.as_deref()
                .is_none_or(|o| p.file_stem().is_some_and(|s| s == o))
        })
        .collect();
    files.sort();
    files
}

fn run(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The error messages of the compiler, with the line they point at.
fn errors_only(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if l.contains(": error:") {
            out.push(l.trim().to_string());
            if let Some(code) = lines.get(i + 2) {
                out.push(format!("    {}", code.trim()));
            }
        }
    }
    if out.is_empty() {
        output.to_string()
    } else {
        out.join("\n")
    }
}

fn check(program: &Path, swiftc: Option<&Path>, work: &Path) -> Result<(), String> {
    let name = program.file_stem().unwrap().to_string_lossy().into_owned();
    let source = std::fs::read_to_string(program).map_err(|e| e.to_string())?;
    let out = ferroswift_transpiler::transpile(&source);
    let errors: Vec<String> = out
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(ToString::to_string)
        .collect();
    let swift_path = work.join(format!("{name}.swift"));
    std::fs::write(&swift_path, &out.swift).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!("translation errors:\n{}", errors.join("\n")));
    }
    let Some(swiftc) = swiftc else { return Ok(()) };
    let rust_bin = work.join(format!("{name}_rust"));
    run(Command::new("rustc")
        .args(["--edition", "2024", "-A", "warnings", "-o"])
        .arg(&rust_bin)
        .arg(program))
    .map_err(|e| format!("rustc failed:\n{e}"))?;
    let expected =
        run(&mut Command::new(&rust_bin)).map_err(|e| format!("the Rust program failed:\n{e}"))?;
    let swift_bin = work.join(format!("{name}_swift"));
    run(Command::new(swiftc)
        .arg(&swift_path)
        .arg("-o")
        .arg(&swift_bin))
    .map_err(|e| {
        format!(
            "swiftc failed on {}:\n{}",
            swift_path.display(),
            errors_only(&e)
        )
    })?;
    let actual = run(&mut Command::new(&swift_bin))
        .map_err(|e| format!("the Swift program failed:\n{e}"))?;
    if expected != actual {
        let mut diff = String::new();
        for (i, (e, a)) in expected.lines().zip(actual.lines()).enumerate() {
            if e != a {
                diff.push_str(&format!("line {}:\n  rust:  {e}\n  swift: {a}\n", i + 1));
            }
        }
        let (el, al) = (expected.lines().count(), actual.lines().count());
        if el != al {
            diff.push_str(&format!("{el} lines from Rust, {al} from Swift\n"));
        }
        return Err(format!(
            "different output ({}):\n{diff}",
            swift_path.display()
        ));
    }
    Ok(())
}

#[test]
fn programs_behave_the_same_in_rust_and_swift() {
    let swiftc = swiftc();
    if swiftc.is_none() {
        eprintln!("swiftc not found (set FERROSWIFT_SWIFTC): only checking the translations");
    }
    let work = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential");
    std::fs::create_dir_all(&work).unwrap();
    let programs = programs();
    assert!(!programs.is_empty());
    let failures = Mutex::new(Vec::new());
    let next = Mutex::new(0usize);
    let threads = std::thread::available_parallelism()
        .map_or(2, |n| n.get())
        .min(6);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let i = {
                        let mut n = next.lock().unwrap();
                        let i = *n;
                        *n += 1;
                        i
                    };
                    let Some(p) = programs.get(i) else { break };
                    if let Err(e) = check(p, swiftc.as_deref(), &work) {
                        failures
                            .lock()
                            .unwrap()
                            .push(format!("== {} ==\n{e}", p.display()));
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} of {} programs failed:\n\n{}",
        failures.len(),
        programs.len(),
        failures.join("\n\n")
    );
}
