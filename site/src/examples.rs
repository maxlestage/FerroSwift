//! The playground's examples: Rust sources from `site/examples/`, compiled into both the
//! server and the browser bundle.

use crate::routes::Lang;

pub struct Example {
    /// In URLs: `/fr/playground/?example=todo`.
    pub id: &'static str,
    /// The name of the Rust file shown above the editor.
    pub file: &'static str,
    /// The name of the Swift file shown above the output.
    pub swift_file: &'static str,
    pub title_fr: &'static str,
    pub title_en: &'static str,
    pub source: &'static str,
}

impl Example {
    pub fn title(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Fr => self.title_fr,
            Lang::En => self.title_en,
        }
    }
}

pub const EXAMPLES: [Example; 5] = [
    Example {
        id: "counter",
        file: "counter.rs",
        swift_file: "Counter.swift",
        title_fr: "Compteur (SwiftUI)",
        title_en: "Counter (SwiftUI)",
        source: include_str!("../examples/counter.rs"),
    },
    Example {
        id: "todo",
        file: "todo.rs",
        swift_file: "TodoList.swift",
        title_fr: "Liste de tâches (List, ForEach, bindings)",
        title_en: "Todo list (List, ForEach, bindings)",
        source: include_str!("../examples/todo.rs"),
    },
    Example {
        id: "shapes",
        file: "shapes.rs",
        swift_file: "Shapes.swift",
        title_fr: "Formes (enums, match, traits)",
        title_en: "Shapes (enums, match, traits)",
        source: include_str!("../examples/shapes.rs"),
    },
    Example {
        id: "errors",
        file: "errors.rs",
        swift_file: "Errors.swift",
        title_fr: "Erreurs (Result, ? → throws)",
        title_en: "Errors (Result, ? → throws)",
        source: include_str!("../examples/errors.rs"),
    },
    Example {
        id: "iterators",
        file: "iterators.rs",
        swift_file: "Iterators.swift",
        title_fr: "Itérateurs (map, filter, fold, HashMap)",
        title_en: "Iterators (map, filter, fold, HashMap)",
        source: include_str!("../examples/iterators.rs"),
    },
];

/// The index of the example `id` (unknown ids give the first one).
pub fn index_of(id: &str) -> usize {
    EXAMPLES.iter().position(|e| e.id == id).unwrap_or(0)
}
