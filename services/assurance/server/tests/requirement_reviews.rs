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

const OWNER: &str = "owner-credential-for-review-integration-only";
const PUBLISHER: &str = "publisher-credential-for-review-integration-only";
const REVIEWER: &str = "reviewer-credential-for-review-integration-only";
const VIEWER: &str = "viewer-credential-for-review-integration-only";

fn digest(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn native(value: &Value) -> azimuth::json::Json {
    azimuth::json::parse(&value.to_string()).unwrap()
}

fn value(json: &azimuth::json::Json) -> Value {
    serde_json::from_str(&azimuth::run::canonical_json(json).unwrap()).unwrap()
}

fn fingerprinted(mut record: Value) -> Value {
    record["fingerprint"] =
        json!(azimuth::assurance_review::record_fingerprint(&native(&record)).unwrap());
    record
}

fn authority() -> Value {
    let model = azimuth::model::Model {
        specs: vec![azimuth::spec::parse_spec(
            "spec.md",
            "# Spec: module\n\n## Requirement: bounded\nCriticality: routine\n\nThe operation SHALL complete.\n\n### Scenario: completes\nGiven ordinary input, when invoked, then complete.\n",
        )
        .unwrap()],
        ..azimuth::model::Model::default()
    };
    value(&azimuth::assurance_review::authority(&model, "project").unwrap())
}

fn requirement_review(authority: &Value, reassesses: Option<&str>, reviewed_at_ms: i64) -> Value {
    fingerprinted(json!({
        "kind": "requirement-review", "reviewer": "reviewer", "reviewed_at_ms": reviewed_at_ms,
        "rationale": "Reviewed the exact declared account.",
        "artifacts": [{"id":"notes","digest":digest('a'),"locator":"reviews/notes.md"}],
        "requirement": "bounded",
        "requirement_fingerprint": authority["requirements"][0]["fingerprint"],
        "dependencies_fingerprint": azimuth::assurance_review::requirement_closure_fingerprint(&native(authority), "bounded").unwrap(),
        "decision": "accepted", "reassesses": reassesses, "challenge_basis": []
    }))
}

fn requirement_status() -> Value {
    json!({
        "kind":"requirement-status","reviewer":"reviewer","reviewed_at_ms":100,
        "rationale":"Status.","artifacts":[],"requirement":"bounded",
        "requirement_fingerprint":digest('b'),"verification_target_fingerprint":digest('c'),
        "requirement_review_fingerprint":digest('d'),"basis":[],"conclusion":"unresolved",
        "residual_risks":[],"challenge_basis":[],"fingerprint":digest('e')
    })
}

async fn post(
    client: &Client,
    url: &str,
    credential: &str,
    body: &Value,
) -> Result<(StatusCode, Value), Box<dyn Error>> {
    let response = client
        .post(url)
        .bearer_auth(credential)
        .json(body)
        .send()
        .await?;
    Ok((response.status(), response.json().await?))
}

#[tokio::test]
async fn only_requirement_reviews_are_ingested_and_stored_without_json_text(
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
                producers: BTreeMap::from([(
                    "producer".into(),
                    "producer-credential-for-review-integration-only".into(),
                )]),
                reviewers: BTreeMap::from([("reviewer".into(), REVIEWER.into())]),
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
    let authority = authority();
    let revision = authority["fingerprint"].as_str().unwrap();
    let (status, body) = post(
        &client,
        &format!("{base}/model-revisions"),
        OWNER,
        &authority,
    )
    .await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    let reviews = format!("{base}/model-revisions/{revision}/reviews");
    let original = requirement_review(&authority, None, 100);
    let (status, body) = post(&client, &reviews, REVIEWER, &original).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["fingerprint"], original["fingerprint"]);
    assert_eq!(body["assessment"], "accepted", "{body}");
    let (status, body) = post(&client, &reviews, REVIEWER, &original).await?;
    assert_eq!(
        (status, body["replayed"].clone()),
        (StatusCode::OK, json!(true))
    );
    let successor = requirement_review(&authority, original["fingerprint"].as_str(), 200);
    let (status, body) = post(&client, &reviews, REVIEWER, &successor).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(
        post(&client, &reviews, VIEWER, &successor).await?.0,
        StatusCode::FORBIDDEN
    );

    let snapshot: Value = client
        .get(format!("{base}/model-revisions/{revision}/snapshot"))
        .bearer_auth(VIEWER)
        .send()
        .await?
        .json()
        .await?;
    let mut stored: Vec<Value> = snapshot["reviews"].as_array().unwrap().clone();
    stored.sort_by_key(|record| record["reviewed_at_ms"].as_i64());
    assert_eq!(stored, vec![original.clone(), successor.clone()]);

    let mut historical = Vec::new();
    for kind in [
        "requirement-status",
        "method-qualification",
        "applicability-decision",
        "requirement-judgment",
    ] {
        let mut record = requirement_status();
        record["kind"] = json!(kind);
        historical.push(record);
    }
    for record in &historical {
        let (status, body) = post(&client, &reviews, REVIEWER, record).await?;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        let detail = body["detail"].as_str().unwrap();
        assert!(
            detail.contains(record["kind"].as_str().unwrap())
                && detail.contains("stores only requirement-review records"),
            "{detail}"
        );
    }
    let publication = json!({
        "format":"azimuth-main-model-publication","version":2,"commit":"a".repeat(40),
        "main_sequence":1,"publication_revision":1,"sources":{"root":"b".repeat(40)},
        "authority":authority,"reviews":[requirement_status()],
        "ratification":{"status":"ratified","verified_at_ms":0}
    });
    let (status, body) = post(
        &client,
        &format!("{base}/main-models"),
        PUBLISHER,
        &publication,
    )
    .await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["detail"]
        .as_str()
        .unwrap()
        .contains("requirement-status records are not accepted"));
    let release = json!({"snapshot":{"reviews":[requirement_status()]}});
    let (status, body) = post(&client, &format!("{base}/releases"), PUBLISHER, &release).await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["detail"]
        .as_str()
        .unwrap()
        .contains("requirement-status records are not accepted"));

    let stored_reviews: i64 = sqlx::query_scalar("SELECT count(*) FROM assurance_reviews")
        .fetch_one(&pool)
        .await?;
    assert_eq!(stored_reviews, 2, "rejected kinds leave no review rows");
    let document_columns: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text||'.'||column_name::text FROM information_schema.columns WHERE table_schema='public' AND table_name LIKE 'assurance_review%' AND column_name LIKE 'f\\_%' AND column_name NOT IN ('f_kind','f_reviewer','f_reviewed_at_ms','f_rationale','f_requirement','f_requirement_fingerprint','f_dependencies_fingerprint','f_decision','f_reassesses','f_fingerprint','f_id','f_digest','f_locator','f_run','f_bundle_fingerprint','f_result_fingerprint','f_target_fingerprint')",
    )
    .fetch_all(&pool)
    .await?;
    assert_eq!(document_columns, Vec::<String>::new());
    server.abort();
    Ok(())
}
