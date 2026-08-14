use azimuth::fingerprint::sha256;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PROJECT_SNAPSHOT_FORMAT: &str = azimuth::assurance::FORMAT;
pub const PROJECT_SNAPSHOT_VERSION: u64 = azimuth::assurance::VERSION;

pub fn record_fingerprint<T: Serialize>(record: &T) -> Result<String, serde_json::Error> {
    serde_json::to_vec(record).map(|bytes| sha256(&bytes))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceDefinition {
    pub id: String,
    pub claim: ClaimReference,
    pub assertion: String,
    pub scope: String,
    pub quantification: String,
    pub oracle: String,
    pub stage: LifecycleStage,
    pub inputs: Vec<String>,
    pub required_context: BTreeMap<String, String>,
    pub declared_at: u64,
}

impl EvidenceDefinition {
    pub fn fingerprint(&self) -> String {
        record_fingerprint(&(
            &self.claim,
            &self.assertion,
            &self.scope,
            &self.quantification,
            &self.oracle,
            self.stage,
            &self.inputs,
            &self.required_context,
        ))
        .expect("evidence definition semantics are serializable")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimReference {
    pub spec: String,
    pub claim: String,
    pub contract_fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractMount {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractArea {
    pub id: String,
    pub mounts: Vec<ContractMount>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractContribution {
    pub area: String,
    pub mount: String,
    pub path: String,
    pub enumerator: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractSurface {
    pub id: String,
    pub contributions: Vec<ContractContribution>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractStep {
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractVerification {
    pub strength: Option<String>,
    pub scope: String,
    pub quantification: Option<String>,
    pub oracle: Option<String>,
    pub residual_required: bool,
    pub residual: Option<String>,
    pub residual_acceptance: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimContract {
    pub contract_fingerprint: String,
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
        self.as_core().fingerprint()
    }

    pub fn reference(&self) -> ClaimReference {
        ClaimReference {
            spec: self.spec.clone(),
            claim: self.claim.clone(),
            contract_fingerprint: self.contract_fingerprint.clone(),
        }
    }

    fn as_core(&self) -> azimuth::assurance::ClaimContract {
        azimuth::assurance::ClaimContract {
            spec: self.spec.clone(),
            claim: self.claim.clone(),
            requirement: self.requirement.clone(),
            criticality: self.criticality.clone(),
            statement: self.statement.clone(),
            steps: self
                .steps
                .iter()
                .map(|step| azimuth::assurance::ContractStep {
                    kind: step.kind.clone(),
                    text: step.text.clone(),
                })
                .collect(),
            domain: self.domain.clone(),
            verification: azimuth::assurance::ContractVerification {
                strength: self.verification.strength.clone(),
                scope: self.verification.scope.clone(),
                quantification: self.verification.quantification.clone(),
                oracle: self.verification.oracle.clone(),
                residual_required: self.verification.residual_required,
                residual: self.verification.residual.clone(),
                residual_acceptance: self.verification.residual_acceptance.clone(),
            },
            surface: self
                .surface
                .as_ref()
                .map(|surface| azimuth::assurance::ContractSurface {
                    id: surface.id.clone(),
                    contributions: surface
                        .contributions
                        .iter()
                        .map(|item| azimuth::assurance::ContractContribution {
                            area: item.area.clone(),
                            mount: item.mount.clone(),
                            path: item.path.clone(),
                            enumerator: item.enumerator.clone(),
                        })
                        .collect(),
                }),
            obligated_areas: self
                .obligated_areas
                .iter()
                .map(|area| azimuth::assurance::ContractArea {
                    id: area.id.clone(),
                    mounts: area
                        .mounts
                        .iter()
                        .map(|mount| azimuth::assurance::ContractMount {
                            id: mount.id.clone(),
                            path: mount.path.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectModelSnapshot {
    pub format: String,
    pub version: u64,
    pub id: String,
    pub project: String,
    pub model_fingerprint: String,
    pub claims: Vec<ClaimContract>,
}

impl ProjectModelSnapshot {
    pub fn fingerprint(&self) -> String {
        let claims = self.claims.iter().map(ClaimContract::as_core).collect();
        azimuth::assurance::ProjectSnapshot {
            id: self.id.clone(),
            project: self.project.clone(),
            model_fingerprint: self.model_fingerprint.clone(),
            claims,
        }
        .fingerprint()
    }

    pub fn contains(&self, reference: &ClaimReference) -> bool {
        self.claims.iter().any(|contract| {
            contract.spec == reference.spec
                && contract.claim == reference.claim
                && contract.contract_fingerprint == reference.contract_fingerprint
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleStage {
    Merge,
    Release,
    Canary,
    Rollout,
    Operation,
}

impl LifecycleStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Release => "release",
            Self::Canary => "canary",
            Self::Rollout => "rollout",
            Self::Operation => "operation",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Qualification {
    pub id: String,
    pub definition_id: String,
    pub definition_fingerprint: String,
    pub verdict: QualificationVerdict,
    pub qualified_at: u64,
    pub rationale: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum QualificationVerdict {
    Qualified,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSubject {
    pub project_snapshot: String,
    pub revision: String,
    pub artifact_digest: Option<String>,
    pub deployment_id: Option<String>,
    pub environment: Option<String>,
    pub cohort: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub id: String,
    pub definition_id: String,
    pub definition_fingerprint: String,
    pub stage: LifecycleStage,
    pub subject: ExecutionSubject,
    pub context: BTreeMap<String, String>,
    pub observed_at: u64,
    pub expires_at: Option<u64>,
    pub outcome: ObservationOutcome,
    pub report: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationOutcome {
    Satisfied,
    Violated,
    Inconclusive,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: String,
    pub source: String,
    pub definition_id: String,
    pub definition_fingerprint: String,
    pub observed_at: u64,
    pub outcome: ChallengeOutcome,
    pub report: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChallengeOutcome {
    Clean,
    Findings,
    Inconclusive,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateRequest {
    pub definition_id: String,
    pub stage: LifecycleStage,
    pub subject: ExecutionSubject,
    pub at: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDecision {
    pub status: GateStatus,
    pub definition_fingerprint: Option<String>,
    pub qualification_id: Option<String>,
    pub observation_id: Option<String>,
    pub reasons: Vec<GateReason>,
    pub work: Vec<WorkKind>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateStatus {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateReason {
    DefinitionMissing,
    ProjectSnapshotMissing,
    ClaimContractInapplicable,
    QualificationMissing,
    QualificationRejected,
    QualificationStale,
    ObservationMissing,
    SubjectMismatch,
    ContextMismatch,
    ObservationExpired,
    AssertionViolated,
    ObservationInconclusive,
    ChallengeFindings,
    ChallengeInconclusive,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkKind {
    RegisterProjectSnapshot,
    ReviseDefinition,
    QualifyDefinition,
    ExecuteDefinition,
    RerunForSubject,
    RerunWithContext,
    RenewObservation,
    DiagnoseViolation,
    ResolveInconclusiveObservation,
    JudgeChallenge,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssuranceAccount {
    pub project_snapshots: Vec<ProjectModelSnapshot>,
    pub definitions: Vec<EvidenceDefinition>,
    pub qualifications: Vec<Qualification>,
    pub observations: Vec<Observation>,
    pub challenges: Vec<Challenge>,
}

impl AssuranceAccount {
    pub fn evaluate(&self, request: &GateRequest) -> GateDecision {
        let Some(definition) = self
            .definitions
            .iter()
            .filter(|item| item.id == request.definition_id && item.declared_at <= request.at)
            .max_by_key(|item| (item.declared_at, item.fingerprint()))
        else {
            return closed(
                None,
                None,
                None,
                vec![GateReason::DefinitionMissing],
                vec![],
            );
        };
        let fingerprint = definition.fingerprint();

        let Some(snapshot) = self
            .project_snapshots
            .iter()
            .find(|item| item.id == request.subject.project_snapshot)
        else {
            return closed(
                Some(fingerprint),
                None,
                None,
                vec![GateReason::ProjectSnapshotMissing],
                vec![WorkKind::RegisterProjectSnapshot],
            );
        };
        if !snapshot.contains(&definition.claim) {
            return closed(
                Some(fingerprint),
                None,
                None,
                vec![GateReason::ClaimContractInapplicable],
                vec![WorkKind::ReviseDefinition, WorkKind::QualifyDefinition],
            );
        }

        let qualification = self
            .qualifications
            .iter()
            .filter(|item| item.definition_id == definition.id && item.qualified_at <= request.at)
            .max_by_key(|item| (item.qualified_at, item.id.as_str()));
        let Some(qualification) = qualification else {
            return closed(
                Some(fingerprint),
                None,
                None,
                vec![GateReason::QualificationMissing],
                vec![WorkKind::QualifyDefinition],
            );
        };
        if qualification.definition_fingerprint != fingerprint {
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                vec![GateReason::QualificationStale],
                vec![WorkKind::QualifyDefinition],
            );
        }
        if qualification.verdict == QualificationVerdict::Rejected {
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                vec![GateReason::QualificationRejected],
                vec![WorkKind::QualifyDefinition],
            );
        }

        let mut reasons = Vec::new();
        let mut work = Vec::new();
        let mut current_challenges: BTreeMap<&str, &Challenge> = BTreeMap::new();
        for challenge in self.challenges.iter().filter(|item| {
            item.definition_id == definition.id
                && item.definition_fingerprint == fingerprint
                && item.observed_at <= request.at
        }) {
            let replace = current_challenges
                .get(challenge.source.as_str())
                .is_none_or(|current| {
                    (challenge.observed_at, challenge.id.as_str())
                        > (current.observed_at, current.id.as_str())
                });
            if replace {
                current_challenges.insert(&challenge.source, challenge);
            }
        }
        for challenge in current_challenges.values() {
            match challenge.outcome {
                ChallengeOutcome::Clean => {}
                ChallengeOutcome::Findings => {
                    push_unique(&mut reasons, GateReason::ChallengeFindings);
                    push_unique(&mut work, WorkKind::JudgeChallenge);
                }
                ChallengeOutcome::Inconclusive => {
                    push_unique(&mut reasons, GateReason::ChallengeInconclusive);
                    push_unique(&mut work, WorkKind::JudgeChallenge);
                }
            }
        }

        let candidates = self
            .observations
            .iter()
            .filter(|item| {
                item.definition_id == definition.id
                    && item.definition_fingerprint == fingerprint
                    && item.stage == request.stage
                    && item.observed_at <= request.at
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            push_unique(&mut reasons, GateReason::ObservationMissing);
            push_unique(&mut work, WorkKind::ExecuteDefinition);
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                reasons,
                work,
            );
        }

        let matching_subject = candidates
            .iter()
            .copied()
            .filter(|item| item.subject == request.subject)
            .collect::<Vec<_>>();
        if matching_subject.is_empty() {
            push_unique(&mut reasons, GateReason::SubjectMismatch);
            push_unique(&mut work, WorkKind::RerunForSubject);
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                reasons,
                work,
            );
        }

        let matching_context = matching_subject
            .iter()
            .copied()
            .filter(|item| context_matches(&definition.required_context, &item.context))
            .max_by_key(|item| (item.observed_at, item.id.as_str()));
        let Some(observation) = matching_context else {
            push_unique(&mut reasons, GateReason::ContextMismatch);
            push_unique(&mut work, WorkKind::RerunWithContext);
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                reasons,
                work,
            );
        };

        if observation
            .expires_at
            .is_some_and(|expires_at| request.at >= expires_at)
        {
            push_unique(&mut reasons, GateReason::ObservationExpired);
            push_unique(&mut work, WorkKind::RenewObservation);
        }
        match observation.outcome {
            ObservationOutcome::Satisfied => {}
            ObservationOutcome::Violated => {
                push_unique(&mut reasons, GateReason::AssertionViolated);
                push_unique(&mut work, WorkKind::DiagnoseViolation);
            }
            ObservationOutcome::Inconclusive => {
                push_unique(&mut reasons, GateReason::ObservationInconclusive);
                push_unique(&mut work, WorkKind::ResolveInconclusiveObservation);
            }
        }

        if reasons.is_empty() {
            GateDecision {
                status: GateStatus::Open,
                definition_fingerprint: Some(fingerprint),
                qualification_id: Some(qualification.id.clone()),
                observation_id: Some(observation.id.clone()),
                reasons,
                work,
            }
        } else {
            closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                Some(observation.id.clone()),
                reasons,
                work,
            )
        }
    }
}

fn context_matches(required: &BTreeMap<String, String>, actual: &BTreeMap<String, String>) -> bool {
    required
        .iter()
        .all(|(key, value)| actual.get(key) == Some(value))
}

fn push_unique<T: Eq + Copy>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn closed(
    definition_fingerprint: Option<String>,
    qualification_id: Option<String>,
    observation_id: Option<String>,
    reasons: Vec<GateReason>,
    work: Vec<WorkKind>,
) -> GateDecision {
    GateDecision {
        status: GateStatus::Closed,
        definition_fingerprint,
        qualification_id,
        observation_id,
        reasons,
        work,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_snapshot_json_is_the_service_wire_contract() {
        let contract = azimuth::assurance::ClaimContract {
            spec: "checkout/performance".into(),
            claim: "latency-objective".into(),
            requirement: "performance".into(),
            criticality: "standard".into(),
            statement: "Checkout latency remains bounded.".into(),
            steps: vec![azimuth::assurance::ContractStep {
                kind: "then".into(),
                text: "latency is below the threshold".into(),
            }],
            domain: "behaviour".into(),
            verification: azimuth::assurance::ContractVerification {
                strength: Some("demonstration".into()),
                scope: "e2e".into(),
                quantification: Some("example".into()),
                oracle: Some("direct".into()),
                residual_required: false,
                residual: None,
                residual_acceptance: None,
            },
            surface: None,
            obligated_areas: vec![],
        };
        let snapshot = azimuth::assurance::ProjectSnapshot::derive(
            "checkout",
            "model-fingerprint",
            vec![contract],
        );

        let wire: ProjectModelSnapshot =
            serde_json::from_str(&snapshot.to_json().to_string_pretty()).unwrap();

        assert_eq!(wire.id, snapshot.id);
        assert_eq!(wire.id, wire.fingerprint());
        assert_eq!(
            wire.claims[0].contract_fingerprint,
            wire.claims[0].fingerprint()
        );
    }
}
