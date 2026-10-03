//! Extracted source/configuration support for Claim-first verification accounts.
//!
//! Emitters own language and deployment-config analysis. Core reads only this strict manifest;
//! declarations establish linkage, not method credibility, execution, or a positive Judgment.

use crate::diag::{validate_id, Diag};
use crate::json::{self, Json};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Clone)]
pub struct SupportArtifact {
    pub id: String,
    pub kind: String,
    pub file: String,
    pub fingerprint: String,
    pub site: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ElementSupport {
    pub claim: String,
    pub element: String,
    pub artifacts: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SupportCheck {
    pub id: String,
    pub terminal: String,
    pub artifacts: Vec<String>,
    pub elements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CaseContribution {
    pub check: String,
    pub case: String,
    pub element: String,
    pub proposition: String,
}

#[derive(Debug, Clone, Default)]
pub struct AccountSupport {
    pub path: String,
    pub artifacts: Vec<SupportArtifact>,
    pub elements: Vec<ElementSupport>,
    pub checks: Vec<SupportCheck>,
    pub contributions: Vec<CaseContribution>,
}

pub fn load(path: &Path) -> Result<AccountSupport, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diag::file(
            &display,
            format!("cannot read support manifest: {error}"),
        )]
    })?;
    let root = json::parse(&source).map_err(|error| {
        vec![Diag::file(
            &display,
            format!("malformed support JSON: {error}"),
        )]
    })?;
    parse(&display, &root)
}

