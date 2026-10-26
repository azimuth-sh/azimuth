//! Work Provenance storage and reads (contracts/work-provenance.md, "Assurance Service").
//!
//! Provenance is never an assessment input: nothing here is read by review currentness,
//! Requirement Status or deployment assessment, and these tables raise no assessment events.

use super::*;
use sqlx::{postgres::PgRow, PgConnection};
use std::collections::BTreeSet;

const LEDGER_FORMAT: &str = "azimuth-work-provenance-ledger";
const ATTRIBUTION_FORMAT: &str = "azimuth-failure-attribution";
const PROVENANCE_BASIS: &str = "publisher-transported-assertion";

pub(super) async fn submit_ledger(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let publisher = state.authorize(&project, &headers, Some(Role::Publisher))?;
    let (ledger, fingerprint) = compute(move || {
        let native = core_json(&body)?;
        azimuth::provenance::validate_ledger(&native).map_err(LedgerError::invalid)?;
        Ok((
            to_value(&native)?,
            azimuth::provenance::ledger_fingerprint(&native),
        ))
    })
    .await?;
    if ledger["project"].as_str() != Some(&project) {
        return Err(LedgerError::invalid("ledger project must match route"));
    }
    let change = text(&ledger, "change")?.to_owned();
    let mut tx = state.pool.begin().await?;
    // Exports of one change must append in a single order so the stored ledger stays one lineage.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|provenance-ledger|{change}"))
        .execute(&mut *tx)
        .await?;
    let replayed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM provenance_ledgers WHERE project=$1 AND change=$2 AND ledger_fingerprint=$3)")
        .bind(&project).bind(&change).bind(&fingerprint).fetch_one(&mut *tx).await?;
    if replayed {
        tx.commit().await?;
        return Ok((
            StatusCode::OK,
            Json(ledger_receipt(&project, &change, &fingerprint, true, 0, 0)),
        ));
    }
    let stored = reassemble_ledger(&mut tx, &project, &change).await?;
    if let Some(stored) = &stored {
        require_append(stored, &ledger)?;
    }
    let (instances_added, attempts_added) =
        write_ledger(&mut tx, &project, &change, &ledger, stored.as_ref()).await?;
    sqlx::query("INSERT INTO provenance_ledgers(project,change,ledger_fingerprint,submitted_by,provenance_basis) VALUES($1,$2,$3,$4,$5)")
        .bind(&project).bind(&change).bind(&fingerprint).bind(&publisher).bind(PROVENANCE_BASIS).execute(&mut *tx).await?;
    // Typed columns must reproduce the submitted ledger exactly; anything they cannot hold is refused.
    let reassembled = reassemble_ledger(&mut tx, &project, &change).await?;
    if reassembled.as_ref() != Some(&ledger) {
        return Err(LedgerError::invalid(
            "ledger contains content that relational provenance storage cannot reproduce exactly",
        ));
    }
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(ledger_receipt(
            &project,
            &change,
            &fingerprint,
            false,
            instances_added,
            attempts_added,
        )),
    ))
}

fn ledger_receipt(
    project: &str,
    change: &str,
    fingerprint: &str,
    replayed: bool,
    instances_added: usize,
    attempts_added: usize,
) -> Value {
    json!({"project":project,"change":change,"ledger_fingerprint":fingerprint,"replayed":replayed,
        "instances_added":instances_added,"attempts_added":attempts_added,"provenance_basis":PROVENANCE_BASIS})
}

fn instances(ledger: &Value) -> Result<&Vec<Value>, LedgerError> {
    ledger["instances"]
        .as_array()
        .ok_or_else(|| LedgerError::invalid("ledger instances require an array"))
}

fn attempts(instance: &Value) -> Result<&Vec<Value>, LedgerError> {
    instance["attempts"]
        .as_array()
        .ok_or_else(|| LedgerError::invalid("instance attempts require an array"))
}

fn find<'a>(items: &'a [Value], id: &str) -> Option<&'a Value> {
    items.iter().find(|item| item["id"].as_str() == Some(id))
}

// The ledger is append-only across exports: every stored instance and attempt must reappear, and
// only an active instance may still change its disposition.
fn require_append(stored: &Value, submitted: &Value) -> Result<(), LedgerError> {
    let submitted_instances = instances(submitted)?;
    for instance in instances(stored)? {
        let id = text(instance, "id")?;
        let Some(candidate) = find(submitted_instances, id) else {
            return Err(LedgerError::conflict(format!(
                "ledger omits stored instance {id}"
            )));
        };
        for field in ["predecessor", "definition_fingerprint", "template"] {
            if instance[field] != candidate[field] {
                return Err(LedgerError::conflict(format!(
                    "instance {id} changed its {field}"
                )));
            }
        }
        if instance["disposition"] != "active"
            && instance["disposition"] != candidate["disposition"]
        {
            return Err(LedgerError::conflict(format!(
                "instance {id} is {} and cannot change its disposition",
                instance["disposition"].as_str().unwrap_or_default()
            )));
        }
        let candidate_attempts = attempts(candidate)?;
        for attempt in attempts(instance)? {
            let attempt_id = text(attempt, "id")?;
            match find(candidate_attempts, attempt_id) {
                None => {
                    return Err(LedgerError::conflict(format!(
                        "ledger omits stored attempt {attempt_id}"
                    )))
                }
                Some(candidate) if candidate != attempt => {
                    return Err(LedgerError::conflict(format!(
                        "attempt {attempt_id} differs from its stored content"
                    )))
                }
                Some(_) => (),
            }
        }
    }
    Ok(())
}

async fn write_ledger(
    tx: &mut PgConnection,
    project: &str,
    change: &str,
    ledger: &Value,
    stored: Option<&Value>,
) -> Result<(usize, usize), LedgerError> {
    let stored_attempts: BTreeSet<&str> = stored
        .map(instances)
        .transpose()?
        .into_iter()
        .flatten()
        .flat_map(|instance| instance["attempts"].as_array().into_iter().flatten())
        .filter_map(|attempt| attempt["id"].as_str())
        .collect();
    let stored_instances: BTreeSet<&str> = stored
        .map(instances)
        .transpose()?
        .into_iter()
        .flatten()
        .filter_map(|instance| instance["id"].as_str())
        .collect();
    let (mut instances_added, mut attempts_added) = (0, 0);
    for (position, instance) in instances(ledger)?.iter().enumerate() {
        let instance_id = text(instance, "id")?;
        if !stored_instances.contains(instance_id) {
            instances_added += 1;
        }
        let template = &instance["template"];
        let predecessor = &instance["predecessor"];
        let (predecessor_id, predecessor_ref, predecessor_revision, predecessor_fingerprint) =
            if predecessor.is_null() {
                (None, None, None, None)
            } else {
                (
                    Some(text(predecessor, "id")?),
                    Some(text(predecessor, "ref")?),
                    Some(integer(predecessor, "revision")?),
                    Some(text(predecessor, "fingerprint")?),
                )
            };
        sqlx::query("INSERT INTO provenance_instances(project,change,instance_id,position,predecessor_id,predecessor_ref,predecessor_revision,predecessor_fingerprint,disposition,definition_fingerprint,template_id,template_revision) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) ON CONFLICT(project,change,instance_id) DO UPDATE SET position=excluded.position,disposition=excluded.disposition")
            .bind(project).bind(change).bind(instance_id).bind(index(position)?)
            .bind(predecessor_id).bind(predecessor_ref).bind(predecessor_revision).bind(predecessor_fingerprint)
            .bind(text(instance, "disposition")?)
            .bind(text(instance, "definition_fingerprint")?)
            .bind(if template.is_null() { None } else { Some(text(template, "id")?) })
            .bind(if template.is_null() { None } else { Some(integer(template, "revision")?) })
            .execute(&mut *tx).await?;
        for attempt in attempts(instance)? {
            let attempt_id = text(attempt, "id")?;
            if stored_attempts.contains(attempt_id) {
                continue;
            }
            let elsewhere: Option<String> = sqlx::query_scalar(
                "SELECT change FROM provenance_attempts WHERE project=$1 AND attempt_id=$2",
            )
            .bind(project)
            .bind(attempt_id)
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(elsewhere) = elsewhere {
                return Err(LedgerError::conflict(format!(
                    "attempt {attempt_id} is already recorded for change {elsewhere}"
                )));
            }
            insert_attempt(tx, project, change, instance_id, attempt).await?;
            attempts_added += 1;
        }
    }
    Ok((instances_added, attempts_added))
}

