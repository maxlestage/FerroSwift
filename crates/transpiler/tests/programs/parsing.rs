use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
}

struct Lexer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer { src, pos: 0 }
    }

    fn tokens(&mut self) -> Result<Vec<Tok>, String> {
        let chars: Vec<char> = self.src.chars().collect();
        let mut out = Vec::new();
        while self.pos < chars.len() {
            let c = chars[self.pos];
            if c.is_whitespace() {
                self.pos += 1;
            } else if c.is_ascii_digit() || c == '.' {
                let start = self.pos;
                while self.pos < chars.len() && (chars[self.pos].is_ascii_digit() || chars[self.pos] == '.') {
                    self.pos += 1;
                }
                let text: String = chars[start..self.pos].iter().collect();
                let n: f64 = text.parse().map_err(|_| format!("bad number {text}"))?;
                out.push(Tok::Num(n));
            } else if c.is_alphabetic() || c == '_' {
                let start = self.pos;
                while self.pos < chars.len() && (chars[self.pos].is_alphanumeric() || chars[self.pos] == '_') {
                    self.pos += 1;
                }
                out.push(Tok::Ident(chars[start..self.pos].iter().collect()));
            } else {
                self.pos += 1;
                match c {
                    '(' => out.push(Tok::LParen),
                    ')' => out.push(Tok::RParen),
                    '+' | '-' | '*' | '/' | '^' => out.push(Tok::Op(c)),
                    other => return Err(format!("unexpected {other:?} at {}", self.pos - 1)),
                }
            }
        }
        Ok(out)
    }
}

struct Parser {
    toks: Vec<Tok>,
    i: usize,
    vars: HashMap<String, f64>,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.i)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.i).cloned();
        self.i += 1;
        t
    }

    fn expr(&mut self) -> Result<f64, String> {
        let mut left = self.term()?;
        while let Some(Tok::Op(op)) = self.peek().cloned() {
            if op != '+' && op != '-' {
                break;
            }
            self.next();
            let right = self.term()?;
            left = if op == '+' { left + right } else { left - right };
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut left = self.factor()?;
        loop {
            match self.peek() {
                Some(Tok::Op('*')) => {
                    self.next();
                    left *= self.factor()?;
                }
                Some(Tok::Op('/')) => {
                    self.next();
                    let d = self.factor()?;
                    if d == 0.0 {
                        return Err("division by zero".into());
                    }
                    left /= d;
                }
                _ => return Ok(left),
            }
        }
    }

    fn factor(&mut self) -> Result<f64, String> {
        let base = self.atom()?;
        if let Some(Tok::Op('^')) = self.peek() {
            self.next();
            let exp = self.factor()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn atom(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(n),
            Some(Tok::Ident(name)) => self.vars.get(&name).copied().ok_or(format!("unknown variable {name}")),
            Some(Tok::Op('-')) => Ok(-self.atom()?),
            Some(Tok::LParen) => {
                let v = self.expr()?;
                match self.next() {
                    Some(Tok::RParen) => Ok(v),
                    other => Err(format!("expected ), got {other:?}")),
                }
            }
            other => Err(format!("unexpected {other:?}")),
        }
    }
}

fn eval(src: &str, vars: &HashMap<String, f64>) -> Result<f64, String> {
    let toks = Lexer::new(src).tokens()?;
    let mut p = Parser { toks, i: 0, vars: vars.clone() };
    let v = p.expr()?;
    if p.i < p.toks.len() {
        return Err(format!("trailing input at token {}", p.i));
    }
    Ok(v)
}

fn word_freq(text: &str) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    pairs
}

fn rle(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        let mut n = 1;
        while it.peek() == Some(&c) {
            it.next();
            n += 1;
        }
        if n > 1 {
            out.push_str(&n.to_string());
        }
        out.push(c);
    }
    out
}

fn caesar(s: &str, shift: u8) -> String {
    s.bytes()
        .map(|b| match b {
            b'a'..=b'z' => (b'a' + (b - b'a' + shift) % 26) as char,
            b'A'..=b'Z' => (b'A' + (b - b'A' + shift) % 26) as char,
            _ => b as char,
        })
        .collect()
}

fn is_palindrome(s: &str) -> bool {
    let cleaned: Vec<char> = s.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect();
    cleaned.iter().eq(cleaned.iter().rev())
}

