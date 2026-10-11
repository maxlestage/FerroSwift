//! What the whole crate declares, collected before translating: types and their fields,
//! enum variants, functions and methods with their signatures. The translation of a file
//! needs it to order struct fields, find enum variants, know which functions throw…

use std::collections::{HashMap, HashSet};

use syn::{Attribute, Expr, Fields, FnArg, GenericParam, Item, ReturnType, Type};

use crate::names;
use crate::ty::{Ty, from_name};

#[derive(Clone, Debug, PartialEq)]
pub enum FieldRole {
    Plain,
    /// `#[state]`, with its initial value if given.
    State(Option<Expr>),
    Binding,
    /// `#[environment(key)]`.
    Environment(String),
    /// `#[observation_ignored]`.
    Ignored,
}

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub rust: String,
    pub swift: String,
    pub ty: Ty,
    pub role: FieldRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeKind {
    Plain,
    /// `#[derive(View)]` or `impl View for`.
    View,
    /// `impl App for`.
    App,
    /// `#[derive(Observable)]`: a class.
    Observable,
}

#[derive(Clone, Debug)]
pub struct StructInfo {
    pub swift: String,
    pub fields: Vec<FieldInfo>,
    /// `struct Meters(f64);`
    pub tuple: bool,
    pub generics: Vec<String>,
    pub derives: Vec<String>,
    pub kind: TypeKind,
    /// `struct Node { next: Option<Box<Node>> }`: a Swift value type cannot contain itself,
    /// so the struct becomes a class.
    pub recursive: bool,
}

impl StructInfo {
    pub fn field(&self, rust: &str) -> Option<&FieldInfo> {
        self.fields.iter().find(|f| f.rust == rust)
    }
}

#[derive(Clone, Debug)]
pub enum VariantShape {
    Unit,
    Tuple(Vec<Ty>),
    Struct(Vec<(String, Ty)>),
}

#[derive(Clone, Debug)]
pub struct VariantInfo {
    pub rust: String,
    pub swift: String,
    pub shape: VariantShape,
}

#[derive(Clone, Debug)]
pub struct EnumInfo {
    pub swift: String,
    pub variants: Vec<VariantInfo>,
    pub generics: Vec<String>,
    pub derives: Vec<String>,
}

