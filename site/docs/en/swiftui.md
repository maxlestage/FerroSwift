---
title: SwiftUI in Rust
description: Views, children, modifiers, state and bindings, lists, navigation and the app entry point with the ferroswift crate.
order: 2
---

The `ferroswift` crate describes SwiftUI in Rust. Its functions return builders that do nothing at run time: they exist so that `cargo check` and rust-analyzer know your views, and each one documents the SwiftUI code it becomes. Import everything with:

```rust
use ferroswift::prelude::*;
```

> **Note:** the transpiler recognizes the views of a crate through this `use` line, or through `#[derive(View)]`, `impl View` and `impl App`. In a file with none of them, `vstack()` and friends are translated as ordinary function calls.

## A view

A view is a struct with `#[derive(View)]` and an `impl View` whose `body` returns the content:

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

Plain fields become `let` properties, given by the parent: `Greeting { name: "Ada".into() }` becomes `Greeting(name: "Ada")`.

## Built-in views

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

Every function of the crate is documented with its SwiftUI counterpart: hover it in your editor.

## Children

Containers take their children with `.child(…)`, in order. Rust control flow works inside, as in a SwiftUI result builder:

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

`cond.then(|| view)` becomes `if cond { … }`, `option.map(|x| view)` becomes `if let x = option { … }`, and `match` becomes `switch`.

## Modifiers

Modifiers are chained methods, in SwiftUI's order: `.padding()`, `.font(Font::Title)`, `.foreground(Color::Secondary)`, `.background(Color::Yellow)`, `.corner_radius(12.0)`, `.frame_max_width(f64::INFINITY)`… Consecutive `frame_…` modifiers merge into one `.frame(…)`. Modifiers work on your own views too: `TodoRow { … }.padding()`.

Style values are enums whose variants become SwiftUI members: `Font::LargeTitle` is `.largeTitle`, `Color::Accent` is `.accentColor`, `Color::rgb(1.0, 0.5, 0.0)` is `Color(red: 1, green: 0.5, blue: 0)`.

## State and bindings

```rust
#[derive(View, Default)]
pub struct Settings {
    #[state]
    name: String,
    #[state(3)]
    level: i64,
}
```

- `#[state]` becomes `@State var name = ""` (the type's default value); `#[state(3)]` gives the initial value.
- `body` takes `&mut self`, so actions write state directly: `button("+", || self.level += 1)` becomes `Button("+") { level += 1 }`.
- `&mut self.name` passed to a control becomes the binding `$name`.
- A `#[binding]` field becomes `@Binding var`: the parent passes `&mut self.its_field`.
- `#[environment(dismiss)] dismiss: DismissAction` becomes `@Environment(\.dismiss) var dismiss`.
- `#[derive(Observable)]` on a struct makes an `@Observable final class`, shared by reference between views; `#[observation_ignored]` marks fields that do not trigger updates.

## Lists

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

`for_each(items, |item| row)` becomes `ForEach`. Items whose type has an `id` field are `Identifiable`; numbers and strings use `id: \.self`. Iterating `&mut self.items` gives bindings to each row. `.children(items.iter().map(|i| …))` is another way to write a `ForEach`.

## Navigation and presentation

- `navigation_stack().child(…)` → `NavigationStack { … }`, with `.navigation_title("…")` and `.toolbar(…)` on the content.
- `navigation_link("Details", DetailView { … })` → `NavigationLink("Details") { DetailView(…) }`; `navigation_link_label(destination, label)` takes a view as label.
- `.sheet(&mut self.showing, content)` → `.sheet(isPresented: $showing) { … }`; `.alert(title, &mut self.flag, actions)` likewise.
- `.searchable(&mut self.query)`, `.swipe_actions(…)`, `.context_menu(…)`, `.tab_item("Books", "books.vertical")`.
- `with_animation(|| self.open = !self.open)` → `withAnimation { open = !open }`.

## The app and previews

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

The `App` implementation becomes `@main struct NotesApp: App` with a `WindowGroup`; each `#[preview]` function becomes a `#Preview` block (leave them out with `ferroswift build --no-previews`).
