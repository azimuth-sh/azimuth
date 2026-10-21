use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ContractSubmission {
    contract: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaptureSubmission {
    capture: Value,
}

fn submission<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, LedgerError> {
    let value = to_value(&core_json(body)?)?;
    serde_json::from_value(value).map_err(|error| LedgerError::invalid(error.to_string()))
}

async fn now(connection: &mut sqlx::PgConnection) -> Result<i64, LedgerError> {
    Ok(
        sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
            .fetch_one(connection)
            .await?,
    )
}
fn fingerprint(value: &Value) -> Result<String, LedgerError> {
    run::canonical_fingerprint(&core_json(value.to_string().as_bytes())?)
        .map_err(LedgerError::invalid)
}

pub(super) async fn attribute_run(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    bundle: &str,
    run: &str,
    submitter: &str,
) -> Result<(), LedgerError> {
    let key = format!("{bundle}:{submitter}");
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assurance_records WHERE kind='run_attribution' AND project=$1 AND record_key=$2)")
        .bind(project).bind(&key).fetch_one(&mut **tx).await?;
    if !exists {
        let time = now(&mut **tx).await?;
        relational::store(&mut **tx, "run_attribution", project, &key,
            &json!({"run":run,"bundle_fingerprint":bundle,"submitted_by":submitter,"received_at_ms":time})).await?;
    }
    Ok(())
}

pub(super) async fn register_contract(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let owner = state.authorize(&project, &headers, Some(Role::Owner))?;
    let body: ContractSubmission = submission(&body)?;
    azimuth::deployment_status::validate_context_contract(&core_json(
        body.contract.to_string().as_bytes(),
    )?)
    .map_err(LedgerError::invalid)?;
    let key = body.contract["fingerprint"].as_str().unwrap();
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|context-contract|{key}"))
        .execute(&mut *tx)
        .await?;
    let existing: Option<Value> = sqlx::query_scalar("SELECT assurance_record('context_contract',project,record_key) FROM assurance_records WHERE kind='context_contract' AND project=$1 AND record_key=$2")
        .bind(&project).bind(key).fetch_optional(&mut *tx).await?;
    let replayed = existing.is_some();
    if let Some(existing) = existing {
        if existing["contract"] != body.contract {
            return Err(LedgerError::conflict(
                "context contract fingerprint is immutable",
            ));
        }
    } else {
        let time = now(&mut *tx).await?;
        relational::store(
            &mut *tx,
            "context_contract",
            &project,
            key,
            &json!({"contract":body.contract,"registered_by":owner,"registered_at_ms":time}),
        )
        .await?;
    }
    tx.commit().await?;
    Ok((
        if replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(json!({"fingerprint":key,"replayed":replayed})),
    ))
}

async fn records(
    connection: &mut sqlx::PgConnection,
    project: &str,
    kind: &str,
) -> Result<Vec<Value>, LedgerError> {
    let values: Vec<Value> = sqlx::query_scalar("SELECT assurance_record(kind,project,record_key) FROM assurance_records WHERE project=$1 AND kind=$2 ORDER BY record_key LIMIT 5001")
        .bind(project).bind(kind).fetch_all(connection).await?;
    if values.len() > 5000 || values.iter().map(|v| v.to_string().len()).sum::<usize>() > 33_554_432
    {
        return Err(LedgerError::invalid(
            "context snapshot exceeds 5000 records or 32 MiB; no partial assessment inferred",
        ));
    }
    Ok(values)
}

pub(super) async fn contracts(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let mut connection = state.pool.acquire().await?;
    Ok(Json(
        json!({"records":records(&mut connection,&project,"context_contract").await?}),
    ))
}

pub(super) async fn captures(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let mut connection = state.pool.acquire().await?;
    Ok(Json(
        json!({"records":records(&mut connection,&project,"context_capture").await?}),
    ))
}

