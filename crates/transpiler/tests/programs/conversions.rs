use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Priority {
    Low,
    Medium,
    High,
}

impl FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" | "med" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            other => Err(format!("unknown priority: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Percent(u8);

impl TryFrom<i32> for Percent {
    type Error = String;
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        if (0..=100).contains(&v) {
            Ok(Percent(v as u8))
        } else {
            Err(format!("{v} is not a percentage"))
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Task {
    title: String,
    done: bool,
    tags: Vec<String>,
}

fn render(tasks: &[Task]) -> String {
    let mut out = String::new();
    writeln!(out, "{} tasks", tasks.len()).unwrap();
    for (i, t) in tasks.iter().enumerate() {
        write!(out, "{}. [{}] {}", i + 1, if t.done { "x" } else { " " }, t.title).unwrap();
        if !t.tags.is_empty() {
            write!(out, " #{}", t.tags.join(" #")).unwrap();
        }
        out.push('\n');
    }
    out
}

fn main() {
    println!("{:?} {:?}", "high".parse::<Priority>(), " MED ".parse::<Priority>());
    println!("{:?}", Priority::from_str("urgent"));
    let p: Priority = "low".parse().unwrap();
    println!("{:?} {}", p, p < Priority::High);
    let parsed: Result<Vec<Priority>, String> = "low,high,medium".split(',').map(|s| s.parse()).collect();
    println!("{:?}", parsed);
    println!("{:?} {:?}", Percent::try_from(42), Percent::try_from(101));
    let pc: Result<Percent, _> = 7.try_into();
    println!("{:?}", pc);
    let t: (i32,) = (5,);
    println!("{:?} {} {:?}", t, t.0, ((1, 2), (3,)));
    let (only,) = t;
    println!("{only}");
    println!("{:?} {:?} {:?}", "".parse::<i32>(), "99999999999".parse::<i32>(), "-99999999999".parse::<i64>());
    println!("{:?} {:?} {:?}", "-5".parse::<u8>(), "+5".parse::<u8>(), "5x".parse::<u8>());
    println!("{:?} {:?} {:?}", "".parse::<f64>(), "abc".parse::<f64>(), "1e400".parse::<f64>());
    println!("{:?} {:?} {:?}", "yes".parse::<bool>(), "".parse::<char>(), "ab".parse::<char>());
    println!("{:?} {:?}", u8::from_str_radix("zz", 36), i8::from_str_radix("-81", 16));
    println!("{:?} {:?}", String::from_utf8(vec![104, 105]), String::from_utf8(vec![104, 255, 105]));
    println!("{:?}", "7".parse::<u32>().map(|n| n * 6));
    let mut tasks = vec![
        Task { title: "write".into(), ..Default::default() },
        Task { title: "ship".into(), done: true, tags: vec!["v1".into()] },
    ];
    tasks.last_mut().unwrap().tags.push("done".into());
    tasks.first_mut().unwrap().done = true;
    tasks.iter_mut().for_each(|t| t.title = t.title.to_uppercase());
    tasks.get_mut(1).unwrap().title.push('!');
    print!("{}", render(&tasks));
    let mut groups: HashMap<String, Vec<i32>> = HashMap::new();
    groups.insert("a".into(), vec![1]);
    groups.get_mut("a").unwrap().push(2);
    if let Some(list) = groups.get_mut("a") {
        list.push(3);
        list.retain(|x| *x != 2);
    }
    println!("{:?}", groups["a"]);
    let mut counts = [0u32; 3];
    for c in counts.iter_mut() {
        *c += 1;
    }
    counts.iter_mut().for_each(|c| *c *= 10);
    println!("{:?}", counts);
    let words = vec!["a", "bb", "ccc"];
    let lens: Vec<usize> = words.iter().map(|w| w.len()).collect();
    let total: usize = lens.iter().sum();
    println!("{:?} {}", lens, total);
    let mut s = String::new();
    write!(&mut s, "{}-{}", 1, 2).unwrap();
    write!(s, "{:>4}", "x").unwrap();
    println!("{s}");
    let args: Vec<String> = std::env::args().collect();
    println!("{}", args.len() >= 1);
    let nested = vec![vec![1, 2], vec![3]];
    let sums: Vec<i32> = nested.iter().map(|v| v.iter().sum()).collect();
    println!("{:?}", sums);
    let opt_sum: Option<i32> = [Some(1), Some(2)].iter().copied().sum();
    println!("{:?}", opt_sum);
    let res_sum: Result<i32, String> = vec![Ok(1), Err("bad".to_string()), Ok(3)].into_iter().sum();
    println!("{:?}", res_sum);
    println!("{:?}", "a\tb".split('\t').map(str::to_uppercase).collect::<Vec<_>>());
    println!("{:?}", "x".parse::<i32>().is_err());
    println!("{:?}", "12".parse::<i32>().unwrap_or(-1) + "zz".parse::<i32>().unwrap_or(-1));
    println!("{:?}", "3".parse::<u8>().ok().map(|n| n as u32 * 2));
    println!("{:?}", "true".parse::<bool>().unwrap() && !"false".parse::<bool>().unwrap());
    println!("{:?}", "x".parse::<i32>().map_err(|e| e.to_string()));
    println!("{}", match "9".parse::<i32>() { Ok(n) => n * 2, Err(_) => 0 });
    if let Err(e) = "q".parse::<f64>() {
        println!("float error: {e}");
    }
    if let Err(e) = "".parse::<i32>() {
        println!("int error: {e}");
    }
    if let Err(e) = "300".parse::<u8>() {
        println!("u8 error: {e}");
    }
    if let Err(e) = "maybe".parse::<bool>() {
        println!("bool error: {e}");
    }
}
