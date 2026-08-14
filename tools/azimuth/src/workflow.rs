//! Authoring and work-package operations that do not need the accepted assurance model.
//!
//! These operations deliberately stop at filesystem artifacts and deterministic instructions.
//! Product edits and agent dispatch remain agent-tier actions because the core cannot make either
//! portable across repositories or coding-agent runtimes.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_STANDARDS: &str = "# Verification standards\nDefault scope: unit\n\n## Level: critical\nStrength: demonstration\nQuantification: universal\nResidual: required\n\n## Level: standard\nStrength: demonstration\nQuantification: example\nResidual: optional\n\n## Level: routine\nStrength: none\nResidual: optional\n";
const DEFAULT_WORKSPACE: &str = "{\n  \"format\": \"azimuth-workspace\",\n  \"version\": 1,\n  \"areas\": [],\n  \"surfaces\": [],\n  \"realization_obligations\": []\n}\n";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChangeSummary {
    pub id: String,
    pub path: PathBuf,
    pub archived: bool,
    pub status: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WorkPackage {
    pub id: String,
    pub status: PackageStatus,
    pub depends_on: Vec<String>,
    pub owns: Vec<String>,
    pub objective: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PackageStatus {
    Pending,
    InProgress,
    Complete,
}

impl PackageStatus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in-progress",
            Self::Complete => "complete",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "pending" => Some(Self::Pending),
            "in-progress" => Some(Self::InProgress),
            "complete" => Some(Self::Complete),
            _ => None,
        }
    }
}

pub fn initialize(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut created = Vec::new();
    for relative in [
        "model",
        "changes/archive",
        "explorations/archive",
        "standards",
    ] {
        let path = root.join(relative);
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|error| format!("cannot create {}: {error}", path.display()))?;
            created.push(path);
        }
    }
    let standards = root.join("standards/verification.md");
    if !standards.exists() {
        fs::write(&standards, DEFAULT_STANDARDS)
            .map_err(|error| format!("cannot write {}: {error}", standards.display()))?;
        created.push(standards);
    }
    let workspace = root.join("workspace.json");
    if !workspace.exists() {
        fs::write(&workspace, DEFAULT_WORKSPACE)
            .map_err(|error| format!("cannot write {}: {error}", workspace.display()))?;
        created.push(workspace);
    }
    Ok(created)
}

pub fn create_change(changes: &Path, id: &str, title: &str) -> Result<PathBuf, String> {
    validate_id(id)?;
    let root = changes.join(id);
    if root.exists() {
        return Err(format!(
            "change `{id}` already exists at {}",
            root.display()
        ));
    }
    fs::create_dir_all(root.join("specs"))
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    fs::write(
        root.join("proposal.md"),
        format!(
            "# Change: {id}\n\nStatus: proposed\n\n## Problem\n\n## Outcome\n\n## Scope\n\n## Affected claims\n\n## Completion conditions\n\n"
        ),
    )
    .map_err(|error| format!("cannot write proposal.md: {error}"))?;
    fs::write(
        root.join("plan.md"),
        format!("# Plan: {title}\n\n- [ ] Add the accepted intent delta.\n- [ ] Implement and verify the change.\n- [ ] Record the outcome and complete the change.\n"),
    )
    .map_err(|error| format!("cannot write plan.md: {error}"))?;
    Ok(root)
}

