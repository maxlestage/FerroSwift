use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq)]
struct Reading {
    sensor: String,
    day: u32,
    value: f64,
}

fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some((channel(0)?, channel(2)?, channel(4)?))
}

fn runs(xs: &[i32]) -> Vec<(i32, usize)> {
    xs.chunk_by(|a, b| a == b).map(|run| (run[0], run.len())).collect()
}

fn main() {
    let raw = "t1,1,20.5;t2,1,18.0;t1,2,21.0;t2,2,17.5;t1,3,19.0;t3,1,25.25;t2,3,18.5";
    let readings: Vec<Reading> = raw
        .split(';')
        .filter_map(|rec| {
            let mut parts = rec.split(',');
            let sensor = parts.next()?.to_string();
            let day = parts.next()?.parse().ok()?;
            let value = parts.next()?.parse().ok()?;
            Some(Reading { sensor, day, value })
        })
        .collect();
    println!("{} {:?}", readings.len(), readings.last());
    let mut by_sensor: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for r in &readings {
        by_sensor.entry(r.sensor.clone()).or_default().push(r.value);
    }
    for (sensor, values) in &by_sensor {
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let deltas: Vec<f64> = values.windows(2).map(|w| w[1] - w[0]).collect();
        println!("{sensor}: n={n} mean={mean:.2} min={min} max={max} deltas={deltas:?}");
    }
    let mut per_day: BTreeMap<u32, BTreeMap<&str, f64>> = BTreeMap::new();
    for r in &readings {
        per_day.entry(r.day).or_default().insert(r.sensor.as_str(), r.value);
    }
    println!("{:?}", per_day);
    let best_day = per_day.iter().max_by(|a, b| {
        let sa: f64 = a.1.values().sum();
        let sb: f64 = b.1.values().sum();
        sa.partial_cmp(&sb).unwrap()
    });
    println!("{:?}", best_day.map(|(d, m)| (d, m.len())));
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for r in &readings {
        *counts.entry(r.sensor.as_str()).or_insert(0) += 1;
    }
    let mut counts: Vec<(&str, usize)> = counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    println!("{:?}", counts);
    let mut kept = readings.clone();
    let mut index = 0;
    kept.retain(|r| {
        index += 1;
        index % 2 == 1 || r.value > 20.0
    });
    println!("{:?}", kept.iter().map(|r| format!("{}@{}", r.sensor, r.day)).collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", runs(&[1, 1, 2, 2, 2, 3, 1, 1]), runs(&[]), runs(&[7]));
    let groups: Vec<Vec<i32>> = [1, 2, 4, 5, 6, 9].chunk_by(|a, b| b - a == 1).map(|g| g.to_vec()).collect();
    println!("{:?}", groups);
    println!("{:?} {:?} {:?} {:?}", parse_hex_color("#ff8000"), parse_hex_color("#FFF"), parse_hex_color("ff8000"), parse_hex_color("#gg0000"));
    let chars: Vec<char> = "hello".chars().collect();
    let back: String = chars.iter().rev().collect();
    let hexed: String = chars.iter().map(|c| format!("{:02x}", *c as u32)).collect::<Vec<_>>().join(":");
    println!("{:?} {} {} {:?} {:?}", chars, back, hexed, chars.iter().position(|&c| c == 'l'), chars.split_last().map(|(l, rest)| (*l, rest.len())));
    println!("{:?} {:?} {:?}", 'f'.to_digit(16), 'g'.to_digit(16), 'F'.is_digit(16));
    let opt_opt: Option<Option<i32>> = Some(None);
    let nested: Option<Option<i32>> = Some(Some(3));
    println!("{:?} {:?} {:?} {:?}", opt_opt.flatten(), nested.flatten(), nested.map(|o| o.unwrap_or(0)), opt_opt.is_some_and(|o| o.is_none()));
    let m1: f64 = std::cmp::max(3, 7) as f64;
    let m2 = f64::max(1.5, -2.0).min(1.0);
    println!("{m1} {m2} {} {}", usize::MAX.saturating_sub(1) == usize::MAX - 1, 5usize.saturating_sub(9));
}
