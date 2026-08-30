//! Strict source-linkage manifest reader for the alpha 5 model.

use crate::diag::{validate_id, Diag};
use crate::json::{self, Json};
use crate::model::{
    Artifact, CheckImplementation, ClassMember, Enumeration, MechanismImplementation, Site,
    SourceIdentity,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

const TOP_LEVEL: &[&str] = &[
    "realizes",
    "check_implementations",
    "mechanism_implementations",
    "class_members",
    "enumerations",
    "artifacts",
];
const SOURCE_FIELDS: &[&str] = &["area", "address_kind", "address", "mount"];

#[derive(Debug, Default, Clone)]
pub struct Manifest {
    pub realizes: Vec<Site>,
    pub check_implementations: Vec<CheckImplementation>,
    pub mechanism_implementations: Vec<MechanismImplementation>,
    pub class_members: Vec<ClassMember>,
    pub enumerations: Vec<Enumeration>,
    pub artifacts: Vec<Artifact>,
}

pub fn load(path: &Path) -> Result<Manifest, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diag::file(
            &display,
            format!("cannot read manifest: {error}"),
        )]
    })?;
    let root = json::parse(&source)
        .map_err(|error| vec![Diag::file(&display, format!("malformed manifest: {error}"))])?;
    parse(&display, &root)
}

