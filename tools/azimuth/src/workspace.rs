//! Local architectural declarations used by extraction and checking.
//!
//! Federation already establishes `area + mount` as the stable ownership vocabulary. The local
//! workspace reuses that shape so a monorepo does not invent a second meaning for an area merely
//! because all of its sources share one checkout.

use crate::diag::{validate_id, Diag};
use crate::json::{self, Json};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Area {
    pub id: String,
    pub mounts: Vec<Mount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceContribution {
    pub area: String,
    pub mount: String,
    pub enumerator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    pub id: String,
    pub contributions: Vec<SurfaceContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationObligation {
    pub spec: String,
    pub claim: String,
    pub areas: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub path: String,
    pub areas: Vec<Area>,
    pub surfaces: Vec<Surface>,
    pub realization_obligations: Vec<RealizationObligation>,
}

impl Workspace {
    pub fn surface(&self, id: &str) -> Option<&Surface> {
        self.surfaces.iter().find(|surface| surface.id == id)
    }

    pub fn area_for_file(&self, file: &str) -> Option<&Area> {
        let normalized = file.replace('\\', "/");
        self.areas
            .iter()
            .filter_map(|area| {
                area.mounts
                    .iter()
                    .filter(|mount| path_contains(&mount.path, &normalized))
                    .map(|mount| (area, mount.path.len()))
                    .max_by_key(|(_, length)| *length)
            })
            .max_by_key(|(_, length)| *length)
            .map(|(area, _)| area)
    }

    pub fn obligation(&self, spec: &str, claim: &str) -> Option<&RealizationObligation> {
        self.realization_obligations
            .iter()
            .find(|item| item.spec == spec && item.claim == claim)
    }
}

pub fn load(path: &Path) -> Result<Workspace, Vec<Diag>> {
    if !path.exists() {
        return Ok(Workspace::default());
    }
    let display = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diag::file(
            &display,
            format!("cannot read workspace declarations: {error}"),
        )]
    })?;
    let root = json::parse(&source).map_err(|error| {
        vec![Diag::file(
            &display,
            format!("malformed workspace declarations: {error}"),
        )]
    })?;
    parse(&display, &root)
}

