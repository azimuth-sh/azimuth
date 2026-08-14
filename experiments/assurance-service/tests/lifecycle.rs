use azimuth_assurance_lifecycle::{
    AssuranceAccount, Challenge, ChallengeOutcome, ClaimContract, ContractStep,
    ContractVerification, EvidenceDefinition, ExecutionSubject, GateReason, GateRequest,
    GateStatus, LifecycleStage, Observation, ObservationOutcome, ProjectModelSnapshot,
    Qualification, QualificationVerdict, WorkKind, PROJECT_SNAPSHOT_FORMAT,
    PROJECT_SNAPSHOT_VERSION,
};
use std::collections::BTreeMap;

const FIRST_RUN: u64 = 1_786_442_400;
const SECOND_RUN: u64 = FIRST_RUN + 300;

#[test]
fn successful_ci_executions_reuse_one_qualification() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let first_subject = ci_subject("revision-a");
    let second_subject = ci_subject("revision-b");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification.clone()],
        observations: vec![
            satisfied(&definition, "ci-a", first_subject.clone(), FIRST_RUN, None),
            satisfied(
                &definition,
                "ci-b",
                second_subject.clone(),
                SECOND_RUN,
                None,
            ),
        ],
        challenges: vec![],
    };

    let first = account.evaluate(&request(
        &definition,
        first_subject,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));
    let second = account.evaluate(&request(
        &definition,
        second_subject,
        SECOND_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(first.status, GateStatus::Open);
    assert_eq!(second.status, GateStatus::Open);
    assert_eq!(first.qualification_id, Some(qualification.id.clone()));
    assert_eq!(second.qualification_id, Some(qualification.id));
    assert!(first.work.is_empty());
    assert!(second.work.is_empty());

    let semantic_judgments = account.qualifications.len();
    let repository_result_writes = 0;
    assert_eq!(semantic_judgments, 1);
    assert_eq!(repository_result_writes, 0);
}

#[test]
fn observation_is_confined_to_its_exact_subject() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![satisfied(
            &definition,
            "ci-a",
            ci_subject("revision-a"),
            FIRST_RUN,
            None,
        )],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_subject("revision-b"),
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::SubjectMismatch]);
    assert_eq!(decision.work, vec![WorkKind::RerunForSubject]);
}

#[test]
fn injected_time_expires_a_production_observation() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Canary);
    let subject = production_subject("sha256:artifact-a", "deployment-a");
    let expires_at = FIRST_RUN + 3_600;
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![satisfied(
            &definition,
            "canary-a",
            subject.clone(),
            FIRST_RUN,
            Some(expires_at),
        )],
        challenges: vec![],
    };

    let current = account.evaluate(&request(
        &definition,
        subject.clone(),
        expires_at - 1,
        LifecycleStage::Canary,
    ));
    let expired = account.evaluate(&request(
        &definition,
        subject,
        expires_at,
        LifecycleStage::Canary,
    ));

    assert_eq!(current.status, GateStatus::Open);
    assert_eq!(expired.status, GateStatus::Closed);
    assert_eq!(expired.reasons, vec![GateReason::ObservationExpired]);
    assert_eq!(expired.work, vec![WorkKind::RenewObservation]);
}

#[test]
fn violation_closes_only_the_observed_gate() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let subject = ci_subject("revision-a");
    let mut observation = satisfied(&definition, "ci-a", subject.clone(), FIRST_RUN, None);
    observation.outcome = ObservationOutcome::Violated;
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![observation],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        subject,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::AssertionViolated]);
    assert_eq!(decision.work, vec![WorkKind::DiagnoseViolation]);
}

#[test]
fn changed_definition_invalidates_qualification_before_execution() {
    let (mut definition, qualification) = qualified_definition(LifecycleStage::Merge);
    definition.assertion = "p95 latency is below 250 milliseconds".into();
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_subject("revision-a"),
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::QualificationStale]);
    assert_eq!(decision.work, vec![WorkKind::QualifyDefinition]);
}

#[test]
fn future_qualification_cannot_open_an_earlier_gate() {
    let (definition, mut qualification) = qualified_definition(LifecycleStage::Merge);
    qualification.qualified_at = FIRST_RUN + 1;
    let subject = ci_subject("revision-a");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![satisfied(
            &definition,
            "ci-a",
            subject.clone(),
            FIRST_RUN,
            None,
        )],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        subject,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::QualificationMissing]);
    assert_eq!(decision.work, vec![WorkKind::QualifyDefinition]);
}

#[test]
fn incompatible_context_requires_a_matching_rerun() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let subject = ci_subject("revision-a");
    let mut observation = satisfied(&definition, "ci-a", subject.clone(), FIRST_RUN, None);
    observation
        .context
        .insert("capacity-profile".into(), "small".into());
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![observation],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        subject,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::ContextMismatch]);
    assert_eq!(decision.work, vec![WorkKind::RerunWithContext]);
}

#[test]
fn challenge_findings_require_judgment_despite_successful_execution() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let subject = ci_subject("revision-a");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![satisfied(
            &definition,
            "ci-a",
            subject.clone(),
            FIRST_RUN,
            None,
        )],
        challenges: vec![Challenge {
            id: "mutation-a".into(),
            source: "stryker".into(),
            definition_id: definition.id.clone(),
            definition_fingerprint: definition.fingerprint(),
            observed_at: FIRST_RUN,
            outcome: ChallengeOutcome::Findings,
            report: None,
        }],
    };

    let decision = account.evaluate(&request(
        &definition,
        subject,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::ChallengeFindings]);
    assert_eq!(decision.work, vec![WorkKind::JudgeChallenge]);
}

