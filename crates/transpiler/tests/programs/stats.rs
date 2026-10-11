use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct Sample {
    label: String,
    values: Vec<f32>,
    weight: u8,
}

fn mean(xs: &[f32]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().map(|&x| x as f64).sum::<f64>() / xs.len() as f64
}

fn stddev(xs: &[f32]) -> f64 {
    let m = mean(xs);
    let var = xs.iter().map(|&x| (x as f64 - m).powi(2)).sum::<f64>() / xs.len().max(1) as f64;
    var.sqrt()
}

fn percent(part: usize, whole: usize) -> u32 {
    if whole == 0 {
        return 0;
    }
    (part as f64 / whole as f64 * 100.0).round() as u32
}

fn histogram(values: &[f32], buckets: usize, width: usize) -> Vec<String> {
    let (lo, hi) = values.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let span = (hi - lo).max(f32::EPSILON);
    let mut counts = vec![0usize; buckets];
    for &v in values {
        let mut idx = ((v - lo) / span * buckets as f32) as usize;
        if idx >= buckets {
            idx = buckets - 1;
        }
        counts[idx] += 1;
    }
    let max = *counts.iter().max().unwrap_or(&1);
    counts
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let from = lo + span * i as f32 / buckets as f32;
            let bar = "#".repeat(c * width / max.max(1));
            format!("{:>6.2} |{:<w$}| {:>3}", from, bar, c, w = width)
        })
        .collect()
}

fn main() {
    let samples = vec![
        Sample { label: "alpha".into(), values: vec![1.5, 2.25, 3.0, 2.75, 1.0], weight: 3 },
        Sample { label: "beta".into(), values: vec![10.0, 12.5, 9.75], weight: 1 },
        Sample { label: "gamma".into(), values: vec![], weight: 0 },
        Sample { label: "delta".into(), values: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6], weight: 2 },
    ];
    println!("{:<8}{:>6}{:>8}{:>8}{:>6}", "label", "n", "mean", "sd", "w%");
    let total_weight: usize = samples.iter().map(|s| s.weight as usize).sum();
    for s in &samples {
        println!("{:<8}{:>6}{:>8.3}{:>8.3}{:>5}%", s.label, s.values.len(), mean(&s.values), stddev(&s.values), percent(s.weight as usize, total_weight));
    }
    let all: Vec<f32> = samples.iter().flat_map(|s| s.values.iter().copied()).collect();
    println!("{} values, mean {:.4}, sd {:.4}, min {:?}, max {:?}", all.len(), mean(&all), stddev(&all), all.iter().cloned().reduce(f32::min), all.iter().cloned().fold(f32::NAN, f32::max));
    for line in histogram(&all, 4, 10) {
        println!("{line}");
    }
    let deltas: Vec<f32> = all.windows(2).map(|w| w[1] - w[0]).collect();
    println!("{:?}", deltas.iter().map(|d| format!("{d:+.2}")).collect::<Vec<_>>());
    let moving: Vec<String> = all.windows(3).map(|w| format!("{:.2}", w.iter().sum::<f32>() / 3.0)).collect();
    println!("{}", moving.join(" "));
    let steps: Vec<f32> = all.iter().zip(all.iter().skip(1)).map(|(a, b)| b / a).collect();
    println!("{:?}", steps.iter().map(|s| (s * 100.0).round() / 100.0).collect::<Vec<_>>());
    let mut ranked: Vec<(&str, f64)> = samples.iter().map(|s| (s.label.as_str(), mean(&s.values))).collect();
    ranked.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(b.0)));
    println!("{:?}", ranked.iter().map(|(l, m)| format!("{l}={m:.2}")).collect::<Vec<_>>());
    let top: Vec<&str> = ranked.iter().rev().take(2).map(|(l, _)| *l).collect();
    println!("{:?}", top);
    let mut by_weight: BTreeMap<u8, Vec<&str>> = BTreeMap::new();
    for s in &samples {
        by_weight.entry(s.weight).or_default().push(&s.label);
    }
    println!("{:?}", by_weight);
    let mut counter: u8 = 250;
    let mut wraps = 0;
    for _ in 0..10 {
        let (next, wrapped) = counter.overflowing_add(3);
        counter = next;
        if wrapped {
            wraps += 1;
        }
    }
    println!("{counter} {wraps} {}", 7usize.saturating_sub(9));
    println!("{:?} {:?} {}", 10i32.checked_div(0), 10i32.checked_div(3), (2.0f64).powf(10.0));
    println!("{:^12}|{:^9.1}|{:>+7.1}|", "mid", 3.14159, -2.5);
    println!("{}", format!("{:>w$}", "x", w = 4).len());
}