pub fn parse(path: &str, root: &Json) -> Result<Workspace, Vec<Diag>> {
    let mut errors = Vec::new();
    reject_unknown_fields(
        path,
        "workspace",
        root,
        &[
            "format",
            "version",
            "areas",
            "surfaces",
            "realization_obligations",
        ],
        &mut errors,
    );
    expect_literal(path, root, "format", "azimuth-workspace", &mut errors);
    match root.get("version").and_then(Json::as_num) {
        Some(1.0) => {}
        Some(version) => errors.push(Diag::file(
            path,
            format!("unsupported-version: {version}; this tool accepts 1"),
        )),
        None => errors.push(Diag::file(path, "workspace has no numeric `version`")),
    }

    let areas = objects(path, root, "areas", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("areas[{index}]");
            reject_unknown_fields(path, &where_, item, &["id", "mounts"], &mut errors);
            let id = string(path, item, &where_, "id", &mut errors);
            validate_declared_id(path, &where_, &id, false, &mut errors);
            let mounts = objects(path, item, "mounts", &mut errors)
                .into_iter()
                .enumerate()
                .map(|(mount_index, mount)| {
                    let mount_where = format!("{where_}.mounts[{mount_index}]");
                    reject_unknown_fields(path, &mount_where, mount, &["id", "path"], &mut errors);
                    let mount_id = string(path, mount, &mount_where, "id", &mut errors);
                    validate_declared_id(path, &mount_where, &mount_id, false, &mut errors);
                    let mount_path = string(path, mount, &mount_where, "path", &mut errors);
                    if !normalized_relative_path(&mount_path) {
                        errors.push(Diag::file(
                            path,
                            format!(
                                "{mount_where}.path must be a normalized workspace-relative path"
                            ),
                        ));
                    }
                    Mount {
                        id: mount_id,
                        path: mount_path,
                    }
                })
                .collect::<Vec<_>>();
            unique(
                path,
                &format!("mount in area `{id}`"),
                mounts.iter().map(|m| &m.id),
                &mut errors,
            );
            Area { id, mounts }
        })
        .collect::<Vec<_>>();
    unique(path, "area", areas.iter().map(|area| &area.id), &mut errors);
    unique(
        path,
        "area mount path",
        areas
            .iter()
            .flat_map(|area| area.mounts.iter().map(|mount| &mount.path)),
        &mut errors,
    );

    let surfaces = objects(path, root, "surfaces", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("surfaces[{index}]");
            reject_unknown_fields(path, &where_, item, &["id", "contributions"], &mut errors);
            let id = string(path, item, &where_, "id", &mut errors);
            validate_declared_id(path, &where_, &id, true, &mut errors);
            let contributions = objects(path, item, "contributions", &mut errors)
                .into_iter()
                .enumerate()
                .map(|(contribution_index, contribution)| {
                    let contribution_where =
                        format!("{where_}.contributions[{contribution_index}]");
                    reject_unknown_fields(
                        path,
                        &contribution_where,
                        contribution,
                        &["area", "mount", "enumerator"],
                        &mut errors,
                    );
                    SurfaceContribution {
                        area: string(path, contribution, &contribution_where, "area", &mut errors),
                        mount: string(
                            path,
                            contribution,
                            &contribution_where,
                            "mount",
                            &mut errors,
                        ),
                        enumerator: string(
                            path,
                            contribution,
                            &contribution_where,
                            "enumerator",
                            &mut errors,
                        ),
                    }
                })
                .collect::<Vec<_>>();
            if contributions.is_empty() {
                errors.push(Diag::file(
                    path,
                    format!("surface `{id}` has no enumerator contributions"),
                ));
            }
            Surface { id, contributions }
        })
        .collect::<Vec<_>>();
    unique(
        path,
        "surface",
        surfaces.iter().map(|surface| &surface.id),
        &mut errors,
    );

    let realization_obligations =
        optional_objects(path, root, "realization_obligations", &mut errors)
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let where_ = format!("realization_obligations[{index}]");
                reject_unknown_fields(
                    path,
                    &where_,
                    item,
                    &["spec", "claim", "areas"],
                    &mut errors,
                );
                let spec = string(path, item, &where_, "spec", &mut errors);
                let claim = string(path, item, &where_, "claim", &mut errors);
                validate_declared_id(path, &where_, &spec, true, &mut errors);
                validate_declared_id(path, &where_, &claim, false, &mut errors);
                let areas = strings(path, item, &where_, "areas", &mut errors);
                if areas.is_empty() {
                    errors.push(Diag::file(
                        path,
                        format!("{where_}.areas must name at least one area"),
                    ));
                }
                unique(
                    path,
                    &format!("area in {spec}#{claim} obligation"),
                    areas.iter(),
                    &mut errors,
                );
                RealizationObligation { spec, claim, areas }
            })
            .collect::<Vec<_>>();
    unique(
        path,
        "realization obligation",
        realization_obligations
            .iter()
            .map(|item| format!("{}#{}", item.spec, item.claim))
            .collect::<Vec<_>>()
            .iter(),
        &mut errors,
    );

    for surface in &surfaces {
        for contribution in &surface.contributions {
            match areas.iter().find(|area| area.id == contribution.area) {
                None => errors.push(Diag::file(
                    path,
                    format!(
                        "unknown-area: surface `{}` names undeclared area `{}`",
                        surface.id, contribution.area
                    ),
                )),
                Some(area)
                    if !area
                        .mounts
                        .iter()
                        .any(|mount| mount.id == contribution.mount) =>
                {
                    errors.push(Diag::file(
                        path,
                        format!(
                            "unknown-mount: surface `{}` names `{}` in area `{}`, but that mount is not declared",
                            surface.id, contribution.mount, contribution.area
                        ),
                    ));
                }
                Some(_) => {}
            }
        }
    }
    for obligation in &realization_obligations {
        for area in &obligation.areas {
            if !areas.iter().any(|declared| declared.id == *area) {
                errors.push(Diag::file(
                    path,
                    format!(
                        "unknown-area: realization obligation `{}#{}` names undeclared area `{area}`",
                        obligation.spec, obligation.claim
                    ),
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(Workspace {
            path: path.to_string(),
            areas,
            surfaces,
            realization_obligations,
        })
    } else {
        Err(errors)
    }
}

fn path_contains(root: &str, file: &str) -> bool {
    file == root
        || file
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn normalized_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && !value.contains('\\')
}

fn validate_declared_id(
    path: &str,
    where_: &str,
    value: &str,
    allow_slash: bool,
    errors: &mut Vec<Diag>,
) {
    if let Err(reason) = validate_id(value, allow_slash) {
        errors.push(Diag::file(
            path,
            format!("{where_} has invalid id `{value}`: {reason}"),
        ));
    }
}

fn expect_literal(path: &str, root: &Json, field: &str, expected: &str, errors: &mut Vec<Diag>) {
    match root.get(field).and_then(Json::as_str) {
        Some(value) if value == expected => {}
        Some(value) => errors.push(Diag::file(
            path,
            format!("`{field}` is `{value}`, expected `{expected}`"),
        )),
        None => errors.push(Diag::file(
            path,
            format!("workspace has no string `{field}`"),
        )),
    }
}

fn string(path: &str, root: &Json, where_: &str, field: &str, errors: &mut Vec<Diag>) -> String {
    match root.get(field).and_then(Json::as_str) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => {
            errors.push(Diag::file(
                path,
                format!("{where_}.{field} must be a non-empty string"),
            ));
            String::new()
        }
    }
}

fn objects<'a>(path: &str, root: &'a Json, field: &str, errors: &mut Vec<Diag>) -> Vec<&'a Json> {
    match root.get(field).and_then(Json::as_array) {
        Some(items) if items.iter().all(|item| matches!(item, Json::Obj(_))) => {
            items.iter().collect()
        }
        _ => {
            errors.push(Diag::file(
                path,
                format!("`{field}` must be an array of objects"),
            ));
            Vec::new()
        }
    }
}

