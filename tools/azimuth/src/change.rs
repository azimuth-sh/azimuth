//! The change lifecycle projects reviewed intent without treating unfinished behavior as accepted.
//!
//! The machine derives additions and their obligations, and verifies that an accepted change was
//! applied before archiving. Explanations of departures and residual acceptance remain authored
//! because deriving them would manufacture the judgment the archive exists to preserve.

use crate::json::Json;
use crate::model::{Model, Rigor};
use crate::spec::parse_spec;
use crate::validation::{self, Finding};
use std::fs;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Addition {
    pub spec: String,
    pub requirement: String,
    pub realization: Option<Rigor>,
    pub verification: Option<Rigor>,
    pub statement: String,
    pub scenarios: Vec<AddedScenario>,
    pub applied: bool,
}

#[derive(Debug, Clone)]
pub struct AddedScenario {
    pub id: String,
    pub statement: String,
}

#[derive(Debug)]
pub struct Report {
    pub id: String,
    pub additions: Vec<Addition>,
    pub mutations: usize,
    pub unapplied_mutations: usize,
    pub unchanged_intent_reason: Option<String>,
    pub correction_requirements: Vec<String>,
    pub authority_model_fingerprint: String,
    pub incomplete_plan_items: usize,
    pub current_requirements: usize,
    pub target_requirements: usize,
}

