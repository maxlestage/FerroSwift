use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Op(char),
    Ident(String),
    LParen,
    RParen,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Low = 1,
    Mid = 5,
    High = 10,
}

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Tri(f64, f64, f64),
    Empty,
}

impl Shape {
    fn name(&self) -> &'static str {
        match self {
            Shape::Circle { .. } => "circle",
            Shape::Rect { w, h } if (w - h).abs() < 1e-9 => "square",
            Shape::Rect { .. } => "rect",
            Shape::Tri(a, b, c) if a == b && b == c => "equilateral",
            Shape::Tri(..) => "triangle",
            Shape::Empty => "nothing",
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Num(n) => write!(f, "{n}"),
            Token::Op(c) => write!(f, "{c}"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}

fn classify(n: i64) -> &'static str {
    match n {
        i64::MIN..=-1 => "negative",
        0 => "zero",
        1..=9 => "digit",
        10 | 20 | 30 => "round",
        x if x % 2 == 0 => "even",
        _ => "odd",
    }
}

fn describe(pair: (i32, i32)) -> String {
    match pair {
        (0, 0) => "origin".to_string(),
        (x, 0) | (0, x) => format!("axis {x}"),
        (x, y) if x == y => format!("diagonal {x}"),
        (x, y) => format!("({x}, {y})"),
    }
}

fn head_tail(v: &[i32]) -> String {
    match v {
        [] => "empty".into(),
        [x] => format!("one {x}"),
        [first, .., last] if first == last => format!("bookends {first}"),
        [first, rest @ ..] => format!("{first} then {}", rest.len()),
    }
}

fn letter(c: char) -> &'static str {
    match c {
        'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
        'a'..='z' => "consonant",
        '0'..='9' => "digit",
        _ => "other",
    }
}

fn opt_pair(a: Option<i32>, b: Option<i32>) -> i32 {
    match (a, b) {
        (Some(x), Some(y)) => x + y,
        (Some(x), None) | (None, Some(x)) => x,
        (None, None) => 0,
    }
}

fn nested(v: Option<Result<i32, String>>) -> String {
    match v {
        Some(Ok(n @ 1..=5)) => format!("small {n}"),
        Some(Ok(n)) => format!("ok {n}"),
        Some(Err(e)) if e.is_empty() => "empty error".to_string(),
        Some(Err(e)) => format!("err {e}"),
        None => "none".to_string(),
    }
}

