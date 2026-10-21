//! Deterministic validation and Challenge Plan resolution over the alpha 5 model.

use crate::json::Json;
use crate::model::{Criticality, Model};
use crate::verification::{
    ChallengeDomain, ChallengePlan, MethodQualificationVerdict, RequirementJudgmentVerdict,
    Selector, SemanticScopeKind,
};
use std::collections::{BTreeMap, BTreeSet};

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
    Judgment,
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
            Self::Judgment => "judgment",
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
    Unclassified,
    Unrealized,
    DanglingRealization,
    DanglingDesignEntry,
    DanglingVerificationScenario,
    UndeclaredMechanism,
    UnresolvedDesignBinding,
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
    BindingMissingCheck,
    BindingMissingScenario,
    BindingMissingPolicy,
    MissingMethodQualification,
    DanglingMethodQualification,
    RejectedMethodQualification,
    StaleMethodQualification,
    MissingApplicabilityDecision,
    DanglingApplicabilityDecision,
    RejectedApplicabilityDecision,
    StaleApplicabilityDecision,
    MissingRequirementJudgment,
    PendingRequirementReview,
    MissingRequirementAssurance,
    InvalidRequirementReview,
    VerificationElementSupportUnresolved,
    ScenarioCheckContributionUnresolved,
    CheckWithoutScenarioContribution,
    MechanismSupportUnresolved,
    RejectedRequirementJudgment,
    StaleRequirementJudgment,
    InvalidRequirementJudgment,
    UnimplementedCheck,
    DanglingCheckImplementation,
    UnstableCheckImplementation,
    InapplicableVerification,
    MissingVerificationChallenge,
    UnresolvedChallengePlan,
    MissingChallengeDecision,
    StaleChallengeDecision,
    RejectedChallengeDecision,
    InvalidChallengeDecision,
    InapplicableChallengeDecision,
    UnresolvedChallengeRelation,
    InvalidChallengeResolution,
    MissingRequiredChallenge,
    InsufficientChallengeScope,
}

impl FindingKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Unclassified => "unclassified",
            Self::Unrealized => "unrealized",
            Self::DanglingRealization => "dangling-realization",
            Self::DanglingDesignEntry => "dangling-design-entry",
            Self::DanglingVerificationScenario => "dangling-verification-scenario",
            Self::UndeclaredMechanism => "undeclared-mechanism",
            Self::UnresolvedDesignBinding => "unresolved-design-binding",
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
            Self::BindingMissingCheck => "binding-missing-check",
            Self::BindingMissingScenario => "binding-missing-scenario",
            Self::BindingMissingPolicy => "binding-missing-policy",
            Self::MissingMethodQualification => "missing-method-qualification",
            Self::DanglingMethodQualification => "dangling-method-qualification",
            Self::RejectedMethodQualification => "rejected-method-qualification",
            Self::StaleMethodQualification => "stale-method-qualification",
            Self::MissingApplicabilityDecision => "missing-applicability-decision",
            Self::DanglingApplicabilityDecision => "dangling-applicability-decision",
            Self::RejectedApplicabilityDecision => "rejected-applicability-decision",
            Self::StaleApplicabilityDecision => "stale-applicability-decision",
            Self::MissingRequirementJudgment => "missing-requirement-judgment",
            Self::PendingRequirementReview => "pending-requirement-review",
            Self::MissingRequirementAssurance => "missing-requirement-assurance",
            Self::InvalidRequirementReview => "invalid-requirement-review",
            Self::VerificationElementSupportUnresolved => "verification-element-support-unresolved",
            Self::ScenarioCheckContributionUnresolved => "scenario-check-contribution-unresolved",
            Self::CheckWithoutScenarioContribution => "check-without-scenario-contribution",
            Self::MechanismSupportUnresolved => "mechanism-support-unresolved",
            Self::RejectedRequirementJudgment => "rejected-requirement-judgment",
            Self::StaleRequirementJudgment => "stale-requirement-judgment",
            Self::InvalidRequirementJudgment => "invalid-requirement-judgment",
            Self::UnimplementedCheck => "unimplemented-check",
            Self::DanglingCheckImplementation => "dangling-check-implementation",
            Self::UnstableCheckImplementation => "unstable-check-implementation",
            Self::InapplicableVerification => "inapplicable-verification",
            Self::MissingVerificationChallenge => "missing-verification-challenge",
            Self::UnresolvedChallengePlan => "unresolved-challenge-plan",
            Self::MissingChallengeDecision => "missing-challenge-decision",
            Self::StaleChallengeDecision => "stale-challenge-decision",
            Self::RejectedChallengeDecision => "rejected-challenge-decision",
            Self::InvalidChallengeDecision => "invalid-challenge-decision",
            Self::InapplicableChallengeDecision => "inapplicable-challenge-decision",
            Self::UnresolvedChallengeRelation => "unresolved-challenge-relation",
            Self::InvalidChallengeResolution => "invalid-challenge-resolution",
            Self::MissingRequiredChallenge => "missing-required-challenge",
            Self::InsufficientChallengeScope => "insufficient-challenge-scope",
        }
    }

    pub fn category(self) -> FindingCategory {
        match self {
            Self::Unclassified => FindingCategory::Intent,
            Self::Unrealized
            | Self::DanglingRealization
            | Self::MissingRequiredRealization
            | Self::DanglingRealizationObligation => FindingCategory::Realization,
            Self::DanglingDesignEntry
            | Self::UndeclaredMechanism
            | Self::UnresolvedDesignBinding
            | Self::EnforcementMismatch
            | Self::DanglingMechanismImplementation => FindingCategory::Mechanism,
            Self::MissingSurface
            | Self::UnknownSurface
            | Self::EnumeratorUnsoundOrUnderived
            | Self::InvariantBreach => FindingCategory::Surface,
            Self::MissingMethodQualification
            | Self::DanglingMethodQualification
            | Self::RejectedMethodQualification
            | Self::StaleMethodQualification
            | Self::MissingApplicabilityDecision
            | Self::DanglingApplicabilityDecision
            | Self::RejectedApplicabilityDecision
            | Self::StaleApplicabilityDecision
            | Self::MissingRequirementJudgment
            | Self::PendingRequirementReview
            | Self::InvalidRequirementReview
            | Self::RejectedRequirementJudgment
            | Self::StaleRequirementJudgment
            | Self::InvalidRequirementJudgment => FindingCategory::Judgment,
            Self::MissingRequirementAssurance => FindingCategory::Assurance,
            Self::MissingVerificationChallenge
            | Self::UnresolvedChallengePlan
            | Self::MissingChallengeDecision
            | Self::StaleChallengeDecision
            | Self::RejectedChallengeDecision
            | Self::InvalidChallengeDecision
            | Self::InapplicableChallengeDecision
            | Self::UnresolvedChallengeRelation
            | Self::InvalidChallengeResolution
            | Self::MissingRequiredChallenge
            | Self::InsufficientChallengeScope => FindingCategory::Verification,
            _ => FindingCategory::Verification,
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Self::Unclassified => "Declare the Requirement's criticality explicitly.",
            Self::Unrealized => "Link production code that establishes the Requirement predicate.",
            Self::DanglingRealization => {
                "Retarget or remove the production link to the unknown Requirement."
            }
            Self::DanglingDesignEntry => "Retarget or remove the design entry.",
            Self::DanglingVerificationScenario => "Retarget or remove the Scenario verification declaration.",
            Self::UndeclaredMechanism => "Declare how the critical Requirement is enforced.",
            Self::UnresolvedDesignBinding => "Bind the mechanism to one extracted artifact.",
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
            Self::BindingMissingCheck => "Retarget the binding to a declared Check.",
            Self::BindingMissingScenario => "Retarget the binding to a current Scenario.",
            Self::BindingMissingPolicy => "Retarget the binding to a declared Decision Policy.",
            Self::MissingMethodQualification => {
                "Record the Check method's reviewed Method Qualification."
            }
            Self::DanglingMethodQualification => "Retarget or remove the Method Qualification.",
            Self::RejectedMethodQualification => {
                "Resolve the objection before qualifying the method."
            }
            Self::StaleMethodQualification => {
                "Re-qualify the method against its current fingerprint."
            }
            Self::MissingApplicabilityDecision => {
                "Record the binding's reviewed Applicability Decision."
            }
            Self::DanglingApplicabilityDecision => "Retarget or remove the Applicability Decision.",
            Self::RejectedApplicabilityDecision => {
                "Resolve the objection before accepting applicability."
            }
            Self::StaleApplicabilityDecision => {
                "Review applicability against its current fingerprint."
            }
            Self::MissingRequirementJudgment => "Record the Requirement's total-composition Judgment.",
            Self::PendingRequirementReview => "Complete the pending historical Requirement review against implementation and evidence before judgment.",
            Self::MissingRequirementAssurance => "Supply a matching native assurance snapshot with current accepted Requirement Review and supported exact-target Requirement Status.",
            Self::InvalidRequirementReview => "Repair the Requirement review references before judgment.",
            Self::VerificationElementSupportUnresolved => "Provide source/configuration support for the required verification element.",
            Self::ScenarioCheckContributionUnresolved => "Declare a source-authored Check contribution to this Scenario.",
            Self::CheckWithoutScenarioContribution => "Declare what Scenario this Check contributes to, or remove the unsupported Check.",
            Self::MechanismSupportUnresolved => "Provide implementation support for the selected mechanism.",
            Self::RejectedRequirementJudgment => "Resolve the objection before accepting the Requirement.",
            Self::StaleRequirementJudgment => "Rejudge the Requirement against its current composition.",
            Self::InvalidRequirementJudgment => {
                "Repair the Requirement composition before recording a current Judgment."
            }
            Self::UnimplementedCheck => {
                "Mark at least one stable source implementation of the Check."
            }
            Self::DanglingCheckImplementation => "Retarget the source marker to a declared Check.",
            Self::UnstableCheckImplementation => {
                "Resolve semantic source identity and fingerprint."
            }
            Self::InapplicableVerification => "Remove verification from the routine Requirement.",
            Self::MissingVerificationChallenge => "Retarget the Challenge Plan to a declared Verification Challenge.",
            Self::UnresolvedChallengePlan => "Select at least one current accepted decision.",
            Self::MissingChallengeDecision => "Author the decision reached by this selector.",
            Self::StaleChallengeDecision => "Refresh the decision reached by this selector.",
            Self::RejectedChallengeDecision => {
                "Resolve the rejection before selecting this decision."
            }
            Self::InvalidChallengeDecision => {
                "Repair the decision composition reached by this selector."
            }
            Self::InapplicableChallengeDecision => {
                "Retarget the selector to an applicable decision relation."
            }
            Self::UnresolvedChallengeRelation => {
                "Retarget the selector to a current semantic relation."
            }
            Self::InvalidChallengeResolution => {
                "Remove conflicting declarations before resolving the Challenge Plan."
            }
            Self::MissingRequiredChallenge => {
                "Declare a Verification Challenge and Plan for every policy-required form."
            }
            Self::InsufficientChallengeScope => {
                "Select the decision through scope covering every Verification Challenge requirement."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: Severity,
    pub requirement: Option<String>,
    pub entity_kind: Option<crate::diag::EntityKind>,
    pub criticality: Option<Criticality>,
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
            (
                "criticality",
                self.criticality
                    .map(|value| Json::str(value.name()))
                    .unwrap_or(Json::Null),
            ),
            ("file", Json::str(&self.path)),
            ("line", Json::Num(self.line as f64)),
            ("detail", Json::str(&self.detail)),
            ("help", Json::str(self.kind.help())),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecisionKind {
    ApplicabilityDecision,
    RequirementJudgment,
    MethodQualification,
}

impl DecisionKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::ApplicabilityDecision => "applicability-decision",
            Self::RequirementJudgment => "requirement-judgment",
            Self::MethodQualification => "method-qualification",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationKind {
    Binding,
    Scenario,
    Check,
    Requirement,
    MethodQualification,
    Mechanism,
    Realization,
}

impl RelationKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::Scenario => "scenario",
            Self::Check => "check",
            Self::Requirement => "requirement",
            Self::MethodQualification => "method-qualification",
            Self::Mechanism => "mechanism",
            Self::Realization => "realization",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectorIdentity {
    pub target: DecisionKind,
    pub from: RelationKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationIdentity {
    pub kind: RelationKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionCandidateTarget {
    pub kind: DecisionKind,
    pub id: String,
    pub expected_fingerprint: Option<String>,
    pub authored_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateDisposition {
    Selected,
    MissingDecision,
    StaleDecision,
    RejectedDecision,
    InvalidDecision,
    Inapplicable,
    UnresolvedRelation,
}

impl CandidateDisposition {
    pub fn name(self) -> &'static str {
        match self {
            Self::Selected => "selected",
            Self::MissingDecision => "missing-decision",
            Self::StaleDecision => "stale-decision",
            Self::RejectedDecision => "rejected-decision",
            Self::InvalidDecision => "invalid-decision",
            Self::Inapplicable => "inapplicable",
            Self::UnresolvedRelation => "unresolved-relation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeCandidate {
    pub selector: SelectorIdentity,
    pub relation: RelationIdentity,
    pub target: Option<DecisionCandidateTarget>,
    pub disposition: CandidateDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeResolution {
    pub plan: String,
    pub verification_challenge: String,
    pub candidates: Vec<ChallengeCandidate>,
    pub issues: Vec<String>,
}

pub fn resolve_challenge_plan(model: &Model, plan: &ChallengePlan) -> ChallengeResolution {
    let selectors = plan
        .selectors
        .iter()
        .map(selector_identity)
        .collect::<BTreeSet<_>>();
    let mut candidates = BTreeMap::new();
    let mut issues = Vec::new();
    for selector in selectors {
        for candidate in resolve_selector(model, &selector) {
            let key = candidate_key(&candidate);
            if candidates.contains_key(&key) {
                issues.push(format!(
                    "duplicate candidate `{} from {} {}` through {} `{}`",
                    candidate.selector.target.name(),
                    candidate.selector.from.name(),
                    candidate.selector.id,
                    candidate.relation.kind.name(),
                    candidate.relation.id
                ));
            } else {
                candidates.insert(key, candidate);
            }
        }
    }
    issues.sort();
    issues.dedup();
    ChallengeResolution {
        plan: plan.id.clone(),
        verification_challenge: plan.verification_challenge.clone(),
        candidates: candidates.into_values().collect(),
        issues,
    }
}

impl ChallengeResolution {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("format", Json::str("azimuth-challenge-resolution")),
            ("version", Json::Num(2.0)),
            ("plan", Json::str(&self.plan)),
            (
                "verification_challenge",
                Json::str(&self.verification_challenge),
            ),
            (
                "candidates",
                Json::Arr(
                    self.candidates
                        .iter()
                        .map(ChallengeCandidate::to_json)
                        .collect(),
                ),
            ),
            (
                "issues",
                Json::Arr(self.issues.iter().map(Json::str).collect()),
            ),
        ])
    }

    pub fn selected(&self) -> impl Iterator<Item = &ChallengeCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.disposition == CandidateDisposition::Selected)
    }

    pub fn is_runnable(&self) -> bool {
        self.issues.is_empty()
            && !self.candidates.is_empty()
            && self
                .candidates
                .iter()
                .all(|candidate| candidate.disposition == CandidateDisposition::Selected)
    }
}

impl ChallengeCandidate {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("selector", self.selector.to_json()),
            ("relation", self.relation.to_json()),
            (
                "target",
                self.target
                    .as_ref()
                    .map(DecisionCandidateTarget::to_json)
                    .unwrap_or(Json::Null),
            ),
            ("disposition", Json::str(self.disposition.name())),
        ])
    }
}

