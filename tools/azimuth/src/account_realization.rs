//! Requirement-first, prose-preserving realization document parser.
//!
//! This parser is deliberately independent of the accepted alpha realization model. Assembly and
//! review semantics must be chosen before its declarations become model authority.

use crate::diag::{validate_id, Diag};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Requirement,
    Scenario,
    Section,
    Element,
    Mechanism,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub id: String,
    pub line: usize,
    pub level: usize,
    pub areas: Vec<String>,
    pub element_kind: Option<String>,
    pub writes: Vec<String>,
    pub scenarios: Vec<String>,
    /// All direct prose and Markdown outside subordinate declarations, including its whitespace.
    pub body: String,
    /// Exact authored text from this heading to the next declaration at this level or above.
    pub source: String,
    pub children: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct AccountRealization {
    pub spec: String,
    pub path: String,
    /// Exact text between the document heading and first declaration.
    pub introduction: String,
    pub declarations: Vec<Declaration>,
    /// Exact source, retained so later projection/fingerprinting never reconstructs prose.
    pub source: String,
}

#[derive(Debug)]
struct Line<'a> {
    number: usize,
    start: usize,
    end: usize,
    text: &'a str,
}

#[derive(Debug)]
struct Heading {
    kind: DeclarationKind,
    id: String,
    line: usize,
    level: usize,
    start: usize,
    body_start: usize,
    end: usize,
    parent: Option<usize>,
    children: Vec<usize>,
}

pub fn load_account_realization(path: &Path) -> Result<AccountRealization, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path)
        .map_err(|error| vec![Diag::file(&display, format!("cannot read: {error}"))])?;
    parse_account_realization(&display, &source)
}

