//! Explicit coordination state is separate from model and execution authority.
use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const KINDS: &[&str] = &[
    "establish-change-scope",
    "author-intent-delta",
    "author-design",
    "author-verification",
    "review-intent-delta",
    "review-design",
    "review-verification",
    "author-engineering-plan",
    "request-implementation-approval",
    "implement-change",
    "capture-source-support",
    "validate-candidate",
    "register-candidate-model-revision",
    "qualify-method",
    "execute-qualified-check",
    "assess-applicability",
    "judge-claim",
];
fn obj(value: &Json, names: &[&str]) -> Result<(), String> {
    let Json::Obj(fields) = value else {
        return Err("expected object".into());
    };
    let mut seen = BTreeSet::new();
    for (key, _) in fields {
        if !names.contains(&key.as_str()) {
            return Err(format!("unknown field {key}"));
        }
        if !seen.insert(key) {
            return Err(format!("duplicate field {key}"));
        }
    }
    for key in names {
        if value.get(key).is_none() {
            return Err(format!("missing field {key}"));
        }
    }
    Ok(())
}
fn text<'a>(v: &'a Json, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(Json::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("{key} requires nonempty string"))
}
fn arr<'a>(v: &'a Json, key: &str) -> Result<&'a [Json], String> {
    v.get(key)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{key} requires array"))
}
fn number(v: &Json, key: &str) -> Result<u64, String> {
    let n = v
        .get(key)
        .and_then(Json::as_num)
        .ok_or_else(|| format!("{key} requires integer"))?;
    if n < 0.0 || n.fract() != 0.0 || n > 9_007_199_254_740_991.0 {
        return Err(format!("{key} requires safe nonnegative integer"));
    }
    Ok(n as u64)
}
fn identifier(s: &str) -> Result<(), String> {
    if s.is_empty()
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_/".contains(&b))
    {
        return Err(format!("invalid identifier {s}"));
    }
    Ok(())
}
fn hash(bytes: &[u8]) -> String {
    format!("sha256:{}", crate::fingerprint::sha256(bytes))
}
fn digest(v: &Json) -> String {
    crate::fingerprint::canonical_sha256(v)
}
fn fingerprint(s: &str) -> Result<(), String> {
    if !s.strip_prefix("sha256:").is_some_and(|h| {
        h.len() == 64
            && h.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err("expected lowercase sha256 fingerprint".into());
    }
    Ok(())
}
fn duplicate_tree(v: &Json) -> Result<(), String> {
    match v {
        Json::Obj(fields) => {
            let mut seen = BTreeSet::new();
            for (k, v) in fields {
                if !seen.insert(k) {
                    return Err(format!("duplicate field {k}"));
                }
                duplicate_tree(v)?;
            }
        }
        Json::Arr(values) => {
            for v in values {
                duplicate_tree(v)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn read(path: &Path) -> Result<Json, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v = crate::json::parse(&source).map_err(|e| format!("{}: {e}", path.display()))?;
    duplicate_tree(&v).map_err(|e| format!("{}: line 1: {e}", path.display()))?;
    Ok(v)
}
fn portable(s: &str) -> Result<(), String> {
    if s.contains('\\')
        || s.contains('\0')
        || s.contains("//")
        || s.ends_with('/')
        || Path::new(s)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("non-normal portable reference {s}"));
    }
    Ok(())
}
fn ref_shape(v: &Json, hashed: bool) -> Result<(), String> {
    obj(
        v,
        if hashed {
            &["kind", "repository", "ref", "fingerprint"]
        } else {
            &["kind", "repository", "ref"]
        },
    )?;
    identifier(text(v, "kind")?)?;
    identifier(text(v, "repository")?)?;
    portable(text(v, "ref")?)?;
    if hashed {
        fingerprint(text(v, "fingerprint")?)?;
    }
    Ok(())
}
fn roots(v: &Json) -> Result<BTreeMap<String, PathBuf>, String> {
    let Json::Obj(fields) = v else {
        return Err("roots requires object".into());
    };
    let mut result = BTreeMap::new();
    for (id, path) in fields {
        identifier(id)?;
        let p = PathBuf::from(path.as_str().ok_or("root requires path")?);
        if !p.is_absolute() {
            return Err("root must be absolute".into());
        }
        let canonical = fs::canonicalize(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        if !canonical.is_dir() {
            return Err("root must be directory".into());
        }
        if canonical != p {
            return Err("root must be canonical absolute path".into());
        }
        result.insert(id.clone(), p);
    }
    Ok(result)
}
fn resolve(v: &Json, r: &BTreeMap<String, PathBuf>) -> Result<PathBuf, String> {
    portable(text(v, "ref")?)?;
    let root = r
        .get(text(v, "repository")?)
        .ok_or("unknown reference repository")?;
    let p = root.join(text(v, "ref")?);
    let actual = fs::canonicalize(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    if !actual.starts_with(root) || actual != p || !actual.is_file() {
        return Err("reference must be a regular file without symlink or root escape".into());
    }
    Ok(actual)
}
fn materialize(v: &Json, r: &BTreeMap<String, PathBuf>) -> Result<Json, String> {
    let bytes = fs::read(resolve(v, r)?).map_err(|e| e.to_string())?;
    Ok(Json::obj(vec![
        ("kind", Json::str(text(v, "kind")?)),
        ("repository", Json::str(text(v, "repository")?)),
        ("ref", Json::str(text(v, "ref")?)),
        ("fingerprint", Json::str(hash(&bytes))),
    ]))
}
fn check_ref(v: &Json, r: &BTreeMap<String, PathBuf>) -> Result<(), String> {
    ref_shape(v, true)?;
    if materialize(v, r)?.get("fingerprint") != v.get("fingerprint") {
        return Err(format!("record changed: {}", text(v, "ref")?));
    }
    Ok(())
}
fn definition(v: &Json) -> Result<(), String> {
    obj(v, &["format", "version", "id", "tasks"])?;
    if text(v, "format")? != "azimuth-workflow-definition" || number(v, "version")? != 1 {
        return Err("expected workflow definition version 1".into());
    }
    identifier(text(v, "id")?)?;
    let tasks = arr(v, "tasks")?;
    if tasks.is_empty() {
        return Err("definition requires finite nonempty task set".into());
    }
    let mut seen = BTreeSet::new();
    for task in tasks {
        obj(
            task,
            &[
                "id",
                "kind",
                "target",
                "depends_on",
                "inputs",
                "guidance",
                "required_output",
                "correction",
            ],
        )?;
        identifier(text(task, "id")?)?;
        if !seen.insert(text(task, "id")?) {
            return Err("duplicate task ID".into());
        }
        if !KINDS.contains(&text(task, "kind")?) {
            return Err("unregistered workflow task kind".into());
        }
        let Json::Obj(target) = task.get("target").unwrap() else {
            return Err("target requires object".into());
        };
        for (k, v) in target {
            if ![
                "change", "module", "claim", "case", "check", "probe", "subject",
            ]
            .contains(&k.as_str())
            {
                return Err("unknown target locator".into());
            }
            if k == "subject" {
                fingerprint(v.as_str().ok_or("subject requires fingerprint")?)?;
            } else {
                identifier(v.as_str().ok_or("target requires identifiers")?)?;
            }
        }
        for required in match text(task, "kind")? {
            "qualify-method" | "execute-qualified-check" => vec!["check"],
            "assess-applicability" => vec!["check", "case", "subject"],
            "judge-claim" => vec!["claim", "subject"],
            _ => vec![],
        } {
            text(task.get("target").unwrap(), required)?;
        }
        let mut deps = BTreeSet::new();
        for dep in arr(task, "depends_on")? {
            let d = dep.as_str().ok_or("dependency requires task ID")?;
            if !deps.insert(d) {
                return Err("duplicate dependency".into());
            }
        }
        match task.get("correction").unwrap() {
            Json::Null => {}
            Json::Str(id) => {
                if !deps.contains(id.as_str()) {
                    return Err("correction must identify a direct prerequisite task".into());
                }
            }
            _ => return Err("correction requires null or direct dependency ID".into()),
        }
        let mut inputs = BTreeSet::new();
        for input in arr(task, "inputs")? {
            ref_shape(input, false)?;
            if !inputs.insert((
                text(input, "kind")?,
                text(input, "repository")?,
                text(input, "ref")?,
            )) {
                return Err("duplicate task input".into());
            }
        }
        for guidance in arr(task, "guidance")? {
            obj(guidance, &["kind", "id"])?;
            if !["skill", "reference", "package"].contains(&text(guidance, "kind")?) {
                return Err("invalid guidance kind".into());
            }
            identifier(text(guidance, "id")?)?;
        }
        obj(
            task.get("required_output").unwrap(),
            &["kind", "description"],
        )?;
        identifier(text(task.get("required_output").unwrap(), "kind")?)?;
        text(task.get("required_output").unwrap(), "description")?;
    }
    fn visit(
        id: &str,
        tasks: &[Json],
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        if done.contains(id) {
            return Ok(());
        }
        if !active.insert(id.into()) {
            return Err("workflow dependency cycle".into());
        }
        let task = tasks
            .iter()
            .find(|t| t.get("id").and_then(Json::as_str) == Some(id))
            .ok_or("unknown task dependency")?;
        for dep in arr(task, "depends_on")? {
            visit(dep.as_str().unwrap(), tasks, active, done)?;
        }
        active.remove(id);
        done.insert(id.into());
        Ok(())
    }
    let mut done = BTreeSet::new();
    for task in tasks {
        visit(text(task, "id")?, tasks, &mut BTreeSet::new(), &mut done)?;
    }
    Ok(())
}
fn instance(v: &Json) -> Result<(), String> {
    obj(
        v,
        &[
            "format",
            "version",
            "id",
            "revision",
            "definition",
            "definition_fingerprint",
            "roots",
            "history",
        ],
    )?;
    if text(v, "format")? != "azimuth-workflow-instance" || number(v, "version")? != 1 {
        return Err("expected workflow instance version 1".into());
    }
    identifier(text(v, "id")?)?;
    number(v, "revision")?;
    definition(v.get("definition").unwrap())?;
    if text(v, "definition_fingerprint")? != digest(v.get("definition").unwrap()) {
        return Err("definition fingerprint mismatch".into());
    }
    roots(v.get("roots").unwrap())?;
    let history = arr(v, "history")?;
    if history.len() as u64 != number(v, "revision")? {
        return Err("revision/history mismatch".into());
    }
    let mut ids = BTreeSet::new();
    for item in history {
        obj(item, &["result", "result_fingerprint"])?;
        let result = item.get("result").unwrap();
        result_shape(result)?;
        if text(item, "result_fingerprint")? != digest(result) {
            return Err("history result fingerprint mismatch".into());
        }
        if !ids.insert(text(result, "id")?) {
            return Err("duplicate historical result ID".into());
        }
    }
    Ok(())
}
fn result_shape(v: &Json) -> Result<(), String> {
    obj(
        v,
        &[
            "format", "version", "id", "task", "basis", "actor", "outcome", "records",
        ],
    )?;
    if text(v, "format")? != "azimuth-workflow-result" || number(v, "version")? != 1 {
        return Err("expected workflow result version 1".into());
    }
    for key in ["id", "task", "actor"] {
        identifier(text(v, key)?)?;
    }
    fingerprint(text(v, "basis")?)?;
    if ![
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
    ]
    .contains(&text(v, "outcome")?)
    {
        return Err("invalid task outcome".into());
    }
    if arr(v, "records")?.is_empty() {
        return Err("result requires supporting records".into());
    }
    let mut records = BTreeSet::new();
    for record in arr(v, "records")? {
        ref_shape(record, true)?;
        if !records.insert((
            text(record, "kind")?,
            text(record, "repository")?,
            text(record, "ref")?,
        )) {
            return Err("duplicate result record".into());
        }
    }
    Ok(())
}
fn latest<'a>(state: &'a Json, id: &str) -> Option<&'a Json> {
    state
        .get("history")?
        .as_array()?
        .iter()
        .rev()
        .map(|h| h.get("result").unwrap())
        .find(|r| r.get("task").and_then(Json::as_str) == Some(id))
}
fn successful(result: &Json) -> bool {
    matches!(
        result.get("outcome").and_then(Json::as_str),
        Some("completed" | "accepted" | "supported" | "approved")
    )
}
fn review_kind(kind: &str) -> bool {
    kind.starts_with("review-")
}
fn author_for(kind: &str) -> Option<&'static str> {
    match kind {
        "review-intent-delta" => Some("author-intent-delta"),
        "review-design" => Some("author-design"),
        "review-verification" => Some("author-verification"),
        _ => None,
    }
}
fn basis(task: &Json, inputs: &[Json], deps: &[Json], correction: &[Json]) -> String {
    digest(&Json::obj(vec![
        ("task", task.clone()),
        ("inputs", Json::Arr(inputs.to_vec())),
        ("dependencies", Json::Arr(deps.to_vec())),
        ("corrections", Json::Arr(correction.to_vec())),
    ]))
}
struct Evaluation {
    eligible: Vec<Json>,
    blocked: Vec<Json>,
    current: BTreeMap<String, bool>,
}
fn evaluate(state: &Json, r: &BTreeMap<String, PathBuf>) -> Result<Evaluation, String> {
    let tasks = arr(state.get("definition").unwrap(), "tasks")?;
    let mut current: BTreeMap<String, bool> = BTreeMap::new();
    let mut eligible = Vec::new();
    let mut blocked = Vec::new();
    let mut pending: Vec<&Json> = tasks.iter().collect();
    while !pending.is_empty() {
        let mut progress = false;
        let mut rest = Vec::new();
        for task in pending {
            let id = text(task, "id")?;
            let deps = arr(task, "depends_on")?;
            if deps
                .iter()
                .any(|d| !current.contains_key(d.as_str().unwrap()))
            {
                rest.push(task);
                continue;
            }
            progress = true;
            let mut input_values = Vec::new();
            let mut reasons = Vec::new();
            for input in arr(task, "inputs")? {
                match materialize(input, r) {
                    Ok(v) => input_values.push(v),
                    Err(e) => reasons.push(e),
                }
            }
            let mut dep_results = Vec::new();
            for dep in deps {
                let d = dep.as_str().unwrap();
                if !current[d] {
                    reasons.push(format!("dependency {d} is incomplete, adverse or stale"));
                }
                if let Some(result) = latest(state, d) {
                    dep_results.push(result.clone());
                }
            }
            let mut corrections = Vec::new();
            let mut correction_pending = false;
            for review in tasks {
                if review.get("correction").and_then(Json::as_str) == Some(id)
                    || (review.get("correction") == Some(&Json::Null)
                        && author_for(text(review, "kind")?) == Some(text(task, "kind")?)
                        && arr(review, "depends_on")?
                            .iter()
                            .any(|d| d.as_str() == Some(id)))
                {
                    if let Some(rejection) = arr(state, "history")?
                        .iter()
                        .rev()
                        .map(|h| h.get("result").unwrap())
                        .find(|result| {
                            result.get("task").and_then(Json::as_str)
                                == Some(text(review, "id").unwrap())
                                && matches!(
                                    result.get("outcome").and_then(Json::as_str),
                                    Some(
                                        "rejected"
                                            | "violated"
                                            | "unresolved"
                                            | "failed"
                                            | "denied"
                                            | "deferred"
                                    )
                                )
                        })
                    {
                        let own = latest(state, id);
                        let history = arr(state, "history")?;
                        let pos = |result: &Json| {
                            history
                                .iter()
                                .position(|h| h.get("result") == Some(result))
                                .unwrap()
                        };
                        if own.is_none_or(|o| pos(o) < pos(rejection)) {
                            correction_pending = true;
                        }
                        corrections.push(rejection.clone());
                    }
                }
            }
            let mut basis_inputs = input_values.clone();
            let mut basis_dependencies = dep_results.clone();
            if ["qualify-method", "assess-applicability", "judge-claim"]
                .contains(&text(task, "kind")?)
            {
                for input in &mut basis_inputs {
                    if input.get("kind").and_then(Json::as_str) == Some("assurance-snapshot") {
                        let snapshot = read(&resolve(input, r)?)?;
                        validate_snapshot(&snapshot)?;
                        let identity = if text(task, "kind")? == "qualify-method" {
                            let check = task
                                .get("target")
                                .and_then(|v| v.get("check"))
                                .ok_or("qualification target requires check")?;
                            arr(snapshot.get("authority").unwrap(), "checks")?
                                .iter()
                                .find(|v| v.get("id") == Some(check))
                                .ok_or("Check missing from snapshot")?
                                .clone()
                        } else {
                            Json::obj(vec![("project", snapshot.get("project").unwrap().clone())])
                        };
                        if let Json::Obj(fields) = input {
                            for (key, v) in fields {
                                if key == "fingerprint" {
                                    *v = Json::str(digest(&identity));
                                }
                            }
                        }
                    }
                }
            }

            if text(task, "kind")? == "qualify-method" {
                basis_inputs.retain(|v| {
                    !["review-history", "run-bundle"]
                        .contains(&v.get("kind").and_then(Json::as_str).unwrap_or(""))
                });
                for input in &mut basis_inputs {
                    if input.get("kind").and_then(Json::as_str) == Some("model-authority") {
                        let authority = read(&resolve(input, r)?)?;
                        crate::assurance_review::validate_authority(&authority)?;
                        let check = task
                            .get("target")
                            .and_then(|v| v.get("check"))
                            .ok_or("qualification target requires check")?;
                        let selected = arr(&authority, "checks")?
                            .iter()
                            .find(|v| v.get("id") == Some(check))
                            .ok_or("qualification Check absent from authority")?;
                        if let Json::Obj(fields) = input {
                            for (key, v) in fields {
                                if key == "fingerprint" {
                                    *v = Json::str(digest(selected));
                                }
                            }
                        }
                    }
                }
                basis_dependencies = deps
                    .iter()
                    .map(|dep| Json::obj(vec![("task", dep.clone())]))
                    .collect();
            }
            if let Some(previous) = latest(state, id) {
                if !successful(previous) {
                    let configured = task.get("correction").and_then(Json::as_str);
                    let correction = configured.or_else(|| {
                        author_for(text(task, "kind").unwrap()).and_then(|kind| {
                            tasks
                                .iter()
                                .find(|t| {
                                    text(t, "kind").ok() == Some(kind)
                                        && deps.iter().any(|d| d.as_str() == text(t, "id").ok())
                                })
                                .and_then(|t| t.get("id").and_then(Json::as_str))
                        })
                    });
                    if let Some(correction) = correction {
                        let history = arr(state, "history")?;
                        let pos = |v: &Json| {
                            history
                                .iter()
                                .position(|h| h.get("result") == Some(v))
                                .unwrap()
                        };
                        if latest(state, correction).is_none_or(|v| pos(v) < pos(previous)) {
                            reasons.push(format!(
                                "correction {correction} required before reassessment"
                            ));
                        }
                    }
                }
            }
            let b = basis(task, &basis_inputs, &basis_dependencies, &corrections);
            let valid = !correction_pending
                && latest(state, id).is_some_and(|result| {
                    successful(result)
                        && result.get("basis").and_then(Json::as_str) == Some(&b)
                        && arr(result, "records").is_ok_and(|records| {
                            records.iter().all(|record| check_ref(record, r).is_ok())
                        })
                });
            let handoff = Json::obj(vec![
                ("inputs", Json::Arr(input_values.clone())),
                ("dependencies", Json::Arr(dep_results.clone())),
                ("target", task.get("target").unwrap().clone()),
            ]);
            let valid = valid
                && latest(state, id).is_none_or(|result| {
                    review_record(result, &handoff, text(task, "kind").unwrap(), r).is_ok()
                });
            current.insert(id.into(), valid && reasons.is_empty());
            if valid && reasons.is_empty() {
                continue;
            }
            if reasons.is_empty() {
                eligible.push(Json::obj(vec![("id",Json::str(id)),("task",Json::str(text(task,"kind")?)),("target",task.get("target").unwrap().clone()),("basis",Json::str(b)),("reason",Json::str(if !corrections.is_empty(){"Perform the configured correction after an adverse result; independent reassessment will follow."}else if latest(state,id).is_some(){"The previous result is adverse or its dependencies changed."}else{"The configured prerequisites are satisfied."})),("guidance",task.get("guidance").unwrap().clone()),("inputs",Json::Arr(input_values)),("dependencies",Json::Arr(dep_results)),("corrections",Json::Arr(corrections)),("required_output",task.get("required_output").unwrap().clone())]));
            } else {
                blocked.push(Json::obj(vec![
                    ("id", Json::str(id)),
                    ("task", Json::str(text(task, "kind")?)),
                    (
                        "reasons",
                        Json::Arr(reasons.into_iter().map(Json::str).collect()),
                    ),
                ]));
            }
        }
        if !progress {
            return Err("unresolved graph".into());
        }
        pending = rest;
    }
    Ok(Evaluation {
        eligible,
        blocked,
        current,
    })
}
fn record_of<'a>(records: &'a [Json], kind: &str) -> Result<&'a Json, String> {
    let matches: Vec<_> = records
        .iter()
        .filter(|r| r.get("kind").and_then(Json::as_str) == Some(kind))
        .collect();
    if matches.len() != 1 {
        return Err(format!("requires exactly one {kind} record"));
    }
    Ok(matches[0])
}
fn snapshot_runs(snapshot: &Json) -> Result<Vec<crate::run::RunBundle>, String> {
    let runs = arr(snapshot, "bundles")?
        .iter()
        .map(|v| {
            crate::run::parse("assurance-snapshot.bundles", &v.to_string_pretty())
                .map_err(|e| format!("invalid snapshot Run: {e:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let findings = runs.iter().flat_map(crate::run::verify).collect::<Vec<_>>();
    if !findings.is_empty() {
        return Err(format!("snapshot Run protocol findings: {findings:?}"));
    }
    Ok(runs)
}
fn validate_snapshot(v: &Json) -> Result<(), String> {
    obj(
        v,
        &[
            "project",
            "model_revision",
            "authority",
            "reviews",
            "bundles",
        ],
    )?;
    identifier(text(v, "project")?)?;
    fingerprint(text(v, "model_revision")?)?;
    crate::assurance_review::validate_authority(v.get("authority").unwrap())?;
    if v.get("model_revision") != v.get("authority").unwrap().get("fingerprint")
        || v.get("project") != v.get("authority").unwrap().get("project")
    {
        return Err("snapshot project/revision differs from authority".into());
    }
    for review in arr(v, "reviews")? {
        crate::assurance_review::validate_record(review)?;
    }
    snapshot_runs(v)?;
    Ok(())
}
fn review_record(
    result: &Json,
    handoff: &Json,
    kind: &str,
    r: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    let records = arr(result, "records")?;
    if kind == "request-implementation-approval" {
        let approval = read(&resolve(record_of(records, "implementation-approval")?, r)?)?;
        obj(
            &approval,
            &[
                "format",
                "version",
                "task",
                "actor",
                "outcome",
                "dependencies",
                "rationale",
            ],
        )?;
        if text(&approval, "format")? != "azimuth-workflow-approval"
            || number(&approval, "version")? != 1
        {
            return Err("expected workflow approval version 1".into());
        }
        for key in ["task", "actor", "outcome"] {
            if approval.get(key) != result.get(key) {
                return Err(format!("approval {key} mismatch"));
            }
        }
        text(&approval, "rationale")?;
        let mut expected = arr(handoff, "inputs")?.to_vec();
        for dependency in arr(handoff, "dependencies")? {
            expected.extend(arr(dependency, "records")?.iter().cloned());
        }
        if approval.get("dependencies") != Some(&Json::Arr(expected)) {
            return Err("approval dependencies do not match exact handoff".into());
        }
    } else if review_kind(kind) {
        let v = read(&resolve(record_of(records, "authoring-review")?, r)?)?;
        obj(
            &v,
            &[
                "format",
                "version",
                "task",
                "reviewer",
                "outcome",
                "dependencies",
                "findings",
                "rationale",
            ],
        )?;
        if text(&v, "format")? != "azimuth-authoring-review" || number(&v, "version")? != 1 {
            return Err("expected authoring review version 1".into());
        }
        if v.get("task") != result.get("task")
            || v.get("reviewer") != result.get("actor")
            || v.get("outcome") != result.get("outcome")
        {
            return Err("review actor, outcome or task mismatch".into());
        }
        text(&v, "rationale")?;
        for finding in arr(&v, "findings")? {
            if finding.as_str().is_none_or(str::is_empty) {
                return Err("finding requires nonempty prose".into());
            }
        }
        if text(&v, "outcome")? == "rejected" && arr(&v, "findings")?.is_empty() {
            return Err("rejected review requires findings".into());
        }
        let mut expected = arr(handoff, "inputs")?.to_vec();
        for dep in arr(handoff, "dependencies")? {
            for record in arr(dep, "records")? {
                expected.push(record.clone());
            }
            if dep.get("actor") == result.get("actor")
                && arr(dep, "records")?.iter().any(|record| {
                    record.get("kind").and_then(Json::as_str)
                        == Some(match kind {
                            "review-intent-delta" => "intent-delta",
                            "review-design" => "claim-design",
                            _ => "verification-account",
                        })
                })
            {
                return Err("reviewer must differ from prerequisite author actor; identity is a local assertion".into());
            }
        }
        if v.get("dependencies") != Some(&Json::Arr(expected)) {
            return Err(
                "review dependencies must equal exact handoff inputs then prerequisite records"
                    .into(),
            );
        }
    } else if ["qualify-method", "assess-applicability", "judge-claim"].contains(&kind) {
        let expected = match kind {
            "qualify-method" => "method-qualification",
            "assess-applicability" => "applicability-decision",
            _ => "claim-judgment",
        };
        let review = read(&resolve(record_of(records, expected)?, r)?)?;
        crate::assurance_review::validate_record(&review)?;
        for key in ["check", "case", "claim"] {
            if let Some(expected) = handoff.get("target").and_then(|t| t.get(key)) {
                if review.get(key) != Some(expected) {
                    return Err(format!("assurance review {key} differs from task target"));
                }
            }
        }
        if review.get("kind").and_then(Json::as_str) != Some(expected)
            || review.get("reviewer") != result.get("actor")
        {
            return Err("assurance review kind or reviewer mismatch".into());
        }
        if kind == "qualify-method" {
            for dependency in arr(handoff, "dependencies")? {
                if dependency.get("actor") == result.get("actor")
                    && arr(dependency, "records")?.iter().any(|record| {
                        ["implementation-result", "check-implementation"]
                            .contains(&record.get("kind").and_then(Json::as_str).unwrap_or(""))
                    })
                {
                    return Err("method reviewer must differ from prerequisite author actor (local assertion)".into());
                }
            }
        }
        let inputs = arr(handoff, "inputs")?;
        let snapshot = read(&resolve(record_of(inputs, "assurance-snapshot")?, r)?)?;
        validate_snapshot(&snapshot)?;
        let authority = snapshot.get("authority").unwrap();
        let prior = arr(&snapshot, "reviews")?;
        if !prior.iter().any(|submitted| submitted == &review) {
            return Err("review is absent from supplied ASS revision snapshot".into());
        }
        let target = crate::assurance_review::review_target(&review)?;
        if crate::assurance_review::latest_review(prior, &target)? != Some(&review) {
            return Err(
                "review has been superseded or snapshot current target is ambiguous".into(),
            );
        }
        let runs = snapshot_runs(&snapshot)?;
        if let Some(subject) = handoff.get("target").and_then(|v| v.get("subject")) {
            if review.get("subject_fingerprint") != Some(subject) {
                return Err("review Subject differs from task target".into());
            }
        }
        let assessment = crate::assurance_review::assess_record(&review, authority, prior, &runs);
        let desired = text(result, "outcome")?;
        let matches = assessment.status == desired
            || (kind == "judge-claim"
                && assessment.status == "accepted"
                && review.get("conclusion").and_then(Json::as_str) == Some(desired));
        if !matches {
            return Err(format!(
                "review does not establish submitted outcome: {} {:?}",
                assessment.status, assessment.diagnostics
            ));
        }
    }
    Ok(())
}
struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn lock(path: &Path) -> Result<Lock, String> {
    let p = path.with_extension("json.lock");
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&p)
        .map_err(|e| {
            format!(
                "cannot acquire {}: {e}; stale locks require explicit operator inspection",
                p.display()
            )
        })?
        .sync_all()
        .map_err(|e| e.to_string())?;
    Ok(Lock(p))
}
fn write_atomic(path: &Path, value: &Json) -> Result<(), String> {
    let temp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    let action = (|| {
        f.write_all(value.to_string_pretty().as_bytes())
            .map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&temp, path).map_err(|e| e.to_string())?;
        fs::File::open(path.parent().unwrap_or(Path::new(".")))
            .and_then(|d| d.sync_all())
            .map_err(|e| e.to_string())
    })(); // Durable state precedes the receipt, so retries recover a lost output response.
    if action.is_err() {
        let _ = fs::remove_file(temp);
    }
    action
}
pub fn command(args: &[String]) -> Result<Json, String> {
    let operation = args
        .first()
        .ok_or("workflow requires init, next, update or verify")?;
    let allowed = match operation.as_str() {
        "init" => vec!["--definition", "--instance", "--change", "--root"],
        "next" | "verify" => vec!["--instance", "--change"],
        "update" => vec!["--instance", "--change", "--result", "--expected-revision"],
        _ => return Err("unknown workflow operation".into()),
    };
    let mut flags: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut i = 1;
    while i < args.len() {
        let flag = &args[i];
        if !allowed.contains(&flag.as_str()) {
            return Err(format!("unknown workflow flag {flag}"));
        }
        let value = args.get(i + 1).ok_or("flag requires value")?;
        if value.starts_with("--") {
            return Err("flag requires value".into());
        }
        if flag != "--root" && flags.contains_key(flag) {
            return Err(format!("duplicate flag {flag}"));
        }
        flags.entry(flag.clone()).or_default().push(value.clone());
        i += 2;
    }
    let get = |key: &str| flags.get(key).and_then(|v| v.first()).map(String::as_str);
    if get("--instance").is_some() && get("--change").is_some() {
        return Err("use either --instance or --change directory".into());
    }
    let path = if let Some(p) = get("--instance") {
        PathBuf::from(p)
    } else if let Some(p) = get("--change") {
        Path::new(p).join("workflow.json")
    } else {
        return Err("explicit --instance or --change directory required".into());
    };
    if operation == "init" {
        let definition_path = Path::new(get("--definition").ok_or("init requires --definition")?);
        let def = read(definition_path)?;
        definition(&def).map_err(|e| format!("{}: line 1: {e}", definition_path.display()))?;
        let mut configured = Vec::new();
        let mut seen = BTreeSet::new();
        for pair in flags
            .get("--root")
            .ok_or("init requires explicit --root id=path")?
        {
            let (id, p) = pair.split_once('=').ok_or("root requires id=path")?;
            if !seen.insert(id) {
                return Err("duplicate root ID".into());
            }
            configured.push((id.to_string(), Json::str(p)));
        }
        let root_json = Json::Obj(configured);
        let r = roots(&root_json)?;
        for task in arr(&def, "tasks")? {
            for input in arr(task, "inputs")? {
                if !r.contains_key(text(input, "repository")?) {
                    return Err("unknown configured input root".into());
                }
            }
        }
        fs::create_dir_all(path.parent().unwrap_or(Path::new("."))).map_err(|e| e.to_string())?;
        let _lock = lock(&path)?;
        if path.exists() {
            return Err("workflow instance already exists".into());
        }
        let state = Json::obj(vec![
            ("format", Json::str("azimuth-workflow-instance")),
            ("version", Json::Num(1.0)),
            ("id", Json::str(text(&def, "id")?)),
            ("revision", Json::Num(0.0)),
            ("definition_fingerprint", Json::str(digest(&def))),
            ("definition", def),
            ("roots", root_json),
            ("history", Json::Arr(vec![])),
        ]);
        write_atomic(&path, &state)?;
        return Ok(state);
    }
    let _lock = if operation == "update" {
        Some(lock(&path)?)
    } else {
        None
    };
    let mut state = read(&path)?;
    instance(&state).map_err(|e| format!("{}: line 1: {e}", path.display()))?;
    let r = roots(state.get("roots").unwrap())?;
    if operation == "update" {
        let result_path = Path::new(get("--result").ok_or("update requires --result")?);
        let result = read(result_path)?;
        result_shape(&result).map_err(|e| format!("{}: line 1: {e}", result_path.display()))?;
        let expected = get("--expected-revision")
            .ok_or("update requires --expected-revision")?
            .parse::<u64>()
            .map_err(|_| "invalid expected revision")?;
        for (position, old) in arr(&state, "history")?.iter().enumerate() {
            if old.get("result").unwrap().get("id") == result.get("id") {
                if old.get("result") != Some(&result) {
                    return Err("result ID reused with different content".into());
                }
                return Ok(Json::obj(vec![
                    ("revision", state.get("revision").unwrap().clone()),
                    ("result", result),
                    ("idempotent", Json::Bool(true)),
                    ("applied_revision", Json::Num((position + 1) as f64)),
                ]));
            }
        }
        if expected != number(&state, "revision")? {
            return Err("workflow revision conflict".into());
        }
        let evaluation = evaluate(&state, &r)?;
        let handoff = evaluation
            .eligible
            .iter()
            .find(|t| t.get("id") == result.get("task"))
            .ok_or("task is not eligible")?;
        if handoff.get("basis") != result.get("basis") {
            return Err("stale result basis".into());
        }
        for record in arr(&result, "records")? {
            check_ref(record, &r)?;
        }
        let kind = text(handoff, "task")?;
        let outcome = text(&result, "outcome")?;
        let valid = if kind == "request-implementation-approval" {
            ["approved", "denied", "deferred"].contains(&outcome)
        } else if review_kind(kind) || ["qualify-method", "assess-applicability"].contains(&kind) {
            ["accepted", "rejected"].contains(&outcome)
        } else if kind == "judge-claim" {
            ["supported", "violated", "unresolved"].contains(&outcome)
        } else {
            ["completed", "failed"].contains(&outcome)
        };
        if !valid {
            return Err("outcome not permitted for task kind".into());
        }
        let required = if outcome == "failed" {
            "failure-report"
        } else {
            text(handoff.get("required_output").unwrap(), "kind")?
        };
        record_of(arr(&result, "records")?, required)?;
        if outcome != "failed" {
            review_record(&result, handoff, kind, &r)
                .map_err(|e| format!("{}: line 1: {e}", result_path.display()))?;
        }
        if kind == "execute-qualified-check" && outcome == "completed" {
            let reference = record_of(arr(&result, "records")?, "run-bundle")?;
            let path = resolve(reference, &r)?;
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let bundle = crate::run::parse(&path.display().to_string(), &source)
                .map_err(|e| format!("invalid Run: {e:?}"))?;
            if let Some(check) = handoff
                .get("target")
                .and_then(|v| v.get("check"))
                .and_then(Json::as_str)
            {
                if !bundle
                    .check_executions
                    .iter()
                    .any(|execution| execution.check.id == check)
                {
                    return Err("Run has no execution for configured Check".into());
                }
            }
            if let Some(subject) = handoff
                .get("target")
                .and_then(|v| v.get("subject"))
                .and_then(Json::as_str)
            {
                if bundle.subject_fingerprint != subject {
                    return Err("Run Subject differs from task target".into());
                }
            }
            let findings = crate::run::verify(&bundle);
            if !findings.is_empty() {
                return Err(format!("Run protocol findings: {findings:?}"));
            }
        }
        let before = evaluate(&state, &r)?;
        if before
            .eligible
            .iter()
            .find(|t| t.get("id") == result.get("task"))
            .and_then(|t| t.get("basis"))
            != result.get("basis")
        {
            return Err("dependencies changed during update".into());
        }
        for record in arr(&result, "records")? {
            check_ref(record, &r)?;
        }
        let additions = arr(&result, "records")?
            .iter()
            .filter(|record| {
                record.get("kind").and_then(Json::as_str) == Some("workflow-definition")
            })
            .collect::<Vec<_>>();
        if additions.len() > 1 {
            return Err("one workflow definition extension allowed per result".into());
        }
        let extension = if let Some(reference) = additions.first() {
            if outcome != "completed" {
                return Err("only completed work may extend workflow definition".into());
            }
            let extended = read(&resolve(reference, &r)?)?;
            definition(&extended).map_err(|e| {
                format!(
                    "{}: line 1: {e}",
                    resolve(reference, &r)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| "workflow-definition".into())
                )
            })?;
            let old = state.get("definition").unwrap();
            if extended.get("id") != old.get("id")
                || !arr(&extended, "tasks")?.starts_with(arr(old, "tasks")?)
                || arr(&extended, "tasks")?.len() <= arr(old, "tasks")?.len()
            {
                return Err("definition extension must append tasks preserving existing definitions and identity".into());
            }
            for task in arr(&extended, "tasks")? {
                for input in arr(task, "inputs")? {
                    if !r.contains_key(text(input, "repository")?) {
                        return Err("extended task input references unknown root".into());
                    }
                }
            }
            Some(extended)
        } else {
            None
        };
        if let Json::Obj(fields) = &mut state {
            for (k, v) in fields {
                if let Some(extended) = &extension {
                    if k == "definition" {
                        *v = extended.clone();
                    }
                    if k == "definition_fingerprint" {
                        *v = Json::str(digest(extended));
                    }
                }
                if k == "revision" {
                    *v = Json::Num((expected + 1) as f64);
                }
                if k == "history" {
                    if let Json::Arr(values) = v {
                        values.push(Json::obj(vec![
                            ("result_fingerprint", Json::str(digest(&result))),
                            ("result", result.clone()),
                        ]));
                    }
                }
            }
        }
        write_atomic(&path, &state)?;
        return Ok(Json::obj(vec![
            ("revision", state.get("revision").unwrap().clone()),
            ("result", result),
            ("idempotent", Json::Bool(false)),
            ("applied_revision", Json::Num((expected + 1) as f64)),
        ]));
    }
    let evaluation = evaluate(&state, &r)?;
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-workflow-handoff")),
        ("version", Json::Num(1.0)),
        ("workflow_revision", state.get("revision").unwrap().clone()),
        ("instance", Json::str(text(&state, "id")?)),
        (
            "definition_fingerprint",
            state.get("definition_fingerprint").unwrap().clone(),
        ),
        (
            "next",
            evaluation
                .eligible
                .first()
                .and_then(|v| v.get("id"))
                .cloned()
                .unwrap_or(Json::Null),
        ),
        ("eligible", Json::Arr(evaluation.eligible)),
        ("blocked", Json::Arr(evaluation.blocked)),
        (
            "current",
            Json::Obj(
                evaluation
                    .current
                    .into_iter()
                    .map(|(id, current)| (id, Json::Bool(current)))
                    .collect(),
            ),
        ),
    ]))
}

