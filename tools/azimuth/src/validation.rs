//! Deterministic validation over the alpha 5 model.

use crate::json::Json;
use crate::model::Model;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingCategory {
    Intent,
    Realization,
    Verification,
    Mechanism,
    Assurance,
    Surface,
    Execution,
}

impl FindingCategory {
    pub fn name(self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Realization => "realization",
            Self::Verification => "verification",
            Self::Mechanism => "mechanism",
            Self::Assurance => "assurance",
            Self::Surface => "surface",
            Self::Execution => "execution",
        }
    }
}

macro_rules! define_finding_kinds {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum FindingKind { $($variant),+ }
        impl FindingKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

define_finding_kinds! {
    ConditionUnknownComponent,
    ConditionUnknownLeaf,
    ConditionTypeMismatch,
    ConditionConflict,
    CheckUnknownLeaf,
    Unrealized,
    DanglingRealization,
    DanglingRealizationEntry,
    DanglingVerificationScenario,
    UnresolvedRealizationBinding,
    EnforcementMismatch,
    MissingSurface,
    UnknownSurface,
    EnumeratorUnsoundOrUnderived,
    InvariantBreach,
    MissingRequiredRealization,
    DanglingRealizationObligation,
    DanglingMechanismImplementation,
    UnboundScenario,
    CheckWithoutBinding,
    MissingRequirementAssurance,
    MissingRealizationReview,
    UnknownReviewHead,
    VerificationElementSupportUnresolved,
    ScenarioCheckContributionUnresolved,
    CheckWithoutScenarioContribution,
    MechanismSupportUnresolved,
    UnimplementedCheck,
    DanglingCheckImplementation,
}

impl FindingKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::ConditionUnknownComponent => "condition-unknown-component",
            Self::ConditionUnknownLeaf => "condition-unknown-leaf",
            Self::ConditionTypeMismatch => "condition-type-mismatch",
            Self::ConditionConflict => "condition-conflict",
            Self::CheckUnknownLeaf => "check-unknown-leaf",
            Self::Unrealized => "unrealized",
            Self::DanglingRealization => "dangling-realization",
            Self::DanglingRealizationEntry => "dangling-realization-entry",
            Self::DanglingVerificationScenario => "dangling-verification-scenario",
            Self::UnresolvedRealizationBinding => "unresolved-realization-binding",
            Self::EnforcementMismatch => "enforcement-mismatch",
            Self::MissingSurface => "missing-surface",
            Self::UnknownSurface => "unknown-surface",
            Self::EnumeratorUnsoundOrUnderived => "enumerator-unsound-or-underived",
            Self::InvariantBreach => "invariant-breach",
            Self::MissingRequiredRealization => "missing-required-realization",
            Self::DanglingRealizationObligation => "dangling-realization-obligation",
            Self::DanglingMechanismImplementation => "dangling-mechanism-implementation",
            Self::UnboundScenario => "unbound-scenario",
            Self::CheckWithoutBinding => "check-without-binding",
            Self::MissingRequirementAssurance => "missing-requirement-assurance",
            Self::MissingRealizationReview => "missing-realization-review",
            Self::UnknownReviewHead => "unknown-review-head",
            Self::VerificationElementSupportUnresolved => "verification-element-support-unresolved",
            Self::ScenarioCheckContributionUnresolved => "scenario-check-contribution-unresolved",
            Self::CheckWithoutScenarioContribution => "check-without-scenario-contribution",
            Self::MechanismSupportUnresolved => "mechanism-support-unresolved",
            Self::UnimplementedCheck => "unimplemented-check",
            Self::DanglingCheckImplementation => "dangling-check-implementation",
        }
    }

    pub fn category(self) -> FindingCategory {
        match self {
            Self::ConditionUnknownComponent
            | Self::ConditionUnknownLeaf
            | Self::ConditionTypeMismatch
            | Self::ConditionConflict => FindingCategory::Intent,
            Self::CheckUnknownLeaf => FindingCategory::Verification,
            Self::Unrealized
            | Self::DanglingRealization
            | Self::MissingRequiredRealization
            | Self::DanglingRealizationObligation => FindingCategory::Realization,
            Self::DanglingRealizationEntry
            | Self::UnresolvedRealizationBinding
            | Self::EnforcementMismatch
            | Self::DanglingMechanismImplementation => FindingCategory::Mechanism,
            Self::MissingSurface
            | Self::UnknownSurface
            | Self::EnumeratorUnsoundOrUnderived
            | Self::InvariantBreach => FindingCategory::Surface,
            Self::MissingRequirementAssurance
            | Self::MissingRealizationReview
            | Self::UnknownReviewHead => FindingCategory::Assurance,
            _ => FindingCategory::Verification,
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Self::ConditionUnknownComponent => {
                "Declare the component in the project catalog `components`, or remove the condition."
            }
            Self::ConditionUnknownLeaf => {
                "Declare the leaf in the component's execution-context declaration, or remove the condition."
            }
            Self::ConditionTypeMismatch => {
                "Write a value of the type the leaf declares."
            }
            Self::CheckUnknownLeaf => {
                "Declare the component and leaf in the project catalog and its execution-context declaration, or remove the dependency from the Check's `Deployment:` contexts."
            }
            Self::ConditionConflict => {
                "Remove one of the conditions; all conditions hold together and a Scenario cannot override an inherited one."
            }
            Self::Unrealized => "Link production code that establishes the Requirement predicate.",
            Self::DanglingRealization => {
                "Retarget or remove the production link to the unknown Requirement."
            }
            Self::DanglingRealizationEntry => "Retarget or remove the realization entry.",
            Self::DanglingVerificationScenario => "Retarget or remove the Scenario verification declaration.",
            Self::UnresolvedRealizationBinding => "Bind the mechanism to one extracted artifact.",
            Self::EnforcementMismatch => {
                "Align enforcement with the artifact's derived properties."
            }
            Self::MissingSurface => "Declare the independently derived surface.",
            Self::UnknownSurface => "Declare the referenced surface in the workspace.",
            Self::EnumeratorUnsoundOrUnderived => "Run or repair every surface enumerator.",
            Self::InvariantBreach => "Discharge the invariant for the surface member.",
            Self::MissingRequiredRealization => "Add a realization from the required area.",
            Self::DanglingRealizationObligation => "Retarget or remove the realization obligation.",
            Self::DanglingMechanismImplementation => "Retarget the implementation to a mechanism.",
            Self::UnboundScenario => "Bind at least one deliberately enrolled Check to the Scenario.",
            Self::CheckWithoutBinding => "Add an Evidence Binding or remove the unused Check.",
            Self::MissingRealizationReview => "Ratify the Requirement with a committed current accepted Realization Review; it is an opinion about a code snapshot and never evidence.",
            Self::UnknownReviewHead => "Remove the review head: its Requirement is absent or does not declare that review.",
            Self::MissingRequirementAssurance => "Ratify the Requirement with a committed current accepted Verification Review plus the change's acceptance.json; deployment Requirement Status is computed after rollout and is never authored.",
            Self::VerificationElementSupportUnresolved => "Provide source/configuration support for the required verification element.",
            Self::ScenarioCheckContributionUnresolved => "Declare a source-authored Check contribution to this Scenario.",
            Self::CheckWithoutScenarioContribution => "Declare what Scenario this Check contributes to, or remove the unsupported Check.",
            Self::MechanismSupportUnresolved => "Provide implementation support for the selected mechanism.",
            Self::UnimplementedCheck => {
                "Mark at least one stable source implementation of the Check."
            }
            Self::DanglingCheckImplementation => "Retarget the source marker to a declared Check.",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: Severity,
    pub requirement: Option<String>,
    pub entity_kind: Option<crate::diag::EntityKind>,
    pub path: String,
    pub line: usize,
    pub detail: String,
}

impl Finding {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("kind", Json::str(self.kind.name())),
            ("category", Json::str(self.kind.category().name())),
            ("severity", Json::str(self.severity.name())),
            (
                "entity_kind",
                self.entity_kind
                    .map(|kind| Json::str(kind.name()))
                    .unwrap_or(Json::Null),
            ),
            (
                "requirement",
                self.requirement
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            ("file", Json::str(&self.path)),
            ("line", Json::Num(self.line as f64)),
            ("detail", Json::str(&self.detail)),
            ("help", Json::str(self.kind.help())),
        ])
    }
}

