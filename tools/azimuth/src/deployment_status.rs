//! Deployment evidence applicability and deterministic Requirement Status.
use crate::json::Json;
use crate::run::{CheckResultOutcome, RunBundle, RunStatus, VerificationTarget};
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
        "verification_target",
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
        if text(v, "verification_target")? != "source" {
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
    if !matches!(
        text(v, "verification_target")?,
        "source" | "artifact" | "deployment"
    ) {
        return Err("unsupported deployment Check verification target".into());
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
        if text(v, "verification_target")? != "deployment" {
            return Err("Monitor requires deployment execution Verification Target".into());
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
    if text(report, "format")? != "azimuth-execution-context" || number(report, "version")? != 2 {
        return Err("unsupported Execution Context report".into());
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
            "verification_target_fingerprint",
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
    digest(text(v, "verification_target_fingerprint")?)?;
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
    verification_target: String,
    components: BTreeMap<String, String>,
    deployment_reasons: Vec<String>,
}

fn validate_observation(v: &Json) -> Result<(), String> {
    shape(
        v,
        &[
            "verification_target_fingerprint",
            "observed_at_ms",
            "valid_for_seconds",
            "state",
            "components",
            "trusted",
        ],
    )?;
    digest(text(v, "verification_target_fingerprint")?)?;
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

struct Parts<'a> {
    publication: &'a Json,
    deployment: &'a Json,
    observation: &'a Json,
    contracts: &'a [Json],
    captures: &'a [Json],
    now: u64,
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
            || number(request, "version")? != 2
        {
            return Err("unsupported deployment assessment request".into());
        }
        crate::release::verify(field(request, "publication")?)?;
        Self::build(
            Parts {
                publication: field(request, "publication")?,
                deployment: field(request, "deployment")?,
                observation: field(request, "deployment_observation")?,
                contracts: array(request, "context_contracts")?,
                captures: array(request, "captures")?,
                now: number(request, "evaluated_at_ms")?,
            },
            true,
        )
    }

    fn build(parts: Parts<'a>, verify: bool) -> Result<Self, String> {
        let Parts {
            publication,
            deployment,
            observation,
            contracts,
            captures,
            now,
        } = parts;
        let release = field(publication, "release")?;
        let artifacts = array(release, "artifacts")?
            .iter()
            .map(|a| text(a, "digest").map(str::to_string))
            .collect::<Result<BTreeSet<_>, _>>()?;
        crate::release::verify_deployment(deployment, &artifacts)?;
        let verification_target = text(deployment, "verification_target_fingerprint")?.to_string();
        let native =
            crate::run::verification_target_from_json(field(deployment, "verification_target")?)?;
        let VerificationTarget::Deployment {
            artifacts: deployed,
            ..
        } = native
        else {
            return Err("assessment requires deployment Verification Target".into());
        };
        let components = deployed
            .into_iter()
            .map(|a| (a.id, a.digest))
            .collect::<BTreeMap<_, _>>();
        validate_observation(observation)?;
        if verify {
            let mut fingerprints = BTreeSet::new();
            for contract in contracts {
                validate_context_contract(contract)?;
                if !fingerprints.insert(text(contract, "fingerprint")?) {
                    return Err("duplicate context contract".into());
                }
            }
            for capture in captures {
                validate_capture(capture, contracts)?;
            }
        }
        let mut reasons = Vec::new();
        if !trusted(observation)? {
            reasons.push("deployment observation is not trusted".into());
        }
        if text(observation, "verification_target_fingerprint")? != verification_target {
            reasons.push("deployment observation belongs to another Verification Target".into());
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
            reasons.push("observed components differ from deployment Verification Target".into());
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
            verification_target,
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
        match (
            text(policy, "verification_target")?,
            &bundle.verification_target,
        ) {
            ("deployment", VerificationTarget::Deployment { .. })
                if bundle.verification_target_fingerprint == self.verification_target =>
            {
                Ok(())
            }
            ("artifact", VerificationTarget::Artifact { artifacts }) => {
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
            ("source", VerificationTarget::Source { repositories })
            | ("source", VerificationTarget::CiCandidate { repositories }) => {
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
            _ => Err(
                "Run Verification Target kind or deployment identity differs from Check policy"
                    .into(),
            ),
        }
    }

    fn contexts(
        &self,
        policy: &Json,
        bundle: &RunBundle,
        execution: &crate::run::CheckExecution,
        scenario: &str,
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
                        if !attempt.outcomes.contains_key(scenario) {
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
                                || text(capture, "verification_target_fingerprint")?
                                    != bundle.verification_target_fingerprint
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
                            if bundle.actual_selection.execution_context.get(&key)
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
                            positive |= attempt.outcomes.get(scenario)
                                == Some(&CheckResultOutcome::Satisfied);
                            tested.push(captured.unwrap());
                        }
                    }
                }
                if tested.is_empty()
                    || (!controlled.is_empty()
                        && !positive
                        && execution.results.iter().any(|check_result| {
                            check_result.scenario == scenario
                                && check_result.outcome == CheckResultOutcome::Satisfied
                        }))
                {
                    return Err(
                        "control activities cannot alone establish positive Scenario support"
                            .into(),
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
                                && c.get("verification_target_fingerprint")
                                    .and_then(Json::as_str)
                                    == Some(self.verification_target.as_str())
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
    check_result: &'a crate::run::CheckResult,
    dependencies: Vec<Json>,
    inapplicable: Option<String>,
}

pub fn assess(request: &Json) -> Result<Json, String> {
    assess_selected(request, None)
}

/// Evidence the caller has already verified: the publication, the context contracts and captures, every Run
/// individually and as a history, and every Requirement Review. Evaluation does not repeat that work.
pub struct Evidence<'a> {
    pub publication: &'a Json,
    pub deployment: &'a Json,
    pub observation: &'a Json,
    pub contracts: &'a [Json],
    pub captures: &'a [Json],
    pub bundles: &'a [RunBundle],
    pub reviews: &'a [Json],
    pub evaluated_at_ms: u64,
}

/// Verifies a release publication together with the model policies it carries.
pub fn verify_publication(publication: &Json) -> Result<(), String> {
    crate::release::verify(publication)?;
    validate_policies(field(field(publication, "snapshot")?, "authority")?)
}

/// Parses one Run and requires it to be protocol-valid on its own.
pub fn verify_run(source: &str) -> Result<RunBundle, String> {
    let bundle = crate::run::parse("deployment.bundles", source)
        .map_err(|e| format!("invalid Run: {e:?}"))?;
    let findings = crate::run::verify(&bundle);
    if findings.is_empty() {
        Ok(bundle)
    } else {
        Err(format!("invalid Run history: {findings:?}"))
    }
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
    let mut known = raw
        .iter()
        .filter_map(|bundle| canonical(bundle).ok())
        .collect::<BTreeSet<_>>();
    for bundle in array(request, "bundles")? {
        if canonical(bundle).map_or(true, |canonical| known.insert(canonical)) {
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
    let mut reviews = array(snapshot, "reviews")?.to_vec();
    let mut known_reviews = reviews
        .iter()
        .filter_map(|review| canonical(review).ok())
        .collect::<BTreeSet<_>>();
    for review in array(request, "reviews")? {
        crate::assurance_review::validate_record(review)?;
        if canonical(review).map_or(true, |canonical| known_reviews.insert(canonical)) {
            reviews.push(review.clone());
        }
    }
    evaluate(&input, authority, &bundles, &reviews, selected)
}

/// Evaluates evidence that has already been verified; see [`Evidence`].
pub fn assess_evidence(
    evidence: &Evidence,
    selected: Option<&BTreeSet<String>>,
) -> Result<Json, String> {
    let input = Inputs::build(
        Parts {
            publication: evidence.publication,
            deployment: evidence.deployment,
            observation: evidence.observation,
            contracts: evidence.contracts,
            captures: evidence.captures,
            now: evidence.evaluated_at_ms,
        },
        false,
    )?;
    let authority = field(field(evidence.publication, "snapshot")?, "authority")?;
    evaluate(
        &input,
        authority,
        evidence.bundles,
        evidence.reviews,
        selected,
    )
}

fn evaluate(
    input: &Inputs,
    authority: &Json,
    bundles: &[RunBundle],
    reviews: &[Json],
    selected: Option<&BTreeSet<String>>,
) -> Result<Json, String> {
    let runs = crate::assurance_review::Runs::verified(bundles);
    let mut heads = BTreeMap::<&str, &RunBundle>::new();
    for bundle in bundles {
        if heads
            .get(bundle.run_id.as_str())
            .is_none_or(|b| b.bundle_revision < bundle.bundle_revision)
        {
            heads.insert(&bundle.run_id, bundle);
        }
    }
    if let Some(selected) = selected {
        let declared = array(authority, "requirements")?
            .iter()
            .map(|requirement| text(requirement, "id"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if selected.iter().any(|id| !declared.contains(id.as_str())) {
            return Err("selected Requirement is absent from the complete authority".into());
        }
    }
    let mut requirements = Vec::new();
    for requirement in array(authority, "requirements")? {
        let id = text(requirement, "id")?;
        if selected.is_some_and(|ids| !ids.contains(id)) {
            continue;
        }
        let target = Json::obj(vec![
            ("kind", Json::str("requirement-review")),
            ("requirement", Json::str(id)),
        ]);
        let latest = crate::assurance_review::latest_review(reviews, &target);
        let (review_status, review_record) = if text(requirement, "criticality")? == "routine" {
            ("not-required".to_string(), None)
        } else {
            match latest {
                Ok(Some(r)) => (
                    crate::assurance_review::assess_record_in(r, authority, reviews, &runs).status,
                    Some(r),
                ),
                Ok(None) => ("missing".into(), None),
                Err(_) => ("invalid".into(), None),
            }
        };
        let review_ok = review_status == "accepted" || review_status == "not-required";
        let mut reasons = input.deployment_reasons.clone();
        if !review_ok {
            reasons.push(format!("Requirement Review is {review_status}"));
        }
        let mut dependencies = vec![Json::obj(vec![
            ("kind", Json::str("release")),
            ("fingerprint", field(input.release, "fingerprint")?.clone()),
        ])];
        if let Some(r) = review_record {
            dependencies.push(Json::obj(vec![
                ("kind", Json::str("requirement-review")),
                ("fingerprint", field(r, "fingerprint")?.clone()),
            ]));
        }
        let scenarios = strings(requirement, "scenarios")?;
        let mut supported_scenarios = BTreeSet::new();
        let mut bindings_out = Vec::new();
        let mut coverage = Vec::new();
        let mut supporting_evidence = Vec::new();
        let mut violated_evidence = Vec::new();
        let mut inconclusive_evidence = Vec::new();
        let mut unassessed_evidence = Vec::new();
        let mut adverse = Vec::new();
        let mut definitive = false;
        let mut all_bindings = true;
        let requirement_bindings = array(authority, "bindings")?
            .iter()
            .filter(|b| {
                b.get("scenario")
                    .and_then(Json::as_str)
                    .is_some_and(|c| scenarios.contains(c))
            })
            .collect::<Vec<_>>();
        if requirement_bindings.is_empty() {
            all_bindings = false;
            reasons.push("Requirement has no required Check/Scenario bindings".into());
        }
        for binding in requirement_bindings {
            let check_id = text(binding, "check")?;
            let scenario = text(binding, "scenario")?;
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
                        let check_results = execution
                            .results
                            .iter()
                            .filter(|o| o.scenario == scenario)
                            .collect::<Vec<_>>();
                        if check_results.is_empty() {
                            continue;
                        }
                        if let Err(reason) = input.scope(policy, bundle) {
                            missing.push(reason);
                            continue;
                        }
                        let (context_dependencies, inapplicable) =
                            match input.contexts(policy, bundle, execution, scenario) {
                                Ok(v) => (v, None),
                                Err(reason) if monitored => (Vec::new(), Some(reason)),
                                Err(reason) => {
                                    missing.push(reason);
                                    continue;
                                }
                            };
                        for check_result in check_results {
                            candidates.push(Candidate {
                                bundle,
                                check_result,
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
                if let Some(latest) = selected.iter().map(|c| c.check_result.observed_at_ms).max() {
                    selected.retain(|c| c.check_result.observed_at_ms == latest);
                }
                let latest_check_result = selected
                    .iter()
                    .map(|c| c.check_result.observed_at_ms)
                    .max()
                    .unwrap_or(0);
                if incomplete_monitors
                    .iter()
                    .any(|time| *time >= latest_check_result)
                {
                    selected.clear();
                    missing.push("latest Monitor execution is incomplete".into());
                }
            }
            let mut outcomes = BTreeSet::new();
            let mut binding_check_results = Vec::new();
            let mut binding_ok = !selected.is_empty();
            for candidate in &selected {
                let mut usable = input.deployment_reasons.is_empty();
                if let Some(reason) = &candidate.inapplicable {
                    usable = false;
                    missing.push(reason.clone());
                }
                let check_result = candidate.check_result;
                if candidate.bundle.status != RunStatus::Complete {
                    usable = false;
                    missing.push("Run is incomplete".into());
                }
                if check_result.observed_at_ms > input.now.saturating_add(CLOCK_SKEW_MS) {
                    usable = false;
                    missing.push("Check Result is future-dated".into());
                }
                if monitored {
                    let freshness =
                        number(field(policy.unwrap(), "monitor")?, "freshness_seconds")?
                            .saturating_mul(1000);
                    if input.now.saturating_sub(check_result.observed_at_ms) > freshness {
                        usable = false;
                        missing.push("Monitor Check Result is overdue".into());
                    }
                }
                let reference = Json::obj(vec![
                    ("kind", Json::str("check-result")),
                    ("run", Json::str(&candidate.bundle.run_id)),
                    (
                        "bundle_fingerprint",
                        Json::str(&candidate.bundle.bundle_fingerprint),
                    ),
                    ("fingerprint", Json::str(&check_result.fingerprint)),
                    ("check_result", Json::str(&check_result.fingerprint)),
                    ("check", Json::str(check_id)),
                    ("scenario", Json::str(scenario)),
                    (
                        "observed_at_ms",
                        Json::Num(check_result.observed_at_ms as f64),
                    ),
                    ("selected_in_basis", Json::Bool(usable)),
                    ("accepted_basis", Json::Bool(usable)),
                    (
                        "diagnostics",
                        Json::Arr(check_result.diagnostics.iter().map(Json::str).collect()),
                    ),
                    (
                        "verification_target_fingerprint",
                        Json::str(&candidate.bundle.verification_target_fingerprint),
                    ),
                    ("outcome", Json::str(check_result.outcome.name())),
                    ("applicable", Json::Bool(usable)),
                ]);
                binding_check_results.push(reference.clone());
                if !usable {
                    unassessed_evidence.push(reference.clone());
                }
                if usable {
                    match check_result.outcome {
                        CheckResultOutcome::Satisfied => {
                            supporting_evidence.push(reference.clone())
                        }
                        CheckResultOutcome::Violated => violated_evidence.push(reference.clone()),
                        CheckResultOutcome::Inconclusive => {
                            inconclusive_evidence.push(reference.clone())
                        }
                    }
                    outcomes.insert(check_result.outcome.name());
                    if check_result.outcome == CheckResultOutcome::Violated {
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
                missing.push("current Check Results disagree".into());
            }
            if outcomes.contains("inconclusive") {
                binding_ok = false;
                missing.push("current Check Result is inconclusive".into());
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
                        "adverse Check Result has no declared counterexample contribution".into(),
                    );
                }
            }
            if selected.is_empty() {
                missing.push("no applicable current Check Result".into());
                binding_ok = false;
            }
            if outcomes.contains("satisfied")
                && binding_ok
                && policy.is_some_and(|p| {
                    p.get("contribution").and_then(Json::as_str) != Some("counterexample")
                })
            {
                supported_scenarios.insert(scenario.to_string());
            }
            if !binding_ok {
                all_bindings = false;
            }
            missing.sort();
            missing.dedup();
            if !binding_ok {
                reasons.extend(
                    missing
                        .iter()
                        .map(|m| format!("{check_id}/{scenario}: {m}")),
                );
            }
            coverage.push(Json::obj(vec![
                ("check", Json::str(check_id)),
                ("scenario", Json::str(scenario)),
                (
                    "status",
                    Json::str(if binding_ok {
                        "satisfied"
                    } else {
                        "unresolved"
                    }),
                ),
                ("check_results", Json::Arr(binding_check_results)),
            ]));
            bindings_out.push(Json::obj(vec![
                ("check", Json::str(check_id)),
                ("scenario", Json::str(scenario)),
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
        for scenario in scenarios.difference(&supported_scenarios) {
            reasons.push(format!(
                "Scenario `{scenario}` lacks applicable supporting evidence"
            ));
        }
        for gap in array(requirement, "gaps")? {
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
            && scenarios == supported_scenarios
            && !scenarios.is_empty()
            && reasons.is_empty()
        {
            "supported"
        } else {
            "unresolved"
        };
        let summary = if status == "supported" {
            "All required deployment evidence supports the Requirement".into()
        } else if status == "violated" {
            "Applicable evidence provides a declared counterexample to the Requirement".into()
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
                "requirement_status",
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
        requirements.push(Json::obj(vec![
            ("requirement", Json::str(id)),
            ("status", Json::str(status)),
            ("review", Json::str(review_status)),
            (
                "diagnostics",
                Json::Arr(reasons.iter().map(Json::str).collect()),
            ),
            ("gaps", field(requirement, "gaps")?.clone()),
            ("bindings", Json::Arr(bindings_out)),
            ("challenges", Json::Arr(vec![])),
            ("explanation", explanation),
            ("dependencies", Json::Arr(dependencies)),
            ("evaluated_at_ms", Json::Num(input.now as f64)),
        ]));
    }
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-deployment-status")),
        ("version", Json::Num(2.0)),
        ("project", field(authority, "project")?.clone()),
        (
            "authority_fingerprint",
            field(authority, "fingerprint")?.clone(),
        ),
        (
            "model_fingerprint",
            field(authority, "model_fingerprint")?.clone(),
        ),
        (
            "verification_target_fingerprint",
            Json::str(&input.verification_target),
        ),
        ("deployment", field(input.deployment, "id")?.clone()),
        ("evaluated_at_ms", Json::Num(input.now as f64)),
        ("requirements", Json::Arr(requirements)),
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
