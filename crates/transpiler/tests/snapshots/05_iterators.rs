use std::collections::HashMap;

pub fn stats(values: &[f64]) -> (f64, f64) {
    let total: f64 = values.iter().sum();
    let mean = total / values.len() as f64;
    let max = values.iter().cloned().fold(f64::MIN, f64::max);
    (mean, max)
}

pub fn word_counts(text: &str) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in text.split_whitespace() {
        // Case does not matter.
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    pairs
}

pub fn evens_squared(limit: u32) -> Vec<u32> {
    (0..limit).filter(|n| n % 2 == 0).map(|n| n * n).collect()
}

pub fn report(names: &[&str]) -> String {
    names
        .iter()
        .enumerate()
        .map(|(i, name)| format!("{:>2}. {}", i + 1, name))
        .collect::<Vec<_>>()
        .join("\n")
}
