//! Diagnostics.
//!
//! D17 makes good errors a matter of discipline rather than of a library default: without a
//! parser crate, nothing supplies file/line/expected for free. D11 requires the parser to fail
//! loudly, and the format's strictness is only tolerable when the errors are precise — so every
//! parse failure carries where it happened and what was expected instead.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    pub path: String,
    /// 1-indexed. Zero means the diagnostic is about the file as a whole.
    pub line: usize,
    pub message: String,
    /// What the parser was looking for. Omitted when the message is self-contained.
    pub expected: Option<String>,
}

impl Diag {
    pub fn at(path: &str, line: usize, message: impl Into<String>) -> Self {
        Diag {
            path: path.to_string(),
            line,
            message: message.into(),
            expected: None,
        }
    }

    pub fn expecting(
        path: &str,
        line: usize,
        message: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Diag {
            path: path.to_string(),
            line,
            message: message.into(),
            expected: Some(expected.into()),
        }
    }

    pub fn file(path: &str, message: impl Into<String>) -> Self {
        Diag {
            path: path.to_string(),
            line: 0,
            message: message.into(),
            expected: None,
        }
    }
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(f, "{}:{}: {}", self.path, self.line, self.message)?;
        } else {
            write!(f, "{}: {}", self.path, self.message)?;
        }
        if let Some(e) = &self.expected {
            write!(f, "\n  expected: {e}")?;
        }
        Ok(())
    }
}

/// Identifier charset, shared by spec, requirement and scenario ids.
///
/// Lowercase kebab-case. Spec ids additionally admit `/` as a namespace separator — the slash is
/// part of the id string, never derived from the filesystem (D11, azimuth/formats/spec.md).
pub fn validate_id(id: &str, allow_slash: bool) -> Result<(), String> {
    if id.is_empty() {
        return Err("id is empty".into());
    }
    for segment in id.split('/') {
        if !allow_slash && segment.len() != id.len() {
            return Err("`/` is only allowed in spec ids".into());
        }
        if segment.is_empty() {
            return Err(format!("`{id}` has an empty path segment"));
        }
        let bytes = segment.as_bytes();
        if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
            return Err(format!("`{segment}` starts or ends with `-`"));
        }
        for &b in bytes {
            let ok = b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
            if !ok {
                return Err(format!(
                    "`{segment}` contains `{}`; ids are lowercase kebab-case",
                    b as char
                ));
            }
        }
    }
    Ok(())
}
