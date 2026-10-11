use std::cell::RefCell;
use std::rc::Rc;

struct Res(&'static str);

impl Drop for Res {
    fn drop(&mut self) {
        println!("free {}", self.0);
    }
}

struct Owner {
    res: Rc<Res>,
}

fn main() {
    let a = Rc::new(Res("shared"));
    {
        let b = Rc::clone(&a);
        println!("inner {}", b.0);
    }
    println!("after inner");
    let owners = vec![Owner { res: Rc::clone(&a) }, Owner { res: Rc::clone(&a) }];
    drop(a);
    println!("dropped a, owners {}", owners.len());
    drop(owners);
    println!("dropped owners");
    let cell = Rc::new(RefCell::new(vec![Res("x")]));
    cell.borrow_mut().push(Res("y"));
    cell.borrow_mut().remove(0);
    println!("end");
}
