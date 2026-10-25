use azimuth_assurance_lifecycle::{
    AssuranceAccount, Challenge, ChallengeOutcome, CheckResult, CheckResultOutcome,
    ContractVerification, EvidenceDefinition, ExecutionVerificationTarget, GateReason, GateRequest,
    GateStatus, LifecycleStage, ProjectModelSnapshot, Qualification, QualificationVerdict,
    RequirementContract, WorkKind, PROJECT_SNAPSHOT_FORMAT, PROJECT_SNAPSHOT_VERSION,
};
use std::collections::BTreeMap;

const FIRST_RUN: u64 = 1_786_442_400;
const SECOND_RUN: u64 = FIRST_RUN + 300;

#[test]
fn successful_ci_executions_reuse_one_qualification() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let first_verification_target = ci_verification_target("revision-a");
    let second_verification_target = ci_verification_target("revision-b");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification.clone()],
        check_results: vec![
            satisfied(
                &definition,
                "ci-a",
                first_verification_target.clone(),
                FIRST_RUN,
                None,
            ),
            satisfied(
                &definition,
                "ci-b",
                second_verification_target.clone(),
                SECOND_RUN,
                None,
            ),
        ],
        challenges: vec![],
    };

    let first = account.evaluate(&request(
        &definition,
        first_verification_target,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));
    let second = account.evaluate(&request(
        &definition,
        second_verification_target,
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
fn check_result_is_confined_to_its_exact_verification_target() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![satisfied(
            &definition,
            "ci-a",
            ci_verification_target("revision-a"),
            FIRST_RUN,
            None,
        )],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_verification_target("revision-b"),
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(
        decision.reasons,
        vec![GateReason::VerificationTargetMismatch]
    );
    assert_eq!(decision.work, vec![WorkKind::RerunForVerificationTarget]);
}

#[test]
fn injected_time_expires_a_production_check_result() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Canary);
    let verification_target = production_verification_target("sha256:artifact-a", "deployment-a");
    let expires_at = FIRST_RUN + 3_600;
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![satisfied(
            &definition,
            "canary-a",
            verification_target.clone(),
            FIRST_RUN,
            Some(expires_at),
        )],
        challenges: vec![],
    };

    let current = account.evaluate(&request(
        &definition,
        verification_target.clone(),
        expires_at - 1,
        LifecycleStage::Canary,
    ));
    let expired = account.evaluate(&request(
        &definition,
        verification_target,
        expires_at,
        LifecycleStage::Canary,
    ));

    assert_eq!(current.status, GateStatus::Open);
    assert_eq!(expired.status, GateStatus::Closed);
    assert_eq!(expired.reasons, vec![GateReason::CheckResultExpired]);
    assert_eq!(expired.work, vec![WorkKind::RenewCheckResult]);
}

#[test]
fn violation_closes_only_the_observed_gate() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let verification_target = ci_verification_target("revision-a");
    let mut check_result = satisfied(
        &definition,
        "ci-a",
        verification_target.clone(),
        FIRST_RUN,
        None,
    );
    check_result.outcome = CheckResultOutcome::Violated;
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![check_result],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        verification_target,
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
        check_results: vec![],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_verification_target("revision-a"),
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
    let verification_target = ci_verification_target("revision-a");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![satisfied(
            &definition,
            "ci-a",
            verification_target.clone(),
            FIRST_RUN,
            None,
        )],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        verification_target,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::QualificationMissing]);
    assert_eq!(decision.work, vec![WorkKind::QualifyDefinition]);
}

#[test]
fn incompatible_execution_context_requires_a_matching_rerun() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let verification_target = ci_verification_target("revision-a");
    let mut check_result = satisfied(
        &definition,
        "ci-a",
        verification_target.clone(),
        FIRST_RUN,
        None,
    );
    check_result
        .execution_context
        .insert("capacity-profile".into(), "small".into());
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![check_result],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        verification_target,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::ExecutionContextMismatch]);
    assert_eq!(decision.work, vec![WorkKind::RerunWithExecutionContext]);
}

#[test]
fn challenge_findings_require_judgment_despite_successful_execution() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let verification_target = ci_verification_target("revision-a");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![satisfied(
            &definition,
            "ci-a",
            verification_target.clone(),
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
        verification_target,
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.status, GateStatus::Closed);
    assert_eq!(decision.reasons, vec![GateReason::ChallengeFindings]);
    assert_eq!(decision.work, vec![WorkKind::JudgeChallenge]);
}

