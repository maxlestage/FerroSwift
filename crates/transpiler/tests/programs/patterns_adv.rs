#[derive(Debug, Clone)]
enum Shape { Circle { r: f64 }, Rect { w: f64, h: f64 }, Tri(f64, f64, f64), Group(Vec<Shape>) }

#[derive(Debug)]
enum Msg { Move { x: i32, y: i32 }, Write(String), Color(u8, u8, u8), Quit }

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

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } if *r <= 0.0 => 0.0,
        Shape::Circle { r } => 3.14159 * r * r,
        Shape::Rect { w, h } => w * h,
        &Shape::Tri(a, b, c) => {
            let p = (a + b + c) / 2.0;
            (p * (p - a) * (p - b) * (p - c)).sqrt()
        }
        Shape::Group(items) => items.iter().map(area).sum(),
    }
}

fn describe(m: &Msg) -> String {
    match m {
        Msg::Move { x: 0, y } => format!("vertical {y}"),
        Msg::Move { x, y: 0 } => format!("horizontal {x}"),
        Msg::Move { x, y } if x == y => format!("diagonal {x}"),
        Msg::Move { .. } => "move".to_string(),
        Msg::Write(s) if s.is_empty() => "empty".into(),
        Msg::Write(s) => format!("write {} chars", s.chars().count()),
        Msg::Color(r, _, b) if r == b => format!("balanced {r}"),
        Msg::Color(r @ 200..=255, g, b) => format!("reddish {r} {g} {b}"),
        Msg::Color(..) => "color".into(),
        Msg::Quit => "quit".into(),
    }
}

fn parse_kv(s: &str) -> Option<(String, i32)> {
    let Some((k, v)) = s.split_once('=') else { return None; };
    let Ok(n) = v.trim().parse::<i32>() else { return None; };
    Some((k.trim().to_string(), n))
}

fn first_big(v: &[i32]) -> Option<i32> {
    if let Some(&x) = v.first() && x > 10 {
        Some(x)
    } else if let [_, second, ..] = v && *second > 10 {
        Some(*second)
    } else {
        None
    }
}

fn main() {
    for n in [-5, 0, 7, 20, 44, 51] { print!("{} ", classify(n)); }
    println!();
    let shapes = vec![Shape::Circle { r: 1.0 }, Shape::Rect { w: 2.0, h: 3.0 }, Shape::Tri(3.0, 4.0, 5.0), Shape::Circle { r: -1.0 }, Shape::Group(vec![Shape::Rect { w: 1.0, h: 1.0 }, Shape::Circle { r: 2.0 }])];
    for s in &shapes { print!("{:.2} ", area(s)); }
    println!();
    let msgs = [Msg::Move { x: 0, y: 5 }, Msg::Move { x: 3, y: 0 }, Msg::Move { x: 2, y: 2 }, Msg::Move { x: 1, y: 9 }, Msg::Write(String::new()), Msg::Write("héllo".into()), Msg::Color(9, 0, 9), Msg::Color(230, 10, 20), Msg::Color(1, 2, 3), Msg::Quit];
    for m in &msgs { println!("{}", describe(m)); }
    println!("{:?} {:?} {:?}", parse_kv("a = 4"), parse_kv("b=x"), parse_kv("nothing"));
    println!("{:?} {:?} {:?}", first_big(&[12, 1]), first_big(&[1, 15, 3]), first_big(&[1]));
    let pairs = [(1, 'a'), (2, 'b'), (3, 'c')];
    for &(n, c) in pairs.iter().rev() { print!("{c}{n} "); }
    println!();
    let nested = Some(Some(3));
    if let Some(Some(x @ 1..=5)) = nested { println!("inner {x}"); }
    let tuple = (1, (2, [3, 4, 5]));
    let (a, (b, [c, rest @ ..])) = tuple;
    println!("{a} {b} {c} {:?}", rest);
}
