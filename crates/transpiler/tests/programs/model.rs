use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn all() -> [Priority; 3] {
        [Priority::Low, Priority::Medium, Priority::High]
    }
    fn label(self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        }
    }
    fn from_label(s: &str) -> Option<Priority> {
        Priority::all().into_iter().find(|p| p.label() == s)
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label().to_uppercase())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    priority: Priority,
    done: bool,
    tags: Vec<String>,
    estimate: Option<f32>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mark = if self.done { "x" } else { " " };
        write!(f, "[{mark}] #{:03} {:<12} {:>6}", self.id, self.title, self.priority)?;
        if let Some(e) = self.estimate {
            write!(f, " ~{e:.1}h")?;
        }
        if !self.tags.is_empty() {
            write!(f, " ({})", self.tags.join(","))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Command {
    Add { title: String, priority: Priority },
    Done(u32),
    Remove(u32),
    Tag(u32, String),
    Clear,
}

#[derive(Debug, PartialEq)]
enum ModelError {
    NotFound(u32),
    Duplicate(String),
    Empty,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModelError::NotFound(id) => write!(f, "task {id} not found"),
            ModelError::Duplicate(t) => write!(f, "task '{t}' exists"),
            ModelError::Empty => write!(f, "empty title"),
        }
    }
}

#[derive(Default)]
struct Model {
    tasks: Vec<Task>,
    next_id: u32,
    history: Vec<String>,
}

impl Model {
    fn apply(&mut self, cmd: Command) -> Result<u32, ModelError> {
        let outcome = match cmd {
            Command::Add { title, priority } => {
                let title = title.trim().to_string();
                if title.is_empty() {
                    return Err(ModelError::Empty);
                }
                if self.tasks.iter().any(|t| t.title.eq_ignore_ascii_case(&title)) {
                    return Err(ModelError::Duplicate(title));
                }
                self.next_id += 1;
                let id = self.next_id;
                self.tasks.push(Task { id, title: title.clone(), priority, done: false, tags: vec![], estimate: None });
                self.history.push(format!("add {title}"));
                id
            }
            Command::Done(id) => {
                let i = self.position(id)?;
                self.tasks[i].done = true;
                self.history.push(format!("done {id}"));
                id
            }
            Command::Remove(id) => {
                let pos = self.tasks.iter().position(|t| t.id == id).ok_or(ModelError::NotFound(id))?;
                let removed = self.tasks.remove(pos);
                self.history.push(format!("remove {}", removed.title));
                removed.id
            }
            Command::Tag(id, tag) => {
                let i = self.position(id)?;
                if !self.tasks[i].tags.contains(&tag) {
                    self.tasks[i].tags.push(tag);
                    self.tasks[i].tags.sort();
                }
                id
            }
            Command::Clear => {
                let n = self.tasks.len() as u32;
                self.tasks.retain(|t| !t.done);
                self.history.push("clear".into());
                n - self.tasks.len() as u32
            }
        };
        Ok(outcome)
    }

    fn position(&self, id: u32) -> Result<usize, ModelError> {
        self.tasks.iter().position(|t| t.id == id).ok_or(ModelError::NotFound(id))
    }

    fn pending(&self) -> Vec<&Task> {
        let mut v: Vec<&Task> = self.tasks.iter().filter(|t| !t.done).collect();
        v.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.title.cmp(&b.title)));
        v
    }

    fn by_priority(&self) -> HashMap<Priority, usize> {
        let mut m = HashMap::new();
        for t in &self.tasks {
            *m.entry(t.priority).or_insert(0) += 1;
        }
        m
    }

    fn estimate_total(&self) -> f32 {
        self.tasks.iter().filter_map(|t| t.estimate).sum()
    }

    fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        self.tasks.iter().filter(|t| t.done).count() as f64 / self.tasks.len() as f64 * 100.0
    }
}

fn main() {
    let mut m = Model::default();
    let cmds = vec![
        Command::Add { title: "Write docs".into(), priority: Priority::Medium },
        Command::Add { title: "Fix bug".into(), priority: Priority::High },
        Command::Add { title: "  ".into(), priority: Priority::Low },
        Command::Add { title: "fix BUG".into(), priority: Priority::Low },
        Command::Add { title: "Refactor".into(), priority: Priority::Low },
        Command::Tag(2, "urgent".into()),
        Command::Tag(2, "backend".into()),
        Command::Tag(9, "x".into()),
        Command::Done(1),
        Command::Remove(42),
    ];
    for cmd in cmds {
        let desc = format!("{:?}", cmd);
        match m.apply(cmd) {
            Ok(n) => println!("ok {n:>3} <- {desc}"),
            Err(e) => println!("error: {e} <- {desc}"),
        }
    }
    m.tasks[1].estimate = Some(2.5);
    m.tasks[2].estimate = Some(0.75);
    for t in &m.tasks {
        println!("{t}");
    }
    println!("pending: {:?}", m.pending().iter().map(|t| t.id).collect::<Vec<_>>());
    let counts = m.by_priority();
    for p in Priority::all() {
        println!("{p}: {}", counts.get(&p).copied().unwrap_or(0));
    }
    println!("{:.2} {:.1}%", m.estimate_total(), m.progress());
    println!("{:?}", m.apply(Command::Clear));
    println!("{} {:?}", m.tasks.len(), m.history);
    println!("{:?} {:?}", Priority::from_label("high"), Priority::from_label("??"));
    println!("{}", Priority::Low < Priority::High);
    let mut ps = vec![Priority::High, Priority::Low, Priority::Medium];
    ps.sort();
    println!("{:?} {}", ps, ps.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("<"));
    println!("{:?}", m.apply(Command::Done(99)));
    println!("{}", ModelError::NotFound(99));
    println!("{}", m.apply(Command::Remove(2)).map(|id| id * 100).unwrap_or(0));
    println!("{:?}", m.tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
    let titles: Vec<String> = m.tasks.iter().map(|t| format!("{:>10}|", t.title)).collect();
    println!("{}", titles.join(""));
    println!("{:>8}|{:<8}|{:^8}|", Priority::Low, Priority::High, Priority::Medium);
    println!("{:>8}|", m.tasks[0].priority.to_string());
    println!("{:?}", m.tasks[0]);
}
