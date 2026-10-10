//! Views written with the `ferroswift` crate → SwiftUI.
//!
//! `vstack().spacing(8.0).child(text("Hi")).padding()` becomes
//!
//! ```swift
//! VStack(spacing: 8) {
//!     Text("Hi")
//! }
//! .padding()
//! ```

use syn::spanned::Spanned;
use syn::{Expr, ExprCall, ExprMethodCall};

use crate::expr::{Ex, swift_string};
use crate::names;
use crate::stmt::Sink;
use crate::symbols::TypeKind;
use crate::translate::{FnCx, Translator, indent_lines};
use crate::ty::Ty;

/// The view functions of the ferroswift crate.
const VIEW_FNS: &[&str] = &[
    "text",
    "button",
    "button_role",
    "button_label",
    "vstack",
    "hstack",
    "zstack",
    "lazy_vstack",
    "lazy_hstack",
    "scroll_view",
    "list",
    "form",
    "section",
    "group",
    "navigation_stack",
    "tab_view",
    "disclosure_group",
    "for_each",
    "navigation_link",
    "navigation_link_label",
    "spacer",
    "divider",
    "empty_view",
    "image",
    "system_image",
    "label",
    "async_image",
    "link",
    "toggle",
    "text_field",
    "secure_field",
    "text_editor",
    "slider",
    "stepper",
    "picker",
    "progress_view",
    "progress",
    "rectangle",
    "circle",
    "capsule",
    "ellipse",
    "rounded_rectangle",
    "content_unavailable",
];

/// Containers whose children go in a trailing `{ … }`.
const CONTAINERS: &[&str] = &[
    "vstack",
    "hstack",
    "zstack",
    "lazy_vstack",
    "lazy_hstack",
    "scroll_view",
    "list",
    "form",
    "section",
    "group",
    "navigation_stack",
    "tab_view",
    "disclosure_group",
    "picker",
];

/// Style enums of the ferroswift crate, written as SwiftUI members.
const STYLE_ENUMS: &[&str] = &[
    "Color",
    "Font",
    "Weight",
    "Design",
    "Alignment",
    "Edge",
    "Axis",
    "TextAlignment",
    "ListStyle",
    "ButtonStyle",
    "ButtonRole",
    "TextFieldStyle",
    "PickerStyle",
    "Animation",
    "Transition",
    "ContentMode",
    "ImageScale",
    "ColorScheme",
];

/// Modifiers that set the frame: merged into one `.frame(…)` when consecutive.
const FRAME: &[&str] = &[
    "frame",
    "frame_width",
    "frame_height",
    "frame_min_width",
    "frame_min_height",
    "frame_max_width",
    "frame_max_height",
    "fill_width",
    "fill_height",
    "frame_alignment",
];

