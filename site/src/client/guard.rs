//! Calls the transpiler so that a panic (a bug of the transpiler on some input) does not
//! stop the playground.
//!
//! WebAssembly built with `panic = "abort"` traps on a panic, and the trap unwinds to the
//! nearest JavaScript frame. The transpiler is therefore called through a tiny JavaScript
//! function with a `try`/`catch`: the trap stops at that function, the Rust code around it
//! (the event handler, the island's states) carries on, and the panic hook has recorded the
//! message to show.

use std::cell::RefCell;

use ferroswift_transpiler::Output;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function guarded(f, s) { try { f(s); return true; } catch (e) { return false; } }"
)]
extern "C" {
    fn guarded(f: &JsValue, source: &str) -> bool;
}

thread_local! {
    /// The output of the last call, written by the closure called from JavaScript.
    static OUTPUT: RefCell<Option<Output>> = const { RefCell::new(None) };
    /// The message of the last panic.
    static PANIC: RefCell<Option<String>> = const { RefCell::new(None) };
    static RUN: Closure<dyn Fn(String)> = Closure::new(|source: String| {
        let out = ferroswift_transpiler::transpile(&source);
        OUTPUT.with(|o| *o.borrow_mut() = Some(out));
    });
}

/// Records panic messages (and logs them in the console).
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let message = info.to_string();
        web_sys::console::error_1(&JsValue::from_str(&message));
        PANIC.with(|p| *p.borrow_mut() = Some(message));
    }));
}

/// `ferroswift_transpiler::transpile(source)`, or the panic message.
pub fn transpile(source: &str) -> Result<Output, String> {
    OUTPUT.with(|o| o.borrow_mut().take());
    PANIC.with(|p| p.borrow_mut().take());
    let finished = RUN.with(|run| guarded(run.as_ref(), source));
    match OUTPUT.with(|o| o.borrow_mut().take()) {
        Some(out) if finished => Ok(out),
        _ => Err(PANIC
            .with(|p| p.borrow_mut().take())
            .unwrap_or_else(|| "the transpiler stopped".into())),
    }
}
