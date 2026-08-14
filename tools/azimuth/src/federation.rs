//! Multi-repository project assembly.
//!
//! A repository manifest is a revision-bound observation produced by one repository. A project
//! catalog declares which repositories, model sources and areas make a complete account; a
//! workset supplies concrete checkouts and observations. Merely passing whichever manifests happen
//! to be present would turn absence into a false green result, so completeness is part of the
//! derived state rather than a caller convention.

use crate::diag::Diag;
use crate::fingerprint::sha256;
use crate::json::{self, Json};
use crate::manifest::{self, Manifest};
use crate::model::SourceIdentity;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const FORMAT_VERSION: u64 = 1;

#[derive(Debug, Clone)]
pub struct ProjectReference {
    pub project: String,
    pub repository: String,
    pub catalog: PathBuf,
    pub workset: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RepositoryDecl {
    pub id: String,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct MountDecl {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct AreaDecl {
    pub id: String,
    pub repository: String,
    pub mounts: Vec<MountDecl>,
}

#[derive(Debug, Clone)]
pub struct ModelSourceDecl {
    pub id: String,
    pub repository: String,
    pub path: String,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct StandardsDecl {
    pub repository: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct ReceiptRequirement {
    pub id: String,
    pub subjects: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub repositories: Vec<RepositoryDecl>,
    pub areas: Vec<AreaDecl>,
    pub model_sources: Vec<ModelSourceDecl>,
    pub standards: StandardsDecl,
    pub required_receipts: Vec<ReceiptRequirement>,
}

/// Resolves the project context that travels with a repository checkout. The reference is small
/// enough to duplicate because it is a locator, not another source of project authority.
pub fn load_project_reference(path: &Path) -> Result<ProjectReference, Vec<Diag>> {
    let root = read_json(path, "project reference")?;
    let display = path.display().to_string();
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut errors = Vec::new();
    require_format(&display, &root, "azimuth-project-reference", &mut errors);
    let project = required_string(&display, &root, "project", &mut errors).unwrap_or_default();
    let repository =
        required_string(&display, &root, "repository", &mut errors).unwrap_or_default();
    let catalog = required_string(&display, &root, "catalog", &mut errors)
        .map(|value| resolve(base, &value))
        .unwrap_or_default();
    let workset = root
        .get("workset")
        .and_then(Json::as_str)
        .map(|value| resolve(base, value));
    if root.get("workset").is_some() && workset.is_none() {
        errors.push(Diag::file(
            &display,
            "`workset` must be a string when present",
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    match load_project(&catalog) {
        Ok(declared) => {
            if declared.id != project {
                errors.push(Diag::file(
                    &display,
                    format!(
                        "project reference names `{project}`, but catalog declares `{}`",
                        declared.id
                    ),
                ));
            }
            if !declared
                .repositories
                .iter()
                .any(|candidate| candidate.id == repository)
            {
                errors.push(Diag::file(
                    &display,
                    format!("catalog does not declare repository `{repository}`"),
                ));
            }
        }
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    if errors.is_empty() {
        let catalog = fs::canonicalize(&catalog).unwrap_or(catalog);
        Ok(ProjectReference {
            project,
            repository,
            catalog,
            workset,
        })
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone)]
pub struct WorkRepository {
    pub id: String,
    pub root: PathBuf,
    pub revision: String,
    pub manifest: PathBuf,
    pub manifest_digest: String,
}

#[derive(Debug, Clone)]
pub struct Workset {
    pub project: String,
    pub repositories: Vec<WorkRepository>,
    pub receipts: Vec<WorkReceipt>,
}

#[derive(Debug, Clone)]
pub struct WorkReceipt {
    pub path: PathBuf,
    pub digest: String,
}

#[derive(Debug, Clone)]
pub struct ModelSourceObservation {
    pub id: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeState {
    Active,
    Archived,
}

impl ChangeState {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChangeObservation {
    pub id: String,
    pub state: ChangeState,
    pub path: String,
    pub digest: String,
    pub repository: String,
}

#[derive(Debug)]
pub struct RepositoryManifest {
    pub path: PathBuf,
    pub digest: String,
    pub project: String,
    pub repository: String,
    pub revision: String,
    pub producer: String,
    pub areas: Vec<String>,
    pub model_sources: Vec<ModelSourceObservation>,
    pub standards_digest: Option<String>,
    pub changes: Vec<ChangeObservation>,
    pub linkage: Manifest,
}

#[derive(Debug, Clone)]
pub struct ReceiptSubject {
    pub repository: String,
    pub revision: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionReceipt {
    pub id: String,
    pub project: String,
    pub outcome: String,
    pub subjects: Vec<ReceiptSubject>,
    pub digest: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RepositorySnapshot {
    pub id: String,
    pub revision: String,
    pub manifest_digest: String,
    pub dirty: bool,
    pub root: PathBuf,
}

#[derive(Debug)]
pub struct Assembly {
    pub project: Project,
    pub catalog_digest: String,
    pub complete: bool,
    pub local_repository: Option<String>,
    pub missing_inputs: Vec<String>,
    pub repositories: Vec<RepositorySnapshot>,
    pub model_roots: Vec<PathBuf>,
    pub standards_path: Option<PathBuf>,
    pub manifests: Vec<Manifest>,
    pub receipts: Vec<ExecutionReceipt>,
    pub changes: Vec<ChangeObservation>,
}

#[derive(Debug, Clone)]
pub struct AcceptedChange {
    pub id: String,
    pub repository: String,
    pub archive_date: String,
    pub archive_digest: String,
    pub pre_archive_revisions: Vec<(String, String)>,
}

impl Assembly {
    pub fn snapshot_json(&self) -> Result<String, String> {
        self.snapshot_json_with_change(None)
    }

    fn snapshot_json_with_change(
        &self,
        accepted_change: Option<&AcceptedChange>,
    ) -> Result<String, String> {
        if !self.complete {
            return Err("a partial project assembly cannot be finalized".into());
        }
        if self.repositories.iter().any(|repository| repository.dirty) {
            return Err("a project with dirty repository inputs cannot be finalized".into());
        }
        let loaded = crate::load_assembly(self, &[]).map_err(|diagnostics| {
            diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        if !loaded.warnings.is_empty() {
            return Err(format!(
                "project model has {} warning(s)",
                loaded.warnings.len()
            ));
        }
        let holes = crate::check::rtm(&loaded.model);
        let summary = crate::check::summarize(&loaded.model, &holes);
        if summary.errors > 0 || summary.warnings > 0 {
            return Err(format!(
                "project model has {} error(s), {} warning(s)",
                summary.errors, summary.warnings
            ));
        }
        let (model_fingerprint, _) = crate::change::finalization(&loaded.model, &holes);
        let repositories = self
            .repositories
            .iter()
            .map(|repository| {
                Json::obj(vec![
                    ("id", Json::str(&repository.id)),
                    ("revision", Json::str(&repository.revision)),
                    ("manifest_digest", Json::str(&repository.manifest_digest)),
                ])
            })
            .collect();
        let receipts = self
            .receipts
            .iter()
            .map(|receipt| {
                Json::obj(vec![
                    ("id", Json::str(&receipt.id)),
                    ("digest", Json::str(&receipt.digest)),
                ])
            })
            .collect();
        let areas = self
            .project
            .areas
            .iter()
            .map(|area| {
                Json::obj(vec![
                    ("id", Json::str(&area.id)),
                    ("repository", Json::str(&area.repository)),
                    (
                        "mounts",
                        Json::Arr(
                            area.mounts
                                .iter()
                                .map(|mount| {
                                    Json::obj(vec![
                                        ("id", Json::str(&mount.id)),
                                        ("path", Json::str(&mount.path)),
                                    ])
                                })
                                .collect(),
                        ),
                    ),
                ])
            })
            .collect();
        let changes = self
            .changes
            .iter()
            .map(|change| {
                Json::obj(vec![
                    ("id", Json::str(&change.id)),
                    ("state", Json::str(change.state.name())),
                    ("repository", Json::str(&change.repository)),
                    ("path", Json::str(&change.path)),
                    ("digest", Json::str(&change.digest)),
                ])
            })
            .collect();
        let mut fields = vec![
            ("format".to_string(), Json::str("azimuth-project-snapshot")),
            ("version".to_string(), Json::Num(FORMAT_VERSION as f64)),
            ("project".to_string(), Json::str(&self.project.id)),
            (
                "catalog_digest".to_string(),
                Json::str(&self.catalog_digest),
            ),
            (
                "model_fingerprint".to_string(),
                Json::str(&model_fingerprint),
            ),
            ("areas".to_string(), Json::Arr(areas)),
            ("repositories".to_string(), Json::Arr(repositories)),
            ("receipts".to_string(), Json::Arr(receipts)),
            ("changes".to_string(), Json::Arr(changes)),
        ];
        if let Some(change) = accepted_change {
            fields.push((
                "accepted_change".to_string(),
                Json::obj(vec![
                    ("id", Json::str(&change.id)),
                    ("repository", Json::str(&change.repository)),
                    ("archive_date", Json::str(&change.archive_date)),
                    ("archive_digest", Json::str(&change.archive_digest)),
                    (
                        "pre_archive_revisions",
                        Json::Arr(
                            change
                                .pre_archive_revisions
                                .iter()
                                .map(|(repository, revision)| {
                                    Json::obj(vec![
                                        ("repository", Json::str(repository)),
                                        ("revision", Json::str(revision)),
                                    ])
                                })
                                .collect(),
                        ),
                    ),
                ]),
            ));
        }
        Ok(Json::Obj(fields).to_string_pretty())
    }
}

pub fn load_project(path: &Path) -> Result<Project, Vec<Diag>> {
    let root = read_json(path, "project catalog")?;
    let display = path.display().to_string();
    let mut errors = Vec::new();
    require_format(&display, &root, "azimuth-project", &mut errors);
    let id = required_string(&display, &root, "project", &mut errors).unwrap_or_default();
    let repositories = object_array(&display, &root, "repositories", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("repositories[{index}]");
            RepositoryDecl {
                id: nested_string(&display, &where_, item, "id", &mut errors),
                required: item.get("required").and_then(Json::as_bool).unwrap_or(true),
            }
        })
        .collect::<Vec<_>>();
    let areas = object_array(&display, &root, "areas", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("areas[{index}]");
            let mounts = object_array(&display, item, "mounts", &mut errors)
                .into_iter()
                .enumerate()
                .map(|(mount_index, mount)| MountDecl {
                    id: nested_string(
                        &display,
                        &format!("{where_}.mounts[{mount_index}]"),
                        mount,
                        "id",
                        &mut errors,
                    ),
                    path: nested_string(
                        &display,
                        &format!("{where_}.mounts[{mount_index}]"),
                        mount,
                        "path",
                        &mut errors,
                    ),
                })
                .collect();
            AreaDecl {
                id: nested_string(&display, &where_, item, "id", &mut errors),
                repository: nested_string(&display, &where_, item, "repository", &mut errors),
                mounts,
            }
        })
        .collect::<Vec<_>>();
    let model_sources = object_array(&display, &root, "model_sources", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("model_sources[{index}]");
            ModelSourceDecl {
                id: nested_string(&display, &where_, item, "id", &mut errors),
                repository: nested_string(&display, &where_, item, "repository", &mut errors),
                path: nested_string(&display, &where_, item, "path", &mut errors),
                required: item.get("required").and_then(Json::as_bool).unwrap_or(true),
            }
        })
        .collect::<Vec<_>>();
    let standards = match root.get("standards") {
        Some(value) => StandardsDecl {
            repository: nested_string(&display, "standards", value, "repository", &mut errors),
            path: nested_string(&display, "standards", value, "path", &mut errors),
        },
        None => {
            errors.push(Diag::file(
                &display,
                "project catalog has no `standards` object",
            ));
            StandardsDecl {
                repository: String::new(),
                path: String::new(),
            }
        }
    };
    let required_receipts =
        optional_object_array(&display, &root, "required_receipts", &mut errors)
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let where_ = format!("required_receipts[{index}]");
                ReceiptRequirement {
                    id: nested_string(&display, &where_, item, "id", &mut errors),
                    subjects: string_array(&display, &where_, item, "subjects", &mut errors),
                }
            })
            .collect::<Vec<_>>();

    validate_catalog(
        &display,
        &repositories,
        &areas,
        &model_sources,
        &standards,
        &required_receipts,
        &mut errors,
    );
    if errors.is_empty() {
        Ok(Project {
            id,
            repositories,
            areas,
            model_sources,
            standards,
            required_receipts,
        })
    } else {
        Err(errors)
    }
}

pub fn load_workset(path: &Path) -> Result<Workset, Vec<Diag>> {
    let root = read_json(path, "workset")?;
    let display = path.display().to_string();
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut errors = Vec::new();
    require_format(&display, &root, "azimuth-workset", &mut errors);
    let project = required_string(&display, &root, "project", &mut errors).unwrap_or_default();
    let repositories = object_array(&display, &root, "repositories", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("repositories[{index}]");
            WorkRepository {
                id: nested_string(&display, &where_, item, "id", &mut errors),
                root: resolve(
                    base,
                    &nested_string(&display, &where_, item, "root", &mut errors),
                ),
                revision: nested_string(&display, &where_, item, "revision", &mut errors),
                manifest: resolve(
                    base,
                    &nested_string(&display, &where_, item, "manifest", &mut errors),
                ),
                manifest_digest: nested_string(
                    &display,
                    &where_,
                    item,
                    "manifest_digest",
                    &mut errors,
                ),
            }
        })
        .collect::<Vec<_>>();
    let receipts = optional_object_array(&display, &root, "receipts", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("receipts[{index}]");
            WorkReceipt {
                path: resolve(
                    base,
                    &nested_string(&display, &where_, item, "path", &mut errors),
                ),
                digest: nested_string(&display, &where_, item, "digest", &mut errors),
            }
        })
        .collect();
    duplicate_ids(
        &display,
        "workset repository",
        repositories.iter().map(|repository| repository.id.as_str()),
        &mut errors,
    );
    if errors.is_empty() {
        Ok(Workset {
            project,
            repositories,
            receipts,
        })
    } else {
        Err(errors)
    }
}

pub fn assemble(
    project_path: &Path,
    workset_path: &Path,
    local_repository: Option<&str>,
) -> Result<Assembly, Vec<Diag>> {
    let catalog_digest = fs::read(project_path)
        .map(|content| sha256(&content))
        .map_err(|error| {
            vec![Diag::file(
                &project_path.display().to_string(),
                format!("cannot fingerprint project catalog: {error}"),
            )]
        })?;
    let project = load_project(project_path)?;
    let workset = load_workset(workset_path)?;
    let display = workset_path.display().to_string();
    let mut errors = Vec::new();
    if workset.project != project.id {
        errors.push(Diag::file(
            &display,
            format!(
                "workset is for project `{}`, expected `{}`",
                workset.project, project.id
            ),
        ));
    }
    if let Some(local) = local_repository {
        if !project
            .repositories
            .iter()
            .any(|repository| repository.id == local)
        {
            errors.push(Diag::file(
                &display,
                format!("unknown local repository `{local}`"),
            ));
        }
        if !workset
            .repositories
            .iter()
            .any(|repository| repository.id == local)
        {
            errors.push(Diag::file(
                &display,
                format!("missing-input: requested local repository `{local}`"),
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let selected = workset
        .repositories
        .iter()
        .filter(|entry| local_repository.is_none_or(|local| entry.id == local))
        .collect::<Vec<_>>();
    let selected_ids = selected
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut missing_inputs = Vec::new();
    for repository in project
        .repositories
        .iter()
        .filter(|repository| repository.required)
    {
        if !selected_ids.contains(repository.id.as_str()) {
            missing_inputs.push(format!("repository:{}", repository.id));
            if local_repository.is_none() {
                errors.push(Diag::file(
                    &display,
                    format!("missing-input: required repository `{}`", repository.id),
                ));
            }
        }
    }
    if local_repository.is_none() {
        for source in project
            .model_sources
            .iter()
            .filter(|source| source.required)
        {
            if !selected_ids.contains(source.repository.as_str()) {
                missing_inputs.push(format!("model-source:{}", source.id));
                errors.push(Diag::file(
                    &display,
                    format!(
                        "missing-input: model source `{}` requires repository `{}`",
                        source.id, source.repository
                    ),
                ));
            }
        }
        if !selected_ids.contains(project.standards.repository.as_str()) {
            missing_inputs.push("standards".into());
            errors.push(Diag::file(
                &display,
                format!(
                    "missing-input: standards require repository `{}`",
                    project.standards.repository
                ),
            ));
        }
    }

    let known_repositories = project
        .repositories
        .iter()
        .map(|repository| repository.id.as_str())
        .collect::<BTreeSet<_>>();
    for &entry in &selected {
        if !known_repositories.contains(entry.id.as_str()) {
            errors.push(Diag::file(
                &display,
                format!("workset contains unknown repository `{}`", entry.id),
            ));
        }
    }

    let mut parsed = Vec::new();
    let mut snapshots = Vec::new();
    for &entry in &selected {
        let actual_revision = git_output(&entry.root, &["rev-parse", "HEAD"]);
        match actual_revision {
            Ok(actual) if actual == entry.revision => {}
            Ok(actual) => errors.push(Diag::file(
                &display,
                format!(
                    "revision-mismatch: repository `{}` checkout is `{actual}`, workset names `{}`",
                    entry.id, entry.revision
                ),
            )),
            Err(error) => errors.push(Diag::file(
                &display,
                format!("cannot inspect Git repository `{}`: {error}", entry.id),
            )),
        }
        let dirty = git_output(
            &entry.root,
            &["status", "--porcelain", "--untracked-files=all"],
        )
        .map(|output| !output.is_empty())
        .unwrap_or(true);
        match load_repository_manifest(&entry.manifest) {
            Ok(repository_manifest) => {
                if repository_manifest.digest != entry.manifest_digest {
                    errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!(
                            "manifest-digest-mismatch: workset pins {}, observed {}",
                            entry.manifest_digest, repository_manifest.digest
                        ),
                    ));
                }
                if repository_manifest.project != project.id {
                    errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!(
                            "repository manifest is for project `{}`, expected `{}`",
                            repository_manifest.project, project.id
                        ),
                    ));
                }
                if repository_manifest.repository != entry.id {
                    errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!(
                            "repository manifest identifies `{}`, workset assigns it to `{}`",
                            repository_manifest.repository, entry.id
                        ),
                    ));
                }
                if repository_manifest.revision != entry.revision {
                    errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!(
                            "revision-mismatch: manifest observes `{}`, workset names `{}`",
                            repository_manifest.revision, entry.revision
                        ),
                    ));
                }
                snapshots.push(RepositorySnapshot {
                    id: entry.id.clone(),
                    revision: entry.revision.clone(),
                    manifest_digest: repository_manifest.digest.clone(),
                    dirty,
                    root: entry.root.clone(),
                });
                parsed.push((entry, repository_manifest));
            }
            Err(mut manifest_errors) => errors.append(&mut manifest_errors),
        }
    }

    validate_observations(
        &project,
        &parsed,
        local_repository,
        &mut missing_inputs,
        &mut errors,
    );
    validate_change_authority(&parsed, &mut errors);
    let mut model_roots = Vec::new();
    let mut standards_path = None;
    for (entry, repository_manifest) in &parsed {
        for source in project
            .model_sources
            .iter()
            .filter(|source| source.repository == entry.id)
        {
            let root = match contained_path(&entry.root, &source.path) {
                Ok(path) => path,
                Err(error) => {
                    errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!(
                            "model source `{}` escapes its repository: {error}",
                            source.id
                        ),
                    ));
                    continue;
                }
            };
            match non_versioned_paths(&entry.root, &source.path) {
                Ok(paths) if !paths.is_empty() => errors.push(Diag::file(
                    &repository_manifest.path.display().to_string(),
                    format!(
                        "model source `{}` contains non-versioned input(s): {}",
                        source.id,
                        paths.join(", ")
                    ),
                )),
                Ok(_) => {}
                Err(error) => errors.push(Diag::file(
                    &repository_manifest.path.display().to_string(),
                    format!("cannot inspect model source `{}`: {error}", source.id),
                )),
            }
            let observed = repository_manifest
                .model_sources
                .iter()
                .find(|observation| observation.id == source.id);
            match observed {
                None if source.required => {
                    missing_inputs.push(format!("model-source:{}", source.id));
                    errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!("missing-input: model source `{}`", source.id),
                    ));
                }
                None => {}
                Some(observation) => match tree_digest(&root) {
                    Ok(actual) if actual == observation.digest => model_roots.push(root),
                    Ok(actual) => errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!(
                            "model-source-mismatch: `{}` declares {}, checkout contains {actual}",
                            source.id, observation.digest
                        ),
                    )),
                    Err(error) => errors.push(Diag::file(
                        &root.display().to_string(),
                        format!("cannot fingerprint model source `{}`: {error}", source.id),
                    )),
                },
            }
        }
        if project.standards.repository == entry.id {
            let path = match contained_path(&entry.root, &project.standards.path) {
                Ok(path) => path,
                Err(error) => {
                    errors.push(Diag::file(
                        &repository_manifest.path.display().to_string(),
                        format!("standards escape their repository: {error}"),
                    ));
                    continue;
                }
            };
            if !git_file_tracked(&entry.root, &project.standards.path) {
                errors.push(Diag::file(
                    &repository_manifest.path.display().to_string(),
                    "standards are not a tracked file at the selected revision",
                ));
            }
            match (
                repository_manifest.standards_digest.as_ref(),
                fs::read(&path),
            ) {
                (Some(expected), Ok(content)) if *expected == sha256(&content) => {
                    standards_path = Some(path)
                }
                (Some(expected), Ok(content)) => errors.push(Diag::file(
                    &repository_manifest.path.display().to_string(),
                    format!(
                        "standards-mismatch: manifest declares {expected}, checkout contains {}",
                        sha256(&content)
                    ),
                )),
                (None, _) => errors.push(Diag::file(
                    &repository_manifest.path.display().to_string(),
                    "missing-input: standards digest",
                )),
                (_, Err(error)) => errors.push(Diag::file(
                    &path.display().to_string(),
                    format!("cannot read standards: {error}"),
                )),
            }
        }
    }

    let mut receipts = Vec::new();
    for selected_receipt in &workset.receipts {
        match load_receipt(&selected_receipt.path) {
            Ok(receipt) => {
                if receipt.digest != selected_receipt.digest {
                    errors.push(Diag::file(
                        &receipt.path.display().to_string(),
                        format!(
                            "receipt-digest-mismatch: workset pins {}, observed {}",
                            selected_receipt.digest, receipt.digest
                        ),
                    ));
                }
                receipts.push(receipt)
            }
            Err(mut receipt_errors) => errors.append(&mut receipt_errors),
        }
    }
    validate_receipts(
        &project,
        &snapshots,
        &receipts,
        local_repository,
        &mut missing_inputs,
        &mut errors,
    );
    validate_source_identities(&project, &parsed, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }
    let manifests = parsed
        .iter()
        .map(|(_, repository_manifest)| repository_manifest.linkage.clone())
        .collect();
    let mut changes = parsed
        .iter()
        .flat_map(|(_, repository_manifest)| repository_manifest.changes.clone())
        .collect::<Vec<_>>();
    changes.sort();
    let complete = local_repository.is_none() && missing_inputs.is_empty();
    snapshots.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(Assembly {
        project,
        catalog_digest,
        complete,
        local_repository: local_repository.map(str::to_string),
        missing_inputs,
        repositories: snapshots,
        model_roots,
        standards_path,
        manifests,
        receipts,
        changes,
    })
}

