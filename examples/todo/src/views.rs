//! The views, translated to SwiftUI.

use ferroswift::prelude::*;

use crate::model::{Priority, Todo, TodoList};

/// The list of tasks with a field to add one.
#[derive(View, Default)]
pub struct ContentView {
    #[state]
    list: TodoList,
    #[state]
    draft: String,
    #[state(Priority::Normal)]
    priority: Priority,
    #[state]
    show_about: bool,
}

impl ContentView {
    fn add(&mut self) {
        if self.list.add(&self.draft, self.priority) {
            self.draft = String::new();
            self.list.sort();
        }
    }

    /// The field and the priority picker.
    fn composer(&mut self) -> impl View {
        section()
            .header("New task")
            .child(
                text_field("What needs to be done?", &mut self.draft)
                    .on_submit(|| self.add()),
            )
            .child(
                picker("Priority", &mut self.priority)
                    .child(text("Low").tag(Priority::Low))
                    .child(text("Normal").tag(Priority::Normal))
                    .child(text("High").tag(Priority::High))
                    .picker_style(PickerStyle::Segmented),
            )
            .child(button("Add", || self.add()).disabled(self.draft.trim().is_empty()))
    }
}

impl View for ContentView {
    fn body(&mut self) -> impl View {
        navigation_stack().child(
            list()
                .child(self.composer())
                .child(
                    section()
                        .header(self.list.summary())
                        .child(
                            for_each(&mut self.list.items, |todo| TodoRow { todo: todo.clone() })
                                .on_delete(|offsets| self.list.items.remove_offsets(offsets)),
                        ),
                )
                .navigation_title("To do")
                .toolbar(
                    hstack()
                        .child(button("Clear done", || self.list.clear_done()).disabled(self.list.remaining() == self.list.items.len()))
                        .child(button_label(|| self.show_about = true, system_image("info.circle"))),
                )
                .sheet(&mut self.show_about, about()),
        )
    }
}

/// One task: a checkbox, its title and its priority.
#[derive(View)]
pub struct TodoRow {
    #[binding]
    todo: Todo,
}

impl View for TodoRow {
    fn body(&mut self) -> impl View {
        hstack()
            .child(toggle("", &mut self.todo.done).labels_hidden())
            .child(
                text(&self.todo.title)
                    .strikethrough(self.todo.done)
                    .foreground(if self.todo.done { Color::Secondary } else { Color::Primary }),
            )
            .child(spacer())
            .child(system_image(self.todo.priority.symbol()).foreground(Color::Orange))
    }
}

/// The "about" sheet.
fn about() -> impl View {
    vstack()
        .spacing(12.0)
        .child(system_image("checklist").font(Font::LargeTitle).foreground(Color::Accent))
        .child(text("Written in Rust").font(Font::Headline))
        .child(text("Translated to SwiftUI by FerroSwift.").foreground(Color::Secondary))
        .padding()
}

/// The app.
pub struct TodoApp;

impl App for TodoApp {
    fn body(&mut self) -> impl Scene {
        window_group(ContentView::default())
    }
}