fn condition_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in &model.specs {
        for requirement in &spec.requirements {
            let mut report = |kind: FindingKind, line: usize, detail: String| {
                findings.push(Finding {
                    kind,
                    severity: Severity::Error,
                    requirement: Some(requirement.id.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: spec.path.clone(),
                    line,
                    detail,
                });
            };
            let scenario_conditions = requirement
                .scenarios
                .iter()
                .flat_map(|scenario| &scenario.conditions);
            for condition in requirement.conditions.iter().chain(scenario_conditions) {
                let Some(component) = model.component(&condition.component) else {
                    report(
                        FindingKind::ConditionUnknownComponent,
                        condition.line,
                        format!(
                            "condition `{}` names component `{}` which the project catalog does not declare",
                            condition.render(),
                            condition.component
                        ),
                    );
                    continue;
                };
                let Some(declared) = component
                    .leaves
                    .as_ref()
                    .and_then(|leaves| leaves.get(&condition.leaf))
                else {
                    report(
                        FindingKind::ConditionUnknownLeaf,
                        condition.line,
                        format!(
                            "condition `{}` names leaf `{}` which component `{}` does not declare",
                            condition.render(),
                            condition.leaf,
                            condition.component
                        ),
                    );
                    continue;
                };
                if condition.value.leaf_type() != *declared {
                    report(
                        FindingKind::ConditionTypeMismatch,
                        condition.line,
                        format!(
                            "condition `{}` compares {} leaf `{}` of `{}` with a {} value",
                            condition.render(),
                            declared.name(),
                            condition.leaf,
                            condition.component,
                            condition.value.leaf_type().name()
                        ),
                    );
                }
            }
            let conflicting = |first: &crate::model::Condition,
                               second: &crate::model::Condition| {
                first.component == second.component
                    && first.leaf == second.leaf
                    && first.value != second.value
            };
            for (index, later) in requirement.conditions.iter().enumerate() {
                if let Some(earlier) = requirement.conditions[..index]
                    .iter()
                    .find(|earlier| conflicting(earlier, later))
                {
                    report(
                        FindingKind::ConditionConflict,
                        later.line,
                        format!(
                            "condition `{}` conflicts with `{}` on line {}",
                            later.render(),
                            earlier.render(),
                            earlier.line
                        ),
                    );
                }
            }
            for scenario in &requirement.scenarios {
                for (index, later) in scenario.conditions.iter().enumerate() {
                    let earlier = requirement
                        .conditions
                        .iter()
                        .chain(&scenario.conditions[..index])
                        .find(|earlier| conflicting(earlier, later));
                    if let Some(earlier) = earlier {
                        report(
                            FindingKind::ConditionConflict,
                            later.line,
                            format!(
                                "condition `{}` of Scenario `{}` conflicts with `{}` on line {}",
                                later.render(),
                                scenario.id,
                                earlier.render(),
                                earlier.line
                            ),
                        );
                    }
                }
            }
        }
    }
    findings
}

