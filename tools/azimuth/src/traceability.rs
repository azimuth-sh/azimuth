//! Pure derived Claim, realization, and verification traceability projection.

use crate::json::Json;
use crate::model::{Criticality, Model, Site, StepKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityReport {
    pub claims: Vec<TraceabilityClaim>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityClaim {
    pub id: String,
    pub parent_requirement: String,
    pub criticality: Option<Criticality>,
    pub statement: String,
    pub steps: Vec<TraceabilityStep>,
    pub realizations: Vec<String>,
    pub verification: Vec<TraceabilityVerification>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityStep {
    pub kind: StepKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceabilityVerification {
    pub check: String,
    pub binding: String,
    pub applicable: bool,
    pub current: bool,
    pub qualification: Option<String>,
    pub verdict: Option<String>,
}

pub fn project(model: &Model) -> TraceabilityReport {
    let mut realizations = BTreeMap::<(String, String), BTreeSet<String>>::new();
    for site in &model.realizes {
        realizations
            .entry((site.spec.clone(), site.scenario.clone()))
            .or_default()
            .extend(realization_identity(site));
    }

    let mut claims = BTreeMap::<String, TraceabilityClaim>::new();
    for claim in model.claims() {
        let id = claim.id();
        let relation_key = (claim.spec.id.clone(), claim.scenario.id.clone());
        let mut verification = model
            .evidence_bindings()
            .filter(|binding| binding.claim == id)
            .map(|binding| {
                let applicable = matches!(
                    claim.requirement.criticality,
                    Some(Criticality::Standard | Criticality::Critical)
                );
                let qualification = model
                    .qualifications()
                    .find(|qualification| qualification.id == binding.id);
                let current_qualification = qualification.filter(|qualification| {
                    applicable
                        && model
                            .expected_qualification_fingerprint(binding)
                            .is_some_and(|expected| qualification.fingerprint == expected)
                });
                let current = current_qualification.is_some();
                TraceabilityVerification {
                    check: binding.check.clone(),
                    binding: binding.id.clone(),
                    applicable,
                    current,
                    qualification: current_qualification
                        .map(|qualification| qualification.fingerprint.clone()),
                    verdict: current_qualification
                        .map(|qualification| qualification.verdict.name().to_string()),
                }
            })
            .collect::<Vec<_>>();
        verification.sort_by(|left, right| {
            (
                &left.binding,
                &left.check,
                left.applicable,
                left.current,
                &left.qualification,
                &left.verdict,
            )
                .cmp(&(
                    &right.binding,
                    &right.check,
                    right.applicable,
                    right.current,
                    &right.qualification,
                    &right.verdict,
                ))
        });
        verification.dedup();
        claims.insert(
            id.clone(),
            TraceabilityClaim {
                id,
                parent_requirement: claim.requirement.id.clone(),
                criticality: claim.requirement.criticality,
                statement: claim.requirement.statement.clone(),
                steps: claim
                    .scenario
                    .steps
                    .iter()
                    .map(|step| TraceabilityStep {
                        kind: step.kind,
                        text: step.text.clone(),
                    })
                    .collect(),
                realizations: realizations
                    .remove(&relation_key)
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                verification,
            },
        );
    }
    TraceabilityReport {
        claims: claims.into_values().collect(),
    }
}

impl TraceabilityReport {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("version", Json::Num(2.0)),
            (
                "claims",
                Json::Arr(self.claims.iter().map(TraceabilityClaim::to_json).collect()),
            ),
        ])
    }
}

impl TraceabilityClaim {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("id", Json::str(&self.id)),
            ("parent_requirement", Json::str(&self.parent_requirement)),
            (
                "criticality",
                self.criticality
                    .map(|criticality| Json::str(criticality.name()))
                    .unwrap_or(Json::Null),
            ),
            ("statement", Json::str(&self.statement)),
            (
                "steps",
                Json::Arr(self.steps.iter().map(TraceabilityStep::to_json).collect()),
            ),
            (
                "realizations",
                Json::Arr(self.realizations.iter().map(Json::str).collect()),
            ),
            (
                "verification",
                Json::Arr(
                    self.verification
                        .iter()
                        .map(TraceabilityVerification::to_json)
                        .collect(),
                ),
            ),
        ])
    }
}

impl TraceabilityStep {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("kind", Json::str(self.kind.name())),
            ("text", Json::str(&self.text)),
        ])
    }
}

impl TraceabilityVerification {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("check", Json::str(&self.check)),
            ("binding", Json::str(&self.binding)),
            ("applicable", Json::Bool(self.applicable)),
            ("current", Json::Bool(self.current)),
            (
                "qualification",
                self.qualification
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "verdict",
                self.verdict.as_ref().map(Json::str).unwrap_or(Json::Null),
            ),
        ])
    }
}

fn realization_identity(site: &Site) -> Option<String> {
    site.source.as_ref().map(|source| source.key())
}