impl Translator<'_> {
    fn ui(&self) -> bool {
        self.syms.ui
    }

    fn is_view_fn(&self, name: &str) -> bool {
        self.ui()
            && VIEW_FNS.contains(&name)
            && !self.syms.fns.contains_key(name)
            && self.lookup(name).is_none()
    }

    /// `Color::Red` → `.red` (in arguments) or `Color.red`.
    pub fn ui_constant(&mut self, owner: &str, last: &str, expected: &Ty) -> Option<Ex> {
        if !self.ui() || !STYLE_ENUMS.contains(&owner) || self.syms.is_user_type(owner) {
            return None;
        }
        self.used_ui = true;
        let member = match (owner, last) {
            ("Color", "Accent") => "accentColor".to_string(),
            _ => names::case_name(last),
        };
        let ty = Ty::Named(owner.to_string(), vec![]);
        // `Color.accentColor` has no `ShapeStyle` shorthand: always written in full.
        let explicit = matches!(owner, "Color" | "Font")
            && !self.implicit_member
            && expected.named() != Some(owner)
            || member == "accentColor";
        if explicit {
            Some(Ex::primary(format!("{owner}.{member}"), ty))
        } else {
            Some(Ex::primary(format!(".{member}"), ty))
        }
    }

    /// A style enum case in a pattern: `Color::Red =>` → `case .red:`.
    pub fn ui_case(&mut self, owner: &str, last: &str) -> Option<String> {
        if !self.ui() || !STYLE_ENUMS.contains(&owner) || self.syms.is_user_type(owner) {
            return None;
        }
        Some(format!(".{}", names::case_name(last)))
    }

    /// A ferroswift function called on its own (no modifier): `spacer()`, `text("Hi")`,
    /// `Color::rgb(…)`, `window_group(…)`, `with_animation(…)`.
    pub fn ui_call(
        &mut self,
        name: &str,
        args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        _expected: &Ty,
    ) -> Option<Ex> {
        if !self.ui() {
            return None;
        }
        let args: Vec<Expr> = args.iter().cloned().collect();
        match name {
            "window_group" if args.len() == 1 => {
                self.used_ui = true;
                let lines = self.view_content(&args[0]);
                return Some(Ex::primary(
                    format!("WindowGroup {{\n{}\n}}", indent_lines(&lines.join("\n"))),
                    Ty::View,
                ));
            }
            "with_animation" if args.len() == 1 => {
                let f = self.action(&args[0]);
                return Some(Ex::primary(format!("withAnimation {f}"), Ty::Unit));
            }
            "with_animation_of" if args.len() == 2 => {
                let a = self.ui_arg(&args[0]);
                let f = self.action(&args[1]);
                return Some(Ex::primary(format!("withAnimation({a}) {f}"), Ty::Unit));
            }
            "rgb" | "rgba" | "hex" | "system" | "system_weight" | "system_design" => {
                // Reached through `Color::rgb` / `Font::system`: see ui_static.
            }
            _ => {}
        }
        if self.is_view_fn(name) {
            self.used_ui = true;
            return Some(Ex::primary(self.view_root(name, &args, &[]), Ty::View));
        }
        None
    }

    /// `Color::rgb(r, g, b)`, `Font::system(20.0)`…
    fn ui_static(&mut self, owner: &str, name: &str, args: &[Expr]) -> Option<String> {
        if !self.ui() || self.syms.is_user_type(owner) {
            return None;
        }
        let arity = match (owner, name) {
            ("Color", "rgb") => 3,
            ("Color", "rgba") => 4,
            ("Color", "hex") | ("Font", "system") => 1,
            ("Font", "system_weight" | "system_design") => 2,
            _ => return None,
        };
        if args.len() != arity {
            // rustc rejects the call too; the rest of the file is still translated.
            let span = args
                .first()
                .map_or_else(proc_macro2::Span::call_site, syn::spanned::Spanned::span);
            self.error(
                span,
                format!(
                    "{owner}::{name} takes {arity} argument{}, not {}",
                    if arity == 1 { "" } else { "s" },
                    args.len()
                ),
            );
            return Some(format!(
                "fatalError(\"FerroSwift: wrong arguments for {owner}::{name}\")"
            ));
        }
        let a: Vec<String> = args.iter().map(|e| self.ui_arg(e)).collect();
        let implicit = self.implicit_member;
        let prefix = if implicit { "" } else { owner };
        Some(match (owner, name) {
            ("Color", "rgb") => format!("Color(red: {}, green: {}, blue: {})", a[0], a[1], a[2]),
            ("Color", "rgba") => format!(
                "Color(red: {}, green: {}, blue: {}, opacity: {})",
                a[0], a[1], a[2], a[3]
            ),
            ("Color", "hex") => {
                self.helper("color_hex");
                format!("Color(hex: {})", a[0])
            }
            ("Font", "system") => format!("{prefix}.system(size: {})", a[0]),
            ("Font", "system_weight") => {
                format!("{prefix}.system(size: {}, weight: {})", a[0], a[1])
            }
            ("Font", "system_design") => {
                format!("{prefix}.system(size: {}, design: {})", a[0], a[1])
            }
            _ => return None,
        })
    }

    /// A SwiftUI argument: style enums as members, `16.0` as `16`.
    fn ui_arg(&mut self, e: &Expr) -> String {
        let saved = self.implicit_member;
        self.implicit_member = true;
        let v = match e {
            Expr::Call(c) => match self.static_call(c) {
                Some(code) => Ex::primary(code, Ty::Unknown),
                None => self.expr(e),
            },
            _ => self.expr(e),
        };
        let v = self.settle(v);
        self.implicit_member = saved;
        v.code
    }

    fn static_call(&mut self, c: &ExprCall) -> Option<String> {
        let Expr::Path(p) = &*c.func else { return None };
        if p.path.segments.len() != 2 {
            return None;
        }
        let owner = p.path.segments[0].ident.to_string();
        let name = p.path.segments[1].ident.to_string();
        let args: Vec<Expr> = c.args.iter().cloned().collect();
        self.ui_static(&owner, &name, &args)
    }

    /// The text of a `Text` or a button: a literal stays localizable, a formatted string
    /// whose values are not all strings or numbers is shown verbatim.
    fn ui_text(&mut self, e: &Expr) -> String {
        if let Expr::Macro(m) = e {
            if m.mac.path.is_ident("format") {
                self.format_types.clear();
                let v = self.expr(e);
                let safe = self
                    .format_types
                    .iter()
                    .all(|t| matches!(t, Ty::Str | Ty::Int(_) | Ty::Float(_)));
                if v.code.starts_with('"') && !safe {
                    return format!("verbatim: {}", v.code);
                }
                return v.code;
            }
        }
        let v = self.expr_with(e, &Ty::Str);
        let v = self.settle(v);
        if v.ty == Ty::Str || v.code.starts_with('"') {
            v.code
        } else {
            format!("String(describing: {})", v.code)
        }
    }

    /// An action closure (or function) for a trailing closure: `{ count += 1 }`.
    fn action(&mut self, e: &Expr) -> String {
        let v = self.expr_with(e, &Ty::Fn(vec![], Box::new(Ty::Unit)));
        let v = self.settle(v);
        if v.code.starts_with('{') {
            v.code
        } else {
            format!("{{ {}() }}", v.code)
        }
    }

    /// The content of a closure taking views (`.overlay { … }`, `NavigationLink { … }`).
    fn view_content(&mut self, e: &Expr) -> Vec<String> {
        self.view_closure_depth += 1;
        let saved = self.value_yield;
        self.value_yield = 0;
        let lines = self.view_lines(e);
        self.value_yield = saved;
        self.view_closure_depth -= 1;
        lines
    }

    /// An expression in a result builder: one or more views.
    pub fn view_lines(&mut self, e: &Expr) -> Vec<String> {
        match e {
            Expr::If(_) | Expr::Match(_) => {
                self.view_closure_depth += 1;
                let saved = self.value_yield;
                self.value_yield = 0;
                let lines = self.tail(e, &Sink::Yield);
                self.value_yield = saved;
                self.view_closure_depth -= 1;
                lines
            }
            Expr::Block(b) => {
                self.view_closure_depth += 1;
                let lines = self.block_lines(&b.block, Sink::Yield);
                self.view_closure_depth -= 1;
                lines
            }
            Expr::Paren(p) => self.view_lines(&p.expr),
            Expr::Reference(r) => self.view_lines(&r.expr),
            Expr::Array(a) => a.elems.iter().flat_map(|e| self.view_lines(e)).collect(),
            Expr::Macro(m) if m.mac.path.is_ident("vec") => {
                let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
                match syn::parse::Parser::parse2(parser, m.mac.tokens.clone()) {
                    Ok(items) => items.iter().flat_map(|e| self.view_lines(e)).collect(),
                    Err(_) => vec![self.unsupported(m.span(), "this vec! of views")],
                }
            }
            Expr::Path(p) if p.path.is_ident("None") => vec![],
            Expr::Call(c)
                if matches!(&*c.func, Expr::Path(p) if p.path.is_ident("Some"))
                    && c.args.len() == 1 =>
            {
                self.view_lines(&c.args[0])
            }
            Expr::MethodCall(m) if m.method == "then" && m.args.len() == 1 => {
                // `cond.then(|| view)` → `if cond { view }`.
                let cond = self.expr_with(&m.receiver, &Ty::Bool);
                let cond = self.settle(cond);
                let body = match &m.args[0] {
                    Expr::Closure(c) => self.view_content(&c.body),
                    other => self.view_lines(other),
                };
                vec![format!(
                    "if {} {{\n{}\n}}",
                    cond.code,
                    indent_lines(&body.join("\n"))
                )]
            }
            Expr::MethodCall(m)
                if m.method == "map" && m.args.len() == 1 && self.is_option_expr(&m.receiver) =>
            {
                // `opt.map(|x| view)` → `if let x = opt { view }`.
                if let Expr::Closure(c) = &m.args[0] {
                    let v = self.expr(&m.receiver);
                    let v = self.settle(v);
                    self.push_scope();
                    let name = match c.inputs.first() {
                        Some(syn::Pat::Ident(id)) => {
                            self.declare(&id.ident.to_string(), v.ty.item())
                        }
                        _ => "_".into(),
                    };
                    let body = self.view_content(&c.body);
                    self.pop_scope();
                    return vec![format!(
                        "if let {name} = {} {{\n{}\n}}",
                        v.code,
                        indent_lines(&body.join("\n"))
                    )];
                }
                let v = self.expr(e);
                vec![v.code]
            }
            _ => {
                let v = self.expr(e);
                let v = self.settle(v);
                if v.code.is_empty() {
                    vec![]
                } else {
                    vec![v.code]
                }
            }
        }
    }

    fn is_option_expr(&mut self, e: &Expr) -> bool {
        let saved = self.diags.len();
        let preludes = self.preludes.len();
        self.preludes.push(Vec::new());
        let v = self.expr(e);
        self.preludes.truncate(preludes);
        self.diags.truncate(saved);
        v.ty.is_option()
    }

    /// A method chain whose root is a view: `vstack().child(…).padding()`.
    pub fn ui_method_chain(&mut self, m: &ExprMethodCall, _expected: &Ty) -> Option<Ex> {
        if !self.ui() {
            return None;
        }
        let mut methods: Vec<&ExprMethodCall> = vec![m];
        let mut root: &Expr = &m.receiver;
        while let Expr::MethodCall(inner) = root {
            methods.push(inner);
            root = &inner.receiver;
        }
        methods.reverse();
        let root_code = match root {
            Expr::Call(c) => {
                let Expr::Path(p) = &*c.func else { return None };
                let segs: Vec<String> = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                let args: Vec<Expr> = c.args.iter().cloned().collect();
                if segs.len() == 1 && self.is_view_fn(&segs[0]) {
                    self.used_ui = true;
                    let name = segs[0].clone();
                    return Some(Ex::primary(
                        self.view_root(&name, &args, &methods),
                        Ty::View,
                    ));
                }
                if segs.len() == 2 && STYLE_ENUMS.contains(&segs[0].as_str()) {
                    self.ui_static(&segs[0], &segs[1], &args)?
                } else {
                    return self.view_modified(root, &methods);
                }
            }
            Expr::Path(p) if p.path.segments.len() == 2 && p.path.segments[0].ident == "Color" => {
                let v = self.expr(root);
                v.code
            }
            Expr::Struct(s) => {
                let name = s.path.segments.last()?.ident.to_string();
                let is_view = self
                    .syms
                    .structs
                    .get(&self.rust_type_name(&name))
                    .is_some_and(|st| st.kind == TypeKind::View);
                if !is_view {
                    return None;
                }
                return self.view_modified(root, &methods);
            }
            _ => return self.view_modified(root, &methods),
        };
        // `Color::rgb(…).opacity(0.5)`.
        let mut code = root_code;
        for mc in &methods {
            let args: Vec<String> = mc.args.iter().map(|a| self.ui_arg(a)).collect();
            code = format!(
                "{code}.{}({})",
                names::lower_camel(&mc.method.to_string()),
                args.join(", ")
            );
        }
        Some(Ex::primary(code, Ty::Named("Color".into(), vec![])))
    }

    /// Modifiers on a view that is not built by a ferroswift function (a struct of the crate,
    /// a variable): only when the receiver is a view.
    fn view_modified(&mut self, root: &Expr, methods: &[&ExprMethodCall]) -> Option<Ex> {
        let first = methods.first()?.method.to_string();
        if !is_modifier(&first) {
            return None;
        }
        let saved_diags = self.diags.len();
        let preludes = self.preludes.len();
        self.preludes.push(Vec::new());
        let probe = self.expr(root);
        self.preludes.truncate(preludes);
        self.diags.truncate(saved_diags);
        let is_view = probe.ty == Ty::View
            || matches!(&probe.ty, Ty::Named(n, _) if self.struct_by_swift(n).is_some_and(|s| s.kind == TypeKind::View));
        if !is_view {
            return None;
        }
        let v = self.expr(root);
        let v = self.settle(v);
        let modifiers = self.modifiers(methods);
        Some(Ex::primary(attach(v.code, &modifiers), Ty::View))
    }

    /// The SwiftUI code of a ferroswift view function with its methods.
    fn view_root(&mut self, name: &str, args: &[Expr], methods: &[&ExprMethodCall]) -> String {
        // Configuration (init arguments) and content.
        let mut spacing = None;
        let mut alignment = None;
        let mut axes = None;
        let mut header = None;
        let mut step = None;
        let mut children: Vec<String> = Vec::new();
        let mut rest: Vec<&ExprMethodCall> = Vec::new();
        let has_children = methods
            .iter()
            .any(|mc| mc.method == "child" || mc.method == "children");
        if let (true, Some(first)) = (has_children, methods.first()) {
            // `vstack() // note`: at the top of the content.
            children.extend(
                self.comments
                    .take_trailing(first.receiver.span().end().line),
            );
        }
        for (i, mc) in methods.iter().enumerate() {
            let m = mc.method.to_string();
            if m == "child" || m == "children" {
                // The comments written before `.child(…)` go before the child.
                let prev = match i {
                    0 => mc.receiver.span().end().line,
                    _ => methods[i - 1].span().end().line,
                };
                let dot = mc.dot_token.span().start().line;
                children.extend(self.comments.take_trailing(prev));
                children.extend(self.comments.take_between(prev, dot));
            }
            match m.as_str() {
                "spacing" => spacing = Some(self.ui_arg(&mc.args[0])),
                "alignment" if CONTAINERS.contains(&name) => {
                    alignment = Some(self.ui_arg(&mc.args[0]))
                }
                "axes" => axes = Some(self.ui_arg(&mc.args[0])),
                "header" => header = Some(self.ui_text(&mc.args[0])),
                "step" => step = Some(self.ui_arg(&mc.args[0])),
                "child" | "children" => {
                    let mut lines = if m == "child" {
                        self.view_content(&mc.args[0])
                    } else {
                        self.view_children(&mc.args[0])
                    };
                    if let (Some(c), Some(last)) = (
                        self.comments.take_trailing(mc.span().end().line),
                        lines.last_mut(),
                    ) {
                        last.push_str(&format!(" {c}"));
                    }
                    children.extend(lines);
                }
                _ => rest.push(mc),
            }
        }
        let mut init: Vec<String> = Vec::new();
        if let Some(a) = alignment {
            init.push(format!("alignment: {a}"));
        }
        if let Some(s) = spacing {
            init.push(format!("spacing: {s}"));
        }
        let content = |children: &[String]| -> String {
            if children.is_empty() {
                " {}".to_string()
            } else {
                format!(" {{\n{}\n}}", indent_lines(&children.join("\n")))
            }
        };
        let base = match name {
            "vstack" | "hstack" | "zstack" | "lazy_vstack" | "lazy_hstack" => {
                let swift = match name {
                    "vstack" => "VStack",
                    "hstack" => "HStack",
                    "zstack" => "ZStack",
                    "lazy_vstack" => "LazyVStack",
                    _ => "LazyHStack",
                };
                if name == "zstack" {
                    init.retain(|a| a.starts_with("alignment"));
                }
                let args = if init.is_empty() {
                    String::new()
                } else {
                    format!("({})", init.join(", "))
                };
                format!("{swift}{args}{}", content(&children))
            }
            "scroll_view" => {
                let args = axes.map(|a| format!("({a})")).unwrap_or_default();
                format!("ScrollView{args}{}", content(&children))
            }
            "list" | "form" | "group" | "navigation_stack" | "tab_view" => {
                let swift = match name {
                    "list" => "List",
                    "form" => "Form",
                    "group" => "Group",
                    "navigation_stack" => "NavigationStack",
                    _ => "TabView",
                };
                format!("{swift}{}", content(&children))
            }
            "section" => match header {
                Some(h) => format!("Section({h}){}", content(&children)),
                None => format!("Section{}", content(&children)),
            },
            "disclosure_group" => format!(
                "DisclosureGroup({}){}",
                self.ui_text(&args[0]),
                content(&children)
            ),
            "picker" => {
                let label = self.ui_text(&args[0]);
                let sel = self.binding_arg(&args[1]);
                format!("Picker({label}, selection: {sel}){}", content(&children))
            }
            "text" => format!("Text({})", self.ui_text(&args[0])),
            "button" => {
                let label = self.ui_text(&args[0]);
                let action = self.action(&args[1]);
                format!("Button({label}) {action}")
            }
            "button_role" => {
                let label = self.ui_text(&args[0]);
                let role = self.ui_arg(&args[1]);
                let action = self.action(&args[2]);
                format!("Button({label}, role: {role}) {action}")
            }
            "button_label" => {
                let action = self.action(&args[0]);
                let label = self.view_content(&args[1]);
                format!(
                    "Button {action} label: {{\n{}\n}}",
                    indent_lines(&label.join("\n"))
                )
            }
            "for_each" => self.for_each(&args[0], &args[1]),
            "navigation_link" => {
                let label = self.ui_text(&args[0]);
                let dest = self.view_content(&args[1]);
                format!(
                    "NavigationLink({label}) {{\n{}\n}}",
                    indent_lines(&dest.join("\n"))
                )
            }
            "navigation_link_label" => {
                let dest = self.view_content(&args[0]);
                let label = self.view_content(&args[1]);
                format!(
                    "NavigationLink {{\n{}\n}} label: {{\n{}\n}}",
                    indent_lines(&dest.join("\n")),
                    indent_lines(&label.join("\n"))
                )
            }
            "spacer" => "Spacer()".into(),
            "divider" => "Divider()".into(),
            "empty_view" => "EmptyView()".into(),
            "rectangle" => "Rectangle()".into(),
            "circle" => "Circle()".into(),
            "capsule" => "Capsule()".into(),
            "ellipse" => "Ellipse()".into(),
            "progress_view" => "ProgressView()".into(),
            "rounded_rectangle" => {
                format!("RoundedRectangle(cornerRadius: {})", self.ui_arg(&args[0]))
            }
            "image" => format!("Image({})", self.ui_text(&args[0])),
            "system_image" => format!("Image(systemName: {})", self.ui_text(&args[0])),
            "label" => format!(
                "Label({}, systemImage: {})",
                self.ui_text(&args[0]),
                self.ui_text(&args[1])
            ),
            "async_image" => format!("AsyncImage(url: URL(string: {}))", self.ui_text(&args[0])),
            "link" => format!(
                "Link({}, destination: URL(string: {})!)",
                self.ui_text(&args[0]),
                self.ui_text(&args[1])
            ),
            "toggle" => format!(
                "Toggle({}, isOn: {})",
                self.ui_text(&args[0]),
                self.binding_arg(&args[1])
            ),
            "text_field" => format!(
                "TextField({}, text: {})",
                self.ui_text(&args[0]),
                self.binding_arg(&args[1])
            ),
            "secure_field" => format!(
                "SecureField({}, text: {})",
                self.ui_text(&args[0]),
                self.binding_arg(&args[1])
            ),
            "text_editor" => format!("TextEditor(text: {})", self.binding_arg(&args[0])),
            "slider" => {
                let v = self.binding_arg(&args[0]);
                let r = self.ui_arg(&args[1]);
                match step {
                    Some(s) => format!("Slider(value: {v}, in: {r}, step: {s})"),
                    None => format!("Slider(value: {v}, in: {r})"),
                }
            }
            "stepper" => format!(
                "Stepper({}, value: {}, in: {})",
                self.ui_text(&args[0]),
                self.binding_arg(&args[1]),
                self.ui_arg(&args[2])
            ),
            "progress" => format!(
                "ProgressView(value: {}, total: {})",
                self.ui_arg(&args[0]),
                self.ui_arg(&args[1])
            ),
            "content_unavailable" => format!(
                "ContentUnavailableView({}, systemImage: {}, description: Text({}))",
                self.ui_text(&args[0]),
                self.ui_text(&args[1]),
                self.ui_text(&args[2])
            ),
            other => self.unsupported(
                args.first()
                    .map_or_else(proc_macro2::Span::call_site, Spanned::span),
                &format!("the view {other}()"),
            ),
        };
        if !children.is_empty() && !CONTAINERS.contains(&name) {
            self.warn(
                methods
                    .first()
                    .map_or_else(proc_macro2::Span::call_site, |m| m.span()),
                format!("{name}() has no children: .child() is ignored"),
            );
        }
        let modifiers = self.modifiers(&rest);
        attach(base, &modifiers)
    }

    /// `.children(…)`: a list of views or a `ForEach`.
    fn view_children(&mut self, e: &Expr) -> Vec<String> {
        match e {
            Expr::Array(_) => self.view_lines(e),
            Expr::Macro(m) if m.mac.path.is_ident("vec") => self.view_lines(e),
            Expr::MethodCall(m) if m.method == "map" && m.args.len() == 1 => {
                let items = strip_iter(&m.receiver);
                vec![self.for_each(items, &m.args[0])]
            }
            Expr::MethodCall(m) if m.method == "collect" => self.view_children(&m.receiver),
            other => {
                let v = self.expr(other);
                let v = self.settle(v);
                vec![format!(
                    "ForEach(Array({}.enumerated()), id: \\.offset) {{ $0.element }}",
                    v.code
                )]
            }
        }
    }

    /// `for_each(items, |item| row)` → `ForEach(items) { item in row }`.
    fn for_each(&mut self, items: &Expr, row: &Expr) -> String {
        let binding = matches!(items, Expr::Reference(r) if r.mutability.is_some());
        let (items_expr, enumerated) = match items {
            Expr::MethodCall(m) if m.method == "enumerate" => (strip_iter(&m.receiver), true),
            e => (strip_iter(e), false),
        };
        let v = self.expr(items_expr);
        let v = self.settle(v);
        let item_ty = v.ty.item();
        let mut items_code = v.code.clone();
        let id = if enumerated {
            items_code = format!("Array({items_code}.enumerated())");
            ", id: \\.offset".to_string()
        } else if binding {
            items_code = match items_code.strip_prefix("self.") {
                Some(rest) => format!("${rest}"),
                None => format!("${items_code}"),
            };
            String::new()
        } else {
            match &item_ty {
                Ty::Int(_) | Ty::Str | Ty::Char | Ty::Bool | Ty::Float(_) => ", id: \\.self".into(),
                Ty::Named(n, _) => {
                    let identifiable = self
                        .struct_by_swift(n)
                        .is_some_and(|s| s.field("id").is_some());
                    if identifiable {
                        String::new()
                    } else {
                        ", id: \\.self".into()
                    }
                }
                Ty::Unknown if matches!(v.ty, Ty::Range(_)) => ", id: \\.self".into(),
                _ => String::new(),
            }
        };
        let Expr::Closure(c) = row else {
            let f = self.expr(row);
            return format!("ForEach({items_code}{id}, content: {})", f.code);
        };
        self.push_scope();
        let param = match c.inputs.first() {
            Some(syn::Pat::Ident(id)) => {
                let n = self.declare(&id.ident.to_string(), item_ty.clone());
                if binding {
                    self.binding_items.push(n.clone());
                    format!("${n}")
                } else {
                    n
                }
            }
            Some(syn::Pat::Tuple(t)) if enumerated => {
                let mut names = Vec::new();
                for (i, p) in t.elems.iter().enumerate() {
                    let ty = if i == 0 {
                        Ty::Int(crate::ty::IntKind::Usize)
                    } else {
                        item_ty.clone()
                    };
                    match p {
                        syn::Pat::Ident(id) => names.push(self.declare(&id.ident.to_string(), ty)),
                        _ => names.push("_".into()),
                    }
                }
                format!("({})", names.join(", "))
            }
            Some(syn::Pat::Reference(r)) => match &*r.pat {
                syn::Pat::Ident(id) => self.declare(&id.ident.to_string(), item_ty.clone()),
                _ => "_".into(),
            },
            _ => "_".into(),
        };
        self.fns.push(FnCx {
            ret: Ty::View,
            throws: None,
            init: false,
            closure: false,
            self_name: None,
        });
        let body = self.view_content(&c.body);
        self.fns.pop();
        if binding {
            self.binding_items.pop();
        }
        self.pop_scope();
        format!(
            "ForEach({items_code}{id}) {{ {param} in\n{}\n}}",
            indent_lines(&body.join("\n"))
        )
    }

    /// The modifiers of a view, one per line.
    fn modifiers(&mut self, methods: &[&ExprMethodCall]) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < methods.len() {
            let name = methods[i].method.to_string();
            // The comments written before `.padding()` go before it.
            let before = methods[i].receiver.span().end().line;
            let dot = methods[i].dot_token.span().start().line;
            out.extend(self.comments.take_between(before, dot));
            if FRAME.contains(&name.as_str()) {
                let mut j = i;
                let mut fixed: Vec<(String, String)> = Vec::new();
                let mut flexible: Vec<(String, String)> = Vec::new();
                let mut align = None;
                while j < methods.len() && FRAME.contains(&methods[j].method.to_string().as_str()) {
                    let m = methods[j];
                    let a: Vec<String> = m.args.iter().map(|e| self.ui_arg(e)).collect();
                    match m.method.to_string().as_str() {
                        "frame" => {
                            fixed.push(("width".into(), a[0].clone()));
                            fixed.push(("height".into(), a[1].clone()));
                        }
                        "frame_width" => fixed.push(("width".into(), a[0].clone())),
                        "frame_height" => fixed.push(("height".into(), a[0].clone())),
                        "frame_min_width" => flexible.push(("minWidth".into(), a[0].clone())),
                        "frame_min_height" => flexible.push(("minHeight".into(), a[0].clone())),
                        "frame_max_width" => flexible.push(("maxWidth".into(), a[0].clone())),
                        "frame_max_height" => flexible.push(("maxHeight".into(), a[0].clone())),
                        "fill_width" => flexible.push(("maxWidth".into(), ".infinity".into())),
                        "fill_height" => flexible.push(("maxHeight".into(), ".infinity".into())),
                        _ => align = Some(a[0].clone()),
                    }
                    j += 1;
                }
                let order = [
                    "width",
                    "height",
                    "minWidth",
                    "idealWidth",
                    "maxWidth",
                    "minHeight",
                    "idealHeight",
                    "maxHeight",
                ];
                let sort = |v: &mut Vec<(String, String)>| {
                    v.sort_by_key(|(k, _)| order.iter().position(|o| o == k))
                };
                sort(&mut fixed);
                sort(&mut flexible);
                let render = |v: &[(String, String)], align: Option<&String>| {
                    let mut parts: Vec<String> =
                        v.iter().map(|(k, x)| format!("{k}: {x}")).collect();
                    if let Some(a) = align {
                        parts.push(format!("alignment: {a}"));
                    }
                    format!(".frame({})", parts.join(", "))
                };
                match (fixed.is_empty(), flexible.is_empty()) {
                    (false, true) => out.push(render(&fixed, align.as_ref())),
                    (true, false) => out.push(render(&flexible, align.as_ref())),
                    (false, false) => {
                        out.push(render(&fixed, None));
                        out.push(render(&flexible, align.as_ref()));
                    }
                    (true, true) => {
                        if let Some(a) = align {
                            out.push(format!(".frame(alignment: {a})"));
                        }
                    }
                }
                i = j;
                continue;
            }
            let mut code = self.modifier(methods[i]);
            // `.padding() // around`
            if let Some(c) = self.comments.take_trailing(methods[i].span().end().line) {
                code.push(' ');
                code.push_str(&c);
            }
            out.push(code);
            i += 1;
        }
        out
    }

    fn modifier(&mut self, mc: &ExprMethodCall) -> String {
        let name = mc.method.to_string();
        let args: Vec<Expr> = mc.args.iter().cloned().collect();
        let a = |t: &mut Self, i: usize| t.ui_arg(&args[i]);
        let content = |t: &mut Self, i: usize| -> String {
            let lines = t.view_content(&args[i]);
            format!("{{\n{}\n}}", indent_lines(&lines.join("\n")))
        };
        match name.as_str() {
            "padding" => ".padding()".into(),
            "padding_len" => format!(".padding({})", a(self, 0)),
            "padding_edges" => format!(".padding({}, {})", a(self, 0), a(self, 1)),
            "font" => format!(".font({})", a(self, 0)),
            "font_size" => format!(".font(.system(size: {}))", a(self, 0)),
            "foreground" | "foreground_color" | "foreground_style" => {
                format!(".foregroundStyle({})", a(self, 0))
            }
            "background_view" => format!(".background {}", content(self, 0)),
            "overlay" => format!(".overlay {}", content(self, 0)),
            "stroke" => format!(".stroke({}, lineWidth: {})", a(self, 0), a(self, 1)),
            "border" => format!(".border({}, width: {})", a(self, 0), a(self, 1)),
            "corner_radius" => format!(".clipShape(.rect(cornerRadius: {}))", a(self, 0)),
            "clip_circle" => ".clipShape(.circle)".into(),
            "shadow" => format!(".shadow(radius: {})", a(self, 0)),
            "blur" => format!(".blur(radius: {})", a(self, 0)),
            "rotation" => format!(".rotationEffect(.degrees({}))", a(self, 0)),
            "scale" => format!(".scaleEffect({})", a(self, 0)),
            "offset" => format!(".offset(x: {}, y: {})", a(self, 0), a(self, 1)),
            "aspect_ratio" => format!(".aspectRatio(contentMode: {})", a(self, 0)),
            "navigation_inline" => {
                "#if os(iOS)\n.navigationBarTitleDisplayMode(.inline)\n#endif".into()
            }
            "toolbar" => format!(".toolbar {}", content(self, 0)),
            "tab_item" => format!(
                ".tabItem {{\n{}\n}}",
                indent_lines(&format!(
                    "Label({}, systemImage: {})",
                    self.ui_text(&args[0]),
                    self.ui_text(&args[1])
                ))
            ),
            "sheet" => format!(
                ".sheet(isPresented: {}) {}",
                self.binding_arg(&args[0]),
                content(self, 1)
            ),
            "alert" => format!(
                ".alert({}, isPresented: {}) {}",
                self.ui_text(&args[0]),
                self.binding_arg(&args[1]),
                content(self, 2)
            ),
            "searchable" => format!(".searchable(text: {})", self.binding_arg(&args[0])),
            "swipe_actions" => format!(".swipeActions {}", content(self, 0)),
            "context_menu" => format!(".contextMenu {}", content(self, 0)),
            "navigation_title" => format!(".navigationTitle({})", self.ui_text(&args[0])),
            "accessibility_label" | "help" => {
                format!(".{}({})", names::lower_camel(&name), self.ui_text(&args[0]))
            }
            "on_appear" | "on_disappear" | "on_tap_gesture" | "on_submit" => {
                format!(".{} {}", names::lower_camel(&name), self.action(&args[0]))
            }
            "on_change" => format!(".onChange(of: {}) {}", a(self, 0), self.action(&args[1])),
            "on_delete" => {
                let f = self.expr_with(
                    &args[0],
                    &Ty::Fn(
                        vec![Ty::Named("IndexSet".into(), vec![])],
                        Box::new(Ty::Unit),
                    ),
                );
                format!(".onDelete {}", f.code)
            }
            "on_move" => {
                let f = self.expr_with(
                    &args[0],
                    &Ty::Fn(
                        vec![
                            Ty::Named("IndexSet".into(), vec![]),
                            Ty::Int(crate::ty::IntKind::Usize),
                        ],
                        Box::new(Ty::Unit),
                    ),
                );
                format!(".onMove {}", f.code)
            }
            "animation" => format!(".animation({}, value: {})", a(self, 0), a(self, 1)),
            "glass_effect" => ".glassEffect()".into(),
            other => {
                let codes: Vec<String> = (0..args.len()).map(|i| a(self, i)).collect();
                if !is_modifier(other) {
                    self.warn(mc.span(), format!("unknown modifier .{other}(): translated as .{}(…), check its SwiftUI signature", names::lower_camel(other)));
                }
                format!(".{}({})", names::lower_camel(other), codes.join(", "))
            }
        }
    }
}

