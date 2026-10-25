use super::*;

const RELEVANT_EVENT: &str = r#"
    e.project=d.project AND (d.published_generation=0 OR (
        e.kind IN ('authority','revision') AND e.record_key=d.revision
        OR e.kind='review' AND EXISTS(SELECT 1 FROM model_revision_reviews linked WHERE linked.project=d.project AND linked.revision_fingerprint=d.revision AND linked.review_fingerprint=e.record_key)
        OR e.kind='run' AND (
            EXISTS(SELECT 1 FROM definition_run_dependencies dependency JOIN run_bundles b ON b.project=dependency.project AND b.run_id=dependency.run WHERE dependency.project=d.project AND dependency.revision=d.revision AND b.fingerprint=e.record_key)
            OR EXISTS(SELECT 1 FROM definition_requirements c CROSS JOIN LATERAL jsonb_array_elements(CASE WHEN EXISTS(SELECT 1 FROM assurance_records a JOIN {verification_challenge_executions} target ON target._record_id=a.id WHERE a.kind='run' AND a.project=e.project AND a.record_key=e.record_key) THEN assurance_record_field('run',e.project,e.record_key,'verification_challenge_executions') ELSE '[]'::jsonb END) execution WHERE c.project=d.project AND c.revision=d.revision AND execution->'target'->>'fingerprint'=c.record_fingerprint)
        )
    ))
"#;

fn relevant_event() -> String {
    relational::with_tables(RELEVANT_EVENT)
}

pub(super) async fn process(state: &LedgerState) -> Result<(), LedgerError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("INSERT INTO definition_assessments(project,revision) SELECT project,fingerprint FROM model_revisions ON CONFLICT DO NOTHING").execute(&mut *tx).await?;
    let sequences: Vec<i64> = sqlx::query_scalar(
        "SELECT sequence FROM definition_events ORDER BY sequence LIMIT 1000 FOR UPDATE",
    )
    .fetch_all(&mut *tx)
    .await?;
    let relevant = relevant_event();
    sqlx::query(&format!("UPDATE definition_assessments d SET generation=generation+1,retry_at=now() WHERE EXISTS(SELECT 1 FROM definition_events e WHERE e.sequence=ANY($1) AND {relevant})")).bind(&sequences).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM definition_events WHERE sequence=ANY($1)")
        .bind(&sequences)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let jobs:Vec<(String,String,i64)>=sqlx::query_as("SELECT project,revision,generation FROM definition_assessments WHERE generation<>published_generation AND retry_at<=now() ORDER BY (EXISTS(SELECT 1 FROM deployment_assessments a WHERE a.project=definition_assessments.project AND a.authority=definition_assessments.revision AND a.observed_current)) DESC,retry_at LIMIT 1").fetch_all(&state.pool).await?;
    for (project, revision, generation) in jobs {
        if let Err(error) = refresh(state, &project, &revision, generation).await {
            tracing::error!(%error,project,revision,"definition review assessment remains pending");
            sqlx::query("UPDATE definition_assessments SET last_error=$3,retry_at=now()+interval '10 seconds' WHERE project=$1 AND revision=$2").bind(project).bind(revision).bind(error.detail).execute(&state.pool).await?;
        }
    }
    Ok(())
}

