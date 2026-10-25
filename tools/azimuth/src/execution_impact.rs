use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct Action {
    fingerprint: String,
    dependencies: BTreeSet<String>,
    complete: bool,
}

struct Snapshot {
    target: Json,
    target_kind: String,
    target_fingerprint: String,
    context: BTreeMap<String, String>,
    complete: bool,
    inputs: BTreeMap<String, String>,
    shared: BTreeSet<String>,
    checks: BTreeMap<String, Action>,
    challenges: BTreeMap<String, Action>,
}

pub fn plan_affected(input: &Json) -> Result<Json, String> {
    exact(
        input,
        &["format", "version", "baseline", "candidate", "pending"],
        "request",
    )?;
    if string(input, "format", "request")? != "azimuth-affected-execution-request"
        || input.get("version").and_then(Json::as_num) != Some(1.0)
    {
        return Err("expected azimuth-affected-execution-request version 1".into());
    }
    let baseline = match field(input, "baseline", "request")? {
        Json::Null => None,
        value => Some(snapshot(value, "baseline")?),
    };
    let candidate = snapshot(field(input, "candidate", "request")?, "candidate")?;
    let pending = field(input, "pending", "request")?;
    exact(pending, &["checks", "challenges"], "pending")?;
    let pending_checks = strings(field(pending, "checks", "pending")?, "pending.checks")?;
    let pending_challenges = strings(
        field(pending, "challenges", "pending")?,
        "pending.challenges",
    )?;
    let mut changed = BTreeSet::new();
    let mut widening = BTreeSet::new();
    let mut diagnostics = BTreeSet::new();
    if !candidate.complete {
        widening.insert("candidate-inventory-incomplete".to_string());
        diagnostics.insert("candidate inventory is incomplete; declared work cannot establish the required inventory".to_string());
    }
    if let Some(previous) = &baseline {
        for id in previous.inputs.keys().chain(candidate.inputs.keys()) {
            if previous.inputs.get(id) != candidate.inputs.get(id) {
                changed.insert(id.clone());
            }
        }
        if !previous.complete {
            widening.insert("baseline-inventory-incomplete".to_string());
        }
        if previous.context != candidate.context {
            widening.insert("execution-context-changed".to_string());
        }
        if previous.shared != candidate.shared {
            widening.insert("shared-input-declaration-changed".to_string());
        }
        if previous.target_kind != candidate.target_kind
            || (candidate.target_kind != "source"
                && previous.target_fingerprint != candidate.target_fingerprint)
        {
            widening.insert("exact-target-changed".to_string());
        }
        let consumers: BTreeSet<_> = previous
            .checks
            .values()
            .chain(previous.challenges.values())
            .chain(candidate.checks.values())
            .chain(candidate.challenges.values())
            .flat_map(|action| action.dependencies.iter().cloned())
            .collect();
        for id in &changed {
            if previous.shared.contains(id) || candidate.shared.contains(id) {
                widening.insert(format!("shared-input-changed:{id}"));
            } else if !consumers.contains(id) {
                widening.insert(format!("changed-input-without-consumer:{id}"));
            }
        }
    } else {
        changed.extend(candidate.inputs.keys().cloned());
        widening.insert("baseline-unavailable".to_string());
    }
    for (name, current) in [("candidate", &candidate)]
        .into_iter()
        .chain(baseline.as_ref().map(|value| ("baseline", value)))
    {
        for (kind, actions) in [
            ("check", &current.checks),
            ("challenge", &current.challenges),
        ] {
            for (id, action) in actions {
                if !action.complete || action.dependencies.is_empty() {
                    widening.insert(format!("dependency-inventory-unknown:{name}:{kind}:{id}"));
                }
                for dependency in &action.dependencies {
                    if !current.inputs.contains_key(dependency) {
                        widening.insert(format!("dependency-input-missing:{name}:{dependency}"));
                        if name == "candidate" {
                            diagnostics.insert(format!(
                                "{kind} {id} depends on missing candidate input {dependency}"
                            ));
                        }
                    }
                }
            }
        }
        for id in &current.shared {
            if !current.inputs.contains_key(id) {
                widening.insert(format!("shared-input-missing:{name}:{id}"));
                if name == "candidate" {
                    diagnostics.insert(format!("shared candidate input {id} is missing"));
                }
            }
        }
    }
    let (checks, selected_checks, removed_checks) = select(
        "check",
        &candidate.checks,
        baseline.as_ref().map(|s| &s.checks),
        &pending_checks,
        &changed,
        &widening,
        &mut diagnostics,
    );
    let (challenges, selected_challenges, removed_challenges) = select(
        "challenge",
        &candidate.challenges,
        baseline.as_ref().map(|s| &s.challenges),
        &pending_challenges,
        &changed,
        &widening,
        &mut diagnostics,
    );
    let mut result = Json::obj(vec![
        ("format", Json::str("azimuth-affected-execution")),
        ("version", Json::Num(1.0)),
        (
            "status",
            Json::str(if diagnostics.is_empty() {
                "complete"
            } else {
                "incomplete"
            }),
        ),
        (
            "request_fingerprint",
            Json::str(crate::run::canonical_fingerprint(input)?),
        ),
        (
            "baseline_verification_target",
            baseline.as_ref().map_or(Json::Null, |s| s.target.clone()),
        ),
        (
            "baseline_verification_target_fingerprint",
            baseline
                .as_ref()
                .map_or(Json::Null, |s| Json::str(&s.target_fingerprint)),
        ),
        ("verification_target", candidate.target),
        (
            "verification_target_fingerprint",
            Json::str(candidate.target_fingerprint),
        ),
        ("changed_inputs", json_strings(&changed)),
        ("widening", json_strings(&widening)),
        ("checks", Json::Arr(checks)),
        ("challenges", Json::Arr(challenges)),
        (
            "selected",
            Json::obj(vec![
                ("checks", json_strings(&selected_checks)),
                ("challenges", json_strings(&selected_challenges)),
            ]),
        ),
        (
            "removed",
            Json::obj(vec![
                ("checks", json_strings(&removed_checks)),
                ("challenges", json_strings(&removed_challenges)),
            ]),
        ),
        ("diagnostics", json_strings(&diagnostics)),
    ]);
    let fingerprint = crate::run::canonical_fingerprint(&result)?;
    if let Json::Obj(fields) = &mut result {
        fields.push(("fingerprint".into(), Json::str(fingerprint)));
    }
    Ok(result)
}

