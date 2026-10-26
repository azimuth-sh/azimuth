//! Work Provenance ledger, Failure Attribution and dataset formats. See contracts/work-provenance.md.
//!
//! Provenance is a separate domain from assurance: nothing here is read by review currentness,
//! Requirement Status, acceptance or any workflow basis or attempt identity.

use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const DESCRIPTOR_FORMAT: &str = "azimuth-agent-descriptor";
pub const LEDGER_FORMAT: &str = "azimuth-work-provenance-ledger";
pub const ATTRIBUTION_FORMAT: &str = "azimuth-failure-attribution";
pub const ATTRIBUTIONS_FORMAT: &str = "azimuth-failure-attributions";

const BASES: &[&str] = &["harness", "declared", "unknown"];
const DISPOSITIONS: &[&str] = &["completed", "active", "superseded", "abandoned"];
const SCOPES: &[&str] = &[
    "requirement",
    "check",
    "verification-challenge",
    "module",
    "change",
];
const OUTCOMES: &[&str] = &[
    "completed",
    "approved",
    "denied",
    "deferred",
    "failed",
    "accepted",
    "rejected",
    "supported",
    "violated",
    "unresolved",
];
const GUIDANCE_KINDS: &[&str] = &["skill", "reference", "package"];
const ADVERSE_KINDS: &[&str] = &[
    "requirement-status",
    "check-result",
    "verification-challenge-result",
    "verification-review",
    "realization-review",
];
const CONFIDENCES: &[&str] = &["low", "medium", "high"];
/// Record kinds under which `workflow update` retains the authored result file itself.
const AUTHORED_RESULT_KINDS: &[&str] = &[
    "work-result",
    "failure-report",
    "implementation-result",
    "authoring-review",
    "implementation-approval",
];
const ATTEMPT_FIELDS: &[&str] = &[
    "id",
    "sequence",
    "action",
    "task",
    "scope",
    "target",
    "actor",
    "outcome",
    "report",
    "handoff_captured_at_ms",
    "recorded_at_ms",
    "agent",
    "guidance",
    "trace",
    "usage",
    "used",
    "generated",
];

/// Token fields of a usage window, recorded with the provider's own semantics: an OpenAI
/// `input_tokens` includes cached input, an Anthropic one excludes it. No cross-provider
/// normalization is applied.
pub const TOKEN_FIELDS: &[&str] = &[
    "input_tokens",
    "cache_read_input_tokens",
    "cache_write_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
];

// Strict-shape helpers

fn exact(value: &Json, names: &[&str], what: &str) -> Result<(), String> {
    let Json::Obj(fields) = value else {
        return Err(format!("{what} requires an object"));
    };
    let mut seen = BTreeSet::new();
    for (key, _) in fields {
        if !names.contains(&key.as_str()) {
            return Err(format!("{what} has unknown field `{key}`"));
        }
        if !seen.insert(key.as_str()) {
            return Err(format!("{what} has duplicate field `{key}`"));
        }
    }
    for name in names {
        if !seen.contains(name) {
            return Err(format!("{what} is missing field `{name}`"));
        }
    }
    Ok(())
}

fn nonempty<'a>(value: &'a Json, key: &str, what: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Json::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("{what} `{key}` requires a nonempty string"))
}

fn optional_text<'a>(value: &'a Json, key: &str, what: &str) -> Result<Option<&'a str>, String> {
    match value.get(key) {
        Some(Json::Null) => Ok(None),
        Some(Json::Str(text)) if !text.is_empty() => Ok(Some(text)),
        _ => Err(format!("{what} `{key}` requires null or a nonempty string")),
    }
}

fn safe_integer(value: &Json) -> Option<u64> {
    let number = value.as_num()?;
    (number >= 0.0 && number.fract() == 0.0 && number <= 9_007_199_254_740_991.0)
        .then_some(number as u64)
}

fn integer(value: &Json, key: &str, what: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(safe_integer)
        .ok_or_else(|| format!("{what} `{key}` requires a safe nonnegative integer"))
}

fn optional_integer(value: &Json, key: &str, what: &str) -> Result<Option<u64>, String> {
    match value.get(key) {
        Some(Json::Null) => Ok(None),
        _ => integer(value, key, what).map(Some),
    }
}

fn items<'a>(value: &'a Json, key: &str, what: &str) -> Result<&'a [Json], String> {
    value
        .get(key)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{what} `{key}` requires an array"))
}

fn sha256_fingerprint(text: &str) -> bool {
    text.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

fn fingerprint_field(value: &Json, key: &str, what: &str) -> Result<(), String> {
    if !sha256_fingerprint(nonempty(value, key, what)?) {
        return Err(format!(
            "{what} `{key}` requires a lowercase sha256 fingerprint"
        ));
    }
    Ok(())
}

fn duplicate_keys(value: &Json) -> Result<(), String> {
    match value {
        Json::Obj(fields) => {
            let mut seen = BTreeSet::new();
            for (key, value) in fields {
                if !seen.insert(key) {
                    return Err(format!("duplicate field `{key}`"));
                }
                duplicate_keys(value)?;
            }
        }
        Json::Arr(values) => values.iter().try_for_each(duplicate_keys)?,
        _ => {}
    }
    Ok(())
}

/// Parses a JSON file, rejecting duplicate keys as every Azimuth record format does.
pub fn read_json(path: &Path) -> Result<Json, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let value = crate::json::parse(&source).map_err(|e| format!("{}: {e}", path.display()))?;
    duplicate_keys(&value).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(value)
}

fn portable(reference: &str) -> bool {
    !reference.is_empty()
        && !reference.contains('\\')
        && !reference.contains('\0')
        && Path::new(reference)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

/// Single-line JSON in insertion order, for JSONL rows.
pub fn compact(value: &Json) -> String {
    fn write(value: &Json, out: &mut String) {
        match value {
            Json::Arr(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write(item, out);
                }
                out.push(']');
            }
            Json::Obj(fields) => {
                out.push('{');
                for (index, (key, item)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write(&Json::Str(key.clone()), out);
                    out.push(':');
                    write(item, out);
                }
                out.push('}');
            }
            scalar => out.push_str(scalar.to_string_pretty().trim_end()),
        }
    }
    let mut out = String::new();
    write(value, &mut out);
    out
}

// Agent descriptor

fn valued(value: &Json, key: &str, allowed: Option<&[&str]>) -> Result<(), String> {
    let what = format!("agent `{key}`");
    let field = value
        .get(key)
        .ok_or_else(|| format!("agent descriptor is missing field `{key}`"))?;
    exact(field, &["value", "basis"], &what)?;
    let basis = nonempty(field, "basis", &what)?;
    if !BASES.contains(&basis) {
        return Err(format!("{what} basis must be harness, declared or unknown"));
    }
    match (basis, field.get("value").unwrap()) {
        ("unknown", Json::Null) => Ok(()),
        ("unknown", _) => Err(format!("{what} with basis unknown requires a null value")),
        (_, Json::Str(text)) if !text.is_empty() => match allowed {
            Some(allowed) if !allowed.contains(&text.as_str()) => Err(format!(
                "{what} value must be one of {}",
                allowed.join(", ")
            )),
            _ => Ok(()),
        },
        _ => Err(format!(
            "{what} with basis {basis} requires a nonempty string value"
        )),
    }
}

fn harness(value: &Json) -> Result<(), String> {
    let field = value
        .get("harness")
        .ok_or("agent descriptor is missing field `harness`")?;
    exact(field, &["name", "version", "basis"], "agent `harness`")?;
    let basis = nonempty(field, "basis", "agent `harness`")?;
    if !BASES.contains(&basis) {
        return Err("agent `harness` basis must be harness, declared or unknown".into());
    }
    if basis == "unknown" {
        if field.get("name") != Some(&Json::Null) || field.get("version") != Some(&Json::Null) {
            return Err("agent `harness` with basis unknown requires null name and version".into());
        }
        return Ok(());
    }
    nonempty(field, "name", "agent `harness`")?;
    optional_text(field, "version", "agent `harness`")?;
    Ok(())
}

fn agent_fields(value: &Json) -> Result<(), String> {
    valued(value, "kind", Some(&["model", "human"]))?;
    valued(value, "model", None)?;
    valued(value, "effort", None)?;
    valued(value, "agent_type", None)?;
    harness(value)
}

pub const AGENT_FIELDS: &[&str] = &["kind", "model", "effort", "agent_type", "harness"];

/// The agent object carried by provenance entries, ledgers and attributions.
pub fn validate_agent(value: &Json) -> Result<(), String> {
    exact(value, AGENT_FIELDS, "agent")?;
    agent_fields(value)
}

fn token_counts(value: &Json, extra: &[&str], what: &str) -> Result<(), String> {
    let mut names: Vec<&str> = extra.to_vec();
    names.extend_from_slice(TOKEN_FIELDS);
    exact(value, &names, what)?;
    for name in extra {
        integer(value, name, what)?;
    }
    integer(value, "input_tokens", what)?;
    integer(value, "output_tokens", what)?;
    for name in [
        "cache_read_input_tokens",
        "cache_write_input_tokens",
        "reasoning_output_tokens",
    ] {
        optional_integer(value, name, what)?;
    }
    Ok(())
}

