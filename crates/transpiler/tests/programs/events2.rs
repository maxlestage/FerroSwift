use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Deposited { amount: u64 },
    Withdrew { amount: u64 },
    Renamed(String),
    Closed,
}

impl Event {
    fn amount(&self) -> u64 {
        match self {
            Event::Deposited { amount } | Event::Withdrew { amount } => *amount,
            _ => 0,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Event::Deposited { .. } => "deposit",
            Event::Withdrew { .. } => "withdrawal",
            Event::Renamed(_) => "rename",
            Event::Closed => "closed",
        }
    }
}

impl TryFrom<&str> for Event {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut parts = s.splitn(2, ' ');
        let verb = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();
        match verb {
            "dep" => rest.parse().map(|amount| Event::Deposited { amount }).map_err(|e| format!("dep: {e}")),
            "wd" => rest.parse().map(|amount| Event::Withdrew { amount }).map_err(|e| format!("wd: {e}")),
            "name" if !rest.is_empty() => Ok(Event::Renamed(rest.to_string())),
            "close" => Ok(Event::Closed),
            other => Err(format!("unknown verb '{other}'")),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::Deposited { amount } => write!(f, "+{amount}"),
            Event::Withdrew { amount } => write!(f, "-{amount}"),
            Event::Renamed(n) => write!(f, "~{n}"),
            Event::Closed => f.write_str("x"),
        }
    }
}

#[derive(Debug, Default)]
struct Account {
    name: String,
    balance: u64,
    open: bool,
    history: Vec<Event>,
}

impl Account {
    fn new() -> Account {
        Account { open: true, ..Default::default() }
    }

    fn apply(&mut self, e: &Event) -> Result<(), String> {
        if !self.open {
            return Err("account closed".into());
        }
        match e {
            Event::Deposited { amount } => self.balance += amount,
            Event::Withdrew { amount } => {
                self.balance = self.balance.checked_sub(*amount).ok_or(format!("insufficient: {} < {}", self.balance, amount))?;
            }
            Event::Renamed(n) => self.name = n.clone(),
            Event::Closed => self.open = false,
        }
        self.history.push(e.clone());
        Ok(())
    }

    fn apply_all<I: IntoIterator<Item = Event>>(&mut self, events: I) -> usize {
        let mut applied = 0;
        for e in events {
            if self.apply(&e).is_ok() {
                applied += 1;
            }
        }
        applied
    }
}

fn main() {
    let lines = ["dep 100", "wd 30", "name main", "wd 500", "dep x", "jump 3", "name ", "close", "dep 5"];
    let events: Vec<Result<Event, String>> = lines.iter().map(|l| Event::try_from(*l)).collect();
    for (line, e) in lines.iter().zip(&events) {
        match e {
            Ok(ev) => println!("{line:<10} -> {ev} ({})", ev.kind()),
            Err(msg) => println!("{line:<10} -> error: {msg}"),
        }
    }
    let mut accounts: HashMap<String, Account> = HashMap::new();
    let acct = accounts.entry("alice".to_string()).or_insert_with(Account::new);
    for e in events.iter().flatten() {
        match acct.apply(e) {
            Ok(()) => println!("applied {e}: balance {:>6}", acct.balance),
            Err(msg) => println!("rejected {e}: {msg}"),
        }
    }
    let bob = accounts.entry("bob".to_string()).or_insert_with(Account::new);
    let n = bob.apply_all(vec![Event::Deposited { amount: 7 }, Event::Withdrew { amount: 9 }, Event::Deposited { amount: 1 }]);
    println!("bob applied {n}, balance {}", bob.balance);
    let mut names: Vec<&String> = accounts.keys().collect();
    names.sort();
    for name in names {
        let a = &accounts[name];
        println!("{name}: {} {} {} {:?}", a.name, a.balance, a.open, a.history.iter().map(|e| e.kind()).collect::<Vec<_>>());
    }
    println!("{}", accounts.get("carol").map(|a| a.balance).unwrap_or_default());

    let history = &accounts["alice"].history;
    println!("{:?}", history.iter().rev().find_map(|e| match e { Event::Deposited { amount } => Some(*amount), _ => None }));
    println!("{}", history.iter().filter(|e| matches!(e, Event::Withdrew { amount } if *amount > 10)).count());
    let mut sorted = history.clone();
    sorted.sort_unstable_by_key(|e| Reverse(e.amount()));
    println!("{:?}", sorted.iter().map(|e| e.amount()).collect::<Vec<_>>());
    let mut kinds = history.clone();
    kinds.dedup_by_key(|e| e.kind());
    println!("{:?}", kinds.iter().map(|e| e.kind()).collect::<Vec<_>>());
    println!("{}", history.windows(2).filter(|w| w[0].kind() == w[1].kind()).count());
    let ties = [("a", 3), ("b", 5), ("c", 5), ("d", 1), ("e", 1)];
    println!("{:?} {:?}", ties.iter().max_by_key(|t| t.1), ties.iter().min_by_key(|t| t.1));
    println!("{:?} {:?}", ties.iter().max_by(|x, y| x.1.cmp(&y.1)), ties.iter().min_by(|x, y| x.1.cmp(&y.1)));
    println!("{:?} {:?}", ties.iter().map(|t| t.1).max(), ties.iter().rev().max_by_key(|t| t.1));
    println!("{:?}", history.iter().zip(1..).map(|(e, i)| format!("{i}:{e}")).collect::<Vec<_>>());
    println!("{:?}", lines.iter().enumerate().skip(1).step_by(3).map(|(i, _)| i).collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", history.last().map(|e| e.kind()), history.iter().nth(2), std::iter::repeat(Event::Closed).take(2).count());
    let mut trimmed = history.clone();
    trimmed.retain(|e| !matches!(e, Event::Closed | Event::Renamed(_)));
    println!("{}", trimmed.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" "));
}
