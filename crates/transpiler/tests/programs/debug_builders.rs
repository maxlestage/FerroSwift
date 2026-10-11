use std::collections::BTreeMap;
use std::fmt;

struct Secret(String);

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Secret").field("len", &self.0.len()).field("hint", &self.0.chars().next()).finish()
    }
}

struct Pair(i32, &'static str);

impl fmt::Debug for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Pair").field(&self.0).field(&self.1).finish()
    }
}

struct Bag {
    items: Vec<u8>,
    tags: BTreeMap<&'static str, bool>,
}

impl fmt::Debug for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bag")
            .field("items", &self.items)
            .field("tags", &self.tags)
            .finish()
    }
}

struct Items(Vec<i32>);

impl fmt::Debug for Items {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter().map(|x| x * 10)).finish()
    }
}

struct Table(Vec<(char, u32)>);

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter().map(|(k, v)| (k, v))).finish()
    }
}

struct Money(i64);

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = format!("${}.{:02}", self.0 / 100, self.0 % 100);
        if f.alternate() {
            write!(f, "[{text}]")
        } else {
            f.pad(&text)
        }
    }
}

struct Cell(f64);

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match f.precision() {
            Some(p) => write!(f, "{:.*}", p, self.0),
            None => write!(f, "{}", self.0),
        }
    }
}

fn main() {
    let s = Secret("hunter2".into());
    println!("{:?} | {:#?}", s, s);
    println!("{:?} {:#?}", Pair(7, "x"), Pair(-1, ""));
    let bag = Bag { items: vec![1, 2], tags: BTreeMap::from([("a", true), ("b", false)]) };
    println!("{:?}", bag);
    println!("{:#?}", bag);
    println!("{:?} {:#?}", Items(vec![1, 2]), Items(vec![]));
    println!("{:?}", Table(vec![('x', 1), ('y', 2)]));
    println!("{:#?}", Table(vec![('z', 0)]));
    let m = Money(123456);
    println!("[{m}] [{m:#}] [{m:>12}] [{m:<12}] [{m:^12}] [{:*>10}]", Money(5));
    println!("[{}] [{:.1}] [{:.3}] [{:8.2}]", Cell(3.14159), Cell(3.14159), Cell(2.0), Cell(1.5));
    let bytes = [0u8, 15, 255, 128];
    println!("{:x?} {:X?} {:02x?} {:#x?}", bytes, bytes, bytes, bytes);
    println!("{:x?} {:?}", vec![Some(255u8), None], (10u8, 11u8));
    println!("{:#x?}", (255u16, "s"));
    println!("[{:+.3e}] [{:10.3e}] [{:<10.1e}] [{:e}]", 1234.5678, 0.000123, 98765.0, 1.0f32);
    println!("[{:^9.3}] [{:>7.2}] [{:3.1}] [{:#>5}]", "abcdef", "héllo", "xyz", "é");
    println!("{:?} {:?} {:?}", format!("{:5}", 'c'), format!("{:<3}", true), format!("{:>4?}", ()));
    println!("{:05.1} {:+05} {:08.3} {:+08.2}", -1.25, 7, -2.5, 3.14159);
    println!("{:#?}", vec![(1, "a"), (2, "b")]);
    println!("{:#?}", Some(vec![Some(1), None]));
}