/// Adds modifiers below a view.
fn attach(base: String, modifiers: &[String]) -> String {
    if modifiers.is_empty() {
        return base;
    }
    let single_line = !base.contains('\n')
        && !base.contains('{')
        && modifiers.len() == 1
        && !modifiers[0].contains('\n')
        && base.len() + modifiers[0].len() < 60;
    if single_line {
        return format!("{base}{}", modifiers[0]);
    }
    // Below a closing brace, modifiers align with it; below a one-line view, they are
    // indented.
    let indent = if base.contains('\n') && base.ends_with('}') {
        ""
    } else {
        crate::translate::INDENT
    };
    let mut out = base;
    for m in modifiers {
        for line in m.lines() {
            out.push('\n');
            if !line.starts_with('#') {
                out.push_str(indent);
            }
            out.push_str(line);
        }
    }
    out
}

/// `items.iter()` → `items`.
fn strip_iter(e: &Expr) -> &Expr {
    match e {
        Expr::MethodCall(m)
            if matches!(
                m.method.to_string().as_str(),
                "iter" | "into_iter" | "iter_mut" | "clone" | "cloned"
            ) && m.args.is_empty() =>
        {
            strip_iter(&m.receiver)
        }
        Expr::Reference(r) => strip_iter(&r.expr),
        Expr::Paren(p) => strip_iter(&p.expr),
        e => e,
    }
}

