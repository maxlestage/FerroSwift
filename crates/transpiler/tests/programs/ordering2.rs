use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Token {
    Num(i64),
    Word(String),
    Pair(u8, u8),
    Open { depth: usize },
    End,
}

#[derive(Debug, Clone, PartialEq, Default)]
enum Mode {
    #[default]
    Idle,
    Busy(u32),
}

#[derive(Debug)]
enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>),
}

impl Tree {
    fn sum(&self) -> i32 {
        match self {
            Tree::Leaf(v) => *v,
            Tree::Node(l, r) => l.sum() + r.sum(),
        }
    }

    fn depth(&self) -> usize {
        match self {
            Tree::Leaf(_) => 1,
            Tree::Node(l, r) => 1 + l.depth().max(r.depth()),
        }
    }
}

fn main() {
    let mut tokens = vec![
        Token::Word("b".into()),
        Token::Num(3),
        Token::End,
        Token::Pair(1, 9),
        Token::Open { depth: 2 },
        Token::Num(-1),
        Token::Word("a".into()),
        Token::Pair(1, 2),
        Token::Open { depth: 1 },
    ];
    tokens.sort();
    println!("{:?}", tokens);
    println!("{} {} {}", Token::Num(5) < Token::Word("".into()), Token::Pair(1, 2) < Token::Pair(1, 10), Token::Open { depth: 3 } > Token::End);
    println!("{:?} {:?}", tokens.iter().max(), tokens.iter().min());
    let nums: Vec<i64> = tokens.iter().filter_map(|t| if let Token::Num(n) = t { Some(*n) } else { None }).collect();
    println!("{:?} {}", nums, tokens.iter().filter(|t| matches!(t, Token::Pair(1, _) | Token::Open { .. })).count());
    let set: HashSet<Token> = tokens.iter().cloned().collect();
    let mut set_sorted: Vec<&Token> = set.iter().collect();
    set_sorted.sort();
    println!("{} {:?}", set.len(), set_sorted.first());
    let mut by_kind: HashMap<&str, Vec<&Token>> = HashMap::new();
    for t in &tokens {
        let kind = match t {
            Token::Num(_) => "num",
            Token::Word(_) => "word",
            Token::Pair(..) => "pair",
            Token::Open { .. } => "open",
            Token::End => "end",
        };
        by_kind.entry(kind).or_default().push(t);
    }
    let mut kinds: Vec<_> = by_kind.iter().map(|(k, v)| (*k, v.len())).collect();
    kinds.sort();
    println!("{:?}", kinds);

    let mut hands: BTreeMap<Suit, Vec<u8>> = BTreeMap::new();
    for (suit, card) in [(Suit::Spades, 3), (Suit::Clubs, 11), (Suit::Spades, 1), (Suit::Hearts, 7)] {
        hands.entry(suit).or_default().push(card);
    }
    println!("{:?}", hands);
    println!("{:?} {} {:?}", hands.keys().next(), Suit::Diamonds as u8, hands.get(&Suit::Diamonds));
    let order: Vec<Suit> = {
        let mut s = vec![Suit::Hearts, Suit::Clubs, Suit::Spades];
        s.sort_by_key(|x| std::cmp::Reverse(*x as u8));
        s
    };
    println!("{:?} {}", order, order.contains(&Suit::Clubs));

    let modes = [Mode::default(), Mode::Busy(2), Mode::Busy(0)];
    let busy: Vec<u32> = modes.iter().filter_map(|m| match m { Mode::Busy(n) if *n > 0 => Some(*n), _ => None }).collect();
    println!("{:?} {:?} {}", modes, busy, modes.iter().filter(|m| **m == Mode::Idle).count());
    let maybe: Option<Mode> = Some(Mode::Busy(4));
    match maybe {
        Some(Mode::Busy(n)) if n > 3 => println!("very busy {n}"),
        Some(Mode::Busy(n)) => println!("busy {n}"),
        Some(Mode::Idle) | None => println!("idle"),
    }
    let tree = Tree::Node(Box::new(Tree::Leaf(1)), Box::new(Tree::Node(Box::new(Tree::Leaf(2)), Box::new(Tree::Leaf(3)))));
    println!("{} {} {:?}", tree.sum(), tree.depth(), tree);
    if let Tree::Node(left, _) = &tree {
        if let Tree::Leaf(v) = **left {
            println!("left leaf {v}");
        }
    }
    let nested = Some(Some(Token::Pair(4, 5)));
    if let Some(Some(Token::Pair(a, b))) = nested {
        println!("{}", a + b);
    }
    println!("{:?} {:?}", Some(Some(3)).flatten(), Option::<Option<i32>>::None.flatten());
}