pub(super) async fn record_capture(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let mut body: CaptureSubmission = submission(&body)?;
    let deployment = body.capture["kind"] == "deployment";
    let principal = if deployment {
        state.authorize(&project, &headers, Some(Role::Publisher))?
    } else {
        match state.authorize(&project, &headers, Some(Role::Producer)) {
            Ok(identity) => identity,
            Err(_) => state.authorize(&project, &headers, Some(Role::Publisher))?,
        }
    };
    if !body.capture.is_object() || body.capture.get("trusted").is_some() {
        return Err(LedgerError::invalid(
            "capture must omit server-controlled trusted field",
        ));
    }
    let key = fingerprint(&body.capture)?;
    body.capture
        .as_object_mut()
        .unwrap()
        .insert("trusted".into(), json!(true));
    let mut tx = state.pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx)
        .await?;
    let contract_records = records(&mut *tx, &project, "context_contract").await?;
    let contracts = contract_records
        .iter()
        .map(|v| core_json(v["contract"].to_string().as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;
    azimuth::deployment_status::validate_capture(
        &core_json(body.capture.to_string().as_bytes())?,
        &contracts,
    )
    .map_err(LedgerError::invalid)?;
    let captured = &body.capture;
    let time = now(&mut *tx).await?;
    if captured["finished_at_ms"]
        .as_i64()
        .is_none_or(|v| v > time + 5000)
    {
        return Err(LedgerError::invalid("capture is future-dated"));
    }
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|context-capture|{key}"))
        .execute(&mut *tx)
        .await?;
    let existing: Option<Value> = sqlx::query_scalar("SELECT assurance_record('context_capture',project,record_key) FROM assurance_records WHERE kind='context_capture' AND project=$1 AND record_key=$2")
        .bind(&project).bind(&key).fetch_optional(&mut *tx).await?;
    let replayed = existing.is_some();
    if let Some(existing) = existing {
        if existing["capture"] != body.capture || existing["submitted_by"] != principal {
            return Err(LedgerError::conflict(
                "context capture has conflicting attribution",
            ));
        }
    } else {
        if deployment {
            verify_deployment_capture(&mut tx, &project, captured, time).await?;
        } else {
            verify_execution_capture(&mut tx, &project, captured, &principal).await?;
        }
        relational::store(&mut *tx,"context_capture",&project,&key,
            &json!({"fingerprint":key,"capture":body.capture,"submitted_by":principal,"received_at_ms":time})).await?;
    }
    tx.commit().await?;
    Ok((
        if replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(json!({"fingerprint":key,"replayed":replayed})),
    ))
}

async fn verify_execution_capture(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    capture: &Value,
    principal: &str,
) -> Result<(), LedgerError> {
    let fp = capture["bundle_fingerprint"].as_str().unwrap();
    let bundle: Option<Value> = sqlx::query_scalar("SELECT assurance_record('run',project,fingerprint) FROM run_bundles WHERE project=$1 AND fingerprint=$2 AND run_id=$3 AND verification_target_fingerprint=$4")
        .bind(project).bind(fp).bind(capture["run"].as_str()).bind(capture["verification_target_fingerprint"].as_str())
        .fetch_optional(&mut **tx).await?;
    let bundle = bundle.ok_or_else(|| {
        LedgerError::invalid("capture requires the exact stored Run and Verification Target")
    })?;
    let attributed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assurance_records WHERE kind='run_attribution' AND project=$1 AND record_key=$2) OR EXISTS(SELECT 1 FROM release_run_submissions WHERE project=$1 AND run_fingerprint=$3 AND submitted_by=$4)")
        .bind(project).bind(format!("{fp}:{principal}")).bind(fp).bind(principal).fetch_one(&mut **tx).await?;
    if !attributed {
        return Err(LedgerError::forbidden());
    }
    let execution = if let Some(activity) = capture["activity"].as_str() {
        bundle["activities"]
            .as_array()
            .and_then(|activities| activities.iter().find(|entry| entry["id"] == activity))
            .ok_or_else(|| {
                LedgerError::invalid("capture references an activity absent from the exact Run")
            })?
    } else {
        &bundle
    };
    if capture["started_at_ms"].as_u64() > execution["started_at_ms"].as_u64()
        || capture["finished_at_ms"].as_u64() < execution["finished_at_ms"].as_u64()
    {
        return Err(LedgerError::invalid(
            "capture must enclose the exact execution interval",
        ));
    }
    for (name, context) in capture["before"]["report"]["contexts"].as_object().unwrap() {
        let fp = context["contract_revision"].as_str().unwrap();
        let context_key = match capture["activity"].as_str() {
            Some(activity) => format!("context/{fp}/activity/{activity}"),
            None => format!("context/{fp}"),
        };
        if capture["after"]["report"]["contexts"].get(name) != Some(context)
            || bundle["actual_selection"]["execution_context"][context_key] != fingerprint(context)?
        {
            return Err(LedgerError::invalid(
                "Run does not pin a stable captured Execution Context",
            ));
        }
    }
    Ok(())
}

