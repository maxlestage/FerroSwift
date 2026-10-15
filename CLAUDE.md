# Working on FerroSwift

- **Branch:** work on `master` and push it when a piece of work is finished — every time,
  without waiting to be asked. No pull request is needed.
- **Before pushing**, run what CI runs: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo clippy -p ferroswift-transpiler -p ferroswift-site --lib --target wasm32-unknown-unknown -- -D warnings`,
  and `cargo test --workspace` (with `swiftc` on the `PATH`, or `FERROSWIFT_SWIFTC` pointing
  at it, so that the Rust/Swift comparisons run).
- **CI** (`.github/workflows/ci.yml`) also type-checks the translations against Apple's SwiftUI
  on macOS, for iOS 17 and macOS 14, in Swift 5 and Swift 6 modes, and builds a new app made
  with `ferroswift new`. The site (`site/`) is built and tested by CI; it is not published on
  GitHub Pages (GitHub only hosts the code).
- **Tests of the transpiler:** `crates/transpiler/tests/programs/*.rs` are run with `rustc`
  and, translated, with `swiftc`; their outputs must be identical. Add a program there for any
  new piece of Rust that should translate. Snapshots are blessed with `FERROSWIFT_BLESS=1`.
- Reply to the user in French; code, comments and commit messages are in English.
