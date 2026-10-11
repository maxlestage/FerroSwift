---
title: Limitations
description: What FerroSwift does not translate, or translates differently: ownership semantics, shared mutability, threads, async, unsafe code and other crates.
order: 5
---

FerroSwift translates the Rust you write for an app's model, logic and views. Some parts of Rust have no Swift counterpart, or are not translated yet. Most of them are reported with a diagnostic; the ones below deserve to be known before you start.

## Ownership, borrowing and lifetimes

rustc checks ownership and lifetimes; Swift does not need them. FerroSwift drops lifetimes (`&'a str` is `String`) and translates references as values:

- `&T` becomes `T`, and a `&mut T` parameter becomes `inout T`: the caller sees the changes, as in Rust.
- A reference **stored** in a struct becomes a copy. Changes made through it are not seen by the original.
- Moves and `Drop` have no effect: Swift copies structs and counts references to classes.
- A method returning a mutable reference into its receiver (`fn find_mut(&mut self, id) -> Option<&mut Task>`, `-> &mut T`, `-> Result<&mut T, E>`) returns a `WritableKeyPath` into the type instead, and its callers read and write `self[keyPath: path]`: `self.find_mut(id)?.done = true`, `if let Some(t) = inv.find_mut(id) { t.qty += 1 }` and `let acct = bank.get(name)?;` change the element as in Rust. The body must end with a place of `self` (`&mut self.items[i]`, `self.map.get_mut(&k)`, `….ok_or(e)`, `self.map.entry(k).or_insert(v)`, `self.items.iter_mut().find(…)`); such a method must take `&mut self`. A `&mut` into a collection obtained on the spot (`v.last_mut().unwrap()`, `map.get_mut(&k)`, `v.iter_mut().find(…)`) changes the element too: the name stands for `v[i]`. A cursor on linked nodes (`let mut cur = &mut self.head;` then `cur = &mut node.next`, `*cur = Some(…)`, `cur.take()`) is a small `RsLink` of the support file holding the node and the field it points into, so the idioms of Rust lists and trees keep their meaning with nodes that Swift makes classes.

Write the model with values (structs, enums, `Vec`, `HashMap`) and pass `&mut` to the functions that change it: that code means the same thing in both languages.

## Shared mutability

