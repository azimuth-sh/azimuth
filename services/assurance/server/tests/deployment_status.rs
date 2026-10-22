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
const OTHER_PRODUCER: &str = "another-producer-credential-for-context-integration-only";
const VIEWER: &str = "viewer-credential-for-context-integration-only";
include!("support/deployment_evidence.rs");

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
                producers: BTreeMap::from([
                    ("producer".into(), PRODUCER.into()),
                    ("another-producer".into(), OTHER_PRODUCER.into()),
                ]),
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
        .post(format!("{base}/model-revisions"))
        .bearer_auth(OWNER)
        .json(&package["snapshot"]["authority"])
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
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
    let result = ready_status(&client, &base, "deployment").await?;
    assert_eq!(result["requirements"][0]["status"], "unresolved");
    assert_eq!(result["requirements"][0]["requirement"], "restricted");
    assert_eq!(
        client
            .get(format!("{base}/deployments/absent/status"))
            .bearer_auth(VIEWER)
            .send()
            .await?
            .status(),
        StatusCode::NOT_FOUND
    );
    let snapshot = json!({"captured_at_ms":1,"instance":"process-a","report":{"format":"azimuth-execution-context","version":2,"contexts":{"api":{"contract_revision":contract["fingerprint"],"values":{"enabled":true,"revision":1,"mode":"restricted"}}}}});
    let mut capture = json!({"kind":"execution","activity":null,"run":fp('1'),"bundle_fingerprint":fp('2'),"verification_target_fingerprint":fp('3'),"component":"api","instance":"process-a","started_at_ms":1,"finished_at_ms":1,"before":snapshot,"after":snapshot});
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
    bundle.verification_target = VerificationTarget::Source {
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
    for (leaf, leaf_value) in snapshot["report"]["contexts"]["api"]["values"]
        .as_object()
        .unwrap()
    {
        let pin = azimuth::run::canonical_fingerprint(&native(leaf_value)).unwrap();
        let key = format!("context/api/{leaf}");
        bundle
            .plan
            .required_execution_context
            .insert(key.clone(), pin.clone());
        bundle.actual_selection.execution_context.insert(key, pin);
    }
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
    capture["verification_target_fingerprint"] = json!(bundle.verification_target_fingerprint);
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
    let status = ready_status(&client, &base, deployment_id).await?;
    assert_eq!(status["requirements"][0]["status"], "supported", "{status}");
    let mut concurrent_bundle = bundle.clone();
    concurrent_bundle.provenance.source.execution = "native-concurrent-recording".into();
    refresh(&mut concurrent_bundle);
    assert!(azimuth::run::verify(&concurrent_bundle).is_empty());
    let response = client
        .post(format!("{base}/runs"))
        .bearer_auth(OTHER_PRODUCER)
        .json(&value(&azimuth::run::to_json(&concurrent_bundle)))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "{}",
        response.text().await?
    );
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
    capture["verification_target_fingerprint"] =
        deployed["deployment"]["verification_target_fingerprint"].clone();
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
        json!({"verification_target":"deployment","components":["api"],"contexts":[],"contribution":"both","monitor":{"id":"access-monitor","interval_seconds":1,"freshness_seconds":60}}),
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
    poll.verification_target = azimuth::run::verification_target_from_json(&native(
        &deployed["deployment"]["verification_target"],
    ))?;
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
    poll.check_executions[0].results[0].observed_at_ms = poll.finished_at_ms;
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
    sqlx::query("INSERT INTO run_bundles(project,run_id,revision,fingerprint,verification_target_fingerprint,model_fingerprint) SELECT 'project',record_key,0,record_key,$1,$2 FROM assurance_records WHERE project='project' AND kind='run' AND record_key LIKE 'expired-monitor-%'").bind(&poll.verification_target_fingerprint).bind(&poll.plan.model_fingerprint).execute(&pool).await?;
    let result = ready_status(&client, &base, &selected).await?;
    assert_eq!(result["requirements"][0]["status"], "supported", "{result}");
    let columns:Vec<String>=sqlx::query_scalar("SELECT table_name||'.'||column_name FROM information_schema.columns WHERE table_schema='public' AND udt_name IN ('json','jsonb')").fetch_all(&pool).await?;
    assert!(
        columns.is_empty(),
        "unexpected document columns {columns:?}"
    );
    server.abort();
    Ok(())
}