pub fn parse(path: &str, root: &Json) -> Result<Manifest, Vec<Diag>> {
    let Json::Obj(fields) = root else {
        return Err(vec![Diag::expecting(
            path,
            0,
            "manifest root is not an object",
            "an object containing source-linkage arrays",
        )]);
    };
    let mut errors = Vec::new();
    let mut top_level_keys = BTreeSet::new();
    for (key, _) in fields {
        if !top_level_keys.insert(key) {
            errors.push(Diag::at(
                path,
                0,
                format!("manifest repeats top-level key `{key}`"),
            ));
        }
        if !TOP_LEVEL.contains(&key.as_str()) {
            let message = match key.as_str() {
                "covers" | "mechanism_covers" | "observations" => format!(
                    "legacy manifest key `{key}` is not supported; use Check implementations and \
                     repository verification declarations"
                ),
                _ => format!("unknown manifest key `{key}`"),
            };
            errors.push(Diag::at(path, 0, message));
        }
    }
    if TOP_LEVEL.iter().all(|key| root.get(key).is_none()) {
        errors.push(Diag::expecting(
            path,
            0,
            "manifest declares no source linkage",
            "at least one current source-linkage array",
        ));
    }

    let mut out = Manifest::default();
    let mut realizes = BTreeSet::new();
    for_each(
        path,
        root,
        "realizes",
        &mut errors,
        |where_, item, errors| {
            reject_unknown_fields(
                path,
                where_,
                item,
                &with_source(&[
                    "spec",
                    "claim",
                    "site",
                    "file",
                    "lang",
                    "source_fingerprint",
                ]),
                errors,
            );
            let value = Site {
                spec: required_string(path, where_, item, "spec", errors),
                claim: required_string(path, where_, item, "claim", errors),
                site: required_string(path, where_, item, "site", errors),
                file: required_string(path, where_, item, "file", errors),
                lang: required_string(path, where_, item, "lang", errors),
                source: source_identity(path, where_, item, errors),
                source_fingerprint: optional_fingerprint(
                    path,
                    where_,
                    item,
                    "source_fingerprint",
                    errors,
                )
                .unwrap_or_default(),
            };
            let identity = format!(
                "{}|{}|{}|{}|{}",
                value.spec, value.claim, value.site, value.file, value.lang
            );
            if realizes.insert(identity.clone()) {
                out.realizes.push(value);
            } else {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("duplicate realization `{identity}`"),
                ));
            }
        },
    );

    let mut checks = BTreeSet::new();
    for_each(
        path,
        root,
        "check_implementations",
        &mut errors,
        |where_, item, errors| {
            reject_unknown_fields(
                path,
                where_,
                item,
                &with_source(&["check", "site", "file", "lang", "source_fingerprint"]),
                errors,
            );
            let check = required_string(path, where_, item, "check", errors);
            if let Err(reason) = validate_id(&check, true) {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("{where_} has invalid Check id: {reason}"),
                ));
            }
            let source = source_identity(path, where_, item, errors);
            let value = CheckImplementation {
                check,
                site: required_string(path, where_, item, "site", errors),
                file: required_string(path, where_, item, "file", errors),
                lang: required_string(path, where_, item, "lang", errors),
                source,
                source_fingerprint: required_fingerprint(
                    path,
                    where_,
                    item,
                    "source_fingerprint",
                    errors,
                ),
            };
            let identity = value
                .source
                .as_ref()
                .map(|source| format!("{}|{}", value.check, source.key()))
                .unwrap_or_else(|| {
                    format!(
                        "{}|{}|{}|{}",
                        value.check, value.file, value.site, value.lang
                    )
                });
            if checks.insert(identity.clone()) {
                out.check_implementations.push(value);
            } else {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("duplicate Check implementation `{identity}`"),
                ));
            }
        },
    );

    let mut mechanisms = BTreeSet::new();
    for_each(
        path,
        root,
        "mechanism_implementations",
        &mut errors,
        |where_, item, errors| {
            reject_unknown_fields(
                path,
                where_,
                item,
                &with_source(&[
                    "spec",
                    "mechanism",
                    "site",
                    "binding",
                    "file",
                    "lang",
                    "source_fingerprint",
                ]),
                errors,
            );
            let value = MechanismImplementation {
                spec: required_string(path, where_, item, "spec", errors),
                mechanism: required_string(path, where_, item, "mechanism", errors),
                site: required_string(path, where_, item, "site", errors),
                binding: required_string(path, where_, item, "binding", errors),
                file: required_string(path, where_, item, "file", errors),
                lang: required_string(path, where_, item, "lang", errors),
                source: source_identity(path, where_, item, errors),
                source_fingerprint: required_fingerprint(
                    path,
                    where_,
                    item,
                    "source_fingerprint",
                    errors,
                ),
            };
            let identity = format!(
                "{}|{}|{}|{}|{}|{}",
                value.spec, value.mechanism, value.site, value.binding, value.file, value.lang
            );
            if mechanisms.insert(identity.clone()) {
                out.mechanism_implementations.push(value);
            } else {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("duplicate mechanism implementation `{identity}`"),
                ));
            }
        },
    );

    let mut members = BTreeSet::new();
    for_each(
        path,
        root,
        "class_members",
        &mut errors,
        |where_, item, errors| {
            reject_unknown_fields(
                path,
                where_,
                item,
                &with_source(&["class", "site", "file", "lang"]),
                errors,
            );
            let value = ClassMember {
                class: required_string(path, where_, item, "class", errors),
                site: required_string(path, where_, item, "site", errors),
                file: required_string(path, where_, item, "file", errors),
                lang: required_string(path, where_, item, "lang", errors),
                source: source_identity(path, where_, item, errors),
            };
            let identity = format!(
                "{}|{}|{}|{}",
                value.class, value.site, value.file, value.lang
            );
            if members.insert(identity.clone()) {
                out.class_members.push(value);
            } else {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("duplicate class member `{identity}`"),
                ));
            }
        },
    );

    let mut enumerations = BTreeSet::new();
    for_each(
        path,
        root,
        "enumerations",
        &mut errors,
        |where_, item, errors| {
            reject_unknown_fields(
                path,
                where_,
                item,
                &with_source(&["class", "kind", "source", "source_fingerprint"]),
                errors,
            );
            let value = Enumeration {
                class: required_string(path, where_, item, "class", errors),
                kind: required_string(path, where_, item, "kind", errors),
                source: required_string(path, where_, item, "source", errors),
                source_fingerprint: required_fingerprint(
                    path,
                    where_,
                    item,
                    "source_fingerprint",
                    errors,
                ),
                identity: source_identity(path, where_, item, errors),
            };
            let identity = format!("{}|{}|{}", value.class, value.kind, value.source);
            if enumerations.insert(identity.clone()) {
                out.enumerations.push(value);
            } else {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("duplicate enumeration `{identity}`"),
                ));
            }
        },
    );

    let mut artifacts = BTreeSet::new();
    for_each(
        path,
        root,
        "artifacts",
        &mut errors,
        |where_, item, errors| {
            reject_unknown_fields(
                path,
                where_,
                item,
                &with_source(&["id", "kind", "file", "unique", "columns", "predicate"]),
                errors,
            );
            let id = required_string(path, where_, item, "id", errors);
            let value = Artifact {
                id: id.clone(),
                kind: required_string(path, where_, item, "kind", errors),
                file: required_string(path, where_, item, "file", errors),
                unique: optional_bool(path, where_, item, "unique", errors),
                columns: optional_string_array(path, where_, item, "columns", errors),
                predicate: optional_string(path, where_, item, "predicate", errors),
                source: source_identity(path, where_, item, errors),
            };
            let identity = (id.clone(), value.kind.clone(), value.file.clone());
            if artifacts.insert(identity.clone()) {
                out.artifacts.push(value);
            } else {
                errors.push(Diag::at(
                    path,
                    0,
                    format!(
                        "duplicate artifact companion `{}|{}|{}`",
                        identity.0, identity.1, identity.2
                    ),
                ));
            }
        },
    );

    validate_mechanism_accounts(path, &out, &mut errors);

    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors)
    }
}