`Box<T>` and `Rc<T>` become plain `T`: an `Rc` of a value is copied, which changes nothing as long as the value is not mutated through the `Rc`. A program that counts its references (`Rc::strong_count`, `Rc::weak_count`, `Rc::downgrade`, `Weak<T>`) gets `RsRc<T>` and `RsWeak<T>` instead, which count them as Rust does: a clone adds one, a `drop` or the end of a scope removes one, the value goes away with the last strong reference, `upgrade()` then gives `None`, and `{:?}` of a `Weak` prints `(Weak)`. `RefCell<T>`, `Cell<T>`, `Mutex<T>` and `RwLock<T>` become `RsRefCell<T>`, a class of the support file holding the value: `Rc::clone`, `borrow_mut()`, `set()`, `lock().unwrap()`, `Rc::ptr_eq` and `{:?}` (`RefCell { value: 1 }`) behave as in Rust, and a `let mut guard = cell.borrow_mut();` stands for the value itself. What is not modelled: the locks never block (the program is single-threaded), a `Mutex` is never poisoned, and cloning a struct that holds a `RefCell` shares the cell rather than copying it. For state shared between screens, a model with `#[derive(Observable)]` becomes an `@Observable` class — see [SwiftUI in Rust](../swiftui/#state-and-bindings).

## Concurrency and async

- `async fn` becomes `async func` and `.await` becomes `await`; executors and async libraries (Tokio, futures) are other crates, not translated.
- `std::thread` (`spawn`, `scope`, `sleep`, `yield_now`, `JoinHandle::join`), `mpsc` channels (`channel`, `sync_channel`, `send`, `recv`, `try_recv`, `recv_timeout`, iteration), atomics (`AtomicUsize`, `AtomicBool`… with `load`, `store`, `swap`, `fetch_add` and the other `fetch_…`, `compare_exchange`), `Arc`, `Mutex` and `RwLock` are translated with types of the support file. A spawned thread is a task run when something waits for it: its `join`, a `recv` on an empty channel, the end of its `thread::scope`, a `sleep` or `yield_now`. The program thus prints what the Rust program prints once its threads are joined, deterministically and without data races; what differs is timing: work is not spread over cores, a thread never joined nor waited for does not run, and a thread that panics stops the program instead of making `join` return an error.

## Macros

Macros of the standard library are translated: `println!`, `eprintln!`, `format!`, `write!`, `vec!`, `assert!`, `assert_eq!`, `matches!`, `panic!`, `todo!`, `unreachable!`, `stringify!`, `concat!`… Your own `macro_rules!` are expanded before translation, as rustc expands them: every fragment kind (`expr`, `ident`, `ty`, `tt`, `literal`, `pat`, `block`, `stmt`, `path`, `item`…), repetitions (`$(…),*`, `+`, `?`, nested), several rules, recursion, uses in expressions, statements, items, `impl` blocks and the arguments of other macros. Procedural macros (`#[derive(Serialize)]` of a crate, attribute macros) are not: they belong to other crates.

## Unsafe code, FFI and other crates

- `unsafe` blocks are translated as ordinary code, with a warning; raw pointers and FFI have no equivalent.
- Only `std` and `ferroswift` are known. Calls into other crates (`serde`, `regex`, `rand`…) are copied as is, with a warning, and will not compile in Swift. Keep the code you translate free of dependencies, or behind your own functions.

## Behaviour differences

- **Panics** are Rust's: `unwrap()` on `None` or `Err`, `expect(…)`, an index past the end of a `Vec`, a slice out of range, a missing key in `map[&k]`, a division by zero, a string sliced inside a character, `assert!`, `assert_eq!`, `panic!`, `unreachable!` and `todo!` print the message Rust prints, with the same file, line and column (`thread 'main' (…) panicked at src/main.rs:4:13:`), and exit with code 101. A panic cannot be caught: `std::panic::catch_unwind` has no equivalent.
- **Integer overflow** stops a Swift program in every build; Rust panics in debug builds and wraps in release builds. `i32` is Swift's 64-bit `Int`, so `+` overflows at 2⁶³ rather than 2³¹; `wrapping_add`, `checked_add`, `saturating_add`, `overflowing_add`, `rotate_left`, `count_ones`, `s.parse::<i32>()`, `i32::try_from` and `x as i32` keep the 32-bit limits. `as` conversions keep Rust's meaning: `n as u8` truncates, a float cast to an integer saturates and `NaN` gives 0. `usize` is `Int` too: `usize::MAX` is 2⁶³ − 1.
- **Display and width:** as in Rust, `{:>10}` is ignored by a `Display` impl written with `write!` (one using `f.pad(…)`, `f.pad_integral(…)`, `self.0.fmt(f)` or the formatter's queries honors it); `{:?}` applies a width to the numbers and booleans inside a value, not to its strings. Widths and precisions count scalars, as Rust counts `char`s.
- **Hash maps and sets** are iterated in an unspecified order in both languages, but not the same one. A `BTreeMap` is iterated in key order.
- **`Deref`:** `*x` on a value whose type has an `impl Deref` gives the target (also when `x` is a reference to such a value: write `x.clone()` for the value itself). A trait with type parameters cannot be used as a `dyn Converter<f64>` value: its associated type has no Swift existential.
- **`i128` and `u128`** become `Int128` and `UInt128`, which Apple's platforms provide from iOS 18 and macOS 15 on.
- **Strings** are indexed by bytes in Rust and by characters in Swift: slicing (`&s[0..3]`) goes through a support helper that counts bytes, like Rust.

## What is reported

| Diagnostic | Meaning |
| --- | --- |
| error | The Swift output is incomplete or will not compile (a construct that is not supported, an unknown macro…). |
| warning | The Swift output compiles but may behave differently, or calls something FerroSwift does not know. |

The [playground](../../playground/) shows the diagnostics as you type: it is the quickest way to know whether a piece of code translates.
