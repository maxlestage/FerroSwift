struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct Holder {
    name: String,
    first: Guard,
    second: Guard,
}

impl Drop for Holder {
    fn drop(&mut self) {
        println!("drop holder {}", self.name);
    }
}

fn make(name: &'static str) -> Guard {
    let g = Guard(name);
    println!("made {name}");
    g
}

fn consume(g: Guard) {
    println!("consuming {}", g.0);
}

fn main() {
    let c = CustomSmartPointer { data: String::from("my stuff") };
    let d = CustomSmartPointer { data: String::from("other stuff") };
    println!("CustomSmartPointers created.");
    {
        let _a = make("a");
        let b = make("b");
        consume(b);
        println!("inner end");
    }
    let early = Guard("early");
    drop(early);
    let v = vec![Guard("v1"), Guard("v2")];
    let h = Holder { name: "h".into(), first: Guard("f1"), second: Guard("f2") };
    make("temp");
    let _ = make("ignored");
    let kept = if c.data.len() > 3 { Some(Guard("some")) } else { None };
    println!("{} {} {}", v.len(), h.name, kept.is_some());
    let mut slot = Guard("old");
    slot = Guard("new");
    println!("end of main {}", slot.0);
    let _ = d;
}
