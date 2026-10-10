//! Built-in views. Each function documents the SwiftUI code it becomes.

use std::ops::RangeInclusive;

use crate::element::Element;
use crate::style::ButtonRole;
use crate::View;

/// `Text(content)`. With `format!`, `Text("Count: \(count)")`.
pub fn text(_content: impl Into<String>) -> Element {
    Element::new()
}

/// `Button(label) { action }`.
pub fn button(_label: impl Into<String>, _action: impl FnMut()) -> Element {
    Element::new()
}

/// `Button(label, role: role) { action }`.
pub fn button_role(
    _label: impl Into<String>,
    _role: ButtonRole,
    _action: impl FnMut(),
) -> Element {
    Element::new()
}

/// `Button { action } label: { label }`: a button showing any view.
pub fn button_label(_action: impl FnMut(), _label: impl View) -> Element {
    Element::new()
}

/// `VStack { children }`; see [`Element::spacing`] and [`Element::alignment`].
pub fn vstack() -> Element {
    Element::new()
}

/// `HStack { children }`.
pub fn hstack() -> Element {
    Element::new()
}

/// `ZStack { children }`.
pub fn zstack() -> Element {
    Element::new()
}

/// `LazyVStack { children }`.
pub fn lazy_vstack() -> Element {
    Element::new()
}

/// `LazyHStack { children }`.
pub fn lazy_hstack() -> Element {
    Element::new()
}

/// `ScrollView { children }`; [`Element::axes`] for `ScrollView(.horizontal)`.
pub fn scroll_view() -> Element {
    Element::new()
}

/// `List { children }`.
pub fn list() -> Element {
    Element::new()
}

/// `Form { children }`.
pub fn form() -> Element {
    Element::new()
}

/// `Section { children }`; [`Element::header`] for `Section("Header") { … }`.
pub fn section() -> Element {
    Element::new()
}

/// `Group { children }`.
pub fn group() -> Element {
    Element::new()
}

/// `NavigationStack { children }`.
pub fn navigation_stack() -> Element {
    Element::new()
}

/// `TabView { children }`; give each child a [`Element::tab_item`].
pub fn tab_view() -> Element {
    Element::new()
}

/// `DisclosureGroup(label) { children }`.
pub fn disclosure_group(_label: impl Into<String>) -> Element {
    Element::new()
}

/// `ForEach(items) { item in row }`: one row per item. Items whose type has an `id` field
/// are `Identifiable`; numbers and strings use `id: \.self`.
pub fn for_each<T, V: View>(
    items: impl IntoIterator<Item = T>,
    mut row: impl FnMut(T) -> V,
) -> Element {
    for item in items {
        let _ = row(item);
    }
    Element::new()
}

/// `NavigationLink(label) { destination }`.
pub fn navigation_link(_label: impl Into<String>, _destination: impl View) -> Element {
    Element::new()
}

/// `NavigationLink { destination } label: { label }`.
pub fn navigation_link_label(_destination: impl View, _label: impl View) -> Element {
    Element::new()
}

/// `Spacer()`.
pub fn spacer() -> Element {
    Element::new()
}

/// `Divider()`.
pub fn divider() -> Element {
    Element::new()
}

/// `EmptyView()`.
pub fn empty_view() -> Element {
    Element::new()
}

/// `Image(name)`: an image of the asset catalog.
pub fn image(_name: impl Into<String>) -> Element {
    Element::new()
}

/// `Image(systemName: name)`: an SF Symbol.
pub fn system_image(_name: impl Into<String>) -> Element {
    Element::new()
}

/// `Label(title, systemImage: icon)`.
pub fn label(_title: impl Into<String>, _icon: impl Into<String>) -> Element {
    Element::new()
}

/// `AsyncImage(url: URL(string: url))`.
pub fn async_image(_url: impl Into<String>) -> Element {
    Element::new()
}

/// `Link(label, destination: URL(string: url)!)`.
pub fn link(_label: impl Into<String>, _url: impl Into<String>) -> Element {
    Element::new()
}

/// `Toggle(label, isOn: $value)`.
pub fn toggle(_label: impl Into<String>, _is_on: &mut bool) -> Element {
    Element::new()
}

/// `TextField(placeholder, text: $value)`.
pub fn text_field(_placeholder: impl Into<String>, _text: &mut String) -> Element {
    Element::new()
}

/// `SecureField(placeholder, text: $value)`.
pub fn secure_field(_placeholder: impl Into<String>, _text: &mut String) -> Element {
    Element::new()
}

/// `TextEditor(text: $value)`.
pub fn text_editor(_text: &mut String) -> Element {
    Element::new()
}

/// `Slider(value: $value, in: range)`; [`Element::step`] for `step:`.
pub fn slider<T>(_value: &mut T, _range: RangeInclusive<T>) -> Element {
    Element::new()
}

/// `Stepper(label, value: $value, in: range)`.
pub fn stepper<T>(_label: impl Into<String>, _value: &mut T, _range: RangeInclusive<T>) -> Element {
    Element::new()
}

/// `Picker(label, selection: $value) { children }`; tag each child with [`Element::tag`].
pub fn picker<T>(_label: impl Into<String>, _selection: &mut T) -> Element {
    Element::new()
}

/// `DatePicker(label, selection: $date)`; the date is a number of seconds since 1970.
pub fn date_picker(_label: impl Into<String>, _date: &mut f64) -> Element {
    Element::new()
}

/// `ProgressView()`: a spinner.
pub fn progress_view() -> Element {
    Element::new()
}

/// `ProgressView(value: value, total: total)`: a progress bar.
pub fn progress(_value: f64, _total: f64) -> Element {
    Element::new()
}

/// `Rectangle()`.
pub fn rectangle() -> Element {
    Element::new()
}

/// `Circle()`.
pub fn circle() -> Element {
    Element::new()
}

/// `Capsule()`.
pub fn capsule() -> Element {
    Element::new()
}

/// `Ellipse()`.
pub fn ellipse() -> Element {
    Element::new()
}

/// `RoundedRectangle(cornerRadius: radius)`.
pub fn rounded_rectangle(_radius: f64) -> Element {
    Element::new()
}

/// `ContentUnavailableView(title, systemImage: icon, description: Text(description))`.
pub fn content_unavailable(
    _title: impl Into<String>,
    _icon: impl Into<String>,
    _description: impl Into<String>,
) -> Element {
    Element::new()
}
