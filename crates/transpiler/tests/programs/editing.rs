use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
struct Settings {
    volume: u8,
    name: String,
    tags: Vec<String>,
    parent: Option<Box<Settings>>,
}

#[derive(Debug)]
struct Cell {
    alive: bool,
    age: u32,
}

struct Grid {
    cells: Vec<Vec<Cell>>,
}

impl Grid {
    fn new(w: usize, h: usize) -> Grid {
        Grid { cells: (0..h).map(|y| (0..w).map(|x| Cell { alive: (x + y) % 3 == 0, age: 0 }).collect()).collect() }
    }

    fn cell_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.cells.get_mut(y).and_then(|row| row.get_mut(x))
    }

    fn age_all(&mut self) {
        for row in self.cells.iter_mut() {
            for c in row.iter_mut().filter(|c| c.alive) {
                c.age += 1;
            }
        }
    }

    fn render(&self) -> String {
        self.cells.iter().map(|row| row.iter().map(|c| if c.alive { char::from_digit(c.age % 10, 10).unwrap() } else { '.' }).collect::<String>()).collect::<Vec<_>>().join("/")
    }
}

#[derive(Debug)]
struct Player {
    name: String,
    score: i32,
    items: Vec<&'static str>,
}

fn main() {
    let base = Settings { volume: 5, name: "base".into(), ..Default::default() };
    let child = Settings { name: "child".into(), parent: Some(Box::new(base.clone())), ..base.clone() };
    println!("{:?}", child);
    let mut s = child;
    s.tags.push("a".into());
    let old_name = std::mem::replace(&mut s.name, "renamed".into());
    let taken = std::mem::take(&mut s.tags);
    let parent = s.parent.take();
    println!("{old_name} {:?} {:?} {:?} {:?}", taken, parent.map(|p| p.volume), s.parent.is_none(), s.tags);
    let mut grid = Grid::new(5, 3);
    grid.age_all();
    grid.age_all();
    if let Some(c) = grid.cell_mut(1, 1) {
        c.alive = true;
        c.age = 7;
    }
    println!("{} {:?}", grid.render(), grid.cell_mut(9, 9).is_none());
    if let Some(row) = grid.cells.last_mut() {
        row.retain(|c| c.alive);
        if let Some(first) = row.first_mut() {
            first.age += 100;
        }
    }
    println!("{} {:?}", grid.render(), grid.cells[2].iter().map(|c| c.age).collect::<Vec<_>>());
    let mut players = vec![
        Player { name: "ann".into(), score: 10, items: vec!["sword"] },
        Player { name: "bob".into(), score: 30, items: vec![] },
        Player { name: "cy".into(), score: 20, items: vec!["bow", "arrow"] },
    ];
    if let Some(p) = players.iter_mut().find(|p| p.name == "bob") {
        p.score += 5;
        p.items.push("shield");
    }
    players.sort_by_key(|p| std::cmp::Reverse(p.score));
    println!("{:?}", players.iter().map(|p| (&p.name, p.score, p.items.len())).collect::<Vec<_>>());
    players.retain_mut(|p| {
        p.score -= 15;
        p.score > 0
    });
    println!("{:?}", players.iter().map(|p| format!("{}={}", p.name, p.score)).collect::<Vec<_>>());
    let mut scores: HashMap<&str, Vec<i32>> = HashMap::new();
    for (name, s) in [("ann", 3), ("bob", 5), ("ann", 4), ("cy", 1), ("bob", 2)] {
        scores.entry(name).or_default().push(s);
    }
    if let Some(v) = scores.get_mut("ann") {
        v.sort_unstable();
        v.dedup();
        v.insert(0, 0);
    }
    scores.entry("bob").and_modify(|v| v.truncate(1)).or_insert_with(Vec::new);
    scores.entry("dee").and_modify(|v| v.clear()).or_insert_with(|| vec![9]);
    let mut keys: Vec<_> = scores.keys().copied().collect();
    keys.sort();
    for k in keys {
        println!("{k}: {:?}", scores[k]);
    }
    let mut v: Vec<i32> = (1..=10).collect();
    let tail = v.split_off(7);
    let drained: Vec<i32> = v.drain(1..3).collect();
    v.append(&mut vec![100, 200]);
    v.extend_from_slice(&tail);
    let removed = v.swap_remove(0);
    v.insert(2, -1);
    let last = v.remove(v.len() - 1);
    v.dedup_by(|a, b| (*a - *b).abs() == 1);
    println!("{:?} {:?} {:?} {} {} {:?}", v, tail, drained, removed, last, v.iter().rposition(|&x| x < 0));
    let mut opts: Vec<Option<Box<i32>>> = vec![Some(Box::new(1)), None, Some(Box::new(3))];
    if let Some(slot) = opts.get_mut(2) {
        *slot = slot.take().map(|b| Box::new(*b + 10));
    }
    opts[1] = Some(Box::new(2));
    let total: i32 = opts.iter().flatten().map(|b| **b).sum();
    println!("{:?} {}", opts, total);
    let mut words = vec!["b", "a", "c"];
    words.swap(0, 2);
    let (x, y) = (words[0], words[2]);
    if let Some(w) = words.first_mut() {
        *w = "z";
    }
    println!("{x} {y} {:?}", words);
}