fn select(
    kind: &str,
    current: &BTreeMap<String, Action>,
    previous: Option<&BTreeMap<String, Action>>,
    pending: &BTreeSet<String>,
    changed: &BTreeSet<String>,
    widening: &BTreeSet<String>,
    diagnostics: &mut BTreeSet<String>,
) -> (Vec<Json>, BTreeSet<String>, BTreeSet<String>) {
    let mut entries = Vec::new();
    let mut selected = BTreeSet::new();
    for (id, action) in current {
        let mut reasons = widening.clone();
        let earlier = previous.and_then(|actions| actions.get(id));
        if pending.contains(id) {
            reasons.insert("pending-obligation".into());
        }
        if let Some(old) = earlier {
            if old.fingerprint != action.fingerprint {
                reasons.insert("definition-changed".into());
            }
            if old.dependencies != action.dependencies {
                reasons.insert("dependencies-changed".into());
            }
        } else {
            reasons.insert("action-added".into());
        }
        let dependencies: BTreeSet<_> = action
            .dependencies
            .iter()
            .chain(earlier.into_iter().flat_map(|old| old.dependencies.iter()))
            .filter(|id| changed.contains(*id))
            .cloned()
            .collect();
        if !dependencies.is_empty() {
            reasons.insert("dependency-input-changed".into());
        }
        let status = if reasons.is_empty() {
            reasons.insert("declared-inputs-unchanged".into());
            "unaffected"
        } else {
            selected.insert(id.clone());
            "selected"
        };
        entries.push(Json::obj(vec![
            ("id", Json::str(id)),
            ("status", Json::str(status)),
            ("definition_fingerprint", Json::str(&action.fingerprint)),
            ("reasons", json_strings(&reasons)),
            ("changed_dependencies", json_strings(&dependencies)),
        ]));
    }
    for id in pending.iter().filter(|id| !current.contains_key(*id)) {
        diagnostics.insert(format!(
            "pending {kind} {id} is absent from the candidate; explicit disposition is required"
        ));
        entries.push(Json::obj(vec![
            ("id", Json::str(id)),
            ("status", Json::str("missing")),
            ("definition_fingerprint", Json::Null),
            (
                "reasons",
                Json::Arr(vec![Json::str("pending-obligation-without-candidate")]),
            ),
            ("changed_dependencies", Json::Arr(Vec::new())),
        ]));
    }
    entries.sort_by_key(|entry| {
        entry
            .get("id")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string()
    });
    let removed = previous
        .into_iter()
        .flat_map(|items| items.keys())
        .filter(|id| !current.contains_key(*id))
        .cloned()
        .collect();
    (entries, selected, removed)
}