impl SelectorIdentity {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("target", Json::str(self.target.name())),
            ("from", Json::str(self.from.name())),
            ("id", Json::str(&self.id)),
        ])
    }
}

impl RelationIdentity {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("kind", Json::str(self.kind.name())),
            ("id", Json::str(&self.id)),
        ])
    }
}

impl DecisionCandidateTarget {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("kind", Json::str(self.kind.name())),
            ("id", Json::str(&self.id)),
            (
                "expected_fingerprint",
                self.expected_fingerprint
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "authored_fingerprint",
                self.authored_fingerprint
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
        ])
    }
}

fn selector_identity(selector: &Selector) -> SelectorIdentity {
    match selector {
        Selector::MethodQualificationFromMethodQualification(id) => selector_id(
            DecisionKind::MethodQualification,
            RelationKind::MethodQualification,
            id,
        ),
        Selector::MethodQualificationFromCheck(id) => {
            selector_id(DecisionKind::MethodQualification, RelationKind::Check, id)
        }
        Selector::MethodQualificationFromRealization(id) => selector_id(
            DecisionKind::MethodQualification,
            RelationKind::Realization,
            id,
        ),
        Selector::MethodQualificationFromMechanism(id) => selector_id(
            DecisionKind::MethodQualification,
            RelationKind::Mechanism,
            id,
        ),
        Selector::ApplicabilityDecisionFromBinding(id) => selector_id(
            DecisionKind::ApplicabilityDecision,
            RelationKind::Binding,
            id,
        ),
        Selector::ApplicabilityDecisionFromScenario(id) => selector_id(
            DecisionKind::ApplicabilityDecision,
            RelationKind::Scenario,
            id,
        ),
        Selector::ApplicabilityDecisionFromCheck(id) => {
            selector_id(DecisionKind::ApplicabilityDecision, RelationKind::Check, id)
        }
        Selector::ApplicabilityDecisionFromRealization(id) => selector_id(
            DecisionKind::ApplicabilityDecision,
            RelationKind::Realization,
            id,
        ),
        Selector::ApplicabilityDecisionFromMechanism(id) => selector_id(
            DecisionKind::ApplicabilityDecision,
            RelationKind::Mechanism,
            id,
        ),
        Selector::RequirementJudgmentFromRequirement(id) => selector_id(
            DecisionKind::RequirementJudgment,
            RelationKind::Requirement,
            id,
        ),
        Selector::RequirementJudgmentFromRealization(id) => selector_id(
            DecisionKind::RequirementJudgment,
            RelationKind::Realization,
            id,
        ),
        Selector::RequirementJudgmentFromMechanism(id) => selector_id(
            DecisionKind::RequirementJudgment,
            RelationKind::Mechanism,
            id,
        ),
    }
}

fn selector_id(target: DecisionKind, from: RelationKind, id: &str) -> SelectorIdentity {
    SelectorIdentity {
        target,
        from,
        id: id.to_string(),
    }
}

fn candidate_key(
    candidate: &ChallengeCandidate,
) -> (
    SelectorIdentity,
    RelationIdentity,
    Option<(DecisionKind, String)>,
) {
    (
        candidate.selector.clone(),
        candidate.relation.clone(),
        candidate
            .target
            .as_ref()
            .map(|target| (target.kind, target.id.clone())),
    )
}

