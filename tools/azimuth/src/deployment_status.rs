//! Deployment evidence applicability and deterministic Claim Status.
use crate::json::Json;
use crate::run::{ObservationOutcome, RunBundle, RunStatus, Subject};
use std::collections::{BTreeMap, BTreeSet};

const MAX_INTEGER: u64 = 9_007_199_254_740_991;
const CLOCK_SKEW_MS: u64 = 5_000;

fn field<'a>(v: &'a Json, k: &str) -> Result<&'a Json, String> {
    v.get(k).ok_or_else(|| format!("missing `{k}`"))
}
fn text<'a>(v: &'a Json, k: &str) -> Result<&'a str, String> {
    field(v, k)?
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("`{k}` requires nonempty string"))
}
fn array<'a>(v: &'a Json, k: &str) -> Result<&'a [Json], String> {
    field(v, k)?
        .as_array()
        .ok_or_else(|| format!("`{k}` requires array"))
}
fn number(v: &Json, k: &str) -> Result<u64, String> {
    let n = field(v, k)?
        .as_num()
        .ok_or_else(|| format!("`{k}` requires integer"))?;
    if !n.is_finite() || n < 0.0 || n > MAX_INTEGER as f64 || n.fract() != 0.0 {
        return Err(format!("`{k}` requires nonnegative safe integer"));
    }
    Ok(n as u64)
}
fn shape(v: &Json, required: &[&str]) -> Result<(), String> {
    let Json::Obj(items) = v else {
        return Err("expected object".into());
    };
    let mut seen = BTreeSet::new();
    for (k, _) in items {
        if !required.contains(&k.as_str()) || !seen.insert(k) {
            return Err(format!("unexpected or duplicate field `{k}`"));
        }
    }
    for k in required {
        field(v, k)?;
    }
    Ok(())
}
fn digest(v: &str) -> Result<(), String> {
    let s = v
        .strip_prefix("sha256:")
        .ok_or("expected sha256 fingerprint")?;
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("expected lowercase sha256 fingerprint".into());
    }
    Ok(())
}
fn strings(v: &Json, k: &str) -> Result<BTreeSet<String>, String> {
    let mut values = BTreeSet::new();
    for item in array(v, k)? {
        let s = item
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| format!("`{k}` requires nonempty strings"))?;
        if !values.insert(s.to_string()) {
            return Err(format!("duplicate `{k}` entry"));
        }
    }
    Ok(values)
}
fn canonical(v: &Json) -> Result<String, String> {
    crate::run::canonical_fingerprint(v)
}
fn same(a: &Json, b: &Json) -> bool {
    matches!((canonical(a), canonical(b)), (Ok(left), Ok(right)) if left == right)
}
fn trusted(v: &Json) -> Result<bool, String> {
    field(v, "trusted")?
        .as_bool()
        .ok_or("trusted requires boolean".into())
}

pub fn validate_policy(v: &Json) -> Result<(), String> {
    let mut fields = vec![
        "subject",
        "components",
        "contexts",
        "contribution",
        "monitor",
    ];
    for optional in ["units", "controls", "configuration_sources"] {
        if v.get(optional).is_some() {
            fields.push(optional);
        }
    }
    shape(v, &fields)?;
    if v.get("configuration_sources").is_some() {
        if text(v, "subject")? != "source" {
            return Err("configuration sources require source execution".into());
        }
        for repository in strings(v, "configuration_sources")? {
            crate::diag::validate_id(&repository, false)?;
        }
    }
    let units = if v.get("units").is_some() {
        strings(v, "units")?
    } else {
        BTreeSet::new()
    };
    if v.get("units").is_some() && units.is_empty() {
        return Err("required unit inventory cannot be empty".into());
    }
    if v.get("controls").is_some() {
        let mut controlled = BTreeSet::new();
        for control in array(v, "controls")? {
            shape(control, &["unit", "contexts"])?;
            let unit = text(control, "unit")?;
            let contexts = strings(control, "contexts")?;
            if !units.contains(unit)
                || !controlled.insert(unit)
                || contexts.is_empty()
                || !contexts.is_subset(&strings(v, "contexts")?)
            {
                return Err("controls require unique declared units and required contexts".into());
            }
        }
    }
    if !matches!(text(v, "subject")?, "source" | "artifact" | "deployment") {
        return Err("unsupported deployment Check subject".into());
    }
    let components = strings(v, "components")?;
    if components.is_empty() {
        return Err("deployment Check requires component scope".into());
    }
    for id in components {
        crate::diag::validate_id(&id, false)?;
    }
    for fp in strings(v, "contexts")? {
        digest(&fp)?;
    }
    if !matches!(
        text(v, "contribution")?,
        "support" | "counterexample" | "both"
    ) {
        return Err("unknown deployment Check contribution".into());
    }
    let monitor = field(v, "monitor")?;
    if *monitor != Json::Null {
        shape(monitor, &["id", "interval_seconds", "freshness_seconds"])?;
        crate::diag::validate_id(text(monitor, "id")?, false)?;
        let interval = number(monitor, "interval_seconds")?;
        let freshness = number(monitor, "freshness_seconds")?;
        if interval == 0 || freshness < interval || freshness > MAX_INTEGER / 1000 {
            return Err("Monitor freshness must cover its positive interval".into());
        }
        if text(v, "subject")? != "deployment" {
            return Err("Monitor requires deployment execution Subject".into());
        }
    }
    Ok(())
}

