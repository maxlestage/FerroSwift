use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;

trait Entity {
    fn id(&self) -> u32;
    fn label(&self) -> String;
}

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u32,
    name: String,
    age: u8,
}

impl Entity for User {
    fn id(&self) -> u32 {
        self.id
    }
    fn label(&self) -> String {
        format!("{}#{}", self.name, self.id)
    }
}

struct Repository<T> {
    items: Vec<T>,
    hits: Cell<u32>,
    on_change: Option<Box<dyn Fn(&str)>>,
}

impl<T> Default for Repository<T> {
    fn default() -> Self {
        Repository { items: Vec::new(), hits: Cell::new(0), on_change: None }
    }
}

impl<T: Entity + Clone> Repository<T> {
    fn find(&self, id: u32) -> Option<&T> {
        self.hits.set(self.hits.get() + 1);
        self.items.iter().find(|it| it.id() == id)
    }

    fn upsert(&mut self, item: T) -> bool {
        let label = item.label();
        let replaced = if let Some(slot) = self.items.iter_mut().find(|it| it.id() == item.id()) {
            *slot = item;
            true
        } else {
            self.items.push(item);
            false
        };
        if let Some(cb) = &self.on_change {
            cb(&label);
        }
        replaced
    }

    fn remove(&mut self, id: u32) -> Option<T> {
        let pos = self.items.iter().position(|it| it.id() == id)?;
        Some(self.items.remove(pos))
    }

    fn labels(&self) -> Vec<String> {
        self.items.iter().map(Entity::label).collect()
    }
}

impl<T: Entity> FromIterator<T> for Repository<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Repository { items: iter.into_iter().collect(), hits: Cell::new(0), on_change: None }
    }
}

struct Validator<T> {
    rules: Vec<Box<dyn Fn(&T) -> Option<String>>>,
}

impl<T> Validator<T> {
    fn new() -> Self {
        Validator { rules: Vec::new() }
    }

    fn rule(mut self, f: impl Fn(&T) -> Option<String> + 'static) -> Self {
        self.rules.push(Box::new(f));
        self
    }

    fn check(&self, value: &T) -> Vec<String> {
        self.rules.iter().filter_map(|r| r(value)).collect()
    }
}

fn describe_all<T: Display + ?Sized>(items: &[&T]) -> String {
    items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
}

#[derive(Debug)]
enum State {
    Idle,
    Running { step: u32 },
    Done(String),
}

struct Machine {
    state: State,
    last_result: Option<String>,
    log: Rc<RefCell<Vec<String>>>,
}

impl Machine {
    fn advance(&mut self) {
        let old = std::mem::replace(&mut self.state, State::Idle);
        self.state = match old {
            State::Idle => State::Running { step: 1 },
            State::Running { step } if step < 3 => State::Running { step: step + 1 },
            State::Running { step } => State::Done(format!("finished after {step}")),
            State::Done(msg) => {
                self.last_result = Some(msg);
                State::Idle
            }
        };
        self.log.borrow_mut().push(format!("{:?}", self.state));
    }

    fn take_result(&mut self) -> Option<String> {
        self.last_result.take()
    }
}

fn main() {
    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&log);
    let mut repo: Repository<User> = Repository::default();
    repo.on_change = Some(Box::new(move |label| sink.borrow_mut().push(format!("changed {label}"))));
    println!("{}", repo.upsert(User { id: 1, name: "ann".into(), age: 30 }));
    println!("{}", repo.upsert(User { id: 2, name: "bo".into(), age: 17 }));
    println!("{}", repo.upsert(User { id: 1, name: "anne".into(), age: 31 }));
    println!("{:?} {:?} hits {}", repo.find(1).map(|u| &u.name), repo.find(9), repo.hits.get());
    println!("{:?} {:?}", repo.remove(2).map(|u| u.name), repo.remove(2));
    println!("{:?} {:?}", repo.labels(), log.borrow());
    let imported: Repository<User> = (1..=3).map(|i| User { id: i * 10, name: format!("u{i}"), age: 20 + i as u8 }).collect();
    println!("{:?} {}", imported.labels(), imported.items.len());

    let validator = Validator::new()
        .rule(|u: &User| (u.age < 18).then(|| format!("{} is a minor", u.name)))
        .rule(|u: &User| u.name.is_empty().then_some("empty name".to_string()))
        .rule(|u| if u.name.chars().all(char::is_lowercase) { None } else { Some(format!("{}: not lowercase", u.name)) });
    for u in [User { id: 3, name: "".into(), age: 10 }, User { id: 4, name: "Cy".into(), age: 40 }, User { id: 5, name: "di".into(), age: 20 }] {
        println!("{} -> {:?}", u.id, validator.check(&u));
    }
    println!("{}", describe_all::<str>(&["a", "b"]));
    println!("{}", describe_all(&[&1.5, &2.0]));

    let mut m = Machine { state: State::Idle, last_result: None, log: Rc::clone(&log) };
    for _ in 0..6 {
        m.advance();
    }
    println!("{:?} {:?} {:?}", m.take_result(), m.take_result(), m.state);
    println!("{} {:?}", log.borrow().len(), log.borrow().last());
    let mut a = vec![1, 2];
    let mut b = vec![3];
    std::mem::swap(&mut a, &mut b);
    let old = std::mem::replace(&mut a, vec![9, 9]);
    let taken = std::mem::take(&mut b);
    println!("{:?} {:?} {:?} {:?}", a, b, old, taken);
    let groups: BTreeMap<u8, Vec<&str>> = imported.items.iter().fold(BTreeMap::new(), |mut acc, u| {
        acc.entry(u.age / 10).or_insert_with(Vec::new).push(u.name.as_str());
        acc
    });
    println!("{:?}", groups);
    println!("{}", imported.items.iter().map(|u| u.name.as_str()).collect::<Vec<&str>>().join("/"));
}
