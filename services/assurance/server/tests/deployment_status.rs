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
    let mut result = json!({"name":"settings","fields":{"enabled":"boolean","revision":"integer","mode":"string"}});
    result["fingerprint"] = json!(azimuth::run::canonical_fingerprint(&native(&result)).unwrap());
    result
}
fn publication() -> Value {
    publication_policy(
        json!({"subject":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}),
    )
}
fn publication_policy(policy: Value) -> Value {
    let deployment_id = if policy["monitor"].is_null() {
        "deployment"
    } else {
        "monitor-deployment"
    };
    let model=azimuth::model::Model {
        specs:vec![azimuth::spec::parse_spec("spec.md","# Spec: sample\n\n## Claim: restricted\nCriticality: routine\n\nAccess SHALL be restricted.\n\n### Case: works\nEvent: request\nRequired: restricted access\n").unwrap()],
        account_verifications:vec![azimuth::account_verification::parse_account_verification("verification.md",&format!("# Verification: sample\n\n## Claim verification: restricted\n\n### Check: recovery\n- Deployment: {policy}\n- Evidence bindings:\n  - Case: `works`\n    - Contribution: Establishes access restrictions.\n\nExercise access restrictions.\n")).unwrap()],
        ..Default::default()
    };
    let authority = value(&azimuth::assurance_review::authority(&model, "project").unwrap());
    let subject = json!({"kind":"deployment","environment":"production","deployment":deployment_id,"deployment_fingerprint":fp('c'),"artifacts":[{"id":"api","digest":fp('e')}]});
    let subject_fp = azimuth::run::subject_fingerprint(
        &azimuth::run::subject_from_json(&native(&subject)).unwrap(),
    );
    let deployment = json!({"id":deployment_id,"environment":"production","configuration_fingerprint":fp('c'),"artifacts":[fp('e')],"subject":subject,"subject_fingerprint":subject_fp});
    value(&azimuth::release::prepare(&native(&json!({
        "format":"azimuth-release-request","version":1,"project":"project","label":"Synthetic release",
        "baseline_model_revision":fp('a'),"changes":[{"id":"restricted","revision":fp('a')}],
        "sources":[{"repository":"root","commit":"a".repeat(40),"content_fingerprint":fp('a')}],
        "builds":[{"id":"build-api","sources":["root"],"toolchain":{"compiler":"1"},"configuration_fingerprint":fp('b'),"artifacts":[fp('e')]}],
        "artifacts":[{"digest":fp('e'),"kind":"container"}],"deployments":[deployment],"components":[{"id":"api","digest":fp('e')}],
        "snapshot":{"project":"project","model_revision":authority["fingerprint"],"authority":authority,"reviews":[],"bundles":[]},"evidence":[]
    }))).unwrap())
}

