#[derive(Debug, Clone, PartialEq)]
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

impl Node {
    fn push_front(self, value: i32) -> Node {
        Node { value, next: Some(Box::new(self)) }
    }
    fn len(&self) -> usize {
        1 + self.next.as_ref().map_or(0, |n| n.len())
    }
    fn sum(&self) -> i32 {
        self.value + self.next.as_ref().map(|n| n.sum()).unwrap_or(0)
    }
    fn to_vec(&self) -> Vec<i32> {
        let mut out = vec![self.value];
        let mut cur = &self.next;
        while let Some(n) = cur {
            out.push(n.value);
            cur = &n.next;
        }
        out
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Lit(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Var(String),
}

impl Expr {
    fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::Lit(v) => *v,
            Expr::Add(a, b) => a.eval(x) + b.eval(x),
            Expr::Mul(a, b) => a.eval(x) * b.eval(x),
            Expr::Neg(a) => -a.eval(x),
            Expr::Var(_) => x,
        }
    }
    fn show(&self) -> String {
        match self {
            Expr::Lit(v) => format!("{v}"),
            Expr::Add(a, b) => format!("({} + {})", a.show(), b.show()),
            Expr::Mul(a, b) => format!("{} * {}", a.show(), b.show()),
            Expr::Neg(a) => format!("-{}", a.show()),
            Expr::Var(n) => n.clone(),
        }
    }
    fn depth(&self) -> usize {
        match self {
            Expr::Lit(_) | Expr::Var(_) => 1,
            Expr::Neg(a) => 1 + a.depth(),
            Expr::Add(a, b) | Expr::Mul(a, b) => 1 + a.depth().max(b.depth()),
        }
    }
}

#[derive(Debug)]
struct Tree {
    root: Option<Box<TreeNode>>,
}

#[derive(Debug)]
struct TreeNode {
    key: i32,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl TreeNode {
    fn insert(node: Option<Box<TreeNode>>, key: i32) -> Option<Box<TreeNode>> {
        match node {
            None => Some(Box::new(TreeNode { key, left: None, right: None })),
            Some(mut n) => {
                if key < n.key {
                    n.left = TreeNode::insert(n.left.take(), key);
                } else if key > n.key {
                    n.right = TreeNode::insert(n.right.take(), key);
                }
                Some(n)
            }
        }
    }
    fn inorder(&self, out: &mut Vec<i32>) {
        if let Some(l) = &self.left {
            l.inorder(out);
        }
        out.push(self.key);
        if let Some(r) = &self.right {
            r.inorder(out);
        }
    }
    fn height(&self) -> usize {
        let l = self.left.as_ref().map_or(0, |n| n.height());
        let r = self.right.as_ref().map_or(0, |n| n.height());
        1 + l.max(r)
    }
}

impl Tree {
    fn new() -> Tree {
        Tree { root: None }
    }
    fn insert(&mut self, key: i32) {
        self.root = TreeNode::insert(self.root.take(), key);
    }
    fn sorted(&self) -> Vec<i32> {
        let mut out = Vec::new();
        if let Some(r) = &self.root {
            r.inorder(&mut out);
        }
        out
    }
    fn contains(&self, key: i32) -> bool {
        let mut cur = &self.root;
        while let Some(n) = cur {
            if key == n.key {
                return true;
            }
            cur = if key < n.key { &n.left } else { &n.right };
        }
        false
    }
}

fn hanoi(n: u32, from: char, to: char, via: char, moves: &mut Vec<String>) {
    if n == 0 {
        return;
    }
    hanoi(n - 1, from, via, to, moves);
    moves.push(format!("{from}->{to}"));
    hanoi(n - 1, via, to, from, moves);
}

fn permutations(items: &[i32]) -> Vec<Vec<i32>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let x = rest.remove(i);
        for mut p in permutations(&rest) {
            p.insert(0, x);
            out.push(p);
        }
    }
    out
}

fn flatten(v: &[Vec<i32>]) -> Vec<i32> {
    v.iter().flat_map(|inner| inner.iter().copied()).collect()
}

fn main() {
    let list = Node { value: 3, next: None }.push_front(2).push_front(1);
    println!("{} {} {:?}", list.len(), list.sum(), list.to_vec());
    println!("{:?}", list.next.as_ref().map(|n| n.value));
    let e = Expr::Add(
        Box::new(Expr::Mul(Box::new(Expr::Lit(2.0)), Box::new(Expr::Var("x".into())))),
        Box::new(Expr::Neg(Box::new(Expr::Lit(1.5)))),
    );
    println!("{} = {} (depth {})", e.show(), e.eval(4.0), e.depth());
    println!("{:?}", Expr::Neg(Box::new(Expr::Lit(1.0))));
    let mut t = Tree::new();
    for k in [5, 3, 8, 1, 4, 7, 9, 3] {
        t.insert(k);
    }
    println!("{:?} {} {} {}", t.sorted(), t.contains(4), t.contains(6), t.root.as_ref().map_or(0, |r| r.height()));
    let mut moves = Vec::new();
    hanoi(3, 'A', 'C', 'B', &mut moves);
    println!("{} {}", moves.len(), moves.join(" "));
    let perms = permutations(&[1, 2, 3]);
    println!("{} {:?}", perms.len(), perms);
    println!("{:?}", flatten(&[vec![1], vec![2, 3], vec![]]));
}
