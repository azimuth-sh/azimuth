use super::*;

pub(super) async fn publish(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let submitter = state.authorize(&project, &headers, Some(Role::Publisher))?;
    // Native verification must not monopolize the async worker serving liveness requests.
    let (payload, mut bundles) = tokio::task::spawn_blocking(move || {
        let native = core_json(&body)?;
        azimuth::release::verify(&native).map_err(LedgerError::invalid)?;
        let payload = to_value(&native)?;
        let bundles = array(&payload["snapshot"], "bundles")?
            .iter()
            .map(|value| parse_bundle(&value.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok::<_, LedgerError>((payload, bundles))
    })
    .await
    .map_err(validation_worker_error)??;
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
        "SELECT assurance_record('release',project,fingerprint) FROM assurance_releases WHERE project=$1 AND fingerprint=$2",
    )
    .bind(&project)
    .bind(fingerprint)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(existing) = existing {
        if existing != payload {
            return Err(LedgerError::conflict("release publication is immutable"));
        }
        if let Some(catalog) = payload.get("catalog") {
            model_catalog::store(
                &mut tx,
                &project,
                revision,
                catalog,
                &submitter,
                "publisher-publication",
            )
            .await?;
        }
        tx.commit().await?;
        return Ok((
            StatusCode::OK,
            Json(json!({"fingerprint":fingerprint,"replayed":true})),
        ));
    }
    crate::relational::store(
        &mut *tx,
        "authority",
        &project,
        revision,
        &snapshot["authority"],
    )
    .await?;
    crate::relational::store(&mut *tx, "release", &project, fingerprint, &payload).await?;
    sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(&project).bind(revision).bind(&submitter).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(&project).bind(revision).bind(&submitter).execute(&mut *tx).await?;
    let integration = integration_attestation(release)?;
    sqlx::query("INSERT INTO assurance_releases(project,fingerprint,model_revision,label,integration_commit,integration_sequence,submitted_by) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(&project).bind(fingerprint).bind(revision).bind(text(release, "label")?).bind(integration.as_ref().map(|i| i.0)).bind(integration.as_ref().map(|i| i.1)).bind(&submitter).execute(&mut *tx).await?;
    if let Some(catalog) = payload.get("catalog") {
        model_catalog::store(
            &mut tx,
            &project,
            revision,
            catalog,
            &submitter,
            "publisher-publication",
        )
        .await?;
    }
    let no_changes = Vec::new();
    let changes = match release.get("changes") {
        None | Some(Value::Null) => &no_changes,
        Some(_) => array(release, "changes")?,
    };
    for change in changes {
        let key = format!("{fingerprint}|{}", text(change, "id")?);
        crate::relational::store(&mut *tx, "change", &project, &key, change).await?;
        sqlx::query("INSERT INTO release_changes(project,release_fingerprint,change_id,change_revision) VALUES($1,$2,$3,$4)")
            .bind(&project).bind(fingerprint).bind(text(change,"id")?).bind(text(change,"revision")?).execute(&mut *tx).await?;
    }
    bundles.sort_by(|a, b| (&a.run_id, a.bundle_revision).cmp(&(&b.run_id, b.bundle_revision)));
    for bundle in bundles {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("{project}|{}", bundle.run_id))
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query(
            "SELECT assurance_record('run',project,fingerprint) AS payload FROM run_bundles WHERE project=$1 AND run_id=$2 ORDER BY revision",
        )
        .bind(&project)
        .bind(&bundle.run_id)
        .fetch_all(&mut *tx)
        .await?;
        let stored = rows
            .iter()
            .map(|row| row.try_get::<Value, _>("payload"))
            .collect::<Result<Vec<_>, _>>()?;
        let (bundle, insertion) = tokio::task::spawn_blocking(move || {
            let mut history = stored
                .iter()
                .map(|value| parse_bundle(&value.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(previous) = history
                .iter()
                .find(|item| item.bundle_revision == bundle.bundle_revision)
            {
                if previous.bundle_fingerprint != bundle.bundle_fingerprint {
                    return Err(LedgerError::conflict(
                        "Run revision has different immutable content",
                    ));
                }
                return Ok((bundle, None));
            }
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
            let payload = to_value(&run::to_json(&bundle))?;
            Ok((bundle, Some(payload)))
        })
        .await
        .map_err(validation_worker_error)??;
        if let Some(payload) = insertion {
            crate::relational::store(
                &mut *tx,
                "run",
                &project,
                &bundle.bundle_fingerprint,
                &payload,
            )
            .await?;
            sqlx::query("INSERT INTO run_bundles(project,run_id,revision,fingerprint,verification_target_fingerprint,model_fingerprint) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(&project).bind(&bundle.run_id).bind(bundle.bundle_revision as i64).bind(&bundle.bundle_fingerprint)
                .bind(&bundle.verification_target_fingerprint).bind(&bundle.plan.model_fingerprint).execute(&mut *tx).await?;
        }
        deployment_status::attribute_run(
            &mut tx,
            &project,
            &bundle.bundle_fingerprint,
            &bundle.run_id,
            &submitter,
        )
        .await?;
        sqlx::query("INSERT INTO release_run_submissions(project,release_fingerprint,run_fingerprint,submitted_by) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
            .bind(&project).bind(fingerprint).bind(&bundle.bundle_fingerprint).bind(&submitter).execute(&mut *tx).await?;
    }
    for record in array(snapshot, "reviews")? {
        let native = core_json(record.to_string().as_bytes())?;
        let target = assurance_review::review_target(&native).map_err(LedgerError::invalid)?;
        crate::relational::store(
            &mut *tx,
            "review",
            &project,
            text(record, "fingerprint")?,
            record,
        )
        .await?;
        sqlx::query("INSERT INTO assurance_reviews(project,fingerprint,target_fingerprint,submitted_by,provenance_basis) VALUES($1,$2,$3,$4,'publisher-transported-assertion') ON CONFLICT DO NOTHING")
            .bind(&project).bind(text(record,"fingerprint")?).bind(run::canonical_fingerprint(&target).map_err(LedgerError::invalid)?).bind(&submitter).execute(&mut *tx).await?;
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

fn validation_worker_error(error: tokio::task::JoinError) -> LedgerError {
    tracing::error!(error = %error, "Release validation worker failed");
    LedgerError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        detail: "Release validation did not complete".into(),
    }
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

fn integration_attestation(release: &Value) -> Result<Option<(&str, i64)>, LedgerError> {
    let Some(integration) = release.get("integration").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let fields = integration
        .as_object()
        .filter(|fields| {
            fields
                .keys()
                .all(|key| key == "commit" || key == "main_sequence")
        })
        .ok_or_else(|| LedgerError::invalid("integration allows only commit and main_sequence"))?;
    let commit = fields
        .get("commit")
        .and_then(Value::as_str)
        .filter(|commit| is_commit(commit))
        .ok_or_else(|| {
            LedgerError::invalid("integration commit requires 40 or 64 lowercase hex digits")
        })?;
    let sequence = fields
        .get("main_sequence")
        .and_then(Value::as_i64)
        .filter(|sequence| *sequence >= 1)
        .ok_or_else(|| {
            LedgerError::invalid("integration main_sequence requires a positive integer")
        })?;
    Ok(Some((commit, sequence)))
}

fn is_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

const RELEASE_INTEGRATION: &str = "CASE WHEN r.integration_commit IS NULL THEN NULL ELSE jsonb_build_object('commit',r.integration_commit,'main_sequence',r.integration_sequence,'ratification',assurance_record_field('main_publication',r.project,r.integration_commit,'ratification')) END";

pub(super) async fn list(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let limit = page.limit()?;
    let rows = sqlx::query(&format!("SELECT fingerprint,jsonb_build_object('fingerprint',fingerprint,'release',jsonb_build_object('fingerprint',fingerprint,'label',label,'model_revision',model_revision,'integration',{RELEASE_INTEGRATION}),'submitted_by',submitted_by,'submitted_at',submitted_at) AS payload FROM assurance_releases r WHERE project=$1 AND fingerprint>$2 ORDER BY fingerprint LIMIT $3"))
        .bind(&project).bind(&page.after).bind(limit+1).fetch_all(&state.pool).await?;
    paged(rows, limit)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SummaryOptions {
    #[serde(default)]
    include: String,
}
pub(super) async fn summary(
    State(state): State<LedgerState>,
    Path((project, fingerprint)): Path<(String, String)>,
    Query(options): Query<SummaryOptions>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let record: Option<Value> = sqlx::query_scalar(&format!("SELECT jsonb_build_object('publication',jsonb_build_object('release',jsonb_build_object('fingerprint',fingerprint,'label',label,'model_revision',model_revision,'integration',{RELEASE_INTEGRATION})),'submitted_by',submitted_by,'submitted_at',submitted_at) FROM assurance_releases r WHERE project=$1 AND fingerprint=$2"))
        .bind(&project).bind(&fingerprint).fetch_optional(&state.pool).await?;
    let mut record = record.ok_or_else(LedgerError::not_found)?;
    match options.include.as_str() {
        "" => {}
        "checks" => {
            record["requirements"] =
                Value::Array(requirement_checks(&state, &project, &fingerprint).await?);
        }
        _ => return Err(LedgerError::invalid("include must be empty or `checks`")),
    }
    Ok(Json(record))
}

async fn requirement_checks(
    state: &LedgerState,
    project: &str,
    fingerprint: &str,
) -> Result<Vec<Value>, LedgerError> {
    let release: Value = sqlx::query_scalar("SELECT assurance_record('release',project,fingerprint) FROM assurance_releases WHERE project=$1 AND fingerprint=$2")
        .bind(project).bind(fingerprint).fetch_one(&state.pool).await?;
    let model_revision = text(&release["release"], "model_revision")?.to_owned();
    let authority_field = |field: &'static str| {
        let pool = state.pool.clone();
        let model_revision = model_revision.clone();
        let project = project.to_owned();
        async move {
            let value: Option<Value> =
                sqlx::query_scalar("SELECT assurance_record_field('authority',$1,$2,$3)")
                    .bind(project)
                    .bind(model_revision)
                    .bind(field)
                    .fetch_one(&pool)
                    .await?;
            Ok::<_, LedgerError>(
                value
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default(),
            )
        }
    };
    let requirements = authority_field("requirements").await?;
    let bindings = authority_field("bindings").await?;
    let runs: Vec<Value> = sqlx::query_scalar("SELECT assurance_record('run',s.project,s.run_fingerprint) FROM release_run_submissions s WHERE s.project=$1 AND s.release_fingerprint=$2 ORDER BY s.run_fingerprint")
        .bind(project).bind(fingerprint).fetch_all(&state.pool).await?;
    Ok(requirements
        .iter()
        .map(|requirement| requirement_check_account(&release["release"], requirement, &bindings, &runs))
        .collect())
}

fn requirement_check_account(
    release: &Value,
    requirement: &Value,
    bindings: &[Value],
    runs: &[Value],
) -> Value {
    let scenarios: Vec<&str> = requirement["scenarios"]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut check_scenarios = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for binding in bindings {
        if let (Some(check), Some(scenario)) =
            (binding["check"].as_str(), binding["scenario"].as_str())
        {
            if scenarios.contains(&scenario) {
                check_scenarios.entry(check).or_default().push(scenario);
            }
        }
    }
    let checks: Vec<Value> = check_scenarios
        .into_iter()
        .map(|(check_id, scenarios)| {
            let mut executed_for_release = Vec::new();
            let mut would_apply = Vec::new();
            let mut skipped = false;
            let mut inconclusive = false;
            for run in runs {
                let Some(execution) = run["check_executions"]
                    .as_array()
                    .and_then(|list| list.iter().find(|e| e["check"]["id"] == check_id))
                else {
                    continue;
                };
                let units: Vec<Value> = execution["units"]
                    .as_array()
                    .map(|list| {
                        list.iter()
                            .map(|unit| {
                                json!({
                                    "id":unit["id"],
                                    "input_fingerprint":unit["input_fingerprint"],
                                    "matched":unit_matches_release(release, unit),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let run_skipped = execution["units"].as_array().is_some_and(|list| {
                    list.iter().any(|unit| {
                        unit["executed_tests"].as_array().is_some_and(|tests| {
                            tests.iter().any(|test| {
                                matches!(test["outcome"].as_str(), Some("skipped" | "not-executed"))
                            })
                        })
                    })
                });
                let outcomes: serde_json::Map<String, Value> = execution["results"]
                    .as_array()
                    .map(|results| {
                        results
                            .iter()
                            .filter(|result| {
                                result["scenario"].as_str().is_some_and(|s| scenarios.contains(&s))
                            })
                            .map(|result| {
                                (
                                    result["scenario"].as_str().unwrap_or_default().to_owned(),
                                    result["outcome"].clone(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if outcomes.is_empty() {
                    continue;
                }
                skipped |= run_skipped;
                inconclusive |= outcomes.values().any(|o| o == "inconclusive");
                let entry = json!({
                    "run_fingerprint":run["bundle_fingerprint"],
                    "run_id":run["run_id"],
                    "subject":run_subject(run),
                    "executed_at_ms":run["finished_at_ms"],
                    "units":units,
                    "outcomes":outcomes,
                    "skipped_tests":run_skipped,
                });
                if run_has_release_subject(release, run) {
                    executed_for_release.push(entry);
                } else if !units.is_empty() && units.iter().all(|unit| unit["matched"] == true) {
                    would_apply.push(entry);
                }
            }
            let state = if executed_for_release.is_empty() && would_apply.is_empty() {
                "missing"
            } else if skipped {
                "skipped"
            } else if inconclusive {
                "inconclusive"
            } else if executed_for_release.is_empty() {
                "would-apply"
            } else {
                "executed"
            };
            json!({
                "check_id":check_id,
                "scenarios":scenarios,
                "state":state,
                "executed_for_release":executed_for_release,
                "would_apply":would_apply,
            })
        })
        .collect();
    json!({
        "requirement":requirement["id"],
        "criticality":requirement["criticality"],
        "checks":checks,
    })
}

fn run_subject(run: &Value) -> Value {
    let target = &run["verification_target"];
    match target["kind"].as_str() {
        Some("source" | "ci-candidate" | "workspace") => target["repositories"]
            .as_array()
            .and_then(|list| list.iter().min_by_key(|r| r["id"].as_str().unwrap_or_default()))
            .map_or(Value::Null, |repository| {
                json!({"repository":repository["id"],"commit":repository["revision"]})
            }),
        Some("artifact") => json!({"artifacts":target["artifacts"]}),
        _ => Value::Null,
    }
}

fn run_has_release_subject(release: &Value, run: &Value) -> bool {
    let target = &run["verification_target"];
    match target["kind"].as_str() {
        Some("source" | "ci-candidate") => {
            let sources = release["sources"].as_array();
            target["repositories"].as_array().is_some_and(|repositories| {
                !repositories.is_empty()
                    && repositories.iter().all(|repository| {
                        sources.is_some_and(|sources| {
                            sources.iter().any(|source| {
                                source["repository"] == repository["id"]
                                    && source["commit"] == repository["revision"]
                                    && source["content_fingerprint"]
                                        == repository["content_fingerprint"]
                            })
                        })
                    })
            })
        }
        Some("artifact") => {
            let released = release["artifacts"].as_array();
            target["artifacts"].as_array().is_some_and(|artifacts| {
                !artifacts.is_empty()
                    && artifacts.iter().all(|artifact| {
                        released.is_some_and(|list| list.iter().any(|a| a["digest"] == artifact["digest"]))
                    })
            })
        }
        _ => false,
    }
}

fn unit_matches_release(release: &Value, unit: &Value) -> bool {
    release["unit_inputs"]["units"].as_array().is_some_and(|manifest| {
        manifest.iter().any(|entry| {
            entry["id"] == unit["id"]
                && entry["hermetic"] == true
                && entry["input_fingerprint"] == unit["input_fingerprint"]
        })
    })
}
pub(super) async fn show(
    State(state): State<LedgerState>,
    Path((project, fingerprint)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let record: Option<Value> = sqlx::query_scalar(&format!("SELECT jsonb_build_object('publication',jsonb_set(assurance_record('release',r.project,r.fingerprint),'{{release,integration}}',COALESCE({RELEASE_INTEGRATION},'null'::jsonb)),'submitted_by',submitted_by,'submitted_at',submitted_at,'deployments',COALESCE((SELECT jsonb_agg(assurance_record('deployment',d.project,d.deployment_id)) FROM release_deployments d WHERE d.project=r.project AND d.release_fingerprint=r.fingerprint),'[]'::jsonb)) FROM assurance_releases r WHERE project=$1 AND fingerprint=$2"))
        .bind(&project).bind(&fingerprint).fetch_optional(&state.pool).await?;
    Ok(Json(record.ok_or_else(LedgerError::not_found)?))
}
pub(super) fn paged(
    rows: Vec<sqlx::postgres::PgRow>,
    limit: i64,
) -> Result<Json<Value>, LedgerError> {
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
    let rows=sqlx::query("SELECT change_id AS fingerprint,jsonb_build_object('id',change_id,'revisions',jsonb_agg(jsonb_build_object('release',release_fingerprint,'change',assurance_record('change',project,release_fingerprint||'|'||change_id)))) AS payload FROM release_changes WHERE project=$1 AND change_id>$2 GROUP BY change_id ORDER BY change_id LIMIT $3")
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
    let rows=sqlx::query("SELECT deployment_id AS fingerprint,jsonb_build_object('deployment',assurance_record('deployment',project,deployment_id),'release_fingerprint',release_fingerprint,'submitted_by',submitted_by,'submitted_at',submitted_at) AS payload FROM release_deployments WHERE project=$1 AND deployment_id>$2 ORDER BY deployment_id LIMIT $3")
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
        "SELECT assurance_record_field('release',project,fingerprint,'release') FROM assurance_releases WHERE project=$1 AND fingerprint=$2",
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
pub(super) async fn insert_deployment(
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
    let existing: Option<(String,Value)>=sqlx::query_as("SELECT release_fingerprint,assurance_record('deployment',project,deployment_id) FROM release_deployments WHERE project=$1 AND deployment_id=$2")
        .bind(project).bind(id).fetch_optional(&mut **tx).await?;
    if let Some((previous, payload)) = existing {
        if previous != release || payload != *deployment {
            return Err(LedgerError::conflict("deployment identity is immutable"));
        }
        return Ok(());
    }
    crate::relational::store(&mut **tx, "deployment", project, id, deployment).await?;
    sqlx::query("INSERT INTO release_deployments(project,deployment_id,release_fingerprint,submitted_by) VALUES($1,$2,$3,$4)")
        .bind(project).bind(id).bind(release).bind(submitter).execute(&mut **tx).await?;
    Ok(())
}

const MAIN_PUBLICATION_FIELDS: [&str; 7] = [
    "format",
    "version",
    "commit",
    "main_sequence",
    "authority",
    "reviews",
    "ratification",
];
const RATIFICATION_FIELDS: [&str; 4] = ["status", "reason", "pull_request", "verified_at_ms"];

struct MainPublication {
    payload: Value,
    commit: String,
    main_sequence: i64,
    ratification_status: String,
    verified_at_ms: i64,
    authority: CoreJson,
    reviews: Vec<CoreJson>,
}

fn parse_main_publication(body: &[u8], project: &str) -> Result<MainPublication, LedgerError> {
    let payload = to_value(&core_json(body)?)?;
    let fields = payload
        .as_object()
        .ok_or_else(|| LedgerError::invalid("main model publication requires an object"))?;
    if let Some(key) = fields
        .keys()
        .find(|key| *key != "catalog" && !MAIN_PUBLICATION_FIELDS.contains(&key.as_str()))
    {
        return Err(LedgerError::invalid(format!(
            "main model publication: undeclared field {key}"
        )));
    }
    if payload["format"] != "azimuth-main-model-publication" || payload["version"] != 1 {
        return Err(LedgerError::invalid(
            "azimuth-main-model-publication version 1 required",
        ));
    }
    let commit = text(&payload, "commit")?;
    if !is_commit(commit) {
        return Err(LedgerError::invalid(
            "commit requires 40 or 64 lowercase hex digits",
        ));
    }
    let main_sequence = payload["main_sequence"]
        .as_i64()
        .filter(|sequence| *sequence >= 1)
        .ok_or_else(|| LedgerError::invalid("main_sequence requires a positive integer"))?;
    let ratification = payload["ratification"]
        .as_object()
        .filter(|fields| {
            fields
                .keys()
                .all(|key| RATIFICATION_FIELDS.contains(&key.as_str()))
        })
        .ok_or_else(|| {
            LedgerError::invalid(
                "ratification requires status, reason, pull_request and verified_at_ms only",
            )
        })?;
    let ratification_status = ratification
        .get("status")
        .and_then(Value::as_str)
        .filter(|status| matches!(*status, "ratified" | "unratified"))
        .ok_or_else(|| LedgerError::invalid("ratification status requires ratified or unratified"))?
        .to_owned();
    let verified_at_ms = ratification
        .get("verified_at_ms")
        .and_then(Value::as_i64)
        .filter(|verified| *verified >= 0)
        .ok_or_else(|| {
            LedgerError::invalid("ratification verified_at_ms requires a positive integer")
        })?;
    let authority = core_json(payload["authority"].to_string().as_bytes())?;
    assurance_review::validate_authority(&authority).map_err(LedgerError::invalid)?;
    if payload["authority"]["project"].as_str() != Some(project) {
        return Err(LedgerError::invalid("authority project must match route"));
    }
    let mut reviews = Vec::new();
    for record in array(&payload, "reviews")? {
        let native = core_json(record.to_string().as_bytes())?;
        assurance_review::validate_record(&native).map_err(LedgerError::invalid)?;
        if !matches!(
            record["kind"].as_str(),
            Some("requirement-review" | "requirement-status")
        ) {
            return Err(LedgerError::invalid(
                "main model publications carry requirement-review or requirement-status records only",
            ));
        }
        reviews.push(native);
    }
    Ok(MainPublication {
        commit: commit.to_owned(),
        main_sequence,
        ratification_status,
        verified_at_ms,
        authority,
        reviews,
        payload,
    })
}

async fn run_heads_for_model(
    pool: &sqlx::PgPool,
    project: &str,
    model_fingerprint: &str,
) -> Result<Vec<run::RunBundle>, LedgerError> {
    let rows: Vec<Value> = sqlx::query_scalar("SELECT assurance_record('run',project,fingerprint) FROM (SELECT DISTINCT ON (run_id) project,fingerprint FROM run_bundles WHERE project=$1 AND model_fingerprint=$2 ORDER BY run_id,revision DESC LIMIT 5001) heads")
        .bind(project).bind(model_fingerprint).fetch_all(pool).await?;
    if rows.len() > 5000 {
        return Err(LedgerError::invalid(
            "review re-assessment exceeds 5000 Runs; no partial assessment is inferred",
        ));
    }
    rows.iter()
        .map(|value| parse_bundle(&value.to_string()))
        .collect()
}

async fn replayed_main_publication(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project: &str,
    commit: &str,
) -> Result<Value, LedgerError> {
    let assessments = sqlx::query("SELECT review_fingerprint,assessment,diagnostics FROM main_publication_reviews WHERE project=$1 AND commit_sha=$2 ORDER BY review_fingerprint")
        .bind(project).bind(commit).fetch_all(&mut **tx).await?;
    let reviews = assessments
        .iter()
        .map(|row| {
            Ok(json!({"fingerprint":row.try_get::<String,_>("review_fingerprint")?,"assessment":row.try_get::<String,_>("assessment")?,"diagnostics":row.try_get::<Vec<String>,_>("diagnostics")?}))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(Value::Array(reviews))
}

pub(super) async fn publish_main(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), LedgerError> {
    let submitter = state.authorize(&project, &headers, Some(Role::Publisher))?;
    let route_project = project.clone();
    let publication =
        tokio::task::spawn_blocking(move || parse_main_publication(&body, &route_project))
            .await
            .map_err(validation_worker_error)??;
    let revision = text(&publication.payload["authority"], "fingerprint")?.to_owned();
    let bundles = run_heads_for_model(
        &state.pool,
        &project,
        text(&publication.payload["authority"], "model_fingerprint")?,
    )
    .await?;
    let assessments = compute({
        let (authority, reviews) = (publication.authority.clone(), publication.reviews.clone());
        move || {
            Ok(reviews
                .iter()
                .map(|record| {
                    (
                        record.clone(),
                        assurance_review::assess_record(record, &authority, &reviews, &bundles),
                    )
                })
                .collect::<Vec<_>>())
        }
    })
    .await?;
    if let Some((_, rejected)) = assessments.iter().find(|(_, a)| a.status == "invalid") {
        return Err(LedgerError::invalid(rejected.diagnostics.join("; ")));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|main-publication"))
        .execute(&mut *tx)
        .await?;
    let commit = publication.commit.as_str();
    let existing: Option<Value> = sqlx::query_scalar(
        "SELECT assurance_record('main_publication',project,commit_sha) FROM main_publications WHERE project=$1 AND commit_sha=$2",
    )
    .bind(&project)
    .bind(commit)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(existing) = existing {
        if existing != publication.payload {
            return Err(LedgerError::conflict(
                "main model publication for this commit has different content",
            ));
        }
        let reviews = replayed_main_publication(&mut tx, &project, commit).await?;
        tx.commit().await?;
        return Ok((
            StatusCode::OK,
            Json(
                json!({"project":project,"commit":commit,"main_sequence":publication.main_sequence,"model_revision":revision,"ratification_status":publication.ratification_status,"review_assessments":reviews,"replayed":true}),
            ),
        ));
    }
    let sequence_owner: Option<String> = sqlx::query_scalar(
        "SELECT commit_sha FROM main_publications WHERE project=$1 AND main_sequence=$2",
    )
    .bind(&project)
    .bind(publication.main_sequence)
    .fetch_optional(&mut *tx)
    .await?;
    if sequence_owner.is_some() {
        return Err(LedgerError::conflict(
            "main_sequence is already published for a different commit",
        ));
    }
    crate::relational::store(
        &mut *tx,
        "main_publication",
        &project,
        commit,
        &publication.payload,
    )
    .await?;
    sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(&project).bind(&revision).bind(&submitter).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(&project).bind(&revision).bind(&submitter).execute(&mut *tx).await?;
    if let Some(catalog) = publication.payload.get("catalog") {
        model_catalog::store(
            &mut tx,
            &project,
            &revision,
            catalog,
            &submitter,
            "publisher-publication",
        )
        .await?;
    }
    sqlx::query("INSERT INTO main_publications(project,commit_sha,main_sequence,model_revision,ratification_status,ratification_verified_at_ms,published_by) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(&project).bind(commit).bind(publication.main_sequence).bind(&revision).bind(&publication.ratification_status).bind(publication.verified_at_ms).bind(&submitter).execute(&mut *tx).await?;
    let mut review_assessments = Vec::new();
    for (record, assessment) in &assessments {
        let value = to_value(record)?;
        let fingerprint = text(&value, "fingerprint")?;
        let target = assurance_review::review_target(record).map_err(LedgerError::invalid)?;
        sqlx::query("INSERT INTO assurance_reviews(project,fingerprint,target_fingerprint,submitted_by,provenance_basis) VALUES($1,$2,$3,$4,'publisher-transported-assertion') ON CONFLICT DO NOTHING")
            .bind(&project).bind(fingerprint).bind(run::canonical_fingerprint(&target).map_err(LedgerError::invalid)?).bind(&submitter).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO model_revision_reviews(project,revision_fingerprint,review_fingerprint,submitted_by) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
            .bind(&project).bind(&revision).bind(fingerprint).bind(&submitter).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO main_publication_reviews(project,commit_sha,review_fingerprint,assessment,diagnostics) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
            .bind(&project).bind(commit).bind(fingerprint).bind(&assessment.status).bind(&assessment.diagnostics).execute(&mut *tx).await?;
        review_assessments.push(json!({"fingerprint":fingerprint,"assessment":assessment.status,"diagnostics":assessment.diagnostics}));
    }
    let is_default: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS(SELECT 1 FROM main_publications WHERE project=$1 AND main_sequence>$2)",
    )
    .bind(&project)
    .bind(publication.main_sequence)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(
            json!({"project":project,"commit":commit,"main_sequence":publication.main_sequence,"model_revision":revision,"ratification_status":publication.ratification_status,"default":is_default,"review_assessments":review_assessments,"replayed":false,"review_provenance":"publisher-transported-assertion"}),
        ),
    ))
}
