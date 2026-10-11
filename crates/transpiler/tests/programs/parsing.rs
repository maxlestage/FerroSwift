use std::fmt::{self, Write as _};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
struct Node {
    key: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(key: i32) -> Self {
        Node { key, left: None, right: None }
    }
}

fn insert(root: &mut Option<Box<Node>>, key: i32) -> bool {
    match root {
        None => {
            *root = Some(Box::new(Node::new(key)));
            true
        }
        Some(n) if key < n.key => insert(&mut n.left, key),
        Some(n) if key > n.key => insert(&mut n.right, key),
        Some(_) => false,
    }
}

fn inorder(root: &Option<Box<Node>>, out: &mut Vec<i32>) {
    if let Some(n) = root {
        inorder(&n.left, out);
        out.push(n.key);
        inorder(&n.right, out);
    }
}

fn height(root: &Option<Box<Node>>) -> usize {
    match root {
        None => 0,
        Some(n) => 1 + height(&n.left).max(height(&n.right)),
    }
}

fn min_key(root: &Option<Box<Node>>) -> Option<i32> {
    let mut cur = root.as_ref()?;
    while let Some(l) = &cur.left {
        cur = l;
    }
    Some(cur.key)
}

#[derive(Debug, Clone, PartialEq)]
struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<Vec<f64>>,
}

impl Matrix {
    fn identity(n: usize) -> Self {
        let mut data = vec![vec![0.0; n]; n];
        for (i, row) in data.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        Matrix { rows: n, cols: n, data }
    }

    fn from_rows(data: Vec<Vec<f64>>) -> Result<Self, String> {
        let rows = data.len();
        let cols = data.first().map_or(0, |r| r.len());
        if data.iter().any(|r| r.len() != cols) {
            return Err("ragged rows".to_string());
        }
        Ok(Matrix { rows, cols, data })
    }

    fn mul(&self, other: &Matrix) -> Result<Matrix, String> {
        if self.cols != other.rows {
            return Err(format!("cannot multiply {}x{} by {}x{}", self.rows, self.cols, other.rows, other.cols));
        }
        let mut data = vec![vec![0.0; other.cols]; self.rows];
        for i in 0..self.rows {
            for j in 0..other.cols {
                data[i][j] = (0..self.cols).map(|k| self.data[i][k] * other.data[k][j]).sum();
            }
        }
        Ok(Matrix { rows: self.rows, cols: other.cols, data })
    }

    fn transpose(&self) -> Matrix {
        let data = (0..self.cols).map(|j| (0..self.rows).map(|i| self.data[i][j]).collect()).collect();
        Matrix { rows: self.cols, cols: self.rows, data }
    }

    fn trace(&self) -> f64 {
        (0..self.rows.min(self.cols)).map(|i| self.data[i][i]).sum()
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, row) in self.data.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "[")?;
            for (j, v) in row.iter().enumerate() {
                if j > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{v:>6.2}")?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Sym(char),
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a str) -> Self {
        Lexer { chars: s.chars().peekable() }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Tok;
    fn next(&mut self) -> Option<Tok> {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
        let c = *self.chars.peek()?;
        if c.is_ascii_digit() || c == '.' {
            let mut s = String::new();
            while let Some(&d) = self.chars.peek() {
                if d.is_ascii_digit() || d == '.' {
                    s.push(d);
                    self.chars.next();
                } else {
                    break;
                }
            }
            return Some(Tok::Num(s.parse().unwrap_or(f64::NAN)));
        }
        if c.is_alphabetic() {
            let mut s = String::new();
            while let Some(&d) = self.chars.peek() {
                if d.is_alphanumeric() {
                    s.push(d);
                    self.chars.next();
                } else {
                    break;
                }
            }
            return Some(Tok::Ident(s));
        }
        self.chars.next();
        Some(Tok::Sym(c))
    }
}

fn parse_csv(text: &str) -> Result<Vec<(String, f64, u32)>, String> {
    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let [name, price, qty] = fields[..] else {
            return Err(format!("line {}: expected 3 fields, got {}", lineno + 1, fields.len()));
        };
        let price: f64 = price.parse().map_err(|e| format!("line {}: price: {e}", lineno + 1))?;
        let qty: u32 = qty.parse().map_err(|e| format!("line {}: qty: {e}", lineno + 1))?;
        out.push((name.to_string(), price, qty));
    }
    Ok(out)
}

fn main() {
    let mut root = None;
    let inserted: Vec<bool> = [50, 30, 70, 20, 40, 60, 80, 30, 65].iter().map(|&k| insert(&mut root, k)).collect();
    let mut keys = Vec::new();
    inorder(&root, &mut keys);
    println!("{:?} {:?} {} {:?} {:?}", inserted, keys, height(&root), min_key(&root), min_key(&None));
    println!("{:?}", root.as_ref().map(|n| (n.key, n.left.as_ref().map(|l| l.key), n.right.as_ref().and_then(|r| r.left.as_ref()).map(|x| x.key))));
    let a = Matrix::from_rows(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
    let b = a.transpose();
    let c = a.mul(&b).unwrap();
    println!("{c}\n{} {} {:?}", c.trace(), c == c.transpose(), a.mul(&a).err());
    println!("{}", Matrix::identity(2).mul(&Matrix::from_rows(vec![vec![0.5, -1.25], vec![100.0, 3.14159]]).unwrap()).unwrap());
    println!("{:?} {:?}", Matrix::from_rows(vec![vec![1.0], vec![]]).err(), Matrix::from_rows(vec![]).map(|m| (m.rows, m.cols)));
    let toks: Vec<Tok> = Lexer::new("x1 = 3.5 * (y + .25)").collect();
    println!("{:?}", toks);
    println!("{} {:?}", Lexer::new("  ").count(), Lexer::new("a+").last());
    let mut lx = Lexer::new("12 ab 34");
    let first = lx.next();
    let nums: Vec<Tok> = lx.by_ref().filter(|t| matches!(t, Tok::Num(_))).collect();
    println!("{:?} {:?} {:?}", first, nums, lx.next());
    let csv = "name,price,qty\nwidget, 2.50, 10\n\n# comment\ngadget,10,3\n";
    println!("{:?}", parse_csv(csv));
    println!("{:?}", parse_csv("h\na,b\n"));
    println!("{:?}", parse_csv("h\na,2.x,1\n"));
    println!("{:?}", parse_csv("h\na,2,-1\n"));
    let total: f64 = parse_csv(csv).unwrap().iter().map(|(_, p, q)| p * *q as f64).sum();
    println!("{total:.2}");
    let mut counter = 0;
    let squares: Vec<i32> = std::iter::from_fn(|| { counter += 1; if counter <= 4 { Some(counter * counter) } else { None } }).collect();
    let mut seed = 7u32;
    let rolls: Vec<u32> = std::iter::repeat_with(|| { seed = seed.wrapping_mul(1103515245).wrapping_add(12345) % (1 << 31); seed % 6 + 1 }).take(5).collect();
    println!("{:?} {:?}", squares, rolls);
    let mut it = (1..=10).peekable();
    let mut groups = Vec::new();
    while let Some(x) = it.next() {
        let mut g = vec![x];
        while let Some(&n) = it.peek() {
            if n % 3 != 0 { g.push(n); it.next(); } else { break; }
        }
        groups.push(g);
    }
    println!("{:?}", groups);
    let mut report = String::new();
    for (name, price, qty) in parse_csv(csv).unwrap() {
        writeln!(report, "{name:<8}{price:>7.2}{qty:>4}").unwrap();
    }
    write!(report, "{:-<19}", "").unwrap();
    println!("{report}");
}
