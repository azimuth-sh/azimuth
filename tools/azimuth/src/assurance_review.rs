use crate::json::Json;
use crate::model::Model;
use crate::run::{ObservationOutcome, RunBundle};
use std::collections::{BTreeMap, BTreeSet};

pub fn authority(model: &Model, project: &str) -> Result<Json, String> {
    crate::diag::validate_id(project, false)?;
    let checks = model
        .checks()
        .map(|check| {
            Json::obj(vec![
                ("id", Json::str(&check.id)),
                (
                    "fingerprint",
                    Json::str(model.check_execution_fingerprint(check)),
                ),
            ])
        })
        .collect::<Vec<_>>();
    let cases = model
        .cases()
        .map(|case| {
            Json::obj(vec![
                ("id", Json::str(&case.case.id)),
                ("claim", Json::str(&case.claim.id)),
                (
                    "fingerprint",
                    Json::str(model.case_digest(&case.case.id).unwrap_or_default()),
                ),
            ])
        })
        .collect::<Vec<_>>();
    let mut bindings = Vec::new();
    for document in &model.account_verifications {
        for check in &document.checks {
            for binding in &check.bindings {
                let definition = Json::obj(vec![
                    ("check", Json::str(&check.definition.id)),
                    ("case", Json::str(&binding.case)),
                    (
                        "check_fingerprint",
                        Json::str(model.check_execution_fingerprint(&check.definition)),
                    ),
                    (
                        "case_fingerprint",
                        Json::str(model.case_digest(&binding.case).unwrap_or_default()),
                    ),
                    ("contribution", Json::str(&binding.contribution)),
                ]);
                let mut fields = match definition {
                    Json::Obj(fields) => fields,
                    _ => unreachable!(),
                };
                let fingerprint = crate::fingerprint::canonical_sha256(&Json::Obj(fields.clone()));
                fields.push(("fingerprint".into(), Json::str(fingerprint)));
                bindings.push(Json::Obj(fields));
            }
        }
    }
    for binding in model.evidence_bindings() {
        let Some(check) = model.checks().find(|check| check.id == binding.check) else {
            continue;
        };
        let definition = Json::obj(vec![
            ("check", Json::str(&binding.check)),
            ("case", Json::str(&binding.case)),
            (
                "check_fingerprint",
                Json::str(model.check_execution_fingerprint(check)),
            ),
            (
                "case_fingerprint",
                Json::str(model.case_digest(&binding.case).unwrap_or_default()),
            ),
            ("contribution", Json::str(&binding.proposition)),
        ]);
        let mut fields = match definition {
            Json::Obj(fields) => fields,
            _ => unreachable!(),
        };
        fields.push((
            "fingerprint".into(),
            Json::str(crate::fingerprint::canonical_sha256(&Json::Obj(
                fields.clone(),
            ))),
        ));
        bindings.push(Json::Obj(fields));
    }
    let mut claims = Vec::new();
    for view in model.claims() {
        let id = &view.claim.id;
        let mut gaps = Vec::new();
        let mut mechanisms = Vec::new();
        if let Some((_, declaration)) = model.account_design_for_claim(&view.spec.id, id) {
            collect_mechanisms(model, declaration, &mut mechanisms, &mut gaps);
        } else if let Some(design) = model.design_for(&view.spec.id) {
            for entry in &design.entries {
                if entry.target.id() == *id {
                    for mechanism in &entry.mechanisms {
                        let implementations = implementation_records(model, &mechanism.id);
                        if implementations.is_empty() && mechanism.binding.is_none() {
                            gaps.push(format!("mechanism {} has no implementation", mechanism.id));
                        }
                        let record = Json::obj(vec![
                            ("id", Json::str(&mechanism.id)),
                            ("definition", Json::str(format!("{:?}", mechanism))),
                            ("implementations", Json::Arr(implementations)),
                        ]);
                        mechanisms.push(Json::obj(vec![
                            ("id", Json::str(&mechanism.id)),
                            (
                                "fingerprint",
                                Json::str(crate::fingerprint::canonical_sha256(&record)),
                            ),
                        ]));
                    }
                }
            }
        }
        let case_ids = view
            .claim
            .cases
            .iter()
            .map(|case| case.id.clone())
            .collect::<BTreeSet<_>>();
        let relevant = bindings
            .iter()
            .filter(|binding| {
                binding
                    .get("case")
                    .and_then(Json::as_str)
                    .is_some_and(|case| case_ids.contains(case))
            })
            .cloned()
            .collect::<Vec<_>>();
        let criticality = view
            .claim
            .criticality
            .map(|value| value.name())
            .unwrap_or("unspecified");
        if criticality != "routine" {
            for case in &case_ids {
                if !relevant
                    .iter()
                    .any(|binding| binding.get("case").and_then(Json::as_str) == Some(case))
                {
                    gaps.push(format!("Case {case} has no Evidence Binding"));
                }
            }
            for binding in &relevant {
                let check = text(binding, "check").unwrap_or_default();
                if !model.check_implementations.iter().any(|implementation| {
                    implementation.check == check
                        && implementation.source.is_some()
                        && !implementation.source_fingerprint.is_empty()
                }) {
                    gaps.push(format!("Check {check} has no stable source implementation"));
                }
            }
            if let Some(obligation) = model.realization_obligation(&view.spec.id, id) {
                for area in obligation.areas {
                    if !model.realizes.iter().any(|site| {
                        site.claim == *id
                            && site
                                .source
                                .as_ref()
                                .is_some_and(|source| source.area == area)
                            && !site.source_fingerprint.is_empty()
                    }) {
                        gaps.push(format!("Claim has no stable realization in Area {area}"));
                    }
                }
            } else {
                gaps.push("Claim has no design-owned realization obligation".into());
            }
        }
        gaps.sort();
        gaps.dedup();
        mechanisms.sort_by_key(|mechanism| text(mechanism, "id").unwrap_or_default().to_string());
        let dependencies = Json::obj(vec![
            (
                "intent",
                Json::str(model.claim_digest(id).unwrap_or_default()),
            ),
            (
                "design",
                model
                    .account_design_digest(&view.spec.id, id)
                    .map_or(Json::Null, Json::str),
            ),
            (
                "verification",
                model
                    .account_verification_digest(&view.spec.id, id)
                    .map_or(Json::Null, Json::str),
            ),
            ("mechanisms", Json::Arr(mechanisms.clone())),
            ("bindings", Json::Arr(relevant.clone())),
            (
                "realizations",
                Json::Arr({
                    let mut sites = model
                        .realizes
                        .iter()
                        .filter(|site| site.claim == *id)
                        .collect::<Vec<_>>();
                    sites.sort_by_key(|site| {
                        (
                            site.source.as_ref().map(|source| source.key()),
                            site.source_fingerprint.clone(),
                        )
                    });
                    sites
                        .into_iter()
                        .map(|site| {
                            Json::obj(vec![
                                (
                                    "source",
                                    site.source
                                        .as_ref()
                                        .map_or(Json::Null, |source| Json::str(source.key())),
                                ),
                                ("fingerprint", Json::str(&site.source_fingerprint)),
                            ])
                        })
                        .collect()
                }),
            ),
            ("gaps", Json::Arr(gaps.iter().map(Json::str).collect())),
        ]);
        claims.push(Json::obj(vec![
            ("id", Json::str(id)),
            ("criticality", Json::str(criticality)),
            (
                "fingerprint",
                Json::str(crate::fingerprint::canonical_sha256(&dependencies)),
            ),
            ("cases", Json::Arr(case_ids.iter().map(Json::str).collect())),
            ("mechanisms", Json::Arr(mechanisms)),
            (
                "bindings",
                Json::Arr(
                    relevant
                        .iter()
                        .map(|binding| {
                            Json::obj(vec![
                                ("check", binding.get("check").cloned().unwrap_or(Json::Null)),
                                ("case", binding.get("case").cloned().unwrap_or(Json::Null)),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("gaps", Json::Arr(gaps.iter().map(Json::str).collect())),
        ]));
    }
    let findings = crate::validation::validate(model);
    let result = Json::obj(vec![
        ("project", Json::str(project)),
        (
            "model_fingerprint",
            Json::str(format!(
                "sha256:{}",
                crate::fingerprint::model_digest(model, &findings)
            )),
        ),
        ("checks", sorted(checks, "id")),
        ("cases", sorted(cases, "id")),
        ("bindings", sorted_pairs(bindings)),
        ("claims", sorted(claims, "id")),
    ]);
    let mut result = result;
    let digest = record_fingerprint(&result)?;
    if let Json::Obj(fields) = &mut result {
        fields.push(("fingerprint".into(), Json::str(digest)));
    }
    validate_authority(&result)?;
    Ok(result)
}
fn sorted(mut values: Vec<Json>, field: &str) -> Json {
    values.sort_by_key(|value| text(value, field).unwrap_or_default().to_string());
    Json::Arr(values)
}
fn sorted_pairs(mut values: Vec<Json>) -> Json {
    values.sort_by_key(|value| {
        (
            text(value, "check").unwrap_or_default().to_string(),
            text(value, "case").unwrap_or_default().to_string(),
        )
    });
    Json::Arr(values)
}
fn implementation_records(model: &Model, id: &str) -> Vec<Json> {
    let mut values = model
        .mechanism_implementations
        .iter()
        .filter(|implementation| implementation.mechanism == id)
        .map(|implementation| {
            Json::obj(vec![
                (
                    "source",
                    implementation
                        .source
                        .as_ref()
                        .map_or(Json::Null, |source| Json::str(source.key())),
                ),
                ("fingerprint", Json::str(&implementation.source_fingerprint)),
            ])
        })
        .collect::<Vec<_>>();
    values.sort_by_key(|value| text(value, "source").unwrap_or_default().to_string());
    values
}
fn collect_mechanisms(
    model: &Model,
    declaration: &crate::account_design::Declaration,
    values: &mut Vec<Json>,
    gaps: &mut Vec<String>,
) {
    if declaration.kind == crate::account_design::DeclarationKind::Mechanism {
        let implementations = implementation_records(model, &declaration.id);
        if implementations.is_empty()
            || implementations.iter().any(|value| {
                value.get("source") == Some(&Json::Null) || text(value, "fingerprint").is_err()
            })
        {
            gaps.push(format!(
                "mechanism {} has incomplete implementation support",
                declaration.id
            ));
        }
        let definition = Json::obj(vec![
            ("id", Json::str(&declaration.id)),
            ("body", Json::str(&declaration.body)),
            (
                "cases",
                Json::Arr(declaration.cases.iter().map(Json::str).collect()),
            ),
            ("implementations", Json::Arr(implementations)),
        ]);
        values.push(Json::obj(vec![
            ("id", Json::str(&declaration.id)),
            (
                "fingerprint",
                Json::str(crate::fingerprint::canonical_sha256(&definition)),
            ),
        ]));
    }
    for child in &declaration.children {
        collect_mechanisms(model, child, values, gaps);
    }
}

fn object(value: &Json, allowed: &[&str]) -> Result<(), String> {
    let Json::Obj(fields) = value else {
        return Err("expected object".into());
    };
    let mut seen = BTreeSet::new();
    for (name, _) in fields {
        if !allowed.contains(&name.as_str()) || !seen.insert(name) {
            return Err(format!("unknown or duplicate field {name}"));
        }
    }
    Ok(())
}
fn text<'a>(value: &'a Json, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Json::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{field} requires nonempty string"))
}
fn fingerprint(value: &Json, field: &str) -> Result<(), String> {
    let value = text(value, field)?;
    if !value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    }) {
        return Err(format!("{field} requires sha256 fingerprint"));
    }
    Ok(())
}
fn id(value: &Json, field: &str) -> Result<(), String> {
    crate::diag::validate_id(text(value, field)?, false)
}
fn array<'a>(value: &'a Json, field: &str) -> Result<&'a [Json], String> {
    value
        .get(field)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{field} requires array"))
}
fn string_array(value: &Json, field: &str) -> Result<(), String> {
    if array(value, field)?
        .iter()
        .any(|value| value.as_str().is_none_or(|value| value.trim().is_empty()))
    {
        return Err(format!("{field} requires nonempty string entries"));
    }
    Ok(())
}
fn timestamp(value: &Json) -> Result<u64, String> {
    let value = value
        .get("reviewed_at_ms")
        .and_then(Json::as_num)
        .ok_or("reviewed_at_ms requires integer")?;
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > 9007199254740991.0 {
        return Err("reviewed_at_ms requires safe nonnegative integer".into());
    }
    Ok(value as u64)
}

pub fn validate_authority(value: &Json) -> Result<(), String> {
    object(
        value,
        &[
            "project",
            "model_fingerprint",
            "fingerprint",
            "checks",
            "cases",
            "bindings",
            "claims",
        ],
    )?;
    id(value, "project")?;
    fingerprint(value, "model_fingerprint")?;
    fingerprint(value, "fingerprint")?;
    if text(value, "fingerprint")? != record_fingerprint(value)? {
        return Err("model authority fingerprint differs from exact projection".into());
    }
    let mut checks = BTreeSet::new();
    for check in array(value, "checks")? {
        object(check, &["id", "fingerprint"])?;
        id(check, "id")?;
        fingerprint(check, "fingerprint")?;
        if !checks.insert(text(check, "id")?) {
            return Err("duplicate authority Check".into());
        }
    }
    let mut claims = BTreeSet::new();
    for claim in array(value, "claims")? {
        object(
            claim,
            &[
                "id",
                "criticality",
                "fingerprint",
                "cases",
                "mechanisms",
                "bindings",
                "gaps",
            ],
        )?;
        id(claim, "id")?;
        fingerprint(claim, "fingerprint")?;
        if !matches!(
            text(claim, "criticality")?,
            "routine" | "standard" | "critical" | "unspecified"
        ) {
            return Err("invalid Claim criticality".into());
        }
        if !claims.insert(text(claim, "id")?) {
            return Err("duplicate authority Claim".into());
        }
        string_array(claim, "gaps")?;
        let mut cases = BTreeSet::new();
        for case in array(claim, "cases")? {
            let case = case.as_str().ok_or("Case dependency requires ID")?;
            crate::diag::validate_id(case, false)?;
            if !cases.insert(case) {
                return Err("duplicate Claim Case dependency".into());
            }
        }
        let mut mechanisms = BTreeSet::new();
        for mechanism in array(claim, "mechanisms")? {
            object(mechanism, &["id", "fingerprint"])?;
            id(mechanism, "id")?;
            fingerprint(mechanism, "fingerprint")?;
            if !mechanisms.insert(text(mechanism, "id")?) {
                return Err("duplicate mechanism dependency".into());
            }
        }
        let mut bindings = BTreeSet::new();
        for binding in array(claim, "bindings")? {
            object(binding, &["check", "case"])?;
            id(binding, "check")?;
            id(binding, "case")?;
            if !bindings.insert((text(binding, "check")?, text(binding, "case")?)) {
                return Err("duplicate Claim binding dependency".into());
            }
        }
    }
    let mut cases = BTreeMap::new();
    for case in array(value, "cases")? {
        object(case, &["id", "claim", "fingerprint"])?;
        id(case, "id")?;
        id(case, "claim")?;
        fingerprint(case, "fingerprint")?;
        if !claims.contains(text(case, "claim")?)
            || cases
                .insert(text(case, "id")?, text(case, "claim")?)
                .is_some()
        {
            return Err("unknown Claim or duplicate Case authority".into());
        }
    }
    let mut pairs = BTreeSet::new();
    for binding in array(value, "bindings")? {
        object(
            binding,
            &[
                "check",
                "case",
                "check_fingerprint",
                "case_fingerprint",
                "contribution",
                "fingerprint",
            ],
        )?;
        id(binding, "check")?;
        id(binding, "case")?;
        for name in ["check_fingerprint", "case_fingerprint", "fingerprint"] {
            fingerprint(binding, name)?;
        }
        text(binding, "contribution")?;
        let pair = (text(binding, "check")?, text(binding, "case")?);
        if !checks.contains(pair.0) || !cases.contains_key(pair.1) || !pairs.insert(pair) {
            return Err("unknown binding target or duplicate pair".into());
        }
        let check = array(value, "checks")?
            .iter()
            .find(|check| text(check, "id").ok() == Some(pair.0))
            .unwrap();
        let case = array(value, "cases")?
            .iter()
            .find(|case| text(case, "id").ok() == Some(pair.1))
            .unwrap();
        if text(binding, "check_fingerprint")? != text(check, "fingerprint")?
            || text(binding, "case_fingerprint")? != text(case, "fingerprint")?
        {
            return Err("binding target fingerprint mismatch".into());
        }
        let Json::Obj(fields) = binding else {
            unreachable!()
        };
        let definition = Json::Obj(
            fields
                .iter()
                .filter(|(name, _)| name != "fingerprint")
                .cloned()
                .collect(),
        );
        if text(binding, "fingerprint")? != crate::fingerprint::canonical_sha256(&definition) {
            return Err("binding fingerprint mismatch".into());
        }
    }
    for claim in array(value, "claims")? {
        let id = text(claim, "id")?;
        let owned = cases
            .iter()
            .filter(|(_, owner)| **owner == id)
            .map(|(case, _)| *case)
            .collect::<BTreeSet<_>>();
        let declared = array(claim, "cases")?
            .iter()
            .filter_map(Json::as_str)
            .collect::<BTreeSet<_>>();
        if owned != declared {
            return Err("Claim Case dependency inventory mismatch".into());
        }
        let expected = pairs
            .iter()
            .filter(|(_, case)| cases.get(case) == Some(&id))
            .copied()
            .collect::<BTreeSet<_>>();
        let declared = array(claim, "bindings")?
            .iter()
            .map(|binding| {
                (
                    text(binding, "check").unwrap(),
                    text(binding, "case").unwrap(),
                )
            })
            .collect::<BTreeSet<_>>();
        if expected != declared {
            return Err("Claim binding dependency inventory mismatch".into());
        }
    }
    Ok(())
}

pub fn record_fingerprint(value: &Json) -> Result<String, String> {
    let Json::Obj(fields) = value else {
        return Err("review record requires object".into());
    };
    crate::run::canonical_fingerprint(&Json::Obj(
        fields
            .iter()
            .filter(|(name, _)| name != "fingerprint")
            .cloned()
            .collect(),
    ))
}

pub fn validate_record(value: &Json) -> Result<(), String> {
    let kind = text(value, "kind")?;
    let mut fields = vec![
        "kind",
        "reviewer",
        "reviewed_at_ms",
        "rationale",
        "artifacts",
        "fingerprint",
    ];
    fields.extend(match kind {
        "method-qualification" => vec!["check", "check_fingerprint", "decision"],
        "applicability-decision" => vec![
            "check",
            "case",
            "check_fingerprint",
            "case_fingerprint",
            "binding_fingerprint",
            "subject_fingerprint",
            "method_qualification_fingerprint",
            "basis",
            "decision",
            "limitations",
        ],
        "claim-judgment" => vec![
            "claim",
            "claim_fingerprint",
            "subject_fingerprint",
            "cases",
            "mechanisms",
            "bindings",
            "conclusion",
            "residual_risks",
        ],
        _ => return Err("unknown review kind".into()),
    });
    object(value, &fields)?;
    text(value, "reviewer")?;
    text(value, "rationale")?;
    timestamp(value)?;
    fingerprint(value, "fingerprint")?;
    let artifacts = array(value, "artifacts")?;
    if artifacts.is_empty() {
        return Err("review requires accountable artifacts".into());
    }
    let mut ids = BTreeSet::new();
    for artifact in artifacts {
        object(artifact, &["id", "digest", "locator"])?;
        text(artifact, "id")?;
        fingerprint(artifact, "digest")?;
        text(artifact, "locator")?;
        if !ids.insert(text(artifact, "id")?) {
            return Err("duplicate review artifact".into());
        }
    }
    match kind {
        "method-qualification" => {
            id(value, "check")?;
            fingerprint(value, "check_fingerprint")?;
            decision(value)?;
        }
        "applicability-decision" => {
            id(value, "check")?;
            id(value, "case")?;
            for name in [
                "check_fingerprint",
                "case_fingerprint",
                "binding_fingerprint",
                "subject_fingerprint",
                "method_qualification_fingerprint",
            ] {
                fingerprint(value, name)?;
            }
            decision(value)?;
            string_array(value, "limitations")?;
            let basis = array(value, "basis")?;
            if basis.is_empty() {
                return Err("Applicability Decision requires Observation basis".into());
            }
            let mut seen = BTreeSet::new();
            for item in basis {
                object(item, &["run", "bundle_fingerprint", "observation"])?;
                for field in ["run", "bundle_fingerprint", "observation"] {
                    fingerprint(item, field)?;
                }
                let key = (
                    text(item, "run")?,
                    text(item, "bundle_fingerprint")?,
                    text(item, "observation")?,
                );
                if !seen.insert(key) {
                    return Err("duplicate Observation basis".into());
                }
            }
        }
        "claim-judgment" => {
            id(value, "claim")?;
            fingerprint(value, "claim_fingerprint")?;
            fingerprint(value, "subject_fingerprint")?;
            if !matches!(
                text(value, "conclusion")?,
                "supported" | "violated" | "unresolved"
            ) {
                return Err("unknown Claim conclusion".into());
            }
            string_array(value, "residual_risks")?;
            for field in ["cases", "mechanisms"] {
                let mut seen = BTreeSet::new();
                for item in array(value, field)? {
                    object(item, &["id", "fingerprint"])?;
                    id(item, "id")?;
                    fingerprint(item, "fingerprint")?;
                    if !seen.insert(text(item, "id")?) {
                        return Err(format!("duplicate {field} dependency"));
                    }
                }
            }
            let mut pairs = BTreeSet::new();
            for item in array(value, "bindings")? {
                object(
                    item,
                    &["check", "case", "fingerprint", "applicability_fingerprint"],
                )?;
                id(item, "check")?;
                id(item, "case")?;
                fingerprint(item, "fingerprint")?;
                if !pairs.insert((text(item, "check")?, text(item, "case")?)) {
                    return Err("duplicate judgment binding".into());
                }
                match item.get("applicability_fingerprint") {
                    Some(Json::Null) => {}
                    Some(_) => fingerprint(item, "applicability_fingerprint")?,
                    None => {
                        return Err(
                            "binding requires explicit applicability fingerprint or null".into(),
                        )
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    if text(value, "fingerprint")? != record_fingerprint(value)? {
        return Err("review fingerprint differs from exact record".into());
    }
    Ok(())
}
fn decision(value: &Json) -> Result<(), String> {
    if matches!(text(value, "decision")?, "accepted" | "rejected") {
        Ok(())
    } else {
        Err("decision requires accepted or rejected".into())
    }
}

pub fn review_target(value: &Json) -> Result<Json, String> {
    let kind = text(value, "kind")?;
    let mut result = vec![("kind", Json::str(kind))];
    match kind {
        "method-qualification" => result.push(("check", Json::str(text(value, "check")?))),
        "applicability-decision" => {
            result.push(("check", Json::str(text(value, "check")?)));
            result.push(("case", Json::str(text(value, "case")?)));
            result.push((
                "subject_fingerprint",
                Json::str(text(value, "subject_fingerprint")?),
            ));
        }
        "claim-judgment" => {
            result.push(("claim", Json::str(text(value, "claim")?)));
            result.push((
                "subject_fingerprint",
                Json::str(text(value, "subject_fingerprint")?),
            ));
        }
        _ => return Err("unknown review target".into()),
    };
    Ok(Json::obj(result))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    pub status: String,
    pub diagnostics: Vec<String>,
}
fn assessed(status: &str, message: impl Into<String>) -> Assessment {
    let message = message.into();
    Assessment {
        status: status.into(),
        diagnostics: if message.is_empty() {
            Vec::new()
        } else {
            vec![message]
        },
    }
}

pub fn latest_review<'a>(records: &'a [Json], target: &Json) -> Result<Option<&'a Json>, String> {
    let target = review_target(target)?;
    let found = records
        .iter()
        .filter(|record| review_target(record).ok().as_ref() == Some(&target))
        .collect::<Vec<_>>();
    let latest = found
        .iter()
        .filter_map(|record| timestamp(record).ok())
        .max();
    let Some(time) = latest else {
        return Ok(None);
    };
    let mut current = found
        .into_iter()
        .filter(|record| timestamp(record).ok() == Some(time));
    let value = current.next().unwrap();
    if current.any(|record| record.get("fingerprint") != value.get("fingerprint")) {
        return Err("ambiguous reviews at latest review time".into());
    }
    Ok(Some(value))
}

fn authority_item<'a>(
    authority: &'a Json,
    collection: &str,
    field: &str,
    id: &str,
) -> Option<&'a Json> {
    authority
        .get(collection)?
        .as_array()?
        .iter()
        .find(|value| value.get(field).and_then(Json::as_str) == Some(id))
}
fn authority_binding<'a>(authority: &'a Json, check: &str, case: &str) -> Option<&'a Json> {
    authority.get("bindings")?.as_array()?.iter().find(|value| {
        value.get("check").and_then(Json::as_str) == Some(check)
            && value.get("case").and_then(Json::as_str) == Some(case)
    })
}
fn active_heads(bundles: &[RunBundle]) -> Result<Vec<&RunBundle>, String> {
    for bundle in bundles {
        let findings = crate::run::verify(bundle);
        if !findings.is_empty() {
            return Err(format!(
                "Run head is protocol-invalid: {}",
                findings[0].detail
            ));
        }
    }
    let mut heads: BTreeMap<&str, &RunBundle> = BTreeMap::new();
    for bundle in bundles {
        match heads.get(bundle.run_id.as_str()) {
            Some(previous) if previous.bundle_revision > bundle.bundle_revision => {}
            Some(previous)
                if previous.bundle_revision == bundle.bundle_revision
                    && previous.bundle_fingerprint != bundle.bundle_fingerprint =>
            {
                return Err("ambiguous corrected Run head".into())
            }
            _ => {
                heads.insert(&bundle.run_id, bundle);
            }
        }
    }
    Ok(heads.into_values().collect())
}

