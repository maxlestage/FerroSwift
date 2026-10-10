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
fn shared_mutability_is_a_var_and_a_warning() {
    let out = transpile(
        "use std::{cell::RefCell, rc::Rc};\nfn main() {\n    let shared = Rc::new(RefCell::new(0));\n    *shared.borrow_mut() += 1;\n}",
    );
    assert!(out.swift.contains("var shared = 0"), "{}", out.swift);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| !d.is_error() && d.message.contains("RefCell")),
        "{:?}",
        out.diagnostics
    );
}

#[test]
fn environment_fields_are_not_passed() {
    let out = swift(
        "use ferroswift::prelude::*;\n#[derive(View)]\nstruct Detail {\n    name: String,\n    #[environment(dismiss)]\n    dismiss: DismissAction,\n}\nimpl View for Detail {\n    fn body(&mut self) -> impl View { text(&self.name) }\n}\nfn make(name: String) -> Detail {\n    Detail { name, dismiss: DismissAction }\n}",
    );
    assert!(out.contains("Detail(name: name)"), "{out}");
}
