use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Error {
    Empty,
    TooLong(usize),
    Bad { func: String, class: i32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Empty => write!(f, "empty"),
            Error::TooLong(n) => write!(f, "too long ({n})"),
            Error::Bad { func, class } => write!(f, "bad {func} in class {class}"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

fn check(s: &str) -> Result<usize> {
    if s.is_empty() {
        return Err(Error::Empty);
    }
    if s.len() > 5 {
        return Err(Error::TooLong(s.len()));
    }
    if s.starts_with('!') {
        return Err(Error::Bad { func: s.to_string(), class: 7 });
    }
    Ok(s.len())
}

fn total(items: &[&str]) -> Result<usize> {
    let mut sum = 0;
    for it in items {
        sum += check(it)?;
    }
    Ok(sum)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Mode {
    Fast,
    #[default]
    Normal,
    Slow,
}

#[derive(Debug, Default)]
struct Config {
    mode: Mode,
    func: u32,
    protocol: String,
    extension: Option<String>,
    inout: bool,
}

trait Named {
    fn name(&self) -> String;
}

trait Greeter: Named {
    fn greeting(&self) -> String {
        format!("hello, {}", self.name())
    }
}

struct Person(String);

impl Named for Person {
    fn name(&self) -> String {
        self.0.clone()
    }
}

impl Greeter for Person {}

struct Robot {
    id: u32,
    loud: bool,
}

impl Named for Robot {
    fn name(&self) -> String {
        format!("unit-{}", self.id)
    }
}

impl Greeter for Robot {
    fn greeting(&self) -> String {
        if self.loud { self.name().to_uppercase() } else { self.name() }
    }
}

struct Filter<T> {
    rules: Vec<(String, Box<dyn Fn(&T) -> bool>)>,
    fallback: Option<Box<dyn Greeter>>,
}

impl<T> Filter<T> {
    fn apply<'a>(&self, items: &'a [T]) -> Vec<&'a T> {
        items.iter().filter(|it| self.rules.iter().all(|(_, r)| r(it))).collect()
    }
    fn failing(&self, item: &T) -> Vec<&str> {
        self.rules.iter().filter(|(_, r)| !r(item)).map(|(n, _)| n.as_str()).collect()
    }
}

fn main() {
    println!("{:?} {:?} {:?} {:?}", check("abc"), check(""), check("abcdefg"), check("!x"));
    println!("{:?} {:?}", total(&["ab", "cd"]), total(&["ab", "", "x"]));
    for e in [Error::Empty, Error::TooLong(9), Error::Bad { func: "f".into(), class: 1 }] {
        println!("{e} | {e:?} | {}", e == Error::Empty);
    }
    let r: Result<u8> = Err(Error::Empty);
    println!("{} {:?}", r.is_err(), r.as_ref().err().map(|e| e.to_string()));
    let func = 3;
    let protocol = "tcp";
    let extension = vec![1, 2];
    let class = func * 2;
    let defer = class + 1;
    let repeat = defer % 2 == 0;
    let var = 5;
    let nil = 0;
    let inout = !repeat;
    let init = 9;
    let guard = init - var;
    let subscript = "s";
    println!("{func} {protocol} {:?} {class} {defer} {repeat} {var} {nil} {inout} {init} {guard} {subscript}", extension);
    let cfg = Config { func: 1, protocol: "udp".into(), ..Default::default() };
    println!("{:?} {:?} {}", cfg, Mode::default(), cfg.mode == Mode::Normal);
    println!("{:?}", Config { mode: Mode::Slow, extension: Some("x".into()), inout: true, ..Default::default() });
    let greeters: Vec<Box<dyn Greeter>> = vec![Box::new(Person("ann".into())), Box::new(Robot { id: 7, loud: true }), Box::new(Robot { id: 8, loud: false })];
    for g in &greeters {
        println!("{} / {}", g.greeting(), g.name());
    }
    let f = Filter {
        rules: vec![("positive".into(), Box::new(|x: &i32| *x > 0)), ("even".into(), Box::new(|x| x % 2 == 0))],
        fallback: Some(Box::new(Person("bob".into()))),
    };
    let data = [4, -2, 3, 8, 0];
    println!("{:?} {:?} {:?} {:?}", f.apply(&data), f.failing(&-3), f.failing(&4), f.fallback.as_ref().map(|g| g.greeting()));
    let none: Filter<i32> = Filter { rules: vec![], fallback: None };
    println!("{:?} {}", none.apply(&data), none.fallback.is_none());
}