fn main() {
    let tokens = vec![Token::Num(1.5), Token::Op('+'), Token::Ident("x".into()), Token::LParen, Token::RParen, Token::Eof];
    for t in &tokens {
        let kind = match t {
            Token::Num(n) if *n > 1.0 => "big number",
            Token::Num(_) => "number",
            Token::Op('+') | Token::Op('-') => "additive",
            Token::Op(_) => "operator",
            Token::Ident(name) if name.len() == 1 => "short ident",
            Token::Ident(_) => "ident",
            Token::LParen | Token::RParen => "paren",
            Token::Eof => "end",
        };
        println!("{t} -> {kind} {:?}", t);
    }
    println!("{}", tokens.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" "));
    println!("{}", tokens.contains(&Token::Op('+')));
    println!("{}", tokens.iter().filter(|t| matches!(t, Token::Num(_) | Token::Ident(_))).count());
    if let Some(Token::Num(n)) = tokens.first() {
        println!("first number {n}");
    }
    let levels = [Level::High, Level::Low, Level::Mid];
    let mut sorted = levels.to_vec();
    sorted.sort();
    println!("{:?} {:?}", sorted, levels.iter().max());
    println!("{} {} {}", Level::Mid as i32, Level::Low < Level::High, Level::High as u8 + 1);
    println!("{:?}", levels.iter().map(|l| *l as i32).sum::<i32>());
    let shapes = vec![
        Shape::Circle { r: 1.0 },
        Shape::Rect { w: 2.0, h: 2.0 },
        Shape::Rect { w: 2.0, h: 3.0 },
        Shape::Tri(1.0, 1.0, 1.0),
        Shape::Tri(1.0, 2.0, 2.5),
        Shape::Empty,
    ];
    for s in &shapes {
        println!("{} {:?}", s.name(), s);
    }
    println!("{:?}", shapes.iter().filter(|s| matches!(s, Shape::Rect { w, .. } if *w > 1.0)).count());
    for n in [-5, 0, 7, 20, 44, 45] {
        print!("{} ", classify(n));
    }
    println!();
    for p in [(0, 0), (3, 0), (0, 4), (5, 5), (1, 2)] {
        print!("{} | ", describe(p));
    }
    println!();
    for v in [vec![], vec![1], vec![2, 5, 2], vec![1, 2, 3]] {
        print!("{} | ", head_tail(&v));
    }
    println!();
    for c in ['a', 'x', '5', '!'] {
        print!("{} ", letter(c));
    }
    println!();
    println!("{} {} {} {}", opt_pair(Some(1), Some(2)), opt_pair(Some(3), None), opt_pair(None, Some(4)), opt_pair(None, None));
    println!("{} {} {} {} {}", nested(Some(Ok(3))), nested(Some(Ok(30))), nested(Some(Err("".into()))), nested(Some(Err("bad".into()))), nested(None));
    let x = 7;
    let msg = match x {
        n if n < 0 => "neg",
        0 => "zero",
        _ => "pos",
    };
    println!("{msg}");
    let point = (3, -3);
    let (a, b) = point;
    let (mut c, d) = (a * 2, b * 2);
    c += 1;
    println!("{a} {b} {c} {d}");
    let ((e, f), g) = ((1, 2), 3);
    println!("{}", e + f + g);
    let mut p = (1, 2);
    p.0 += 10;
    p = (p.1, p.0);
    println!("{:?}", p);
    let values = [Some(1), None, Some(3)];
    let mut total = 0;
    for v in values.iter().flatten() {
        total += v;
    }
    println!("{total}");
    for (i, v) in values.iter().enumerate() {
        if let Some(n) = v {
            println!("{i}: {n}");
        } else {
            println!("{i}: -");
        }
    }
    let mut stack = vec![Token::Num(1.0), Token::Op('*')];
    while let Some(Token::Op(op)) = stack.last() {
        println!("op {op}");
        stack.pop();
    }
    println!("{}", stack.len());
    let grade = |score: u32| match score {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        _ => 'F',
    };
    println!("{}", [95, 85, 72, 10].iter().map(|&s| grade(s)).collect::<String>());
    let text = "hello";
    match text {
        "hi" | "hello" => println!("greeting"),
        _ => println!("other"),
    }
    match text.len() {
        0 => println!("empty"),
        1..=3 => println!("short"),
        n => println!("long {n}"),
    }
    let b = true;
    let r = match (b, x > 5) {
        (true, true) => 1,
        (true, false) => 2,
        (false, _) => 3,
    };
    println!("{r}");
    let shape = &shapes[1];
    if let Shape::Rect { w, h } = shape {
        println!("area {}", w * h);
    }
    let Shape::Circle { r } = &shapes[0] else {
        panic!("not a circle");
    };
    println!("radius {r}");
    let n = 42u8;
    let kind = match n {
        0 => "zero",
        1 | 2 | 3 => "few",
        4..=9 => "several",
        10.. => "many",
    };
    println!("{kind}");
    let ch = 'q';
    println!("{}", match ch { 'a'..='m' => 1, 'n'..='z' => 2, _ => 0 });
    let arr = [1, 2, 3];
    let [first, second, third] = arr;
    println!("{}", first + second * third);
    let t = Token::Ident("abc".to_string());
    if let Token::Ident(ref name) = t {
        println!("{name} {}", name.len());
    }
    println!("{:?}", t);
    let bools = [true, false, true];
    let count = bools.iter().filter(|&&b| b).count();
    println!("{count}");
    let o: Option<&str> = Some("x");
    println!("{}", match o { Some("x") => "exact", Some(_) => "some", None => "none" });
    let big = 1_000_000i64;
    println!("{}", match big { 0..=999 => "small", 1_000..=999_999 => "medium", _ => "large" });
}
