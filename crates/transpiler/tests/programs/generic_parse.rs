use std::fmt::Debug;
use std::str::FromStr;

fn read<T: FromStr>(s: &str) -> Option<T> {
    s.trim().parse().ok()
}

fn check<T>(s: &str) -> String
where
    T: FromStr + Debug,
    T::Err: Debug,
{
    format!("{:?}", s.parse::<T>())
}

fn all<T: FromStr>(items: &[&str], _sample: T) -> Result<Vec<T>, <T as FromStr>::Err> {
    items.iter().map(|s| s.parse::<T>()).collect()
}

struct Reader {
    strict: bool,
}

impl Reader {
    fn get<T: FromStr + Default>(&self, s: &str) -> T {
        match s.parse::<T>() {
            Ok(v) => v,
            Err(_) if !self.strict => T::default(),
            Err(_) => panic!("bad input {s}"),
        }
    }
}

fn main() {
    let a: Option<i32> = read("2147483648");
    let b: Option<i64> = read("2147483648");
    let c = read::<i32>("-2147483648");
    let d: Option<u8> = read("256");
    println!("{a:?} {b:?} {c:?} {d:?}");
    println!("{}", check::<i32>("99999999999"));
    println!("{}", check::<i64>("99999999999"));
    println!("{}", check::<i32>("-99999999999"));
    println!("{}", check::<f32>("1e3"));
    println!("{:?}", all(&["1", "4294967296"], 0i32).map_err(|e| e.to_string()));
    println!("{:?}", all(&["1", "4294967296"], 0i64));
    let r = Reader { strict: false };
    let x: i32 = r.get("3000000000");
    let y: i64 = r.get("3000000000");
    println!("{x} {y} {}", r.get::<i32>("12"));
}
