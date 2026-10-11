struct Tokenizer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &'a str) -> Self { Tokenizer { src, pos: 0 } }
    fn next_word(&mut self) -> Option<&'a str> {
        let rest = &self.src[self.pos..];
        let start = rest.find(|c: char| !c.is_whitespace())?;
        let rest = &rest[start..];
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        self.pos += start + end;
        Some(&rest[..end])
    }
}

fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|n| n % 2 == 0)
}

fn squares_of<'a>(v: &'a [i32]) -> impl Iterator<Item = i32> + 'a {
    v.iter().map(|x| x * x)
}

fn choose(desc: bool) -> Box<dyn Iterator<Item = i32>> {
    if desc { Box::new((1..=4).rev()) } else { Box::new(1..=4) }
}

fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

fn main() {
    let mut t = Tokenizer::new("  alpha beta   gamma ");
    let mut words = vec![];
    while let Some(w) = t.next_word() { words.push(w); }
    println!("{:?} {}", words, t.pos);
    println!("{:?}", evens(9).collect::<Vec<_>>());
    println!("{}", evens(100).take(3).map(|x| x.to_string()).collect::<Vec<_>>().join("+"));
    let v = vec![3, -2, 5];
    println!("{:?} {}", squares_of(&v).collect::<Vec<_>>(), squares_of(&v).max().unwrap());
    println!("{:?} {:?}", choose(true).collect::<Vec<_>>(), choose(false).sum::<i32>());
    let s1 = String::from("short");
    let res;
    {
        let s2 = String::from("longer one");
        res = longest(&s1, &s2).to_string();
    }
    println!("{res} | {}", first_line("one\ntwo"));
    let mut it = choose(false).skip(1).step_by(2);
    println!("{:?} {:?} {:?}", it.next(), it.next(), it.next());
}
