# ferroswift-macros

The derives and attributes of [FerroSwift](https://maxlestage.github.io/FerroSwift/):
`#[derive(View)]`, `#[derive(Observable)]` and `#[preview]`. They are markers read by the
Rust → Swift transpiler; the `ferroswift` crate re-exports them, so you never depend on this
crate directly.

```toml
[dependencies]
ferroswift = { git = "https://github.com/maxlestage/FerroSwift" }
```

Proprietary software: © 2026 Maxime Nathan Lestage, all rights reserved. Free to use as a
dependency of your own programs, commercial ones included; see
[LICENSE](https://maxlestage.github.io/FerroSwift/LICENSE.txt).