pub fn create_exploration(explorations: &Path, id: &str, title: &str) -> Result<PathBuf, String> {
    validate_id(id)?;
    let root = explorations.join(id);
    if root.exists() {
        return Err(format!(
            "exploration `{id}` already exists at {}",
            root.display()
        ));
    }
    fs::create_dir_all(&root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    fs::write(
        root.join("exploration.md"),
        format!(
            "# Exploration: {title}\n\nId: {id}\nStatus: exploring\n\n## Objective\n\n## Boundaries\n\n## Existing context\n\n## Findings\n\n## Decisions\n\n## Open questions\n\n## Result\n\n"
        ),
    )
    .map_err(|error| format!("cannot write exploration.md: {error}"))?;
    Ok(root)
}

pub fn list_changes(changes: &Path) -> Result<Vec<ChangeSummary>, String> {
    let mut summaries = Vec::new();
    collect_changes(changes, false, &mut summaries)?;
    let archive = changes.join("archive");
    if archive.exists() {
        collect_changes(&archive, true, &mut summaries)?;
    }
    summaries.sort_by(|left, right| left.id.cmp(&right.id).then(left.path.cmp(&right.path)));
    Ok(summaries)
}

fn collect_changes(
    root: &Path,
    archived: bool,
    summaries: &mut Vec<ChangeSummary>,
) -> Result<(), String> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("cannot read {}: {error}", root.display())),
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", root.display()))?;
        let path = entry.path();
        if !path.is_dir() || (!archived && entry.file_name() == "archive") {
            continue;
        }
        let proposal = path.join("proposal.md");
        if !proposal.is_file() {
            continue;
        }
        let directory = entry.file_name().to_string_lossy().to_string();
        let id = if archived {
            directory.get(11..).unwrap_or(&directory).to_string()
        } else {
            directory
        };
        let source = fs::read_to_string(&proposal)
            .map_err(|error| format!("cannot read {}: {error}", proposal.display()))?;
        summaries.push(ChangeSummary {
            id,
            path,
            archived,
            status: field(&source, "Status").unwrap_or_else(|| "unknown".into()),
        });
    }
    Ok(())
}

pub fn resolve_change(changes: &Path, value: &str) -> Result<PathBuf, String> {
    let direct = PathBuf::from(value);
    if direct.join("proposal.md").is_file() {
        return Ok(direct);
    }
    let active = changes.join(value);
    if active.join("proposal.md").is_file() {
        return Ok(active);
    }
    let matches = list_changes(changes)?
        .into_iter()
        .filter(|summary| summary.id == value)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [only] => Ok(only.path.clone()),
        [] => Err(format!(
            "change `{value}` was not found under {}",
            changes.display()
        )),
        _ => Err(format!(
            "change `{value}` has more than one authority under {}",
            changes.display()
        )),
    }
}

pub fn render_change(root: &Path) -> Result<String, String> {
    let mut rendered = String::new();
    for name in [
        "proposal.md",
        "design.md",
        "verification.md",
        "plan.md",
        "work-packages.md",
        "outcome.md",
    ] {
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        rendered.push_str(&format!("--- {name} ---\n{source}"));
        if !source.ends_with('\n') {
            rendered.push('\n');
        }
    }
    if rendered.is_empty() {
        return Err(format!("{} contains no change artifacts", root.display()));
    }
    Ok(rendered)
}

pub fn load_work_packages(root: &Path) -> Result<Vec<WorkPackage>, Vec<String>> {
    let path = root.join("work-packages.md");
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => return Err(vec![format!("cannot read {}: {error}", path.display())]),
    };
    let mut errors = Vec::new();
    let mut packages = Vec::new();
    let mut current: Option<(String, BTreeMap<String, String>, usize)> = None;
    for (index, line) in source.lines().enumerate() {
        if let Some(id) = line.trim().strip_prefix("## Work package:") {
            if let Some(item) = current.take() {
                finish_package(&path, item, &mut packages, &mut errors);
            }
            current = Some((id.trim().to_string(), BTreeMap::new(), index + 1));
            continue;
        }
        let Some((_, fields, _)) = current.as_mut() else {
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            if !["Status", "Depends on", "Owns", "Objective", "Evidence"].contains(&key) {
                errors.push(format!(
                    "{}:{}: unknown work-package field `{key}`",
                    path.display(),
                    index + 1
                ));
            } else if fields
                .insert(key.to_string(), value.trim().to_string())
                .is_some()
            {
                errors.push(format!(
                    "{}:{}: duplicate work-package field `{key}`",
                    path.display(),
                    index + 1
                ));
            }
        } else {
            errors.push(format!(
                "{}:{}: expected a work-package field",
                path.display(),
                index + 1
            ));
        }
    }
    if let Some(item) = current {
        finish_package(&path, item, &mut packages, &mut errors);
    }
    validate_packages(&path, &packages, &mut errors);
    if errors.is_empty() {
        Ok(packages)
    } else {
        Err(errors)
    }
}