fn main() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 3.0);
    vars.insert("pi".to_string(), 3.14159);
    for src in ["1 + 2 * 3", "(1 + 2) * 3", "2 ^ 3 ^ 2", "-x * 2", "pi * x ^ 2", "4 / (2 - 2)", "y + 1", "3 $ 4", "(1 + 2", "1 2"] {
        match eval(src, &vars) {
            Ok(v) => println!("{src} = {v}"),
            Err(e) => println!("{src} -> error: {e}"),
        }
    }
    println!("{:?}", Lexer::new("a1 + 2.5").tokens());
    for (w, n) in word_freq("The cat and the hat. The END, the end!").iter().take(3) {
        println!("{w}: {n}");
    }
    println!("{} {} {}", rle("aaabccdddd"), rle(""), rle("x"));
    println!("{} {}", caesar("Hello, World!", 3), caesar(&caesar("abcxyz", 13), 13));
    println!("{} {}", is_palindrome("A man, a plan, a canal: Panama"), is_palindrome("rust"));
    let mut scores = vec![("ann", 7.5), ("bob", 9.0), ("cy", 7.5), ("dee", 3.25)];
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal).then_with(|| a.0.cmp(b.0)));
    println!("{:?}", scores);
    let best = scores.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|s| s.0);
    println!("{:?}", best);
    println!("{:?}", 1.5f64.partial_cmp(&2.5));
    println!("{:?} {:?}", 3.cmp(&3), "b".cmp("a"));
    let ord = 5.cmp(&7);
    println!("{}", match ord { Ordering::Less => "less", Ordering::Equal => "equal", Ordering::Greater => "greater" });
    println!("{:?} {:?}", ord.reverse(), ord.then(Ordering::Greater));
    println!("{} {}", ord == Ordering::Less, ord.is_lt());
    let bytes = "héllo".as_bytes();
    println!("{} {} {:?}", bytes.len(), bytes[0], &bytes[1..3]);
    let s = String::from_utf8_lossy(&bytes[0..1]);
    println!("{s}");
    let mut buf = Vec::new();
    buf.extend_from_slice(b"ab");
    buf.push(b'c');
    println!("{:?} {}", buf, String::from_utf8(buf.clone()).unwrap());
    let hex: String = buf.iter().map(|b| format!("{:02x}", b)).collect();
    println!("{hex}");
    let mut it = [10, 20, 30].iter();
    println!("{:?} {:?}", it.next(), it.next());
    let rest: Vec<_> = it.collect();
    println!("{:?}", rest);
    let mut pk = "a,b".chars().peekable();
    while let Some(c) = pk.next() {
        if let Some(&n) = pk.peek() {
            print!("{c}>{n} ");
        } else {
            println!("{c}.");
        }
    }
    let data = [3, 1, 4, 1, 5, 9, 2, 6];
    let mut sorted = data.to_vec();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    sorted.dedup();
    println!("{:?}", sorted);
    let (evens, odds): (Vec<i32>, Vec<i32>) = data.iter().partition(|&&x| x % 2 == 0);
    println!("{:?} {:?}", evens, odds);
    let pos = data.iter().rposition(|&x| x == 1);
    let maxi = data.iter().enumerate().max_by_key(|(_, v)| **v).map(|(i, _)| i);
    println!("{:?} {:?}", pos, maxi);
    let cycled: Vec<i32> = [1, 2].iter().cycle().take(5).copied().collect();
    println!("{:?}", cycled);
    let pairs: Vec<(i32, i32)> = data.chunks_exact(2).map(|c| (c[0], c[1])).collect();
    println!("{:?}", pairs);
    let total: i32 = data.iter().rev().enumerate().map(|(i, &x)| i as i32 * x).sum();
    println!("{total}");
    let mut v2 = vec![1, 2, 3, 4, 5, 6];
    v2.retain_mut(|x| {
        *x *= 10;
        *x != 30
    });
    println!("{:?}", v2);
    v2.dedup_by_key(|x| *x / 25);
    println!("{:?}", v2);
    let taken = std::mem::take(&mut v2);
    println!("{:?} {:?}", taken, v2);
    let mut a = 1;
    let mut b = 2;
    std::mem::swap(&mut a, &mut b);
    let old = std::mem::replace(&mut a, 9);
    println!("{a} {b} {old}");
}
