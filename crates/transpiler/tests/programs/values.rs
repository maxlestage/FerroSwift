#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn classify(n: i32) -> Result<&'static str, String> {
    let kind = match n {
        i32::MIN..=-1 => return Err(format!("negative: {n}")),
        0 => "zero",
        1..=9 => "digit",
        _ => "big",
    };
    Ok(kind)
}

fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let (a, b) = match s.split_once(',') {
        Some((a, b)) => (a.trim().parse().ok()?, b.trim().parse().ok()?),
        None => return None,
    };
    Some((a, b))
}

fn first_even(v: &[i32]) -> Option<i32> {
    let mut i = 0;
    let found = loop {
        if i >= v.len() {
            break None;
        }
        if v[i] % 2 == 0 {
            break Some(v[i]);
        }
        i += 1;
    };
    found
}

fn sum_valid(items: &[&str]) -> Result<i32, String> {
    let mut total = 0;
    for (i, item) in items.iter().enumerate() {
        let value = match item.parse::<i32>() {
            Ok(v) if v >= 0 => v,
            Ok(_) => continue,
            Err(e) => return Err(format!("item {i}: {e}")),
        };
        total += value;
    }
    Ok(total)
}

fn describe(p: &Point) -> String {
    let quadrant = if p.x == 0 || p.y == 0 {
        return "on an axis".to_string();
    } else if p.x > 0 {
        if p.y > 0 { 1 } else { 4 }
    } else if p.y > 0 {
        2
    } else {
        3
    };
    format!("quadrant {quadrant}")
}

fn mirror(p: Point) -> Point {
    Point {
        x: match p.x {
            0 => 0,
            x => -x,
        },
        y: {
            let doubled = p.y * 2;
            doubled - p.y
        },
    }
}

fn lengths(words: &[&str]) -> Result<Vec<usize>, String> {
    let mut out = Vec::new();
    for w in words {
        out.push(match w.len() {
            0 => return Err("empty word".into()),
            n if n > 10 => return Err(format!("too long: {w}")),
            n => n,
        });
    }
    Ok(out)
}

fn main() {
    for n in [-5, 0, 7, 42] {
        println!("{n}: {:?}", classify(n));
    }
    println!("{:?} {:?} {:?} {:?}", parse_pair("3, 4"), parse_pair("x,1"), parse_pair("nocomma"), parse_pair(" 10 ,-2"));
    println!("{:?} {:?}", first_even(&[1, 3, 8, 5]), first_even(&[1, 3]));
    println!("{:?} {:?}", sum_valid(&["1", "-4", "7"]), sum_valid(&["2", "x"]));
    for p in [Point { x: 3, y: -1 }, Point { x: -2, y: 5 }, Point { x: 0, y: 9 }, Point { x: -1, y: -1 }] {
        println!("{:?} {} {:?}", p, describe(&p), mirror(Point { x: p.x, y: p.y }));
    }
    println!("{:?} {:?} {:?}", lengths(&["ab", "c"]), lengths(&["ab", ""]), lengths(&["abcdefghijkl"]));
    let total: i32 = [1, 2, 3].iter().map(|&x| if x % 2 == 0 { x * 10 } else { x }).sum();
    let label = {
        let base = "n".repeat(2);
        format!("{base}{total}")
    };
    println!("{total} {label}");
    let mut attempts = 0;
    let outcome = loop {
        attempts += 1;
        let r = match attempts {
            1 | 2 => continue,
            3 => "third",
            _ => break "late",
        };
        if r == "third" {
            break r;
        }
    };
    println!("{outcome} {attempts}");
    let grid = vec![vec![1, 0, 3], vec![0, 5, 0]];
    let mut firsts = Vec::new();
    for row in &grid {
        let first = match row.iter().position(|&c| c != 0) {
            Some(i) => row[i],
            None => continue,
        };
        firsts.push(first);
    }
    println!("{:?}", firsts);
    let nested: Vec<i32> = (1..=6)
        .filter_map(|n| {
            let half = match n % 2 {
                0 => n / 2,
                _ => return None,
            };
            Some(half * half)
        })
        .collect();
    println!("{:?}", nested);
}
