use std::collections::VecDeque;

struct Queue {
    items: VecDeque<String>,
}

impl Queue {
    fn take_at(&mut self, i: usize) -> Option<String> {
        self.items.remove(i)
    }
}

fn drain_odd(q: &mut VecDeque<i32>) -> Vec<i32> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < q.len() {
        if q[i] % 2 != 0 {
            if let Some(v) = q.remove(i) {
                out.push(v);
            }
        } else {
            i += 1;
        }
    }
    out
}

fn main() {
    let mut q: VecDeque<i32> = (1..=8).collect();
    let (r1, r2) = (q.remove(2), q.remove(100));
    println!("{:?} {:?} {:?}", r1, r2, q);
    let odd = drain_odd(&mut q);
    println!("{:?} {:?}", odd, q);
    let mut d = VecDeque::from(vec![10, 20, 30, 40]);
    let a = d.swap_remove_back(0);
    println!("{:?} {:?}", a, d);
    let b = d.swap_remove_front(2);
    let c = d.swap_remove_back(9);
    println!("{:?} {:?} {:?}", b, c, d);
    let mut queue = Queue { items: VecDeque::new() };
    queue.items.push_back("a".into());
    queue.items.push_back("b".into());
    queue.items.push_front("z".into());
    let (x, y) = (queue.take_at(1), queue.take_at(5));
    println!("{:?} {:?} {:?}", x, y, queue.items);
    let mut v = vec![1, 2, 3];
    println!("{} {:?}", v.remove(0), v);
}
