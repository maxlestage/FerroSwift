use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::rc::Rc;

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser { src, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.pos += c.len_utf8();
        }
    }

    fn word(&mut self) -> Option<&'a str> {
        self.skip_ws();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        if start == self.pos { None } else { Some(&self.src[start..self.pos]) }
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
enum Level {
    Low = 1,
    Mid = 5,
    High = 10,
}

impl TryFrom<u8> for Level {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, String> {
        match v {
            1 => Ok(Level::Low),
            5 => Ok(Level::Mid),
            10 => Ok(Level::High),
            _ => Err(format!("no level {v}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Config {
    name: String,
    retries: u32,
    verbose: bool,
    tags: Vec<String>,
    limits: BTreeMap<String, i64>,
}

#[derive(Debug)]
struct Node {
    value: i32,
    children: Vec<Rc<RefCell<Node>>>,
}

fn sum_tree(n: &Rc<RefCell<Node>>) -> i32 {
    let node = n.borrow();
    node.value + node.children.iter().map(sum_tree).sum::<i32>()
}

fn main() {
    let mut p = Parser::new("  hello wörld_42 ; tail");
    let mut words = Vec::new();
    while let Some(w) = p.word() {
        words.push(w);
    }
    println!("{:?} {:?} {}", words, p.rest(), p.pos);
    let text = "the quick brown fox jumps over the lazy dog the end";
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in text.split_whitespace() {
        *freq.entry(w).or_insert(0) += 1;
    }
    let mut pairs: Vec<(&str, usize)> = freq.iter().map(|(k, v)| (*k, *v)).collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("{:?}", &pairs[..4]);
    let mut lengths: HashMap<usize, Vec<&str>> = HashMap::new();
    for w in text.split(' ') {
        lengths.entry(w.len()).or_default().push(w);
    }
    let mut keys: Vec<_> = lengths.keys().copied().collect();
    keys.sort_unstable_by(|a, b| b.cmp(a));
    for k in keys.iter().take(2) {
        let mut ws = lengths[k].clone();
        ws.sort();
        ws.dedup();
        println!("{k}: {}", ws.join(","));
    }
    let mut counts: HashMap<char, i32> = HashMap::new();
    for c in "mississippi".chars() {
        counts.entry(c).and_modify(|n| *n += 1).or_insert(1);
    }
    let mut cs: Vec<(char, i32)> = counts.into_iter().collect();
    cs.sort_by_key(|&(c, n)| (std::cmp::Reverse(n), c));
    println!("{:?}", cs);
    if let Some(n) = freq.get_mut("the") {
        *n *= 10;
    }
    let removed_fox = freq.remove("fox");
    println!("{:?} {:?} {}", freq.get("the"), removed_fox, freq.contains_key("fox"));
    let mut bt: BTreeMap<i32, &str> = BTreeMap::new();
    for (i, w) in ["zero", "one", "two", "three", "four", "five"].iter().enumerate() {
        bt.insert(i as i32 * 10, w);
    }
    println!("{:?} {:?}", bt.first_key_value(), bt.last_key_value());
    println!("{:?}", bt.range(15..=40).map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>());
    bt.retain(|k, _| k % 20 == 0);
    println!("{:?} {}", bt, bt.len());
    let mut dq: VecDeque<i32> = VecDeque::new();
    for i in 1..=5 {
        if i % 2 == 0 { dq.push_front(i) } else { dq.push_back(i) }
    }
    println!("{:?} {:?} {:?} {:?}", dq, dq.front(), dq.back(), dq.iter().rev().collect::<Vec<_>>());
    dq.rotate_left(2);
    let popped = (dq.pop_front(), dq.pop_back());
    println!("{:?} {:?} {} {:?}", dq, popped, dq.len(), dq.get(1));
    let mut grid = vec![vec![0u32; 4]; 3];
    for (r, row) in grid.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (r * 4 + c) as u32;
        }
    }
    grid[1][2] += 100;
    for row in &mut grid {
        row.reverse();
    }
    println!("{:?}", grid);
    let col_sums: Vec<u32> = (0..4).map(|c| grid.iter().map(|row| row[c]).sum()).collect();
    let diag: Vec<u32> = (0..3).map(|i| grid[i][i]).collect();
    println!("{:?} {:?} {}", col_sums, diag, grid.iter().flatten().filter(|&&x| x > 50).count());
    let found = loop {
        let mut i = 0;
        while i < 100 {
            if i * i > 300 {
                break;
            }
            i += 1;
        }
        break i;
    };
    println!("{found}");
    println!("{} {} {:?} {:?}", Level::Mid as u8, Level::High as i32 + 1, Level::try_from(5), Level::try_from(7));
    let mut levels = vec![Level::High, Level::Low, Level::Mid];
    levels.sort();
    println!("{:?} {:?} {}", levels, levels.iter().max(), Level::Low < Level::High);
    let base = Config { name: "base".into(), retries: 3, ..Default::default() };
    let custom = Config { verbose: true, tags: vec!["a".into(), "b".into()], ..base.clone() };
    let mut limited = custom.clone();
    limited.limits.insert("cpu".into(), 2);
    limited.limits.insert("mem".into(), 512);
    println!("{:?}", base);
    println!("{:#?}", limited);
    println!("{} {}", base == custom, custom == Config { verbose: true, tags: vec!["a".into(), "b".into()], ..base.clone() });
    let leaf = |v: i32| Rc::new(RefCell::new(Node { value: v, children: vec![] }));
    let root = Rc::new(RefCell::new(Node { value: 1, children: vec![leaf(2), leaf(3)] }));
    let extra = leaf(10);
    root.borrow_mut().children.push(Rc::clone(&extra));
    extra.borrow_mut().value += 5;
    extra.borrow_mut().children.push(leaf(100));
    println!("{} {} {}", sum_tree(&root), root.borrow().children.len(), Rc::ptr_eq(&extra, &root.borrow().children[2]));
    let shared = Rc::new(RefCell::new(Vec::<i32>::new()));
    let s2 = Rc::clone(&shared);
    let push = move |x: i32| s2.borrow_mut().push(x);
    push(1);
    push(2);
    println!("{:?} {}", shared.borrow(), shared.borrow().len());
    let mut v: Vec<i32> = (1..=10).collect();
    v.retain_mut(|x| {
        *x *= 3;
        *x % 2 == 0
    });
    let drained: Vec<i32> = v.drain(1..3).collect();
    v.extend_from_slice(&[7, 7]);
    v.insert(0, -1);
    let removed = v.swap_remove(1);
    v.truncate(4);
    println!("{:?} {:?} {} {:?}", v, drained, removed, v.binary_search(&7));
    v.resize(6, 0);
    let tail = v.split_off(4);
    println!("{:?} {:?} {} {:?}", v, tail, v.contains(&12), v.iter().position(|&x| x == 12));
    v.rotate_right(1);
    v.dedup();
    if let Some(x) = v.first_mut() {
        *x += 1;
    }
    println!("{:?} {:?} {:?}", v, v.first(), v.last());
    let strs = ["Hello", "WORLD", "mIxEd"];
    println!("{:?}", strs.iter().map(|s| s.eq_ignore_ascii_case("hello")).collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", "key=value=x".split_once('='), "key=value=x".rsplit_once('='), "a,b,c,d".splitn(3, ',').collect::<Vec<_>>());
    println!("{:?} {:?}", "a,b,c,d".rsplitn(2, ',').collect::<Vec<_>>(), "prefix_body".strip_prefix("prefix_"));
    println!("{:?} {:?} {}", "body.rs".strip_suffix(".rs"), "xxabcxx".trim_start_matches('x'), "ab".repeat(3));
    println!("{:?}", "line one\nline two\n\nline four".lines().map(|l| l.len()).collect::<Vec<_>>());
    println!("{:?} {:?}", "héllo".as_bytes().len(), "héllo".bytes().map(|b| b as u32).sum::<u32>());
    println!("{} {:?}", String::from_utf8_lossy(&[72, 105, 255]), "abc".char_indices().map(|(i, c)| format!("{i}{c}")).collect::<Vec<_>>());
    println!("{:?} {}", "hello".chars().rev().collect::<String>(), "Hello World".to_lowercase().split(' ').map(|w| w.chars().rev().collect::<String>()).collect::<Vec<_>>().join(" "));
    let bytes = b"bytes!";
    println!("{:?} {} {}", bytes, bytes[0], bytes.len());
}
