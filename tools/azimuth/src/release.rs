use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};

const PACKAGE_LIMIT: usize = 8 * 1024 * 1024;
const RECORD_LIMIT: usize = 512 * 1024;

fn field<'a>(value: &'a Json, name: &str) -> Result<&'a Json, String> {
    value.get(name).ok_or_else(|| format!("missing `{name}`"))
}
fn text<'a>(value: &'a Json, name: &str) -> Result<&'a str, String> {
    field(value, name)?
        .as_str()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| format!("`{name}` must be a nonempty string"))
}
fn array<'a>(value: &'a Json, name: &str) -> Result<&'a [Json], String> {
    field(value, name)?
        .as_array()
        .ok_or_else(|| format!("`{name}` must be an array"))
}
fn shape(value: &Json, required: &[&str], optional: &[&str]) -> Result<(), String> {
    let Json::Obj(pairs) = value else {
        return Err("expected object".into());
    };
    let mut seen = BTreeSet::new();
    for (key, _) in pairs {
        if !seen.insert(key)
            || (!required.contains(&key.as_str()) && !optional.contains(&key.as_str()))
        {
            return Err(format!("unexpected or duplicate field `{key}`"));
        }
    }
    for key in required {
        field(value, key)?;
    }
    Ok(())
}
fn digest(value: &str) -> Result<(), String> {
    let Some(hash) = value.strip_prefix("sha256:") else {
        return Err("expected sha256 digest".into());
    };
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
    {
        return Err("expected lowercase sha256 digest with 64 hexadecimal digits".into());
    }
    Ok(())
}
fn digest_field(value: &Json, key: &str) -> Result<(), String> {
    digest(text(value, key)?)
}
fn id(value: &Json, key: &str) -> Result<String, String> {
    let id = text(value, key)?;
    if id.len() > 256 || id.chars().any(char::is_control) {
        return Err(format!("invalid `{key}`"));
    }
    Ok(id.into())
}
fn strings(value: &Json, key: &str) -> Result<BTreeSet<String>, String> {
    let mut values = BTreeSet::new();
    for item in array(value, key)? {
        let name = item
            .as_str()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("`{key}` requires strings"))?;
        if !values.insert(name.to_string()) {
            return Err(format!("duplicate `{key}` member `{name}`"));
        }
    }
    Ok(values)
}
fn fingerprint(value: &Json) -> Result<String, String> {
    crate::run::canonical_fingerprint(value)
}

pub fn prepare(request: &Json) -> Result<Json, String> {
    shape(
        request,
        &[
            "format",
            "version",
            "project",
            "label",
            "baseline_model_revision",
            "changes",
            "sources",
            "builds",
            "artifacts",
            "deployments",
            "snapshot",
            "evidence",
        ],
        &["catalog", "composition", "required_reviews"],
    )?;
    if text(request, "format")? != "azimuth-release-request"
        || field(request, "version")?.as_num() != Some(1.0)
    {
        return Err("expected azimuth-release-request version 1".into());
    }
    let mut release = vec![
        ("label", field(request, "label")?.clone()),
        (
            "baseline_model_revision",
            field(request, "baseline_model_revision")?.clone(),
        ),
        (
            "model_revision",
            field(field(request, "snapshot")?, "model_revision")?.clone(),
        ),
    ];
    for key in ["changes", "sources", "builds", "artifacts", "deployments"] {
        release.push((key, field(request, key)?.clone()));
    }
    if let Some(composition) = request.get("composition") {
        release.push(("composition", composition.clone()));
    }
    if let Some(reviews) = request.get("required_reviews") {
        release.push(("required_reviews", reviews.clone()));
    }
    let hash = fingerprint(&Json::obj(release.clone()))?;
    release.push(("fingerprint", Json::str(hash)));
    let mut package = vec![
        ("format", Json::str("azimuth-release-publication")),
        ("version", Json::Num(1.0)),
        ("project", field(request, "project")?.clone()),
        ("release", Json::obj(release)),
        ("snapshot", field(request, "snapshot")?.clone()),
        ("evidence", field(request, "evidence")?.clone()),
    ];
    if let Some(catalog) = request.get("catalog") {
        package.push(("catalog", catalog.clone()));
    }
    let package = Json::obj(package);
    verify(&package)?;
    Ok(package)
}

