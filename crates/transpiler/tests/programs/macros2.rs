macro_rules! make_enum {
    ($name:ident { $($variant:ident = $label:expr),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum $name {
            $($variant),+
        }

        impl $name {
            const ALL: &'static [$name] = &[$($name::$variant),+];

            fn label(&self) -> &'static str {
                match self {
                    $($name::$variant => $label),+
                }
            }

            fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),+
                }
            }
        }
    };
}

make_enum!(Color { Red = "rouge", Green = "vert", Blue = "bleu" });

macro_rules! record {
    ($name:ident { $($field:ident : $t:ty = $default:expr),* }) => {
        #[derive(Debug)]
        struct $name {
            $($field: $t),*
        }

        impl Default for $name {
            fn default() -> Self {
                $name { $($field: $default),* }
            }
        }

        impl $name {
            fn describe(&self) -> String {
                let mut parts = Vec::new();
                $( parts.push(format!("{}={:?}", stringify!($field), self.$field)); )*
                parts.join(", ")
            }
        }
    };
}

record!(Settings { width: u32 = 80, title: String = String::from("untitled"), dark: bool = false });

macro_rules! matrix {
    ($([$($x:expr),*]),*) => {
        vec![$(vec![$($x),*]),*]
    };
}

macro_rules! sum {
    ($($x:expr),*) => { 0 $(+ $x)* };
}

macro_rules! assert_close {
    ($a:expr, $b:expr) => {
        assert!(($a - $b).abs() < 1e-9, "{} != {}", $a, $b)
    };
}

macro_rules! op {
    (add $a:expr, $b:expr) => { $a + $b };
    (mul $a:expr, $b:expr) => { $a * $b };
    (neg $a:expr) => { -$a };
}

macro_rules! with_label {
    ($label:literal) => {
        concat!("[", $label, "]")
    };
}

fn main() {
    for c in Color::ALL {
        println!("{:?} {} {}", c, c.label(), c.name());
    }
    let s = Settings::default();
    println!("{:?}", s);
    println!("{}", s.describe());
    let m = matrix!([1, 2, 3], [4, 5, 6]);
    println!("{:?} {}", m, m[1][2]);
    println!("{} {}", sum!(1, 2, 3, 4), sum!());
    assert_close!(0.1f64 + 0.2, 0.3f64);
    println!("{} {} {}", op!(add 2, 3), op!(mul 4, 5), op!(neg 7));
    println!("{} {}", with_label!("ok"), stringify!(a + b * c));
    let width = 3;
    println!("{}", concat!("w", "=", 3));
    let _ = width;
}
