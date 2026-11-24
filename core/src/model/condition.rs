//! Conditions: the `Context:` lines under a Requirement or Scenario, their typed values and the
//! effective condition of a Check.

use std::collections::BTreeMap;

use unicode_normalization::UnicodeNormalization;

use crate::canonical;

/// The largest number of digits that writing a number without an exponent may add. A number such
/// as `1e999999999` has an exact decimal, but writing it out would fill memory.
const MOST_ZEROS: i64 = 1000;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Boolean(bool),
    /// Decoded from its JSON string and normalized to NFC.
    Text(String),
    /// Written without an exponent, with no unnecessary zeros.
    Number(String),
}

impl Literal {
    /// The spelling that compares and digests: a one-of value keeps its quotes, in the JSON
    /// string spelling of RFC 8785, so `true` and `"true"` never compare equal.
    pub fn normalized(&self) -> String {
        match self {
            Literal::Boolean(flag) => flag.to_string(),
            Literal::Text(text) => canonical::json_string(text),
            Literal::Number(number) => number.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ContextLine {
    pub line: u64,
    pub key: String,
    pub value: Literal,
}

/// The condition a Check runs under: each key once, with its normalized value. Where lines
/// disagree about a key the first one stands, because lint reports the disagreement.
#[derive(Debug, Default, PartialEq)]
pub struct Condition {
    pub values: BTreeMap<String, String>,
}

impl Condition {
    pub fn extend(&mut self, lines: &[ContextLine]) {
        for line in lines {
            self.values.entry(line.key.clone()).or_insert_with(|| line.value.normalized());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The lines that follow the claim text in the claim part, sorted by key.
    pub fn claim_lines(&self) -> String {
        self.values.iter().map(|(key, value)| format!("\nContext: {key}={value}")).collect()
    }

    pub fn describe(&self) -> String {
        self.values.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join(", ")
    }
}

/// A key is lowercase segments joined by dots; a segment starts with an ASCII lowercase letter
/// and continues with ASCII lowercase letters, digits or hyphens.
fn is_valid_key(key: &str) -> bool {
    key.split('.').all(|segment| {
        let mut bytes = segment.bytes();
        bytes.next().is_some_and(|byte| byte.is_ascii_lowercase()) && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    })
}

/// The text after `Context:` read as one assignment. The error says how to write it.
pub fn parse(rest: &str) -> Result<(String, Literal), String> {
    let rest = rest.trim_matches([' ', '\t']);
    let Some((key, value)) = rest.split_once('=') else {
        return Err("the `Context:` line has no `key=value` assignment. Write `Context: <key>=<value>`".to_string());
    };
    let key = key.trim_matches([' ', '\t']);
    if !is_valid_key(key) {
        return Err(format!(
            "the `Context:` key `{key}` is not spelled as a key. A key is lowercase segments joined by dots, and a segment starts with a lowercase Latin letter and continues with lowercase Latin letters, digits or hyphens"
        ));
    }
    let value = value.trim_matches([' ', '\t']);
    if value.is_empty() {
        return Err(format!("the `Context:` line has no value after `{key}=`. Write `Context: {key}=<value>`"));
    }
    Ok((key.to_string(), parse_value(value)?))
}

const ONE_ASSIGNMENT: &str = "A `Context:` line states one key and one value; write each further key on a line of its own";

fn parse_value(value: &str) -> Result<Literal, String> {
    if value.starts_with('"') {
        return parse_string(value);
    }
    if value.contains('=') {
        return Err(format!("the `Context:` line holds more than one assignment. {ONE_ASSIGNMENT}"));
    }
    if value.contains([' ', '\t']) {
        return Err(format!("the `Context:` value `{value}` is more than one value. {ONE_ASSIGNMENT}, and a condition states no range"));
    }
    if value.eq_ignore_ascii_case("true") {
        return Ok(Literal::Boolean(true));
    }
    if value.eq_ignore_ascii_case("false") {
        return Ok(Literal::Boolean(false));
    }
    if value.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Err(format!(
            "the `Context:` value `{value}` is not a boolean and not a number. Write a one-of value as a JSON string in quotes, such as `\"{value}\"`"
        ));
    }
    normalize_number(value).map(Literal::Number)
}

fn parse_string(value: &str) -> Result<Literal, String> {
    let mut escaped = false;
    let mut end = None;
    for (index, c) in value.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            end = Some(index + 1);
            break;
        }
    }
    let Some(end) = end else {
        return Err(format!("the `Context:` value {value} is an unfinished JSON string. Close it with a quote"));
    };
    let (token, rest) = value.split_at(end);
    let rest = rest.trim_matches([' ', '\t']);
    if rest.contains('=') {
        return Err(format!("the `Context:` line holds more than one assignment. {ONE_ASSIGNMENT}"));
    }
    if !rest.is_empty() {
        return Err(format!("the `Context:` value has `{rest}` after its JSON string. {ONE_ASSIGNMENT}, and a condition states no range"));
    }
    match serde_json::from_str::<String>(token) {
        Ok(text) => Ok(Literal::Text(text.nfc().collect())),
        Err(error) => Err(format!("the `Context:` value {token} is not a JSON string: {error}")),
    }
}

/// A number in JSON decimal notation, written without an exponent. The digits are kept exactly,
/// so no floating-point approximation can make two numbers equal.
fn normalize_number(text: &str) -> Result<String, String> {
    let malformed = || format!("the `Context:` value `{text}` is a malformed number. A number uses the JSON decimal notation: an optional minus, digits with no leading zero, an optional fraction and an optional exponent");
    let bytes = text.as_bytes();
    let mut at = 0;
    let negative = bytes.first() == Some(&b'-');
    if negative {
        at += 1;
    }
    let digits = |at: &mut usize| {
        let start = *at;
        while bytes.get(*at).is_some_and(u8::is_ascii_digit) {
            *at += 1;
        }
        &text[start..*at]
    };
    let whole = digits(&mut at);
    if whole.is_empty() || (whole.len() > 1 && whole.starts_with('0')) {
        return Err(malformed());
    }
    let mut fraction = "";
    if bytes.get(at) == Some(&b'.') {
        at += 1;
        fraction = digits(&mut at);
        if fraction.is_empty() {
            return Err(malformed());
        }
    }
    let mut exponent: i64 = 0;
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at += 1;
        let mut exponent_negative = false;
        if let Some(sign @ (b'+' | b'-')) = bytes.get(at) {
            exponent_negative = *sign == b'-';
            at += 1;
        }
        let exponent_digits = digits(&mut at);
        if exponent_digits.is_empty() {
            return Err(malformed());
        }
        exponent = match exponent_digits.trim_start_matches('0') {
            trimmed if trimmed.len() > 9 => return Err(too_long(text)),
            trimmed => trimmed.parse().unwrap_or(0),
        };
        if exponent_negative {
            exponent = -exponent;
        }
    }
    if at != bytes.len() {
        return Err(malformed());
    }
    let mut all = format!("{whole}{fraction}");
    let mut scale = exponent - fraction.len() as i64;
    all = all.trim_start_matches('0').to_string();
    let trailing = all.len() - all.trim_end_matches('0').len();
    all.truncate(all.len() - trailing);
    scale += trailing as i64;
    if all.is_empty() {
        return Ok("0".to_string());
    }
    let sign = if negative { "-" } else { "" };
    let length = all.len() as i64;
    let written = if scale >= 0 {
        if scale > MOST_ZEROS {
            return Err(too_long(text));
        }
        format!("{all}{}", "0".repeat(scale as usize))
    } else if length + scale > 0 {
        let point = (length + scale) as usize;
        format!("{}.{}", &all[..point], &all[point..])
    } else {
        if -(length + scale) > MOST_ZEROS {
            return Err(too_long(text));
        }
        format!("0.{}{all}", "0".repeat((-(length + scale)) as usize))
    };
    Ok(format!("{sign}{written}"))
}

fn too_long(text: &str) -> String {
    format!("the `Context:` number `{text}` would take more than {MOST_ZEROS} zeros to write without an exponent. State a number that is nearer to zero")
}