fn validate_mechanism_accounts(path: &str, manifest: &Manifest, errors: &mut Vec<Diag>) {
    let mut owned_artifacts = BTreeSet::new();
    let mut reserved_raw_ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for implementation in &manifest.mechanism_implementations {
        if let Err(reason) = validate_id(&implementation.spec, true) {
            errors.push(Diag::file(
                path,
                format!("mechanism implementation has invalid spec id: {reason}"),
            ));
        }
        if let Err(reason) = validate_id(&implementation.mechanism, false) {
            errors.push(Diag::file(
                path,
                format!("mechanism implementation has invalid mechanism id: {reason}"),
            ));
        }
        if !normalized_relative_path(&implementation.file) {
            errors.push(Diag::file(
                path,
                "mechanism implementation `file` must be a normalized workspace-relative path",
            ));
        }
        if !targets.insert((
            implementation.spec.clone(),
            implementation.mechanism.clone(),
        )) {
            errors.push(Diag::file(
                path,
                format!(
                    "multiple marker implementations for mechanism `#{}`",
                    implementation.mechanism
                ),
            ));
        }
        if implementation.site.trim() != implementation.site
            || implementation.site.is_empty()
            || implementation.site.contains('|')
            || implementation.site.chars().any(char::is_control)
        {
            errors.push(Diag::file(
                path,
                "mechanism implementation `site` must be one trimmed path-free semantic identity",
            ));
        }
        let Some(expected_kind) = mechanism_address_kind(&implementation.lang) else {
            errors.push(Diag::file(
                path,
                format!(
                    "mechanism implementation has unsupported language `{}`",
                    implementation.lang
                ),
            ));
            continue;
        };
        reserved_raw_ids.insert(format!("{expected_kind}:{}", implementation.site));
        let (binding, companion_id) = if let Some(source) = &implementation.source {
            if source.kind != expected_kind || source.address != implementation.site {
                errors.push(Diag::file(
                    path,
                    "assembled mechanism source identity disagrees with `lang` or `site`",
                ));
            }
            let key = source.key();
            (key.clone(), key)
        } else {
            let expected = format!("{expected_kind}:{}", implementation.site);
            if implementation.binding != expected
                || implementation.binding.contains('|')
                || implementation.binding.split_once(':').is_none()
                || implementation.site.contains('#')
            {
                errors.push(Diag::file(
                    path,
                    format!(
                        "mechanism implementation binding must be exact typed `{expected_kind}:<site>` with suffix equal to `site`"
                    ),
                ));
            }
            (expected.clone(), expected)
        };
        if implementation.binding != binding {
            errors.push(Diag::file(
                path,
                "mechanism implementation binding disagrees with its semantic source identity",
            ));
        }
        let matches = manifest
            .artifacts
            .iter()
            .enumerate()
            .filter(|(_, artifact)| {
                artifact.id == companion_id
                    && artifact.kind == expected_kind
                    && artifact.file == implementation.file
                    && match (&implementation.source, &artifact.source) {
                        (Some(left), Some(right)) => left == right,
                        (None, None) => true,
                        _ => false,
                    }
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let [artifact_index] = matches.as_slice() else {
            errors.push(Diag::file(
                path,
                format!(
                    "mechanism implementation `{}` requires exactly one companion Artifact `{companion_id}|{expected_kind}|{}`",
                    implementation.mechanism, implementation.file
                ),
            ));
            continue;
        };
        if !owned_artifacts.insert(*artifact_index) {
            errors.push(Diag::file(
                path,
                format!("marker companion Artifact `{companion_id}` has multiple targets"),
            ));
        }
    }

    let owned_ids = owned_artifacts
        .iter()
        .map(|index| manifest.artifacts[*index].id.as_str())
        .collect::<BTreeSet<_>>();
    let mut ordinary_ids = BTreeSet::new();
    for (index, artifact) in manifest.artifacts.iter().enumerate() {
        if !owned_artifacts.contains(&index) {
            if owned_ids.contains(artifact.id.as_str())
                || reserved_raw_ids.contains(artifact.id.as_str())
            {
                errors.push(Diag::file(
                    path,
                    format!(
                        "marker companion Artifact `{}` collides with an ordinary Artifact",
                        artifact.id
                    ),
                ));
            } else if !ordinary_ids.insert(&artifact.id) {
                errors.push(Diag::file(
                    path,
                    format!("duplicate ordinary artifact id `{}`", artifact.id),
                ));
            }
        }
    }
}

fn normalized_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && !value.contains('\\')
}

pub(crate) fn mechanism_address_kind(language: &str) -> Option<String> {
    match language {
        "csharp" => Some("dotnet-symbol".into()),
        "cpp" | "go" | "java" | "javascript" | "kotlin" | "python" | "rust" | "typescript" => {
            Some(format!("{language}-symbol"))
        }
        _ => None,
    }
}

fn for_each(
    path: &str,
    root: &Json,
    key: &str,
    errors: &mut Vec<Diag>,
    mut parse: impl FnMut(&str, &Json, &mut Vec<Diag>),
) {
    let Some(value) = root.get(key) else { return };
    let Some(items) = value.as_array() else {
        errors.push(Diag::expecting(
            path,
            0,
            format!("`{key}` is not an array"),
            "an array",
        ));
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let where_ = format!("{key}[{index}]");
        if !matches!(item, Json::Obj(_)) {
            errors.push(Diag::at(path, 0, format!("{where_} is not an object")));
            continue;
        }
        parse(&where_, item, errors);
    }
}

fn with_source(fields: &[&'static str]) -> Vec<&'static str> {
    fields
        .iter()
        .copied()
        .chain(SOURCE_FIELDS.iter().copied())
        .collect()
}

fn reject_unknown_fields(
    path: &str,
    where_: &str,
    item: &Json,
    allowed: &[&str],
    errors: &mut Vec<Diag>,
) {
    let Json::Obj(fields) = item else { return };
    let mut seen = BTreeSet::new();
    for (key, _) in fields {
        if !allowed.contains(&key.as_str()) {
            errors.push(Diag::at(
                path,
                0,
                format!("{where_} has unknown field `{key}`"),
            ));
        }
        if !seen.insert(key) {
            errors.push(Diag::at(path, 0, format!("{where_} repeats field `{key}`")));
        }
    }
}

fn required_string(
    path: &str,
    where_: &str,
    item: &Json,
    key: &str,
    errors: &mut Vec<Diag>,
) -> String {
    match item.get(key).and_then(Json::as_str) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => {
            errors.push(Diag::expecting(
                path,
                0,
                format!("{where_} is missing non-empty `{key}`"),
                format!("a non-empty string `{key}`"),
            ));
            String::new()
        }
    }
}

fn optional_string(
    path: &str,
    where_: &str,
    item: &Json,
    key: &str,
    errors: &mut Vec<Diag>,
) -> Option<String> {
    match item.get(key) {
        Some(value) => match value.as_str() {
            Some(value) => Some(value.to_string()),
            None => {
                errors.push(Diag::at(path, 0, format!("{where_}.{key} is not a string")));
                None
            }
        },
        None => None,
    }
}

fn required_fingerprint(
    path: &str,
    where_: &str,
    item: &Json,
    key: &str,
    errors: &mut Vec<Diag>,
) -> String {
    let value = required_string(path, where_, item, key, errors);
    validate_fingerprint(path, where_, key, &value, errors);
    value
}

fn optional_fingerprint(
    path: &str,
    where_: &str,
    item: &Json,
    key: &str,
    errors: &mut Vec<Diag>,
) -> Option<String> {
    let value = optional_string(path, where_, item, key, errors)?;
    validate_fingerprint(path, where_, key, &value, errors);
    Some(value)
}

fn validate_fingerprint(path: &str, where_: &str, key: &str, value: &str, errors: &mut Vec<Diag>) {
    let valid = value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    });
    if !valid {
        errors.push(Diag::expecting(
            path,
            0,
            format!("{where_}.{key} is not a SHA-256 fingerprint"),
            "sha256: followed by 64 lowercase hexadecimal digits",
        ));
    }
}

