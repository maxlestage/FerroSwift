use std::collections::HashMap;
use std::fmt::{self, Display, Write};

mod geometry {
    pub const UNIT: f64 = 1.0;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Pt {
        pub x: f64,
        pub y: f64,
    }

    impl Pt {
        pub fn dist(&self, o: &Pt) -> f64 {
            ((self.x - o.x).powi(2) + (self.y - o.y).powi(2)).sqrt()
        }
    }

    pub fn centroid(pts: &[Pt]) -> Option<Pt> {
        if pts.is_empty() {
            return None;
        }
        let n = pts.len() as f64;
        Some(Pt { x: pts.iter().map(|p| p.x).sum::<f64>() / n, y: pts.iter().map(|p| p.y).sum::<f64>() / n })
    }

    pub mod shapes {
        use super::Pt;

        pub trait Shape {
            fn area(&self) -> f64;
            fn name(&self) -> String;
            fn describe(&self) -> String {
                format!("{} with area {:.2}", self.name(), self.area())
            }
        }

        pub struct Circle {
            pub center: Pt,
            pub r: f64,
        }

        pub struct Poly {
            pub pts: Vec<Pt>,
        }

        impl Shape for Circle {
            fn area(&self) -> f64 {
                std::f64::consts::PI * self.r * self.r
            }
            fn name(&self) -> String {
                "circle".to_string()
            }
        }

        impl Shape for Poly {
            fn area(&self) -> f64 {
                let n = self.pts.len();
                let mut s = 0.0;
                for i in 0..n {
                    let (a, b) = (self.pts[i], self.pts[(i + 1) % n]);
                    s += a.x * b.y - b.x * a.y;
                }
                s.abs() / 2.0
            }
            fn name(&self) -> String {
                format!("{}-gon", self.pts.len())
            }
            fn describe(&self) -> String {
                format!("polygon of {} points", self.pts.len())
            }
        }
    }
}

use geometry::shapes::{Circle, Poly, Shape};
use geometry::{Pt, centroid};

#[derive(Debug, Clone, PartialEq)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone + Display> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, x: T) -> &mut Self {
        self.items.push(x);
        self
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
    fn render(&self) -> String {
        let mut s = String::new();
        for (i, it) in self.items.iter().enumerate() {
            if i > 0 {
                s.push_str(" | ");
            }
            write!(s, "{it}").unwrap();
        }
        s
    }
}

impl<T> From<Vec<T>> for Stack<T> {
    fn from(items: Vec<T>) -> Self {
        Stack { items }
    }
}

fn largest<T: PartialOrd + Copy>(xs: &[T]) -> Option<T> {
    let mut best = *xs.first()?;
    for &x in xs {
        if x > best {
            best = x;
        }
    }
    Some(best)
}

fn pair_up<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<(A, B)>
where
    A: Display,
{
    a.iter().cloned().zip(b.iter().cloned()).collect()
}

fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn loudest(s: &dyn Shape) -> String {
    s.describe().to_uppercase()
}

fn print_all(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join("; ")
}

struct Fib {
    a: u64,
    b: u64,
}

impl Iterator for Fib {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let r = self.a;
        self.a = self.b;
        self.b += r;
        Some(r)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    fn symbol(self) -> char {
        match self {
            Self::Clubs => 'C',
            Self::Diamonds => 'D',
            Self::Hearts => 'H',
            Self::Spades => 'S',
        }
    }

