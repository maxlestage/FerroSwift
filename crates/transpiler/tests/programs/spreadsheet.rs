use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Cell {
    Empty,
    Num(f64),
    Text(String),
    Formula(String),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Empty => f.pad(""),
            Cell::Num(n) => f.pad(&format!("{n}")),
            Cell::Text(s) => f.pad(s),
            Cell::Formula(src) => f.pad(&format!("={src}")),
        }
    }
}

struct Sheet {
    cells: BTreeMap<(u8, u8), Cell>,
    width: u8,
    height: u8,
}

impl Sheet {
    fn new(width: u8, height: u8) -> Sheet {
        Sheet { cells: BTreeMap::new(), width, height }
    }

    fn set(&mut self, col: char, row: u8, cell: Cell) -> Result<(), String> {
        let c = col.to_ascii_uppercase() as u8;
        if !(b'A'..b'A' + self.width).contains(&c) || row == 0 || row > self.height {
            return Err(format!("{col}{row} is outside the sheet"));
        }
        self.cells.insert((c - b'A', row - 1), cell);
        Ok(())
    }

    fn get(&self, col: u8, row: u8) -> &Cell {
        static EMPTY: Cell = Cell::Empty;
        self.cells.get(&(col, row)).unwrap_or(&EMPTY)
    }

    fn eval(&self, cell: &Cell, depth: u8) -> Result<f64, String> {
        match cell {
            Cell::Empty => Ok(0.0),
            Cell::Num(n) => Ok(*n),
            Cell::Text(t) => Err(format!("text {t:?} is not a number")),
            Cell::Formula(src) => {
                if depth > 8 {
                    return Err("too deep".into());
                }
                let (op, args) = src.split_once('(').ok_or("bad formula")?;
                let args = args.strip_suffix(')').ok_or("missing )")?;
                let values: Vec<f64> = args
                    .split(',')
                    .map(|a| self.resolve(a.trim(), depth + 1))
                    .collect::<Result<_, _>>()?;
                match op {
                    "SUM" => Ok(values.iter().sum()),
                    "MAX" => values.iter().cloned().reduce(f64::max).ok_or_else(|| "MAX of nothing".to_string()),
                    "AVG" if values.is_empty() => Err("AVG of nothing".into()),
                    "AVG" => Ok(values.iter().sum::<f64>() / values.len() as f64),
                    other => Err(format!("unknown function {other}")),
                }
            }
        }
    }

    fn resolve(&self, reference: &str, depth: u8) -> Result<f64, String> {
        let mut chars = reference.chars();
        let col = chars.next().ok_or("empty reference")?;
        let row: u8 = chars.as_str().parse().map_err(|_| format!("bad reference {reference:?}"))?;
        if !col.is_ascii_alphabetic() || row == 0 {
            return Err(format!("bad reference {reference:?}"));
        }
        self.eval(self.get(col.to_ascii_uppercase() as u8 - b'A', row - 1), depth)
    }

    fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("   ");
        for c in 0..self.width {
            out.push_str(&format!("|{:^9}", (b'A' + c) as char));
        }
        out.push('\n');
        for r in 0..self.height {
            out.push_str(&format!("{:>2} ", r + 1));
            for c in 0..self.width {
                let cell = self.get(c, r);
                let shown = match cell {
                    Cell::Formula(_) => match self.eval(cell, 0) {
                        Ok(v) => format!("{v:.2}"),
                        Err(_) => "#ERR".to_string(),
                    },
                    other => format!("{other}"),
                };
                out.push_str(&format!("|{shown:>9}"));
            }
            out.push('\n');
        }
        out
    }
}

fn main() {
    let mut sheet = Sheet::new(3, 4);
    let edits = [
        ('a', 1, Cell::Num(10.0)),
        ('a', 2, Cell::Num(32.5)),
        ('b', 1, Cell::Text("note".into())),
        ('a', 3, Cell::Formula("SUM(A1, A2)".into())),
        ('b', 3, Cell::Formula("AVG(A1,A2,A3)".into())),
        ('c', 1, Cell::Formula("MAX(A1, B1)".into())),
        ('c', 2, Cell::Formula("SUM(C2)".into())),
        ('c', 3, Cell::Formula("POW(A1)".into())),
        ('c', 4, Cell::Formula("SUM(Z9)".into())),
        ('d', 1, Cell::Num(1.0)),
        ('a', 9, Cell::Num(1.0)),
    ];
    for (col, row, cell) in edits {
        if let Err(e) = sheet.set(col, row, cell) {
            println!("error: {e}");
        }
    }
    print!("{}", sheet.render());
    for r in ["A3", "B3", "C1", "C2", "C3", "C4", "B1", "A", "a0", "B2"] {
        println!("{r} -> {:?}", sheet.resolve(r, 0));
    }
    println!("{} [{:>6}] [{:<6}] [{}]", sheet.cells.len(), Cell::Num(2.5), Cell::Text("ab".into()), Cell::Empty);
    println!("{:?}", sheet.cells.keys().map(|(c, r)| format!("{}{}", (b'A' + c) as char, r + 1)).collect::<Vec<_>>());
}
