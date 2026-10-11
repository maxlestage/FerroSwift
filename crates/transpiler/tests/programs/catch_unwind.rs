use std::panic;

fn risky(n: i32) -> i32 {
    if n > 2 {
        panic!("too big: {n}");
    }
    n * 10
}

fn main() {
    let ok = panic::catch_unwind(|| risky(1));
    println!("{:?}", ok);
    let bad = panic::catch_unwind(|| risky(5));
    println!("{}", bad.is_err());
    let v: Vec<i32> = vec![1];
    let r = std::panic::catch_unwind(|| v[3]);
    match r {
        Ok(x) => println!("got {x}"),
        Err(_) => println!("caught"),
    }
    println!("end");
}
