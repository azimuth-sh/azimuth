use crate::relational;
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use azimuth::{assurance_review, fingerprint::sha256, json::Json as CoreJson, run};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::{collections::BTreeMap, sync::Arc};

mod deployment_observation;
mod environment;
mod model_catalog;
mod release;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Publisher,
    Producer,
    Reviewer,
    Owner,
    Viewer,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCredentials {
    #[serde(default)]
    pub publishers: BTreeMap<String, String>,
    pub producers: BTreeMap<String, String>,
    pub reviewers: BTreeMap<String, String>,
    pub owners: BTreeMap<String, String>,
    #[serde(default)]
    pub viewers: BTreeMap<String, String>,
}

struct Credential {
    identity: String,
    role: Role,
    digest: String,
}

#[derive(Clone)]
pub struct LedgerState {
    pool: PgPool,
    credentials: Arc<BTreeMap<String, Vec<Credential>>>,
}

impl LedgerState {
    pub fn new(
        pool: PgPool,
        projects: BTreeMap<String, ProjectCredentials>,
    ) -> Result<Self, String> {
        if projects.is_empty() {
            return Err("at least one project is required".into());
        }
        let mut credentials = BTreeMap::new();
        let mut unique = std::collections::BTreeSet::new();
        for (project, roles) in projects {
            azimuth::diag::validate_id(&project, false).map_err(|_| "invalid project ID")?;
            if roles.producers.is_empty() || roles.reviewers.is_empty() || roles.owners.is_empty() {
                return Err(
                    "each project requires separate producer, reviewer and owner credentials"
                        .into(),
                );
            }
            let mut entries = Vec::new();
            for (role, identities) in [
                (Role::Publisher, roles.publishers),
                (Role::Producer, roles.producers),
                (Role::Reviewer, roles.reviewers),
                (Role::Owner, roles.owners),
                (Role::Viewer, roles.viewers),
            ] {
                for (identity, token) in identities {
                    azimuth::diag::validate_id(&identity, false)
                        .map_err(|_| "invalid credential identity")?;
                    if token.len() < 32 || token.bytes().any(|b| b.is_ascii_whitespace()) {
                        return Err("credentials require at least 32 non-whitespace bytes".into());
                    }
                    let digest = sha256(token.as_bytes());
                    if !unique.insert(digest.clone()) {
                        return Err("credentials must be distinct across roles and projects".into());
                    }
                    entries.push(Credential {
                        identity,
                        role,
                        digest,
                    });
                }
            }
            credentials.insert(project, entries);
        }
        Ok(Self {
            pool,
            credentials: Arc::new(credentials),
        })
    }

    fn authorize(
        &self,
        project: &str,
        headers: &HeaderMap,
        required: Option<Role>,
    ) -> Result<String, LedgerError> {
        let mut values = headers.get_all("authorization").iter();
        let supplied = values
            .next()
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if values.next().is_some() {
            return Err(LedgerError::denied());
        }
        let entries = self
            .credentials
            .get(project)
            .ok_or_else(LedgerError::denied)?;
        let actual = sha256(supplied.ok_or_else(LedgerError::denied)?.as_bytes());
        let mut matched = None;
        for entry in entries {
            let difference = entry
                .digest
                .bytes()
                .zip(actual.bytes())
                .fold(0u8, |d, (a, b)| d | (a ^ b));
            if difference == 0 && entry.digest.len() == actual.len() {
                matched = Some(entry);
            }
        }
        let entry = matched.ok_or_else(LedgerError::denied)?;
        if required.is_some_and(|role| role != entry.role) {
            return Err(LedgerError::forbidden());
        }
        Ok(entry.identity.clone())
    }
}

pub fn app(state: LedgerState) -> Router {
    Router::new()
        .route("/health", get(|| async { Json(json!({"status":"ok"})) }))
        .route(
            "/v1/projects/{project}/authority",
            post(register_authority).get(current_authority),
        )
        .route("/v1/projects/{project}/claim-reviews", get(claim_reviews))
        .route(
            "/v1/projects/{project}/reviews",
            post(ingest_review).get(review_history),
        )
        .route(
            "/v1/projects/{project}/model-revisions",
            post(register_model_revision).get(list_model_revisions),
        )
        .route(
            "/v1/projects/{project}/model-revisions/{revision}",
            get(model_revision),
        )
        .route(
            "/v1/projects/{project}/model-revisions/{revision}/snapshot",
            get(model_revision_snapshot),
        )
        .route(
            "/v1/projects/{project}/model-revisions/{revision}/claim-reviews",
            get(model_revision_claim_reviews),
        )
        .route(
            "/v1/projects/{project}/model-revisions/{revision}/reviews",
            post(ingest_model_revision_review),
        )
        .route(
            "/v1/projects/{project}/model-revisions/{revision}/subjects/{subject}/state",
            get(model_revision_subject_state),
        )
        .route(
            "/v1/projects/{project}/model-revisions/{revision}/run-models",
            post(associate_run_model),
        )
        .route(
            "/v1/projects/{project}/model-revisions/{revision}/catalog",
            post(model_catalog::register).get(model_catalog::show),
        )
        .route(
            "/v1/projects/{project}/environments/{environment}/observations",
            post(deployment_observation::record).get(deployment_observation::current),
        )
        .route(
            "/v1/projects/{project}/environments",
            get(environment::list),
        )
        .route(
            "/v1/projects/{project}/environments/{environment}/deployment",
            post(environment::select).get(environment::current),
        )
        .route(
            "/v1/projects/{project}/deployments/{deployment}",
            get(environment::deployment),
        )
        .route("/v1/projects/{project}/runs", post(ingest).get(list_runs))
        .route("/v1/projects/{project}/subjects", get(list_subjects))
        .route("/v1/projects/{project}/runs/{run_id}", get(history))
        .route(
            "/v1/projects/{project}/subjects/{subject}/state",
            get(subject_state),
        )
        .route(
            "/v1/projects/{project}/releases",
            post(release::publish).get(release::list),
        )
        .route(
            "/v1/projects/{project}/releases/{release}",
            get(release::show),
        )
        .route(
            "/v1/projects/{project}/releases/{release}/summary",
            get(release::summary),
        )
        .route(
            "/v1/projects/{project}/releases/{release}/acceptance",
            post(release::accept),
        )
        .route("/v1/projects/{project}/changes", get(release::changes))
        .route(
            "/v1/projects/{project}/deployments",
            post(release::deploy).get(release::deployments),
        )
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .with_state(state)
}

