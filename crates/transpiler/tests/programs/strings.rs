fn is_palindrome(s: &str) -> bool {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let reversed: String = cleaned.chars().rev().collect();
    cleaned == reversed
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + &word[1..],
        None => String::new(),
    }
}

fn caesar(text: &str, shift: u8) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                (((c as u8 - b'a' + shift) % 26) + b'a') as char
            } else {
                c
            }
        })
        .collect()
}

fn main() {
    let s = "  Hello, World!  ";
    println!("[{}] [{}] [{}]", s.trim(), s.to_uppercase().trim(), s.to_lowercase());
    let words: Vec<&str> = "the quick  brown fox".split_whitespace().collect();
    println!("{:?} {}", words, words.len());
    let parts: Vec<&str> = "a,b,,c".split(',').collect();
    println!("{:?}", parts);
    println!("{}", "one two".replace("two", "three"));
    println!("{} {}", "rustacean".starts_with("rust"), "file.rs".ends_with(".swift"));
    println!("{:?} {:?}", "banana".find("an"), "banana".find("x"));
    println!("{} {}", is_palindrome("A man, a plan, a canal: Panama"), is_palindrome("swift"));
    println!("{}", capitalize("ferris"));
    println!("{}", caesar("hello", 3));
    let mut owned = String::from("abc");
    owned.push('d');
    owned.push_str("ef");
    owned += "g";
    println!("{} len {} empty {}", owned, owned.len(), owned.is_empty());
    let n = 42;
    let name = "Ferris";
    println!("{name} is {n} years old");
    println!("[{:>6}] [{:<6}] [{:^6}] [{:06}] [{:.1}]", n, n, n, n, 3.14159);
    println!("[{:>8}] [{:-<8}] [{:*^9}]", name, name, name);
    println!("hex {:x} {:X} {:#x} bin {:b} oct {:o}", 255, 255, 255, 5, 8);
    println!("debug {:?} {:?} {:?}", "quote\"d", 'c', Some("x"));
    let csv = "name,age\nann,31\nbob,27";
    for (i, line) in csv.lines().enumerate().skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        let age: u32 = fields[1].parse().unwrap();
        println!("{i}: {} is {}", fields[0], age + 1);
    }
    let count = "mississippi".chars().filter(|&c| c == 's').count();
    println!("s count {count}");
    let initials: String = words.iter().map(|w| w.chars().next().unwrap()).collect();
    println!("initials {initials}");
    let digits: u32 = "a1b2c3".chars().filter_map(|c| c.to_digit(10)).sum();
    println!("digit sum {digits}");
    println!("{}", "ab".repeat(3));
    let text = format!("{}-{}", "x", 1);
    println!("{text} {}", text.len());
    let slice = &"hello world"[6..];
    println!("{slice}");
    if let Some(rest) = "key=value".strip_prefix("key=") {
        println!("value is {rest}");
    }
}
