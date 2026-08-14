//! Verification plans and the project standards.
//!
//! The evidence facet (D3). A plan records **what would be sufficient to believe a claim**, never
//! what exists — existing evidence is derived from `covers` tags, and hand-listing it would create
//! a second copy that drifts (D4.5).
//!
//! Entries are deviations only. A claim with no entry is not unplanned: the standard applies.
//!
//! Two field groups that are easy to confuse, and are kept apart syntactically:
//!
//! - `Scope`, `Quantification`, `Oracle` state the **required** form, overriding the standard;
//! - `Evidence` plus its `Strength` declares a **provided** non-test evidence item.
//!
//! `Strength` without `Evidence` is therefore an error: on its own it would read as either.

use crate::diag::{validate_id, Diag};
use crate::labels::read_block;
use crate::model::{Criticality, Oracle, Quantification, Scope, Strength};
use std::fs;
use std::path::{Path, PathBuf};

const CLAIM_LABELS: &[&str] = &[
    "Scope",
    "Quantification",
    "Oracle",
    "Strength",
    "Evidence",
    "Binding",
    "Re-established",
    "Dies silently",
    "Detector test",
    "Detector binding",
    "Residual",
    "Accepted",
];

const RESIDUAL_LABELS: &[&str] = &["Accepted"];

const STANDARD_LABELS: &[&str] = &["Strength", "Quantification", "Residual"];

#[derive(Debug, Clone)]
pub struct EvidenceItem {
    pub description: String,
    pub strength: Strength,
    pub re_established: Option<String>,
    pub dies_silently: Option<String>,
    pub detector_test: Option<String>,
    pub bindings: Vec<String>,
    pub detector_bindings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlanEntry {
    pub scenario: String,
    pub scope: Option<Scope>,
    pub quantification: Option<Quantification>,
    pub oracle: Option<Oracle>,
    pub evidence: Option<EvidenceItem>,
    pub residual: Option<String>,
    pub accepted: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Residual {
    pub id: String,
    pub description: String,
    pub accepted: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub spec: String,
    pub path: String,
    pub entries: Vec<PlanEntry>,
    pub residuals: Vec<Residual>,
}

impl Plan {
    pub fn entry(&self, scenario: &str) -> Option<&PlanEntry> {
        self.entries.iter().find(|e| e.scenario == scenario)
    }
}

/// The project-level mapping from criticality to required evidence, written once (D6.1).
#[derive(Debug, Clone)]
pub struct LevelStandard {
    pub criticality: Criticality,
    /// `None` means no evidence is required at this level — D6.5's `routine`.
    pub strength: Option<Strength>,
    pub quantification: Option<Quantification>,
    pub residual_required: bool,
}

#[derive(Debug, Clone)]
pub struct Standards {
    pub path: String,
    /// D15: scope is not derived from criticality. Default `unit`, raised per claim where truth
    /// depends on something real.
    pub default_scope: Scope,
    pub levels: Vec<LevelStandard>,
}

impl Standards {
    pub fn for_level(&self, c: Criticality) -> Option<&LevelStandard> {
        self.levels.iter().find(|l| l.criticality == c)
    }
}

pub fn load_standards(path: &Path) -> Result<Standards, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path)
        .map_err(|e| vec![Diag::file(&display, format!("cannot read standards: {e}"))])?;
    parse_standards(&display, &source)
}

