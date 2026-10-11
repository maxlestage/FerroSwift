use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Token { Num(f64), Ident(String), Op(char), LParen, RParen }

struct Lexer<'a> { chars: std::iter::Peekable<std::str::Chars<'a>> }

impl<'a> Lexer<'a> {
    fn new(s: &'a str) -> Self { Lexer { chars: s.chars().peekable() } }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, String>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() { self.chars.next(); } else { break; }
        }
        let c = *self.chars.peek()?;
        Some(if c.is_ascii_digit() || c == '.' {
            let mut s = String::new();
            while let Some(&d) = self.chars.peek() {
                if d.is_ascii_digit() || d == '.' { s.push(d); self.chars.next(); } else { break; }
            }
            s.parse().map(Token::Num).map_err(|_| format!("bad number {s}"))
        } else if c.is_alphabetic() {
            let mut s = String::new();
            while let Some(&d) = self.chars.peek() {
                if d.is_alphanumeric() || d == '_' { s.push(d); self.chars.next(); } else { break; }
            }
            Ok(Token::Ident(s))
        } else {
            self.chars.next();
            match c {
                '(' => Ok(Token::LParen),
                ')' => Ok(Token::RParen),
                '+' | '-' | '*' | '/' | '^' => Ok(Token::Op(c)),
                _ => Err(format!("unexpected '{c}'")),
            }
        })
    }
}

#[derive(Debug)]
enum Ast { Num(f64), Var(String), Bin(Box<Ast>, char, Box<Ast>), Neg(Box<Ast>) }

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ast::Num(n) => write!(f, "{n}"),
            Ast::Var(v) => write!(f, "{v}"),
            Ast::Bin(a, op, b) => write!(f, "({a} {op} {b})"),
            Ast::Neg(a) => write!(f, "-{a}"),
        }
    }
}

struct Parser { toks: Vec<Token>, pos: usize }

impl Parser {
    fn peek(&self) -> Option<&Token> { self.toks.get(self.pos) }
    fn bump(&mut self) -> Option<Token> { let t = self.toks.get(self.pos).cloned(); self.pos += 1; t }
    fn prec(op: char) -> u8 { match op { '+' | '-' => 1, '*' | '/' => 2, '^' => 3, _ => 0 } }
    fn expr(&mut self, min: u8) -> Result<Ast, String> {
        let mut lhs = match self.bump().ok_or("unexpected end")? {
            Token::Num(n) => Ast::Num(n),
            Token::Ident(v) => Ast::Var(v),
            Token::Op('-') => Ast::Neg(Box::new(self.expr(3)?)),
            Token::LParen => {
                let e = self.expr(0)?;
                match self.bump() { Some(Token::RParen) => e, other => return Err(format!("expected ) got {other:?}")) }
            }
            t => return Err(format!("unexpected {t:?}")),
        };
        while let Some(Token::Op(op)) = self.peek().cloned() {
            let p = Self::prec(op);
            if p < min { break; }
            self.bump();
            let next = if op == '^' { p } else { p + 1 };
            let rhs = self.expr(next)?;
            lhs = Ast::Bin(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }
}

fn eval(a: &Ast, env: &HashMap<&str, f64>) -> Result<f64, String> {
    Ok(match a {
        Ast::Num(n) => *n,
        Ast::Var(v) => *env.get(v.as_str()).ok_or_else(|| format!("unbound {v}"))?,
        Ast::Neg(x) => -eval(x, env)?,
        Ast::Bin(l, op, r) => {
            let (x, y) = (eval(l, env)?, eval(r, env)?);
            match op { '+' => x + y, '-' => x - y, '*' => x * y, '/' => x / y, _ => x.powf(y) }
        }
    })
}

fn main() {
    let env: HashMap<&str, f64> = [("x", 2.0), ("pi", 3.14159)].into_iter().collect();
    for src in ["1 + 2 * 3", "(1 + 2) * 3", "2 ^ 3 ^ 2", "-x * pi", "x / 0", "y + 1", "3 $ 4", "(1 + 2", "1.5.2 + 1"] {
        let toks: Result<Vec<Token>, String> = Lexer::new(src).collect();
        let result = toks.and_then(|t| Parser { toks: t, pos: 0 }.expr(0)).and_then(|ast| {
            let v = eval(&ast, &env)?;
            Ok(format!("{ast} = {v:.3}"))
        });
        match result {
            Ok(s) => println!("{src:<12} -> {s}"),
            Err(e) => println!("{src:<12} !! {e}"),
        }
    }
}
