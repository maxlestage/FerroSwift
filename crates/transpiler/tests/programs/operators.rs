use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeSet, BinaryHeap};
use std::fmt;
use std::ops::{Add, AddAssign, Index, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct V2 {
    x: f64,
    y: f64,
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2 { x: self.x + o.x, y: self.y + o.y }
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2 { x: self.x - o.x, y: self.y - o.y }
    }
}

impl Mul<f64> for V2 {
    type Output = V2;
    fn mul(self, k: f64) -> V2 {
        V2 { x: self.x * k, y: self.y * k }
    }
}

impl Neg for V2 {
    type Output = V2;
    fn neg(self) -> V2 {
        V2 { x: -self.x, y: -self.y }
    }
}

impl AddAssign for V2 {
    fn add_assign(&mut self, o: V2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

struct Poly(Vec<i64>);

impl Index<usize> for Poly {
    type Output = i64;
    fn index(&self, i: usize) -> &i64 {
        &self.0[i]
    }
}

impl Poly {
    fn eval(&self, x: i64) -> i64 {
        self.0.iter().rev().fold(0, |acc, c| acc * x + c)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Task {
    priority: u8,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Push(i64),
    Add,
    Mul,
    Dup,
}

fn run(ops: &[Op]) -> Result<Vec<i64>, String> {
    let mut stack: Vec<i64> = Vec::new();
    for (i, op) in ops.iter().enumerate() {
        match *op {
            Op::Push(n) => stack.push(n),
            Op::Add | Op::Mul => {
                let (Some(b), Some(a)) = (stack.pop(), stack.pop()) else {
                    return Err(format!("stack underflow at {i}"));
                };
                stack.push(if *op == Op::Add { a + b } else { a * b });
            }
            Op::Dup => {
                let Some(&top) = stack.last() else {
                    return Err("empty".to_string());
                };
                stack.push(top);
            }
        }
    }
    Ok(stack)
}

fn generic_max<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut it = items.iter();
    let mut best = *it.next()?;
    for &x in it {
        if x > best {
            best = x;
        }
    }
    Some(best)
}

fn pair_up<T: Clone, U>(a: &[T], b: Vec<U>) -> Vec<(T, U)> {
    a.iter().cloned().zip(b).collect()
}

fn main() {
    let a = V2 { x: 1.0, y: 2.0 };
    let b = V2 { x: 0.5, y: -1.0 };
    let mut c = a + b * 2.0 - V2::default();
    c += -a;
    println!("{} {} {:?}", a + b, c, -b == V2 { x: -0.5, y: 1.0 });
    let p = Poly(vec![1, 2, 3]);
    println!("{} {} {}", p[0], p[2], p.eval(2));
    let mut heap = BinaryHeap::new();
    heap.push(Task { priority: 2, name: "b".into() });
    heap.push(Task { priority: 5, name: "a".into() });
    heap.push(Task { priority: 2, name: "a".into() });
    while let Some(t) = heap.pop() {
        print!("{}{} ", t.priority, t.name);
    }
    println!();
    let mut mins = BinaryHeap::new();
    for x in [5, 1, 8, 3] {
        mins.push(Reverse(x));
    }
    let Reverse(smallest) = mins.pop().unwrap();
    println!("{smallest} {:?}", mins.peek());
    let ordered: BTreeSet<Task> = [Task { priority: 1, name: "z".into() }, Task { priority: 1, name: "a".into() }].into_iter().collect();
    println!("{:?}", ordered.iter().map(|t| t.name.as_str()).collect::<Vec<_>>());
    println!("{:?} {:?}", run(&[Op::Push(2), Op::Push(3), Op::Add, Op::Dup, Op::Mul]), run(&[Op::Add]));
    println!("{:?}", run(&[Op::Dup]));
    println!("{:?} {:?} {:?}", generic_max(&[3, 9, 2]), generic_max(&[1.5, -2.0]), generic_max::<u8>(&[]));
    println!("{:?}", pair_up(&["a", "b"], vec![1, 2, 3]));
    println!("{:?} {:?}", 1.cmp(&2), "b".cmp("a"));
    println!("{:?} {:?}", 2.5f64.partial_cmp(&2.5), f64::NAN.partial_cmp(&1.0));
    let order = [3, 1, 2].iter().fold(Ordering::Equal, |acc, _| acc.then(Ordering::Less));
    println!("{:?} {:?} {:?}", order, Ordering::Greater.reverse(), Ordering::Less.is_lt());
    println!("{} {} {} {}", 300i32 as u8, -1i32 as u32, 3.99f64 as i32, -3.99f64 as i64);
    println!("{} {} {} {}", 1e20f64 as i32, f64::NAN as i32, 255u8 as i8, (65u8 as char));
    println!("{} {} {}", u32::MAX as f32, (1u64 << 40) as i32, i64::MIN as f64);
    println!("{} {} {} {}", 0b1010 & 0b0110, 0b1010 | 1, 0b1010 ^ 0b1111, !0u8);
    println!("{} {} {} {}", 1u32 << 31, -16i32 >> 2, 1u8.rotate_left(7), 0x0Fu8.count_ones());
    println!("{} {} {}", 40u32.leading_zeros(), 40u32.trailing_zeros(), 7u8.abs_diff(10));
    println!("{} {} {} {}", 2i32.pow(10), 2u64.pow(40), (-2i32).pow(3), 10u8.pow(2));
    println!("{} {} {}", 17 % 5, -17 % 5, 17.5 % 5.0);
    println!("{} {} {:.3} {:.3}", 9f64.sqrt(), 2f64.powi(10), 2f64.powf(0.5), (1.0f64).exp());
    println!("{} {} {} {}", 2.5f64.round(), 3.5f64.round(), (-2.5f64).round(), 2.5f64.floor());
    println!("{} {} {} {}", 2.7f64.trunc(), (-2.7f64).ceil(), 7.0f64.rem_euclid(3.0), (-7.0f64).rem_euclid(3.0));
    println!("{} {} {}", f64::MAX, f64::MIN_POSITIVE, f64::EPSILON);
    println!("{} {} {} {}", 1.0f64 / 3.0, 1.0f32 / 3.0, 100000000.0f32, 0.1 + 0.2);
    println!("{:?} {:?} {:?} {:?}", 1.0f64, 1.5e-7f64, 1e16f64, 123456789.0f32);
    println!("{} {} {}", 1e16f64, 1e15f64, 1.0e-5f64);
    println!("{:?} {:?}", f64::INFINITY, -0.0f64);
    println!("{:?} {:?} {:?}", "tab\t\"quote\" \\ é", 'é', '\n');
    println!("{:#?}", (1, "two", [3.0, 4.5]));
    println!("{:#?}", Task { priority: 1, name: "n".into() });
    println!("{:#?}", vec![Some(1), None]);
    println!("{:?}", (u8::MAX, i8::MIN, u16::MAX, i64::MAX, u64::MAX));
    println!("{:>8.3}|{:<8.2e}|{:^+9.1}|", 3.14159, 1234.5, 2.0);
    println!("{:08.2}|{:+08}|{:#010b}|{:#06x}", -3.14159, 42, 5, 255);
    println!("{:3}|{:<3}|{:>3}|{:^3}|", "abcd", "ab", "ab", "ab");
    println!("{:?} {:?}", String::from("multi\nline"), "ünïcödé".len());
    println!("{:5}|{:<5}|{:>5?}|{:^7?}|", true, 'c', "s", Some(1));
}
