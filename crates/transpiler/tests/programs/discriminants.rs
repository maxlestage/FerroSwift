use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
enum Color {
    Red = 1,
    Green = 2,
    Blue = 4,
    Custom = 200,
}

impl Color {
    const ALL: [Color; 4] = [Color::Red, Color::Green, Color::Blue, Color::Custom];

    const fn bits(self) -> u8 {
        self as u8
    }

    fn mix(self, other: Color) -> u8 {
        self.bits() | other.bits()
    }
}

impl TryFrom<u8> for Color {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Color::ALL.iter().copied().find(|c| c.bits() == v).ok_or_else(|| format!("no color {v}"))
    }
}

impl From<Color> for u32 {
    fn from(c: Color) -> u32 {
        c as u32 * 10
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = format!("{self:?}").to_lowercase();
        write!(f, "{name}#{}", self.bits())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Trace,
    Debug,
    Info = 10,
    Warn,
    Error,
}

mod geometry {
    pub const UNIT: f64 = 1.0;
    pub static NAMES: [&str; 3] = ["point", "line", "plane"];

    pub fn dims(name: &str) -> Option<usize> {
        NAMES.iter().position(|n| *n == name)
    }

    pub mod shapes {
        use super::UNIT;

        #[derive(Debug, Default)]
        pub struct Square {
            pub side: f64,
        }

        impl Square {
            pub fn unit() -> Self {
                Square { side: UNIT }
            }

            pub fn area(&self) -> f64 {
                self.side * self.side
            }

            pub(crate) fn scaled(&self, k: f64) -> Self {
                Square { side: self.side * k }
            }
        }

        pub fn describe(s: &Square) -> String {
            format!("{} of area {}", super::NAMES[2], s.area())
        }
    }
}

use geometry::shapes::{self, Square};

fn main() {
    for c in Color::ALL {
        println!("{c} {:?} {} {} {:?}", c, c as i32, u32::from(c), Color::try_from(c.bits()).map(|x| x == c));
    }
    println!("{:?} {:?} {:?}", Color::try_from(3), Color::try_from(200u8), u8::try_from(300i32).ok().and_then(|v| Color::try_from(v).ok()));
    println!("{} {} {:08b} {}", Color::Red.mix(Color::Blue), Color::Green.mix(Color::Green), Color::Custom.mix(Color::Red), Color::Red < Color::Custom);
    let mut sorted = vec![Color::Custom, Color::Blue, Color::Red];
    sorted.sort();
    println!("{:?} {:?}", sorted, sorted.iter().map(|c| c.bits() as u32).sum::<u32>());
    println!("{} {} {} {} {}", Level::Trace as i32, Level::Debug as i32, Level::Info as i32, Level::Warn as i32, Level::Error as u8);
    println!("{} {:?}", Level::Warn > Level::Info, [Level::Error, Level::Trace, Level::Warn].iter().max());
    let levels: Vec<i32> = [Level::Trace, Level::Info, Level::Error].iter().map(|l| *l as i32).collect();
    println!("{:?} {}", levels, levels.iter().any(|&l| l == 12));
    println!("{} {:?} {:?} {:?}", geometry::UNIT, geometry::NAMES, geometry::dims("line"), geometry::dims("cube"));
    let sq = Square::unit().scaled(3.0);
    println!("{:?} {} {} {:?}", sq, sq.area(), shapes::describe(&sq), Square::default());
    println!("{}", shapes::describe(&Square { side: 0.5 }));
}
