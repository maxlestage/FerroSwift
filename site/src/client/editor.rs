//! The code editor's keyboard: Tab indents with four spaces (Shift+Tab removes them, on
//! every selected line), Enter keeps the indentation (and opens a block after `{`), a
//! closing bracket on an empty line goes back one level, Escape then Tab leaves the editor.
//! Text is inserted with `insertText`, so undo keeps working.

use active::{Event, State};
use wasm_bindgen::JsCast;
use web_sys::{HtmlDocument, HtmlElement, HtmlSelectElement, HtmlTextAreaElement, KeyboardEvent};

const INDENT: &str = "    ";

fn area_of(dom: &web_sys::Element) -> Option<&HtmlTextAreaElement> {
    dom.dyn_ref::<HtmlTextAreaElement>()
}

/// The text of the editor element `dom`.
pub fn value(dom: &web_sys::Element) -> Option<String> {
    area_of(dom).map(HtmlTextAreaElement::value)
}

fn selection(area: &HtmlTextAreaElement) -> (u32, u32) {
    let start = area.selection_start().ok().flatten().unwrap_or(0);
    let end = area.selection_end().ok().flatten().unwrap_or(start);
    (start.min(end), start.max(end))
}

fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

/// Replaces the selection with `text` (an undoable edit that fires `input`).
fn insert(area: &HtmlTextAreaElement, text: &str) {
    let done = super::window()
        .document()
        .and_then(|d| d.dyn_into::<HtmlDocument>().ok())
        .and_then(|d| {
            d.exec_command_with_show_ui_and_value("insertText", false, text)
                .ok()
        })
        .unwrap_or(false);
    if !done {
        let (start, end) = selection(area);
        let _ = area.set_range_text_with_start_and_end(text, start, end);
        let caret = start + utf16_len(text);
        let _ = area.set_selection_range(caret, caret);
        if let Ok(event) = web_sys::Event::new("input") {
            let _ = area.dispatch_event(&event);
        }
    }
}

/// The start of the line holding UTF-16 offset `at`, and the offset of the end of the line
/// holding `to`.
fn line_bounds(text: &[u16], at: u32, to: u32) -> (u32, u32) {
    let at = (at as usize).min(text.len());
    let to = (to as usize).min(text.len());
    let start = text[..at]
        .iter()
        .rposition(|&c| c == u16::from(b'\n'))
        .map_or(0, |i| i + 1);
    let end = text[to..]
        .iter()
        .position(|&c| c == u16::from(b'\n'))
        .map_or(text.len(), |i| to + i);
    (start as u32, end as u32)
}

/// Tab and Shift+Tab on several lines: indents or dedents each of them.
fn indent_lines(area: &HtmlTextAreaElement, dedent: bool) {
    let text: Vec<u16> = area.value().encode_utf16().collect();
    let (start, end) = selection(area);
    // A selection ending at the start of a line does not include that line.
    let end = if end > start && text.get(end as usize - 1) == Some(&u16::from(b'\n')) {
        end - 1
    } else {
        end
    };
    let (first, last) = line_bounds(&text, start, end);
    let block = String::from_utf16_lossy(&text[first as usize..last as usize]);
    let changed: Vec<String> = block
        .split('\n')
        .map(|line| {
            if dedent {
                let spaces = line.len() - line.trim_start_matches(' ').len();
                line[spaces.min(INDENT.len())..].to_string()
            } else if line.is_empty() {
                String::new()
            } else {
                format!("{INDENT}{line}")
            }
        })
        .collect();
    let changed = changed.join("\n");
    if changed == block {
        return;
    }
    let _ = area.set_selection_range(first, last);
    insert(area, &changed);
    let _ = area.set_selection_range(first, first + utf16_len(&changed));
}

/// Enter: a new line with the indentation of the current one, one level deeper after an
/// opening bracket; between brackets, the closing one goes on its own line.
fn new_line(area: &HtmlTextAreaElement) {
    let text: Vec<u16> = area.value().encode_utf16().collect();
    let (start, end) = selection(area);
    let (line_start, _) = line_bounds(&text, start, start);
    let before = String::from_utf16_lossy(&text[line_start as usize..start as usize]);
    let indent: String = before.chars().take_while(|c| *c == ' ').collect();
    let opens = before.trim_end().ends_with(['{', '(', '[']);
    let closes = text
        .get(end as usize)
        .is_some_and(|&c| c == u16::from(b'}') || c == u16::from(b')') || c == u16::from(b']'));
    if opens && closes {
        let inner = format!("\n{indent}{INDENT}");
        insert(area, &format!("{inner}\n{indent}"));
        let caret = start + utf16_len(&inner);
        let _ = area.set_selection_range(caret, caret);
    } else if opens {
        insert(area, &format!("\n{indent}{INDENT}"));
    } else {
        insert(area, &format!("\n{indent}"));
    }
}

