fn main() {
    let samples = ["straße", "Ωmega", "e\u{301}", "日本語", "ǆ", "ﬁ"];
    for s in samples {
        println!("{s} {} {} {} {} [{:>6}] [{:<6}] [{:.2}] [{:^7.1}]", s.len(), s.chars().count(), s.to_uppercase(), s.to_lowercase(), s, s, s, s);
    }
    println!("{:?} {:?} {:?}", 'ß'.to_uppercase().collect::<String>(), 'ǅ'.to_lowercase().to_string(), 'İ'.to_lowercase().count());
    println!("{} {} {} {}", '²'.is_numeric(), '²'.is_ascii_digit(), '²'.to_digit(10).is_none(), 'é'.is_alphabetic());
    println!("{} {} {} {}", 'Ⅷ'.is_alphanumeric(), ' '.is_whitespace(), '\u{a0}'.is_whitespace(), '_'.is_alphanumeric());
    println!("{:?} {:?} {:?}", 'é'.escape_unicode().to_string(), "a\u{301}".escape_default().to_string(), '\u{1F600}'.len_utf8());
    let text = "one--two--three--";
    println!("{:?} {:?}", text.split("--").collect::<Vec<_>>(), text.rsplit("--").collect::<Vec<_>>());
    println!("{:?} {:?}", text.splitn(2, "--").collect::<Vec<_>>(), text.rsplitn(2, "--").collect::<Vec<_>>());
    println!("{:?} {:?}", text.matches("--").count(), text.rmatch_indices("--").map(|(i, _)| i).collect::<Vec<_>>());
    println!("{:?} {:?}", text.trim_matches(|c| c == '-' || c == 'e'), text.trim_end_matches("--"));
    println!("{:?} {:?}", "ab12cd".find(char::is_numeric), "ab12cd".rfind(|c: char| c.is_ascii_digit()));
    println!("{:?} {:?}", "prefix-body".strip_prefix("prefix-").and_then(|r| r.strip_suffix("y")), "x".strip_prefix("xy"));
    println!("{:?} {:?} {:?}", char::from_u32(0x1F600), u32::from('A'), char::from(0x41u8));
    println!("{:?} {:?} {:?}", '\u{301}', "\u{7f}\u{80}", "tab\tnl\n");
    let mixed = "Hello, Wörld! 123";
    let classes: String = mixed.chars().map(|c| match c {
        c if c.is_uppercase() => 'U',
        c if c.is_lowercase() => 'l',
        c if c.is_ascii_digit() => 'd',
        c if c.is_whitespace() => '_',
        _ => '.',
    }).collect();
    println!("{classes}");
    let title: String = mixed.split(' ').map(|w| {
        let mut cs = w.chars();
        match cs.next() {
            Some(f) => f.to_uppercase().chain(cs.flat_map(|c| c.to_lowercase())).collect(),
            None => String::new(),
        }
    }).collect::<Vec<String>>().join(" ");
    println!("{title}");
    println!("{:?}", "日本語テキスト".char_indices().filter(|(i, _)| i % 6 == 0).map(|(_, c)| c).collect::<String>());
    println!("{:?} {:?}", "ab".cmp("aB"), "é" > "z");
    println!("{}", "ab\u{301}c".chars().rev().collect::<String>().chars().count());
    println!("{:?} {:?}", "naïve".char_indices().nth(3), "naïve".bytes().nth(3));
    println!("[{:>4}] [{:<4}] [{:^5}]", 'ü', "ü", '日');
}
