//! Strict D47/D48 Run planning and launch-plan identity.
//!
//! The planner consumes a complete already-loaded model. It never loads provider semantics, applies
//! a partial model selection. Provider routing is a separate launch layer over the unchanged D46
//! semantic Plan.

use crate::adapter::{AdapterConfiguration, CapabilityClass};
use crate::diag::validate_id;
use crate::json::Json;
use crate::model::{Model, SemanticChallengeScope, SemanticScopeComponent, SemanticScopeLocator};
use crate::run::{
    self, ChallengeLane, ChallengeScopeItem, ChallengeScopeKind, ChallengeSelection,
    ChallengeTarget, ChallengeTargetKind, ChallengerRef, CheckSelection, Implementation,
    LaunchInput, LaunchInputKind, LaunchInputSource, LaunchRoute, Plan, RouteCapability,
    RouteCapabilityClass, RouteSelection, RouteSelectionKind, Subject, WorkUnit,
};
use crate::validation::{resolve_challenge_plan, DecisionKind};
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
    pub challenges: Vec<RequestedChallenge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedCheck {
    pub id: String,
    pub capability: String,
    pub units: Vec<WorkUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedChallenge {
    pub id: String,
    pub capability: String,
    pub max_candidates: u64,
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
        if let Some(check_adapter) = &selected_adapter {
            if check_adapter != &adapter {
                errors.push("one launch plan cannot route through several adapters".into());
            }
        } else {
            selected_adapter = Some(adapter);
        }
    }
    routes.extend(challenge_routes);

    if !errors.is_empty() {
        return Err(planning_errors(errors));
    }
    let adapter_id = selected_adapter.expect("a valid request has a non-empty selection list");
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
    let mut selected_forms = BTreeMap::<(DecisionKind, String), (String, BTreeSet<String>)>::new();
    for requested in &request.challenges {
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
        let challengers = model
            .challengers()
            .filter(|challenger| challenger.id == authored.challenger)
            .collect::<Vec<_>>();
        let challenger = match challengers.as_slice() {
            [challenger] => *challenger,
            _ => {
                errors.push(format!(
                    "Challenge Plan `{}` does not name exactly one current Challenger",
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
                .contains(&challenger.form),
            standards
                .schedule
                .scheduled_challenges
                .contains(&challenger.form),
        ) {
            (true, false) => ChallengeLane::Gate,
            (false, true) => ChallengeLane::Scheduled,
            _ => {
                errors.push(format!(
                    "Challenge form `{}` must have exactly one schedule lane",
                    challenger.form
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
            || !capability.challenge_forms.contains(&challenger.form)
        {
            errors.push(format!(
                "capability `{}` must support `{}` and Challenge form `{}`",
                requested.capability,
                required_class.name(),
                challenger.form
            ));
            continue;
        }
        if adapter_id.as_ref().is_some_and(|id| id != &adapter.id) {
            errors.push("one launch plan cannot route through several adapters".into());
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
            if !challenger
                .required_scope
                .iter()
                .all(|required| present.contains(required))
            {
                errors.push(format!(
                    "Challenge Plan `{}` does not satisfy Challenger `{}` required scope",
                    requested.id, challenger.id
                ));
                continue;
            }
            let policy_id = match kind {
                DecisionKind::Qualification => {
                    let Some(binding) = model.evidence_bindings().find(|item| item.id == target_id)
                    else {
                        errors.push(format!("missing Evidence Binding `{target_id}`"));
                        continue;
                    };
                    if binding.context != request.required_context {
                        errors.push(format!(
                            "Qualification `{target_id}` context must equal required_context"
                        ));
                        continue;
                    }
                    binding.policy.clone()
                }
                DecisionKind::ClaimJudgment => {
                    let Some(judgment) = model.claim_judgments().find(|item| item.id == target_id)
                    else {
                        errors.push(format!("missing Claim Judgment `{target_id}`"));
                        continue;
                    };
                    judgment.policy.clone()
                }
            };
            selected_forms
                .entry((kind, target_fingerprint.clone()))
                .or_insert_with(|| (policy_id, BTreeSet::new()))
                .1
                .insert(challenger.form.clone());

            let challenger_fingerprint = crate::fingerprint::challenger_fingerprint(challenger);
            let target_kind = match kind {
                DecisionKind::Qualification => ChallengeTargetKind::Qualification,
                DecisionKind::ClaimJudgment => ChallengeTargetKind::ClaimJudgment,
            };
            let id = run::challenge_selection_id(
                &challenger_fingerprint,
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
                    challenger: ChallengerRef {
                        id: challenger.id.clone(),
                        fingerprint: challenger_fingerprint,
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
                    challenge_form: Some(challenger.form.clone()),
                    fingerprint: capability.fingerprint.clone(),
                },
                semantic_scope,
            };
            if let Some(existing) = pending.get_mut(&id) {
                if existing.selection.challenger != candidate.selection.challenger
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

    let mut selections = Vec::new();
    let mut routes = Vec::new();
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
        S::Claim => ChallengeScopeKind::Claim,
        S::Binding => ChallengeScopeKind::Binding,
        S::Qualification => ChallengeScopeKind::Qualification,
        S::ClaimJudgment => ChallengeScopeKind::ClaimJudgment,
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
    run::launch_fingerprint(
        match launch.operation {
            RunOperation::Execute => run::ProvenanceMode::Execute,
            RunOperation::Import => run::ProvenanceMode::Import,
        },
        launch.planned_at_ms,
        &launch.subject,
        &launch.subject_fingerprint,
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
        ("subject", run::subject_to_json(&request.subject)),
        (
            "required_context",
            string_map_json(&request.required_context),
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
            "challenges",
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
        subject,
        required_context,
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
        &["id", "capability", "max_candidates", "units"],
    )?;
    let id = nonempty(fields, where_, "id")?;
    validate_id(&id, true).map_err(|detail| format!("{where_}.id {detail}"))?;
    let capability = nonempty(fields, where_, "capability")?;
    validate_capability_address(&capability)
        .map_err(|detail| format!("{where_}.capability {detail}"))?;
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
        id,
        capability,
        max_candidates,
        units,
    })
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
    if request.checks.is_empty() && request.challenges.is_empty() {
        return Err("$.checks and $.challenges must not both be empty".into());
    }
    ensure_sorted_unique(&request.checks, |check| check.id.as_str(), "$.checks")?;
    for (index, check) in request.checks.iter().enumerate() {
        validate_id(&check.id, true).map_err(|detail| format!("$.checks[{index}].id {detail}"))?;
        validate_capability_address(&check.capability)
            .map_err(|detail| format!("$.checks[{index}].capability {detail}"))?;
        validate_units(&check.units, &format!("$.checks[{index}].units"))?;
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

fn requested_challenge_json(challenge: &RequestedChallenge) -> Json {
    Json::obj(vec![
        ("id", Json::str(&challenge.id)),
        ("capability", Json::str(&challenge.capability)),
        ("max_candidates", Json::Num(challenge.max_candidates as f64)),
        (
            "units",
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
