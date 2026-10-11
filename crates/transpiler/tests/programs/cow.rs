use std::borrow::Cow;

fn normalize(s: &str) -> Cow<'_, str> {
    if s.contains('\t') || s.chars().any(|c| c.is_uppercase()) {
        Cow::Owned(s.replace('\t', " ").to_lowercase())
    } else {
        Cow::Borrowed(s)
    }
}

fn shout(s: &str) -> String {
    s.to_uppercase()
}

fn kind(c: &Cow<str>) -> &'static str {
    match c {
        Cow::Borrowed(_) => "borrowed",
        Cow::Owned(_) => "owned",
    }
}

fn main() {
    for s in ["plain", "Mixed", "tab\there"] {
        let n = normalize(s);
        println!("{} {} {} {} {}", kind(&n), n, n.len(), matches!(n, Cow::Borrowed(_)), shout(&n));
        println!("{:?} {}", n, n == "plain");
    }
    let mut c = normalize("keep");
    println!("{}", kind(&c));
    c.to_mut().push_str("-edited");
    println!("{} {}", kind(&c), c);
    let owned: String = normalize("ABC").into_owned();
    println!("{owned}");
    match normalize("x") {
        Cow::Borrowed(b) => println!("borrowed {b}"),
        Cow::Owned(o) => println!("owned {o}"),
    }
    let from_lit: Cow<str> = Cow::from("lit");
    let from_string: Cow<str> = Cow::from(String::from("built"));
    println!("{} {}", kind(&from_lit), kind(&from_string));
    let words: Vec<Cow<str>> = vec![normalize("a"), normalize("B")];
    println!("{:?} {}", words, words.iter().filter(|w| matches!(w, Cow::Owned(_))).count());
}
