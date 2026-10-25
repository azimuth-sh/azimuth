//! The spec parser.
//!
//! Replaces the alpha's OpenSpec reader. The grammar is documented beside the model
//! packages, and it is deliberately rigid so that a strict line-oriented parser is straightforward
//! without a parser crate; see `Cargo.toml` for the empty dependency set.
//!
//! Two failure modes are kept apart on purpose:
//!
//! - an unrecognized **construct** fails the parse (fail loudly);
//! - a missing **declaration** is valid (a requirement without `Verification:` parses and owes
//!   no verification).
//!
//! Conflating them would either let syntax errors through as findings, or turn a semantic gap
//! into something a reviewer never sees in the matrix.

use crate::diag::{validate_id, Diag};
use crate::model::{Condition, Domain, Requirement, Rigor, Scenario, Spec, Term};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Loaded {
    pub specs: Vec<Spec>,
    /// Non-fatal. Folder layout diverging from a declared id is a warning, never an error: ids are
    /// declared and path-independent, so the tree is a navigation aid rather than the authority.
    pub warnings: Vec<Diag>,
}

pub fn load_specs(root: &Path) -> Result<Loaded, Vec<Diag>> {
    let mut files = Vec::new();
    collect_markdown(root, &mut files).map_err(|e| {
        vec![Diag::file(
            &root.display().to_string(),
            format!("cannot read specs: {e}"),
        )]
    })?;
    files.sort();

    let mut specs: Vec<Spec> = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for path in files {
        let display = path.display().to_string();
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(Diag::file(&display, format!("cannot read: {e}")));
                continue;
            }
        };
        match parse_spec(&display, &source) {
            Ok(spec) => {
                if let Some(prev) = specs.iter().find(|s| s.id == spec.id) {
                    errors.push(Diag::at(
                        &display,
                        1,
                        format!("spec id `{}` is already declared by {}", spec.id, prev.path),
                    ));
                    continue;
                }
                if let Some(expected) = expected_id_from_path(root, &path) {
                    if expected != spec.id {
                        warnings.push(Diag::at(
                            &display,
                            1,
                            format!(
                                "declared id `{}` does not match its location (`{}`); \
                                 ids are path-independent, so this is a navigation hint only",
                                spec.id, expected
                            ),
                        ));
                    }
                }
                specs.push(spec);
            }
            Err(mut diags) => errors.append(&mut diags),
        }
    }

    let mut requirement_ids = std::collections::BTreeMap::new();
    let mut scenario_ids = std::collections::BTreeMap::new();
    for spec in &specs {
        for requirement in &spec.requirements {
            if let Some(previous) =
                requirement_ids.insert(requirement.id.clone(), spec.path.clone())
            {
                errors.push(Diag::at(&spec.path, requirement.line, format!("Requirement ID `{}` is already declared by {previous}; IDs are project-wide within their kind", requirement.id)));
            }
            for scenario in &requirement.scenarios {
                if let Some(previous) = scenario_ids.insert(scenario.id.clone(), spec.path.clone())
                {
                    errors.push(Diag::at(&spec.path, scenario.line, format!("Scenario ID `{}` is already declared by {previous}; parent Requirement does not disambiguate identity", scenario.id)));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(Loaded { specs, warnings })
    } else {
        Err(errors)
    }
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("spec.md") {
            out.push(path);
        }
    }
    Ok(())
}

fn expected_id_from_path(root: &Path, path: &Path) -> Option<String> {
    let rel = path.parent()?.strip_prefix(root).ok()?;
    let id = rel.to_str()?.replace('\\', "/");
    (!id.is_empty()).then_some(id)
}

pub fn parse_spec(path: &str, source: &str) -> Result<Spec, Vec<Diag>> {
    let mut p = SpecParser {
        path,
        errors: Vec::new(),
        id: None,
        requirements: Vec::new(),
        terms: Vec::new(),
        fenced: false,
    };
    p.run(source);
    p.finish()
}

struct SpecParser<'a> {
    path: &'a str,
    errors: Vec<Diag>,
    id: Option<String>,
    requirements: Vec<Requirement>,
    terms: Vec<Term>,
    fenced: bool,
}

