---
title: Rust → Swift
description: The supported Rust subset and what it becomes in Swift: types, structs, enums, traits, generics, closures, pattern matching, errors and formatting.
order: 3
---

FerroSwift translates ordinary Rust — the code you write for an app's model and logic — into the Swift you would write by hand. Types are tracked expression by expression, so that each construct gets the right Swift: `Int` or `Double`, an optional, a `throws`, a binding. Try any of this in the [playground](../../playground/).

## Types

| Rust | Swift |
| --- | --- |
| `i32`, `i64`, `isize`, `usize` | `Int` |
| `i8`, `i16`, `u8`, `u16`, `u32`, `u64` | `Int8`, `Int16`, `UInt8`, `UInt16`, `UInt32`, `UInt64` |
| `f64`, `f32` | `Double`, `Float` |
| `bool`, `char` | `Bool`, `Character` |
| `String`, `&str` | `String` |
| `Vec<T>`, `[T; N]`, `&[T]` | `[T]` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `[K: V]` (a `BTreeMap` is iterated in key order) |
| `HashSet<T>`, `BTreeSet<T>` | `Set<T>` |
| `HashMap<(A, B), V>`, `HashSet<(A, B)>` | `RsTupleMap2<A, B, V>`, `RsTupleSet2<A, B>`: support types with the same methods, since Swift cannot hash a tuple (pairs and triples) |
| `Option<T>` | `T?` |
| `Result<T, E>` | a function that `throws(E)` and returns `T` |
| `(A, B)`, `()` | `(A, B)`, `Void` |
| `(A,)` | `RsTuple1<A>`, a struct of the support file (Swift has no tuple of one element) |
| `BinaryHeap<T>` | `RsBinaryHeap<T>`, a max-heap of the support file laid out like Rust's; a tuple in it, or in a `Reverse`, is stored as `RsOrd2` and given back as a tuple |
| `Box<T>`, `Rc<T>`, `Arc<T>`, `Cow<T>` | `T` |
| `RefCell<T>`, `Cell<T>`, `Mutex<T>`, `RwLock<T>` | `RsRefCell<T>`, a class holding the value: `Rc` clones share it |
| `&T`, `&mut T` | `T` (`inout T` for a `&mut` parameter; a method returning `&mut T` returns a `WritableKeyPath<Self, T>`) |
| `impl Trait`, `dyn Trait` | `some Proto`, `any Proto` |
| `fn(A) -> B`, closures | `(A) -> B` |

`usize` lengths are Swift's `Int`: `v.len()` is `v.count`. A string's `len()` counts UTF-8 bytes, as in Rust: `s.utf8.count`.

## Items

- **Structs** become `struct`s with their memberwise initializer; tuple structs get fields `_0`, `_1`… `#[derive(Default)]` gives every field its default value. A struct holding itself (`next: Option<Box<Node>>`) becomes a `final class`, since a Swift value type cannot contain itself; its `clone()` copies the whole structure and `==` compares it.
- **Derives:** `PartialEq`/`Eq` → `Equatable`, `Hash` → `Hashable`, `PartialOrd`/`Ord` → `Comparable`. `Clone` and `Copy` need nothing: Swift structs are values. `Debug` is handled by `{:?}` (see below). For enums with payloads, `==`, `hash(into:)` and `<` are written out when Swift cannot synthesize them: variants compare in declaration order, then field by field, as Rust's derives do.
- **Enums** become `enum`s; variants with data get associated values (`Card { last4: String }` → `case card(last4: String)`). Changing the fields of a variant through a `match` on a place (`Shape::Circle { r } => *r *= k`) stores the variant back at the end of the arm.
- **`impl` blocks** become `extension`s. `fn new(…) -> Self` becomes an `init`, `&mut self` methods become `mutating func`, associated `const`s become `static let`. A method returning `&mut Self` (a builder) is called as a statement, so `a.b().c()` becomes `a.b()` then `a.c()`. A method with the name of a field (`fn count(&self)` next to `count`) becomes a computed property, the field being stored as `_count`.
- **`impl Iterator for Type`** makes the type a `Sequence`: `for x in Counter { … }`, `.filter`, `.sum()` work on it. Its adapters are lazy, like `(1..)`'s, so an endless iterator ends with `take(5)`; `let mut it = …` keeps an iterator that `next()`, `peek()` and `nth()` advance.
- **Traits** become `protocol`s; default methods go into a protocol extension, associated types into `associatedtype`. `impl Trait for Type` adds the conformance.
- **Operator traits** (`Add`, `Sub`, `Mul`, `Neg`…) become operator functions, `impl Index` a `subscript` (with a `set` when `IndexMut` is implemented too); `impl Display` becomes `CustomStringConvertible`.
- **Names** follow Swift's conventions (`my_field` → `myField`, `MAX_SIZE` → `maxSize`); a type named like one of Swift's own (`enum Error`, `struct Task`) gets a `Type` suffix (`ErrorType`), and `type Result<T> = std::result::Result<T, Error>` becomes a generic `typealias`.
- **Conversions:** `impl From<A> for B` becomes an `init`; `impl FromStr for B` and `impl TryFrom<A> for B` become the static functions `fromStr` and `tryFrom`, which `s.parse::<B>()`, `B::from_str(s)`, `B::try_from(x)` and `x.try_into()` call.
- **Generics** keep their bounds: `fn largest<T: Describe>(items: &[T])` → `func largest<T: Describe>(_ items: [T])`. `where` clauses are kept.
- **Functions** take unlabeled parameters (`_ name: Type`), so calls look like Rust's.
- `const` and `static` become global `let`s, `type` aliases become `typealias`es, `mod` blocks are flattened.

