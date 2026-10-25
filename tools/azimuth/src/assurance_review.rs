use crate::json::Json;
use crate::model::Model;
use crate::run::{ChallengeTargetKind, CheckResultOutcome, RunBundle};
use std::collections::{BTreeMap, BTreeSet};

pub fn authority(model: &Model, project: &str) -> Result<Json, String> {
    crate::diag::validate_id(project, false)?;
    let checks = model
        .checks()
        .map(|check| {
            let mut fields = vec![
                ("id".into(), Json::str(&check.id)),
                (
                    "fingerprint".into(),
                    Json::str(model.check_execution_fingerprint(check)),
                ),
            ];
            if let Some(policy) = model
                .account_verifications
                .iter()
                .flat_map(|document| &document.checks)
                .find(|authored| authored.definition.id == check.id)
                .and_then(|authored| authored.inputs.get("deployment-policy"))
            {
                fields.push((
                    "deployment".into(),
                    crate::run::strict_json("Deployment", policy)
                        .expect("validated deployment policy"),
                ));
            }
            Json::Obj(fields)
        })
        .collect::<Vec<_>>();
    let scenarios = model
        .scenarios()
        .map(|scenario| {
            let mut fields = vec![
                ("id", Json::str(&scenario.scenario.id)),
                ("requirement", Json::str(&scenario.requirement.id)),
                (
                    "fingerprint",
                    Json::str(
                        model
                            .scenario_digest(&scenario.scenario.id)
                            .unwrap_or_default(),
                    ),
                ),
            ];
            if !scenario.scenario.conditions.is_empty() {
                fields.push((
                    "conditions",
                    Json::Arr(
                        scenario
                            .scenario
                            .conditions
                            .iter()
                            .map(crate::model::Condition::to_json)
                            .collect(),
                    ),
                ));
            }
            Json::obj(fields)
        })
        .collect::<Vec<_>>();
    let mut bindings = Vec::new();
    for document in &model.account_verifications {
        for check in &document.checks {
            for binding in &check.bindings {
                let definition = Json::obj(vec![
                    ("check", Json::str(&check.definition.id)),
                    ("scenario", Json::str(&binding.scenario)),
                    (
                        "check_fingerprint",
                        Json::str(model.check_execution_fingerprint(&check.definition)),
                    ),
                    (
                        "scenario_fingerprint",
                        Json::str(model.scenario_digest(&binding.scenario).unwrap_or_default()),
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
            ("scenario", Json::str(&binding.scenario)),
            (
                "check_fingerprint",
                Json::str(model.check_execution_fingerprint(check)),
            ),
            (
                "scenario_fingerprint",
                Json::str(model.scenario_digest(&binding.scenario).unwrap_or_default()),
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
    let mut requirements = Vec::new();
    let mut requirement_intents = Vec::new();
    for view in model.requirements() {
        let id = &view.requirement.id;
        let intent = model.requirement_digest(id).unwrap_or_default();
        requirement_intents.push(Json::obj(vec![
            ("id", Json::str(id)),
            ("fingerprint", Json::str(&intent)),
        ]));
        let mut gaps = Vec::new();
        let mut mechanisms = Vec::new();
        if let Some((_, declaration)) = model.account_design_for_requirement(&view.spec.id, id) {
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
        let scenario_ids = view
            .requirement
            .scenarios
            .iter()
            .map(|scenario| scenario.id.clone())
            .collect::<BTreeSet<_>>();
        let relevant = bindings
            .iter()
            .filter(|binding| {
                binding
                    .get("scenario")
                    .and_then(Json::as_str)
                    .is_some_and(|scenario| scenario_ids.contains(scenario))
            })
            .cloned()
            .collect::<Vec<_>>();
        let criticality = view
            .requirement
            .criticality
            .map(|value| value.name())
            .unwrap_or("unspecified");
        if criticality != "routine" {
            for scenario in &scenario_ids {
                if !relevant
                    .iter()
                    .any(|binding| binding.get("scenario").and_then(Json::as_str) == Some(scenario))
                {
                    gaps.push(format!("Scenario {scenario} has no Evidence Binding"));
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
                        site.requirement == *id
                            && site
                                .source
                                .as_ref()
                                .is_some_and(|source| source.area == area)
                            && !site.source_fingerprint.is_empty()
                    }) {
                        gaps.push(format!(
                            "Requirement has no stable realization in Area {area}"
                        ));
                    }
                }
            } else {
                gaps.push("Requirement has no design-owned realization obligation".into());
            }
        }
        gaps.sort();
        gaps.dedup();
        mechanisms.sort_by_key(|mechanism| text(mechanism, "id").unwrap_or_default().to_string());
        let dependencies = Json::obj(vec![
            ("intent", Json::str(intent)),
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
                "verification_challenges",
                requirement_verification_challenges(model, id),
            ),
            (
                "realizations",
                Json::Arr({
                    let mut sites = model
                        .realizes
                        .iter()
                        .filter(|site| site.requirement == *id)
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
        let mut requirement_fields = vec![
            ("id", Json::str(id)),
            ("criticality", Json::str(criticality)),
            (
                "fingerprint",
                Json::str(crate::fingerprint::canonical_sha256(&dependencies)),
            ),
            (
                "scenarios",
                Json::Arr(scenario_ids.iter().map(Json::str).collect()),
            ),
            ("mechanisms", Json::Arr(mechanisms)),
            (
                "bindings",
                Json::Arr(
                    relevant
                        .iter()
                        .map(|binding| {
                            Json::obj(vec![
                                ("check", binding.get("check").cloned().unwrap_or(Json::Null)),
                                (
                                    "scenario",
                                    binding.get("scenario").cloned().unwrap_or(Json::Null),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("gaps", Json::Arr(gaps.iter().map(Json::str).collect())),
            (
                "verification_challenges",
                requirement_verification_challenges(model, id),
            ),
        ];
        if !view.requirement.conditions.is_empty() {
            requirement_fields.push((
                "conditions",
                Json::Arr(
                    view.requirement
                        .conditions
                        .iter()
                        .map(crate::model::Condition::to_json)
                        .collect(),
                ),
            ));
        }
        requirements.push(Json::obj(requirement_fields));
    }
    let result = Json::obj(vec![
        ("project", Json::str(project)),
        (
            "model_fingerprint",
            Json::str(format!(
                "sha256:{}",
                crate::fingerprint::authority_model_digest(model)
            )),
        ),
        ("checks", sorted(checks, "id")),
        ("scenarios", sorted(scenarios, "id")),
        ("bindings", sorted_pairs(bindings)),
        ("requirements", sorted(requirements, "id")),
        // Outside every review dependency packet, so a Work Provenance reader can tell an intent
        // change from a dependency change without that distinction ever staling a review.
        ("requirement_intents", sorted(requirement_intents, "id")),
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
            text(value, "scenario").unwrap_or_default().to_string(),
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
                "scenarios",
                Json::Arr(declaration.scenarios.iter().map(Json::str).collect()),
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
            "scenarios",
            "bindings",
            "requirements",
            "requirement_intents",
        ],
    )?;
    id(value, "project")?;
    fingerprint(value, "model_fingerprint")?;
    fingerprint(value, "fingerprint")?;
    if text(value, "fingerprint")? != record_fingerprint(value)? {
        return Err("model authority fingerprint differs from exact projection".into());
    }
    crate::deployment_status::validate_policies(value)?;
    let mut checks = BTreeSet::new();
    for check in array(value, "checks")? {
        let mut fields = vec!["id", "fingerprint"];
        if let Some(policy) = check.get("deployment") {
            crate::deployment_status::validate_policy(policy)?;
            fields.push("deployment");
        }
        object(check, &fields)?;
        id(check, "id")?;
        fingerprint(check, "fingerprint")?;
        if !checks.insert(text(check, "id")?) {
            return Err("duplicate authority Check".into());
        }
    }
    let mut requirements = BTreeSet::new();
    for requirement in array(value, "requirements")? {
        let mut requirement_fields = vec![
            "id",
            "criticality",
            "fingerprint",
            "scenarios",
            "mechanisms",
            "bindings",
            "gaps",
        ];
        if requirement.get("verification_challenges").is_some() {
            requirement_fields.push("verification_challenges");
        }
        if let Some(conditions) = requirement.get("conditions") {
            requirement_fields.push("conditions");
            crate::deployment_status::validate_conditions(conditions)?;
        }
        object(requirement, &requirement_fields)?;
        if requirement.get("verification_challenges").is_some() {
            let mut seen = BTreeSet::new();
            for verification_challenge in array(requirement, "verification_challenges")? {
                object(
                    verification_challenge,
                    &[
                        "id",
                        "fingerprint",
                        "form",
                        "targets",
                        "requirements",
                        "mechanisms",
                        "checks",
                        "searches_for",
                        "rationale",
                        "implementations",
                        "required_scope",
                    ],
                )?;
                id(verification_challenge, "id")?;
                fingerprint(verification_challenge, "fingerprint")?;
                if !seen.insert(text(verification_challenge, "id")?) {
                    return Err("duplicate authority Verification Challenge".into());
                }
                for field in ["form", "searches_for"] {
                    text(verification_challenge, field)?;
                }
                for field in [
                    "targets",
                    "requirements",
                    "mechanisms",
                    "checks",
                    "required_scope",
                ] {
                    string_array(verification_challenge, field)?;
                }
                array(verification_challenge, "implementations")?;
            }
        }
        id(requirement, "id")?;
        fingerprint(requirement, "fingerprint")?;
        if !matches!(
            text(requirement, "criticality")?,
            "routine" | "standard" | "critical" | "unspecified"
        ) {
            return Err("invalid Requirement criticality".into());
        }
        if !requirements.insert(text(requirement, "id")?) {
            return Err("duplicate authority Requirement".into());
        }
        string_array(requirement, "gaps")?;
        let mut scenarios = BTreeSet::new();
        for scenario in array(requirement, "scenarios")? {
            let scenario = scenario.as_str().ok_or("Scenario dependency requires ID")?;
            crate::diag::validate_id(scenario, false)?;
            if !scenarios.insert(scenario) {
                return Err("duplicate Requirement Scenario dependency".into());
            }
        }
        let mut mechanisms = BTreeSet::new();
        for mechanism in array(requirement, "mechanisms")? {
            object(mechanism, &["id", "fingerprint"])?;
            id(mechanism, "id")?;
            fingerprint(mechanism, "fingerprint")?;
            if !mechanisms.insert(text(mechanism, "id")?) {
                return Err("duplicate mechanism dependency".into());
            }
        }
        let mut bindings = BTreeSet::new();
        for binding in array(requirement, "bindings")? {
            object(binding, &["check", "scenario"])?;
            id(binding, "check")?;
            id(binding, "scenario")?;
            if !bindings.insert((text(binding, "check")?, text(binding, "scenario")?)) {
                return Err("duplicate Requirement binding dependency".into());
            }
        }
    }
    let mut intents = Vec::new();
    for intent in array(value, "requirement_intents")? {
        object(intent, &["id", "fingerprint"])?;
        id(intent, "id")?;
        fingerprint(intent, "fingerprint")?;
        intents.push(text(intent, "id")?);
    }
    if !intents.windows(2).all(|pair| pair[0] < pair[1])
        || intents.iter().copied().collect::<BTreeSet<_>>() != requirements
    {
        return Err(
            "requirement_intents requires exactly one entry per authority Requirement, sorted by id"
                .into(),
        );
    }
    let mut scenarios = BTreeMap::new();
    for scenario in array(value, "scenarios")? {
        let mut scenario_fields = vec!["id", "requirement", "fingerprint"];
        if let Some(conditions) = scenario.get("conditions") {
            scenario_fields.push("conditions");
            crate::deployment_status::validate_conditions(conditions)?;
        }
        object(scenario, &scenario_fields)?;
        id(scenario, "id")?;
        id(scenario, "requirement")?;
        fingerprint(scenario, "fingerprint")?;
        if !requirements.contains(text(scenario, "requirement")?)
            || scenarios
                .insert(text(scenario, "id")?, text(scenario, "requirement")?)
                .is_some()
        {
            return Err("unknown Requirement or duplicate Scenario authority".into());
        }
    }
    let mut pairs = BTreeSet::new();
    for binding in array(value, "bindings")? {
        object(
            binding,
            &[
                "check",
                "scenario",
                "check_fingerprint",
                "scenario_fingerprint",
                "contribution",
                "fingerprint",
            ],
        )?;
        id(binding, "check")?;
        id(binding, "scenario")?;
        for name in ["check_fingerprint", "scenario_fingerprint", "fingerprint"] {
            fingerprint(binding, name)?;
        }
        text(binding, "contribution")?;
        let pair = (text(binding, "check")?, text(binding, "scenario")?);
        if !checks.contains(pair.0) || !scenarios.contains_key(pair.1) || !pairs.insert(pair) {
            return Err("unknown binding target or duplicate pair".into());
        }
        let check = array(value, "checks")?
            .iter()
            .find(|check| text(check, "id").ok() == Some(pair.0))
            .unwrap();
        let scenario = array(value, "scenarios")?
            .iter()
            .find(|scenario| text(scenario, "id").ok() == Some(pair.1))
            .unwrap();
        if text(binding, "check_fingerprint")? != text(check, "fingerprint")?
            || text(binding, "scenario_fingerprint")? != text(scenario, "fingerprint")?
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
    for requirement in array(value, "requirements")? {
        let id = text(requirement, "id")?;
        let owned = scenarios
            .iter()
            .filter(|(_, owner)| **owner == id)
            .map(|(scenario, _)| *scenario)
            .collect::<BTreeSet<_>>();
        let declared = array(requirement, "scenarios")?
            .iter()
            .filter_map(Json::as_str)
            .collect::<BTreeSet<_>>();
        if owned != declared {
            return Err("Requirement Scenario dependency inventory mismatch".into());
        }
        let expected = pairs
            .iter()
            .filter(|(_, scenario)| scenarios.get(scenario) == Some(&id))
            .copied()
            .collect::<BTreeSet<_>>();
        let declared = array(requirement, "bindings")?
            .iter()
            .map(|binding| {
                (
                    text(binding, "check").unwrap(),
                    text(binding, "scenario").unwrap(),
                )
            })
            .collect::<BTreeSet<_>>();
        if expected != declared {
            return Err("Requirement binding dependency inventory mismatch".into());
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
    if kind == "requirement-status" {
        return Err("requirement-status records are no longer authored; deployment Requirement Status is computed from applicable evidence".into());
    }
    let mut fields = vec![
        "kind",
        "reviewer",
        "reviewed_at_ms",
        "rationale",
        "artifacts",
        "fingerprint",
    ];
    if kind == "requirement-review" {
        if value.get("provenance").is_some() {
            return Err("Requirement Review carries no provenance; agent and model provenance belongs to the change's Work Provenance ledger and reassesses is a top-level field".into());
        }
        if value.get("dependencies").is_some() {
            return Err(
                "Requirement Review pins dependencies_fingerprint, not a dependencies packet"
                    .into(),
            );
        }
    } else if value.get("provenance").is_some() {
        fields.push("provenance");
    }
    fields.extend(match kind {
        "requirement-review" => vec![
            "requirement",
            "requirement_fingerprint",
            "dependencies_fingerprint",
            "decision",
            "reassesses",
            "challenge_basis",
        ],
        "method-qualification" => vec!["check", "check_fingerprint", "decision"],
        "applicability-decision" => vec![
            "check",
            "scenario",
            "check_fingerprint",
            "scenario_fingerprint",
            "binding_fingerprint",
            "verification_target_fingerprint",
            "method_qualification_fingerprint",
            "basis",
            "decision",
            "limitations",
        ],
        "requirement-judgment" => vec![
            "requirement",
            "requirement_fingerprint",
            "verification_target_fingerprint",
            "scenarios",
            "mechanisms",
            "bindings",
            "conclusion",
            "residual_risks",
        ],
        _ => return Err("unknown review kind".into()),
    });
    if kind == "requirement-review" {
        for optional in ["issues", "dispositions"] {
            if value.get(optional).is_some() {
                fields.push(optional);
            }
        }
    }
    object(value, &fields)?;
    if let Some(missing) = fields.iter().find(|field| value.get(field).is_none()) {
        return Err(format!("{kind} requires {missing}"));
    }
    if let Some(provenance) = value.get("provenance") {
        object(provenance, &["agent", "model", "reassesses"])?;
        for field in ["agent", "model"] {
            if provenance.get(field) != Some(&Json::Null) {
                text(provenance, field)?;
            }
        }
        if provenance.get("reassesses") != Some(&Json::Null) {
            fingerprint(provenance, "reassesses")?;
        }
    }
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
        "requirement-review" => validate_requirement_review(value)?,
        "method-qualification" => {
            id(value, "check")?;
            fingerprint(value, "check_fingerprint")?;
            decision(value)?;
        }
        "applicability-decision" => {
            id(value, "check")?;
            id(value, "scenario")?;
            for name in [
                "check_fingerprint",
                "scenario_fingerprint",
                "binding_fingerprint",
                "verification_target_fingerprint",
                "method_qualification_fingerprint",
            ] {
                fingerprint(value, name)?;
            }
            decision(value)?;
            string_array(value, "limitations")?;
            let basis = array(value, "basis")?;
            if basis.is_empty() {
                return Err("Applicability Decision requires Check Result basis".into());
            }
            let mut seen = BTreeSet::new();
            for item in basis {
                object(item, &["run", "bundle_fingerprint", "check_result"])?;
                for field in ["run", "bundle_fingerprint", "check_result"] {
                    fingerprint(item, field)?;
                }
                let key = (
                    text(item, "run")?,
                    text(item, "bundle_fingerprint")?,
                    text(item, "check_result")?,
                );
                if !seen.insert(key) {
                    return Err("duplicate Check Result basis".into());
                }
            }
        }
        "requirement-judgment" => {
            id(value, "requirement")?;
            fingerprint(value, "requirement_fingerprint")?;
            fingerprint(value, "verification_target_fingerprint")?;
            if !matches!(
                text(value, "conclusion")?,
                "supported" | "violated" | "unresolved"
            ) {
                return Err("unknown Requirement conclusion".into());
            }
            string_array(value, "residual_risks")?;
            for field in ["scenarios", "mechanisms"] {
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
                    &[
                        "check",
                        "scenario",
                        "fingerprint",
                        "applicability_fingerprint",
                    ],
                )?;
                id(item, "check")?;
                id(item, "scenario")?;
                fingerprint(item, "fingerprint")?;
                if !pairs.insert((text(item, "check")?, text(item, "scenario")?)) {
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
            result.push(("scenario", Json::str(text(value, "scenario")?)));
            result.push((
                "verification_target_fingerprint",
                Json::str(text(value, "verification_target_fingerprint")?),
            ));
        }
        "requirement-review" => {
            result.push(("requirement", Json::str(text(value, "requirement")?)))
        }
        "requirement-judgment" => {
            result.push(("requirement", Json::str(text(value, "requirement")?)));
            result.push((
                "verification_target_fingerprint",
                Json::str(text(value, "verification_target_fingerprint")?),
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
fn authority_binding<'a>(authority: &'a Json, check: &str, scenario: &str) -> Option<&'a Json> {
    authority.get("bindings")?.as_array()?.iter().find(|value| {
        value.get("check").and_then(Json::as_str) == Some(check)
            && value.get("scenario").and_then(Json::as_str) == Some(scenario)
    })
}
/// Runs whose protocol validity and active heads are established once, however many assessments consult them.
pub struct Runs<'a> {
    bundles: &'a [RunBundle],
    individually_verified: bool,
    heads: std::cell::OnceCell<Result<Vec<usize>, String>>,
}

impl<'a> Runs<'a> {
    pub fn new(bundles: &'a [RunBundle]) -> Self {
        Self {
            bundles,
            individually_verified: false,
            heads: std::cell::OnceCell::new(),
        }
    }

    /// For Runs the caller has already passed through `run::verify`, so assessment does not repeat that work.
    pub fn verified(bundles: &'a [RunBundle]) -> Self {
        Self {
            individually_verified: true,
            ..Self::new(bundles)
        }
    }

    fn active_heads(&self) -> Result<Vec<&RunBundle>, String> {
        self.heads
            .get_or_init(|| {
                if !self.individually_verified {
                    for bundle in self.bundles {
                        let findings = crate::run::verify(bundle);
                        if !findings.is_empty() {
                            return Err(format!(
                                "Run head is protocol-invalid: {}",
                                findings[0].detail
                            ));
                        }
                    }
                }
                let mut heads: BTreeMap<&str, usize> = BTreeMap::new();
                for (index, bundle) in self.bundles.iter().enumerate() {
                    match heads.get(bundle.run_id.as_str()) {
                        Some(&previous)
                            if self.bundles[previous].bundle_revision > bundle.bundle_revision => {}
                        Some(&previous)
                            if self.bundles[previous].bundle_revision == bundle.bundle_revision
                                && self.bundles[previous].bundle_fingerprint
                                    != bundle.bundle_fingerprint =>
                        {
                            return Err("ambiguous corrected Run head".into())
                        }
                        _ => {
                            heads.insert(&bundle.run_id, index);
                        }
                    }
                }
                Ok(heads.into_values().collect())
            })
            .as_ref()
            .map(|indexes| indexes.iter().map(|&index| &self.bundles[index]).collect())
            .map_err(Clone::clone)
    }
}

impl std::ops::Deref for Runs<'_> {
    type Target = [RunBundle];
    fn deref(&self) -> &[RunBundle] {
        self.bundles
    }
}

pub fn assess_record(
    record: &Json,
    authority: &Json,
    reviews: &[Json],
    bundles: &[RunBundle],
) -> Assessment {
    assess_record_in(record, authority, reviews, &Runs::new(bundles))
}

pub fn assess_record_in(
    record: &Json,
    authority: &Json,
    reviews: &[Json],
    bundles: &Runs,
) -> Assessment {
    if let Err(error) = validate_authority(authority) {
        return assessed("invalid", format!("invalid model authority: {error}"));
    }
    if let Err(error) = validate_record(record) {
        return assessed("invalid", error);
    }
    let kind = text(record, "kind").unwrap();
    if kind == "requirement-review" {
        return assess_requirement_review(record, authority, reviews, bundles);
    }
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
        let scenario = text(record, "scenario").unwrap();
        let Some(binding) = authority_binding(authority, check, scenario) else {
            return assessed(
                "stale",
                "Check–Scenario binding is absent from current authority",
            );
        };
        for (field, target) in [
            ("check_fingerprint", "check_fingerprint"),
            ("scenario_fingerprint", "scenario_fingerprint"),
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
        let qualification_state = assess_record_in(qualification, authority, reviews, bundles);
        if qualification_state.status != "accepted" {
            return assessed(
                &qualification_state.status,
                "Method Qualification is not currently accepted",
            );
        }
        let heads = match bundles.active_heads() {
            Ok(heads) => heads,
            Err(error) => return assessed("invalid", error),
        };
        let verification_target = text(record, "verification_target_fingerprint").unwrap();
        let check_fp = text(record, "check_fingerprint").unwrap();
        let basis = array(record, "basis").unwrap();
        let mut observed = BTreeSet::new();
        let mut latest = 0;
        for bundle in &heads {
            if bundle.verification_target_fingerprint == verification_target {
                for execution in &bundle.check_executions {
                    if execution.check.id == check && execution.check.fingerprint == check_fp {
                        for check_result in &execution.results {
                            if check_result.scenario == scenario {
                                latest = latest.max(check_result.observed_at_ms);
                                observed.insert((
                                    bundle.run_id.as_str(),
                                    bundle.bundle_fingerprint.as_str(),
                                    check_result.fingerprint.as_str(),
                                    check_result.observed_at_ms,
                                ));
                            }
                        }
                    }
                }
            }
        }
        if observed.is_empty() {
            return assessed("missing", "No current exact-target Check Result exists");
        }
        for item in basis {
            let run = text(item, "run").unwrap();
            let bundle = text(item, "bundle_fingerprint").unwrap();
            let check_result = text(item, "check_result").unwrap();
            if !observed
                .iter()
                .any(|value| value.0 == run && value.1 == bundle && value.2 == check_result)
            {
                return assessed(
                    "stale",
                    "Check Result basis is missing, corrected or from another Check/Scenario/Verification Target",
                );
            }
        }
        for (run, bundle, check_result, time) in observed {
            if time == latest
                && !basis.iter().any(|value| {
                    text(value, "run").ok() == Some(run)
                        && text(value, "bundle_fingerprint").ok() == Some(bundle)
                        && text(value, "check_result").ok() == Some(check_result)
                })
            {
                return assessed(
                    "stale",
                    "Applicability basis omits a latest active Check Result",
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
    let requirement_id = text(record, "requirement").unwrap();
    let Some(requirement) = authority_item(authority, "requirements", "id", requirement_id) else {
        return assessed("stale", "Requirement is absent from current authority");
    };
    if text(requirement, "criticality").ok() == Some("routine") {
        return assessed(
            "invalid",
            "Routine Requirements do not enroll Requirement Judgments",
        );
    }
    if record.get("requirement_fingerprint") != requirement.get("fingerprint") {
        return assessed(
            "stale",
            "Requirement account or supporting implementation changed",
        );
    }
    let scenario_ids = array(requirement, "scenarios")
        .unwrap()
        .iter()
        .filter_map(Json::as_str)
        .collect::<BTreeSet<_>>();
    let mut reviewed = BTreeSet::new();
    for scenario in array(record, "scenarios").unwrap() {
        let id = text(scenario, "id").unwrap();
        let Some(current) = authority_item(authority, "scenarios", "id", id) else {
            return assessed("stale", "Judgment includes absent Scenario");
        };
        if scenario.get("fingerprint") != current.get("fingerprint") {
            return assessed("stale", "Scenario contract changed");
        }
        reviewed.insert(id);
    }
    if reviewed != scenario_ids {
        return assessed(
            "stale",
            "Judgment Scenario inventory is incomplete or obsolete",
        );
    }
    let mechanisms = array(requirement, "mechanisms").unwrap();
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
    let expected = array(requirement, "bindings").unwrap();
    let declared = array(record, "bindings").unwrap();
    if expected.len() != declared.len()
        || expected.iter().any(|binding| {
            !declared.iter().any(|value| {
                value.get("check") == binding.get("check")
                    && value.get("scenario") == binding.get("scenario")
            })
        })
    {
        return assessed(
            "stale",
            "Judgment binding inventory is incomplete or obsolete",
        );
    }
    let conclusion = text(record, "conclusion").unwrap();
    let verification_target = text(record, "verification_target_fingerprint").unwrap();
    let mut satisfied = false;
    let mut violated = false;
    let mut unresolved = false;
    for binding in declared {
        let check = text(binding, "check").unwrap();
        let scenario = text(binding, "scenario").unwrap();
        let Some(current) = authority_binding(authority, check, scenario) else {
            return assessed("stale", "Judgment binding is absent");
        };
        if binding.get("fingerprint") != current.get("fingerprint") {
            return assessed("stale", "Judgment binding definition changed");
        }
        let target = Json::obj(vec![
            ("kind", Json::str("applicability-decision")),
            ("check", Json::str(check)),
            ("scenario", Json::str(scenario)),
            (
                "verification_target_fingerprint",
                Json::str(verification_target),
            ),
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
        let state = assess_record_in(application, authority, reviews, bundles);
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
                CheckResultOutcome::Satisfied => satisfied = true,
                CheckResultOutcome::Violated => violated = true,
                CheckResultOutcome::Inconclusive => unresolved = true,
            }
        }
    }
    let gaps = !array(requirement, "gaps").unwrap().is_empty()
        || text(requirement, "criticality").ok() == Some("unspecified");
    match conclusion {
        "supported" if gaps || unresolved || violated || !satisfied => assessed(
            "invalid",
            "Supported judgment has structural gaps, incomplete evidence or counterevidence",
        ),
        "violated" if !violated => assessed(
            "invalid",
            "Violated judgment lacks applicable violated Check Result",
        ),
        _ => assessed("accepted", ""),
    }
}
fn bound_outcomes(record: &Json, bundles: &Runs) -> Vec<CheckResultOutcome> {
    let mut result = Vec::new();
    for basis in array(record, "basis").unwrap_or_default() {
        for bundle in bundles.iter() {
            if text(basis, "run").ok() == Some(bundle.run_id.as_str())
                && text(basis, "bundle_fingerprint").ok()
                    == Some(bundle.bundle_fingerprint.as_str())
            {
                for execution in &bundle.check_executions {
                    if text(record, "check").ok() == Some(execution.check.id.as_str()) {
                        for check_result in &execution.results {
                            if text(basis, "check_result").ok()
                                == Some(check_result.fingerprint.as_str())
                            {
                                result.push(check_result.outcome);
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn historical_verification_target_state(
    authority: &Json,
    reviews: &[Json],
    bundles: &[RunBundle],
    verification_target_fingerprint: &str,
) -> Json {
    historical_verification_target_state_in(
        authority,
        reviews,
        &Runs::new(bundles),
        verification_target_fingerprint,
    )
}

pub fn historical_verification_target_state_in(
    authority: &Json,
    reviews: &[Json],
    bundles: &Runs,
    verification_target_fingerprint: &str,
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
            Ok(Some(record)) => assess_record_in(record, authority, reviews, bundles),
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
    let mut requirements = Vec::new();
    for requirement in array(authority, "requirements").unwrap() {
        let id = text(requirement, "id").unwrap();
        let mut binding_states = Vec::new();
        for binding in array(requirement, "bindings").unwrap() {
            let target = Json::obj(vec![
                ("kind", Json::str("applicability-decision")),
                ("check", binding.get("check").cloned().unwrap()),
                ("scenario", binding.get("scenario").cloned().unwrap()),
                (
                    "verification_target_fingerprint",
                    Json::str(verification_target_fingerprint),
                ),
            ]);
            let assessment = match latest_review(reviews, &target) {
                Ok(Some(record)) => assess_record_in(record, authority, reviews, bundles),
                Ok(None) => assessed("missing", "Applicability Decision is absent"),
                Err(error) => assessed("invalid", error),
            };
            binding_states.push(Json::obj(vec![
                ("check", binding.get("check").cloned().unwrap()),
                ("scenario", binding.get("scenario").cloned().unwrap()),
                ("status", Json::str(assessment.status)),
                (
                    "diagnostics",
                    Json::Arr(assessment.diagnostics.iter().map(Json::str).collect()),
                ),
            ]));
        }
        let target = Json::obj(vec![
            ("kind", Json::str("requirement-judgment")),
            ("requirement", Json::str(id)),
            (
                "verification_target_fingerprint",
                Json::str(verification_target_fingerprint),
            ),
        ]);
        let (state, assessment) = if text(requirement, "criticality").ok() == Some("routine") {
            ("review-not-required".into(), assessed("not-required", ""))
        } else {
            match latest_review(reviews, &target) {
                Ok(Some(record)) => {
                    let assessment = assess_record_in(record, authority, reviews, bundles);
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
                    assessed("missing", "Requirement Judgment is absent"),
                ),
                Err(error) => ("unresolved".into(), assessed("invalid", error)),
            }
        };
        requirements.push(Json::obj(vec![
            ("requirement", Json::str(id)),
            ("status", Json::str(state)),
            ("judgment", Json::str(assessment.status)),
            (
                "diagnostics",
                Json::Arr(assessment.diagnostics.iter().map(Json::str).collect()),
            ),
            ("gaps", requirement.get("gaps").cloned().unwrap()),
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
        (
            "verification_target_fingerprint",
            Json::str(verification_target_fingerprint),
        ),
        ("requirements", Json::Arr(requirements)),
    ])
}

pub fn review_input(
    model: &Model,
    project: &str,
    request: &Json,
    reviews: &[Json],
    bundles: &[RunBundle],
) -> Result<Json, String> {
    review_input_in(model, project, request, reviews, &Runs::new(bundles))
}

pub fn review_input_in(
    model: &Model,
    project: &str,
    request: &Json,
    reviews: &[Json],
    bundles: &Runs,
) -> Result<Json, String> {
    let kind = text(request, "kind")?;
    match kind {
        "requirement-review" => requirement_review_input(model, project, request, reviews, bundles),
        "requirement-status" => Err("requirement-status records are no longer authored; deployment Requirement Status is computed from applicable evidence".into()),
        "method-qualification" | "applicability-decision" | "requirement-judgment" => Err("new review input requires requirement-review; historical native records remain inspectable".into()),
        _ => Err("review input kind requires requirement-review".into()),
    }
}

fn review_definitions(
    model: &Model,
    check_ids: &BTreeSet<String>,
    scenario_ids: &BTreeSet<String>,
    requirement_ids: &BTreeSet<String>,
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
                                                    section.requirement.is_none()
                                                        || authored.is_some_and(|check| {
                                                            section.requirement.as_deref()
                                                                == Some(&check.requirement)
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
    let scenarios = model
        .scenarios()
        .filter(|view| scenario_ids.contains(&view.scenario.id))
        .map(|view| {
            Json::obj(vec![
                ("id", Json::str(&view.scenario.id)),
                ("requirement", Json::str(&view.requirement.id)),
                ("statement", Json::str(&view.scenario.statement)),
                (
                    "requirement_statement",
                    Json::str(&view.requirement.statement),
                ),
                ("domain", Json::str(view.requirement.domain.name())),
                (
                    "over",
                    view.requirement.over.as_ref().map_or(Json::Null, Json::str),
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
    let requirements = model
        .requirements()
        .filter(|view| requirement_ids.contains(&view.requirement.id))
        .map(|view| {
            Json::obj(vec![
                ("id", Json::str(&view.requirement.id)),
                ("statement", Json::str(&view.requirement.statement)),
                (
                    "verification_challenges",
                    requirement_verification_challenges(model, &view.requirement.id),
                ),
                (
                    "realizations",
                    Json::Arr(
                        model
                            .realizes
                            .iter()
                            .filter(|site| site.requirement == view.requirement.id)
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
                (
                    "design",
                    model
                        .account_design_for_requirement(&view.spec.id, &view.requirement.id)
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
                                        "requirement_prose",
                                        Json::Arr(
                                            file.requirements
                                                .iter()
                                                .filter(|requirement| {
                                                    requirement.requirement == view.requirement.id
                                                })
                                                .map(|requirement| Json::str(&requirement.prose))
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
        ("scenarios", Json::Arr(scenarios)),
        ("requirements", Json::Arr(requirements)),
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

fn explanation_record(record: Option<&Json>, assessment: &Assessment) -> Json {
    let field = |name| {
        record
            .and_then(|r| r.get(name))
            .cloned()
            .unwrap_or(Json::Null)
    };
    Json::obj(vec![
        ("status", Json::str(&assessment.status)),
        ("fingerprint", field("fingerprint")),
        ("reviewer", field("reviewer")),
        ("reviewed_at_ms", field("reviewed_at_ms")),
        ("rationale", field("rationale")),
        (
            "diagnostics",
            Json::Arr(assessment.diagnostics.iter().map(Json::str).collect()),
        ),
    ])
}

/// Requirement Status is never authored; a deployment's status is computed in
/// `deployment_status`. This account therefore shows the Requirement Review and the observed exact-target
/// evidence as unassessed facts and concludes nothing from them.
fn verification_target_explanation(
    authority: &Json,
    requirement: &Json,
    reviews: &[Json],
    heads: &[&RunBundle],
    verification_target: &str,
    review: &Assessment,
) -> Json {
    let id = text(requirement, "id").unwrap();
    let review_target = Json::obj(vec![
        ("kind", Json::str("requirement-review")),
        ("requirement", Json::str(id)),
    ]);
    let review_record = latest_review(reviews, &review_target).ok().flatten();
    let mut reasons = Vec::new();
    if review.status != "accepted" {
        reasons.push(Json::obj(vec![
            (
                "code",
                Json::str(format!("requirement-review-{}", review.status)),
            ),
            ("category", Json::str("requirement-review")),
            ("status", Json::str(&review.status)),
            (
                "message",
                Json::str(format!(
                    "Requirement Review is {} for this assessment.",
                    review.status
                )),
            ),
            (
                "diagnostics",
                Json::Arr(review.diagnostics.iter().map(Json::str).collect()),
            ),
        ]));
    }
    if !array(requirement, "gaps").unwrap().is_empty() {
        reasons.push(Json::obj(vec![
            ("code", Json::str("structural-gaps")),
            ("category", Json::str("definition")),
            (
                "message",
                Json::str("The Requirement has unresolved structural gaps."),
            ),
            ("details", requirement.get("gaps").unwrap().clone()),
        ]));
    }
    let mut unassessed = Vec::new();
    let mut coverage = Vec::new();
    for binding in array(requirement, "bindings").unwrap() {
        let check_id = text(binding, "check").unwrap();
        let scenario_id = text(binding, "scenario").unwrap();
        let check_fp = authority_item(authority, "checks", "id", check_id)
            .and_then(|c| c.get("fingerprint"))
            .and_then(Json::as_str);
        let mut observed = Vec::new();
        for bundle in heads
            .iter()
            .filter(|b| b.verification_target_fingerprint == verification_target)
        {
            for execution in bundle.check_executions.iter().filter(|e| {
                e.check.id == check_id && Some(e.check.fingerprint.as_str()) == check_fp
            }) {
                for check_result in execution
                    .results
                    .iter()
                    .filter(|o| o.scenario == scenario_id)
                {
                    observed.push(Json::obj(vec![
                        ("check", Json::str(check_id)),
                        ("scenario", Json::str(scenario_id)),
                        ("run", Json::str(&bundle.run_id)),
                        ("bundle_fingerprint", Json::str(&bundle.bundle_fingerprint)),
                        ("check_result", Json::str(&check_result.fingerprint)),
                        ("outcome", Json::str(check_result.outcome.name())),
                        (
                            "observed_at_ms",
                            Json::Num(check_result.observed_at_ms as f64),
                        ),
                        (
                            "diagnostics",
                            Json::Arr(check_result.diagnostics.iter().map(Json::str).collect()),
                        ),
                    ]));
                }
            }
        }
        let coverage_status = if observed.is_empty() {
            reasons.push(Json::obj(vec![
                ("code", Json::str("current-check-result-missing")),
                ("category", Json::str("evidence")),
                ("check", Json::str(check_id)), ("scenario", Json::str(scenario_id)),
                ("message", Json::str(format!("No current matching Check Result is recorded for Check {check_id} and Scenario {scenario_id}."))),
            ]));
            "missing"
        } else {
            "unassessed"
        };
        unassessed.extend(observed.iter().cloned());
        coverage.push(Json::obj(vec![
            ("check", Json::str(check_id)),
            ("scenario", Json::str(scenario_id)),
            ("status", Json::str(coverage_status)),
            ("check_results", Json::Arr(observed)),
        ]));
    }
    let mut summary = reasons
        .iter()
        .filter_map(|r| r.get("message").and_then(Json::as_str))
        .collect::<Vec<_>>();
    summary.push("Requirement Status is not computed for this Verification Target; deployment Requirement Status is computed from applicable evidence.");
    Json::obj(vec![
        ("summary", Json::str(summary.join(" "))),
        ("status", Json::str("unresolved")),
        ("reasons", Json::Arr(reasons)),
        ("review", explanation_record(review_record, review)),
        ("coverage", Json::Arr(coverage)),
        (
            "evidence",
            Json::obj(vec![
                ("supporting", Json::Arr(vec![])),
                ("violations", Json::Arr(vec![])),
                ("inconclusive", Json::Arr(vec![])),
                ("unassessed", Json::Arr(unassessed)),
            ]),
        ),
    ])
}

pub fn verification_target_state(
    authority: &Json,
    reviews: &[Json],
    bundles: &[RunBundle],
    verification_target_fingerprint: &str,
) -> Json {
    verification_target_state_in(
        authority,
        reviews,
        &Runs::new(bundles),
        verification_target_fingerprint,
    )
}

pub fn verification_target_state_in(
    authority: &Json,
    reviews: &[Json],
    bundles: &Runs,
    verification_target_fingerprint: &str,
) -> Json {
    if let Err(error) = validate_authority(authority) {
        return Json::obj(vec![
            ("status", Json::str("invalid")),
            ("diagnostics", Json::Arr(vec![Json::str(error)])),
        ]);
    }
    let heads = match bundles.active_heads() {
        Ok(heads) => heads,
        Err(error) => {
            return Json::obj(vec![
                ("status", Json::str("invalid")),
                ("diagnostics", Json::Arr(vec![Json::str(error)])),
            ])
        }
    };
    let requirements = array(authority, "requirements")
        .unwrap()
        .iter()
        .map(|requirement| {
            let id = text(requirement, "id").unwrap();
            let review_target = Json::obj(vec![
                ("kind", Json::str("requirement-review")),
                ("requirement", Json::str(id)),
            ]);
            let review = match latest_review(reviews, &review_target) {
                Ok(Some(record)) => assess_record_in(record, authority, reviews, bundles),
                Ok(None) => assessed("missing", "Requirement Review absent"),
                Err(error) => assessed("invalid", error),
            };
            let fingerprints = latest_review(reviews, &review_target)
                .ok()
                .flatten()
                .and_then(|record| record.get("fingerprint").and_then(Json::as_str))
                .into_iter()
                .collect::<BTreeSet<_>>();
            let challenges = heads
                .iter()
                .flat_map(|bundle| {
                    bundle
                        .verification_challenge_executions
                        .iter()
                        .filter(|execution| {
                            execution.target.id == id
                                && execution.target.kind == ChallengeTargetKind::RequirementReview
                                && fingerprints.contains(execution.target.fingerprint.as_str())
                        })
                        .map(|execution| {
                            Json::obj(vec![
                                (
                                    "verification_challenge",
                                    Json::str(&execution.verification_challenge.id),
                                ),
                                ("target_kind", Json::str(execution.target.kind.name())),
                                (
                                    "target_fingerprint",
                                    Json::str(&execution.target.fingerprint),
                                ),
                                ("result", Json::str(execution.result.outcome.name())),
                                (
                                    "result_fingerprint",
                                    Json::str(&execution.result.fingerprint),
                                ),
                                ("run", Json::str(&bundle.run_id)),
                                (
                                    "execution_verification_target_fingerprint",
                                    Json::str(&bundle.verification_target_fingerprint),
                                ),
                            ])
                        })
                })
                .collect();
            let explanation = verification_target_explanation(
                authority,
                requirement,
                reviews,
                &heads,
                verification_target_fingerprint,
                &review,
            );
            Json::obj(vec![
                ("explanation", explanation),
                ("requirement", Json::str(id)),
                ("status", Json::str("unresolved")),
                ("review", Json::str(&review.status)),
                (
                    "diagnostics",
                    Json::Arr(review.diagnostics.iter().map(Json::str).collect()),
                ),
                ("gaps", requirement.get("gaps").unwrap().clone()),
                ("bindings", Json::Arr(vec![])),
                ("challenges", Json::Arr(challenges)),
            ])
        })
        .collect();
    Json::obj(vec![
        ("project", authority.get("project").unwrap().clone()),
        (
            "authority_fingerprint",
            authority.get("fingerprint").unwrap().clone(),
        ),
        (
            "model_fingerprint",
            authority.get("model_fingerprint").unwrap().clone(),
        ),
        (
            "verification_target_fingerprint",
            Json::str(verification_target_fingerprint),
        ),
        ("requirements", Json::Arr(requirements)),
    ])
}

fn requirement_verification_challenges(model: &Model, requirement: &str) -> Json {
    let mut entries = model
        .verification_challenges()
        .filter(|verification_challenge| {
            verification_challenge
                .requirements
                .iter()
                .any(|id| id == requirement)
        })
        .map(|verification_challenge| {
            let implementations = model
                .package_producers
                .iter()
                .filter(|producer| {
                    producer.package == "azimuth.challenge"
                        && producer.contract == "verification-challenge-implementation"
                        && producer.entity == verification_challenge.id
                })
                .map(|producer| producer.to_json())
                .collect();
            Json::obj(vec![
                ("id", Json::str(&verification_challenge.id)),
                (
                    "fingerprint",
                    Json::str(crate::fingerprint::verification_challenge_fingerprint(
                        verification_challenge,
                    )),
                ),
                ("form", Json::str(&verification_challenge.form)),
                (
                    "targets",
                    Json::Arr(
                        verification_challenge
                            .targets
                            .iter()
                            .map(Json::str)
                            .collect(),
                    ),
                ),
                (
                    "requirements",
                    Json::Arr(
                        verification_challenge
                            .requirements
                            .iter()
                            .map(Json::str)
                            .collect(),
                    ),
                ),
                (
                    "mechanisms",
                    Json::Arr(
                        verification_challenge
                            .mechanisms
                            .iter()
                            .map(Json::str)
                            .collect(),
                    ),
                ),
                (
                    "checks",
                    Json::Arr(
                        verification_challenge
                            .checks
                            .iter()
                            .map(Json::str)
                            .collect(),
                    ),
                ),
                (
                    "searches_for",
                    Json::str(&verification_challenge.searches_for),
                ),
                ("rationale", Json::str(&verification_challenge.rationale)),
                (
                    "required_scope",
                    Json::Arr(
                        verification_challenge
                            .required_scope
                            .iter()
                            .map(|kind| Json::str(kind.name()))
                            .collect(),
                    ),
                ),
                ("implementations", Json::Arr(implementations)),
            ])
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| text(entry, "id").unwrap().to_string());
    Json::Arr(entries)
}

/// The review dependency packet; its canonical fingerprint is a Requirement's closure identity.
pub fn requirement_closure_fingerprint(
    authority: &Json,
    requirement_id: &str,
) -> Result<String, String> {
    Ok(crate::fingerprint::canonical_sha256(
        &requirement_dependencies(authority, requirement_id)?,
    ))
}

pub fn requirement_dependencies(authority: &Json, requirement_id: &str) -> Result<Json, String> {
    let requirement = authority_item(authority, "requirements", "id", requirement_id)
        .ok_or("unknown Requirement")?;
    let scenarios = array(requirement, "scenarios")?;
    let pairs = array(requirement, "bindings")?;
    let checks = array(authority, "checks")?
        .iter()
        .filter(|v| pairs.iter().any(|b| b.get("check") == v.get("id")))
        .cloned()
        .collect();
    let selected_scenarios = array(authority, "scenarios")?
        .iter()
        .filter(|v| scenarios.contains(v.get("id").unwrap()))
        .cloned()
        .collect();
    let bindings = pairs
        .iter()
        .map(|b| {
            authority_binding(
                authority,
                text(b, "check").unwrap(),
                text(b, "scenario").unwrap(),
            )
            .unwrap()
            .clone()
        })
        .collect();
    Ok(Json::obj(vec![
        ("checks", Json::Arr(checks)),
        ("scenarios", Json::Arr(selected_scenarios)),
        ("requirements", Json::Arr(vec![requirement.clone()])),
        ("bindings", Json::Arr(bindings)),
    ]))
}
pub fn implementation_challenge_fingerprint(
    authority: &Json,
    challenge_id: &str,
    source_scope_fingerprint: &str,
    verification_target_fingerprint: &str,
) -> Result<String, String> {
    let mut declaration = None;
    for requirement in array(authority, "requirements")? {
        for challenge in requirement
            .get("verification_challenges")
            .and_then(Json::as_array)
            .unwrap_or(&[])
        {
            if challenge.get("id").and_then(Json::as_str) == Some(challenge_id) {
                if declaration
                    .as_ref()
                    .is_some_and(|previous| previous != challenge)
                {
                    return Err("inconsistent Verification Challenge declarations".into());
                }
                declaration = Some(challenge.clone());
            }
        }
    }
    let declaration = declaration.ok_or("Verification Challenge absent from authority")?;
    let mut closures = Vec::new();
    for requirement in array(&declaration, "requirements")? {
        let id = requirement
            .as_str()
            .ok_or("Challenge Requirement must be an id")?;
        closures.push(Json::obj(vec![
            ("requirement", Json::str(id)),
            (
                "fingerprint",
                Json::str(requirement_closure_fingerprint(authority, id)?),
            ),
        ]));
    }
    let mut checks = Vec::new();
    for check in array(&declaration, "checks")? {
        let id = check.as_str().ok_or("Challenge Check must be an id")?;
        checks.push(
            authority_item(authority, "checks", "id", id)
                .ok_or("Challenge Check absent from authority")?
                .clone(),
        );
    }
    Ok(crate::run::canonical_fingerprint(&Json::obj(vec![
        (
            "format",
            Json::str("azimuth-challenge-implementation-basis"),
        ),
        ("version", Json::Num(1.0)),
        ("verification_challenge", declaration),
        ("requirements", Json::Arr(closures)),
        ("checks", Json::Arr(checks)),
        (
            "source_scope_fingerprint",
            Json::str(source_scope_fingerprint),
        ),
        (
            "verification_target_fingerprint",
            Json::str(verification_target_fingerprint),
        ),
    ]))?)
}

pub fn implementation_challenge_current(
    authority: &Json,
    bundle: &RunBundle,
    execution: &crate::run::VerificationChallengeExecution,
) -> Result<bool, String> {
    if execution.target.kind != ChallengeTargetKind::Implementation
        || execution.target.id != execution.verification_challenge.id
    {
        return Ok(false);
    }
    let declaration = array(authority, "requirements")?
        .iter()
        .flat_map(|item| {
            item.get("verification_challenges")
                .and_then(Json::as_array)
                .unwrap_or(&[])
        })
        .find(|challenge| {
            challenge.get("id").and_then(Json::as_str)
                == Some(execution.verification_challenge.id.as_str())
        })
        .ok_or("Verification Challenge absent from current authority")?;
    if declaration.get("fingerprint").and_then(Json::as_str)
        != Some(execution.verification_challenge.fingerprint.as_str())
    {
        return Ok(false);
    }
    let selection = bundle
        .plan
        .challenges
        .iter()
        .find(|selection| selection.id == execution.challenge)
        .ok_or("Challenge selection absent from Run plan")?;
    let scopes = selection
        .units
        .iter()
        .map(|unit| {
            unit.parameters
                .get("scope_fingerprint")
                .cloned()
                .ok_or_else(|| {
                    "implementation Challenge unit has no source scope fingerprint".to_string()
                })
        })
        .collect::<Result<BTreeSet<_>, String>>()?;
    if scopes.len() != 1 {
        return Err("implementation Challenge requires one exact source scope".into());
    }
    let expected = implementation_challenge_fingerprint(
        authority,
        &execution.verification_challenge.id,
        scopes.iter().next().unwrap(),
        &bundle.verification_target_fingerprint,
    )?;
    Ok(expected == execution.target.fingerprint)
}

pub fn challenge_issue_id(
    execution: &crate::run::VerificationChallengeExecution,
    objection: &str,
) -> String {
    format!(
        "challenge-issue-{}",
        crate::fingerprint::canonical_sha256(&Json::obj(vec![
            (
                "verification_challenge",
                Json::str(&execution.verification_challenge.id)
            ),
            ("target", Json::str(&execution.target.fingerprint)),
            ("objection", Json::str(objection)),
        ]))
        .trim_start_matches("sha256:")
    )
}

fn validate_review_issues(value: &Json) -> Result<(), String> {
    let mut identities = BTreeSet::new();
    if value.get("issues").is_some() {
        for issue in array(value, "issues")? {
            object(
                issue,
                &[
                    "id",
                    "proposition",
                    "location",
                    "reason",
                    "correction",
                    "blocking",
                ],
            )?;
            id(issue, "id")?;
            for field in ["proposition", "location", "reason", "correction"] {
                text(issue, field)?;
            }
            issue
                .get("blocking")
                .and_then(Json::as_bool)
                .ok_or("issue blocking requires boolean")?;
            if !identities.insert(text(issue, "id")?) {
                return Err("duplicate review issue id".into());
            }
            if text(value, "decision")? == "accepted"
                && issue.get("blocking").and_then(Json::as_bool) == Some(true)
            {
                return Err("accepted review cannot retain a blocking issue".into());
            }
        }
        if text(value, "decision")? == "rejected" && identities.is_empty() {
            return Err("rejected review requires at least one stable issue".into());
        }
    }
    let mut disposed = BTreeSet::new();
    if value.get("dispositions").is_some() {
        for disposition in array(value, "dispositions")? {
            object(disposition, &["issue", "outcome", "rationale"])?;
            text(disposition, "issue")?;
            text(disposition, "rationale")?;
            let outcome = text(disposition, "outcome")?;
            if !matches!(
                outcome,
                "corrected" | "equivalent" | "irrelevant" | "confirmed-defect" | "unresolved"
            ) {
                return Err("unsupported issue disposition".into());
            }
            if !disposed.insert(text(disposition, "issue")?) {
                return Err("duplicate issue disposition".into());
            }
            if text(value, "decision")? == "accepted"
                && matches!(outcome, "confirmed-defect" | "unresolved")
            {
                return Err(
                    "accepted review cannot retain an unresolved or confirmed defect".into(),
                );
            }
        }
    }
    Ok(())
}

fn validate_requirement_review(value: &Json) -> Result<(), String> {
    validate_review_issues(value)?;
    id(value, "requirement")?;
    fingerprint(value, "requirement_fingerprint")?;
    fingerprint(value, "dependencies_fingerprint")?;
    if value.get("reassesses") != Some(&Json::Null) {
        fingerprint(value, "reassesses")?;
    }
    let mut seen = BTreeSet::new();
    for item in array(value, "challenge_basis")? {
        object(
            item,
            &[
                "run",
                "bundle_fingerprint",
                "result_fingerprint",
                "target_fingerprint",
            ],
        )?;
        for field in [
            "run",
            "bundle_fingerprint",
            "result_fingerprint",
            "target_fingerprint",
        ] {
            fingerprint(item, field)?;
        }
        if !seen.insert(item.to_string_pretty()) {
            return Err("duplicate Challenge basis".into());
        }
    }
    decision(value)
}
fn assess_requirement_review(
    record: &Json,
    authority: &Json,
    reviews: &[Json],
    bundles: &Runs,
) -> Assessment {
    let requirement = text(record, "requirement").unwrap();
    if let Some(previous) = record.get("reassesses").and_then(Json::as_str) {
        if let Some(before) = reviews
            .iter()
            .find(|review| review.get("fingerprint").and_then(Json::as_str) == Some(previous))
        {
            for issue in before.get("issues").and_then(Json::as_array).unwrap_or(&[]) {
                if issue.get("blocking").and_then(Json::as_bool) != Some(true) {
                    continue;
                }
                let reference = format!("{}#{}", previous, text(issue, "id").unwrap_or_default());
                let disposition = record
                    .get("dispositions")
                    .and_then(Json::as_array)
                    .unwrap_or(&[])
                    .iter()
                    .find(|item| {
                        item.get("issue").and_then(Json::as_str) == Some(reference.as_str())
                    });
                if disposition.is_none_or(|item| {
                    !matches!(
                        item.get("outcome").and_then(Json::as_str),
                        Some("corrected" | "equivalent" | "irrelevant")
                    )
                }) && record.get("decision").and_then(Json::as_str) == Some("accepted")
                {
                    return assessed(
                        "unresolved",
                        format!("Predecessor issue {reference} requires independent disposition"),
                    );
                }
            }
            if review_target(before).ok() != review_target(record).ok()
                || before.get("reviewed_at_ms").and_then(Json::as_num)
                    >= record.get("reviewed_at_ms").and_then(Json::as_num)
            {
                return assessed(
                    "invalid",
                    "Reassessment must have the same target and later review time",
                );
            }
        } else {
            return assessed("missing", "Reassessed review absent from retained history");
        }
    }
    let closure = match requirement_closure_fingerprint(authority, requirement) {
        Ok(v) => v,
        Err(e) => return assessed("stale", e),
    };
    if record
        .get("dependencies_fingerprint")
        .and_then(Json::as_str)
        != Some(closure.as_str())
        || record.get("requirement_fingerprint")
            != authority_item(authority, "requirements", "id", requirement)
                .and_then(|v| v.get("fingerprint"))
    {
        return assessed(
            "stale",
            "Requirement, implementation or verification dependencies changed",
        );
    }
    let challenge_heads = match bundles.active_heads() {
        Ok(heads) => heads,
        Err(error) => return assessed("invalid", error),
    };
    for item in array(record, "challenge_basis").unwrap() {
        let found = challenge_heads.iter().any(|bundle| {
            item.get("run").and_then(Json::as_str) == Some(bundle.run_id.as_str())
                && item.get("bundle_fingerprint").and_then(Json::as_str)
                    == Some(bundle.bundle_fingerprint.as_str())
                && bundle
                    .verification_challenge_executions
                    .iter()
                    .any(|execution| {
                        item.get("result_fingerprint").and_then(Json::as_str)
                            == Some(execution.result.fingerprint.as_str())
                            && item.get("target_fingerprint").and_then(Json::as_str)
                                == Some(execution.target.fingerprint.as_str())
                            && ((execution.target.id == requirement
                                && execution.target.kind == ChallengeTargetKind::RequirementReview)
                                || (execution.target.kind == ChallengeTargetKind::Implementation
                                    && implementation_challenge_current(
                                        authority, bundle, execution,
                                    )
                                    .unwrap_or(false)
                                    && authority_item(
                                        authority,
                                        "requirements",
                                        "id",
                                        requirement,
                                    )
                                    .and_then(|item| item.get("verification_challenges"))
                                    .and_then(Json::as_array)
                                    .unwrap_or(&[])
                                    .iter()
                                    .any(|challenge| {
                                        challenge.get("id").and_then(Json::as_str)
                                            == Some(execution.verification_challenge.id.as_str())
                                    })))
                    })
        });
        if !found {
            return assessed(
                "stale",
                "Relied-on Challenge Result is missing or corrected",
            );
        }
    }
    if record.get("decision").and_then(Json::as_str) == Some("rejected") {
        return assessed("rejected", "");
    }
    for item in array(record, "challenge_basis").unwrap() {
        for bundle in &challenge_heads {
            if item.get("bundle_fingerprint").and_then(Json::as_str)
                != Some(bundle.bundle_fingerprint.as_str())
            {
                continue;
            }
            for execution in &bundle.verification_challenge_executions {
                if item.get("result_fingerprint").and_then(Json::as_str)
                    != Some(execution.result.fingerprint.as_str())
                    || execution.target.kind != ChallengeTargetKind::Implementation
                {
                    continue;
                }
                if bundle.status != crate::run::RunStatus::Complete
                    || execution.result.diagnostics.iter().any(|id| {
                        bundle.diagnostics.iter().any(|diagnostic| {
                            diagnostic.id == *id
                                && diagnostic.class != crate::run::DiagnosticClass::Objection
                        })
                    })
                    || execution.result.outcome == crate::run::ChallengeOutcome::Inconclusive
                {
                    return assessed(
                        "unresolved",
                        "Relied-on implementation Challenge execution is inconclusive",
                    );
                }
                for objection in &execution.result.objections {
                    let issue = challenge_issue_id(execution, objection);
                    let disposition = record
                        .get("dispositions")
                        .and_then(Json::as_array)
                        .unwrap_or(&[])
                        .iter()
                        .find(|disposition| {
                            disposition.get("issue").and_then(Json::as_str) == Some(issue.as_str())
                        });
                    if disposition.is_none_or(|disposition| {
                        !matches!(
                            disposition.get("outcome").and_then(Json::as_str),
                            Some("equivalent" | "irrelevant")
                        )
                    }) {
                        return assessed("unresolved", format!("Implementation Challenge issue {issue} requires independent equivalence or irrelevance disposition"));
                    }
                }
            }
        }
    }
    for execution in challenge_heads
        .iter()
        .flat_map(|bundle| &bundle.verification_challenge_executions)
    {
        if execution.target.kind == ChallengeTargetKind::RequirementReview
            && execution.target.id == requirement
            && Some(execution.target.fingerprint.as_str())
                == record.get("fingerprint").and_then(Json::as_str)
            && execution.result.outcome.name() != "clean"
        {
            return assessed(
                "unresolved",
                format!(
                    "Exact assessment has {} Challenge Result; reassessment required",
                    execution.result.outcome.name()
                ),
            );
        }
    }
    assessed(text(record, "decision").unwrap(), "")
}
fn reassessment_history(selected: Vec<Json>, reviews: &[Json]) -> Vec<Json> {
    let mut retained = BTreeMap::new();
    let mut pending = selected;
    while let Some(record) = pending.pop() {
        let fingerprint = text(&record, "fingerprint").unwrap().to_string();
        if retained.contains_key(&fingerprint) {
            continue;
        }
        if let Some(previous) = record.get("reassesses").and_then(Json::as_str) {
            if let Some(ancestor) = reviews
                .iter()
                .find(|value| value.get("fingerprint").and_then(Json::as_str) == Some(previous))
            {
                pending.push(ancestor.clone());
            }
        }
        retained.insert(fingerprint, record);
    }
    retained.into_values().collect()
}

fn requirement_review_input(
    model: &Model,
    project: &str,
    request: &Json,
    reviews: &[Json],
    bundles: &Runs,
) -> Result<Json, String> {
    object(request, &["kind", "requirement"])?;
    let requirement = text(request, "requirement")?;
    let authority = authority(model, project)?;
    let deps = requirement_dependencies(&authority, requirement)?;
    let check_ids = array(&deps, "checks")?
        .iter()
        .map(|v| text(v, "id").unwrap().to_string())
        .collect();
    let scenario_ids = array(&deps, "scenarios")?
        .iter()
        .map(|v| text(v, "id").unwrap().to_string())
        .collect();
    let requirement_ids = [requirement.to_string()].into_iter().collect();
    for review in reviews {
        validate_record(review)?;
    }
    let heads = bundles.active_heads()?;
    let mut challenge_results = Vec::new();
    let mut challenge_issues = Vec::new();
    let mut challenge_bundles = Vec::new();
    let mut evidence_bundles = Vec::new();
    let mut check_results = Vec::new();
    let mut verification_targets = Vec::new();
    for bundle in &heads {
        let serialized = crate::run::to_json(bundle);
        let mut relevant_challenge = false;
        for execution in &bundle.verification_challenge_executions {
            let relevant = if execution.target.kind == ChallengeTargetKind::Implementation {
                array(&deps, "requirements")?
                    .iter()
                    .flat_map(|item| {
                        item.get("verification_challenges")
                            .and_then(Json::as_array)
                            .unwrap_or(&[])
                    })
                    .any(|challenge| {
                        challenge.get("id").and_then(Json::as_str)
                            == Some(execution.verification_challenge.id.as_str())
                    })
                    && implementation_challenge_current(&authority, bundle, execution)?
            } else {
                execution.target.id == requirement
                    && execution.target.kind == ChallengeTargetKind::RequirementReview
            };
            if !relevant {
                continue;
            }
            relevant_challenge = true;
            let wire = serialized
                .get("verification_challenge_executions")
                .and_then(Json::as_array)
                .unwrap()
                .iter()
                .find(|wire| {
                    wire.get("challenge").and_then(Json::as_str)
                        == Some(execution.challenge.as_str())
                })
                .unwrap();
            challenge_results.push(Json::obj(vec![
                ("run", Json::str(&bundle.run_id)),
                ("bundle_fingerprint", Json::str(&bundle.bundle_fingerprint)),
                (
                    "execution_verification_target_fingerprint",
                    Json::str(&bundle.verification_target_fingerprint),
                ),
                ("execution", wire.clone()),
                ("plan", serialized.get("plan").unwrap().clone()),
                ("artifacts", serialized.get("artifacts").unwrap().clone()),
            ]));
            for objection in &execution.result.objections {
                challenge_issues.push(Json::obj(vec![
                    ("id", Json::str(challenge_issue_id(execution, objection))),
                    (
                        "verification_challenge",
                        Json::str(&execution.verification_challenge.id),
                    ),
                    (
                        "target_fingerprint",
                        Json::str(&execution.target.fingerprint),
                    ),
                    (
                        "result_fingerprint",
                        Json::str(&execution.result.fingerprint),
                    ),
                    ("objection", Json::str(objection)),
                ]));
            }
        }
        let mut relevant_check = false;
        for execution in serialized
            .get("check_executions")
            .and_then(Json::as_array)
            .unwrap_or(&[])
        {
            let check = execution
                .get("check")
                .ok_or("Check execution requires Check reference")?;
            if !array(&deps, "checks")?.iter().any(|definition| {
                definition.get("id") == check.get("id")
                    && definition.get("fingerprint") == check.get("fingerprint")
            }) {
                continue;
            }
            relevant_check = true;
            check_results.push(Json::obj(vec![
                ("run", Json::str(&bundle.run_id)),
                ("bundle_fingerprint", Json::str(&bundle.bundle_fingerprint)),
                (
                    "verification_target_fingerprint",
                    Json::str(&bundle.verification_target_fingerprint),
                ),
                ("execution", execution.clone()),
            ]));
        }
        if relevant_check || relevant_challenge {
            let target = serialized.get("verification_target").unwrap().clone();
            if !verification_targets.contains(&target) {
                verification_targets.push(target);
            }
            let closure = bundles
                .bundles
                .iter()
                .filter(|item| item.run_id == bundle.run_id)
                .map(|item| crate::run::to_json(item))
                .collect::<Vec<_>>();
            for retained in closure {
                let destination = if relevant_challenge {
                    &mut challenge_bundles
                } else {
                    &mut evidence_bundles
                };
                if !destination.contains(&retained) {
                    destination.push(retained);
                }
            }
        }
    }
    let record = latest_review(reviews, request)?;
    let state = record
        .map(|r| assess_record_in(r, &authority, reviews, bundles))
        .unwrap_or_else(|| assessed("missing", "Review absent"));
    let status = Json::obj(vec![
        ("target", request.clone()),
        ("record_status", Json::str(&state.status)),
        (
            "record_fingerprint",
            record
                .and_then(|r| r.get("fingerprint"))
                .cloned()
                .unwrap_or(Json::Null),
        ),
        (
            "diagnostics",
            Json::Arr(state.diagnostics.iter().map(Json::str).collect()),
        ),
    ]);
    let gaps = if state.status == "accepted" {
        Vec::new()
    } else {
        vec![status.clone()]
    };
    let history = reassessment_history(record.into_iter().cloned().collect(), reviews);
    let review_issues = history
        .iter()
        .flat_map(|previous| {
            previous
                .get("issues")
                .and_then(Json::as_array)
                .unwrap_or(&[])
                .iter()
                .map(|issue| {
                    Json::obj(vec![
                        (
                            "reference",
                            Json::str(format!(
                                "{}#{}",
                                text(previous, "fingerprint").unwrap(),
                                text(issue, "id").unwrap()
                            )),
                        ),
                        ("review", previous.get("fingerprint").unwrap().clone()),
                        ("issue", issue.clone()),
                    ])
                })
        })
        .collect();
    let mut definitions = review_definitions(model, &check_ids, &scenario_ids, &requirement_ids);
    if let Json::Obj(fields) = &mut definitions {
        fields.push(("challenge_results".into(), Json::Arr(challenge_results)));
        fields.push(("challenge_issues".into(), Json::Arr(challenge_issues)));
        fields.push(("review_issues".into(), Json::Arr(review_issues)));
    }
    // The packet keeps the full dependency definitions for the reviewer; the authored record pins
    // only their canonical fingerprint.
    let mut output = Json::obj(vec![
        ("format", Json::str("azimuth-review-input")),
        ("version", Json::Num(2.0)),
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
        ("dependencies", deps),
        ("definitions", definitions),
        ("verification_targets", Json::Arr(verification_targets)),
        ("check_results", Json::Arr(check_results)),
        ("evidence_bundles", Json::Arr(evidence_bundles)),
        ("challenge_bundles", Json::Arr(challenge_bundles)),
        ("review_states", Json::Arr(vec![status])),
        ("existing_reviews", Json::Arr(history)),
        ("gaps", Json::Arr(gaps)),
    ]);
    let fp = record_fingerprint(&output)?;
    if let Json::Obj(fields) = &mut output {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    Ok(output)
}
