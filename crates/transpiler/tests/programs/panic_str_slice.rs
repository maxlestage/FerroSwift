fn main() {
    let s = String::from("héllo");
    let n = std::env::args().count();
    let k = 1;
    match k {
        1 => println!("{}", &s[0..n + 1]),
        2 => println!("{}", &s[n + 1..]),
        3 => println!("{}", &s[..n + 20]),
        4 => println!("{}", &s[n + 3..n + 1]),
        _ => println!("{}", &s[n + 1..n + 3]),
    }
}
