use std::collections::{BTreeSet, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Kind {
    Leaf(u32),
    Pair(Box<Kind>, Box<Kind>),
    Named { name: String, weight: i64 },
}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.age)
    }
}

fn weight(k: &Kind) -> i64 {
    match k {
        Kind::Leaf(n @ 0..=9) => *n as i64,
        Kind::Leaf(n) if n % 2 == 0 => *n as i64 * 2,
        Kind::Leaf(_) => -1,
        Kind::Pair(a, b) => weight(a) + weight(b),
        Kind::Named { weight, .. } if *weight < 0 => 0,
        Kind::Named { name, weight } => *weight + name.len() as i64,
    }
}

fn describe(xs: &[i32]) -> String {
    match xs {
        [] => "none".into(),
        [x] if *x < 0 => format!("one negative {x}"),
        [x] => format!("one {x}"),
        [first, rest @ ..] if rest.iter().all(|r| r == first) => format!("all {first} x{}", xs.len()),
        [first, .., last] => format!("{first}..{last} ({} inside)", xs.len() - 2),
    }
}

fn apply_n<F: FnMut(i32) -> i32>(mut f: F, n: usize, start: i32) -> i32 {
    let mut v = start;
    for _ in 0..n {
        v = f(v);
    }
    v
}

fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|x| x % 2 == 0)
}

fn squares_of(xs: &[i32]) -> impl Iterator<Item = i32> + '_ {
    xs.iter().map(|x| x * x)
}

struct Inventory {
    items: Vec<(String, u32)>,
}

impl Inventory {
    fn names(&self) -> impl Iterator<Item = &str> {
        self.items.iter().map(|(n, _)| n.as_str())
    }
    fn total(&self) -> u32 {
        self.items.iter().map(|&(_, q)| q).sum()
    }
    fn restock(&mut self, name: &str, qty: u32) -> bool {
        if let Some(item) = self.items.iter_mut().find(|(n, _)| n == name) && item.1 < 100 {
            item.1 += qty;
            true
        } else {
            false
        }
    }
}