/// `null`, or the model work between the handoff and the recorded result.
pub fn validate_usage(value: &Json) -> Result<(), String> {
    if *value == Json::Null {
        return Ok(());
    }
    let what = "usage";
    let mut names = vec![
        "basis",
        "model_calls",
        "tool_calls",
        "overlapping",
        "delegated",
    ];
    names.extend_from_slice(TOKEN_FIELDS);
    exact(value, &names, what)?;
    if !matches!(nonempty(value, "basis", what)?, "harness" | "declared") {
        return Err("usage basis must be harness or declared".into());
    }
    integer(value, "model_calls", what)?;
    integer(value, "tool_calls", what)?;
    if value.get("overlapping").and_then(Json::as_bool).is_none() {
        return Err("usage `overlapping` requires a boolean".into());
    }
    let mut own = value.clone();
    if let Json::Obj(fields) = &mut own {
        fields.retain(|(key, _)| TOKEN_FIELDS.contains(&key.as_str()));
    }
    token_counts(&own, &[], what)?;
    token_counts(
        value.get("delegated").unwrap(),
        &["agents", "model_calls"],
        "usage `delegated`",
    )
}

/// Where a harness hook saves the descriptor for the attempt that a handoff file was produced
/// for. A hook may not rewrite the command (Codex applies a rewrite only together with a
/// permission grant), so `workflow update` finds the descriptor beside its own `--handoff`.
pub fn captured_descriptor_path(handoff: &Path) -> PathBuf {
    let mut name = handoff.file_name().unwrap_or_default().to_os_string();
    name.push(".agent-descriptor.json");
    handoff.with_file_name(name)
}

/// The descriptor a hook saved for this handoff, if any. Agents reuse one handoff path across
/// attempts, so a descriptor saved before the handoff was captured belongs to an earlier
/// attempt and is ignored.
pub fn captured_descriptor(handoff: &Path, captured_at_ms: u64) -> Result<Option<Json>, String> {
    let path = captured_descriptor_path(handoff);
    let Ok(metadata) = fs::metadata(&path) else {
        return Ok(None);
    };
    let saved_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|elapsed| elapsed.as_millis() as u64);
    if saved_ms.is_none_or(|saved| saved < captured_at_ms) {
        return Ok(None);
    }
    let value = read_json(&path)?;
    validate_descriptor(&value).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Some(value))
}

/// The `azimuth-agent-descriptor` version 1 file accepted by `workflow update --descriptor`.
pub fn validate_descriptor(value: &Json) -> Result<(), String> {
    exact(
        value,
        &[
            "format",
            "version",
            "kind",
            "model",
            "effort",
            "agent_type",
            "harness",
            "trace",
            "usage",
        ],
        "agent descriptor",
    )?;
    if value.get("format").and_then(Json::as_str) != Some(DESCRIPTOR_FORMAT)
        || value.get("version").and_then(safe_integer) != Some(1)
    {
        return Err(format!("expected {DESCRIPTOR_FORMAT} version 1"));
    }
    agent_fields(value)?;
    optional_text(value, "trace", "agent descriptor")?;
    validate_usage(value.get("usage").unwrap())
}

pub fn unknown_agent() -> Json {
    let unknown = || Json::obj(vec![("value", Json::Null), ("basis", Json::str("unknown"))]);
    Json::obj(vec![
        ("kind", unknown()),
        ("model", unknown()),
        ("effort", unknown()),
        ("agent_type", unknown()),
        (
            "harness",
            Json::obj(vec![
                ("name", Json::Null),
                ("version", Json::Null),
                ("basis", Json::str("unknown")),
            ]),
        ),
    ])
}

fn descriptor_agent(descriptor: &Json) -> Json {
    Json::obj(
        AGENT_FIELDS
            .iter()
            .copied()
            .map(|key| (key, descriptor.get(key).unwrap().clone()))
            .collect(),
    )
}

// Workflow capture

fn shipped_guidance(kind: &str, id: &str) -> Option<&'static str> {
    match kind {
        "skill" => crate::resources::SKILLS
            .iter()
            .find(|skill| skill.id == id)
            .map(|skill| skill.source),
        "reference" => crate::resources::reference(id).map(|reference| reference.prose),
        _ => None,
    }
}

/// Guidance as supplied to the attempt. Only content this CLI ships can be fingerprinted; a package
/// locator or a resource this release does not embed is recorded with a null fingerprint rather
/// than a guess about what the harness actually resolved.
pub fn guidance_fingerprints(guidance: &[Json]) -> Result<Vec<Json>, String> {
    guidance
        .iter()
        .map(|entry| {
            exact(entry, &["kind", "id"], "handoff guidance")?;
            let kind = nonempty(entry, "kind", "handoff guidance")?;
            let id = nonempty(entry, "id", "handoff guidance")?;
            if !GUIDANCE_KINDS.contains(&kind) {
                return Err("handoff guidance kind must be skill, reference or package".into());
            }
            Ok(Json::obj(vec![
                ("kind", Json::str(kind)),
                ("id", Json::str(id)),
                (
                    "fingerprint",
                    shipped_guidance(kind, id)
                        .map(|source| {
                            Json::str(format!(
                                "sha256:{}",
                                crate::fingerprint::sha256(source.as_bytes())
                            ))
                        })
                        .unwrap_or(Json::Null),
                ),
            ]))
        })
        .collect()
}

/// The provenance sibling of a recorded workflow result.
pub fn history_provenance(
    handoff_captured_at_ms: u64,
    recorded_at_ms: u64,
    descriptor: Option<&Json>,
    guidance: &[Json],
) -> Result<Json, String> {
    let (agent, trace, usage) = match descriptor {
        Some(descriptor) => {
            validate_descriptor(descriptor)?;
            (
                descriptor_agent(descriptor),
                descriptor.get("trace").unwrap().clone(),
                descriptor.get("usage").unwrap().clone(),
            )
        }
        None => (unknown_agent(), Json::Null, Json::Null),
    };
    Ok(Json::obj(vec![
        (
            "handoff_captured_at_ms",
            Json::Num(handoff_captured_at_ms as f64),
        ),
        ("recorded_at_ms", Json::Num(recorded_at_ms as f64)),
        ("agent", agent),
        ("guidance", Json::Arr(guidance_fingerprints(guidance)?)),
        ("trace", trace),
        ("usage", usage),
    ]))
}

fn guidance_entries(value: &Json, key: &str, what: &str) -> Result<(), String> {
    for entry in items(value, key, what)? {
        exact(entry, &["kind", "id", "fingerprint"], "guidance entry")?;
        if !GUIDANCE_KINDS.contains(&nonempty(entry, "kind", "guidance entry")?) {
            return Err("guidance entry kind must be skill, reference or package".into());
        }
        nonempty(entry, "id", "guidance entry")?;
        if entry.get("fingerprint") != Some(&Json::Null) {
            fingerprint_field(entry, "fingerprint", "guidance entry")?;
        }
    }
    Ok(())
}

pub fn validate_history_provenance(value: &Json) -> Result<(), String> {
    let what = "history provenance";
    exact(
        value,
        &[
            "handoff_captured_at_ms",
            "recorded_at_ms",
            "agent",
            "guidance",
            "trace",
            "usage",
        ],
        what,
    )?;
    integer(value, "handoff_captured_at_ms", what)?;
    integer(value, "recorded_at_ms", what)?;
    validate_agent(value.get("agent").unwrap())?;
    guidance_entries(value, "guidance", what)?;
    optional_text(value, "trace", what)?;
    validate_usage(value.get("usage").unwrap())
}

/// The ledger identity of an instance. Version 1 and 2 instances were all named after their
/// definition, so historical generations of one change would collide; their content fingerprint
/// disambiguates them deterministically.
pub fn instance_identity(value: &Json) -> Result<String, String> {
    let id = nonempty(value, "id", "workflow instance")?;
    match value.get("version").and_then(safe_integer) {
        Some(3) => Ok(id.to_string()),
        Some(1 | 2) => Ok(format!(
            "{id}-{}",
            &crate::fingerprint::canonical_sha256(value)[7..19]
        )),
        _ => Err("expected workflow instance version 1, 2 or 3".into()),
    }
}

fn instance_header(value: &Json) -> Result<u64, String> {
    if value.get("format").and_then(Json::as_str) != Some("azimuth-workflow-instance") {
        return Err("expected azimuth-workflow-instance".into());
    }
    let version = integer(value, "version", "workflow instance")?;
    if !(1..=3).contains(&version) {
        return Err(format!(
            "workflow instance version {version} is not readable; expected version 1, 2 or 3"
        ));
    }
    nonempty(value, "id", "workflow instance")?;
    integer(value, "revision", "workflow instance")?;
    items(value, "history", "workflow instance")?;
    if !matches!(value.get("definition"), Some(Json::Obj(_))) {
        return Err("workflow instance `definition` requires an object".into());
    }
    Ok(version)
}

