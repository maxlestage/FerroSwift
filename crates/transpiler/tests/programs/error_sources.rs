use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum CalcError {
    Parse(ParseIntError),
    DivZero { at: usize },
    Unknown(char),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::Parse(e) => write!(f, "bad number: {e}"),
            CalcError::DivZero { at } => write!(f, "division by zero at {at}"),
            CalcError::Unknown(c) => write!(f, "unknown operator '{c}'"),
        }
    }
}

impl Error for CalcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CalcError::Parse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ParseIntError> for CalcError {
    fn from(e: ParseIntError) -> Self {
        CalcError::Parse(e)
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Num(i64),
    Bin(Box<Expr>, char, Box<Expr>),
}

fn parse(tokens: &[&str]) -> Result<Expr, CalcError> {
    let mut expr = Expr::Num(tokens[0].parse()?);
    let mut i = 1;
    while i + 1 < tokens.len() {
        let op = tokens[i].chars().next().unwrap();
        if !"+-*/".contains(op) {
            return Err(CalcError::Unknown(op));
        }
        let rhs = Expr::Num(tokens[i + 1].parse()?);
        expr = Expr::Bin(Box::new(expr), op, Box::new(rhs));
        i += 2;
    }
    Ok(expr)
}

fn eval(e: &Expr, pos: &mut usize) -> Result<i64, CalcError> {
    match e {
        Expr::Num(n) => Ok(*n),
        Expr::Bin(l, op, r) => {
            let a = eval(l, pos)?;
            *pos += 1;
            let b = eval(r, pos)?;
            Ok(match op {
                '+' => a + b,
                '-' => a - b,
                '*' => a * b,
                _ => {
                    if b == 0 {
                        return Err(CalcError::DivZero { at: *pos });
                    }
                    a / b
                }
            })
        }
    }
}

fn run(src: &str) -> Result<i64, Box<dyn Error>> {
    let tokens: Vec<&str> = src.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("empty input".into());
    }
    let e = parse(&tokens)?;
    let mut pos = 0;
    Ok(eval(&e, &mut pos)?)
}

fn main() {
    for src in ["1 + 2 * 3", "10 / 0", "4 % 2", "x + 1", "", "7 - 10 * 2"] {
        match run(src) {
            Ok(v) => println!("{src:>10} = {v}"),
            Err(e) => {
                print!("{src:>10} ! {e}");
                if let Some(s) = e.source() {
                    print!(" (caused by: {s})");
                }
                println!();
            }
        }
    }
    let e = parse(&["5", "*", "6"]).unwrap();
    println!("{:?}", e);
    let err = parse(&["q"]).unwrap_err();
    println!("{:?} / {}", err, err);
}
