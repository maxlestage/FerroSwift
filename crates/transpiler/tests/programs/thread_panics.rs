use std::thread;
fn main() {
    let h = thread::spawn(|| {
        let v: Vec<i32> = Vec::new();
        println!("in thread");
        v[3]
    });
    let r = h.join();
    println!("joined: {}", r.is_err());
    match r {
        Ok(v) => println!("ok {v}"),
        Err(e) => println!("err {:?}", e),
    }
    let h2 = thread::spawn(|| -> i32 { panic!("second {}", 2) });
    let r2 = h2.join();
    println!("{:?}", r2.is_ok());
    let total: i32 = thread::scope(|s| {
        let a = s.spawn(|| 1);
        let b = s.spawn(|| 2);
        a.join().unwrap() + b.join().unwrap()
    });
    println!("{total}");
    let h3 = thread::spawn(|| -> i32 { None::<i32>.unwrap() });
    h3.join().unwrap();
}
