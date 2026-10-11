use std::fmt::Display;

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String;
    fn describe(&self) -> String {
        format!("{} with area {:.2}", self.name(), self.area())
    }
    fn scaled(&self, k: f64) -> Box<dyn Shape>;
}

struct Circle(f64);
struct Rect {
    w: f64,
    h: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.0 * self.0
    }
    fn name(&self) -> String {
        "circle".into()
    }
    fn scaled(&self, k: f64) -> Box<dyn Shape> {
        Box::new(Circle(self.0 * k))
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
    fn name(&self) -> String {
        format!("rect {}x{}", self.w, self.h)
    }
    fn describe(&self) -> String {
        format!("a {} rectangle", if self.w == self.h { "square" } else { "plain" })
    }
    fn scaled(&self, k: f64) -> Box<dyn Shape> {
        Box::new(Rect { w: self.w * k, h: self.h * k })
    }
}

fn make(kind: &str, size: f64) -> Option<Box<dyn Shape>> {
    match kind {
        "circle" => Some(Box::new(Circle(size))),
        "square" => Some(Box::new(Rect { w: size, h: size })),
        _ => None,
    }
}

fn largest<'a>(shapes: &'a [Box<dyn Shape>]) -> Option<&'a dyn Shape> {
    shapes.iter().map(|s| s.as_ref()).max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
}

fn total_area(shapes: &[&dyn Shape]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

trait Container {
    type Item;
    fn get(&self, i: usize) -> Option<&Self::Item>;
    fn first(&self) -> Option<&Self::Item> {
        self.get(0)
    }
    fn size(&self) -> usize;
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, item: T) -> &mut Self {
        self.items.push(item);
        self
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
}

impl<T: Display> Stack<T> {
    fn render(&self) -> String {
        self.items.iter().map(|i| format!("<{i}>")).collect::<Vec<_>>().join("")
    }
}

impl<T> Container for Stack<T> {
    type Item = T;
    fn get(&self, i: usize) -> Option<&T> {
        self.items.get(i)
    }
    fn size(&self) -> usize {
        self.items.len()
    }
}

struct Countdown {
    from: u32,
}

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.from == 0 {
            None
        } else {
            self.from -= 1;
            Some(self.from + 1)
        }
    }
}

trait Named {
    fn name(&self) -> String;
}

trait Greeter: Named {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Person(String);

impl Named for Person {
    fn name(&self) -> String {
        self.0.clone()
    }
}

impl Greeter for Person {}

fn show_container<C: Container>(c: &C) -> String
where
    C::Item: Display,
{
    match c.first() {
        Some(x) => format!("first={x} size={}", c.size()),
        None => "empty".to_string(),
    }
}

fn sum_all<I>(iter: I) -> u32
where
    I: Iterator<Item = u32>,
{
    iter.fold(0, |acc, x| acc + x)
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle(1.0)), Box::new(Rect { w: 2.0, h: 3.0 }), Box::new(Rect { w: 2.0, h: 2.0 })];
    for s in &shapes {
        println!("{} | {}", s.describe(), s.scaled(2.0).describe());
    }
    println!("{:?}", largest(&shapes).map(|s| s.name()));
    let refs: Vec<&dyn Shape> = shapes.iter().map(|b| b.as_ref()).collect();
    println!("{:.3}", total_area(&refs));
    println!("{:?} {:?}", make("square", 3.0).map(|s| s.area()), make("hexagon", 1.0).is_none());
    let mut st: Stack<i32> = Stack::new();
    st.push(1).push(2).push(3);
    println!("{} {:?} {:?} {}", st.render(), st.peek(), st.get(1), st.size());
    println!("{:?} {} {}", st.pop(), st.render(), show_container(&st));
    let empty: Stack<String> = Stack::new();
    println!("{} {:?}", show_container(&empty), empty.first());
    let mut words = Stack::new();
    words.push("a".to_string());
    println!("{} {:?}", words.render(), words.first());
    println!("{:?}", Countdown { from: 4 }.collect::<Vec<_>>());
    println!("{} {}", sum_all(Countdown { from: 5 }), sum_all(Countdown { from: 3 }.map(|x| x * 10)));
    let cd = Countdown { from: 6 };
    println!("{:?}", cd.filter(|x| x % 2 == 0).map(|x| x * x).collect::<Vec<_>>());
    let p = Person("Ada".into());
    println!("{} {}", p.greet(), p.name());
    let mut it = Countdown { from: 2 };
    println!("{:?} {:?} {:?} {:?}", it.next(), it.next(), it.next(), it.next());
    println!("{:?}", Countdown { from: 3 }.zip(Countdown { from: 10 }).collect::<Vec<_>>());
    println!("{:?}", Countdown { from: 5 }.step_by(2).collect::<Vec<_>>());
    println!("{:?}", Countdown { from: 4 }.enumerate().last());
    println!("{:?} {:?}", Countdown { from: 4 }.max(), Countdown { from: 0 }.min());
    println!("{}", Countdown { from: 3 }.map(|x| x.to_string()).collect::<Vec<_>>().join(","));
}
