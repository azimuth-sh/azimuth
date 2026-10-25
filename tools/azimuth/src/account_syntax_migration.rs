//! Rewrites authored account syntax from the `alpha-claim-case` line to the Requirement, Scenario,
//! Verification Target and Verification Challenge vocabulary.
//!
//! Only whitelisted syntax positions change: heading labels, delta operation declarations, metadata
//! labels, the closed vocabularies carried by `Select`, `Required scope` and `Targets`, and the
//! `subject` key of a Check's `Deployment:` policy. Identifiers, prose and fenced
//! blocks are never touched, so stable Claim and Case identifiers survive even when they contain the
//! retired words, and recorded history text keeps its original wording.

use crate::json::Json;

const HEADINGS: &[(&str, &str)] = &[
    ("Claim", "Requirement"),
    ("Case", "Scenario"),
    ("Claim realization", "Requirement realization"),
    ("Case realization", "Scenario realization"),
    ("Claim verification", "Requirement verification"),
    ("Case verification", "Scenario verification"),
    ("Claim review", "Verification review"),
    ("Claim health", "Requirement status"),
    ("Claim judgment", "Requirement judgment"),
    ("Claim Judgment", "Requirement Judgment"),
    ("Challenger", "Verification Challenge"),
];

const OPERATIONS: &[&str] = &["Add ", "Replace ", "Remove "];

/// Every declaration an intent or account delta operation could name on the earlier line, with its
/// canonical capitalized spelling on the current line.
const DECLARATIONS: &[(&str, &str)] = &[
    ("claim", "Requirement"),
    ("case", "Scenario"),
    ("claim realization", "Requirement realization"),
    ("case realization", "Scenario realization"),
    ("claim verification", "Requirement verification"),
    ("case verification", "Scenario verification"),
    ("claim review", "Verification review"),
    ("challenger", "Verification Challenge"),
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
    ("claim-review", "verification-review"),
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
            let kind = trimmed
                .strip_prefix('`')
                .and_then(|kind| kind.strip_suffix('`'))
                .unwrap_or(trimmed);
            match mapped(KINDS, kind) {
                Some(new) => item.replacen(kind, new, 1),
                None => item.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// `Add claim realization`, `Remove case verification`, `Add challenger` and the other delta operations.
/// A lowercase declaration keeps lowercase, as the account delta parser requires; a capitalized one
/// takes the canonical spelling.
fn rewrite_operation(label: &str) -> Option<String> {
    let operation = OPERATIONS
        .iter()
        .find(|operation| label.starts_with(**operation))?;
    let declaration = &label[operation.len()..];
    let (_, new) = DECLARATIONS
        .iter()
        .find(|(old, _)| old.eq_ignore_ascii_case(declaration))?;
    let new = if declaration.starts_with(|c: char| c.is_ascii_lowercase()) {
        new.to_lowercase()
    } else {
        new.to_string()
    };
    Some(format!("{operation}{new}"))
}

/// The heading block a line belongs to decides which positional rewrites apply to its metadata.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Block {
    Other,
    /// Evidence Binding and Method Qualification blocks carry the retired `Context:` label for their
    /// Execution Context; elsewhere `Context:` is ordinary authored text and stays.
    ExecutionContext,
    /// A Check's `Deployment:` policy names its execution scope under the retired `subject` key.
    Check,
}

fn block(line: &str) -> Option<Block> {
    heading(line).map(|(_, label, _)| {
        let declaration = OPERATIONS
            .iter()
            .find_map(|operation| label.strip_prefix(operation))
            .unwrap_or(label);
        if declaration.eq_ignore_ascii_case("check") {
            Block::Check
        } else if matches!(declaration, "Evidence Binding" | "Method Qualification") {
            Block::ExecutionContext
        } else {
            Block::Other
        }
    })
}

/// Renames only the top-level `"subject"` key of a deployment policy object; nested values, other
/// keys and string contents are left byte-identical.
fn rewrite_deployment_policy(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth = depth.saturating_sub(1),
            b'"' => {
                let start = index;
                index += 1;
                while index < bytes.len() && bytes[index] != b'"' {
                    index += if bytes[index] == b'\\' { 2 } else { 1 };
                }
                let token = &value[start..(index + 1).min(value.len())];
                let after = value[(index + 1).min(value.len())..].trim_start();
                if depth == 1 && token == "\"subject\"" && after.starts_with(':') {
                    return format!(
                        "{}\"verification_target\"{}",
                        &value[..start],
                        &value[index + 1..]
                    );
                }
            }
            _ => {}
        }
        index += 1;
    }
    value.to_string()
}

