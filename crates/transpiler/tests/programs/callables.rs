use std::collections::HashMap;

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn mul(a: i32, b: i32) -> i32 {
    a * b
}

fn apply_twice(f: &mut impl FnMut(i32), x: i32) {
    f(x);
    f(x * 2);
}

fn repeat_call<F: FnMut() -> u32>(mut f: F, n: usize) -> Vec<u32> {
    (0..n).map(|_| f()).collect()
}

fn total(it: impl Iterator<Item = u32>) -> u32 {
    it.sum()
}

fn longest(words: impl IntoIterator<Item = String>) -> Option<String> {
    words.into_iter().max_by_key(|w| w.len())
}

fn shout(text: impl AsRef<str>) -> String {
    text.as_ref().to_uppercase() + "!"
}

fn label(name: impl Into<String>, n: u8) -> String {
    let mut s: String = name.into();
    s.push('#');
    s.push_str(&n.to_string());
    s
}

fn run_op(op: fn(i32, i32) -> i32, a: i32, b: i32) -> i32 {
    op(a, b)
}

struct Stats {
    values: Vec<f64>,
}

impl Stats {
    fn scale(x: &f64) -> f64 {
        x * 10.0
    }

    fn scaled(&self) -> Vec<f64> {
        self.values.iter().map(Self::scale).collect()
    }

    fn describe(&self) -> Vec<String> {
        self.values.iter().map(ToString::to_string).collect()
    }
}

fn main() {
    let ops: [(&str, fn(i32, i32) -> i32); 3] = [("add", add), ("mul", mul), ("sub", |a, b| a - b)];
    for (name, op) in ops {
        println!("{name}: {} {}", op(6, 3), run_op(op, 2, 2));
    }
    let table: HashMap<&str, fn(i32, i32) -> i32> = ops.iter().cloned().collect();
    let mut names: Vec<_> = table.keys().copied().collect();
    names.sort_unstable();
    println!("{:?} {}", names, table["mul"](7, 6));
    let mut seen = Vec::new();
    let mut collect = |x: i32| seen.push(x);
    apply_twice(&mut collect, 5);
    apply_twice(&mut collect, 1);
    println!("{:?}", seen);
    let mut state = 1;
    let doubled = repeat_call(|| { state *= 2; state }, 5);
    println!("{:?} {}", doubled, state);
    println!("{} {}", total([1u32, 2, 3].into_iter()), total((1..=4).map(|x| x * x)));
    println!("{:?} {:?}", longest(vec!["aa".to_string(), "bbb".into(), "cc".into()]), longest(Vec::new()));
    println!("{} {} {}", shout("hey"), shout(String::from("yo")), shout(&String::from("ref")));
    println!("{} {}", label("item", 3), label(String::from("x"), 0));
    let raw = ["  a ", "b  ", " c"];
    let trimmed: Vec<&str> = raw.iter().map(|s| s.trim()).collect();
    let owned: Vec<String> = raw.iter().copied().map(str::trim).map(String::from).collect();
    println!("{:?} {:?}", trimmed, owned);
    let parsed: Vec<i32> = ["1", "x", "3"].iter().map(|s| s.parse::<i32>()).filter_map(Result::ok).collect();
    let somes: Vec<i32> = vec![Some(1), Some(2)].into_iter().map(Option::unwrap).collect();
    let lens: Vec<usize> = owned.iter().map(String::len).collect();
    println!("{:?} {:?} {:?}", parsed, somes, lens);
    let st = Stats { values: vec![0.5, 1.25] };
    println!("{:?} {:?}", st.scaled(), st.describe());
    let squares = [1, 2, 3].map(|x| x * x);
    let strs = squares.map(|x| x.to_string());
    println!("{:?} {:?} {}", squares, strs, squares.len());
    let pair = (String::from("left"), vec![1, 2]);
    let (ref l, ref v) = pair;
    println!("{l} {v:?} {}", pair.0.len());
    let mut mp = (3, 4);
    {
        let (ref mut a, ref mut b) = mp;
        std::mem::swap(a, b);
        *a += 10;
    }
    println!("{:?}", mp);
    let mut counter = 0;
    let mut bump = || {
        counter += 1;
        counter * 100
    };
    let first = bump();
    let second = bump();
    println!("{first} {second} {counter}");
    let compose = |f: fn(i32) -> i32, g: fn(i32) -> i32| move |x| g(f(x));
    let inc_then_double = compose(|x| x + 1, |x| x * 2);
    println!("{}", inc_then_double(4));
    let words = vec!["one", "two", "three"];
    let caps: Vec<String> = words.iter().map(|w| w.to_uppercase()).collect();
    let joined = caps.iter().map(String::as_str).collect::<Vec<_>>().join("-");
    println!("{joined}");
    if joined.len() > 5 {
        std::process::exit(3);
    }
    println!("unreachable");
}
