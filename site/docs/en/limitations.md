---
title: Limitations
description: What FerroSwift does not translate (yet): ownership semantics, shared mutability, async and threads, macros, unsafe code and other crates.
order: 5
---

FerroSwift translates the Rust you write for an app's model, logic and views. Some parts of Rust have no Swift counterpart, or are not translated yet. Most of them are reported with a diagnostic; the ones below deserve to be known before you start.

## Ownership, borrowing and lifetimes

rustc checks ownership and lifetimes; Swift does not need them. FerroSwift drops lifetimes (`&'a str` is `String`) and translates references as values:

- `&T` becomes `T`, and a `&mut T` parameter becomes `inout T`: the caller sees the changes, as in Rust.
- A reference **stored** in a struct becomes a copy. Changes made through it are not seen by the original.
- Moves and `Drop` have no effect: Swift copies structs and counts references to classes.

Write the model with values (structs, enums, `Vec`, `HashMap`) and pass `&mut` to the functions that change it: that code means the same thing in both languages.

## Shared mutability

`Rc<RefCell<T>>`, `Arc<Mutex<T>>`, `Cell<T>` and `Box<T>` become plain `T`. The sharing is lost: two `Rc` clones of the same value become two independent copies, and FerroSwift warns at each `RefCell::new`, `Cell::new`, `Mutex::new` and `RwLock::new`. For state shared between screens, use a model with `#[derive(Observable)]`, which becomes an `@Observable` class (a reference type) — see [SwiftUI in Rust](../swiftui/#state-and-bindings).

## Concurrency and async

- `async fn` becomes `async func` and `.await` becomes `await`, but there is no mapping of executors or async libraries (Tokio, futures, channels).
- `std::thread`, `Mutex`, atomics and channels are not translated: calls to them are reported as unknown functions.

## Macros

Macros of the standard library are translated: `println!`, `eprintln!`, `format!`, `write!`, `vec!`, `assert!`, `assert_eq!`, `matches!`, `panic!`, `todo!`, `unreachable!`… Your own `macro_rules!` and procedural macros are not: their definitions are skipped and their uses reported as errors.

## Unsafe code, FFI and other crates

- `unsafe` blocks are translated as ordinary code, with a warning; raw pointers and FFI have no equivalent.
- Only `std` and `ferroswift` are known. Calls into other crates (`serde`, `regex`, `rand`…) are copied as is, with a warning, and will not compile in Swift. Keep the code you translate free of dependencies, or behind your own functions.

## Behaviour differences

- **Integer overflow** stops a Swift program in every build; Rust panics in debug builds and wraps in release builds. `as` conversions keep Rust's meaning (`n as u8` truncates).
- **Hash maps and sets** are iterated in an unspecified order in both languages, but not the same one. A `BTreeMap` is iterated in key order.
- **Strings** are indexed by bytes in Rust and by characters in Swift: slicing (`&s[0..3]`) goes through a support helper that counts bytes, like Rust.

## What is reported

| Diagnostic | Meaning |
| --- | --- |
| error | The Swift output is incomplete or will not compile (a construct that is not supported, an unknown macro…). |
| warning | The Swift output compiles but may behave differently, or calls something FerroSwift does not know. |

The [playground](../../playground/) shows the diagnostics as you type: it is the quickest way to know whether a piece of code translates.