fn check_leaf_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    for document in &model.account_verifications {
        for check in &document.checks {
            let Some(policy) = check
                .inputs
                .get("deployment-policy")
                .and_then(|raw| crate::run::strict_json("Deployment", raw).ok())
            else {
                continue;
            };
            let Ok(dependencies) =
                crate::deployment_status::context_dependencies(&policy, "contexts")
            else {
                continue;
            };
            for (component, leaf) in dependencies {
                if model.component_leaf_type(&component, &leaf).is_some() {
                    continue;
                }
                findings.push(Finding {
                    kind: FindingKind::CheckUnknownLeaf,
                    severity: Severity::Error,
                    requirement: Some(check.requirement.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: document.path.clone(),
                    line: 1,
                    detail: format!(
                        "Check `{}` depends on leaf `{leaf}` of component `{component}`, which the project catalog and its execution-context declaration do not declare",
                        check.definition.id
                    ),
                });
            }
        }
    }
    findings
}

pub fn validate(model: &Model) -> Vec<Finding> {
    let mut findings = condition_findings(model);
    findings.extend(check_leaf_findings(model));
    for requirement in model.requirements() {
        if requirement.requirement.realization.is_some() {
            findings.push(Finding {
                kind: FindingKind::MissingRealizationReview,
                severity: Severity::Error,
                requirement: Some(requirement.id()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                path: requirement.spec.path.clone(),
                line: requirement.requirement.line,
                detail: "Requirement declares `Realization: reviewed` and requires a ratified current accepted Realization Review".into(),
            });
        }
        if requirement.requirement.realization.is_none() {
            continue;
        }
        if !model
            .realizes
            .iter()
            .any(|site| site.requirement == requirement.requirement.id)
        {
            findings.push(Finding {
                kind: FindingKind::Unrealized,
                severity: Severity::Error,
                requirement: Some(requirement.id()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                path: requirement.spec.path.clone(),
                line: requirement.requirement.line,
                detail: "no production code realizes this Requirement".into(),
            });
        }
    }
    for head in &model.review_heads {
        let declared = model
            .requirements()
            .find(|view| view.requirement.id == head.requirement)
            .is_some_and(|view| match head.kind {
                crate::review_heads::HeadKind::Verification => {
                    view.requirement.verification.is_some()
                }
                crate::review_heads::HeadKind::Realization => {
                    view.requirement.realization.is_some()
                }
            });
        if !declared {
            findings.push(Finding {
                kind: FindingKind::UnknownReviewHead,
                severity: Severity::Error,
                requirement: Some(head.requirement.clone()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                path: head.path.clone(),
                line: head.line,
                detail: format!(
                    "{} head names a Requirement that is absent or does not declare that review",
                    head.kind.record_kind()
                ),
            });
        }
    }
    for site in &model.realizes {
        if !model.has_requirement(&site.requirement) {
            findings.push(Finding {
                kind: FindingKind::DanglingRealization,
                severity: Severity::Error,
                requirement: Some(site.requirement.clone()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                path: site.file.clone(),
                line: 0,
                detail: format!("`{}` realizes a Requirement that does not exist", site.site),
            });
        }
    }
    for implementation in &model.mechanism_implementations {
        let declared = model.realizations.iter().any(|realization| {
            realization
                .entries
                .iter()
                .flat_map(|entry| &entry.mechanisms)
                .any(|mechanism| mechanism.id == implementation.mechanism)
        }) || model.account_realizations.iter().any(|realization| {
            realization
                .declarations
                .iter()
                .any(|item| account_mechanism_declared(item, &implementation.mechanism))
        });
        if !declared {
            findings.push(Finding {
                kind: FindingKind::DanglingMechanismImplementation,
                severity: Severity::Error,
                requirement: Some(implementation.mechanism.clone()),
                entity_kind: Some(crate::diag::EntityKind::Mechanism),
                path: implementation.file.clone(),
                line: 0,
                detail: "implementation names no realization-owned mechanism".into(),
            });
        }
    }
    findings.extend(verification_findings(model));
    findings.extend(realization_findings(model));
    findings.extend(realization_obligation_findings(model));
    findings.extend(account_findings(model));
    findings.extend(surface_findings(model));
    findings.sort_by(|left, right| {
        (
            &left.path,
            left.line,
            left.kind.name(),
            &left.requirement,
            &left.detail,
        )
            .cmp(&(
                &right.path,
                right.line,
                right.kind.name(),
                &right.requirement,
                &right.detail,
            ))
    });
    findings
}

fn verification_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    let authored_checks = || {
        model
            .account_verifications
            .iter()
            .flat_map(|file| &file.checks)
    };
    for scenario in model.scenarios() {
        if scenario.requirement.verification.is_some()
            && !authored_checks().any(|check| {
                check
                    .bindings
                    .iter()
                    .any(|binding| binding.scenario == scenario.id())
            })
        {
            findings.push(Finding {
                kind: FindingKind::UnboundScenario,
                severity: Severity::Error,
                requirement: Some(scenario.id()),
                entity_kind: Some(crate::diag::EntityKind::Scenario),
                path: scenario.spec.path.clone(),
                line: scenario.scenario.line,
                detail: "Scenario of a Requirement that declares `Verification:` has no Evidence Binding".into(),
            });
        }
    }
    for requirement in model.requirements() {
        if requirement.requirement.verification.is_some() {
            findings.push(simple(
                FindingKind::MissingRequirementAssurance,
                &requirement.spec.path,
                requirement.requirement.line,
                Some(requirement.id()),
                Some(crate::diag::EntityKind::Requirement),
                "Requirement declares `Verification: reviewed` and requires a ratified current accepted Verification Review; definitions alone do not establish one".into(),
            ));
        }
    }
    for check in authored_checks() {
        let definition = &check.definition;
        if check.bindings.is_empty() {
            findings.push(simple(
                FindingKind::CheckWithoutBinding,
                &definition.path,
                definition.line,
                None,
                None,
                format!("Check `{}` has no Evidence Binding", definition.id),
            ));
        }
    }
    for implementation in &model.check_implementations {
        if !model.checks().any(|check| check.id == implementation.check) {
            findings.push(simple(
                FindingKind::DanglingCheckImplementation,
                &implementation.file,
                0,
                None,
                None,
                format!(
                    "source implementation names unknown Check `{}`",
                    implementation.check
                ),
            ));
        }
    }
    findings
}

fn simple(
    kind: FindingKind,
    path: &str,
    line: usize,
    requirement: Option<String>,
    entity_kind: Option<crate::diag::EntityKind>,
    detail: String,
) -> Finding {
    Finding {
        kind,
        severity: Severity::Error,
        entity_kind: requirement.as_ref().and(entity_kind),
        requirement,
        path: path.to_string(),
        line,
        detail,
    }
}

fn realization_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();

    for realization in &model.realizations {
        let Some(spec) = model.specs.iter().find(|s| s.id == realization.spec) else {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationEntry,
                severity: Severity::Error,
                requirement: None,
                entity_kind: None,
                path: realization.path.clone(),
                line: 1,
                detail: format!(
                    "realizations spec `{}`, which does not exist",
                    realization.spec
                ),
            });
            continue;
        };
        for entry in &realization.entries {
            let id = entry.target.id();
            let requirement = spec
                .requirements
                .iter()
                .find(|requirement| requirement.id == id);
            if requirement.is_none() {
                findings.push(Finding {
                    kind: FindingKind::DanglingRealizationEntry,
                    severity: Severity::Error,
                    requirement: Some(id.to_string()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: realization.path.clone(),
                    line: entry.line,
                    detail: "names a Requirement that does not exist".into(),
                });
            }

            for mechanism in &entry.mechanisms {
                if let Some(requirement) = requirement {
                    for scenario in &mechanism.scenarios {
                        if !requirement
                            .scenarios
                            .iter()
                            .any(|candidate| candidate.id == *scenario)
                        {
                            findings.push(Finding {
                                kind: FindingKind::DanglingRealizationEntry,
                                severity: Severity::Error,
                                requirement: Some(id.to_string()),
                                entity_kind: Some(crate::diag::EntityKind::Requirement),
                                path: realization.path.clone(),
                                line: mechanism.line,
                                detail: format!(
                                    "mechanism `{}` names Scenario `{scenario}` that does not exist under Requirement `{id}`",
                                    mechanism.id
                                ),
                            });
                        }
                    }
                }
                let bindings = model.mechanism_bindings(&realization.spec, mechanism);
                if bindings.is_empty() || (mechanism.binding.is_some() && bindings.len() != 1) {
                    findings.push(Finding {
                        kind: FindingKind::UnresolvedRealizationBinding,
                        severity: Severity::Error,
                        requirement: Some(id.to_string()),
                        entity_kind: Some(crate::diag::EntityKind::Requirement),
                        path: realization.path.clone(),
                        line: mechanism.line,
                        detail: if bindings.is_empty() {
                            format!(
                                "mechanism `{}` has neither an explicit artifact binding nor one extractor-resolved implementation",
                                mechanism.id
                            )
                        } else {
                            format!("mechanism `{}` mixes an explicit binding with extractor-resolved sites", mechanism.id)
                        },
                    });
                    continue;
                }
                for binding in bindings {
                    let artifacts = model
                        .artifacts
                        .iter()
                        .filter(|a| a.id == binding)
                        .collect::<Vec<_>>();
                    let [artifact] = artifacts.as_slice() else {
                        findings.push(Finding {
                            kind: FindingKind::UnresolvedRealizationBinding,
                            severity: Severity::Error,
                            requirement: Some(id.to_string()),
                            entity_kind: Some(crate::diag::EntityKind::Requirement),
                            path: realization.path.clone(),
                            line: mechanism.line,
                            detail: format!(
                            "binding `{binding}` does not resolve to exactly one emitted artifact"
                        ),
                        });
                        continue;
                    };

                    let mut mismatches = Vec::new();
                    if let Some(expected) = mechanism.expected_unique {
                        if artifact.unique != Some(expected) {
                            mismatches.push(format!(
                                "expected unique={expected}, found {:?}",
                                artifact.unique
                            ));
                        }
                    }
                    if !mechanism.expected_columns.is_empty()
                        && artifact.columns != mechanism.expected_columns
                    {
                        mismatches.push(format!(
                            "expected columns {}, found {}",
                            mechanism.expected_columns.join(","),
                            artifact.columns.join(",")
                        ));
                    }
                    if let Some(expected) = &mechanism.expected_predicate {
                        if artifact.predicate.as_ref() != Some(expected) {
                            mismatches.push(format!(
                                "expected predicate `{expected}`, found `{}`",
                                artifact.predicate.as_deref().unwrap_or("<none>")
                            ));
                        }
                    }

                    let mismatch = match (mechanism.kind, artifact.kind.as_str()) {
                    (crate::realization::Enforcement::Constraint, "database-index") => (artifact.unique
                        != Some(true))
                    .then_some("a non-unique index does not reject duplicate state".to_string()),
                    (kind, "database-index") if kind != crate::realization::Enforcement::Constraint => {
                        Some(format!(
                            "a database index cannot establish `{}` enforcement",
                            kind.name()
                        ))
                    }
                    (crate::realization::Enforcement::Type, "dotnet-method") => Some(
                        "type enforcement must bind the type, not one method on it".to_string(),
                    ),
                    (crate::realization::Enforcement::ChokePoint, "dotnet-type") => Some(
                        "choke-point enforcement must bind the operation, not its containing type"
                            .to_string(),
                    ),
                    _ => None,
                };
                    if let Some(detail) = mismatch {
                        mismatches.push(detail);
                    }
                    if !mismatches.is_empty() {
                        findings.push(Finding {
                            kind: FindingKind::EnforcementMismatch,
                            severity: Severity::Error,
                            requirement: Some(id.to_string()),
                            entity_kind: Some(crate::diag::EntityKind::Requirement),
                            path: realization.path.clone(),
                            line: mechanism.line,
                            detail: format!("binding `{}`: {}", binding, mismatches.join("; ")),
                        });
                    }
                }
            }
        }
    }

    findings
}

/// Requirements whose domain is a set of sites.
///
/// Membership has two sources, and the second exists because the first is not enough.
///
/// A site joins by realizing any requirement in the named spec — but that only reaches sites somebody
/// already tagged, so a file carrying no tags at all can never be a member. That is the enumerator
/// failure: an enumerator drawn from annotations reproduces the very bug the rule prevents and
/// reports green. A project's extractor may therefore emit `class_members` derived from what the
/// build produced — a route table, a container, a migration set — and those join too, whether
/// or
/// not anyone annotated them.
///
/// Identity differs between the two, which is deliberate. A tag-derived member is a named site in a
/// file and discharges only at that site. An emitted member *is* the file, because the enumerator
/// names files, and a discharge anywhere in it discharges the member.
///
/// Discharge is a `realizes` tag naming the invariant. No new tag: one requirement type parameterized by
/// domain means one of everything downstream.
///
/// **Limitation, stated rather than hidden.** This verifies the weakest rung of the enforcement
/// ladder — a guard at every site. A choke point that every member routes through would show as
/// N−1
/// breaches, which is exactly the defect the enforcement ladder names. Crediting a choke point
/// needs
/// call-graph analysis, which belongs to the extractor rather than derived-model validation.
fn surface_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();

    for spec in &model.specs {
        for requirement in &spec.requirements {
            if requirement.domain != crate::model::Domain::Sites {
                continue;
            }
            let requirement_id = requirement.id.clone();
            let Some(over) = &requirement.over else {
                findings.push(Finding {
                    kind: FindingKind::MissingSurface,
                    severity: Severity::Error,
                    requirement: Some(requirement_id),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: "site-domain requirement declares no `Over:` surface".into(),
                });
                continue;
            };
            let Some(surface) = model.workspace.surface(over) else {
                findings.push(Finding {
                    kind: FindingKind::UnknownSurface,
                    severity: Severity::Error,
                    requirement: Some(requirement_id),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: format!(
                        "`Over: {over}` names no surface in {}",
                        model.workspace.path
                    ),
                });
                continue;
            };

            let missing_contributions = surface
                .contributions
                .iter()
                .filter(|contribution| {
                    !model.enumerations.iter().any(|enumeration| {
                        enumeration.class == surface.id
                            && enumeration.kind == contribution.enumerator
                            && enumeration.identity.as_ref().is_some_and(|identity| {
                                identity.area == contribution.area
                                    && identity.mount == contribution.mount
                            })
                    })
                })
                .collect::<Vec<_>>();
            if !missing_contributions.is_empty() {
                let missing = missing_contributions
                    .iter()
                    .map(|item| format!("{}:{} via {}", item.area, item.mount, item.enumerator))
                    .collect::<Vec<_>>()
                    .join(", ");
                findings.push(Finding {
                    kind: FindingKind::EnumeratorUnsoundOrUnderived,
                    severity: Severity::Error,
                    requirement: Some(requirement_id),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: format!(
                        "surface `{}` has no successful witness for contribution(s) {missing}; tag-derived membership is not complete",
                        surface.id
                    ),
                });
                continue;
            }

            let class_spec = model
                .specs
                .iter()
                .find(|candidate| candidate.id == surface.id);
            let behavioural: Vec<&str> = class_spec
                .into_iter()
                .flat_map(|class_spec| &class_spec.requirements)
                .filter(|requirement| requirement.domain == crate::model::Domain::Behaviour)
                .map(|requirement| requirement.id.as_str())
                .collect();

            // (site, file, by_file): `by_file` marks a member the extractor enumerated, whose
            // identity is the file rather than a named site inside it.
            let mut members: Vec<(&str, &str, bool)> = model
                .realizes
                .iter()
                .filter(|site| {
                    site.spec == surface.id && behavioural.contains(&site.requirement.as_str())
                })
                .map(|site| (site.site.as_str(), site.file.as_str(), false))
                .collect();

            members.extend(
                model
                    .class_members
                    .iter()
                    .filter(|m| m.class == surface.id)
                    .map(|m| (m.site.as_str(), m.file.as_str(), true)),
            );

            members.sort();
            members.dedup();

            let discharges: Vec<(&str, &str)> = model
                .realizes
                .iter()
                .filter(|site| site.requirement == requirement.id)
                .map(|site| (site.site.as_str(), site.file.as_str()))
                .collect();

            for (site, file, by_file) in members {
                let discharged = if by_file {
                    discharges.iter().any(|(_, f)| *f == file)
                } else {
                    discharges.iter().any(|(s, f)| *s == site && *f == file)
                };
                if discharged {
                    continue;
                }
                findings.push(Finding {
                    kind: FindingKind::InvariantBreach,
                    severity: Severity::Error,
                    requirement: Some(requirement.id.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: file.to_string(),
                    line: 0,
                    detail: format!("`{site}` is in the class and discharges nothing"),
                });
            }
        }
    }

    findings
}

fn realization_obligation_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    for realization in &model.account_realizations {
        for declaration in &realization.declarations {
            if declaration.kind != crate::account_realization::DeclarationKind::Requirement {
                continue;
            }
            let spec = realization.spec.as_str();
            let requirement = declaration.id.as_str();
            if model.workspace.obligation(spec, requirement).is_some() {
                findings.push(Finding {
                    kind: FindingKind::DanglingRealizationObligation,
                    severity: Severity::Error,
                    requirement: Some(declaration.id.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: realization.path.clone(),
                    line: declaration.line,
                    detail: "Requirement Areas are authored in both realization and workspace; remove the workspace realization obligation".into(),
                });
            }
        }
    }
    for obligation in &model.realization_obligations() {
        let realization_location =
            model.account_realization_for_requirement(&obligation.spec, &obligation.requirement);
        let path = realization_location
            .map_or(model.workspace.path.as_str(), |(realization, _)| {
                realization.path.as_str()
            });
        let line = realization_location.map_or(0, |(_, declaration)| declaration.line);
        let Some(requirement) = model.find_requirement(&obligation.requirement) else {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationObligation,
                severity: Severity::Error,
                requirement: Some(obligation.requirement.clone()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                path: path.to_string(),
                line,
                detail: "realization obligation names a requirement that does not exist".into(),
            });
            continue;
        };
        if requirement.requirement.domain != crate::model::Domain::Behaviour
            || requirement.requirement.realization.is_none()
        {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationObligation,
                severity: Severity::Error,
                requirement: Some(requirement.id()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                path: path.to_string(),
                line,
                detail:
                    "area obligations apply only to behavioral requirements that declare `Realization:`"
                        .into(),
            });
            continue;
        }

        for area in &obligation.areas {
            let realized = model.realizes.iter().any(|site| {
                site.requirement == obligation.requirement
                    && site
                        .source
                        .as_ref()
                        .map(|source| source.area.as_str())
                        .or_else(|| {
                            model
                                .workspace
                                .area_for_file(&site.file)
                                .map(|declared| declared.id.as_str())
                        })
                        == Some(area.as_str())
            });
            if !realized {
                findings.push(Finding {
                    kind: FindingKind::MissingRequiredRealization,
                    severity: Severity::Error,
                    requirement: Some(requirement.id()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: path.to_string(),
                    line,
                    detail: format!(
                        "required area `{area}` has no realization of this requirement"
                    ),
                });
            }
        }
    }
    findings
}

