use std::collections::BTreeMap;

fn main() {
    let words = ["alpha", "beta", "gamma", "delta", "epsilon"];
    let pairs: Vec<(usize, char)> = words.iter().flat_map(|w| w.chars().enumerate().filter(|(_, c)| "aeiou".contains(*c))).take(6).collect();
    println!("{:?}", pairs);
    let triples: Vec<(i32, char, bool)> = (1..).zip("xyz".chars()).zip([true, false, true]).map(|((a, b), c)| (a, b, c)).collect();
    println!("{:?}", triples);
    let v = [3, 1, 4, 1, 5, 9, 2, 6];
    let rising = v.windows(2).fold(0, |acc, w| if w[1] > w[0] { acc + 1 } else { acc });
    let running: Vec<i32> = v.iter().scan((0, 0), |st, &x| { st.0 += x; st.1 += 1; Some(st.0 / st.1) }).collect();
    println!("{rising} {:?}", running);
    let head: Vec<_> = v.iter().take_while(|&&x| x < 5).collect();
    let tail: Vec<_> = v.iter().skip_while(|&&x| x < 5).collect();
    println!("{:?} {:?}", head, tail);
    println!("{:?} {:?}", "abcdefgh".chars().step_by(3).collect::<String>(), v.iter().enumerate().rev().take(2).collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", v.iter().last(), v.iter().rposition(|&x| x == 1), v.iter().rev().position(|&x| x == 1));
    let floats = [2.5f64, -1.0, 9.75, 0.5];
    println!("{:?} {:?}", floats.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()), floats.iter().cloned().min_by(|a, b| a.total_cmp(b)));
    let people = [("ann", 30, 1.70), ("bob", 25, 1.80), ("cy", 30, 1.65)];
    println!("{:?} {:?}", people.iter().min_by_key(|p| (p.1, p.0)), people.iter().max_by_key(|p| (p.1, -(p.2 * 100.0) as i32)));
    let total: f32 = [0.1f32, 0.2, 0.3].iter().sum();
    println!("{} {} {}", total, total as f64, (1..=10u64).product::<u64>());
    let (names, ages): (Vec<&str>, Vec<u32>) = people.iter().map(|p| (p.0, p.1)).unzip();
    println!("{:?} {:?}", names, ages);
    let (even, odd): (Vec<i32>, Vec<i32>) = v.iter().partition(|&&x| x % 2 == 0);
    println!("{:?} {:?}", even, odd);
    let groups: BTreeMap<usize, Vec<&str>> = words.iter().fold(BTreeMap::new(), |mut m, w| { m.entry(w.len()).or_insert_with(Vec::new).push(*w); m });
    println!("{:?}", groups);
    let mut dups = vec![(1, 'a'), (1, 'b'), (2, 'c'), (3, 'd'), (3, 'e')];
    dups.dedup_by_key(|p| p.0);
    println!("{:?} {:?}", dups, v.chunks(3).map(|c| c.iter().sum::<i32>()).collect::<Vec<_>>());
    println!("{:?} {}", v.chunks(3).collect::<Vec<_>>().concat(), "stressed".chars().rev().collect::<String>());
    let chained: Vec<i32> = v.iter().copied().chain(std::iter::once(100)).chain([200, 300]).skip(6).collect();
    println!("{:?} {:?}", chained, "ab".chars().cycle().take(5).collect::<String>());
    let powers: Vec<u32> = std::iter::successors(Some(1u32), |&x| if x < 500 { Some(x * 3) } else { None }).collect();
    println!("{:?}", powers);
    let opt_lists = vec![Some(vec![1, 2]), None, Some(vec![3])];
    println!("{:?} {:?}", opt_lists.iter().flatten().flatten().collect::<Vec<_>>(), opt_lists.iter().map(|o| o.as_ref().map(Vec::len)).collect::<Vec<_>>());
    let nested = vec![vec![1, 2], vec![], vec![3, 4, 5]];
    println!("{:?} {:?} {}", nested.iter().map(Vec::len).collect::<Vec<_>>(), nested.concat(), nested.iter().flatten().count());
    let mut it = v.iter().peekable();
    let mut runs = Vec::new();
    while let Some(&x) = it.next() {
        let mut len = 1;
        while it.next_if(|&&y| y > x).is_some() {
            len += 1;
        }
        runs.push(len);
    }
    println!("{:?}", runs);
    let idx: Vec<usize> = v.iter().enumerate().filter_map(|(i, &x)| (x > 3).then_some(i)).collect();
    println!("{:?} {:?}", idx, v.iter().map(Some).collect::<Vec<_>>().len());
    println!("{:?}", (1..=4).flat_map(|i| (1..=i).map(move |j| i * j)).filter(|x| x % 2 == 0).collect::<Vec<_>>());
    println!("{:?}", words.iter().map(|w| w.len()).rev().skip(1).step_by(2).collect::<Vec<_>>());
    println!("{:?} {:?}", v.iter().max_by_key(|&&x| x % 5), v.iter().min_by_key(|&&x| x % 5));
    println!("{}", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().concat());
    println!("{:?}", v.iter().zip(v.iter().skip(1)).filter(|(a, b)| a < b).map(|(a, b)| b - a).collect::<Vec<_>>());
}
