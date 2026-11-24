use std::collections::BTreeSet;

use toml::{Table, Value};
use unicode_normalization::UnicodeNormalization;

use super::{Key, KeyType, KEY_FILE};
use crate::reading::ReadingError;
use crate::source::Source;

const FIELDS: [&str; 3] = ["type", "values", "components"];
const KIND: &str = "execution context";

pub fn read(source: &dyn Source) -> (Vec<Key>, Vec<ReadingError>) {
    let Some(bytes) = source.read(KEY_FILE) else {
        return (Vec::new(), Vec::new());
    };
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => return (Vec::new(), vec![ReadingError::new(KIND, KEY_FILE, 1, "the key declarations are not valid TOML: the file must be UTF-8")]),
    };
    let table = match text.parse::<Table>() {
        Ok(table) => table,
        Err(error) => {
            let line = error.span().map_or(1, |span| text[..span.start].bytes().filter(|byte| *byte == b'\n').count() as u64 + 1);
            let message = format!("the key declarations are not valid TOML: {}", error.message());
            return (Vec::new(), vec![ReadingError::new(KIND, KEY_FILE, line, message)]);
        }
    };
    let mut reader = Reader { keys: Vec::new(), errors: Vec::new() };
    reader.children(&table, "");
    (reader.keys, reader.errors)
}

struct Reader {
    keys: Vec<Key>,
    errors: Vec<ReadingError>,
}

impl Reader {
    fn error(&mut self, name: &str, message: &str) {
        self.errors.push(ReadingError::new(KIND, KEY_FILE, 1, format!("the key table `{name}`: {message}")));
    }

    fn children(&mut self, table: &Table, parent: &str) {
        for (segment, value) in table {
            self.child(segment, value, parent);
        }
    }

    fn child(&mut self, segment: &str, value: &Value, parent: &str) {
        let name = if parent.is_empty() { segment.to_string() } else { format!("{parent}.{segment}") };
        let mut bytes = segment.bytes();
        if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            self.error(&name, "each name segment must start with an ASCII lowercase letter and continue with ASCII lowercase letters, digits or hyphens");
        }
        match value {
            Value::Table(child) => self.table(child, &name),
            _ => self.error(&name, "intermediate tables may hold only nested key tables"),
        }
    }

    fn table(&mut self, table: &Table, name: &str) {
        // A declaration may also have child tables: a prefix collision is readable and
        // belongs to lint, so reading must preserve both keys.
        let declaration = table.is_empty() || table.values().any(|value| !value.is_table());
        if !declaration {
            self.children(table, name);
            return;
        }
        for (field, value) in table {
            if !FIELDS.contains(&field.as_str()) && !value.is_table() {
                self.error(name, &format!("unknown field `{field}`; key fields are `type`, `values` and `components`, and intermediate tables hold only nested key tables"));
            }
        }
        let kind = match table.get("type") {
            Some(Value::String(kind)) => match kind.as_str() {
                "boolean" => Some(KeyType::Boolean),
                "number" => Some(KeyType::Number),
                "one of" => self.strings(table, name, "values").map(KeyType::OneOf),
                _ => {
                    self.error(name, "unknown `type`; use `boolean`, `one of` or `number`");
                    None
                }
            },
            Some(_) => {
                self.error(name, "`type` must be a string: `boolean`, `one of` or `number`");
                None
            }
            None => {
                self.error(name, "missing `type`; declare `boolean`, `one of` or `number`");
                None
            }
        };
        if !matches!(table.get("type"), Some(Value::String(kind)) if kind == "one of") && table.contains_key("values") {
            self.error(name, "`values` is allowed only for a key of type `one of`");
        }
        let components = self.strings(table, name, "components");
        if let (Some(kind), Some(components)) = (kind, components) {
            let key = Key { name: name.to_string(), kind, components };
            if let KeyType::OneOf(values) = &key.kind {
                if values.is_empty() {
                    self.error(name, "`values` must be a nonempty list of distinct strings");
                }
                let mut distinct = BTreeSet::new();
                for value in values {
                    if !distinct.insert(value.nfc().collect::<String>()) {
                        self.error(name, "`values` repeats a string after NFC normalization");
                    }
                }
            }
            self.keys.push(key);
        }
        for (field, value) in table.iter().filter(|(field, value)| !FIELDS.contains(&field.as_str()) && value.is_table()) {
            self.child(field, value, name);
        }
    }

    fn strings(&mut self, table: &Table, name: &str, field: &str) -> Option<Vec<String>> {
        match table.get(field) {
            Some(Value::Array(values)) => {
                let strings: Option<Vec<String>> = values.iter().map(|value| value.as_str().map(str::to_string)).collect();
                if strings.is_none() {
                    self.error(name, &format!("`{field}` must contain only strings"));
                }
                strings
            }
            Some(_) => {
                self.error(name, &format!("`{field}` must be a list of strings"));
                None
            }
            None => {
                self.error(name, &format!("missing `{field}`; declare a list of strings"));
                None
            }
        }
    }
}
