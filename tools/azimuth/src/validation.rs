//! Deterministic validation over the derived model.
//!
//! Validation reports structural Findings across every model facet (D44). Findings are distinct
//! from enrolled Checks: validation interprets the derived model and never executes a verification
//! method.

use crate::design::Target;
use crate::json::Json;
use crate::model::{
    Criticality, Model, ObservationRole, ObservationSubjectRelation, Quantification, Required,
    Scope, Site, Strength,
};
use crate::plan::EvidenceItem;
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
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
    ($( $(#[$meta:meta])* $variant:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum FindingKind {
            $(
                $(#[$meta])*
                $variant,
            )+
        }

        impl FindingKind {
            /// The single exhaustive registry for validation output and summaries.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

define_finding_kinds! {
    /// intent present, mechanism absent
    Unrealized,
    /// intent present, evidence absent
    Uncovered,
    /// evidence present, intent absent — the tag names a claim that does not exist
    DanglingTag,
    /// mechanism present, intent absent
    DanglingRealization,
    /// evidence present, intent absent — a plan entry for a claim that does not exist
    DanglingPlanEntry,
    /// intent and evidence present, but no evidence meets the declared standard
    WrongForm,
    /// D6.2: a requirement without a declared criticality.
    ///
    /// Not a missing-facet combination: the intent facet is *present but incomplete*. See
    /// `UnacceptedWeakening` and the note on `validate`.
    Unclassified,
    /// A plan requires less than the standard without recording and accepting the residual.
    /// Incomplete-facet, like `Unclassified`.
    UnacceptedWeakening,
    /// mechanism present, intent absent — a design entry for a requirement that does not exist
    DanglingDesignEntry,
    /// D6.5 requires a design entry for a `critical` requirement. Incomplete-facet: the mechanism
    /// may well exist in code, but its strategy is undeclared and therefore uncheckable.
    UndeclaredMechanism,
    /// A current design names an artifact no extractor found.
    UnresolvedDesignBinding,
    /// The extractor found the artifact, but its derived properties contradict the enforcement.
    EnforcementMismatch,
    /// A plan declares proof-strength evidence that no proof-capable mechanism backs.
    UnbackedProof,
    /// The agent tier judged the covering evidence as not discriminating: it exists, it passes, and
    /// it would also pass against an implementation that is wrong.
    ToothlessEvidence,
    /// The agent tier judged a tag as declaring a stronger form than the test has.
    DishonestTag,
    /// The agent tier judged a realization site as not establishing the claim predicate.
    DishonestRealization,
    /// The agent tier found a behaviour the spec should name and does not.
    SpecGap,
    /// A judgment whose fingerprint no longer matches what it looked at.
    StaleJudgment,
    /// A critical claim the agent tier has never judged. Incomplete-facet, like `Unclassified`.
    Unjudged,
    /// A site that joined a claim's class and discharges nothing.
    ///
    /// The one finding kind the per-scenario matrix structurally cannot find: a claim quantified
    /// over a *set of sites* is not established by evidence about one site, however good.
    InvariantBreach,
    /// A site-domain claim has no successful derivation witness for the class it ranges over.
    EnumeratorUnsoundOrUnderived,
    /// An imported external result is a recorded failure, so it cannot be counted as coverage.
    FailedEvidence,
    /// An imported result passed but is older than the validity window declared by its adapter.
    ExpiredEvidence,
    /// A provided non-test evidence item names a detector artifact no extractor emitted.
    UnresolvedEvidenceBinding,
    /// A detection item names a detector-test artifact no extractor emitted.
    UnresolvedDetectorBinding,
    /// A site-domain claim omitted the independently derived surface it ranges over.
    MissingSurface,
    /// A site-domain claim names no declared surface.
    UnknownSurface,
    /// A declared area contribution has no realization of the claim.
    MissingRequiredRealization,
    /// A realization obligation names no current claim or is attached to an inapplicable claim.
    DanglingRealizationObligation,
    /// An implementation tag names no design-owned mechanism.
    DanglingMechanismImplementation,
    /// Mechanism evidence names no design-owned mechanism.
    DanglingMechanismCover,
    /// Two immutable execution accounts claim the same identity.
    DuplicateObservation,
    /// A judgment challenge no longer resolves to the claim sites or mechanisms it targeted.
    UnresolvedObservationBinding,
}

impl FindingKind {
    pub fn name(self) -> &'static str {
        match self {
            FindingKind::Unrealized => "unrealized",
            FindingKind::Uncovered => "uncovered",
            FindingKind::DanglingTag => "dangling-tag",
            FindingKind::DanglingRealization => "dangling-realization",
            FindingKind::DanglingPlanEntry => "dangling-plan-entry",
            FindingKind::WrongForm => "wrong-form",
            FindingKind::Unclassified => "unclassified",
            FindingKind::UnacceptedWeakening => "unaccepted-weakening",
            FindingKind::DanglingDesignEntry => "dangling-design-entry",
            FindingKind::UndeclaredMechanism => "undeclared-mechanism",
            FindingKind::UnresolvedDesignBinding => "unresolved-design-binding",
            FindingKind::EnforcementMismatch => "enforcement-mismatch",
            FindingKind::UnbackedProof => "unbacked-proof",
            FindingKind::ToothlessEvidence => "toothless-evidence",
            FindingKind::DishonestTag => "dishonest-tag-judged",
            FindingKind::DishonestRealization => "dishonest-realization",
            FindingKind::SpecGap => "spec-gap",
            FindingKind::StaleJudgment => "stale-judgment",
            FindingKind::Unjudged => "unjudged",
            FindingKind::InvariantBreach => "invariant-breach",
            FindingKind::EnumeratorUnsoundOrUnderived => "enumerator-unsound-or-underived",
            FindingKind::FailedEvidence => "failed-evidence",
            FindingKind::ExpiredEvidence => "expired-evidence",
            FindingKind::UnresolvedEvidenceBinding => "unresolved-evidence-binding",
            FindingKind::UnresolvedDetectorBinding => "unresolved-detector-binding",
            FindingKind::MissingSurface => "missing-surface",
            FindingKind::UnknownSurface => "unknown-surface",
            FindingKind::MissingRequiredRealization => "missing-required-realization",
            FindingKind::DanglingRealizationObligation => "dangling-realization-obligation",
            FindingKind::DanglingMechanismImplementation => "dangling-mechanism-implementation",
            FindingKind::DanglingMechanismCover => "dangling-mechanism-cover",
            FindingKind::DuplicateObservation => "duplicate-observation",
            FindingKind::UnresolvedObservationBinding => "unresolved-observation-binding",
        }
    }

    pub fn category(self) -> FindingCategory {
        match self {
            Self::Unclassified => FindingCategory::Intent,
            Self::Unrealized
            | Self::DanglingRealization
            | Self::MissingRequiredRealization
            | Self::DanglingRealizationObligation => FindingCategory::Realization,
            Self::Uncovered
            | Self::DanglingTag
            | Self::DanglingPlanEntry
            | Self::WrongForm
            | Self::UnacceptedWeakening
            | Self::UnbackedProof
            | Self::FailedEvidence
            | Self::ExpiredEvidence
            | Self::UnresolvedEvidenceBinding
            | Self::UnresolvedDetectorBinding => FindingCategory::Verification,
            Self::DanglingDesignEntry
            | Self::UndeclaredMechanism
            | Self::UnresolvedDesignBinding
            | Self::EnforcementMismatch
            | Self::DanglingMechanismImplementation
            | Self::DanglingMechanismCover => FindingCategory::Mechanism,
            Self::ToothlessEvidence
            | Self::DishonestTag
            | Self::DishonestRealization
            | Self::SpecGap
            | Self::StaleJudgment
            | Self::Unjudged => FindingCategory::Judgment,
            Self::InvariantBreach
            | Self::EnumeratorUnsoundOrUnderived
            | Self::MissingSurface
            | Self::UnknownSurface => FindingCategory::Surface,
            Self::DuplicateObservation | Self::UnresolvedObservationBinding => {
                FindingCategory::Execution
            }
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Self::Unrealized => "Link production code that establishes the Claim predicate.",
            Self::Uncovered => "Provide evidence that meets the Claim's verification standard.",
            Self::DanglingTag => "Retarget or remove the evidence link to the unknown Claim.",
            Self::DanglingRealization => {
                "Retarget or remove the production link to the unknown Claim."
            }
            Self::DanglingPlanEntry => {
                "Retarget or remove the verification plan entry for the unknown Claim."
            }
            Self::WrongForm => "Provide evidence with the required strength, scope, and form.",
            Self::Unclassified => "Declare the requirement's criticality explicitly.",
            Self::UnacceptedWeakening => {
                "Meet the project standard or record and accept the residual explicitly."
            }
            Self::DanglingDesignEntry => {
                "Retarget or remove the design entry for the unknown requirement."
            }
            Self::UndeclaredMechanism => {
                "Declare how the critical requirement is enforced in the current design."
            }
            Self::UnresolvedDesignBinding => {
                "Correct the design binding so an extractor resolves its artifact."
            }
            Self::EnforcementMismatch => {
                "Align the declared enforcement with the artifact's derived properties."
            }
            Self::UnbackedProof => {
                "Bind proof-strength evidence to a mechanism that makes violation unrepresentable."
            }
            Self::ToothlessEvidence => {
                "Strengthen or replace evidence so it discriminates an incorrect implementation."
            }
            Self::DishonestTag => {
                "Correct the evidence form declaration to match what the evidence establishes."
            }
            Self::DishonestRealization => {
                "Move, narrow, or remove the realization link so it establishes the Claim."
            }
            Self::SpecGap => "Add the missing behavior to the authoritative specification.",
            Self::StaleJudgment => "Re-evaluate the Claim against its current fingerprint.",
            Self::Unjudged => "Record an agent judgment for the critical Claim.",
            Self::InvariantBreach => "Discharge the invariant for the reported surface member.",
            Self::EnumeratorUnsoundOrUnderived => {
                "Run or repair the declared enumerator before evaluating surface members."
            }
            Self::FailedEvidence => "Refresh the evidence with a successful execution result.",
            Self::ExpiredEvidence => "Refresh the evidence within its declared validity window.",
            Self::UnresolvedEvidenceBinding => {
                "Correct the evidence binding so an extractor resolves its detector artifact."
            }
            Self::UnresolvedDetectorBinding => {
                "Correct the detector-test binding so an extractor resolves its artifact."
            }
            Self::MissingSurface => {
                "Declare which independently derived surface the Claim ranges over."
            }
            Self::UnknownSurface => "Declare the referenced surface in the workspace model.",
            Self::MissingRequiredRealization => {
                "Add a realization for the required participating area."
            }
            Self::DanglingRealizationObligation => {
                "Retarget or remove the realization obligation that resolves to no applicable \
                 Claim."
            }
            Self::DanglingMechanismImplementation => {
                "Retarget or remove the implementation link to the unknown mechanism."
            }
            Self::DanglingMechanismCover => {
                "Retarget or remove the evidence link to the unknown mechanism."
            }
            Self::DuplicateObservation => {
                "Assign every imported execution observation a unique immutable identity."
            }
            Self::UnresolvedObservationBinding => {
                "Retarget the imported challenge to current realization or mechanism subjects."
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
                match &self.claim {
                    Some(c) => Json::str(c),
                    None => Json::Null,
                },
            ),
            (
                "criticality",
                match self.criticality {
                    Some(c) => Json::str(c.name()),
                    None => Json::Null,
                },
            ),
            ("file", Json::str(&self.path)),
            ("line", Json::Num(self.line as f64)),
            ("detail", Json::str(&self.detail)),
            ("help", Json::str(self.kind.help())),
        ])
    }
}

/// D9.2: severity comes from criticality, not from the validation rule. One non-zero exit for
/// "something failed" stops being useful when validation has many rules.
///
/// `routine` warns rather than fails, because D6.5 gives it a spec entry and nothing else — it is
/// the tier D9.2 means by low-criticality. `standard` fails: it requires a verification plan, so
/// its findings are real.
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
        let id = claim.id();
        let criticality = claim.requirement.criticality;

        // D20: a routine claim stops at intent. It makes no assertion about where production code
        // realizes it or which tests cover it, so neither linkage facet can have a finding.
        if criticality == Some(Criticality::Routine) {
            continue;
        }

        let severity = severity_for(criticality);

        let realized = model
            .realizes
            .iter()
            .any(|s| s.spec == claim.spec.id && s.scenario == claim.scenario.id);
        if !realized {
            findings.push(Finding {
                kind: FindingKind::Unrealized,
                severity,
                claim: Some(id.clone()),
                criticality,
                path: claim.spec.path.clone(),
                line: claim.scenario.line,
                detail: "no production code realizes this claim".into(),
            });
        }

        let required = model.required_for(&claim);
        let evidence_required = match required {
            Some(r) => r.strength.is_some(),
            None => criticality.map(|c| c.requires_evidence()).unwrap_or(true),
        };
        if !evidence_required {
            continue;
        }

        let tags: Vec<&Site> = model
            .covers
            .iter()
            .filter(|s| s.spec == claim.spec.id && s.scenario == claim.scenario.id)
            .filter(|site| receipt_is_usable(site, current_unix_seconds()))
            .collect();

        // Non-test evidence declared in the plan. The machine cannot verify a manual pass or an
        // attestation; that is the agent tier's job (D14). What it can do is refuse to let the
        // item stand in for a stronger requirement than it claims.
        let declared = model
            .plan_for(&claim.spec.id)
            .and_then(|p| p.entry(&claim.scenario.id))
            .and_then(|e| e.evidence.as_ref());

        if tags.is_empty() && declared.is_none() {
            findings.push(Finding {
                kind: FindingKind::Uncovered,
                severity,
                claim: Some(id),
                criticality,
                path: claim.spec.path.clone(),
                line: claim.scenario.line,
                detail: "no evidence covers this claim".into(),
            });
            continue;
        }

        let Some(required) = required else { continue };
        let Some(min_strength) = required.strength else {
            continue;
        };

        // D7's identity: strong enforcement is self-evidencing, so proof-strength evidence
        // satisfies a demonstration requirement without any test. The old model penalized exactly
        // this design.
        let satisfied_by_declaration = declared.is_some_and(|e| e.strength >= min_strength);
        let satisfied_by_test = tags.iter().any(|t| satisfies(t, &required));

        if !satisfied_by_declaration && !satisfied_by_test {
            findings.push(Finding {
                kind: FindingKind::WrongForm,
                severity,
                claim: Some(id),
                criticality,
                path: claim.spec.path.clone(),
                line: claim.scenario.line,
                detail: format!(
                    "requires {} at {} scope, {} quantification; found {}",
                    min_strength.name(),
                    required.scope.name(),
                    required.quantification.map(|q| q.name()).unwrap_or("any"),
                    describe_evidence(&tags, declared)
                ),
            });
        }
    }

    findings.extend(plan_findings(model));
    findings.extend(design_findings(model));
    findings.extend(judgment_findings(model));
    findings.extend(realization_obligation_findings(model));
    findings.extend(surface_findings(model));
    findings.extend(receipt_findings_at(model, current_unix_seconds()));

    for (sites, kind) in [
        (&model.covers, FindingKind::DanglingTag),
        (&model.realizes, FindingKind::DanglingRealization),
    ] {
        for site in sites {
            if !model.has_claim(&site.spec, &site.scenario) {
                findings.push(Finding {
                    kind,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", site.spec, site.scenario)),
                    criticality: None,
                    path: site.file.clone(),
                    line: 0,
                    detail: format!("`{}` names a claim that does not exist", site.site),
                });
            }
        }
    }

    for implementation in &model.mechanism_implementations {
        if !has_mechanism(model, &implementation.spec, &implementation.mechanism) {
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
                detail: format!(
                    "`{}` implements a mechanism the design does not declare",
                    implementation.binding
                ),
            });
        }
    }
    for cover in &model.mechanism_covers {
        if !has_mechanism(model, &cover.spec, &cover.mechanism) {
            findings.push(Finding {
                kind: FindingKind::DanglingMechanismCover,
                severity: Severity::Error,
                claim: Some(format!("{}#{}", cover.spec, cover.mechanism)),
                criticality: None,
                path: cover.file.clone(),
                line: 0,
                detail: format!(
                    "`{}` covers a mechanism the design does not declare",
                    cover.site
                ),
            });
        }
    }

    let mut observation_ids = BTreeSet::new();
    for observation in &model.observations {
        if !observation_ids.insert(&observation.id) {
            findings.push(Finding {
                kind: FindingKind::DuplicateObservation,
                severity: Severity::Error,
                claim: None,
                criticality: None,
                path: observation.report.clone(),
                line: 0,
                detail: format!("observation id `{}` is not unique", observation.id),
            });
        }
        for binding in observation
            .bindings
            .iter()
            .filter(|binding| binding.role == ObservationRole::Challenge)
        {
            let unresolved = binding
                .subjects
                .iter()
                .filter(|subject| !observation_subject_resolves(model, binding, subject))
                .map(|subject| format!("{}:{}", subject.relation.name(), subject.identity))
                .collect::<Vec<_>>();
            if !model.has_claim(&binding.spec, &binding.scenario) || !unresolved.is_empty() {
                findings.push(Finding {
                    kind: FindingKind::UnresolvedObservationBinding,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", binding.spec, binding.scenario)),
                    criticality: None,
                    path: observation.report.clone(),
                    line: 0,
                    detail: format!(
                        "challenge `{}` does not resolve to its claim account; missing subjects [{}]",
                        observation.id,
                        unresolved.join(", ")
                    ),
                });
            }
        }
    }

    findings.sort_by(|a, b| {
        (a.path.clone(), a.line, a.kind.name()).cmp(&(b.path.clone(), b.line, b.kind.name()))
    });
    findings
}

