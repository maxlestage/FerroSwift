//! Markers for the FerroSwift transpiler. They generate no code: Rust only needs to know the
//! attributes exist, the transpiler reads them from the source.

use proc_macro::TokenStream;

/// A SwiftUI view: `#[derive(View)]` on a struct whose `impl View` describes its `body`.
///
/// Field attributes:
/// - `#[state]` / `#[state(expr)]`: `@State var field = expr` (the type's default without
///   an expression);
/// - `#[binding]`: `@Binding var field`, given by the parent as `&mut self.its_field`;
/// - `#[environment(key)]`: `@Environment(\.key) var field`.
#[proc_macro_derive(View, attributes(state, binding, environment))]
pub fn derive_view(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// An observable model: `@Observable final class`, shared by reference between views.
/// `#[observation_ignored]` on a field gives `@ObservationIgnored`.
#[proc_macro_derive(Observable, attributes(observation_ignored))]
pub fn derive_observable(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// A preview: `#[preview] fn preview() -> impl View { … }` becomes `#Preview { … }`.
#[proc_macro_attribute]
pub fn preview(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut out: TokenStream = "#[allow(dead_code)]".parse().expect("attribute");
    out.extend(item);
    out
}
