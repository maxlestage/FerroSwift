use std::fmt;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

impl<T: Copy + std::ops::Add<Output = T>> Tree<T> {
    fn sum(&self) -> T {
        match self {
            Tree::Leaf(v) => *v,
            Tree::Node(l, r) => l.sum() + r.sum(),
        }
    }
    fn depth(&self) -> usize {
        match self {
            Tree::Leaf(_) => 1,
            Tree::Node(l, r) => 1 + l.depth().max(r.depth()),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tree::Leaf(v) => write!(f, "{v}"),
            Tree::Node(l, r) => write!(f, "({l} {r})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Pair<T> {
    first: T,
    second: T,
}

impl<T: PartialOrd + Clone> Pair<T> {
    fn max(&self) -> T {
        if self.first >= self.second { self.first.clone() } else { self.second.clone() }
    }
    fn swapped(&self) -> Pair<T> {
        Pair { first: self.second.clone(), second: self.first.clone() }
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, x: T) {
        self.items.push(x);
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T> Index<usize> for Stack<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.items[i]
    }
}

trait Animal {
    fn new(name: &str) -> Self
    where
        Self: Sized;
    fn name(&self) -> String;
    fn sound(&self) -> String;
    fn speak(&self) -> String {
        format!("{} says {}", self.name(), self.sound())
    }
}

struct Dog(String);
struct Cat {
    name: String,
    lives: u8,
}

impl Animal for Dog {
    fn new(name: &str) -> Self {
        Dog(name.to_string())
    }
    fn name(&self) -> String {
        self.0.clone()
    }
    fn sound(&self) -> String {
        "woof".into()
    }
}

impl Animal for Cat {
    fn new(name: &str) -> Self {
        Cat { name: name.to_string(), lives: 9 }
    }
    fn name(&self) -> String {
        format!("{} ({} lives)", self.name, self.lives)
    }
    fn sound(&self) -> String {
        "meow".into()
    }
    fn speak(&self) -> String {
        format!("{} purrs", self.name)
    }
}

fn make<T: Animal>(name: &str) -> T {
    T::new(name)
}

fn chorus(animals: &[Box<dyn Animal>]) -> String {
    animals.iter().map(|a| a.speak()).collect::<Vec<_>>().join("; ")
}

fn map_all<T, U, F: Fn(&T) -> U>(items: &[T], f: F) -> Vec<U> {
    items.iter().map(f).collect()
}

fn apply_twice<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32,
{
    f(f(x))
}

fn total(it: impl Iterator<Item = u32>) -> u32 {
    it.sum()
}

fn average<T: Into<f64> + Copy>(xs: &[T]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().map(|&x| x.into()).sum::<f64>() / xs.len() as f64
}

fn largest<T: PartialOrd>(xs: &[T]) -> Option<&T> {
    let mut best = xs.first()?;
    for x in xs {
        if x > best {
            best = x;
        }
    }
    Some(best)
}

fn main() {
    let t = Tree::Node(Box::new(Tree::Leaf(1)), Box::new(Tree::Node(Box::new(Tree::Leaf(2)), Box::new(Tree::Leaf(3)))));
    println!("{t} {} {} {:?}", t.sum(), t.depth(), Tree::Leaf(2.5).sum());
    println!("{:?} {}", Tree::Node(Box::new(Tree::Leaf("a")), Box::new(Tree::Leaf("b"))), t == t.clone());
    let p = Pair { first: 3, second: 8 };
    let q: Pair<String> = Pair { first: "x".into(), second: "a".into() };
    println!("{} {} {:?} {:?} {}", p.max(), q.max(), p.swapped(), Pair::<f64>::default(), p == p.swapped().swapped());
    let mut s: Stack<&str> = Stack::new();
    s.push("a");
    s.push("b");
    s.push("c");
    println!("{} {:?} {} {}", s.len(), s.pop(), s[0], s[s.len() - 1]);
    let d: Dog = make("rex");
    let c = make::<Cat>("tom");
    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(d), Box::new(c), Box::new(Dog::new("fido"))];
    println!("{}", chorus(&zoo));
    println!("{:?} {:?}", map_all(&[1, 2, 3], |x| x * x), map_all(&["a", "bb"], |s| s.len()));
    println!("{} {} {}", apply_twice(|x| x + 3, 1), apply_twice(|x| x * x, 3), total([1u32, 2, 3].into_iter()));
    println!("{} {} {} {}", average(&[1u8, 2, 4]), average(&[1.5f32, 2.5]), average::<i32>(&[]), total((1..=4).map(|x| x * 2)));
    println!("{:?} {:?} {:?}", largest(&[3, 9, 2]), largest(&["b", "a"]), largest::<f64>(&[]));
}
