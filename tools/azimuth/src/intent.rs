//! Deterministic projection of a reviewed intent delta onto accepted spec sources.

use crate::fingerprint::sha256;
use crate::model::{Model, Requirement};
use crate::spec::parse_spec;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FileProjection {
    pub path: PathBuf,
    pub before: String,
    pub after: String,
}

#[derive(Debug)]
pub struct Projection {
    pub files: Vec<FileProjection>,
    pub operations: Vec<(String, bool)>,
    pub target_requirements: usize,
}

impl Projection {
    pub fn preview(&self) -> String {
        let mut output = String::new();
        for file in &self.files {
            output.push_str(&format!(
                "--- {} (accepted)\n+++ {} (target)\n",
                file.path.display(),
                file.path.display()
            ));
            let before: Vec<&str> = file.before.lines().collect();
            let after: Vec<&str> = file.after.lines().collect();
            let mut prefix = 0;
            while prefix < before.len() && prefix < after.len() && before[prefix] == after[prefix] {
                prefix += 1;
            }
            let mut suffix = 0;
            while suffix < before.len() - prefix
                && suffix < after.len() - prefix
                && before[before.len() - suffix - 1] == after[after.len() - suffix - 1]
            {
                suffix += 1;
            }
            if prefix > 3 {
                output.push_str(&format!("... {} unchanged leading line(s)\n", prefix - 3));
            }
            for line in &before[prefix.saturating_sub(3)..prefix] {
                output.push_str(&format!(" {line}\n"));
            }
            for line in &before[prefix..before.len() - suffix] {
                output.push_str(&format!("-{line}\n"));
            }
            for line in &after[prefix..after.len() - suffix] {
                output.push_str(&format!("+{line}\n"));
            }
            for line in &after[after.len() - suffix..(after.len() - suffix + 3).min(after.len())] {
                output.push_str(&format!(" {line}\n"));
            }
            if suffix > 3 {
                output.push_str(&format!("... {} unchanged trailing line(s)\n", suffix - 3));
            }
            output.push_str(&format!(
                "# Target spec: {}\n{}\n",
                file.path.display(),
                file.after
            ));
        }
        for (operation, applied) in &self.operations {
            output.push_str(&format!(
                "{operation}: {}\n",
                if *applied { "applied" } else { "planned" }
            ));
        }
        output
    }
}

#[derive(Debug)]
struct Operation {
    spec: String,
    kind: Kind,
    id: String,
    source: String,
    path: PathBuf,
    line: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Add,
    AddTerm,
    Replace,
    RemoveRequirement,
    RemoveScenario,
}

pub fn fingerprint(block: &str) -> String {
    format!("sha256:{}", sha256(block.trim_end().as_bytes()))
}

