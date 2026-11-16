//! A reading error: the model or the project file cannot be read as written. Every command reads
//! through the same reader, so an error has the same words wherever it is printed.

use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct ReadingError {
    pub kind: &'static str,
    /// A path from the repository root.
    pub file: String,
    /// Counted from 1.
    pub line: u64,
    pub message: String,
}

impl ReadingError {
    pub fn new(kind: &'static str, file: &str, line: u64, message: impl Into<String>) -> ReadingError {
        ReadingError { kind, file: file.to_string(), line, message: message.into() }
    }

    pub fn to_json(&self) -> Value {
        json!({ "kind": self.kind, "file": self.file, "line": self.line, "message": self.message })
    }

    pub fn to_text(&self) -> String {
        format!("{}:{}: {}", self.file, self.line, self.message)
    }
}