pub fn parse_standards(path: &str, source: &str) -> Result<Standards, Vec<Diag>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut errors = Vec::new();
    let mut default_scope = None;
    let mut levels: Vec<LevelStandard> = Vec::new();
    let mut saw_heading = false;
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        let ln = i + 1;

        if trimmed == "# Verification standards" {
            saw_heading = true;
            let (block, next) = read_block(&lines, i + 1, &["Default scope"]);
            for (text, line) in &block.stray {
                errors.push(Diag::expecting(
                    path,
                    *line,
                    format!("unrecognized line `{text}`"),
                    "`Default scope:`",
                ));
            }
            match block.value("Default scope") {
                Some(v) => match Scope::parse(v) {
                    Some(s) => default_scope = Some(s),
                    None => errors.push(Diag::expecting(
                        path,
                        ln,
                        format!("unknown scope `{v}`"),
                        "unit, component or e2e",
                    )),
                },
                None => errors.push(Diag::expecting(
                    path,
                    ln,
                    "no default scope",
                    "`Default scope: unit` directly under the title (D15)",
                )),
            }
            i = next;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("## Level:") {
            let name = rest.trim();
            let (block, next) = read_block(&lines, i + 1, STANDARD_LABELS);
            i = next;

            let Some(criticality) = Criticality::parse(name) else {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("unknown level `{name}`"),
                    "critical, standard or routine",
                ));
                continue;
            };
            if levels.iter().any(|l| l.criticality == criticality) {
                errors.push(Diag::at(
                    path,
                    ln,
                    format!("level `{name}` is declared twice"),
                ));
                continue;
            }

            let mut strength = None;
            match block.value("Strength") {
                Some("none") => {}
                Some(v) => match Strength::parse(v) {
                    Some(s) => strength = Some(s),
                    None => errors.push(Diag::expecting(
                        path,
                        ln,
                        format!("unknown strength `{v}`"),
                        "proof, demonstration, detection or none",
                    )),
                },
                None => errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("level `{name}` declares no minimum strength"),
                    "`Strength:` — use `none` where no evidence is required",
                )),
            }

            let mut quantification = None;
            if let Some(v) = block.value("Quantification") {
                match Quantification::parse(v) {
                    Some(q) => quantification = Some(q),
                    None => errors.push(Diag::expecting(
                        path,
                        ln,
                        format!("unknown quantification `{v}`"),
                        "example or universal",
                    )),
                }
            } else if strength.is_some() {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("level `{name}` requires evidence but no quantification"),
                    "`Quantification: example` or `universal`",
                ));
            }

            let residual_required = match block.value("Residual") {
                Some("required") => true,
                Some("optional") => false,
                Some(v) => {
                    errors.push(Diag::expecting(
                        path,
                        ln,
                        format!("unknown residual policy `{v}`"),
                        "required or optional",
                    ));
                    false
                }
                None => {
                    errors.push(Diag::expecting(
                        path,
                        ln,
                        format!("level `{name}` declares no residual policy"),
                        "`Residual: required` or `optional`",
                    ));
                    false
                }
            };

            levels.push(LevelStandard {
                criticality,
                strength,
                quantification,
                residual_required,
            });
            continue;
        }

        i += 1;
    }

    if !saw_heading {
        errors.push(Diag::expecting(
            path,
            0,
            "not a standards file",
            "`# Verification standards`",
        ));
    }
    for c in [
        Criticality::Critical,
        Criticality::Standard,
        Criticality::Routine,
    ] {
        if !levels.iter().any(|l| l.criticality == c) {
            errors.push(Diag::expecting(
                path,
                0,
                format!("no standard for `{}`", c.name()),
                "a `## Level:` block for every criticality — the set is closed (D6.4)",
            ));
        }
    }

    if errors.is_empty() {
        Ok(Standards {
            path: path.to_string(),
            default_scope: default_scope.unwrap(),
            levels,
        })
    } else {
        Err(errors)
    }
}

