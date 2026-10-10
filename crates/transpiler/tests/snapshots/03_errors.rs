#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
    Invalid { key: String, value: String },
}

pub fn port(text: &str) -> Result<u16, ConfigError> {
    let value = text.trim();
    if value.is_empty() {
        return Err(ConfigError::Missing("port".to_string()));
    }
    let Some(port) = value.parse::<u16>().ok() else {
        return Err(ConfigError::Invalid { key: "port".into(), value: value.into() });
    };
    Ok(port)
}

pub fn address(host: &str, port_text: &str) -> Result<String, ConfigError> {
    let p = port(port_text)?;
    Ok(format!("{host}:{p}"))
}

pub fn port_or_default(text: &str) -> u16 {
    port(text).unwrap_or(8080)
}

pub fn first_char_upper(s: &str) -> Option<char> {
    let c = s.chars().next()?;
    Some(c.to_ascii_uppercase())
}
