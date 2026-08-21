//! Pure, derived Claim-and-realization traceability projection (D44).

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityStep {
    pub kind: StepKind,
    pub text: String,
}

/// Derives traceability from the Claims already selected into `model`.
///
/// Loading owns selection. Keeping selection out of this projection prevents a report-only
/// selector language from becoming a second source of model authority. Claims and realization
/// identities are ordered and deduplicated so extractor and model traversal order cannot affect
/// the result.
pub fn project(model: &Model) -> TraceabilityReport {
    let mut realizations = BTreeMap::<(String, String), BTreeSet<String>>::new();
    for site in &model.realizes {
        realizations
            .entry((site.spec.clone(), site.scenario.clone()))
            .or_default()
            .insert(realization_identity(site));
    }

    let mut claims = BTreeMap::<String, TraceabilityClaim>::new();
    for claim in model.claims() {
        let id = claim.id();
        let relation_key = (claim.spec.id.clone(), claim.scenario.id.clone());
        let item = TraceabilityClaim {
            id: id.clone(),
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
        };
        claims.entry(id).or_insert(item);
    }

    TraceabilityReport {
        claims: claims.into_values().collect(),
    }
}

impl TraceabilityReport {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("version", Json::Num(1.0)),
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

fn realization_identity(site: &Site) -> String {
    site.source
        .as_ref()
        .map(|source| source.key())
        .unwrap_or_else(|| format!("{}#{}|{}", site.file, site.site, site.lang))
}