pub fn project(change: &Path, model_root: &Path, model: &Model) -> Result<Projection, Vec<String>> {
    let operations = read_operations(change)?;
    let retained_requirements = operations
        .iter()
        .filter(|operation| matches!(operation.kind, Kind::Add | Kind::Replace))
        .map(|operation| operation.id.clone())
        .collect::<BTreeSet<_>>();
    let retained_scenarios = operations
        .iter()
        .filter(|operation| matches!(operation.kind, Kind::Add | Kind::Replace))
        .flat_map(|operation| {
            operation.source.lines().filter_map(|line| {
                line.strip_prefix("### Scenario: ")
                    .or_else(|| line.strip_prefix("### Add scenario: "))
                    .map(str::to_string)
            })
        })
        .collect::<BTreeSet<_>>();
    let mut grouped: BTreeMap<String, Vec<Operation>> = BTreeMap::new();
    for operation in operations {
        grouped
            .entry(operation.spec.clone())
            .or_default()
            .push(operation);
    }
    let mut files = Vec::new();
    let mut statuses = Vec::new();
    let mut errors = Vec::new();
    let mut target_requirements = model.requirement_count();
    for (spec, operations) in grouped {
        let old_requirements = model
            .specs
            .iter()
            .find(|item| item.id == spec)
            .map_or(0, |item| item.requirements.len());
        let path = model
            .specs
            .iter()
            .find(|item| item.id == spec)
            .map(|item| PathBuf::from(&item.path))
            .unwrap_or_else(|| model_root.join(&spec).join("spec.md"));
        let before = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    errors.push(format!("{}: {error}", path.display()));
                    continue;
                }
            }
        } else {
            format!("# Spec: {spec}\n")
        };
        let mut after = before.clone();
        let mut touched = BTreeSet::new();
        for operation in operations {
            let requirement_id = if operation.kind == Kind::RemoveScenario {
                model
                    .find_scenario(&operation.id)
                    .map(|view| view.requirement.id.as_str())
                    .unwrap_or(&operation.id)
            } else {
                operation.id.as_str()
            };
            let identity = if operation.kind == Kind::AddTerm {
                format!("{spec} term {}", operation.id)
            } else {
                requirement_id.to_string()
            };
            if !touched.insert(identity.clone()) {
                errors.push(format!(
                    "{}:{}: multiple operations on `{identity}` are unsupported",
                    operation.path.display(),
                    operation.line
                ));
                continue;
            }
            match apply_operation(
                &after,
                &operation,
                model,
                &retained_requirements,
                &retained_scenarios,
            ) {
                Ok((next, applied)) => {
                    after = next;
                    statuses.push((
                        format!(
                            "{} {} (module {})",
                            operation_name(&operation.kind),
                            operation.id,
                            spec
                        ),
                        applied,
                    ));
                }
                Err(message) => errors.push(format!(
                    "{}:{}: {message}",
                    operation.path.display(),
                    operation.line
                )),
            }
        }
        match parse_spec(&path.display().to_string(), &after) {
            Ok(parsed) if parsed.id == spec && !parsed.requirements.is_empty() => {
                target_requirements =
                    target_requirements + parsed.requirements.len() - old_requirements;
            }
            Ok(parsed) if parsed.id == spec && parsed.requirements.is_empty() => {
                target_requirements -= old_requirements;
                after.clear();
            }
            Ok(_) => errors.push(format!(
                "{}: target spec id differs from `{spec}`",
                path.display()
            )),
            Err(diags) => errors.extend(diags.into_iter().map(|diag| diag.to_string())),
        }
        if (before != after && (path.exists() || !after.is_empty()))
            || (!path.exists() && !after.is_empty())
        {
            files.push(FileProjection {
                path,
                before: if model.specs.iter().any(|item| item.id == spec) {
                    before
                } else {
                    String::new()
                },
                after,
            });
        }
    }
    let mut target_specs = model.specs.clone();
    for file in &files {
        target_specs.retain(|spec| Path::new(&spec.path) != file.path);
        if !file.after.is_empty() {
            match parse_spec(&file.path.display().to_string(), &file.after) {
                Ok(spec) => target_specs.push(spec),
                Err(diags) => errors.extend(diags.into_iter().map(|diag| diag.to_string())),
            }
        }
    }
    let projected = Model {
        specs: target_specs,
        ..Default::default()
    };
    errors.extend(
        projected
            .entity_declaration_issues()
            .into_iter()
            .map(|diag| diag.to_string()),
    );
    if errors.is_empty() {
        Ok(Projection {
            files,
            operations: statuses,
            target_requirements,
        })
    } else {
        Err(errors)
    }
}

