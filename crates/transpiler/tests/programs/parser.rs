use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
}

fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut it = src.chars().peekable();
    while let Some(&c) = it.peek() {
        match c {
            ' ' | '\t' => {
                it.next();
            }
            '0'..='9' | '.' => {
                let mut text = String::new();
                while let Some(&d) = it.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        text.push(d);
                        it.next();
                    } else {
                        break;
                    }
                }
                let n: f64 = text.parse().map_err(|_| format!("bad number {text:?}"))?;
                out.push(Token::Num(n));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut name = String::new();
                while let Some(&d) = it.peek() {
                    if d.is_alphanumeric() || d == '_' {
                        name.push(d);
                        it.next();
                    } else {
                        break;
                    }
                }
                out.push(Token::Ident(name));
            }
            '+' | '-' | '*' | '/' | '^' => {
                out.push(Token::Op(c));
                it.next();
            }
            '(' => {
                out.push(Token::LParen);
                it.next();
            }
            ')' => {
                out.push(Token::RParen);
                it.next();
            }
            other => return Err(format!("unexpected {other:?}")),
        }
    }
    Ok(out)
}

#[derive(Debug)]
enum Expr {
    Num(f64),
    Var(String),
    Neg(Box<Expr>),
    Bin(Box<Expr>, char, Box<Expr>),
    Call(String, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Num(n) => write!(f, "{n}"),
            Expr::Var(v) => write!(f, "{v}"),
            Expr::Neg(e) => write!(f, "(-{e})"),
            Expr::Bin(a, op, b) => write!(f, "({a} {op} {b})"),
            Expr::Call(name, arg) => write!(f, "{name}({arg})"),
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expr(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut lhs = self.atom()?;
        loop {
            let op = match self.peek() {
                Some(Token::Op(op)) => *op,
                _ => break,
            };
            let (prec, right_assoc) = match op {
                '+' | '-' => (1, false),
                '*' | '/' => (2, false),
                '^' => (3, true),
                _ => return Err(format!("unknown operator {op}")),
            };
            if prec < min_prec {
                break;
            }
            self.next();
            let rhs = self.expr(if right_assoc { prec } else { prec + 1 })?;
            lhs = Expr::Bin(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn atom(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(Expr::Num(n)),
            Some(Token::Ident(name)) => {
                if self.peek() == Some(&Token::LParen) {
                    self.next();
                    let arg = self.expr(0)?;
                    match self.next() {
                        Some(Token::RParen) => Ok(Expr::Call(name, Box::new(arg))),
                        _ => Err("expected )".to_string()),
                    }
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Some(Token::Op('-')) => Ok(Expr::Neg(Box::new(self.expr(3)?))),
            Some(Token::LParen) => {
                let e = self.expr(0)?;
                match self.next() {
                    Some(Token::RParen) => Ok(e),
                    other => Err(format!("expected ), got {other:?}")),
                }
            }
            other => Err(format!("unexpected token {other:?}")),
        }
    }
}

fn eval(e: &Expr, x: f64) -> Result<f64, String> {
    Ok(match e {
        Expr::Num(n) => *n,
        Expr::Var(v) if v == "x" => x,
        Expr::Var(v) if v == "pi" => std::f64::consts::PI,
        Expr::Var(v) => return Err(format!("unknown variable {v}")),
        Expr::Neg(inner) => -eval(inner, x)?,
        Expr::Bin(a, op, b) => {
            let (l, r) = (eval(a, x)?, eval(b, x)?);
            match op {
                '+' => l + r,
                '-' => l - r,
                '*' => l * r,
                '/' if r == 0.0 => return Err("division by zero".into()),
                '/' => l / r,
                '^' => l.powf(r),
                _ => unreachable!(),
            }
        }
        Expr::Call(name, arg) => {
            let v = eval(arg, x)?;
            match name.as_str() {
                "sqrt" => v.sqrt(),
                "abs" => v.abs(),
                "sq" => v * v,
                _ => return Err(format!("unknown function {name}")),
            }
        }
    })
}

fn run(src: &str, x: f64) -> Result<(String, f64), String> {
    let tokens = tokenize(src)?;
    let mut p = Parser { tokens, pos: 0 };
    let e = p.expr(0)?;
    if p.pos < p.tokens.len() {
        return Err(format!("trailing tokens from {}", p.pos));
    }
    let v = eval(&e, x)?;
    Ok((e.to_string(), v))
}

fn main() {
    let cases = ["1 + 2 * 3", "(1 + 2) * 3", "2 ^ 3 ^ 2", "-x ^ 2 + sqrt(16)", "sq(x) / (x - 2)", "abs(-7.5) - pi", "2 *", "1 + $", "foo(1)", "y + 1", "3 / (x - x)", "1.2.3 + 1", "((4)"];
    for src in cases {
        match run(src, 2.0) {
            Ok((text, v)) => println!("{src:<20} => {text} = {v:.4}"),
            Err(e) => println!("{src:<20} => error: {e}"),
        }
    }
    println!("{:?}", tokenize("a1 + 2.5*(b)").unwrap());
    println!("{:?}", tokenize("1 + 2").unwrap().iter().filter(|t| matches!(t, Token::Num(_))).count());
}
