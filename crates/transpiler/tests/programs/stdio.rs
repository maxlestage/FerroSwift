use std::io::{self, BufRead, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("args {:?}", args);
    let stdin = io::stdin();
    let mut total = 0i64;
    let mut lines = 0;
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        lines += 1;
        for w in line.split_whitespace() {
            if let Ok(n) = w.parse::<i64>() {
                total += n;
            }
        }
    }
    let mut out = io::stdout();
    writeln!(out, "lines={lines} total={total}").unwrap();
    write!(out, "no newline").unwrap();
    out.flush().unwrap();
    println!();
    let mut rest = String::new();
    io::stdin().read_to_string(&mut rest).unwrap();
    println!("rest {:?}", rest);
    let err = io::stderr();
    let mut e = err.lock();
    writeln!(e, "to stderr {}", 42).unwrap();
    eprint!("eprint ");
    eprintln!("done");
    if total == 0 {
        std::process::exit(3);
    }
    println!("unreachable");
}
