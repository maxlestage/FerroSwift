use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, PartialEq)]
struct Config { name: String, retries: u32, verbose: bool, ratio: f64, tags: Vec<String> }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Pos { x: i32, y: i32 }

struct User { name: String, email: String }

impl User {
    fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        User { name: name.into(), email: email.into() }
    }
}

fn shout<S: AsRef<str>>(s: S) -> String { s.as_ref().to_uppercase() }

fn total_len<I, S>(items: I) -> usize where I: IntoIterator<Item = S>, S: AsRef<str> {
    items.into_iter().map(|s| s.as_ref().len()).sum()
}

#[derive(Debug)]
struct Celsius(f64);
#[derive(Debug)]
struct Fahrenheit(f64);
impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self { Fahrenheit(c.0 * 9.0 / 5.0 + 32.0) }
}

fn main() {
    let base = Config { name: "base".into(), retries: 3, ..Default::default() };
    let custom = Config { verbose: true, tags: vec!["x".into()], ..base.clone() };
    println!("{:?}\n{:?} {}", base, custom, base == custom);
    let u = User::new("ann", String::from("a@x.io"));
    println!("{} {} {} {}", u.name, u.email, shout("hey"), shout(String::from("you")));
    println!("{} {}", total_len(vec!["ab", "cde"]), total_len(&["x".to_string()]));
    let f: Fahrenheit = Celsius(100.0).into();
    println!("{:?} {:?}", f, Fahrenheit::from(Celsius(-40.0)));
    let mut seen: HashSet<Pos> = HashSet::new();
    let moves = [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 0)];
    let mut p = Pos { x: 0, y: 0 };
    seen.insert(p);
    for (dx, dy) in moves {
        p = Pos { x: p.x + dx, y: p.y + dy };
        if !seen.insert(p) { println!("revisit {:?}", p); }
    }
    let mut cost: HashMap<Pos, u32> = HashMap::new();
    for (i, q) in seen.iter().enumerate() { cost.insert(*q, i as u32 * 0 + (q.x.abs() + q.y.abs()) as u32); }
    let mut keys: Vec<&Pos> = cost.keys().collect();
    keys.sort();
    println!("{:?}", keys.iter().map(|k| (k.x, k.y, cost[k])).collect::<Vec<_>>());
    let mut ratios = vec![0.5, 2.25, -1.0, 1.5];
    ratios.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let parsed: f64 = "1.5 x 2 3.25".split(' ').filter_map(|s| s.parse::<f64>().ok()).sum();
    println!("{:?} {}", ratios, parsed);
}
