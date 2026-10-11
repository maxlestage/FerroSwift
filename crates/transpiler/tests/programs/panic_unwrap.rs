fn f(k: u32) -> Result<i32, String> { if k > 0 { Err(format!("bad {k}")) } else { Ok(1) } }
fn main() {
    let none: Option<i32> = None;
    let v = vec![1, 2, 3];
    let i = 7;
    let k = 1;
    println!("start {}", v.len() + i * 0);
    match k {
        1 => { none.unwrap(); }
        2 => { none.expect("needed a value"); }
        3 => { f(1).unwrap(); }
        4 => { f(2).expect("loading"); }
        5 => { assert!(v.len() > 5, "too short: {}", v.len()); }
        6 => { assert_eq!(v[0] + 1, 3); }
        7 => { assert_ne!(v[0], 1, "first is {}", v[0]); }
        _ => { assert!(v.is_empty()); }
    }
}
