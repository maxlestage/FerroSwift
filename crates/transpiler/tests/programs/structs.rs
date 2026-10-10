#[derive(Debug, Clone, Default, PartialEq)]
struct Config {
    name: String,
    retries: u32,
    verbose: bool,
    ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug)]
struct Meters(f64);

impl Meters {
    fn to_feet(&self) -> f64 {
        self.0 * 3.28084
    }
}

#[derive(Debug, Default)]
struct Builder {
    width: u32,
    height: u32,
    title: String,
}

impl Builder {
    fn width(mut self, w: u32) -> Self {
        self.width = w;
        self
    }

    fn height(mut self, h: u32) -> Self {
        self.height = h;
        self
    }

    fn title(mut self, t: &str) -> Self {
        self.title = t.to_string();
        self
    }

    fn area(&self) -> u32 {
        self.width * self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Level {
    Low,
    Mid,
    High,
}

impl Level {
    fn next(self) -> Level {
        match self {
            Level::Low => Level::Mid,
            Level::Mid | Level::High => Level::High,
        }
    }
}

struct Counter {
    value: i64,
    history: Vec<i64>,
}

impl Counter {
    const STEP: i64 = 5;

    fn new(start: i64) -> Counter {
        Counter { value: start, history: vec![start] }
    }

    fn bump(&mut self) -> i64 {
        self.value += Self::STEP;
        self.history.push(self.value);
        self.value
    }
}

fn main() {
    let base = Config::default();
    println!("{:?}", base);
    let custom = Config { name: "prod".to_string(), retries: 3, ..Default::default() };
    println!("{:?} same as base: {}", custom, custom == base);
    let copy = Config { verbose: true, ..custom.clone() };
    println!("{} {} {}", copy.name, copy.retries, copy.verbose);

    let mut versions = vec![
        Version { major: 1, minor: 2, patch: 3 },
        Version { major: 0, minor: 9, patch: 0 },
        Version { major: 1, minor: 0, patch: 7 },
    ];
    versions.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for v in &versions {
        println!("v{}.{}.{}", v.major, v.minor, v.patch);
    }
    println!("newer: {}", versions[2] > versions[0]);

    let m = Meters(10.0);
    println!("{:?} = {:.1} ft", m, m.to_feet());

    let b = Builder::default().width(4).height(5).title("box");
    println!("{} area {} {:?}", b.title, b.area(), b);

    let mut lvl = Level::Low;
    for _ in 0..3 {
        lvl = lvl.next();
    }
    println!("{:?} {}", lvl, Level::Low < Level::High);
    let mut levels = vec![Level::High, Level::Low, Level::Mid];
    levels.sort();
    println!("{:?}", levels);

    let mut c = Counter::new(10);
    c.bump();
    c.bump();
    println!("{} {:?}", c.value, c.history);
}
