use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    Todo,
    Doing,
    Done,
}

impl Status {
    fn next(self) -> Status {
        match self {
            Status::Todo => Status::Doing,
            Status::Doing => Status::Done,
            Status::Done => Status::Done,
        }
    }

    fn symbol(&self) -> char {
        match self {
            Status::Todo => ' ',
            Status::Doing => '~',
            Status::Done => 'x',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Due {
    year: u16,
    month: u8,
    day: u8,
}

impl fmt::Display for Due {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone)]
struct Task {
    id: u32,
    title: String,
    status: Status,
    due: Option<Due>,
    tags: Vec<String>,
    estimate: f32,
}

#[derive(Default)]
struct Board {
    tasks: Vec<Task>,
    next_id: u32,
}

impl Board {
    fn add(&mut self, title: &str, due: Option<(u16, u8, u8)>, tags: &[&str], estimate: f32) -> u32 {
        self.next_id += 1;
        self.tasks.push(Task {
            id: self.next_id,
            title: title.trim().to_string(),
            status: Status::Todo,
            due: due.map(|(y, m, d)| Due { year: y, month: m, day: d }),
            tags: tags.iter().map(|t| t.to_lowercase()).collect(),
            estimate,
        });
        self.next_id
    }

    fn advance(&mut self, id: u32) -> bool {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(t) => {
                t.status = t.status.next();
                true
            }
            None => false,
        }
    }

    fn remove_done(&mut self) -> usize {
        let before = self.tasks.len();
        self.tasks.retain(|t| t.status != Status::Done);
        before - self.tasks.len()
    }

    fn by_status(&self) -> BTreeMap<Status, Vec<&Task>> {
        let mut m: BTreeMap<Status, Vec<&Task>> = BTreeMap::new();
        for t in &self.tasks {
            m.entry(t.status).or_default().push(t);
        }
        m
    }

    fn overdue(&self, today: &Due) -> Vec<&Task> {
        let mut v: Vec<&Task> = self.tasks.iter().filter(|t| t.status != Status::Done && t.due.as_ref().is_some_and(|d| d < today)).collect();
        v.sort_by(|a, b| a.due.cmp(&b.due).then_with(|| a.id.cmp(&b.id)));
        v
    }

    fn tagged(&self, tag: &str) -> impl Iterator<Item = &Task> + '_ {
        let tag = tag.to_lowercase();
        self.tasks.iter().filter(move |t| t.tags.iter().any(|x| *x == tag))
    }

    fn progress(&self) -> f32 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        let done: f32 = self.tasks.iter().filter(|t| t.status == Status::Done).map(|t| t.estimate).sum();
        let total: f32 = self.tasks.iter().map(|t| t.estimate).sum();
        done / total * 100.0
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for t in &self.tasks {
            let due = t.due.as_ref().map(|d| d.to_string()).unwrap_or_else(|| "-".repeat(10));
            writeln!(f, "[{}] #{:<3}{:<16}{} {:>4.1}h {}", t.status.symbol(), t.id, t.title, due, t.estimate, t.tags.join(","))?;
        }
        write!(f, "{} task(s), {:.1}% done", self.tasks.len(), self.progress())
    }
}

fn main() {
    let mut board = Board::default();
    board.add("  Write docs ", Some((2024, 3, 1)), &["Docs", "writing"], 2.5);
    board.add("Fix bug #12", Some((2024, 2, 10)), &["code"], 1.0);
    board.add("Plan release", None, &["planning", "DOCS"], 4.0);
    let id = board.add("Refactor", Some((2024, 2, 28)), &[], 3.25);
    println!("{board}");
    println!("{} {} {}", board.advance(2), board.advance(2), board.advance(99));
    board.advance(id);
    println!("{board}");
    let today = Due { year: 2024, month: 2, day: 20 };
    println!("{:?}", board.overdue(&today).iter().map(|t| (t.id, t.due.as_ref().unwrap().to_string())).collect::<Vec<_>>());
    for (status, tasks) in board.by_status() {
        println!("{status:?}: {}", tasks.iter().map(|t| t.title.as_str()).collect::<Vec<_>>().join(" | "));
    }
    println!("{:?}", board.tagged("docs").map(|t| t.id).collect::<Vec<_>>());
    println!("{} removed, {} left, {:.2}%", board.remove_done(), board.tasks.len(), board.progress());
    println!("{:?} {:?}", board.tasks.iter().map(|t| t.estimate).fold(0.0f32, f32::max), board.tasks.last().map(|t| &t.title));
    println!("{}", board.tasks.iter().rev().skip(1).take(1).map(|t| t.id.to_string()).chain(std::iter::once("end".into())).collect::<Vec<_>>().join("->"));
    let (short, long): (Vec<&Task>, Vec<&Task>) = board.tasks.iter().partition(|t| t.estimate < 3.0);
    println!("{} {} {:?}", short.len(), long.len(), long.iter().map(|t| t.id).collect::<Vec<_>>());
    println!("{}", board.tasks.iter().all(|t| t.status != Status::Done) && board.tasks.iter().any(|t| t.due.is_none()));
}
