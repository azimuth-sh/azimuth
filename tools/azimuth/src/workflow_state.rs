//! Explicit coordination state is separate from model and execution authority.
use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const KINDS: &[&str] = &[
    "establish-change-scope",
    "author-proposal",
    "review-proposal",
    "inventory-assurance-targets",
    "produce-artifact",
    "author-spec-delta",
    "author-design",
    "author-verification",
    "review-spec-delta",
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
fn work_key(value: &Json) -> Result<String, String> {
    obj(value, &["action", "target"])?;
    identifier(text(value, "action")?)?;
    let Json::Obj(fields) = value.get("target").unwrap() else {
        return Err("work reference target requires object".into());
    };
    for (key, value) in fields {
        if ![
            "project",
            "change",
            "module",
            "claim",
            "case",
            "check",
            "probe",
            "subject",
            "model_revision",
        ]
        .contains(&key.as_str())
        {
            return Err("unknown work target locator".into());
        }
        let value = value
            .as_str()
            .ok_or("work target requires string locators")?;
        if key == "subject" || key == "model_revision" {
            fingerprint(value)?;
        } else {
            identifier(value)?;
        }
    }
    Ok(format!(
        "work:{}",
        crate::fingerprint::canonical_json(value)
    ))
}
fn decode_work(value: Json) -> Result<Json, String> {
    match value {
        Json::Obj(fields)
            if fields.len() == 2
                && fields.iter().any(|(k, _)| k == "action")
                && fields.iter().any(|(k, _)| k == "target") =>
        {
            Ok(Json::str(work_key(&Json::Obj(fields))?))
        }
        Json::Obj(fields) => Ok(Json::Obj(
            fields
                .into_iter()
                .map(|(key, value)| Ok((key, decode_work(value)?)))
                .collect::<Result<_, String>>()?,
        )),
        Json::Arr(values) => Ok(Json::Arr(
            values
                .into_iter()
                .map(decode_work)
                .collect::<Result<_, _>>()?,
        )),
        Json::Str(value) if value.starts_with("work:") => {
            Err("work references require structured action and target, not encoded strings".into())
        }
        other => Ok(other),
    }
}
fn encode_work(value: &Json) -> Json {
    match value {
        Json::Str(value) if value.starts_with("work:") => {
            crate::json::parse(&value[5..]).expect("validated internal work reference")
        }
        Json::Obj(fields) => Json::Obj(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), encode_work(value)))
                .collect(),
        ),
        Json::Arr(values) => Json::Arr(values.iter().map(encode_work).collect()),
        other => other.clone(),
    }
}
fn require_work(value: &str) -> Result<(), String> {
    let encoded = value
        .strip_prefix("work:")
        .ok_or("work reference requires structured action and target")?;
    work_key(&crate::json::parse(encoded)?)?;
    Ok(())
}
fn identifier(s: &str) -> Result<(), String> {
    if s.starts_with("work:") {
        return require_work(s);
    }
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
    crate::fingerprint::canonical_sha256(&encode_work(v))
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
    decode_work(v).map_err(|e| format!("{}: line 1: {e}", path.display()))
}
fn portable(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.contains('\\')
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
fn reference_repository(v: &Json) -> &str {
    v.get("repository").and_then(Json::as_str).unwrap_or("")
}
fn ref_shape(v: &Json, hashed: bool) -> Result<(), String> {
    let mut fields = vec!["kind", "ref"];
    if v.get("repository").is_some() {
        fields.push("repository");
        identifier(text(v, "repository")?)?;
    }
    if hashed {
        fields.push("fingerprint");
    }
    obj(v, &fields)?;
    identifier(text(v, "kind")?)?;
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
fn repository_locations(r: &BTreeMap<String, PathBuf>) -> Json {
    Json::Obj(
        r.iter()
            .filter(|(id, _)| !id.is_empty())
            .map(|(id, path)| (id.clone(), Json::str(path.to_string_lossy())))
            .collect(),
    )
}
fn repository_file(args: &[String]) -> Result<PathBuf, String> {
    let flag = |name: &str| {
        args.windows(2)
            .find(|p| p[0] == name)
            .map(|p| PathBuf::from(&p[1]))
    };
    if let Some(path) = flag("--repositories") {
        return Ok(path);
    }
    let project = flag("--project").ok_or(
        "workflow requires --repositories FILE or --project FILE for local repository locations",
    )?;
    let project = fs::canonicalize(&project).map_err(|e| format!("{}: {e}", project.display()))?;
    if !project.is_file() {
        return Err("--project requires a project file".into());
    }
    Ok(project.parent().unwrap().join("local-repositories.json"))
}
fn runtime_locations(
    args: &[String],
    instance_path: &Path,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let path = repository_file(args)?;
    let value = read(&path)?;
    let mut locations = (|| {
        obj(&value, &["repositories"])?;
        roots(value.get("repositories").unwrap())
    })()
    .map_err(|e: String| format!("{}: line 1: {e}", path.display()))?;
    let parent = instance_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let directory = fs::canonicalize(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    locations.insert(String::new(), directory);
    Ok(locations)
}
fn resolve(v: &Json, r: &BTreeMap<String, PathBuf>) -> Result<PathBuf, String> {
    portable(text(v, "ref")?)?;
    let root = r
        .get(reference_repository(v))
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
    let mut fields = vec![
        ("kind", Json::str(text(v, "kind")?)),
        ("ref", Json::str(text(v, "ref")?)),
        ("fingerprint", Json::str(hash(&bytes))),
    ];
    if let Some(repository) = v.get("repository") {
        fields.push(("repository", repository.clone()));
    }
    Ok(Json::obj(fields))
}
fn check_ref(v: &Json, r: &BTreeMap<String, PathBuf>) -> Result<(), String> {
    ref_shape(v, true)?;
    if materialize(v, r)?.get("fingerprint") != v.get("fingerprint") {
        return Err(format!("record changed: {}", text(v, "ref")?));
    }
    Ok(())
}
fn definition(v: &Json) -> Result<(), String> {
    let mut fields = vec!["format", "version", "id", "tasks"];
    if v.get("template").is_some() || v.get("parameters").is_some() {
        fields.extend(["template", "parameters", "template_fingerprint"]);
        template(v.get("template").ok_or("missing template")?)?;
        if text(v, "template_fingerprint")? != digest(v.get("template").unwrap()) {
            return Err("template fingerprint mismatch".into());
        }
        if expand(
            v.get("template").unwrap(),
            v.get("parameters").ok_or("missing parameters")?,
        )? != *v
        {
            return Err("resolved definition differs from pinned template expansion".into());
        }
    }
    obj(v, &fields)?;
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
        let mut fields = vec![
            "id",
            "kind",
            "target",
            "depends_on",
            "inputs",
            "guidance",
            "required_output",
            "correction",
        ];
        if task.get("action").is_some() {
            fields.push("action");
            identifier(text(task, "action")?)?;
        }
        if task.get("outcomes").is_some() || task.get("satisfies").is_some() {
            fields.extend(["outcomes", "satisfies"]);
            outcome_contract(task)?;
        }
        obj(task, &fields)?;
        require_work(text(task, "id")?)?;
        let action = task
            .get("action")
            .and_then(Json::as_str)
            .unwrap_or(text(task, "kind")?);
        if text(task, "id")? != concrete_id(action, task.get("target").unwrap()) {
            return Err("work reference must match the task action and target".into());
        }
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
                "project",
                "change",
                "module",
                "claim",
                "case",
                "check",
                "probe",
                "subject",
                "model_revision",
            ]
            .contains(&k.as_str())
            {
                return Err("unknown target locator".into());
            }
            if k == "subject" || k == "model_revision" {
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
                reference_repository(input),
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
        output_contract(task)?;
    }
    for task in tasks {
        if let Some(author_kind) = author_for(text(task, "kind")?) {
            let authors = tasks
                .iter()
                .filter(|candidate| {
                    text(candidate, "kind").ok() == Some(author_kind)
                        && arr(task, "depends_on")
                            .unwrap()
                            .iter()
                            .any(|d| d == candidate.get("id").unwrap())
                })
                .collect::<Vec<_>>();
            if authors.len() != 1 {
                return Err("authoring review requires exactly one direct author task".into());
            }
            if task.get("target") != authors[0].get("target") {
                return Err("authoring review target must equal its author target".into());
            }
            if task.get("correction") != Some(&Json::Null)
                && task.get("correction") != authors[0].get("id")
            {
                return Err("authoring review correction must reopen its author".into());
            }
        }
        if ["qualify-method", "assess-applicability", "judge-claim"].contains(&text(task, "kind")?)
            && arr(task, "inputs")?
                .iter()
                .filter(|i| i.get("kind").and_then(Json::as_str) == Some("assurance-snapshot"))
                .count()
                != 1
        {
            return Err("assurance action requires exactly one assurance-snapshot input".into());
        }
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
    let history = arr(v, "history")?;
    if history.len() as u64 != number(v, "revision")? {
        return Err("revision/history mismatch".into());
    }
    let mut ids = BTreeSet::new();
    for item in history {
        obj(item, &["result", "result_fingerprint"])?;
        let result = item.get("result").unwrap();
        result_shape(result)?;
        let task = arr(v.get("definition").unwrap(), "tasks")?
            .iter()
            .find(|t| t.get("id") == result.get("task"))
            .ok_or("history contains unknown task")?;
        if !permitted_outcomes(text(task, "kind")?).contains(&text(result, "outcome")?)
            || task.get("outcomes").is_some_and(|o| {
                !o.as_array()
                    .unwrap()
                    .contains(result.get("outcome").unwrap())
            })
        {
            return Err("history contains outcome not permitted by task definition".into());
        }
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
    require_work(text(v, "task")?)?;
    for key in ["id", "actor"] {
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
            reference_repository(record),
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
        "review-proposal" => Some("author-proposal"),
        "review-spec-delta" => Some("author-spec-delta"),
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
}
fn evaluate(state: &Json, r: &BTreeMap<String, PathBuf>) -> Result<Evaluation, String> {
    let tasks = arr(state.get("definition").unwrap(), "tasks")?;
    let mut current: BTreeMap<String, bool> = BTreeMap::new();
    let mut eligible = Vec::new();
    let selected_revision = state
        .get("definition")
        .and_then(|d| d.get("parameters"))
        .and_then(|p| p.get("assurance"))
        .and_then(|i| i.get("model_revision"));
    let mut pending: Vec<&Json> = tasks.iter().collect();
    while !pending.is_empty() {
        let mut progress = false;
        let mut rest = Vec::new();
        for task in pending {
            let id = text(task, "id")?;
            if task
                .get("target")
                .and_then(|t| t.get("model_revision"))
                .is_some_and(|v| Some(v) != selected_revision)
                && selected_revision.is_some()
            {
                current.insert(id.into(), false);
                progress = true;
                continue;
            }
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
            let normalization = (|| -> Result<(), String> {
                if ["qualify-method", "assess-applicability", "judge-claim"]
                    .contains(&text(task, "kind")?)
                {
                    for input in &mut basis_inputs {
                        if input.get("kind").and_then(Json::as_str) == Some("assurance-snapshot") {
                            let snapshot = read(&resolve(input, r)?)?;
                            validate_snapshot(&snapshot)?;
                            snapshot_target(&snapshot, task.get("target").unwrap())?;
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
                                Json::obj(vec![(
                                    "project",
                                    snapshot.get("project").unwrap().clone(),
                                )])
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
                Ok(())
            })();
            if let Err(reason) = normalization {
                reasons.push(reason);
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
                        && task.get("satisfies").is_none_or(|allowed| {
                            allowed.as_array().is_some_and(|values| {
                                values.contains(result.get("outcome").unwrap())
                            })
                        })
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
                eligible.push(Json::obj(vec![("id",Json::str(id)),("task",Json::str(text(task,"kind")?)),("action",task.get("action").cloned().unwrap_or_else(|| Json::str(text(task,"kind").unwrap()))),("target",task.get("target").unwrap().clone()),("repositories",repository_locations(r)),("workflow_directory",Json::str(r.get("").unwrap().to_string_lossy())),("basis",Json::str(b)),("reason",Json::str(if !corrections.is_empty(){"Perform the configured correction after an adverse result; independent reassessment will follow."}else if latest(state,id).is_some(){"The previous result is adverse or its dependencies changed."}else{"The configured prerequisites are satisfied."})),("guidance",task.get("guidance").unwrap().clone()),("inputs",Json::Arr(input_values)),("dependencies",Json::Arr(dep_results)),("corrections",Json::Arr(corrections)),("required_output",task.get("required_output").unwrap().clone())]));
            }
        }
        if !progress {
            return Err("unresolved graph".into());
        }
        pending = rest;
    }
    Ok(Evaluation { eligible })
}
fn handoff_inputs(handoff: &Json) -> Result<Vec<Json>, String> {
    let mut references = arr(handoff, "inputs")?.to_vec();
    for dependency in arr(handoff, "dependencies")? {
        references.extend(arr(dependency, "records")?.iter().cloned());
    }
    let mut inputs = Vec::new();
    let mut seen = BTreeMap::new();
    for reference in references {
        let key = (
            text(&reference, "kind")?.to_owned(),
            reference_repository(&reference).to_owned(),
            text(&reference, "ref")?.to_owned(),
        );
        if let Some(previous) = seen.get(&key) {
            if previous != &reference {
                return Err("conflicting fingerprints for a handoff input reference".into());
            }
        } else {
            seen.insert(key, reference.clone());
            inputs.push(reference);
        }
    }
    Ok(inputs)
}
fn public_handoff(handoff: &Json) -> Result<Json, String> {
    let inputs = handoff_inputs(handoff)?;
    let Json::Obj(fields) = handoff else {
        return Err("handoff requires object".into());
    };
    Ok(Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "dependencies")
            .map(|(key, value)| {
                (
                    key.clone(),
                    if key == "inputs" {
                        Json::Arr(inputs.clone())
                    } else {
                        value.clone()
                    },
                )
            })
            .collect(),
    ))
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
        let expected = handoff_inputs(handoff)?;
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
        let expected = handoff_inputs(handoff)?;
        for dep in arr(handoff, "dependencies")? {
            if dep.get("actor") == result.get("actor")
                && arr(dep, "records")?.iter().any(|record| {
                    record.get("kind").and_then(Json::as_str)
                        == Some(match kind {
                            "review-proposal" => "proposal",
                            "review-spec-delta" => "intent-delta",
                            "review-design" => "claim-design",
                            _ => "verification-account",
                        })
                })
            {
                return Err("reviewer must differ from prerequisite author actor; identity is a local assertion".into());
            }
        }
        if v.get("dependencies") != Some(&Json::Arr(expected)) {
            return Err("review dependencies must equal exact published handoff inputs".into());
        }
    } else if kind == "inventory-assurance-targets" {
        let inventory = read(&resolve(
            record_of(records, "assurance-target-inventory")?,
            r,
        )?)?;
        inventory_shape(&inventory)?;
        let snapshot = read(&resolve(inventory.get("snapshot").unwrap(), r)?)?;
        validate_snapshot(&snapshot)?;
        snapshot_target(&snapshot, handoff.get("target").unwrap())?;
        if snapshot.get("model_revision") != inventory.get("model_revision")
            || snapshot.get("project") != inventory.get("project")
        {
            return Err("inventory snapshot project/revision changed".into());
        }
        let registration = arr(handoff, "dependencies")?
            .iter()
            .flat_map(|d| arr(d, "records").unwrap_or(&[]))
            .find(|r| r.get("kind").and_then(Json::as_str) == Some("candidate-registration"))
            .ok_or("inventory has no direct candidate registration")?;
        let receipt = read(&resolve(registration, r)?)?;
        if receipt.get("model_revision") != inventory.get("model_revision")
            || receipt.get("project") != inventory.get("project")
        {
            return Err("inventory differs from current candidate registration".into());
        }
    } else if ["qualify-method", "assess-applicability", "judge-claim"].contains(&kind) {
        let expected = match kind {
            "qualify-method" => "method-qualification",
            "assess-applicability" => "applicability-decision",
            _ => "claim-judgment",
        };
        let review = read(&resolve(record_of(records, expected)?, r)?)?;
        crate::assurance_review::validate_record(&review)?;
        for key in if kind == "qualify-method" {
            vec!["check"]
        } else if kind == "assess-applicability" {
            vec!["check", "case"]
        } else {
            vec!["claim"]
        } {
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
        snapshot_target(&snapshot, handoff.get("target").unwrap())?;
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
        f.write_all(encode_work(value).to_string_pretty().as_bytes())
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
    command_internal(args).map(|value| encode_work(&value))
}
fn command_internal(args: &[String]) -> Result<Json, String> {
    let operation = args
        .first()
        .ok_or("workflow requires init, next, update or verify")?;
    let allowed = match operation.as_str() {
        "init" => vec![
            "--definition",
            "--workflow",
            "--inputs",
            "--instance",
            "--change",
            "--repositories",
            "--project",
        ],
        "next" | "verify" => vec!["--instance", "--change", "--repositories", "--project"],
        "update" => vec![
            "--instance",
            "--change",
            "--result",
            "--expected-revision",
            "--repositories",
            "--project",
        ],
        "definition" => vec!["--workflow"],
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
        if flags.contains_key(flag) {
            return Err(format!("duplicate flag {flag}"));
        }
        flags.entry(flag.clone()).or_default().push(value.clone());
        i += 2;
    }
    let get = |key: &str| flags.get(key).and_then(|v| v.first()).map(String::as_str);
    if operation == "definition" {
        let selected =
            shipped_template(get("--workflow").ok_or("definition requires --workflow")?)?;
        template(&selected)?;
        return Ok(selected);
    }
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
        let def = initial_definition(get("--definition"), get("--workflow"), get("--inputs"))?;
        definition(&def)?;
        fs::create_dir_all(
            path.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new(".")),
        )
        .map_err(|e| e.to_string())?;
        let r = runtime_locations(args, &path)?;
        for task in arr(&def, "tasks")? {
            for input in arr(task, "inputs")? {
                if !r.contains_key(reference_repository(input)) {
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
    let r = runtime_locations(args, &path)?;
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
        let configured = arr(state.get("definition").unwrap(), "tasks")?
            .iter()
            .find(|t| t.get("id") == result.get("task"))
            .unwrap();
        if !valid
            || configured.get("outcomes").is_some_and(|values| {
                !values
                    .as_array()
                    .unwrap()
                    .contains(result.get("outcome").unwrap())
            })
        {
            return Err("outcome not permitted by task definition".into());
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
        if kind == "establish-change-scope"
            && outcome == "completed"
            && state.get("definition").unwrap().get("parameters").is_some()
        {
            let scope = read(&resolve(
                record_of(arr(&result, "records")?, "change-scope")?,
                &r,
            )?)?;
            obj(
                &scope,
                &[
                    "format",
                    "version",
                    "project",
                    "change",
                    "modules",
                    "rationale",
                ],
            )?;
            if text(&scope, "format")? != "azimuth-change-scope" || number(&scope, "version")? != 1
            {
                return Err("expected change scope version 1".into());
            }
            let parameters = state.get("definition").unwrap().get("parameters").unwrap();
            for key in ["project", "change", "modules"] {
                if scope.get(key) != parameters.get(key) {
                    return Err(format!("scope {key} differs from declared workflow inputs"));
                }
            }
            text(&scope, "rationale")?;
        }
        if kind == "register-candidate-model-revision" && outcome == "completed" {
            let receipt = read(&resolve(
                record_of(arr(&result, "records")?, "candidate-registration")?,
                &r,
            )?)?;
            registration(&receipt, handoff.get("target").unwrap())?;
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
        let inventory_extension = if kind == "inventory-assurance-targets" && outcome == "completed"
        {
            let reference = record_of(arr(&result, "records")?, "assurance-target-inventory")?;
            let inventory = read(&resolve(reference, &r)?)?;
            validate_inventory(&inventory, &state, &r)?;
            let old = state.get("definition").unwrap();
            let mut parameters = old
                .get("parameters")
                .ok_or("inventory expansion requires pinned template")?
                .clone();
            let previous = parameters.get("assurance").unwrap().clone();
            if previous != Json::Null && previous != inventory {
                if previous.get("model_revision") == inventory.get("model_revision") {
                    return Err("same candidate revision requires unchanged target inventory and snapshot reference".into());
                }
                let mut retained = parameters
                    .get("previous_assurance")
                    .and_then(Json::as_array)
                    .unwrap_or(&[])
                    .to_vec();
                if retained
                    .iter()
                    .any(|v| v.get("model_revision") == inventory.get("model_revision"))
                {
                    return Err("cannot reactivate a superseded candidate revision".into());
                }
                retained.push(previous);
                set(&mut parameters, "previous_assurance", Json::Arr(retained));
            }
            set(&mut parameters, "assurance", inventory);
            Some(expand(old.get("template").unwrap(), &parameters)?)
        } else {
            None
        };
        if inventory_extension.is_some() && !additions.is_empty() {
            return Err("inventory expansion cannot also supply a concrete definition".into());
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
            if old.get("template").is_some() {
                return Err(
                    "pinned template workflows expand only through validated inventories".into(),
                );
            }
            if extended.get("id") != old.get("id")
                || !arr(&extended, "tasks")?.starts_with(arr(old, "tasks")?)
                || arr(&extended, "tasks")?.len() <= arr(old, "tasks")?.len()
            {
                return Err("definition extension must append tasks preserving existing definitions and identity".into());
            }
            for task in arr(&extended, "tasks")? {
                for input in arr(task, "inputs")? {
                    if !r.contains_key(reference_repository(input)) {
                        return Err("extended task input references unknown root".into());
                    }
                }
            }
            Some(extended)
        } else {
            inventory_extension
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
            "eligible",
            Json::Arr(
                evaluation
                    .eligible
                    .iter()
                    .map(public_handoff)
                    .collect::<Result<Vec<_>, _>>()?,
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
    if canonical(&repository_file(args)?)? == output {
        return Err("output aliases local repository locations".into());
    }
    if canonical(&path)? == output {
        return Err("output aliases workflow instance".into());
    }
    if !path.exists() {
        if args.first().is_some_and(|v| v == "init") {
            let raw = |flag: &str| {
                args.windows(2)
                    .find(|p| p[0] == flag)
                    .map(|p| p[1].as_str())
            };
            let definition =
                initial_definition(raw("--definition"), raw("--workflow"), raw("--inputs"))?;
            let r = runtime_locations(args, &path)?;
            for task in arr(&definition, "tasks")? {
                for reference in arr(task, "inputs")? {
                    let p = r
                        .get(reference_repository(reference))
                        .ok_or("unknown root")?
                        .join(text(reference, "ref")?);
                    if canonical(&p)? == output {
                        return Err("output aliases configured input".into());
                    }
                }
            }
        }
        return Ok(());
    }
    let state = read(&path)?;
    instance(&state).map_err(|e| format!("{}: line 1: {e}", path.display()))?;
    let r = runtime_locations(args, &path)?;
    let mut refs = Vec::new();
    for task in arr(state.get("definition").unwrap(), "tasks")? {
        refs.extend(arr(task, "inputs")?.iter());
    }
    if let Some(parameters) = state.get("definition").unwrap().get("parameters") {
        if let Some(inventory) = parameters.get("assurance").filter(|v| *v != &Json::Null) {
            refs.push(inventory.get("snapshot").unwrap());
        }
        if let Some(inventories) = parameters
            .get("previous_assurance")
            .and_then(Json::as_array)
        {
            for inventory in inventories {
                refs.push(inventory.get("snapshot").unwrap());
            }
        }
    }
    for history in arr(&state, "history")? {
        refs.extend(arr(history.get("result").unwrap(), "records")?.iter());
    }
    for reference in refs {
        let p = r
            .get(reference_repository(reference))
            .ok_or("unknown input root")?
            .join(text(reference, "ref")?);
        if canonical(&p)? == output {
            return Err("output aliases declared input or supporting record".into());
        }
    }
    Ok(())
}

fn set(value: &mut Json, key: &str, replacement: Json) {
    if let Json::Obj(fields) = value {
        if let Some((_, current)) = fields.iter_mut().find(|(k, _)| k == key) {
            *current = replacement;
        } else {
            fields.push((key.into(), replacement));
        }
    }
}
fn permitted_outcomes(kind: &str) -> &'static [&'static str] {
    match kind {
        "request-implementation-approval" => &["approved", "denied", "deferred"],
        "qualify-method"
        | "assess-applicability"
        | "review-proposal"
        | "review-spec-delta"
        | "review-design"
        | "review-verification" => &["accepted", "rejected"],
        "judge-claim" => &["supported", "violated", "unresolved"],
        _ => &["completed", "failed"],
    }
}
fn outcome_contract(action: &Json) -> Result<(), String> {
    let allowed = arr(action, "outcomes")?;
    let satisfies = arr(action, "satisfies")?;
    if allowed.is_empty() || satisfies.is_empty() {
        return Err("outcomes and satisfies must be nonempty".into());
    }
    let mut seen = BTreeSet::new();
    for value in allowed {
        let outcome = value.as_str().ok_or("outcome requires string")?;
        if !permitted_outcomes(text(action, "kind")?).contains(&outcome) || !seen.insert(outcome) {
            return Err("unknown, duplicate or semantically invalid action outcome".into());
        }
    }
    let mut seen = BTreeSet::new();
    for value in satisfies {
        if !allowed.contains(value)
            || !["completed", "accepted", "approved", "supported"]
                .contains(&value.as_str().unwrap_or(""))
            || !seen.insert(value.as_str())
        {
            return Err("satisfies requires unique permitted positive outcomes".into());
        }
    }
    Ok(())
}
fn template(v: &Json) -> Result<(), String> {
    obj(v, &["format", "version", "id", "revision", "actions"])?;
    if text(v, "format")? != "azimuth-workflow-template"
        || number(v, "version")? != 1
        || number(v, "revision")? == 0
    {
        return Err("expected workflow template version 1 with positive revision".into());
    }
    identifier(text(v, "id")?)?;
    let actions = arr(v, "actions")?;
    if actions.is_empty() {
        return Err("template requires actions".into());
    }
    let mut seen = BTreeSet::new();
    for action in actions {
        obj(
            action,
            &[
                "id",
                "kind",
                "collection",
                "depends_on",
                "inputs",
                "guidance",
                "required_output",
                "correction",
                "outcomes",
                "satisfies",
            ],
        )?;
        identifier(text(action, "id")?)?;
        if !seen.insert(text(action, "id")?) || !KINDS.contains(&text(action, "kind")?) {
            return Err("duplicate action ID or unknown action semantic kind".into());
        }
        if ![
            "change",
            "modules",
            "methods",
            "checks",
            "contributions",
            "claims",
        ]
        .contains(&text(action, "collection")?)
        {
            return Err("unknown target collection".into());
        }
        outcome_contract(action)?;
        obj(
            action.get("required_output").unwrap(),
            &["kind", "description"],
        )?;
        identifier(text(action.get("required_output").unwrap(), "kind")?)?;
        text(action.get("required_output").unwrap(), "description")?;
        output_contract(action)?;
        for guidance in arr(action, "guidance")? {
            obj(guidance, &["kind", "id"])?;
            if !["skill", "reference", "package"].contains(&text(guidance, "kind")?) {
                return Err("invalid guidance kind".into());
            }
            identifier(text(guidance, "id")?)?;
        }
        let mut input_bindings = BTreeSet::new();
        for input in arr(action, "inputs")? {
            if !input_bindings.insert(input.as_str().ok_or("input requires binding name")?) {
                return Err("duplicate input binding".into());
            }
            if !["request", "instructions", "snapshot"]
                .contains(&input.as_str().ok_or("input requires binding name")?)
            {
                return Err("unknown required input binding".into());
            }
        }
        if ["qualify-method", "assess-applicability", "judge-claim"]
            .contains(&text(action, "kind")?)
            && !input_bindings.contains("snapshot")
        {
            return Err("assurance action requires snapshot binding".into());
        }
        if let Some(author) = author_for(text(action, "kind")?) {
            let authors = arr(action, "depends_on")?
                .iter()
                .filter(|dep| {
                    actions.iter().any(|a| {
                        a.get("id") == dep.get("action") && text(a, "kind").ok() == Some(author)
                    })
                })
                .count();
            if authors != 1 {
                return Err("review action requires one direct author action".into());
            }
        }
        let mut deps = BTreeSet::new();
        for dep in arr(action, "depends_on")? {
            obj(dep, &["action", "match", "condition"])?;
            let id = text(dep, "action")?;
            if !deps.insert(id) || id == text(action, "id")? || !seen.contains(id) {
                return Err("dependency must name a distinct earlier action; cycles and forward/unreachable gates are invalid".into());
            }
            if text(dep, "condition")? != "current-success"
                || ![
                    "all",
                    "same-module",
                    "same-check",
                    "same-subject",
                    "claim-contributions",
                ]
                .contains(&text(dep, "match")?)
            {
                return Err("unknown bounded transition condition or target match".into());
            }
            let prerequisite = actions
                .iter()
                .find(|a| a.get("id") == dep.get("action"))
                .unwrap();
            let collection = text(action, "collection")?;
            let other = text(prerequisite, "collection")?;
            let meaningful = match text(dep, "match")? {
                "same-module" => collection == "modules" && other == "modules",
                "same-check" => {
                    ["methods", "checks", "contributions"].contains(&collection)
                        && ["methods", "checks", "contributions"].contains(&other)
                }
                "same-subject" => {
                    ["checks", "contributions", "claims"].contains(&collection)
                        && ["checks", "contributions", "claims"].contains(&other)
                }
                "claim-contributions" => collection == "claims" && other == "contributions",
                _ => true,
            };
            if !meaningful {
                return Err("dependency target matcher cannot match declared collections".into());
            }
            if ["change", "modules"].contains(&collection)
                && !["change", "modules"].contains(&other)
            {
                return Err("initial task cannot depend on unavailable later inventory".into());
            }
        }
        if let Some(correction) = action.get("correction").and_then(Json::as_str) {
            if !deps.contains(correction) {
                return Err("correction must name a direct prerequisite action".into());
            }
        } else if action.get("correction") != Some(&Json::Null) {
            return Err("correction requires action ID or null".into());
        }
    }
    Ok(())
}
fn parameters(v: &Json) -> Result<(), String> {
    let mut fields = vec![
        "project",
        "change",
        "request",
        "instructions",
        "modules",
        "assurance",
    ];
    if v.get("previous_assurance").is_some() {
        fields.push("previous_assurance");
        for inventory in arr(v, "previous_assurance")? {
            inventory_shape(inventory)?;
        }
    }
    obj(v, &fields)?;
    identifier(text(v, "project")?)?;
    identifier(text(v, "change")?)?;
    ref_shape(v.get("request").unwrap(), false)?;
    if text(v.get("request").unwrap(), "kind")? != "initial-request" {
        return Err("request binding requires initial-request kind".into());
    }
    for input in arr(v, "instructions")? {
        ref_shape(input, false)?;
    }
    let modules = arr(v, "modules")?;
    if modules.is_empty() {
        return Err(
            "explicit nonempty module inventory required; unknown scope is not completion".into(),
        );
    }
    let mut seen = BTreeSet::new();
    for module in modules {
        obj(module, &["module"])?;
        identifier(text(module, "module")?)?;
        if !seen.insert(text(module, "module")?) {
            return Err("duplicate inventory module".into());
        }
    }
    if v.get("assurance") != Some(&Json::Null) {
        inventory_shape(v.get("assurance").unwrap())?;
    }
    Ok(())
}
fn inventory_shape(v: &Json) -> Result<(), String> {
    obj(
        v,
        &[
            "format",
            "version",
            "project",
            "model_revision",
            "snapshot",
            "checks",
            "contributions",
            "claims",
        ],
    )?;
    if text(v, "format")? != "azimuth-assurance-target-inventory" || number(v, "version")? != 1 {
        return Err("expected assurance target inventory version 1".into());
    }
    identifier(text(v, "project")?)?;
    fingerprint(text(v, "model_revision")?)?;
    ref_shape(v.get("snapshot").unwrap(), false)?;
    if text(v.get("snapshot").unwrap(), "kind")? != "assurance-snapshot" {
        return Err("inventory snapshot requires assurance-snapshot kind".into());
    }
    for (collection, fields) in [
        ("checks", vec!["check", "subject"]),
        ("contributions", vec!["check", "case", "claim", "subject"]),
        ("claims", vec!["claim", "subject"]),
    ] {
        let mut seen = BTreeSet::new();
        for target in arr(v, collection)? {
            obj(target, &fields)?;
            for field in &fields {
                if *field == "subject" {
                    fingerprint(text(target, field)?)?;
                } else {
                    identifier(text(target, field)?)?;
                }
            }
            if !seen.insert(digest(target)) {
                return Err("duplicate assurance inventory target".into());
            }
        }
    }
    Ok(())
}
fn concrete_id(action: &str, target: &Json) -> String {
    work_key(&Json::obj(vec![
        ("action", Json::str(action)),
        ("target", target.clone()),
    ]))
    .expect("validated action and target")
}
fn expand_generation(source: &Json, bindings: &Json) -> Result<Json, String> {
    template(source)?;
    parameters(bindings)?;
    let mut tasks: Vec<Json> = Vec::new();
    let mut by_action: BTreeMap<String, Vec<Json>> = BTreeMap::new();
    let base = Json::obj(vec![
        ("project", bindings.get("project").unwrap().clone()),
        ("change", bindings.get("change").unwrap().clone()),
    ]);
    for action in arr(source, "actions")? {
        let collection = text(action, "collection")?;
        let targets = match collection {
            "change" => vec![Json::obj(vec![])],
            "modules" => arr(bindings, "modules")?.to_vec(),
            "methods" => {
                if bindings.get("assurance") == Some(&Json::Null) {
                    vec![]
                } else {
                    let mut seen = BTreeSet::new();
                    arr(bindings.get("assurance").unwrap(), "checks")?
                        .iter()
                        .filter_map(|c| {
                            let check = c.get("check").unwrap().clone();
                            if seen.insert(check.as_str().unwrap().to_string()) {
                                Some(Json::obj(vec![("check", check)]))
                            } else {
                                None
                            }
                        })
                        .collect()
                }
            }
            _ => {
                if bindings.get("assurance") == Some(&Json::Null) {
                    vec![]
                } else {
                    arr(bindings.get("assurance").unwrap(), collection)?.to_vec()
                }
            }
        };
        let mut expanded = Vec::new();
        for selected in targets {
            let mut target = base.clone();
            if let Json::Obj(fields) = selected {
                for (k, v) in fields {
                    set(&mut target, &k, v);
                }
            }
            if !["change", "modules"].contains(&collection) {
                set(
                    &mut target,
                    "model_revision",
                    bindings
                        .get("assurance")
                        .unwrap()
                        .get("model_revision")
                        .unwrap()
                        .clone(),
                );
            }
            let mut deps = Vec::new();
            let mut correction = Json::Null;
            for dependency in arr(action, "depends_on")? {
                let id = text(dependency, "action")?;
                let candidates = by_action.get(id).ok_or("unknown action dependency")?;
                let matched = candidates
                    .iter()
                    .filter(|task| {
                        let other = task.get("target").unwrap();
                        match text(dependency, "match").unwrap() {
                            "same-module" => target.get("module") == other.get("module"),
                            "same-check" => {
                                target.get("check") == other.get("check")
                                    && (other.get("subject").is_none()
                                        || target.get("subject") == other.get("subject"))
                            }
                            "same-subject" => target.get("subject") == other.get("subject"),
                            "claim-contributions" => {
                                target.get("claim") == other.get("claim")
                                    && target.get("subject") == other.get("subject")
                            }
                            _ => true,
                        }
                    })
                    .collect::<Vec<_>>();
                if matched.is_empty() {
                    return Err(format!(
                        "required dependency {id} has no matching validated inventory target"
                    ));
                }
                for task in &matched {
                    deps.push(task.get("id").unwrap().clone());
                }
                if action.get("correction").and_then(Json::as_str) == Some(id) {
                    if matched.len() != 1 {
                        return Err("correction must resolve to exactly one target".into());
                    }
                    correction = matched[0].get("id").unwrap().clone();
                }
            }
            let mut inputs = Vec::new();
            for binding in arr(action, "inputs")? {
                match binding.as_str().unwrap() {
                    "request" => inputs.push(bindings.get("request").unwrap().clone()),
                    "instructions" => inputs.extend(arr(bindings, "instructions")?.iter().cloned()),
                    "snapshot" => inputs.push(
                        bindings
                            .get("assurance")
                            .filter(|v| *v != &Json::Null)
                            .ok_or("snapshot binding unavailable")?
                            .get("snapshot")
                            .unwrap()
                            .clone(),
                    ),
                    _ => unreachable!(),
                }
            }
            let task = Json::obj(vec![
                ("id", Json::str(concrete_id(text(action, "id")?, &target))),
                ("kind", action.get("kind").unwrap().clone()),
                ("action", action.get("id").unwrap().clone()),
                ("target", target),
                ("depends_on", Json::Arr(deps)),
                ("inputs", Json::Arr(inputs)),
                ("guidance", action.get("guidance").unwrap().clone()),
                (
                    "required_output",
                    action.get("required_output").unwrap().clone(),
                ),
                ("correction", correction),
                ("outcomes", action.get("outcomes").unwrap().clone()),
                ("satisfies", action.get("satisfies").unwrap().clone()),
            ]);
            tasks.push(task.clone());
            expanded.push(task);
        }
        by_action.insert(text(action, "id")?.into(), expanded);
    }
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-workflow-definition")),
        ("version", Json::Num(1.0)),
        ("id", bindings.get("change").unwrap().clone()),
        ("tasks", Json::Arr(tasks)),
        ("template", source.clone()),
        ("template_fingerprint", Json::str(digest(source))),
        ("parameters", bindings.clone()),
    ]))
}
fn initial_definition_inner(
    file: Option<&str>,
    standard: Option<&str>,
    inputs: Option<&str>,
) -> Result<Json, String> {
    if file.is_some() == standard.is_some() {
        return Err("select exactly one --definition FILE or --workflow change-development".into());
    }
    let selected = if let Some(file) = file {
        read(Path::new(file))?
    } else {
        shipped_template(standard.unwrap())?
    };
    if selected.get("format").and_then(Json::as_str) == Some("azimuth-workflow-template") {
        let bindings = read(Path::new(
            inputs.ok_or("template selection requires --inputs FILE")?,
        ))?;
        if bindings.get("previous_assurance").is_some() {
            return Err("initial inputs cannot inject historical assurance inventories".into());
        }
        if bindings.get("assurance") != Some(&Json::Null) {
            return Err("initial assurance inventory must be null; record validated inventory after candidate registration".into());
        }
        expand(&selected, &bindings)
    } else {
        if inputs.is_some() {
            return Err("concrete definition accepts no --inputs".into());
        }
        Ok(selected)
    }
}
fn validate_inventory(
    v: &Json,
    state: &Json,
    locations: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    inventory_shape(v)?;
    let parameters = state
        .get("definition")
        .unwrap()
        .get("parameters")
        .ok_or("inventory requires pinned template")?;
    if v.get("project") != parameters.get("project") {
        return Err("inventory belongs to another project".into());
    }
    let snapshot = read(&resolve(v.get("snapshot").unwrap(), locations)?)?;
    validate_snapshot(&snapshot)?;
    if snapshot.get("project") != v.get("project")
        || snapshot.get("model_revision") != v.get("model_revision")
    {
        return Err("inventory project/revision differs from snapshot".into());
    }
    let tasks = arr(state.get("definition").unwrap(), "tasks")?;
    if let Some(task) = tasks
        .iter()
        .find(|t| t.get("kind").and_then(Json::as_str) == Some("register-candidate-model-revision"))
    {
        let result =
            latest(state, text(task, "id")?).ok_or("candidate registration not recorded")?;
        let receipt = read(&resolve(
            record_of(arr(result, "records")?, "candidate-registration")?,
            locations,
        )?)?;
        registration(&receipt, task.get("target").unwrap())?;
        if receipt.get("model_revision") != v.get("model_revision") {
            return Err("inventory differs from registered candidate revision".into());
        }
    } else {
        return Err("assurance inventory requires candidate registration gate".into());
    }
    let authority = snapshot.get("authority").unwrap();
    for check in arr(v, "checks")? {
        if !arr(authority, "checks")?
            .iter()
            .any(|c| c.get("id") == check.get("check"))
        {
            return Err("inventory Check absent from candidate authority".into());
        }
    }
    for contribution in arr(v, "contributions")? {
        if !arr(v, "checks")?.iter().any(|c| {
            c.get("check") == contribution.get("check")
                && c.get("subject") == contribution.get("subject")
        }) {
            return Err("contribution has no Check/Subject execution target".into());
        }
        if !arr(authority, "cases")?.iter().any(|c| {
            c.get("id") == contribution.get("case") && c.get("claim") == contribution.get("claim")
        }) {
            return Err("contribution Case/Claim absent from candidate authority".into());
        }
        let claim = arr(authority, "claims")?
            .iter()
            .find(|c| c.get("id") == contribution.get("claim"))
            .ok_or("unknown contribution Claim")?;
        if !arr(claim, "bindings")?.iter().any(|b| {
            b.get("check") == contribution.get("check") && b.get("case") == contribution.get("case")
        }) {
            return Err("contribution Check–Case relation absent from candidate authority".into());
        }
    }
    for claim in arr(v, "claims")? {
        let candidate = arr(authority, "claims")?
            .iter()
            .find(|c| c.get("id") == claim.get("claim"))
            .ok_or("unknown inventory Claim")?;
        for binding in arr(candidate, "bindings")? {
            if !arr(v, "contributions")?.iter().any(|c| {
                c.get("claim") == claim.get("claim")
                    && c.get("subject") == claim.get("subject")
                    && c.get("check") == binding.get("check")
                    && c.get("case") == binding.get("case")
            }) {
                return Err(
                    "inventory omits a declared Claim contribution for exact Subject".into(),
                );
            }
        }
        if !arr(authority, "claims")?.iter().any(|c| {
            c.get("id") == claim.get("claim")
                && c.get("criticality").and_then(Json::as_str) != Some("routine")
        }) {
            return Err("Judgment inventory requires a non-routine candidate Claim".into());
        }
    }
    for claim in arr(authority, "claims")? {
        if text(claim, "criticality")? != "routine"
            && !arr(v, "claims")?
                .iter()
                .any(|c| c.get("claim") == claim.get("id"))
        {
            return Err("inventory omits a non-routine candidate Claim".into());
        }
    }
    Ok(())
}

fn snapshot_target(snapshot: &Json, target: &Json) -> Result<(), String> {
    for field in ["project", "model_revision"] {
        if let Some(expected) = target.get(field) {
            if snapshot.get(field) != Some(expected) {
                return Err(format!("snapshot {field} differs from pinned task target"));
            }
        }
    }
    Ok(())
}
fn registration(v: &Json, target: &Json) -> Result<(), String> {
    obj(v, &["format", "version", "project", "model_revision"])?;
    if text(v, "format")? != "azimuth-candidate-registration" || number(v, "version")? != 1 {
        return Err("expected candidate registration version 1".into());
    }
    identifier(text(v, "project")?)?;
    fingerprint(text(v, "model_revision")?)?;
    if target
        .get("project")
        .is_some_and(|p| Some(p) != v.get("project"))
    {
        return Err("candidate registration belongs to another project".into());
    }
    Ok(())
}

fn output_contract(action: &Json) -> Result<(), String> {
    let expected = match text(action, "kind")? {
        "author-spec-delta" => Some("intent-delta"),
        "author-design" => Some("claim-design"),
        "author-verification" => Some("verification-account"),
        "review-proposal" | "review-spec-delta" | "review-design" | "review-verification" => {
            Some("authoring-review")
        }
        "implement-change" => Some("implementation-result"),
        "request-implementation-approval" => Some("implementation-approval"),
        "register-candidate-model-revision" => Some("candidate-registration"),
        "inventory-assurance-targets" => Some("assurance-target-inventory"),
        "qualify-method" => Some("method-qualification"),
        "execute-qualified-check" => Some("run-bundle"),
        "assess-applicability" => Some("applicability-decision"),
        "judge-claim" => Some("claim-judgment"),
        _ => None,
    };
    if expected.is_some_and(|kind| {
        action
            .get("required_output")
            .and_then(|o| o.get("kind"))
            .and_then(Json::as_str)
            != Some(kind)
    }) {
        return Err("output kind differs from action's semantic record contract".into());
    }
    Ok(())
}

fn expand(source: &Json, bindings: &Json) -> Result<Json, String> {
    let mut generations = bindings
        .get("previous_assurance")
        .and_then(Json::as_array)
        .unwrap_or(&[])
        .to_vec();
    generations.push(
        bindings
            .get("assurance")
            .ok_or("missing assurance inventory binding")?
            .clone(),
    );
    let mut tasks = Vec::new();
    let mut seen: BTreeMap<String, Json> = BTreeMap::new();
    for inventory in generations {
        let mut parameters = bindings.clone();
        set(&mut parameters, "assurance", inventory);
        let expanded = expand_generation(source, &parameters)?;
        for task in arr(&expanded, "tasks")? {
            let id = text(task, "id")?.to_string();
            if let Some(prior) = seen.get(&id) {
                if prior != task {
                    return Err("inventory expansion would rewrite an existing task".into());
                }
            } else {
                seen.insert(id, task.clone());
                tasks.push(task.clone());
            }
        }
    }
    let mut expanded = expand_generation(source, bindings)?;
    set(&mut expanded, "tasks", Json::Arr(tasks));
    Ok(expanded)
}

fn shipped_template(id: &str) -> Result<Json, String> {
    let descriptor = crate::resources::WORKFLOWS
        .iter()
        .find(|w| w.id == id)
        .ok_or("unknown shipped workflow; available: change-development")?;
    crate::json::parse(descriptor.source).map_err(|e| format!("shipped workflow {id}: {e}"))
}
fn initial_definition(
    file: Option<&str>,
    standard: Option<&str>,
    inputs: Option<&str>,
) -> Result<Json, String> {
    initial_definition_inner(file, standard, inputs)
        .and_then(|v| {
            definition(&v)?;
            Ok(v)
        })
        .map_err(|e| {
            format!(
                "{}{}: line 1: {e}",
                file.unwrap_or(standard.unwrap_or("workflow selection")),
                inputs
                    .map(|p| format!(" with inputs {p}"))
                    .unwrap_or_default()
            )
        })
}
