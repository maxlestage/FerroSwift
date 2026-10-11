struct Noisy {
    id: u32,
}

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.id);
    }
}

trait Shape {
    fn area(&self) -> f64;
}

struct Square(f64, Noisy);

impl Shape for Square {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
}

fn pick(flag: bool) -> Option<Noisy> {
    let a = Noisy { id: 10 };
    let b = Noisy { id: 11 };
    if flag {
        return Some(a);
    }
    println!("not picked");
    Some(b)
}

fn keep_if(n: Noisy, keep: bool) -> Option<Noisy> {
    if keep { Some(n) } else { None }
}

fn main() {
    for i in 0..3 {
        let n = Noisy { id: i };
        if i == 1 {
            println!("skip {}", n.id);
            continue;
        }
        println!("loop {}", n.id);
    }
    let all = vec![Noisy { id: 20 }, Noisy { id: 21 }];
    for n in all {
        println!("item {}", n.id);
    }
    println!("{:?}", pick(true).map(|n| n.id));
    println!("{:?}", pick(false).map(|n| n.id));
    let kept = keep_if(Noisy { id: 30 }, true);
    let dropped = keep_if(Noisy { id: 31 }, false);
    println!("{} {}", kept.is_some(), dropped.is_none());
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Square(2.0, Noisy { id: 40 }))];
    println!("{}", shapes[0].area());
    let mut stack = vec![Noisy { id: 50 }, Noisy { id: 51 }];
    stack.pop();
    println!("after pop");
    let mut opt = Some(Noisy { id: 60 });
    let taken = opt.take();
    println!("{} {}", opt.is_none(), taken.is_some());
    let replaced = std::mem::replace(&mut stack[0], Noisy { id: 52 });
    println!("replaced {}", replaced.id);
    let v: Vec<i32> = Vec::new();
    let _guard = Noisy { id: 99 };
    if stack.len() == 1 {
        let idx = v.len() + 3;
        println!("{}", v[idx]);
    }
}