fn account_mechanism_declared(item: &crate::account_realization::Declaration, id: &str) -> bool {
    (item.kind == crate::account_realization::DeclarationKind::Mechanism && item.id == id)
        || item
            .children
            .iter()
            .any(|child| account_mechanism_declared(child, id))
}

fn account_mechanism_findings(
    model: &Model,
    realization: &crate::account_realization::AccountRealization,
    item: &crate::account_realization::Declaration,
    enclosing_requirement: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    if item.kind == crate::account_realization::DeclarationKind::Scenario
        && !model
            .find_scenario(&item.id)
            .is_some_and(|view| enclosing_requirement == Some(view.requirement.id.as_str()))
    {
        findings.push(Finding {
            kind: FindingKind::DanglingRealizationEntry,
            severity: Severity::Error,
            requirement: Some(item.id.clone()),
            entity_kind: Some(crate::diag::EntityKind::Scenario),
            path: realization.path.clone(),
            line: item.line,
            detail: format!(
                "Scenario realization `{}` names no Scenario owned by its enclosing Requirement",
                item.id
            ),
        });
    }
    if item.kind == crate::account_realization::DeclarationKind::Mechanism {
        for scenario in &item.scenarios {
            if model.find_scenario(scenario).is_none() {
                findings.push(Finding {
                    kind: FindingKind::DanglingRealizationEntry,
                    severity: Severity::Error,
                    requirement: Some(scenario.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Scenario),
                    path: realization.path.clone(),
                    line: item.line,
                    detail: format!(
                        "Mechanism `{}` names unknown Scenario `{scenario}`",
                        item.id
                    ),
                });
            }
        }
        {
            let mechanism = item.id.as_str();
            let supported = model
                .mechanism_implementations
                .iter()
                .any(|implementation| implementation.mechanism == mechanism);
            if !supported {
                findings.push(Finding {
                    kind: FindingKind::MechanismSupportUnresolved,
                    severity: Severity::Error,
                    requirement: None,
                    entity_kind: None,
                    path: realization.path.clone(),
                    line: item.line,
                    detail: format!(
                        "selected Mechanism `{}` has no extracted implementation",
                        item.id
                    ),
                });
            }
        }
    }
    for child in &item.children {
        let owner = if item.kind == crate::account_realization::DeclarationKind::Requirement {
            Some(item.id.as_str())
        } else {
            enclosing_requirement
        };
        account_mechanism_findings(model, realization, child, owner, findings);
    }
}