impl EnumInfo {
    pub fn variant(&self, rust: &str) -> Option<&VariantInfo> {
        self.variants.iter().find(|v| v.rust == rust)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfKind {
    None,
    Ref,
    RefMut,
    Value,
}

#[derive(Clone, Debug)]
pub struct FnSig {
    pub params: Vec<ParamInfo>,
    pub ret: Ty,
    pub self_kind: SelfKind,
    /// `fn new(…) -> Self` translated as an initializer.
    pub is_init: bool,
    /// `fn count(&self) -> usize` next to a field `count`: a computed property in Swift.
    pub property: bool,
    /// `fn add(&mut self, …) -> &mut Self`: a builder method, chained as statements.
    pub returns_self_mut: bool,
    /// `fn get(&mut self, …) -> &mut T` (or `Option`/`Result` of it): returns a
    /// `WritableKeyPath` to the value.
    pub key_path: bool,
    /// Declared in an `impl Trait for`: the protocol fixes its shape (a function).
    pub trait_method: bool,
    /// The type parameters, in order (`fn largest<T: PartialOrd>(…)`).
    pub generics: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub ty: Ty,
    /// `&mut T`: an `inout` parameter.
    pub inout: bool,
}

#[derive(Default, Debug)]
pub struct Symbols {
    pub structs: HashMap<String, StructInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub traits: HashSet<String>,
    pub fns: HashMap<String, FnSig>,
    /// Methods and associated functions by type (Rust name) then method name.
    pub methods: HashMap<String, HashMap<String, FnSig>>,
    pub consts: HashMap<String, Ty>,
    pub aliases: HashMap<String, Ty>,
    /// The type parameters of the generic aliases (`type Result<T> = …`).
    pub alias_params: HashMap<String, Vec<String>>,
    /// The structs and enums of the crate, by Rust name, known before anything else.
    pub user_type_names: HashSet<String>,
    /// Types that implement `Display` (their `description`).
    pub display: HashSet<String>,
    /// Types whose `Display` asks its formatter for the spec (`f.pad(…)`, `f.precision()`,
    /// `self.0.fmt(f)`): written with a formatter, so that the width applies to them, unlike
    /// `write!`-based impls.
    pub fmt_queries: HashSet<String>,
    /// Types whose `Debug` uses its formatter (`f.debug_struct(…)`, `f.alternate()`).
    pub debug_queries: HashSet<String>,
    /// Types used as the error of a `Result`: they conform to `Error`.
    pub errors: HashSet<String>,
    /// Types with a `From<…>` implementation, by target.
    pub from_impls: HashMap<String, Vec<Ty>>,
    /// Associated types of the impls (`type Err = String;`), by (Swift type name, name).
    pub assoc: HashMap<(String, String), Ty>,
    /// Associated constants (`const ALL: [Weekday; 7]`), by (Rust type name, name).
    pub assoc_consts: HashMap<(String, String), Ty>,
    /// The ferroswift UI library is imported somewhere.
    pub ui: bool,
    /// Names brought in by `use` (`PI` → `std::f64::consts::PI`), without the globs.
    pub imports: HashMap<String, Vec<String>>,
    /// Types implementing `Iterator`, with their `Item` type (by Rust and by Swift name).
    pub iterators: HashMap<String, Ty>,
    /// `fn index_mut` of the `IndexMut` impls, by Rust type name: the setter of the subscript.
    pub index_mut: HashMap<String, syn::ImplItemFn>,
}

/// The names of a `use` tree with their full paths.
fn collect_imports(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    out: &mut HashMap<String, Vec<String>>,
) {
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            collect_imports(&p.tree, prefix, out);
            prefix.pop();
        }
        syn::UseTree::Name(n) => {
            let name = n.ident.to_string();
            if name != "self" {
                let mut path = prefix.clone();
                path.push(name.clone());
                out.insert(name, path);
            } else if let Some(last) = prefix.last() {
                out.insert(last.clone(), prefix.clone());
            }
        }
        syn::UseTree::Rename(r) => {
            let mut path = prefix.clone();
            path.push(r.ident.to_string());
            out.insert(r.rename.to_string(), path);
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                collect_imports(t, prefix, out);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

impl Symbols {
    /// Collects the declarations of every file.
    pub fn collect(files: &[&syn::File]) -> Symbols {
        let mut s = Symbols::default();
        // The names of the types first: an alias may mention them.
        for file in files {
            s.walk(&file.items, &mut |s, item| match item {
                Item::Struct(st) => {
                    s.user_type_names.insert(st.ident.to_string());
                }
                Item::Enum(en) => {
                    s.user_type_names.insert(en.ident.to_string());
                }
                _ => {}
            });
        }
        // Then the aliases: the other declarations may use them.
        for file in files {
            s.walk(&file.items, &mut |s, item| {
                if let Item::Type(t) = item {
                    let ty = s.ty(&t.ty, None);
                    s.aliases.insert(t.ident.to_string(), ty);
                    s.alias_params
                        .insert(t.ident.to_string(), type_params(&t.generics));
                }
            });
        }
        for file in files {
            s.walk(&file.items, &mut |s, item| s.declare(item));
        }
        for file in files {
            s.walk(&file.items, &mut |s, item| s.declare_impl(item));
        }
        s.properties_for_clashes();
        s
    }

    /// A field `count` and a method `count(&self)` cannot share a name in Swift: the method
    /// becomes a computed property `count`, the stored field `_count`.
    fn properties_for_clashes(&mut self) {
        for (type_name, st) in &mut self.structs {
            let Some(methods) = self.methods.get_mut(type_name) else {
                continue;
            };
            for field in &mut st.fields {
                let clash = methods.get_mut(&field.rust).filter(|m| {
                    m.params.is_empty()
                        && matches!(m.self_kind, SelfKind::Ref | SelfKind::Value)
                        && !m.trait_method
                });
                if let Some(m) = clash {
                    m.property = true;
                    field.swift = format!("_{}", field.swift);
                } else if methods.contains_key(&field.rust) {
                    // A field `redo` and a method `redo(&mut self)`: the stored field is renamed.
                    field.swift = format!("_{}", field.swift);
                }
            }
        }
    }

    fn walk(&mut self, items: &[Item], f: &mut dyn FnMut(&mut Symbols, &Item)) {
        for item in items {
            if is_test_only(item_attrs(item)) {
                continue;
            }
            match item {
                Item::Mod(m) => {
                    if let Some((_, inner)) = &m.content {
                        self.walk(inner, f);
                    }
                }
                Item::Use(u) => {
                    if quote::quote!(#u).to_string().contains("ferroswift") {
                        self.ui = true;
                    }
                    collect_imports(&u.tree, &mut Vec::new(), &mut self.imports);
                }
                Item::Fn(func) => {
                    f(self, item);
                    // Functions declared inside a function body.
                    let nested: Vec<Item> = func
                        .block
                        .stmts
                        .iter()
                        .filter_map(|s| match s {
                            syn::Stmt::Item(it) => Some(it.clone()),
                            _ => None,
                        })
                        .collect();
                    self.walk(&nested, f);
                }
                _ => f(self, item),
            }
        }
    }

    fn declare(&mut self, item: &Item) {
        match item {
            Item::Struct(st) => {
                let derives = derives(&st.attrs);
                let generics = type_params(&st.generics);
                let mut kind = TypeKind::Plain;
                if derives.iter().any(|d| d == "View") {
                    kind = TypeKind::View;
                    self.ui = true;
                }
                if derives.iter().any(|d| d == "Observable") {
                    kind = TypeKind::Observable;
                    self.ui = true;
                }
                let tuple = matches!(st.fields, Fields::Unnamed(_));
                let fields: Vec<FieldInfo> = st
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let rust = f
                            .ident
                            .as_ref()
                            .map_or_else(|| i.to_string(), ToString::to_string);
                        let swift = match &f.ident {
                            Some(id) => names::lower_camel(&id.to_string()),
                            None => format!("_{i}"),
                        };
                        FieldInfo {
                            rust,
                            swift,
                            ty: self.ty(&f.ty, None),
                            role: field_role(&f.attrs),
                        }
                    })
                    .collect();
                let swift = names::type_name(&st.ident.to_string());
                let recursive = fields.iter().any(|f| directly_contains(&f.ty, &swift));
                self.structs.insert(
                    st.ident.to_string(),
                    StructInfo {
                        swift,
                        fields,
                        tuple,
                        generics,
                        derives,
                        kind,
                        recursive,
                    },
                );
            }
            Item::Enum(en) => {
                let variants = en
                    .variants
                    .iter()
                    .map(|v| VariantInfo {
                        rust: v.ident.to_string(),
                        swift: names::case_name(&v.ident.to_string()),
                        shape: match &v.fields {
                            Fields::Unit => VariantShape::Unit,
                            Fields::Unnamed(f) => VariantShape::Tuple(
                                f.unnamed.iter().map(|f| self.ty(&f.ty, None)).collect(),
                            ),
                            Fields::Named(f) => VariantShape::Struct(
                                f.named
                                    .iter()
                                    .map(|f| {
                                        (
                                            f.ident
                                                .as_ref()
                                                .map(ToString::to_string)
                                                .unwrap_or_default(),
                                            self.ty(&f.ty, None),
                                        )
                                    })
                                    .collect(),
                            ),
                        },
                    })
                    .collect();
                self.enums.insert(
                    en.ident.to_string(),
                    EnumInfo {
                        swift: names::type_name(&en.ident.to_string()),
                        variants,
                        generics: type_params(&en.generics),
                        derives: derives(&en.attrs),
                    },
                );
            }
            Item::Trait(t) => {
                self.traits.insert(t.ident.to_string());
            }
            Item::Fn(f) => {
                let sig = self.sig(&f.sig, None);
                self.note_error_type(&sig.ret);
                self.fns.insert(f.sig.ident.to_string(), sig);
            }
            Item::Const(c) => {
                let ty = self.ty(&c.ty, None);
                self.consts.insert(c.ident.to_string(), ty);
            }
            Item::Static(c) => {
                let ty = self.ty(&c.ty, None);
                self.consts.insert(c.ident.to_string(), ty);
            }
            _ => {}
        }
    }

    fn declare_impl(&mut self, item: &Item) {
        let Item::Impl(imp) = item else { return };
        let Some(type_name) = type_name(&imp.self_ty) else {
            return;
        };
        let self_ty = self.ty(&imp.self_ty, None);
        let trait_name = imp
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|s| s.ident.to_string());
        match trait_name.as_deref() {
            Some("View") => {
                self.ui = true;
                if let Some(st) = self.structs.get_mut(&type_name) {
                    if st.kind == TypeKind::Plain {
                        st.kind = TypeKind::View;
                    }
                }
            }
            Some("App") => {
                self.ui = true;
                if let Some(st) = self.structs.get_mut(&type_name) {
                    st.kind = TypeKind::App;
                }
            }
            Some("Display") => {
                self.display.insert(type_name.clone());
                if impl_queries_formatter(imp) {
                    self.fmt_queries.insert(type_name.clone());
                }
            }
            Some("Debug") => {
                if impl_queries_formatter(imp) {
                    self.debug_queries.insert(type_name.clone());
                }
            }
            Some("Iterator") => {
                let item = imp
                    .items
                    .iter()
                    .find_map(|it| match it {
                        syn::ImplItem::Type(t) if t.ident == "Item" => {
                            Some(self.ty(&t.ty, Some(&self_ty)))
                        }
                        _ => None,
                    })
                    .unwrap_or(Ty::Unknown);
                self.iterators.insert(type_name.clone(), item.clone());
                if let Ty::Named(swift, _) = &self_ty {
                    self.iterators.insert(swift.clone(), item);
                }
            }
            Some("Error") => {
                self.errors.insert(type_name.clone());
            }
            Some("IndexMut") => {
                for it in &imp.items {
                    if let syn::ImplItem::Fn(f) = it {
                        if f.sig.ident == "index_mut" {
                            self.index_mut.insert(type_name.clone(), f.clone());
                        }
                    }
                }
            }
            Some("From") => {
                if let Some((_, path, _)) = &imp.trait_ {
                    if let Some(syn::PathArguments::AngleBracketed(args)) =
                        path.segments.last().map(|s| &s.arguments)
                    {
                        if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                            let from = self.ty(t, Some(&self_ty));
                            self.from_impls
                                .entry(type_name.clone())
                                .or_default()
                                .push(from);
                        }
                    }
                }
            }
            _ => {}
        }
        for it in &imp.items {
            if let syn::ImplItem::Const(c) = it {
                let ty = self.ty(&c.ty, Some(&self_ty));
                self.assoc_consts
                    .insert((type_name.clone(), c.ident.to_string()), ty);
            }
        }
        // `type Err = String;` before the signatures that mention `Self::Err`.
        for it in &imp.items {
            if let syn::ImplItem::Type(t) = it {
                let ty = self.ty(&t.ty, Some(&self_ty));
                if let Ty::Named(swift, _) = &self_ty {
                    self.assoc.insert((swift.clone(), t.ident.to_string()), ty);
                }
            }
        }
        let observable = self
            .structs
            .get(&type_name)
            .is_some_and(|s| s.kind == TypeKind::Observable);
        for it in &imp.items {
            if let syn::ImplItem::Fn(f) = it {
                let mut sig = self.sig(&f.sig, Some(&self_ty));
                sig.trait_method = trait_name.is_some();
                let name = f.sig.ident.to_string();
                if trait_name.is_none() && !observable && sig.self_kind == SelfKind::None {
                    sig.is_init = name == "new" && returns_self(&sig.ret, &self_ty);
                }
                if trait_name.as_deref() == Some("Default") && name == "default" {
                    sig.is_init = true;
                }
                if trait_name.as_deref() == Some("From") && name == "from" {
                    sig.is_init = true;
                }
                self.note_error_type(&sig.ret);
                self.methods
                    .entry(type_name.clone())
                    .or_default()
                    .insert(name, sig);
            }
        }
        // Two initializers with no parameter would clash: `new()` stays a static method.
        if let Some(methods) = self.methods.get_mut(&type_name) {
            let default_init = methods.get("default").is_some_and(|m| m.is_init)
                || self.structs.get(&type_name).is_some_and(|s| {
                    s.derives.iter().any(|d| d == "Default")
                        // Optional fields give Swift structs an `init()` of their own.
                        || !s.tuple
                            && !s.recursive
                            && !s.fields.is_empty()
                            && s.fields.iter().all(|f| matches!(f.ty, Ty::Option(_)))
                });
            if default_init {
                if let Some(new) = methods.get_mut("new") {
                    if new.params.is_empty() {
                        new.is_init = false;
                    }
                }
            }
        }
    }

    fn note_error_type(&mut self, ret: &Ty) {
        if let Ty::Result(_, e) = ret {
            if let Ty::Named(n, _) = &**e {
                self.errors.insert(n.clone());
            }
        }
    }

    /// Generic parameters bounded by a closure trait (`F: Fn(i32) -> i32`), with the closure
    /// type that replaces them.
    pub fn closure_generics(&self, g: &syn::Generics, self_ty: Option<&Ty>) -> HashMap<String, Ty> {
        let mut out = HashMap::new();
        for p in &g.params {
            if let GenericParam::Type(t) = p {
                if let Ty::Fn(a, r) = self.bounds_ty(&t.bounds, self_ty, true) {
                    out.insert(t.ident.to_string(), Ty::Fn(a, r));
                }
            }
        }
        if let Some(w) = &g.where_clause {
            for pred in &w.predicates {
                if let syn::WherePredicate::Type(pt) = pred {
                    if let Type::Path(tp) = &pt.bounded_ty {
                        if let Some(id) = tp.path.get_ident() {
                            if let Ty::Fn(a, r) = self.bounds_ty(&pt.bounds, self_ty, true) {
                                out.insert(id.to_string(), Ty::Fn(a, r));
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// The signature of a function or method.
    pub fn sig(&self, sig: &syn::Signature, self_ty: Option<&Ty>) -> FnSig {
        let closures = self.closure_generics(&sig.generics, self_ty);
        let mut self_kind = SelfKind::None;
        let mut params = Vec::new();
        for arg in &sig.inputs {
            match arg {
                FnArg::Receiver(r) => {
                    self_kind = match (&r.reference, r.mutability.is_some()) {
                        (Some(_), true) => SelfKind::RefMut,
                        (Some(_), false) => SelfKind::Ref,
                        (None, _) => SelfKind::Value,
                    };
                }
                FnArg::Typed(p) => {
                    let ty = substitute(&self.ty(&p.ty, self_ty), &closures);
                    // A closure is shared, not passed `inout`, as the signature has it.
                    let inout = matches!(&*p.ty, Type::Reference(r) if r.mutability.is_some())
                        && !matches!(ty, Ty::Fn(..) | Ty::View);
                    params.push(ParamInfo { ty, inout });
                }
            }
        }
        let ret = match &sig.output {
            ReturnType::Default => Ty::Unit,
            ReturnType::Type(_, t) => self.ty(t, self_ty),
        };
        let returns_self_mut = match &sig.output {
            syn::ReturnType::Type(_, t) => {
                matches!(&**t, Type::Reference(r) if r.mutability.is_some())
                    && self_ty.is_some_and(|st| returns_self(&ret, st))
            }
            syn::ReturnType::Default => false,
        };
        let key_path = match &sig.output {
            syn::ReturnType::Type(_, t) => has_mut_reference(t) && !returns_self_mut,
            syn::ReturnType::Default => false,
        };
        FnSig {
            params,
            ret,
            self_kind,
            is_init: false,
            property: false,
            returns_self_mut,
            key_path,
            trait_method: false,
            generics: type_params(&sig.generics),
        }
    }

    /// A Rust type as the translator sees it. `self_ty` replaces `Self`.
    pub fn ty(&self, t: &Type, self_ty: Option<&Ty>) -> Ty {
        match t {
            Type::Path(p) => {
                if p.qself.is_some() {
                    return Ty::Unknown;
                }
                let Some(last) = p.path.segments.last() else {
                    return Ty::Unknown;
                };
                let name = last.ident.to_string();
                if name == "Self" && p.path.segments.len() == 1 {
                    return self_ty
                        .cloned()
                        .unwrap_or_else(|| Ty::Named("Self".into(), vec![]));
                }
                // `Self::Err`, `Self::Item`: an associated type of the impl.
                if p.path.segments.len() == 2 && p.path.segments[0].ident == "Self" {
                    if let Some(Ty::Named(swift, _)) = self_ty {
                        if let Some(t) = self.assoc.get(&(swift.clone(), name.clone())) {
                            return t.clone();
                        }
                    }
                }
                let mut args = Vec::new();
                if let syn::PathArguments::AngleBracketed(a) = &last.arguments {
                    for arg in &a.args {
                        if let syn::GenericArgument::Type(t) = arg {
                            args.push(self.ty(t, self_ty));
                        }
                    }
                }
                let first = p.path.segments.first().map(|s| s.ident.to_string());
                // `fmt::Result`, `io::Result<T>`, `io::Error`.
                if p.path.segments.len() > 1 {
                    match (first.as_deref(), name.as_str()) {
                        (Some("fmt"), "Result") => {
                            return Ty::Result(
                                Box::new(Ty::Unit),
                                Box::new(Ty::Dyn("Error".into())),
                            );
                        }
                        (Some("io"), "Result") => {
                            let ok = args.first().cloned().unwrap_or(Ty::Unit);
                            return Ty::Result(Box::new(ok), Box::new(Ty::Dyn("Error".into())));
                        }
                        (_, "Error") if !self.is_user_type("Error") => {
                            return Ty::Dyn("Error".into());
                        }
                        _ => {}
                    }
                }
                // A user alias, generic or not (`type Result<T> = …`); never for a `std` path.
                if !matches!(first.as_deref(), Some("std" | "core" | "alloc")) {
                    if let Some(alias) = self.aliases.get(&name) {
                        let params = self.alias_params.get(&name).cloned().unwrap_or_default();
                        if params.is_empty() {
                            return alias.clone();
                        }
                        let map: HashMap<String, Ty> =
                            params.into_iter().zip(args.iter().cloned()).collect();
                        return substitute(alias, &map);
                    }
                }
                if self.user_type_names.contains(&name)
                    || self.structs.contains_key(&name)
                    || self.enums.contains_key(&name)
                {
                    return Ty::Named(names::type_name(&name), args);
                }
                if name == "Element" && first.as_deref() != Some("std") && self.ui {
                    return Ty::View;
                }
                from_name(&name, args)
            }
            Type::Reference(r) => self.ty(&r.elem, self_ty),
            Type::Slice(s) => Ty::Vec(Box::new(self.ty(&s.elem, self_ty))),
            Type::Array(a) => Ty::Vec(Box::new(self.ty(&a.elem, self_ty))),
            Type::Tuple(t) if t.elems.is_empty() => Ty::Unit,
            Type::Tuple(t) => Ty::Tuple(t.elems.iter().map(|e| self.ty(e, self_ty)).collect()),
            Type::Never(_) => Ty::Never,
            Type::Paren(p) => self.ty(&p.elem, self_ty),
            Type::Group(g) => self.ty(&g.elem, self_ty),
            Type::BareFn(f) => Ty::Fn(
                f.inputs.iter().map(|a| self.ty(&a.ty, self_ty)).collect(),
                Box::new(match &f.output {
                    ReturnType::Default => Ty::Unit,
                    ReturnType::Type(_, t) => self.ty(t, self_ty),
                }),
            ),
            Type::ImplTrait(i) => self.bounds_ty(&i.bounds, self_ty, true),
            Type::TraitObject(o) => self.bounds_ty(&o.bounds, self_ty, false),
            _ => Ty::Unknown,
        }
    }

    pub fn is_user_type(&self, name: &str) -> bool {
        self.user_type_names.contains(name)
            || self.structs.contains_key(name)
            || self.enums.contains_key(name)
    }

    /// `impl Trait` / `dyn Trait`.
    fn bounds_ty(
        &self,
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
        self_ty: Option<&Ty>,
        opaque: bool,
    ) -> Ty {
        for b in bounds {
            let syn::TypeParamBound::Trait(t) = b else {
                continue;
            };
            let Some(seg) = t.path.segments.last() else {
                continue;
            };
            let name = seg.ident.to_string();
            match name.as_str() {
                "Fn" | "FnMut" | "FnOnce" => {
                    if let syn::PathArguments::Parenthesized(p) = &seg.arguments {
                        let params = p.inputs.iter().map(|t| self.ty(t, self_ty)).collect();
                        let ret = match &p.output {
                            ReturnType::Default => Ty::Unit,
                            ReturnType::Type(_, t) => self.ty(t, self_ty),
                        };
                        return Ty::Fn(params, Box::new(ret));
                    }
                }
                "Iterator" | "IntoIterator" | "DoubleEndedIterator" | "ExactSizeIterator" => {
                    let mut item = Ty::Unknown;
                    if let syn::PathArguments::AngleBracketed(a) = &seg.arguments {
                        for arg in &a.args {
                            if let syn::GenericArgument::AssocType(at) = arg {
                                if at.ident == "Item" {
                                    item = self.ty(&at.ty, self_ty);
                                }
                            }
                        }
                    }
                    return Ty::Iter(Box::new(item));
                }
                "Into" | "AsRef" | "Borrow" => {
                    if let syn::PathArguments::AngleBracketed(a) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = a.args.first() {
                            return self.ty(t, self_ty);
                        }
                    }
                }
                "ToString" | "Display" => return Ty::Opaque("CustomStringConvertible".into()),
                "View" => return Ty::View,
                "Scene" => return Ty::Opaque("Scene".into()),
                "Send" | "Sync" | "Sized" | "Unpin" | "Clone" | "Copy" | "Debug" => continue,
                _ => {
                    let swift = crate::translate::protocol_name(&name)
                        .unwrap_or_else(|| names::upper_camel(&name));
                    return if opaque {
                        Ty::Opaque(swift)
                    } else {
                        Ty::Dyn(swift)
                    };
                }
            }
        }
        Ty::Unknown
    }
}

/// Replaces generic parameters by types (`F` → `(Int) -> Int`).
pub fn substitute(t: &Ty, map: &HashMap<String, Ty>) -> Ty {
    let s = |t: &Ty| substitute(t, map);
    match t {
        Ty::Named(n, args) if args.is_empty() => map.get(n).cloned().unwrap_or_else(|| t.clone()),
        Ty::Named(n, args) => Ty::Named(n.clone(), args.iter().map(s).collect()),
        Ty::Vec(a) => Ty::Vec(Box::new(s(a))),
        Ty::Option(a) => Ty::Option(Box::new(s(a))),
        Ty::Iter(a) => Ty::Iter(Box::new(s(a))),
        Ty::Set(a, o) => Ty::Set(Box::new(s(a)), *o),
        Ty::Map(k, v, o) => Ty::Map(Box::new(s(k)), Box::new(s(v)), *o),
        Ty::Result(a, b) => Ty::Result(Box::new(s(a)), Box::new(s(b))),
        Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(s).collect()),
        Ty::Fn(ps, r) => Ty::Fn(ps.iter().map(s).collect(), Box::new(s(r))),
        other => other.clone(),
    }
}

/// The name of the type an `impl` block is for (`Point` for `impl<T> Point<T>`).
pub fn type_name(t: &Type) -> Option<String> {
    match t {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        Type::Reference(r) => type_name(&r.elem),
        _ => None,
    }
}

/// The formatter parameter of `fn fmt(&self, f: &mut Formatter) -> fmt::Result`.
pub fn formatter_param(f: &syn::ImplItemFn) -> Option<String> {
    f.sig.inputs.iter().find_map(|a| match a {
        FnArg::Typed(p) => match &*p.pat {
            syn::Pat::Ident(i) => Some(i.ident.to_string()),
            _ => None,
        },
        _ => None,
    })
}

fn impl_queries_formatter(imp: &syn::ItemImpl) -> bool {
    imp.items.iter().any(|it| match it {
        syn::ImplItem::Fn(f) => f.sig.ident == "fmt" && formatter_queried(f),
        _ => false,
    })
}

/// Whether the `fmt` of a `Display` or `Debug` asks its formatter for anything beyond writing
/// text (`f.pad`, `f.precision()`, `f.alternate()`, `f.debug_struct`, `x.fmt(f)`): it is then
/// translated with a formatter, instead of as a plain `description`.
pub fn formatter_queried(f: &syn::ImplItemFn) -> bool {
    use syn::visit::Visit;

    let Some(param) = formatter_param(f) else {
        return false;
    };
    struct Finder<'a> {
        param: &'a str,
        queried: bool,
    }
    fn mentions(tokens: proc_macro2::TokenStream, param: &str) -> bool {
        tokens.into_iter().any(|t| match t {
            proc_macro2::TokenTree::Ident(i) => i == param,
            proc_macro2::TokenTree::Group(g) => mentions(g.stream(), param),
            _ => false,
        })
    }
    impl<'ast> Visit<'ast> for Finder<'_> {
        fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
            if let Expr::Path(p) = &*m.receiver {
                if p.path.is_ident(self.param) {
                    if !matches!(
                        m.method.to_string().as_str(),
                        "write_str" | "write_char" | "write_fmt"
                    ) {
                        self.queried = true;
                    }
                    for a in &m.args {
                        self.visit_expr(a);
                    }
                    return;
                }
            }
            syn::visit::visit_expr_method_call(self, m);
        }

        fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
            if p.path.is_ident(self.param) {
                self.queried = true;
            }
        }

        fn visit_macro(&mut self, mac: &'ast syn::Macro) {
            let name = mac
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            let mut tokens = mac.tokens.clone().into_iter().peekable();
            // `write!(f, …)`: the formatter as the target only writes.
            if matches!(name.as_str(), "write" | "writeln") {
                if let Some(proc_macro2::TokenTree::Ident(i)) = tokens.peek() {
                    if i == self.param {
                        tokens.next();
                    }
                }
            }
            if mentions(tokens.collect(), self.param) {
                self.queried = true;
            }
        }
    }
    let mut finder = Finder {
        param: &param,
        queried: false,
    };
    finder.visit_block(&f.block);
    finder.queried
}

