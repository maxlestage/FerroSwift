use std::collections::HashMap;
use std::fmt;

/// A shape.
#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Empty,
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rect { w, h } => w * h,
            Shape::Empty => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn manhattan(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }

    fn translate(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug)]
enum ParseError {
    Empty,
    Bad(String),
}

fn parse_num(s: &str) -> Result<i32, ParseError> {
    if s.is_empty() {
        return Err(ParseError::Empty);
    }
    match s.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(ParseError::Bad(s.to_string())),
    }
}

fn sum_all(items: &[&str]) -> Result<i32, ParseError> {
    let mut total = 0;
    for item in items {
        total += parse_num(item)?;
    }
    Ok(total)
}

fn first_even(v: &[i32]) -> Option<i32> {
    let x = v.iter().find(|x| *x % 2 == 0)?;
    Some(*x * 10)
}

fn main() {
    let shapes = vec![Shape::Circle(1.0), Shape::Rect { w: 2.0, h: 3.5 }, Shape::Empty];
    for s in &shapes {
        println!("{:?} has area {:.2}", s, s.area());
    }
    let total: f64 = shapes.iter().map(|s| s.area()).sum();
    println!("total = {:.3}", total);

    let mut p = Point::new(3, -4);
    println!("p = {} manhattan {}", p, p.manhattan());
    p.translate(1, 1);
    println!("moved: {:?}", p);

    let mut counts: HashMap<String, i32> = HashMap::new();
    for word in "a b a c b a".split(' ') {
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }
    let mut keys: Vec<_> = counts.keys().cloned().collect();
    keys.sort();
    for k in &keys {
        println!("{} -> {}", k, counts[k]);
    }

    match sum_all(&["1", "2", "3"]) {
        Ok(n) => println!("sum = {}", n),
        Err(e) => println!("error {:?}", e),
    }
    match sum_all(&["1", "x"]) {
        Ok(n) => println!("sum = {}", n),
        Err(e) => println!("error {:?}", e),
    }
    if let Some(v) = first_even(&[1, 3, 4, 5]) {
        println!("first even x10 = {}", v);
    }
    let label = if total > 10.0 { "big" } else { "small" };
    println!("{label}");
    let squares: Vec<i32> = (1..=5).map(|i| i * i).collect();
    println!("{:?}", squares);
    let evens = squares.iter().filter(|&&x| x % 2 == 0).count();
    println!("evens: {}", evens);
}