pub fn accept_change(
    project_path: &Path,
    before_workset: &Path,
    after_workset: &Path,
    change_id: &str,
    archive_date: &str,
) -> Result<String, Vec<Diag>> {
    if !valid_archive_date(archive_date) {
        return Err(vec![Diag::file(
            &project_path.display().to_string(),
            format!("invalid archive date `{archive_date}`; expected YYYY-MM-DD"),
        )]);
    }
    let before = assemble(project_path, before_workset, None)?;
    let after = assemble(project_path, after_workset, None)?;
    let display = format!("project change `{change_id}`");
    let mut errors = Vec::new();

    for (name, assembly) in [("pre-archive", &before), ("post-archive", &after)] {
        if let Err(error) = assembly.snapshot_json() {
            errors.push(Diag::file(&display, format!("{name} account: {error}")));
        }
    }

    let active = before
        .changes
        .iter()
        .find(|change| change.id == change_id && change.state == ChangeState::Active);
    let archived = after
        .changes
        .iter()
        .find(|change| change.id == change_id && change.state == ChangeState::Archived);
    let expected_archive_path = format!("azimuth/changes/archive/{archive_date}-{change_id}");
    if active.is_none() {
        errors.push(Diag::file(
            &display,
            "pre-archive account has no singular active change authority",
        ));
    }
    if after
        .changes
        .iter()
        .any(|change| change.id == change_id && change.state == ChangeState::Active)
    {
        errors.push(Diag::file(
            &display,
            "post-archive account still declares the change active",
        ));
    }
    if archived.is_none() {
        errors.push(Diag::file(
            &display,
            format!("post-archive account has no archive at `{expected_archive_path}`"),
        ));
    }

    if let (Some(active), Some(archived)) = (active, archived) {
        if archived.repository != active.repository {
            errors.push(Diag::file(
                &display,
                format!(
                    "change authority moved from `{}` to `{}` while archiving",
                    active.repository, archived.repository
                ),
            ));
        }
        if archived.path != expected_archive_path {
            errors.push(Diag::file(
                &display,
                format!(
                    "archive path is `{}`, expected `{expected_archive_path}`",
                    archived.path
                ),
            ));
        }
        if archived.digest != active.digest {
            errors.push(Diag::file(
                &display,
                "archive content differs from the accepted active change",
            ));
        }
        validate_change_completion(&before, active, &display, &mut errors);
        validate_archive_transition(&before, &after, active, &display, &mut errors);
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    let active = active.unwrap();
    let archived = archived.unwrap();
    let accepted = AcceptedChange {
        id: change_id.to_string(),
        repository: active.repository.clone(),
        archive_date: archive_date.to_string(),
        archive_digest: archived.digest.clone(),
        pre_archive_revisions: before
            .repositories
            .iter()
            .map(|repository| (repository.id.clone(), repository.revision.clone()))
            .collect(),
    };
    after
        .snapshot_json_with_change(Some(&accepted))
        .map_err(|error| vec![Diag::file(&display, error)])
}

fn validate_change_completion(
    assembly: &Assembly,
    change: &ChangeObservation,
    display: &str,
    errors: &mut Vec<Diag>,
) {
    let Some(repository) = assembly
        .repositories
        .iter()
        .find(|repository| repository.id == change.repository)
    else {
        errors.push(Diag::file(display, "change authority repository is absent"));
        return;
    };
    let loaded = match crate::load_assembly(assembly, &[]) {
        Ok(loaded) => loaded,
        Err(diagnostics) => {
            errors.extend(diagnostics);
            return;
        }
    };
    let root = repository.root.join(&change.path);
    match crate::change::inspect(&root, &loaded.model) {
        Ok(report) => {
            for issue in crate::change::completion_issues(&root, &report) {
                errors.push(Diag::file(display, issue));
            }
        }
        Err(issues) => {
            errors.extend(issues.into_iter().map(|issue| Diag::file(display, issue)));
        }
    }
}

fn validate_archive_transition(
    before: &Assembly,
    after: &Assembly,
    accepted: &ChangeObservation,
    display: &str,
    errors: &mut Vec<Diag>,
) {
    let before_repositories = before
        .repositories
        .iter()
        .map(|repository| repository.id.as_str())
        .collect::<BTreeSet<_>>();
    let after_repositories = after
        .repositories
        .iter()
        .map(|repository| repository.id.as_str())
        .collect::<BTreeSet<_>>();
    if before_repositories != after_repositories {
        errors.push(Diag::file(
            display,
            "pre-archive and post-archive repository sets differ",
        ));
    }
    for pre_repository in &before.repositories {
        let Some(post_repository) = after
            .repositories
            .iter()
            .find(|repository| repository.id == pre_repository.id)
        else {
            errors.push(Diag::file(
                display,
                format!(
                    "post-archive account omits repository `{}`",
                    pre_repository.id
                ),
            ));
            continue;
        };
        if pre_repository.id != accepted.repository {
            if pre_repository.revision != post_repository.revision {
                errors.push(Diag::file(
                    display,
                    format!(
                        "unrelated repository `{}` changed during archive",
                        pre_repository.id
                    ),
                ));
            }
            continue;
        }
        if pre_repository.revision == post_repository.revision {
            errors.push(Diag::file(
                display,
                "authority repository revision did not advance for the archive",
            ));
        }
        let post_change = after
            .changes
            .iter()
            .find(|change| change.id == accepted.id)
            .expect("validated archive exists");
        let pre_content = tracked_tree_digest_without(&pre_repository.root, &accepted.path);
        let post_content = tracked_tree_digest_without(&post_repository.root, &post_change.path);
        match (pre_content, post_content) {
            (Ok(pre_content), Ok(post_content)) if pre_content == post_content => {}
            (Ok(_), Ok(_)) => errors.push(Diag::file(
                display,
                "authority repository changed outside the accepted change directory during archive",
            )),
            (Err(error), _) | (_, Err(error)) => errors.push(Diag::file(display, error)),
        }
    }

    let before_other = before
        .changes
        .iter()
        .filter(|change| change.id != accepted.id)
        .collect::<BTreeSet<_>>();
    let after_other = after
        .changes
        .iter()
        .filter(|change| change.id != accepted.id)
        .collect::<BTreeSet<_>>();
    if before_other != after_other {
        errors.push(Diag::file(
            display,
            "another change authority moved or changed during archive",
        ));
    }
}

fn tracked_tree_digest_without(root: &Path, excluded: &str) -> Result<String, String> {
    let mut input = Vec::new();
    for relative in tracked_paths(root)? {
        if is_within(&relative, excluded) {
            continue;
        }
        let path = root.join(&relative);
        input.extend_from_slice(relative.as_bytes());
        input.push(0);
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            input.extend_from_slice(
                fs::read_link(&path)
                    .map_err(|error| error.to_string())?
                    .to_string_lossy()
                    .as_bytes(),
            );
        } else {
            input.extend_from_slice(&fs::read(&path).map_err(|error| error.to_string())?);
        }
        input.push(0xff);
    }
    Ok(sha256(&input))
}