pub fn parse_account_realization(
    path: &str,
    source: &str,
) -> Result<AccountRealization, Vec<Diag>> {
    let lines = lines(source);
    let mut errors = Vec::new();
    let first = lines.first().map(|line| line.text.trim());
    let spec = first
        .and_then(|line| line.strip_prefix("# Realization: "))
        .unwrap_or("");
    if let Err(reason) = validate_id(spec, true) {
        errors.push(Diag::expecting(
            path,
            1,
            format!("invalid realization id: {reason}"),
            "# Realization: <spec-id>",
        ));
    }
    let title_end = lines.first().map_or(0, |line| line.end);
    let mut headings: Vec<Heading> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in lines.iter().skip(1) {
        let trimmed = line.text.trim_start();
        if let Some((character, width)) = fence_marker(trimmed) {
            match fence {
                None => fence = Some((character, width)),
                Some((open, count)) if open == character && width >= count => fence = None,
                _ => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let Some((level, text)) = markdown_heading(trimmed) else {
            continue;
        };
        if level == 1 {
            errors.push(Diag::expecting(
                path,
                line.number,
                "another top-level heading is not a realization declaration",
                "a single # Realization: <spec-id> heading",
            ));
            continue;
        }
        let Some((kind, id)) = declaration(level, text) else {
            if looks_reserved(text) {
                errors.push(Diag::expecting(
                    path,
                    line.number,
                    format!("malformed realization declaration `{text}`"),
                    "## Requirement realization: <requirement>, ### Scenario realization: <scenario>, or a Section, Element, or Mechanism heading",
                ));
            }
            continue;
        };
        while stack
            .last()
            .is_some_and(|index| headings[*index].level >= level)
        {
            let closed = stack.pop().unwrap();
            headings[closed].end = line.start;
        }
        let parent = stack.last().copied();
        if !allowed_parent(
            kind,
            level,
            parent.map(|index| (headings[index].kind, headings[index].level)),
        ) {
            errors.push(Diag::expecting(path, line.number, "realization declaration is at the wrong nesting level", "Requirement realization or Section at level two; Scenario realization or Mechanism directly under a Requirement at level three; Sections one level under a Requirement, Scenario realization or Section; Elements and nested Mechanisms one level under a Scenario realization or Section"));
        }
        let index = headings.len();
        headings.push(Heading {
            kind,
            id: id.to_string(),
            line: line.number,
            level,
            start: line.start,
            body_start: line.end,
            end: source.len(),
            parent,
            children: Vec::new(),
        });
        if let Some(parent) = parent {
            headings[parent].children.push(index);
        }
        stack.push(index);
    }
    while let Some(index) = stack.pop() {
        headings[index].end = source.len();
    }
    if headings.iter().all(|heading| {
        !matches!(
            heading.kind,
            DeclarationKind::Requirement | DeclarationKind::Section
        )
    }) {
        errors.push(Diag::expecting(
            path,
            0,
            "realization has no Requirement account or shared Section",
            "at least one ## Requirement realization: <requirement> or ## Section: <caption>",
        ));
    }

    let mut scenario_realizations = BTreeSet::new();
    let mut sections = BTreeSet::new();
    let mut elements = BTreeSet::new();
    let mut mechanisms = BTreeSet::new();
    let mut requirements = BTreeSet::new();
    let mut parsed: Vec<Option<Declaration>> = Vec::with_capacity(headings.len());
    for heading in &headings {
        let mut ancestor = heading.parent;
        let mut enclosing_requirement = None;
        while let Some(parent) = ancestor {
            if headings[parent].kind == DeclarationKind::Requirement {
                enclosing_requirement = Some(headings[parent].id.clone());
                break;
            }
            ancestor = headings[parent].parent;
        }
        let authored_id = &heading.id;
        if heading.kind == DeclarationKind::Scenario && enclosing_requirement.is_none() {
            errors.push(Diag::expecting(
                path,
                heading.line,
                "Scenario realization has no enclosing Requirement",
                "a Scenario realization directly beneath its Requirement realization",
            ));
        }
        let id = authored_id.clone();
        let local_id = authored_id.as_str();
        if let Err(reason) = validate_id(local_id, false) {
            errors.push(Diag::expecting(
                path,
                heading.line,
                format!("invalid declaration id `{authored_id}`: {reason}"),
                "a stable lower-kebab ID; membership does not qualify identity",
            ));
        }
        let identities = match heading.kind {
            DeclarationKind::Requirement => &mut requirements,
            DeclarationKind::Scenario => &mut scenario_realizations,
            DeclarationKind::Section => &mut sections,
            DeclarationKind::Element => &mut elements,
            DeclarationKind::Mechanism => &mut mechanisms,
        };
        if !identities.insert(id.clone()) {
            errors.push(Diag::at(
                path,
                heading.line,
                format!("duplicate realization declaration `{id}`"),
            ));
        }
        let body = direct_body(source, heading, &headings);
        let metadata_end = heading
            .children
            .first()
            .map_or(heading.end, |child| headings[*child].start);
        let (areas, element_kind, writes, scenarios) = metadata(
            path,
            heading,
            &source[heading.body_start..metadata_end],
            enclosing_requirement.as_deref(),
            &mut errors,
        );
        parsed.push(Some(Declaration {
            kind: heading.kind,
            id,
            line: heading.line,
            level: heading.level,
            areas,
            element_kind,
            writes,
            scenarios,
            body,
            source: source[heading.start..heading.end].to_string(),
            children: Vec::new(),
        }));
    }
    for item in parsed.iter().flatten() {
        if item.kind == DeclarationKind::Element {
            for written in &item.writes {
                if !elements.contains(written) {
                    errors.push(Diag::at(
                        path,
                        item.line,
                        format!(
                            "element `{}` writes undeclared element `{written}`",
                            item.id
                        ),
                    ));
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    // Descendants are assembled only after validating all identities and relationships.
    let mut roots = Vec::new();
    for index in (0..headings.len()).rev() {
        let item = parsed[index].take().unwrap();
        if let Some(parent) = headings[index].parent {
            parsed[parent].as_mut().unwrap().children.push(item);
        } else {
            roots.push(item);
        }
    }
    roots.reverse();
    for item in &mut roots {
        reverse_children(item);
    }
    let introduction_end = headings
        .first()
        .map_or(source.len(), |heading| heading.start);
    Ok(AccountRealization {
        spec: spec.to_string(),
        path: path.to_string(),
        introduction: source[title_end..introduction_end].to_string(),
        declarations: roots,
        source: source.to_string(),
    })
}

fn reverse_children(node: &mut Declaration) {
    node.children.reverse();
    for child in &mut node.children {
        reverse_children(child);
    }
}

fn lines(source: &str) -> Vec<Line<'_>> {
    let mut result = Vec::new();
    let mut start = 0;
    for (index, part) in source.split_inclusive('\n').enumerate() {
        let end = start + part.len();
        result.push(Line {
            number: index + 1,
            start,
            end,
            text: part.trim_end_matches(['\r', '\n']),
        });
        start = end;
    }
    result
}

fn fence_marker(text: &str) -> Option<(char, usize)> {
    let character = text.chars().next()?;
    if character != '`' && character != '~' {
        return None;
    }
    let width = text.chars().take_while(|value| *value == character).count();
    (width >= 3).then_some((character, width))
}

fn markdown_heading(text: &str) -> Option<(usize, &str)> {
    let level = text.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&level) || text.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    Some((level, text[level + 1..].trim()))
}

fn declaration(level: usize, text: &str) -> Option<(DeclarationKind, &str)> {
    let kinds = [
        ("Requirement realization: ", DeclarationKind::Requirement),
        ("Scenario realization: ", DeclarationKind::Scenario),
        ("Section: ", DeclarationKind::Section),
        ("Element: ", DeclarationKind::Element),
        ("Mechanism: ", DeclarationKind::Mechanism),
    ];
    if !(2..=6).contains(&level) {
        return None;
    }
    kinds
        .into_iter()
        .find_map(|(prefix, kind)| text.strip_prefix(prefix).map(|id| (kind, id.trim())))
}

fn looks_reserved(text: &str) -> bool {
    [
        "Requirement realization",
        "Scenario realization",
        "Section",
        "Element",
        "Mechanism",
    ]
    .iter()
    .any(|prefix| text.starts_with(prefix))
}

fn allowed_parent(
    kind: DeclarationKind,
    level: usize,
    parent: Option<(DeclarationKind, usize)>,
) -> bool {
    match (kind, level, parent) {
        (DeclarationKind::Requirement | DeclarationKind::Section, 2, None) => true,
        (
            DeclarationKind::Scenario | DeclarationKind::Mechanism,
            3,
            Some((DeclarationKind::Requirement, 2)),
        ) => true,
        (
            DeclarationKind::Section,
            3..=6,
            Some((
                DeclarationKind::Requirement | DeclarationKind::Scenario | DeclarationKind::Section,
                parent_level,
            )),
        ) if level == parent_level + 1 => true,
        (
            DeclarationKind::Element | DeclarationKind::Mechanism,
            3..=6,
            Some((DeclarationKind::Scenario | DeclarationKind::Section, parent_level)),
        ) if level == parent_level + 1 => true,
        _ => false,
    }
}

fn direct_body(source: &str, heading: &Heading, headings: &[Heading]) -> String {
    let mut result = String::new();
    let mut cursor = heading.body_start;
    for child in &heading.children {
        let child = &headings[*child];
        result.push_str(&source[cursor..child.start]);
        cursor = child.end;
    }
    result.push_str(&source[cursor..heading.end]);
    result
}

fn metadata(
    path: &str,
    heading: &Heading,
    body: &str,
    _enclosing_requirement: Option<&str>,
    errors: &mut Vec<Diag>,
) -> (Vec<String>, Option<String>, Vec<String>, Vec<String>) {
    let mut fields: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let body_lines: Vec<&str> = body.lines().collect();
    let mut index = 0;
    while index < body_lines.len() && body_lines[index].trim().is_empty() {
        index += 1;
    }
    while index < body_lines.len() {
        let text = body_lines[index];
        if text.trim().is_empty() {
            break;
        }
        if !text.starts_with("- ") {
            break;
        }
        let Some((name, scalar)) = text[2..].split_once(':') else {
            break;
        };
        let name = name.trim();
        let accepted = match heading.kind {
            DeclarationKind::Requirement => &["Areas"][..],
            DeclarationKind::Element => &["Kind", "Areas", "Writes"][..],
            DeclarationKind::Mechanism => &["Scenarios"][..],
            _ => &[][..],
        };
        let line = heading.line + 1 + index;
        if !accepted.contains(&name) {
            errors.push(Diag::expecting(
                path,
                line,
                format!("unknown `{name}` field"),
                format!("{} fields", accepted.join(", ")),
            ));
        }
        if fields.contains_key(name) {
            errors.push(Diag::at(path, line, format!("duplicate `{name}` field")));
        }
        index += 1;
        let mut values = Vec::new();
        if !scalar.trim().is_empty() {
            if let Some(value) = scalar_value(scalar.trim()) {
                values.push(value.to_string());
            } else {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("invalid `{name}` value"),
                    "one backtick-delimited value",
                ));
            }
        } else {
            while index < body_lines.len() && body_lines[index].starts_with("  - ") {
                let value_line = heading.line + 1 + index;
                if let Some(value) = scalar_value(body_lines[index][4..].trim()) {
                    values.push(value.to_string());
                } else {
                    errors.push(Diag::expecting(
                        path,
                        value_line,
                        "invalid list value",
                        "one backtick-delimited value",
                    ));
                }
                index += 1;
            }
        }
        if accepted.contains(&name) {
            fields.insert(name, values);
        }
    }
    let declared_scenarios = fields.contains_key("Scenarios");
    let authored_scenarios = fields.remove("Scenarios").unwrap_or_default();
    let areas = fields.remove("Areas").unwrap_or_default();
    let writes = fields.remove("Writes").unwrap_or_default();
    let kinds = fields.remove("Kind").unwrap_or_default();
    if heading.kind == DeclarationKind::Requirement && areas.is_empty() {
        errors.push(Diag::expecting(
            path,
            heading.line,
            "Requirement realization has no Areas",
            "- Areas: followed by at least one area",
        ));
    }
    if heading.kind == DeclarationKind::Element && kinds.len() != 1 {
        errors.push(Diag::expecting(
            path,
            heading.line,
            "Element needs one Kind",
            "- Kind: `kind`",
        ));
    }
    for (name, values) in [("Areas", &areas), ("Writes", &writes)] {
        let mut seen = BTreeSet::new();
        for value in values {
            if let Err(reason) = validate_id(value, false) {
                errors.push(Diag::at(
                    path,
                    heading.line,
                    format!("invalid {name} value `{value}`: {reason}"),
                ));
            }
            if !seen.insert(value) {
                errors.push(Diag::at(
                    path,
                    heading.line,
                    format!("duplicate {name} value `{value}`"),
                ));
            }
        }
    }
    if let Some(kind) = kinds.first() {
        if let Err(reason) = validate_id(kind, false) {
            errors.push(Diag::at(
                path,
                heading.line,
                format!("invalid Element Kind `{kind}`: {reason}"),
            ));
        }
    }
    if declared_scenarios && authored_scenarios.is_empty() {
        errors.push(Diag::expecting(
            path,
            heading.line,
            "Mechanism Scenarios is empty",
            "- Scenarios: followed by at least one stable Scenario ID",
        ));
    }
    let mut scenarios = Vec::new();
    let mut seen_scenarios = BTreeSet::new();
    for scenario in &authored_scenarios {
        let resolved = scenario.clone();
        if let Err(reason) = validate_id(&resolved, false) {
            errors.push(Diag::expecting(
                path,
                heading.line,
                format!("invalid Mechanism Scenario `{scenario}`: {reason}"),
                "a stable Scenario ID independent of module or parent",
            ));
        }
        if !seen_scenarios.insert(resolved.clone()) {
            errors.push(Diag::at(
                path,
                heading.line,
                format!("duplicate Mechanism Scenario `{resolved}`"),
            ));
        }
        scenarios.push(resolved);
    }
    if matches!(
        heading.kind,
        DeclarationKind::Mechanism | DeclarationKind::Scenario
    ) && body_lines[index..]
        .iter()
        .all(|line| line.trim().is_empty())
    {
        errors.push(Diag::expecting(
            path,
            heading.line,
            format!("{:?} realization has no explanation", heading.kind),
            "prose explaining the Scenario implementation or selected control and failure boundary",
        ));
    }
    (areas, kinds.into_iter().next(), writes, scenarios)
}

fn scalar_value(value: &str) -> Option<&str> {
    value
        .strip_prefix('`')?
        .strip_suffix('`')
        .filter(|value| !value.is_empty() && !value.contains('`'))
}