pub fn inspect(root: &Path, model: &Model) -> Result<Report, Vec<String>> {
    let mut errors = Vec::new();
    let Some(id) = root
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
    else {
        return Err(vec!["change path has no id".into()]);
    };
    let proposal_path = root.join("proposal.md");
    let proposal = match fs::read_to_string(&proposal_path) {
        Ok(proposal) => proposal,
        Err(_) => {
            errors.push("proposal.md is missing".into());
            String::new()
        }
    };
    let unchanged_intent_reason =
        parse_unchanged_intent_reason(&proposal_path, &proposal, &mut errors);
    let plan = match fs::read_to_string(root.join("plan.md")) {
        Ok(plan) => plan,
        Err(_) => {
            errors.push("plan.md is missing".into());
            String::new()
        }
    };
    let incomplete_plan_items = plan
        .lines()
        .filter(|line| line.trim_start().starts_with("- [ ]"))
        .count();

    let mut additions = Vec::new();
    let mut mutations = 0;
    match crate::intent::delta_paths(root) {
        Ok(files) => {
            for file in files {
                match fs::read_to_string(&file) {
                    Ok(source) => parse_delta(
                        &file,
                        &source,
                        model,
                        &mut additions,
                        &mut mutations,
                        &mut errors,
                    ),
                    Err(error) => errors.push(format!("cannot read {}: {error}", file.display())),
                }
            }
        }
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }

    if unchanged_intent_reason.is_some() && (!additions.is_empty() || mutations > 0) {
        errors.push(
            "proposal declares unchanged intent but specs contain a supported intent delta".into(),
        );
    }

    let unapplied_mutations = if mutations > 0 && errors.is_empty() {
        match crate::intent::project(root, Path::new("azimuth/model"), model) {
            Ok(projected) => projected
                .operations
                .iter()
                .filter(|(name, applied)| !*applied && !name.starts_with("add "))
                .count(),
            Err(projection_errors) => {
                errors.extend(projection_errors);
                0
            }
        }
    } else {
        0
    };
    let correction_requirements = match read_correction(root) {
        Ok(Some(correction)) => correction.requirements,
        Ok(None) => Vec::new(),
        Err(mut diagnostics) => {
            errors.append(&mut diagnostics);
            Vec::new()
        }
    };
    for requirement in &correction_requirements {
        if !model
            .requirements()
            .any(|view| view.requirement.id == *requirement)
            && !additions
                .iter()
                .any(|addition| addition.requirement == *requirement)
        {
            errors.push(format!(
                "correction scope names unknown Requirement {requirement}"
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let unapplied_requirements = additions
        .iter()
        .filter(|addition| !addition.applied)
        .count();
    Ok(Report {
        id,
        additions,
        mutations,
        unapplied_mutations,
        unchanged_intent_reason,
        correction_requirements,
        authority_model_fingerprint: format!(
            "sha256:{}",
            crate::fingerprint::authority_model_digest(model)
        ),
        incomplete_plan_items,
        current_requirements: model.requirement_count(),
        target_requirements: model.requirement_count() + unapplied_requirements,
    })
}

fn parse_unchanged_intent_reason(
    path: &Path,
    proposal: &str,
    errors: &mut Vec<String>,
) -> Option<String> {
    let metadata = proposal
        .lines()
        .enumerate()
        .take_while(|(_, line)| !line.trim_start().starts_with("## "))
        .collect::<Vec<_>>();
    let declarations = metadata
        .iter()
        .enumerate()
        .filter(|(_, (_, line))| line.trim_start().starts_with("Intent delta:"))
        .collect::<Vec<_>>();

    if declarations.len() > 1 {
        errors.push(format!(
            "{}: proposal metadata declares `Intent delta:` more than once",
            path.display()
        ));
        return None;
    }
    let Some((metadata_index, (line_index, declaration))) = declarations.first().copied() else {
        return None;
    };
    if declaration.trim() != "Intent delta: none" {
        errors.push(format!(
            "{}:{}: proposal metadata supports only `Intent delta: none`",
            path.display(),
            line_index + 1
        ));
        return None;
    }

    let Some((because_line, because)) = metadata.get(metadata_index + 1).copied() else {
        errors.push(format!(
            "{}:{}: `Intent delta: none` requires an adjacent non-empty `Because:`",
            path.display(),
            line_index + 1
        ));
        return None;
    };
    let Some(reason) = because.trim().strip_prefix("Because:").map(str::trim) else {
        errors.push(format!(
            "{}:{}: `Intent delta: none` requires an adjacent non-empty `Because:`",
            path.display(),
            because_line + 1
        ));
        return None;
    };
    if reason.is_empty() {
        errors.push(format!(
            "{}:{}: `Because:` must state why accepted intent is unchanged",
            path.display(),
            because_line + 1
        ));
        return None;
    }
    Some(reason.to_string())
}

fn parse_delta(
    path: &Path,
    source: &str,
    model: &Model,
    additions: &mut Vec<Addition>,
    mutations: &mut usize,
    errors: &mut Vec<String>,
) {
    let mut normalized = String::new();
    let lines: Vec<&str> = source.lines().collect();
    let Some(first) = lines.first() else {
        errors.push(format!(
            "{}: expected `# Intent delta: <spec-id>`",
            path.display()
        ));
        return;
    };
    let Some(spec) = first.trim().strip_prefix("# Intent delta:").map(str::trim) else {
        errors.push(format!(
            "{}: expected `# Intent delta: <spec-id>`",
            path.display()
        ));
        return;
    };
    normalized.push_str("# Spec: ");
    normalized.push_str(spec);
    normalized.push('\n');

    let mut has_addition = false;
    let mut index = 1;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if let Some(requirement) = trimmed.strip_prefix("## Add requirement:") {
            normalized.push_str("## Requirement:");
            normalized.push_str(requirement);
            has_addition = true;
        } else if let Some(term) = trimmed.strip_prefix("## Add term:") {
            normalized.push_str("## Term:");
            normalized.push_str(term);
            has_addition = true;
            *mutations += 1;
        } else if let Some(scenario) = trimmed.strip_prefix("### Add scenario:") {
            normalized.push_str("### Scenario:");
            normalized.push_str(scenario);
        } else if trimmed.starts_with("## Replace requirement:")
            || trimmed.starts_with("## Remove requirement:")
            || trimmed.starts_with("## Remove scenario:")
        {
            *mutations += 1;
            index += 1;
            while index < lines.len() && !lines[index].starts_with("## ") {
                index += 1;
            }
            continue;
        } else if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            errors.push(format!(
                "{}:{}: unsupported delta operation `{trimmed}`; additions are machine-projected",
                path.display(),
                index + 1,
            ));
        } else {
            normalized.push_str(line);
        }
        normalized.push('\n');
        index += 1;
    }

    if !errors.is_empty() {
        return;
    }
    if !has_addition {
        return;
    }
    let parsed = match parse_spec(&path.display().to_string(), &normalized) {
        Ok(parsed) => parsed,
        Err(diags) => {
            errors.extend(diags.into_iter().map(|diag| diag.to_string()));
            return;
        }
    };

    for requirement in parsed.requirements {
        if requirement.scenarios.is_empty() {
            errors.push(format!(
                "{}: `{}` adds no Scenarios",
                path.display(),
                requirement.id
            ));
            continue;
        }
        let scenarios: Vec<AddedScenario> = requirement
            .scenarios
            .iter()
            .map(|scenario| AddedScenario {
                id: scenario.id.clone(),
                statement: scenario.statement.clone(),
            })
            .collect();
        let applied = model.specs.iter().any(|candidate| {
            candidate.id == parsed.id
                && candidate.requirements.iter().any(|existing| {
                    existing.id == requirement.id
                        && existing.realization == requirement.realization
                        && existing.verification == requirement.verification
                        && existing.statement == requirement.statement
                        && existing.scenarios.len() == scenarios.len()
                        && scenarios
                            .iter()
                            .zip(&existing.scenarios)
                            .all(|(scenario, item)| {
                                item.id == scenario.id && item.statement == scenario.statement
                            })
                })
        });
        additions.push(Addition {
            spec: parsed.id.clone(),
            requirement: requirement.id,
            realization: requirement.realization,
            verification: requirement.verification,
            statement: requirement.statement,
            scenarios,
            applied,
        });
    }
}

pub fn completion_issues(root: &Path, report: &Report) -> Vec<String> {
    let mut issues = Vec::new();
    let retained = root.join("retained/manifest.json");
    if retained.exists() {
        if let Err(error) = crate::retained_inputs::verify_change(root) {
            issues.push(error);
        }
    }
    let workflow_finalization = fs::read_to_string(crate::provenance::default_instance(root))
        .ok()
        .and_then(|source| crate::json::parse(&source).ok())
        .and_then(|value| value.get("definition").cloned())
        .and_then(|definition| {
            definition
                .get("tasks")
                .and_then(Json::as_array)
                .map(|tasks| tasks.to_vec())
        })
        .is_some_and(|tasks| {
            tasks
                .iter()
                .any(|task| task.get("kind").and_then(Json::as_str) == Some("finalize-change"))
        });
    if !retained.exists() && workflow_finalization {
        issues.push("retained/manifest.json is required for workflow finalization; capture the relied-on input closure".into());
    }
    if report.additions.is_empty()
        && report.mutations == 0
        && report.unchanged_intent_reason.is_none()
    {
        issues.push("change declares no supported intent delta".into());
    }
    if report.incomplete_plan_items > 0 {
        issues.push(format!(
            "{} plan item(s) remain incomplete",
            report.incomplete_plan_items
        ));
    }
    for addition in &report.additions {
        if !addition.applied {
            issues.push(format!(
                "{}#{} has not been applied to current specs",
                addition.spec, addition.requirement
            ));
        }
    }
    if report.unapplied_mutations > 0 {
        issues.push(format!(
            "{} Requirement or Scenario mutation(s) have not been applied to current specs",
            report.unapplied_mutations
        ));
    }
    let proposal = fs::read_to_string(root.join("proposal.md")).unwrap_or_default();
    if !proposal.lines().any(|line| {
        line.trim()
            .eq_ignore_ascii_case("Status: accepted and complete")
            || line
                .trim()
                .eq_ignore_ascii_case("Status: **accepted and complete**")
    }) {
        issues.push("proposal status is not `accepted and complete`".into());
    }
    let outcome = fs::read_to_string(root.join("outcome.md")).unwrap_or_default();
    if outcome.is_empty() {
        issues.push("outcome.md is missing".into());
    } else {
        for heading in ["## Departures", "## Residual decisions"] {
            if !outcome.lines().any(|line| line.trim() == heading) {
                issues.push(format!("outcome.md is missing `{heading}`"));
            }
        }
        let complete = outcome
            .lines()
            .any(|line| line.trim() == "Status: complete");
        let historically_accepted = outcome
            .lines()
            .any(|line| line.trim() == "Status: accepted");
        if !complete && !historically_accepted {
            issues.push("outcome status is not `complete`".into());
        }
        if complete || workflow_finalization {
            if let Err(error) = verify_change_acceptance(root, &report.authority_model_fingerprint)
            {
                issues.push(error);
            }
        }
    }
    issues
}

/// A recorded human decision that accepts reviewed Requirements without a locally authored
/// Requirement Status. The record preserves who decided, when and on what basis; it supplies no
/// review, Run or Status, and Assurance State stays the computed account of evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct Acceptance {
    pub change: String,
    pub accepted_by: String,
    pub accepted_on: String,
    pub basis: String,
    pub requirements: Vec<String>,
}

const ACCEPTANCE_FILE: &str = "acceptance.json";
const ACCEPTANCE_KEYS: [&str; 6] = [
    "format",
    "version",
    "accepted_by",
    "accepted_on",
    "requirements",
    "basis",
];

fn plain_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

pub fn read_acceptance(change_root: &Path) -> Result<Option<Acceptance>, Vec<String>> {
    let path = change_root.join(ACCEPTANCE_FILE);
    let display = path.display().to_string();
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(vec![format!("{display}: {error}")]),
    };
    let fail = |detail: &str| vec![format!("{display}: {detail}")];
    let value = crate::json::parse(&content).map_err(|error| fail(&error.to_string()))?;
    let Json::Obj(pairs) = &value else {
        return Err(fail("acceptance record must be a JSON object"));
    };
    if let Some((key, _)) = pairs
        .iter()
        .find(|(key, _)| !ACCEPTANCE_KEYS.contains(&key.as_str()))
    {
        return Err(fail(&format!("unknown acceptance field `{key}`")));
    }
    if value.get("format").and_then(Json::as_str) != Some("azimuth-change-acceptance")
        || value.get("version").and_then(Json::as_num) != Some(1.0)
    {
        return Err(fail(
            "expected format `azimuth-change-acceptance` and version 1",
        ));
    }
    let text = |key: &str| -> Result<String, Vec<String>> {
        match value.get(key).and_then(Json::as_str) {
            Some(text) if !text.trim().is_empty() => Ok(text.to_string()),
            _ => Err(fail(&format!("`{key}` requires a non-empty string"))),
        }
    };
    let accepted_by = text("accepted_by")?;
    if !accepted_by
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(fail(
            "`accepted_by` must be a plain identifier of letters, digits, `-`, `_` or `.`",
        ));
    }
    let accepted_on = text("accepted_on")?;
    if !plain_date(&accepted_on) {
        return Err(fail("`accepted_on` must be YYYY-MM-DD"));
    }
    let basis = text("basis")?;
    let listed = value
        .get("requirements")
        .and_then(Json::as_array)
        .ok_or_else(|| fail("`requirements` requires an array"))?;
    let mut requirements = Vec::new();
    for item in listed {
        let id = item
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or_else(|| fail("`requirements` entries must be non-empty strings"))?;
        if requirements.iter().any(|seen| seen == id) {
            return Err(fail(&format!("duplicate accepted Requirement `{id}`")));
        }
        requirements.push(id.to_string());
    }
    if requirements.is_empty() {
        return Err(fail("`requirements` must name at least one Requirement"));
    }
    let change = change_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(Some(Acceptance {
        change,
        accepted_by,
        accepted_on,
        basis,
        requirements,
    }))
}

