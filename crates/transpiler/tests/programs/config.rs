use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    Missing(String),
    BadNumber { key: String, source: ParseIntError },
    Range { key: String, value: i64, max: i64 },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Missing(k) => write!(f, "missing key {k:?}"),
            ConfigError::BadNumber { key, source } => write!(f, "{key}: {source}"),
            ConfigError::Range { key, value, max } => write!(f, "{key}={value} exceeds {max}"),
        }
    }
}

impl Error for ConfigError {}

type BoxResult<T> = Result<T, Box<dyn Error>>;

struct Config {
    values: HashMap<String, String>,
}

impl Config {
    fn parse(text: &str) -> Result<Config, String> {
        let mut values = HashMap::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (k, v) = line.split_once('=').ok_or_else(|| format!("line {}: no '='", n + 1))?;
            values.insert(k.trim().to_string(), v.trim().to_string());
        }
        Ok(Config { values })
    }

    fn get(&self, key: &str) -> Result<&str, ConfigError> {
        self.values.get(key).map(String::as_str).ok_or_else(|| ConfigError::Missing(key.to_string()))
    }

    fn int(&self, key: &str, max: i64) -> Result<i64, ConfigError> {
        let raw = self.get(key)?;
        let value = raw.parse::<i64>().map_err(|source| ConfigError::BadNumber { key: key.to_string(), source })?;
        if value > max {
            return Err(ConfigError::Range { key: key.to_string(), value, max });
        }
        Ok(value)
    }
}

fn load(text: &str) -> BoxResult<(String, i64, i64)> {
    let cfg = Config::parse(text)?;
    let name = cfg.get("name")?.to_string();
    let port = cfg.int("port", 65535)?;
    let workers = cfg.int("workers", 64)?;
    if name.is_empty() {
        return Err("name must not be empty".into());
    }
    if workers == 0 {
        return Err(Box::new(ConfigError::Range { key: "workers".into(), value: 0, max: 64 }));
    }
    Ok((name, port, workers))
}

fn describe(e: &dyn Error) -> String {
    format!("<{e}>")
}

fn main() {
    let inputs = [
        "name = demo\nport = 8080\nworkers = 4",
        "# comment\nname=x\nport=99999\nworkers=1",
        "name=x\nport=80\nworkers=zero",
        "name=x\nport=80",
        "name=\nport=80\nworkers=2",
        "name=x\nport=80\nworkers=0",
        "garbage line",
    ];
    for text in inputs {
        match load(text) {
            Ok((name, port, workers)) => println!("ok {name} {port} {workers}"),
            Err(e) => println!("err {} / {}", e, describe(e.as_ref())),
        }
    }
    let r: BoxResult<i32> = "12".parse::<i32>().map_err(|e| e.into());
    let r2: BoxResult<i32> = "1x".parse::<i32>().map_err(Box::from);
    println!("{:?} {}", r.ok(), r2.map_err(|e| e.to_string()).unwrap_err());
    let errors: Vec<Box<dyn Error>> = vec![Box::new(ConfigError::Missing("k".into())), "plain".into(), Box::from(String::from("owned"))];
    println!("{}", errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "));
    let nested = ConfigError::BadNumber { key: "n".into(), source: "q".parse::<i32>().unwrap_err() };
    println!("{nested} | {nested:?}");
}
