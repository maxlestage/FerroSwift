//! Conway's game of life on a small torus.

const W: usize = 8;
const H: usize = 6;

struct Grid {
    cells: Vec<Vec<bool>>,
}

impl Grid {
    fn new() -> Grid {
        Grid { cells: vec![vec![false; W]; H] }
    }

    fn set(&mut self, x: usize, y: usize) {
        self.cells[y][x] = true;
    }

    fn neighbours(&self, x: usize, y: usize) -> usize {
        let mut n = 0;
        for dy in [H - 1, 0, 1] {
            for dx in [W - 1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if self.cells[(y + dy) % H][(x + dx) % W] {
                    n += 1;
                }
            }
        }
        n
    }

    fn step(&self) -> Grid {
        let mut next = Grid::new();
        for y in 0..H {
            for x in 0..W {
                let alive = self.cells[y][x];
                let n = self.neighbours(x, y);
                next.cells[y][x] = matches!((alive, n), (true, 2) | (true, 3) | (false, 3));
            }
        }
        next
    }

    fn render(&self) -> String {
        self.cells
            .iter()
            .map(|row| row.iter().map(|&c| if c { '#' } else { '.' }).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn population(&self) -> usize {
        self.cells.iter().flatten().filter(|c| **c).count()
    }
}

fn main() {
    let mut grid = Grid::new();
    // A glider.
    for (x, y) in [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)] {
        grid.set(x, y);
    }
    for generation in 0..4 {
        println!("generation {} ({} alive)", generation, grid.population());
        println!("{}", grid.render());
        grid = grid.step();
    }
}
