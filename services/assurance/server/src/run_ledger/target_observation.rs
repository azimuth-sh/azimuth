use super::*;
use sqlx::Acquire;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Submission {
    observation: Value,
}

fn fingerprint(value: &Value) -> Result<String, LedgerError> {
    run::canonical_fingerprint(&core_json(value.to_string().as_bytes())?)
        .map_err(LedgerError::invalid)
}
fn shape(value: &Value, keys: &[&str], path: &str) -> Result<(), LedgerError> {
    if value.as_object().is_none_or(|fields| {
        fields.len() != keys.len() || keys.iter().any(|key| !fields.contains_key(*key))
    }) {
        return Err(LedgerError::invalid(format!(
            "{path}: expected exactly {}",
            keys.join(", ")
        )));
    }
    Ok(())
}
fn identifier(value: &Value, path: &str) -> Result<(), LedgerError> {
    let limit = if path == "target.instance" { 512 } else { 256 };
    if value
        .as_str()
        .is_none_or(|v| v.trim().is_empty() || v.len() > limit || v.chars().any(char::is_control))
    {
        return Err(LedgerError::invalid(format!(
            "{path}: expected bounded nonempty identity"
        )));
    }
    Ok(())
}
fn interval(value: &Value, time: i64) -> Result<(), LedgerError> {
    let start = value["started_at_ms"].as_i64();
    let end = value["finished_at_ms"].as_i64();
    if !matches!((start, end), (Some(a), Some(b)) if a >= 0 && a <= b && b <= time) {
        return Err(LedgerError::invalid(
            "ordered nonnegative integer measurement times required; future measurements rejected",
        ));
    }
    Ok(())
}

fn validate(raw: &Value, time: i64) -> Result<(), LedgerError> {
    shape(
        raw,
        &[
            "format",
            "version",
            "kind",
            "source",
            "execution_id",
            "environment",
            "started_at_ms",
            "finished_at_ms",
            "targets",
            "check",
            "capture",
        ],
        "observation",
    )?;
    if raw["format"] != "azimuth-target-observation"
        || raw["version"] != 1
        || raw.to_string().len() > 262144
    {
        return Err(LedgerError::invalid(
            "expected bounded azimuth-target-observation version 1",
        ));
    }
    for field in ["source", "execution_id", "environment"] {
        identifier(&raw[field], field)?;
    }
    azimuth::diag::validate_id(raw["environment"].as_str().unwrap(), false)
        .map_err(LedgerError::invalid)?;
    interval(raw, time)?;
    let targets = raw["targets"]
        .as_array()
        .filter(|v| v.len() <= 256)
        .ok_or_else(|| LedgerError::invalid("targets: expected at most 256 measured identities"))?;
    let mut unique = BTreeSet::new();
    for target in targets {
        shape(target, &["component", "image", "instance"], "target")?;
        identifier(&target["component"], "target.component")?;
        identifier(&target["instance"], "target.instance")?;
        let image = target["image"].as_str().unwrap_or("");
        if image.len() > 512
            || image.chars().any(char::is_whitespace)
            || image
                .rsplit_once("@sha256:")
                .is_none_or(|(repository, digest)| {
                    repository.is_empty()
                        || digest.len() != 64
                        || !digest
                            .bytes()
                            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                })
        {
            return Err(LedgerError::invalid(
                "target.image: immutable repository@sha256 digest required",
            ));
        }
        if !unique.insert((
            target["component"].as_str(),
            image,
            target["instance"].as_str(),
        )) {
            return Err(LedgerError::invalid("duplicate measured target identity"));
        }
    }
    match raw["kind"].as_str() {
        Some("check") if raw["capture"].is_null() => {
            shape(
                &raw["check"],
                &["definition", "outcome", "details"],
                "check",
            )?;
            if !["violated", "inconclusive"]
                .contains(&raw["check"]["outcome"].as_str().unwrap_or(""))
                || raw["check"]["details"]
                    .as_str()
                    .is_none_or(|s| s.len() > 65536)
            {
                return Err(LedgerError::invalid(
                    "only violated/inconclusive target outcomes and bounded details supported",
                ));
            }
            if targets.is_empty() && raw["check"]["outcome"] != "inconclusive" {
                return Err(LedgerError::invalid(
                    "target-free collection facts can only be inconclusive",
                ));
            }
            let definition = &raw["check"]["definition"];
            shape(
                definition,
                &["plan", "adapters", "routes"],
                "check.definition",
            )?;
            if definition["plan"].get("fingerprint").is_some() {
                return Err(LedgerError::invalid(
                    "subject-free plan must omit fingerprint",
                ));
            }
            let subject = run::Subject::Deployment {
                environment: "validation".into(),
                deployment: "validation".into(),
                deployment_fingerprint: format!("sha256:{}", "0".repeat(64)),
                artifacts: vec![run::ArtifactState {
                    id: "validation".into(),
                    digest: format!("sha256:{}", "0".repeat(64)),
                }],
            };
            materialize_run(raw, &subject, "validation", "validation")?;
        }
        Some("context") if raw["check"].is_null() => {
            let c = &raw["capture"];
            shape(
                c,
                &[
                    "component",
                    "instance",
                    "started_at_ms",
                    "finished_at_ms",
                    "before",
                    "after",
                ],
                "capture",
            )?;
            interval(c, time)?;
            if c["started_at_ms"] != raw["started_at_ms"]
                || c["finished_at_ms"] != raw["finished_at_ms"]
                || targets.len() != 1
                || targets[0]["component"] != c["component"]
                || targets[0]["instance"] != c["instance"]
                || c["before"]["instance"] != c["instance"]
                || c["after"]["instance"] != c["instance"]
                || c["before"]["report"] != c["after"]["report"]
            {
                return Err(LedgerError::invalid("context requires stable before/after reports and exactly one matching runtime target"));
            }
        }
        _ => {
            return Err(LedgerError::invalid(
                "kind requires exactly its check or capture record",
            ))
        }
    }
    Ok(())
}

