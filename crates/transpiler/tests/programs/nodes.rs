use std::fmt;

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

struct Stack<T> {
    head: Option<Box<Node<T>>>,
}

impl<T: PartialOrd + Copy + fmt::Debug> Stack<T> {
    fn new() -> Self {
        Stack { head: None }
    }

    fn push(&mut self, value: T) {
        let old = self.head.take();
        self.head = Some(Box::new(Node { value, next: old }));
    }

    fn insert_sorted(&mut self, value: T) {
        let mut cur = &mut self.head;
        while cur.as_ref().is_some_and(|node| node.value < value) {
            cur = &mut cur.as_mut().unwrap().next;
        }
        let rest = cur.take();
        *cur = Some(Box::new(Node { value, next: rest }));
    }

    fn items(&self) -> Vec<T> {
        let mut out = Vec::new();
        let mut cur = &self.head;
        while let Some(node) = cur {
            out.push(node.value);
            cur = &node.next;
        }
        out
    }

    fn drop_while(&mut self, pred: impl Fn(T) -> bool) -> usize {
        let mut dropped = 0;
        while let Some(node) = &self.head {
            if !pred(node.value) {
                break;
            }
            self.head = self.head.take().and_then(|n| n.next);
            dropped += 1;
        }
        dropped
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

impl<T: fmt::Display + Clone> fmt::Display for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tree::Leaf(v) => write!(f, "{v}"),
            Tree::Node(l, r) => write!(f, "({l} {r})"),
        }
    }
}

impl<T: Clone> Tree<T> {
    fn leaves(&self) -> Vec<T> {
        match self {
            Tree::Leaf(v) => vec![v.clone()],
            Tree::Node(l, r) => {
                let mut out = l.leaves();
                out.extend(r.leaves());
                out
            }
        }
    }

    fn map<U>(&self, f: &impl Fn(&T) -> U) -> Tree<U> {
        match self {
            Tree::Leaf(v) => Tree::Leaf(f(v)),
            Tree::Node(l, r) => Tree::Node(Box::new(l.map(f)), Box::new(r.map(f))),
        }
    }

    fn depth(&self) -> usize {
        match self {
            Tree::Leaf(_) => 1,
            Tree::Node(l, r) => 1 + l.depth().max(r.depth()),
        }
    }
}

fn build(values: &[i32]) -> Tree<i32> {
    if values.len() == 1 {
        return Tree::Leaf(values[0]);
    }
    let mid = values.len() / 2;
    Tree::Node(Box::new(build(&values[..mid])), Box::new(build(&values[mid..])))
}

fn main() {
    let mut s: Stack<i32> = Stack::new();
    for v in [5, 1, 9, 3, 7, 3] {
        s.insert_sorted(v);
    }
    println!("{:?}", s.items());
    s.push(0);
    println!("{:?} dropped {} -> {:?}", s.items(), s.drop_while(|v| v < 4), s.items());
    println!("{}", s.drop_while(|v| v < 100));
    println!("{:?} {:?}", s.items(), s.head.is_none());
    let mut words: Stack<&str> = Stack::new();
    for w in ["pear", "apple", "fig"] {
        words.insert_sorted(w);
    }
    println!("{:?}", words.items());

    let t = build(&[1, 2, 3, 4, 5]);
    println!("{} depth {} leaves {:?}", t, t.depth(), t.leaves());
    let doubled = t.map(&|v| v * 2);
    let labels = t.map(&|v| format!("n{v}"));
    println!("{} | {}", doubled, labels);
    println!("{:?} {}", labels.leaves(), doubled == build(&[2, 4, 6, 8, 10]));
    println!("{:?}", Tree::Node(Box::new(Tree::Leaf('a')), Box::new(Tree::Leaf('b'))));
}
