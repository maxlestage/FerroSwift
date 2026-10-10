//! A to-do list. The model is plain Rust, tested with `cargo test`; the views use the
//! ferroswift builders. `ferroswift build` translates both into Swift.

mod model;
mod views;

pub use model::{Priority, Todo, TodoList};
pub use views::{ContentView, TodoApp};
