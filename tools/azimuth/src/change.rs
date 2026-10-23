//! The change lifecycle projects reviewed intent without treating unfinished behavior as accepted.
//!
//! The machine derives additions and their obligations, and verifies that an accepted change was
//! applied before archiving. Explanations of departures and residual acceptance remain authored
//! because deriving them would manufacture the judgment the archive exists to preserve.

use crate::json::Json;
use crate::labels::read_block;
use crate::model::{Criticality, Model};
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
    pub criticality: Criticality,
    pub statement: String,
    pub scenarios: Vec<AddedScenario>,
    pub applied: bool,
}

#[derive(Debug, Clone)]
pub struct AddedScenario {
    pub id: String,
    pub statement: String,
}

#[derive(Debug, Clone)]
pub struct CriticalityChange {
    pub spec: String,
    pub requirement: String,
    pub from: Criticality,
    pub to: Criticality,
    pub because: String,
    pub revisit: Option<String>,
    pub applied: bool,
}

#[derive(Debug)]
pub struct Report {
    pub id: String,
    pub additions: Vec<Addition>,
    pub criticality_changes: Vec<CriticalityChange>,
    pub mutations: usize,
    pub unapplied_mutations: usize,
    pub unchanged_intent_reason: Option<String>,
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
    let mut criticality_changes = Vec::new();
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
                        &mut criticality_changes,
                        &mut mutations,
                        &mut errors,
                    ),
                    Err(error) => errors.push(format!("cannot read {}: {error}", file.display())),
                }
            }
        }
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }

    if unchanged_intent_reason.is_some()
        && (!additions.is_empty() || !criticality_changes.is_empty() || mutations > 0)
    {
        errors.push(
            "proposal declares unchanged intent but specs contain a supported intent delta".into(),
        );
    }

    let unapplied_mutations = if mutations > 0 && errors.is_empty() {
        match crate::intent::project(root, Path::new("azimuth/model"), model) {
            Ok(projected) => projected
                .operations
                .iter()
                .filter(|(name, applied)| {
                    !*applied && !name.starts_with("add ") && !name.starts_with("criticality ")
                })
                .count(),
            Err(projection_errors) => {
                errors.extend(projection_errors);
                0
            }
        }
    } else {
        0
    };
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
        criticality_changes,
        mutations,
        unapplied_mutations,
        unchanged_intent_reason,
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
    criticality_changes: &mut Vec<CriticalityChange>,
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
        } else if let Some(requirement) = trimmed.strip_prefix("## Change criticality:") {
            let (block, next) =
                read_block(&lines, index + 1, &["From", "To", "Because", "Revisit"]);
            parse_criticality_change(
                path,
                index + 1,
                spec,
                requirement.trim(),
                &block,
                model,
                criticality_changes,
                errors,
            );
            index = next;
            continue;
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
                "{}:{}: unsupported delta operation `{trimmed}`; additions and criticality changes are machine-projected",
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
        let Some(criticality) = requirement.criticality else {
            errors.push(format!(
                "{}: `{}` declares no criticality",
                path.display(),
                requirement.id
            ));
            continue;
        };
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
                        && existing.criticality == Some(criticality)
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
            criticality,
            statement: requirement.statement,
            scenarios,
            applied,
        });
    }
}

