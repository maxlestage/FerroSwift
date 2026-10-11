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
            Json::Arr(items) => 1 + items.iter().map(|j| j.depth()).max().unwrap_or(0),
            Json::Obj(pairs) => 1 + pairs.iter().map(|(_, j)| j.depth()).max().unwrap_or(0),
            _ => 0,
        }
    }

    fn count(&self, pred: &dyn Fn(&Json) -> bool) -> usize {
        let own = usize::from(pred(self));
        own + match self {
            Json::Arr(items) => items.iter().map(|j| j.count(pred)).sum::<usize>(),
            Json::Obj(pairs) => pairs.iter().map(|(_, j)| j.count(pred)).sum(),
            _ => 0,
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Json::Null => write!(f, "null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Num(n) => write!(f, "{n}"),
            Json::Str(s) => write!(f, "{s:?}"),
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
                    write!(f, "{k:?}:{v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    src: &'a str,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser { chars: src.chars().collect(), pos: 0, src }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        self.skip_ws();
        if self.chars.get(self.pos) == Some(&c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected {c:?} at {} in {:?}", self.pos, self.src))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.chars.get(self.pos).copied() {
            None => Err("unexpected end".to_string()),
            Some('n') => self.literal("null", Json::Null),
            Some('t') => self.literal("true", Json::Bool(true)),
            Some('f') => self.literal("false", Json::Bool(false)),
            Some('"') => self.string().map(Json::Str),
            Some('[') => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    if self.chars.get(self.pos) == Some(&']') {
                        self.pos += 1;
                        break;
                    }
                    if !items.is_empty() {
                        self.expect(',')?;
                    }
                    items.push(self.value()?);
                }
                Ok(Json::Arr(items))
            }
            Some('{') => {
                self.pos += 1;
                let mut pairs = Vec::new();
                loop {
                    self.skip_ws();
                    if self.chars.get(self.pos) == Some(&'}') {
                        self.pos += 1;
                        break;
                    }
                    if !pairs.is_empty() {
                        self.expect(',')?;
                        self.skip_ws();
                    }
                    let key = self.string()?;
                    self.expect(':')?;
                    let v = self.value()?;
                    pairs.push((key, v));
                }
                Ok(Json::Obj(pairs))
            }
            Some(c) if c == '-' || c.is_ascii_digit() => {
                let start = self.pos;
                while self.pos < self.chars.len() && (self.chars[self.pos].is_ascii_digit() || matches!(self.chars[self.pos], '-' | '.' | 'e' | 'E' | '+')) {
                    self.pos += 1;
                }
                let text: String = self.chars[start..self.pos].iter().collect();
                text.parse::<f64>().map(Json::Num).map_err(|e| format!("{text}: {e}"))
            }
            Some(c) => Err(format!("unexpected {c:?} at {}", self.pos)),
        }
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        let end = self.pos + word.len();
        let got: String = self.chars.get(self.pos..end).map(|s| s.iter().collect()).unwrap_or_default();
        if got == word {
            self.pos = end;
            Ok(value)
        } else {
            Err(format!("bad literal {got:?}"))
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.chars.get(self.pos) {
                None => return Err("unterminated string".into()),
                Some('"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some('\\') => {
                    self.pos += 1;
                    let esc = self.chars.get(self.pos).copied().ok_or("bad escape")?;
                    out.push(match esc {
                        'n' => '\n',
                        't' => '\t',
                        other => other,
                    });
                    self.pos += 1;
                }
                Some(&c) => {
                    out.push(c);
                    self.pos += 1;
                }
            }
        }
    }
}

fn parse(src: &str) -> Result<Json, String> {
    let mut p = Parser::new(src);
    let v = p.value()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(format!("trailing data at {}", p.pos));
    }
    Ok(v)
}

fn main() {
    let src = r#" {"name": "Ferro\"Swift", "tags": ["a", "b", true, null], "n": -12.5e1, "nested": {"deep": [[1], [2, [3]]]}, "empty": {}} "#;
    match parse(src) {
        Ok(j) => {
            println!("{j}");
            println!("{:?}", j.get("name"));
            println!("{:?} {:?}", j.get("n").and_then(Json::as_f64), j.get("tags").and_then(|t| t.at(2)));
            println!("{} {} {}", j.depth(), j.count(&|x| matches!(x, Json::Num(_))), j.count(&|x| x == &Json::Null));
            println!("{:?}", j.get("nested").and_then(|n| n.get("deep")).and_then(|d| d.at(1)).and_then(|d| d.at(1)).map(|v| v.to_string()));
            println!("{}", j.get("missing").is_none() && j.at(0).is_none());
        }
        Err(e) => println!("error: {e}"),
    }
    for bad in ["[1, 2", "{\"a\" 1}", "tru", "[1,]", "\"abc", "1 2", "[-]", ""] {
        println!("{bad:?} -> {:?}", parse(bad).map(|j| j.to_string()));
    }
    let counts: HashMap<&str, usize> = ["x", "y", "x"].iter().fold(HashMap::new(), |mut m, k| {
        *m.entry(*k).or_insert(0) += 1;
        m
    });
    let mut keys: Vec<_> = counts.iter().collect();
    keys.sort();
    println!("{:?}", keys);
    let round_trip = parse(&parse("[1,[2,3],{\"k\":[]}]").unwrap().to_string()).unwrap();
    println!("{round_trip} {}", round_trip == Json::Arr(vec![Json::Num(1.0), Json::Arr(vec![Json::Num(2.0), Json::Num(3.0)]), Json::Obj(vec![("k".into(), Json::Arr(vec![]))])]));
}
