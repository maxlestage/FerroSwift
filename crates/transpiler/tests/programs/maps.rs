use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Kind {
    Fruit,
    Veg,
}

fn main() {
    let mut stock: HashMap<String, u32> = HashMap::new();
    stock.insert("apple".to_string(), 3);
    stock.insert("pear".to_string(), 0);
    let old = stock.insert("apple".to_string(), 5);
    println!("{:?} {}", old, stock.len());
    *stock.entry("apple".to_string()).or_insert(0) += 2;
    *stock.entry("kiwi".to_string()).or_default() += 1;
    stock.entry("pear".to_string()).and_modify(|n| *n += 10).or_insert(99);
    stock.entry("fig".to_string()).and_modify(|n| *n += 10).or_insert(99);
    let mut keys: Vec<&String> = stock.keys().collect();
    keys.sort();
    for k in &keys {
        println!("{k} = {}", stock[*k]);
    }
    if let Some(n) = stock.get_mut("kiwi") {
        *n *= 4;
    }
    println!("{:?} {:?}", stock.get("kiwi"), stock.get("nope"));
    println!("{}", stock.contains_key("fig"));
    let total: u32 = stock.values().sum();
    println!("total {total}");
    let mut sorted: Vec<(&String, &u32)> = stock.iter().collect();
    sorted.sort();
    println!("{:?}", sorted);
    let removed = stock.remove("pear");
    println!("{:?} {:?}", removed, stock.remove("pear"));
    stock.retain(|_, v| *v > 1);
    let mut left: Vec<_> = stock.into_iter().collect();
    left.sort();
    println!("{:?}", left);

    let text = "the cat and the dog and the bird";
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for w in text.split(' ') {
        *counts.entry(w).or_insert(0) += 1;
    }
    println!("{:?}", counts);
    for (word, n) in &counts {
        if *n > 1 {
            println!("{word}: {n}");
        }
    }
    let most = counts.iter().max_by_key(|(_, n)| **n).map(|(w, _)| *w);
    println!("{:?}", most);
    println!("{:?} {:?}", counts.keys().next(), counts.values().last());
    println!("{:?}", counts.first_key_value());
    println!("{:?}", counts.last_key_value());
    let lens: BTreeMap<usize, Vec<&str>> = counts.keys().fold(BTreeMap::new(), |mut m, w| {
        m.entry(w.len()).or_insert_with(Vec::new).push(*w);
        m
    });
    println!("{:?}", lens);
    let by_kind: HashMap<Kind, Vec<&str>> = HashMap::from([
        (Kind::Fruit, vec!["apple", "fig"]),
        (Kind::Veg, vec!["kale"]),
    ]);
    let mut kinds: Vec<&Kind> = by_kind.keys().collect();
    kinds.sort();
    println!("{:?} {:?}", kinds, by_kind[&Kind::Veg]);
    let inverted: BTreeMap<&str, Kind> = by_kind
        .iter()
        .flat_map(|(k, names)| names.iter().map(move |n| (*n, k.clone())))
        .collect();
    println!("{:?}", inverted);
    let mut values: Vec<u32> = counts.values().map(|n| *n as u32).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    println!("{:?}", values);
    for v in counts.values_mut() {
        *v *= 100;
    }
    println!("{:?}", counts.values().collect::<Vec<_>>());
    println!("{}", counts.is_empty());
    counts.clear();
    println!("{} {:?}", counts.len(), counts);

    let a: HashSet<i32> = [1, 2, 3, 4].into_iter().collect();
    let b: HashSet<i32> = [3, 4, 5].into_iter().collect();
    let mut u: Vec<_> = a.union(&b).copied().collect();
    u.sort();
    let mut i: Vec<_> = a.intersection(&b).copied().collect();
    i.sort();
    let mut d: Vec<_> = a.difference(&b).copied().collect();
    d.sort();
    let mut sd: Vec<_> = a.symmetric_difference(&b).copied().collect();
    sd.sort();
    println!("{:?} {:?} {:?} {:?}", u, i, d, sd);
    println!("{} {} {}", a.is_subset(&b), a.contains(&2), b.is_disjoint(&a));
    let mut s = HashSet::new();
    println!("{} {} {}", s.insert("x"), s.insert("x"), s.remove("x"));
    println!("{} {}", s.len(), s.is_empty());
    let ordered: BTreeSet<char> = "hello world".chars().filter(|c| c.is_alphabetic()).collect();
    println!("{:?} {}", ordered, ordered.iter().collect::<String>());
    println!("{:?} {:?}", ordered.first(), ordered.last());
    println!("{}", ordered.contains(&'w'));
    let evens: BTreeSet<i32> = (1..10).filter(|n| n % 2 == 0).collect();
    println!("{:?} {}", evens, evens.len());
    let small: BTreeSet<i32> = [2, 3, 4].into_iter().collect();
    let mut both: Vec<&i32> = evens.intersection(&small).collect();
    both.sort();
    println!("{:?}", both);

    let mut queue: VecDeque<i32> = VecDeque::new();
    queue.push_back(1);
    queue.push_back(2);
    queue.push_front(0);
    println!("{:?} {:?} {:?} {}", queue, queue.front(), queue.back(), queue.len());
    while let Some(x) = queue.pop_front() {
        print!("{x} ");
        if x == 1 {
            queue.push_back(10);
        }
    }
    println!();
    let mut dq: VecDeque<&str> = ["a", "b", "c"].into_iter().collect();
    dq.rotate_left(1);
    println!("{:?} {:?}", dq, dq.iter().rev().collect::<Vec<_>>());
    println!("{:?} {}", dq.pop_back(), dq.contains(&"a"));
    let v: Vec<&str> = dq.into_iter().collect();
    println!("{:?}", v);

    let pairs = vec![("x", 1), ("y", 2), ("x", 3)];
    let m: HashMap<&str, i32> = pairs.iter().cloned().collect();
    let mut mv: Vec<_> = m.iter().collect();
    mv.sort();
    println!("{:?}", mv);
    let grouped: BTreeMap<&str, Vec<i32>> = pairs.iter().fold(BTreeMap::new(), |mut acc, (k, v)| {
        acc.entry(*k).or_default().push(*v);
        acc
    });
    println!("{:?}", grouped);
    let sums: BTreeMap<&str, i32> = grouped.iter().map(|(k, vs)| (*k, vs.iter().sum())).collect();
    println!("{:?}", sums);
    let nested: BTreeMap<&str, BTreeMap<&str, i32>> = BTreeMap::from([("a", BTreeMap::from([("x", 1)]))]);
    println!("{:?} {:?}", nested, nested.get("a").and_then(|m| m.get("x")));
    println!("{:#?}", sums);
}
