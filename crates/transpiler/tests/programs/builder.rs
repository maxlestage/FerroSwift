use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Default, PartialEq)]
struct Request {
    url: String,
    method: String,
    retries: u32,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl Request {
    fn new(url: &str) -> Self {
        Request { url: url.to_string(), method: "GET".into(), ..Default::default() }
    }
    fn method(mut self, m: &str) -> Self {
        self.method = m.to_string();
        self
    }
    fn header(mut self, k: &str, v: &str) -> Self {
        self.headers.push((k.into(), v.into()));
        self
    }
    fn body(mut self, b: impl Into<String>) -> Self {
        self.body = Some(b.into());
        self
    }
    fn retries(self, n: u32) -> Self {
        Request { retries: n, ..self }
    }
    fn content_type(&self) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-type")).map(|(_, v)| v.as_str())
    }
    fn body_len(&self) -> usize {
        self.body.as_deref().map_or(0, str::len)
    }
    fn take_body(&mut self) -> Option<String> {
        self.body.take()
    }
    fn reset(&mut self) -> Vec<(String, String)> {
        self.retries = 0;
        std::mem::take(&mut self.headers)
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.method, self.url)?;
        if self.retries > 0 {
            write!(f, " (x{})", self.retries)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Light {
    Red,
    Green,
    Yellow,
}

impl Light {
    fn next(self) -> Light {
        match self {
            Light::Red => Light::Green,
            Light::Green => Light::Yellow,
            Light::Yellow => Light::Red,
        }
    }
    fn after(self, n: usize) -> Light {
        (0..n).fold(self, |l, _| l.next())
    }
}

struct Table {
    width: usize,
    rows: Vec<(String, f64)>,
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (name, value) in &self.rows {
            writeln!(f, "{name:<w$}|{value:>8.2}|", w = self.width)?;
        }
        write!(f, "{:=<1$}", "", self.width + 10)
    }
}

fn main() {
    let req = Request::new("http://x/a").method("POST").header("Content-Type", "text/plain").header("X-N", "1").body("hello").retries(2);
    println!("{req} | {:?} | {} | {:?}", req.content_type(), req.body_len(), req.headers.len());
    let mut r2 = req.clone();
    let taken = r2.take_body();
    let old_headers = r2.reset();
    println!("{:?} {:?} {} {:?} {}", taken, r2.body, r2, old_headers.len(), r2 == req);
    println!("{:?}", Request::new("u"));
    println!("{:?} {:?} {:?}", Light::Red.next(), Light::Red.after(4), Light::Yellow.after(0));
    let items = [(1, 'a'), (3, 'b'), (3, 'c'), (2, 'd'), (1, 'e')];
    println!("{:?} {:?}", items.iter().max_by_key(|p| p.0), items.iter().min_by_key(|p| p.0));
    println!("{:?} {:?}", items.iter().max_by(|a, b| a.0.cmp(&b.0)), items.iter().min_by(|a, b| a.0.cmp(&b.0)));
    println!("{:?} {:?}", items.iter().map(|p| p.0).max(), items.iter().rev().max_by_key(|p| p.0));
    let keys: Vec<i32> = items.iter().map(|p| p.0).collect();
    println!("{:?} {:?} {:?}", keys.iter().max(), keys.iter().position(|&k| k == 3), keys.iter().rposition(|&k| k == 3));
    let floats = [2.5, 1.5, 2.5, 0.5];
    println!("{:?} {:?}", floats.iter().cloned().fold(f64::NAN, f64::max), floats.iter().max_by(|a, b| a.partial_cmp(b).unwrap()));
    let mut stable: Vec<(i32, char)> = items.to_vec();
    stable.sort_by_key(|p| p.0);
    println!("{:?}", stable);
    stable.sort_by(|a, b| b.0.cmp(&a.0));
    println!("{:?}", stable);
    stable.dedup_by_key(|p| p.0);
    println!("{:?}", stable);
    println!("{:.2} {:.2} {:.0} {:.0} {:.3} {:.1} {:.1}", 0.125, 0.375, 2.5, 3.5, 0.0625, 0.25, 0.75);
    println!("{:.2} {:.2} {:.2} {:.1} {:.4}", 1.005, 1.015, 1.025, 0.05, 1.00005);
    println!("[{:>+5}] [{:<05}] [{:^05}] [{:+05}] [{:05}]", 7, 7, 7, -7, 7.5);
    println!("{:e} {:E} {:e} {:e} {:?} {:?}", 1500, 1500u64, 0, -123456789i64, 1.0e-7f32, 1e300);
    println!("[{:5.1}] [{:.3}] [{:>8.3}] [{:<6.2}] [{:^7.1}] [{:10}] [{:>3}]", "hello", "hello", "hello", "hello", "hello", true, 'x');
    println!("{:?} {}", '\u{1F600}', '\u{1F600}');
    println!("{:?} {:?} {:?}", 1.0f64 / 3.0, 2.0f32 / 3.0, 1e-7);
    let mut single = HashMap::new();
    single.insert("k", 1);
    println!("{:?}", single);
    println!("{:#?}", single);
    let t = Table { width: 6, rows: vec![("ab".into(), 1.0), ("cdefgh".into(), -2.345), ("x".into(), 1000.0)] };
    println!("{t}");
    println!("[{:>1$}] [{:<2$}] [{:^3$}]", "a", 3, 4, 5);
    println!("{}", format!("{:>5}", Light::Red.after(1) == Light::Green));
    let names = ["b", "a", "c"];
    let mut sorted = names.to_vec();
    sorted.sort_unstable();
    println!("{:?} {:?} {:?}", sorted, names.iter().max(), names.iter().min_by_key(|n| n.len()));
}