pub fn snapshot_from_model(
    model: &crate::model::Model,
    project: &str,
    verification_target: &Json,
    execution_context: &Json,
    shared_inputs: &Json,
) -> Result<Json, String> {
    let authority = crate::assurance_review::authority(model, project)?;
    let mut inputs = BTreeMap::new();
    let mut shared = BTreeSet::new();
    for entry in array(shared_inputs, "shared_inputs")? {
        exact(entry, &["id", "digest"], "shared input")?;
        let id = identity(string(entry, "id", "shared input")?, "shared input id")?;
        let digest = digest(
            string(entry, "digest", "shared input")?,
            "shared input digest",
        )?;
        if inputs.insert(id.clone(), digest).is_some() {
            return Err(format!("duplicate input {id}"));
        }
        shared.insert(id);
    }
    let mut requirements = BTreeMap::new();
    for requirement in array(
        field(&authority, "requirements", "authority")?,
        "authority.requirements",
    )? {
        let id = string(requirement, "id", "Requirement")?.to_string();
        let input = format!("requirement:{id}");
        insert_input(
            &mut inputs,
            &input,
            string(requirement, "fingerprint", "Requirement")?,
        )?;
        requirements.insert(
            id,
            (
                input,
                array(
                    field(requirement, "gaps", "Requirement")?,
                    "Requirement.gaps",
                )?
                .is_empty(),
            ),
        );
    }
    let mut source_inputs = BTreeSet::new();
    let target = crate::run::verification_target_from_json(verification_target)?;
    if let crate::run::VerificationTarget::Source { repositories } = &target {
        for repository in repositories {
            let input = format!("repository:{}", repository.id);
            insert_input(&mut inputs, &input, &repository.content_fingerprint)?;
            source_inputs.insert(input);
        }
    }
    let mut checks = Vec::new();
    for check in array(
        field(&authority, "checks", "authority")?,
        "authority.checks",
    )? {
        let id = string(check, "id", "Check")?;
        let fingerprint = string(check, "fingerprint", "Check")?;
        let own = format!("check:{id}");
        insert_input(&mut inputs, &own, fingerprint)?;
        let mut dependencies = source_inputs.clone();
        dependencies.insert(own);
        let mut complete = true;
        for requirement in array(
            field(&authority, "requirements", "authority")?,
            "authority.requirements",
        )? {
            if array(
                field(requirement, "bindings", "Requirement")?,
                "Requirement.bindings",
            )?
            .iter()
            .any(|binding| binding.get("check").and_then(Json::as_str) == Some(id))
            {
                let (input, available) = requirements
                    .get(string(requirement, "id", "Requirement")?)
                    .ok_or("Requirement index missing")?;
                dependencies.insert(input.clone());
                complete &= available;
            }
        }
        for binding in array(
            field(&authority, "bindings", "authority")?,
            "authority.bindings",
        )? {
            if binding.get("check").and_then(Json::as_str) == Some(id) {
                let input = format!("binding:{id}:{}", string(binding, "scenario", "binding")?);
                insert_input(
                    &mut inputs,
                    &input,
                    string(binding, "fingerprint", "binding")?,
                )?;
                dependencies.insert(input);
            }
        }
        checks.push(action_json(id, fingerprint, &dependencies, complete));
    }
    let mut challenges = Vec::new();
    for challenge in model.verification_challenges() {
        let id = &challenge.id;
        let definition = crate::fingerprint::verification_challenge_fingerprint(challenge);
        let own = format!("challenge:{id}");
        insert_input(&mut inputs, &own, &definition)?;
        let mut dependencies = source_inputs.clone();
        dependencies.insert(own);
        let mut complete = true;
        for requirement in &challenge.requirements {
            if let Some((input, available)) = requirements.get(requirement) {
                dependencies.insert(input.clone());
                complete &= available;
            } else {
                complete = false;
            }
        }
        for check in &challenge.checks {
            let input = format!("check:{check}");
            complete &= inputs.contains_key(&input);
            dependencies.insert(input);
        }
        let producers: Vec<_> = model
            .package_producers
            .iter()
            .filter(|producer| {
                producer.package == "azimuth.challenge"
                    && producer.contract == "verification-challenge-implementation"
                    && producer.entity == *id
            })
            .map(|producer| producer.to_json())
            .collect();
        if producers.is_empty() {
            complete = false;
        }
        let producer = format!("challenge-implementation:{id}");
        insert_input(
            &mut inputs,
            &producer,
            &crate::run::canonical_fingerprint(&Json::Arr(producers))?,
        )?;
        dependencies.insert(producer);
        challenges.push(action_json(id, &definition, &dependencies, complete));
    }
    checks.sort_by_key(|entry| {
        entry
            .get("id")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string()
    });
    challenges.sort_by_key(|entry| {
        entry
            .get("id")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string()
    });
    let result = Json::obj(vec![
        ("verification_target", verification_target.clone()),
        ("execution_context", execution_context.clone()),
        ("inventory_complete", Json::Bool(true)),
        (
            "inputs",
            Json::Arr(
                inputs
                    .iter()
                    .map(|(id, digest)| {
                        Json::obj(vec![("id", Json::str(id)), ("digest", Json::str(digest))])
                    })
                    .collect(),
            ),
        ),
        ("shared_inputs", json_strings(&shared)),
        ("checks", Json::Arr(checks)),
        ("challenges", Json::Arr(challenges)),
    ]);
    snapshot(&result, "derived snapshot")?;
    Ok(result)
}