fn change_roots(changes_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    for directory in [changes_dir.to_path_buf(), changes_dir.join("archive")] {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.path().is_dir() && entry.file_name() != "archive" {
                roots.push(entry.path());
            }
        }
    }
    roots.sort();
    roots
}

/// Reads the acceptance records of active and archived changes under one `changes/` directory.
pub fn acceptances(changes_dir: &Path) -> Result<Vec<Acceptance>, Vec<String>> {
    let mut found = Vec::new();
    let mut errors = Vec::new();
    for root in change_roots(changes_dir) {
        match read_acceptance(&root) {
            Ok(Some(acceptance)) => {
                let scoped = (|| -> Result<(), Vec<String>> {
                    let mut allowed = std::collections::BTreeSet::new();
                    for path in crate::intent::delta_paths(&root)? {
                        let source = fs::read_to_string(&path)
                            .map_err(|e| vec![format!("{}: {e}", path.display())])?;
                        let mut fence = false;
                        for line in source.lines() {
                            if line.trim_start().starts_with("```") {
                                fence = !fence;
                                continue;
                            }
                            if !fence {
                                if let Some(id) = line.strip_prefix("## Add requirement: ") {
                                    allowed.insert(id.trim().to_string());
                                }
                            }
                        }
                    }
                    if let Some(correction) = read_correction(&root)? {
                        allowed.extend(correction.requirements);
                    }
                    for requirement in &acceptance.requirements {
                        if !allowed.contains(requirement) {
                            return Err(vec![format!(
                                "{}/acceptance.json accepts {requirement} outside its added or verified correction scope",
                                root.display()
                            )]);
                        }
                    }
                    Ok(())
                })();
                match scoped {
                    Ok(()) => found.push(acceptance),
                    Err(mut diagnostics) => errors.append(&mut diagnostics),
                }
            }
            Ok(None) => {}
            Err(mut issues) => errors.append(&mut issues),
        }
    }
    if errors.is_empty() {
        Ok(found)
    } else {
        Err(errors)
    }
}

fn acceptance_fingerprint(value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err("acceptance basis requires a SHA-256 fingerprint".into());
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("acceptance basis requires a lowercase SHA-256 fingerprint".into());
    }
    Ok(())
}

