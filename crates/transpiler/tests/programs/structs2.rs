#[derive(Debug, Clone, PartialEq, Default)]
struct Address {
    street: String,
    city: String,
    zip: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Person {
    name: String,
    age: u32,
    address: Address,
    tags: Vec<String>,
    scores: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Id(u32);

#[derive(Debug, Clone, PartialEq)]
struct Pair<A, B>(A, B);

#[derive(Debug, Default, Clone)]
struct Config {
    verbose: bool,
    retries: u8,
    name: String,
    ratio: f32,
    limits: (u32, u32),
}

impl Config {
    fn builder() -> Config {
        Config::default()
    }
    fn verbose(mut self, v: bool) -> Config {
        self.verbose = v;
        self
    }
    fn retries(mut self, n: u8) -> Config {
        self.retries = n;
        self
    }
    fn name(mut self, n: &str) -> Config {
        self.name = n.to_string();
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Status {
    Active { since: u32 },
    Suspended(String),
    Deleted,
}

#[derive(Debug, Clone)]
struct Account {
    id: Id,
    owner: Person,
    status: Status,
    balance: i64,
}

impl Account {
    fn new(id: u32, owner: Person) -> Self {
        Account { id: Id(id), owner, status: Status::Active { since: 2024 }, balance: 0 }
    }
    fn deposit(&mut self, amount: i64) -> &mut Self {
        self.balance += amount;
        self
    }
    fn suspend(&mut self, reason: &str) {
        self.status = Status::Suspended(reason.to_string());
    }
    fn is_active(&self) -> bool {
        matches!(self.status, Status::Active { .. })
    }
    fn summary(&self) -> String {
        format!("#{} {} ({}): {}", self.id.0, self.owner.name, self.owner.age, self.balance)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3 {
    const ORIGIN: Point3 = Point3 { x: 0.0, y: 0.0, z: 0.0 };
    fn dist(&self, o: &Point3) -> f64 {
        ((self.x - o.x).powi(2) + (self.y - o.y).powi(2) + (self.z - o.z).powi(2)).sqrt()
    }
    fn scaled(&self, k: f64) -> Point3 {
        Point3 { x: self.x * k, ..*self }
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone + std::fmt::Debug> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, item: T) {
        self.items.push(item);
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

fn main() {
    let home = Address { street: "1 Main St".into(), city: "Paris".into(), zip: Some("75001".into()) };
    let mut ann = Person {
        name: "Ann".to_string(),
        age: 30,
        address: home.clone(),
        tags: vec!["admin".into(), "dev".into()],
        scores: vec![("math".into(), 9.5), ("art".into(), 7.0)],
    };
    println!("{:?}", ann);
    println!("{:#?}", ann.address);
    let bob = Person { name: "Bob".into(), age: 25, ..ann.clone() };
    println!("{} {} {}", bob.name, bob.address.city, bob == ann);
    ann.age += 1;
    ann.tags.push("lead".into());
    ann.address.zip = None;
    println!("{} {:?} {:?}", ann.age, ann.tags, ann.address.zip);
    let copy = ann.clone();
    ann.name.push_str("ie");
    println!("{} {}", ann.name, copy.name);
    println!("{:?}", Person::default());
    println!("{:?} {:?}", Address::default(), Id::default());
    let ids = vec![Id(3), Id(1), Id(2)];
    let mut sorted = ids.clone();
    sorted.sort();
    println!("{:?} {:?} {}", sorted, ids.iter().max(), Id(1) < Id(2));
    let p = Pair("x", 1.5);
    let q = Pair(vec![1], Some('c'));
    println!("{:?} {:?} {} {}", p, q, p.0, q.1.unwrap());
    let cfg = Config::builder().verbose(true).retries(3).name("svc");
    println!("{:?}", cfg);
    println!("{} {} {} {} {:?}", cfg.verbose, cfg.retries, cfg.name, cfg.ratio, cfg.limits);
    let mut acct = Account::new(7, bob.clone());
    acct.deposit(100).deposit(-30);
    println!("{} {}", acct.summary(), acct.is_active());
    acct.suspend("fraud");
    println!("{:?} {}", acct.status, acct.is_active());
    let acct2 = Account { status: Status::Deleted, ..acct.clone() };
    println!("{:?} {:?}", acct2.status, acct.status);
    println!("{:#?}", acct2.status);
    let pts = [Point3 { x: 1.0, y: 2.0, z: 2.0 }, Point3::ORIGIN];
    println!("{:.3} {:?}", pts[0].dist(&pts[1]), pts[0].scaled(2.0));
    let mut st: Stack<String> = Stack::new();
    st.push("a".into());
    st.push("b".into());
    println!("{:?} {} {}", st.peek(), st.len(), st.is_empty());
    println!("{:?} {:?} {:?}", st.pop(), st.pop(), st.pop());
    let mut nums = Stack::new();
    nums.push(1);
    nums.push(2);
    println!("{:?} {}", nums.peek().map(|n| n * 10), nums.len());
    let people = vec![ann.clone(), bob.clone()];
    let names: Vec<&str> = people.iter().map(|p| p.name.as_str()).collect();
    let oldest = people.iter().max_by_key(|p| p.age).unwrap();
    println!("{:?} {}", names, oldest.name);
    let total_score: f64 = people.iter().flat_map(|p| p.scores.iter().map(|(_, s)| s)).sum();
    println!("{total_score}");
    let mut by_city: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for p in &people {
        by_city.entry(p.address.city.clone()).or_default().push(p.name.clone());
    }
    println!("{:?}", by_city);
    let tuple_of_structs = (Id(1), Point3::ORIGIN, Status::Deleted);
    println!("{:?}", tuple_of_structs);
    let opt: Option<Address> = Some(home.clone());
    println!("{:?}", opt);
    println!("{:?}", opt.as_ref().map(|a| a.city.len()));
    let nested = vec![Some(Id(1)), None];
    println!("{:?}", nested);
    println!("{:#?}", Pair(Id(5), vec![Status::Deleted]));
}
