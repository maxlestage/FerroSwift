use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Suit {
    Hearts,
    Spades,
}

#[derive(Debug, Clone, PartialEq)]
struct Card {
    suit: Suit,
    rank: u8,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.rank {
            1 => "A".to_string(),
            11 => "J".to_string(),
            n => n.to_string(),
        };
        write!(f, "{}{}", name, if self.suit == Suit::Hearts { '♥' } else { '♠' })
    }
}

struct Deck {
    cards: VecDeque<Card>,
}

impl Deck {
    fn new() -> Self {
        let mut cards = VecDeque::new();
        for suit in [Suit::Hearts, Suit::Spades] {
            for rank in 1..=3 {
                cards.push_back(Card { suit, rank });
            }
        }
        Deck { cards }
    }

    fn draw(&mut self) -> Option<Card> {
        self.cards.pop_front()
    }

    fn put_back(&mut self, card: Card) {
        self.cards.push_back(card);
    }
}

fn describe(cards: &[Card]) -> String {
    cards.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")
}

fn histogram(text: &str) -> BTreeMap<char, usize> {
    let mut h = BTreeMap::new();
    for c in text.chars().filter(|c| c.is_alphabetic()) {
        *h.entry(c.to_ascii_lowercase()).or_insert(0) += 1;
    }
    h
}

fn main() {
    let mut deck = Deck::new();
    let first = deck.draw().unwrap();
    let second = deck.draw().unwrap();
    println!("{first} {second} {}", deck.cards.len());
    deck.put_back(first.clone());
    println!("{}", describe(&deck.cards.iter().cloned().collect::<Vec<_>>()));
    println!("{:?}", deck.cards.back());
    println!("{:?} {:?}", deck.cards.front().map(|c| c.rank), deck.cards.iter().position(|c| c.rank == 1));
    let h = histogram("Hello, World");
    println!("{:?}", h);
    let top: Vec<(char, usize)> = h.iter().map(|(c, n)| (*c, *n)).filter(|(_, n)| *n > 1).collect();
    println!("{:?}", top);
    let mut seen = HashSet::new();
    let dupes: Vec<i32> = [1, 2, 2, 3, 3, 3].iter().filter(|&&x| !seen.insert(x)).copied().collect();
    println!("{:?}", dupes);
    let mut queue: VecDeque<i32> = (1..=5).collect();
    queue.rotate_left(2);
    println!("{:?} {:?}", queue, queue.iter().rev().take(2).collect::<Vec<_>>());
    while let Some(x) = queue.pop_back() {
        if x % 2 == 0 {
            queue.push_front(x / 2);
            if queue.len() > 6 {
                break;
            }
        }
        if x == 1 {
            break;
        }
    }
    println!("{:?}", queue);
    let mut counts: HashMap<Suit, u32> = HashMap::new();
    for c in &deck.cards {
        *counts.entry(c.suit).or_default() += 1;
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort();
    println!("{:?}", pairs);
    let text = "the quick brown fox jumps over the lazy dog the end";
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in text.split_whitespace() {
        *freq.entry(w).or_insert(0) += 1;
    }
    let mut by_count: Vec<(&str, usize)> = freq.iter().map(|(w, c)| (*w, *c)).collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("{:?}", &by_count[..3]);
    let longest = text.split(' ').max_by_key(|w| w.len()).unwrap();
    let shortest = text.split(' ').min_by_key(|w| w.len()).unwrap();
    println!("{longest} {shortest}");
    let caps: String = text.split(' ').map(|w| {
        let mut cs = w.chars();
        match cs.next() {
            Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
            None => String::new(),
        }
    }).collect::<Vec<_>>().join(" ");
    println!("{caps}");
    let acronym: String = text.split(' ').filter_map(|w| w.chars().next()).filter(|c| c.is_alphabetic()).map(|c| c.to_ascii_uppercase()).collect();
    println!("{acronym}");
    let nested: Vec<Vec<u8>> = vec![vec![1, 2], vec![], vec![3]];
    let flat: Vec<u8> = nested.iter().flatten().copied().collect();
    let total: u32 = flat.iter().map(|&b| b as u32).sum();
    println!("{:?} {} {:?}", flat, total, nested.iter().map(Vec::len).collect::<Vec<_>>());
    let windows: Vec<i32> = [1, 4, 9, 16].windows(2).map(|w| w[1] - w[0]).collect();
    let chunks: Vec<i32> = [1, 2, 3, 4, 5].chunks(2).map(|c| c.iter().sum()).collect();
    println!("{:?} {:?}", windows, chunks);
    let zipped: Vec<String> = ["a", "b"].iter().zip([1, 2].iter()).map(|(s, n)| format!("{s}{n}")).collect();
    println!("{:?}", zipped);
    let (evens, odds): (Vec<i32>, Vec<i32>) = (1..=6).partition(|n| n % 2 == 0);
    println!("{:?} {:?}", evens, odds);
    let mut sorted = vec![3.5, -1.0, 2.25];
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{:?} {:?}", sorted, sorted.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let pos = sorted.iter().position(|&x| x > 2.0);
    println!("{:?} {:?}", pos, sorted.binary_search_by(|p| p.partial_cmp(&2.25).unwrap()));
    let mut names = vec!["bob", "alice", "Carol"];
    names.sort_by_key(|n| n.to_lowercase());
    println!("{:?}", names);
    names.dedup();
    names.retain(|n| n.len() > 3);
    println!("{:?} {}", names, names.contains(&"alice"));
    let joined = names.iter().map(|n| n.to_uppercase()).collect::<Vec<_>>().join("-");
    println!("{joined} {}", joined.to_lowercase().replace('-', "+"));
}