/// `&mut T`, `Option<&mut T>`, `Result<&mut T, E>` as a return type.
pub fn has_mut_reference(t: &Type) -> bool {
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

pub fn returns_self_ty(ret: &Ty, self_ty: &Ty) -> bool {
    returns_self(ret, self_ty)
}

fn returns_self(ret: &Ty, self_ty: &Ty) -> bool {
    match ret {
        Ty::Option(t) => &**t == self_ty,
        Ty::Result(t, _) => &**t == self_ty,
        t => t == self_ty,
    }
}

pub fn type_params(g: &syn::Generics) -> Vec<String> {
    g.params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(t) => Some(t.ident.to_string()),
            _ => None,
        })
        .collect()
}

/// The names in `#[derive(…)]`.
pub fn derives(attrs: &[Attribute]) -> Vec<String> {
    let mut out = Vec::new();
    for a in attrs {
        if a.path().is_ident("derive") {
            let _ = a.parse_nested_meta(|m| {
                if let Some(last) = m.path.segments.last() {
                    out.push(last.ident.to_string());
                }
                Ok(())
            });
        }
    }
    out
}

fn field_role(attrs: &[Attribute]) -> FieldRole {
    for a in attrs {
        let path = a.path();
        if path.is_ident("state") {
            return match &a.meta {
                syn::Meta::List(list) => {
                    FieldRole::State(syn::parse2::<Expr>(list.tokens.clone()).ok())
                }
                _ => FieldRole::State(None),
            };
        }
        if path.is_ident("binding") {
            return FieldRole::Binding;
        }
        if path.is_ident("observation_ignored") {
            return FieldRole::Ignored;
        }
        if path.is_ident("environment") {
            if let syn::Meta::List(list) = &a.meta {
                return FieldRole::Environment(list.tokens.to_string().replace(' ', ""));
            }
        }
    }
    FieldRole::Plain
}

pub fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        Item::Fn(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Mod(i) => &i.attrs,
        Item::Static(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Trait(i) => &i.attrs,
        Item::Type(i) => &i.attrs,
        Item::Use(i) => &i.attrs,
        Item::Macro(i) => &i.attrs,
        _ => &[],
    }
}

/// `#[cfg(test)]` and `#[test]` items are not translated.
pub fn is_test_only(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        if a.path().is_ident("test") {
            return true;
        }
        if a.path().is_ident("cfg") {
            if let syn::Meta::List(list) = &a.meta {
                return list.tokens.to_string().trim() == "test";
            }
        }
        false
    })
}

/// Whether `t` holds a `name` directly or through an `Option` or a tuple (not through a
/// collection, which Swift can store).
fn directly_contains(t: &Ty, name: &str) -> bool {
    match t {
        Ty::Named(n, _) => n == name,
        Ty::Option(t) => directly_contains(t, name),
        Ty::Tuple(ts) => ts.iter().any(|t| directly_contains(t, name)),
        Ty::Result(a, b) => directly_contains(a, name) || directly_contains(b, name),
        _ => false,
    }
}
