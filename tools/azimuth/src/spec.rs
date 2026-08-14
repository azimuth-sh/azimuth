//! The spec parser.
//!
//! Replaces the alpha's OpenSpec reader (D11, D2.2). The grammar is documented beside the model
//! packages, and it is deliberately rigid so that a strict line-oriented parser is straightforward
//! without a parser crate (D17).
//!
//! Two failure modes are kept apart on purpose:
//!
//! - an unrecognized **construct** fails the parse (D11 — fail loudly);
//! - a missing **declaration** becomes a hole (D6.2 — a requirement without `Criticality:` parses
//!   and is reported as `unclassified`).
//!
//! Conflating them would either let syntax errors through as findings, or turn a semantic gap
//! into something a reviewer never sees in the matrix.

use crate::diag::{validate_id, Diag};
use crate::model::{Criticality, Domain, Requirement, Scenario, Spec, Step, StepKind};
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
            // claims nothing, or it is the source of claims and nothing restates it.
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
                    "`# Spec:`, `## Requirement:` or `### Scenario:`",
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

    /// A claim whose domain is a set of sites (D13).
    ///
    /// It carries no scenarios: there is no WHEN, because the claim does not range over executions.
    /// One implicit scenario is synthesized so that tags, plans and judgments key on it exactly as
    /// they do for a behavioural claim — one claim type, parameterized by domain, means one of
    /// everything downstream.
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
        let mut criticality = None;
        let mut over = None;
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() {
                i += 1;
                break;
            }
            let ln = i + 1;
            if let Some(value) = trimmed.strip_prefix("Criticality:") {
                match Criticality::parse(value.trim()) {
                    Some(c) => criticality = Some(c),
                    None => self.errors.push(Diag::expecting(
                        self.path,
                        ln,
                        format!("unknown criticality `{}`", value.trim()),
                        "critical, standard or routine",
                    )),
                }
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
                    "Criticality: or Over:",
                ));
            } else {
                self.errors.push(Diag::expecting(
                    self.path,
                    ln,
                    "prose directly under an invariant heading",
                    "labelled lines first, then a blank line, then the SHALL statement",
                ));
            }
            i += 1;
        }

        let mut statement = String::new();
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("## ")
                || trimmed.starts_with("# ")
                || trimmed.starts_with("### ")
            {
                break;
            }
            if !trimmed.is_empty() && !trimmed.starts_with('>') {
                if !statement.is_empty() {
                    statement.push(' ');
                }
                statement.push_str(trimmed);
            }
            i += 1;
        }
        if statement.is_empty() {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("invariant `{id}` has no statement"),
                "a SHALL statement in prose",
            ));
        }

        self.requirements.push(Requirement {
            id: id.clone(),
            criticality,
            statement,
            scenarios: vec![Scenario {
                id,
                steps: Vec::new(),
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
        let mut criticality = None;
        let mut saw_criticality = false;

        // Labelled lines sit directly under the heading and end at the first blank line.
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() {
                i += 1;
                break;
            }
            let ln = i + 1;
            if let Some(value) = trimmed.strip_prefix("Criticality:") {
                let value = value.trim();
                if saw_criticality {
                    self.errors
                        .push(Diag::at(self.path, ln, "`Criticality:` is declared twice"));
                }
                saw_criticality = true;
                match Criticality::parse(value) {
                    Some(c) => criticality = Some(c),
                    None => self.errors.push(Diag::expecting(
                        self.path,
                        ln,
                        format!("unknown criticality `{value}`"),
                        "critical, standard or routine",
                    )),
                }
                i += 1;
            } else if let Some((label, _)) = split_label(trimmed) {
                self.errors.push(Diag::expecting(
                    self.path,
                    ln,
                    format!("unknown label `{label}:` on a requirement"),
                    "Criticality:",
                ));
                i += 1;
            } else {
                self.errors.push(Diag::expecting(
                    self.path,
                    ln,
                    "prose directly under a requirement heading",
                    "labelled lines first, then a blank line, then the SHALL statement",
                ));
                i += 1;
            }
        }

        // Statement prose, until the first scenario or the next requirement.
        let mut statement = String::new();
        let mut fenced = false;
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("```") {
                fenced = !fenced;
                i += 1;
                continue;
            }
            if !fenced && (trimmed.starts_with("## ") || trimmed.starts_with("### ")) {
                break;
            }
            if !fenced && !trimmed.is_empty() && !trimmed.starts_with('>') {
                if !statement.is_empty() {
                    statement.push(' ');
                }
                statement.push_str(trimmed);
            }
            i += 1;
        }
        if statement.is_empty() {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("requirement `{id}` has no statement"),
                "a SHALL statement in prose",
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
                "at least one `### Scenario:` — the scenario is the unit of coverage",
            ));
        }

        self.requirements.push(Requirement {
            id,
            criticality,
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

        let mut steps: Vec<Step> = Vec::new();
        let mut i = start + 1;
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                break;
            }
            let ln = i + 1;
            match step_kind(trimmed) {
                Some((kind, text)) => {
                    if text.is_empty() {
                        self.errors.push(Diag::at(
                            self.path,
                            ln,
                            format!("`{}` has no text", kind.name().to_uppercase()),
                        ));
                    }
                    if kind == StepKind::And && steps.is_empty() {
                        self.errors.push(Diag::expecting(
                            self.path,
                            ln,
                            "`AND` with nothing to continue",
                            "a GIVEN, WHEN or THEN before it",
                        ));
                    }
                    if kind == StepKind::Given
                        && steps
                            .iter()
                            .any(|s| s.kind == StepKind::When || s.kind == StepKind::Then)
                    {
                        self.errors.push(Diag::expecting(
                            self.path,
                            ln,
                            "`GIVEN` after a WHEN or THEN",
                            "GIVEN clauses before the WHEN",
                        ));
                    }
                    if kind == StepKind::When && steps.iter().any(|s| s.kind == StepKind::Then) {
                        self.errors.push(Diag::expecting(
                            self.path,
                            ln,
                            "`WHEN` after a THEN",
                            "every WHEN before the first THEN",
                        ));
                    }
                    steps.push(Step {
                        kind,
                        text: text.to_string(),
                    });
                    i += 1;
                }
                None => {
                    self.errors.push(Diag::expecting(
                        self.path,
                        ln,
                        format!("unrecognized line in scenario `{id}`"),
                        "GIVEN, WHEN, THEN or AND",
                    ));
                    i += 1;
                }
            }
        }

        if !steps.iter().any(|s| s.kind == StepKind::When) {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("scenario `{id}` has no WHEN"),
                "exactly what triggers the behaviour",
            ));
        }
        if !steps.iter().any(|s| s.kind == StepKind::Then) {
            self.errors.push(Diag::expecting(
                self.path,
                line_no,
                format!("scenario `{id}` has no THEN"),
                "an observable outcome — a claim with no outcome cannot be satisfied or not",
            ));
        }

        (
            Some(Scenario {
                id,
                steps,
                line: line_no,
            }),
            i,
        )
    }

    fn finish(self) -> Result<Spec, Vec<Diag>> {
        let mut errors = self.errors;

        // Scenario ids are unique per *spec*, not per requirement — that is what makes splitting or
        // merging a requirement free, since tags key on (spec, scenario) and never move.
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for r in &self.requirements {
            for s in &r.scenarios {
                if let Some((other, _)) = seen.iter().find(|(id, _)| *id == s.id.as_str()) {
                    let _ = other;
                    errors.push(Diag::expecting(
                        self.path,
                        s.line,
                        format!("scenario id `{}` is not unique within this spec", s.id),
                        "scenario ids unique per spec, so that tags survive a requirement split",
                    ));
                }
                seen.push((&s.id, &r.id));
            }
        }

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

fn step_kind(line: &str) -> Option<(StepKind, &str)> {
    for (word, kind) in [
        ("GIVEN ", StepKind::Given),
        ("WHEN ", StepKind::When),
        ("THEN ", StepKind::Then),
        ("AND ", StepKind::And),
    ] {
        if let Some(rest) = line.strip_prefix(word) {
            return Some((kind, rest.trim()));
        }
    }
    for (word, kind) in [
        ("GIVEN", StepKind::Given),
        ("WHEN", StepKind::When),
        ("THEN", StepKind::Then),
        ("AND", StepKind::And),
    ] {
        if line == word {
            return Some((kind, ""));
        }
    }
    None
}
