//! The realization artifact.
//!
//! The realization facet: what makes a requirement true, and how strongly. Nothing structural is
//! written here — that is derivable from production realization linkage — so an entry is a
//! **falsifiable assertion about a named artifact**. When the code stops matching, that is a
//! Finding rather than stale prose, which is what realization documents have never had.
//!
//! Optional for every Requirement (contracts/realization.md).

use crate::diag::{validate_id, Diag};
use crate::json::{self, Json};
use crate::labels::read_block;
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_LABELS: &[&str] = &["Mechanism", "Enforcement", "Scenarios", "Binding", "Expect"];

/// The enforcement ladder, strongest first. Strength is never written: it is derived from the
/// kind, and writing it would duplicate a derivable fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enforcement {
    /// unrepresentable in the type system
    Type,
    /// unrepresentable in the data schema
    Schema,
    /// rejected by storage — unique index, FK, check, RLS
    Constraint,
    /// only possible through one place
    ChokePoint,
    /// prevented where applied, and application is opt-in
    Middleware,
    /// checked at each site
    Guard,
}

impl Enforcement {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "type" => Some(Enforcement::Type),
            "schema" => Some(Enforcement::Schema),
            "constraint" => Some(Enforcement::Constraint),
            "choke-point" => Some(Enforcement::ChokePoint),
            "middleware" => Some(Enforcement::Middleware),
            "guard" => Some(Enforcement::Guard),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Enforcement::Type => "type",
            Enforcement::Schema => "schema",
            Enforcement::Constraint => "constraint",
            Enforcement::ChokePoint => "choke-point",
            Enforcement::Middleware => "middleware",
            Enforcement::Guard => "guard",
        }
    }

    pub fn rung(self) -> u8 {
        match self {
            Enforcement::Type | Enforcement::Schema => 1,
            Enforcement::Constraint | Enforcement::ChokePoint => 2,
            Enforcement::Middleware => 3,
            Enforcement::Guard => 4,
        }
    }

    /// The top two rungs **are** proof-strength evidence — strong enforcement is
    /// self-evidencing. It does not follow that they establish any particular Requirement; that belongs
    /// to the Verification Review rather than a fictitious executable Check.
    pub fn is_proof_capable(self) -> bool {
        self.rung() <= 2
    }
}

#[derive(Debug, Clone)]
pub struct Mechanism {
    /// Stable identity owned by the realization. Implementation markers refer to this id; neither a
    /// symbol rename nor a source address becomes the conceptual identity by accident.
    pub id: String,
    pub kind: Enforcement,
    /// Empty means the mechanism bears on the complete Requirement. Otherwise these are exact local
    /// Scenario ids whose relevance is reviewed as part of total Requirement composition.
    pub scenarios: Vec<String>,
    /// Explicit for non-code artifacts. Code extractors normally derive this from an
    /// `ImplementsMechanism` site instead.
    pub binding: Option<String>,
    pub expected_unique: Option<bool>,
    pub expected_columns: Vec<String>,
    pub expected_predicate: Option<String>,
    pub line: usize,
}

/// An entry keys on the coarsest level where its statement is true. One unique index makes all
/// several `captured-once` Scenarios true, and recording it repeatedly would be duplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Requirement(String),
}