/// The `predecessor` record a fresh instance keeps for the instance file it replaces.
pub fn predecessor_record(predecessor: &Path, instance_directory: &Path) -> Result<Json, String> {
    let value = read_json(predecessor)?;
    instance_header(&value).map_err(|e| format!("{}: {e}", predecessor.display()))?;
    let actual =
        fs::canonicalize(predecessor).map_err(|e| format!("{}: {e}", predecessor.display()))?;
    let directory = fs::canonicalize(instance_directory)
        .map_err(|e| format!("{}: {e}", instance_directory.display()))?;
    let relative = actual.strip_prefix(&directory).map_err(|_| {
        format!(
            "{}: the predecessor must be retained inside the new instance directory {}",
            predecessor.display(),
            directory.display()
        )
    })?;
    let reference = relative.to_string_lossy().replace('\\', "/");
    if !portable(&reference) {
        return Err(format!(
            "{}: predecessor reference `{reference}` is not a normal relative path",
            predecessor.display()
        ));
    }
    let identity =
        instance_identity(&value).map_err(|e| format!("{}: {e}", predecessor.display()))?;
    Ok(Json::obj(vec![
        ("id", Json::str(identity)),
        ("ref", Json::str(reference)),
        (
            "revision",
            Json::Num(integer(&value, "revision", "workflow instance")? as f64),
        ),
        (
            "fingerprint",
            Json::str(crate::fingerprint::canonical_sha256(&value)),
        ),
    ]))
}

fn predecessor_shape(value: &Json, what: &str) -> Result<(), String> {
    if value == &Json::Null {
        return Ok(());
    }
    exact(value, &["id", "ref", "revision", "fingerprint"], what)?;
    nonempty(value, "id", what)?;
    if !portable(nonempty(value, "ref", what)?) {
        return Err(format!("{what} `ref` requires a normal relative path"));
    }
    integer(value, "revision", what)?;
    fingerprint_field(value, "fingerprint", what)
}

pub fn validate_predecessor(value: &Json) -> Result<(), String> {
    predecessor_shape(value, "workflow predecessor")
}

pub fn validate_abandonment(value: &Json) -> Result<(), String> {
    if value == &Json::Null {
        return Ok(());
    }
    exact(value, &["at_ms", "reason"], "workflow abandonment")?;
    integer(value, "at_ms", "workflow abandonment")?;
    nonempty(value, "reason", "workflow abandonment")?;
    Ok(())
}

// Ledger export

struct Instance {
    path: PathBuf,
    value: Json,
    fingerprint: String,
    identity: String,
}

fn load_instance(path: &Path) -> Result<Instance, String> {
    let value = read_json(path)?;
    instance_header(&value).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Instance {
        path: fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?,
        fingerprint: crate::fingerprint::canonical_sha256(&value),
        identity: instance_identity(&value).map_err(|e| format!("{}: {e}", path.display()))?,
        value,
    })
}

fn first_recorded_at(instance: &Instance) -> Option<u64> {
    instance
        .value
        .get("history")?
        .as_array()?
        .first()?
        .get("provenance")?
        .get("recorded_at_ms")
        .and_then(safe_integer)
}

fn definition_identity(instance: &Instance, key: &str) -> Result<String, String> {
    let definition = instance.value.get("definition").unwrap();
    if let Some(value) = definition
        .get("parameters")
        .and_then(|parameters| parameters.get(key))
        .and_then(Json::as_str)
    {
        return Ok(value.to_string());
    }
    let named = definition
        .get("tasks")
        .and_then(Json::as_array)
        .unwrap_or(&[])
        .iter()
        .filter_map(|task| task.get("target")?.get(key)?.as_str())
        .collect::<BTreeSet<_>>();
    match named.len() {
        1 => Ok(named.into_iter().next().unwrap().to_string()),
        count => Err(format!(
            "{}: cannot determine the {key}: the definition has no parameters and its task targets name {count} distinct {key} values",
            instance.path.display()
        )),
    }
}

fn flattened_target(target: &Json) -> Result<Json, String> {
    let Json::Obj(fields) = target else {
        return Err("result task target requires an object".into());
    };
    let mut pairs = fields
        .iter()
        .map(|(name, value)| {
            let rendered = match value {
                Json::Str(text) => text.clone(),
                nested => crate::fingerprint::canonical_json(nested)
                    .trim_end()
                    .to_string(),
            };
            (name.clone(), rendered)
        })
        .collect::<Vec<_>>();
    pairs.sort();
    Ok(Json::Arr(
        pairs
            .into_iter()
            .map(|(name, value)| {
                Json::obj(vec![("name", Json::str(name)), ("value", Json::str(value))])
            })
            .collect(),
    ))
}

fn target_names(target: &[Json]) -> BTreeSet<&str> {
    target
        .iter()
        .filter_map(|pair| pair.get("name").and_then(Json::as_str))
        .collect()
}

fn scope_of(names: &BTreeSet<&str>) -> &'static str {
    if names.contains("requirement") {
        "requirement"
    } else if names.contains("check") {
        "check"
    } else if names.contains("verification_challenge") {
        "verification-challenge"
    } else if names.contains("module") {
        "module"
    } else {
        "change"
    }
}

/// Where recorded references resolve: workflow-local references beneath the instance directory,
/// repository references through the local repository map.
struct Roots<'a> {
    workflow: &'a Path,
    repositories: &'a BTreeMap<String, PathBuf>,
}

/// The selected review-bundle entry's native record fingerprint. The entry is accepted only while
/// its content still hashes to the fingerprint recorded at update, which is the content `workflow
/// update` validated; anything else would link the attempt to a review it never produced.
fn record_fingerprint(reference: &Json, record: &str, roots: &Roots) -> Result<String, String> {
    let reference_text = nonempty(reference, "ref", "recorded reference")?;
    let base = match reference.get("repository").and_then(Json::as_str) {
        None => roots.workflow.to_path_buf(),
        Some(repository) => roots.repositories.get(repository).cloned().ok_or_else(|| {
            format!(
                "review bundle {reference_text} names repository `{repository}`, which no local repository map supplies; pass --repositories or provide azimuth/local-repositories.json"
            )
        })?,
    };
    if !portable(reference_text) {
        return Err(format!(
            "review bundle reference `{reference_text}` is not a normal relative path"
        ));
    }
    let original = base.join(reference_text);
    let path = match fs::symlink_metadata(&original) {
        Ok(metadata) => {
            let canonical = fs::canonicalize(&original).map_err(|error| error.to_string())?;
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || canonical != original
                || !canonical.starts_with(&base)
            {
                return Err(
                    "recorded review reference must be a regular file without symlink/root escape"
                        .into(),
                );
            }
            original
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            crate::workflow_references::relocated(roots.workflow, &base, reference_text)?
                .ok_or_else(|| format!("{}: {error}", original.display()))?
        }
        Err(error) => return Err(format!("{}: {error}", original.display())),
    };
    let bundle = read_json(&path)?;
    let entry = bundle
        .get("records")
        .and_then(Json::as_array)
        .unwrap_or(&[])
        .iter()
        .find(|entry| entry.get("id").and_then(Json::as_str) == Some(record))
        .ok_or_else(|| {
            format!(
                "{}: selected review record `{record}` is absent",
                path.display()
            )
        })?;
    let Json::Obj(fields) = entry else {
        return Err(format!(
            "{}: review record `{record}` requires an object",
            path.display()
        ));
    };
    let selected = Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "assessment")
            .cloned()
            .collect(),
    );
    let digest = format!(
        "sha256:{}",
        crate::fingerprint::sha256(crate::fingerprint::canonical_json(&selected).as_bytes())
    );
    if Some(digest.as_str()) != reference.get("fingerprint").and_then(Json::as_str) {
        return Err(format!(
            "{}: review record `{record}` differs from the entry the attempt recorded",
            path.display()
        ));
    }
    let native = entry
        .get("record")
        .and_then(|native| native.get("fingerprint"))
        .and_then(Json::as_str)
        .filter(|fingerprint| sha256_fingerprint(fingerprint))
        .ok_or_else(|| {
            format!(
                "{}: review record `{record}` has no native record fingerprint",
                path.display()
            )
        })?;
    Ok(native.to_string())
}

fn entity(reference: &Json, roots: Option<&Roots>) -> Result<Json, String> {
    let what = "recorded reference";
    let mut fields = vec![
        ("kind", Json::str(nonempty(reference, "kind", what)?)),
        (
            "repository",
            reference.get("repository").cloned().unwrap_or(Json::Null),
        ),
        ("ref", Json::str(nonempty(reference, "ref", what)?)),
        (
            "fingerprint",
            Json::str(nonempty(reference, "fingerprint", what)?),
        ),
    ];
    if let Some(roots) = roots {
        let record = match reference.get("record") {
            None => None,
            Some(_) => Some(nonempty(reference, "record", what)?),
        };
        fields.push(("record", record.map(Json::str).unwrap_or(Json::Null)));
        fields.push((
            "record_fingerprint",
            match record {
                Some(record) => Json::str(record_fingerprint(reference, record, roots)?),
                None => Json::Null,
            },
        ));
    }
    Ok(Json::obj(fields))
}

