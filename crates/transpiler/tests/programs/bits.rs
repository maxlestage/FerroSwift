use std::fmt;

const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
static GREETING: &str = "hi";
const LIMITS: &[(u32, &str)] = &[(10, "low"), (100, "mid"), (1000, "high")];

fn caesar(s: &str, shift: u8) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' => ((c as u8 - b'a' + shift) % 26 + b'a') as char,
            'A'..='Z' => ((c as u8 - b'A' + shift) % 26 + b'A') as char,
            _ => c,
        })
        .collect()
}

fn classify(n: u32) -> &'static str {
    LIMITS.iter().find(|(limit, _)| n < *limit).map(|(_, name)| *name).unwrap_or("huge")
}

struct Pipeline {
    step: Box<dyn Fn(i64) -> i64>,
    check: fn(i64) -> bool,
    log: Vec<String>,
    cache: Option<Vec<i64>>,
}

impl Pipeline {
    fn run(&mut self, x: i64) -> Option<i64> {
        let y = (self.step)(x);
        if !(self.check)(y) {
            self.log.push(format!("rejected {y}"));
            return None;
        }
        self.cache.as_mut().map(|c| c.push(y));
        if let Some(c) = self.cache.as_mut() {
            c.sort_unstable();
        }
        Some(y)
    }
}

struct Pair<T> {
    a: T,
    b: T,
}

impl<T: PartialOrd + Copy> Pair<T> {
    fn largest(&self) -> T {
        if self.a > self.b { self.a } else { self.b }
    }
    fn swap(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}

impl<T: fmt::Display> fmt::Display for Pair<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}>", self.a, self.b)
    }
}

fn key_value(line: &str) -> Option<(&str, u64)> {
    let rest = line.strip_prefix("set ")?;
    let (k, v) = rest.split_once('=')?;
    let v = v.trim().parse::<u64>().ok()?;
    Some((k.trim(), v))
}

fn middle(s: &str) -> &str {
    let start = s.find('[').map(|i| i + 1).unwrap_or(0);
    let end = s.rfind(']').unwrap_or(s.len());
    if start <= end { &s[start..end] } else { "" }
}

fn main() {
    println!("{} {} {}", caesar("Hello, World!", 3), caesar(&caesar("xyz", 3), 23), caesar("", 1));
    println!("{} {} {} {}", VOWELS.contains(&'e'), VOWELS.iter().position(|&c| c == 'o').unwrap(), GREETING.len(), LIMITS.len());
    println!("{} {} {} {}", classify(5), classify(10), classify(999), classify(1_000_000));
    let letters: String = (b'a'..=b'e').map(|b| b as char).rev().collect();
    let digits: Vec<u8> = (b'0'..b'5').collect();
    println!("{letters} {:?} {}", digits, (b'A'..=b'Z').count());
    let n: u32 = 0x12345678;
    let be = n.to_be_bytes();
    let le = n.to_le_bytes();
    println!("{:?} {:?} {} {} {:#x}", be, le, u32::from_be_bytes(be), u32::from_le_bytes(be), n.swap_bytes());
    println!("{:032b} {} {} {:#x}", 0b1011u32, 0b1011u32.reverse_bits().leading_zeros(), (-1i16).to_be_bytes().len(), u16::from_le_bytes([0x34, 0x12]));
    let mut p = Pipeline { step: Box::new(|x| x * 3 - 1), check: |y| y % 2 == 0, log: vec![], cache: Some(vec![]) };
    println!("{:?} {:?} {:?} {:?}", p.run(1), p.run(2), p.run(3), p.run(-1));
    println!("{:?} {:?}", p.log, p.cache);
    p.cache = None;
    println!("{:?} {:?}", p.run(5), p.cache);
    let mut pair = Pair { a: 3.5, b: 2.25 };
    pair.swap();
    let words = Pair { a: "pear", b: "apple" };
    println!("{} {} {} {}", pair, pair.largest(), words.largest(), words);
    let total = {
        let base = 10;
        let mut acc = 0;
        for i in 1..=3 {
            acc += base * i;
        }
        acc
    };
    let label = if total > 50 {
        let t = total - 50;
        format!("over by {t}")
    } else {
        String::from("under")
    };
    let kind = match total % 4 {
        0 => {
            let half = total / 2;
            format!("quad {half}")
        }
        r => format!("rem {r}"),
    };
    println!("{total} {label} {kind}");
    for line in ["set a = 5", "set b=x", "get c=1", "set  d = 42 "] {
        println!("{:?}", key_value(line));
    }
    println!("{:?} {:?} {:?} {:?}", middle("x[abc]y"), middle("no brackets"), middle("]["), middle("[]"));
    let (head, tail) = "hello world".split_at(5);
    println!("{head:?} {tail:?} {:?} {:?}", "héllo".split_at(3), "héllo".char_indices().map(|(i, _)| i).collect::<Vec<_>>());
    println!("{:.3} {:.10} {} {} {}", 0.1f32, 0.1f32, 1.0f32 / 3.0, 16777216.0f32 + 1.0, (2.0f32).sqrt());
    println!("{} {} {:?} {}", 0.1f32 as f64, (0.1f32 as f64) == 0.1, 1e-45f32, f32::EPSILON);
}