pub fn verify(value: &Json) -> Result<Json, String> {
    shape(
        value,
        &[
            "format", "version", "project", "release", "snapshot", "evidence",
        ],
        &["catalog"],
    )?;
    if text(value, "format")? != "azimuth-release-publication"
        || field(value, "version")?.as_num() != Some(1.0)
    {
        return Err("expected azimuth-release-publication version 1".into());
    }
    if value.to_string_pretty().len() > PACKAGE_LIMIT {
        return Err("release package exceeds 8 MiB; move full reports/logs out of native records at their producer, preserving required observations".into());
    }
    let project = text(value, "project")?;
    crate::diag::validate_id(project, false)?;
    let snapshot = field(value, "snapshot")?;
    if text(snapshot, "project")? != project {
        return Err("snapshot project differs from publication".into());
    }
    let assessment = crate::assurance_publication::verify_snapshot(snapshot)?;
    let unavailable = array(value, "evidence")?
        .iter()
        .filter(|item| item.get("availability").and_then(Json::as_str) == Some("unavailable"))
        .filter_map(|item| {
            item.get("digest")
                .and_then(Json::as_str)
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    for key in ["reviews", "bundles"] {
        for (index, record) in array(snapshot, key)?.iter().enumerate() {
            if record.to_string_pretty().len() > RECORD_LIMIT {
                return Err(format!("snapshot.{key}[{index}] exceeds 512 KiB; export compact records at their producer; never rehash immutable history to remove evidence"));
            }
            portable_evidence(record, &format!("snapshot.{key}[{index}]"), &unavailable)?;
        }
    }
    let release = field(value, "release")?;
    shape(
        release,
        &[
            "fingerprint",
            "label",
            "baseline_model_revision",
            "model_revision",
            "changes",
            "sources",
            "builds",
            "artifacts",
            "deployments",
        ],
        &["composition", "required_reviews"],
    )?;
    id(release, "label")?;
    digest_field(release, "baseline_model_revision")?;
    digest_field(release, "model_revision")?;
    if release.get("model_revision") != snapshot.get("model_revision") {
        return Err("release model differs from snapshot".into());
    }
    let Json::Obj(fields) = release else {
        unreachable!()
    };
    let body = Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "fingerprint")
            .cloned()
            .collect(),
    );
    if text(release, "fingerprint")? != fingerprint(&body)? {
        return Err("release fingerprint mismatch".into());
    }
    let mut change_ids = BTreeSet::new();
    for change in array(release, "changes")? {
        shape(change, &["id", "revision"], &["metadata"])?;
        if let Some(metadata) = change.get("metadata") {
            verify_change_metadata(metadata)?;
        }
        digest_field(change, "revision")?;
        if !change_ids.insert(id(change, "id")?) {
            return Err("duplicate Change identity".into());
        }
    }
    if change_ids.is_empty() {
        return Err("release must explicitly include at least one Change".into());
    }
    let mut repositories = BTreeSet::new();
    for source in array(release, "sources")? {
        shape(
            source,
            &["repository", "commit", "content_fingerprint"],
            &[],
        )?;
        digest_field(source, "content_fingerprint")?;
        let commit = text(source, "commit")?;
        if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|v| v.is_ascii_hexdigit()) {
            return Err(
                "release sources require full Git commit IDs, not branch names or abbreviated SHAs"
                    .into(),
            );
        }
        if !repositories.insert(id(source, "repository")?) {
            return Err("duplicate source repository".into());
        }
    }
    if repositories.is_empty() {
        return Err("release sources must be explicit".into());
    }
    let mut artifacts = BTreeSet::new();
    for artifact in array(release, "artifacts")? {
        shape(artifact, &["digest", "kind"], &[])?;
        digest_field(artifact, "digest")?;
        id(artifact, "kind")?;
        if !artifacts.insert(text(artifact, "digest")?.to_string()) {
            return Err("duplicate artifact digest".into());
        }
    }
    let mut builds = BTreeSet::new();
    let mut produced = BTreeSet::new();
    for build in array(release, "builds")? {
        shape(
            build,
            &[
                "id",
                "sources",
                "toolchain",
                "configuration_fingerprint",
                "artifacts",
            ],
            &[],
        )?;
        if !builds.insert(id(build, "id")?) {
            return Err("duplicate build identity".into());
        }
        let Json::Obj(toolchain) = field(build, "toolchain")? else {
            return Err("build toolchain must be an object of named string versions".into());
        };
        if toolchain.is_empty() {
            return Err("build toolchain must be explicit".into());
        }
        for (name, version) in toolchain {
            if name.is_empty() || version.as_str().is_none_or(|v| v.trim().is_empty()) {
                return Err("toolchain entries require names and nonempty string values".into());
            }
        }
        digest_field(build, "configuration_fingerprint")?;
        let inputs = strings(build, "sources")?;
        let outputs = strings(build, "artifacts")?;
        if inputs.is_empty()
            || !inputs.is_subset(&repositories)
            || outputs.is_empty()
            || !outputs.is_subset(&artifacts)
        {
            return Err(
                "build requires explicit registered source inputs and artifact outputs".into(),
            );
        }
        produced.extend(outputs);
    }
    if produced != artifacts {
        return Err("every artifact requires build provenance".into());
    }
    let mut deployments = BTreeSet::new();
    for deployment in array(release, "deployments")? {
        verify_deployment(deployment, &artifacts)?;
        if !deployments.insert(id(deployment, "id")?) {
            return Err("duplicate deployment identity".into());
        }
    }
    let mut evidence = BTreeSet::new();
    for artifact in array(value, "evidence")? {
        shape(artifact, &["digest", "media_type", "availability"], &[])?;
        digest_field(artifact, "digest")?;
        id(artifact, "media_type")?;
        if text(artifact, "availability")? != "unavailable" {
            return Err("evidence storage is deferred; availability must be unavailable".into());
        }
        if !evidence.insert(text(artifact, "digest")?.to_string()) {
            return Err("duplicate evidence digest".into());
        }
    }
    if let Some(catalog) = value.get("catalog") {
        verify_catalog(catalog, field(snapshot, "authority")?)?;
    }
    if release.get("required_reviews").is_some() {
        let selected = strings(release, "required_reviews")?;
        let reviews = array(snapshot, "reviews")?;
        let bundles = array(snapshot, "bundles")?
            .iter()
            .map(|bundle| {
                crate::run::parse("publication.bundles", &bundle.to_string_pretty())
                    .map_err(|errors| format!("invalid native Run: {errors:?}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut covered = BTreeSet::new();
        for fingerprint in selected {
            digest(&fingerprint)?;
            let record = reviews.iter().find(|record| record.get("fingerprint").and_then(Json::as_str) == Some(fingerprint.as_str())).ok_or_else(|| format!("required review `{fingerprint}` missing; publish its native record and meaningful dependencies"))?;
            if text(record, "kind")? != "claim-review" {
                return Err(
                    "required_reviews selects Claim Review records, not Claim Health".into(),
                );
            }
            covered.insert(text(record, "claim")?.to_string());
            let latest = crate::assurance_review::latest_review(reviews, record)?
                .ok_or("required review has no current target")?;
            let assessment = crate::assurance_review::assess_record(
                record,
                field(snapshot, "authority")?,
                reviews,
                &bundles,
            );
            if latest.get("fingerprint") != record.get("fingerprint")
                || assessment.status != "accepted"
                || record.get("decision").and_then(Json::as_str) != Some("accepted")
            {
                return Err(format!("required review `{fingerprint}` is not current and accepted: {:?}; resolve locally, CI never invokes an agent", assessment.diagnostics));
            }
        }
        for claim in array(field(snapshot, "authority")?, "claims")? {
            if text(claim, "criticality")? != "routine" && !covered.contains(text(claim, "id")?) {
                return Err(format!("required_reviews omits non-routine Claim `{}`; include its latest accepted current review", text(claim, "id")?));
            }
        }
    }
    if let Some(composition) = release.get("composition") {
        verify_composition(composition)?;
        if composition.get("baseline_model_revision") != release.get("baseline_model_revision")
            || change_pairs(composition)? != change_pairs(release)?
        {
            return Err("release composition baseline/Changes differ from release".into());
        }
    }
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-release-validation")),
        ("version", Json::Num(1.0)),
        ("project", Json::str(project)),
        (
            "release_fingerprint",
            field(release, "fingerprint")?.clone(),
        ),
        ("valid", Json::Bool(true)),
        ("acceptance", Json::Bool(false)),
        ("assurance", assessment),
    ]))
}

