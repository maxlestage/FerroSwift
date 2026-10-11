use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Cell {
    Wall,
    Open,
    Start,
    End,
}

fn parse_grid(text: &str) -> (Vec<Vec<Cell>>, (usize, usize), (usize, usize)) {
    let mut grid = Vec::new();
    let mut start = (0, 0);
    let mut end = (0, 0);
    for (y, line) in text.lines().enumerate() {
        let mut row = Vec::new();
        for (x, c) in line.chars().enumerate() {
            let cell = match c {
                '#' => Cell::Wall,
                'S' => {
                    start = (x, y);
                    Cell::Start
                }
                'E' => {
                    end = (x, y);
                    Cell::End
                }
                _ => Cell::Open,
            };
            row.push(cell);
        }
        grid.push(row);
    }
    (grid, start, end)
}

fn bfs(grid: &[Vec<Cell>], start: (usize, usize), end: (usize, usize)) -> Option<usize> {
    let h = grid.len();
    let w = grid[0].len();
    let mut seen = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((start, 0usize));
    seen.insert(start);
    while let Some(((x, y), d)) = queue.pop_front() {
        if (x, y) == end {
            return Some(d);
        }
        let dirs: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for (dx, dy) in dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let next = (nx as usize, ny as usize);
            if grid[next.1][next.0] == Cell::Wall || !seen.insert(next) {
                continue;
            }
            queue.push_back((next, d + 1));
        }
    }
    None
}

#[derive(Debug, Default)]
struct Inventory {
    stock: BTreeMap<String, u32>,
    log: Vec<String>,
}

impl Inventory {
    fn add(&mut self, item: &str, n: u32) -> &mut Self {
        *self.stock.entry(item.to_string()).or_insert(0) += n;
        self.log.push(format!("+{n} {item}"));
        self
    }

    fn take(&mut self, item: &str, n: u32) -> Result<u32, String> {
        let have = self.stock.get_mut(item).ok_or(format!("no {item}"))?;
        if *have < n {
            return Err(format!("only {have} {item}"));
        }
        *have -= n;
        let left = *have;
        if left == 0 {
            self.stock.remove(item);
        }
        self.log.push(format!("-{n} {item}"));
        Ok(left)
    }

    fn total(&self) -> u32 {
        self.stock.values().sum()
    }
}

fn word_stats(text: &str) -> (usize, f64, Option<String>) {
    let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return (0, 0.0, None);
    }
    let total: usize = words.iter().map(|w| w.len()).sum();
    let avg = total as f64 / words.len() as f64;
    let mut freq: HashMap<String, usize> = HashMap::new();
    for w in &words {
        *freq.entry(w.to_lowercase()).or_default() += 1;
    }
    let top = freq
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(w, _)| w.clone());
    (words.len(), avg, top)
}

fn main() {
    let maze = "S..#\n.#.#\n.#..\n...E";
    let (grid, start, end) = parse_grid(maze);
    println!("{:?} {:?} {:?}", grid[0], start, end);
    println!("{:?}", bfs(&grid, start, end));
    let (grid2, s2, e2) = parse_grid("S#E");
    println!("{:?}", bfs(&grid2, s2, e2));
    let walls = grid.iter().flatten().filter(|c| **c == Cell::Wall).count();
    println!("{walls} {:?}", grid.iter().map(|r| r.iter().filter(|c| **c != Cell::Wall).count()).collect::<Vec<_>>());
    let mut kinds: Vec<Cell> = grid.iter().flatten().copied().collect::<HashSet<_>>().into_iter().collect();
    kinds.sort();
    println!("{:?} {:?}", kinds, Cell::Wall < Cell::End);
    let mut inv = Inventory::default();
    inv.add("bolt", 5).add("nut", 2).add("bolt", 1);
    println!("{:?} {}", inv.stock, inv.total());
    println!("{:?} {:?} {:?}", inv.take("bolt", 4), inv.take("nut", 5), inv.take("gear", 1));
    println!("{:?} {:?}", inv.take("nut", 2), inv.stock.keys().collect::<Vec<_>>());
    println!("{}", inv.log.join(" "));
    let (n, avg, top) = word_stats("The cat and the hat; THE end.");
    println!("{n} {avg:.2} {top:?}");
    println!("{:?}", word_stats("  ,,  "));
    let mut by_len: BTreeMap<usize, BTreeMap<char, Vec<&str>>> = BTreeMap::new();
    for w in ["apple", "avocado", "bean", "beet", "corn", "cherry"] {
        by_len.entry(w.len()).or_default().entry(w.chars().next().unwrap()).or_default().push(w);
    }
    for (len, by_first) in &by_len {
        let parts: Vec<String> = by_first.iter().map(|(c, ws)| format!("{c}:{}", ws.join("+"))).collect();
        println!("{len} -> {}", parts.join(" "));
    }
    let first_key = by_len.keys().next().copied();
    let last_entry = by_len.iter().next_back().map(|(k, v)| (*k, v.len()));
    println!("{:?} {:?} {}", first_key, last_entry, by_len.contains_key(&6));
    let range: Vec<usize> = by_len.range(5..7).map(|(k, _)| *k).collect();
    println!("{:?}", range);
    if let Some(v) = by_len.get_mut(&4) {
        v.remove(&'b');
    }
    println!("{:?}", by_len.get(&4).map(|m| m.len()));
}
