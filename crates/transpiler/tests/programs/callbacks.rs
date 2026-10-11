use std::collections::HashMap;

struct Debouncer<F: FnMut(&str) -> usize> {
    action: F,
    calls: usize,
}

impl<F: FnMut(&str) -> usize> Debouncer<F> {
    fn new(action: F) -> Self {
        Debouncer { action, calls: 0 }
    }

    fn trigger(&mut self, input: &str) -> usize {
        self.calls += 1;
        if input.is_empty() {
            return 0;
        }
        (self.action)(input)
    }
}

struct Transformer<F>
where
    F: Fn(i32) -> i32,
{
    f: F,
    name: &'static str,
}

impl<F: Fn(i32) -> i32> Transformer<F> {
    fn apply_all(&self, xs: &[i32]) -> Vec<i32> {
        xs.iter().map(|&x| (self.f)(x)).collect()
    }
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn mul(a: i32, b: i32) -> i32 {
    a * b
}

fn apply(op: fn(i32, i32) -> i32, a: i32, b: i32) -> i32 {
    op(a, b)
}

fn pick(name: &str) -> Option<fn(i32, i32) -> i32> {
    match name {
        "add" => Some(add),
        "mul" => Some(mul),
        _ => None,
    }
}

fn main() {
    let mut total = 0;
    let mut d = Debouncer::new(|s: &str| {
        total += s.len();
        s.len()
    });
    println!("{} {} {}", d.trigger("abc"), d.trigger(""), d.trigger("hello"));
    let calls = d.calls;
    drop(d);
    println!("{calls} {total}");

    let t = Transformer { f: |x| x * x - 1, name: "square-1" };
    println!("{} {:?}", t.name, t.apply_all(&[1, 2, 3]));
    let offset = 10;
    let shifted = Transformer { f: move |x| x + offset, name: "shift" };
    println!("{} {:?}", shifted.name, shifted.apply_all(&[-10, 0]));


    let ops: Vec<(&str, fn(i32, i32) -> i32)> = vec![("add", add), ("mul", mul)];
    for (name, op) in &ops {
        println!("{name}: {} {}", op(3, 4), apply(*op, 10, 20));
    }
    let table: HashMap<&str, fn(i32, i32) -> i32> = ops.iter().copied().collect();
    let mut keys: Vec<&&str> = table.keys().collect();
    keys.sort();
    println!("{:?} {}", keys, table["mul"](6, 7));
    println!("{:?} {:?}", pick("add").map(|f| f(1, 2)), pick("pow").is_none());
    let chosen: fn(i32, i32) -> i32 = if ops.len() > 1 { mul } else { add };
    println!("{}", chosen(5, 5));
    let mut steps: Vec<Box<dyn FnMut() -> i32>> = Vec::new();
    let mut counter = 0;
    steps.push(Box::new(move || {
        counter += 1;
        counter
    }));
    let mut other = 100;
    steps.push(Box::new(move || {
        other -= 10;
        other
    }));
    for _ in 0..2 {
        for step in steps.iter_mut() {
            print!("{} ", step());
        }
    }
    println!();
    println!("{}", counter);
}