pub fn tree_digest(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files).map_err(|error| error.to_string())?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut input = Vec::new();
    for (relative, path) in files {
        input.extend_from_slice(relative.as_bytes());
        input.push(0);
        input.extend_from_slice(
            &fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?,
        );
        input.push(0xff);
    }
    Ok(sha256(&input))
}

fn observe_changes(root: &Path, repository: &str) -> Result<Vec<ChangeObservation>, String> {
    let relative_root = "azimuth/changes";
    let changes_root = root.join(relative_root);
    if !changes_root.exists() {
        return Ok(Vec::new());
    }
    let non_versioned = non_versioned_paths(root, relative_root)?;
    if !non_versioned.is_empty() {
        return Err(format!(
            "change source contains non-versioned input(s): {}",
            non_versioned.join(", ")
        ));
    }

    let mut changes = Vec::new();
    for entry in fs::read_dir(&changes_root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            return Err(format!(
                "symbolic link cannot declare change authority: {}",
                entry.path().display()
            ));
        }
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "archive" {
            for archived in fs::read_dir(entry.path()).map_err(|error| error.to_string())? {
                let archived = archived.map_err(|error| error.to_string())?;
                let file_type = archived.file_type().map_err(|error| error.to_string())?;
                if file_type.is_symlink() {
                    return Err(format!(
                        "symbolic link cannot declare archived change authority: {}",
                        archived.path().display()
                    ));
                }
                if !file_type.is_dir() {
                    continue;
                }
                let archived_name = archived.file_name().to_string_lossy().to_string();
                let Some(id) = archived_change_id(&archived_name) else {
                    return Err(format!(
                        "archived change directory `{archived_name}` must be `YYYY-MM-DD-<change-id>`"
                    ));
                };
                changes.push(change_observation(
                    root,
                    repository,
                    id,
                    ChangeState::Archived,
                    &format!("{relative_root}/archive/{archived_name}"),
                )?);
            }
        } else {
            changes.push(change_observation(
                root,
                repository,
                &name,
                ChangeState::Active,
                &format!("{relative_root}/{name}"),
            )?);
        }
    }
    changes.sort();
    Ok(changes)
}

