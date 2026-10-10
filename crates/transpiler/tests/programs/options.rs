#[derive(Debug, Clone, PartialEq)]
struct User {
    name: String,
    age: Option<u32>,
    email: Option<String>,
}

fn find_user<'a>(users: &'a [User], name: &str) -> Option<&'a User> {
    users.iter().find(|u| u.name == name)
}

fn age_next_year(users: &[User], name: &str) -> Option<u32> {
    let user = find_user(users, name)?;
    let age = user.age?;
    Some(age + 1)
}

fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let (a, b) = s.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn compute(s: &str) -> Result<i32, String> {
    let n: i32 = s.parse().map_err(|_| format!("not a number: {s}"))?;
    let half = divide(n, 2)?;
    Ok(half * 3)
}

fn main() {
    let users = vec![
        User { name: "ann".into(), age: Some(30), email: Some("ann@x.io".into()) },
        User { name: "bob".into(), age: None, email: None },
    ];
    println!("{:?} {:?} {:?}", age_next_year(&users, "ann"), age_next_year(&users, "bob"), age_next_year(&users, "zed"));
    let ann = find_user(&users, "ann").unwrap();
    println!("{}", ann.email.as_deref().unwrap_or("no email"));
    println!("{}", users[1].email.clone().unwrap_or_else(|| "none".to_string()));
    println!("{}", users[1].age.unwrap_or_default());
    println!("{:?}", ann.age.map(|a| a * 2));
    println!("{:?}", ann.age.and_then(|a| if a > 18 { Some("adult") } else { None }));
    println!("{:?}", users[1].age.or(Some(0)));
    println!("{:?}", ann.age.filter(|a| *a > 40));
    println!("{}", ann.age.is_some() && users[1].age.is_none());
    println!("{}", ann.age.is_some_and(|a| a == 30));
    println!("{:?}", ann.age.map_or("unknown".to_string(), |a| a.to_string()));
    println!("{:?}", ann.age.map_or_else(|| 0, |a| a + 1));
    println!("{:?}", ann.age.ok_or("missing"));
    println!("{:?}", users[1].age.ok_or_else(|| "missing".to_string()));
    println!("{:?}", ann.age.xor(users[1].age));
    println!("{:?}", ann.age.zip(Some("x")));
    let mut maybe = Some(3);
    let taken = maybe.take();
    println!("{:?} {:?}", taken, maybe);
    let old = maybe.replace(9);
    println!("{:?} {:?}", old, maybe);
    let v = maybe.get_or_insert(1);
    *v += 1;
    println!("{:?}", maybe);
    if let Some(x) = maybe.as_mut() {
        *x *= 2;
    }
    println!("{:?}", maybe);
    let nested: Option<Option<i32>> = Some(Some(4));
    println!("{:?}", nested.flatten());
    let names: Vec<Option<&str>> = vec![Some("a"), None, Some("c")];
    let present: Vec<&str> = names.iter().flatten().copied().collect();
    println!("{:?} {}", present, names.iter().filter(|n| n.is_none()).count());
    let all: Option<Vec<&str>> = names.iter().copied().collect();
    println!("{:?}", all);
    let some_all: Option<Vec<i32>> = vec![Some(1), Some(2)].into_iter().collect();
    println!("{:?}", some_all);
    println!("{:?} {:?}", parse_pair("3, 4"), parse_pair("3;4"));
    println!("{:?} {:?}", parse_pair("x,1"), parse_pair("5,6").map(|(a, b)| a * b));
    for s in ["10", "x", "7"] {
        match compute(s) {
            Ok(v) => println!("ok {v}"),
            Err(e) => println!("err {e}"),
        }
    }
    println!("{:?}", compute("8"));
    println!("{:?}", compute("y"));
    println!("{}", compute("8").is_ok());
    println!("{}", compute("y").is_err());
    println!("{:?}", compute("8").ok());
    println!("{:?}", compute("y").err());
    println!("{}", compute("y").unwrap_or(-1));
    println!("{}", compute("y").unwrap_or_else(|e| e.len() as i32));
    println!("{}", compute("y").unwrap_or_default());
    println!("{:?}", compute("8").map(|v| v + 1));
    println!("{:?}", compute("y").map_err(|e| e.to_uppercase()));
    println!("{:?}", compute("8").and_then(|v| divide(v, 0)));
    println!("{:?}", divide(1, 0).or_else(|_| divide(1, 1)));
    println!("{}", divide(9, 3).is_ok_and(|v| v == 3));
    let results: Result<Vec<i32>, String> = ["1", "2", "3"].iter().map(|s| compute(s)).collect();
    println!("{:?}", results);
    let results: Result<Vec<i32>, String> = ["1", "q", "3"].iter().map(|s| compute(s)).collect();
    println!("{:?}", results);
    let oks: Vec<i32> = ["1", "q", "3"].iter().filter_map(|s| compute(s).ok()).collect();
    println!("{:?}", oks);
    let (good, bad): (Vec<_>, Vec<_>) = ["1", "q"].iter().map(|s| compute(s)).partition(|r| r.is_ok());
    println!("{} {}", good.len(), bad.len());
    if let Ok(v) = compute("4") {
        println!("got {v}");
    }
    let Ok(v) = compute("6") else {
        println!("unreachable");
        return;
    };
    println!("let-else {v}");
    let Some(u) = find_user(&users, "nobody") else {
        println!("no such user");
        return;
    };
    println!("{:?}", u);
}
