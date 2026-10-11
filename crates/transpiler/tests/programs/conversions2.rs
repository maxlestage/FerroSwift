use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Index;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Card {
    rank: u8,
    suit: char,
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.rank, self.suit).cmp(&(other.rank, other.suit))
    }
}

impl Hash for Card {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rank.hash(state);
        self.suit.hash(state);
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rank = match self.rank {
            1 => "A".to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            n => n.to_string(),
        };
        write!(f, "{rank}{}", self.suit)
    }
}

#[derive(Debug, Default)]
struct Deck {
    cards: Vec<Card>,
}

impl IntoIterator for Deck {
    type Item = Card;
    type IntoIter = std::vec::IntoIter<Card>;
    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
    }
}

impl<'a> IntoIterator for &'a Deck {
    type Item = &'a Card;
    type IntoIter = std::slice::Iter<'a, Card>;
    fn into_iter(self) -> Self::IntoIter {
        self.cards.iter()
    }
}

impl FromIterator<Card> for Deck {
    fn from_iter<I: IntoIterator<Item = Card>>(iter: I) -> Self {
        Deck { cards: iter.into_iter().collect() }
    }
}

impl Extend<Card> for Deck {
    fn extend<I: IntoIterator<Item = Card>>(&mut self, iter: I) {
        for c in iter {
            self.cards.push(c);
        }
    }
}

impl Index<usize> for Deck {
    type Output = Card;
    fn index(&self, i: usize) -> &Card {
        &self.cards[i]
    }
}

struct Config {
    values: HashMap<String, String>,
}

impl Index<&str> for Config {
    type Output = String;
    fn index(&self, key: &str) -> &String {
        self.values.get(key).expect("missing key")
    }
}

struct Name(String);

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn shout(s: impl AsRef<str>) -> String {
    s.as_ref().to_uppercase()
}

fn main() {
    let mut deck: Deck = [(1, 's'), (13, 'h'), (7, 'd'), (7, 'c')].iter().map(|&(rank, suit)| Card { rank, suit }).collect();
    deck.extend(vec![Card { rank: 11, suit: 's' }]);
    deck.extend([(2, 'h')].iter().map(|&(rank, suit)| Card { rank, suit }));
    println!("{} {}", deck.cards.len(), deck[1]);
    for c in &deck {
        print!("{c} ");
    }
    println!();
    let mut sorted: Vec<Card> = deck.cards.clone();
    sorted.sort();
    println!("{}", sorted.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" "));
    println!("{:?} {:?}", sorted.iter().max(), deck.cards.iter().min().map(|c| c.rank));
    let mut counts: HashMap<Card, u32> = HashMap::new();
    for c in deck.cards.iter().chain(sorted.iter()) {
        *counts.entry(c.clone()).or_insert(0) += 1;
    }
    println!("{} {:?}", counts.len(), counts.get(&Card { rank: 7, suit: 'd' }));
    let total: u32 = deck.into_iter().map(|c| c.rank as u32).sum();
    println!("{total}");
    let cfg = Config { values: HashMap::from([("host".to_string(), "localhost".to_string()), ("port".to_string(), "8080".to_string())]) };
    println!("{}:{} {}", cfg["host"], cfg["port"], cfg["port"].parse::<u16>().unwrap() + 1);
    println!("{} {} {}", shout(Name("ada".into())), shout("bob"), shout(String::from("cy")));
    let pairs: Deck = (1..=3).map(|r| Card { rank: r, suit: 'x' }).collect();
    let names: Vec<String> = pairs.into_iter().map(|c| format!("{c}")).collect();
    println!("{:?}", names);
    println!("{}", Card { rank: 12, suit: 'h' } > Card { rank: 12, suit: 'd' });
}
