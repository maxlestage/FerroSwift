use std::collections::HashMap;

fn fib_memo(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
    if n < 2 {
        return n;
    }
    if let Some(&v) = memo.get(&n) {
        return v;
    }
    let v = fib_memo(n - 1, memo).wrapping_add(fib_memo(n - 2, memo));
    memo.insert(n, v);
    v
}

fn grid_paths(r: usize, c: usize, memo: &mut HashMap<(usize, usize), u64>) -> u64 {
    if r == 0 || c == 0 {
        return 1;
    }
    if let Some(&v) = memo.get(&(r, c)) {
        return v;
    }
    let v = grid_paths(r - 1, c, memo) + grid_paths(r, c - 1, memo);
    memo.insert((r, c), v);
    v
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 0..=a.len() {
        dp[i][0] = i;
    }
    for j in 0..=b.len() {
        dp[0][j] = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1).min(dp[i][j - 1] + 1).min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[a.len()][b.len()]
}

fn lcs(a: &str, b: &str) -> String {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            dp[i][j] = if a[i] == b[j] { dp[i + 1][j + 1] + 1 } else { dp[i + 1][j].max(dp[i][j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = String::new();
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            out.push(a[i]);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

fn coin_change(coins: &[usize], target: usize) -> Option<usize> {
    let mut best = vec![usize::MAX; target + 1];
    best[0] = 0;
    for t in 1..=target {
        for &c in coins {
            if c <= t && best[t - c] != usize::MAX {
                best[t] = best[t].min(best[t - c] + 1);
            }
        }
    }
    if best[target] == usize::MAX { None } else { Some(best[target]) }
}

fn knapsack(items: &[(usize, u32)], cap: usize) -> u32 {
    let mut dp = vec![0u32; cap + 1];
    for &(w, v) in items {
        for c in (w..=cap).rev() {
            dp[c] = dp[c].max(dp[c - w] + v);
        }
    }
    dp[cap]
}

fn permutations(items: &mut Vec<i32>, k: usize, out: &mut Vec<Vec<i32>>) {
    if k == items.len() {
        out.push(items.clone());
        return;
    }
    for i in k..items.len() {
        items.swap(k, i);
        permutations(items, k + 1, out);
        items.swap(k, i);
    }
}

fn queens(n: usize, row: usize, cols: &mut Vec<usize>, count: &mut usize, first: &mut Option<Vec<usize>>) {
    if row == n {
        *count += 1;
        if first.is_none() {
            *first = Some(cols.clone());
        }
        return;
    }
    for c in 0..n {
        let ok = cols.iter().enumerate().all(|(r, &cc)| cc != c && (row - r) != c.abs_diff(cc));
        if ok {
            cols.push(c);
            queens(n, row + 1, cols, count, first);
            cols.pop();
        }
    }
}

fn main() {
    let mut memo = HashMap::new();
    println!("{} {} {}", fib_memo(10, &mut memo), fib_memo(90, &mut memo), fib_memo(93, &mut memo));
    println!("{} {}", fib_memo(100, &mut memo), memo.len());
    let mut paths = HashMap::new();
    println!("{} {} {}", grid_paths(2, 2, &mut paths), grid_paths(10, 10, &mut paths), paths.len());
    println!("{} {} {} {}", edit_distance("kitten", "sitting"), edit_distance("", "abc"), edit_distance("flaw", "lawn"), edit_distance("héllo", "hello"));
    println!("{:?} {:?} {:?}", lcs("ABCBDAB", "BDCABA"), lcs("abc", "xyz"), lcs("banana", "atana"));
    println!("{:?} {:?} {:?}", coin_change(&[1, 5, 10, 25], 63), coin_change(&[2], 7), coin_change(&[3, 7], 0));
    println!("{} {}", knapsack(&[(1, 1), (3, 4), (4, 5), (5, 7)], 7), knapsack(&[], 10));
    let mut perms = Vec::new();
    permutations(&mut vec![1, 2, 3], 0, &mut perms);
    println!("{} {:?} {:?}", perms.len(), perms.first(), perms.last());
    let subsets: Vec<Vec<char>> = (0..(1u32 << 3)).map(|mask| "abc".chars().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, c)| c).collect()).collect();
    println!("{:?}", subsets.iter().map(|s| s.iter().collect::<String>()).collect::<Vec<_>>());
    for n in [4, 6, 8] {
        let (mut count, mut first) = (0, None);
        queens(n, 0, &mut Vec::new(), &mut count, &mut first);
        println!("{n}: {count} {:?}", first);
    }
}
