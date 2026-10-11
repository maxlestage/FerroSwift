use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

struct Owner {
    name: String,
    gadgets: RefCell<Vec<Weak<Gadget>>>,
}

struct Gadget {
    id: i32,
    owner: Rc<Owner>,
}

fn main() {
    let leaf = Rc::new(Node { value: 3, parent: RefCell::new(Weak::new()), children: RefCell::new(vec![]) });
    println!("leaf strong = {}, weak = {}", Rc::strong_count(&leaf), Rc::weak_count(&leaf));
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
    {
        let branch = Rc::new(Node { value: 5, parent: RefCell::new(Weak::new()), children: RefCell::new(vec![Rc::clone(&leaf)]) });
        *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
        println!("branch strong = {}, weak = {}", Rc::strong_count(&branch), Rc::weak_count(&branch));
        println!("leaf strong = {}, weak = {}", Rc::strong_count(&leaf), Rc::weak_count(&leaf));
        println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
        println!("children {:?}", branch.children.borrow().iter().map(|c| c.value).collect::<Vec<_>>());
    }
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
    println!("leaf strong = {}, weak = {}", Rc::strong_count(&leaf), Rc::weak_count(&leaf));

    let owner = Rc::new(Owner { name: "Gadget Man".to_string(), gadgets: RefCell::new(vec![]) });
    let g1 = Rc::new(Gadget { id: 1, owner: Rc::clone(&owner) });
    let g2 = Rc::new(Gadget { id: 2, owner: owner.clone() });
    owner.gadgets.borrow_mut().push(Rc::downgrade(&g1));
    owner.gadgets.borrow_mut().push(Rc::downgrade(&g2));
    for g in owner.gadgets.borrow().iter() {
        if let Some(g) = g.upgrade() {
            println!("gadget {} owned by {}", g.id, g.owner.name);
        }
    }
    println!("owner strong {}", Rc::strong_count(&owner));
    drop(g2);
    let alive: Vec<i32> = owner.gadgets.borrow().iter().filter_map(|w| w.upgrade()).map(|g| g.id).collect();
    println!("alive {:?} owner strong {}", alive, Rc::strong_count(&owner));

    let a = Rc::new(5);
    let b = Rc::clone(&a);
    let c = Rc::new(5);
    println!("{} {} {} {}", Rc::ptr_eq(&a, &b), Rc::ptr_eq(&a, &c), a == c, *a + *b);
    println!("{} {}", Rc::strong_count(&a), a);
    let w = Rc::downgrade(&a);
    drop(a);
    println!("{} {:?}", Rc::strong_count(&b), w.upgrade());
    drop(b);
    println!("{:?} {}", w.upgrade(), w.strong_count());
    let shared = Rc::new(RefCell::new(vec![1]));
    let other = Rc::clone(&shared);
    other.borrow_mut().push(2);
    println!("{:?} {}", shared.borrow(), Rc::strong_count(&shared));
}