impl<'a> SpecParser<'a> {
    fn run(&mut self, source: &str) {
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let raw = lines[i];
            let line_no = i + 1;
            let trimmed = raw.trim();

            // Fenced blocks are non-normative and never parsed. A diagram either illustrates and
            // claims nothing, or it is the source of requirements and nothing restates it.
            if trimmed.starts_with("```") {
                self.fenced = !self.fenced;
                i += 1;
                continue;
            }
            if self.fenced || trimmed.is_empty() || trimmed.starts_with('>') {
                i += 1;
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("# ") {
                self.spec_heading(rest, line_no);
                i += 1;
            } else if let Some(rest) = trimmed.strip_prefix("## Term:") {
                i = self.term(rest, line_no, &lines, i);
            } else if let Some(rest) = trimmed.strip_prefix("## Invariant:") {
                i = self.invariant(rest, line_no, &lines, i);
            } else if let Some(rest) = trimmed.strip_prefix("## ") {
                i = self.requirement(rest, line_no, &lines, i);
            } else if trimmed.starts_with("### ") {
                self.errors.push(Diag::expecting(
                    self.path,
                    line_no,
                    "scenario outside a requirement",
                    "`### Scenario:` to follow a `## Requirement:` heading",
                ));
                i += 1;
            } else if trimmed.starts_with('#') {
                self.errors.push(Diag::expecting(
                    self.path,
                    line_no,
                    format!("unrecognized heading `{trimmed}`"),
                    "`# Spec:`, `## Term:`, `## Requirement:` or `### Scenario:`",
                ));
                i += 1;
            } else {
                // Prose. Non-normative outside a requirement statement.
                i += 1;
            }
        }
    }

