/// Anything with an area and a name.
pub trait Describe {
    fn area(&self) -> f64;

    fn name(&self) -> String;

    /// A default method: every shape gets it for free.
    fn describe(&self) -> String {
        format!("{} of area {:.2}", self.name(), self.area())
    }
}

/// An enum with payloads becomes a Swift enum with associated values.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle(f64, f64, f64),
}

impl Describe for Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle(a, b, c) => {
                // Heron's formula.
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
        }
    }

    fn name(&self) -> String {
        match self {
            Shape::Circle { .. } => "circle".to_string(),
            Shape::Rectangle { width, height } if width == height => "square".to_string(),
            Shape::Rectangle { .. } => "rectangle".to_string(),
            Shape::Triangle(..) => "triangle".to_string(),
        }
    }
}

/// Generic over any `Describe`: a Swift generic constrained by the protocol.
fn largest<T: Describe>(items: &[T]) -> Option<&T> {
    let mut best: Option<&T> = None;
    for item in items {
        match best {
            Some(b) if b.area() >= item.area() => {}
            _ => best = Some(item),
        }
    }
    best
}

fn main() {
    let shapes = vec![
        Shape::Circle { radius: 1.5 },
        Shape::Rectangle { width: 2.0, height: 2.0 },
        Shape::Rectangle { width: 3.0, height: 1.0 },
        Shape::Triangle(3.0, 4.0, 5.0),
    ];
    for shape in &shapes {
        println!("{}", shape.describe());
    }
    if let Some(big) = largest(&shapes) {
        println!("Largest: {:?}", big);
    }
}