fn rewrite_line(line: &str, block: Block) -> String {
    if let Some((marks, label, tail)) = heading(line) {
        if let Some(new) = mapped(HEADINGS, label) {
            return format!("{marks}{new}:{tail}");
        }
        if let Some(new) = rewrite_operation(label) {
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
    if block == Block::ExecutionContext && prefix.is_empty() && label == "Context" {
        return format!("Execution Context:{tail}");
    }
    if block == Block::Check && label == "Deployment" {
        return format!("{prefix}{label}:{}", rewrite_deployment_policy(tail));
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
    let mut position = Position::default();
    for line in source.split_inclusive('\n') {
        let (body, ending) = match line.strip_suffix('\n') {
            Some(body) => match body.strip_suffix('\r') {
                Some(body) => (body, "\r\n"),
                None => (body, "\n"),
            },
            None => (line, ""),
        };
        match position.rewrite(body) {
            Some(rewritten) => {
                out.push_str(&rewritten);
                out.push_str(ending);
            }
            None => out.push_str(line),
        }
    }
    out
}

/// Lines of an active artifact that still use retired syntax, as `(line number, line)`.
pub fn retired_syntax(source: &str) -> Vec<(usize, String)> {
    let mut position = Position::default();
    let mut found = Vec::new();
    for (index, line) in source.lines().enumerate() {
        if position
            .rewrite(line)
            .is_some_and(|rewritten| rewritten != line)
        {
            found.push((index + 1, line.to_string()));
        }
    }
    found
}

/// Where a line sits: inside a fence, which heading block, and whether it continues a `Targets:`
/// list whose items are closed assessment kinds.
#[derive(Default)]
struct Position {
    fenced: bool,
    block: Option<Block>,
    targets_list: bool,
}

impl Position {
    /// Returns the rewritten line, or `None` for a fence marker or fenced content left verbatim.
    fn rewrite(&mut self, line: &str) -> Option<String> {
        if fence(line) {
            self.fenced = !self.fenced;
            return None;
        }
        if self.fenced {
            return None;
        }
        if let Some(next) = block(line) {
            self.block = Some(next);
            self.targets_list = false;
            return Some(rewrite_line(line, next));
        }
        if self.targets_list {
            if let Some(item) = list_item(line) {
                return Some(item);
            }
            self.targets_list = false;
        }
        if let Some((_, "Targets", tail)) = metadata(line) {
            self.targets_list = tail.trim().is_empty();
        }
        Some(rewrite_line(line, self.block.unwrap_or(Block::Other)))
    }
}

/// An indented `- \`claim-review\`` item of a `Targets:` list, with its kind renamed.
fn list_item(line: &str) -> Option<String> {
    let indent = line.bytes().take_while(|byte| *byte == b' ').count();
    let value = line[indent..].strip_prefix("- ")?;
    if indent == 0 {
        return None;
    }
    let (open, kind, close) = match value.strip_prefix('`').and_then(|v| v.strip_suffix('`')) {
        Some(kind) => ("`", kind, "`"),
        None => ("", value, ""),
    };
    let kind = mapped(KINDS, kind).unwrap_or(kind);
    Some(format!("{}- {open}{kind}{close}", &line[..indent]))
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
        let source = "# Spec: claims\n\n## Claim: verification-context-is-observed\nVerification: reviewed\n\n\
                      The claim holds in case of retries.\n\n### Case: case-one\nIt works.\n\n\
                      ```md\n## Claim: fenced\n```\n\n## Mechanism Case scope verification: prose\n";
        let expected = "# Spec: claims\n\n## Requirement: verification-context-is-observed\nVerification: reviewed\n\n\
                        The claim holds in case of retries.\n\n### Scenario: case-one\nIt works.\n\n\
                        ```md\n## Claim: fenced\n```\n\n## Mechanism Case scope verification: prose\n";
        assert_eq!(rewrite_markdown(source), expected);
        assert!(retired_syntax(expected).is_empty());
        assert_eq!(retired_syntax(source).len(), 2);
    }

    #[test]
    fn rewrites_metadata_labels_and_closed_vocabulary_values() {
        let source = "### Challenger: mutation/search\n- Form: mutation\n- Targets: claim-review, claim-health\n- Targets: `claim-review`\n\
                      - Claims: claim-review\n  - Subjects: artifact\nRequired scope: [\"claim\",\"case\",\"mechanism\"]\n\
                      Select: claim-judgment from claim claim\nSelect: applicability-decision from case case-id\n\
                      Remove cases: a, b\n";
        let expected = "### Verification Challenge: mutation/search\n- Form: mutation\n- Targets: verification-review, claim-health\n- Targets: `verification-review`\n\
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
    fn rewrites_every_delta_operation_declaration_and_check_deployment_scope() {
        let source = "## Add claim realization: a\n### Add case realization: a/s\n## Replace claim verification: a\n\
                      ### Remove case verification: a/s\n## Add claim review: a\n### Add challenger: m/s\n\
                      ### Replace challenger: m/s\n### Remove challenger: m/s\n## Add Claim realization: b\n\
                      ## Add term: subject\n\n### Check: admission\n\
                      - Deployment: {\"subject\":\"source\",\"components\":[\"api\"],\"monitor\":{\"subject\":\"x\"}}\n\
                      - Note: {\"subject\":\"source\"}\n\n## Section: other\n- Deployment: {\"subject\":\"source\"}\n";
        let expected = "## Add requirement realization: a\n### Add scenario realization: a/s\n## Replace requirement verification: a\n\
                        ### Remove scenario verification: a/s\n## Add verification review: a\n### Add verification challenge: m/s\n\
                        ### Replace verification challenge: m/s\n### Remove verification challenge: m/s\n## Add Requirement realization: b\n\
                        ## Add term: subject\n\n### Check: admission\n\
                        - Deployment: {\"verification_target\":\"source\",\"components\":[\"api\"],\"monitor\":{\"subject\":\"x\"}}\n\
                        - Note: {\"subject\":\"source\"}\n\n## Section: other\n- Deployment: {\"subject\":\"source\"}\n";
        assert_eq!(rewrite_markdown(source), expected);
        assert!(retired_syntax(expected).is_empty());
        assert_eq!(retired_syntax(source).len(), 10);
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
