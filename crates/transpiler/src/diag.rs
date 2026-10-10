//! Diagnostics: what could not be translated, with its position in the Rust source.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// The Swift output is incomplete or will not compile.
    Error,
    /// The Swift output compiles but may behave differently, or needs a check.
    Warning,
}

/// A message about a position of the Rust source (lines and columns start at 1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    /// The file, when translating several.
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl Diagnostic {
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        match &self.file {
            Some(file) => write!(
                f,
                "{file}:{}:{}: {kind}: {}",
                self.line, self.column, self.message
            ),
            None => write!(f, "{}:{}: {kind}: {}", self.line, self.column, self.message),
        }
    }
}

/// Line and column (1-based) of the start of a span.
pub fn position(span: proc_macro2::Span) -> (usize, usize) {
    let start = span.start();
    (start.line.max(1), start.column + 1)
}