pub fn capture(change: &Path, model: &Model) -> Result<Vec<PathBuf>, Vec<String>> {
    let operations = read_operations(change)?;
    let mut by_file: BTreeMap<PathBuf, Vec<Operation>> = BTreeMap::new();
    for operation in operations {
        if matches!(
            operation.kind,
            Kind::Replace | Kind::RemoveRequirement | Kind::RemoveScenario
        ) {
            by_file
                .entry(operation.path.clone())
                .or_default()
                .push(operation);
        }
    }
    let mut edited = Vec::new();
    let mut errors = Vec::new();
    for (path, operations) in by_file {
        let mut lines: Vec<String> = fs::read_to_string(&path)
            .map_err(|error| vec![format!("{}: {error}", path.display())])?
            .lines()
            .map(str::to_string)
            .collect();
        for operation in operations.into_iter().rev() {
            let Some(spec) = model.specs.iter().find(|spec| spec.id == operation.spec) else {
                errors.push(format!(
                    "{}:{}: spec `{}` has no accepted source",
                    path.display(),
                    operation.line,
                    operation.spec
                ));
                continue;
            };
            let source = match fs::read_to_string(&spec.path) {
                Ok(source) => source,
                Err(error) => {
                    errors.push(format!("{}: {error}", spec.path));
                    continue;
                }
            };
            let requirement_id = if operation.kind == Kind::RemoveScenario {
                model
                    .find_scenario(&operation.id)
                    .map(|view| view.requirement.id.as_str())
                    .unwrap_or(&operation.id)
            } else {
                operation.id.as_str()
            };
            let Some((start, end)) =
                find_block(&source, &format!("## Requirement: {requirement_id}"))
            else {
                errors.push(format!(
                    "{}:{}: accepted Requirement `{}` does not exist",
                    path.display(),
                    operation.line,
                    requirement_id
                ));
                continue;
            };
            let block = &source[start..end];
            let subject = if operation.kind == Kind::RemoveScenario {
                let scenario_id = operation.id.as_str();
                match find_block(block, &format!("### Scenario: {scenario_id}")) {
                    Some((scenario_start, scenario_end)) => &block[scenario_start..scenario_end],
                    None => {
                        errors.push(format!(
                            "{}:{}: accepted Scenario `{}` does not exist",
                            path.display(),
                            operation.line,
                            operation.id
                        ));
                        continue;
                    }
                }
            } else {
                block
            };
            let expected = format!("From: {}", fingerprint(subject));
            let index = operation.line;
            if lines
                .get(index)
                .is_some_and(|line| line.starts_with("From: "))
            {
                if lines[index] != expected {
                    errors.push(format!("{}:{}: existing `From:` differs from accepted block; revise the change deliberately", path.display(), index+1));
                }
            } else {
                lines.insert(index, expected);
            }
        }
        if errors.is_empty() {
            let mut content = lines.join("\n");
            content.push('\n');
            edited.push((path, content));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let paths = edited.iter().map(|(path, _)| path.clone()).collect();
    for (path, source) in edited {
        fs::write(&path, source).map_err(|error| vec![format!("{}: {error}", path.display())])?;
    }
    Ok(paths)
}

fn operation_name(kind: &Kind) -> &'static str {
    match kind {
        Kind::Add => "add",
        Kind::AddTerm => "add term",
        Kind::Replace => "replace",
        Kind::RemoveRequirement => "remove requirement",
        Kind::RemoveScenario => "remove scenario",
    }
}

/// Resolves exactly one intent-delta tree for a change. Existing accepted spec
/// transitions remain owned by this projector after account facets move to `deltas/`.
pub fn delta_paths(change: &Path) -> Result<Vec<PathBuf>, Vec<String>> {
    let legacy_root = change.join("specs");
    let account_root = change.join("deltas");
    let mut legacy = Vec::new();
    let mut account = Vec::new();
    if legacy_root.exists() {
        collect(&legacy_root, &mut legacy)?;
    }
    if account_root.exists() {
        collect_named_spec(&account_root, &mut account)?;
    }
    if !legacy.is_empty() && !account.is_empty() {
        return Err(vec![format!(
            "{}:1: both `specs/` and `deltas/**/spec.md` contain intent deltas; choose one authority",
            change.display()
        )]);
    }
    for path in &account {
        let relative = path
            .parent()
            .and_then(|parent| parent.strip_prefix(&account_root).ok());
        let Some(relative) = relative else {
            return Err(vec![format!(
                "{}:1: spec delta needs a module directory",
                path.display()
            )]);
        };
        let module = relative
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        if module.is_empty() {
            return Err(vec![format!(
                "{}:1: spec delta needs a module directory",
                path.display()
            )]);
        }
        let expected = format!("# Intent delta: {module}");
        let source = fs::read_to_string(path).map_err(|error| {
            vec![format!(
                "{}:1: cannot read intent delta: {error}",
                path.display()
            )]
        })?;
        if source.lines().next() != Some(expected.as_str()) {
            return Err(vec![format!("{}:1: expected `{expected}`", path.display())]);
        }
    }
    let mut paths = if account.is_empty() { legacy } else { account };
    paths.sort();
    Ok(paths)
}

fn collect_named_spec(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), Vec<String>> {
    let entries =
        fs::read_dir(root).map_err(|error| vec![format!("{}:1: {error}", root.display())])?;
    for entry in entries {
        let entry = entry.map_err(|error| vec![format!("{}:1: {error}", root.display())])?;
        let kind = entry
            .file_type()
            .map_err(|error| vec![format!("{}:1: {error}", entry.path().display())])?;
        if kind.is_dir() {
            collect_named_spec(&entry.path(), paths)?;
        } else if kind.is_file() && entry.file_name() == "spec.md" {
            paths.push(entry.path());
        }
    }
    Ok(())
}

fn read_operations(change: &Path) -> Result<Vec<Operation>, Vec<String>> {
    let paths = delta_paths(change)?;
    let mut operations = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                errors.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        let lines: Vec<&str> = source.lines().collect();
        let Some(spec) = lines
            .first()
            .and_then(|line| line.strip_prefix("# Intent delta: "))
        else {
            errors.push(format!(
                "{}:1: expected `# Intent delta: <spec-id>`",
                path.display()
            ));
            continue;
        };
        let mut start = 1;
        let mut outside_fence = true;
        while start < lines.len() {
            if lines[start].trim_start().starts_with("```") {
                outside_fence = !outside_fence;
                start += 1;
                continue;
            }
            if !outside_fence || !lines[start].starts_with("## ") {
                start += 1;
                continue;
            }
            let mut end = start + 1;
            let mut fenced = false;
            while end < lines.len() {
                if lines[end].trim_start().starts_with("```") {
                    fenced = !fenced;
                }
                if !fenced && lines[end].starts_with("## ") {
                    break;
                }
                end += 1;
            }
            let heading = lines[start].trim_start_matches("## ");
            let parsed = [
                ("Add requirement: ", Kind::Add),
                ("Add term: ", Kind::AddTerm),
                ("Replace requirement: ", Kind::Replace),
                ("Remove requirement: ", Kind::RemoveRequirement),
                ("Remove scenario: ", Kind::RemoveScenario),
            ]
            .into_iter()
            .find_map(|(prefix, kind)| {
                heading
                    .strip_prefix(prefix)
                    .map(|id| (kind, id.trim().to_string()))
            });
            if let Some((kind, id)) = parsed {
                if let Err(reason) = crate::diag::validate_entity_id(&id) {
                    errors.push(format!("{}:{}: {reason}", path.display(), start + 1));
                    start = end;
                    continue;
                }
                operations.push(Operation {
                    spec: spec.to_string(),
                    kind,
                    id,
                    source: lines[start + 1..end].join("\n"),
                    path: path.clone(),
                    line: start + 1,
                });
            } else {
                errors.push(format!(
                    "{}:{}: unsupported intent operation `{heading}`",
                    path.display(),
                    start + 1
                ));
            }
            start = end;
        }
    }
    if errors.is_empty() {
        Ok(operations)
    } else {
        Err(errors)
    }
}

