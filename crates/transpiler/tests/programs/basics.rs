// Arithmetic, variables, shadowing, loops and tuples.

const LIMIT: u32 = 5;

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}

fn swap_pair(pair: (i32, i32)) -> (i32, i32) {
    let (a, b) = pair;
    (b, a)
}

fn increment(x: &mut i32) {
    *x += 1;
}

fn main() {
    let x = 5;
    let x = x * 2; // shadowing
    let mut total = 0;
    for i in 0..LIMIT {
        total += i;
    }
    println!("x = {x}, total = {total}");

    let mut n = 0;
    while n < 3 {
        n += 1;
    }
    let mut count = 0;
    loop {
        count += 1;
        if count == 4 {
            break;
        }
    }
    println!("n = {n}, count = {count}");

    'outer: for i in 0..5 {
        for j in 0..5 {
            if i * j == 6 {
                println!("found {i} * {j}");
                break 'outer;
            }
            if j > i {
                continue 'outer;
            }
        }
    }

    for i in (1..=10).rev().step_by(3) {
        print!("{i} ");
    }
    println!();

    let fb: Vec<String> = (1..=15).map(fizzbuzz).collect();
    println!("{}", fb.join(","));
    println!("gcd = {}", gcd(48, 180));

    let (a, b) = swap_pair((1, 2));
    println!("swapped: {a} {b}");

    let mut counter = 10;
    increment(&mut counter);
    increment(&mut counter);
    println!("counter = {counter}");

    let pi = 3.14159_f64;
    let r = 2.0;
    println!("area = {:.3}, half = {}, int = {}", pi * r * r, 1.0 / 2.0, (pi * 100.0) as i64);
    println!("float display: {} {} {} {}", 1.0, 2.5, -0.5, 1e10);
    println!("division: {} {} {}", 7 / 2, -7 / 2, 7 % 3);
    println!("bits: {} {} {} {}", 6 & 3, 6 | 3, 6 ^ 3, 1 << 4);
    println!("max: {} min: {}", i32::MAX, u8::MAX);

    let grid = [[1, 2, 3], [4, 5, 6]];
    let mut sum = 0;
    for row in grid.iter() {
        for v in row {
            sum += v;
        }
    }
    println!("grid sum = {sum}");

    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        print!("{top}");
    }
    println!();

    let t = (1, "two", 3.0);
    println!("tuple: {} {} {}", t.0, t.1, t.2);
    let flag = !(3 > 2) || 2 + 2 == 4 && true;
    println!("flag = {flag}");
    let value = if total > 5 { "big" } else { "small" };
    println!("{value}");
    let neg: i32 = -x;
    println!("neg = {neg}, abs = {}", neg.abs());
    let big: u64 = 2u64.pow(40);
    println!("pow = {big}");
    let c = 'z';
    println!("char {} code {}", c, c as u32);
    let b = 97u8 as char;
    println!("from byte {}", b);
}