pub fn validate_context_contract(v: &Json) -> Result<(), String> {
    shape(v, &["name", "fields", "fingerprint"])?;
    crate::diag::validate_id(text(v, "name")?, false)?;
    digest(text(v, "fingerprint")?)?;
    let Json::Obj(fields) = field(v, "fields")? else {
        return Err("context fields requires object".into());
    };
    if fields.is_empty() {
        return Err("context contract requires fields".into());
    }
    let mut seen = BTreeSet::new();
    for (name, ty) in fields {
        if name.trim().is_empty()
            || !seen.insert(name)
            || !matches!(ty.as_str(), Some("string" | "boolean" | "integer"))
        {
            return Err("invalid context field declaration".into());
        }
        let lower = name.to_ascii_lowercase();
        if [
            "password",
            "secret",
            "credential",
            "token",
            "privatekey",
            "private_key",
        ]
        .iter()
        .any(|s| lower.contains(s))
        {
            return Err("secret-bearing context field is forbidden".into());
        }
    }
    let body = Json::obj(vec![
        ("name", field(v, "name")?.clone()),
        ("fields", field(v, "fields")?.clone()),
    ]);
    if canonical(&body)? != text(v, "fingerprint")? {
        return Err("context contract fingerprint mismatch".into());
    }
    Ok(())
}

pub fn validate_context_report(report: &Json, contracts: &[Json]) -> Result<(), String> {
    shape(report, &["format", "version", "contexts"])?;
    if text(report, "format")? != "azimuth-verification-context" || number(report, "version")? != 1
    {
        return Err("unsupported verification context report".into());
    }
    let Json::Obj(contexts) = field(report, "contexts")? else {
        return Err("contexts requires object".into());
    };
    let mut names = BTreeSet::new();
    for (name, context) in contexts {
        if !names.insert(name) {
            return Err("duplicate context name".into());
        }
        shape(context, &["contract_revision", "values"])?;
        let fp = text(context, "contract_revision")?;
        digest(fp)?;
        let contract = contracts
            .iter()
            .find(|v| {
                v.get("fingerprint").and_then(Json::as_str) == Some(fp)
                    && v.get("name").and_then(Json::as_str) == Some(name)
            })
            .ok_or("context contract is not registered")?;
        validate_context_contract(contract)?;
        let Json::Obj(fields) = field(contract, "fields")? else {
            unreachable!()
        };
        let Json::Obj(values) = field(context, "values")? else {
            return Err("context values requires object".into());
        };
        let expected = fields.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>();
        shape(field(context, "values")?, &expected)?;
        for (key, ty) in fields {
            let value = values.iter().find(|(k, _)| k == key).unwrap().1.clone();
            let valid = match ty.as_str().unwrap() {
                "string" => value.as_str().is_some(),
                "boolean" => value.as_bool().is_some(),
                "integer" => number(field(context, "values")?, key).is_ok(),
                _ => false,
            };
            if !valid {
                return Err(format!("context field `{key}` differs from declared type"));
            }
        }
    }
    Ok(())
}

fn validate_snapshot(v: &Json, contracts: &[Json]) -> Result<(), String> {
    shape(v, &["captured_at_ms", "instance", "report"])?;
    number(v, "captured_at_ms")?;
    text(v, "instance")?;
    validate_context_report(field(v, "report")?, contracts)
}

pub fn validate_capture(v: &Json, contracts: &[Json]) -> Result<(), String> {
    shape(
        v,
        &[
            "kind",
            "run",
            "activity",
            "bundle_fingerprint",
            "subject_fingerprint",
            "component",
            "instance",
            "started_at_ms",
            "finished_at_ms",
            "before",
            "after",
            "trusted",
        ],
    )?;
    let execution = match text(v, "kind")? {
        "execution" => true,
        "deployment" => false,
        _ => return Err("unknown context capture kind".into()),
    };
    for key in ["run", "bundle_fingerprint"] {
        if execution {
            digest(text(v, key)?)?;
        } else if *field(v, key)? != Json::Null {
            return Err("deployment capture cannot declare Run identity".into());
        }
    }
    if *field(v, "activity")? != Json::Null {
        if !execution {
            return Err("deployment capture cannot declare activity".into());
        }
        text(v, "activity")?;
    }
    digest(text(v, "subject_fingerprint")?)?;
    crate::diag::validate_id(text(v, "component")?, false)?;
    text(v, "instance")?;
    trusted(v)?;
    let start = number(v, "started_at_ms")?;
    let finish = number(v, "finished_at_ms")?;
    if finish < start {
        return Err("context capture interval is reversed".into());
    }
    for (key, time) in [("before", start), ("after", finish)] {
        let snapshot = field(v, key)?;
        validate_snapshot(snapshot, contracts)?;
        if number(snapshot, "captured_at_ms")? != time
            || snapshot.get("instance") != v.get("instance")
        {
            return Err("capture interval and process-lifetime snapshots differ".into());
        }
    }
    Ok(())
}

struct Inputs<'a> {
    release: &'a Json,
    deployment: &'a Json,
    observation: &'a Json,
    contracts: &'a [Json],
    captures: &'a [Json],
    now: u64,
    subject: String,
    components: BTreeMap<String, String>,
    deployment_reasons: Vec<String>,
}

