//! The translator: its state, and the translation of items (types, traits, impls, functions).

use std::collections::{BTreeSet, HashMap, HashSet};

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{
    Attribute, Fields, FnArg, GenericParam, ImplItem, Item, Pat, ReturnType, TraitItem, Type,
};

use crate::Options;
use crate::comments::Comments;
use crate::diag::{Diagnostic, Severity, position};
use crate::names;
use crate::stmt::Sink;
use crate::symbols::{
    EnumInfo, FieldRole, SelfKind, Symbols, TypeKind, VariantShape, derives, is_test_only,
    item_attrs,
};
use crate::ty::Ty;

pub const INDENT: &str = "    ";

/// A local variable.
#[derive(Clone, Debug)]
pub struct Local {
    /// Its Swift name, or the Swift expression it stands for (`items[i]` for the item of a
    /// `for x in items.iter_mut()` loop).
    pub swift: String,
    pub ty: Ty,
    /// The placeholder of its declared type, refined by later uses (`v.push(1)`).
    pub slot: Option<usize>,
    pub alias: bool,
}

#[derive(Default)]
pub struct Scope {
    pub vars: Vec<(String, Local)>,
    pub declared: HashSet<String>,
}

/// How a function returns.
#[derive(Clone, Debug)]
pub struct FnCx {
    /// The Rust return type.
    pub ret: Ty,
    /// The error type when the Rust function returns a `Result`: the Swift one `throws`.
    pub throws: Option<Ty>,
    /// `new` / `default` / `from` translated as `init`.
    pub init: bool,
    pub closure: bool,
    /// The name `self` is copied to (`mut self` receivers).
    pub self_name: Option<String>,
}

/// The type whose `impl` is being translated.
#[derive(Clone, Debug)]
pub struct ImplCx {
    pub rust: String,
    pub swift: String,
    pub self_ty: Ty,
    pub kind: TypeKind,
    /// In an operator function (`static func + (lhs:rhs:)`): `self` is `lhs`.
    pub self_rename: Option<String>,
}

pub struct Translator<'a> {
    pub syms: &'a Symbols,
    pub opts: &'a Options,
    pub diags: Vec<Diagnostic>,
    pub helpers: BTreeSet<&'static str>,
    pub comments: Comments,
    pub file: Option<String>,
    pub scopes: Vec<Scope>,
    pub fns: Vec<FnCx>,
    pub preludes: Vec<Vec<String>>,
    pub slots: Vec<Ty>,
    pub tmp: usize,
    pub imp: Option<ImplCx>,
    /// Translating an argument of a SwiftUI modifier: `Color::Red` is `.red`.
    pub implicit_member: bool,
    /// The file uses SwiftUI.
    pub used_ui: bool,
    /// Generic parameters in scope.
    pub generics: Vec<String>,
    /// Generic parameters of the current function bounded by `Fn(…)`: their closure type.
    pub closure_generics: std::collections::HashMap<String, Ty>,
    /// Labels of the loops being translated, innermost last.
    pub loops: Vec<Option<String>>,
    /// `switch` statements between the innermost loop and the code being translated: a
    /// `break` there must name the loop.
    pub switch_depth: usize,
    /// Generated loop labels that a `break` names.
    pub used_labels: std::collections::HashSet<String>,
    /// The type of the branches of the `if`/`switch` expression being translated.
    pub last_branch_ty: Option<Ty>,
    /// Depth of closures whose body is a result builder of views.
    pub view_closure_depth: usize,
    /// Depth of `if`/`switch` expressions whose branches are values (not views).
    pub value_yield: usize,
    /// Items of `ForEach($items) { $item in … }`: `&mut item.done` is `$item.done`.
    pub binding_items: Vec<String>,
    /// Types of the values of the last `format!`.
    pub format_types: Vec<Ty>,
    /// A type learned by the method call being translated, for its receiver.
    pub pending_refine: Option<Ty>,
    /// Statements to put at the start of the next loop body (`for mut x in …`).
    pub prelude_body: Vec<String>,
    /// Inside a condition (`if`, `while`, `guard`, `for … in`): no trailing closures there.
    pub no_trailing: usize,
    /// Inside the receiver of a `collect()` into a plain collection: closures returning a
    /// `Result` return it as a value (`.map(|s| s.parse::<i32>()).collect::<Vec<_>>()`).
    pub result_values: usize,
    /// Inside the arguments of a call to a generic function: an empty array literal is cast
    /// to its type, which Swift cannot infer otherwise.
    pub generic_call: usize,
    /// Parameters of the function being translated that its body uses as values.
    pub escaping_params: std::collections::HashSet<String>,
    /// Locals being declared (`let max = …`): their initializers already see the shadowing.
    pub pending_locals: Vec<String>,
    /// For each loop being translated, the variable its `break value` assigns.
    pub loop_values: Vec<Option<String>>,
    /// Nesting of the expression being translated.
    pub depth: usize,
}

impl<'a> Translator<'a> {
    pub fn new(syms: &'a Symbols, opts: &'a Options, source: &str, file: Option<String>) -> Self {
        Translator {
            syms,
            opts,
            diags: Vec::new(),
            helpers: BTreeSet::new(),
            comments: Comments::parse(source),
            file,
            scopes: Vec::new(),
            fns: Vec::new(),
            preludes: Vec::new(),
            slots: Vec::new(),
            tmp: 0,
            imp: None,
            implicit_member: false,
            used_ui: false,
            generics: Vec::new(),
            closure_generics: std::collections::HashMap::new(),
            loops: Vec::new(),
            switch_depth: 0,
            used_labels: std::collections::HashSet::new(),
            last_branch_ty: None,
            view_closure_depth: 0,
            value_yield: 0,
            binding_items: Vec::new(),
            format_types: Vec::new(),
            pending_refine: None,
            prelude_body: Vec::new(),
            no_trailing: 0,
            result_values: 0,
            generic_call: 0,
            escaping_params: std::collections::HashSet::new(),
            pending_locals: Vec::new(),
            loop_values: Vec::new(),
            depth: 0,
        }
    }

    // ----- Diagnostics -----

    pub fn diag(&mut self, severity: Severity, span: Span, message: impl Into<String>) {
        let (line, column) = position(span);
        self.diags.push(Diagnostic {
            severity,
            message: message.into(),
            file: self.file.clone(),
            line,
            column,
        });
    }

    pub fn error(&mut self, span: Span, message: impl Into<String>) {
        self.diag(Severity::Error, span, message);
    }

    pub fn warn(&mut self, span: Span, message: impl Into<String>) {
        self.diag(Severity::Warning, span, message);
    }

    /// Reports an unsupported construct and returns Swift code that compiles in its place.
    pub fn unsupported(&mut self, span: Span, what: &str) -> String {
        self.error(span, format!("not supported yet: {what}"));
        format!(
            "fatalError(\"FerroSwift: {} is not supported\")",
            what.replace('"', "'")
        )
    }

    pub fn helper(&mut self, name: &'static str) {
        self.helpers.insert(name);
    }

    // ----- Scopes -----

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declares a local and returns its Swift name, renamed when the current scope already
    /// declares that name (Rust allows `let x = …; let x = …;`, Swift does not).
    pub fn declare(&mut self, rust: &str, ty: Ty) -> String {
        let base = if rust == "self" {
            "copy".to_string()
        } else {
            names::lower_camel(rust)
        };
        if self.scopes.is_empty() {
            self.push_scope();
        }
        let scope = self.scopes.last_mut().expect("scope");
        let mut swift = base.clone();
        let mut n = 2;
        while scope.declared.contains(&swift) {
            let bare = base.trim_matches('`');
            swift = format!("{bare}{n}");
            n += 1;
        }
        scope.declared.insert(swift.clone());
        scope.vars.push((
            rust.to_string(),
            Local {
                swift: swift.clone(),
                ty,
                slot: None,
                alias: false,
            },
        ));
        swift
    }

    /// Declares a local standing for a Swift expression (`items[i]`).
    pub fn declare_alias(&mut self, rust: &str, expr: String, ty: Ty) {
        if self.scopes.is_empty() {
            self.push_scope();
        }
        let scope = self.scopes.last_mut().expect("scope");
        scope.vars.push((
            rust.to_string(),
            Local {
                swift: expr,
                ty,
                slot: None,
                alias: true,
            },
        ));
    }

    pub fn lookup(&self, rust: &str) -> Option<&Local> {
        self.scopes
            .iter()
            .rev()
            .flat_map(|s| s.vars.iter().rev())
            .find(|(n, _)| n == rust)
            .map(|(_, l)| l)
    }

    fn lookup_mut(&mut self, rust: &str) -> Option<&mut Local> {
        self.scopes
            .iter_mut()
            .rev()
            .flat_map(|s| s.vars.iter_mut().rev())
            .find(|(n, _)| n == rust)
            .map(|(_, l)| l)
    }

    /// Whether a local with this Swift name is visible (then `self.x` keeps its `self.`).
    /// `max`, `abs`, `zip`… of the Swift standard library, qualified when a local hides them.
    pub fn global_fn(&self, name: &str) -> String {
        // A field or method `min` of the type hides the global function in its methods.
        let member = self.imp.as_ref().is_some_and(|i| {
            self.syms
                .structs
                .get(&i.rust)
                .is_some_and(|s| s.fields.iter().any(|f| f.swift == name))
                || self
                    .syms
                    .methods
                    .get(&i.rust)
                    .is_some_and(|m| m.keys().any(|k| names::lower_camel(k) == name))
        });
        if member || self.swift_local_visible(name) || self.pending_locals.iter().any(|l| l == name)
        {
            format!("Swift.{name}")
        } else {
            name.to_string()
        }
    }

    pub fn swift_local_visible(&self, swift: &str) -> bool {
        self.scopes
            .iter()
            .any(|s| s.vars.iter().any(|(_, l)| !l.alias && l.swift == swift))
    }

    /// The current type of a local.
    pub fn local_ty(&self, local: &Local) -> Ty {
        match local.slot {
            Some(slot) => self.slots[slot].clone(),
            None => local.ty.clone(),
        }
    }

    /// A type annotation that later uses can complete: `: [Int]` once `v.push(1)` is seen.
    pub fn new_slot(&mut self, ty: Ty) -> (usize, String) {
        self.slots.push(ty);
        let id = self.slots.len() - 1;
        (id, format!("\u{1}{id}\u{1}"))
    }

    pub fn attach_slot(&mut self, rust: &str, slot: usize) {
        if let Some(local) = self.lookup_mut(rust) {
            local.slot = Some(slot);
        }
    }

    /// Learns more about the type of a local from its use.
    pub fn refine(&mut self, rust: &str, ty: &Ty) {
        let Some(local) = self.lookup(rust) else {
            return;
        };
        match local.slot {
            Some(slot) => self.slots[slot].merge(ty),
            None => {
                let local = self.lookup_mut(rust).expect("local");
                local.ty.merge(ty);
            }
        }
    }

    /// Replaces the type placeholders of `code` with the types learned since.
    pub fn resolve_slots(&mut self, code: &str) -> String {
        if !code.contains('\u{1}') {
            return code.to_string();
        }
        // The types written here need their support code too.
        for i in 0..self.slots.len() {
            let t = self.slots[i].clone();
            self.note_types(&t);
        }
        let mut out = String::with_capacity(code.len());
        let mut parts = code.split('\u{1}');
        if let Some(first) = parts.next() {
            out.push_str(first);
        }
        let mut is_slot = true;
        for part in parts {
            if is_slot {
                if let Some(t) = part.parse::<usize>().ok().and_then(|i| self.slots.get(i)) {
                    if let Some(swift) = t.swift() {
                        out.push_str(": ");
                        out.push_str(&swift);
                    }
                }
            } else {
                out.push_str(part);
            }
            is_slot = !is_slot;
        }
        out
    }

