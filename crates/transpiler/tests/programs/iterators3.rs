use std::collections::{BTreeMap, HashSet};

trait Container {
    type Item;
    fn get(&self, i: usize) -> Option<&Self::Item>;
    fn first(&self) -> Option<&Self::Item> {
        self.get(0)
    }
    fn size(&self) -> usize;
}

struct Ring {
    items: Vec<String>,
    offset: usize,
}

impl Container for Ring {
    type Item = String;
    fn get(&self, i: usize) -> Option<&String> {
        if self.items.is_empty() {
            None
        } else {
            self.items.get((i + self.offset) % self.items.len())
        }
    }
    fn size(&self) -> usize {
        self.items.len()
    }
}

trait Summary {
    fn summary(&self) -> String;
}

impl Summary for Vec<i32> {
    fn summary(&self) -> String {
        format!("{} items, sum {}", self.len(), self.iter().sum::<i32>())
    }
}

impl Summary for str {
    fn summary(&self) -> String {
        format!("{} chars", self.chars().count())
    }
}

fn describe<C: Container>(c: &C) -> String
where
    C::Item: std::fmt::Debug,
{
    format!("{} / {:?} / {:?}", c.size(), c.first(), c.get(c.size() + 1))
}

fn main() {
    let ring = Ring { items: vec!["a".into(), "b".into(), "c".into()], offset: 2 };
    println!("{}", describe(&ring));
    println!("{}", describe(&Ring { items: vec![], offset: 0 }));
    println!("{} | {}", vec![3, 4, 5].summary(), "héllo".summary());
    let data = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
    let running: Vec<i32> = data.iter().scan(0, |acc, &x| { *acc += x; Some(*acc) }).collect();
    println!("{:?}", running);
    let until_big: Vec<&i32> = data.iter().take_while(|&&x| x < 5).collect();
    let after_big: Vec<i32> = data.iter().skip_while(|&&x| x < 5).copied().collect();
    println!("{:?} {:?}", until_big, after_big);
    println!("{:?}", data.windows(3).filter(|w| w[0] < w[1] && w[1] < w[2]).count());
    println!("{:?}", data.iter().rev().enumerate().filter(|(i, _)| i % 3 == 0).map(|(_, x)| x).collect::<Vec<_>>());
    println!("{:?} {:?}", data.iter().max_by_key(|&&x| x % 5), data.iter().min_by_key(|&&x| x % 5));
    println!("{:?}", data.iter().copied().reduce(|a, b| a.max(b) - 1));
    let mut sorted = data.to_vec();
    sorted.sort_by_key(|x| x % 3);
    println!("{:?}", sorted);
    sorted.dedup_by(|a, b| *a % 3 == *b % 3);
    println!("{:?}", sorted);
    let words = ["apple", "fig", "banana", "kiwi", "cherry"];
    let by_len: BTreeMap<usize, Vec<&str>> = words.iter().fold(BTreeMap::new(), |mut m, w| {
        m.entry(w.len()).or_insert_with(Vec::new).push(*w);
        m
    });
    println!("{:?}", by_len);
    let lens: BTreeMap<&str, usize> = words.iter().map(|w| (*w, w.len())).collect();
    println!("{:?} {:?}", lens.get("fig"), lens.values().sum::<usize>());
    let (short, long): (Vec<&str>, Vec<&str>) = words.iter().partition(|w| w.len() <= 4);
    println!("{:?} {:?}", short, long);
    let letters: HashSet<char> = words.iter().flat_map(|w| w.chars()).collect();
    let mut letters: Vec<char> = letters.into_iter().collect();
    letters.sort_unstable();
    println!("{}", letters.iter().collect::<String>());
    let caps: Vec<String> = words.iter().map(|w| w.chars().map(|c| c.to_ascii_uppercase()).collect()).collect();
    println!("{:?}", caps);
    println!("{}", words.iter().map(|w| w.len()).zip(words.iter().skip(1).map(|w| w.len())).all(|(a, b)| a <= b));
    println!("{} {}", words.map(|w| w.len()).iter().sum::<usize>(), data.is_sorted());
    println!("{:?}", data.split(|&x| x == 1).map(|s| s.len()).collect::<Vec<_>>());
    println!("{:?}", data.iter().chain(std::iter::once(&0)).rev().take(3).collect::<Vec<_>>());
    println!("{:?}", std::iter::repeat("ab").take(3).collect::<String>());
    println!("{:?}", std::iter::successors(Some(1u64), |&x| if x < 100 { Some(x * 3) } else { None }).collect::<Vec<_>>());
    println!("{:?} {} {}", (1..=5).rev().collect::<Vec<_>>(), (0..10).step_by(3).sum::<i32>(), (1..=100).contains(&100));
    println!("{} {} {:?}", (1..=5).count(), (3..3).is_empty(), (1..4).map(|x| x * x).last());
    let opts = [Some(2), None, Some(5)];
    println!("{:?} {}", opts.iter().flatten().collect::<Vec<_>>(), opts.iter().flatten().count());
    println!("{:?}", Some(4).iter().chain(None.iter()).copied().collect::<Vec<i32>>());
    println!("{:?}", data.iter().enumerate().skip(1).step_by(4).map(|(i, x)| i as i32 * x).collect::<Vec<_>>());
    println!("{:?} {:?}", data.first().copied().unwrap_or_default(), data.iter().rposition(|&x| x == 5));
    let product: u64 = (1..=10u64).product();
    let fsum: f32 = [0.1f32, 0.2, 0.3].iter().sum();
    println!("{product} {fsum} {}", [1.5f64, 2.5].iter().product::<f64>());
    println!("{:?}", data.chunks(4).zip(data.chunks(4).skip(1)).map(|(a, b)| a.len() + b.len()).collect::<Vec<_>>());
    println!("{:?}", data.iter().map(|x| x.to_string()).collect::<Vec<_>>().concat());
    println!("{:?}", "a1b22c333".chars().filter(char::is_ascii_digit).collect::<String>());
    println!("{:?}", "a1b22c333".split(char::is_numeric).filter(|s| !s.is_empty()).collect::<Vec<_>>());
    println!("{:?}", ["x", "yy", "zzz"].iter().map(|s| s.len()).rev().skip(1).step_by(2).collect::<Vec<_>>());
}
