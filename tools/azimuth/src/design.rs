//! The design artifact.
//!
//! The mechanism facet: what makes a claim true, and how strongly. Nothing structural is
//! written here — that is derivable from production realization linkage — so an entry is a
//! **falsifiable assertion about a named artifact**. When the code stops matching, that is a
//! Finding rather than stale prose, which is what design documents have never had.
//!
//! Required for `critical` Claims, optional for `standard`, absent for `routine`
//! (contracts/spec.md, criticality).

use crate::diag::{validate_id, Diag};
use crate::json::{self, Json};
use crate::labels::read_block;
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_LABELS: &[&str] = &["Mechanism", "Enforcement", "Cases", "Binding", "Expect"];

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
    /// self-evidencing. It does not follow that they establish any particular Claim; that belongs
    /// to total Claim Judgment rather than a fictitious executable Check.
    pub fn is_proof_capable(self) -> bool {
        self.rung() <= 2
    }
}

#[derive(Debug, Clone)]
pub struct Mechanism {
    /// Stable identity owned by the design. Implementation markers refer to this id; neither a
    /// symbol rename nor a source address becomes the conceptual identity by accident.
    pub id: String,
    pub kind: Enforcement,
    /// Empty means the mechanism bears on the complete Claim. Otherwise these are exact local
    /// Case ids whose relevance is reviewed as part of total Claim composition.
    pub cases: Vec<String>,
    /// Explicit for non-code artifacts. Code extractors normally derive this from an
    /// `ImplementsMechanism` site instead.
    pub binding: Option<String>,
    pub expected_unique: Option<bool>,
    pub expected_columns: Vec<String>,
    pub expected_predicate: Option<String>,
    pub line: usize,
}

/// An entry keys on the coarsest level where its statement is true. One unique index makes all
/// several `captured-once` Cases true, and recording it repeatedly would be duplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Claim(String),
}

impl Target {
    pub fn id(&self) -> &str {
        match self {
            Target::Claim(id) => id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesignEntry {
    pub target: Target,
    pub mechanisms: Vec<Mechanism>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Design {
    pub spec: String,
    pub path: String,
    pub entries: Vec<DesignEntry>,
    /// Never parsed, never derived. Orientation, danger zones, deliberately broken corners — the
    /// durable half, and the one part the machine must never pretend to understand.
    pub residue: String,
}

impl Design {
    pub fn for_claim(&self, id: &str) -> Option<&DesignEntry> {
        self.entries
            .iter()
            .find(|e| e.target == Target::Claim(id.to_string()))
    }
}

pub fn load_designs(root: &Path) -> Result<Vec<Design>, Vec<Diag>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect(root, &mut files).map_err(|e| {
        vec![Diag::file(
            &root.display().to_string(),
            format!("cannot read designs: {e}"),
        )]
    })?;
    files.sort();

    let mut designs: Vec<Design> = Vec::new();
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
        match parse_design(&display, &source) {
            Ok(design) => {
                if let Some(prev) = designs.iter().find(|d| d.spec == design.spec) {
                    errors.push(Diag::at(
                        &display,
                        1,
                        format!(
                            "a design for `{}` is already declared by {}",
                            design.spec, prev.path
                        ),
                    ));
                    continue;
                }
                designs.push(design);
            }
            Err(mut d) => errors.append(&mut d),
        }
    }

    if errors.is_empty() {
        Ok(designs)
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
        } else if path.file_name().and_then(|n| n.to_str()) == Some("design.md") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn parse_design(path: &str, source: &str) -> Result<Design, Vec<Diag>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut errors = Vec::new();
    let mut spec: Option<String> = None;
    let mut entries: Vec<DesignEntry> = Vec::new();
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
            match rest.strip_prefix("Design:") {
                Some(id) => {
                    let id = id.trim();
                    if spec.is_some() {
                        errors.push(Diag::at(path, ln, "a file designs exactly one spec"));
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
                    "`# Design: <spec-id>`",
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

        let target = trimmed
            .strip_prefix("## Claim:")
            .map(|r| (Target::Claim(r.trim().to_string()), r.trim().to_string()));

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
                            cases: Vec::new(),
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
                    "Cases" => match pending.as_mut() {
                        Some(draft) if draft.kind.is_some() && draft.cases.is_empty() => {
                            draft.cases = parse_cases(path, label.line, &label.value, &mut errors);
                        }
                        Some(draft) if draft.kind.is_none() => errors.push(Diag::expecting(
                            path,
                            label.line,
                            "`Cases:` with no enforcement",
                            "an `Enforcement:` line before it",
                        )),
                        Some(_) => errors.push(Diag::at(
                            path,
                            label.line,
                            "a mechanism declares Case relevance twice",
                        )),
                        None => errors.push(Diag::expecting(
                            path,
                            label.line,
                            "`Cases:` with no mechanism",
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

            entries.push(DesignEntry {
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
                "`# Design:`, `## Claim:` or `## Residue`",
            ));
        }
        i += 1;
    }

    let Some(spec) = spec else {
        errors.push(Diag::expecting(
            path,
            0,
            "no spec designed",
            "a `# Design: <spec-id>` heading",
        ));
        return Err(errors);
    };

    if errors.is_empty() {
        Ok(Design {
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
    cases: Vec<String>,
    binding: Option<String>,
    expected_unique: Option<bool>,
    expected_columns: Vec<String>,
    expected_predicate: Option<String>,
}

fn parse_cases(path: &str, line: usize, value: &str, errors: &mut Vec<Diag>) -> Vec<String> {
    let parsed = match json::parse(value) {
        Ok(Json::Arr(items)) => items,
        Ok(_) => {
            errors.push(Diag::expecting(
                path,
                line,
                "Cases is not an array",
                "a non-empty JSON array of unique local Case ids",
            ));
            return Vec::new();
        }
        Err(reason) => {
            errors.push(Diag::at(
                path,
                line,
                format!("invalid Cases JSON: {reason}"),
            ));
            return Vec::new();
        }
    };
    let mut cases = Vec::new();
    for item in parsed {
        let Json::Str(id) = item else {
            errors.push(Diag::expecting(
                path,
                line,
                "Cases contains a non-string value",
                "a non-empty JSON array of unique local Case ids",
            ));
            continue;
        };
        if validate_id(&id, false).is_err() {
            errors.push(Diag::at(path, line, format!("invalid Case id `{id}`")));
        }
        cases.push(id);
    }
    let authored_cases = cases.clone();
    cases.sort();
    let original_len = cases.len();
    cases.dedup();
    if cases.is_empty() {
        errors.push(Diag::at(path, line, "Cases must not be empty"));
    } else if cases.len() != original_len {
        errors.push(Diag::at(path, line, "Cases repeats a Case id"));
    } else if cases != authored_cases {
        errors.push(Diag::expecting(
            path,
            line,
            "Cases is not sorted",
            "local Case ids in ascending byte order",
        ));
    }
    cases
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
        cases: draft.cases,
        binding: draft.binding,
        expected_unique: draft.expected_unique,
        expected_columns: draft.expected_columns,
        expected_predicate: draft.expected_predicate,
        line: draft.line,
    });
}
