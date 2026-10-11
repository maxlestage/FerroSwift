//! The types the translator reasons about, read from Rust and written as Swift.

use std::fmt::Write;

use crate::names;

/// Integer types. `i32`, `i64`, `isize` and `usize` all become Swift's `Int` (64 bits on every
/// Apple platform), the others keep their size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntKind {
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

impl IntKind {
    pub fn parse(name: &str) -> Option<IntKind> {
        Some(match name {
            "i8" => IntKind::I8,
            "i16" => IntKind::I16,
            "i32" => IntKind::I32,
            "i64" => IntKind::I64,
            "i128" => IntKind::I128,
            "isize" => IntKind::Isize,
            "u8" => IntKind::U8,
            "u16" => IntKind::U16,
            "u32" => IntKind::U32,
            "u64" => IntKind::U64,
            "u128" => IntKind::U128,
            "usize" => IntKind::Usize,
            _ => return None,
        })
    }

    pub fn swift(self) -> &'static str {
        match self {
            IntKind::I32 | IntKind::I64 | IntKind::Isize | IntKind::Usize => "Int",
            IntKind::I8 => "Int8",
            IntKind::I16 => "Int16",
            IntKind::I128 => "Int128",
            IntKind::U8 => "UInt8",
            IntKind::U16 => "UInt16",
            IntKind::U32 => "UInt32",
            IntKind::U64 => "UInt64",
            IntKind::U128 => "UInt128",
        }
    }

    /// Bits of the Swift type.
    pub fn bits(self) -> u32 {
        match self.swift() {
            "Int8" | "UInt8" => 8,
            "Int16" | "UInt16" => 16,
            "UInt32" => 32,
            "Int128" | "UInt128" => 128,
            _ => 64,
        }
    }

    pub fn signed(self) -> bool {
        !self.swift().starts_with('U')
    }

    /// Bits of the Rust type.
    pub fn rust_bits(self) -> u32 {
        match self {
            IntKind::I8 | IntKind::U8 => 8,
            IntKind::I16 | IntKind::U16 => 16,
            IntKind::I32 | IntKind::U32 => 32,
            IntKind::I64 | IntKind::U64 | IntKind::Isize | IntKind::Usize => 64,
            IntKind::I128 | IntKind::U128 => 128,
        }
    }

    /// The bits of the Rust type when Swift's is wider (`i32` in `Int`): the limits to keep.
    pub fn narrow_bits(self) -> Option<u32> {
        let rust = self.rust_bits();
        (rust < self.bits()).then_some(rust)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatKind {
    F32,
    F64,
}

/// A type, as far as the translator knows it.
#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Unknown,
    Unit,
    Never,
    Bool,
    Char,
    /// `String` and `&str`.
    Str,
    Int(IntKind),
    Float(FloatKind),
    /// `Vec<T>`, `[T; N]`, `&[T]`, `VecDeque<T>`.
    Vec(Box<Ty>),
    /// `HashMap<K, V>`; `true` for `BTreeMap` (iterated in key order).
    Map(Box<Ty>, Box<Ty>, bool),
    /// `HashSet<T>`; `true` for `BTreeSet`.
    Set(Box<Ty>, bool),
    Option(Box<Ty>),
    Result(Box<Ty>, Box<Ty>),
    Tuple(Vec<Ty>),
    /// Closures and function pointers: parameters, result.
    Fn(Vec<Ty>, Box<Ty>),
    /// `a..b`, `a..=b`.
    Range(Box<Ty>),
    /// A lazy iterator over items of this type.
    Iter(Box<Ty>),
    /// A user type, a generic parameter, or a library type, with its arguments.
    Named(String, Vec<Ty>),
    /// `dyn Trait`: `any Trait`.
    Dyn(String),
    /// `impl Trait` in argument or return position: `some Trait`.
    Opaque(String),
    /// A SwiftUI view built with the ferroswift functions.
    View,
}

