use std::env;

fn main() {
    match env::var("FERRO_SURELY_UNSET_VAR") {
        Ok(v) => println!("set {v}"),
        Err(e) => println!("unset: {e} / {:?}", e),
    }
    let home_set = env::var("PATH").is_ok();
    println!("{home_set}");
    let n: i32 = -17;
    println!("{} {} {} {}", n.abs(), n.signum(), n.rem_euclid(5), n.div_euclid(5));
    println!("{} {} {}", 7u32.pow(3), 2i64.pow(40), 10u8.saturating_sub(20));
    println!("{} {}", 255u8.wrapping_add(3), 100i8.checked_add(100).is_none());
    println!("{:?} {:?}", 5u32.checked_sub(7), 5i32.overflowing_sub(7));
    println!("{} {} {}", 0b1011u8.count_ones(), 16u32.trailing_zeros(), 1u64.leading_zeros());
    println!("{} {}", i32::from_str_radix("-ff", 16).unwrap(), u8::from_str_radix("101", 2).unwrap());
    println!("{} {}", 17i32.abs_diff(-3), 3u8.abs_diff(10));
    println!("{} {} {}", (2.5f64).round(), (-2.5f64).round(), 2.5f64.floor() as i32);
    println!("{} {:.3} {}", f64::EPSILON > 0.0, (2.0f64).ln(), (1e300 * 1e10f64).is_infinite());
    println!("{} {}", i64::MAX.to_string().len(), u64::MAX);
}
