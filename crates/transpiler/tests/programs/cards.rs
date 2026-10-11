use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Suit {
    Hearts,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Card {
    rank: u8,
    suit: Suit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r = match self.rank {
            1 => "A".to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            n => n.to_string(),
        };
        let s = if self.suit == Suit::Hearts { '♥' } else { '♠' };
        write!(f, "{r}{s}")
    }
}

#[derive(Debug, Default, Clone)]
struct Stats {
    count: usize,
    total: i64,
    min: Option<i64>,
    max: Option<i64>,
    histogram: HashMap<i64, usize>,
}

impl Stats {
    fn add(&mut self, x: i64) -> &mut Self {
        self.count += 1;
        self.total += x;
        self.min = Some(self.min.map_or(x, |m| m.min(x)));
        self.max = Some(match self.max {
            Some(m) if m >= x => m,
            _ => x,
        });
        *self.histogram.entry(x).or_insert(0) += 1;
        self
    }

    fn mean(&self) -> Option<f64> {
        if self.count == 0 { None } else { Some(self.total as f64 / self.count as f64) }
    }

    fn mode(&self) -> Option<i64> {
        self.histogram.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0))).map(|(k, _)| *k)
    }
}

struct Lcg(u64);

impl Lcg {
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next_u32() % n
    }
    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u32 + 1) as usize;
            items.swap(i, j);
        }
    }
}

fn main() {
    let mut deck: Vec<Card> = Vec::new();
    for suit in [Suit::Spades, Suit::Hearts] {
        for rank in 1..=13 {
            deck.push(Card { rank, suit });
        }
    }
    let mut rng = Lcg(42);
    rng.shuffle(&mut deck);
    let hand: Vec<Card> = deck.drain(..5).collect();
    println!("{} | {}", hand.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" "), deck.len());
    let mut sorted = hand.clone();
    sorted.sort();
    sorted.dedup();
    println!("{:?}", sorted.iter().map(ToString::to_string).collect::<Vec<_>>());
    println!("{:?} {:?}", sorted.iter().max(), sorted.iter().min_by_key(|c| (c.suit, std::cmp::Reverse(c.rank))));
    let by_suit: HashMap<Suit, Vec<u8>> = hand.iter().fold(HashMap::new(), |mut m, c| {
        m.entry(c.suit).or_insert_with(Vec::new).push(c.rank);
        m
    });
    let mut suits: Vec<(&Suit, &Vec<u8>)> = by_suit.iter().collect();
    suits.sort();
    println!("{:?}", suits);
    let mut stats = Stats::default();
    stats.add(3).add(7).add(3).add(-2);
    for _ in 0..5 {
        stats.add(rng.below(10) as i64);
    }
    println!("{} {} {:?} {:?} {:?} {:?}", stats.count, stats.total, stats.min, stats.max, stats.mean().map(|m| (m * 100.0).round() / 100.0), stats.mode());
    let empty = Stats::default();
    println!("{:?} {:?} {}", empty.mean(), empty.mode(), empty.histogram.is_empty());
    let draws: Vec<u32> = (0..8).map(|_| rng.below(6) + 1).collect();
    let counts = draws.iter().fold([0usize; 6], |mut acc, &d| {
        acc[d as usize - 1] += 1;
        acc
    });
    println!("{:?} {:?} {}", draws, counts, counts.iter().sum::<usize>());
    let matrix = [[1, 2, 3], [4, 5, 6]];
    let transposed: Vec<Vec<i32>> = (0..3).map(|c| matrix.iter().map(|row| row[c]).collect()).collect();
    let flat: Vec<i32> = matrix.iter().flat_map(|r| r.iter().copied()).collect();
    println!("{:?} {:?} {} {:?}", transposed, flat, matrix.len() * matrix[0].len(), matrix.iter().map(|r| r.iter().sum::<i32>()).collect::<Vec<_>>());
    let mut grid = [[0u8; 3]; 3];
    grid[1][1] = 5;
    grid[2] = [7, 8, 9];
    for row in grid.iter_mut() {
        row[0] += 1;
    }
    println!("{:?} {} {:?}", grid, grid.iter().flatten().filter(|&&x| x > 0).count(), grid.concat());
    let words = ["alpha", "beta", "gamma", "delta"];
    let lookup: HashMap<&str, usize> = words.iter().enumerate().map(|(i, w)| (*w, i)).collect();
    let mut order: Vec<(&&str, &usize)> = lookup.iter().collect();
    order.sort_by_key(|(_, i)| **i);
    println!("{:?} {:?} {:?}", order, lookup.get("gamma"), words.iter().map(|w| w.len()).rev().collect::<Vec<_>>());
}
