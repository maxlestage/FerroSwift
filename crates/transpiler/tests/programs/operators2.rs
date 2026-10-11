use std::fmt;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
struct V2 {
    x: f64,
    y: f64,
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2 { x: self.x + o.x, y: self.y + o.y }
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2 { x: self.x - o.x, y: self.y - o.y }
    }
}

impl Mul<f64> for V2 {
    type Output = V2;
    fn mul(self, k: f64) -> V2 {
        V2 { x: self.x * k, y: self.y * k }
    }
}

impl Neg for V2 {
    type Output = V2;
    fn neg(self) -> V2 {
        V2 { x: -self.x, y: -self.y }
    }
}

impl AddAssign for V2 {
    fn add_assign(&mut self, o: V2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

struct Grid {
    w: usize,
    cells: Vec<u8>,
}

impl Index<(usize, usize)> for Grid {
    type Output = u8;
    fn index(&self, (r, c): (usize, usize)) -> &u8 {
        &self.cells[r * self.w + c]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut u8 {
        &mut self.cells[r * self.w + c]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    Num(i64),
    Op(char),
    LParen,
    RParen,
}

fn tokenize(s: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        match c {
            '0'..='9' => {
                let mut n = 0i64;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    n = n * 10 + cs[i].to_digit(10).unwrap() as i64;
                    i += 1;
                }
                out.push(Token::Num(n));
                continue;
            }
            '+' | '-' | '*' | '/' => out.push(Token::Op(c)),
            '(' => out.push(Token::LParen),
            ')' => out.push(Token::RParen),
            ' ' => {}
            other => panic!("bad char {other}"),
        }
        i += 1;
    }
    out
}

fn prec(op: char) -> u8 {
    match op {
        '+' | '-' => 1,
        '*' | '/' => 2,
        _ => 0,
    }
}

fn eval(tokens: &[Token]) -> i64 {
    let mut nums: Vec<i64> = Vec::new();
    let mut ops: Vec<Token> = Vec::new();
    fn apply(nums: &mut Vec<i64>, op: char) {
        let b = nums.pop().unwrap();
        let a = nums.pop().unwrap();
        nums.push(match op {
            '+' => a + b,
            '-' => a - b,
            '*' => a * b,
            '/' => a / b,
            _ => unreachable!(),
        });
    }
    for t in tokens {
        match t {
            Token::Num(n) => nums.push(*n),
            Token::LParen => ops.push(t.clone()),
            Token::RParen => {
                while let Some(Token::Op(op)) = ops.last().cloned() {
                    apply(&mut nums, op);
                    ops.pop();
                }
                ops.pop();
            }
            Token::Op(op) => {
                while let Some(Token::Op(top)) = ops.last() {
                    if prec(*top) >= prec(*op) {
                        let top = *top;
                        apply(&mut nums, top);
                        ops.pop();
                    } else {
                        break;
                    }
                }
                ops.push(t.clone());
            }
        }
    }
    while let Some(Token::Op(op)) = ops.pop() {
        apply(&mut nums, op);
    }
    nums[0]
}

fn classify(xs: &[i32]) -> String {
    match xs {
        [] => "empty".to_string(),
        [x] => format!("one {x}"),
        [first, .., last] if first == last => format!("bookends {first}"),
        [a, b, rest @ ..] => format!("{a},{b} then {} more", rest.len()),
    }
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn make_counter() -> impl FnMut() -> u32 {
    let mut c = 0;
    move || {
        c += 1;
        c
    }
}

fn compose(f: Box<dyn Fn(i32) -> i32>, g: fn(i32) -> i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| g(f(x)))
}

fn double(x: i32) -> i32 {
    x * 2
}

fn main() {
    let mut p = V2 { x: 1.0, y: 2.0 };
    let q = V2 { x: 0.5, y: -1.5 };
    println!("{} {} {} {} {:?}", p + q, p - q, p * 2.0, -p, p.partial_cmp(&q));
    p += q;
    println!("{p} {} {:?}", p == V2 { x: 1.5, y: 0.5 }, V2::default());
    println!("{:>12}|{:<12}|{:^12}|", format!("{p}"), format!("{q}"), format!("{}", -q));
    let mut g = Grid { w: 3, cells: vec![0; 6] };
    g[(1, 2)] = 7;
    g[(0, 0)] += 2;
    println!("{} {} {:?}", g[(1, 2)], g[(0, 0)], g.cells);
    for e in ["1 + 2 * 3", "(1 + 2) * 3", "10 / 3 - 4 * (2 - 5)", "2 * (3 + 4) * 5"] {
        let toks = tokenize(e);
        println!("{e} = {} ({} tokens, max {:?})", eval(&toks), toks.len(), toks.iter().max());
    }
    let mut toks = tokenize("3 + (4 * 2)");
    toks.sort();
    println!("{:?}", toks);
    println!("{} | {} | {} | {}", classify(&[]), classify(&[4]), classify(&[1, 2, 1]), classify(&[1, 2, 3, 4]));
    let add5 = make_adder(5);
    let mut counter = make_counter();
    let both = compose(Box::new(add5), double);
    println!("{} {} {} {}", both(1), counter(), counter(), counter());
    let fs: Vec<Box<dyn Fn(i32) -> i32>> = vec![Box::new(|x| x + 1), Box::new(|x| x * x), Box::new(make_adder(-3))];
    println!("{:?}", fs.iter().map(|f| f(7)).collect::<Vec<_>>());
    let ptrs: [fn(i32) -> i32; 2] = [double, |x| x - 1];
    println!("{}", ptrs.iter().fold(10, |acc, f| f(acc)));
    let mut a = vec![1, 2];
    let mut b = vec![3];
    std::mem::swap(&mut a, &mut b);
    let old = std::mem::replace(&mut a, vec![9, 9]);
    let taken = std::mem::take(&mut b);
    println!("{:?} {:?} {:?} {:?}", a, b, old, taken);
    println!("{:08.3} {:+} {:+.2} {:e} {:E} {:#x} {:#010b} {:#o}", 3.14159, 42, -0.5, 1234.5, 0.00012, 255, 5, 64);
    println!("{:-^20} {:*<6} {:>+8.2} {:08}", "mid", "ab", 2.5, -42);
    let w = 9;
    let pr = 2;
    println!("[{:>w$.pr$}] [{:<w$}] [{:^+w$}] [{:>6$}] [{:.*}]", 3.14159, 7, 1, 3, 2, 2.71828, 6, w = w, pr = pr);
    println!("{}", (0..5).map(|i| (b'a' + i as u8) as char).collect::<String>());
    println!("{} {} {:?} {:?} {}", 'A' as u32, 97u8 as char, '7'.to_digit(10), std::char::from_digit(11, 16), ('a'..='e').count());
    println!("{} {} {:?} {:?}", char::from_u32(0x1F600).is_some(), ('z' as u8 - b'a') as i32, 'ß'.to_uppercase().collect::<String>(), 'A'.to_lowercase().next());
    println!("{:?} {:?} {:?} {}", i32::MAX.checked_add(1), 250u8.overflowing_add(10), 5u8.overflowing_sub(6), 1u32.leading_zeros());
    println!("{} {} {} {} {}", 2i64.pow(40), 7u32.abs_diff(10), 17u32.div_ceil(5), 37u32.next_power_of_two(), 64u32.is_power_of_two());
    println!("{} {} {} {}", (-7i32).rem_euclid(3), (-7i32).div_euclid(3), (-7i32) % 3, -7i32 / 3);
    println!("{:?} {} {}", u32::from_str_radix("ff", 16), i64::from_str_radix("-101", 2).unwrap(), 255u8.count_ones());
    println!("{} {} {}", f64::EPSILON > 0.0, (0.1 + 0.2 - 0.3f64).abs() < f64::EPSILON, (2.0f64).powi(10));
    println!("{} {} {} {}", 10f64.log10(), 8f64.log2(), std::f64::consts::E.ln(), 2f64.powf(0.5));
    println!("{} {} {} {} {}", 2.5f64.round(), (-2.5f64).round(), 2.5f64.floor(), (-2.5f64).ceil(), 2.7f64.trunc());
    println!("{} {} {} {}", 7.0f64.sqrt(), 1e10f64, 1.5e-7, f64::MAX);
    println!("{} {} {} {}", 1.0f64 / 3.0, 2.0f32 / 3.0, 100.0f64, -0.0f64);
    println!("{} {} {}", f64::NAN.max(1.0), 1.0f64.min(f64::NAN), f64::INFINITY);
    println!("{:?} {:?} {:?}", 1.0f64, 0.1f32 + 0.2f32, 1e21f64);
    println!("{} {} {}", 5i32.clamp(1, 3), 2.5f64.clamp(0.0, 1.0), (-3i32).signum());
    println!("{} {} {} {}", i16::MAX, u16::MAX, i64::MIN, u64::MAX);
    println!("{} {} {}", 300i32 as u8, -1i32 as u32, 3.99f64 as i32);
    println!("{} {} {}", 1e20f64 as i32, -1e20f64 as i64, f64::NAN as u8);
    println!("{} {} {}", 255u8 as i8, 65u8 as char, true as i32);
    println!("{} {}", (1u64 << 40) + 1, 1i32 << 3 >> 1);
    println!("{} {} {}", 0b1010 & 0b0110, 0b1010 | 0b0110, 0b1010 ^ 0b0110);
    println!("{} {}", !0u8, !5i32);
}
