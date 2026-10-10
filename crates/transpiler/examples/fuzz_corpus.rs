//! Runs the translator on many Rust files and reports the ones that make it panic:
//! `find … -name '*.rs' | cargo run --release -p ferroswift-transpiler --example fuzz_corpus`.

use std::collections::BTreeMap;
use std::panic;
use std::sync::{Arc, Mutex};

fn main() {
    let mut files: Vec<String> = std::env::args().skip(1).collect();
    if files.is_empty() {
        files = std::io::read_to_string(std::io::stdin())
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect();
    }
    let last: Arc<Mutex<String>> = Arc::default();
    let hook_last = last.clone();
    panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let message = info
            .payload()
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        *hook_last.lock().unwrap() = format!("{location}: {message}");
    }));
    let mut causes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total = 0;
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        if src.len() > 200_000 {
            continue;
        }
        total += 1;
        if std::env::var_os("FUZZ_TRACE").is_some() {
            eprintln!("{f}");
        }
        if panic::catch_unwind(|| ferroswift_transpiler::transpile(&src)).is_err() {
            causes
                .entry(last.lock().unwrap().clone())
                .or_default()
                .push(f.clone());
        }
    }
    let panics: usize = causes.values().map(Vec::len).sum();
    for (cause, files) in &causes {
        println!("{} × {cause}\n    e.g. {}", files.len(), files[0]);
    }
    println!("{total} files, {panics} panics");
}
