//! Deterministic validation and Challenge Plan resolution over the alpha 5 model.

use crate::json::Json;
use crate::model::{Criticality, Model};
use crate::verification::{
    ChallengeDomain, ChallengePlan, ClaimJudgmentVerdict, MethodQualificationVerdict, Selector,
    SemanticScopeKind,
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
    UnboundCase,
    CheckWithoutBinding,
    BindingMissingCheck,
    BindingMissingCase,
    BindingMissingPolicy,
    MissingMethodQualification,
    DanglingMethodQualification,
    RejectedMethodQualification,
    StaleMethodQualification,
    MissingApplicabilityDecision,
    DanglingApplicabilityDecision,
    RejectedApplicabilityDecision,
    StaleApplicabilityDecision,
    MissingClaimJudgment,
    RejectedClaimJudgment,
    StaleClaimJudgment,
    InvalidClaimJudgment,
    UnimplementedCheck,
    DanglingCheckImplementation,
    UnstableCheckImplementation,
    InapplicableVerification,
    MissingChallenger,
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
            Self::UnboundCase => "unbound-case",
            Self::CheckWithoutBinding => "check-without-binding",
            Self::BindingMissingCheck => "binding-missing-check",
            Self::BindingMissingCase => "binding-missing-case",
            Self::BindingMissingPolicy => "binding-missing-policy",
            Self::MissingMethodQualification => "missing-method-qualification",
            Self::DanglingMethodQualification => "dangling-method-qualification",
            Self::RejectedMethodQualification => "rejected-method-qualification",
            Self::StaleMethodQualification => "stale-method-qualification",
            Self::MissingApplicabilityDecision => "missing-applicability-decision",
            Self::DanglingApplicabilityDecision => "dangling-applicability-decision",
            Self::RejectedApplicabilityDecision => "rejected-applicability-decision",
            Self::StaleApplicabilityDecision => "stale-applicability-decision",
            Self::MissingClaimJudgment => "missing-claim-judgment",
            Self::RejectedClaimJudgment => "rejected-claim-judgment",
            Self::StaleClaimJudgment => "stale-claim-judgment",
            Self::InvalidClaimJudgment => "invalid-claim-judgment",
            Self::UnimplementedCheck => "unimplemented-check",
            Self::DanglingCheckImplementation => "dangling-check-implementation",
            Self::UnstableCheckImplementation => "unstable-check-implementation",
            Self::InapplicableVerification => "inapplicable-verification",
            Self::MissingChallenger => "missing-challenger",
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
            | Self::MissingClaimJudgment
            | Self::RejectedClaimJudgment
            | Self::StaleClaimJudgment
            | Self::InvalidClaimJudgment => FindingCategory::Judgment,
            Self::MissingChallenger
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
            Self::Unclassified => "Declare the Claim's criticality explicitly.",
            Self::Unrealized => "Link production code that establishes the Claim predicate.",
            Self::DanglingRealization => {
                "Retarget or remove the production link to the unknown Claim."
            }
            Self::DanglingDesignEntry => "Retarget or remove the design entry.",
            Self::UndeclaredMechanism => "Declare how the critical Claim is enforced.",
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
            Self::UnboundCase => "Bind at least one deliberately enrolled Check to the Case.",
            Self::CheckWithoutBinding => "Add an Evidence Binding or remove the unused Check.",
            Self::BindingMissingCheck => "Retarget the binding to a declared Check.",
            Self::BindingMissingCase => "Retarget the binding to a current Case.",
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
            Self::MissingClaimJudgment => "Record the Claim's total-composition Judgment.",
            Self::RejectedClaimJudgment => "Resolve the objection before accepting the Claim.",
            Self::StaleClaimJudgment => "Rejudge the Claim against its current composition.",
            Self::InvalidClaimJudgment => {
                "Repair the Claim composition before recording a current Judgment."
            }
            Self::UnimplementedCheck => {
                "Mark at least one stable source implementation of the Check."
            }
            Self::DanglingCheckImplementation => "Retarget the source marker to a declared Check.",
            Self::UnstableCheckImplementation => {
                "Resolve semantic source identity and fingerprint."
            }
            Self::InapplicableVerification => "Remove verification from the routine Claim.",
            Self::MissingChallenger => "Retarget the Challenge Plan to a declared Challenger.",
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
                "Declare a Challenger and Plan for every policy-required form."
            }
            Self::InsufficientChallengeScope => {
                "Select the decision through scope covering every Challenger requirement."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: Severity,
    pub claim: Option<String>,
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
                "claim",
                self.claim.as_ref().map(Json::str).unwrap_or(Json::Null),
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
    ClaimJudgment,
    MethodQualification,
}