pub fn load_plans(root: &Path) -> Result<Vec<Plan>, Vec<Diag>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect(root, &mut files).map_err(|e| {
        vec![Diag::file(
            &root.display().to_string(),
            format!("cannot read plans: {e}"),
        )]
    })?;
    files.sort();

    let mut plans: Vec<Plan> = Vec::new();
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
        match parse_plan(&display, &source) {
            Ok(plan) => {
                if let Some(prev) = plans.iter().find(|p| p.spec == plan.spec) {
                    errors.push(Diag::at(
                        &display,
                        1,
                        format!(
                            "a plan for `{}` is already declared by {}",
                            plan.spec, prev.path
                        ),
                    ));
                    continue;
                }
                plans.push(plan);
            }
            Err(mut d) => errors.append(&mut d),
        }
    }

    if errors.is_empty() {
        Ok(plans)
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
        } else if path.file_name().and_then(|n| n.to_str()) == Some("verification.md") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn parse_plan(path: &str, source: &str) -> Result<Plan, Vec<Diag>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut errors = Vec::new();
    let mut spec: Option<String> = None;
    let mut entries: Vec<PlanEntry> = Vec::new();
    let mut residuals: Vec<Residual> = Vec::new();
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
            match rest.strip_prefix("Verification:") {
                Some(id) => {
                    let id = id.trim();
                    if spec.is_some() {
                        errors.push(Diag::at(path, ln, "a file plans exactly one spec"));
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
                    "`# Verification: <spec-id>`",
                )),
            }
            i += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("## Claim:") {
            let id = rest.trim().to_string();
            let (block, next) = read_block(&lines, i + 1, CLAIM_LABELS);
            i = next;
            if let Err(why) = validate_id(&id, false) {
                errors.push(Diag::at(path, ln, format!("invalid scenario id: {why}")));
                continue;
            }
            if entries.iter().any(|e| e.scenario == id) {
                errors.push(Diag::at(path, ln, format!("claim `{id}` has two entries")));
                continue;
            }
            match claim_entry(path, ln, &id, &block, &mut errors) {
                Some(entry) => entries.push(entry),
                None => continue,
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("## Residual:") {
            let id = rest.trim().to_string();
            let (block, next) = read_block(&lines, i + 1, RESIDUAL_LABELS);
            i = next;
            if let Err(why) = validate_id(&id, false) {
                errors.push(Diag::at(path, ln, format!("invalid residual id: {why}")));
                continue;
            }
            for (text, sl) in &block.stray {
                errors.push(Diag::expecting(
                    path,
                    *sl,
                    format!("unrecognized line `{text}` under residual `{id}`"),
                    "`Accepted:` first, then a blank line, then the description",
                ));
            }
            let accepted = match block.value("Accepted") {
                Some(v) if !v.is_empty() => v.to_string(),
                _ => {
                    errors.push(Diag::expecting(
                        path,
                        ln,
                        format!("residual `{id}` is not accepted"),
                        "`Accepted:` — an unrecorded absence is not an exemption (D6.3)",
                    ));
                    String::new()
                }
            };
            if block.prose.is_empty() {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("residual `{id}` has no description"),
                    "prose saying what is not covered",
                ));
            }
            residuals.push(Residual {
                id,
                description: block.prose,
                accepted,
                line: ln,
            });
            continue;
        }

        if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
            errors.push(Diag::expecting(
                path,
                ln,
                format!("unrecognized heading `{trimmed}`"),
                "`# Verification:`, `## Claim:` or `## Residual:`",
            ));
        }
        i += 1;
    }

    let Some(spec) = spec else {
        errors.push(Diag::expecting(
            path,
            0,
            "no spec planned",
            "a `# Verification: <spec-id>` heading",
        ));
        return Err(errors);
    };

    if errors.is_empty() {
        Ok(Plan {
            spec,
            path: path.to_string(),
            entries,
            residuals,
        })
    } else {
        Err(errors)
    }
}

