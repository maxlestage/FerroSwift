use std::fmt;
use std::ops::{Add, Index, IndexMut, Mul};

#[derive(Debug, Clone, PartialEq)]
struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    fn new(rows: usize, cols: usize) -> Matrix {
        Matrix { rows, cols, data: vec![0.0; rows * cols] }
    }

    fn identity(n: usize) -> Matrix {
        let mut m = Matrix::new(n, n);
        for i in 0..n {
            m[(i, i)] = 1.0;
        }
        m
    }

    fn transpose(&self) -> Matrix {
        let mut t = Matrix::new(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                t[(c, r)] = self[(r, c)];
            }
        }
        t
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;
    fn index(&self, (r, c): (usize, usize)) -> &f64 {
        &self.data[r * self.cols + c]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut f64 {
        &mut self.data[r * self.cols + c]
    }
}

impl<'a> Add<&'a Matrix> for &'a Matrix {
    type Output = Matrix;
    fn add(self, other: &Matrix) -> Matrix {
        let data = self.data.iter().zip(&other.data).map(|(a, b)| a + b).collect();
        Matrix { rows: self.rows, cols: self.cols, data }
    }
}

impl Mul<&Matrix> for &Matrix {
    type Output = Matrix;
    fn mul(self, other: &Matrix) -> Matrix {
        let mut out = Matrix::new(self.rows, other.cols);
        for r in 0..self.rows {
            for c in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self[(r, k)] * other[(k, c)];
                }
                out[(r, c)] = sum;
            }
        }
        out
    }
}

impl Mul<f64> for Matrix {
    type Output = Matrix;
    fn mul(mut self, k: f64) -> Matrix {
        for x in self.data.iter_mut() {
            *x *= k;
        }
        self
    }
}

impl PartialEq<f64> for Matrix {
    fn eq(&self, other: &f64) -> bool {
        self.data.iter().all(|x| x == other)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for r in 0..self.rows {
            for c in 0..self.cols {
                if c > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{:5.1}", self[(r, c)])?;
            }
            writeln!(f)?;
        }
        write!(f, "({}x{})", self.rows, self.cols)
    }
}

fn dot<T>(a: &[T], b: &[T]) -> T
where
    T: Copy + Default + Add<Output = T> + Mul<Output = T>,
{
    a.iter().zip(b).fold(T::default(), |acc, (x, y)| acc + *x * *y)
}

fn main() {
    let mut m = Matrix::new(2, 3);
    for r in 0..2 {
        for c in 0..3 {
            m[(r, c)] = (r * 3 + c) as f64;
        }
    }
    println!("{m}");
    let t = m.transpose();
    println!("{t}");
    let p = &m * &t;
    println!("{p}");
    let sum = &p + &Matrix::identity(2);
    println!("{sum}");
    let scaled = sum.clone() * 0.5;
    println!("{:?}", scaled.data);
    println!("{} {} {}", Matrix::new(2, 2) == 0.0, scaled == 0.0, scaled == sum);
    println!("{} {}", dot(&[1, 2, 3], &[4, 5, 6]), dot(&[1.5, 2.0], &[2.0, 0.25]));
    let v = [2.5, -1.0, f64::NAN, 7.25, 0.5];
    println!("{} {}", v.iter().copied().fold(f64::NAN, f64::max), v.iter().copied().fold(f64::INFINITY, f64::min));
    println!("{:?} {:?}", v.iter().copied().reduce(f64::max), [1.0f64, 2.0].iter().copied().reduce(|a, b| a * b));
    let mut sorted = v.to_vec();
    sorted.sort_by(f64::total_cmp);
    println!("{:?}", sorted);
    sorted.sort_by(|a, b| b.total_cmp(a));
    println!("{:?} {:?}", sorted, sorted.iter().position(|x| x.is_nan()));
    let means: Vec<f64> = [1.0, 2.0, 3.0, 4.0, 5.0].chunks(2).map(|c| c.iter().sum::<f64>() / c.len() as f64).collect();
    println!("{:?} {}", means, (0.1 + 0.2 - 0.3f64).abs() < f64::EPSILON);
    println!("{}", [3, 4].iter().map(|x| x * x).sum::<i32>() == dot(&[3, 4], &[3, 4]));
}
