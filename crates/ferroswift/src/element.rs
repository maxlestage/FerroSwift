//! [`Element`]: the value every built-in view function returns, with the SwiftUI modifiers.

use crate::View;
use crate::style::*;
use crate::IndexSet;

/// A built-in view under construction. Create one with a view function such as
/// [`vstack`](crate::vstack) or [`text`](crate::text), add its children, then chain
/// modifiers: each method documents the SwiftUI code it becomes.
///
/// ```
/// use ferroswift::prelude::*;
///
/// let card = vstack()
///     .spacing(8.0)
///     .alignment(Alignment::Leading)
///     .child(text("Title").font(Font::Headline))
///     .child(text("Subtitle").foreground(Color::Secondary))
///     .padding()
///     .background(Color::Yellow)
///     .corner_radius(12.0);
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Element {
    _private: (),
}

impl View for Element {}

impl Element {
    pub(crate) fn new() -> Element {
        Element { _private: () }
    }

    // ----- Content -----

    /// Adds a child to a stack, list, form, section…
    pub fn child(self, _child: impl View) -> Self {
        self
    }

    /// Adds several children: `.children(items.iter().map(|i| text(&i.name)))` becomes a
    /// `ForEach`.
    pub fn children<V: View>(self, _children: impl IntoIterator<Item = V>) -> Self {
        self
    }

    /// `VStack(spacing: n)`, `HStack(spacing: n)`…
    pub fn spacing(self, _spacing: f64) -> Self {
        self
    }

    /// `VStack(alignment: .leading)`, `HStack(alignment: .top)`, `ZStack(alignment: …)`.
    pub fn alignment(self, _alignment: Alignment) -> Self {
        self
    }

    /// `ScrollView(.horizontal)`.
    pub fn axes(self, _axis: Axis) -> Self {
        self
    }

    /// `Section("Header") { … }`.
    pub fn header(self, _header: impl Into<String>) -> Self {
        self
    }

    /// `Slider(…, step: step)`.
    pub fn step(self, _step: f64) -> Self {
        self
    }

    // ----- Layout -----

    /// `.padding()`.
    pub fn padding(self) -> Self {
        self
    }

    /// `.padding(length)`.
    pub fn padding_len(self, _length: f64) -> Self {
        self
    }

    /// `.padding(.horizontal, length)`.
    pub fn padding_edges(self, _edges: Edge, _length: f64) -> Self {
        self
    }

    /// `.frame(width: w, height: h)`.
    pub fn frame(self, _width: f64, _height: f64) -> Self {
        self
    }

    /// `.frame(width: w)`.
    pub fn frame_width(self, _width: f64) -> Self {
        self
    }

    /// `.frame(height: h)`.
    pub fn frame_height(self, _height: f64) -> Self {
        self
    }

    /// `.frame(minWidth: w)`.
    pub fn frame_min_width(self, _width: f64) -> Self {
        self
    }

    /// `.frame(minHeight: h)`.
    pub fn frame_min_height(self, _height: f64) -> Self {
        self
    }

    /// `.frame(maxWidth: w)`; `f64::INFINITY` is `.infinity`.
    pub fn frame_max_width(self, _width: f64) -> Self {
        self
    }

    /// `.frame(maxHeight: h)`.
    pub fn frame_max_height(self, _height: f64) -> Self {
        self
    }

    /// `.frame(maxWidth: .infinity)`: as wide as possible.
    pub fn fill_width(self) -> Self {
        self
    }

    /// `.frame(maxHeight: .infinity)`: as tall as possible.
    pub fn fill_height(self) -> Self {
        self
    }

    /// The `alignment:` of the frame set by the `frame_…` modifiers just before.
    pub fn frame_alignment(self, _alignment: Alignment) -> Self {
        self
    }

    /// `.offset(x: x, y: y)`.
    pub fn offset(self, _x: f64, _y: f64) -> Self {
        self
    }

    /// `.ignoresSafeArea()`.
    pub fn ignores_safe_area(self) -> Self {
        self
    }

    // ----- Text -----