/// The authored result's `report`, read back only while the retained file still has the recorded
/// bytes; a missing or altered file yields null rather than a report the attempt never submitted.
fn authored_report(records: &[Json], workflow_directory: &Path) -> Json {
    let local = |kind: &str| {
        records.iter().find(|record| {
            record.get("kind").and_then(Json::as_str) == Some(kind)
                && record.get("repository").is_none()
        })
    };
    let Some(record) = AUTHORED_RESULT_KINDS.iter().find_map(|kind| local(kind)) else {
        return Json::Null;
    };
    let (Some(reference), Some(expected)) = (
        record.get("ref").and_then(Json::as_str),
        record.get("fingerprint").and_then(Json::as_str),
    ) else {
        return Json::Null;
    };
    if !portable(reference) {
        return Json::Null;
    }
    let Ok(bytes) = fs::read(workflow_directory.join(reference)) else {
        return Json::Null;
    };
    if format!("sha256:{}", crate::fingerprint::sha256(&bytes)) != expected {
        return Json::Null;
    }
    std::str::from_utf8(&bytes)
        .ok()
        .and_then(|source| crate::json::parse(source).ok())
        .and_then(|value| value.get("report").cloned())
        .filter(|report| report.as_str().is_some_and(|text| !text.is_empty()))
        .unwrap_or(Json::Null)
}

