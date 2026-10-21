//! Rewrites authored account syntax from the `alpha-claim-case` line to the Requirement, Scenario,
//! Verification Target and Verification Challenge vocabulary.
//!
//! Only whitelisted syntax positions change: heading labels, metadata labels and the closed
//! vocabularies carried by `Select`, `Required scope` and `Targets`. Identifiers, prose and fenced
//! blocks are never touched, so stable Claim and Case identifiers survive even when they contain the
//! retired words, and recorded history text keeps its original wording.

use crate::json::Json;

const HEADINGS: &[(&str, &str)] = &[
    ("Claim", "Requirement"),
    ("Case", "Scenario"),
    ("Claim design", "Requirement design"),
    ("Case design", "Scenario design"),
    ("Claim verification", "Requirement verification"),
    ("Case verification", "Scenario verification"),
    ("Claim review", "Requirement review"),
    ("Claim health", "Requirement status"),
    ("Claim judgment", "Requirement judgment"),
    ("Claim Judgment", "Requirement Judgment"),
    ("Challenger", "Verification Challenge"),
    ("Add claim", "Add requirement"),
    ("Add case", "Add scenario"),
    ("Replace claim", "Replace requirement"),
    ("Remove claim", "Remove requirement"),
    ("Remove case", "Remove scenario"),
];

const LABELS: &[(&str, &str)] = &[
    ("Claim", "Requirement"),
    ("Claims", "Requirements"),
    ("Case", "Scenario"),
    ("Cases", "Scenarios"),
    ("Challenger", "Verification Challenge"),
    ("Subjects", "Verification Targets"),
    ("Remove cases", "Remove scenarios"),
];

const KINDS: &[(&str, &str)] = &[
    ("claim", "requirement"),
    ("case", "scenario"),
    ("claim-review", "requirement-review"),
    ("claim-health", "requirement-status"),
    ("claim-judgment", "requirement-judgment"),
];