pub fn parse(path: &str, root: &Json) -> Result<AccountSupport, Vec<Diag>> {
    let mut errors = Vec::new();
    fields(
        path,
        "root",
        root,
        &[
            "format",
            "version",
            "artifacts",
            "elements",
            "checks",
            "contributions",
        ],
        &mut errors,
    );
    if string(path, "root", root, "format", &mut errors) != "azimuth-account-support" {
        errors.push(Diag::expecting(
            path,
            0,
            "unsupported account support format",
            "format: azimuth-account-support",
        ));
    }
    if root.get("version").and_then(Json::as_num) != Some(1.0) {
        errors.push(Diag::expecting(
            path,
            0,
            "unsupported account support version",
            "version: 1",
        ));
    }
    let artifacts = objects(path, root, "artifacts", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("artifacts[{index}]");
            fields(
                path,
                &where_,
                item,
                &["id", "kind", "file", "fingerprint", "site"],
                &mut errors,
            );
            let id = string(path, &where_, item, "id", &mut errors);
            identifier(path, &where_, &id, true, &mut errors);
            let kind = string(path, &where_, item, "kind", &mut errors);
            if kind != "source" && kind != "configuration" {
                errors.push(Diag::expecting(
                    path,
                    0,
                    format!("{where_}.kind is `{kind}`"),
                    "source or configuration",
                ));
            }
            let file = string(path, &where_, item, "file", &mut errors);
            if !relative_path(&file) {
                errors.push(Diag::expecting(
                    path,
                    0,
                    format!("{where_}.file is not a normalized relative path"),
                    "workspace-relative file path",
                ));
            }
            let fingerprint = string(path, &where_, item, "fingerprint", &mut errors);
            if !fingerprint_ok(&fingerprint) {
                errors.push(Diag::expecting(
                    path,
                    0,
                    format!("{where_}.fingerprint is invalid"),
                    "sha256:<64 lowercase hex>",
                ));
            }
            let site = match item.get("site") {
                Some(value) => match value.as_str() {
                    Some(site) if !site.trim().is_empty() => Some(site.to_string()),
                    _ => {
                        errors.push(Diag::expecting(
                            path,
                            0,
                            format!("{where_}.site is empty or not a string"),
                            "nonempty semantic source or configuration site",
                        ));
                        None
                    }
                },
                None => None,
            };
            SupportArtifact {
                id,
                kind,
                file,
                fingerprint,
                site,
            }
        })
        .collect::<Vec<_>>();
    unique(
        path,
        "Artifact id",
        artifacts.iter().map(|item| item.id.as_str()),
        &mut errors,
    );

    let elements = objects(path, root, "elements", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("elements[{index}]");
            fields(
                path,
                &where_,
                item,
                &["claim", "element", "artifacts", "depends_on"],
                &mut errors,
            );
            let claim = string(path, &where_, item, "claim", &mut errors);
            identifier(path, &format!("{where_}.claim"), &claim, false, &mut errors);
            let element = string(path, &where_, item, "element", &mut errors);
            qualified(path, &format!("{where_}.element"), &element, &mut errors);
            let artifacts = strings(path, &where_, item, "artifacts", true, &mut errors);
            let depends_on = strings(path, &where_, item, "depends_on", false, &mut errors);
            for dependency in &depends_on {
                qualified(
                    path,
                    &format!("{where_}.depends_on"),
                    dependency,
                    &mut errors,
                );
            }
            ElementSupport {
                claim,
                element,
                artifacts,
                depends_on,
            }
        })
        .collect::<Vec<_>>();

    let checks = objects(path, root, "checks", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("checks[{index}]");
            fields(
                path,
                &where_,
                item,
                &["id", "terminal", "artifacts", "elements"],
                &mut errors,
            );
            let id = string(path, &where_, item, "id", &mut errors);
            identifier(path, &format!("{where_}.id"), &id, false, &mut errors);
            let terminal = string(path, &where_, item, "terminal", &mut errors);
            if terminal.trim().is_empty() {
                errors.push(Diag::expecting(
                    path,
                    0,
                    format!("{where_}.terminal is empty"),
                    "one independently decidable terminal proposition",
                ));
            }
            let artifacts = strings(path, &where_, item, "artifacts", true, &mut errors);
            let elements = strings(path, &where_, item, "elements", true, &mut errors);
            for element in &elements {
                qualified(path, &format!("{where_}.elements"), element, &mut errors);
            }
            SupportCheck {
                id,
                terminal,
                artifacts,
                elements,
            }
        })
        .collect::<Vec<_>>();
    unique(
        path,
        "Check id",
        checks.iter().map(|item| item.id.as_str()),
        &mut errors,
    );

    let contributions = objects(path, root, "contributions", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("contributions[{index}]");
            fields(
                path,
                &where_,
                item,
                &["check", "case", "element", "proposition"],
                &mut errors,
            );
            let check = string(path, &where_, item, "check", &mut errors);
            identifier(path, &format!("{where_}.check"), &check, false, &mut errors);
            let case = string(path, &where_, item, "case", &mut errors);
            case_id(path, &format!("{where_}.case"), &case, &mut errors);
            let element = string(path, &where_, item, "element", &mut errors);
            qualified(path, &format!("{where_}.element"), &element, &mut errors);
            let proposition = string(path, &where_, item, "proposition", &mut errors);
            if proposition.trim().is_empty() {
                errors.push(Diag::expecting(
                    path,
                    0,
                    format!("{where_}.proposition is empty"),
                    "why this Check bears on this Case",
                ));
            }
            CaseContribution {
                check,
                case,
                element,
                proposition,
            }
        })
        .collect::<Vec<_>>();
    unique(
        path,
        "Check-to-Case contribution",
        contributions
            .iter()
            .map(|item| format!("{}|{}", item.check, item.case))
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str),
        &mut errors,
    );

    let ids = artifacts
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    for element in &elements {
        for artifact in &element.artifacts {
            if !ids.contains(artifact.as_str()) {
                errors.push(Diag::at(
                    path,
                    0,
                    format!(
                        "Element support `{}` names unknown Artifact `{artifact}`",
                        element.element
                    ),
                ));
            }
        }
    }
    for check in &checks {
        for artifact in &check.artifacts {
            if !ids.contains(artifact.as_str()) {
                errors.push(Diag::at(
                    path,
                    0,
                    format!("Check `{}` names unknown Artifact `{artifact}`", check.id),
                ));
            }
        }
    }
    let check_ids = checks
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    for contribution in &contributions {
        if !check_ids.contains(contribution.check.as_str()) {
            errors.push(Diag::at(
                path,
                0,
                format!(
                    "Case contribution names unknown Check `{}`",
                    contribution.check
                ),
            ));
        }
        if !checks.iter().any(|check| {
            check.id == contribution.check && check.elements.contains(&contribution.element)
        }) {
            errors.push(Diag::at(
                path,
                0,
                format!(
                    "Case contribution `{}` does not use Check element `{}`",
                    contribution.check, contribution.element
                ),
            ));
        }
    }
    if errors.is_empty() {
        Ok(AccountSupport {
            path: path.to_string(),
            artifacts,
            elements,
            checks,
            contributions,
        })
    } else {
        Err(errors)
    }
}