fn change_observation(
    root: &Path,
    repository: &str,
    id: &str,
    state: ChangeState,
    path: &str,
) -> Result<ChangeObservation, String> {
    if id.is_empty() || !root.join(path).join("proposal.md").is_file() {
        return Err(format!("{} change `{id}` has no proposal.md", state.name()));
    }
    Ok(ChangeObservation {
        id: id.to_string(),
        state,
        path: path.to_string(),
        digest: tree_digest(&root.join(path))?,
        repository: repository.to_string(),
    })
}

fn archived_change_id(name: &str) -> Option<&str> {
    let bytes = name.as_bytes();
    let date = name.get(..10)?;
    if bytes.len() <= 11 || bytes.get(10) != Some(&b'-') || !valid_archive_date(date) {
        return None;
    }
    Some(&name[11..])
}

fn valid_archive_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

/// Envelops ordinary extractor output as one repository observation. Area ownership is derived
/// from catalog mounts rather than restated at every tag. Extractors retain responsibility for the
/// compiler-resolved site name; this layer gives it a project-wide typed namespace.
pub fn observe_repository(
    project_path: &Path,
    repository: &str,
    root: &Path,
    producer: &str,
    linkage_paths: &[PathBuf],
) -> Result<String, Vec<Diag>> {
    let project = load_project(project_path)?;
    let display = project_path.display().to_string();
    let mut errors = Vec::new();
    if !project
        .repositories
        .iter()
        .any(|declared| declared.id == repository)
    {
        errors.push(Diag::file(
            &display,
            format!("unknown repository `{repository}`"),
        ));
    }
    if producer.trim().is_empty() {
        errors.push(Diag::file(&display, "producer must be non-empty"));
    }
    let revision = match git_output(root, &["rev-parse", "HEAD"]) {
        Ok(revision) => revision,
        Err(error) => {
            errors.push(Diag::file(
                &root.display().to_string(),
                format!("cannot inspect Git revision: {error}"),
            ));
            String::new()
        }
    };
    let areas = project
        .areas
        .iter()
        .filter(|area| area.repository == repository)
        .collect::<Vec<_>>();
    let mut linkage = Manifest::default();
    for path in linkage_paths {
        match manifest::load(path) {
            Ok(observed) => merge_manifest(&mut linkage, observed),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    assign_sources(&mut linkage, &areas, &mut errors, producer);

    let mut model_sources = Vec::new();
    for source in project
        .model_sources
        .iter()
        .filter(|source| source.repository == repository)
    {
        match non_versioned_paths(root, &source.path) {
            Ok(paths) if !paths.is_empty() => errors.push(Diag::file(
                &root.join(&source.path).display().to_string(),
                format!(
                    "model source `{}` contains non-versioned input(s): {}",
                    source.id,
                    paths.join(", ")
                ),
            )),
            Ok(_) => {}
            Err(error) => errors.push(Diag::file(
                &root.display().to_string(),
                format!("cannot inspect model source `{}`: {error}", source.id),
            )),
        }
        match tree_digest(&root.join(&source.path)) {
            Ok(digest) => model_sources.push(ModelSourceObservation {
                id: source.id.clone(),
                digest,
            }),
            Err(error) => errors.push(Diag::file(
                &root.join(&source.path).display().to_string(),
                format!("cannot fingerprint model source `{}`: {error}", source.id),
            )),
        }
    }
    let standards_digest = if project.standards.repository == repository {
        let path = root.join(&project.standards.path);
        match fs::read(&path) {
            Ok(content) => Some(sha256(&content)),
            Err(error) => {
                errors.push(Diag::file(
                    &path.display().to_string(),
                    format!("cannot read standards: {error}"),
                ));
                None
            }
        }
    } else {
        None
    };
    let changes = match observe_changes(root, repository) {
        Ok(changes) => changes,
        Err(error) => {
            errors.push(Diag::file(
                &root.join("azimuth/changes").display().to_string(),
                error,
            ));
            Vec::new()
        }
    };
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut fields = vec![
        (
            "format".to_string(),
            Json::str("azimuth-repository-manifest"),
        ),
        ("version".to_string(), Json::Num(FORMAT_VERSION as f64)),
        ("project".to_string(), Json::str(&project.id)),
        ("repository".to_string(), Json::str(repository)),
        ("revision".to_string(), Json::str(revision)),
        ("producer".to_string(), Json::str(producer)),
        (
            "areas".to_string(),
            Json::Arr(areas.iter().map(|area| Json::str(&area.id)).collect()),
        ),
        (
            "model_sources".to_string(),
            Json::Arr(
                model_sources
                    .iter()
                    .map(|source| {
                        Json::obj(vec![
                            ("id", Json::str(&source.id)),
                            ("digest", Json::str(&source.digest)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "changes".to_string(),
            Json::Arr(
                changes
                    .iter()
                    .map(|change| {
                        Json::obj(vec![
                            ("id", Json::str(&change.id)),
                            ("state", Json::str(change.state.name())),
                            ("path", Json::str(&change.path)),
                            ("digest", Json::str(&change.digest)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ];
    if let Some(digest) = standards_digest {
        fields.push(("standards_digest".to_string(), Json::str(digest)));
    }
    fields.push(("linkage".to_string(), linkage_json(&linkage)));
    Ok(Json::Obj(fields).to_string_pretty())
}

fn merge_manifest(target: &mut Manifest, source: Manifest) {
    target.realizes.extend(source.realizes);
    target.covers.extend(source.covers);
    target
        .mechanism_implementations
        .extend(source.mechanism_implementations);
    target.mechanism_covers.extend(source.mechanism_covers);
    target.class_members.extend(source.class_members);
    target.enumerations.extend(source.enumerations);
    target.artifacts.extend(source.artifacts);
    target.observations.extend(source.observations);
}

fn assign_sources(
    manifest: &mut Manifest,
    areas: &[&AreaDecl],
    errors: &mut Vec<Diag>,
    producer: &str,
) {
    for item in manifest.realizes.iter_mut().chain(&mut manifest.covers) {
        item.source = locate_source(
            areas,
            &item.file,
            address_kind(&item.lang, &item.file, &item.site),
            address_value(&item.lang, &item.file, &item.site),
            errors,
            producer,
        );
    }
    for item in &mut manifest.mechanism_implementations {
        let (kind, address) = split_typed_binding(&item.binding, &item.lang);
        item.source = locate_source(areas, &item.file, kind, address, errors, producer);
    }
    for item in &mut manifest.mechanism_covers {
        item.source = locate_source(
            areas,
            &item.file,
            address_kind(&item.lang, &item.file, &item.site),
            address_value(&item.lang, &item.file, &item.site),
            errors,
            producer,
        );
    }
    for item in &mut manifest.class_members {
        item.source = locate_source(
            areas,
            &item.file,
            "class-member".into(),
            format!("{}#{}", item.class, item.site),
            errors,
            producer,
        );
    }
    for item in &mut manifest.enumerations {
        item.identity = locate_source(
            areas,
            &item.source,
            "enumerator".into(),
            format!("{}#{}", item.class, item.kind),
            errors,
            producer,
        );
    }
    for item in &mut manifest.artifacts {
        item.source = locate_source(
            areas,
            &item.file,
            item.kind.clone(),
            item.id.clone(),
            errors,
            producer,
        );
    }
    for item in &mut manifest.observations {
        let locator = item.inputs.first().unwrap_or(&item.report);
        item.source = locate_source(
            areas,
            locator,
            "assurance-observation".into(),
            item.id.clone(),
            errors,
            producer,
        );
    }
}

fn locate_source(
    areas: &[&AreaDecl],
    file: &str,
    kind: String,
    address: String,
    errors: &mut Vec<Diag>,
    producer: &str,
) -> Option<SourceIdentity> {
    let mut matches = areas
        .iter()
        .flat_map(|area| area.mounts.iter().map(move |mount| (*area, mount)))
        .filter(|(_, mount)| is_within(file, &mount.path))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.1.path.len().cmp(&left.1.path.len()));
    let Some((area, mount)) = matches.first() else {
        errors.push(Diag::file(
            producer,
            format!("source locator `{file}` belongs to no declared area mount"),
        ));
        return None;
    };
    if matches
        .get(1)
        .is_some_and(|(_, other)| other.path.len() == mount.path.len())
    {
        errors.push(Diag::file(
            producer,
            format!("source locator `{file}` matches ambiguous area mounts"),
        ));
        return None;
    }
    Some(SourceIdentity {
        area: area.id.clone(),
        kind,
        address,
        mount: mount.id.clone(),
    })
}

fn address_kind(language: &str, file: &str, site: &str) -> String {
    if language == "csharp" {
        "dotnet-symbol".into()
    } else if language == "prometheus" && file.ends_with(".rules.test.yml") {
        "prometheus-rule-test".into()
    } else if language == "prometheus" {
        "prometheus-alert".into()
    } else if language == "typescript"
        && file.replace('\\', "/").contains("/app/")
        && file.ends_with("/route.ts")
        && matches!(site, "GET" | "POST" | "PUT" | "PATCH" | "DELETE")
    {
        "next-route".into()
    } else if language == "typescript" {
        "typescript-symbol".into()
    } else {
        format!("{language}-symbol")
    }
}

fn address_value(language: &str, file: &str, site: &str) -> String {
    if address_kind(language, file, site) != "next-route" {
        return site.to_string();
    }
    let normalized = file.replace('\\', "/");
    let route = normalized
        .split("/app/")
        .nth(1)
        .unwrap_or(&normalized)
        .trim_end_matches("/route.ts");
    format!("{site} /{route}")
}

fn split_typed_binding(binding: &str, language: &str) -> (String, String) {
    binding
        .split_once(':')
        .map(|(kind, address)| (kind.to_string(), address.to_string()))
        .unwrap_or_else(|| (format!("{language}-symbol"), binding.to_string()))
}

fn linkage_json(manifest: &Manifest) -> Json {
    let model = crate::model::Model {
        realizes: manifest.realizes.clone(),
        covers: manifest.covers.clone(),
        mechanism_implementations: manifest.mechanism_implementations.clone(),
        mechanism_covers: manifest.mechanism_covers.clone(),
        class_members: manifest.class_members.clone(),
        enumerations: manifest.enumerations.clone(),
        artifacts: manifest.artifacts.clone(),
        observations: manifest.observations.clone(),
        ..Default::default()
    };
    let export = model.to_json(&[]);
    Json::Obj(
        [
            "realizes",
            "covers",
            "mechanism_implementations",
            "mechanism_covers",
            "class_members",
            "enumerations",
            "artifacts",
            "observations",
        ]
        .into_iter()
        .map(|key| {
            (
                key.to_string(),
                export.get(key).cloned().unwrap_or(Json::Arr(Vec::new())),
            )
        })
        .collect(),
    )
}

fn collect_files(
    root: &Path,
    current: &Path,
    out: &mut Vec<(String, PathBuf)>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "symbolic link is not a repository-owned tree input: {}",
                    path.display()
                ),
            ));
        }
        if file_type.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) != Some(".git") {
                collect_files(root, &path, out)?;
            }
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, path));
        }
    }
    Ok(())
}

fn load_repository_manifest(path: &Path) -> Result<RepositoryManifest, Vec<Diag>> {
    let bytes = fs::read(path).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("cannot read repository manifest: {error}"),
        )]
    })?;
    let source = String::from_utf8(bytes.clone()).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("repository manifest is not UTF-8: {error}"),
        )]
    })?;
    let root = json::parse(&source).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("malformed repository manifest: {error}"),
        )]
    })?;
    let display = path.display().to_string();
    let mut errors = Vec::new();
    require_format(&display, &root, "azimuth-repository-manifest", &mut errors);
    let project = required_string(&display, &root, "project", &mut errors).unwrap_or_default();
    let repository =
        required_string(&display, &root, "repository", &mut errors).unwrap_or_default();
    let revision = required_string(&display, &root, "revision", &mut errors).unwrap_or_default();
    let producer = required_string(&display, &root, "producer", &mut errors).unwrap_or_default();
    let areas = optional_string_array(&display, &root, "areas", &mut errors);
    let model_sources = optional_object_array(&display, &root, "model_sources", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| ModelSourceObservation {
            id: nested_string(
                &display,
                &format!("model_sources[{index}]"),
                item,
                "id",
                &mut errors,
            ),
            digest: nested_string(
                &display,
                &format!("model_sources[{index}]"),
                item,
                "digest",
                &mut errors,
            ),
        })
        .collect();
    let standards_digest = root
        .get("standards_digest")
        .and_then(Json::as_str)
        .map(str::to_string);
    let changes = object_array(&display, &root, "changes", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let where_ = format!("changes[{index}]");
            let state_name = nested_string(&display, &where_, item, "state", &mut errors);
            let state = match ChangeState::parse(&state_name) {
                Some(state) => state,
                None => {
                    errors.push(Diag::file(
                        &display,
                        format!("{where_}.state must be `active` or `archived`"),
                    ));
                    ChangeState::Active
                }
            };
            ChangeObservation {
                id: nested_string(&display, &where_, item, "id", &mut errors),
                state,
                path: nested_string(&display, &where_, item, "path", &mut errors),
                digest: nested_string(&display, &where_, item, "digest", &mut errors),
                repository: repository.clone(),
            }
        })
        .collect();
    let linkage = match root.get("linkage") {
        Some(linkage) => manifest::parse(&display, linkage),
        None => Err(vec![Diag::file(
            &display,
            "repository manifest has no `linkage` object",
        )]),
    };
    let linkage = match linkage {
        Ok(linkage) => Some(linkage),
        Err(mut linkage_errors) => {
            errors.append(&mut linkage_errors);
            None
        }
    };
    if errors.is_empty() {
        Ok(RepositoryManifest {
            path: path.to_path_buf(),
            digest: sha256(&bytes),
            project,
            repository,
            revision,
            producer,
            areas,
            model_sources,
            standards_digest,
            changes,
            linkage: linkage.unwrap(),
        })
    } else {
        Err(errors)
    }
}