fn claim_entry(
    path: &str,
    line: usize,
    id: &str,
    block: &crate::labels::Block,
    errors: &mut Vec<Diag>,
) -> Option<PlanEntry> {
    for (text, ln) in &block.stray {
        errors.push(Diag::expecting(
            path,
            *ln,
            format!("unrecognized line `{text}` under claim `{id}`"),
            format!("one of: {}", CLAIM_LABELS.join(", ")),
        ));
    }
    for dup in block.duplicates() {
        errors.push(Diag::at(
            path,
            dup.line,
            format!("`{}:` is declared twice", dup.key),
        ));
    }

    let mut scope = None;
    if let Some(v) = block.value("Scope") {
        match Scope::parse(v) {
            Some(s) => scope = Some(s),
            None => errors.push(Diag::expecting(
                path,
                line,
                format!("unknown scope `{v}`"),
                "unit, component or e2e",
            )),
        }
    }

    let mut quantification = None;
    if let Some(v) = block.value("Quantification") {
        match Quantification::parse(v) {
            Some(q) => quantification = Some(q),
            None => errors.push(Diag::expecting(
                path,
                line,
                format!("unknown quantification `{v}`"),
                "example or universal",
            )),
        }
    }

    let evidence = match (block.value("Evidence"), block.value("Strength")) {
        (Some(description), Some(strength_text)) => {
            let Some(strength) = Strength::parse(strength_text) else {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("unknown strength `{strength_text}`"),
                    "proof, demonstration or detection",
                ));
                return None;
            };
            let re_established = block.value("Re-established").map(str::to_string);
            let dies_silently = block.value("Dies silently").map(str::to_string);
            let detector_test = block.value("Detector test").map(str::to_string);
            let bindings = binding_list(block.value("Binding"));
            let detector_bindings = binding_list(block.value("Detector binding"));

            // D4.3: a monitor that can no longer fire is worse than no monitor, because it is
            // carried on the books as evidence. The detector test is what makes it checkable
            // before release.
            if strength == Strength::Detection {
                for (value, want) in [
                    (&re_established, "Re-established:"),
                    (&dies_silently, "Dies silently:"),
                    (&detector_test, "Detector test:"),
                ] {
                    if value.is_none() {
                        errors.push(Diag::expecting(
                            path,
                            line,
                            format!("detection evidence for `{id}` is incomplete"),
                            format!("{want} — required for every detection item (D4.3)"),
                        ));
                    }
                }
                for (bindings, want) in [
                    (&bindings, "Binding:"),
                    (&detector_bindings, "Detector binding:"),
                ] {
                    if bindings.is_empty() {
                        errors.push(Diag::expecting(
                            path,
                            line,
                            format!("detection evidence for `{id}` has no machine binding"),
                            format!("{want} — required for every detection item"),
                        ));
                    }
                }
            }
            Some(EvidenceItem {
                description: description.to_string(),
                strength,
                re_established,
                dies_silently,
                detector_test,
                bindings,
                detector_bindings,
            })
        }
        (Some(_), None) => {
            errors.push(Diag::expecting(
                path,
                line,
                format!("evidence for `{id}` declares no strength"),
                "`Strength:` — how far this evidence reaches (D4.1)",
            ));
            None
        }
        (None, Some(_)) => {
            errors.push(Diag::expecting(
                path,
                line,
                format!("`Strength:` on `{id}` without evidence"),
                "an `Evidence:` line — Strength qualifies a provided item, while Scope and \
                 Quantification state what is required",
            ));
            None
        }
        (None, None) => {
            if block.value("Binding").is_some() || block.value("Detector binding").is_some() {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("bindings on `{id}` without evidence"),
                    "an `Evidence:` and `Strength:` declaration",
                ));
            }
            None
        }
    };

    let residual = block.value("Residual").map(str::to_string);
    let accepted = block.value("Accepted").map(str::to_string);
    if residual.is_some() != accepted.is_some() {
        errors.push(Diag::expecting(
            path,
            line,
            format!("`{id}` records a residual without accepting it, or the reverse"),
            "`Residual:` and `Accepted:` together — silent weakening is not available",
        ));
    }

    // "An entry without a reason is a number nobody can review." An accepted residual is itself
    // the reason, so it stands in for prose.
    if block.prose.is_empty() && accepted.is_none() {
        errors.push(Diag::expecting(
            path,
            line,
            format!("entry for `{id}` gives no reason"),
            "prose saying why this claim needs what it needs",
        ));
    }

    let oracle = block
        .value("Oracle")
        .and_then(|value| match Oracle::parse(value) {
            Some(oracle) => Some(oracle),
            None => {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("unknown oracle `{value}`"),
                    "direct, golden, relational, metamorphic, model-based or contract",
                ));
                None
            }
        });

    Some(PlanEntry {
        scenario: id.to_string(),
        scope,
        quantification,
        oracle,
        evidence,
        residual,
        accepted,
        line,
    })
}

fn binding_list(value: Option<&str>) -> Vec<String> {
    value
        .into_iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}
