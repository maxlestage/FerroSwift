# FerroSwift

[English](README.md) · **Français**

**Écrivez du Rust. Livrez du Swift et du SwiftUI.** FerroSwift traduit du Rust en Swift idiomatique : la logique de votre application *et* ses vues SwiftUI, vérifiées par `rustc`, testées avec `cargo test`, ouvertes dans Xcode.

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

`ferroswift build` écrit :

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

Les vues s'écrivent avec des fonctions Rust ordinaires (`vstack()`, `text()`, `button()`…) du crate [`ferroswift`](crates/ferroswift) : `cargo check` et rust-analyzer les connaissent, et chacune documente le code SwiftUI qu'elle devient. Tout le reste est du Rust normal — structs, enums, traits, `match`, `Option`, `Result`, itérateurs — traduit en Swift tel que vous l'écririez à la main.

📚 **Site web, documentation et bac à sable en direct** (le transpileur compilé en WebAssembly, dans votre navigateur) : [`site/`](site), écrit avec [active](https://github.com/maxlestage/Active). Lancez-le avec `cargo run -p ferroswift-site`.

## Pourquoi

- **Seulement du Rust.** Un seul langage pour le modèle et les vues, et l'outillage Rust pour les deux : le compilateur, Clippy, `cargo test` pour la logique, rust-analyzer dans l'éditeur.
- **Du Swift idiomatique.** Structs et enums avec valeurs associées, `extension` pour les blocs `impl`, protocoles pour les traits, `switch` avec `where`, `guard let`, `throws` typé pour `Result`, `@State`, `@Binding`, `@Observable`. Les commentaires et la documentation sont conservés. Pas de runtime : un petit fichier de support, seulement quand le code Rust en a besoin.
- **Le même comportement.** Les tests exécutent chaque programme d'exemple deux fois, compilé par `rustc` puis, traduit, par `swiftc`, et comparent les sorties octet par octet : formatage `{:?}` et `{:.2}`, alignements, conversions d'entiers, ordre des `BTreeMap`, `?` et retours anticipés.
- **Des diagnostics honnêtes.** Ce qui ne peut pas être traduit est signalé comme le fait `rustc`, avec fichier, ligne et colonne, et le Swift produit compile quand même (un `fatalError` marque l'endroit).
- **Rapide.** Un crate se traduit en quelques millisecondes ; `ferroswift watch` retraduit à chaque enregistrement.

## Installation

```sh
cargo install --git https://github.com/maxlestage/FerroSwift ferroswift-cli
```

## Démarrage rapide

```sh
ferroswift new hello      # un crate Rust (src/lib.rs) et son package Swift (swift/)
cd hello
ferroswift watch          # traduit src/ dans swift/Sources/Hello à chaque enregistrement
cd swift && swift run     # macOS : l'application s'ouvre
```

Pour iOS, créez une application dans Xcode et ajoutez-y le dossier `swift/Sources/Hello` : avec les dossiers synchronisés de Xcode, les fichiers ajoutés ou supprimés par FerroSwift suivent d'eux-mêmes.

| Commande | |
| --- | --- |
| `ferroswift new <nom>` | Une nouvelle application : `Cargo.toml`, `src/lib.rs`, `swift/Package.swift` |
| `ferroswift build [crate] [-o dossier]` | Traduit `src/**/*.rs`, un fichier Swift par fichier Rust, plus `FerroSwiftSupport.swift` si besoin |
| `ferroswift watch [crate]` | Pareil, à chaque modification |
| `ferroswift check [crate]` | Seulement les diagnostics |
| `ferroswift file app.rs` | Un seul fichier, sur la sortie standard |

## SwiftUI depuis Rust

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

Piles, listes, formulaires, sections, navigation, onglets, boutons, champs de texte, interrupteurs, curseurs, compteurs, sélecteurs, images, labels, formes, barres de progression et une soixantaine de modificateurs : voir [le guide SwiftUI](site/docs/fr/swiftui.md) et les [exemples](examples).

## Rust → Swift

| Rust | Swift |
| --- | --- |
| `i32`, `i64`, `isize`, `usize` · `u8` … `u64` · `f64` · `f32` | `Int` · `UInt8` … `UInt64` · `Double` · `Float` |
| `String`, `&str` · `char` · `Vec<T>`, `&[T]` · `HashMap<K, V>` · `HashSet<T>` | `String` · `Character` · `[T]` · `[K: V]` · `Set<T>` |
| `Option<T>` · `Box<T>`, `Rc<T>` | `T?` · `T` |
| `fn f() -> Result<T, E>` · `f()?` · `Err(e)` | `func f() throws(E) -> T` · `try f()` · `throw e` |
| `struct` · `enum` avec données · `impl X` · `trait` · `impl Trait for X` | `struct` · `enum` à valeurs associées (`indirect` si récursif) · `extension X` · `protocol` · `extension X: Trait` |
| `fn new(…) -> Self` · `&mut self` · `mut self` | `init(…)` · `mutating func` · une copie modifiée |
| `#[derive(PartialEq, Hash, PartialOrd)]` · `impl Display` · `impl Add` | `Equatable`, `Hashable`, `Comparable` · `CustomStringConvertible` · `static func +` |
| `match` · `if let` · `let … else` · `while let` · `for x in &mut v` | `switch` · `if let` / `if case` · `guard` · `while let` · une boucle sur les indices |
| `\|x\| x + 1` · `impl Fn(i32) -> i32` | `{ x in x + 1 }` · `(Int) -> Int` |
| `v.iter().filter(…).map(…).collect()` · `.sum()` · `.fold(…)` | `v.filter { … }.map { … }` · `.reduce(0, +)` · `.reduce(…) { … }` |
| `format!("{name}: {:>8.2}", x)` · `println!` · `{:?}` | interpolation · `print` · `rsDebug(x)` (le format de Rust) |

La liste complète, et ce qui n'est pas pris en charge, se trouve dans [le guide du langage](site/docs/fr/language.md).

## Limites

- Pas de sémantique d'emprunt en Swift : les valeurs sont copiées. `Rc<RefCell<T>>`, `Cell<T>` et `Arc<Mutex<T>>` deviennent une classe qui porte la valeur, partagée comme en Rust ; les verrous ne bloquent jamais et `Rc::strong_count` n'a pas d'équivalent.
- `i32`, `i64` et `usize` deviennent tous l'`Int` 64 bits de Swift : `+` déborde à 2⁶³, pas à 2³¹ (le parsing, les conversions `as` et les méthodes `wrapping_*`/`checked_*` gardent les limites 32 bits).
- Non traduits : les durées de vie (ignorées), `unsafe` et les pointeurs bruts, `macro_rules!`, les threads, les runtimes `async`, les objets-traits à méthodes génériques, les crates autres que `std` et `ferroswift`.

## Contenu du dépôt

| Dossier | |
| --- | --- |
| [`crates/ferroswift`](crates/ferroswift) | Le côté Rust de SwiftUI : vues, modificateurs, état, `View` et `App` |
| [`crates/macros`](crates/macros) | `#[derive(View)]`, `#[derive(Observable)]`, `#[preview]` (des marqueurs, sans code généré) |
| [`crates/transpiler`](crates/transpiler) | Le traducteur : arbre syntaxique syn → traduction typée → Swift |
| [`crates/cli`](crates/cli) | La commande `ferroswift` |
| [`examples`](examples) | `counter`, `todo` (modèle testé avec `cargo test`), `showcase` (onglets, navigation, modèle observable, formulaires) et `gallery` (toutes les vues et tous les modificateurs du crate) |
| [`site`](site) | Site web, documentation (en/fr) et bac à sable, écrits avec active |

## Tests

```sh
cargo test --workspace
```

- `tests/differential.rs` compile et exécute chaque programme de `crates/transpiler/tests/programs` avec `rustc` et avec `swiftc`, puis compare leurs sorties.
- `tests/swiftui.rs` traduit les exemples et les vérifie : avec le SwiftUI d'Apple sur macOS (`FERROSWIFT_SDK=iphonesimulator`), avec [un substitut aux mêmes signatures](crates/transpiler/tests/swiftui-shim/SwiftUI.swift) ailleurs.
- `tests/snapshots.rs` compare les traductions au Swift attendu dans `tests/snapshots`.
- `tests/deployment.rs` compile chaque programme, snapshot et exemple en mode Swift 6 et, sur macOS, pour iOS 17 et macOS 14.
- `crates/cli/tests/scaffold.rs` lance `ferroswift new`, puis `cargo check` et `swift build` sur la nouvelle application.

Les parties Swift demandent `swiftc` dans le `PATH` (ou `FERROSWIFT_SWIFTC`) et sont ignorées sans lui.

## Licence

[MIT](LICENSE)