fn account_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    for realization in &model.account_realizations {
        for item in &realization.declarations {
            account_mechanism_findings(model, realization, item, None, &mut findings);
        }
    }
    for verification in &model.account_verifications {
        for check in &verification.checks {
            if !crate::account_model::check_has_implementation(
                check,
                model.check_implementations.iter(),
            ) {
                findings.push(Finding {
                    kind: FindingKind::UnimplementedCheck,
                    severity: Severity::Error,
                    requirement: Some(check.requirement.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    path: verification.path.clone(),
                    line: check.definition.line,
                    detail: format!(
                        "document-authored Check `{}` has no stable extracted implementation",
                        check.definition.id
                    ),
                });
            }
        }
        for scope in &verification.requirements {
            let Some(target) = model.find_requirement(&scope.requirement) else {
                continue;
            };
            for scenario in verification
                .scenarios
                .iter()
                .filter(|scenario| scenario.requirement == scope.requirement)
            {
                if !model
                    .find_scenario(&scenario.scenario)
                    .is_some_and(|view| view.requirement.id == scenario.requirement)
                {
                    findings.push(Finding {
                        kind: FindingKind::DanglingVerificationScenario,
                        severity: Severity::Error,
                        requirement: Some(scope.requirement.clone()),
                        entity_kind: Some(crate::diag::EntityKind::Requirement),
                        path: verification.path.clone(),
                        line: scenario.line,
                        detail: format!(
                            "Scenario verification `{}` names no Scenario owned by Requirement `{}`",
                            scenario.scenario, scenario.requirement
                        ),
                    });
                }
            }
            for element in &scope.required_elements {
                let qualified = format!("{}#{element}", verification.owner);
                let supported = model
                    .account_supports
                    .iter()
                    .flat_map(|support| &support.elements)
                    .any(|support| {
                        support.requirement == scope.requirement && support.element == qualified
                    });
                if !supported {
                    findings.push(Finding {
                        kind: FindingKind::VerificationElementSupportUnresolved,
                        severity: Severity::Error,
                        requirement: Some(scope.requirement.clone()),
                        entity_kind: Some(crate::diag::EntityKind::Requirement),
                        path: verification.path.clone(),
                        line: scope.line,
                        detail: format!("required verification Element `{qualified}` has no extracted source/configuration support"),
                    });
                }
            }
            for scenario in &target.requirement.scenarios {
                let id = scenario.id.clone();
                let contributed = model
                    .account_verifications
                    .iter()
                    .flat_map(|file| &file.checks)
                    .any(|check| {
                        check.scenarios.contains(&id)
                            && crate::account_model::check_has_implementation(
                                check,
                                model.check_implementations.iter(),
                            )
                    })
                    || model
                        .account_supports
                        .iter()
                        .flat_map(|support| &support.contributions)
                        .any(|contribution| contribution.scenario == id);
                if !contributed {
                    findings.push(Finding {
                        kind: FindingKind::ScenarioCheckContributionUnresolved,
                        severity: Severity::Error,
                        requirement: Some(scope.requirement.clone()),
                        entity_kind: Some(crate::diag::EntityKind::Requirement),
                        path: verification.path.clone(),
                        line: scope.line,
                        detail: format!("Scenario `{id}` has no extracted Check contribution"),
                    });
                }
            }
        }
    }
    for support in &model.account_supports {
        for check in &support.checks {
            if !model
                .account_supports
                .iter()
                .flat_map(|item| &item.contributions)
                .any(|contribution| contribution.check == check.id)
            {
                findings.push(Finding {
                    kind: FindingKind::CheckWithoutScenarioContribution,
                    severity: Severity::Error,
                    requirement: None,
                    entity_kind: None,
                    path: support.path.clone(),
                    line: 0,
                    detail: format!("Check `{}` has no explicit Scenario contribution", check.id),
                });
            }
        }
    }
    findings
}

pub struct Summary {
    pub requirements: usize,
    pub errors: usize,
    pub warnings: usize,
}

pub fn summarize(model: &Model, findings: &[Finding]) -> Summary {
    Summary {
        requirements: model.requirement_count(),
        errors: findings
            .iter()
            .filter(|h| h.severity == Severity::Error)
            .count(),
        warnings: findings
            .iter()
            .filter(|h| h.severity == Severity::Warning)
            .count(),
    }
}

pub fn counts_by_kind(findings: &[Finding]) -> Vec<(&'static str, usize)> {
    FindingKind::ALL
        .iter()
        .map(|k| (k.name(), findings.iter().filter(|h| h.kind == *k).count()))
        .filter(|(_, n)| *n > 0)
        .collect()
}
