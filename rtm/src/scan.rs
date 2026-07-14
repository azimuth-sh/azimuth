//! Scan source for the linkage-tag convention. A tag is a line-leading comment — `covers` on a
//! test, `realizes` on production code — followed by the item it applies to. Matching only
//! line-leading comments keeps the scanner from catching the marker strings inside its own
//! parsing code. The convention is a plain comment so any language can carry it; here it reads
//! Rust `.rs` files.
//!
//! `covers: <spec> <req> <scenario> <scope> <quant> [oracle]`  ·  `realizes: <spec> <req> <scenario>`

use crate::{Form, Key, Oracle, Quantification, Realization, Scope, Tag};
use std::path::Path;

const COVERS: &str = "covers:";
const REALIZES: &str = "realizes:";

pub fn scan_dir(dir: &str) -> (Vec<Tag>, Vec<Realization>) {
    let mut tags = Vec::new();
    let mut realizations = Vec::new();
    walk(Path::new(dir), &mut tags, &mut realizations);
    (tags, realizations)
}

fn walk(path: &Path, tags: &mut Vec<Tag>, realizations: &mut Vec<Realization>) {
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                walk(&entry.path(), tags, realizations);
            }
        }
    } else if matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "cs" | "ts" | "tsx")
    ) {
        if let Ok(text) = std::fs::read_to_string(path) {
            scan_text(&text, tags, realizations);
        }
    }
}

fn scan_text(text: &str, tags: &mut Vec<Tag>, realizations: &mut Vec<Realization>) {
    let lines: Vec<&str> = text.lines().collect();

    for (index, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        let comment = match line.strip_prefix("//") {
            Some(rest) => rest.trim_start(),
            None => continue,
        };

        if let Some(body) = comment.strip_prefix(COVERS) {
            if let Some((key, form, oracle)) = parse_covers(body) {
                tags.push(Tag {
                    key,
                    form,
                    oracle,
                    site: item_after(&lines, index),
                });
            }
        } else if let Some(body) = comment.strip_prefix(REALIZES) {
            if let Some(key) = parse_realizes(body) {
                realizations.push(Realization {
                    key,
                    site: item_after(&lines, index),
                });
            }
        }
    }
}

fn parse_covers(body: &str) -> Option<(Key, Form, Option<Oracle>)> {
    let fields: Vec<&str> = body.split_whitespace().collect();
    let (form, oracle) = match fields.as_slice() {
        [_, _, _, scope, quant] => (form_of(scope, quant)?, None),
        [_, _, _, scope, quant, oracle] => (form_of(scope, quant)?, Oracle::parse(oracle)),
        _ => return None,
    };
    Some((key(fields[0], fields[1], fields[2]), form, oracle))
}

fn form_of(scope: &str, quant: &str) -> Option<Form> {
    Some(Form::new(
        Scope::parse(scope)?,
        Quantification::parse(quant)?,
    ))
}

fn parse_realizes(body: &str) -> Option<Key> {
    let fields: Vec<&str> = body.split_whitespace().collect();
    match fields.as_slice() {
        [spec, req, scenario] => Some(key(spec, req, scenario)),
        _ => None,
    }
}

fn key(spec: &str, req: &str, scenario: &str) -> Key {
    Key {
        spec_id: spec.to_string(),
        req_id: req.to_string(),
        scenario_id: scenario.to_string(),
    }
}

/// The name of the declared item at or after the marker — the tagged site. Recognises a keyword
/// declaration (`fn`/`struct`/`enum`/`class`/`function`/`def`, across Rust, C#, TS, Python) or,
/// failing that, a method-style `Name(` — enough to name sites in several languages.
fn item_after(lines: &[&str], from: usize) -> String {
    for line in lines.iter().skip(from + 1).take(8) {
        for keyword in ["fn ", "struct ", "enum ", "class ", "function ", "def "] {
            if let Some(pos) = line.find(keyword) {
                if let Some(name) = ident(&line[pos + keyword.len()..]) {
                    return name;
                }
            }
        }
        if let Some(name) = method_name(line) {
            return name;
        }
    }
    "unknown".to_string()
}

fn ident(text: &str) -> Option<String> {
    let name: String = text
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// The identifier immediately before the first `(` — a method/function declaration or call, used
/// for languages (C#) whose method declarations have no leading keyword.
fn method_name(line: &str) -> Option<String> {
    let open = line.find('(')?;
    let before: String = line[..open]
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let name: String = before.chars().rev().collect();
    (!name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_'))
    .then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The fixtures are single-line literals with embedded newlines so that no line in this file
    // itself begins with the marker — otherwise the scanner would read its own test fixtures as
    // real tags when it walks the source tree.

    #[test]
    fn reads_a_covers_marker_and_its_site() {
        let text = "// covers: demo do-thing it-works unit example\nfn it_works_test() {}\n";
        let mut tags = Vec::new();
        let mut realizations = Vec::new();
        scan_text(text, &mut tags, &mut realizations);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].key.scenario_id, "it-works");
        assert_eq!(
            tags[0].form,
            Form::new(Scope::Unit, Quantification::Example)
        );
        assert_eq!(tags[0].site, "it_works_test");
    }

    #[test]
    fn reads_an_optional_oracle_field() {
        let text =
            "// covers: demo rank stable component invariant metamorphic\nfn stable_test() {}\n";
        let mut tags = Vec::new();
        let mut realizations = Vec::new();
        scan_text(text, &mut tags, &mut realizations);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].oracle, Some(Oracle::Metamorphic));
    }

    #[test]
    fn ignores_the_marker_string_when_not_line_leading() {
        let text = "let pattern = \"covers: a b c unit example\";";
        let mut tags = Vec::new();
        let mut realizations = Vec::new();
        scan_text(text, &mut tags, &mut realizations);
        assert!(tags.is_empty());
    }
}
