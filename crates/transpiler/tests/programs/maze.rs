use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

const WALL: u8 = 1 << 0;
const START: u8 = 1 << 1;
const GOAL: u8 = 1 << 2;
const SEEN: u8 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cell {
    Open,
    Wall,
    Start,
    Goal,
}

impl Cell {
    fn from_char(c: char) -> Cell {
        match c {
            '#' => Cell::Wall,
            'S' => Cell::Start,
            'G' => Cell::Goal,
            _ => Cell::Open,
        }
    }

    fn glyph(self) -> char {
        match self {
            Cell::Open => '.',
            Cell::Wall => '#',
            Cell::Start => 'S',
            Cell::Goal => 'G',
        }
    }
}

#[derive(Debug, Default)]
struct Maze {
    cells: Vec<Vec<Cell>>,
    flags: Vec<Vec<u8>>,
    width: usize,
    height: usize,
}

impl Maze {
    fn parse(text: &str) -> Maze {
        let cells: Vec<Vec<Cell>> = text.lines().map(|l| l.chars().map(Cell::from_char).collect()).collect();
        let height = cells.len();
        let width = cells.first().map_or(0, Vec::len);
        let mut flags = vec![vec![0u8; width]; height];
        for (y, row) in cells.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                flags[y][x] |= match c {
                    Cell::Wall => WALL,
                    Cell::Start => START,
                    Cell::Goal => GOAL,
                    Cell::Open => 0,
                };
            }
        }
        Maze { cells, flags, width, height }
    }

    fn find(&self, target: Cell) -> Option<Pos> {
        for (y, row) in self.cells.iter().enumerate() {
            if let Some(x) = row.iter().position(|&c| c == target) {
                return Some(Pos { x, y });
            }
        }
        None
    }

    fn neighbors(&self, p: Pos) -> Vec<Pos> {
        let dirs: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        let mut out = Vec::new();
        for (dx, dy) in dirs {
            let nx = p.x as isize + dx;
            let ny = p.y as isize + dy;
            if (0..self.width as isize).contains(&nx) && (0..self.height as isize).contains(&ny) {
                let (nx, ny) = (nx as usize, ny as usize);
                if self.flags[ny][nx] & WALL == 0 {
                    out.push(Pos { x: nx, y: ny });
                }
            }
        }
        out
    }

    fn solve(&mut self) -> Option<Vec<Pos>> {
        let start = self.find(Cell::Start)?;
        let goal = self.find(Cell::Goal)?;
        let mut queue = VecDeque::from([start]);
        let mut came_from: HashMap<Pos, Pos> = HashMap::new();
        let mut visited: HashSet<(usize, usize)> = HashSet::new();
        visited.insert((start.x, start.y));
        while let Some(cur) = queue.pop_front() {
            self.flags[cur.y][cur.x] |= SEEN;
            if cur == goal {
                let mut path = vec![cur];
                let mut at = cur;
                while let Some(&prev) = came_from.get(&at) {
                    path.push(prev);
                    at = prev;
                }
                path.reverse();
                return Some(path);
            }
            for n in self.neighbors(cur) {
                if visited.insert((n.x, n.y)) {
                    came_from.insert(n, cur);
                    queue.push_back(n);
                }
            }
        }
        None
    }

    fn render(&self, path: &[Pos]) -> String {
        let on_path: HashSet<Pos> = path.iter().copied().collect();
        let mut lines = Vec::new();
        for (y, row) in self.cells.iter().enumerate() {
            let line: String = row
                .iter()
                .enumerate()
                .map(|(x, c)| {
                    let p = Pos { x, y };
                    if *c == Cell::Open && on_path.contains(&p) {
                        '*'
                    } else if *c == Cell::Open && self.flags[y][x] & SEEN != 0 {
                        ','
                    } else {
                        c.glyph()
                    }
                })
                .collect();
            lines.push(line);
        }
        lines.join("\n")
    }
}

fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn stats(xs: &[f64]) -> (f64, f64, f64) {
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let max = xs.iter().cloned().fold(f64::MIN, f64::max);
    (mean, var.sqrt(), max)
}

fn main() {
    let text = "S..#....\n.#.#.##.\n.#...#..\n.####.#.\n......#G";
    let mut maze = Maze::parse(text);
    println!("{}x{} {:?} {:?}", maze.width, maze.height, maze.find(Cell::Start), maze.find(Cell::Goal));
    println!("{:?}", maze.neighbors(Pos { x: 0, y: 0 }));
    let path = maze.solve();
    println!("{:?}", path.as_ref().map(|p| p.len()));
    println!("{}", maze.render(path.as_deref().unwrap_or(&[])));
    let seen = maze.flags.iter().flatten().filter(|f| *f & SEEN != 0).count();
    println!("{seen} {:?} {}", path.as_ref().and_then(|p| p.last()), path.as_ref().is_some_and(|p| p.windows(2).all(|w| w[0].x.abs_diff(w[1].x) + w[0].y.abs_diff(w[1].y) == 1)));
    let mut blocked = Maze::parse("S#\n#G");
    println!("{:?} {:?}", blocked.solve(), Maze::default().solve());
    println!("{:016x} {:x} {}", fnv(b"hello"), fnv(&[]), fnv(text.as_bytes()) % 1000);
    let mut scores: BTreeMap<(i32, i32), f64> = BTreeMap::new();
    for (i, w) in ["ab", "c", "def", "gh", "i"].iter().enumerate() {
        *scores.entry((w.len() as i32, i as i32 % 2)).or_insert(0.0) += (i * i) as f64 / 2.0;
    }
    println!("{:?}", scores);
    let mut ranked: Vec<((i32, i32), f64)> = scores.iter().map(|(k, v)| (*k, *v)).collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(&b.0)));
    println!("{:?}", ranked);
    let (mean, sd, max) = stats(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    println!("{mean:.3} {sd:.3} {max}");
    for (a, b) in [(1, 2), (2, 2), (3, 2)] {
        let word = match a.cmp(&b) {
            Ordering::Less => "less",
            Ordering::Equal => "equal",
            Ordering::Greater => "greater",
        };
        println!("{a} vs {b}: {word} {:?} {} {}", a.cmp(&b).reverse(), a.cmp(&b).is_lt(), a.cmp(&b).then(b.cmp(&a)) == Ordering::Equal);
    }
    let pairs: Vec<(i32, i32)> = (1..=6).collect::<Vec<_>>().chunks(2).map(|c| (c[0], c[1])).collect();
    let zipped: Vec<i32> = (0..3).zip(10..).map(|(a, b)| a * b).collect();
    println!("{:?} {:?} {:?}", pairs, zipped, (1..4).zip("abc".chars()).skip(1).collect::<Vec<_>>());
    let mut grid = vec![vec!['.'; 5]; 3];
    grid[1][2] = '#';
    let (mut a, mut b) = (grid[0].clone(), grid[1].clone());
    std::mem::swap(&mut a, &mut b);
    grid[0] = a;
    grid[1] = b;
    let flat: String = grid.iter().map(|r| r.iter().collect::<String>()).collect::<Vec<_>>().join("/");
    println!("{flat} {}", grid.iter().flatten().filter(|&&c| c == '#').count());
    let flags = WALL | GOAL;
    println!("{} {} {} {} {:#010b}", flags & WALL != 0, flags & START != 0, flags ^ GOAL, !flags & 0xF, flags << 2);
}
