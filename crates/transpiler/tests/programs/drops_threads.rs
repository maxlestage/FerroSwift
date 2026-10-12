use std::thread;

struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let g = Guard("worker");
    let h = thread::spawn(move || {
        println!("in thread {}", g.0);
        7
    });
    println!("joined {}", h.join().unwrap());
    let a = Guard("a");
    let b = Guard("b");
    thread::scope(|s| {
        // Printed before the spawn: two threads printing at once have no fixed order.
        println!("b is {}", b.0);
        s.spawn(move || println!("scoped {}", a.0));
    });
    println!("end");
}
