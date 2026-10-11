use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn at(&self, i: usize) -> Option<&Json> {
        match self {
            Json::Arr(items) => items.get(i),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        if let Json::Num(n) = self { Some(*n) } else { None }
    }

    fn depth(&self) -> usize {
        match self {
            Json::Arr(items) => 1 + items.iter().map(Json::depth).max().unwrap_or(0),
            Json::Obj(pairs) => 1 + pairs.iter().map(|(_, v)| v.depth()).max().unwrap_or(0),
            _ => 0,
        }
    }

    fn count_leaves(&self) -> usize {
        match self {
            Json::Arr(items) => items.iter().map(|j| j.count_leaves()).sum(),
            Json::Obj(pairs) => pairs.iter().map(|(_, v)| v.count_leaves()).sum(),
            _ => 1,
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Null => write!(f, "null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Num(n) => write!(f, "{n}"),
            Json::Str(s) => write!(f, "\"{}\"", s.replace('"', "\\\"")),
            Json::Arr(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Json::Obj(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{k}\":{v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Parser { chars: text.chars().peekable() }
    }

    fn skip_ws(&mut self) {
        while matches!(self.chars.peek(), Some(c) if c.is_whitespace()) {
            self.chars.next();
        }
    }

    fn expect(&mut self, want: char) -> Result<(), String> {
        self.skip_ws();
        match self.chars.next() {
            Some(c) if c == want => Ok(()),
            Some(c) => Err(format!("expected '{want}', found '{c}'")),
            None => Err(format!("expected '{want}', found end")),
        }
    }

    fn parse(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.chars.peek().copied() {
            None => Err("unexpected end".to_string()),
            Some('n') => self.literal("null", Json::Null),
            Some('t') => self.literal("true", Json::Bool(true)),
            Some('f') => self.literal("false", Json::Bool(false)),
            Some('"') => self.string().map(Json::Str),
            Some('[') => {
                self.chars.next();
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    if self.chars.peek() == Some(&']') {
                        self.chars.next();
                        break;
                    }
                    if !items.is_empty() {
                        self.expect(',')?;
                    }
                    items.push(self.parse()?);
                }
                Ok(Json::Arr(items))
            }
            Some('{') => {
                self.chars.next();
                let mut pairs = Vec::new();
                loop {
                    self.skip_ws();
                    if self.chars.peek() == Some(&'}') {
                        self.chars.next();
                        break;
                    }
                    if !pairs.is_empty() {
                        self.expect(',')?;
                        self.skip_ws();
                    }
                    let key = self.string()?;
                    self.expect(':')?;
                    let value = self.parse()?;
                    pairs.push((key, value));
                }
                Ok(Json::Obj(pairs))
            }
            Some(c) if c == '-' || c.is_ascii_digit() => {
                let mut text = String::new();
                while let Some(&c) = self.chars.peek() {
                    if c == '-' || c == '.' || c == 'e' || c == '+' || c.is_ascii_digit() {
                        text.push(c);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                text.parse::<f64>().map(Json::Num).map_err(|e| format!("bad number '{text}': {e}"))
            }
            Some(c) => Err(format!("unexpected '{c}'")),
        }
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        for expected in word.chars() {
            match self.chars.next() {
                Some(c) if c == expected => {}
                _ => return Err(format!("bad literal, wanted {word}")),
            }
        }
        Ok(value)
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.chars.next() {
                None => return Err("unterminated string".into()),
                Some('"') => return Ok(out),
                Some('\\') => match self.chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(c) => out.push(c),
                    None => return Err("bad escape".into()),
                },
                Some(c) => out.push(c),
            }
        }
    }
}

fn main() {
    let inputs = [
        r#"{"name": "Ada", "age": 36, "tags": ["x", "y"], "nested": {"ok": true, "n": null}}"#,
        "[1, 2.5, -3e2, [], {}]",
        r#""esc\"aped\n""#,
        "[1, 2",
        "{\"a\" 1}",
        "nul",
        "  true  ",
        "-",
    ];
    for input in inputs {
        let mut p = Parser::new(input);
        match p.parse() {
            Ok(j) => println!("ok {} depth {} leaves {}", j, j.depth(), j.count_leaves()),
            Err(e) => println!("err {e}"),
        }
    }
    let doc = Parser::new(r#"{"items": [{"id": 1, "price": 2.5}, {"id": 2, "price": 4}], "total": 6.5}"#).parse().unwrap();
    let total: f64 = (0..2).filter_map(|i| doc.get("items")?.at(i)?.get("price")?.as_f64()).sum();
    println!("{} {:?} {:?}", total, doc.get("total").and_then(Json::as_f64), doc.get("missing"));
    println!("{:?}", doc.get("items").and_then(|it| it.at(1)).and_then(|o| o.get("id")));
    let mut counts: HashMap<&str, usize> = HashMap::new();
    fn walk<'a>(j: &'a Json, counts: &mut HashMap<&'a str, usize>) {
        let kind = match j {
            Json::Null => "null",
            Json::Bool(_) => "bool",
            Json::Num(_) => "num",
            Json::Str(_) => "str",
            Json::Arr(items) => {
                for i in items {
                    walk(i, counts);
                }
                "arr"
            }
            Json::Obj(pairs) => {
                for (_, v) in pairs {
                    walk(v, counts);
                }
                "obj"
            }
        };
        *counts.entry(kind).or_insert(0) += 1;
    }
    walk(&doc, &mut counts);
    let mut kinds: Vec<_> = counts.into_iter().collect();
    kinds.sort();
    println!("{:?}", kinds);
    println!("{}", Json::Arr(vec![Json::Str("a\"b".into()), Json::Bool(false), Json::Null]) == Parser::new(r#"["a\"b", false, null]"#).parse().unwrap());
}
