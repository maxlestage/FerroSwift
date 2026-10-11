struct Node {
    val: i32,
    next: Option<Box<Node>>,
}

impl Drop for Node {
    fn drop(&mut self) {
        println!("drop node {}", self.val);
    }
}

struct Session {
    name: String,
    log: Vec<String>,
}

impl Session {
    fn new(name: &str) -> Self {
        println!("open {name}");
        Session { name: name.to_string(), log: vec![] }
    }
    fn record(&mut self, s: &str) {
        self.log.push(s.to_string());
    }
    fn finish(self) -> usize {
        println!("finish {} with {}", self.name, self.log.len());
        self.log.len()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        println!("close {}", self.name);
    }
}

enum Event {
    Start(Session),
    Idle,
}

fn main() {
    let list = Node { val: 1, next: Some(Box::new(Node { val: 2, next: Some(Box::new(Node { val: 3, next: None })) })) };
    println!("head {}", list.val);
    let mut s = Session::new("a");
    s.record("x");
    s.record("y");
    let n = s.finish();
    println!("n = {n}");
    let events = vec![Event::Start(Session::new("b")), Event::Idle];
    for e in events {
        match e {
            Event::Start(s) => println!("start {}", s.name),
            Event::Idle => println!("idle"),
        }
    }
    let t = Session::new("t");
    let moved = t;
    println!("moved {}", moved.name);
    {
        let _scoped = Session::new("scoped");
        println!("in scope");
    }
    println!("end");
}