fn observation_subject_resolves(
    model: &Model,
    binding: &crate::model::ObservationBinding,
    subject: &crate::model::ObservationSubject,
) -> bool {
    match subject.relation {
        ObservationSubjectRelation::Realization => model.realizes.iter().any(|site| {
            site.spec == binding.spec
                && site.scenario == binding.scenario
                && site
                    .subject_identities()
                    .iter()
                    .any(|identity| !identity.is_empty() && identity == &subject.identity)
        }),
        ObservationSubjectRelation::Evidence => model.covers.iter().any(|site| {
            site.spec == binding.spec
                && site.scenario == binding.scenario
                && site
                    .subject_identities()
                    .iter()
                    .any(|identity| !identity.is_empty() && identity == &subject.identity)
        }),
        ObservationSubjectRelation::Mechanism => {
            model
                .mechanism_implementations
                .iter()
                .any(|implementation| {
                    implementation.spec == binding.spec
                        && (subject.identity
                            == format!("{}#{}", implementation.spec, implementation.mechanism)
                            || implementation
                                .source
                                .as_ref()
                                .is_some_and(|source| source.key() == subject.identity)
                            || subject.identity
                                == format!(
                                    "{}#{}|{}",
                                    implementation.file,
                                    implementation.binding,
                                    implementation.lang
                                ))
                })
        }
    }
}

