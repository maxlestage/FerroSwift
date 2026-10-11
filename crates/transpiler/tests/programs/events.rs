use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn Fn(&str) -> String>>>,
    on_change: Option<Box<dyn Fn(usize)>>,
    counter: Box<dyn FnMut() -> u32>,
}

impl EventBus {
    fn new() -> Self {
        let mut n = 0;
        EventBus {
            handlers: HashMap::new(),
            on_change: None,
            counter: Box::new(move || {
                n += 1;
                n
            }),
        }
    }

    fn subscribe(&mut self, event: &str, f: impl Fn(&str) -> String + 'static) {
        self.handlers.entry(event.to_string()).or_default().push(Box::new(f));
        let total: usize = self.handlers.values().map(|v| v.len()).sum();
        if let Some(cb) = &self.on_change {
            cb(total);
        }
    }

    fn emit(&mut self, event: &str, payload: &str) -> Vec<String> {
        let tick = (self.counter)();
        match self.handlers.get(event) {
            Some(hs) => hs.iter().map(|h| format!("{tick}:{}", h(payload))).collect(),
            None => vec![format!("{tick}:nobody")],
        }
    }

    fn multiplier(&self, k: i32) -> impl Fn(i32) -> i32 + '_ {
        move |x| x * k + self.handlers.len() as i32
    }
}

fn apply_n(f: &dyn Fn(i32) -> i32, n: usize, x: i32) -> i32 {
    (0..n).fold(x, |acc, _| f(acc))
}

fn make_ops() -> Vec<(&'static str, Box<dyn Fn(f64, f64) -> f64>)> {
    vec![
        ("add", Box::new(|a, b| a + b)),
        ("hyp", Box::new(f64::hypot)),
        ("pow", Box::new(|a: f64, b| a.powf(b))),
    ]
}

fn main() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut bus = EventBus::new();
    let log2 = Rc::clone(&log);
    bus.on_change = Some(Box::new(move |n| log2.borrow_mut().push(format!("handlers={n}"))));
    let prefix = String::from(">> ");
    bus.subscribe("greet", move |p| format!("{prefix}hello {p}"));
    bus.subscribe("greet", |p| p.to_uppercase());
    let log3 = log.clone();
    bus.subscribe("log", move |p| {
        log3.borrow_mut().push(p.to_string());
        format!("logged {}", log3.borrow().len())
    });
    println!("{:?}", bus.emit("greet", "ann"));
    println!("{:?}", bus.emit("log", "first"));
    println!("{:?}", bus.emit("log", "second"));
    println!("{:?}", bus.emit("missing", "x"));
    println!("{:?} {}", log.borrow(), log.borrow().len());
    let times3 = bus.multiplier(3);
    println!("{} {}", times3(5), apply_n(&times3, 3, 1));
    println!("{}", apply_n(&|x| x - 1, 4, 10));
    for (name, op) in make_ops() {
        println!("{name}: {}", op(3.0, 4.0));
    }
    let mut words = vec!["pear".to_string(), "fig".into(), "banana".into()];
    let by_len = |a: &String, b: &String| a.len().cmp(&b.len());
    words.sort_by(by_len);
    println!("{:?} {:?}", words, words.iter().map(String::len).collect::<Vec<_>>());
    let mut history: Vec<Box<dyn FnMut() -> String>> = Vec::new();
    for i in 1..=3 {
        let mut calls = 0;
        history.push(Box::new(move || {
            calls += 1;
            format!("{i}/{calls}")
        }));
    }
    let outputs: Vec<String> = history.iter_mut().map(|f| f()).collect();
    let again: Vec<String> = history.iter_mut().map(|f| f()).collect();
    println!("{:?} {:?}", outputs, again);
    let fact: fn(u64) -> u64 = |n| (1..=n).product();
    let table: Vec<(u64, u64)> = (0..6).map(|n| (n, fact(n))).collect();
    println!("{:?}", table);
    let early = |v: &[i32]| -> Option<i32> {
        for &x in v {
            if x < 0 {
                return None;
            }
        }
        Some(v.iter().sum())
    };
    println!("{:?} {:?}", early(&[1, 2, 3]), early(&[1, -2]));
}