fn validate_observation(v: &Json) -> Result<(), String> {
    shape(
        v,
        &[
            "subject_fingerprint",
            "observed_at_ms",
            "valid_for_seconds",
            "state",
            "components",
            "trusted",
        ],
    )?;
    digest(text(v, "subject_fingerprint")?)?;
    number(v, "observed_at_ms")?;
    let seconds = number(v, "valid_for_seconds")?;
    if seconds == 0 || seconds > MAX_INTEGER / 1000 {
        return Err("invalid deployment observation lifetime".into());
    }
    if !matches!(text(v, "state")?, "complete" | "partial" | "unknown") {
        return Err("unknown deployment observation state".into());
    }
    trusted(v)?;
    let mut ids = BTreeSet::new();
    let mut instances = BTreeSet::new();
    for component in array(v, "components")? {
        shape(component, &["id", "digest", "instances"])?;
        crate::diag::validate_id(text(component, "id")?, false)?;
        if !ids.insert(text(component, "id")?) {
            return Err("duplicate observed component".into());
        }
        digest(text(component, "digest")?)?;
        for instance in strings(component, "instances")? {
            if !instances.insert(instance) {
                return Err("process lifetime belongs to multiple components".into());
            }
        }
    }
    Ok(())
}

impl<'a> Inputs<'a> {
    fn new(request: &'a Json) -> Result<Self, String> {
        shape(
            request,
            &[
                "format",
                "version",
                "publication",
                "deployment",
                "deployment_observation",
                "context_contracts",
                "captures",
                "bundles",
                "reviews",
                "evaluated_at_ms",
            ],
        )?;
        if text(request, "format")? != "azimuth-deployment-assessment"
            || number(request, "version")? != 1
        {
            return Err("unsupported deployment assessment request".into());
        }
        crate::release::verify(field(request, "publication")?)?;
        let release = field(field(request, "publication")?, "release")?;
        let deployment = field(request, "deployment")?;
        let artifacts = array(release, "artifacts")?
            .iter()
            .map(|a| text(a, "digest").map(str::to_string))
            .collect::<Result<BTreeSet<_>, _>>()?;
        crate::release::verify_deployment(deployment, &artifacts)?;
        let subject = text(deployment, "subject_fingerprint")?.to_string();
        let native = crate::run::subject_from_json(field(deployment, "subject")?)?;
        let Subject::Deployment {
            artifacts: deployed,
            ..
        } = native
        else {
            return Err("assessment requires deployment Subject".into());
        };
        let components = deployed
            .into_iter()
            .map(|a| (a.id, a.digest))
            .collect::<BTreeMap<_, _>>();
        let observation = field(request, "deployment_observation")?;
        validate_observation(observation)?;
        let contracts = array(request, "context_contracts")?;
        let mut fingerprints = BTreeSet::new();
        for contract in contracts {
            validate_context_contract(contract)?;
            if !fingerprints.insert(text(contract, "fingerprint")?) {
                return Err("duplicate context contract".into());
            }
        }
        let captures = array(request, "captures")?;
        for capture in captures {
            validate_capture(capture, contracts)?;
        }
        let now = number(request, "evaluated_at_ms")?;
        let mut reasons = Vec::new();
        if !trusted(observation)? {
            reasons.push("deployment observation is not trusted".into());
        }
        if text(observation, "subject_fingerprint")? != subject {
            reasons.push("deployment observation belongs to another Subject".into());
        }
        if text(observation, "state")? != "complete" {
            reasons.push("deployment composition is not completely observed".into());
        }
        let time = number(observation, "observed_at_ms")?;
        if time > now
            || now.saturating_sub(time)
                > number(observation, "valid_for_seconds")?.saturating_mul(1000)
        {
            reasons.push("deployment observation is stale or future-dated".into());
        }
        let observed = array(observation, "components")?
            .iter()
            .map(|c| Ok((text(c, "id")?.to_string(), text(c, "digest")?.to_string())))
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        if observed != components {
            reasons.push("observed components differ from deployment Subject".into());
        }
        if let Some(release_components) = release.get("components").and_then(Json::as_array) {
            for (id, digest) in &components {
                if !release_components.iter().any(|c| {
                    c.get("id").and_then(Json::as_str) == Some(id)
                        && c.get("digest").and_then(Json::as_str) == Some(digest)
                }) {
                    reasons.push(format!(
                        "component `{id}` is not linked to release composition"
                    ));
                }
            }
        } else {
            reasons.push("release has no component identities".into());
        }
        Ok(Self {
            release,
            deployment,
            observation,
            contracts,
            captures,
            now,
            subject,
            components,
            deployment_reasons: reasons,
        })
    }

