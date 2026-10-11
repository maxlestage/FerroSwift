use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
struct Config {
    name: String,
    retries: u32,
    verbose: bool,
    tags: Vec<String>,
    limits: Option<(u32, u32)>,
}

impl Config {
    fn with_name(name: &str) -> Self {
        Config { name: name.to_string(), retries: 3, ..Default::default() }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri(f64, f64, f64),
    Empty,
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { r } => std::f64::consts::PI * r * r,
            Shape::Rect { w, h } => w * h,
            Shape::Tri(a, b, c) => {
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
            Shape::Empty => 0.0,
        }
    }

    fn scale(&mut self, k: f64) {
        match self {
            Shape::Circle { r } => *r *= k,
            Shape::Rect { w, h } => {
                *w *= k;
                *h *= k;
            }
            Shape::Tri(a, b, c) => {
                *a *= k;
                *b *= k;
                *c *= k;
            }
            Shape::Empty => {}
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Shape::Circle { .. } => "circle",
            Shape::Rect { w, h } if (w - h).abs() < 1e-9 => "square",
            Shape::Rect { .. } => "rect",
            Shape::Tri(a, b, c) if a == b && b == c => "equilateral",
            Shape::Tri(..) => "triangle",
            Shape::Empty => "empty",
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Circle { r } => write!(f, "circle(r={r})"),
            Shape::Rect { w, h } => write!(f, "rect({w}x{h})"),
            Shape::Tri(a, b, c) => write!(f, "tri({a},{b},{c})"),
            Shape::Empty => f.write_str("empty"),
        }
    }
}

trait Visitor {
    fn visit(&mut self, shape: &Shape);
    fn finish(&self) -> String;
}

struct AreaSum(f64);
struct Counter(HashMap<&'static str, usize>);

impl Visitor for AreaSum {
    fn visit(&mut self, shape: &Shape) {
        self.0 += shape.area();
    }
    fn finish(&self) -> String {
        format!("{:.2}", self.0)
    }
}

impl Visitor for Counter {
    fn visit(&mut self, shape: &Shape) {
        *self.0.entry(shape.kind()).or_insert(0) += 1;
    }
    fn finish(&self) -> String {
        let mut keys: Vec<_> = self.0.iter().collect();
        keys.sort();
        keys.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(",")
    }
}

fn walk(shapes: &[Shape], visitor: &mut dyn Visitor) -> String {
    for s in shapes {
        visitor.visit(s);
    }
    visitor.finish()
}

struct Fib {
    a: u64,
    b: u64,
}

impl Iterator for Fib {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let out = self.a;
        self.a = self.b;
        self.b += out;
        Some(out)
    }
}

fn group_by_len<'a>(words: &[&'a str]) -> BTreeMap<usize, Vec<&'a str>> {
    let mut out: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for w in words {
        out.entry(w.len()).or_default().push(w);
    }
    out
}

fn main() {
    let base = Config::with_name("base");
    let dev = Config { verbose: true, tags: vec!["dev".into()], ..base.clone() };
    let limited = Config { limits: Some((1, 10)), ..dev.clone() };
    println!("{:?}", base);
    println!("{:?} {}", dev.tags, dev.retries);
    println!("{:?} {}", limited.limits, limited == dev);
    println!("{:#?}", limited);
    let mut shapes = vec![
        Shape::Circle { r: 1.0 },
        Shape::Rect { w: 2.0, h: 2.0 },
        Shape::Rect { w: 2.0, h: 3.0 },
        Shape::Tri(3.0, 4.0, 5.0),
        Shape::Tri(2.0, 2.0, 2.0),
        Shape::Empty,
    ];
    for s in &shapes {
        println!("{s} {} {:.3}", s.kind(), s.area());
    }
    shapes.iter_mut().for_each(|s| s.scale(2.0));
    shapes[0].scale(0.5);
    println!("{}", shapes.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" | "));
    let mut sum = AreaSum(0.0);
    let mut counter = Counter(HashMap::new());
    println!("{} {}", walk(&shapes, &mut sum), walk(&shapes, &mut counter));
    let big: Vec<&Shape> = shapes.iter().filter(|s| s.area() > 10.0).collect();
    println!("{}", big.len());
    if let Some(Shape::Rect { w, .. }) = shapes.get(1) {
        println!("w={w}");
    }
    let total_r: f64 = shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Circle { r } => Some(*r),
            _ => None,
        })
        .sum();
    println!("{total_r}");
    let fib: Vec<u64> = Fib { a: 0, b: 1 }.take(10).collect();
    println!("{:?}", fib);
    let even_sum: u64 = Fib { a: 0, b: 1 }.take_while(|&x| x < 100).filter(|x| x % 2 == 0).sum();
    println!("{even_sum}");
    let mut it = Fib { a: 0, b: 1 }.skip(5).step_by(2);
    println!("{:?} {:?} {:?}", it.next(), it.next(), it.nth(2));
    let groups = group_by_len(&["a", "bb", "cc", "d", "eee"]);
    println!("{:?}", groups);
    for (len, ws) in &groups {
        println!("{len}: {}", ws.join("/"));
    }
    let first_long = groups.iter().find(|(l, _)| **l > 1).map(|(_, ws)| ws[0]);
    println!("{:?} {:?}", first_long, groups.keys().next_back());
    let (small, large): (Vec<_>, Vec<_>) = groups.into_iter().partition(|(l, _)| *l < 2);
    println!("{} {}", small.len(), large.len());
    let matrix = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let transposed: Vec<Vec<i32>> = (0..3).map(|c| matrix.iter().map(|row| row[c]).collect()).collect();
    println!("{:?}", transposed);
    let diag: i32 = (0..2).map(|i| matrix[i][i]).sum();
    let flat_max = matrix.iter().flat_map(|r| r.iter()).max().copied();
    println!("{diag} {:?} {:?}", flat_max, matrix.concat());
    let mut grid = [[0u8; 3]; 2];
    grid[1][2] = 7;
    grid[0][0] += 1;
    println!("{:?} {}", grid, grid.len() * grid[0].len());
    let pairs: Vec<(usize, char)> = "abc".chars().enumerate().collect();
    let (idx, chs): (Vec<usize>, Vec<char>) = pairs.iter().cloned().unzip();
    println!("{:?} {:?} {:?}", pairs, idx, chs);
    let lookup: HashMap<char, usize> = chs.iter().copied().zip(idx.iter().copied()).collect();
    let mut found: Vec<_> = lookup.iter().filter(|(c, _)| **c != 'b').map(|(c, i)| format!("{c}{i}")).collect();
    found.sort();
    println!("{:?} {:?}", found, lookup.get(&'z').copied().unwrap_or(99));
}