/// Query output cannot overwrite any configured input, submitted record or operational instance.
pub fn validate_output(args: &[String], output: &Path) -> Result<(), String> {
    let value = |flag: &str| {
        args.windows(2)
            .find(|p| p[0] == flag)
            .map(|p| PathBuf::from(&p[1]))
    };
    let path = value("--instance").or_else(|| value("--change").map(|p| p.join("workflow.json")));
    let Some(path) = path else {
        return Ok(());
    };
    let canonical = |p: &Path| -> Result<PathBuf, String> {
        let mut ancestor = p.to_path_buf();
        let mut suffix = Vec::new();
        while !ancestor.exists() {
            let name = ancestor
                .file_name()
                .ok_or("path has no existing ancestor")?
                .to_owned();
            suffix.push(name);
            ancestor = ancestor.parent().ok_or("path has no parent")?.to_path_buf();
            if ancestor.as_os_str().is_empty() {
                ancestor = PathBuf::from(".");
            }
        }
        let mut resolved = fs::canonicalize(ancestor).map_err(|e| e.to_string())?;
        for name in suffix.into_iter().rev() {
            resolved.push(name);
        }
        Ok(resolved)
    };
    let output = canonical(output)?;
    if canonical(&path)? == output {
        return Err("output aliases workflow instance".into());
    }
    if !path.exists() {
        if args.first().is_some_and(|v| v == "init") {
            if let Some(definition_path) = value("--definition") {
                let definition = read(&definition_path)?;
                let mut configured = Vec::new();
                for pair in args.windows(2).filter(|p| p[0] == "--root") {
                    let (id, root) = pair[1].split_once('=').ok_or("root requires id=path")?;
                    configured.push((id.to_string(), Json::str(root)));
                }
                let r = roots(&Json::Obj(configured))?;
                for task in arr(&definition, "tasks")? {
                    for reference in arr(task, "inputs")? {
                        let p = r
                            .get(text(reference, "repository")?)
                            .ok_or("unknown root")?
                            .join(text(reference, "ref")?);
                        if canonical(&p)? == output {
                            return Err("output aliases configured input".into());
                        }
                    }
                }
            }
        }
        return Ok(());
    }
    let state = read(&path)?;
    instance(&state).map_err(|e| format!("{}: line 1: {e}", path.display()))?;
    let r = roots(state.get("roots").unwrap())?;
    let mut refs = Vec::new();
    for task in arr(state.get("definition").unwrap(), "tasks")? {
        refs.extend(arr(task, "inputs")?.iter());
    }
    for history in arr(&state, "history")? {
        refs.extend(arr(history.get("result").unwrap(), "records")?.iter());
    }
    for reference in refs {
        let p = r
            .get(text(reference, "repository")?)
            .ok_or("unknown input root")?
            .join(text(reference, "ref")?);
        if canonical(&p)? == output {
            return Err("output aliases declared input or supporting record".into());
        }
    }
    Ok(())
}
