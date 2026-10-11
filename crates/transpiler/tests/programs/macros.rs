use std::collections::HashMap;

macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}

macro_rules! max_of {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let a = $x;
        let b = max_of!($($rest),+);
        if a > b { a } else { b }
    }};
}

macro_rules! hashmap {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut m = HashMap::new();
        $( m.insert($k, $v); )*
        m
    }};
}

macro_rules! point_struct {
    ($name:ident, $t:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct $name {
            x: $t,
            y: $t,
        }

        impl $name {
            fn new(x: $t, y: $t) -> Self {
                $name { x, y }
            }
            fn sum(&self) -> $t {
                self.x + self.y
            }
        }
    };
}

point_struct!(PointI, i32);
point_struct!(PointF, f64);

macro_rules! log {
    ($level:literal, $($arg:tt)*) => {
        println!("[{}] {}", $level, format!($($arg)*))
    };
}

macro_rules! count {
    () => { 0usize };
    ($head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}

macro_rules! check {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            println!("check failed: {}", $msg);
        }
    };
}

macro_rules! getter {
    ($field:ident: $t:ty) => {
        fn $field(&self) -> $t {
            self.$field.clone()
        }
    };
}

struct User {
    name: String,
    age: u32,
}

impl User {
    getter!(name: String);
    getter!(age: u32);
}

fn main() {
    println!("{} {} {}", square!(7), square!(2 + 3), square!(1.5));
    println!("{}", max_of!(3, 9, 4, 1));
    println!("{}", max_of!(5));
    let m = hashmap! { "a" => 1, "b" => 2, "c" => 3, };
    let mut keys: Vec<_> = m.iter().collect();
    keys.sort();
    println!("{:?}", keys);
    let p = PointI::new(2, 3);
    let q = PointF::new(0.5, 0.25);
    println!("{:?} {} {:?} {}", p, p.sum(), q, q.sum());
    log!("INFO", "{} items at {:.1}%", 3, 99.5);
    log!("WARN", "plain");
    println!("{}", count!(a b c d e));
    check!(1 + 1 == 2, "math");
    check!(square!(3) == 10, format!("square was {}", square!(3)));
    let u = User { name: "ann".into(), age: 40 };
    println!("{} {}", u.name(), u.age());
    let v = vec![square!(1), square!(2), max_of!(1, 2)];
    println!("{:?}", v);
    macro_rules! local_double {
        ($e:expr) => { $e * 2 };
    }
    println!("{}", local_double!(21));
}
