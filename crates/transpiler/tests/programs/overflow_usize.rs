// Translated with `--overflow-checks`: `usize` cannot go below zero.
fn gap(v: &[usize], i: usize, j: usize) -> usize {
    v[i] - v[j]
}

fn main() {
    let v = vec![10, 4, 7];
    println!("{}", gap(&v, 0, 1));
    println!("{}", gap(&v, 2, 1) * 3);
    let mut countdown: usize = 2;
    while countdown > 0 {
        countdown -= 1;
        println!("tick {countdown}");
    }
    println!("{}", gap(&v, 1, 2));
}
