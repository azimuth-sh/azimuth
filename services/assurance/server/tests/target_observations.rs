use azimuth::run::*;
use azimuth_assurance_server::{
    connect, migrate,
    run_ledger::{self, LedgerState, ProjectCredentials},
};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ImageExt},
};

const OWNER: &str = "owner-credential-for-context-integration-only";
const PUBLISHER: &str = "publisher-credential-for-context-integration-only";
const PRODUCER: &str = "producer-credential-for-context-integration-only";
const VIEWER: &str = "viewer-credential-for-context-integration-only";
fn fp(c: char) -> String {
    format!("sha256:{}", c.to_string().repeat(64))
}
fn native(value: &Value) -> azimuth::json::Json {
    azimuth::json::parse(&value.to_string()).unwrap()
}
fn value(value: &azimuth::json::Json) -> Value {
    serde_json::from_str(&value.to_string_pretty()).unwrap()
}
fn contract() -> Value {
    let mut result = json!({"name":"api","fields":{"enabled":"boolean","revision":"integer","mode":"string"}});
    result["fingerprint"] = json!(azimuth::run::canonical_fingerprint(&native(&result)).unwrap());
    result
}
fn publication_policy(policy: Value) -> Value {
    publication_variant(policy, 'e')
}
fn publication_variant(policy: Value, artifact: char) -> Value {
    let deployment_id = format!("deployment-{artifact}");
    let model=azimuth::model::Model {
        specs:vec![azimuth::spec::parse_spec("spec.md","# Spec: sample\n\n## Requirement: restricted\nCriticality: routine\n\nAccess SHALL be restricted.\n\n### Scenario: works\nEvent: request\nRequired: restricted access\n").unwrap()],
        account_verifications:vec![azimuth::account_verification::parse_account_verification("verification.md",&format!("# Verification: sample\n\n## Requirement verification: restricted\n\n### Check: recovery\n- Deployment: {policy}\n- Evidence bindings:\n  - Scenario: `works`\n    - Contribution: Establishes access restrictions.\n\nExercise access restrictions.\n")).unwrap()],
        ..Default::default()
    };
    let authority = value(&azimuth::assurance_review::authority(&model, "project").unwrap());
    let verification_target = json!({"kind":"deployment","environment":"production","deployment":deployment_id,"deployment_fingerprint":fp('c'),"artifacts":[{"id":"api","digest":fp(artifact)}]});
    let verification_target_fp = azimuth::run::verification_target_fingerprint(
        &azimuth::run::verification_target_from_json(&native(&verification_target)).unwrap(),
    );
    let deployment = json!({"id":deployment_id,"environment":"production","configuration_fingerprint":fp('c'),"artifacts":[fp(artifact)],"verification_target":verification_target,"verification_target_fingerprint":verification_target_fp});
    value(&azimuth::release::prepare(&native(&json!({
        "format":"azimuth-release-request","version":2,"project":"project","label":"Synthetic release",
        "changes":[{"id":"restricted","revision":fp('a')}],
        "sources":[{"repository":"root","commit":artifact.to_string().repeat(40),"content_fingerprint":fp('a')}],
        "builds":[{"id":"build-api","sources":["root"],"toolchain":{"compiler":"1"},"configuration_fingerprint":fp('b'),"artifacts":[fp(artifact)]}],
        "artifacts":[{"digest":fp(artifact),"kind":"container"}],"deployments":[deployment],"components":[{"id":"api","digest":fp(artifact)}],
        "snapshot":{"project":"project","model_revision":authority["fingerprint"],"authority":authority,"reviews":[],"bundles":[]},"evidence":[]
    }))).unwrap())
}