async fn insert_attempt(
    tx: &mut PgConnection,
    project: &str,
    change: &str,
    instance_id: &str,
    attempt: &Value,
) -> Result<(), LedgerError> {
    let attempt_id = text(attempt, "id")?;
    let agent = Descriptor::from_value(&attempt["agent"])?;
    let usage = Usage::from_value(&attempt["usage"])?;
    sqlx::query("INSERT INTO provenance_attempts(project,attempt_id,change,instance_id,sequence,action,task,scope,actor,outcome,report,handoff_captured_at_ms,recorded_at_ms,agent_kind,agent_kind_basis,model,model_basis,effort,effort_basis,agent_type,agent_type_basis,harness_name,harness_version,harness_basis,trace,usage_basis,model_calls,tool_calls,input_tokens,cache_read_input_tokens,cache_write_input_tokens,output_tokens,reasoning_output_tokens,usage_overlapping,delegated_agents,delegated_model_calls,delegated_input_tokens,delegated_cache_read_input_tokens,delegated_cache_write_input_tokens,delegated_output_tokens,delegated_reasoning_output_tokens) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34,$35,$36,$37,$38,$39,$40,$41)")
        .bind(project).bind(attempt_id).bind(change).bind(instance_id)
        .bind(i32::try_from(integer(attempt, "sequence")?).map_err(|_| LedgerError::invalid("attempt sequence is out of range"))?)
        .bind(text(attempt, "action")?).bind(text(attempt, "task")?).bind(text(attempt, "scope")?)
        .bind(text(attempt, "actor")?).bind(text(attempt, "outcome")?).bind(optional_text(attempt, "report")?)
        .bind(optional_integer(attempt, "handoff_captured_at_ms")?).bind(optional_integer(attempt, "recorded_at_ms")?)
        .bind(&agent.kind).bind(&agent.kind_basis).bind(&agent.model).bind(&agent.model_basis)
        .bind(&agent.effort).bind(&agent.effort_basis).bind(&agent.agent_type).bind(&agent.agent_type_basis)
        .bind(&agent.harness_name).bind(&agent.harness_version).bind(&agent.harness_basis)
        .bind(optional_text(attempt, "trace")?)
        .bind(&usage.basis).bind(usage.model_calls).bind(usage.tool_calls)
        .bind(usage.own.input).bind(usage.own.cache_read_input).bind(usage.own.cache_write_input)
        .bind(usage.own.output).bind(usage.own.reasoning_output).bind(usage.overlapping)
        .bind(usage.delegated_agents).bind(usage.delegated_model_calls)
        .bind(usage.delegated.input).bind(usage.delegated.cache_read_input).bind(usage.delegated.cache_write_input)
        .bind(usage.delegated.output).bind(usage.delegated.reasoning_output)
        .execute(&mut *tx).await?;
    for (position, target) in list(attempt, "target")?.iter().enumerate() {
        sqlx::query("INSERT INTO provenance_attempt_targets(project,attempt_id,position,name,value) VALUES($1,$2,$3,$4,$5)")
            .bind(project).bind(attempt_id).bind(index(position)?).bind(text(target, "name")?)
            .bind(text(target, "value")?).execute(&mut *tx).await?;
    }
    for (position, guidance) in list(attempt, "guidance")?.iter().enumerate() {
        sqlx::query("INSERT INTO provenance_attempt_guidance(project,attempt_id,position,kind,guidance_id,fingerprint) VALUES($1,$2,$3,$4,$5,$6)")
            .bind(project).bind(attempt_id).bind(index(position)?).bind(text(guidance, "kind")?)
            .bind(text(guidance, "id")?).bind(optional_text(guidance, "fingerprint")?).execute(&mut *tx).await?;
    }
    for role in ["used", "generated"] {
        for (position, entity) in list(attempt, role)?.iter().enumerate() {
            sqlx::query("INSERT INTO provenance_attempt_entities(project,attempt_id,role,position,kind,repository,ref,fingerprint,record,record_fingerprint) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
                .bind(project).bind(attempt_id).bind(role).bind(index(position)?)
                .bind(text(entity, "kind")?).bind(optional_text(entity, "repository")?)
                .bind(optional_text(entity, "ref")?).bind(optional_text(entity, "fingerprint")?)
                .bind(optional_text(entity, "record")?)
                .bind(optional_text(entity, "record_fingerprint")?).execute(&mut *tx).await?;
        }
    }
    Ok(())
}

async fn reassemble_ledger(
    conn: &mut PgConnection,
    project: &str,
    change: &str,
) -> Result<Option<Value>, LedgerError> {
    let submitted: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM provenance_ledgers WHERE project=$1 AND change=$2)",
    )
    .bind(project)
    .bind(change)
    .fetch_one(&mut *conn)
    .await?;
    if !submitted {
        return Ok(None);
    }
    let instance_rows = sqlx::query("SELECT instance_id,predecessor_id,predecessor_ref,predecessor_revision,predecessor_fingerprint,disposition,definition_fingerprint,template_id,template_revision FROM provenance_instances WHERE project=$1 AND change=$2 ORDER BY position")
        .bind(project).bind(change).fetch_all(&mut *conn).await?;
    let attempt_rows = sqlx::query("SELECT * FROM provenance_attempts WHERE project=$1 AND change=$2 ORDER BY instance_id,sequence")
        .bind(project).bind(change).fetch_all(&mut *conn).await?;
    let mut targets: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in sqlx::query("SELECT t.attempt_id,t.name,t.value FROM provenance_attempt_targets t JOIN provenance_attempts a USING(project,attempt_id) WHERE a.project=$1 AND a.change=$2 ORDER BY t.attempt_id,t.position")
        .bind(project).bind(change).fetch_all(&mut *conn).await? {
        targets.entry(row.try_get("attempt_id")?).or_default().push(
            json!({"name":row.try_get::<String,_>("name")?,"value":row.try_get::<String,_>("value")?}),
        );
    }
    let mut guidance: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in sqlx::query("SELECT g.attempt_id,g.kind,g.guidance_id,g.fingerprint FROM provenance_attempt_guidance g JOIN provenance_attempts a USING(project,attempt_id) WHERE a.project=$1 AND a.change=$2 ORDER BY g.attempt_id,g.position")
        .bind(project).bind(change).fetch_all(&mut *conn).await? {
        guidance.entry(row.try_get("attempt_id")?).or_default().push(json!({
            "kind":row.try_get::<String,_>("kind")?,"id":row.try_get::<String,_>("guidance_id")?,
            "fingerprint":row.try_get::<Option<String>,_>("fingerprint")?,
        }));
    }
    let mut entities: BTreeMap<(String, String), Vec<Value>> = BTreeMap::new();
    for row in sqlx::query("SELECT e.attempt_id,e.role,e.kind,e.repository,e.ref,e.fingerprint,e.record,e.record_fingerprint FROM provenance_attempt_entities e JOIN provenance_attempts a USING(project,attempt_id) WHERE a.project=$1 AND a.change=$2 ORDER BY e.attempt_id,e.role,e.position")
        .bind(project).bind(change).fetch_all(&mut *conn).await? {
        let role: String = row.try_get("role")?;
        let mut entity = json!({
            "kind":row.try_get::<String,_>("kind")?,"repository":row.try_get::<Option<String>,_>("repository")?,
            "ref":row.try_get::<Option<String>,_>("ref")?,"fingerprint":row.try_get::<Option<String>,_>("fingerprint")?,
        });
        if role == "generated" {
            entity["record"] = json!(row.try_get::<Option<String>, _>("record")?);
            entity["record_fingerprint"] =
                json!(row.try_get::<Option<String>, _>("record_fingerprint")?);
        }
        entities
            .entry((row.try_get("attempt_id")?, role))
            .or_default()
            .push(entity);
    }
    let mut by_instance: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in attempt_rows {
        let id: String = row.try_get("attempt_id")?;
        let attempt = json!({
            "id": id,
            "sequence": row.try_get::<i32,_>("sequence")?,
            "action": row.try_get::<String,_>("action")?,
            "task": row.try_get::<String,_>("task")?,
            "scope": row.try_get::<String,_>("scope")?,
            "target": targets.remove(&id).unwrap_or_default(),
            "actor": row.try_get::<String,_>("actor")?,
            "outcome": row.try_get::<String,_>("outcome")?,
            "report": row.try_get::<Option<String>,_>("report")?,
            "handoff_captured_at_ms": row.try_get::<Option<i64>,_>("handoff_captured_at_ms")?,
            "recorded_at_ms": row.try_get::<Option<i64>,_>("recorded_at_ms")?,
            "agent": Descriptor::from_row(&row)?.to_value(),
            "guidance": guidance.remove(&id).unwrap_or_default(),
            "trace": row.try_get::<Option<String>,_>("trace")?,
            "usage": Usage::from_row(&row)?.to_value(),
            "used": entities.remove(&(id.clone(), "used".into())).unwrap_or_default(),
            "generated": entities.remove(&(id.clone(), "generated".into())).unwrap_or_default(),
        });
        by_instance
            .entry(row.try_get("instance_id")?)
            .or_default()
            .push(attempt);
    }
    let mut instances = Vec::new();
    for row in instance_rows {
        let id: String = row.try_get("instance_id")?;
        instances.push(json!({
            "id": id,
            "predecessor": match row.try_get::<Option<String>,_>("predecessor_id")? {
                Some(predecessor) => json!({
                    "id": predecessor,
                    "ref": row.try_get::<String,_>("predecessor_ref")?,
                    "revision": row.try_get::<i64,_>("predecessor_revision")?,
                    "fingerprint": row.try_get::<String,_>("predecessor_fingerprint")?,
                }),
                None => Value::Null,
            },
            "disposition": row.try_get::<String,_>("disposition")?,
            "definition_fingerprint": row.try_get::<String,_>("definition_fingerprint")?,
            "template": match row.try_get::<Option<String>,_>("template_id")? {
                Some(template) => json!({"id": template, "revision": row.try_get::<i64,_>("template_revision")?}),
                None => Value::Null,
            },
            "attempts": by_instance.remove(&id).unwrap_or_default(),
        }));
    }
    Ok(Some(json!({
        "format": LEDGER_FORMAT, "version": 1, "project": project, "change": change,
        "instances": instances,
    })))
}

