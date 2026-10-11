use std::collections::HashMap;
use std::ops::{Deref, Div, Range, RangeInclusive, Rem};

#[derive(Debug, Clone)]
struct Item {
    id: u32,
    name: String,
    price: f64,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Money(i64);

impl Div<i64> for Money {
    type Output = Money;
    fn div(self, rhs: i64) -> Money {
        Money(self.0 / rhs)
    }
}

impl Rem<i64> for Money {
    type Output = i64;
    fn rem(self, rhs: i64) -> i64 {
        self.0 % rhs
    }
}

struct Name(String);

impl Deref for Name {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

struct Button {
    label: String,
    on_click: Box<dyn Fn(i32) -> i32>,
}

impl Button {
    fn new(label: &str, on_click: Box<dyn Fn(i32) -> i32>) -> Button {
        Button { label: label.to_string(), on_click }
    }

    fn click(&self, x: i32) -> i32 {
        (self.on_click)(x)
    }
}

struct Slot {
    hours: RangeInclusive<u8>,
    seats: Range<usize>,
}

impl Slot {
    fn open_at(&self, h: u8) -> bool {
        self.hours.contains(&h)
    }

    fn capacity(&self) -> usize {
        self.seats.len()
    }
}

trait Converter<T> {
    fn convert(&self, x: T) -> String;
}

struct Celsius;

impl Converter<f64> for Celsius {
    fn convert(&self, x: f64) -> String {
        format!("{:.1}°F", x * 9.0 / 5.0 + 32.0)
    }
}

impl Converter<i32> for Celsius {
    fn convert(&self, x: i32) -> String {
        format!("{}°C", x)
    }
}

fn run_once<F: FnOnce() -> String>(f: F) -> String {
    let s = f();
    format!("[{}]", s)
}

fn apply_twice<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32,
{
    f(f(x))
}

fn main() {
    let a = Item { id: 1, name: "pen".into(), price: 1.5 };
    let b = Item { id: 1, name: "pencil".into(), price: 0.5 };
    let c = Item { id: 2, name: "pen".into(), price: 1.5 };
    println!("{} {} {}", a == b, a == c, a != c);
    let items = vec![a.clone(), b.clone(), c.clone()];
    println!("contains c: {}", items.contains(&c));
    println!("position of id 2: {:?}", items.iter().position(|i| *i == c));
    let found = items.iter().filter(|i| **i == a).count();
    println!("equal to a: {}", found);

    let m = Money(1000);
    println!("{:?} {:?} {}", m / 3, m / 7, m % 7);
    println!("{}", m / 2 == Money(500));

    let n = Name("Ada Lovelace".to_string());
    println!("{} {} {}", n.len(), n.to_uppercase(), n.starts_with("Ada"));
    let words: Vec<&str> = n.split(' ').collect();
    println!("{:?}", words);
    println!("{}", *n);

    let scale = 3;
    let button = Button::new("triple", Box::new(move |x| x * scale));
    let dec = Button { label: "dec".to_string(), on_click: Box::new(|x| x - 1) };
    println!("{} -> {}", button.label, button.click(14));
    println!("{} -> {}", dec.label, dec.click(14));
    let mut registry: HashMap<String, Box<dyn Fn(i32) -> i32>> = HashMap::new();
    registry.insert("double".to_string(), Box::new(|x| x * 2));
    registry.insert("square".to_string(), Box::new(|x| x * x));
    registry.insert("neg".to_string(), Box::new(|x: i32| -x));
    let mut names: Vec<&String> = registry.keys().collect();
    names.sort();
    for name in names {
        let f = &registry[name];
        println!("{}(7) = {}", name, f(7));
    }
    if let Some(f) = registry.get("square") {
        println!("square(9) = {}", f(9));
    }

    let slot = Slot { hours: 9..=17, seats: 10..25 };
    println!("{} {} {} cap {}", slot.open_at(8), slot.open_at(9), slot.open_at(17), slot.capacity());
    println!("{:?} {:?} {} {}", slot.hours, slot.seats, slot.hours.start(), slot.seats.end);
    let total: usize = slot.seats.clone().sum();
    println!("seat sum {}", total);
    let evening: RangeInclusive<u8> = 18..=22;
    println!("{}", evening.contains(&20) && !slot.hours.contains(&20));

    let conv = Celsius;
    println!("{} {}", conv.convert(100.0), conv.convert(21));

    let greeting = String::from("hello");
    println!("{}", run_once(move || greeting + " world"));
    println!("{}", run_once(|| "plain".to_string()));
    println!("{}", apply_twice(|x| x * x + 1, 2));
    println!("{}", apply_twice(&|x: i32| x - 3, 10));
}
