use std::fmt;

/// Errors are plain enums: `Result<T, ConfigError>` becomes `throws(ConfigError)`.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    Missing(String),
    Invalid { key: String, value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "missing key `{key}`"),
            ConfigError::Invalid { key, value } => write!(f, "`{value}` is not valid for `{key}`"),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub port: u16,
    pub verbose: bool,
}

/// Finds `key = value` in the text.
fn lookup<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim());
            }
        }
    }
    None
}

fn require<'a>(text: &'a str, key: &str) -> Result<&'a str, ConfigError> {
    lookup(text, key).ok_or(ConfigError::Missing(key.to_string()))
}

/// `?` propagates the error: `try` in Swift.
pub fn parse(text: &str) -> Result<Config, ConfigError> {
    let name = require(text, "name")?.to_string();
    let port_text = require(text, "port")?;
    let Ok(port) = port_text.parse::<u16>() else {
        return Err(ConfigError::Invalid {
            key: "port".to_string(),
            value: port_text.to_string(),
        });
    };
    let verbose = lookup(text, "verbose") == Some("true");
    Ok(Config { name, port, verbose })
}

fn main() {
    let inputs = ["name = demo\nport = 8080", "name = demo", "name = demo\nport = http"];
    for input in inputs {
        match parse(input) {
            Ok(config) => println!("ok: {:?}", config),
            Err(error) => println!("error: {error}"),
        }
    }
}
