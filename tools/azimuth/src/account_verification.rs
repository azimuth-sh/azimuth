//! Claim-first verification explanation. Source-authored Checks and their evidence relationships
//! remain separate; this parser reads the repository account's scope and method vocabulary.

use crate::diag::{resolve_check_id, validate_id, Diag};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountVerification {
    pub owner: String,
    pub path: String,
    pub introduction: String,
    pub claims: Vec<ClaimVerification>,
    pub sections: Vec<VerificationSection>,
    pub cases: Vec<CaseVerification>,
    pub elements: Vec<VerificationElement>,
    pub checks: Vec<AuthoredCheck>,
    pub package_entities: Vec<crate::verification_packages::Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimVerification {
    pub claim: String,
    pub required_elements: Vec<String>,
    pub prose: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseVerification {
    pub case: String,
    pub claim: String,
    pub prose: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredCheck {
    pub definition: crate::verification::Check,
    pub claim: String,
    pub cases: Vec<String>,
    pub mechanisms: Vec<String>,
    pub proposition: Option<String>,
    pub bindings: Vec<crate::verification_packages::Binding>,
    pub inputs: std::collections::BTreeMap<String, String>,
    pub selectors: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationSection {
    pub id: String,
    /// A section outside a Claim is shared. A nested section belongs to one Claim.
    pub claim: Option<String>,
    pub prose: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationElement {
    pub id: String,
    pub kind: String,
    /// A shared element has no Claim owner; Claim-specific elements inherit the enclosing Claim.
    pub claim: Option<String>,
    pub section: Option<String>,
    pub case: Option<String>,
    pub prose: String,
    pub line: usize,
}

#[derive(Debug)]
struct Heading<'a> {
    level: usize,
    label: &'a str,
    id: &'a str,
    line: usize,
}

pub fn load_account_verification(path: &Path) -> Result<AccountVerification, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path)
        .map_err(|error| vec![Diag::file(&display, format!("cannot read: {error}"))])?;
    parse_account_verification(&display, &source)
}

pub fn parse_account_verification(
    path: &str,
    source: &str,
) -> Result<AccountVerification, Vec<Diag>> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut errors = Vec::new();
    let headings = collect_headings(path, &lines, &mut errors);
    let mut owner = None;
    let mut introduction = String::new();
    let mut claims = Vec::new();
    let mut sections = Vec::new();
    let mut cases = Vec::new();
    let mut elements = Vec::new();
    let mut checks = Vec::new();
    let mut package_entities = Vec::new();
    let mut ancestors: Vec<usize> = Vec::new();

    for (index, heading) in headings.iter().enumerate() {
        while ancestors
            .last()
            .is_some_and(|index| headings[*index].level >= heading.level)
        {
            ancestors.pop();
        }
        let structural_parent = ancestors.iter().rev().find_map(|index| {
            let parent = &headings[*index];
            (!parent.label.is_empty()).then_some(parent.label)
        });
        let active_claim = ancestors.iter().rev().find_map(|index| {
            let parent = &headings[*index];
            (parent.label == "Claim verification").then(|| parent.id.to_string())
        });
        let active_case = ancestors.iter().rev().find_map(|index| {
            let parent = &headings[*index];
            (parent.label == "Case verification").then(|| parent.id.to_string())
        });
        let active_section = ancestors.iter().rev().find_map(|index| {
            let parent = &headings[*index];
            (parent.label == "Section").then(|| parent.id.to_string())
        });
        let parent_level = ancestors.last().map_or(1, |index| headings[*index].level);
        ancestors.push(index);
        let start = heading.line;
        let end = headings
            .get(index + 1)
            .map_or(lines.len(), |next| next.line - 1);
        let block = &lines[start..end];
        if heading.level == 1 {
            if heading.label != "Verification" {
                errors.push(Diag::expecting(
                    path,
                    heading.line,
                    "unrecognized document heading",
                    "`# Verification: <module-id>`",
                ));
            } else if owner.is_some() {
                errors.push(Diag::at(
                    path,
                    heading.line,
                    "verification authority is declared twice",
                ));
            } else if let Err(reason) = validate_id(heading.id, true) {
                errors.push(Diag::at(
                    path,
                    heading.line,
                    format!("invalid module id: {reason}"),
                ));
            } else {
                owner = Some(heading.id.to_string());
                introduction = prose(block);
            }

            continue;
        }
        if owner.is_none() {
            errors.push(Diag::expecting(
                path,
                heading.line,
                "declaration precedes verification authority",
                "`# Verification: <module-id>` first",
            ));
            continue;
        }
        if heading.label.is_empty() {
            let caption = lines[heading.line - 1..end].join("\n");
            let target = match structural_parent {
                Some("Surface" | "Expectation set" | "Probe") => package_entities.last_mut().map(
                    |entity: &mut crate::verification_packages::Declaration| &mut entity.prose,
                ),
                Some("Check") => checks
                    .last_mut()
                    .map(|check: &mut AuthoredCheck| &mut check.definition.rationale),
                Some("Element") => elements
                    .last_mut()
                    .map(|element: &mut VerificationElement| &mut element.prose),
                Some("Case verification") => cases
                    .last_mut()
                    .map(|case: &mut CaseVerification| &mut case.prose),
                Some("Section") => sections
                    .last_mut()
                    .map(|section: &mut VerificationSection| &mut section.prose),
                Some("Claim verification") => claims
                    .last_mut()
                    .map(|claim: &mut ClaimVerification| &mut claim.prose),
                _ => Some(&mut introduction),
            };
            if let Some(target) = target {
                if !target.is_empty() {
                    target.push_str("\n\n");
                }
                target.push_str(&caption);
            }
            continue;
        }
        match (heading.level, heading.label) {
            (3, "Surface" | "Expectation set" | "Probe") => {
                if active_claim.is_some() {
                    errors.push(Diag::at(path, heading.line, "package declarations must be shared outside a Claim"));
                }
                let declaration = crate::verification_packages::read_declaration(path, heading.line, heading.label, heading.id, block, &mut errors);
                if package_entities.iter().any(|item: &crate::verification_packages::Declaration| item.kind == declaration.kind && item.id == declaration.id) {
                    errors.push(Diag::at(path, heading.line, "duplicate package declaration"));
                }
                package_entities.push(declaration);
            }
            (2, "Claim verification") => {
                if let Err(reason) = validate_id(heading.id, false) {
                    errors.push(Diag::expecting(
                        path,
                        heading.line,
                        format!("invalid local Claim id: {reason}"),
                        "## Claim verification: <claim-id> within the declared module",
                    ));
                }
                let identity = heading.id.to_string();
                let (fields, body) = read_fields(
                    path,
                    heading.line,
                    block,
                    &["Required elements"],
                    &mut errors,
                );
                let scope = ClaimVerification {
                    claim: identity.clone(),
                    required_elements: fields[0].clone(),
                    prose: body,
                    line: heading.line,
                };
                if claims
                    .iter()
                    .any(|existing: &ClaimVerification| existing.claim == scope.claim)
                {
                    errors.push(Diag::at(
                        path,
                        heading.line,
                        format!("Claim verification `{}` is declared twice", scope.claim),
                    ));
                }
                claims.push(scope);




            }
            (3, "Case verification") => {
                validate_local(path, heading.line, "Case", heading.id, &mut errors);
                let Some(identity) = active_claim.clone() else {
                    errors.push(Diag::expecting(path, heading.line,
                        "Case verification has no enclosing Claim",
                        "a `## Claim verification:` heading"));
                    continue;
                };
                let case = heading.id.to_string();
                if cases.iter().any(|item: &CaseVerification| item.case == case) {
                    errors.push(Diag::at(path, heading.line,
                        format!("Case verification `{case}` is declared twice")));
                }
                cases.push(CaseVerification {
                    case: case.clone(), claim: identity, prose: prose(block), line: heading.line,
                });



            }
            (2, "Section") | (3, "Section") => {
                if heading.level == 3 && active_claim.is_none() {
                    errors.push(Diag::expecting(
                        path,
                        heading.line,
                        "nested Section has no Claim",
                        "a `## Claim verification:` heading",
                    ));
                }
                if heading.level == 2 {

                } else if parent_level != 2 && parent_level != 3 && parent_level != 4 {
                    errors.push(Diag::expecting(
                        path,
                        heading.line,
                        "Section has no enclosing Claim",
                        "a `## Claim verification:` heading",
                    ));
                }
                validate_local(path, heading.line, "Section", heading.id, &mut errors);
                if sections
                    .iter()
                    .any(|existing: &VerificationSection| existing.id == heading.id)
                {
                    errors.push(Diag::at(
                        path,
                        heading.line,
                        format!("Section `{}` is declared twice", heading.id),
                    ));
                }
                sections.push(VerificationSection {
                    id: heading.id.to_string(),
                    claim: active_claim.clone(),
                    prose: prose(block),
                    line: heading.line,
                });



            }
            (3..=6, "Check") => {
                let valid_parent = active_claim.is_some()
                    && matches!(structural_parent, Some("Claim verification" | "Case verification" | "Section"))
                    && heading.level == parent_level + 1;
                if !valid_parent {
                    errors.push(Diag::expecting(path, heading.line, "Check has invalid nesting",
                        "a Check one heading level below a Claim, its Case verification, Section or ordinary caption, never inside a Check or Element"));
                    continue;
                }
                let check_id = match resolve_check_id(heading.id) {
                    Ok(id) => id,
                    Err(reason) => {
                        errors.push(Diag::at(path, heading.line, format!("invalid Check id: {reason}")));
                        continue;
                    }
                };
                let identity = active_claim.as_ref().unwrap();
                let parsed = read_check(path, heading.line, &check_id, identity, active_case.as_deref(), block, &mut errors);
                if checks.iter().any(|item: &AuthoredCheck| item.definition.id == check_id) {
                    errors.push(Diag::at(path, heading.line, format!("Check `{check_id}` is declared twice")));
                }

                checks.push(parsed);

            }
            (3, "Element") | (4, "Element") => {
                let valid_parent = (heading.level == 3 && active_case.is_none()
                    && ((active_claim.is_none() && active_section.is_some() && matches!(parent_level, 2 | 3))
                        || (active_claim.is_some() && active_section.is_none() && matches!(parent_level, 2 | 3))))
                    || (heading.level == 4 && active_claim.is_some()
                        && (active_section.is_some() || active_case.is_some())
                        && matches!(parent_level, 3 | 4));
                if !valid_parent {
                    errors.push(Diag::expecting(
                        path,
                        heading.line,
                        "Element has invalid nesting",
                        "`### Element:` under a Claim or shared Section, or `#### Element:` under a Claim Section or Case verification",
                    ));
                }
                validate_local(path, heading.line, "Element", heading.id, &mut errors);
                let (fields, body) = read_fields(path, heading.line, block, &["Kind"], &mut errors);
                let kind = fields[0].first().cloned().unwrap_or_default();
                validate_local(path, heading.line, "Element kind", &kind, &mut errors);
                if elements
                    .iter()
                    .any(|existing: &VerificationElement| existing.id == heading.id)
                {
                    errors.push(Diag::at(
                        path,
                        heading.line,
                        format!("Element `{}` is declared twice", heading.id),
                    ));
                }
                elements.push(VerificationElement {
                    id: heading.id.to_string(),
                    kind,
                    claim: active_claim.clone(),
                    section: active_section.clone(),
                    case: active_case.clone(),
                    prose: body,
                    line: heading.line,
                });

            }
            _ => errors.push(Diag::expecting(
                path,
                heading.line,
                format!(
                    "unrecognized heading `{}{}`",
                    "#".repeat(heading.level),
                    format!(" {}: {}", heading.label, heading.id)
                ),
                "a Claim verification, Case verification, Section, Check or Element at its permitted nesting level",
            )),
        }
    }
    let Some(owner) = owner else {
        errors.push(Diag::expecting(
            path,
            1,
            "verification authority is missing",
            "`# Verification: <module-id>`",
        ));
        return Err(errors);
    };
    if claims.is_empty() {
        errors.push(Diag::expecting(
            path,
            1,
            "verification has no Claim scope",
            "at least one `## Claim verification:` declaration",
        ));
    }
    for claim in &claims {
        for id in &claim.required_elements {
            if let Some(element) = elements.iter().find(|element| &element.id == id) {
                if element.claim.is_some() {
                    errors.push(Diag::at(
                        path,
                        claim.line,
                        format!("explicit Required elements must reference a shared Element; `{id}` belongs to a Claim"),
                    ));
                }
            } else {
                errors.push(Diag::at(
                    path,
                    claim.line,
                    format!("Claim `{}` requires undeclared Element `{id}`", claim.claim),
                ));
            }
        }
    }
    for claim in &mut claims {
        claim.required_elements.extend(
            elements
                .iter()
                .filter(|element| element.claim.as_deref() == Some(claim.claim.as_str()))
                .map(|element| element.id.clone()),
        );
        claim.required_elements.sort();
    }
    if errors.is_empty() {
        Ok(AccountVerification {
            owner,
            path: path.to_string(),
            introduction,
            claims,
            sections,
            cases,
            elements,
            checks,
            package_entities,
        })
    } else {
        Err(errors)
    }
}

fn collect_headings<'a>(path: &str, lines: &'a [&str], errors: &mut Vec<Diag>) -> Vec<Heading<'a>> {
    let mut headings = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let marker = trimmed.chars().next().unwrap_or(' ');
        if marker == '`' || marker == '~' {
            let width = trimmed.chars().take_while(|value| *value == marker).count();
            if width >= 3 {
                match fence {
                    None => fence = Some((marker, width)),
                    Some((open, minimum)) if open == marker && width >= minimum => fence = None,
                    _ => {}
                }
                continue;
            }
        }
        if fence.is_some() || !line.starts_with('#') {
            continue;
        }
        let level = line.chars().take_while(|value| *value == '#').count();
        if !(1..=6).contains(&level)
            || !line
                .as_bytes()
                .get(level)
                .is_some_and(|value| *value == b' ')
        {
            errors.push(Diag::expecting(
                path,
                index + 1,
                "invalid Markdown heading",
                "one to six `#` followed by a space and heading text",
            ));
            continue;
        }
        let rest = &line[level + 1..];
        let declarations = [
            "Verification",
            "Claim verification",
            "Case verification",
            "Section",
            "Check",
            "Element",
            "Surface",
            "Expectation set",
            "Probe",
        ];
        let (label, id) = match rest.split_once(": ") {
            Some((label, id)) if declarations.contains(&label) => (label, id),
            _ => {
                if declarations
                    .iter()
                    .any(|label| rest == *label || rest.starts_with(&format!("{label}:")))
                {
                    errors.push(Diag::expecting(
                        path,
                        index + 1,
                        format!("malformed declaration `{line}`"),
                        "a declaration label followed by colon, space and identity",
                    ));
                    continue;
                }
                ("", rest)
            }
        };
        headings.push(Heading {
            level,
            label,
            id,
            line: index + 1,
        });
    }
    headings
}

