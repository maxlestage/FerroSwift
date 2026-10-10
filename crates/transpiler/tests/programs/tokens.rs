//! A tokenizer and a small calculator: iterators kept in variables, `peekable`, `next_if`.

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Op(char),
    LParen,
    RParen,
}

fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' => continue,
            '0'..='9' | '.' => {
                let mut text = c.to_string();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        text.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let n: f64 = text.parse().map_err(|_| format!("bad number {text}"))?;
                tokens.push(Token::Num(n));
            }
            '+' | '-' | '*' | '/' => tokens.push(Token::Op(c)),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            other => return Err(format!("unexpected '{other}'")),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.peek();
        self.pos += 1;
        t
    }

    fn expr(&mut self) -> f64 {
        let mut value = self.term();
        while let Some(Token::Op(op)) = self.peek() {
            if op != '+' && op != '-' {
                break;
            }
            self.next();
            let rhs = self.term();
            value = if op == '+' { value + rhs } else { value - rhs };
        }
        value
    }

    fn term(&mut self) -> f64 {
        let mut value = self.atom();
        while let Some(Token::Op(op)) = self.peek() {
            if op != '*' && op != '/' {
                break;
            }
            self.next();
            let rhs = self.atom();
            value = if op == '*' { value * rhs } else { value / rhs };
        }
        value
    }

    fn atom(&mut self) -> f64 {
        match self.next() {
            Some(Token::Num(n)) => n,
            Some(Token::Op('-')) => -self.atom(),
            Some(Token::LParen) => {
                let v = self.expr();
                self.next();
                v
            }
            _ => f64::NAN,
        }
    }
}

fn main() {
    for src in ["1 + 2 * 3", "(1 + 2) * 3", "10 / 4 - -1", "2 * (3 + 4) * 5", "1 + x", "3.5.1 + 1"] {
        match tokenize(src) {
            Ok(tokens) => {
                let mut p = Parser { tokens, pos: 0 };
                println!("{src} = {}", p.expr());
            }
            Err(e) => println!("{src}: error: {e}"),
        }
    }
    let mut words = "one two three four".split(' ');
    let first = words.next().unwrap_or("");
    let second = words.next().unwrap_or("");
    let rest: Vec<&str> = words.collect();
    println!("{first} | {second} | {:?}", rest);
}
