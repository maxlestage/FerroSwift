#[derive(Debug, Clone, Copy, PartialEq)]
struct Id(u32);

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Click { at: Point, button: u8 },
    Key(char),
    Scroll(i32, i32),
    Quit,
}

fn describe(o: Option<i32>) -> &'static str {
    match o {
        None => "nothing",
        Some(0) => "zero",
        Some(n) if n < 0 => "negative",
        Some(1 | 2 | 3) => "small",
        Some(n @ 4..=9) if n % 2 == 0 => "even digit",
        Some(_) => "other",
    }
}

fn nested(o: Option<Option<i32>>) -> i32 {
    match o {
        None => -1,
        Some(None) => 0,
        Some(Some(n)) => n,
    }
}

fn result_opt(r: Result<Option<i32>, String>) -> String {
    match r {
        Ok(None) => "ok none".into(),
        Ok(Some(n)) if n > 10 => format!("ok big {n}"),
        Ok(Some(n)) => format!("ok {n}"),
        Err(e) if e.is_empty() => "err empty".into(),
        Err(e) => format!("err {e}"),
    }
}

fn point_kind(p: &Point) -> &'static str {
    match p {
        Point { x: 0, y: 0 } => "origin",
        Point { x: 0, .. } => "on y axis",
        Point { y: 0, .. } => "on x axis",
        Point { x, y } if x == y => "diagonal",
        _ => "somewhere",
    }
}

fn event_text(e: &Event) -> String {
    match e {
        Event::Click { at: Point { x, y }, button: 1 } => format!("left click at {x},{y}"),
        Event::Click { button, .. } => format!("button {button}"),
        Event::Key('q') | Event::Quit => "quit".to_string(),
        Event::Key(c) if c.is_ascii_digit() => format!("digit {c}"),
        Event::Key(c) => format!("key {c}"),
        Event::Scroll(0, 0) => "no scroll".to_string(),
        Event::Scroll(dx, dy) => format!("scroll {dx} {dy}"),
    }
}

fn ids(v: &[Id]) -> String {
    match v {
        [] => "none".into(),
        [Id(0)] => "just zero".into(),
        [Id(a)] => format!("just {a}"),
        [Id(a), Id(b)] => format!("{a} and {b}"),
        [Id(first), .., Id(last)] => format!("{first}..{last}"),
    }
}

fn pair(a: Option<i32>, b: Option<i32>) -> i32 {
    match (a, b) {
        (None, None) => 0,
        (Some(x), None) | (None, Some(x)) => x,
        (Some(x), Some(y)) => x * y,
    }
}

fn main() {
    for o in [None, Some(0), Some(-5), Some(2), Some(6), Some(7), Some(42)] {
        print!("{} | ", describe(o));
    }
    println!();
    println!("{} {} {}", nested(None), nested(Some(None)), nested(Some(Some(7))));
    for r in [Ok(None), Ok(Some(3)), Ok(Some(30)), Err(String::new()), Err("bad".to_string())] {
        print!("{} | ", result_opt(r));
    }
    println!();
    for p in [Point { x: 0, y: 0 }, Point { x: 0, y: 2 }, Point { x: 3, y: 0 }, Point { x: 4, y: 4 }, Point { x: 1, y: 2 }] {
        print!("{} | ", point_kind(&p));
    }
    println!();
    let events = vec![
        Event::Click { at: Point { x: 1, y: 2 }, button: 1 },
        Event::Click { at: Point { x: 1, y: 2 }, button: 2 },
        Event::Key('q'),
        Event::Quit,
        Event::Key('7'),
        Event::Key('z'),
        Event::Scroll(0, 0),
        Event::Scroll(3, -1),
    ];
    for e in &events {
        println!("{} <- {:?}", event_text(e), e);
    }
    println!("{} | {} | {} | {} | {}", ids(&[]), ids(&[Id(0)]), ids(&[Id(4)]), ids(&[Id(1), Id(2)]), ids(&[Id(1), Id(2), Id(3)]));
    println!("{} {} {} {}", pair(None, None), pair(Some(3), None), pair(None, Some(4)), pair(Some(2), Some(5)));
    let x: Option<i32> = None;
    if let None = x {
        println!("x is none");
    }
    if let Some(ref v) = Some(String::from("ref")) {
        println!("{v} {}", v.len());
    }
    let maybe_id = Some(Id(9));
    if let Some(Id(n)) = maybe_id {
        println!("id {n}");
    }
    let Some(Id(m)) = maybe_id else {
        return;
    };
    println!("still {m}");
    let v = vec![Some(1), None, Some(3)];
    let mut found = 0;
    for item in &v {
        match item {
            None => continue,
            Some(n) => found += n,
        }
    }
    println!("{found}");
    let mut opt = Some(5);
    while let Some(n) = opt {
        opt = if n > 1 { Some(n - 2) } else { None };
        print!("{n} ");
    }
    println!();
    let t = (1, ("two", 3.0), Some('c'));
    let (a, (b, c), d) = t;
    println!("{a} {b} {c} {}", d.unwrap_or('-'));
    let (first, second) = match t {
        (n, _, Some(ch)) if n > 0 => (n, ch),
        _ => (0, '?'),
    };
    println!("{first} {second}");
    println!("{}", matches!(x, None) && !matches!(maybe_id, None));
    println!("{}", match maybe_id { Some(Id(0)) => "zero", Some(Id(_)) => "some", None => "none" });
    let arr = [Some(1), Some(2)];
    if let [Some(a), Some(b)] = arr {
        println!("{}", a + b);
    }
    let refs: Vec<&Option<i32>> = v.iter().collect();
    let count = refs.iter().filter(|o| matches!(o, Some(n) if *n > 1)).count();
    println!("{count}");
}
