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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Producer,
    Reviewer,
    Owner,
    Viewer,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCredentials {
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
        .route(
            "/v1/projects/{project}/reviews",
            post(ingest_review).get(review_history),
        )
        .route("/v1/projects/{project}/runs", post(ingest).get(list_runs))
        .route("/v1/projects/{project}/subjects", get(list_subjects))
        .route("/v1/projects/{project}/runs/{run_id}", get(history))
        .route(
            "/v1/projects/{project}/subjects/{subject}/state",
            get(subject_state),
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
    let rows = sqlx::query("SELECT revision, fingerprint, payload FROM run_bundles WHERE project = $1 AND run_id = $2 ORDER BY revision")
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
    sqlx::query("INSERT INTO run_bundles(project, run_id, revision, fingerprint, subject_fingerprint, model_fingerprint, payload) VALUES ($1,$2,$3,$4,$5,$6,$7)")
        .bind(&project).bind(&bundle.run_id).bind(revision).bind(&bundle.bundle_fingerprint)
        .bind(&bundle.subject_fingerprint).bind(&bundle.plan.model_fingerprint).bind(payload)
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
        "SELECT revision, CASE WHEN sum(CASE WHEN ordinal < $4 THEN octet_length(payload::text) ELSE 0 END) OVER () > 33554432 THEN NULL WHEN ordinal < $4 THEN payload ELSE 'null'::jsonb END AS payload FROM (SELECT revision,payload,row_number() OVER (ORDER BY revision) AS ordinal FROM (SELECT revision,payload FROM run_bundles WHERE project = $1 AND run_id = $2 AND revision > $3 ORDER BY revision LIMIT $4) candidates) page ORDER BY revision",
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
    let result = assurance_review::subject_state(&authority, &reviews, &bundles, &subject);
    Ok(Json(to_value(&result)?))
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
    sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,payload,registered_by) VALUES ($1,$2,$3,$4) ON CONFLICT (project,fingerprint) DO NOTHING")
        .bind(&project).bind(fingerprint).bind(&payload).bind(&owner).execute(&mut *transaction).await?;
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
    let reviewer = state.authorize(&project, &headers, Some(Role::Reviewer))?;
    let record = core_json(&body)?;
    assurance_review::validate_record(&record).map_err(LedgerError::invalid)?;
    let payload = to_value(&record)?;
    if payload.get("reviewer").and_then(Value::as_str) != Some(&reviewer) {
        return Err(LedgerError::forbidden());
    }
    let fingerprint =
        assurance_review::record_fingerprint(&record).map_err(LedgerError::invalid)?;
    let (authority, mut reviews, bundles) = load_snapshot(&state.pool, &project).await?;
    reviews.push(record.clone());
    let assessment = assurance_review::assess_record(&record, &authority, &reviews, &bundles);
    if assessment.status == "invalid" {
        return Err(LedgerError::invalid(assessment.diagnostics.join("; ")));
    }
    let target = assurance_review::review_target(&record).map_err(LedgerError::invalid)?;
    let target_fp = run::canonical_fingerprint(&target).map_err(LedgerError::invalid)?;
    let inserted = sqlx::query("INSERT INTO assurance_reviews(project,fingerprint,target_fingerprint,payload,submitted_by) VALUES ($1,$2,$3,$4,$5) ON CONFLICT(project,fingerprint) DO NOTHING")
        .bind(&project).bind(&fingerprint).bind(target_fp).bind(payload).bind(&reviewer).execute(&state.pool).await?.rows_affected() == 1;
    Ok((
        if inserted {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(
            json!({"project":project,"fingerprint":fingerprint,"replayed":!inserted,"assessment":assessment.status,"diagnostics":assessment.diagnostics}),
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
    let rows = sqlx::query("SELECT fingerprint, CASE WHEN sum(CASE WHEN ordinal < $3 THEN octet_length(payload::text) ELSE 0 END) OVER () > 33554432 THEN NULL WHEN ordinal < $3 THEN payload ELSE 'null'::jsonb END AS payload FROM (SELECT fingerprint,payload,row_number() OVER (ORDER BY fingerprint) AS ordinal FROM (SELECT fingerprint,payload FROM assurance_reviews WHERE project=$1 AND fingerprint > $2 ORDER BY fingerprint LIMIT $3) candidates) page ORDER BY fingerprint")
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
        records.push(json!({"record":payload,"assessment":assessment.status,"diagnostics":assessment.diagnostics}));
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
    let rows = sqlx::query("SELECT DISTINCT ON (run_id) run_id,revision,fingerprint,subject_fingerprint,model_fingerprint,payload->'status' AS status,payload->'subject' AS subject FROM run_bundles WHERE project=$1 AND run_id > $2 ORDER BY run_id,revision DESC LIMIT $3")
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
    let rows = sqlx::query("SELECT DISTINCT ON (subject_fingerprint) subject_fingerprint,payload->'subject' AS subject FROM run_bundles WHERE project=$1 AND subject_fingerprint > $2 ORDER BY subject_fingerprint,run_id,revision DESC LIMIT $3")
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
    let value: Option<Value> = sqlx::query_scalar("SELECT a.payload FROM assurance_authorities a JOIN current_authorities c ON a.project=c.project AND a.fingerprint=c.fingerprint WHERE a.project=$1")
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
    // One database snapshot prevents a correction or authority selection from producing a torn assessment.
    // A complete bounded snapshot is preferable to a misleading conclusion from truncated evidence.
    let value: Value = sqlx::query_scalar(r#"
        WITH heads AS (SELECT DISTINCT ON (run_id) run_id,payload FROM run_bundles WHERE project=$1 ORDER BY run_id,revision DESC),
        reviews AS (SELECT fingerprint,payload FROM assurance_reviews WHERE project=$1),
        limits AS (SELECT
            (SELECT count(*) FROM heads) + (SELECT count(*) FROM reviews) AS records,
            COALESCE((SELECT sum(octet_length(payload::text)) FROM heads),0) +
            COALESCE((SELECT sum(octet_length(payload::text)) FROM reviews),0) AS bytes)
        SELECT CASE WHEN records > 5000 OR bytes > 33554432 THEN jsonb_build_object('too_large',true)
        ELSE jsonb_build_object(
            'authority', (SELECT a.payload FROM assurance_authorities a JOIN current_authorities c
                ON a.project=c.project AND a.fingerprint=c.fingerprint WHERE a.project=$1),
            'reviews', COALESCE((SELECT jsonb_agg(payload ORDER BY fingerprint) FROM reviews), '[]'::jsonb),
            'runs', COALESCE((SELECT jsonb_agg(payload ORDER BY run_id) FROM heads), '[]'::jsonb)) END
        FROM limits
        "#).bind(project).fetch_one(pool).await?;
    if value["too_large"] == true {
        return Err(LedgerError::invalid(
            "assessment exceeds 5000 records or 32 MiB; no partial Assurance State is inferred",
        ));
    }
    if value["authority"].is_null() {
        return Err(LedgerError::not_found());
    }
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
