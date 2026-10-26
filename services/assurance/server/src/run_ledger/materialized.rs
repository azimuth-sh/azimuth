use super::*;
use std::collections::BTreeSet;

pub(super) fn start(state: LedgerState) {
    tokio::spawn(async move {
        loop {
            if let Err(error) = tick(&state).await {
                tracing::error!(
                    ?error,
                    "deployment assessment worker failed; durable work retained"
                );
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });
}

async fn time(conn: &mut sqlx::PgConnection) -> Result<i64, LedgerError> {
    Ok(
        sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
            .fetch_one(conn)
            .await?,
    )
}

const RENEWAL_LEAD_MS: i64 = 30_000;

async fn events(state: &LedgerState) -> Result<(), LedgerError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(848332105)")
        .execute(&mut *tx)
        .await?;
    let projects: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT project FROM release_deployments ORDER BY project")
            .fetch_all(&mut *tx)
            .await?;
    for project in projects {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("assessment initialization|{project}"))
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO deployment_assessments(project,deployment,release) SELECT project,deployment_id,release_fingerprint FROM release_deployments ON CONFLICT DO NOTHING").execute(&mut *tx).await?;
    let replay:Vec<(i64,String,String,String)>=sqlx::query_as("SELECT sequence,project,kind,record_key FROM assessment_events WHERE NOT invalidated ORDER BY sequence LIMIT 1000 FOR UPDATE").fetch_all(&mut *tx).await?;
    for (sequence, project, kind, key) in replay {
        sqlx::query(
            "INSERT INTO assessment_events(project,kind,record_key,replayed) VALUES($1,$2,$3,true)",
        )
        .bind(project)
        .bind(kind)
        .bind(key)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM assessment_events WHERE sequence=$1")
            .bind(sequence)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("SET CONSTRAINTS invalidate_assessment_event IMMEDIATE")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM assessment_events WHERE sequence IN (SELECT sequence FROM assessment_events WHERE invalidated ORDER BY sequence LIMIT 1000)").execute(&mut *tx).await?;
    sqlx::query("SELECT 1 FROM deployment_assessments ORDER BY project,deployment FOR UPDATE")
        .execute(&mut *tx)
        .await?;
    let now = time(&mut tx).await?;
    // Renewal starts before expiry, while the published assessment is still current, so a Requirement is never
    // read as pending merely because its evidence ages: the new evaluation replaces it at publication.
    let expired: Vec<(String,String)> = sqlx::query_as("UPDATE assessment_requirements SET generation=generation+1,renewing=true WHERE generation=published_generation AND valid_until_ms-$1 <= LEAST($2,(valid_until_ms-COALESCE(evaluated_at_ms,0))/2) RETURNING project,deployment").bind(now).bind(RENEWAL_LEAD_MS).fetch_all(&mut *tx).await?;
    for (project, deployment) in expired {
        sqlx::query("UPDATE deployment_assessments SET generation=generation+1,retry_at=now() WHERE project=$1 AND deployment=$2").bind(project).bind(deployment).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub(super) async fn tick(state: &LedgerState) -> Result<(), LedgerError> {
    let mut lease = state.pool.begin().await?;
    let leader: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(848332106)")
        .fetch_one(&mut *lease)
        .await?;
    if !leader {
        return Ok(());
    }
    let result = tick_leader(state).await;
    lease.rollback().await?;
    result
}

async fn tick_leader(state: &LedgerState) -> Result<(), LedgerError> {
    let started = std::time::Instant::now();
    // Resolution is ingestion-side background work, never a page-read side effect.
    for project in state.credentials.keys() {
        target_observation::resolve_pending(state, project).await?;
    }
    let resolution = started.elapsed();
    let stage = std::time::Instant::now();
    events(state).await?;
    let event_processing = stage.elapsed();
    let jobs: Vec<(String,String,i64)> = sqlx::query_as("SELECT project,deployment,generation FROM deployment_assessments WHERE (generation<>published_generation OR authority IS NULL) AND retry_at<=now() ORDER BY (EXISTS(SELECT 1 FROM deployment_observations o WHERE o.project=deployment_assessments.project AND o.deployment_id=deployment_assessments.deployment AND o.revision=(SELECT max(latest.revision) FROM deployment_observations latest WHERE latest.project=o.project AND latest.environment=o.environment))) DESC,retry_at,project,deployment LIMIT 1").fetch_all(&state.pool).await?;
    for (project, deployment, generation) in jobs {
        if let Err(error) = refresh(state, &project, &deployment, generation).await {
            tracing::error!(
                ?error,
                project,
                deployment,
                "assessment remains pending after computation failure"
            );
            sqlx::query("UPDATE deployment_assessments SET last_error=$3,retry_at=now()+interval '10 seconds' WHERE project=$1 AND deployment=$2").bind(&project).bind(&deployment).bind(format!("{error:?}")).execute(&state.pool).await?;
        }
    }
    let stage = std::time::Instant::now();
    materialized_reviews::process(state).await?;
    let total = started.elapsed();
    if total >= std::time::Duration::from_secs(1) {
        tracing::info!(
            resolution_ms = resolution.as_millis() as u64,
            event_processing_ms = event_processing.as_millis() as u64,
            review_processing_ms = stage.elapsed().as_millis() as u64,
            total_ms = total.as_millis() as u64,
            "assessment worker tick"
        );
    }
    Ok(())
}

fn deadline(request: &Value, requirement: &Value) -> i64 {
    let now = request["evaluated_at_ms"].as_i64().unwrap();
    let observation = &request["deployment_observation"];
    let validity = observation["valid_for_seconds"]
        .as_i64()
        .unwrap_or(0)
        .saturating_mul(1000);
    let mut boundaries = Vec::new();
    let mut add = |at: i64, duration: i64| {
        boundaries.extend([
            at.saturating_sub(5000),
            at,
            at.saturating_add(duration).saturating_add(1),
        ]);
    };
    add(
        observation["observed_at_ms"].as_i64().unwrap_or(0),
        validity,
    );
    let checks = request["publication"]["snapshot"]["authority"]["checks"]
        .as_array()
        .unwrap();
    let global_window = checks
        .iter()
        .filter_map(|check| check["deployment"]["monitor"]["freshness_seconds"].as_i64())
        .max()
        .unwrap_or(0)
        .saturating_mul(1000);
    let mut relevant = BTreeMap::new();
    let mut contexts = BTreeSet::new();
    for binding in requirement["bindings"].as_array().unwrap() {
        let check_id = binding["check"].as_str().unwrap();
        let check = checks.iter().find(|check| check["id"] == check_id).unwrap();
        relevant.insert(
            check_id,
            check["deployment"]["monitor"]["freshness_seconds"]
                .as_i64()
                .map(|seconds| seconds.saturating_mul(1000)),
        );
        if let Some(required) = check["deployment"]["contexts"].as_array() {
            for dependency in required {
                contexts.insert(dependency["component"].as_str().unwrap());
            }
        }
    }
    for condition in requirement["explanation"]["conditions"]
        .as_array()
        .into_iter()
        .flatten()
    {
        contexts.insert(condition["component"].as_str().unwrap());
    }
    // Only the newest deployment capture of an instance is ever selected, so older ones cannot change a status by ageing out.
    let capture_instance = |capture: &Value| {
        (
            capture["verification_target_fingerprint"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            capture["component"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            capture["instance"].as_str().unwrap_or_default().to_string(),
        )
    };
    let mut newest_deployment_capture: BTreeMap<(String, String, String), i64> = BTreeMap::new();
    for capture in request["captures"].as_array().unwrap() {
        if capture["kind"] == "deployment" {
            let at = capture["finished_at_ms"].as_i64().unwrap();
            let newest = newest_deployment_capture
                .entry(capture_instance(capture))
                .or_insert(at);
            *newest = (*newest).max(at);
        }
    }
    for capture in request["captures"].as_array().unwrap() {
        let matches = capture["before"]["report"]["contexts"]
            .as_object()
            .is_some_and(|values| {
                values
                    .keys()
                    .any(|component| contexts.contains(component.as_str()))
            });
        if matches {
            let at = capture["finished_at_ms"].as_i64().unwrap();
            if capture["kind"] == "deployment"
                && newest_deployment_capture.get(&capture_instance(capture)) != Some(&at)
            {
                continue;
            }
            add(
                at,
                if capture["kind"] == "deployment" {
                    validity
                } else {
                    0
                },
            );
        }
    }
    // A monitored Check is judged by the Check Results the evaluation selected; superseded results never regain
    // influence by ageing, so their freshness windows are not boundaries.
    let selected_results: BTreeSet<&str> =
        ["supporting", "violations", "inconclusive", "unassessed"]
            .into_iter()
            .filter_map(|group| requirement["explanation"]["evidence"][group].as_array())
            .flatten()
            .filter_map(|reference| reference["check_result"].as_str())
            .collect();
    let bundles = request["bundles"].as_array().unwrap().iter().chain(
        request["publication"]["snapshot"]["bundles"]
            .as_array()
            .unwrap(),
    );
    for bundle in bundles {
        if let Some(executions) = bundle["check_executions"].as_array() {
            for execution in executions {
                let Some(window) = execution["check"]["id"]
                    .as_str()
                    .and_then(|id| relevant.get(id))
                else {
                    continue;
                };
                let mut selected = window.is_none();
                if let Some(results) = execution["results"].as_array() {
                    for result in results {
                        if window.is_some()
                            && !result["fingerprint"]
                                .as_str()
                                .is_some_and(|fingerprint| selected_results.contains(fingerprint))
                        {
                            continue;
                        }
                        selected = true;
                        add(
                            result["observed_at_ms"].as_i64().unwrap(),
                            window.unwrap_or(0),
                        );
                    }
                }
                if let (Some(window), true) = (window, selected) {
                    add(bundle["finished_at_ms"].as_i64().unwrap_or(0), *window);
                    add(
                        bundle["finished_at_ms"].as_i64().unwrap_or(0),
                        global_window,
                    );
                }
            }
        }
    }
    boundaries
        .into_iter()
        .filter(|boundary| *boundary > now)
        .min()
        .unwrap_or(now.saturating_add(3_600_000))
}

async fn refresh(
    state: &LedgerState,
    project: &str,
    deployment: &str,
    generation: i64,
) -> Result<(), LedgerError> {
    let rows: Vec<(String,i64)> = sqlx::query_as("SELECT requirement,generation FROM assessment_requirements WHERE project=$1 AND deployment=$2 AND generation<>published_generation ORDER BY requirement").bind(project).bind(deployment).fetch_all(&state.pool).await?;
    let selected = if rows.is_empty() {
        None
    } else {
        Some(
            rows.iter()
                .map(|(id, _)| id.clone())
                .collect::<BTreeSet<_>>(),
        )
    };
    let (result, request) =
        deployment_status::compute_deployment(state, project, deployment, selected).await?;
    let publication = std::time::Instant::now();
    let published = publish(
        state, project, deployment, generation, &rows, &result, &request,
    )
    .await;
    tracing::info!(
        project,
        deployment,
        publish_ms = publication.elapsed().as_millis() as u64,
        "assessment published"
    );
    published
}

async fn publish(
    state: &LedgerState,
    project: &str,
    deployment: &str,
    generation: i64,
    rows: &[(String, i64)],
    result: &Value,
    request: &Value,
) -> Result<(), LedgerError> {
    let mut tx = state.pool.begin().await?;
    let (current,published): (i64,i64) = sqlx::query_as("SELECT generation,published_generation FROM deployment_assessments WHERE project=$1 AND deployment=$2 FOR UPDATE").bind(project).bind(deployment).fetch_one(&mut *tx).await?;
    let initial_or_unbound:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assessment_events WHERE project=$1 AND (NOT invalidated OR $2))").bind(project).bind(published==0).fetch_one(&mut *tx).await?;
    // Evidence that arrives while a computation runs invalidates only the Requirements it touches, so
    // an overtaken computation still publishes every Requirement whose own generation is unchanged.
    let overtaken = current != generation;
    if (overtaken && (published == 0 || rows.is_empty()))
        || published == generation
        || initial_or_unbound
    {
        return Ok(());
    }
    let current_requirements: BTreeMap<String, i64> = sqlx::query_as::<_, (String, i64)>("SELECT requirement,generation FROM assessment_requirements WHERE project=$1 AND deployment=$2")
        .bind(project).bind(deployment).fetch_all(&mut *tx).await?.into_iter().collect();
    let authority = &request["publication"]["snapshot"]["authority"];
    for requirement in result["requirements"].as_array().unwrap() {
        let id = requirement["requirement"].as_str().unwrap();
        let due = deadline(&request, requirement);
        let requirement_generation = rows
            .iter()
            .find(|(requirement, _)| requirement == id)
            .map(|(_, generation)| *generation)
            .unwrap_or(1);
        if !rows.is_empty()
            && current_requirements
                .get(id)
                .is_some_and(|current| *current != requirement_generation)
        {
            continue;
        }
        let key = format!("{deployment}:{id}:{requirement_generation}");
        let previous:Option<String>=sqlx::query_scalar("SELECT record_key FROM assessment_requirements WHERE project=$1 AND deployment=$2 AND requirement=$3").bind(project).bind(deployment).bind(id).fetch_optional(&mut *tx).await?.flatten();
        relational::store(
            &mut tx,
            "assessment_requirement",
            project,
            &key,
            requirement,
        )
        .await?;
        sqlx::query("INSERT INTO assessment_requirements(project,deployment,requirement,generation,published_generation,record_key,status,verification_review,realization_review,realization_code_drift,realization_reviewed_commit,summary,evaluated_at_ms,valid_until_ms,supported_since_ms) VALUES($1,$2,$3,$4,$4,$5,$6::text,$7,$8,$9,$10,$11,$12::bigint,$13,CASE WHEN $6::text='supported' THEN $12::bigint END) ON CONFLICT(project,deployment,requirement) DO UPDATE SET published_generation=excluded.published_generation,record_key=excluded.record_key,supported_since_ms=CASE WHEN excluded.status<>'supported' THEN NULL WHEN assessment_requirements.status='supported' AND assessment_requirements.supported_since_ms IS NOT NULL THEN assessment_requirements.supported_since_ms ELSE excluded.evaluated_at_ms END,status=excluded.status,verification_review=excluded.verification_review,realization_review=excluded.realization_review,realization_code_drift=excluded.realization_code_drift,realization_reviewed_commit=excluded.realization_reviewed_commit,summary=excluded.summary,evaluated_at_ms=excluded.evaluated_at_ms,valid_until_ms=excluded.valid_until_ms,renewing=false")
            .bind(project).bind(deployment).bind(id).bind(requirement_generation).bind(&key).bind(requirement["status"].as_str()).bind(requirement["verification_review"].as_str()).bind(requirement["realization_review"].as_str()).bind(requirement["realization_code_drift"].as_bool()).bind(requirement["explanation"]["realization_review"]["reviewed_commit"].as_str()).bind(requirement["explanation"]["summary"].as_str()).bind(result["evaluated_at_ms"].as_i64()).bind(due).execute(&mut *tx).await?;
        if let Some(previous) = previous.filter(|previous| previous != &key) {
            relational::discard_assessment_requirement(&mut tx, project, &previous).await?;
        }
        let declaration = authority["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap();
        let mut dependencies = BTreeSet::new();
        dependencies.insert((
            "environment".to_string(),
            request["deployment"]["environment"]
                .as_str()
                .unwrap()
                .to_string(),
        ));
        for condition in requirement["explanation"]["conditions"]
            .as_array()
            .into_iter()
            .flatten()
        {
            dependencies.insert((
                "context".into(),
                condition["component"].as_str().unwrap().into(),
            ));
        }
        for binding in declaration["bindings"].as_array().unwrap() {
            let check_id = binding["check"].as_str().unwrap();
            dependencies.insert(("check".into(), check_id.into()));
            if let Some(check) = authority["checks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == check_id)
            {
                if let Some(contexts) = check["deployment"]["contexts"].as_array() {
                    for context in contexts {
                        dependencies.insert((
                            "context".into(),
                            context["component"].as_str().unwrap().into(),
                        ));
                    }
                }
            }
        }
        if let Some(review) =
            requirement["explanation"]["realization_review"]["fingerprint"].as_str()
        {
            dependencies.insert(("review".into(), review.into()));
        }
        if let Some(review) =
            requirement["explanation"]["verification_review"]["fingerprint"].as_str()
        {
            dependencies.insert(("review".into(), review.into()));
            if let Some(record) = request["reviews"]
                .as_array()
                .unwrap()
                .iter()
                .chain(
                    request["publication"]["snapshot"]["reviews"]
                        .as_array()
                        .unwrap(),
                )
                .find(|record| record["fingerprint"] == review)
            {
                if let Some(basis) = record["challenge_basis"].as_array() {
                    for item in basis {
                        dependencies
                            .insert(("review-run".into(), item["run"].as_str().unwrap().into()));
                    }
                }
            }
        }
        let (dependency_kinds, dependency_identities): (Vec<String>, Vec<String>) =
            dependencies.into_iter().unzip();
        sqlx::query("INSERT INTO assessment_dependencies(project,deployment,requirement,kind,identity) SELECT $1,$2,$3,kind,identity FROM UNNEST($4::text[],$5::text[]) AS d(kind,identity) ON CONFLICT DO NOTHING")
            .bind(project).bind(deployment).bind(id).bind(&dependency_kinds).bind(&dependency_identities).execute(&mut *tx).await?;
        // One statement per Requirement keeps the row locks of the publication short; a repeated binding keeps
        // its last result, as sequential upserts would.
        let mut scenario_results: BTreeMap<(String, String), (String, String)> = BTreeMap::new();
        for binding in requirement["bindings"].as_array().unwrap() {
            let coverage = requirement["explanation"]["coverage"]
                .as_array()
                .unwrap()
                .iter()
                .find(|coverage| {
                    coverage["check"] == binding["check"]
                        && coverage["scenario"] == binding["scenario"]
                })
                .unwrap();
            let evidence = run::canonical_fingerprint(&core_json(coverage.to_string().as_bytes())?)
                .map_err(LedgerError::invalid)?;
            scenario_results.insert(
                (
                    binding["scenario"].as_str().unwrap().to_string(),
                    binding["check"].as_str().unwrap().to_string(),
                ),
                (binding["status"].as_str().unwrap().to_string(), evidence),
            );
        }
        let (scenario_ids, check_ids): (Vec<String>, Vec<String>) =
            scenario_results.keys().cloned().unzip();
        let (statuses, fingerprints): (Vec<String>, Vec<String>) =
            scenario_results.into_values().unzip();
        sqlx::query("INSERT INTO assessment_scenarios(project,deployment,requirement,scenario_id,check_id,status,generation,result_fingerprint) SELECT $1,$2,$3,s.scenario_id,s.check_id,s.status,$4,s.result_fingerprint FROM UNNEST($5::text[],$6::text[],$7::text[],$8::text[]) AS s(scenario_id,check_id,status,result_fingerprint) ON CONFLICT(project,deployment,requirement,scenario_id,check_id) DO UPDATE SET status=excluded.status,generation=excluded.generation,result_fingerprint=excluded.result_fingerprint WHERE assessment_scenarios.status IS DISTINCT FROM excluded.status OR assessment_scenarios.result_fingerprint IS DISTINCT FROM excluded.result_fingerprint")
            .bind(project).bind(deployment).bind(id).bind(requirement_generation).bind(&scenario_ids).bind(&check_ids).bind(&statuses).bind(&fingerprints).execute(&mut *tx).await?;
    }
    let unpublished: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assessment_requirements WHERE project=$1 AND deployment=$2 AND generation<>published_generation)")
        .bind(project).bind(deployment).fetch_one(&mut *tx).await?;
    let published_generation = if unpublished { published } else { current };
    sqlx::query("UPDATE deployment_assessments SET authority=$3,model=$4,verification_target=$5,published_generation=$6,evaluated_at_ms=$7,valid_until_ms=(SELECT min(valid_until_ms) FROM assessment_requirements WHERE project=$1 AND deployment=$2),observed_current=$8,last_error=NULL,retry_at=now() WHERE project=$1 AND deployment=$2")
        .bind(project).bind(deployment).bind(result["authority_fingerprint"].as_str()).bind(result["model_fingerprint"].as_str()).bind(result["verification_target_fingerprint"].as_str()).bind(published_generation).bind(result["evaluated_at_ms"].as_i64()).bind(request["deployment_observation"]["state"]=="complete" && request["deployment_observation"]["trusted"]==true).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn read(
    state: &LedgerState,
    project: &str,
    deployment: &str,
    summary: bool,
) -> Result<Value, LedgerError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let row =
        sqlx::query("SELECT a.*, EXISTS(SELECT 1 FROM assessment_events e WHERE e.project=a.project AND NOT e.invalidated) AS unbound FROM deployment_assessments a WHERE project=$1 AND deployment=$2")
            .bind(project)
            .bind(deployment)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(row) = row else {
        let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM release_deployments WHERE project=$1 AND deployment_id=$2)").bind(project).bind(deployment).fetch_one(&mut *tx).await?;
        if !exists {
            return Err(LedgerError::not_found());
        }
        return Ok(
            json!({"format":"azimuth-deployment-status","version":2,"project":project,"deployment":deployment,"requirements":[],"assessment":{"state":"pending","generation":0,"published_generation":0,"evaluated_at_ms":null,"valid_until_ms":null}}),
        );
    };
    let now = time(&mut tx).await?;
    let generation: i64 = row.get("generation");
    let published: i64 = row.get("published_generation");
    let due: Option<i64> = row.get("valid_until_ms");
    // A renewing Requirement keeps its published assessment until that assessment's own validity ends.
    let invalidated: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assessment_requirements WHERE project=$1 AND deployment=$2 AND generation<>published_generation AND NOT renewing)").bind(project).bind(deployment).fetch_one(&mut *tx).await?;
    let freshness = if (generation != published && invalidated)
        || published == 0
        || row.get::<bool, _>("unbound")
    {
        "pending"
    } else if due.is_some_and(|due| due <= now) {
        "expired"
    } else {
        "ready"
    };
    let entries=sqlx::query("SELECT requirement,record_key,status,verification_review,realization_review,realization_code_drift,realization_reviewed_commit,summary,evaluated_at_ms,supported_since_ms,generation<>published_generation AND NOT renewing AS unpublished,valid_until_ms FROM assessment_requirements WHERE project=$1 AND deployment=$2 ORDER BY requirement").bind(project).bind(deployment).fetch_all(&mut *tx).await?;
    let mut requirements = Vec::new();
    for entry in entries {
        let mut requirement = if summary {
            let checks: Option<Value> = sqlx::query_scalar::<_, Option<Value>>(
                "SELECT assurance_record_field('assessment_requirement',$1,$2,'checks')",
            )
            .bind(project)
            .bind(entry.get::<Option<String>, _>("record_key"))
            .fetch_optional(&mut *tx)
            .await?
            .flatten();
            json!({"requirement":entry.get::<String,_>("requirement"),"status":entry.get::<String,_>("status"),"verification_review":entry.get::<String,_>("verification_review"),"realization_review":entry.get::<String,_>("realization_review"),"realization_code_drift":entry.get::<Option<bool>,_>("realization_code_drift"),"diagnostics":[],"gaps":[],"explanation":{"summary":entry.get::<String,_>("summary"),"realization_review":{"status":entry.get::<String,_>("realization_review"),"reviewed_commit":entry.get::<Option<String>,_>("realization_reviewed_commit"),"code_drift":entry.get::<Option<bool>,_>("realization_code_drift")}},"checks":checks.unwrap_or_else(|| json!([]))})
        } else {
            sqlx::query_scalar::<_, Value>(
                "SELECT assurance_record('assessment_requirement',$1,$2)",
            )
            .bind(project)
            .bind(entry.get::<Option<String>, _>("record_key"))
            .fetch_one(&mut *tx)
            .await?
        };
        // A Requirement is as current as its own evidence: another Requirement's pending evidence does not hide it.
        let requirement_freshness = if published == 0
            || row.get::<bool, _>("unbound")
            || entry.get::<bool, _>("unpublished")
        {
            "pending"
        } else if entry
            .get::<Option<i64>, _>("valid_until_ms")
            .is_some_and(|due| due <= now)
        {
            "expired"
        } else {
            "ready"
        };
        if requirement_freshness != "ready" {
            let message = format!(
                "Assessment is {requirement_freshness}; current evidence is being evaluated"
            );
            requirement["status"] = json!("unresolved");
            requirement["diagnostics"] = json!([message]);
            requirement["explanation"] = json!({"summary":message,"status":"unresolved","reasons":[{"code":"assessment-pending","category":"freshness","message":message}],"coverage":[],"evidence":{"supporting":[],"violations":[],"inconclusive":[],"unassessed":[]}});
            requirement["bindings"] = json!([]);
            requirement["checks"] = json!([]);
        }
        requirement["supported_since_ms"] =
            if requirement_freshness == "ready" && requirement["status"] == "supported" {
                json!(entry.get::<Option<i64>, _>("supported_since_ms"))
            } else {
                Value::Null
            };
        requirements.push(requirement);
    }
    Ok(
        json!({"format":"azimuth-deployment-status","version":2,"project":project,"deployment":deployment,
        "authority_fingerprint":row.get::<Option<String>,_>("authority"),"model_fingerprint":row.get::<Option<String>,_>("model"),"verification_target_fingerprint":row.get::<Option<String>,_>("verification_target"),
        "evaluated_at_ms":row.get::<Option<i64>,_>("evaluated_at_ms"),"requirements":requirements,
        "assessment":{"state":freshness,"generation":generation,"published_generation":published,"evaluated_at_ms":row.get::<Option<i64>,_>("evaluated_at_ms"),"valid_until_ms":due}}),
    )
}

pub(super) async fn rebuild(
    State(state): State<LedgerState>,
    Path((project, deployment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, Some(Role::Owner))?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("assessment initialization|{project}"))
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO deployment_assessments(project,deployment,release) SELECT project,deployment_id,release_fingerprint FROM release_deployments WHERE project=$1 AND deployment_id=$2 ON CONFLICT DO NOTHING").bind(&project).bind(&deployment).execute(&mut *tx).await?;
    let generation:Option<i64>=sqlx::query_scalar("UPDATE deployment_assessments SET generation=generation+1,retry_at=now() WHERE project=$1 AND deployment=$2 RETURNING generation").bind(&project).bind(&deployment).fetch_optional(&mut *tx).await?;
    let generation = generation.ok_or_else(LedgerError::not_found)?;
    sqlx::query(
        "UPDATE assessment_requirements SET generation=generation+1 WHERE project=$1 AND deployment=$2",
    )
    .bind(&project)
    .bind(&deployment)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE definition_assessments SET generation=generation+1,retry_at=now() WHERE project=$1 AND revision=(SELECT r.model_revision FROM release_deployments d JOIN assurance_releases r ON r.project=d.project AND r.fingerprint=d.release_fingerprint WHERE d.project=$1 AND d.deployment_id=$2)").bind(&project).bind(&deployment).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"state":"pending","generation":generation})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use azimuth::run::*;
    use testcontainers_modules::{
        postgres::Postgres,
        testcontainers::{runners::AsyncRunner, ImageExt},
    };
    #[allow(dead_code)]
    mod evidence {
        use super::*;
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/deployment_evidence.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/review_challenge.rs"
        ));
        pub(super) fn verification_challenge(record: &str, model: &str) -> RunBundle {
            let mut bundle = challenge_bundle();
            let challenge = &mut bundle.plan.challenges[0];
            challenge.target = ChallengeTarget {
                kind: ChallengeTargetKind::RequirementReview,
                id: "restricted".into(),
                fingerprint: record.into(),
            };
            challenge.id = challenge_selection_id(
                &challenge.verification_challenge.fingerprint,
                challenge.target.kind,
                &challenge.target.fingerprint,
            );
            bundle.actual_selection.challenges[0] = challenge.clone();
            bundle.verification_challenge_executions[0].challenge = challenge.id.clone();
            bundle.verification_challenge_executions[0].target = challenge.target.clone();
            bundle.provenance.adapter.routes[1].selection.id = challenge.id.clone();
            bundle.diagnostics[0].scope = DiagnosticScope::VerificationChallengeExecution {
                verification_challenge_fingerprint: challenge
                    .verification_challenge
                    .fingerprint
                    .clone(),
                target_fingerprint: record.into(),
            };
            bundle.plan.model_fingerprint = model.into();
            refresh(&mut bundle);
            assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
            bundle
        }
        pub(super) fn correction(previous: &RunBundle, outcome: ChallengeOutcome) -> RunBundle {
            let mut bundle = previous.clone();
            bundle.bundle_revision += 1;
            bundle.corrects = Some(previous.bundle_fingerprint.clone());
            bundle.correction_reason =
                Some("Correct the recorded synthetic Challenge result.".into());
            bundle.verification_challenge_executions[0].result.outcome = outcome;
            bundle.verification_challenge_executions[0].units[0].attempts[0].outcome = outcome;
            if outcome == ChallengeOutcome::Clean {
                bundle.diagnostics.clear();
                for activity in &mut bundle.activities {
                    activity.diagnostics.clear();
                }
                bundle.verification_challenge_executions[0]
                    .result
                    .objections
                    .clear();
            } else {
                let original = challenge_bundle();
                bundle.diagnostics = original.diagnostics;
                bundle.diagnostics[0].scope = DiagnosticScope::VerificationChallengeExecution {
                    verification_challenge_fingerprint: bundle.plan.challenges[0]
                        .verification_challenge
                        .fingerprint
                        .clone(),
                    target_fingerprint: bundle.plan.challenges[0].target.fingerprint.clone(),
                };
                bundle.activities[0].diagnostics = vec!["mutation/survivor".into()];
                bundle.verification_challenge_executions[0]
                    .result
                    .objections = vec!["mutation/survivor".into()];
            }
            refresh(&mut bundle);
            assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
            assert_eq!(bundle.run_id, previous.run_id);
            bundle
        }
        pub(super) fn request(freshness: i64) -> Value {
            request_with_header(freshness, "")
        }
        pub(super) fn request_with_header(freshness: i64, extra_header: &str) -> Value {
            let publication = publication_policy_with_header(
                json!({"verification_target":"deployment","components":["api"],"contexts":[],"contribution":"both","monitor":{"id":"monitor","interval_seconds":1,"freshness_seconds":freshness}}),
                extra_header,
            );
            let deployment = publication["release"]["deployments"][0].clone();
            let mut bundle = valid_bundle();
            bundle.verification_target =
                verification_target_from_json(&native(&deployment["verification_target"])).unwrap();
            let definition = publication["snapshot"]["authority"]["checks"][0]["fingerprint"]
                .as_str()
                .unwrap()
                .to_string();
            bundle.plan.checks[0].fingerprint = definition.clone();
            bundle.actual_selection.checks[0].fingerprint = definition.clone();
            bundle.check_executions[0].check.fingerprint = definition;
            refresh(&mut bundle);
            json!({"format":"azimuth-deployment-assessment","version":2,"publication":publication,"deployment":deployment,
                "deployment_observation":{"verification_target_fingerprint":bundle.verification_target_fingerprint,"observed_at_ms":4,"valid_for_seconds":120,"state":"complete","trusted":true,"components":[{"id":"api","digest":fp('e'),"instances":["instance"]}]},
                "context_contracts":[],"captures":[],"bundles":[value(&to_json(&bundle))],"reviews":[],"evaluated_at_ms":5})
        }
    }
    #[test]
    fn superseded_probe_results_and_captures_do_not_shorten_validity() {
        let capture = |finished_at_ms: i64| {
            json!({"kind":"deployment","finished_at_ms":finished_at_ms,"verification_target_fingerprint":"target","component":"api","instance":"one",
                "before":{"report":{"contexts":{"context":{"contract_revision":"revision"}}}}})
        };
        let result = |fingerprint: &str, observed_at_ms: i64| json!({"fingerprint":fingerprint,"observed_at_ms":observed_at_ms});
        let request = json!({
            "evaluated_at_ms":1_000_000,
            "deployment_observation":{"observed_at_ms":999_000,"valid_for_seconds":120},
            "publication":{"snapshot":{"bundles":[],"authority":{"checks":[
                {"id":"probe","deployment":{"monitor":{"freshness_seconds":900},"contexts":[{"component":"context","leaves":["mode"]}]}}]}}},
            "captures":[capture(900_000),capture(990_000)],
            "bundles":[
                {"finished_at_ms":150_000,"check_executions":[{"check":{"id":"probe"},"results":[result("first",150_000)]}]},
                {"finished_at_ms":990_000,"check_executions":[{"check":{"id":"probe"},"results":[result("latest",990_000)]}]}]});
        let assessed = |selected: &str| json!({"bindings":[{"check":"probe"}],"explanation":{"evidence":{"supporting":[{"check_result":selected}],"violations":[],"inconclusive":[],"unassessed":[]}}});
        assert_eq!(
            deadline(&request, &assessed("latest")),
            990_000 + 120_000 + 1,
            "only the newest capture and the selected result bound validity"
        );
        assert_eq!(
            deadline(&request, &assessed("first")),
            150_000 + 900_000 + 1,
            "a result the evaluation selected still bounds validity even when a newer one exists"
        );
    }
    #[test]
    fn expiry_matches_native_boundary_and_selected_evaluation_keeps_global_validation() {
        let mut request = evidence::request(10);
        let native = core_json(request.to_string().as_bytes()).unwrap();
        let full = to_value(&azimuth::deployment_status::assess(&native).unwrap()).unwrap();
        assert_eq!(full["requirements"][0]["status"], "supported", "{full}");
        // Future-date eligibility is conservative; expiry itself is the first invalid millisecond.
        request["evaluated_at_ms"] = json!(10_003);
        let at_boundary = to_value(
            &azimuth::deployment_status::assess(
                &core_json(request.to_string().as_bytes()).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(at_boundary["requirements"][0]["status"], "supported");
        assert_eq!(deadline(&request, &at_boundary["requirements"][0]), 10_004);
        request["evaluated_at_ms"] = json!(10_004);
        let expired = to_value(
            &azimuth::deployment_status::assess(
                &core_json(request.to_string().as_bytes()).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(expired["requirements"][0]["status"], "unresolved");
        assert!(expired["requirements"][0]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason.as_str().unwrap().contains("overdue")));
        let empty = to_value(
            &azimuth::deployment_status::assess_selected(&native, Some(&BTreeSet::new())).unwrap(),
        )
        .unwrap();
        let mut expected = full.clone();
        expected["requirements"] = json!([]);
        assert_eq!(
            empty, expected,
            "empty selection skips Requirement evaluation without changing the complete header"
        );
        let selected = Some(BTreeSet::from(["restricted".to_string()]));
        let subset = to_value(
            &azimuth::deployment_status::assess_selected(&native, selected.as_ref()).unwrap(),
        )
        .unwrap();
        assert_eq!(subset, full);
        request["bundles"][0]["bundle_fingerprint"] = json!("invalid-global-history");
        assert!(azimuth::deployment_status::assess_selected(
            &core_json(request.to_string().as_bytes()).unwrap(),
            Some(&BTreeSet::new())
        )
        .is_err());
    }

    #[tokio::test]
    async fn a_realization_review_is_a_status_neutral_dimension_with_informational_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut request = evidence::request_with_header(10, "\nRealization: reviewed");
        let authority = request["publication"]["snapshot"]["authority"].clone();
        let native_authority = core_json(authority.to_string().as_bytes())?;
        let current_sites = authority["requirement_realizations"][0]["sites_fingerprint"]
            .as_str()
            .unwrap()
            .to_owned();
        let realization = |sites: &str, dependencies: String| -> Value {
            let mut review = json!({"kind":"realization-review","requirement":"restricted",
                "requirement_fingerprint":authority["requirement_intents"][0]["fingerprint"],
                "dependencies_fingerprint":dependencies,"sites_fingerprint":sites,"reviewed_commit":"b".repeat(40),
                "decision":"accepted","reassesses":null,"reviewer":"reviewer","reviewed_at_ms":200,
                "rationale":"The realization sites do what the Scenarios say.",
                "artifacts":[{"id":"notes","digest":format!("sha256:{}","a".repeat(64)),"locator":"reviews/notes.md"}]});
            review["fingerprint"] = json!(assurance_review::record_fingerprint(
                &core_json(review.to_string().as_bytes()).unwrap()
            )
            .unwrap());
            review
        };
        let dependencies = azimuth::fingerprint::canonical_sha256(
            &assurance_review::realization_dependencies(&native_authority, "restricted")?,
        );
        let assess = |request: &Value| {
            to_value(
                &azimuth::deployment_status::assess(
                    &core_json(request.to_string().as_bytes()).unwrap(),
                )
                .unwrap(),
            )
            .unwrap()
        };
        let cases = [
            (None, "missing", Value::Null),
            (
                Some(realization(&current_sites, dependencies.clone())),
                "accepted",
                json!(false),
            ),
            (
                Some(realization(
                    &format!("sha256:{}", "d".repeat(64)),
                    dependencies.clone(),
                )),
                "accepted",
                json!(true),
            ),
            (
                Some(realization(
                    &current_sites,
                    format!("sha256:{}", "9".repeat(64)),
                )),
                "stale",
                json!(false),
            ),
        ];
        let mut drifted = None;
        for (record, expected, drift) in cases {
            request["reviews"] = json!(record.iter().collect::<Vec<_>>());
            let result = assess(&request);
            let requirement = &result["requirements"][0];
            assert_eq!(
                requirement["status"], "supported",
                "a Realization Review never makes or blocks Satisfied: {result}"
            );
            assert_eq!(requirement["verification_review"], "accepted", "{result}");
            assert_eq!(requirement["realization_review"], expected, "{result}");
            assert_eq!(requirement["realization_code_drift"], drift, "{result}");
            assert_eq!(
                requirement["explanation"]["realization_review"]["status"],
                expected
            );
            if drift == json!(true) {
                assert_eq!(
                    requirement["explanation"]["realization_review"]["reviewed_commit"],
                    "b".repeat(40)
                );
                drifted = Some(result.clone());
            }
        }
        let drifted = drifted.unwrap();
        request["reviews"] = json!([]);

        let postgres = Postgres::default().with_tag("17-alpine").start().await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres.get_host_port_ipv4(5432).await?
        ))
        .await?;
        crate::migrate(&pool).await?;
        let state = LedgerState {
            pool: pool.clone(),
            credentials: Arc::new(BTreeMap::new()),
            evidence: Default::default(),
        };
        sqlx::query("INSERT INTO deployment_assessments(project,deployment,release) VALUES('project','monitor-deployment','release')").execute(&pool).await?;
        publish(
            &state,
            "project",
            "monitor-deployment",
            1,
            &[],
            &drifted,
            &request,
        )
        .await?;
        let stored: Value = sqlx::query_scalar("SELECT assurance_record('assessment_requirement','project','monitor-deployment:restricted:1')").fetch_one(&pool).await?;
        assert_eq!(
            stored, drifted["requirements"][0],
            "typed storage keeps both review dimensions"
        );
        // The fixture evaluates at a fixed past instant; only a current projection exposes its explanation.
        sqlx::query("UPDATE assessment_requirements SET valid_until_ms=9007199254740991")
            .execute(&pool)
            .await?;
        sqlx::query("UPDATE deployment_assessments SET valid_until_ms=9007199254740991")
            .execute(&pool)
            .await?;
        let summary = read(&state, "project", "monitor-deployment", true).await?;
        let row = &summary["requirements"][0];
        assert_eq!(
            (
                &row["verification_review"],
                &row["realization_review"],
                &row["realization_code_drift"]
            ),
            (&json!("accepted"), &json!("accepted"), &json!(true)),
            "{summary}"
        );
        assert_eq!(
            row["explanation"]["realization_review"],
            json!({"status":"accepted","reviewed_commit":"b".repeat(40),"code_drift":true}),
            "an overview names the drifted review's commit without the full explanation: {summary}"
        );
        Ok(())
    }

    async fn store_run(pool: &PgPool, bundle: &RunBundle) -> Result<(), LedgerError> {
        let mut tx = pool.begin().await?;
        relational::store(
            &mut tx,
            "run",
            "project",
            &bundle.bundle_fingerprint,
            &to_value(&run::to_json(bundle))?,
        )
        .await?;
        sqlx::query("INSERT INTO run_bundles(project,run_id,revision,fingerprint,verification_target_fingerprint,model_fingerprint) VALUES('project',$1,$2,$3,$4,$5)").bind(&bundle.run_id).bind(bundle.bundle_revision as i64).bind(&bundle.bundle_fingerprint).bind(&bundle.verification_target_fingerprint).bind(&bundle.plan.model_fingerprint).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    #[tokio::test]
    async fn durable_invalidations_are_selective_atomic_and_expire_without_incoming_facts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().with_tag("17-alpine").start().await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres.get_host_port_ipv4(5432).await?
        ))
        .await?;
        crate::migrate(&pool).await?;
        let state = LedgerState {
            pool: pool.clone(),
            credentials: Arc::new(BTreeMap::new()),
            evidence: Default::default(),
        };
        sqlx::query("INSERT INTO deployment_assessments(project,deployment,release,authority,model,verification_target,generation,published_generation,valid_until_ms) VALUES('project','deployment','release','authority','model','verification_target',1,1,9007199254740991)").execute(&pool).await?;
        for (requirement, check) in [("first", "recovery"), ("second", "unrelated")] {
            sqlx::query("INSERT INTO assessment_requirements(project,deployment,requirement,generation,published_generation,status,verification_review,realization_review,summary,valid_until_ms) VALUES('project','deployment',$1,1,1,'unresolved','not-declared','not-declared','No evidence',9007199254740991)").bind(requirement).execute(&pool).await?;
            sqlx::query("INSERT INTO assessment_dependencies(project,deployment,requirement,kind,identity) VALUES('project','deployment',$1,'check',$2)").bind(requirement).bind(check).execute(&pool).await?;
        }
        let mut tx = pool.begin().await?;
        let bundle = evidence::request(10)["bundles"][0].clone();
        let fp = bundle["bundle_fingerprint"].as_str().unwrap();
        relational::store(&mut tx, "run", "project", fp, &bundle).await?;
        sqlx::query("INSERT INTO run_bundles(project,run_id,revision,fingerprint,verification_target_fingerprint,model_fingerprint) VALUES('project',$1,0,$2,$3,$4)").bind(bundle["run_id"].as_str()).bind(fp).bind(bundle["verification_target_fingerprint"].as_str()).bind(bundle["plan"]["model_fingerprint"].as_str()).execute(&mut *tx).await?;
        let before: Value = read(&state, "project", "deployment", true).await?;
        assert_eq!(
            before["assessment"]["state"], "ready",
            "uncommitted facts cannot dirty the read"
        );
        tx.commit().await?;
        let pending = read(&state, "project", "deployment", true).await?;
        assert_eq!(pending["assessment"]["state"], "pending");
        let events_before: i64 = sqlx::query_scalar("SELECT count(*) FROM assessment_events")
            .fetch_one(&pool)
            .await?;
        read(&state, "project", "deployment", true).await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM assessment_events")
                .fetch_one(&pool)
                .await?,
            events_before,
            "GET must not consume work"
        );
        events(&state).await?;
        let generations: Vec<(String, i64)> = sqlx::query_as(
            "SELECT requirement,generation FROM assessment_requirements ORDER BY requirement",
        )
        .fetch_all(&pool)
        .await?;
        assert_eq!(generations, vec![("first".into(), 2), ("second".into(), 1)]);
        // The stale worker must discard before it can touch the supplied result or input.
        publish(
            &state,
            "project",
            "deployment",
            1,
            &[],
            &Value::Null,
            &Value::Null,
        )
        .await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT published_generation FROM deployment_assessments")
                .fetch_one(&pool)
                .await?,
            1
        );
        sqlx::query("UPDATE assessment_requirements SET published_generation=generation,valid_until_ms=CASE requirement WHEN 'first' THEN 0 ELSE 9007199254740991 END").execute(&pool).await?;
        sqlx::query(
            "UPDATE deployment_assessments SET published_generation=generation,valid_until_ms=0",
        )
        .execute(&pool)
        .await?;
        assert_eq!(
            read(&state, "project", "deployment", true).await?["assessment"]["state"],
            "expired"
        );
        let restarted = LedgerState {
            pool: pool.clone(),
            credentials: Arc::new(BTreeMap::new()),
            evidence: Default::default(),
        };
        events(&restarted).await?;
        let generations: Vec<(String, i64)> = sqlx::query_as(
            "SELECT requirement,generation FROM assessment_requirements ORDER BY requirement",
        )
        .fetch_all(&pool)
        .await?;
        assert_eq!(generations, vec![("first".into(), 3), ("second".into(), 1)]);
        let mut leader = pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(848332106)")
            .execute(&mut *leader)
            .await?;
        tokio::time::timeout(std::time::Duration::from_secs(1), tick(&restarted)).await??;
        leader.rollback().await?;
        let request = evidence::request(10);
        let full = to_value(
            &azimuth::deployment_status::assess(&core_json(request.to_string().as_bytes())?)
                .unwrap(),
        )?;
        sqlx::query("INSERT INTO deployment_assessments(project,deployment,release) VALUES('project','monitor-deployment','release')").execute(&pool).await?;
        publish(
            &state,
            "project",
            "monitor-deployment",
            1,
            &[],
            &full,
            &request,
        )
        .await?;
        let stored:Value=sqlx::query_scalar("SELECT assurance_record('assessment_requirement','project','monitor-deployment:restricted:1')").fetch_one(&pool).await?;
        assert_eq!(
            stored, full["requirements"][0],
            "typed storage must preserve the complete native Requirement result"
        );
        let version: i32 = sqlx::query_scalar("SELECT max(version) FROM assurance_storage_schema")
            .fetch_one(&pool)
            .await?;
        assert_eq!(version, 10);
        sqlx::query("UPDATE deployment_assessments SET generation=2 WHERE project='project' AND deployment='monitor-deployment'").execute(&pool).await?;
        sqlx::query("UPDATE assessment_requirements SET generation=2 WHERE project='project' AND deployment='monitor-deployment'").execute(&pool).await?;
        let mut refreshed_request = request.clone();
        refreshed_request["evaluated_at_ms"] = json!(6);
        let refreshed = to_value(
            &azimuth::deployment_status::assess(&core_json(
                refreshed_request.to_string().as_bytes(),
            )?)
            .unwrap(),
        )?;
        publish(
            &state,
            "project",
            "monitor-deployment",
            2,
            &[("restricted".into(), 2)],
            &refreshed,
            &refreshed_request,
        )
        .await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM assurance_records WHERE kind='assessment_requirement'"
            )
            .fetch_one(&pool)
            .await?,
            1,
            "rebuildable projections retain only their current generation"
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM assurance_records WHERE kind='run'")
                .fetch_one(&pool)
                .await?,
            1,
            "immutable original evidence remains retained"
        );
        sqlx::query(
            "UPDATE deployment_assessments SET generation=3 WHERE deployment='monitor-deployment'",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "UPDATE assessment_requirements SET generation=3 WHERE deployment='monitor-deployment'",
        )
        .execute(&pool)
        .await?;
        let mut concurrent = pool.begin().await?;
        relational::store(&mut concurrent,"run_attribution","project",&format!("{fp}:other-producer"),&json!({"run":bundle["run_id"],"bundle_fingerprint":fp,"submitted_by":"other-producer","received_at_ms":100})).await?;
        let changed_requirements = [("restricted".into(), 3)];
        let publish_job = publish(
            &state,
            "project",
            "monitor-deployment",
            3,
            &changed_requirements,
            &refreshed,
            &refreshed_request,
        );
        let commit_job = async { concurrent.commit().await.map_err(LedgerError::from) };
        let (published, committed) =
            tokio::time::timeout(std::time::Duration::from_secs(3), async {
                tokio::join!(publish_job, commit_job)
            })
            .await?;
        published?;
        committed?;
        assert!(sqlx::query_scalar::<_,i64>("SELECT generation FROM deployment_assessments WHERE deployment='monitor-deployment'").fetch_one(&pool).await?>=4,"concurrent evidence must invalidate or supersede publication without a header/Requirement deadlock");
        let authority = &request["publication"]["snapshot"]["authority"];
        let revision = authority["fingerprint"].as_str().unwrap();
        let mut registered = pool.begin().await?;
        relational::store(&mut registered, "authority", "project", revision, authority).await?;
        sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES('project',$1,'owner')").bind(revision).execute(&mut *registered).await?;
        sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES('project',$1,'owner')").bind(revision).execute(&mut *registered).await?;
        let mut review = json!({"kind":"verification-review","requirement":"restricted","requirement_fingerprint":authority["requirements"][0]["fingerprint"],
            "dependencies_fingerprint":assurance_review::requirement_closure_fingerprint(&core_json(authority.to_string().as_bytes())?,"restricted")?,
            "decision":"accepted","reassesses":null,"challenge_basis":[],"reviewer":"reviewer","reviewed_at_ms":100,"rationale":"Reviewed the exact declared account.",
            "artifacts":[{"id":"notes","digest":format!("sha256:{}","a".repeat(64)),"locator":"reviews/notes.md"}]});
        review["fingerprint"] = json!(assurance_review::record_fingerprint(&core_json(
            review.to_string().as_bytes()
        )?)?);
        let record_fp = review["fingerprint"].as_str().unwrap();
        assurance_review::validate_record(&core_json(review.to_string().as_bytes())?)?;
        relational::store(&mut registered, "review", "project", record_fp, &review).await?;
        let target = assurance_review::review_target(&core_json(review.to_string().as_bytes())?)?;
        let target_fp = run::canonical_fingerprint(&target)?;
        sqlx::query("INSERT INTO assurance_reviews(project,fingerprint,target_fingerprint,submitted_by) VALUES('project',$1,$2,'reviewer')").bind(record_fp).bind(target_fp).execute(&mut *registered).await?;
        sqlx::query("INSERT INTO model_revision_reviews(project,revision_fingerprint,review_fingerprint,submitted_by) VALUES('project',$1,$2,'reviewer')").bind(revision).bind(record_fp).execute(&mut *registered).await?;
        registered.commit().await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["state"],
            "pending"
        );
        materialized_reviews::process(&state).await?;
        let reviews = materialized_reviews::read(&state, "project", Some(revision))
            .await?
            .0;
        assert_eq!(reviews["assessment"]["state"], "ready", "{reviews}");
        assert_eq!(
            reviews["requirements"][0]["assessment"], "accepted",
            "{reviews}"
        );
        assert_eq!(
            reviews["requirements"][0]["record"], review,
            "original independently authored review must remain intact"
        );
        let review_generation = reviews["assessment"]["generation"].clone();
        sqlx::query(
            "INSERT INTO definition_events(project,kind,record_key) VALUES('project','run',$1)",
        )
        .bind(fp)
        .execute(&pool)
        .await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["state"],
            "ready",
            "ordinary unrelated Monitor execution cannot invalidate definition review"
        );
        materialized_reviews::process(&state).await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["generation"],
            review_generation
        );
        sqlx::query("UPDATE definition_assessments SET assessor='previous-build'")
            .execute(&pool)
            .await?;
        materialized_reviews::process(&state).await?;
        let reassessed = materialized_reviews::read(&state, "project", Some(revision))
            .await?
            .0;
        assert_eq!(
            reassessed["assessment"]["generation"].as_i64().unwrap(),
            review_generation.as_i64().unwrap() + 1,
            "a different assessing build recomputes stored assessments without any event"
        );
        assert_eq!(reassessed["assessment"]["state"], "ready", "{reassessed}");
        materialized_reviews::process(&state).await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["generation"],
            reassessed["assessment"]["generation"],
            "the same build does not recompute again"
        );
        let publication = &request["publication"];
        let release_fp = publication["release"]["fingerprint"].as_str().unwrap();
        let mut released = pool.begin().await?;
        relational::store(&mut released, "release", "project", release_fp, publication).await?;
        sqlx::query("INSERT INTO assurance_releases(project,fingerprint,model_revision,label,submitted_by) VALUES('project',$1,$2,'Synthetic release','publisher')").bind(release_fp).bind(revision).execute(&mut *released).await?;
        release::insert_deployment(
            &mut released,
            "project",
            release_fp,
            &request["deployment"],
            "publisher",
        )
        .await?;
        sqlx::query("UPDATE deployment_assessments SET release=$1 WHERE project='project' AND deployment='monitor-deployment'").bind(release_fp).execute(&mut *released).await?;
        released.commit().await?;
        let (computed, complete_input) =
            deployment_status::compute_deployment(&state, "project", "monitor-deployment", None)
                .await?;
        assert_eq!(
            computed["requirements"][0]["verification_review"], "accepted",
            "{computed}"
        );
        let generation: i64 = sqlx::query_scalar(
            "SELECT generation FROM deployment_assessments WHERE deployment='monitor-deployment'",
        )
        .fetch_one(&pool)
        .await?;
        let requirement_generation:i64=sqlx::query_scalar("SELECT generation FROM assessment_requirements WHERE deployment='monitor-deployment' AND requirement='restricted'").fetch_one(&pool).await?;
        publish(
            &state,
            "project",
            "monitor-deployment",
            generation,
            &[("restricted".into(), requirement_generation)],
            &computed,
            &complete_input,
        )
        .await?;
        let before_challenge: i64 = sqlx::query_scalar(
            "SELECT generation FROM deployment_assessments WHERE deployment='monitor-deployment'",
        )
        .fetch_one(&pool)
        .await?;
        let challenged = evidence::verification_challenge(
            record_fp,
            authority["model_fingerprint"].as_str().unwrap(),
        );
        store_run(&pool, &challenged).await?;
        assert!(sqlx::query_scalar::<_,i64>("SELECT generation FROM deployment_assessments WHERE deployment='monitor-deployment'").fetch_one(&pool).await?>before_challenge,"exact review target also invalidates the dependent deployment Requirement");
        let (adverse, complete_input) =
            deployment_status::compute_deployment(&state, "project", "monitor-deployment", None)
                .await?;
        assert!(
            complete_input["bundles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|bundle| bundle["bundle_fingerprint"] == challenged.bundle_fingerprint),
            "Challenge-only review evidence must survive the bounded deployment input selection"
        );
        assert_eq!(
            adverse["requirements"][0]["verification_review"], "unresolved",
            "{adverse}"
        );

        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["state"],
            "pending",
            "new exact-target Verification Challenge invalidates its definition revision"
        );
        materialized_reviews::process(&state).await?;
        let challenged_review = materialized_reviews::read(&state, "project", Some(revision))
            .await?
            .0;
        assert_eq!(
            challenged_review["requirements"][0]["assessment"], "unresolved",
            "{challenged_review}"
        );
        let corrected = evidence::correction(&challenged, ChallengeOutcome::Clean);
        assert!(run::verify_set(&[challenged.clone(), corrected.clone()]).is_empty());
        store_run(&pool, &corrected).await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["state"],
            "pending"
        );
        materialized_reviews::process(&state).await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["requirements"][0]["assessment"],
            "accepted"
        );
        let mut successor = review.clone();
        successor["reviewed_at_ms"] = json!(200);
        successor["reassesses"] = json!(record_fp);
        successor["challenge_basis"] = json!([{"run":corrected.run_id,"bundle_fingerprint":corrected.bundle_fingerprint,"result_fingerprint":corrected.verification_challenge_executions[0].result.fingerprint,"target_fingerprint":corrected.verification_challenge_executions[0].target.fingerprint}]);
        successor.as_object_mut().unwrap().remove("fingerprint");
        successor["fingerprint"] = json!(assurance_review::record_fingerprint(&core_json(
            successor.to_string().as_bytes()
        )?)?);
        let successor_fp = successor["fingerprint"].as_str().unwrap();
        let mut tx = pool.begin().await?;
        relational::store(&mut tx, "review", "project", successor_fp, &successor).await?;
        sqlx::query("INSERT INTO assurance_reviews(project,fingerprint,target_fingerprint,submitted_by) VALUES('project',$1,$2,'reviewer')").bind(successor_fp).bind(run::canonical_fingerprint(&assurance_review::review_target(&core_json(successor.to_string().as_bytes())?)?)?).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO model_revision_reviews(project,revision_fingerprint,review_fingerprint,submitted_by) VALUES('project',$1,$2,'reviewer')").bind(revision).bind(successor_fp).execute(&mut *tx).await?;
        tx.commit().await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["state"],
            "pending"
        );
        materialized_reviews::process(&state).await?;
        let successor_review = materialized_reviews::read(&state, "project", Some(revision))
            .await?
            .0;
        assert_eq!(successor_review["requirements"][0]["record"], successor);
        assert_eq!(
            successor_review["requirements"][0]["assessment"], "accepted",
            "{successor_review}"
        );
        let corrected_again = evidence::correction(&corrected, ChallengeOutcome::Findings);
        store_run(&pool, &corrected_again).await?;
        assert_eq!(
            materialized_reviews::read(&state, "project", Some(revision))
                .await?
                .0["assessment"]["state"],
            "pending"
        );
        materialized_reviews::process(&state).await?;
        let stale = materialized_reviews::read(&state, "project", Some(revision))
            .await?
            .0;
        assert_eq!(
            stale["requirements"][0]["assessment"], "stale",
            "corrected relied-on Challenge basis must invalidate the successor: {stale}"
        );
        let mut old_snapshot = pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .execute(&mut *old_snapshot)
            .await?;
        sqlx::query("SELECT count(*) FROM deployment_assessments")
            .execute(&mut *old_snapshot)
            .await?;
        sqlx::query("INSERT INTO assessment_events(project,kind,record_key) VALUES('project','deployment','bootstrap-deployment')").execute(&mut *old_snapshot).await?;
        sqlx::query("INSERT INTO deployment_assessments(project,deployment,release) VALUES('project','bootstrap-deployment','release')").execute(&pool).await?;
        old_snapshot.commit().await?;
        publish(
            &state,
            "project",
            "bootstrap-deployment",
            1,
            &[],
            &Value::Null,
            &Value::Null,
        )
        .await?;
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT published_generation FROM deployment_assessments WHERE deployment='bootstrap-deployment'").fetch_one(&pool).await?,0,"a fact transaction predating the initial header must prevent stale initial publication");
        events(&state).await?;
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT generation FROM deployment_assessments WHERE deployment='bootstrap-deployment'").fetch_one(&pool).await?,2,"a snapshot that predates initialization is replayed against current dependency rows");
        let columns:Vec<String>=sqlx::query_scalar("SELECT table_name||'.'||column_name FROM information_schema.columns WHERE table_schema='public' AND udt_name IN ('json','jsonb')").fetch_all(&pool).await?;
        assert!(columns.is_empty(), "{columns:?}");
        Ok(())
    }

    async fn migrated_state(
    ) -> Result<(LedgerState, PgPool, impl Sized), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().with_tag("17-alpine").start().await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres.get_host_port_ipv4(5432).await?
        ))
        .await?;
        crate::migrate(&pool).await?;
        let state = LedgerState {
            pool: pool.clone(),
            credentials: Arc::new(BTreeMap::new()),
            evidence: Default::default(),
        };
        Ok((state, pool, postgres))
    }

    async fn requirement_generation(pool: &PgPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT generation FROM assessment_requirements WHERE requirement='restricted'",
        )
        .fetch_one(pool)
        .await
    }

    async fn store_deployment_capture(
        pool: &PgPool,
        key: &str,
        finished_at_ms: i64,
        mode: &str,
    ) -> Result<(), LedgerError> {
        let snapshot = json!({"captured_at_ms":finished_at_ms,"instance":"process-a","report":{"format":"azimuth-execution-context","version":2,"contexts":{"settings":{"contract_revision":"contract-a","values":{"mode":mode}}}}});
        let capture = json!({"kind":"deployment","activity":null,"run":null,"bundle_fingerprint":null,"verification_target_fingerprint":"deployment-target","component":"api","instance":"process-a","started_at_ms":finished_at_ms,"finished_at_ms":finished_at_ms,"before":snapshot,"after":snapshot,"trusted":true});
        let mut tx = pool.begin().await?;
        relational::store(&mut tx, "context_capture", "project", key, &json!({"fingerprint":key,"capture":capture,"submitted_by":"collector","received_at_ms":finished_at_ms})).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn store_observation(
        pool: &PgPool,
        revision: i64,
        observed_at_ms: i64,
        instances: &[&str],
    ) -> Result<(), LedgerError> {
        let observation = json!({"format":"azimuth-deployment-observation","version":1,"observed_at_ms":observed_at_ms,"valid_for_seconds":60,"state":"complete","release_fingerprint":"release","applied_revision":"a".repeat(40),"rollout_id":"rollout-1","configuration_fingerprint":"configuration","components":[{"name":"api","image":"registry/api@digest","template_fingerprint":"template","ready":true,"instances":instances}],"diagnostics":[]});
        let fingerprint = format!("observation-{revision}");
        let mut tx = pool.begin().await?;
        relational::store(
            &mut tx,
            "deployment_observation",
            "project",
            &fingerprint,
            &observation,
        )
        .await?;
        sqlx::query("INSERT INTO deployment_observations(project,environment,revision,fingerprint,submitted_by,deployment_id,observed_at_ms,valid_for_seconds,state,release_fingerprint) VALUES('project','production',$1,$2,'reconciler',NULL,$3,60,'complete','release')").bind(revision).bind(&fingerprint).bind(observed_at_ms).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    #[tokio::test]
    async fn heartbeats_repeating_the_previous_fact_do_not_invalidate_but_changes_do(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (_state, pool, _postgres) = migrated_state().await?;
        sqlx::query("INSERT INTO deployment_assessments(project,deployment,release,authority,model,verification_target,generation,published_generation,valid_until_ms,observed_current) VALUES('project','deployment','release','authority','model','deployment-target',1,1,9007199254740991,true)").execute(&pool).await?;
        sqlx::query("INSERT INTO assessment_requirements(project,deployment,requirement,generation,published_generation,status,verification_review,realization_review,summary,valid_until_ms) VALUES('project','deployment','restricted',1,1,'unresolved','not-declared','not-declared','No evidence',9007199254740991)").execute(&pool).await?;
        for (kind, identity) in [("context", "settings"), ("environment", "production")] {
            sqlx::query("INSERT INTO assessment_dependencies(project,deployment,requirement,kind,identity) VALUES('project','deployment','restricted',$1,$2)").bind(kind).bind(identity).execute(&pool).await?;
        }

        store_deployment_capture(&pool, "capture-1", 100, "restricted").await?;
        assert_eq!(
            requirement_generation(&pool).await?,
            2,
            "the first capture of an instance is new evidence"
        );
        store_deployment_capture(&pool, "capture-2", 130, "restricted").await?;
        assert_eq!(
            requirement_generation(&pool).await?,
            2,
            "an identical later capture only extends freshness"
        );
        store_deployment_capture(&pool, "capture-3", 160, "open").await?;
        assert_eq!(
            requirement_generation(&pool).await?,
            3,
            "a changed effective context invalidates"
        );

        store_observation(&pool, 1, 1_000, &["process-a"]).await?;
        assert_eq!(
            requirement_generation(&pool).await?,
            4,
            "the first observation of an environment is new evidence"
        );
        store_observation(&pool, 2, 2_000, &["process-a"]).await?;
        assert_eq!(
            requirement_generation(&pool).await?,
            4,
            "an identical composition only extends freshness"
        );
        store_observation(&pool, 3, 3_000, &["process-a", "process-b"]).await?;
        assert_eq!(
            requirement_generation(&pool).await?,
            5,
            "a changed composition invalidates"
        );
        Ok(())
    }

    #[tokio::test]
    async fn an_overtaken_computation_publishes_requirements_whose_evidence_is_unchanged(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (state, pool, _postgres) = migrated_state().await?;
        let request = evidence::request(10);
        let result = to_value(
            &azimuth::deployment_status::assess(&core_json(request.to_string().as_bytes())?)
                .unwrap(),
        )?;
        sqlx::query("INSERT INTO deployment_assessments(project,deployment,release) VALUES('project','deployment','release')").execute(&pool).await?;
        publish(&state, "project", "deployment", 1, &[], &result, &request).await?;
        sqlx::query("INSERT INTO assessment_requirements(project,deployment,requirement,generation,published_generation,status,verification_review,realization_review,summary,valid_until_ms) VALUES('project','deployment','other',1,1,'unresolved','not-declared','not-declared','No evidence',9007199254740991)").execute(&pool).await?;

        // Evidence for both Requirements arrives and the computation starts; then more evidence reaches only `other`.
        sqlx::query("UPDATE deployment_assessments SET generation=2")
            .execute(&pool)
            .await?;
        sqlx::query("UPDATE assessment_requirements SET generation=2")
            .execute(&pool)
            .await?;
        sqlx::query("UPDATE deployment_assessments SET generation=3")
            .execute(&pool)
            .await?;
        sqlx::query("UPDATE assessment_requirements SET generation=3 WHERE requirement='other'")
            .execute(&pool)
            .await?;
        publish(
            &state,
            "project",
            "deployment",
            2,
            &[("restricted".into(), 2)],
            &result,
            &request,
        )
        .await?;

        let published: i64 = sqlx::query_scalar("SELECT published_generation FROM assessment_requirements WHERE requirement='restricted'").fetch_one(&pool).await?;
        assert_eq!(
            published, 2,
            "the Requirement whose evidence did not change is published"
        );
        let header: i64 =
            sqlx::query_scalar("SELECT published_generation FROM deployment_assessments")
                .fetch_one(&pool)
                .await?;
        assert_eq!(
            header, 1,
            "the assessment stays behind while another Requirement awaits evaluation"
        );
        sqlx::query("UPDATE assessment_requirements SET valid_until_ms=9007199254740991")
            .execute(&pool)
            .await?;
        let view = read(&state, "project", "deployment", true).await?;
        assert_eq!(view["assessment"]["state"], "pending");
        for entry in view["requirements"].as_array().unwrap() {
            let pending = entry["explanation"]["reasons"]
                .as_array()
                .is_some_and(|reasons| {
                    reasons
                        .iter()
                        .any(|reason| reason["code"] == "assessment-pending")
                });
            assert_eq!(pending, entry["requirement"] == "other", "{entry}");
        }

        // Once nothing awaits evaluation the assessment catches up to the current generation.
        sqlx::query("UPDATE assessment_requirements SET published_generation=generation WHERE requirement='other'").execute(&pool).await?;
        sqlx::query("UPDATE deployment_assessments SET generation=4")
            .execute(&pool)
            .await?;
        sqlx::query(
            "UPDATE assessment_requirements SET generation=4 WHERE requirement='restricted'",
        )
        .execute(&pool)
        .await?;
        publish(
            &state,
            "project",
            "deployment",
            4,
            &[("restricted".into(), 4)],
            &result,
            &request,
        )
        .await?;
        let header: i64 =
            sqlx::query_scalar("SELECT published_generation FROM deployment_assessments")
                .fetch_one(&pool)
                .await?;
        assert_eq!(header, 4);
        Ok(())
    }
    #[tokio::test]
    async fn a_requirement_is_renewed_before_it_expires_without_reading_as_pending(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().with_tag("17-alpine").start().await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres.get_host_port_ipv4(5432).await?
        ))
        .await?;
        crate::migrate(&pool).await?;
        let state = LedgerState {
            pool: pool.clone(),
            credentials: Arc::new(BTreeMap::new()),
            evidence: Default::default(),
        };
        let now = time(&mut *pool.acquire().await?).await?;
        sqlx::query("INSERT INTO deployment_assessments(project,deployment,release,authority,model,verification_target,generation,published_generation,valid_until_ms,evaluated_at_ms) VALUES('project','deployment','release','authority','model','verification_target',1,1,$1,$2)").bind(now + 20_000).bind(now - 100_000).execute(&pool).await?;
        for (requirement, lifetime) in [("renewed", 20_000), ("distant", 3_600_000)] {
            sqlx::query("INSERT INTO assessment_requirements(project,deployment,requirement,generation,published_generation,status,verification_review,realization_review,summary,valid_until_ms,evaluated_at_ms) VALUES('project','deployment',$1,1,1,'unresolved','not-declared','not-declared','No evidence',$2,$3)").bind(requirement).bind(now + lifetime).bind(now - 100_000).execute(&pool).await?;
        }
        events(&state).await?;
        let rows: Vec<(String, i64, bool)> = sqlx::query_as(
            "SELECT requirement,generation,renewing FROM assessment_requirements ORDER BY requirement",
        )
        .fetch_all(&pool)
        .await?;
        assert_eq!(
            rows,
            vec![("distant".into(), 1, false), ("renewed".into(), 2, true)]
        );
        let view = read(&state, "project", "deployment", true).await?;
        assert_eq!(
            view["assessment"]["state"], "ready",
            "renewal alone must not hide the published assessment"
        );
        assert!(view["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .all(|requirement| requirement["diagnostics"].as_array().unwrap().is_empty()));
        sqlx::query("UPDATE assessment_requirements SET generation=generation+1,renewing=false WHERE requirement='renewed'").execute(&pool).await?;
        let invalidated = read(&state, "project", "deployment", true).await?;
        assert_eq!(invalidated["assessment"]["state"], "pending");
        Ok(())
    }
}
