use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Meters(f64);

impl Deref for Meters {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
struct Tagged {
    id: u32,
    weight: Meters,
}

fn copy_out(m: &Meters) -> Meters {
    *m
}

fn raw(m: &Meters) -> f64 {
    **m
}

fn total(ms: &[Meters]) -> f64 {
    let mut t = 0.0;
    for m in ms {
        let own: Meters = *m;
        t += *own + **m;
    }
    t
}

fn main() {
    let a = Meters(2.5);
    let b = copy_out(&a);
    println!("{:?} {} {}", b, raw(&a), *a);
    let r = &a;
    let c = *r;
    println!("{:?} {}", c, **r + 1.0);
    let v = vec![Meters(1.0), Meters(2.0)];
    println!("{}", total(&v));
    let best = v.iter().map(|m| **m).fold(0.0, f64::max);
    let copies: Vec<Meters> = v.iter().map(|m| *m).collect();
    println!("{best} {:?}", copies);
    let t = Tagged { id: 1, weight: Meters(4.0) };
    let tr = &t;
    let w = tr.weight;
    println!("{} {:?} {}", tr.id, w, *w * 2.0);
    let same = *r == a;
    println!("{same}");
}
