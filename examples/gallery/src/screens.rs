//! One screen per family of views and modifiers.

use ferroswift::prelude::*;

use crate::{Photo, sample_photos};

/// Fonts, weights, designs, decorations, frames and padding.
#[derive(View, Default)]
pub struct TextStylesView {
    #[state]
    big: bool,
}

impl View for TextStylesView {
    fn body(&mut self) -> impl View {
        scroll_view()
            .child(
                vstack()
                    .alignment(Alignment::Leading)
                    .spacing(12.0)
                    .child(text("Large title").font(Font::LargeTitle).bold())
                    .child(text("Rounded headline").font(Font::Headline).font_design(Design::Rounded))
                    .child(text("Serif body").font(Font::Body).font_design(Design::Serif))
                    .child(text("Heavy").font_weight(Weight::Heavy))
                    .child(text("Thin and italic").font_weight(Weight::Thin).italic())
                    .child(text("Underlined").underline())
                    .child(text("Struck").strikethrough(true))
                    .child(text("Monospaced").monospaced())
                    .child(text("Custom size").font_size(if self.big { 28.0 } else { 18.0 }))
                    .child(text("Caption").font(Font::Caption))
                    .child(
                        text("A long paragraph that wraps over several lines, centered, and limited to two lines of text.")
                            .multiline_text_alignment(TextAlignment::Center)
                            .line_limit(2)
                            .frame_max_width(260.0),
                    )
                    .child(text("Padded").padding_len(24.0).background(Color::Yellow))
                    .child(text("Edges").padding_edges(Edge::Horizontal, 32.0).background(Color::Mint))
                    .child(text("Fixed").frame(120.0, 44.0).border(Color::Gray, 1.0))
                    .child(text("Wide").frame_width(200.0).background(Color::Teal.opacity(0.3)))
                    .child(text("Tall").frame_height(60.0).background(Color::Cyan.opacity(0.3)))
                    .child(text("Bounded").frame_min_width(80.0).frame_max_height(40.0).frame_min_height(20.0))
                    .child(text("Offset").offset(10.0, -4.0))
                    .child(text("Hidden").hidden())
                    .child(text("Labelled").accessibility_label("A label for assistive technologies"))
                    .child(text("Hover me").help("A tooltip"))
                    .child(
                        button("Toggle size", || self.big = !self.big)
                            .button_style(ButtonStyle::Bordered)
                            .tint(Color::Indigo),
                    )
                    .padding(),
            )
            .navigation_title("Text styles")
            .navigation_inline()
    }
}

/// Images, labels, links and visual effects.
#[derive(View, Default)]
pub struct ImagesView {
    #[state(0.0)]
    angle: f64,
}

impl View for ImagesView {
    fn body(&mut self) -> impl View {
        scroll_view()
            .child(
                vstack()
                    .spacing(16.0)
                    .child(
                        image("Logo")
                            .resizable()
                            .scaled_to_fit()
                            .frame_width(120.0)
                            .clip_circle()
                            .shadow(4.0),
                    )
                    .child(
                        system_image("photo")
                            .resizable()
                            .aspect_ratio(ContentMode::Fill)
                            .frame(80.0, 60.0)
                            .corner_radius(8.0)
                            .border(Color::Secondary, 2.0),
                    )
                    .child(
                        system_image("star.fill")
                            .image_scale(ImageScale::Large)
                            .foreground(Color::Yellow),
                    )
                    .child(
                        system_image("cloud")
                            .scaled_to_fill()
                            .frame_height(30.0)
                            .blur(1.5),
                    )
                    .child(
                        system_image("arrow.right")
                            .rotation(self.angle)
                            .scale(1.5)
                            .animation(Animation::Spring, self.angle),
                    )
                    .child(button("Spin", || self.angle += 90.0))
                    .child(label("Favourites", "heart").badge(3))
                    .child(link("Rust", "https://www.rust-lang.org"))
                    .child(async_image("https://example.com/photo.jpg").frame(100.0, 100.0))
                    .child(
                        text("Over a shape")
                            .padding()
                            .background_view(rounded_rectangle(12.0).fill(Color::Blue.opacity(0.2)))
                            .overlay(rounded_rectangle(12.0).stroke(Color::Blue, 1.0)),
                    )
                    .child(
                        zstack()
                            .child(ellipse().fill(Color::Orange).frame(120.0, 60.0))
                            .child(text("Ellipse").foreground(Color::White)),
                    )
                    .padding(),
            )
            .navigation_title("Images")
    }
}

/// Text fields, editors, steppers, sliders, progress and groups.
#[derive(View, Default)]
pub struct InputsView {
    #[state]
    name: String,
    #[state]
    password: String,
    #[state]
    notes: String,
    #[state(0.3)]
    progress: f64,
    #[state(0)]
    submissions: u32,
    #[state(0)]
    edits: u32,
}

