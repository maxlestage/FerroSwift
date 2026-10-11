use std::collections::HashMap;
use std::ops::{AddAssign, Index, Mul, Neg, Not};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2 { x: self.x * k, y: self.y * k }
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Vec2 {
        Vec2 { x: self * v.x, y: self * v.y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2 { x: -self.x, y: -self.y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Flags(u8);

impl Not for Flags {
    type Output = Flags;
    fn not(self) -> Flags {
        Flags(!self.0)
    }
}

struct Config {
    values: HashMap<String, String>,
}

impl Index<&str> for Config {
    type Output = String;
    fn index(&self, key: &str) -> &String {
        &self.values[key]
    }
}

fn parse_all(items: &[&str]) -> Result<Vec<i32>, std::num::ParseIntError> {
    items.iter().map(|s| -> Result<i32, std::num::ParseIntError> { Ok(s.trim().parse::<i32>()? * 2) }).collect()
}

fn stats(values: &[f64]) -> (f64, f64, f64) {
    let (min, max, sum) = values.iter().fold((f64::INFINITY, f64::NEG_INFINITY, 0.0), |(lo, hi, s), &v| (lo.min(v), hi.max(v), s + v));
    (min, max, sum / values.len() as f64)
}

fn classify(n: Option<i32>, label: &str) -> String {
    fn describe(n: i32) -> &'static str {
        if n % 2 == 0 { "even" } else { "odd" }
    }
    if let Some(v) = n && v > 10 && label.len() > 2 {
        format!("big {} {}", describe(v), label)
    } else if let Some(v) = n {
        format!("small {}", describe(v))
    } else {
        "none".to_string()
    }
}

fn main() {
    let v = Vec2 { x: 1.5, y: -2.0 };
    let mut w = v * 2.0;
    w += 0.5 * v;
    println!("{:?} {:?} {:?} {}", w, -w, !Flags(0b1010_0101), (!Flags(1)).0);
    let cfg = Config { values: HashMap::from([("host".to_string(), "localhost".to_string()), ("port".to_string(), "8080".to_string())]) };
    println!("{}:{} {}", cfg["host"], cfg["port"], cfg["port"].len());

    println!("{:?} {:?}", parse_all(&["1", " 2 ", "3"]), parse_all(&["1", "x"]).map_err(|e| e.to_string()));
    println!("{:?}", stats(&[3.0, 1.0, 4.0, 1.5]));
    println!("{} | {} | {} | {}", classify(Some(12), "abc"), classify(Some(12), "ab"), classify(Some(3), "abc"), classify(None, "x"));

    let data = [1, 1, 2, 3, 3, 3, 4, 5, 5];
    let groups: Vec<Vec<i32>> = data.chunk_by(|a, b| a == b).map(|g| g.to_vec()).collect();
    println!("{:?}", groups);
    let runs: Vec<(i32, usize)> = data.chunk_by(|a, b| a == b).map(|g| (g[0], g.len())).collect();
    println!("{:?}", runs);
    println!("{:?} {:?}", data.chunks_exact(4).map(|c| c.iter().sum::<i32>()).collect::<Vec<_>>(), data.chunks_exact(4).remainder());
    println!("{:?}", data.rchunks(4).map(|c| c.len()).collect::<Vec<_>>());
    println!("{:?}", data.split(|&x| x == 3).map(|p| p.len()).collect::<Vec<_>>());
    let nested = vec![vec![1, 2], vec![], vec![3]];
    println!("{:?} {:?}", nested.concat(), nested.iter().map(|v| v.len()).collect::<Vec<_>>());
    let words = vec![vec!["a", "b"], vec!["c"]];
    println!("{:?} {}", words.concat(), words.iter().map(|w| w.join("+")).collect::<Vec<_>>().join(" | "));
    let mut names = vec!["bob", "Alice", "carol", "Dave"];
    names.sort_by_cached_key(|n| n.to_lowercase());
    println!("{:?}", names);
    println!("{:?}", (0..=255u8).step_by(51).collect::<Vec<_>>());
    println!("{:?}", (-3i8..=3).rev().step_by(2).collect::<Vec<_>>());
    println!("{} {} {}", 300u16.wrapping_mul(300), 65535u16.wrapping_add(2), 7u16.wrapping_sub(9));
    println!("{} {} {}", u64::MAX / 3, u64::MAX % 7, 10u64.pow(18));
    let mut acc = Vec2 { x: 0.0, y: 0.0 };
    for i in 1..=3 {
        acc += Vec2 { x: i as f64, y: 1.0 } * 0.5;
    }
    println!("{:?} {}", acc, acc == Vec2 { x: 3.0, y: 1.5 });
    let label = "  padded  ";
    let label = label.trim();
    let label = label.len();
    println!("{label}");
    let found = 'search: loop {
        for (i, row) in [[1, 2], [3, 4], [5, 6]].iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                if cell == 4 {
                    break 'search Some((i, j));
                }
            }
        }
        break None;
    };
    println!("{:?}", found);
}
