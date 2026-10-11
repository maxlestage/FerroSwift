use std::collections::BTreeSet;
use std::fmt;

struct Meters(f64);

impl fmt::Display for Meters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)?;
        f.write_str("m")
    }
}

struct Wrapper(String);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

struct Count(u32);

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(w) = f.width() {
            write!(f, "{}#{}", self.0, w)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

struct Id(i64);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let digits = self.0.unsigned_abs().to_string();
        f.pad_integral(self.0 >= 0, "#", &digits)
    }
}

#[derive(Debug)]
struct Inner {
    a: i32,
}

struct Outer {
    inner: Inner,
    label: &'static str,
}

impl fmt::Debug for Outer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Outer").field("inner", &self.inner).field("label", &self.label).finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct Holder {
    outer: Outer,
    list: Vec<Outer>,
}

struct Tags(BTreeSet<&'static str>);

impl fmt::Debug for Tags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.0.iter()).finish()
    }
}

struct Registry(Vec<(u8, String)>);

impl fmt::Debug for Registry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut m = f.debug_map();
        for (k, v) in &self.0 {
            m.key(k).value(v);
        }
        m.finish()
    }
}

enum Level {
    Low,
    High,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Level::Low => "low",
            Level::High => "HIGH",
        };
        f.pad(s)
    }
}

struct Flags(u8);

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", if f.sign_plus() { "+" } else { "" }, self.0)?;
        if f.alternate() {
            write!(f, " (fill {:?})", f.fill())?;
        }
        Ok(())
    }
}

struct Empty;

impl fmt::Debug for Empty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Empty").finish()
    }
}

struct Unit;

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Unit").finish()
    }
}

struct Single(i32);

impl fmt::Debug for Single {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("").field(&self.0).finish()
    }
}

fn main() {
    let m = Meters(2.5);
    println!("[{m}] [{m:>8}] [{m:<8.2}] [{m:+}] [{m:08.1}]");
    let w = Wrapper("hi".to_string());
    println!("[{w}] [{w:>5}] [{w:-<5}] [{w:.1}]");
    let c = Count(3);
    println!("[{c}] [{c:7}] [{c:>2}]");
    for id in [Id(42), Id(-7)] {
        println!("[{id}] [{id:+}] [{id:#}] [{id:>6}] [{id:06}] [{id:#08}] [{id:<5}]");
    }
    let o = Outer { inner: Inner { a: 1 }, label: "x" };
    println!("{o:?}");
    println!("{o:#?}");
    let h = Holder { outer: Outer { inner: Inner { a: 2 }, label: "y" }, list: vec![Outer { inner: Inner { a: 3 }, label: "z" }] };
    println!("{h:?}");
    println!("{h:#?}");
    let t = Tags(BTreeSet::from(["b", "a"]));
    println!("{t:?} {t:#?}");
    let r = Registry(vec![(2, "two".into()), (1, "one".into())]);
    println!("{r:?}");
    println!("{r:#?}");
    println!("[{}] [{:>6}] [{:^7}] [{:.2}]", Level::Low, Level::High, Level::Low, Level::High);
    println!("[{}] [{:+}] [{:#}] [{:+#}] [{:*^#9}]", Flags(1), Flags(2), Flags(3), Flags(4), Flags(5));
    println!("{:?} {:#?} {:?} {:#?} {:?} {:#?}", Empty, Empty, Unit, Unit, Single(5), Single(6));
    println!("{:?}", vec![Single(1), Single(2)]);
    println!("{:#?}", Some(Outer { inner: Inner { a: 9 }, label: "n" }));
    println!("{:5?}|{:<4?}", Outer { inner: Inner { a: 7 }, label: "w" }, Single(8));
}