impl Ty {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Ty::Unknown)
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int(_) | Ty::Float(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Ty::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Ty::Float(_))
    }

    pub fn is_option(&self) -> bool {
        matches!(self, Ty::Option(_))
    }

    pub fn is_result(&self) -> bool {
        matches!(self, Ty::Result(..))
    }

    pub fn named(&self) -> Option<&str> {
        match self {
            Ty::Named(n, _) => Some(n),
            _ => None,
        }
    }

    /// The items of a collection or an iterator.
    pub fn item(&self) -> Ty {
        match self {
            Ty::Vec(t) | Ty::Set(t, _) | Ty::Iter(t) | Ty::Range(t) | Ty::Option(t) => {
                (**t).clone()
            }
            Ty::Map(k, v, _) => Ty::Tuple(vec![(**k).clone(), (**v).clone()]),
            Ty::Str => Ty::Char,
            Ty::Named(n, args) if n == "RsBinaryHeap" || n == "RsIter" => {
                args.first().cloned().unwrap_or(Ty::Unknown)
            }
            _ => Ty::Unknown,
        }
    }

    /// Whether the type is complete enough to be written in Swift.
    pub fn is_complete(&self) -> bool {
        match self {
            Ty::Unknown => false,
            Ty::Vec(t) | Ty::Set(t, _) | Ty::Option(t) | Ty::Range(t) | Ty::Iter(t) => {
                t.is_complete()
            }
            Ty::Map(k, v, _) | Ty::Result(k, v) => k.is_complete() && v.is_complete(),
            Ty::Tuple(ts) => ts.iter().all(Ty::is_complete),
            Ty::Fn(ps, r) => ps.iter().all(Ty::is_complete) && r.is_complete(),
            Ty::Named(_, args) => args.iter().all(Ty::is_complete),
            _ => true,
        }
    }

    /// Fills the unknown parts of `self` with `other`.
    pub fn merge(&mut self, other: &Ty) {
        match (&mut *self, other) {
            (_, Ty::Unknown) => {}
            (Ty::Unknown, o) => *self = o.clone(),
            (Ty::Vec(a), Ty::Vec(b))
            | (Ty::Set(a, _), Ty::Set(b, _))
            | (Ty::Option(a), Ty::Option(b))
            | (Ty::Range(a), Ty::Range(b))
            | (Ty::Iter(a), Ty::Iter(b)) => a.merge(b),
            (Ty::Map(a, b, _), Ty::Map(c, d, _)) | (Ty::Result(a, b), Ty::Result(c, d)) => {
                a.merge(c);
                b.merge(d);
            }
            (Ty::Tuple(a), Ty::Tuple(b)) if a.len() == b.len() => {
                for (x, y) in a.iter_mut().zip(b) {
                    x.merge(y);
                }
            }
            (Ty::Named(n, a), Ty::Named(m, b)) if n == m && a.len() == b.len() => {
                for (x, y) in a.iter_mut().zip(b) {
                    x.merge(y);
                }
            }
            _ => {}
        }
    }

    /// The Swift spelling, `None` while some part is unknown.
    pub fn swift(&self) -> Option<String> {
        if !self.is_complete() {
            return None;
        }
        let mut out = String::new();
        self.write_swift(&mut out);
        Some(out)
    }

    /// The Swift spelling, with `_` for unknown parts (for messages).
    pub fn swift_lossy(&self) -> String {
        let mut out = String::new();
        self.write_swift(&mut out);
        out
    }

    fn write_swift(&self, out: &mut String) {
        match self {
            Ty::Unknown => out.push('_'),
            Ty::Unit => out.push_str("Void"),
            Ty::Never => out.push_str("Never"),
            Ty::Bool => out.push_str("Bool"),
            Ty::Char => out.push_str("Character"),
            Ty::Str => out.push_str("String"),
            Ty::Int(k) => out.push_str(k.swift()),
            Ty::Float(FloatKind::F32) => out.push_str("Float"),
            Ty::Float(FloatKind::F64) => out.push_str("Double"),
            Ty::Vec(t) | Ty::Iter(t) => {
                out.push('[');
                t.write_swift(out);
                out.push(']');
            }
            // Swift cannot hash a tuple: keys of two or three elements get a support type.
            Ty::Map(k, v, _) if matches!(&**k, Ty::Tuple(ts) if ts.len() == 2 || ts.len() == 3) => {
                let Ty::Tuple(ts) = &**k else { unreachable!() };
                let _ = write!(out, "RsTupleMap{}<", ts.len());
                for t in ts {
                    t.write_swift(out);
                    out.push_str(", ");
                }
                v.write_swift(out);
                out.push('>');
            }
            Ty::Map(k, v, _) => {
                out.push('[');
                k.write_swift(out);
                out.push_str(": ");
                v.write_swift(out);
                out.push(']');
            }
            Ty::Set(t, _) if matches!(&**t, Ty::Tuple(ts) if ts.len() == 2 || ts.len() == 3) => {
                let Ty::Tuple(ts) = &**t else { unreachable!() };
                let _ = write!(out, "RsTupleSet{}<", ts.len());
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    t.write_swift(out);
                }
                out.push('>');
            }
            Ty::Set(t, _) => {
                out.push_str("Set<");
                t.write_swift(out);
                out.push('>');
            }
            Ty::Option(t) => {
                let wrap = matches!(**t, Ty::Fn(..) | Ty::Dyn(_) | Ty::Opaque(_));
                if wrap {
                    out.push('(');
                }
                t.write_swift(out);
                if wrap {
                    out.push(')');
                }
                out.push('?');
            }
            Ty::Result(t, e) => {
                out.push_str("Result<");
                t.write_swift(out);
                out.push_str(", ");
                e.write_swift(out);
                out.push('>');
            }
            // Swift has no tuple of one element: a support struct holds it.
            Ty::Tuple(ts) if ts.len() == 1 => {
                out.push_str("RsTuple1<");
                ts[0].write_swift(out);
                out.push('>');
            }
            Ty::Tuple(ts) => {
                out.push('(');
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    t.write_swift(out);
                }
                out.push(')');
            }
            Ty::Fn(ps, r) => {
                out.push('(');
                for (i, t) in ps.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    t.write_swift(out);
                }
                out.push_str(") -> ");
                r.write_swift(out);
            }
            Ty::Range(t) => {
                out.push_str("Range<");
                t.write_swift(out);
                out.push('>');
            }
            // `Reverse<(A, B)>`, `BinaryHeap<(A, B)>`: the tuple made comparable.
            Ty::Named(n, args)
                if (n == "RsReverse" || n == "RsBinaryHeap")
                    && matches!(args.first(), Some(Ty::Tuple(ts)) if ts.len() == 2 || ts.len() == 3) =>
            {
                let Some(Ty::Tuple(ts)) = args.first() else {
                    unreachable!()
                };
                let _ = write!(out, "{n}<RsOrd{}<", ts.len());
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    t.write_swift(out);
                }
                out.push_str(">>");
            }
            Ty::Named(n, args) => {
                out.push_str(n);
                if !args.is_empty() {
                    out.push('<');
                    for (i, t) in args.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        t.write_swift(out);
                    }
                    out.push('>');
                }
            }
            Ty::Dyn(name) => {
                let _ = write!(out, "any {name}");
            }
            Ty::Opaque(name) => {
                let _ = write!(out, "some {name}");
            }
            Ty::View => out.push_str("some View"),
        }
    }

    /// A Swift expression of the default value (`Default::default()`), if there is one.
    pub fn default_value(&self) -> Option<String> {
        Some(match self {
            Ty::Bool => "false".into(),
            Ty::Char => "Character(\"\\0\")".into(),
            Ty::Str => "\"\"".into(),
            Ty::Int(_) => "0".into(),
            Ty::Float(_) => "0.0".into(),
            Ty::Vec(_) | Ty::Set(..) => "[]".into(),
            Ty::Map(..) => "[:]".into(),
            Ty::Option(_) => "nil".into(),
            Ty::Unit => "()".into(),
            Ty::Tuple(ts) if ts.len() == 1 => format!("RsTuple1({})", ts[0].default_value()?),
            Ty::Tuple(ts) => {
                let parts: Option<Vec<String>> = ts.iter().map(Ty::default_value).collect();
                format!("({})", parts?.join(", "))
            }
            Ty::Named(n, args) if n == "RsRefCell" => {
                format!("RsRefCell({})", args.first()?.default_value()?)
            }
            Ty::Named(n, _) => format!("{n}()"),
            _ => return None,
        })
    }
}

