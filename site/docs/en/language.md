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
| `Option<T>` | `T?` |
| `Result<T, E>` | a function that `throws(E)` and returns `T` |
| `(A, B)`, `()` | `(A, B)`, `Void` |
| `Box<T>`, `Rc<T>`, `Arc<T>`, `Cow<T>` | `T` |
| `&T`, `&mut T` | `T` (`inout T` for a `&mut` parameter) |
| `impl Trait`, `dyn Trait` | `some Proto`, `any Proto` |
| `fn(A) -> B`, closures | `(A) -> B` |

`usize` lengths are Swift's `Int`: `v.len()` is `v.count`. A string's `len()` counts UTF-8 bytes, as in Rust: `s.utf8.count`.

## Items

- **Structs** become `struct`s with their memberwise initializer; tuple structs get fields `_0`, `_1`… `#[derive(Default)]` gives every field its default value. A struct holding itself (`next: Option<Box<Node>>`) becomes a `final class`, since a Swift value type cannot contain itself; its `clone()` copies the whole structure and `==` compares it.
- **Derives:** `PartialEq`/`Eq` → `Equatable`, `Hash` → `Hashable`, `PartialOrd`/`Ord` → `Comparable`. `Clone` and `Copy` need nothing: Swift structs are values. `Debug` is handled by `{:?}` (see below).
- **Enums** become `enum`s; variants with data get associated values (`Card { last4: String }` → `case card(last4: String)`).
- **`impl` blocks** become `extension`s. `fn new(…) -> Self` becomes an `init`, `&mut self` methods become `mutating func`, associated `const`s become `static let`. A method returning `&mut Self` (a builder) is called as a statement, so `a.b().c()` becomes `a.b()` then `a.c()`. A method with the name of a field (`fn count(&self)` next to `count`) becomes a computed property, the field being stored as `_count`.
- **`impl Iterator for Type`** makes the type a `Sequence`: `for x in Counter { … }`, `.filter`, `.sum()` work on it.
- **Traits** become `protocol`s; default methods go into a protocol extension, associated types into `associatedtype`. `impl Trait for Type` adds the conformance.
- **Operator traits** (`Add`, `Sub`, `Mul`, `Neg`…) become operator functions; `impl Display` becomes `CustomStringConvertible`.
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
| `x as f64`, `n as u8` | `Double(x)`, `UInt8(truncatingIfNeeded: n)` |
| `matches!(x, Some(1..=5))` | a `switch` that returns a `Bool` |
| `assert!(c, "…")`, `panic!("…")` | `precondition(c, "…")`, `fatalError("…")` |

Closures become Swift closures (`|x| x * 2` → `{ x in x * 2 }`), and iterator chains become the matching Swift methods: `map`, `filter`, `fold` (→ `reduce`), `sum`, `any` (→ `contains(where:)`), `find` (→ `first(where:)`), `position` (→ `firstIndex`), `enumerate`, `zip`, `rev`, `max_by`, `collect`… A few methods without a direct equivalent come from the support file.

## Errors

A function returning `Result<T, E>` becomes a function that `throws(E)`; `?` becomes `try`, `Err(e)` a `throw`, and a `match` on a `Result` catches the error:

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

## Strings and formatting

`format!`, `println!`, `eprintln!` and `write!` become string interpolation. Values print exactly as in Rust thanks to small functions of the support file: `{}` of `1.0` is `1`, `{:.2}` rounds the same way, `{:?}` writes `Some(Point { x: 1.0, y: 2.5 })`, and widths, alignment and padding (`{:>8}`, `{:05}`) are kept. Common string methods have their Swift equivalent (`trim` → `trimmingCharacters(in:)`, `split` → `components(separatedBy:)`, `to_uppercase` → `uppercased()`, `starts_with` → `hasPrefix`…).

## Comments

Comments and `///` documentation are kept, next to the code they describe, so the generated Swift stays readable: before and after statements, between the children and the modifiers of a view, and blank lines between groups of statements stay. Comments inside other chains of method calls (`v.iter() // … .map(…)`) are not kept.

## The support file

When the translation needs a helper — Rust formatting, `Ordering`, string slicing by bytes, a few collection methods (`retain`, `dedup`, `windows`…) — FerroSwift writes it once in `FerroSwiftSupport.swift` (or at the end of the file in the playground). Only the helpers you use are included.