fn collect(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), Vec<String>> {
    let entries =
        fs::read_dir(root).map_err(|error| vec![format!("{}: {error}", root.display())])?;
    for entry in entries {
        let entry = entry.map_err(|error| vec![format!("{}: {error}", root.display())])?;
        if entry.path().is_dir() {
            collect(&entry.path(), paths)?;
        } else if entry.path().extension().and_then(|value| value.to_str()) == Some("md") {
            paths.push(entry.path());
        }
    }
    Ok(())
}

fn apply_operation(
    source: &str,
    operation: &Operation,
    model: &Model,
    retained_requirements: &BTreeSet<String>,
    retained_scenarios: &BTreeSet<String>,
) -> Result<(String, bool), String> {
    let requirement_id = if operation.kind == Kind::RemoveScenario {
        model
            .find_scenario(&operation.id)
            .map(|view| view.requirement.id.as_str())
            .unwrap_or(&operation.id)
    } else {
        operation.id.as_str()
    };
    let current = find_block(source, &format!("## Requirement: {requirement_id}"));
    let identity = requirement_id.to_string();
    match operation.kind {
        Kind::AddTerm => {
            let block = format!("## Term: {}\n{}", operation.id, operation.source.trim_end());
            if let Some((start, end)) = find_block(source, &format!("## Term: {}", operation.id)) {
                if source[start..end].trim_end() == block.trim_end() {
                    return Ok((source.to_string(), true));
                }
                return Err(format!(
                    "term `{}` already exists with different content",
                    operation.id
                ));
            }
            let candidate = format!("{}\n\n{}\n", source.trim_end(), block.trim_end());
            parse_spec(&operation.path.display().to_string(), &candidate).map_err(
                |diagnostics| {
                    diagnostics
                        .into_iter()
                        .map(|diag| diag.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            )?;
            Ok((candidate, false))
        }
        Kind::Add => {
            let body = operation
                .source
                .replace("### Add scenario: ", "### Scenario: ");
            let body = body.trim_end();
            // A Requirement without labels opens with its statement, which the spec grammar
            // separates from the heading by one blank line.
            let body = if body.trim_start_matches([' ', '\t']).starts_with('\n') {
                format!("\n{}", body.trim_start())
            } else {
                body.trim_start().to_string()
            };
            let block = format!("## Requirement: {}\n{}", operation.id, body);
            parse_requirement(&operation.spec, &block)?;
            if let Some((start, end)) = current {
                if same_requirement(&source[start..end], &block, &operation.spec)? {
                    return Ok((source.to_string(), true));
                }
                return Err(format!(
                    "`{identity}` already exists with different content"
                ));
            }
            Ok((
                format!("{}\n\n{}\n", source.trim_end(), block.trim_end()),
                false,
            ))
        }
        Kind::Replace => {
            let (context_lines, header_source) = take_context_labels(&operation.source);
            let (labels, body) = labels_and_body(
                &header_source,
                &[
                    "From",
                    "Because",
                    "Title",
                    "Realization",
                    "Verification",
                    "Remove scenarios",
                ],
            )?;
            let from = required(&labels, "From")?;
            let because = required(&labels, "Because")?;
            let _ = because;
            let rigor_lines = ["Realization", "Verification"]
                .into_iter()
                .filter_map(|label| labels.get(label).map(|value| format!("{label}: {value}\n")))
                .collect::<String>();
            let title = labels
                .get("Title")
                .map(|value| format!("Title: {value}\n"))
                .unwrap_or_default();
            let context = context_lines
                .iter()
                .map(|line| format!("Context: {line}\n"))
                .collect::<String>();
            let block = format!(
                "## Requirement: {requirement_id}\n{title}{rigor_lines}{context}\n{}",
                body.trim()
            );
            let target = parse_requirement(&operation.spec, &block)?;
            let Some((start, end)) = current else {
                return Err(format!("`{identity}` does not exist"));
            };
            let old = &source[start..end];
            if same_requirement(old, &block, &operation.spec)? {
                return Ok((source.to_string(), true));
            }
            check_from(from, old)?;
            let accepted = parse_requirement(&operation.spec, old)?;
            let removed: BTreeSet<_> = accepted
                .scenarios
                .iter()
                .map(|scenario| scenario.id.as_str())
                .filter(|id| !target.scenarios.iter().any(|scenario| scenario.id == *id))
                .collect();
            let declared: BTreeSet<_> = labels
                .get("Remove scenarios")
                .map(|value| {
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|id| !id.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            if removed != declared {
                return Err(format!(
                    "`{identity}` removes Scenarios {:?}; declare exactly these in `Remove scenarios:`",
                    removed
                ));
            }
            for id in &removed {
                if !retained_scenarios.contains(*id) {
                    ensure_scenario_unreferenced(model, &operation.spec, requirement_id, id)?;
                }
            }
            Ok((replace_range(source, start, end, &block), false))
        }
        Kind::RemoveRequirement => {
            let (labels, body) =
                labels_and_body(&operation.source, &["From", "Because", "Replaced-by"])?;
            if !body.trim().is_empty() {
                return Err("remove requirement accepts labels only".into());
            }
            let from = required(&labels, "From")?;
            required(&labels, "Because")?;
            let Some((start, end)) = current else {
                return Ok((source.to_string(), true));
            };
            check_from(from, &source[start..end])?;
            if !retained_requirements.contains(requirement_id) {
                ensure_requirement_unreferenced(model, &operation.spec, requirement_id)?;
            }
            if let Some(accepted) = model.find_requirement(requirement_id) {
                for scenario in &accepted.requirement.scenarios {
                    if !retained_scenarios.contains(&scenario.id) {
                        ensure_scenario_unreferenced(
                            model,
                            &operation.spec,
                            requirement_id,
                            &scenario.id,
                        )?;
                    }
                }
            }
            Ok((replace_range(source, start, end, ""), false))
        }
        Kind::RemoveScenario => {
            let scenario_id = operation.id.as_str();
            if model.find_scenario(scenario_id).is_none() {
                return Ok((source.to_string(), true));
            }
            let (labels, body) =
                labels_and_body(&operation.source, &["From", "Because", "Replaced-by"])?;
            if !body.trim().is_empty() {
                return Err("remove scenario accepts labels only".into());
            }
            let from = required(&labels, "From")?;
            required(&labels, "Because")?;
            let Some((requirement_start, requirement_end)) = current else {
                return Err(format!("`{identity}` does not exist"));
            };
            let block = &source[requirement_start..requirement_end];
            let Some((scenario_start, scenario_end)) =
                find_block(block, &format!("### Scenario: {scenario_id}"))
            else {
                return Ok((source.to_string(), true));
            };
            check_from(from, &block[scenario_start..scenario_end])?;
            if !retained_scenarios.contains(scenario_id) {
                ensure_scenario_unreferenced(model, &operation.spec, requirement_id, scenario_id)?;
            }
            let next = replace_range(
                source,
                requirement_start + scenario_start,
                requirement_start + scenario_end,
                "",
            );
            let (_, new_end) =
                find_block(&next, &format!("## Requirement: {requirement_id}")).unwrap();
            parse_requirement(&operation.spec, &next[requirement_start..new_end])?;
            Ok((next, false))
        }
    }
}

/// Context lines repeat, so they leave the header before the single-valued labels are read.
fn take_context_labels(source: &str) -> (Vec<String>, String) {
    let (header, body) = source.split_once("\n\n").unwrap_or((source, ""));
    let mut context = Vec::new();
    let mut remaining = Vec::new();
    for line in header.lines() {
        match line.strip_prefix(crate::model::Condition::LABEL) {
            Some(value) => context.push(value.trim().to_string()),
            None => remaining.push(line),
        }
    }
    (context, format!("{}\n\n{body}", remaining.join("\n")))
}

fn labels_and_body(
    source: &str,
    allowed: &[&str],
) -> Result<(BTreeMap<String, String>, String), String> {
    let (header, body) = source.split_once("\n\n").unwrap_or((source, ""));
    let mut labels = BTreeMap::new();
    for line in header.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once(": ") else {
            return Err(format!("unrecognized label line `{line}`"));
        };
        if !allowed.contains(&key) {
            return Err(format!(
                "unknown label `{key}:`; expected {}",
                allowed.join(", ")
            ));
        }
        if labels.insert(key.to_string(), value.to_string()).is_some() {
            return Err(format!("`{key}:` is declared twice"));
        }
    }
    Ok((labels, body.to_string()))
}

fn required<'a>(labels: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    labels
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("missing `{key}:`"))
}

fn check_from(expected: &str, source: &str) -> Result<(), String> {
    let actual = fingerprint(source);
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "stale `From:`; accepted block is {actual}, expected {expected}"
        ))
    }
}