fn context() -> BTreeMap<String, String> {
    BTreeMap::new()
}
fn valid_bundle() -> RunBundle {
    let check = CheckSelection {
        id: "recovery".into(),
        fingerprint: fp('b'),
        scenarios: vec!["works".into()],
        implementations: vec![Implementation {
            identity: "payments|rust-symbol|recovery::replay".into(),
            source_fingerprint: fp('c'),
        }],
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::new(),
        }],
    };
    let mut bundle = RunBundle {
        contributions: Vec::new(),
        run_id: fp('0'),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: fp('0'),
        verification_target: VerificationTarget::Workspace {
            repositories: vec![RepositoryState {
                id: "root".into(),
                revision: "revision-a".into(),
                content_fingerprint: fp('a'),
            }],
        },
        verification_target_fingerprint: fp('0'),
        planned_at_ms: 1,
        started_at_ms: 2,
        finished_at_ms: 4,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: fp('d'),
            required_execution_context: context(),
            checks: vec![check.clone()],
            challenges: vec![],
            fingerprint: fp('0'),
        },
        actual_selection: ActualSelection {
            execution_context: context(),
            plan_fingerprint: fp('0'),
            checks: vec![check],
            challenges: vec![],
            fingerprint: fp('0'),
        },
        provenance: Provenance {
            mode: ProvenanceMode::Execute,
            source: SourceProvenance {
                system: "local-runner".into(),
                execution: "native-17".into(),
                uri: None,
            },
            normalizer: Normalizer {
                id: "adapter/synthetic".into(),
                version: "alpha.2".into(),
                build_fingerprint: fp('e'),
            },
            adapter: AdapterProvenance {
                id: "synthetic".into(),
                adapter_version: "alpha.2".into(),
                adapter_fingerprint: fp('e'),
                descriptor_fingerprint: fp('f'),
                configuration_fingerprint: fp('7'),
                launch_fingerprint: fp('8'),
                routes: vec![LaunchRoute {
                    selection: RouteSelection {
                        kind: RouteSelectionKind::Check,
                        id: "recovery".into(),
                    },
                    capability: RouteCapability {
                        address: "synthetic/checks".into(),
                        class: RouteCapabilityClass::CheckExecute,
                        challenge_form: None,
                        fingerprint: fp('9'),
                    },
                    inputs: Vec::new(),
                }],
                import_inputs: vec![],
            },
            generated_at_ms: 5,
            principal: None,
            attributes: None,
        },
        artifacts: vec![],
        diagnostics: vec![],
        activities: vec![Activity {
            id: "check-attempt".into(),
            status: ActivityStatus::Completed,
            started_at_ms: 2,
            finished_at_ms: 3,
            artifacts: vec![],
            diagnostics: vec![],
            attributes: BTreeMap::new(),
        }],
        check_executions: vec![CheckExecution {
            check: CheckRef {
                id: "recovery".into(),
                fingerprint: fp('b'),
            },
            units: vec![CheckExecutionUnit {
                id: "whole".into(),
                input_fingerprint: None,
                executed_tests: Vec::new(),
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "check-attempt".into(),
                    outcomes: [("works".into(), CheckResultOutcome::Satisfied)]
                        .into_iter()
                        .collect(),
                }],
            }],
            results: vec![CheckResult {
                scenario: "works".into(),
                outcome: CheckResultOutcome::Satisfied,
                observed_at_ms: 3,
                fingerprint: fp('0'),
                artifacts: vec![],
                diagnostics: vec![],
            }],
        }],
        verification_challenge_executions: vec![],
    };
    refresh(&mut bundle);
    bundle
}
fn refresh(bundle: &mut RunBundle) {
    bundle.verification_target_fingerprint =
        verification_target_fingerprint(&bundle.verification_target);
    bundle.plan.fingerprint =
        plan_fingerprint(&bundle.verification_target_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    bundle.provenance.adapter.launch_fingerprint = launch_fingerprint(
        bundle.provenance.mode,
        bundle.planned_at_ms,
        &bundle.verification_target,
        &bundle.verification_target_fingerprint,
        &bundle.plan,
        &LaunchAdapterIdentity {
            id: bundle.provenance.adapter.id.clone(),
            adapter_version: bundle.provenance.adapter.adapter_version.clone(),
            adapter_fingerprint: bundle.provenance.adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: bundle.provenance.adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: bundle.provenance.adapter.configuration_fingerprint.clone(),
        },
        &bundle.provenance.adapter.routes,
    );
    bundle.run_id = run_id(bundle);
    for index in 0..bundle.check_executions.len() {
        for result_index in 0..bundle.check_executions[index].results.len() {
            let fingerprint = check_result_fingerprint(
                bundle,
                &bundle.check_executions[index],
                &bundle.check_executions[index].results[result_index],
            );
            bundle.check_executions[index].results[result_index].fingerprint = fingerprint;
        }
    }
    for index in 0..bundle.verification_challenge_executions.len() {
        bundle.verification_challenge_executions[index]
            .result
            .fingerprint =
            challenge_result_fingerprint(bundle, &bundle.verification_challenge_executions[index]);
    }
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}

fn target(start: i64, end: i64, execution: &str, contract: &Value) -> Value {
    let report = json!({"format":"azimuth-execution-context","version":2,"contexts":{"api":{"contract_revision":contract["fingerprint"],"values":{"enabled":true,"revision":1,"mode":"strict"}}}});
    json!({"format":"azimuth-target-observation","version":2,"kind":"context","source":"runtime","execution_id":execution,"environment":"production","started_at_ms":start,"finished_at_ms":end,"targets":[{"component":"api","image":format!("registry/api@{}",fp('e')),"instance":"process-a"}],"check":null,"capture":{"component":"api","instance":"process-a","started_at_ms":start,"finished_at_ms":end,"before":{"captured_at_ms":start,"instance":"process-a","report":report},"after":{"captured_at_ms":end,"instance":"process-a","report":report}}})
}
fn check_target(mut raw: Value, package: &Value, execution: &str) -> Value {
    let bundle = valid_bundle();
    let mut plan = value(&plan_to_json(&bundle.plan));
    plan.as_object_mut().unwrap().remove("fingerprint");
    plan["checks"][0]["fingerprint"] =
        package["snapshot"]["authority"]["checks"][0]["fingerprint"].clone();
    let mut adapter = value(&adapter_provenance_to_json(&bundle.provenance.adapter));
    let mut routes = adapter["routes"].clone();
    routes[0]["capability"]["class"] = json!("check.import");
    for field in ["routes", "import_inputs", "launch_fingerprint"] {
        adapter.as_object_mut().unwrap().remove(field);
    }
    raw["kind"] = json!("check");
    raw["execution_id"] = json!(execution);
    raw["capture"] = Value::Null;
    raw["check"] = json!({"definition":{"plan":plan,"adapters":[adapter],"routes":routes},"outcome":"inconclusive","details":"Uncontrolled observation does not establish isolation"});
    raw
}

#[tokio::test]
async fn retained_target_evidence_resolves_historically_with_exact_replay_and_roles(
) -> Result<(), Box<dyn Error>> {
    let postgres = Postgres::default().with_tag("17-alpine").start().await?;
    let pool = connect(&format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        postgres.get_host_port_ipv4(5432).await?
    ))
    .await?;
    migrate(&pool).await?;
    let state = LedgerState::new(
        pool.clone(),
        BTreeMap::from([(
            "project".into(),
            ProjectCredentials {
                owners: BTreeMap::from([("owner".into(), OWNER.into())]),
                publishers: BTreeMap::from([("publisher".into(), PUBLISHER.into())]),
                producers: BTreeMap::from([("producer".into(), PRODUCER.into())]),
                reviewers: BTreeMap::from([(
                    "reviewer".into(),
                    "reviewer-credential-for-target-integration-only".into(),
                )]),
                viewers: BTreeMap::from([("viewer".into(), VIEWER.into())]),
            },
        )]),
    )
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}/v1/projects/project", listener.local_addr()?);
    let server =
        tokio::spawn(async move { axum::serve(listener, run_ledger::app(state)).await.unwrap() });
    let client = Client::new();
    let contract = contract();
    let response = client
        .post(format!("{base}/context-contracts"))
        .bearer_auth(OWNER)
        .json(&json!({"contract":contract}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let package = publication_policy(
        json!({"verification_target":"deployment","components":["api"],"contexts":[],"contribution":"counterexample","monitor":null}),
    );
    let response = client
        .post(format!("{base}/releases"))
        .bearer_auth(PUBLISHER)
        .json(&package)
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let now: i64 =
        sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
            .fetch_one(&pool)
            .await?;
    let mut observed = json!({"format":"azimuth-deployment-observation","version":1,"observed_at_ms":now-5000,"valid_for_seconds":60,"state":"complete","release_fingerprint":package["release"]["fingerprint"],"applied_revision":"a".repeat(40),"rollout_id":"rollout-1","configuration_fingerprint":fp('c'),"components":[{"name":"api","image":format!("registry/api@{}",fp('e')),"template_fingerprint":fp('c'),"ready":true,"instances":["process-a"]}],"diagnostics":[]});
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":0,"observation":observed}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let raw = target(now - 4000, now - 3000, "context-1", &contract);
    let endpoint = format!("{base}/target-observations");
    assert_eq!(
        client
            .post(&endpoint)
            .bearer_auth(PRODUCER)
            .json(&json!({"observation":raw}))
            .send()
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    for expected in [StatusCode::CREATED, StatusCode::OK] {
        let response = client
            .post(&endpoint)
            .bearer_auth(PUBLISHER)
            .json(&json!({"observation":raw}))
            .send()
            .await?;
        assert_eq!(response.status(), expected, "{}", response.text().await?);
    }
    let pending: Value = client
        .get(&endpoint)
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    assert!(pending["records"][0]["resolution"].is_null(), "{pending}");
    assert_eq!(pending["records"][0]["observation"], raw);
    let first_receipt = pending["records"][0]["received_at_ms"].clone();
    let mut conflict = raw.clone();
    conflict["targets"][0]["instance"] = json!("another");
    conflict["capture"]["instance"] = json!("another");
    conflict["capture"]["before"]["instance"] = json!("another");
    conflict["capture"]["after"]["instance"] = json!("another");
    assert_eq!(
        client
            .post(&endpoint)
            .bearer_auth(PUBLISHER)
            .json(&json!({"observation":conflict}))
            .send()
            .await?
            .status(),
        StatusCode::CONFLICT
    );
    let check = check_target(raw.clone(), &package, "check-1");
    let response = client
        .post(&endpoint)
        .bearer_auth(PRODUCER)
        .json(&json!({"observation":check}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    observed["observed_at_ms"] = json!(now - 2000);
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":1,"observation":observed}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let resolved: Value = client
        .get(&endpoint)
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(resolved["records"].as_array().unwrap().len(), 2);
    let old_deployment = resolved["records"][0]["resolution"]["deployment"].clone();
    assert!(old_deployment.is_string(), "{resolved}");
    assert_eq!(resolved["records"][0]["received_at_ms"], first_receipt);
    assert!(resolved["records"][0]["resolution"]["capture_fingerprint"].is_string());
    let run_fp = resolved["records"][1]["resolution"]["run_fingerprint"]
        .as_str()
        .unwrap();
    let run: Value = sqlx::query_scalar(
        "SELECT assurance_record('run',project,fingerprint) FROM run_bundles WHERE fingerprint=$1",
    )
    .bind(run_fp)
    .fetch_one(&pool)
    .await?;
    let native = azimuth::run::parse("stored", &run.to_string()).unwrap();
    assert!(azimuth::run::verify(&native).is_empty());
    assert_eq!(
        run["provenance"]["normalizer"]["id"],
        "adapter/assurance-target-observations"
    );
    assert_eq!(
        run["provenance"]["attributes"]["source-adapter-fingerprint"],
        check["check"]["definition"]["adapters"][0]["adapter_fingerprint"]
    );
    assert!(
        run["provenance"]["generated_at_ms"].as_i64().unwrap()
            > run["finished_at_ms"].as_i64().unwrap()
    );
    let normalizers: Value = client
        .get(format!("{base}/target-normalizers"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(normalizers["records"].as_array().unwrap().len(), 1);
    let descriptor = &normalizers["records"][0]["description"];
    azimuth::adapter::parse_description(&descriptor.to_string()).unwrap();
    assert_eq!(
        descriptor["adapter_fingerprint"],
        run["provenance"]["normalizer"]["build_fingerprint"]
    );
    assert_eq!(
        descriptor["descriptor_fingerprint"],
        run["provenance"]["adapter"]["descriptor_fingerprint"]
    );
    assert_eq!(
        normalizers["records"][0]["configuration"]["fingerprint"],
        run["provenance"]["adapter"]["configuration_fingerprint"]
    );

    assert_eq!(
        run["provenance"]["attributes"]["target-observation"],
        resolved["records"][1]["fingerprint"]
    );
    assert_eq!(resolved["records"][1]["submitted_by"], "producer");
    observed["rollout_id"] = json!("rollback");
    observed["observed_at_ms"] = json!(now - 1000);
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":2,"observation":observed}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let delayed = target(now - 4000, now - 3000, "delayed", &contract);
    let response = client
        .post(&endpoint)
        .bearer_auth(PUBLISHER)
        .json(&json!({"observation":delayed}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let crossing = target(now - 1500, now - 500, "crossing-rollout", &contract);
    let response = client
        .post(&endpoint)
        .bearer_auth(PUBLISHER)
        .json(&json!({"observation":crossing}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    observed["observed_at_ms"] = json!(now);
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":3,"observation":observed}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let records: Value = client
        .get(&endpoint)
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(
        records["records"][2]["resolution"]["deployment"], old_deployment,
        "{records}"
    );
    assert!(records["records"][3]["resolution"].is_null(), "{records}");
    let concurrent = target(now - 4000, now - 3000, "concurrent", &contract);
    let a = client
        .post(&endpoint)
        .bearer_auth(PUBLISHER)
        .json(&json!({"observation":concurrent}));
    let b = client
        .post(&endpoint)
        .bearer_auth(PUBLISHER)
        .json(&json!({"observation":concurrent}));
    let (a, b) = tokio::join!(a.send(), b.send());
    let mut statuses = vec![a?.status().as_u16(), b?.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, vec![200, 201]);
    let mut invalid = check.clone();
    invalid["execution_id"] = json!("positive");
    invalid["check"]["outcome"] = json!("satisfied");
    assert_eq!(
        client
            .post(&endpoint)
            .bearer_auth(PRODUCER)
            .json(&json!({"observation":invalid}))
            .send()
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut previous_version = check.clone();
    previous_version["execution_id"] = json!("previous-version");
    previous_version["version"] = json!(1);
    let response = client
        .post(&endpoint)
        .bearer_auth(PRODUCER)
        .json(&json!({"observation":previous_version}))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response
        .text()
        .await?
        .contains("expected bounded azimuth-target-observation version 2"));
    let mut unknown = check.clone();
    unknown["execution_id"] = json!("unknown");
    unknown["check"]["definition"]["plan"]["checks"][0]["fingerprint"] = json!(fp('9'));
    let response = client
        .post(&endpoint)
        .bearer_auth(PRODUCER)
        .json(&json!({"observation":unknown}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let mut empty = check.clone();
    empty["execution_id"] = json!("empty");
    empty["targets"] = json!([]);
    let response = client
        .post(&endpoint)
        .bearer_auth(PRODUCER)
        .json(&json!({"observation":empty}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let records: Value = client
        .get(&endpoint)
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    assert!(records["records"][5]["resolution"].is_null());
    assert!(records["records"][6]["resolution"].is_null());
    sqlx::query("UPDATE assurance_deployment SET f_verification_target_fingerprint=$3 WHERE _record_id IN (SELECT id FROM assurance_records WHERE kind='deployment' AND project=$1 AND record_key=$2)").bind("project").bind(old_deployment.as_str()).bind(fp('8')).execute(&pool).await?;
    let poison = check_target(
        target(now - 4000, now - 3000, "unused", &contract),
        &package,
        "legacy-invalid-verification-target",
    );
    let response = client
        .post(&endpoint)
        .bearer_auth(PRODUCER)
        .json(&json!({"observation":poison}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let current = check_target(
        target(now - 900, now - 700, "unused", &contract),
        &package,
        "current-after-poison",
    );
    let response = client
        .post(&endpoint)
        .bearer_auth(PRODUCER)
        .json(&json!({"observation":current}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let retained: Value = client
        .get(&endpoint)
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    assert!(retained["records"][7]["resolution"].is_null(), "{retained}");
    assert!(
        retained["records"][7]["diagnostics"][0]
            .as_str()
            .unwrap()
            .contains("Verification Target fingerprint differs"),
        "{retained}"
    );
    assert!(
        retained["records"][8]["resolution"]["run_fingerprint"].is_string(),
        "{retained}"
    );
    let current_deployment = retained["records"][8]["resolution"]["deployment"]
        .as_str()
        .unwrap();
    let status = client
        .get(format!("{base}/deployments/{current_deployment}/status"))
        .bearer_auth(VIEWER)
        .send()
        .await?;
    assert_eq!(status.status(), StatusCode::OK, "{}", status.text().await?);
    let second_package = publication_variant(
        json!({"verification_target":"deployment","components":["api"],"contexts":[],"contribution":"counterexample","monitor":null}),
        'f',
    );
    let response = client
        .post(format!("{base}/releases"))
        .bearer_auth(PUBLISHER)
        .json(&second_package)
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let later: i64 =
        sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint-1000")
            .fetch_one(&pool)
            .await?;
    observed["observed_at_ms"] = json!(later - 500);
    observed["release_fingerprint"] = second_package["release"]["fingerprint"].clone();
    observed["rollout_id"] = json!("new-application-release");
    observed["applied_revision"] = json!("f".repeat(40));
    observed["components"][0]["image"] = json!(format!("registry/api@{}", fp('f')));
    observed["components"][0]["instances"] = json!(["new-process"]);
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":4,"observation":observed}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let mut next_check = check_target(
        target(later - 400, later - 300, "unused", &contract),
        &second_package,
        "second-release",
    );
    next_check["targets"][0]["image"] = json!(format!("registry/api@{}", fp('f')));
    next_check["targets"][0]["instance"] = json!("new-process");
    assert_eq!(
        next_check["check"]["definition"],
        check["check"]["definition"]
    );
    let response = client
        .post(&endpoint)
        .bearer_auth(PRODUCER)
        .json(&json!({"observation":next_check}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    observed["observed_at_ms"] = json!(later - 100);
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":5,"observation":observed}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let records: Value = client
        .get(&endpoint)
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    let second_run = records["records"][9]["resolution"]["run_fingerprint"]
        .as_str()
        .unwrap();
    let second_run: Value = sqlx::query_scalar(
        "SELECT assurance_record('run',project,fingerprint) FROM run_bundles WHERE fingerprint=$1",
    )
    .bind(second_run)
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        second_run["verification_target"]["artifacts"][0]["digest"],
        fp('f')
    );
    assert_eq!(
        second_run["plan"]["model_fingerprint"],
        run["plan"]["model_fingerprint"]
    );
    assert_eq!(
        second_run["provenance"]["normalizer"],
        run["provenance"]["normalizer"]
    );
    assert_ne!(
        second_run["verification_target_fingerprint"],
        run["verification_target_fingerprint"]
    );
    let columns: i64 = sqlx::query_scalar("SELECT count(*) FROM information_schema.columns WHERE table_schema='public' AND udt_name IN ('json','jsonb')").fetch_one(&pool).await?;
    assert_eq!(columns, 0);
    migrate(&pool).await?;
    server.abort();
    Ok(())
}
