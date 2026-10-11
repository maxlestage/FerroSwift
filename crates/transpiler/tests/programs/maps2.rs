use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

fn main() {
    let mut scores: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (name, s) in [("ann", 7), ("bob", 3), ("ann", 9), ("cid", 5), ("bob", 8)] {
        scores.entry(name.to_string()).or_default().push(s);
    }
    scores.entry("dee".into()).and_modify(|v| v.push(1)).or_insert_with(|| vec![0]);
    scores.entry("ann".into()).and_modify(|v| v.push(1)).or_insert_with(|| vec![0]);
    scores.entry("eve".into()).or_insert_with_key(|k| vec![k.len() as u32]);
    println!("{:?}", scores);
    for (name, v) in scores.iter_mut() {
        if name.starts_with('b') {
            v.push(100);
        }
        v.sort_unstable();
    }
    println!("{:?} {:?}", scores.get("bob"), scores.get_key_value("cid"));
    println!("{:?} {:?} {}", scores.remove_entry("eve"), scores.pop_first(), scores.len());
    let totals: BTreeMap<&str, u32> = scores.iter().map(|(k, v)| (k.as_str(), v.iter().sum())).collect();
    println!("{:?} {:?}", totals, totals.iter().max_by_key(|(_, t)| **t));
    let keys: BTreeSet<String> = scores.keys().cloned().collect();
    let vals: Vec<Vec<u32>> = scores.clone().into_values().collect();
    let ks: Vec<String> = scores.clone().into_keys().collect();
    println!("{:?} {:?} {:?}", keys, vals, ks);
    let mut nested: HashMap<String, HashMap<char, usize>> = HashMap::new();
    for word in ["apple", "avocado", "banana", "blueberry", "cherry"] {
        let first = word.chars().next().unwrap().to_string();
        for c in word.chars() {
            *nested.entry(first.clone()).or_default().entry(c).or_insert(0) += 1;
        }
    }
    let mut groups: Vec<(&String, usize, Option<(&char, &usize)>)> = nested
        .iter()
        .map(|(k, m)| (k, m.values().sum::<usize>(), m.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))))
        .collect();
    groups.sort();
    println!("{:?}", groups);
    let mut index: HashMap<char, Vec<usize>> = HashMap::new();
    for (i, c) in "hello world".char_indices() {
        if c != ' ' {
            index.entry(c).or_insert_with(Vec::new).push(i);
        }
    }
    let mut repeated: Vec<(char, &Vec<usize>)> = index.iter().filter(|(_, v)| v.len() > 1).map(|(c, v)| (*c, v)).collect();
    repeated.sort();
    println!("{:?} {}", repeated, index.len());
    let base = HashMap::from([("x", 1), ("y", 2)]);
    let mut merged = base.clone();
    merged.extend([("y", 20), ("z", 30)]);
    merged.extend(BTreeMap::from([("w", 0)]));
    let mut pairs: Vec<_> = merged.iter().collect();
    pairs.sort();
    println!("{:?} {} {:?}", pairs, merged.contains_key("w"), merged.get("q").copied().unwrap_or(-1));
    let drained: BTreeMap<&str, i32> = merged.drain().collect();
    println!("{:?} {}", drained, merged.is_empty());
    let mut seen: HashSet<u32> = HashSet::new();
    let firsts: Vec<bool> = [3, 1, 3, 2, 1].iter().map(|&x| seen.insert(x)).collect();
    println!("{:?} {} {} {}", firsts, seen.remove(&3), seen.remove(&9), seen.contains(&1));
    seen.extend([10, 20]);
    let a: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
    let b: HashSet<u32> = [3, 4, 5].into_iter().collect();
    let mut inter: Vec<u32> = a.intersection(&b).copied().collect();
    inter.sort();
    let uni: BTreeSet<u32> = a.union(&b).cloned().collect();
    let sym: BTreeSet<&u32> = a.symmetric_difference(&b).collect();
    println!("{:?} {:?} {:?} {} {}", inter, uni, sym, a.is_subset(&b), a.intersection(&b).count());
    let mut c = a.clone();
    c.retain(|x| x % 2 == 0);
    let mut cs: Vec<u32> = c.drain().collect();
    cs.sort();
    println!("{:?} {} {:?} {:?}", cs, c.is_empty(), seen.iter().copied().min(), seen.iter().copied().max());
    let mut ring: VecDeque<i32> = VecDeque::with_capacity(4);
    for i in 1..=6 {
        if ring.len() == 4 {
            ring.pop_front();
        }
        ring.push_back(i);
    }
    ring.rotate_right(1);
    ring.extend([7, 8]);
    println!("{:?} {:?} {} {:?}", ring, ring.iter().rev().take(2).collect::<Vec<_>>(), ring.contains(&5), ring.get(0));
    ring.truncate(3);
    let slices_len = { let (front, back) = ring.as_slices(); front.len() + back.len() };
    let contiguous = ring.make_contiguous().to_vec();
    println!("{:?} {} {:?} {:?}", contiguous, slices_len, ring.front(), ring.iter().position(|&x| x == 6));
    let drained: Vec<i32> = ring.drain(..).collect();
    println!("{:?} {}", drained, ring.is_empty());
    let v: VecDeque<_> = (1..=5).collect();
    let (left, right): (Vec<i32>, Vec<i32>) = v.iter().partition(|&&x| x <= 2);
    println!("{:?} {:?} {:?}", left, right, Vec::from(v));
}
