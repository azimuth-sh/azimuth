//! Deterministic projection of a reviewed intent delta onto accepted spec sources.

use crate::fingerprint::sha256;
use crate::model::{Claim, Model};
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
    pub target_claims: usize,
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
    RemoveClaim,
    RemoveCase,
    Criticality,
}

pub fn fingerprint(block: &str) -> String {
    format!("sha256:{}", sha256(block.trim_end().as_bytes()))
}

pub fn project(change: &Path, model_root: &Path, model: &Model) -> Result<Projection, Vec<String>> {
    let operations = read_operations(change)?;
    let retained_claims = operations
        .iter()
        .filter(|operation| matches!(operation.kind, Kind::Add | Kind::Replace))
        .map(|operation| operation.id.clone())
        .collect::<BTreeSet<_>>();
    let retained_cases = operations
        .iter()
        .filter(|operation| matches!(operation.kind, Kind::Add | Kind::Replace))
        .flat_map(|operation| {
            operation
                .source
                .lines()
                .filter_map(|line| {
                    line.strip_prefix("### Case: ")
                        .or_else(|| line.strip_prefix("### Add case: "))
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
    let mut target_claims = model.claim_count();
    for (spec, operations) in grouped {
        let old_claims = model
            .specs
            .iter()
            .find(|item| item.id == spec)
            .map_or(0, |item| item.claims.len());
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
            let claim_id = if operation.kind == Kind::RemoveCase {
                model
                    .find_case(&operation.id)
                    .map(|view| view.claim.id.as_str())
                    .unwrap_or(&operation.id)
            } else {
                operation.id.as_str()
            };
            let identity = if operation.kind == Kind::AddTerm {
                format!("{spec} term {}", operation.id)
            } else {
                claim_id.to_string()
            };
            if !touched.insert(identity.clone()) {
                errors.push(format!(
                    "{}:{}: multiple operations on `{identity}` are unsupported",
                    operation.path.display(),
                    operation.line
                ));
                continue;
            }
            match apply_operation(&after, &operation, model, &retained_claims, &retained_cases) {
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
            Ok(parsed) if parsed.id == spec && !parsed.claims.is_empty() => {
                target_claims = target_claims + parsed.claims.len() - old_claims;
            }
            Ok(parsed) if parsed.id == spec && parsed.claims.is_empty() => {
                target_claims -= old_claims;
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
            target_claims,
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
            Kind::Replace | Kind::RemoveClaim | Kind::RemoveCase
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
            let claim_id = if operation.kind == Kind::RemoveCase {
                model
                    .find_case(&operation.id)
                    .map(|view| view.claim.id.as_str())
                    .unwrap_or(&operation.id)
            } else {
                operation.id.as_str()
            };
            let Some((start, end)) = find_block(&source, &format!("## Claim: {claim_id}")) else {
                errors.push(format!(
                    "{}:{}: accepted Claim `{}` does not exist",
                    path.display(),
                    operation.line,
                    claim_id
                ));
                continue;
            };
            let block = &source[start..end];
            let subject = if operation.kind == Kind::RemoveCase {
                let case_id = operation.id.as_str();
                match find_block(block, &format!("### Case: {case_id}")) {
                    Some((case_start, case_end)) => &block[case_start..case_end],
                    None => {
                        errors.push(format!(
                            "{}:{}: accepted Case `{}` does not exist",
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
        Kind::RemoveClaim => "remove claim",
        Kind::RemoveCase => "remove case",
        Kind::Criticality => "criticality",
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
                ("Add claim: ", Kind::Add),
                ("Add term: ", Kind::AddTerm),
                ("Replace claim: ", Kind::Replace),
                ("Remove claim: ", Kind::RemoveClaim),
                ("Remove case: ", Kind::RemoveCase),
                ("Change criticality: ", Kind::Criticality),
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
    retained_claims: &BTreeSet<String>,
    retained_cases: &BTreeSet<String>,
) -> Result<(String, bool), String> {
    let claim_id = if operation.kind == Kind::RemoveCase {
        model
            .find_case(&operation.id)
            .map(|view| view.claim.id.as_str())
            .unwrap_or(&operation.id)
    } else {
        operation.id.as_str()
    };
    let current = find_block(source, &format!("## Claim: {claim_id}"));
    let identity = claim_id.to_string();
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
            let body = operation.source.replace("### Add case: ", "### Case: ");
            let block = format!("## Claim: {}\n{}", operation.id, body.trim());
            parse_claim(&operation.spec, &block)?;
            if let Some((start, end)) = current {
                if same_claim(&source[start..end], &block, &operation.spec)? {
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
            let (labels, body) = labels_and_body(
                &operation.source,
                &["From", "Because", "Criticality", "Remove cases"],
            )?;
            let from = required(&labels, "From")?;
            let because = required(&labels, "Because")?;
            let _ = because;
            let criticality = required(&labels, "Criticality")?;
            let block = format!(
                "## Claim: {claim_id}\nCriticality: {criticality}\n\n{}",
                body.trim()
            );
            let target = parse_claim(&operation.spec, &block)?;
            let Some((start, end)) = current else {
                return Err(format!("`{identity}` does not exist"));
            };
            let old = &source[start..end];
            if same_claim(old, &block, &operation.spec)? {
                return Ok((source.to_string(), true));
            }
            check_from(from, old)?;
            let accepted = parse_claim(&operation.spec, old)?;
            let removed: BTreeSet<_> = accepted
                .cases
                .iter()
                .map(|case| case.id.as_str())
                .filter(|id| !target.cases.iter().any(|case| case.id == *id))
                .collect();
            let declared: BTreeSet<_> = labels
                .get("Remove cases")
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
                    "`{identity}` removes Cases {:?}; declare exactly these in `Remove cases:`",
                    removed
                ));
            }
            for id in &removed {
                if !retained_cases.contains(*id) {
                    ensure_case_unreferenced(model, &operation.spec, claim_id, id)?;
                }
            }
            Ok((replace_range(source, start, end, &block), false))
        }
        Kind::RemoveClaim => {
            let (labels, body) =
                labels_and_body(&operation.source, &["From", "Because", "Replaced-by"])?;
            if !body.trim().is_empty() {
                return Err("remove claim accepts labels only".into());
            }
            let from = required(&labels, "From")?;
            required(&labels, "Because")?;
            let Some((start, end)) = current else {
                return Ok((source.to_string(), true));
            };
            check_from(from, &source[start..end])?;
            if !retained_claims.contains(claim_id) {
                ensure_claim_unreferenced(model, &operation.spec, claim_id)?;
            }
            if let Some(accepted) = model.find_claim(claim_id) {
                for case in &accepted.claim.cases {
                    if !retained_cases.contains(&case.id) {
                        ensure_case_unreferenced(model, &operation.spec, claim_id, &case.id)?;
                    }
                }
            }
            Ok((replace_range(source, start, end, ""), false))
        }
        Kind::RemoveCase => {
            let case_id = operation.id.as_str();
            if model.find_case(case_id).is_none() {
                return Ok((source.to_string(), true));
            }
            let (labels, body) =
                labels_and_body(&operation.source, &["From", "Because", "Replaced-by"])?;
            if !body.trim().is_empty() {
                return Err("remove case accepts labels only".into());
            }
            let from = required(&labels, "From")?;
            required(&labels, "Because")?;
            let Some((claim_start, claim_end)) = current else {
                return Err(format!("`{identity}` does not exist"));
            };
            let block = &source[claim_start..claim_end];
            let Some((case_start, case_end)) = find_block(block, &format!("### Case: {case_id}"))
            else {
                return Ok((source.to_string(), true));
            };
            check_from(from, &block[case_start..case_end])?;
            if !retained_cases.contains(case_id) {
                ensure_case_unreferenced(model, &operation.spec, claim_id, case_id)?;
            }
            let next = replace_range(source, claim_start + case_start, claim_start + case_end, "");
            let (_, new_end) = find_block(&next, &format!("## Claim: {claim_id}")).unwrap();
            parse_claim(&operation.spec, &next[claim_start..new_end])?;
            Ok((next, false))
        }
        Kind::Criticality => {
            let (labels, body) =
                labels_and_body(&operation.source, &["From", "To", "Because", "Revisit"])?;
            if !body.trim().is_empty() {
                return Err("criticality change accepts labels only".into());
            }
            let from = required(&labels, "From")?;
            let to = required(&labels, "To")?;
            required(&labels, "Because")?;
            let Some((start, end)) = current else {
                return Err(format!("`{identity}` does not exist"));
            };
            let block = &source[start..end];
            let old = format!("Criticality: {from}");
            let new = format!("Criticality: {to}");
            if block.lines().any(|line| line == new) {
                return Ok((source.to_string(), true));
            }
            if !block.lines().any(|line| line == old) {
                return Err(format!(
                    "`{identity}` is not declared `From: {from}` or `To: {to}`"
                ));
            }
            Ok((
                replace_range(source, start, end, &block.replacen(&old, &new, 1)),
                false,
            ))
        }
    }
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

fn parse_claim(spec: &str, block: &str) -> Result<Claim, String> {
    let source = format!("# Spec: {spec}\n\n{}\n", block.trim_end());
    parse_spec("<target intent>", &source)
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?
        .claims
        .into_iter()
        .next()
        .ok_or_else(|| "operation contains no Claim".into())
}

fn same_claim(left: &str, right: &str, spec: &str) -> Result<bool, String> {
    let a = parse_claim(spec, left)?;
    let b = parse_claim(spec, right)?;
    Ok(a.id == b.id
        && a.criticality == b.criticality
        && a.statement == b.statement
        && a.cases.len() == b.cases.len()
        && a.cases
            .iter()
            .zip(&b.cases)
            .all(|(a, b)| a.id == b.id && a.statement == b.statement))
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

fn ensure_claim_unreferenced(model: &Model, spec: &str, claim: &str) -> Result<(), String> {
    let identity = claim.to_string();
    if model.account_design_for_claim(spec, claim).is_some()
        || model.account_verifications.iter().any(|document| document.claims.iter().any(|scope| scope.claim == identity))
        || model.account_reviews.iter().any(|document| document.claims.iter().any(|scope| scope.claim == identity))
        || model.account_supports.iter().any(|support| support.elements.iter().any(|element| element.claim == identity))
        || model.workspace.obligation(spec, claim).is_some()
        || model
        .realizes
        .iter()
        .any(|site| site.claim == claim)
        || model
            .designs
            .iter()
            .any(|design| design.spec == spec && design.for_claim(claim).is_some())
        || model.verifications.iter().any(|verification| {
            verification.owner == spec
                && (verification
                    .claim_judgments
                    .iter()
                    .any(|item| item.id == claim || item.id == identity)
                    || verification.challenge_plans.iter().any(|plan| plan.selectors.iter().any(|selector| matches!(selector, crate::verification::Selector::ClaimJudgmentFromClaim(id) if id == &identity)))
                    || verification
                        .bindings
                        .iter()
                        .any(|item| model.find_case(&item.case).is_some_and(|view| view.claim.id == claim)))
        })
    {
        return Err(format!(
            "Claim `{claim}` in module `{spec}` has current realization, design or verification references"
        ));
    }
    Ok(())
}

fn ensure_case_unreferenced(
    model: &Model,
    spec: &str,
    claim: &str,
    case: &str,
) -> Result<(), String> {
    let identity = case.to_string();
    if model.account_verifications.iter().any(|document| document.cases.iter().any(|scope| scope.case == identity) || document.checks.iter().any(|check| check.cases.contains(&identity)))
        || model.account_reviews.iter().any(|document| document.claims.iter().any(|scope| scope.cases.contains(&identity)))
        || model.account_supports.iter().any(|support| support.contributions.iter().any(|contribution| contribution.case == identity))
        || model.verifications.iter().any(|verification| {
        verification
            .bindings
            .iter()
            .any(|item| item.case == identity)
            || verification.claim_judgments.iter().any(|judgment| judgment.basis.iter().any(|basis| basis == &identity))
            || verification.challenge_plans.iter().any(|plan| plan.selectors.iter().any(|selector| matches!(selector, crate::verification::Selector::ApplicabilityDecisionFromCase(id) if id == &identity)))
    }) || model.designs.iter().any(|design| {
        design.spec == spec
            && design.for_claim(claim).is_some_and(|entry| {
                entry
                    .mechanisms
                    .iter()
                    .any(|mechanism| mechanism.cases.iter().any(|id| id == case))
            })
    }) {
        return Err(format!(
            "`{identity}` has current design or verification references"
        ));
    }
    Ok(())
}
