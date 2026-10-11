struct Counter<'a> {
    count: &'a mut i32,
    log: &'a mut Vec<String>,
    step: i32,
}

impl<'a> Counter<'a> {
    fn new(count: &'a mut i32, log: &'a mut Vec<String>) -> Self {
        Counter { count, log, step: 2 }
    }
    fn bump(&mut self) {
        *self.count += self.step;
        self.log.push(format!("now {}", self.count));
    }
    fn current(&self) -> i32 {
        *self.count
    }
}

struct Window<'a> {
    data: &'a [i32],
}

impl<'a> Window<'a> {
    fn sum(&self) -> i32 {
        self.data.iter().sum()
    }
}

fn fill(target: &mut Vec<i32>, n: i32) {
    let mut w = Writer { out: target };
    for i in 0..n {
        w.write(i);
    }
}

struct Writer<'a> {
    out: &'a mut Vec<i32>,
}

impl Writer<'_> {
    fn write(&mut self, x: i32) {
        self.out.push(x * 10);
    }
}

fn main() {
    let mut n = 1;
    let mut log = Vec::new();
    {
        let mut c = Counter::new(&mut n, &mut log);
        c.bump();
        c.bump();
        println!("{}", c.current());
    }
    println!("{n} {:?}", log);
    let v = vec![1, 2, 3];
    let w = Window { data: &v[1..] };
    println!("{}", w.sum());
    let mut out = vec![];
    fill(&mut out, 3);
    println!("{:?}", out);
}
