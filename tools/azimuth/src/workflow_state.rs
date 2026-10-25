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
    "author-design-delta",
    "author-verification",
    "author-verification-delta",
    "review-spec-delta",
    "review-design",
    "review-design-delta",
    "review-verification",
    "review-verification-delta",
    "author-engineering-plan",
    "author-change-outcome",
    "request-change-acceptance",
    "finalize-change",
    "request-implementation-approval",
    "implement-change",
    "capture-source-support",
    "validate-candidate",
    "validate-change-account",
    "prepare-candidate-model-revision",
    "register-candidate-model-revision",
    "prepare-publication-inputs",
    "review-requirement",
    "execute-check",
    "run-verification-challenges",
    "run-verification-challenge",
    "qualify-method",
    "execute-qualified-check",
    "assess-applicability",
];
/// Action kinds whose outcome was an authored Verification-Target-specific Requirement assessment.
/// Requirement Status is now computed from applicable evidence, so instances pinned to these
/// templates are rejected rather than silently losing their assessment gates.
const AUTHORED_REQUIREMENT_STATUS_KINDS: &[&str] =
    &["assess-requirement-status", "judge-requirement"];
fn registered_kind(kind: &str) -> Result<(), String> {
    if AUTHORED_REQUIREMENT_STATUS_KINDS.contains(&kind) {
        return Err(format!(
            "workflow action kind `{kind}` authors a Requirement Status assessment, which is no longer supported; deployment Requirement Status is computed by the Assurance Service from applicable evidence. Start a new instance from the shipped change-development template"
        ));
    }
    if !KINDS.contains(&kind) {
        return Err(format!("unregistered workflow task kind `{kind}`"));
    }
    Ok(())
}
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
            "requirement",
            "scenario",
            "check",
            "probe",
            "verification_challenge",
            "verification_target",
            "model_revision",
        ]
        .contains(&key.as_str())
        {
            return Err("unknown work target locator".into());
        }
        let value = value
            .as_str()
            .ok_or("work target requires string locators")?;
        if key == "verification_target" || key == "model_revision" {
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
    let actual = match fs::canonicalize(&p) {
        Ok(actual) => actual,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if let Some(directory) = r.get("") {
                if let Some(relocated) =
                    crate::workflow_references::relocated(directory, root, text(v, "ref")?)?
                {
                    return Ok(relocated);
                }
            }
            return Err(format!("{}: {error}", p.display()));
        }
        Err(error) => return Err(format!("{}: {error}", p.display())),
    };
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
    let bytes = if text(v, "kind")? == "proposal-content" {
        let source = String::from_utf8(bytes).map_err(|_| "proposal content requires UTF-8")?;
        let mut metadata = true;
        let mut content = String::new();
        for line in source.split_inclusive('\n') {
            if line.starts_with("## ") {
                metadata = false;
            }
            if metadata && line.trim_start().starts_with("Status:") {
                content.push_str("Status: <workflow-owned>\n");
            } else {
                content.push_str(line);
            }
        }
        content.into_bytes()
    } else {
        bytes
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
        .and_then(|review| review.get("reassesses"))
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
    if text(v, "format")? != "azimuth-workflow-definition" || number(v, "version")? != 2 {
        return Err("expected workflow definition version 2".into());
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
        registered_kind(text(task, "kind")?)?;
        let Json::Obj(target) = task.get("target").unwrap() else {
            return Err("target requires object".into());
        };
        for (k, v) in target {
            if ![
                "project",
                "change",
                "module",
                "requirement",
                "scenario",
                "check",
                "probe",
                "verification_challenge",
                "verification_target",
                "model_revision",
            ]
            .contains(&k.as_str())
            {
                return Err("unknown target locator".into());
            }
            if k == "verification_target" || k == "model_revision" {
                fingerprint(
                    v.as_str()
                        .ok_or("verification target requires fingerprint")?,
                )?;
            } else {
                identifier(v.as_str().ok_or("target requires identifiers")?)?;
            }
        }
        for required in match text(task, "kind")? {
            "qualify-method" | "execute-qualified-check" | "execute-check" => vec!["check"],
            "review-requirement" => vec!["requirement"],
            "run-verification-challenge" => vec!["verification_challenge"],
            "assess-applicability" => vec!["check", "scenario", "verification_target"],
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
            "review-requirement",
            "run-verification-challenge",
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
    if let (Some("azimuth-workflow-instance"), Some(version @ (1.0 | 2.0))) = (
        v.get("format").and_then(Json::as_str),
        v.get("version").and_then(Json::as_num),
    ) {
        return Err(format!("workflow instance version {version} predates Work Provenance capture and is read only by `azimuth provenance ledger`; preserve it inside the change's .azimuth/ and run `workflow init --predecessor <it>` for a fresh version 3 instance"));
    }
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
            "predecessor",
            "abandoned",
        ],
    )?;
    if text(v, "format")? != "azimuth-workflow-instance" || number(v, "version")? != 3 {
        return Err("expected workflow instance version 3".into());
    }
    crate::provenance::validate_predecessor(v.get("predecessor").unwrap())?;
    crate::provenance::validate_abandonment(v.get("abandoned").unwrap())?;
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
        obj(item, &["result", "result_fingerprint", "provenance"])?;
        let result = item.get("result").unwrap();
        result_shape(result)?;
        crate::provenance::validate_history_provenance(item.get("provenance").unwrap())?;
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
    for optional in [
        "inputs",
        "decision",
        "challenge_assessment",
        "check_assessment",
    ] {
        if v.get(optional).is_some() {
            fields.push(optional);
        }
    }
    obj(v, &fields)?;
    if let Some(assessment) = v.get("check_assessment") {
        obj(assessment, &["passed"])?;
        if !matches!(assessment.get("passed"), Some(Json::Bool(_))) {
            return Err("invalid derived Check assessment".into());
        }
    }
    if let Some(assessment) = v.get("challenge_assessment") {
        obj(
            assessment,
            &["assessment", "execution_gaps", "reviewable_findings"],
        )?;
        if !["no-findings", "findings", "inconclusive"].contains(&text(assessment, "assessment")?)
            || !matches!(assessment.get("execution_gaps"), Some(Json::Bool(_)))
            || !matches!(assessment.get("reviewable_findings"), Some(Json::Bool(_)))
        {
            return Err("invalid derived Verification Challenge assessment".into());
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
    if text(v, "format")? != "azimuth-workflow-result" || number(v, "version")? != 3 {
        return Err("expected workflow result version 3".into());
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
fn verification_challenge_requirement_inventory(authority: &Json) -> Result<Json, String> {
    let mut selected: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for requirement in arr(authority, "requirements")? {
        for verification_challenge in requirement
            .get("verification_challenges")
            .and_then(Json::as_array)
            .unwrap_or(&[])
        {
            for target in arr(verification_challenge, "requirements")? {
                selected
                    .entry(text(verification_challenge, "id")?.into())
                    .or_default()
                    .insert(
                        target
                            .as_str()
                            .ok_or("Verification Challenge Requirement requires identity")?
                            .into(),
                    );
            }
        }
    }
    Ok(Json::Arr(
        selected
            .into_iter()
            .map(|(id, requirements)| {
                Json::obj(vec![
                    ("verification_challenge", Json::str(id)),
                    (
                        "requirements",
                        Json::Arr(requirements.iter().map(Json::str).collect()),
                    ),
                ])
            })
            .collect(),
    ))
}
fn verification_challenge_requirements(
    bindings: &Json,
    target: &Json,
) -> Result<Vec<Json>, String> {
    let inventory = bindings
        .get("assurance")
        .ok_or("Verification Challenge inventory missing")?;
    let mapping = arr(inventory, "verification_challenge_requirements")?
        .iter()
        .find(|mapping| mapping.get("verification_challenge") == target.get("verification_challenge"))
        .ok_or("Verification Challenge Requirement mapping missing; derive a fresh candidate inventory")?;
    Ok(arr(mapping, "requirements")?.to_vec())
}
fn challenge_assessment(
    result: &Json,
    target: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    let reference = record_of(arr(result, "records")?, "run-bundle")?;
    let path = resolve(reference, roots)?;
    let bundle = crate::run::load(&path)
        .map_err(|e| format!("invalid Verification Challenge Run: {e:?}"))?;
    if !crate::run::verify(&bundle).is_empty()
        || !bundle.check_executions.is_empty()
        || bundle.verification_challenge_executions.is_empty()
    {
        return Err(
            "Verification Challenge task requires a valid nonempty Challenge-only Run".into(),
        );
    }
    let verification_challenge = text(target, "verification_challenge")?;
    if bundle
        .verification_challenge_executions
        .iter()
        .any(|e| e.verification_challenge.id != verification_challenge)
    {
        return Err("Run contains a different Verification Challenge than the task target".into());
    }
    let snapshot = read(&resolve(
        record_of(arr(result, "inputs")?, "assurance-snapshot")?,
        roots,
    )?)?;
    if snapshot.get("model_revision") != target.get("model_revision") {
        return Err("Verification Challenge Run candidate revision differs from task".into());
    }
    let declarations = arr(snapshot.get("authority").unwrap(), "requirements")?
        .iter()
        .flat_map(|c| {
            c.get("verification_challenges")
                .and_then(Json::as_array)
                .unwrap_or(&[])
        })
        .filter(|d| d.get("id").and_then(Json::as_str) == Some(verification_challenge))
        .collect::<Vec<_>>();
    let declaration = declarations
        .first()
        .ok_or("task Verification Challenge absent from candidate")?;
    if declarations.iter().any(|d| d != declaration) {
        return Err("conflicting Verification Challenge declarations".into());
    }
    let expected = arr(declaration, "requirements")?
        .iter()
        .map(|id| id.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let actual = bundle
        .verification_challenge_executions
        .iter()
        .map(|e| e.target.id.as_str())
        .collect::<BTreeSet<_>>();
    if (bundle
        .verification_challenge_executions
        .iter()
        .any(|e| e.target.kind != crate::run::ChallengeTargetKind::Implementation)
        && actual != expected)
        || bundle.verification_challenge_executions.iter().any(|e| {
            Some(e.verification_challenge.fingerprint.as_str())
                != declaration.get("fingerprint").and_then(Json::as_str)
        })
    {
        return Err(
            "Run does not cover the exact declared Verification Challenge and all its Requirements"
                .into(),
        );
    }
    for execution in &bundle.verification_challenge_executions {
        if execution.target.kind == crate::run::ChallengeTargetKind::Implementation
            && !crate::assurance_review::implementation_challenge_current(
                snapshot.get("authority").unwrap(),
                &bundle,
                execution,
            )?
        {
            return Err("Challenge implementation basis differs from current candidate".into());
        }
    }
    let findings = bundle
        .verification_challenge_executions
        .iter()
        .any(|e| e.result.outcome.name() == "findings");
    let gaps = bundle.status != crate::run::RunStatus::Complete
        || bundle.diagnostics.iter().any(|d| {
            d.class != crate::run::DiagnosticClass::Objection
                && d.severity != crate::run::Severity::Info
        })
        || bundle
            .activities
            .iter()
            .any(|a| a.status != crate::run::ActivityStatus::Completed)
        || bundle.verification_challenge_executions.iter().any(|e| {
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
        (
            "reviewable_findings",
            Json::Bool(
                findings && !gaps && proposed_challenge_dispositions(result, &bundle, roots)?,
            ),
        ),
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
    .map_err(|error| format!("invalid Verification Challenge Run: {error:?}"))?;
    let mut reviews = Vec::new();
    for dependency in dependencies {
        if text(dependency, "outcome")? != "accepted" {
            continue;
        }
        for reference in arr(dependency, "records")? {
            if reference.get("kind").and_then(Json::as_str) == Some("requirement-review") {
                reviews.push(supporting_record(reference, roots)?);
            }
        }
    }
    for execution in &bundle.verification_challenge_executions {
        if execution.result.outcome.name() != "findings" {
            continue;
        }
        if !reviews.iter().any(|review| {
            (if execution.target.kind == crate::run::ChallengeTargetKind::Implementation {
                let snapshot = arr(result, "inputs")
                    .ok()
                    .and_then(|inputs| record_of(inputs, "assurance-snapshot").ok())
                    .and_then(|reference| resolve(reference, roots).ok())
                    .and_then(|path| read(&path).ok());
                snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.get("authority"))
                    .and_then(|authority| authority.get("requirements"))
                    .and_then(Json::as_array)
                    .is_some_and(|requirements| {
                        requirements.iter().any(|requirement| {
                            requirement.get("id") == review.get("requirement")
                                && requirement
                                    .get("verification_challenges")
                                    .and_then(Json::as_array)
                                    .is_some_and(|declarations| {
                                        declarations.iter().any(|declaration| {
                                            declaration.get("id").and_then(Json::as_str)
                                                == Some(
                                                    execution.verification_challenge.id.as_str(),
                                                )
                                        })
                                    })
                        })
                    })
            } else {
                review.get("requirement").and_then(Json::as_str)
                    == Some(execution.target.id.as_str())
            }) && review.get("fingerprint").and_then(Json::as_str)
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
fn proposed_challenge_dispositions(
    result: &Json,
    bundle: &crate::run::RunBundle,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<bool, String> {
    let Some(reference) = arr(result, "records")?
        .iter()
        .find(|value| value.get("kind").and_then(Json::as_str) == Some("challenge-dispositions"))
    else {
        return Ok(false);
    };
    let value = supporting_record(reference, roots)?;
    obj(&value, &["format", "version", "results"])?;
    if text(&value, "format")? != "azimuth-challenge-dispositions"
        || number(&value, "version")? != 1
    {
        return Err("expected challenge dispositions version 1".into());
    }
    let rows = arr(&value, "results")?;
    let mut seen_results = BTreeSet::new();
    let mut dispositions = BTreeMap::new();
    for row in rows {
        obj(row, &["result_fingerprint", "issues"])?;
        let fingerprint = text(row, "result_fingerprint")?;
        if !seen_results.insert(fingerprint) {
            return Err("duplicate Challenge disposition result".into());
        }
        let execution = bundle
            .verification_challenge_executions
            .iter()
            .find(|execution| {
                execution.result.fingerprint == fingerprint
                    && execution.result.outcome == crate::run::ChallengeOutcome::Findings
            })
            .ok_or("disposition names no findings result in this Run")?;
        let expected = execution
            .result
            .objections
            .iter()
            .map(|objection| crate::assurance_review::challenge_issue_id(execution, objection))
            .collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::new();
        let mut justified = true;
        for issue in arr(row, "issues")? {
            obj(issue, &["issue", "disposition", "rationale"])?;
            let id = text(issue, "issue")?;
            if !expected.contains(id) || !seen.insert(id.to_string()) {
                return Err("unknown or duplicate Challenge issue disposition".into());
            }
            text(issue, "rationale")?;
            match text(issue, "disposition")? {
                "equivalent" | "irrelevant" => {}
                "defect" => justified = false,
                _ => return Err("unknown Challenge disposition".into()),
            }
        }
        if seen != expected {
            return Err("Challenge disposition does not cover every objection".into());
        }
        dispositions.insert(fingerprint, justified);
    }
    Ok(bundle
        .verification_challenge_executions
        .iter()
        .filter(|execution| execution.result.outcome == crate::run::ChallengeOutcome::Findings)
        .all(|execution| dispositions.get(execution.result.fingerprint.as_str()) == Some(&true)))
}

fn check_assessment(
    result: &Json,
    target: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    let bundle = crate::run::load(&resolve(
        record_of(arr(result, "records")?, "run-bundle")?,
        roots,
    )?)
    .map_err(|errors| format!("invalid Check Run: {errors:?}"))?;
    if !crate::run::verify(&bundle).is_empty() {
        return Err("Check Run has protocol findings".into());
    }
    let check_id = text(target, "check")?;
    let executions = bundle
        .check_executions
        .iter()
        .filter(|execution| execution.check.id == check_id)
        .collect::<Vec<_>>();
    if executions.is_empty() {
        return Err("Check Run has no selected Check execution".into());
    }
    if target.get("verification_target").and_then(Json::as_str)
        != Some(bundle.verification_target_fingerprint.as_str())
    {
        return Err("Check Run Verification Target differs from task".into());
    }
    let snapshot = read(&resolve(
        record_of(arr(result, "inputs")?, "assurance-snapshot")?,
        roots,
    )?)?;
    validate_snapshot(&snapshot)?;
    snapshot_target(&snapshot, target)?;
    let authority = snapshot.get("authority").unwrap();
    let check = arr(authority, "checks")?
        .iter()
        .find(|check| check.get("id").and_then(Json::as_str) == Some(check_id))
        .ok_or("Check absent from current candidate")?;
    if executions.iter().any(|execution| {
        check.get("fingerprint").and_then(Json::as_str)
            != Some(execution.check.fingerprint.as_str())
    }) {
        return Err("Check execution definition differs from current candidate".into());
    }
    let expected = arr(authority, "bindings")?
        .iter()
        .filter(|binding| binding.get("check").and_then(Json::as_str) == Some(check_id))
        .filter_map(|binding| binding.get("scenario").and_then(Json::as_str))
        .collect::<BTreeSet<_>>();
    let observed = executions
        .iter()
        .flat_map(|execution| &execution.results)
        .map(|result| result.scenario.as_str())
        .collect::<BTreeSet<_>>();
    let declared_units = check
        .get("deployment")
        .and_then(|policy| policy.get("units"))
        .and_then(Json::as_array)
        .map(|units| {
            units
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>()
        });
    let captured_units = arr(result, "records")?
        .iter()
        .find(|reference| reference.get("kind").and_then(Json::as_str) == Some("check-unit-inputs"))
        .map(|reference| -> Result<BTreeSet<String>, String> {
            let manifest = supporting_record(reference, roots)?;
            if manifest.get("format").and_then(Json::as_str) != Some("azimuth-unit-input-manifest")
            {
                return Err("expected native unit-input manifest".into());
            }
            crate::unit_inputs::validate_release_manifest(&manifest)?;
            Ok(arr(&manifest, "units")?
                .iter()
                .filter(|unit| {
                    unit.get("checks")
                        .and_then(Json::as_array)
                        .is_none_or(|ids| {
                            ids.is_empty() || ids.iter().any(|id| id.as_str() == Some(check_id))
                        })
                })
                .map(|unit| text(unit, "id").map(str::to_owned))
                .collect::<Result<_, _>>()?)
        })
        .transpose()?;
    let required_units = match (captured_units, declared_units) {
        (Some(mut captured), Some(declared)) => {
            captured.extend(declared);
            captured
        }
        (Some(captured), None) => captured,
        (None, Some(declared)) => declared,
        (None, None) => BTreeSet::from(["whole".to_string()]),
    };
    let observed_units = executions
        .iter()
        .flat_map(|execution| &execution.units)
        .map(|unit| unit.id.clone())
        .collect::<BTreeSet<_>>();
    let complete_units = !required_units.is_empty() && observed_units == required_units;
    let passed = complete_units
        && bundle.status == crate::run::RunStatus::Complete
        && !observed.is_empty()
        && expected == observed
        && bundle
            .activities
            .iter()
            .all(|activity| activity.status == crate::run::ActivityStatus::Completed)
        && executions.iter().all(|execution| {
            !execution.units.is_empty()
                && !execution.results.is_empty()
                && execution
                    .results
                    .iter()
                    .all(|result| result.outcome == crate::run::CheckResultOutcome::Satisfied)
                && execution.units.iter().all(|unit| {
                    !unit.attempts.is_empty()
                        && unit.attempts.iter().all(|attempt| {
                            attempt
                                .outcomes
                                .keys()
                                .map(String::as_str)
                                .collect::<BTreeSet<_>>()
                                == expected
                                && !attempt.outcomes.is_empty()
                                && attempt.outcomes.values().all(|outcome| {
                                    *outcome == crate::run::CheckResultOutcome::Satisfied
                                })
                        })
                        && unit
                            .executed_tests
                            .iter()
                            .all(|test| test.outcome == crate::run::ExecutedTestOutcome::Passed)
                })
        })
        && !bundle.diagnostics.iter().any(|diagnostic| {
            diagnostic.class != crate::run::DiagnosticClass::Objection
                && diagnostic.severity != crate::run::Severity::Info
        });
    Ok(Json::obj(vec![("passed", Json::Bool(passed))]))
}

fn successful(result: &Json) -> bool {
    if result
        .get("check_assessment")
        .is_some_and(|assessment| assessment.get("passed") != Some(&Json::Bool(true)))
    {
        return false;
    }
    if let Some(assessment) = result.get("challenge_assessment") {
        return result.get("outcome").and_then(Json::as_str) == Some("completed")
            && (assessment.get("assessment").and_then(Json::as_str) == Some("no-findings")
                || assessment.get("reviewable_findings") == Some(&Json::Bool(true)))
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
        "review-proposal"
            | "review-spec-delta"
            | "review-design"
            | "review-verification"
            | "review-design-delta"
            | "review-verification-delta"
    )
}
fn author_for(kind: &str) -> Option<&'static str> {
    match kind {
        "review-proposal" => Some("author-proposal"),
        "review-spec-delta" => Some("author-spec-delta"),
        "review-design" => Some("author-design"),
        "review-design-delta" => Some("author-design-delta"),
        "review-verification" => Some("author-verification"),
        "review-verification-delta" => Some("author-verification-delta"),
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
    if let Some(inventory) = state
        .get("definition")
        .and_then(|definition| definition.get("parameters"))
        .and_then(|parameters| parameters.get("assurance"))
        .filter(|inventory| **inventory != Json::Null)
    {
        validate_inventory(inventory, state, r)?;
    }
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
            if let Some(verification_target) = task
                .get("target")
                .and_then(|v| v.get("verification_target"))
            {
                if let Some(verification_target_inputs) = state
                    .get("definition")
                    .and_then(|v| v.get("parameters"))
                    .and_then(|v| v.get("assurance"))
                    .and_then(|v| v.get("verification_target_inputs"))
                    .and_then(Json::as_array)
                {
                    for input in verification_target_inputs
                        .iter()
                        .filter(|v| v.get("verification_target") == Some(verification_target))
                    {
                        if let Err(error) = check_ref(input.get("input").unwrap(), r) {
                            reasons.push(format!("selected Verification Target input: {error}"));
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
            if text(task, "kind")? == "review-requirement" {
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
                                && (text(review, "kind").ok() != Some("run-verification-challenge")
                                    || result
                                        .get("challenge_assessment")
                                        .and_then(|v| v.get("assessment"))
                                        .and_then(Json::as_str)
                                        == Some("findings"))
                                && !successful(result)
                                && (result
                                    .get("challenge_assessment")
                                    .and_then(|v| v.get("assessment"))
                                    .and_then(Json::as_str)
                                    == Some("findings")
                                    || !successful(result)
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
                if ["qualify-method", "assess-applicability"].contains(&text(task, "kind")?) {
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
                    && (text(task, "kind")? != "run-verification-challenge"
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
            let disposition = text(task, "kind")? == "run-verification-challenge"
                && reasons.is_empty()
                && latest(state, id).is_some_and(|result| {
                    challenge_disposition(result, task, &dep_results, r).unwrap_or(false)
                });
            let valid = !correction_pending
                && latest(state, id).is_some_and(|result| {
                    (successful(result) || disposition)
                        && (text(task, "kind").ok() != Some("run-verification-challenge")
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
                    if text(task, "kind").ok() == Some("execute-check")
                        && text(result, "outcome").ok() == Some("completed")
                    {
                        if check_assessment(result, task.get("target").unwrap(), r)
                            .ok()
                            .as_ref()
                            != result.get("check_assessment")
                        {
                            return false;
                        }
                    }
                    review_record(result, &handoff, text(task, "kind").unwrap(), r).is_ok()
                });
            current.insert(id.into(), valid && reasons.is_empty());
            if valid && reasons.is_empty() {
                continue;
            }
            if reasons.is_empty() {
                eligible.push(Json::obj(vec![("id",Json::str(id)),("task",Json::str(text(task,"kind")?)),("action",task.get("action").cloned().unwrap_or_else(|| Json::str(text(task,"kind").unwrap()))),("target",task.get("target").unwrap().clone()),("repositories",repository_locations(r)),("workflow_directory",Json::str(r.get("").unwrap().to_string_lossy())),("basis",Json::str(b)),("reason",Json::str(if !corrections.is_empty(){"Perform the configured correction after an adverse result; independent reassessment will follow."}else if latest(state,id).is_some(){"The previous result is adverse or its dependencies changed."}else{"The configured prerequisites are satisfied."})),("guidance",task.get("guidance").unwrap().clone()),("inputs",Json::Arr(input_values)),("dependencies",Json::Arr(dep_results)),("corrections",Json::Arr(corrections)),("required_output",task.get("required_output").unwrap().clone())]));
                if text(task, "kind")? == "run-verification-challenge" {
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
        if text(reference, "kind")? == "verification-challenge-report" {
            check_ref(reference, roots)?;
            feedback.push(Json::obj(vec![
                ("kind", Json::str("verification-challenge-report")),
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
            ("kind", Json::str("verification-challenge-assessment")),
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
            (
                "issues",
                decision
                    .get("findings")
                    .cloned()
                    .unwrap_or_else(|| Json::Arr(vec![])),
            ),
        ]));
    }
    let mut issue_references = BTreeSet::new();
    for reference in arr(result, "records")? {
        if let Ok(native) = supporting_record(reference, roots) {
            if let Some(issues) = native.get("issues").and_then(Json::as_array) {
                let fingerprint = native
                    .get("fingerprint")
                    .and_then(Json::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| digest(&native));
                for issue in issues {
                    if let Some(id) = issue.get("id").and_then(Json::as_str) {
                        issue_references
                            .insert(crate::authoring_review::issue_reference(&fingerprint, id));
                    }
                }
            }
        }
    }
    if let Some(decision) = result.get("decision") {
        if let Some(issues) = decision.get("findings").and_then(Json::as_array) {
            let fingerprint = digest(decision);
            for issue in issues {
                if let Some(id) = issue.get("id").and_then(Json::as_str) {
                    issue_references
                        .insert(crate::authoring_review::issue_reference(&fingerprint, id));
                }
            }
        }
    }
    let inputs = arr(result, "inputs")?;
    let diagnostics = inputs
        .iter()
        .filter_map(|input| check_ref(input, roots).err().map(Json::str))
        .collect::<Vec<_>>();
    Ok(Json::obj(vec![
        ("work", task.clone()),
        ("issue_references", Json::Arr(issue_references.iter().map(Json::str).collect())),
        ("outcome", result.get("outcome").unwrap().clone()),
        ("reviewer", result.get("actor").unwrap().clone()),
        ("reviewed_model_revision", encode_work(task).get("target").and_then(|target| target.get("model_revision")).cloned().unwrap_or(Json::Null)),
        ("input_currentness", Json::obj(vec![
            ("status", Json::str(if diagnostics.is_empty() { "unchanged" } else { "changed-or-unavailable" })),
            ("scope", Json::str("Retained review inputs only; source captures are not a fresh assessment of current working trees.")),
            ("diagnostics", Json::Arr(diagnostics)),
        ])),
        ("reviews", Json::Arr(feedback)),
        ("reviewed_artifacts", Json::Arr(inputs.iter().filter(|input| matches!(input.get("kind").and_then(Json::as_str), Some("candidate-facet" | "intent-delta" | "requirement-design" | "verification-account" | "source-scope-report"))).cloned().collect())),
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
    if text(handoff, "task")? == "run-verification-challenge" {
        set(
            &mut result,
            "verification_challenge",
            verification_challenge_handoff(handoff, roots)?,
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
fn verification_challenge_handoff(
    handoff: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    let snapshot = read(&resolve(
        record_of(arr(handoff, "inputs")?, "assurance-snapshot")?,
        roots,
    )?)?;
    let identity = text(handoff.get("target").unwrap(), "verification_challenge")?;
    let mut declaration = None;
    for requirement in arr(snapshot.get("authority").unwrap(), "requirements")? {
        for verification_challenge in requirement
            .get("verification_challenges")
            .and_then(Json::as_array)
            .unwrap_or(&[])
        {
            if text(verification_challenge, "id")? == identity {
                if declaration.is_some_and(|previous| previous != verification_challenge) {
                    return Err("conflicting Verification Challenge declarations".into());
                }
                declaration = Some(verification_challenge);
            }
        }
    }
    let declaration = declaration.ok_or("Verification Challenge missing from candidate")?;
    let mut reviews = Vec::new();
    for dependency in arr(handoff, "dependencies")? {
        for reference in arr(dependency, "records")? {
            if reference.get("kind").and_then(Json::as_str) != Some("requirement-review") {
                continue;
            }
            let native = supporting_record(reference, roots)?;
            if !arr(declaration, "requirements")?
                .contains(native.get("requirement").unwrap_or(&Json::Null))
            {
                continue;
            }
            let source = correction_source(&native, roots);
            let mut fields = vec![
                ("requirement", native.get("requirement").unwrap().clone()),
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
            if scope.get("verification_challenge").and_then(Json::as_str) == Some(identity) {
                scopes.push(scope);
                scope_inputs.push(input);
            }
        } else if matches!(
            input.get("kind").and_then(Json::as_str),
            Some("source-capture" | "verification-challenge-configuration")
        ) {
            scope_inputs.push(input);
        }
    }
    if scopes.len() > 1 {
        return Err("ambiguous prepared Verification Challenge source scope".into());
    }
    let resolved_scope = scopes.pop().unwrap_or(Json::Null);
    Ok(Json::obj(vec![
        ("resolved_scope", resolved_scope),
        ("declaration", declaration.clone()),
        ("reviews", Json::Arr(reviews)),
        ("scope_inputs", Json::Arr(scope_inputs)),
        ("instructions", Json::Arr(vec![
            Json::str("Read the declaration and authored Requirement Reviews. Resolve its implementation configuration files and current source scope; retain selected sites, exclusions, unresolved mappings and tool expansions. Missing or stale scope is not permission for a repository-wide fallback."),
            Json::str("Use explicit isolated tool/environment configuration and an exact source capture. Execute only this Verification Challenge and its selected Checks. Preserve native reports, a readable verification-challenge-report and the imported run-bundle in a fresh attempt directory."),
            Json::str("Record execution as completed or failed. Azimuth derives assessment no-findings, findings or inconclusive from the Run (native clean means no-findings). Preserve findings together with execution gaps; no findings never establishes Requirement Status."),
            Json::str("For execution failure or inconclusive search, diagnose configuration, prerequisites, scope and tool limitations and retry this work. Do not classify an execution problem as a product defect. Surviving mutations require relevance/equivalence/coverage disposition and independent reassessment."),
        ])),
        ("destinations", Json::obj(vec![
            ("directory", Json::str(format!("artifacts/verification-challenges/{identity}/<fresh-attempt>"))),
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
fn retain_finalization_result(
    result: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    let receipt = resolve(
        record_of(arr(result, "records")?, "change-finalization")?,
        roots,
    )?;
    let path = receipt
        .parent()
        .ok_or("finalized change directory missing")?
        .join("provenance/finalization-result.json");
    fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
    if path.exists() {
        if read(&path)? != *result {
            return Err("finalization receipt already holds a different actual result".into());
        }
    } else {
        write_atomic(&path, result)?;
    }
    Ok(())
}

fn retain_change_acceptance(
    result: &Json,
    handoff: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    let outcome_ref = handoff_inputs(handoff)?
        .into_iter()
        .find(|reference| reference.get("kind").and_then(Json::as_str) == Some("change-outcome"))
        .ok_or("change acceptance requires actual outcome")?;
    let outcome = resolve(&outcome_ref, roots)?;
    let change = outcome.parent().ok_or("outcome has no change directory")?;
    let manifest = crate::retained_inputs::verify_change(change)?;
    let snapshot = read(&resolve(
        record_of(arr(handoff, "inputs")?, "assurance-snapshot")?,
        roots,
    )?)?;
    validate_snapshot(&snapshot)?;
    snapshot_target(&snapshot, handoff.get("target").unwrap())?;
    let decision = result
        .get("decision")
        .ok_or("change acceptance has no actual decision")?;
    let mut record = Json::obj(vec![
        ("format", Json::str("azimuth-change-acceptance-decision")),
        ("version", Json::Num(1.0)),
        ("actor", result.get("actor").unwrap().clone()),
        ("decision", result.get("outcome").unwrap().clone()),
        (
            "reviewed_at_ms",
            Json::Num(crate::provenance::now_ms() as f64),
        ),
        (
            "candidate_model_fingerprint",
            snapshot
                .get("authority")
                .unwrap()
                .get("model_fingerprint")
                .unwrap()
                .clone(),
        ),
        (
            "outcome_fingerprint",
            Json::str(format!(
                "sha256:{}",
                crate::fingerprint::sha256(&fs::read(&outcome).map_err(|error| error.to_string())?)
            )),
        ),
        (
            "retained_inputs_fingerprint",
            manifest.get("fingerprint").unwrap().clone(),
        ),
        ("report", decision.get("rationale").unwrap().clone()),
    ]);
    let path = change.join("change-acceptance.json");
    if path.exists() {
        let previous = read(&path)?;
        let mut comparison = record.clone();
        set(
            &mut comparison,
            "reviewed_at_ms",
            previous
                .get("reviewed_at_ms")
                .ok_or("acceptance time missing")?
                .clone(),
        );
        if previous == comparison {
            return Ok(());
        }
        let history = change.join("acceptance-history");
        fs::create_dir_all(&history).map_err(|error| error.to_string())?;
        let retained = history.join(format!(
            "{}.json",
            digest(&previous).trim_start_matches("sha256:")
        ));
        if !retained.exists() {
            write_atomic(&retained, &previous)?;
        }
        record = record.clone();
    }
    write_atomic(&path, &record)
}

fn delta_facet(kind: &str) -> Option<&'static str> {
    match kind {
        "author-spec-delta" => Some("spec"),
        "author-design-delta" => Some("design"),
        "author-verification-delta" => Some("verification"),
        _ => None,
    }
}
fn required_result_kind<'a>(
    kind: &str,
    outcome: &str,
    required: &'a str,
    records: &[Json],
) -> &'a str {
    if outcome != "failed"
        && delta_facet(kind).is_some()
        && records.iter().any(|reference| {
            reference.get("kind").and_then(Json::as_str) == Some("facet-unchanged")
        })
    {
        "facet-unchanged"
    } else {
        required
    }
}
fn validate_unchanged_facet(
    result: &Json,
    handoff: &Json,
    kind: &str,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    let Some(facet) = delta_facet(kind) else {
        return Ok(());
    };
    let Some(reference) = arr(result, "records")?
        .iter()
        .find(|reference| reference.get("kind").and_then(Json::as_str) == Some("facet-unchanged"))
    else {
        return Ok(());
    };
    let value = supporting_record(reference, roots)?;
    obj(
        &value,
        &[
            "format",
            "version",
            "module",
            "facet",
            "baseline",
            "rationale",
        ],
    )?;
    if text(&value, "format")? != "azimuth-facet-unchanged"
        || number(&value, "version")? != 1
        || text(&value, "facet")? != facet
        || value.get("module")
            != handoff
                .get("target")
                .and_then(|target| target.get("module"))
    {
        return Err("unchanged facet declaration differs from assigned facet/module".into());
    }
    text(&value, "rationale")?;
    let baseline = value.get("baseline").unwrap();
    let mut fields = vec!["kind", "ref", "fingerprint"];
    if baseline.get("repository").is_some() {
        fields.push("repository");
        identifier(text(baseline, "repository")?)?;
    }
    obj(baseline, &fields)?;
    if text(baseline, "kind")? != "accepted-facet" {
        return Err("unchanged facet must pin an accepted facet".into());
    }
    portable(text(baseline, "ref")?)?;
    let baseline_root = roots
        .get(reference_repository(baseline))
        .ok_or("unknown accepted facet repository")?;
    let baseline_path = baseline_root.join(text(baseline, "ref")?);
    if baseline_path.file_name().and_then(|name| name.to_str())
        != Some(format!("{facet}.md").as_str())
    {
        return Err("unchanged facet must name its actual facet filename".into());
    }
    let module_spec = baseline_path
        .parent()
        .ok_or("accepted facet has no module directory")?
        .join("spec.md");
    let spec_relative = module_spec
        .strip_prefix(baseline_root)
        .map_err(|_| "accepted module escapes repository")?
        .to_str()
        .ok_or("accepted module path is not UTF-8")?;
    let spec_ref = Json::obj(vec![
        ("kind", Json::str("accepted-facet")),
        ("ref", Json::str(spec_relative)),
        ("repository", Json::str(reference_repository(baseline))),
    ]);
    let mut spec_ref = spec_ref;
    if reference_repository(baseline).is_empty() {
        if let Json::Obj(fields) = &mut spec_ref {
            fields.retain(|(key, _)| key != "repository");
        }
    }
    let spec = fs::read_to_string(resolve(&spec_ref, roots)?).map_err(|error| error.to_string())?;
    let expected = format!("# Spec: {}", text(&value, "module")?);
    if !spec.lines().any(|line| line.trim() == expected) {
        return Err("accepted module Spec identity differs from unchanged facet target".into());
    }
    if facet == "spec" && baseline.get("fingerprint") == Some(&Json::Null) {
        return Err("a new Spec requires an actual delta, not an unchanged assertion".into());
    }
    if baseline.get("fingerprint") != Some(&Json::Null) {
        check_ref(baseline, roots)?;
        let source =
            fs::read_to_string(resolve(baseline, roots)?).map_err(|error| error.to_string())?;
        let heading = format!(
            "# {}: {}",
            match facet {
                "spec" => "Spec",
                "design" => "Design",
                _ => "Verification",
            },
            text(&value, "module")?
        );
        if !source.lines().any(|line| line.trim() == heading) {
            return Err("accepted facet declaration identity differs from assigned module".into());
        }
    } else {
        portable(text(baseline, "ref")?)?;
        let root = roots
            .get(reference_repository(baseline))
            .ok_or("unknown unchanged facet repository")?;
        let mut path = root.clone();
        let components = Path::new(text(baseline, "ref")?)
            .components()
            .collect::<Vec<_>>();
        for (index, component) in components.iter().enumerate() {
            path.push(component.as_os_str());
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err("unchanged facet baseline traverses a symlink".into())
                }
                Ok(_) if index + 1 == components.len() => {
                    return Err("unchanged facet asserts absence of an existing path".into())
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    Ok(())
}

fn validate_completion_output(
    result: &Json,
    handoff: &Json,
    kind: &str,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    if kind == "author-change-outcome" {
        let path = resolve(record_of(arr(result, "records")?, "change-outcome")?, roots)?;
        if path.file_name().and_then(|name| name.to_str()) != Some("outcome.md") {
            return Err("change outcome must be the actual outcome.md".into());
        }
        let source = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        if source.trim().is_empty() {
            return Err("change outcome is empty".into());
        }
    }
    if kind != "finalize-change" {
        return Ok(());
    }
    let receipt_path = resolve(
        record_of(arr(result, "records")?, "change-finalization")?,
        roots,
    )?;
    let archived = receipt_path
        .parent()
        .ok_or("finalization has no owning change")?;
    if receipt_path.file_name().and_then(|name| name.to_str()) != Some("finalization.json")
        || archived
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            != Some("archive")
        || archived
            .parent()
            .and_then(Path::parent)
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            != Some("changes")
    {
        return Err("finalize-change requires the actual receipt in changes/archive/<dated-change>/finalization.json".into());
    }
    let receipt = read(&receipt_path)?;
    if number(&receipt, "version")? != 3
        || number(&receipt, "errors")? != 0
        || number(&receipt, "warnings")? != 0
    {
        return Err("finalization receipt is not a successful native version 3 receipt".into());
    }
    let snapshot = read(&resolve(
        record_of(arr(handoff, "inputs")?, "assurance-snapshot")?,
        roots,
    )?)?;
    validate_snapshot(&snapshot)?;
    snapshot_target(&snapshot, handoff.get("target").unwrap())?;
    let authority = snapshot.get("authority").unwrap();
    if receipt.get("authority_model_fingerprint") != authority.get("model_fingerprint") {
        return Err("finalization model differs from accepted candidate".into());
    }
    let expected_change = text(handoff.get("target").unwrap(), "change")?;
    if !archived
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(&format!("-{expected_change}")))
    {
        return Err("finalized archive belongs to another change".into());
    }
    crate::change::verify_change_acceptance(archived, text(authority, "model_fingerprint")?)?;
    let pinned_outcome = handoff_inputs(handoff)?
        .into_iter()
        .find(|reference| reference.get("kind").and_then(Json::as_str) == Some("change-outcome"))
        .ok_or("finalization has no pinned outcome")?;
    if pinned_outcome.get("fingerprint").and_then(Json::as_str)
        != Some(
            format!(
                "sha256:{}",
                crate::fingerprint::sha256(
                    &fs::read(archived.join("outcome.md")).map_err(|error| error.to_string())?
                )
            )
            .as_str(),
        )
    {
        return Err("archived outcome differs from the approved input".into());
    }
    crate::retained_inputs::verify_change(archived)?;
    let proposal =
        fs::read_to_string(archived.join("proposal.md")).map_err(|error| error.to_string())?;
    if !proposal
        .lines()
        .any(|line| line.trim() == "Status: accepted and complete")
    {
        return Err("archived change is not accepted and complete".into());
    }
    let outcome =
        fs::read_to_string(archived.join("outcome.md")).map_err(|error| error.to_string())?;
    if outcome.trim().is_empty() {
        return Err("archived outcome is empty".into());
    }
    let dependencies = arr(handoff, "dependencies")?;
    if !dependencies.iter().any(|dependency| {
        dependency.get("outcome").and_then(Json::as_str) == Some("approved")
            && dependency
                .get("decision")
                .and_then(|decision| decision.get("format"))
                .and_then(Json::as_str)
                == Some("azimuth-workflow-approval")
    }) {
        return Err("finalization requires recorded current human change acceptance".into());
    }
    let mut reviews = arr(&snapshot, "reviews")?.to_vec();
    let mut runs = snapshot_runs(&snapshot)?;
    for reference in handoff_inputs(handoff)? {
        match text(&reference, "kind")? {
            "requirement-review" => {
                for review in supporting_review_history(&reference, roots)? {
                    if !reviews.contains(&review) {
                        reviews.push(review);
                    }
                }
            }
            "run-bundle" => {
                let run = crate::run::load(&resolve(&reference, roots)?)
                    .map_err(|errors| format!("invalid completion Run: {errors:?}"))?;
                if !runs.contains(&run) {
                    runs.push(run);
                }
            }
            _ => {}
        }
    }
    for requirement in arr(authority, "requirements")?
        .iter()
        .filter(|requirement| {
            requirement.get("criticality").and_then(Json::as_str) != Some("routine")
        })
    {
        let target = Json::obj(vec![
            ("kind", Json::str("requirement-review")),
            ("requirement", requirement.get("id").unwrap().clone()),
        ]);
        let review = crate::assurance_review::latest_review(&reviews, &target)?
            .ok_or("completion Requirement Review missing")?;
        if crate::assurance_review::assess_record(review, authority, &reviews, &runs).status
            != "accepted"
        {
            return Err("completion Requirement Review is not current and accepted".into());
        }
    }
    let issues = crate::change::provenance_archive_issues(archived);
    if !issues.is_empty() {
        return Err(issues.join("\n"));
    }
    Ok(())
}

fn review_record(
    result: &Json,
    handoff: &Json,
    kind: &str,
    r: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    let records = arr(result, "records")?;
    validate_completion_output(result, handoff, kind, r)?;
    validate_unchanged_facet(result, handoff, kind, r)?;
    if matches!(
        kind,
        "request-implementation-approval" | "request-change-acceptance"
    ) {
        let approval = decision_record(
            result,
            handoff,
            if kind == "request-change-acceptance" {
                "change-acceptance"
            } else {
                "implementation-approval"
            },
            r,
        )?;
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
        crate::authoring_review::validate_issues(
            v.get("findings")
                .ok_or("authoring review findings missing")?,
            text(&v, "outcome")?,
        )?;
        if text(&v, "outcome")? == "rejected" && arr(&v, "findings")?.is_empty() {
            return Err("rejected review requires findings".into());
        }
        let expected = handoff_inputs(handoff)?;
        for dep in arr(handoff, "dependencies")? {
            if dep.get("actor") == result.get("actor")
                && arr(dep, "records")?.iter().any(|record| {
                    record.get("kind").and_then(Json::as_str) == Some("facet-unchanged")
                        || (kind == "review-proposal"
                            && record.get("kind").and_then(Json::as_str)
                                == Some("proposal-content"))
                        || record.get("kind").and_then(Json::as_str)
                            == Some(match kind {
                                "review-proposal" => "proposal",
                                "review-spec-delta" => "intent-delta",
                                "review-design" => "requirement-design",
                                "review-design-delta" => "design-delta",
                                "review-verification-delta" => "verification-delta",
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
        "review-requirement",
    ]
    .contains(&kind)
    {
        let expected = match kind {
            "qualify-method" => "method-qualification",
            "assess-applicability" => "applicability-decision",
            _ => "requirement-review",
        };
        let review = supporting_record(record_of(records, expected)?, r)?;
        crate::assurance_review::validate_record(&review)?;
        for key in if kind == "qualify-method" {
            vec!["check"]
        } else if kind == "assess-applicability" {
            vec!["check", "scenario"]
        } else {
            vec!["requirement"]
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
        if ["qualify-method", "review-requirement"].contains(&kind) {
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
        if kind == "review-requirement" {
            for dependency in arr(handoff, "dependencies")? {
                for reference in arr(dependency, "records")? {
                    if text(reference, "kind")? == "requirement-review" {
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
        if let Some(verification_target) = handoff
            .get("target")
            .and_then(|v| v.get("verification_target"))
        {
            if review.get("verification_target_fingerprint") != Some(verification_target) {
                return Err("review Verification Target differs from task target".into());
            }
        }
        let assessment = crate::assurance_review::assess_record(&review, authority, prior, &runs);
        if assessment.status != text(result, "outcome")? {
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
            source
                .get("issues")
                .cloned()
                .unwrap_or_else(|| Json::Arr(vec![])),
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
    if !["review-requirement", "run-verification-challenge"].contains(&text(task, "kind")?) {
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
    for optional in ["target", "report", "issues", "dispositions"] {
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
    if handoffs.get("format").and_then(Json::as_str) == Some("azimuth-workflow-handoff")
        && handoffs.get("version").and_then(Json::as_num) == Some(2.0)
    {
        return Err("--handoff is a version 2 handoff without captured_at_ms; capture a fresh handoff with `workflow next` before performing the work".into());
    }
    if handoffs.get("format").and_then(Json::as_str) != Some("azimuth-workflow-handoff")
        || number(handoffs, "version")? != 3
        || handoffs.get("instance") != state.get("id")
    {
        return Err("--handoff requires this instance's workflow next output".into());
    }
    number(handoffs, "captured_at_ms")?;
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
    if text(handoff, "task")? == "implement-change" {
        let required = handoff
            .get("corrections")
            .and_then(Json::as_array)
            .unwrap_or(&[])
            .iter()
            .flat_map(|correction| {
                correction
                    .get("issue_references")
                    .and_then(Json::as_array)
                    .unwrap_or(&[])
            })
            .filter_map(Json::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if !required.is_empty() {
            let dispositions = source
                .get("dispositions")
                .ok_or("correction requires a disposition for every captured stable issue")?;
            crate::authoring_review::validate_dispositions(dispositions, &required)?;
            if text(source, "outcome")? == "completed"
                && !crate::authoring_review::dispositions_resolved(dispositions)
            {
                return Err(
                    "unresolved correction dispositions cannot complete implementation".into(),
                );
            }
        }
    }
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
    let required = required_result_kind(
        text(handoff, "task")?,
        text(source, "outcome")?,
        required,
        &records,
    );
    let uses_report = required == "failure-report"
        || required == "implementation-result"
        || required == "change-acceptance"
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
        ("version", Json::Num(3.0)),
        ("id", Json::str(format!("attempt-{}", &identity[7..]))),
        ("task", handoff.get("id").unwrap().clone()),
        ("basis", handoff.get("basis").unwrap().clone()),
        ("actor", Json::str(actor)),
        ("outcome", source.get("outcome").unwrap().clone()),
        ("records", Json::Arr(records)),
        ("inputs", Json::Arr(arr(handoff, "inputs")?.to_vec())),
    ]);
    if text(handoff, "task")? == "execute-check" && text(source, "outcome")? == "completed" {
        let assessment = check_assessment(&result, handoff.get("target").unwrap(), roots)?;
        set(&mut result, "check_assessment", assessment);
    }
    if text(handoff, "task")? == "run-verification-challenge"
        && (text(source, "outcome")? == "completed"
            || arr(&result, "records")?
                .iter()
                .any(|r| r.get("kind").and_then(Json::as_str) == Some("run-bundle")))
    {
        text(source, "report")?;
        record_of(arr(&result, "records")?, "verification-challenge-report")?;
        let assessment = challenge_assessment(&result, handoff.get("target").unwrap(), roots)?;
        set(&mut result, "challenge_assessment", assessment);
    }
    if text(source, "outcome")? != "failed"
        && (review_kind(text(handoff, "task")?)
            || matches!(
                text(handoff, "task")?,
                "request-implementation-approval" | "request-change-acceptance"
            ))
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
        .ok_or("workflow requires init, next, update, verify or abandon")?;
    let allowed = match operation.as_str() {
        "init" => vec![
            "--definition",
            "--workflow",
            "--inputs",
            "--instance",
            "--change",
            "--repositories",
            "--project",
            "--predecessor",
        ],
        "assurance-targets" => vec![
            "--instance",
            "--change",
            "--repositories",
            "--project",
            "--snapshot",
            "--verification-targets",
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
            "--descriptor",
        ],
        "abandon" => vec!["--instance", "--change", "--expected-revision", "--reason"],
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
        let predecessor = match get("--predecessor") {
            Some(file) => {
                crate::provenance::predecessor_record(Path::new(file), r.get("").unwrap())?
            }
            None => Json::Null,
        };
        // Generations of one change share a definition, so the instance id carries its creation
        // time to stay distinct in the Work Provenance ledger and in predecessor links.
        let id = format!("{}-{}", text(&def, "id")?, crate::provenance::now_ms());
        let state = Json::obj(vec![
            ("format", Json::str("azimuth-workflow-instance")),
            ("version", Json::Num(3.0)),
            ("id", Json::Str(id)),
            ("revision", Json::Num(0.0)),
            ("definition_fingerprint", Json::str(digest(&def))),
            ("definition", def),
            ("history", Json::Arr(vec![])),
            ("predecessor", predecessor),
            ("abandoned", Json::Null),
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
    if operation == "abandon" {
        let _lock = lock(&path)?;
        let state = read(&path)?;
        instance(&state).map_err(|e| format!("{}: line 1: {e}", path.display()))?;
        return abandon(
            state,
            &path,
            get("--expected-revision").ok_or("abandon requires --expected-revision")?,
            get("--reason").ok_or("abandon requires --reason TEXT")?,
        );
    }
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
        let selection = get("--verification-targets").map(|value| {
            Json::obj(vec![
                ("kind", Json::str("assurance-verification-target-selection")),
                ("ref", Json::str(value)),
            ])
        });
        return assurance_definition(&state, &snapshot, selection.as_ref(), &r);
    }
    if operation == "update" {
        if state.get("abandoned") != Some(&Json::Null) {
            return Err(format!(
                "{}: the workflow instance is abandoned and accepts no further updates",
                path.display()
            ));
        }
        let result_path = Path::new(get("--result").ok_or("update requires --result")?);
        let authored = read(result_path)?;
        let handoff_path = Path::new(
            get("--handoff")
                .ok_or("update requires --handoff FILE captured before performing the work")?,
        );
        let handoffs = read(handoff_path)?;
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
        let descriptor = match get("--descriptor") {
            Some(file) => {
                let value = crate::provenance::read_json(Path::new(file))?;
                crate::provenance::validate_descriptor(&value)
                    .map_err(|e| format!("{file}: {e}"))?;
                Some(value)
            }
            None => crate::provenance::captured_descriptor(
                handoff_path,
                number(&handoffs, "captured_at_ms")?,
            )?,
        };
        // Recording consumes a hook's descriptor. One left behind is ignored once a later
        // handoff is captured, so a failed removal cannot misattribute another attempt.
        let consume_captured = || {
            let _ = fs::remove_file(crate::provenance::captured_descriptor_path(handoff_path));
        };
        let captured_guidance = arr(&handoffs, "eligible")?
            .iter()
            .find(|handoff| handoff.get("id") == result.get("task"))
            .and_then(|handoff| handoff.get("guidance"))
            .and_then(Json::as_array)
            .unwrap_or(&[])
            .to_vec();
        let entry = Json::obj(vec![
            ("result_fingerprint", Json::str(digest(&result))),
            ("result", result.clone()),
            (
                "provenance",
                crate::provenance::history_provenance(
                    number(&handoffs, "captured_at_ms")?,
                    crate::provenance::now_ms(),
                    descriptor.as_ref(),
                    &captured_guidance,
                )?,
            ),
        ]);
        for (position, old) in arr(&state, "history")?.iter().enumerate() {
            if old.get("result").unwrap().get("id") == result.get("id") {
                if old.get("result") != Some(&result) {
                    return Err("result ID reused with different content".into());
                }
                if authored.get("work").and_then(Json::as_str) == Some("finalize-change")
                    && result.get("outcome").and_then(Json::as_str) == Some("completed")
                {
                    retain_finalization_result(&result, &r)?;
                }
                consume_captured();
                return Ok(Json::obj(vec![
                    ("revision", state.get("revision").unwrap().clone()),
                    ("result", result),
                    ("provenance", old.get("provenance").unwrap().clone()),
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
        let valid = if matches!(
            kind,
            "request-implementation-approval" | "request-change-acceptance"
        ) {
            ["approved", "denied", "deferred"].contains(&outcome)
        } else if review_kind(kind)
            || [
                "qualify-method",
                "assess-applicability",
                "review-requirement",
            ]
            .contains(&kind)
        {
            ["accepted", "rejected"].contains(&outcome)
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
        let required = required_result_kind(kind, outcome, required, arr(&result, "records")?);
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
            "run-verification-challenges",
            "run-verification-challenge",
        ]
        .contains(&kind)
            && (outcome == "completed"
                || kind == "run-verification-challenge"
                    && arr(&result, "records")?
                        .iter()
                        .any(|r| r.get("kind").and_then(Json::as_str) == Some("run-bundle")))
        {
            let reference = record_of(arr(&result, "records")?, "run-bundle")?;
            let path = resolve(reference, &r)?;
            let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let bundle = crate::run::parse(&path.display().to_string(), &source)
                .map_err(|e| format!("invalid Run: {e:?}"))?;
            if ["run-verification-challenges", "run-verification-challenge"].contains(&kind)
                && bundle.verification_challenge_executions.is_empty()
            {
                return Err(
                    "Verification Challenge execution requires recorded Challenge Results".into(),
                );
            }
            if ["run-verification-challenges", "run-verification-challenge"].contains(&kind) {
                let snapshot = read(&resolve(
                    record_of(arr(handoff, "inputs")?, "assurance-snapshot")?,
                    &r,
                )?)?;
                validate_snapshot(&snapshot)?;
                let authority = snapshot.get("authority").unwrap();
                let mut reviews = arr(&snapshot, "reviews")?.to_vec();
                for dependency in arr(handoff, "dependencies")? {
                    for reference in arr(dependency, "records")? {
                        if text(reference, "kind")? == "requirement-review" {
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
                for execution in &bundle.verification_challenge_executions {
                    let declarations =
                        arr(authority, "requirements")?
                            .iter()
                            .flat_map(|requirement| {
                                requirement
                                    .get("verification_challenges")
                                    .and_then(Json::as_array)
                                    .unwrap_or(&[])
                            });
                    if !declarations.into_iter().any(|verification_challenge| {
                        verification_challenge.get("id").and_then(Json::as_str)
                            == Some(execution.verification_challenge.id.as_str())
                            && verification_challenge
                                .get("fingerprint")
                                .and_then(Json::as_str)
                                == Some(execution.verification_challenge.fingerprint.as_str())
                            && verification_challenge
                                .get("requirements")
                                .and_then(Json::as_array)
                                .is_some_and(|requirements| {
                                    requirements.iter().any(|requirement| {
                                        execution.target.kind
                                            == crate::run::ChallengeTargetKind::Implementation
                                            || requirement.as_str()
                                                == Some(execution.target.id.as_str())
                                    })
                                })
                    }) {
                        return Err("Challenge does not match the current declared Verification Challenge and Requirement scope".into());
                    }
                    if execution.target.kind == crate::run::ChallengeTargetKind::Implementation {
                        if !crate::assurance_review::implementation_challenge_current(
                            authority, &bundle, execution,
                        )? {
                            return Err("Challenge implementation basis is stale".into());
                        }
                        continue;
                    }
                    if execution.target.kind != crate::run::ChallengeTargetKind::RequirementReview {
                        return Err("New workflow Challenges must target Requirement Review".into());
                    }
                    let review = reviews
                        .iter()
                        .find(|review| {
                            review.get("fingerprint").and_then(Json::as_str)
                                == Some(execution.target.fingerprint.as_str())
                        })
                        .ok_or("Challenge target assessment missing from retained inputs")?;
                    if review.get("requirement").and_then(Json::as_str)
                        != Some(execution.target.id.as_str())
                        || review.get("kind").and_then(Json::as_str)
                            != Some(execution.target.kind.name())
                    {
                        return Err(
                            "Challenge target kind or Requirement differs from retained assessment"
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
            if let Some(verification_target) = handoff
                .get("target")
                .and_then(|v| v.get("verification_target"))
                .and_then(Json::as_str)
            {
                if bundle.verification_target_fingerprint != verification_target {
                    return Err("Run Verification Target differs from task target".into());
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
                    r.get("kind").and_then(Json::as_str)
                        == Some("assurance-verification-target-selection")
                })) {
            let snapshot = records.iter().find(|r| r.get("kind").and_then(Json::as_str) == Some("assurance-snapshot"))
                .cloned().or_else(|| state.get("definition").unwrap().get("parameters")?.get("assurance")?.get("snapshot").cloned())
                .ok_or("candidate preparation requires an assurance-snapshot output for automatic target derivation")?;
            let selection = records.iter().find(|r| {
                r.get("kind").and_then(Json::as_str)
                    == Some("assurance-verification-target-selection")
            });
            let mut prepared = state.clone();
            if let Json::Obj(fields) = &mut prepared {
                if let Some((_, Json::Arr(history))) =
                    fields.iter_mut().find(|(key, _)| key == "history")
                {
                    history.push(entry.clone());
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
                        values.push(entry.clone());
                    }
                }
            }
        }
        if kind == "request-change-acceptance" {
            retain_change_acceptance(&result, handoff, &r)?;
        }
        if kind == "finalize-change" && outcome == "completed" {
            retain_finalization_result(&result, &r)?;
        }
        write_atomic(&path, &state)?;
        consume_captured();
        return Ok(Json::obj(vec![
            ("revision", state.get("revision").unwrap().clone()),
            ("result", result),
            ("provenance", entry.get("provenance").unwrap().clone()),
            ("idempotent", Json::Bool(false)),
            ("applied_revision", Json::Num((expected + 1) as f64)),
        ]));
    }
    let evaluation = evaluate(&state, &r)?;
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-workflow-handoff")),
        ("version", Json::Num(3.0)),
        (
            "captured_at_ms",
            Json::Num(crate::provenance::now_ms() as f64),
        ),
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

fn abandon(mut state: Json, path: &Path, expected: &str, reason: &str) -> Result<Json, String> {
    let expected = expected
        .parse::<u64>()
        .map_err(|_| "invalid expected revision")?;
    if expected != number(&state, "revision")? {
        return Err("workflow revision conflict".into());
    }
    if state.get("abandoned") != Some(&Json::Null) {
        return Err(format!(
            "{}: the workflow instance is already abandoned",
            path.display()
        ));
    }
    if reason.trim().is_empty() {
        return Err("abandon requires a nonempty --reason".into());
    }
    let abandoned = Json::obj(vec![
        ("at_ms", Json::Num(crate::provenance::now_ms() as f64)),
        ("reason", Json::str(reason)),
    ]);
    set(&mut state, "abandoned", abandoned.clone());
    write_atomic(path, &state)?;
    Ok(Json::obj(vec![
        ("instance", state.get("id").unwrap().clone()),
        ("revision", state.get("revision").unwrap().clone()),
        ("abandoned", abandoned),
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
    if args.first().is_some_and(|operation| operation == "abandon") {
        if canonical(&path)? == output {
            return Err("output aliases workflow instance".into());
        }
        return Ok(());
    }
    for flag in [
        "--handoff",
        "--result",
        "--definition",
        "--inputs",
        "--descriptor",
        "--predecessor",
    ] {
        if let Some(input) = value(flag) {
            if canonical(&input)? == output {
                return Err(format!("output aliases {flag} input"));
            }
        }
    }
    for flag in ["--snapshot", "--verification-targets"] {
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
        "request-implementation-approval" | "request-change-acceptance" => {
            &["approved", "denied", "deferred"]
        }
        "review-requirement"
        | "qualify-method"
        | "assess-applicability"
        | "review-proposal"
        | "review-spec-delta"
        | "review-design"
        | "review-verification"
        | "review-design-delta"
        | "review-verification-delta" => &["accepted", "rejected"],
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
        || number(v, "version")? != 2
        || number(v, "revision")? == 0
    {
        return Err("expected workflow template version 2 with positive revision".into());
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
        if !seen.insert(text(action, "id")?) {
            return Err("duplicate action ID".into());
        }
        registered_kind(text(action, "kind")?)?;
        if ![
            "change",
            "modules",
            "methods",
            "requirement-definitions",
            "verification-challenges",
            "checks",
            "contributions",
            "requirements",
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
            "review-requirement",
            "run-verification-challenge",
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
                    "same-verification-target",
                    "requirement-contributions",
                    "same-requirement",
                    "verification-challenge-requirements",
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
                "same-verification-target" => {
                    ["checks", "contributions", "requirements"].contains(&collection)
                        && ["checks", "contributions", "requirements"].contains(&other)
                }
                "requirement-contributions" => {
                    collection == "requirements" && other == "contributions"
                }
                "verification-challenge-requirements" => {
                    collection == "verification-challenges" && other == "requirement-definitions"
                }
                "same-requirement" => {
                    ["requirements", "requirement-definitions"].contains(&collection)
                        && ["requirements", "requirement-definitions"].contains(&other)
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
        "requirements",
    ];
    for optional in [
        "methods",
        "requirement_definitions",
        "verification_challenges",
        "verification_challenge_requirements",
        "pending_verification_targets",
        "selection",
        "verification_target_inputs",
    ] {
        if v.get(optional).is_some() {
            fields.push(optional);
        }
    }
    obj(v, &fields)?;
    if let Some(mapping) = v.get("verification_challenge_requirements") {
        let mut ids = BTreeSet::new();
        for entry in mapping
            .as_array()
            .ok_or("Verification Challenge Requirement mapping requires array")?
        {
            obj(entry, &["verification_challenge", "requirements"])?;
            identifier(text(entry, "verification_challenge")?)?;
            if !ids.insert(text(entry, "verification_challenge")?)
                || arr(entry, "requirements")?.is_empty()
            {
                return Err(
                    "Verification Challenge mapping requires unique identities and nonempty Requirement scope".into(),
                );
            }
            for requirement in arr(entry, "requirements")? {
                identifier(
                    requirement
                        .as_str()
                        .ok_or("Verification Challenge Requirement requires identifier")?,
                )?;
            }
        }
    }
    if v.get("verification_challenges").is_some() {
        let mut seen = BTreeSet::new();
        for verification_challenge in arr(v, "verification_challenges")? {
            let verification_challenge = verification_challenge
                .as_str()
                .ok_or("Verification Challenge ID requires string")?;
            identifier(verification_challenge)?;
            if !seen.insert(verification_challenge) {
                return Err("duplicate inventory Verification Challenge".into());
            }
        }
    }
    if v.get("requirement_definitions").is_some() {
        let mut seen = BTreeSet::new();
        for target in arr(v, "requirement_definitions")? {
            obj(target, &["requirement"])?;
            identifier(text(target, "requirement")?)?;
            if !seen.insert(text(target, "requirement")?) {
                return Err("duplicate Requirement definition target".into());
            }
        }
    }
    if v.get("verification_target_inputs").is_some() {
        for input in arr(v, "verification_target_inputs")? {
            obj(input, &["verification_target", "input"])?;
            fingerprint(text(input, "verification_target")?)?;
            ref_shape(input.get("input").unwrap(), true)?;
        }
    }
    if let Some(selection) = v.get("selection") {
        ref_shape(selection, false)?;
    }
    for field in ["methods", "pending_verification_targets"] {
        if v.get(field).is_some() {
            let mut seen = BTreeSet::new();
            for target in arr(v, field)? {
                if field == "methods" {
                    obj(target, &["check"])?;
                    identifier(text(target, "check")?)?;
                } else {
                    obj(target, &["kind", "id"])?;
                    identifier(text(target, "id")?)?;
                    if !["check", "requirement"].contains(&text(target, "kind")?) {
                        return Err(
                            "pending Verification Target target requires check or requirement kind"
                                .into(),
                        );
                    }
                }
                if !seen.insert(digest(target)) {
                    return Err("duplicate derived assurance target".into());
                }
            }
        }
    }
    if text(v, "format")? != "azimuth-assurance-target-inventory" || number(v, "version")? != 2 {
        return Err("expected assurance target inventory version 2".into());
    }
    identifier(text(v, "project")?)?;
    fingerprint(text(v, "model_revision")?)?;
    ref_shape(v.get("snapshot").unwrap(), false)?;
    if text(v.get("snapshot").unwrap(), "kind")? != "assurance-snapshot" {
        return Err("inventory snapshot requires assurance-snapshot kind".into());
    }
    for (collection, fields) in [
        ("checks", vec!["check", "verification_target"]),
        (
            "contributions",
            vec!["check", "scenario", "requirement", "verification_target"],
        ),
        ("requirements", vec!["requirement", "verification_target"]),
    ] {
        let mut seen = BTreeSet::new();
        for target in arr(v, collection)? {
            obj(target, &fields)?;
            for field in &fields {
                if *field == "verification_target" {
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
        if text(action, "kind")? == "run-verification-challenges"
            && bindings
                .get("assurance")
                .and_then(|inventory| inventory.get("verification_challenges"))
                .and_then(Json::as_array)
                .is_some_and(|verification_challenges| verification_challenges.is_empty())
        {
            by_action.insert(text(action, "id")?.into(), Vec::new());
            continue;
        }
        if collection == "change"
            && bindings
                .get("assurance")
                .and_then(|v| v.get("pending_verification_targets"))
                .and_then(Json::as_array)
                .is_some_and(|v| !v.is_empty())
            && arr(action, "depends_on")?.iter().any(|dep| {
                arr(source, "actions").unwrap().iter().any(|a| {
                    a.get("id") == dep.get("action")
                        && matches!(text(a, "collection").ok(), Some("requirements" | "checks"))
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
            "requirement-definitions" => {
                if bindings.get("assurance") == Some(&Json::Null) {
                    vec![]
                } else {
                    arr(
                        bindings.get("assurance").unwrap(),
                        "requirement_definitions",
                    )?
                    .to_vec()
                }
            }
            "verification-challenges" => {
                if bindings.get("assurance") == Some(&Json::Null) {
                    vec![]
                } else {
                    arr(
                        bindings.get("assurance").unwrap(),
                        "verification_challenges",
                    )?
                    .iter()
                    .map(|id| Json::obj(vec![("verification_challenge", id.clone())]))
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
                                    && (other.get("verification_target").is_none()
                                        || target.get("verification_target")
                                            == other.get("verification_target"))
                            }
                            "same-verification-target" => {
                                target.get("verification_target")
                                    == other.get("verification_target")
                            }
                            "same-requirement" => {
                                target.get("requirement") == other.get("requirement")
                            }
                            "verification-challenge-requirements" => {
                                verification_challenge_requirements(bindings, &target).is_ok_and(
                                    |requirements| {
                                        other.get("requirement").is_some_and(|requirement| {
                                            requirements.contains(requirement)
                                        })
                                    },
                                )
                            }
                            "requirement-contributions" => {
                                target.get("requirement") == other.get("requirement")
                                    && target.get("verification_target")
                                        == other.get("verification_target")
                            }
                            _ => true,
                        }
                    })
                    .collect::<Vec<_>>();
                if matched.is_empty() {
                    let complete_empty_verification_challenges = text(dependency, "match")?
                        == "all"
                        && arr(source, "actions")?.iter().any(|action| {
                            action.get("id") == dependency.get("action")
                                && matches!(
                                    text(action, "kind").ok(),
                                    Some(
                                        "run-verification-challenges"
                                            | "run-verification-challenge"
                                    )
                                )
                        })
                        && bindings
                            .get("assurance")
                            .and_then(|inventory| inventory.get("verification_challenges"))
                            .and_then(Json::as_array)
                            .is_some_and(|verification_challenges| {
                                verification_challenges.is_empty()
                            });
                    let complete_empty_reviews = text(dependency, "match")? == "all"
                        && arr(source, "actions")?.iter().any(|action| {
                            action.get("id") == dependency.get("action")
                                && text(action, "collection").ok()
                                    == Some("requirement-definitions")
                        })
                        && bindings
                            .get("assurance")
                            .and_then(|inventory| inventory.get("requirement_definitions"))
                            .and_then(Json::as_array)
                            .is_some_and(|requirements| requirements.is_empty());
                    let complete_empty_requirements = bindings
                        .get("assurance")
                        .filter(|v| *v != &Json::Null)
                        .is_some_and(|v| {
                            arr(v, "requirements").is_ok_and(|requirements| requirements.is_empty())
                        })
                        && text(dependency, "match")? == "all"
                        && arr(source, "actions")?.iter().any(|a| {
                            text(a, "id").ok() == Some(id)
                                && text(a, "collection").ok() == Some("requirements")
                        });
                    let complete_empty_checks = text(dependency, "match")? == "all"
                        && arr(source, "actions")?.iter().any(|action| {
                            action.get("id") == dependency.get("action")
                                && text(action, "collection").ok() == Some("checks")
                        })
                        && bindings
                            .get("assurance")
                            .and_then(|inventory| inventory.get("methods"))
                            .and_then(Json::as_array)
                            .is_some_and(|methods| methods.is_empty());
                    if !complete_empty_checks
                        && !complete_empty_requirements
                        && !complete_empty_verification_challenges
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
            if target.get("verification_target").is_some() {
                if let Some(verification_target_inputs) = bindings
                    .get("assurance")
                    .and_then(|v| v.get("verification_target_inputs"))
                    .and_then(Json::as_array)
                {
                    for selected in verification_target_inputs.iter().filter(|v| {
                        v.get("verification_target") == target.get("verification_target")
                    }) {
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
        ("version", Json::Num(2.0)),
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
        .and_then(|v| v.get("pending_verification_targets"))
        .and_then(Json::as_array)
        .unwrap_or(&[]);
    Ok(Json::Arr(pending.iter().map(|target| Json::obj(vec![
        ("kind", Json::str("assurance-verification-target-selection")),
        ("target", target.clone()),
        ("reason", Json::str("Select the exact source or artifact execution Verification Target for this Check. Deployment-only Checks execute after rollout; model equality alone does not transfer evidence.")),
    ])).collect()))
}

fn local_check(check: &Json) -> bool {
    check
        .get("deployment")
        .and_then(|policy| policy.get("verification_target"))
        .and_then(Json::as_str)
        != Some("deployment")
}

fn portable_assurance_reference(
    reference: &Json,
    locations: &BTreeMap<String, PathBuf>,
) -> Result<Json, String> {
    if reference.get("fingerprint").is_some() {
        check_ref(reference, locations)?;
    } else {
        ref_shape(reference, false)?;
    }
    let mut portable = reference.clone();
    if let Json::Obj(fields) = &mut portable {
        fields.retain(|(key, _)| key != "fingerprint");
    }
    Ok(portable)
}

fn affected_execution_selection(
    selection: Option<&Json>,
    authority: &Json,
    roots: &BTreeMap<String, PathBuf>,
) -> Result<Option<Json>, String> {
    let Some(selection) = selection else {
        return Ok(None);
    };
    let selection = read(&resolve(selection, roots)?)?;
    let Some(reference) = selection.get("affected") else {
        return Ok(None);
    };
    ref_shape(reference, false)?;
    if text(reference, "kind")? != "affected-execution-request" {
        return Err("affected selection requires an affected-execution-request reference".into());
    }
    let request = read(&resolve(reference, roots)?)?;
    let report = crate::execution_impact::plan_affected(&request)?;
    if report.get("status").and_then(Json::as_str) != Some("complete") {
        return Err("affected execution selection is incomplete; resolve its diagnostics before narrowing workflow obligations".into());
    }
    let candidate = request
        .get("candidate")
        .ok_or("affected candidate missing")?;
    let checks = arr(candidate, "checks")?;
    if checks.len() != arr(authority, "checks")?.len()
        || arr(authority, "checks")?.iter().any(|check| {
            !checks.iter().any(|action| {
                action.get("id") == check.get("id")
                    && action.get("definition_fingerprint") == check.get("fingerprint")
            })
        })
    {
        return Err("affected Check inventory differs from current complete authority".into());
    }
    let mut declarations = BTreeMap::new();
    for requirement in arr(authority, "requirements")? {
        for declaration in arr(requirement, "verification_challenges")? {
            let id = text(declaration, "id")?.to_string();
            if declarations
                .insert(id, declaration)
                .is_some_and(|previous| previous != declaration)
            {
                return Err("conflicting current Challenge declaration".into());
            }
        }
    }
    let challenges = arr(candidate, "challenges")?;
    if challenges.len() != declarations.len()
        || declarations.iter().any(|(id, declaration)| {
            !challenges.iter().any(|action| {
                action.get("id").and_then(Json::as_str) == Some(id.as_str())
                    && action.get("definition_fingerprint") == declaration.get("fingerprint")
            })
        })
    {
        return Err("affected Challenge inventory differs from current complete authority".into());
    }
    Ok(Some(report))
}
fn execution_selected(report: Option<&Json>, collection: &str, id: &str) -> bool {
    report.is_none_or(|report| {
        report
            .get("selected")
            .and_then(|selected| selected.get(collection))
            .and_then(Json::as_array)
            .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(id)))
    })
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
    let snapshot_ref = portable_assurance_reference(snapshot_ref, locations)?;
    let snapshot = read(&resolve(&snapshot_ref, locations)?)?;
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
    let affected_selection = affected_execution_selection(selection_ref, authority, locations)?;
    let mut checks = Vec::new();
    let mut verification_target_inputs = Vec::new();
    if let Some(reference) = selection_ref {
        portable_assurance_reference(reference, locations)?;
        if text(reference, "kind")? != "assurance-verification-target-selection" {
            return Err("expected assurance-verification-target-selection reference".into());
        }
        let selection = read(&resolve(reference, locations)?)?;
        let mut selection_fields = vec![
            "format",
            "version",
            "project",
            "model_revision",
            "checks",
            "requirements",
        ];
        if selection.get("affected").is_some() {
            selection_fields.push("affected");
        }
        obj(&selection, &selection_fields)?;
        if text(&selection, "format")? != "azimuth-assurance-verification-target-selection"
            || number(&selection, "version")? != 2
        {
            return Err("expected assurance Verification Target selection version 2".into());
        }
        if selection.get("project") != snapshot.get("project")
            || selection.get("model_revision") != snapshot.get("model_revision")
        {
            return Err(
                "Verification Target selection differs from candidate project/revision".into(),
            );
        }
        if !arr(&selection, "requirements")?.is_empty() {
            return Err("Requirement Verification Target selections are not supported: Requirement Status is never authored, and deployment Requirement Status is computed from applicable evidence; leave requirements empty".into());
        }
        {
            let mut seen = BTreeSet::new();
            for target in arr(&selection, "checks")? {
                obj(target, &["check", "verification_target"])?;
                identifier(text(target, "check")?)?;
                if !arr(authority, "checks")?
                    .iter()
                    .any(|entity| entity.get("id") == target.get("check"))
                {
                    return Err("selected target absent from candidate authority".into());
                }
                let reference = target.get("verification_target").unwrap();
                ref_shape(reference, false)?;
                if text(reference, "kind")? != "verification-target" {
                    return Err(
                        "selected Verification Target requires verification target reference kind"
                            .into(),
                    );
                }
                let value = read(&resolve(reference, locations)?)?;
                let verification_target = crate::run::verification_target_from_json(&value)?;
                if !crate::run::validate_verification_target_component(&verification_target)
                    .is_empty()
                {
                    return Err(
                        "selected Verification Target fails native protocol validation".into(),
                    );
                }
                let input = Json::obj(vec![
                    (
                        "verification_target",
                        Json::str(crate::run::verification_target_fingerprint(
                            &verification_target,
                        )),
                    ),
                    ("input", materialize(reference, locations)?),
                ]);
                if !verification_target_inputs.contains(&input) {
                    verification_target_inputs.push(input);
                }
                let target = Json::obj(vec![
                    ("check", target.get("check").unwrap().clone()),
                    (
                        "verification_target",
                        Json::str(crate::run::verification_target_fingerprint(
                            &verification_target,
                        )),
                    ),
                ]);
                if !seen.insert(digest(&target)) {
                    return Err("duplicate selected target/Verification Target".into());
                }
                checks.push(target);
            }
        }
    }
    checks.retain(|target| {
        arr(authority, "checks").unwrap().iter().any(|check| {
            check.get("id") == target.get("check")
                && local_check(check)
                && execution_selected(
                    affected_selection.as_ref(),
                    "checks",
                    check.get("id").and_then(Json::as_str).unwrap_or_default(),
                )
        })
    });
    let methods = arr(authority, "checks")?
        .iter()
        .filter(|check| {
            local_check(check)
                && execution_selected(
                    affected_selection.as_ref(),
                    "checks",
                    check.get("id").and_then(Json::as_str).unwrap_or_default(),
                )
        })
        .map(|check| Json::obj(vec![("check", check.get("id").unwrap().clone())]))
        .collect::<Vec<_>>();
    let mut contributions = Vec::new();
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
    for requirement in arr(authority, "requirements")? {
        if text(requirement, "criticality")? == "routine" {
            continue;
        }
        for binding in arr(requirement, "bindings")? {
            for check in checks
                .iter()
                .filter(|check| check.get("check") == binding.get("check"))
            {
                contributions.push(Json::obj(vec![
                    ("check", binding.get("check").unwrap().clone()),
                    ("scenario", binding.get("scenario").unwrap().clone()),
                    ("requirement", requirement.get("id").unwrap().clone()),
                    (
                        "verification_target",
                        check.get("verification_target").unwrap().clone(),
                    ),
                ]));
            }
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
        ("version", Json::Num(2.0)),
        ("project", snapshot.get("project").unwrap().clone()),
        (
            "model_revision",
            snapshot.get("model_revision").unwrap().clone(),
        ),
        ("snapshot", snapshot_ref),
        ("methods", Json::Arr(methods)),
        (
            "requirement_definitions",
            Json::Arr(
                arr(authority, "requirements")?
                    .iter()
                    .filter(|requirement| text(requirement, "criticality").ok() != Some("routine"))
                    .map(|requirement| {
                        Json::obj(vec![(
                            "requirement",
                            requirement.get("id").unwrap().clone(),
                        )])
                    })
                    .collect(),
            ),
        ),
        ("checks", Json::Arr(checks)),
        ("contributions", Json::Arr(contributions)),
        ("requirements", Json::Arr(Vec::new())),
        ("pending_verification_targets", Json::Arr(pending)),
        (
            "verification_target_inputs",
            Json::Arr(verification_target_inputs),
        ),
    ]);
    if arr(authority, "requirements")?
        .iter()
        .all(|requirement| requirement.get("verification_challenges").is_some())
    {
        let verification_challenges = arr(authority, "requirements")?
            .iter()
            .flat_map(|requirement| arr(requirement, "verification_challenges").unwrap())
            .map(|verification_challenge| text(verification_challenge, "id").unwrap().to_string())
            .filter(|id| execution_selected(affected_selection.as_ref(), "challenges", id))
            .collect::<BTreeSet<_>>();
        set(
            &mut inventory,
            "verification_challenges",
            Json::Arr(verification_challenges.iter().map(Json::str).collect()),
        );
    }
    set(
        &mut inventory,
        "verification_challenge_requirements",
        verification_challenge_requirement_inventory(authority)?,
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
    let affected_selection =
        affected_execution_selection(v.get("selection"), authority, locations)?;
    if let Some(requirements) = v.get("requirement_definitions") {
        let expected = Json::Arr(
            arr(authority, "requirements")?
                .iter()
                .filter(|requirement| text(requirement, "criticality").ok() != Some("routine"))
                .map(|requirement| {
                    Json::obj(vec![(
                        "requirement",
                        requirement.get("id").unwrap().clone(),
                    )])
                })
                .collect(),
        );
        if requirements != &expected {
            return Err("Requirement definition targets differ from candidate authority".into());
        }
    }
    if let Some(verification_challenges) = v.get("verification_challenges") {
        if !arr(authority, "requirements")?
            .iter()
            .all(|requirement| requirement.get("verification_challenges").is_some())
        {
            return Err(
                "Historical authority does not establish Verification Challenge inventory completeness".into(),
            );
        }
        let expected = arr(authority, "requirements")?
            .iter()
            .flat_map(|requirement| arr(requirement, "verification_challenges").unwrap())
            .map(|verification_challenge| text(verification_challenge, "id").unwrap().to_string())
            .filter(|id| execution_selected(affected_selection.as_ref(), "challenges", id))
            .collect::<BTreeSet<_>>();
        if verification_challenges != &Json::Arr(expected.iter().map(Json::str).collect()) {
            return Err("Verification Challenge inventory differs from candidate authority".into());
        }
    }
    if let Some(mapping) = v.get("verification_challenge_requirements") {
        if mapping != &verification_challenge_requirement_inventory(authority)? {
            return Err(
                "Verification Challenge Requirement mapping differs from candidate authority"
                    .into(),
            );
        }
    }
    if let Some(methods) = v.get("methods").and_then(Json::as_array) {
        let expected = arr(authority, "checks")?
            .iter()
            .filter(|check| {
                local_check(check)
                    && execution_selected(
                        affected_selection.as_ref(),
                        "checks",
                        check.get("id").and_then(Json::as_str).unwrap_or_default(),
                    )
            })
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
            .any(|c| c.get("id") == check.get("check") && local_check(c))
        {
            return Err("inventory Check absent from local candidate execution scope".into());
        }
    }
    for contribution in arr(v, "contributions")? {
        if !arr(v, "checks")?.iter().any(|c| {
            c.get("check") == contribution.get("check")
                && c.get("verification_target") == contribution.get("verification_target")
        }) {
            return Err("contribution has no Check/Verification Target execution target".into());
        }
        if !arr(authority, "scenarios")?.iter().any(|c| {
            c.get("id") == contribution.get("scenario")
                && c.get("requirement") == contribution.get("requirement")
        }) {
            return Err("contribution Scenario/Requirement absent from candidate authority".into());
        }
        let requirement = arr(authority, "requirements")?
            .iter()
            .find(|c| c.get("id") == contribution.get("requirement"))
            .ok_or("unknown contribution Requirement")?;
        if !arr(requirement, "bindings")?.iter().any(|b| {
            b.get("check") == contribution.get("check")
                && b.get("scenario") == contribution.get("scenario")
        }) {
            return Err(
                "contribution Check–Scenario relation absent from candidate authority".into(),
            );
        }
    }
    if !arr(v, "requirements")?.is_empty() {
        return Err("assurance inventory Requirement targets are not supported: Requirement Status is never authored".into());
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
        "author-design" => Some("requirement-design"),
        "author-design-delta" => Some("design-delta"),
        "author-verification" => Some("verification-account"),
        "author-verification-delta" => Some("verification-delta"),
        "review-proposal"
        | "review-spec-delta"
        | "review-design"
        | "review-verification"
        | "review-design-delta"
        | "review-verification-delta" => Some("authoring-review"),
        "author-change-outcome" => Some("change-outcome"),
        "request-change-acceptance" => Some("change-acceptance"),
        "finalize-change" => Some("change-finalization"),
        "implement-change" => Some("implementation-result"),
        "validate-change-account" => Some("candidate-validation"),
        "request-implementation-approval" => Some("implementation-approval"),
        "prepare-candidate-model-revision" => Some("candidate-preparation"),
        "register-candidate-model-revision" => Some("candidate-registration"),
        "prepare-publication-inputs" => Some("publication-inputs"),
        "qualify-method" => Some("method-qualification"),
        "review-requirement" => Some("requirement-review"),
        "execute-check" | "run-verification-challenges" | "run-verification-challenge" => {
            Some("run-bundle")
        }
        "execute-qualified-check" => Some("run-bundle"),
        "assess-applicability" => Some("applicability-decision"),
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

#[cfg(test)]
mod deployment_status_workflow_tests {
    use super::*;

    fn bindings(pending: bool) -> Json {
        let fingerprint = Json::str(format!("sha256:{}", "a".repeat(64)));
        Json::obj(vec![
            ("project", Json::str("sample-project")),
            ("change", Json::str("sample-change")),
            (
                "request",
                Json::obj(vec![
                    ("kind", Json::str("initial-request")),
                    ("ref", Json::str("request.md")),
                ]),
            ),
            ("instructions", Json::Arr(vec![])),
            (
                "modules",
                Json::Arr(vec![Json::obj(vec![("module", Json::str("catalog"))])]),
            ),
            (
                "assurance",
                Json::obj(vec![
                    ("format", Json::str("azimuth-assurance-target-inventory")),
                    ("version", Json::Num(2.0)),
                    ("project", Json::str("sample-project")),
                    ("model_revision", fingerprint),
                    (
                        "snapshot",
                        Json::obj(vec![
                            ("kind", Json::str("assurance-snapshot")),
                            ("ref", Json::str("snapshot.json")),
                        ]),
                    ),
                    (
                        "methods",
                        Json::Arr(if pending {
                            vec![Json::obj(vec![("check", Json::str("source-check"))])]
                        } else {
                            vec![]
                        }),
                    ),
                    ("checks", Json::Arr(vec![])),
                    ("contributions", Json::Arr(vec![])),
                    ("requirements", Json::Arr(vec![])),
                    (
                        "requirement_definitions",
                        Json::Arr(vec![Json::obj(vec![(
                            "requirement",
                            Json::str("restricted-access"),
                        )])]),
                    ),
                    ("verification_challenges", Json::Arr(vec![])),
                    ("verification_challenge_requirements", Json::Arr(vec![])),
                    (
                        "pending_verification_targets",
                        Json::Arr(if pending {
                            vec![Json::obj(vec![
                                ("kind", Json::str("check")),
                                ("id", Json::str("source-check")),
                            ])]
                        } else {
                            vec![]
                        }),
                    ),
                ]),
            ),
        ])
    }

    #[test]
    fn hosted_only_inventory_can_prepare_publication_without_authored_requirement_status() {
        let template = shipped_template("change-development").unwrap();
        let definition = expand_generation(&template, &bindings(false)).unwrap();
        assert!(arr(&definition, "tasks")
            .unwrap()
            .iter()
            .any(|task| text(task, "kind").unwrap() == "prepare-publication-inputs"));
        assert!(arr(&definition, "tasks")
            .unwrap()
            .iter()
            .any(|task| text(task, "kind").unwrap() == "review-requirement"));
    }

    #[test]
    fn missing_local_verification_target_blocks_publication_expansion() {
        let template = shipped_template("change-development").unwrap();
        let definition = expand_generation(&template, &bindings(true)).unwrap();
        assert!(!arr(&definition, "tasks")
            .unwrap()
            .iter()
            .any(|task| text(task, "kind").unwrap() == "prepare-publication-inputs"));
    }

    #[test]
    fn local_execution_scope_excludes_deployment_only_checks() {
        let hosted = Json::obj(vec![(
            "deployment",
            Json::obj(vec![("verification_target", Json::str("deployment"))]),
        )]);
        let source = Json::obj(vec![(
            "deployment",
            Json::obj(vec![("verification_target", Json::str("source"))]),
        )]);
        assert!(!local_check(&hosted));
        assert!(local_check(&source));
        assert!(local_check(&Json::obj(vec![])));
    }

    fn retire_review_action(value: &Json) -> Json {
        match value {
            Json::Obj(fields) => Json::Obj(
                fields
                    .iter()
                    .map(|(key, item)| {
                        let item = if key == "kind" && item.as_str() == Some("review-requirement") {
                            Json::str("assess-requirement-status")
                        } else {
                            retire_review_action(item)
                        };
                        (key.clone(), item)
                    })
                    .collect(),
            ),
            Json::Arr(items) => Json::Arr(items.iter().map(retire_review_action).collect()),
            other => other.clone(),
        }
    }

    #[test]
    fn authored_requirement_status_actions_are_rejected_with_their_replacement() {
        for kind in ["assess-requirement-status", "judge-requirement"] {
            let error = registered_kind(kind).unwrap_err();
            assert!(error.contains(&format!("`{kind}`")), "{error}");
            assert!(
                error.contains("computed by the Assurance Service"),
                "{error}"
            );
        }
        assert!(registered_kind("review-requirement").is_ok());
        assert_eq!(
            registered_kind("invented").unwrap_err(),
            "unregistered workflow task kind `invented`"
        );
        let shipped = shipped_template("change-development").unwrap();
        let error = template(&retire_review_action(&shipped)).unwrap_err();
        assert!(
            error.contains("`assess-requirement-status` authors a Requirement Status assessment"),
            "{error}"
        );
    }
    #[test]
    fn candidate_reference_materialization_preserves_identity_and_rejects_changed_content() {
        let directory = std::env::temp_dir().join(format!(
            "azimuth-candidate-references-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let directory = fs::canonicalize(directory).unwrap();
        let locations = BTreeMap::from([(String::new(), directory.clone())]);
        for kind in [
            "candidate-authority",
            "assurance-snapshot",
            "assurance-verification-target-selection",
        ] {
            let path = directory.join(kind);
            fs::write(&path, b"{\"selection\":\"exact\"}").unwrap();
            let portable = Json::obj(vec![("kind", Json::str(kind)), ("ref", Json::str(kind))]);
            let materialized = materialize(&portable, &locations).unwrap();
            assert_eq!(
                portable_assurance_reference(&materialized, &locations).unwrap(),
                portable
            );
            assert_eq!(
                portable_assurance_reference(&portable, &locations).unwrap(),
                portable
            );
            fs::write(&path, b"{\"selection\":\"changed\"}").unwrap();
            assert!(portable_assurance_reference(&materialized, &locations)
                .unwrap_err()
                .contains("record changed"));
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
