//! Unit reuse core: the project partition rule, per-unit input fingerprints, and the
//! bundle-side checks that keep a reused unit result sound.
//!
//! A unit's inputs are every tracked file except the private paths of other units. A path claimed
//! by two units is shared by both, which widens each unit's inputs and is recorded, never refused.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use crate::fingerprint::{canonical_sha256, sha256};
use crate::json::{self, Json};
use crate::run::{CheckExecutionUnit, CheckResultOutcome, ExecutedTest, WorkUnit};
use crate::run_plan::{whole_units, RequestedCheck};

pub const RULE_FORMAT: &str = "azimuth-unit-rule";
pub const MANIFEST_FORMAT: &str = "azimuth-unit-input-manifest";
pub const EXPLANATION_FORMAT: &str = "azimuth-unit-plan-explanation";

pub const SELECTION_PARAMETER: &str = "selection";
pub const INPUT_FINGERPRINT_PARAMETER: &str = "input_fingerprint";
pub const HERMETIC_PARAMETER: &str = "hermetic";
pub const RULE_PRIVATE_PATHS_PARAMETER: &str = "rule_private_paths";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRule {
    pub units: Vec<RuleUnit>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleUnit {
    pub id: String,
    pub selection: String,
    pub private_paths: Vec<String>,
    pub hermetic: bool,
    /// Checks this unit partitions; empty means every Check that requests default units.
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedUnit {
    pub id: String,
    pub selection: String,
    pub hermetic: bool,
    pub input_fingerprint: String,
    pub input_file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidenedPath {
    pub path: String,
    pub claimed_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitInputs {
    pub rule_fingerprint: String,
    pub units: Vec<ComputedUnit>,
    pub widened_paths: Vec<WidenedPath>,
}

pub fn load_rule(path: &Path) -> Result<UnitRule, String> {
    let source =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let selections = path
        .parent()
        .map(|directory| directory.join("test-selection.json"))
        .filter(|candidate| candidate.is_file())
        .map(|candidate| load_selection_names(&candidate))
        .transpose()?;
    parse_rule(&source, selections.as_ref())
        .map_err(|detail| format!("{}: {detail}", path.display()))
}

fn load_selection_names(path: &Path) -> Result<BTreeSet<String>, String> {
    let source =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let document =
        json::parse(&source).map_err(|detail| format!("{}: {detail}", path.display()))?;
    match document.get("selections") {
        Some(Json::Obj(pairs)) => Ok(pairs.iter().map(|(name, _)| name.clone()).collect()),
        _ => Err(format!(
            "{}: `selections` must be an object",
            path.display()
        )),
    }
}

pub fn parse_rule(source: &str, selections: Option<&BTreeSet<String>>) -> Result<UnitRule, String> {
    let document = json::parse(source)?;
    let pairs = match &document {
        Json::Obj(pairs) => pairs,
        _ => return Err("unit rule must be an object".into()),
    };
    reject_unknown_fields(pairs, &["format", "version", "units"], "$")?;
    if document.get("format").and_then(Json::as_str) != Some(RULE_FORMAT) {
        return Err(format!("$.format must be `{RULE_FORMAT}`"));
    }
    if document.get("version").and_then(Json::as_num) != Some(1.0) {
        return Err("$.version must be 1".into());
    }
    let items = document
        .get("units")
        .and_then(Json::as_array)
        .ok_or("$.units must be an array")?;
    let mut units = Vec::new();
    for (index, item) in items.iter().enumerate() {
        units.push(parse_rule_unit(
            item,
            &format!("$.units[{index}]"),
            selections,
        )?);
    }
    units.sort_by(|left, right| left.id.cmp(&right.id));
    if units.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err("$.units ids must be unique".into());
    }
    let mut rule = UnitRule {
        units,
        fingerprint: String::new(),
    };
    rule.fingerprint = canonical_sha256(&rule_json(&rule));
    Ok(rule)
}

fn parse_rule_unit(
    value: &Json,
    where_: &str,
    selections: Option<&BTreeSet<String>>,
) -> Result<RuleUnit, String> {
    let pairs = match value {
        Json::Obj(pairs) => pairs,
        _ => return Err(format!("{where_} must be an object")),
    };
    reject_unknown_fields(
        pairs,
        &["id", "selection", "private_paths", "hermetic", "checks"],
        where_,
    )?;
    let id = required_str(value, "id", where_)?.to_string();
    crate::diag::validate_id(&id, false).map_err(|detail| format!("{where_}.id {detail}"))?;
    if id == "whole" {
        return Err(format!("{where_}.id must not be `whole`"));
    }
    let selection = required_str(value, "selection", where_)?.to_string();
    if selections.is_some_and(|known| !known.contains(&selection)) {
        return Err(format!(
            "{where_}.selection `{selection}` is not a selection in test-selection.json"
        ));
    }
    let private_paths = string_list(value, "private_paths", where_, true)?;
    for pattern in &private_paths {
        validate_pattern(pattern)
            .map_err(|detail| format!("{where_}.private_paths `{pattern}` {detail}"))?;
    }
    let hermetic = match value.get("hermetic") {
        None => false,
        Some(Json::Bool(flag)) => *flag,
        Some(_) => return Err(format!("{where_}.hermetic must be a boolean")),
    };
    let checks = if value.get("checks").is_some() {
        string_list(value, "checks", where_, false)?
    } else {
        Vec::new()
    };
    Ok(RuleUnit {
        id,
        selection,
        private_paths,
        hermetic,
        checks,
    })
}

fn reject_unknown_fields(
    pairs: &[(String, Json)],
    allowed: &[&str],
    where_: &str,
) -> Result<(), String> {
    match pairs
        .iter()
        .find(|(key, _)| !allowed.contains(&key.as_str()))
    {
        Some((key, _)) => Err(format!("{where_} has unknown field `{key}`")),
        None => Ok(()),
    }
}

fn required_str<'a>(value: &'a Json, field: &str, where_: &str) -> Result<&'a str, String> {
    match value.get(field).and_then(Json::as_str) {
        Some(text) if !text.is_empty() => Ok(text),
        _ => Err(format!("{where_}.{field} must be a non-empty string")),
    }
}

fn string_list(
    value: &Json,
    field: &str,
    where_: &str,
    require_nonempty: bool,
) -> Result<Vec<String>, String> {
    let items = value
        .get(field)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{where_}.{field} must be an array"))?;
    if require_nonempty && items.is_empty() {
        return Err(format!("{where_}.{field} must not be empty"));
    }
    let mut values = Vec::new();
    for item in items {
        match item.as_str() {
            Some(text) if !text.is_empty() => values.push(text.to_string()),
            _ => {
                return Err(format!(
                    "{where_}.{field} entries must be non-empty strings"
                ))
            }
        }
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(format!("{where_}.{field} entries must be unique"));
    }
    Ok(values)
}

fn rule_json(rule: &UnitRule) -> Json {
    Json::obj(vec![
        ("format", Json::str(RULE_FORMAT)),
        ("version", Json::Num(1.0)),
        (
            "units",
            Json::Arr(
                rule.units
                    .iter()
                    .map(|unit| {
                        Json::obj(vec![
                            ("id", Json::str(&unit.id)),
                            ("selection", Json::str(&unit.selection)),
                            (
                                "private_paths",
                                Json::Arr(unit.private_paths.iter().map(Json::str).collect()),
                            ),
                            ("hermetic", Json::Bool(unit.hermetic)),
                            (
                                "checks",
                                Json::Arr(unit.checks.iter().map(Json::str).collect()),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn validate_pattern(pattern: &str) -> Result<(), String> {
    if pattern.starts_with('/') || pattern.contains('\\') {
        return Err("must be a repository-relative `/`-separated path".into());
    }
    for segment in pattern.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err("has an empty, `.` or `..` segment".into());
        }
        if segment.contains("**") && segment != "**" {
            return Err("may use `**` only as a whole segment".into());
        }
    }
    Ok(())
}

pub fn pattern_matches(pattern: &str, path: &str) -> bool {
    let pattern_segments = pattern.split('/').collect::<Vec<_>>();
    let path_segments = path.split('/').collect::<Vec<_>>();
    segments_match(&pattern_segments, &path_segments)
}

fn segments_match(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => {
            (0..=path.len()).any(|skipped| segments_match(rest, &path[skipped..]))
        }
        Some((segment, rest)) => match path.split_first() {
            Some((name, remaining)) => {
                segment_matches(segment, name) && segments_match(rest, remaining)
            }
            None => false,
        },
    }
}

fn segment_matches(pattern: &str, name: &str) -> bool {
    let pattern = pattern.as_bytes();
    let name = name.as_bytes();
    let mut reachable = vec![false; name.len() + 1];
    reachable[0] = true;
    for &symbol in pattern {
        let mut next = vec![false; name.len() + 1];
        if symbol == b'*' {
            let mut open = false;
            for index in 0..=name.len() {
                open = open || reachable[index];
                next[index] = open;
            }
        } else {
            for index in 0..name.len() {
                if reachable[index] && (symbol == b'?' || symbol == name[index]) {
                    next[index + 1] = true;
                }
            }
        }
        reachable = next;
    }
    reachable[name.len()]
}

fn owners_of(rule: &UnitRule, path: &str) -> Vec<usize> {
    rule.units
        .iter()
        .enumerate()
        .filter(|(_, unit)| {
            unit.private_paths
                .iter()
                .any(|pattern| pattern_matches(pattern, path))
        })
        .map(|(index, _)| index)
        .collect()
}

pub fn compute_inputs(rule: &UnitRule, files: &[SourceFile]) -> UnitInputs {
    let mut files = files.to_vec();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let mut per_unit: Vec<Vec<&SourceFile>> = vec![Vec::new(); rule.units.len()];
    let mut widened_paths = Vec::new();
    for file in &files {
        let owners = owners_of(rule, &file.path);
        match owners.as_slice() {
            [] => per_unit.iter_mut().for_each(|inputs| inputs.push(file)),
            [owner] => per_unit[*owner].push(file),
            claimants => {
                per_unit.iter_mut().for_each(|inputs| inputs.push(file));
                widened_paths.push(WidenedPath {
                    path: file.path.clone(),
                    claimed_by: claimants
                        .iter()
                        .map(|index| rule.units[*index].id.clone())
                        .collect(),
                });
            }
        }
    }
    let units = rule
        .units
        .iter()
        .zip(per_unit)
        .map(|(unit, inputs)| ComputedUnit {
            id: unit.id.clone(),
            selection: unit.selection.clone(),
            hermetic: unit.hermetic,
            input_fingerprint: input_fingerprint(&rule.fingerprint, &inputs),
            input_file_count: inputs.len(),
        })
        .collect();
    UnitInputs {
        rule_fingerprint: rule.fingerprint.clone(),
        units,
        widened_paths,
    }
}

fn input_fingerprint(rule_fingerprint: &str, inputs: &[&SourceFile]) -> String {
    canonical_sha256(&Json::obj(vec![
        ("rule_fingerprint", Json::str(rule_fingerprint)),
        (
            "inputs",
            Json::Arr(
                inputs
                    .iter()
                    .map(|file| {
                        Json::obj(vec![
                            ("path", Json::str(&file.path)),
                            ("digest", Json::str(&file.digest)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]))
}

/// Reads the exact tracked file set of a checkout; untracked files are never inputs.
pub fn read_checkout(root: &Path) -> Result<Vec<SourceFile>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", "--cached"])
        .output()
        .map_err(|error| format!("{}: {error}", root.display()))?;
    if !output.status.success() {
        return Err(format!(
            "{}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let mut files = Vec::new();
    for raw in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|raw| !raw.is_empty())
    {
        let path = String::from_utf8(raw.to_vec())
            .map_err(|_| format!("{}: tracked path is not UTF-8", root.display()))?;
        let full = root.join(&path);
        let metadata = std::fs::symlink_metadata(&full)
            .map_err(|error| format!("{}: {error}", full.display()))?;
        let content = if metadata.file_type().is_symlink() {
            let target = std::fs::read_link(&full)
                .map_err(|error| format!("{}: {error}", full.display()))?;
            format!("symlink:{}", target.to_string_lossy()).into_bytes()
        } else if metadata.is_dir() {
            continue;
        } else {
            std::fs::read(&full).map_err(|error| format!("{}: {error}", full.display()))?
        };
        files.push(SourceFile {
            path,
            digest: format!("sha256:{}", sha256(&content)),
        });
    }
    Ok(files)
}

pub fn unit_parameters(rule: &UnitRule, computed: &ComputedUnit) -> BTreeMap<String, String> {
    let private_paths = rule
        .units
        .iter()
        .flat_map(|unit| {
            unit.private_paths
                .iter()
                .map(|pattern| format!("{}:{pattern}", unit.id))
        })
        .collect::<Vec<_>>()
        .join("\n");
    BTreeMap::from([
        (SELECTION_PARAMETER.to_string(), computed.selection.clone()),
        (
            INPUT_FINGERPRINT_PARAMETER.to_string(),
            computed.input_fingerprint.clone(),
        ),
        (
            HERMETIC_PARAMETER.to_string(),
            computed.hermetic.to_string(),
        ),
        (RULE_PRIVATE_PATHS_PARAMETER.to_string(), private_paths),
    ])
}

/// Replaces default `whole` units with the rule's units and returns, per Check, the units it got.
pub fn assign_units(
    requested: &mut [RequestedCheck],
    rule: &UnitRule,
    inputs: &UnitInputs,
) -> BTreeMap<String, Vec<String>> {
    let mut assigned = BTreeMap::new();
    for check in requested {
        if !check.units.is_empty() && check.units != whole_units() {
            continue;
        }
        let mut units = rule
            .units
            .iter()
            .zip(&inputs.units)
            .filter(|(unit, _)| unit.checks.is_empty() || unit.checks.contains(&check.id))
            .map(|(_, computed)| WorkUnit {
                id: computed.id.clone(),
                parameters: unit_parameters(rule, computed),
            })
            .collect::<Vec<_>>();
        if units.is_empty() {
            continue;
        }
        units.sort_by(|left, right| left.id.cmp(&right.id));
        assigned.insert(
            check.id.clone(),
            units.iter().map(|unit| unit.id.clone()).collect(),
        );
        check.units = units;
    }
    assigned
}

pub fn manifest_json(inputs: &UnitInputs) -> Json {
    Json::obj(vec![
        ("format", Json::str(MANIFEST_FORMAT)),
        ("version", Json::Num(1.0)),
        ("rule_fingerprint", Json::str(&inputs.rule_fingerprint)),
        (
            "units",
            Json::Arr(
                inputs
                    .units
                    .iter()
                    .map(|unit| {
                        Json::obj(vec![
                            ("id", Json::str(&unit.id)),
                            ("input_fingerprint", Json::str(&unit.input_fingerprint)),
                            ("hermetic", Json::Bool(unit.hermetic)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

pub fn release_manifest_json(inputs: &UnitInputs, repository: &str) -> Json {
    let Json::Obj(mut pairs) = manifest_json(inputs) else {
        unreachable!("manifest_json builds an object")
    };
    pairs.insert(2, ("repository".to_string(), Json::str(repository)));
    Json::Obj(pairs)
}

/// The repository is optional so manifests produced without it stay valid; the remaining fields are
/// validated by the assessor's manifest parser so the two cannot drift.
pub fn validate_release_manifest(manifest: &Json) -> Result<Option<String>, String> {
    let Json::Obj(pairs) = manifest else {
        return Err("unit input manifest must be an object".into());
    };
    let repository = match manifest.get("repository") {
        None => None,
        Some(value) => Some(
            value
                .as_str()
                .filter(|id| !id.trim().is_empty())
                .ok_or("unit input manifest `repository` must be a nonempty string")?
                .to_string(),
        ),
    };
    let without_repository = Json::Obj(
        pairs
            .iter()
            .filter(|(key, _)| key != "repository")
            .cloned()
            .collect(),
    );
    crate::deployment_status::validate_unit_manifest(&without_repository)?;
    Ok(repository)
}

pub fn explanation_json(
    inputs: &UnitInputs,
    assigned: &BTreeMap<String, Vec<String>>,
    unpartitioned_checks: &[String],
) -> Json {
    Json::obj(vec![
        ("format", Json::str(EXPLANATION_FORMAT)),
        ("version", Json::Num(1.0)),
        ("rule_fingerprint", Json::str(&inputs.rule_fingerprint)),
        (
            "units",
            Json::Arr(
                inputs
                    .units
                    .iter()
                    .map(|unit| {
                        Json::obj(vec![
                            ("id", Json::str(&unit.id)),
                            ("selection", Json::str(&unit.selection)),
                            ("hermetic", Json::Bool(unit.hermetic)),
                            ("input_fingerprint", Json::str(&unit.input_fingerprint)),
                            ("input_file_count", Json::Num(unit.input_file_count as f64)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "widened_paths",
            Json::Arr(
                inputs
                    .widened_paths
                    .iter()
                    .map(|widened| {
                        Json::obj(vec![
                            ("path", Json::str(&widened.path)),
                            (
                                "claimed_by",
                                Json::Arr(widened.claimed_by.iter().map(Json::str).collect()),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "checks",
            Json::Obj(
                assigned
                    .iter()
                    .map(|(check, units)| {
                        (
                            check.clone(),
                            Json::Arr(units.iter().map(Json::str).collect()),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "whole_checks",
            Json::Arr(unpartitioned_checks.iter().map(Json::str).collect()),
        ),
    ])
}

fn rule_owners_from_parameter(planned: &WorkUnit) -> Vec<(String, String)> {
    planned
        .parameters
        .get(RULE_PRIVATE_PATHS_PARAMETER)
        .map(|joined| {
            joined
                .lines()
                .filter_map(|line| line.split_once(':'))
                .map(|(unit, pattern)| (unit.to_string(), pattern.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn is_valid_reported_file(file: &str) -> bool {
    !file.starts_with('/')
        && !file.contains('\\')
        && file
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

/// Checks a reported unit against the plan entry that launched it. Findings use the `run/` codes
/// of the Run bundle verifier.
pub fn verify_reported_unit(
    check: &str,
    planned: &WorkUnit,
    reported: &CheckExecutionUnit,
    add: &mut impl FnMut(&str, String),
) {
    let planned_fingerprint = planned.parameters.get(INPUT_FINGERPRINT_PARAMETER);
    if planned_fingerprint != reported.input_fingerprint.as_ref() {
        add(
            "run/unit-fingerprint-mismatch",
            format!(
                "Check `{check}` unit `{}` input fingerprint differs from the plan",
                reported.id
            ),
        );
    }
    let hermetic = planned
        .parameters
        .get(HERMETIC_PARAMETER)
        .is_some_and(|flag| flag == "true");
    if hermetic && reported.executed_tests.is_empty() {
        add(
            "run/unit-tests-unreported",
            format!(
                "Check `{check}` unit `{}` is hermetic and must report its executed tests",
                reported.id
            ),
        );
    }
    let owners = rule_owners_from_parameter(planned);
    let mut seen = BTreeSet::new();
    for test in &reported.executed_tests {
        if !seen.insert(test.native_id.as_str()) {
            add(
                "run/test-duplicate",
                format!(
                    "Check `{check}` unit `{}` reports test `{}` more than once",
                    reported.id, test.native_id
                ),
            );
        }
        if !is_valid_reported_file(&test.file) {
            add(
                "run/test-file-invalid",
                format!(
                    "Check `{check}` unit `{}` test `{}` has an invalid defining file `{}`",
                    reported.id, test.native_id, test.file
                ),
            );
            continue;
        }
        let claimants = owners
            .iter()
            .filter(|(_, pattern)| pattern_matches(pattern, &test.file))
            .map(|(unit, _)| unit.as_str())
            .collect::<BTreeSet<_>>();
        if claimants.len() == 1 && !claimants.contains(reported.id.as_str()) {
            add(
                "run/test-outside-unit-inputs",
                format!(
                    "Check `{check}` unit `{}` executed test `{}` defined in `{}`, which is private to unit `{}`",
                    reported.id,
                    test.native_id,
                    test.file,
                    claimants.iter().next().expect("one claimant")
                ),
            );
        }
    }
    let evidence_free = reported
        .executed_tests
        .iter()
        .any(|test: &ExecutedTest| test.outcome.is_evidence_free());
    if evidence_free
        && reported.attempts.iter().any(|attempt| {
            attempt
                .outcomes
                .values()
                .any(|outcome| *outcome == CheckResultOutcome::Satisfied)
        })
    {
        add(
            "run/skipped-test-support",
            format!(
                "Check `{check}` unit `{}` has skipped or unexecuted tests and cannot report a satisfied outcome",
                reported.id
            ),
        );
    }
}
