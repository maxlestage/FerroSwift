use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

type BoxError = Box<dyn Error + Send + Sync>;

struct Wrapper<T>(T);

impl<T: fmt::Display> fmt::Display for Wrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Reading {
    sensor: String,
    value: f64,
}

fn parse_reading(line: &str) -> Result<Reading, BoxError> {
    let (sensor, value) = line.split_once('=').ok_or("missing '='")?;
    if sensor.is_empty() {
        return Err("empty sensor name".into());
    }
    let value: f64 = value.trim().parse()?;
    if !value.is_finite() {
        return Err(format!("{sensor}: not a finite value").into());
    }
    Ok(Reading { sensor: sensor.to_string(), value })
}

fn pipeline() -> Vec<Box<dyn Fn(i64) -> i64>> {
    let mut steps: Vec<Box<dyn Fn(i64) -> i64>> = Vec::new();
    for k in 1..=3 {
        steps.push(Box::new(move |x| x * k));
    }
    steps.push(Box::new(|x| x - 1));
    steps
}

fn compare(a: &Reading, b: &Reading) -> Ordering {
    match a.sensor.cmp(&b.sensor) {
        Ordering::Equal => b.value.partial_cmp(&a.value).unwrap_or(Ordering::Equal),
        other => other,
    }
}

fn settle(x: Option<i32>) -> i32 {
    x.unwrap_or_else(|| {
        eprintln!("no value: giving up");
        std::process::exit(3)
    })
}

fn main() -> Result<(), String> {
    println!("{} {} {}", Wrapper(5), Wrapper("text"), Wrapper(Wrapper(1.5)));
    for line in ["temp=21.5", "hum = 40", "bad", "=3", "temp=abc", "x=inf"] {
        match parse_reading(line) {
            Ok(r) => println!("ok {:?}", r),
            Err(e) => println!("err {}", e),
        }
    }
    let steps = pipeline();
    println!("{} {}", steps.iter().fold(2, |acc, f| f(acc)), steps.len());
    let composed: i64 = steps.iter().rev().fold(10, |acc, f| f(acc));
    println!("{composed}");

    let mut readings = vec![
        Reading { sensor: "b".into(), value: 1.0 },
        Reading { sensor: "a".into(), value: 2.0 },
        Reading { sensor: "a".into(), value: 5.0 },
    ];
    readings.sort_by(compare);
    println!("{:?}", readings.iter().map(|r| format!("{}{}", r.sensor, r.value)).collect::<Vec<_>>());
    let o = 3.cmp(&5);
    println!("{:?} {:?} {} {} {:?}", o, o.reverse(), o.is_lt(), o.is_ge(), o.then(Ordering::Greater));
    println!("{:?} {:?}", Ordering::Equal.then_with(|| 2.cmp(&1)), [1, 2, 3][..].cmp(&[1, 2][..]));
    let sorted = [1, 3, 5, 7, 9, 11];
    println!("{:?} {:?} {}", sorted.binary_search_by(|p| p.cmp(&7)), sorted.binary_search_by(|p| p.cmp(&4)), sorted.partition_point(|&x| x < 6));
    println!("{:?}", sorted.binary_search_by_key(&10, |&x| x * 2));
    println!("{} {} {} {}", 64u32.is_power_of_two(), 65u32.is_power_of_two(), 65u32.next_power_of_two(), 0b0001_0110u8.reverse_bits());
    println!("{:?} {:?} {} {}", 1u8.checked_shl(7), 1u8.checked_shl(8), 1u8.wrapping_shl(9), (-8i32).signum());
    println!("{} {}", 0b0111u8.trailing_ones(), 0xF8u8.leading_ones());
    println!("{}", settle(Some(4)));
    let total: f32 = [0.1f32, 0.2, 0.3].iter().sum();
    println!("{} {} {}", total, total as f64, (total * 10.0).round() / 10.0);
    let code = settle(None);
    println!("unreachable {code}");
    Err("done".to_string())
}
