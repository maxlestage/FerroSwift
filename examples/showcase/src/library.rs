//! A list of books in an observable model, a detail screen and a sheet to add one.

use ferroswift::prelude::*;

/// A book of the library.
#[derive(Debug, Clone, PartialEq)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub rating: u8,
    pub read: bool,
}

impl Book {
    /// "★★★☆☆".
    pub fn stars(&self) -> String {
        let full = "★".repeat(self.rating as usize);
        let empty = "☆".repeat(5 - self.rating as usize);
        format!("{full}{empty}")
    }
}

/// The library: a class shared by every screen that shows it.
#[derive(Observable, Default, Clone)]
pub struct Library {
    pub books: Vec<Book>,
    pub query: String,
}

impl Library {
    pub fn add(&mut self, title: &str, author: &str) {
        let id = self.books.iter().map(|b| b.id).max().unwrap_or(0) + 1;
        self.books.push(Book {
            id,
            title: title.to_string(),
            author: author.to_string(),
            rating: 3,
            read: false,
        });
    }

    /// The books whose title or author contains the query.
    pub fn visible(&self) -> Vec<Book> {
        let q = self.query.to_lowercase();
        self.books
            .iter()
            .filter(|b| {
                q.is_empty()
                    || b.title.to_lowercase().contains(&q)
                    || b.author.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }
}

/// The list of books.
#[derive(View)]
pub struct LibraryView {
    pub library: Library,
}

#[derive(View, Default)]
struct AddBookButton {
    #[state]
    showing: bool,
}

impl View for LibraryView {
    fn body(&mut self) -> impl View {
        navigation_stack().child(
            list()
                .child(if self.library.books.is_empty() {
                    content_unavailable("No books", "books.vertical", "Add your first book with +.")
                } else {
                    text(format!("{} books", self.library.books.len())).foreground(Color::Secondary)
                })
                .child(for_each(self.library.visible(), |book| {
                    navigation_link_label(
                        BookDetail { book: book.clone() },
                        BookRow { book: book.clone() },
                    )
                    .swipe_actions(button_role(
                        "Delete",
                        ButtonRole::Destructive,
                        || self.library.books.retain(|b| b.id != book.id),
                    ))
                }))
                .navigation_title("Library")
                .searchable(&mut self.library.query)
                .toolbar(AddBookButton::default()),
        )
    }
}

impl View for AddBookButton {
    fn body(&mut self) -> impl View {
        button_label(|| self.showing = true, label("Add", "plus"))
            .sheet(&mut self.showing, NewBookForm::default())
    }
}

/// A row: title, author and rating.
#[derive(View)]
struct BookRow {
    book: Book,
}

impl View for BookRow {
    fn body(&mut self) -> impl View {
        hstack()
            .child(
                vstack()
                    .alignment(Alignment::Leading)
                    .child(text(&self.book.title).font(Font::Headline))
                    .child(
                        text(&self.book.author)
                            .font(Font::Subheadline)
                            .foreground(Color::Secondary),
                    ),
            )
            .child(spacer())
            .child(text(self.book.stars()).foreground(Color::Orange))
    }
}

/// The detail of a book.
#[derive(View)]
struct BookDetail {
    book: Book,
}

impl View for BookDetail {
    fn body(&mut self) -> impl View {
        scroll_view()
            .child(
                vstack()
                    .alignment(Alignment::Leading)
                    .spacing(12.0)
                    .child(text(&self.book.title).font(Font::LargeTitle).bold())
                    .child(label(&self.book.author, "person"))
                    .child(
                        text(self.book.stars())
                            .font(Font::Title2)
                            .foreground(Color::Orange),
                    )
                    .child(if self.book.read {
                        label("Read", "checkmark.circle.fill").foreground(Color::Green)
                    } else {
                        label("Not read yet", "circle").foreground(Color::Secondary)
                    })
                    .padding()
                    .frame_max_width(f64::INFINITY)
                    .frame_alignment(Alignment::Leading),
            )
            .navigation_title(&self.book.title)
            .navigation_inline()
    }
}

/// A form in a sheet.
#[derive(View, Default)]
struct NewBookForm {
    #[state]
    title: String,
    #[state]
    author: String,
    #[state]
    confirm: bool,
    #[environment(dismiss)]
    dismiss: DismissAction,
}

impl View for NewBookForm {
    fn body(&mut self) -> impl View {
        navigation_stack().child(
            form()
                .child(text_field("Title", &mut self.title))
                .child(text_field("Author", &mut self.author))
                .child(
                    button("Save", || self.confirm = true)
                        .disabled(self.title.is_empty() || self.author.is_empty()),
                )
                .navigation_title("New book")
                .alert(
                    format!("Add “{}”?", self.title),
                    &mut self.confirm,
                    button("Add", || self.dismiss.call()),
                ),
        )
    }
}
