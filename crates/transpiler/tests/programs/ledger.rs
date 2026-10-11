use std::collections::BTreeMap;
use std::fmt::Display;

struct Ledger {
    entries: Vec<(String, i64)>,
}

impl Ledger {
    fn credits(&self) -> impl Iterator<Item = &(String, i64)> + '_ {
        self.entries.iter().filter(|(_, v)| *v > 0)
    }

    fn names(&self) -> impl Iterator<Item = String> + '_ {
        self.entries.iter().map(|(n, _)| n.to_uppercase())
    }

    fn total(&self) -> i64 {
        self.entries.iter().map(|(_, v)| v).sum()
    }
}

fn adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn compose<F, G>(f: F, g: G) -> impl Fn(i32) -> i32
where
    F: Fn(i32) -> i32,
    G: Fn(i32) -> i32,
{
    move |x| g(f(x))
}

fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

struct Pipeline {
    steps: Vec<Box<dyn Fn(f64) -> f64>>,
    name: String,
}

impl Pipeline {
    fn new(name: &str) -> Self {
        Pipeline { steps: Vec::new(), name: name.to_string() }
    }

    fn add(mut self, step: impl Fn(f64) -> f64 + 'static) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    fn run(&self, input: f64) -> f64 {
        self.steps.iter().fold(input, |acc, step| step(acc))
    }
}

trait Describe {
    const KIND: &'static str;
    fn name(&self) -> String;
    fn describe(&self) -> String {
        format!("{} {} ({})", Self::KIND, self.name(), self.name().len())
    }
}

struct Dog;
struct Robot {
    id: u32,
}

impl Describe for Dog {
    const KIND: &'static str = "animal";
    fn name(&self) -> String {
        "Rex".to_string()
    }
}

impl Describe for Robot {
    const KIND: &'static str = "machine";
    fn name(&self) -> String {
        format!("RB-{}", self.id)
    }
    fn describe(&self) -> String {
        format!("{} #{}", Self::KIND, self.id)
    }
}

fn show_all<T>(items: &[T]) -> String
where
    T: Display,
{
    items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("|")
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser { src, pos: 0 }
    }

    fn next_word(&mut self) -> Option<&'a str> {
        let rest = &self.src[self.pos..];
        let rest_trimmed = rest.trim_start();
        self.pos += rest.len() - rest_trimmed.len();
        if rest_trimmed.is_empty() {
            return None;
        }
        let end = rest_trimmed.find(' ').unwrap_or(rest_trimmed.len());
        self.pos += end;
        Some(&rest_trimmed[..end])
    }
}

fn main() {
    let ledger = Ledger {
        entries: vec![("rent".into(), -900), ("salary".into(), 2500), ("gift".into(), 100)],
    };
    let credits: Vec<&(String, i64)> = ledger.credits().collect();
    println!("{:?} total={}", credits, ledger.total());
    println!("{}", ledger.names().collect::<Vec<_>>().join(","));
    for (n, v) in ledger.credits() {
        println!("  {n}: {v}");
    }
    let add5 = adder(5);
    let both = compose(adder(1), |x| x * 10);
    println!("{} {} {}", add5(1), both(2), compose(add5, adder(-3))(0));
    let mut counter = make_counter();
    counter();
    counter();
    println!("counter={}", counter());
    let p = Pipeline::new("scale").add(|x| x * 2.0).add(|x| x - 1.0).add(f64::sqrt);
    println!("{} {} {}", p.name, p.run(5.0), p.steps.len());
    println!("{} / {}", Dog.describe(), Robot { id: 7 }.describe());
    println!("{} {}", Dog::KIND, <Robot as Describe>::KIND);
    println!("{}", show_all(&[1.5, 2.0, -0.25]));
    println!("{}", show_all(&["a", "b"]));
    let mut map = BTreeMap::new();
    for (i, w) in ["delta", "alpha", "charlie", "bravo", "echo"].iter().enumerate() {
        map.insert(*w, i * 10);
    }
    let middle: Vec<_> = map.range("b".."d").collect();
    println!("{:?}", middle);
    println!("{:?} {:?}", map.first_key_value(), map.last_key_value());
    println!("{:?}", map.range("c"..).map(|(k, _)| *k).collect::<Vec<_>>());
    println!("{:?}", map.range(..="bravo").count());
    let mut parser = Parser::new("  let x =  42 ;");
    let mut words = Vec::new();
    while let Some(w) = parser.next_word() {
        words.push(w);
    }
    println!("{:?} pos={}", words, parser.pos);
    let nums = BTreeMap::from([(3, 'c'), (1, 'a'), (7, 'g'), (5, 'e')]);
    println!("{:?}", nums.range(2..6).collect::<Vec<_>>());
    println!("{:?}", nums.range(4..).next());
    println!("{:?}", nums.range(..4).next_back());
}