fn parse_requirement(spec: &str, block: &str) -> Result<Requirement, String> {
    let source = format!("# Spec: {spec}\n\n{}\n", block.trim_end());
    parse_spec("<target intent>", &source)
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?
        .requirements
        .into_iter()
        .next()
        .ok_or_else(|| "operation contains no Requirement".into())
}

fn same_requirement(left: &str, right: &str, spec: &str) -> Result<bool, String> {
    let a = parse_requirement(spec, left)?;
    let b = parse_requirement(spec, right)?;
    Ok(a.id == b.id
        && a.title == b.title
        && a.realization == b.realization
        && a.verification == b.verification
        && a.conditions == b.conditions
        && a.statement == b.statement
        && a.scenarios.len() == b.scenarios.len()
        && a.scenarios.iter().zip(&b.scenarios).all(|(a, b)| {
            a.id == b.id && a.conditions == b.conditions && a.statement == b.statement
        }))
}

fn find_block(source: &str, heading: &str) -> Option<(usize, usize)> {
    let mut offset = 0;
    let mut start = None;
    let level = heading.chars().take_while(|ch| *ch == '#').count();
    let mut fenced = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if trimmed.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        if !fenced {
            if trimmed == heading {
                start = Some(offset);
            } else if let Some(begin) = start {
                let hashes = trimmed.chars().take_while(|ch| *ch == '#').count();
                if hashes > 0 && hashes <= level && trimmed.chars().nth(hashes) == Some(' ') {
                    return Some((begin, offset));
                }
            }
        }
        offset += line.len();
    }
    start.map(|begin| (begin, source.len()))
}

