use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    priority: u8,
    done: bool,
}

fn main() {
    let mut v = vec![5, 3, 8, 1, 9, 3];
    v.push(7);
    v.insert(0, 4);
    let removed = v.remove(2);
    println!("{:?} removed {}", v, removed);
    v.sort();
    v.dedup();
    println!("sorted {:?} len {} first {:?} last {:?}", v, v.len(), v.first(), v.last());
    v.retain(|x| x % 2 == 1);
    println!("odd {:?} contains 9: {}", v, v.contains(&9));
    v.reverse();
    println!("reversed {:?}", v);
    let popped = v.pop();
    println!("popped {:?} -> {:?}", popped, v);
    v.extend(vec![10, 20]);
    v.truncate(3);
    println!("truncated {:?}", v);
    println!("get {:?} {:?}", v.get(1), v.get(10));
    let windows: Vec<i32> = [1, 2, 3, 4].windows(2).map(|w| w[0] + w[1]).collect();
    println!("windows {:?}", windows);
    let chunks: Vec<Vec<i32>> = [1, 2, 3, 4, 5].chunks(2).map(|c| c.to_vec()).collect();
    println!("chunks {:?}", chunks);

    let mut tasks = vec![
        Task { id: 1, title: "write".into(), priority: 2, done: false },
        Task { id: 2, title: "test".into(), priority: 1, done: true },
        Task { id: 3, title: "ship".into(), priority: 3, done: false },
    ];
    tasks.sort_by_key(|t| t.priority);
    let titles: Vec<&str> = tasks.iter().map(|t| t.title.as_str()).collect();
    println!("by priority: {:?}", titles);
    tasks.sort_by(|a, b| b.id.cmp(&a.id));
    println!("by id desc: {:?}", tasks.iter().map(|t| t.id).collect::<Vec<_>>());
    for t in tasks.iter_mut() {
        t.done = true;
    }
    println!("all done: {}", tasks.iter().all(|t| t.done));
    if let Some(t) = tasks.iter().find(|t| t.title == "ship") {
        println!("found {:?}", t);
    }

    let mut scores: HashMap<&str, Vec<u32>> = HashMap::new();
    for (name, score) in [("ann", 3), ("bob", 5), ("ann", 7)] {
        scores.entry(name).or_insert_with(Vec::new).push(score);
    }
    let mut names: Vec<_> = scores.keys().collect();
    names.sort();
    for name in names {
        let s = &scores[name];
        println!("{name}: {:?} total {}", s, s.iter().sum::<u32>());
    }
    println!("has bob: {}, has zed: {}", scores.contains_key("bob"), scores.contains_key("zed"));
    if let Some(list) = scores.get("ann") {
        println!("ann played {} times", list.len());
    }
    let removed = scores.remove("bob");
    println!("removed {:?}, left {}", removed, scores.len());

    let mut seen = HashSet::new();
    let mut dups = Vec::new();
    for x in [1, 2, 3, 2, 1, 4] {
        if !seen.insert(x) {
            dups.push(x);
        }
    }
    println!("dups {:?} unique count {}", dups, seen.len());

    let mut tree = BTreeMap::new();
    tree.insert("pear", 3);
    tree.insert("apple", 5);
    tree.insert("fig", 1);
    for (k, v) in &tree {
        println!("{k} = {v}");
    }
    let total: i32 = tree.values().sum();
    println!("fruit total {total}");

    let mut queue = VecDeque::new();
    queue.push_back(1);
    queue.push_back(2);
    queue.push_front(0);
    while let Some(x) = queue.pop_front() {
        print!("{x} ");
    }
    println!();
}
