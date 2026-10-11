use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Number(i64),
    Punct(char),
}

fn lex(src: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut Vec<Token>| {
        if word.is_empty() {
            return;
        }
        let t = match word.parse::<i64>() {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Word(word.clone()),
        };
        out.push(t);
        word.clear();
    };
    for c in src.chars() {
        if c.is_alphanumeric() || c == '-' && word.is_empty() {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            if !c.is_whitespace() {
                out.push(Token::Punct(c));
            }
        }
    }
    flush(&mut word, &mut out);
    out
}

struct Interp {
    vars: HashMap<String, i64>,
    output: Vec<String>,
}

impl Interp {
    fn run(&mut self, line: &str) -> Result<(), String> {
        let tokens = lex(line);
        match tokens.as_slice() {
            [Token::Word(cmd), rest @ ..] if cmd == "set" => match rest {
                [Token::Word(name), Token::Punct('='), value] => {
                    let v = self.value(value)?;
                    self.vars.insert(name.clone(), v);
                    Ok(())
                }
                _ => Err("usage: set NAME = VALUE".into()),
            },
            [Token::Word(cmd), args @ ..] if cmd == "add" || cmd == "mul" => {
                let mut acc = if cmd == "add" { 0 } else { 1 };
                for a in args {
                    let v = self.value(a)?;
                    acc = if cmd == "add" { acc + v } else { acc * v };
                }
                self.output.push(format!("{cmd} => {acc}"));
                Ok(())
            }
            [Token::Word(cmd), Token::Word(name)] if cmd == "del" => {
                self.vars.remove(name).map(|_| ()).ok_or_else(|| format!("no variable {name}"))
            }
            [Token::Word(cmd)] if cmd == "dump" => {
                let mut names: Vec<_> = self.vars.iter().collect();
                names.sort();
                self.output.push(names.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "));
                Ok(())
            }
            [] => Ok(()),
            [first, ..] => Err(format!("unknown command {first:?} ({} tokens)", tokens.len())),
        }
    }

    fn value(&self, t: &Token) -> Result<i64, String> {
        match t {
            Token::Number(n) => Ok(*n),
            Token::Word(name) => self.vars.get(name).copied().ok_or_else(|| format!("undefined {name}")),
            Token::Punct(c) => Err(format!("unexpected {c:?}")),
        }
    }
}

fn main() {
    println!("{:?}", lex("set x = -42; add x 1, y"));
    let mut it = Interp { vars: HashMap::new(), output: Vec::new() };
    let script = ["set x = 5", "set y = x", "add x y 10", "mul x y -1", "set z = w", "del y", "del y", "dump", "frob 1 2", "", "set a b c", "add x ;", "set n = -3", "mul n n n", "dump"];
    for line in script {
        match it.run(line) {
            Ok(()) => {}
            Err(e) => it.output.push(format!("error: {e}")),
        }
    }
    for (i, line) in it.output.iter().enumerate() {
        println!("{i:>2} {line}");
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for t in lex("a b a 1 2 1 , .") {
        let kind = match t {
            Token::Word(_) => "word",
            Token::Number(_) => "number",
            Token::Punct(_) => "punct",
        };
        *counts.entry(kind).or_default() += 1;
    }
    let mut kinds: Vec<_> = counts.into_iter().collect();
    kinds.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("{:?}", kinds);
}