pub fn verify_change_acceptance(
    root: &Path,
    candidate_model_fingerprint: &str,
) -> Result<Json, String> {
    acceptance_fingerprint(candidate_model_fingerprint)?;
    let path = root.join("change-acceptance.json");
    let metadata = fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("change-acceptance.json must be a regular retained file".into());
    }
    let value = crate::provenance::read_json(&path)?;
    correction_object(
        &value,
        &[
            "format",
            "version",
            "actor",
            "decision",
            "reviewed_at_ms",
            "candidate_model_fingerprint",
            "outcome_fingerprint",
            "retained_inputs_fingerprint",
            "report",
        ],
    )?;
    if correction_text(&value, "format")? != "azimuth-change-acceptance-decision"
        || value.get("version").and_then(Json::as_num) != Some(1.0)
    {
        return Err("expected azimuth-change-acceptance-decision version 1".into());
    }
    correction_text(&value, "actor")?;
    correction_text(&value, "report")?;
    if correction_text(&value, "decision")? != "approved" {
        return Err("change acceptance is not an actual approved decision".into());
    }
    let time = value
        .get("reviewed_at_ms")
        .and_then(Json::as_num)
        .filter(|time| *time > 0.0 && time.fract() == 0.0 && *time <= 9_007_199_254_740_991.0)
        .ok_or("reviewed_at_ms requires an actual positive safe UTC millisecond time")?;
    let _ = time;
    if correction_text(&value, "candidate_model_fingerprint")? != candidate_model_fingerprint {
        return Err("change acceptance is stale for the current candidate authority".into());
    }
    let outcome = fs::read(root.join("outcome.md")).map_err(|e| format!("outcome.md: {e}"))?;
    let outcome_fingerprint = format!("sha256:{}", crate::fingerprint::sha256(&outcome));
    if correction_text(&value, "outcome_fingerprint")? != outcome_fingerprint {
        return Err("change acceptance is stale for the authored outcome".into());
    }
    let retained = crate::retained_inputs::verify_change(root)?;
    if value.get("retained_inputs_fingerprint") != retained.get("fingerprint") {
        return Err("change acceptance is stale for the retained evidence/input closure".into());
    }
    Ok(value)
}

pub fn change_acceptance_record(
    root: &Path,
    candidate_model_fingerprint: &str,
    actor: &str,
    decision: &str,
    reviewed_at_ms: u64,
    report: &str,
) -> Result<Json, String> {
    acceptance_fingerprint(candidate_model_fingerprint)?;
    if actor.trim().is_empty()
        || report.trim().is_empty()
        || !["approved", "denied", "deferred"].contains(&decision)
        || reviewed_at_ms == 0
        || reviewed_at_ms > 9_007_199_254_740_991
    {
        return Err("change acceptance requires the actual actor, decision, positive UTC time and substantive report".into());
    }
    let outcome = fs::read(root.join("outcome.md")).map_err(|e| format!("outcome.md: {e}"))?;
    let retained = crate::retained_inputs::verify_change(root)?;
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-change-acceptance-decision")),
        ("version", Json::Num(1.0)),
        ("actor", Json::str(actor)),
        ("decision", Json::str(decision)),
        ("reviewed_at_ms", Json::Num(reviewed_at_ms as f64)),
        (
            "candidate_model_fingerprint",
            Json::str(candidate_model_fingerprint),
        ),
        (
            "outcome_fingerprint",
            Json::str(format!("sha256:{}", crate::fingerprint::sha256(&outcome))),
        ),
        (
            "retained_inputs_fingerprint",
            retained.get("fingerprint").unwrap().clone(),
        ),
        ("report", Json::str(report)),
    ]))
}

#[derive(Debug, Clone)]
pub struct Correction {
    pub source_path: String,
    pub source_fingerprint: String,
    pub requirements: Vec<String>,
    pub failed_revision: Option<String>,
    pub issues: Vec<String>,
}

fn correction_object(value: &Json, keys: &[&str]) -> Result<(), String> {
    let Json::Obj(fields) = value else {
        return Err("expected an object".into());
    };
    if fields.len() != keys.len()
        || fields.iter().any(|(key, _)| !keys.contains(&key.as_str()))
        || keys.iter().any(|key| value.get(key).is_none())
    {
        return Err(format!(
            "expected exact correction fields: {}",
            keys.join(", ")
        ));
    }
    Ok(())
}
fn correction_text<'a>(value: &'a Json, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Json::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("{key} requires a nonempty string"))
}
fn correction_ids(value: &Json, key: &str, required: bool) -> Result<Vec<String>, String> {
    let values = value
        .get(key)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{key} requires an array"))?;
    let mut result = std::collections::BTreeSet::new();
    for value in values {
        let text = value
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| format!("{key} requires nonempty strings"))?;
        if key == "requirements" {
            crate::diag::validate_id(text, false)?;
        }
        if !result.insert(text.to_string()) {
            return Err(format!("duplicate correction {key}: {text}"));
        }
    }
    if required && result.is_empty() {
        return Err(format!("{key} must not be empty"));
    }
    Ok(result.into_iter().collect())
}
fn correction_source(root: &Path, relative: &str) -> Result<std::path::PathBuf, String> {
    use std::path::Component;
    let parts = Path::new(relative).components().collect::<Vec<_>>();
    if relative.contains(['\\', '\0'])
        || parts.len() != 2
        || parts.iter().any(|c| !matches!(c, Component::Normal(_)))
        || parts[0].as_os_str() != "archive"
    {
        return Err(
            "correction source requires archive/<dated-change-id> below the same changes directory"
                .into(),
        );
    }
    let parent = root
        .parent()
        .ok_or("correction change has no changes directory")?;
    let changes = if parent.file_name().is_some_and(|name| name == "archive") {
        parent.parent().ok_or("archive has no changes directory")?
    } else {
        parent
    };
    let mut path = changes.canonicalize().map_err(|e| e.to_string())?;
    for part in parts {
        path.push(part.as_os_str());
        let m = fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if m.file_type().is_symlink() || !m.is_dir() {
            return Err(format!(
                "correction source is not a normal directory: {}",
                path.display()
            ));
        }
    }
    if !path.join("proposal.md").is_file() || !path.join("finalization.json").is_file() {
        return Err("correction source must be a finalized archived change".into());
    }
    Ok(path)
}

