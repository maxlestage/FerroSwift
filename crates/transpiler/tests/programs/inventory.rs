use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Add { name: String, qty: u32, price: f64 },
    Remove(String, u32),
    Report,
}

#[derive(Debug)]
enum InvError {
    UnknownCommand(String),
    BadNumber(String),
    NotEnough { name: String, have: u32, want: u32 },
}

impl fmt::Display for InvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvError::UnknownCommand(c) => write!(f, "unknown command '{c}'"),
            InvError::BadNumber(s) => write!(f, "'{s}' is not a number"),
            InvError::NotEnough { name, have, want } => {
                write!(f, "only {have} {name} left, cannot remove {want}")
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    qty: u32,
    price: f64,
}

#[derive(Default)]
struct Inventory {
    items: HashMap<String, Item>,
    log: Vec<String>,
}

fn number<T: std::str::FromStr>(s: &str) -> Result<T, InvError> {
    s.parse::<T>().map_err(|_| InvError::BadNumber(s.to_string()))
}

fn parse(line: &str) -> Result<Command, InvError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.as_slice() {
        ["add", name, qty, price] => Ok(Command::Add {
            name: name.to_string(),
            qty: number(qty)?,
            price: number(price)?,
        }),
        ["remove", name, qty] => Ok(Command::Remove(name.to_string(), number(qty)?)),
        ["report"] => Ok(Command::Report),
        _ => Err(InvError::UnknownCommand(line.to_string())),
    }
}

impl Inventory {
    fn run(&mut self, cmd: Command) -> Result<(), InvError> {
        match cmd {
            Command::Add { name, qty, price } => {
                let item = self.items.entry(name.clone()).or_insert(Item { qty: 0, price });
                item.qty += qty;
                item.price = price;
                self.log.push(format!("+{qty} {name}"));
            }
            Command::Remove(name, qty) => {
                let have = self.items.get(&name).map(|i| i.qty).unwrap_or(0);
                if have < qty {
                    return Err(InvError::NotEnough { name, have, want: qty });
                }
                if let Some(item) = self.items.get_mut(&name) {
                    item.qty -= qty;
                }
                self.log.push(format!("-{qty} {name}"));
            }
            Command::Report => {
                let mut names: Vec<&String> = self.items.keys().collect();
                names.sort();
                for name in names {
                    let item = &self.items[name];
                    println!("{:<8}{:>4} x {:>6.2} = {:>8.2}", name, item.qty, item.price, item.qty as f64 * item.price);
                }
                let total: f64 = self.items.values().map(|i| i.qty as f64 * i.price).sum();
                println!("{:<8}{:>26.2}", "total", total);
            }
        }
        Ok(())
    }
}

fn main() {
    let script = "add apple 10 0.5\nadd pear 4 0.75\nremove apple 3\nremove pear 9\nfly away\nadd fig x 1\nadd apple 2 0.55\nreport";
    let mut inv = Inventory::default();
    for line in script.lines() {
        match parse(line).and_then(|cmd| inv.run(cmd)) {
            Ok(()) => {}
            Err(e) => println!("error: {e}"),
        }
    }
    println!("log: {}", inv.log.join(", "));
}
