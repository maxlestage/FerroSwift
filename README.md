# FerroSwift

**English** · [Français](README.fr.md)

**Write Rust. Ship Swift and SwiftUI.** FerroSwift translates Rust into idiomatic Swift: the logic of your app *and* its SwiftUI views, type-checked by `rustc`, tested with `cargo test`, opened in Xcode.

```rust
use ferroswift::prelude::*;

#[derive(View, Default)]
pub struct Counter {
    #[state]
    count: i32,
}

impl View for Counter {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(12.0)
            .child(text(format!("Count: {}", self.count)).font(Font::LargeTitle))
            .child(button("+", || self.count += 1).button_style(ButtonStyle::BorderedProminent))
            .padding()
    }
}
```

`ferroswift build` writes:

```swift
import SwiftUI

struct Counter: View {
    @State var count = 0

    var body: some View {
        VStack(spacing: 12) {
            Text("Count: \(count)").font(.largeTitle)
            Button("+") { count += 1 }
                .buttonStyle(.borderedProminent)
        }
        .padding()
    }
}
```

The views are written with ordinary Rust functions (`vstack()`, `text()`, `button()`…) from the [`ferroswift`](crates/ferroswift) crate: `cargo check` and rust-analyzer know them, and each one documents the SwiftUI code it becomes. Everything else is plain Rust — structs, enums, traits, `match`, `Option`, `Result`, iterators — translated into the Swift you would write by hand.

