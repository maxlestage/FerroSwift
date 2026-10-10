fn main() {
    let path = std::env::args().nth(1).expect("file");
    let src = std::fs::read_to_string(&path).expect("read");
    let out = ferroswift_transpiler::transpile(&src);
    print!("{}", out.swift);
    for d in &out.diagnostics {
        eprintln!("{d}");
    }
}