fn has_mechanism(model: &Model, spec: &str, id: &str) -> bool {
    model.design_for(spec).is_some_and(|design| {
        design
            .entries
            .iter()
            .any(|entry| entry.mechanisms.iter().any(|mechanism| mechanism.id == id))
    })
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn receipt_is_usable(site: &Site, now: u64) -> bool {
    site.evidence_kind.is_none()
        || (site.evidence_outcome.as_deref() == Some("passed")
            && site.expires_at.is_some_and(|expiry| expiry > now))
}

pub fn receipt_findings_at(model: &Model, now: u64) -> Vec<Finding> {
    model
        .covers
        .iter()
        .filter(|site| site.evidence_kind.is_some())
        .filter_map(|site| {
            let claim = model.find_claim(&site.spec, &site.scenario);
            let criticality = claim.and_then(|claim| claim.requirement.criticality);
            let base = || Finding {
                kind: FindingKind::FailedEvidence,
                severity: severity_for(criticality),
                claim: Some(format!("{}#{}", site.spec, site.scenario)),
                criticality,
                path: site.file.clone(),
                line: 0,
                detail: String::new(),
            };
            if site.evidence_outcome.as_deref() == Some("failed") {
                let mut finding = base();
                finding.detail = format!(
                    "external manual result `{}` failed at {}",
                    site.site,
                    site.observed_at.as_deref().unwrap_or("an unknown instant")
                );
                Some(finding)
            } else if site.expires_at.is_some_and(|expiry| expiry <= now) {
                let mut finding = base();
                finding.kind = FindingKind::ExpiredEvidence;
                finding.detail = format!(
                    "external manual result `{}` expired at Unix second {}",
                    site.site,
                    site.expires_at.unwrap_or_default()
                );
                Some(finding)
            } else {
                None
            }
        })
        .collect()
}

/// A tag declares what the test *actually* is. An emitter that omits a form is read at the weakest
/// rung rather than the strongest — an unstated claim should never satisfy a requirement.
fn satisfies(tag: &Site, required: &Required) -> bool {
    let scope = tag.scope.unwrap_or(Scope::Unit);
    let quantification = tag.quantification.unwrap_or(Quantification::Example);
    Strength::Demonstration >= required.strength.unwrap_or(Strength::Detection)
        && scope >= required.scope
        && required.quantification.is_none_or(|q| quantification >= q)
}

fn describe_evidence(tags: &[&Site], declared: Option<&EvidenceItem>) -> String {
    let mut parts: Vec<String> = tags
        .iter()
        .map(|t| {
            format!(
                "{} ({}/{})",
                t.site,
                t.scope.map(|s| s.name()).unwrap_or("unit, undeclared"),
                t.quantification
                    .map(|q| q.name())
                    .unwrap_or("example, undeclared")
            )
        })
        .collect();
    if let Some(e) = declared {
        parts.push(format!("declared {} evidence", e.strength.name()));
    }
    if parts.is_empty() {
        "nothing".into()
    } else {
        parts.join(", ")
    }
}

/// Findings about the plan itself rather than about a claim's facets.
fn plan_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    for plan in &model.plans {
        let spec_exists = model.specs.iter().any(|s| s.id == plan.spec);
        if !spec_exists {
            findings.push(Finding {
                kind: FindingKind::DanglingPlanEntry,
                severity: Severity::Error,
                claim: Some(plan.spec.clone()),
                criticality: None,
                path: plan.path.clone(),
                line: 1,
                detail: format!("plans spec `{}`, which does not exist", plan.spec),
            });
            continue;
        }

        for entry in &plan.entries {
            let Some(claim) = model.find_claim(&plan.spec, &entry.scenario) else {
                findings.push(Finding {
                    kind: FindingKind::DanglingPlanEntry,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", plan.spec, entry.scenario)),
                    criticality: None,
                    path: plan.path.clone(),
                    line: entry.line,
                    detail: "names a claim that does not exist".into(),
                });
                continue;
            };

            if let Some(evidence) = &entry.evidence {
                for (bindings, kind, role) in [
                    (
                        &evidence.bindings,
                        FindingKind::UnresolvedEvidenceBinding,
                        "evidence",
                    ),
                    (
                        &evidence.detector_bindings,
                        FindingKind::UnresolvedDetectorBinding,
                        "detector test",
                    ),
                ] {
                    for binding in bindings {
                        if !model
                            .artifacts
                            .iter()
                            .any(|artifact| artifact.id == *binding)
                        {
                            findings.push(Finding {
                                kind,
                                severity: severity_for(claim.requirement.criticality),
                                claim: Some(format!("{}#{}", plan.spec, entry.scenario)),
                                criticality: claim.requirement.criticality,
                                path: plan.path.clone(),
                                line: entry.line,
                                detail: format!(
                                    "{role} binding `{binding}` was not emitted by any extractor"
                                ),
                            });
                        }
                    }
                }
            }

            // "Silent weakening is not available." A plan may require less than the standard, but
            // only with an accepted residual (D6.3 applied to evidence).
            let (Some(standards), Some(criticality)) =
                (model.standards.as_ref(), claim.requirement.criticality)
            else {
                continue;
            };
            let Some(level) = standards.for_level(criticality) else {
                continue;
            };
            let weakened = match (entry.quantification, level.quantification) {
                (Some(entry_q), Some(level_q)) => entry_q < level_q,
                _ => false,
            };
            if weakened && (entry.residual.is_none() || entry.accepted.is_none()) {
                findings.push(Finding {
                    kind: FindingKind::UnacceptedWeakening,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", plan.spec, entry.scenario)),
                    criticality: Some(criticality),
                    path: plan.path.clone(),
                    line: entry.line,
                    detail: format!(
                        "requires {} where the {} standard is {}, with no accepted residual",
                        entry.quantification.unwrap().name(),
                        criticality.name(),
                        level.quantification.unwrap().name()
                    ),
                });
            }
        }
    }
    findings
}

