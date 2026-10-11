use std::collections::VecDeque;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Add(String),
    Remove(usize),
    Toggle(usize),
    Rename { index: usize, title: String },
    Clear,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Todo {
    title: String,
    done: bool,
}

#[derive(Debug, Default)]
struct Store {
    todos: Vec<Todo>,
    history: Vec<Vec<Todo>>,
    redo: VecDeque<Vec<Todo>>,
}

impl Store {
    fn apply(&mut self, cmd: Command) -> Result<(), String> {
        let before = self.todos.clone();
        match cmd {
            Command::Add(title) if title.trim().is_empty() => return Err("empty title".into()),
            Command::Add(title) => self.todos.push(Todo { title, ..Default::default() }),
            Command::Remove(i) => {
                if i >= self.todos.len() {
                    return Err(format!("no todo {i}"));
                }
                self.todos.remove(i);
            }
            Command::Toggle(i) => match self.todos.get_mut(i) {
                Some(t) => t.done = !t.done,
                None => return Err(format!("no todo {i}")),
            },
            Command::Rename { index, title } => {
                let Some(t) = self.todos.get_mut(index) else {
                    return Err(format!("no todo {index}"));
                };
                t.title = title;
            }
            Command::Clear => self.todos.clear(),
        }
        self.history.push(before);
        self.redo.clear();
        Ok(())
    }

    fn undo(&mut self) -> bool {
        match self.history.pop() {
            Some(prev) => {
                let current = std::mem::replace(&mut self.todos, prev);
                self.redo.push_front(current);
                true
            }
            None => false,
        }
    }

    fn redo(&mut self) -> bool {
        if let Some(next) = self.redo.pop_front() {
            self.history.push(std::mem::take(&mut self.todos));
            self.todos = next;
            true
        } else {
            false
        }
    }

    fn render(&self) -> String {
        let mut out = String::new();
        for (i, t) in self.todos.iter().enumerate() {
            let _ = writeln!(out, "{i:>2}. [{}] {}", if t.done { 'x' } else { ' ' }, t.title);
        }
        let done = self.todos.iter().filter(|t| t.done).count();
        let _ = write!(out, "{done}/{} done", self.todos.len());
        out
    }
}

fn duration(seconds: u64) -> String {
    let (h, m, s) = (seconds / 3600, seconds % 3600 / 60, seconds % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

fn parse_duration(text: &str) -> Option<u64> {
    let parts: Vec<u64> = text.split(':').map(|p| p.parse().ok()).collect::<Option<Vec<_>>>()?;
    match parts.as_slice() {
        [s] => Some(*s),
        [m, s] if *s < 60 => Some(m * 60 + s),
        [h, m, s] if *m < 60 && *s < 60 => Some(h * 3600 + m * 60 + s),
        _ => None,
    }
}

fn rotate(grid: &[Vec<char>]) -> Vec<Vec<char>> {
    let rows = grid.len();
    let cols = grid.first().map_or(0, Vec::len);
    (0..cols).map(|c| (0..rows).rev().map(|r| grid[r][c]).collect()).collect()
}

fn main() {
    let mut store = Store::default();
    let results: Vec<Result<(), String>> = vec![
        store.apply(Command::Add("write".into())),
        store.apply(Command::Add("  ".into())),
        store.apply(Command::Add("test".into())),
        store.apply(Command::Toggle(0)),
        store.apply(Command::Rename { index: 1, title: "test more".into() }),
        store.apply(Command::Remove(5)),
    ];
    println!("{:?}", results);
    println!("{}", store.render());
    println!("{} {} {}", store.undo(), store.undo(), store.todos.len());
    println!("{} {} {}", store.redo(), store.redo(), store.redo());
    println!("{}", store.render().lines().count());
    store.apply(Command::Clear).unwrap();
    println!("{:?} {} {}", store.todos, store.history.len(), store.redo.len());
    println!("{} {} {}", duration(59), duration(3600), duration(3725));
    println!("{:?} {:?} {:?} {:?}", parse_duration("90"), parse_duration("1:30"), parse_duration("1:99"), parse_duration("1:02:03"));
    println!("{:?} {:?}", parse_duration("x"), parse_duration("1:2:3:4"));
    let grid: Vec<Vec<char>> = ["ab", "cd", "ef"].iter().map(|r| r.chars().collect()).collect();
    let turned = rotate(&grid);
    println!("{:?}", turned.iter().map(|r| r.iter().collect::<String>()).collect::<Vec<_>>());
    println!("{}", rotate(&rotate(&turned)).len());
    let maybe: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    println!("{:?} {:?}", maybe.iter().flatten().sum::<i32>(), maybe.iter().rev().find_map(|x| *x));
    let nested: Option<Result<i32, String>> = Some(Ok(4));
    println!("{:?} {:?}", nested.clone().transpose(), None::<Result<i32, String>>.transpose());
    let res: Result<Option<i32>, String> = Err("no".into());
    println!("{:?}", res.transpose());
    let mut items = vec!["a", "b", "c", "d"];
    let removed = items.swap_remove(1);
    println!("{removed} {:?}", items);
    items.sort_by_cached_key(|s| std::cmp::Reverse(s.to_string()));
    println!("{:?}", items);
    let pairs: Vec<(char, char)> = "abcd".chars().zip("abcd".chars().skip(1)).collect();
    println!("{:?}", pairs);
    let digits: Vec<u32> = "4a7".chars().filter_map(|c| c.to_digit(10)).collect();
    println!("{:?} {}", digits, digits.iter().fold(0, |acc, d| acc * 10 + d));
    let shifted: String = "Hello".bytes().map(|b| (b + 1) as char).collect();
    println!("{shifted} {}", (b'a' + 25) as char);
}