fn fields(path: &str, where_: &str, value: &Json, allowed: &[&str], errors: &mut Vec<Diag>) {
    let Json::Obj(pairs) = value else {
        errors.push(Diag::expecting(
            path,
            0,
            format!("{where_} is not an object"),
            "JSON object",
        ));
        return;
    };
    let mut seen = BTreeSet::new();
    for (key, _) in pairs {
        if !seen.insert(key) {
            errors.push(Diag::at(path, 0, format!("{where_} repeats key `{key}`")));
        }
        if !allowed.contains(&key.as_str()) {
            errors.push(Diag::at(
                path,
                0,
                format!("{where_} has unknown key `{key}`"),
            ));
        }
    }
}

fn objects<'a>(path: &str, root: &'a Json, key: &str, errors: &mut Vec<Diag>) -> Vec<&'a Json> {
    match root.get(key) {
        None => Vec::new(),
        Some(Json::Arr(items)) => items.iter().collect(),
        _ => {
            errors.push(Diag::expecting(
                path,
                0,
                format!("{key} is not an array"),
                "JSON array of objects",
            ));
            Vec::new()
        }
    }
}

fn string(path: &str, where_: &str, root: &Json, key: &str, errors: &mut Vec<Diag>) -> String {
    match root.get(key).and_then(Json::as_str) {
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => {
            errors.push(Diag::expecting(
                path,
                0,
                format!("{where_}.{key} is missing or empty"),
                "nonempty string",
            ));
            String::new()
        }
    }
}

fn strings(
    path: &str,
    where_: &str,
    root: &Json,
    key: &str,
    required: bool,
    errors: &mut Vec<Diag>,
) -> Vec<String> {
    let Some(value) = root.get(key) else {
        if required {
            errors.push(Diag::expecting(
                path,
                0,
                format!("{where_}.{key} is missing"),
                "nonempty string array",
            ));
        }
        return Vec::new();
    };
    let Json::Arr(items) = value else {
        errors.push(Diag::expecting(
            path,
            0,
            format!("{where_}.{key} is not an array"),
            "string array",
        ));
        return Vec::new();
    };
    if required && items.is_empty() {
        errors.push(Diag::expecting(
            path,
            0,
            format!("{where_}.{key} is empty"),
            "nonempty string array",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut values = Vec::new();
    for (index, item) in items.iter().enumerate() {
        match item.as_str() {
            Some(value) if !value.trim().is_empty() => {
                if !seen.insert(value) {
                    errors.push(Diag::at(
                        path,
                        0,
                        format!("{where_}.{key}[{index}] duplicates `{value}`"),
                    ));
                }
                values.push(value.to_string());
            }
            _ => errors.push(Diag::expecting(
                path,
                0,
                format!("{where_}.{key}[{index}] is not a nonempty string"),
                "nonempty string",
            )),
        }
    }
    values
}

fn identifier(path: &str, where_: &str, value: &str, slash: bool, errors: &mut Vec<Diag>) {
    if let Err(reason) = validate_id(value, slash) {
        errors.push(Diag::at(path, 0, format!("{where_}: {reason}")));
    }
}

fn qualified(path: &str, where_: &str, value: &str, errors: &mut Vec<Diag>) {
    let Some((module, local)) = value.split_once('#') else {
        errors.push(Diag::expecting(
            path,
            0,
            format!("{where_} is not qualified: `{value}`"),
            "<module>#<local-id>",
        ));
        return;
    };
    identifier(path, where_, module, true, errors);
    identifier(path, where_, local, false, errors);
}

fn case_id(path: &str, where_: &str, value: &str, errors: &mut Vec<Diag>) {
    identifier(path, where_, value, false, errors);
}

fn unique<'a>(path: &str, kind: &str, ids: impl Iterator<Item = &'a str>, errors: &mut Vec<Diag>) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            errors.push(Diag::at(path, 0, format!("duplicate {kind} `{id}`")));
        }
    }
}

fn fingerprint_ok(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\\')
        && !value.contains("//")
        && !value.ends_with('/')
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}
