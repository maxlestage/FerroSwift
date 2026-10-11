# ferroswift-cli

The `ferroswift` command of [FerroSwift](https://maxlestage.github.io/FerroSwift/).

```sh
cargo install --git https://github.com/maxlestage/FerroSwift ferroswift-cli

ferroswift new my-app        # a Rust crate and its Swift package
ferroswift build             # src/**/*.rs → swift/Sources/MyApp/*.swift
ferroswift watch             # the same on every save
ferroswift check             # only the diagnostics
ferroswift file app.rs       # one file to stdout
```
