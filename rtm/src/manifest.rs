//! Read a language-neutral linkage manifest (`*.manifest.json`) into tags and realizations — the
//! polyglot path alongside the comment scanner. A codebase emits its `realizes`/`covers` tags as
//! JSON (see `schema/manifest.schema.json`) and `rtm` ingests them the same as scanned comments.
//! std-only: a tiny JSON reader lives in the `json` submodule so the tool builds offline.

use crate::{Form, Key, Oracle, Quantification, Realization, Scope, Tag, UntracedTest};
use std::path::Path;

pub fn read_manifest(path: &Path) -> (Vec<Tag>, Vec<Realization>, Vec<UntracedTest>) {
    match std::fs::read_to_string(path) {
        Ok(text) => parse_manifest(&text),
        Err(_) => (Vec::new(), Vec::new(), Vec::new()),
    }
}

// realizes: azimuth-rtm ingest-manifest manifest-entries-ingested
pub fn parse_manifest(text: &str) -> (Vec<Tag>, Vec<Realization>, Vec<UntracedTest>) {
    let mut tags = Vec::new();
    let mut realizations = Vec::new();
    let mut untraced = Vec::new();

    let Some(root) = json::parse(text) else {
        return (tags, realizations, untraced);
    };

    if let Some(items) = root.get("realizes").and_then(json::Value::as_array) {
        realizations.extend(items.iter().filter_map(realization_of));
    }
    if let Some(items) = root.get("covers").and_then(json::Value::as_array) {
        tags.extend(items.iter().filter_map(tag_of));
    }
    if let Some(items) = root.get("untraced_tests").and_then(json::Value::as_array) {
        untraced.extend(items.iter().filter_map(untraced_of));
    }

    (tags, realizations, untraced)
}

fn key_of(item: &json::Value) -> Option<Key> {
    Some(Key {
        spec_id: item.get("spec")?.as_str()?.to_string(),
        req_id: item.get("req")?.as_str()?.to_string(),
        scenario_id: item.get("scenario")?.as_str()?.to_string(),
    })
}