fn resolve_selector(model: &Model, selector: &SelectorIdentity) -> Vec<ChallengeCandidate> {
    match (selector.target, selector.from) {
        (DecisionKind::MethodQualification, RelationKind::MethodQualification) => {
            let Some(qualification) = model
                .method_qualifications()
                .find(|qualification| qualification.id == selector.id)
            else {
                return vec![unresolved(
                    selector,
                    RelationKind::MethodQualification,
                    &selector.id,
                )];
            };
            vec![method_qualification_candidate(
                model,
                selector,
                qualification,
                true,
            )]
        }
        (DecisionKind::MethodQualification, RelationKind::Check) => {
            let qualifications = model
                .method_qualifications()
                .filter(|qualification| qualification.check == selector.id)
                .collect::<Vec<_>>();
            if qualifications.is_empty() {
                vec![unresolved(selector, RelationKind::Check, &selector.id)]
            } else {
                qualifications
                    .into_iter()
                    .map(|qualification| {
                        method_qualification_candidate(model, selector, qualification, true)
                    })
                    .collect()
            }
        }
        (DecisionKind::MethodQualification, RelationKind::Realization) => {
            method_qualification_relation_candidates(
                model,
                selector,
                scenarios_for_realization(model, &selector.id),
                ChallengeDomain::Realization,
            )
        }
        (DecisionKind::MethodQualification, RelationKind::Mechanism) => {
            method_qualification_relation_candidates(
                model,
                selector,
                scenarios_for_mechanism(model, &selector.id),
                ChallengeDomain::Mechanism,
            )
        }
        (DecisionKind::ApplicabilityDecision, RelationKind::Binding) => {
            let Some(binding) = model
                .evidence_bindings()
                .find(|binding| binding.id == selector.id)
            else {
                return vec![unresolved(selector, RelationKind::Binding, &selector.id)];
            };
            vec![applicability_candidate(model, selector, binding, true)]
        }
        (DecisionKind::ApplicabilityDecision, RelationKind::Scenario) => {
            let bindings = model
                .evidence_bindings()
                .filter(|binding| binding.scenario == selector.id)
                .collect::<Vec<_>>();
            if model.find_scenario(&selector.id).is_none() || bindings.is_empty() {
                vec![unresolved(selector, RelationKind::Scenario, &selector.id)]
            } else {
                bindings
                    .into_iter()
                    .map(|binding| applicability_candidate(model, selector, binding, true))
                    .collect()
            }
        }
        (DecisionKind::ApplicabilityDecision, RelationKind::Check) => {
            let check_exists = model.checks().any(|check| check.id == selector.id);
            let bindings = model
                .evidence_bindings()
                .filter(|binding| binding.check == selector.id)
                .collect::<Vec<_>>();
            if !check_exists || bindings.is_empty() {
                vec![unresolved(selector, RelationKind::Check, &selector.id)]
            } else {
                bindings
                    .into_iter()
                    .map(|binding| applicability_candidate(model, selector, binding, true))
                    .collect()
            }
        }
        (DecisionKind::ApplicabilityDecision, RelationKind::Realization) => {
            applicability_relation_candidates(
                model,
                selector,
                scenarios_for_realization(model, &selector.id),
                ChallengeDomain::Realization,
            )
        }
        (DecisionKind::ApplicabilityDecision, RelationKind::Mechanism) => {
            applicability_relation_candidates(
                model,
                selector,
                scenarios_for_mechanism(model, &selector.id),
                ChallengeDomain::Mechanism,
            )
        }
        (DecisionKind::RequirementJudgment, RelationKind::Requirement) => {
            if model
                .requirements()
                .any(|requirement| requirement.id() == selector.id)
            {
                vec![judgment_candidate(model, selector, &selector.id)]
            } else {
                vec![unresolved(
                    selector,
                    RelationKind::Requirement,
                    &selector.id,
                )]
            }
        }
        (DecisionKind::RequirementJudgment, RelationKind::Realization) => {
            judgment_relation_candidates(
                model,
                selector,
                requirements_for_realization(model, &selector.id),
            )
        }
        (DecisionKind::RequirementJudgment, RelationKind::Mechanism) => {
            judgment_relation_candidates(
                model,
                selector,
                requirements_for_mechanism(model, &selector.id),
            )
        }
        _ => unreachable!("the parser admits only supported selector forms"),
    }
}

fn requirements_for_realization(model: &Model, identity: &str) -> BTreeSet<String> {
    model
        .realizes
        .iter()
        .filter(|site| {
            site.source
                .as_ref()
                .is_some_and(|source| source.key() == identity)
                && model.has_requirement(&site.requirement)
        })
        .map(|site| site.requirement.clone())
        .collect()
}

fn requirements_for_mechanism(model: &Model, identity: &str) -> BTreeSet<String> {
    let mechanism_id = identity;
    let Some(design) = model.designs.iter().find(|design| {
        design.entries.iter().any(|entry| {
            entry
                .mechanisms
                .iter()
                .any(|mechanism| mechanism.id == identity)
        })
    }) else {
        return BTreeSet::new();
    };
    design
        .entries
        .iter()
        .filter(|entry| {
            entry
                .mechanisms
                .iter()
                .any(|mechanism| mechanism.id == mechanism_id)
        })
        .map(|entry| entry.target.id().to_string())
        .collect()
}

fn scenarios_for_realization(model: &Model, identity: &str) -> BTreeSet<String> {
    requirements_for_realization(model, identity)
        .into_iter()
        .flat_map(|requirement_id| {
            model
                .requirements()
                .find(|requirement| requirement.id() == requirement_id)
                .into_iter()
                .flat_map(|requirement| {
                    requirement
                        .requirement
                        .scenarios
                        .iter()
                        .map(move |scenario| scenario.id.clone())
                })
        })
        .collect()
}

fn scenarios_for_mechanism(model: &Model, identity: &str) -> BTreeSet<String> {
    let mechanism_id = identity;
    let Some(design) = model.designs.iter().find(|design| {
        design.entries.iter().any(|entry| {
            entry
                .mechanisms
                .iter()
                .any(|mechanism| mechanism.id == identity)
        })
    }) else {
        return BTreeSet::new();
    };
    let mut scenarios = BTreeSet::new();
    for entry in &design.entries {
        let Some(requirement) = model.find_requirement(entry.target.id()) else {
            continue;
        };
        for mechanism in entry
            .mechanisms
            .iter()
            .filter(|mechanism| mechanism.id == mechanism_id)
        {
            for scenario in &requirement.requirement.scenarios {
                if mechanism.scenarios.is_empty() || mechanism.scenarios.contains(&scenario.id) {
                    scenarios.insert(scenario.id.clone());
                }
            }
        }
    }
    scenarios
}

fn applicability_relation_candidates(
    model: &Model,
    selector: &SelectorIdentity,
    scenarios: BTreeSet<String>,
    domain: ChallengeDomain,
) -> Vec<ChallengeCandidate> {
    if scenarios.is_empty() {
        return vec![unresolved(selector, selector.from, &selector.id)];
    }
    let mut candidates = Vec::new();
    for scenario in scenarios {
        let bindings = model
            .evidence_bindings()
            .filter(|binding| binding.scenario == scenario)
            .collect::<Vec<_>>();
        if bindings.is_empty() {
            candidates.push(unresolved(selector, RelationKind::Scenario, &scenario));
        } else {
            candidates.extend(bindings.into_iter().map(|binding| {
                applicability_candidate(
                    model,
                    selector,
                    binding,
                    binding.challenge_domain.contains(&domain),
                )
            }));
        }
    }
    candidates
}

fn method_qualification_relation_candidates(
    model: &Model,
    selector: &SelectorIdentity,
    scenarios: BTreeSet<String>,
    domain: ChallengeDomain,
) -> Vec<ChallengeCandidate> {
    if scenarios.is_empty() {
        return vec![unresolved(selector, selector.from, &selector.id)];
    }
    let qualification_ids = model
        .evidence_bindings()
        .filter(|binding| scenarios.contains(&binding.scenario))
        .map(|binding| binding.method_qualification.clone())
        .collect::<BTreeSet<_>>();
    if qualification_ids.is_empty() {
        return vec![unresolved(selector, selector.from, &selector.id)];
    }
    qualification_ids
        .into_iter()
        .map(|id| {
            let Some(qualification) = model
                .method_qualifications()
                .find(|qualification| qualification.id == id)
            else {
                return unresolved(selector, RelationKind::MethodQualification, &id);
            };
            method_qualification_candidate(
                model,
                selector,
                qualification,
                qualification.challenge_domain.contains(&domain),
            )
        })
        .collect()
}

fn judgment_relation_candidates(
    model: &Model,
    selector: &SelectorIdentity,
    requirements: BTreeSet<String>,
) -> Vec<ChallengeCandidate> {
    if requirements.is_empty() {
        return vec![unresolved(selector, selector.from, &selector.id)];
    }
    requirements
        .iter()
        .map(|requirement| judgment_candidate(model, selector, requirement))
        .collect()
}

fn applicability_candidate(
    model: &Model,
    selector: &SelectorIdentity,
    binding: &crate::verification::EvidenceBinding,
    relation_applicable: bool,
) -> ChallengeCandidate {
    let requirement = model
        .scenarios()
        .find(|requirement| requirement.id() == binding.scenario);
    let decision = model
        .applicability_decisions()
        .find(|decision| decision.id == binding.id);
    let expected = model.expected_applicability_fingerprint(binding);
    let authored = decision.map(|decision| decision.fingerprint.clone());
    let inapplicable = !relation_applicable
        || requirement.as_ref().is_some_and(|requirement| {
            requirement.requirement.criticality == Some(Criticality::Routine)
        });
    let disposition = if inapplicable {
        CandidateDisposition::Inapplicable
    } else if decision.is_none() {
        CandidateDisposition::MissingDecision
    } else if expected.is_none() {
        CandidateDisposition::InvalidDecision
    } else if authored.as_ref() != expected.as_ref() {
        CandidateDisposition::StaleDecision
    } else if decision.is_some_and(|decision| {
        decision.verdict == crate::verification::ApplicabilityVerdict::Rejected
    }) {
        CandidateDisposition::RejectedDecision
    } else {
        CandidateDisposition::Selected
    };
    ChallengeCandidate {
        selector: selector.clone(),
        relation: RelationIdentity {
            kind: RelationKind::Binding,
            id: binding.id.clone(),
        },
        target: Some(DecisionCandidateTarget {
            kind: DecisionKind::ApplicabilityDecision,
            id: binding.id.clone(),
            expected_fingerprint: expected,
            authored_fingerprint: authored,
        }),
        disposition,
    }
}

