//! The browser bundle: hydrates the playground island. Everything else on the site is
//! plain HTML and CSS.

pub mod editor;
pub mod guard;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    guard::install_panic_hook();
    // Links of a site published under a sub-path (GitHub Pages: `/FerroSwift`).
    if let Some(base) = window()
        .document()
        .and_then(|d| d.document_element())
        .and_then(|html| html.get_attribute("data-base"))
    {
        crate::routes::set_base(&base);
    }
    active::hydrate(&crate::islands::ISLANDS);
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

/// Milliseconds since the page loaded.
pub fn now() -> Option<f64> {
    window().performance().map(|p| p.now())
}

/// Copies `text` to the clipboard (the promise is left alone: a refusal changes nothing).
pub fn copy_text(text: &str) {
    let _ = window().navigator().clipboard().write_text(text);
}

/// Runs `f` once, in `ms` milliseconds.
pub fn after(ms: i32, f: impl FnOnce() + 'static) {
    let callback = Closure::once_into_js(f);
    let _ = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), ms);
}

/// A parameter of the page's query string (`?example=todo`), not decoded: the playground's
/// parameters are plain words.
pub fn query_param(name: &str) -> Option<String> {
    let search = window().location().search().ok()?;
    search.trim_start_matches('?').split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

/// Replaces the query string of the URL, without a new history entry.
pub fn replace_query(query: &str) {
    if let Ok(history) = window().history() {
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(query));
    }
}