#[tokio::test]
async fn context_contracts_and_capture_authorization_preserve_relational_storage(
) -> Result<(), Box<dyn Error>> {
    let postgres = Postgres::default().with_tag("17-alpine").start().await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let pool = connect(&format!(
        "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
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
                    "reviewer-credential-for-context-integration-only".into(),
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
    let url = format!("{base}/context-contracts");
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(PUBLISHER)
            .json(&json!({"contract":contract}))
            .send()
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    for (expected, replayed) in [(StatusCode::CREATED, false), (StatusCode::OK, true)] {
        let response = client
            .post(&url)
            .bearer_auth(OWNER)
            .json(&json!({"contract":contract}))
            .send()
            .await?;
        assert_eq!(response.status(), expected, "{}", response.text().await?);
        let retained: Vec<Value> = client
            .get(&url)
            .bearer_auth(VIEWER)
            .send()
            .await?
            .json::<Value>()
            .await?["records"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(retained.len(), 1);
        assert_eq!(retained[0]["contract"], contract);
        assert_eq!(retained[0]["registered_by"], "owner");
        let _ = replayed;
    }
    let mut malformed = contract.clone();
    malformed["fields"]["accessToken"] = json!("string");
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(OWNER)
            .json(&json!({"contract":malformed}))
            .send()
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let package = publication();
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
    let response = client
        .get(format!("{base}/deployments/deployment/status"))
        .bearer_auth(VIEWER)
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let result: Value = client
        .get(format!("{base}/deployments/deployment/status"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(result["claims"][0]["status"], "unresolved");
    assert_eq!(result["claims"][0]["claim"], "restricted");
    assert_eq!(
        client
            .get(format!("{base}/deployments/absent/status"))
            .bearer_auth(VIEWER)
            .send()
            .await?
            .status(),
        StatusCode::NOT_FOUND
    );
    let snapshot = json!({"captured_at_ms":1,"instance":"process-a","report":{"format":"azimuth-verification-context","version":1,"contexts":{"settings":{"contract_revision":contract["fingerprint"],"values":{"enabled":true,"revision":1,"mode":"restricted"}}}}});
    let mut capture = json!({"kind":"execution","activity":null,"run":fp('1'),"bundle_fingerprint":fp('2'),"subject_fingerprint":fp('3'),"component":"api","instance":"process-a","started_at_ms":1,"finished_at_ms":1,"before":snapshot,"after":snapshot});
    assert_eq!(
        client
            .post(format!("{base}/context-captures"))
            .bearer_auth(VIEWER)
            .json(&json!({"capture":capture}))
            .send()
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        client
            .post(format!("{base}/context-captures"))
            .bearer_auth(PRODUCER)
            .json(&json!({"capture":capture}))
            .send()
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    capture["trusted"] = json!(true);
    assert_eq!(
        client
            .post(format!("{base}/context-captures"))
            .bearer_auth(PRODUCER)
            .json(&json!({"capture":capture}))
            .send()
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut bundle = valid_bundle();
    let definition = package["snapshot"]["authority"]["checks"][0]["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();
    bundle.subject = Subject::Source {
        repositories: vec![RepositoryState {
            id: "root".into(),
            revision: "a".repeat(40),
            content_fingerprint: fp('a'),
        }],
    };
    bundle.plan.model_fingerprint = fp('f');
    bundle.plan.checks[0].fingerprint = definition.clone();
    bundle.actual_selection.checks[0].fingerprint = definition.clone();
    bundle.check_executions[0].check.fingerprint = definition;
    let context_fp =
        azimuth::run::canonical_fingerprint(&native(&snapshot["report"]["contexts"]["settings"]))
            .unwrap();
    let context_key = format!("context/{}", contract["fingerprint"].as_str().unwrap());
    bundle
        .plan
        .required_context
        .insert(context_key.clone(), context_fp.clone());
    bundle
        .actual_selection
        .context
        .insert(context_key, context_fp);
    refresh(&mut bundle);
    assert!(azimuth::run::verify(&bundle).is_empty());
    let response = client
        .post(format!("{base}/runs"))
        .bearer_auth(PRODUCER)
        .json(&value(&azimuth::run::to_json(&bundle)))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    capture.as_object_mut().unwrap().remove("trusted");
    capture["run"] = json!(bundle.run_id);
    capture["bundle_fingerprint"] = json!(bundle.bundle_fingerprint);
    capture["subject_fingerprint"] = json!(bundle.subject_fingerprint);
    capture["finished_at_ms"] = json!(5);
    capture["after"]["captured_at_ms"] = json!(5);
    let response = client
        .post(format!("{base}/context-captures"))
        .bearer_auth(PRODUCER)
        .json(&json!({"capture":capture}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    assert_eq!(
        client
            .post(format!("{base}/context-captures"))
            .bearer_auth(PRODUCER)
            .json(&json!({"capture":capture}))
            .send()
            .await?
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .post(format!("{base}/context-captures"))
            .bearer_auth(PUBLISHER)
            .json(&json!({"capture":capture}))
            .send()
            .await?
            .status(),
        StatusCode::CONFLICT
    );
    let at: i64 = sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
        .fetch_one(&pool)
        .await?;
    let observation = json!({"format":"azimuth-deployment-observation","version":1,"observed_at_ms":at,"valid_for_seconds":60,"state":"complete","release_fingerprint":package["release"]["fingerprint"],"applied_revision":"a".repeat(40),"rollout_id":"rollout-1","configuration_fingerprint":fp('c'),"components":[{"name":"api","image":format!("registry/api@{}",fp('e')),"template_fingerprint":fp('c'),"ready":true,"instances":["process-a"]}],"diagnostics":[]});
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":0,"observation":observation}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let observed: Value = client
        .get(format!("{base}/environments/production/observations"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    let deployment_id = observed["deployment_id"].as_str().unwrap();
    let response = client
        .get(format!("{base}/deployments/{deployment_id}/status"))
        .bearer_auth(VIEWER)
        .send()
        .await?;
    let code = response.status();
    let status: Value = response.json().await?;
    assert_eq!(code, StatusCode::OK, "{status}");
    assert_eq!(status["claims"][0]["status"], "supported", "{status}");
    let deployed: Value = client
        .get(format!("{base}/deployments/{deployment_id}"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    capture["kind"] = json!("deployment");
    capture["run"] = Value::Null;
    capture["bundle_fingerprint"] = Value::Null;
    capture["subject_fingerprint"] = deployed["deployment"]["subject_fingerprint"].clone();
    capture["started_at_ms"] = json!(at + 1);
    capture["before"]["captured_at_ms"] = json!(at + 1);
    capture["finished_at_ms"] = json!(at + 2);
    capture["after"]["captured_at_ms"] = json!(at + 2);
    let response = client
        .post(format!("{base}/context-captures"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"capture":capture}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    capture["instance"] = json!("unobserved-process");
    capture["before"]["instance"] = json!("unobserved-process");
    capture["after"]["instance"] = json!("unobserved-process");
    assert_eq!(
        client
            .post(format!("{base}/context-captures"))
            .bearer_auth(PUBLISHER)
            .json(&json!({"capture":capture}))
            .send()
            .await?
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM assurance_records WHERE kind='context_capture'")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 2);
    let monitored = publication_policy(
        json!({"subject":"deployment","components":["api"],"contexts":[],"contribution":"both","monitor":{"id":"access-monitor","interval_seconds":1,"freshness_seconds":60}}),
    );
    let response = client
        .post(format!("{base}/releases"))
        .bearer_auth(PUBLISHER)
        .json(&monitored)
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    let current: i64 =
        sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
            .fetch_one(&pool)
            .await?;
    let mut next = observation.clone();
    next["observed_at_ms"] = json!(current);
    next["release_fingerprint"] = monitored["release"]["fingerprint"].clone();
    next["rollout_id"] = json!("rollout-2");
    let response = client
        .post(format!("{base}/environments/production/observations"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"expected_revision":1,"observation":next}))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{}",
        response.text().await?
    );
    let observed: Value = client
        .get(format!("{base}/environments/production/observations"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    let selected = observed["deployment_id"].as_str().unwrap();
    let deployed: Value = client
        .get(format!("{base}/deployments/{selected}"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    let mut poll = valid_bundle();
    poll.subject = azimuth::run::subject_from_json(&native(&deployed["deployment"]["subject"]))?;
    poll.plan.model_fingerprint = fp('f');
    let definition = monitored["snapshot"]["authority"]["checks"][0]["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();
    poll.plan.checks[0].fingerprint = definition.clone();
    poll.actual_selection.checks[0].fingerprint = definition.clone();
    poll.check_executions[0].check.fingerprint = definition;
    poll.planned_at_ms = (current - 3) as u64;
    poll.started_at_ms = (current - 2) as u64;
    poll.finished_at_ms = (current - 1) as u64;
    poll.provenance.generated_at_ms = current as u64;
    poll.activities[0].started_at_ms = poll.started_at_ms;
    poll.activities[0].finished_at_ms = poll.finished_at_ms;
    poll.check_executions[0].observations[0].observed_at_ms = poll.finished_at_ms;
    refresh(&mut poll);
    assert!(azimuth::run::verify(&poll).is_empty());
    let response = client
        .post(format!("{base}/runs"))
        .bearer_auth(PRODUCER)
        .json(&value(&azimuth::run::to_json(&poll)))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
    // Expired metadata must be excluded before document hydration; only the fresh Run supplies evidence.
    sqlx::query("INSERT INTO assurance_records(kind,project,record_key) SELECT 'run','project','expired-monitor-'||g FROM generate_series(1,5001) g").execute(&pool).await?;
    for table in [
        "assurance_run",
        "assurance_run_plan",
        "assurance_run_plan_checks",
        "assurance_run_plan_checks_item",
    ] {
        let columns:Vec<String>=sqlx::query_scalar("SELECT column_name FROM information_schema.columns WHERE table_schema='public' AND table_name=$1 ORDER BY ordinal_position").bind(table).fetch_all(&pool).await?;
        let selections = columns
            .iter()
            .map(|column| match column.as_str() {
                "_record_id" => "a.id".to_string(),
                "f_finished_at_ms" | "f_started_at_ms" | "f_planned_at_ms" => "0".to_string(),
                _ => format!("source.\"{column}\""),
            })
            .collect::<Vec<_>>()
            .join(",");
        let names = columns
            .iter()
            .map(|column| format!("\"{column}\""))
            .collect::<Vec<_>>()
            .join(",");
        sqlx::query(&format!("INSERT INTO {table}({names}) SELECT {selections} FROM assurance_records a CROSS JOIN {table} source JOIN assurance_records template ON template.id=source._record_id WHERE a.project='project' AND a.kind='run' AND a.record_key LIKE 'expired-monitor-%' AND template.project='project' AND template.kind='run' AND template.record_key=$1")).bind(&poll.bundle_fingerprint).execute(&pool).await?;
    }
    sqlx::query("INSERT INTO run_bundles(project,run_id,revision,fingerprint,subject_fingerprint,model_fingerprint) SELECT 'project',record_key,0,record_key,$1,$2 FROM assurance_records WHERE project='project' AND kind='run' AND record_key LIKE 'expired-monitor-%'").bind(&poll.subject_fingerprint).bind(&poll.plan.model_fingerprint).execute(&pool).await?;
    let response = client
        .get(format!("{base}/deployments/{selected}/status"))
        .bearer_auth(VIEWER)
        .send()
        .await?;
    let code = response.status();
    let result: Value = response.json().await?;
    assert_eq!(code, StatusCode::OK, "{result}");
    assert_eq!(result["claims"][0]["status"], "supported", "{result}");
    let columns:Vec<String>=sqlx::query_scalar("SELECT table_name||'.'||column_name FROM information_schema.columns WHERE table_schema='public' AND udt_name IN ('json','jsonb')").fetch_all(&pool).await?;
    assert!(
        columns.is_empty(),
        "unexpected document columns {columns:?}"
    );
    server.abort();
    Ok(())
}

fn context() -> BTreeMap<String, String> {
    BTreeMap::new()
}
fn valid_bundle() -> RunBundle {
    let check = CheckSelection {
        id: "recovery".into(),
        fingerprint: fp('b'),
        cases: vec!["works".into()],
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
        subject: Subject::Workspace {
            repositories: vec![RepositoryState {
                id: "root".into(),
                revision: "revision-a".into(),
                content_fingerprint: fp('a'),
            }],
        },
        subject_fingerprint: fp('0'),
        planned_at_ms: 1,
        started_at_ms: 2,
        finished_at_ms: 4,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: fp('d'),
            required_context: context(),
            checks: vec![check.clone()],
            challenges: vec![],
            fingerprint: fp('0'),
        },
        actual_selection: ActualSelection {
            context: context(),
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
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "check-attempt".into(),
                    outcomes: [("works".into(), ObservationOutcome::Satisfied)]
                        .into_iter()
                        .collect(),
                }],
            }],
            observations: vec![Observation {
                case: "works".into(),
                outcome: ObservationOutcome::Satisfied,
                observed_at_ms: 3,
                fingerprint: fp('0'),
                artifacts: vec![],
                diagnostics: vec![],
            }],
        }],
        challenger_executions: vec![],
    };
    refresh(&mut bundle);
    bundle
}
fn refresh(bundle: &mut RunBundle) {
    bundle.subject_fingerprint = subject_fingerprint(&bundle.subject);
    bundle.plan.fingerprint = plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    bundle.provenance.adapter.launch_fingerprint = launch_fingerprint(
        bundle.provenance.mode,
        bundle.planned_at_ms,
        &bundle.subject,
        &bundle.subject_fingerprint,
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
        for observation_index in 0..bundle.check_executions[index].observations.len() {
            let fingerprint = observation_fingerprint(
                bundle,
                &bundle.check_executions[index],
                &bundle.check_executions[index].observations[observation_index],
            );
            bundle.check_executions[index].observations[observation_index].fingerprint =
                fingerprint;
        }
    }
    for index in 0..bundle.challenger_executions.len() {
        bundle.challenger_executions[index].result.fingerprint =
            challenge_result_fingerprint(bundle, &bundle.challenger_executions[index]);
    }
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}
