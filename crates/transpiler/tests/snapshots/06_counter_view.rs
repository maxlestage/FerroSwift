use ferroswift::prelude::*;

/// A counter.
#[derive(View, Default)]
pub struct Counter {
    #[state]
    count: i32,
    title: String,
}

impl View for Counter {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(12.0)
            .alignment(Alignment::Leading)
            .child(text(&self.title).font(Font::Headline))
            .child(text(format!("Count: {}", self.count)).font(Font::LargeTitle).bold())
            .child(
                hstack()
                    .child(button("−", || self.count -= 1))
                    .child(button("+", || self.count += 1)),
            )
            .padding()
            .fill_width()
            .frame_alignment(Alignment::Leading)
            .background(Color::Yellow.opacity(0.2))
            .corner_radius(16.0)
    }
}

#[preview]
fn preview() -> impl View {
    Counter { count: 3, title: "Preview".into() }
}
