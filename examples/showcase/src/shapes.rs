//! Shapes, colors, overlays and animations.

use ferroswift::prelude::*;

/// Circles that grow when tapped.
#[derive(View, Default)]
pub struct ShapesView {
    #[state]
    big: bool,
    #[state]
    taps: u32,
}

impl ShapesView {
    fn palette() -> Vec<Color> {
        vec![Color::Red, Color::Orange, Color::Yellow, Color::Green, Color::Blue]
    }
}

impl View for ShapesView {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(24.0)
            .child(
                zstack()
                    .child(circle().fill(Color::hex(0xF05138)).frame(if self.big { 200.0 } else { 120.0 }, if self.big { 200.0 } else { 120.0 }))
                    .child(text(format!("{}", self.taps)).font(Font::system_weight(40.0, Weight::Bold)).foreground(Color::White))
                    .on_tap_gesture(|| {
                        with_animation_of(Animation::Bouncy, || {
                            self.big = !self.big;
                            self.taps += 1;
                        })
                    }),
            )
            .child(
                hstack().spacing(8.0).children(Self::palette().into_iter().map(|color| {
                    rounded_rectangle(8.0)
                        .fill(color)
                        .frame(40.0, 40.0)
                        .shadow(2.0)
                })),
            )
            .child(
                text("Tap the circle")
                    .padding()
                    .background(Color::Gray.opacity(0.15))
                    .corner_radius(12.0)
                    .overlay(capsule().stroke(Color::Orange, 2.0)),
            )
            .child(progress(self.taps as f64 % 10.0, 10.0).tint(Color::Orange))
            .padding()
    }
}
