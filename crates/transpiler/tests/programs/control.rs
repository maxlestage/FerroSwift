fn find_pair(nums: &[i32], target: i32) -> Option<(usize, usize)> {
    for i in 0..nums.len() {
        for j in (i + 1)..nums.len() {
            if nums[i] + nums[j] == target {
                return Some((i, j));
            }
        }
    }
    None
}

fn collatz_steps(mut n: u64) -> u32 {
    let mut steps = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

fn describe(n: i32) -> String {
    let kind: &str = match n {
        x if x < 0 => return format!("{n} is negative"),
        0 => "zero",
        x if x % 2 == 0 => "even",
        _ => "odd",
    };
    let size = {
        let abs = n.abs();
        if abs > 100 { "large" } else { "small" }
    };
    format!("{n} is {size} and {kind}")
}

fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

fn main() {
    println!("{:?} {:?}", find_pair(&[1, 4, 6, 9], 10), find_pair(&[1, 2], 10));
    println!("collatz(27) = {}", collatz_steps(27));
    for n in [-3, 0, 8, 7, 250] {
        println!("{}", describe(n));
    }
    println!("[{}] [{}]", first_word("hello world"), first_word("solo"));

    let mut grid = vec![vec![0; 4]; 3];
    'rows: for (r, row) in grid.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            if r * c > 3 {
                continue 'rows;
            }
            *cell = r * 10 + c;
        }
    }
    println!("{:?}", grid);

    let mut stack = vec![(0, "root")];
    let mut visited = Vec::new();
    while let Some((depth, name)) = stack.pop() {
        visited.push(format!("{}{}", "-".repeat(depth), name));
        if depth < 2 {
            stack.push((depth + 1, "left"));
            stack.push((depth + 1, "right"));
        }
    }
    println!("{}", visited.join(" "));

    let x = 5;
    {
        let x = x * 2;
        println!("inner x = {x}");
    }
    println!("outer x = {x}");
    let mut fib = vec![0u64, 1];
    while fib.len() < 15 {
        let n = fib[fib.len() - 1] + fib[fib.len() - 2];
        fib.push(n);
    }
    println!("{:?}", &fib[10..]);
    let mut i = 0;
    let found = loop {
        i += 7;
        if i % 5 == 0 {
            break i;
        }
    };
    println!("found {found}");
}
