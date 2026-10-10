use ferroswift::prelude::*;

/// A counter: tap the buttons, the number follows.
#[derive(View, Default)]
pub struct Counter {
    #[state]
    count: i64,
}

impl View for Counter {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(16.0)
            .child(
                text(format!("{}", self.count))
                    .font(Font::LargeTitle)
                    .bold()
                    .foreground(if self.count < 0 { Color::Red } else { Color::Orange }),
            )
            .child(
                hstack()
                    .spacing(12.0)
                    .child(button("−1", || self.count -= 1))
                    .child(button("+1", || self.count += 1))
                    .button_style(ButtonStyle::Bordered),
            )
            .child(
                button_role("Reset", ButtonRole::Destructive, || self.count = 0)
                    .disabled(self.count == 0),
            )
            .padding()
    }
}

/// The app: one window showing the counter.
struct CounterApp;

impl App for CounterApp {
    fn body(&mut self) -> impl Scene {
        window_group(Counter::default())
    }
}
