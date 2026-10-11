//! Translations of code that once panicked or produced Swift that does not compile.

use ferroswift_transpiler::transpile;

fn swift(source: &str) -> String {
    transpile(source).swift
}

#[test]
fn imported_float_constants() {
    let out =
        swift("use std::f64::consts::{self, PI};\nfn f(r: f64) -> f64 { PI * r + consts::E }");
    assert!(out.contains("Double.pi * r + Double(M_E)"), "{out}");
}

#[test]
fn ui_constructors_with_wrong_arguments_are_errors() {
    let out = transpile(
        "use ferroswift::prelude::*;\n#[derive(View)]\nstruct A;\nimpl View for A {\n    fn body(&mut self) -> impl View { text(\"x\").foreground(Color::rgb(1.0)) }\n}",
    );
    let errors: Vec<String> = out
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(ToString::to_string)
        .collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Color::rgb takes 3 arguments, not 1"),
        "{errors:?}"
    );
}

#[test]
fn shared_mutability_is_a_cell_class() {
    let out = transpile(
        "use std::{cell::RefCell, rc::Rc};\nfn main() {\n    let shared = Rc::new(RefCell::new(0));\n    let other = Rc::clone(&shared);\n    *shared.borrow_mut() += 1;\n    println!(\"{}\", other.borrow());\n}",
    );
    assert!(
        out.swift.contains("let shared = RsRefCell(0)"),
        "{}",
        out.swift
    );
    assert!(out.swift.contains("let other = shared"), "{}", out.swift);
    assert!(out.swift.contains("shared.value += 1"), "{}", out.swift);
    assert!(
        out.swift.contains("final class RsRefCell<T>"),
        "{}",
        out.swift
    );
    assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
}

#[test]
fn environment_fields_are_not_passed() {
    let out = swift(
        "use ferroswift::prelude::*;\n#[derive(View)]\nstruct Detail {\n    name: String,\n    #[environment(dismiss)]\n    dismiss: DismissAction,\n}\nimpl View for Detail {\n    fn body(&mut self) -> impl View { text(&self.name) }\n}\nfn make(name: String) -> Detail {\n    Detail { name, dismiss: DismissAction }\n}",
    );
    assert!(out.contains("Detail(name: name)"), "{out}");
}

#[test]
fn comments_between_children_are_kept() {
    let out = swift(
        "use ferroswift::prelude::*;\n#[derive(View, Default)]\nstruct V;\nimpl View for V {\n    fn body(&mut self) -> impl View {\n        vstack() // screen\n            .spacing(8.0)\n            // title\n            .child(text(\"a\"))\n            .child(text(\"b\")) // second\n            .padding()\n    }\n}",
    );
    assert!(
        out.contains(
            "VStack(spacing: 8) {\n            // screen\n            // title\n            Text(\"a\")\n            Text(\"b\") // second\n        }"
        ),
        "{out}"
    );
}

#[test]
fn blank_lines_and_modifier_comments_are_kept() {
    let out = swift(
        "use ferroswift::prelude::*;\n#[derive(View, Default)]\nstruct V;\nimpl View for V {\n    fn body(&mut self) -> impl View {\n        text(\"hi\")\n            // bigger\n            .font(Font::Title)\n            .padding() // around\n    }\n}\nfn compute(items: &[i32]) -> i32 {\n    let mut total = 0;\n\n    // Sum.\n    for x in items {\n        total += x;\n    }\n\n\n    total\n}",
    );
    assert!(
        out.contains("Text(\"hi\")\n            // bigger\n            .font(.title)\n            .padding() // around"),
        "{out}"
    );
    assert!(
        out.contains("var total = 0\n\n    // Sum.\n    for x in items {\n        total += x\n    }\n\n    return total"),
        "{out}"
    );
}

#[test]
fn views_are_recognized_without_the_prelude_import() {
    let out = swift(
        "#[derive(View, Default)]\nstruct V;\nimpl View for V {\n    fn body(&mut self) -> impl View { vstack().child(text(\"a\")) }\n}",
    );
    assert!(
        out.contains("VStack {\n            Text(\"a\")\n        }"),
        "{out}"
    );
}

#[test]
fn a_none_arm_before_some_matches_nothing_else() {
    let out = swift(
        "fn f(x: Option<i32>) -> i32 {\n    match x {\n        None => 0,\n        Some(n) => n,\n    }\n}",
    );
    assert!(
        out.contains("case .none:\n        return 0\n    case .some(let n):"),
        "{out}"
    );
}

#[test]
fn recursive_structs_become_classes() {
    let out = swift(
        "#[derive(Debug, Clone, PartialEq)]\nstruct Node {\n    value: i32,\n    next: Option<Box<Node>>,\n}\nfn main() {\n    let a = Node { value: 1, next: None };\n    let b = a.clone();\n    println!(\"{}\", a == b);\n}",
    );
    assert!(out.contains("final class Node: Equatable {"), "{out}");
    assert!(out.contains("init(value: Int, next: Node? = nil)"), "{out}");
    assert!(
        out.contains("func clone() -> Node {\n        Node(value: value, next: next?.clone())"),
        "{out}"
    );
    assert!(out.contains("let b = a.clone()"), "{out}");
}