#[test]
fn production_observation_does_not_cross_artifacts_or_deployments() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Canary);
    let observed = production_subject("sha256:artifact-a", "deployment-a");
    let requested = production_subject("sha256:artifact-b", "deployment-b");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![satisfied(
            &definition,
            "canary-a",
            observed,
            FIRST_RUN,
            Some(FIRST_RUN + 3_600),
        )],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        requested,
        FIRST_RUN,
        LifecycleStage::Canary,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::SubjectMismatch]);
}

#[test]
fn unknown_snapshot_creates_registration_work() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let account = AssuranceAccount {
        project_snapshots: vec![],
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_subject("revision-a"),
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.reasons, vec![GateReason::ProjectSnapshotMissing]);
    assert_eq!(decision.work, vec![WorkKind::RegisterProjectSnapshot]);
}

#[test]
fn changed_claim_contract_requires_a_new_definition_and_qualification() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let mut changed_contract = claim_contract();
    changed_contract.statement = "Checkout latency has a different accepted bound.".into();
    changed_contract.contract_fingerprint = changed_contract.fingerprint();
    let account = AssuranceAccount {
        project_snapshots: vec![ProjectModelSnapshot {
            format: PROJECT_SNAPSHOT_FORMAT.into(),
            version: PROJECT_SNAPSHOT_VERSION,
            id: "snapshot-ci".into(),
            project: "checkout".into(),
            model_fingerprint: "model-changed".into(),
            claims: vec![changed_contract],
        }],
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        observations: vec![],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_subject("revision-a"),
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(
        decision.reasons,
        vec![GateReason::ClaimContractInapplicable]
    );
    assert_eq!(
        decision.work,
        vec![WorkKind::ReviseDefinition, WorkKind::QualifyDefinition]
    );
}

fn qualified_definition(stage: LifecycleStage) -> (EvidenceDefinition, Qualification) {
    let definition = EvidenceDefinition {
        id: "expected-load".into(),
        claim: claim_contract().reference(),
        assertion: "p95 latency is below 300 milliseconds".into(),
        scope: "e2e".into(),
        quantification: "example".into(),
        oracle: "direct".into(),
        stage,
        inputs: vec!["tests/load.js@sha256:definition".into()],
        required_context: BTreeMap::from([("capacity-profile".into(), "production-like".into())]),
        declared_at: FIRST_RUN - 2,
    };
    let qualification = Qualification {
        id: "qualification-1".into(),
        definition_id: definition.id.clone(),
        definition_fingerprint: definition.fingerprint(),
        verdict: QualificationVerdict::Qualified,
        qualified_at: FIRST_RUN - 1,
        rationale: "The direct threshold and production-like context test the claim.".into(),
    };
    (definition, qualification)
}

fn satisfied(
    definition: &EvidenceDefinition,
    id: &str,
    subject: ExecutionSubject,
    observed_at: u64,
    expires_at: Option<u64>,
) -> Observation {
    Observation {
        id: id.into(),
        definition_id: definition.id.clone(),
        definition_fingerprint: definition.fingerprint(),
        stage: definition.stage,
        subject,
        context: definition.required_context.clone(),
        observed_at,
        expires_at,
        outcome: ObservationOutcome::Satisfied,
        report: None,
    }
}

fn request(
    definition: &EvidenceDefinition,
    subject: ExecutionSubject,
    at: u64,
    stage: LifecycleStage,
) -> GateRequest {
    GateRequest {
        definition_id: definition.id.clone(),
        stage,
        subject,
        at,
    }
}

fn ci_subject(revision: &str) -> ExecutionSubject {
    ExecutionSubject {
        project_snapshot: "snapshot-ci".into(),
        revision: revision.into(),
        artifact_digest: None,
        deployment_id: None,
        environment: Some("ci".into()),
        cohort: None,
    }
}

fn snapshots(definition: &EvidenceDefinition) -> Vec<ProjectModelSnapshot> {
    let contract = claim_contract();
    assert_eq!(definition.claim, contract.reference());
    ["snapshot-ci", "snapshot-release"]
        .into_iter()
        .map(|id| ProjectModelSnapshot {
            format: PROJECT_SNAPSHOT_FORMAT.into(),
            version: PROJECT_SNAPSHOT_VERSION,
            id: id.into(),
            project: "checkout".into(),
            model_fingerprint: format!("model-{id}"),
            claims: vec![contract.clone()],
        })
        .collect()
}

fn claim_contract() -> ClaimContract {
    let mut contract = ClaimContract {
        contract_fingerprint: String::new(),
        spec: "checkout/performance".into(),
        claim: "latency-objective".into(),
        requirement: "performance".into(),
        criticality: "standard".into(),
        statement: "Checkout latency remains bounded.".into(),
        steps: vec![ContractStep {
            kind: "then".into(),
            text: "p95 latency is below the qualified threshold".into(),
        }],
        domain: "behaviour".into(),
        verification: ContractVerification {
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
    contract.contract_fingerprint = contract.fingerprint();
    contract
}

fn production_subject(artifact_digest: &str, deployment_id: &str) -> ExecutionSubject {
    ExecutionSubject {
        project_snapshot: "snapshot-release".into(),
        revision: "revision-release".into(),
        artifact_digest: Some(artifact_digest.into()),
        deployment_id: Some(deployment_id.into()),
        environment: Some("production".into()),
        cohort: Some("5-percent".into()),
    }
}
