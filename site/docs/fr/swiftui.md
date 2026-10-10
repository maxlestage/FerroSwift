---
title: SwiftUI en Rust
description: Vues, enfants, modificateurs, état et bindings, listes, navigation et point d’entrée de l’app avec le crate ferroswift.
order: 2
---

Le crate `ferroswift` décrit SwiftUI en Rust. Ses fonctions renvoient des builders qui ne font rien à l’exécution : ils existent pour que `cargo check` et rust-analyzer connaissent vos vues, et chacun documente le code SwiftUI qu’il devient. Importez tout avec :

```rust
use ferroswift::prelude::*;
```

> **Note :** le transpileur reconnaît les vues d’un fichier grâce à cette ligne `use`. Sans elle, `vstack()` et les autres sont traduits comme des appels de fonction ordinaires.

## Une vue

Une vue est une struct avec `#[derive(View)]` et un `impl View` dont `body` renvoie le contenu :

```rust
#[derive(View)]
pub struct Greeting {
    name: String,
}

impl View for Greeting {
    fn body(&mut self) -> impl View {
        text(format!("Hello, {}!", self.name)).font(Font::Title)
    }
}
```

```swift
struct Greeting: View {
    let name: String

    var body: some View {
        Text("Hello, \(name)!").font(.title)
    }
}
```

Les champs simples deviennent des propriétés `let`, données par le parent : `Greeting { name: "Ada".into() }` devient `Greeting(name: "Ada")`.

## Vues intégrées

| Rust | SwiftUI |
| --- | --- |
| `text("Hi")`, `text(format!(…))` | `Text("Hi")`, `Text("…\(x)…")` |
| `button("OK", \|\| …)` | `Button("OK") { … }` |
| `button_role("Delete", ButtonRole::Destructive, \|\| …)` | `Button("Delete", role: .destructive) { … }` |
| `button_label(\|\| …, label("Add", "plus"))` | `Button { … } label: { Label("Add", systemImage: "plus") }` |
| `vstack()`, `hstack()`, `zstack()` | `VStack`, `HStack`, `ZStack` |
| `list()`, `form()`, `section().header("…")` | `List`, `Form`, `Section("…")` |
| `scroll_view()`, `group()`, `tab_view()` | `ScrollView`, `Group`, `TabView` |
| `image("logo")`, `system_image("star")` | `Image("logo")`, `Image(systemName: "star")` |
| `toggle("Wi-Fi", &mut self.on)` | `Toggle("Wi-Fi", isOn: $on)` |
| `text_field("Name", &mut self.name)` | `TextField("Name", text: $name)` |
| `slider(&mut self.v, 0.0..=1.0)` | `Slider(value: $v, in: 0...1)` |
| `stepper("Level", &mut self.n, 1..=10)` | `Stepper("Level", value: $n, in: 1...10)` |
| `picker("Size", &mut self.size)` | `Picker("Size", selection: $size)` |
| `spacer()`, `divider()`, `progress_view()` | `Spacer()`, `Divider()`, `ProgressView()` |
| `rectangle()`, `circle()`, `rounded_rectangle(12.0)` | `Rectangle()`, `Circle()`, `RoundedRectangle(cornerRadius: 12)` |

Chaque fonction du crate est documentée avec son équivalent SwiftUI : survolez-la dans votre éditeur.

## Les enfants

Les conteneurs reçoivent leurs enfants avec `.child(…)`, dans l’ordre. Le contrôle de flux de Rust fonctionne à l’intérieur, comme dans un result builder SwiftUI :

```rust
vstack()
    .spacing(8.0)
    .child(text("Title").font(Font::Headline))
    .child(if self.items.is_empty() {
        text("Nothing yet")
    } else {
        text(format!("{} items", self.items.len()))
    })
    .child(self.loading.then(|| progress_view()))
    .child(self.error.clone().map(|e| text(e).foreground(Color::Red)))
```

