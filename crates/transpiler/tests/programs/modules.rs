mod geometry {
    pub const UNIT: f64 = 1.0;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }

    impl Point {
        pub fn new(x: f64, y: f64) -> Self {
            Point { x, y }
        }

        pub fn norm(&self) -> f64 {
            helpers::hypot(self.x, self.y)
        }
    }

    pub fn midpoint(a: Point, b: Point) -> Point {
        Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
    }

    pub mod helpers {
        pub fn hypot(a: f64, b: f64) -> f64 {
            (a * a + b * b).sqrt()
        }

        pub(crate) fn clamp01(x: f64) -> f64 {
            x.clamp(0.0, 1.0)
        }
    }

    pub mod shapes {
        use super::Point;

        #[derive(Debug)]
        pub enum Shape {
            Circle(Point, f64),
            Segment(Point, Point),
        }

        impl Shape {
            pub fn length(&self) -> f64 {
                match self {
                    Shape::Circle(_, r) => 2.0 * std::f64::consts::PI * r,
                    Shape::Segment(a, b) => super::helpers::hypot(a.x - b.x, a.y - b.y),
                }
            }
        }
    }
}

mod text {
    pub fn shout(s: &str) -> String {
        format!("{}!", s.to_uppercase())
    }

    pub fn initials(name: &str) -> String {
        name.split_whitespace().filter_map(|w| w.chars().next()).collect()
    }

    pub struct Counter {
        count: usize,
    }

    impl Counter {
        pub fn new() -> Self {
            Counter { count: 0 }
        }
        pub fn feed(&mut self, s: &str) {
            self.count += s.split_whitespace().count();
        }
        pub fn count(&self) -> usize {
            self.count
        }
    }

    impl Default for Counter {
        fn default() -> Self {
            Self::new()
        }
    }
}

use geometry::shapes::Shape;
use geometry::{midpoint, Point};
use text::Counter;

const MAX_ITEMS: usize = 3;
const PI2: f64 = std::f64::consts::PI * 2.0;
static VERSION: &str = "1.2.0";
const SIZES: [u32; 3] = [10, 20, 30];

struct Settings;

impl Settings {
    const DEFAULT_NAME: &'static str = "untitled";
    const MAX: u32 = 99;

    fn describe() -> String {
        format!("{} up to {}", Self::DEFAULT_NAME, Self::MAX)
    }
}

fn helper() -> u32 {
    fn inner(x: u32) -> u32 {
        x * 2
    }
    inner(21)
}

fn main() {
    let a = Point::new(0.0, 0.0);
    let b = geometry::Point { x: 3.0, y: 4.0 };
    println!("{:?} {} {}", midpoint(a, b), b.norm(), geometry::UNIT);
    println!("{}", geometry::helpers::clamp01(1.7));
    let shapes = vec![Shape::Circle(a, 1.0), Shape::Segment(a, b)];
    for s in &shapes {
        println!("{:?} {:.3}", s, s.length());
    }
    println!("{} {}", text::shout("hey"), text::initials("Ada Lovelace King"));
    let mut c = Counter::new();
    c.feed("one two three");
    c.feed("four");
    println!("{} {}", c.count(), Counter::default().count());
    println!("{} {} {} {:?}", MAX_ITEMS, PI2, VERSION, SIZES);
    println!("{} {}", Settings::describe(), Settings::MAX + 1);
    println!("{}", helper());
    println!("{}", SIZES.iter().sum::<u32>() as usize / MAX_ITEMS);
    let version_parts: Vec<u32> = VERSION.split('.').map(|p| p.parse().unwrap()).collect();
    println!("{:?} {}", version_parts, version_parts[0] * 100 + version_parts[1]);
}