📚 **Website, documentation and live playground** (the transpiler compiled to WebAssembly, in your browser): [`site/`](site), written with [Active](https://maxlestage.github.io/Active/). Run it with `cargo run -p ferroswift-site`.

## Why

- **Only Rust.** One language for the model and the views, the Rust tooling for both: the compiler, Clippy, `cargo test` for the logic, rust-analyzer in the editor.
- **Idiomatic Swift.** Structs and enums with payloads, `extension`s for `impl` blocks, protocols for traits, `switch` with `where`, `guard let`, typed `throws` for `Result`, `@State`, `@Binding`, `@Observable`. Comments and doc comments are kept. No runtime: a small support file only when the Rust code needs it.
- **The same behavior.** The test suite runs every sample program twice, compiled by `rustc` and, translated, by `swiftc`, and compares the outputs byte for byte: `{:?}` and `{:.2}` formatting, padding, integer casts, `BTreeMap` order, `?` and early returns.
- **Honest diagnostics.** What cannot be translated is reported like `rustc` does, with its file, line and column, and the Swift output still compiles (a `fatalError` marks the spot).
- **Fast.** A crate translates in milliseconds; `ferroswift watch` translates again on every save.

## Install

```sh
cargo install --git https://github.com/maxlestage/FerroSwift ferroswift-cli
```

## Quick start

```sh
ferroswift new hello      # a Rust crate (src/lib.rs) and its Swift package (swift/)
cd hello
ferroswift watch          # translates src/ into swift/Sources/Hello on every save
cd swift && swift run     # macOS: the app opens
```

For iOS, create an app in Xcode and add the `swift/Sources/Hello` folder to it: with Xcode's synchronized folders, files that FerroSwift adds or removes follow by themselves.

| Command | |
| --- | --- |
| `ferroswift new <name>` | A new app: `Cargo.toml`, `src/lib.rs`, `swift/Package.swift` |
| `ferroswift build [crate] [-o dir]` | Translates `src/**/*.rs`, one Swift file per Rust file, plus `FerroSwiftSupport.swift` when needed |
| `ferroswift watch [crate]` | The same, again after every change |
| `ferroswift check [crate]` | Only the diagnostics |
| `ferroswift file app.rs` | One file, to stdout |

## SwiftUI from Rust

| Rust | Swift |
| --- | --- |
| `#[derive(View)] struct X` + `impl View for X { fn body(&mut self) -> impl View }` | `struct X: View { var body: some View }` |
| `#[state] count: i32` · `#[state(5)]` | `@State var count = 0` · `@State var count = 5` |
| `#[binding] value: bool` | `@Binding var value: Bool` |
| `toggle("On", &mut self.on)` | `Toggle("On", isOn: $on)` |
| `#[derive(Observable)] struct Store` | `@Observable final class Store` |
| `vstack().spacing(8.0).child(a).child(b)` | `VStack(spacing: 8) { a; b }` |
| `for_each(&self.items, \|item\| row)` | `ForEach(items) { item in row }` |
| `for_each(&mut self.items, \|item\| toggle(…, &mut item.done))` | `ForEach($items) { $item in Toggle(…, isOn: $item.done) }` |
| `.child(if c { a } else { b })` | `if c { a } else { b }` |
| `.font(Font::Title).foreground(Color::Red)` | `.font(.title).foregroundStyle(.red)` |
| `.fill_width().frame_alignment(Alignment::Leading)` | `.frame(maxWidth: .infinity, alignment: .leading)` |
| `.sheet(&mut self.show, view)` | `.sheet(isPresented: $show) { view }` |
| `impl App for MyApp { fn body(&mut self) -> impl Scene { window_group(V::default()) } }` | `@main struct MyApp: App { var body: some Scene { WindowGroup { V() } } }` |
| `#[preview] fn preview() -> impl View` | `#Preview { … }` |

Stacks, lists, forms, sections, navigation, tabs, buttons, text fields, toggles, sliders, steppers, pickers, images, labels, shapes, progress views and about sixty modifiers: see [the SwiftUI guide](site/docs/en/swiftui.md) and the [examples](examples).

## Rust → Swift

| Rust | Swift |
| --- | --- |
| `i32`, `i64`, `isize`, `usize` · `u8` … `u64` · `f64` · `f32` | `Int` · `UInt8` … `UInt64` · `Double` · `Float` |
| `String`, `&str` · `char` · `Vec<T>`, `&[T]` · `HashMap<K, V>` · `HashSet<T>` | `String` · `Character` · `[T]` · `[K: V]` · `Set<T>` |
| `Option<T>` · `Box<T>`, `Rc<T>` | `T?` · `T` |
| `fn f() -> Result<T, E>` · `f()?` · `Err(e)` | `func f() throws(E) -> T` · `try f()` · `throw e` |
| `struct` · `enum` with payloads · `impl X` · `trait` · `impl Trait for X` | `struct` · `enum` with associated values (`indirect` when recursive) · `extension X` · `protocol` · `extension X: Trait` |
| `fn new(…) -> Self` · `&mut self` · `mut self` | `init(…)` · `mutating func` · a modified copy |
| `#[derive(PartialEq, Hash, PartialOrd)]` · `impl Display` · `impl Add` | `Equatable`, `Hashable`, `Comparable` · `CustomStringConvertible` · `static func +` |
| `match` · `if let` · `let … else` · `while let` · `for x in &mut v` | `switch` · `if let` / `if case` · `guard` · `while let` · a loop on the indices |
| `\|x\| x + 1` · `impl Fn(i32) -> i32` | `{ x in x + 1 }` · `(Int) -> Int` |
| `v.iter().filter(…).map(…).collect()` · `.sum()` · `.fold(…)` | `v.filter { … }.map { … }` · `.reduce(0, +)` · `.reduce(…) { … }` |
| `format!("{name}: {:>8.2}", x)` · `println!` · `{:?}` | string interpolation · `print` · `rsDebug(x)` (Rust's format) |

The complete list, and what is not supported, is in [the language guide](site/docs/en/language.md).

## Limits

- No borrow checker semantics in Swift: values are copied, and a `&mut` kept in a struct reads and writes the place it was taken from. `Rc<RefCell<T>>`, `Cell<T>` and `Arc<Mutex<T>>` become a class holding the value, shared as in Rust; the locks never block. `Rc::strong_count`, `Weak` and `Rc::downgrade` are counted as in Rust.
- `i32`, `i64` and `usize` all become Swift's 64-bit `Int`: `+` overflows at 2⁶³, not 2³¹ (parsing, `as` casts and the `wrapping_*`/`checked_*` methods keep the 32-bit limits); `--overflow-checks` makes the arithmetic panic as Rust's debug builds do.
- `macro_rules!` are expanded before translation; threads, channels and atomics run as tasks of the support file, deterministically (see [limitations](site/docs/en/limitations.md)).
- Not translated: lifetimes (dropped), `unsafe` and raw pointers, procedural macros and `async` runtimes of other crates, crates other than `std` and `ferroswift`.

## Repository

| Folder | |
| --- | --- |
| [`crates/ferroswift`](crates/ferroswift) | The Rust side of SwiftUI: views, modifiers, state, `View` and `App` |
| [`crates/macros`](crates/macros) | `#[derive(View)]`, `#[derive(Observable)]`, `#[preview]` (markers, no generated code) |
| [`crates/transpiler`](crates/transpiler) | The translator: syn syntax tree → typed translation → Swift |
| [`crates/cli`](crates/cli) | The `ferroswift` command |
| [`examples`](examples) | `counter`, `todo` (model tested with `cargo test`), `showcase` (tabs, navigation, observable model, forms) and `gallery` (every view and modifier of the crate) |
| [`site`](site) | Website, docs (en/fr) and playground, written with Active |

## Tests

```sh
cargo test --workspace
```

- `tests/differential.rs` compiles and runs each program of `crates/transpiler/tests/programs` with `rustc` and with `swiftc`, and compares their outputs.
- `tests/swiftui.rs` translates the examples and type-checks them: against Apple's SwiftUI on macOS (`FERROSWIFT_SDK=iphonesimulator`), against [a stand-in with the same signatures](crates/transpiler/tests/swiftui-shim/SwiftUI.swift) elsewhere.
- `tests/snapshots.rs` compares translations with the expected Swift in `tests/snapshots`.
- `tests/deployment.rs` compiles every program, snapshot and example in the Swift 6 language mode and, on macOS, for iOS 17 and macOS 14.
- `crates/cli/tests/scaffold.rs` runs `ferroswift new`, then `cargo check` and `swift build` on the new app.

The Swift parts need `swiftc` on the `PATH` (or `FERROSWIFT_SWIFTC`) and are skipped without it.

## License

FerroSwift is proprietary software: © 2026 Maxime Nathan Lestage, all rights
reserved. It is not open source. The source code is published on crates.io
and GitHub so that it can be read, audited and built, and you may use it
freely as a dependency of your own programs, commercial ones included.
Modifying it or redistributing its source needs the author's written
permission. Full terms: [LICENSE](LICENSE).

Contributions (code, documentation, ideas) are welcome and become part of
the project under the same terms: by submitting one, you assign its rights
to the author, as the license states.

## Credits

Designed and developed by Maxime Nathan Lestage,
as part of one family of Rust projects: [Active](https://maxlestage.github.io/Active/), [Vitesse](https://maxlestage.github.io/Vitesse/), [Securite](https://maxlestage.github.io/Securite/), [Memoire](https://maxlestage.github.io/Memoire/), [Paquets](https://maxlestage.github.io/Paquets/) and [FerroSwift](https://maxlestage.github.io/FerroSwift/).
