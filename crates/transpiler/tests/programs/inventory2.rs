#[derive(Debug, Clone, PartialEq)]
struct Item {
    name: String,
    qty: u32,
    tags: Vec<String>,
}

#[derive(Debug, Default)]
struct Counters {
    hits: Vec<u32>,
    last: Option<u32>,
}

#[derive(Debug, Default)]
struct Inventory {
    items: Vec<Item>,
    counters: Counters,
    log: Vec<String>,
}

impl Inventory {
    fn add(&mut self, name: &str, qty: u32) -> &mut Item {
        self.items.push(Item { name: name.to_string(), qty, tags: Vec::new() });
        self.items.last_mut().unwrap()
    }

    fn find_mut(&mut self, name: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.name == name)
    }

    fn at(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }

    fn first(&mut self) -> Option<&mut Item> {
        self.items.first_mut()
    }

    fn slot(&mut self, i: usize) -> &mut u32 {
        while self.counters.hits.len() <= i {
            self.counters.hits.push(0);
        }
        &mut self.counters.hits[i]
    }

    fn last_hit(&mut self) -> Option<&mut u32> {
        self.counters.last.as_mut()
    }

    fn counters_mut(&mut self) -> &mut Counters {
        &mut self.counters
    }

    fn bump(&mut self, name: &str) -> Option<u32> {
        let item = self.find_mut(name)?;
        item.qty += 1;
        let q = item.qty;
        self.log.push(format!("{name}->{q}"));
        Some(q)
    }

    fn tag(&mut self, name: &str, tag: &str) -> bool {
        if let Some(item) = self.find_mut(name) {
            item.tags.push(tag.to_string());
            true
        } else {
            false
        }
    }

    fn rename_first(&mut self, new_name: &str) {
        match self.first() {
            Some(item) => item.name = new_name.to_string(),
            None => self.log.push("empty".into()),
        }
    }
}

fn main() {
    let mut inv = Inventory::default();
    inv.add("bolt", 3).tags.push("metal".into());
    inv.add("nut", 5);
    let gear = inv.add("gear", 1);
    gear.qty *= 10;
    gear.tags.extend(["big".to_string(), "steel".to_string()]);
    println!("{:?}", inv.items.iter().map(|i| (i.name.as_str(), i.qty, i.tags.len())).collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", inv.bump("nut"), inv.bump("zip"), inv.bump("nut"));
    println!("{} {} {:?}", inv.tag("bolt", "small"), inv.tag("none", "x"), inv.find_mut("bolt").map(|i| i.tags.clone()));
    if let Some(item) = inv.at(1) {
        item.qty += 100;
    }
    if let Some(item) = inv.at(10) {
        item.qty += 100;
    }
    inv.at(0).unwrap().name.push_str("-x");
    println!("{:?}", inv.items.iter().map(|i| format!("{}={}", i.name, i.qty)).collect::<Vec<_>>());
    *inv.slot(2) += 7;
    *inv.slot(0) = 1;
    let s = inv.slot(4);
    *s += 2;
    *s *= 3;
    let last_hit = inv.last_hit().copied();
    println!("{:?} {:?}", inv.counters.hits, last_hit);
    inv.counters.last = Some(9);
    if let Some(l) = inv.last_hit() {
        *l += 1;
    }
    inv.counters_mut().hits.clear();
    inv.counters_mut().last = inv.counters.last.map(|x| x * 2);
    println!("{:?} {:?}", inv.counters, inv.log);
    inv.rename_first("BOLT");
    let mut empty = Inventory::default();
    empty.rename_first("x");
    let no_first = empty.first().is_none();
    println!("{:?} {:?} {:?}", inv.items[0].name, empty.log, no_first);
    let r = &mut inv;
    r.slot(1);
    *r.slot(1) += 40;
    let qty2 = r.at(2).map_or(0, |i| i.qty);
    println!("{:?} {}", r.counters.hits, qty2);
}