fn parse_criticality_change(
    path: &Path,
    line: usize,
    spec: &str,
    requirement: &str,
    block: &crate::labels::Block,
    model: &Model,
    changes: &mut Vec<CriticalityChange>,
    errors: &mut Vec<String>,
) {
    for duplicate in block.duplicates() {
        errors.push(format!(
            "{}:{}: `{}` is declared twice",
            path.display(),
            duplicate.line,
            duplicate.key
        ));
    }
    for (text, source_line) in &block.stray {
        errors.push(format!(
            "{}:{}: unrecognized line `{text}`; expected From, To, Because or Revisit",
            path.display(),
            source_line
        ));
    }
    if !block.prose.is_empty() {
        errors.push(format!(
            "{}:{line}: criticality rationale belongs in `Because:` and the lowering condition in `Revisit:`",
            path.display()
        ));
    }

    let parse_level = |label: &str, errors: &mut Vec<String>| -> Option<Criticality> {
        let Some(value) = block.value(label) else {
            errors.push(format!(
                "{}:{line}: criticality change is missing `{label}:`",
                path.display()
            ));
            return None;
        };
        match Criticality::parse(value) {
            Some(level) => Some(level),
            None => {
                errors.push(format!(
                    "{}:{line}: unknown criticality `{value}` in `{label}:`",
                    path.display()
                ));
                None
            }
        }
    };
    let from = parse_level("From", errors);
    let to = parse_level("To", errors);
    let because = block.value("Because").unwrap_or_default().trim();
    if because.is_empty() {
        errors.push(format!(
            "{}:{line}: criticality change is missing `Because:`",
            path.display()
        ));
    }
    let (Some(from), Some(to)) = (from, to) else {
        return;
    };
    if from == to {
        errors.push(format!(
            "{}:{line}: criticality change keeps `{requirement}` at `{}`",
            path.display(),
            from.name()
        ));
    }
    let revisit = block
        .value("Revisit")
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if to < from && revisit.is_none() {
        errors.push(format!(
            "{}:{line}: lowering `{requirement}` requires `Revisit:` with the condition that raises it again",
            path.display()
        ));
    }
    if changes
        .iter()
        .any(|change| change.spec == spec && change.requirement == requirement)
    {
        errors.push(format!(
            "{}:{line}: criticality for `{spec}#{requirement}` changes more than once",
            path.display()
        ));
        return;
    }

    let current = model
        .specs
        .iter()
        .find(|candidate| candidate.id == spec)
        .and_then(|candidate| {
            candidate
                .requirements
                .iter()
                .find(|candidate| candidate.id == requirement)
        });
    let Some(current) = current else {
        errors.push(format!(
            "{}:{line}: `{spec}#{requirement}` does not exist in current intent",
            path.display()
        ));
        return;
    };
    let Some(current_level) = current.criticality else {
        errors.push(format!(
            "{}:{line}: `{spec}#{requirement}` is unclassified and cannot match `From:`",
            path.display()
        ));
        return;
    };
    if current_level != from && current_level != to {
        errors.push(format!(
            "{}:{line}: `{spec}#{requirement}` is `{}`, not declared `From: {}` or `To: {}`",
            path.display(),
            current_level.name(),
            from.name(),
            to.name()
        ));
        return;
    }
    if from != to && !because.is_empty() {
        changes.push(CriticalityChange {
            spec: spec.to_string(),
            requirement: requirement.to_string(),
            from,
            to,
            because: because.to_string(),
            revisit: revisit.map(str::to_string),
            applied: current_level == to,
        });
    }
}

