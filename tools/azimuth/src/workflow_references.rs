//! A completed change can move without rewriting the exact references in its workflow history.

use crate::json::Json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

fn normal(reference: &str) -> Result<(), String> {
    if reference.is_empty()
        || reference.contains(['\\', '\0', '#'])
        || reference
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || Path::new(reference)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("workflow relocation requires a normal portable relative reference".into());
    }
    Ok(())
}

fn regular(root: &Path, relative: &str) -> Result<Option<PathBuf>, String> {
    normal(relative)?;
    let mut path = root.to_path_buf();
    for part in Path::new(relative).components() {
        path.push(part.as_os_str());
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symlink in relocated workflow reference: {}",
                path.display()
            ));
        }
    }
    if !path.is_file() {
        return Err(format!(
            "relocated workflow reference is not a regular file: {}",
            path.display()
        ));
    }
    let canonical = fs::canonicalize(&path).map_err(|error| error.to_string())?;
    if canonical != path || !canonical.starts_with(root) {
        return Err("relocated workflow reference escapes its owning change".into());
    }
    Ok(Some(canonical))
}

fn pinned_change(workflow_directory: &Path) -> Result<Option<String>, String> {
    let Some(path) = regular(workflow_directory, "workflow.json")? else {
        return Ok(None);
    };
    let instance = crate::provenance::read_json(&path)?;
    let definition = instance
        .get("definition")
        .ok_or("conventional workflow has no pinned definition")?;
    let declared = definition
        .get("parameters")
        .and_then(|parameters| parameters.get("change"))
        .and_then(Json::as_str);
    let task_names = definition
        .get("tasks")
        .and_then(Json::as_array)
        .unwrap_or(&[])
        .iter()
        .filter_map(|task| task.get("target")?.get("change")?.as_str())
        .collect::<BTreeSet<_>>();
    let change = match declared {
        Some(change) => {
            if task_names.iter().any(|target| *target != change) {
                return Err(
                    "workflow relocation has conflicting declared change identities".into(),
                );
            }
            change
        }
        None if task_names.len() == 1 => *task_names.iter().next().unwrap(),
        _ => return Err("workflow relocation requires one explicit pinned change identity".into()),
    };
    crate::diag::validate_id(change, false)?;
    Ok(Some(change.to_string()))
}

fn date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return false;
    }
    let Ok(year) = value[..4].parse::<u32>() else {
        return false;
    };
    let Ok(month) = value[5..7].parse::<usize>() else {
        return false;
    };
    let Ok(day) = value[8..10].parse::<u32>() else {
        return false;
    };
    let leap = year % 400 == 0 || (year % 4 == 0 && year % 100 != 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    (1..=12).contains(&month) && day > 0 && day <= days[month - 1]
}

fn retained(change: &Path, candidates: &[String]) -> Result<Option<PathBuf>, String> {
    let Some(manifest) = regular(change, "retained/manifest.json")? else {
        return Ok(None);
    };
    let manifest = crate::retained_inputs::verify(&manifest)?;
    let inputs = manifest
        .get("inputs")
        .and_then(Json::as_array)
        .ok_or("retained manifest has no inputs")?;
    for candidate in candidates {
        normal(candidate)?;
        if inputs.iter().any(|input| {
            input.get("source_path").and_then(Json::as_str) == Some(candidate.as_str())
        }) {
            return crate::retained_inputs::resolve(change, candidate).map(Some);
        }
    }
    Ok(None)
}

/// Resolves only the current conventional change's known old/current locators. It never searches
/// for similarly named changes or alters a recorded locator/fingerprint.
pub fn relocated(
    workflow_directory: &Path,
    repository_root: &Path,
    reference: &str,
) -> Result<Option<PathBuf>, String> {
    normal(reference)?;
    let directory = fs::canonicalize(workflow_directory)
        .map_err(|error| format!("{}: {error}", workflow_directory.display()))?;
    if directory != workflow_directory
        || directory.file_name().and_then(|name| name.to_str()) != Some(".azimuth")
    {
        return Ok(None);
    }
    let Some(change) = directory.parent() else {
        return Ok(None);
    };
    let Some(parent) = change.parent() else {
        return Ok(None);
    };
    let archived = parent.file_name().and_then(|name| name.to_str()) == Some("archive");
    let changes = if archived {
        let Some(changes) = parent.parent() else {
            return Ok(None);
        };
        changes
    } else {
        parent
    };
    if changes.file_name().and_then(|name| name.to_str()) != Some("changes") {
        return Ok(None);
    }
    let Some(identity) = pinned_change(&directory)? else {
        return Ok(None);
    };
    let name = change
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("change directory requires a UTF-8 name")?;
    if archived {
        let Some(prefix) = name.strip_suffix(&format!("-{identity}")) else {
            return Err("archive directory differs from its pinned workflow change".into());
        };
        if !date(prefix) {
            return Err("workflow archive requires YYYY-MM-DD-<pinned-change-id>".into());
        }
        if regular(change, "finalization.json")?.is_none() {
            return Err("relocated archived workflow has no finalization receipt".into());
        }
    } else if name != identity {
        return Err("active change directory differs from its pinned workflow change".into());
    }
    let proposal =
        regular(change, "proposal.md")?.ok_or("conventional workflow has no owning proposal")?;
    let source = fs::read_to_string(&proposal).map_err(|error| error.to_string())?;
    if !source.lines().any(|line| line.starts_with("# Change: ")) {
        return Err("owning proposal has no Change heading".into());
    }
    let ids = source
        .lines()
        .take_while(|line| !line.starts_with("## "))
        .filter_map(|line| line.strip_prefix("Id:").map(str::trim))
        .collect::<Vec<_>>();
    if ids.len() > 1 || ids.first().is_some_and(|id| *id != identity) {
        return Err("proposal Id differs from pinned workflow identity".into());
    }
    let repository = fs::canonicalize(repository_root)
        .map_err(|error| format!("{}: {error}", repository_root.display()))?;
    if repository != repository_root {
        return Err("workflow repository root must be canonical".into());
    }
    if repository == directory {
        return retained(change, &[format!(".azimuth/{reference}")]);
    }
    if !change.starts_with(&repository) {
        return Ok(None);
    }
    let old_prefix = changes.join(&identity);
    let old_prefix = old_prefix
        .strip_prefix(&repository)
        .map_err(|_| "change root is outside selected repository")?;
    let current_prefix = change
        .strip_prefix(&repository)
        .map_err(|_| "change root is outside selected repository")?;
    let relative = Path::new(reference);
    let tail = relative
        .strip_prefix(old_prefix)
        .or_else(|_| relative.strip_prefix(current_prefix));
    let Ok(tail) = tail else {
        return Ok(None);
    };
    let tail = tail.to_str().ok_or("workflow reference must be UTF-8")?;
    normal(tail)?;
    if let Some(actual) = regular(change, tail)? {
        return Ok(Some(actual));
    }
    retained(change, &[tail.to_string(), reference.to_string()])
}
