use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
enum Priority {
    Low,
    #[default]
    Normal,
    High,
}

#[derive(Debug, Clone, PartialEq)]
struct Note {
    id: usize,
    title: String,
    body: String,
    priority: Priority,
    tags: Vec<String>,
    done: bool,
}

impl Default for Note {
    fn default() -> Self {
        Note { id: 0, title: String::from("untitled"), body: String::new(), priority: Priority::default(), tags: vec![], done: false }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mark = if self.done { 'x' } else { ' ' };
        write!(f, "[{}] #{:<3}{:<12}{:?}", mark, self.id, self.title, self.priority)
    }
}

fn normalize(s: &str) -> Cow<'_, str> {
    if s.chars().any(|c| c.is_uppercase() || c == '\t') {
        Cow::Owned(s.to_lowercase().replace('\t', " "))
    } else {
        Cow::Borrowed(s)
    }
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

fn count_calls<F: FnMut() -> usize>(mut f: F, times: usize) -> usize {
    let mut total = 0;
    for _ in 0..times {
        total += f();
    }
    total
}

fn visit_all(notes: &[Note], visitor: &mut impl FnMut(&Note)) {
    for n in notes {
        visitor(n);
    }
}

struct Notebook {
    notes: Vec<Note>,
    next_id: usize,
    trash: Vec<Note>,
}

impl Notebook {
    fn new() -> Self {
        Notebook { notes: Vec::new(), next_id: 1, trash: Vec::new() }
    }

    fn add(&mut self, title: &str, body: &str, priority: Priority) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let tags = words(body).into_iter().filter(|w| w.starts_with('#') || w.len() > 6).collect();
        self.notes.push(Note { id, title: title.to_string(), body: body.to_string(), priority, tags, ..Default::default() });
        id
    }

    fn duplicate(&mut self, id: usize) -> Option<usize> {
        let base = self.notes.iter().find(|n| n.id == id)?.clone();
        let new_id = self.next_id;
        self.next_id += 1;
        self.notes.push(Note { id: new_id, title: format!("{} (copy)", base.title), done: false, ..base });
        Some(new_id)
    }

    fn toggle(&mut self, id: usize) -> bool {
        let Some(note) = self.notes.iter_mut().find(|n| n.id == id) else {
            return false;
        };
        note.done = !note.done;
        true
    }

    fn archive_done(&mut self) -> usize {
        let notes = std::mem::take(&mut self.notes);
        let (done, open): (Vec<Note>, Vec<Note>) = notes.into_iter().partition(|n| n.done);
        let count = done.len();
        self.trash.extend(done);
        self.notes = open;
        count
    }

    fn index(&self) -> BTreeMap<String, Vec<usize>> {
        let mut index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for note in &self.notes {
            for w in words(&note.title).into_iter().chain(words(&note.body)) {
                let ids = index.entry(w).or_default();
                if !ids.contains(&note.id) {
                    ids.push(note.id);
                }
            }
        }
        index
    }
}

fn main() {
    let mut book = Notebook::new();
    book.add("Groceries", "milk, eggs, #weekend bread", Priority::Low);
    book.add("Report", "Finish the quarterly report\tbefore Friday", Priority::High);
    book.add("Call", "call the plumber about the kitchen sink", Priority::Normal);
    let copy = book.duplicate(2);
    println!("copy -> {:?}, missing -> {:?}", copy, book.duplicate(99));
    for n in &book.notes {
        println!("{n}|");
    }
    println!("{:?}", book.notes.iter().map(|n| (n.id, n.tags.clone())).collect::<Vec<_>>());

    let pairs: Vec<(String, i32)> = vec![("a".into(), 1), ("b".into(), 2), ("c".into(), 3)];
    let described: Vec<String> = pairs.clone().into_iter().map(|(k, v)| format!("{k}={v}")).collect();
    println!("{}", described.join(","));
    let total: i32 = pairs.into_iter().map(|(_, v)| v * 10).sum();
    println!("total {total}");

    for s in ["plain text", "Mixed Case", "tab\tseparated"] {
        let n = normalize(s);
        println!("normalized: {} ({} chars) {}", n, n.len(), n.as_ref() == s);
    }
    let owned: String = normalize("Hello").into_owned();
    println!("{owned}");

    println!("{} {}", book.toggle(1), book.toggle(42));
    book.toggle(3);
    let index = book.index();
    let busiest = index.iter().max_by_key(|(_, ids)| ids.len()).map(|(w, ids)| (w.clone(), ids.len()));
    println!("{:?}", busiest);
    let shared: Vec<String> = index.iter().filter_map(|(w, ids)| (ids.len() > 1).then(|| w.clone())).collect();
    println!("{:?}", shared);
    println!("{:?}", index.get("report"));

    let mut seen = 0;
    let mut titles = Vec::new();
    visit_all(&book.notes, &mut |n| {
        seen += 1;
        if n.priority >= Priority::Normal {
            titles.push(n.title.clone());
        }
    });
    println!("{seen} {:?}", titles);
    let mut counter = 0;
    let calls = count_calls(|| { counter += 1; counter * counter }, 4);
    println!("{calls} {counter}");

    let archived = book.archive_done();
    println!("archived {archived}, left {:?}, trash {:?}", book.notes.iter().map(|n| n.id).collect::<Vec<_>>(), book.trash.iter().map(|n| &n.title).collect::<Vec<_>>());
    let default_note = Note::default();
    println!("{default_note}| {:?} {}", default_note.priority, default_note == Note { ..Default::default() });
    println!("{:?} {:?}", Priority::default(), [Priority::High, Priority::Low, Priority::Normal].iter().max());

    'outer: for (i, note) in book.notes.iter().enumerate().rev() {
        for w in &note.tags {
            if w.contains("quarterly") {
                println!("skip {i}");
                continue 'outer;
            }
        }
        println!("keep {i} {}", note.title);
    }
    let grid = vec![vec!['a', 'b', 'c'], vec!['d', 'e', 'f']];
    let transposed: Vec<String> = (0..3).map(|i| grid.iter().map(|row| row[i]).collect()).collect();
    println!("{:?}", transposed);
    let pairs: Vec<String> = [1, 2, 3, 4, 5].chunks(2).map(|c| format!("{:?}", c)).collect();
    println!("{}", pairs.join(" "));
    let x = "5";
    let x: i32 = x.parse().unwrap();
    let x = x as f64 / 2.0;
    println!("{x}");
    println!("{}", matches!(book.notes.first(), Some(n) if n.id == 1));
}
