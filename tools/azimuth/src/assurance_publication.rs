use crate::json::Json;

pub fn verify_snapshot(value: &Json) -> Result<Json, String> {
    let fields = [
        "project",
        "model_revision",
        "authority",
        "reviews",
        "bundles",
    ];
    let Json::Obj(items) = value else {
        return Err("expected assurance snapshot object".into());
    };
    if items.len() != fields.len()
        || items.iter().any(|(key, _)| !fields.contains(&key.as_str()))
        || fields.iter().any(|key| value.get(key).is_none())
    {
        return Err(
            "expected exact project, model_revision, authority, reviews and bundles fields".into(),
        );
    }
    let authority = value.get("authority").unwrap();
    crate::assurance_review::validate_authority(authority)?;
    if value.get("project") != authority.get("project")
        || value.get("model_revision") != authority.get("fingerprint")
    {
        return Err("snapshot project/revision differs from its authority".into());
    }
    let reviews = value
        .get("reviews")
        .and_then(Json::as_array)
        .ok_or("reviews must be an array")?;
    for review in reviews {
        crate::assurance_review::validate_record(review)?;
        if !matches!(
            review.get("kind").and_then(Json::as_str),
            Some("requirement-review" | "requirement-status")
        ) {
            return Err("publication requires current Requirement Review/Status kinds; historical records remain in their existing ledger".into());
        }
    }
    let raw_bundles = value
        .get("bundles")
        .and_then(Json::as_array)
        .ok_or("bundles must be an array")?;
    let bundles = raw_bundles
        .iter()
        .map(|bundle| {
            crate::run::parse("snapshot.bundles", &bundle.to_string_pretty())
                .map_err(|errors| format!("invalid snapshot Run: {errors:?}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let findings = crate::run::verify_set(&bundles);
    if !findings.is_empty() {
        return Err(format!("invalid snapshot Run history: {findings:?}"));
    }
    let assessments = reviews
        .iter()
        .map(|record| {
            let assessment =
                crate::assurance_review::assess_record(record, authority, reviews, &bundles);
            Json::obj(vec![
                ("fingerprint", record.get("fingerprint").unwrap().clone()),
                ("status", Json::str(&assessment.status)),
                (
                    "diagnostics",
                    Json::Arr(assessment.diagnostics.iter().map(Json::str).collect()),
                ),
            ])
        })
        .collect();
    Ok(Json::obj(vec![
        (
            "format",
            Json::str("azimuth-assurance-publication-validation"),
        ),
        ("version", Json::Num(2.0)),
        ("project", value.get("project").unwrap().clone()),
        (
            "model_revision",
            value.get("model_revision").unwrap().clone(),
        ),
        (
            "snapshot_fingerprint",
            Json::str(crate::fingerprint::canonical_sha256(value)),
        ),
        ("valid", Json::Bool(true)),
        ("acceptance", Json::Bool(false)),
        ("assessments", Json::Arr(assessments)),
    ]))
}

pub fn supported_requirements(
    model: &crate::model::Model,
    snapshot: &Json,
) -> Result<std::collections::BTreeSet<String>, String> {
    verify_snapshot(snapshot)?;
    let project = snapshot
        .get("project")
        .and_then(Json::as_str)
        .ok_or("snapshot project missing")?;
    let expected = crate::assurance_review::authority(model, project)?;
    let authority = snapshot
        .get("authority")
        .ok_or("snapshot authority missing")?;
    if expected.get("fingerprint") != authority.get("fingerprint") {
        return Err("assurance snapshot authority does not match the complete reviewed model and source support".into());
    }
    let reviews = snapshot
        .get("reviews")
        .and_then(Json::as_array)
        .ok_or("snapshot reviews missing")?;
    let bundles = snapshot
        .get("bundles")
        .and_then(Json::as_array)
        .ok_or("snapshot Runs missing")?
        .iter()
        .map(|value| {
            crate::run::parse("snapshot.bundles", &value.to_string_pretty())
                .map_err(|errors| format!("invalid snapshot Run: {errors:?}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut reviewed = std::collections::BTreeSet::new();
    let mut status_supported = std::collections::BTreeSet::new();
    for record in reviews {
        let assessment =
            crate::assurance_review::assess_record(record, authority, reviews, &bundles);
        if assessment.status != "accepted" {
            continue;
        }
        let requirement = record
            .get("requirement")
            .and_then(Json::as_str)
            .ok_or("review Requirement missing")?
            .to_string();
        match record.get("kind").and_then(Json::as_str) {
            Some("requirement-review")
                if record.get("decision").and_then(Json::as_str) == Some("accepted") =>
            {
                reviewed.insert(requirement);
            }
            Some("requirement-status")
                if record.get("conclusion").and_then(Json::as_str) == Some("supported") =>
            {
                status_supported.insert(requirement);
            }
            _ => {}
        }
    }
    Ok(reviewed.intersection(&status_supported).cloned().collect())
}
