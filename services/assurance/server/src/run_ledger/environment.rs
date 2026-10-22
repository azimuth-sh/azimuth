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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HistoryPage {
    limit: Option<i64>,
    #[serde(default)]
    page_token: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CandidateLimit {
    limit: Option<i64>,
}

const ENVIRONMENT_DEPLOYMENTS: &str = "SELECT o.deployment_id,d.release_fingerprint,r.label,r.model_revision,min(o.observed_at_ms) AS first_observed_ms,max(o.observed_at_ms) AS last_observed_ms,(array_agg(o.fingerprint ORDER BY o.revision DESC))[1] AS last_observation FROM deployment_observations o JOIN release_deployments d ON d.project=o.project AND d.deployment_id=o.deployment_id JOIN assurance_releases r ON r.project=d.project AND r.fingerprint=d.release_fingerprint WHERE o.project=$1 AND o.environment=$2 AND o.state='complete' GROUP BY o.deployment_id,d.release_fingerprint,r.label,r.model_revision";

const CURRENT_DEPLOYMENT: &str = "(SELECT deployment_id FROM environment_deployment_selections WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1)";

pub(super) async fn history(
    State(state): State<LedgerState>,
    Path((project, environment)): Path<(String, String)>,
    Query(page): Query<HistoryPage>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit.unwrap_or(25);
    if !(1..=100).contains(&limit) || page.page_token.len() > 1024 {
        return Err(LedgerError::invalid(
            "limit must be 1..100 and page_token at most 1024 bytes",
        ));
    }
    let (after_ms, after_id) = match page.page_token.split_once('|') {
        Some((ms, id)) => (
            ms.parse::<i64>().map_err(|_| {
                LedgerError::invalid("page_token is not a deployment history cursor")
            })?,
            id.to_owned(),
        ),
        None if page.page_token.is_empty() => (i64::MAX, String::new()),
        None => {
            return Err(LedgerError::invalid(
                "page_token is not a deployment history cursor",
            ))
        }
    };
    let rows = sqlx::query(&format!("SELECT history.*,(assurance_record_field('deployment_observation',$1,history.last_observation,'rollout_id') #>> '{{}}') AS rollout_id,history.deployment_id=(SELECT deployment_id FROM environment_deployment_selections WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1) AS current FROM ({ENVIRONMENT_DEPLOYMENTS}) history WHERE (history.first_observed_ms,history.deployment_id) < ($3,$4) ORDER BY history.first_observed_ms DESC,history.deployment_id DESC LIMIT $5"))
        .bind(&project).bind(&environment).bind(after_ms).bind(&after_id).bind(limit + 1).fetch_all(&state.pool).await?;
    let mut deployments = Vec::new();
    for row in rows.iter().take(limit as usize) {
        deployments.push(json!({
            "deployment_id":row.try_get::<String,_>("deployment_id")?,
            "release_fingerprint":row.try_get::<String,_>("release_fingerprint")?,
            "label":row.try_get::<String,_>("label")?,
            "first_observed_ms":row.try_get::<i64,_>("first_observed_ms")?,
            "last_observed_ms":row.try_get::<i64,_>("last_observed_ms")?,
            "rollout_id":row.try_get::<Option<String>,_>("rollout_id")?,
            "current":row.try_get::<Option<bool>,_>("current")?.unwrap_or(false),
        }));
    }
    let next_page_token = if rows.len() > limit as usize {
        rows.get(limit as usize - 1)
            .map(|row| {
                Ok::<_, sqlx::Error>(format!(
                    "{}|{}",
                    row.try_get::<i64, _>("first_observed_ms")?,
                    row.try_get::<String, _>("deployment_id")?
                ))
            })
            .transpose()?
    } else {
        None
    };
    Ok(Json(
        json!({"project":project,"environment":environment,"deployments":deployments,"next_page_token":next_page_token}),
    ))
}

pub(super) async fn delivery_gap(
    State(state): State<LedgerState>,
    Path((project, environment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let current = sqlx::query("SELECT d.deployment_id,d.release_fingerprint,r.model_revision FROM environment_deployment_selections s JOIN release_deployments d ON d.project=s.project AND d.deployment_id=s.deployment_id JOIN assurance_releases r ON r.project=d.project AND r.fingerprint=d.release_fingerprint WHERE s.project=$1 AND s.environment=$2 ORDER BY s.revision DESC LIMIT 1")
        .bind(&project).bind(&environment).fetch_optional(&state.pool).await?
        .ok_or_else(LedgerError::not_found)?;
    let main = sqlx::query("SELECT commit_sha,main_sequence,model_revision FROM main_publications WHERE project=$1 ORDER BY main_sequence DESC LIMIT 1")
        .bind(&project).fetch_optional(&state.pool).await?
        .ok_or_else(LedgerError::not_found)?;
    let deployment_id: String = current.try_get("deployment_id")?;
    let release_revision: String = current.try_get("model_revision")?;
    let main_revision: String = main.try_get("model_revision")?;
    let requirements_of = |revision: String| {
        let pool = state.pool.clone();
        let project = project.clone();
        async move {
            let value: Option<Value> = sqlx::query_scalar(
                "SELECT assurance_record_field('authority',$1,$2,'requirements')",
            )
            .bind(project)
            .bind(revision)
            .fetch_one(&pool)
            .await?;
            Ok::<_, LedgerError>(
                value
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default(),
            )
        }
    };
    let released = requirements_of(release_revision.clone()).await?;
    let merged = requirements_of(main_revision.clone()).await?;
    let statuses: std::collections::BTreeMap<String, String> = sqlx::query_as::<_, (String, String)>("SELECT requirement,status FROM assessment_requirements WHERE project=$1 AND deployment=$2")
        .bind(&project).bind(&deployment_id).fetch_all(&state.pool).await?.into_iter().collect();
    let find = |list: &[Value], id: &str| list.iter().find(|r| r["id"] == id).cloned();
    let mut requirements = Vec::new();
    for requirement in &merged {
        let id = requirement["id"].as_str().unwrap_or_default();
        let difference = match find(&released, id) {
            None => "added",
            Some(previous) if previous["fingerprint"] != requirement["fingerprint"] => "changed",
            Some(_) => continue,
        };
        requirements.push(json!({"id":id,"difference":difference,"criticality":requirement["criticality"],"release_status":statuses.get(id)}));
    }
    for requirement in &released {
        let id = requirement["id"].as_str().unwrap_or_default();
        if find(&merged, id).is_none() {
            requirements.push(json!({"id":id,"difference":"removed","criticality":requirement["criticality"],"release_status":statuses.get(id)}));
        }
    }
    Ok(Json(json!({
        "environment":environment,
        "deployment_id":deployment_id,
        "release_fingerprint":current.try_get::<String,_>("release_fingerprint")?,
        "release_model_revision":release_revision,
        "main_model_revision":main_revision,
        "main_commit":main.try_get::<String,_>("commit_sha")?,
        "main_sequence":main.try_get::<i64,_>("main_sequence")?,
        "requirements":requirements,
    })))
}

pub(super) async fn rollback_candidates(
    State(state): State<LedgerState>,
    Path((project, environment)): Path<(String, String)>,
    Query(page): Query<CandidateLimit>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit.unwrap_or(5);
    if !(1..=50).contains(&limit) {
        return Err(LedgerError::invalid("limit must be 1..50"));
    }
    let rows = sqlx::query(&format!("SELECT history.deployment_id,history.release_fingerprint,history.label,history.first_observed_ms,history.last_observed_ms,(assurance_record_field('deployment_observation',$1,history.last_observation,'rollout_id') #>> '{{}}') AS rollout_id,a.evaluated_at_ms AS last_status_at_ms,(SELECT count(*) FROM jsonb_array_elements(assurance_record_field('authority',$1,history.model_revision,'requirements')) q WHERE q->>'criticality'<>'routine') AS non_routine_requirements FROM ({ENVIRONMENT_DEPLOYMENTS}) history JOIN deployment_assessments a ON a.project=$1 AND a.deployment=history.deployment_id WHERE history.deployment_id IS DISTINCT FROM {CURRENT_DEPLOYMENT} AND a.evaluated_at_ms IS NOT NULL AND NOT EXISTS(SELECT 1 FROM jsonb_array_elements(assurance_record_field('authority',$1,history.model_revision,'requirements')) q WHERE q->>'criticality'<>'routine' AND NOT EXISTS(SELECT 1 FROM assessment_requirements s WHERE s.project=$1 AND s.deployment=history.deployment_id AND s.requirement=q->>'id' AND s.status='supported')) ORDER BY history.last_observed_ms DESC,history.deployment_id DESC LIMIT $3"))
        .bind(&project).bind(&environment).bind(limit).fetch_all(&state.pool).await?;
    let mut candidates = Vec::new();
    for row in rows {
        candidates.push(json!({
            "deployment_id":row.try_get::<String,_>("deployment_id")?,
            "release_fingerprint":row.try_get::<String,_>("release_fingerprint")?,
            "label":row.try_get::<String,_>("label")?,
            "first_observed_ms":row.try_get::<i64,_>("first_observed_ms")?,
            "last_observed_ms":row.try_get::<i64,_>("last_observed_ms")?,
            "rollout_id":row.try_get::<Option<String>,_>("rollout_id")?,
            "last_status_at_ms":row.try_get::<i64,_>("last_status_at_ms")?,
            "non_routine_requirements":row.try_get::<i64,_>("non_routine_requirements")?,
        }));
    }
    Ok(Json(
        json!({"project":project,"environment":environment,"candidates":candidates}),
    ))
}