impl DecisionKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::ApplicabilityDecision => "applicability-decision",
            Self::ClaimJudgment => "claim-judgment",
            Self::MethodQualification => "method-qualification",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationKind {
    Binding,
    Case,
    Check,
    Claim,
    MethodQualification,
    Mechanism,
    Realization,
}

impl RelationKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::Case => "case",
            Self::Check => "check",
            Self::Claim => "claim",
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
    pub challenger: String,
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
        challenger: plan.challenger.clone(),
        candidates: candidates.into_values().collect(),
        issues,
    }
}

impl ChallengeResolution {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("format", Json::str("azimuth-challenge-resolution")),
            ("version", Json::Num(1.0)),
            ("plan", Json::str(&self.plan)),
            ("challenger", Json::str(&self.challenger)),
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
        Selector::ApplicabilityDecisionFromCase(id) => {
            selector_id(DecisionKind::ApplicabilityDecision, RelationKind::Case, id)
        }
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
        Selector::ClaimJudgmentFromClaim(id) => {
            selector_id(DecisionKind::ClaimJudgment, RelationKind::Claim, id)
        }
        Selector::ClaimJudgmentFromRealization(id) => {
            selector_id(DecisionKind::ClaimJudgment, RelationKind::Realization, id)
        }
        Selector::ClaimJudgmentFromMechanism(id) => {
            selector_id(DecisionKind::ClaimJudgment, RelationKind::Mechanism, id)
        }
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
                cases_for_realization(model, &selector.id),
                ChallengeDomain::Realization,
            )
        }
        (DecisionKind::MethodQualification, RelationKind::Mechanism) => {
            method_qualification_relation_candidates(
                model,
                selector,
                cases_for_mechanism(model, &selector.id),
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
        (DecisionKind::ApplicabilityDecision, RelationKind::Case) => {
            let bindings = model
                .evidence_bindings()
                .filter(|binding| binding.case == selector.id)
                .collect::<Vec<_>>();
            if model.find_case(&selector.id).is_none() || bindings.is_empty() {
                vec![unresolved(selector, RelationKind::Case, &selector.id)]
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
                cases_for_realization(model, &selector.id),
                ChallengeDomain::Realization,
            )
        }
        (DecisionKind::ApplicabilityDecision, RelationKind::Mechanism) => {
            applicability_relation_candidates(
                model,
                selector,
                cases_for_mechanism(model, &selector.id),
                ChallengeDomain::Mechanism,
            )
        }
        (DecisionKind::ClaimJudgment, RelationKind::Claim) => {
            if model.claims().any(|claim| claim.id() == selector.id) {
                vec![judgment_candidate(model, selector, &selector.id)]
            } else {
                vec![unresolved(selector, RelationKind::Claim, &selector.id)]
            }
        }
        (DecisionKind::ClaimJudgment, RelationKind::Realization) => judgment_relation_candidates(
            model,
            selector,
            claims_for_realization(model, &selector.id),
        ),
        (DecisionKind::ClaimJudgment, RelationKind::Mechanism) => {
            judgment_relation_candidates(model, selector, claims_for_mechanism(model, &selector.id))
        }
        _ => unreachable!("the parser admits only supported selector forms"),
    }
}

fn claims_for_realization(model: &Model, identity: &str) -> BTreeSet<String> {
    model
        .realizes
        .iter()
        .filter(|site| {
            site.source
                .as_ref()
                .is_some_and(|source| source.key() == identity)
                && model.has_claim(&site.spec, &site.claim)
        })
        .map(|site| format!("{}#{}", site.spec, site.claim))
        .collect()
}

