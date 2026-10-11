use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

fn main() {
    // VecDeque as a ring buffer.
    let mut ring: VecDeque<i32> = VecDeque::with_capacity(4);
    for i in 1..=6 {
        if ring.len() == 4 {
            ring.pop_front();
        }
        ring.push_back(i * 10);
    }
    ring.push_front(0);
    println!("{:?} len {} front {:?} back {:?}", ring, ring.len(), ring.front(), ring.back());
    ring.rotate_left(2);
    println!("{:?} {:?} {}", ring, ring.get(1), ring.contains(&40));
    if let Some(f) = ring.front_mut() {
        *f += 1;
    }
    ring.retain(|x| x % 20 != 0);
    let drained: Vec<i32> = ring.drain(..1).collect();
    println!("{:?} {:?} {:?}", drained, ring, ring.iter().rev().collect::<Vec<_>>());
    ring.insert(1, 99);
    let removed = ring.pop_front();
    ring.extend([7, 8]);
    ring.truncate(3);
    let contiguous = ring.make_contiguous().to_vec();
    println!("{:?} {:?} {:?}", removed, ring, contiguous);
    println!("{:?} {:?}", ring.iter().position(|&x| x == 8), ring.pop_back());

    // BTreeMap range queries.
    let mut scores: BTreeMap<u32, &str> = BTreeMap::new();
    for (k, v) in [(10, "ten"), (30, "thirty"), (20, "twenty"), (40, "forty"), (50, "fifty")] {
        scores.insert(k, v);
    }
    println!("{:?}", scores.range(15..=40).collect::<Vec<_>>());
    println!("{:?} {:?}", scores.range(..20).next(), scores.range(35..).map(|(k, _)| *k).collect::<Vec<_>>());
    println!("{:?} {:?}", scores.first_key_value(), scores.last_key_value());
    let popped = scores.pop_first();
    let tail = scores.split_off(&40);
    println!("{:?} {:?} {:?}", popped, scores, tail);
    println!("{:?}", scores.keys().rev().collect::<Vec<_>>());
    let mut set: BTreeSet<&str> = ["pear", "apple", "fig", "kiwi"].into_iter().collect();
    println!("{:?} {:?} {:?}", set.first(), set.last(), set.range("b".."l").collect::<Vec<_>>());
    println!("{} {} {:?}", set.insert("apple"), set.insert("date"), set.iter().rev().take(2).collect::<Vec<_>>());
    set.remove("fig");
    println!("{:?} {}", set, set.len());

    // HashSet algebra, printed sorted.
    let a: HashSet<i32> = (1..=6).collect();
    let b: HashSet<i32> = [4, 5, 6, 7, 8].into_iter().collect();
    let mut inter: Vec<_> = a.intersection(&b).copied().collect();
    inter.sort();
    let mut sym: Vec<_> = a.symmetric_difference(&b).copied().collect();
    sym.sort_unstable();
    let mut uni: Vec<_> = a.union(&b).copied().collect();
    uni.sort();
    let mut diff: Vec<_> = b.difference(&a).copied().collect();
    diff.sort();
    println!("{:?} {:?} {:?} {:?}", inter, sym, uni, diff);
    println!("{} {} {}", a.is_disjoint(&b), a.is_subset(&uni.iter().copied().collect()), a.contains(&3));
    let mut c = a.clone();
    c.retain(|x| x % 2 == 0);
    let mut cv: Vec<_> = c.into_iter().collect();
    cv.sort();
    println!("{:?}", cv);
    let mut counts: HashMap<char, usize> = HashMap::new();
    for ch in "mississippi".chars() {
        *counts.entry(ch).or_insert(0) += 1;
    }
    let mut cv: Vec<(char, usize)> = counts.into_iter().collect();
    cv.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    println!("{:?}", cv);

    // Strings.
    let mut s = String::from("hello world");
    s.insert_str(5, ",");
    s.insert(0, '>');
    let first = s.remove(0);
    let last = s.pop();
    println!("{s} {first} {:?} {}", last, s.len());
    s.retain(|c| c != 'l');
    println!("{s}");
    let drained: String = s.drain(..3).collect();
    println!("{drained}|{s}");
    let rest = s.split_off(3);
    println!("{s}|{rest}");
    s.extend(['a', 'b']);
    s.extend("cd".chars());
    s.truncate(5);
    println!("{s} {:?} {:?}", s.find(char::is_whitespace), "a b c".rfind(' '));
    println!("{} {} {:?}", "aaa".replacen("a", "b", 2), "x1y22z".trim_start_matches(char::is_alphabetic), "file.rs".strip_suffix(".rs"));
    let (left, right) = "key=value".split_at(3);
    println!("{left} {right} {:?}", "héllo".char_indices().rev().map(|(i, c)| format!("{i}{c}")).collect::<Vec<_>>());
    println!("{} {}", "Hello".eq_ignore_ascii_case("hELLO"), "abc".eq_ignore_ascii_case("abd"));
    let mut up = String::from("shout");
    up.make_ascii_uppercase();
    println!("{up} {} {}", "héllo".is_char_boundary(2), "héllo".is_char_boundary(3));
    let bytes = "hé".as_bytes();
    println!("{:?} {} {:?}", bytes, bytes[1], String::from_utf8(bytes.to_vec()));
    println!("{:?} {:?}", String::from_utf8(vec![0xff, 0x41]).is_err(), String::from_utf8_lossy(&[0x41, 0xff, 0x42]));
    println!("{:?} {:?} {:?}", i64::from_str_radix("-ff", 16), u8::from_str_radix("101", 2), u8::from_str_radix("102", 2).is_err());
    println!("{:?} {:?} {:?}", char::from_digit(7, 10), char::from_digit(11, 16), char::from_digit(11, 10));
    println!("{:>w$.p$}|{:<w$}|{:^w$}|", 3.14159, "ab", 7, w = 8, p = 3);
    println!("{:?}", "a,b,,c".split(',').map(|p| p.len()).collect::<Vec<_>>());
    println!("{:?} {:?}", "ab cd ef".rsplitn(2, ' ').collect::<Vec<_>>(), "a-b_c".split(['-', '_']).collect::<Vec<_>>());
    println!("{:?}", "line1\nline2\r\nline3\n".lines().collect::<Vec<_>>());
    println!("{:?} {:?}", "abc".bytes().rev().collect::<Vec<u8>>(), "x".repeat(3).chars().rev().collect::<String>());
    println!("{:?}", "The quick brown fox".split_whitespace().map(|w| w.chars().next().unwrap()).collect::<String>());
    println!("{:?}", "hello".chars().map(|c| ((c as u8 - b'a' + 1) % 26 + b'a') as char).collect::<String>());
}
