#[derive(Debug, Clone, Copy, PartialEq)]
enum Suit {
    Hearts,
    Spades,
}

#[derive(Debug, Clone, PartialEq)]
enum Card {
    Number(u8, Suit),
    Face { name: String, suit: Suit },
    Joker,
}

#[derive(Debug)]
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

fn eval(e: &Expr) -> f64 {
    match e {
        Expr::Num(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
        Expr::Neg(a) => -eval(a),
    }
}

fn describe(card: &Card) -> String {
    match card {
        Card::Number(1, _) => "ace".to_string(),
        Card::Number(n @ 2..=10, Suit::Hearts) => format!("{n} of hearts"),
        Card::Number(n, suit) if *n > 10 => format!("weird {n} of {:?}", suit),
        Card::Number(n, Suit::Spades) => format!("{} of spades", n),
        Card::Number(n, suit) => format!("odd {n} of {:?}", suit),
        Card::Face { name, suit: Suit::Spades } => format!("{} of spades", name),
        Card::Face { name, .. } => format!("{} (red)", name),
        Card::Joker => "joker".to_string(),
    }
}

fn classify(n: i32) -> &'static str {
    match n {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        1 | 2 | 3 => "small",
        4..=9 => "medium",
        _ => "large",
    }
}

fn grade(score: u32) -> char {
    match score {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        _ => 'F',
    }
}

fn sum_first_last(v: &[i32]) -> i32 {
    match v {
        [] => 0,
        [one] => *one,
        [first, .., last] => first + last,
    }
}

fn point_kind(p: (i32, i32)) -> String {
    match p {
        (0, 0) => "origin".into(),
        (x, 0) | (0, x) => format!("axis at {x}"),
        (x, y) if x == y => "diagonal".into(),
        _ => "elsewhere".into(),
    }
}

fn main() {
    let e = Expr::Add(
        Box::new(Expr::Num(2.0)),
        Box::new(Expr::Mul(Box::new(Expr::Num(3.0)), Box::new(Expr::Neg(Box::new(Expr::Num(4.0)))))),
    );
    println!("eval = {}", eval(&e));
    let cards = vec![
        Card::Number(1, Suit::Spades),
        Card::Number(7, Suit::Hearts),
        Card::Number(9, Suit::Spades),
        Card::Number(12, Suit::Hearts),
        Card::Face { name: "King".to_string(), suit: Suit::Spades },
        Card::Face { name: "Queen".to_string(), suit: Suit::Hearts },
        Card::Joker,
    ];
    for c in &cards {
        println!("{:?} -> {}", c, describe(c));
    }
    for n in [-5, 0, 2, 7, 100] {
        print!("{} ", classify(n));
    }
    println!();
    println!("{} {} {}", grade(95), grade(81), grade(12));
    println!("{} {} {}", sum_first_last(&[]), sum_first_last(&[5]), sum_first_last(&[1, 2, 3, 10]));
    for p in [(0, 0), (3, 0), (0, -2), (4, 4), (1, 2)] {
        println!("{:?}: {}", p, point_kind(p));
    }
    let maybe = Some(3);
    if let Some(x) = maybe && x > 2 {
        println!("big {x}");
    }
    let is_joker = matches!(cards.last(), Some(Card::Joker));
    println!("last is joker: {is_joker}");
    let reds = cards.iter().filter(|c| matches!(c, Card::Number(_, Suit::Hearts) | Card::Face { suit: Suit::Hearts, .. })).count();
    println!("red cards: {reds}");
    let msg = match maybe {
        Some(0) => "zero",
        Some(n) if n % 2 == 1 => "odd",
        Some(_) => "even",
        None => "nothing",
    };
    println!("{msg}");
}
