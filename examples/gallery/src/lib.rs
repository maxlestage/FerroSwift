//! Every view and modifier of the ferroswift crate, screen by screen. Less a real app than
//! a check that each one translates into SwiftUI that compiles.

mod screens;

use ferroswift::prelude::*;

pub use screens::{ImagesView, InputsView, LazyView, ListsView, TextStylesView};

/// A row of the gallery's list.
#[derive(Debug, Clone, PartialEq)]
pub struct Photo {
    pub id: u32,
    pub title: String,
    pub likes: i64,
}

/// A few photos to start with.
pub fn sample_photos() -> Vec<Photo> {
    vec![
        Photo {
            id: 1,
            title: "Dawn".to_string(),
            likes: 12,
        },
        Photo {
            id: 2,
            title: "Harbour".to_string(),
            likes: 3,
        },
        Photo {
            id: 3,
            title: "Ridge".to_string(),
            likes: 40,
        },
    ]
}

/// The first screen: a list of the other screens.
#[derive(View, Default)]
pub struct GalleryView {
    #[state(true)]
    dark: bool,
}

impl View for GalleryView {
    fn body(&mut self) -> impl View {
        navigation_stack()
            .child(
                list()
                    .child(
                        section()
                            .header("Screens")
                            .child(navigation_link("Text styles", TextStylesView::default()))
                            .child(navigation_link("Images", ImagesView::default()))
                            .child(navigation_link("Inputs", InputsView::default()))
                            .child(navigation_link("Lists", ListsView::default()))
                            .child(navigation_link("Lazy stacks", LazyView::default())),
                    )
                    .child(
                        section()
                            .header("Appearance")
                            .child(toggle("Dark", &mut self.dark)),
                    )
                    .list_style(ListStyle::Inset)
                    .navigation_title("Gallery"),
            )
            .preferred_color_scheme(if self.dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            })
    }
}

/// The app.
pub struct GalleryApp;

impl App for GalleryApp {
    fn body(&mut self) -> impl Scene {
        window_group(GalleryView::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_has_ids_in_order() {
        let ids: Vec<u32> = sample_photos().iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }
}