pub fn archive_fingerprint(root: &Path) -> Result<String, String> {
    fn visit(root: &Path, relative: &Path, files: &mut Vec<Json>) -> Result<(), String> {
        let directory = root.join(relative);
        let mut entries = fs::read_dir(&directory)
            .map_err(|e| format!("{}: {e}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if entry.file_name() == ".azimuth" {
                continue;
            }
            let path = relative.join(entry.file_name());
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                return Err(format!(
                    "symlink in archived correction basis: {}",
                    root.join(&path).display()
                ));
            }
            if kind.is_dir() {
                visit(root, &path, files)?;
            } else if kind.is_file() {
                let bytes = fs::read(root.join(&path)).map_err(|e| e.to_string())?;
                files.push(Json::obj(vec![
                    (
                        "path",
                        Json::str(path.to_str().ok_or("archive path must be UTF-8")?),
                    ),
                    (
                        "fingerprint",
                        Json::str(format!("sha256:{}", crate::fingerprint::sha256(&bytes))),
                    ),
                ]));
            } else {
                return Err(format!(
                    "non-regular archived input: {}",
                    root.join(&path).display()
                ));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, Path::new(""), &mut files)?;
    Ok(crate::fingerprint::canonical_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-archived-change-basis")),
        ("version", Json::Num(1.0)),
        ("files", Json::Arr(files)),
    ])))
}
fn archived_requirement_scope(root: &Path) -> Result<std::collections::BTreeSet<String>, String> {
    let mut result = std::collections::BTreeSet::new();
    for path in crate::intent::delta_paths(root).map_err(|e| e.join("\n"))? {
        let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut fence = false;
        for line in source.lines() {
            if line.trim_start().starts_with("```") {
                fence = !fence;
                continue;
            }
            if fence {
                continue;
            }
            for prefix in [
                "## Add requirement: ",
                "## Replace requirement: ",
                "## Remove requirement: ",
            ] {
                if let Some(id) = line.strip_prefix(prefix) {
                    crate::diag::validate_id(id.trim(), false)?;
                    result.insert(id.trim().into());
                }
            }
        }
    }
    for review in read_reviews(root).map_err(|e| e.join("\n"))? {
        if let Some(id) = review.get("requirement").and_then(Json::as_str) {
            result.insert(id.into());
        }
    }
    if let Some(acceptance) = read_acceptance(root).map_err(|e| e.join("\n"))? {
        result.extend(acceptance.requirements);
    }
    Ok(result)
}
fn correction_from_json(root: &Path, value: &Json) -> Result<Correction, String> {
    correction_object(
        value,
        &[
            "format",
            "version",
            "source",
            "requirements",
            "failed_revision",
            "issues",
        ],
    )?;
    if correction_text(value, "format")? != "azimuth-change-correction"
        || value.get("version").and_then(Json::as_num) != Some(1.0)
    {
        return Err("expected azimuth-change-correction version 1".into());
    }
    let source = value.get("source").ok_or("missing correction source")?;
    correction_object(source, &["path", "fingerprint"])?;
    let source_path = correction_text(source, "path")?;
    let source_root = correction_source(root, source_path)?;
    let source_fingerprint = correction_text(source, "fingerprint")?;
    if source_fingerprint != archive_fingerprint(&source_root)? {
        return Err("archived correction source differs from its pinned durable content".into());
    }
    let requirements = correction_ids(value, "requirements", true)?;
    let scope = archived_requirement_scope(&source_root)?;
    for requirement in &requirements {
        if !scope.contains(requirement) {
            return Err(format!(
                "correction Requirement {requirement} is outside the archived change's declared/reviewed scope"
            ));
        }
    }
    let failed_revision = match value.get("failed_revision") {
        Some(Json::Null) => None,
        Some(Json::Str(s))
            if matches!(s.len(), 40 | 64)
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) =>
        {
            Some(s.clone())
        }
        _ => return Err("failed_revision requires null or a full lowercase Git revision".into()),
    };
    Ok(Correction {
        source_path: source_path.into(),
        source_fingerprint: source_fingerprint.into(),
        requirements,
        failed_revision,
        issues: correction_ids(value, "issues", false)?,
    })
}

pub fn read_correction(root: &Path) -> Result<Option<Correction>, Vec<String>> {
    let path = root.join("correction.json");
    if !path.exists() {
        return Ok(None);
    }
    let value = crate::provenance::read_json(&path).map_err(|e| vec![e])?;
    correction_from_json(root, &value)
        .map(Some)
        .map_err(|e| vec![format!("{}: {e}", path.display())])
}

pub fn correction_record(
    root: &Path,
    source: &Path,
    requirements: &[String],
    failed_revision: Option<&str>,
    issues: &[String],
) -> Result<Json, String> {
    let changes = root
        .parent()
        .ok_or("correction change has no parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let source = source
        .canonicalize()
        .map_err(|e| format!("{}: {e}", source.display()))?;
    let relative = source
        .strip_prefix(&changes)
        .map_err(|_| "correction source must be below this changes directory")?
        .to_str()
        .ok_or("source path must be UTF-8")?;
    let value = Json::obj(vec![
        ("format", Json::str("azimuth-change-correction")),
        ("version", Json::Num(1.0)),
        (
            "source",
            Json::obj(vec![
                ("path", Json::str(relative)),
                ("fingerprint", Json::str(archive_fingerprint(&source)?)),
            ]),
        ),
        (
            "requirements",
            Json::Arr(requirements.iter().map(Json::str).collect()),
        ),
        (
            "failed_revision",
            failed_revision.map(Json::str).unwrap_or(Json::Null),
        ),
        ("issues", Json::Arr(issues.iter().map(Json::str).collect())),
    ]);
    correction_from_json(root, &value)?;
    Ok(value)
}

pub fn write_correction(
    root: &Path,
    source: &Path,
    requirements: &[String],
    failed_revision: Option<&str>,
    issues: &[String],
) -> Result<Json, String> {
    if root
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name == "archive")
    {
        return Err("an archived correction record is immutable".into());
    }
    let value = correction_record(root, source, requirements, failed_revision, issues)?;
    let path = root.join("correction.json");
    if path.exists() {
        let previous = crate::provenance::read_json(&path)?;
        return if previous == value {
            Ok(previous)
        } else {
            Err("correction.json already records a different immutable source/scope".into())
        };
    }
    use std::io::Write;
    let temporary = root.join(format!(
        ".correction-{}-{}.tmp",
        std::process::id(),
        crate::provenance::now_ms()
    ));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| format!("{}: {e}", temporary.display()))?;
        file.write_all(value.to_string_pretty().as_bytes())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        fs::hard_link(&temporary, &path).map_err(|e| {
            format!(
                "cannot publish immutable correction record {}: {e}",
                path.display()
            )
        })?;
        fs::File::open(root)
            .and_then(|directory| directory.sync_all())
            .map_err(|e| e.to_string())?;
        Ok(value)
    })();
    let _ = fs::remove_file(&temporary);
    result
}