fn main() {
    let tree = Kind::Pair(Box::new(Kind::Leaf(5)), Box::new(Kind::Pair(Box::new(Kind::Leaf(12)), Box::new(Kind::Named { name: "ab".into(), weight: 3 }))));
    println!("{} {} {} {}", weight(&tree), weight(&Kind::Leaf(13)), weight(&Kind::Named { name: "x".into(), weight: -4 }), weight(&Kind::Leaf(0)));
    let mut kinds = vec![Kind::Leaf(3), Kind::Named { name: "z".into(), weight: 1 }, Kind::Leaf(1), Kind::Pair(Box::new(Kind::Leaf(0)), Box::new(Kind::Leaf(0)))];
    kinds.sort();
    println!("{:?}", kinds);
    println!("{} {}", kinds.contains(&Kind::Leaf(1)), kinds.iter().collect::<HashSet<_>>().len());
    for xs in [&[][..], &[-3], &[4], &[7, 7, 7], &[1, 2, 3, 4]] {
        println!("{}", describe(xs));
    }
    let mut calls = 0;
    let result = apply_n(|v| { calls += 1; v * 2 + 1 }, 5, 1);
    println!("{result} {calls}");
    let mut history = Vec::new();
    let r2 = apply_n(|v| { history.push(v); v - 3 }, 3, 10);
    println!("{r2} {:?}", history);
    println!("{:?} {} {:?}", evens(10).collect::<Vec<_>>(), evens(7).count(), squares_of(&[1, 2, 3]).skip(1).collect::<Vec<_>>());
    let mut inv = Inventory { items: vec![("bolt".into(), 40), ("nut".into(), 99), ("gear".into(), 5)] };
    let restocked = (inv.restock("nut", 5), inv.restock("bolt", 5), inv.restock("cog", 1));
    println!("{:?} {} {:?}", inv.names().collect::<Vec<_>>(), inv.total(), restocked);
    println!("{:?}", inv.items);
    let a: Option<i32> = Some(5);
    let b: Option<i32> = None;
    if let Some(x) = a && x > 3 && let None = b {
        println!("chain {x}");
    }
    if let Some(x) = a && let Some(y) = b {
        println!("never {x} {y}");
    } else if let Some(x) = a {
        println!("fallback {x}");
    }
    let mut stack = vec![1, 5, 2];
    while let Some(&top) = stack.last() && top != 5 {
        stack.pop();
    }
    println!("{:?}", stack);
    let pairs = [(1, 'a'), (2, 'b'), (3, 'c')];
    for &(n, c) in &pairs {
        if n == 2 { continue; }
        print!("{n}{c} ");
    }
    println!();
    let (mut lo, mut hi) = (0, 10);
    while lo < hi {
        let mid = (lo + hi) / 2;
        if mid * mid < 30 { lo = mid + 1 } else { hi = mid }
    }
    println!("{lo}");
    let people = vec![Person { name: "ann".into(), age: 30 }, Person { name: "bob".into(), age: 25 }, Person { name: "cid".into(), age: 30 }, Person { name: "dee".into(), age: 25 }, Person { name: "eve".into(), age: 30 }];
    let mut by_age = people.clone();
    by_age.sort_by_key(|p| p.age);
    println!("{}", by_age.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "));
    let mut unstable = people.clone();
    unstable.sort_unstable_by_key(|p| std::cmp::Reverse(p.age));
    println!("{}", unstable.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(","));
    let mut dd = vec![1, 1, 2, 3, 3, 3, 4, 1];
    dd.dedup_by_key(|x| *x / 2);
    println!("{:?} {:?}", dd, people.iter().map(|p| p.age).max_by_key(|a| a % 7));
    let s1: BTreeSet<i32> = (1..=6).collect();
    let s2: BTreeSet<i32> = [4, 5, 6, 7, 8].into_iter().collect();
    println!("{:?} {:?}", s1.union(&s2).collect::<Vec<_>>(), s1.intersection(&s2).cloned().collect::<Vec<_>>());
    println!("{:?} {:?}", s1.difference(&s2).collect::<Vec<_>>(), s1.symmetric_difference(&s2).count());
    println!("{} {} {:?} {:?}", s1.is_subset(&s2), s1.is_disjoint(&BTreeSet::from([9])), s1.first(), s2.last());
    let mut s3 = s1.clone();
    let popped = s3.pop_first();
    let inserted = (s3.insert(3), s3.insert(42));
    let removed = (s3.remove(&2), s3.remove(&99));
    println!("{:?} {:?} {:?} {:?} {:?}", popped, inserted, removed, s3, s3.range(4..).collect::<Vec<_>>());
    println!("{:?} {:?}", s3.iter().rev().take(2).collect::<Vec<_>>(), s3.iter().filter(|x| *x % 2 == 0).count());
    let tuples: BTreeSet<(i32, &str)> = [(2, "b"), (1, "z"), (1, "a")].into_iter().collect();
    println!("{:?} {:?}", tuples, tuples.iter().next());
    let x: f64 = 3i32.into();
    let s: String = 'c'.into();
    let big: u64 = u64::from(3u32) + u64::from(true) + u64::from(b'a');
    println!("{x} {s} {big} {} {} {}", i64::from(-3i32), f64::from(1.5f32), f64::from(3u8));
    println!("{:?} {:?} {:?} {:?}", u8::try_from(300i32), i8::try_from(-129i64), u32::try_from(-1i32).is_err(), usize::try_from(5u64));
    println!("{:?} {:?} {:?}", u8::from_str_radix("ff", 16), i32::from_str_radix("-zz", 36), u8::from_str_radix("100", 16).is_err());
    println!("{:b} {:08b} {:x} {:#X} {:o} {:#b}", 5u8, 5u8, u64::MAX, 255u8, 8, 0);
    println!("{} {} {} {} {}", 1u32 << 31, !0u32, u32::MAX.count_ones(), (-128i8).wrapping_abs(), i16::MAX.saturating_add(1));
    println!("{:?} {} {} {:?}", u16::MAX.checked_add(1), 200u8.saturating_mul(2), 7u8.leading_zeros(), char::from_digit(35, 36));
    println!("{0} {0} {1} {{literal}} {name:>6.2} {2:?}", 1, "two", (), name = 3.14159);
    let w = 7;
    println!("[{:>w$}] [{:<1$}] [{:^w$.1}]", "ab", 4, 2.55);
    println!("{:5}|{:<7}|{:>9}|", Person { name: "al".into(), age: 1 }, format!("{}", Person { name: "bo".into(), age: 2 }), 'z');
    eprintln!("to stderr 1");
    println!("to stdout 1");
    eprintln!("to stderr 2");
    println!("to stdout 2");
    eprint!("no newline");
    eprintln!(" done");
}