fn claims_for_mechanism(model: &Model, identity: &str) -> BTreeSet<String> {
    let Some((spec_id, mechanism_id)) = identity.split_once('#') else {
        return BTreeSet::new();
    };
    let Some(design) = model.design_for(spec_id) else {
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
        .map(|entry| format!("{spec_id}#{}", entry.target.id()))
        .collect()
}

fn cases_for_realization(model: &Model, identity: &str) -> BTreeSet<String> {
    claims_for_realization(model, identity)
        .into_iter()
        .flat_map(|claim_id| {
            model
                .claims()
                .find(|claim| claim.id() == claim_id)
                .into_iter()
                .flat_map(|claim| {
                    claim.claim.cases.iter().map(move |case| {
                        format!("{}#{}/{}", claim.spec.id, claim.claim.id, case.id)
                    })
                })
        })
        .collect()
}

fn cases_for_mechanism(model: &Model, identity: &str) -> BTreeSet<String> {
    let Some((spec_id, mechanism_id)) = identity.split_once('#') else {
        return BTreeSet::new();
    };
    let Some(design) = model.design_for(spec_id) else {
        return BTreeSet::new();
    };
    let mut cases = BTreeSet::new();
    for entry in &design.entries {
        let Some(claim) = model.find_claim(spec_id, entry.target.id()) else {
            continue;
        };
        for mechanism in entry
            .mechanisms
            .iter()
            .filter(|mechanism| mechanism.id == mechanism_id)
        {
            for case in &claim.claim.cases {
                if mechanism.cases.is_empty() || mechanism.cases.contains(&case.id) {
                    cases.insert(format!("{spec_id}#{}/{}", claim.claim.id, case.id));
                }
            }
        }
    }
    cases
}

fn applicability_relation_candidates(
    model: &Model,
    selector: &SelectorIdentity,
    cases: BTreeSet<String>,
    domain: ChallengeDomain,
) -> Vec<ChallengeCandidate> {
    if cases.is_empty() {
        return vec![unresolved(selector, selector.from, &selector.id)];
    }
    let mut candidates = Vec::new();
    for case in cases {
        let bindings = model
            .evidence_bindings()
            .filter(|binding| binding.case == case)
            .collect::<Vec<_>>();
        if bindings.is_empty() {
            candidates.push(unresolved(selector, RelationKind::Case, &case));
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
    cases: BTreeSet<String>,
    domain: ChallengeDomain,
) -> Vec<ChallengeCandidate> {
    if cases.is_empty() {
        return vec![unresolved(selector, selector.from, &selector.id)];
    }
    let qualification_ids = model
        .evidence_bindings()
        .filter(|binding| cases.contains(&binding.case))
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
    claims: BTreeSet<String>,
) -> Vec<ChallengeCandidate> {
    if claims.is_empty() {
        return vec![unresolved(selector, selector.from, &selector.id)];
    }
    claims
        .iter()
        .map(|claim| judgment_candidate(model, selector, claim))
        .collect()
}

fn applicability_candidate(
    model: &Model,
    selector: &SelectorIdentity,
    binding: &crate::verification::EvidenceBinding,
    relation_applicable: bool,
) -> ChallengeCandidate {
    let claim = model.cases().find(|claim| claim.id() == binding.case);
    let decision = model
        .applicability_decisions()
        .find(|decision| decision.id == binding.id);
    let expected = model.expected_applicability_fingerprint(binding);
    let authored = decision.map(|decision| decision.fingerprint.clone());
    let inapplicable = !relation_applicable
        || claim
            .as_ref()
            .is_some_and(|claim| claim.claim.criticality == Some(Criticality::Routine));
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
            && model.find_case(&binding.case).is_some_and(|case| {
                matches!(
                    case.claim.criticality,
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
    claim_id: &str,
) -> ChallengeCandidate {
    let claim = model.claims().find(|claim| claim.id() == claim_id);
    let judgment = model
        .claim_judgments()
        .find(|judgment| judgment.id == claim_id);
    let expected =
        judgment.and_then(|judgment| model.expected_claim_judgment_fingerprint(judgment));
    let authored = judgment.map(|judgment| judgment.fingerprint.clone());
    let disposition = if claim
        .as_ref()
        .is_some_and(|claim| claim.claim.criticality == Some(Criticality::Routine))
    {
        CandidateDisposition::Inapplicable
    } else if judgment.is_none() {
        CandidateDisposition::MissingDecision
    } else if expected.is_none() {
        CandidateDisposition::InvalidDecision
    } else if authored.as_ref() != expected.as_ref() {
        CandidateDisposition::StaleDecision
    } else if judgment.is_some_and(|judgment| judgment.verdict == ClaimJudgmentVerdict::Rejected) {
        CandidateDisposition::RejectedDecision
    } else {
        CandidateDisposition::Selected
    };
    ChallengeCandidate {
        selector: selector.clone(),
        relation: RelationIdentity {
            kind: RelationKind::Claim,
            id: claim_id.to_string(),
        },
        target: Some(DecisionCandidateTarget {
            kind: DecisionKind::ClaimJudgment,
            id: claim_id.to_string(),
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
        for claim in &spec.claims {
            if claim.criticality.is_none() {
                findings.push(Finding {
                    kind: FindingKind::Unclassified,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", spec.id, claim.id)),
                    criticality: None,
                    path: spec.path.clone(),
                    line: claim.line,
                    detail: format!("Claim `{}` declares no criticality", claim.id),
                });
            }
        }
    }
    for claim in model.claims() {
        if claim.claim.criticality == Some(Criticality::Routine) {
            continue;
        }
        if !model
            .realizes
            .iter()
            .any(|site| site.spec == claim.spec.id && site.claim == claim.claim.id)
        {
            findings.push(Finding {
                kind: FindingKind::Unrealized,
                severity: severity_for(claim.claim.criticality),
                claim: Some(claim.id()),
                criticality: claim.claim.criticality,
                path: claim.spec.path.clone(),
                line: claim.claim.line,
                detail: "no production code realizes this Claim".into(),
            });
        }
    }
    for site in &model.realizes {
        if !model.has_claim(&site.spec, &site.claim) {
            findings.push(Finding {
                kind: FindingKind::DanglingRealization,
                severity: Severity::Error,
                claim: Some(format!("{}#{}", site.spec, site.claim)),
                criticality: None,
                path: site.file.clone(),
                line: 0,
                detail: format!("`{}` realizes a Claim that does not exist", site.site),
            });
        }
    }
    for implementation in &model.mechanism_implementations {
        let declared = model
            .design_for(&implementation.spec)
            .is_some_and(|design| {
                design
                    .entries
                    .iter()
                    .flat_map(|entry| &entry.mechanisms)
                    .any(|mechanism| mechanism.id == implementation.mechanism)
            });
        if !declared {
            findings.push(Finding {
                kind: FindingKind::DanglingMechanismImplementation,
                severity: Severity::Error,
                claim: Some(format!(
                    "{}#{}",
                    implementation.spec, implementation.mechanism
                )),
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
    findings.extend(surface_findings(model));
    findings.sort_by(|left, right| {
        (
            &left.path,
            left.line,
            left.kind.name(),
            &left.claim,
            &left.detail,
        )
            .cmp(&(
                &right.path,
                right.line,
                right.kind.name(),
                &right.claim,
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
    for case in model.cases() {
        let bindings = model
            .evidence_bindings()
            .filter(|binding| binding.case == case.id())
            .collect::<Vec<_>>();
        if case.claim.criticality == Some(Criticality::Routine) {
            for binding in bindings {
                findings.push(Finding {
                    kind: FindingKind::InapplicableVerification,
                    severity: Severity::Warning,
                    claim: Some(case.id()),
                    criticality: case.claim.criticality,
                    path: binding.path.clone(),
                    line: binding.line,
                    detail: format!("Evidence Binding `{}` targets a routine Case", binding.id),
                });
            }
        } else if case.claim.criticality.is_some() && bindings.is_empty() {
            findings.push(Finding {
                kind: FindingKind::UnboundCase,
                severity: severity_for(case.claim.criticality),
                claim: Some(case.id()),
                criticality: case.claim.criticality,
                path: case.spec.path.clone(),
                line: case.case.line,
                detail: "non-routine Case has no Evidence Binding".into(),
            });
        }
    }
    for claim in model.claims() {
        if claim.claim.criticality == Some(Criticality::Routine) {
            for judgment in model
                .claim_judgments()
                .filter(|judgment| judgment.id == claim.id())
            {
                findings.push(Finding {
                    kind: FindingKind::InapplicableVerification,
                    severity: Severity::Warning,
                    claim: Some(claim.id()),
                    criticality: claim.claim.criticality,
                    path: judgment.path.clone(),
                    line: judgment.line,
                    detail: format!("Claim Judgment `{}` targets a routine Claim", judgment.id),
                });
            }
            continue;
        }
        if !matches!(
            claim.claim.criticality,
            Some(Criticality::Standard | Criticality::Critical)
        ) {
            continue;
        }
        let judgment = model
            .claim_judgments()
            .find(|judgment| judgment.id == claim.id());
        match judgment {
            None => findings.push(Finding {
                kind: FindingKind::MissingClaimJudgment,
                severity: Severity::Error,
                claim: Some(claim.id()),
                criticality: claim.claim.criticality,
                path: claim.spec.path.clone(),
                line: claim.claim.line,
                detail: "non-routine Claim has no total-composition Judgment".into(),
            }),
            Some(judgment) => {
                let expected = model.expected_claim_judgment_fingerprint(judgment);
                if expected.is_none() {
                    findings.push(simple(
                        FindingKind::InvalidClaimJudgment,
                        &judgment.path,
                        judgment.line,
                        Some(claim.id()),
                        format!(
                            "Claim Judgment `{}` has unavailable expected composition",
                            judgment.id
                        ),
                    ));
                } else if expected.as_ref() != Some(&judgment.fingerprint) {
                    findings.push(simple(
                        FindingKind::StaleClaimJudgment,
                        &judgment.path,
                        judgment.line,
                        Some(claim.id()),
                        format!(
                            "Claim Judgment `{}` expected {}, found {}",
                            judgment.id,
                            expected.as_deref().unwrap_or_default(),
                            judgment.fingerprint
                        ),
                    ));
                } else if judgment.verdict == ClaimJudgmentVerdict::Rejected {
                    findings.push(simple(
                        FindingKind::RejectedClaimJudgment,
                        &judgment.path,
                        judgment.line,
                        Some(claim.id()),
                        format!("Claim Judgment `{}` is rejected", judgment.id),
                    ));
                }
            }
        }
    }
    for check in model.checks() {
        if !model
            .evidence_bindings()
            .any(|binding| binding.check == check.id)
        {
            findings.push(simple(
                FindingKind::CheckWithoutBinding,
                &check.path,
                check.line,
                None,
                format!("Check `{}` has no Evidence Binding", check.id),
            ));
        }
        let has_applicable_binding = model.evidence_bindings().any(|binding| {
            binding.check == check.id
                && model
                    .cases()
                    .find(|claim| claim.id() == binding.case)
                    .is_some_and(|claim| {
                        matches!(
                            claim.claim.criticality,
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
                format!("Check `{}` has no source implementation", check.id),
            ));
        }
    }
    for implementation in &model.check_implementations {
        let check_exists = model.checks().any(|check| check.id == implementation.check);
        if !check_exists {
            findings.push(simple(
                FindingKind::DanglingCheckImplementation,
                &implementation.file,
                0,
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
                    .cases()
                    .find(|claim| claim.id() == binding.case)
                    .is_some_and(|claim| {
                        matches!(
                            claim.claim.criticality,
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
                format!(
                    "Check `{}` lacks stable semantic identity or source fingerprint",
                    implementation.check
                ),
            ));
        }
    }
    for binding in model.evidence_bindings() {
        let claim = model.cases().find(|claim| claim.id() == binding.case);
        if claim
            .as_ref()
            .is_some_and(|claim| claim.claim.criticality == Some(Criticality::Routine))
        {
            continue;
        }
        if !model.checks().any(|check| check.id == binding.check) {
            findings.push(simple(
                FindingKind::BindingMissingCheck,
                &binding.path,
                binding.line,
                Some(binding.case.clone()),
                format!(
                    "binding `{}` names unknown Check `{}`",
                    binding.id, binding.check
                ),
            ));
        }
        if claim.is_none() {
            findings.push(simple(
                FindingKind::BindingMissingCase,
                &binding.path,
                binding.line,
                Some(binding.case.clone()),
                format!("binding `{}` names no current Case", binding.id),
            ));
        }
        if !policies.iter().any(|policy| policy.id == binding.policy) {
            findings.push(simple(
                FindingKind::BindingMissingPolicy,
                &binding.path,
                binding.line,
                Some(binding.case.clone()),
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
                Some(binding.case.clone()),
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
                Some(binding.case.clone()),
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
                Some(binding.case.clone()),
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
                    Some(binding.case.clone()),
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
                    Some(binding.case.clone()),
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
                format!(
                    "Method Qualification `{}` has no current Check-to-Case edge",
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
                format!(
                    "Applicability Decision `{}` names no Evidence Binding",
                    decision.id
                ),
            ));
        }
    }
    for plan in model.challenge_plans() {
        if !model
            .challengers()
            .any(|challenger| challenger.id == plan.challenger)
        {
            findings.push(simple(
                FindingKind::MissingChallenger,
                &plan.path,
                plan.line,
                None,
                format!(
                    "Challenge Plan `{}` names unknown Challenger `{}`",
                    plan.id, plan.challenger
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
                candidate_target_claim(model, candidate),
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

fn candidate_target_claim(model: &Model, candidate: &ChallengeCandidate) -> Option<String> {
    let target = candidate.target.as_ref()?;
    match target.kind {
        DecisionKind::ClaimJudgment => Some(target.id.clone()),
        DecisionKind::ApplicabilityDecision => model
            .evidence_bindings()
            .find(|binding| binding.id == target.id)
            .map(|binding| binding.case.clone()),
        DecisionKind::MethodQualification => model
            .evidence_bindings()
            .filter(|binding| binding.method_qualification == target.id)
            .map(|binding| binding.case.clone())
            .next(),
    }
}

/// Selection keeps an authored Plan atomically when any one of its selectors touches the selected
/// semantic graph. Applicability and decision freshness deliberately do not participate: pruning
/// only successful selectors would hide adverse siblings from the derived account.
pub fn challenge_plan_relevant_to_selection(
    model: &Model,
    plan: &ChallengePlan,
    claims: &BTreeSet<String>,
    bindings: &BTreeSet<String>,
    checks: &BTreeSet<String>,
) -> bool {
    plan.selectors.iter().any(|selector| match selector {
        Selector::MethodQualificationFromMethodQualification(id) => model
            .evidence_bindings()
            .any(|binding| binding.method_qualification == *id && claims.contains(&binding.case)),
        Selector::MethodQualificationFromCheck(id)
        | Selector::ApplicabilityDecisionFromCheck(id) => checks.contains(id),
        Selector::ApplicabilityDecisionFromBinding(id) => bindings.contains(id),
        Selector::ApplicabilityDecisionFromCase(id) => claims.contains(id),
        Selector::MethodQualificationFromRealization(identity)
        | Selector::ApplicabilityDecisionFromRealization(identity) => {
            cases_for_realization(model, identity)
                .iter()
                .any(|claim| claims.contains(claim))
        }
        Selector::MethodQualificationFromMechanism(identity)
        | Selector::ApplicabilityDecisionFromMechanism(identity) => {
            cases_for_mechanism(model, identity)
                .iter()
                .any(|claim| claims.contains(claim))
        }
        Selector::ClaimJudgmentFromClaim(id) => claims
            .iter()
            .any(|case| case.starts_with(&format!("{id}/"))),
        Selector::ClaimJudgmentFromRealization(identity) => claims_for_realization(model, identity)
            .iter()
            .any(|parent| {
                claims
                    .iter()
                    .any(|case| case.starts_with(&format!("{parent}/")))
            }),
        Selector::ClaimJudgmentFromMechanism(identity) => {
            claims_for_mechanism(model, identity).iter().any(|parent| {
                claims
                    .iter()
                    .any(|case| case.starts_with(&format!("{parent}/")))
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
    for judgment in model.claim_judgments() {
        let selector = selector_id(
            DecisionKind::ClaimJudgment,
            RelationKind::Claim,
            &judgment.id,
        );
        let candidate = judgment_candidate(model, &selector, &judgment.id);
        if candidate.disposition == CandidateDisposition::Selected {
            decisions.push((
                candidate.target.expect("a reached Claim has a target"),
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
            let challengers = model
                .challengers()
                .filter(|challenger| challenger.form == *form)
                .collect::<Vec<_>>();
            if challengers.is_empty() {
                findings.push(simple(
                    FindingKind::MissingRequiredChallenge,
                    &path,
                    line,
                    target_claim(model, &target),
                    format!(
                        "{} `{}` requires Challenge form `{form}`, which has no Challenger",
                        target.kind.name(),
                        target.id
                    ),
                ));
                continue;
            }

            let mut reaches_target = false;
            let covered = challengers.iter().any(|challenger| {
                plans.iter().any(|(plan, resolution)| {
                    if plan.challenger != challenger.id || !resolution.is_runnable() {
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
                    let kinds =
                        matching
                            .into_iter()
                            .try_fold(BTreeSet::new(), |mut union, candidate| {
                                union.extend(challenge_candidate_scope_kinds(model, candidate)?);
                                Some(union)
                            });
                    let Some(kinds) = kinds else {
                        return false;
                    };
                    challenger
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
                    target_claim(model, &target),
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

fn target_claim(model: &Model, target: &DecisionCandidateTarget) -> Option<String> {
    match target.kind {
        DecisionKind::ClaimJudgment => Some(target.id.clone()),
        DecisionKind::ApplicabilityDecision => model
            .evidence_bindings()
            .find(|binding| binding.id == target.id)
            .map(|binding| binding.case.clone()),
        DecisionKind::MethodQualification => model
            .evidence_bindings()
            .find(|binding| binding.method_qualification == target.id)
            .map(|binding| binding.case.clone()),
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
        RelationKind::Case => {
            kinds.insert(SemanticScopeKind::Case);
        }
        RelationKind::Check => {
            kinds.insert(SemanticScopeKind::Check);
        }
        RelationKind::Claim => {
            kinds.insert(SemanticScopeKind::Claim);
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
        DecisionKind::ClaimJudgment => {
            kinds.insert(SemanticScopeKind::ClaimJudgment);
            kinds.insert(SemanticScopeKind::Claim);
            kinds.insert(SemanticScopeKind::Policy);
            if let Some((spec, claim)) = target.id.split_once('#') {
                if model.workspace.obligation(spec, claim).is_some() {
                    kinds.insert(SemanticScopeKind::RealizationObligation);
                    kinds.insert(SemanticScopeKind::Area);
                }
            }
            if let Some(claim) = model.claims().find(|claim| claim.id() == target.id) {
                if let Some(surface_id) = &claim.claim.over {
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
                        .flat_map(|spec| &spec.claims)
                        .filter(|claim| claim.domain == crate::model::Domain::Behaviour)
                        .map(|claim| &claim.id)
                        .collect::<BTreeSet<_>>();
                    let tagged_member = model
                        .realizes
                        .iter()
                        .any(|site| site.spec == *surface_id && behavioural.contains(&site.claim));
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
                    .any(|site| site.spec == claim.spec.id && site.claim == claim.claim.id)
                {
                    if !stable_claim_realizations(model, &target.id) {
                        return None;
                    }
                    kinds.insert(SemanticScopeKind::Realization);
                }
                if let Some(design) = model.design_for(&claim.spec.id) {
                    if let Some(entry) = design.for_claim(&claim.claim.id) {
                        for mechanism in &entry.mechanisms {
                            kinds.insert(SemanticScopeKind::Mechanism);
                            if !add_mechanism_relation_kinds(
                                model,
                                &format!("{}#{}", claim.spec.id, mechanism.id),
                                &mut kinds,
                            ) {
                                return None;
                            }
                        }
                    }
                }
            }
            let case_prefix = format!("{}/", target.id);
            for binding in model
                .evidence_bindings()
                .filter(|binding| binding.case.starts_with(&case_prefix))
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
    model.case_digest(&binding.case)?;
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
        SemanticScopeKind::Case,
        SemanticScopeKind::Claim,
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
    let Some((spec_id, mechanism_id)) = identity.split_once('#') else {
        return false;
    };
    let Some(design) = model.design_for(spec_id) else {
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
    let bindings = model.mechanism_bindings(spec_id, mechanism);
    let artifact_is_exact = if let [binding] = bindings.as_slice() {
        let artifacts = model
            .artifacts
            .iter()
            .filter(|artifact| artifact.id == *binding)
            .collect::<Vec<_>>();
        matches!(artifacts.as_slice(), [artifact] if artifact.source.is_some())
    } else {
        false
    };
    if !artifact_is_exact {
        return false;
    }
    kinds.insert(SemanticScopeKind::Artifact);
    let implementations = model
        .mechanism_implementations
        .iter()
        .filter(|implementation| {
            implementation.spec == spec_id && implementation.mechanism == mechanism_id
        })
        .collect::<Vec<_>>();
    if mechanism.binding.is_none() {
        let [implementation] = implementations.as_slice() else {
            return false;
        };
        if implementation.source.is_none() || !valid_fingerprint(&implementation.source_fingerprint)
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

fn stable_claim_realizations(model: &Model, claim_id: &str) -> bool {
    let Some((spec, claim)) = claim_id.split_once('#') else {
        return false;
    };
    let mut identities = BTreeMap::<String, String>::new();
    let sites = model
        .realizes
        .iter()
        .filter(|site| site.spec == spec && site.claim == claim)
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

fn simple(
    kind: FindingKind,
    path: &str,
    line: usize,
    claim: Option<String>,
    detail: String,
) -> Finding {
    Finding {
        kind,
        severity: Severity::Error,
        claim,
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
                claim: Some(design.spec.clone()),
                criticality: None,
                path: design.path.clone(),
                line: 1,
                detail: format!("designs spec `{}`, which does not exist", design.spec),
            });
            continue;
        };
        for entry in &design.entries {
            let id = entry.target.id();
            let claim = spec.claims.iter().find(|claim| claim.id == id);
            if claim.is_none() {
                findings.push(Finding {
                    kind: FindingKind::DanglingDesignEntry,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", design.spec, id)),
                    criticality: None,
                    path: design.path.clone(),
                    line: entry.line,
                    detail: "names a Claim that does not exist".into(),
                });
            }

            for mechanism in &entry.mechanisms {
                if let Some(claim) = claim {
                    for case in &mechanism.cases {
                        if !claim.cases.iter().any(|candidate| candidate.id == *case) {
                            findings.push(Finding {
                                kind: FindingKind::DanglingDesignEntry,
                                severity: Severity::Error,
                                claim: Some(format!("{}#{}", design.spec, id)),
                                criticality: claim.criticality,
                                path: design.path.clone(),
                                line: mechanism.line,
                                detail: format!(
                                    "mechanism `{}` names Case `{case}` that does not exist under Claim `{id}`",
                                    mechanism.id
                                ),
                            });
                        }
                    }
                }
                let bindings = model.mechanism_bindings(&design.spec, mechanism);
                if bindings.len() != 1 {
                    findings.push(Finding {
                        kind: FindingKind::UnresolvedDesignBinding,
                        severity: Severity::Error,
                        claim: Some(format!("{}#{}", design.spec, id)),
                        criticality: None,
                        path: design.path.clone(),
                        line: mechanism.line,
                        detail: if bindings.is_empty() {
                            format!(
                                "mechanism `{}` has neither an explicit artifact binding nor one extractor-resolved implementation",
                                mechanism.id
                            )
                        } else {
                            format!(
                                "mechanism `{}` resolves to {} bindings; declare one atomic mechanism per implementation site",
                                mechanism.id,
                                bindings.len()
                            )
                        },
                    });
                    continue;
                }
                let binding = bindings[0];
                let Some(artifact) = model.artifacts.iter().find(|a| a.id == binding) else {
                    findings.push(Finding {
                        kind: FindingKind::UnresolvedDesignBinding,
                        severity: Severity::Error,
                        claim: Some(format!("{}#{}", design.spec, id)),
                        criticality: None,
                        path: design.path.clone(),
                        line: mechanism.line,
                        detail: format!(
                            "binding `{binding}` was not emitted by any compiler or schema extractor"
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
                        claim: Some(format!("{}#{}", design.spec, id)),
                        criticality: None,
                        path: design.path.clone(),
                        line: mechanism.line,
                        detail: format!("binding `{}`: {}", binding, mismatches.join("; ")),
                    });
                }
            }
        }
    }

    // A design entry is required for `critical`, optional for `standard`, absent for
    // `routine`. Nothing here says the mechanism is missing from the code — only that its
    // strategy
    // is undeclared, and therefore that validation cannot compare the claim against reality.
    //
    // Gated on the artifact being in use at all. Each mechanism must be usable alone
    // — `validate` without the design artifact — and a project that has not adopted it must not
    // be told that every critical Claim is a finding. Partial adoption still reports: one
    // design file means the artifact is in use, and the specs it omits are visible.
    for spec in &model.specs {
        if model.designs.is_empty() {
            break;
        }
        let design = model.design_for(&spec.id);
        for claim in &spec.claims {
            if claim.criticality != Some(Criticality::Critical) {
                continue;
            }
            let declared = design.is_some_and(|d| d.for_claim(&claim.id).is_some());
            if !declared {
                findings.push(Finding {
                    kind: FindingKind::UndeclaredMechanism,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", spec.id, claim.id)),
                    criticality: claim.criticality,
                    path: spec.path.clone(),
                    line: claim.line,
                    detail: "critical Claim declares no enforcement mechanism".into(),
                });
            }
        }
    }

    findings
}

/// Claims whose domain is a set of sites.
///
/// Membership has two sources, and the second exists because the first is not enough.
///
/// A site joins by realizing any claim in the named spec — but that only reaches sites somebody
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
/// Discharge is a `realizes` tag naming the invariant. No new tag: one claim type parameterized by
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
        for claim in &spec.claims {
            if claim.domain != crate::model::Domain::Sites {
                continue;
            }
            let claim_id = format!("{}#{}", spec.id, claim.id);
            let Some(over) = &claim.over else {
                findings.push(Finding {
                    kind: FindingKind::MissingSurface,
                    severity: severity_for(claim.criticality),
                    claim: Some(claim_id),
                    criticality: claim.criticality,
                    path: spec.path.clone(),
                    line: claim.line,
                    detail: "site-domain claim declares no `Over:` surface".into(),
                });
                continue;
            };
            let Some(surface) = model.workspace.surface(over) else {
                findings.push(Finding {
                    kind: FindingKind::UnknownSurface,
                    severity: Severity::Error,
                    claim: Some(claim_id),
                    criticality: claim.criticality,
                    path: spec.path.clone(),
                    line: claim.line,
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
                    severity: severity_for(claim.criticality),
                    claim: Some(claim_id),
                    criticality: claim.criticality,
                    path: spec.path.clone(),
                    line: claim.line,
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
                .flat_map(|class_spec| &class_spec.claims)
                .filter(|claim| claim.domain == crate::model::Domain::Behaviour)
                .map(|claim| claim.id.as_str())
                .collect();

            // (site, file, by_file): `by_file` marks a member the extractor enumerated, whose
            // identity is the file rather than a named site inside it.
            let mut members: Vec<(&str, &str, bool)> = model
                .realizes
                .iter()
                .filter(|site| {
                    site.spec == surface.id && behavioural.contains(&site.claim.as_str())
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
                .filter(|site| site.spec == spec.id && site.claim == claim.id)
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
                    severity: severity_for(claim.criticality),
                    claim: Some(format!("{}#{}", spec.id, claim.id)),
                    criticality: claim.criticality,
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
    for obligation in &model.workspace.realization_obligations {
        let Some(claim) = model.find_claim(&obligation.spec, &obligation.claim) else {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationObligation,
                severity: Severity::Error,
                claim: Some(format!("{}#{}", obligation.spec, obligation.claim)),
                criticality: None,
                path: model.workspace.path.clone(),
                line: 0,
                detail: "realization obligation names a claim that does not exist".into(),
            });
            continue;
        };
        if claim.claim.domain != crate::model::Domain::Behaviour
            || !matches!(
                claim.claim.criticality,
                Some(Criticality::Standard | Criticality::Critical)
            )
        {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationObligation,
                severity: Severity::Error,
                claim: Some(claim.id()),
                criticality: claim.claim.criticality,
                path: model.workspace.path.clone(),
                line: 0,
                detail: "area obligations apply only to standard or critical behavioral claims"
                    .into(),
            });
            continue;
        }

        for area in &obligation.areas {
            let realized = model.realizes.iter().any(|site| {
                site.spec == obligation.spec
                    && site.claim == obligation.claim
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
                    severity: severity_for(claim.claim.criticality),
                    claim: Some(claim.id()),
                    criticality: claim.claim.criticality,
                    path: model.workspace.path.clone(),
                    line: 0,
                    detail: format!("required area `{area}` has no realization of this claim"),
                });
            }
        }
    }
    findings
}

pub struct Summary {
    pub claims: usize,
    pub errors: usize,
    pub warnings: usize,
}

pub fn summarize(model: &Model, findings: &[Finding]) -> Summary {
    Summary {
        claims: model.claim_count(),
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
