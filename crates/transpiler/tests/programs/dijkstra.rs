use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::fmt;

#[derive(Debug, Clone, Default)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone + fmt::Debug> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }

    fn push(&mut self, item: T) -> &mut Self {
        self.items.push(item);
        self
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

    fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Stack<U> {
        Stack { items: self.items.iter().map(f).collect() }
    }
}

impl<T: fmt::Display> fmt::Display for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.items.iter().map(|i| i.to_string()).collect();
        write!(f, "[{}]", parts.join(" > "))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Pair<A, B> {
    first: A,
    second: B,
}

impl<A: Clone, B: Clone> Pair<A, B> {
    fn swap(&self) -> Pair<B, A> {
        Pair { first: self.second.clone(), second: self.first.clone() }
    }
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for Pair<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.first, self.second)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn parse(s: &str) -> Option<Version> {
        let nums: Vec<u32> = s.split('.').map(|p| p.parse().ok()).collect::<Option<Vec<_>>>()?;
        if let [major, minor, patch] = nums[..] {
            Some(Version { major, minor, patch })
        } else {
            None
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major.cmp(&other.major).then(self.minor.cmp(&other.minor)).then(self.patch.cmp(&other.patch))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn largest<T: PartialOrd + Clone>(items: &[T]) -> Option<T> {
    let mut best: Option<&T> = None;
    for item in items {
        if best.map_or(true, |b| item > b) {
            best = Some(item);
        }
    }
    best.cloned()
}

fn dijkstra(edges: &HashMap<&str, Vec<(&str, u32)>>, start: &str, goal: &str) -> Option<(u32, Vec<String>)> {
    let mut dist: HashMap<String, u32> = HashMap::new();
    let mut prev: HashMap<String, String> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(start.to_string(), 0);
    heap.push(Reverse((0u32, start.to_string())));
    while let Some(Reverse((d, node))) = heap.pop() {
        if node == goal {
            let mut path = vec![goal.to_string()];
            let mut cur = goal.to_string();
            while let Some(p) = prev.get(&cur) {
                path.push(p.clone());
                cur = p.clone();
            }
            path.reverse();
            return Some((d, path));
        }
        if d > *dist.get(&node).unwrap_or(&u32::MAX) {
            continue;
        }
        if let Some(neighbors) = edges.get(node.as_str()) {
            for (next, w) in neighbors {
                let nd = d + w;
                if nd < *dist.get(*next).unwrap_or(&u32::MAX) {
                    dist.insert(next.to_string(), nd);
                    prev.insert(next.to_string(), node.clone());
                    heap.push(Reverse((nd, next.to_string())));
                }
            }
        }
    }
    None
}

fn main() {
    let mut s: Stack<i32> = Stack::new();
    s.push(1).push(2).push(3);
    println!("{} {:?} {}", s, s.peek(), s.len());
    println!("{:?}", s.pop());
    let doubled = s.map(|x| x * 2);
    let named = s.map(|x| format!("n{x}"));
    println!("{doubled} {named} {:?}", named.items);
    let empty: Stack<String> = Stack::default();
    println!("{} {}", empty, empty.len());
    let p = Pair { first: "x", second: 2.5 };
    println!("{} {} {:?}", p, p.swap(), p.swap().swap() == p);
    let mut versions: Vec<Version> = ["1.2.3", "1.10.0", "0.9.9", "1.2.10", "bad", "1.2"].iter().filter_map(|s| Version::parse(s)).collect();
    versions.sort();
    println!("{}", versions.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" < "));
    println!("{:?} {:?}", largest(&versions).map(|v| v.to_string()), largest(&[3.5, 9.25, -1.0]));
    println!("{:?} {}", largest::<i32>(&[]), Version::parse("2.0.0").unwrap() > versions[0]);
    println!("{:?}", versions.iter().max().map(|v| v.minor));
    println!("{:?}", versions.binary_search(&Version { major: 1, minor: 2, patch: 10 }));
    let mut edges: HashMap<&str, Vec<(&str, u32)>> = HashMap::new();
    edges.insert("a", vec![("b", 7), ("c", 9), ("f", 14)]);
    edges.insert("b", vec![("c", 10), ("d", 15)]);
    edges.insert("c", vec![("d", 11), ("f", 2)]);
    edges.insert("d", vec![("e", 6)]);
    edges.insert("f", vec![("e", 9)]);
    println!("{:?}", dijkstra(&edges, "a", "e"));
    println!("{:?}", dijkstra(&edges, "e", "a"));
    println!("{:?}", dijkstra(&edges, "a", "a").map(|(d, p)| (d, p.len())));
}
