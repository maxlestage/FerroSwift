use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

impl fmt::Display for Meters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}m", self.0)
    }
}

impl std::ops::Add for Meters {
    type Output = Meters;
    fn add(self, o: Meters) -> Meters {
        Meters(self.0 + o.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Dot,
    Line { from: (i32, i32), to: (i32, i32) },
    Text(String, Option<u8>),
}

fn split_pair(s: &str) -> ((usize, usize), char) {
    let n = s.len();
    ((n / 2, n - n / 2), s.chars().next().unwrap_or(' '))
}

fn main() {
    println!("{} {} {:?} {}", Meters(1.0) + Meters(2.345), Meters(0.005), Meters(2.5), Meters(1.0) < Meters(1.5));
    println!("{:.0} {:.0} {:.0} {:.0} {:.1} {:.1} {:.2} {:.2} {:.1}", 0.5, 1.5, 2.5, -0.5, 0.25, 0.35, 1.005, 2.675, 0.05);
    println!("{:.3} {:.3} {:.3} {:.1} {:.0}", 1.0005, 1.0015, 1234.5678, 9.95, 1e21);
    println!("{} {} {} {} {}", 1e100, f32::MAX, 0.1 + 0.2, 1.0 / 0.0, -0.0f64);
    println!("{} {} {} {} {}", 100000000000000000000.0f64, 1.5e300 * 1e10, f64::MIN_POSITIVE, 5e-324, 123456789.123456789f32);
    println!("{} {} {} {} {}", 1e15, 1e16, 1e17, 123456789012345680.0, 0.000001);
    println!("{} {} {} {}", 0.1f32, 16777217.0f32, 1.0e-7f32, 3.4028235e38f32);
    println!("{:?} {:?} {:?} {:?} {:?}", 1e15, 1e16, 1e17, 0.0001, 0.00001);
    println!("{:e} {:e} {:e} {:E} {:.2e} {:e}", 0.0, 123456.0, 0.00123, 1e100, 1234.5678, 5);
    println!("{} {} {} {} {}", i64::MAX as f64, u64::MAX as f32, 0.1f32 as f64, 255u8 as i8, (-1i8) as u8 as u32);
    println!("{} {} {} {}", (2.0f64).powf(0.5) as f32, 1e10 as f32, (16777217i32) as f32, 0.1f64 as f32);
    println!("{} {} {} {}", i64::MAX.wrapping_add(1), (i32::MAX as i64 + 1) as i32, -7i64 >> 1, -7i64 / 2);
    println!("{} {} {} {}", 7u32.pow(3), 2u64.pow(63), (-2i32).pow(3), 10i64.pow(18));
    println!("{:5}|{:<5}|{:^5}|{:>5}|{:05}|{:+05}|{:#06x}|{:08.2}|", true, 'x', "ab", 7, -42, 42, 255, -3.14159);
    println!("{:>8}|{:<8}|{:^9}|{:*>8}|{:+}|{:+.1}|{:08.3}|", "héllo", "wörld", "ünï", "ab", 5, -0.05, 2.5);
    println!("{:?} {:?} {:?} {:?} {:?}", "quote\"d", '\'', '\n', "tab\there", "üñí\u{200b}");
    println!("{:?} {:?} {:?}", "\\", "\r\n", "\u{1F600} \u{7f} \u{ad}");
    let s = "Hello, Wörld! 123";
    println!("{} {} {} {}", s.len(), s.chars().count(), s.to_uppercase(), s.to_lowercase());
    println!("{:?} {:?} {:?} {:?}", s.find("Wö"), s.find('!'), s.rfind('l'), s.find("zzz"));
    println!("{} {} {}", s.replace("l", "L"), s.replacen("l", "_", 2), s.matches('l').count());
    println!("{:?}", s.match_indices(|c: char| c.is_ascii_digit()).collect::<Vec<_>>());
    println!("{:?}", s.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect::<Vec<_>>());
    println!("{:?} {:?}", s.split_terminator('!').collect::<Vec<_>>(), "a-b_c".rsplit(['-', '_']).collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", s.chars().filter(char::is_ascii_punctuation).collect::<String>(), s.trim_end_matches(char::is_numeric), s.trim_start_matches(['H', 'e']));
    println!("{} {} {}", s.starts_with(['H', 'h']), s.ends_with("123"), s.contains(char::is_whitespace));
    println!("{:?} {:?} {:?} {:?}", s.get(0..5), s.get(7..9), s.chars().nth(8), s.char_indices().nth(9));
    println!("{:?} {:?}", s.chars().rev().take(3).collect::<String>(), s.bytes().rev().take(3).map(|b| b as char).collect::<String>());
    println!("{} {} {}", s.is_char_boundary(8), s.is_char_boundary(9), &s[..5]);
    let mut t = String::with_capacity(8);
    t.push_str("abc");
    t.insert(0, '>');
    t.insert_str(1, "--");
    t += "def";
    t = t + "!" + &"?".repeat(2);
    let popped = t.pop();
    let removed = t.remove(0);
    println!("{t} {:?} {} {}", popped, removed, t.len());
    t.truncate(5);
    t.retain(|c| c != '-');
    t.extend(['x', 'y'].iter());
    t.extend("zz".chars());
    let drained: String = t.drain(..2).collect();
    let tail = t.split_off(3);
    println!("{t:?} {drained:?} {tail:?} {}", t.is_empty());
    t.clear();
    t.make_ascii_uppercase();
    println!("{:?} {} {}", t, "MiXed".to_ascii_lowercase(), "straße".to_uppercase());
    println!("{:?} {:?} {:?} {:?}", "inf".parse::<f64>(), "NaN".parse::<f64>().map(|x| x.is_nan()), "1e3".parse::<f64>(), "+5".parse::<f64>());
    println!("{:?} {:?} {:?} {:?}", ".5".parse::<f64>(), "5.".parse::<f64>(), "1_000".parse::<f64>().is_err(), " 1".parse::<f64>().is_err());
    println!("{:?} {:?} {:?} {:?}", "true".parse::<bool>(), "True".parse::<bool>().is_err(), "x".parse::<char>(), "xy".parse::<char>().is_err());
    println!("{:?} {:?} {:?}", "+42".parse::<i32>(), "-0".parse::<u8>(), "٣".parse::<u8>().is_err());
    let bytes = s.as_bytes();
    println!("{} {} {:?} {}", bytes[0], bytes.len(), &bytes[7..10], bytes.iter().filter(|b| b.is_ascii_uppercase()).count());
    let digits: u32 = "4096".chars().map(|c| c.to_digit(10).unwrap()).sum();
    let from_bytes = String::from_utf8(vec![0x52, 0x75, 0x73, 0x74]).unwrap();
    println!("{digits} {from_bytes} {:?} {:?}", char::from_u32(0x41), char::from_u32(0xD800));
    println!("{} {} {} {}", 'é'.len_utf8(), '😀'.len_utf8(), 'é'.is_alphabetic(), '٣'.is_numeric());
    println!("{} {} {:?}", ('a'..='e').step_by(2).collect::<String>(), ('a'..'a').count(), ('x'..='z').rev().collect::<Vec<_>>());
    println!("{} {} {}", 'Z'.to_ascii_lowercase(), char::from(97u8), ('a' as u8 + 25) as char);
    let shapes = vec![Shape::Dot, Shape::Line { from: (0, 0), to: (3, -4) }, Shape::Text("hi".into(), Some(3)), Shape::Text(String::new(), None)];
    println!("{:?}", shapes);
    println!("{:#?}", shapes[1..3].to_vec());
    let nested: Option<Vec<(i32, String)>> = Some(vec![(1, "a".into()), (2, "b\"c".into())]);
    println!("{:?}", nested);
    println!("{:#?}", nested.as_ref().map(|v| &v[..1]));
    let ((a, b), c) = split_pair("hello");
    let pairs = [(1, "b"), (1, "a"), (0, "z")];
    println!("{a} {b} {c} {} {} {:?}", (1, "b") < (1, "c"), (2, 0) > (1, 9), pairs.iter().max());
    let (evens, odds): (Vec<i32>, Vec<i32>) = (1..=9).partition(|x| x % 2 == 0);
    let (xs, ys): (Vec<i32>, Vec<char>) = vec![(1, 'a'), (2, 'b')].into_iter().unzip();
    println!("{:?} {:?} {:?} {:?}", evens, odds, xs, ys);
    let data = [3.5, -1.0, 2.25, 9.0];
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    println!("{:?} {:?} {:?}", sorted, data.iter().cloned().fold(f64::NEG_INFINITY, f64::max), data.iter().max_by(|a, b| a.total_cmp(b)));
    println!("{:?} {:?} {}", data.iter().map(|x| x.round() as i64).collect::<Vec<_>>(), data.iter().position(|&x| x < 0.0), data.iter().sum::<f64>() / data.len() as f64);
    let v: Vec<i32> = (1..=10).collect();
    println!("{:?} {:?} {:?}", v.binary_search(&11), v.binary_search(&0), v.partition_point(|&x| x < 4));
    println!("{:?} {:?} {}", v.rchunks(3).map(|c| c.len()).collect::<Vec<_>>(), v.split_at(3).0, v.windows(2).all(|w| w[0] <= w[1]));
    let mut arr = [0u8; 6];
    arr[..3].copy_from_slice(&[1, 2, 3]);
    arr.swap(0, 5);
    println!("{:?} {:?} {:?}", arr, [[1, 2], [3, 4]].concat(), [vec![1], vec![2, 3]].join(&0));
    println!("{:?} {:?}", v.iter().cycle().skip(8).take(5).collect::<Vec<_>>(), v.iter().rev().enumerate().find(|&(_, &x)| x % 4 == 0));
    println!("{:?} {:?} {:?}", v.iter().nth(3), v.iter().last(), v.iter().skip(2).take(3).rev().collect::<Vec<_>>());
    println!("{:?}", v.iter().scan(1i64, |acc, &x| { *acc *= x as i64; Some(*acc) }).take_while(|&p| p < 1000).collect::<Vec<_>>());
}