fn read_fields(
    path: &str,
    line: usize,
    block: &[&str],
    names: &[&str],
    errors: &mut Vec<Diag>,
) -> (Vec<Vec<String>>, String) {
    let mut values = vec![Vec::new(); names.len()];
    let mut seen = vec![false; names.len()];
    let mut cursor = 0;
    while cursor < block.len() && block[cursor].trim().is_empty() {
        cursor += 1;
    }
    while cursor < block.len() && block[cursor].starts_with("- ") {
        let field_line = line + 1 + cursor;
        let Some((label, suffix)) = block[cursor][2..].split_once(':') else {
            errors.push(Diag::expecting(
                path,
                field_line,
                "invalid metadata",
                "`- <field>:`",
            ));
            cursor += 1;
            continue;
        };
        let Some(position) = names.iter().position(|name| *name == label) else {
            errors.push(Diag::expecting(
                path,
                field_line,
                format!("unknown metadata `{label}`"),
                format!("one of: {}", names.join(", ")),
            ));
            cursor += 1;
            continue;
        };
        if seen[position] {
            errors.push(Diag::at(
                path,
                field_line,
                format!("`{label}` is declared twice"),
            ));
        }
        seen[position] = true;
        if label == "Kind" {
            let Some(value) = suffix
                .trim()
                .strip_prefix('`')
                .and_then(|value| value.strip_suffix('`'))
            else {
                errors.push(Diag::expecting(
                    path,
                    field_line,
                    "invalid Kind",
                    "`- Kind: `<kind>`",
                ));
                cursor += 1;
                continue;
            };
            values[position].push(value.to_string());
            cursor += 1;
        } else {
            if !suffix.trim().is_empty() {
                errors.push(Diag::expecting(
                    path,
                    field_line,
                    format!("`{label}` has an inline value"),
                    "an indented list of backtick-delimited ids",
                ));
            }
            cursor += 1;
            let mut unique = BTreeSet::new();
            while cursor < block.len() && block[cursor].starts_with("  - ") {
                let item_line = line + 1 + cursor;
                let text = &block[cursor][4..];
                if let Some(value) = text
                    .strip_prefix('`')
                    .and_then(|value| value.strip_suffix('`'))
                {
                    if !unique.insert(value) {
                        errors.push(Diag::at(
                            path,
                            item_line,
                            format!("`{label}` repeats `{value}`"),
                        ));
                    }
                    if let Err(reason) = validate_id(value, false) {
                        errors.push(Diag::at(
                            path,
                            item_line,
                            format!("invalid `{label}` reference: {reason}"),
                        ));
                    }
                    values[position].push(value.to_string());
                } else {
                    errors.push(Diag::expecting(
                        path,
                        item_line,
                        "invalid list item",
                        "`  - `<local-id>`",
                    ));
                }
                cursor += 1;
            }
        }
    }
    for (position, name) in names.iter().enumerate() {
        if (*name == "Kind" && !seen[position]) || (seen[position] && values[position].is_empty()) {
            errors.push(Diag::expecting(
                path,
                line,
                format!("missing or empty `{name}` metadata"),
                format!("`- {name}:` with at least one value"),
            ));
        }
    }
    (values, prose(&block[cursor..]))
}

