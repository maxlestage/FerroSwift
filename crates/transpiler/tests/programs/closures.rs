fn apply_all(fs: &[Box<dyn Fn(i32) -> i32>], x: i32) -> Vec<i32> {
    fs.iter().map(|f| f(x)).collect()
}

fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

fn retry<F: FnMut() -> Option<String>>(mut attempt: F, times: usize) -> Option<String> {
    for _ in 0..times {
        if let Some(v) = attempt() {
            return Some(v);
        }
    }
    None
}

fn compose2(f: impl Fn(i32) -> i32, g: impl Fn(i32) -> i32) -> impl Fn(i32) -> i32 {
    move |x| g(f(x))
}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    let offset = 10;
    let fs: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(|x| x + 1),
        Box::new(move |x| x * offset),
        Box::new(|x| x * x),
    ];
    println!("{:?}", apply_all(&fs, 3));

    let mut next = make_counter();
    println!("{} {} {}", next(), next(), next());

    let mut tries = 0;
    let found = retry(
        || {
            tries += 1;
            if tries == 3 { Some(format!("ok after {tries}")) } else { None }
        },
        5,
    );
    println!("{:?}", found);

    let inc_then_double = compose2(|x| x + 1, |x| x * 2);
    println!("{}", inc_then_double(5));

    let mut people = vec![
        Person { name: "Zoe".into(), age: 30 },
        Person { name: "Adam".into(), age: 25 },
        Person { name: "Mia".into(), age: 30 },
    ];
    people.sort_by(|a, b| b.age.cmp(&a.age).then(a.name.cmp(&b.name)));
    for p in &people {
        println!("{} ({})", p.name, p.age);
    }
    let oldest = people.iter().map(|p| p.age).max().unwrap_or(0);
    let names_of_oldest: Vec<String> = people
        .iter()
        .filter(|p| p.age == oldest)
        .map(|p| p.name.clone())
        .collect();
    println!("oldest: {}", names_of_oldest.join(", "));

    let maybe_age = people.iter().find(|p| p.name == "Adam").map(|p| p.age + 1);
    println!("{:?}", maybe_age);
    let missing = people.iter().find(|p| p.name == "Bob").map(|p| p.age).unwrap_or_else(|| 0);
    println!("{missing}");
    let parsed = Some("12").and_then(|s| s.parse::<i32>().ok()).filter(|n| *n > 10);
    println!("{:?}", parsed);
    let total: u32 = people.iter().fold(0, |acc, p| acc + p.age);
    println!("total age {total}");
    let mut log = Vec::new();
    let mut record = |msg: &str| log.push(msg.to_uppercase());
    record("start");
    record("stop");
    println!("{:?}", log);
}
