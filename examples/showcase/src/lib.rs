//! A tour of what FerroSwift translates: tabs, navigation, an observable model shared by
//! several screens, forms, sheets, alerts, shapes and animations.

mod library;
mod settings;
mod shapes;

use ferroswift::prelude::*;

pub use library::{Book, Library};

/// Tabs at the bottom of the screen.
#[derive(View, Default)]
pub struct RootView {
    #[state]
    library: Library,
}

impl View for RootView {
    fn body(&mut self) -> impl View {
        tab_view()
            .child(
                library::LibraryView {
                    library: self.library.clone(),
                }
                .tab_item("Books", "books.vertical"),
            )
            .child(shapes::ShapesView::default().tab_item("Shapes", "circle.hexagongrid"))
            .child(settings::SettingsView::default().tab_item("Settings", "gearshape"))
    }
}

/// The app.
pub struct ShowcaseApp;

impl App for ShowcaseApp {
    fn body(&mut self) -> impl Scene {
        window_group(RootView::default())
    }
}