/// A closing bracket typed on a line holding only spaces: one level less indentation.
fn close_block(area: &HtmlTextAreaElement, closer: &str) -> bool {
    let text: Vec<u16> = area.value().encode_utf16().collect();
    let (start, end) = selection(area);
    let (line_start, _) = line_bounds(&text, start, start);
    let before = &text[line_start as usize..start as usize];
    let blank = before.iter().all(|&c| c == u16::from(b' '));
    if start != end || !blank || before.len() < INDENT.len() {
        return false;
    }
    let _ = area.set_selection_range(start - INDENT.len() as u32, start);
    insert(area, closer);
    true
}

/// The editor's `keydown` handler.
pub fn keydown(e: &Event, escape: State<bool>) {
    let Some(key) = e.raw().dyn_ref::<KeyboardEvent>() else {
        return;
    };
    let Some(target) = e.current_target() else {
        return;
    };
    let Some(area) = area_of(&target) else {
        return;
    };
    let name = key.key();
    let leaving = escape.get();
    if name != "Escape" && leaving {
        escape.set(false);
    }
    if key.ctrl_key() || key.meta_key() || key.alt_key() || key.is_composing() {
        return;
    }
    match name.as_str() {
        "Escape" => escape.set(true),
        // After Escape, Tab moves the focus as usual.
        "Tab" if leaving => {}
        "Tab" => {
            key.prevent_default();
            let (start, end) = selection(area);
            let multi_line = area
                .value()
                .encode_utf16()
                .skip(start as usize)
                .take((end - start) as usize)
                .any(|c| c == u16::from(b'\n'));
            if key.shift_key() || multi_line {
                indent_lines(area, key.shift_key());
            } else {
                insert(area, INDENT);
            }
        }
        "Enter" if !key.shift_key() => {
            key.prevent_default();
            new_line(area);
        }
        "}" | ")" | "]" if close_block(area, &name) => key.prevent_default(),
        _ => {}
    }
}

/// Shows example `code` in the editor: the text, the picker, the scroll position and, on
/// the playground page, the URL.
pub fn show(
    editor: Option<&web_sys::Element>,
    scroller: Option<&web_sys::Element>,
    picker: Option<&web_sys::Element>,
    code: &str,
    id: &str,
    page: bool,
) {
    if let Some(area) = editor.and_then(area_of) {
        area.set_value(code);
        let _ = area.set_selection_range(0, 0);
    }
    if let Some(scroll) = scroller {
        scroll.set_scroll_top(0);
        scroll.set_scroll_left(0);
    }
    if let Some(select) = picker.and_then(|p| p.dyn_ref::<HtmlSelectElement>()) {
        select.set_value(id);
    }
    if page {
        super::replace_query(&format!("?example={id}"));
    }
}

/// Selects line `line` from `column` in the editor and scrolls it into view.
pub fn select(
    editor: &web_sys::Element,
    scroller: &web_sys::Element,
    source: &str,
    line: usize,
    column: usize,
) {
    let Some(area) = area_of(editor) else {
        return;
    };
    let mut offset = 0;
    let mut range = (0, 0);
    let mut lines = 0;
    for (n, text) in source.split('\n').enumerate() {
        lines = n + 1;
        if n + 1 == line {
            let skipped: String = text.chars().take(column.saturating_sub(1)).collect();
            let from = offset + utf16_len(&skipped);
            let to = offset + utf16_len(text);
            range = (from.min(to), to);
        }
        offset += utf16_len(text) + 1;
    }
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    let _ = area.focus_with_options(&options);
    let _ = area.set_selection_range(range.0, range.1);
    // Scroll so that the line is in the upper third of the editor.
    let gutter = scroller
        .query_selector(".pg-gutter")
        .ok()
        .flatten()
        .and_then(|g| g.dyn_into::<HtmlElement>().ok());
    if let Some(gutter) = gutter {
        let line_height = f64::from(gutter.offset_height()) / lines.max(1) as f64;
        let top = (line.saturating_sub(1)) as f64 * line_height
            - f64::from(scroller.client_height()) / 3.0;
        scroller.set_scroll_top(top.max(0.0) as i32);
    }
    // Selecting may scroll sideways: the start of the line stays visible.
    scroller.set_scroll_left(0);
}
