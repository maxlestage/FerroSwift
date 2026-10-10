#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

mod element;
mod style;
mod views;

pub use element::Element;
pub use ferroswift_macros::{Observable, View, preview};
pub use style::*;
pub use views::*;

/// A SwiftUI view: `struct X: View { var body: some View { … } }`.
///
/// ```
/// use ferroswift::prelude::*;
///
/// #[derive(View)]
/// struct Greeting {
///     name: String,
/// }
///
/// impl View for Greeting {
///     fn body(&mut self) -> impl View {
///         text(format!("Hello, {}!", self.name)).font(Font::Title)
///     }
/// }
/// ```
///
/// `body` takes `&mut self` so that actions can write `#[state]` fields directly:
/// `button("+", || self.count += 1)` becomes `Button("+") { count += 1 }`.
pub trait View {
    /// The content of the view: `var body: some View`.
    fn body(&mut self) -> impl View {
        Never
    }
}

/// The type of views that have no body of their own (the built-in ones).
#[derive(Debug, Clone, Copy, Default)]
pub struct Never;

impl View for Never {}

impl<V: View> View for Option<V> {}

/// A SwiftUI app: `@main struct X: App { var body: some Scene { … } }`.
///
/// ```
/// use ferroswift::prelude::*;
///
/// #[derive(View, Default)]
/// struct ContentView;
/// impl View for ContentView {
///     fn body(&mut self) -> impl View {
///         text("Hello")
///     }
/// }
///
/// struct HelloApp;
/// impl App for HelloApp {
///     fn body(&mut self) -> impl Scene {
///         window_group(ContentView)
///     }
/// }
/// ```
pub trait App {
    /// The scenes of the app: `var body: some Scene`.
    fn body(&mut self) -> impl Scene;
}

/// A part of an app's user interface, such as a window: `some Scene`.
pub trait Scene {}

/// A window of the app (`WindowGroup { … }`); see [`window_group`].
#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct SceneElement;

impl Scene for SceneElement {}

/// `WindowGroup { content }`: the main window of an app.
pub fn window_group(_content: impl View) -> SceneElement {
    SceneElement
}

/// The positions of the rows of a list, given to [`Element::on_delete`] and
/// [`Element::on_move`]: SwiftUI's `IndexSet`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexSet(pub Vec<usize>);

/// List editing helpers on `Vec`, with the names of their SwiftUI counterparts.
pub trait VecExt {
    /// `items.remove(atOffsets: offsets)`.
    fn remove_offsets(&mut self, offsets: IndexSet);
    /// `items.move(fromOffsets: offsets, toOffset: destination)`.
    fn move_offsets(&mut self, offsets: IndexSet, destination: usize);
}

impl<T> VecExt for Vec<T> {
    fn remove_offsets(&mut self, offsets: IndexSet) {
        let mut positions = offsets.0;
        positions.sort_unstable();
        positions.dedup();
        for index in positions.into_iter().rev() {
            if index < self.len() {
                self.remove(index);
            }
        }
    }

    fn move_offsets(&mut self, offsets: IndexSet, destination: usize) {
        let mut positions = offsets.0;
        positions.sort_unstable();
        positions.dedup();
        positions.retain(|&i| i < self.len());
        let before = positions.iter().filter(|&&i| i < destination).count();
        let mut moved = Vec::with_capacity(positions.len());
        for &index in positions.iter().rev() {
            moved.push(self.remove(index));
        }
        moved.reverse();
        let at = destination.saturating_sub(before).min(self.len());
        for (offset, item) in moved.into_iter().enumerate() {
            self.insert(at + offset, item);
        }
    }
}

/// `withAnimation { body }`: state changes made by `body` are animated.
pub fn with_animation(mut body: impl FnMut()) {
    body();
}

/// `withAnimation(animation) { body }`.
pub fn with_animation_of(_animation: Animation, mut body: impl FnMut()) {
    body();
}

/// Everything an app file needs: `use ferroswift::prelude::*;`.
pub mod prelude {
    pub use crate::style::*;
    pub use crate::views::*;
    pub use crate::{
        App, Element, IndexSet, Never, Observable, Scene, SceneElement, VecExt, View, preview,
        window_group, with_animation, with_animation_of,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_and_moves_offsets_like_swiftui() {
        let mut v = vec!['a', 'b', 'c', 'd', 'e'];
        v.remove_offsets(IndexSet(vec![3, 1]));
        assert_eq!(v, ['a', 'c', 'e']);

        let mut v = vec!['a', 'b', 'c', 'd', 'e'];
        v.move_offsets(IndexSet(vec![0]), 3);
        assert_eq!(v, ['b', 'c', 'a', 'd', 'e']);

        let mut v = vec!['a', 'b', 'c', 'd', 'e'];
        v.move_offsets(IndexSet(vec![4]), 1);
        assert_eq!(v, ['a', 'e', 'b', 'c', 'd']);

        let mut v = vec!['a', 'b', 'c', 'd', 'e'];
        v.move_offsets(IndexSet(vec![1, 2]), 5);
        assert_eq!(v, ['a', 'd', 'e', 'b', 'c']);
    }
}