const REVIEWS_DIRECTORY: &str = "reviews";
const RATIFICATION_PROJECT: &str = "ratification";

/// Reads the sealed native review records committed under `reviews/` of one change.
pub fn read_reviews(change_root: &Path) -> Result<Vec<Json>, Vec<String>> {
    let directory = change_root.join(REVIEWS_DIRECTORY);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(vec![format!("{}: {error}", directory.display())]),
    };
    let mut paths = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    let mut records = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        let display = path.display().to_string();
        let loaded = fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|content| crate::json::parse(&content).map_err(|error| error.to_string()))
            .and_then(|value| crate::review_authoring::records(&value));
        match loaded {
            Ok(mut found) => records.append(&mut found),
            Err(error) => errors.push(format!("{display}: {error}")),
        }
    }
    if errors.is_empty() {
        Ok(records)
    } else {
        Err(errors)
    }
}

/// Reads the committed review records of the active changes under one `changes/` directory.
/// An archived change's reviews are history: the model's review heads carry what still ratifies.
pub fn reviews(changes_dir: &Path) -> Result<Vec<Json>, Vec<String>> {
    let mut found = Vec::new();
    let mut errors = Vec::new();
    for root in change_roots(changes_dir) {
        if root
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            == Some("archive")
        {
            continue;
        }
        match read_reviews(&root) {
            Ok(mut records) => found.append(&mut records),
            Err(mut issues) => errors.append(&mut issues),
        }
    }
    if errors.is_empty() {
        Ok(found)
    } else {
        Err(errors)
    }
}

/// An accepted Verification or Realization Review that is still current against the merged model.
#[derive(Debug, Clone, PartialEq)]
pub struct Ratification {
    pub requirement: String,
    pub kind: crate::review_heads::HeadKind,
    pub review: String,
    pub record: Json,
}

/// Assesses the latest review of each declared kind for every Requirement against the merged
/// model, so a review or head that went stale in a rebase or merge cannot ratify. Heads stored in
/// the model compete with the change's own sealed reviews on review time.
pub fn ratifications(
    model: &Model,
    reviews: &[Json],
    bundles: &[crate::run::RunBundle],
) -> Result<Vec<Ratification>, String> {
    let mut all = reviews.to_vec();
    all.extend(model.review_heads.iter().map(|head| head.record.clone()));
    let authority = crate::assurance_review::authority(model, RATIFICATION_PROJECT)?;
    let mut ratified = Vec::new();
    for view in model.requirements() {
        let declared = [
            (
                crate::review_heads::HeadKind::Verification,
                view.requirement.verification.is_some(),
            ),
            (
                crate::review_heads::HeadKind::Realization,
                view.requirement.realization.is_some(),
            ),
        ];
        for (kind, declares) in declared {
            if !declares {
                continue;
            }
            let target = Json::obj(vec![
                ("kind", Json::str(kind.record_kind())),
                ("requirement", Json::str(view.requirement.id.clone())),
            ]);
            let Some(review) = crate::assurance_review::latest_review(&all, &target)? else {
                continue;
            };
            let is_head = model.review_heads.iter().any(|head| &head.record == review);
            let assessment = if is_head {
                crate::assurance_review::assess_head(review, &authority, &all, bundles)
            } else {
                crate::assurance_review::assess_record(review, &authority, &all, bundles)
            };
            if assessment.status != "accepted" {
                continue;
            }
            let fingerprint = review
                .get("fingerprint")
                .and_then(Json::as_str)
                .ok_or("review record has no fingerprint")?;
            ratified.push(Ratification {
                requirement: view.requirement.id.clone(),
                kind,
                review: fingerprint.to_string(),
                record: review.clone(),
            });
        }
    }
    ratified.sort_by(|left, right| {
        (&left.requirement, left.kind).cmp(&(&right.requirement, right.kind))
    });
    ratified.dedup();
    Ok(ratified)
}

/// Heads the model does not yet carry, or carries with a superseded record, for the ratified reviews.
pub fn head_updates(model: &Model, ratified: &[Ratification]) -> Vec<crate::review_heads::Head> {
    ratified
        .iter()
        .filter(|item| {
            !model.review_heads.iter().any(|head| {
                head.kind == item.kind
                    && head.requirement == item.requirement
                    && head.record.get("fingerprint").and_then(Json::as_str)
                        == Some(item.review.as_str())
            })
        })
        .filter_map(|item| {
            let view = model
                .requirements()
                .find(|view| view.requirement.id == item.requirement)?;
            Some(crate::review_heads::Head {
                path: String::new(),
                owner: view.spec.id.clone(),
                kind: item.kind,
                requirement: item.requirement.clone(),
                record: item.record.clone(),
                line: 0,
            })
        })
        .collect()
}

