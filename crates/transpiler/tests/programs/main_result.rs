use std::fmt;

#[derive(Debug)]
enum CliError {
    Missing(&'static str),
    Invalid { field: String, value: i64 },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::Missing(what) => write!(f, "missing {what}"),
            CliError::Invalid { field, value } => write!(f, "{field} cannot be {value}"),
        }
    }
}

impl std::error::Error for CliError {}

fn check(values: &[(&'static str, i64)]) -> Result<i64, CliError> {
    let mut total = 0;
    for (name, v) in values {
        if *v < 0 {
            return Err(CliError::Invalid { field: name.to_string(), value: *v });
        }
        total += v;
    }
    if values.is_empty() {
        return Err(CliError::Missing("values"));
    }
    Ok(total)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", check(&[("a", 1), ("b", 2)])?);
    let n: i32 = "12".parse()?;
    println!("{n}");
    match check(&[]) {
        Ok(_) => println!("unexpected"),
        Err(e) => println!("handled: {e}"),
    }
    let bad: Result<i32, _> = "x1".parse::<i32>();
    if let Err(e) = &bad {
        println!("parse: {e}");
    }
    check(&[("c", -5)])?;
    println!("not reached");
    Ok(())
}

fn main() -> Result<(), CliError> {
    println!("start");
    if let Err(e) = run() {
        println!("run failed: {e}");
    }
    eprintln!("about to fail");
    check(&[("d", 7), ("e", -1)])?;
    println!("never");
    Ok(())
}