fn load_receipt(path: &Path) -> Result<ExecutionReceipt, Vec<Diag>> {
    let bytes = fs::read(path).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("cannot read execution receipt: {error}"),
        )]
    })?;
    let source = String::from_utf8(bytes.clone()).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("execution receipt is not UTF-8: {error}"),
        )]
    })?;
    let root = json::parse(&source).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("malformed execution receipt: {error}"),
        )]
    })?;
    let display = path.display().to_string();
    let mut errors = Vec::new();
    require_format(&display, &root, "azimuth-execution-receipt", &mut errors);
    let id = required_string(&display, &root, "id", &mut errors).unwrap_or_default();
    let project = required_string(&display, &root, "project", &mut errors).unwrap_or_default();
    let outcome = required_string(&display, &root, "outcome", &mut errors).unwrap_or_default();
    let subjects = object_array(&display, &root, "subjects", &mut errors)
        .into_iter()
        .enumerate()
        .map(|(index, item)| ReceiptSubject {
            repository: nested_string(
                &display,
                &format!("subjects[{index}]"),
                item,
                "repository",
                &mut errors,
            ),
            revision: nested_string(
                &display,
                &format!("subjects[{index}]"),
                item,
                "revision",
                &mut errors,
            ),
        })
        .collect();
    if errors.is_empty() {
        Ok(ExecutionReceipt {
            id,
            project,
            outcome,
            subjects,
            digest: sha256(&bytes),
            path: path.to_path_buf(),
        })
    } else {
        Err(errors)
    }
}