## Expressions and control flow

| Rust | Swift |
| --- | --- |
| `match x { A => …, B(v) if v > 0 => …, _ => … }` | `switch x { case .a: … case .b(let v) where v > 0: … default: … }` |
| `if let Some(x) = opt { … }` | `if let x = opt { … }` |
| `let Some(x) = opt else { return };` | `guard let x = opt else { return }` |
| `while let Some(x) = stack.pop()` | `while let x = stack.popLast()` |
| `for (i, x) in v.iter().enumerate()` | `for (i, x) in v.enumerated()` |
| `for i in 0..n`, `0..=n`, `(0..n).rev()` | `for i in 0..<n`, `0...n`, `(0..<n).reversed()` |
| `if c { a } else { b }` as a value | `c ? a : b` |
| `v.push(match op { '+' => a + b, … })` | `v.append({ () -> Int in switch op { … } }())` |
| `x as f64`, `n as u8` | `Double(x)`, `UInt8(truncatingIfNeeded: n)` |
| `matches!(x, Some(1..=5))` | a `switch` that returns a `Bool` |
| `assert!(c, "…")`, `panic!("…")` | `precondition(c, "…")`, `fatalError("…")` |

Closures become Swift closures (`|x| x * 2` → `{ x in x * 2 }`), and iterator chains become the matching Swift methods: `map`, `filter`, `fold` (→ `reduce`), `sum`, `any` (→ `contains(where:)`), `find` (→ `first(where:)`), `position` (→ `firstIndex`), `enumerate`, `zip`, `rev`, `max_by`, `collect`… A few methods without a direct equivalent come from the support file.

## Errors

A function returning `Result<T, E>` becomes a function that `throws(E)`; `?` becomes `try`, `Err(e)` a `throw`, and a `match` on a `Result` catches the error. An error that is a `String`, a number, a `bool` or a `char` is thrown as that value (the support file makes the type an `Error`). A `main` returning an `Err` prints `Error: {:?}` on the standard error and exits with 1, as in Rust:

```rust
fn check(name: &str) -> Result<&str, LoginError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(LoginError::EmptyName);
    }
    Ok(name)
}
```

```swift
func check(_ name: String) throws(LoginError) -> String {
    let name = name.trimmingCharacters(in: .whitespacesAndNewlines)
    if name.isEmpty {
        throw .emptyName
    }
    return name
}
```

Error enums conform to `Error`. Typed `throws` needs Swift 6 (Xcode 16).

`s.parse::<i32>()` (the target may also be a later use of the value, as in `let n = s.parse().ok()?;` followed by `Point { x: n }`) and the other conversions of the standard library (`i32::from_str_radix`, `u8::try_from`, `x.try_into()`, `String::from_utf8`, `char::try_from`) return `Result`s whose errors are Rust's, in `{:?}` as in `{}`: `Err(ParseIntError { kind: InvalidDigit })`, `invalid digit found in string`, `PosOverflow` for a number too large for the type, `TryFromIntError(NegOverflow)`. A `?` on an error that converts into the function's (`impl From<ParseIntError> for AppError`) converts it the same way. `v.last_mut().unwrap().done = true`, `map.get_mut(&k).unwrap().push(x)`, `if let Some(t) = v.iter_mut().find(…)` and `v.iter_mut().for_each(…)` change the element in place, as in Rust.

## Strings and formatting

`format!`, `println!`, `eprintln!` and `write!` become string interpolation. Values print exactly as in Rust thanks to small functions of the support file: `{}` of `1.0` is `1`, `{:.2}` rounds the same way, `{:?}` writes `Some(Point { x: 1.0, y: 2.5 })`, and widths, alignment and padding (`{:>8}`, `{:05}`) are kept — in `{:>7?}` they reach the numbers inside the value, as with Rust's derived `Debug`. Common string methods have their Swift equivalent (`trim` → `trimmingCharacters(in:)`, `split` → `components(separatedBy:)`, `to_uppercase` → `uppercased()`, `starts_with` → `hasPrefix`…). `s.chars()` iterates Unicode scalars, as Rust does (`s.rsChars`): `chars().count()`, `rev()` and `nth()` agree with Rust on combining marks and emoji sequences, which Swift's own iteration groups into one character. The pattern of `starts_with`, `split`, `find`, `trim_matches`… can be a text, a character, a closure or an array of characters; `s.get(a..b)` is `None` off a character boundary; `to_ascii_uppercase` changes only the ASCII letters, as in Rust.

## Comments

Comments and `///` documentation are kept, next to the code they describe, so the generated Swift stays readable: before and after statements, between the children and the modifiers of a view, and blank lines between groups of statements stay. Comments inside other chains of method calls (`v.iter() // … .map(…)`) are not kept.

## The support file

When the translation needs a helper — Rust formatting, `Ordering`, string slicing by bytes, a few collection methods (`retain`, `dedup`, `windows`…) — FerroSwift writes it once in `FerroSwiftSupport.swift` (or at the end of the file in the playground). Only the helpers you use are included.
