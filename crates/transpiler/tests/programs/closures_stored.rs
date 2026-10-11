use std::collections::HashMap;

struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn Fn(&str) -> String>>>,
}

impl EventBus {
    fn new() -> Self { EventBus { handlers: HashMap::new() } }
    fn on(&mut self, ev: &str, f: Box<dyn Fn(&str) -> String>) {
        self.handlers.entry(ev.to_string()).or_default().push(f);
    }
    fn emit(&self, ev: &str, payload: &str) -> Vec<String> {
        self.handlers.get(ev).map(|hs| hs.iter().map(|h| h(payload)).collect()).unwrap_or_default()
    }
}

fn apply_n<F: FnMut() -> i32>(n: usize, mut f: F) -> Vec<i32> {
    (0..n).map(|_| f()).collect()
}

fn make_counter(step: i32) -> impl FnMut() -> i32 {
    let mut c = 0;
    move || { c += step; c }
}

fn twice(f: fn(i32) -> i32, x: i32) -> i32 { f(f(x)) }
fn inc(x: i32) -> i32 { x + 1 }
fn sq(x: i32) -> i32 { x * x }

struct Memo<F: Fn(u64) -> u64> { f: F, cache: HashMap<u64, u64> }
impl<F: Fn(u64) -> u64> Memo<F> {
    fn get(&mut self, x: u64) -> u64 {
        if let Some(&v) = self.cache.get(&x) { return v; }
        let v = (self.f)(x);
        self.cache.insert(x, v);
        v
    }
}

fn main() {
    let mut bus = EventBus::new();
    let prefix = String::from(">>");
    bus.on("greet", Box::new(move |p| format!("{prefix} hello {p}")));
    bus.on("greet", Box::new(|p| p.to_uppercase()));
    bus.on("bye", Box::new(|p| format!("bye {}", p.len())));
    println!("{:?}", bus.emit("greet", "ann"));
    println!("{:?} {:?}", bus.emit("bye", "bob"), bus.emit("none", "x"));
    println!("{:?}", apply_n(4, make_counter(3)));
    let mut total = 0;
    let v = apply_n(3, || { total += 10; total });
    println!("{:?} {total}", v);
    let ops: HashMap<&str, fn(i32) -> i32> = [("inc", inc as fn(i32) -> i32), ("sq", sq)].into_iter().collect();
    let mut names: Vec<_> = ops.keys().copied().collect();
    names.sort();
    for n in names { println!("{n}: {}", twice(ops[n], 3)); }
    let pipeline: Vec<Box<dyn Fn(i32) -> i32>> = vec![Box::new(inc), Box::new(sq), Box::new(|x| x - 7)];
    println!("{}", pipeline.iter().fold(5, |acc, f| f(acc)));
    let mut m = Memo { f: |x| (1..=x).product(), cache: HashMap::new() };
    println!("{} {} {}", m.get(10), m.get(5), m.cache.len());
    let adders: Vec<Box<dyn Fn(i32) -> i32>> = (1..=3).map(|k| Box::new(move |x| x + k) as Box<dyn Fn(i32) -> i32>).collect();
    println!("{:?}", adders.iter().map(|f| f(10)).collect::<Vec<_>>());
}
