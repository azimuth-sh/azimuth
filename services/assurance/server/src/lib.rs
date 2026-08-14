use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use azimuth_assurance_domain::{
    record_fingerprint, AssuranceAccount, Challenge, EvidenceDefinition, GateDecision, GateRequest,
    Observation, ProjectModelSnapshot, Qualification, PROJECT_SNAPSHOT_FORMAT,
    PROJECT_SNAPSHOT_VERSION,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashSet;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/projects", post(create_project).get(list_projects))
        .route(
            "/v1/projects/{project_id}/model-snapshots",
            post(register_model_snapshot).get(list_model_snapshots),
        )
        .route(
            "/v1/projects/{project_id}/definitions",
            post(register_definition).get(list_definitions),
        )
        .route(
            "/v1/projects/{project_id}/qualifications",
            post(register_qualification).get(list_qualifications),
        )
        .route(
            "/v1/projects/{project_id}/observations",
            post(register_observation).get(list_observations),
        )
        .route(
            "/v1/projects/{project_id}/challenges",
            post(register_challenge).get(list_challenges),
        )
        .route(
            "/v1/projects/{project_id}/gates/evaluate",
            post(evaluate_gate),
        )
        .route(
            "/v1/projects/{project_id}/gate-decisions",
            get(list_gate_decisions),
        )
        .route("/v1/projects/{project_id}/gates", get(list_current_gates))
        .route("/v1/projects/{project_id}/work-items", get(list_work_items))
        .route("/v1/projects/{project_id}/snapshot", get(project_snapshot))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Ingested {
    replayed: bool,
    content_fingerprint: String,
    definition_fingerprint: Option<String>,
}

async fn create_project(
    State(state): State<AppState>,
    Json(project): Json<Project>,
) -> Result<(StatusCode, Json<Ingested>), AppError> {
    validate_id(&project.id, "project id")?;
    validate_text(&project.name, "project name")?;
    let created_at = to_db_time(project.created_at, "createdAt")?;
    let content_fingerprint = fingerprint(&project)?;
    let result = sqlx::query(
        "INSERT INTO projects(id, name, created_at, content_fingerprint) \
         VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
    )
    .bind(&project.id)
    .bind(&project.name)
    .bind(created_at)
    .bind(&content_fingerprint)
    .execute(&state.pool)
    .await?;
    let replayed = resolve_replay(
        result.rows_affected(),
        sqlx::query_scalar::<_, String>("SELECT content_fingerprint FROM projects WHERE id = $1")
            .bind(&project.id)
            .fetch_one(&state.pool)
            .await?,
        &content_fingerprint,
        "project",
    )?;
    Ok((
        if replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(Ingested {
            replayed,
            content_fingerprint,
            definition_fingerprint: None,
        }),
    ))
}

async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<Project>>, AppError> {
    let rows = sqlx::query("SELECT id, name, created_at FROM projects ORDER BY id")
        .fetch_all(&state.pool)
        .await?;
    rows.into_iter()
        .map(|row| {
            Ok(Project {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                created_at: from_db_time(row.try_get("created_at")?)?,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()
        .map(Json)
}

async fn register_model_snapshot(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(snapshot): Json<ProjectModelSnapshot>,
) -> Result<(StatusCode, Json<Ingested>), AppError> {
    ensure_project(&state.pool, &project_id).await?;
    validate_model_snapshot(&snapshot, &project_id)?;
    let content_fingerprint = fingerprint(&snapshot)?;
    let payload = serde_json::to_value(&snapshot)?;
    let result = sqlx::query(
        "INSERT INTO project_model_snapshots(\
            project_id, id, model_fingerprint, content_fingerprint, payload\
         ) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(&project_id)
    .bind(&snapshot.id)
    .bind(&snapshot.model_fingerprint)
    .bind(&content_fingerprint)
    .bind(payload)
    .execute(&state.pool)
    .await?;
    let stored = sqlx::query_scalar::<_, String>(
        "SELECT content_fingerprint FROM project_model_snapshots \
         WHERE project_id = $1 AND id = $2",
    )
    .bind(&project_id)
    .bind(&snapshot.id)
    .fetch_one(&state.pool)
    .await?;
    let replayed = resolve_replay(
        result.rows_affected(),
        stored,
        &content_fingerprint,
        "project model snapshot",
    )?;
    Ok((
        if replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(Ingested {
            replayed,
            content_fingerprint,
            definition_fingerprint: None,
        }),
    ))
}

async fn register_definition(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(definition): Json<EvidenceDefinition>,
) -> Result<(StatusCode, Json<Ingested>), AppError> {
    ensure_project(&state.pool, &project_id).await?;
    validate_id(&definition.id, "definition id")?;
    validate_definition(&definition)?;
    ensure_claim_contract_known(&state.pool, &project_id, &definition.claim).await?;
    let definition_fingerprint = definition.fingerprint();
    let content_fingerprint = fingerprint(&definition)?;
    let payload = serde_json::to_value(&definition)?;
    let result = sqlx::query(
        "INSERT INTO evidence_definitions(\
            project_id, logical_id, definition_fingerprint, declared_at, content_fingerprint, payload\
         ) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING",
    )
    .bind(&project_id)
    .bind(&definition.id)
    .bind(&definition_fingerprint)
    .bind(to_db_time(definition.declared_at, "declaredAt")?)
    .bind(&content_fingerprint)
    .bind(payload)
    .execute(&state.pool)
    .await?;
    let stored = sqlx::query_scalar::<_, String>(
        "SELECT content_fingerprint FROM evidence_definitions \
         WHERE project_id = $1 AND logical_id = $2 AND definition_fingerprint = $3",
    )
    .bind(&project_id)
    .bind(&definition.id)
    .bind(&definition_fingerprint)
    .fetch_one(&state.pool)
    .await?;
    let replayed = resolve_replay(
        result.rows_affected(),
        stored,
        &content_fingerprint,
        "definition version",
    )?;
    Ok((
        if replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(Ingested {
            replayed,
            content_fingerprint,
            definition_fingerprint: Some(definition_fingerprint),
        }),
    ))
}

async fn register_qualification(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(qualification): Json<Qualification>,
) -> Result<(StatusCode, Json<Ingested>), AppError> {
    validate_id(&qualification.id, "qualification id")?;
    validate_text(&qualification.rationale, "qualification rationale")?;
    to_db_time(qualification.qualified_at, "qualifiedAt")?;
    load_definition_version(
        &state.pool,
        &project_id,
        &qualification.definition_id,
        &qualification.definition_fingerprint,
    )
    .await?;
    insert_immutable(
        &state.pool,
        ImmutableWrite {
            table: ImmutableTable::Qualification,
            project_id: &project_id,
            id: &qualification.id,
            definition_id: &qualification.definition_id,
            definition_fingerprint: &qualification.definition_fingerprint,
            observed_at: qualification.qualified_at,
            record: &qualification,
            source: None,
        },
    )
    .await
}

async fn register_observation(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(observation): Json<Observation>,
) -> Result<(StatusCode, Json<Ingested>), AppError> {
    validate_id(&observation.id, "observation id")?;
    validate_subject(&observation.subject)?;
    to_db_time(observation.observed_at, "observedAt")?;
    if observation
        .expires_at
        .is_some_and(|expires_at| expires_at <= observation.observed_at)
    {
        return Err(AppError::unprocessable(
            "expiresAt must be later than observedAt",
        ));
    }
    if let Some(report) = &observation.report {
        validate_text(report, "observation report")?;
    }
    let definition = load_definition_version(
        &state.pool,
        &project_id,
        &observation.definition_id,
        &observation.definition_fingerprint,
    )
    .await?;
    let snapshot = load_model_snapshot(
        &state.pool,
        &project_id,
        &observation.subject.project_snapshot,
    )
    .await?;
    if !snapshot.contains(&definition.claim) {
        return Err(AppError::unprocessable(
            "observation snapshot does not contain the definition claim contract",
        ));
    }
    if observation.stage != definition.stage {
        return Err(AppError::unprocessable(
            "observation stage does not match its definition",
        ));
    }
    insert_immutable(
        &state.pool,
        ImmutableWrite {
            table: ImmutableTable::Observation,
            project_id: &project_id,
            id: &observation.id,
            definition_id: &observation.definition_id,
            definition_fingerprint: &observation.definition_fingerprint,
            observed_at: observation.observed_at,
            record: &observation,
            source: None,
        },
    )
    .await
}

async fn register_challenge(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(challenge): Json<Challenge>,
) -> Result<(StatusCode, Json<Ingested>), AppError> {
    validate_id(&challenge.id, "challenge id")?;
    validate_id(&challenge.source, "challenge source")?;
    to_db_time(challenge.observed_at, "observedAt")?;
    if let Some(report) = &challenge.report {
        validate_text(report, "challenge report")?;
    }
    load_definition_version(
        &state.pool,
        &project_id,
        &challenge.definition_id,
        &challenge.definition_fingerprint,
    )
    .await?;
    insert_immutable(
        &state.pool,
        ImmutableWrite {
            table: ImmutableTable::Challenge,
            project_id: &project_id,
            id: &challenge.id,
            definition_id: &challenge.definition_id,
            definition_fingerprint: &challenge.definition_fingerprint,
            observed_at: challenge.observed_at,
            record: &challenge,
            source: Some(&challenge.source),
        },
    )
    .await
}

#[derive(Clone, Copy)]
enum ImmutableTable {
    Qualification,
    Observation,
    Challenge,
}

impl ImmutableTable {
    fn name(self) -> &'static str {
        match self {
            Self::Qualification => "qualifications",
            Self::Observation => "observations",
            Self::Challenge => "challenges",
        }
    }

    fn insert_query(self) -> &'static str {
        match self {
            Self::Qualification => {
            "INSERT INTO qualifications(project_id, id, definition_id, definition_fingerprint, qualified_at, content_fingerprint, payload) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING"
            }
            Self::Observation => {
            "INSERT INTO observations(project_id, id, definition_id, definition_fingerprint, observed_at, content_fingerprint, payload) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING"
            }
            Self::Challenge => {
            "INSERT INTO challenges(project_id, id, definition_id, definition_fingerprint, observed_at, content_fingerprint, payload, source) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT DO NOTHING"
            }
        }
    }
}

struct ImmutableWrite<'a, T> {
    table: ImmutableTable,
    project_id: &'a str,
    id: &'a str,
    definition_id: &'a str,
    definition_fingerprint: &'a str,
    observed_at: u64,
    record: &'a T,
    source: Option<&'a str>,
}

async fn insert_immutable<T: Serialize>(
    pool: &PgPool,
    write: ImmutableWrite<'_, T>,
) -> Result<(StatusCode, Json<Ingested>), AppError> {
    let content_fingerprint = fingerprint(write.record)?;
    let payload = serde_json::to_value(write.record)?;
    let time = to_db_time(write.observed_at, "record time")?;
    let mut statement = sqlx::query(write.table.insert_query())
        .bind(write.project_id)
        .bind(write.id)
        .bind(write.definition_id)
        .bind(write.definition_fingerprint)
        .bind(time)
        .bind(&content_fingerprint)
        .bind(payload);
    if matches!(write.table, ImmutableTable::Challenge) {
        statement = statement.bind(write.source.expect("challenge source is required"));
    }
    let result = statement.execute(pool).await?;
    let table = write.table.name();
    let stored = sqlx::query_scalar::<_, String>(&format!(
        "SELECT content_fingerprint FROM {table} WHERE project_id = $1 AND id = $2"
    ))
    .bind(write.project_id)
    .bind(write.id)
    .fetch_one(pool)
    .await?;
    let replayed = resolve_replay(result.rows_affected(), stored, &content_fingerprint, table)?;
    Ok((
        if replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(Ingested {
            replayed,
            content_fingerprint,
            definition_fingerprint: None,
        }),
    ))
}

async fn list_definitions(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<EvidenceDefinition>>, AppError> {
    load_records(
        &state.pool,
        &project_id,
        "evidence_definitions",
        "declared_at, logical_id, definition_fingerprint",
    )
    .await
    .map(Json)
}

async fn list_model_snapshots(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<ProjectModelSnapshot>>, AppError> {
    load_records(
        &state.pool,
        &project_id,
        "project_model_snapshots",
        "sequence",
    )
    .await
    .map(Json)
}

async fn list_qualifications(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Qualification>>, AppError> {
    load_records(
        &state.pool,
        &project_id,
        "qualifications",
        "qualified_at, id",
    )
    .await
    .map(Json)
}

async fn list_observations(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Observation>>, AppError> {
    load_records(&state.pool, &project_id, "observations", "observed_at, id")
        .await
        .map(Json)
}

async fn list_challenges(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Challenge>>, AppError> {
    load_records(&state.pool, &project_id, "challenges", "observed_at, id")
        .await
        .map(Json)
}

async fn load_records<T: DeserializeOwned>(
    pool: &PgPool,
    project_id: &str,
    table: &str,
    order: &str,
) -> Result<Vec<T>, AppError> {
    ensure_project(pool, project_id).await?;
    let rows = sqlx::query(&format!(
        "SELECT payload FROM {table} WHERE project_id = $1 ORDER BY {order}"
    ))
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| serde_json::from_value(row.try_get("payload")?).map_err(AppError::from))
        .collect()
}

async fn evaluate_gate(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(request): Json<GateRequest>,
) -> Result<(StatusCode, Json<GateDecisionRecord>), AppError> {
    validate_id(&request.definition_id, "definition id")?;
    validate_subject(&request.subject)?;
    to_db_time(request.at, "at")?;
    let account = load_account(&state.pool, &project_id).await?;
    let decision = account.evaluate(&request);
    let id = Uuid::new_v4();
    let evaluated_at = now_unix_seconds();
    let target_fingerprint = fingerprint(&GateTarget::from(&request))?;
    let sequence = sqlx::query_scalar::<_, i64>(
        "INSERT INTO gate_decisions(\
            project_id, id, definition_id, stage, target_fingerprint, evaluated_at, request, decision\
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING sequence",
    )
    .bind(&project_id)
    .bind(id)
    .bind(&request.definition_id)
    .bind(request.stage.as_str())
    .bind(target_fingerprint)
    .bind(to_db_time(evaluated_at, "evaluatedAt")?)
    .bind(serde_json::to_value(&request)?)
    .bind(serde_json::to_value(&decision)?)
    .fetch_one(&state.pool)
    .await?;
    let record = GateDecisionRecord {
        sequence: from_db_time(sequence)?,
        id,
        request,
        decision,
        evaluated_at,
    };
    Ok((StatusCode::CREATED, Json(record)))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDecisionRecord {
    pub sequence: u64,
    pub id: Uuid,
    pub request: GateRequest,
    pub decision: GateDecision,
    pub evaluated_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GateTarget<'a> {
    definition_id: &'a str,
    stage: azimuth_assurance_domain::LifecycleStage,
    subject: &'a azimuth_assurance_domain::ExecutionSubject,
}

impl<'a> From<&'a GateRequest> for GateTarget<'a> {
    fn from(request: &'a GateRequest) -> Self {
        Self {
            definition_id: &request.definition_id,
            stage: request.stage,
            subject: &request.subject,
        }
    }
}

async fn list_gate_decisions(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<GateDecisionRecord>>, AppError> {
    load_gate_decisions(&state.pool, &project_id)
        .await
        .map(Json)
}

async fn list_current_gates(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<GateDecisionRecord>>, AppError> {
    let history = load_gate_decisions(&state.pool, &project_id).await?;
    Ok(Json(current_gate_records(history)))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkItem {
    decision_id: Uuid,
    definition_id: String,
    stage: azimuth_assurance_domain::LifecycleStage,
    subject: azimuth_assurance_domain::ExecutionSubject,
    reasons: Vec<azimuth_assurance_domain::GateReason>,
    work: Vec<azimuth_assurance_domain::WorkKind>,
    evaluated_at: u64,
}

async fn list_work_items(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<WorkItem>>, AppError> {
    let history = load_gate_decisions(&state.pool, &project_id).await?;
    let items = current_gate_records(history)
        .into_iter()
        .filter(|record| !record.decision.work.is_empty())
        .map(|record| WorkItem {
            decision_id: record.id,
            definition_id: record.request.definition_id,
            stage: record.request.stage,
            subject: record.request.subject,
            reasons: record.decision.reasons,
            work: record.decision.work,
            evaluated_at: record.evaluated_at,
        })
        .collect();
    Ok(Json(items))
}

fn current_gate_records(history: Vec<GateDecisionRecord>) -> Vec<GateDecisionRecord> {
    let mut seen = HashSet::new();
    history
        .into_iter()
        .filter(|record| {
            let key = fingerprint(&GateTarget::from(&record.request))
                .expect("gate target serialization is infallible");
            seen.insert(key)
        })
        .collect()
}

async fn load_gate_decisions(
    pool: &PgPool,
    project_id: &str,
) -> Result<Vec<GateDecisionRecord>, AppError> {
    ensure_project(pool, project_id).await?;
    let rows = sqlx::query(
        "SELECT sequence, id, evaluated_at, request, decision FROM gate_decisions \
         WHERE project_id = $1 ORDER BY sequence DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(GateDecisionRecord {
                sequence: from_db_time(row.try_get("sequence")?)?,
                id: row.try_get("id")?,
                request: serde_json::from_value(row.try_get("request")?)?,
                decision: serde_json::from_value(row.try_get("decision")?)?,
                evaluated_at: from_db_time(row.try_get("evaluated_at")?)?,
            })
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSnapshot {
    project: Project,
    account: AssuranceAccount,
    gate_decisions: Vec<GateDecisionRecord>,
}

async fn project_snapshot(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectSnapshot>, AppError> {
    let project = load_project(&state.pool, &project_id).await?;
    let account = load_account(&state.pool, &project_id).await?;
    let gate_decisions = load_gate_decisions(&state.pool, &project_id).await?;
    Ok(Json(ProjectSnapshot {
        project,
        account,
        gate_decisions,
    }))
}

async fn load_account(pool: &PgPool, project_id: &str) -> Result<AssuranceAccount, AppError> {
    Ok(AssuranceAccount {
        project_snapshots: load_records(pool, project_id, "project_model_snapshots", "sequence")
            .await?,
        definitions: load_records(
            pool,
            project_id,
            "evidence_definitions",
            "declared_at, logical_id, definition_fingerprint",
        )
        .await?,
        qualifications: load_records(pool, project_id, "qualifications", "qualified_at, id")
            .await?,
        observations: load_records(pool, project_id, "observations", "observed_at, id").await?,
        challenges: load_records(pool, project_id, "challenges", "observed_at, id").await?,
    })
}

async fn load_model_snapshot(
    pool: &PgPool,
    project_id: &str,
    snapshot_id: &str,
) -> Result<ProjectModelSnapshot, AppError> {
    ensure_project(pool, project_id).await?;
    let payload = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT payload FROM project_model_snapshots WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(snapshot_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::unprocessable("referenced project snapshot does not exist"))?;
    Ok(serde_json::from_value(payload)?)
}

async fn ensure_claim_contract_known(
    pool: &PgPool,
    project_id: &str,
    reference: &azimuth_assurance_domain::ClaimReference,
) -> Result<(), AppError> {
    let snapshots: Vec<ProjectModelSnapshot> =
        load_records(pool, project_id, "project_model_snapshots", "sequence").await?;
    if snapshots
        .iter()
        .any(|snapshot| snapshot.contains(reference))
    {
        Ok(())
    } else {
        Err(AppError::unprocessable(
            "definition references a claim contract absent from registered project snapshots",
        ))
    }
}

async fn load_project(pool: &PgPool, project_id: &str) -> Result<Project, AppError> {
    let row = sqlx::query("SELECT id, name, created_at FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("project not found"))?;
    Ok(Project {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        created_at: from_db_time(row.try_get("created_at")?)?,
    })
}

async fn ensure_project(pool: &PgPool, project_id: &str) -> Result<(), AppError> {
    validate_id(project_id, "project id")?;
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1)")
            .bind(project_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(AppError::not_found("project not found"))
    }
}

async fn load_definition_version(
    pool: &PgPool,
    project_id: &str,
    definition_id: &str,
    definition_fingerprint: &str,
) -> Result<EvidenceDefinition, AppError> {
    ensure_project(pool, project_id).await?;
    let payload = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT payload FROM evidence_definitions \
         WHERE project_id = $1 AND logical_id = $2 AND definition_fingerprint = $3",
    )
    .bind(project_id)
    .bind(definition_id)
    .bind(definition_fingerprint)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::unprocessable("referenced definition version does not exist"))?;
    Ok(serde_json::from_value(payload)?)
}

fn validate_definition(definition: &EvidenceDefinition) -> Result<(), AppError> {
    validate_id(&definition.claim.spec, "claim spec")?;
    validate_id(&definition.claim.claim, "claim id")?;
    validate_id(
        &definition.claim.contract_fingerprint,
        "claim contract fingerprint",
    )?;
    validate_text(&definition.assertion, "assertion")?;
    validate_text(&definition.scope, "scope")?;
    validate_text(&definition.quantification, "quantification")?;
    validate_text(&definition.oracle, "oracle")?;
    to_db_time(definition.declared_at, "declaredAt")?;
    Ok(())
}

fn validate_model_snapshot(
    snapshot: &ProjectModelSnapshot,
    project_id: &str,
) -> Result<(), AppError> {
    if snapshot.format != PROJECT_SNAPSHOT_FORMAT || snapshot.version != PROJECT_SNAPSHOT_VERSION {
        return Err(AppError::unprocessable(
            "unsupported assurance project snapshot format or version",
        ));
    }
    if snapshot.project != project_id {
        return Err(AppError::unprocessable(
            "snapshot project does not match the assurance account",
        ));
    }
    validate_id(&snapshot.id, "snapshot id")?;
    validate_id(&snapshot.model_fingerprint, "model fingerprint")?;
    let mut claims = HashSet::new();
    for contract in &snapshot.claims {
        validate_id(&contract.spec, "contract spec")?;
        validate_id(&contract.claim, "contract claim")?;
        validate_id(&contract.requirement, "contract requirement")?;
        validate_text(&contract.statement, "contract statement")?;
        if !matches!(contract.criticality.as_str(), "standard" | "critical") {
            return Err(AppError::unprocessable(
                "claim contracts are only valid for standard or critical claims",
            ));
        }
        if !claims.insert((contract.spec.as_str(), contract.claim.as_str())) {
            return Err(AppError::unprocessable(
                "snapshot contains a duplicate claim contract",
            ));
        }
        if contract.contract_fingerprint != contract.fingerprint() {
            return Err(AppError::unprocessable(
                "claim contract fingerprint does not match its content",
            ));
        }
    }
    if snapshot.id != snapshot.fingerprint() {
        return Err(AppError::unprocessable(
            "snapshot id does not match its content",
        ));
    }
    Ok(())
}

fn validate_id(value: &str, label: &str) -> Result<(), AppError> {
    if value.trim().is_empty() || value.len() > 200 {
        return Err(AppError::unprocessable(format!(
            "{label} must contain between 1 and 200 characters"
        )));
    }
    Ok(())
}

fn validate_text(value: &str, label: &str) -> Result<(), AppError> {
    if value.trim().is_empty() || value.len() > 4_000 {
        return Err(AppError::unprocessable(format!(
            "{label} must contain between 1 and 4000 characters"
        )));
    }
    Ok(())
}

fn validate_subject(subject: &azimuth_assurance_domain::ExecutionSubject) -> Result<(), AppError> {
    validate_id(&subject.project_snapshot, "project snapshot")?;
    validate_id(&subject.revision, "revision")?;
    for (value, label) in [
        (&subject.artifact_digest, "artifact digest"),
        (&subject.deployment_id, "deployment id"),
        (&subject.environment, "environment"),
        (&subject.cohort, "cohort"),
    ] {
        if let Some(value) = value {
            validate_id(value, label)?;
        }
    }
    Ok(())
}

fn fingerprint<T: Serialize>(value: &T) -> Result<String, AppError> {
    record_fingerprint(value).map_err(AppError::from)
}

fn resolve_replay(
    rows_affected: u64,
    stored_fingerprint: String,
    submitted_fingerprint: &str,
    record: &str,
) -> Result<bool, AppError> {
    if rows_affected == 1 {
        Ok(false)
    } else if stored_fingerprint == submitted_fingerprint {
        Ok(true)
    } else {
        Err(AppError::conflict(format!(
            "{record} identity already contains different content"
        )))
    }
}

fn to_db_time(value: u64, label: &str) -> Result<i64, AppError> {
    i64::try_from(value)
        .map_err(|_| AppError::unprocessable(format!("{label} exceeds supported range")))
}

fn from_db_time(value: i64) -> Result<u64, AppError> {
    u64::try_from(value).map_err(|_| AppError::internal("stored time is negative"))
}

fn now_unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time is after the Unix epoch")
        .as_secs()
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    title: &'static str,
    detail: String,
}

impl AppError {
    fn not_found(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            title: "Not found",
            detail: detail.into(),
        }
    }

    fn conflict(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            title: "Identity conflict",
            detail: detail.into(),
        }
    }

    fn unprocessable(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            title: "Invalid assurance record",
            detail: detail.into(),
        }
    }

    fn internal(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            title: "Assurance service failure",
            detail: detail.into(),
        }
    }
}

#[derive(Serialize)]
struct ProblemDetails<'a> {
    #[serde(rename = "type")]
    problem_type: &'a str,
    title: &'a str,
    status: u16,
    detail: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ProblemDetails {
            problem_type: "about:blank",
            title: self.title,
            status: self.status.as_u16(),
            detail: &self.detail,
        });
        (self.status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(error = %error, "database operation failed");
        Self::internal("database operation failed")
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        tracing::error!(error = %error, "assurance record serialization failed");
        Self::internal("assurance record serialization failed")
    }
}
