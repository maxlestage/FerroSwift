use std::num::{IntErrorKind, ParseFloatError, ParseIntError};
use std::str::FromStr;

fn generic<T: FromStr>(s: &str) -> String
where
    T::Err: std::fmt::Debug + std::fmt::Display,
{
    match s.parse::<T>() {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("{:?} / {}", e, e),
    }
}

fn first_word(t: (String,)) -> (usize,) {
    (t.0.len(),)
}

fn wrap(x: i32) -> (i32,) {
    (x,)
}

fn parse_two(a: &str, b: &str) -> Result<i32, ParseIntError> {
    Ok(a.parse::<i32>()? + b.parse::<i32>()?)
}

fn main() {
    for s in ["", "x", "99999999999", "-99999999999", "+", "-", "12a", " 1"] {
        println!("{:?} {:?}", s.parse::<i32>(), s.parse::<u8>());
    }
    for s in ["", "abc", "1.5.2", "inf"] {
        println!("{:?}", s.parse::<f64>());
    }
    println!("{:?} {:?} {:?}", "yes".parse::<bool>(), "".parse::<char>(), "ab".parse::<char>());
    let e = "300".parse::<u8>().unwrap_err();
    println!("{:?} {} {:?}", e, e, e.kind());
    match "".parse::<i64>() {
        Err(e) if *e.kind() == IntErrorKind::Empty => println!("empty: {e:?}"),
        other => println!("{other:?}"),
    }
    let kinds: Vec<IntErrorKind> = ["", "z", "999", "-999"].iter().filter_map(|s| s.parse::<i8>().err()).map(|e| e.kind().clone()).collect();
    println!("{:?}", kinds);
    let fe: ParseFloatError = "nope".parse::<f32>().unwrap_err();
    println!("{fe:?} {fe}");
    println!("{} | {} | {}", generic::<i32>("q"), generic::<f64>("q"), generic::<bool>("q"));
    println!("{} | {}", generic::<char>("qq"), generic::<u8>("-1"));
    println!("{:?} {:?}", parse_two("1", "2"), parse_two("1", "x"));
    let r: Result<Vec<u32>, _> = "1,2,x".split(',').map(str::parse::<u32>).collect();
    println!("{:?}", r);
    let boxed: Result<i32, Box<dyn std::error::Error>> = "q".parse::<i32>().map_err(|e| e.into());
    println!("{:?}", boxed.map_err(|e| e.to_string()));
    println!("{:?}", "7".parse::<i32>().ok());

    let one = (5,);
    let named: (String,) = ("hi".to_string(),);
    println!("{:?} {:?} {} {:?}", one, named, one.0 + 1, first_word(named.clone()));
    println!("{:#?}", (1.5,));
    let v: Vec<(i32,)> = (1..4).map(wrap).collect();
    println!("{:?} {}", v, v.iter().map(|t| t.0).sum::<i32>());
    let (x,) = wrap(9);
    println!("{x}");
    let nested = ((1, 2), (3,), ((4,),));
    println!("{:?} {}", nested, (nested.2).0.0);
    let mut m = (vec![1],);
    m.0.push(2);
    println!("{:?} {}", m, (7,) == (7,));
    let opt: Option<(char,)> = Some(('z',));
    if let Some((c,)) = opt {
        println!("{c}");
    }
    let mut ts = vec![(3,), (1,), (2,)];
    ts.sort();
    println!("{:?} {:?}", ts, ts.iter().max());
    match wrap(2) {
        (0,) => println!("zero"),
        (n,) if n % 2 == 0 => println!("even {n}"),
        _ => println!("odd"),
    }
}