fn attempts(
    instance: &Instance,
    repositories: &BTreeMap<String, PathBuf>,
) -> Result<Vec<Json>, String> {
    let fail = |error: String| format!("{}: {error}", instance.path.display());
    let directory = instance.path.parent().unwrap_or(Path::new("."));
    let tasks = instance
        .value
        .get("definition")
        .unwrap()
        .get("tasks")
        .and_then(Json::as_array)
        .ok_or_else(|| fail("definition `tasks` requires an array".into()))?;
    let history = items(&instance.value, "history", "workflow instance").map_err(&fail)?;
    let mut attempts = Vec::new();
    for (index, entry) in history.iter().enumerate() {
        let what = format!("history entry {}", index + 1);
        let result = entry
            .get("result")
            .ok_or_else(|| fail(format!("{what} has no result")))?;
        let work = result
            .get("task")
            .ok_or_else(|| fail(format!("{what} result has no task")))?;
        let work_key = crate::fingerprint::canonical_json(work);
        let task = tasks
            .iter()
            .find(|task| {
                task.get("id")
                    .is_some_and(|id| crate::fingerprint::canonical_json(id) == work_key)
            })
            .ok_or_else(|| fail(format!("{what} names a task absent from the definition")))?;
        // Finalization consumes this ledger; its own receipt is retained separately to avoid a
        // completion record changing the very input it certifies.
        if task.get("kind").and_then(Json::as_str) == Some("finalize-change")
            || work.get("action").and_then(Json::as_str) == Some("finalize-change")
        {
            continue;
        }
        let target = flattened_target(
            work.get("target")
                .ok_or_else(|| fail(format!("{what} result task has no target")))?,
        )
        .map_err(&fail)?;
        let scope = scope_of(&target_names(target.as_array().unwrap()));
        let records = items(result, "records", &what).map_err(&fail)?;
        let inputs = result.get("inputs").and_then(Json::as_array).unwrap_or(&[]);
        let provenance = entry.get("provenance");
        let recorded = |key: &str, absent: Json| {
            provenance
                .and_then(|provenance| provenance.get(key))
                .cloned()
                .unwrap_or(absent)
        };
        let used = inputs
            .iter()
            .map(|reference| entity(reference, None))
            .collect::<Result<Vec<_>, _>>()
            .map_err(&fail)?;
        let generated = records
            .iter()
            .map(|reference| {
                entity(
                    reference,
                    Some(&Roots {
                        workflow: directory,
                        repositories,
                    }),
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(&fail)?;
        attempts.push(Json::obj(vec![
            (
                "id",
                Json::str(nonempty(result, "id", &what).map_err(&fail)?),
            ),
            ("sequence", Json::Num((attempts.len() + 1) as f64)),
            (
                "action",
                Json::str(nonempty(work, "action", &what).map_err(&fail)?),
            ),
            (
                "task",
                Json::str(nonempty(task, "kind", "task").map_err(&fail)?),
            ),
            ("scope", Json::str(scope)),
            ("target", target),
            (
                "actor",
                Json::str(nonempty(result, "actor", &what).map_err(&fail)?),
            ),
            (
                "outcome",
                Json::str(nonempty(result, "outcome", &what).map_err(&fail)?),
            ),
            ("report", authored_report(records, directory)),
            (
                "handoff_captured_at_ms",
                recorded("handoff_captured_at_ms", Json::Null),
            ),
            ("recorded_at_ms", recorded("recorded_at_ms", Json::Null)),
            ("agent", recorded("agent", unknown_agent())),
            ("guidance", recorded("guidance", Json::Arr(vec![]))),
            ("trace", recorded("trace", Json::Null)),
            ("usage", recorded("usage", Json::Null)),
            ("used", Json::Arr(used)),
            ("generated", Json::Arr(generated)),
        ]));
    }
    Ok(attempts)
}

fn change_completed(change: &Path) -> bool {
    change.join("finalization.json").is_file()
        || fs::canonicalize(change)
            .ok()
            .and_then(|path| {
                path.parent()
                    .and_then(|parent| parent.file_name())
                    .map(|name| name == "archive")
            })
            .unwrap_or(false)
}

pub fn default_instance(change: &Path) -> PathBuf {
    change.join(".azimuth").join("workflow.json")
}

pub fn default_ledger(change: &Path) -> PathBuf {
    change.join("provenance").join("ledger.json")
}

fn predecessor_chain(current: &Path) -> Result<Vec<Instance>, String> {
    let mut chain = Vec::new();
    let mut visited = BTreeSet::new();
    let mut next = Some(load_instance(current)?);
    while let Some(instance) = next.take() {
        if !visited.insert(instance.path.clone()) {
            return Err(format!(
                "{}: predecessor links form a cycle",
                instance.path.display()
            ));
        }
        let predecessor = instance
            .value
            .get("predecessor")
            .cloned()
            .unwrap_or(Json::Null);
        predecessor_shape(&predecessor, "workflow predecessor")
            .map_err(|e| format!("{}: {e}", instance.path.display()))?;
        if predecessor != Json::Null {
            let path = instance
                .path
                .parent()
                .unwrap()
                .join(predecessor.get("ref").and_then(Json::as_str).unwrap());
            let loaded = load_instance(&path).map_err(|e| {
                format!(
                    "{e} (the predecessor recorded by {} must remain retained)",
                    instance.path.display()
                )
            })?;
            if predecessor.get("fingerprint").and_then(Json::as_str)
                != Some(loaded.fingerprint.as_str())
            {
                return Err(format!(
                    "{}: content differs from the predecessor fingerprint recorded by {}",
                    path.display(),
                    instance.path.display()
                ));
            }
            if predecessor.get("id").and_then(Json::as_str) != Some(loaded.identity.as_str()) {
                return Err(format!(
                    "{}: instance identity `{}` differs from the predecessor id recorded by {}",
                    path.display(),
                    loaded.identity,
                    instance.path.display()
                ));
            }
            next = Some(loaded);
        }
        chain.push(instance);
    }
    chain.reverse();
    Ok(chain)
}

fn exported_instance(
    instance: &Instance,
    disposition: &str,
    repositories: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    let located = |error: String| format!("{}: {error}", instance.path.display());
    let definition = instance.value.get("definition").unwrap();
    let template = match definition.get("template") {
        Some(template) => Json::obj(vec![
            (
                "id",
                Json::str(nonempty(template, "id", "definition template").map_err(&located)?),
            ),
            (
                "revision",
                Json::Num(
                    integer(template, "revision", "definition template").map_err(&located)? as f64,
                ),
            ),
        ]),
        None => Json::Null,
    };
    Ok(Json::obj(vec![
        ("id", Json::str(&instance.identity)),
        (
            "predecessor",
            instance
                .value
                .get("predecessor")
                .cloned()
                .unwrap_or(Json::Null),
        ),
        ("disposition", Json::str(disposition)),
        (
            "definition_fingerprint",
            Json::str(
                nonempty(
                    &instance.value,
                    "definition_fingerprint",
                    "workflow instance",
                )
                .map_err(&located)?,
            ),
        ),
        ("template", template),
        ("attempts", Json::Arr(attempts(instance, repositories)?)),
    ]))
}

/// Exports the change's ledger from its current instance, its predecessor chain and any extra
/// historical instances.
/// The conventional project-local map, `azimuth/local-repositories.json`, of the repository that
/// holds the change.
fn conventional_repositories(change: &Path) -> Option<PathBuf> {
    let change = fs::canonicalize(change).ok()?;
    let azimuth = change
        .ancestors()
        .find(|ancestor| {
            ancestor.file_name().is_some_and(|name| name == "changes")
                && ancestor
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|name| name == "azimuth")
        })?
        .parent()?;
    Some(azimuth.join("local-repositories.json")).filter(|path| path.is_file())
}

fn repository_map(
    change: &Path,
    repositories: Option<&Path>,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let Some(path) = repositories
        .map(Path::to_path_buf)
        .or_else(|| conventional_repositories(change))
    else {
        return Ok(BTreeMap::new());
    };
    let value = read_json(&path)?;
    let located = |error: String| format!("{}: {error}", path.display());
    exact(&value, &["repositories"], "local repository map").map_err(&located)?;
    let Some(Json::Obj(entries)) = value.get("repositories") else {
        return Err(located("`repositories` requires an object".into()));
    };
    entries
        .iter()
        .map(|(id, location)| {
            let location = location
                .as_str()
                .filter(|location| Path::new(location).is_absolute())
                .ok_or_else(|| located(format!("repository `{id}` requires an absolute path")))?;
            Ok((id.clone(), PathBuf::from(location)))
        })
        .collect()
}

pub fn export_ledger(
    change: &Path,
    extra: &[PathBuf],
    repositories: Option<&Path>,
) -> Result<Json, String> {
    let repositories = repository_map(change, repositories)?;
    let current = default_instance(change);
    let chain = if current.is_file() {
        predecessor_chain(&current)?
    } else if extra.is_empty() {
        return Err(format!(
            "{} is missing; pass --instance for historical instances",
            current.display()
        ));
    } else {
        Vec::new()
    };
    let mut historical: Vec<Instance> = Vec::new();
    for path in extra {
        let instance = load_instance(path)?;
        if chain
            .iter()
            .chain(historical.iter())
            .any(|known| known.path == instance.path)
        {
            continue;
        }
        historical.push(instance);
    }
    // Historical instances predate predecessor links; untimed version 2 histories keep the order
    // in which they were named.
    historical.sort_by_key(first_recorded_at);
    let instances = historical.into_iter().chain(chain).collect::<Vec<_>>();

    let project = definition_identity(&instances[0], "project")?;
    let change_id = definition_identity(&instances[0], "change")?;
    for instance in &instances[1..] {
        if definition_identity(instance, "project")? != project
            || definition_identity(instance, "change")? != change_id
        {
            return Err(format!(
                "{}: instance belongs to a different project or change than {}",
                instance.path.display(),
                instances[0].path.display()
            ));
        }
    }
    let completed = change_completed(change);
    let mut exported = Vec::new();
    for (index, instance) in instances.iter().enumerate() {
        let abandoned = instance
            .value
            .get("abandoned")
            .is_some_and(|value| value != &Json::Null);
        let named_as_predecessor = instances.iter().any(|other| {
            other
                .value
                .get("predecessor")
                .and_then(|predecessor| predecessor.get("fingerprint"))
                .and_then(Json::as_str)
                == Some(instance.fingerprint.as_str())
        });
        let disposition = if abandoned {
            "abandoned"
        } else if named_as_predecessor || index + 1 < instances.len() {
            "superseded"
        } else if completed {
            "completed"
        } else {
            "active"
        };
        exported.push(exported_instance(instance, disposition, &repositories)?);
    }
    let ledger = Json::obj(vec![
        ("format", Json::str(LEDGER_FORMAT)),
        ("version", Json::Num(1.0)),
        ("project", Json::str(project)),
        ("change", Json::str(change_id)),
        ("instances", Json::Arr(exported)),
    ]);
    validate_ledger(&ledger)
        .map_err(|e| format!("exported ledger for {}: {e}", change.display()))?;
    Ok(ledger)
}

/// Exported attempts are immutable: a re-export may append, never rewrite or drop.
fn append_only(previous: &Json, next: &Json) -> Result<(), String> {
    let attempts = |ledger: &Json| {
        ledger
            .get("instances")
            .and_then(Json::as_array)
            .unwrap_or(&[])
            .iter()
            .flat_map(|instance| {
                instance
                    .get("attempts")
                    .and_then(Json::as_array)
                    .unwrap_or(&[])
                    .to_vec()
            })
            .collect::<Vec<_>>()
    };
    let current = attempts(next);
    for attempt in attempts(previous) {
        let id = attempt.get("id").and_then(Json::as_str).unwrap_or("");
        match current
            .iter()
            .find(|candidate| candidate.get("id") == attempt.get("id"))
        {
            None => {
                return Err(format!(
                    "previously exported attempt {id} is missing from the new export; pass its historical instance with --instance"
                ));
            }
            Some(candidate) if candidate != &attempt => {
                return Err(format!(
                    "previously exported attempt {id} would change; exported attempts are immutable"
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("{} has no file name", path.display()))?
        .to_string_lossy();
    let temporary = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
    fs::write(&temporary, bytes).map_err(|e| format!("{}: {e}", temporary.display()))?;
    fs::rename(&temporary, path).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        format!("{}: {e}", path.display())
    })
}

/// Exports and writes the ledger, refusing to rewrite attempts an existing ledger already holds.
pub fn write_ledger(
    change: &Path,
    extra: &[PathBuf],
    repositories: Option<&Path>,
    out: Option<&Path>,
) -> Result<(PathBuf, Json), String> {
    let ledger = export_ledger(change, extra, repositories)?;
    let path = out
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_ledger(change));
    if out.is_none() {
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    if path.exists() {
        let previous = read_json(&path)?;
        validate_ledger(&previous).map_err(|e| format!("{}: {e}", path.display()))?;
        append_only(&previous, &ledger).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    write_atomic(&path, ledger.to_string_pretty().as_bytes())?;
    Ok((path, ledger))
}

/// Why a change's committed ledger cannot be archived, if anything.
pub fn committed_ledger_issues(change: &Path) -> Vec<String> {
    let committed = default_ledger(change);
    if !default_instance(change).is_file() {
        // Without local workflow state there is nothing to re-export; the committed ledger must
        // still be a well-formed record.
        if !committed.is_file() {
            return vec![];
        }
        return match read_json(&committed).and_then(|value| {
            validate_ledger(&value).map_err(|e| format!("{}: {e}", committed.display()))
        }) {
            Ok(()) => vec![],
            Err(error) => vec![error],
        };
    }
    let Ok(recorded) = fs::read_to_string(&committed) else {
        return vec![format!(
            "{} is missing; run `azimuth change finalize` to export Work Provenance",
            committed.display()
        )];
    };
    match export_ledger(change, &[], None) {
        Ok(fresh) if recorded == fresh.to_string_pretty() => vec![],
        Ok(_) => vec![format!(
            "{} differs from a fresh export; run `azimuth change finalize` again",
            committed.display()
        )],
        Err(error) => vec![error],
    }
}

// Ledger validation

fn entity_shape(value: &Json, with_record: bool, what: &str) -> Result<(), String> {
    let mut names = vec!["kind", "repository", "ref", "fingerprint"];
    if with_record {
        names.extend(["record", "record_fingerprint"]);
    }
    exact(value, &names, what)?;
    nonempty(value, "kind", what)?;
    optional_text(value, "repository", what)?;
    if !portable(nonempty(value, "ref", what)?) {
        return Err(format!("{what} `ref` requires a normal relative path"));
    }
    fingerprint_field(value, "fingerprint", what)?;
    if with_record {
        let record = optional_text(value, "record", what)?;
        match (record, value.get("record_fingerprint").unwrap()) {
            (None, Json::Null) => {}
            (None, _) => {
                return Err(format!(
                    "{what} `record_fingerprint` must be null when no record is selected"
                ));
            }
            (Some(_), _) => fingerprint_field(value, "record_fingerprint", what)?,
        }
    }
    Ok(())
}

fn attempt_shape(attempt: &Json, sequence: usize) -> Result<(), String> {
    let what = "attempt";
    exact(attempt, ATTEMPT_FIELDS, what)?;
    nonempty(attempt, "id", what)?;
    if integer(attempt, "sequence", what)? != sequence as u64 {
        return Err(format!(
            "attempt `sequence` must be {sequence}, its 1-based position in the instance history"
        ));
    }
    for key in ["action", "task", "actor"] {
        nonempty(attempt, key, what)?;
    }
    if !OUTCOMES.contains(&nonempty(attempt, "outcome", what)?) {
        return Err("attempt `outcome` is not a workflow outcome".into());
    }
    let target = items(attempt, "target", what)?;
    let mut previous: Option<&str> = None;
    for pair in target {
        exact(pair, &["name", "value"], "attempt target pair")?;
        let name = nonempty(pair, "name", "attempt target pair")?;
        nonempty(pair, "value", "attempt target pair")?;
        if previous.is_some_and(|previous| previous >= name) {
            return Err("attempt `target` requires unique names in sorted order".into());
        }
        previous = Some(name);
    }
    let scope = nonempty(attempt, "scope", what)?;
    if !SCOPES.contains(&scope) {
        return Err(
            "attempt `scope` must be requirement, check, verification-challenge, module or change"
                .into(),
        );
    }
    if scope != scope_of(&target_names(target)) {
        return Err(format!(
            "attempt `scope` {scope} differs from the scope its target names"
        ));
    }
    optional_text(attempt, "report", what)?;
    optional_integer(attempt, "handoff_captured_at_ms", what)?;
    optional_integer(attempt, "recorded_at_ms", what)?;
    validate_agent(attempt.get("agent").unwrap())?;
    guidance_entries(attempt, "guidance", what)?;
    optional_text(attempt, "trace", what)?;
    validate_usage(attempt.get("usage").unwrap())?;
    for used in items(attempt, "used", what)? {
        entity_shape(used, false, "attempt used entity")?;
    }
    for generated in items(attempt, "generated", what)? {
        entity_shape(generated, true, "attempt generated entity")?;
    }
    Ok(())
}

/// Strict validation of an `azimuth-work-provenance-ledger` version 1 document, self-contained on
/// the JSON so a server can apply it on ingestion.
pub fn validate_ledger(value: &Json) -> Result<(), String> {
    exact(
        value,
        &["format", "version", "project", "change", "instances"],
        "ledger",
    )?;
    if value.get("format").and_then(Json::as_str) != Some(LEDGER_FORMAT)
        || value.get("version").and_then(safe_integer) != Some(1)
    {
        return Err(format!("expected {LEDGER_FORMAT} version 1"));
    }
    nonempty(value, "project", "ledger")?;
    nonempty(value, "change", "ledger")?;
    let instances = items(value, "instances", "ledger")?;
    if instances.is_empty() {
        return Err("ledger requires at least one instance".into());
    }
    let mut instance_ids = BTreeSet::new();
    let mut attempt_ids = BTreeSet::new();
    for (index, instance) in instances.iter().enumerate() {
        let located = |error: String| format!("ledger instance {}: {error}", index + 1);
        exact(
            instance,
            &[
                "id",
                "predecessor",
                "disposition",
                "definition_fingerprint",
                "template",
                "attempts",
            ],
            "instance",
        )
        .map_err(&located)?;
        let id = nonempty(instance, "id", "instance").map_err(&located)?;
        let predecessor = instance.get("predecessor").unwrap();
        predecessor_shape(predecessor, "instance predecessor").map_err(&located)?;
        if let Some(previous) = predecessor.get("id").and_then(Json::as_str) {
            if !instance_ids.contains(previous) {
                return Err(located(format!(
                    "predecessor `{previous}` must name an earlier instance in this ledger"
                )));
            }
        }
        if !instance_ids.insert(id) {
            return Err(located(format!("duplicate instance id `{id}`")));
        }
        let disposition = nonempty(instance, "disposition", "instance").map_err(&located)?;
        if !DISPOSITIONS.contains(&disposition) {
            return Err(located(
                "disposition must be completed, active, superseded or abandoned".into(),
            ));
        }
        let last = index + 1 == instances.len();
        if (disposition == "superseded" && last)
            || (["completed", "active"].contains(&disposition) && !last)
        {
            return Err(located(format!(
                "disposition {disposition} is impossible here; only the last instance can be completed or active, and it cannot be superseded"
            )));
        }
        fingerprint_field(instance, "definition_fingerprint", "instance").map_err(&located)?;
        let template = instance.get("template").unwrap();
        if template != &Json::Null {
            exact(template, &["id", "revision"], "instance template").map_err(&located)?;
            nonempty(template, "id", "instance template").map_err(&located)?;
            if integer(template, "revision", "instance template").map_err(&located)? == 0 {
                return Err(located("template revision must be positive".into()));
            }
        }
        let attempts = items(instance, "attempts", "instance").map_err(&located)?;
        for (position, attempt) in attempts.iter().enumerate() {
            attempt_shape(attempt, position + 1)
                .map_err(|error| located(format!("attempt {}: {error}", position + 1)))?;
            let attempt_id = attempt.get("id").and_then(Json::as_str).unwrap();
            if !attempt_ids.insert(attempt_id) {
                return Err(located(format!("duplicate attempt id `{attempt_id}`")));
            }
        }
    }
    Ok(())
}

pub fn ledger_fingerprint(value: &Json) -> String {
    crate::fingerprint::canonical_sha256(value)
}

/// One flattened dataset row per attempt, in ledger order.
pub fn dataset_rows(ledger: &Json) -> Result<Vec<Json>, String> {
    validate_ledger(ledger)?;
    let mut rows = Vec::new();
    for instance in ledger.get("instances").and_then(Json::as_array).unwrap() {
        for attempt in instance.get("attempts").and_then(Json::as_array).unwrap() {
            let agent = attempt.get("agent").unwrap();
            let part = |field: &str, key: &str| agent.get(field).unwrap().get(key).unwrap().clone();
            let mut row = vec![
                ("project", ledger.get("project").unwrap().clone()),
                ("change", ledger.get("change").unwrap().clone()),
                ("instance", instance.get("id").unwrap().clone()),
                ("disposition", instance.get("disposition").unwrap().clone()),
            ];
            for key in ATTEMPT_FIELDS {
                if *key == "agent" {
                    row.extend([
                        ("agent_kind", part("kind", "value")),
                        ("agent_kind_basis", part("kind", "basis")),
                        ("model", part("model", "value")),
                        ("model_basis", part("model", "basis")),
                        ("effort", part("effort", "value")),
                        ("effort_basis", part("effort", "basis")),
                        ("agent_type", part("agent_type", "value")),
                        ("agent_type_basis", part("agent_type", "basis")),
                        ("harness_name", part("harness", "name")),
                        ("harness_version", part("harness", "version")),
                        ("harness_basis", part("harness", "basis")),
                    ]);
                } else if *key == "usage" {
                    row.extend(usage_columns(attempt.get("usage").unwrap()));
                } else {
                    row.push((key, attempt.get(key).unwrap().clone()));
                }
            }
            rows.push(Json::obj(row));
        }
    }
    Ok(rows)
}

/// The dataset's flat usage columns; every column is null when usage is unknown.
pub fn usage_columns(usage: &Json) -> Vec<(&'static str, Json)> {
    let own = |key: &str| usage.get(key).cloned().unwrap_or(Json::Null);
    let delegated = |key: &str| {
        usage
            .get("delegated")
            .and_then(|delegated| delegated.get(key))
            .cloned()
            .unwrap_or(Json::Null)
    };
    vec![
        ("usage_basis", own("basis")),
        ("model_calls", own("model_calls")),
        ("tool_calls", own("tool_calls")),
        ("input_tokens", own("input_tokens")),
        ("cache_read_input_tokens", own("cache_read_input_tokens")),
        ("cache_write_input_tokens", own("cache_write_input_tokens")),
        ("output_tokens", own("output_tokens")),
        ("reasoning_output_tokens", own("reasoning_output_tokens")),
        ("usage_overlapping", own("overlapping")),
        ("delegated_agents", delegated("agents")),
        ("delegated_model_calls", delegated("model_calls")),
        ("delegated_input_tokens", delegated("input_tokens")),
        (
            "delegated_cache_read_input_tokens",
            delegated("cache_read_input_tokens"),
        ),
        (
            "delegated_cache_write_input_tokens",
            delegated("cache_write_input_tokens"),
        ),
        ("delegated_output_tokens", delegated("output_tokens")),
        (
            "delegated_reasoning_output_tokens",
            delegated("reasoning_output_tokens"),
        ),
    ]
}

/// JSONL dataset over every committed ledger of active and archived changes below `root`.
pub fn dataset(root: &Path, change: Option<&str>) -> Result<String, String> {
    let changes = root.join("azimuth").join("changes");
    if !changes.is_dir() {
        return Err(format!("{} is not a directory", changes.display()));
    }
    let mut ledgers = Vec::new();
    for directory in [changes.clone(), changes.join("archive")] {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", directory.display()))?;
            if directory == changes && entry.file_name() == "archive" {
                continue;
            }
            let path = default_ledger(&entry.path());
            if path.is_file() {
                ledgers.push(path);
            }
        }
    }
    let mut selected = Vec::new();
    for path in ledgers {
        let ledger = read_json(&path)?;
        let rows = dataset_rows(&ledger).map_err(|e| format!("{}: {e}", path.display()))?;
        let id = ledger
            .get("change")
            .and_then(Json::as_str)
            .unwrap()
            .to_string();
        if change.is_none_or(|wanted| wanted == id) {
            selected.push((id, path, rows));
        }
    }
    selected.sort_by(|left, right| (&left.0, &left.1).cmp(&(&right.0, &right.1)));
    let mut out = String::new();
    for (_, _, rows) in selected {
        for row in rows {
            out.push_str(&compact(&row));
            out.push('\n');
        }
    }
    Ok(out)
}

// Failure Attribution

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// `YYYY-MM-DDTHH:MM:SS[.fff]Z`, UTC only, to Unix milliseconds.
fn utc_ms(text: &str) -> Option<u64> {
    let text = text.strip_suffix('Z')?;
    let (main, fraction) = match text.split_once('.') {
        Some((main, fraction)) => (main, Some(fraction)),
        None => (text, None),
    };
    let bytes = main.as_bytes();
    if bytes.len() != 19
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let field = |range: std::ops::Range<usize>| -> Option<i64> {
        let part = &main[range];
        if part.bytes().all(|b| b.is_ascii_digit()) {
            part.parse().ok()
        } else {
            None
        }
    };
    let (year, month, day) = (field(0..4)?, field(5..7)?, field(8..10)?);
    let (hour, minute, second) = (field(11..13)?, field(14..16)?, field(17..19)?);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&month)
        || day < 1
        || day > month_days[(month - 1) as usize]
        || hour > 23
        || minute > 59
        || second > 59
        || year < 1970
    {
        return None;
    }
    let millis = match fraction {
        None => 0,
        Some(digits)
            if (1..=3).contains(&digits.len()) && digits.bytes().all(|b| b.is_ascii_digit()) =>
        {
            format!("{digits:0<3}").parse::<i64>().ok()?
        }
        Some(_) => return None,
    };
    let seconds = days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second;
    Some((seconds * 1_000 + millis) as u64)
}

fn attribution_id(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn code_span(text: &str) -> Option<&str> {
    text.strip_prefix('`')?
        .strip_suffix('`')
        .filter(|inner| !inner.is_empty() && !inner.contains('`'))
}

fn declared(value: Option<&str>) -> Json {
    match value {
        Some(value) => Json::obj(vec![
            ("value", Json::str(value)),
            ("basis", Json::str("declared")),
        ]),
        None => Json::obj(vec![("value", Json::Null), ("basis", Json::str("unknown"))]),
    }
}

pub fn attribution_fingerprint(record: &Json) -> String {
    let Json::Obj(fields) = record else {
        return crate::fingerprint::canonical_sha256(record);
    };
    crate::fingerprint::canonical_sha256(&Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "fingerprint")
            .cloned()
            .collect(),
    ))
}

struct Lines<'a> {
    file: &'a str,
    lines: Vec<&'a str>,
    position: usize,
}

impl<'a> Lines<'a> {
    fn current(&self) -> Option<&'a str> {
        self.lines.get(self.position).copied()
    }

    fn error(&self, line: usize, expected: &str) -> String {
        format!("{}:{}: expected {expected}", self.file, line + 1)
    }

    fn skip_blank(&mut self) {
        while self.current().is_some_and(|line| line.trim().is_empty()) {
            self.position += 1;
        }
    }

    fn field(&mut self, label: &str) -> Result<(usize, &'a str), String> {
        let line = self.position;
        let prefix = format!("- {label}:");
        let value = self
            .current()
            .and_then(|text| text.strip_prefix(prefix.as_str()))
            .ok_or_else(|| self.error(line, &format!("`- {label}: …`")))?;
        self.position += 1;
        Ok((line, value.trim()))
    }

    fn value(&mut self, label: &str) -> Result<(usize, &'a str), String> {
        let (line, value) = self.field(label)?;
        if value.is_empty() {
            return Err(self.error(line, &format!("a value after `- {label}:`")));
        }
        Ok((line, value))
    }

    fn optional(&mut self, label: &str) -> Result<Option<&'a str>, String> {
        let (_, value) = self.value(label)?;
        Ok((value != "none").then_some(value))
    }

    fn attempts(&mut self, label: &str) -> Result<Vec<(usize, &'a str)>, String> {
        let (line, value) = self.field(label)?;
        if value == "none" {
            return Ok(vec![]);
        }
        if !value.is_empty() {
            return Err(self.error(
                line,
                &format!("`- {label}:` followed by nested attempt items, or `- {label}: none`"),
            ));
        }
        let mut attempts = Vec::new();
        while let Some(item) = self.current().and_then(|text| text.strip_prefix("  - ")) {
            let attempt = code_span(item.trim())
                .filter(|attempt| attempt.starts_with("attempt-"))
                .ok_or_else(|| {
                    self.error(
                        self.position,
                        "an attempt id in backticks, such as `attempt-…`",
                    )
                })?;
            attempts.push((self.position, attempt));
            self.position += 1;
        }
        if attempts.is_empty() {
            return Err(self.error(
                self.position,
                &format!("at least one `  - `attempt-…`` item under `- {label}:`, or `none`"),
            ));
        }
        Ok(attempts)
    }
}

