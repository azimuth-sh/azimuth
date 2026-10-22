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
fn optional_array<'a>(value: &'a Json, name: &str) -> Result<&'a [Json], String> {
    match value.get(name) {
        None => Ok(&[]),
        Some(_) => array(value, name),
    }
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

pub struct UnitInputsSource<'a> {
    pub rule: Option<&'a std::path::Path>,
    pub checkout: &'a std::path::Path,
    pub repository: Option<&'a str>,
}

/// Computes the release unit-input manifest for one source checkout. Without a partition rule the
/// request is returned unchanged, because unit reuse is opt-in.
pub fn with_unit_inputs(request: &Json, source: &UnitInputsSource) -> Result<Json, String> {
    let default_rule = source.checkout.join("azimuth/standards/units.json");
    let rule_path = match source.rule {
        Some(path) => path,
        None if default_rule.is_file() => &default_rule,
        None => return Ok(request.clone()),
    };
    if request.get("unit_inputs").is_some() {
        return Err("request already carries unit_inputs; remove it or drop the unit rule".into());
    }
    let repository = match source.repository {
        Some(repository) => repository.to_string(),
        None => match array(request, "sources")? {
            [only] => text(only, "repository")?.to_string(),
            _ => return Err("release has several sources; name the checkout's with --repository".into()),
        },
    };
    if !array(request, "sources")?
        .iter()
        .any(|source| text(source, "repository").is_ok_and(|id| id == repository))
    {
        return Err(format!("`{repository}` is not a release source"));
    }
    let rule = crate::unit_inputs::load_rule(rule_path)?;
    let files = crate::unit_inputs::read_checkout(source.checkout)?;
    let inputs = crate::unit_inputs::compute_inputs(&rule, &files);
    let Json::Obj(mut pairs) = request.clone() else {
        return Err("expected object".into());
    };
    pairs.push((
        "unit_inputs".to_string(),
        crate::unit_inputs::release_manifest_json(&inputs, &repository),
    ));
    Ok(Json::Obj(pairs))
}

