use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Selection {
    deployment_id: String,
    expected_revision: i64,
    reason: String,
}

pub(super) async fn select(
    State(state): State<LedgerState>,
    Path((project, environment)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<Selection>,
) -> Result<Json<Value>, LedgerError> {
    let publisher = state.authorize(&project, &headers, Some(Role::Publisher))?;
    azimuth::diag::validate_id(&environment, false).map_err(LedgerError::invalid)?;
    azimuth::diag::validate_id(&body.deployment_id, false).map_err(LedgerError::invalid)?;
    if body.expected_revision < 0
        || body.expected_revision == i64::MAX
        || body.reason.trim().is_empty()
        || body.reason.len() > 16384
    {
        return Err(LedgerError::invalid(
            "nonnegative expected_revision and bounded nonempty reason required",
        ));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|current-deployment|{environment}"))
        .execute(&mut *tx)
        .await?;
    let target: Option<Value> = sqlx::query_scalar(
        "SELECT assurance_record('deployment',project,deployment_id) FROM release_deployments WHERE project=$1 AND deployment_id=$2",
    )
    .bind(&project)
    .bind(&body.deployment_id)
    .fetch_optional(&mut *tx)
    .await?;
    let target = target.ok_or_else(LedgerError::not_found)?;
    if target["environment"].as_str() != Some(environment.as_str()) {
        return Err(LedgerError::invalid(
            "selected deployment must belong to the route environment",
        ));
    }
    let previous: Option<(i64,String,String,String)> = sqlx::query_as("SELECT revision,deployment_id,reason,selected_by FROM environment_deployment_selections WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1")
        .bind(&project).bind(&environment).fetch_optional(&mut *tx).await?;
    let revision = previous.as_ref().map(|v| v.0).unwrap_or(0);
    let replay = previous.as_ref().is_some_and(|(r, id, reason, by)| {
        *r == body.expected_revision + 1
            && id == &body.deployment_id
            && reason == &body.reason
            && by == &publisher
    });
    if replay {
        tx.commit().await?;
        return Ok(Json(
            json!({"project":project,"environment":environment,"revision":revision,"deployment_id":body.deployment_id,"replayed":true}),
        ));
    }
    if revision != body.expected_revision {
        return Err(LedgerError::conflict(
            "current deployment changed; reload its selection revision before selecting",
        ));
    }
    let next = revision + 1;
    sqlx::query("INSERT INTO environment_deployment_selections(project,environment,revision,deployment_id,reason,selected_by) VALUES($1,$2,$3,$4,$5,$6)")
        .bind(&project).bind(&environment).bind(next).bind(&body.deployment_id).bind(&body.reason).bind(&publisher).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(
        json!({"project":project,"environment":environment,"revision":next,"deployment_id":body.deployment_id,"replayed":false}),
    ))
}

pub(super) async fn current(
    State(state): State<LedgerState>,
    Path((project, environment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let value: Option<Value> = sqlx::query_scalar("SELECT jsonb_build_object('project',s.project,'environment',s.environment,'revision',s.revision,'deployment',assurance_record('deployment',d.project,d.deployment_id),'release_fingerprint',d.release_fingerprint,'reason',s.reason,'selected_by',s.selected_by,'selected_at',s.selected_at) FROM environment_deployment_selections s JOIN release_deployments d ON d.project=s.project AND d.deployment_id=s.deployment_id WHERE s.project=$1 AND s.environment=$2 ORDER BY s.revision DESC LIMIT 1")
        .bind(&project).bind(&environment).fetch_optional(&state.pool).await?;
    value.map(Json).ok_or_else(LedgerError::not_found)
}

pub(super) async fn deployment(
    State(state): State<LedgerState>,
    Path((project, deployment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let value: Option<Value> = sqlx::query_scalar("SELECT jsonb_build_object('deployment',assurance_record('deployment',project,deployment_id),'release_fingerprint',release_fingerprint,'submitted_by',submitted_by,'submitted_at',submitted_at) FROM release_deployments WHERE project=$1 AND deployment_id=$2")
        .bind(&project).bind(&deployment).fetch_optional(&state.pool).await?;
    value.map(Json).ok_or_else(LedgerError::not_found)
}

pub(super) async fn list(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows = sqlx::query("SELECT environment AS fingerprint,jsonb_build_object('environment',environment,'revision',revision,'deployment_id',deployment_id,'selected_by',selected_by,'selected_at',selected_at) AS payload FROM (SELECT DISTINCT ON(environment) environment,revision,deployment_id,selected_by,selected_at FROM environment_deployment_selections WHERE project=$1 AND environment>$2 ORDER BY environment,revision DESC) selected ORDER BY environment LIMIT $3")
        .bind(&project).bind(&page.after).bind(limit+1).fetch_all(&state.pool).await?;
    release::paged(rows, limit)
}