fn validate_catalog(
    path: &str,
    repositories: &[RepositoryDecl],
    areas: &[AreaDecl],
    model_sources: &[ModelSourceDecl],
    standards: &StandardsDecl,
    receipts: &[ReceiptRequirement],
    errors: &mut Vec<Diag>,
) {
    duplicate_ids(
        path,
        "repository",
        repositories.iter().map(|item| item.id.as_str()),
        errors,
    );
    duplicate_ids(
        path,
        "area",
        areas.iter().map(|item| item.id.as_str()),
        errors,
    );
    duplicate_ids(
        path,
        "model source",
        model_sources.iter().map(|item| item.id.as_str()),
        errors,
    );
    duplicate_ids(
        path,
        "required receipt",
        receipts.iter().map(|item| item.id.as_str()),
        errors,
    );
    let repository_ids = repositories
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    for area in areas {
        if !repository_ids.contains(area.repository.as_str()) {
            errors.push(Diag::file(
                path,
                format!(
                    "area `{}` belongs to unknown repository `{}`",
                    area.id, area.repository
                ),
            ));
        }
        duplicate_ids(
            path,
            &format!("mount in area `{}`", area.id),
            area.mounts.iter().map(|mount| mount.id.as_str()),
            errors,
        );
        for mount in &area.mounts {
            if !repository_relative(&mount.path) {
                errors.push(Diag::file(
                    path,
                    format!(
                        "mount `{}` in area `{}` must be a normalized repository-relative path",
                        mount.id, area.id
                    ),
                ));
            }
        }
    }
    for source in model_sources {
        if !repository_ids.contains(source.repository.as_str()) {
            errors.push(Diag::file(
                path,
                format!(
                    "model source `{}` belongs to unknown repository `{}`",
                    source.id, source.repository
                ),
            ));
        }
        if !repository_relative(&source.path) {
            errors.push(Diag::file(
                path,
                format!(
                    "model source `{}` must use a normalized repository-relative path",
                    source.id
                ),
            ));
        }
    }
    if !repository_ids.contains(standards.repository.as_str()) {
        errors.push(Diag::file(
            path,
            format!(
                "standards belong to unknown repository `{}`",
                standards.repository
            ),
        ));
    }
    if !repository_relative(&standards.path) {
        errors.push(Diag::file(
            path,
            "standards must use a normalized repository-relative path",
        ));
    }
    for receipt in receipts {
        duplicate_ids(
            path,
            &format!("subject in receipt `{}`", receipt.id),
            receipt.subjects.iter().map(String::as_str),
            errors,
        );
        for subject in &receipt.subjects {
            if !repository_ids.contains(subject.as_str()) {
                errors.push(Diag::file(
                    path,
                    format!(
                        "receipt `{}` requires unknown repository `{subject}`",
                        receipt.id
                    ),
                ));
            }
        }
    }
}

fn validate_observations(
    project: &Project,
    parsed: &[(&WorkRepository, RepositoryManifest)],
    local: Option<&str>,
    missing: &mut Vec<String>,
    errors: &mut Vec<Diag>,
) {
    let mut observed_areas = BTreeMap::<String, String>::new();
    for (entry, manifest) in parsed {
        validate_repository_changes(entry, manifest, errors);
        for area in &manifest.areas {
            match project.areas.iter().find(|declared| declared.id == *area) {
                None => errors.push(Diag::file(
                    &manifest.path.display().to_string(),
                    format!("manifest claims unknown area `{area}`"),
                )),
                Some(declared) if declared.repository != entry.id => errors.push(Diag::file(
                    &manifest.path.display().to_string(),
                    format!(
                        "area-ownership-conflict: `{area}` belongs to `{}`, claimed by `{}`",
                        declared.repository, entry.id
                    ),
                )),
                Some(_) => {
                    if let Some(previous) = observed_areas.insert(area.clone(), entry.id.clone()) {
                        errors.push(Diag::file(
                            &manifest.path.display().to_string(),
                            format!(
                                "area-ownership-conflict: `{area}` is claimed by `{previous}` and `{}`",
                                entry.id
                            ),
                        ));
                    }
                }
            }
        }
        let mut observed_sources = BTreeSet::new();
        for source in &manifest.model_sources {
            if !observed_sources.insert(source.id.as_str()) {
                errors.push(Diag::file(
                    &manifest.path.display().to_string(),
                    format!("duplicate model source observation `{}`", source.id),
                ));
            }
            match project
                .model_sources
                .iter()
                .find(|declared| declared.id == source.id)
            {
                None => errors.push(Diag::file(
                    &manifest.path.display().to_string(),
                    format!("manifest observes unknown model source `{}`", source.id),
                )),
                Some(declared) if declared.repository != entry.id => errors.push(Diag::file(
                    &manifest.path.display().to_string(),
                    format!(
                        "model-source-ownership-conflict: `{}` belongs to `{}`, observed by `{}`",
                        source.id, declared.repository, entry.id
                    ),
                )),
                Some(_) => {}
            }
        }
    }
    for area in project
        .areas
        .iter()
        .filter(|area| local.is_none_or(|repository| area.repository == repository))
    {
        if !observed_areas.contains_key(&area.id) {
            missing.push(format!("area:{}", area.id));
            errors.push(Diag::file(
                "project",
                format!("missing-input: area `{}`", area.id),
            ));
        }
    }
}