    pub fn fresh(&mut self, base: &str) -> String {
        self.tmp += 1;
        format!("{base}{}", self.tmp)
    }

    pub fn fn_cx(&self) -> Option<&FnCx> {
        self.fns.last()
    }

    /// Statements to write before the statement being translated (`guard let` for `?` on
    /// an `Option`, temporaries).
    pub fn prelude(&mut self, line: String) {
        if let Some(p) = self.preludes.last_mut() {
            p.push(line);
        }
    }

    // ----- Files -----

    pub fn file(&mut self, file: &syn::File, entry: bool) -> String {
        let mut out = String::new();
        if self.opts.header {
            match &self.file {
                Some(path) => {
                    out.push_str(&format!(
                        "// Generated by FerroSwift from {path}. Do not edit.\n"
                    ));
                }
                None => out.push_str("// Generated by FerroSwift. Do not edit.\n"),
            }
        }
        for a in &file.attrs {
            if let Some(doc) = doc_text(a) {
                out.push_str(&format!("//{doc}\n"));
            }
        }
        if !out.is_empty() {
            out.push('\n');
        }
        let ui = self.syms.ui
            && file
                .items
                .iter()
                .any(|i| matches!(i, Item::Use(u) if quote::quote!(#u).to_string().contains("ferroswift")));
        let mut items = Vec::new();
        let mut impls_by_type: HashMap<String, Vec<&syn::ItemImpl>> = HashMap::new();
        collect_impls(&file.items, &mut impls_by_type);
        let mut has_main = false;
        let mut has_app = false;
        self.items(
            &file.items,
            &impls_by_type,
            &mut items,
            &mut has_main,
            &mut has_app,
            0,
        );
        let uses_ui = ui || self.used_ui;
        self.used_ui = uses_ui;
        out.push_str(if uses_ui {
            "import SwiftUI\n"
        } else {
            "import Foundation\n"
        });
        for item in items {
            out.push('\n');
            out.push_str(&item);
            if !item.ends_with('\n') {
                out.push('\n');
            }
        }
        if entry && has_main && !has_app {
            out.push_str("\nmain()\n");
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn items(
        &mut self,
        items: &[Item],
        impls: &HashMap<String, Vec<&syn::ItemImpl>>,
        out: &mut Vec<String>,
        has_main: &mut bool,
        has_app: &mut bool,
        mut prev_line: usize,
    ) {
        for item in items {
            if is_test_only(item_attrs(item)) {
                prev_line = item.span().end().line;
                continue;
            }
            let start = first_line(item_attrs(item), item.span());
            let leading = self.comments.take_between(prev_line, start);
            prev_line = item.span().end().line;
            let code = match item {
                Item::Mod(m) => {
                    if let Some((_, inner)) = &m.content {
                        let line = m.span().start().line;
                        self.items(inner, impls, out, has_main, has_app, line);
                    }
                    continue;
                }
                Item::Use(_) | Item::ExternCrate(_) => continue,
                Item::Struct(s) => {
                    if let Some(info) = self.syms.structs.get(&s.ident.to_string()) {
                        if info.kind == TypeKind::App {
                            *has_app = true;
                        }
                    }
                    self.item_struct(s, impls.get(&s.ident.to_string()))
                }
                Item::Enum(e) => self.item_enum(e),
                Item::Impl(i) => {
                    let ty_name = crate::symbols::type_name(&i.self_ty).unwrap_or_default();
                    let trait_name = impl_trait_name(i);
                    let merged = matches!(trait_name.as_deref(), Some("View" | "App"))
                        && self
                            .syms
                            .structs
                            .get(&ty_name)
                            .is_some_and(|s| matches!(s.kind, TypeKind::View | TypeKind::App))
                        && impls.contains_key(&ty_name)
                        && struct_in(items, &ty_name);
                    if merged {
                        continue;
                    }
                    self.item_impl(i)
                }
                Item::Trait(t) => self.item_trait(t),
                Item::Fn(f) => {
                    if f.sig.ident == "main" {
                        *has_main = true;
                    }
                    if has_attr(&f.attrs, "preview") {
                        if !self.opts.previews {
                            continue;
                        }
                        self.preview(f)
                    } else {
                        self.item_fn(f)
                    }
                }
                Item::Const(c) => self.global(&c.attrs, &c.ident, &c.ty, &c.expr, false),
                Item::Static(s) => {
                    let mutable = matches!(s.mutability, syn::StaticMutability::Mut(_));
                    self.global(&s.attrs, &s.ident, &s.ty, &s.expr, mutable)
                }
                Item::Type(t) => {
                    let ty = self.ty(&t.ty);
                    let generics = self.generic_params(&t.generics);
                    format!(
                        "{}typealias {}{} = {}",
                        docs(&t.attrs),
                        names::upper_camel(&t.ident.to_string()),
                        generics,
                        self.swift_ty(&ty, t.ty.span())
                    )
                }
                Item::Macro(m) => {
                    if m.mac.path.is_ident("macro_rules") {
                        self.warn(m.span(), "macro_rules! definitions are not translated");
                    } else {
                        self.warn(m.span(), "item macros are not translated");
                    }
                    continue;
                }
                other => {
                    self.error(other.span(), "this kind of item is not supported");
                    continue;
                }
            };
            let mut code = self.resolve_slots(&code);
            if !leading.is_empty() {
                code = format!("{}\n{code}", leading.join("\n"));
            }
            out.push(code);
        }
    }

    fn global(
        &mut self,
        attrs: &[Attribute],
        ident: &syn::Ident,
        ty: &Type,
        expr: &syn::Expr,
        mutable: bool,
    ) -> String {
        let t = self.ty(ty);
        self.push_scope();
        let value = self.expr_with(expr, &t);
        let value = self.settle(value);
        self.pop_scope();
        let annot = if natural_type_matches(&value.ty, &t) {
            String::new()
        } else {
            format!(": {}", self.swift_ty(&t, ty.span()))
        };
        format!(
            "{}{} {}{annot} = {}",
            docs(attrs),
            if mutable { "var" } else { "let" },
            names::constant(&ident.to_string()),
            value.code
        )
    }

    // ----- Types -----

    pub fn ty(&self, t: &Type) -> Ty {
        let self_ty = self.imp.as_ref().map(|i| i.self_ty.clone());
        let ty = self.syms.ty(t, self_ty.as_ref());
        if self.closure_generics.is_empty() {
            ty
        } else {
            crate::symbols::substitute(&ty, &self.closure_generics)
        }
    }

    /// The Swift spelling of a type; unknown parts are reported.
    pub fn swift_ty(&mut self, t: &Ty, span: Span) -> String {
        self.note_types(t);
        match t.swift() {
            Some(s) => s,
            None => {
                self.error(
                    span,
                    format!("cannot translate the type `{}`", t.swift_lossy()),
                );
                t.swift_lossy().replace('_', "Any")
            }
        }
    }

    /// The support code that a type written in Swift needs.
    pub fn note_types(&mut self, t: &Ty) {
        // `Result<T, String>` anywhere: `String` must be an `Error`.
        if mentions_string_error(t) {
            self.helper("StringError");
        }
        self.note_support_types(t);
    }

    /// The support code behind the library types that `t` mentions (`RsParseIntError`…).
    pub fn note_support_types(&mut self, t: &Ty) {
        let mut names = Vec::new();
        named_types(t, &mut names);
        for n in names {
            let helper = match n.as_str() {
                "RsParseIntError" | "RsParseFloatError" | "RsParseBoolError"
                | "RsParseCharError" | "RsCharTryFromError" => "parse",
                "RsTryFromIntError" => "try_from",
                "RsFromUtf8Error" | "RsUtf8Error" => "utf8",
                "RsTuple1" => "tuple1",
                "RsBinaryHeap" => "heap",
                "RsReverse" => "reverse",
                "RsTupleKeys" => "tuple_keys",
                "RsOrdTuple" => "ord_tuple",
                _ => continue,
            };
            self.helper(helper);
        }
    }

    /// Generic parameters: `<T: Comparable, U>`.
    pub fn generic_params(&mut self, g: &syn::Generics) -> String {
        let mut parts = Vec::new();
        for p in &g.params {
            match p {
                GenericParam::Type(t)
                    if self.closure_generics.contains_key(&t.ident.to_string()) => {}
                GenericParam::Type(t) => {
                    let bounds = self.bounds(&t.bounds);
                    if bounds.is_empty() {
                        parts.push(t.ident.to_string());
                    } else {
                        parts.push(format!("{}: {}", t.ident, bounds.join(" & ")));
                    }
                }
                GenericParam::Const(c) => {
                    self.error(c.span(), "const generic parameters are not supported");
                }
                GenericParam::Lifetime(_) => {}
            }
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("<{}>", parts.join(", "))
        }
    }

    /// `where T: P` clauses.
    pub fn where_clause(&mut self, g: &syn::Generics) -> String {
        let Some(w) = &g.where_clause else {
            return String::new();
        };
        let mut parts = Vec::new();
        for pred in &w.predicates {
            if let syn::WherePredicate::Type(pt) = pred {
                let ty = self.ty(&pt.bounded_ty);
                if matches!(ty, Ty::Fn(..)) {
                    continue;
                }
                let name = ty.swift_lossy();
                for b in self.bounds(&pt.bounds) {
                    parts.push(format!("{name}: {b}"));
                }
            }
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(" where {}", parts.join(", "))
        }
    }

    /// Trait bounds as Swift protocols (`Display` → `CustomStringConvertible`; `Clone`,
    /// `Copy`, `Send`… have no Swift counterpart and disappear).
    pub fn bounds(
        &mut self,
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
    ) -> Vec<String> {
        let mut out = Vec::new();
        for b in bounds {
            if let syn::TypeParamBound::Trait(t) = b {
                let Some(seg) = t.path.segments.last() else {
                    continue;
                };
                let name = seg.ident.to_string();
                if matches!(name.as_str(), "Fn" | "FnMut" | "FnOnce") {
                    // `F: Fn(i32) -> i32`: the parameter is replaced by the closure type.
                    continue;
                }
                // `{:?}` goes through reflection: no protocol to require.
                if name == "Debug" {
                    continue;
                }
                if let Some(p) = protocol_name(&name) {
                    if !out.contains(&p) {
                        out.push(p);
                    }
                } else if !is_dropped_trait(&name) {
                    out.push(names::upper_camel(&name));
                }
            }
        }
        out
    }

    // ----- Structs -----

    fn item_struct(&mut self, s: &syn::ItemStruct, impls: Option<&Vec<&syn::ItemImpl>>) -> String {
        let name = s.ident.to_string();
        let info = self
            .syms
            .structs
            .get(&name)
            .cloned()
            .expect("collected struct");
        let swift = info.swift.clone();
        let generics = self.generic_params(&s.generics);
        let where_clause = self.where_clause(&s.generics);
        self.generics = info.generics.clone();
        let derives = derives(&s.attrs);
        let derive_default = derives.iter().any(|d| d == "Default");
        let mut conformances = self.derive_conformances(&derives, s.span());
        if info.fields.iter().any(|f| f.rust == "id") && info.kind != TypeKind::App {
            conformances.push("Identifiable".into());
        }
        // A `BTreeSet<T>` is a `Set<T>`: an ordered type is hashable too, when its fields are.
        if derives.iter().any(|d| d == "Ord")
            && !conformances.iter().any(|c| c == "Hashable")
            && info.fields.iter().all(|f| self.hashable(&f.ty))
        {
            conformances.push("Hashable".into());
            conformances.retain(|c| c != "Equatable");
        }
        if self.syms.errors.contains(&swift) && !conformances.iter().any(|c| c == "Error") {
            conformances.push("Error".into());
        }
        let mut body: Vec<String> = Vec::new();
        let mut prefix = String::new();
        let keyword = match info.kind {
            TypeKind::View => {
                self.used_ui = true;
                conformances.insert(0, "View".into());
                "struct"
            }
            TypeKind::App => {
                self.used_ui = true;
                conformances.insert(0, "App".into());
                prefix.push_str("@main\n");
                "struct"
            }
            TypeKind::Observable => {
                self.used_ui = true;
                prefix.push_str("@Observable\n");
                "final class"
            }
            // A value type cannot contain itself: a class, with the value semantics written out.
            TypeKind::Plain if info.recursive => "final class",
            TypeKind::Plain => "struct",
        };
        if info.recursive && conformances.iter().any(|c| c == "Hashable") {
            self.warn(
                s.span(),
                "a recursive struct becomes a class: Hashable is dropped",
            );
            conformances.retain(|c| c != "Hashable");
        }
        conformances.retain(|c| {
            !(info.kind == TypeKind::Observable && (c == "Equatable" || c == "Hashable"))
        });
        dedup(&mut conformances);
        // A generic type conforms when its parameters do: `extension Pair: Equatable where A: Equatable`.
        let conditional: Vec<String> = if info.generics.is_empty() {
            Vec::new()
        } else {
            let moved: Vec<String> = conformances
                .iter()
                .filter(|c| is_synthesized(c))
                .cloned()
                .collect();
            conformances.retain(|c| !is_synthesized(c));
            moved
        };

        // Fields.
        let field_spans: Vec<(Span, Vec<Attribute>)> = s
            .fields
            .iter()
            .map(|f| (f.span(), f.attrs.clone()))
            .collect();
        let mut prev = s.span().start().line;
        for (i, field) in info.fields.iter().enumerate() {
            // Two structs of the same name in different modules: the other one was collected.
            let Some((span, attrs)) = field_spans.get(i) else {
                break;
            };
            let leading = self.comments.take_between(prev, span.start().line);
            prev = span.end().line;
            body.extend(leading);
            let doc = docs(attrs);
            let ty = self.swift_ty(&field.ty, *span);
            let line = match (&field.role, info.kind) {
                (FieldRole::State(init), _) => {
                    let (value, value_ty) = match init {
                        Some(e) => {
                            self.push_scope();
                            let v = self.expr_with(e, &field.ty);
                            let v = self.settle(v);
                            self.pop_scope();
                            (v.code, v.ty)
                        }
                        None => {
                            match field.ty.default_value() {
                                Some(v) => (v, field.ty.clone()),
                                None => {
                                    self.error(*span, "give this #[state] field an initial value: #[state(value)]");
                                    ("fatalError()".into(), Ty::Unknown)
                                }
                            }
                        }
                    };
                    if natural_type_matches(&value_ty, &field.ty)
                        && !value.starts_with('.')
                        && !matches!(
                            field.ty,
                            Ty::Option(_) | Ty::Vec(_) | Ty::Map(..) | Ty::Set(..)
                        )
                    {
                        format!("@State var {} = {value}", field.swift)
                    } else {
                        format!("@State var {}: {ty} = {value}", field.swift)
                    }
                }
                (FieldRole::Binding, _) => format!("@Binding var {}: {ty}", field.swift),
                (FieldRole::Environment(key), _) => {
                    format!(
                        "@Environment(\\.{}) var {}",
                        names::lower_camel(key),
                        field.swift
                    )
                }
                (FieldRole::Ignored, _) => {
                    let default = self.field_default(field, derive_default);
                    format!("@ObservationIgnored var {}: {ty}{default}", field.swift)
                }
                // An observable model given by the parent: `@Bindable` lets the view bind
                // to its properties (`$library.query`).
                (FieldRole::Plain, TypeKind::View) if self.is_observable(&field.ty) => {
                    format!("@Bindable var {}: {ty}", field.swift)
                }
                (FieldRole::Plain, TypeKind::View) => format!("let {}: {ty}", field.swift),
                (FieldRole::Plain, _) => {
                    let default = self.field_default(
                        field,
                        derive_default
                            || info.kind == TypeKind::Observable
                                && field.ty.default_value().is_some()
                                && derive_default,
                    );
                    format!("var {}: {ty}{default}", field.swift)
                }
            };
            let trailing = self
                .comments
                .take_trailing(span.end().line)
                .map(|c| format!(" {c}"))
                .unwrap_or_default();
            body.push(format!("{doc}{line}{trailing}"));
        }

        // Initializers that Swift does not synthesize.
        if info.tuple && !info.fields.is_empty() {
            let params: Vec<String> = info
                .fields
                .iter()
                .map(|f| format!("_ {}: {}", f.swift, f.ty.swift_lossy()))
                .collect();
            let assigns: Vec<String> = info
                .fields
                .iter()
                .map(|f| format!("{INDENT}self.{0} = {0}", f.swift))
                .collect();
            body.push(String::new());
            body.push(format!(
                "init({}) {{\n{}\n}}",
                params.join(", "),
                assigns.join("\n")
            ));
            if derive_default {
                body.push(String::new());
                body.push("init() {}".into());
            }
        }
        if info.recursive {
            // Classes get no synthesized `==`, and `clone()` must copy deeply.
            if conformances.iter().any(|c| c == "Equatable") {
                let terms: Vec<String> = info
                    .fields
                    .iter()
                    .map(|f| format!("lhs.{0} == rhs.{0}", f.swift))
                    .collect();
                body.push(String::new());
                body.push(format!(
                    "static func == (lhs: {swift}, rhs: {swift}) -> Bool {{\n{INDENT}{}\n}}",
                    terms.join(" && ")
                ));
            }
            if derives.iter().any(|d| d == "Clone") {
                let args: Vec<String> = info
                    .fields
                    .iter()
                    .map(|f| format!("{}: {}", f.swift, self.deep_copy(&f.swift, &f.ty)))
                    .collect();
                body.push(String::new());
                body.push(format!(
                    "/// `clone()`: a copy of the whole structure.\nfunc clone() -> {swift} {{\n{INDENT}{swift}({})\n}}",
                    args.join(", ")
                ));
            }
        }
        // Swift synthesizes no `==` over arrays of tuples: written out.
        if conformances.iter().any(|c| c == "Equatable")
            && info.fields.iter().any(|f| has_tuple_collection(&f.ty))
        {
            let terms: Vec<String> = info
                .fields
                .iter()
                .map(|f| match &f.ty {
                    Ty::Vec(t) if matches!(**t, Ty::Tuple(_)) => format!(
                        "lhs.{0}.count == rhs.{0}.count && zip(lhs.{0}, rhs.{0}).allSatisfy {{ $0 == $1 }}",
                        f.swift
                    ),
                    // Nor over optional tuples.
                    Ty::Option(t) if matches!(**t, Ty::Tuple(_)) => format!(
                        "(lhs.{0} == nil && rhs.{0} == nil || lhs.{0} != nil && rhs.{0} != nil && lhs.{0}! == rhs.{0}!)",
                        f.swift
                    ),
                    _ => format!("lhs.{0} == rhs.{0}", f.swift),
                })
                .collect();
            body.push(String::new());
            body.push(format!(
                "static func == (lhs: {swift}, rhs: {swift}) -> Bool {{\n{INDENT}{}\n}}",
                terms.join(" && ")
            ));
        }
        if info.kind == TypeKind::Observable || info.recursive {
            let params: Vec<String> = info
                .fields
                .iter()
                .filter(|f| f.role != FieldRole::Ignored)
                .map(|f| {
                    let optional = info.recursive && matches!(f.ty, Ty::Option(_));
                    let default = self.field_default(f, derive_default || optional);
                    format!("{}: {}{default}", f.swift, f.ty.swift_lossy())
                })
                .collect();
            let assigns: Vec<String> = info
                .fields
                .iter()
                .filter(|f| f.role != FieldRole::Ignored)
                .map(|f| format!("{INDENT}self.{0} = {0}", f.swift))
                .collect();
            body.push(String::new());
            body.push(format!(
                "init({}) {{\n{}\n}}",
                params.join(", "),
                assigns.join("\n")
            ));
        }
        let less = (derives.iter().any(|d| d == "PartialOrd" || d == "Ord")
            && info.kind != TypeKind::Observable)
            .then(|| {
                lexicographic_less(
                    &swift,
                    &info
                        .fields
                        .iter()
                        .map(|f| f.swift.clone())
                        .collect::<Vec<_>>(),
                )
            });
        if let Some(less) = &less {
            if conditional.is_empty() {
                body.push(String::new());
                body.push(less.clone());
            }
        }

        // The body of a view or an app, from its `impl View` / `impl App`.
        if matches!(info.kind, TypeKind::View | TypeKind::App) {
            if let Some(impls) = impls {
                for imp in impls {
                    let tn = impl_trait_name(imp);
                    if matches!(tn.as_deref(), Some("View" | "App")) {
                        let self_ty = self.ty(&imp.self_ty);
                        self.imp = Some(ImplCx {
                            rust: name.clone(),
                            swift: swift.clone(),
                            self_ty,
                            kind: info.kind,
                            self_rename: None,
                        });
                        for it in &imp.items {
                            if let ImplItem::Fn(f) = it {
                                if f.sig.ident == "body" {
                                    if !body.is_empty() {
                                        body.push(String::new());
                                    }
                                    body.push(self.view_body(f, tn.as_deref() == Some("App")));
                                }
                            }
                        }
                        self.imp = None;
                    }
                }
            }
        }

        let conf = if conformances.is_empty() {
            String::new()
        } else {
            format!(": {}", conformances.join(", "))
        };
        let closing = self.comments.take_between(prev, s.span().end().line);
        body.extend(closing);
        self.generics.clear();
        let header = format!(
            "{}{prefix}{keyword} {swift}{generics}{conf}{where_clause}",
            docs(&s.attrs)
        );
        let mut out = if body.is_empty() {
            format!("{header} {{}}")
        } else {
            format!("{header} {{\n{}\n}}", indent_lines(&body.join("\n")))
        };
        out.push_str(&conditional_conformances(
            &swift,
            &info.generics,
            &conditional,
            less.as_deref(),
        ));
        out
    }

    /// Whether `ty` is a `#[derive(Observable)]` class.
    /// Whether the type is a struct of the crate translated as a class (recursive).
    pub fn is_recursive_class(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Named(n, _) if self.syms.structs.values().any(|s| &s.swift == n && s.recursive))
    }

    /// The expression copying a field of a recursive class in `clone()`.
    fn deep_copy(&self, field: &str, ty: &Ty) -> String {
        match ty {
            t if self.is_recursive_class(t) => format!("{field}.clone()"),
            Ty::Option(t) if self.is_recursive_class(t) => format!("{field}?.clone()"),
            Ty::Vec(t) if self.is_recursive_class(t) => format!("{field}.map {{ $0.clone() }}"),
            _ => field.to_string(),
        }
    }

    pub fn is_observable(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Named(n, _) if self.syms.structs.values().any(|s| &s.swift == n && s.kind == TypeKind::Observable))
    }

    /// ` = default` for a field, when the type has a default and it is wanted.
    fn field_default(&self, field: &crate::symbols::FieldInfo, wanted: bool) -> String {
        if !wanted {
            return String::new();
        }
        field
            .ty
            .default_value()
            .map(|v| format!(" = {v}"))
            .unwrap_or_default()
    }

    /// Protocol conformances from `#[derive(…)]`.
    /// `==`, `hash(into:)` and `<` for an enum with payloads, when Swift cannot synthesize
    /// them: arrays of tuples defeat `==` and `hash`, and `<` is never synthesized. Like
    /// Rust's derives, `<` orders by variant first, then by the payload, field by field.
    fn payload_conformances(&self, info: &EnumInfo, conformances: &[String]) -> Vec<String> {
        let payloads: Vec<(String, Vec<Ty>)> = info
            .variants
            .iter()
            .map(|v| {
                let tys = match &v.shape {
                    VariantShape::Tuple(tys) => tys.clone(),
                    VariantShape::Struct(fs) => fs.iter().map(|(_, t)| t.clone()).collect(),
                    VariantShape::Unit => Vec::new(),
                };
                (v.swift.clone(), tys)
            })
            .collect();
        let awkward = payloads
            .iter()
            .any(|(_, tys)| tys.iter().any(has_tuple_collection));
        let has = |c: &str| conformances.iter().any(|x| x == c);
        let binds = |prefix: &str, n: usize| -> String {
            (0..n)
                .map(|i| format!("let {prefix}{i}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut out = Vec::new();
        if awkward && (has("Equatable") || has("Hashable")) {
            let mut arms = Vec::new();
            for (case, tys) in &payloads {
                if tys.is_empty() {
                    arms.push(format!("case (.{case}, .{case}): return true"));
                    continue;
                }
                let terms: Vec<String> = tys
                    .iter()
                    .enumerate()
                    .map(|(i, t)| equality_term(&format!("a{i}"), &format!("b{i}"), t))
                    .collect();
                arms.push(format!(
                    "case (.{case}({}), .{case}({})): return {}",
                    binds("a", tys.len()),
                    binds("b", tys.len()),
                    terms.join(" && ")
                ));
            }
            arms.push("default: return false".into());
            out.push(String::new());
            out.push(format!(
                "static func == (lhs: {0}, rhs: {0}) -> Bool {{\n{INDENT}switch (lhs, rhs) {{\n{1}\n{INDENT}}}\n}}",
                info.swift,
                indent_lines(&arms.join("\n"))
            ));
        }
        if awkward && has("Hashable") {
            let mut arms = Vec::new();
            for (i, (case, tys)) in payloads.iter().enumerate() {
                let mut lines = vec![format!("hasher.combine({i})")];
                for (j, t) in tys.iter().enumerate() {
                    lines.push(hash_term(&format!("a{j}"), t));
                }
                let pattern = if tys.is_empty() {
                    format!(".{case}")
                } else {
                    format!(".{case}({})", binds("a", tys.len()))
                };
                arms.push(format!(
                    "case {pattern}:\n{}",
                    indent_lines(&lines.join("\n"))
                ));
            }
            out.push(String::new());
            out.push(format!(
                "func hash(into hasher: inout Hasher) {{\n{INDENT}switch self {{\n{}\n{INDENT}}}\n}}",
                indent_lines(&arms.join("\n"))
            ));
        }
        if has("Comparable") {
            let mut arms = Vec::new();
            for (case, tys) in &payloads {
                if tys.is_empty() {
                    continue;
                }
                let mut lines = Vec::new();
                for (i, t) in tys.iter().enumerate() {
                    let (a, b) = (format!("a{i}"), format!("b{i}"));
                    if i + 1 == tys.len() {
                        lines.push(format!("return {}", less_term(&a, &b, t)));
                    } else {
                        lines.push(format!(
                            "if !({}) {{ return {} }}",
                            equality_term(&a, &b, t),
                            less_term(&a, &b, t)
                        ));
                    }
                }
                arms.push(format!(
                    "case (.{case}({}), .{case}({})):\n{}",
                    binds("a", tys.len()),
                    binds("b", tys.len()),
                    indent_lines(&lines.join("\n"))
                ));
            }
            arms.push("default: return lhs.rsVariantIndex < rhs.rsVariantIndex".into());
            out.push(String::new());
            out.push(format!(
                "static func < (lhs: {0}, rhs: {0}) -> Bool {{\n{INDENT}switch (lhs, rhs) {{\n{1}\n{INDENT}}}\n}}",
                info.swift,
                indent_lines(&arms.join("\n"))
            ));
            let ordinals: Vec<String> = payloads
                .iter()
                .enumerate()
                .map(|(i, (case, _))| format!("case .{case}: {i}"))
                .collect();
            out.push(String::new());
            out.push(format!(
                "private var rsVariantIndex: Int {{\n{INDENT}switch self {{\n{}\n{INDENT}}}\n}}",
                indent_lines(&ordinals.join("\n"))
            ));
        }
        out
    }

    /// Whether Swift can synthesize `Hashable` for values of this type.
    fn hashable(&self, t: &Ty) -> bool {
        match t {
            Ty::Int(_) | Ty::Float(_) | Ty::Bool | Ty::Char | Ty::Str | Ty::Unit => true,
            Ty::Vec(i) | Ty::Set(i, _) | Ty::Option(i) | Ty::Range(i) => self.hashable(i),
            Ty::Map(k, v, _) => self.hashable(k) && self.hashable(v),
            Ty::Tuple(ts) => ts.iter().all(|t| self.hashable(t)),
            Ty::Named(n, args) if n == "RsReverse" || n == "RsTuple1" => {
                args.iter().all(|a| self.hashable(a))
            }
            Ty::Named(n, _) => {
                let Some(rust) = self.rust_name_of_swift(n) else {
                    return false;
                };
                let derives = match self.syms.structs.get(&rust) {
                    Some(s) => &s.derives,
                    None => match self.syms.enums.get(&rust) {
                        Some(e) => &e.derives,
                        None => return false,
                    },
                };
                derives.iter().any(|d| d == "Hash" || d == "Ord")
            }
            _ => false,
        }
    }

    fn derive_conformances(&mut self, derives: &[String], span: Span) -> Vec<String> {
        let mut out = Vec::new();
        for d in derives {
            let p = match d.as_str() {
                "PartialEq" | "Eq" => "Equatable",
                "Hash" => "Hashable",
                "PartialOrd" | "Ord" => "Comparable",
                "Serialize" | "Deserialize" => "Codable",
                "Debug" | "Clone" | "Copy" | "Default" | "View" | "Observable" => continue,
                other => {
                    self.warn(span, format!("#[derive({other})] is not translated"));
                    continue;
                }
            };
            if !out.iter().any(|o| o == p) {
                out.push(p.to_string());
            }
        }
        if out.iter().any(|o| o == "Hashable") {
            out.retain(|o| o != "Equatable");
        }
        out
    }

    // ----- Enums -----

    fn item_enum(&mut self, e: &syn::ItemEnum) -> String {
        let name = e.ident.to_string();
        let info = self.syms.enums.get(&name).cloned().expect("collected enum");
        let generics = self.generic_params(&e.generics);
        self.generics = info.generics.clone();
        let derives = derives(&e.attrs);
        let mut conformances = self.derive_conformances(&derives, e.span());
        let all_unit = e.variants.iter().all(|v| matches!(v.fields, Fields::Unit));
        let has_discriminant = e.variants.iter().any(|v| v.discriminant.is_some());
        if has_discriminant {
            conformances.insert(0, "Int".into());
        }
        if self.syms.errors.contains(&info.swift) && !conformances.iter().any(|c| c == "Error") {
            conformances.push("Error".into());
        }
        if all_unit && !has_discriminant {
            // Swift synthesizes Equatable and Hashable for enums without payloads.
            conformances.retain(|c| c != "Equatable");
            if derives.iter().any(|d| d == "Hash") || derives.iter().any(|d| d == "PartialEq") {
                conformances.retain(|c| c != "Hashable");
                conformances.push("Hashable".into());
            }
        }
        dedup(&mut conformances);
        let conditional: Vec<String> = if info.generics.is_empty() {
            Vec::new()
        } else {
            let moved: Vec<String> = conformances
                .iter()
                .filter(|c| is_synthesized(c))
                .cloned()
                .collect();
            conformances.retain(|c| !is_synthesized(c));
            moved
        };
        let recursive = e
            .variants
            .iter()
            .any(|v| v.fields.iter().any(|f| mentions_type(&f.ty, &name)));
        let mut body = Vec::new();
        let mut prev = e.span().start().line;
        for v in &e.variants {
            let leading = self
                .comments
                .take_between(prev, first_line(&v.attrs, v.span()));
            prev = v.span().end().line;
            body.extend(leading);
            let case = names::case_name(&v.ident.to_string());
            let payload = match &v.fields {
                Fields::Unit => String::new(),
                Fields::Unnamed(f) => {
                    let tys: Vec<String> = f
                        .unnamed
                        .iter()
                        .map(|f| {
                            let t = self.ty(&f.ty);
                            self.swift_ty(&t, f.ty.span())
                        })
                        .collect();
                    format!("({})", tys.join(", "))
                }
                Fields::Named(f) => {
                    let tys: Vec<String> = f
                        .named
                        .iter()
                        .map(|f| {
                            let t = self.ty(&f.ty);
                            format!(
                                "{}: {}",
                                names::lower_camel(&f.ident.as_ref().expect("named").to_string()),
                                self.swift_ty(&t, f.ty.span())
                            )
                        })
                        .collect();
                    format!("({})", tys.join(", "))
                }
            };
            let discriminant = match &v.discriminant {
                Some((_, expr)) => {
                    let d = self.expr(expr);
                    format!(" = {}", d.code)
                }
                None => String::new(),
            };
            let trailing = self
                .comments
                .take_trailing(v.span().end().line)
                .map(|c| format!(" {c}"))
                .unwrap_or_default();
            body.push(format!(
                "{}case {case}{payload}{discriminant}{trailing}",
                docs(&v.attrs)
            ));
        }
        if derives.iter().any(|d| d == "Default") {
            if let Some(v) = e.variants.iter().find(|v| has_attr(&v.attrs, "default")) {
                body.push(String::new());
                body.push(format!(
                    "init() {{\n{INDENT}self = .{}\n}}",
                    names::case_name(&v.ident.to_string())
                ));
            }
        }
        // Swift synthesizes no `<` for enums with raw values: by discriminant, like Rust.
        if has_discriminant && conformances.iter().any(|c| c == "Comparable") {
            body.push(String::new());
            body.push(format!(
                "static func < (lhs: {0}, rhs: {0}) -> Bool {{\n{INDENT}lhs.rawValue < rhs.rawValue\n}}",
                info.swift
            ));
        }
        if !all_unit {
            body.extend(self.payload_conformances(&info, &conformances));
        }
        let closing = self.comments.take_between(prev, e.span().end().line);
        body.extend(closing);
        self.generics.clear();
        let conf = if conformances.is_empty() {
            String::new()
        } else {
            format!(": {}", conformances.join(", "))
        };
        let mut out = format!(
            "{}{}enum {}{generics}{conf}{} {{\n{}\n}}",
            docs(&e.attrs),
            if recursive { "indirect " } else { "" },
            info.swift,
            self.where_clause(&e.generics),
            indent_lines(&body.join("\n"))
        );
        out.push_str(&conditional_conformances(
            &info.swift,
            &info.generics,
            &conditional,
            None,
        ));
        // With a `Display` implementation, `{:?}` needs the Rust names of the cases.
        if derives.iter().any(|d| d == "Debug") && self.syms.display.contains(&name) {
            self.helper("debug");
            out.push_str(&debug_description(&info));
        }
        out
    }

    // ----- Traits -----

    fn item_trait(&mut self, t: &syn::ItemTrait) -> String {
        let name = names::upper_camel(&t.ident.to_string());
        let mut supers = self.bounds(&t.supertraits);
        supers.retain(|s| s != "Copy");
        let conf = if supers.is_empty() {
            String::new()
        } else {
            format!(": {}", supers.join(", "))
        };
        let self_ty = Ty::Named("Self".into(), vec![]);
        self.imp = Some(ImplCx {
            rust: t.ident.to_string(),
            swift: name.clone(),
            self_ty,
            kind: TypeKind::Plain,
            self_rename: None,
        });
        let mut reqs = Vec::new();
        let mut defaults = Vec::new();
        for item in &t.items {
            match item {
                TraitItem::Fn(f) => {
                    self.push_scope();
                    let sig = self.signature(&f.sig, Place::Protocol);
                    self.pop_scope();
                    reqs.push(format!("{}{}", docs(&f.attrs), sig.header));
                    if let Some(block) = &f.default {
                        defaults.push(self.function(
                            &f.attrs,
                            &f.sig,
                            block,
                            Place::Method {
                                mutating_allowed: true,
                            },
                        ));
                    }
                }
                TraitItem::Type(ty) => {
                    let bounds = self.bounds(&ty.bounds);
                    if bounds.is_empty() {
                        reqs.push(format!("associatedtype {}", ty.ident));
                    } else {
                        reqs.push(format!(
                            "associatedtype {}: {}",
                            ty.ident,
                            bounds.join(" & ")
                        ));
                    }
                }
                TraitItem::Const(c) => {
                    let ty = self.ty(&c.ty);
                    let swift = self.swift_ty(&ty, c.ty.span());
                    reqs.push(format!(
                        "static var {}: {swift} {{ get }}",
                        names::constant(&c.ident.to_string())
                    ));
                }
                other => {
                    self.error(other.span(), "this trait item is not supported");
                }
            }
        }
        self.imp = None;
        let mut out = format!(
            "{}protocol {name}{conf} {{\n{}\n}}",
            docs(&t.attrs),
            indent_lines(&reqs.join("\n"))
        );
        if !defaults.is_empty() {
            out.push_str(&format!(
                "\n\nextension {name} {{\n{}\n}}",
                indent_lines(&defaults.join("\n\n"))
            ));
        }
        out
    }

    // ----- Impls -----

    fn item_impl(&mut self, imp: &syn::ItemImpl) -> String {
        let Some(rust_name) = crate::symbols::type_name(&imp.self_ty) else {
            return self.unsupported(imp.self_ty.span(), "impl for this type");
        };
        let self_ty = self.ty(&imp.self_ty);
        let trait_name = impl_trait_name(imp);
        let info = self.syms.structs.get(&rust_name);
        let kind = info.map_or(TypeKind::Plain, |s| s.kind);
        // A recursive struct is a class: its methods are never `mutating`.
        let recursive = info.is_some_and(|s| s.recursive);
        let swift = match &self_ty {
            Ty::Named(n, _) => n.clone(),
            Ty::Vec(_) => "Array".into(),
            Ty::Map(..) => "Dictionary".into(),
            Ty::Set(..) => "Set".into(),
            Ty::Option(_) => "Optional".into(),
            other => other.swift_lossy(),
        };
        self.generics = crate::symbols::type_params(&imp.generics);
        self.imp = Some(ImplCx {
            rust: rust_name.clone(),
            swift: swift.clone(),
            self_ty: self_ty.clone(),
            kind,
            self_rename: None,
        });
        let mut members = Vec::new();
        let mut conformance: Option<String> = None;
        let mut where_parts = Vec::new();
        // `impl<T: Display> Wrapper<T>` → `extension Wrapper where T: CustomStringConvertible`.
        for p in &imp.generics.params {
            if let GenericParam::Type(t) = p {
                for b in self.bounds(&t.bounds) {
                    where_parts.push(format!("{}: {b}", t.ident));
                }
            }
        }
        let wc = self.where_clause(&imp.generics);
        if let Some(w) = wc.strip_prefix(" where ") {
            where_parts.push(w.to_string());
        }
        // `impl Summary for Vec<i32>` → `extension Array: Summary where Element == Int`.
        match &self_ty {
            Ty::Vec(t)
                if t.is_complete()
                    && !matches!(**t, Ty::Named(ref n, _) if self.generics.contains(n)) =>
            {
                where_parts.push(format!("Element == {}", t.swift_lossy()));
            }
            _ => {}
        }
        let mut prev = imp.span().start().line;
        match trait_name.as_deref() {
            Some("Display") => {
                conformance = Some("CustomStringConvertible".into());
                for it in &imp.items {
                    if let ImplItem::Fn(f) = it {
                        members.push(self.fmt_impl(f, "description"));
                    }
                }
            }
            Some("Debug") => {
                conformance = Some("CustomDebugStringConvertible".into());
                for it in &imp.items {
                    if let ImplItem::Fn(f) = it {
                        members.push(self.fmt_impl(f, "debugDescription"));
                    }
                }
            }
            Some("Error") => conformance = Some("Error".into()),
            Some("Iterator") => {
                conformance = Some("Sequence, IteratorProtocol".into());
                for it in &imp.items {
                    if let ImplItem::Fn(f) = it {
                        members.push(self.function(
                            &f.attrs,
                            &f.sig,
                            &f.block,
                            Place::Method {
                                mutating_allowed: true,
                            },
                        ));
                    }
                }
            }
            Some(op) if operator_of(op).is_some() => {
                for it in &imp.items {
                    if let ImplItem::Fn(f) = it {
                        members.push(self.operator_fn(op, f));
                    }
                }
            }
            Some("PartialEq") => {
                conformance = Some("Equatable".into());
                for it in &imp.items {
                    if let ImplItem::Fn(f) = it {
                        members.push(self.function(
                            &f.attrs,
                            &f.sig,
                            &f.block,
                            Place::Method {
                                mutating_allowed: false,
                            },
                        ));
                        if f.sig.ident == "eq" {
                            members.push(format!("static func == (lhs: {swift}, rhs: {swift}) -> Bool {{\n{INDENT}lhs.eq(rhs)\n}}"));
                        }
                    }
                }
            }
            Some("Ord") | Some("PartialOrd") => {
                for it in &imp.items {
                    if let ImplItem::Fn(f) = it {
                        members.push(self.function(
                            &f.attrs,
                            &f.sig,
                            &f.block,
                            Place::Method {
                                mutating_allowed: false,
                            },
                        ));
                        if f.sig.ident == "cmp" {
                            conformance = Some("Comparable".into());
                            members.push(format!("static func < (lhs: {swift}, rhs: {swift}) -> Bool {{\n{INDENT}lhs.cmp(rhs) == .less\n}}"));
                        }
                        if f.sig.ident == "partial_cmp"
                            && trait_name.as_deref() == Some("PartialOrd")
                        {
                            let ord = self
                                .syms
                                .methods
                                .get(&rust_name)
                                .is_some_and(|m| m.contains_key("cmp"));
                            if !ord {
                                conformance = Some("Comparable".into());
                                members.push(format!("static func < (lhs: {swift}, rhs: {swift}) -> Bool {{\n{INDENT}lhs.partialCmp(rhs) == .less\n}}"));
                            }
                        }
                    }
                }
                self.helper("Ordering");
            }
            Some("Hash") | Some("Clone") | Some("Copy") | Some("Eq") | Some("Send")
            | Some("Sync") => {
                self.imp = None;
                self.generics.clear();
                return String::new();
            }
            Some("Drop") => {
                self.warn(
                    imp.span(),
                    "Drop is not translated: Swift structs have no destructor",
                );
                self.imp = None;
                self.generics.clear();
                return String::new();
            }
            Some("Default") | Some("From") | None | Some(_) => {
                if let Some(t) = &trait_name {
                    // Conversions have no protocol: their functions are called directly.
                    if !matches!(
                        t.as_str(),
                        "Default" | "From" | "Into" | "FromStr" | "TryFrom" | "TryInto"
                    ) {
                        conformance =
                            Some(protocol_name(t).unwrap_or_else(|| names::upper_camel(t)));
                    }
                }
                for it in &imp.items {
                    let start = it.span().start().line;
                    let leading = self.comments.take_between(prev, start);
                    prev = it.span().end().line;
                    let mut code = match it {
                        ImplItem::Fn(f) => {
                            let sig = self
                                .syms
                                .methods
                                .get(&rust_name)
                                .and_then(|m| m.get(&f.sig.ident.to_string()))
                                .cloned();
                            if sig.as_ref().is_some_and(|s| s.is_init) {
                                self.init_fn(f)
                            } else {
                                self.function(
                                    &f.attrs,
                                    &f.sig,
                                    &f.block,
                                    Place::Method {
                                        mutating_allowed: kind == TypeKind::Plain && !recursive,
                                    },
                                )
                            }
                        }
                        ImplItem::Const(c) => {
                            let ty = self.ty(&c.ty);
                            self.push_scope();
                            let v = self.expr_with(&c.expr, &ty);
                            let v = self.settle(v);
                            self.pop_scope();
                            // `[.mon, .tue]` needs the type to resolve its cases.
                            let leading_dots = v.code.starts_with('.') || v.code.contains("[.");
                            let annot = if natural_type_matches(&v.ty, &ty) && !leading_dots {
                                String::new()
                            } else {
                                format!(": {}", self.swift_ty(&ty, c.ty.span()))
                            };
                            format!(
                                "{}static let {}{annot} = {}",
                                docs(&c.attrs),
                                names::constant(&c.ident.to_string()),
                                v.code
                            )
                        }
                        // The error type of `FromStr` or `TryFrom` is resolved where it is used.
                        ImplItem::Type(t) if t.ident == "Err" || t.ident == "Error" => {
                            members.extend(leading);
                            continue;
                        }
                        ImplItem::Type(t) => {
                            let ty = self.ty(&t.ty);
                            format!(
                                "typealias {} = {}",
                                t.ident,
                                self.swift_ty(&ty, t.ty.span())
                            )
                        }
                        other => self.unsupported(other.span(), "this impl item"),
                    };
                    if !leading.is_empty() {
                        code = format!("{}\n{code}", leading.join("\n"));
                    }
                    members.push(code);
                }
            }
        }
        self.imp = None;
        self.generics.clear();
        let conf = conformance.map(|c| format!(": {c}")).unwrap_or_default();
        let wc = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" where {}", where_parts.join(", "))
        };
        let closing = self.comments.take_between(prev, imp.span().end().line);
        members.extend(closing);
        if members.is_empty() {
            format!("extension {swift}{conf}{wc} {{}}")
        } else {
            format!(
                "extension {swift}{conf}{wc} {{\n{}\n}}",
                indent_lines(&members.join("\n\n"))
            )
        }
    }

    /// `fn fmt(&self, f: &mut Formatter) -> fmt::Result` → `var description: String`.
    fn fmt_impl(&mut self, f: &syn::ImplItemFn, property: &str) -> String {
        let param = f.sig.inputs.iter().find_map(|a| match a {
            FnArg::Typed(p) => match &*p.pat {
                Pat::Ident(i) => Some(i.ident.to_string()),
                _ => None,
            },
            _ => None,
        });
        self.push_scope();
        self.fns.push(FnCx {
            ret: Ty::Unit,
            throws: None,
            init: false,
            closure: false,
            self_name: None,
        });
        let out_name = "out".to_string();
        if let Some(p) = &param {
            self.declare_alias(
                p,
                format!("\u{2}fmt:{out_name}"),
                Ty::Named("Formatter".into(), vec![]),
            );
        }
        let body = self.block_lines(&f.block, Sink::Discard);
        self.fns.pop();
        self.pop_scope();
        let body = self.resolve_slots(&body.join("\n"));
        // A single `write!`: the string itself.
        if let Some(value) = body.strip_prefix(&format!("{out_name} += ")) {
            if !body.contains('\n') {
                return format!(
                    "{}var {property}: String {{\n{INDENT}{value}\n}}",
                    docs(&f.attrs)
                );
            }
        }
        // A `match` whose arms each write once: a `switch` giving the string.
        if let Some(switch) = switch_of_writes(&body, &out_name) {
            return format!(
                "{}var {property}: String {{\n{}\n}}",
                docs(&f.attrs),
                indent_lines(&switch)
            );
        }
        let body = indent_lines(&body);
        format!(
            "{}var {property}: String {{\n{INDENT}var {out_name} = \"\"\n{body}\n{INDENT}return {out_name}\n}}",
            docs(&f.attrs)
        )
    }

    /// `impl Add for V { fn add(self, o: V) -> V }` → `static func + (lhs: V, rhs: V) -> V`.
    fn operator_fn(&mut self, trait_name: &str, f: &syn::ImplItemFn) -> String {
        let (op, assign, prefix) = operator_of(trait_name).expect("operator trait");
        let imp = self.imp.clone().expect("impl");
        let swift = imp.swift.clone();
        if f.sig.ident == "index" || f.sig.ident == "index_mut" {
            return self.subscript_fn(f);
        }
        self.push_scope();
        let mut params = Vec::new();
        for arg in &f.sig.inputs {
            if let FnArg::Typed(p) = arg {
                let ty = self.ty(&p.ty);
                if let Pat::Ident(i) = &*p.pat {
                    let swift_name = self.declare(&i.ident.to_string(), ty.clone());
                    params.push(format!(
                        "_ {swift_name}: {}",
                        self.swift_ty(&ty, p.ty.span())
                    ));
                }
            }
        }
        let ret = match &f.sig.output {
            ReturnType::Default => Ty::Unit,
            ReturnType::Type(_, t) => self.ty(t),
        };
        if let Some(i) = self.imp.as_mut() {
            i.self_rename = Some("lhs".into());
        }
        self.fns.push(FnCx {
            ret: ret.clone(),
            throws: None,
            init: false,
            closure: false,
            self_name: None,
        });
        let sink = if ret == Ty::Unit {
            Sink::Discard
        } else {
            Sink::Return
        };
        let body = self.block_lines(&f.block, sink);
        self.fns.pop();
        if let Some(i) = self.imp.as_mut() {
            i.self_rename = None;
        }
        self.pop_scope();
        let body = indent_lines(&self.resolve_slots(&body.join("\n")));
        let rhs = params.first().cloned().unwrap_or_default();
        let rhs = rhs.replacen("_ ", "", 1);
        if prefix {
            format!("static prefix func {op} (lhs: {swift}) -> {swift} {{\n{body}\n}}")
        } else if assign {
            format!("static func {op} (lhs: inout {swift}, {rhs}) {{\n{body}\n}}")
        } else {
            let ret_swift = self.swift_ty(&ret, f.sig.output.span());
            format!("static func {op} (lhs: {swift}, {rhs}) -> {ret_swift} {{\n{body}\n}}")
        }
    }

    fn subscript_fn(&mut self, f: &syn::ImplItemFn) -> String {
        self.push_scope();
        let mut params = Vec::new();
        let mut prologue = Vec::new();
        for arg in &f.sig.inputs {
            if let FnArg::Typed(p) = arg {
                let ty = self.ty(&p.ty);
                match &*p.pat {
                    Pat::Ident(i) => {
                        let name = self.declare(&i.ident.to_string(), ty.clone());
                        params.push(format!("{name}: {}", self.swift_ty(&ty, p.ty.span())));
                    }
                    // `fn index(&self, (r, c): (usize, usize))`.
                    pat => {
                        let name = self.fresh("index");
                        params.push(format!("_ {name}: {}", self.swift_ty(&ty, p.ty.span())));
                        let (lines, _) = self.let_pattern(pat, &name, &ty, false);
                        prologue.extend(lines);
                    }
                }
            }
        }
        let ret = match &f.sig.output {
            ReturnType::Default => Ty::Unit,
            ReturnType::Type(_, t) => self.ty(t),
        };
        self.fns.push(FnCx {
            ret: ret.clone(),
            throws: None,
            init: false,
            closure: false,
            self_name: None,
        });
        let mut body = prologue;
        body.extend(self.block_lines(&f.block, Sink::Return));
        self.fns.pop();
        self.pop_scope();
        let ret_swift = self.swift_ty(&ret, f.sig.output.span());
        let body = indent_lines(&self.resolve_slots(&body.join("\n")));
        format!(
            "subscript({}) -> {ret_swift} {{\n{body}\n}}",
            params.join(", ")
        )
    }

    // ----- Functions -----

    fn item_fn(&mut self, f: &syn::ItemFn) -> String {
        self.function(&f.attrs, &f.sig, &f.block, Place::Free)
    }

    /// A function or method with its body.
    pub fn function(
        &mut self,
        attrs: &[Attribute],
        sig: &syn::Signature,
        block: &syn::Block,
        place: Place,
    ) -> String {
        let outer_generics = self.generics.clone();
        self.generics
            .extend(crate::symbols::type_params(&sig.generics));
        self.push_scope();
        // Closure parameters the body stores or passes on must be `@escaping`.
        self.escaping_params = sig
            .inputs
            .iter()
            .filter_map(|a| match a {
                FnArg::Typed(p) => match &*p.pat {
                    Pat::Ident(id) if used_as_value(block, &id.ident.to_string()) => {
                        Some(id.ident.to_string())
                    }
                    _ => None,
                },
                FnArg::Receiver(_) => None,
            })
            .collect();
        let mut s = self.signature(sig, place);
        self.escaping_params.clear();
        let own_sig = self
            .imp
            .as_ref()
            .and_then(|i| self.syms.methods.get(&i.rust))
            .and_then(|ms| ms.get(&sig.ident.to_string()))
            .cloned();
        // `fn count(&self) -> usize` next to a field `count`: `var count: Int { … }`.
        if own_sig.as_ref().is_some_and(|m| m.property) {
            let ret = s.cx.ret.swift_lossy();
            s.header = format!("var {}: {ret}", names::lower_camel(&sig.ident.to_string()));
        }
        // `fn add(&mut self, …) -> &mut Self`: a builder, chained as statements by the caller.
        if own_sig.as_ref().is_some_and(|m| m.returns_self_mut) {
            s.cx.ret = Ty::Unit;
            s.header = s
                .header
                .rsplit_once(" -> ")
                .map_or(s.header.clone(), |(h, _)| h.to_string());
        }
        self.fns.push(s.cx.clone());
        let mut body = s.prologue.clone();
        let sink = if s.cx.ret == Ty::Unit || s.cx.ret == Ty::Never {
            Sink::Discard
        } else if s.view {
            Sink::Yield
        } else {
            Sink::Return
        };
        let view = s.view;
        if view {
            self.used_ui = true;
        }
        body.extend(self.block_lines(block, sink));
        self.fns.pop();
        self.pop_scope();
        self.generics = outer_generics;
        let deprecated = if has_attr(attrs, "deprecated") {
            "@available(*, deprecated)\n"
        } else {
            ""
        };
        let builder = if view { "@ViewBuilder\n" } else { "" };
        // `Ok(())` at the end of a function: nothing to write.
        if body.len() > 1 && body.last().is_some_and(|l| l == "return") {
            body.pop();
        }
        // A body that only returns a value: Swift's implicit return.
        if body.len() == 1 {
            if let Some(value) = body[0].strip_prefix("return ") {
                body[0] = value.to_string();
            }
        }
        let body = self.resolve_slots(&body.join("\n"));
        format!(
            "{}{deprecated}{builder}{} {{\n{}\n}}",
            docs(attrs),
            s.header,
            indent_lines(&body)
        )
    }

    /// `fn new(…) -> Self` (or `default`, `from`) as an initializer.
    fn init_fn(&mut self, f: &syn::ImplItemFn) -> String {
        self.push_scope();
        let mut s = self.signature(
            &f.sig,
            Place::Method {
                mutating_allowed: false,
            },
        );
        let (failable, throws) = match &s.cx.ret {
            Ty::Option(_) => (true, None),
            Ty::Result(_, e) => (false, Some((**e).clone())),
            _ => (false, None),
        };
        s.cx.init = true;
        s.cx.throws = throws.clone();
        self.fns.push(s.cx.clone());
        let mut body = s.prologue.clone();
        body.extend(self.block_lines(&f.block, Sink::InitSelf));
        self.fns.pop();
        self.pop_scope();
        let throws_clause = match &throws {
            Some(Ty::Dyn(_)) | Some(Ty::Unknown) => " throws".to_string(),
            Some(e) => format!(" throws({})", e.swift_lossy()),
            None => String::new(),
        };
        let body = self.resolve_slots(&body.join("\n"));
        format!(
            "{}init{}({}){throws_clause} {{\n{}\n}}",
            docs(&f.attrs),
            if failable { "?" } else { "" },
            s.params.join(", "),
            indent_lines(&body)
        )
    }

    /// The `var body: some View` of a view, or `var body: some Scene` of an app.
    fn view_body(&mut self, f: &syn::ImplItemFn, app: bool) -> String {
        self.push_scope();
        self.fns.push(FnCx {
            ret: Ty::View,
            throws: None,
            init: false,
            closure: false,
            self_name: None,
        });
        let body = self.block_lines(&f.block, Sink::Yield);
        self.fns.pop();
        self.pop_scope();
        let body = self.resolve_slots(&body.join("\n"));
        format!(
            "{}var body: some {} {{\n{}\n}}",
            docs(&f.attrs),
            if app { "Scene" } else { "View" },
            indent_lines(&body)
        )
    }

    /// `#[preview] fn preview() -> impl View { … }` → `#Preview { … }`.
    fn preview(&mut self, f: &syn::ItemFn) -> String {
        self.used_ui = true;
        self.push_scope();
        self.fns.push(FnCx {
            ret: Ty::View,
            throws: None,
            init: false,
            closure: false,
            self_name: None,
        });
        let body = self.block_lines(&f.block, Sink::Yield);
        self.fns.pop();
        self.pop_scope();
        let body = self.resolve_slots(&body.join("\n"));
        format!("#Preview {{\n{}\n}}", indent_lines(&body))
    }

    /// Translates a signature, declaring its parameters in the current scope.
    pub fn signature(&mut self, sig: &syn::Signature, place: Place) -> Signature {
        let name = sig.ident.to_string();
        let self_ty = self.imp.as_ref().map(|i| i.self_ty.clone());
        self.closure_generics = self.syms.closure_generics(&sig.generics, self_ty.as_ref());
        let generics = self.generic_params(&sig.generics);
        let where_clause = self.where_clause(&sig.generics);
        let mut params = Vec::new();
        let mut prologue = Vec::new();
        let mut self_kind = SelfKind::None;
        let mut self_name = None;
        for (i, arg) in sig.inputs.iter().enumerate() {
            match arg {
                FnArg::Receiver(r) => {
                    self_kind = match (&r.reference, r.mutability.is_some()) {
                        (Some(_), true) => SelfKind::RefMut,
                        (Some(_), false) => SelfKind::Ref,
                        (None, true) => {
                            // `mut self`: a mutable copy.
                            let n = self.declare("self", Ty::Unknown);
                            prologue.push(format!("var {n} = self"));
                            self_name = Some(n);
                            SelfKind::Value
                        }
                        (None, false) => SelfKind::Value,
                    };
                }
                FnArg::Typed(p) => {
                    if matches!(&*p.ty, Type::Ptr(_)) {
                        self.error(p.ty.span(), "raw pointers have no Swift equivalent here");
                    }
                    let ty = self.ty(&p.ty);
                    let inout = matches!(&*p.ty, Type::Reference(r) if r.mutability.is_some())
                        && !matches!(ty, Ty::Fn(..) | Ty::View);
                    let returns_closure = match &sig.output {
                        ReturnType::Type(_, t) => matches!(self.ty(t), Ty::Fn(..)),
                        ReturnType::Default => false,
                    };
                    // A closure stored somewhere (`self.handlers.push(handler)`) escapes.
                    let stored = matches!(ty, Ty::Fn(..))
                        && matches!(&*p.pat, Pat::Ident(id) if self.escaping_params.contains(&id.ident.to_string()));
                    let mut swift_ty = self.param_ty(&ty, &p.ty, returns_closure || stored);
                    if inout {
                        // `&mut dyn Visitor`: an existential cannot be passed `inout` from a
                        // concrete variable; an opaque type (a generic) can.
                        if matches!(ty, Ty::Dyn(_)) {
                            swift_ty = swift_ty.replacen("any ", "some ", 1);
                        }
                        swift_ty = format!("inout {swift_ty}");
                    }
                    match &*p.pat {
                        Pat::Ident(id) => {
                            let rust = id.ident.to_string();
                            let n = self.declare(&rust, ty.clone());
                            params.push(format!("_ {n}: {swift_ty}"));
                            if id.mutability.is_some() && !inout {
                                prologue.push(format!("var {n} = {n}"));
                            }
                        }
                        Pat::Wild(_) => params.push(format!("_ _: {swift_ty}")),
                        pat => {
                            let n = format!("p{i}");
                            params.push(format!("_ {n}: {swift_ty}"));
                            let (lines, _) = self.let_pattern(pat, &n, &ty, false);
                            prologue.extend(lines);
                        }
                    }
                }
            }
        }
        let ret = match &sig.output {
            ReturnType::Default => Ty::Unit,
            ReturnType::Type(_, t) => self.ty(t),
        };
        if let ReturnType::Type(_, t) = &sig.output {
            let self_ty = self.imp.as_ref().map(|i| i.self_ty.clone());
            if has_mut_reference(t)
                && !self_ty.is_some_and(|st| crate::symbols::returns_self_ty(&ret, &st))
            {
                self.error(
                    t.span(),
                    "returning a mutable reference is not translated (Swift copies values): return an index, or change the value inside the method",
                );
            }
        }
        let mut throws = None;
        let mut view = false;
        let ret_swift = match &ret {
            Ty::Unit => String::new(),
            Ty::Result(ok, err) => {
                throws = Some((**err).clone());
                match &**ok {
                    Ty::Unit => String::new(),
                    ok => format!(" -> {}", self.swift_ty(ok, sig.output.span())),
                }
            }
            Ty::View => {
                view = true;
                " -> some View".to_string()
            }
            Ty::Iter(item) => format!(" -> [{}]", self.swift_ty(item, sig.output.span())),
            t => format!(" -> {}", self.swift_ty(t, sig.output.span())),
        };
        let throws_clause = match &throws {
            None => "",
            Some(Ty::Dyn(_)) | Some(Ty::Unknown) => " throws",
            Some(_) => "",
        };
        let typed_throws = match &throws {
            Some(e @ Ty::Named(..)) | Some(e @ Ty::Str) => format!(" throws({})", e.swift_lossy()),
            Some(Ty::Int(k)) => format!(" throws({})", k.swift()),
            _ => String::new(),
        };
        if matches!(throws, Some(Ty::Str)) {
            self.helper("StringError");
        }
        if let Some(e) = &throws {
            self.note_support_types(e);
        }
        let asyncness = if sig.asyncness.is_some() {
            " async"
        } else {
            ""
        };
        let imp_kind = self.imp.as_ref().map(|i| i.kind);
        let in_value_type = matches!(imp_kind, Some(TypeKind::Plain));
        let mut modifiers = String::new();
        let allowed = match place {
            Place::Free => None,
            Place::Method { mutating_allowed } => Some(mutating_allowed),
            Place::Protocol => Some(true),
        };
        if let Some(allowed) = allowed {
            if self_kind == SelfKind::None {
                modifiers.push_str("static ");
            } else if self_kind == SelfKind::RefMut
                && allowed
                && (in_value_type || matches!(place, Place::Protocol))
            {
                modifiers.push_str("mutating ");
            }
        }
        let swift_name = match name.as_str() {
            "main" if matches!(place, Place::Free) => "main".to_string(),
            n => names::lower_camel(n),
        };
        let header = format!(
            "{modifiers}func {swift_name}{generics}({}){asyncness}{throws_clause}{typed_throws}{ret_swift}{where_clause}",
            params.join(", ")
        );
        Signature {
            header,
            params,
            prologue,
            view,
            cx: FnCx {
                ret,
                throws,
                init: false,
                closure: false,
                self_name,
            },
        }
    }

    /// The Swift type of a parameter: closures stored by the function are `@escaping`.
    fn param_ty(&mut self, ty: &Ty, syn_ty: &Type, returns_closure: bool) -> String {
        let s = self.swift_ty(ty, syn_ty.span());
        let boxed = quote::quote!(#syn_ty).to_string().replace(' ', "");
        let stored =
            boxed.starts_with("Box<") || boxed.starts_with("Rc<") || boxed.starts_with("Arc<");
        // A closure kept by the function (boxed, or captured by the closure it returns).
        if matches!(ty, Ty::Fn(..)) && (stored || returns_closure) {
            format!("@escaping {s}")
        } else {
            s
        }
    }
}

/// Where a function is declared.
#[derive(Clone, Copy, Debug)]
pub enum Place {
    Free,
    Method {
        mutating_allowed: bool,
    },
    /// A protocol requirement (no body).
    Protocol,
}

pub struct Signature {
    pub header: String,
    pub params: Vec<String>,
    /// Statements at the start of the body (`var x = x` for `mut x` parameters).
    pub prologue: Vec<String>,
    pub view: bool,
    pub cx: FnCx,
}

/// The Swift protocol for a Rust trait of the standard library.
pub fn protocol_name(name: &str) -> Option<String> {
    Some(
        match name {
            "Display" | "ToString" => "CustomStringConvertible",
            "Debug" => "CustomDebugStringConvertible",
            "PartialEq" | "Eq" => "Equatable",
            "PartialOrd" | "Ord" => "Comparable",
            "Hash" => "Hashable",
            "Error" => "Error",
            "Iterator" => "IteratorProtocol",
            "Default" => return None,
            "Serialize" | "Deserialize" => "Codable",
            // As a bound only: an `impl FromStr` is a `fromStr` function, called directly.
            "FromStr" => "LosslessStringConvertible",
            "View" => "View",
            "Add" => "AdditiveArithmetic",
            _ => return None,
        }
        .to_string(),
    )
}

fn is_dropped_trait(name: &str) -> bool {
    matches!(
        name,
        "Clone"
            | "Copy"
            | "Send"
            | "Sync"
            | "Sized"
            | "Unpin"
            | "Default"
            | "Debug"
            | "Into"
            | "From"
            | "AsRef"
            | "FromStr"
            | "TryFrom"
            | "TryInto"
    )
}

/// Swift operator for an operator trait: (operator, compound assignment, prefix).
fn operator_of(trait_name: &str) -> Option<(&'static str, bool, bool)> {
    Some(match trait_name {
        "Add" => ("+", false, false),
        "Sub" => ("-", false, false),
        "Mul" => ("*", false, false),
        "Div" => ("/", false, false),
        "Rem" => ("%", false, false),
        "AddAssign" => ("+=", true, false),
        "SubAssign" => ("-=", true, false),
        "MulAssign" => ("*=", true, false),
        "DivAssign" => ("/=", true, false),
        "Neg" => ("-", false, true),
        "Not" => ("!", false, true),
        "Index" | "IndexMut" => ("[]", false, false),
        _ => return None,
    })
}

pub fn impl_trait_name(imp: &syn::ItemImpl) -> Option<String> {
    imp.trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .map(|s| s.ident.to_string())
}

fn collect_impls<'f>(items: &'f [Item], out: &mut HashMap<String, Vec<&'f syn::ItemImpl>>) {
    for item in items {
        match item {
            Item::Impl(i) => {
                if let Some(n) = crate::symbols::type_name(&i.self_ty) {
                    out.entry(n).or_default().push(i);
                }
            }
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_impls(inner, out);
                }
            }
            _ => {}
        }
    }
}

fn struct_in(items: &[Item], name: &str) -> bool {
    items.iter().any(|i| match i {
        Item::Struct(s) => s.ident == name,
        Item::Mod(m) => m
            .content
            .as_ref()
            .is_some_and(|(_, inner)| struct_in(inner, name)),
        _ => false,
    })
}

fn mentions_type(t: &Type, name: &str) -> bool {
    let s = quote::quote!(#t).to_string();
    s.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|w| w == name || w == "Self")
}

pub fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|a| a.path().segments.last().is_some_and(|s| s.ident == name))
}

