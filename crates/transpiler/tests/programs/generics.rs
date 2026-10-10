use std::fmt::Display;

#[derive(Debug, Clone)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone + Display> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
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

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn render(&self) -> String {
        self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(" | ")
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Pair<A, B> {
    first: A,
    second: B,
}

impl<A: Clone, B: Clone> Pair<A, B> {
    fn swap(&self) -> Pair<B, A> {
        Pair { first: self.second.clone(), second: self.first.clone() }
    }
}

fn describe<T: Display>(label: &str, value: T) -> String {
    format!("{label}: {value}")
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

fn compose(f: Box<dyn Fn(i32) -> i32>, g: Box<dyn Fn(i32) -> i32>) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| g(f(x)))
}

fn main() {
    let mut s: Stack<i32> = Stack::new();
    s.push(1);
    s.push(2);
    s.push(3);
    println!("{} len={} peek={:?}", s.render(), s.len(), s.peek());
    println!("pop {:?} {:?}", s.pop(), s.pop());
    println!("empty? {}", s.is_empty());
    let mut words = Stack::new();
    words.push("hello".to_string());
    words.push("world".to_string());
    println!("{}", words.render());

    let p = Pair { first: 1, second: "one" };
    let q = p.swap();
    println!("{:?} -> {:?}", p, q);
    println!("{}", describe("pi", 3.14));
    println!("{}", describe("name", "Ferris"));

    let square = |x: i32| x * x;
    println!("{} {}", apply(square, 7), apply_twice(|x| x + 3, 1));
    let add5 = make_adder(5);
    println!("{}", add5(10));
    let both = compose(Box::new(|x| x * 2), Box::new(|x| x - 1));
    println!("{}", both(10));
    let mut count = 0;
    let mut inc = || {
        count += 1;
    };
    inc();
    inc();
    println!("count {count}");
    let names = vec!["b", "a", "c"];
    let mut sorted = names.clone();
    sorted.sort_by_key(|s| s.to_string());
    println!("{:?}", sorted);
}