async fn ingest(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    state.authorize(&project, &headers, Some(Role::Producer))?;
    let source = std::str::from_utf8(&body)
        .map_err(|_| LedgerError::invalid("expected UTF-8 Run bundle"))?;
    let bundle = parse_bundle(source)?;
    let canonical = run::canonical_json(&run::to_json(&bundle)).map_err(LedgerError::invalid)?;
    let payload: Value = serde_json::from_str(&canonical)
        .map_err(|_| LedgerError::invalid("invalid canonical bundle"))?;
    let revision = i64::try_from(bundle.bundle_revision)
        .map_err(|_| LedgerError::invalid("revision is out of range"))?;
    let mut transaction = state.pool.begin().await?;
    // Concurrent initial submissions and corrections must share one serial history.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("{project}|{}", bundle.run_id))
        .execute(&mut *transaction)
        .await?;
    let rows = sqlx::query("SELECT revision, fingerprint, payload FROM (SELECT storage.*,assurance_record('run',storage.project,storage.fingerprint) AS payload FROM run_bundles storage) AS run_bundles WHERE project = $1 AND run_id = $2 ORDER BY revision")
        .bind(&project).bind(&bundle.run_id).fetch_all(&mut *transaction).await?;
    let mut previous = Vec::new();
    for row in rows {
        let stored_revision: i64 = row.try_get("revision")?;
        let fingerprint: String = row.try_get("fingerprint")?;
        if stored_revision == revision {
            if fingerprint != bundle.bundle_fingerprint {
                return Err(LedgerError::conflict(
                    "Run revision already has different content",
                ));
            }
            transaction.commit().await?;
            return Ok((
                StatusCode::OK,
                Json(
                    json!({"replayed":true,"run_id":bundle.run_id,"bundle_revision":bundle.bundle_revision,"bundle_fingerprint":bundle.bundle_fingerprint}),
                ),
            ));
        }
        let value: Value = row.try_get("payload")?;
        previous.push(parse_bundle(&value.to_string())?);
    }
    if revision != previous.len() as i64 {
        return Err(LedgerError::conflict(
            "submit revision zero or the immediate next correction",
        ));
    }
    previous.push(bundle.clone());
    let findings = run::verify_set(&previous);
    if !findings.is_empty() {
        return Err(LedgerError::conflict(
            findings
                .iter()
                .map(|f| format!("{}: {}", f.code, f.detail))
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    relational::store(
        &mut transaction,
        "run",
        &project,
        &bundle.bundle_fingerprint,
        &payload,
    )
    .await?;
    sqlx::query("INSERT INTO run_bundles(project, run_id, revision, fingerprint, subject_fingerprint, model_fingerprint) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(&project).bind(&bundle.run_id).bind(revision).bind(&bundle.bundle_fingerprint)
        .bind(&bundle.subject_fingerprint).bind(&bundle.plan.model_fingerprint)
        .execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(
            json!({"replayed":false,"run_id":bundle.run_id,"bundle_revision":bundle.bundle_revision,"bundle_fingerprint":bundle.bundle_fingerprint}),
        ),
    ))
}

async fn history(
    State(state): State<LedgerState>,
    Path((project, run_id)): Path<(String, String)>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let after = if page.after.is_empty() {
        -1
    } else {
        page.after
            .parse::<i64>()
            .map_err(|_| LedgerError::invalid("Run cursor must be a nonnegative revision"))?
    };
    if after < -1 {
        return Err(LedgerError::invalid(
            "Run cursor must be a nonnegative revision",
        ));
    }
    let rows = sqlx::query(
        "SELECT revision, CASE WHEN sum(CASE WHEN ordinal < $4 THEN octet_length(payload::text) ELSE 0 END) OVER () > 33554432 THEN NULL WHEN ordinal < $4 THEN payload ELSE 'null'::jsonb END AS payload FROM (SELECT revision,payload,row_number() OVER (ORDER BY revision) AS ordinal FROM (SELECT revision,payload FROM (SELECT storage.*,assurance_record('run',storage.project,storage.fingerprint) AS payload FROM run_bundles storage) AS run_bundles WHERE project = $1 AND run_id = $2 AND revision > $3 ORDER BY revision LIMIT $4) candidates) page ORDER BY revision",
    )
    .bind(&project)
    .bind(&run_id)
    .bind(after)
    .bind(limit + 1)
    .fetch_all(&state.pool)
    .await?;
    require_bounded_payloads(&rows)?;
    let next = rows
        .get(limit as usize - 1)
        .filter(|_| rows.len() > limit as usize)
        .map(|r| r.try_get::<i64, _>("revision").map(|v| v.to_string()))
        .transpose()?;
    let values = rows
        .into_iter()
        .take(limit as usize)
        .map(|r| r.try_get::<Value, _>("payload"))
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        return Err(LedgerError::not_found());
    }
    Ok(Json(
        json!({"project":project,"run_id":run_id,"revisions":values,"next":next}),
    ))
}

async fn subject_state(
    State(state): State<LedgerState>,
    Path((project, subject)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM run_bundles WHERE project=$1 AND subject_fingerprint=$2)",
    )
    .bind(&project)
    .bind(&subject)
    .fetch_one(&state.pool)
    .await?;
    if !exists {
        return Err(LedgerError::not_found());
    }
    let (authority, reviews, bundles) = load_snapshot(&state.pool, &project).await?;
    compute(move || {
        Ok(Json(to_value(&assurance_review::subject_state(
            &authority, &reviews, &bundles, &subject,
        ))?))
    })
    .await
}