async fn verify_deployment_capture(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    capture: &Value,
    time: i64,
) -> Result<(), LedgerError> {
    let row: Option<(String,Value)> = sqlx::query_as("SELECT deployment_id,assurance_record('deployment',project,deployment_id) FROM release_deployments WHERE project=$1 AND assurance_record_field('deployment',project,deployment_id,'verification_target_fingerprint') #>> '{}' = $2")
        .bind(project).bind(capture["verification_target_fingerprint"].as_str()).fetch_optional(&mut **tx).await?;
    let (deployment_id, deployment) = row.ok_or_else(|| {
        LedgerError::invalid("capture requires a registered deployment Verification Target")
    })?;
    let observed: Option<(Value,Option<String>)> = sqlx::query_as("SELECT assurance_record('deployment_observation',project,fingerprint),deployment_id FROM deployment_observations WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1")
        .bind(project).bind(deployment["environment"].as_str()).fetch_optional(&mut **tx).await?;
    let (observed, selected) =
        observed.ok_or_else(|| LedgerError::invalid("deployment observation is missing"))?;
    let at = observed["observed_at_ms"].as_i64().unwrap_or(-1);
    let valid_for = observed["valid_for_seconds"].as_i64().unwrap_or(0) * 1000;
    if selected.as_deref() != Some(deployment_id.as_str())
        || observed["state"] != "complete"
        || at > time + 5000
        || at + valid_for < time
        || capture["finished_at_ms"]
            .as_i64()
            .is_none_or(|v| v < time - valid_for || v.abs_diff(at) > valid_for as u64)
    {
        return Err(LedgerError::invalid(
            "capture must match the latest fresh complete deployment observation",
        ));
    }
    let observed_instance = observed["components"]
        .as_array()
        .unwrap()
        .iter()
        .any(|component| {
            component["name"] == capture["component"]
                && component["instances"]
                    .as_array()
                    .is_some_and(|instances| instances.contains(&capture["instance"]))
        });
    if !observed_instance {
        return Err(LedgerError::invalid(
            "capture instance is not independently observed in the deployment component",
        ));
    }
    Ok(())
}

