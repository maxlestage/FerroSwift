use ferroswift::prelude::*;

/// One task of the list. Its `id` makes it `Identifiable` in Swift.
#[derive(Clone, Debug, PartialEq)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

#[derive(View, Default)]
pub struct TodoList {
    #[state]
    todos: Vec<Todo>,
    #[state]
    draft: String,
    #[state]
    next_id: u64,
}

impl TodoList {
    /// Adds the draft as a new task, unless it is blank.
    fn add(&mut self) {
        let title = self.draft.trim().to_string();
        if title.is_empty() {
            return;
        }
        self.next_id += 1;
        self.todos.push(Todo { id: self.next_id, title, done: false });
        self.draft.clear();
    }

    fn remaining(&self) -> usize {
        self.todos.iter().filter(|todo| !todo.done).count()
    }
}

impl View for TodoList {
    fn body(&mut self) -> impl View {
        navigation_stack().child(
            list()
                .child(
                    hstack()
                        .child(text_field("New task", &mut self.draft).on_submit(|| self.add()))
                        .child(button("Add", || self.add()).disabled(self.draft.is_empty())),
                )
                .child(
                    section()
                        .header(format!("{} left", self.remaining()))
                        .child(
                            for_each(&mut self.todos, |todo| {
                                toggle(todo.title.clone(), &mut todo.done)
                            })
                            .on_delete(|offsets| self.todos.remove_offsets(offsets)),
                        ),
                )
                .navigation_title("Todo"),
        )
    }
}
