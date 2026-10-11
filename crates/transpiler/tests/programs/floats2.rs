use std::collections::HashSet;

fn main() {
    let xs = [3.5, f64::NAN, -1.0, f64::INFINITY, 0.0, -0.0, f64::NEG_INFINITY, 2.0];
    let mut sorted = xs.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    println!("{:?}", sorted);
    println!("{:?} {:?}", xs.iter().copied().max_by(|a, b| a.total_cmp(b)), xs.iter().copied().min_by(|a, b| a.total_cmp(b)));
    println!("{} {} {} {}", 1.0f64.to_bits(), f64::from_bits(4611686018427387904), (-0.0f64).to_bits(), 1.5f32.to_bits());
    println!("{:.6} {:.6} {:.6} {:.6} {:.6}", 1.0f64.sin(), 2.0f64.cos(), 0.5f64.exp(), 10.0f64.ln(), 3.0f64.hypot(4.0));
    println!("{:.4} {:.4} {} {} {}", 180.0f64.to_radians(), 1.0f64.to_degrees(), 2.0f64.powi(-2), 5.0f64.copysign(-1.0), (-7.25f64).fract());
    println!("{} {} {} {} {}", (0.1 + 0.2 == 0.3), f64::NAN == f64::NAN, (0.0f64 / 0.0).is_nan(), (1e300f64 * 1e300).is_finite(), (-1.0f64 / 0.0));
    println!("{} {} {} {}", i64::MAX as f64 as i64, u64::MAX as f32 as u64, (2.5f64).round() as i64, (-2.5f64).round() as i64);
    println!("{} {} {}", 16777217u32 as f32 as u32, 9007199254740993u64 as f64 as u64, 300.7f32 as u8);
    let mut it = [1, 2, 3, 10, 11, 4].iter().peekable();
    let mut small = Vec::new();
    while let Some(x) = it.next_if(|&&x| x < 5) {
        small.push(*x);
    }
    println!("{:?} {:?} {:?}", small, it.next(), it.peek());
    let v = [1, 4, 9, 16, 25, 36, 49];
    println!("{:?} {:?}", v.iter().map_while(|&x| if x < 20 { Some(x * 2) } else { None }).collect::<Vec<_>>(), v.iter().skip_while(|&&x| x < 10).take_while(|&&x| x < 40).collect::<Vec<_>>());
    let mut seen = Vec::new();
    let total: i32 = v.iter().inspect(|&&x| seen.push(x)).filter(|&&x| x % 2 == 1).sum();
    println!("{total} {:?}", seen);
    let mut words = vec!["banana", "Apple", "cherry", "apple", "Banana"];
    words.sort_by_cached_key(|w| w.to_lowercase());
    println!("{:?}", words);
    let people = [("ann", 30), ("bob", 25), ("cid", 35), ("dee", 40)];
    println!("{:?} {:?}", people.binary_search_by_key(&35, |&(_, age)| age), people.binary_search_by_key(&33, |&(_, age)| age));
    let data = [1, 2, 3, 4, 5, 6, 7];
    let chunks = data.chunks_exact(3);
    println!("{:?} {:?} {:?}", chunks.clone().map(|c| c.iter().sum::<i32>()).collect::<Vec<_>>(), chunks.remainder(), data.rchunks(3).next());
    let (names, ages): (Vec<&str>, Vec<u32>) = people.iter().cloned().unzip();
    println!("{:?} {:?} {:?}", names, ages, names.iter().zip(ages.iter()).rev().take(2).collect::<Vec<_>>());
    let text = "first line\r\nsecond\tline\n\nlast";
    println!("{:?} {:?}", text.lines().collect::<Vec<_>>(), text.split(char::is_whitespace).filter(|s| !s.is_empty()).count());
    println!("{} {} {} {}", text.is_ascii(), "héllo".is_ascii(), text.bytes().all(|b| b.is_ascii()), "abc".bytes().map(|b| b as u32).sum::<u32>());
    println!("{:?} {:?}", "héllo wörld".char_indices().filter(|(_, c)| !c.is_ascii()).collect::<Vec<_>>(), String::from_iter(['a', 'b', 'c']));
    println!("{:?} {:?} {:?}", ["x", "y", "z"].iter().map(|s| s.to_uppercase()).collect::<String>(), "a,b,,c".split(',').collect::<Vec<_>>(), "a,b,,c".split(',').filter(|s| !s.is_empty()).map(str::to_owned).collect::<Vec<String>>().concat());
    let uniq: HashSet<char> = "mississippi".chars().collect();
    let mut uniq: Vec<char> = uniq.into_iter().collect();
    uniq.sort_unstable_by(|a, b| b.cmp(a));
    println!("{:?} {}", uniq, "mississippi".chars().filter(|c| "aeiou".contains(*c)).count());
    println!("{:?} {:?}", "abc".chars().rev().enumerate().map(|(i, c)| format!("{i}{c}")).collect::<Vec<_>>(), "tic-tac-toe".splitn(2, '-').last());
    let nested = vec![vec![1, 2], vec![], vec![3]];
    println!("{:?} {:?} {:?} {}", nested.concat(), nested.iter().map(Vec::len).max(), nested.iter().flatten().rev().collect::<Vec<_>>(), nested.iter().any(Vec::is_empty));
    println!("{:?} {:?}", (1..=4).flat_map(|x| (0..x).map(move |y| x * 10 + y)).filter(|n| n % 3 == 0).collect::<Vec<_>>(), (1..=3).map(|x| x.to_string()).collect::<Vec<_>>().join("+"));
}