    fn scope(&self, policy: &Json, bundle: &RunBundle) -> Result<(), String> {
        let selected = strings(policy, "components")?;
        let mut digests = BTreeMap::new();
        for id in selected {
            let digest = self
                .components
                .get(&id)
                .ok_or_else(|| format!("component `{id}` is not deployed"))?;
            digests.insert(id, digest.clone());
        }
        match (text(policy, "subject")?, &bundle.subject) {
            ("deployment", Subject::Deployment { .. })
                if bundle.subject_fingerprint == self.subject =>
            {
                Ok(())
            }
            ("artifact", Subject::Artifact { artifacts }) => {
                let actual = artifacts
                    .iter()
                    .map(|a| (a.id.clone(), a.digest.clone()))
                    .collect::<BTreeMap<_, _>>();
                if actual == digests {
                    Ok(())
                } else {
                    Err("Run artifacts differ from scoped deployed components".into())
                }
            }
            ("source", Subject::Source { repositories })
            | ("source", Subject::CiCandidate { repositories }) => {
                let mut required = BTreeSet::new();
                for digest in digests.values() {
                    let producers = array(self.release, "builds")?
                        .iter()
                        .filter(|b| {
                            array(b, "artifacts")
                                .is_ok_and(|a| a.iter().any(|v| v.as_str() == Some(digest)))
                        })
                        .collect::<Vec<_>>();
                    if producers.is_empty() {
                        return Err("component artifact has no build provenance".into());
                    }
                    let mut sets = BTreeSet::new();
                    for build in producers {
                        sets.insert(strings(build, "sources")?);
                    }
                    if sets.len() != 1 {
                        return Err("component artifact has conflicting build source inputs".into());
                    }
                    required.extend(sets.into_iter().next().unwrap());
                }
                if policy.get("configuration_sources").is_some() {
                    required.extend(strings(policy, "configuration_sources")?);
                }
                let mut expected = BTreeSet::new();
                for id in required {
                    let source = array(self.release, "sources")?
                        .iter()
                        .find(|s| s.get("repository").and_then(Json::as_str) == Some(&id))
                        .ok_or("build source is missing from release")?;
                    expected.insert((
                        id,
                        text(source, "commit")?.to_string(),
                        text(source, "content_fingerprint")?.to_string(),
                    ));
                }
                let actual = repositories
                    .iter()
                    .map(|r| {
                        (
                            r.id.clone(),
                            r.revision.clone(),
                            r.content_fingerprint.clone(),
                        )
                    })
                    .collect::<BTreeSet<_>>();
                if expected == actual {
                    Ok(())
                } else {
                    Err("Run source inputs differ from complete component build inputs".into())
                }
            }
            _ => Err("Run Subject kind or deployment identity differs from Check policy".into()),
        }
    }

    fn contexts(
        &self,
        policy: &Json,
        bundle: &RunBundle,
        execution: &crate::run::CheckExecution,
        case: &str,
    ) -> Result<Vec<Json>, String> {
        if policy.get("units").is_some() {
            let required = strings(policy, "units")?;
            let planned = bundle
                .plan
                .checks
                .iter()
                .find(|check| check.id == execution.check.id)
                .ok_or("Check missing from Run plan")?
                .units
                .iter()
                .map(|unit| unit.id.clone())
                .collect::<BTreeSet<_>>();
            let actual = execution
                .units
                .iter()
                .map(|unit| unit.id.clone())
                .collect::<BTreeSet<_>>();
            if required != planned || required != actual {
                return Err("Run does not cover exact required unit inventory".into());
            }
        }
        let mut dependencies = Vec::new();
        for fp in strings(policy, "contexts")? {
            let contract = self
                .contracts
                .iter()
                .find(|c| c.get("fingerprint").and_then(Json::as_str) == Some(&fp))
                .ok_or("required context contract is missing")?;
            let name = text(contract, "name")?;
            for component in strings(policy, "components")? {
                let controlled = policy
                    .get("controls")
                    .and_then(Json::as_array)
                    .unwrap_or(&[])
                    .iter()
                    .filter(|control| {
                        strings(control, "contexts").is_ok_and(|values| values.contains(&fp))
                    })
                    .map(|control| text(control, "unit").map(str::to_owned))
                    .collect::<Result<BTreeSet<_>, _>>()?;
                let mut tested = Vec::new();
                let mut positive = false;
                for unit in &execution.units {
                    for attempt in &unit.attempts {
                        if !attempt.outcomes.contains_key(case) {
                            continue;
                        }
                        let activity = bundle
                            .activities
                            .iter()
                            .find(|activity| activity.id == attempt.activity)
                            .ok_or("Check attempt activity is absent")?;
                        let candidates = self
                            .captures
                            .iter()
                            .filter(|capture| {
                                capture.get("kind").and_then(Json::as_str) == Some("execution")
                                    && capture.get("run").and_then(Json::as_str)
                                        == Some(bundle.run_id.as_str())
                                    && capture.get("bundle_fingerprint").and_then(Json::as_str)
                                        == Some(bundle.bundle_fingerprint.as_str())
                                    && capture.get("component").and_then(Json::as_str)
                                        == Some(component.as_str())
                                    && (capture.get("activity") == Some(&Json::Null)
                                        || capture.get("activity").and_then(Json::as_str)
                                            == Some(activity.id.as_str()))
                                    && contains_context(capture, name)
                            })
                            .collect::<Vec<_>>();
                        if candidates.is_empty() {
                            return Err(format!(
                                "activity `{}` has no required context capture",
                                activity.id
                            ));
                        }
                        let mut captured = None;
                        for capture in candidates {
                            let whole_run = capture.get("activity") == Some(&Json::Null);
                            let (start, finish) = if whole_run {
                                (bundle.started_at_ms, bundle.finished_at_ms)
                            } else {
                                (activity.started_at_ms, activity.finished_at_ms)
                            };
                            if !trusted(capture)?
                                || number(capture, "finished_at_ms")?
                                    > self.now.saturating_add(CLOCK_SKEW_MS)
                                || text(capture, "subject_fingerprint")?
                                    != bundle.subject_fingerprint
                                || number(capture, "started_at_ms")? > start
                                || number(capture, "finished_at_ms")? < finish
                            {
                                return Err("execution context capture does not cover trusted activity interval".into());
                            }
                            let value = stable_context(capture, name, &fp)?;
                            let key = if whole_run {
                                format!("context/{fp}")
                            } else {
                                format!("context/{fp}/activity/{}", activity.id)
                            };
                            if bundle.actual_selection.context.get(&key)
                                != Some(&canonical(&value)?)
                            {
                                return Err(
                                    "Run selection does not pin captured context fingerprint"
                                        .into(),
                                );
                            }
                            if captured
                                .as_ref()
                                .is_some_and(|previous| !same(previous, &value))
                            {
                                return Err("activity instances disagree on context".into());
                            }
                            captured = Some(value);
                            dependencies.push(capture_reference(capture)?);
                        }
                        if !controlled.contains(&unit.id) {
                            positive |=
                                attempt.outcomes.get(case) == Some(&ObservationOutcome::Satisfied);
                            tested.push(captured.unwrap());
                        }
                    }
                }
                if tested.is_empty()
                    || (!controlled.is_empty()
                        && !positive
                        && execution.observations.iter().any(|observation| {
                            observation.case == case
                                && observation.outcome == ObservationOutcome::Satisfied
                        }))
                {
                    return Err(
                        "control activities cannot alone establish positive Case support".into(),
                    );
                }
                let observed = array(self.observation, "components")?
                    .iter()
                    .find(|c| c.get("id").and_then(Json::as_str) == Some(component.as_str()))
                    .ok_or("production component context is not observed")?;
                let instances = strings(observed, "instances")?;
                if instances.is_empty() {
                    return Err("production context has no observed live instances".into());
                }
                for instance in instances {
                    let captures = self
                        .captures
                        .iter()
                        .filter(|c| {
                            c.get("kind").and_then(Json::as_str) == Some("deployment")
                                && c.get("subject_fingerprint").and_then(Json::as_str)
                                    == Some(self.subject.as_str())
                                && c.get("component").and_then(Json::as_str)
                                    == Some(component.as_str())
                                && c.get("instance").and_then(Json::as_str)
                                    == Some(instance.as_str())
                                && contains_context(c, name)
                        })
                        .collect::<Vec<_>>();
                    let latest = captures
                        .iter()
                        .map(|c| number(c, "finished_at_ms"))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .max()
                        .ok_or_else(|| {
                            format!("production instance `{instance}` has no context capture")
                        })?;
                    if latest > self.now
                        || self.now.saturating_sub(latest)
                            > number(self.observation, "valid_for_seconds")?.saturating_mul(1000)
                    {
                        return Err("production context capture is stale or future-dated".into());
                    }
                    for capture in captures
                        .into_iter()
                        .filter(|c| number(c, "finished_at_ms").ok() == Some(latest))
                    {
                        if !trusted(capture)? {
                            return Err("production context capture is not trusted".into());
                        }
                        // Startup-fixed values belong to the process lifetime, so independent observers need not collect at the same instant.
                        if latest.abs_diff(number(self.observation, "observed_at_ms")?)
                            > number(self.observation, "valid_for_seconds")?.saturating_mul(1000)
                        {
                            return Err("production context capture is outside the current observation window".into());
                        }
                        if tested.iter().any(|value| {
                            !stable_context(capture, name, &fp)
                                .is_ok_and(|production| same(value, &production))
                        }) {
                            return Err(format!(
                                "effective context `{name}` differs on instance `{instance}`"
                            ));
                        }
                        dependencies.push(capture_reference(capture)?);
                    }
                }
            }
        }
        Ok(dependencies)
    }
}