pub(super) async fn show_change(
    State(state): State<LedgerState>,
    Path((project, change)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let mut tx = read_snapshot(&state).await?;
    let ledger = reassemble_ledger(&mut tx, &project, &change).await?;
    tx.commit().await?;
    Ok(Json(ledger.ok_or_else(LedgerError::not_found)?))
}

// Rows come from the core dataset projection so the service and `azimuth report provenance` agree.
fn dataset_rows(ledger: &Value) -> Result<Vec<(String, Value)>, LedgerError> {
    let native = core_json(ledger.to_string().as_bytes())?;
    let rows = azimuth::provenance::dataset_rows(&native).map_err(LedgerError::invalid)?;
    let ids: Vec<String> = instances(ledger)?
        .iter()
        .flat_map(|instance| instance["attempts"].as_array().into_iter().flatten())
        .map(|attempt| text(attempt, "id").map(str::to_owned))
        .collect::<Result<_, _>>()?;
    if rows.len() != ids.len() {
        return Err(LedgerError::invalid(
            "dataset projection does not have one row per attempt",
        ));
    }
    ids.into_iter()
        .zip(rows)
        .map(|(id, row)| Ok((id, to_value(&row)?)))
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AttemptPage {
    cursor: Option<String>,
    limit: Option<i64>,
}

pub(super) async fn list_attempts(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<AttemptPage>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit.unwrap_or(25);
    let cursor = page.cursor.filter(|cursor| !cursor.is_empty());
    if !(1..=100).contains(&limit) || cursor.as_ref().is_some_and(|c| c.len() > 1024) {
        return Err(LedgerError::invalid(
            "limit must be 1..100 and cursor at most 1024 bytes",
        ));
    }
    let mut tx = read_snapshot(&state).await?;
    const ORDERED: &str = "SELECT a.attempt_id,a.change,i.position,a.sequence FROM provenance_attempts a JOIN provenance_instances i ON i.project=a.project AND i.change=a.change AND i.instance_id=a.instance_id WHERE a.project=$1";
    if let Some(cursor) = &cursor {
        let known: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM provenance_attempts WHERE project=$1 AND attempt_id=$2)",
        )
        .bind(&project)
        .bind(cursor)
        .fetch_one(&mut *tx)
        .await?;
        if !known {
            return Err(LedgerError::invalid("cursor names no stored attempt"));
        }
    }
    let rows = sqlx::query(&format!("WITH ordered AS ({ORDERED}) SELECT attempt_id,change FROM ordered WHERE $2::text IS NULL OR (change,position,sequence) > (SELECT change,position,sequence FROM ordered WHERE attempt_id=$2) ORDER BY change,position,sequence LIMIT $3"))
        .bind(&project).bind(&cursor).bind(limit + 1).fetch_all(&mut *tx).await?;
    let more = rows.len() > limit as usize;
    let page: Vec<(String, String)> = rows
        .iter()
        .take(limit as usize)
        .map(|row| Ok((row.try_get("attempt_id")?, row.try_get("change")?)))
        .collect::<Result<_, sqlx::Error>>()?;
    let mut by_change: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for change in page
        .iter()
        .map(|(_, change)| change)
        .collect::<BTreeSet<_>>()
    {
        let ledger = reassemble_ledger(&mut tx, &project, change)
            .await?
            .ok_or_else(|| LedgerError::invalid("stored attempts have no ledger"))?;
        by_change.insert(change.clone(), dataset_rows(&ledger)?.into_iter().collect());
    }
    tx.commit().await?;
    let items = page
        .iter()
        .map(|(attempt, change)| {
            by_change
                .get_mut(change)
                .and_then(|rows| rows.remove(attempt))
                .ok_or_else(|| LedgerError::invalid("stored attempt has no dataset row"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let next = page.last().filter(|_| more).map(|(attempt, _)| attempt);
    Ok(Json(json!({"project":project,"items":items,"next":next})))
}

pub(super) async fn requirement_history(
    State(state): State<LedgerState>,
    Path((project, requirement)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let mut tx = read_snapshot(&state).await?;
    // A release without an integration sequence has no place in the main lineage.
    let releases = sqlx::query("SELECT fingerprint,label,integration_sequence,model_revision,assurance_record('authority',project,model_revision) AS authority FROM assurance_releases WHERE project=$1 AND integration_sequence IS NOT NULL ORDER BY integration_sequence,submitted_at,fingerprint")
        .bind(&project).fetch_all(&mut *tx).await?;
    let mut previous: Option<(String, Intent)> = None;
    let mut ledgers: BTreeMap<String, Value> = BTreeMap::new();
    let mut lineage = Vec::new();
    let mut known = false;
    for release in releases {
        let authority: Value = release.try_get("authority")?;
        let current = requirement_state(&authority, &requirement)?;
        known |= current.is_some();
        let Some((closure, intent)) = current else {
            previous = None;
            continue;
        };
        let relation = match &previous {
            Some((previous_closure, _)) if *previous_closure == closure => {
                previous = Some((closure, intent));
                continue;
            }
            Some((_, previous_intent)) if *previous_intent == intent => "dependency",
            _ => "intent",
        };
        let fingerprint: String = release.try_get("fingerprint")?;
        let change_ids: Vec<String> = sqlx::query_scalar("SELECT change_id FROM release_changes WHERE project=$1 AND release_fingerprint=$2 ORDER BY change_id")
            .bind(&project).bind(&fingerprint).fetch_all(&mut *tx).await?;
        let mut changes = Vec::new();
        for change in change_ids {
            if !ledgers.contains_key(&change) {
                let attempts = match reassemble_ledger(&mut tx, &project, &change).await? {
                    Some(ledger) => history_attempts(&ledger)?,
                    None => Vec::new(),
                };
                ledgers.insert(change.clone(), Value::Array(attempts));
            }
            changes.push(json!({"change":change,"attempts":ledgers[&change]}));
        }
        lineage.push(json!({
            "release": fingerprint, "label": release.try_get::<String,_>("label")?,
            "integration_sequence": release.try_get::<i64,_>("integration_sequence")?,
            "model_revision": release.try_get::<String,_>("model_revision")?,
            "relation": relation, "changes": changes,
        }));
        previous = Some((closure, intent));
    }
    if !known {
        return Err(LedgerError::not_found());
    }
    let attribution_ids: Vec<String> = sqlx::query_scalar("SELECT attribution_id FROM provenance_failure_attributions WHERE project=$1 AND requirement=$2 ORDER BY recorded_at_ms,attribution_id")
        .bind(&project).bind(&requirement).fetch_all(&mut *tx).await?;
    let mut attributions = Vec::new();
    for id in attribution_ids {
        attributions.push(
            reassemble_attribution(&mut tx, &project, &id)
                .await?
                .ok_or_else(LedgerError::not_found)?,
        );
    }
    tx.commit().await?;
    Ok(Json(
        json!({"requirement":requirement,"releases":lineage,"attributions":attributions}),
    ))
}

// Ledger order (instances as exported, predecessors first, then sequence) is the dataset row order.
fn history_attempts(ledger: &Value) -> Result<Vec<Value>, LedgerError> {
    let mut attempts = Vec::new();
    for instance in instances(ledger)? {
        for attempt in self::attempts(instance)? {
            let mut attempt = attempt.clone();
            attempt["instance"] = instance["id"].clone();
            attempt["disposition"] = instance["disposition"].clone();
            attempts.push(attempt);
        }
    }
    Ok(attempts)
}

/// The authored intent of a Requirement: its `requirement_intents` digest and its Scenario fingerprints.
#[derive(PartialEq)]
struct Intent {
    digest: Option<String>,
    scenarios: Vec<(String, String)>,
}

fn requirement_state(
    authority: &Value,
    requirement: &str,
) -> Result<Option<(String, Intent)>, LedgerError> {
    let Some(entry) = authority["requirements"]
        .as_array()
        .ok_or_else(|| LedgerError::invalid("stored authority Requirements missing"))?
        .iter()
        .find(|entry| entry["id"].as_str() == Some(requirement))
    else {
        return Ok(None);
    };
    let native = core_json(authority.to_string().as_bytes())?;
    let closure = assurance_review::requirement_closure_fingerprint(&native, requirement)
        .map_err(LedgerError::invalid)?;
    let digest = authority["requirement_intents"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|intent| intent["id"].as_str() == Some(requirement))
        .and_then(|intent| intent["fingerprint"].as_str())
        .map(str::to_owned);
    let mut scenarios = Vec::new();
    for scenario in entry["scenarios"].as_array().into_iter().flatten() {
        let id = scenario
            .as_str()
            .ok_or_else(|| LedgerError::invalid("Requirement Scenario reference must be text"))?;
        let fingerprint = authority["scenarios"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|candidate| candidate["id"].as_str() == Some(id))
            .and_then(|candidate| candidate["fingerprint"].as_str())
            .ok_or_else(|| LedgerError::invalid(format!("Scenario {id} has no fingerprint")))?;
        scenarios.push((id.to_owned(), fingerprint.to_owned()));
    }
    scenarios.sort();
    Ok(Some((closure, Intent { digest, scenarios })))
}

pub(super) async fn submit_attribution(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let publisher = state.authorize(&project, &headers, Some(Role::Publisher))?;
    let attribution = compute(move || {
        let native = core_json(&body)?;
        azimuth::provenance::validate_attribution(&native).map_err(LedgerError::invalid)?;
        to_value(&native)
    })
    .await?;
    let id = text(&attribution, "id")?.to_owned();
    let fingerprint = text(&attribution, "fingerprint")?.to_owned();
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|failure-attribution|{id}"))
        .execute(&mut *tx)
        .await?;
    let existing: Option<String> = sqlx::query_scalar("SELECT fingerprint FROM provenance_failure_attributions WHERE project=$1 AND attribution_id=$2")
        .bind(&project).bind(&id).fetch_optional(&mut *tx).await?;
    let receipt = |replayed: bool| json!({"project":project,"id":id,"fingerprint":fingerprint,"replayed":replayed,"provenance_basis":PROVENANCE_BASIS});
    match existing {
        Some(stored) if stored == fingerprint => {
            tx.commit().await?;
            return Ok((StatusCode::OK, Json(receipt(true))));
        }
        Some(_) => {
            return Err(LedgerError::conflict(format!(
                "Failure Attribution {id} is immutable and already has different content"
            )))
        }
        None => (),
    }
    let adverse = &attribution["adverse"];
    let agent = Descriptor::from_value(&attribution["agent"])?;
    sqlx::query("INSERT INTO provenance_failure_attributions(project,attribution_id,requirement,adverse_kind,adverse_ref,investigator,agent_kind,agent_kind_basis,model,model_basis,effort,effort_basis,agent_type,agent_type_basis,harness_name,harness_version,harness_basis,recorded_at_ms,confidence,revises,rationale,fingerprint,submitted_by,provenance_basis) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24)")
        .bind(&project).bind(&id).bind(text(&attribution, "requirement")?)
        .bind(text(adverse, "kind")?).bind(text(adverse, "ref")?).bind(text(&attribution, "investigator")?)
        .bind(&agent.kind).bind(&agent.kind_basis).bind(&agent.model).bind(&agent.model_basis)
        .bind(&agent.effort).bind(&agent.effort_basis).bind(&agent.agent_type).bind(&agent.agent_type_basis)
        .bind(&agent.harness_name).bind(&agent.harness_version).bind(&agent.harness_basis)
        .bind(integer(&attribution, "recorded_at_ms")?).bind(text(&attribution, "confidence")?)
        .bind(optional_text(&attribution, "revises")?).bind(text(&attribution, "rationale")?)
        .bind(&fingerprint).bind(&publisher).bind(PROVENANCE_BASIS).execute(&mut *tx).await?;
    for (position, attempt) in list(&attribution, "attempts")?.iter().enumerate() {
        sqlx::query("INSERT INTO provenance_failure_attribution_attempts(project,attribution_id,position,attempt_id,role) VALUES($1,$2,$3,$4,$5)")
            .bind(&project).bind(&id).bind(index(position)?).bind(text(attempt, "attempt")?)
            .bind(text(attempt, "role")?).execute(&mut *tx).await?;
    }
    if reassemble_attribution(&mut tx, &project, &id)
        .await?
        .as_ref()
        != Some(&attribution)
    {
        return Err(LedgerError::invalid(
            "Failure Attribution contains content that relational provenance storage cannot reproduce exactly",
        ));
    }
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(receipt(false))))
}

async fn reassemble_attribution(
    conn: &mut PgConnection,
    project: &str,
    id: &str,
) -> Result<Option<Value>, LedgerError> {
    let Some(row) = sqlx::query(
        "SELECT * FROM provenance_failure_attributions WHERE project=$1 AND attribution_id=$2",
    )
    .bind(project)
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?
    else {
        return Ok(None);
    };
    let attempts = sqlx::query("SELECT attempt_id,role FROM provenance_failure_attribution_attempts WHERE project=$1 AND attribution_id=$2 ORDER BY position")
        .bind(project).bind(id).fetch_all(&mut *conn).await?
        .iter()
        .map(|attempt| Ok(json!({"attempt":attempt.try_get::<String,_>("attempt_id")?,"role":attempt.try_get::<String,_>("role")?})))
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(Some(json!({
        "format": ATTRIBUTION_FORMAT, "version": 1, "id": id,
        "requirement": row.try_get::<String,_>("requirement")?,
        "adverse": {"kind": row.try_get::<String,_>("adverse_kind")?, "ref": row.try_get::<String,_>("adverse_ref")?},
        "investigator": row.try_get::<String,_>("investigator")?,
        "agent": Descriptor::from_row(&row)?.to_value(),
        "recorded_at_ms": row.try_get::<i64,_>("recorded_at_ms")?,
        "confidence": row.try_get::<String,_>("confidence")?,
        "attempts": attempts,
        "revises": row.try_get::<Option<String>,_>("revises")?,
        "rationale": row.try_get::<String,_>("rationale")?,
        "fingerprint": row.try_get::<String,_>("fingerprint")?,
    })))
}

/// The agent descriptor as typed columns; the same column names serve attempts and attributions.
struct Descriptor {
    kind: Option<String>,
    kind_basis: String,
    model: Option<String>,
    model_basis: String,
    effort: Option<String>,
    effort_basis: String,
    agent_type: Option<String>,
    agent_type_basis: String,
    harness_name: Option<String>,
    harness_version: Option<String>,
    harness_basis: String,
}

impl Descriptor {
    fn from_value(agent: &Value) -> Result<Self, LedgerError> {
        let field = |name: &str| -> Result<(Option<String>, String), LedgerError> {
            let entry = &agent[name];
            Ok((
                optional_text(entry, "value")?.map(str::to_owned),
                text(entry, "basis")?.to_owned(),
            ))
        };
        let (kind, kind_basis) = field("kind")?;
        let (model, model_basis) = field("model")?;
        let (effort, effort_basis) = field("effort")?;
        let (agent_type, agent_type_basis) = field("agent_type")?;
        let harness = &agent["harness"];
        Ok(Self {
            kind,
            kind_basis,
            model,
            model_basis,
            effort,
            effort_basis,
            agent_type,
            agent_type_basis,
            harness_name: optional_text(harness, "name")?.map(str::to_owned),
            harness_version: optional_text(harness, "version")?.map(str::to_owned),
            harness_basis: text(harness, "basis")?.to_owned(),
        })
    }

    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            kind: row.try_get("agent_kind")?,
            kind_basis: row.try_get("agent_kind_basis")?,
            model: row.try_get("model")?,
            model_basis: row.try_get("model_basis")?,
            effort: row.try_get("effort")?,
            effort_basis: row.try_get("effort_basis")?,
            agent_type: row.try_get("agent_type")?,
            agent_type_basis: row.try_get("agent_type_basis")?,
            harness_name: row.try_get("harness_name")?,
            harness_version: row.try_get("harness_version")?,
            harness_basis: row.try_get("harness_basis")?,
        })
    }

    fn to_value(&self) -> Value {
        json!({
            "kind": {"value": self.kind, "basis": self.kind_basis},
            "model": {"value": self.model, "basis": self.model_basis},
            "effort": {"value": self.effort, "basis": self.effort_basis},
            "agent_type": {"value": self.agent_type, "basis": self.agent_type_basis},
            "harness": {"name": self.harness_name, "version": self.harness_version, "basis": self.harness_basis},
        })
    }
}

