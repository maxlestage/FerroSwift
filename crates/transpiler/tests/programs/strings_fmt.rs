fn main() {
    let s = "  Hello, Wörld! 123 héllo  ";
    let t = s.trim();
    println!("[{}] [{}] [{}]", t, s.trim_start(), s.trim_end());
    println!("{:?}", t.char_indices().filter(|(_, c)| c.is_uppercase()).collect::<Vec<_>>());
    println!("{:?} {:?}", t.find('o'), t.rfind('o'));
    println!("{:?}", t.splitn(2, ' ').collect::<Vec<_>>());
    println!("{:?}", t.rsplitn(2, ' ').collect::<Vec<_>>());
    println!("{:?} {:?}", t.strip_prefix("Hello"), t.strip_suffix("xyz"));
    println!("{}", t.trim_start_matches("Hel").trim_end_matches('o'));
    println!("{:?}", t.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect::<Vec<_>>());
    println!("{}", t.replace("l", "L").replacen('L', "_", 2));
    println!("{} {}", t.to_lowercase(), t.to_uppercase());
    println!("{}", t.chars().rev().collect::<String>());
    println!("{:?}", t.bytes().take(5).collect::<Vec<u8>>());
    println!("{} {}", t.len(), t.chars().count());
    println!("{:?}", t.split_once(", "));
    println!("{}", "abc".repeat(3));
    println!("{:?}", "a,b,,c".split(',').collect::<Vec<_>>());
    println!("{:?}", "line1\nline2\r\nline3".lines().collect::<Vec<_>>());
    println!("{}", t.starts_with(char::is_alphabetic));
    println!("{:?}", t.matches('l').count());
    println!("{:?}", "Mary had a little lamb".split_whitespace().map(|w| w.len()).collect::<Vec<_>>());
    let n = 255;
    println!("{:+} {:#x} {:#X} {:#b} {:#o} {:08.3} {:>+7.1}", n, n, n, 5, 8, 3.14159, 2.5);
    println!("{:^9}|{:-^9}|{:*<5}|{:>5}", "mid", "x", 7, true);
    println!("{:5}|{:<5}|{:e}|{:10.3e}", 42, 'c', 0.00012, 123456.0);
    let w = 12;
    let p = 3;
    println!("{:w$.p$}|{:>1$}|{:.*}", 2.0f64.sqrt(), 8, 2, 1.23456);
    println!("{0} {1} {0} {name}", "a", "b", name = "n");
    println!("{:?} {:?} {:?}", 'a', "q\"uote\n", '\u{1F600}');
    println!("{}", format!("{:>width$}", "r", width = 4));
    let v = vec![1.5, -0.0, f64::NAN, f64::INFINITY, 1e-7, 1e16];
    println!("{:?}", v);
    for x in &v {
        print!("{} ", x);
    }
    println!();
}
