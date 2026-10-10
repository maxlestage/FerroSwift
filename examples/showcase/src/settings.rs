//! Form controls bound to state.

use ferroswift::prelude::*;

/// The choices of the appearance picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// Settings: toggles, a slider, a stepper and a picker.
#[derive(View, Default)]
pub struct SettingsView {
    #[state(true)]
    notifications: bool,
    #[state(0.5)]
    volume: f64,
    #[state(3)]
    reminders: i32,
    #[state]
    theme: Theme,
    #[state]
    name: String,
}

impl SettingsView {
    fn summary(&self) -> String {
        let theme = match self.theme {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        };
        format!("{} reminders, {}% volume, {} theme", self.reminders, (self.volume * 100.0).round(), theme)
    }
}

impl View for SettingsView {
    fn body(&mut self) -> impl View {
        navigation_stack().child(
            form()
                .child(
                    section()
                        .header("Profile")
                        .child(text_field("Your name", &mut self.name))
                        .child(text(if self.name.is_empty() {
                            "Hello!".to_string()
                        } else {
                            format!("Hello, {}!", self.name)
                        })),
                )
                .child(
                    section()
                        .header("Notifications")
                        .child(toggle("Notifications", &mut self.notifications))
                        .child(stepper(format!("Reminders: {}", self.reminders), &mut self.reminders, 0..=10).disabled(!self.notifications))
                        .child(slider(&mut self.volume, 0.0..=1.0).step(0.1)),
                )
                .child(
                    section()
                        .header("Appearance")
                        .child(
                            picker("Theme", &mut self.theme)
                                .child(text("System").tag(Theme::System))
                                .child(text("Light").tag(Theme::Light))
                                .child(text("Dark").tag(Theme::Dark)),
                        ),
                )
                .child(section().child(text(self.summary()).font(Font::Footnote).foreground(Color::Secondary)))
                .navigation_title("Settings"),
        )
    }
}
