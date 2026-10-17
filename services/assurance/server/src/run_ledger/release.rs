use super::*;

pub(super) async fn publish(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let submitter = state.authorize(&project, &headers, Some(Role::Publisher))?;
    let native = core_json(&body)?;
    azimuth::release::verify(&native).map_err(LedgerError::invalid)?;
    let payload = to_value(&native)?;
    if payload["project"].as_str() != Some(&project) {
        return Err(LedgerError::invalid("publication project must match route"));
    }
    let release = &payload["release"];
    let fingerprint = text(release, "fingerprint")?;
    let revision = text(release, "model_revision")?;
    let snapshot = &payload["snapshot"];
    let mut tx = state.pool.begin().await?;
    // Publication and direct ingestion must serialize on the same Run histories.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|release-publication"))
        .execute(&mut *tx)
        .await?;
    let existing: Option<Value> = sqlx::query_scalar(
        "SELECT payload FROM assurance_releases WHERE project=$1 AND fingerprint=$2",
    )
    .bind(&project)
    .bind(fingerprint)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(existing) = existing {
        if existing != payload {
            return Err(LedgerError::conflict("release publication is immutable"));
        }
        tx.commit().await?;
        return Ok((
            StatusCode::OK,
            Json(json!({"fingerprint":fingerprint,"replayed":true})),
        ));
    }
    sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,payload,registered_by) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
        .bind(&project).bind(revision).bind(&snapshot["authority"]).bind(&submitter).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(&project).bind(revision).bind(&submitter).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO assurance_releases(project,fingerprint,model_revision,payload,submitted_by) VALUES($1,$2,$3,$4,$5)")
        .bind(&project).bind(fingerprint).bind(revision).bind(&payload).bind(&submitter).execute(&mut *tx).await?;
    for change in array(release, "changes")? {
        sqlx::query("INSERT INTO release_changes(project,release_fingerprint,change_id,change_revision,payload) VALUES($1,$2,$3,$4,$5)")
            .bind(&project).bind(fingerprint).bind(text(change,"id")?).bind(text(change,"revision")?).bind(change).execute(&mut *tx).await?;
    }
    let mut bundles = array(snapshot, "bundles")?
        .iter()
        .map(|v| parse_bundle(&v.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    bundles.sort_by(|a, b| (&a.run_id, a.bundle_revision).cmp(&(&b.run_id, b.bundle_revision)));
    for bundle in bundles {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("{project}|{}", bundle.run_id))
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query(
            "SELECT payload FROM run_bundles WHERE project=$1 AND run_id=$2 ORDER BY revision",
        )
        .bind(&project)
        .bind(&bundle.run_id)
        .fetch_all(&mut *tx)
        .await?;
        let mut history = rows
            .iter()
            .map(|r| {
                let v: Value = r.try_get("payload")?;
                parse_bundle(&v.to_string())
            })
            .collect::<Result<Vec<_>, LedgerError>>()?;
        if let Some(previous) = history
            .iter()
            .find(|b| b.bundle_revision == bundle.bundle_revision)
        {
            if previous.bundle_fingerprint != bundle.bundle_fingerprint {
                return Err(LedgerError::conflict(
                    "Run revision has different immutable content",
                ));
            }
        } else {
            if bundle.bundle_revision as u128 != history.len() as u128 {
                return Err(LedgerError::conflict(
                    "publication lacks immediate Run predecessor",
                ));
            }
            history.push(bundle.clone());
            if !run::verify_set(&history).is_empty() {
                return Err(LedgerError::invalid(
                    "published Run history is inconsistent",
                ));
            }
            sqlx::query("INSERT INTO run_bundles(project,run_id,revision,fingerprint,subject_fingerprint,model_fingerprint,payload) VALUES($1,$2,$3,$4,$5,$6,$7)")
                .bind(&project).bind(&bundle.run_id).bind(bundle.bundle_revision as i64).bind(&bundle.bundle_fingerprint)
                .bind(&bundle.subject_fingerprint).bind(&bundle.plan.model_fingerprint).bind(to_value(&run::to_json(&bundle))?).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO release_run_submissions(project,release_fingerprint,run_fingerprint,submitted_by) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
            .bind(&project).bind(fingerprint).bind(&bundle.bundle_fingerprint).bind(&submitter).execute(&mut *tx).await?;
    }
    for record in array(snapshot, "reviews")? {
        let native = core_json(record.to_string().as_bytes())?;
        let target = assurance_review::review_target(&native).map_err(LedgerError::invalid)?;
        sqlx::query("INSERT INTO assurance_reviews(project,fingerprint,target_fingerprint,payload,submitted_by,provenance_basis) VALUES($1,$2,$3,$4,$5,'publisher-transported-assertion') ON CONFLICT DO NOTHING")
            .bind(&project).bind(text(record,"fingerprint")?).bind(run::canonical_fingerprint(&target).map_err(LedgerError::invalid)?).bind(record).bind(&submitter).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO model_revision_reviews(project,revision_fingerprint,review_fingerprint,submitted_by) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
            .bind(&project).bind(revision).bind(text(record,"fingerprint")?).bind(&submitter).execute(&mut *tx).await?;
    }
    for deployment in array(release, "deployments")? {
        insert_deployment(&mut tx, &project, fingerprint, deployment, &submitter).await?;
    }
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(
            json!({"project":project,"fingerprint":fingerprint,"model_revision":revision,"status":"candidate","submitted_by":submitter,"review_provenance":"publisher-transported-assertion"}),
        ),
    ))
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, LedgerError> {
    value[key]
        .as_str()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| LedgerError::invalid(format!("{key} requires nonempty text")))
}
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, LedgerError> {
    value[key]
        .as_array()
        .ok_or_else(|| LedgerError::invalid(format!("{key} requires an array")))
}

