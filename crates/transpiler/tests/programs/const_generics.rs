#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector<const N: usize> {
    data: [f64; N],
}

impl<const N: usize> Vector<N> {
    fn zero() -> Self {
        Vector { data: [0.0; N] }
    }
    fn from(data: [f64; N]) -> Self {
        Vector { data }
    }
    fn dot(&self, other: &Vector<N>) -> f64 {
        (0..N).map(|i| self.data[i] * other.data[i]).sum()
    }
    fn len(&self) -> usize {
        N
    }
}

struct Buffer<T, const CAP: usize> {
    items: Vec<T>,
}

impl<T: std::fmt::Debug, const CAP: usize> Buffer<T, CAP> {
    fn new() -> Self {
        Buffer { items: Vec::new() }
    }
    fn push(&mut self, x: T) -> bool {
        if self.items.len() < CAP {
            self.items.push(x);
            true
        } else {
            false
        }
    }
}

fn sum_all<const N: usize>(xs: [i32; N]) -> i32 {
    xs.iter().sum::<i32>() + N as i32
}

fn repeat_char<const N: usize>(c: char) -> String {
    std::iter::repeat(c).take(N).collect()
}

fn main() {
    let a = Vector::from([1.0, 2.0, 3.0]);
    let b: Vector<3> = Vector::zero();
    println!("{} {} {:?}", a.dot(&a), b.len(), b);
    let mut buf: Buffer<&str, 2> = Buffer::new();
    println!("{} {} {}", buf.push("a"), buf.push("b"), buf.push("c"));
    println!("{:?}", buf.items);
    println!("{}", sum_all([1, 2, 3, 4]));
    println!("{}", repeat_char::<5>('*'));
}
