use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Employee {
    name: String,
    dept: String,
    salary: u32,
}

fn main() {
    let nums = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
    let doubled: Vec<i32> = nums.iter().map(|x| x * 2).collect();
    let evens: Vec<&i32> = nums.iter().filter(|x| *x % 2 == 0).collect();
    let sum: i32 = nums.iter().sum();
    let product: i64 = nums.iter().map(|&x| x as i64).product();
    println!("{:?}\n{:?}\n{} {}", doubled, evens, sum, product);
    println!("max {:?} min {:?}", nums.iter().max(), nums.iter().min());
    println!("any>8 {} all>0 {}", nums.iter().any(|&x| x > 8), nums.iter().all(|&x| x > 0));
    println!("position of 9: {:?}", nums.iter().position(|&x| x == 9));
    let folded = nums.iter().fold(String::new(), |acc, x| acc + &x.to_string());
    println!("folded {folded}");
    for (i, x) in nums.iter().enumerate().take(3) {
        print!("{i}:{x} ");
    }
    println!();
    let pairs: Vec<(i32, char)> = vec![1, 2, 3].into_iter().zip("abc".chars()).collect();
    println!("{:?}", pairs);
    let flat: Vec<i32> = vec![vec![1, 2], vec![3], vec![]].into_iter().flatten().collect();
    println!("{:?}", flat);
    let fm: Vec<i32> = (1..4).flat_map(|i| (0..i).map(move |j| i * 10 + j)).collect();
    println!("{:?}", fm);
    let skipped: Vec<i32> = nums.iter().skip(7).cloned().collect();
    println!("{:?} last {:?}", skipped, nums.last());
    let squares: Vec<u32> = (1..=10).filter(|n| n % 3 == 0).map(|n| n * n).collect();
    println!("{:?}", squares);
    let total: f64 = [1.5, 2.25, 3.0].iter().sum();
    println!("total {total}");
    let chained: Vec<i32> = [1, 2].iter().chain([3, 4].iter()).copied().collect();
    println!("{:?}", chained);

    let staff = vec![
        Employee { name: "Ana".into(), dept: "eng".into(), salary: 120 },
        Employee { name: "Ben".into(), dept: "ops".into(), salary: 90 },
        Employee { name: "Cy".into(), dept: "eng".into(), salary: 100 },
        Employee { name: "Di".into(), dept: "ops".into(), salary: 95 },
    ];
    let top = staff.iter().max_by_key(|e| e.salary).unwrap();
    println!("top earner {}", top.name);
    let mut by_dept: HashMap<String, u32> = HashMap::new();
    for e in &staff {
        *by_dept.entry(e.dept.clone()).or_insert(0) += e.salary;
    }
    let mut depts: Vec<(&String, &u32)> = by_dept.iter().collect();
    depts.sort();
    println!("{:?}", depts);
    let names: Vec<String> = staff.iter().filter(|e| e.dept == "eng").map(|e| e.name.to_uppercase()).collect();
    println!("{}", names.join(" & "));
    let avg = staff.iter().map(|e| e.salary as f64).sum::<f64>() / staff.len() as f64;
    println!("average {:.2}", avg);
    let (eng, ops): (Vec<&Employee>, Vec<&Employee>) = staff.iter().partition(|e| e.dept == "eng");
    println!("{} eng, {} ops", eng.len(), ops.len());
    let lookup: HashMap<&str, u32> = staff.iter().map(|e| (e.name.as_str(), e.salary)).collect();
    println!("Cy earns {}", lookup["Cy"]);
    let count = (1..=100).filter(|n| n % 7 == 0).count();
    println!("multiples of 7: {count}");
    let mut sorted = nums.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    println!("{:?}", sorted);
    let found = nums.iter().find(|&&x| x > 4);
    println!("{:?}", found);
    let rev: Vec<i32> = nums.iter().rev().take(2).cloned().collect();
    println!("{:?}", rev);
}