    /// `.font(font)`.
    pub fn font(self, _font: Font) -> Self {
        self
    }

    /// `.font(.system(size: size))`.
    pub fn font_size(self, _size: f64) -> Self {
        self
    }

    /// `.fontWeight(weight)`.
    pub fn font_weight(self, _weight: Weight) -> Self {
        self
    }

    /// `.fontDesign(design)`.
    pub fn font_design(self, _design: Design) -> Self {
        self
    }

    /// `.bold()`.
    pub fn bold(self) -> Self {
        self
    }

    /// `.italic()`.
    pub fn italic(self) -> Self {
        self
    }

    /// `.monospaced()`.
    pub fn monospaced(self) -> Self {
        self
    }

    /// `.underline()`.
    pub fn underline(self) -> Self {
        self
    }

    /// `.strikethrough(active)`.
    pub fn strikethrough(self, _active: bool) -> Self {
        self
    }

    /// `.multilineTextAlignment(alignment)`.
    pub fn multiline_text_alignment(self, _alignment: TextAlignment) -> Self {
        self
    }

    /// `.lineLimit(lines)`.
    pub fn line_limit(self, _lines: usize) -> Self {
        self
    }

    // ----- Colors and effects -----

    /// `.foregroundStyle(color)`.
    pub fn foreground(self, _color: Color) -> Self {
        self
    }

    /// `.background(color)`.
    pub fn background(self, _color: Color) -> Self {
        self
    }

    /// `.background { view }`.
    pub fn background_view(self, _view: impl View) -> Self {
        self
    }

    /// `.overlay { view }`.
    pub fn overlay(self, _view: impl View) -> Self {
        self
    }

    /// `.tint(color)`.
    pub fn tint(self, _color: Color) -> Self {
        self
    }

    /// `.fill(color)`, for shapes.
    pub fn fill(self, _color: Color) -> Self {
        self
    }

    /// `.stroke(color, lineWidth: width)`, for shapes.
    pub fn stroke(self, _color: Color, _width: f64) -> Self {
        self
    }

    /// `.border(color, width: width)`.
    pub fn border(self, _color: Color, _width: f64) -> Self {
        self
    }

    /// `.clipShape(.rect(cornerRadius: radius))`.
    pub fn corner_radius(self, _radius: f64) -> Self {
        self
    }

    /// `.clipShape(.circle)`.
    pub fn clip_circle(self) -> Self {
        self
    }

    /// `.opacity(value)`.
    pub fn opacity(self, _value: f64) -> Self {
        self
    }

    /// `.shadow(radius: radius)`.
    pub fn shadow(self, _radius: f64) -> Self {
        self
    }

    /// `.blur(radius: radius)`.
    pub fn blur(self, _radius: f64) -> Self {
        self
    }

    /// `.rotationEffect(.degrees(degrees))`.
    pub fn rotation(self, _degrees: f64) -> Self {
        self
    }

    /// `.scaleEffect(scale)`.
    pub fn scale(self, _scale: f64) -> Self {
        self
    }

    /// `.glassEffect()` (Liquid Glass, iOS 26 and macOS 26).
    pub fn glass_effect(self) -> Self {
        self
    }

    // ----- Images -----

    /// `.resizable()`.
    pub fn resizable(self) -> Self {
        self
    }

    /// `.scaledToFit()`.
    pub fn scaled_to_fit(self) -> Self {
        self
    }

    /// `.scaledToFill()`.
    pub fn scaled_to_fill(self) -> Self {
        self
    }

    /// `.aspectRatio(contentMode: mode)`.
    pub fn aspect_ratio(self, _mode: ContentMode) -> Self {
        self
    }

    /// `.imageScale(scale)`.
    pub fn image_scale(self, _scale: ImageScale) -> Self {
        self
    }

    // ----- Controls -----

    /// `.disabled(disabled)`.
    pub fn disabled(self, _disabled: bool) -> Self {
        self
    }

    /// `.hidden()`.
    pub fn hidden(self) -> Self {
        self
    }

    /// `.labelsHidden()`: controls without their label.
    pub fn labels_hidden(self) -> Self {
        self
    }