/// Compiles `azimuth/provenance/failure-attributions.md` into the attribution collection.
pub fn compile_attributions(file: &str, source: &str) -> Result<Json, String> {
    let mut lines = Lines {
        file,
        lines: source.lines().collect(),
        position: 0,
    };
    while lines
        .current()
        .is_some_and(|line| !line.starts_with("## ") && !line.starts_with("### "))
    {
        lines.position += 1;
    }
    let mut compiled: Vec<Json> = Vec::new();
    while let Some(heading_text) = lines.current() {
        let heading = lines.position;
        let id = heading_text
            .strip_prefix("## Failure attribution: ")
            .map(str::trim)
            .filter(|id| attribution_id(id))
            .ok_or_else(|| {
                lines.error(
                    heading,
                    "`## Failure attribution: <id>` with an id of letters, digits, `-` or `_`",
                )
            })?;
        if compiled
            .iter()
            .any(|record| record.get("id").and_then(Json::as_str) == Some(id))
        {
            return Err(format!(
                "{file}:{}: duplicate Failure Attribution id `{id}`",
                heading + 1
            ));
        }
        lines.position += 1;
        lines.skip_blank();
        let (line, requirement) = lines.value("Requirement")?;
        let requirement = code_span(requirement)
            .ok_or_else(|| lines.error(line, "a Requirement id in backticks"))?;
        let (line, adverse) = lines.value("Adverse fact")?;
        let (kind, reference) = adverse
            .split_once(' ')
            .and_then(|(kind, reference)| Some((kind, code_span(reference.trim())?)))
            .filter(|(kind, _)| ADVERSE_KINDS.contains(kind))
            .ok_or_else(|| {
                lines.error(
                    line,
                    &format!(
                        "`<kind> `<reference>`` with kind {}",
                        ADVERSE_KINDS.join(" | ")
                    ),
                )
            })?;
        let (_, investigator) = lines.value("Investigator")?;
        let (line, agent_kind) = lines.value("Agent kind")?;
        let agent_kind = match agent_kind {
            "human" | "model" => Some(agent_kind),
            "none" => None,
            _ => return Err(lines.error(line, "agent kind human | model | none")),
        };
        let model = lines.optional("Model")?;
        let effort = lines.optional("Effort")?;
        let agent_type = lines.optional("Agent type")?;
        let (line, harness) = lines.value("Harness")?;
        let harness = if harness == "none" {
            Json::obj(vec![
                ("name", Json::Null),
                ("version", Json::Null),
                ("basis", Json::str("unknown")),
            ])
        } else {
            let parts = harness.split_whitespace().collect::<Vec<_>>();
            if parts.len() > 2 {
                return Err(lines.error(line, "`<harness name> [<version>]` or `none`"));
            }
            Json::obj(vec![
                ("name", Json::str(parts[0])),
                (
                    "version",
                    parts.get(1).map(|v| Json::str(*v)).unwrap_or(Json::Null),
                ),
                ("basis", Json::str("declared")),
            ])
        };
        let (line, recorded) = lines.value("Recorded at")?;
        let recorded_at_ms = utc_ms(recorded)
            .ok_or_else(|| lines.error(line, "a UTC timestamp `YYYY-MM-DDTHH:MM:SSZ`"))?;
        let (line, confidence) = lines.value("Confidence")?;
        if !CONFIDENCES.contains(&confidence) {
            return Err(lines.error(line, "confidence low | medium | high"));
        }
        let caused = lines.attempts("Caused")?;
        let allowed = lines.attempts("Allowed through")?;
        if caused.is_empty() && allowed.is_empty() {
            return Err(format!(
                "{file}:{}: expected at least one `Caused` or `Allowed through` attempt",
                heading + 1
            ));
        }
        let mut attempts = Vec::new();
        let mut seen = BTreeSet::new();
        for (role, entries) in [("caused", &caused), ("allowed-through", &allowed)] {
            for (line, attempt) in entries {
                if !seen.insert(*attempt) {
                    return Err(format!(
                        "{file}:{}: attempt `{attempt}` is named more than once; each attempt has one role",
                        line + 1
                    ));
                }
                attempts.push(Json::obj(vec![
                    ("attempt", Json::str(*attempt)),
                    ("role", Json::str(role)),
                ]));
            }
        }
        let (line, revises) = lines.value("Revises")?;
        let revises = if revises == "none" {
            Json::Null
        } else {
            let earlier = code_span(revises)
                .filter(|earlier| {
                    compiled
                        .iter()
                        .any(|record| record.get("id").and_then(Json::as_str) == Some(*earlier))
                })
                .ok_or_else(|| {
                    lines.error(
                        line,
                        "`none` or the id of an earlier attribution in backticks",
                    )
                })?;
            Json::str(earlier)
        };
        lines.skip_blank();
        if lines.current().map(str::trim_end) != Some("### Rationale") {
            return Err(lines.error(lines.position, "`### Rationale`"));
        }
        let rationale_line = lines.position;
        lines.position += 1;
        let start = lines.position;
        while lines.current().is_some_and(|line| !line.starts_with("## ")) {
            lines.position += 1;
        }
        let rationale = lines.lines[start..lines.position]
            .join("\n")
            .trim()
            .to_string();
        if rationale.is_empty() {
            return Err(lines.error(
                rationale_line,
                "substantive rationale after `### Rationale`",
            ));
        }
        let mut record = Json::obj(vec![
            ("format", Json::str(ATTRIBUTION_FORMAT)),
            ("version", Json::Num(1.0)),
            ("id", Json::str(id)),
            ("requirement", Json::str(requirement)),
            (
                "adverse",
                Json::obj(vec![
                    ("kind", Json::str(kind)),
                    ("ref", Json::str(reference)),
                ]),
            ),
            ("investigator", Json::str(investigator)),
            (
                "agent",
                Json::obj(vec![
                    ("kind", declared(agent_kind)),
                    ("model", declared(model)),
                    ("effort", declared(effort)),
                    ("agent_type", declared(agent_type)),
                    ("harness", harness),
                ]),
            ),
            ("recorded_at_ms", Json::Num(recorded_at_ms as f64)),
            ("confidence", Json::str(confidence)),
            ("attempts", Json::Arr(attempts)),
            ("revises", revises),
            ("rationale", Json::str(rationale)),
        ]);
        let fingerprint = attribution_fingerprint(&record);
        if let Json::Obj(fields) = &mut record {
            fields.push(("fingerprint".into(), Json::str(fingerprint)));
        }
        validate_attribution(&record).map_err(|e| format!("{file}:{}: {e}", heading + 1))?;
        compiled.push(record);
    }
    Ok(Json::obj(vec![
        ("format", Json::str(ATTRIBUTIONS_FORMAT)),
        ("version", Json::Num(1.0)),
        ("attributions", Json::Arr(compiled)),
    ]))
}

