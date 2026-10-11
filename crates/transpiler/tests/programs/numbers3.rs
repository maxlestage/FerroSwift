fn show_f64(label: &str, x: f64) {
    println!("{label}: {} | {:?} | {:.3} | {:e}", x, x, x, x);
}

fn show_f32(label: &str, x: f32) {
    println!("{label}: {} | {:?}", x, x);
}

fn parse_report<T: std::str::FromStr + std::fmt::Debug>(kind: &str, inputs: &[&str]) {
    let results: Vec<String> = inputs
        .iter()
        .map(|s| match s.parse::<T>() {
            Ok(v) => format!("{:?}", v),
            Err(_) => "err".to_string(),
        })
        .collect();
    println!("{kind}: {}", results.join(" "));
}

fn main() {
    show_f64("one", 1.0);
    show_f64("tenth", 0.1);
    show_f64("sum", 0.1 + 0.2);
    show_f64("big", 1e21);
    show_f64("big15", 1e15);
    show_f64("big16", 1e16);
    show_f64("big17", 123456789012345680.0);
    show_f64("small", 1e-7);
    show_f64("small5", 0.00001);
    show_f64("small4", 0.0001);
    show_f64("neg", -2.5);
    show_f64("negzero", -0.0);
    show_f64("third", 1.0 / 3.0);
    show_f64("max", f64::MAX);
    show_f64("minpos", f64::MIN_POSITIVE);
    show_f64("huge", 1.5e300);
    show_f64("pi", std::f64::consts::PI);
    show_f64("half", 0.5);
    show_f64("2.5", 2.5);
    show_f64("1.005", 1.005);
    show_f64("1e100", 1e100);
    println!("{} {} {}", f64::NAN, f64::INFINITY, -f64::INFINITY);
    println!("{:?} {:?} {:?}", f64::NAN, f64::INFINITY, -f64::INFINITY);
    println!("{:.2} {:.0} {:.0} {:.0} {:.0}", f64::NAN, 0.5, 1.5, 2.5, -0.5);
    println!("{:.1} {:.1} {:.1} {:.2} {:.2}", 0.25, 0.35, 0.45, 1.005, 2.675);
    println!("{:8.3}|{:<8.3}|{:^8.3}|{:+.1}|{:08.2}|{:+08.2}", 3.14159, 3.14159, 3.14159, 2.0, -3.14159, 3.14159);
    println!("{:e} {:E} {:e} {:e} {:e}", 1234.5, 1234.5, 0.00012, 0.0, 100.0);
    println!("{:.2e} {:10.3e}|", 123456.0, 0.000123456);
    show_f32("f32 tenth", 0.1);
    show_f32("f32 third", 1.0 / 3.0);
    show_f32("f32 big", 16777217.0);
    show_f32("f32 small", 1e-10);
    show_f32("f32 max", f32::MAX);
    println!("{} {}", 0.1f32 as f64, 0.1f64 as f32);
    println!("{} {} {}", 16777217i32 as f32, 9007199254740993i64 as f64, u64::MAX as f64);

    println!("casts: {} {} {} {} {}", -1i32 as u32, 300i32 as u8, -129i32 as i8, 255u8 as i8, 65u8 as char);
    println!("casts: {} {} {} {} {}", 3.99f64 as i32, -3.99f64 as i32, f64::NAN as i32, 1e20 as i32, -1e20 as i64);
    println!("casts: {} {} {} {}", 300.7f64 as u8, -5.0f64 as u8, 'é' as u32, 'é' as u8);
    println!("casts: {} {} {}", i64::MAX as i32, u32::MAX as i32, (1u64 << 40) as u16);
    println!("casts: {} {} {}", true as i32, false as u8, 'A' as u8 + 2);
    println!("casts: {} {} {}", 200u8 as char, char::from(97u8), char::from_u32(0x1F600).map(|c| c.len_utf8()).unwrap_or(0));

    parse_report::<i32>("i32", &["42", "+5", "-0", " 5", "5 ", "5.0", "1e3", "0x10", "", "-", "+", "2147483647", "-2147483648", "007", "٣", "١٢"]);
    println!("{:?} {:?} {:?} {:?}", "2147483648".parse::<i32>().is_err(), "-2147483649".parse::<i32>().is_err(), "256".parse::<u8>().is_err(), "-1".parse::<u8>().is_err());
    println!("{:?} {:?} {:?}", "4294967295".parse::<u32>(), "4294967296".parse::<u32>().is_err(), "-32768".parse::<i16>());
    parse_report::<u8>("u8", &["0", "255", "256", "-1", "+0", "1_0"]);
    parse_report::<f64>("f64", &["1e3", "inf", "-inf", "NaN", "nan", ".5", "5.", "1_000", "0x10", "+1.5", " 1", "1e", "e1", "infinity", "1.5E2", "-0.0"]);
    parse_report::<bool>("bool", &["true", "false", "True", "1", ""]);
    parse_report::<char>("char", &["a", "ab", "", "é", "😀"]);
    parse_report::<i64>("i64", &["9223372036854775807", "9223372036854775808", "-9223372036854775808"]);
    parse_report::<u64>("u64", &["18446744073709551615", "18446744073709551616"]);

    println!("{:?} {:?} {:?} {:?} {:?} {:?}", 'a', '\n', '\'', '"', '\t', 'é');
    println!("{:?} {:?} {:?} {:?}", "tab\there", "quote\"q", "back\\slash", "nl\nx");
    println!("{:?} {:?} {:?} {:?}", "é", "😀", "\u{7f}", "\u{0}");
    println!("{:?} {:?} {:?}", "\r", "\u{1b}[0m", "\u{200b}");
    println!("{:?} {:?}", '\u{7f}', '\u{200b}');
    println!("{} {} {} {}", "ß".to_uppercase(), "ǅ".to_lowercase(), "İ".to_lowercase().chars().count(), "ﬁ".to_uppercase());
    let s = "a😀é";
    println!("{} {} {} {:?}", s.len(), s.chars().count(), s.bytes().len(), s.char_indices().map(|(i, _)| i).collect::<Vec<_>>());
    println!("{} {} {}", 'ß'.is_lowercase(), 'ß'.to_uppercase(), 'ǅ'.is_uppercase());
    println!("{:?} {:?}", "Hello".to_lowercase(), "ПРИВЕТ".to_lowercase());
    println!("{}", i32::MAX.wrapping_add(1));
    println!("{} {} {}", 7 / -2, 7 % -2, (-7i32).rem_euclid(2));
    println!("{} {} {}", (-7i32).div_euclid(2), 7i32.div_euclid(-2), (-7i32).pow(3));
    println!("{} {} {} {}", (2.0f64).powi(-2), (2.0f64).powf(0.5), (-8.0f64).cbrt(), (0.0f64).powi(0));
    println!("{} {} {}", (1e308f64 * 10.0), (-1e308f64 * 10.0), (0.0f64 / 0.0).is_nan());
    println!("{} {} {} {}", 5.0f64 % 3.0, -5.0f64 % 3.0, 5.5f64 % -2.0, (5.5f64).rem_euclid(-2.0));
    println!("{} {} {}", (2.5f64).round(), (-2.5f64).round(), (3.5f64).round());
    println!("{} {} {}", (2.5f64).trunc(), (-2.7f64).floor(), (-2.2f64).ceil());
    println!("{} {} {} {}", f64::EPSILON, f64::MIN, i64::MAX, u64::MAX);
}
