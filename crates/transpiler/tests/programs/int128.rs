fn factorial(n: u32) -> u128 {
    (1..=n as u128).product()
}

fn fib(n: u32) -> i128 {
    let (mut a, mut b) = (0i128, 1i128);
    for _ in 0..n {
        let t = a + b;
        a = b;
        b = t;
    }
    a
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Big(i128);

fn main() {
    println!("{}", i128::MAX);
    println!("{}", i128::MIN);
    println!("{}", u128::MAX);
    println!("{}", factorial(30));
    println!("{}", fib(150));
    let x: i128 = -170141183460469231731687303715884105728;
    let y = 170141183460469231731687303715884105727i128;
    println!("{x} {y} {}", x + y);
    let big = 12345678901234567890123456789i128;
    println!("{} {} {}", big / 7, big % 7, -big / 1000);
    println!("{:?} {:x} {:b}", Big(big), 255u128 << 100, 5i128);
    println!("{:>45} |{:<10}|", big, -3i128);
    println!("{}", big.pow(1) * 2);
    println!("{:?}", "340282366920938463463374607431768211455".parse::<u128>());
    println!("{:?}", "340282366920938463463374607431768211456".parse::<u128>().is_err());
    println!("{:?}", (-big).checked_mul(big));
    println!("{}", (big as f64) / 1e20);
    println!("{}", (u64::MAX as u128 * 3) as u64);
    println!("{}", -7i128 >> 1);
    let mut v = vec![Big(3), Big(-10), Big(big)];
    v.sort();
    println!("{:?}", v);
    let total: i128 = v.iter().map(|b| b.0).sum();
    println!("{total} {}", total.abs().count_ones());
    println!("{:?}", i128::try_from(u128::MAX).is_err());
    println!("{}", u128::from(7u8) + 1);
    println!("{}", (big as i64) as i128);
    println!("{}", big.wrapping_mul(big));
    println!("{:?}", 10i128.checked_div(0));
    println!("{}", big.rem_euclid(-1000));
}