async fn claim_reviews(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let (authority, reviews, bundles) = load_snapshot(&state.pool, &project).await?;
    compute(move || definition_reviews(&authority, &reviews, &bundles)).await
}

async fn model_revision_claim_reviews(
    State(state): State<LedgerState>,
    Path((project, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let (authority, reviews, bundles) =
        load_revision_snapshot(&state.pool, &project, Some(&revision)).await?;
    compute(move || definition_reviews(&authority, &reviews, &bundles)).await
}

fn definition_reviews(
    authority: &CoreJson,
    reviews: &[CoreJson],
    bundles: &[run::RunBundle],
) -> Result<Json<Value>, LedgerError> {
    let projection = to_value(authority)?;
    let mut claims = Vec::new();
    for claim in projection["claims"]
        .as_array()
        .ok_or_else(|| LedgerError::invalid("authority Claims missing"))?
    {
        let id = claim["id"]
            .as_str()
            .ok_or_else(|| LedgerError::invalid("Claim identity missing"))?;
        let target = CoreJson::obj(vec![
            ("kind", CoreJson::str("claim-review")),
            ("claim", CoreJson::str(id)),
        ]);
        match assurance_review::latest_review(reviews, &target) {
            Ok(Some(record)) => {
                let assessment = assurance_review::assess_record(record, authority, reviews, bundles);
                claims.push(json!({"claim":id,"record":to_value(record)?,"assessment":assessment.status,"diagnostics":assessment.diagnostics}));
            }
            Ok(None) => claims.push(json!({"claim":id,"record":null,"assessment":"missing","diagnostics":["Claim Review is absent"]})),
            Err(error) => claims.push(json!({"claim":id,"record":null,"assessment":"invalid","diagnostics":[error]})),
        }
    }
    Ok(Json(
        json!({"authority_fingerprint":projection["fingerprint"],"claims":claims}),
    ))
}

async fn register_model_revision(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let owner = state.authorize(&project, &headers, Some(Role::Owner))?;
    let authority = core_json(&body)?;
    assurance_review::validate_authority(&authority).map_err(LedgerError::invalid)?;
    let payload = to_value(&authority)?;
    if payload["project"].as_str() != Some(&project) {
        return Err(LedgerError::invalid("authority project must match route"));
    }
    let fingerprint = payload["fingerprint"]
        .as_str()
        .ok_or_else(|| LedgerError::invalid("authority fingerprint missing"))?;
    let mut transaction = state.pool.begin().await?;
    relational::store(
        &mut transaction,
        "authority",
        &project,
        fingerprint,
        &payload,
    )
    .await?;
    sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES ($1,$2,$3) ON CONFLICT(project,fingerprint) DO NOTHING")
        .bind(&project).bind(fingerprint).bind(&owner).execute(&mut *transaction).await?;
    let inserted = sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES ($1,$2,$3) ON CONFLICT(project,fingerprint) DO NOTHING")
        .bind(&project).bind(fingerprint).bind(&owner).execute(&mut *transaction).await?.rows_affected() == 1;
    transaction.commit().await?;
    Ok((
        if inserted {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(
            json!({"project":project,"revision":fingerprint,"registration_status":"candidate","replayed":!inserted}),
        ),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunModelAssociation {
    run_fingerprint: String,
    reason: String,
}

async fn associate_run_model(
    State(state): State<LedgerState>,
    Path((project, revision)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<RunModelAssociation>,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let owner = state.authorize(&project, &headers, Some(Role::Owner))?;
    if body.reason.trim().is_empty() || body.reason.len() > 16384 {
        return Err(LedgerError::invalid(
            "bounded association rationale required",
        ));
    }
    let authority: Option<Value> = sqlx::query_scalar("SELECT a.payload FROM (SELECT storage.*,assurance_record('authority',storage.project,storage.fingerprint) AS payload FROM assurance_authorities storage) a JOIN model_revisions r USING(project,fingerprint) WHERE r.project=$1 AND r.fingerprint=$2")
        .bind(&project).bind(&revision).fetch_optional(&state.pool).await?;
    let authority = authority.ok_or_else(LedgerError::not_found)?;
    let payload = relational::load(&state.pool, "run", &project, &body.run_fingerprint).await?;
    let payload = payload.ok_or_else(LedgerError::not_found)?;
    let checks = payload["plan"]["checks"]
        .as_array()
        .ok_or_else(|| LedgerError::invalid("Run Check plan missing"))?;
    let current = authority["checks"]
        .as_array()
        .ok_or_else(|| LedgerError::invalid("authority Checks missing"))?;
    if checks.is_empty()
        || !payload["plan"]["challenges"]
            .as_array()
            .is_some_and(Vec::is_empty)
        || checks.iter().any(|check| {
            !current.iter().any(|candidate| {
                candidate["id"] == check["id"] && candidate["fingerprint"] == check["fingerprint"]
            })
        })
    {
        return Err(LedgerError::invalid(
            "representative Run must be Check-only with exact current Check identities",
        ));
    }
    let bundle = tokio::task::spawn_blocking(move || parse_bundle(&payload.to_string()))
        .await
        .map_err(|_| LedgerError::invalid("Run validation worker failed"))??;
    let inserted = sqlx::query("INSERT INTO model_revision_run_models(project,revision_fingerprint,run_model_fingerprint,representative_run_fingerprint,reason,associated_by) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING")
        .bind(&project).bind(&revision).bind(&bundle.plan.model_fingerprint).bind(&body.run_fingerprint)
        .bind(&body.reason).bind(&owner).execute(&state.pool).await?.rows_affected() == 1;
    Ok((
        if inserted {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(json!({
            "project":project,"revision":revision,"run_model_fingerprint":bundle.plan.model_fingerprint,
            "replayed":!inserted,"scope":"Run context association only; review currentness and exact-Subject Claim Health remain independently assessed"
        })),
    ))
}

async fn model_revision(
    State(state): State<LedgerState>,
    Path((project, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let value: Option<Value> = sqlx::query_scalar("SELECT jsonb_build_object('project',r.project,'revision',r.fingerprint,'registration_status',r.registration_status,'registered_by',r.registered_by,'registered_at',r.registered_at,'selected',EXISTS(SELECT 1 FROM current_authorities c WHERE c.project=r.project AND c.fingerprint=r.fingerprint),'authority',a.payload) FROM model_revisions r JOIN (SELECT storage.*,assurance_record('authority',storage.project,storage.fingerprint) AS payload FROM assurance_authorities storage) a USING(project,fingerprint) WHERE r.project=$1 AND r.fingerprint=$2")
        .bind(&project).bind(&revision).fetch_optional(&state.pool).await?;
    Ok(Json(value.ok_or_else(LedgerError::not_found)?))
}

async fn list_model_revisions(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows = sqlx::query("SELECT r.fingerprint,jsonb_build_object('revision',r.fingerprint,'registration_status',r.registration_status,'registered_by',r.registered_by,'registered_at',r.registered_at,'selected',EXISTS(SELECT 1 FROM current_authorities c WHERE c.project=r.project AND c.fingerprint=r.fingerprint)) AS payload FROM model_revisions r WHERE r.project=$1 AND r.fingerprint > $2 ORDER BY r.fingerprint LIMIT $3")
        .bind(&project).bind(&page.after).bind(limit+1).fetch_all(&state.pool).await?;
    let next = rows
        .get(limit as usize - 1)
        .filter(|_| rows.len() > limit as usize)
        .map(|row| row.try_get::<String, _>("fingerprint"))
        .transpose()?;
    let items = rows
        .into_iter()
        .take(limit as usize)
        .map(|row| row.try_get::<Value, _>("payload"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(json!({"project":project,"items":items,"next":next})))
}

async fn model_revision_snapshot(
    State(state): State<LedgerState>,
    Path((project, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let (authority, reviews, bundles) =
        load_revision_snapshot(&state.pool, &project, Some(&revision)).await?;
    let reviews = reviews
        .iter()
        .map(to_value)
        .collect::<Result<Vec<_>, _>>()?;
    let runs = bundles
        .iter()
        .map(|bundle| to_value(&run::to_json(bundle)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(
        json!({"project":project,"model_revision":revision,"authority":to_value(&authority)?,"reviews":reviews,"bundles":runs}),
    ))
}

async fn model_revision_subject_state(
    State(state): State<LedgerState>,
    Path((project, revision, subject)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let deployment_registered: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM release_deployments d JOIN assurance_releases r          ON r.project=d.project AND r.fingerprint=d.release_fingerprint          WHERE d.project=$1 AND r.model_revision=$2          AND assurance_record_field('deployment',d.project,d.deployment_id,'subject_fingerprint')=to_jsonb($3::text))",
    )
    .bind(&project)
    .bind(&revision)
    .bind(&subject)
    .fetch_one(&state.pool)
    .await?;
    if !deployment_registered {
        let observed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM run_bundles WHERE project=$1 AND subject_fingerprint=$2)",
        )
        .bind(&project)
        .bind(&subject)
        .fetch_one(&state.pool)
        .await?;
        if !observed {
            return Err(LedgerError::not_found());
        }
    }
    let (authority, reviews, bundles) =
        load_revision_snapshot(&state.pool, &project, Some(&revision)).await?;
    if !deployment_registered
        && !bundles
            .iter()
            .any(|bundle| bundle.subject_fingerprint == subject)
    {
        return Err(LedgerError::not_found());
    }
    compute(move || {
        Ok(Json(to_value(&assurance_review::subject_state(
            &authority, &reviews, &bundles, &subject,
        ))?))
    })
    .await
}

async fn register_authority(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let owner = state.authorize(&project, &headers, Some(Role::Owner))?;
    let authority = core_json(&body)?;
    assurance_review::validate_authority(&authority).map_err(LedgerError::invalid)?;
    let payload = to_value(&authority)?;
    if payload.get("project").and_then(Value::as_str) != Some(&project) {
        return Err(LedgerError::invalid("authority project must match route"));
    }
    let fingerprint = payload
        .get("fingerprint")
        .and_then(Value::as_str)
        .ok_or_else(|| LedgerError::invalid("authority fingerprint missing"))?;
    let mut transaction = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("{project}|authority"))
        .execute(&mut *transaction)
        .await?;
    let current: Option<String> =
        sqlx::query_scalar("SELECT fingerprint FROM current_authorities WHERE project=$1")
            .bind(&project)
            .fetch_optional(&mut *transaction)
            .await?;
    let release_managed: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assurance_releases WHERE project=$1)")
            .bind(&project)
            .fetch_one(&mut *transaction)
            .await?;
    if release_managed {
        return Err(LedgerError::conflict(
            "release-managed projects promote models through explicit composed-release acceptance",
        ));
    }
    let match_fields = headers.get_all("if-match").iter().collect::<Vec<_>>();
    let none_fields = headers.get_all("if-none-match").iter().collect::<Vec<_>>();
    if match_fields.len() > 1
        || none_fields.len() > 1
        || (!match_fields.is_empty() && !none_fields.is_empty())
    {
        return Err(LedgerError::conflict(
            "supply exactly one authority selection precondition",
        ));
    }
    let if_match = match_fields.first().and_then(|value| value.to_str().ok());
    let initial = none_fields.first().and_then(|value| value.to_str().ok()) == Some("*");
    let replay = current.as_deref() == Some(fingerprint);
    let valid = match current.as_deref() {
        None => initial,
        Some(current) => if_match == Some(current) || (replay && initial),
    };
    if !valid {
        return Err(LedgerError::conflict("select authority using If-None-Match: * initially or If-Match with the current fingerprint"));
    }
    if replay {
        transaction.commit().await?;
        return Ok((
            StatusCode::OK,
            Json(
                json!({"project":project,"fingerprint":fingerprint,"selected_by":owner,"replayed":true}),
            ),
        ));
    }
    relational::store(
        &mut transaction,
        "authority",
        &project,
        fingerprint,
        &payload,
    )
    .await?;
    sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES ($1,$2,$3) ON CONFLICT (project,fingerprint) DO NOTHING")
        .bind(&project).bind(fingerprint).bind(&owner).execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES ($1,$2,$3) ON CONFLICT(project,fingerprint) DO NOTHING")
        .bind(&project).bind(fingerprint).bind(&owner).execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO current_authorities(project,fingerprint) VALUES ($1,$2) ON CONFLICT(project) DO UPDATE SET fingerprint=EXCLUDED.fingerprint")
        .bind(&project).bind(fingerprint).execute(&mut *transaction).await?;
    sqlx::query(
        "INSERT INTO authority_selections(project,fingerprint,selected_by) VALUES ($1,$2,$3)",
    )
    .bind(&project)
    .bind(fingerprint)
    .bind(&owner)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"project":project,"fingerprint":fingerprint,"selected_by":owner})),
    ))
}

async fn current_authority(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    Ok(Json(to_value(
        &load_authority(&state.pool, &project).await?,
    )?))
}

async fn ingest_review(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    submit_review(&state, &project, &headers, &body, None).await
}

async fn ingest_model_revision_review(
    State(state): State<LedgerState>,
    Path((project, revision)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    submit_review(&state, &project, &headers, &body, Some(&revision)).await
}

async fn submit_review(
    state: &LedgerState,
    project: &str,
    headers: &HeaderMap,
    body: &[u8],
    revision: Option<&str>,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let reviewer = state.authorize(project, headers, Some(Role::Reviewer))?;
    let record = core_json(body)?;
    assurance_review::validate_record(&record).map_err(LedgerError::invalid)?;
    let payload = to_value(&record)?;
    if !matches!(
        payload.get("kind").and_then(Value::as_str),
        Some("claim-review" | "claim-health")
    ) {
        return Err(LedgerError::invalid("new review submissions require claim-review or claim-health; earlier review kinds remain readable history"));
    }
    if payload.get("reviewer").and_then(Value::as_str) != Some(&reviewer) {
        return Err(LedgerError::forbidden());
    }
    let fingerprint =
        assurance_review::record_fingerprint(&record).map_err(LedgerError::invalid)?;
    let (authority, mut reviews, bundles) =
        load_revision_snapshot(&state.pool, project, revision).await?;
    let revision = to_value(&authority)?["fingerprint"]
        .as_str()
        .ok_or_else(|| LedgerError::invalid("authority fingerprint missing"))?
        .to_owned();
    reviews.push(record.clone());
    let assessment = assurance_review::assess_record(&record, &authority, &reviews, &bundles);
    if assessment.status == "invalid" {
        return Err(LedgerError::invalid(assessment.diagnostics.join("; ")));
    }
    let target = assurance_review::review_target(&record).map_err(LedgerError::invalid)?;
    let target_fp = run::canonical_fingerprint(&target).map_err(LedgerError::invalid)?;
    let mut transaction = state.pool.begin().await?;
    relational::store(&mut transaction, "review", project, &fingerprint, &payload).await?;
    sqlx::query("INSERT INTO assurance_reviews(project,fingerprint,target_fingerprint,submitted_by) VALUES ($1,$2,$3,$4) ON CONFLICT(project,fingerprint) DO NOTHING")
        .bind(project).bind(&fingerprint).bind(target_fp).bind(&reviewer).execute(&mut *transaction).await?;
    let inserted = sqlx::query("INSERT INTO model_revision_reviews(project,revision_fingerprint,review_fingerprint,submitted_by) VALUES ($1,$2,$3,$4) ON CONFLICT(project,revision_fingerprint,review_fingerprint) DO NOTHING")
        .bind(project).bind(&revision).bind(&fingerprint).bind(&reviewer).execute(&mut *transaction).await?.rows_affected() == 1;
    transaction.commit().await?;
    Ok((
        if inserted {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(
            json!({"project":project,"model_revision":revision,"fingerprint":fingerprint,"replayed":!inserted,"assessment":assessment.status,"diagnostics":assessment.diagnostics}),
        ),
    ))
}

async fn review_history(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows = sqlx::query("SELECT fingerprint, provenance_basis, submitted_by, submitted_at::text AS submitted_at, CASE WHEN sum(CASE WHEN ordinal < $3 THEN octet_length(payload::text) ELSE 0 END) OVER () > 33554432 THEN NULL WHEN ordinal < $3 THEN payload ELSE 'null'::jsonb END AS payload FROM (SELECT fingerprint,payload,provenance_basis,submitted_by,submitted_at,row_number() OVER (ORDER BY fingerprint) AS ordinal FROM (SELECT fingerprint,payload,provenance_basis,submitted_by,submitted_at FROM (SELECT storage.*,assurance_record('review',storage.project,storage.fingerprint) AS payload FROM assurance_reviews storage) AS assurance_reviews WHERE project=$1 AND fingerprint > $2 AND EXISTS (SELECT 1 FROM model_revision_reviews r JOIN current_authorities c ON r.project=c.project AND r.revision_fingerprint=c.fingerprint WHERE r.project=assurance_reviews.project AND r.review_fingerprint=assurance_reviews.fingerprint) ORDER BY fingerprint LIMIT $3) candidates) page ORDER BY fingerprint")
        .bind(&project).bind(&page.after).bind(limit + 1).fetch_all(&state.pool).await?;
    require_bounded_payloads(&rows)?;
    let (authority, reviews, bundles) = load_snapshot(&state.pool, &project).await?;
    let mut records = Vec::new();
    let next = rows
        .get(limit as usize - 1)
        .filter(|_| rows.len() > limit as usize)
        .map(|r| r.try_get::<String, _>("fingerprint"))
        .transpose()?;
    for row in rows.into_iter().take(limit as usize) {
        let payload: Value = row.try_get("payload")?;
        let record = core_json(payload.to_string().as_bytes())?;
        let assessment = assurance_review::assess_record(&record, &authority, &reviews, &bundles);
        records.push(json!({"historical": !matches!(payload.get("kind").and_then(Value::as_str), Some("claim-review" | "claim-health")), "record":payload,"assessment":assessment.status,"diagnostics":assessment.diagnostics,"provenance_basis":row.try_get::<String, _>("provenance_basis")?,"submitted_by":row.try_get::<String, _>("submitted_by")?,"submitted_at":row.try_get::<String, _>("submitted_at")?}));
    }
    Ok(Json(
        json!({"project":project,"records":records,"next":next}),
    ))
}

fn require_bounded_payloads(rows: &[sqlx::postgres::PgRow]) -> Result<(), LedgerError> {
    // Many individually valid records must not exhaust API memory when fetched together.
    for row in rows {
        if row.try_get::<Option<Value>, _>("payload")?.is_none() {
            return Err(LedgerError::invalid(
                "page exceeds 32 MiB; request a smaller limit",
            ));
        }
    }
    Ok(())
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Page {
    limit: Option<i64>,
    #[serde(default)]
    after: String,
}
impl Page {
    fn limit(&self) -> Result<i64, LedgerError> {
        let limit = self.limit.unwrap_or(25);
        if !(1..=100).contains(&limit) || self.after.len() > 1024 {
            return Err(LedgerError::invalid(
                "limit must be 1..100 and cursor at most 1024 bytes",
            ));
        }
        Ok(limit)
    }
}

async fn list_runs(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows = sqlx::query("SELECT DISTINCT ON (run_id) run_id,revision,fingerprint,subject_fingerprint,model_fingerprint,assurance_record_field('run',project,fingerprint,'status') AS status,assurance_record_field('run',project,fingerprint,'subject') AS subject FROM (SELECT storage.*,assurance_record('run',storage.project,storage.fingerprint) AS payload FROM run_bundles storage) AS run_bundles WHERE project=$1 AND run_id > $2 ORDER BY run_id,revision DESC LIMIT $3")
        .bind(&project).bind(&page.after).bind(limit + 1).fetch_all(&state.pool).await?;
    let next = rows
        .get(limit as usize - 1)
        .filter(|_| rows.len() > limit as usize)
        .map(|r| r.try_get::<String, _>("run_id"))
        .transpose()?;
    let mut items = Vec::new();
    for row in rows.into_iter().take(limit as usize) {
        items.push(json!({"run_id":row.try_get::<String,_>("run_id")?,"revision":row.try_get::<i64,_>("revision")?,"fingerprint":row.try_get::<String,_>("fingerprint")?,"subject_fingerprint":row.try_get::<String,_>("subject_fingerprint")?,"model_fingerprint":row.try_get::<String,_>("model_fingerprint")?,"status":row.try_get::<Value,_>("status")?,"subject":row.try_get::<Value,_>("subject")?}));
    }
    Ok(Json(json!({"project":project,"items":items,"next":next})))
}

async fn list_subjects(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows = sqlx::query("SELECT DISTINCT ON (subject_fingerprint) subject_fingerprint,assurance_record_field('run',project,fingerprint,'subject') AS subject FROM (SELECT storage.*,assurance_record('run',storage.project,storage.fingerprint) AS payload FROM run_bundles storage) AS run_bundles WHERE project=$1 AND subject_fingerprint > $2 ORDER BY subject_fingerprint,run_id,revision DESC LIMIT $3")
        .bind(&project).bind(&page.after).bind(limit + 1).fetch_all(&state.pool).await?;
    let next = rows
        .get(limit as usize - 1)
        .filter(|_| rows.len() > limit as usize)
        .map(|r| r.try_get::<String, _>("subject_fingerprint"))
        .transpose()?;
    let mut items = Vec::new();
    for row in rows.into_iter().take(limit as usize) {
        items.push(json!({"fingerprint":row.try_get::<String,_>("subject_fingerprint")?,"subject":row.try_get::<Value,_>("subject")?}));
    }
    Ok(Json(json!({"project":project,"items":items,"next":next})))
}

async fn load_authority(pool: &PgPool, project: &str) -> Result<CoreJson, LedgerError> {
    let value: Option<Value> = sqlx::query_scalar("SELECT a.payload FROM (SELECT storage.*,assurance_record('authority',storage.project,storage.fingerprint) AS payload FROM assurance_authorities storage) a JOIN current_authorities c ON a.project=c.project AND a.fingerprint=c.fingerprint WHERE a.project=$1")
        .bind(project).fetch_optional(pool).await?;
    core_json(
        value
            .ok_or_else(LedgerError::not_found)?
            .to_string()
            .as_bytes(),
    )
}

async fn load_snapshot(
    pool: &PgPool,
    project: &str,
) -> Result<(CoreJson, Vec<CoreJson>, Vec<run::RunBundle>), LedgerError> {
    load_revision_snapshot(pool, project, None).await
}

async fn load_revision_snapshot(
    pool: &PgPool,
    project: &str,
    revision: Option<&str>,
) -> Result<(CoreJson, Vec<CoreJson>, Vec<run::RunBundle>), LedgerError> {
    // One database snapshot prevents a correction or authority selection from producing a torn assessment.
    // A complete bounded snapshot is preferable to a misleading conclusion from truncated evidence.
    let value: Value = sqlx::query_scalar(r#"
        WITH selected AS MATERIALIZED (
            SELECT fingerprint FROM model_revisions WHERE project=$1 AND fingerprint=$2
            UNION ALL SELECT fingerprint FROM current_authorities WHERE project=$1 AND $2::text IS NULL),
        authority AS MATERIALIZED (
            SELECT a.fingerprint,assurance_record('authority',a.project,a.fingerprint) AS payload
            FROM assurance_authorities a JOIN selected s ON s.fingerprint=a.fingerprint WHERE a.project=$1),
        associated_plans AS MATERIALIZED (
            SELECT b.run_id,assurance_record_field('run',b.project,b.fingerprint,'plan') AS plan
            FROM run_bundles b JOIN model_revision_run_models m
                ON m.project=b.project AND m.run_model_fingerprint=b.model_fingerprint
            JOIN selected s ON s.fingerprint=m.revision_fingerprint WHERE b.project=$1),
        relevant_runs AS (
            SELECT b.run_id FROM run_bundles b JOIN authority a
                ON a.payload->>'model_fingerprint'=b.model_fingerprint WHERE b.project=$1
            UNION
            SELECT b.run_id FROM associated_plans b CROSS JOIN authority a
                WHERE jsonb_array_length(b.plan->'checks') > 0
                    AND jsonb_array_length(b.plan->'challenges') = 0
                    AND NOT EXISTS (
                        SELECT 1 FROM jsonb_array_elements(b.plan->'checks') c
                        WHERE NOT EXISTS (SELECT 1 FROM jsonb_array_elements(a.payload->'checks') current_check
                            WHERE current_check->>'id'=c->>'id' AND current_check->>'fingerprint'=c->>'fingerprint'))
            UNION
            SELECT b.run_id FROM run_bundles b JOIN release_run_submissions r
                ON r.project=b.project AND r.run_fingerprint=b.fingerprint
                JOIN assurance_releases p ON p.project=r.project AND p.fingerprint=r.release_fingerprint
                JOIN selected s ON s.fingerprint=p.model_revision WHERE b.project=$1),
        head_keys AS MATERIALIZED (
            SELECT DISTINCT ON (b.run_id) b.run_id,b.project,b.fingerprint FROM run_bundles b
            JOIN relevant_runs r ON r.run_id=b.run_id WHERE b.project=$1 ORDER BY b.run_id,b.revision DESC),
        heads AS MATERIALIZED (
            SELECT run_id,assurance_record('run',project,fingerprint) AS payload FROM head_keys),
        reviews AS MATERIALIZED (
            SELECT a.fingerprint,assurance_record('review',a.project,a.fingerprint) AS payload
            FROM assurance_reviews a JOIN model_revision_reviews r
                ON a.project=r.project AND a.fingerprint=r.review_fingerprint
            JOIN selected s ON s.fingerprint=r.revision_fingerprint WHERE a.project=$1),
        limits AS (SELECT
            (SELECT count(*) FROM heads) + (SELECT count(*) FROM reviews) AS records,
            COALESCE((SELECT sum(octet_length(payload::text)) FROM heads),0) +
            COALESCE((SELECT sum(octet_length(payload::text)) FROM reviews),0) +
            COALESCE((SELECT sum(octet_length(payload::text)) FROM authority),0) AS bytes)
        SELECT CASE WHEN records > 5000 OR bytes > 33554432 THEN jsonb_build_object('too_large',true)
        ELSE jsonb_build_object(
            'authority', (SELECT payload FROM authority),
            'reviews', COALESCE((SELECT jsonb_agg(payload ORDER BY fingerprint) FROM reviews), '[]'::jsonb),
            'runs', COALESCE((SELECT jsonb_agg(payload ORDER BY run_id) FROM heads), '[]'::jsonb)) END
        FROM limits
        "#).bind(project).bind(revision).fetch_one(pool).await?;
    if value["too_large"] == true {
        return Err(LedgerError::invalid(
            "assessment exceeds 5000 records or 32 MiB; no partial Assurance State is inferred",
        ));
    }
    if value["authority"].is_null() {
        return Err(LedgerError::not_found());
    }
    compute(move || {
        let authority = core_json(value["authority"].to_string().as_bytes())?;
        let reviews = value["reviews"]
            .as_array()
            .ok_or_else(|| LedgerError::invalid("invalid stored review set"))?
            .iter()
            .map(|v| core_json(v.to_string().as_bytes()))
            .collect::<Result<Vec<_>, _>>()?;
        let bundles = value["runs"]
            .as_array()
            .ok_or_else(|| LedgerError::invalid("invalid stored Run set"))?
            .iter()
            .map(|v| parse_bundle(&v.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((authority, reviews, bundles))
    })
    .await
}

fn core_json(body: &[u8]) -> Result<CoreJson, LedgerError> {
    let source =
        std::str::from_utf8(body).map_err(|_| LedgerError::invalid("expected UTF-8 JSON"))?;
    run::strict_json("request", source).map_err(|e| LedgerError::invalid(e.to_string()))
}

fn to_value(value: &CoreJson) -> Result<Value, LedgerError> {
    let canonical = run::canonical_json(value).map_err(LedgerError::invalid)?;
    serde_json::from_str(&canonical).map_err(|_| LedgerError::invalid("invalid canonical JSON"))
}

fn parse_bundle(source: &str) -> Result<run::RunBundle, LedgerError> {
    let bundle = run::parse("request", source).map_err(|errors| {
        LedgerError::invalid(
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    let findings = run::verify(&bundle);
    if !findings.is_empty() {
        return Err(LedgerError::invalid(
            findings
                .iter()
                .map(|f| format!("{}: {}", f.code, f.detail))
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    Ok(bundle)
}

pub struct LedgerError {
    status: StatusCode,
    detail: String,
}
impl LedgerError {
    fn denied() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            detail: "valid project credential required".into(),
        }
    }
    fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            detail: "credential role does not authorize this operation".into(),
        }
    }
    fn invalid(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            detail: detail.into(),
        }
    }
    fn conflict(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            detail: detail.into(),
        }
    }
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            detail: "record not found".into(),
        }
    }
}
impl From<sqlx::Error> for LedgerError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Protocol(detail) = &error {
            return Self::invalid(detail.clone());
        }
        tracing::error!(error = %error, "Run ledger database operation failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            detail: "Run ledger database operation failed".into(),
        }
    }
}
impl IntoResponse for LedgerError {
    fn into_response(self) -> Response {
        let mut response = (
            self.status,
            Json(json!({"type":"about:blank","status":self.status.as_u16(),"detail":self.detail})),
        )
            .into_response();
        if self.status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert("www-authenticate", "Bearer".parse().unwrap());
        }
        response
    }
}

async fn compute<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, LedgerError> + Send + 'static,
) -> Result<T, LedgerError> {
    static LIMIT: std::sync::OnceLock<Arc<tokio::sync::Semaphore>> = std::sync::OnceLock::new();
    let permit = LIMIT
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| LedgerError::invalid("assessment worker unavailable"))?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        work()
    })
    .await
    .map_err(|_| LedgerError::invalid("assessment worker failed"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::{
        postgres::Postgres,
        testcontainers::{runners::AsyncRunner, ImageExt},
    };

    #[tokio::test]
    async fn registered_deployment_without_runs_lists_unresolved_claims(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().with_tag("17-alpine").start().await?;
        let port = postgres.get_host_port_ipv4(5432).await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
        ))
        .await?;
        crate::migrate(&pool).await?;
        let model = azimuth::model::Model {
            specs: vec![azimuth::spec::parse_spec(
                "spec.md",
                "# Spec: module\n\n## Claim: bounded\nCriticality: routine\n\nThe operation SHALL complete.\n\n### Case: completes\nGiven ordinary input, when invoked, then complete.\n",
            ).unwrap()],
            ..Default::default()
        };
        let authority = to_value(&assurance_review::authority(&model, "project")?)
            .map_err(|error| error.detail)?;
        let revision = authority["fingerprint"].as_str().unwrap();
        let digest = format!("sha256:{}", "a".repeat(64));
        let subject = json!({"kind":"deployment","environment":"production","deployment":"deployment","deployment_fingerprint":digest,"artifacts":[{"id":"service","digest":digest}]});
        let subject_fingerprint = run::subject_fingerprint(&run::subject_from_json(
            &core_json(subject.to_string().as_bytes()).map_err(|error| error.detail)?,
        )?);
        let deployment = json!({"id":"deployment","environment":"production","configuration_fingerprint":digest,"artifacts":[digest],"subject":subject,"subject_fingerprint":subject_fingerprint});
        let mut tx = pool.begin().await?;
        relational::store(&mut tx, "authority", "project", revision, &authority).await?;
        sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES ('project',$1,'owner')").bind(revision).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES ('project',$1,'owner')").bind(revision).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO assurance_releases(project,fingerprint,model_revision,label,baseline_model_revision,submitted_by) VALUES ('project',$1,$2,'Release',$2,'publisher')").bind(&digest).bind(revision).execute(&mut *tx).await?;
        release::insert_deployment(&mut tx, "project", &digest, &deployment, "owner")
            .await
            .map_err(|error| error.detail)?;
        tx.commit().await?;
        let credential = "viewer-credential-for-http-regression-only";
        let state = LedgerState::new(
            pool.clone(),
            BTreeMap::from([(
                "project".into(),
                ProjectCredentials {
                    publishers: BTreeMap::new(),
                    producers: BTreeMap::from([(
                        "producer".into(),
                        "producer-credential-for-http-regression-only".into(),
                    )]),
                    reviewers: BTreeMap::from([(
                        "reviewer".into(),
                        "reviewer-credential-for-http-regression-only".into(),
                    )]),
                    owners: BTreeMap::from([(
                        "owner".into(),
                        "owner-credential-for-http-regression-only".into(),
                    )]),
                    viewers: BTreeMap::from([("viewer".into(), credential.into())]),
                },
            )]),
        )
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server = tokio::spawn(async move { axum::serve(listener, app(state)).await.unwrap() });
        let client = reqwest::Client::new();
        let url =
            format!("http://{address}/v1/projects/project/model-revisions/{revision}/subjects");
        let response = client
            .get(format!("{url}/{subject_fingerprint}/state"))
            .bearer_auth(credential)
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value = response.json().await?;
        assert_eq!(result["claims"].as_array().unwrap().len(), 1);
        assert_eq!(result["claims"][0]["claim"], "bounded");
        assert_eq!(result["claims"][0]["status"], "unresolved");
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM run_bundles")
                .fetch_one(&pool)
                .await?,
            0
        );
        let unknown = format!("sha256:{}", "b".repeat(64));
        assert_eq!(
            client
                .get(format!("{url}/{unknown}/state"))
                .bearer_auth(credential)
                .send()
                .await?
                .status(),
            StatusCode::NOT_FOUND
        );
        let wrong_model = format!("http://{address}/v1/projects/project/model-revisions/{unknown}/subjects/{subject_fingerprint}/state");
        assert_eq!(
            client
                .get(wrong_model)
                .bearer_auth(credential)
                .send()
                .await?
                .status(),
            StatusCode::NOT_FOUND
        );
        server.abort();
        Ok(())
    }
}