fn mapped(table: &[(&str, &'static str)], value: &str) -> Option<&'static str> {
    table
        .iter()
        .find(|(old, _)| *old == value)
        .map(|(_, new)| *new)
}

fn fence(line: &str) -> bool {
    let indent = line.bytes().take_while(|byte| *byte == b' ').count();
    indent <= 3 && (line[indent..].starts_with("```") || line[indent..].starts_with("~~~"))
}

fn heading(line: &str) -> Option<(&str, &str, &str)> {
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&level) || line.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    let rest = &line[level + 1..];
    let (label, tail) = rest.split_once(':')?;
    Some((&line[..level + 1], label, tail))
}

fn metadata(line: &str) -> Option<(&str, &str, &str)> {
    let indent = line.bytes().take_while(|byte| *byte == b' ').count();
    let marker = if line[indent..].starts_with("- ") {
        2
    } else {
        0
    };
    let prefix_end = indent + marker;
    let rest = &line[prefix_end..];
    let (label, tail) = rest.split_once(':')?;
    if label.is_empty() || !(tail.is_empty() || tail.starts_with(' ')) {
        return None;
    }
    Some((&line[..prefix_end], label, tail))
}

fn rewrite_select(value: &str) -> Option<String> {
    let (decision, remainder) = value.trim_start().split_once(" from ")?;
    let (kind, id) = remainder.split_once(' ').unwrap_or((remainder, ""));
    if mapped(KINDS, decision).is_none() && mapped(KINDS, kind).is_none() {
        return None;
    }
    let decision = mapped(KINDS, decision).unwrap_or(decision);
    let kind = mapped(KINDS, kind).unwrap_or(kind);
    let id = if id.is_empty() {
        String::new()
    } else {
        format!(" {id}")
    };
    Some(format!(" {decision} from {kind}{id}"))
}

fn rewrite_scope(value: &str) -> String {
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find('"') {
        let Some(length) = rest[start + 1..].find('"') else {
            break;
        };
        let item = &rest[start + 1..start + 1 + length];
        out.push_str(&rest[..=start]);
        out.push_str(mapped(KINDS, item).unwrap_or(item));
        out.push('"');
        rest = &rest[start + length + 2..];
    }
    out.push_str(rest);
    out
}

fn rewrite_targets(value: &str) -> String {
    value
        .split(',')
        .map(|item| {
            let trimmed = item.trim();
            match mapped(KINDS, trimmed) {
                Some(new) => item.replacen(trimmed, new, 1),
                None => item.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Evidence Binding and Method Qualification blocks carry the retired `Context:` label for their
/// Execution Context; elsewhere `Context:` is ordinary authored text and stays.
fn declares_execution_context(line: &str) -> Option<bool> {
    heading(line).map(|(_, label, _)| matches!(label, "Evidence Binding" | "Method Qualification"))
}

fn rewrite_line(line: &str, execution_context_block: bool) -> String {
    if let Some((marks, label, tail)) = heading(line) {
        if let Some(new) = mapped(HEADINGS, label) {
            return format!("{marks}{new}:{tail}");
        }
        return line.to_string();
    }
    let Some((prefix, label, tail)) = metadata(line) else {
        return line.to_string();
    };
    if let Some(new) = mapped(LABELS, label) {
        return format!("{prefix}{new}:{tail}");
    }
    if execution_context_block && prefix.is_empty() && label == "Context" {
        return format!("Execution Context:{tail}");
    }
    match label {
        "Select" => rewrite_select(tail)
            .map(|value| format!("{prefix}{label}:{value}"))
            .unwrap_or_else(|| line.to_string()),
        "Required scope" => format!("{prefix}{label}:{}", rewrite_scope(tail)),
        "Targets" => format!("{prefix}{label}:{}", rewrite_targets(tail)),
        _ => line.to_string(),
    }
}

/// Rewrites one authored Markdown document. Line endings and fenced content are preserved exactly.
pub fn rewrite_markdown(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut fenced = false;
    let mut execution_context_block = false;
    for line in source.split_inclusive('\n') {
        let (body, ending) = match line.strip_suffix('\n') {
            Some(body) => match body.strip_suffix('\r') {
                Some(body) => (body, "\r\n"),
                None => (body, "\n"),
            },
            None => (line, ""),
        };
        if fence(body) {
            fenced = !fenced;
            out.push_str(line);
            continue;
        }
        if fenced {
            out.push_str(line);
        } else {
            if let Some(declares) = declares_execution_context(body) {
                execution_context_block = declares;
            }
            out.push_str(&rewrite_line(body, execution_context_block));
            out.push_str(ending);
        }
    }
    out
}

/// Lines of an active artifact that still use retired syntax, as `(line number, line)`.
pub fn retired_syntax(source: &str) -> Vec<(usize, String)> {
    let mut fenced = false;
    let mut execution_context_block = false;
    let mut found = Vec::new();
    for (index, line) in source.lines().enumerate() {
        if fence(line) {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(declares) = declares_execution_context(line) {
            execution_context_block = declares;
        }
        if rewrite_line(line, execution_context_block) != line {
            found.push((index + 1, line.to_string()));
        }
    }
    found
}

/// Rewrites a version 1 workspace to version 2, renaming realization obligation `claim` keys.
/// Returns `None` when the document is already version 2.
pub fn rewrite_workspace(source: &str) -> Result<Option<String>, String> {
    let mut root = crate::json::parse(source)?;
    if root.get("format").and_then(Json::as_str) != Some("azimuth-workspace") {
        return Err("expected azimuth-workspace".into());
    }
    match root.get("version").and_then(Json::as_num) {
        Some(version) if version == 2.0 => return Ok(None),
        Some(version) if version == 1.0 => {}
        _ => return Err("unsupported workspace version".into()),
    }
    let Json::Obj(fields) = &mut root else {
        return Err("workspace must be an object".into());
    };
    for (key, value) in fields.iter_mut() {
        if key == "version" {
            *value = Json::Num(2.0);
        }
        if key == "realization_obligations" {
            let Json::Arr(items) = value else {
                return Err("realization_obligations must be an array".into());
            };
            for item in items {
                let Json::Obj(obligation) = item else {
                    return Err("realization obligation must be an object".into());
                };
                for (field, _) in obligation.iter_mut() {
                    if field == "claim" {
                        *field = "requirement".into();
                    }
                }
            }
        }
    }
    Ok(Some(root.to_string_pretty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_syntax_positions_and_preserves_identifiers_prose_and_fences() {
        let source = "# Spec: claims\n\n## Claim: verification-context-is-observed\nCriticality: standard\n\n\
                      The claim holds in case of retries.\n\n### Case: case-one\nIt works.\n\n\
                      ```md\n## Claim: fenced\n```\n\n## Mechanism Case scope verification: prose\n";
        let expected = "# Spec: claims\n\n## Requirement: verification-context-is-observed\nCriticality: standard\n\n\
                        The claim holds in case of retries.\n\n### Scenario: case-one\nIt works.\n\n\
                        ```md\n## Claim: fenced\n```\n\n## Mechanism Case scope verification: prose\n";
        assert_eq!(rewrite_markdown(source), expected);
        assert!(retired_syntax(expected).is_empty());
        assert_eq!(retired_syntax(source).len(), 2);
    }

    #[test]
    fn rewrites_metadata_labels_and_closed_vocabulary_values() {
        let source = "### Challenger: mutation/search\n- Form: mutation\n- Targets: claim-review, claim-health\n\
                      - Claims: claim-review\n  - Subjects: artifact\nRequired scope: [\"claim\",\"case\",\"mechanism\"]\n\
                      Select: claim-judgment from claim claim\nSelect: applicability-decision from case case-id\n\
                      Remove cases: a, b\n";
        let expected = "### Verification Challenge: mutation/search\n- Form: mutation\n- Targets: requirement-review, requirement-status\n\
                        - Requirements: claim-review\n  - Verification Targets: artifact\nRequired scope: [\"requirement\",\"scenario\",\"mechanism\"]\n\
                        Select: requirement-judgment from requirement claim\nSelect: applicability-decision from scenario case-id\n\
                        Remove scenarios: a, b\n";
        assert_eq!(rewrite_markdown(source), expected);
    }

    #[test]
    fn renames_context_only_inside_binding_and_qualification_blocks() {
        let source = "## Evidence Binding: a/edge\nCheck: c\nContext: {}\n\n\
                      ## Method Qualification: a/method\nCheck: c\nContext: {\"os\":\"linux\"}\n\n\
                      ## Check: c\nContext: unrelated\n\n### Scenario: s\n- Context: given state\n\n\
                      ## Evidence Binding: b/edge\n```md\nContext: {}\n```\n- Context: listed\n";
        let expected = "## Evidence Binding: a/edge\nCheck: c\nExecution Context: {}\n\n\
                        ## Method Qualification: a/method\nCheck: c\nExecution Context: {\"os\":\"linux\"}\n\n\
                        ## Check: c\nContext: unrelated\n\n### Scenario: s\n- Context: given state\n\n\
                        ## Evidence Binding: b/edge\n```md\nContext: {}\n```\n- Context: listed\n";
        assert_eq!(rewrite_markdown(source), expected);
        assert_eq!(retired_syntax(source).len(), 2);
        assert!(retired_syntax(expected).is_empty());
    }

    #[test]
    fn rewrites_workspace_version_and_obligation_keys_once() {
        let source = "{\"format\":\"azimuth-workspace\",\"version\":1,\"areas\":[],\"surfaces\":[],\
                      \"realization_obligations\":[{\"spec\":\"s\",\"claim\":\"c\",\"areas\":[\"a\"]}]}";
        let rewritten = rewrite_workspace(source).unwrap().unwrap();
        assert!(rewritten.contains("\"version\": 2"));
        assert!(rewritten.contains("\"requirement\": \"c\""));
        assert!(!rewritten.contains("\"claim\""));
        assert_eq!(rewrite_workspace(&rewritten).unwrap(), None);
    }
}
