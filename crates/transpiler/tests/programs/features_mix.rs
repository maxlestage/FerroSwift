use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Matrix<const N: usize> {
    cells: [[i32; N]; N],
}

impl<const N: usize> Matrix<N> {
    const SIZE: usize = N;

    fn identity() -> Self {
        let mut cells = [[0; N]; N];
        for i in 0..N {
            cells[i][i] = 1;
        }
        Matrix { cells }
    }

    fn trace(&self) -> i32 {
        (0..N).map(|i| self.cells[i][i]).sum()
    }
}

struct Countdown {
    n: u32,
}

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.n == 0 {
            None
        } else {
            self.n -= 1;
            Some(self.n + 1)
        }
    }
}

trait Animal {
    const LEGS: u32;
    fn name(&self) -> String;
    fn describe(&self) -> String {
        format!("{} with {} legs", self.name(), Self::LEGS)
    }
}

struct Dog;
impl Animal for Dog {
    const LEGS: u32 = 4;
    fn name(&self) -> String {
        "dog".into()
    }
}

struct Temp(f64);
impl fmt::Display for Temp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}°", self.0)
    }
}

fn make_adder(k: i32) -> impl Fn(i32) -> i32 {
    move |x| x + k
}

fn compose<F, G>(f: F, g: G) -> impl Fn(i32) -> i32
where
    F: Fn(i32) -> i32,
    G: Fn(i32) -> i32,
{
    move |x| g(f(x))
}

fn main() {
    let m: Matrix<3> = Matrix::identity();
    println!("{} {} {:?}", m.trace(), Matrix::<3>::SIZE, m.cells[1]);
    let v: Vec<u32> = Countdown { n: 4 }.collect();
    println!("{:?} {}", v, Countdown { n: 3 }.map(|x| x * 2).sum::<u32>());
    for x in (Countdown { n: 2 }) {
        print!("{x} ");
    }
    println!();
    println!("{}", Dog.describe());
    println!("{} {:>8} [{:<7}]", Temp(21.456), Temp(-3.0).to_string(), Temp(0.04).to_string());
    let add5 = make_adder(5);
    let f = compose(add5, |x| x * 3);
    println!("{}", f(2));
    let mut words = vec!["pear", "apple", "fig", "banana", "kiwi"];
    words.sort_by_key(|w| Reverse(w.len()));
    println!("{:?}", words);
    let mut heap = BinaryHeap::new();
    for x in [5, 1, 8, 3] {
        heap.push(Reverse(x));
    }
    let mut order = vec![];
    while let Some(Reverse(x)) = heap.pop() {
        order.push(x);
    }
    println!("{:?}", order);
    let text = "the quick brown fox jumps over the lazy dog the end";
    let mut freq: BTreeMap<&str, usize> = BTreeMap::new();
    for w in text.split_whitespace() {
        *freq.entry(w).or_insert(0) += 1;
    }
    let top: Vec<(&str, usize)> = freq.iter().filter(|&(_, &c)| c > 1).map(|(w, c)| (*w, *c)).collect();
    println!("{:?}", top);
    let uniq: HashSet<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    println!("{}", uniq.len());
    let arr = [1, 2, 3, 4, 5, 6];
    match arr {
        [first, .., last] => println!("{first} {last}"),
    }
    if let [a, b, rest @ ..] = &arr[..] {
        println!("{a} {b} {:?}", rest);
    }
    let nums = [3, -1, 4, -1, 5];
    let label = 'outer: loop {
        for (i, n) in nums.iter().enumerate() {
            if *n < 0 && i > 2 {
                break 'outer i;
            }
        }
        break 99;
    };
    println!("{label} {}", matches!(nums[0], 1..=3));
    println!("{:e} {:.2e} {:?}", 1234.5f64, 0.000123f64, 1e21f64);
}
