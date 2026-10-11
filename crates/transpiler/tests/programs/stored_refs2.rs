#[derive(Debug, Default)]
struct Stats {
    hits: u32,
    names: Vec<String>,
}

struct Recorder<'a> {
    stats: &'a mut Stats,
    grid: &'a mut Vec<Vec<i32>>,
    total: &'a mut i64,
}

impl<'a> Recorder<'a> {
    fn hit(&mut self, name: &str, r: usize, c: usize) {
        self.stats.hits += 1;
        self.stats.names.push(name.to_uppercase());
        self.grid[r][c] += 1;
        *self.total += (r * 10 + c) as i64;
        bump(self.total);
    }

    fn summary(&self) -> String {
        format!("{} hits, {} rows, total {}", self.stats.hits, self.grid.len(), self.total)
    }
}

fn bump(x: &mut i64) {
    *x += 1000;
}

struct Cursor<'a> {
    buf: &'a mut Vec<u8>,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a mut Vec<u8>) -> Self {
        Cursor { buf, pos: 0 }
    }
    fn write(&mut self, b: u8) {
        if self.pos < self.buf.len() {
            self.buf[self.pos] = b;
        } else {
            self.buf.push(b);
        }
        self.pos += 1;
    }
}

fn main() {
    let mut stats = Stats::default();
    let mut grid = vec![vec![0; 3]; 2];
    let mut total = 0i64;
    let mut rec = Recorder { stats: &mut stats, grid: &mut grid, total: &mut total };
    rec.hit("a", 0, 1);
    rec.hit("b", 1, 2);
    println!("{}", rec.summary());
    println!("{:?} {:?} {}", stats, grid, total);
    let mut buf = vec![9, 9];
    {
        let mut cur = Cursor::new(&mut buf);
        for b in [1u8, 2, 3] {
            cur.write(b);
        }
        println!("{}", cur.pos);
    }
    println!("{:?}", buf);
}