pub fn assess_record(
    record: &Json,
    authority: &Json,
    reviews: &[Json],
    bundles: &[RunBundle],
) -> Assessment {
    if let Err(error) = validate_authority(authority) {
        return assessed("invalid", format!("invalid model authority: {error}"));
    }
    if let Err(error) = validate_record(record) {
        return assessed("invalid", error);
    }
    let kind = text(record, "kind").unwrap();
    if kind == "method-qualification" {
        let Some(check) = authority_item(authority, "checks", "id", text(record, "check").unwrap())
        else {
            return assessed("stale", "Check is absent from current authority");
        };
        if record.get("check_fingerprint") != check.get("fingerprint") {
            return assessed("stale", "Check definition or implementation changed");
        }
        return assessed(
            if text(record, "decision").unwrap() == "accepted" {
                "accepted"
            } else {
                "rejected"
            },
            "",
        );
    }
    if kind == "applicability-decision" {
        let check = text(record, "check").unwrap();
        let case = text(record, "case").unwrap();
        let Some(binding) = authority_binding(authority, check, case) else {
            return assessed(
                "stale",
                "Check–Case binding is absent from current authority",
            );
        };
        for (field, target) in [
            ("check_fingerprint", "check_fingerprint"),
            ("case_fingerprint", "case_fingerprint"),
            ("binding_fingerprint", "fingerprint"),
        ] {
            if record.get(field) != binding.get(target) {
                return assessed("stale", format!("{field} changed"));
            }
        }
        let target = Json::obj(vec![
            ("kind", Json::str("method-qualification")),
            ("check", Json::str(check)),
        ]);
        let qualification = match latest_review(reviews, &target) {
            Ok(Some(value)) => value,
            Ok(None) => return assessed("missing", "Method Qualification is absent"),
            Err(error) => return assessed("invalid", error),
        };
        if record.get("method_qualification_fingerprint") != qualification.get("fingerprint") {
            return assessed("stale", "Method Qualification was superseded");
        }
        let qualification_state = assess_record(qualification, authority, reviews, bundles);
        if qualification_state.status != "accepted" {
            return assessed(
                &qualification_state.status,
                "Method Qualification is not currently accepted",
            );
        }
        let heads = match active_heads(bundles) {
            Ok(heads) => heads,
            Err(error) => return assessed("invalid", error),
        };
        let subject = text(record, "subject_fingerprint").unwrap();
        let check_fp = text(record, "check_fingerprint").unwrap();
        let basis = array(record, "basis").unwrap();
        let mut observed = BTreeSet::new();
        let mut latest = 0;
        for bundle in &heads {
            if bundle.subject_fingerprint == subject {
                for execution in &bundle.check_executions {
                    if execution.check.id == check && execution.check.fingerprint == check_fp {
                        for observation in &execution.observations {
                            if observation.case == case {
                                latest = latest.max(observation.observed_at_ms);
                                observed.insert((
                                    bundle.run_id.as_str(),
                                    bundle.bundle_fingerprint.as_str(),
                                    observation.fingerprint.as_str(),
                                    observation.observed_at_ms,
                                ));
                            }
                        }
                    }
                }
            }
        }
        if observed.is_empty() {
            return assessed("missing", "No current exact-Subject Observation exists");
        }
        for item in basis {
            let run = text(item, "run").unwrap();
            let bundle = text(item, "bundle_fingerprint").unwrap();
            let observation = text(item, "observation").unwrap();
            if !observed
                .iter()
                .any(|value| value.0 == run && value.1 == bundle && value.2 == observation)
            {
                return assessed(
                    "stale",
                    "Observation basis is missing, corrected or from another Check/Case/Subject",
                );
            }
        }
        for (run, bundle, observation, time) in observed {
            if time == latest
                && !basis.iter().any(|value| {
                    text(value, "run").ok() == Some(run)
                        && text(value, "bundle_fingerprint").ok() == Some(bundle)
                        && text(value, "observation").ok() == Some(observation)
                })
            {
                return assessed(
                    "stale",
                    "Applicability basis omits a latest active Observation",
                );
            }
        }
        return assessed(
            if text(record, "decision").unwrap() == "accepted" {
                "accepted"
            } else {
                "rejected"
            },
            "",
        );
    }
    let claim_id = text(record, "claim").unwrap();
    let Some(claim) = authority_item(authority, "claims", "id", claim_id) else {
        return assessed("stale", "Claim is absent from current authority");
    };
    if text(claim, "criticality").ok() == Some("routine") {
        return assessed("invalid", "Routine Claims do not enroll Claim Judgments");
    }
    if record.get("claim_fingerprint") != claim.get("fingerprint") {
        return assessed(
            "stale",
            "Claim account or supporting implementation changed",
        );
    }
    let case_ids = array(claim, "cases")
        .unwrap()
        .iter()
        .filter_map(Json::as_str)
        .collect::<BTreeSet<_>>();
    let mut reviewed = BTreeSet::new();
    for case in array(record, "cases").unwrap() {
        let id = text(case, "id").unwrap();
        let Some(current) = authority_item(authority, "cases", "id", id) else {
            return assessed("stale", "Judgment includes absent Case");
        };
        if case.get("fingerprint") != current.get("fingerprint") {
            return assessed("stale", "Case contract changed");
        }
        reviewed.insert(id);
    }
    if reviewed != case_ids {
        return assessed("stale", "Judgment Case inventory is incomplete or obsolete");
    }
    let mechanisms = array(claim, "mechanisms").unwrap();
    let declared = array(record, "mechanisms").unwrap();
    if mechanisms.len() != declared.len()
        || mechanisms.iter().any(|current| {
            !declared.iter().any(|value| {
                text(value, "id").ok() == text(current, "id").ok()
                    && text(value, "fingerprint").ok() == text(current, "fingerprint").ok()
            })
        })
    {
        return assessed("stale", "Judgment mechanism dependencies changed");
    }
    let expected = array(claim, "bindings").unwrap();
    let declared = array(record, "bindings").unwrap();
    if expected.len() != declared.len()
        || expected.iter().any(|binding| {
            !declared.iter().any(|value| {
                value.get("check") == binding.get("check")
                    && value.get("case") == binding.get("case")
            })
        })
    {
        return assessed(
            "stale",
            "Judgment binding inventory is incomplete or obsolete",
        );
    }
    let conclusion = text(record, "conclusion").unwrap();
    let subject = text(record, "subject_fingerprint").unwrap();
    let mut satisfied = false;
    let mut violated = false;
    let mut unresolved = false;
    for binding in declared {
        let check = text(binding, "check").unwrap();
        let case = text(binding, "case").unwrap();
        let Some(current) = authority_binding(authority, check, case) else {
            return assessed("stale", "Judgment binding is absent");
        };
        if binding.get("fingerprint") != current.get("fingerprint") {
            return assessed("stale", "Judgment binding definition changed");
        }
        let target = Json::obj(vec![
            ("kind", Json::str("applicability-decision")),
            ("check", Json::str(check)),
            ("case", Json::str(case)),
            ("subject_fingerprint", Json::str(subject)),
        ]);
        let application = match latest_review(reviews, &target) {
            Ok(Some(value)) => value,
            Ok(None) => {
                if binding.get("applicability_fingerprint") != Some(&Json::Null) {
                    return assessed("missing", "Pinned Applicability Decision is absent");
                }
                unresolved = true;
                continue;
            }
            Err(error) => return assessed("invalid", error),
        };
        if binding.get("applicability_fingerprint") != application.get("fingerprint") {
            return assessed("stale", "Judgment Applicability Decision was superseded");
        }
        let state = assess_record(application, authority, reviews, bundles);
        if state.status != "accepted" {
            if matches!(state.status.as_str(), "stale" | "invalid") {
                return assessed(
                    &state.status,
                    "Judgment applicability dependencies are no longer current",
                );
            }
            unresolved = true;
            if conclusion != "unresolved" {
                return assessed(
                    &state.status,
                    "Judgment lacks current accepted applicability",
                );
            }
            continue;
        }
        let outcomes = bound_outcomes(application, bundles);
        if outcomes.is_empty() {
            unresolved = true;
        }
        for outcome in outcomes {
            match outcome {
                ObservationOutcome::Satisfied => satisfied = true,
                ObservationOutcome::Violated => violated = true,
                ObservationOutcome::Inconclusive => unresolved = true,
            }
        }
    }
    let gaps = !array(claim, "gaps").unwrap().is_empty()
        || text(claim, "criticality").ok() == Some("unspecified");
    match conclusion {
        "supported" if gaps || unresolved || violated || !satisfied => assessed(
            "invalid",
            "Supported judgment has structural gaps, incomplete evidence or counterevidence",
        ),
        "violated" if !violated => assessed(
            "invalid",
            "Violated judgment lacks applicable violated Observation",
        ),
        _ => assessed("accepted", ""),
    }
}
fn bound_outcomes(record: &Json, bundles: &[RunBundle]) -> Vec<ObservationOutcome> {
    let mut result = Vec::new();
    for basis in array(record, "basis").unwrap_or_default() {
        for bundle in bundles {
            if text(basis, "run").ok() == Some(bundle.run_id.as_str())
                && text(basis, "bundle_fingerprint").ok()
                    == Some(bundle.bundle_fingerprint.as_str())
            {
                for execution in &bundle.check_executions {
                    if text(record, "check").ok() == Some(execution.check.id.as_str()) {
                        for observation in &execution.observations {
                            if text(basis, "observation").ok()
                                == Some(observation.fingerprint.as_str())
                            {
                                result.push(observation.outcome);
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn subject_state(
    authority: &Json,
    reviews: &[Json],
    bundles: &[RunBundle],
    subject_fingerprint: &str,
) -> Json {
    if let Err(error) = validate_authority(authority) {
        return Json::obj(vec![
            ("status", Json::str("invalid")),
            ("diagnostics", Json::Arr(vec![Json::str(error)])),
        ]);
    }
    let mut qualifications = Vec::new();
    for check in array(authority, "checks").unwrap() {
        let target = Json::obj(vec![
            ("kind", Json::str("method-qualification")),
            ("check", check.get("id").cloned().unwrap()),
        ]);
        let assessment = match latest_review(reviews, &target) {
            Ok(Some(record)) => assess_record(record, authority, reviews, bundles),
            Ok(None) => assessed("missing", "Method Qualification is absent"),
            Err(error) => assessed("invalid", error),
        };
        qualifications.push(Json::obj(vec![
            ("check", check.get("id").cloned().unwrap()),
            (
                "check_fingerprint",
                check.get("fingerprint").cloned().unwrap(),
            ),
            ("status", Json::str(assessment.status)),
            (
                "diagnostics",
                Json::Arr(assessment.diagnostics.iter().map(Json::str).collect()),
            ),
        ]));
    }
    let mut claims = Vec::new();
    for claim in array(authority, "claims").unwrap() {
        let id = text(claim, "id").unwrap();
        let mut binding_states = Vec::new();
        for binding in array(claim, "bindings").unwrap() {
            let target = Json::obj(vec![
                ("kind", Json::str("applicability-decision")),
                ("check", binding.get("check").cloned().unwrap()),
                ("case", binding.get("case").cloned().unwrap()),
                ("subject_fingerprint", Json::str(subject_fingerprint)),
            ]);
            let assessment = match latest_review(reviews, &target) {
                Ok(Some(record)) => assess_record(record, authority, reviews, bundles),
                Ok(None) => assessed("missing", "Applicability Decision is absent"),
                Err(error) => assessed("invalid", error),
            };
            binding_states.push(Json::obj(vec![
                ("check", binding.get("check").cloned().unwrap()),
                ("case", binding.get("case").cloned().unwrap()),
                ("status", Json::str(assessment.status)),
                (
                    "diagnostics",
                    Json::Arr(assessment.diagnostics.iter().map(Json::str).collect()),
                ),
            ]));
        }
        let target = Json::obj(vec![
            ("kind", Json::str("claim-judgment")),
            ("claim", Json::str(id)),
            ("subject_fingerprint", Json::str(subject_fingerprint)),
        ]);
        let (state, assessment) = if text(claim, "criticality").ok() == Some("routine") {
            ("review-not-required".into(), assessed("not-required", ""))
        } else {
            match latest_review(reviews, &target) {
                Ok(Some(record)) => {
                    let assessment = assess_record(record, authority, reviews, bundles);
                    let state = if assessment.status == "accepted" {
                        text(record, "conclusion")
                            .unwrap_or("unresolved")
                            .to_string()
                    } else {
                        "unresolved".into()
                    };
                    (state, assessment)
                }
                Ok(None) => (
                    "unresolved".into(),
                    assessed("missing", "Claim Judgment is absent"),
                ),
                Err(error) => ("unresolved".into(), assessed("invalid", error)),
            }
        };
        claims.push(Json::obj(vec![
            ("claim", Json::str(id)),
            ("status", Json::str(state)),
            ("judgment", Json::str(assessment.status)),
            (
                "diagnostics",
                Json::Arr(assessment.diagnostics.iter().map(Json::str).collect()),
            ),
            ("gaps", claim.get("gaps").cloned().unwrap()),
            ("bindings", Json::Arr(binding_states)),
        ]));
    }
    Json::obj(vec![
        ("project", authority.get("project").cloned().unwrap()),
        (
            "authority_fingerprint",
            authority.get("fingerprint").cloned().unwrap(),
        ),
        ("method_qualifications", Json::Arr(qualifications)),
        (
            "model_fingerprint",
            authority.get("model_fingerprint").cloned().unwrap(),
        ),
        ("subject_fingerprint", Json::str(subject_fingerprint)),
        ("claims", Json::Arr(claims)),
    ])
}

pub fn review_input(
    model: &Model,
    project: &str,
    request: &Json,
    reviews: &[Json],
    bundles: &[RunBundle],
) -> Result<Json, String> {
    let authority = authority(model, project)?;
    let kind = text(request, "kind")?;
    match kind {
        "method-qualification" => object(request, &["kind", "check"]),
        "applicability-decision" => object(request, &["kind", "check", "case", "subject_fingerprint"]),
        "claim-judgment" => object(request, &["kind", "claim", "subject_fingerprint"]),
        _ => return Err("review input kind requires method-qualification, applicability-decision or claim-judgment".into()),
    }?;
    for name in ["check", "case", "claim"] {
        if request.get(name).is_some() {
            crate::diag::validate_id(text(request, name)?, false)?;
        }
    }
    let subject = if kind == "method-qualification" {
        None
    } else {
        fingerprint(request, "subject_fingerprint")?;
        Some(text(request, "subject_fingerprint")?)
    };
    for review in reviews {
        validate_record(review)?;
    }
    let findings = crate::run::verify_set(bundles);
    if let Some(finding) = findings.first() {
        return Err(format!(
            "review input Run history is invalid: {}",
            finding.detail
        ));
    }
    let heads = active_heads(bundles)?;
    let mut check_ids = BTreeSet::new();
    let mut case_ids = BTreeSet::new();
    let mut claim_ids = BTreeSet::new();
    let mut bindings = Vec::new();
    if kind == "claim-judgment" {
        let id = text(request, "claim")?;
        let claim = authority_item(&authority, "claims", "id", id)
            .ok_or_else(|| format!("unknown Claim {id}"))?;
        if text(claim, "criticality")? == "routine" {
            return Err("routine Claims do not require independent Claim Judgment".into());
        }
        claim_ids.insert(id.to_string());
        for case in array(claim, "cases")? {
            case_ids.insert(case.as_str().unwrap().to_string());
        }
        for pair in array(claim, "bindings")? {
            let check = text(pair, "check")?;
            let case = text(pair, "case")?;
            check_ids.insert(check.to_string());
            bindings.push(authority_binding(&authority, check, case).unwrap().clone());
        }
    } else {
        let check = text(request, "check")?;
        authority_item(&authority, "checks", "id", check)
            .ok_or_else(|| format!("unknown Check {check}"))?;
        check_ids.insert(check.to_string());
        if kind == "applicability-decision" {
            let case = text(request, "case")?;
            let binding = authority_binding(&authority, check, case)
                .ok_or_else(|| format!("unknown Check–Case binding {check}/{case}"))?;
            case_ids.insert(case.to_string());
            bindings.push(binding.clone());
        }
    }
    let selected = |field: &str, ids: &BTreeSet<String>| -> Json {
        Json::Arr(
            array(&authority, field)
                .unwrap()
                .iter()
                .filter(|value| ids.contains(text(value, "id").unwrap()))
                .cloned()
                .collect(),
        )
    };
    let mut gaps = Vec::new();
    let mut review_states = Vec::new();
    let mut existing_reviews = Vec::new();
    let mut targets = check_ids
        .iter()
        .map(|check| {
            Json::obj(vec![
                ("kind", Json::str("method-qualification")),
                ("check", Json::str(check)),
            ])
        })
        .collect::<Vec<_>>();
    if let Some(subject) = subject {
        for binding in &bindings {
            targets.push(Json::obj(vec![
                ("kind", Json::str("applicability-decision")),
                ("check", binding.get("check").unwrap().clone()),
                ("case", binding.get("case").unwrap().clone()),
                ("subject_fingerprint", Json::str(subject)),
            ]));
        }
        if kind == "claim-judgment" {
            targets.push(request.clone());
        }
    }
    for target in targets {
        let (record, state) = match latest_review(reviews, &target) {
            Ok(Some(record)) => (
                Some(record),
                assess_record(record, &authority, reviews, bundles),
            ),
            Ok(None) => (None, assessed("missing", "Independent review is absent")),
            Err(error) => (None, assessed("invalid", error)),
        };
        if let Some(record) = record {
            existing_reviews.push(record.clone());
        }
        if state.status != "accepted" {
            gaps.push(Json::obj(vec![
                ("target", target.clone()),
                ("status", Json::str(&state.status)),
                (
                    "diagnostics",
                    Json::Arr(state.diagnostics.iter().map(Json::str).collect()),
                ),
            ]));
        }
        review_states.push(Json::obj(vec![
            ("target", target),
            ("record_status", Json::str(&state.status)),
            (
                "record_fingerprint",
                record
                    .and_then(|v| v.get("fingerprint"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "diagnostics",
                Json::Arr(state.diagnostics.iter().map(Json::str).collect()),
            ),
        ]));
    }
    let mut observations = Vec::new();
    let mut evidence_bundles = Vec::new();
    let mut subjects = Vec::new();
    if let Some(subject) = subject {
        for bundle in heads {
            if bundle.subject_fingerprint != subject {
                continue;
            }
            let mut relevant = false;
            for execution in &bundle.check_executions {
                if !check_ids.contains(&execution.check.id) {
                    continue;
                }
                let check =
                    authority_item(&authority, "checks", "id", &execution.check.id).unwrap();
                if text(check, "fingerprint")? != execution.check.fingerprint {
                    continue;
                }
                for observation in &execution.observations {
                    if !case_ids.contains(&observation.case)
                        || !bindings.iter().any(|binding| {
                            text(binding, "check").ok() == Some(execution.check.id.as_str())
                                && text(binding, "case").ok() == Some(observation.case.as_str())
                        })
                    {
                        continue;
                    }
                    relevant = true;
                    observations.push(Json::obj(vec![
                        ("check", Json::str(&execution.check.id)),
                        ("case", Json::str(&observation.case)),
                        ("run", Json::str(&bundle.run_id)),
                        ("bundle_fingerprint", Json::str(&bundle.bundle_fingerprint)),
                        ("observation", Json::str(&observation.fingerprint)),
                        ("outcome", Json::str(observation.outcome.name())),
                        (
                            "observed_at_ms",
                            Json::Num(observation.observed_at_ms as f64),
                        ),
                    ]));
                }
            }
            if relevant {
                let descriptor = crate::run::subject_to_json(&bundle.subject);
                if !subjects.contains(&descriptor) {
                    subjects.push(descriptor);
                }
                evidence_bundles.push(crate::run::to_json(bundle));
            }
        }
        for binding in &bindings {
            if !observations.iter().any(|observation| {
                observation.get("check") == binding.get("check")
                    && observation.get("case") == binding.get("case")
            }) {
                gaps.push(Json::obj(vec![
                    ("check", binding.get("check").unwrap().clone()),
                    ("case", binding.get("case").unwrap().clone()),
                    ("status", Json::str("missing")),
                    (
                        "diagnostics",
                        Json::Arr(vec![Json::str(
                            "No current exact-Subject Observation exists",
                        )]),
                    ),
                ]));
            }
        }
    }
    for claim in array(&authority, "claims")?
        .iter()
        .filter(|claim| claim_ids.contains(text(claim, "id").unwrap()))
    {
        for gap in array(claim, "gaps")? {
            gaps.push(Json::obj(vec![
                ("claim", claim.get("id").unwrap().clone()),
                ("status", Json::str("structural-gap")),
                ("diagnostics", Json::Arr(vec![gap.clone()])),
            ]));
        }
    }
    for check in &check_ids {
        if !model.check_implementations.iter().any(|site| {
            site.check == *check && site.source.is_some() && !site.source_fingerprint.is_empty()
        }) {
            gaps.push(Json::obj(vec![
                ("check", Json::str(check)),
                ("status", Json::str("structural-gap")),
                (
                    "diagnostics",
                    Json::Arr(vec![Json::str("Check has no stable source implementation")]),
                ),
            ]));
        }
    }
    let definitions = review_definitions(model, &check_ids, &case_ids, &claim_ids);
    let mut output = Json::obj(vec![
        ("format", Json::str("azimuth-review-input")),
        ("version", Json::Num(1.0)),
        ("project", Json::str(project)),
        ("target", request.clone()),
        (
            "authority_fingerprint",
            authority.get("fingerprint").unwrap().clone(),
        ),
        (
            "model_fingerprint",
            authority.get("model_fingerprint").unwrap().clone(),
        ),
        (
            "dependencies",
            Json::obj(vec![
                ("checks", selected("checks", &check_ids)),
                ("cases", selected("cases", &case_ids)),
                ("claims", selected("claims", &claim_ids)),
                ("bindings", Json::Arr(bindings)),
            ]),
        ),
        ("definitions", definitions),
        ("subjects", Json::Arr(subjects)),
        ("observations", Json::Arr(observations)),
        ("evidence_bundles", Json::Arr(evidence_bundles)),
        ("review_states", Json::Arr(review_states)),
        ("existing_reviews", Json::Arr(existing_reviews)),
        ("gaps", Json::Arr(gaps)),
    ]);
    let fingerprint = record_fingerprint(&output)?;
    if let Json::Obj(fields) = &mut output {
        fields.push(("fingerprint".into(), Json::str(fingerprint)));
    }
    Ok(output)
}

fn review_definitions(
    model: &Model,
    check_ids: &BTreeSet<String>,
    case_ids: &BTreeSet<String>,
    claim_ids: &BTreeSet<String>,
) -> Json {
    let checks = model
        .checks()
        .filter(|check| check_ids.contains(&check.id))
        .map(|check| {
            let authored = model
                .account_verifications
                .iter()
                .flat_map(|file| &file.checks)
                .find(|item| item.definition.id == check.id);
            Json::obj(vec![
                ("id", Json::str(&check.id)),
                (
                    "methods",
                    Json::Arr(check.methods.iter().map(Json::str).collect()),
                ),
                ("terminal", Json::str(&check.terminal)),
                ("authored_rationale", Json::str(&check.rationale)),
                (
                    "verification_context",
                    Json::Arr(
                        model
                            .account_verifications
                            .iter()
                            .filter(|file| {
                                file.checks
                                    .iter()
                                    .any(|item| item.definition.id == check.id)
                            })
                            .map(|file| {
                                Json::obj(vec![
                                    ("introduction", Json::str(&file.introduction)),
                                    (
                                        "sections",
                                        Json::Arr(
                                            file.sections
                                                .iter()
                                                .filter(|section| {
                                                    section.claim.is_none()
                                                        || authored.is_some_and(|check| {
                                                            section.claim.as_deref()
                                                                == Some(&check.claim)
                                                        })
                                                })
                                                .map(|section| Json::str(&section.prose))
                                                .collect(),
                                        ),
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
                (
                    "inputs",
                    authored.map_or(Json::Null, |item| {
                        crate::verification_packages::map_json(&item.inputs)
                    }),
                ),
                (
                    "selectors",
                    authored.map_or(Json::Null, |item| {
                        crate::verification_packages::map_json(&item.selectors)
                    }),
                ),
                (
                    "implementations",
                    Json::Arr(
                        model
                            .check_implementations
                            .iter()
                            .filter(|site| site.check == check.id)
                            .map(|site| {
                                Json::obj(vec![
                                    ("site", Json::str(&site.site)),
                                    (
                                        "source",
                                        site.source
                                            .as_ref()
                                            .map_or(Json::Null, |source| Json::str(source.key())),
                                    ),
                                    ("fingerprint", Json::str(&site.source_fingerprint)),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ])
        })
        .collect();
    let cases = model
        .cases()
        .filter(|view| case_ids.contains(&view.case.id))
        .map(|view| {
            Json::obj(vec![
                ("id", Json::str(&view.case.id)),
                ("claim", Json::str(&view.claim.id)),
                ("statement", Json::str(&view.case.statement)),
                ("claim_statement", Json::str(&view.claim.statement)),
                ("domain", Json::str(view.claim.domain.name())),
                (
                    "over",
                    view.claim.over.as_ref().map_or(Json::Null, Json::str),
                ),
                (
                    "terms",
                    Json::Arr(
                        view.spec
                            .terms
                            .iter()
                            .map(|term| {
                                Json::obj(vec![
                                    ("id", Json::str(&term.id)),
                                    ("definition", Json::str(&term.definition)),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ])
        })
        .collect();
    let claims = model
        .claims()
        .filter(|view| claim_ids.contains(&view.claim.id))
        .map(|view| {
            Json::obj(vec![
                ("id", Json::str(&view.claim.id)),
                ("statement", Json::str(&view.claim.statement)),
                (
                    "design",
                    model
                        .account_design_for_claim(&view.spec.id, &view.claim.id)
                        .map_or(Json::Null, |(file, declaration)| {
                            Json::obj(vec![
                                ("context", Json::str(&file.introduction)),
                                ("source", Json::str(&declaration.source)),
                            ])
                        }),
                ),
                (
                    "verification",
                    Json::Arr(
                        model
                            .account_verifications
                            .iter()
                            .filter(|file| file.owner == view.spec.id)
                            .map(|file| {
                                Json::obj(vec![
                                    ("context", Json::str(&file.introduction)),
                                    (
                                        "claim_prose",
                                        Json::Arr(
                                            file.claims
                                                .iter()
                                                .filter(|claim| claim.claim == view.claim.id)
                                                .map(|claim| Json::str(&claim.prose))
                                                .collect(),
                                        ),
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ])
        })
        .collect();
    let mechanism_ids = model
        .account_verifications
        .iter()
        .flat_map(|file| &file.checks)
        .filter(|check| check_ids.contains(&check.definition.id))
        .flat_map(|check| check.mechanisms.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut mechanisms = Vec::new();
    for file in &model.account_designs {
        review_mechanisms(
            model,
            &file.declarations,
            &file.introduction,
            &mechanism_ids,
            &mut mechanisms,
        );
    }
    let input_ids = model
        .account_verifications
        .iter()
        .flat_map(|file| &file.checks)
        .filter(|check| check_ids.contains(&check.definition.id))
        .flat_map(|check| check.inputs.values().cloned())
        .collect::<BTreeSet<_>>();
    let entities = model
        .account_verifications
        .iter()
        .flat_map(|file| &file.package_entities)
        .filter(|entity| input_ids.contains(&entity.id))
        .cloned()
        .collect::<Vec<_>>();
    let producers = model
        .package_producers
        .iter()
        .filter(|producer| input_ids.contains(&producer.entity))
        .map(|producer| producer.to_json())
        .collect();
    Json::obj(vec![
        ("checks", Json::Arr(checks)),
        ("cases", Json::Arr(cases)),
        ("claims", Json::Arr(claims)),
        (
            "package_entities",
            crate::verification_packages::declarations_json(&entities),
        ),
        ("producers", Json::Arr(producers)),
        ("mechanisms", Json::Arr(mechanisms)),
    ])
}

fn review_mechanisms(
    model: &Model,
    declarations: &[crate::account_design::Declaration],
    context: &str,
    selected: &BTreeSet<String>,
    output: &mut Vec<Json>,
) {
    for declaration in declarations {
        if declaration.kind == crate::account_design::DeclarationKind::Mechanism
            && selected.contains(&declaration.id)
        {
            output.push(Json::obj(vec![
                ("id", Json::str(&declaration.id)),
                ("context", Json::str(context)),
                ("source", Json::str(&declaration.source)),
                (
                    "implementations",
                    Json::Arr(implementation_records(model, &declaration.id)),
                ),
            ]));
        }
        review_mechanisms(model, &declaration.children, context, selected, output);
    }
}
