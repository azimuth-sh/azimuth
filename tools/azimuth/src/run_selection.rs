//! Explicit execution eligibility and capability routing over the complete account.
use crate::adapter::AdapterConfiguration;
use crate::diag::Diag;
use crate::json::Json;
use crate::model::Model;
use crate::run::{self, Subject};
use crate::run_plan::{self, PlanRequest, RequestedCheck, RunOperation, SchemaError};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRequirements {
    pub subjects: Vec<String>,
    pub family: String,
    pub requirements: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionPolicy {
    pub operation: RunOperation,
    pub planned_at_ms: u64,
    pub subject: Subject,
    pub required_context: BTreeMap<String, String>,
    pub families: Vec<String>,
    pub available_requirements: Vec<String>,
    pub routes: BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionFinding {
    pub check: String,
    pub code: String,
    pub detail: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionEntry {
    pub check: String,
    pub status: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionReport {
    pub request: Option<PlanRequest>,
    pub findings: Vec<SelectionFinding>,
    pub coverage: Vec<SelectionEntry>,
    pub policy: SelectionPolicy,
    pub model_fingerprint: String,
}

pub fn extract_execution(
    path: &str,
    line: usize,
    block: &[&str],
    errors: &mut Vec<Diag>,
) -> (Vec<String>, BTreeMap<String, String>) {
    let mut filtered = block
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    let mut fields = BTreeMap::new();
    let mut found = false;
    let mut cursor = 0;
    while cursor < block.len() && block[cursor].trim().is_empty() {
        cursor += 1;
    }
    while cursor < block.len() {
        if block[cursor].trim().is_empty()
            || (!block[cursor].starts_with("- ") && !block[cursor].starts_with("  "))
        {
            break;
        }
        if !block[cursor].starts_with("- Execution:") {
            cursor += 1;
            continue;
        }
        if found {
            errors.push(Diag::at(
                path,
                line + cursor + 1,
                "Check repeats Execution declaration",
            ));
        }
        found = true;
        if block[cursor] != "- Execution:" {
            errors.push(Diag::at(
                path,
                line + cursor + 1,
                "Execution requires an indented typed map",
            ));
        }
        filtered[cursor] = "\u{0}".into();
        cursor += 1;
        while cursor < block.len() && block[cursor].starts_with("  ") {
            let at = line + cursor + 1;
            let entry = block[cursor]
                .strip_prefix("  - ")
                .and_then(|s| s.split_once(':'));
            if let Some((name, raw)) = entry {
                let key = match name {
                    "Subjects" => "execution-subjects",
                    "Family" => "execution-family",
                    "Requirements" => "execution-requirements",
                    _ => "",
                };
                let value = match execution_scalar(raw.trim()) {
                    Ok(value) => value,
                    Err(detail) => {
                        errors.push(Diag::at(path, at, detail));
                        raw.trim().to_string()
                    }
                };
                if key.is_empty() || fields.insert(key.to_string(), value).is_some() {
                    errors.push(Diag::at(path, at, "unknown or repeated Execution field; expected Subjects, Family or Requirements"));
                }
            } else {
                errors.push(Diag::at(
                    path,
                    at,
                    "Execution field must have form '  - Subjects|Family|Requirements: value'",
                ));
            }
            filtered[cursor] = "\u{0}".into();
            cursor += 1;
        }
    }
    if found {
        if let Err(detail) = execution_from_inputs(&fields) {
            errors.push(Diag::at(path, line, detail));
        }
    }
    (filtered, fields)
}

pub fn execution_from_inputs(
    inputs: &BTreeMap<String, String>,
) -> Result<Option<ExecutionRequirements>, String> {
    let Some(family) = inputs.get("execution-family") else {
        return if inputs.keys().any(|key| key.starts_with("execution-")) {
            Err("Execution requires Family and Subjects".into())
        } else {
            Ok(None)
        };
    };
    if !segment(family) {
        return Err("Execution Family must be one lower-kebab identifier".into());
    }
    let raw_subjects = inputs
        .get("execution-subjects")
        .ok_or("Execution requires Subjects")?;
    let subjects = csv(raw_subjects)?;
    if subjects.is_empty() || subjects.iter().any(|s| !subject_kind_valid(s)) {
        return Err("Execution Subjects must name supported Run Subject kinds".into());
    }
    let requirements = csv(inputs
        .get("execution-requirements")
        .map_or("", String::as_str))?;
    Ok(Some(ExecutionRequirements {
        subjects,
        family: family.clone(),
        requirements,
    }))
}

pub fn parse_policy(path: &str, source: &str) -> Result<SelectionPolicy, Vec<SchemaError>> {
    let root = run::strict_json(path, source).map_err(|e| {
        vec![SchemaError {
            path: e.path,
            detail: e.detail,
        }]
    })?;
    parse_policy_value(&root).map_err(|detail| {
        vec![SchemaError {
            path: path.into(),
            detail,
        }]
    })
}
fn parse_policy_value(value: &Json) -> Result<SelectionPolicy, String> {
    let fields = obj(
        value,
        &[
            "format",
            "version",
            "operation",
            "planned_at_ms",
            "subject",
            "required_context",
            "families",
            "available_requirements",
            "routes",
        ],
    )?;
    if text(get(fields, "format")?)? != "azimuth-run-selection-policy"
        || get(fields, "version")? != &Json::Num(1.0)
    {
        return Err("expected azimuth-run-selection-policy version 1".into());
    }
    let operation = match text(get(fields, "operation")?)? {
        "execute" => RunOperation::Execute,
        "import" => RunOperation::Import,
        _ => return Err("operation must be execute or import".into()),
    };
    let planned = get(fields, "planned_at_ms")?
        .as_num()
        .ok_or("planned_at_ms must be integer")?;
    if planned < 0.0 || planned.fract() != 0.0 || planned > 9_007_199_254_740_991.0 {
        return Err("planned_at_ms must be nonnegative safe integer".into());
    }
    let subject = run::subject_from_json(get(fields, "subject")?)?;
    let validation = run::validate_subject_component(&subject);
    if !validation.is_empty() {
        return Err(validation
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; "));
    }
    let required_context = string_map(get(fields, "required_context")?)?;
    let families = strings(get(fields, "families")?)?;
    if families.is_empty() {
        return Err("families must be nonempty".into());
    }
    let available_requirements = strings(get(fields, "available_requirements")?)?;
    let mut routes = BTreeMap::new();
    for route in get(fields, "routes")?
        .as_array()
        .ok_or("routes must be an array")?
    {
        let fields = obj(route, &["family", "execution-capability"])?;
        let family = text(get(fields, "family")?)?.to_string();
        let capability = text(get(fields, "execution-capability")?)?.to_string();
        if !segment(&family) || !families.contains(&family) {
            return Err("route family must be selected by policy".into());
        }
        if capability.split('/').count() != 2 || !capability.split('/').all(segment) {
            return Err("execution-capability must have explicit adapter/capability form".into());
        }
        if routes.insert(family.clone(), capability).is_some() {
            return Err(format!("ambiguous repeated route for family `{family}`"));
        }
    }
    Ok(SelectionPolicy {
        operation,
        planned_at_ms: planned as u64,
        subject,
        required_context,
        families,
        available_requirements,
        routes,
    })
}

pub fn select(
    model: &Model,
    configuration: &AdapterConfiguration,
    policy: &SelectionPolicy,
) -> SelectionReport {
    let mut report = SelectionReport {
        request: None,
        findings: Vec::new(),
        coverage: Vec::new(),
        policy: policy.clone(),
        model_fingerprint: format!(
            "sha256:{}",
            crate::fingerprint::model_digest(model, &crate::validation::validate(model))
        ),
    };
    let mut selected = Vec::new();
    let kind = subject_kind(&policy.subject);
    for check in model.checks() {
        let authored = model
            .account_verifications
            .iter()
            .flat_map(|file| &file.checks)
            .find(|item| item.definition.id == check.id);
        let requirements = authored.and_then(|a| execution_from_inputs(&a.inputs).ok().flatten());
        let status = match requirements {
            None => {
                finding(&mut report,&check.id,"undeclared-execution","Check has no execution eligibility; explicit Check selection remains available");
                "undeclared"
            }
            Some(requirements) if !policy.families.contains(&requirements.family) => {
                "outside-policy"
            }
            Some(requirements) if !requirements.subjects.iter().any(|s| s == kind) => {
                finding(
                    &mut report,
                    &check.id,
                    "incompatible-subject",
                    "selected family cannot execute this Check against this Subject kind",
                );
                "incompatible"
            }
            Some(requirements)
                if requirements
                    .requirements
                    .iter()
                    .any(|r| !policy.available_requirements.contains(r)) =>
            {
                let missing = requirements
                    .requirements
                    .iter()
                    .filter(|r| !policy.available_requirements.contains(r))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                finding(
                    &mut report,
                    &check.id,
                    "missing-execution-requirements",
                    &format!("unavailable requirements: {missing}"),
                );
                "incompatible"
            }
            Some(requirements) => {
                match policy.routes.get(&requirements.family).and_then(|address| {
                    configuration
                        .capability(address)
                        .map(|(_, capability)| (address, capability))
                }) {
                    None => {
                        finding(
                            &mut report,
                            &check.id,
                            "unrouted-check",
                            "selected eligible Check has no explicit configured capability route",
                        );
                        "unrouted"
                    }
                    Some((_, capability))
                        if !capability.supports(policy.operation.check_class()) =>
                    {
                        finding(
                            &mut report,
                            &check.id,
                            "incompatible-capability",
                            "configured capability does not support selected operation",
                        );
                        "incompatible"
                    }
                    Some((address, _)) => {
                        selected.push(RequestedCheck {
                            id: check.id.clone(),
                            capability: address.clone(),
                            cases: Vec::new(),
                            units: run_plan::whole_units(),
                        });
                        "selected"
                    }
                }
            }
        };
        report.coverage.push(SelectionEntry {
            check: check.id.clone(),
            status: status.into(),
        });
    }
    selected.sort_by(|a, b| a.id.cmp(&b.id));
    report.coverage.sort_by(|a, b| a.check.cmp(&b.check));
    report
        .findings
        .sort_by(|a, b| (&a.check, &a.code).cmp(&(&b.check, &b.code)));
    if !selected.is_empty() {
        report.request = Some(PlanRequest {
            operation: policy.operation,
            planned_at_ms: policy.planned_at_ms,
            subject: policy.subject.clone(),
            required_context: policy.required_context.clone(),
            checks: selected,
            challenges: Vec::new(),
        });
    }
    report
}
fn finding(report: &mut SelectionReport, check: &str, code: &str, detail: &str) {
    report.findings.push(SelectionFinding {
        check: check.into(),
        code: code.into(),
        detail: detail.into(),
    });
}
pub fn to_json(report: &SelectionReport) -> Json {
    Json::obj(vec![
        ("format", Json::str("azimuth-run-selection")),
        ("version", Json::Num(1.0)),
        ("policy", policy_to_json(&report.policy)),
        ("model_fingerprint", Json::str(&report.model_fingerprint)),
        (
            "request",
            report
                .request
                .as_ref()
                .map_or(Json::Null, run_plan::plan_request_to_json),
        ),
        (
            "findings",
            Json::Arr(
                report
                    .findings
                    .iter()
                    .map(|f| {
                        Json::obj(vec![
                            ("check", Json::str(&f.check)),
                            ("code", Json::str(&f.code)),
                            ("detail", Json::str(&f.detail)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "coverage",
            Json::Arr(
                report
                    .coverage
                    .iter()
                    .map(|f| {
                        Json::obj(vec![
                            ("check", Json::str(&f.check)),
                            ("status", Json::str(&f.status)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}
pub fn subject_kind(subject: &Subject) -> &'static str {
    match subject {
        Subject::Workspace { .. } => "workspace",
        Subject::CiCandidate { .. } => "ci-candidate",
        Subject::Artifact { .. } => "artifact",
        Subject::Deployment { .. } => "deployment",
        Subject::Service { .. } => "service",
        Subject::MonitoringWindow { .. } => "monitoring-window",
    }
}
fn subject_kind_valid(value: &str) -> bool {
    [
        "workspace",
        "ci-candidate",
        "artifact",
        "deployment",
        "service",
        "monitoring-window",
    ]
    .contains(&value)
}
fn segment(value: &str) -> bool {
    crate::diag::validate_id(value, false).is_ok()
}
fn csv(value: &str) -> Result<Vec<String>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let mut values = value
        .split(',')
        .map(|v| v.trim().to_string())
        .collect::<Vec<_>>();
    if values.iter().any(|v| !segment(v)) {
        return Err("Execution values must be lower-kebab identifiers".into());
    }
    values.sort();
    if values.windows(2).any(|p| p[0] == p[1]) {
        return Err("Execution values must be unique".into());
    }
    Ok(values)
}
fn obj<'a>(value: &'a Json, allowed: &[&str]) -> Result<&'a [(String, Json)], String> {
    let Json::Obj(fields) = value else {
        return Err("expected an object".into());
    };
    if fields
        .iter()
        .any(|(key, _)| !allowed.contains(&key.as_str()))
    {
        return Err("unknown policy field".into());
    }
    Ok(fields)
}
fn get<'a>(fields: &'a [(String, Json)], key: &str) -> Result<&'a Json, String> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("missing `{key}`"))
}
fn text(value: &Json) -> Result<&str, String> {
    value
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("expected nonempty string".into())
}
fn strings(value: &Json) -> Result<Vec<String>, String> {
    let values = value
        .as_array()
        .ok_or("expected array")?
        .iter()
        .map(|v| text(v).map(str::to_string))
        .collect::<Result<Vec<_>, _>>()?;
    if values.iter().any(|v| !segment(v)) || values.windows(2).any(|p| p[0] >= p[1]) {
        return Err("values must be sorted unique lower-kebab ids".into());
    }
    Ok(values)
}
fn string_map(value: &Json) -> Result<BTreeMap<String, String>, String> {
    let Json::Obj(fields) = value else {
        return Err("required_context must be a string map".into());
    };
    fields
        .iter()
        .map(|(k, v)| {
            if k.is_empty() {
                Err("context key must be nonempty".into())
            } else {
                v.as_str()
                    .map(|s| (k.clone(), s.to_string()))
                    .ok_or("context values must be strings".into())
            }
        })
        .collect()
}

pub fn policy_to_json(policy: &SelectionPolicy) -> Json {
    Json::obj(vec![
        ("format", Json::str("azimuth-run-selection-policy")),
        ("version", Json::Num(1.0)),
        ("operation", Json::str(policy.operation.name())),
        ("planned_at_ms", Json::Num(policy.planned_at_ms as f64)),
        ("subject", run::subject_to_json(&policy.subject)),
        (
            "required_context",
            Json::Obj(
                policy
                    .required_context
                    .iter()
                    .map(|(k, v)| (k.clone(), Json::str(v)))
                    .collect(),
            ),
        ),
        (
            "families",
            Json::Arr(policy.families.iter().map(Json::str).collect()),
        ),
        (
            "available_requirements",
            Json::Arr(
                policy
                    .available_requirements
                    .iter()
                    .map(Json::str)
                    .collect(),
            ),
        ),
        (
            "routes",
            Json::Arr(
                policy
                    .routes
                    .iter()
                    .map(|(family, capability)| {
                        Json::obj(vec![
                            ("family", Json::str(family)),
                            ("execution-capability", Json::str(capability)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}
pub fn request_from_selection(
    model: &Model,
    configuration: &AdapterConfiguration,
    path: &str,
    source: &str,
) -> Result<PlanRequest, Vec<SchemaError>> {
    let parse = || -> Result<PlanRequest, String> {
        let value = run::strict_json(path, source).map_err(|e| e.to_string())?;
        let fields = obj(
            &value,
            &[
                "format",
                "version",
                "policy",
                "model_fingerprint",
                "request",
                "findings",
                "coverage",
            ],
        )?;
        if text(get(fields, "format")?)? != "azimuth-run-selection"
            || get(fields, "version")? != &Json::Num(1.0)
        {
            return Err("expected azimuth-run-selection version 1".into());
        }
        let policy = parse_policy_value(get(fields, "policy")?)?;
        let report = select(model, configuration, &policy);
        if !report.findings.is_empty() {
            return Err(
                "selection has unresolved eligibility, requirement or routing findings".into(),
            );
        }
        if run::canonical_json(&value)? != run::canonical_json(&to_json(&report))? {
            return Err(
                "selection report is stale or differs from recomputed current policy selection"
                    .into(),
            );
        }
        report
            .request
            .ok_or("selection has no eligible routed Checks".into())
    };
    parse().map_err(|detail| {
        vec![SchemaError {
            path: path.into(),
            detail,
        }]
    })
}

fn execution_scalar(raw: &str) -> Result<String, String> {
    raw.split(',')
        .map(|value| {
            let value = value.trim();
            if value.contains('`') {
                value
                    .strip_prefix('`')
                    .and_then(|value| value.strip_suffix('`'))
                    .filter(|value| !value.contains('`'))
                    .map(str::to_string)
                    .ok_or(
                        "Execution values require complete backtick-delimited identifiers".into(),
                    )
            } else {
                Ok(value.to_string())
            }
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|values| values.join(", "))
}
