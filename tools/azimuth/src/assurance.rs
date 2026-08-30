//! Stable alpha 1 wire types used by the optional Assurance Service.
//!
//! Repository-to-service projection is deliberately absent. The Run-ledger change replaces this
//! protocol atomically rather than making the verification-binding change a compatibility reader.

use crate::fingerprint::sha256;
use crate::json::Json;

pub const FORMAT: &str = "azimuth-assurance-project-snapshot";
pub const VERSION: u64 = 2;

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
    pub case_statement: String,
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
    pub fn derive(project: &str, model_fingerprint: &str, mut claims: Vec<ClaimContract>) -> Self {
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
    let mut fields = Vec::new();
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
            "caseStatement".to_string(),
            Json::str(&contract.case_statement),
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
