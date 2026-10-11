#[derive(Debug)]
struct ListNode {
    value: i32,
    next: Option<Box<ListNode>>,
}

struct List {
    head: Option<Box<ListNode>>,
    len: usize,
}

impl List {
    fn new() -> List {
        List { head: None, len: 0 }
    }

    fn push_front(&mut self, value: i32) {
        let old = self.head.take();
        self.head = Some(Box::new(ListNode { value, next: old }));
        self.len += 1;
    }

    fn push_back(&mut self, value: i32) {
        let mut cur = &mut self.head;
        while let Some(node) = cur {
            cur = &mut node.next;
        }
        *cur = Some(Box::new(ListNode { value, next: None }));
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<i32> {
        let node = self.head.take()?;
        self.head = node.next;
        self.len -= 1;
        Some(node.value)
    }

    fn reverse(&mut self) {
        let mut prev: Option<Box<ListNode>> = None;
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

    fn remove_all(&mut self, target: i32) -> usize {
        let mut removed = 0;
        let mut cur = &mut self.head;
        loop {
            match cur {
                None => break,
                Some(node) if node.value == target => {
                    *cur = node.next.take();
                    removed += 1;
                }
                Some(node) => {
                    cur = &mut node.next;
                }
            }
        }
        self.len -= removed;
        removed
    }

    fn nth(&self, n: usize) -> Option<i32> {
        let mut cur = self.head.as_ref();
        for _ in 0..n {
            cur = cur?.next.as_ref();
        }
        cur.map(|node| node.value)
    }
}

#[derive(Debug)]
struct Tree {
    key: i32,
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
}

impl Tree {
    fn leaf(key: i32) -> Box<Tree> {
        Box::new(Tree { key, left: None, right: None })
    }

    fn insert(node: &mut Option<Box<Tree>>, key: i32) -> bool {
        match node {
            None => {
                *node = Some(Tree::leaf(key));
                true
            }
            Some(n) if key < n.key => Tree::insert(&mut n.left, key),
            Some(n) if key > n.key => Tree::insert(&mut n.right, key),
            Some(_) => false,
        }
    }

    fn min(node: &Option<Box<Tree>>) -> Option<i32> {
        let mut cur = node.as_ref()?;
        while let Some(l) = cur.left.as_ref() {
            cur = l;
        }
        Some(cur.key)
    }

    fn inorder(node: &Option<Box<Tree>>, out: &mut Vec<i32>) {
        if let Some(n) = node {
            Tree::inorder(&n.left, out);
            out.push(n.key);
            Tree::inorder(&n.right, out);
        }
    }

    fn height(node: &Option<Box<Tree>>) -> usize {
        node.as_ref().map_or(0, |n| 1 + Tree::height(&n.left).max(Tree::height(&n.right)))
    }

    fn contains(node: &Option<Box<Tree>>, key: i32) -> bool {
        let mut cur = node;
        while let Some(n) = cur {
            if key == n.key {
                return true;
            }
            cur = if key < n.key { &n.left } else { &n.right };
        }
        false
    }
}

fn main() {
    let mut list = List::new();
    for v in [3, 1, 4, 1, 5] {
        list.push_back(v);
    }
    list.push_front(9);
    println!("{:?} len {} nth2 {:?} nth9 {:?}", list.to_vec(), list.len, list.nth(2), list.nth(9));
    list.reverse();
    println!("{:?} {:?}", list.to_vec(), list.pop_front());
    println!("removed {} -> {:?} len {}", list.remove_all(1), list.to_vec(), list.len);
    println!("{:?} {:?}", list.remove_all(42), list.head.as_ref().map(|n| n.value));
    while let Some(v) = list.pop_front() {
        print!("{v} ");
    }
    println!("| {} {:?}", list.len, list.head.is_none());

    let mut root: Option<Box<Tree>> = None;
    let inserted: Vec<bool> = [50, 30, 70, 20, 40, 60, 80, 30].iter().map(|&k| Tree::insert(&mut root, k)).collect();
    println!("{:?}", inserted);
    let mut keys = Vec::new();
    Tree::inorder(&root, &mut keys);
    println!("{:?} min {:?} height {} {} {}", keys, Tree::min(&root), Tree::height(&root), Tree::contains(&root, 60), Tree::contains(&root, 65));
    println!("{:?}", root.as_ref().map(|r| (r.key, r.left.as_ref().map(|l| l.key), r.right.as_ref().map(|x| x.key))));
    if let Some(r) = root.as_mut() {
        r.key += 1;
        if let Some(l) = r.left.as_mut() {
            l.right = None;
        }
    }
    let mut keys = Vec::new();
    Tree::inorder(&root, &mut keys);
    println!("{:?} {}", keys, Tree::height(&root));
    let single = ListNode { value: 7, next: None };
    println!("{:?}", single);
}
