//! [`Element`]: the value every built-in view function returns, with the SwiftUI modifiers.

use crate::View;
use crate::style::*;
use crate::IndexSet;

/// A built-in view under construction. Create one with a view function such as
/// [`vstack`](crate::vstack) or [`text`](crate::text), add its children, then chain
/// [modifiers](Modifiers): each method documents the SwiftUI code it becomes.
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
}

/// The SwiftUI modifiers, available on every view: the built-in ones and yours
/// (`TodoRow { … }.padding()`). Each method documents the SwiftUI code it becomes.
pub trait Modifiers: View + Sized {
    // ----- Layout -----

    /// `.padding()`.
    fn padding(self) -> Element {
        Element::new()
    }

    /// `.padding(length)`.
    fn padding_len(self, _length: f64) -> Element {
        Element::new()
    }

    /// `.padding(.horizontal, length)`.
    fn padding_edges(self, _edges: Edge, _length: f64) -> Element {
        Element::new()
    }

    /// `.frame(width: w, height: h)`.
    fn frame(self, _width: f64, _height: f64) -> Element {
        Element::new()
    }

    /// `.frame(width: w)`.
    fn frame_width(self, _width: f64) -> Element {
        Element::new()
    }

    /// `.frame(height: h)`.
    fn frame_height(self, _height: f64) -> Element {
        Element::new()
    }

    /// `.frame(minWidth: w)`.
    fn frame_min_width(self, _width: f64) -> Element {
        Element::new()
    }

    /// `.frame(minHeight: h)`.
    fn frame_min_height(self, _height: f64) -> Element {
        Element::new()
    }

    /// `.frame(maxWidth: w)`; `f64::INFINITY` is `.infinity`.
    fn frame_max_width(self, _width: f64) -> Element {
        Element::new()
    }

    /// `.frame(maxHeight: h)`.
    fn frame_max_height(self, _height: f64) -> Element {
        Element::new()
    }

    /// `.frame(maxWidth: .infinity)`: as wide as possible.
    fn fill_width(self) -> Element {
        Element::new()
    }

    /// `.frame(maxHeight: .infinity)`: as tall as possible.
    fn fill_height(self) -> Element {
        Element::new()
    }

    /// The `alignment:` of the frame set by the `frame_…` modifiers just before.
    fn frame_alignment(self, _alignment: Alignment) -> Element {
        Element::new()
    }

    /// `.offset(x: x, y: y)`.
    fn offset(self, _x: f64, _y: f64) -> Element {
        Element::new()
    }

    /// `.ignoresSafeArea()`.
    fn ignores_safe_area(self) -> Element {
        Element::new()
    }

    // ----- Text -----

    /// `.font(font)`.
    fn font(self, _font: Font) -> Element {
        Element::new()
    }

    /// `.font(.system(size: size))`.
    fn font_size(self, _size: f64) -> Element {
        Element::new()
    }

    /// `.fontWeight(weight)`.
    fn font_weight(self, _weight: Weight) -> Element {
        Element::new()
    }

    /// `.fontDesign(design)`.
    fn font_design(self, _design: Design) -> Element {
        Element::new()
    }

    /// `.bold()`.
    fn bold(self) -> Element {
        Element::new()
    }

    /// `.italic()`.
    fn italic(self) -> Element {
        Element::new()
    }

    /// `.monospaced()`.
    fn monospaced(self) -> Element {
        Element::new()
    }

    /// `.underline()`.
    fn underline(self) -> Element {
        Element::new()
    }

    /// `.strikethrough(active)`.
    fn strikethrough(self, _active: bool) -> Element {
        Element::new()
    }

    /// `.multilineTextAlignment(alignment)`.
    fn multiline_text_alignment(self, _alignment: TextAlignment) -> Element {
        Element::new()
    }

    /// `.lineLimit(lines)`.
    fn line_limit(self, _lines: usize) -> Element {
        Element::new()
    }

    // ----- Colors and effects -----

    /// `.foregroundStyle(color)`.
    fn foreground(self, _color: Color) -> Element {
        Element::new()
    }

    /// `.background(color)`.
    fn background(self, _color: Color) -> Element {
        Element::new()
    }

    /// `.background { view }`.
    fn background_view(self, _view: impl View) -> Element {
        Element::new()
    }

    /// `.overlay { view }`.
    fn overlay(self, _view: impl View) -> Element {
        Element::new()
    }

    /// `.tint(color)`.
    fn tint(self, _color: Color) -> Element {
        Element::new()
    }

    /// `.fill(color)`, for shapes.
    fn fill(self, _color: Color) -> Element {
        Element::new()
    }

    /// `.stroke(color, lineWidth: width)`, for shapes.
    fn stroke(self, _color: Color, _width: f64) -> Element {
        Element::new()
    }

    /// `.border(color, width: width)`.
    fn border(self, _color: Color, _width: f64) -> Element {
        Element::new()
    }

    /// `.clipShape(.rect(cornerRadius: radius))`.
    fn corner_radius(self, _radius: f64) -> Element {
        Element::new()
    }

    /// `.clipShape(.circle)`.
    fn clip_circle(self) -> Element {
        Element::new()
    }

    /// `.opacity(value)`.
    fn opacity(self, _value: f64) -> Element {
        Element::new()
    }

    /// `.shadow(radius: radius)`.
    fn shadow(self, _radius: f64) -> Element {
        Element::new()
    }

    /// `.blur(radius: radius)`.
    fn blur(self, _radius: f64) -> Element {
        Element::new()
    }