`cond.then(|| vue)` devient `if cond { … }`, `option.map(|x| vue)` devient `if let x = option { … }`, et `match` devient `switch`.

## Les modificateurs

Les modificateurs sont des méthodes chaînées, dans l’ordre de SwiftUI : `.padding()`, `.font(Font::Title)`, `.foreground(Color::Secondary)`, `.background(Color::Yellow)`, `.corner_radius(12.0)`, `.frame_max_width(f64::INFINITY)`… Les modificateurs `frame_…` consécutifs fusionnent en un seul `.frame(…)`. Ils s’appliquent aussi à vos propres vues : `TodoRow { … }.padding()`.

Les valeurs de style sont des enums dont les variantes deviennent des membres SwiftUI : `Font::LargeTitle` donne `.largeTitle`, `Color::Accent` donne `.accentColor`, `Color::rgb(1.0, 0.5, 0.0)` donne `Color(red: 1, green: 0.5, blue: 0)`.

## État et bindings

```rust
#[derive(View, Default)]
pub struct Settings {
    #[state]
    name: String,
    #[state(3)]
    level: i64,
}
```

- `#[state]` devient `@State var name = ""` (la valeur par défaut du type) ; `#[state(3)]` donne la valeur initiale.
- `body` prend `&mut self` : les actions écrivent l’état directement, `button("+", || self.level += 1)` devient `Button("+") { level += 1 }`.
- `&mut self.name` passé à un contrôle devient le binding `$name`.
- Un champ `#[binding]` devient `@Binding var` : le parent passe `&mut self.son_champ`.
- `#[environment(dismiss)] dismiss: DismissAction` devient `@Environment(\.dismiss) var dismiss`.
- `#[derive(Observable)]` sur une struct donne une `@Observable final class`, partagée par référence entre les vues ; `#[observation_ignored]` marque les champs qui ne déclenchent pas de mise à jour.

## Les listes

```rust
list().child(
    for_each(&mut self.todos, |todo| toggle(todo.title.clone(), &mut todo.done))
        .on_delete(|offsets| self.todos.remove_offsets(offsets)),
)
```

```swift
List {
    ForEach($todos) { $todo in
        Toggle(todo.title, isOn: $todo.done)
    }
    .onDelete { offsets in todos.remove(atOffsets: offsets) }
}
```

`for_each(items, |item| ligne)` devient `ForEach`. Les éléments dont le type a un champ `id` sont `Identifiable` ; les nombres et les chaînes utilisent `id: \.self`. Parcourir `&mut self.items` donne un binding vers chaque ligne. `.children(items.iter().map(|i| …))` est une autre façon d’écrire un `ForEach`.

## Navigation et présentation

- `navigation_stack().child(…)` → `NavigationStack { … }`, avec `.navigation_title("…")` et `.toolbar(…)` sur le contenu.
- `navigation_link("Details", DetailView { … })` → `NavigationLink("Details") { DetailView(…) }` ; `navigation_link_label(destination, label)` prend une vue comme libellé.
- `.sheet(&mut self.showing, contenu)` → `.sheet(isPresented: $showing) { … }` ; `.alert(titre, &mut self.flag, actions)` de même.
- `.searchable(&mut self.query)`, `.swipe_actions(…)`, `.context_menu(…)`, `.tab_item("Books", "books.vertical")`.
- `with_animation(|| self.open = !self.open)` → `withAnimation { open = !open }`.

## L’app et les previews

```rust
pub struct NotesApp;

impl App for NotesApp {
    fn body(&mut self) -> impl Scene {
        window_group(ContentView::default())
    }
}

#[preview]
fn preview() -> impl View {
    ContentView::default()
}
```

L’implémentation de `App` devient `@main struct NotesApp: App` avec un `WindowGroup` ; chaque fonction `#[preview]` devient un bloc `#Preview` (à exclure avec `ferroswift build --no-previews`).
