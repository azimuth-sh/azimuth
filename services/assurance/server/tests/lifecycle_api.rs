use azimuth_assurance_domain::{
    Challenge, ChallengeOutcome, ClaimContract, ContractArea, ContractMount, ContractStep,
    ContractVerification, EvidenceDefinition, ExecutionSubject, GateReason, GateRequest,
    GateStatus, LifecycleStage, Observation, ObservationOutcome, ProjectModelSnapshot,
    Qualification, QualificationVerdict, WorkKind, PROJECT_SNAPSHOT_FORMAT,
    PROJECT_SNAPSHOT_VERSION,
};
use azimuth_assurance_server::{app, connect, migrate, AppState, GateDecisionRecord, Project};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error};
use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};
use tokio::{net::TcpListener, task::JoinHandle};

const PROJECT_ID: &str = "checkout";
const FIRST_RUN: u64 = 1_786_442_400;

#[tokio::test]
async fn lifecycle_protocol_survives_http_and_postgres() -> Result<(), Box<dyn Error>> {
    let postgres = Postgres::default().start().await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect(&database_url).await?;
    migrate(&pool).await?;
    let (base_url, server) = serve(pool.clone()).await?;
    let api = Api::new(base_url);

    let project = Project {
        id: PROJECT_ID.into(),
        name: "Checkout assurance".into(),
        created_at: FIRST_RUN - 100,
    };
    let created = api.post("/v1/projects", &project).await?;
    assert_eq!(created.status(), StatusCode::CREATED);
    let replayed = api.post("/v1/projects", &project).await?;
    assert_eq!(replayed.status(), StatusCode::OK);
    assert!(replayed.json::<Value>().await?["replayed"]
        .as_bool()
        .is_some_and(|value| value));
    let conflict = api
        .post(
            "/v1/projects",
            &Project {
                name: "Changed identity".into(),
                ..project.clone()
            },
        )
        .await?;
    assert_eq!(conflict.status(), StatusCode::CONFLICT);

    let contract = claim_contract();
    let snapshot_ci = model_snapshot("model-ci", vec![contract.clone()]);
    let snapshot_release = model_snapshot("model-release", vec![contract.clone()]);
    let unregistered_definition = definition(
        "expected-load",
        LifecycleStage::Merge,
        FIRST_RUN - 80,
        &contract,
    );
    assert_eq!(
        api.post(&project_path("definitions"), &unregistered_definition)
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for snapshot in [&snapshot_ci, &snapshot_release] {
        assert_eq!(
            api.post(&project_path("model-snapshots"), snapshot)
                .await?
                .status(),
            StatusCode::CREATED
        );
    }
    let replayed_snapshot = api
        .post(&project_path("model-snapshots"), &snapshot_ci)
        .await?;
    assert_eq!(replayed_snapshot.status(), StatusCode::OK);

    let merge_definition = unregistered_definition;
    let merge_fingerprint = register_definition(&api, &merge_definition).await?;
    let qualification = Qualification {
        id: "qualification-1".into(),
        definition_id: merge_definition.id.clone(),
        definition_fingerprint: merge_fingerprint.clone(),
        verdict: QualificationVerdict::Qualified,
        qualified_at: FIRST_RUN - 60,
        rationale: "The threshold, oracle and execution context match the claim.".into(),
    };
    assert_eq!(
        api.post(&project_path("qualifications"), &qualification)
            .await?
            .status(),
        StatusCode::CREATED
    );

    let subject_a = ci_subject(&snapshot_ci.id, "revision-a");
    let subject_b = ci_subject(&snapshot_ci.id, "revision-b");
    let observation_a = observation(
        "ci-a",
        &merge_definition,
        &merge_fingerprint,
        subject_a.clone(),
        FIRST_RUN,
        ObservationOutcome::Satisfied,
        None,
    );
    let observation_b = observation(
        "ci-b",
        &merge_definition,
        &merge_fingerprint,
        subject_b.clone(),
        FIRST_RUN + 10,
        ObservationOutcome::Satisfied,
        None,
    );
    for observation in [&observation_a, &observation_b] {
        assert_eq!(
            api.post(&project_path("observations"), observation)
                .await?
                .status(),
            StatusCode::CREATED
        );
    }

    let gate_a = evaluate(
        &api,
        gate_request(&merge_definition, subject_a.clone(), FIRST_RUN),
    )
    .await?;
    let gate_b = evaluate(
        &api,
        gate_request(&merge_definition, subject_b.clone(), FIRST_RUN + 10),
    )
    .await?;
    assert_eq!(gate_a.decision.status, GateStatus::Open);
    assert_eq!(gate_b.decision.status, GateStatus::Open);
    assert_eq!(
        gate_a.decision.qualification_id,
        Some(qualification.id.clone())
    );
    assert_eq!(
        gate_b.decision.qualification_id,
        Some(qualification.id.clone())
    );

    let replay = api
        .post(&project_path("observations"), &observation_a)
        .await?;
    assert_eq!(replay.status(), StatusCode::OK);
    let conflicting_observation = Observation {
        outcome: ObservationOutcome::Violated,
        ..observation_a.clone()
    };
    assert_eq!(
        api.post(&project_path("observations"), &conflicting_observation)
            .await?
            .status(),
        StatusCode::CONFLICT
    );

    let wrong_subject = evaluate(
        &api,
        gate_request(
            &merge_definition,
            ci_subject(&snapshot_ci.id, "revision-z"),
            FIRST_RUN + 20,
        ),
    )
    .await?;
    assert_closed(
        &wrong_subject,
        GateReason::SubjectMismatch,
        WorkKind::RerunForSubject,
    );

    let context_subject = ci_subject(&snapshot_ci.id, "revision-context");
    let mut wrong_context = observation(
        "ci-context",
        &merge_definition,
        &merge_fingerprint,
        context_subject.clone(),
        FIRST_RUN + 25,
        ObservationOutcome::Satisfied,
        None,
    );
    wrong_context
        .context
        .insert("capacity-profile".into(), "small".into());
    api.post(&project_path("observations"), &wrong_context)
        .await?;
    let context_gate = evaluate(
        &api,
        gate_request(&merge_definition, context_subject, FIRST_RUN + 25),
    )
    .await?;
    assert_closed(
        &context_gate,
        GateReason::ContextMismatch,
        WorkKind::RerunWithContext,
    );

    let violated_subject = ci_subject(&snapshot_ci.id, "revision-violated");
    let violated = observation(
        "ci-violated",
        &merge_definition,
        &merge_fingerprint,
        violated_subject.clone(),
        FIRST_RUN + 30,
        ObservationOutcome::Violated,
        None,
    );
    api.post(&project_path("observations"), &violated).await?;
    let violation_gate = evaluate(
        &api,
        gate_request(&merge_definition, violated_subject, FIRST_RUN + 30),
    )
    .await?;
    assert_closed(
        &violation_gate,
        GateReason::AssertionViolated,
        WorkKind::DiagnoseViolation,
    );

    let finding = Challenge {
        id: "mutation-finding".into(),
        source: "stryker".into(),
        definition_id: merge_definition.id.clone(),
        definition_fingerprint: merge_fingerprint.clone(),
        observed_at: FIRST_RUN + 40,
        outcome: ChallengeOutcome::Findings,
        report: Some("reports/mutation.json".into()),
    };
    api.post(&project_path("challenges"), &finding).await?;
    let challenged = evaluate(
        &api,
        gate_request(&merge_definition, subject_b.clone(), FIRST_RUN + 40),
    )
    .await?;
    assert_closed(
        &challenged,
        GateReason::ChallengeFindings,
        WorkKind::JudgeChallenge,
    );
    let resolved = Challenge {
        id: "mutation-clean".into(),
        observed_at: FIRST_RUN + 50,
        outcome: ChallengeOutcome::Clean,
        ..finding
    };
    api.post(&project_path("challenges"), &resolved).await?;
    let clean_gate = evaluate(
        &api,
        gate_request(&merge_definition, subject_b.clone(), FIRST_RUN + 50),
    )
    .await?;
    assert_eq!(clean_gate.decision.status, GateStatus::Open);

    let canary_definition = definition(
        "canary-health",
        LifecycleStage::Canary,
        FIRST_RUN - 70,
        &contract,
    );
    let canary_fingerprint = register_definition(&api, &canary_definition).await?;
    let canary_qualification = Qualification {
        id: "qualification-canary".into(),
        definition_id: canary_definition.id.clone(),
        definition_fingerprint: canary_fingerprint.clone(),
        verdict: QualificationVerdict::Qualified,
        qualified_at: FIRST_RUN - 50,
        rationale: "The canary health oracle is direct and deployment-confined.".into(),
    };
    api.post(&project_path("qualifications"), &canary_qualification)
        .await?;
    let production = production_subject(&snapshot_release.id, "sha256:artifact-a", "deployment-a");
    let expiry = FIRST_RUN + 3_600;
    let canary_observation = observation(
        "canary-a",
        &canary_definition,
        &canary_fingerprint,
        production.clone(),
        FIRST_RUN,
        ObservationOutcome::Satisfied,
        Some(expiry),
    );
    api.post(&project_path("observations"), &canary_observation)
        .await?;
    let expired = evaluate(
        &api,
        GateRequest {
            definition_id: canary_definition.id.clone(),
            stage: LifecycleStage::Canary,
            subject: production,
            at: expiry,
        },
    )
    .await?;
    assert_closed(
        &expired,
        GateReason::ObservationExpired,
        WorkKind::RenewObservation,
    );

    let drifted_definition = EvidenceDefinition {
        assertion: "p95 latency is below 250 milliseconds".into(),
        declared_at: FIRST_RUN + 60,
        ..merge_definition.clone()
    };
    register_definition(&api, &drifted_definition).await?;
    let stale = evaluate(
        &api,
        gate_request(&drifted_definition, subject_b, FIRST_RUN + 60),
    )
    .await?;
    assert_closed(
        &stale,
        GateReason::QualificationStale,
        WorkKind::QualifyDefinition,
    );

    let snapshot_ci_next = model_snapshot("model-ci-next", vec![contract.clone()]);
    api.post(&project_path("model-snapshots"), &snapshot_ci_next)
        .await?;
    let next_subject = ci_subject(&snapshot_ci_next.id, "revision-next");
    let needs_new_execution = evaluate(
        &api,
        gate_request(&merge_definition, next_subject.clone(), FIRST_RUN + 55),
    )
    .await?;
    assert_closed(
        &needs_new_execution,
        GateReason::SubjectMismatch,
        WorkKind::RerunForSubject,
    );
    let next_observation = observation(
        "ci-next",
        &merge_definition,
        &merge_fingerprint,
        next_subject.clone(),
        FIRST_RUN + 55,
        ObservationOutcome::Satisfied,
        None,
    );
    api.post(&project_path("observations"), &next_observation)
        .await?;
    let reused_qualification = evaluate(
        &api,
        gate_request(&merge_definition, next_subject, FIRST_RUN + 55),
    )
    .await?;
    assert_eq!(reused_qualification.decision.status, GateStatus::Open);
    assert_eq!(
        reused_qualification.decision.qualification_id,
        Some(qualification.id.clone())
    );

    let mut drifted_contract = contract.clone();
    drifted_contract.obligated_areas.push(ContractArea {
        id: "payments".into(),
        mounts: vec![ContractMount {
            id: "code".into(),
            path: "services/payments".into(),
        }],
    });
    drifted_contract.contract_fingerprint = drifted_contract.fingerprint();
    let drifted_snapshot = model_snapshot("model-drifted", vec![drifted_contract]);
    api.post(&project_path("model-snapshots"), &drifted_snapshot)
        .await?;
    let inapplicable = evaluate(
        &api,
        gate_request(
            &merge_definition,
            ci_subject(&drifted_snapshot.id, "revision-drifted"),
            FIRST_RUN + 80,
        ),
    )
    .await?;
    assert_closed(
        &inapplicable,
        GateReason::ClaimContractInapplicable,
        WorkKind::QualifyDefinition,
    );
    assert!(inapplicable
        .decision
        .work
        .contains(&WorkKind::ReviseDefinition));
    let inapplicable_observation = observation(
        "ci-inapplicable",
        &merge_definition,
        &merge_fingerprint,
        ci_subject(&drifted_snapshot.id, "revision-inapplicable"),
        FIRST_RUN + 85,
        ObservationOutcome::Satisfied,
        None,
    );
    assert_eq!(
        api.post(&project_path("observations"), &inapplicable_observation)
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let unknown_snapshot_observation = observation(
        "ci-unknown-snapshot",
        &merge_definition,
        &merge_fingerprint,
        ci_subject("missing-snapshot", "revision-missing"),
        FIRST_RUN + 90,
        ObservationOutcome::Satisfied,
        None,
    );
    assert_eq!(
        api.post(&project_path("observations"), &unknown_snapshot_observation,)
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let history: Vec<GateDecisionRecord> = api.get_json(&project_path("gate-decisions")).await?;
    assert_eq!(history.len(), 12);
    assert!(history
        .windows(2)
        .all(|pair| pair[0].sequence > pair[1].sequence));
    let work_items: Vec<Value> = api.get_json(&project_path("work-items")).await?;
    assert!(work_items.iter().any(|item| {
        item["work"]
            .as_array()
            .is_some_and(|work| work.contains(&json!("qualify-definition")))
    }));

    server.abort();
    let rebuilt_pool = connect(&database_url).await?;
    let (rebuilt_url, rebuilt_server) = serve(rebuilt_pool).await?;
    let rebuilt_api = Api::new(rebuilt_url);
    let snapshot: Value = rebuilt_api.get_json(&project_path("snapshot")).await?;
    assert_eq!(snapshot["project"]["id"], PROJECT_ID);
    assert_eq!(
        snapshot["account"]["definitions"].as_array().map(Vec::len),
        Some(3)
    );
    assert_eq!(
        snapshot["account"]["projectSnapshots"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        snapshot["account"]["qualifications"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        snapshot["gateDecisions"].as_array().map(Vec::len),
        Some(history.len())
    );
    rebuilt_server.abort();

    Ok(())
}

fn definition(
    id: &str,
    stage: LifecycleStage,
    declared_at: u64,
    contract: &ClaimContract,
) -> EvidenceDefinition {
    EvidenceDefinition {
        id: id.into(),
        claim: contract.reference(),
        assertion: "p95 latency is below 300 milliseconds".into(),
        scope: "e2e".into(),
        quantification: "example".into(),
        oracle: "direct".into(),
        stage,
        inputs: vec!["tests/load.js@sha256:definition".into()],
        required_context: BTreeMap::from([("capacity-profile".into(), "production-like".into())]),
        declared_at,
    }
}

async fn register_definition(
    api: &Api,
    definition: &EvidenceDefinition,
) -> Result<String, Box<dyn Error>> {
    let response = api.post(&project_path("definitions"), definition).await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.json::<Value>().await?;
    Ok(body["definitionFingerprint"]
        .as_str()
        .expect("definition fingerprint response")
        .to_owned())
}

fn observation(
    id: &str,
    definition: &EvidenceDefinition,
    definition_fingerprint: &str,
    subject: ExecutionSubject,
    observed_at: u64,
    outcome: ObservationOutcome,
    expires_at: Option<u64>,
) -> Observation {
    Observation {
        id: id.into(),
        definition_id: definition.id.clone(),
        definition_fingerprint: definition_fingerprint.into(),
        stage: definition.stage,
        subject,
        context: definition.required_context.clone(),
        observed_at,
        expires_at,
        outcome,
        report: Some(format!("reports/{id}.json")),
    }
}

fn gate_request(
    definition: &EvidenceDefinition,
    subject: ExecutionSubject,
    at: u64,
) -> GateRequest {
    GateRequest {
        definition_id: definition.id.clone(),
        stage: definition.stage,
        subject,
        at,
    }
}

async fn evaluate(api: &Api, request: GateRequest) -> Result<GateDecisionRecord, Box<dyn Error>> {
    let response = api.post(&project_path("gates/evaluate"), &request).await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    Ok(response.json().await?)
}

fn assert_closed(record: &GateDecisionRecord, reason: GateReason, work: WorkKind) {
    assert_eq!(record.decision.status, GateStatus::Closed);
    assert!(record.decision.reasons.contains(&reason));
    assert!(record.decision.work.contains(&work));
}

fn ci_subject(project_snapshot: &str, revision: &str) -> ExecutionSubject {
    ExecutionSubject {
        project_snapshot: project_snapshot.into(),
        revision: revision.into(),
        artifact_digest: None,
        deployment_id: None,
        environment: Some("ci".into()),
        cohort: None,
    }
}

fn production_subject(
    project_snapshot: &str,
    artifact_digest: &str,
    deployment_id: &str,
) -> ExecutionSubject {
    ExecutionSubject {
        project_snapshot: project_snapshot.into(),
        revision: "revision-release".into(),
        artifact_digest: Some(artifact_digest.into()),
        deployment_id: Some(deployment_id.into()),
        environment: Some("production".into()),
        cohort: Some("5-percent".into()),
    }
}

fn model_snapshot(model_fingerprint: &str, claims: Vec<ClaimContract>) -> ProjectModelSnapshot {
    let mut snapshot = ProjectModelSnapshot {
        format: PROJECT_SNAPSHOT_FORMAT.into(),
        version: PROJECT_SNAPSHOT_VERSION,
        id: String::new(),
        project: PROJECT_ID.into(),
        model_fingerprint: model_fingerprint.into(),
        claims,
    };
    snapshot.id = snapshot.fingerprint();
    snapshot
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
            text: "p95 latency stays below the qualified threshold".into(),
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

fn project_path(resource: &str) -> String {
    format!("/v1/projects/{PROJECT_ID}/{resource}")
}

async fn serve(
    pool: sqlx::PgPool,
) -> Result<(String, JoinHandle<Result<(), std::io::Error>>), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, app(AppState::new(pool))).await });
    Ok((format!("http://{address}"), server))
}

struct Api {
    client: Client,
    base_url: String,
}

impl Api {
    fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    async fn post<T: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, Box<dyn Error>> {
        let response = self
            .client
            .get(format!("{}{path}", self.base_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(response.json().await?)
    }
}
