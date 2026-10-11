use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Tracker {
    hits: Cell<u32>,
    log: RefCell<Vec<String>>,
    cache: RefCell<HashMap<u64, u64>>,
}

impl Tracker {
    fn touch(&self, what: &str) {
        self.hits.set(self.hits.get() + 1);
        self.log.borrow_mut().push(format!("{}:{}", self.hits.get(), what));
    }

    fn fib(&self, n: u64) -> u64 {
        if n < 2 {
            return n;
        }
        if let Some(&v) = self.cache.borrow().get(&n) {
            self.touch("hit");
            return v;
        }
        let v = self.fib(n - 1) + self.fib(n - 2);
        self.cache.borrow_mut().insert(n, v);
        v
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Account {
    owner: String,
    balance: i64,
}

type Shared = Rc<RefCell<Account>>;

fn transfer(from: &Shared, to: &Shared, amount: i64) -> Result<(), String> {
    if from.borrow().balance < amount {
        return Err(format!("{} has only {}", from.borrow().owner, from.borrow().balance));
    }
    from.borrow_mut().balance -= amount;
    to.borrow_mut().balance += amount;
    Ok(())
}

struct Counter {
    value: Rc<Cell<i32>>,
}

impl Counter {
    fn bump(&self) -> i32 {
        self.value.set(self.value.get() + 1);
        self.value.get()
    }
}

fn main() {
    let t = Tracker::default();
    println!("{} {}", t.fib(20), t.hits.get());
    t.touch("done");
    println!("{:?}", t.log.borrow().iter().rev().take(3).collect::<Vec<_>>());
    println!("{} {}", t.log.borrow().len(), t.cache.borrow().len());
    let a: Shared = Rc::new(RefCell::new(Account { owner: "ann".into(), balance: 100 }));
    let b: Shared = Rc::new(RefCell::new(Account { owner: "bob".into(), balance: 20 }));
    let accounts = vec![Rc::clone(&a), Rc::clone(&b), Rc::clone(&a)];
    println!("{:?}", transfer(&a, &b, 70));
    println!("{:?}", transfer(&a, &b, 70));
    println!("{} {} {}", a.borrow().balance, b.borrow().balance, accounts.iter().map(|x| x.borrow().balance).sum::<i64>());
    println!("{} {} {}", Rc::ptr_eq(&accounts[0], &accounts[2]), Rc::ptr_eq(&accounts[0], &accounts[1]), *a.borrow() == *accounts[2].borrow());
    accounts[1].borrow_mut().owner.push_str("by");
    println!("{:?}", b);
    println!("{:?}", a.borrow().clone());
    let shared = Rc::new(Cell::new(10));
    let c1 = Counter { value: Rc::clone(&shared) };
    let c2 = Counter { value: shared.clone() };
    println!("{} {} {} {:?}", c1.bump(), c2.bump(), shared.get(), shared);
    let old = shared.replace(0);
    let taken = shared.take();
    shared.set(7);
    println!("{old} {taken} {}", shared.get());
    let total = Arc::new(Mutex::new(0u64));
    let names = Arc::new(Mutex::new(Vec::<String>::new()));
    for i in 1..=4u64 {
        let total = Arc::clone(&total);
        let names = Arc::clone(&names);
        let work = move || {
            *total.lock().unwrap() += i * i;
            names.lock().unwrap().push(format!("w{i}"));
        };
        work();
    }
    {
        let mut guard = names.lock().unwrap();
        guard.sort_by(|x, y| y.cmp(x));
        guard.truncate(3);
    }
    println!("{} {:?}", *total.lock().unwrap(), names.lock().unwrap());
    println!("{:?}", total);
    let counter = RefCell::new(0);
    let incr = || *counter.borrow_mut() += 1;
    incr();
    incr();
    println!("{} {:?} {:?}", counter.borrow(), counter, RefCell::new(vec![1, 2]));
    let list: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
    let pusher = {
        let list = Rc::clone(&list);
        move |x: i32| list.borrow_mut().push(x)
    };
    pusher(5);
    pusher(6);
    let doubled: Vec<i32> = list.borrow().iter().map(|x| x * 2).collect();
    list.borrow_mut().retain(|&x| x > 5);
    println!("{:?} {:?} {}", list.borrow(), doubled, list.borrow().contains(&6));
    let mut cells = vec![Rc::new(RefCell::new(3)), Rc::new(RefCell::new(1)), Rc::new(RefCell::new(2))];
    let first = Rc::clone(&cells[0]);
    cells.sort();
    *first.borrow_mut() = 0;
    println!("{:?} {:?}", cells.iter().map(|c| *c.borrow()).collect::<Vec<_>>(), cells.iter().max());
    println!("{:#?}", RefCell::new(Account { owner: "zed".into(), balance: 1 }));
    println!("{:#?}", Mutex::new(vec![1]));
}
