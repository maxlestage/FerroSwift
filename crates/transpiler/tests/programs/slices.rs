fn bump_all(slice: &mut [i32], by: i32) {
    for x in slice.iter_mut() {
        *x += by;
    }
}

fn first_negative(slice: &[i32]) -> Option<usize> {
    slice.iter().position(|&x| x < 0)
}

fn transpose(m: &[Vec<i32>]) -> Vec<Vec<i32>> {
    if m.is_empty() {
        return vec![];
    }
    (0..m[0].len()).map(|c| m.iter().map(|row| row[c]).collect()).collect()
}

fn matmul(a: &[Vec<i32>], b: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let n = a.len();
    let k = b.len();
    let m = b[0].len();
    let mut out = vec![vec![0; m]; n];
    for i in 0..n {
        for j in 0..m {
            for t in 0..k {
                out[i][j] += a[i][t] * b[t][j];
            }
        }
    }
    out
}

fn normalize(row: &mut Vec<f64>) {
    let sum: f64 = row.iter().sum();
    if sum != 0.0 {
        row.iter_mut().for_each(|x| *x /= sum);
    }
}

fn main() {
    let mut v = vec![5, 3, -8, 1, 9, -2, 7];
    bump_all(&mut v[2..5], 10);
    println!("{:?} {:?} {:?}", v, first_negative(&v), first_negative(&v[..5]));
    bump_all(&mut v, -1);
    v[..3].reverse();
    v[3..].sort();
    println!("{:?}", v);
    let (left, right) = v.split_at_mut(3);
    left[0] += right[0];
    right[1] *= 2;
    println!("{:?} {:?}", left, right);
    println!("{:?}", v);
    for chunk in v.chunks_mut(3) {
        chunk[0] = 0;
    }
    println!("{:?}", v);
    v.copy_within(0..3, 4);
    println!("{:?}", v);
    v[1..4].fill(42);
    v.rotate_right(2);
    println!("{:?} {:?}", v, v[2..5].to_vec());
    let tail = &v[v.len() - 2..];
    println!("{:?} {} {}", tail, tail.contains(&42), v[..2].iter().sum::<i32>());
    let grid = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let t = transpose(&grid);
    println!("{:?}", t);
    println!("{:?}", matmul(&grid, &t));
    println!("{:?}", grid.iter().flatten().filter(|&&x| x % 2 == 0).collect::<Vec<_>>());
    println!("{:?} {}", grid.concat(), grid.iter().map(|r| r.len()).sum::<usize>());
    println!("{:?}", grid.iter().map(|r| r.iter().max().unwrap()).collect::<Vec<_>>());
    let mut rows = vec![vec![1.0, 1.0, 2.0], vec![0.0, 0.0], vec![3.0]];
    for row in rows.iter_mut() {
        normalize(row);
    }
    println!("{:?}", rows);
    normalize(&mut rows[0]);
    rows[1].push(0.5);
    println!("{:?} {}", rows, rows.iter().any(|r| r.contains(&0.5)));
    let mut m = vec![vec![0u8; 4]; 3];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (r * 4 + c) as u8;
        }
    }
    m[1].swap(0, 3);
    m.swap(0, 2);
    println!("{:?}", m);
    let diag: Vec<u8> = (0..3).map(|i| m[i][i]).collect();
    println!("{:?} {:?}", diag, m.iter().map(|r| r.iter().map(|&x| x as u32).sum::<u32>()).collect::<Vec<_>>());
    let col: Vec<u8> = m.iter().map(|r| r[1]).collect();
    println!("{:?} {:?}", col, m.last().and_then(|r| r.first()));
    let flat: Vec<u8> = m.into_iter().flatten().collect();
    println!("{:?} {:?}", flat.windows(3).filter(|w| w[0] < w[1] && w[1] < w[2]).count(), flat.chunks(5).map(|c| c.len()).collect::<Vec<_>>());
    let mut arr = [9, 4, 7, 1];
    arr.sort_unstable();
    arr[1..3].swap(0, 1);
    println!("{:?} {:?} {:?}", arr, arr.iter().rev().collect::<Vec<_>>(), arr.first().zip(arr.last()));
    let (a, b) = arr.split_at(1);
    println!("{:?} {:?} {:?} {:?}", a, b, arr.split_first(), arr.split_last().map(|(l, rest)| (l, rest.len())));
    let words = ["x", "yy", "zzz", "ww"];
    let (short, long): (Vec<&str>, Vec<&str>) = words.iter().partition(|w| w.len() < 2);
    println!("{:?} {:?} {:?}", short, long, words[1..].join("/"));
    let mut s = [3, 1, 2];
    s.reverse();
    let doubled: Vec<i32> = s.iter().map(|x| x * 2).collect();
    println!("{:?} {:?} {}", s, doubled, s.starts_with(&[2, 1]));
    println!("{:?} {:?}", s.iter().rposition(|&x| x > 1), s.binary_search(&5));
}
