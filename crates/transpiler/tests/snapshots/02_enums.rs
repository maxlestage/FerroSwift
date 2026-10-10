#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle(f64),
    Rect { width: f64, height: f64 },
    Empty,
}

impl Shape {
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rect { width, height } => width * height,
            Shape::Empty => 0.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Shape::Circle(_) => "circle",
            Shape::Rect { width, height } if width == height => "square",
            Shape::Rect { .. } => "rectangle",
            Shape::Empty => "nothing",
        }
    }
}

pub enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>),
}

pub fn sum(t: &Tree) -> i32 {
    match t {
        Tree::Leaf(v) => *v,
        Tree::Node(l, r) => sum(l) + sum(r),
    }
}
