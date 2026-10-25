//! Pure derived Requirement, realization, and verification traceability projection.

use crate::json::Json;
use crate::model::{Model, Rigor, Site};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityReport {
    pub scenarios: Vec<TraceabilityScenario>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityScenario {
    pub id: String,
    pub parent_requirement: String,
    pub realization_rigor: Option<Rigor>,
    pub verification_rigor: Option<Rigor>,
    pub statement: String,
    pub scenario_statement: String,
    pub realizations: Vec<String>,
    pub verification: Vec<TraceabilityVerification>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceabilityVerification {
    pub check: String,
    pub contribution: String,
}

pub fn project(model: &Model) -> TraceabilityReport {
    let mut realizations = BTreeMap::<String, BTreeSet<String>>::new();
    for site in &model.realizes {
        realizations
            .entry(site.requirement.clone())
            .or_default()
            .extend(realization_identity(site));
    }

    let mut scenarios = BTreeMap::<String, TraceabilityScenario>::new();
    for view in model.scenarios() {
        let id = view.id();
        let mut verification = model
            .account_verifications
            .iter()
            .flat_map(|file| &file.checks)
            .flat_map(|check| {
                check
                    .bindings
                    .iter()
                    .filter(|binding| binding.scenario == id)
                    .map(|binding| TraceabilityVerification {
                        check: check.definition.id.clone(),
                        contribution: binding.contribution.clone(),
                    })
            })
            .collect::<Vec<_>>();
        verification.sort();
        verification.dedup();
        scenarios.insert(
            id.clone(),
            TraceabilityScenario {
                id,
                parent_requirement: view.requirement.id.clone(),
                realization_rigor: view.requirement.realization,
                verification_rigor: view.requirement.verification,
                statement: view.requirement.statement.clone(),
                scenario_statement: view.scenario.statement.clone(),
                realizations: realizations
                    .get(&view.requirement.id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                verification,
            },
        );
    }
    TraceabilityReport {
        scenarios: scenarios.into_values().collect(),
    }
}

impl TraceabilityReport {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("version", Json::Num(6.0)),
            (
                "scenarios",
                Json::Arr(
                    self.scenarios
                        .iter()
                        .map(TraceabilityScenario::to_json)
                        .collect(),
                ),
            ),
        ])
    }
}

impl TraceabilityScenario {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("id", Json::str(&self.id)),
            ("parent_requirement", Json::str(&self.parent_requirement)),
            (
                "realization_rigor",
                self.realization_rigor
                    .map(|rigor| Json::str(rigor.name()))
                    .unwrap_or(Json::Null),
            ),
            (
                "verification_rigor",
                self.verification_rigor
                    .map(|rigor| Json::str(rigor.name()))
                    .unwrap_or(Json::Null),
            ),
            ("statement", Json::str(&self.statement)),
            ("scenario_statement", Json::str(&self.scenario_statement)),
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

impl TraceabilityVerification {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("check", Json::str(&self.check)),
            ("contribution", Json::str(&self.contribution)),
        ])
    }
}

fn realization_identity(site: &Site) -> Option<String> {
    site.source.as_ref().map(|source| source.key())
}