fn validate_repository_changes(
    entry: &WorkRepository,
    manifest: &RepositoryManifest,
    errors: &mut Vec<Diag>,
) {
    let display = manifest.path.display().to_string();
    let mut declared = manifest.changes.clone();
    declared.sort();
    if declared.windows(2).any(|pair| pair[0] == pair[1]) {
        errors.push(Diag::file(
            &display,
            "repository manifest contains a duplicate change observation",
        ));
    }
    for change in &declared {
        if !repository_relative(&change.path) {
            errors.push(Diag::file(
                &display,
                format!(
                    "change `{}` must use a normalized repository-relative path",
                    change.id
                ),
            ));
        }
    }
    match observe_changes(&entry.root, &entry.id) {
        Ok(actual) if actual == declared => {}
        Ok(actual) => {
            let declared_set = declared.iter().collect::<BTreeSet<_>>();
            let actual_set = actual.iter().collect::<BTreeSet<_>>();
            for missing in actual_set.difference(&declared_set) {
                errors.push(Diag::file(
                    &display,
                    format!(
                        "change-observation-mismatch: `{}` {} at `{}` is absent or stale",
                        missing.id,
                        missing.state.name(),
                        missing.path
                    ),
                ));
            }
            for extra in declared_set.difference(&actual_set) {
                errors.push(Diag::file(
                    &display,
                    format!(
                        "change-observation-mismatch: manifest claims `{}` {} at `{}`",
                        extra.id,
                        extra.state.name(),
                        extra.path
                    ),
                ));
            }
        }
        Err(error) => errors.push(Diag::file(&display, error)),
    }
}

fn validate_change_authority(
    parsed: &[(&WorkRepository, RepositoryManifest)],
    errors: &mut Vec<Diag>,
) {
    let mut authorities = BTreeMap::<&str, &ChangeObservation>::new();
    for (_, manifest) in parsed {
        for change in &manifest.changes {
            if let Some(previous) = authorities.insert(&change.id, change) {
                errors.push(Diag::file(
                    &manifest.path.display().to_string(),
                    format!(
                        "change-authority-conflict: `{}` is declared by `{}` ({}) and `{}` ({})",
                        change.id,
                        previous.repository,
                        previous.state.name(),
                        change.repository,
                        change.state.name()
                    ),
                ));
            }
        }
    }
}

fn validate_source_identities(
    project: &Project,
    parsed: &[(&WorkRepository, RepositoryManifest)],
    errors: &mut Vec<Diag>,
) {
    let mut observed = BTreeMap::<String, (String, String)>::new();
    for (entry, manifest) in parsed {
        let path = manifest.path.display().to_string();
        let tracked = match tracked_paths(&entry.root) {
            Ok(paths) => paths,
            Err(error) => {
                errors.push(Diag::file(
                    &path,
                    format!("cannot enumerate tracked source locators: {error}"),
                ));
                BTreeSet::new()
            }
        };
        for (source, fingerprint, file) in source_records(&manifest.linkage) {
            let Some(source) = source else {
                errors.push(Diag::file(
                    &path,
                    format!("source-without-area: `{file}` has no typed source identity"),
                ));
                continue;
            };
            if !manifest.areas.iter().any(|area| area == &source.area) {
                errors.push(Diag::file(
                    &path,
                    format!(
                        "source `{}` uses area `{}` not claimed by repository `{}`",
                        source.address, source.area, manifest.repository
                    ),
                ));
            }
            match best_mount(project, &manifest.repository, file) {
                Ok((area, mount)) if area.id == source.area && mount.id == source.mount => {}
                Ok((area, mount)) => errors.push(Diag::file(
                    &path,
                    format!(
                        "source locator `{file}` resolves to area `{}` mount `{}`, not claimed area `{}` mount `{}`",
                        area.id, mount.id, source.area, source.mount
                    ),
                )),
                Err(message) => errors.push(Diag::file(&path, message)),
            }
            if let Err(error) = contained_path(&entry.root, file) {
                errors.push(Diag::file(
                    &path,
                    format!("source locator `{file}` escapes its repository: {error}"),
                ));
            }
            if !tracked.contains(&file.replace('\\', "/")) {
                errors.push(Diag::file(
                    &path,
                    format!(
                        "source locator `{file}` is not a tracked file at revision `{}`",
                        manifest.revision
                    ),
                ));
            }
            let key = source.key();
            if let Some((previous_fingerprint, previous_file)) = observed.get(&key) {
                if previous_fingerprint != fingerprint || previous_file != file {
                    errors.push(Diag::file(
                        &path,
                        format!(
                            "source-identity-conflict: `{key}` resolves inconsistently to `{previous_file}` and `{file}`"
                        ),
                    ));
                }
            } else {
                observed.insert(key, (fingerprint.to_string(), file.to_string()));
            }
        }
    }
}

fn is_within(path: &str, root: &str) -> bool {
    if !repository_relative(path) || !repository_relative(root) {
        return false;
    }
    let normalized_path = path.trim_matches('/');
    let normalized_root = root.trim_matches('/');
    normalized_path == normalized_root
        || normalized_path
            .strip_prefix(normalized_root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn repository_relative(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    !normalized.is_empty()
        && Path::new(&normalized)
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn contained_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if !repository_relative(relative) {
        return Err("locator is not a normalized repository-relative path".into());
    }
    let repository = fs::canonicalize(root)
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;
    let candidate = fs::canonicalize(root.join(relative))
        .map_err(|error| format!("cannot resolve locator: {error}"))?;
    if !candidate.starts_with(&repository) {
        return Err("resolved locator is outside the repository root".into());
    }
    Ok(candidate)
}

fn best_mount<'a>(
    project: &'a Project,
    repository: &str,
    file: &str,
) -> Result<(&'a AreaDecl, &'a MountDecl), String> {
    let mut matches = project
        .areas
        .iter()
        .filter(|area| area.repository == repository)
        .flat_map(|area| area.mounts.iter().map(move |mount| (area, mount)))
        .filter(|(_, mount)| is_within(file, &mount.path))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.1.path.len().cmp(&left.1.path.len()));
    let Some(&(area, mount)) = matches.first() else {
        return Err(format!(
            "source locator `{file}` belongs to no declared area mount"
        ));
    };
    if matches
        .get(1)
        .is_some_and(|(_, other)| other.path.len() == mount.path.len())
    {
        return Err(format!(
            "source locator `{file}` matches ambiguous area mounts"
        ));
    }
    Ok((area, mount))
}

fn non_versioned_paths(root: &Path, relative: &str) -> Result<Vec<String>, String> {
    let mut paths = BTreeSet::new();
    for arguments in [
        &["ls-files", "--others", "--exclude-standard", "--"][..],
        &[
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--",
        ][..],
    ] {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(arguments)
            .arg(relative)
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        for path in String::from_utf8_lossy(&output.stdout).lines() {
            if !path.is_empty() {
                paths.insert(path.to_string());
            }
        }
    }
    Ok(paths.into_iter().collect())
}

fn git_file_tracked(root: &Path, relative: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(relative)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn tracked_paths(root: &Path) -> Result<BTreeSet<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files"])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|path| !path.is_empty())
        .map(str::to_string)
        .collect())
}