async fn ready_status(
    client: &Client,
    base: &str,
    deployment: &str,
) -> Result<Value, Box<dyn Error>> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(40);
    loop {
        let response = client
            .get(format!("{base}/deployments/{deployment}/status"))
            .bearer_auth(VIEWER)
            .send()
            .await?;
        let code = response.status();
        let value: Value = response.json().await?;
        assert_eq!(code, StatusCode::OK, "{value}");
        if value["assessment"]["state"] == "ready" {
            return Ok(value);
        }
        assert!(
            std::time::Instant::now() < deadline,
            "assessment failed to converge: {value}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

struct Harness {
    _postgres: testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
    pool: sqlx::PgPool,
    client: Client,
    base: String,
    server: tokio::task::JoinHandle<()>,
}

async fn harness() -> Result<Harness, Box<dyn Error>> {
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
    Ok(Harness {
        _postgres: postgres,
        pool,
        client: Client::new(),
        base,
        server,
    })
}

#[tokio::test]
async fn a_context_leaf_that_changed_during_capture_leaves_the_condition_unresolved_in_the_service(
) -> Result<(), Box<dyn Error>> {
    let Harness {
        _postgres,
        pool,
        client,
        base,
        server,
    } = harness().await?;
    let contract = contract();
    let package = publication_policy_with_header(
        json!({"verification_target":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}),
        "routine",
        "\nContext: api: enabled = true",
    );
    for (url, token, body) in [
        (
            format!("{base}/context-contracts"),
            OWNER,
            json!({"contract":contract}),
        ),
        (
            format!("{base}/model-revisions"),
            OWNER,
            package["snapshot"]["authority"].clone(),
        ),
        (format!("{base}/releases"), PUBLISHER, package.clone()),
    ] {
        let response = client.post(&url).bearer_auth(token).json(&body).send().await?;
        assert_eq!(response.status(), StatusCode::CREATED, "{url}: {}", response.text().await?);
    }
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
    assert_eq!(response.status(), StatusCode::OK, "{}", response.text().await?);
    let observed: Value = client
        .get(format!("{base}/environments/production/observations"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    let deployment_id = observed["deployment_id"].as_str().unwrap().to_string();
    let deployed: Value = client
        .get(format!("{base}/deployments/{deployment_id}"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    let snapshot = |time: i64, enabled: bool| {
        json!({"captured_at_ms":time,"instance":"process-a","report":{"format":"azimuth-execution-context","version":2,"contexts":{"api":{"contract_revision":contract["fingerprint"],"values":{"enabled":enabled,"revision":1,"mode":"restricted"}}}}})
    };
    let capture = |start: i64, before: bool, after: bool| {
        json!({"kind":"deployment","activity":null,"run":null,"bundle_fingerprint":null,"verification_target_fingerprint":deployed["deployment"]["verification_target_fingerprint"],"component":"api","instance":"process-a","started_at_ms":start,"finished_at_ms":start + 1,"before":snapshot(start, before),"after":snapshot(start + 1, after)})
    };

    let response = client
        .post(format!("{base}/context-captures"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"capture":capture(at + 1, true, false)}))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::CREATED, "{}", response.text().await?);
    let unstable = ready_status(&client, &base, &deployment_id).await?;
    assert_eq!(unstable["requirements"][0]["status"], "unresolved", "{unstable}");

    let response = client
        .post(format!("{base}/context-captures"))
        .bearer_auth(PUBLISHER)
        .json(&json!({"capture":capture(at + 3, false, false)}))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::CREATED, "{}", response.text().await?);
    let stable = ready_status(&client, &base, &deployment_id).await?;
    assert_eq!(stable["requirements"][0]["status"], "not-applicable", "{stable}");
    server.abort();
    Ok(())
}

fn main_publication(commit: &str, ratification: Value) -> Value {
    let package = publication();
    json!({"format":"azimuth-main-model-publication","version":1,"commit":commit,"main_sequence":1,"authority":package["snapshot"]["authority"],"reviews":[],"ratification":ratification})
}

#[tokio::test]
async fn main_publication_accepts_exactly_the_contract_commit_and_ratification_shapes(
) -> Result<(), Box<dyn Error>> {
    let Harness {
        _postgres,
        client,
        base,
        server,
        ..
    } = harness().await?;
    let ratified = json!({"status":"ratified","reason":"approved","pull_request":{"number":1,"approvals":[]},"verified_at_ms":0});
    let publish = |body: Value| {
        let client = client.clone();
        let url = format!("{base}/main-models");
        async move {
            let response = client.post(url).bearer_auth(PUBLISHER).json(&body).send().await.unwrap();
            let status = response.status();
            (status, response.text().await.unwrap())
        }
    };
    for commit in ["A".repeat(40), "a".repeat(39), "a".repeat(41), "g".repeat(40), "a".repeat(63)] {
        let (status, body) = publish(main_publication(&commit, ratified.clone())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{commit}: {body}");
        assert!(body.contains("40 or 64 lowercase hex"), "{body}");
    }
    let unreasoned = json!({"status":"unratified","reason":"  ","pull_request":null,"verified_at_ms":1});
    let (status, body) = publish(main_publication(&"a".repeat(40), unreasoned)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.contains("nonempty reason"), "{body}");
    let negative = json!({"status":"ratified","reason":"approved","pull_request":null,"verified_at_ms":-1});
    let (status, body) = publish(main_publication(&"a".repeat(40), negative)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.contains("non-negative"), "{body}");

    let (status, body) = publish(main_publication(&"a".repeat(64), ratified.clone())).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let reasoned = json!({"status":"unratified","reason":"no owner approval","pull_request":null,"verified_at_ms":0});
    let mut second = main_publication(&"b".repeat(40), reasoned);
    second["main_sequence"] = json!(2);
    let (status, body) = publish(second).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    server.abort();
    Ok(())
}
