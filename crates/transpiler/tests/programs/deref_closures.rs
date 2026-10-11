use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Meters(f64);

impl Deref for Meters {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.0
    }
}

fn main() {
    let v = vec![Meters(1.0), Meters(2.5)];
    v.iter().for_each(|m| println!("{:?}", *m));
    let found = v.iter().any(|m| *m == Meters(2.5));
    let firsts = v.iter().map(|m| *m).collect::<Vec<_>>();
    let sum: f64 = v.iter().map(|m| **m).sum();
    let pos = v.iter().position(|m| **m > 2.0);
    println!("{found} {:?} {sum} {:?}", firsts, pos);
    let big: Vec<&Meters> = v.iter().filter(|m| ***m > 1.5).collect();
    println!("{:?}", big);
    let w = v[0];
    let doubled = *w * 2.0;
    println!("{doubled}");
}
