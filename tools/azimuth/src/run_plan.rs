//! Strict D47 Check-only Run planning and launch-plan identity.
//!
//! The planner consumes a complete already-loaded model. It never loads provider semantics, applies
//! a partial model selection, or derives Challenges. Provider routing is a separate launch layer
//! over the unchanged D46 semantic Plan.

use crate::adapter::{AdapterConfiguration, CapabilityClass};
use crate::diag::validate_id;
use crate::json::Json;
use crate::model::Model;
use crate::run::{
    self, CheckSelection, Implementation, LaunchRoute, Plan, RouteCapability, RouteCapabilityClass,
    RouteSelection, RouteSelectionKind, Subject, WorkUnit,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const REQUEST_FORMAT: &str = "azimuth-run-plan-request";
pub const LAUNCH_FORMAT: &str = "azimuth-run-launch-plan";
pub const VERSION: u64 = 1;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    pub path: String,
    pub detail: String,
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningError {
    pub detail: String,
}

impl std::fmt::Display for PlanningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOperation {
    Execute,
    Import,
}

impl RunOperation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Execute => "execute",
            Self::Import => "import",
        }
    }

    pub fn adapter_operation(self) -> crate::adapter::AdapterOperation {
        match self {
            Self::Execute => crate::adapter::AdapterOperation::Execute,
            Self::Import => crate::adapter::AdapterOperation::Import,
        }
    }

    fn check_class(self) -> CapabilityClass {
        match self {
            Self::Execute => CapabilityClass::CheckExecute,
            Self::Import => CapabilityClass::CheckImport,
        }
    }

    fn route_check_class(self) -> RouteCapabilityClass {
        match self {
            Self::Execute => RouteCapabilityClass::CheckExecute,
            Self::Import => RouteCapabilityClass::CheckImport,
        }
    }

    fn route_challenge_class(self) -> RouteCapabilityClass {
        match self {
            Self::Execute => RouteCapabilityClass::ChallengeExecute,
            Self::Import => RouteCapabilityClass::ChallengeImport,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRequest {
    pub operation: RunOperation,
    pub planned_at_ms: u64,
    pub subject: Subject,
    pub required_context: BTreeMap<String, String>,
    pub checks: Vec<RequestedCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedCheck {
    pub id: String,
    pub capability: String,
    pub units: Vec<WorkUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub operation: RunOperation,
    pub planned_at_ms: u64,
    pub subject: Subject,
    pub subject_fingerprint: String,
    pub plan: Plan,
    pub adapter: LaunchAdapter,
    pub routes: Vec<LaunchRoute>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchAdapter {
    pub id: String,
    pub adapter_version: String,
    pub adapter_fingerprint: String,
    pub descriptor_fingerprint: String,
    pub configuration_fingerprint: String,
}

pub fn load_plan_request(path: &Path) -> Result<PlanRequest, Vec<SchemaError>> {
    let source = fs::read_to_string(path).map_err(|error| {
        vec![SchemaError {
            path: path.display().to_string(),
            detail: error.to_string(),
        }]
    })?;
    parse_plan_request(&path.display().to_string(), &source)
}

pub fn parse_plan_request(path: &str, source: &str) -> Result<PlanRequest, Vec<SchemaError>> {
    let root = run::strict_json(path, source).map_err(|error| {
        vec![SchemaError {
            path: error.path,
            detail: error.detail,
        }]
    })?;
    parse_request_value(&root).map_err(|detail| {
        vec![SchemaError {
            path: path.into(),
            detail,
        }]
    })
}

pub fn load_launch_plan(path: &Path) -> Result<LaunchPlan, Vec<SchemaError>> {
    let source = fs::read_to_string(path).map_err(|error| {
        vec![SchemaError {
            path: path.display().to_string(),
            detail: error.to_string(),
        }]
    })?;
    parse_launch_plan(&path.display().to_string(), &source)
}

pub fn parse_launch_plan(path: &str, source: &str) -> Result<LaunchPlan, Vec<SchemaError>> {
    let root = run::strict_json(path, source).map_err(|error| {
        vec![SchemaError {
            path: error.path,
            detail: error.detail,
        }]
    })?;
    let launch = parse_launch_value(&root).map_err(|detail| {
        vec![SchemaError {
            path: path.into(),
            detail,
        }]
    })?;
    let errors = validate_launch_plan(&launch);
    if errors.is_empty() {
        Ok(launch)
    } else {
        Err(errors
            .into_iter()
            .map(|detail| SchemaError {
                path: path.into(),
                detail,
            })
            .collect())
    }
}

/// Resolves a strict Check request against one complete, unselected model.
pub fn plan(
    model: &Model,
    configuration: &AdapterConfiguration,
    request: &PlanRequest,
) -> Result<LaunchPlan, Vec<PlanningError>> {
    let mut errors = Vec::new();
    if let Err(detail) = validate_request(request) {
        errors.push(detail);
    }

    let mut checks = Vec::new();
    let mut routes = Vec::new();
    let mut selected_adapter = None;
    for requested in &request.checks {
        let matching_checks = model
            .checks()
            .filter(|check| check.id == requested.id)
            .collect::<Vec<_>>();
        let check = match matching_checks.as_slice() {
            [] => {
                errors.push(format!("unknown Check `{}`", requested.id));
                continue;
            }
            [check] => *check,
            _ => {
                errors.push(format!(
                    "Check `{}` is declared more than once",
                    requested.id
                ));
                continue;
            }
        };

        let implementations = model
            .check_implementations
            .iter()
            .filter(|implementation| implementation.check == requested.id)
            .collect::<Vec<_>>();
        if implementations.is_empty() {
            errors.push(format!("Check `{}` has no implementation", requested.id));
            continue;
        }
        if implementations
            .iter()
            .any(|implementation| implementation.source.is_none())
        {
            errors.push(format!(
                "Check `{}` has an implementation without stable SourceIdentity",
                requested.id
            ));
            continue;
        }
        let mut resolved_implementations = implementations
            .into_iter()
            .map(|implementation| Implementation {
                identity: implementation.semantic_identity(),
                source_fingerprint: implementation.source_fingerprint.clone(),
            })
            .collect::<Vec<_>>();
        resolved_implementations.sort_by(|left, right| left.identity.cmp(&right.identity));
        if duplicate_by(&resolved_implementations, |item| item.identity.as_str()) {
            errors.push(format!(
                "Check `{}` has duplicate stable implementation identity",
                requested.id
            ));
            continue;
        }

        let Some((adapter, capability)) = configuration.capability(&requested.capability) else {
            errors.push(format!(
                "unknown configured capability `{}`",
                requested.capability
            ));
            continue;
        };
        if !capability.supports(request.operation.check_class()) {
            errors.push(format!(
                "capability `{}` does not support `{}`",
                requested.capability,
                request.operation.check_class().name()
            ));
            continue;
        }
        if let Some(id) = &selected_adapter {
            if id != &adapter.id {
                errors.push("one launch plan cannot route through several adapters".into());
                continue;
            }
        } else {
            selected_adapter = Some(adapter.id.clone());
        }

        checks.push(CheckSelection {
            id: check.id.clone(),
            fingerprint: crate::fingerprint::check_fingerprint(check, &model.check_implementations),
            implementations: resolved_implementations,
            units: requested.units.clone(),
        });
        routes.push(LaunchRoute {
            selection: RouteSelection {
                kind: RouteSelectionKind::Check,
                id: requested.id.clone(),
            },
            capability: RouteCapability {
                address: requested.capability.clone(),
                class: request.operation.route_check_class(),
                challenge_form: None,
                fingerprint: capability.fingerprint.clone(),
            },
        });
    }

    if !errors.is_empty() {
        return Err(planning_errors(errors));
    }
    let adapter_id = selected_adapter.expect("a valid request has a non-empty Check list");
    let adapter = configuration
        .adapter(&adapter_id)
        .expect("selected capability belongs to a configured adapter");
    let subject_fingerprint = run::subject_fingerprint(&request.subject);
    let findings = crate::validation::validate(model);
    let model_fingerprint = format!(
        "sha256:{}",
        crate::fingerprint::model_digest(model, &findings)
    );
    let semantic_plan = run::construct_plan(
        &subject_fingerprint,
        model_fingerprint,
        request.required_context.clone(),
        checks,
        Vec::new(),
    )
    .map_err(|items| {
        planning_errors(
            items
                .into_iter()
                .map(|item| format!("semantic plan {}: {}", item.path, item.detail)),
        )
    })?;
    let mut launch = LaunchPlan {
        operation: request.operation,
        planned_at_ms: request.planned_at_ms,
        subject: request.subject.clone(),
        subject_fingerprint,
        plan: semantic_plan,
        adapter: LaunchAdapter {
            id: adapter.id.clone(),
            adapter_version: adapter.adapter_version.clone(),
            adapter_fingerprint: adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: adapter.configuration_fingerprint.clone(),
        },
        routes,
        fingerprint: zero_fingerprint(),
    };
    launch.fingerprint = launch_fingerprint(&launch);
    let mut validation_errors = validate_launch_plan(&launch);
    validation_errors.extend(validate_launch_configuration(&launch, configuration));
    if validation_errors.is_empty() {
        Ok(launch)
    } else {
        Err(planning_errors(validation_errors))
    }
}

pub fn validate_launch_configuration(
    launch: &LaunchPlan,
    configuration: &AdapterConfiguration,
) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(adapter) = configuration.adapter(&launch.adapter.id) else {
        return vec![format!(
            "launch adapter `{}` is not configured",
            launch.adapter.id
        )];
    };
    let configured_identity = LaunchAdapter {
        id: adapter.id.clone(),
        adapter_version: adapter.adapter_version.clone(),
        adapter_fingerprint: adapter.adapter_fingerprint.clone(),
        descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
        configuration_fingerprint: adapter.configuration_fingerprint.clone(),
    };
    if launch.adapter != configured_identity {
        errors.push(format!(
            "launch adapter identity differs from configured adapter `{}`",
            adapter.id
        ));
    }
    for route in &launch.routes {
        let Some(capability) = adapter.capability(&route.capability.address) else {
            errors.push(format!(
                "route capability `{}` is not configured for adapter `{}`",
                route.capability.address, adapter.id
            ));
            continue;
        };
        let class = match route.capability.class {
            RouteCapabilityClass::CheckExecute => CapabilityClass::CheckExecute,
            RouteCapabilityClass::CheckImport => CapabilityClass::CheckImport,
            RouteCapabilityClass::ChallengeExecute => CapabilityClass::ChallengeExecute,
            RouteCapabilityClass::ChallengeImport => CapabilityClass::ChallengeImport,
        };
        if !capability.supports(class) {
            errors.push(format!(
                "route capability `{}` does not support `{}`",
                route.capability.address,
                class.name()
            ));
        }
        if capability.fingerprint != route.capability.fingerprint {
            errors.push(format!(
                "route capability `{}` fingerprint differs from configuration",
                route.capability.address
            ));
        }
        if let Some(form) = &route.capability.challenge_form {
            if !capability.challenge_forms.contains(form) {
                errors.push(format!(
                    "route capability `{}` does not support Challenge form `{form}`",
                    route.capability.address
                ));
            }
        }
    }
    errors
}

pub fn validate_launch_plan(launch: &LaunchPlan) -> Vec<String> {
    let mut errors = Vec::new();
    let subject_errors = run::validate_subject_component(&launch.subject);
    let subject_is_valid = subject_errors.is_empty();
    errors.extend(
        subject_errors
            .into_iter()
            .map(|error| format!("subject {}: {}", error.path, error.detail)),
    );
    if subject_is_valid {
        let expected_subject = run::subject_fingerprint(&launch.subject);
        if launch.subject_fingerprint != expected_subject {
            errors.push(format!("subject fingerprint must be `{expected_subject}`"));
        }
    }
    errors.extend(
        run::validate_plan_component(&launch.subject_fingerprint, &launch.plan)
            .into_iter()
            .map(|error| format!("plan {}: {}", error.path, error.detail)),
    );
    let planned_time_is_valid = launch.planned_at_ms <= MAX_SAFE_INTEGER;
    if !planned_time_is_valid {
        errors.push("planned_at_ms exceeds the safe-integer limit".into());
    }
    if !valid_segment(&launch.adapter.id) {
        errors.push("launch adapter id is not one lower-kebab segment".into());
    }
    if launch.adapter.adapter_version.is_empty() {
        errors.push("launch adapter version must be non-empty".into());
    }
    for (name, value) in [
        ("adapter", &launch.adapter.adapter_fingerprint),
        ("descriptor", &launch.adapter.descriptor_fingerprint),
        ("configuration", &launch.adapter.configuration_fingerprint),
    ] {
        if !valid_fingerprint(value) {
            errors.push(format!("launch {name} fingerprint has invalid shape"));
        }
    }

    let mut expected = Vec::new();
    expected.extend(launch.plan.checks.iter().map(|check| {
        (
            RouteSelectionKind::Check,
            check.id.as_str(),
            launch.operation.route_check_class(),
        )
    }));
    expected.extend(launch.plan.challenges.iter().map(|challenge| {
        (
            RouteSelectionKind::Challenge,
            challenge.id.as_str(),
            launch.operation.route_challenge_class(),
        )
    }));
    if launch.routes.len() != expected.len() {
        errors.push("routes must contain exactly one entry for every Plan selection".into());
    }
    for (index, route) in launch.routes.iter().enumerate() {
        let Some((kind, id, class)) = expected.get(index) else {
            break;
        };
        if route.selection.kind != *kind || route.selection.id != *id {
            errors.push(format!(
                "routes[{index}] does not match the canonical Plan selection"
            ));
        }
        if route.capability.class != *class {
            errors.push(format!("routes[{index}] class must be `{}`", class.name()));
        }
        let prefix = route
            .capability
            .address
            .split_once('/')
            .map(|parts| parts.0);
        if prefix != Some(launch.adapter.id.as_str()) {
            errors.push(format!(
                "routes[{index}] address must use adapter `{}`",
                launch.adapter.id
            ));
        }
    }
    if subject_is_valid && planned_time_is_valid {
        let expected_fingerprint = launch_fingerprint(launch);
        if launch.fingerprint != expected_fingerprint {
            errors.push(format!(
                "launch fingerprint must be `{expected_fingerprint}`"
            ));
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

pub fn launch_fingerprint(launch: &LaunchPlan) -> String {
    run::canonical_fingerprint(&launch_fingerprint_json(launch))
        .expect("typed launch plans must be canonicalizable")
}

pub fn launch_plan_to_json(launch: &LaunchPlan) -> Json {
    let mut fields = launch_fields(launch);
    fields.push(("fingerprint".into(), Json::str(&launch.fingerprint)));
    Json::Obj(fields)
}

pub fn plan_request_to_json(request: &PlanRequest) -> Json {
    Json::obj(vec![
        ("format", Json::str(REQUEST_FORMAT)),
        ("version", Json::Num(VERSION as f64)),
        ("operation", Json::str(request.operation.name())),
        ("planned_at_ms", Json::Num(request.planned_at_ms as f64)),
        ("subject", run::subject_to_json(&request.subject)),
        (
            "required_context",
            string_map_json(&request.required_context),
        ),
        (
            "checks",
            Json::Arr(request.checks.iter().map(requested_check_json).collect()),
        ),
    ])
}

fn launch_fingerprint_json(launch: &LaunchPlan) -> Json {
    let mut fields = launch_fields(launch);
    fields[0].1 = Json::str("azimuth-run-launch-fingerprint");
    Json::Obj(fields)
}

fn launch_fields(launch: &LaunchPlan) -> Vec<(String, Json)> {
    vec![
        ("format".into(), Json::str(LAUNCH_FORMAT)),
        ("version".into(), Json::Num(VERSION as f64)),
        ("operation".into(), Json::str(launch.operation.name())),
        (
            "planned_at_ms".into(),
            Json::Num(launch.planned_at_ms as f64),
        ),
        ("subject".into(), run::subject_to_json(&launch.subject)),
        (
            "subject_fingerprint".into(),
            Json::str(&launch.subject_fingerprint),
        ),
        ("plan".into(), run::plan_to_json(&launch.plan)),
        ("adapter".into(), launch_adapter_json(&launch.adapter)),
        (
            "routes".into(),
            Json::Arr(
                launch
                    .routes
                    .iter()
                    .map(run::launch_route_to_json)
                    .collect(),
            ),
        ),
    ]
}

fn parse_request_value(value: &Json) -> Result<PlanRequest, String> {
    let fields = object(
        value,
        "$",
        &[
            "format",
            "version",
            "operation",
            "planned_at_ms",
            "subject",
            "required_context",
            "checks",
        ],
    )?;
    exact_string(fields, "format", "$", REQUEST_FORMAT)?;
    exact_integer(fields, "version", "$", VERSION)?;
    let operation = operation(fields, "$", "operation")?;
    let planned_at_ms = integer(fields, "$", "planned_at_ms")?;
    let subject = run::subject_from_json(required(fields, "subject", "$")?)?;
    let required_context = string_map(
        required(fields, "required_context", "$")?,
        "$.required_context",
    )?;
    let check_values = array(required(fields, "checks", "$")?, "$.checks")?;
    let mut checks = Vec::new();
    for (index, value) in check_values.iter().enumerate() {
        checks.push(parse_requested_check(value, &format!("$.checks[{index}]"))?);
    }
    let request = PlanRequest {
        operation,
        planned_at_ms,
        subject,
        required_context,
        checks,
    };
    validate_request(&request)?;
    Ok(request)
}

fn parse_requested_check(value: &Json, where_: &str) -> Result<RequestedCheck, String> {
    let fields = object(value, where_, &["id", "capability", "units"])?;
    let id = nonempty(fields, where_, "id")?;
    validate_id(&id, true).map_err(|detail| format!("{where_}.id {detail}"))?;
    let capability = nonempty(fields, where_, "capability")?;
    validate_capability_address(&capability)
        .map_err(|detail| format!("{where_}.capability {detail}"))?;
    let values = array(
        required(fields, "units", where_)?,
        &format!("{where_}.units"),
    )?;
    let mut units = Vec::new();
    for (index, value) in values.iter().enumerate() {
        units.push(parse_unit(value, &format!("{where_}.units[{index}]"))?);
    }
    validate_units(&units, &format!("{where_}.units"))?;
    Ok(RequestedCheck {
        id,
        capability,
        units,
    })
}

fn parse_unit(value: &Json, where_: &str) -> Result<WorkUnit, String> {
    let fields = object(value, where_, &["id", "parameters"])?;
    let id = nonempty(fields, where_, "id")?;
    validate_id(&id, true).map_err(|detail| format!("{where_}.id {detail}"))?;
    Ok(WorkUnit {
        id,
        parameters: string_map(
            required(fields, "parameters", where_)?,
            &format!("{where_}.parameters"),
        )?,
    })
}

fn parse_launch_value(value: &Json) -> Result<LaunchPlan, String> {
    let fields = object(
        value,
        "$",
        &[
            "format",
            "version",
            "operation",
            "planned_at_ms",
            "subject",
            "subject_fingerprint",
            "plan",
            "adapter",
            "routes",
            "fingerprint",
        ],
    )?;
    exact_string(fields, "format", "$", LAUNCH_FORMAT)?;
    exact_integer(fields, "version", "$", VERSION)?;
    let subject = run::subject_from_json(required(fields, "subject", "$")?)?;
    let plan = run::plan_from_json(required(fields, "plan", "$")?)?;
    let route_values = array(required(fields, "routes", "$")?, "$.routes")?;
    let mut routes = Vec::new();
    for value in route_values {
        routes.push(run::launch_route_from_json(value)?);
    }
    Ok(LaunchPlan {
        operation: operation(fields, "$", "operation")?,
        planned_at_ms: integer(fields, "$", "planned_at_ms")?,
        subject,
        subject_fingerprint: fingerprint(fields, "$", "subject_fingerprint")?,
        plan,
        adapter: parse_launch_adapter(required(fields, "adapter", "$")?)?,
        routes,
        fingerprint: fingerprint(fields, "$", "fingerprint")?,
    })
}

fn parse_launch_adapter(value: &Json) -> Result<LaunchAdapter, String> {
    let fields = object(
        value,
        "$.adapter",
        &[
            "id",
            "adapter_version",
            "adapter_fingerprint",
            "descriptor_fingerprint",
            "configuration_fingerprint",
        ],
    )?;
    let id = nonempty(fields, "$.adapter", "id")?;
    if !valid_segment(&id) {
        return Err("$.adapter.id is not one lower-kebab segment".into());
    }
    Ok(LaunchAdapter {
        id,
        adapter_version: nonempty(fields, "$.adapter", "adapter_version")?,
        adapter_fingerprint: fingerprint(fields, "$.adapter", "adapter_fingerprint")?,
        descriptor_fingerprint: fingerprint(fields, "$.adapter", "descriptor_fingerprint")?,
        configuration_fingerprint: fingerprint(fields, "$.adapter", "configuration_fingerprint")?,
    })
}

fn validate_request(request: &PlanRequest) -> Result<(), String> {
    if request.planned_at_ms > MAX_SAFE_INTEGER {
        return Err("$.planned_at_ms exceeds the safe-integer limit".into());
    }
    let subject_errors = run::validate_subject_component(&request.subject);
    if let Some(error) = subject_errors.first() {
        return Err(format!("$.subject {}: {}", error.path, error.detail));
    }
    if request.checks.is_empty() {
        return Err("$.checks must not be empty".into());
    }
    ensure_sorted_unique(&request.checks, |check| check.id.as_str(), "$.checks")?;
    for (index, check) in request.checks.iter().enumerate() {
        validate_id(&check.id, true).map_err(|detail| format!("$.checks[{index}].id {detail}"))?;
        validate_capability_address(&check.capability)
            .map_err(|detail| format!("$.checks[{index}].capability {detail}"))?;
        validate_units(&check.units, &format!("$.checks[{index}].units"))?;
    }
    Ok(())
}

fn validate_units(units: &[WorkUnit], where_: &str) -> Result<(), String> {
    if units.is_empty() {
        return Err(format!("{where_} must not be empty"));
    }
    ensure_sorted_unique(units, |unit| unit.id.as_str(), where_)?;
    for unit in units {
        validate_id(&unit.id, true).map_err(|detail| format!("{where_} {detail}"))?;
        if unit.parameters.keys().any(|key| key.is_empty()) {
            return Err(format!("{where_} parameter keys must be non-empty"));
        }
    }
    Ok(())
}

fn validate_capability_address(value: &str) -> Result<(), String> {
    let Some((adapter, capability)) = value.split_once('/') else {
        return Err("must have `<adapter-id>/<capability-id>` form".into());
    };
    if capability.contains('/') {
        return Err("must contain exactly two lower-kebab segments".into());
    }
    if valid_segment(adapter) && valid_segment(capability) {
        Ok(())
    } else {
        Err("must contain exactly two lower-kebab segments".into())
    }
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value.as_bytes()[0] != b'-'
        && value.as_bytes()[value.len() - 1] != b'-'
        && !value.contains("--")
}

fn launch_adapter_json(adapter: &LaunchAdapter) -> Json {
    Json::obj(vec![
        ("id", Json::str(&adapter.id)),
        ("adapter_version", Json::str(&adapter.adapter_version)),
        (
            "adapter_fingerprint",
            Json::str(&adapter.adapter_fingerprint),
        ),
        (
            "descriptor_fingerprint",
            Json::str(&adapter.descriptor_fingerprint),
        ),
        (
            "configuration_fingerprint",
            Json::str(&adapter.configuration_fingerprint),
        ),
    ])
}

fn requested_check_json(check: &RequestedCheck) -> Json {
    Json::obj(vec![
        ("id", Json::str(&check.id)),
        ("capability", Json::str(&check.capability)),
        (
            "units",
            Json::Arr(
                check
                    .units
                    .iter()
                    .map(|unit| {
                        Json::obj(vec![
                            ("id", Json::str(&unit.id)),
                            ("parameters", string_map_json(&unit.parameters)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn string_map_json(values: &BTreeMap<String, String>) -> Json {
    Json::Obj(
        values
            .iter()
            .map(|(key, value)| (key.clone(), Json::str(value)))
            .collect(),
    )
}

fn object<'a>(
    value: &'a Json,
    where_: &str,
    allowed: &[&str],
) -> Result<&'a [(String, Json)], String> {
    let Json::Obj(fields) = value else {
        return Err(format!("{where_} must be an object"));
    };
    for (key, _) in fields {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{where_} contains unknown field `{key}`"));
        }
    }
    Ok(fields)
}

fn required<'a>(fields: &'a [(String, Json)], key: &str, where_: &str) -> Result<&'a Json, String> {
    fields
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("{where_} is missing `{key}`"))
}

fn array<'a>(value: &'a Json, where_: &str) -> Result<&'a [Json], String> {
    match value {
        Json::Arr(values) => Ok(values),
        _ => Err(format!("{where_} must be an array")),
    }
}

fn nonempty(fields: &[(String, Json)], where_: &str, key: &str) -> Result<String, String> {
    match required(fields, key, where_)? {
        Json::Str(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(format!("{where_}.{key} must be a non-empty string")),
    }
}

fn exact_string(
    fields: &[(String, Json)],
    key: &str,
    where_: &str,
    expected: &str,
) -> Result<(), String> {
    match required(fields, key, where_)? {
        Json::Str(value) if value == expected => Ok(()),
        _ => Err(format!("{where_}.{key} must be `{expected}`")),
    }
}

fn integer(fields: &[(String, Json)], where_: &str, key: &str) -> Result<u64, String> {
    match required(fields, key, where_)? {
        Json::Num(value)
            if value.is_finite()
                && *value >= 0.0
                && value.fract() == 0.0
                && *value <= MAX_SAFE_INTEGER as f64 =>
        {
            Ok(*value as u64)
        }
        _ => Err(format!(
            "{where_}.{key} must be a non-negative safe integer"
        )),
    }
}

fn exact_integer(
    fields: &[(String, Json)],
    key: &str,
    where_: &str,
    expected: u64,
) -> Result<(), String> {
    let actual = integer(fields, where_, key)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{where_}.{key} must be `{expected}`"))
    }
}

fn operation(fields: &[(String, Json)], where_: &str, key: &str) -> Result<RunOperation, String> {
    match required(fields, key, where_)? {
        Json::Str(value) if value == "execute" => Ok(RunOperation::Execute),
        Json::Str(value) if value == "import" => Ok(RunOperation::Import),
        _ => Err(format!("{where_}.{key} must be `execute | import`")),
    }
}

fn fingerprint(fields: &[(String, Json)], where_: &str, key: &str) -> Result<String, String> {
    let value = nonempty(fields, where_, key)?;
    if valid_fingerprint(&value) {
        Ok(value)
    } else {
        Err(format!(
            "{where_}.{key} must be `sha256:` followed by 64 lowercase hex digits"
        ))
    }
}

fn string_map(value: &Json, where_: &str) -> Result<BTreeMap<String, String>, String> {
    let Json::Obj(fields) = value else {
        return Err(format!("{where_} must be an object"));
    };
    let mut result = BTreeMap::new();
    for (key, value) in fields {
        if key.is_empty() {
            return Err(format!("{where_} contains an empty key"));
        }
        let Json::Str(value) = value else {
            return Err(format!("{where_}.{key} must be a string"));
        };
        if result.insert(key.clone(), value.clone()).is_some() {
            return Err(format!("{where_} contains duplicate key `{key}`"));
        }
    }
    Ok(result)
}

fn ensure_sorted_unique<T, F>(items: &[T], key: F, where_: &str) -> Result<(), String>
where
    F: Fn(&T) -> &str,
{
    for pair in items.windows(2) {
        let left = key(&pair[0]);
        let right = key(&pair[1]);
        if left == right {
            return Err(format!("{where_} contains duplicate identity `{left}`"));
        }
        if left > right {
            return Err(format!("{where_} must be sorted by identity"));
        }
    }
    Ok(())
}

fn duplicate_by<T, F>(items: &[T], key: F) -> bool
where
    F: Fn(&T) -> &str,
{
    let mut seen = BTreeSet::new();
    items.iter().any(|item| !seen.insert(key(item)))
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn zero_fingerprint() -> String {
    format!("sha256:{}", "0".repeat(64))
}

fn planning_errors(items: impl IntoIterator<Item = String>) -> Vec<PlanningError> {
    items
        .into_iter()
        .map(|detail| PlanningError { detail })
        .collect()
}
