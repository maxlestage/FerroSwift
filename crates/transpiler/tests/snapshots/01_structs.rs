/// A point on a grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// The origin.
    pub const ORIGIN: Point = Point { x: 0, y: 0 };

    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    pub fn manhattan(&self, other: &Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn shift(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn with_x(mut self, x: i32) -> Self {
        self.x = x;
        self
    }
}
