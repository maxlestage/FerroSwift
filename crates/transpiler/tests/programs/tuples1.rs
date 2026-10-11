use std::collections::HashMap;
use std::num::{IntErrorKind, ParseFloatError, ParseIntError};
use std::str::FromStr;

#[derive(Debug)]
enum AppError {
    Int(ParseIntError),
    Float(ParseFloatError),
    Other(String),
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Int(e)
    }
}

impl From<ParseFloatError> for AppError {
    fn from(e: ParseFloatError) -> Self {
        AppError::Float(e)
    }
}

#[derive(Debug)]
struct Failure {
    line: usize,
    cause: ParseIntError,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Wrapper {
    id: (u32,),
    tags: Vec<(String,)>,
}

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Single((i32,)),
    Pair((i32,), (i32,)),
}

fn load(a: &str, b: &str) -> Result<f64, AppError> {
    let n: i64 = a.trim().parse()?;
    let x: f64 = b.trim().parse()?;
    if n < 0 {
        return Err(AppError::Other(format!("negative {n}")));
    }
    Ok(n as f64 * x)
}

fn describe(e: &ParseIntError) -> &'static str {
    match e.kind() {
        IntErrorKind::Empty => "empty",
        IntErrorKind::InvalidDigit => "digit",
        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => "overflow",
        _ => "other",
    }
}

fn sum_single((x,): (i32,), (y,): (i32,)) -> (i32,) {
    (x + y,)
}

fn main() {
    for (a, b) in [("2", "1.5"), ("x", "1"), ("2", "y"), ("-3", "1"), ("", "")] {
        println!("{:?}", load(a, b));
    }
    let failures: Vec<Failure> = ["1", "a", "", "999999999999999999999"].iter().enumerate().filter_map(|(i, s)| s.parse::<i64>().err().map(|cause| Failure { line: i + 1, cause })).collect();
    println!("{:?}", failures);
    println!("{:#?}", failures.first());
    for f in &failures {
        println!("{} {} {}", f.line, describe(&f.cause), f.cause);
    }
    println!("{:?} {:?}", u32::from_str_radix("zz", 16), i8::from_str_radix("-81", 16));
    println!("{:?} {:?} {:?}", i32::from_str("12"), f64::from_str("x"), bool::from_str("true"));
    println!("{}", "".parse::<u16>().unwrap_err().kind() == &IntErrorKind::Empty);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for s in ["1", "x", "", "2", "y"] {
        let key = match s.parse::<u8>() {
            Ok(_) => "ok".to_string(),
            Err(e) => format!("{:?}", e.kind()),
        };
        *counts.entry(key).or_default() += 1;
    }
    let mut keys: Vec<_> = counts.into_iter().collect();
    keys.sort();
    println!("{:?}", keys);
    let results: Vec<Result<u8, String>> = ["7", "700"].iter().map(|s| s.parse::<u8>().map_err(|e| e.to_string())).collect();
    println!("{:?}", results);

    let mut w = Wrapper::default();
    println!("{:?}", w);
    w.id.0 = 7;
    w.tags.push(("a".into(),));
    w.tags.push(("b".into(),));
    println!("{:?} {} {}", w, w.id.0 * 2, w.tags.iter().map(|t| t.0.as_str()).collect::<Vec<_>>().join(","));
    for (tag,) in &w.tags {
        print!("{tag} ");
    }
    println!();
    let shapes = vec![Shape::Single((1,)), Shape::Pair((2,), (3,))];
    for s in &shapes {
        match s {
            Shape::Single((v,)) => println!("single {v}"),
            Shape::Pair((a,), (b,)) => println!("pair {} {}", a, b),
        }
    }
    println!("{:?} {:?}", shapes, sum_single((4,), (5,)));
    let t: (i32,) = (1,);
    let (mut a,) = t;
    a += 10;
    println!("{a} {:?} {:?}", t, ((1,),));
    let mut index: HashMap<(char,), u32> = HashMap::new();
    index.insert(('a',), 1);
    *index.entry(('a',)).or_insert(0) += 5;
    println!("{:?} {:?}", index.get(&('a',)), index.get(&('b',)));
    let squares: Vec<(i64,)> = (1..=3).map(|n| (n * n,)).collect();
    let total: i64 = squares.iter().map(|(n,)| n).sum();
    println!("{:?} {} {:>6}|", squares, total, format!("{:?}", (3,)));
    let r: Result<(u8,), String> = Ok((2,));
    if let Ok((v,)) = r {
        println!("{v}");
    }
}
