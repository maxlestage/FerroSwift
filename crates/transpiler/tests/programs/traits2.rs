use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, AddAssign, Index, Mul, Neg, Sub};

trait Describe {
    const KIND: &'static str;
    fn name(&self) -> String;
    fn describe(&self) -> String {
        format!("{} {} ({})", Self::KIND, self.name(), self.detail())
    }
    fn detail(&self) -> String {
        "no detail".to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn dot(self, o: Vec2) -> f64 {
        self.x * o.x + self.y * o.y
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

impl Describe for Vec2 {
    const KIND: &'static str = "vector";
    fn name(&self) -> String {
        format!("{self}")
    }
    fn detail(&self) -> String {
        format!("len {:.2}", self.len())
    }
}

struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    fn identity(n: usize) -> Matrix {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Matrix { rows: n, cols: n, data }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;
    fn index(&self, (r, c): (usize, usize)) -> &f64 {
        &self.data[r * self.cols + c]
    }
}

impl Describe for Matrix {
    const KIND: &'static str = "matrix";
    fn name(&self) -> String {
        format!("{}x{}", self.rows, self.cols)
    }
}

struct Counter {
    n: u32,
    limit: u32,
}

impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.n < self.limit {
            self.n += 1;
            Some(self.n)
        } else {
            None
        }
    }
}

trait Animal {
    fn name(&self) -> String;
    fn sound(&self) -> String;
    fn speak(&self) -> String {
        format!("{} says {}", self.name(), self.sound())
    }
}

struct Dog;
struct Cat {
    lives: u8,
}

impl Animal for Dog {
    fn name(&self) -> String {
        "Dog".into()
    }
    fn sound(&self) -> String {
        "woof".into()
    }
}

impl Animal for Cat {
    fn name(&self) -> String {
        format!("Cat({})", self.lives)
    }
    fn sound(&self) -> String {
        "meow".into()
    }
    fn speak(&self) -> String {
        format!("{} purrs", self.name())
    }
}

fn loudest(animals: &[Box<dyn Animal>]) -> String {
    animals.iter().map(|a| a.speak()).max_by_key(|s| s.len()).unwrap_or_default()
}

fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

fn apply_twice(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn make_counter() -> impl FnMut() -> u32 {
    let mut c = 0;
    move || {
        c += 1;
        c
    }
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

fn double(x: i32) -> i32 {
    x * 2
}

fn largest<T: PartialOrd + Copy>(items: &[T]) -> T {
    let mut best = items[0];
    for &it in items {
        if it > best {
            best = it;
        }
    }
    best
}

fn summarize<T: fmt::Display>(items: &[T]) -> String {
    items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
}

impl From<(u32, u32)> for Version {
    fn from((major, minor): (u32, u32)) -> Self {
        Version { major, minor }
    }
}

impl From<Version> for String {
    fn from(v: Version) -> String {
        format!("v{}.{}", v.major, v.minor)
    }
}

struct Wrapper<T> {
    inner: T,
}

impl<T: fmt::Display> fmt::Display for Wrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.inner)
    }
}

impl<T: Clone> Wrapper<T> {
    fn duplicate(&self) -> (T, T) {
        (self.inner.clone(), self.inner.clone())
    }
}

fn main() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2 { x: 3.0, y: -1.0 };
    let mut c = a + b * 2.0 - Vec2::ZERO;
    c += -a;
    println!("{} {} {} {:.3}", a + b, c, a.dot(b), (a - b).len());
    println!("{}", a.describe());
    println!("{}", Vec2::KIND);
    println!("{:?} {}", Vec2::default(), Vec2::default() == Vec2::ZERO);
    let m = Matrix::identity(3);
    println!("{} {} {}", m[(0, 0)], m[(0, 1)], m.describe());
    let total: u32 = Counter { n: 0, limit: 5 }.sum();
    let evens: Vec<u32> = Counter { n: 0, limit: 10 }.filter(|n| n % 2 == 0).collect();
    println!("{total} {:?}", evens);
    for n in (Counter { n: 2, limit: 4 }) {
        print!("{n} ");
    }
    println!();
    let mut counter = Counter { n: 0, limit: 2 };
    println!("{:?} {:?} {:?}", counter.next(), counter.next(), counter.next());
    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat { lives: 9 })];
    for a in &zoo {
        println!("{}", a.speak());
    }
    println!("{}", loudest(&zoo));
    let names: Vec<String> = zoo.iter().map(|a| a.name()).collect();
    println!("{:?}", names);
    println!("{} {} {}", apply(double, 4), apply(|x| x - 1, 4), apply_twice(double, 3));
    let add5 = make_adder(5);
    println!("{} {}", add5(1), apply(&add5, 2));
    let mut next = make_counter();
    println!("{} {} {}", next(), next(), next());
    let f = compose(double, |x| x + 1);
    let g = compose(|s: &str| s.len(), |n| n * 10);
    println!("{} {}", f(5), g("abc"));
    let fs: Vec<Box<dyn Fn(i32) -> i32>> = vec![Box::new(double), Box::new(make_adder(3)), Box::new(|x| x * x)];
    println!("{:?}", fs.iter().map(|f| f(4)).collect::<Vec<_>>());
    let ops: HashMap<&str, fn(i32, i32) -> i32> = HashMap::from([("add", (|a, b| a + b) as fn(i32, i32) -> i32), ("mul", |a, b| a * b)]);
    println!("{} {}", ops["add"](2, 3), ops["mul"](2, 3));
    println!("{} {} {}", largest(&[3, 9, 2]), largest(&[1.5, 0.5]), largest(&['x', 'b']));
    println!("{}", summarize(&[1, 2, 3]));
    println!("{}", summarize(&["a", "b"]));
    println!("{}", summarize(&[a, b]));
    let v: Version = (1, 4).into();
    let v2 = Version::from((1, 10));
    println!("{:?} {} {}", v, v < v2, String::from(v2.clone()));
    let s: String = v.into();
    println!("{s}");
    let mut versions = vec![Version { major: 2, minor: 0 }, Version { major: 1, minor: 9 }, Version { major: 1, minor: 10 }];
    versions.sort();
    println!("{:?}", versions.iter().map(|v| format!("{}.{}", v.major, v.minor)).collect::<Vec<_>>());
    println!("{:?}", versions.iter().max());
    let w = Wrapper { inner: 42 };
    let ws = Wrapper { inner: "hi".to_string() };
    println!("{} {} {:?} {:?}", w, ws, w.duplicate(), ws.duplicate());
    let sq = |x: f64| x * x;
    let vals: Vec<f64> = [1.0, 2.0, 3.0].iter().map(|&x| sq(x)).collect();
    println!("{:?}", vals);
    let mut log: Vec<String> = Vec::new();
    let mut record = |s: &str| log.push(s.to_uppercase());
    record("a");
    record("b");
    println!("{:?}", log);
    let factor = 3;
    let scale = move |x: i32| x * factor;
    println!("{}", (1..=3).map(scale).sum::<i32>());
    let pipeline: Vec<fn(f64) -> f64> = vec![f64::sqrt, f64::floor];
    println!("{}", pipeline.iter().fold(20.0, |acc, f| f(acc)));
}
