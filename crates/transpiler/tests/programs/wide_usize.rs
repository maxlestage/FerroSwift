// FNV-1a over `usize`: the offset basis is past `i64::MAX`.
fn hash(s: &str) -> usize {
    let mut h: usize = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as usize;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

struct Table {
    buckets: Vec<Vec<String>>,
}

impl Table {
    fn new(n: usize) -> Self {
        Table { buckets: vec![Vec::new(); n] }
    }

    fn insert(&mut self, key: &str) {
        let i = hash(key) % self.buckets.len();
        self.buckets[i].push(key.to_string());
    }
}

fn main() {
    for word in ["hello", "world", "", "FerroSwift"] {
        let h = hash(word);
        println!("{word:>10} {h:>20} {h:016x} {}", h % 97);
    }
    let mut t = Table::new(7);
    for w in ["a", "b", "c", "hello", "world"] {
        t.insert(w);
    }
    for (i, b) in t.buckets.iter().enumerate() {
        if !b.is_empty() {
            println!("{i}: {:?}", b);
        }
    }
}
