#[derive(Clone, PartialEq)]
struct Life { w: usize, h: usize, cells: Vec<bool> }

impl Life {
    fn from_str(s: &str) -> Life {
        let rows: Vec<&str> = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        let h = rows.len();
        let w = rows[0].len();
        let cells = rows.iter().flat_map(|r| r.chars().map(|c| c == '#')).collect();
        Life { w, h, cells }
    }
    fn alive(&self, r: isize, c: isize) -> bool {
        let r = r.rem_euclid(self.h as isize) as usize;
        let c = c.rem_euclid(self.w as isize) as usize;
        self.cells[r * self.w + c]
    }
    fn neighbors(&self, r: usize, c: usize) -> usize {
        let mut n = 0;
        for dr in -1..=1 {
            for dc in -1..=1 {
                if (dr, dc) != (0, 0) && self.alive(r as isize + dr, c as isize + dc) {
                    n += 1;
                }
            }
        }
        n
    }
    fn step(&self) -> Life {
        let mut next = self.clone();
        for r in 0..self.h {
            for c in 0..self.w {
                let n = self.neighbors(r, c);
                let a = self.cells[r * self.w + c];
                next.cells[r * self.w + c] = matches!((a, n), (true, 2) | (_, 3));
            }
        }
        next
    }
    fn render(&self) -> String {
        self.cells.chunks(self.w).map(|row| row.iter().map(|&b| if b { '#' } else { '.' }).collect::<String>()).collect::<Vec<_>>().join("\n")
    }
    fn population(&self) -> usize { self.cells.iter().filter(|&&b| b).count() }
}

fn main() {
    let mut g = Life::from_str("
        ......
        ..#...
        ...#..
        .###..
        ......
        ......
    ");
    let start = g.clone();
    for gn in 0..4 {
        println!("gen {gn} pop {}\n{}", g.population(), g.render());
        g = g.step();
    }
    let mut steps = 0;
    let mut h = start.clone();
    loop {
        h = h.step();
        steps += 1;
        if h == start || steps > 50 { break; }
    }
    println!("period {steps}");
    let hist: Vec<usize> = (0..3).scan(start, |s, _| { *s = s.step(); Some(s.population()) }).collect();
    println!("{:?}", hist);
}
