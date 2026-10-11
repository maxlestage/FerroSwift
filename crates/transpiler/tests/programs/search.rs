use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn neighbors(&self) -> Vec<Pos> {
        [(0, 1), (1, 0), (0, -1), (-1, 0)]
            .iter()
            .map(|(dx, dy)| Pos { x: self.x + dx, y: self.y + dy })
            .collect()
    }
}

fn bfs(walls: &HashSet<Pos>, start: Pos, goal: Pos, size: i32) -> Option<Vec<Pos>> {
    let mut queue = VecDeque::new();
    let mut came_from: HashMap<Pos, Pos> = HashMap::new();
    queue.push_back(start.clone());
    let mut seen = HashSet::new();
    seen.insert(start.clone());
    while let Some(cur) = queue.pop_front() {
        if cur == goal {
            let mut path = vec![cur.clone()];
            let mut at = cur;
            while let Some(prev) = came_from.get(&at) {
                path.push(prev.clone());
                at = prev.clone();
            }
            path.reverse();
            return Some(path);
        }
        for n in cur.neighbors() {
            if n.x < 0 || n.y < 0 || n.x >= size || n.y >= size || walls.contains(&n) {
                continue;
            }
            if seen.insert(n.clone()) {
                came_from.insert(n.clone(), cur.clone());
                queue.push_back(n);
            }
        }
    }
    None
}

fn word_freq(text: &str) -> Vec<(String, usize)> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for word in text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
        *freq.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    let mut pairs: Vec<(String, usize)> = freq.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    pairs
}

fn main() {
    let walls: HashSet<Pos> = [(1, 0), (1, 1), (1, 2), (3, 1), (3, 2), (3, 3)].iter().map(|&(x, y)| Pos { x, y }).collect();
    match bfs(&walls, Pos { x: 0, y: 0 }, Pos { x: 4, y: 0 }, 5) {
        Some(path) => {
            println!("{} steps", path.len() - 1);
            println!("{}", path.iter().map(|p| format!("({},{})", p.x, p.y)).collect::<Vec<_>>().join(">"));
        }
        None => println!("no path"),
    }
    println!("{:?}", bfs(&walls, Pos { x: 0, y: 0 }, Pos { x: 1, y: 1 }, 5).map(|p| p.len()));
    let mut full = walls.clone();
    full.insert(Pos { x: 1, y: 3 });
    full.insert(Pos { x: 1, y: 4 });
    println!("{:?}", bfs(&full, Pos { x: 0, y: 0 }, Pos { x: 4, y: 4 }, 5).is_none());
    let text = "The cat and the hat. The CAT sat; and sat, and SAT!";
    for (w, n) in word_freq(text).iter().take(4) {
        println!("{w:<5}{n:>3}");
    }
    let mut dq: VecDeque<i32> = (1..=5).collect();
    dq.rotate_left(2);
    dq.push_front(0);
    let back = dq.pop_back();
    println!("{:?} {:?} {:?} {:?} {}", dq, back, dq.front(), dq.back(), dq.len());
    dq.retain(|x| x % 2 == 1);
    println!("{:?} {:?} {:?}", dq, dq.iter().rev().collect::<Vec<_>>(), dq.contains(&3));
    let (a, b) = dq.as_slices();
    println!("{} {}", a.len() + b.len(), dq.iter().sum::<i32>());
    dq.extend([7, 9]);
    dq[0] = 100;
    println!("{:?} {:?} {:?}", dq, dq.get(1), dq.iter().position(|&x| x == 9));
    let mut grid: BTreeMap<(i32, i32), char> = BTreeMap::new();
    for (i, c) in "abcdef".chars().enumerate() {
        grid.insert(((i % 3) as i32, (i / 3) as i32), c);
    }
    println!("{:?}", grid);
    let row1: String = grid.iter().filter(|((_, y), _)| *y == 1).map(|(_, c)| *c).collect();
    println!("{row1} {:?} {:?}", grid.get(&(2, 0)), grid.keys().next_back());
    let mut counts: BTreeMap<char, Vec<usize>> = BTreeMap::new();
    for (i, c) in "mississippi".char_indices() {
        counts.entry(c).or_default().push(i);
    }
    for (c, idx) in &counts {
        println!("{c}: {idx:?}");
    }
    let uniq: HashSet<char> = "mississippi".chars().collect();
    let mut u: Vec<_> = uniq.into_iter().collect();
    u.sort();
    println!("{:?} {}", u, u.iter().collect::<String>());
    let inverted: HashMap<usize, Vec<char>> = counts.iter().fold(HashMap::new(), |mut acc, (c, idx)| {
        acc.entry(idx.len()).or_insert_with(Vec::new).push(*c);
        acc
    });
    let mut keys: Vec<_> = inverted.keys().collect();
    keys.sort_unstable_by(|a, b| b.cmp(a));
    for k in keys {
        println!("{k} -> {:?}", inverted[k]);
    }
}