fn finish_package(
    path: &Path,
    item: (String, BTreeMap<String, String>, usize),
    packages: &mut Vec<WorkPackage>,
    errors: &mut Vec<String>,
) {
    let (id, fields, line) = item;
    if validate_id(&id).is_err() {
        errors.push(format!(
            "{}:{line}: invalid work-package id `{id}`",
            path.display()
        ));
        return;
    }
    let status_value = fields.get("Status").map(String::as_str).unwrap_or("");
    let Some(status) = PackageStatus::parse(status_value) else {
        errors.push(format!(
            "{}:{line}: `{id}` has unknown Status `{status_value}`",
            path.display()
        ));
        return;
    };
    let list = |key: &str| {
        fields
            .get(key)
            .map(|value| comma_list(value))
            .unwrap_or_default()
    };
    let objective = fields.get("Objective").cloned().unwrap_or_default();
    if objective.is_empty() {
        errors.push(format!(
            "{}:{line}: `{id}` is missing Objective",
            path.display()
        ));
    }
    let owns = list("Owns");
    if owns.is_empty() {
        errors.push(format!(
            "{}:{line}: `{id}` is missing owned paths",
            path.display()
        ));
    }
    for owned in &owns {
        let owned_path = Path::new(owned);
        if owned_path.is_absolute()
            || owned_path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            errors.push(format!(
                "{}:{line}: `{id}` has unsafe owned path `{owned}`",
                path.display()
            ));
        }
    }
    packages.push(WorkPackage {
        id,
        status,
        depends_on: list("Depends on")
            .into_iter()
            .filter(|value| value != "none")
            .collect(),
        owns,
        objective,
        evidence: fields.get("Evidence").cloned().unwrap_or_default(),
    });
}

fn validate_packages(path: &Path, packages: &[WorkPackage], errors: &mut Vec<String>) {
    let ids = packages
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != packages.len() {
        errors.push(format!(
            "{}: work-package ids are not unique",
            path.display()
        ));
    }
    for package in packages {
        for dependency in &package.depends_on {
            if dependency == &package.id {
                errors.push(format!(
                    "{}: `{}` depends on itself",
                    path.display(),
                    package.id
                ));
            } else if !ids.contains(dependency.as_str()) {
                errors.push(format!(
                    "{}: `{}` depends on unknown package `{dependency}`",
                    path.display(),
                    package.id
                ));
            }
        }
    }
    for left in packages {
        for right in packages {
            if left.id >= right.id {
                continue;
            }
            for left_path in &left.owns {
                if right
                    .owns
                    .iter()
                    .any(|right_path| paths_overlap(left_path, right_path))
                {
                    errors.push(format!(
                        "{}: `{}` and `{}` have overlapping ownership at `{left_path}`",
                        path.display(),
                        left.id,
                        right.id
                    ));
                }
            }
        }
    }
    for package in packages {
        let mut visiting = BTreeSet::new();
        if has_cycle(&package.id, packages, &mut visiting) {
            errors.push(format!(
                "{}: dependency cycle reaches `{}`",
                path.display(),
                package.id
            ));
            break;
        }
    }
}

fn has_cycle<'a>(
    id: &'a str,
    packages: &'a [WorkPackage],
    visiting: &mut BTreeSet<&'a str>,
) -> bool {
    if !visiting.insert(id) {
        return true;
    }
    let cyclic = packages
        .iter()
        .find(|item| item.id == id)
        .into_iter()
        .flat_map(|item| &item.depends_on)
        .any(|dependency| has_cycle(dependency, packages, visiting));
    visiting.remove(id);
    cyclic
}

pub fn eligible_packages(packages: &[WorkPackage]) -> Vec<&WorkPackage> {
    packages
        .iter()
        .filter(|package| {
            package.status == PackageStatus::Pending
                && package.depends_on.iter().all(|dependency| {
                    packages.iter().any(|candidate| {
                        candidate.id == *dependency && candidate.status == PackageStatus::Complete
                    })
                })
        })
        .collect()
}