fn prose(lines: &[&str]) -> String {
    lines.join("\n").trim().to_string()
}

fn validate_local(path: &str, line: usize, kind: &str, id: &str, errors: &mut Vec<Diag>) {
    if let Err(reason) = validate_id(id, false) {
        errors.push(Diag::at(path, line, format!("invalid {kind} id: {reason}")));
    }
}

fn read_check(
    path: &str,
    line: usize,
    id: &str,
    claim: &str,
    enclosing_case: Option<&str>,
    block: &[&str],
    errors: &mut Vec<Diag>,
) -> AuthoredCheck {
    if block
        .iter()
        .any(|line| line.starts_with("- Evidence bindings:"))
    {
        return crate::verification_packages::read_check(
            path,
            line,
            id,
            claim,
            enclosing_case,
            block,
            errors,
        );
    }
    let mut methods = Vec::new();
    let mut terminal = None;
    let mut proposition = None;
    let mut cases = None;
    let mut mechanisms = Vec::new();
    let mut seen = BTreeSet::new();
    let mut cursor = 0;
    while cursor < block.len() && block[cursor].trim().is_empty() {
        cursor += 1;
    }
    while cursor < block.len() && block[cursor].starts_with("- ") {
        let number = line + cursor + 1;
        let Some((label, suffix)) = block[cursor][2..].split_once(':') else {
            errors.push(Diag::expecting(
                path,
                number,
                "invalid Check metadata",
                "- Method:, Terminal:, Cases:, Mechanisms: or Proposition:",
            ));
            cursor += 1;
            continue;
        };
        if label != "Method" && !seen.insert(label) {
            errors.push(Diag::at(
                path,
                number,
                format!("Check `{id}` repeats `{label}`"),
            ));
        }
        match label {
            "Method" | "Terminal" | "Proposition" => {
                let text = suffix.trim();
                if text.is_empty() {
                    errors.push(Diag::expecting(
                        path,
                        number,
                        format!("Check `{id}` has empty `{label}`"),
                        "nonempty free-form text",
                    ));
                }
                match label {
                    "Method" => methods.push(text.to_string()),
                    "Terminal" => terminal = Some(text.to_string()),
                    _ => proposition = Some(text.to_string()),
                }
                cursor += 1;
            }
            "Cases" | "Mechanisms" => {
                if !suffix.trim().is_empty() {
                    errors.push(Diag::expecting(
                        path,
                        number,
                        format!("{label} has an inline value"),
                        "an indented list of backtick-delimited ids",
                    ));
                }
                cursor += 1;
                let mut items = Vec::new();
                let mut unique = BTreeSet::new();
                while cursor < block.len() && block[cursor].starts_with("  - ") {
                    let at = line + cursor + 1;
                    let raw = block[cursor][4..]
                        .strip_prefix('`')
                        .and_then(|v| v.strip_suffix('`'));
                    if let Some(raw) = raw {
                        let value = raw.to_string();
                        if let Err(reason) = validate_id(raw, false) {
                            errors.push(Diag::expecting(
                                path,
                                at,
                                format!("invalid Check {label} reference `{raw}`: {reason}"),
                                "a stable entity ID; kind is determined by its reference field",
                            ));
                        }
                        if !unique.insert(value.clone()) {
                            errors.push(Diag::at(
                                path,
                                at,
                                format!("Check `{id}` repeats {label} identity `{value}`"),
                            ));
                        }
                        items.push(value);
                    } else {
                        errors.push(Diag::expecting(
                            path,
                            at,
                            "invalid Check reference",
                            "an indented backtick-delimited id",
                        ));
                    }
                    cursor += 1;
                }
                if items.is_empty() {
                    errors.push(Diag::at(
                        path,
                        number,
                        format!("Check `{id}` has empty `{label}`"),
                    ));
                }
                if label == "Cases" {
                    cases = Some(items);
                } else {
                    mechanisms = items;
                }
            }
            _ => {
                errors.push(Diag::expecting(
                    path,
                    number,
                    format!("unknown Check metadata `{label}`"),
                    "Method, Terminal, Cases, Mechanisms or Proposition",
                ));
                cursor += 1;
            }
        }
    }
    if methods.is_empty() {
        errors.push(Diag::expecting(
            path,
            line,
            format!("Check `{id}` has no Method"),
            "at least one - Method: statement",
        ));
    }
    if terminal.is_none() {
        errors.push(Diag::expecting(
            path,
            line,
            format!("Check `{id}` has no Terminal"),
            "exactly one - Terminal: proposition",
        ));
    }
    let cases = cases.unwrap_or_else(|| {
        enclosing_case
            .map(|v| vec![v.to_string()])
            .unwrap_or_default()
    });
    if cases.is_empty() {
        errors.push(Diag::expecting(
            path,
            line,
            format!("Check `{id}` has no Case relationship"),
            "Case nesting or an explicit nonempty Cases list",
        ));
    }
    if enclosing_case.is_some_and(|owner| !cases.iter().any(|case| case == owner)) {
        errors.push(Diag::at(
            path,
            line,
            format!("Check `{id}` Cases omit its enclosing Case"),
        ));
    }
    AuthoredCheck {
        definition: crate::verification::Check {
            id: id.to_string(),
            methods,
            terminal: terminal.unwrap_or_default(),
            rationale: prose(&block[cursor..]),
            path: path.to_string(),
            line,
        },
        claim: claim.to_string(),
        cases,
        mechanisms,
        proposition,
        bindings: Vec::new(),
        inputs: std::collections::BTreeMap::new(),
        selectors: std::collections::BTreeMap::new(),
    }
}