impl View for InputsView {
    fn body(&mut self) -> impl View {
        form()
            .child(
                section()
                    .header("Account")
                    .child(
                        text_field("Name", &mut self.name)
                            .text_field_style(TextFieldStyle::RoundedBorder)
                            .on_submit(|| self.submissions += 1)
                            .on_change(self.name.len(), || self.edits += 1),
                    )
                    .child(secure_field("Password", &mut self.password))
                    .child(
                        text(format!(
                            "{} submissions, {} edits",
                            self.submissions, self.edits
                        ))
                        .font(Font::Footnote),
                    ),
            )
            .child(
                section()
                    .header("Notes")
                    .child(text_editor(&mut self.notes).frame_height(80.0))
                    .child(if self.notes.is_empty() {
                        text("Empty").foreground(Color::Secondary)
                    } else {
                        empty_view()
                    }),
            )
            .child(
                section()
                    .header("Progress")
                    .child(progress(self.progress, 1.0))
                    .child(progress_view())
                    .child(slider(&mut self.progress, 0.0..=1.0))
                    .child(
                        group()
                            .child(divider())
                            .child(text(format!("{:.0}%", self.progress * 100.0)))
                            .child(divider()),
                    ),
            )
            .child(
                disclosure_group("Advanced")
                    .child(text("Shown when expanded"))
                    .child(button_role("Reset", ButtonRole::Destructive, || {
                        self.progress = 0.0
                    })),
            )
            .navigation_title("Inputs")
    }
}

/// A list with search, editing, context menus and swipe actions.
#[derive(View, Default)]
pub struct ListsView {
    #[state]
    photos: Vec<Photo>,
    #[state]
    query: String,
    #[state]
    selected: Option<u32>,
}

impl ListsView {
    fn visible(&self) -> Vec<Photo> {
        self.photos
            .iter()
            .filter(|p| {
                self.query.is_empty() || p.title.to_lowercase().contains(&self.query.to_lowercase())
            })
            .cloned()
            .collect()
    }
}

impl View for ListsView {
    fn body(&mut self) -> impl View {
        list()
            .child(
                section()
                    .header(format!("{} photos", self.photos.len()))
                    .child(
                        for_each(self.visible(), |photo| {
                            navigation_link_label(
                                PhotoDetail {
                                    photo: photo.clone(),
                                },
                                hstack().child(text(&photo.title)).child(spacer()).child(
                                    text(format!("{}", photo.likes)).foreground(Color::Secondary),
                                ),
                            )
                            .id(photo.id)
                            .context_menu(button("Select", || self.selected = Some(photo.id)))
                            .swipe_actions(button_role(
                                "Delete",
                                ButtonRole::Destructive,
                                || self.photos.retain(|p| p.id != photo.id),
                            ))
                        })
                        .on_delete(|offsets| self.photos.remove_offsets(offsets))
                        .on_move(|from, to| self.photos.move_offsets(from, to)),
                    ),
            )
            .child(if let Some(id) = self.selected {
                text(format!("Selected #{id}"))
            } else {
                text("Nothing selected")
            })
            .list_style(ListStyle::Plain)
            .searchable(&mut self.query)
            .on_appear(|| self.photos = sample_photos())
            .on_disappear(|| self.selected = None)
            .navigation_title("Lists")
    }
}

/// One photo.
#[derive(View)]
pub struct PhotoDetail {
    pub photo: Photo,
}

impl View for PhotoDetail {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(8.0)
            .child(text(&self.photo.title).font(Font::Title))
            .child(text(format!("{} likes", self.photo.likes)))
            .child(
                button_label(
                    || {},
                    hstack().child(system_image("heart")).child(text("Like")),
                )
                .button_style(ButtonStyle::BorderedProminent),
            )
            .padding()
            .navigation_title(&self.photo.title)
    }
}

/// Lazy stacks of shapes, with transitions.
#[derive(View, Default)]
pub struct LazyView {
    #[state(true)]
    show_banner: bool,
}

impl View for LazyView {
    fn body(&mut self) -> impl View {
        scroll_view()
            .child(
                lazy_vstack()
                    .spacing(12.0)
                    .child(if self.show_banner {
                        text("Banner")
                            .fill_width()
                            .padding()
                            .background(Color::Purple)
                            .foreground(Color::White)
                            .transition(Transition::Slide)
                    } else {
                        empty_view()
                    })
                    .child(button("Toggle banner", || {
                        with_animation(|| self.show_banner = !self.show_banner)
                    }))
                    .child(scroll_view().axes(Axis::Horizontal).child(
                        lazy_hstack().spacing(8.0).child(for_each(0..20, |i| {
                            rectangle()
                                .fill(if i % 2 == 0 {
                                    Color::Pink
                                } else {
                                    Color::Brown
                                })
                                .frame(60.0, 60.0)
                                .corner_radius(6.0)
                        })),
                    ))
                    .child(
                        capsule()
                            .fill(Color::Green)
                            .frame_height(30.0)
                            .fill_height(),
                    )
                    .child(circle().stroke(Color::Red, 3.0).frame(50.0, 50.0))
                    .child(content_unavailable(
                        "Nothing here",
                        "tray",
                        "Add something first",
                    ))
                    .padding(),
            )
            .ignores_safe_area()
            .navigation_title("Lazy stacks")
    }
}