pub fn verify_deployment(value: &Json, artifacts: &BTreeSet<String>) -> Result<(), String> {
    shape(
        value,
        &[
            "id",
            "environment",
            "configuration_fingerprint",
            "artifacts",
        ],
        &["subject", "subject_fingerprint"],
    )?;
    id(value, "id")?;
    id(value, "environment")?;
    digest_field(value, "configuration_fingerprint")?;
    let deployed = strings(value, "artifacts")?;
    if deployed.is_empty() || !deployed.is_subset(artifacts) {
        return Err("deployment must identify registered artifact digests".into());
    }
    match (value.get("subject"), value.get("subject_fingerprint")) {
        (None, None) => {}
        (Some(subject), Some(fingerprint)) => {
            let native = crate::run::subject_from_json(subject)?;
            if !crate::run::validate_subject_component(&native).is_empty() {
                return Err("deployment Subject is invalid".into());
            }
            if fingerprint.as_str() != Some(crate::run::subject_fingerprint(&native).as_str()) {
                return Err("deployment Subject fingerprint mismatch".into());
            }
            let crate::run::Subject::Deployment {
                environment,
                deployment,
                artifacts,
                ..
            } = native
            else {
                return Err("deployment requires a native deployment Subject".into());
            };
            let digests = artifacts
                .iter()
                .map(|artifact| artifact.digest.clone())
                .collect::<BTreeSet<_>>();
            if environment != text(value, "environment")?
                || deployment != text(value, "id")?
                || digests != deployed
            {
                return Err("deployment Subject must describe the recorded environment, deployment and exact artifact digests".into());
            }
        }
        _ => {
            return Err(
                "deployment subject and subject_fingerprint must be supplied together".into(),
            )
        }
    }
    Ok(())
}