impl Target {
    pub fn id(&self) -> &str {
        match self {
            Target::Requirement(id) => id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RealizationEntry {
    pub target: Target,
    pub mechanisms: Vec<Mechanism>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Realization {
    pub spec: String,
    pub path: String,
    pub entries: Vec<RealizationEntry>,
    /// Never parsed, never derived. Orientation, danger zones, deliberately broken corners — the
    /// durable half, and the one part the machine must never pretend to understand.
    pub residue: String,
}

impl Realization {
    pub fn for_requirement(&self, id: &str) -> Option<&RealizationEntry> {
        self.entries
            .iter()
            .find(|e| e.target == Target::Requirement(id.to_string()))
    }
}

pub fn load_realizations(root: &Path) -> Result<Vec<Realization>, Vec<Diag>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect(root, &mut files).map_err(|e| {
        vec![Diag::file(
            &root.display().to_string(),
            format!("cannot read realizations: {e}"),
        )]
    })?;
    files.sort();

    let mut realizations: Vec<Realization> = Vec::new();
    let mut errors = Vec::new();
    for path in files {
        let display = path.display().to_string();
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(Diag::file(&display, format!("cannot read: {e}")));
                continue;
            }
        };
        match parse_realization(&display, &source) {
            Ok(realization) => {
                if let Some(prev) = realizations.iter().find(|d| d.spec == realization.spec) {
                    errors.push(Diag::at(
                        &display,
                        1,
                        format!(
                            "a realization for `{}` is already declared by {}",
                            realization.spec, prev.path
                        ),
                    ));
                    continue;
                }
                realizations.push(realization);
            }
            Err(mut d) => errors.append(&mut d),
        }
    }

    if errors.is_empty() {
        Ok(realizations)
    } else {
        Err(errors)
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("realization.md") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn parse_realization(path: &str, source: &str) -> Result<Realization, Vec<Diag>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut errors = Vec::new();
    let mut spec: Option<String> = None;
    let mut entries: Vec<RealizationEntry> = Vec::new();
    let mut residue = String::new();
    let mut declared_mechanisms: Vec<String> = Vec::new();
    let mut fenced = false;
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        let ln = i + 1;

        if trimmed.starts_with("```") {
            fenced = !fenced;
            i += 1;
            continue;
        }
        if fenced {
            i += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("# ") {
            match rest.strip_prefix("Realization:") {
                Some(id) => {
                    let id = id.trim();
                    if spec.is_some() {
                        errors.push(Diag::at(path, ln, "a file realizations exactly one spec"));
                    } else if let Err(why) = validate_id(id, true) {
                        errors.push(Diag::at(path, ln, format!("invalid spec id: {why}")));
                    } else {
                        spec = Some(id.to_string());
                    }
                }
                None => errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("unrecognized top-level heading `# {rest}`"),
                    "`# Realization: <spec-id>`",
                )),
            }
            i += 1;
            continue;
        }

        if trimmed == "## Residue" {
            let (block, next) = read_block(&lines, i + 1, &[]);
            residue = block.prose;
            i = next;
            continue;
        }

        let target = trimmed.strip_prefix("## Requirement:").map(|r| {
            (
                Target::Requirement(r.trim().to_string()),
                r.trim().to_string(),
            )
        });

        if let Some((target, id)) = target {
            let (block, next) = read_block(&lines, i + 1, ENTRY_LABELS);
            i = next;
            if let Err(why) = validate_id(&id, false) {
                errors.push(Diag::at(path, ln, format!("invalid id: {why}")));
                continue;
            }
            if entries.iter().any(|e| e.target == target) {
                errors.push(Diag::at(path, ln, format!("`{id}` has two entries")));
                continue;
            }
            for (text, sl) in &block.stray {
                errors.push(Diag::expecting(
                    path,
                    *sl,
                    format!("unrecognized line `{text}` under `{id}`"),
                    "a `Mechanism:` followed by `Enforcement:`, optional `Binding:` and `Expect:`",
                ));
            }

            let mut mechanisms: Vec<Mechanism> = Vec::new();
            let mut pending: Option<MechanismDraft> = None;
            for label in &block.labels {
                match label.key.as_str() {
                    "Mechanism" => {
                        finish_mechanism(path, pending.take(), &mut mechanisms, &mut errors);
                        if let Err(why) = validate_id(&label.value, false) {
                            errors.push(Diag::at(
                                path,
                                label.line,
                                format!("invalid mechanism id: {why}"),
                            ));
                        }
                        if declared_mechanisms.contains(&label.value) {
                            errors.push(Diag::at(
                                path,
                                label.line,
                                format!("mechanism `{}` is declared twice", label.value),
                            ));
                        }
                        declared_mechanisms.push(label.value.clone());
                        pending = Some(MechanismDraft {
                            id: label.value.clone(),
                            line: label.line,
                            kind: None,
                            scenarios: Vec::new(),
                            binding: None,
                            expected_unique: None,
                            expected_columns: Vec::new(),
                            expected_predicate: None,
                        });
                    }
                    "Enforcement" => match pending.as_mut() {
                        Some(draft) if draft.kind.is_none() => {
                            match Enforcement::parse(&label.value) {
                                Some(kind) => draft.kind = Some(kind),
                                None => errors.push(Diag::expecting(
                                    path,
                                    label.line,
                                    format!("unknown enforcement `{}`", label.value),
                                    "type, schema, constraint, choke-point, middleware or guard",
                                )),
                            }
                        }
                        Some(_) => errors.push(Diag::at(
                            path,
                            label.line,
                            "a mechanism declares enforcement twice",
                        )),
                        None => errors.push(Diag::expecting(
                            path,
                            label.line,
                            "`Enforcement:` with no mechanism",
                            "a `Mechanism:` line before it",
                        )),
                    },
                    "Scenarios" => match pending.as_mut() {
                        Some(draft) if draft.kind.is_some() && draft.scenarios.is_empty() => {
                            draft.scenarios =
                                parse_scenarios(path, label.line, &label.value, &mut errors);
                        }
                        Some(draft) if draft.kind.is_none() => errors.push(Diag::expecting(
                            path,
                            label.line,
                            "`Scenarios:` with no enforcement",
                            "an `Enforcement:` line before it",
                        )),
                        Some(_) => errors.push(Diag::at(
                            path,
                            label.line,
                            "a mechanism declares Scenario relevance twice",
                        )),
                        None => errors.push(Diag::expecting(
                            path,
                            label.line,
                            "`Scenarios:` with no mechanism",
                            "a `Mechanism:` and `Enforcement:` before it",
                        )),
                    },
                    "Binding" => match pending.as_mut() {
                        Some(draft) if draft.kind.is_some() && draft.binding.is_none() => {
                            if label.value.is_empty() {
                                errors.push(Diag::at(path, label.line, "`Binding:` is empty"));
                            }
                            draft.binding = Some(label.value.clone());
                        }
                        Some(draft) if draft.kind.is_none() => errors.push(Diag::expecting(
                            path,
                            label.line,
                            "`Binding:` with no enforcement",
                            "an `Enforcement:` line before it",
                        )),
                        Some(_) => errors.push(Diag::at(
                            path,
                            label.line,
                            "a mechanism declares a binding twice",
                        )),
                        None => errors.push(Diag::expecting(
                            path,
                            label.line,
                            "`Binding:` with no mechanism",
                            "a `Mechanism:` and `Enforcement:` before it",
                        )),
                    },
                    "Expect" => {
                        let Some(draft) = pending.as_mut() else {
                            errors.push(Diag::expecting(
                                path,
                                label.line,
                                "`Expect:` with no mechanism",
                                "a `Mechanism:` and `Enforcement:` before it",
                            ));
                            continue;
                        };
                        if draft.kind.is_none() {
                            errors.push(Diag::expecting(
                                path,
                                label.line,
                                "`Expect:` before enforcement",
                                "an `Enforcement:` line before it",
                            ));
                            continue;
                        }
                        for part in label
                            .value
                            .split(';')
                            .map(str::trim)
                            .filter(|p| !p.is_empty())
                        {
                            let Some((key, value)) = part.split_once('=') else {
                                errors.push(Diag::at(
                                    path,
                                    label.line,
                                    format!("invalid expected property `{part}`"),
                                ));
                                continue;
                            };
                            match key.trim() {
                                "unique" => match value.trim() {
                                    "true" => draft.expected_unique = Some(true),
                                    "false" => draft.expected_unique = Some(false),
                                    other => errors.push(Diag::at(
                                        path,
                                        label.line,
                                        format!("expected unique is not a boolean: `{other}`"),
                                    )),
                                },
                                "columns" => {
                                    draft.expected_columns = value
                                        .split(',')
                                        .map(str::trim)
                                        .filter(|column| !column.is_empty())
                                        .map(str::to_string)
                                        .collect();
                                }
                                "predicate" => {
                                    draft.expected_predicate = Some(value.trim().to_string())
                                }
                                other => errors.push(Diag::at(
                                    path,
                                    label.line,
                                    format!("unknown expected property `{other}`"),
                                )),
                            }
                        }
                    }
                    _ => unreachable!("labels are restricted at read time"),
                }
            }
            finish_mechanism(path, pending, &mut mechanisms, &mut errors);

            if mechanisms.is_empty() {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("`{id}` declares no mechanism"),
                    "a `Mechanism:` and `Enforcement:` pair",
                ));
            }
            // Without a reason, an entry records a fact the code already knows.
            if block.prose.is_empty() {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("`{id}` gives no reason"),
                    "prose: why this mechanism, what was rejected, what breaks if it changes",
                ));
            }

            entries.push(RealizationEntry {
                target,
                mechanisms,
                line: ln,
            });
            continue;
        }

        if trimmed.starts_with('#') {
            errors.push(Diag::expecting(
                path,
                ln,
                format!("unrecognized heading `{trimmed}`"),
                "`# Realization:`, `## Requirement:` or `## Residue`",
            ));
        }
        i += 1;
    }

    let Some(spec) = spec else {
        errors.push(Diag::expecting(
            path,
            0,
            "no spec designed",
            "a `# Realization: <spec-id>` heading",
        ));
        return Err(errors);
    };

    if errors.is_empty() {
        Ok(Realization {
            spec,
            path: path.to_string(),
            entries,
            residue,
        })
    } else {
        Err(errors)
    }
}

