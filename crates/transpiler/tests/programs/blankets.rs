use std::fmt::{self, Write};

trait Describe {
    fn describe(&self) -> String;
    fn shout(&self) -> String {
        self.describe().to_uppercase()
    }
}

impl Describe for i32 {
    fn describe(&self) -> String {
        format!("int {}", self)
    }
}

impl Describe for String {
    fn describe(&self) -> String {
        format!("text '{}'", self)
    }
}

impl<T: Describe> Describe for Vec<T> {
    fn describe(&self) -> String {
        let parts: Vec<String> = self.iter().map(|x| x.describe()).collect();
        format!("[{}]", parts.join("; "))
    }
}

impl<T: Describe> Describe for Option<T> {
    fn describe(&self) -> String {
        match self {
            Some(x) => format!("some {}", x.describe()),
            None => "nothing".to_string(),
        }
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }

    fn push(&mut self, x: T) {
        self.items.push(x);
    }

    fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.items.iter().rev()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T: PartialEq> Stack<T> {
    fn contains(&self, x: &T) -> bool {
        self.items.contains(x)
    }
}

impl<T: fmt::Debug> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.items.iter().rev()).finish()
    }
}

impl<T: Describe> Stack<T> {
    fn render(&self, out: &mut String) -> fmt::Result {
        writeln!(out, "stack of {}", self.len())?;
        for (i, item) in self.iter().enumerate() {
            writeln!(out, "  {i}: {}", item.describe())?;
        }
        write!(out, "end")
    }
}

fn make_adder(a: i32) -> impl Fn(i32) -> i32 {
    move |b| a + b
}

fn main() {
    println!("{} | {} | {}", 7.describe(), "hi".to_string().shout(), vec![1, 2].describe());
    println!("{} | {}", Some(3).describe(), None::<String>.describe());
    println!("{}", vec![Some(1), None].describe());
    let nested: Vec<Vec<i32>> = vec![vec![1], vec![], vec![2, 3]];
    println!("{}", nested.describe());

    let mut stack: Stack<String> = Stack::new();
    for w in ["a", "b", "c"] {
        stack.push(w.to_string());
    }
    println!("{:?} {} {} {}", stack, stack.len(), stack.contains(&"b".to_string()), stack.contains(&"z".to_string()));
    println!("{:?}", stack.iter().map(|s| s.len()).collect::<Vec<_>>());
    let mut out = String::new();
    stack.render(&mut out).unwrap();
    println!("{out}");
    let mut ints: Stack<i32> = Stack::new();
    ints.push(5);
    ints.push(-5);
    let mut out2 = String::new();
    let _ = ints.render(&mut out2);
    println!("{}", out2.lines().count());
    for x in ints.iter() {
        print!("{x} ");
    }
    println!();

    let add5 = make_adder(5);
    let add = |a: i32| move |b: i32| a * 10 + b;
    println!("{} {} {}", add5(1), add(1)(2), [1, 2, 3].iter().map(|&x| add5(x)).sum::<i32>());
    let chained: Vec<i32> = std::iter::once(0).chain(ints.items.iter().copied()).chain(std::iter::once(99)).collect();
    println!("{:?}", chained);
    let text = "Hello, World! 42 ok?";
    println!("{} {} {} {}", text.chars().filter(|c| c.is_ascii_punctuation()).count(), text.chars().filter(char::is_ascii_graphic).count(), text.chars().filter(|c| c.is_ascii_whitespace()).count(), text.chars().filter(|c| c.is_alphabetic()).count());
    println!("{:?}", text.char_indices().filter(|(_, c)| c.is_uppercase()).map(|(i, _)| i).collect::<Vec<_>>());
    println!("{:?}", text.chars().rev().enumerate().find(|(_, c)| c.is_ascii_digit()));
    println!("{}", String::from_iter(text.chars().filter(|c| !c.is_whitespace())));
    println!("{}", text.split(|c: char| c == ',' || c == '!').map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("|"));
}
