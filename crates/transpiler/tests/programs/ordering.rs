use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Priority {
    Low,
    Medium(u8),
    High { urgent: bool },
}

#[derive(Debug, Default, Clone, PartialEq)]
enum Mode {
    #[default]
    Idle,
    Running(u32),
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
struct Tag(String, u8);

#[derive(Debug, Default)]
struct Config {
    name: String,
    retries: u32,
    mode: Mode,
    tags: Vec<Tag>,
    ratio: f64,
    enabled: bool,
    extra: Option<Box<Config>>,
}

fn main() {
    let v1 = Version { major: 1, minor: 2, patch: 3 };
    let v2 = Version { major: 1, minor: 10, patch: 0 };
    let v3 = Version { major: 0, minor: 99, patch: 99 };
    println!("{} {} {} {:?}", v1 < v2, v2 > v3, v1 == v1, v1.cmp(&v3));
    let mut versions = vec![v2, v1, v3, v1];
    versions.sort();
    println!("{:?}", versions);
    versions.dedup();
    println!("{:?} {:?} {:?}", versions.len(), versions.iter().max(), versions.iter().min().map(|v| v.minor));
    println!("{:?}", versions.binary_search(&v2));
    println!("{:?}", versions.binary_search_by_key(&10, |v| v.minor));
    let set: HashSet<Version> = versions.iter().copied().collect();
    println!("{} {}", set.len(), set.contains(&v1));
    let ps = vec![Priority::High { urgent: false }, Priority::Low, Priority::Medium(5), Priority::Medium(2), Priority::High { urgent: true }];
    let mut sorted = ps.clone();
    sorted.sort();
    println!("{:?}", sorted);
    println!("{} {} {}", Priority::Low < Priority::Medium(0), Priority::Medium(9) < Priority::High { urgent: false }, Priority::Medium(1) < Priority::Medium(2));
    println!("{:?} {:?}", ps.iter().max(), ps.iter().min());
    let mut counts: HashMap<Priority, usize> = HashMap::new();
    for p in &ps {
        *counts.entry(p.clone()).or_insert(0) += 1;
    }
    let mut keys: Vec<_> = counts.keys().cloned().collect();
    keys.sort_by(|a, b| b.cmp(a));
    println!("{:?}", keys);
    println!("{:?} {:?}", Mode::default(), Config::default());
    let t1 = Tag("b".into(), 1);
    let t2 = Tag("a".into(), 9);
    println!("{} {:?} {:?}", t1 > t2, t1.max(t2.clone()), Tag::default());
    println!("{} {} {}", vec![1, 2] < vec![1, 3], vec![1, 2, 3] < vec![1, 2], [3, 1] > [2, 9]);
    println!("{} {} {}", Some(1) < None, Some(5) > Some(3), None::<i32> < Some(0));
    println!("{} {} {}", (1, "b") < (1, "c"), (2, "a") < (1, "z"), ("x", 2.5) > ("x", 2.0));
    println!("{} {} {}", "abc" < "abd", "Z" < "a", "" < "a");
    println!("{:?} {:?}", 1.cmp(&2).reverse(), 2.cmp(&2).then(Ordering::Greater));
    println!("{:?}", 1.cmp(&1).then_with(|| "b".cmp("a")));
    let mut words = vec!["pear", "fig", "banana", "kiwi", "apple"];
    words.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
    println!("{:?}", words);
    words.sort_by_key(|w| Reverse(w.len()));
    println!("{:?}", words);
    words.sort_unstable_by(|a, b| b.cmp(a));
    println!("{:?} {}", words, words.is_sorted_by(|a, b| a >= b));
    println!("{} {}", [1, 2, 2, 3].is_sorted(), [3, 1].is_sorted());
    let mut pairs = vec![(2, 'b'), (1, 'z'), (2, 'a'), (1, 'a')];
    pairs.sort();
    println!("{:?}", pairs);
    pairs.dedup_by_key(|p| p.0);
    println!("{:?}", pairs);
    println!("{:?} {:?}", pairs.iter().max_by(|a, b| a.1.cmp(&b.1)), pairs.iter().min_by_key(|p| (p.1, p.0)));
    let ordered: BTreeSet<Priority> = ps.iter().cloned().collect();
    println!("{:?}", ordered);
    println!("{:?} {:?}", ordered.first(), ordered.last());
    println!("{:?}", ordered.range(Priority::Medium(0)..).count());
    println!("{} {}", v1.clamp(v3, v2) == v1, Priority::Low.clamp(Priority::Medium(1), Priority::Medium(3)) == Priority::Medium(1));
    println!("{:?} {:?}", std::cmp::max(v1, v2).minor, std::cmp::min("b", "a"));
    println!("{:?}", versions.iter().map(|v| v.patch).rev().collect::<Vec<_>>());
    println!("{:?} {:?}", 3.0f64.partial_cmp(&f64::NAN), 1.0f64.partial_cmp(&2.0));
    println!("{:?}", [2.5, -1.0, 9.0].iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let mut floats = vec![2.5, -1.0, 9.0, 0.0];
    floats.sort_by(|a, b| b.partial_cmp(a).unwrap());
    println!("{:?}", floats);
    floats.sort_by(f64::total_cmp);
    println!("{:?} {:?}", floats, floats.iter().copied().reduce(f64::min));
    println!("{}", matches!(Priority::Medium(3), Priority::Medium(1..=5)));
    println!("{}", Mode::Running(3) == Mode::Running(3) && Mode::Idle != Mode::Running(0));
}
