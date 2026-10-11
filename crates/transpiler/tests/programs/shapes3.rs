use std::any::Any;
use std::fmt;
use std::rc::Rc;

trait Shape: fmt::Debug {
    fn area(&self) -> f64;
    fn name(&self) -> String;
    fn clone_box(&self) -> Box<dyn Shape>;
    fn as_any(&self) -> &dyn Any;
    fn describe(&self) -> String {
        format!("{} of area {:.2}", self.name(), self.area())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Circle {
    r: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct Rect {
    w: f64,
    h: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        3.0 * self.r * self.r
    }
    fn name(&self) -> String {
        "circle".into()
    }
    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
    fn name(&self) -> String {
        "rect".into()
    }
    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn describe(&self) -> String {
        format!("{}x{} rect", self.w, self.h)
    }
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

struct State {
    log: Vec<String>,
    total: f64,
}

type Middleware = Box<dyn Fn(&mut State, &dyn Shape)>;

fn largest<'a>(shapes: &'a [Box<dyn Shape>]) -> Option<&'a dyn Shape> {
    shapes.iter().map(|s| s.as_ref()).max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
}

fn radius_of(shape: &dyn Shape) -> Option<f64> {
    shape.as_any().downcast_ref::<Circle>().map(|c| c.r)
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle { r: 1.0 }), Box::new(Rect { w: 2.0, h: 3.5 }), Box::new(Circle { r: 0.5 })];
    let copies = shapes.clone();
    println!("{} {}", copies.len(), copies.iter().map(|s| s.describe()).collect::<Vec<_>>().join("; "));
    println!("{:?}", largest(&shapes).map(|s| s.name()));
    println!("{:?}", shapes.iter().map(|s| radius_of(s.as_ref())).collect::<Vec<_>>());
    for s in &shapes {
        if let Some(r) = s.as_any().downcast_ref::<Rect>() {
            println!("rect {}x{} {}", r.w, r.h, *r == Rect { w: 2.0, h: 3.5 });
        } else if s.as_any().is::<Circle>() {
            println!("circle {:?}", s);
        }
    }
    let shared: Rc<dyn Shape> = Rc::new(Circle { r: 2.0 });
    let other = Rc::clone(&shared);
    println!("{} {} {}", shared.area(), other.describe(), shared.name() == other.name());

    let middlewares: Vec<Middleware> = vec![
        Box::new(|state, shape| state.log.push(format!("saw {}", shape.name()))),
        Box::new(|state, shape| state.total += shape.area()),
        Box::new(|state, shape| {
            if shape.area() > 5.0 {
                state.log.push("big".to_string());
            }
        }),
    ];
    let mut state = State { log: Vec::new(), total: 0.0 };
    for s in &shapes {
        for m in &middlewares {
            m(&mut state, s.as_ref());
        }
    }
    println!("{:?} {:.2}", state.log, state.total);
    let boxed: Box<dyn Any> = Box::new(42i32);
    println!("{:?} {:?} {}", boxed.downcast_ref::<i32>(), boxed.downcast_ref::<String>(), boxed.is::<i32>());
    let values: Vec<Box<dyn Any>> = vec![Box::new(1u8), Box::new("text"), Box::new(2.5f64)];
    for v in &values {
        if let Some(n) = v.downcast_ref::<u8>() {
            println!("u8 {n}");
        } else if let Some(s) = v.downcast_ref::<&str>() {
            println!("str {s}");
        } else {
            println!("other");
        }
    }
}
