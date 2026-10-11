use std::collections::HashMap;
fn main() {
    let v = vec![1, 2, 3];
    let m: HashMap<&str, i32> = HashMap::new();
    let (i, z) = (7usize, std::env::args().count() as i32 - 1);
    let k = 2;
    let s = "héllo";
    match k {
        1 => { let x = v[i]; println!("{x}"); }
        2 => { let x = 10 / z; println!("{x}"); }
        3 => { let x = 10 % z; println!("{x}"); }
        4 => { let x = &v[1..i]; println!("{x:?}"); }
        5 => { let x = &v[i..]; println!("{x:?}"); }
        6 => { let x = m["missing"]; println!("{x}"); }
        _ => { let x = &s[0..2]; println!("{x}"); }
    }
}