/// An accepted Requirement whose declared review has no current accepted record blocks the change
/// that records it.
pub fn ratification_issues(
    model: &Model,
    acceptances: &[Acceptance],
    ratified: &[Ratification],
    report: &Report,
) -> Vec<String> {
    let mut issues = Vec::new();
    for acceptance in acceptances.iter().filter(|value| value.change == report.id) {
        for requirement in &acceptance.requirements {
            let Some(view) = model
                .requirements()
                .find(|view| &view.requirement.id == requirement)
            else {
                continue;
            };
            for (kind, label, declares) in [
                (
                    crate::review_heads::HeadKind::Verification,
                    "Verification",
                    view.requirement.verification.is_some(),
                ),
                (
                    crate::review_heads::HeadKind::Realization,
                    "Realization",
                    view.requirement.realization.is_some(),
                ),
            ] {
                if declares
                    && !ratified
                        .iter()
                        .any(|item| &item.requirement == requirement && item.kind == kind)
                {
                    issues.push(format!(
                        "{ACCEPTANCE_FILE} accepts `{requirement}`, which has no accepted {label} Review that is current against the merged model; commit it under {REVIEWS_DIRECTORY}/"
                    ));
                }
            }
        }
    }
    issues
}

/// A correction may accept existing Requirements only within its verified archived-source scope.
pub fn acceptance_issues(acceptances: &[Acceptance], report: &Report) -> Vec<String> {
    let mut issues = Vec::new();
    for acceptance in acceptances.iter().filter(|value| value.change == report.id) {
        for requirement in &acceptance.requirements {
            if !report
                .additions
                .iter()
                .any(|addition| &addition.requirement == requirement)
                && !report.correction_requirements.contains(requirement)
            {
                issues.push(format!(
                    "{ACCEPTANCE_FILE} accepts `{requirement}`, which this change neither adds nor names in a verified correction scope"
                ));
            }
        }
    }
    issues
}

/// Removes the missing-review findings of Requirements whose declared review is current and accepted.
pub fn waive_accepted_requirements(findings: &mut Vec<Finding>, ratified: &[Ratification]) {
    findings.retain(|finding| {
        let kind = match finding.kind {
            validation::FindingKind::MissingRequirementAssurance => {
                crate::review_heads::HeadKind::Verification
            }
            validation::FindingKind::MissingRealizationReview => {
                crate::review_heads::HeadKind::Realization
            }
            _ => return true,
        };
        !finding.requirement.as_ref().is_some_and(|requirement| {
            ratified
                .iter()
                .any(|item| &item.requirement == requirement && item.kind == kind)
        })
    });
}

