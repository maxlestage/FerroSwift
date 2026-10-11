use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Write;

#[derive(Debug)]
enum InventoryError {
    Parse(String),
    Missing(String),
    Negative(i64),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InventoryError::Parse(s) => write!(f, "cannot parse '{}'", s),
            InventoryError::Missing(name) => write!(f, "no item named {}", name),
            InventoryError::Negative(n) => write!(f, "negative quantity {}", n),
        }
    }
}

impl Error for InventoryError {}

impl From<std::num::ParseIntError> for InventoryError {
    fn from(e: std::num::ParseIntError) -> Self {
        InventoryError::Parse(e.to_string())
    }
}

struct Item {
    name: String,
    qty: i64,
    tags: Vec<String>,
}

impl Clone for Item {
    fn clone(&self) -> Self {
        Item { name: self.name.clone(), qty: self.qty, tags: self.tags.clone() }
    }
}

struct Inventory {
    items: Vec<Item>,
    cache: Option<HashMap<String, usize>>,
    grid: HashMap<(i32, i32), char>,
}

impl Inventory {
    fn new() -> Self {
        Inventory { items: Vec::with_capacity(8), cache: None, grid: HashMap::new() }
    }

    fn index(&mut self) -> &HashMap<String, usize> {
        let items = &self.items;
        self.cache.get_or_insert_with(|| {
            items.iter().enumerate().map(|(i, it)| (it.name.clone(), i)).collect::<HashMap<_, _>>()
        })
    }

    fn add(&mut self, line: &str) -> Result<(), InventoryError> {
        let Some((name, rest)) = line.split_once(':') else {
            return Err(InventoryError::Parse(line.to_string()));
        };
        let mut parts = rest.split(',');
        let qty: i64 = parts.next().unwrap_or("").trim().parse()?;
        if qty < 0 {
            return Err(InventoryError::Negative(qty));
        }
        let tags: Vec<String> = parts.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
        self.items.push(Item { name: name.trim().to_string(), qty, tags });
        self.cache = None;
        Ok(())
    }

    fn quantity(&mut self, name: &str) -> Result<i64, InventoryError> {
        let idx = self.index().get(name).copied();
        let Some(i) = idx else {
            return Err(InventoryError::Missing(name.to_string()));
        };
        Ok(self.items[i].qty)
    }

    fn first_tagged(&self, tag: &str) -> Option<&Item> {
        self.items.iter().find(|it| it.tags.iter().any(|t| t == tag))
    }

    fn first_numeric_tag(&self) -> Option<i64> {
        self.items.iter().flat_map(|it| it.tags.iter()).find_map(|t| t.parse::<i64>().ok())
    }

    fn in_stock(&self, name: &str) -> Option<bool> {
        let item = self.items.iter().find(|it| it.name == name)?;
        (item.qty > 0).then_some(true)
    }
}

fn describe<T>(label: &str, values: &[T]) -> String
where
    T: fmt::Display + Clone,
{
    let mut out = String::new();
    write!(out, "{}:", label).unwrap();
    for v in values {
        let copy = v.clone();
        write!(out, " {}", copy).unwrap();
    }
    writeln!(out, " ({} values)", values.len()).unwrap();
    out
}

fn label(item: &Option<Item>) -> String {
    match *item {
        Some(ref it) if it.qty == 0 => format!("{} (out of stock)", it.name),
        Some(ref it) => format!("{} x{}", it.name, it.qty),
        None => String::from("nothing"),
    }
}

fn all_quantities(lines: &[&str]) -> Option<Vec<i64>> {
    lines
        .iter()
        .map(|l| {
            l.split_once(':')
                .and_then(|(_, r)| r.split(',').next())
                .and_then(|q| q.trim().parse::<i64>().ok())
        })
        .collect()
}

fn parse_all(lines: &[&str]) -> Result<Vec<i64>, InventoryError> {
    lines.iter().map(|l| l.trim().parse::<i64>().map_err(InventoryError::from)).collect()
}

fn planned() -> i32 {
    todo!()
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut inv = Inventory::new();
    let lines = ["bolt: 40, metal, small", "nut: 0, metal", "glue: 3, 12, liquid", "oops", "tape: x1"];
    for line in lines {
        match inv.add(line) {
            Ok(()) => println!("added {}", line),
            Err(e) => println!("skipped '{}': {}", line, e),
        }
    }
    println!("{} items, index has {} entries", inv.items.len(), inv.index().len());
    for name in ["bolt", "nut", "screw"] {
        match inv.quantity(name) {
            Ok(q) => println!("{name}: {q}"),
            Err(e) => println!("{name}: error: {e} / {e:?}"),
        }
    }
    match inv.first_tagged("metal") {
        Some(it) => println!("first metal: {} {:?}", it.name, it.tags),
        None => println!("no metal"),
    }
    println!("first numeric tag: {:?}", inv.first_numeric_tag());
    println!("in stock: {:?} {:?} {:?}", inv.in_stock("bolt"), inv.in_stock("nut"), inv.in_stock("gum"));

    print!("{}", describe("names", &inv.items.iter().map(|i| i.name.clone()).collect::<Vec<_>>()));
    print!("{}", describe("qty", &[1.5, 2.25]));
    let cloned = inv.items[1].clone();
    println!("{}", label(&Some(cloned)));
    println!("{}", label(&Some(inv.items[0].clone())));
    println!("{}", label(&None));

    println!("{:?}", all_quantities(&["a: 1", "b: 2, x", "c: 3"]));
    println!("{:?}", all_quantities(&["a: 1", "b", "c: 3"]));
    match parse_all(&["7", " 8 ", "9"]) {
        Ok(v) => println!("parsed {:?} sum {}", v, v.iter().sum::<i64>()),
        Err(e) => println!("error {}", e),
    }
    match parse_all(&["7", "eight", "9"]) {
        Ok(v) => println!("parsed {:?}", v),
        Err(e) => println!("error {}", e),
    }

    inv.grid.insert((0, 1), 'b');
    inv.grid.insert((2, -1), 'n');
    inv.grid.insert((0, 0), 'g');
    let mut keys: Vec<(i32, i32)> = inv.grid.keys().copied().collect();
    keys.sort();
    for k in &keys {
        println!("{:?} -> {}", k, inv.grid[k]);
    }
    if let Some(c) = inv.grid.get(&(2, -1)) {
        println!("at (2,-1): {c}");
    }
    println!("has (1,1): {}", inv.grid.contains_key(&(1, 1)));

    let data = [3, 1, 4, 1, 5, 9, 2, 6];
    let mut it = data.iter();
    let head: Vec<i32> = it.by_ref().take(3).copied().collect();
    let mut seen = Vec::new();
    let rest: i32 = it.inspect(|x| seen.push(**x)).filter(|x| **x % 2 == 0).sum();
    println!("head {:?} rest-even-sum {} seen {:?}", head, rest, seen);

    if inv.items.len() > 100 {
        println!("{}", planned());
    }
    inv.add("bad: -4")?;
    println!("not reached");
    Ok(())
}
