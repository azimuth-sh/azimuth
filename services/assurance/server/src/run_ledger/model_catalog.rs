use super::*;

pub(super) async fn store(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    revision: &str,
    catalog: &Value,
    submitter: &str,
    provenance: &str,
) -> Result<bool, LedgerError> {
    let authority: Option<Value> = sqlx::query_scalar("SELECT a.payload FROM assurance_authorities a JOIN model_revisions r USING(project,fingerprint) WHERE r.project=$1 AND r.fingerprint=$2")
        .bind(project).bind(revision).fetch_optional(&mut **tx).await?;
    let authority = authority.ok_or_else(LedgerError::not_found)?;
    let authority = core_json(authority.to_string().as_bytes())?;
    let native = core_json(catalog.to_string().as_bytes())?;
    azimuth::release::verify_catalog(&native, &authority).map_err(LedgerError::invalid)?;
    let fingerprint = run::canonical_fingerprint(&native).map_err(LedgerError::invalid)?;
    let inserted = sqlx::query("INSERT INTO model_revision_catalogs(project,revision_fingerprint,catalog_fingerprint,payload,submitted_by,provenance_basis) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING")
        .bind(project).bind(revision).bind(&fingerprint).bind(catalog).bind(submitter).bind(provenance)
        .execute(&mut **tx).await?.rows_affected() == 1;
    let existing: Value = sqlx::query_scalar(
        "SELECT payload FROM model_revision_catalogs WHERE project=$1 AND revision_fingerprint=$2",
    )
    .bind(project)
    .bind(revision)
    .fetch_one(&mut **tx)
    .await?;
    if existing != *catalog {
        return Err(LedgerError::conflict(
            "revision catalog is immutable; conflicting catalog rejected",
        ));
    }
    Ok(inserted)
}

pub(super) async fn register(
    State(state): State<LedgerState>,
    Path((project, revision)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let owner = state.authorize(&project, &headers, Some(Role::Owner))?;
    let native = core_json(&body)?;
    let payload = to_value(&native)?;
    let mut tx = state.pool.begin().await?;
    let inserted = store(
        &mut tx,
        &project,
        &revision,
        &payload,
        &owner,
        "owner-registration",
    )
    .await?;
    tx.commit().await?;
    Ok((
        if inserted {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(json!({"project":project,"model_revision":revision,"replayed":!inserted})),
    ))
}

pub(super) async fn show(
    State(state): State<LedgerState>,
    Path((project, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let value: Option<Value> = sqlx::query_scalar("SELECT jsonb_build_object('project',project,'model_revision',revision_fingerprint,'fingerprint',catalog_fingerprint,'catalog',payload,'submitted_by',submitted_by,'provenance_basis',provenance_basis,'submitted_at',submitted_at) FROM model_revision_catalogs WHERE project=$1 AND revision_fingerprint=$2")
        .bind(&project).bind(&revision).fetch_optional(&state.pool).await?;
    value.map(Json).ok_or_else(LedgerError::not_found)
}