#[test]
fn production_check_result_does_not_cross_artifacts_or_deployments() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Canary);
    let observed = production_verification_target("sha256:artifact-a", "deployment-a");
    let requested = production_verification_target("sha256:artifact-b", "deployment-b");
    let account = AssuranceAccount {
        project_snapshots: snapshots(&definition),
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![satisfied(
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
    assert_eq!(
        decision.reasons,
        vec![GateReason::VerificationTargetMismatch]
    );
}

#[test]
fn unknown_snapshot_creates_registration_work() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let account = AssuranceAccount {
        project_snapshots: vec![],
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_verification_target("revision-a"),
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(decision.reasons, vec![GateReason::ProjectSnapshotMissing]);
    assert_eq!(decision.work, vec![WorkKind::RegisterProjectSnapshot]);
}

#[test]
fn changed_requirement_contract_requires_a_new_definition_and_qualification() {
    let (definition, qualification) = qualified_definition(LifecycleStage::Merge);
    let mut changed_contract = requirement_contract();
    changed_contract.statement = "Checkout latency has a different accepted bound.".into();
    changed_contract.contract_fingerprint = changed_contract.fingerprint();
    let account = AssuranceAccount {
        project_snapshots: vec![ProjectModelSnapshot {
            format: PROJECT_SNAPSHOT_FORMAT.into(),
            version: PROJECT_SNAPSHOT_VERSION,
            id: "snapshot-ci".into(),
            project: "checkout".into(),
            model_fingerprint: "model-changed".into(),
            requirements: vec![changed_contract],
        }],
        definitions: vec![definition.clone()],
        qualifications: vec![qualification],
        check_results: vec![],
        challenges: vec![],
    };

    let decision = account.evaluate(&request(
        &definition,
        ci_verification_target("revision-a"),
        FIRST_RUN,
        LifecycleStage::Merge,
    ));

    assert_eq!(
        decision.reasons,
        vec![GateReason::RequirementContractInapplicable]
    );
    assert_eq!(
        decision.work,
        vec![WorkKind::ReviseDefinition, WorkKind::QualifyDefinition]
    );
}

fn qualified_definition(stage: LifecycleStage) -> (EvidenceDefinition, Qualification) {
    let definition = EvidenceDefinition {
        id: "expected-load".into(),
        requirement: requirement_contract().reference(),
        assertion: "p95 latency is below 300 milliseconds".into(),
        scope: "e2e".into(),
        quantification: "example".into(),
        oracle: "direct".into(),
        stage,
        inputs: vec!["tests/load.js@sha256:definition".into()],
        required_execution_context: BTreeMap::from([(
            "capacity-profile".into(),
            "production-like".into(),
        )]),
        declared_at: FIRST_RUN - 2,
    };
    let qualification = Qualification {
        id: "qualification-1".into(),
        definition_id: definition.id.clone(),
        definition_fingerprint: definition.fingerprint(),
        verdict: QualificationVerdict::Qualified,
        qualified_at: FIRST_RUN - 1,
        rationale:
            "The direct threshold and production-like Execution Context test the requirement."
                .into(),
    };
    (definition, qualification)
}

fn satisfied(
    definition: &EvidenceDefinition,
    id: &str,
    verification_target: ExecutionVerificationTarget,
    observed_at: u64,
    expires_at: Option<u64>,
) -> CheckResult {
    CheckResult {
        id: id.into(),
        definition_id: definition.id.clone(),
        definition_fingerprint: definition.fingerprint(),
        stage: definition.stage,
        verification_target,
        execution_context: definition.required_execution_context.clone(),
        observed_at,
        expires_at,
        outcome: CheckResultOutcome::Satisfied,
        report: None,
    }
}

fn request(
    definition: &EvidenceDefinition,
    verification_target: ExecutionVerificationTarget,
    at: u64,
    stage: LifecycleStage,
) -> GateRequest {
    GateRequest {
        definition_id: definition.id.clone(),
        stage,
        verification_target,
        at,
    }
}

fn ci_verification_target(revision: &str) -> ExecutionVerificationTarget {
    ExecutionVerificationTarget {
        project_snapshot: "snapshot-ci".into(),
        revision: revision.into(),
        artifact_digest: None,
        deployment_id: None,
        environment: Some("ci".into()),
        cohort: None,
    }
}

fn snapshots(definition: &EvidenceDefinition) -> Vec<ProjectModelSnapshot> {
    let contract = requirement_contract();
    assert_eq!(definition.requirement, contract.reference());
    ["snapshot-ci", "snapshot-release"]
        .into_iter()
        .map(|id| ProjectModelSnapshot {
            format: PROJECT_SNAPSHOT_FORMAT.into(),
            version: PROJECT_SNAPSHOT_VERSION,
            id: id.into(),
            project: "checkout".into(),
            model_fingerprint: format!("model-{id}"),
            requirements: vec![contract.clone()],
        })
        .collect()
}

fn requirement_contract() -> RequirementContract {
    let mut contract = RequirementContract {
        contract_fingerprint: String::new(),
        spec: "checkout/performance".into(),
        requirement: "latency-objective".into(),
        requirement_kind: "performance".into(),
        realization_rigor: "none".into(),
        verification_rigor: "reviewed".into(),
        statement: "Checkout latency remains bounded.".into(),
        scenario_statement: "P95 latency is below the qualified threshold.".into(),
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

fn production_verification_target(
    artifact_digest: &str,
    deployment_id: &str,
) -> ExecutionVerificationTarget {
    ExecutionVerificationTarget {
        project_snapshot: "snapshot-release".into(),
        revision: "revision-release".into(),
        artifact_digest: Some(artifact_digest.into()),
        deployment_id: Some(deployment_id.into()),
        environment: Some("production".into()),
        cohort: Some("5-percent".into()),
    }
}