/// Whether a method name is a known ferroswift modifier.
fn is_modifier(name: &str) -> bool {
    FRAME.contains(&name)
        || matches!(
            name,
            "padding"
                | "padding_len"
                | "padding_edges"
                | "font"
                | "font_size"
                | "font_weight"
                | "font_design"
                | "bold"
                | "italic"
                | "monospaced"
                | "underline"
                | "strikethrough"
                | "multiline_text_alignment"
                | "line_limit"
                | "foreground"
                | "background"
                | "background_view"
                | "overlay"
                | "tint"
                | "fill"
                | "stroke"
                | "border"
                | "corner_radius"
                | "clip_circle"
                | "opacity"
                | "shadow"
                | "blur"
                | "rotation"
                | "scale"
                | "glass_effect"
                | "resizable"
                | "scaled_to_fit"
                | "scaled_to_fill"
                | "aspect_ratio"
                | "image_scale"
                | "disabled"
                | "hidden"
                | "button_style"
                | "text_field_style"
                | "picker_style"
                | "list_style"
                | "tag"
                | "id"
                | "badge"
                | "accessibility_label"
                | "help"
                | "labels_hidden"
                | "preferred_color_scheme"
                | "navigation_title"
                | "navigation_inline"
                | "toolbar"
                | "tab_item"
                | "sheet"
                | "alert"
                | "searchable"
                | "swipe_actions"
                | "context_menu"
                | "on_appear"
                | "on_disappear"
                | "on_tap_gesture"
                | "on_submit"
                | "on_change"
                | "on_delete"
                | "on_move"
                | "animation"
                | "transition"
                | "offset"
                | "ignores_safe_area"
        )
}

#[allow(dead_code)]
fn quoted(s: &str) -> String {
    swift_string(s)
}