    fn spec_heading(&mut self, rest: &str, line_no: usize) {
        let Some(id) = rest.strip_prefix("Spec:") else {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("unrecognized top-level heading `# {rest}`"),
                "`# Spec: <spec-id>`",
            ));
            return;
        };
        let id = id.trim();
        if self.id.is_some() {
            self.errors.push(Diag::at(
                self.path,
                line_no,
                "a file declares exactly one spec",
            ));
            return;
        }
        if let Err(why) = validate_id(id, true) {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("invalid spec id: {why}"),
                "lowercase kebab-case, `/` permitted between segments",
            ));
            return;
        }
        self.id = Some(id.to_string());
    }

    fn term(&mut self, rest: &str, line_no: usize, lines: &[&str], start: usize) -> usize {
        let id = rest.trim().to_string();
        if let Err(reason) = validate_id(&id, false) {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("invalid term id: {reason}"),
                "lowercase kebab-case",
            ));
        }
        if self.terms.iter().any(|term| term.id == id) {
            self.errors.push(Diag::at(
                self.path,
                line_no,
                format!("term id `{id}` is declared twice in this spec"),
            ));
        }
        let (definition, next) = term_body(lines, start + 1);
        if definition.is_empty() {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("term `{id}` has no definition"),
                "non-empty free-form Markdown",
            ));
        }
        self.terms.push(Term {
            id,
            definition,
            line: line_no,
        });
        next
    }

    /// A requirement whose domain is a set of sites (contracts/spec.md, site-domain invariants).
    ///
    /// It carries no authored scenarios because the requirement does not range over executions.
    /// One implicit scenario is synthesized so every derived relation keys it exactly as it does a
    /// behavioural Requirement — one Requirement type, parameterized by domain, means one identity downstream.
    fn invariant(&mut self, rest: &str, line_no: usize, lines: &[&str], start: usize) -> usize {
        let id = rest.trim().to_string();
        if let Err(why) = validate_id(&id, false) {
            self.errors.push(Diag::at(
                self.path,
                line_no,
                format!("invalid invariant id: {why}"),
            ));
        }
        if self.requirements.iter().any(|r| r.id == id) {
            self.errors.push(Diag::at(
                self.path,
                line_no,
                format!("`{id}` is declared twice"),
            ));
        }

        let mut i = start + 1;
        let mut realization = None;
        let mut verification = None;
        let mut title = None;
        let mut over = None;
        let mut conditions = Vec::new();
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() {
                i += 1;
                break;
            }
            let ln = i + 1;
            if let Some(value) = trimmed.strip_prefix("Title:") {
                let value = value.trim();
                if title.is_some() {
                    self.errors
                        .push(Diag::at(self.path, ln, "`Title:` is declared twice"));
                }
                if value.is_empty() || value.chars().count() > 120 {
                    self.errors.push(Diag::expecting(
                        self.path,
                        ln,
                        "invalid Requirement title",
                        "1 to 120 characters on one line",
                    ));
                }
                title = Some(value.to_string());
            } else if let Some(value) = trimmed.strip_prefix("Realization:") {
                self.rigor("Realization", value, ln, &mut realization);
            } else if let Some(value) = trimmed.strip_prefix("Verification:") {
                self.rigor("Verification", value, ln, &mut verification);
            } else if let Some(value) = trimmed.strip_prefix(Condition::LABEL) {
                self.condition(value, ln, &mut conditions);
            } else if let Some(value) = trimmed.strip_prefix("Over:") {
                let value = value.trim();
                if let Err(why) = validate_id(value, true) {
                    self.errors.push(Diag::at(
                        self.path,
                        ln,
                        format!("invalid `Over:` id: {why}"),
                    ));
                } else {
                    over = Some(value.to_string());
                }
            } else if let Some((label, _)) = split_label(trimmed) {
                self.errors.push(Diag::expecting(
                    self.path,
                    ln,
                    format!("unknown label `{label}:` on an invariant"),
                    "Title:, Realization:, Verification:, Context: or Over:",
                ));
            } else {
                self.errors.push(Diag::expecting(
                    self.path,
                    ln,
                    "prose directly under an invariant heading",
                    "labelled lines first, then a blank line, then the normative Markdown body",
                ));
            }
            i += 1;
        }

        let (statement, next) = markdown_body(lines, i);
        i = next;
        if statement.is_empty() {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("invariant `{id}` has no statement"),
                "non-empty free-form normative Markdown",
            ));
        }

        self.requirements.push(Requirement {
            id: id.clone(),
            title,
            realization,
            verification,
            conditions,
            statement,
            scenarios: vec![Scenario {
                id,
                conditions: Vec::new(),
                statement: String::new(),
                line: line_no,
            }],
            line: line_no,
            domain: Domain::Sites,
            over,
        });
        i
    }

    /// Consumes a requirement and every scenario under it. Returns the next unconsumed index.
    fn requirement(&mut self, rest: &str, line_no: usize, lines: &[&str], start: usize) -> usize {
        let Some(id) = rest.strip_prefix("Requirement:") else {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("unrecognized heading `## {rest}`"),
                "`## Requirement: <requirement-id>`",
            ));
            return start + 1;
        };
        let id = id.trim().to_string();
        if let Err(why) = validate_id(&id, false) {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("invalid requirement id: {why}"),
                "lowercase kebab-case",
            ));
        }
        if self.requirements.iter().any(|r| r.id == id) {
            self.errors.push(Diag::at(
                self.path,
                line_no,
                format!("requirement id `{id}` is declared twice in this spec"),
            ));
        }

        let mut i = start + 1;
        let mut realization = None;
        let mut verification = None;
        let mut title = None;
        let mut conditions = Vec::new();

        // Labelled lines sit directly under the heading and end at the first blank line.
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() {
                i += 1;
                break;
            }
            let ln = i + 1;
            if let Some(value) = trimmed.strip_prefix("Title:") {
                let value = value.trim();
                if title.is_some() {
                    self.errors
                        .push(Diag::at(self.path, ln, "`Title:` is declared twice"));
                }
                if value.is_empty() || value.chars().count() > 120 {
                    self.errors.push(Diag::expecting(
                        self.path,
                        ln,
                        "invalid Requirement title",
                        "1 to 120 characters on one line",
                    ));
                }
                title = Some(value.to_string());
                i += 1;
            } else if let Some(value) = trimmed.strip_prefix("Realization:") {
                self.rigor("Realization", value, ln, &mut realization);
                i += 1;
            } else if let Some(value) = trimmed.strip_prefix("Verification:") {
                self.rigor("Verification", value, ln, &mut verification);
                i += 1;
            } else if let Some(value) = trimmed.strip_prefix(Condition::LABEL) {
                self.condition(value, ln, &mut conditions);
                i += 1;
            } else if let Some((label, _)) = split_label(trimmed) {
                self.errors.push(Diag::expecting(
                    self.path,
                    ln,
                    format!("unknown label `{label}:` on a requirement"),
                    "Title:, Realization:, Verification: or Context:",
                ));
                i += 1;
            } else {
                self.errors.push(Diag::expecting(
                    self.path,
                    ln,
                    "prose directly under a requirement heading",
                    "labelled lines first, then a blank line, then the normative Markdown body",
                ));
                i += 1;
            }
        }

        // Normative free-form Markdown, until the first Scenario or the next Requirement.
        let (statement, next) = markdown_body(lines, i);
        i = next;
        if statement.is_empty() {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("requirement `{id}` has no statement"),
                "non-empty free-form normative Markdown",
            ));
        }

        // Scenarios.
        let mut scenarios: Vec<Scenario> = Vec::new();
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                break;
            }
            if let Some(rest) = trimmed.strip_prefix("### ") {
                let (scenario, next) = self.scenario(rest, i + 1, lines, i);
                i = next;
                if let Some(scenario) = scenario {
                    if scenarios.iter().any(|s| s.id == scenario.id) {
                        self.errors.push(Diag::at(
                            self.path,
                            scenario.line,
                            format!("scenario id `{}` is declared twice", scenario.id),
                        ));
                    }
                    scenarios.push(scenario);
                }
            } else {
                i += 1;
            }
        }

        if scenarios.is_empty() {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("requirement `{id}` has no scenarios"),
                "at least one normative `### Scenario:`",
            ));
        }

        self.requirements.push(Requirement {
            id,
            title,
            realization,
            verification,
            conditions,
            statement,
            scenarios,
            line: line_no,
            domain: Domain::Behaviour,
            over: None,
        });
        i
    }

    fn scenario(
        &mut self,
        rest: &str,
        line_no: usize,
        lines: &[&str],
        start: usize,
    ) -> (Option<Scenario>, usize) {
        let Some(id) = rest.strip_prefix("Scenario:") else {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("unrecognized heading `### {rest}`"),
                "`### Scenario: <scenario-id>`",
            ));
            return (None, start + 1);
        };
        let id = id.trim().to_string();
        if let Err(why) = validate_id(&id, false) {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("invalid scenario id: {why}"),
                "lowercase kebab-case",
            ));
        }

        let mut conditions = Vec::new();
        let mut body_start = start + 1;
        while let Some(value) = lines
            .get(body_start)
            .and_then(|line| line.trim().strip_prefix(Condition::LABEL))
        {
            self.condition(value, body_start + 1, &mut conditions);
            body_start += 1;
        }

        let (statement, i) = markdown_body(lines, body_start);
        if statement.is_empty() {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("scenario `{id}` has no statement"),
                "non-empty free-form normative Markdown",
            ));
        }

        (
            Some(Scenario {
                id,
                conditions,
                statement,
                line: line_no,
            }),
            i,
        )
    }

    fn rigor(&mut self, label: &str, value: &str, line: usize, slot: &mut Option<Rigor>) {
        if slot.is_some() {
            self.errors.push(Diag::at(
                self.path,
                line,
                format!("`{label}:` is declared twice"),
            ));
        }
        match Rigor::parse(value.trim()) {
            Some(rigor) => *slot = Some(rigor),
            None => self.errors.push(Diag::expecting(
                self.path,
                line,
                format!("unknown {} rigor `{}`", label.to_lowercase(), value.trim()),
                "reviewed",
            )),
        }
    }

    fn condition(&mut self, text: &str, line: usize, conditions: &mut Vec<Condition>) {
        match Condition::parse(text, line) {
            Ok(condition) => conditions.push(condition),
            Err(why) => self.errors.push(Diag::expecting(
                self.path,
                line,
                format!("invalid condition: {why}"),
                Condition::EXPECTED,
            )),
        }
    }

    fn finish(self) -> Result<Spec, Vec<Diag>> {
        let mut errors = self.errors;

        // Scenario ids are local to their Requirement. The complete address includes both ids, making
        // authorship boundaries explicit and making promotion or movement an intentional
        // identity transition.

        let Some(id) = self.id.clone() else {
            errors.push(Diag::expecting(
                self.path,
                0,
                "no spec declared",
                "a `# Spec: <spec-id>` heading",
            ));
            return Err(errors);
        };

        if errors.is_empty() {
            Ok(Spec {
                id,
                path: self.path.to_string(),
                requirements: self.requirements,
                terms: self.terms,
            })
        } else {
            Err(errors)
        }
    }
}

