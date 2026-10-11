use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct V2 {
    x: f64,
    y: f64,
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 { V2 { x: self.x + o.x, y: self.y + o.y } }
}
impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 { V2 { x: self.x - o.x, y: self.y - o.y } }
}
impl Mul<f64> for V2 {
    type Output = V2;
    fn mul(self, k: f64) -> V2 { V2 { x: self.x * k, y: self.y * k } }
}
impl Neg for V2 {
    type Output = V2;
    fn neg(self) -> V2 { V2 { x: -self.x, y: -self.y } }
}
impl AddAssign for V2 {
    fn add_assign(&mut self, o: V2) { self.x += o.x; self.y += o.y; }
}
impl Sum for V2 {
    fn sum<I: Iterator<Item = V2>>(iter: I) -> V2 { iter.fold(V2::default(), |a, b| a + b) }
}
impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("({:.1}, {:.1})", self.x, self.y);
        f.pad(&s)
    }
}

trait Describe {
    fn describe(&self) -> String;
}
impl<T: fmt::Display> Describe for T {
    fn describe(&self) -> String { format!("<{}>", self) }
}

#[derive(Debug, PartialEq, Eq)]
struct Version { major: u32, minor: u32, tag: String }
impl Ord for Version {
    fn cmp(&self, o: &Self) -> Ordering {
        self.major.cmp(&o.major).then(self.minor.cmp(&o.minor)).then_with(|| o.tag.len().cmp(&self.tag.len()))
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) }
}

struct Grid { w: usize, cells: Vec<char> }
impl Index<(usize, usize)> for Grid {
    type Output = char;
    fn index(&self, (r, c): (usize, usize)) -> &char { &self.cells[r * self.w + c] }
}
impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut char { &mut self.cells[r * self.w + c] }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Level { Low = 1, Mid = 5, High = 10 }

fn show<T: Describe>(x: T) -> String {
    x.describe()
}

fn largest<T: PartialOrd + Copy>(xs: &[T]) -> T {
    let mut m = xs[0];
    for &x in xs { if x > m { m = x; } }
    m
}

fn scale_all<T>(xs: &[T], k: f64) -> Vec<f64> where T: Into<f64> + Copy {
    xs.iter().map(|&x| x.into() * k).collect()
}

fn main() {
    let a = V2 { x: 1.0, y: 2.0 };
    let mut b = a * 3.0 - V2 { x: 0.5, y: 0.5 };
    b += -a;
    let total: V2 = vec![a, b, V2 { x: 10.0, y: 0.0 }].into_iter().sum();
    println!("{} {} [{:>14}] [{:<12}]", b, total, a, total);
    println!("{} {}", 42.describe(), "hi".describe());
    let mut vs = vec![
        Version { major: 1, minor: 2, tag: "".into() },
        Version { major: 1, minor: 2, tag: "beta".into() },
        Version { major: 0, minor: 9, tag: "".into() },
    ];
    vs.sort();
    println!("{:?}", vs.iter().map(|v| format!("{}.{}{}", v.major, v.minor, v.tag)).collect::<Vec<_>>());
    println!("{:?}", vs[0].cmp(&vs[1]));
    let mut g = Grid { w: 3, cells: "abcdefghi".chars().collect() };
    g[(1, 1)] = '*';
    println!("{} {} {}", g[(0, 2)], g[(1, 1)], g.cells.iter().collect::<String>());
    let mut levels = vec![Level::High, Level::Low, Level::Mid];
    levels.sort();
    println!("{:?} {} {:?}", levels, Level::Mid as i32, Level::Low.max(Level::High));
    let mut counts: HashMap<Level, u32> = HashMap::new();
    for l in [Level::Low, Level::High, Level::Low] { *counts.entry(l).or_insert(0) += 1; }
    let mut cs: Vec<_> = counts.into_iter().collect();
    cs.sort();
    println!("{:?}", cs);
    println!("{} {} {}", largest(&[3, 9, 2]), largest(&[1.5, -2.0]), largest(&['x', 'b']));
    println!("{:?}", scale_all(&[1u8, 2, 3], 0.5));
    println!("{:?}", scale_all(&[1.5f32, 2.5], 2.0));
    println!("{} {}", show(3.5), show('z'));
}
