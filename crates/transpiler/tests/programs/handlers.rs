use std::collections::{BTreeMap, HashMap};

trait Handler {
    fn handle(&mut self, input: &str) -> String;
    fn calls(&self) -> usize;
}

struct Echo {
    count: usize,
}

struct Upper {
    count: usize,
    prefix: String,
}

impl Handler for Echo {
    fn handle(&mut self, input: &str) -> String {
        self.count += 1;
        input.to_string()
    }
    fn calls(&self) -> usize {
        self.count
    }
}

impl Handler for Upper {
    fn handle(&mut self, input: &str) -> String {
        self.count += 1;
        format!("{}{}", self.prefix, input.to_uppercase())
    }
    fn calls(&self) -> usize {
        self.count
    }
}

struct Numbers {
    v: Vec<i32>,
}

impl Numbers {
    fn evens(&self) -> Box<dyn Iterator<Item = &i32> + '_> {
        Box::new(self.v.iter().filter(|x| *x % 2 == 0))
    }

    fn scaled(&self, k: i32) -> Box<dyn Iterator<Item = i32> + '_> {
        Box::new(self.v.iter().map(move |x| x * k))
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone> Stack<T> {
    const MAX: usize = 3;

    fn push(&mut self, x: T) -> bool {
        if self.items.len() >= Self::MAX {
            return false;
        }
        self.items.push(x);
        true
    }
}

fn merge<K, V>(a: &BTreeMap<K, V>, b: &BTreeMap<K, V>) -> BTreeMap<K, V>
where
    K: Ord + Clone,
    V: Clone + std::ops::Add<Output = V>,
{
    let mut out = a.clone();
    for (k, v) in b {
        match out.get(k) {
            Some(existing) => {
                let sum = existing.clone() + v.clone();
                out.insert(k.clone(), sum);
            }
            None => {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    out
}

fn main() {
    let mut handlers: HashMap<String, Box<dyn Handler>> = HashMap::new();
    handlers.insert("echo".to_string(), Box::new(Echo { count: 0 }));
    handlers.insert("upper".to_string(), Box::new(Upper { count: 0, prefix: ">> ".into() }));
    for (name, input) in [("echo", "hi"), ("upper", "hi"), ("upper", "again"), ("missing", "x")] {
        if let Some(h) = handlers.get_mut(name) {
            println!("{name}: {}", h.handle(input));
        } else {
            println!("{name}: no handler");
        }
    }
    let mut names: Vec<&String> = handlers.keys().collect();
    names.sort();
    for n in names {
        println!("{n} called {} times", handlers[n].calls());
    }
    let total: usize = handlers.values().map(|h| h.calls()).sum();
    println!("total {total}");

    let nums = Numbers { v: vec![1, 2, 3, 4, 5, 6] };
    println!("{:?} {:?}", nums.evens().collect::<Vec<_>>(), nums.scaled(10).take(2).collect::<Vec<_>>());
    println!("{} {:?}", nums.evens().count(), nums.scaled(-1).max());

    let mut stack = Stack { items: Vec::new() };
    let pushed: Vec<bool> = ["a", "b", "c", "d"].iter().map(|s| stack.push(s.to_string())).collect();
    println!("{:?} {:?} {}", pushed, stack.items, Stack::<String>::MAX);

    let a: BTreeMap<&str, i32> = [("x", 1), ("y", 2)].into_iter().collect();
    let b: BTreeMap<&str, i32> = [("y", 20), ("z", 30)].into_iter().collect();
    println!("{:?}", merge(&a, &b));
    let fa: BTreeMap<u8, f64> = BTreeMap::from([(1, 0.5)]);
    let fb: BTreeMap<u8, f64> = BTreeMap::from([(1, 0.25), (2, 1.0)]);
    println!("{:?}", merge(&fa, &fb));

    let nested: Option<Vec<Result<i32, String>>> = Some(vec![Ok(1), Err("bad".into()), Ok(3)]);
    println!("{:?}", nested);
    println!("{:#?}", nested.as_ref().map(|v| v.len()));
    let oks: Vec<i32> = nested.iter().flatten().filter_map(|r| r.as_ref().ok()).copied().collect();
    println!("{:?}", oks);

    let mut rows = vec![vec![3, 1], vec![1, 2, 3], vec![1, 2], vec![], vec![3]];
    rows.sort();
    println!("{:?}", rows);
    println!("{} {} {}", vec![1, 2] < vec![1, 3], vec![1, 2, 0] > vec![1, 2], Vec::<i32>::new() < vec![0]);
    println!("{:?} {:?}", rows.iter().max(), rows.iter().min_by_key(|r| (r.len(), r.iter().sum::<i32>())));
    let mut keyed: Vec<(usize, Vec<char>)> = vec![(2, vec!['b']), (1, vec!['z']), (2, vec!['a'])];
    keyed.sort_by_key(|(n, v)| (*n, v.clone()));
    println!("{:?}", keyed);
    println!("{}", (1, "b") < (1, "c"));
    println!("{:?} {:?}", "abc".cmp("abd"), [1.5, 2.0].partial_cmp(&[1.5, 1.0]));
}