    fn is_red(self) -> bool {
        matches!(self, Self::Diamonds | Self::Hearts)
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Card {
    rank: u8,
    suit: Suit,
}

#[derive(Debug)]
enum AppError {
    Parse(String),
    Range(i64),
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self {
        AppError::Parse(e.to_string())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Parse(s) => write!(f, "parse error: {s}"),
            AppError::Range(n) => write!(f, "{n} is out of range"),
        }
    }
}

fn parse_percent(s: &str) -> Result<u8, AppError> {
    let n: i64 = s.trim().parse()?;
    if !(0..=100).contains(&n) {
        return Err(AppError::Range(n));
    }
    Ok(n as u8)
}

fn first_even_square(xs: &[i32]) -> Option<i32> {
    let x = xs.iter().find(|&&x| x % 2 == 0)?;
    let sq = x.checked_mul(*x)?;
    Some(sq)
}

struct Pipeline {
    steps: Vec<(String, Box<dyn Fn(i64) -> i64>)>,
    fallback: Option<Box<dyn Fn(i64) -> i64>>,
}

impl Pipeline {
    fn run(&self, x: i64) -> i64 {
        let mut v = x;
        for (_, f) in &self.steps {
            v = f(v);
        }
        match &self.fallback {
            Some(f) if v < 0 => f(v),
            _ => v,
        }
    }
}

fn memo_count(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
    if n <= 1 {
        return 1;
    }
    if let Some(&v) = memo.get(&n) {
        return v;
    }
    let v = memo_count(n - 1, memo) + memo_count(n - 2, memo);
    memo.insert(n, v);
    v
}

fn main() {
    let pts = vec![Pt { x: 0.0, y: 0.0 }, Pt { x: 4.0, y: 0.0 }, Pt { x: 4.0, y: 3.0 }, Pt { x: 0.0, y: 3.0 }];
    println!("{:?} {:.3} {} {:?}", centroid(&pts), pts[0].dist(&pts[2]), geometry::UNIT, centroid(&[]));
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle { center: pts[0], r: 1.5 }), Box::new(Poly { pts: pts.clone() })];
    println!("{:.3} {}", total_area(&shapes), print_all(shapes.iter().map(|s| s.describe())));
    println!("{} | {}", loudest(shapes[0].as_ref()), loudest(&Poly { pts: pts[..3].to_vec() }));
    let mut st: Stack<String> = Stack::new();
    st.push("a".into()).push("b".into()).push("c".into());
    let top = st.pop();
    println!("{:?} {:?} {} [{}]", top, st.peek(), st.len(), st.render());
    let nums: Stack<i32> = Stack::from(vec![3, 1, 2]);
    println!("{} {:?} {}", nums.render(), nums, nums == Stack { items: vec![3, 1, 2] });
    println!("{:?} {:?} {:?} {:?}", largest(&[3, 9, 2]), largest(&[1.5, -2.0]), largest::<u8>(&[]), largest(&['x', 'b']));
    println!("{:?}", pair_up(&["a", "b", "c"], &[1, 2]));
    println!("{:?} {}", Fib { a: 0, b: 1 }.take(10).collect::<Vec<_>>(), Fib { a: 0, b: 1 }.take_while(|&x| x < 100).filter(|x| x % 2 == 0).sum::<u64>());
    println!("{:?}", Fib { a: 0, b: 1 }.skip(5).step_by(2).take(3).map(|x| x.to_string()).collect::<Vec<_>>().join("-"));
    let mut deck: Vec<Card> = Suit::ALL.iter().flat_map(|&s| (1..=3).map(move |r| Card { rank: r, suit: s })).collect();
    deck.sort_by(|a, b| b.cmp(a));
    println!("{}", deck.iter().take(5).map(|c| format!("{}{}", c.rank, c.suit)).collect::<Vec<_>>().join(" "));
    let reds = deck.iter().filter(|c| c.suit.is_red()).count();
    let by_suit: HashMap<Suit, Vec<u8>> = deck.iter().fold(HashMap::new(), |mut m, c| {
        m.entry(c.suit).or_insert_with(Vec::new).push(c.rank);
        m
    });
    let mut suits: Vec<&Suit> = by_suit.keys().collect();
    suits.sort();
    println!("{reds} {:?} {:?} {}", suits, by_suit[&Suit::Hearts], Suit::Spades > Suit::Clubs);
    println!("{:?} {}", deck.iter().max(), deck.iter().min().map(|c| c.suit.symbol()).unwrap_or('?'));
    for s in ["42", " 7 ", "abc", "250", "-1", ""] {
        match parse_percent(s) {
            Ok(v) => println!("ok {v}"),
            Err(e) => println!("err {e} / {e:?}"),
        }
    }
    let results: Result<Vec<u8>, AppError> = ["1", "2", "x"].iter().map(|s| parse_percent(s)).collect();
    println!("{}", results.map(|v| v.len()).unwrap_or_else(|e| e.to_string().len()));
    println!("{:?} {:?} {:?}", first_even_square(&[3, 4, 5]), first_even_square(&[1, 3]), first_even_square(&[65536]));
    let o: Option<i32> = Some(4);
    println!("{:?} {:?} {:?} {:?} {:?}", o.zip(Some('a')), o.xor(None), o.filter(|x| x % 2 == 1), o.map_or(0, |x| x * 10), o.ok_or_else(|| "none".to_string()));
    println!("{:?} {:?} {}", o.and_then(|x| x.checked_sub(5)).or(Some(-1)), None::<i32>.unwrap_or_default(), o.is_some_and(|x| x > 3));
    let r: Result<i32, String> = Err("bad".into());
    println!("{:?} {:?} {} {:?}", r.as_ref().map(|x| x + 1), r.as_ref().map_err(|e| e.len()), r.clone().unwrap_or_else(|e| e.len() as i32), r.as_ref().ok());
    let p = Pipeline {
        steps: vec![("double".into(), Box::new(|x| x * 2)), ("dec".into(), Box::new(|x| x - 10))],
        fallback: Some(Box::new(|x| -x)),
    };
    let q = Pipeline { steps: vec![], fallback: None };
    println!("{} {} {} {}", p.run(3), p.run(10), q.run(-5), p.steps.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(">"));
    let mut memo = HashMap::new();
    println!("{} {}", memo_count(50, &mut memo), memo.len());
}
