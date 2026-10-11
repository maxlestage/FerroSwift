use std::collections::HashMap;

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String {
        "shape".to_string()
    }
    fn describe(&self) -> String {
        format!("{} with area {:.2}", self.name(), self.area())
    }
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
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
}

fn largest<'a>(shapes: &'a [Box<dyn Shape>]) -> &'a dyn Shape {
    let mut best = &shapes[0];
    for s in shapes {
        if s.area() > best.area() {
            best = s;
        }
    }
    best.as_ref()
}

fn apply<F: Fn(i32) -> i32>(f: F, times: usize, start: i32) -> i32 {
    (0..times).fold(start, |acc, _| f(acc))
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

#[derive(Debug, Default, Clone)]
struct Stats {
    count: usize,
    sum: f64,
    min: Option<f64>,
    max: Option<f64>,
}

impl Stats {
    fn add(&mut self, x: f64) -> &mut Self {
        self.count += 1;
        self.sum += x;
        self.min = Some(self.min.map_or(x, |m| m.min(x)));
        self.max = Some(match self.max {
            Some(m) if m > x => m,
            _ => x,
        });
        self
    }

    fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle(1.0)), Box::new(Rect { w: 2.0, h: 3.0 })];
    for s in &shapes {
        println!("{}", s.describe());
    }
    println!("{}", largest(&shapes).name());
    let total: f64 = shapes.iter().map(|s| s.area()).sum();
    println!("{total:.3}");
    println!("{}", apply(|x| x * 2, 5, 1));
    let add5 = make_adder(5);
    let mut counter = make_counter();
    println!("{} {} {} {}", add5(1), counter(), counter(), counter());
    let f = compose(|x: i32| x + 1, |y: i32| y * 10);
    println!("{}", f(4));
    let double_then_str = compose(|x: i32| x * 2, |y: i32| format!("<{y}>"));
    println!("{}", double_then_str(21));
    let mut stats = Stats::default();
    stats.add(3.0).add(-1.5).add(10.25);
    println!("{:?} {:?}", stats, stats.mean());
    println!("{:?}", Stats::default().mean());
    let mut cache: HashMap<u64, u64> = HashMap::new();
    fn fib(n: u64, cache: &mut HashMap<u64, u64>) -> u64 {
        if n < 2 {
            return n;
        }
        if let Some(&v) = cache.get(&n) {
            return v;
        }
        let v = fib(n - 1, cache) + fib(n - 2, cache);
        cache.insert(n, v);
        v
    }
    println!("{} {}", fib(50, &mut cache), cache.len());
    let data = [3, 1, 4, 1, 5, 9, 2, 6];
    let mut best = (0, 0);
    'outer: for i in 0..data.len() {
        for j in (i + 1)..data.len() {
            if data[i] + data[j] == 10 {
                best = (i, j);
                break 'outer;
            }
        }
    }
    println!("{:?}", best);
    let label = match data.len() {
        0 => "empty",
        1..=3 => "small",
        n if n % 2 == 0 => "even",
        _ => "odd",
    };
    println!("{label}");
    let v = data.to_vec();
    match v.as_slice() {
        [] => println!("none"),
        [one] => println!("one {one}"),
        [first, .., last] => println!("{first}..{last}"),
    }
    if let [a, b, rest @ ..] = &data[..] {
        println!("{a} {b} {}", rest.len());
    }
    let x = 5i32;
    let x = x * 2;
    let x = format!("{x}!");
    println!("{x}");
    let (mut lo, mut hi) = (0usize, data.len());
    while lo < hi {
        let mid = (lo + hi) / 2;
        if data[mid] < 4 {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    println!("{lo}");
    let mut stack = vec![1];
    let mut log = Vec::new();
    while let Some(top) = stack.pop() {
        log.push(top);
        if top < 4 {
            stack.push(top * 2);
            stack.push(top * 2 + 1);
        }
    }
    println!("{:?}", log);
    let c = 'ß';
    println!("{} {} {} {:?}", c.is_lowercase(), c.len_utf8(), c.to_uppercase(), c.to_uppercase().collect::<String>());
    println!("{} {} {}", 'a' as u32, (b'a' + 1) as char, char::from_digit(7, 10).unwrap());
    println!("{:?} {:?}", 'x'.to_digit(10), 'f'.to_digit(16));
    let opt: Option<&str> = Some("hi");
    println!("{:?} {:?} {:?}", opt.map(str::len), opt.filter(|s| s.len() > 5), opt.and_then(|s| s.chars().last()));
    println!("{:?} {:?}", opt.xor(None), opt.or(Some("other")));
    println!("{}", opt.map_or(0, |s| s.len()) + opt.map_or_else(|| 100, |s| s.len() * 2));
    let n: Option<i32> = None;
    println!("{} {:?} {:?}", n.unwrap_or_default(), n.or_else(|| Some(9)), n.is_none());
    println!("{:?}", Some(3).zip(Some('c')));
    println!("{:?}", Some(4).take_if(|v| *v > 3));
    let words = ["apple", "banana", "cherry"];
    let lens: HashMap<&str, usize> = words.iter().map(|w| (*w, w.len())).collect();
    let mut keys: Vec<_> = lens.keys().copied().collect();
    keys.sort_unstable();
    println!("{:?} {}", keys, lens["banana"]);
    println!("{}", i32::MAX.wrapping_add(1));
    println!("{:?} {:?}", 200u8.checked_add(100), 200u8.saturating_add(100));
    println!("{} {}", (-7i32).rem_euclid(3), (-7i32) % 3);
    println!("{} {} {}", 7 / 2, -7 / 2, 7.0 / 2.0);
    println!("{:.1} {:e} {}", 1234.5678, 1234.5678, 1e21);
    println!("{:5}|{:<5}|{:^5}|{:>5}|{:05}", 42, 42, 42, 42, -42);
    println!("{:+} {:#x} {:#b} {:08.3}", 7, 255, 5, 3.14159);
    println!("{0} {1} {0} {name}", "a", "b", name = "n");
    let width = 8;
    println!("[{:>width$}] [{:>1$}] [{:.*}]", "r", 6, 2, 1.23456);
}
