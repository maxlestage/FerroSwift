---
title: Démarrer
description: Installer la CLI, créer une app, la traduire en Swift et la lancer avec Xcode ou swift run.
order: 1
---

FerroSwift vous permet d’écrire une app iOS ou macOS en Rust — sa logique et ses vues SwiftUI — et la traduit en fichiers Swift qu’Xcode compile comme les autres. `cargo check` et rust-analyzer vérifient votre code pendant que vous tapez ; `ferroswift` écrit le Swift.

## Prérequis

- **Rust** (stable, édition 2024) : [rustup.rs](https://rustup.rs).
- **Xcode 16 ou plus récent** (Swift 6) pour compiler l’app. Sur macOS, la chaîne d’outils Swift seule suffit pour la lancer avec `swift run`.

La traduction elle-même fonctionne partout où Rust fonctionne : vous pouvez écrire et vérifier la partie Rust sous Linux ou Windows.

## Installer la CLI

```sh
cargo install ferroswift-cli
ferroswift --version
```

## Créer une app

```sh
ferroswift new MyApp
cd MyApp
```

`ferroswift new` crée un crate Rust et le package Swift dans lequel il est traduit, puis le traduit une première fois :

```text
MyApp/
├── Cargo.toml          le crate, qui dépend de `ferroswift`
├── src/
│   └── lib.rs          votre app, en Rust
├── swift/
│   ├── Package.swift   le package Swift (macOS 14, iOS 17)
│   └── Sources/MyApp/  généré par FerroSwift : ne pas modifier
├── .gitignore
└── README.md
```

`src/lib.rs` commence avec un premier écran et l’app elle-même :

```rust
use ferroswift::prelude::*;

/// Le premier écran.
#[derive(View, Default)]
pub struct ContentView {
    #[state]
    taps: u32,
}

impl View for ContentView {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(16.0)
            .child(text(format!("Tapped {} times", self.taps)).font(Font::Title2))
            .child(button("Tap me", || self.taps += 1).button_style(ButtonStyle::BorderedProminent))
            .padding()
    }
}

/// L’app : une fenêtre.
pub struct MyAppApp;

impl App for MyAppApp {
    fn body(&mut self) -> impl Scene {
        window_group(ContentView::default())
    }
}
```

Les fonctions du crate `ferroswift` ne font rien à l’exécution : elles existent pour que le compilateur, rust-analyzer et la documentation connaissent vos vues. Voir [SwiftUI en Rust](../swiftui/).

## Traduire

```sh
ferroswift build    # une fois
ferroswift watch    # à nouveau après chaque sauvegarde
```

Chaque fichier `src/**/*.rs` devient un fichier `.swift` dans `swift/Sources/MyApp` (`src/views/todo_row.rs` → `TodoRow.swift`), plus `FerroSwiftSupport.swift` quand le code a besoin d’une aide (formatage à la Rust, quelques méthodes de collections). Ce qui ne se traduit pas est signalé avec son fichier, sa ligne et sa colonne, et `ferroswift build` se termine alors en erreur.

Gardez aussi `cargo check` (ou rust-analyzer) actif : FerroSwift traduit du code qui compile, il ne remplace pas le compilateur.

## Lancer l’app

### Sur macOS

```sh
cd swift
swift run
```

ou ouvrez `swift/Package.swift` dans Xcode et appuyez sur ⌘R.

### Sur iOS

1. Dans Xcode, créez un projet **iOS App** (interface SwiftUI).
2. Supprimez les fichiers `ContentView.swift` et `…App.swift` du modèle : votre `App` Rust devient l’app `@main`.
3. Glissez le dossier `swift/Sources/MyApp` dans le projet et ajoutez-le à la cible de l’app. En dossier synchronisé, les fichiers que FerroSwift ajoute ou supprime apparaissent d’eux-mêmes.
4. Lancez `ferroswift watch` dans un terminal, puis compilez et lancez dans Xcode.

## Et ensuite

- [SwiftUI en Rust](../swiftui/) : vues, modificateurs, état et navigation.
- [Rust → Swift](../language/) : le sous-ensemble de Rust pris en charge et ce qu’il devient.
- [La CLI](../cli/) : toutes les commandes et options.
- [Limites](../limitations/) : ce qui n’est pas (encore) traduit.
- Le [playground](../../playground/) traduit pendant que vous tapez, dans votre navigateur.