/// Findings about the mechanism facet, including the rules that need all three artifacts.
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

    // The three-artifact rule. A plan may cite proof-strength evidence, but proof comes from a
    // mechanism at the top of the enforcement ladder (D7) — and the developer owns which
    // mechanism
    // that is. A plan claiming proof with no proof-capable mechanism behind it is asserting the
    // strongest available result out of thin air.
    for plan in &model.plans {
        for entry in &plan.entries {
            let Some(evidence) = &entry.evidence else {
                continue;
            };
            if evidence.strength != Strength::Proof {
                continue;
            }
            let Some(claim) = model.find_claim(&plan.spec, &entry.scenario) else {
                continue;
            };
            let backed = model.design_for(&plan.spec).is_some_and(|d| {
                let for_scenario = d.for_scenario(&entry.scenario);
                let for_requirement = d.for_requirement(&claim.requirement.id);
                for_scenario
                    .into_iter()
                    .chain(for_requirement)
                    .any(|e| e.mechanisms.iter().any(|m| m.kind.is_proof_capable()))
            });
            if !backed {
                findings.push(Finding {
                    kind: FindingKind::UnbackedProof,
                    severity: Severity::Error,
                    claim: Some(format!("{}#{}", plan.spec, entry.scenario)),
                    criticality: claim.requirement.criticality,
                    path: plan.path.clone(),
                    line: entry.line,
                    detail: format!(
                        "claims proof-strength evidence, but `{}` declares no mechanism at the top \
                         two rungs of the enforcement ladder",
                        claim.requirement.id
                    ),
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

/// Findings the machine tier cannot find on its own.
///
/// The machine makes structure checkable; it does not make truth checkable. Everything here comes
/// from a verdict the agent tier recorded, and the tool's contribution is to hold that verdict to a
/// fingerprint so it cannot quietly outlive what it judged.
fn judgment_findings(model: &Model) -> Vec<Finding> {
    let mut findings = Vec::new();
    if model.judgments.is_empty() {
        // D8.1: each mechanism is usable alone. A project that has not adopted the agent tier is
        // not told that every critical claim is unjudged.
        return findings;
    }

    for claim in model.claims() {
        let id = claim.id();
        let judged = model
            .judgments_for(&claim.spec.id)
            .and_then(|j| j.entry(&claim.scenario.id));

        let Some(judgment) = judged else {
            if claim.requirement.criticality == Some(Criticality::Critical) {
                findings.push(Finding {
                    kind: FindingKind::Unjudged,
                    severity: Severity::Error,
                    claim: Some(id),
                    criticality: claim.requirement.criticality,
                    path: claim.spec.path.clone(),
                    line: claim.scenario.line,
                    detail: "critical claim carries no agent-tier judgment".into(),
                });
            }
            continue;
        };

        let expected = crate::judgment::fingerprint(
            &model.claim_text(&claim),
            model.judgment_inputs(&claim.spec.id, &claim.scenario.id),
        );
        let path = model
            .judgments_for(&claim.spec.id)
            .map(|j| j.path.clone())
            .unwrap_or_default();

        if judgment.fingerprint != expected {
            findings.push(Finding {
                kind: FindingKind::StaleJudgment,
                severity: severity_for(claim.requirement.criticality),
                claim: Some(id.clone()),
                criticality: claim.requirement.criticality,
                path: path.clone(),
                line: judgment.line,
                detail: format!(
                    "judged `{}` against {}, but the claim or its evidence has changed since (now {})",
                    judgment.verdict.name(),
                    judgment.fingerprint,
                    expected
                ),
            });
            continue;
        }

        let kind = match judgment.verdict {
            crate::judgment::Verdict::Sound => continue,
            crate::judgment::Verdict::Toothless => FindingKind::ToothlessEvidence,
            crate::judgment::Verdict::DishonestTag => FindingKind::DishonestTag,
            crate::judgment::Verdict::DishonestRealization => FindingKind::DishonestRealization,
            crate::judgment::Verdict::SpecGap => FindingKind::SpecGap,
        };

        findings.push(Finding {
            kind,
            severity: severity_for(claim.requirement.criticality),
            claim: Some(id),
            criticality: claim.requirement.criticality,
            path,
            line: judgment.line,
            detail: judgment.reason.clone(),
        });
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