pub fn snapshot_from_model_with_units(
    model: &crate::model::Model,
    project: &str,
    verification_target: &Json,
    execution_context: &Json,
    shared_inputs: &Json,
    units: &crate::unit_inputs::UnitInputs,
) -> Result<Json, String> {
    let mut result = snapshot_from_model(
        model,
        project,
        verification_target,
        execution_context,
        shared_inputs,
    )?;
    let target = crate::run::verification_target_from_json(verification_target)?;
    let crate::run::VerificationTarget::Source { repositories } = target else {
        return Err("unit-derived affected selection requires a source Verification Target".into());
    };
    if repositories.len() != 1 {
        return Err("unit-derived affected selection requires one repository; multi-repository unit scope must be explicit".into());
    }
    let repository_input = format!("repository:{}", repositories[0].id);
    let mut added = BTreeMap::new();
    if let Json::Obj(fields) = &mut result {
        if let Some((_, Json::Arr(checks))) = fields.iter_mut().find(|(key, _)| key == "checks") {
            for check in checks {
                let id = string(check, "id", "Check")?.to_string();
                let applicable: Vec<_> = units
                    .units
                    .iter()
                    .filter(|unit| unit.checks.is_empty() || unit.checks.contains(&id))
                    .collect();
                if applicable.is_empty() || applicable.iter().any(|unit| !unit.hermetic) {
                    continue;
                }
                let mut dependencies =
                    strings(field(check, "dependencies", "Check")?, "Check.dependencies")?;
                dependencies.remove(&repository_input);
                for unit in applicable {
                    let input = format!("unit:{}:{}", repositories[0].id, unit.id);
                    let fingerprint = digest(&unit.input_fingerprint, "unit input fingerprint")?;
                    if let Some(previous) = added.insert(input.clone(), fingerprint.clone()) {
                        if previous != fingerprint {
                            return Err(format!("conflicting unit input {input}"));
                        }
                    }
                    dependencies.insert(input);
                }
                if let Json::Obj(values) = check {
                    values
                        .iter_mut()
                        .find(|(key, _)| key == "dependencies")
                        .unwrap()
                        .1 = json_strings(&dependencies);
                }
            }
        }
        let source_required = fields
            .iter()
            .filter(|(key, _)| key == "checks" || key == "challenges")
            .filter_map(|(_, value)| value.as_array())
            .flatten()
            .any(|action| {
                action
                    .get("dependencies")
                    .and_then(Json::as_array)
                    .is_some_and(|dependencies| {
                        dependencies
                            .iter()
                            .any(|item| item.as_str() == Some(&repository_input))
                    })
            });
        if let Some((_, Json::Arr(inputs))) = fields.iter_mut().find(|(key, _)| key == "inputs") {
            if !source_required {
                inputs.retain(|input| {
                    input.get("id").and_then(Json::as_str) != Some(&repository_input)
                });
            }
            for (id, digest) in added {
                if inputs
                    .iter()
                    .any(|input| input.get("id").and_then(Json::as_str) == Some(&id))
                {
                    return Err(format!("input identity collision {id}"));
                }
                inputs.push(Json::obj(vec![
                    ("id", Json::str(id)),
                    ("digest", Json::str(digest)),
                ]));
            }
            inputs.sort_by_key(|input| {
                input
                    .get("id")
                    .and_then(Json::as_str)
                    .unwrap_or_default()
                    .to_string()
            });
        }
    }
    snapshot(&result, "unit-derived snapshot")?;
    Ok(result)
}

