use std::cmp::Reverse;

#[derive(Debug, Clone, PartialEq)]
struct Reading {
    sensor: String,
    value: f64,
    ok: bool,
}

fn stats(values: &[f64]) -> Option<(f64, f64, f64)> {
    if values.is_empty() {
        return None;
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    Some((min, max, mean))
}

fn median(values: &mut Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    let n = values.len();
    if n % 2 == 0 { (values[n / 2 - 1] + values[n / 2]) / 2.0 } else { values[n / 2] }
}

fn main() {
    let mut readings = vec![
        Reading { sensor: "a".into(), value: 1.5, ok: true },
        Reading { sensor: "b".into(), value: -2.25, ok: false },
        Reading { sensor: "a".into(), value: 3.0, ok: true },
        Reading { sensor: "c".into(), value: 0.0, ok: true },
    ];
    readings.sort_by(|a, b| a.sensor.cmp(&b.sensor).then(b.value.partial_cmp(&a.value).unwrap()));
    println!("{:?}", readings.iter().map(|r| format!("{}{}", r.sensor, r.value)).collect::<Vec<_>>());
    readings.sort_unstable_by_key(|r| Reverse((r.value * 100.0) as i64));
    println!("{:?}", readings.iter().map(|r| r.value).collect::<Vec<_>>());
    readings.retain_mut(|r| {
        r.value *= 2.0;
        r.ok
    });
    println!("{} {:?}", readings.len(), readings.last().map(|r| r.value));
    readings.dedup_by_key(|r| r.sensor.clone());
    println!("{:?}", readings.iter().map(|r| &r.sensor).collect::<Vec<_>>());
    let values: Vec<f64> = readings.iter().map(|r| r.value).collect();
    println!("{:?} {:?}", stats(&values), stats(&[]));
    let mut v2 = values.clone();
    v2.extend_from_slice(&[7.5, -1.0]);
    println!("{} {:?}", median(&mut v2), v2);
    v2.insert(0, 100.0);
    v2.swap(0, 1);
    v2.truncate(4);
    println!("{:?} {:?} {:?}", v2, v2.first(), v2.split_first().map(|(h, t)| (*h, t.len())));
    let drained: Vec<f64> = v2.drain(1..3).collect();
    println!("{:?} {:?}", drained, v2);
    let mut arr = [5, 3, 8, 1, 9, 2];
    arr.rotate_right(2);
    println!("{:?} {:?}", arr, arr.iter().rposition(|&x| x > 4));
    arr.sort_unstable();
    arr[..3].fill(0);
    println!("{:?} {} {}", arr, arr.contains(&9), arr.starts_with(&[0, 0]));
    let pairs: Vec<(usize, i32)> = arr.iter().rev().enumerate().map(|(i, &x)| (i, x)).filter(|(_, x)| *x > 0).collect();
    println!("{:?}", pairs);
    let chunks: Vec<i32> = arr.chunks_exact(4).map(|c| c.iter().sum()).collect();
    println!("{:?} {:?}", chunks, arr.chunks_exact(4).remainder());
    let nested = vec![vec![1, 2], vec![3], vec![]];
    println!("{:?} {:?} {:?}", nested.concat(), nested.join(&0), nested.iter().map(Vec::len).max());
    let (lo, hi): (Vec<i32>, Vec<i32>) = arr.iter().partition(|&&x| x < 5);
    println!("{:?} {:?}", lo, hi);
    let acc = arr.iter().fold((0, 1), |(s, p), &x| (s + x, p * x.max(1)));
    println!("{:?}", acc);
    println!("{:?}", arr.iter().zip(1..).map(|(x, i)| x * i).collect::<Vec<_>>());
    println!("{:?}", arr.iter().min_by(|a, b| a.cmp(b)).copied());
    println!("{:?}", values.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()));
    println!("{:?}", values.iter().cloned().max_by(|a, b| a.total_cmp(b)));
    println!("{} {} {} {}", (-7.5f64).abs(), (-7.5f64).signum(), 0.0f64.signum(), 12.7f64.clamp(0.0, 10.0));
    println!("{} {} {:?} {:?}", 17i32.clamp(1, 10), (-3i32).signum(), 7i32.checked_div(0), 7i32.checked_rem(2));
    println!("{} {} {}", 3usize.pow(4), 2u64.pow(63), (1u64 << 63).trailing_zeros());
    println!("{} {} {} {}", 1.1f32, 1.1f32 as f64, 16777217i32 as f32, 0.1f32 + 0.2f32);
    println!("{:e} {:E} {:e} {:.2e}", 1234567, 0.00012, 100u8, 123456.789);
    let width = 9;
    let prec = 3;
    println!("[{:>width$.prec$}] [{:<width$.prec$}] [{:^width$}]", 3.14159, 2.71828, "mid");
    println!("[{:>w2$}] [{:08.3}] [{:+.1}]", 42, -1.5, 2.25, w2 = 6);
    println!("{} {} {}", char::from(65u8), char::from(b'z'), b'!'.is_ascii_punctuation());
    println!("{} {} {}", "Zebra" < "apple", "é" > "z", "abc".cmp("abd") == std::cmp::Ordering::Less);
    println!("{:?} {:?}", "x".repeat(3).cmp(&"xx".to_string()), vec![1, 2, 3] < vec![1, 3]);
    println!("{:?}", (1, "b") < (1, "c"));
    let mut words = vec!["pear", "fig", "apple", "kiwi"];
    words.sort_by_key(|w| (w.len(), *w));
    println!("{:?}", words);
    words.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    println!("{:?}", words);
    println!("{:?}", words.iter().map(|w| w.len()).rev().collect::<Vec<_>>());
    println!("{:?}", words.binary_search(&"fig").is_ok());
    let total_len: usize = words.iter().map(|w| w.len()).sum();
    let avg = total_len as f64 / words.len() as f64;
    println!("{total_len} {avg:.3} {}", avg.round() as i32);
}
