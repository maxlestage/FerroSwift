use std::collections::HashMap;

trait Scorer {
    const NAME: &'static str;
    fn score(&self, word: &str) -> u32;
    fn best<'a>(&self, words: &[&'a str]) -> Option<&'a str> {
        words.iter().copied().max_by_key(|w| self.score(w))
    }
}

struct Length;
struct Vowels {
    bonus: u32,
}

impl Scorer for Length {
    const NAME: &'static str = "length";
    fn score(&self, word: &str) -> u32 {
        word.len() as u32
    }
}

impl Scorer for Vowels {
    const NAME: &'static str = "vowels";
    fn score(&self, word: &str) -> u32 {
        word.chars().filter(|c| "aeiou".contains(*c)).count() as u32 * self.bonus
    }
}

fn report<S: Scorer>(scorer: &S, words: &[&str]) -> String {
    format!("{}: {:?}", S::NAME, scorer.best(words))
}

struct EventBus {
    handlers: Vec<Box<dyn Fn(&str) -> Option<String>>>,
    log: Vec<String>,
}

impl EventBus {
    fn new() -> Self {
        EventBus { handlers: Vec::new(), log: Vec::new() }
    }

    fn on(&mut self, handler: impl Fn(&str) -> Option<String> + 'static) {
        self.handlers.push(Box::new(handler));
    }

    fn emit(&mut self, event: &str) -> usize {
        let mut handled = 0;
        for h in &self.handlers {
            if let Some(msg) = h(event) {
                self.log.push(msg);
                handled += 1;
            }
        }
        handled
    }
}

fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|n| n % 2 == 0)
}

fn squares<'a>(values: &'a [i64]) -> impl Iterator<Item = i64> + 'a {
    values.iter().map(|v| v * v)
}

fn make_ops() -> Vec<(&'static str, fn(i64, i64) -> i64)> {
    fn add(a: i64, b: i64) -> i64 {
        a + b
    }
    fn mul(a: i64, b: i64) -> i64 {
        a * b
    }
    vec![("add", add), ("mul", mul), ("max", i64::max)]
}

fn apply_all(ops: &[(&str, fn(i64, i64) -> i64)], a: i64, b: i64) -> Vec<String> {
    ops.iter().map(|(name, f)| format!("{name}={}", f(a, b))).collect()
}

fn compose_all(fs: Vec<Box<dyn Fn(i32) -> i32>>) -> impl Fn(i32) -> i32 {
    move |x| fs.iter().fold(x, |acc, f| f(acc))
}

fn counter() -> impl FnMut() -> u32 {
    let mut n = 0;
    move || {
        n += 1;
        n * n
    }
}

fn main() {
    let words = ["sky", "queue", "strength", "aeiou"];
    println!("{}", report(&Length, &words));
    println!("{}", report(&Vowels { bonus: 2 }, &words));
    println!("{} {}", Length::NAME, <Vowels as Scorer>::NAME.len());
    let mut bus = EventBus::new();
    let prefix = String::from("got ");
    bus.on(move |e| if e.starts_with('a') { Some(format!("{prefix}{e}")) } else { None });
    bus.on(|e| Some(e.to_uppercase()));
    bus.on(|e| e.strip_prefix("x").map(|rest| rest.to_string()));
    println!("{} {} {}", bus.emit("apple"), bus.emit("xray"), bus.emit("b"));
    println!("{:?}", bus.log);
    println!("{:?} {}", evens(10).collect::<Vec<_>>(), evens(7).sum::<u32>());
    let vals = [3, -4, 5];
    println!("{:?} {}", squares(&vals).collect::<Vec<_>>(), squares(&vals).max().unwrap());
    let ops = make_ops();
    println!("{:?}", apply_all(&ops, 6, 7));
    let lookup: HashMap<&str, fn(i64, i64) -> i64> = ops.iter().cloned().collect();
    println!("{:?}", lookup.get("mul").map(|f| f(3, 4)));
    let pipeline = compose_all(vec![Box::new(|x| x + 1), Box::new(|x| x * 10), Box::new(|x| x - 3)]);
    println!("{} {}", pipeline(1), pipeline(-1));
    let mut next_square = counter();
    let firsts: Vec<u32> = (0..4).map(|_| next_square()).collect();
    println!("{:?}", firsts);
    let mut memo: HashMap<u64, u64> = HashMap::new();
    let mut collatz = |mut n: u64| -> u64 {
        let start = n;
        let mut steps = 0;
        while n != 1 {
            if let Some(&known) = memo.get(&n) {
                steps += known;
                break;
            }
            n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
            steps += 1;
        }
        memo.insert(start, steps);
        steps
    };
    println!("{} {} {}", collatz(6), collatz(27), collatz(6));
    println!("{}", memo.len());
    let adders: Vec<Box<dyn Fn(i32) -> i32>> = (1..=3).map(|k| Box::new(move |x| x + k) as Box<dyn Fn(i32) -> i32>).collect();
    println!("{:?}", adders.iter().map(|f| f(10)).collect::<Vec<_>>());
    let mut sorted = words.to_vec();
    let key = |w: &&str| std::cmp::Reverse(w.len());
    sorted.sort_by_key(key);
    println!("{:?}", sorted);
    let is_long = |w: &str| w.len() > 4;
    println!("{:?}", words.iter().map(|w| is_long(w)).collect::<Vec<_>>());
    println!("{}", words.iter().filter(|w| is_long(w)).count());
}
