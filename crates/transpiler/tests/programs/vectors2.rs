use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    fn len(&self) -> f32 {
        self.x.hypot(self.y)
    }

    fn angle(&self) -> f32 {
        self.y.atan2(self.x).to_degrees()
    }

    fn normalized(self) -> Vec2 {
        let l = self.len();
        if l == 0.0 { Vec2::ZERO } else { self * (1.0 / l) }
    }

    fn dot(self, o: Vec2) -> f32 {
        self.x.mul_add(o.x, self.y * o.y)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f32) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Vec2 {
        v * self
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl Sum for Vec2 {
    fn sum<I: Iterator<Item = Vec2>>(iter: I) -> Vec2 {
        iter.fold(Vec2::ZERO, |a, b| a + b)
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(t: (f32, f32)) -> Vec2 {
        Vec2::new(t.0, t.1)
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f32>,
}

impl Matrix {
    fn identity(n: usize) -> Matrix {
        let mut m = Matrix { rows: n, cols: n, data: vec![0.0; n * n] };
        for i in 0..n {
            m[(i, i)] = 1.0;
        }
        m
    }

    fn apply(&self, v: Vec2) -> Vec2 {
        Vec2::new(self[(0, 0)] * v.x + self[(0, 1)] * v.y, self[(1, 0)] * v.x + self[(1, 1)] * v.y)
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f32;
    fn index(&self, (r, c): (usize, usize)) -> &f32 {
        &self.data[r * self.cols + c]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut f32 {
        &mut self.data[r * self.cols + c]
    }
}

fn main() {
    let a = Vec2::new(3.0, 4.0);
    let b: Vec2 = (1.0, -2.0).into();
    println!("{} {} {:?} {}", a + b, a - b, a * 2.0, 0.5 * b);
    println!("{} {} {:.3} {}", -a, a.len(), a.angle(), a.dot(b));
    println!("{} {} {}", a.normalized(), Vec2::ZERO.normalized(), Vec2::default() == Vec2::ZERO);
    let pts = vec![a, b, Vec2::new(-1.5, 0.25)];
    let total: Vec2 = pts.iter().copied().sum();
    let mut acc = Vec2::ZERO;
    for p in &pts {
        acc += *p;
    }
    println!("{total} {acc} {}", total == acc);
    let mut sorted = pts.clone();
    sorted.sort_by(|p, q| p.len().partial_cmp(&q.len()).unwrap());
    println!("{:?}", sorted.iter().map(|p| p.len()).collect::<Vec<_>>());
    println!("{:?}", pts.iter().map(|p| p.len().round() as i32).collect::<Vec<_>>());
    let far = pts.iter().max_by(|p, q| p.len().total_cmp(&q.len())).unwrap();
    println!("{far} {}", pts.iter().map(|p| p.x).fold(f32::INFINITY, f32::min));
    let mut m = Matrix::identity(2);
    m[(0, 1)] = 2.0;
    m[(1, 0)] = -1.0;
    println!("{} {} {:?} {}", m.apply(a), m.apply(Vec2::ZERO), m.data, m.rows * m.cols);
    let theta = std::f32::consts::FRAC_PI_4;
    let rot = Matrix { rows: 2, cols: 2, data: vec![theta.cos(), -theta.sin(), theta.sin(), theta.cos()] };
    let r = rot.apply(Vec2::new(1.0, 0.0));
    println!("{r} {:.4} {:.1}", r.len(), r.angle());
    println!("{} {} {} {}", 1.0f32 / 3.0, 0.1f32 + 0.2f32, 16777217.0f32, 1e-8f32);
    println!("{} {} {}", (2.5f32).powi(3), 2f32.sqrt(), (-7.25f32).abs().clamp(0.0, 5.0));
    println!("{:?} {:?} {}", f32::NAN.max(1.0), 1.0f32.min(f32::NAN), f32::MAX);
    println!("{} {} {}", 3.7f32 as i32, -3.7f32 as i32, 300.0f32 as u8);
    println!("{} {} {:?}", f32::EPSILON, 1.0f32 + f32::EPSILON > 1.0, (0.1f32 as f64));
    println!("{:.3} {:e} {:?}", theta, 1234.5f32, 100000000.0f32);
}
