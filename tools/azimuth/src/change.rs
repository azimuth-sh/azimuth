//! The change lifecycle is deliberately narrower than the accepted-state model.
//!
//! The machine derives additions and their obligations, and verifies that an accepted change was
//! applied before archiving. Explanations of departures and residual acceptance remain authored
//! because deriving them would manufacture the judgment the archive exists to preserve.

use crate::check::{self, Hole};
use crate::fingerprint::sha256;
use crate::json::Json;
use crate::labels::read_block;
use crate::model::{Criticality, Model, StepKind};
use crate::spec::parse_spec;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub steps: Vec<(StepKind, String)>,
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
    pub incomplete_plan_items: usize,
    pub current_claims: usize,
    pub target_claims: usize,
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
    if !root.join("proposal.md").is_file() {
        errors.push("proposal.md is missing".into());
    }
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
    let specs = root.join("specs");
    if specs.exists() {
        let mut files = Vec::new();
        collect_markdown(&specs, &mut files, &mut errors);
        files.sort();
        for file in files {
            match fs::read_to_string(&file) {
                Ok(source) => parse_delta(
                    &file,
                    &source,
                    model,
                    &mut additions,
                    &mut criticality_changes,
                    &mut errors,
                ),
                Err(error) => errors.push(format!("cannot read {}: {error}", file.display())),
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    let unapplied_claims: usize = additions
        .iter()
        .filter(|addition| !addition.applied)
        .map(|addition| addition.scenarios.len())
        .sum();
    Ok(Report {
        id,
        additions,
        criticality_changes,
        incomplete_plan_items,
        current_claims: model.scenario_count(),
        target_claims: model.scenario_count() + unapplied_claims,
    })
}

fn collect_markdown(root: &Path, files: &mut Vec<PathBuf>, errors: &mut Vec<String>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("cannot read {}: {error}", root.display()));
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, files, errors);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            files.push(path);
        }
    }
}

fn parse_delta(
    path: &Path,
    source: &str,
    model: &Model,
    additions: &mut Vec<Addition>,
    criticality_changes: &mut Vec<CriticalityChange>,
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
                "{}: `{}` adds no scenarios",
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
                steps: scenario
                    .steps
                    .iter()
                    .map(|step| (step.kind, step.text.clone()))
                    .collect(),
            })
            .collect();
        let applied = model.specs.iter().any(|candidate| {
            candidate.id == parsed.id
                && candidate.requirements.iter().any(|existing| {
                    existing.id == requirement.id
                        && existing.criticality == Some(criticality)
                        && existing.statement == requirement.statement
                        && scenarios.iter().all(|scenario| {
                            existing.scenarios.iter().any(|item| {
                                item.id == scenario.id
                                    && item.steps.len() == scenario.steps.len()
                                    && item.steps.iter().zip(&scenario.steps).all(
                                        |(current, target)| {
                                            current.kind == target.0 && current.text == target.1
                                        },
                                    )
                            })
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
    if report.additions.is_empty() && report.criticality_changes.is_empty() {
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

pub fn finalization(model: &Model, holes: &[Hole]) -> (String, String) {
    let model_json = model.to_json(holes).to_string_pretty();
    let fingerprint = sha256(model_json.as_bytes());
    let summary = check::summarize(model, holes);
    let json = Json::obj(vec![
        ("version", Json::Num(1.0)),
        ("model_fingerprint", Json::str(&fingerprint)),
        ("claims", Json::Num(summary.claims as f64)),
        ("specs", Json::Num(model.specs.len() as f64)),
        ("errors", Json::Num(summary.errors as f64)),
        ("warnings", Json::Num(summary.warnings as f64)),
    ])
    .to_string_pretty();
    (fingerprint, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::parse_spec;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_CHANGE: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn additions_project_until_the_current_facet_contains_them() {
        let root = temp_change();
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(root.join("proposal.md"), "# Change: x\n\nStatus: active\n").unwrap();
        fs::write(root.join("plan.md"), "- [ ] Apply it.\n").unwrap();
        fs::write(
            root.join("specs/alpha.md"),
            "# Intent delta: alpha\n\n## Add requirement: added\nCriticality: routine\n\nText.\n\n### Add scenario: visible\nWHEN x\nTHEN y\n",
        )
        .unwrap();
        let current = parse_spec(
            "alpha.md",
            "# Spec: alpha\n\n## Requirement: old\nCriticality: routine\n\nOld.\n\n### Scenario: existing\nWHEN x\nTHEN y\n",
        )
        .unwrap();
        let model = Model {
            specs: vec![current],
            ..Default::default()
        };
        let report = inspect(&root, &model).unwrap();
        assert_eq!(report.current_claims, 1);
        assert_eq!(report.target_claims, 2);
        assert!(!report.additions[0].applied);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_applied_addition_requires_the_same_statement_and_steps() {
        let root = temp_change();
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(root.join("proposal.md"), "# Change: x\n\nStatus: active\n").unwrap();
        fs::write(root.join("plan.md"), "- [ ] Apply it.\n").unwrap();
        fs::write(
            root.join("specs/alpha.md"),
            "# Intent delta: alpha\n\n## Add requirement: added\nCriticality: routine\n\nTarget statement.\n\n### Add scenario: visible\nWHEN x\nTHEN target behavior\n",
        )
        .unwrap();
        let current = parse_spec(
            "alpha.md",
            "# Spec: alpha\n\n## Requirement: added\nCriticality: routine\n\nDifferent statement.\n\n### Scenario: visible\nWHEN x\nTHEN different behavior\n",
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
    fn a_criticality_raise_projects_without_changing_claim_identity() {
        let root = transition_change(
            "## Change criticality: existing\nFrom: routine\nTo: standard\nBecause: riders now rely on this value\n",
        );
        let model = model_at(Criticality::Routine);

        let report = inspect(&root, &model).unwrap();

        assert_eq!(report.current_claims, 1);
        assert_eq!(report.target_claims, 1);
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

    #[test]
    fn criticality_is_part_of_judgment_freshness() {
        let routine = model_at(Criticality::Routine);
        let standard = model_at(Criticality::Standard);
        let routine_claim = routine.claims().next().unwrap();
        let standard_claim = standard.claims().next().unwrap();

        assert_ne!(
            routine.claim_text(&routine_claim),
            standard.claim_text(&standard_claim)
        );
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
                "# Spec: alpha\n\n## Requirement: existing\nCriticality: {}\n\nOld.\n\n### Scenario: visible\nWHEN x\nTHEN y\n",
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