pub fn finalization(
    model: &Model,
    findings: &[Finding],
    acceptances: &[Acceptance],
    ratified: &[Ratification],
) -> (String, String) {
    let fingerprint = crate::fingerprint::model_digest(model, findings);
    let summary = validation::summarize(model, findings);
    let mut pairs = vec![
        ("version", Json::Num(3.0)),
        ("model_fingerprint", Json::str(&fingerprint)),
        (
            "authority_model_fingerprint",
            Json::str(format!(
                "sha256:{}",
                crate::fingerprint::authority_model_digest(model)
            )),
        ),
        ("requirements", Json::Num(summary.requirements as f64)),
        ("specs", Json::Num(model.specs.len() as f64)),
        ("errors", Json::Num(summary.errors as f64)),
        ("warnings", Json::Num(summary.warnings as f64)),
    ];
    if !acceptances.is_empty() {
        pairs.push((
            "human_acceptances",
            Json::Arr(
                acceptances
                    .iter()
                    .map(|acceptance| {
                        Json::obj(vec![
                            ("change", Json::str(&acceptance.change)),
                            ("accepted_by", Json::str(&acceptance.accepted_by)),
                            ("accepted_on", Json::str(&acceptance.accepted_on)),
                            ("basis", Json::str(&acceptance.basis)),
                            (
                                "requirements",
                                Json::Arr(acceptance.requirements.iter().map(Json::str).collect()),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ));
    }
    if !ratified.is_empty() {
        pairs.push((
            "ratifying_reviews",
            Json::Arr(
                ratified
                    .iter()
                    .map(|item| {
                        Json::obj(vec![
                            ("requirement", Json::str(&item.requirement)),
                            ("kind", Json::str(item.kind.record_kind())),
                            ("review", Json::str(&item.review)),
                        ])
                    })
                    .collect(),
            ),
        ));
    }
    (fingerprint, Json::obj(pairs).to_string_pretty())
}

/// Writes `provenance/ledger.json` when the change was coordinated by a workflow instance. The
/// ledger is produced here rather than at archive so that archiving moves an unchanged tree, which
/// federation's change-acceptance digest compares.
pub fn finalize_provenance(root: &Path) -> Result<Option<std::path::PathBuf>, String> {
    if !crate::provenance::default_instance(root).is_file() {
        return Ok(None);
    }
    crate::provenance::write_ledger(root, &[], None, None).map(|(path, _)| Some(path))
}

/// Archive refuses a committed ledger that a fresh export would not reproduce.
pub fn provenance_archive_issues(root: &Path) -> Vec<String> {
    crate::provenance::committed_ledger_issues(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::parse_spec;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_CHANGE: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn finalization_receipt_without_acceptance_keeps_its_recorded_shape() {
        let model = Model::default();
        let (fingerprint, receipt) = finalization(&model, &[], &[], &[]);
        let expected = Json::obj(vec![
            ("version", Json::Num(3.0)),
            ("model_fingerprint", Json::str(&fingerprint)),
            (
                "authority_model_fingerprint",
                Json::str(format!(
                    "sha256:{}",
                    crate::fingerprint::authority_model_digest(&model)
                )),
            ),
            ("requirements", Json::Num(0.0)),
            ("specs", Json::Num(0.0)),
            ("errors", Json::Num(0.0)),
            ("warnings", Json::Num(0.0)),
        ])
        .to_string_pretty();
        assert_eq!(receipt, expected);

        let ratified = [Ratification {
            requirement: "r".into(),
            kind: crate::review_heads::HeadKind::Verification,
            review: "sha256:x".into(),
            record: Json::Null,
        }];
        let (_, with_review) = finalization(&model, &[], &[], &ratified);
        assert!(with_review.contains("ratifying_reviews"));
    }

    #[test]
    fn additions_project_until_the_current_facet_contains_them() {
        let root = temp_change();
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(root.join("proposal.md"), "# Change: x\n\nStatus: active\n").unwrap();
        fs::write(root.join("plan.md"), "- [ ] Apply it.\n").unwrap();
        fs::write(
            root.join("specs/alpha.md"),
            "# Intent delta: alpha\n\n## Add requirement: added\n\nText.\n\n### Add scenario: visible\nY occurs when X occurs.\n",
        )
        .unwrap();
        let current = parse_spec(
            "alpha.md",
            "# Spec: alpha\n\n## Requirement: old\n\nOld.\n\n### Scenario: existing\nY occurs when X occurs.\n",
        )
        .unwrap();
        let model = Model {
            specs: vec![current],
            ..Default::default()
        };
        let report = inspect(&root, &model).unwrap();
        assert_eq!(report.current_requirements, 1);
        assert_eq!(report.target_requirements, 2);
        assert!(!report.additions[0].applied);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_applied_addition_requires_the_same_requirement_and_scenario_statements() {
        let root = temp_change();
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(root.join("proposal.md"), "# Change: x\n\nStatus: active\n").unwrap();
        fs::write(root.join("plan.md"), "- [ ] Apply it.\n").unwrap();
        fs::write(
            root.join("specs/alpha.md"),
            "# Intent delta: alpha\n\n## Add requirement: added\n\nTarget statement.\n\n### Add scenario: visible\nThe target behavior occurs.\n",
        )
        .unwrap();
        let current = parse_spec(
            "alpha.md",
            "# Spec: alpha\n\n## Requirement: added\n\nDifferent statement.\n\n### Scenario: visible\nDifferent behavior occurs.\n",
        )
        .unwrap();
        let model = Model {
            specs: vec![current],
            ..Default::default()
        };

        let report = inspect(&root, &model).unwrap();

        assert!(!report.additions[0].applied);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completion_requires_an_intent_delta() {
        let root = temp_change();
        fs::write(
            root.join("proposal.md"),
            "# Change: x\n\nStatus: accepted and complete\n",
        )
        .unwrap();
        fs::write(root.join("plan.md"), "- [x] Complete.\n").unwrap();
        fs::write(
            root.join("outcome.md"),
            "# Outcome: x\n\nStatus: accepted\n\n## Departures\n\nNone.\n\n## Residual decisions\n\nNone.\n",
        )
        .unwrap();
        let report = inspect(&root, &Model::default()).unwrap();

        assert_eq!(
            completion_issues(&root, &report),
            vec!["change declares no supported intent delta"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_explicit_unchanged_intent_reason_satisfies_only_the_delta_gate() {
        let root = temp_change();
        fs::write(
            root.join("proposal.md"),
            "# Change: x\n\nStatus: proposed\n\nIntent delta: none\nBecause: the parser contract changes without changing accepted intent\n\n## Problem\n",
        )
        .unwrap();
        fs::write(root.join("plan.md"), "- [ ] Complete.\n").unwrap();
        let report = inspect(&root, &Model::default()).unwrap();

        let issues = completion_issues(&root, &report);

        assert_eq!(
            report.unchanged_intent_reason.as_deref(),
            Some("the parser contract changes without changing accepted intent")
        );
        assert!(!issues
            .iter()
            .any(|issue| issue == "change declares no supported intent delta"));
        assert!(issues
            .iter()
            .any(|issue| issue == "1 plan item(s) remain incomplete"));
        assert!(issues
            .iter()
            .any(|issue| issue == "proposal status is not `accepted and complete`"));
        assert!(issues.iter().any(|issue| issue == "outcome.md is missing"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_unchanged_intent_declaration_requires_an_adjacent_reason() {
        let root = temp_change();
        fs::write(
            root.join("proposal.md"),
            "# Change: x\n\nStatus: proposed\n\nIntent delta: none\n\nBecause: separated\n\n## Problem\n",
        )
        .unwrap();
        fs::write(root.join("plan.md"), "- [ ] Complete.\n").unwrap();

        let errors = inspect(&root, &Model::default()).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.contains("requires an adjacent non-empty `Because:`")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unchanged_intent_cannot_be_declared_twice() {
        let root = temp_change();
        fs::write(
            root.join("proposal.md"),
            "# Change: x\n\nStatus: proposed\n\nIntent delta: none\nBecause: first\nIntent delta: none\nBecause: second\n\n## Problem\n",
        )
        .unwrap();
        fs::write(root.join("plan.md"), "- [ ] Complete.\n").unwrap();

        let errors = inspect(&root, &Model::default()).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.contains("declares `Intent delta:` more than once")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unchanged_intent_contradicts_a_supported_delta() {
        let root = temp_change();
        fs::write(
            root.join("proposal.md"),
            "# Change: x\n\nStatus: proposed\n\nIntent delta: none\nBecause: no accepted intent changes\n\n## Problem\n",
        )
        .unwrap();
        fs::write(root.join("plan.md"), "- [ ] Complete.\n").unwrap();
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(
            root.join("specs/alpha.md"),
            "# Intent delta: alpha\n\n## Add requirement: added\n\nText.\n\n### Add scenario: visible\nY occurs when X occurs.\n",
        )
        .unwrap();

        let errors = inspect(&root, &Model::default()).unwrap_err();

        assert!(errors.iter().any(|error| error.contains(
            "proposal declares unchanged intent but specs contain a supported intent delta"
        )));
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_change() -> PathBuf {
        let nonce = NEXT_CHANGE.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("azimuth-change-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationFacet {
    Structure,
    Evidence,
    Readiness,
}
impl ValidationFacet {
    pub fn name(self) -> &'static str {
        match self {
            Self::Structure => "structure",
            Self::Evidence => "evidence",
            Self::Readiness => "readiness",
        }
    }
}

pub fn validation_facet(kind: validation::FindingKind) -> ValidationFacet {
    use validation::FindingKind::*;
    match kind {
        MissingRequirementAssurance | MissingRealizationReview | UnknownReviewHead => {
            ValidationFacet::Readiness
        }
        Unrealized
        | UnresolvedRealizationBinding
        | MissingRequiredRealization
        | UnimplementedCheck
        | VerificationElementSupportUnresolved
        | ScenarioCheckContributionUnresolved
        | MechanismSupportUnresolved => ValidationFacet::Evidence,
        _ => ValidationFacet::Structure,
    }
}