fn optional_objects<'a>(
    path: &str,
    root: &'a Json,
    field: &str,
    errors: &mut Vec<Diag>,
) -> Vec<&'a Json> {
    match root.get(field) {
        None => Vec::new(),
        Some(_) => objects(path, root, field, errors),
    }
}

fn strings(
    path: &str,
    root: &Json,
    where_: &str,
    field: &str,
    errors: &mut Vec<Diag>,
) -> Vec<String> {
    match root.get(field).and_then(Json::as_array) {
        Some(items) => items
            .iter()
            .filter_map(|item| match item.as_str() {
                Some(value) if !value.is_empty() => Some(value.to_string()),
                _ => {
                    errors.push(Diag::file(
                        path,
                        format!("{where_}.{field} contains a non-string or empty value"),
                    ));
                    None
                }
            })
            .collect(),
        None => {
            errors.push(Diag::file(
                path,
                format!("{where_}.{field} must be an array"),
            ));
            Vec::new()
        }
    }
}

fn unique<'a, I, T>(path: &str, kind: &str, values: I, errors: &mut Vec<Diag>)
where
    I: IntoIterator<Item = &'a T>,
    T: AsRef<str> + 'a + ?Sized,
{
    let mut seen = BTreeSet::new();
    for value in values {
        let value = value.as_ref();
        if !seen.insert(value.to_string()) {
            errors.push(Diag::file(path, format!("duplicate {kind} `{value}`")));
        }
    }
}

fn reject_unknown_fields(
    path: &str,
    where_: &str,
    root: &Json,
    allowed: &[&str],
    errors: &mut Vec<Diag>,
) {
    let Json::Obj(fields) = root else {
        return;
    };
    for (field, _) in fields {
        if !allowed.contains(&field.as_str()) {
            errors.push(Diag::file(
                path,
                format!("{where_} has unknown field `{field}`"),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_text(source: &str) -> Result<Workspace, Vec<Diag>> {
        parse("workspace.json", &json::parse(source).unwrap())
    }

    #[test]
    fn parses_areas_surfaces_and_obligations() {
        let workspace = parse_text(
            r#"{
              "format":"azimuth-workspace","version":1,
              "areas":[
                {"id":"trips","mounts":[{"id":"code","path":"app/services/Trips"}]},
                {"id":"rider-experience","mounts":[{"id":"code","path":"app/web/rider"}]}
              ],
              "surfaces":[{"id":"trips/rider-view","contributions":[
                {"area":"rider-experience","mount":"code","enumerator":"next-routes"}
              ]}],
              "realization_obligations":[{"spec":"referrals/rewards",
                "claim":"referral-summary-explains-state","areas":["trips","rider-experience"]}]
            }"#,
        )
        .unwrap();

        assert_eq!(workspace.surfaces[0].id, "trips/rider-view");
        assert_eq!(
            workspace
                .area_for_file("app/web/rider/src/app/referrals/page.tsx")
                .map(|area| area.id.as_str()),
            Some("rider-experience")
        );
    }

    #[test]
    fn rejects_unknown_surface_areas() {
        let errors = parse_text(
            r#"{"format":"azimuth-workspace","version":1,"areas":[],
              "surfaces":[{"id":"trips/rider-view","contributions":[
                {"area":"missing","mount":"code","enumerator":"next-routes"}
              ]}]}"#,
        )
        .unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.message.contains("unknown-area")));
    }

    #[test]
    fn rejects_fields_that_would_silently_drop_an_obligation() {
        let errors = parse_text(
            r#"{"format":"azimuth-workspace","version":1,"areas":[],"surfaces":[],
              "realization-obligation":[]}"#,
        )
        .unwrap_err();
        assert!(errors.iter().any(|error| error
            .message
            .contains("unknown field `realization-obligation`")));
    }

    #[test]
    fn rejects_an_ambiguous_area_mount_path() {
        let errors = parse_text(
            r#"{"format":"azimuth-workspace","version":1,"areas":[
              {"id":"first","mounts":[{"id":"code","path":"app/shared"}]},
              {"id":"second","mounts":[{"id":"code","path":"app/shared"}]}
            ],"surfaces":[]}"#,
        )
        .unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.message.contains("duplicate area mount path")));
    }
}
