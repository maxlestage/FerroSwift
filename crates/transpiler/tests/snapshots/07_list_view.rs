use ferroswift::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub done: bool,
}

#[derive(View)]
pub struct TaskList {
    #[state]
    tasks: Vec<Task>,
    #[state]
    draft: String,
}

impl View for TaskList {
    fn body(&mut self) -> impl View {
        navigation_stack().child(
            list()
                .child(
                    hstack()
                        .child(text_field("New task", &mut self.draft))
                        .child(button("Add", || {
                            let id = self.tasks.len() as u32 + 1;
                            self.tasks.push(Task { id, title: self.draft.clone(), done: false });
                            self.draft.clear();
                        })),
                )
                .child(for_each(&mut self.tasks, |task| {
                    toggle(&task.title, &mut task.done)
                }).on_delete(|offsets| self.tasks.remove_offsets(offsets)))
                .child(if self.tasks.is_empty() { Some(text("Nothing to do")) } else { None })
                .navigation_title(format!("{} tasks", self.tasks.len())),
        )
    }
}