pub fn package_instructions(root: &Path, package: &WorkPackage) -> String {
    format!(
        "Implement Azimuth work package `{}` for change `{}`.\n\nObjective: {}\nOwned paths: {}\nEvidence: {}\n\nRead proposal.md, design.md, verification.md and plan.md when present. Do not edit outside the owned paths. Run the package evidence and report changed files, commands and residuals to the coordinator. Do not finalize or archive the change.\n",
        package.id,
        root.file_name().and_then(|value| value.to_str()).unwrap_or("unknown"),
        package.objective,
        display_list(&package.owns),
        if package.evidence.is_empty() { "declared in the change verification plan" } else { &package.evidence },
    )
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".into()
    } else {
        values.join(", ")
    }
}

fn comma_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn paths_overlap(left: &str, right: &str) -> bool {
    let left = left.trim_end_matches('/');
    let right = right.trim_end_matches('/');
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn field(source: &str, name: &str) -> Option<String> {
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&format!("{name}:"))
            .map(|value| value.trim().trim_matches('*').to_string())
    })
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.starts_with('-')
        || id.ends_with('-')
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!(
            "invalid id `{id}`; use lowercase letters, digits and interior hyphens"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temporary_root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "azimuth-workflow-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn initialization_is_additive_and_idempotent() {
        let root = temporary_root().join("azimuth");
        let first = initialize(&root).unwrap();
        let second = initialize(&root).unwrap();

        assert!(!first.is_empty());
        assert!(second.is_empty());
        assert!(root.join("standards/verification.md").is_file());
        assert!(root.join("workspace.json").is_file());
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }

    #[test]
    fn change_creation_uses_the_lightweight_shape() {
        let root = temporary_root();
        let change = create_change(&root.join("changes"), "show-density", "Show density").unwrap();

        assert!(change.join("proposal.md").is_file());
        assert!(change.join("plan.md").is_file());
        assert!(change.join("specs").is_dir());
        assert!(!change.join("design.md").exists());
        assert!(!change.join("verification.md").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn work_packages_form_a_non_overlapping_dag_and_expose_the_frontier() {
        let root = temporary_root();
        fs::write(
            root.join("work-packages.md"),
            "# Work packages: x\n\n## Work package: contract\nStatus: complete\nDepends on: none\nOwns: packages/contracts\nObjective: Freeze contracts\nEvidence: contract tests\n\n## Work package: service\nStatus: pending\nDepends on: contract\nOwns: app/service\nObjective: Build service\nEvidence: component tests\n",
        )
        .unwrap();

        let packages = load_work_packages(&root).unwrap();
        let eligible = eligible_packages(&packages);

        assert_eq!(
            eligible
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["service"]
        );
        assert!(package_instructions(&root, eligible[0]).contains("Do not edit outside"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn overlapping_ownership_and_cycles_fail_closed() {
        let root = temporary_root();
        fs::write(
            root.join("work-packages.md"),
            "# Work packages: x\n\n## Work package: first\nStatus: pending\nDepends on: second\nOwns: app/service\nObjective: First\n\n## Work package: second\nStatus: pending\nDepends on: first\nOwns: app/service/api\nObjective: Second\n",
        )
        .unwrap();

        let errors = load_work_packages(&root).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.contains("overlapping ownership")));
        assert!(errors
            .iter()
            .any(|error| error.contains("dependency cycle")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn work_package_fields_and_owned_paths_fail_closed() {
        let root = temporary_root();
        fs::write(
            root.join("work-packages.md"),
            "# Work packages: x\n\n## Work package: unsafe\nStatus: pending\nStatus: complete\nDepends on: none\nOwns: ../shared\nObjective: First\nSurprise: hidden policy\n",
        )
        .unwrap();

        let errors = load_work_packages(&root).unwrap_err();

        assert!(errors.iter().any(|error| error.contains("duplicate")));
        assert!(errors.iter().any(|error| error.contains("unknown")));
        assert!(errors.iter().any(|error| error.contains("unsafe")));
        fs::remove_dir_all(root).unwrap();
    }
}