fn action_json(
    id: &str,
    fingerprint: &str,
    dependencies: &BTreeSet<String>,
    complete: bool,
) -> Json {
    Json::obj(vec![
        ("id", Json::str(id)),
        ("definition_fingerprint", Json::str(fingerprint)),
        ("dependencies", json_strings(dependencies)),
        ("dependencies_complete", Json::Bool(complete)),
    ])
}

fn insert_input(
    inputs: &mut BTreeMap<String, String>,
    id: &str,
    fingerprint: &str,
) -> Result<(), String> {
    let fingerprint = digest(fingerprint, id)?;
    if inputs.insert(id.to_string(), fingerprint).is_some() {
        return Err(format!("input identity collision {id}"));
    }
    Ok(())
}

fn snapshot(value: &Json, name: &str) -> Result<Snapshot, String> {
    exact(
        value,
        &[
            "verification_target",
            "execution_context",
            "inventory_complete",
            "inputs",
            "shared_inputs",
            "checks",
            "challenges",
        ],
        name,
    )?;
    let target =
        crate::run::verification_target_from_json(field(value, "verification_target", name)?)?;
    let errors = crate::run::validate_verification_target_component(&target);
    if !errors.is_empty() {
        return Err(format!("{name}: invalid Verification Target: {errors:?}"));
    }
    let target_json = crate::run::verification_target_to_json(&target);
    let target_kind = string(&target_json, "kind", name)?.to_string();
    let mut context = BTreeMap::new();
    let Json::Obj(fields) = field(value, "execution_context", name)? else {
        return Err(format!("{name}.execution_context must be an object"));
    };
    for (key, item) in fields {
        identity(key, "execution context key")?;
        let text = item
            .as_str()
            .ok_or_else(|| format!("execution context {key} must be a string"))?;
        if context.insert(key.clone(), text.to_string()).is_some() {
            return Err(format!("duplicate execution context {key}"));
        }
    }
    let mut inputs = BTreeMap::new();
    for entry in array(field(value, "inputs", name)?, name)? {
        exact(entry, &["id", "digest"], "input")?;
        let id = identity(string(entry, "id", "input")?, "input id")?;
        let fingerprint = digest(string(entry, "digest", "input")?, "input digest")?;
        if inputs.insert(id.clone(), fingerprint).is_some() {
            return Err(format!("duplicate input {id}"));
        }
    }
    Ok(Snapshot {
        target: target_json,
        target_kind,
        target_fingerprint: crate::run::verification_target_fingerprint(&target),
        context,
        complete: boolean(value, "inventory_complete", name)?,
        inputs,
        shared: strings(field(value, "shared_inputs", name)?, "shared_inputs")?,
        checks: actions(field(value, "checks", name)?, "checks")?,
        challenges: actions(field(value, "challenges", name)?, "challenges")?,
    })
}