fn site_of(item: &json::Value) -> String {
    item.get("site")
        .and_then(json::Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn realization_of(item: &json::Value) -> Option<Realization> {
    Some(Realization {
        key: key_of(item)?,
        site: site_of(item),
    })
}

fn untraced_of(item: &json::Value) -> Option<UntracedTest> {
    Some(UntracedTest {
        site: item.get("site")?.as_str()?.to_string(),
        file: item
            .get("file")
            .and_then(json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
    })
}

fn tag_of(item: &json::Value) -> Option<Tag> {
    let scope = Scope::parse(item.get("scope")?.as_str()?)?;
    let quantification = Quantification::parse(item.get("quantification")?.as_str()?)?;
    let oracle = item
        .get("oracle")
        .and_then(json::Value::as_str)
        .and_then(Oracle::parse);
    Some(Tag {
        key: key_of(item)?,
        form: Form::new(scope, quantification),
        oracle,
        site: site_of(item),
    })
}

/// A minimal recursive-descent JSON reader — just enough to walk a manifest's objects, arrays, and
/// string fields without pulling a serde dependency (the core stays std-only). Lenient: a malformed
/// document yields `None` and the manifest is treated as empty rather than aborting the run.
mod json {
    // Scalars a manifest never reads (numbers, bools, null) are recognised so a valid document
    // parses, but carry no payload — the reader only inspects strings, arrays, and objects.
    #[derive(Debug, Clone)]
    pub enum Value {
        Null,
        Bool,
        Num,
        Str(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    impl Value {
        pub fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Object(entries) => entries
                    .iter()
                    .find(|(name, _)| name == key)
                    .map(|(_, value)| value),
                _ => None,
            }
        }

        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::Str(text) => Some(text),
                _ => None,
            }
        }

        pub fn as_array(&self) -> Option<&[Value]> {
            match self {
                Value::Array(items) => Some(items),
                _ => None,
            }
        }
    }

    pub fn parse(text: &str) -> Option<Value> {
        let mut parser = Parser {
            chars: text.chars().collect(),
            pos: 0,
        };
        let value = parser.value()?;
        parser.skip_ws();
        parser.at_end().then_some(value)
    }

    struct Parser {
        chars: Vec<char>,
        pos: usize,
    }

    impl Parser {
        fn peek(&self) -> Option<char> {
            self.chars.get(self.pos).copied()
        }

        fn bump(&mut self) -> Option<char> {
            let current = self.peek();
            if current.is_some() {
                self.pos += 1;
            }
            current
        }

        fn at_end(&self) -> bool {
            self.pos >= self.chars.len()
        }

        fn skip_ws(&mut self) {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.pos += 1;
            }
        }

        fn value(&mut self) -> Option<Value> {
            self.skip_ws();
            match self.peek()? {
                '{' => self.object(),
                '[' => self.array(),
                '"' => self.string().map(Value::Str),
                't' | 'f' => self.boolean(),
                'n' => self.keyword("null", Value::Null),
                _ => self.number(),
            }
        }

        fn object(&mut self) -> Option<Value> {
            self.expect('{')?;
            let mut entries = Vec::new();
            self.skip_ws();
            if self.peek() == Some('}') {
                self.pos += 1;
                return Some(Value::Object(entries));
            }
            loop {
                self.skip_ws();
                let key = self.string()?;
                self.skip_ws();
                self.expect(':')?;
                let value = self.value()?;
                entries.push((key, value));
                self.skip_ws();
                match self.bump()? {
                    ',' => continue,
                    '}' => return Some(Value::Object(entries)),
                    _ => return None,
                }
            }
        }

        fn array(&mut self) -> Option<Value> {
            self.expect('[')?;
            let mut items = Vec::new();
            self.skip_ws();
            if self.peek() == Some(']') {
                self.pos += 1;
                return Some(Value::Array(items));
            }
            loop {
                items.push(self.value()?);
                self.skip_ws();
                match self.bump()? {
                    ',' => continue,
                    ']' => return Some(Value::Array(items)),
                    _ => return None,
                }
            }
        }

        fn string(&mut self) -> Option<String> {
            self.expect('"')?;
            let mut text = String::new();
            loop {
                match self.bump()? {
                    '"' => return Some(text),
                    '\\' => text.push(self.escape()?),
                    c => text.push(c),
                }
            }
        }

        fn escape(&mut self) -> Option<char> {
            Some(match self.bump()? {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                'b' => '\u{0008}',
                'f' => '\u{000C}',
                'u' => {
                    let mut code = 0u32;
                    for _ in 0..4 {
                        code = code * 16 + self.bump()?.to_digit(16)?;
                    }
                    char::from_u32(code)?
                }
                _ => return None,
            })
        }

        fn boolean(&mut self) -> Option<Value> {
            self.keyword("true", Value::Bool)
                .or_else(|| self.keyword("false", Value::Bool))
        }

        fn keyword(&mut self, word: &str, value: Value) -> Option<Value> {
            let end = self.pos + word.len();
            let matches = end <= self.chars.len()
                && self.chars[self.pos..end].iter().copied().eq(word.chars());
            matches.then(|| {
                self.pos = end;
                value
            })
        }

        fn number(&mut self) -> Option<Value> {
            let start = self.pos;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'))
            {
                self.pos += 1;
            }
            let text: String = self.chars[start..self.pos].iter().collect();
            text.parse::<f64>().ok().map(|_| Value::Num)
        }

        fn expect(&mut self, expected: char) -> Option<()> {
            (self.bump()? == expected).then_some(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The JSON fixtures are ordinary string literals, not line-leading `//` comments, so the
    // source scanner never mistakes them for real tags when it walks this file.

    #[test]
    // covers: azimuth-rtm ingest-manifest manifest-entries-ingested unit example
    fn ingests_realizes_and_covers_entries() {
        let text = r#"{
          "realizes": [
            { "spec": "demo", "req": "do", "scenario": "it-works",
              "site": "DoThing", "file": "a.cs", "lang": "csharp" }
          ],
          "covers": [
            { "spec": "demo", "req": "do", "scenario": "it-works",
              "scope": "component", "quantification": "invariant", "oracle": "direct",
              "site": "DoThingTest", "file": "a.test.cs", "lang": "csharp" }
          ]
        }"#;
        let (tags, realizations, _untraced) = parse_manifest(text);
        assert_eq!(realizations.len(), 1);
        assert_eq!(realizations[0].key.scenario_id, "it-works");
        assert_eq!(realizations[0].site, "DoThing");
        assert_eq!(tags.len(), 1);
        assert_eq!(
            tags[0].form,
            Form::new(Scope::Component, Quantification::Invariant)
        );
        assert_eq!(tags[0].oracle, Some(Oracle::Direct));
        assert_eq!(tags[0].site, "DoThingTest");
    }

    #[test]
    // covers: azimuth-rtm ingest-manifest manifest-entries-ingested unit example
    fn ingests_untraced_test_entries() {
        let text = r#"{
          "untraced_tests": [
            { "site": "RevokeTests.SeedsFixtures", "file": "RevokeTests.cs" }
          ]
        }"#;
        let (_, _, untraced) = parse_manifest(text);
        assert_eq!(untraced.len(), 1);
        assert_eq!(untraced[0].site, "RevokeTests.SeedsFixtures");
        assert_eq!(untraced[0].file, "RevokeTests.cs");
    }

    #[test]
    fn a_covers_entry_without_scope_or_quant_is_skipped() {
        let text = r#"{ "covers": [ { "spec": "d", "req": "r", "scenario": "s", "site": "T" } ] }"#;
        let (tags, _, _) = parse_manifest(text);
        assert!(tags.is_empty());
    }

    #[test]
    fn a_malformed_manifest_is_treated_as_empty() {
        let (tags, realizations, untraced) = parse_manifest("{ not json ");
        assert!(tags.is_empty());
        assert!(realizations.is_empty());
        assert!(untraced.is_empty());
    }

    #[test]
    fn an_omitted_array_yields_nothing() {
        let (tags, realizations, untraced) = parse_manifest(r#"{ "realizes": [] }"#);
        assert!(tags.is_empty());
        assert!(realizations.is_empty());
        assert!(untraced.is_empty());
    }
}
