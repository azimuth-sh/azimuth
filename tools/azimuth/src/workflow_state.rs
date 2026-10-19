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
    "validate-change-account",
    "prepare-candidate-model-revision",
    "register-candidate-model-revision",
    "prepare-publication-inputs",
    "review-claim",
    "execute-check",
    "assess-claim-health",
    "run-challengers",
    "run-challenger",
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
            "challenger",
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
    if v.get("record").is_some() {
        fields.push("record");
        identifier(text(v, "record")?)?;
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
    let bytes = if v.get("record").is_some() {
        crate::fingerprint::canonical_json(&selected_record(v, r)?).into_bytes()
    } else {
        fs::read(resolve(v, r)?).map_err(|e| e.to_string())?
    };
    let mut fields = vec![
        ("kind", Json::str(text(v, "kind")?)),
        ("ref", Json::str(text(v, "ref")?)),
        ("fingerprint", Json::str(hash(&bytes))),
    ];
    if let Some(repository) = v.get("repository") {
        fields.push(("repository", repository.clone()));
    }
    if let Some(record) = v.get("record") {
        fields.push(("record", record.clone()));
    }
    Ok(Json::obj(fields))
}
fn selected_record(reference: &Json, roots: &BTreeMap<String, PathBuf>) -> Result<Json, String> {
    let value = read(&resolve(reference, roots)?)?;
    let Some(id) = reference.get("record").and_then(Json::as_str) else {
        return Ok(value);
    };
    crate::review_authoring::records(&value)?;
    let entry = arr(&value, "records")?
        .iter()
        .find(|entry| entry.get("id").and_then(Json::as_str) == Some(id))
        .ok_or("selected review record does not exist in compiled bundle")?;
    let Json::Obj(fields) = entry else {
        return Err("compiled review entry requires object".into());
    };
    Ok(Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "assessment")
            .cloned()
            .collect(),
    ))
}
fn supporting_record(reference: &Json, roots: &BTreeMap<String, PathBuf>) -> Result<Json, String> {
    let value = selected_record(reference, roots)?;
    Ok(
        if text(reference, "kind")? == "candidate-preparation" && value.get("fingerprint").is_some()
        {
            crate::assurance_review::validate_authority(&value)?;
            Json::obj(vec![
                ("format", Json::str("azimuth-candidate-preparation")),
                ("version", Json::Num(1.0)),
                ("project", value.get("project").unwrap().clone()),
                ("model_revision", value.get("fingerprint").unwrap().clone()),
            ])
        } else if reference.get("record").is_some() {
            value
                .get("record")
                .ok_or("compiled entry has no native review")?
                .clone()
        } else {
            value
        },
    )
}
fn supporting_review_history(
    reference: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Vec<Json>, String> {
    let selected = supporting_record(reference, roots)?;
    let retained = crate::review_authoring::records(&read(&resolve(reference, roots)?)?)?;
    let mut history = vec![selected];
    let mut seen = BTreeSet::new();
    while let Some(previous) = history
        .last()
        .and_then(|review| review.get("provenance"))
        .and_then(|provenance| provenance.get("reassesses"))
        .and_then(Json::as_str)
    {
        if !seen.insert(previous.to_string()) {
            return Err("cyclic reassessment history".into());
        }
        let Some(before) = retained
            .iter()
            .find(|review| review.get("fingerprint").and_then(Json::as_str) == Some(previous))
        else {
            break;
        };
        history.push(before.clone());
    }
    Ok(history)
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
                "challenger",
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
            "qualify-method" | "execute-qualified-check" | "execute-check" => vec!["check"],
            "review-claim" => vec!["claim"],
            "run-challenger" => vec!["challenger"],
            "assess-claim-health" => vec!["claim", "subject"],
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
                input.get("record").and_then(Json::as_str),
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
        if [
            "qualify-method",
            "assess-applicability",
            "judge-claim",
            "review-claim",
            "assess-claim-health",
            "run-challenger",
        ]
        .contains(&text(task, "kind")?)
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
    let mut fields = vec![
        "format", "version", "id", "task", "basis", "actor", "outcome", "records",
    ];
    for optional in ["inputs", "decision", "challenge_assessment"] {
        if v.get(optional).is_some() {
            fields.push(optional);
        }
    }
    obj(v, &fields)?;
    if let Some(assessment) = v.get("challenge_assessment") {
        obj(assessment, &["assessment", "execution_gaps"])?;
        if !["no-findings", "findings", "inconclusive"].contains(&text(assessment, "assessment")?)
            || !matches!(assessment.get("execution_gaps"), Some(Json::Bool(_)))
        {
            return Err("invalid derived Challenger assessment".into());
        }
    }
    if let Some(inputs) = v.get("inputs") {
        for reference in inputs.as_array().ok_or("captured inputs require array")? {
            ref_shape(reference, true)?;
        }
    }
    if let Some(decision) = v.get("decision") {
        let mut fields = vec![
            "format",
            "version",
            "task",
            "outcome",
            "dependencies",
            "rationale",
        ];
        match text(decision, "format")? {
            "azimuth-authoring-review" => fields.extend(["reviewer", "findings"]),
            "azimuth-workflow-approval" => fields.push("actor"),
            _ => return Err("unknown captured decision kind".into()),
        }
        obj(decision, &fields)?;
        if number(decision, "version")? != 1 {
            return Err("captured decision requires version 1".into());
        }
        for reference in arr(decision, "dependencies")? {
            ref_shape(reference, true)?;
        }
    }
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
            record.get("record").and_then(Json::as_str),
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
fn challenger_claim_inventory(authority: &Json) -> Result<Json, String> {
    let mut selected: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for claim in arr(authority, "claims")? {
        for challenger in claim
            .get("challengers")
            .and_then(Json::as_array)
            .unwrap_or(&[])
        {
            for target in arr(challenger, "claims")? {
                selected
                    .entry(text(challenger, "id")?.into())
                    .or_default()
                    .insert(
                        target
                            .as_str()
                            .ok_or("Challenger Claim requires identity")?
                            .into(),
                    );
            }
        }
    }
    Ok(Json::Arr(
        selected
            .into_iter()
            .map(|(id, claims)| {
                Json::obj(vec![
                    ("challenger", Json::str(id)),
                    ("claims", Json::Arr(claims.iter().map(Json::str).collect())),
                ])
            })
            .collect(),
    ))
}
fn challenger_claims(bindings: &Json, target: &Json) -> Result<Vec<Json>, String> {
    let inventory = bindings
        .get("assurance")
        .ok_or("Challenger inventory missing")?;
    let mapping = arr(inventory, "challenger_claims")?
        .iter()
        .find(|mapping| mapping.get("challenger") == target.get("challenger"))
        .ok_or("Challenger Claim mapping missing; derive a fresh candidate inventory")?;
    Ok(arr(mapping, "claims")?.to_vec())
}
fn challenge_assessment(
    result: &Json,
    target: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    let reference = record_of(arr(result, "records")?, "run-bundle")?;
    let path = resolve(reference, roots)?;
    let bundle = crate::run::load(&path).map_err(|e| format!("invalid Challenger Run: {e:?}"))?;
    if !crate::run::verify(&bundle).is_empty()
        || !bundle.check_executions.is_empty()
        || bundle.challenger_executions.is_empty()
    {
        return Err("Challenger task requires a valid nonempty Challenge-only Run".into());
    }
    let challenger = text(target, "challenger")?;
    if bundle
        .challenger_executions
        .iter()
        .any(|e| e.challenger.id != challenger)
    {
        return Err("Run contains a different Challenger than the task target".into());
    }
    let snapshot = read(&resolve(
        record_of(arr(result, "inputs")?, "assurance-snapshot")?,
        roots,
    )?)?;
    if snapshot.get("model_revision") != target.get("model_revision") {
        return Err("Challenger Run candidate revision differs from task".into());
    }
    let declarations = arr(snapshot.get("authority").unwrap(), "claims")?
        .iter()
        .flat_map(|c| c.get("challengers").and_then(Json::as_array).unwrap_or(&[]))
        .filter(|d| d.get("id").and_then(Json::as_str) == Some(challenger))
        .collect::<Vec<_>>();
    let declaration = declarations
        .first()
        .ok_or("task Challenger absent from candidate")?;
    if declarations.iter().any(|d| d != declaration) {
        return Err("conflicting Challenger declarations".into());
    }
    let expected = arr(declaration, "claims")?
        .iter()
        .map(|id| id.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let actual = bundle
        .challenger_executions
        .iter()
        .map(|e| e.target.id.as_str())
        .collect::<BTreeSet<_>>();
    if actual != expected
        || bundle.challenger_executions.iter().any(|e| {
            Some(e.challenger.fingerprint.as_str())
                != declaration.get("fingerprint").and_then(Json::as_str)
        })
    {
        return Err("Run does not cover the exact declared Challenger and all its Claims".into());
    }
    let findings = bundle
        .challenger_executions
        .iter()
        .any(|e| e.result.outcome.name() == "findings");
    let gaps = bundle.diagnostics.iter().any(|d| {
        d.class != crate::run::DiagnosticClass::Objection
            && d.severity != crate::run::Severity::Info
    }) || bundle
        .activities
        .iter()
        .any(|a| a.status != crate::run::ActivityStatus::Completed)
        || bundle.challenger_executions.iter().any(|e| {
            e.result.outcome.name() == "inconclusive"
                || e.units.iter().any(|u| {
                    u.attempts
                        .iter()
                        .any(|a| a.outcome.name() == "inconclusive")
                })
        });
    Ok(Json::obj(vec![
        (
            "assessment",
            Json::str(if findings {
                "findings"
            } else if gaps {
                "inconclusive"
            } else {
                "no-findings"
            }),
        ),
        ("execution_gaps", Json::Bool(gaps)),
    ]))
}
fn challenge_disposition(
    result: &Json,
    task: &Json,
    dependencies: &[Json],
    roots: &BTreeMap<String, PathBuf>,
) -> Result<bool, String> {
    for input in arr(result, "inputs")? {
        if input.get("kind").and_then(Json::as_str) != Some("assurance-snapshot") {
            check_ref(input, roots)?;
        }
    }
    let assessment = challenge_assessment(result, task.get("target").unwrap(), roots)?;
    if text(result, "outcome")? != "completed"
        || text(&assessment, "assessment")? != "findings"
        || assessment.get("execution_gaps") != Some(&Json::Bool(false))
    {
        return Ok(false);
    }
    let bundle = crate::run::load(&resolve(
        record_of(arr(result, "records")?, "run-bundle")?,
        roots,
    )?)
    .map_err(|error| format!("invalid Challenger Run: {error:?}"))?;
    let mut reviews = Vec::new();
    for dependency in dependencies {
        if text(dependency, "outcome")? != "accepted" {
            continue;
        }
        for reference in arr(dependency, "records")? {
            if reference.get("kind").and_then(Json::as_str) == Some("claim-review") {
                reviews.push(supporting_record(reference, roots)?);
            }
        }
    }
    for execution in &bundle.challenger_executions {
        if execution.result.outcome.name() != "findings" {
            continue;
        }
        if !reviews.iter().any(|review| {
            review.get("claim").and_then(Json::as_str) == Some(execution.target.id.as_str())
                && review.get("fingerprint").and_then(Json::as_str)
                    != Some(execution.target.fingerprint.as_str())
                && review
                    .get("challenge_basis")
                    .and_then(Json::as_array)
                    .is_some_and(|proofs| {
                        proofs.iter().any(|proof| {
                            proof.get("run").and_then(Json::as_str) == Some(bundle.run_id.as_str())
                                && proof.get("bundle_fingerprint").and_then(Json::as_str)
                                    == Some(bundle.bundle_fingerprint.as_str())
                                && proof.get("result_fingerprint").and_then(Json::as_str)
                                    == Some(execution.result.fingerprint.as_str())
                                && proof.get("target_fingerprint").and_then(Json::as_str)
                                    == Some(execution.target.fingerprint.as_str())
                        })
                    })
        }) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn successful(result: &Json) -> bool {
    if let Some(assessment) = result.get("challenge_assessment") {
        return result.get("outcome").and_then(Json::as_str) == Some("completed")
            && assessment.get("assessment").and_then(Json::as_str) == Some("no-findings")
            && assessment.get("execution_gaps") == Some(&Json::Bool(false));
    }
    matches!(
        result.get("outcome").and_then(Json::as_str),
        Some("completed" | "accepted" | "supported" | "approved")
    )
}
fn review_kind(kind: &str) -> bool {
    matches!(
        kind,
        "review-proposal" | "review-spec-delta" | "review-design" | "review-verification"
    )
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
            if let Some(subject) = task.get("target").and_then(|v| v.get("subject")) {
                if let Some(subject_inputs) = state
                    .get("definition")
                    .and_then(|v| v.get("parameters"))
                    .and_then(|v| v.get("assurance"))
                    .and_then(|v| v.get("subject_inputs"))
                    .and_then(Json::as_array)
                {
                    for input in subject_inputs
                        .iter()
                        .filter(|v| v.get("subject") == Some(subject))
                    {
                        if let Err(error) = check_ref(input.get("input").unwrap(), r) {
                            reasons.push(format!("selected Subject input: {error}"));
                        }
                    }
                }
            }

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
            if text(task, "kind")? == "review-claim" {
                let history = arr(state, "history")?;
                let implementation_position = dep_results
                    .iter()
                    .filter(|result| {
                        arr(result, "records").is_ok_and(|records| {
                            records.iter().any(|reference| {
                                reference.get("kind").and_then(Json::as_str)
                                    == Some("implementation-result")
                            })
                        })
                    })
                    .filter_map(|result| {
                        history
                            .iter()
                            .position(|entry| entry.get("result") == Some(result))
                    })
                    .max();
                if let Some(position) = implementation_position {
                    for entry in &history[..position] {
                        for reference in arr(entry.get("result").unwrap(), "records")? {
                            if reference.get("kind").and_then(Json::as_str) != Some("run-bundle") {
                                continue;
                            }
                            match check_ref(reference, r) {
                                Ok(()) => {
                                    if !input_values.contains(reference) {
                                        input_values.push(reference.clone());
                                    }
                                }
                                Err(error) => reasons.push(error),
                            }
                        }
                    }
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
                                && (text(review, "kind").ok() != Some("run-challenger")
                                    || result
                                        .get("challenge_assessment")
                                        .and_then(|v| v.get("assessment"))
                                        .and_then(Json::as_str)
                                        == Some("findings"))
                                && (result
                                    .get("challenge_assessment")
                                    .and_then(|v| v.get("assessment"))
                                    .and_then(Json::as_str)
                                    == Some("findings")
                                    || matches!(
                                        result.get("outcome").and_then(Json::as_str),
                                        Some(
                                            "rejected"
                                                | "violated"
                                                | "unresolved"
                                                | "failed"
                                                | "denied"
                                                | "deferred"
                                        )
                                    ))
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
                if !successful(previous)
                    && (text(task, "kind")? != "run-challenger"
                        || previous
                            .get("challenge_assessment")
                            .and_then(|v| v.get("assessment"))
                            .and_then(Json::as_str)
                            == Some("findings"))
                {
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
            let disposition = text(task, "kind")? == "run-challenger"
                && reasons.is_empty()
                && latest(state, id).is_some_and(|result| {
                    challenge_disposition(result, task, &dep_results, r).unwrap_or(false)
                });
            let valid = !correction_pending
                && latest(state, id).is_some_and(|result| {
                    (successful(result) || disposition)
                        && (text(task, "kind").ok() != Some("run-challenger")
                            || challenge_assessment(result, task.get("target").unwrap(), r)
                                .is_ok_and(|value| {
                                    result.get("challenge_assessment") == Some(&value)
                                }))
                        && task.get("satisfies").is_none_or(|allowed| {
                            allowed.as_array().is_some_and(|values| {
                                values.contains(result.get("outcome").unwrap())
                            })
                        })
                        && (disposition || result.get("basis").and_then(Json::as_str) == Some(&b))
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
                if text(task, "kind")? == "run-challenger" {
                    if let Some(previous) = latest(state, id) {
                        set(
                            eligible.last_mut().unwrap(),
                            "previous_attempt",
                            previous.clone(),
                        );
                    }
                }
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
        references.extend(
            arr(dependency, "inputs")?
                .iter()
                .filter(|reference| {
                    reference.get("kind").and_then(Json::as_str) == Some("run-bundle")
                })
                .cloned(),
        );
    }
    let mut inputs = Vec::new();
    let mut seen = BTreeMap::new();
    for reference in references {
        let key = (
            text(&reference, "kind")?.to_owned(),
            reference_repository(&reference).to_owned(),
            text(&reference, "ref")?.to_owned(),
            reference
                .get("record")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_owned(),
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
fn correction_source(native: &Json, roots: &BTreeMap<String, PathBuf>) -> Result<Json, String> {
    let artifacts = arr(native, "artifacts")?;
    let mut matches = Vec::new();
    for artifact in artifacts {
        let Some((path, section)) = text(artifact, "locator")?.split_once('#') else {
            continue;
        };
        portable(path)?;
        let mut locations = roots.clone();
        if let Some(directory) = roots
            .get("")
            .filter(|p| p.file_name().is_some_and(|n| n == ".azimuth"))
        {
            locations.insert(String::new(), directory.parent().unwrap().to_path_buf());
        }
        for (repository, root) in &locations {
            let reference = Json::obj(vec![
                ("kind", Json::str("review-source")),
                ("ref", Json::str(path)),
            ]);
            let context = BTreeMap::from([(String::new(), root.clone())]);
            let Ok(resolved) = resolve(&reference, &context) else {
                continue;
            };
            let Ok(source) = fs::read_to_string(&resolved) else {
                continue;
            };
            let Ok(line) = crate::review_authoring::section_location(
                &source,
                section,
                text(artifact, "digest")?,
            ) else {
                continue;
            };
            let value = Json::obj(vec![
                ("kind", Json::str("review-source")),
                ("directory", Json::str(root.to_string_lossy())),
                ("ref", Json::str(path)),
                ("section", Json::str(section)),
                ("line", Json::Num(line as f64)),
                (
                    "section_fingerprint",
                    artifact.get("digest").unwrap().clone(),
                ),
                (
                    "repository",
                    if repository.is_empty() {
                        Json::Null
                    } else {
                        Json::str(repository)
                    },
                ),
            ]);
            if !matches.iter().any(|(p, _)| p == &resolved) {
                matches.push((resolved, value));
            }
        }
    }
    if matches.len() != 1 {
        return Err("authored review source is missing, changed, outside declared roots or ambiguous; use the retained feedback".into());
    }
    Ok(matches.remove(0).1)
}
fn public_correction(result: &Json, roots: &BTreeMap<String, PathBuf>) -> Result<Json, String> {
    let task = result
        .get("task")
        .ok_or("correction has no work reference")?;
    let mut feedback = Vec::new();
    for reference in arr(result, "records")? {
        if ["work-result", "run-bundle"].contains(&text(reference, "kind")?) {
            continue;
        }
        if text(reference, "kind")? == "challenger-report" {
            check_ref(reference, roots)?;
            feedback.push(Json::obj(vec![
                ("kind", Json::str("challenger-report")),
                ("report", reference.clone()),
                (
                    "directory",
                    Json::str(
                        roots
                            .get(reference_repository(reference))
                            .ok_or("unknown report root")?
                            .to_string_lossy(),
                    ),
                ),
            ]));
            continue;
        }
        match check_ref(reference, roots).and_then(|()| supporting_record(reference, roots)) {
            Ok(native) => {
                let source = correction_source(&native, roots);
                let mut fields = vec![
                    ("kind", reference.get("kind").unwrap().clone()),
                    ("review", source.as_ref().cloned().unwrap_or(Json::Null)),
                ];
                if let Some(id) = reference.get("record") {
                    fields.push(("id", id.clone()));
                }
                if let Err(error) = source {
                    fields.push(("source_diagnostic", Json::str(error)));
                    fields.push((
                        "feedback",
                        native
                            .get("rationale")
                            .or_else(|| native.get("report"))
                            .cloned()
                            .unwrap_or(Json::Null),
                    ));
                }
                feedback.push(Json::obj(fields));
            }
            Err(error) => feedback.push(Json::obj(vec![
                ("kind", reference.get("kind").unwrap().clone()),
                ("source_diagnostic", Json::str(error)),
            ])),
        }
    }
    if let Some(assessment) = result.get("challenge_assessment") {
        feedback.push(Json::obj(vec![
            ("kind", Json::str("challenger-assessment")),
            ("assessment", assessment.clone()),
            ("guidance", Json::str("Investigate each objection; distinguish a product defect, inadequate Check coverage, equivalent or irrelevant mutation, and execution limitations. Record an independently reassessable disposition; do not turn surviving mutants into automatic product defects.")),
        ]));
    }
    if let Some(decision) = result.get("decision") {
        feedback.push(Json::obj(vec![
            ("kind", Json::str("authoring-review")),
            (
                "feedback",
                decision.get("rationale").cloned().unwrap_or(Json::Null),
            ),
        ]));
    }
    let inputs = arr(result, "inputs")?;
    let diagnostics = inputs
        .iter()
        .filter_map(|input| check_ref(input, roots).err().map(Json::str))
        .collect::<Vec<_>>();
    Ok(Json::obj(vec![
        ("work", task.clone()),
        ("outcome", result.get("outcome").unwrap().clone()),
        ("reviewer", result.get("actor").unwrap().clone()),
        ("reviewed_model_revision", encode_work(task).get("target").and_then(|target| target.get("model_revision")).cloned().unwrap_or(Json::Null)),
        ("input_currentness", Json::obj(vec![
            ("status", Json::str(if diagnostics.is_empty() { "unchanged" } else { "changed-or-unavailable" })),
            ("scope", Json::str("Retained review inputs only; source captures are not a fresh assessment of current working trees.")),
            ("diagnostics", Json::Arr(diagnostics)),
        ])),
        ("reviews", Json::Arr(feedback)),
        ("reviewed_artifacts", Json::Arr(inputs.iter().filter(|input| matches!(input.get("kind").and_then(Json::as_str), Some("candidate-facet" | "intent-delta" | "claim-design" | "verification-account" | "source-scope-report"))).cloned().collect())),
    ]))
}
fn public_handoff(handoff: &Json, roots: &BTreeMap<String, PathBuf>) -> Result<Json, String> {
    let inputs = handoff_inputs(handoff)?;
    let corrections = arr(handoff, "corrections")?
        .iter()
        .map(|result| public_correction(result, roots))
        .collect::<Result<Vec<_>, _>>()?;
    let previous_attempt = handoff
        .get("previous_attempt")
        .map(|previous| public_correction(previous, roots))
        .transpose()?;
    let Json::Obj(fields) = handoff else {
        return Err("handoff requires object".into());
    };
    let mut result = Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "dependencies")
            .map(|(key, value)| {
                (
                    key.clone(),
                    if key == "inputs" {
                        Json::Arr(inputs.clone())
                    } else if key == "corrections" {
                        Json::Arr(corrections.clone())
                    } else if key == "previous_attempt" {
                        previous_attempt.as_ref().unwrap().clone()
                    } else {
                        value.clone()
                    },
                )
            })
            .collect(),
    );
    if text(handoff, "task")? == "run-challenger" {
        set(
            &mut result,
            "challenger",
            challenger_handoff(handoff, roots)?,
        );
    }
    if !corrections.is_empty() {
        set(&mut result, "correction_guidance", Json::Arr(vec![
            Json::str("Read the referenced authored reviews and current change artifacts; compare each finding with current source and the reviewed inputs before relying on it."),
            Json::str("Update the existing engineering plan as needed, implement bounded corrections, and obtain a decision before material scope or intent changes."),
            Json::str("In the result report, identify each review and finding, record changes and actual verification or a justified disagreement, and retain unresolved limitations for independent reassessment."),
        ]));
        let mut required = handoff.get("required_output").unwrap().clone();
        let description = format!("{} Include a finding-by-finding disposition in the result report: changes and actual verification, justified disagreement, or an explicit unresolved limitation. Independent reassessment remains required.", text(&required, "description")?);
        set(&mut required, "description", Json::str(description));
        set(&mut result, "required_output", required);
    }
    Ok(result)
}
fn challenger_handoff(handoff: &Json, roots: &BTreeMap<String, PathBuf>) -> Result<Json, String> {
    let snapshot = read(&resolve(
        record_of(arr(handoff, "inputs")?, "assurance-snapshot")?,
        roots,
    )?)?;
    let identity = text(handoff.get("target").unwrap(), "challenger")?;
    let mut declaration = None;
    for claim in arr(snapshot.get("authority").unwrap(), "claims")? {
        for challenger in claim
            .get("challengers")
            .and_then(Json::as_array)
            .unwrap_or(&[])
        {
            if text(challenger, "id")? == identity {
                if declaration.is_some_and(|previous| previous != challenger) {
                    return Err("conflicting Challenger declarations".into());
                }
                declaration = Some(challenger);
            }
        }
    }
    let declaration = declaration.ok_or("Challenger missing from candidate")?;
    let mut reviews = Vec::new();
    for dependency in arr(handoff, "dependencies")? {
        for reference in arr(dependency, "records")? {
            if reference.get("kind").and_then(Json::as_str) != Some("claim-review") {
                continue;
            }
            let native = supporting_record(reference, roots)?;
            if !arr(declaration, "claims")?.contains(native.get("claim").unwrap_or(&Json::Null)) {
                continue;
            }
            let source = correction_source(&native, roots);
            let mut fields = vec![
                ("claim", native.get("claim").unwrap().clone()),
                ("review", source.as_ref().cloned().unwrap_or(Json::Null)),
                ("reviewed_model_revision", snapshot.get("model_revision").unwrap().clone()),
                ("input_currentness", Json::str("Current prerequisite basis; recapture and reassess if implementation, scope or configuration changes.")),
            ];
            if let Err(error) = source {
                fields.push(("source_diagnostic", Json::str(error)));
            }
            reviews.push(Json::obj(fields));
        }
    }
    let mut scopes = Vec::new();
    let mut scope_inputs = Vec::new();
    for input in handoff_inputs(handoff)? {
        if input.get("kind").and_then(Json::as_str) == Some("source-scope-report") {
            let scope = read(&resolve(&input, roots)?)?;
            if scope.get("challenger").and_then(Json::as_str) == Some(identity) {
                scopes.push(scope);
                scope_inputs.push(input);
            }
        } else if matches!(
            input.get("kind").and_then(Json::as_str),
            Some("source-capture" | "challenger-configuration")
        ) {
            scope_inputs.push(input);
        }
    }
    if scopes.len() > 1 {
        return Err("ambiguous prepared Challenger source scope".into());
    }
    let resolved_scope = scopes.pop().unwrap_or(Json::Null);
    Ok(Json::obj(vec![
        ("resolved_scope", resolved_scope),
        ("declaration", declaration.clone()),
        ("reviews", Json::Arr(reviews)),
        ("scope_inputs", Json::Arr(scope_inputs)),
        ("instructions", Json::Arr(vec![
            Json::str("Read the declaration and authored Claim Reviews. Resolve its implementation configuration files and current source scope; retain selected sites, exclusions, unresolved mappings and tool expansions. Missing or stale scope is not permission for a repository-wide fallback."),
            Json::str("Use explicit isolated tool/environment configuration and an exact source capture. Execute only this Challenger and its selected Checks. Preserve native reports, a readable challenger-report and the imported run-bundle in a fresh attempt directory."),
            Json::str("Record execution as completed or failed. Azimuth derives assessment no-findings, findings or inconclusive from the Run (native clean means no-findings). Preserve findings together with execution gaps; no findings never establishes Claim Health."),
            Json::str("For execution failure or inconclusive search, diagnose configuration, prerequisites, scope and tool limitations and retry this work. Do not classify an execution problem as a product defect. Surviving mutations require relevance/equivalence/coverage disposition and independent reassessment."),
        ])),
        ("destinations", Json::obj(vec![
            ("directory", Json::str(format!("artifacts/challengers/{identity}/<fresh-attempt>"))),
            ("readable_report", Json::str("report.md")),
            ("native_report", Json::str("native-report.json")),
            ("run_bundle", Json::str("run-bundle.json")),
            ("result", Json::str(format!("results/{identity}-<fresh-attempt>.json"))),
        ])),
    ]))
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
        let approval = decision_record(result, handoff, "implementation-approval", r)?;
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
        let v = decision_record(result, handoff, "authoring-review", r)?;
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
    } else if [
        "qualify-method",
        "assess-applicability",
        "judge-claim",
        "review-claim",
        "assess-claim-health",
    ]
    .contains(&kind)
    {
        let expected = match kind {
            "qualify-method" => "method-qualification",
            "assess-applicability" => "applicability-decision",
            "review-claim" => "claim-review",
            "assess-claim-health" => "claim-health",
            _ => "claim-judgment",
        };
        let review = supporting_record(record_of(records, expected)?, r)?;
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
        if ["qualify-method", "review-claim"].contains(&kind) {
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
        let mut local_reviews = arr(&snapshot, "reviews")?.to_vec();
        let mut runs = snapshot_runs(&snapshot)?;
        if ["review-claim", "assess-claim-health"].contains(&kind) {
            for dependency in arr(handoff, "dependencies")? {
                for reference in arr(dependency, "records")? {
                    if ["claim-review", "claim-health"].contains(&text(reference, "kind")?) {
                        for prior in supporting_review_history(reference, r)? {
                            if !local_reviews.contains(&prior) {
                                local_reviews.push(prior);
                            }
                        }
                    }
                }
            }
            for reference in handoff_inputs(handoff)? {
                if text(&reference, "kind")? == "run-bundle" {
                    let path = resolve(&reference, r)?;
                    let source = fs::read_to_string(&path).map_err(|error| error.to_string())?;
                    let bundle = crate::run::parse(&path.display().to_string(), &source)
                        .map_err(|error| format!("invalid prerequisite Run: {error:?}"))?;
                    if !runs.contains(&bundle) {
                        runs.push(bundle);
                    }
                }
            }
        }
        for retained in supporting_review_history(record_of(records, expected)?, r)? {
            if !local_reviews.contains(&retained) {
                local_reviews.push(retained);
            }
        }
        let prior = local_reviews.as_slice();
        let target = crate::assurance_review::review_target(&review)?;
        if crate::assurance_review::latest_review(prior, &target)? != Some(&review) {
            return Err(
                "review has been superseded or snapshot current target is ambiguous".into(),
            );
        }
        if let Some(subject) = handoff.get("target").and_then(|v| v.get("subject")) {
            if review.get("subject_fingerprint") != Some(subject) {
                return Err("review Subject differs from task target".into());
            }
        }
        let assessment = crate::assurance_review::assess_record(&review, authority, prior, &runs);
        let desired = text(result, "outcome")?;
        let matches = assessment.status == desired
            || (["judge-claim", "assess-claim-health"].contains(&kind)
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
fn decision_record(
    result: &Json,
    handoff: &Json,
    kind: &str,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    if let Some(decision) = result.get("decision") {
        return Ok(decision.clone());
    }
    let source = supporting_record(record_of(arr(result, "records")?, kind)?, roots)?;
    if source.get("work").is_none() {
        return Ok(source);
    }
    let mut fields = vec![
        (
            "format",
            Json::str(if kind == "authoring-review" {
                "azimuth-authoring-review"
            } else {
                "azimuth-workflow-approval"
            }),
        ),
        ("version", Json::Num(1.0)),
        ("task", result.get("task").unwrap().clone()),
        (
            if kind == "authoring-review" {
                "reviewer"
            } else {
                "actor"
            },
            result.get("actor").unwrap().clone(),
        ),
        ("outcome", result.get("outcome").unwrap().clone()),
        ("dependencies", Json::Arr(handoff_inputs(handoff)?)),
        ("rationale", Json::str(text(&source, "report")?)),
    ];
    if kind == "authoring-review" {
        fields.push((
            "findings",
            Json::Arr(if text(result, "outcome")? == "rejected" {
                vec![Json::str(text(&source, "report")?)]
            } else {
                vec![]
            }),
        ));
    }
    Ok(Json::obj(fields))
}
fn unchanged_inflight_assessment(
    state: &Json,
    captured: &Json,
    result: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Option<Json>, String> {
    let id = text(result, "task")?;
    let Some(task) = arr(state.get("definition").unwrap(), "tasks")?
        .iter()
        .find(|task| task.get("id") == result.get("task"))
    else {
        return Ok(None);
    };
    if !["review-claim", "run-challenger"].contains(&text(task, "kind")?) {
        return Ok(None);
    }
    let Some(handoff) = arr(captured, "eligible")?
        .iter()
        .find(|handoff| handoff.get("id") == result.get("task"))
    else {
        return Ok(None);
    };
    let captured_revision = number(captured, "workflow_revision")? as usize;
    if arr(state, "history")?
        .iter()
        .skip(captured_revision)
        .any(|entry| entry.get("result").and_then(|value| value.get("task")) == result.get("task"))
    {
        return Ok(None);
    }
    for input in arr(handoff, "inputs")? {
        if check_ref(input, roots).is_err() {
            return Ok(None);
        }
    }
    let inputs = arr(task, "inputs")?
        .iter()
        .map(|input| materialize(input, roots))
        .collect::<Result<Vec<_>, _>>()?;
    let mut dependencies = Vec::new();
    for dependency in arr(task, "depends_on")? {
        let Some(previous) = latest(state, dependency.as_str().unwrap()) else {
            return Ok(None);
        };
        if !successful(previous) {
            return Ok(None);
        }
        dependencies.push(previous.clone());
    }
    let current_basis = basis(task, &inputs, &dependencies, &[]);
    if handoff.get("basis").and_then(Json::as_str) != Some(current_basis.as_str())
        || result.get("basis").and_then(Json::as_str) != Some(current_basis.as_str())
    {
        return Ok(None);
    }
    // A sibling adverse assessment may reopen implementation while these captured inputs remain unchanged.
    Ok(Some(Json::obj(vec![
        ("id", Json::str(id)),
        ("task", task.get("kind").unwrap().clone()),
        ("target", task.get("target").unwrap().clone()),
        ("inputs", Json::Arr(inputs)),
        ("dependencies", Json::Arr(dependencies)),
        ("basis", Json::str(current_basis)),
        (
            "required_output",
            task.get("required_output").unwrap().clone(),
        ),
    ])))
}

fn authored_result(
    source: &Json,
    path: &Path,
    handoffs: &Json,
    state: &Json,
    actor: &str,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    let mut fields = vec!["work", "outcome", "outputs"];
    for optional in ["target", "report"] {
        if source.get(optional).is_some() {
            fields.push(optional);
        }
    }
    obj(source, &fields)?;
    identifier(text(source, "work")?)?;
    identifier(actor)?;
    if source.get("report").is_some() {
        text(source, "report")?;
    }
    if handoffs.get("format").and_then(Json::as_str) != Some("azimuth-workflow-handoff")
        || number(handoffs, "version")? != 1
        || handoffs.get("instance") != state.get("id")
    {
        return Err("--handoff requires this instance's workflow next output".into());
    }
    let requested = source.get("target");
    if let Some(target) = requested {
        work_key(&Json::obj(vec![
            ("action", source.get("work").unwrap().clone()),
            ("target", target.clone()),
        ]))?;
    }
    let candidates = arr(handoffs, "eligible")?
        .iter()
        .filter(|work| {
            work.get("action") == source.get("work")
                && requested.is_none_or(|target| {
                    if let Json::Obj(fields) = target {
                        fields.iter().all(|(key, value)| {
                            work.get("target").and_then(|v| v.get(key)) == Some(value)
                        })
                    } else {
                        false
                    }
                })
        })
        .collect::<Vec<_>>();
    if candidates.len() != 1 {
        return Err("work and target must select exactly one captured eligible handoff".into());
    }
    let handoff = candidates[0];
    let mut records = arr(source, "outputs")?
        .iter()
        .map(|reference| {
            ref_shape(reference, false)?;
            let mut reference = reference.clone();
            if text(handoff, "task")? == "prepare-candidate-model-revision"
                && text(&reference, "kind")? == "candidate-authority"
            {
                set(&mut reference, "kind", Json::str("candidate-preparation"));
            }
            materialize(&reference, roots)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let required = if text(source, "outcome")? == "failed" {
        "failure-report"
    } else {
        text(
            handoff
                .get("required_output")
                .ok_or("handoff has no output contract")?,
            "kind",
        )?
    };
    let uses_report = required == "failure-report"
        || required == "implementation-result"
        || required == "implementation-approval"
        || required == "authoring-review";
    let actual = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let root = roots.get("").unwrap();
    if !actual.starts_with(root) {
        return Err("authored result must be retained inside the workflow directory".into());
    }
    let relative = actual.strip_prefix(root).unwrap().to_string_lossy();
    if uses_report
        && !records
            .iter()
            .any(|record| record.get("kind").and_then(Json::as_str) == Some(required))
    {
        text(source, "report")?;
        records.push(materialize(
            &Json::obj(vec![
                ("kind", Json::str(required)),
                ("ref", Json::str(relative.as_ref())),
            ]),
            roots,
        )?);
    }
    if !records.iter().any(|record| {
        text(record, "ref").ok() == Some(relative.as_ref())
            && reference_repository(record).is_empty()
    }) {
        records.push(materialize(
            &Json::obj(vec![
                ("kind", Json::str("work-result")),
                ("ref", Json::str(relative.as_ref())),
            ]),
            roots,
        )?);
    }
    let identity = digest(&Json::obj(vec![
        ("source", source.clone()),
        ("actor", Json::str(actor)),
        ("task", handoff.get("id").unwrap().clone()),
        ("basis", handoff.get("basis").unwrap().clone()),
    ]));
    let mut result = Json::obj(vec![
        ("format", Json::str("azimuth-workflow-result")),
        ("version", Json::Num(1.0)),
        ("id", Json::str(format!("attempt-{}", &identity[7..]))),
        ("task", handoff.get("id").unwrap().clone()),
        ("basis", handoff.get("basis").unwrap().clone()),
        ("actor", Json::str(actor)),
        ("outcome", source.get("outcome").unwrap().clone()),
        ("records", Json::Arr(records)),
        ("inputs", Json::Arr(arr(handoff, "inputs")?.to_vec())),
    ]);
    if text(handoff, "task")? == "run-challenger"
        && (text(source, "outcome")? == "completed"
            || arr(&result, "records")?
                .iter()
                .any(|r| r.get("kind").and_then(Json::as_str) == Some("run-bundle")))
    {
        text(source, "report")?;
        record_of(arr(&result, "records")?, "challenger-report")?;
        let assessment = challenge_assessment(&result, handoff.get("target").unwrap(), roots)?;
        set(&mut result, "challenge_assessment", assessment);
    }
    if text(source, "outcome")? != "failed"
        && (review_kind(text(handoff, "task")?)
            || text(handoff, "task")? == "request-implementation-approval")
    {
        let context = Json::obj(vec![
            ("inputs", handoff.get("inputs").unwrap().clone()),
            ("dependencies", Json::Arr(vec![])),
        ]);
        let decision = decision_record(&result, &context, required, roots)?;
        set(&mut result, "decision", decision);
    }
    Ok(result)
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
        "assurance-targets" => vec![
            "--instance",
            "--change",
            "--repositories",
            "--project",
            "--snapshot",
            "--subjects",
        ],
        "next" | "verify" => vec!["--instance", "--change", "--repositories", "--project"],
        "update" => vec![
            "--instance",
            "--change",
            "--result",
            "--handoff",
            "--actor",
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
    if operation == "assurance-targets" {
        let snapshot =
            Json::obj(vec![
                ("kind", Json::str("assurance-snapshot")),
                (
                    "ref",
                    Json::str(get("--snapshot").ok_or(
                        "assurance-targets requires --snapshot workflow-relative reference",
                    )?),
                ),
            ]);
        let selection = get("--subjects").map(|value| {
            Json::obj(vec![
                ("kind", Json::str("assurance-subject-selection")),
                ("ref", Json::str(value)),
            ])
        });
        return assurance_definition(&state, &snapshot, selection.as_ref(), &r);
    }
    if operation == "update" {
        let result_path = Path::new(get("--result").ok_or("update requires --result")?);
        let authored = read(result_path)?;
        let handoffs = read(Path::new(get("--handoff").ok_or(
            "update requires --handoff FILE captured before performing the work",
        )?))?;
        let result = authored_result(
            &authored,
            result_path,
            &handoffs,
            &state,
            get("--actor").ok_or("update requires --actor ID from the executing harness")?,
            &r,
        )
        .map_err(|error| format!("{}: line 1: {error}", result_path.display()))?;
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
        if expected < number(&handoffs, "workflow_revision")?
            || handoffs.get("definition_fingerprint") != state.get("definition_fingerprint")
        {
            return Err("handoff revision or definition differs from update context".into());
        }
        if expected != number(&state, "revision")? {
            return Err("workflow revision conflict".into());
        }
        let evaluation = evaluate(&state, &r)?;
        let inflight = unchanged_inflight_assessment(&state, &handoffs, &result, &r)?;
        let handoff = evaluation
            .eligible
            .iter()
            .find(|t| t.get("id") == result.get("task"))
            .or(inflight.as_ref())
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
        } else if review_kind(kind)
            || ["qualify-method", "assess-applicability", "review-claim"].contains(&kind)
        {
            ["accepted", "rejected"].contains(&outcome)
        } else if ["judge-claim", "assess-claim-health"].contains(&kind) {
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
        if kind == "prepare-publication-inputs" && outcome == "completed" {
            let inputs = supporting_record(
                record_of(arr(&result, "records")?, "publication-inputs")?,
                &r,
            )?;
            crate::release::publication_inputs(&inputs)?;
            let request = inputs.get("request").unwrap();
            snapshot_target(
                request
                    .get("snapshot")
                    .ok_or("publication snapshot missing")?,
                handoff.get("target").unwrap(),
            )?;
        }
        if [
            "prepare-candidate-model-revision",
            "register-candidate-model-revision",
        ]
        .contains(&kind)
            && outcome == "completed"
        {
            let receipt = supporting_record(
                record_of(
                    arr(&result, "records")?,
                    if kind == "prepare-candidate-model-revision" {
                        "candidate-preparation"
                    } else {
                        "candidate-registration"
                    },
                )?,
                &r,
            )?;
            registration(&receipt, handoff.get("target").unwrap())?;
            let expected_format = if kind == "prepare-candidate-model-revision" {
                "azimuth-candidate-preparation"
            } else {
                "azimuth-candidate-registration"
            };
            if text(&receipt, "format")? != expected_format {
                return Err(format!("{kind} requires {expected_format}"));
            }
        }
        if [
            "execute-qualified-check",
            "execute-check",
            "run-challengers",
            "run-challenger",
        ]
        .contains(&kind)
            && (outcome == "completed"
                || kind == "run-challenger"
                    && arr(&result, "records")?
                        .iter()
                        .any(|r| r.get("kind").and_then(Json::as_str) == Some("run-bundle")))
        {
            let reference = record_of(arr(&result, "records")?, "run-bundle")?;
            let path = resolve(reference, &r)?;
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let bundle = crate::run::parse(&path.display().to_string(), &source)
                .map_err(|e| format!("invalid Run: {e:?}"))?;
            if ["run-challengers", "run-challenger"].contains(&kind)
                && bundle.challenger_executions.is_empty()
            {
                return Err("Challenger execution requires recorded Challenge Results".into());
            }
            if ["run-challengers", "run-challenger"].contains(&kind) {
                let snapshot = read(&resolve(
                    record_of(arr(handoff, "inputs")?, "assurance-snapshot")?,
                    &r,
                )?)?;
                validate_snapshot(&snapshot)?;
                let authority = snapshot.get("authority").unwrap();
                let mut reviews = arr(&snapshot, "reviews")?.to_vec();
                for dependency in arr(handoff, "dependencies")? {
                    for reference in arr(dependency, "records")? {
                        if ["claim-review", "claim-health"].contains(&text(reference, "kind")?) {
                            for review in supporting_review_history(reference, &r)? {
                                if !reviews.contains(&review) {
                                    reviews.push(review);
                                }
                            }
                        }
                    }
                }
                let mut runs = snapshot_runs(&snapshot)?;
                for reference in handoff_inputs(handoff)? {
                    if text(&reference, "kind")? == "run-bundle" {
                        check_ref(&reference, &r)?;
                        let path = resolve(&reference, &r)?;
                        let source =
                            fs::read_to_string(&path).map_err(|error| error.to_string())?;
                        let previous = crate::run::parse(&path.display().to_string(), &source)
                            .map_err(|error| {
                                format!("invalid assessment Run history: {error:?}")
                            })?;
                        if !runs.contains(&previous) {
                            runs.push(previous);
                        }
                    }
                }
                for execution in &bundle.challenger_executions {
                    let declarations = arr(authority, "claims")?.iter().flat_map(|claim| {
                        claim
                            .get("challengers")
                            .and_then(Json::as_array)
                            .unwrap_or(&[])
                    });
                    if !declarations.into_iter().any(|challenger| {
                        challenger.get("id").and_then(Json::as_str)
                            == Some(execution.challenger.id.as_str())
                            && challenger.get("fingerprint").and_then(Json::as_str)
                                == Some(execution.challenger.fingerprint.as_str())
                            && challenger
                                .get("claims")
                                .and_then(Json::as_array)
                                .is_some_and(|claims| {
                                    claims.iter().any(|claim| {
                                        claim.as_str() == Some(execution.target.id.as_str())
                                    })
                                })
                    }) {
                        return Err("Challenge does not match the current declared Challenger and Claim scope".into());
                    }
                    if !["claim-review", "claim-health"].contains(&execution.target.kind.name()) {
                        return Err(
                            "New workflow Challenges must target Claim Review or Claim Health"
                                .into(),
                        );
                    }
                    let review = reviews
                        .iter()
                        .find(|review| {
                            review.get("fingerprint").and_then(Json::as_str)
                                == Some(execution.target.fingerprint.as_str())
                        })
                        .ok_or("Challenge target assessment missing from retained inputs")?;
                    if review.get("claim").and_then(Json::as_str)
                        != Some(execution.target.id.as_str())
                        || review.get("kind").and_then(Json::as_str)
                            != Some(execution.target.kind.name())
                    {
                        return Err(
                            "Challenge target kind or Claim differs from retained assessment"
                                .into(),
                        );
                    }
                    let target = crate::assurance_review::review_target(review)?;
                    if crate::assurance_review::latest_review(&reviews, &target)? != Some(review)
                        || crate::assurance_review::assess_record(
                            review, authority, &reviews, &runs,
                        )
                        .status
                            != "accepted"
                    {
                        return Err("Challenge target is not current and eligible".into());
                    }
                }
            }
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
        let live_inflight = unchanged_inflight_assessment(&state, &handoffs, &result, &r)?;
        if before
            .eligible
            .iter()
            .find(|t| t.get("id") == result.get("task"))
            .or(live_inflight.as_ref())
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
        let records = arr(&result, "records")?;
        let inventory_extension = if outcome != "failed"
            && state.get("definition").unwrap().get("template").is_some()
            && (kind == "prepare-candidate-model-revision"
                || records.iter().any(|r| {
                    r.get("kind").and_then(Json::as_str) == Some("assurance-subject-selection")
                })) {
            let snapshot = records.iter().find(|r| r.get("kind").and_then(Json::as_str) == Some("assurance-snapshot"))
                .cloned().or_else(|| state.get("definition").unwrap().get("parameters")?.get("assurance")?.get("snapshot").cloned())
                .ok_or("candidate preparation requires an assurance-snapshot output for automatic target derivation")?;
            let selection = records.iter().find(|r| {
                r.get("kind").and_then(Json::as_str) == Some("assurance-subject-selection")
            });
            let mut prepared = state.clone();
            if let Json::Obj(fields) = &mut prepared {
                if let Some((_, Json::Arr(history))) =
                    fields.iter_mut().find(|(key, _)| key == "history")
                {
                    history.push(Json::obj(vec![
                        ("result_fingerprint", Json::str(digest(&result))),
                        ("result", result.clone()),
                    ]));
                }
            }
            Some(assurance_definition(&prepared, &snapshot, selection, &r)?)
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
        ("required_inputs", assurance_requirements(&state)?),
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
                    .map(|handoff| public_handoff(handoff, &r))
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
    for flag in ["--handoff", "--result", "--definition", "--inputs"] {
        if let Some(input) = value(flag) {
            if canonical(&input)? == output {
                return Err(format!("output aliases {flag} input"));
            }
        }
    }
    for flag in ["--snapshot", "--subjects"] {
        if let Some(input) = value(flag) {
            let input = path.parent().unwrap_or(Path::new(".")).join(input);
            if canonical(&input)? == output {
                return Err(format!("output aliases {flag} input"));
            }
        }
    }
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
        "review-claim"
        | "qualify-method"
        | "assess-applicability"
        | "review-proposal"
        | "review-spec-delta"
        | "review-design"
        | "review-verification" => &["accepted", "rejected"],
        "judge-claim" | "assess-claim-health" => &["supported", "violated", "unresolved"],
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
            "claim-definitions",
            "challengers",
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
        if [
            "qualify-method",
            "assess-applicability",
            "judge-claim",
            "review-claim",
            "assess-claim-health",
            "run-challenger",
        ]
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
                    "same-claim",
                    "challenger-claims",
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
                "challenger-claims" => collection == "challengers" && other == "claim-definitions",
                "same-claim" => {
                    ["claims", "claim-definitions"].contains(&collection)
                        && ["claims", "claim-definitions"].contains(&other)
                }
                _ => true,
            };
            if !meaningful {
                return Err("dependency target matcher cannot match declared collections".into());
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
    let mut fields = vec![
        "format",
        "version",
        "project",
        "model_revision",
        "snapshot",
        "checks",
        "contributions",
        "claims",
    ];
    for optional in [
        "methods",
        "claim_definitions",
        "challengers",
        "challenger_claims",
        "pending_subjects",
        "selection",
        "subject_inputs",
    ] {
        if v.get(optional).is_some() {
            fields.push(optional);
        }
    }
    obj(v, &fields)?;
    if let Some(mapping) = v.get("challenger_claims") {
        let mut ids = BTreeSet::new();
        for entry in mapping
            .as_array()
            .ok_or("Challenger Claim mapping requires array")?
        {
            obj(entry, &["challenger", "claims"])?;
            identifier(text(entry, "challenger")?)?;
            if !ids.insert(text(entry, "challenger")?) || arr(entry, "claims")?.is_empty() {
                return Err(
                    "Challenger mapping requires unique identities and nonempty Claim scope".into(),
                );
            }
            for claim in arr(entry, "claims")? {
                identifier(
                    claim
                        .as_str()
                        .ok_or("Challenger Claim requires identifier")?,
                )?;
            }
        }
    }
    if v.get("challengers").is_some() {
        let mut seen = BTreeSet::new();
        for challenger in arr(v, "challengers")? {
            let challenger = challenger.as_str().ok_or("Challenger ID requires string")?;
            identifier(challenger)?;
            if !seen.insert(challenger) {
                return Err("duplicate inventory Challenger".into());
            }
        }
    }
    if v.get("claim_definitions").is_some() {
        let mut seen = BTreeSet::new();
        for target in arr(v, "claim_definitions")? {
            obj(target, &["claim"])?;
            identifier(text(target, "claim")?)?;
            if !seen.insert(text(target, "claim")?) {
                return Err("duplicate Claim definition target".into());
            }
        }
    }
    if v.get("subject_inputs").is_some() {
        for input in arr(v, "subject_inputs")? {
            obj(input, &["subject", "input"])?;
            fingerprint(text(input, "subject")?)?;
            ref_shape(input.get("input").unwrap(), true)?;
        }
    }
    if let Some(selection) = v.get("selection") {
        ref_shape(selection, false)?;
    }
    for field in ["methods", "pending_subjects"] {
        if v.get(field).is_some() {
            let mut seen = BTreeSet::new();
            for target in arr(v, field)? {
                if field == "methods" {
                    obj(target, &["check"])?;
                    identifier(text(target, "check")?)?;
                } else {
                    obj(target, &["kind", "id"])?;
                    identifier(text(target, "id")?)?;
                    if !["check", "claim"].contains(&text(target, "kind")?) {
                        return Err("pending Subject target requires check or claim kind".into());
                    }
                }
                if !seen.insert(digest(target)) {
                    return Err("duplicate derived assurance target".into());
                }
            }
        }
    }
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
        if text(action, "kind")? == "run-challengers"
            && bindings
                .get("assurance")
                .and_then(|inventory| inventory.get("challengers"))
                .and_then(Json::as_array)
                .is_some_and(|challengers| challengers.is_empty())
        {
            by_action.insert(text(action, "id")?.into(), Vec::new());
            continue;
        }
        if collection == "change"
            && bindings
                .get("assurance")
                .and_then(|v| v.get("pending_subjects"))
                .and_then(Json::as_array)
                .is_some_and(|v| !v.is_empty())
            && arr(action, "depends_on")?.iter().any(|dep| {
                arr(source, "actions").unwrap().iter().any(|a| {
                    a.get("id") == dep.get("action") && text(a, "collection").ok() == Some("claims")
                })
            })
        {
            by_action.insert(text(action, "id")?.into(), Vec::new());
            continue;
        }
        if bindings.get("assurance") == Some(&Json::Null)
            && arr(action, "depends_on")?.iter().any(|dep| {
                by_action
                    .get(text(dep, "action").unwrap())
                    .is_some_and(Vec::is_empty)
            })
        {
            by_action.insert(text(action, "id")?.into(), Vec::new());
            continue;
        }
        let targets = match collection {
            "change" => vec![Json::obj(vec![])],
            "modules" => arr(bindings, "modules")?.to_vec(),
            "claim-definitions" => {
                if bindings.get("assurance") == Some(&Json::Null) {
                    vec![]
                } else {
                    arr(bindings.get("assurance").unwrap(), "claim_definitions")?.to_vec()
                }
            }
            "challengers" => {
                if bindings.get("assurance") == Some(&Json::Null) {
                    vec![]
                } else {
                    arr(bindings.get("assurance").unwrap(), "challengers")?
                        .iter()
                        .map(|id| Json::obj(vec![("challenger", id.clone())]))
                        .collect()
                }
            }
            "methods" => {
                if bindings.get("assurance") == Some(&Json::Null) {
                    vec![]
                } else {
                    let mut seen = BTreeSet::new();
                    let inventory = bindings.get("assurance").unwrap();
                    arr(
                        inventory,
                        if inventory.get("methods").is_some() {
                            "methods"
                        } else {
                            "checks"
                        },
                    )?
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
            let follows_assurance = arr(action, "depends_on")?.iter().any(|dep| {
                by_action
                    .get(text(dep, "action").unwrap())
                    .is_some_and(|tasks| {
                        tasks.iter().any(|task| {
                            task.get("target")
                                .is_some_and(|t| t.get("model_revision").is_some())
                        })
                    })
            });
            if !["change", "modules"].contains(&collection) || follows_assurance {
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
                            "same-claim" => target.get("claim") == other.get("claim"),
                            "challenger-claims" => {
                                challenger_claims(bindings, &target).is_ok_and(|claims| {
                                    other
                                        .get("claim")
                                        .is_some_and(|claim| claims.contains(claim))
                                })
                            }
                            "claim-contributions" => {
                                target.get("claim") == other.get("claim")
                                    && target.get("subject") == other.get("subject")
                            }
                            _ => true,
                        }
                    })
                    .collect::<Vec<_>>();
                if matched.is_empty() {
                    let complete_empty_challengers = text(dependency, "match")? == "all"
                        && arr(source, "actions")?.iter().any(|action| {
                            action.get("id") == dependency.get("action")
                                && matches!(
                                    text(action, "kind").ok(),
                                    Some("run-challengers" | "run-challenger")
                                )
                        })
                        && bindings
                            .get("assurance")
                            .and_then(|inventory| inventory.get("challengers"))
                            .and_then(Json::as_array)
                            .is_some_and(|challengers| challengers.is_empty());
                    let complete_empty_reviews = text(dependency, "match")? == "all"
                        && arr(source, "actions")?.iter().any(|action| {
                            action.get("id") == dependency.get("action")
                                && text(action, "collection").ok() == Some("claim-definitions")
                        })
                        && bindings
                            .get("assurance")
                            .and_then(|inventory| inventory.get("claim_definitions"))
                            .and_then(Json::as_array)
                            .is_some_and(|claims| claims.is_empty());
                    let complete_empty_claims = bindings
                        .get("assurance")
                        .filter(|v| *v != &Json::Null)
                        .is_some_and(|v| arr(v, "claims").is_ok_and(|claims| claims.is_empty()))
                        && text(dependency, "match")? == "all"
                        && arr(source, "actions")?.iter().any(|a| {
                            text(a, "id").ok() == Some(id)
                                && text(a, "collection").ok() == Some("claims")
                        });
                    if !complete_empty_claims
                        && !complete_empty_challengers
                        && !complete_empty_reviews
                    {
                        return Err(format!(
                            "required dependency {id} has no matching validated inventory target"
                        ));
                    }
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
            if target.get("subject").is_some() {
                if let Some(subject_inputs) = bindings
                    .get("assurance")
                    .and_then(|v| v.get("subject_inputs"))
                    .and_then(Json::as_array)
                {
                    for selected in subject_inputs
                        .iter()
                        .filter(|v| v.get("subject") == target.get("subject"))
                    {
                        let mut input = selected.get("input").unwrap().clone();
                        if let Json::Obj(fields) = &mut input {
                            fields.retain(|(key, _)| key != "fingerprint");
                        }
                        if !inputs.contains(&input) {
                            inputs.push(input);
                        }
                    }
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
            return Err("initial assurance inventory must be null; candidate preparation derives the inventory automatically".into());
        }
        expand(&selected, &bindings)
    } else {
        if inputs.is_some() {
            return Err("concrete definition accepts no --inputs".into());
        }
        Ok(selected)
    }
}
fn assurance_requirements(state: &Json) -> Result<Json, String> {
    let pending = state
        .get("definition")
        .and_then(|v| v.get("parameters"))
        .and_then(|v| v.get("assurance"))
        .and_then(|v| v.get("pending_subjects"))
        .and_then(Json::as_array)
        .unwrap_or(&[]);
    Ok(Json::Arr(pending.iter().map(|target| Json::obj(vec![
        ("kind", Json::str("assurance-subject-selection")),
        ("target", target.clone()),
        ("reason", Json::str("Select an exact execution Subject for this target. Claim composition requires every declared contribution on that same Subject; model revision equality does not join different Subjects.")),
    ])).collect()))
}

fn assurance_definition(
    state: &Json,
    snapshot_ref: &Json,
    selection_ref: Option<&Json>,
    locations: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    let old = state.get("definition").unwrap();
    let mut parameters = old
        .get("parameters")
        .ok_or("automatic assurance expansion requires a pinned template")?
        .clone();
    let snapshot = read(&resolve(snapshot_ref, locations)?)?;
    validate_snapshot(&snapshot)?;
    let authority = snapshot.get("authority").unwrap();
    let current = parameters.get("assurance").unwrap();
    let selection_ref = selection_ref.or_else(|| {
        if current.get("model_revision") == snapshot.get("model_revision") {
            current.get("selection")
        } else {
            None
        }
    });
    let mut checks = Vec::new();
    let mut selected_claims = Vec::new();
    let mut subject_inputs = Vec::new();
    if let Some(reference) = selection_ref {
        ref_shape(reference, false)?;
        if text(reference, "kind")? != "assurance-subject-selection" {
            return Err("expected assurance-subject-selection reference".into());
        }
        let selection = read(&resolve(reference, locations)?)?;
        obj(
            &selection,
            &[
                "format",
                "version",
                "project",
                "model_revision",
                "checks",
                "claims",
            ],
        )?;
        if text(&selection, "format")? != "azimuth-assurance-subject-selection"
            || number(&selection, "version")? != 1
        {
            return Err("expected assurance Subject selection version 1".into());
        }
        if selection.get("project") != snapshot.get("project")
            || selection.get("model_revision") != snapshot.get("model_revision")
        {
            return Err("Subject selection differs from candidate project/revision".into());
        }
        for collection in ["checks", "claims"] {
            let field = if collection == "checks" {
                "check"
            } else {
                "claim"
            };
            let mut seen = BTreeSet::new();
            for target in arr(&selection, collection)? {
                obj(target, &[field, "subject"])?;
                identifier(text(target, field)?)?;
                let entity = arr(authority, collection)?
                    .iter()
                    .find(|entity| entity.get("id") == target.get(field))
                    .ok_or("selected target absent from candidate authority")?;
                if collection == "claims" && text(entity, "criticality")? == "routine" {
                    return Err("routine Claim has no Judgment obligation".into());
                }
                let reference = target.get("subject").unwrap();
                ref_shape(reference, false)?;
                if text(reference, "kind")? != "subject" {
                    return Err("selected Subject requires subject reference kind".into());
                }
                let value = read(&resolve(reference, locations)?)?;
                let subject = crate::run::subject_from_json(&value)?;
                if !crate::run::validate_subject_component(&subject).is_empty() {
                    return Err("selected Subject fails native protocol validation".into());
                }
                let input = Json::obj(vec![
                    (
                        "subject",
                        Json::str(crate::run::subject_fingerprint(&subject)),
                    ),
                    ("input", materialize(reference, locations)?),
                ]);
                if !subject_inputs.contains(&input) {
                    subject_inputs.push(input);
                }
                let target = Json::obj(vec![
                    (field, target.get(field).unwrap().clone()),
                    (
                        "subject",
                        Json::str(crate::run::subject_fingerprint(&subject)),
                    ),
                ]);
                if !seen.insert(digest(&target)) {
                    return Err("duplicate selected target/Subject".into());
                }
                if collection == "checks" {
                    checks.push(target);
                } else {
                    selected_claims.push(target);
                }
            }
        }
    }
    let methods = arr(authority, "checks")?
        .iter()
        .map(|check| Json::obj(vec![("check", check.get("id").unwrap().clone())]))
        .collect::<Vec<_>>();
    let mut contributions = Vec::new();
    let mut claims = Vec::new();
    let mut pending = Vec::new();
    for method in &methods {
        if !checks
            .iter()
            .any(|check| check.get("check") == method.get("check"))
        {
            pending.push(Json::obj(vec![
                ("kind", Json::str("check")),
                ("id", method.get("check").unwrap().clone()),
            ]));
        }
    }
    for claim in arr(authority, "claims")? {
        if text(claim, "criticality")? == "routine" {
            continue;
        }
        for binding in arr(claim, "bindings")? {
            for check in checks
                .iter()
                .filter(|check| check.get("check") == binding.get("check"))
            {
                contributions.push(Json::obj(vec![
                    ("check", binding.get("check").unwrap().clone()),
                    ("case", binding.get("case").unwrap().clone()),
                    ("claim", claim.get("id").unwrap().clone()),
                    ("subject", check.get("subject").unwrap().clone()),
                ]));
            }
        }
        let targets = selected_claims
            .iter()
            .filter(|target| target.get("claim") == claim.get("id"))
            .collect::<Vec<_>>();
        let mut complete = !targets.is_empty();
        for target in targets {
            if arr(claim, "bindings")?.iter().all(|binding| {
                checks.iter().any(|check| {
                    check.get("check") == binding.get("check")
                        && check.get("subject") == target.get("subject")
                })
            }) {
                claims.push(target.clone());
            } else {
                complete = false;
            }
        }
        if !complete {
            pending.push(Json::obj(vec![
                ("kind", Json::str("claim")),
                ("id", claim.get("id").unwrap().clone()),
            ]));
        }
    }
    let portable_ref = |reference: &Json| {
        let mut value = reference.clone();
        if let Json::Obj(fields) = &mut value {
            fields.retain(|(key, _)| key != "fingerprint");
        }
        value
    };
    let mut inventory = Json::obj(vec![
        ("format", Json::str("azimuth-assurance-target-inventory")),
        ("version", Json::Num(1.0)),
        ("project", snapshot.get("project").unwrap().clone()),
        (
            "model_revision",
            snapshot.get("model_revision").unwrap().clone(),
        ),
        ("snapshot", portable_ref(snapshot_ref)),
        ("methods", Json::Arr(methods)),
        (
            "claim_definitions",
            Json::Arr(
                arr(authority, "claims")?
                    .iter()
                    .filter(|claim| text(claim, "criticality").ok() != Some("routine"))
                    .map(|claim| Json::obj(vec![("claim", claim.get("id").unwrap().clone())]))
                    .collect(),
            ),
        ),
        ("checks", Json::Arr(checks)),
        ("contributions", Json::Arr(contributions)),
        ("claims", Json::Arr(claims)),
        ("pending_subjects", Json::Arr(pending)),
        ("subject_inputs", Json::Arr(subject_inputs)),
    ]);
    if arr(authority, "claims")?
        .iter()
        .all(|claim| claim.get("challengers").is_some())
    {
        let challengers = arr(authority, "claims")?
            .iter()
            .flat_map(|claim| arr(claim, "challengers").unwrap())
            .map(|challenger| text(challenger, "id").unwrap().to_string())
            .collect::<BTreeSet<_>>();
        set(
            &mut inventory,
            "challengers",
            Json::Arr(challengers.iter().map(Json::str).collect()),
        );
    }
    set(
        &mut inventory,
        "challenger_claims",
        challenger_claim_inventory(authority)?,
    );
    if let Some(selection) = selection_ref {
        set(&mut inventory, "selection", portable_ref(selection));
    }
    validate_inventory(&inventory, state, locations)?;
    let previous = parameters.get("assurance").unwrap().clone();
    if previous != Json::Null && previous.get("model_revision") != inventory.get("model_revision") {
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
    let extended = expand(old.get("template").unwrap(), &parameters)?;
    for history in arr(state, "history")? {
        let id = history.get("result").unwrap().get("task").unwrap();
        let previous = arr(old, "tasks")?
            .iter()
            .find(|task| task.get("id") == Some(id))
            .ok_or("historical task absent")?;
        if !arr(&extended, "tasks")?.iter().any(|task| task == previous) {
            return Err("automatic target expansion would rewrite completed work".into());
        }
    }
    Ok(extended)
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
    if let Some(task) = tasks.iter().find(|t| {
        [
            "prepare-candidate-model-revision",
            "register-candidate-model-revision",
        ]
        .contains(&t.get("kind").and_then(Json::as_str).unwrap_or(""))
    }) {
        let result = latest(state, text(task, "id")?)
            .ok_or("candidate preparation or registration not recorded")?;
        let receipt = supporting_record(
            record_of(
                arr(result, "records")?,
                if text(task, "kind")? == "prepare-candidate-model-revision" {
                    "candidate-preparation"
                } else {
                    "candidate-registration"
                },
            )?,
            locations,
        )?;
        registration(&receipt, task.get("target").unwrap())?;
        if receipt.get("model_revision") != v.get("model_revision") {
            return Err("inventory differs from prepared candidate revision".into());
        }
    } else {
        return Err(
            "assurance inventory requires candidate preparation or registration gate".into(),
        );
    }
    let authority = snapshot.get("authority").unwrap();
    if let Some(claims) = v.get("claim_definitions") {
        let expected = Json::Arr(
            arr(authority, "claims")?
                .iter()
                .filter(|claim| text(claim, "criticality").ok() != Some("routine"))
                .map(|claim| Json::obj(vec![("claim", claim.get("id").unwrap().clone())]))
                .collect(),
        );
        if claims != &expected {
            return Err("Claim definition targets differ from candidate authority".into());
        }
    }
    if let Some(challengers) = v.get("challengers") {
        if !arr(authority, "claims")?
            .iter()
            .all(|claim| claim.get("challengers").is_some())
        {
            return Err(
                "Historical authority does not establish Challenger inventory completeness".into(),
            );
        }
        let expected = arr(authority, "claims")?
            .iter()
            .flat_map(|claim| arr(claim, "challengers").unwrap())
            .map(|challenger| text(challenger, "id").unwrap().to_string())
            .collect::<BTreeSet<_>>();
        if challengers != &Json::Arr(expected.iter().map(Json::str).collect()) {
            return Err("Challenger inventory differs from candidate authority".into());
        }
    }
    if let Some(mapping) = v.get("challenger_claims") {
        if mapping != &challenger_claim_inventory(authority)? {
            return Err("Challenger Claim mapping differs from candidate authority".into());
        }
    }
    if let Some(methods) = v.get("methods").and_then(Json::as_array) {
        let expected = arr(authority, "checks")?
            .iter()
            .map(|check| text(check, "id").map(str::to_owned))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let actual = methods
            .iter()
            .map(|method| text(method, "check").map(str::to_owned))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if actual != expected {
            return Err("derived methods differ from candidate Checks".into());
        }
    }
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
            && !v
                .get("pending_subjects")
                .and_then(Json::as_array)
                .is_some_and(|pending| {
                    pending.iter().any(|p| {
                        text(p, "kind").ok() == Some("claim") && p.get("id") == claim.get("id")
                    })
                })
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
    if ![
        "azimuth-candidate-registration",
        "azimuth-candidate-preparation",
    ]
    .contains(&text(v, "format")?)
        || number(v, "version")? != 1
    {
        return Err("expected candidate preparation or registration version 1".into());
    }
    identifier(text(v, "project")?)?;
    fingerprint(text(v, "model_revision")?)?;
    if target
        .get("project")
        .is_some_and(|p| Some(p) != v.get("project"))
    {
        return Err("candidate registration belongs to another project".into());
    }
    if target
        .get("model_revision")
        .is_some_and(|revision| v.get("model_revision") != Some(revision))
    {
        return Err("candidate receipt differs from pinned model revision".into());
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
        "validate-change-account" => Some("candidate-validation"),
        "request-implementation-approval" => Some("implementation-approval"),
        "prepare-candidate-model-revision" => Some("candidate-preparation"),
        "register-candidate-model-revision" => Some("candidate-registration"),
        "prepare-publication-inputs" => Some("publication-inputs"),
        "qualify-method" => Some("method-qualification"),
        "review-claim" => Some("claim-review"),
        "execute-check" | "run-challengers" | "run-challenger" => Some("run-bundle"),
        "assess-claim-health" => Some("claim-health"),
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