fn doc_text(a: &Attribute) -> Option<String> {
    if !a.path().is_ident("doc") {
        return None;
    }
    if let syn::Meta::NameValue(nv) = &a.meta {
        if let syn::Expr::Lit(l) = &nv.value {
            if let syn::Lit::Str(s) = &l.lit {
                return Some(s.value());
            }
        }
    }
    None
}

/// Doc comments: `/// text` lines, followed by a newline.
pub fn docs(attrs: &[Attribute]) -> String {
    let mut out = String::new();
    for a in attrs {
        if let Some(text) = doc_text(a) {
            for line in text.lines() {
                out.push_str("///");
                if !line.is_empty() && !line.starts_with(' ') {
                    out.push(' ');
                }
                out.push_str(line.trim_end());
                out.push('\n');
            }
            if text.is_empty() {
                out.push_str("///\n");
            }
        }
    }
    out
}

/// The first line of an item, its attributes included.
pub fn first_line(attrs: &[Attribute], span: Span) -> usize {
    attrs
        .iter()
        .map(|a| a.span().start().line)
        .chain(std::iter::once(span.start().line))
        .min()
        .unwrap_or(1)
}

/// Indents every non-empty line by one level.
pub fn indent_lines(s: &str) -> String {
    s.lines()
        .map(|l| {
            if l.is_empty() {
                String::new()
            } else {
                format!("{INDENT}{l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn dedup(v: &mut Vec<String>) {
    let mut seen = HashSet::new();
    v.retain(|x| seen.insert(x.clone()));
}

/// Whether Swift infers the right type from the value alone (then no annotation is needed).
pub fn natural_type_matches(value: &Ty, declared: &Ty) -> bool {
    match (value, declared) {
        (_, Ty::Unknown) => true,
        (Ty::Int(a), Ty::Int(b)) => a.swift() == "Int" && b.swift() == "Int",
        (Ty::Float(a), Ty::Float(b)) => a == b && *b == crate::ty::FloatKind::F64,
        (Ty::Str, Ty::Str) | (Ty::Bool, Ty::Bool) => true,
        (Ty::Named(a, x), Ty::Named(b, y)) => a == b && x.is_empty() && y.is_empty(),
        (Ty::Vec(a), Ty::Vec(b)) => a.is_complete() && natural_type_matches(a, b),
        _ => false,
    }
}

/// `{:?}` of an enum, written like Rust's derived `Debug`.
fn debug_description(info: &crate::symbols::EnumInfo) -> String {
    use crate::symbols::VariantShape;
    let mut cases = Vec::new();
    for v in &info.variants {
        let (pattern, text) = match &v.shape {
            VariantShape::Unit => (format!(".{}", v.swift), v.rust.clone()),
            VariantShape::Tuple(ts) => {
                let names: Vec<String> = (0..ts.len()).map(|i| format!("v{i}")).collect();
                let values: Vec<String> =
                    names.iter().map(|n| format!("\\(rsDebug({n}))")).collect();
                (
                    format!(
                        ".{}({})",
                        v.swift,
                        names
                            .iter()
                            .map(|n| format!("let {n}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    format!("{}({})", v.rust, values.join(", ")),
                )
            }
            VariantShape::Struct(fields) => {
                let names: Vec<String> =
                    fields.iter().map(|(f, _)| names::lower_camel(f)).collect();
                let values: Vec<String> = fields
                    .iter()
                    .zip(&names)
                    .map(|((f, _), n)| format!("{f}: \\(rsDebug({n}))"))
                    .collect();
                (
                    format!(
                        ".{}({})",
                        v.swift,
                        names
                            .iter()
                            .map(|n| format!("let {n}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    format!("{} {{ {} }}", v.rust, values.join(", ")),
                )
            }
        };
        cases.push(format!("case {pattern}:\n{INDENT}return \"{text}\""));
    }
    format!(
        "\n\nextension {}: CustomDebugStringConvertible {{\n{INDENT}var debugDescription: String {{\n{}\n{INDENT}}}\n}}",
        info.swift,
        indent_lines(&indent_lines(&format!(
            "switch self {{\n{}\n}}",
            cases.join("\n")
        )))
    )
}

/// `switch x {\ncase …:\n    out += "a"\n…}` → the same `switch` with `"a"` as the value of
/// each case, when every case writes exactly once.
fn switch_of_writes(body: &str, out: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    if !lines.first()?.starts_with("switch ") || *lines.last()? != "}" {
        return None;
    }
    let prefix = format!("{INDENT}{out} += ");
    let mut result = vec![lines[0].to_string()];
    let mut writes_in_case = 0;
    for line in &lines[1..lines.len() - 1] {
        if line.starts_with("case ") || line.starts_with("default:") {
            if result.len() > 1 && writes_in_case != 1 {
                return None;
            }
            writes_in_case = 0;
            result.push((*line).to_string());
        } else {
            let value = line.strip_prefix(&prefix)?;
            writes_in_case += 1;
            result.push(format!("{INDENT}{value}"));
        }
    }
    if writes_in_case != 1 {
        return None;
    }
    result.push("}".into());
    Some(result.join("\n"))
}

/// Protocols Swift synthesizes from the members.
fn is_synthesized(p: &str) -> bool {
    matches!(p, "Equatable" | "Hashable" | "Comparable" | "Codable")
}

/// `extension Pair: Equatable where A: Equatable, B: Equatable {}` for each protocol.
fn conditional_conformances(
    swift: &str,
    generics: &[String],
    protocols: &[String],
    less: Option<&str>,
) -> String {
    let mut out = String::new();
    for p in protocols {
        let wc: Vec<String> = generics.iter().map(|g| format!("{g}: {p}")).collect();
        let body = if p == "Comparable" {
            less.map(|l| format!(" {{\n{}\n}}", indent_lines(l)))
                .unwrap_or_else(|| " {}".into())
        } else {
            " {}".into()
        };
        out.push_str(&format!(
            "\n\nextension {swift}: {p} where {}{body}",
            wc.join(", ")
        ));
    }
    out
}

/// `static func <` comparing fields in order, like `#[derive(PartialOrd)]`.
fn lexicographic_less(swift: &str, fields: &[String]) -> String {
    let mut body = String::new();
    for f in fields.iter().take(fields.len().saturating_sub(1)) {
        body.push_str(&format!(
            "{INDENT}if lhs.{f} != rhs.{f} {{ return lhs.{f} < rhs.{f} }}\n"
        ));
    }
    match fields.last() {
        Some(f) => body.push_str(&format!("{INDENT}return lhs.{f} < rhs.{f}")),
        None => body.push_str(&format!("{INDENT}return false")),
    }
    format!("static func < (lhs: {swift}, rhs: {swift}) -> Bool {{\n{body}\n}}")
}

/// Whether a `Result<_, String>` appears in the type.
fn mentions_string_error(t: &Ty) -> bool {
    match t {
        Ty::Result(ok, err) => {
            matches!(**err, Ty::Str) || mentions_string_error(ok) || mentions_string_error(err)
        }
        Ty::Vec(t) | Ty::Set(t, _) | Ty::Option(t) | Ty::Range(t) | Ty::Iter(t) => {
            mentions_string_error(t)
        }
        Ty::Map(k, v, _) => mentions_string_error(k) || mentions_string_error(v),
        Ty::Tuple(ts) | Ty::Named(_, ts) => ts.iter().any(mentions_string_error),
        Ty::Fn(ps, r) => ps.iter().any(mentions_string_error) || mentions_string_error(r),
        _ => false,
    }
}

/// The names of the library and user types that `t` mentions (`RsTuple1` for `(T,)`).
fn named_types(t: &Ty, out: &mut Vec<String>) {
    match t {
        Ty::Named(n, args) => {
            out.push(n.clone());
            if (n == "RsReverse" || n == "RsBinaryHeap")
                && args
                    .first()
                    .is_some_and(|a| crate::ty::ord_tuple_arity(a).is_some())
            {
                out.push("RsOrdTuple".into());
            }
            for a in args {
                named_types(a, out);
            }
        }
        Ty::Set(t, _) if matches!(&**t, Ty::Tuple(ts) if ts.len() == 2 || ts.len() == 3) => {
            out.push("RsTupleKeys".into());
            named_types(t, out);
        }
        Ty::Vec(t) | Ty::Set(t, _) | Ty::Option(t) | Ty::Range(t) | Ty::Iter(t) => {
            named_types(t, out);
        }
        Ty::Map(k, v, _) | Ty::Result(k, v) => {
            if matches!(&**k, Ty::Tuple(ts) if ts.len() == 2 || ts.len() == 3) {
                out.push("RsTupleKeys".into());
            }
            named_types(k, out);
            named_types(v, out);
        }
        Ty::Tuple(ts) => {
            if ts.len() == 1 {
                out.push("RsTuple1".into());
            }
            for t in ts {
                named_types(t, out);
            }
        }
        Ty::Fn(ps, r) => {
            for p in ps {
                named_types(p, out);
            }
            named_types(r, out);
        }
        _ => {}
    }
}

/// Whether `name` appears in `block` other than as the function of a call: a closure
/// parameter passed on or stored escapes.
fn used_as_value(block: &syn::Block, name: &str) -> bool {
    use syn::visit::Visit;
    struct V<'a> {
        name: &'a str,
        found: bool,
    }
    impl<'ast> Visit<'ast> for V<'_> {
        fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
            // `handler(e)`: a call, not a use as a value.
            if !matches!(&*c.func, syn::Expr::Path(p) if p.path.is_ident(self.name)) {
                self.visit_expr(&c.func);
            }
            for a in &c.args {
                self.visit_expr(a);
            }
        }
        fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
            if p.path.is_ident(self.name) {
                self.found = true;
            }
        }
    }
    let mut v = V { name, found: false };
    v.visit_block(block);
    v.found
}

/// `a == b` for a payload field, written out where Swift has no `==` (arrays of tuples).
fn equality_term(a: &str, b: &str, t: &Ty) -> String {
    match t {
        Ty::Vec(i) if matches!(**i, Ty::Tuple(_)) => {
            format!("{a}.count == {b}.count && zip({a}, {b}).allSatisfy {{ $0 == $1 }}")
        }
        Ty::Option(i) if matches!(**i, Ty::Tuple(_)) => {
            format!("({a} == nil && {b} == nil || {a} != nil && {b} != nil && {a}! == {b}!)")
        }
        _ => format!("{a} == {b}"),
    }
}

/// `hasher.combine(a)` for a payload field, element by element for tuples.
fn hash_term(a: &str, t: &Ty) -> String {
    match t {
        Ty::Vec(i) if matches!(**i, Ty::Tuple(_)) => {
            let Ty::Tuple(ts) = &**i else { unreachable!() };
            let parts: Vec<String> = (0..ts.len())
                .map(|k| format!("hasher.combine(x.{k})"))
                .collect();
            format!("for x in {a} {{ {} }}", parts.join("; "))
        }
        Ty::Option(i) if matches!(**i, Ty::Tuple(_)) => {
            let Ty::Tuple(ts) = &**i else { unreachable!() };
            let parts: Vec<String> = (0..ts.len())
                .map(|k| format!("hasher.combine(x.{k})"))
                .collect();
            format!("if let x = {a} {{ {} }}", parts.join("; "))
        }
        _ => format!("hasher.combine({a})"),
    }
}

/// `a < b` for a payload field: arrays lexicographically, `None` before `Some`, `false`
/// before `true`, as Rust orders them.
fn less_term(a: &str, b: &str, t: &Ty) -> String {
    match t {
        Ty::Vec(i) if matches!(**i, Ty::Tuple(_)) => {
            format!("{a}.lexicographicallyPrecedes({b}, by: {{ $0 < $1 }})")
        }
        Ty::Vec(_) => format!("{a}.lexicographicallyPrecedes({b})"),
        Ty::Option(i) => format!(
            "({a} == nil ? {b} != nil : ({b} != nil && {}))",
            less_term(&format!("{a}!"), &format!("{b}!"), i)
        ),
        Ty::Bool => format!("(!{a} && {b})"),
        _ => format!("{a} < {b}"),
    }
}

/// `Vec<(A, B)>`: Swift cannot synthesize `==` or `hash` for it.
fn has_tuple_collection(t: &Ty) -> bool {
    match t {
        Ty::Vec(i) | Ty::Option(i) | Ty::Set(i, _) => {
            matches!(**i, Ty::Tuple(_)) || has_tuple_collection(i)
        }
        Ty::Map(_, v, _) => matches!(**v, Ty::Tuple(_)) || has_tuple_collection(v),
        _ => false,
    }
}

/// `&mut T`, `Option<&mut T>`, `Result<&mut T, E>` as a return type.
fn has_mut_reference(t: &Type) -> bool {
    match t {
        Type::Reference(r) => r.mutability.is_some(),
        Type::Path(p) => p.path.segments.last().is_some_and(|seg| {
            matches!(seg.ident.to_string().as_str(), "Option" | "Result")
                && matches!(&seg.arguments, syn::PathArguments::AngleBracketed(a)
                    if matches!(a.args.first(), Some(syn::GenericArgument::Type(inner)) if has_mut_reference(inner)))
        }),
        _ => false,
    }
}