pub fn prepare(request: &Json) -> Result<Json, String> {
    shape(
        request,
        &[
            "format",
            "version",
            "project",
            "label",
            "sources",
            "builds",
            "artifacts",
            "deployments",
            "snapshot",
            "evidence",
        ],
        &[
            "baseline_model_revision",
            "changes",
            "integration",
            "unit_inputs",
            "catalog",
            "composition",
            "required_reviews",
            "components",
        ],
    )?;
    if text(request, "format")? != "azimuth-release-request"
        || field(request, "version")?.as_num() != Some(2.0)
    {
        return Err("expected azimuth-release-request version 2".into());
    }
    let mut release = vec![
        ("label", field(request, "label")?.clone()),
        (
            "model_revision",
            field(field(request, "snapshot")?, "model_revision")?.clone(),
        ),
    ];
    for key in ["baseline_model_revision", "changes", "integration", "unit_inputs"] {
        if let Some(value) = request.get(key) {
            release.push((key, value.clone()));
        }
    }
    for key in ["sources", "builds", "artifacts", "deployments"] {
        release.push((key, field(request, key)?.clone()));
    }
    if let Some(components) = request.get("components") {
        release.push(("components", components.clone()));
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
        ("version", Json::Num(2.0)),
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
        || field(value, "version")?.as_num() != Some(2.0)
    {
        return Err("expected azimuth-release-publication version 2".into());
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
            "model_revision",
            "sources",
            "builds",
            "artifacts",
            "deployments",
        ],
        &[
            "baseline_model_revision",
            "changes",
            "integration",
            "unit_inputs",
            "composition",
            "required_reviews",
            "components",
        ],
    )?;
    let manifest_repository = match release.get("unit_inputs") {
        Some(manifest) => crate::unit_inputs::validate_release_manifest(manifest)?,
        None => None,
    };
    id(release, "label")?;
    if release.get("baseline_model_revision").is_some() {
        digest_field(release, "baseline_model_revision")?;
    }
    if let Some(integration) = release.get("integration") {
        verify_integration(integration)?;
    }
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
    for change in optional_array(release, "changes")? {
        shape(change, &["id", "revision"], &["metadata"])?;
        if let Some(metadata) = change.get("metadata") {
            verify_change_metadata(metadata)?;
        }
        digest_field(change, "revision")?;
        if !change_ids.insert(id(change, "id")?) {
            return Err("duplicate Change identity".into());
        }
    }
    if let Some(components) = release.get("components") {
        let components = components.as_array().ok_or("components requires array")?;
        let mut ids = BTreeSet::new();
        let artifacts = array(release, "artifacts")?;
        for component in components {
            shape(component, &["id", "digest"], &[])?;
            crate::diag::validate_id(text(component, "id")?, false)?;
            digest_field(component, "digest")?;
            if !ids.insert(text(component, "id")?)
                || !artifacts
                    .iter()
                    .any(|a| a.get("digest") == component.get("digest"))
            {
                return Err(
                    "component must have a unique identity and declared artifact digest".into(),
                );
            }
        }
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
        if !is_git_id(commit) {
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
    if let Some(repository) = manifest_repository {
        if !repositories.contains(&repository) {
            return Err("unit input manifest repository is not a release source".into());
        }
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
            if text(record, "kind")? != "requirement-review" {
                return Err(
                    "required_reviews selects Requirement Review records, not Requirement Status"
                        .into(),
                );
            }
            covered.insert(text(record, "requirement")?.to_string());
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
        for requirement in array(field(snapshot, "authority")?, "requirements")? {
            if text(requirement, "criticality")? != "routine"
                && !covered.contains(text(requirement, "id")?)
            {
                return Err(format!("required_reviews omits non-routine Requirement `{}`; include its latest accepted current review", text(requirement, "id")?));
            }
        }
    }
    if let Some(composition) = release.get("composition") {
        verify_composition(composition)?;
        if release.get("baseline_model_revision").is_none()
            || composition.get("baseline_model_revision") != release.get("baseline_model_revision")
            || change_pairs(composition)? != change_pairs(release)?
        {
            return Err("release composition baseline/Changes differ from release".into());
        }
    }
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-release-validation")),
        ("version", Json::Num(2.0)),
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
        &["verification_target", "verification_target_fingerprint"],
    )?;
    id(value, "id")?;
    id(value, "environment")?;
    digest_field(value, "configuration_fingerprint")?;
    let deployed = strings(value, "artifacts")?;
    if deployed.is_empty() || !deployed.is_subset(artifacts) {
        return Err("deployment must identify registered artifact digests".into());
    }
    match (value.get("verification_target"), value.get("verification_target_fingerprint")) {
        (None, None) => {}
        (Some(verification_target), Some(fingerprint)) => {
            let native = crate::run::verification_target_from_json(verification_target)?;
            if !crate::run::validate_verification_target_component(&native).is_empty() {
                return Err("deployment Verification Target is invalid".into());
            }
            if fingerprint.as_str() != Some(crate::run::verification_target_fingerprint(&native).as_str()) {
                return Err("deployment Verification Target fingerprint mismatch".into());
            }
            let crate::run::VerificationTarget::Deployment {
                environment,
                deployment,
                artifacts,
                ..
            } = native
            else {
                return Err("deployment requires a native deployment Verification Target".into());
            };
            let digests = artifacts
                .iter()
                .map(|artifact| artifact.digest.clone())
                .collect::<BTreeSet<_>>();
            if environment != text(value, "environment")?
                || deployment != text(value, "id")?
                || digests != deployed
            {
                return Err("deployment Verification Target must describe the recorded environment, deployment and exact artifact digests".into());
            }
        }
        _ => {
            return Err(
                "deployment verification target and verification_target_fingerprint must be supplied together".into(),
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

pub fn verify_catalog(catalog: &Json, authority: &Json) -> Result<(), String> {
    shape(
        catalog,
        &["modules", "requirements", "scenarios", "checks"],
        &[],
    )?;
    let mut members = BTreeSet::new();
    let mut modules = BTreeMap::new();
    for module in array(catalog, "modules")? {
        shape(module, &["id", "title", "requirements"], &["parent"])?;
        let module_id = id(module, "id")?;
        id(module, "title")?;
        let requirements = strings(module, "requirements")?;
        for requirement in requirements {
            if !members.insert(requirement) {
                return Err("Requirement belongs to multiple catalog Modules".into());
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
    for key in ["requirements", "scenarios", "checks"] {
        let mut actual = BTreeSet::new();
        for item in array(catalog, key)? {
            shape(
                item,
                &[
                    "id",
                    if key == "requirements" {
                        "statement"
                    } else {
                        "description"
                    },
                ],
                if key == "requirements" {
                    &["title"]
                } else {
                    &[]
                },
            )?;
            if let Some(title) = item.get("title") {
                let value = title
                    .as_str()
                    .ok_or("catalog Requirement title must be text")?;
                if value.trim().is_empty()
                    || value.chars().count() > 120
                    || value.contains(['\n', '\r'])
                {
                    return Err("catalog Requirement title must contain 1 to 120 characters".into());
                }
            }
            text(
                item,
                if key == "requirements" {
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
        if actual != expected || (key == "requirements" && members != expected) {
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
                                "requirements",
                                Json::Arr(
                                    spec.requirements
                                        .iter()
                                        .map(|requirement| Json::str(&requirement.id))
                                        .collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "requirements",
            Json::Arr(
                model
                    .requirements()
                    .map(|view| {
                        let mut fields = vec![
                            ("id", Json::str(&view.requirement.id)),
                            ("statement", Json::str(&view.requirement.statement)),
                        ];
                        if let Some(title) = &view.requirement.title {
                            fields.push(("title", Json::str(title)));
                        }
                        Json::obj(fields)
                    })
                    .collect(),
            ),
        ),
        (
            "scenarios",
            Json::Arr(
                model
                    .scenarios()
                    .map(|view| {
                        Json::obj(vec![
                            ("id", Json::str(&view.scenario.id)),
                            ("description", Json::str(&view.scenario.statement)),
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

fn change_pairs(value: &Json) -> Result<BTreeMap<String, String>, String> {
    optional_array(value, "changes")?
        .iter()
        .map(|change| {
            Ok((
                text(change, "id")?.to_string(),
                text(change, "revision")?.to_string(),
            ))
        })
        .collect()
}
/// Lowercase only, as `git rev-parse` prints and the service stores, so one commit has one spelling.
fn is_git_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|v| matches!(v, b'0'..=b'9' | b'a'..=b'f'))
}
fn verify_integration(value: &Json) -> Result<(), String> {
    shape(value, &["commit", "main_sequence"], &[])?;
    let commit = text(value, "commit")?;
    if !is_git_id(commit) {
        return Err("integration commit must be a full lowercase Git ID".into());
    }
    match field(value, "main_sequence")?.as_num() {
        Some(sequence) if sequence >= 1.0 && sequence.fract() == 0.0 => Ok(()),
        _ => Err("integration main_sequence must be a positive integer".into()),
    }
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
            if !is_git_id(commit) {
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

pub fn publication_inputs(value: &Json) -> Result<Json, String> {
    shape(value, &["format", "version", "request"], &["composition"])?;
    if text(value, "format")? != "azimuth-publication-inputs"
        || field(value, "version")?.as_num() != Some(1.0)
    {
        return Err("expected azimuth-publication-inputs version 1".into());
    }
    let request = field(value, "request")?;
    for key in ["builds", "artifacts", "deployments"] {
        if !array(request, key)?.is_empty() {
            return Err("offline inputs cannot assert CI builds, artifacts or deployments".into());
        }
    }
    array(request, "required_reviews")?;
    if request.get("composition") != value.get("composition") {
        return Err("request must retain the exact composed candidate".into());
    }
    prepare(request)?;
    if value.to_string_pretty().len() > PACKAGE_LIMIT {
        return Err(
            "publication inputs exceed 8 MiB; compact producer records without rewriting history"
                .into(),
        );
    }
    Ok(value.clone())
}