fn stable_context(capture: &Json, name: &str, fp: &str) -> Result<Json, String> {
    let before = field(
        field(field(field(capture, "before")?, "report")?, "contexts")?,
        name,
    )?;
    let after = field(
        field(field(field(capture, "after")?, "report")?, "contexts")?,
        name,
    )?;
    if text(before, "contract_revision")? != fp || !same(before, after) {
        return Err(format!("context `{name}` changed or uses another contract"));
    }
    Ok(before.clone())
}
fn capture_reference(capture: &Json) -> Result<Json, String> {
    Ok(Json::obj(vec![
        ("kind", Json::str("context-capture")),
        ("fingerprint", Json::str(capture_fingerprint(capture)?)),
        ("component", field(capture, "component")?.clone()),
        ("instance", field(capture, "instance")?.clone()),
    ]))
}

struct Candidate<'a> {
    bundle: &'a RunBundle,
    observation: &'a crate::run::Observation,
    dependencies: Vec<Json>,
    inapplicable: Option<String>,
}

pub fn assess(request: &Json) -> Result<Json, String> {
    assess_selected(request, None)
}

/// Selection limits evaluation only; authority and evidence validation retain their full closure.
pub fn assess_selected(
    request: &Json,
    selected: Option<&BTreeSet<String>>,
) -> Result<Json, String> {
    let input = Inputs::new(request)?;
    let snapshot = field(field(request, "publication")?, "snapshot")?;
    let authority = field(snapshot, "authority")?;
    validate_policies(authority)?;
    let mut raw = array(snapshot, "bundles")?.to_vec();
    for bundle in array(request, "bundles")? {
        if !raw.iter().any(|v| same(v, bundle)) {
            raw.push(bundle.clone());
        }
    }
    let bundles = raw
        .iter()
        .map(|v| {
            crate::run::parse("deployment.bundles", &v.to_string_pretty())
                .map_err(|e| format!("invalid Run: {e:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let findings = crate::run::verify_set(&bundles);
    if !findings.is_empty() {
        return Err(format!("invalid Run history: {findings:?}"));
    }
    let mut heads = BTreeMap::<&str, &RunBundle>::new();
    for bundle in &bundles {
        if heads
            .get(bundle.run_id.as_str())
            .is_none_or(|b| b.bundle_revision < bundle.bundle_revision)
        {
            heads.insert(&bundle.run_id, bundle);
        }
    }
    let mut reviews = array(snapshot, "reviews")?.to_vec();
    for review in array(request, "reviews")? {
        crate::assurance_review::validate_record(review)?;
        if !reviews.iter().any(|v| same(v, review)) {
            reviews.push(review.clone());
        }
    }
    if let Some(selected) = selected {
        let declared = array(authority, "claims")?
            .iter()
            .map(|claim| text(claim, "id"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if selected.iter().any(|id| !declared.contains(id.as_str())) {
            return Err("selected Claim is absent from the complete authority".into());
        }
    }
    let mut claims = Vec::new();
    for claim in array(authority, "claims")? {
        let id = text(claim, "id")?;
        if selected.is_some_and(|ids| !ids.contains(id)) {
            continue;
        }
        let target = Json::obj(vec![
            ("kind", Json::str("claim-review")),
            ("claim", Json::str(id)),
        ]);
        let latest = crate::assurance_review::latest_review(&reviews, &target);
        let (review_status, review_record) = if text(claim, "criticality")? == "routine" {
            ("not-required".to_string(), None)
        } else {
            match latest {
                Ok(Some(r)) => (
                    crate::assurance_review::assess_record(r, authority, &reviews, &bundles).status,
                    Some(r),
                ),
                Ok(None) => ("missing".into(), None),
                Err(_) => ("invalid".into(), None),
            }
        };
        let review_ok = review_status == "accepted" || review_status == "not-required";
        let mut reasons = input.deployment_reasons.clone();
        if !review_ok {
            reasons.push(format!("Claim Review is {review_status}"));
        }
        let mut dependencies = vec![Json::obj(vec![
            ("kind", Json::str("release")),
            ("fingerprint", field(input.release, "fingerprint")?.clone()),
        ])];
        if let Some(r) = review_record {
            dependencies.push(Json::obj(vec![
                ("kind", Json::str("claim-review")),
                ("fingerprint", field(r, "fingerprint")?.clone()),
            ]));
        }
        let cases = strings(claim, "cases")?;
        let mut supported_cases = BTreeSet::new();
        let mut bindings_out = Vec::new();
        let mut coverage = Vec::new();
        let mut supporting_evidence = Vec::new();
        let mut violated_evidence = Vec::new();
        let mut inconclusive_evidence = Vec::new();
        let mut unassessed_evidence = Vec::new();
        let mut adverse = Vec::new();
        let mut definitive = false;
        let mut all_bindings = true;
        let claim_bindings = array(authority, "bindings")?
            .iter()
            .filter(|b| {
                b.get("case")
                    .and_then(Json::as_str)
                    .is_some_and(|c| cases.contains(c))
            })
            .collect::<Vec<_>>();
        if claim_bindings.is_empty() {
            all_bindings = false;
            reasons.push("Claim has no required Check/Case bindings".into());
        }
        for binding in claim_bindings {
            let check_id = text(binding, "check")?;
            let case = text(binding, "case")?;
            let check = array(authority, "checks")?
                .iter()
                .find(|c| c.get("id").and_then(Json::as_str) == Some(check_id))
                .ok_or("binding Check absent from authority")?;
            let mut missing = Vec::new();
            let mut candidates = Vec::new();
            let policy = check.get("deployment");
            let monitored = policy
                .and_then(|p| p.get("monitor"))
                .is_some_and(|m| *m != Json::Null);
            let mut incomplete_monitors = Vec::new();
            if let Some(policy) = policy {
                validate_policy(policy)?;
                for bundle in heads.values() {
                    if monitored
                        && bundle.status != RunStatus::Complete
                        && bundle.plan.checks.iter().any(|c| {
                            c.id == check_id && c.fingerprint == text(check, "fingerprint").unwrap()
                        })
                        && input.scope(policy, bundle).is_ok()
                    {
                        incomplete_monitors.push(bundle.finished_at_ms);
                    }
                    for execution in &bundle.check_executions {
                        if execution.check.id != check_id
                            || execution.check.fingerprint != text(check, "fingerprint")?
                        {
                            continue;
                        }
                        let observations = execution
                            .observations
                            .iter()
                            .filter(|o| o.case == case)
                            .collect::<Vec<_>>();
                        if observations.is_empty() {
                            continue;
                        }
                        if let Err(reason) = input.scope(policy, bundle) {
                            missing.push(reason);
                            continue;
                        }
                        let (context_dependencies, inapplicable) =
                            match input.contexts(policy, bundle, execution, case) {
                                Ok(v) => (v, None),
                                Err(reason) if monitored => (Vec::new(), Some(reason)),
                                Err(reason) => {
                                    missing.push(reason);
                                    continue;
                                }
                            };
                        for observation in observations {
                            candidates.push(Candidate {
                                bundle,
                                observation,
                                dependencies: context_dependencies.clone(),
                                inapplicable: inapplicable.clone(),
                            });
                        }
                    }
                }
            } else {
                missing.push("Check has no deployment applicability policy".into());
            }
            let mut selected = candidates.iter().collect::<Vec<_>>();
            if monitored {
                if let Some(latest) = selected.iter().map(|c| c.observation.observed_at_ms).max() {
                    selected.retain(|c| c.observation.observed_at_ms == latest);
                }
                let latest_observation = selected
                    .iter()
                    .map(|c| c.observation.observed_at_ms)
                    .max()
                    .unwrap_or(0);
                if incomplete_monitors
                    .iter()
                    .any(|time| *time >= latest_observation)
                {
                    selected.clear();
                    missing.push("latest Monitor execution is incomplete".into());
                }
            }
            let mut outcomes = BTreeSet::new();
            let mut binding_observations = Vec::new();
            let mut binding_ok = !selected.is_empty();
            for candidate in &selected {
                let mut usable = input.deployment_reasons.is_empty();
                if let Some(reason) = &candidate.inapplicable {
                    usable = false;
                    missing.push(reason.clone());
                }
                let observation = candidate.observation;
                if candidate.bundle.status != RunStatus::Complete {
                    usable = false;
                    missing.push("Run is incomplete".into());
                }
                if observation.observed_at_ms > input.now.saturating_add(CLOCK_SKEW_MS) {
                    usable = false;
                    missing.push("Check observation is future-dated".into());
                }
                if monitored {
                    let freshness =
                        number(field(policy.unwrap(), "monitor")?, "freshness_seconds")?
                            .saturating_mul(1000);
                    if input.now.saturating_sub(observation.observed_at_ms) > freshness {
                        usable = false;
                        missing.push("Monitor observation is overdue".into());
                    }
                }
                let reference = Json::obj(vec![
                    ("kind", Json::str("observation")),
                    ("run", Json::str(&candidate.bundle.run_id)),
                    (
                        "bundle_fingerprint",
                        Json::str(&candidate.bundle.bundle_fingerprint),
                    ),
                    ("fingerprint", Json::str(&observation.fingerprint)),
                    ("observation", Json::str(&observation.fingerprint)),
                    ("check", Json::str(check_id)),
                    ("case", Json::str(case)),
                    (
                        "observed_at_ms",
                        Json::Num(observation.observed_at_ms as f64),
                    ),
                    ("selected_in_basis", Json::Bool(usable)),
                    ("accepted_basis", Json::Bool(usable)),
                    (
                        "diagnostics",
                        Json::Arr(observation.diagnostics.iter().map(Json::str).collect()),
                    ),
                    (
                        "subject_fingerprint",
                        Json::str(&candidate.bundle.subject_fingerprint),
                    ),
                    ("outcome", Json::str(observation.outcome.name())),
                    ("applicable", Json::Bool(usable)),
                ]);
                binding_observations.push(reference.clone());
                if !usable {
                    unassessed_evidence.push(reference.clone());
                }
                if usable {
                    match observation.outcome {
                        ObservationOutcome::Satisfied => {
                            supporting_evidence.push(reference.clone())
                        }
                        ObservationOutcome::Violated => violated_evidence.push(reference.clone()),
                        ObservationOutcome::Inconclusive => {
                            inconclusive_evidence.push(reference.clone())
                        }
                    }
                    outcomes.insert(observation.outcome.name());
                    if observation.outcome == ObservationOutcome::Violated {
                        adverse.push(reference.clone());
                    }
                    dependencies.push(reference);
                    dependencies.extend(candidate.dependencies.clone());
                } else {
                    binding_ok = false;
                }
            }
            if outcomes.len() > 1 {
                binding_ok = false;
                missing.push("current Check observations disagree".into());
            }
            if outcomes.contains("inconclusive") {
                binding_ok = false;
                missing.push("current Check result is inconclusive".into());
            }
            if outcomes.contains("violated") {
                binding_ok = false;
                let contribution = text(policy.unwrap(), "contribution")?;
                if (contribution == "counterexample" || contribution == "both")
                    && !(monitored && outcomes.len() > 1)
                {
                    definitive = true;
                } else {
                    missing.push(
                        "adverse Check result has no declared counterexample contribution".into(),
                    );
                }
            }
            if selected.is_empty() {
                missing.push("no applicable current Check observation".into());
                binding_ok = false;
            }
            if outcomes.contains("satisfied")
                && binding_ok
                && policy.is_some_and(|p| {
                    p.get("contribution").and_then(Json::as_str) != Some("counterexample")
                })
            {
                supported_cases.insert(case.to_string());
            }
            if !binding_ok {
                all_bindings = false;
            }
            missing.sort();
            missing.dedup();
            if !binding_ok {
                reasons.extend(missing.iter().map(|m| format!("{check_id}/{case}: {m}")));
            }
            coverage.push(Json::obj(vec![
                ("check", Json::str(check_id)),
                ("case", Json::str(case)),
                (
                    "status",
                    Json::str(if binding_ok {
                        "satisfied"
                    } else {
                        "unresolved"
                    }),
                ),
                ("observations", Json::Arr(binding_observations)),
            ]));
            bindings_out.push(Json::obj(vec![
                ("check", Json::str(check_id)),
                ("case", Json::str(case)),
                (
                    "status",
                    Json::str(if binding_ok {
                        "satisfied"
                    } else {
                        "unresolved"
                    }),
                ),
                (
                    "monitor",
                    policy
                        .and_then(|p| p.get("monitor"))
                        .cloned()
                        .unwrap_or(Json::Null),
                ),
                (
                    "reasons",
                    Json::Arr(missing.iter().map(Json::str).collect()),
                ),
            ]));
        }
        for case in cases.difference(&supported_cases) {
            reasons.push(format!(
                "Case `{case}` lacks applicable supporting evidence"
            ));
        }
        for gap in array(claim, "gaps")? {
            if let Some(gap) = gap.as_str() {
                reasons.push(gap.into());
            }
        }
        reasons.sort();
        reasons.dedup();
        let status = if definitive && review_ok {
            "violated"
        } else if review_ok
            && all_bindings
            && cases == supported_cases
            && !cases.is_empty()
            && reasons.is_empty()
        {
            "supported"
        } else {
            "unresolved"
        };
        let summary = if status == "supported" {
            "All required deployment evidence supports the Claim".into()
        } else if status == "violated" {
            "Applicable evidence provides a declared counterexample to the Claim".into()
        } else {
            reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "Deployment evidence is insufficient".into())
        };
        sort_facts(&mut dependencies);
        sort_facts(&mut supporting_evidence);
        sort_facts(&mut violated_evidence);
        sort_facts(&mut inconclusive_evidence);
        sort_facts(&mut unassessed_evidence);
        sort_facts(&mut adverse);
        let reason_records = reasons
            .iter()
            .map(|reason| {
                Json::obj(vec![
                    ("code", Json::str(reason_code(reason))),
                    ("category", Json::str("deployment-evidence")),
                    ("message", Json::str(reason)),
                ])
            })
            .collect();
        let review_details = Json::obj(vec![
            ("status", Json::str(&review_status)),
            (
                "fingerprint",
                review_record
                    .and_then(|r| r.get("fingerprint"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "reviewer",
                review_record
                    .and_then(|r| r.get("reviewer"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "reviewed_at_ms",
                review_record
                    .and_then(|r| r.get("reviewed_at_ms"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "rationale",
                review_record
                    .and_then(|r| r.get("rationale"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
        ]);
        let explanation = Json::obj(vec![
            ("summary", Json::str(summary)),
            ("status", Json::str(status)),
            ("reasons", Json::Arr(reason_records)),
            ("review", review_details),
            (
                "health",
                Json::obj(vec![
                    ("status", Json::str("computed")),
                    ("fingerprint", Json::Null),
                    ("reviewer", Json::Null),
                    ("reviewed_at_ms", Json::Null),
                    ("rationale", Json::Null),
                ]),
            ),
            ("coverage", Json::Arr(coverage)),
            (
                "evidence",
                Json::obj(vec![
                    ("supporting", Json::Arr(supporting_evidence)),
                    ("violations", Json::Arr(violated_evidence)),
                    ("inconclusive", Json::Arr(inconclusive_evidence)),
                    ("unassessed", Json::Arr(unassessed_evidence)),
                ]),
            ),
            ("residual_risks", Json::Arr(vec![])),
            ("dependencies", Json::Arr(dependencies.clone())),
            ("adverse", Json::Arr(adverse)),
        ]);
        claims.push(Json::obj(vec![
            ("claim", Json::str(id)),
            ("status", Json::str(status)),
            ("review", Json::str(review_status)),
            (
                "diagnostics",
                Json::Arr(reasons.iter().map(Json::str).collect()),
            ),
            ("gaps", field(claim, "gaps")?.clone()),
            ("bindings", Json::Arr(bindings_out)),
            ("challenges", Json::Arr(vec![])),
            ("explanation", explanation),
            ("dependencies", Json::Arr(dependencies)),
            ("evaluated_at_ms", Json::Num(input.now as f64)),
        ]));
    }
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-deployment-status")),
        ("version", Json::Num(1.0)),
        ("project", field(authority, "project")?.clone()),
        (
            "authority_fingerprint",
            field(authority, "fingerprint")?.clone(),
        ),
        (
            "model_fingerprint",
            field(authority, "model_fingerprint")?.clone(),
        ),
        ("subject_fingerprint", Json::str(&input.subject)),
        ("deployment", field(input.deployment, "id")?.clone()),
        ("evaluated_at_ms", Json::Num(input.now as f64)),
        ("claims", Json::Arr(claims)),
    ]))
}

fn reason_code(reason: &str) -> &'static str {
    if reason.contains("Review") {
        "review-not-current"
    } else if reason.contains("overdue") {
        "monitor-overdue"
    } else if reason.contains("context") {
        "context-inapplicable"
    } else if reason.contains("source")
        || reason.contains("artifact")
        || reason.contains("component")
    {
        "lineage-inapplicable"
    } else if reason.contains("disagree") {
        "evidence-ambiguous"
    } else if reason.contains("inconclusive") || reason.contains("incomplete") {
        "execution-inconclusive"
    } else if reason.contains("future") {
        "observation-future"
    } else {
        "evidence-missing"
    }
}

pub fn capture_fingerprint(capture: &Json) -> Result<String, String> {
    let Json::Obj(fields) = capture else {
        return Err("capture requires object".into());
    };
    canonical(&Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "trusted")
            .cloned()
            .collect(),
    ))
}

fn sort_facts(values: &mut Vec<Json>) {
    values.sort_by_cached_key(|v| crate::fingerprint::canonical_json(v));
    values.dedup_by(|left, right| same(left, right));
}

pub fn validate_policies(authority: &Json) -> Result<(), String> {
    let mut monitors = BTreeSet::new();
    for check in array(authority, "checks")? {
        if let Some(policy) = check.get("deployment") {
            validate_policy(policy)?;
            let monitor = field(policy, "monitor")?;
            if *monitor != Json::Null && !monitors.insert(text(monitor, "id")?) {
                return Err("Monitor identity must reference exactly one Check".into());
            }
        }
    }
    Ok(())
}

fn contains_context(capture: &Json, name: &str) -> bool {
    ["before", "after"].iter().any(|phase| {
        capture
            .get(phase)
            .and_then(|snapshot| snapshot.get("report"))
            .and_then(|report| report.get("contexts"))
            .and_then(|contexts| contexts.get(name))
            .is_some()
    })
}
