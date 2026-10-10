//! A counter app: `ferroswift build` turns this file into a SwiftUI app.

use ferroswift::prelude::*;

/// The only screen of the app.
#[derive(View, Default)]
pub struct ContentView {
    #[state]
    count: i64,
    #[state(1)]
    step: i64,
}

impl ContentView {
    /// A short comment about the current count.
    fn mood(&self) -> &'static str {
        match self.count {
            i64::MIN..=-1 => "Below zero",
            0 => "Let's go",
            1..=9 => "Warming up",
            _ => "On fire",
        }
    }
}

impl View for ContentView {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(24.0)
            .child(
                text(format!("{}", self.count))
                    .font(Font::system(72.0))
                    .monospaced()
                    .foreground(if self.count < 0 { Color::Red } else { Color::Primary }),
            )
            .child(text(self.mood()).font(Font::Title3).foreground(Color::Secondary))
            .child(
                hstack()
                    .spacing(16.0)
                    .child(button("−", || self.count -= self.step).button_style(ButtonStyle::Bordered))
                    .child(button("+", || self.count += self.step).button_style(ButtonStyle::BorderedProminent)),
            )
            .child(stepper(format!("Step: {}", self.step), &mut self.step, 1..=10))
            .child(button_role("Reset", ButtonRole::Destructive, || self.count = 0).disabled(self.count == 0))
            .padding()
            .animation(Animation::Snappy, self.count)
    }
}

/// The app: one window showing [`ContentView`].
pub struct CounterApp;

impl App for CounterApp {
    fn body(&mut self) -> impl Scene {
        window_group(ContentView::default())
    }
}
