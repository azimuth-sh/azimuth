//! Strict Run planning and launch-plan identity (contracts/run-launch-plan.md).
//!
//! The planner consumes a complete already-loaded model. It never loads provider semantics, applies
//! a partial model selection. Provider routing is a separate launch layer over the unchanged
//! semantic Plan.

use crate::adapter::{AdapterConfiguration, CapabilityClass};
use crate::diag::{validate_check_id, validate_entity_id, validate_id};
use crate::json::Json;
use crate::model::{Model, SemanticChallengeScope, SemanticScopeComponent, SemanticScopeLocator};
use crate::run::{
    self, ChallengeLane, ChallengeScopeItem, ChallengeScopeKind, ChallengeSelection,
    ChallengeTarget, ChallengeTargetKind, CheckSelection, Implementation, LaunchInput,
    LaunchInputKind, LaunchInputSource, LaunchRoute, Plan, RouteCapability, RouteCapabilityClass,
    RouteSelection, RouteSelectionKind, VerificationChallengeRef, VerificationTarget, WorkUnit,
};
use crate::validation::{resolve_challenge_plan, DecisionKind};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const REQUEST_FORMAT: &str = "azimuth-run-plan-request";
pub const LAUNCH_FORMAT: &str = "azimuth-run-launch-plan";
pub const VERSION: u64 = 2;
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

    pub fn check_class(self) -> CapabilityClass {
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

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRequest {
    pub operation: RunOperation,
    pub planned_at_ms: u64,
    pub verification_target: VerificationTarget,
    pub required_execution_context: BTreeMap<String, String>,
    pub checks: Vec<RequestedCheck>,
    pub challenges: Vec<RequestedChallenge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedCheck {
    pub id: String,
    pub capability: String,
    pub scenarios: Vec<String>,
    pub units: Vec<WorkUnit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestedChallenge {
    pub assessment: Option<Json>,
    pub assessment_history: Vec<Json>,
    pub run_history: Vec<Json>,
    pub scope_report: Option<Json>,
    pub id: String,
    pub capability: String,
    pub max_candidates: u64,
    pub units: Vec<WorkUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub operation: RunOperation,
    pub planned_at_ms: u64,
    pub verification_target: VerificationTarget,
    pub verification_target_fingerprint: String,
    pub plan: Plan,
    pub adapter: LaunchAdapter,
    pub adapters: Vec<LaunchAdapter>,
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

/// Resolves a strict Check and Challenge request against one complete, unselected model.
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
    let mut selected_adapters = BTreeSet::new();
    for requested in &request.checks {
        let mut requested = requested.clone();
        if requested.scenarios.is_empty() {
            requested.scenarios = model
                .scenarios()
                .map(|scenario| scenario.id().to_string())
                .filter(|id| model.check_binding_count(&requested.id, id) != 0)
                .collect();
            requested.scenarios.sort();
        }
        if requested.units.is_empty() {
            requested.units = whole_units();
        }
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

        if let Some(authored) = model
            .account_verifications
            .iter()
            .flat_map(|file| &file.checks)
            .find(|item| item.definition.id == requested.id)
        {
            match crate::run_selection::execution_from_inputs(&authored.inputs) {
                Ok(Some(eligibility))
                    if !eligibility.verification_targets.iter().any(|kind| {
                        kind == crate::run_selection::verification_target_kind(
                            &request.verification_target,
                        )
                    }) =>
                {
                    errors.push(format!(
                        "Check `{}` is incompatible with Verification Target kind `{}`",
                        requested.id,
                        crate::run_selection::verification_target_kind(
                            &request.verification_target
                        )
                    ));
                    continue;
                }
                Err(detail) => {
                    errors.push(format!(
                        "Check `{}` has invalid Execution: {detail}",
                        requested.id
                    ));
                    continue;
                }
                _ => {}
            }
        }
        let mut scenario_selection_valid = true;
        for scenario_id in &requested.scenarios {
            if model.find_scenario(scenario_id).is_none() {
                errors.push(format!("unknown Scenario `{scenario_id}`"));
                scenario_selection_valid = false;
                continue;
            }
            if model.check_binding_count(&requested.id, scenario_id) != 1 {
                errors.push(format!(
                    "Check `{}` does not have exactly one Evidence Binding to Scenario `{scenario_id}`",
                    requested.id
                ));
                scenario_selection_valid = false;
                continue;
            }
        }
        if !scenario_selection_valid {
            continue;
        }

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
        selected_adapters.insert(adapter.id.clone());

        checks.push(CheckSelection {
            id: check.id.clone(),
            fingerprint: model.check_execution_fingerprint(check),
            scenarios: requested.scenarios.clone(),
            implementations: resolved_implementations,
            units: requested.units.clone(),
        });
        routes.push(
            run::construct_launch_route(
                RouteSelection {
                    kind: RouteSelectionKind::Check,
                    id: requested.id.clone(),
                },
                RouteCapability {
                    address: requested.capability.clone(),
                    class: request.operation.route_check_class(),
                    challenge_form: None,
                    fingerprint: capability.fingerprint.clone(),
                },
                Vec::new(),
            )
            .expect("model-derived Check route is structurally valid"),
        );
    }

    let (challenges, challenge_routes, challenge_adapter, challenge_errors) =
        resolve_requested_challenges(model, configuration, request);
    errors.extend(challenge_errors);
    if let Some(adapter) = challenge_adapter {
        selected_adapters.insert(adapter);
    }
    for route in &challenge_routes {
        selected_adapters.insert(
            route
                .capability
                .address
                .split_once('/')
                .unwrap()
                .0
                .to_string(),
        );
    }
    routes.extend(challenge_routes);

    if !errors.is_empty() {
        return Err(planning_errors(errors));
    }
    let adapters = selected_adapters
        .iter()
        .map(|id| {
            let adapter = configuration
                .adapter(id)
                .expect("selected adapter is configured");
            LaunchAdapter {
                id: adapter.id.clone(),
                adapter_version: adapter.adapter_version.clone(),
                adapter_fingerprint: adapter.adapter_fingerprint.clone(),
                descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
                configuration_fingerprint: adapter.configuration_fingerprint.clone(),
            }
        })
        .collect::<Vec<_>>();
    let adapter = coordinator_identity(&adapters);
    let verification_target_fingerprint =
        run::verification_target_fingerprint(&request.verification_target);
    let findings = crate::validation::validate(model);
    let model_fingerprint = format!(
        "sha256:{}",
        crate::fingerprint::model_digest(model, &findings)
    );
    let semantic_plan = run::construct_plan(
        &verification_target_fingerprint,
        model_fingerprint,
        request.required_execution_context.clone(),
        checks,
        challenges,
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
        verification_target: request.verification_target.clone(),
        verification_target_fingerprint,
        plan: semantic_plan,
        adapter,
        adapters,
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

fn resolve_requested_challenges(
    model: &Model,
    configuration: &AdapterConfiguration,
    request: &PlanRequest,
) -> (
    Vec<ChallengeSelection>,
    Vec<LaunchRoute>,
    Option<String>,
    Vec<String>,
) {
    #[derive(Clone)]
    struct Pending {
        selection: ChallengeSelection,
        capability: RouteCapability,
        semantic_scope: SemanticChallengeScope,
    }

    let mut errors = Vec::new();
    let mut adapter_id = None;
    let mut pending = BTreeMap::<String, Pending>::new();
    let mut direct_selections = Vec::new();
    let mut direct_routes = Vec::new();
    let mut selected_forms = BTreeMap::<(DecisionKind, String), (String, BTreeSet<String>)>::new();
    for requested in &request.challenges {
        if requested.assessment.is_some() || requested.scope_report.is_some() {
            match resolve_assessment_challenge(model, configuration, request, requested) {
                Ok((selection, route, adapter)) => {
                    adapter_id = Some(adapter);
                    direct_selections.push(selection);
                    direct_routes.push(route);
                }
                Err(error) => errors.push(error),
            }
            continue;
        }
        let plans = model
            .challenge_plans()
            .filter(|plan| plan.id == requested.id)
            .collect::<Vec<_>>();
        let authored = match plans.as_slice() {
            [plan] => *plan,
            [] => {
                errors.push(format!("unknown Challenge Plan `{}`", requested.id));
                continue;
            }
            _ => {
                errors.push(format!(
                    "Challenge Plan `{}` is declared more than once",
                    requested.id
                ));
                continue;
            }
        };
        let resolution = resolve_challenge_plan(model, authored);
        if resolution.candidates.len() as u64 > requested.max_candidates {
            errors.push(format!(
                "Challenge Plan `{}` resolves {} candidates, exceeding max_candidates {}",
                requested.id,
                resolution.candidates.len(),
                requested.max_candidates
            ));
            continue;
        }
        if !resolution.is_runnable() {
            errors.push(if resolution.candidates.is_empty() {
                format!("Challenge Plan `{}` resolves no targets", requested.id)
            } else {
                format!(
                    "Challenge Plan `{}` is not runnable: {:?}",
                    requested.id,
                    resolution
                        .candidates
                        .iter()
                        .map(|candidate| candidate.disposition.name())
                        .collect::<Vec<_>>()
                )
            });
            continue;
        }
        let verification_challenges = model
            .verification_challenges()
            .filter(|verification_challenge| {
                verification_challenge.id == authored.verification_challenge
            })
            .collect::<Vec<_>>();
        let verification_challenge = match verification_challenges.as_slice() {
            [verification_challenge] => *verification_challenge,
            _ => {
                errors.push(format!(
                    "Challenge Plan `{}` does not name exactly one current Verification Challenge",
                    requested.id
                ));
                continue;
            }
        };
        let Some(standards) = model.decision_standards.as_ref() else {
            errors.push("Challenge planning requires Decision Standards".into());
            continue;
        };
        let lane = match (
            standards
                .schedule
                .gate_challenges
                .contains(&verification_challenge.form),
            standards
                .schedule
                .scheduled_challenges
                .contains(&verification_challenge.form),
        ) {
            (true, false) => ChallengeLane::Gate,
            (false, true) => ChallengeLane::Scheduled,
            _ => {
                errors.push(format!(
                    "Challenge form `{}` must have exactly one schedule lane",
                    verification_challenge.form
                ));
                continue;
            }
        };
        let Some((adapter, capability)) = configuration.capability(&requested.capability) else {
            errors.push(format!(
                "unknown configured capability `{}`",
                requested.capability
            ));
            continue;
        };
        let required_class = match request.operation {
            RunOperation::Execute => CapabilityClass::ChallengeExecute,
            RunOperation::Import => CapabilityClass::ChallengeImport,
        };
        if !capability.supports(required_class)
            || !capability
                .challenge_forms
                .contains(&verification_challenge.form)
        {
            errors.push(format!(
                "capability `{}` must support `{}` and Challenge form `{}`",
                requested.capability,
                required_class.name(),
                verification_challenge.form
            ));
            continue;
        }
        adapter_id = Some(adapter.id.clone());

        let mut grouped =
            BTreeMap::<(DecisionKind, String, String), Vec<SemanticChallengeScope>>::new();
        for candidate in resolution.selected() {
            let Some(target) = candidate.target.as_ref() else {
                errors.push(format!(
                    "Challenge Plan `{}` has a selected candidate without a target",
                    requested.id
                ));
                continue;
            };
            let Some(fingerprint) = target.authored_fingerprint.clone() else {
                errors.push(format!(
                    "Challenge Plan `{}` has a target without an authored fingerprint",
                    requested.id
                ));
                continue;
            };
            let Some(scope) = model.challenge_candidate_scope(candidate) else {
                errors.push(format!(
                    "Challenge Plan `{}` cannot derive exact semantic scope",
                    requested.id
                ));
                continue;
            };
            grouped
                .entry((target.kind, target.id.clone(), fingerprint))
                .or_default()
                .push(scope);
        }
        for ((kind, target_id, target_fingerprint), scopes) in grouped {
            let Some(semantic_scope) = SemanticChallengeScope::merge(scopes) else {
                errors.push(format!(
                    "Challenge Plan `{}` has conflicting semantic scope",
                    requested.id
                ));
                continue;
            };
            let present = semantic_scope
                .anchors
                .iter()
                .chain(&semantic_scope.inputs)
                .map(|item| item.kind)
                .collect::<BTreeSet<_>>();
            if !verification_challenge
                .required_scope
                .iter()
                .all(|required| present.contains(required))
            {
                errors.push(format!(
                    "Challenge Plan `{}` does not satisfy Verification Challenge `{}` required scope",
                    requested.id, verification_challenge.id
                ));
                continue;
            }
            let policy_id = match kind {
                DecisionKind::ApplicabilityDecision => {
                    let Some(binding) = model.evidence_bindings().find(|item| item.id == target_id)
                    else {
                        errors.push(format!("missing Evidence Binding `{target_id}`"));
                        continue;
                    };
                    let Some(method) = model
                        .method_qualifications()
                        .find(|item| item.id == binding.method_qualification)
                    else {
                        errors.push(format!(
                            "missing Method Qualification `{}`",
                            binding.method_qualification
                        ));
                        continue;
                    };
                    let mut context = method.execution_context.clone();
                    let conflict = binding.execution_context.iter().any(|(key, value)| {
                        context.get(key).is_some_and(|current| current != value)
                    });
                    context.extend(binding.execution_context.clone());
                    if conflict || context != request.required_execution_context {
                        errors.push(format!(
                            "Applicability Decision `{target_id}` context must equal required_execution_context"
                        ));
                        continue;
                    }
                    binding.policy.clone()
                }
                DecisionKind::MethodQualification => {
                    let Some(qualification) = model
                        .method_qualifications()
                        .find(|item| item.id == target_id)
                    else {
                        errors.push(format!("missing Method Qualification `{target_id}`"));
                        continue;
                    };
                    if qualification.execution_context != request.required_execution_context {
                        errors.push(format!(
                            "Method Qualification `{target_id}` context must equal required_execution_context"
                        ));
                        continue;
                    }
                    qualification.policy.clone()
                }
                DecisionKind::RequirementJudgment => {
                    let Some(judgment) = model
                        .requirement_judgments()
                        .find(|item| item.id == target_id)
                    else {
                        errors.push(format!("missing Requirement Judgment `{target_id}`"));
                        continue;
                    };
                    judgment.policy.clone()
                }
            };
            selected_forms
                .entry((kind, target_fingerprint.clone()))
                .or_insert_with(|| (policy_id, BTreeSet::new()))
                .1
                .insert(verification_challenge.form.clone());

            let verification_challenge_fingerprint =
                crate::fingerprint::verification_challenge_fingerprint(verification_challenge);
            let target_kind = match kind {
                DecisionKind::ApplicabilityDecision => ChallengeTargetKind::ApplicabilityDecision,
                DecisionKind::RequirementJudgment => ChallengeTargetKind::RequirementJudgment,
                DecisionKind::MethodQualification => ChallengeTargetKind::MethodQualification,
            };
            let id = run::challenge_selection_id(
                &verification_challenge_fingerprint,
                target_kind,
                &target_fingerprint,
            );
            let scope = match construct_scope(&semantic_scope) {
                Ok(scope) => scope,
                Err(mut items) => {
                    errors.append(&mut items);
                    continue;
                }
            };
            let candidate = Pending {
                selection: ChallengeSelection {
                    id: id.clone(),
                    verification_challenge: VerificationChallengeRef {
                        id: verification_challenge.id.clone(),
                        fingerprint: verification_challenge_fingerprint,
                    },
                    target: ChallengeTarget {
                        kind: target_kind,
                        id: target_id,
                        fingerprint: target_fingerprint,
                    },
                    lane,
                    scope,
                    units: requested.units.clone(),
                },
                capability: RouteCapability {
                    address: requested.capability.clone(),
                    class: request.operation.route_challenge_class(),
                    challenge_form: Some(verification_challenge.form.clone()),
                    fingerprint: capability.fingerprint.clone(),
                },
                semantic_scope,
            };
            if let Some(existing) = pending.get_mut(&id) {
                if existing.selection.verification_challenge
                    != candidate.selection.verification_challenge
                    || existing.selection.target != candidate.selection.target
                    || existing.selection.lane != candidate.selection.lane
                    || existing.selection.units != candidate.selection.units
                    || existing.capability != candidate.capability
                {
                    errors.push(format!(
                        "duplicate Challenge selection `{id}` has conflicting capability or units"
                    ));
                    continue;
                }
                let Some(merged) = SemanticChallengeScope::merge([
                    existing.semantic_scope.clone(),
                    candidate.semantic_scope,
                ]) else {
                    errors.push(format!(
                        "duplicate Challenge selection `{id}` has conflicting semantic scope"
                    ));
                    continue;
                };
                match construct_scope(&merged) {
                    Ok(scope) => existing.selection.scope = scope,
                    Err(mut items) => errors.append(&mut items),
                }
                existing.semantic_scope = merged;
            } else {
                pending.insert(id, candidate);
            }
        }
    }

    if let Some(standards) = &model.decision_standards {
        for ((kind, fingerprint), (policy_id, forms)) in selected_forms {
            let Some(policy) = standards.policies.iter().find(|item| item.id == policy_id) else {
                errors.push(format!(
                    "selected {} `{fingerprint}` names unknown Decision Policy `{policy_id}`",
                    kind.name()
                ));
                continue;
            };
            for required in &policy.required_challenges {
                if !forms.contains(required) {
                    errors.push(format!(
                        "selected {} `{fingerprint}` is missing required Challenge form `{required}`",
                        kind.name()
                    ));
                }
            }
        }
    }

    let mut selections = direct_selections;
    let mut routes = direct_routes;
    for (_, item) in pending {
        let inputs = match construct_inputs(&item.semantic_scope) {
            Ok(inputs) => inputs,
            Err(mut items) => {
                errors.append(&mut items);
                continue;
            }
        };
        let route = run::construct_launch_route(
            RouteSelection {
                kind: RouteSelectionKind::Challenge,
                id: item.selection.id.clone(),
            },
            item.capability,
            inputs,
        );
        match route {
            Ok(route) => {
                selections.push(item.selection);
                routes.push(route);
            }
            Err(items) => errors.extend(items.into_iter().map(|item| item.detail)),
        }
    }
    (selections, routes, adapter_id, errors)
}

fn construct_scope(scope: &SemanticChallengeScope) -> Result<run::ChallengeScope, Vec<String>> {
    run::construct_challenge_scope(
        scope.anchors.iter().map(scope_item).collect(),
        scope.inputs.iter().map(scope_item).collect(),
    )
    .map_err(|items| items.into_iter().map(|item| item.detail).collect())
}

fn scope_item(item: &SemanticScopeComponent) -> ChallengeScopeItem {
    ChallengeScopeItem {
        kind: scope_kind(item.kind),
        id: item.id.clone(),
        fingerprint: item.fingerprint.clone(),
    }
}

fn scope_kind(kind: crate::verification::SemanticScopeKind) -> ChallengeScopeKind {
    use crate::verification::SemanticScopeKind as S;
    match kind {
        S::ApplicabilityDecision => ChallengeScopeKind::ApplicabilityDecision,
        S::Scenario => ChallengeScopeKind::Scenario,
        S::Requirement => ChallengeScopeKind::Requirement,
        S::Binding => ChallengeScopeKind::Binding,
        S::MethodQualification => ChallengeScopeKind::MethodQualification,
        S::RequirementJudgment => ChallengeScopeKind::RequirementJudgment,
        S::Check => ChallengeScopeKind::Check,
        S::CheckImplementation => ChallengeScopeKind::CheckImplementation,
        S::Realization => ChallengeScopeKind::Realization,
        S::Mechanism => ChallengeScopeKind::Mechanism,
        S::MechanismImplementation => ChallengeScopeKind::MechanismImplementation,
        S::Artifact => ChallengeScopeKind::Artifact,
        S::Context => ChallengeScopeKind::Context,
        S::Policy => ChallengeScopeKind::Policy,
        S::Area => ChallengeScopeKind::Area,
        S::RealizationObligation => ChallengeScopeKind::RealizationObligation,
        S::Surface => ChallengeScopeKind::Surface,
        S::SurfaceMember => ChallengeScopeKind::SurfaceMember,
        S::Enumeration => ChallengeScopeKind::Enumeration,
    }
}

fn construct_inputs(scope: &SemanticChallengeScope) -> Result<Vec<LaunchInput>, Vec<String>> {
    let mut inputs = BTreeMap::new();
    for item in scope.anchors.iter().chain(&scope.inputs) {
        let Some((kind, source)) = launch_input_parts(item) else {
            continue;
        };
        let input =
            run::construct_launch_input(kind, item.id.clone(), item.fingerprint.clone(), source)
                .map_err(|error| vec![error.detail])?;
        let key = (input.kind, input.id.clone(), input.fingerprint.clone());
        if inputs.insert(key, input).is_some() {
            continue;
        }
    }
    Ok(inputs.into_values().collect())
}

fn launch_input_parts(
    item: &SemanticScopeComponent,
) -> Option<(LaunchInputKind, LaunchInputSource)> {
    let kind = match item.kind {
        crate::verification::SemanticScopeKind::CheckImplementation => {
            LaunchInputKind::CheckImplementation
        }
        crate::verification::SemanticScopeKind::Realization => LaunchInputKind::Realization,
        crate::verification::SemanticScopeKind::MechanismImplementation => {
            LaunchInputKind::MechanismImplementation
        }
        crate::verification::SemanticScopeKind::Artifact => LaunchInputKind::Artifact,
        crate::verification::SemanticScopeKind::SurfaceMember => LaunchInputKind::SurfaceMember,
        crate::verification::SemanticScopeKind::Enumeration => LaunchInputKind::Enumeration,
        _ => return None,
    };
    let source = match item.locator.as_ref()? {
        SemanticScopeLocator::Source {
            file,
            language,
            site,
        } => LaunchInputSource::Source {
            file: file.clone(),
            language: language.clone(),
            site: site.clone(),
        },
        SemanticScopeLocator::Artifact {
            file,
            artifact_kind,
            identity,
            unique,
            columns,
            predicate,
        } => LaunchInputSource::Artifact {
            file: file.clone(),
            artifact_kind: artifact_kind.clone(),
            identity: identity.clone(),
            unique: *unique,
            columns: columns.clone(),
            predicate: predicate.clone(),
        },
        SemanticScopeLocator::Enumeration {
            file,
            enumerator_kind,
            identity,
        } => LaunchInputSource::Enumeration {
            file: file.clone(),
            enumerator_kind: enumerator_kind.clone(),
            identity: identity.clone(),
        },
        SemanticScopeLocator::EnumeratedSurfaceMember {
            file,
            language,
            site,
        } => LaunchInputSource::SurfaceMember {
            file: file.clone(),
            language: language.clone(),
            site: site.clone(),
        },
    };
    Some((kind, source))
}

pub fn validate_launch_configuration(
    launch: &LaunchPlan,
    configuration: &AdapterConfiguration,
) -> Vec<String> {
    let mut errors = Vec::new();
    for identity in &launch.adapters {
        let Some(adapter) = configuration.adapter(&identity.id) else {
            return vec![format!(
                "launch adapter `{}` is not configured",
                identity.id
            )];
        };
        let configured_identity = LaunchAdapter {
            id: adapter.id.clone(),
            adapter_version: adapter.adapter_version.clone(),
            adapter_fingerprint: adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: adapter.configuration_fingerprint.clone(),
        };
        if *identity != configured_identity {
            errors.push(format!(
                "launch adapter identity differs from configured adapter `{}`",
                adapter.id
            ));
        }
    }
    for route in &launch.routes {
        let Some((_adapter, capability)) = configuration.capability(&route.capability.address)
        else {
            errors.push(format!(
                "route capability `{}` is not configured for adapter `{}`",
                route.capability.address, launch.adapter.id
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
    if launch.adapters.len() > 64 {
        errors.push("launch has more than 64 adapters".into());
    }
    for identity in &launch.adapters {
        if identity.id == "azimuth-coordinator"
            || !valid_segment(&identity.id)
            || identity.adapter_version.is_empty()
            || [
                &identity.adapter_fingerprint,
                &identity.descriptor_fingerprint,
                &identity.configuration_fingerprint,
            ]
            .iter()
            .any(|fp| !valid_fingerprint(fp))
        {
            errors.push(
                "launch adapters require configured nonreserved identities and valid fingerprints"
                    .into(),
            );
        }
    }
    if launch.adapters.is_empty()
        || launch
            .adapters
            .windows(2)
            .any(|pair| pair[0].id >= pair[1].id)
    {
        errors.push("launch adapters must be nonempty, sorted and unique".into());
    }
    if launch.adapter != coordinator_identity(&launch.adapters) {
        errors.push("launch coordinator identity differs from exact adapter identities".into());
    }
    let used = launch
        .routes
        .iter()
        .filter_map(|r| r.capability.address.split_once('/').map(|(id, _)| id))
        .collect::<BTreeSet<_>>();
    if used
        != launch
            .adapters
            .iter()
            .map(|a| a.id.as_str())
            .collect::<BTreeSet<_>>()
    {
        errors.push("launch adapters must match routed adapters exactly".into());
    }
    let verification_target_errors =
        run::validate_verification_target_component(&launch.verification_target);
    let verification_target_is_valid = verification_target_errors.is_empty();
    errors.extend(
        verification_target_errors
            .into_iter()
            .map(|error| format!("verification target {}: {}", error.path, error.detail)),
    );
    if verification_target_is_valid {
        let expected_verification_target =
            run::verification_target_fingerprint(&launch.verification_target);
        if launch.verification_target_fingerprint != expected_verification_target {
            errors.push(format!(
                "verification target fingerprint must be `{expected_verification_target}`"
            ));
        }
    }
    errors.extend(
        run::validate_plan_component(&launch.verification_target_fingerprint, &launch.plan)
            .into_iter()
            .map(|error| format!("plan {}: {}", error.path, error.detail)),
    );
    errors.extend(
        run::validate_launch_routes_against_plan(&launch.plan, &launch.routes)
            .into_iter()
            .map(|error| format!("routes {}: {}", error.path, error.detail)),
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
        if let Err(detail) = validate_capability_address(&route.capability.address) {
            errors.push(format!("routes[{index}] capability address {detail}"));
        }
        match route.selection.kind {
            RouteSelectionKind::Check => {
                if route.capability.challenge_form.is_some() {
                    errors.push(format!(
                        "routes[{index}] challenge_form is forbidden for a Check route"
                    ));
                }
            }
            RouteSelectionKind::Challenge => match &route.capability.challenge_form {
                Some(form) if valid_path_id(form) => {}
                Some(_) => errors.push(format!(
                    "routes[{index}] challenge_form must be a lower-kebab path id"
                )),
                None => errors.push(format!(
                    "routes[{index}] challenge_form is required for a Challenge route"
                )),
            },
        }
        let Some((kind, id, class)) = expected.get(index) else {
            continue;
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
        if !launch
            .adapters
            .iter()
            .any(|adapter| prefix == Some(adapter.id.as_str()))
        {
            errors.push(format!(
                "routes[{index}] address must use adapter `{}`",
                launch.adapter.id
            ));
        }
    }
    if verification_target_is_valid && planned_time_is_valid {
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
    run::launch_fingerprint(
        match launch.operation {
            RunOperation::Execute => run::ProvenanceMode::Execute,
            RunOperation::Import => run::ProvenanceMode::Import,
        },
        launch.planned_at_ms,
        &launch.verification_target,
        &launch.verification_target_fingerprint,
        &launch.plan,
        &run::LaunchAdapterIdentity {
            id: launch.adapter.id.clone(),
            adapter_version: launch.adapter.adapter_version.clone(),
            adapter_fingerprint: launch.adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: launch.adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: launch.adapter.configuration_fingerprint.clone(),
        },
        &launch.routes,
    )
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
        (
            "verification_target",
            run::verification_target_to_json(&request.verification_target),
        ),
        (
            "required_execution_context",
            string_map_json(&request.required_execution_context),
        ),
        (
            "checks",
            Json::Arr(request.checks.iter().map(requested_check_json).collect()),
        ),
        (
            "challenges",
            Json::Arr(
                request
                    .challenges
                    .iter()
                    .map(requested_challenge_json)
                    .collect(),
            ),
        ),
    ])
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
        (
            "verification_target".into(),
            run::verification_target_to_json(&launch.verification_target),
        ),
        (
            "verification_target_fingerprint".into(),
            Json::str(&launch.verification_target_fingerprint),
        ),
        ("plan".into(), run::plan_to_json(&launch.plan)),
        (
            "adapters".into(),
            Json::Arr(launch.adapters.iter().map(launch_adapter_json).collect()),
        ),
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
            "verification_target",
            "required_execution_context",
            "checks",
            "challenges",
        ],
    )?;
    exact_string(fields, "format", "$", REQUEST_FORMAT)?;
    exact_integer(fields, "version", "$", VERSION)?;
    let operation = operation(fields, "$", "operation")?;
    let planned_at_ms = integer(fields, "$", "planned_at_ms")?;
    let verification_target =
        run::verification_target_from_json(required(fields, "verification_target", "$")?)?;
    let required_execution_context = string_map(
        required(fields, "required_execution_context", "$")?,
        "$.required_execution_context",
    )?;
    let check_values = array(required(fields, "checks", "$")?, "$.checks")?;
    let mut checks = Vec::new();
    for (index, value) in check_values.iter().enumerate() {
        checks.push(parse_requested_check(value, &format!("$.checks[{index}]"))?);
    }
    let challenge_values = array(required(fields, "challenges", "$")?, "$.challenges")?;
    let mut challenges = Vec::new();
    for (index, value) in challenge_values.iter().enumerate() {
        challenges.push(parse_requested_challenge(
            value,
            &format!("$.challenges[{index}]"),
        )?);
    }
    let request = PlanRequest {
        operation,
        planned_at_ms,
        verification_target,
        required_execution_context,
        checks,
        challenges,
    };
    validate_request(&request)?;
    Ok(request)
}

fn parse_requested_challenge(value: &Json, where_: &str) -> Result<RequestedChallenge, String> {
    let fields = object(
        value,
        where_,
        &[
            "id",
            "execution-capability",
            "max_candidates",
            "units",
            "assessment",
            "assessment_history",
            "run_history",
            "scope_report",
        ],
    )?;
    let id = nonempty(fields, where_, "id")?;
    validate_id(&id, true).map_err(|detail| format!("{where_}.id {detail}"))?;
    let capability = nonempty(fields, where_, "execution-capability")?;
    validate_capability_address(&capability)
        .map_err(|detail| format!("{where_}.execution-capability {detail}"))?;
    let max_candidates = integer(fields, where_, "max_candidates")?;
    if max_candidates == 0 {
        return Err(format!("{where_}.max_candidates must be at least 1"));
    }
    let values = array(
        required(fields, "units", where_)?,
        &format!("{where_}.units"),
    )?;
    let mut units = Vec::new();
    for (index, value) in values.iter().enumerate() {
        units.push(parse_unit(value, &format!("{where_}.units[{index}]"))?);
    }
    validate_units(&units, &format!("{where_}.units"))?;
    Ok(RequestedChallenge {
        assessment: fields
            .iter()
            .find(|(key, _)| key == "assessment")
            .map(|(_, value)| value.clone()),
        scope_report: fields
            .iter()
            .find(|(key, _)| key == "scope_report")
            .map(|(_, value)| value.clone()),
        assessment_history: fields
            .iter()
            .find(|(key, _)| key == "assessment_history")
            .map(|(_, value)| {
                value
                    .as_array()
                    .map(|items| items.to_vec())
                    .ok_or_else(|| format!("{where_}.assessment_history must be an array"))
            })
            .transpose()?
            .unwrap_or_default(),
        run_history: fields
            .iter()
            .find(|(key, _)| key == "run_history")
            .map(|(_, value)| {
                value
                    .as_array()
                    .map(|items| items.to_vec())
                    .ok_or_else(|| format!("{where_}.run_history must be an array"))
            })
            .transpose()?
            .unwrap_or_default(),
        id,
        capability,
        max_candidates,
        units,
    })
}

fn parse_requested_check(value: &Json, where_: &str) -> Result<RequestedCheck, String> {
    let fields = object(
        value,
        where_,
        &["id", "execution-capability", "scenarios", "units"],
    )?;
    let id = nonempty(fields, where_, "id")?;
    validate_check_id(&id).map_err(|detail| format!("{where_}.id {detail}"))?;
    let capability = nonempty(fields, where_, "execution-capability")?;
    validate_capability_address(&capability)
        .map_err(|detail| format!("{where_}.execution-capability {detail}"))?;
    let scenarios = match fields.iter().find(|(key, _)| key == "scenarios") {
        Some((_, value)) => parse_scenarios(value, &format!("{where_}.scenarios"))?,
        None => Vec::new(),
    };
    let units = match fields.iter().find(|(key, _)| key == "units") {
        Some((_, value)) => {
            let units = array(value, &format!("{where_}.units"))?
                .iter()
                .enumerate()
                .map(|(index, value)| parse_unit(value, &format!("{where_}.units[{index}]")))
                .collect::<Result<Vec<_>, _>>()?;
            validate_units(&units, &format!("{where_}.units"))?;
            units
        }
        None => whole_units(),
    };
    Ok(RequestedCheck {
        id,
        capability,
        scenarios,
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
            "verification_target",
            "verification_target_fingerprint",
            "plan",
            "adapters",
            "routes",
            "fingerprint",
        ],
    )?;
    exact_string(fields, "format", "$", LAUNCH_FORMAT)?;
    exact_integer(fields, "version", "$", VERSION)?;
    let verification_target =
        run::verification_target_from_json(required(fields, "verification_target", "$")?)?;
    let plan = run::plan_from_json(required(fields, "plan", "$")?)?;
    let route_values = array(required(fields, "routes", "$")?, "$.routes")?;
    let mut routes = Vec::new();
    for value in route_values {
        routes.push(run::launch_route_from_json(value)?);
    }
    let adapters = array(required(fields, "adapters", "$")?, "$.adapters")?
        .iter()
        .map(parse_launch_adapter)
        .collect::<Result<Vec<_>, _>>()?;
    let adapter = coordinator_identity(&adapters);
    Ok(LaunchPlan {
        operation: operation(fields, "$", "operation")?,
        planned_at_ms: integer(fields, "$", "planned_at_ms")?,
        verification_target,
        verification_target_fingerprint: fingerprint(
            fields,
            "$",
            "verification_target_fingerprint",
        )?,
        plan,
        adapter,
        adapters,
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
    if !matches!(
        request.verification_target,
        VerificationTarget::Source { .. }
            | VerificationTarget::Artifact { .. }
            | VerificationTarget::Deployment { .. }
    ) {
        return Err("new Run requests require source, artifact or deployment; historical Verification Target descriptors remain readable without relabeling".into());
    }
    if request.planned_at_ms > MAX_SAFE_INTEGER {
        return Err("$.planned_at_ms exceeds the safe-integer limit".into());
    }
    let verification_target_errors =
        run::validate_verification_target_component(&request.verification_target);
    if let Some(error) = verification_target_errors.first() {
        return Err(format!(
            "$.verification_target {}: {}",
            error.path, error.detail
        ));
    }
    if request.checks.is_empty() && request.challenges.is_empty() {
        return Err("$.checks and $.challenges must not both be empty".into());
    }
    ensure_sorted_unique(&request.checks, |check| check.id.as_str(), "$.checks")?;
    for (index, check) in request.checks.iter().enumerate() {
        validate_check_id(&check.id).map_err(|detail| format!("$.checks[{index}].id {detail}"))?;
        validate_capability_address(&check.capability)
            .map_err(|detail| format!("$.checks[{index}].capability {detail}"))?;
        if !check.scenarios.is_empty() {
            validate_scenarios(&check.scenarios, &format!("$.checks[{index}].scenarios"))?;
        }
        if !check.units.is_empty() {
            validate_units(&check.units, &format!("$.checks[{index}].units"))?;
        }
    }
    ensure_sorted_unique(
        &request.challenges,
        |challenge| challenge.id.as_str(),
        "$.challenges",
    )?;
    for (index, challenge) in request.challenges.iter().enumerate() {
        validate_id(&challenge.id, true)
            .map_err(|detail| format!("$.challenges[{index}].id {detail}"))?;
        validate_capability_address(&challenge.capability)
            .map_err(|detail| format!("$.challenges[{index}].capability {detail}"))?;
        if challenge.max_candidates == 0 || challenge.max_candidates > MAX_SAFE_INTEGER {
            return Err(format!(
                "$.challenges[{index}].max_candidates must be from 1 through the safe-integer limit"
            ));
        }
        validate_units(&challenge.units, &format!("$.challenges[{index}].units"))?;
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

fn valid_path_id(value: &str) -> bool {
    !value.is_empty() && value.split('/').all(valid_segment)
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
    let mut fields = vec![
        ("id", Json::str(&check.id)),
        ("execution-capability", Json::str(&check.capability)),
        (
            "scenarios",
            Json::Arr(check.scenarios.iter().map(Json::str).collect()),
        ),
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
    ];
    if check.scenarios.is_empty() {
        fields.retain(|(key, _)| *key != "scenarios");
    }
    if check.units.is_empty() {
        fields.retain(|(key, _)| *key != "units");
    }
    Json::obj(fields)
}

fn requested_challenge_json(challenge: &RequestedChallenge) -> Json {
    let mut fields = vec![
        ("id".into(), Json::str(&challenge.id)),
        (
            "execution-capability".into(),
            Json::str(&challenge.capability),
        ),
        (
            "max_candidates".into(),
            Json::Num(challenge.max_candidates as f64),
        ),
        (
            "units".into(),
            Json::Arr(
                challenge
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
    ];
    if let Some(record) = &challenge.assessment {
        fields.push(("assessment".into(), record.clone()));
    }
    if let Some(scope) = &challenge.scope_report {
        fields.push(("scope_report".into(), scope.clone()));
    }
    if !challenge.assessment_history.is_empty() {
        fields.push((
            "assessment_history".into(),
            Json::Arr(challenge.assessment_history.clone()),
        ));
    }
    if !challenge.run_history.is_empty() {
        fields.push((
            "run_history".into(),
            Json::Arr(challenge.run_history.clone()),
        ));
    }
    Json::Obj(fields)
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

fn parse_scenarios(value: &Json, where_: &str) -> Result<Vec<String>, String> {
    let values = array(value, where_)?;
    let mut scenarios = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let Json::Str(scenario) = value else {
            return Err(format!("{where_}[{index}] must be a string"));
        };
        scenarios.push(scenario.clone());
    }
    validate_scenarios(&scenarios, where_)?;
    Ok(scenarios)
}

fn validate_scenarios(scenarios: &[String], where_: &str) -> Result<(), String> {
    if scenarios.is_empty() {
        return Err(format!("{where_} must not be empty"));
    }
    ensure_sorted_unique(scenarios, String::as_str, where_)?;
    for scenario in scenarios {
        validate_entity_id(scenario)
            .map_err(|reason| format!("{where_} Scenario `{scenario}`: {reason}"))?;
    }
    Ok(())
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

pub fn whole_units() -> Vec<WorkUnit> {
    vec![WorkUnit {
        id: "whole".into(),
        parameters: BTreeMap::new(),
    }]
}

pub fn coordinator_identity(adapters: &[LaunchAdapter]) -> LaunchAdapter {
    if adapters.len() == 1 {
        return adapters[0].clone();
    }
    let identities = adapters
        .iter()
        .map(|adapter| run::LaunchAdapterIdentity {
            id: adapter.id.clone(),
            adapter_version: adapter.adapter_version.clone(),
            adapter_fingerprint: adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: adapter.configuration_fingerprint.clone(),
        })
        .collect::<Vec<_>>();
    let identity = run::coordinator_identity(&identities);
    LaunchAdapter {
        id: identity.id,
        adapter_version: identity.adapter_version,
        adapter_fingerprint: identity.adapter_fingerprint,
        descriptor_fingerprint: identity.descriptor_fingerprint,
        configuration_fingerprint: identity.configuration_fingerprint,
    }
}

pub fn adapter_launch(launch: &LaunchPlan, adapter: &LaunchAdapter) -> LaunchPlan {
    let routes = launch
        .routes
        .iter()
        .filter(|route| {
            route
                .capability
                .address
                .starts_with(&format!("{}/", adapter.id))
        })
        .cloned()
        .collect::<Vec<_>>();
    let checks = launch
        .plan
        .checks
        .iter()
        .filter(|check| {
            routes.iter().any(|r| {
                r.selection.kind == RouteSelectionKind::Check && r.selection.id == check.id
            })
        })
        .cloned()
        .collect();
    let challenges = launch
        .plan
        .challenges
        .iter()
        .filter(|check| {
            routes.iter().any(|r| {
                r.selection.kind == RouteSelectionKind::Challenge && r.selection.id == check.id
            })
        })
        .cloned()
        .collect();
    let plan = run::construct_plan(
        &launch.verification_target_fingerprint,
        launch.plan.model_fingerprint.clone(),
        launch.plan.required_execution_context.clone(),
        checks,
        challenges,
    )
    .expect("valid route partition");
    let mut child = LaunchPlan {
        operation: launch.operation,
        planned_at_ms: launch.planned_at_ms,
        verification_target: launch.verification_target.clone(),
        verification_target_fingerprint: launch.verification_target_fingerprint.clone(),
        plan,
        adapter: adapter.clone(),
        adapters: vec![adapter.clone()],
        routes,
        fingerprint: zero_fingerprint(),
    };
    child.fingerprint = launch_fingerprint(&child);
    child
}

fn resolve_assessment_challenge(
    model: &Model,
    configuration: &AdapterConfiguration,
    request: &PlanRequest,
    requested: &RequestedChallenge,
) -> Result<(ChallengeSelection, LaunchRoute, String), String> {
    let verification_challenge = model
        .verification_challenges()
        .find(|item| item.id == requested.id && !item.targets.is_empty())
        .ok_or("request does not select one compact machine Verification Challenge")?;
    let (kind, scope_kind, target_id) = if requested.assessment.is_some() {
        let record = requested.assessment.as_ref().unwrap();
        crate::assurance_review::validate_record(record)?;
        let kind_name = record
            .get("kind")
            .and_then(Json::as_str)
            .ok_or("assessment kind missing")?;
        let _ = match kind_name {
            "requirement-review" => (
                ChallengeTargetKind::RequirementReview,
                ChallengeScopeKind::RequirementReview,
            ),
            _ => {
                return Err("machine Verification Challenge targets only Requirement Review".into())
            }
        };
        let authority = crate::assurance_review::authority(model, "challenge-planning")?;
        let bundles = requested
            .run_history
            .iter()
            .map(|value| {
                run::parse("challenge.run_history", &value.to_string_pretty())
                    .map_err(|errors| format!("invalid assessment Run history: {errors:?}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let state = crate::assurance_review::assess_record(
            record,
            &authority,
            &requested.assessment_history,
            &bundles,
        );
        if state.status != "accepted" {
            return Err(format!(
                "assessment is not a current eligible target: {} {}",
                state.status,
                state.diagnostics.join("; ")
            ));
        }
        let requirement = record
            .get("requirement")
            .and_then(Json::as_str)
            .ok_or("assessment Requirement missing")?;
        let reviewed_at = record
            .get("reviewed_at_ms")
            .and_then(Json::as_num)
            .unwrap_or_default();
        for previous in &requested.assessment_history {
            crate::assurance_review::validate_record(previous)?;
            if previous.get("kind") == record.get("kind")
                && previous.get("requirement") == record.get("requirement")
                && previous.get("verification_target_fingerprint")
                    == record.get("verification_target_fingerprint")
                && previous
                    .get("reviewed_at_ms")
                    .and_then(Json::as_num)
                    .unwrap_or_default()
                    > reviewed_at
            {
                return Err(
                    "assessment was superseded by a newer record for the same target".into(),
                );
            }
        }
        if !verification_challenge
            .targets
            .iter()
            .any(|target| target == kind_name)
            || !verification_challenge
                .requirements
                .iter()
                .any(|id| id == requirement)
        {
            return Err(
                "assessment falls outside declared Verification Challenge target scope".into(),
            );
        }
        (
            ChallengeTargetKind::RequirementReview,
            ChallengeScopeKind::RequirementReview,
            requirement.to_string(),
        )
    } else {
        if !verification_challenge
            .targets
            .iter()
            .any(|target| target == "implementation")
        {
            return Err(
                "pre-review Verification Challenge must declare Targets: implementation".into(),
            );
        }
        if !requested.assessment_history.is_empty() {
            return Err("implementation Challenge cannot carry assessment history".into());
        }
        (
            ChallengeTargetKind::Implementation,
            ChallengeScopeKind::Implementation,
            verification_challenge.id.clone(),
        )
    };
    let scope = requested
        .scope_report
        .as_ref()
        .ok_or("machine Verification Challenge requires prepared source scope_report")?;
    if scope.get("format").and_then(Json::as_str) != Some("azimuth-challenge-source-scope")
        || scope.get("version").and_then(Json::as_num) != Some(2.0)
        || scope.get("status").and_then(Json::as_str) != Some("complete")
        || scope.get("verification_challenge").and_then(Json::as_str)
            != Some(&verification_challenge.id)
    {
        return Err(
            "scope report must be complete and identify the selected Verification Challenge".into(),
        );
    }
    let Json::Obj(fields) = scope else {
        return Err("scope report must be object".into());
    };
    let scope_payload = Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "fingerprint")
            .cloned()
            .collect(),
    );
    let scope_fingerprint = run::canonical_fingerprint(&scope_payload)?;
    if scope.get("fingerprint").and_then(Json::as_str) != Some(&scope_fingerprint) {
        return Err("scope report fingerprint mismatch".into());
    }
    let declaration = scope.get("request").ok_or("scope request missing")?;
    for (field, expected) in [
        ("requirements", &verification_challenge.requirements),
        ("mechanisms", &verification_challenge.mechanisms),
        ("checks", &verification_challenge.checks),
    ] {
        let actual = declaration
            .get(field)
            .and_then(Json::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Json::as_str)
                    .map(str::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if field == "requirements"
            && actual.is_empty()
            && !verification_challenge.mechanisms.is_empty()
        {
            continue;
        }
        if actual != expected.iter().cloned().collect::<BTreeSet<_>>() {
            return Err(format!(
                "scope {field} differs from Verification Challenge declaration"
            ));
        }
    }
    let (adapter, capability) = configuration
        .capability(&requested.capability)
        .ok_or("unknown Verification Challenge capability")?;
    let required_class = match request.operation {
        RunOperation::Execute => CapabilityClass::ChallengeExecute,
        RunOperation::Import => CapabilityClass::ChallengeImport,
    };
    if !capability.supports(required_class)
        || !capability
            .challenge_forms
            .contains(&verification_challenge.form)
    {
        return Err(
            "configured capability does not support Verification Challenge form and operation"
                .into(),
        );
    }
    let record_fingerprint = match &requested.assessment {
        Some(record) => crate::assurance_review::record_fingerprint(record)?,
        None => crate::assurance_review::implementation_challenge_fingerprint(
            &crate::assurance_review::authority(model, "challenge-planning")?,
            &verification_challenge.id,
            &scope_fingerprint,
            &run::verification_target_fingerprint(&request.verification_target),
        )?,
    };
    let verification_challenge_fingerprint =
        crate::fingerprint::verification_challenge_fingerprint(verification_challenge);
    let id = run::challenge_selection_id(
        &verification_challenge_fingerprint,
        kind,
        &record_fingerprint,
    );
    let mut inputs = Vec::new();
    let mut scope_inputs = Vec::new();
    for site in scope
        .get("sites")
        .and_then(Json::as_array)
        .ok_or("scope sites missing")?
    {
        let text = |key| {
            site.get(key)
                .and_then(Json::as_str)
                .ok_or_else(|| format!("scope site missing {key}"))
        };
        let relation = text("relationship")?;
        let entity = text("entity")?;
        let address = text("site")?;
        let file = text("file")?;
        let fingerprint = text("source_fingerprint")?;
        let (input_kind, semantic_kind, mappings) = match relation {
            "requirement" => (
                LaunchInputKind::Realization,
                ChallengeScopeKind::Realization,
                model
                    .realizes
                    .iter()
                    .filter(|item| {
                        item.requirement == entity
                            && item.site == address
                            && item.file == file
                            && item.source_fingerprint == fingerprint
                    })
                    .map(|item| (item.lang.clone(), item.source.clone()))
                    .collect::<Vec<_>>(),
            ),
            "mechanism" => (
                LaunchInputKind::MechanismImplementation,
                ChallengeScopeKind::MechanismImplementation,
                model
                    .mechanism_implementations
                    .iter()
                    .filter(|item| {
                        item.mechanism == entity
                            && item.site == address
                            && item.file == file
                            && item.source_fingerprint == fingerprint
                    })
                    .map(|item| (item.lang.clone(), item.source.clone()))
                    .collect::<Vec<_>>(),
            ),
            "check" => (
                LaunchInputKind::CheckImplementation,
                ChallengeScopeKind::CheckImplementation,
                model
                    .check_implementations
                    .iter()
                    .filter(|item| {
                        item.check == entity
                            && item.site == address
                            && item.file == file
                            && item.source_fingerprint == fingerprint
                    })
                    .map(|item| (item.lang.clone(), item.source.clone()))
                    .collect::<Vec<_>>(),
            ),
            _ => return Err("unsupported selected source relationship".into()),
        };
        let [(language, Some(source))] = mappings.as_slice() else {
            return Err(format!(
                "scope source mapping requires one current qualified identity: {address}"
            ));
        };
        let identity = source.key();
        inputs.push(
            run::construct_launch_input(
                input_kind,
                identity.clone(),
                fingerprint.into(),
                LaunchInputSource::Source {
                    file: file.into(),
                    language: language.clone(),
                    site: address.into(),
                },
            )
            .map_err(|error| error.detail)?,
        );
        scope_inputs.push(ChallengeScopeItem {
            kind: semantic_kind,
            id: identity,
            fingerprint: fingerprint.into(),
        });
    }
    inputs.sort();
    inputs.dedup();
    scope_inputs.sort();
    scope_inputs.dedup();
    let anchors = vec![ChallengeScopeItem {
        kind: scope_kind,
        id: target_id.clone(),
        fingerprint: record_fingerprint.clone(),
    }];
    let native_scope = run::construct_challenge_scope(anchors, scope_inputs)
        .map_err(|errors| format!("invalid challenge scope: {errors:?}"))?;
    let mut units = requested.units.clone();
    for unit in &mut units {
        match unit.parameters.get("scope_fingerprint") {
            Some(value) if value == &scope_fingerprint => {}
            _ => return Err("each Challenge unit must pin the prepared scope_fingerprint".into()),
        }
    }
    let selection = ChallengeSelection {
        id: id.clone(),
        verification_challenge: VerificationChallengeRef {
            id: verification_challenge.id.clone(),
            fingerprint: verification_challenge_fingerprint,
        },
        target: ChallengeTarget {
            kind,
            id: target_id,
            fingerprint: record_fingerprint,
        },
        lane: run::ChallengeLane::Gate,
        scope: native_scope,
        units,
    };
    let route = run::construct_launch_route(
        RouteSelection {
            kind: RouteSelectionKind::Challenge,
            id,
        },
        RouteCapability {
            address: requested.capability.clone(),
            class: request.operation.route_challenge_class(),
            challenge_form: Some(verification_challenge.form.clone()),
            fingerprint: capability.fingerprint.clone(),
        },
        inputs,
    )
    .map_err(|errors| format!("invalid Challenge launch route: {errors:?}"))?;
    Ok((selection, route, adapter.id.clone()))
}