/// Token counts of one usage window as typed columns.
#[derive(Default)]
struct Tokens {
    input: Option<i64>,
    cache_read_input: Option<i64>,
    cache_write_input: Option<i64>,
    output: Option<i64>,
    reasoning_output: Option<i64>,
}

impl Tokens {
    fn from_value(tokens: &Value) -> Result<Self, LedgerError> {
        Ok(Self {
            input: Some(integer(tokens, "input_tokens")?),
            cache_read_input: optional_integer(tokens, "cache_read_input_tokens")?,
            cache_write_input: optional_integer(tokens, "cache_write_input_tokens")?,
            output: Some(integer(tokens, "output_tokens")?),
            reasoning_output: optional_integer(tokens, "reasoning_output_tokens")?,
        })
    }

    fn from_row(row: &PgRow, prefix: &str) -> Result<Self, sqlx::Error> {
        let column = |name: &str| row.try_get::<Option<i64>, _>(format!("{prefix}{name}").as_str());
        Ok(Self {
            input: column("input_tokens")?,
            cache_read_input: column("cache_read_input_tokens")?,
            cache_write_input: column("cache_write_input_tokens")?,
            output: column("output_tokens")?,
            reasoning_output: column("reasoning_output_tokens")?,
        })
    }

    fn extend(&self, value: &mut Value) {
        value["input_tokens"] = json!(self.input);
        value["cache_read_input_tokens"] = json!(self.cache_read_input);
        value["cache_write_input_tokens"] = json!(self.cache_write_input);
        value["output_tokens"] = json!(self.output);
        value["reasoning_output_tokens"] = json!(self.reasoning_output);
    }
}

