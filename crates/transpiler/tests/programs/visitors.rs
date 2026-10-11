use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

fn manhattan(Point { x, y }: Point) -> i32 {
    x.abs() + y.abs()
}

fn midpoint(&Point { x: x1, y: y1 }: &Point, (x2, y2): (i32, i32)) -> Point {
    Point { x: (x1 + x2) / 2, y: (y1 + y2) / 2 }
}

trait Visitor {
    fn number(&mut self, n: i64);
    fn text(&mut self, s: &str);
}

enum Item {
    Num(i64),
    Text(String),
    Group(Vec<Item>),
}

impl Item {
    fn accept(&self, v: &mut dyn Visitor) {
        match self {
            Item::Num(n) => v.number(*n),
            Item::Text(s) => v.text(s),
            Item::Group(items) => {
                for it in items {
                    it.accept(v);
                }
            }
        }
    }
}

struct Summer {
    total: i64,
    texts: usize,
}

impl Visitor for Summer {
    fn number(&mut self, n: i64) {
        self.total += n;
    }
    fn text(&mut self, _s: &str) {
        self.texts += 1;
    }
}

struct Collector(Vec<String>);

impl Visitor for Collector {
    fn number(&mut self, n: i64) {
        self.0.push(n.to_string());
    }
    fn text(&mut self, s: &str) {
        self.0.push(format!("{s:?}"));
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn main() {
    let p = Point { x: -3, y: 4 };
    println!("{} {:?}", manhattan(p), midpoint(&p, (5, 10)));
    let tree = Item::Group(vec![Item::Num(5), Item::Text("hi".into()), Item::Group(vec![Item::Num(-2), Item::Text("x y".into())]), Item::Num(10)]);
    let mut s = Summer { total: 0, texts: 0 };
    tree.accept(&mut s);
    let mut c = Collector(Vec::new());
    tree.accept(&mut c);
    println!("{} {} {}", s.total, s.texts, c.0.join(" "));
    let mut handlers: HashMap<String, Box<dyn Fn(i32) -> i32>> = HashMap::new();
    handlers.insert("double".to_string(), Box::new(|x| x * 2));
    let offset = 7;
    handlers.insert("shift".to_string(), Box::new(move |x| x + offset));
    handlers.insert("neg".to_string(), Box::new(|x: i32| -x));
    let mut names: Vec<&String> = handlers.keys().collect();
    names.sort();
    for name in names {
        println!("{name}: {}", handlers[name](5));
    }
    if let Some(h) = handlers.get("double") {
        println!("{}", h(h(3)));
    }
    let opt = Some(4);
    let threshold = 3;
    if let Some(v) = opt && v > threshold {
        println!("big {v}");
    }
    let words = vec!["apple", "kiwi"];
    if let Some(first) = words.first() && let Some(ch) = first.chars().next() && ch == 'a' {
        println!("starts with a: {first}");
    }
    let mut it = words.iter();
    while let Some(w) = it.next() && w.len() > 4 {
        println!("long {w}");
    }
    let bytes = b"Hello, World 42!";
    let digits: Vec<u8> = bytes.iter().copied().filter(u8::is_ascii_digit).collect();
    let upper: String = bytes.iter().map(|b| b.to_ascii_uppercase() as char).collect();
    let letters = bytes.iter().filter(|b| b.is_ascii_alphabetic()).count();
    println!("{:?} {} {} {}", digits, upper, letters, bytes[0].is_ascii_uppercase());
    println!("{:x} {:x} {}", fnv1a(b""), fnv1a(b"hello"), fnv1a(b"hello") % 1000);
    let mut rng = XorShift(0x2545F4914F6CDD1D);
    let draws: Vec<u64> = (0..5).map(|_| rng.below(100)).collect();
    println!("{:?} {}", draws, rng.next());
    let big = u64::MAX.wrapping_add(2).rotate_left(3) ^ 0xff;
    println!("{big} {} {}", big.count_ones(), (big as u32) as u8);
}
