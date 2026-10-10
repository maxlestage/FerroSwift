use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Money {
    cents: i64,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.cents < 0 { "-" } else { "" };
        write!(f, "{sign}${}.{:02}", self.cents.abs() / 100, self.cents.abs() % 100)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Cell {
    Empty,
    Num(i64),
    Text(String),
    Flag(bool),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, "-"),
            Cell::Num(n) => write!(f, "{n}"),
            Cell::Text(s) => write!(f, "{s:?}"),
            Cell::Flag(b) => write!(f, "{}", if *b { "yes" } else { "no" }),
        }
    }
}

#[derive(Debug)]
struct Row {
    cells: Vec<Cell>,
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, c) in self.cells.iter().enumerate() {
            if i > 0 {
                write!(f, " | ")?;
            }
            write!(f, "{c:>6}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Stats {
    count: usize,
    mean: f64,
    min: Option<f64>,
    label: String,
}

struct Temperature(f64);

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}°C", self.0)
    }
}

#[derive(Debug)]
struct Tree {
    value: i32,
    children: Vec<Tree>,
}

impl Tree {
    fn leaf(v: i32) -> Tree {
        Tree { value: v, children: vec![] }
    }
    fn sum(&self) -> i32 {
        self.value + self.children.iter().map(|c| c.sum()).sum::<i32>()
    }
    fn depth(&self) -> usize {
        1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
    }
    fn render(&self, indent: usize, out: &mut String) {
        out.push_str(&format!("{}{}\n", "  ".repeat(indent), self.value));
        for c in &self.children {
            c.render(indent + 1, out);
        }
    }
}

fn main() {
    let prices = [Money { cents: 1999 }, Money { cents: -250 }, Money { cents: 5 }];
    for p in &prices {
        println!("[{p}] [{p:>10}] [{p:<10}] [{p:^10}]");
    }
    println!("{:?}", prices[0]);
    let total: i64 = prices.iter().map(|m| m.cents).sum();
    println!("{}", Money { cents: total });
    let row = Row { cells: vec![Cell::Empty, Cell::Num(42), Cell::Text("hi".into()), Cell::Flag(true)] };
    println!("{row}");
    println!("{:?}", row);
    println!("{:#?}", row.cells[2]);
    let s = Stats { count: 3, mean: 2.5, min: Some(1.0), label: "demo".into() };
    println!("{:?}", s);
    println!("{:#?}", s);
    println!("{:?}", Stats::default());
    let t = Temperature(21.456);
    println!("{t} [{t:>10}]");
    let tree = Tree {
        value: 1,
        children: vec![Tree { value: 2, children: vec![Tree::leaf(4), Tree::leaf(5)] }, Tree::leaf(3)],
    };
    println!("{} {}", tree.sum(), tree.depth());
    let mut out = String::new();
    tree.render(0, &mut out);
    print!("{out}");
    println!("{:?}", Tree::leaf(9));
    println!("{:#?}", Tree { value: 1, children: vec![Tree::leaf(2)] });
    let items = vec![("apple", 3, 0.5), ("kiwi", 12, 1.25), ("watermelon", 1, 3.0)];
    println!("{:<12}|{:>4}|{:>8}", "item", "qty", "price");
    for (name, qty, price) in &items {
        println!("{:<12}|{:>4}|{:>8.2}", name, qty, price);
    }
    println!("{:-<26}", "");
    let total: f64 = items.iter().map(|(_, q, p)| *q as f64 * p).sum();
    println!("{:<12}|{:>4}|{:>8.2}", "total", items.iter().map(|i| i.1).sum::<i32>(), total);
    println!("{:?} {:?}", Cell::Text("a\"b".into()), Cell::Flag(false));
    println!("{}", Cell::Text("q".into()));
    let cells: Vec<String> = row.cells.iter().map(|c| c.to_string()).collect();
    println!("{}", cells.join(","));
    let width = 7;
    println!("[{:>width$}] [{:<width$}] [{:^width$.2}]", "ab", 1.5, 2.0f64 / 3.0);
    println!("[{:>1$}] [{:.*}]", "x", 4, 1.23456);
    println!("{:+} {:+} {:+.2}", 5, -5, 0.0);
    println!("{:05} {:<05} {:^5}|", -42, 7, 7);
    println!("{:#010x} {:#b} {:o}", 255, 10, 64);
    println!("{:e} {:E} {:.2e}", 123456.0, 0.00042, 9876.54321);
    println!("{}", format!("{:>3}", 'c'));
    println!("{}", format!("{:>5}", true));
    println!("{:?} {:?}", "multi\nline", 'x');
    println!("{:?}", vec![Some("a"), None]);
    println!("{:?}", (Money { cents: 1 }, Cell::Empty, [1u8, 2], "s", 2.5f32));
    println!("{:?}", vec![vec![1.5, 2.0], vec![]]);
    println!("{:?} {:?}", String::from("own"), &"borrow");
    println!("{:?}", std::collections::BTreeMap::from([(1, vec!['a']), (2, vec![])]));
    println!("{:?}", ((), ((2, 3), 4)));
    println!("{:?} {:?} {:?}", 1u8, -1i8, 1.0e10f64);
    println!("{:?} {:?}", i64::MAX, f64::MIN_POSITIVE);
    println!("{:10?}|{:<10?}|", 1, 2.5);
    println!("{:?}", Some(Money { cents: 100 }));
    println!("{:#?}", (1, "a"));
    println!("{:#?}", vec![Cell::Num(1), Cell::Empty]);
    println!("{:#?}", Some(Stats::default()));
}
