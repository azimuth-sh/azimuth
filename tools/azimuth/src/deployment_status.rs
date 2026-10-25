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
    let dependencies = context_dependencies(v, "contexts")?;
    if v.get("controls").is_some() {
        let mut controlled = BTreeSet::new();
        for control in array(v, "controls")? {
            shape(control, &["unit", "contexts"])?;
            let unit = text(control, "unit")?;
            let contexts = context_dependencies(control, "contexts")?;
            if !units.contains(unit)
                || !controlled.insert(unit)
                || contexts.is_empty()
                || !contexts.is_subset(&dependencies)
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
    for id in &components {
        crate::diag::validate_id(id, false)?;
    }
    if let Some((component, _)) = dependencies
        .iter()
        .find(|(component, _)| !components.contains(component))
    {
        return Err(format!(
            "context dependency names component `{component}` outside the Check's components"
        ));
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

/// Parses `[{component, leaves}]` into the (component, leaf) pairs a Check depends on.
pub fn context_dependencies(v: &Json, key: &str) -> Result<BTreeSet<(String, String)>, String> {
    let mut pairs = BTreeSet::new();
    let mut components = BTreeSet::new();
    for entry in array(v, key)? {
        shape(entry, &["component", "leaves"])?;
        let component = text(entry, "component")?;
        crate::diag::validate_id(component, false)?;
        if !components.insert(component) {
            return Err(format!(
                "duplicate context dependency component `{component}`"
            ));
        }
        let leaves = strings(entry, "leaves")?;
        if leaves.is_empty() {
            return Err(format!("context dependency `{component}` lists no leaves"));
        }
        for leaf in leaves {
            if !crate::model::is_leaf_path(&leaf) {
                return Err(format!("invalid context leaf `{leaf}`"));
            }
            pairs.insert((component.to_string(), leaf));
        }
    }
    Ok(pairs)
}

/// Requirement and Scenario conditions as carried by the authority: `[{component, leaf, value}]`.
pub fn validate_conditions(v: &Json) -> Result<(), String> {
    parse_conditions(v).map(|_| ())
}

struct ConditionRef {
    component: String,
    leaf: String,
    value: Json,
}

fn parse_conditions(v: &Json) -> Result<Vec<ConditionRef>, String> {
    let items = v.as_array().ok_or("conditions requires array")?;
    items
        .iter()
        .map(|item| {
            shape(item, &["component", "leaf", "value"])?;
            let component = text(item, "component")?;
            crate::diag::validate_id(component, false)?;
            let leaf = text(item, "leaf")?;
            if !crate::model::is_leaf_path(leaf) {
                return Err(format!("invalid condition leaf `{leaf}`"));
            }
            let value = field(item, "value")?;
            if !matches!(value, Json::Bool(_) | Json::Num(_) | Json::Str(_)) {
                return Err("condition value requires boolean, integer or string".into());
            }
            Ok(ConditionRef {
                component: component.to_string(),
                leaf: leaf.to_string(),
                value: value.clone(),
            })
        })
        .collect()
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
    let component = text(v, "component")?;
    crate::diag::validate_id(component, false)?;
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
        let Json::Obj(contexts) = field(field(snapshot, "report")?, "contexts")? else {
            unreachable!("validated report holds a contexts object")
        };
        if !matches!(contexts.as_slice(), [(name, _)] if name == component) {
            return Err(
                "capture must hold exactly one context, named by the capture's component".into(),
            );
        }
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
    captures: &'a [Json],
    now: u64,
    verification_target: String,
    components: BTreeMap<String, String>,
    unit_manifest: BTreeMap<String, ManifestUnit>,
    unit_manifest_repository: Option<String>,
    deployment_reasons: Vec<String>,
}

#[derive(Clone)]
struct ManifestUnit {
    input_fingerprint: String,
    hermetic: bool,
    /// Checks the unit partitions; empty means every Check.
    checks: BTreeSet<String>,
}

pub fn validate_unit_manifest(v: &Json) -> Result<(), String> {
    parse_unit_manifest(v).map(|_| ())
}

struct UnitManifest {
    repository: Option<String>,
    units: BTreeMap<String, ManifestUnit>,
}

fn parse_unit_manifest(v: &Json) -> Result<UnitManifest, String> {
    let repository = match v.get("repository") {
        None => None,
        Some(value) => Some(
            value
                .as_str()
                .filter(|id| !id.trim().is_empty())
                .ok_or("unit input manifest `repository` must be a nonempty string")?
                .to_string(),
        ),
    };
    let Json::Obj(pairs) = v else {
        return Err("expected object".into());
    };
    let without_repository = Json::Obj(
        pairs
            .iter()
            .filter(|(key, _)| key != "repository")
            .cloned()
            .collect(),
    );
    shape(
        &without_repository,
        &["format", "version", "rule_fingerprint", "units"],
    )?;
    if text(v, "format")? != "azimuth-unit-input-manifest" || number(v, "version")? != 1 {
        return Err("unsupported unit input manifest".into());
    }
    digest(text(v, "rule_fingerprint")?)?;
    let mut units = BTreeMap::new();
    let mut previous: Option<&str> = None;
    for unit in array(v, "units")? {
        let optional_checks = unit.get("checks").is_some();
        if optional_checks {
            shape(unit, &["id", "input_fingerprint", "hermetic", "checks"])?;
        } else {
            shape(unit, &["id", "input_fingerprint", "hermetic"])?;
        }
        let checks = if optional_checks {
            let listed = strings(unit, "checks")?;
            if listed.is_empty() {
                return Err("unit input manifest `checks` must be omitted when empty".into());
            }
            listed
        } else {
            BTreeSet::new()
        };
        let id = text(unit, "id")?;
        crate::diag::validate_id(id, false)?;
        if previous.is_some_and(|previous| previous >= id) {
            return Err("unit input manifest units must be sorted and unique".into());
        }
        previous = Some(id);
        let input_fingerprint = text(unit, "input_fingerprint")?;
        digest(input_fingerprint)?;
        let hermetic = field(unit, "hermetic")?
            .as_bool()
            .ok_or("hermetic requires boolean")?;
        units.insert(
            id.to_string(),
            ManifestUnit {
                input_fingerprint: input_fingerprint.to_string(),
                hermetic,
                checks,
            },
        );
    }
    Ok(UnitManifest { repository, units })
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
                        "component `{id}` is not linked to a release component"
                    ));
                }
            }
        } else {
            reasons.push("release has no component identities".into());
        }
        let (unit_manifest, unit_manifest_repository) = match release.get("unit_inputs") {
            Some(manifest) => {
                let parsed = parse_unit_manifest(manifest)?;
                (parsed.units, parsed.repository)
            }
            None => (BTreeMap::new(), None),
        };
        Ok(Self {
            release,
            deployment,
            observation,
            captures,
            now,
            verification_target,
            components,
            unit_manifest,
            unit_manifest_repository,
            deployment_reasons: reasons,
        })
    }

    fn expected_sources(
        &self,
        policy: &Json,
        digests: &BTreeMap<String, String>,
    ) -> Result<BTreeSet<(String, String, String)>, String> {
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
        Ok(expected)
    }

    fn scoped_components(&self, policy: &Json) -> Result<BTreeMap<String, String>, String> {
        let mut digests = BTreeMap::new();
        for id in strings(policy, "components")? {
            let digest = self
                .components
                .get(&id)
                .ok_or_else(|| format!("component `{id}` is not deployed"))?;
            digests.insert(id, digest.clone());
        }
        Ok(digests)
    }

    /// Unit-level reuse for a source Check: every executed unit is hermetic in the release manifest and
    /// its input fingerprint equals the manifest's, so the commit may differ from the release's. The
    /// manifest names the repository its unit rule was computed for; every other build input of the
    /// Check must still equal the release. A manifest without a repository identity is accepted only
    /// when the Check's complete build inputs are one repository.
    fn reuse_scope(
        &self,
        policy: &Json,
        bundle: &RunBundle,
        execution: &crate::run::CheckExecution,
    ) -> Result<(), String> {
        if text(policy, "verification_target")? != "source" {
            return Err("only source Checks reuse unit results".into());
        }
        let (VerificationTarget::Source { repositories }
        | VerificationTarget::CiCandidate { repositories }) = &bundle.verification_target
        else {
            return Err("only source Runs reuse unit results".into());
        };
        if self.unit_manifest.is_empty() || execution.units.is_empty() {
            return Err("release has no unit input manifest for this Check".into());
        }
        let expected = self.expected_sources(policy, &self.scoped_components(policy)?)?;
        match &self.unit_manifest_repository {
            None => {
                let [(repository, _, _)] = expected.iter().collect::<Vec<_>>()[..] else {
                    return Err("unit reuse requires exactly one source repository".into());
                };
                if repositories.len() != 1 || repositories[0].id != *repository {
                    return Err(
                        "Run source repository differs from the Check's build inputs".into(),
                    );
                }
            }
            Some(manifest_repository) => {
                if repositories.len() != expected.len()
                    || !expected.iter().any(|(id, _, _)| id == manifest_repository)
                {
                    return Err(
                        "Run source repositories differ from the Check's build inputs".into(),
                    );
                }
                for (id, commit, content_fingerprint) in &expected {
                    let state = repositories
                        .iter()
                        .find(|state| state.id == *id)
                        .ok_or("Run source repositories differ from the Check's build inputs")?;
                    if id != manifest_repository
                        && (state.revision != *commit
                            || state.content_fingerprint != *content_fingerprint)
                    {
                        return Err(format!(
                            "source repository `{id}` differs from the release and is not the manifest's repository"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// The units of the execution whose results apply to the release: all of them under whole-source
    /// equality, otherwise the hermetic units whose input fingerprint equals the manifest's. A reused
    /// Run must pass in every unit it executed, because its Check Result speaks for all of them; the
    /// units whose inputs changed are then simply left for another Run to cover.
    fn applicable_scope(
        &self,
        policy: &Json,
        bundle: &RunBundle,
        execution: &crate::run::CheckExecution,
        scenario: &str,
    ) -> Result<BTreeSet<String>, String> {
        let executed = || execution.units.iter().map(|unit| unit.id.clone()).collect();
        match self.scope(policy, bundle) {
            Ok(()) => Ok(executed()),
            Err(reason) => {
                if self.reuse_scope(policy, bundle, execution).is_err() {
                    return Err(reason);
                }
                let mut covered = BTreeSet::new();
                for unit in &execution.units {
                    let passing = unit
                        .executed_tests
                        .iter()
                        .all(|test| test.outcome == crate::run::ExecutedTestOutcome::Passed)
                        && unit
                            .attempts
                            .iter()
                            .filter_map(|attempt| attempt.outcomes.get(scenario))
                            .all(|outcome| *outcome == CheckResultOutcome::Satisfied);
                    if !passing {
                        return Err(format!("unit `{}` has no passing result to reuse", unit.id));
                    }
                    let unchanged = self.unit_manifest.get(&unit.id).is_some_and(|manifest| {
                        manifest.hermetic
                            && unit.input_fingerprint.as_deref()
                                == Some(manifest.input_fingerprint.as_str())
                    });
                    if unchanged {
                        covered.insert(unit.id.clone());
                    }
                }
                if covered.is_empty() {
                    return Err(
                        "no unit of the Run has inputs equal to the release or is hermetic".into(),
                    );
                }
                Ok(covered)
            }
        }
    }

    /// The release-manifest units a source Check must have passing results for. A `whole` execution
    /// runs every test, so it covers them all; a project without a manifest requires nothing.
    fn uncovered_units(&self, check_id: &str, covered: &BTreeSet<String>) -> Vec<String> {
        if covered.contains("whole") {
            return Vec::new();
        }
        self.unit_manifest
            .iter()
            .filter(|(id, unit)| {
                (unit.checks.is_empty() || unit.checks.contains(check_id)) && !covered.contains(*id)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    fn scope(&self, policy: &Json, bundle: &RunBundle) -> Result<(), String> {
        let digests = self.scoped_components(policy)?;
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
                let expected = self.expected_sources(policy, &digests)?;
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
        for (component, leaf) in context_dependencies(policy, "contexts")? {
            let controlled = policy
                .get("controls")
                .and_then(Json::as_array)
                .unwrap_or(&[])
                .iter()
                .filter(|control| {
                    context_dependencies(control, "contexts")
                        .is_ok_and(|pairs| pairs.contains(&(component.clone(), leaf.clone())))
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
                                && contains_context(capture, &component)
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
                        let value = stable_leaf(capture, &component, &leaf)?;
                        let key = if whole_run {
                            format!("context/{component}/{leaf}")
                        } else {
                            format!("context/{component}/{leaf}/activity/{}", activity.id)
                        };
                        if bundle.actual_selection.execution_context.get(&key)
                            != Some(&canonical(&value)?)
                        {
                            return Err(
                                "Run selection does not pin captured context leaf value".into()
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
                            attempt.outcomes.get(scenario) == Some(&CheckResultOutcome::Satisfied);
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
                    "control activities cannot alone establish positive Scenario support".into(),
                );
            }
            for instance in self.live_instances(&component)? {
                let capture = self.current_capture(&component, &instance)?;
                let production = stable_leaf(capture, &component, &leaf);
                if tested.iter().any(|value| {
                    !production
                        .as_ref()
                        .is_ok_and(|production| same(value, production))
                }) {
                    return Err(format!(
                        "effective context leaf `{component}.{leaf}` differs on instance `{instance}`"
                    ));
                }
                dependencies.push(capture_reference(capture)?);
            }
        }
        Ok(dependencies)
    }

    fn live_instances(&self, component: &str) -> Result<BTreeSet<String>, String> {
        let observed = array(self.observation, "components")?
            .iter()
            .find(|c| c.get("id").and_then(Json::as_str) == Some(component))
            .ok_or("production component context is not observed")?;
        let instances = strings(observed, "instances")?;
        if instances.is_empty() {
            return Err("production context has no observed live instances".into());
        }
        Ok(instances)
    }

    /// The newest deployment capture of one live instance, required to be trusted and inside the observation window.
    fn current_capture(&self, component: &str, instance: &str) -> Result<&'a Json, String> {
        let captures = self
            .captures
            .iter()
            .filter(|c| {
                c.get("kind").and_then(Json::as_str) == Some("deployment")
                    && c.get("verification_target_fingerprint")
                        .and_then(Json::as_str)
                        == Some(self.verification_target.as_str())
                    && c.get("component").and_then(Json::as_str) == Some(component)
                    && c.get("instance").and_then(Json::as_str) == Some(instance)
                    && contains_context(c, component)
            })
            .collect::<Vec<_>>();
        let latest = captures
            .iter()
            .map(|c| number(c, "finished_at_ms"))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .ok_or_else(|| format!("production instance `{instance}` has no context capture"))?;
        let window = number(self.observation, "valid_for_seconds")?.saturating_mul(1000);
        if latest > self.now || self.now.saturating_sub(latest) > window {
            return Err("production context capture is stale or future-dated".into());
        }
        // Startup-fixed values belong to the process lifetime, so independent observers need not collect at the same instant.
        if latest.abs_diff(number(self.observation, "observed_at_ms")?) > window {
            return Err(
                "production context capture is outside the current observation window".into(),
            );
        }
        let mut newest = captures
            .into_iter()
            .filter(|c| number(c, "finished_at_ms").ok() == Some(latest));
        let capture = newest
            .next()
            .ok_or("production context capture is absent")?;
        for other in std::iter::once(capture).chain(newest) {
            if !trusted(other)? {
                return Err("production context capture is not trusted".into());
            }
        }
        Ok(capture)
    }

    /// Whether the deployment's current context capture satisfies one `Context:` condition.
    fn condition_truth(&self, condition: &ConditionRef) -> Truth {
        if !self.deployment_reasons.is_empty() {
            return Truth::Unknown(format!(
                "context of component `{}` cannot be established from the current deployment observation",
                condition.component
            ));
        }
        let instances = match self.live_instances(&condition.component) {
            Ok(instances) => instances,
            Err(_) => {
                return Truth::Unknown(format!(
                    "no observed instances of component `{}` for its context",
                    condition.component
                ))
            }
        };
        let mut holds = BTreeSet::new();
        for instance in instances {
            let capture = match self.current_capture(&condition.component, &instance) {
                Ok(capture) => capture,
                Err(reason) => {
                    return Truth::Unknown(format!(
                        "no current context capture of component `{}`: {reason}",
                        condition.component
                    ))
                }
            };
            match stable_leaf(capture, &condition.component, &condition.leaf) {
                Ok(value) => holds.insert(same(&value, &condition.value)),
                Err(LeafError::Absent(_)) => holds.insert(false),
                Err(LeafError::Changed(reason)) => {
                    return Truth::Unknown(format!(
                        "context of component `{}` is unstable: {reason}",
                        condition.component
                    ))
                }
            };
        }
        match (holds.contains(&true), holds.contains(&false)) {
            (true, false) => Truth::True,
            (false, true) => Truth::False,
            _ => Truth::Unknown(format!(
                "instances of component `{}` disagree on context leaf `{}`",
                condition.component, condition.leaf
            )),
        }
    }
}

#[derive(Clone)]
enum Truth {
    True,
    False,
    Unknown(String),
}

impl Truth {
    fn name(&self) -> &'static str {
        match self {
            Truth::True => "holds",
            Truth::False => "fails",
            Truth::Unknown(_) => "unknown",
        }
    }
}

fn condition_record(scope: &str, condition: &ConditionRef, truth: &Truth) -> Json {
    Json::obj(vec![
        ("scope", Json::str(scope)),
        ("component", Json::str(&condition.component)),
        ("leaf", Json::str(&condition.leaf)),
        ("value", condition.value.clone()),
        ("state", Json::str(truth.name())),
    ])
}

/// A false condition makes its scope not applicable even beside unknown ones, because no later capture
/// could make the scope applicable while that leaf holds.
fn conjunction(truths: &[Truth]) -> Truth {
    let mut unknown = None;
    for truth in truths {
        match truth {
            Truth::False => return Truth::False,
            Truth::Unknown(reason) => unknown = unknown.or(Some(reason.clone())),
            Truth::True => {}
        }
    }
    unknown.map_or(Truth::True, Truth::Unknown)
}

/// A leaf that changed between the before and after snapshots is unknown, not false: neither value
/// describes the process, so a condition on it must not conclude `not-applicable`.
enum LeafError {
    Absent(String),
    Changed(String),
}

impl From<String> for LeafError {
    fn from(reason: String) -> Self {
        LeafError::Absent(reason)
    }
}

impl From<LeafError> for String {
    fn from(error: LeafError) -> Self {
        match error {
            LeafError::Absent(reason) | LeafError::Changed(reason) => reason,
        }
    }
}

fn stable_leaf(capture: &Json, component: &str, leaf: &str) -> Result<Json, LeafError> {
    let leaf_value = |phase: &str| -> Result<Json, LeafError> {
        let values = field(
            field(
                field(field(field(capture, phase)?, "report")?, "contexts")?,
                component,
            )?,
            "values",
        )?;
        Ok(field(values, leaf)?.clone())
    };
    let before = leaf_value("before")?;
    if leaf_value("after").map_or(true, |after| !same(&before, &after)) {
        return Err(LeafError::Changed(format!(
            "context leaf `{component}.{leaf}` changed during capture"
        )));
    }
    Ok(before)
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
    executed_for_release: bool,
    covered_units: BTreeSet<String>,
    skipped_tests: bool,
    units: Vec<Json>,
}

#[derive(Default)]
struct CheckAccount {
    scenarios: BTreeSet<String>,
    states: BTreeSet<&'static str>,
    runs: BTreeMap<String, Json>,
}

impl CheckAccount {
    fn outcome(&self) -> &'static str {
        ["counterexample", "skipped", "inconclusive", "missing"]
            .into_iter()
            .find(|state| self.states.contains(state))
            .unwrap_or("supported")
    }
}

fn run_subject(bundle: &RunBundle) -> Json {
    match &bundle.verification_target {
        VerificationTarget::Source { repositories }
        | VerificationTarget::CiCandidate { repositories }
        | VerificationTarget::Workspace { repositories } => repositories
            .iter()
            .min_by(|left, right| left.id.cmp(&right.id))
            .map_or(Json::Null, |repository| {
                Json::obj(vec![
                    ("repository", Json::str(&repository.id)),
                    ("commit", Json::str(&repository.revision)),
                ])
            }),
        _ => Json::Null,
    }
}

pub fn assess(request: &Json) -> Result<Json, String> {
    assess_selected(request, None)
}

/// Evidence the caller has already verified: the publication, the context contracts and captures, every Run
/// individually and as a history, and every Verification Review. Evaluation does not repeat that work.
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
            ("kind", Json::str("verification-review")),
            ("requirement", Json::str(id)),
        ]);
        let latest = crate::assurance_review::latest_review(reviews, &target);
        let (review_status, review_record) = if text(requirement, "verification")? == "none" {
            ("not-declared".to_string(), None)
        } else {
            match latest {
                Ok(Some(r)) => (
                    crate::assurance_review::assess_head_in(r, authority, reviews, &runs).status,
                    Some(r),
                ),
                Ok(None) => ("missing".into(), None),
                Err(_) => ("invalid".into(), None),
            }
        };
        let review_ok = review_status == "accepted" || review_status == "not-declared";
        let realization_target = Json::obj(vec![
            ("kind", Json::str("realization-review")),
            ("requirement", Json::str(id)),
        ]);
        let (realization_status, realization_record) = if text(requirement, "realization")?
            == "none"
        {
            ("not-declared".to_string(), None)
        } else {
            match crate::assurance_review::latest_review(reviews, &realization_target) {
                Ok(Some(r)) => (
                    crate::assurance_review::assess_head_in(r, authority, reviews, &runs).status,
                    Some(r),
                ),
                Ok(None) => ("missing".into(), None),
                Err(_) => ("invalid".into(), None),
            }
        };
        let realization_drift = realization_record
            .and_then(|r| crate::assurance_review::realization_drift(r, authority));
        let mut reasons = input.deployment_reasons.clone();
        if !review_ok {
            reasons.push(format!("Verification Review is {review_status}"));
        }
        let mut dependencies = vec![Json::obj(vec![
            ("kind", Json::str("release")),
            ("fingerprint", field(input.release, "fingerprint")?.clone()),
        ])];
        if let Some(r) = review_record {
            dependencies.push(Json::obj(vec![
                ("kind", Json::str("verification-review")),
                ("fingerprint", field(r, "fingerprint")?.clone()),
            ]));
        }
        let scenarios = strings(requirement, "scenarios")?;
        let requirement_conditions = match requirement.get("conditions") {
            Some(conditions) => parse_conditions(conditions)?,
            None => Vec::new(),
        };
        let requirement_truths = requirement_conditions
            .iter()
            .map(|condition| input.condition_truth(condition))
            .collect::<Vec<_>>();
        let mut condition_records = requirement_conditions
            .iter()
            .zip(&requirement_truths)
            .map(|(condition, truth)| condition_record("requirement", condition, truth))
            .collect::<Vec<_>>();
        let requirement_truth = conjunction(&requirement_truths);
        let mut not_applicable_scenarios = BTreeSet::new();
        let mut unknown_scenarios = BTreeMap::new();
        for scenario in &scenarios {
            let declared = array(authority, "scenarios")?
                .iter()
                .find(|item| item.get("id").and_then(Json::as_str) == Some(scenario.as_str()))
                .ok_or("Requirement Scenario absent from authority")?;
            let own_conditions = match declared.get("conditions") {
                Some(conditions) => parse_conditions(conditions)?,
                None => Vec::new(),
            };
            let own_truths = own_conditions
                .iter()
                .map(|condition| input.condition_truth(condition))
                .collect::<Vec<_>>();
            condition_records.extend(
                own_conditions
                    .iter()
                    .zip(&own_truths)
                    .map(|(condition, truth)| condition_record(scenario, condition, truth)),
            );
            match conjunction(&[requirement_truth.clone(), conjunction(&own_truths)]) {
                Truth::False => {
                    not_applicable_scenarios.insert(scenario.clone());
                }
                Truth::Unknown(reason) => {
                    unknown_scenarios.insert(scenario.clone(), reason);
                }
                Truth::True => {}
            }
        }
        let not_applicable = matches!(requirement_truth, Truth::False)
            || (!scenarios.is_empty() && not_applicable_scenarios.len() == scenarios.len());
        let required_scenarios = scenarios
            .difference(&not_applicable_scenarios)
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut check_accounts = BTreeMap::<String, CheckAccount>::new();
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
                !not_applicable
                    && b.get("scenario").and_then(Json::as_str).is_some_and(|c| {
                        required_scenarios.contains(c) && !unknown_scenarios.contains_key(c)
                    })
            })
            .collect::<Vec<_>>();
        if requirement_bindings.is_empty() && !not_applicable {
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
                        let covered_units =
                            match input.applicable_scope(policy, bundle, execution, scenario) {
                                Ok(covered) => covered,
                                Err(reason) => {
                                    missing.push(reason);
                                    continue;
                                }
                            };
                        let executed_for_release = input.scope(policy, bundle).is_ok();
                        let skipped_tests = execution.units.iter().any(|unit| {
                            unit.executed_tests.iter().any(|test| {
                                matches!(
                                    test.outcome,
                                    crate::run::ExecutedTestOutcome::Skipped
                                        | crate::run::ExecutedTestOutcome::NotExecuted
                                )
                            })
                        });
                        let units = execution
                            .units
                            .iter()
                            .map(|unit| {
                                Json::obj(vec![
                                    ("id", Json::str(&unit.id)),
                                    (
                                        "input_fingerprint",
                                        unit.input_fingerprint
                                            .as_ref()
                                            .map_or(Json::Null, Json::str),
                                    ),
                                    (
                                        "matched",
                                        Json::Bool(input.unit_manifest.get(&unit.id).is_some_and(
                                            |manifest| {
                                                unit.input_fingerprint.as_deref()
                                                    == Some(manifest.input_fingerprint.as_str())
                                            },
                                        )),
                                    ),
                                ])
                            })
                            .collect::<Vec<_>>();
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
                                executed_for_release,
                                covered_units: covered_units.clone(),
                                skipped_tests,
                                units: units.clone(),
                            });
                        }
                    }
                }
            } else {
                missing.push("Check has no deployment applicability policy".into());
            }
            if text(policy.unwrap_or(&Json::Null), "verification_target")
                .is_ok_and(|t| t == "source")
            {
                let mut covered = BTreeSet::new();
                for candidate in &candidates {
                    if candidate.check_result.outcome == CheckResultOutcome::Satisfied
                        && candidate.bundle.status == RunStatus::Complete
                        && !candidate.skipped_tests
                        && candidate.inapplicable.is_none()
                    {
                        covered.extend(candidate.covered_units.iter().cloned());
                    }
                }
                let uncovered = input.uncovered_units(check_id, &covered);
                let before = candidates.len();
                if !uncovered.is_empty() {
                    candidates.retain(|c| c.check_result.outcome != CheckResultOutcome::Satisfied);
                    if candidates.len() != before {
                        missing.push(format!(
                            "units without a passing result: {}",
                            uncovered.join(", ")
                        ));
                    }
                }
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
            let account = check_accounts.entry(check_id.to_string()).or_default();
            account.scenarios.insert(scenario.to_string());
            let mut outcomes = BTreeSet::new();
            let mut binding_check_results = Vec::new();
            let mut binding_counterexample = false;
            let mut binding_ok = !selected.is_empty();
            for candidate in &selected {
                let mut usable = input.deployment_reasons.is_empty();
                if let Some(reason) = &candidate.inapplicable {
                    usable = false;
                    missing.push(reason.clone());
                }
                let check_result = candidate.check_result;
                if candidate.skipped_tests {
                    usable = false;
                    missing.push(
                        "selected tests were skipped or not executed, so the result is inconclusive"
                            .into(),
                    );
                }
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
                    (
                        "executed_for_release",
                        Json::Bool(candidate.executed_for_release),
                    ),
                    ("reused", Json::Bool(!candidate.executed_for_release)),
                ]);
                if usable {
                    account.runs.insert(
                        candidate.bundle.bundle_fingerprint.clone(),
                        Json::obj(vec![
                            (
                                "run_fingerprint",
                                Json::str(&candidate.bundle.bundle_fingerprint),
                            ),
                            ("run_id", Json::str(&candidate.bundle.run_id)),
                            (
                                "executed_for_release",
                                Json::Bool(candidate.executed_for_release),
                            ),
                            ("reused", Json::Bool(!candidate.executed_for_release)),
                            ("subject", run_subject(candidate.bundle)),
                            ("units", Json::Arr(candidate.units.clone())),
                            (
                                "executed_at_ms",
                                Json::Num(check_result.observed_at_ms as f64),
                            ),
                        ]),
                    );
                }
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
                    binding_counterexample = true;
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
            account.states.insert(if binding_counterexample {
                "counterexample"
            } else if selected.iter().any(|candidate| candidate.skipped_tests) {
                "skipped"
            } else if outcomes.contains("inconclusive") || outcomes.len() > 1 {
                "inconclusive"
            } else if binding_ok && outcomes.contains("satisfied") {
                "supported"
            } else {
                "missing"
            });
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
        for scenario in required_scenarios.difference(&supported_scenarios) {
            match unknown_scenarios.get(scenario) {
                Some(reason) => reasons.push(format!(
                    "Scenario `{scenario}` condition is unknown: {reason}"
                )),
                None => reasons.push(format!(
                    "Scenario `{scenario}` lacks applicable supporting evidence"
                )),
            }
        }
        for gap in array(requirement, "gaps")? {
            if let Some(gap) = gap.as_str() {
                reasons.push(gap.into());
            }
        }
        if not_applicable {
            reasons.clear();
        }
        reasons.sort();
        reasons.dedup();
        let status = if not_applicable {
            "not-applicable"
        } else if definitive && review_ok {
            "violated"
        } else if review_ok
            && all_bindings
            && required_scenarios == supported_scenarios
            && !required_scenarios.is_empty()
            && unknown_scenarios.is_empty()
            && reasons.is_empty()
        {
            "supported"
        } else {
            "unresolved"
        };
        let summary = if status == "not-applicable" {
            "Not applicable here: the deployment's context fails a condition of the Requirement"
                .into()
        } else if status == "supported" {
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
        let realization_details = Json::obj(vec![
            ("status", Json::str(&realization_status)),
            (
                "fingerprint",
                realization_record
                    .and_then(|r| r.get("fingerprint"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "reviewer",
                realization_record
                    .and_then(|r| r.get("reviewer"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "reviewed_at_ms",
                realization_record
                    .and_then(|r| r.get("reviewed_at_ms"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "reviewed_commit",
                realization_record
                    .and_then(|r| r.get("reviewed_commit"))
                    .cloned()
                    .unwrap_or(Json::Null),
            ),
            (
                "code_drift",
                realization_drift.map_or(Json::Null, Json::Bool),
            ),
        ]);
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
            ("verification_review", review_details),
            ("realization_review", realization_details),
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
            ("conditions", Json::Arr(condition_records)),
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
            ("verification_review", Json::str(review_status)),
            ("realization_review", Json::str(realization_status)),
            (
                "realization_code_drift",
                realization_drift.map_or(Json::Null, Json::Bool),
            ),
            (
                "diagnostics",
                Json::Arr(reasons.iter().map(Json::str).collect()),
            ),
            ("gaps", field(requirement, "gaps")?.clone()),
            ("bindings", Json::Arr(bindings_out)),
            (
                "checks",
                Json::Arr(
                    check_accounts
                        .iter()
                        .map(|(check_id, account)| {
                            Json::obj(vec![
                                ("check_id", Json::str(check_id)),
                                (
                                    "scenarios",
                                    Json::Arr(account.scenarios.iter().map(Json::str).collect()),
                                ),
                                ("outcome", Json::str(account.outcome())),
                                ("runs", Json::Arr(account.runs.values().cloned().collect())),
                            ])
                        })
                        .collect(),
                ),
            ),
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

/// Binding reasons are prefixed with `check/scenario: ` and may quote ids in backticks; ids such as
/// `source-scan` must not steer the classification of the message itself.
fn reason_message(reason: &str) -> String {
    let message = match reason.split_once(": ") {
        Some((prefix, rest)) if !prefix.contains(' ') && prefix.contains('/') => rest,
        _ => reason,
    };
    let mut stripped = String::with_capacity(message.len());
    let mut quoted = false;
    for character in message.chars() {
        if character == '`' {
            quoted = !quoted;
        } else if !quoted {
            stripped.push(character);
        }
    }
    stripped
}

fn reason_code(reason: &str) -> &'static str {
    let reason = reason_message(reason);
    let reason = reason.as_str();
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

#[cfg(test)]
mod reason_code_tests {
    use super::reason_code;

    #[test]
    fn ids_inside_binding_reasons_do_not_steer_the_code() {
        assert_eq!(
            reason_code("source-scan/credential-delivery: no applicable current Check Result"),
            "evidence-missing"
        );
        assert_eq!(
            reason_code("component-artifact-check/a: no applicable current Check Result"),
            "evidence-missing"
        );
        assert_eq!(
            reason_code("Scenario `source-component` lacks applicable supporting evidence"),
            "evidence-missing"
        );
    }

    #[test]
    fn messages_keep_their_codes() {
        assert_eq!(
            reason_code("a/b: current Check Result is inconclusive"),
            "execution-inconclusive"
        );
        assert_eq!(
            reason_code("deployment observation is stale or future-dated"),
            "observation-future"
        );
        assert_eq!(
            reason_code("Verification Review is rejected"),
            "review-not-current"
        );
    }
}
