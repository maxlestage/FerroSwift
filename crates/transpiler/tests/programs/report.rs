use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Category {
    Food,
    Tools,
    Toys,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Category::Food => "food",
            Category::Tools => "tools",
            Category::Toys => "toys",
        };
        f.pad(name)
    }
}

#[derive(Debug, Clone)]
struct Item {
    name: String,
    category: Category,
    price_cents: u32,
    qty: u32,
}

#[derive(Debug)]
enum ReportError {
    Empty,
    BadLine { line: usize, reason: String },
    Price(std::num::ParseIntError),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReportError::Empty => write!(f, "no items"),
            ReportError::BadLine { line, reason } => write!(f, "line {line}: {reason}"),
            ReportError::Price(e) => write!(f, "bad price: {e}"),
        }
    }
}

impl From<std::num::ParseIntError> for ReportError {
    fn from(e: std::num::ParseIntError) -> Self {
        ReportError::Price(e)
    }
}

struct Money(u32);

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = format!("{}.{:02}", self.0 / 100, self.0 % 100);
        f.pad(&text)
    }
}

fn parse_line(line_no: usize, line: &str) -> Result<Item, ReportError> {
    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return Err(ReportError::BadLine { line: line_no, reason: format!("expected 4 fields, got {}", parts.len()) });
    }
    let category = match parts[1] {
        "food" => Category::Food,
        "tools" => Category::Tools,
        "toys" => Category::Toys,
        other => return Err(ReportError::BadLine { line: line_no, reason: format!("unknown category {other:?}") }),
    };
    let price_cents: u32 = parts[2].parse()?;
    let qty = parts[3].parse::<u32>().map_err(ReportError::Price)?;
    Ok(Item { name: parts[0].to_string(), category, price_cents, qty })
}

fn parse_all(text: &str) -> Result<Vec<Item>, ReportError> {
    let items: Vec<Item> = text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|(i, l)| parse_line(i + 1, l))
        .collect::<Result<_, _>>()?;
    if items.is_empty() {
        return Err(ReportError::Empty);
    }
    Ok(items)
}

fn report(items: &[Item]) -> String {
    let mut out = String::new();
    let total: u32 = items.iter().map(|i| i.price_cents * i.qty).sum();
    let mut by_cat: BTreeMap<String, Vec<&Item>> = BTreeMap::new();
    for item in items {
        by_cat.entry(item.category.to_string()).or_default().push(item);
    }
    out += &format!("{:<10}|{:>6}|{:>10}|{:>7}\n", "name", "qty", "price", "share");
    out += &format!("{}\n", "-".repeat(36));
    for (cat, group) in &by_cat {
        let subtotal: u32 = group.iter().map(|i| i.price_cents * i.qty).sum();
        out += &format!("[{:^34}]\n", cat.to_uppercase());
        let mut sorted = group.clone();
        sorted.sort_by(|a, b| (b.price_cents * b.qty).cmp(&(a.price_cents * a.qty)).then_with(|| a.name.cmp(&b.name)));
        for item in sorted {
            let share = (item.price_cents * item.qty) as f64 / total as f64 * 100.0;
            out += &format!("{:<10}|{:>6}|{:>10}|{:>6.1}%\n", item.name, item.qty, Money(item.price_cents), share);
        }
        out += &format!("{:>10} {:>25}\n", "subtotal", Money(subtotal));
    }
    out += &format!("{:=>36}\n{:>10} {:>25}\n", "", "total", Money(total));
    out
}

fn main() {
    let text = "# inventory\napple, food, 120, 10\nhammer, tools, 1599, 2\n\nyoyo, toys, 350, 7\nbread, food, 275, 3\n";
    match parse_all(text) {
        Ok(items) => print!("{}", report(&items)),
        Err(e) => println!("error: {e}"),
    }
    for bad in ["x, food, 12", "x, drinks, 1, 1", "x, food, 1x, 1", "x, food, 1, -1", "# only\n\n"] {
        match parse_all(bad) {
            Ok(items) => println!("parsed {}", items.len()),
            Err(e) => println!("error: {e} / {e:?}"),
        }
    }
    let cats = [Category::Toys, Category::Food];
    println!("[{:>7}] [{:<7}] [{:^7}] [{}]", cats[0], cats[1], cats[0], cats[1]);
    println!("{:?} {}", cats, cats[0] == Category::Toys);
}
