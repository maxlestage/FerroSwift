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
| `(A,)` | `RsTuple1<A>`, a struct of the support file (Swift has no tuple of one element), with `.0`, `{:?}` (`(5,)`), comparisons, hashing, and the patterns `(x,)` in `let`, parameters, `match` arms and payloads (`Some((c,))`) |
| `BinaryHeap<T>` | `RsBinaryHeap<T>`, a max-heap of the support file laid out like Rust's; a tuple in it, or in a `Reverse`, is stored as `RsOrd2` and given back as a tuple |
| `Box<T>`, `Rc<T>`, `Arc<T>` | `T` |
| `Cow<'a, T>` | `RsCow<T>`, an enum of the support file (`.borrowed(T)`, `.owned(T)`): `match` on `Cow::Borrowed`/`Cow::Owned`, `to_mut()`, `into_owned()`; the value's methods, `{}`, `{:?}` and `==` go through it |
| `VecDeque<T>` | `[T]`, its `remove(i)`, `swap_remove_back(i)` and `swap_remove_front(i)` giving an `Option` as in Rust |
| `struct S<I: Iterator<Item = T>> { it: I }` | `struct S { var it: RsIter<T> }`: the iterator parameter is erased, a value given to the field wrapped |
| `Range<T>`, `RangeInclusive<T>` | `Range<T>`, `ClosedRange<T>`: `r.start()`, `r.end`, `r.len()`, `r.contains(&x)` and `{:?}` (`1..5`, `1..=5`) as in Rust |
| `RefCell<T>`, `Cell<T>`, `Mutex<T>`, `RwLock<T>` | `RsRefCell<T>`, a class holding the value: `Rc` clones share it |
| `&T`, `&mut T` | `T` (`inout T` for a `&mut` parameter; a method returning `&mut T` returns a `WritableKeyPath<Self, T>`) |
| `std::time::Duration` | `RsDuration` of the support file: seconds and nanoseconds, with Rust's constructors, accessors, operators and `{:?}` (`1.5s`, `250ms`) |
| `&mut v[a..b]` | a copy of the part, written back after the call; `chunks_mut` and `split_at_mut` work the same way, their parts written back when the loop iteration ends or after their last use |
| `impl Trait`, `dyn Trait` | `some Proto`, `any Proto` |
| `fn(A) -> B`, closures | `(A) -> B` |

`usize` lengths are Swift's `Int`: `v.len()` is `v.count`. A string's `len()` counts UTF-8 bytes, as in Rust: `s.utf8.count`.

## Items