pub(super) async fn record(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    state.authorize(&project, &headers, None)?;
    let body: Submission = serde_json::from_value(to_value(&core_json(&body)?)?)
        .map_err(|e| LedgerError::invalid(e.to_string()))?;
    let principal = if body.observation["kind"] == "check" {
        state
            .authorize(&project, &headers, Some(Role::Producer))
            .or_else(|_| state.authorize(&project, &headers, Some(Role::Publisher)))?
    } else {
        state.authorize(&project, &headers, Some(Role::Publisher))?
    };
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|target-observations"))
        .execute(&mut *tx)
        .await?;
    let time: i64 =
        sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
            .fetch_one(&mut *tx)
            .await?;
    validate(&body.observation, time)?;
    let fp = fingerprint(&body.observation)?;
    let existing: Option<(String,String)> = sqlx::query_as("SELECT fingerprint,submitted_by FROM target_observations WHERE project=$1 AND source=$2 AND execution_id=$3")
        .bind(&project).bind(body.observation["source"].as_str()).bind(body.observation["execution_id"].as_str()).fetch_optional(&mut *tx).await?;
    let replayed = existing.is_some();
    if let Some((existing, by)) = existing {
        if existing != fp || by != principal {
            return Err(LedgerError::conflict(
                "source execution already has different content or attribution",
            ));
        }
    } else {
        if body.observation["kind"] == "context" {
            validate_context(
                &mut tx,
                &project,
                &body.observation,
                &format!("sha256:{}", "0".repeat(64)),
            )
            .await?;
        }
        relational::store(
            &mut *tx,
            "target_observation",
            &project,
            &fp,
            &body.observation,
        )
        .await?;
        sqlx::query("INSERT INTO target_observations(project,fingerprint,source,execution_id,environment,submitted_by,received_at_ms) VALUES($1,$2,$3,$4,$5,$6,$7)")
            .bind(&project).bind(&fp).bind(body.observation["source"].as_str()).bind(body.observation["execution_id"].as_str()).bind(body.observation["environment"].as_str()).bind(&principal).bind(time).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    resolve_pending(&state, &project).await?;
    Ok((
        if replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(json!({"fingerprint":fp,"replayed":replayed})),
    ))
}

pub(super) async fn list(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let after: i64 = if page.after.is_empty() {
        0
    } else {
        page.after
            .parse()
            .map_err(|_| LedgerError::invalid("expected nonnegative sequence cursor"))?
    };
    if after < 0 {
        return Err(LedgerError::invalid("expected nonnegative sequence cursor"));
    }
    let rows: Vec<(i64,Value)> = sqlx::query_as("SELECT t.sequence,jsonb_build_object('fingerprint',t.fingerprint,'observation',assurance_record('target_observation',t.project,t.fingerprint),'submitted_by',t.submitted_by,'received_at_ms',t.received_at_ms,'resolution',CASE WHEN r.deployment_id IS NULL THEN NULL ELSE jsonb_build_object('deployment',r.deployment_id,'subject_fingerprint',r.subject_fingerprint,'before_fingerprint',r.before_fingerprint,'after_fingerprint',r.after_fingerprint,'run_fingerprint',r.run_fingerprint,'capture_fingerprint',r.capture_fingerprint,'resolved_at_ms',r.resolved_at_ms,'normalizer_fingerprint',r.normalizer_fingerprint) END,'diagnostics',CASE WHEN r.deployment_id IS NULL THEN jsonb_build_array(t.resolution_diagnostic) ELSE '[]'::jsonb END) FROM target_observations t LEFT JOIN target_observation_resolutions r ON r.project=t.project AND r.observation_fingerprint=t.fingerprint WHERE t.project=$1 AND t.sequence>$2 ORDER BY t.sequence LIMIT $3")
        .bind(&project).bind(after).bind(limit+1).fetch_all(&state.pool).await?;
    if rows.iter().map(|(_, v)| v.to_string().len()).sum::<usize>() > 33_554_432 {
        return Err(LedgerError::invalid(
            "target observation page exceeds 32 MiB; reduce limit",
        ));
    }
    let next = rows
        .get(limit as usize - 1)
        .filter(|_| rows.len() > limit as usize)
        .map(|(seq, _)| seq.to_string());
    Ok(Json(
        json!({"records":rows.into_iter().take(limit as usize).map(|(_,v)|v).collect::<Vec<_>>(),"next":next}),
    ))
}

#[derive(Debug, Clone)]
struct Reconciliation {
    fingerprint: String,
    deployment: Option<String>,
    observation: Value,
}

fn inventory(observation: &Value) -> Option<BTreeMap<String, (String, BTreeSet<String>)>> {
    let mut result = BTreeMap::new();
    for component in observation["components"].as_array()? {
        if component["ready"] != true {
            return None;
        }
        let instances = component["instances"]
            .as_array()?
            .iter()
            .map(|v| v.as_str().map(str::to_owned))
            .collect::<Option<BTreeSet<_>>>()?;
        if instances.is_empty() || instances.len() != component["instances"].as_array()?.len() {
            return None;
        }
        if result
            .insert(
                component["name"].as_str()?.into(),
                (component["image"].as_str()?.into(), instances),
            )
            .is_some()
        {
            return None;
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn bracket(raw: &Value, history: &[Reconciliation]) -> Option<(Reconciliation, Reconciliation)> {
    let start = raw["started_at_ms"].as_i64()?;
    let end = raw["finished_at_ms"].as_i64()?;
    if raw["targets"].as_array()?.is_empty() {
        return None;
    }
    let before = history.iter().rev().find(|entry| {
        entry.observation["observed_at_ms"]
            .as_i64()
            .is_some_and(|t| t <= start)
    })?;
    let after = history.iter().find(|entry| {
        entry.fingerprint != before.fingerprint
            && entry.observation["observed_at_ms"]
                .as_i64()
                .is_some_and(|t| t >= end)
    })?;
    let deployment = before.deployment.as_ref()?;
    if after.deployment.as_ref()? != deployment
        || before.observation["observed_at_ms"].as_i64()?
            + before.observation["valid_for_seconds"].as_i64()? * 1000
            < start
        || after.observation["observed_at_ms"].as_i64()? - end
            > after.observation["valid_for_seconds"].as_i64()? * 1000
    {
        return None;
    }
    let expected_inventory = inventory(&before.observation)?;
    let begin = before.observation["observed_at_ms"].as_i64()?;
    let finish = after.observation["observed_at_ms"].as_i64()?;
    let mut covered_until = begin;
    for entry in history.iter().filter(|entry| {
        entry.observation["observed_at_ms"]
            .as_i64()
            .is_some_and(|t| begin <= t && t <= finish)
    }) {
        let at = entry.observation["observed_at_ms"].as_i64()?;
        if at > covered_until {
            return None;
        }
        covered_until = at + entry.observation["valid_for_seconds"].as_i64()? * 1000;
        if entry.deployment.as_ref() != Some(deployment)
            || entry.observation["state"] != "complete"
            || inventory(&entry.observation).as_ref() != Some(&expected_inventory)
        {
            return None;
        }
    }
    if !raw["targets"].as_array()?.iter().all(|target| {
        expected_inventory
            .get(target["component"].as_str().unwrap_or(""))
            .is_some_and(|(image, instances)| {
                Some(image.as_str()) == target["image"].as_str()
                    && target["instance"]
                        .as_str()
                        .is_some_and(|instance| instances.contains(instance))
            })
    }) {
        return None;
    }
    Some((before.clone(), after.clone()))
}

pub(super) async fn resolve_pending(state: &LedgerState, project: &str) -> Result<(), LedgerError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|target-observations"))
        .execute(&mut *tx)
        .await?;
    let pending: Vec<(String,Value,String)> = sqlx::query_as("SELECT t.fingerprint,assurance_record('target_observation',t.project,t.fingerprint),t.submitted_by FROM target_observations t WHERE t.project=$1 AND NOT EXISTS(SELECT 1 FROM target_observation_resolutions r WHERE r.project=t.project AND r.observation_fingerprint=t.fingerprint) ORDER BY t.last_resolution_attempt_ms,t.sequence LIMIT 500")
        .bind(project).fetch_all(&mut *tx).await?;
    for (fp, raw, principal) in pending {
        sqlx::query("UPDATE target_observations SET last_resolution_attempt_ms=(extract(epoch FROM clock_timestamp())*1000)::bigint WHERE project=$1 AND fingerprint=$2").bind(project).bind(&fp).execute(&mut *tx).await?;
        let mut item = tx.begin().await?;
        match resolve_one(&mut item, project, &fp, &raw, &principal).await {
            Ok(_) => item.commit().await?,
            Err(error) => {
                item.rollback().await?;
                if error.status.is_server_error() {
                    return Err(error);
                }
                sqlx::query("UPDATE target_observations SET resolution_diagnostic=$3 WHERE project=$1 AND fingerprint=$2").bind(project).bind(&fp).bind(error.detail).execute(&mut *tx).await?;
            }
        }
    }
    tx.commit().await?;
    Ok(())
}

async fn resolve_one(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    fp: &str,
    raw: &Value,
    principal: &str,
) -> Result<bool, LedgerError> {
    let history: Vec<(String,Option<String>,Value)> = sqlx::query_as("WITH bounds AS (SELECT (SELECT observed_at_ms FROM deployment_observations WHERE project=$1 AND environment=$2 AND observed_at_ms<=$3 ORDER BY revision DESC LIMIT 1) AS first,(SELECT observed_at_ms FROM deployment_observations WHERE project=$1 AND environment=$2 AND observed_at_ms>=$4 AND observed_at_ms>$3 ORDER BY revision LIMIT 1) AS last) SELECT fingerprint,deployment_id,assurance_record('deployment_observation',project,fingerprint) FROM deployment_observations,bounds WHERE project=$1 AND environment=$2 AND observed_at_ms BETWEEN bounds.first AND bounds.last ORDER BY revision LIMIT 10001")
            .bind(project).bind(raw["environment"].as_str()).bind(raw["started_at_ms"].as_i64()).bind(raw["finished_at_ms"].as_i64()).fetch_all(&mut **tx).await?;
    if history.len() > 10000 {
        return Ok(false);
    }
    let history = history
        .into_iter()
        .map(|(fingerprint, deployment, observation)| Reconciliation {
            fingerprint,
            deployment,
            observation,
        })
        .collect::<Vec<_>>();
    let Some((before, after)) = bracket(&raw, &history) else {
        return Ok(false);
    };
    let deployment = before.deployment.as_ref().unwrap();
    let (record,release): (Value,String) = sqlx::query_as("SELECT assurance_record('deployment',project,deployment_id),release_fingerprint FROM release_deployments WHERE project=$1 AND deployment_id=$2").bind(project).bind(deployment).fetch_one(&mut **tx).await?;
    let subject_fp = record["subject_fingerprint"].as_str().ok_or_else(|| {
        LedgerError::invalid("historical deployment Subject fingerprint is missing")
    })?;
    let subject = run::subject_from_json(&core_json(record["subject"].to_string().as_bytes())?)
        .map_err(LedgerError::invalid)?;
    if run::subject_fingerprint(&subject) != subject_fp {
        return Err(LedgerError::invalid(
            "historical deployment Subject fingerprint differs",
        ));
    }
    let normalizer = target_normalizer::adapter()?;
    relational::store(
        &mut **tx,
        "target_normalizer",
        project,
        &normalizer.adapter_fingerprint,
        &target_normalizer::record(normalizer),
    )
    .await?;
    let mut run_fp: Option<String> = None;
    let mut capture_fp: Option<String> = None;
    if raw["kind"] == "check" {
        let bundle = materialize_run(&raw, &subject, &before.fingerprint, &after.fingerprint)?;
        let authority: Value = sqlx::query_scalar("SELECT assurance_record_field('release',project,fingerprint,'snapshot')->'authority' FROM assurance_releases WHERE project=$1 AND fingerprint=$2").bind(project).bind(&release).fetch_one(&mut **tx).await?;
        let selected = &bundle.plan.checks[0];
        let expected_cases: BTreeSet<&str> = authority["bindings"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|binding| {
                binding["check"] == selected.id
                    && binding["check_fingerprint"] == selected.fingerprint
            })
            .filter_map(|binding| binding["case"].as_str())
            .collect();
        if expected_cases != selected.cases.iter().map(String::as_str).collect() {
            return Ok(false);
        }
        let policy = authority["checks"]
            .as_array()
            .and_then(|checks| {
                checks.iter().find(|check| {
                    check["id"] == selected.id && check["fingerprint"] == selected.fingerprint
                })
            })
            .map(|check| &check["deployment"]);
        if !policy.is_some_and(|policy| {
            policy["subject"] == "deployment"
                && policy["contribution"] != "support"
                && raw["targets"].as_array().unwrap().iter().all(|target| {
                    policy["components"]
                        .as_array()
                        .is_some_and(|components| components.contains(&target["component"]))
                })
        }) {
            return Ok(false);
        }
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("{project}|{}", bundle.run_id))
            .execute(&mut **tx)
            .await?;
        let existing: Option<String> = sqlx::query_scalar(
            "SELECT fingerprint FROM run_bundles WHERE project=$1 AND run_id=$2 AND revision=0",
        )
        .bind(project)
        .bind(&bundle.run_id)
        .fetch_optional(&mut **tx)
        .await?;
        if existing
            .as_ref()
            .is_some_and(|existing| existing != &bundle.bundle_fingerprint)
        {
            return Err(LedgerError::conflict(
                "native Run identity already has different content",
            ));
        }
        let payload = to_value(&run::to_json(&bundle))?;
        relational::store(
            &mut **tx,
            "run",
            project,
            &bundle.bundle_fingerprint,
            &payload,
        )
        .await?;
        sqlx::query("INSERT INTO run_bundles(project,run_id,revision,fingerprint,subject_fingerprint,model_fingerprint) VALUES($1,$2,0,$3,$4,$5) ON CONFLICT(project,run_id,revision) DO NOTHING")
                .bind(project).bind(&bundle.run_id).bind(&bundle.bundle_fingerprint).bind(subject_fp).bind(&bundle.plan.model_fingerprint).execute(&mut **tx).await?;
        deployment_status::attribute_run(
            tx,
            project,
            &bundle.bundle_fingerprint,
            &bundle.run_id,
            principal,
        )
        .await?;
        run_fp = Some(bundle.bundle_fingerprint);
    } else {
        let capture = validate_context(tx, project, &raw, subject_fp).await?;
        let mut identity = capture.clone();
        identity.as_object_mut().unwrap().remove("trusted");
        let key = fingerprint(&identity)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("{project}|context-capture|{key}"))
            .execute(&mut **tx)
            .await?;
        let time: i64 =
            sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
                .fetch_one(&mut **tx)
                .await?;
        let existing: Option<Value> = sqlx::query_scalar("SELECT assurance_record('context_capture',project,record_key) FROM assurance_records WHERE kind='context_capture' AND project=$1 AND record_key=$2").bind(project).bind(&key).fetch_optional(&mut **tx).await?;
        if let Some(existing) = existing {
            if existing["capture"] != capture {
                return Err(LedgerError::conflict(
                    "materialized capture identity collision",
                ));
            }
        } else {
            relational::store(&mut **tx,"context_capture",project,&key,&json!({"fingerprint":key,"capture":capture,"submitted_by":principal,"received_at_ms":time})).await?;
        }
        capture_fp = Some(key);
    }
    sqlx::query("INSERT INTO target_observation_resolutions(project,observation_fingerprint,deployment_id,subject_fingerprint,before_fingerprint,after_fingerprint,run_fingerprint,capture_fingerprint,normalizer_fingerprint,resolved_at_ms) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,(extract(epoch FROM clock_timestamp())*1000)::bigint)")
            .bind(project).bind(fp).bind(deployment).bind(subject_fp).bind(&before.fingerprint).bind(&after.fingerprint).bind(run_fp).bind(capture_fp).bind(&normalizer.adapter_fingerprint).execute(&mut **tx).await?;
    Ok(true)
}

async fn validate_context(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    raw: &Value,
    subject: &str,
) -> Result<Value, LedgerError> {
    let mut capture = raw["capture"].clone();
    capture["kind"] = json!("deployment");
    for field in ["run", "activity", "bundle_fingerprint"] {
        capture[field] = Value::Null;
    }
    capture["subject_fingerprint"] = json!(subject);
    capture["trusted"] = json!(true);
    let contracts: Vec<Value> = sqlx::query_scalar("SELECT assurance_record('context_contract',project,record_key)->'contract' FROM assurance_records WHERE kind='context_contract' AND project=$1 ORDER BY record_key LIMIT 5001").bind(project).fetch_all(&mut **tx).await?;
    if contracts.len() > 5000 {
        return Err(LedgerError::invalid(
            "context contract inventory exceeds limit",
        ));
    }
    let contracts = contracts
        .iter()
        .map(|v| core_json(v.to_string().as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;
    azimuth::deployment_status::validate_capture(
        &core_json(capture.to_string().as_bytes())?,
        &contracts,
    )
    .map_err(LedgerError::invalid)?;
    Ok(capture)
}

fn materialize_run(
    raw: &Value,
    subject: &run::Subject,
    before: &str,
    after: &str,
) -> Result<run::RunBundle, LedgerError> {
    let definition = &raw["check"]["definition"];
    let mut plan_value = definition["plan"].clone();
    plan_value["fingerprint"] = json!(format!("sha256:{}", "0".repeat(64)));
    let mut plan = run::plan_from_json(&core_json(plan_value.to_string().as_bytes())?)
        .map_err(LedgerError::invalid)?;
    let subject_fp = run::subject_fingerprint(subject);
    plan.fingerprint = run::plan_fingerprint(&subject_fp, &plan);
    if plan.checks.len() != 1
        || plan.checks[0].units.len() != 1
        || !plan.challenges.is_empty()
        || definition["adapters"]
            .as_array()
            .is_none_or(|v| v.len() != 1)
    {
        return Err(LedgerError::invalid(
            "target Check requires one Check, one unit and one import adapter",
        ));
    }
    let mut adapter_value = definition["adapters"][0].clone();
    shape(
        &adapter_value,
        &[
            "id",
            "adapter_version",
            "adapter_fingerprint",
            "descriptor_fingerprint",
            "configuration_fingerprint",
        ],
        "definition.adapter",
    )?;
    adapter_value["launch_fingerprint"] = json!(format!("sha256:{}", "0".repeat(64)));
    adapter_value["routes"] = definition["routes"].clone();
    let raw_bytes = run::canonical_json(&core_json(raw.to_string().as_bytes())?)
        .map_err(LedgerError::invalid)?;
    adapter_value["import_inputs"] = json!([{"id":"target-observation","digest":fingerprint(raw)?,"size_bytes":raw_bytes.len()}]);
    let mut adapter =
        run::adapter_provenance_from_json(&core_json(adapter_value.to_string().as_bytes())?)
            .map_err(LedgerError::invalid)?;
    if adapter
        .routes
        .iter()
        .any(|route| route.capability.class != run::RouteCapabilityClass::CheckImport)
        || !run::validate_launch_routes_against_plan(&plan, &adapter.routes).is_empty()
    {
        return Err(LedgerError::invalid(
            "source definition requires exact Check import routes",
        ));
    }
    let original_adapter = adapter.adapter_fingerprint.clone();
    let normalizer = target_normalizer::adapter()?;
    adapter.id = normalizer.id.clone();
    adapter.adapter_version = normalizer.adapter_version.clone();
    adapter.adapter_fingerprint = normalizer.adapter_fingerprint.clone();
    adapter.descriptor_fingerprint = normalizer.descriptor_fingerprint.clone();
    adapter.configuration_fingerprint = normalizer.configuration_fingerprint.clone();
    for route in &mut adapter.routes {
        route.capability.address = normalizer.capabilities[0].address(&normalizer.id);
        route.capability.fingerprint = normalizer.capabilities[0].fingerprint.clone();
    }
    let identity = run::LaunchAdapterIdentity {
        id: adapter.id.clone(),
        adapter_version: adapter.adapter_version.clone(),
        adapter_fingerprint: adapter.adapter_fingerprint.clone(),
        descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
        configuration_fingerprint: adapter.configuration_fingerprint.clone(),
    };
    let start = raw["started_at_ms"]
        .as_u64()
        .ok_or_else(|| LedgerError::invalid("measurement start required"))?;
    let finish = raw["finished_at_ms"]
        .as_u64()
        .ok_or_else(|| LedgerError::invalid("measurement end required"))?;
    adapter.launch_fingerprint = run::launch_fingerprint(
        run::ProvenanceMode::Import,
        start,
        subject,
        &subject_fp,
        &plan,
        &identity,
        &adapter.routes,
    );
    let check = &plan.checks[0];
    let outcome = if raw["check"]["outcome"] == "violated" {
        run::ObservationOutcome::Violated
    } else {
        run::ObservationOutcome::Inconclusive
    };
    let execution = run::CheckExecution {
        check: run::CheckRef {
            id: check.id.clone(),
            fingerprint: check.fingerprint.clone(),
        },
        units: vec![run::CheckExecutionUnit {
            id: check.units[0].id.clone(),
            attempts: vec![run::CheckAttempt {
                ordinal: 1,
                activity: "measurement".into(),
                outcomes: check.cases.iter().map(|c| (c.clone(), outcome)).collect(),
            }],
        }],
        observations: check
            .cases
            .iter()
            .map(|c| run::Observation {
                case: c.clone(),
                outcome,
                observed_at_ms: finish,
                fingerprint: String::new(),
                artifacts: vec![],
                diagnostics: vec![],
            })
            .collect(),
    };
    let mut bundle = run::RunBundle {
        run_id: String::new(),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: String::new(),
        subject: subject.clone(),
        subject_fingerprint: subject_fp,
        planned_at_ms: start,
        started_at_ms: start,
        finished_at_ms: finish,
        status: run::RunStatus::Complete,
        actual_selection: run::ActualSelection {
            context: plan.required_context.clone(),
            plan_fingerprint: plan.fingerprint.clone(),
            checks: plan.checks.clone(),
            challenges: vec![],
            fingerprint: String::new(),
        },
        plan,
        provenance: run::Provenance {
            mode: run::ProvenanceMode::Import,
            source: run::SourceProvenance {
                system: raw["source"].as_str().unwrap().into(),
                execution: raw["execution_id"].as_str().unwrap().into(),
                uri: None,
            },
            normalizer: run::Normalizer {
                id: format!("adapter/{}", adapter.id),
                version: adapter.adapter_version.clone(),
                build_fingerprint: adapter.adapter_fingerprint.clone(),
            },
            adapter,
            generated_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| LedgerError::invalid("normalization clock unavailable"))?
                .as_millis()
                .try_into()
                .map_err(|_| LedgerError::invalid("normalization clock exceeds native range"))?,
            principal: None,
            attributes: Some(BTreeMap::from([
                ("target-observation".into(), fingerprint(raw)?),
                ("source-adapter-fingerprint".into(), original_adapter),
                ("before-reconciliation".into(), before.into()),
                ("after-reconciliation".into(), after.into()),
                (
                    "normalization".into(),
                    "ASS historical target identity resolution".into(),
                ),
            ])),
        },
        activities: vec![run::Activity {
            id: "measurement".into(),
            status: run::ActivityStatus::Completed,
            started_at_ms: start,
            finished_at_ms: finish,
            artifacts: vec![],
            diagnostics: vec![],
            attributes: BTreeMap::from([
                (
                    "details".into(),
                    raw["check"]["details"].as_str().unwrap_or("").into(),
                ),
                ("target-observation".into(), fingerprint(raw)?),
            ]),
        }],
        check_executions: vec![execution],
        artifacts: vec![],
        diagnostics: vec![],
        challenger_executions: vec![],
        contributions: vec![],
    };
    run::refresh_bundle(&mut bundle);
    let findings = run::verify(&bundle);
    if !findings.is_empty() {
        return Err(LedgerError::invalid(
            findings
                .into_iter()
                .map(|f| format!("{}: {}", f.code, f.detail))
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    parse_bundle(&run::to_json(&bundle).to_string_pretty())?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn raw() -> Value {
        json!({"started_at_ms":2000,"finished_at_ms":3000,"targets":[{"component":"api","image":"registry/api@sha256:aaa","instance":"process"}]})
    }
    fn observed(time: i64, deployment: &str) -> Reconciliation {
        Reconciliation {
            fingerprint: time.to_string(),
            deployment: Some(deployment.into()),
            observation: json!({"observed_at_ms":time,"valid_for_seconds":15,"state":"complete","components":[{"name":"api","image":"registry/api@sha256:aaa","ready":true,"instances":["process"]}]}),
        }
    }
    #[test]
    fn pending_measurement_needs_an_independent_after_boundary() {
        assert!(bracket(&raw(), &[observed(1000, "old")]).is_none());
        assert!(bracket(&raw(), &[observed(1000, "old"), observed(4000, "old")]).is_some());
        let mut instant = raw();
        instant["started_at_ms"] = json!(1000);
        instant["finished_at_ms"] = json!(1000);
        assert!(bracket(&instant, &[observed(1000, "old")]).is_none());
        assert!(bracket(&instant, &[observed(1000, "old"), observed(4000, "old")]).is_some());
    }
    #[test]
    fn reordered_replica_inventory_keeps_counterexample_subset_matching() {
        let mut before = observed(1000, "old");
        let mut after = observed(4000, "old");
        before.observation["components"][0]["instances"] = json!(["process", "replica"]);
        after.observation["components"][0]["instances"] = json!(["replica", "process"]);
        assert!(bracket(&raw(), &[before, after]).is_some());
    }
    #[test]
    fn rollback_reused_images_do_not_transfer_observations() {
        assert!(bracket(&raw(), &[observed(1000, "old"), observed(4000, "rollback")]).is_none());
        let history = [
            observed(1000, "old"),
            observed(4000, "old"),
            observed(5000, "rollback"),
        ];
        assert_eq!(
            bracket(&raw(), &history).unwrap().0.deployment.as_deref(),
            Some("old")
        );
    }
    #[test]
    fn incomplete_or_conflicting_intervening_observation_blocks_resolution() {
        for status in ["partial", "unknown", "drifted", "progressing", "failed"] {
            let mut bad = observed(2500, "old");
            bad.observation["state"] = json!(status);
            assert!(
                bracket(&raw(), &[observed(1000, "old"), bad, observed(4000, "old")]).is_none()
            );
        }
        let mut changed = observed(2500, "old");
        changed.observation["components"][0]["instances"] = json!(["replacement"]);
        assert!(bracket(
            &raw(),
            &[observed(1000, "old"), changed, observed(4000, "old")]
        )
        .is_none());
    }
    #[test]
    fn missing_target_or_inventory_and_expired_coverage_fail_closed() {
        let mut unknown = raw();
        unknown["targets"][0]["instance"] = json!("unknown");
        assert!(bracket(&unknown, &[observed(1000, "old"), observed(4000, "old")]).is_none());
        let mut missing = observed(1000, "old");
        missing.observation["components"][0]
            .as_object_mut()
            .unwrap()
            .remove("instances");
        assert!(bracket(&raw(), &[missing, observed(4000, "old")]).is_none());
        let mut long = raw();
        long["finished_at_ms"] = json!(100000);
        assert!(bracket(&long, &[observed(1000, "old"), observed(101000, "old")]).is_none());
        let mut empty = raw();
        empty["targets"] = json!([]);
        assert!(bracket(&empty, &[observed(1000, "old"), observed(4000, "old")]).is_none());
    }
}
