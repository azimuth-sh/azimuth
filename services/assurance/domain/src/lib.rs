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
    pub requirement: RequirementReference,
    pub assertion: String,
    pub scope: String,
    pub quantification: String,
    pub oracle: String,
    pub stage: LifecycleStage,
    pub inputs: Vec<String>,
    pub required_execution_context: BTreeMap<String, String>,
    pub declared_at: u64,
}

impl EvidenceDefinition {
    pub fn fingerprint(&self) -> String {
        record_fingerprint(&(
            &self.requirement,
            &self.assertion,
            &self.scope,
            &self.quantification,
            &self.oracle,
            self.stage,
            &self.inputs,
            &self.required_execution_context,
        ))
        .expect("evidence definition semantics are serializable")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementReference {
    pub spec: String,
    pub requirement: String,
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
pub struct RequirementContract {
    pub contract_fingerprint: String,
    pub spec: String,
    pub requirement: String,
    pub requirement_kind: String,
    pub realization_rigor: String,
    pub verification_rigor: String,
    pub statement: String,
    pub scenario_statement: String,
    pub domain: String,
    pub verification: ContractVerification,
    pub surface: Option<ContractSurface>,
    pub obligated_areas: Vec<ContractArea>,
}

impl RequirementContract {
    pub fn fingerprint(&self) -> String {
        self.as_core().fingerprint()
    }

    pub fn reference(&self) -> RequirementReference {
        RequirementReference {
            spec: self.spec.clone(),
            requirement: self.requirement.clone(),
            contract_fingerprint: self.contract_fingerprint.clone(),
        }
    }

    fn as_core(&self) -> azimuth::assurance::RequirementContract {
        azimuth::assurance::RequirementContract {
            spec: self.spec.clone(),
            requirement: self.requirement.clone(),
            requirement_kind: self.requirement_kind.clone(),
            realization_rigor: self.realization_rigor.clone(),
            verification_rigor: self.verification_rigor.clone(),
            statement: self.statement.clone(),
            scenario_statement: self.scenario_statement.clone(),
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
    pub requirements: Vec<RequirementContract>,
}

impl ProjectModelSnapshot {
    pub fn fingerprint(&self) -> String {
        let requirements = self
            .requirements
            .iter()
            .map(RequirementContract::as_core)
            .collect();
        azimuth::assurance::ProjectSnapshot {
            id: self.id.clone(),
            project: self.project.clone(),
            model_fingerprint: self.model_fingerprint.clone(),
            requirements,
        }
        .fingerprint()
    }

    pub fn contains(&self, reference: &RequirementReference) -> bool {
        self.requirements.iter().any(|contract| {
            contract.spec == reference.spec
                && contract.requirement == reference.requirement
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
pub struct ExecutionVerificationTarget {
    pub project_snapshot: String,
    pub revision: String,
    pub artifact_digest: Option<String>,
    pub deployment_id: Option<String>,
    pub environment: Option<String>,
    pub cohort: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    pub id: String,
    pub definition_id: String,
    pub definition_fingerprint: String,
    pub stage: LifecycleStage,
    pub verification_target: ExecutionVerificationTarget,
    pub execution_context: BTreeMap<String, String>,
    pub observed_at: u64,
    pub expires_at: Option<u64>,
    pub outcome: CheckResultOutcome,
    pub report: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckResultOutcome {
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
    pub verification_target: ExecutionVerificationTarget,
    pub at: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDecision {
    pub status: GateStatus,
    pub definition_fingerprint: Option<String>,
    pub qualification_id: Option<String>,
    pub check_result_id: Option<String>,
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
    RequirementContractInapplicable,
    QualificationMissing,
    QualificationRejected,
    QualificationStale,
    CheckResultMissing,
    VerificationTargetMismatch,
    ExecutionContextMismatch,
    CheckResultExpired,
    AssertionViolated,
    CheckResultInconclusive,
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
    RerunForVerificationTarget,
    RerunWithExecutionContext,
    RenewCheckResult,
    DiagnoseViolation,
    ResolveInconclusiveCheckResult,
    JudgeChallenge,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssuranceAccount {
    pub project_snapshots: Vec<ProjectModelSnapshot>,
    pub definitions: Vec<EvidenceDefinition>,
    pub qualifications: Vec<Qualification>,
    pub check_results: Vec<CheckResult>,
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
            .find(|item| item.id == request.verification_target.project_snapshot)
        else {
            return closed(
                Some(fingerprint),
                None,
                None,
                vec![GateReason::ProjectSnapshotMissing],
                vec![WorkKind::RegisterProjectSnapshot],
            );
        };
        if !snapshot.contains(&definition.requirement) {
            return closed(
                Some(fingerprint),
                None,
                None,
                vec![GateReason::RequirementContractInapplicable],
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
            .check_results
            .iter()
            .filter(|item| {
                item.definition_id == definition.id
                    && item.definition_fingerprint == fingerprint
                    && item.stage == request.stage
                    && item.observed_at <= request.at
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            push_unique(&mut reasons, GateReason::CheckResultMissing);
            push_unique(&mut work, WorkKind::ExecuteDefinition);
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                reasons,
                work,
            );
        }

        let matching_verification_target = candidates
            .iter()
            .copied()
            .filter(|item| item.verification_target == request.verification_target)
            .collect::<Vec<_>>();
        if matching_verification_target.is_empty() {
            push_unique(&mut reasons, GateReason::VerificationTargetMismatch);
            push_unique(&mut work, WorkKind::RerunForVerificationTarget);
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                reasons,
                work,
            );
        }

        let matching_execution_context = matching_verification_target
            .iter()
            .copied()
            .filter(|item| {
                execution_context_matches(
                    &definition.required_execution_context,
                    &item.execution_context,
                )
            })
            .max_by_key(|item| (item.observed_at, item.id.as_str()));
        let Some(check_result) = matching_execution_context else {
            push_unique(&mut reasons, GateReason::ExecutionContextMismatch);
            push_unique(&mut work, WorkKind::RerunWithExecutionContext);
            return closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                None,
                reasons,
                work,
            );
        };

        if check_result
            .expires_at
            .is_some_and(|expires_at| request.at >= expires_at)
        {
            push_unique(&mut reasons, GateReason::CheckResultExpired);
            push_unique(&mut work, WorkKind::RenewCheckResult);
        }
        match check_result.outcome {
            CheckResultOutcome::Satisfied => {}
            CheckResultOutcome::Violated => {
                push_unique(&mut reasons, GateReason::AssertionViolated);
                push_unique(&mut work, WorkKind::DiagnoseViolation);
            }
            CheckResultOutcome::Inconclusive => {
                push_unique(&mut reasons, GateReason::CheckResultInconclusive);
                push_unique(&mut work, WorkKind::ResolveInconclusiveCheckResult);
            }
        }

        if reasons.is_empty() {
            GateDecision {
                status: GateStatus::Open,
                definition_fingerprint: Some(fingerprint),
                qualification_id: Some(qualification.id.clone()),
                check_result_id: Some(check_result.id.clone()),
                reasons,
                work,
            }
        } else {
            closed(
                Some(fingerprint),
                Some(qualification.id.clone()),
                Some(check_result.id.clone()),
                reasons,
                work,
            )
        }
    }
}

fn execution_context_matches(
    required: &BTreeMap<String, String>,
    actual: &BTreeMap<String, String>,
) -> bool {
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
    check_result_id: Option<String>,
    reasons: Vec<GateReason>,
    work: Vec<WorkKind>,
) -> GateDecision {
    GateDecision {
        status: GateStatus::Closed,
        definition_fingerprint,
        qualification_id,
        check_result_id,
        reasons,
        work,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_snapshot_json_is_the_service_wire_contract() {
        let contract = azimuth::assurance::RequirementContract {
            spec: "checkout/performance".into(),
            requirement: "latency-objective".into(),
            requirement_kind: "performance".into(),
            realization_rigor: "none".into(),
            verification_rigor: "reviewed".into(),
            statement: "Checkout latency remains bounded.".into(),
            scenario_statement: "Latency is below the threshold.".into(),
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
            wire.requirements[0].contract_fingerprint,
            wire.requirements[0].fingerprint()
        );
    }
}