- **Structs** become `struct`s with their memberwise initializer; tuple structs get fields `_0`, `_1`… `#[derive(Default)]` gives every field its default value. A struct holding itself (`next: Option<Box<Node>>`) becomes a `final class`, since a Swift value type cannot contain itself; its `clone()` copies the whole structure and `==` compares it.
- **Derives:** `PartialEq`/`Eq` → `Equatable`, `Hash` → `Hashable`, `PartialOrd`/`Ord` → `Comparable`. `Clone` and `Copy` need nothing: Swift structs are values. `Debug` is handled by `{:?}` (see below). For enums with payloads, `==`, `hash(into:)` and `<` are written out when Swift cannot synthesize them: variants compare in declaration order, then field by field, as Rust's derives do. `Vec`s compare element by element (`v1 < v2`, `rows.sort()` on a `Vec<Vec<i32>>`, a `(usize, Vec<char>)` sort key), options with `None` first, as in Rust.
- **Enums** become `enum`s; variants with data get associated values (`Card { last4: String }` → `case card(last4: String)`). An enum without payloads is `Int`-backed, so `Suit::Hearts as u8` is its rank, as in Rust. Changing the fields of a variant through a `match` on a place (`Shape::Circle { r } => *r *= k`) stores the variant back at the end of the arm.
- **`impl` blocks** become `extension`s. `fn new(…) -> Self` becomes an `init`, `&mut self` methods become `mutating func`, associated `const`s become `static let`. A method returning `&mut Self` (a builder) is called as a statement, so `a.b().c()` becomes `a.b()` then `a.c()`. A method with the name of a field (`fn count(&self)` next to `count`) becomes a computed property, the field being stored as `_count`.
- **`impl Iterator for Type`** makes the type a `Sequence`: `for x in Counter { … }`, `.filter`, `.sum()` work on it. Its adapters are lazy, like `(1..)`'s, so an endless iterator ends with `take(5)`; `let mut it = …` keeps an iterator that `next()`, `peek()` and `nth()` advance.
- **Traits** become `protocol`s; default methods go into a protocol extension, associated types into `associatedtype`. `impl Trait for Type` adds the conformance. A trait with type parameters (`trait Converter<T>`) has them as associated types, which `impl Converter<f64> for Celsius` fixes (`typealias T = Double`); a second impl of the same trait for the type (`impl Converter<i32> for Celsius`) is a plain extension, its functions overloads of the first's. A method called on a `dyn Trait` value has the signature the trait declares. `impl<T: Describe> Describe for Vec<T>` (or `Option<T>`, a map) extends `Array` (`Optional`, `Dictionary`) under `where Element: Describe` (`Wrapped`, `Key`, `Value`).
- **`impl Deref for Name`** (`type Target = String`) becomes `func deref() -> String`: `name.len()` and the other methods of the target are called on `name.deref()`, as is `*name` and a `&name` given to a `&str` parameter. `DerefMut` adds nothing: changes go through the field itself.
- **Operator traits** (`Add`, `Sub`, `Mul`, `Neg`, `BitOr`, `Shl`, the `…Assign` forms…) become operator functions (an operator written on `mut self` changes a copy of its left side; `impl PartialEq<f64> for Matrix` is an `==` between the two types), `impl fmt::Binary` (`LowerHex`, `UpperHex`, `Octal`) the text `{:b}` (`{:x}`, `{:X}`, `{:o}`) writes, `impl Index` a `subscript` (with a `set` when `IndexMut` is implemented too); `impl Display` becomes `CustomStringConvertible`; `impl IntoIterator` makes the type a `Sequence`, `impl FromIterator` its `fromIter` (called by `collect`), `impl Hash` its `hash(into:)`, `impl Extend` and `impl AsRef<str>` plain functions. An argument given to an `impl Into<T>` parameter is converted at the call as `From` gives it (`Value(1)`, `Value([1, 2].map { Value($0) })` for an `impl From<Vec<T>>`). A `fmt` that asks its formatter for the spec (`f.pad(…)`, `f.precision()`, `f.alternate()`, `self.0.fmt(f)`) or lays out a `Debug` with `f.debug_struct(…)` and the other builders is written with a small `RsFormatter` of the support file, so that `{:>8.2}` and `{:#?}` reach it as in Rust.
- **Generic bounds** become protocol constraints (`T: Display` → `T: CustomStringConvertible`, `T: PartialOrd` → `T: Comparable`); `T: Add<Output = T>`, `T: Sub<Output = T>` → `AdditiveArithmetic`, `T: Mul<Output = T>` → `Numeric`); `T: Default` and `T: Into<f64>` use two small protocols of the support file (`RsDefault`, `RsIntoDouble`), which the standard types adopt, and a generic struct deriving `Default` gets its `init()` under `where T: RsDefault`.
- **Names** follow Swift's conventions (`my_field` → `myField`, `MAX_SIZE` → `maxSize`); a type named like one of Swift's own (`enum Error`, `struct Task`) gets a `Type` suffix (`ErrorType`), and `type Result<T> = std::result::Result<T, Error>` becomes a generic `typealias`.
- **Conversions:** `impl From<A> for B` becomes an `init`; `impl FromStr for B` and `impl TryFrom<A> for B` become the static functions `fromStr` and `tryFrom`, which `s.parse::<B>()`, `B::from_str(s)`, `B::try_from(x)` and `x.try_into()` call. On an `Any`, `x.downcast_ref::<T>()` is `x as? T` and `x.is::<T>()` is `x is T`.
- **Generics** keep their bounds: `fn largest<T: Describe>(items: &[T])` → `func largest<T: Describe>(_ items: [T])`. `where` clauses are kept. A type parameter that neither a parameter nor the result mentions (`fn report<T: FromStr>(inputs: &[&str])`, called as `report::<i32>(…)`) is a trailing `T.Type` parameter, given at the call (`report(inputs, Int.self)`); `s.parse::<T>()` in such code reads the text as Rust reads the type `T` stands for, within its limits (a hidden `rsBitsT` argument tells the callee that `T` is an `i32`, which Swift widens to `Int`), and fails with that type's error (`T::Err`), which `{:?}` writes as Rust does (`ParseIntError { kind: PosOverflow }`).
- **Functions called through their type** (`str::len(s)`, `f64::max(a, b)`, `Option::is_some(&o)`, `i32::abs(x)`) are the method on their receiver; `<Square as Shape>::name()` is the type's function, `Square.name()`.
- **Functions** take unlabeled parameters (`_ name: Type`), so calls look like Rust's.
- `const` and `static` become global `let`s (local constants when declared inside a function), `type` aliases become `typealias`es, `mod` blocks are flattened.

## Expressions and control flow