pub(super) async fn show(
    State(state): State<LedgerState>,
    Path((project, deployment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    assess_deployment(&state, &project, &deployment)
        .await
        .map(Json)
}

pub(super) async fn compute_deployment(
    state: &LedgerState,
    project: &str,
    deployment: &str,
    selected: Option<std::collections::BTreeSet<String>>,
) -> Result<(Value, Value), LedgerError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let entry: Option<(String,Value)> = sqlx::query_as("SELECT release_fingerprint,assurance_record('deployment',project,deployment_id) FROM release_deployments WHERE project=$1 AND deployment_id=$2")
        .bind(project).bind(deployment).fetch_optional(&mut *tx).await?;
    let (release, deployment_record) = entry.ok_or_else(LedgerError::not_found)?;
    let publication: Option<Value> = sqlx::query_scalar("SELECT assurance_record('release',project,fingerprint) FROM assurance_releases WHERE project=$1 AND fingerprint=$2")
        .bind(project).bind(&release).fetch_optional(&mut *tx).await?;
    let publication = publication.ok_or_else(LedgerError::not_found)?;
    let authority = &publication["snapshot"]["authority"];
    let revision = authority["fingerprint"]
        .as_str()
        .ok_or_else(|| LedgerError::invalid("release authority fingerprint missing"))?;
    let observed: Option<(Value,Option<String>)> = sqlx::query_as("SELECT assurance_record('deployment_observation',project,fingerprint),deployment_id FROM deployment_observations WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1")
        .bind(project).bind(deployment_record["environment"].as_str()).fetch_optional(&mut *tx).await?;
    let time = now(&mut *tx).await?;
    let mut observation = json!({"verification_target_fingerprint":deployment_record["verification_target_fingerprint"],"observed_at_ms":0,"valid_for_seconds":1,"state":"unknown","components":[],"trusted":false});
    if let Some((observed, selected)) = observed {
        let bound = selected.as_deref() == deployment_record["id"].as_str();
        observation["observed_at_ms"] = observed["observed_at_ms"].clone();
        observation["valid_for_seconds"] = observed["valid_for_seconds"].clone();
        observation["state"] = json!(if bound && observed["state"] == "complete" {
            "complete"
        } else {
            "unknown"
        });
        observation["trusted"] = json!(bound);
        if bound {
            observation["components"] = Value::Array(observed["components"].as_array().unwrap().iter().map(|component| json!({
                "id":component["name"],"digest":component["image"].as_str().unwrap_or("").rsplit_once('@').map(|(_,d)|d).unwrap_or(""),
                "instances":component.get("instances").cloned().unwrap_or_else(||json!([])),
            })).collect());
        }
    }
    let checks = authority["checks"].as_array().unwrap();
    let check_ids: Vec<&str> = checks
        .iter()
        .map(|check| check["id"].as_str().unwrap())
        .collect();
    let check_fingerprints: Vec<&str> = checks
        .iter()
        .map(|check| check["fingerprint"].as_str().unwrap())
        .collect();
    let persistent_checks: Vec<&str> = checks
        .iter()
        .filter(|check| check["deployment"]["monitor"].is_null())
        .map(|check| check["id"].as_str().unwrap())
        .collect();
    let monitor_window = checks
        .iter()
        .filter_map(|check| check["deployment"]["monitor"]["freshness_seconds"].as_i64())
        .max()
        .unwrap_or(0)
        .saturating_mul(1000);
    let cutoff = time.saturating_sub(monitor_window);
    let reviews: Vec<Value> = sqlx::query_scalar("SELECT assurance_record('review',r.project,r.fingerprint) FROM assurance_reviews r JOIN model_revision_reviews m ON m.project=r.project AND m.review_fingerprint=r.fingerprint WHERE r.project=$1 AND m.revision_fingerprint=$2 ORDER BY r.fingerprint LIMIT 5001")
        .bind(project).bind(revision).fetch_all(&mut *tx).await?;
    let review_fingerprints: Vec<&str> = reviews
        .iter()
        .filter_map(|review| review["fingerprint"].as_str())
        .collect();
    let review_runs: Vec<&str> = reviews
        .iter()
        .flat_map(|review| {
            ["basis", "challenge_basis"]
                .into_iter()
                .flat_map(move |field| {
                    review[field]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|basis| basis["run"].as_str())
                })
        })
        .collect();
    let bundles: Vec<Value> = sqlx::query_scalar(&relational::with_tables(r#"
        SELECT assurance_record('run',b.project,b.fingerprint)
        FROM run_bundles b
        WHERE b.project=$1 AND b.run_id IN (
            SELECT r.run_id FROM run_bundles r
            JOIN assurance_records a ON a.kind='run' AND a.project=r.project AND a.record_key=r.fingerprint
            JOIN assurance_run execution ON execution._record_id=a.id
            WHERE r.project=$1 AND (r.run_id=ANY($7::text[]) OR EXISTS(SELECT 1 FROM jsonb_array_elements(CASE WHEN EXISTS(SELECT 1 FROM {verification_challenge_executions} target WHERE target._record_id=a.id) THEN assurance_record_field('run',r.project,r.fingerprint,'verification_challenge_executions') ELSE '[]'::jsonb END) challenge WHERE challenge->'target'->>'fingerprint'=ANY($8::text[])) OR EXISTS (
                SELECT 1 FROM assurance_run_plan_checks_item c
                JOIN unnest($2::text[],$3::text[]) current_check(id,fingerprint)
                  ON c.f_id=current_check.id AND c.f_fingerprint=current_check.fingerprint
                WHERE c._record_id=a.id AND (
                    c.f_id=ANY($4::text[]) OR
                    (r.verification_target_fingerprint=$5 AND COALESCE(execution.f_finished_at_ms,execution.f_started_at_ms)>= $6)
                )
            ))
        ) ORDER BY b.run_id,b.revision LIMIT 5001
    "#)).bind(project).bind(&check_ids).bind(&check_fingerprints).bind(&persistent_checks)
        .bind(deployment_record["verification_target_fingerprint"].as_str()).bind(cutoff).bind(&review_runs).bind(&review_fingerprints).fetch_all(&mut *tx).await?;
    let contract_records = records(&mut *tx, project, "context_contract").await?;
    let bundle_fingerprints: Vec<&str> = bundles
        .iter()
        .filter_map(|bundle| bundle["bundle_fingerprint"].as_str())
        .collect();
    let capture_records: Vec<Value> = sqlx::query_scalar(r#"
        WITH candidates AS (
            SELECT a.project,a.record_key,c.f_kind,
                dense_rank() OVER (PARTITION BY c.f_kind,c.f_verification_target_fingerprint,
                    c.f_bundle_fingerprint,c.f_component,c.f_instance ORDER BY c.f_finished_at_ms DESC) AS ordinal
            FROM assurance_records a JOIN assurance_context_capture_capture c ON c._record_id=a.id
            WHERE a.project=$1 AND a.kind='context_capture' AND (
                c.f_kind='deployment' AND c.f_verification_target_fingerprint=$2
                OR c.f_kind='execution' AND c.f_bundle_fingerprint=ANY($3)))
        SELECT assurance_record('context_capture',project,record_key) FROM candidates
        WHERE f_kind='execution' OR ordinal=1 ORDER BY record_key LIMIT 5001
        "#).bind(project).bind(deployment_record["verification_target_fingerprint"].as_str())
        .bind(&bundle_fingerprints).fetch_all(&mut *tx).await?;
    if capture_records.len() > 5000 {
        return Err(LedgerError::invalid(
            "deployment context snapshot exceeds 5000 records",
        ));
    }
    let contracts: Vec<Value> = contract_records
        .into_iter()
        .map(|record| record["contract"].clone())
        .collect();
    let captures: Vec<Value> = capture_records
        .into_iter()
        .map(|record| record["capture"].clone())
        .collect();
    let request = json!({"format":"azimuth-deployment-assessment","version":2,"publication":publication,
        "deployment":deployment_record,"deployment_observation":observation,"context_contracts":contracts,"captures":captures,
        "bundles":bundles,"reviews":reviews,"evaluated_at_ms":time});
    if bundles.len() > 5000 || reviews.len() > 5000 || request.to_string().len() > 33_554_432 {
        return Err(LedgerError::invalid(
            "deployment assessment exceeds bounded complete snapshot",
        ));
    }
    tx.commit().await?;
    let snapshot = request.clone();
    let result = compute(move || {
        let result = azimuth::deployment_status::assess_selected(
            &core_json(request.to_string().as_bytes())?,
            selected.as_ref(),
        )
        .map_err(LedgerError::invalid)?;
        to_value(&result)
    })
    .await?;
    Ok((result, snapshot))
}

pub(super) async fn assess_deployment(
    state: &LedgerState,
    project: &str,
    deployment: &str,
) -> Result<Value, LedgerError> {
    super::materialized::read(state, project, deployment, false).await
}

pub(super) async fn summary(
    State(state): State<LedgerState>,
    Path((project, deployment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    super::materialized::read(&state, &project, &deployment, true)
        .await
        .map(Json)
}
