use std::collections::BTreeMap;
use std::fmt::{self, Display};

trait Named {
    fn name(&self) -> String;
}

trait Priced: Named {
    fn cents(&self) -> u64;
    fn price(&self) -> String {
        format!("{}: ${}.{:02}", self.name(), self.cents() / 100, self.cents() % 100)
    }
    fn discounted(&self, pct: u64) -> u64 {
        self.cents() * (100 - pct) / 100
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Item { label: String, cents: u64, weight: f64 }

impl Named for Item {
    fn name(&self) -> String { self.label.clone() }
}
impl Priced for Item {
    fn cents(&self) -> u64 { self.cents }
}

struct Bundle { items: Vec<Item>, off: u64 }
impl Named for Bundle {
    fn name(&self) -> String { format!("bundle of {}", self.items.len()) }
}
impl Priced for Bundle {
    fn cents(&self) -> u64 { self.items.iter().map(|i| i.cents).sum::<u64>() - self.off }
    fn price(&self) -> String { format!("[{}] {}", self.items.iter().map(|i| i.label.as_str()).collect::<Vec<_>>().join("+"), self.cents()) }
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:.1}kg)", self.label, self.weight)
    }
}

fn cheapest<'a>(xs: &'a [Box<dyn Priced>]) -> Option<&'a Box<dyn Priced>> {
    xs.iter().min_by_key(|p| p.cents())
}

fn print_all<T: Priced + ?Sized>(xs: &[&T]) {
    for x in xs { println!("  {}", x.price()); }
}

fn main() {
    let a = Item { label: "tea".into(), cents: 450, weight: 0.2 };
    let b = Item { label: "pot".into(), cents: 2599, weight: 1.25 };
    let bundle = Bundle { items: vec![a.clone(), b.clone()], off: 49 };
    let shelf: Vec<Box<dyn Priced>> = vec![Box::new(a.clone()), Box::new(b.clone()), Box::new(bundle)];
    for p in &shelf { println!("{} -> {}", p.price(), p.discounted(10)); }
    if let Some(c) = cheapest(&shelf) { println!("cheapest {}", c.name()); }
    print_all(&[&a, &b]);
    let mut by_name: BTreeMap<String, Box<dyn Priced>> = BTreeMap::new();
    for p in shelf { by_name.insert(p.name(), p); }
    println!("{:?}", by_name.keys().collect::<Vec<_>>());
    println!("{} | {} | {:?}", a, b, a.partial_cmp(&b));
    let mut items = vec![b.clone(), a.clone()];
    items.sort_by(|x, y| x.partial_cmp(y).unwrap());
    println!("{}", items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "));
}
