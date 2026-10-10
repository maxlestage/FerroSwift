//! Colors, fonts and the other values given to modifiers. Variants become SwiftUI's
//! members in lowerCamelCase: `Font::LargeTitle` is `.largeTitle`, `Color::Red` is `.red`.

use crate::View;

/// A color, usable as a view or as a style: `Color.red`, `Color(red:green:blue:)`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Mint,
    Teal,
    Cyan,
    Blue,
    Indigo,
    Purple,
    Pink,
    Brown,
    White,
    Gray,
    Black,
    Clear,
    Primary,
    Secondary,
    /// `Color.accentColor`.
    Accent,
    #[doc(hidden)]
    Custom,
}

impl Color {
    /// `Color(red: r, green: g, blue: b)`, each component between 0 and 1.
    pub fn rgb(_red: f64, _green: f64, _blue: f64) -> Color {
        Color::Custom
    }

    /// `Color(red: r, green: g, blue: b, opacity: a)`.
    pub fn rgba(_red: f64, _green: f64, _blue: f64, _opacity: f64) -> Color {
        Color::Custom
    }

    /// A color from `0xRRGGBB`, e.g. `Color::hex(0xFF5A1F)`.
    pub fn hex(_rgb: u32) -> Color {
        Color::Custom
    }

    /// `color.opacity(value)`.
    #[must_use]
    pub fn opacity(self, _value: f64) -> Color {
        self
    }
}

impl View for Color {}

/// A font: `.font(.title)`, `.font(.system(size: 20))`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum Font {
    LargeTitle,
    Title,
    Title2,
    Title3,
    Headline,
    Subheadline,
    Body,
    Callout,
    Footnote,
    Caption,
    Caption2,
    #[doc(hidden)]
    Custom,
}

impl Font {
    /// `.system(size: size)`.
    pub fn system(_size: f64) -> Font {
        Font::Custom
    }

    /// `.system(size: size, weight: weight)`.
    pub fn system_weight(_size: f64, _weight: Weight) -> Font {
        Font::Custom
    }

    /// `.system(size: size, design: design)`.
    pub fn system_design(_size: f64, _design: Design) -> Font {
        Font::Custom
    }
}

/// `Font.Weight`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Weight {
    UltraLight,
    Thin,
    Light,
    Regular,
    Medium,
    Semibold,
    Bold,
    Heavy,
    Black,
}

/// `Font.Design`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Design {
    Default,
    Serif,
    Rounded,
    Monospaced,
}

/// Alignments of stacks and frames. A `VStack` takes the horizontal ones (`Leading`,
/// `Center`, `Trailing`), an `HStack` the vertical ones (`Top`, `Center`, `Bottom`,
/// `FirstTextBaseline`, `LastTextBaseline`), a `ZStack` or a frame any of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Alignment {
    Center,
    Leading,
    Trailing,
    Top,
    Bottom,
    TopLeading,
    TopTrailing,
    BottomLeading,
    BottomTrailing,
    FirstTextBaseline,
    LastTextBaseline,
}

/// Edges for padding: `Edge.Set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Edge {
    All,
    Top,
    Bottom,
    Leading,
    Trailing,
    Horizontal,
    Vertical,
}

/// `Axis.Set` of a scroll view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// `TextAlignment`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TextAlignment {
    Leading,
    Center,
    Trailing,
}

/// List styles: `.listStyle(.insetGrouped)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ListStyle {
    Automatic,
    Plain,
    Inset,
    Grouped,
    InsetGrouped,
    Sidebar,
}

/// Button styles: `.buttonStyle(.borderedProminent)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ButtonStyle {
    Automatic,
    Bordered,
    BorderedProminent,
    Borderless,
    Plain,
}

/// The role of a button: `Button("Delete", role: .destructive)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ButtonRole {
    Cancel,
    Destructive,
}

/// Text field styles: `.textFieldStyle(.roundedBorder)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TextFieldStyle {
    Automatic,
    Plain,
    RoundedBorder,
}

/// Picker styles: `.pickerStyle(.segmented)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum PickerStyle {
    Automatic,
    Menu,
    Segmented,
    Inline,
    Wheel,
}

/// Animations: `.animation(.spring, value: v)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Animation {
    Default,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Spring,
    Bouncy,
    Smooth,
    Snappy,
}

/// Transitions: `.transition(.slide)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Transition {
    Opacity,
    Slide,
    Scale,
}

/// How an image fills its frame: `.aspectRatio(contentMode: .fit)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ContentMode {
    Fit,
    Fill,
}

/// `Image.Scale`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ImageScale {
    Small,
    Medium,
    Large,
}

/// `ColorScheme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ColorScheme {
    Light,
    Dark,
}