pub(super) async fn list(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows = sqlx::query("SELECT fingerprint,jsonb_build_object('fingerprint',fingerprint,'release',payload->'release','submitted_by',submitted_by,'submitted_at',submitted_at,'accepted',EXISTS(SELECT 1 FROM release_acceptances a WHERE a.project=r.project AND a.release_fingerprint=r.fingerprint)) AS payload FROM assurance_releases r WHERE project=$1 AND fingerprint>$2 ORDER BY fingerprint LIMIT $3")
        .bind(&project).bind(&page.after).bind(limit+1).fetch_all(&state.pool).await?;
    paged(rows, limit)
}
pub(super) async fn show(
    State(state): State<LedgerState>,
    Path((project, fingerprint)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let record: Option<Value> = sqlx::query_scalar("SELECT jsonb_build_object('publication',payload,'submitted_by',submitted_by,'submitted_at',submitted_at,'acceptance',(SELECT to_jsonb(a) FROM release_acceptances a WHERE a.project=r.project AND a.release_fingerprint=r.fingerprint),'deployments',COALESCE((SELECT jsonb_agg(d.payload) FROM release_deployments d WHERE d.project=r.project AND d.release_fingerprint=r.fingerprint),'[]'::jsonb)) FROM assurance_releases r WHERE project=$1 AND fingerprint=$2")
        .bind(&project).bind(&fingerprint).fetch_optional(&state.pool).await?;
    Ok(Json(record.ok_or_else(LedgerError::not_found)?))
}
fn paged(rows: Vec<sqlx::postgres::PgRow>, limit: i64) -> Result<Json<Value>, LedgerError> {
    let next = if rows.len() > limit as usize {
        Some(rows[limit as usize - 1].try_get::<String, _>("fingerprint")?)
    } else {
        None
    };
    let items = rows
        .into_iter()
        .take(limit as usize)
        .map(|r| r.try_get::<Value, _>("payload"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(json!({"items":items,"next":next})))
}
pub(super) async fn changes(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows=sqlx::query("SELECT change_id AS fingerprint,jsonb_build_object('id',change_id,'revisions',jsonb_agg(jsonb_build_object('release',release_fingerprint,'change',payload))) AS payload FROM release_changes WHERE project=$1 AND change_id>$2 GROUP BY change_id ORDER BY change_id LIMIT $3")
        .bind(&project).bind(&page.after).bind(limit+1).fetch_all(&state.pool).await?;
    paged(rows, limit)
}
pub(super) async fn deployments(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows=sqlx::query("SELECT deployment_id AS fingerprint,jsonb_build_object('deployment',payload,'release_fingerprint',release_fingerprint,'submitted_by',submitted_by,'submitted_at',submitted_at) AS payload FROM release_deployments WHERE project=$1 AND deployment_id>$2 ORDER BY deployment_id LIMIT $3")
        .bind(&project).bind(&page.after).bind(limit+1).fetch_all(&state.pool).await?;
    paged(rows, limit)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeploymentSubmission {
    release_fingerprint: String,
    deployment: Value,
}
pub(super) async fn deploy(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    Json(body): Json<DeploymentSubmission>,
) -> Result<Json<Value>, LedgerError> {
    let submitter = state.authorize(&project, &headers, Some(Role::Publisher))?;
    let mut tx = state.pool.begin().await?;
    let release: Option<Value> = sqlx::query_scalar(
        "SELECT payload->'release' FROM assurance_releases WHERE project=$1 AND fingerprint=$2",
    )
    .bind(&project)
    .bind(&body.release_fingerprint)
    .fetch_optional(&mut *tx)
    .await?;
    let release = release.ok_or_else(LedgerError::not_found)?;
    let allowed = array(&release, "artifacts")?
        .iter()
        .map(|v| text(v, "digest").map(str::to_owned))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    azimuth::release::verify_deployment(
        &core_json(body.deployment.to_string().as_bytes())?,
        &allowed,
    )
    .map_err(LedgerError::invalid)?;
    insert_deployment(
        &mut tx,
        &project,
        &body.release_fingerprint,
        &body.deployment,
        &submitter,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"recorded":true,"release_fingerprint":body.release_fingerprint}),
    ))
}
async fn insert_deployment(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    release: &str,
    deployment: &Value,
    submitter: &str,
) -> Result<(), LedgerError> {
    let id = text(deployment, "id")?;
    azimuth::diag::validate_id(id, false).map_err(LedgerError::invalid)?;
    text(deployment, "environment")?;
    let configuration = text(deployment, "configuration_fingerprint")?;
    if !configuration.starts_with("sha256:") || configuration.len() != 71 {
        return Err(LedgerError::invalid(
            "deployment requires exact configuration fingerprint",
        ));
    }
    if array(deployment, "artifacts")?.is_empty() {
        return Err(LedgerError::invalid("deployment artifacts missing"));
    }
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|deployment|{id}"))
        .execute(&mut **tx)
        .await?;
    let existing: Option<(String,Value)>=sqlx::query_as("SELECT release_fingerprint,payload FROM release_deployments WHERE project=$1 AND deployment_id=$2")
        .bind(project).bind(id).fetch_optional(&mut **tx).await?;
    if let Some((previous, payload)) = existing {
        if previous != release || payload != *deployment {
            return Err(LedgerError::conflict("deployment identity is immutable"));
        }
        return Ok(());
    }
    sqlx::query("INSERT INTO release_deployments(project,deployment_id,release_fingerprint,payload,submitted_by) VALUES($1,$2,$3,$4,$5)")
        .bind(project).bind(id).bind(release).bind(deployment).bind(submitter).execute(&mut **tx).await?;
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Acceptance {
    expected_accepted_model_revision: String,
    reason: String,
}
pub(super) async fn accept(
    State(state): State<LedgerState>,
    Path((project, fingerprint)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<Acceptance>,
) -> Result<Json<Value>, LedgerError> {
    let owner = state.authorize(&project, &headers, Some(Role::Owner))?;
    if body.reason.trim().is_empty() || body.reason.len() > 16384 {
        return Err(LedgerError::invalid(
            "bounded acceptance rationale required",
        ));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|authority"))
        .execute(&mut *tx)
        .await?;
    let current: Option<String> =
        sqlx::query_scalar("SELECT fingerprint FROM current_authorities WHERE project=$1")
            .bind(&project)
            .fetch_optional(&mut *tx)
            .await?;
    if current.as_deref() != Some(body.expected_accepted_model_revision.as_str()) {
        return Err(LedgerError::conflict(
            "accepted model changed; reassess release acceptance",
        ));
    }
    let already:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM release_acceptances WHERE project=$1 AND release_fingerprint=$2)").bind(&project).bind(&fingerprint).fetch_one(&mut *tx).await?;
    if already {
        return Err(LedgerError::conflict("release already accepted"));
    }
    let payload: Option<Value> = sqlx::query_scalar(
        "SELECT payload->'release' FROM assurance_releases WHERE project=$1 AND fingerprint=$2",
    )
    .bind(&project)
    .bind(&fingerprint)
    .fetch_optional(&mut *tx)
    .await?;
    let release = payload.ok_or_else(LedgerError::not_found)?;
    if text(&release, "baseline_model_revision")? != body.expected_accepted_model_revision {
        return Err(LedgerError::conflict(
            "release does not extend current accepted baseline",
        ));
    }
    let revision = text(&release, "model_revision")?;
    let previous_composition:Option<Value>=sqlx::query_scalar("SELECT r.payload->'release'->'composition' FROM release_acceptances a JOIN assurance_releases r ON r.project=a.project AND r.fingerprint=a.release_fingerprint WHERE a.project=$1 AND r.model_revision=$2 ORDER BY a.accepted_at DESC LIMIT 1")
        .bind(&project).bind(&body.expected_accepted_model_revision).fetch_optional(&mut *tx).await?;
    if let Some(previous) = previous_composition {
        if !previous.is_null()
            && previous["content_fingerprint"]
                != release["composition"]["baseline_content_fingerprint"]
        {
            return Err(LedgerError::conflict(
                "release composition does not extend accepted content",
            ));
        }
    }
    sqlx::query("INSERT INTO release_acceptances(project,release_fingerprint,previous_model_revision,reason,accepted_by) VALUES($1,$2,$3,$4,$5)")
        .bind(&project).bind(&fingerprint).bind(&body.expected_accepted_model_revision).bind(&body.reason).bind(&owner).execute(&mut *tx).await?;
    sqlx::query("UPDATE current_authorities SET fingerprint=$2 WHERE project=$1")
        .bind(&project)
        .bind(revision)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO authority_selections(project,fingerprint,selected_by) VALUES($1,$2,$3)",
    )
    .bind(&project)
    .bind(revision)
    .bind(&owner)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"release_fingerprint":fingerprint,"accepted_model_revision":revision,"accepted_by":owner}),
    ))
}