/// Validates one compiled `azimuth-failure-attribution` record, including its fingerprint.
pub fn validate_attribution(value: &Json) -> Result<(), String> {
    let what = "attribution";
    exact(
        value,
        &[
            "format",
            "version",
            "id",
            "requirement",
            "adverse",
            "investigator",
            "agent",
            "recorded_at_ms",
            "confidence",
            "attempts",
            "revises",
            "rationale",
            "fingerprint",
        ],
        what,
    )?;
    if value.get("format").and_then(Json::as_str) != Some(ATTRIBUTION_FORMAT)
        || value.get("version").and_then(safe_integer) != Some(1)
    {
        return Err(format!("expected {ATTRIBUTION_FORMAT} version 1"));
    }
    let id = nonempty(value, "id", what)?;
    if !attribution_id(id) {
        return Err("attribution `id` allows only letters, digits, `-` and `_`".into());
    }
    nonempty(value, "requirement", what)?;
    let adverse = value.get("adverse").unwrap();
    exact(adverse, &["kind", "ref"], "attribution adverse fact")?;
    if !ADVERSE_KINDS.contains(&nonempty(adverse, "kind", "attribution adverse fact")?) {
        return Err(format!(
            "attribution adverse fact kind must be {}",
            ADVERSE_KINDS.join(", ")
        ));
    }
    nonempty(adverse, "ref", "attribution adverse fact")?;
    nonempty(value, "investigator", what)?;
    validate_agent(value.get("agent").unwrap())?;
    integer(value, "recorded_at_ms", what)?;
    if !CONFIDENCES.contains(&nonempty(value, "confidence", what)?) {
        return Err("attribution `confidence` must be low, medium or high".into());
    }
    let attempts = items(value, "attempts", what)?;
    if attempts.is_empty() {
        return Err("attribution requires at least one caused or allowed-through attempt".into());
    }
    let mut seen = BTreeSet::new();
    for attempt in attempts {
        exact(attempt, &["attempt", "role"], "attribution attempt")?;
        let named = nonempty(attempt, "attempt", "attribution attempt")?;
        let role = nonempty(attempt, "role", "attribution attempt")?;
        if !["caused", "allowed-through"].contains(&role) {
            return Err("attribution attempt role must be caused or allowed-through".into());
        }
        if !seen.insert(named) {
            return Err(format!(
                "attribution names attempt `{named}` more than once"
            ));
        }
    }
    if optional_text(value, "revises", what)? == Some(id) {
        return Err("an attribution cannot revise itself".into());
    }
    nonempty(value, "rationale", what)?;
    if nonempty(value, "fingerprint", what)? != attribution_fingerprint(value) {
        return Err("attribution `fingerprint` does not match its content".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_timestamps_convert_to_unix_milliseconds() {
        assert_eq!(utc_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(utc_ms("2026-10-24T08:00:00Z"), Some(1_792_828_800_000));
        assert_eq!(utc_ms("2024-02-29T23:59:59.5Z"), Some(1_709_251_199_500));
        assert_eq!(utc_ms("2025-02-29T00:00:00Z"), None);
        assert_eq!(utc_ms("2026-10-24 08:00:00Z"), None);
        assert_eq!(utc_ms("2026-10-24T08:00:00+01:00"), None);
    }

    fn descriptor(kind: Json) -> Json {
        Json::obj(vec![
            ("format", Json::str(DESCRIPTOR_FORMAT)),
            ("version", Json::Num(1.0)),
            ("kind", kind),
            ("model", declared(Some("claude-opus-5-5"))),
            ("effort", declared(None)),
            ("agent_type", declared(None)),
            (
                "harness",
                Json::obj(vec![
                    ("name", Json::str("claude-code")),
                    ("version", Json::Null),
                    ("basis", Json::str("harness")),
                ]),
            ),
            ("trace", Json::Null),
            ("usage", Json::Null),
        ])
    }

    #[test]
    fn descriptor_bases_govern_values() {
        assert!(validate_descriptor(&descriptor(declared(Some("model")))).is_ok());
        assert_eq!(
            validate_descriptor(&descriptor(declared(Some("robot")))).unwrap_err(),
            "agent `kind` value must be one of model, human"
        );
        let unknown_with_value = Json::obj(vec![
            ("value", Json::str("model")),
            ("basis", Json::str("unknown")),
        ]);
        assert_eq!(
            validate_descriptor(&descriptor(unknown_with_value)).unwrap_err(),
            "agent `kind` with basis unknown requires a null value"
        );
        let declared_null = Json::obj(vec![
            ("value", Json::Null),
            ("basis", Json::str("declared")),
        ]);
        assert_eq!(
            validate_descriptor(&descriptor(declared_null)).unwrap_err(),
            "agent `kind` with basis declared requires a nonempty string value"
        );
        let mut extra = descriptor(declared(Some("human")));
        if let Json::Obj(fields) = &mut extra {
            fields.push(("role".into(), Json::str("reviewer")));
        }
        assert_eq!(
            validate_descriptor(&extra).unwrap_err(),
            "agent descriptor has unknown field `role`"
        );
    }

    #[test]
    fn shipped_guidance_is_fingerprinted_and_packages_are_not() {
        let entry = |kind: &str, id: &str| {
            Json::obj(vec![("kind", Json::str(kind)), ("id", Json::str(id))])
        };
        let guidance = guidance_fingerprints(&[
            entry("skill", "azimuth-implement-change"),
            entry("reference", "workflow"),
            entry("package", "example"),
        ])
        .unwrap();
        let skill = crate::resources::SKILLS
            .iter()
            .find(|skill| skill.id == "azimuth-implement-change")
            .unwrap();
        let expected = format!(
            "sha256:{}",
            crate::fingerprint::sha256(skill.source.as_bytes())
        );
        assert_eq!(
            guidance[0].get("fingerprint").and_then(Json::as_str),
            Some(expected.as_str())
        );
        assert!(guidance[1]
            .get("fingerprint")
            .and_then(Json::as_str)
            .is_some_and(sha256_fingerprint));
        assert_eq!(guidance[2].get("fingerprint"), Some(&Json::Null));
    }

    #[test]
    fn scope_follows_the_most_specific_target_locator() {
        let names = |list: &[&'static str]| list.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(
            scope_of(&names(&["change", "module", "requirement"])),
            "requirement"
        );
        assert_eq!(scope_of(&names(&["check", "model_revision"])), "check");
        assert_eq!(
            scope_of(&names(&["verification_challenge"])),
            "verification-challenge"
        );
        assert_eq!(scope_of(&names(&["module", "project"])), "module");
        assert_eq!(scope_of(&names(&["change", "project"])), "change");
    }
}