fn method_qualification_candidate(
    model: &Model,
    selector: &SelectorIdentity,
    qualification: &crate::verification::MethodQualification,
    relation_applicable: bool,
) -> ChallengeCandidate {
    let expected = model.expected_method_qualification_fingerprint(qualification);
    let authored = Some(qualification.fingerprint.clone());
    let has_non_routine_edge = model.evidence_bindings().any(|binding| {
        binding.method_qualification == qualification.id
            && model
                .find_scenario(&binding.scenario)
                .is_some_and(|scenario| {
                    matches!(
                        scenario.requirement.criticality,
                        Some(Criticality::Standard | Criticality::Critical)
                    )
                })
    });
    let disposition = if !relation_applicable || !has_non_routine_edge {
        CandidateDisposition::Inapplicable
    } else if expected.is_none() {
        CandidateDisposition::InvalidDecision
    } else if authored.as_ref() != expected.as_ref() {
        CandidateDisposition::StaleDecision
    } else if qualification.verdict == MethodQualificationVerdict::Rejected {
        CandidateDisposition::RejectedDecision
    } else {
        CandidateDisposition::Selected
    };
    ChallengeCandidate {
        selector: selector.clone(),
        relation: RelationIdentity {
            kind: RelationKind::MethodQualification,
            id: qualification.id.clone(),
        },
        target: Some(DecisionCandidateTarget {
            kind: DecisionKind::MethodQualification,
            id: qualification.id.clone(),
            expected_fingerprint: expected,
            authored_fingerprint: authored,
        }),
        disposition,
    }
}

fn judgment_candidate(
    model: &Model,
    selector: &SelectorIdentity,
    requirement_id: &str,
) -> ChallengeCandidate {
    let requirement = model
        .requirements()
        .find(|requirement| requirement.id() == requirement_id);
    let judgment = model
        .requirement_judgments()
        .find(|judgment| judgment.id == requirement_id);
    let expected =
        judgment.and_then(|judgment| model.expected_requirement_judgment_fingerprint(judgment));
    let authored = judgment.map(|judgment| judgment.fingerprint.clone());
    let disposition = if requirement.as_ref().is_some_and(|requirement| {
        requirement.requirement.criticality == Some(Criticality::Routine)
    }) {
        CandidateDisposition::Inapplicable
    } else if judgment.is_none() {
        CandidateDisposition::MissingDecision
    } else if expected.is_none() {
        CandidateDisposition::InvalidDecision
    } else if authored.as_ref() != expected.as_ref() {
        CandidateDisposition::StaleDecision
    } else if judgment
        .is_some_and(|judgment| judgment.verdict == RequirementJudgmentVerdict::Rejected)
    {
        CandidateDisposition::RejectedDecision
    } else {
        CandidateDisposition::Selected
    };
    ChallengeCandidate {
        selector: selector.clone(),
        relation: RelationIdentity {
            kind: RelationKind::Requirement,
            id: requirement_id.to_string(),
        },
        target: Some(DecisionCandidateTarget {
            kind: DecisionKind::RequirementJudgment,
            id: requirement_id.to_string(),
            expected_fingerprint: expected,
            authored_fingerprint: authored,
        }),
        disposition,
    }
}

fn unresolved(
    selector: &SelectorIdentity,
    relation_kind: RelationKind,
    relation_id: &str,
) -> ChallengeCandidate {
    ChallengeCandidate {
        selector: selector.clone(),
        relation: RelationIdentity {
            kind: relation_kind,
            id: relation_id.to_string(),
        },
        target: None,
        disposition: CandidateDisposition::UnresolvedRelation,
    }
}

fn severity_for(criticality: Option<Criticality>) -> Severity {
    match criticality {
        Some(Criticality::Routine) => Severity::Warning,
        _ => Severity::Error,
    }
}

