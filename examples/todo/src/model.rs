//! The model: no UI here, only Rust.

/// How urgent a task is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}

impl Priority {
    /// An SF Symbol for the priority.
    pub fn symbol(&self) -> &'static str {
        match self {
            Priority::Low => "arrow.down",
            Priority::Normal => "minus",
            Priority::High => "exclamationmark",
        }
    }
}

/// One task.
#[derive(Debug, Clone, PartialEq)]
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub done: bool,
    pub priority: Priority,
}

/// The tasks, in display order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TodoList {
    pub items: Vec<Todo>,
    pub next_id: u32,
}

impl TodoList {
    /// Adds a task; empty titles are ignored. Returns whether it was added.
    pub fn add(&mut self, title: &str, priority: Priority) -> bool {
        let title = title.trim();
        if title.is_empty() {
            return false;
        }
        self.next_id += 1;
        self.items.push(Todo {
            id: self.next_id,
            title: title.to_string(),
            done: false,
            priority,
        });
        true
    }

    /// Tasks left to do.
    pub fn remaining(&self) -> usize {
        self.items.iter().filter(|t| !t.done).count()
    }

    /// Removes the finished tasks.
    pub fn clear_done(&mut self) {
        self.items.retain(|t| !t.done);
    }

    /// Most urgent first, unfinished before finished.
    pub fn sort(&mut self) {
        self.items
            .sort_by(|a, b| a.done.cmp(&b.done).then(b.priority.cmp(&a.priority)));
    }

    /// A one-line summary: "2 of 3 left".
    pub fn summary(&self) -> String {
        match (self.remaining(), self.items.len()) {
            (_, 0) => "Nothing to do".to_string(),
            (0, _) => "All done!".to_string(),
            (left, total) => format!("{} of {} left", left, total),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_counts_and_clears() {
        let mut list = TodoList::default();
        assert!(list.add("  Write Rust ", Priority::High));
        assert!(!list.add("   ", Priority::Low));
        list.add("Ship Swift", Priority::Normal);
        assert_eq!(list.items[0].title, "Write Rust");
        assert_eq!(list.summary(), "2 of 2 left");
        list.items[0].done = true;
        assert_eq!(list.remaining(), 1);
        list.clear_done();
        assert_eq!(list.items.len(), 1);
    }
}
