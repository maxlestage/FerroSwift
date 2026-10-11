use std::fmt;

trait Shape {
    const SIDES: u32;
    fn name() -> String;
    fn area(&self) -> f64;
    fn scaled(&self, k: f64) -> Self
    where
        Self: Sized;
    fn describe(&self) -> String {
        format!("{} with {} sides, area {:.2}", Self::name(), Self::SIDES, self.area())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Square(f64);

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Tri {
    base: f64,
    height: f64,
}

impl Shape for Square {
    const SIDES: u32 = 4;
    fn name() -> String {
        "square".to_string()
    }
    fn area(&self) -> f64 {
        self.0 * self.0
    }
    fn scaled(&self, k: f64) -> Self {
        Square(self.0 * k)
    }
}

impl Shape for Tri {
    const SIDES: u32 = 3;
    fn name() -> String {
        "triangle".into()
    }
    fn area(&self) -> f64 {
        0.5 * self.base * self.height
    }
    fn scaled(&self, k: f64) -> Self {
        Tri { base: self.base * k, height: self.height * k }
    }
}

fn largest_area<T: Shape + Default + Copy>(shapes: &[T]) -> T {
    shapes.iter().copied().fold(T::default(), |best, s| if s.area() > best.area() { s } else { best })
}

fn total_sides<T: Shape>(count: usize) -> u32 {
    T::SIDES * count as u32
}

fn compose(f: impl Fn(i32) -> i32, g: impl Fn(i32) -> i32) -> impl Fn(i32) -> i32 {
    move |x| g(f(x))
}

fn sum_matrix(m: &[[i32; 3]; 2]) -> i32 {
    m.iter().flatten().sum()
}

struct Hex(i32);

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#06x}", self.0)
    }
}

fn main() {
    let squares = [Square(2.0), Square(5.0), Square(3.0)];
    let tris = vec![Tri { base: 2.0, height: 3.0 }, Tri { base: 10.0, height: 1.0 }];
    println!("{:?} {:?}", largest_area(&squares), largest_area(&tris));
    println!("{} | {}", squares[0].describe(), tris[1].scaled(2.0).describe());
    println!("{} {} {}", total_sides::<Square>(3), total_sides::<Tri>(2), Square::SIDES + Tri::SIDES);
    println!("{} {}", <Square as Shape>::name(), Tri::name());

    let values = [3, -7, 12, -1, 5];
    println!("{} {} {}", values.iter().copied().fold(i32::MIN, i32::max), values.iter().copied().fold(i32::MAX, i32::min), values.iter().map(|v| i32::abs(*v)).sum::<i32>());
    println!("{:?} {:?}", values.iter().copied().map(i32::abs).collect::<Vec<_>>(), [4.0f64, 9.0, 2.25].iter().copied().map(f64::sqrt).collect::<Vec<_>>());
    println!("{} {} {} {}", f64::max(1.5, -2.0), f64::min(1.5, -2.0), i32::max(3, 9), u8::pow(2, 7));
    println!("{} {} {}", i32::abs(-5), str::len("héllo"), String::len(&String::from("abc")));
    println!("{} {}", Vec::len(&values.to_vec()), Option::is_some(&Some(1)));
    println!("{}", ["a", "bb", "ccc"].iter().map(|s| s.len()).max().unwrap_or(0));
    println!("{:?}", "a1b2c3".chars().filter(char::is_ascii_digit).collect::<String>());
    println!("{:?}", ["x", " y ", "z "].iter().map(|s| s.trim()).collect::<Vec<_>>());
    println!("{:?}", ["x", "y"].iter().map(ToString::to_string).collect::<Vec<String>>());

    let add_two = |x| x + 2;
    let triple = |x| x * 3;
    let both = compose(add_two, triple);
    println!("{} {}", both(4), compose(triple, add_two)(4));
    let mut m = [[1, 2, 3], [4, 5, 6]];
    for row in m.iter_mut() {
        for cell in row.iter_mut() {
            *cell *= 2;
        }
    }
    m[1][2] = 100;
    println!("{} {:?} {}", sum_matrix(&m), m, m.len() * m[0].len());
    let [a, b, c] = [10, 20, 30];
    let (mut x, mut y) = (a, b);
    (x, y) = (y, x);
    println!("{a} {b} {c} {x} {y}");
    let doubled = [1, 2, 3].map(|v| v * 2);
    println!("{:?} {} {}", doubled, doubled.contains(&4), [1, 2, 3].into_iter().sum::<i32>());
    let arr: [u8; 4] = [1, 2, 3, 4];
    println!("{:?} {:?}", arr.iter().rev().copied().collect::<Vec<_>>(), [0u8; 3]);

    println!("{:x} {:b} {:o} {:X} {:#x} {:#010b} {:08b}", -1i32, -2i8, -8i16, i64::MIN, -255i32, 5u8, 200u8);
    println!("{:>+5}|{:^9}|{:<6}|{:+}|{:05}|{:<05}|", 42, -17, -3, 0, -42, 7);
    println!("{} {}", Hex(255), Hex(-1));
    println!("{:?} {:?} {:?} {:?}", 300i32.checked_rem(0), 7i32.checked_div(0), i32::MIN.checked_neg(), (-5i32).checked_abs());
    println!("{} {} {} {}", 10u32.abs_diff(3), 3u32.abs_diff(10), (-3i32).abs_diff(4), 1000u32.ilog10());
    println!("{} {} {} {}", 1u32.leading_zeros(), 8u8.trailing_zeros(), 0xF0u8.count_zeros(), 255u8.count_ones());
    println!("{:?} {:?} {}", 258u16.to_be_bytes(), 258u32.to_le_bytes(), u16::from_be_bytes([1, 2]));
    println!("{} {} {}", 0x1234u16.swap_bytes(), 1u32.rotate_right(1), 0x80u8.rotate_left(1));
    println!("{:?} {:?} {:?}", 'f'.to_digit(16), 'g'.to_digit(16), 'z'.to_digit(36));
    println!("{} {} {}", 'F'.is_ascii_hexdigit(), 'g'.is_ascii_hexdigit(), '9'.is_ascii_hexdigit());
    println!("{} {} {}", 17i32.isqrt(), 16u64.isqrt(), 99u8.isqrt());
}
