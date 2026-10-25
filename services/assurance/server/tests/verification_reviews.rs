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
            "# Spec: module\n\n## Requirement: bounded\nRealization: reviewed\nVerification: reviewed\n\nThe operation SHALL complete.\n\n### Scenario: completes\nGiven ordinary input, when invoked, then complete.\n",
        )
        .unwrap()],
        ..azimuth::model::Model::default()
    };
    value(&azimuth::assurance_review::authority(&model, "project").unwrap())
}

fn verification_review(authority: &Value, reassesses: Option<&str>, reviewed_at_ms: i64) -> Value {
    fingerprinted(json!({
        "kind": "verification-review", "reviewer": "reviewer", "reviewed_at_ms": reviewed_at_ms,
        "rationale": "Reviewed the exact declared account.",
        "artifacts": [{"id":"notes","digest":digest('a'),"locator":"reviews/notes.md"}],
        "requirement": "bounded",
        "requirement_fingerprint": authority["requirements"][0]["fingerprint"],
        "dependencies_fingerprint": azimuth::assurance_review::requirement_closure_fingerprint(&native(authority), "bounded").unwrap(),
        "decision": "accepted", "reassesses": reassesses, "challenge_basis": []
    }))
}

fn realization_review(authority: &Value, sites_fingerprint: &str) -> Value {
    json!({
        "kind": "realization-review", "reviewer": "reviewer", "reviewed_at_ms": 300,
        "rationale": "The realization sites do what the Scenarios say.",
        "artifacts": [{"id":"notes","digest":digest('a'),"locator":"reviews/notes.md"}],
        "requirement": "bounded",
        "requirement_fingerprint": authority["requirement_intents"][0]["fingerprint"],
        "dependencies_fingerprint": azimuth::fingerprint::canonical_sha256(
            &azimuth::assurance_review::realization_dependencies(&native(authority), "bounded").unwrap(),
        ),
        "sites_fingerprint": sites_fingerprint,
        "reviewed_commit": "c".repeat(40),
        "decision": "accepted", "reassesses": null
    })
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

struct Service {
    _postgres: testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
    pool: sqlx::PgPool,
    base: String,
    client: Client,
    server: tokio::task::JoinHandle<()>,
}

async fn service() -> Result<Service, Box<dyn Error>> {
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
    Ok(Service {
        _postgres: postgres,
        pool,
        base,
        client: Client::new(),
        server,
    })
}

async fn registered(service: &Service) -> Result<Value, Box<dyn Error>> {
    let authority = authority();
    let (status, body) = post(
        &service.client,
        &format!("{}/model-revisions", service.base),
        OWNER,
        &authority,
    )
    .await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    Ok(authority)
}

async fn definition_reviews(service: &Service, revision: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!(
        "{}/model-revisions/{revision}/requirement-reviews",
        service.base
    );
    for _ in 0..400 {
        let reviews: Value = service
            .client
            .get(&url)
            .bearer_auth(VIEWER)
            .send()
            .await?
            .json()
            .await?;
        if reviews["assessment"]["state"] == "ready" {
            return Ok(reviews);
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    Err("definition review projection never became ready".into())
}

#[tokio::test]
async fn only_global_reviews_are_ingested_and_stored_without_json_text(
) -> Result<(), Box<dyn Error>> {
    let service = service().await?;
    let Service {
        pool,
        base,
        client,
        server,
        ..
    } = &service;
    let authority = registered(&service).await?;
    let revision = authority["fingerprint"].as_str().unwrap();

    let reviews = format!("{base}/model-revisions/{revision}/reviews");
    let original = verification_review(&authority, None, 100);
    let (status, body) = post(client, &reviews, REVIEWER, &original).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["fingerprint"], original["fingerprint"]);
    assert_eq!(body["assessment"], "accepted", "{body}");
    let (status, body) = post(client, &reviews, REVIEWER, &original).await?;
    assert_eq!(
        (status, body["replayed"].clone()),
        (StatusCode::OK, json!(true))
    );
    let successor = verification_review(&authority, original["fingerprint"].as_str(), 200);
    let (status, body) = post(client, &reviews, REVIEWER, &successor).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(
        post(client, &reviews, VIEWER, &successor).await?.0,
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
        let (status, body) = post(client, &reviews, REVIEWER, record).await?;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        let detail = body["detail"].as_str().unwrap();
        assert!(
            detail.contains(record["kind"].as_str().unwrap())
                && detail
                    .contains("stores only verification-review and realization-review records"),
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
        client,
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
    let (status, body) = post(client, &format!("{base}/releases"), PUBLISHER, &release).await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["detail"]
        .as_str()
        .unwrap()
        .contains("requirement-status records are not accepted"));

    let stored_reviews: i64 = sqlx::query_scalar("SELECT count(*) FROM assurance_reviews")
        .fetch_one(pool)
        .await?;
    assert_eq!(stored_reviews, 2, "rejected kinds leave no review rows");
    let document_columns: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text||'.'||column_name::text FROM information_schema.columns WHERE table_schema='public' AND table_name LIKE 'assurance_review%' AND column_name LIKE 'f\\_%' AND column_name NOT IN ('f_kind','f_reviewer','f_reviewed_at_ms','f_rationale','f_requirement','f_requirement_fingerprint','f_dependencies_fingerprint','f_decision','f_reassesses','f_fingerprint','f_id','f_digest','f_locator','f_run','f_bundle_fingerprint','f_result_fingerprint','f_target_fingerprint','f_sites_fingerprint','f_reviewed_commit','f_proposition','f_location','f_reason','f_correction','f_blocking','f_issue','f_outcome')",
    )
    .fetch_all(pool)
    .await?;
    assert_eq!(document_columns, Vec::<String>::new());
    server.abort();
    Ok(())
}

#[tokio::test]
async fn realization_reviews_are_assessed_per_kind_with_code_drift_as_information(
) -> Result<(), Box<dyn Error>> {
    let service = service().await?;
    let authority = registered(&service).await?;
    let revision = authority["fingerprint"].as_str().unwrap();
    let reviews = format!("{}/model-revisions/{revision}/reviews", service.base);
    let current_sites = authority["requirement_realizations"][0]["sites_fingerprint"]
        .as_str()
        .unwrap();

    let verification = verification_review(&authority, None, 100);
    let (status, body) = post(&service.client, &reviews, REVIEWER, &verification).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let mut rejected = realization_review(&authority, current_sites);
    rejected["decision"] = json!("rejected");
    rejected["issues"] = json!([{"id":"unguarded-retry","proposition":"Retries stop after the bound.",
        "location":"src/retry.rs","reason":"The loop has no bound.","correction":"Bound the loop.","blocking":true}]);
    let rejected = fingerprinted(rejected);
    let (status, body) = post(&service.client, &reviews, REVIEWER, &rejected).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["assessment"], "rejected", "{body}");

    let projection = definition_reviews(&service, revision).await?;
    let entries = projection["requirements"].as_array().unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|entry| (
                entry["kind"].as_str().unwrap(),
                entry["assessment"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("verification-review", "accepted"),
            ("realization-review", "rejected")
        ],
        "{projection}"
    );
    assert_eq!(
        entries[1]["record"], rejected,
        "stored issues round-trip unchanged"
    );
    assert_eq!(entries[1]["code_drift"], false, "{projection}");
    assert_eq!(entries[0]["code_drift"], Value::Null, "{projection}");

    // The successor looked at different code than the current sites, which is drift, not staleness.
    let mut successor = realization_review(&authority, &digest('f'));
    successor["reviewed_at_ms"] = json!(400);
    successor["reassesses"] = rejected["fingerprint"].clone();
    successor["dispositions"] = json!([{"issue":format!("{}#unguarded-retry", rejected["fingerprint"].as_str().unwrap()),
        "outcome":"corrected","rationale":"The loop is bounded by the configured attempts."}]);
    let successor = fingerprinted(successor);
    let (status, body) = post(&service.client, &reviews, REVIEWER, &successor).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["assessment"], "accepted", "{body}");
    let projection = definition_reviews(&service, revision).await?;
    let realization = &projection["requirements"][1];
    assert_eq!(realization["kind"], "realization-review", "{projection}");
    assert_eq!(realization["assessment"], "accepted", "{projection}");
    assert_eq!(realization["code_drift"], true, "{projection}");
    assert_eq!(realization["record"], successor);

    let mut stale = realization_review(&authority, current_sites);
    stale["reviewed_at_ms"] = json!(500);
    stale["reassesses"] = successor["fingerprint"].clone();
    stale["dependencies_fingerprint"] = json!(digest('9'));
    let stale = fingerprinted(stale);
    let (status, body) = post(&service.client, &reviews, REVIEWER, &stale).await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["assessment"], "stale", "{body}");
    let projection = definition_reviews(&service, revision).await?;
    assert_eq!(
        projection["requirements"][1]["assessment"], "stale",
        "{projection}"
    );
    assert_eq!(
        projection["requirements"][0]["assessment"], "accepted",
        "a Realization Review never changes the Verification Review dimension: {projection}"
    );
    service.server.abort();
    Ok(())
}

