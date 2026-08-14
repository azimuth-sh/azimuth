//! Repository-authored contracts for recurring assurance executions.
//!
//! The service must know whether a qualified definition still applies to a claim, but it must not
//! become a second spec parser or enumerator. This projection keeps that authority in the accepted
//! model and transfers only canonical claim contracts plus one exact model identity.

use crate::check::Hole;
use crate::fingerprint::sha256;
use crate::json::Json;
use crate::model::{Criticality, Model};

pub const FORMAT: &str = "azimuth-assurance-project-snapshot";
pub const VERSION: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractMount {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractArea {
    pub id: String,
    pub mounts: Vec<ContractMount>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractContribution {
    pub area: String,
    pub mount: String,
    pub path: String,
    pub enumerator: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractSurface {
    pub id: String,
    pub contributions: Vec<ContractContribution>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractStep {
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractVerification {
    pub strength: Option<String>,
    pub scope: String,
    pub quantification: Option<String>,
    pub oracle: Option<String>,
    pub residual_required: bool,
    pub residual: Option<String>,
    pub residual_acceptance: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimContract {
    pub spec: String,
    pub claim: String,
    pub requirement: String,
    pub criticality: String,
    pub statement: String,
    pub steps: Vec<ContractStep>,
    pub domain: String,
    pub verification: ContractVerification,
    pub surface: Option<ContractSurface>,
    pub obligated_areas: Vec<ContractArea>,
}

impl ClaimContract {
    pub fn fingerprint(&self) -> String {
        sha256(
            contract_json(&self.canonicalized(), false)
                .to_string_pretty()
                .as_bytes(),
        )
    }

    pub fn identity(&self) -> String {
        format!("{}#{}", self.spec, self.claim)
    }

    fn canonicalized(&self) -> Self {
        let mut contract = self.clone();
        if let Some(surface) = &mut contract.surface {
            surface.contributions.sort_by(|left, right| {
                (&left.area, &left.mount, &left.enumerator, &left.path).cmp(&(
                    &right.area,
                    &right.mount,
                    &right.enumerator,
                    &right.path,
                ))
            });
        }
        for area in &mut contract.obligated_areas {
            area.mounts
                .sort_by(|left, right| (&left.id, &left.path).cmp(&(&right.id, &right.path)));
        }
        contract
            .obligated_areas
            .sort_by(|left, right| left.id.cmp(&right.id));
        contract
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectSnapshot {
    pub id: String,
    pub project: String,
    pub model_fingerprint: String,
    pub claims: Vec<ClaimContract>,
}

impl ProjectSnapshot {
    pub fn derive(project: &str, model_fingerprint: &str, claims: Vec<ClaimContract>) -> Self {
        let mut claims = claims;
        claims.sort_by_key(ClaimContract::identity);
        let mut snapshot = Self {
            id: String::new(),
            project: project.to_string(),
            model_fingerprint: model_fingerprint.to_string(),
            claims,
        };
        snapshot.id = snapshot.fingerprint();
        snapshot
    }

    pub fn fingerprint(&self) -> String {
        let mut canonical = self.clone();
        canonical.claims.sort_by_key(ClaimContract::identity);
        sha256(
            snapshot_json(&canonical, false)
                .to_string_pretty()
                .as_bytes(),
        )
    }

    pub fn to_json(&self) -> Json {
        let mut canonical = self.clone();
        canonical.claims.sort_by_key(ClaimContract::identity);
        snapshot_json(&canonical, true)
    }
}

pub fn snapshot(model: &Model, holes: &[Hole], project: &str) -> ProjectSnapshot {
    let (model_fingerprint, _) = crate::change::finalization(model, holes);
    ProjectSnapshot::derive(project, &model_fingerprint, contracts(model))
}

pub fn contracts(model: &Model) -> Vec<ClaimContract> {
    let mut contracts = model
        .claims()
        .filter_map(|claim| {
            let criticality = claim.requirement.criticality?;
            if criticality == Criticality::Routine {
                return None;
            }
            let required = model.required_for(&claim)?;
            let entry = model
                .plan_for(&claim.spec.id)
                .and_then(|plan| plan.entry(&claim.scenario.id));
            let surface = claim
                .requirement
                .over
                .as_deref()
                .and_then(|id| model.workspace.surface(id))
                .map(|surface| {
                    let mut contributions = surface
                        .contributions
                        .iter()
                        .map(|contribution| ContractContribution {
                            area: contribution.area.clone(),
                            mount: contribution.mount.clone(),
                            path: model
                                .workspace
                                .areas
                                .iter()
                                .find(|area| area.id == contribution.area)
                                .and_then(|area| {
                                    area.mounts
                                        .iter()
                                        .find(|mount| mount.id == contribution.mount)
                                })
                                .map(|mount| mount.path.clone())
                                .unwrap_or_default(),
                            enumerator: contribution.enumerator.clone(),
                        })
                        .collect::<Vec<_>>();
                    contributions.sort_by(|left, right| {
                        (&left.area, &left.mount, &left.enumerator, &left.path).cmp(&(
                            &right.area,
                            &right.mount,
                            &right.enumerator,
                            &right.path,
                        ))
                    });
                    ContractSurface {
                        id: surface.id.clone(),
                        contributions,
                    }
                });
            let mut obligated_areas = model
                .workspace
                .obligation(&claim.spec.id, &claim.scenario.id)
                .map(|obligation| {
                    obligation
                        .areas
                        .iter()
                        .filter_map(|id| model.workspace.areas.iter().find(|area| &area.id == id))
                        .map(|area| {
                            let mut mounts = area
                                .mounts
                                .iter()
                                .map(|mount| ContractMount {
                                    id: mount.id.clone(),
                                    path: mount.path.clone(),
                                })
                                .collect::<Vec<_>>();
                            mounts.sort_by(|left, right| {
                                (&left.id, &left.path).cmp(&(&right.id, &right.path))
                            });
                            ContractArea {
                                id: area.id.clone(),
                                mounts,
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            obligated_areas.sort_by(|left, right| left.id.cmp(&right.id));

            Some(ClaimContract {
                spec: claim.spec.id.clone(),
                claim: claim.scenario.id.clone(),
                requirement: claim.requirement.id.clone(),
                criticality: criticality.name().to_string(),
                statement: claim.requirement.statement.clone(),
                steps: claim
                    .scenario
                    .steps
                    .iter()
                    .map(|step| ContractStep {
                        kind: step.kind.name().to_string(),
                        text: step.text.clone(),
                    })
                    .collect(),
                domain: claim.requirement.domain.name().to_string(),
                verification: ContractVerification {
                    strength: required.strength.map(|value| value.name().to_string()),
                    scope: required.scope.name().to_string(),
                    quantification: required
                        .quantification
                        .map(|value| value.name().to_string()),
                    oracle: entry
                        .and_then(|value| value.oracle)
                        .map(|value| value.name().to_string()),
                    residual_required: model
                        .standards
                        .as_ref()
                        .and_then(|standards| standards.for_level(criticality))
                        .is_some_and(|level| level.residual_required),
                    residual: entry.and_then(|value| value.residual.clone()),
                    residual_acceptance: entry.and_then(|value| value.accepted.clone()),
                },
                surface,
                obligated_areas,
            })
        })
        .collect::<Vec<_>>();
    contracts.sort_by(|left, right| left.identity().cmp(&right.identity()));
    contracts
}

fn snapshot_json(snapshot: &ProjectSnapshot, include_id: bool) -> Json {
    let mut fields = vec![
        ("format".to_string(), Json::str(FORMAT)),
        ("version".to_string(), Json::Num(VERSION as f64)),
    ];
    if include_id {
        fields.push(("id".to_string(), Json::str(&snapshot.id)));
    }
    fields.extend([
        ("project".to_string(), Json::str(&snapshot.project)),
        (
            "modelFingerprint".to_string(),
            Json::str(&snapshot.model_fingerprint),
        ),
        (
            "claims".to_string(),
            Json::Arr(
                snapshot
                    .claims
                    .iter()
                    .map(|contract| contract_json(&contract.canonicalized(), true))
                    .collect(),
            ),
        ),
    ]);
    Json::Obj(fields)
}

fn contract_json(contract: &ClaimContract, include_fingerprint: bool) -> Json {
    let mut fields = vec![];
    if include_fingerprint {
        fields.push((
            "contractFingerprint".to_string(),
            Json::str(contract.fingerprint()),
        ));
    }
    fields.extend([
        ("spec".to_string(), Json::str(&contract.spec)),
        ("claim".to_string(), Json::str(&contract.claim)),
        ("requirement".to_string(), Json::str(&contract.requirement)),
        ("criticality".to_string(), Json::str(&contract.criticality)),
        ("statement".to_string(), Json::str(&contract.statement)),
        (
            "steps".to_string(),
            Json::Arr(
                contract
                    .steps
                    .iter()
                    .map(|step| {
                        Json::obj(vec![
                            ("kind", Json::str(&step.kind)),
                            ("text", Json::str(&step.text)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("domain".to_string(), Json::str(&contract.domain)),
        (
            "verification".to_string(),
            verification_json(&contract.verification),
        ),
        (
            "surface".to_string(),
            contract
                .surface
                .as_ref()
                .map(surface_json)
                .unwrap_or(Json::Null),
        ),
        (
            "obligatedAreas".to_string(),
            Json::Arr(contract.obligated_areas.iter().map(area_json).collect()),
        ),
    ]);
    Json::Obj(fields)
}

fn verification_json(verification: &ContractVerification) -> Json {
    Json::obj(vec![
        (
            "strength",
            verification
                .strength
                .as_ref()
                .map(Json::str)
                .unwrap_or(Json::Null),
        ),
        ("scope", Json::str(&verification.scope)),
        (
            "quantification",
            verification
                .quantification
                .as_ref()
                .map(Json::str)
                .unwrap_or(Json::Null),
        ),
        (
            "oracle",
            verification
                .oracle
                .as_ref()
                .map(Json::str)
                .unwrap_or(Json::Null),
        ),
        (
            "residualRequired",
            Json::Bool(verification.residual_required),
        ),
        (
            "residual",
            verification
                .residual
                .as_ref()
                .map(Json::str)
                .unwrap_or(Json::Null),
        ),
        (
            "residualAcceptance",
            verification
                .residual_acceptance
                .as_ref()
                .map(Json::str)
                .unwrap_or(Json::Null),
        ),
    ])
}

fn surface_json(surface: &ContractSurface) -> Json {
    Json::obj(vec![
        ("id", Json::str(&surface.id)),
        (
            "contributions",
            Json::Arr(
                surface
                    .contributions
                    .iter()
                    .map(|contribution| {
                        Json::obj(vec![
                            ("area", Json::str(&contribution.area)),
                            ("mount", Json::str(&contribution.mount)),
                            ("path", Json::str(&contribution.path)),
                            ("enumerator", Json::str(&contribution.enumerator)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn area_json(area: &ContractArea) -> Json {
    Json::obj(vec![
        ("id", Json::str(&area.id)),
        (
            "mounts",
            Json::Arr(
                area.mounts
                    .iter()
                    .map(|mount| {
                        Json::obj(vec![
                            ("id", Json::str(&mount.id)),
                            ("path", Json::str(&mount.path)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}
