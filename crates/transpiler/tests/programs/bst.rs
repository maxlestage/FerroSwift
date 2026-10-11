use std::fmt::Debug;

#[derive(Debug)]
struct Node<T> {
    val: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

#[derive(Debug, Default)]
struct Bst<T> {
    root: Option<Box<Node<T>>>,
    size: usize,
}

impl<T: Ord + Copy + Debug> Bst<T> {
    fn new() -> Self {
        Bst { root: None, size: 0 }
    }

    fn insert(&mut self, val: T) -> bool {
        let mut cur = &mut self.root;
        while let Some(node) = cur {
            if val == node.val {
                return false;
            }
            cur = if val < node.val { &mut node.left } else { &mut node.right };
        }
        *cur = Some(Box::new(Node { val, left: None, right: None }));
        self.size += 1;
        true
    }

    fn contains(&self, val: T) -> bool {
        let mut cur = &self.root;
        while let Some(node) = cur {
            if val == node.val {
                return true;
            }
            cur = if val < node.val { &node.left } else { &node.right };
        }
        false
    }

    fn in_order(&self) -> Vec<T> {
        fn walk<T: Copy>(n: &Option<Box<Node<T>>>, out: &mut Vec<T>) {
            if let Some(node) = n {
                walk(&node.left, out);
                out.push(node.val);
                walk(&node.right, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.root, &mut out);
        out
    }

    fn height(&self) -> usize {
        fn h<T>(n: &Option<Box<Node<T>>>) -> usize {
            match n {
                None => 0,
                Some(node) => 1 + h(&node.left).max(h(&node.right)),
            }
        }
        h(&self.root)
    }

    fn min(&self) -> Option<T> {
        let mut cur = self.root.as_ref()?;
        while let Some(l) = cur.left.as_ref() {
            cur = l;
        }
        Some(cur.val)
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone> Stack<T> {
    fn new() -> Self { Stack { items: Vec::new() } }
    fn push(&mut self, x: T) -> &mut Self { self.items.push(x); self }
    fn pop(&mut self) -> Option<T> { self.items.pop() }
    fn peek(&self) -> Option<&T> { self.items.last() }
    fn iter(&self) -> impl Iterator<Item = &T> + '_ { self.items.iter().rev() }
}

fn main() {
    let mut t = Bst::new();
    for x in [50, 30, 70, 20, 40, 60, 80, 30, 65] {
        let added = t.insert(x);
        if !added { println!("dup {x}"); }
    }
    println!("{:?} size={} h={}", t.in_order(), t.size, t.height());
    println!("{} {} {:?}", t.contains(65), t.contains(66), t.min());
    let mut words: Bst<&str> = Bst::new();
    for w in "the quick brown fox jumps over the lazy dog".split(' ') { words.insert(w); }
    println!("{:?}", words.in_order());
    let mut s: Stack<String> = Stack::new();
    s.push("a".into()).push("b".into()).push("c".into());
    println!("{:?} {:?}", s.peek(), s.iter().collect::<Vec<_>>());
    println!("{:?} {:?}", s.pop(), s.items);
}
