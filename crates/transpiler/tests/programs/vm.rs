use std::collections::HashMap;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
enum Instr {
    Push(i64),
    Add,
    Mul,
    Div,
    Dup,
    Swap,
    Jz(usize),
    Jmp(usize),
    Print,
    Store(String),
    Load(String),
    Halt,
}

#[derive(Debug, PartialEq)]
enum VmError {
    Parse(String),
    Underflow(usize),
    DivByZero,
    BadJump(usize),
    Unknown(String),
}

impl From<ParseIntError> for VmError {
    fn from(e: ParseIntError) -> Self {
        VmError::Parse(e.to_string())
    }
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VmError::Parse(m) => write!(f, "parse: {m}"),
            VmError::Underflow(pc) => write!(f, "stack underflow at {pc}"),
            VmError::DivByZero => write!(f, "division by zero"),
            VmError::BadJump(t) => write!(f, "bad jump to {t}"),
            VmError::Unknown(op) => write!(f, "unknown op {op:?}"),
        }
    }
}

impl FromStr for Instr {
    type Err = VmError;
    fn from_str(s: &str) -> Result<Instr, VmError> {
        let mut parts = s.split_whitespace();
        let op = parts.next().ok_or_else(|| VmError::Unknown(String::new()))?;
        let arg = parts.next();
        let instr = match (op, arg) {
            ("push", Some(a)) => Instr::Push(a.parse()?),
            ("add", None) => Instr::Add,
            ("mul", None) => Instr::Mul,
            ("div", None) => Instr::Div,
            ("dup", None) => Instr::Dup,
            ("swap", None) => Instr::Swap,
            ("jz", Some(a)) => Instr::Jz(a.parse()?),
            ("jmp", Some(a)) => Instr::Jmp(a.parse()?),
            ("print", None) => Instr::Print,
            ("store", Some(name)) => Instr::Store(name.to_string()),
            ("load", Some(name)) => Instr::Load(name.to_string()),
            ("halt", None) => Instr::Halt,
            _ => return Err(VmError::Unknown(s.to_string())),
        };
        Ok(instr)
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instr::Push(n) => write!(f, "push {n}"),
            Instr::Jz(t) => write!(f, "jz {t}"),
            Instr::Jmp(t) => write!(f, "jmp {t}"),
            Instr::Store(n) => write!(f, "store {n}"),
            Instr::Load(n) => write!(f, "load {n}"),
            other => write!(f, "{}", format!("{other:?}").to_lowercase()),
        }
    }
}

struct Vm {
    code: Vec<Instr>,
    stack: Vec<i64>,
    vars: HashMap<String, i64>,
    output: Vec<i64>,
    steps: usize,
}

impl Vm {
    fn load(text: &str) -> Result<Vm, VmError> {
        let code = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.parse::<Instr>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Vm { code, stack: Vec::new(), vars: HashMap::new(), output: Vec::new(), steps: 0 })
    }

    fn pop(&mut self, pc: usize) -> Result<i64, VmError> {
        self.stack.pop().ok_or(VmError::Underflow(pc))
    }

    fn run(&mut self) -> Result<usize, VmError> {
        let mut pc = 0;
        while pc < self.code.len() {
            self.steps += 1;
            if self.steps > 1000 {
                break;
            }
            let instr = self.code[pc].clone();
            match instr {
                Instr::Push(n) => self.stack.push(n),
                Instr::Add => {
                    let (b, a) = (self.pop(pc)?, self.pop(pc)?);
                    self.stack.push(a + b);
                }
                Instr::Mul => {
                    let (b, a) = (self.pop(pc)?, self.pop(pc)?);
                    self.stack.push(a * b);
                }
                Instr::Div => {
                    let (b, a) = (self.pop(pc)?, self.pop(pc)?);
                    self.stack.push(a.checked_div(b).ok_or(VmError::DivByZero)?);
                }
                Instr::Dup => {
                    let top = *self.stack.last().ok_or(VmError::Underflow(pc))?;
                    self.stack.push(top);
                }
                Instr::Swap => {
                    let n = self.stack.len();
                    if n < 2 {
                        return Err(VmError::Underflow(pc));
                    }
                    self.stack.swap(n - 1, n - 2);
                }
                Instr::Jz(target) => {
                    if self.pop(pc)? == 0 {
                        if target > self.code.len() {
                            return Err(VmError::BadJump(target));
                        }
                        pc = target;
                        continue;
                    }
                }
                Instr::Jmp(target) => {
                    if target > self.code.len() {
                        return Err(VmError::BadJump(target));
                    }
                    pc = target;
                    continue;
                }
                Instr::Print => {
                    let v = self.pop(pc)?;
                    self.output.push(v);
                }
                Instr::Store(name) => {
                    let v = self.pop(pc)?;
                    self.vars.insert(name, v);
                }
                Instr::Load(name) => {
                    let v = *self.vars.get(&name).unwrap_or(&0);
                    self.stack.push(v);
                }
                Instr::Halt => break,
            }
            pc += 1;
        }
        Ok(pc)
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn main() {
    let program = "
        # countdown from 3, printing, then a product
        push 3
        store n
        load n
        jz 12
        load n
        print
        load n
        push -1
        add
        store n
        jmp 2
        halt
        push 6
        push 7
        mul
        print
        push 1
        push 0
        swap
        print
    ";
    match Vm::load(program) {
        Ok(mut vm) => {
            println!("{} instructions: {}", vm.code.len(), vm.code.iter().take(4).map(|i| i.to_string()).collect::<Vec<_>>().join(" / "));
            let result = vm.run();
            println!("{:?} {:?} {:?} {}", result, vm.output, vm.stack, vm.steps);
            let mut names: Vec<_> = vm.vars.iter().collect();
            names.sort();
            println!("{:?}", names);
        }
        Err(e) => println!("load failed: {e}"),
    }
    for bad in ["push x", "pop", "jmp", "push 1\npush 0\ndiv", "add", "jmp 99", "dup"] {
        let outcome = Vm::load(bad).and_then(|mut vm| vm.run());
        match outcome {
            Ok(pc) => println!("ok pc={pc}"),
            Err(e) => println!("error: {e} / {e:?}"),
        }
    }
    println!("{:?} {:?}", "push 5".parse::<Instr>(), "halt".parse::<Instr>().map(|i| i.to_string()));
    println!("{} {} {:?} {}", capitalize("hello"), capitalize("éa"), capitalize(""), capitalize("ßx"));
    let mut it = "abc".chars();
    it.next();
    println!("{:?} {:?}", it.as_str(), it.clone().rev().collect::<String>());
}