#[tokio::test]
async fn malformed_realization_reviews_are_rejected_without_storage() -> Result<(), Box<dyn Error>>
{
    let service = service().await?;
    let authority = registered(&service).await?;
    let revision = authority["fingerprint"].as_str().unwrap();
    let reviews = format!("{}/model-revisions/{revision}/reviews", service.base);
    let sites = authority["requirement_realizations"][0]["sites_fingerprint"]
        .as_str()
        .unwrap();
    let mut without_commit = realization_review(&authority, sites);
    without_commit
        .as_object_mut()
        .unwrap()
        .remove("reviewed_commit");
    let mut with_challenge_basis = realization_review(&authority, sites);
    with_challenge_basis["challenge_basis"] = json!([]);
    let mut with_provenance = realization_review(&authority, sites);
    with_provenance["provenance"] = json!({"agent":null,"model":null,"reassesses":null});
    let mut accepted_with_blocking_issue = realization_review(&authority, sites);
    accepted_with_blocking_issue["issues"] = json!([{"id":"open","proposition":"P.","location":"L.",
        "reason":"R.","correction":"C.","blocking":true}]);
    for (record, expected) in [
        (
            without_commit,
            "realization-review requires reviewed_commit",
        ),
        (with_challenge_basis, "challenge_basis"),
        (with_provenance, "A review carries no provenance"),
        (
            accepted_with_blocking_issue,
            "accepted review cannot retain a blocking issue",
        ),
    ] {
        let record = fingerprinted(record);
        let (status, body) = post(&service.client, &reviews, REVIEWER, &record).await?;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        let detail = body["detail"].as_str().unwrap();
        assert!(detail.contains(expected), "{expected}: {detail}");
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM assurance_reviews")
            .fetch_one(&service.pool)
            .await?,
        0,
        "a malformed review leaves no review rows"
    );
    service.server.abort();
    Ok(())
}
