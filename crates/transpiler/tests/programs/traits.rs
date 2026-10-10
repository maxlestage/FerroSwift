use std::fmt;
use std::ops::Add;

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
        "Dog".to_string()
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
        "meow".to_string()
    }
    fn speak(&self) -> String {
        format!("The cat purrs ({} lives)", self.lives)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2 { x: self.x + other.x, y: self.y + other.y }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
    }
}

trait Describe {
    fn describe(&self) -> String;
}

impl Describe for Vec2 {
    fn describe(&self) -> String {
        format!("vector of length {:.2}", (self.x * self.x + self.y * self.y).sqrt())
    }
}

fn print_all<T: Describe>(items: &[T]) {
    for item in items {
        println!("- {}", item.describe());
    }
}

fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut largest = list[0];
    for &item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

struct Countdown {
    remaining: u32,
}

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            Some(self.remaining + 1)
        }
    }
}

fn main() {
    let animals: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat { lives: 9 })];
    for a in &animals {
        println!("{}", a.speak());
    }
    let v = Vec2 { x: 1.0, y: 2.0 } + Vec2 { x: 0.5, y: -1.0 };
    println!("{} {:?}", v, v);
    print_all(&[v, Vec2 { x: 3.0, y: 4.0 }]);
    println!("largest: {} {} {}", largest(&[3, 9, 2]), largest(&[1.5, 0.2]), largest(&['a', 'q', 'c']));
    let cd = Countdown { remaining: 3 };
    for n in cd {
        print!("{n}..");
    }
    println!("liftoff");
}
