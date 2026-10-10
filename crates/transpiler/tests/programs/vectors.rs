fn main() {
    let mut v: Vec<i32> = Vec::new();
    for i in 1..=10 {
        v.push(i * 3 % 7);
    }
    println!("{:?}", v);
    v.sort_unstable();
    println!("sorted {:?}", v);
    v.dedup();
    println!("dedup {:?}", v);
    let (left, right) = v.split_at(2);
    println!("split {:?} {:?}", left, right);
    println!("position of 4: {:?}, of 42: {:?}", v.iter().position(|&x| x == 4), v.iter().position(|&x| x == 42));
    println!("binary search 5: {:?}", v.binary_search(&5));
    println!("max {:?} min {:?}", v.iter().max(), v.iter().min());
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    println!("doubled {:?}", doubled);
    let total: i32 = doubled.iter().sum();
    let product: i64 = doubled.iter().map(|&x| x as i64).product();
    println!("sum {} product {}", total, product);
    let evens: Vec<&i32> = v.iter().filter(|x| **x % 2 == 0).collect();
    println!("evens {:?}", evens);
    let firsts: Vec<i32> = v.iter().take(3).cloned().collect();
    let rest: Vec<i32> = v.iter().skip(3).cloned().collect();
    println!("take {:?} skip {:?}", firsts, rest);
    let mut w = vec![1, 2, 3, 4, 5, 6];
    w.swap(0, 5);
    w.swap_remove(1);
    println!("swapped {:?}", w);
    w.resize(8, 0);
    println!("resized {:?}", w);
    w.drain(0..2);
    println!("drained {:?}", w);
    let words = vec!["apple", "banana", "cherry"];
    println!("joined {} concat {}", words.join(", "), words.concat());
    let lengths: Vec<usize> = words.iter().map(|w| w.len()).collect();
    println!("lengths {:?}", lengths);
    let longest = words.iter().max_by_key(|w| w.len()).unwrap();
    let shortest = words.iter().min_by_key(|w| w.len()).unwrap();
    println!("longest {} shortest {}", longest, shortest);
    let mut grid = vec![vec![0; 3]; 2];
    grid[1][2] = 7;
    grid[0][0] = 1;
    for row in &grid {
        println!("{:?}", row);
    }
    let flat: Vec<i32> = grid.iter().flatten().copied().collect();
    println!("flat {:?} any 7: {} all >= 0: {}", flat, flat.iter().any(|&x| x == 7), flat.iter().all(|&x| x >= 0));
    let pairs: Vec<(usize, char)> = "abc".chars().enumerate().collect();
    println!("{:?}", pairs);
    let zipped: Vec<(i32, &str)> = vec![1, 2, 3].into_iter().zip(words.iter().copied()).collect();
    println!("{:?}", zipped);
    let (small, big): (Vec<i32>, Vec<i32>) = (1..=8).partition(|x| *x < 4);
    println!("{:?} {:?}", small, big);
    let nested = vec![vec![1, 2], vec![3], vec![]];
    let flat2: Vec<i32> = nested.into_iter().flatten().collect();
    println!("{:?}", flat2);
    let rev: Vec<i32> = (1..5).rev().collect();
    println!("{:?} last {:?}", rev, rev.last());
    let stepped: Vec<i32> = (0..20).step_by(5).collect();
    println!("{:?}", stepped);
    let chained: Vec<i32> = (1..3).chain(7..9).collect();
    println!("{:?}", chained);
    let s: Vec<String> = vec![3, 1, 2].iter().map(|n| format!("<{n}>")).collect();
    println!("{}", s.join(""));
    let mut sorted_words = vec!["pear", "Apple", "fig", "banana"];
    sorted_words.sort();
    println!("{:?}", sorted_words);
    sorted_words.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    println!("{:?}", sorted_words);
    sorted_words.sort_by_key(|w| std::cmp::Reverse(w.len()));
    println!("{:?}", sorted_words);
    let arr = [10, 20, 30];
    println!("{} {} {:?}", arr.len(), arr[1], arr.iter().rev().collect::<Vec<_>>());
    let mut counts = [0u32; 4];
    for x in [1, 3, 3, 0, 1, 3] {
        counts[x] += 1;
    }
    println!("{:?}", counts);
    let last_two = &v[v.len() - 2..];
    println!("{:?} {:?} {:?}", last_two, &v[..2], &v[1..3]);
    println!("{}", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("-"));
    let mut stack = vec![1];
    while let Some(top) = stack.pop() {
        if top < 4 {
            stack.push(top + 1);
            stack.push(top + 2);
        }
        print!("{top} ");
    }
    println!();
    let empty: Vec<i32> = vec![];
    println!("{:?} {:?} {}", empty.first(), empty.iter().max(), empty.is_empty());
    let nums = [1.5, 2.25, 3.0];
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    println!("mean {:.3}", mean);
    let fmax = nums.iter().cloned().fold(f64::MIN, f64::max);
    println!("fmax {}", fmax);
    let idx = nums.iter().rposition(|&x| x > 2.0);
    println!("{:?}", idx);
    println!("{:?}", v.iter().rev().skip_while(|&&x| x > 3).collect::<Vec<_>>());
    println!("{:?}", v.iter().take_while(|&&x| x < 4).collect::<Vec<_>>());
    println!("{:?}", v.iter().map(|x| x * x).filter(|x| x % 2 == 1).last());
    println!("{}", v.iter().count());
    println!("{:?}", v.iter().nth(2));
    println!("{:?}", v.iter().copied().reduce(|a, b| a * 10 + b));
    println!("{:?}", v.windows(3).map(|w| w.iter().sum::<i32>()).collect::<Vec<_>>());
    println!("{:?}", v.chunks(4).map(|c| c.len()).collect::<Vec<_>>());
    println!("{:?}", v.iter().scan(0, |acc, &x| { *acc += x; Some(*acc) }).collect::<Vec<_>>());
    println!("{:?}", v.first().copied().unwrap_or_default());
    println!("{}", v.contains(&3) && !v.is_empty());
    println!("{:?}", v.iter().map(|&x| x as f64 / 2.0).collect::<Vec<f64>>());
    v.clear();
    println!("{:?} {}", v, v.len());
}