pub fn validate(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    for spec in &model.specs {
        for requirement in &spec.requirements {
            if requirement.criticality.is_none() {
                findings.push(Finding {
                    kind: FindingKind::Unclassified,
                    severity: Severity::Error,
                    requirement: Some(requirement.id.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: None,
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: format!("Requirement `{}` declares no criticality", requirement.id),
                });
            }
        }
    }
    for requirement in model.requirements() {
        if requirement.requirement.criticality == Some(Criticality::Routine) {
            continue;
        }
        if !model
            .realizes
            .iter()
            .any(|site| site.requirement == requirement.requirement.id)
        {
            findings.push(Finding {
                kind: FindingKind::Unrealized,
                severity: severity_for(requirement.requirement.criticality),
                requirement: Some(requirement.id()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                criticality: requirement.requirement.criticality,
                path: requirement.spec.path.clone(),
                line: requirement.requirement.line,
                detail: "no production code realizes this Requirement".into(),
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
                criticality: None,
                path: site.file.clone(),
                line: 0,
                detail: format!("`{}` realizes a Requirement that does not exist", site.site),
            });
        }
    }
    for implementation in &model.mechanism_implementations {
        let declared = model.designs.iter().any(|design| {
            design
                .entries
                .iter()
                .flat_map(|entry| &entry.mechanisms)
                .any(|mechanism| mechanism.id == implementation.mechanism)
        }) || model.account_designs.iter().any(|design| {
            design
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
                criticality: None,
                path: implementation.file.clone(),
                line: 0,
                detail: "implementation names no design-owned mechanism".into(),
            });
        }
    }
    findings.extend(verification_findings(model));
    findings.extend(design_findings(model));
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
    let policies = model
        .decision_standards
        .as_ref()
        .map(|standards| &standards.policies[..])
        .unwrap_or(&[]);
    for scenario in model.scenarios() {
        let bindings = model
            .evidence_bindings()
            .filter(|binding| binding.scenario == scenario.id())
            .collect::<Vec<_>>();
        if scenario.requirement.criticality == Some(Criticality::Routine) {
            for binding in bindings {
                findings.push(Finding {
                    kind: FindingKind::InapplicableVerification,
                    severity: Severity::Warning,
                    requirement: Some(scenario.id()),
                    entity_kind: Some(crate::diag::EntityKind::Scenario),
                    criticality: scenario.requirement.criticality,
                    path: binding.path.clone(),
                    line: binding.line,
                    detail: format!(
                        "Evidence Binding `{}` targets a routine Scenario",
                        binding.id
                    ),
                });
            }
        } else if scenario.requirement.criticality.is_some()
            && bindings.is_empty()
            && !model
                .account_verifications
                .iter()
                .flat_map(|file| &file.checks)
                .any(|check| {
                    check
                        .bindings
                        .iter()
                        .any(|binding| binding.scenario == scenario.id())
                })
        {
            findings.push(Finding {
                kind: FindingKind::UnboundScenario,
                severity: severity_for(scenario.requirement.criticality),
                requirement: Some(scenario.id()),
                entity_kind: Some(crate::diag::EntityKind::Scenario),
                criticality: scenario.requirement.criticality,
                path: scenario.spec.path.clone(),
                line: scenario.scenario.line,
                detail: "non-routine Scenario has no Evidence Binding".into(),
            });
        }
    }
    for requirement in model.requirements() {
        if requirement.requirement.criticality == Some(Criticality::Routine) {
            for judgment in model
                .requirement_judgments()
                .filter(|judgment| judgment.id == requirement.id())
            {
                findings.push(Finding {
                    kind: FindingKind::InapplicableVerification,
                    severity: Severity::Warning,
                    requirement: Some(requirement.id()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: requirement.requirement.criticality,
                    path: judgment.path.clone(),
                    line: judgment.line,
                    detail: format!(
                        "Requirement Judgment `{}` targets a routine Requirement",
                        judgment.id
                    ),
                });
            }
            continue;
        }
        if !matches!(
            requirement.requirement.criticality,
            Some(Criticality::Standard | Criticality::Critical)
        ) {
            continue;
        }
        if model.account_verifications.iter().any(|facet| {
            facet
                .requirements
                .iter()
                .any(|entry| entry.requirement == requirement.id())
        }) {
            findings.push(simple(
                FindingKind::MissingRequirementAssurance,
                &requirement.spec.path,
                requirement.requirement.line,
                Some(requirement.id()),
                Some(crate::diag::EntityKind::Requirement),
                "compact Requirement requires current accepted native Requirement Review and supported exact-target Requirement Status; definitions alone establish neither".into(),
            ));
            continue;
        }
        let judgment = model
            .requirement_judgments()
            .find(|judgment| judgment.id == requirement.id());
        match judgment {
            None => {
                let review = model.account_reviews.iter().find_map(|facet| {
                    facet
                        .requirements
                        .iter()
                        .find(|review| review.requirement == requirement.id())
                        .map(|review| (facet, review))
                });
                let (kind, path, line, detail) = match review {
                    Some((facet, review)) => (
                        FindingKind::PendingRequirementReview,
                        facet.path.clone(),
                        review.line,
                        "Requirement review is pending; no accepted total-composition Judgment exists",
                    ),
                    None => (
                        FindingKind::MissingRequirementJudgment,
                        requirement.spec.path.clone(),
                        requirement.requirement.line,
                        "non-routine Requirement has no total-composition Judgment",
                    ),
                };
                findings.push(Finding {
                    kind,
                    severity: Severity::Error,
                    requirement: Some(requirement.id()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: requirement.requirement.criticality,
                    path,
                    line,
                    detail: detail.into(),
                });
            }
            Some(judgment) => {
                let expected = model.expected_requirement_judgment_fingerprint(judgment);
                if expected.is_none() {
                    findings.push(simple(
                        FindingKind::InvalidRequirementJudgment,
                        &judgment.path,
                        judgment.line,
                        Some(requirement.id()),
                        Some(crate::diag::EntityKind::Requirement),
                        format!(
                            "Requirement Judgment `{}` has unavailable expected composition",
                            judgment.id
                        ),
                    ));
                } else if expected.as_ref() != Some(&judgment.fingerprint) {
                    findings.push(simple(
                        FindingKind::StaleRequirementJudgment,
                        &judgment.path,
                        judgment.line,
                        Some(requirement.id()),
                        Some(crate::diag::EntityKind::Requirement),
                        format!(
                            "Requirement Judgment `{}` expected {}, found {}",
                            judgment.id,
                            expected.as_deref().unwrap_or_default(),
                            judgment.fingerprint
                        ),
                    ));
                } else if judgment.verdict == RequirementJudgmentVerdict::Rejected {
                    findings.push(simple(
                        FindingKind::RejectedRequirementJudgment,
                        &judgment.path,
                        judgment.line,
                        Some(requirement.id()),
                        Some(crate::diag::EntityKind::Requirement),
                        format!("Requirement Judgment `{}` is rejected", judgment.id),
                    ));
                }
            }
        }
    }
    for check in model.checks() {
        if !model
            .evidence_bindings()
            .any(|binding| binding.check == check.id)
            && !model
                .account_verifications
                .iter()
                .flat_map(|file| &file.checks)
                .any(|item| item.definition.id == check.id && !item.bindings.is_empty())
        {
            findings.push(simple(
                FindingKind::CheckWithoutBinding,
                &check.path,
                check.line,
                None,
                None,
                format!("Check `{}` has no Evidence Binding", check.id),
            ));
        }
        let has_applicable_binding = model
            .account_verifications
            .iter()
            .flat_map(|file| &file.checks)
            .any(|item| item.definition.id == check.id && !item.bindings.is_empty())
            || model.evidence_bindings().any(|binding| {
                binding.check == check.id
                    && model
                        .scenarios()
                        .find(|requirement| requirement.id() == binding.scenario)
                        .is_some_and(|requirement| {
                            matches!(
                                requirement.requirement.criticality,
                                Some(Criticality::Standard | Criticality::Critical)
                            )
                        })
            });
        if has_applicable_binding
            && !model
                .check_implementations
                .iter()
                .any(|implementation| implementation.check == check.id)
        {
            findings.push(simple(
                FindingKind::UnimplementedCheck,
                &check.path,
                check.line,
                None,
                None,
                format!("Check `{}` has no source implementation", check.id),
            ));
        }
    }
    for implementation in &model.check_implementations {
        let check_exists = model.checks().any(|check| check.id == implementation.check)
            || model
                .account_verifications
                .iter()
                .flat_map(|file| &file.checks)
                .any(|check| check.definition.id == implementation.check);
        if !check_exists {
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
        let has_applicable_binding = model.evidence_bindings().any(|binding| {
            binding.check == implementation.check
                && model
                    .scenarios()
                    .find(|requirement| requirement.id() == binding.scenario)
                    .is_some_and(|requirement| {
                        matches!(
                            requirement.requirement.criticality,
                            Some(Criticality::Standard | Criticality::Critical)
                        )
                    })
        });
        if check_exists
            && has_applicable_binding
            && (implementation.source.is_none()
                || !valid_fingerprint(&implementation.source_fingerprint))
        {
            findings.push(simple(
                FindingKind::UnstableCheckImplementation,
                &implementation.file,
                0,
                None,
                None,
                format!(
                    "Check `{}` lacks stable semantic identity or source fingerprint",
                    implementation.check
                ),
            ));
        }
    }
    for binding in model.evidence_bindings() {
        let requirement = model
            .scenarios()
            .find(|requirement| requirement.id() == binding.scenario);
        if requirement.as_ref().is_some_and(|requirement| {
            requirement.requirement.criticality == Some(Criticality::Routine)
        }) {
            continue;
        }
        if !model.checks().any(|check| check.id == binding.check) {
            findings.push(simple(
                FindingKind::BindingMissingCheck,
                &binding.path,
                binding.line,
                Some(binding.scenario.clone()),
                Some(crate::diag::EntityKind::Scenario),
                format!(
                    "binding `{}` names unknown Check `{}`",
                    binding.id, binding.check
                ),
            ));
        }
        if requirement.is_none() {
            findings.push(simple(
                FindingKind::BindingMissingScenario,
                &binding.path,
                binding.line,
                Some(binding.scenario.clone()),
                Some(crate::diag::EntityKind::Scenario),
                format!("binding `{}` names no current Scenario", binding.id),
            ));
        }
        if !policies.iter().any(|policy| policy.id == binding.policy) {
            findings.push(simple(
                FindingKind::BindingMissingPolicy,
                &binding.path,
                binding.line,
                Some(binding.scenario.clone()),
                Some(crate::diag::EntityKind::Scenario),
                format!(
                    "binding `{}` names unknown policy `{}`",
                    binding.id, binding.policy
                ),
            ));
        }
        let method_qualification = model
            .method_qualifications()
            .find(|qualification| qualification.id == binding.method_qualification);
        let Some(method_qualification) = method_qualification else {
            findings.push(simple(
                FindingKind::MissingMethodQualification,
                &binding.path,
                binding.line,
                Some(binding.scenario.clone()),
                Some(crate::diag::EntityKind::Scenario),
                format!(
                    "binding `{}` names missing Method Qualification `{}`",
                    binding.id, binding.method_qualification
                ),
            ));
            continue;
        };
        if method_qualification.check != binding.check {
            findings.push(simple(
                FindingKind::MissingMethodQualification,
                &binding.path,
                binding.line,
                Some(binding.scenario.clone()),
                Some(crate::diag::EntityKind::Scenario),
                format!(
                    "binding `{}` and Method Qualification `{}` name different Checks",
                    binding.id, method_qualification.id
                ),
            ));
            continue;
        }
        let decision = model
            .applicability_decisions()
            .find(|decision| decision.id == binding.id);
        let Some(decision) = decision else {
            findings.push(simple(
                FindingKind::MissingApplicabilityDecision,
                &binding.path,
                binding.line,
                Some(binding.scenario.clone()),
                Some(crate::diag::EntityKind::Scenario),
                format!("binding `{}` has no Applicability Decision", binding.id),
            ));
            continue;
        };
        if let Some(expected) = model.expected_applicability_fingerprint(binding) {
            if decision.fingerprint != expected {
                findings.push(simple(
                    FindingKind::StaleApplicabilityDecision,
                    &decision.path,
                    decision.line,
                    Some(binding.scenario.clone()),
                    Some(crate::diag::EntityKind::Scenario),
                    format!(
                        "Applicability Decision `{}` expected {}, found {}",
                        decision.id, expected, decision.fingerprint
                    ),
                ));
            } else if decision.verdict == crate::verification::ApplicabilityVerdict::Rejected {
                findings.push(simple(
                    FindingKind::RejectedApplicabilityDecision,
                    &decision.path,
                    decision.line,
                    Some(binding.scenario.clone()),
                    Some(crate::diag::EntityKind::Scenario),
                    format!("Applicability Decision `{}` is rejected", decision.id),
                ));
            }
        }
    }
    for qualification in model.method_qualifications() {
        if !model.checks().any(|check| check.id == qualification.check)
            || !model
                .evidence_bindings()
                .any(|binding| binding.method_qualification == qualification.id)
        {
            findings.push(simple(
                FindingKind::DanglingMethodQualification,
                &qualification.path,
                qualification.line,
                None,
                None,
                format!(
                    "Method Qualification `{}` has no current Check-to-Scenario edge",
                    qualification.id
                ),
            ));
            continue;
        }
        if !policies
            .iter()
            .any(|policy| policy.id == qualification.policy)
        {
            findings.push(simple(
                FindingKind::DanglingMethodQualification,
                &qualification.path,
                qualification.line,
                None,
                None,
                format!(
                    "Method Qualification `{}` names unknown policy `{}`",
                    qualification.id, qualification.policy
                ),
            ));
            continue;
        }
        if let Some(expected) = model.expected_method_qualification_fingerprint(qualification) {
            if qualification.fingerprint != expected {
                findings.push(simple(
                    FindingKind::StaleMethodQualification,
                    &qualification.path,
                    qualification.line,
                    None,
                    None,
                    format!(
                        "Method Qualification `{}` expected {}, found {}",
                        qualification.id, expected, qualification.fingerprint
                    ),
                ));
            } else if qualification.verdict == MethodQualificationVerdict::Rejected {
                findings.push(simple(
                    FindingKind::RejectedMethodQualification,
                    &qualification.path,
                    qualification.line,
                    None,
                    None,
                    format!("Method Qualification `{}` is rejected", qualification.id),
                ));
            }
        }
    }
    for decision in model.applicability_decisions() {
        if !model
            .evidence_bindings()
            .any(|binding| binding.id == decision.id)
        {
            findings.push(simple(
                FindingKind::DanglingApplicabilityDecision,
                &decision.path,
                decision.line,
                None,
                None,
                format!(
                    "Applicability Decision `{}` names no Evidence Binding",
                    decision.id
                ),
            ));
        }
    }
    for plan in model.challenge_plans() {
        if !model
            .verification_challenges()
            .any(|verification_challenge| verification_challenge.id == plan.verification_challenge)
        {
            findings.push(simple(
                FindingKind::MissingVerificationChallenge,
                &plan.path,
                plan.line,
                None,
                None,
                format!(
                    "Challenge Plan `{}` names unknown Verification Challenge `{}`",
                    plan.id, plan.verification_challenge
                ),
            ));
        }
        let resolution = resolve_challenge_plan(model, plan);
        for issue in &resolution.issues {
            findings.push(simple(
                FindingKind::InvalidChallengeResolution,
                &plan.path,
                plan.line,
                None,
                None,
                format!("Challenge Plan `{}`: {issue}", plan.id),
            ));
        }
        for candidate in &resolution.candidates {
            let Some(kind) = adverse_candidate_finding(candidate.disposition) else {
                continue;
            };
            findings.push(simple(
                kind,
                &plan.path,
                plan.line,
                candidate_target_requirement(model, candidate),
                candidate
                    .target
                    .as_ref()
                    .map(|target| decision_entity_kind(target.kind)),
                format!(
                    "Challenge Plan `{}` selector `{} from {} {}` reaches {} `{}` as `{}`",
                    plan.id,
                    candidate.selector.target.name(),
                    candidate.selector.from.name(),
                    candidate.selector.id,
                    candidate.relation.kind.name(),
                    candidate.relation.id,
                    candidate.disposition.name()
                ),
            ));
        }
        if resolution.selected().next().is_none() {
            findings.push(simple(
                FindingKind::UnresolvedChallengePlan,
                &plan.path,
                plan.line,
                None,
                None,
                format!(
                    "Challenge Plan `{}` resolves no current accepted decision",
                    plan.id
                ),
            ));
        }
    }
    findings.extend(required_challenge_coverage_findings(model));
    findings
}

fn adverse_candidate_finding(disposition: CandidateDisposition) -> Option<FindingKind> {
    match disposition {
        CandidateDisposition::Selected => None,
        CandidateDisposition::MissingDecision => Some(FindingKind::MissingChallengeDecision),
        CandidateDisposition::StaleDecision => Some(FindingKind::StaleChallengeDecision),
        CandidateDisposition::RejectedDecision => Some(FindingKind::RejectedChallengeDecision),
        CandidateDisposition::InvalidDecision => Some(FindingKind::InvalidChallengeDecision),
        CandidateDisposition::Inapplicable => Some(FindingKind::InapplicableChallengeDecision),
        CandidateDisposition::UnresolvedRelation => Some(FindingKind::UnresolvedChallengeRelation),
    }
}

fn candidate_target_requirement(model: &Model, candidate: &ChallengeCandidate) -> Option<String> {
    let target = candidate.target.as_ref()?;
    match target.kind {
        DecisionKind::RequirementJudgment => Some(target.id.clone()),
        DecisionKind::ApplicabilityDecision => model
            .evidence_bindings()
            .find(|binding| binding.id == target.id)
            .map(|binding| binding.scenario.clone()),
        DecisionKind::MethodQualification => model
            .evidence_bindings()
            .filter(|binding| binding.method_qualification == target.id)
            .map(|binding| binding.scenario.clone())
            .next(),
    }
}

/// Selection keeps an authored Plan atomically when any one of its selectors touches the selected
/// semantic graph. Applicability and decision freshness deliberately do not participate: pruning
/// only successful selectors would hide adverse siblings from the derived account.
pub fn challenge_plan_relevant_to_selection(
    model: &Model,
    plan: &ChallengePlan,
    requirements: &BTreeSet<String>,
    bindings: &BTreeSet<String>,
    checks: &BTreeSet<String>,
) -> bool {
    plan.selectors.iter().any(|selector| match selector {
        Selector::MethodQualificationFromMethodQualification(id) => {
            model.evidence_bindings().any(|binding| {
                binding.method_qualification == *id && requirements.contains(&binding.scenario)
            })
        }
        Selector::MethodQualificationFromCheck(id)
        | Selector::ApplicabilityDecisionFromCheck(id) => checks.contains(id),
        Selector::ApplicabilityDecisionFromBinding(id) => bindings.contains(id),
        Selector::ApplicabilityDecisionFromScenario(id) => requirements.contains(id),
        Selector::MethodQualificationFromRealization(identity)
        | Selector::ApplicabilityDecisionFromRealization(identity) => {
            scenarios_for_realization(model, identity)
                .iter()
                .any(|requirement| requirements.contains(requirement))
        }
        Selector::MethodQualificationFromMechanism(identity)
        | Selector::ApplicabilityDecisionFromMechanism(identity) => {
            scenarios_for_mechanism(model, identity)
                .iter()
                .any(|requirement| requirements.contains(requirement))
        }
        Selector::RequirementJudgmentFromRequirement(id) => requirements.iter().any(|scenario| {
            model
                .find_scenario(scenario)
                .is_some_and(|view| view.requirement.id == *id)
        }),
        Selector::RequirementJudgmentFromRealization(identity) => {
            requirements_for_realization(model, identity)
                .iter()
                .any(|parent| {
                    requirements.iter().any(|scenario| {
                        model
                            .find_scenario(scenario)
                            .is_some_and(|view| view.requirement.id == *parent)
                    })
                })
        }
        Selector::RequirementJudgmentFromMechanism(identity) => {
            requirements_for_mechanism(model, identity)
                .iter()
                .any(|parent| {
                    requirements.iter().any(|scenario| {
                        model
                            .find_scenario(scenario)
                            .is_some_and(|view| view.requirement.id == *parent)
                    })
                })
        }
    })
}

fn required_challenge_coverage_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(standards) = &model.decision_standards else {
        return findings;
    };
    let plans = model
        .challenge_plans()
        .map(|plan| (plan, resolve_challenge_plan(model, plan)))
        .collect::<Vec<_>>();

    let mut decisions = Vec::<(DecisionCandidateTarget, String, String, usize)>::new();
    for qualification in model.method_qualifications() {
        let selector = selector_id(
            DecisionKind::MethodQualification,
            RelationKind::MethodQualification,
            &qualification.id,
        );
        let candidate = method_qualification_candidate(model, &selector, qualification, true);
        if candidate.disposition == CandidateDisposition::Selected {
            decisions.push((
                candidate.target.expect("a reached method has a target"),
                qualification.policy.clone(),
                qualification.path.clone(),
                qualification.line,
            ));
        }
    }
    for binding in model.evidence_bindings() {
        let selector = selector_id(
            DecisionKind::ApplicabilityDecision,
            RelationKind::Binding,
            &binding.id,
        );
        let candidate = applicability_candidate(model, &selector, binding, true);
        if candidate.disposition == CandidateDisposition::Selected {
            decisions.push((
                candidate.target.expect("a reached binding has a target"),
                binding.policy.clone(),
                binding.path.clone(),
                binding.line,
            ));
        }
    }
    for judgment in model.requirement_judgments() {
        let selector = selector_id(
            DecisionKind::RequirementJudgment,
            RelationKind::Requirement,
            &judgment.id,
        );
        let candidate = judgment_candidate(model, &selector, &judgment.id);
        if candidate.disposition == CandidateDisposition::Selected {
            decisions.push((
                candidate
                    .target
                    .expect("a reached Requirement has a target"),
                judgment.policy.clone(),
                judgment.path.clone(),
                judgment.line,
            ));
        }
    }

    for (target, policy_id, path, line) in decisions {
        let Some(policy) = standards
            .policies
            .iter()
            .find(|policy| policy.id == policy_id)
        else {
            continue;
        };
        for form in &policy.required_challenges {
            let verification_challenges = model
                .verification_challenges()
                .filter(|verification_challenge| verification_challenge.form == *form)
                .collect::<Vec<_>>();
            if verification_challenges.is_empty() {
                findings.push(simple(
                    FindingKind::MissingRequiredChallenge,
                    &path,
                    line,
                    target_requirement(model, &target),
                    Some(decision_entity_kind(target.kind)),
                    format!(
                        "{} `{}` requires Challenge form `{form}`, which has no Verification Challenge",
                        target.kind.name(),
                        target.id
                    ),
                ));
                continue;
            }

            let mut reaches_target = false;
            let covered = verification_challenges
                .iter()
                .any(|verification_challenge| {
                    plans.iter().any(|(plan, resolution)| {
                        if plan.verification_challenge != verification_challenge.id
                            || !resolution.is_runnable()
                        {
                            return false;
                        }
                        let matching = resolution
                            .candidates
                            .iter()
                            .filter(|candidate| {
                                candidate.disposition == CandidateDisposition::Selected
                                    && same_target(candidate.target.as_ref(), &target)
                            })
                            .collect::<Vec<_>>();
                        if matching.is_empty() {
                            return false;
                        }
                        reaches_target = true;
                        let kinds = matching.into_iter().try_fold(
                            BTreeSet::new(),
                            |mut union, candidate| {
                                union.extend(challenge_candidate_scope_kinds(model, candidate)?);
                                Some(union)
                            },
                        );
                        let Some(kinds) = kinds else {
                            return false;
                        };
                        verification_challenge
                            .required_scope
                            .iter()
                            .all(|required| kinds.contains(required))
                    })
                });
            if !covered {
                let kind = if reaches_target {
                    FindingKind::InsufficientChallengeScope
                } else {
                    FindingKind::MissingRequiredChallenge
                };
                findings.push(simple(
                    kind,
                    &path,
                    line,
                    target_requirement(model, &target),
                    Some(decision_entity_kind(target.kind)),
                    format!(
                        "{} `{}` has no Plan covering required form `{form}` with its declared \
                         scope",
                        target.kind.name(),
                        target.id
                    ),
                ));
            }
        }
    }
    findings
}

fn same_target(
    candidate: Option<&DecisionCandidateTarget>,
    target: &DecisionCandidateTarget,
) -> bool {
    candidate.is_some_and(|candidate| {
        candidate.kind == target.kind
            && candidate.id == target.id
            && candidate.expected_fingerprint == target.expected_fingerprint
    })
}

fn target_requirement(model: &Model, target: &DecisionCandidateTarget) -> Option<String> {
    match target.kind {
        DecisionKind::RequirementJudgment => Some(target.id.clone()),
        DecisionKind::ApplicabilityDecision => model
            .evidence_bindings()
            .find(|binding| binding.id == target.id)
            .map(|binding| binding.scenario.clone()),
        DecisionKind::MethodQualification => model
            .evidence_bindings()
            .find(|binding| binding.method_qualification == target.id)
            .map(|binding| binding.scenario.clone()),
    }
}

/// Returns the exact closed kinds present in one selected candidate's selector anchor and
/// decision inputs. Full typed item construction belongs to semantic planning; this projection is
/// intentionally sufficient only for repository declaration-coverage validation.
pub fn challenge_candidate_scope_kinds(
    model: &Model,
    candidate: &ChallengeCandidate,
) -> Option<BTreeSet<SemanticScopeKind>> {
    if candidate.disposition != CandidateDisposition::Selected {
        return None;
    }
    let mut kinds = BTreeSet::new();
    match candidate.selector.from {
        RelationKind::Binding => {
            kinds.insert(SemanticScopeKind::Binding);
        }
        RelationKind::Scenario => {
            kinds.insert(SemanticScopeKind::Scenario);
        }
        RelationKind::Check => {
            kinds.insert(SemanticScopeKind::Check);
        }
        RelationKind::Requirement => {
            kinds.insert(SemanticScopeKind::Requirement);
        }
        RelationKind::MethodQualification => {
            kinds.insert(SemanticScopeKind::MethodQualification);
        }
        RelationKind::Mechanism => {
            kinds.insert(SemanticScopeKind::Mechanism);
            if !add_mechanism_relation_kinds(model, &candidate.selector.id, &mut kinds) {
                return None;
            }
        }
        RelationKind::Realization => {
            if !stable_realization_anchor(model, &candidate.selector.id) {
                return None;
            }
            kinds.insert(SemanticScopeKind::Realization);
        }
    }
    let target = candidate.target.as_ref()?;
    match target.kind {
        DecisionKind::ApplicabilityDecision => {
            let Some(binding) = model
                .evidence_bindings()
                .find(|binding| binding.id == target.id)
            else {
                return None;
            };
            add_applicability_input_kinds(model, binding, &mut kinds)?;
        }
        DecisionKind::MethodQualification => {
            let qualification = model
                .method_qualifications()
                .find(|qualification| qualification.id == target.id)?;
            if model.expected_method_qualification_fingerprint(qualification)?
                != qualification.fingerprint
            {
                return None;
            }
            kinds.extend([
                SemanticScopeKind::MethodQualification,
                SemanticScopeKind::Check,
                SemanticScopeKind::Context,
                SemanticScopeKind::Policy,
            ]);
            if stable_check_implementations(model, &qualification.check)? {
                kinds.insert(SemanticScopeKind::CheckImplementation);
            }
        }
        DecisionKind::RequirementJudgment => {
            kinds.insert(SemanticScopeKind::RequirementJudgment);
            kinds.insert(SemanticScopeKind::Requirement);
            kinds.insert(SemanticScopeKind::Policy);
            if let Some(requirement) = model.find_requirement(&target.id) {
                if model
                    .realization_obligation(&requirement.spec.id, &requirement.requirement.id)
                    .is_some()
                {
                    kinds.insert(SemanticScopeKind::RealizationObligation);
                    kinds.insert(SemanticScopeKind::Area);
                }
            }
            if let Some(requirement) = model
                .requirements()
                .find(|requirement| requirement.id() == target.id)
            {
                if let Some(surface_id) = &requirement.requirement.over {
                    kinds.insert(SemanticScopeKind::Surface);
                    if let Some(surface) = model.workspace.surface(surface_id) {
                        if !surface.contributions.is_empty() {
                            kinds.insert(SemanticScopeKind::Area);
                            kinds.insert(SemanticScopeKind::Enumeration);
                        }
                    }
                    let behavioural = model
                        .specs
                        .iter()
                        .find(|spec| spec.id == *surface_id)
                        .into_iter()
                        .flat_map(|spec| &spec.requirements)
                        .filter(|requirement| requirement.domain == crate::model::Domain::Behaviour)
                        .map(|requirement| &requirement.id)
                        .collect::<BTreeSet<_>>();
                    let tagged_member = model.realizes.iter().any(|site| {
                        site.spec == *surface_id && behavioural.contains(&site.requirement)
                    });
                    let enumerated_member = model
                        .class_members
                        .iter()
                        .any(|member| member.class == *surface_id);
                    if tagged_member || enumerated_member {
                        kinds.insert(SemanticScopeKind::SurfaceMember);
                    }
                }
                if model
                    .realizes
                    .iter()
                    .any(|site| site.requirement == requirement.requirement.id)
                {
                    if !stable_requirement_realizations(model, &target.id) {
                        return None;
                    }
                    kinds.insert(SemanticScopeKind::Realization);
                }
                if let Some(design) = model.design_for(&requirement.spec.id) {
                    if let Some(entry) = design.for_requirement(&requirement.requirement.id) {
                        for mechanism in &entry.mechanisms {
                            kinds.insert(SemanticScopeKind::Mechanism);
                            if !add_mechanism_relation_kinds(model, &mechanism.id, &mut kinds) {
                                return None;
                            }
                        }
                    }
                }
            }
            let scenario_ids = model
                .find_requirement(&target.id)
                .into_iter()
                .flat_map(|requirement| &requirement.requirement.scenarios)
                .map(|scenario| scenario.id.as_str())
                .collect::<BTreeSet<_>>();
            for binding in model
                .evidence_bindings()
                .filter(|binding| scenario_ids.contains(binding.scenario.as_str()))
            {
                add_applicability_input_kinds(model, binding, &mut kinds)?;
            }
        }
    }
    Some(kinds)
}

fn add_applicability_input_kinds(
    model: &Model,
    binding: &crate::verification::EvidenceBinding,
    kinds: &mut BTreeSet<SemanticScopeKind>,
) -> Option<()> {
    model.scenario_digest(&binding.scenario)?;
    model.checks().find(|check| check.id == binding.check)?;
    model
        .decision_standards
        .as_ref()?
        .policies
        .iter()
        .find(|policy| policy.id == binding.policy)?;
    let qualification = model
        .method_qualifications()
        .find(|qualification| qualification.id == binding.method_qualification)?;
    let decision = model
        .applicability_decisions()
        .find(|decision| decision.id == binding.id)?;
    if model.expected_method_qualification_fingerprint(qualification)? != qualification.fingerprint
        || model.expected_applicability_fingerprint(binding)? != decision.fingerprint
    {
        return None;
    }
    kinds.extend([
        SemanticScopeKind::ApplicabilityDecision,
        SemanticScopeKind::MethodQualification,
        SemanticScopeKind::Binding,
        SemanticScopeKind::Scenario,
        SemanticScopeKind::Requirement,
        SemanticScopeKind::Check,
        SemanticScopeKind::Context,
        SemanticScopeKind::Policy,
    ]);
    if stable_check_implementations(model, &binding.check)? {
        kinds.insert(SemanticScopeKind::CheckImplementation);
    }
    Some(())
}

fn add_mechanism_relation_kinds(
    model: &Model,
    identity: &str,
    kinds: &mut BTreeSet<SemanticScopeKind>,
) -> bool {
    let mechanism_id = identity;
    let Some(design) = model.designs.iter().find(|design| {
        design.entries.iter().any(|entry| {
            entry
                .mechanisms
                .iter()
                .any(|mechanism| mechanism.id == identity)
        })
    }) else {
        return false;
    };
    let mechanisms = design
        .entries
        .iter()
        .flat_map(|entry| &entry.mechanisms)
        .filter(|mechanism| mechanism.id == mechanism_id)
        .collect::<Vec<_>>();
    let [mechanism] = mechanisms.as_slice() else {
        return false;
    };
    let bindings = model.mechanism_bindings(&design.spec, mechanism);
    let artifact_is_exact = !bindings.is_empty()
        && bindings.iter().all(|binding| {
            let artifacts = model
                .artifacts
                .iter()
                .filter(|artifact| artifact.id == *binding)
                .collect::<Vec<_>>();
            matches!(artifacts.as_slice(), [artifact] if artifact.source.is_some())
        });
    if !artifact_is_exact {
        return false;
    }
    kinds.insert(SemanticScopeKind::Artifact);
    let implementations = model
        .mechanism_implementations
        .iter()
        .filter(|implementation| implementation.mechanism == mechanism_id)
        .collect::<Vec<_>>();
    if mechanism.binding.is_none() {
        if implementations.is_empty()
            || implementations.iter().any(|implementation| {
                implementation.source.is_none()
                    || !valid_fingerprint(&implementation.source_fingerprint)
            })
        {
            return false;
        }
        kinds.insert(SemanticScopeKind::MechanismImplementation);
    } else if !implementations.is_empty() {
        return false;
    }
    true
}

fn stable_realization_anchor(model: &Model, identity: &str) -> bool {
    let sites = model
        .realizes
        .iter()
        .filter(|site| {
            site.source
                .as_ref()
                .is_some_and(|source| source.key() == identity)
        })
        .collect::<Vec<_>>();
    if sites.is_empty() {
        return false;
    }
    let fingerprints = sites
        .iter()
        .map(|site| site.source_fingerprint.as_str())
        .collect::<BTreeSet<_>>();
    fingerprints.len() == 1
        && fingerprints
            .first()
            .is_some_and(|fingerprint| valid_fingerprint(fingerprint))
}

fn stable_requirement_realizations(model: &Model, requirement_id: &str) -> bool {
    let requirement = requirement_id;
    let mut identities = BTreeMap::<String, String>::new();
    let sites = model
        .realizes
        .iter()
        .filter(|site| site.requirement == requirement)
        .collect::<Vec<_>>();
    if sites.is_empty() {
        return false;
    }
    sites.into_iter().all(|site| {
        let Some(source) = &site.source else {
            return false;
        };
        if !valid_fingerprint(&site.source_fingerprint) {
            return false;
        }
        match identities.get(&source.key()) {
            Some(existing) => existing == &site.source_fingerprint,
            None => {
                identities.insert(source.key(), site.source_fingerprint.clone());
                true
            }
        }
    })
}

fn stable_check_implementations(model: &Model, check: &str) -> Option<bool> {
    let mut identities = BTreeMap::<String, String>::new();
    let mut any = false;
    for implementation in model
        .check_implementations
        .iter()
        .filter(|implementation| implementation.check == check)
    {
        any = true;
        let source = implementation.source.as_ref()?;
        if !valid_fingerprint(&implementation.source_fingerprint) {
            return None;
        }
        match identities.get(&source.key()) {
            Some(existing) if existing != &implementation.source_fingerprint => return None,
            Some(_) => {}
            None => {
                identities.insert(source.key(), implementation.source_fingerprint.clone());
            }
        }
    }
    Some(any)
}

fn decision_entity_kind(kind: DecisionKind) -> crate::diag::EntityKind {
    match kind {
        DecisionKind::RequirementJudgment => crate::diag::EntityKind::Requirement,
        DecisionKind::ApplicabilityDecision | DecisionKind::MethodQualification => {
            crate::diag::EntityKind::Scenario
        }
    }
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
        criticality: None,
        path: path.to_string(),
        line,
        detail,
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn design_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();

    for design in &model.designs {
        let Some(spec) = model.specs.iter().find(|s| s.id == design.spec) else {
            findings.push(Finding {
                kind: FindingKind::DanglingDesignEntry,
                severity: Severity::Error,
                requirement: None,
                entity_kind: None,
                criticality: None,
                path: design.path.clone(),
                line: 1,
                detail: format!("designs spec `{}`, which does not exist", design.spec),
            });
            continue;
        };
        for entry in &design.entries {
            let id = entry.target.id();
            let requirement = spec
                .requirements
                .iter()
                .find(|requirement| requirement.id == id);
            if requirement.is_none() {
                findings.push(Finding {
                    kind: FindingKind::DanglingDesignEntry,
                    severity: Severity::Error,
                    requirement: Some(id.to_string()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: None,
                    path: design.path.clone(),
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
                                kind: FindingKind::DanglingDesignEntry,
                                severity: Severity::Error,
                                requirement: Some(id.to_string()),
                                entity_kind: Some(crate::diag::EntityKind::Requirement),
                                criticality: requirement.criticality,
                                path: design.path.clone(),
                                line: mechanism.line,
                                detail: format!(
                                    "mechanism `{}` names Scenario `{scenario}` that does not exist under Requirement `{id}`",
                                    mechanism.id
                                ),
                            });
                        }
                    }
                }
                let bindings = model.mechanism_bindings(&design.spec, mechanism);
                if bindings.is_empty() || (mechanism.binding.is_some() && bindings.len() != 1) {
                    findings.push(Finding {
                        kind: FindingKind::UnresolvedDesignBinding,
                        severity: Severity::Error,
                        requirement: Some(id.to_string()),
                        entity_kind: Some(crate::diag::EntityKind::Requirement),
                        criticality: None,
                        path: design.path.clone(),
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
                            kind: FindingKind::UnresolvedDesignBinding,
                            severity: Severity::Error,
                            requirement: Some(id.to_string()),
                            entity_kind: Some(crate::diag::EntityKind::Requirement),
                            criticality: None,
                            path: design.path.clone(),
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
                    (crate::design::Enforcement::Constraint, "database-index") => (artifact.unique
                        != Some(true))
                    .then_some("a non-unique index does not reject duplicate state".to_string()),
                    (kind, "database-index") if kind != crate::design::Enforcement::Constraint => {
                        Some(format!(
                            "a database index cannot establish `{}` enforcement",
                            kind.name()
                        ))
                    }
                    (crate::design::Enforcement::Type, "dotnet-method") => Some(
                        "type enforcement must bind the type, not one method on it".to_string(),
                    ),
                    (crate::design::Enforcement::ChokePoint, "dotnet-type") => Some(
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
                            criticality: None,
                            path: design.path.clone(),
                            line: mechanism.line,
                            detail: format!("binding `{}`: {}", binding, mismatches.join("; ")),
                        });
                    }
                }
            }
        }
    }

    // A design entry is required for `critical`, optional for `standard`, absent for
    // `routine`. Nothing here says the mechanism is missing from the code — only that its
    // strategy
    // is undeclared, and therefore that validation cannot compare the requirement against reality.
    //
    // Gated on the artifact being in use at all. Each mechanism must be usable alone
    // — `validate` without the design artifact — and a project that has not adopted it must not
    // be told that every critical Requirement is a finding. Partial adoption still reports: one
    // design file means the artifact is in use, and the specs it omits are visible.
    for spec in &model.specs {
        if model.designs.is_empty() {
            break;
        }
        let design = model.design_for(&spec.id);
        for requirement in &spec.requirements {
            if requirement.criticality != Some(Criticality::Critical) {
                continue;
            }
            let declared = design.is_some_and(|d| d.for_requirement(&requirement.id).is_some());
            if !declared {
                findings.push(Finding {
                    kind: FindingKind::UndeclaredMechanism,
                    severity: Severity::Error,
                    requirement: Some(requirement.id.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: requirement.criticality,
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: "critical Requirement declares no enforcement mechanism".into(),
                });
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
                    severity: severity_for(requirement.criticality),
                    requirement: Some(requirement_id),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: requirement.criticality,
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
                    criticality: requirement.criticality,
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
                    severity: severity_for(requirement.criticality),
                    requirement: Some(requirement_id),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: requirement.criticality,
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
                    severity: severity_for(requirement.criticality),
                    requirement: Some(requirement.id.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: requirement.criticality,
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
    for design in &model.account_designs {
        for declaration in &design.declarations {
            if declaration.kind != crate::account_design::DeclarationKind::Requirement {
                continue;
            }
            let spec = design.spec.as_str();
            let requirement = declaration.id.as_str();
            if model.workspace.obligation(spec, requirement).is_some() {
                findings.push(Finding {
                    kind: FindingKind::DanglingRealizationObligation,
                    severity: Severity::Error,
                    requirement: Some(declaration.id.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: model.find_requirement(requirement).and_then(|view| view.requirement.criticality),
                    path: design.path.clone(),
                    line: declaration.line,
                    detail: "Requirement Areas are authored in both design and workspace; remove the workspace realization obligation".into(),
                });
            }
        }
    }
    for obligation in &model.realization_obligations() {
        let design_location =
            model.account_design_for_requirement(&obligation.spec, &obligation.requirement);
        let path = design_location.map_or(model.workspace.path.as_str(), |(design, _)| {
            design.path.as_str()
        });
        let line = design_location.map_or(0, |(_, declaration)| declaration.line);
        let Some(requirement) = model.find_requirement(&obligation.requirement) else {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationObligation,
                severity: Severity::Error,
                requirement: Some(obligation.requirement.clone()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                criticality: None,
                path: path.to_string(),
                line,
                detail: "realization obligation names a requirement that does not exist".into(),
            });
            continue;
        };
        if requirement.requirement.domain != crate::model::Domain::Behaviour
            || !matches!(
                requirement.requirement.criticality,
                Some(Criticality::Standard | Criticality::Critical)
            )
        {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationObligation,
                severity: Severity::Error,
                requirement: Some(requirement.id()),
                entity_kind: Some(crate::diag::EntityKind::Requirement),
                criticality: requirement.requirement.criticality,
                path: path.to_string(),
                line,
                detail:
                    "area obligations apply only to standard or critical behavioral requirements"
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
                    severity: severity_for(requirement.requirement.criticality),
                    requirement: Some(requirement.id()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: requirement.requirement.criticality,
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

fn account_mechanism_declared(item: &crate::account_design::Declaration, id: &str) -> bool {
    (item.kind == crate::account_design::DeclarationKind::Mechanism && item.id == id)
        || item
            .children
            .iter()
            .any(|child| account_mechanism_declared(child, id))
}

fn account_mechanism_findings(
    model: &Model,
    design: &crate::account_design::AccountDesign,
    item: &crate::account_design::Declaration,
    enclosing_requirement: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    if item.kind == crate::account_design::DeclarationKind::Scenario
        && !model
            .find_scenario(&item.id)
            .is_some_and(|view| enclosing_requirement == Some(view.requirement.id.as_str()))
    {
        findings.push(Finding {
            kind: FindingKind::DanglingDesignEntry,
            severity: Severity::Error,
            requirement: Some(item.id.clone()),
            entity_kind: Some(crate::diag::EntityKind::Scenario),
            criticality: None,
            path: design.path.clone(),
            line: item.line,
            detail: format!(
                "Scenario design `{}` names no Scenario owned by its enclosing Requirement",
                item.id
            ),
        });
    }
    if item.kind == crate::account_design::DeclarationKind::Mechanism {
        for scenario in &item.scenarios {
            if model.find_scenario(scenario).is_none() {
                findings.push(Finding {
                    kind: FindingKind::DanglingDesignEntry,
                    severity: Severity::Error,
                    requirement: Some(scenario.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Scenario),
                    criticality: None,
                    path: design.path.clone(),
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
                    criticality: None,
                    path: design.path.clone(),
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
        let owner = if item.kind == crate::account_design::DeclarationKind::Requirement {
            Some(item.id.as_str())
        } else {
            enclosing_requirement
        };
        account_mechanism_findings(model, design, child, owner, findings);
    }
}

fn account_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    for design in &model.account_designs {
        for item in &design.declarations {
            account_mechanism_findings(model, design, item, None, &mut findings);
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
                    criticality: None,
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
                        criticality: target.requirement.criticality,
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
                        criticality: target.requirement.criticality,
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
                        criticality: target.requirement.criticality,
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
                    criticality: None,
                    path: support.path.clone(),
                    line: 0,
                    detail: format!("Check `{}` has no explicit Scenario contribution", check.id),
                });
            }
        }
    }
    for review in &model.account_reviews {
        for entry in &review.requirements {
            let Some(target) = model.find_requirement(&entry.requirement) else {
                findings.push(Finding {
                    kind: FindingKind::InvalidRequirementReview,
                    severity: Severity::Error,
                    requirement: Some(entry.requirement.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: None,
                    path: review.path.clone(),
                    line: entry.line,
                    detail: format!(
                        "Requirement review `{}` names no spec Requirement",
                        entry.requirement
                    ),
                });
                continue;
            };
            let spec = target.spec.id.as_str();
            let requirement = target.requirement.id.as_str();
            let expected = target
                .requirement
                .scenarios
                .iter()
                .map(|scenario| scenario.id.clone())
                .collect::<BTreeSet<_>>();
            let actual = entry.scenarios.iter().cloned().collect::<BTreeSet<_>>();
            let has_design = entry.design == entry.requirement
                && model
                    .account_design_for_requirement(spec, requirement)
                    .is_some();
            let has_verification = entry.verification == entry.requirement
                && model.account_verifications.iter().any(|facet| {
                    facet.owner == spec
                        && facet
                            .requirements
                            .iter()
                            .any(|scope| scope.requirement == entry.requirement)
                });
            if expected != actual
                || actual.len() != entry.scenarios.len()
                || !has_design
                || !has_verification
            {
                findings.push(Finding {
                    kind: FindingKind::InvalidRequirementReview,
                    severity: Severity::Error,
                    requirement: Some(entry.requirement.clone()),
                    entity_kind: Some(crate::diag::EntityKind::Requirement),
                    criticality: target.requirement.criticality,
                    path: review.path.clone(),
                    line: entry.line,
                    detail: format!("Requirement review `{}` has mismatched Scenarios, Design, or Verification references", entry.requirement),
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