    /// `.buttonStyle(style)`.
    pub fn button_style(self, _style: ButtonStyle) -> Self {
        self
    }

    /// `.textFieldStyle(style)`.
    pub fn text_field_style(self, _style: TextFieldStyle) -> Self {
        self
    }

    /// `.pickerStyle(style)`.
    pub fn picker_style(self, _style: PickerStyle) -> Self {
        self
    }

    /// `.listStyle(style)`.
    pub fn list_style(self, _style: ListStyle) -> Self {
        self
    }

    /// `.tag(value)`, e.g. for the options of a picker.
    pub fn tag<T>(self, _value: T) -> Self {
        self
    }

    /// `.id(value)`.
    pub fn id<T>(self, _value: T) -> Self {
        self
    }

    /// `.badge(count)`.
    pub fn badge(self, _count: i64) -> Self {
        self
    }

    /// `.accessibilityLabel(label)`.
    pub fn accessibility_label(self, _label: impl Into<String>) -> Self {
        self
    }

    /// `.help(text)`.
    pub fn help(self, _text: impl Into<String>) -> Self {
        self
    }

    /// `.preferredColorScheme(scheme)`.
    pub fn preferred_color_scheme(self, _scheme: ColorScheme) -> Self {
        self
    }

    // ----- Navigation and presentation -----

    /// `.navigationTitle(title)`.
    pub fn navigation_title(self, _title: impl Into<String>) -> Self {
        self
    }

    /// `.navigationBarTitleDisplayMode(.inline)` (iOS).
    pub fn navigation_inline(self) -> Self {
        self
    }

    /// `.toolbar { content }`.
    pub fn toolbar(self, _content: impl View) -> Self {
        self
    }

    /// `.tabItem { Label(title, systemImage: icon) }`.
    pub fn tab_item(self, _title: impl Into<String>, _icon: impl Into<String>) -> Self {
        self
    }

    /// `.sheet(isPresented: $flag) { content }`.
    pub fn sheet(self, _is_presented: &mut bool, _content: impl View) -> Self {
        self
    }

    /// `.alert(title, isPresented: $flag) { actions }`.
    pub fn alert(
        self,
        _title: impl Into<String>,
        _is_presented: &mut bool,
        _actions: impl View,
    ) -> Self {
        self
    }

    /// `.searchable(text: $query)`.
    pub fn searchable(self, _query: &mut String) -> Self {
        self
    }

    /// `.swipeActions { content }`.
    pub fn swipe_actions(self, _content: impl View) -> Self {
        self
    }

    /// `.contextMenu { content }`.
    pub fn context_menu(self, _content: impl View) -> Self {
        self
    }

    // ----- Events -----

    /// `.onAppear { action }`.
    pub fn on_appear(self, _action: impl FnMut()) -> Self {
        self
    }

    /// `.onDisappear { action }`.
    pub fn on_disappear(self, _action: impl FnMut()) -> Self {
        self
    }

    /// `.onTapGesture { action }`.
    pub fn on_tap_gesture(self, _action: impl FnMut()) -> Self {
        self
    }

    /// `.onSubmit { action }`.
    pub fn on_submit(self, _action: impl FnMut()) -> Self {
        self
    }

    /// `.onChange(of: value) { action }`.
    pub fn on_change<T>(self, _value: T, _action: impl FnMut()) -> Self {
        self
    }

    /// `.onDelete { offsets in action }`, on the rows of a `for_each`.
    pub fn on_delete(self, _action: impl FnMut(IndexSet)) -> Self {
        self
    }

    /// `.onMove { offsets, destination in action }`, on the rows of a `for_each`.
    pub fn on_move(self, _action: impl FnMut(IndexSet, usize)) -> Self {
        self
    }

    // ----- Animation -----

    /// `.animation(animation, value: value)`.
    pub fn animation<T>(self, _animation: Animation, _value: T) -> Self {
        self
    }

    /// `.transition(transition)`.
    pub fn transition(self, _transition: Transition) -> Self {
        self
    }
}