async fn refresh(
    state: &LedgerState,
    project: &str,
    revision: &str,
    generation: i64,
) -> Result<(), LedgerError> {
    let (authority, reviews, bundles) =
        load_revision_snapshot(&state.pool, project, Some(revision)).await?;
    let challenge_runs: Vec<String> = bundles
        .iter()
        .filter(|bundle| !bundle.verification_challenge_executions.is_empty())
        .map(|bundle| bundle.run_id.clone())
        .collect();
    let result = compute(move || definition_reviews(&authority, &reviews, &bundles))
        .await?
        .0;
    let mut tx = state.pool.begin().await?;
    let current:(i64,i64)=sqlx::query_as("SELECT generation,published_generation FROM definition_assessments WHERE project=$1 AND revision=$2 FOR UPDATE").bind(project).bind(revision).fetch_one(&mut *tx).await?;
    let relevant = relevant_event();
    let pending:bool=sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM definition_assessments d JOIN definition_events e ON {relevant} WHERE d.project=$1 AND d.revision=$2)")).bind(project).bind(revision).fetch_one(&mut *tx).await?;
    if current.0 != generation || current.1 == generation || pending {
        return Ok(());
    }
    sqlx::query("DELETE FROM definition_run_dependencies WHERE project=$1 AND revision=$2")
        .bind(project)
        .bind(revision)
        .execute(&mut *tx)
        .await?;
    for run in challenge_runs {
        sqlx::query("INSERT INTO definition_run_dependencies(project,revision,run) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(project).bind(revision).bind(run).execute(&mut *tx).await?;
    }
    for requirement in result["requirements"].as_array().unwrap() {
        let diagnostics: Vec<&str> = requirement["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        sqlx::query("INSERT INTO definition_requirements(project,revision,requirement,kind,record_fingerprint,status,code_drift,diagnostics) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(project,revision,requirement,kind) DO UPDATE SET record_fingerprint=excluded.record_fingerprint,status=excluded.status,code_drift=excluded.code_drift,diagnostics=excluded.diagnostics").bind(project).bind(revision).bind(requirement["requirement"].as_str()).bind(requirement["kind"].as_str()).bind(requirement["record"]["fingerprint"].as_str()).bind(requirement["assessment"].as_str()).bind(requirement["code_drift"].as_bool()).bind(diagnostics).execute(&mut *tx).await?;
        if let Some(basis) = requirement["record"]["challenge_basis"].as_array() {
            for item in basis {
                if let Some(run) = item["run"].as_str() {
                    sqlx::query("INSERT INTO definition_run_dependencies(project,revision,run) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(project).bind(revision).bind(run).execute(&mut *tx).await?;
                }
            }
        }
    }
    sqlx::query("UPDATE definition_assessments SET published_generation=$3,evaluated_at_ms=(extract(epoch FROM clock_timestamp())*1000)::bigint,last_error=NULL WHERE project=$1 AND revision=$2").bind(project).bind(revision).bind(generation).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn read(
    state: &LedgerState,
    project: &str,
    revision: Option<&str>,
) -> Result<Json<Value>, LedgerError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let selected:Option<String>=sqlx::query_scalar(&format!("SELECT fingerprint FROM model_revisions WHERE project=$1 AND fingerprint=$2 UNION ALL ({DEFAULT_MODEL_SELECT})")).bind(project).bind(revision).fetch_optional(&mut *tx).await?;
    let revision = selected.ok_or_else(LedgerError::not_found)?;
    let relevant = relevant_event();
    let row=sqlx::query(&format!("SELECT d.*,EXISTS(SELECT 1 FROM definition_events e WHERE {relevant}) AS pending FROM definition_assessments d WHERE d.project=$1 AND d.revision=$2")).bind(project).bind(&revision).fetch_optional(&mut *tx).await?;
    let Some(row) = row else {
        return Ok(Json(
            json!({"authority_fingerprint":revision,"requirements":[],"assessment":{"state":"pending","generation":0,"published_generation":0,"evaluated_at_ms":null,"valid_until_ms":null}}),
        ));
    };
    let generation: i64 = row.get("generation");
    let published: i64 = row.get("published_generation");
    let ready = generation == published && published > 0 && !row.get::<bool, _>("pending");
    let entries=sqlx::query("SELECT requirement,kind,status,code_drift,diagnostics,record_fingerprint FROM definition_requirements WHERE project=$1 AND revision=$2 ORDER BY requirement,kind='realization-review'").bind(project).bind(&revision).fetch_all(&mut *tx).await?;
    let mut requirements = Vec::new();
    for entry in entries {
        let record: Option<Value> = if ready {
            match entry.get::<Option<String>, _>("record_fingerprint") {
                Some(fp) => Some(
                    sqlx::query_scalar("SELECT assurance_record('review',$1,$2)")
                        .bind(project)
                        .bind(fp)
                        .fetch_one(&mut *tx)
                        .await?,
                ),
                None => None,
            }
        } else {
            None
        };
        requirements.push(json!({"kind":entry.get::<String,_>("kind"),"requirement":entry.get::<String,_>("requirement"),"record":record,"assessment":if ready {entry.get::<String,_>("status")} else {"pending".into()},"code_drift":if ready {entry.get::<Option<bool>,_>("code_drift")} else {None},"diagnostics":if ready {entry.get::<Vec<String>,_>("diagnostics")} else {vec!["Review assessment is updating".into()]}}));
    }
    Ok(Json(
        json!({"authority_fingerprint":revision,"requirements":requirements,"assessment":{"state":if ready {"ready"} else {"pending"},"generation":generation,"published_generation":published,"evaluated_at_ms":row.get::<Option<i64>,_>("evaluated_at_ms"),"valid_until_ms":null}}),
    ))
}