fn split_label(line: &str) -> Option<(&str, &str)> {
    let idx = line.find(':')?;
    let (label, rest) = line.split_at(idx);
    if label.is_empty() || label.contains(' ') {
        return None;
    }
    Some((label, rest[1..].trim()))
}

/// Terms reserve only top-level and peer headings. Subheadings are definition content.
fn term_body(lines: &[&str], start: usize) -> (String, usize) {
    let mut fenced = false;
    let mut end = start;
    while end < lines.len() {
        let trimmed = lines[end].trim();
        if trimmed.starts_with("```") {
            fenced = !fenced;
        } else if !fenced && (trimmed.starts_with("# ") || trimmed.starts_with("## ")) {
            break;
        }
        end += 1;
    }
    let mut first = start;
    while first < end && lines[first].trim().is_empty() {
        first += 1;
    }
    let mut last = end;
    while last > first && lines[last - 1].trim().is_empty() {
        last -= 1;
    }
    (lines[first..last].join("\n"), end)
}

/// Collects one authoritative Markdown body while reserving the top three heading levels for the
/// Spec, Requirement and Scenario structure. Fenced headings remain content. Outer blank lines are removed;
/// every other byte after newline normalization is preserved because Markdown whitespace can be
/// meaningful and participates in semantic fingerprints.
fn markdown_body(lines: &[&str], start: usize) -> (String, usize) {
    let mut fenced = false;
    let mut end = start;
    while end < lines.len() {
        let trimmed = lines[end].trim();
        if trimmed.starts_with("```") {
            fenced = !fenced;
        } else if !fenced
            && (trimmed.starts_with("# ")
                || trimmed.starts_with("## ")
                || trimmed.starts_with("### "))
        {
            break;
        }
        end += 1;
    }

    let mut first = start;
    while first < end && lines[first].trim().is_empty() {
        first += 1;
    }
    let mut last = end;
    while last > first && lines[last - 1].trim().is_empty() {
        last -= 1;
    }
    (lines[first..last].join("\n"), end)
}
