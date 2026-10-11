use std::fmt;

#[derive(Debug)]
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

struct List {
    head: Option<Box<Node>>,
    len: usize,
}

impl List {
    fn new() -> Self {
        List { head: None, len: 0 }
    }

    fn push(&mut self, value: i32) {
        let old = self.head.take();
        self.head = Some(Box::new(Node { value, next: old }));
        self.len += 1;
    }

    fn pop(&mut self) -> Option<i32> {
        self.head.take().map(|n| {
            self.head = n.next;
            self.len -= 1;
            n.value
        })
    }

    fn peek(&self) -> Option<&i32> {
        self.head.as_ref().map(|n| &n.value)
    }

    fn reverse(&mut self) {
        let mut prev: Option<Box<Node>> = None;
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = node.next.take();
            node.next = prev;
            prev = Some(node);
        }
        self.head = prev;
    }

    fn to_vec(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let mut cur = &self.head;
        while let Some(node) = cur {
            out.push(node.value);
            cur = &node.next;
        }
        out
    }

    fn sum(&self) -> i32 {
        fn walk(n: &Option<Box<Node>>) -> i32 {
            match n {
                Some(node) => node.value + walk(&node.next),
                None => 0,
            }
        }
        walk(&self.head)
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut cur = &self.head;
        let mut first = true;
        while let Some(node) = cur {
            if !first {
                write!(f, " -> ")?;
            }
            write!(f, "{}", node.value)?;
            first = false;
            cur = &node.next;
        }
        write!(f, "]")
    }
}

#[derive(Debug)]
enum Tree {
    Leaf,
    Branch(Box<Tree>, i32, Box<Tree>),
}

impl Tree {
    fn insert(self, v: i32) -> Tree {
        match self {
            Tree::Leaf => Tree::Branch(Box::new(Tree::Leaf), v, Box::new(Tree::Leaf)),
            Tree::Branch(l, x, r) => {
                if v < x {
                    Tree::Branch(Box::new(l.insert(v)), x, r)
                } else {
                    Tree::Branch(l, x, Box::new(r.insert(v)))
                }
            }
        }
    }

    fn inorder(&self, out: &mut Vec<i32>) {
        if let Tree::Branch(l, x, r) = self {
            l.inorder(out);
            out.push(*x);
            r.inorder(out);
        }
    }

    fn depth(&self) -> usize {
        match self {
            Tree::Leaf => 0,
            Tree::Branch(l, _, r) => 1 + l.depth().max(r.depth()),
        }
    }

    fn contains(&self, v: i32) -> bool {
        match self {
            Tree::Leaf => false,
            Tree::Branch(l, x, r) => {
                if v == *x {
                    true
                } else if v < *x {
                    l.contains(v)
                } else {
                    r.contains(v)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Idle,
    Running { speed: u8 },
    Paused(u8),
    Done,
}

#[derive(Debug, Clone, Copy)]
enum Event {
    Start,
    Faster,
    Pause,
    Resume,
    Stop,
}

fn step(state: State, event: Event) -> State {
    match (state, event) {
        (State::Idle, Event::Start) => State::Running { speed: 1 },
        (State::Running { speed }, Event::Faster) if speed < 3 => State::Running { speed: speed + 1 },
        (State::Running { .. }, Event::Faster) => state,
        (State::Running { speed }, Event::Pause) => State::Paused(speed),
        (State::Paused(speed), Event::Resume) => State::Running { speed },
        (State::Running { .. } | State::Paused(_), Event::Stop) => State::Done,
        (s, _) => s,
    }
}

fn main() {
    let mut list = List::new();
    for v in [3, 1, 4, 1, 5] {
        list.push(v);
    }
    println!("{} len={} sum={} peek={:?}", list, list.len, list.sum(), list.peek());
    list.reverse();
    println!("{} {:?}", list, list.to_vec());
    println!("{:?} {:?} {}", list.pop(), list.pop(), list);
    println!("{:?}", list.head.as_ref().map(|n| n.value));
    let empty = List::new();
    println!("{} {:?} {}", empty, empty.peek(), empty.sum());
    let mut tree = Tree::Leaf;
    for v in [50, 30, 70, 20, 40, 60, 80, 30] {
        tree = tree.insert(v);
    }
    let mut out = Vec::new();
    tree.inorder(&mut out);
    println!("{:?} depth={} {} {}", out, tree.depth(), tree.contains(60), tree.contains(65));
    if let Tree::Branch(l, root, _) = &tree {
        println!("root={root} left_depth={}", l.depth());
    }
    println!("{:?}", Tree::Leaf.insert(1));
    let events = [Event::Faster, Event::Start, Event::Faster, Event::Faster, Event::Faster, Event::Pause, Event::Faster, Event::Resume, Event::Stop, Event::Start];
    let mut state = State::Idle;
    let mut trace = Vec::new();
    for e in events {
        let next = step(state, e);
        if next != state {
            trace.push(format!("{e:?}->{next:?}"));
        }
        state = next;
    }
    println!("{}", trace.join(" | "));
    println!("{:?} {}", state, state == State::Done);
}
