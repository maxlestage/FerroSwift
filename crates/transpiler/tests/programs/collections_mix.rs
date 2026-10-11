use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

fn main() {
    let v: Vec<i32> = (1..=12).collect();
    let (even, odd): (Vec<i32>, Vec<i32>) = v.iter().partition(|&&x| x % 2 == 0);
    println!("{:?} {:?}", even, odd);
    let running: Vec<i32> = v.iter().scan(0, |acc, &x| { *acc += x; Some(*acc) }).take_while(|&s| s < 40).collect();
    println!("{:?}", running);
    println!("{:?}", v.iter().skip_while(|&&x| x < 5).step_by(3).collect::<Vec<_>>());
    let (a, b): (Vec<i32>, Vec<char>) = v.iter().take(3).map(|&x| (x, (b'a' + x as u8) as char)).unzip();
    println!("{:?} {:?}", a, b);
    let words = ["apple", "fig", "banana", "kiwi", "cherry"];
    println!("{:?} {:?}", words.iter().min_by_key(|w| w.len()), words.iter().max_by(|x, y| x.len().cmp(&y.len())));
    println!("{:?} {:?} {}", words.iter().rev().enumerate().last(), words.iter().nth(2), words.iter().filter(|w| w.contains('a')).count());
    println!("{}", (1..=5).product::<u64>());
    let mut it = v.iter().peekable();
    let mut groups = vec![];
    while let Some(&x) = it.next() {
        let mut g = vec![x];
        while let Some(&&y) = it.peek() {
            if y - g[g.len() - 1] == 1 && g.len() < 4 { g.push(y); it.next(); } else { break; }
        }
        groups.push(g);
    }
    println!("{:?}", groups);
    let chained: Vec<i32> = v.iter().take(2).chain(v.iter().rev().take(2)).cloned().collect();
    println!("{:?}", chained);
    println!("{:?}", v.chunks(5).map(|c| c.iter().sum::<i32>()).collect::<Vec<_>>());
    println!("{:?}", v.windows(3).filter(|w| w[1] % 4 == 0).collect::<Vec<_>>());
    let mut d = vec![3, 1, 3, 3, 2, 2, 5, 1];
    d.dedup();
    println!("{:?}", d);
    d.sort_unstable();
    d.dedup();
    println!("{:?} {:?} {:?}", d, d.binary_search(&3), d.binary_search(&4));
    d.retain(|&x| x != 2);
    let drained: Vec<i32> = d.drain(..1).collect();
    println!("{:?} {:?}", d, drained);
    let mut q: VecDeque<i32> = VecDeque::new();
    q.push_back(1);
    q.push_back(2);
    q.push_front(0);
    q.rotate_left(1);
    println!("{:?} {:?} {:?} {}", q, q.front(), q.back(), q.contains(&2));
    let s1: BTreeSet<i32> = [1, 2, 3, 4].into_iter().collect();
    let s2: BTreeSet<i32> = [3, 4, 5].into_iter().collect();
    println!("{:?} {:?} {:?} {:?}", s1.union(&s2).collect::<Vec<_>>(), s1.intersection(&s2).collect::<Vec<_>>(), s1.difference(&s2).collect::<Vec<_>>(), s1.symmetric_difference(&s2).collect::<Vec<_>>());
    println!("{:?} {:?}", s1.range(2..4).collect::<Vec<_>>(), s1.iter().next_back());
    let mut m: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (i, w) in words.iter().enumerate() {
        m.entry(w.len().to_string()).or_default().push(i as u32);
    }
    println!("{:?}", m);
    if let Some(list) = m.get_mut("5") {
        list.push(99);
    }
    m.entry("9".into()).and_modify(|l| l.clear()).or_insert_with(|| vec![7]);
    println!("{:?} {:?} {:?}", m.first_key_value(), m.last_key_value(), m.range("4".to_string()..).count());
    let mut hm: HashMap<&str, i32> = HashMap::new();
    *hm.entry("a").or_insert(0) += 5;
    hm.insert("b", 2);
    let mut keys: Vec<_> = hm.iter().map(|(k, v)| format!("{k}={v}")).collect();
    keys.sort();
    println!("{:?} {} {:?}", keys, hm.contains_key("b"), hm.remove("zz"));
    let total: i32 = hm.values().sum();
    println!("{total}");
}
