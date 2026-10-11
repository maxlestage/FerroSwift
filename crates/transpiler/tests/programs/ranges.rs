fn sieve(n: usize) -> Vec<usize> {
    let mut is_prime = vec![true; n + 1];
    is_prime[0] = false;
    if n >= 1 {
        is_prime[1] = false;
    }
    let mut i = 2;
    while i * i <= n {
        if is_prime[i] {
            let mut j = i * i;
            while j <= n {
                is_prime[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    (0..=n).filter(|&k| is_prime[k]).collect()
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lower_bound(v: &[i32], target: i32) -> usize {
    let (mut lo, mut hi) = (0, v.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if v[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

fn spiral(n: usize) -> Vec<Vec<u32>> {
    let mut m = vec![vec![0u32; n]; n];
    let (mut top, mut left) = (0usize, 0usize);
    let (mut bottom, mut right) = (n, n);
    let mut k = 1;
    while top < bottom && left < right {
        for c in left..right {
            m[top][c] = k;
            k += 1;
        }
        top += 1;
        for r in top..bottom {
            m[r][right - 1] = k;
            k += 1;
        }
        right -= 1;
        if top < bottom {
            for c in (left..right).rev() {
                m[bottom - 1][c] = k;
                k += 1;
            }
            bottom -= 1;
        }
        if left < right {
            for r in (top..bottom).rev() {
                m[r][left] = k;
                k += 1;
            }
            left += 1;
        }
    }
    m
}

fn two_sum(v: &[i32], target: i32) -> Option<(usize, usize)> {
    let (mut i, mut j) = (0, v.len().checked_sub(1)?);
    while i < j {
        let s = v[i] + v[j];
        if s == target {
            return Some((i, j));
        } else if s < target {
            i += 1;
        } else {
            j -= 1;
        }
    }
    None
}

fn main() {
    println!("{:?} {:?} {:?}", sieve(30), sieve(1), sieve(2));
    println!("{} {} {}", gcd(48, 18), gcd(7, 0), 48 * 18 / gcd(48, 18));
    let v = [1, 3, 3, 5, 8];
    println!("{} {} {} {}", lower_bound(&v, 3), lower_bound(&v, 4), lower_bound(&v, 0), lower_bound(&v, 9));
    for row in spiral(4) {
        println!("{}", row.iter().map(|x| format!("{x:>3}")).collect::<String>());
    }
    println!("{:?} {:?}", spiral(1), spiral(0));
    println!("{:?} {:?} {:?}", two_sum(&[1, 2, 4, 7, 11], 15), two_sum(&[1, 2], 10), two_sum(&[], 0));
    let (a, b) = (5, 2);
    println!("{} {} {} {:?}", (a..b).count(), (b..=a).sum::<i32>(), (a..b).len(), (a..b).max());
    for i in a..b {
        println!("never {i}");
    }
    for i in (b..a).rev().step_by(2) {
        print!("{i} ");
    }
    println!();
    for i in (1..=7).rev().step_by(3) {
        print!("{i} ");
    }
    println!();
    let n: i32 = -3;
    println!("{} {} {}", (0..n).count(), (n..0).count(), (n..=n).count());
    println!("{} {}", (a..b).contains(&3), (b..a).contains(&3));
    let prefix: Vec<i32> = v.iter().scan(0, |acc, &x| { *acc += x; Some(*acc) }).collect();
    println!("{:?} {}", prefix, prefix[3] - prefix[0]);
    let mut total = 0;
    'outer: for i in 1..=10 {
        for j in i..=10 {
            if i * j > 20 {
                continue 'outer;
            }
            if i + j == 13 {
                break 'outer;
            }
            total += i * j;
        }
    }
    println!("{total}");
    let hi = v.len();
    let lo = hi.saturating_sub(10);
    println!("{:?} {:?}", &v[lo..hi], (lo..hi).map(|i| v[i] * 2).collect::<Vec<_>>());
    let mut countdown = Vec::new();
    for i in (0..5).rev() {
        countdown.push(i);
    }
    println!("{:?} {:?}", countdown, (0..5).rev().collect::<Vec<_>>() == countdown);
    let digits: u32 = (1..=5).map(|d| d * d).filter(|s| s % 2 == 1).sum();
    println!("{digits} {}", (1..=20).step_by(5).map(|x| x.to_string()).collect::<Vec<_>>().join(","));
}
