use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
}

fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n = text.parse::<f64>().map_err(|e| format!("{text}: {e}"))?;
                tokens.push(Token::Num(n));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                tokens.push(Token::Ident(chars[start..i].iter().collect()));
            }
            '+' | '-' | '*' | '/' | '^' => {
                tokens.push(Token::Op(c));
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            other => return Err(format!("unexpected {other:?} at {i}")),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    vars: HashMap<String, f64>,
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

    fn expr(&mut self) -> Result<f64, String> {
        let mut left = self.term()?;
        while let Some(Token::Op(op)) = self.peek().cloned() {
            if op != '+' && op != '-' {
                break;
            }
            self.next();
            let right = self.term()?;
            left = if op == '+' { left + right } else { left - right };
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut left = self.factor()?;
        loop {
            match self.peek() {
                Some(Token::Op('*')) => {
                    self.next();
                    left *= self.factor()?;
                }
                Some(Token::Op('/')) => {
                    self.next();
                    let d = self.factor()?;
                    if d == 0.0 {
                        return Err("division by zero".into());
                    }
                    left /= d;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<f64, String> {
        let base = self.atom()?;
        if let Some(Token::Op('^')) = self.peek() {
            self.next();
            let exp = self.factor()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn atom(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(n),
            Some(Token::Ident(name)) => self
                .vars
                .get(&name)
                .copied()
                .ok_or_else(|| format!("unknown variable {name}")),
            Some(Token::Op('-')) => Ok(-self.atom()?),
            Some(Token::LParen) => {
                let v = self.expr()?;
                match self.next() {
                    Some(Token::RParen) => Ok(v),
                    other => Err(format!("expected ), got {other:?}")),
                }
            }
            other => Err(format!("unexpected token {other:?}")),
        }
    }
}

fn eval(src: &str, vars: &HashMap<String, f64>) -> Result<f64, String> {
    let tokens = tokenize(src)?;
    let mut p = Parser { tokens, pos: 0, vars: vars.clone() };
    let v = p.expr()?;
    if p.pos < p.tokens.len() {
        return Err(format!("trailing input at {}", p.pos));
    }
    Ok(v)
}

fn main() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 3.0);
    vars.insert("pi".to_string(), std::f64::consts::PI);
    for src in ["1 + 2 * 3", "(1 + 2) * 3", "2 ^ 3 ^ 2", "-x * 2", "pi * x ^ 2", "1 / 0", "2 +", "y + 1", "1.2.3", "3 $ 4", "(1 + 2", "4 / 2 / 2"] {
        match eval(src, &vars) {
            Ok(v) => println!("{src} = {v}"),
            Err(e) => println!("{src} -> error: {e}"),
        }
    }
    println!("{:?}", tokenize("a1 + 2"));
    let text = "The quick brown fox";
    println!("{:?} {:?} {:?}", text.find("quick"), text.rfind('o'), text.find('z'));
    println!("{:?} {:?}", text.strip_prefix("The "), text.strip_suffix("cat"));
    println!("{} {}", text.trim_start_matches("The "), text.trim_end_matches('x'));
    let (head, tail) = text.split_at(3);
    println!("[{head}] [{tail}]");
    println!("{:?}", text.split_once(' '));
    println!("{:?}", text.rsplit_once(' '));
    println!("{}", text.chars().rev().collect::<String>());
    println!("{:?}", text.char_indices().filter(|(_, c)| c.is_uppercase()).collect::<Vec<_>>());
    println!("{} {}", "ab".repeat(3), "x".repeat(0).is_empty());
    println!("{:?}", "line one\nline two\n\nlast".lines().collect::<Vec<_>>());
    println!("{:?}", "a,b,,c".split(',').collect::<Vec<_>>());
    println!("{:?}", "a,b,,c".split(',').filter(|s| !s.is_empty()).count());
    println!("{:?}", "  padded  ".trim_start().len());
    println!("{}", &text[4..9]);
    println!("{:?} {:?}", text.bytes().nth(1), text.as_bytes()[0]);
    println!("{}", text.to_lowercase().split(' ').map(|w| w.len().to_string()).collect::<Vec<_>>().join("+"));
    let mut counts = [0usize; 26];
    for b in text.to_lowercase().bytes().filter(u8::is_ascii_lowercase) {
        counts[(b - b'a') as usize] += 1;
    }
    let most = counts.iter().enumerate().max_by_key(|(_, n)| **n).map(|(i, _)| (b'a' + i as u8) as char);
    println!("{:?}", most);
    println!("{}", text.chars().filter(|c| "aeiou".contains(*c)).count());
    println!("{:?}", text.matches('o').count());
    println!("{}", text.replace(' ', "_").to_uppercase());
    println!("{:?}", text.chars().map(|c| if c.is_alphabetic() { ((c.to_ascii_lowercase() as u8 - b'a' + 13) % 26 + b'a') as char } else { c }).collect::<String>().len());
    println!("{}", text.starts_with(char::is_uppercase));
    println!("{:?}", text.char_indices().nth(4));
    println!("{}", text.chars().all(|c| c.is_ascii()));
    println!("{:?}", "x=1;y=2".split(';').map(|kv| kv.split_once('=').unwrap()).collect::<Vec<_>>());
    println!("{}", format!("{:?}", &text[..3]));
    let word = String::from("héllo");
    println!("{} {} {:?}", word.len(), word.chars().count(), word.char_indices().map(|(i, _)| i).collect::<Vec<_>>());
    println!("{:?} {}", word.chars().nth(1), word.is_char_boundary(2));
    let mut s = String::new();
    for (i, w) in ["a", "b", "c"].iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s += w;
    }
    s.insert(0, '[');
    s.push(']');
    println!("{s} {}", s.capacity() >= s.len());
    s.truncate(3);
    println!("{s:?} {:?} {:?}", s.pop(), s.remove(0));
    s.clear();
    println!("{:?} {}", s, s.is_empty());
}