fn portable(value: &Json, at: &str) -> Result<(), String> {
    match value {
        Json::Str(value) if value.starts_with("file:") || value.starts_with("/Users/") || value.starts_with("/private/") || value.starts_with("/tmp/") || value.starts_with(".azimuth/") || value.contains("/.azimuth/") || value.contains(":\\Users\\") => {
            Err(format!("{at}: local-only evidence locator cannot be published; retain its digest and declare unavailable evidence without rewriting a signed native record"))
        }
        Json::Obj(fields) => {
            for (key, value) in fields {
                if matches!(key.as_str(), "stdout" | "stderr") && value.as_str().is_some_and(|v| !v.is_empty()) {
                    return Err(format!("{at}.{key}: full process output belongs in an unavailable supporting artifact; export at the producer boundary"));
                }
                portable(value, &format!("{at}.{key}"))?;
            }
            Ok(())
        }
        Json::Arr(values) => { for (index, value) in values.iter().enumerate() { portable(value, &format!("{at}[{index}]"))?; } Ok(()) }
        _ => Ok(()),
    }
}

fn verify_catalog(catalog: &Json, authority: &Json) -> Result<(), String> {
    shape(catalog, &["modules", "claims", "cases", "checks"], &[])?;
    let mut members = BTreeSet::new();
    let mut modules = BTreeMap::new();
    for module in array(catalog, "modules")? {
        shape(module, &["id", "title", "claims"], &["parent"])?;
        let module_id = id(module, "id")?;
        id(module, "title")?;
        let claims = strings(module, "claims")?;
        for claim in claims {
            if !members.insert(claim) {
                return Err("Claim belongs to multiple catalog Modules".into());
            }
        }
        if modules
            .insert(
                module_id,
                module
                    .get("parent")
                    .and_then(Json::as_str)
                    .map(str::to_string),
            )
            .is_some()
        {
            return Err("duplicate catalog Module".into());
        }
    }
    for module in modules.keys() {
        let mut seen = BTreeSet::new();
        let mut cursor = module;
        while let Some(Some(parent)) = modules.get(cursor) {
            if !seen.insert(cursor) || !modules.contains_key(parent) {
                return Err("catalog Module parent missing or cyclic".into());
            }
            cursor = parent;
        }
    }
    for key in ["claims", "cases", "checks"] {
        let mut actual = BTreeSet::new();
        for item in array(catalog, key)? {
            shape(
                item,
                &[
                    "id",
                    if key == "claims" {
                        "statement"
                    } else {
                        "description"
                    },
                ],
                &[],
            )?;
            text(
                item,
                if key == "claims" {
                    "statement"
                } else {
                    "description"
                },
            )?;
            if !actual.insert(id(item, "id")?) {
                return Err(format!("duplicate catalog {key}"));
            }
        }
        let expected = array(authority, key)?
            .iter()
            .map(|item| id(item, "id"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if actual != expected || (key == "claims" && members != expected) {
            return Err(format!(
                "catalog {key} must exactly cover authority identities"
            ));
        }
    }
    Ok(())
}

pub fn catalog(model: &crate::model::Model) -> Json {
    Json::obj(vec![
        (
            "modules",
            Json::Arr(
                model
                    .specs
                    .iter()
                    .map(|spec| {
                        Json::obj(vec![
                            ("id", Json::str(&spec.id)),
                            ("title", Json::str(&spec.id)),
                            (
                                "claims",
                                Json::Arr(
                                    spec.claims
                                        .iter()
                                        .map(|claim| Json::str(&claim.id))
                                        .collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "claims",
            Json::Arr(
                model
                    .claims()
                    .map(|view| {
                        Json::obj(vec![
                            ("id", Json::str(&view.claim.id)),
                            ("statement", Json::str(&view.claim.statement)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "cases",
            Json::Arr(
                model
                    .cases()
                    .map(|view| {
                        Json::obj(vec![
                            ("id", Json::str(&view.case.id)),
                            ("description", Json::str(&view.case.statement)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "checks",
            Json::Arr(
                model
                    .checks()
                    .map(|check| {
                        Json::obj(vec![
                            ("id", Json::str(&check.id)),
                            (
                                "description",
                                Json::str(if check.methods.is_empty() {
                                    check.id.clone()
                                } else {
                                    check.methods.join("\n")
                                }),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn facet_path(value: &str) -> Result<(), String> {
    let path = std::path::Path::new(value);
    if value.contains('\\')
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        || !matches!(
            path.file_name().and_then(|v| v.to_str()),
            Some("spec.md" | "design.md" | "verification.md")
        )
    {
        return Err(format!(
            "`{value}`: expected a relative model facet path without traversal"
        ));
    }
    Ok(())
}

pub fn compose(request: &Json) -> Result<Json, String> {
    shape(
        request,
        &[
            "format",
            "version",
            "baseline_model_revision",
            "files",
            "changes",
        ],
        &[],
    )?;
    if text(request, "format")? != "azimuth-release-composition-request"
        || field(request, "version")?.as_num() != Some(1.0)
    {
        return Err("expected azimuth-release-composition-request version 1".into());
    }
    digest_field(request, "baseline_model_revision")?;
    let mut files = BTreeMap::new();
    for file in array(request, "files")? {
        shape(file, &["path", "content"], &[])?;
        let path = text(file, "path")?;
        facet_path(path)?;
        let content = field(file, "content")?
            .as_str()
            .ok_or("facet content must be a string")?;
        if files
            .insert(path.to_string(), content.to_string())
            .is_some()
        {
            return Err(format!("duplicate baseline facet `{path}`"));
        }
    }
    let baseline_content = tree_fingerprint(&files)?;
    let mut writes = BTreeSet::new();
    let mut changes = Vec::new();
    let mut change_ids = BTreeSet::new();
    for change in array(request, "changes")? {
        shape(change, &["id", "revision", "files"], &[])?;
        let change_id = id(change, "id")?;
        digest_field(change, "revision")?;
        if !change_ids.insert(change_id.clone()) {
            return Err(format!("duplicate Change `{change_id}`"));
        }
        if array(change, "files")?.is_empty() {
            return Err(format!("Change `{change_id}` has no facet transformations"));
        }
        for file in array(change, "files")? {
            shape(file, &["path", "before_fingerprint", "content"], &[])?;
            let path = text(file, "path")?;
            facet_path(path)?;
            if !writes.insert(path.to_string()) {
                return Err(format!("composition conflict at `{path}`; rebase the Changes locally and reassess affected reviews"));
            }
            let before = files.get(path).map(|content| {
                format!("sha256:{}", crate::fingerprint::sha256(content.as_bytes()))
            });
            let expected = match field(file, "before_fingerprint")? {
                Json::Null => None,
                Json::Str(value) => {
                    digest(value)?;
                    Some(value.clone())
                }
                _ => {
                    return Err(
                        "before_fingerprint must be sha256 or null for an absent facet".into(),
                    )
                }
            };
            if before != expected {
                return Err(format!("baseline drift at `{path}` in Change `{change_id}`; regenerate its projection locally"));
            }
            match field(file, "content")? {
                Json::Null => {
                    if files.remove(path).is_none() {
                        return Err(format!("cannot delete absent facet `{path}`"));
                    }
                }
                Json::Str(content) => {
                    files.insert(path.to_string(), content.clone());
                }
                _ => {
                    return Err(
                        "projected facet content must be a string or null for deletion".into(),
                    )
                }
            }
        }
        changes.push(Json::obj(vec![
            ("id", Json::str(change_id)),
            ("revision", field(change, "revision")?.clone()),
        ]));
    }
    if changes.is_empty() {
        return Err("composition requires explicit Changes".into());
    }
    let body = Json::obj(vec![
        ("format", Json::str("azimuth-release-composition")),
        ("version", Json::Num(1.0)),
        (
            "baseline_model_revision",
            field(request, "baseline_model_revision")?.clone(),
        ),
        ("baseline_content_fingerprint", Json::str(baseline_content)),
        ("changes", Json::Arr(changes)),
        ("files", files_json(&files)),
        ("content_fingerprint", Json::str(tree_fingerprint(&files)?)),
    ]);
    let hash = fingerprint(&body)?;
    let Json::Obj(mut fields) = body else {
        unreachable!()
    };
    fields.push(("fingerprint".into(), Json::str(hash)));
    Ok(Json::Obj(fields))
}
fn files_json(files: &BTreeMap<String, String>) -> Json {
    Json::Arr(
        files
            .iter()
            .map(|(path, content)| {
                Json::obj(vec![
                    ("path", Json::str(path)),
                    ("content", Json::str(content)),
                ])
            })
            .collect(),
    )
}
fn tree_fingerprint(files: &BTreeMap<String, String>) -> Result<String, String> {
    fingerprint(&files_json(files))
}

pub fn verify_composition(value: &Json) -> Result<(), String> {
    shape(
        value,
        &[
            "format",
            "version",
            "baseline_model_revision",
            "baseline_content_fingerprint",
            "changes",
            "files",
            "content_fingerprint",
            "fingerprint",
        ],
        &[],
    )?;
    if text(value, "format")? != "azimuth-release-composition"
        || field(value, "version")?.as_num() != Some(1.0)
    {
        return Err("expected azimuth-release-composition version 1".into());
    }
    for key in [
        "baseline_model_revision",
        "baseline_content_fingerprint",
        "content_fingerprint",
    ] {
        digest_field(value, key)?;
    }
    let Json::Obj(fields) = value else {
        unreachable!()
    };
    if text(value, "fingerprint")?
        != fingerprint(&Json::Obj(
            fields
                .iter()
                .filter(|(key, _)| key != "fingerprint")
                .cloned()
                .collect(),
        ))?
    {
        return Err("composition fingerprint mismatch".into());
    }
    let mut files = BTreeMap::new();
    for file in array(value, "files")? {
        shape(file, &["path", "content"], &[])?;
        let path = text(file, "path")?;
        facet_path(path)?;
        let content = field(file, "content")?
            .as_str()
            .ok_or("facet content must be string")?;
        if files
            .insert(path.to_string(), content.to_string())
            .is_some()
        {
            return Err("duplicate composed facet".into());
        }
    }
    if text(value, "content_fingerprint")? != tree_fingerprint(&files)? {
        return Err("composed content fingerprint mismatch".into());
    }
    Ok(())
}

pub fn materialize(composition: &Json, target: &std::path::Path) -> Result<(), String> {
    verify_composition(composition)?;
    if target.exists() || target.symlink_metadata().is_ok() {
        return Err(
            "--model-out must be an absent directory; accepted model files are never overwritten"
                .into(),
        );
    }
    let parent = target
        .parent()
        .filter(|v| !v.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let parent = parent
        .canonicalize()
        .map_err(|e| format!("model output parent: {e}"))?;
    let name = target
        .file_name()
        .ok_or("model output requires directory name")?;
    let destination = parent.join(name);
    std::fs::create_dir(&destination).map_err(|e| format!("{}: {e}", destination.display()))?;
    for file in array(composition, "files")? {
        let path = destination.join(text(file, "path")?);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(
            &path,
            field(file, "content")?
                .as_str()
                .ok_or("facet content must be a string")?,
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

fn collect_files(
    root: &std::path::Path,
    relative: &std::path::Path,
    facets: bool,
    output: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let directory = root.join(relative);
    for entry in std::fs::read_dir(&directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name();
        if name == ".azimuth" || name == ".git" {
            continue;
        }
        let path = relative.join(&name);
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            return Err(format!(
                "{}: symbolic links cannot define release inputs",
                root.join(&path).display()
            ));
        }
        if kind.is_dir() {
            collect_files(root, &path, facets, output)?;
        } else if kind.is_file() {
            if facets
                && !matches!(
                    name.to_str(),
                    Some("spec.md" | "design.md" | "verification.md")
                )
            {
                continue;
            }
            let source = std::fs::read(root.join(&path)).map_err(|error| error.to_string())?;
            let value = if facets {
                String::from_utf8(source).map_err(|error| error.to_string())?
            } else {
                format!("sha256:{}", crate::fingerprint::sha256(&source))
            };
            output.insert(path.to_string_lossy().replace('\\', "/"), value);
        } else {
            return Err(format!(
                "{}: expected a regular file or directory",
                root.join(&path).display()
            ));
        }
    }
    Ok(())
}

pub fn project(
    model: &crate::model::Model,
    model_root: &std::path::Path,
    roots: &[std::path::PathBuf],
    baseline_revision: &str,
) -> Result<Json, String> {
    digest(baseline_revision)?;
    let mut baseline = BTreeMap::new();
    collect_files(model_root, std::path::Path::new(""), true, &mut baseline)?;
    let mut changes = Vec::new();
    for root in roots {
        let report = crate::change::inspect(root, model).map_err(|errors| errors.join("\n"))?;
        let intent =
            crate::intent::project(root, model_root, model).map_err(|errors| errors.join("\n"))?;
        let account = crate::account_delta::project_new_modules(&root.join("deltas"), model_root)
            .map_err(|errors| errors.join("\n"))?;
        let mut authored = BTreeMap::new();
        collect_files(root, std::path::Path::new(""), false, &mut authored)?;
        let revision = tree_fingerprint(&authored)?;
        let mut files = Vec::new();
        for (path, before, after) in intent
            .files
            .into_iter()
            .map(|file| {
                let before = if file.path.exists() {
                    Some(file.before)
                } else {
                    None
                };
                (
                    file.path,
                    before,
                    if file.after.is_empty() {
                        None
                    } else {
                        Some(file.after)
                    },
                )
            })
            .chain(
                account
                    .files
                    .into_iter()
                    .map(|file| (file.path, file.before, Some(file.after))),
            )
        {
            if before == after {
                continue;
            }
            let relative = path
                .strip_prefix(model_root)
                .map_err(|_| "projected facet lies outside model root")?
                .to_string_lossy()
                .replace('\\', "/");
            files.push(Json::obj(vec![
                ("path", Json::str(relative)),
                (
                    "before_fingerprint",
                    before
                        .map(|text| {
                            Json::str(format!(
                                "sha256:{}",
                                crate::fingerprint::sha256(text.as_bytes())
                            ))
                        })
                        .unwrap_or(Json::Null),
                ),
                ("content", after.map(Json::str).unwrap_or(Json::Null)),
            ]));
        }
        changes.push(Json::obj(vec![
            ("id", Json::str(&report.id)),
            ("revision", Json::str(revision)),
            ("files", Json::Arr(files)),
        ]));
    }
    let request = Json::obj(vec![
        ("format", Json::str("azimuth-release-composition-request")),
        ("version", Json::Num(1.0)),
        ("baseline_model_revision", Json::str(baseline_revision)),
        ("files", files_json(&baseline)),
        ("changes", Json::Arr(changes)),
    ]);
    compose(&request)?;
    Ok(request)
}

fn change_pairs(value: &Json) -> Result<BTreeMap<String, String>, String> {
    array(value, "changes")?
        .iter()
        .map(|change| {
            Ok((
                text(change, "id")?.to_string(),
                text(change, "revision")?.to_string(),
            ))
        })
        .collect()
}
fn verify_change_metadata(value: &Json) -> Result<(), String> {
    shape(value, &[], &["request", "repositories", "contributions"])?;
    portable(value, "change.metadata")?;
    if let Some(request) = value.get("request") {
        shape(request, &["title"], &["ref"])?;
        text(request, "title")?;
        if request.get("ref").is_some() {
            text(request, "ref")?;
        }
    }
    if value.get("repositories").is_some() {
        for source in array(value, "repositories")? {
            shape(source, &["repository", "commit"], &["pr"])?;
            text(source, "repository")?;
            let commit = text(source, "commit")?;
            if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|v| v.is_ascii_hexdigit()) {
                return Err("Change repository commit must be a full Git ID".into());
            }
            if source.get("pr").is_some() && !text(source, "pr")?.starts_with("https://") {
                return Err("Change PR must be an HTTPS reference".into());
            }
        }
    }
    if value.get("contributions").is_some() {
        for contribution in array(value, "contributions")? {
            shape(
                contribution,
                &["kind", "ref", "digest", "author"],
                &["agent", "model"],
            )?;
            for key in ["kind", "ref", "author"] {
                text(contribution, key)?;
            }
            digest_field(contribution, "digest")?;
            for key in ["agent", "model"] {
                if contribution.get(key).is_some() {
                    text(contribution, key)?;
                }
            }
        }
    }
    Ok(())
}

fn portable_evidence(value: &Json, at: &str, unavailable: &BTreeSet<String>) -> Result<(), String> {
    match value {
        Json::Obj(fields) => {
            let historical_artifact = value
                .get("digest")
                .and_then(Json::as_str)
                .is_some_and(|digest| unavailable.contains(digest))
                && value.get("locator").is_some()
                && value.get("media_type").is_some()
                && value.get("size_bytes").is_some();
            for (key, item) in fields {
                if key == "locator" && historical_artifact {
                    continue;
                }
                if matches!(key.as_str(), "stdout" | "stderr")
                    && item.as_str().is_some_and(|v| !v.is_empty())
                {
                    return Err(format!("{at}.{key}: full process output belongs in an unavailable supporting artifact; export at the producer boundary"));
                }
                portable_evidence(item, &format!("{at}.{key}"), unavailable)?;
            }
            Ok(())
        }
        Json::Arr(values) => {
            for (index, item) in values.iter().enumerate() {
                portable_evidence(item, &format!("{at}[{index}]"), unavailable)?;
            }
            Ok(())
        }
        _ => portable(value, at),
    }
}