    /// `.rotationEffect(.degrees(degrees))`.
    fn rotation(self, _degrees: f64) -> Element {
        Element::new()
    }

    /// `.scaleEffect(scale)`.
    fn scale(self, _scale: f64) -> Element {
        Element::new()
    }

    /// `.glassEffect()` (Liquid Glass, iOS 26 and macOS 26).
    fn glass_effect(self) -> Element {
        Element::new()
    }

    // ----- Images -----

    /// `.resizable()`.
    fn resizable(self) -> Element {
        Element::new()
    }

    /// `.scaledToFit()`.
    fn scaled_to_fit(self) -> Element {
        Element::new()
    }

    /// `.scaledToFill()`.
    fn scaled_to_fill(self) -> Element {
        Element::new()
    }

    /// `.aspectRatio(contentMode: mode)`.
    fn aspect_ratio(self, _mode: ContentMode) -> Element {
        Element::new()
    }

    /// `.imageScale(scale)`.
    fn image_scale(self, _scale: ImageScale) -> Element {
        Element::new()
    }

    // ----- Controls -----

    /// `.disabled(disabled)`.
    fn disabled(self, _disabled: bool) -> Element {
        Element::new()
    }

    /// `.hidden()`.
    fn hidden(self) -> Element {
        Element::new()
    }

    /// `.labelsHidden()`: controls without their label.
    fn labels_hidden(self) -> Element {
        Element::new()
    }

    /// `.buttonStyle(style)`.
    fn button_style(self, _style: ButtonStyle) -> Element {
        Element::new()
    }

    /// `.textFieldStyle(style)`.
    fn text_field_style(self, _style: TextFieldStyle) -> Element {
        Element::new()
    }

    /// `.pickerStyle(style)`.
    fn picker_style(self, _style: PickerStyle) -> Element {
        Element::new()
    }

    /// `.listStyle(style)`.
    fn list_style(self, _style: ListStyle) -> Element {
        Element::new()
    }

    /// `.tag(value)`, e.g. for the options of a picker.
    fn tag<T>(self, _value: T) -> Element {
        Element::new()
    }

    /// `.id(value)`.
    fn id<T>(self, _value: T) -> Element {
        Element::new()
    }

    /// `.badge(count)`.
    fn badge(self, _count: i64) -> Element {
        Element::new()
    }

    /// `.accessibilityLabel(label)`.
    fn accessibility_label(self, _label: impl Into<String>) -> Element {
        Element::new()
    }

    /// `.help(text)`.
    fn help(self, _text: impl Into<String>) -> Element {
        Element::new()
    }

    /// `.preferredColorScheme(scheme)`.
    fn preferred_color_scheme(self, _scheme: ColorScheme) -> Element {
        Element::new()
    }

    // ----- Navigation and presentation -----

    /// `.navigationTitle(title)`.
    fn navigation_title(self, _title: impl Into<String>) -> Element {
        Element::new()
    }

    /// `.navigationBarTitleDisplayMode(.inline)` (iOS).
    fn navigation_inline(self) -> Element {
        Element::new()
    }

    /// `.toolbar { content }`.
    fn toolbar(self, _content: impl View) -> Element {
        Element::new()
    }

    /// `.tabItem { Label(title, systemImage: icon) }`.
    fn tab_item(self, _title: impl Into<String>, _icon: impl Into<String>) -> Element {
        Element::new()
    }

    /// `.sheet(isPresented: $flag) { content }`.
    fn sheet(self, _is_presented: &mut bool, _content: impl View) -> Element {
        Element::new()
    }

    /// `.alert(title, isPresented: $flag) { actions }`.
    fn alert(
        self,
        _title: impl Into<String>,
        _is_presented: &mut bool,
        _actions: impl View,
    ) -> Element {
        Element::new()
    }

    /// `.searchable(text: $query)`.
    fn searchable(self, _query: &mut String) -> Element {
        Element::new()
    }

    /// `.swipeActions { content }`.
    fn swipe_actions(self, _content: impl View) -> Element {
        Element::new()
    }

    /// `.contextMenu { content }`.
    fn context_menu(self, _content: impl View) -> Element {
        Element::new()
    }

    // ----- Events -----

    /// `.onAppear { action }`.
    fn on_appear(self, _action: impl FnMut()) -> Element {
        Element::new()
    }

    /// `.onDisappear { action }`.
    fn on_disappear(self, _action: impl FnMut()) -> Element {
        Element::new()
    }

    /// `.onTapGesture { action }`.
    fn on_tap_gesture(self, _action: impl FnMut()) -> Element {
        Element::new()
    }

    /// `.onSubmit { action }`.
    fn on_submit(self, _action: impl FnMut()) -> Element {
        Element::new()
    }

    /// `.onChange(of: value) { action }`.
    fn on_change<T>(self, _value: T, _action: impl FnMut()) -> Element {
        Element::new()
    }

    /// `.onDelete { offsets in action }`, on the rows of a `for_each`.
    fn on_delete(self, _action: impl FnMut(IndexSet)) -> Element {
        Element::new()
    }

    /// `.onMove { offsets, destination in action }`, on the rows of a `for_each`.
    fn on_move(self, _action: impl FnMut(IndexSet, usize)) -> Element {
        Element::new()
    }

    // ----- Animation -----

    /// `.animation(animation, value: value)`.
    fn animation<T>(self, _animation: Animation, _value: T) -> Element {
        Element::new()
    }

    /// `.transition(transition)`.
    fn transition(self, _transition: Transition) -> Element {
        Element::new()
    }
}

impl<V: View> Modifiers for V {}
