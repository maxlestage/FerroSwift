trait Command {
    fn apply(&mut self, text: &mut String);
    fn undo(&mut self, text: &mut String);
    fn name(&self) -> String;
}

struct Insert {
    at: usize,
    what: String,
}

impl Command for Insert {
    fn apply(&mut self, text: &mut String) {
        text.insert_str(self.at, &self.what);
    }
    fn undo(&mut self, text: &mut String) {
        text.replace_range(self.at..self.at + self.what.len(), "");
    }
    fn name(&self) -> String {
        format!("insert {:?}@{}", self.what, self.at)
    }
}

struct Delete {
    from: usize,
    to: usize,
    removed: String,
}

impl Command for Delete {
    fn apply(&mut self, text: &mut String) {
        self.removed = text.drain(self.from..self.to).collect();
    }
    fn undo(&mut self, text: &mut String) {
        text.insert_str(self.from, &self.removed);
    }
    fn name(&self) -> String {
        format!("delete {}..{}", self.from, self.to)
    }
}

struct Upper {
    saved: Option<String>,
}

impl Command for Upper {
    fn apply(&mut self, text: &mut String) {
        self.saved = Some(text.clone());
        *text = text.to_uppercase();
    }
    fn undo(&mut self, text: &mut String) {
        if let Some(s) = self.saved.take() {
            *text = s;
        }
    }
    fn name(&self) -> String {
        "upper".to_string()
    }
}

struct Editor {
    text: String,
    done: Vec<Box<dyn Command>>,
    undone: Vec<Box<dyn Command>>,
}

impl Editor {
    fn run(&mut self, mut cmd: Box<dyn Command>) {
        cmd.apply(&mut self.text);
        self.done.push(cmd);
        self.undone.clear();
    }

    fn undo(&mut self) -> Option<String> {
        let mut cmd = self.done.pop()?;
        cmd.undo(&mut self.text);
        let name = cmd.name();
        self.undone.push(cmd);
        Some(name)
    }

    fn redo(&mut self) -> Option<String> {
        let mut cmd = self.undone.pop()?;
        cmd.apply(&mut self.text);
        let name = cmd.name();
        self.done.push(cmd);
        Some(name)
    }

    fn history(&self) -> Vec<String> {
        self.done.iter().map(|c| c.name()).collect()
    }
}

fn byte_offset_of_char(text: &str, char_index: usize) -> usize {
    text.char_indices().nth(char_index).map(|(i, _)| i).unwrap_or(text.len())
}

fn main() {
    let mut ed = Editor { text: String::from("hello world"), done: Vec::new(), undone: Vec::new() };
    ed.run(Box::new(Insert { at: 5, what: ",".into() }));
    ed.run(Box::new(Delete { from: 0, to: 1, removed: String::new() }));
    ed.run(Box::new(Upper { saved: None }));
    println!("{:?} {:?}", ed.text, ed.history());
    println!("{:?} {:?}", ed.undo(), ed.text);
    println!("{:?} {:?}", ed.undo(), ed.text);
    println!("{:?} {:?}", ed.redo(), ed.text);
    ed.run(Box::new(Insert { at: ed.text.len(), what: "!".into() }));
    let redone = ed.redo();
    println!("{:?} {:?} {:?}", ed.text, redone, ed.history());
    while let Some(name) = ed.undo() {
        println!("undo {name} -> {:?}", ed.text);
    }
    println!("{:?} {} {}", ed.undo(), ed.done.len(), ed.undone.len());
    let accented = "héllo wörld";
    let at = byte_offset_of_char(accented, 7);
    let mut s = accented.to_string();
    s.insert_str(at, "_");
    s.replace_range(..1, "H");
    let tail: String = s.drain(byte_offset_of_char(&s, 9)..).collect();
    println!("{s} {tail} {} {}", s.len(), s.chars().count());
    let mut buf = String::from("abcdef");
    buf.replace_range(1..3, "XYZ");
    buf.replace_range(4.., "");
    buf.insert(0, '>');
    println!("{buf} {:?} {:?}", buf.pop(), buf.char_indices().map(|(i, c)| format!("{i}{c}")).collect::<Vec<_>>());
}
