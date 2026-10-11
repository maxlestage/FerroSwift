use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct Node {
    value: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn leaf(value: i32) -> Self {
        Node { value, left: None, right: None }
    }

    fn insert(&mut self, value: i32) {
        if value < self.value {
            match &mut self.left {
                Some(l) => l.insert(value),
                None => self.left = Some(Box::new(Node::leaf(value))),
            }
        } else {
            match self.right.as_mut() {
                Some(r) => r.insert(value),
                None => self.right = Some(Box::new(Node::leaf(value))),
            }
        }
    }

    fn in_order(&self, out: &mut Vec<i32>) {
        if let Some(l) = &self.left {
            l.in_order(out);
        }
        out.push(self.value);
        if let Some(r) = &self.right {
            r.in_order(out);
        }
    }

    fn depth(&self) -> usize {
        let l = self.left.as_ref().map_or(0, |n| n.depth());
        let r = self.right.as_ref().map_or(0, |n| n.depth());
        1 + l.max(r)
    }

    fn contains(&self, value: i32) -> bool {
        if value == self.value {
            return true;
        }
        let next = if value < self.value { &self.left } else { &self.right };
        match next {
            Some(n) => n.contains(value),
            None => false,
        }
    }

    fn min(&self) -> i32 {
        let mut cur = self;
        while let Some(l) = &cur.left {
            cur = l;
        }
        cur.value
    }
}

#[derive(Debug)]
struct Stack {
    head: Option<Box<Frame>>,
    len: usize,
}

#[derive(Debug)]
struct Frame {
    item: String,
    next: Option<Box<Frame>>,
}

impl Stack {
    fn new() -> Self {
        Stack { head: None, len: 0 }
    }

    fn push(&mut self, item: &str) {
        let old = self.head.take();
        self.head = Some(Box::new(Frame { item: item.to_string(), next: old }));
        self.len += 1;
    }

    fn pop(&mut self) -> Option<String> {
        let frame = self.head.take()?;
        self.head = frame.next;
        self.len -= 1;
        Some(frame.item)
    }

    fn peek(&self) -> Option<&str> {
        self.head.as_ref().map(|f| f.item.as_str())
    }

    fn items(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = &self.head;
        while let Some(f) = cur {
            out.push(f.item.clone());
            cur = &f.next;
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Json {
    Null,
    Bool(bool),
    Num(i64),
    Str(String),
    List(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn render(&self) -> String {
        match self {
            Json::Null => "null".to_string(),
            Json::Bool(b) => b.to_string(),
            Json::Num(n) => n.to_string(),
            Json::Str(s) => format!("{s:?}"),
            Json::List(items) => format!("[{}]", items.iter().map(Json::render).collect::<Vec<_>>().join(",")),
            Json::Obj(fields) => format!(
                "{{{}}}",
                fields.iter().map(|(k, v)| format!("{k:?}:{}", v.render())).collect::<Vec<_>>().join(",")
            ),
        }
    }

    fn count(&self) -> usize {
        match self {
            Json::List(items) => 1 + items.iter().map(|j| j.count()).sum::<usize>(),
            Json::Obj(fields) => 1 + fields.iter().map(|(_, v)| v.count()).sum::<usize>(),
            _ => 1,
        }
    }

    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

fn fact(n: u64) -> u64 {
    if n <= 1 { 1 } else { n * fact(n - 1) }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn hanoi(n: u32, from: char, to: char, via: char, moves: &mut Vec<(char, char)>) {
    if n == 0 {
        return;
    }
    hanoi(n - 1, from, via, to, moves);
    moves.push((from, to));
    hanoi(n - 1, via, to, from, moves);
}

fn memo_fib(n: u32, memo: &mut HashMap<u32, u64>) -> u64 {
    if n < 2 {
        return n as u64;
    }
    if let Some(v) = memo.get(&n) {
        return *v;
    }
    let v = memo_fib(n - 1, memo) + memo_fib(n - 2, memo);
    memo.insert(n, v);
    v
}

fn main() {
    let mut root = Node::leaf(50);
    for v in [30, 70, 20, 40, 60, 80, 35] {
        root.insert(v);
    }
    let mut sorted = Vec::new();
    root.in_order(&mut sorted);
    println!("{:?} depth={} min={}", sorted, root.depth(), root.min());
    println!("{} {}", root.contains(35), root.contains(36));
    let copy = root.clone();
    root.insert(1);
    println!("{} {} {}", copy.depth(), root.depth(), copy == root);
    println!("{:?}", copy.left.as_ref().map(|n| n.value));
    let mut stack = Stack::new();
    for w in ["a", "b", "c"] {
        stack.push(w);
    }
    println!("{:?} {:?} {}", stack.items(), stack.peek(), stack.len);
    println!("{:?} {:?} {:?} {:?}", stack.pop(), stack.pop(), stack.pop(), stack.pop());
    println!("{} {:?}", stack.len, stack.peek());
    let doc = Json::Obj(vec![
        ("name".into(), Json::Str("Ferro \"Swift\"".into())),
        ("tags".into(), Json::List(vec![Json::Num(1), Json::Bool(true), Json::Null])),
        ("nested".into(), Json::Obj(vec![("x".into(), Json::Num(-3))])),
    ]);
    println!("{}", doc.render());
    println!("{} {:?}", doc.count(), doc.get("nested").and_then(|n| n.get("x")));
    println!("{:?}", doc.get("missing").is_none());
    let mut list = vec![Json::Num(3), Json::Null, Json::Str("b".into()), Json::Bool(false), Json::Num(1)];
    list.sort();
    println!("{:?}", list);
    println!("{:?}", list.iter().map(|j| matches!(j, Json::Num(_))).collect::<Vec<_>>());
    println!("{} {} {}", fact(20), gcd(1071, 462), gcd(17, 5));
    let mut moves = Vec::new();
    hanoi(3, 'A', 'C', 'B', &mut moves);
    println!("{} {:?}", moves.len(), &moves[..3]);
    let mut memo = HashMap::new();
    println!("{} {}", memo_fib(80, &mut memo), memo.len());
}