fn source_identity(
    path: &str,
    where_: &str,
    item: &Json,
    errors: &mut Vec<Diag>,
) -> Option<SourceIdentity> {
    let present = SOURCE_FIELDS
        .iter()
        .filter(|field| item.get(field).is_some())
        .count();
    if present == 0 {
        return None;
    }
    if present != SOURCE_FIELDS.len() {
        errors.push(Diag::expecting(
            path,
            0,
            format!("{where_} has a partial source identity"),
            "`area`, `address_kind`, `address`, and `mount` together",
        ));
        return None;
    }
    Some(SourceIdentity {
        area: required_string(path, where_, item, "area", errors),
        kind: required_string(path, where_, item, "address_kind", errors),
        address: required_string(path, where_, item, "address", errors),
        mount: required_string(path, where_, item, "mount", errors),
    })
}

fn optional_bool(
    path: &str,
    where_: &str,
    item: &Json,
    key: &str,
    errors: &mut Vec<Diag>,
) -> Option<bool> {
    match item.get(key) {
        Some(value) => match value.as_bool() {
            Some(value) => Some(value),
            None => {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("{where_}.{key} is not a boolean"),
                ));
                None
            }
        },
        None => None,
    }
}

fn optional_string_array(
    path: &str,
    where_: &str,
    item: &Json,
    key: &str,
    errors: &mut Vec<Diag>,
) -> Vec<String> {
    let Some(value) = item.get(key) else {
        return Vec::new();
    };
    let Some(values) = value.as_array() else {
        errors.push(Diag::at(path, 0, format!("{where_}.{key} is not an array")));
        return Vec::new();
    };
    values
        .iter()
        .filter_map(|value| match value.as_str() {
            Some(value) => Some(value.to_string()),
            None => {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("{where_}.{key} contains a non-string"),
                ));
                None
            }
        })
        .collect()
}
