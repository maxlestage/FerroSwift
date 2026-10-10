# ferroswift-transpiler

The engine of [FerroSwift](https://github.com/maxlestage/FerroSwift): Rust source in, idiomatic Swift (and SwiftUI) out. Pure Rust, it also runs in the browser compiled to WebAssembly.

```rust
let out = ferroswift_transpiler::transpile("fn double(x: i32) -> i32 { x * 2 }");
assert_eq!(out.swift.lines().last(), Some("}"));
for d in &out.diagnostics {
    eprintln!("{d}"); // line:column: error: …
}
```

- `transpile(source)` / `transpile_with(source, &Options)`: one file, support code appended when needed.
- `transpile_crate(&files, entry, &Options)`: the files of a crate translated together (types and functions of one file are known in the others), with the support code in a separate file.

How it works: `syn` parses the file; a first pass collects the declarations of the whole crate (types, fields, enum variants, function signatures); the translation then walks the syntax tree with a light type inference (locals, fields, method results, closure parameters) that picks the right Swift for each construct — `v.len()` is `v.count` for a vector and `s.utf8.count` for a string, `a % b` is `truncatingRemainder` for floats, `Result` functions throw, an integer of another type is converted as Rust's inference would have typed it.