fn replace_range(source: &str, start: usize, end: usize, block: &str) -> String {
    let mut result = String::new();
    result.push_str(&source[..start]);
    if !block.is_empty() {
        result.push_str(block.trim_end());
        result.push_str("\n\n");
    }
    result.push_str(source[end..].trim_start_matches('\n'));
    result
}

fn ensure_requirement_unreferenced(
    model: &Model,
    spec: &str,
    requirement: &str,
) -> Result<(), String> {
    let identity = requirement.to_string();
    if model
        .account_realization_for_requirement(spec, requirement)
        .is_some()
        || model.account_verifications.iter().any(|document| {
            document
                .requirements
                .iter()
                .any(|scope| scope.requirement == identity)
        })
        || model.account_supports.iter().any(|support| {
            support
                .elements
                .iter()
                .any(|element| element.requirement == identity)
        })
        || model.workspace.obligation(spec, requirement).is_some()
        || model
            .realizes
            .iter()
            .any(|site| site.requirement == requirement)
        || model.realizations.iter().any(|realization| {
            realization.spec == spec && realization.for_requirement(requirement).is_some()
        })
    {
        return Err(format!(
            "Requirement `{requirement}` in module `{spec}` has current realization or verification references"
        ));
    }
    Ok(())
}

fn ensure_scenario_unreferenced(
    model: &Model,
    spec: &str,
    requirement: &str,
    scenario: &str,
) -> Result<(), String> {
    let identity = scenario.to_string();
    if model.account_verifications.iter().any(|document| {
        document
            .scenarios
            .iter()
            .any(|scope| scope.scenario == identity)
            || document
                .checks
                .iter()
                .any(|check| check.scenarios.contains(&identity))
    }) || model.account_supports.iter().any(|support| {
        support
            .contributions
            .iter()
            .any(|contribution| contribution.scenario == identity)
    }) || model.realizations.iter().any(|realization| {
        realization.spec == spec
            && realization
                .for_requirement(requirement)
                .is_some_and(|entry| {
                    entry
                        .mechanisms
                        .iter()
                        .any(|mechanism| mechanism.scenarios.iter().any(|id| id == scenario))
                })
    }) {
        return Err(format!(
            "`{identity}` has current realization or verification references"
        ));
    }
    Ok(())
}