/// Attempt usage as typed columns; a null `basis` is usage that was not recorded.
#[derive(Default)]
struct Usage {
    basis: Option<String>,
    model_calls: Option<i64>,
    tool_calls: Option<i64>,
    own: Tokens,
    overlapping: Option<bool>,
    delegated_agents: Option<i64>,
    delegated_model_calls: Option<i64>,
    delegated: Tokens,
}

impl Usage {
    fn from_value(usage: &Value) -> Result<Self, LedgerError> {
        if usage.is_null() {
            return Ok(Self::default());
        }
        let delegated = &usage["delegated"];
        Ok(Self {
            basis: Some(text(usage, "basis")?.to_owned()),
            model_calls: Some(integer(usage, "model_calls")?),
            tool_calls: Some(integer(usage, "tool_calls")?),
            own: Tokens::from_value(usage)?,
            overlapping: Some(
                usage["overlapping"]
                    .as_bool()
                    .ok_or_else(|| LedgerError::invalid("overlapping requires a boolean"))?,
            ),
            delegated_agents: Some(integer(delegated, "agents")?),
            delegated_model_calls: Some(integer(delegated, "model_calls")?),
            delegated: Tokens::from_value(delegated)?,
        })
    }

    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            basis: row.try_get("usage_basis")?,
            model_calls: row.try_get("model_calls")?,
            tool_calls: row.try_get("tool_calls")?,
            own: Tokens::from_row(row, "")?,
            overlapping: row.try_get("usage_overlapping")?,
            delegated_agents: row.try_get("delegated_agents")?,
            delegated_model_calls: row.try_get("delegated_model_calls")?,
            delegated: Tokens::from_row(row, "delegated_")?,
        })
    }

    fn to_value(&self) -> Value {
        let Some(basis) = &self.basis else {
            return Value::Null;
        };
        let mut delegated =
            json!({"agents": self.delegated_agents, "model_calls": self.delegated_model_calls});
        self.delegated.extend(&mut delegated);
        let mut usage = json!({
            "basis": basis, "model_calls": self.model_calls, "tool_calls": self.tool_calls,
            "overlapping": self.overlapping, "delegated": delegated,
        });
        self.own.extend(&mut usage);
        usage
    }
}

async fn read_snapshot(
    state: &LedgerState,
) -> Result<sqlx::Transaction<'static, sqlx::Postgres>, LedgerError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, LedgerError> {
    value[key]
        .as_str()
        .ok_or_else(|| LedgerError::invalid(format!("{key} requires text")))
}

fn optional_text<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>, LedgerError> {
    match &value[key] {
        Value::Null => Ok(None),
        Value::String(text) => Ok(Some(text)),
        _ => Err(LedgerError::invalid(format!("{key} requires text or null"))),
    }
}

fn integer(value: &Value, key: &str) -> Result<i64, LedgerError> {
    value[key]
        .as_i64()
        .ok_or_else(|| LedgerError::invalid(format!("{key} requires an integer")))
}

fn optional_integer(value: &Value, key: &str) -> Result<Option<i64>, LedgerError> {
    match &value[key] {
        Value::Null => Ok(None),
        _ => integer(value, key).map(Some),
    }
}

fn list<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, LedgerError> {
    value[key]
        .as_array()
        .ok_or_else(|| LedgerError::invalid(format!("{key} requires an array")))
}