| Rust | Swift |
| --- | --- |
| `match x { A => …, B(v) if v > 0 => …, _ => … }` | `switch x { case .a: … case .b(let v) where v > 0: … default: … }` |
| `if let Some(x) = opt { … }` | `if let x = opt { … }` |
| `let Some(x) = opt else { return };` | `guard let x = opt else { return }` |
| `while let Some(x) = stack.pop()` | `while let x = stack.popLast()` |
| `for (i, x) in v.iter().enumerate()` | `for (i, x) in v.enumerated()` |
| `for x in v.iter_mut()`, `v.iter_mut().filter(\|x\| x.alive)` | `for i in v.indices { v[i]… }`, `for i in v.indices where v[i].alive { … }`: the elements changed in place |
| `for i in 0..n`, `0..=n`, `(0..n).rev()` | `for i in 0..<n`, `0...n`, `(0..<n).reversed()`; a range with computed bounds (`a..b`, `1..=n`) is `a..<max(a, b)`, empty when its end comes first as in Rust, where Swift's range would trap |
| `if c { a } else { b }` as a value | `c ? a : b` |
| `v.push(match op { '+' => a + b, … })` | `v.append({ () -> Int in switch op { … } }())` |
| `[Token::Word(cmd), rest @ ..] if cmd == "set"` | `case _ where tokens.count >= 1 && { if case .word(_) = tokens[0] … }()`, the bindings taken at the start of the arm (`guard case .word(let cmd) = tokens[0]`), the guard evaluated once they are bound |
| `let f = \|out: &mut Vec<T>\| …; f(&mut v)` | `{ (out: inout [T]) in … }`, `f(&v)` |
| `Ok(match e { … => return Err(…) })`, `let (a, b) = match op { … => break }` | the value is computed by statements before its use (`let value: Int` then a `switch` assigning it): a `return`, `break` or `?` in a branch acts on the function or the loop |
| `x as f64`, `n as u8` | `Double(x)`, `UInt8(truncatingIfNeeded: n)` |
| `matches!(x, Some(1..=5))` | a `switch` that returns a `Bool` |
| `assert!(c, "…")`, `panic!("…")` | `precondition(c, "…")`, `fatalError("…")` |

Closures become Swift closures (`|x| x * 2` → `{ x in x * 2 }`); a struct generic over a closure type (`struct Button<F: Fn()> { action: F }`) stores the closure type itself, and a `move` closure changing a `Copy` variable it captured (`move || { counter += 1; counter }`) owns a copy of it, as in Rust. A `&mut` parameter of a closure type (`Box<dyn Fn(&mut State)>`) is `inout`, for the closures given as for their calls. Iterator chains become the matching Swift methods: `map`, `filter`, `fold` (→ `reduce`), `sum`, `any` (→ `contains(where:)`), `find` (→ `first(where:)`), `position` (→ `firstIndex`), `enumerate`, `zip`, `rev`, `max_by`, `collect`… A few methods without a direct equivalent come from the support file.

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

`s.parse::<i32>()` (the target may also be a later use of the value, as in `let n = s.parse().ok()?;` followed by `Point { x: n }`) and the other conversions of the standard library (`i32::from_str_radix`, `u8::try_from`, `x.try_into()`, `String::from_utf8`, `char::try_from`) return `Result`s whose errors are Rust's, in `{:?}` as in `{}`: `Err(ParseIntError { kind: InvalidDigit })`, `invalid digit found in string`, `PosOverflow` for a number too large for the type, `TryFromIntError(NegOverflow)`. `e.kind()` is the `IntErrorKind` (`Empty`, `InvalidDigit`, `PosOverflow`, `NegOverflow`), which can be matched and compared; `{:#?}` lays these errors out over several lines as Rust does, and `i32::from_str(s)` and `std::str::from_utf8(&bytes)` are translated. A `?` on an error that converts into the function's (`impl From<ParseIntError> for AppError`) converts it the same way. `v.last_mut().unwrap().done = true`, `map.get_mut(&k).unwrap().push(x)`, `if let Some(t) = v.iter_mut().find(…)` and `v.iter_mut().for_each(…)` change the element in place, as in Rust.

## Strings and formatting

`format!`, `println!`, `eprintln!` and `write!` become string interpolation. Values print exactly as in Rust thanks to small functions of the support file: `{}` of `1.0` is `1`, `{:.2}` rounds the same way, `{:?}` writes `Some(Point { x: 1.0, y: 2.5 })`, and widths, alignment and padding (`{:>8}`, `{:05}`) are kept — in `{:>7?}` they reach the numbers inside the value, as with Rust's derived `Debug`. Common string methods have their Swift equivalent (`trim` → `trimmingCharacters(in:)`, `split` → `components(separatedBy:)`, `to_uppercase` → `uppercased()`, `starts_with` → `hasPrefix`…). `s.chars()` iterates Unicode scalars, as Rust does (`s.rsChars`): `chars().count()`, `rev()` and `nth()` agree with Rust on combining marks and emoji sequences, which Swift's own iteration groups into one character. The pattern of `starts_with`, `split`, `find`, `trim_matches`… can be a text, a character, a closure or an array of characters; `s.get(a..b)` is `None` off a character boundary; `to_ascii_uppercase` changes only the ASCII letters, as in Rust.

## Comments

Comments and `///` documentation are kept, next to the code they describe, so the generated Swift stays readable: before and after statements, between the children and the modifiers of a view, and blank lines between groups of statements stay. Comments inside other chains of method calls (`v.iter() // … .map(…)`) are not kept.

## The support file

When the translation needs a helper — Rust formatting, `Ordering`, string slicing by bytes, a few collection methods (`retain`, `dedup`, `windows`…) — FerroSwift writes it once in `FerroSwiftSupport.swift` (or at the end of the file in the playground). Only the helpers you use are included.