fn source_records(manifest: &Manifest) -> Vec<(Option<&SourceIdentity>, &str, &str)> {
    let mut records = Vec::new();
    records.extend(
        manifest
            .realizes
            .iter()
            .chain(&manifest.covers)
            .map(|item| {
                (
                    item.source.as_ref(),
                    item.source_fingerprint.as_str(),
                    item.file.as_str(),
                )
            }),
    );
    records.extend(manifest.mechanism_implementations.iter().map(|item| {
        (
            item.source.as_ref(),
            item.source_fingerprint.as_str(),
            item.file.as_str(),
        )
    }));
    records.extend(manifest.mechanism_covers.iter().map(|item| {
        (
            item.source.as_ref(),
            item.source_fingerprint.as_str(),
            item.file.as_str(),
        )
    }));
    records.extend(
        manifest
            .class_members
            .iter()
            .map(|item| (item.source.as_ref(), "", item.file.as_str())),
    );
    records.extend(manifest.enumerations.iter().map(|item| {
        (
            item.identity.as_ref(),
            item.source_fingerprint.as_str(),
            item.source.as_str(),
        )
    }));
    records.extend(
        manifest
            .artifacts
            .iter()
            .map(|item| (item.source.as_ref(), "", item.file.as_str())),
    );
    records.extend(manifest.observations.iter().map(|item| {
        let locator = item.inputs.first().unwrap_or(&item.report);
        (
            item.source.as_ref(),
            item.source_fingerprint.as_str(),
            locator.as_str(),
        )
    }));
    records
}

fn validate_receipts(
    project: &Project,
    repositories: &[RepositorySnapshot],
    receipts: &[ExecutionReceipt],
    local: Option<&str>,
    missing: &mut Vec<String>,
    errors: &mut Vec<Diag>,
) {
    let requirements = project
        .required_receipts
        .iter()
        .map(|requirement| (requirement.id.as_str(), requirement))
        .collect::<BTreeMap<_, _>>();
    let known_repositories = project
        .repositories
        .iter()
        .map(|repository| repository.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen_receipts = BTreeSet::new();
    for receipt in receipts {
        let display = receipt.path.display().to_string();
        if !seen_receipts.insert(receipt.id.as_str()) {
            errors.push(Diag::file(
                &display,
                format!("duplicate execution receipt `{}`", receipt.id),
            ));
        }
        let Some(requirement) = requirements.get(receipt.id.as_str()) else {
            errors.push(Diag::file(
                &display,
                format!("unexpected execution receipt `{}`", receipt.id),
            ));
            continue;
        };
        if receipt.project != project.id {
            errors.push(Diag::file(
                &display,
                format!(
                    "receipt is for project `{}`, expected `{}`",
                    receipt.project, project.id
                ),
            ));
        }
        if receipt.outcome != "passed" {
            errors.push(Diag::file(
                &display,
                format!(
                    "execution receipt `{}` outcome is `{}`",
                    receipt.id, receipt.outcome
                ),
            ));
        }
        let mut subjects = BTreeMap::new();
        for subject in &receipt.subjects {
            if subjects
                .insert(subject.repository.as_str(), subject)
                .is_some()
            {
                errors.push(Diag::file(
                    &display,
                    format!(
                        "execution receipt `{}` duplicates subject `{}`",
                        receipt.id, subject.repository
                    ),
                ));
            }
            if !known_repositories.contains(subject.repository.as_str()) {
                errors.push(Diag::file(
                    &display,
                    format!(
                        "execution receipt `{}` names unknown subject `{}`",
                        receipt.id, subject.repository
                    ),
                ));
            }
        }
        let expected = requirement
            .subjects
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let actual = subjects.keys().copied().collect::<BTreeSet<_>>();
        for omitted in expected.difference(&actual) {
            errors.push(Diag::file(
                &display,
                format!(
                    "execution receipt `{}` omits subject `{omitted}`",
                    receipt.id
                ),
            ));
        }
        for extra in actual.difference(&expected) {
            errors.push(Diag::file(
                &display,
                format!(
                    "execution receipt `{}` has extra subject `{extra}`",
                    receipt.id
                ),
            ));
        }
        for subject in expected.intersection(&actual) {
            let Some(repository) = repositories.iter().find(|item| item.id == **subject) else {
                if local.is_none() {
                    errors.push(Diag::file(
                        &display,
                        format!("receipt subject `{subject}` is absent from the workset"),
                    ));
                }
                continue;
            };
            let observed = subjects[subject];
            if observed.revision != repository.revision {
                errors.push(Diag::file(
                    &display,
                    format!(
                        "receipt-revision-mismatch: `{subject}` was tested at `{}`, selected `{}`",
                        observed.revision, repository.revision
                    ),
                ));
            }
        }
    }
    for requirement in &project.required_receipts {
        if !seen_receipts.contains(requirement.id.as_str()) {
            missing.push(format!("receipt:{}", requirement.id));
            if local.is_none() {
                errors.push(Diag::file(
                    "project",
                    format!("missing-input: execution receipt `{}`", requirement.id),
                ));
            }
        }
    }
}

fn read_json(path: &Path, kind: &str) -> Result<Json, Vec<Diag>> {
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("cannot read {kind}: {error}"),
        )]
    })?;
    json::parse(&source).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("malformed {kind}: {error}"),
        )]
    })
}

fn require_format(path: &str, root: &Json, expected: &str, errors: &mut Vec<Diag>) {
    match root.get("format").and_then(Json::as_str) {
        Some(actual) if actual == expected => {}
        Some(actual) => errors.push(Diag::file(
            path,
            format!("format is `{actual}`, expected `{expected}`"),
        )),
        None => errors.push(Diag::file(path, "document has no string `format`")),
    }
    match root.get("version").and_then(Json::as_num) {
        Some(actual) if actual == FORMAT_VERSION as f64 => {}
        Some(actual) => errors.push(Diag::file(
            path,
            format!("unsupported-version: {actual}; this tool accepts {FORMAT_VERSION}"),
        )),
        None => errors.push(Diag::file(path, "document has no numeric `version`")),
    }
}

fn required_string(path: &str, root: &Json, field: &str, errors: &mut Vec<Diag>) -> Option<String> {
    match root.get(field).and_then(Json::as_str) {
        Some(value) if !value.is_empty() => Some(value.to_string()),
        _ => {
            errors.push(Diag::file(
                path,
                format!("`{field}` must be a non-empty string"),
            ));
            None
        }
    }
}

fn nested_string(
    path: &str,
    where_: &str,
    root: &Json,
    field: &str,
    errors: &mut Vec<Diag>,
) -> String {
    match root.get(field).and_then(Json::as_str) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => {
            errors.push(Diag::file(
                path,
                format!("{where_}.`{field}` must be a non-empty string"),
            ));
            String::new()
        }
    }
}

fn object_array<'a>(
    path: &str,
    root: &'a Json,
    field: &str,
    errors: &mut Vec<Diag>,
) -> Vec<&'a Json> {
    match root.get(field).and_then(Json::as_array) {
        Some(items) => items.iter().collect(),
        None => {
            errors.push(Diag::file(path, format!("`{field}` must be an array")));
            Vec::new()
        }
    }
}

fn optional_object_array<'a>(
    path: &str,
    root: &'a Json,
    field: &str,
    errors: &mut Vec<Diag>,
) -> Vec<&'a Json> {
    match root.get(field) {
        Some(value) => match value.as_array() {
            Some(items) => items.iter().collect(),
            None => {
                errors.push(Diag::file(path, format!("`{field}` must be an array")));
                Vec::new()
            }
        },
        None => Vec::new(),
    }
}

fn string_array(
    path: &str,
    where_: &str,
    root: &Json,
    field: &str,
    errors: &mut Vec<Diag>,
) -> Vec<String> {
    match root.get(field).and_then(Json::as_array) {
        Some(items) => items
            .iter()
            .filter_map(|item| match item.as_str() {
                Some(value) => Some(value.to_string()),
                None => {
                    errors.push(Diag::file(
                        path,
                        format!("{where_}.`{field}` contains a non-string"),
                    ));
                    None
                }
            })
            .collect(),
        None => {
            errors.push(Diag::file(
                path,
                format!("{where_}.`{field}` must be an array"),
            ));
            Vec::new()
        }
    }
}

fn optional_string_array(
    path: &str,
    root: &Json,
    field: &str,
    errors: &mut Vec<Diag>,
) -> Vec<String> {
    match root.get(field) {
        Some(value) => match value.as_array() {
            Some(items) => items
                .iter()
                .filter_map(|item| match item.as_str() {
                    Some(value) => Some(value.to_string()),
                    None => {
                        errors.push(Diag::file(path, format!("`{field}` contains a non-string")));
                        None
                    }
                })
                .collect(),
            None => {
                errors.push(Diag::file(path, format!("`{field}` must be an array")));
                Vec::new()
            }
        },
        None => Vec::new(),
    }
}

fn duplicate_ids<'a>(
    path: &str,
    kind: &str,
    ids: impl Iterator<Item = &'a str>,
    errors: &mut Vec<Diag>,
) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            errors.push(Diag::file(path, format!("duplicate {kind} id `{id}`")));
        }
    }
}

fn resolve(base: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn git_output(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
