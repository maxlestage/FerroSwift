fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn fib(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

fn is_even(n: u32) -> bool {
    if n == 0 { true } else { is_odd(n - 1) }
}

fn is_odd(n: u32) -> bool {
    if n == 0 { false } else { is_even(n - 1) }
}

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

fn first_negative(rows: &[Vec<i32>]) -> Option<(usize, usize)> {
    let mut found = None;
    'outer: for (r, row) in rows.iter().enumerate() {
        for (c, &v) in row.iter().enumerate() {
            if v < 0 {
                found = Some((r, c));
                break 'outer;
            }
        }
    }
    found
}

fn sieve(limit: usize) -> Vec<usize> {
    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    if limit >= 1 {
        is_prime[1] = false;
    }
    let mut i = 2;
    while i * i <= limit {
        if is_prime[i] {
            let mut j = i * i;
            while j <= limit {
                is_prime[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    (0..=limit).filter(|&n| is_prime[n]).collect()
}

fn binary_search(sorted: &[i32], target: i32) -> Option<usize> {
    let (mut lo, mut hi) = (0usize, sorted.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if sorted[mid] == target {
            return Some(mid);
        } else if sorted[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    None
}

fn bubble_sort(v: &mut Vec<i32>) -> usize {
    let mut swaps = 0;
    loop {
        let mut swapped = false;
        for i in 1..v.len() {
            if v[i - 1] > v[i] {
                v.swap(i - 1, i);
                swapped = true;
                swaps += 1;
            }
        }
        if !swapped {
            break;
        }
    }
    swaps
}

fn collatz_steps(mut n: u64) -> u32 {
    let mut steps = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

fn digits_sum(mut n: u32) -> u32 {
    let mut total = 0;
    while n > 0 {
        total += n % 10;
        n /= 10;
    }
    total
}

const LIMIT: usize = 30;
const NAMES: [&str; 3] = ["zero", "one", "two"];
static GREETING: &str = "hi";

fn main() {
    println!("{} {} {}", gcd(48, 18), fib(20), is_even(10));
    println!("{:?} {:?}", find_pair(&[2, 7, 11, 15], 9), find_pair(&[1, 2], 10));
    println!("{:?} {:?}", first_negative(&[vec![1, 2], vec![3, -4, 5]]), first_negative(&[vec![1]]));
    println!("{:?}", sieve(LIMIT));
    let sorted = [1, 3, 5, 7, 9, 11];
    println!("{:?} {:?} {:?}", binary_search(&sorted, 7), binary_search(&sorted, 4), binary_search(&[], 1));
    let mut v = vec![5, 1, 4, 2, 8];
    let swaps = bubble_sort(&mut v);
    println!("{:?} in {swaps} swaps", v);
    println!("{} {}", collatz_steps(27), digits_sum(98765));
    let found = loop {
        let mut n = 0;
        while n < 100 {
            n += 7;
            if n % 5 == 0 {
                break;
            }
        }
        break n * 2;
    };
    println!("{found}");
    let mut count = 0;
    let result = 'search: loop {
        for i in 0.. {
            if i * i > 50 {
                break 'search i;
            }
            count += 1;
        }
    };
    println!("{result} {count}");
    let label = if count > 5 { "many" } else { "few" };
    println!("{label}");
    let mut acc = String::new();
    for i in (1..=10).rev().step_by(3) {
        acc.push_str(&i.to_string());
        if i > 4 {
            acc.push(',');
            continue;
        }
        acc.push(';');
    }
    println!("{acc}");
    let mut total = 0;
    for (i, name) in NAMES.iter().enumerate() {
        total += i * name.len();
    }
    println!("{total} {} {}", NAMES.len(), GREETING);
    let x = {
        let a = 3;
        let b = 4;
        a * a + b * b
    };
    println!("{x}");
    let grid = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let mut diag = 0;
    for i in 0..3 {
        diag += grid[i][i];
    }
    let transposed: Vec<Vec<i32>> = (0..3).map(|c| grid.iter().map(|row| row[c]).collect()).collect();
    println!("{diag} {:?}", transposed);
    let mut matrix = vec![vec![0; 4]; 3];
    for (r, row) in matrix.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = r * 10 + c;
        }
    }
    println!("{:?}", matrix);
    let flat_sum: usize = matrix.iter().map(|r| r.iter().sum::<usize>()).sum();
    println!("{flat_sum}");
    let mut fizz = Vec::new();
    for n in 1..=15 {
        let s = match (n % 3, n % 5) {
            (0, 0) => "FizzBuzz".to_string(),
            (0, _) => "Fizz".to_string(),
            (_, 0) => "Buzz".to_string(),
            _ => n.to_string(),
        };
        fizz.push(s);
    }
    println!("{}", fizz.join(" "));
    let mut i = 10;
    while i > 0 {
        i -= 3;
        if i == 4 {
            continue;
        }
        print!("{i} ");
    }
    println!();
    let mut countdown = 3;
    let mut log = vec![];
    while countdown > 0 {
        log.push(countdown);
        countdown -= 1;
    }
    log.push(countdown);
    println!("{:?}", log);
    let nested: Vec<(i32, char)> = (1..=2).flat_map(|n| ['a', 'b'].into_iter().map(move |c| (n, c))).collect();
    println!("{:?}", nested);
    let mut shadow = 1;
    shadow += 1;
    let shadow = shadow * 10;
    let shadow = format!("{shadow}!");
    println!("{shadow}");
    let mut fact: u64 = 1;
    for n in 1..=20u64 {
        fact *= n;
    }
    println!("{fact}");
    let powers: Vec<u64> = std::iter::successors(Some(1u64), |&p| if p < 500 { Some(p * 3) } else { None }).collect();
    println!("{:?}", powers);
    let repeated: Vec<&str> = std::iter::repeat("ab").take(3).collect();
    println!("{}", repeated.concat());
    let once: Vec<i32> = std::iter::once(7).chain(std::iter::empty()).collect();
    println!("{:?}", once);
    let r = if let Some(idx) = binary_search(&sorted, 9) { idx as i32 } else { -1 };
    println!("{r}");
    let mut attempts = 0;
    let outcome = loop {
        attempts += 1;
        if attempts * attempts >= 20 {
            break Some(attempts);
        }
        if attempts > 100 {
            break None;
        }
    };
    println!("{:?}", outcome);
}