fn actions(value: &Json, name: &str) -> Result<BTreeMap<String, Action>, String> {
    let mut actions = BTreeMap::new();
    for entry in array(value, name)? {
        exact(
            entry,
            &[
                "id",
                "definition_fingerprint",
                "dependencies",
                "dependencies_complete",
            ],
            name,
        )?;
        let id = identity(string(entry, "id", name)?, "action id")?;
        let action = Action {
            fingerprint: digest(
                string(entry, "definition_fingerprint", name)?,
                "definition fingerprint",
            )?,
            dependencies: strings(field(entry, "dependencies", name)?, "dependencies")?,
            complete: boolean(entry, "dependencies_complete", name)?,
        };
        if actions.insert(id.clone(), action).is_some() {
            return Err(format!("duplicate {name} id {id}"));
        }
    }
    Ok(actions)
}

fn exact(value: &Json, names: &[&str], at: &str) -> Result<(), String> {
    let Json::Obj(fields) = value else {
        return Err(format!("{at} must be an object"));
    };
    let mut seen = BTreeSet::new();
    for (key, _) in fields {
        if !names.contains(&key.as_str()) || !seen.insert(key) {
            return Err(format!("{at}: unknown or duplicate field {key}"));
        }
    }
    for key in names {
        if !seen.contains(&key.to_string()) {
            return Err(format!("{at}: missing field {key}"));
        }
    }
    Ok(())
}
fn field<'a>(value: &'a Json, key: &str, at: &str) -> Result<&'a Json, String> {
    value
        .get(key)
        .ok_or_else(|| format!("{at}: missing field {key}"))
}
fn string<'a>(value: &'a Json, key: &str, at: &str) -> Result<&'a str, String> {
    field(value, key, at)?
        .as_str()
        .ok_or_else(|| format!("{at}.{key} must be a string"))
}
fn boolean(value: &Json, key: &str, at: &str) -> Result<bool, String> {
    field(value, key, at)?
        .as_bool()
        .ok_or_else(|| format!("{at}.{key} must be a boolean"))
}
fn array<'a>(value: &'a Json, at: &str) -> Result<&'a [Json], String> {
    value
        .as_array()
        .ok_or_else(|| format!("{at} must be an array"))
}
fn identity(value: &str, at: &str) -> Result<String, String> {
    if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(format!(
            "{at} must be a nonempty identity without whitespace"
        ));
    }
    Ok(value.to_string())
}
fn digest(value: &str, at: &str) -> Result<String, String> {
    let raw = value.strip_prefix("sha256:").unwrap_or_default();
    if raw.len() != 64
        || !raw
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(format!("{at} must be lowercase sha256"));
    }
    Ok(value.to_string())
}
fn strings(value: &Json, at: &str) -> Result<BTreeSet<String>, String> {
    let mut result = BTreeSet::new();
    for item in array(value, at)? {
        let id = identity(
            item.as_str()
                .ok_or_else(|| format!("{at} entries must be strings"))?,
            at,
        )?;
        if !result.insert(id.clone()) {
            return Err(format!("{at}: duplicate identity {id}"));
        }
    }
    Ok(result)
}
fn json_strings(values: &BTreeSet<String>) -> Json {
    Json::Arr(values.iter().map(Json::str).collect())
}
