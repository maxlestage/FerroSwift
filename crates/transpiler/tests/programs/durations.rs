use std::time::Duration;

fn hms(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

fn with_commas(n: i64) -> String {
    let digits: Vec<char> = n.abs().to_string().chars().rev().collect();
    let mut out = String::new();
    for (i, d) in digits.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(*d);
    }
    let body: String = out.chars().rev().collect();
    if n < 0 { format!("-{body}") } else { body }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 { format!("{bytes} B") } else { format!("{value:.1} {}", UNITS[unit]) }
}

fn main() {
    for t in [0, 59, 60, 3599, 3600, 86399, 90061] {
        print!("{} ", hms(t));
    }
    println!();
    for n in [0, 7, 999, 1000, 1234567, -1234567, i64::MAX] {
        print!("{} ", with_commas(n));
    }
    println!();
    for b in [0, 1023, 1024, 1536, 1048576, 5_368_709_120, u64::MAX] {
        print!("{} | ", human_bytes(b));
    }
    println!();
    let d = Duration::from_secs(90) + Duration::from_millis(1500);
    println!("{} {} {} {}", d.as_secs(), d.as_millis(), d.subsec_millis(), d.as_secs_f64());
    let short = Duration::from_millis(250);
    println!("{:?} {:?} {:?}", d.checked_sub(short).map(|x| x.as_millis()), short.checked_sub(d), d > short);
    println!("{} {:?} {}", (short * 4).as_secs(), Duration::from_secs_f64(1.25).as_millis(), Duration::ZERO.is_zero());
    let mut total = Duration::default();
    for ms in [100, 250, 1650] {
        total += Duration::from_millis(ms);
    }
    println!("{} {} {}", total.as_secs(), total.as_millis() % 1000, hms(total.as_secs() + 3725));
    let laps = [Duration::from_millis(61_500), Duration::from_secs(59), Duration::from_millis(60_000)];
    let fastest = laps.iter().min().unwrap();
    let slowest = laps.iter().max().unwrap();
    println!("{} {} {:.2}", hms(fastest.as_secs()), hms(slowest.as_secs()), laps.iter().map(|l| l.as_secs_f64()).sum::<f64>() / laps.len() as f64);
    println!("{:?} {:?}", Duration::from_micros(1_000_001), Duration::from_nanos(5));
    println!("{:?} {:?} {:?}", Duration::from_secs(1), Duration::from_millis(1500), Duration::from_secs(3661));
    println!("{} {}", Duration::from_secs(2).as_secs_f32(), Duration::from_millis(100).mul_f64(2.5).as_millis());
    println!("{}", Duration::from_secs(10).saturating_sub(Duration::from_secs(20)).as_nanos());
}
