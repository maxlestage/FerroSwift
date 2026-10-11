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
- A method returning a mutable reference into its receiver (`fn find_mut(&mut self, id) -> Option<&mut Task>`, `-> &mut T`, `-> Result<&mut T, E>`) returns a `WritableKeyPath` into the type instead, and its callers read and write `self[keyPath: path]`: `self.find_mut(id)?.done = true`, `if let Some(t) = inv.find_mut(id) { t.qty += 1 }` and `let acct = bank.get(name)?;` change the element as in Rust. The body must end with a place of `self` (`&mut self.items[i]`, `self.map.get_mut(&k)`, `….ok_or(e)`, `self.map.entry(k).or_insert(v)`, `self.items.iter_mut().find(…)`); such a method must take `&mut self`. A `&mut` into a collection obtained on the spot (`v.last_mut().unwrap()`, `map.get_mut(&k)`, `v.iter_mut().find(…)`) changes the element too: the name stands for `v[i]`.

Write the model with values (structs, enums, `Vec`, `HashMap`) and pass `&mut` to the functions that change it: that code means the same thing in both languages.

## Shared mutability

`Box<T>` and `Rc<T>` become plain `T`: an `Rc` of a value is copied, which changes nothing as long as the value is not mutated through the `Rc`. `RefCell<T>`, `Cell<T>`, `Mutex<T>` and `RwLock<T>` become `RsRefCell<T>`, a class of the support file holding the value: `Rc::clone`, `borrow_mut()`, `set()`, `lock().unwrap()`, `Rc::ptr_eq` and `{:?}` (`RefCell { value: 1 }`) behave as in Rust, and a `let mut guard = cell.borrow_mut();` stands for the value itself. What is not modelled: the locks never block (the program is single-threaded), a `Mutex` is never poisoned, `Rc::strong_count` and `Weak` have no equivalent, and cloning a struct that holds a `RefCell` shares the cell rather than copying it. For state shared between screens, a model with `#[derive(Observable)]` becomes an `@Observable` class — see [SwiftUI in Rust](../swiftui/#state-and-bindings).

## Concurrency and async

- `async fn` becomes `async func` and `.await` becomes `await`, but there is no mapping of executors or async libraries (Tokio, futures, channels).
- `std::thread`, `Mutex`, atomics and channels are not translated: calls to them are reported as unknown functions.

## Macros

Macros of the standard library are translated: `println!`, `eprintln!`, `format!`, `write!`, `vec!`, `assert!`, `assert_eq!`, `matches!`, `panic!`, `todo!`, `unreachable!`… Your own `macro_rules!` and procedural macros are not: their definitions are skipped and their uses reported as errors.

## Unsafe code, FFI and other crates

- `unsafe` blocks are translated as ordinary code, with a warning; raw pointers and FFI have no equivalent.
- Only `std` and `ferroswift` are known. Calls into other crates (`serde`, `regex`, `rand`…) are copied as is, with a warning, and will not compile in Swift. Keep the code you translate free of dependencies, or behind your own functions.

## Behaviour differences

- **Integer overflow** stops a Swift program in every build; Rust panics in debug builds and wraps in release builds. `i32` is Swift's 64-bit `Int`, so `+` overflows at 2⁶³ rather than 2³¹; `wrapping_add`, `checked_add`, `saturating_add`, `overflowing_add`, `rotate_left`, `count_ones`, `s.parse::<i32>()`, `i32::try_from` and `x as i32` keep the 32-bit limits. `as` conversions keep Rust's meaning: `n as u8` truncates, a float cast to an integer saturates and `NaN` gives 0. `usize` is `Int` too: `usize::MAX` is 2⁶³ − 1.
- **Display and width:** as in Rust, `{:>10}` is ignored by a `Display` impl written with `write!` (only `f.pad(…)` and `self.0.fmt(f)` honor it); `{:?}` applies a width to the numbers and booleans inside a value, not to its strings.
- **Hash maps and sets** are iterated in an unspecified order in both languages, but not the same one. A `BTreeMap` is iterated in key order.
- **Strings** are indexed by bytes in Rust and by characters in Swift: slicing (`&s[0..3]`) goes through a support helper that counts bytes, like Rust.

## What is reported

| Diagnostic | Meaning |
| --- | --- |
| error | The Swift output is incomplete or will not compile (a construct that is not supported, an unknown macro…). |
| warning | The Swift output compiles but may behave differently, or calls something FerroSwift does not know. |

The [playground](../../playground/) shows the diagnostics as you type: it is the quickest way to know whether a piece of code translates.
