use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

struct Cache<K, V> {
    map: HashMap<K, V>,
    order: VecDeque<K>,
    capacity: usize,
    hits: u32,
    misses: u32,
}

impl<K: Hash + Eq + Clone + Debug, V: Clone> Cache<K, V> {
    fn new(capacity: usize) -> Self {
        Cache { map: HashMap::new(), order: VecDeque::new(), capacity, hits: 0, misses: 0 }
    }

    fn get(&mut self, key: &K) -> Option<V> {
        match self.map.get(key) {
            Some(v) => {
                self.hits += 1;
                Some(v.clone())
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    fn put(&mut self, key: K, value: V) -> Option<K> {
        let mut evicted = None;
        if !self.map.contains_key(&key) {
            if self.order.len() == self.capacity {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                    evicted = Some(old);
                }
            }
            self.order.push_back(key.clone());
        }
        self.map.insert(key, value);
        evicted
    }

    fn keys(&self) -> Vec<K> {
        self.order.iter().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Pair<A, B> {
    first: A,
    second: B,
}

impl<A: Clone, B: Clone> Pair<A, B> {
    fn swap(&self) -> Pair<B, A> {
        Pair { first: self.second.clone(), second: self.first.clone() }
    }

    fn map_first<C>(self, f: impl Fn(A) -> C) -> Pair<C, B> {
        Pair { first: f(self.first), second: self.second }
    }
}

fn group_by<T: Clone, K: Ord, F: Fn(&T) -> K>(items: &[T], key: F) -> BTreeMap<K, Vec<T>> {
    let mut groups: BTreeMap<K, Vec<T>> = BTreeMap::new();
    for item in items {
        groups.entry(key(item)).or_default().push(item.clone());
    }
    groups
}

fn or_default<T: Default>(o: Option<T>) -> T {
    o.unwrap_or_default()
}

fn mean<T: Into<f64> + Copy>(xs: &[T]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().map(|&x| x.into()).sum::<f64>() / xs.len() as f64
}

fn largest<T: PartialOrd + Copy>(xs: &[T]) -> Option<T> {
    let mut it = xs.iter();
    let mut best = *it.next()?;
    for &x in it {
        if x > best {
            best = x;
        }
    }
    Some(best)
}

struct Windowed<T> {
    items: Vec<T>,
    size: usize,
    pos: usize,
}

impl<T: Clone> Iterator for Windowed<T> {
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Vec<T>> {
        if self.pos + self.size > self.items.len() {
            return None;
        }
        let w = self.items[self.pos..self.pos + self.size].to_vec();
        self.pos += 1;
        Some(w)
    }
}

fn main() {
    let mut cache: Cache<String, u32> = Cache::new(2);
    println!("{:?} {:?}", cache.put("a".into(), 1), cache.put("b".into(), 2));
    println!("{:?} {:?} {:?}", cache.get(&"a".to_string()), cache.get(&"z".to_string()), cache.put("c".into(), 3));
    println!("{:?} {:?} {} {}", cache.keys(), cache.get(&"a".to_string()), cache.hits, cache.misses);
    println!("{:?}", cache.put("b".into(), 20));
    println!("{:?} {:?}", cache.keys(), cache.get(&"b".to_string()));
    let p = Pair { first: 1, second: "one".to_string() };
    println!("{:?} {:?} {:?}", p.swap(), p.clone().map_first(|n| n as f64 * 1.5), Pair::<u8, bool>::default());
    println!("{}", p == Pair { first: 1, second: "one".into() });
    let words = ["apple", "avocado", "banana", "blueberry", "cherry", "kiwi"];
    let groups = group_by(&words, |w| w.chars().next().unwrap());
    for (k, v) in &groups {
        println!("{k}: {v:?}");
    }
    let by_len = group_by(&[1, 22, 333, 44, 5], |n| n.to_string().len());
    println!("{:?}", by_len);
    println!("{} {:?} {}", or_default::<i32>(None), or_default(Some(vec![1])), or_default::<String>(None).is_empty());
    println!("{} {} {}", mean(&[1u8, 2, 4]), mean(&[1.5f32, 2.5]), mean::<i32>(&[]));
    println!("{:?} {:?} {:?}", largest(&[3, 9, 2]), largest(&[1.5, -2.0]), largest::<char>(&[]));
    println!("{:?}", largest(&["pear", "zoo", "apple"]));
    let w = Windowed { items: vec![1, 2, 3, 4], size: 3, pos: 0 };
    println!("{:?}", w.collect::<Vec<_>>());
    let letters = Windowed { items: "abcd".chars().collect(), size: 2, pos: 0 };
    println!("{:?}", letters.map(|w| w.iter().collect::<String>()).collect::<Vec<_>>());
    println!("{:?}", Windowed { items: vec![1], size: 2, pos: 0 }.next());
}