/// `(A, B)` or `(A, B, C)`: a tuple that a `BinaryHeap` or a `Reverse` must compare.
pub fn ord_tuple_arity(t: &Ty) -> Option<usize> {
    match t {
        Ty::Tuple(ts) if ts.len() == 2 || ts.len() == 3 => Some(ts.len()),
        _ => None,
    }
}

/// The Swift type that builds a set of these elements: `Set`, or the support type for
/// tuples.
pub fn set_type_name(item: &Ty) -> String {
    match item {
        Ty::Tuple(ts) if ts.len() == 2 || ts.len() == 3 => format!("RsTupleSet{}", ts.len()),
        _ => "Set".into(),
    }
}

/// The Swift type that builds a map with these keys: `Dictionary`, or the support type for
/// tuples.
pub fn map_type_name(key: &Ty) -> String {
    match key {
        Ty::Tuple(ts) if ts.len() == 2 || ts.len() == 3 => format!("RsTupleMap{}", ts.len()),
        _ => "Dictionary".into(),
    }
}

/// Names of the generic parameters in scope, which stay as they are.
pub fn is_primitive_name(name: &str) -> bool {
    matches!(name, "bool" | "char" | "str" | "String" | "f32" | "f64")
        || IntKind::parse(name).is_some()
}

/// Reads a type name as Rust writes it in paths (`i32`, `String`, `Vec`…) with arguments.
pub fn from_name(name: &str, args: Vec<Ty>) -> Ty {
    let arg = |i: usize| args.get(i).cloned().unwrap_or(Ty::Unknown);
    if let Some(k) = IntKind::parse(name) {
        return Ty::Int(k);
    }
    match name {
        "bool" => Ty::Bool,
        "char" => Ty::Char,
        "str" | "String" | "OsString" | "PathBuf" | "Path" => Ty::Str,
        "f32" => Ty::Float(FloatKind::F32),
        "f64" => Ty::Float(FloatKind::F64),
        "Vec" | "VecDeque" | "LinkedList" => Ty::Vec(Box::new(arg(0))),
        "BinaryHeap" => Ty::Named("RsBinaryHeap".into(), vec![arg(0)]),
        "HashMap" | "IndexMap" => Ty::Map(Box::new(arg(0)), Box::new(arg(1)), false),
        "BTreeMap" => Ty::Map(Box::new(arg(0)), Box::new(arg(1)), true),
        "HashSet" | "IndexSet" if !args.is_empty() => Ty::Set(Box::new(arg(0)), false),
        "BTreeSet" => Ty::Set(Box::new(arg(0)), true),
        "Option" => Ty::Option(Box::new(arg(0))),
        "Result" => Ty::Result(
            Box::new(arg(0)),
            Box::new(if args.len() > 1 { arg(1) } else { Ty::Unknown }),
        ),
        // Smart pointers disappear: Swift values are copied, classes shared.
        "Box" | "Rc" | "Arc" | "Cow" => arg(0),
        // The iterators of the standard library kept in a field or a variable: `RsIter`.
        "Peekable" | "Fuse" => match arg(0) {
            Ty::Iter(t) => Ty::Named("RsIter".into(), vec![*t]),
            Ty::Named(n, args) if n == "RsIter" => Ty::Named(n, args),
            t => Ty::Named("RsIter".into(), vec![t.item()]),
        },
        "Chars" => Ty::Named("RsIter".into(), vec![Ty::Char]),
        "CharIndices" => Ty::Named(
            "RsIter".into(),
            vec![Ty::Tuple(vec![Ty::Int(IntKind::Usize), Ty::Char])],
        ),
        "Bytes" => Ty::Named("RsIter".into(), vec![Ty::Int(IntKind::U8)]),
        "Lines" | "SplitWhitespace" | "SplitAsciiWhitespace" => {
            Ty::Named("RsIter".into(), vec![Ty::Str])
        }
        "Iter" | "IntoIter" | "Drain" if !args.is_empty() => {
            Ty::Named("RsIter".into(), vec![arg(0)])
        }
        // A cell is a class holding the value: the clones of an `Rc` share it.
        "RefCell" | "Cell" | "Mutex" | "RwLock" => Ty::Named("RsRefCell".into(), vec![arg(0)]),
        // The errors of the standard library have support types of the same shape.
        "ParseIntError" | "ParseFloatError" | "ParseBoolError" | "ParseCharError"
        | "CharTryFromError" | "TryFromIntError" | "FromUtf8Error" | "Utf8Error" => {
            Ty::Named(format!("Rs{name}"), vec![])
        }
        _ => Ty::Named(names::upper_camel(name), args),
    }
}