struct MechanismDraft {
    id: String,
    line: usize,
    kind: Option<Enforcement>,
    scenarios: Vec<String>,
    binding: Option<String>,
    expected_unique: Option<bool>,
    expected_columns: Vec<String>,
    expected_predicate: Option<String>,
}

fn parse_scenarios(path: &str, line: usize, value: &str, errors: &mut Vec<Diag>) -> Vec<String> {
    let parsed = match json::parse(value) {
        Ok(Json::Arr(items)) => items,
        Ok(_) => {
            errors.push(Diag::expecting(
                path,
                line,
                "Scenarios is not an array",
                "a non-empty JSON array of unique local Scenario ids",
            ));
            return Vec::new();
        }
        Err(reason) => {
            errors.push(Diag::at(
                path,
                line,
                format!("invalid Scenarios JSON: {reason}"),
            ));
            return Vec::new();
        }
    };
    let mut scenarios = Vec::new();
    for item in parsed {
        let Json::Str(id) = item else {
            errors.push(Diag::expecting(
                path,
                line,
                "Scenarios contains a non-string value",
                "a non-empty JSON array of unique local Scenario ids",
            ));
            continue;
        };
        if validate_id(&id, false).is_err() {
            errors.push(Diag::at(path, line, format!("invalid Scenario id `{id}`")));
        }
        scenarios.push(id);
    }
    let authored_scenarios = scenarios.clone();
    scenarios.sort();
    let original_len = scenarios.len();
    scenarios.dedup();
    if scenarios.is_empty() {
        errors.push(Diag::at(path, line, "Scenarios must not be empty"));
    } else if scenarios.len() != original_len {
        errors.push(Diag::at(path, line, "Scenarios repeats a Scenario id"));
    } else if scenarios != authored_scenarios {
        errors.push(Diag::expecting(
            path,
            line,
            "Scenarios is not sorted",
            "local Scenario ids in ascending byte order",
        ));
    }
    scenarios
}

fn finish_mechanism(
    path: &str,
    draft: Option<MechanismDraft>,
    mechanisms: &mut Vec<Mechanism>,
    errors: &mut Vec<Diag>,
) {
    let Some(draft) = draft else { return };
    let Some(kind) = draft.kind else {
        errors.push(Diag::expecting(
            path,
            draft.line,
            format!("mechanism `{}` has no enforcement", draft.id),
            "an `Enforcement:` line after every `Mechanism:`",
        ));
        return;
    };
    mechanisms.push(Mechanism {
        id: draft.id,
        kind,
        scenarios: draft.scenarios,
        binding: draft.binding,
        expected_unique: draft.expected_unique,
        expected_columns: draft.expected_columns,
        expected_predicate: draft.expected_predicate,
        line: draft.line,
    });
}
