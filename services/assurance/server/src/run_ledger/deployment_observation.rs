use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Observation {
    expected_revision: i64,
    observation: Value,
}

pub(super) async fn record(
    State(state): State<LedgerState>,
    Path((project, environment)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<Observation>,
) -> Result<Json<Value>, LedgerError> {
    let publisher = state.authorize(&project, &headers, Some(Role::Publisher))?;
    azimuth::diag::validate_id(&environment, false).map_err(LedgerError::invalid)?;
    let o = &body.observation;
    let keys = [
        "format",
        "version",
        "observed_at_ms",
        "valid_for_seconds",
        "state",
        "release_fingerprint",
        "applied_revision",
        "rollout_id",
        "configuration_fingerprint",
        "components",
        "diagnostics",
    ];
    if !o.is_object()
        || o.as_object().unwrap().len() != keys.len()
        || keys.iter().any(|k| o.get(*k).is_none())
        || o["format"] != "azimuth-deployment-observation"
        || o["version"] != 1
    {
        return Err(LedgerError::invalid(
            "expected strict azimuth-deployment-observation version 1",
        ));
    }
    let observed = o["observed_at_ms"]
        .as_i64()
        .ok_or_else(|| LedgerError::invalid("observed_at_ms required"))?;
    let lifetime = o["valid_for_seconds"]
        .as_i64()
        .filter(|v| (15..=300).contains(v))
        .ok_or_else(|| LedgerError::invalid("valid_for_seconds must be 15..300"))?;
    let status = o["state"].as_str().unwrap_or("");
    if ![
        "complete",
        "progressing",
        "failed",
        "partial",
        "unknown",
        "drifted",
    ]
    .contains(&status)
        || body.expected_revision < 0
        || body.expected_revision == i64::MAX
        || o.to_string().len() > 262144
    {
        return Err(LedgerError::invalid(
            "invalid observation state, revision or size",
        ));
    }
    if !o["components"].is_array()
        || !o["diagnostics"].is_array()
        || o["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| !v.is_string())
    {
        return Err(LedgerError::invalid(
            "components and diagnostics arrays required",
        ));
    }
    if !o["release_fingerprint"].is_null() && !valid_digest(&o["release_fingerprint"])
        || !o["configuration_fingerprint"].is_null()
            && !valid_digest(&o["configuration_fingerprint"])
    {
        return Err(LedgerError::invalid("invalid observation fingerprint"));
    }
    if status == "complete"
        && o["rollout_id"]
            .as_str()
            .is_none_or(|s| s.trim().is_empty() || s.len() > 256)
    {
        return Err(LedgerError::invalid("complete rollout identity required"));
    }
    if o["components"].as_array().unwrap().len() > 32 {
        return Err(LedgerError::invalid("at most 32 components permitted"));
    }
    validate_components(&o["components"])?;
    let fp = format!(
        "sha256:{}",
        sha256(
            run::canonical_json(&core_json(o.to_string().as_bytes())?)
                .map_err(LedgerError::invalid)?
                .as_bytes()
        )
    );
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{project}|observation|{environment}"))
        .execute(&mut *tx)
        .await?;
    let replay:Option<(i64,Value,String)>=sqlx::query_as("SELECT revision,assurance_record('deployment_observation',project,fingerprint),submitted_by FROM deployment_observations WHERE project=$1 AND environment=$2 AND fingerprint=$3").bind(&project).bind(&environment).bind(&fp).fetch_optional(&mut *tx).await?;
    if let Some((revision, payload, by)) = replay {
        if payload != *o || by != publisher {
            return Err(LedgerError::conflict(
                "observation identity belongs to another publisher",
            ));
        }
        tx.commit().await?;
        return Ok(Json(
            json!({"fingerprint":fp,"revision":revision,"replayed":true}),
        ));
    }
    let now: i64 =
        sqlx::query_scalar("SELECT (extract(epoch FROM clock_timestamp())*1000)::bigint")
            .fetch_one(&mut *tx)
            .await?;
    if observed > now + 5000 || observed < now - lifetime * 1000 {
        return Err(LedgerError::invalid(
            "observation is stale or in the future; observe again",
        ));
    }
    let previous:Option<(i64,i64)>=sqlx::query_as("SELECT revision,observed_at_ms FROM deployment_observations WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1").bind(&project).bind(&environment).fetch_optional(&mut *tx).await?;
    if previous.as_ref().map(|v| v.0).unwrap_or(0) != body.expected_revision
        || previous.as_ref().is_some_and(|v| observed <= v.1)
    {
        return Err(LedgerError::conflict(
            "observation advanced; read current revision before observing again",
        ));
    }
    let mut observed_deployment: Option<String> = None;
    if status == "complete" {
        let release = o["release_fingerprint"].as_str().ok_or_else(|| {
            LedgerError::invalid("complete observation requires release fingerprint")
        })?;
        let artifacts: Option<Value> = sqlx::query_scalar(
            "SELECT assurance_record_field('release',project,fingerprint,'release') FROM assurance_releases WHERE project=$1 AND fingerprint=$2",
        )
        .bind(&project)
        .bind(release)
        .fetch_optional(&mut *tx)
        .await?;
        let artifacts = artifacts.ok_or_else(LedgerError::not_found)?;
        let expected_components = artifacts["components"]
            .as_array()
            .filter(|c| !c.is_empty())
            .ok_or_else(|| {
                LedgerError::invalid("release requires explicit deployed component bindings")
            })?;
        let allowed: std::collections::BTreeSet<String> = expected_components
            .iter()
            .filter_map(|a| a["digest"].as_str().map(str::to_owned))
            .collect();
        let mut seen = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for c in o["components"].as_array().unwrap() {
            if c.as_object()
                .is_none_or(|v| v.len() != 4 + usize::from(v.contains_key("instances")))
                || ["name", "image", "template_fingerprint", "ready"]
                    .iter()
                    .any(|k| c.get(*k).is_none())
                || c["ready"] != true
            {
                return Err(LedgerError::invalid(
                    "complete component requires name, image, template_fingerprint and ready=true",
                ));
            }
            let name = c["name"]
                .as_str()
                .filter(|n| !n.is_empty())
                .ok_or_else(|| LedgerError::invalid("component name required"))?;
            if !names.insert(name) {
                return Err(LedgerError::invalid("duplicate component name"));
            }
            let image = c["image"].as_str().unwrap_or("");
            let digest = image.rsplit_once('@').map(|v| v.1).unwrap_or("");
            if !expected_components
                .iter()
                .any(|e| e["id"] == name && e["digest"] == digest)
            {
                return Err(LedgerError::invalid(
                    "observed image is not a release artifact",
                ));
            }
            seen.insert(digest.to_string());
        }
        if seen != allowed
            || names.len() != expected_components.len()
            || seen.is_empty()
            || o["applied_revision"]
                .as_str()
                .is_none_or(|v| v.len() != 40 && v.len() != 64)
        {
            return Err(LedgerError::invalid(
                "complete observation must cover the exact release artifacts and applied revision",
            ));
        }
        let config = o["configuration_fingerprint"]
            .as_str()
            .filter(|v| v.starts_with("sha256:") && v.len() == 71)
            .ok_or_else(|| LedgerError::invalid("observed configuration fingerprint required"))?;
        let identity=core_json(json!({"environment":environment,"release":release,"configuration":config,"rollout_id":o["rollout_id"],"applied_revision":o["applied_revision"]}).to_string().as_bytes())?;
        let deployment_fp = run::canonical_fingerprint(&identity).map_err(LedgerError::invalid)?;
        let deployment_id = format!("observed-{}", &deployment_fp[7..]);
        let (subject, native) =
            observed_subject(&environment, &deployment_id, config, expected_components)?;
        observed_deployment = Some(deployment_id.clone());
        let deployment = json!({"id":deployment_id,"environment":environment,"configuration_fingerprint":config,"artifacts":seen,"subject":subject,"subject_fingerprint":run::subject_fingerprint(&native)});
        release::insert_deployment(&mut tx, &project, release, &deployment, &publisher).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("{project}|current-deployment|{environment}"))
            .execute(&mut *tx)
            .await?;
        let selected:Option<String>=sqlx::query_scalar("SELECT deployment_id FROM environment_deployment_selections WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1").bind(&project).bind(&environment).fetch_optional(&mut *tx).await?;
        if selected.as_deref() != deployment["id"].as_str() {
            let revision:i64=sqlx::query_scalar("SELECT COALESCE(MAX(revision),0)+1 FROM environment_deployment_selections WHERE project=$1 AND environment=$2").bind(&project).bind(&environment).fetch_one(&mut *tx).await?;
            sqlx::query("INSERT INTO environment_deployment_selections(project,environment,revision,deployment_id,reason,selected_by) VALUES($1,$2,$3,$4,$5,$6)").bind(&project).bind(&environment).bind(revision).bind(deployment["id"].as_str().unwrap()).bind(format!("Observed complete rollout {fp}")).bind(&publisher).execute(&mut *tx).await?;
        }
    }
    let revision = body.expected_revision + 1;
    crate::relational::store(&mut *tx, "deployment_observation", &project, &fp, o).await?;
    sqlx::query("INSERT INTO deployment_observations(project,environment,revision,fingerprint,submitted_by,deployment_id,observed_at_ms,valid_for_seconds,state,release_fingerprint) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)").bind(&project).bind(&environment).bind(revision).bind(&fp).bind(&publisher).bind(observed_deployment).bind(observed).bind(lifetime).bind(status).bind(o["release_fingerprint"].as_str()).execute(&mut *tx).await?;
    tx.commit().await?;
    target_observation::resolve_pending(&state, &project).await?;
    Ok(Json(
        json!({"fingerprint":fp,"revision":revision,"replayed":false}),
    ))
}

pub(super) async fn current(
    State(state): State<LedgerState>,
    Path((project, environment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let record:Option<Value>=sqlx::query_scalar("SELECT jsonb_build_object('revision',revision,'deployment_id',deployment_id,'fingerprint',fingerprint,'observation',assurance_record('deployment_observation',project,fingerprint),'submitted_by',submitted_by,'received_at',received_at,'fresh',(extract(epoch FROM clock_timestamp())*1000)::bigint <= observed_at_ms+valid_for_seconds*1000) FROM deployment_observations WHERE project=$1 AND environment=$2 ORDER BY revision DESC LIMIT 1").bind(&project).bind(&environment).fetch_optional(&state.pool).await?;
    record.map(Json).ok_or_else(LedgerError::not_found)
}

fn valid_digest(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.strip_prefix("sha256:").is_some_and(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
    })
}

fn validate_components(value: &Value) -> Result<(), LedgerError> {
    let components = value
        .as_array()
        .ok_or_else(|| LedgerError::invalid("components array required"))?;
    let keys = ["name", "image", "template_fingerprint", "ready"];
    for component in components {
        if component.as_object().is_none_or(|fields| {
            keys.iter().any(|key| !fields.contains_key(*key))
                || fields
                    .keys()
                    .any(|key| !keys.contains(&key.as_str()) && key != "instances")
        }) || component["name"]
            .as_str()
            .is_none_or(|name| name.trim().is_empty())
            || !(component["image"].is_null() || component["image"].is_string())
            || !component["ready"].is_boolean()
            || !valid_digest(&component["template_fingerprint"])
            || component.get("instances").is_some_and(|instances| {
                let Some(values) = instances.as_array() else {
                    return true;
                };
                let mut unique = std::collections::BTreeSet::new();
                values.iter().any(|value| {
                    value
                        .as_str()
                        .is_none_or(|id| id.is_empty() || !unique.insert(id))
                })
            })
        {
            return Err(LedgerError::invalid(
                "component requires name, nullable image, template fingerprint and boolean ready with no extra fields",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component() -> Value {
        json!({"name":"web-api","image":null,"template_fingerprint":format!("sha256:{}", "a".repeat(64)),"ready":false})
    }

    #[test]
    fn incomplete_component_preserves_nullable_image() {
        assert!(validate_components(&json!([component()])).is_ok());
    }

    #[test]
    fn component_shape_rejects_untyped_or_unknown_fields() {
        for (field, value) in [
            ("name", json!(null)),
            ("name", json!(" ")),
            ("image", json!(42)),
            ("ready", json!("false")),
            ("ready", json!(null)),
            ("template_fingerprint", json!("invalid")),
            ("unknown", json!(true)),
        ] {
            let mut invalid = component();
            invalid[field] = value;
            assert!(
                validate_components(&json!([invalid])).is_err(),
                "accepted invalid {field}"
            );
        }
        let mut missing = component();
        missing.as_object_mut().unwrap().remove("image");
        assert!(validate_components(&json!([missing])).is_err());
        assert!(validate_components(&json!(["not an object"])).is_err());
    }
}

fn observed_subject(
    environment: &str,
    deployment: &str,
    configuration: &str,
    components: &[Value],
) -> Result<(Value, run::Subject), LedgerError> {
    let mut artifacts: Vec<Value> = components
        .iter()
        .map(|component| json!({"id":component["id"],"digest":component["digest"]}))
        .collect();
    artifacts.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    let subject = json!({"kind":"deployment","environment":environment,"deployment":deployment,"deployment_fingerprint":configuration,"artifacts":artifacts});
    let native = run::subject_from_json(&core_json(subject.to_string().as_bytes())?)
        .map_err(LedgerError::invalid)?;
    Ok((subject, native))
}

#[cfg(test)]
mod subject_tests {
    use super::*;

    #[test]
    fn release_component_order_does_not_change_the_valid_deployment_subject() {
        let digest = format!("sha256:{}", "a".repeat(64));
        let components = [
            json!({"id":"web","digest":digest}),
            json!({"id":"api","digest":digest}),
            json!({"id":"gateway","digest":digest}),
        ];
        let (subject, native) = observed_subject("production", "rollout", &digest, &components)
            .unwrap_or_else(|error| panic!("{}", error.detail));
        let reordered = [
            components[2].clone(),
            components[0].clone(),
            components[1].clone(),
        ];
        let (other, other_native) = observed_subject("production", "rollout", &digest, &reordered)
            .unwrap_or_else(|error| panic!("{}", error.detail));
        assert_eq!(subject, other);
        assert_eq!(
            run::subject_fingerprint(&native),
            run::subject_fingerprint(&other_native)
        );
        let parsed = run::subject_from_json(
            &core_json(subject.to_string().as_bytes())
                .unwrap_or_else(|error| panic!("{}", error.detail)),
        )
        .unwrap();
        assert_eq!(
            run::subject_fingerprint(&native),
            run::subject_fingerprint(&parsed)
        );
        assert_eq!(subject["artifacts"][0]["id"], "api");
        assert_eq!(subject["artifacts"][1]["id"], "gateway");
        assert_eq!(subject["artifacts"][2]["id"], "web");
    }
}