pub fn completion_issues(root: &Path, report: &Report) -> Vec<String> {
    let mut issues = Vec::new();
    if report.additions.is_empty()
        && report.criticality_changes.is_empty()
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
    for change in &report.criticality_changes {
        if !change.applied {
            issues.push(format!(
                "{}#{} criticality has not changed from {} to {}",
                change.spec,
                change.requirement,
                change.from.name(),
                change.to.name()
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
        if !outcome
            .lines()
            .any(|line| line.trim() == "Status: accepted")
        {
            issues.push("outcome status is not `accepted`".into());
        }
    }
    issues
}

/// A recorded human decision that accepts non-routine Requirements without a locally authored
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
            Ok(Some(acceptance)) => found.push(acceptance),
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
        .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
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

/// Reads the committed review records of active and archived changes under one `changes/` directory.
pub fn reviews(changes_dir: &Path) -> Result<Vec<Json>, Vec<String>> {
    let mut found = Vec::new();
    let mut errors = Vec::new();
    for root in change_roots(changes_dir) {
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

/// An accepted Requirement Review that is still current against the merged model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ratification {
    pub requirement: String,
    pub review: String,
}

/// Assesses the latest Requirement Review of every accepted Requirement against the merged model,
/// so a review that went stale in a rebase or merge cannot ratify.
pub fn ratifications(
    model: &Model,
    acceptances: &[Acceptance],
    reviews: &[Json],
) -> Result<Vec<Ratification>, String> {
    if acceptances.is_empty() {
        return Ok(Vec::new());
    }
    let authority = crate::assurance_review::authority(model, RATIFICATION_PROJECT)?;
    let mut ratified = Vec::new();
    for requirement in acceptances
        .iter()
        .flat_map(|acceptance| &acceptance.requirements)
    {
        let target = Json::obj(vec![
            ("kind", Json::str("requirement-review")),
            ("requirement", Json::str(requirement)),
        ]);
        let Some(review) = crate::assurance_review::latest_review(reviews, &target)? else {
            continue;
        };
        if crate::assurance_review::assess_record(review, &authority, reviews, &[]).status
            != "accepted"
        {
            continue;
        }
        let fingerprint = review
            .get("fingerprint")
            .and_then(Json::as_str)
            .ok_or("review record has no fingerprint")?;
        ratified.push(Ratification {
            requirement: requirement.clone(),
            review: fingerprint.to_string(),
        });
    }
    ratified.sort_by(|left, right| left.requirement.cmp(&right.requirement));
    ratified.dedup();
    Ok(ratified)
}

/// An accepted Requirement without a current accepted review blocks the change that records it.
pub fn ratification_issues(
    acceptances: &[Acceptance],
    ratified: &[Ratification],
    report: &Report,
) -> Vec<String> {
    let mut issues = Vec::new();
    for acceptance in acceptances.iter().filter(|value| value.change == report.id) {
        for requirement in &acceptance.requirements {
            if !ratified.iter().any(|item| &item.requirement == requirement) {
                issues.push(format!(
                    "{ACCEPTANCE_FILE} accepts `{requirement}`, which has no accepted Requirement Review that is current against the merged model; commit it under {REVIEWS_DIRECTORY}/"
                ));
            }
        }
    }
    issues
}

/// An active change may accept only Requirements it adds; archived records are immutable.
pub fn acceptance_issues(acceptances: &[Acceptance], report: &Report) -> Vec<String> {
    let mut issues = Vec::new();
    for acceptance in acceptances.iter().filter(|value| value.change == report.id) {
        for requirement in &acceptance.requirements {
            if !report
                .additions
                .iter()
                .any(|addition| &addition.requirement == requirement)
            {
                issues.push(format!(
                    "{ACCEPTANCE_FILE} accepts `{requirement}`, which this change does not add"
                ));
            }
        }
    }
    issues
}

/// Removes the missing-assurance findings of Requirements whose human acceptance is backed by a
/// current accepted Requirement Review.
pub fn waive_accepted_requirements(
    findings: &mut Vec<Finding>,
    acceptances: &[Acceptance],
    ratified: &[Ratification],
) {
    findings.retain(|finding| {
        finding.kind != validation::FindingKind::MissingRequirementAssurance
            || !finding.requirement.as_ref().is_some_and(|requirement| {
                acceptances
                    .iter()
                    .any(|acceptance| acceptance.requirements.contains(requirement))
                    && ratified.iter().any(|item| &item.requirement == requirement)
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
        ("version", Json::Num(2.0)),
        ("model_fingerprint", Json::str(&fingerprint)),
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
            ("version", Json::Num(2.0)),
            ("model_fingerprint", Json::str(&fingerprint)),
            ("requirements", Json::Num(0.0)),
            ("specs", Json::Num(0.0)),
            ("errors", Json::Num(0.0)),
            ("warnings", Json::Num(0.0)),
        ])
        .to_string_pretty();
        assert_eq!(receipt, expected);

        let ratified = [Ratification {
            requirement: "r".into(),
            review: "sha256:x".into(),
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
            "# Intent delta: alpha\n\n## Add requirement: added\nCriticality: routine\n\nText.\n\n### Add scenario: visible\nY occurs when X occurs.\n",
        )
        .unwrap();
        let current = parse_spec(
            "alpha.md",
            "# Spec: alpha\n\n## Requirement: old\nCriticality: routine\n\nOld.\n\n### Scenario: existing\nY occurs when X occurs.\n",
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
            "# Intent delta: alpha\n\n## Add requirement: added\nCriticality: routine\n\nTarget statement.\n\n### Add scenario: visible\nThe target behavior occurs.\n",
        )
        .unwrap();
        let current = parse_spec(
            "alpha.md",
            "# Spec: alpha\n\n## Requirement: added\nCriticality: routine\n\nDifferent statement.\n\n### Scenario: visible\nDifferent behavior occurs.\n",
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
            "# Intent delta: alpha\n\n## Add requirement: added\nCriticality: routine\n\nText.\n\n### Add scenario: visible\nY occurs when X occurs.\n",
        )
        .unwrap();

        let errors = inspect(&root, &Model::default()).unwrap_err();

        assert!(errors.iter().any(|error| error.contains(
            "proposal declares unchanged intent but specs contain a supported intent delta"
        )));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_criticality_raise_projects_without_changing_requirement_identity() {
        let root = transition_change(
            "## Change criticality: existing\nFrom: routine\nTo: standard\nBecause: riders now rely on this value\n",
        );
        let model = model_at(Criticality::Routine);

        let report = inspect(&root, &model).unwrap();

        assert_eq!(report.current_requirements, 1);
        assert_eq!(report.target_requirements, 1);
        assert_eq!(report.criticality_changes.len(), 1);
        assert!(!report.criticality_changes[0].applied);
        assert_eq!(report.criticality_changes[0].to, Criticality::Standard);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_criticality_transition_is_applied_only_at_the_declared_target() {
        let root = transition_change(
            "## Change criticality: existing\nFrom: routine\nTo: standard\nBecause: riders now rely on this value\n",
        );

        let report = inspect(&root, &model_at(Criticality::Standard)).unwrap();

        assert!(report.criticality_changes[0].applied);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lowering_criticality_requires_the_condition_that_raises_it_again() {
        let root = transition_change(
            "## Change criticality: existing\nFrom: critical\nTo: routine\nBecause: the value is no longer user-visible\n",
        );

        let errors = inspect(&root, &model_at(Criticality::Critical)).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.contains("requires `Revisit:`")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_stale_from_level_fails_instead_of_reinterpreting_the_transition() {
        let root = transition_change(
            "## Change criticality: existing\nFrom: routine\nTo: standard\nBecause: riders now rely on this value\n",
        );

        let errors = inspect(&root, &model_at(Criticality::Critical)).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.contains("not declared `From:")));
        fs::remove_dir_all(root).unwrap();
    }

    fn transition_change(delta: &str) -> PathBuf {
        let root = temp_change();
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(root.join("proposal.md"), "# Change: x\n\nStatus: active\n").unwrap();
        fs::write(root.join("plan.md"), "- [ ] Apply it.\n").unwrap();
        fs::write(
            root.join("specs/alpha.md"),
            format!("# Intent delta: alpha\n\n{delta}"),
        )
        .unwrap();
        root
    }

    fn model_at(criticality: Criticality) -> Model {
        let current = parse_spec(
            "alpha.md",
            &format!(
                "# Spec: alpha\n\n## Requirement: existing\nCriticality: {}\n\nOld.\n\n### Scenario: visible\nY occurs when X occurs.\n",
                criticality.name()
            ),
        )
        .unwrap();
        Model {
            specs: vec![current],
            ..Default::default()
        }
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
        MissingRequirementJudgment
        | PendingRequirementReview
        | MissingRequirementAssurance
        | RejectedRequirementJudgment
        | StaleRequirementJudgment
        | MissingMethodQualification
        | RejectedMethodQualification
        | StaleMethodQualification
        | MissingApplicabilityDecision
        | RejectedApplicabilityDecision
        | StaleApplicabilityDecision
        | MissingChallengeDecision
        | StaleChallengeDecision
        | RejectedChallengeDecision
        | MissingRequiredChallenge
        | InsufficientChallengeScope => ValidationFacet::Readiness,
        Unrealized
        | UnresolvedDesignBinding
        | MissingRequiredRealization
        | UnimplementedCheck
        | VerificationElementSupportUnresolved
        | ScenarioCheckContributionUnresolved
        | MechanismSupportUnresolved
        | MissingVerificationChallenge
        | InapplicableVerification => ValidationFacet::Evidence,
        _ => ValidationFacet::Structure,
    }
}
