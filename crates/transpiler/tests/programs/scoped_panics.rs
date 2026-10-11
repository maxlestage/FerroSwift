fn main() {
    let n = std::thread::scope(|s| {
        let a = s.spawn(|| -> i32 { panic!("inner") });
        let _b = s.spawn(|| 5);
        println!("{}", a.join().is_err());
        s.spawn(|| { let v: Vec<i32> = vec![]; v[0] });
        7
    });
    println!("{n}");
}
