//! Deterministic validation and Challenge Plan resolution over the alpha 2 model.

use crate::design::Target;
use crate::json::Json;
use crate::model::{Criticality, Model};
use crate::verification::{ChallengeDomain, ChallengePlan, QualificationVerdict, Selector};
use std::collections::BTreeSet;

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
    UnboundClaim,
    CheckWithoutBinding,
    BindingMissingCheck,
    BindingMissingClaim,
    BindingMissingPolicy,
    MissingQualification,
    DanglingQualification,
    RejectedQualification,
    StaleQualification,
    UnimplementedCheck,
    DanglingCheckImplementation,
    UnstableCheckImplementation,
    InapplicableVerification,
    MissingChallenger,
    UnresolvedChallengePlan,
    UnresolvedChallengeSelector,
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
            Self::UnboundClaim => "unbound-claim",
            Self::CheckWithoutBinding => "check-without-binding",
            Self::BindingMissingCheck => "binding-missing-check",
            Self::BindingMissingClaim => "binding-missing-claim",
            Self::BindingMissingPolicy => "binding-missing-policy",
            Self::MissingQualification => "missing-qualification",
            Self::DanglingQualification => "dangling-qualification",
            Self::RejectedQualification => "rejected-qualification",
            Self::StaleQualification => "stale-qualification",
            Self::UnimplementedCheck => "unimplemented-check",
            Self::DanglingCheckImplementation => "dangling-check-implementation",
            Self::UnstableCheckImplementation => "unstable-check-implementation",
            Self::InapplicableVerification => "inapplicable-verification",
            Self::MissingChallenger => "missing-challenger",
            Self::UnresolvedChallengePlan => "unresolved-challenge-plan",
            Self::UnresolvedChallengeSelector => "unresolved-challenge-selector",
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
            Self::MissingQualification
            | Self::DanglingQualification
            | Self::RejectedQualification
            | Self::StaleQualification => FindingCategory::Judgment,
            Self::MissingChallenger
            | Self::UnresolvedChallengePlan
            | Self::UnresolvedChallengeSelector => FindingCategory::Verification,
            _ => FindingCategory::Verification,
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Self::Unclassified => "Declare the requirement's criticality explicitly.",
            Self::Unrealized => "Link production code that establishes the Claim predicate.",
            Self::DanglingRealization => {
                "Retarget or remove the production link to the unknown Claim."
            }
            Self::DanglingDesignEntry => "Retarget or remove the design entry.",
            Self::UndeclaredMechanism => "Declare how the critical requirement is enforced.",
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
            Self::UnboundClaim => "Bind at least one deliberately enrolled Check to the Claim.",
            Self::CheckWithoutBinding => "Add an Evidence Binding or remove the unused Check.",
            Self::BindingMissingCheck => "Retarget the binding to a declared Check.",
            Self::BindingMissingClaim => "Retarget the binding to a current case-level Claim.",
            Self::BindingMissingPolicy => {
                "Retarget the binding to a declared Qualification policy."
            }
            Self::MissingQualification => "Record the binding's reviewed Qualification.",
            Self::DanglingQualification => "Retarget or remove the Qualification.",
            Self::RejectedQualification => "Resolve the objection before qualifying the binding.",
            Self::StaleQualification => "Re-qualify the binding against its current fingerprint.",
            Self::UnimplementedCheck => {
                "Mark at least one stable source implementation of the Check."
            }
            Self::DanglingCheckImplementation => "Retarget the source marker to a declared Check.",
            Self::UnstableCheckImplementation => {
                "Resolve semantic source identity and fingerprint."
            }
            Self::InapplicableVerification => "Remove verification from the routine Claim.",
            Self::MissingChallenger => "Retarget the Challenge Plan to a declared Challenger.",
            Self::UnresolvedChallengePlan => "Select at least one current Qualification.",
            Self::UnresolvedChallengeSelector => {
                "Retarget the semantic selector to a current decision."
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QualificationTarget {
    pub fingerprint: String,
    pub binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeResolution {
    pub plan: String,
    pub challenger: String,
    pub qualifications: Vec<QualificationTarget>,
    pub unresolved_selectors: Vec<String>,
}

pub fn resolve_challenge_plan(model: &Model, plan: &ChallengePlan) -> ChallengeResolution {
    let mut qualifications = BTreeSet::new();
    let mut unresolved_selectors = Vec::new();
    for selector in &plan.selectors {
        let targets = resolve_selector(model, selector);
        if targets.is_empty() {
            unresolved_selectors.push(selector.canonical());
        }
        qualifications.extend(targets);
    }
    unresolved_selectors.sort();
    unresolved_selectors.dedup();
    ChallengeResolution {
        plan: plan.id.clone(),
        challenger: plan.challenger.clone(),
        qualifications: qualifications.into_iter().collect(),
        unresolved_selectors,
    }
}

fn resolve_selector(model: &Model, selector: &Selector) -> Vec<QualificationTarget> {
    let mut bindings = BTreeSet::<String>::new();
    match selector {
        Selector::QualificationFromBinding(id) => {
            bindings.insert(id.clone());
        }
        Selector::QualificationFromCheck(id) => {
            bindings.extend(
                model
                    .evidence_bindings()
                    .filter(|binding| binding.check == *id)
                    .map(|binding| binding.id.clone()),
            );
        }
        Selector::QualificationFromRealization(identity) => {
            let claims = model
                .realizes
                .iter()
                .filter(|site| {
                    site.source
                        .as_ref()
                        .is_some_and(|source| source.key() == *identity)
                })
                .map(|site| format!("{}#{}", site.spec, site.scenario))
                .collect::<BTreeSet<_>>();
            bindings.extend(
                model
                    .evidence_bindings()
                    .filter(|binding| {
                        claims.contains(&binding.claim)
                            && binding
                                .challenge_domain
                                .contains(&ChallengeDomain::Realization)
                    })
                    .map(|binding| binding.id.clone()),
            );
        }
        Selector::QualificationFromMechanism(identity) => {
            let claims = claims_for_mechanism(model, identity);
            bindings.extend(
                model
                    .evidence_bindings()
                    .filter(|binding| {
                        claims.contains(&binding.claim)
                            && binding
                                .challenge_domain
                                .contains(&ChallengeDomain::Mechanism)
                    })
                    .map(|binding| binding.id.clone()),
            );
        }
        Selector::ClaimJudgmentFromClaim(_)
        | Selector::ClaimJudgmentFromRealization(_)
        | Selector::ClaimJudgmentFromMechanism(_) => {}
    }
    bindings
        .into_iter()
        .filter_map(|id| current_qualification_target(model, &id))
        .collect()
}

fn claims_for_mechanism(model: &Model, identity: &str) -> BTreeSet<String> {
    let Some((spec_id, mechanism_id)) = identity.split_once('#') else {
        return BTreeSet::new();
    };
    let Some(design) = model.design_for(spec_id) else {
        return BTreeSet::new();
    };
    model
        .claims()
        .filter(|claim| claim.spec.id == spec_id)
        .filter(|claim| {
            design
                .for_scenario(&claim.scenario.id)
                .into_iter()
                .chain(design.for_requirement(&claim.requirement.id))
                .any(|entry| {
                    entry
                        .mechanisms
                        .iter()
                        .any(|mechanism| mechanism.id == mechanism_id)
                })
        })
        .map(|claim| claim.id())
        .collect()
}

fn current_qualification_target(model: &Model, binding_id: &str) -> Option<QualificationTarget> {
    let binding = model
        .evidence_bindings()
        .find(|binding| binding.id == binding_id)?;
    let claim = model.claims().find(|claim| claim.id() == binding.claim)?;
    if !matches!(
        claim.requirement.criticality,
        Some(Criticality::Standard | Criticality::Critical)
    ) {
        return None;
    }
    let qualification = model
        .qualifications()
        .find(|qualification| qualification.id == binding_id)?;
    let expected = model.expected_qualification_fingerprint(binding)?;
    (qualification.fingerprint == expected).then(|| QualificationTarget {
        binding: binding.id.clone(),
        fingerprint: expected,
    })
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
                    claim: Some(format!("{}#{}", spec.id, requirement.id)),
                    criticality: None,
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: format!("requirement `{}` declares no criticality", requirement.id),
                });
            }
        }
    }
    for claim in model.claims() {
        if claim.requirement.criticality == Some(Criticality::Routine) {
            continue;
        }
        if !model
            .realizes
            .iter()
            .any(|site| site.spec == claim.spec.id && site.scenario == claim.scenario.id)
        {
            findings.push(Finding {
                kind: FindingKind::Unrealized,
                severity: severity_for(claim.requirement.criticality),
                claim: Some(claim.id()),
                criticality: claim.requirement.criticality,
                path: claim.spec.path.clone(),
                line: claim.scenario.line,
                detail: "no production code realizes this Claim".into(),
            });
        }
    }
    for site in &model.realizes {
        if !model.has_claim(&site.spec, &site.scenario) {
            findings.push(Finding {
                kind: FindingKind::DanglingRealization,
                severity: Severity::Error,
                claim: Some(format!("{}#{}", site.spec, site.scenario)),
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
        .qualification_policies
        .as_ref()
        .map(|standards| &standards.policies[..])
        .unwrap_or(&[]);
    for claim in model.claims() {
        let bindings = model
            .evidence_bindings()
            .filter(|binding| binding.claim == claim.id())
            .collect::<Vec<_>>();
        if claim.requirement.criticality == Some(Criticality::Routine) {
            for binding in bindings {
                findings.push(Finding {
                    kind: FindingKind::InapplicableVerification,
                    severity: Severity::Warning,
                    claim: Some(claim.id()),
                    criticality: claim.requirement.criticality,
                    path: binding.path.clone(),
                    line: binding.line,
                    detail: format!("Evidence Binding `{}` targets a routine Claim", binding.id),
                });
            }
        } else if claim.requirement.criticality.is_some() && bindings.is_empty() {
            findings.push(Finding {
                kind: FindingKind::UnboundClaim,
                severity: severity_for(claim.requirement.criticality),
                claim: Some(claim.id()),
                criticality: claim.requirement.criticality,
                path: claim.spec.path.clone(),
                line: claim.scenario.line,
                detail: "non-routine Claim has no Evidence Binding".into(),
            });
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
                    .claims()
                    .find(|claim| claim.id() == binding.claim)
                    .is_some_and(|claim| {
                        matches!(
                            claim.requirement.criticality,
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
                    .claims()
                    .find(|claim| claim.id() == binding.claim)
                    .is_some_and(|claim| {
                        matches!(
                            claim.requirement.criticality,
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
        let claim = model.claims().find(|claim| claim.id() == binding.claim);
        if claim
            .as_ref()
            .is_some_and(|claim| claim.requirement.criticality == Some(Criticality::Routine))
        {
            continue;
        }
        if !model.checks().any(|check| check.id == binding.check) {
            findings.push(simple(
                FindingKind::BindingMissingCheck,
                &binding.path,
                binding.line,
                Some(binding.claim.clone()),
                format!(
                    "binding `{}` names unknown Check `{}`",
                    binding.id, binding.check
                ),
            ));
        }
        if claim.is_none() {
            findings.push(simple(
                FindingKind::BindingMissingClaim,
                &binding.path,
                binding.line,
                Some(binding.claim.clone()),
                format!("binding `{}` names no current case-level Claim", binding.id),
            ));
        }
        if !policies
            .iter()
            .any(|policy| policy.id == binding.qualification_policy)
        {
            findings.push(simple(
                FindingKind::BindingMissingPolicy,
                &binding.path,
                binding.line,
                Some(binding.claim.clone()),
                format!(
                    "binding `{}` names unknown policy `{}`",
                    binding.id, binding.qualification_policy
                ),
            ));
        }
        let qualification = model
            .qualifications()
            .find(|qualification| qualification.id == binding.id);
        let Some(qualification) = qualification else {
            findings.push(simple(
                FindingKind::MissingQualification,
                &binding.path,
                binding.line,
                Some(binding.claim.clone()),
                format!("binding `{}` has no Qualification", binding.id),
            ));
            continue;
        };
        if qualification.verdict == QualificationVerdict::Rejected {
            findings.push(simple(
                FindingKind::RejectedQualification,
                &qualification.path,
                qualification.line,
                Some(binding.claim.clone()),
                format!("Qualification `{}` is rejected", qualification.id),
            ));
        }
        if let Some(expected) = model.expected_qualification_fingerprint(binding) {
            if qualification.fingerprint != expected {
                findings.push(simple(
                    FindingKind::StaleQualification,
                    &qualification.path,
                    qualification.line,
                    Some(binding.claim.clone()),
                    format!(
                        "Qualification `{}` expected {}, found {}",
                        qualification.id, expected, qualification.fingerprint
                    ),
                ));
            }
        }
    }
    for qualification in model.qualifications() {
        if !model
            .evidence_bindings()
            .any(|binding| binding.id == qualification.id)
        {
            findings.push(simple(
                FindingKind::DanglingQualification,
                &qualification.path,
                qualification.line,
                None,
                format!(
                    "Qualification `{}` names no Evidence Binding",
                    qualification.id
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
        for selector in &resolution.unresolved_selectors {
            findings.push(simple(
                FindingKind::UnresolvedChallengeSelector,
                &plan.path,
                plan.line,
                None,
                format!(
                    "Challenge Plan `{}` selector `{}` resolves zero decisions",
                    plan.id, selector
                ),
            ));
        }
        if resolution.qualifications.is_empty() && !resolution.unresolved_selectors.is_empty() {
            findings.push(simple(
                FindingKind::UnresolvedChallengePlan,
                &plan.path,
                plan.line,
                None,
                format!(
                    "Challenge Plan `{}` resolves no current Qualification",
                    plan.id
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
            let exists = match &entry.target {
                Target::Requirement(_) => spec.requirements.iter().any(|r| r.id == id),
                Target::Scenario(_) => spec
                    .requirements
                    .iter()
                    .any(|r| r.scenarios.iter().any(|s| s.id == id)),
            };
            if !exists {
                findings.push(Finding {
                    kind: FindingKind::DanglingDesignEntry,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", design.spec, id)),
                    criticality: None,
                    path: design.path.clone(),
                    line: entry.line,
                    detail: "names a requirement or claim that does not exist".into(),
                });
            }

            for mechanism in &entry.mechanisms {
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

    // D6.5: a design entry is required for `critical`, optional for `standard`, absent for
    // `routine`. Nothing here says the mechanism is missing from the code — only that its
    // strategy
    // is undeclared, and therefore that validation cannot compare the claim against reality.
    //
    // Gated on the artifact being in use at all. D8.1 requires each mechanism to be usable alone
    // — `validate` without the design artifact — and a project that has not adopted it must not
    // be told that every critical requirement is a finding. Partial adoption still reports: one
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
            let declared = design.is_some_and(|d| {
                d.for_requirement(&requirement.id).is_some()
                    || requirement
                        .scenarios
                        .iter()
                        .any(|s| d.for_scenario(&s.id).is_some())
            });
            if !declared {
                findings.push(Finding {
                    kind: FindingKind::UndeclaredMechanism,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", spec.id, requirement.id)),
                    criticality: requirement.criticality,
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: "critical requirement declares no enforcement mechanism".into(),
                });
            }
        }
    }

    findings
}

/// Claims whose domain is a set of sites (D13).
///
/// Membership has two sources, and the second exists because the first is not enough.
///
/// A site joins by realizing any claim in the named spec — but that only reaches sites somebody
/// already tagged, so a file carrying no tags at all can never be a member. That is the failure
/// D13.1 names: an enumerator drawn from annotations reproduces the very bug the rule prevents and
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
/// breaches, which is exactly the defect D7 names in the alpha. Crediting a choke point needs
/// call-graph analysis, which belongs to the extractor rather than derived-model validation.
fn surface_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();

    for spec in &model.specs {
        for requirement in &spec.requirements {
            if requirement.domain != crate::model::Domain::Sites {
                continue;
            }
            let claim_id = format!("{}#{}", spec.id, requirement.id);
            let Some(over) = &requirement.over else {
                findings.push(Finding {
                    kind: FindingKind::MissingSurface,
                    severity: severity_for(requirement.criticality),
                    claim: Some(claim_id),
                    criticality: requirement.criticality,
                    path: spec.path.clone(),
                    line: requirement.line,
                    detail: "site-domain claim declares no `Over:` surface".into(),
                });
                continue;
            };
            let Some(surface) = model.workspace.surface(over) else {
                findings.push(Finding {
                    kind: FindingKind::UnknownSurface,
                    severity: Severity::Error,
                    claim: Some(claim_id),
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
                    claim: Some(claim_id),
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
                .filter(|r| r.domain == crate::model::Domain::Behaviour)
                .flat_map(|r| r.scenarios.iter().map(|s| s.id.as_str()))
                .collect();

            // (site, file, by_file): `by_file` marks a member the extractor enumerated, whose
            // identity is the file rather than a named site inside it.
            let mut members: Vec<(&str, &str, bool)> = model
                .realizes
                .iter()
                .filter(|site| {
                    site.spec == surface.id && behavioural.contains(&site.scenario.as_str())
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
                .filter(|site| site.spec == spec.id && site.scenario == requirement.id)
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
                    claim: Some(format!("{}#{}", spec.id, requirement.id)),
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
        if claim.requirement.domain != crate::model::Domain::Behaviour
            || !matches!(
                claim.requirement.criticality,
                Some(Criticality::Standard | Criticality::Critical)
            )
        {
            findings.push(Finding {
                kind: FindingKind::DanglingRealizationObligation,
                severity: Severity::Error,
                claim: Some(claim.id()),
                criticality: claim.requirement.criticality,
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
                    && site.scenario == obligation.claim
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
                    severity: severity_for(claim.requirement.criticality),
                    claim: Some(claim.id()),
                    criticality: claim.requirement.criticality,
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
        claims: model.scenario_count(),
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
