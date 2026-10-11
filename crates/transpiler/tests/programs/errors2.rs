use std::collections::HashMap;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    Parse(ParseIntError),
    Negative(i64),
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Parse(e) => write!(f, "parse error: {e}"),
            AppError::Negative(n) => write!(f, "{n} is negative"),
        }
    }
}

fn positive(s: &str) -> Result<i64, AppError> {
    let n: i64 = s.trim().parse()?;
    if n < 0 {
        return Err(AppError::Negative(n));
    }
    Ok(n)
}

fn total(parts: &[&str]) -> Result<i32, ParseIntError> {
    let mut sum = 0;
    for p in parts {
        sum += p.parse::<i32>()?;
    }
    Ok(sum)
}

fn boxed(s: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let v: f64 = s.parse()?;
    Ok(v * 2.0)
}

#[derive(Debug, Clone, PartialEq)]
struct Item {
    name: String,
    qty: u32,
}

fn main() {
    println!("{:?} {:?} {:?}", positive(" 42 "), positive("-3"), positive("x"));
    match positive("abc") {
        Ok(n) => println!("ok {n}"),
        Err(e) => println!("error: {e}"),
    }
    println!("{:?} {:?}", total(&["1", "2", "3"]), total(&["1", "", "3"]));
    println!("{:?} {:?}", boxed("1.5"), boxed("z").is_err());
    if let Err(e) = boxed("nope") {
        println!("boxed: {e}");
    }
    println!("{:?} {:?}", u8::try_from(300i32), u8::try_from(200i32));
    println!("{:?} {:?}", i32::try_from(5_000_000_000i64), i32::try_from(-7i64));
    let big: i64 = 70000;
    let narrow: Result<i16, _> = big.try_into();
    println!("{:?} {:?}", narrow, i16::try_from(-70000i64).is_err());
    println!("{:?} {:?}", char::try_from(65u32), char::try_from(0xD800u32));
    println!("{:?} {:?}", char::from_u32(0x1F600), char::from_u32(0x110000));
    println!("{:?} {:?}", i32::from_str_radix("-80000000", 16), i32::from_str_radix("7fffffff", 16));
    println!("{:?} {:?}", "1.5".parse::<f32>(), "nan".parse::<f64>().map(|x| x.is_nan()));
    println!("{:?} {:?}", "18446744073709551615".parse::<u64>(), "18446744073709551616".parse::<u64>());
    println!("{:?} {:?}", "-0".parse::<i32>(), "0x10".parse::<i32>());
    println!("{:?}", "hello".parse::<String>());
    let floats: Option<f64> = [Some(1.5), Some(2.5)].into_iter().sum();
    let none: Option<i32> = [Some(1), None].into_iter().sum();
    let prod: Result<i32, String> = vec![Ok(2), Ok(3)].into_iter().product();
    println!("{:?} {:?} {:?}", floats, none, prod);
    let mut items = vec![
        Item { name: "a".into(), qty: 1 },
        Item { name: "b".into(), qty: 2 },
    ];
    let last = items.last_mut().unwrap();
    last.qty += 10;
    last.name.push('!');
    items.iter_mut().find(|i| i.name == "a").unwrap().qty = 99;
    if let Some(found) = items.iter_mut().find(|i| i.qty == 99) {
        found.name = "found".into();
    }
    println!("{:?}", items);
    let mut stock: HashMap<&str, u32> = HashMap::new();
    stock.insert("apple", 3);
    *stock.get_mut("apple").unwrap() += 2;
    let apple = stock.get_mut("apple").unwrap();
    *apple *= 10;
    println!("{:?}", stock["apple"]);
    let mut maybe: Option<Vec<i32>> = Some(vec![1]);
    maybe.as_mut().unwrap().push(2);
    if let Some(v) = maybe.as_mut() {
        v.push(3);
    }
    println!("{:?}", maybe);
    let mut grid = vec![vec![0; 3]; 2];
    grid.iter_mut().for_each(|row| row[0] = 7);
    for row in grid.iter_mut() {
        row.iter_mut().for_each(|c| *c += 1);
    }
    println!("{:?}", grid);
    let mut words = vec!["b".to_string(), "a".to_string()];
    words.iter_mut().for_each(|w| w.push('x'));
    words.sort();
    println!("{:?}", words);
    let pair = (1,);
    let (x,) = pair;
    let single: Vec<(u8,)> = vec![(1,), (2,)];
    let firsts: Vec<u8> = single.iter().map(|(a,)| *a).collect();
    println!("{} {:?} {:?} {}", x, single, firsts, single[0] < single[1]);
    match single[0] {
        (1,) => println!("one"),
        (n,) => println!("other {n}"),
    }
    let e = "".parse::<i32>().unwrap_err();
    println!("{e} / {e:?}");
    let msg = "q".parse::<u8>().map_err(|e| format!("bad: {e}")).unwrap_err();
    println!("{msg}");
    let n = "12".parse::<i32>().unwrap_or_else(|_| 0) + "zz".parse::<i32>().unwrap_or_default();
    println!("{n}");
    let ok = "5".parse::<i32>().is_ok_and(|v| v > 3);
    println!("{ok}");
    let results: Vec<Result<i32, ParseIntError>> = ["1", "x"].iter().map(|s| s.parse()).collect();
    println!("{:?}", results);
    let (good, bad): (Vec<_>, Vec<_>) = ["1", "x", "3"].iter().map(|s| s.parse::<i32>()).partition(|r| r.is_ok());
    println!("{} {}", good.len(), bad.len());
    let nums: Vec<i32> = "1 2 x 4".split(' ').filter_map(|s| s.parse().ok()).collect();
    println!("{:?}", nums);
    let all: Result<Vec<i32>, _> = "1 2 3".split(' ').map(str::parse::<i32>).collect();
    println!("{:?}", all);
    let bad_all: Result<Vec<i32>, ParseIntError> = "1 b 3".split(' ').map(|s| s.parse::<i32>()).collect();
    println!("{:?}", bad_all);
    println!("{:?}", "3.0".parse::<f64>().unwrap() == 3.0);
    println!("{:?}", "-inf".parse::<f64>());
    println!("{}", "2".parse::<i32>().expect("number") + "3".parse::<i32>().unwrap());
}