fn index(position: usize) -> Result<i32, LedgerError> {
    i32::try_from(position).map_err(|_| LedgerError::invalid("list is too long"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::{
        postgres::Postgres,
        testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
    };

    const PUBLISHER: &str = "publisher-credential-for-provenance-tests-only";
    const REVIEWER: &str = "reviewer-credential-for-provenance-tests-only";
    const VIEWER: &str = "viewer-credential-for-provenance-tests-only";

    fn digest(seed: char) -> String {
        format!("sha256:{}", seed.to_string().repeat(64))
    }

    struct Service {
        _postgres: ContainerAsync<Postgres>,
        pool: PgPool,
        base: String,
        client: reqwest::Client,
        server: tokio::task::JoinHandle<()>,
    }

    impl Drop for Service {
        fn drop(&mut self) {
            self.server.abort();
        }
    }

    impl Service {
        async fn start() -> Result<Self, Box<dyn std::error::Error>> {
            let postgres = Postgres::default().with_tag("17-alpine").start().await?;
            let pool = crate::connect(&format!(
                "postgres://postgres:postgres@127.0.0.1:{}/postgres",
                postgres.get_host_port_ipv4(5432).await?
            ))
            .await?;
            crate::migrate(&pool).await?;
            let state = LedgerState::new(
                pool.clone(),
                BTreeMap::from([(
                    "project".into(),
                    ProjectCredentials {
                        publishers: BTreeMap::from([("publisher".into(), PUBLISHER.into())]),
                        producers: BTreeMap::from([(
                            "producer".into(),
                            "producer-credential-for-provenance-tests-only".into(),
                        )]),
                        reviewers: BTreeMap::from([("reviewer".into(), REVIEWER.into())]),
                        owners: BTreeMap::from([(
                            "owner".into(),
                            "owner-credential-for-provenance-tests-only".into(),
                        )]),
                        viewers: BTreeMap::from([("viewer".into(), VIEWER.into())]),
                    },
                )]),
            )?;
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
            let base = format!(
                "http://{}/v1/projects/project/provenance",
                listener.local_addr()?
            );
            let server =
                tokio::spawn(async move { axum::serve(listener, app(state)).await.unwrap() });
            Ok(Self {
                _postgres: postgres,
                pool,
                base,
                client: reqwest::Client::new(),
                server,
            })
        }

        async fn post(
            &self,
            path: &str,
            credential: &str,
            body: &Value,
        ) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
            let response = self
                .client
                .post(format!("{}/{path}", self.base))
                .bearer_auth(credential)
                .json(body)
                .send()
                .await?;
            Ok((response.status(), response.json().await?))
        }

        async fn get(
            &self,
            path: &str,
            credential: &str,
        ) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
            let response = self
                .client
                .get(format!("{}/{path}", self.base))
                .bearer_auth(credential)
                .send()
                .await?;
            let status = response.status();
            let body = response.text().await?;
            Ok((
                status,
                serde_json::from_str(&body).unwrap_or(Value::String(body)),
            ))
        }
    }

    fn unknown_agent() -> Value {
        json!({"kind":{"value":null,"basis":"unknown"},"model":{"value":null,"basis":"unknown"},
            "effort":{"value":null,"basis":"unknown"},"agent_type":{"value":null,"basis":"unknown"},
            "harness":{"name":null,"version":null,"basis":"unknown"}})
    }

    fn harness_agent() -> Value {
        json!({"kind":{"value":"model","basis":"harness"},"model":{"value":"claude-opus-5-5","basis":"harness"},
            "effort":{"value":"high","basis":"harness"},"agent_type":{"value":"general-purpose","basis":"harness"},
            "harness":{"name":"claude-code","version":"2.1.309","basis":"harness"}})
    }

    fn attempt(change: &str, instance: &str, sequence: usize, action: &str) -> Value {
        json!({
            "id": format!("attempt-{change}-{instance}-{sequence}"),
            "sequence": sequence,
            "action": action,
            "task": action,
            "scope": "change",
            "target": [{"name":"change","value":change},{"name":"project","value":"project"}],
            "actor": format!("{action}-session"),
            "outcome": "completed",
            "report": format!("Completed {action}."),
            "handoff_captured_at_ms": 1761200000000i64 + sequence as i64,
            "recorded_at_ms": 1761200400000i64 + sequence as i64,
            "agent": if sequence % 2 == 0 { unknown_agent() } else { harness_agent() },
            "guidance": [{"kind":"skill","id":"azimuth-apply","fingerprint":digest('9')}],
            "trace": null,
            "usage": null,
            "used": [{"kind":"initial-request","repository":null,"ref":"inputs/request.md","fingerprint":digest('1')}],
            "generated": [{"kind":"proposal","repository":"application","ref":format!("azimuth/changes/{change}/proposal.md"),"fingerprint":digest('2'),"record":null,"record_fingerprint":null}],
        })
    }

    fn instance(
        change: &str,
        id: &str,
        predecessor: Option<&str>,
        disposition: &str,
        actions: &[&str],
    ) -> Value {
        json!({
            "id": id,
            "predecessor": predecessor.map(|id| json!({
                "id": id, "ref": format!("predecessors/{id}.json"), "revision": 7, "fingerprint": digest('8'),
            })),
            "disposition": disposition,
            "definition_fingerprint": digest('3'),
            "template": {"id":"change-development","revision":10},
            "attempts": actions.iter().enumerate().map(|(i, action)| attempt(change, id, i + 1, action)).collect::<Vec<_>>(),
        })
    }

    fn ledger(change: &str, instances: Vec<Value>) -> Value {
        json!({"format":LEDGER_FORMAT,"version":1,"project":"project","change":change,"instances":instances})
    }

    fn attribution(id: &str, requirement: &str, rationale: &str, recorded_at_ms: i64) -> Value {
        let mut value = json!({
            "format": ATTRIBUTION_FORMAT, "version": 1, "id": id, "requirement": requirement,
            "adverse": {"kind":"requirement-status","ref":digest('4')},
            "investigator": "dima",
            "agent": {"kind":{"value":"human","basis":"declared"},"model":{"value":null,"basis":"unknown"},
                "effort":{"value":null,"basis":"unknown"},"agent_type":{"value":"investigator","basis":"declared"},
                "harness":{"name":null,"version":null,"basis":"unknown"}},
            "recorded_at_ms": recorded_at_ms, "confidence": "medium",
            "attempts": [{"attempt":"attempt-c1-i1-1","role":"caused"},{"attempt":"attempt-c1-i1-2","role":"allowed-through"}],
            "revises": null, "rationale": rationale,
        });
        let native = core_json(value.to_string().as_bytes()).unwrap();
        value["fingerprint"] = json!(azimuth::fingerprint::canonical_sha256(&native));
        value
    }

    #[tokio::test]
    async fn ledgers_append_replay_and_reject_changed_history(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service = Service::start().await?;
        let first = ledger(
            "c1",
            vec![instance("c1", "i1", None, "active", &["author-proposal"])],
        );
        let (status, receipt) = service.post("ledgers", REVIEWER, &first).await?;
        assert_eq!(status, StatusCode::FORBIDDEN, "{receipt}");
        let (status, receipt) = service.post("ledgers", PUBLISHER, &first).await?;
        assert_eq!(status, StatusCode::CREATED, "{receipt}");
        assert_eq!(receipt["attempts_added"], 1);
        assert_eq!(receipt["provenance_basis"], PROVENANCE_BASIS);
        let (status, receipt) = service.post("ledgers", PUBLISHER, &first).await?;
        assert_eq!(status, StatusCode::OK, "{receipt}");
        assert_eq!(receipt["replayed"], true);
        assert_eq!(
            service.get("changes/c1", VIEWER).await?,
            (StatusCode::OK, first.clone())
        );

        let second = ledger(
            "c1",
            vec![
                instance(
                    "c1",
                    "i1",
                    None,
                    "superseded",
                    &["author-proposal", "review-proposal"],
                ),
                instance("c1", "i2", Some("i1"), "completed", &["implement"]),
            ],
        );
        let (status, receipt) = service.post("ledgers", PUBLISHER, &second).await?;
        assert_eq!(status, StatusCode::CREATED, "{receipt}");
        assert_eq!(
            (
                receipt["instances_added"].clone(),
                receipt["attempts_added"].clone()
            ),
            (json!(1), json!(2))
        );
        assert_eq!(
            service.get("changes/c1", VIEWER).await?,
            (StatusCode::OK, second.clone())
        );
        let (status, _) = service.post("ledgers", PUBLISHER, &first).await?;
        assert_eq!(
            status,
            StatusCode::OK,
            "an earlier accepted export replays without rewriting"
        );
        assert_eq!(service.get("changes/c1", VIEWER).await?.1, second);

        let mut changed = second.clone();
        changed["instances"][0]["attempts"][0]["report"] = json!("Rewritten report.");
        let (status, error) = service.post("ledgers", PUBLISHER, &changed).await?;
        assert_eq!(status, StatusCode::CONFLICT, "{error}");
        assert!(error["detail"]
            .as_str()
            .unwrap()
            .contains("differs from its stored content"));
        let mut omitted = second.clone();
        omitted["instances"][0]["attempts"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert_eq!(
            service.post("ledgers", PUBLISHER, &omitted).await?.0,
            StatusCode::CONFLICT
        );
        let mut reopened = second.clone();
        reopened["instances"][1]["disposition"] = json!("active");
        let (status, error) = service.post("ledgers", PUBLISHER, &reopened).await?;
        assert_eq!(status, StatusCode::CONFLICT, "{error}");
        let mut borrowed = ledger("c2", vec![instance("c2", "j1", None, "active", &[])]);
        let mut reused = second["instances"][1]["attempts"][0].clone();
        reused["target"][0]["value"] = json!("c2");
        borrowed["instances"][0]["attempts"] = json!([reused]);
        assert_eq!(
            service.post("ledgers", PUBLISHER, &borrowed).await?.0,
            StatusCode::CONFLICT
        );
        let mut foreign = first.clone();
        foreign["project"] = json!("other");
        assert_eq!(
            service.post("ledgers", PUBLISHER, &foreign).await?.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            service.get("changes/c1", VIEWER).await?.1,
            second,
            "rejected exports leave the ledger unchanged"
        );
        assert_eq!(
            service.get("changes/unknown", VIEWER).await?.0,
            StatusCode::NOT_FOUND
        );
        let unauthenticated = service
            .client
            .get(format!("{}/changes/c1", service.base))
            .send()
            .await?;
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
        let stored: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM provenance_ledgers WHERE project='project' AND change='c1'",
        )
        .fetch_one(&service.pool)
        .await?;
        assert_eq!(stored, 2);
        Ok(())
    }

    #[tokio::test]
    async fn failure_attributions_are_immutable_by_id() -> Result<(), Box<dyn std::error::Error>> {
        let service = Service::start().await?;
        let original = attribution(
            "gap-1",
            "bounded",
            "The review accepted an untested path.",
            10,
        );
        let (status, receipt) = service.post("attributions", VIEWER, &original).await?;
        assert_eq!(status, StatusCode::FORBIDDEN, "{receipt}");
        let (status, receipt) = service.post("attributions", PUBLISHER, &original).await?;
        assert_eq!(status, StatusCode::CREATED, "{receipt}");
        assert_eq!(receipt["fingerprint"], original["fingerprint"]);
        let (status, receipt) = service.post("attributions", PUBLISHER, &original).await?;
        assert_eq!(
            (status, receipt["replayed"].clone()),
            (StatusCode::OK, json!(true))
        );
        let rewritten = attribution("gap-1", "bounded", "A different account.", 10);
        let (status, error) = service.post("attributions", PUBLISHER, &rewritten).await?;
        assert_eq!(status, StatusCode::CONFLICT, "{error}");
        let stored: String = sqlx::query_scalar("SELECT rationale FROM provenance_failure_attributions WHERE project='project' AND attribution_id='gap-1'")
            .fetch_one(&service.pool)
            .await?;
        assert_eq!(stored, "The review accepted an untested path.");
        let mut connection = service.pool.acquire().await?;
        assert_eq!(
            reassemble_attribution(&mut connection, "project", "gap-1").await?,
            Some(original)
        );
        Ok(())
    }

    async fn release(
        pool: &PgPool,
        fingerprint: &str,
        sequence: Option<i64>,
        authority: &Value,
        changes: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let revision = authority["fingerprint"].as_str().unwrap();
        let mut tx = pool.begin().await?;
        let known: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM model_revisions WHERE project='project' AND fingerprint=$1)").bind(revision).fetch_one(&mut *tx).await?;
        if !known {
            relational::store(&mut tx, "authority", "project", revision, authority).await?;
            sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES('project',$1,'owner')").bind(revision).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES('project',$1,'owner')").bind(revision).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO assurance_releases(project,fingerprint,model_revision,label,integration_commit,integration_sequence,submitted_by) VALUES('project',$1,$2,$3,$4,$5,'publisher')")
            .bind(fingerprint).bind(revision).bind(format!("Release {fingerprint}"))
            .bind(sequence.map(|s| format!("{s:0>40}"))).bind(sequence).execute(&mut *tx).await?;
        for change in changes {
            sqlx::query("INSERT INTO release_changes(project,release_fingerprint,change_id,change_revision) VALUES('project',$1,$2,$3)")
                .bind(fingerprint).bind(change).bind(digest('5')).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    fn authority(scenario: &str, intent: char) -> Value {
        let model = azimuth::model::Model {
            specs: vec![azimuth::spec::parse_spec(
                "spec.md",
                &format!("# Spec: module\n\n## Requirement: bounded\n\nThe operation SHALL complete.\n\n### Scenario: completes\n{scenario}\n"),
            ).unwrap()],
            ..azimuth::model::Model::default()
        };
        let mut value = to_value(&assurance_review::authority(&model, "project").unwrap()).unwrap();
        value["requirement_intents"] = json!([{"id":"bounded","fingerprint":digest(intent)}]);
        value["fingerprint"] = json!(digest_of(&value));
        value
    }

    fn digest_of(value: &Value) -> String {
        let mut value = value.clone();
        value.as_object_mut().unwrap().remove("fingerprint");
        azimuth::fingerprint::canonical_sha256(&core_json(value.to_string().as_bytes()).unwrap())
    }

    // A Check bound to the Requirement's Scenario changes its closure without changing its intent.
    fn with_check(authority: &Value, check_fingerprint: char) -> Value {
        let mut value = authority.clone();
        value["checks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"probe","fingerprint":digest(check_fingerprint)}));
        value["bindings"]
            .as_array_mut()
            .unwrap()
            .push(json!({"check":"probe","scenario":"completes","fingerprint":digest('6')}));
        value["requirements"][0]["bindings"]
            .as_array_mut()
            .unwrap()
            .push(json!({"check":"probe","scenario":"completes"}));
        value["fingerprint"] = json!(digest_of(&value));
        value
    }

    #[tokio::test]
    async fn requirement_history_labels_intent_and_dependency_changes_by_release_lineage(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service = Service::start().await?;
        let pool = &service.pool;
        let base = authority("Given ordinary input, when invoked, then complete.", 'a');
        let dependency = with_check(&base, 'b');
        let intent = with_check(
            &authority("Given bounded input, when invoked, then complete.", 'a'),
            'b',
        );
        let renamed_intent = {
            let mut value = intent.clone();
            value["requirement_intents"] = json!([{"id":"bounded","fingerprint":digest('c')}]);
            value["requirements"][0]["bindings"]
                .as_array_mut()
                .unwrap()
                .clear();
            value["fingerprint"] = json!(digest_of(&value));
            value
        };
        // Arrival order differs from the lineage order to show that integration_sequence governs.
        release(pool, "r4", Some(4), &intent, &["c3"]).await?;
        release(pool, "r1", Some(1), &base, &["c1"]).await?;
        release(pool, "r3", Some(3), &dependency, &["c1", "c2"]).await?;
        release(pool, "r2", Some(2), &base, &["unrelated"]).await?;
        release(pool, "r5", Some(5), &renamed_intent, &[]).await?;
        release(
            pool,
            "unintegrated",
            None,
            &authority("Given no lineage.", 'd'),
            &["c9"],
        )
        .await?;
        let mut c1 = ledger(
            "c1",
            vec![
                instance("c1", "i1", None, "superseded", &["author-proposal"]),
                instance(
                    "c1",
                    "i2",
                    Some("i1"),
                    "completed",
                    &["implement", "review-requirement-verification"],
                ),
            ],
        );
        c1["instances"][1]["attempts"][1]["generated"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"verification-review","repository":"application","ref":"azimuth/changes/c1/reviews/bundle.json","fingerprint":digest('7'),"record":"bounded","record_fingerprint":digest('6')}));
        c1["instances"][1]["attempts"][1]["usage"] = harness_usage();
        assert_eq!(
            service.post("ledgers", PUBLISHER, &c1).await?.0,
            StatusCode::CREATED
        );
        let gap = attribution(
            "gap-1",
            "bounded",
            "The review accepted an untested path.",
            10,
        );
        assert_eq!(
            service.post("attributions", PUBLISHER, &gap).await?.0,
            StatusCode::CREATED
        );
        let other = attribution("gap-2", "other", "Another Requirement.", 5);
        assert_eq!(
            service.post("attributions", PUBLISHER, &other).await?.0,
            StatusCode::CREATED
        );

        let (status, history) = service.get("requirements/bounded", VIEWER).await?;
        assert_eq!(status, StatusCode::OK, "{history}");
        let releases = history["releases"].as_array().unwrap();
        let summary: Vec<(&str, &str)> = releases
            .iter()
            .map(|r| {
                (
                    r["release"].as_str().unwrap(),
                    r["relation"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                ("r1", "intent"),
                ("r3", "dependency"),
                ("r4", "intent"),
                ("r5", "intent")
            ]
        );
        assert_eq!(releases[0]["integration_sequence"], 1);
        assert_eq!(releases[0]["label"], "Release r1");
        assert_eq!(releases[0]["model_revision"], base["fingerprint"]);
        let c1_rows = history_attempts(&c1)?;
        let order: Vec<(&str, i64)> = c1_rows
            .iter()
            .map(|a| {
                (
                    a["instance"].as_str().unwrap(),
                    a["sequence"].as_i64().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            order,
            [("i1", 1), ("i2", 1), ("i2", 2)],
            "predecessors first, then sequence"
        );
        assert_eq!(c1_rows[2]["disposition"], "completed");
        assert_eq!(
            c1_rows[2]["generated"][1]["record_fingerprint"],
            digest('6')
        );
        assert_eq!(c1_rows[2]["usage"], harness_usage());
        assert_eq!(
            c1_rows[1]["agent"]["agent_type"]["value"],
            "general-purpose"
        );
        assert_eq!(
            c1_rows[2]["agent"]["kind"]["basis"],
            c1["instances"][1]["attempts"][1]["agent"]["kind"]["basis"]
        );
        assert_eq!(
            releases[0]["changes"],
            json!([{"change":"c1","attempts":c1_rows}])
        );
        assert_eq!(
            releases[1]["changes"],
            json!([{"change":"c1","attempts":c1_rows},{"change":"c2","attempts":[]}]),
            "a release change without a stored ledger is listed with no attempts"
        );
        assert_eq!(releases[3]["changes"], json!([]));
        assert_eq!(history["attributions"], json!([gap]));
        assert_eq!(
            service.get("requirements/absent", VIEWER).await?.0,
            StatusCode::NOT_FOUND,
            "a Requirement that no release authority contains has no history"
        );
        assert_eq!(
            service.get("requirements/other", VIEWER).await?.0,
            StatusCode::NOT_FOUND,
            "an attribution alone does not place a Requirement in the lineage"
        );
        Ok(())
    }

    #[tokio::test]
    async fn attempts_page_dataset_rows_in_ledger_order() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = Service::start().await?;
        let mut second = ledger(
            "c2",
            vec![instance("c2", "k1", None, "active", &["author-proposal"])],
        );
        // A historical instance can predate templates.
        second["instances"][0]["template"] = Value::Null;
        let first = ledger(
            "c1",
            vec![
                instance(
                    "c1",
                    "i1",
                    None,
                    "superseded",
                    &["author-proposal", "review-proposal"],
                ),
                instance("c1", "i2", Some("i1"), "active", &["implement", "verify"]),
            ],
        );
        for ledger in [&second, &first] {
            assert_eq!(
                service.post("ledgers", PUBLISHER, ledger).await?.0,
                StatusCode::CREATED
            );
        }
        let expected: Vec<Value> = dataset_rows(&first)?
            .into_iter()
            .chain(dataset_rows(&second)?)
            .map(|(_, row)| row)
            .collect();
        let mut collected = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let path = match &cursor {
                Some(cursor) => format!("attempts?limit=2&cursor={cursor}"),
                None => "attempts?limit=2".into(),
            };
            let (status, page) = service.get(&path, VIEWER).await?;
            assert_eq!(status, StatusCode::OK, "{page}");
            assert!(page["items"].as_array().unwrap().len() <= 2);
            collected.extend(page["items"].as_array().unwrap().iter().cloned());
            match page["next"].as_str() {
                Some(next) => cursor = Some(next.into()),
                None => break,
            }
        }
        assert_eq!(collected, expected);
        let (status, all) = service.get("attempts", VIEWER).await?;
        assert_eq!(
            (
                status,
                all["items"].as_array().unwrap().len(),
                all["next"].clone()
            ),
            (StatusCode::OK, 5, Value::Null)
        );
        assert_eq!(
            service.get("attempts?cursor=unknown", VIEWER).await?.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            service.get("attempts?limit=0", VIEWER).await?.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            service.get("attempts?after=x", VIEWER).await?.0,
            StatusCode::BAD_REQUEST
        );
        Ok(())
    }

    fn harness_usage() -> Value {
        json!({
            "basis": "harness", "model_calls": 4, "tool_calls": 11,
            "input_tokens": 120_000, "cache_read_input_tokens": 98_000, "cache_write_input_tokens": null,
            "output_tokens": 2_400, "reasoning_output_tokens": null, "overlapping": true,
            "delegated": {"agents": 2, "model_calls": 7, "input_tokens": 40_000,
                "cache_read_input_tokens": null, "cache_write_input_tokens": 1_200,
                "output_tokens": 900, "reasoning_output_tokens": 300},
        })
    }

    fn usage_ledger() -> Value {
        let mut value = ledger(
            "c1",
            vec![instance(
                "c1",
                "i1",
                None,
                "active",
                &["implement", "verify"],
            )],
        );
        value["instances"][0]["attempts"][0]["usage"] = harness_usage();
        value
    }

    #[tokio::test]
    async fn usage_and_agent_type_round_trip_exactly() -> Result<(), Box<dyn std::error::Error>> {
        let service = Service::start().await?;
        let submitted = usage_ledger();
        let (status, receipt) = service.post("ledgers", PUBLISHER, &submitted).await?;
        assert_eq!(status, StatusCode::CREATED, "{receipt}");
        let response = service
            .client
            .get(format!("{}/changes/c1", service.base))
            .bearer_auth(VIEWER)
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await?, serde_json::to_string(&submitted)?);
        let stored: (Option<String>, String, Option<String>, Option<i64>, Option<bool>) = sqlx::query_as("SELECT agent_type,agent_type_basis,usage_basis,delegated_cache_write_input_tokens,usage_overlapping FROM provenance_attempts WHERE attempt_id='attempt-c1-i1-1'")
            .fetch_one(&service.pool)
            .await?;
        assert_eq!(
            stored,
            (
                Some("general-purpose".into()),
                "harness".into(),
                Some("harness".into()),
                Some(1_200),
                Some(true)
            ),
            "usage is stored in typed columns"
        );
        let unrecorded: (String, Option<String>, Option<i64>) = sqlx::query_as("SELECT agent_type_basis,usage_basis,input_tokens FROM provenance_attempts WHERE attempt_id='attempt-c1-i1-2'")
            .fetch_one(&service.pool)
            .await?;
        assert_eq!(unrecorded, ("unknown".into(), None, None));

        let mut changed = submitted.clone();
        changed["instances"][0]["attempts"][0]["usage"]["output_tokens"] = json!(2_401);
        let (status, error) = service.post("ledgers", PUBLISHER, &changed).await?;
        assert_eq!(status, StatusCode::CONFLICT, "{error}");
        assert!(error["detail"]
            .as_str()
            .unwrap()
            .contains("attempt attempt-c1-i1-1 differs from its stored content"));
        let mut forgotten = submitted.clone();
        forgotten["instances"][0]["attempts"][0]["usage"] = Value::Null;
        assert_eq!(
            service.post("ledgers", PUBLISHER, &forgotten).await?.0,
            StatusCode::CONFLICT,
            "recorded usage cannot be withdrawn"
        );
        let mut claimed = submitted.clone();
        claimed["instances"][0]["attempts"][1]["usage"] = harness_usage();
        assert_eq!(
            service.post("ledgers", PUBLISHER, &claimed).await?.0,
            StatusCode::CONFLICT,
            "unrecorded usage cannot be added later"
        );
        assert_eq!(service.get("changes/c1", VIEWER).await?.1, submitted);

        let (status, page) = service.get("attempts", VIEWER).await?;
        assert_eq!(status, StatusCode::OK, "{page}");
        let native = core_json(submitted.to_string().as_bytes())?;
        let core: Vec<Value> = azimuth::provenance::dataset_rows(&native)?
            .iter()
            .map(to_value)
            .collect::<Result<_, _>>()?;
        assert_eq!(page["items"], json!(core));
        let rows = page["items"].as_array().unwrap();
        assert_eq!(
            (
                rows[0]["agent_type"].clone(),
                rows[0]["input_tokens"].clone(),
                rows[0]["delegated_agents"].clone(),
                rows[0]["usage_overlapping"].clone(),
            ),
            (
                json!("general-purpose"),
                json!(120_000),
                json!(2),
                json!(true)
            )
        );
        for column in [
            "usage_basis",
            "model_calls",
            "input_tokens",
            "usage_overlapping",
            "delegated_agents",
            "delegated_output_tokens",
        ] {
            assert_eq!(rows[1][column], Value::Null, "{column}");
        }
        assert_eq!(rows[1]["agent_type_basis"], "unknown");
        Ok(())
    }

    // Attempts ingested before agent types and usage existed have no values in those columns.
    #[tokio::test]
    async fn attempts_stored_without_usage_read_back_as_unknown(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service = Service::start().await?;
        let mut tx = service.pool.begin().await?;
        sqlx::query("INSERT INTO provenance_ledgers(project,change,ledger_fingerprint,submitted_by,provenance_basis) VALUES('project','c1',$1,'publisher',$2)")
            .bind(digest('0')).bind(PROVENANCE_BASIS).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO provenance_instances(project,change,instance_id,position,disposition,definition_fingerprint) VALUES('project','c1','i1',0,'active',$1)")
            .bind(digest('3')).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO provenance_attempts(project,attempt_id,change,instance_id,sequence,action,task,scope,actor,outcome,agent_kind,agent_kind_basis,model,model_basis,effort,effort_basis,harness_name,harness_version,harness_basis) VALUES('project','attempt-1','c1','i1',1,'implement','implement','change','implement-session','completed','model','harness','claude-opus-5-5','harness',NULL,'unknown','claude-code','2.1.309','harness')")
            .execute(&mut *tx).await?;
        tx.commit().await?;
        let partial = sqlx::query(
            "UPDATE provenance_attempts SET usage_basis='harness' WHERE attempt_id='attempt-1'",
        )
        .execute(&service.pool)
        .await
        .expect_err("usage columns are recorded together or not at all");
        assert_eq!(
            partial
                .as_database_error()
                .and_then(|e| e.code())
                .as_deref(),
            Some("23514")
        );
        let mut agent = harness_agent();
        agent["effort"] = json!({"value":null,"basis":"unknown"});
        agent["agent_type"] = json!({"value":null,"basis":"unknown"});
        let exported = json!({"format":LEDGER_FORMAT,"version":1,"project":"project","change":"c1","instances":[{
            "id":"i1","predecessor":null,"disposition":"active","definition_fingerprint":digest('3'),"template":null,
            "attempts":[{"id":"attempt-1","sequence":1,"action":"implement","task":"implement","scope":"change",
                "target":[],"actor":"implement-session","outcome":"completed","report":null,
                "handoff_captured_at_ms":null,"recorded_at_ms":null,"agent":agent,"guidance":[],
                "trace":null,"usage":null,"used":[],"generated":[]}],
        }]});
        let (status, stored) = service.get("changes/c1", VIEWER).await?;
        assert_eq!(status, StatusCode::OK, "{stored}");
        assert_eq!(stored, exported);
        assert_eq!(stored["instances"][0]["attempts"][0]["usage"], Value::Null);
        let (status, receipt) = service.post("ledgers", PUBLISHER, &exported).await?;
        assert_eq!(status, StatusCode::CREATED, "{receipt}");
        assert_eq!(
            receipt["attempts_added"], 0,
            "a re-export carrying the new fields matches the stored attempt"
        );
        Ok(())
    }

    const PROVENANCE_TABLES: [&str; 8] = [
        "provenance_ledgers",
        "provenance_instances",
        "provenance_attempts",
        "provenance_attempt_targets",
        "provenance_attempt_guidance",
        "provenance_attempt_entities",
        "provenance_failure_attributions",
        "provenance_failure_attribution_attempts",
    ];

    #[test]
    fn assessment_code_never_reads_provenance() {
        let migration = include_str!("../../relational_migrations/0009_work_provenance.sql");
        for table in PROVENANCE_TABLES {
            assert!(
                migration.contains(&format!("CREATE TABLE {table} (")),
                "{table} is not a Work Provenance table"
            );
        }
        let assessment = [
            (
                "assessment_triggers.sql",
                include_str!("../relational/assessment_triggers.sql"),
            ),
            ("run_ledger.rs", include_str!("../run_ledger.rs")),
            ("materialized.rs", include_str!("materialized.rs")),
            (
                "materialized_reviews.rs",
                include_str!("materialized_reviews.rs"),
            ),
            ("deployment_status.rs", include_str!("deployment_status.rs")),
            ("evidence_cache.rs", include_str!("evidence_cache.rs")),
            ("release.rs", include_str!("release.rs")),
            ("environment.rs", include_str!("environment.rs")),
        ];
        for (file, source) in assessment {
            for table in PROVENANCE_TABLES {
                assert!(!source.contains(table), "{file} references {table}");
            }
        }
        let own = include_str!("provenance.rs");
        let own = &own[..own.find("#[cfg(test)]").unwrap()];
        for assessment_state in [
            "assessment_events",
            "definition_events",
            "assessment_requirements",
            "assessment_dependencies",
            "deployment_assessments",
            "definition_assessments",
            "definition_requirements",
            "INSERT INTO assurance",
            "relational::store",
        ] {
            assert!(
                !own.contains(assessment_state),
                "provenance writes assessment state {assessment_state}"
            );
        }
    }

    #[tokio::test]
    async fn provenance_tables_raise_no_assessment_events() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = Service::start().await?;
        let triggers: Vec<String> = sqlx::query_scalar("SELECT c.relname::text FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid WHERE NOT t.tgisinternal AND c.relname LIKE 'provenance%'")
            .fetch_all(&service.pool).await?;
        assert_eq!(triggers, Vec::<String>::new());
        for table in PROVENANCE_TABLES {
            let functions: Vec<String> = sqlx::query_scalar(
                "SELECT proname::text FROM pg_proc WHERE prosrc LIKE '%' || $1 || '%'",
            )
            .bind(table)
            .fetch_all(&service.pool)
            .await?;
            assert_eq!(functions, Vec::<String>::new(), "{table}");
        }
        let json_columns: Vec<String> = sqlx::query_scalar("SELECT table_name::text||'.'||column_name::text FROM information_schema.columns WHERE table_schema='public' AND table_name LIKE 'provenance%' AND udt_name NOT IN ('text','int4','int8','bool','timestamptz')")
            .fetch_all(&service.pool).await?;
        assert_eq!(
            json_columns,
            Vec::<String>::new(),
            "provenance columns are typed scalars"
        );
        let first = ledger(
            "c1",
            vec![instance("c1", "i1", None, "active", &["author-proposal"])],
        );
        let events = "SELECT (SELECT count(*) FROM assessment_events)+(SELECT count(*) FROM definition_events)";
        let before: i64 = sqlx::query_scalar(events).fetch_one(&service.pool).await?;
        assert_eq!(
            service.post("ledgers", PUBLISHER, &first).await?.0,
            StatusCode::CREATED
        );
        let gap = attribution("gap-1", "bounded", "Rationale.", 10);
        assert_eq!(
            service.post("attributions", PUBLISHER, &gap).await?.0,
            StatusCode::CREATED
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(events)
                .fetch_one(&service.pool)
                .await?,
            before
        );
        Ok(())
    }
}
