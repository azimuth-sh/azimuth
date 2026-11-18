//! What a change did to the model, and its class, computed from what differs between two trees:
//! the common ancestor with the target, and the working tree. The preview builds nothing, so it
//! knows declarations and marker lines and never test names.

use std::collections::{BTreeMap, BTreeSet};

use crate::git::Entry;
use crate::markers::Marker;
use crate::model::{Model, MODEL_FOLDER};
use crate::project::{Project, PROJECT_FILE};
use crate::skills;

/// One side of the comparison.
pub struct Side<'a> {
    pub files: &'a BTreeMap<String, Entry>,
    /// `None` when the project file of that tree cannot be read.
    pub project: Option<&'a Project>,
    pub model: &'a Model,
    pub markers: &'a [Marker],
}

#[derive(Default)]
pub struct Lists {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub removed: Vec<String>,
}

pub struct Change {
    pub class: char,
    pub requirements: Lists,
    pub scenarios: Lists,
    pub checks: Lists,
    pub markers_removed: usize,
    pub not_compared: Vec<String>,
    pub rigor_lowered: Vec<Lowered>,
}

/// One rigor declaration that the change removed from a Requirement or a Check it kept.
pub struct Lowered {
    /// `reuse`, `verification` or `realization`.
    pub kind: &'static str,
    /// `check` or `requirement`.
    pub subject: &'static str,
    pub id: String,
}

pub fn compare(before: &Side, after: &Side) -> Change {
    let differing = crate::tree::differing_paths(before.files, after.files);
    let marked_files: BTreeSet<&str> = before.markers.iter().chain(after.markers).map(|marker| marker.file.as_str()).collect();

    let requirements = requirement_lists(before.model, after.model);
    let scenarios = scenario_lists(before.model, after.model);
    let checks = check_lists(before.model, after.model);
    let markers_removed = markers_removed(before, after);
    let rigor_lowered = rigor_lowered(before.model, after.model);

    let mut class = differing.iter().map(|path| file_class(path, &marked_files, before, after)).max().unwrap_or('A');
    // Removing proof and loosening rigor are changes of intent, whatever file they sit in.
    if !checks.removed.is_empty() || markers_removed > 0 || !rigor_lowered.is_empty() {
        class = 'D';
    }

    let not_compared = differing.iter().filter(|path| before.files.contains_key(*path) && marked_files.contains(path.as_str())).cloned().collect();
    Change { class, requirements, scenarios, checks, markers_removed, not_compared, rigor_lowered }
}

/// The declarations that Requirements and Checks present on both sides lost. A Requirement or a
/// Check removed whole is on its removed list instead.
fn rigor_lowered(before: &Model, after: &Model) -> Vec<Lowered> {
    let mut lowered = Vec::new();
    for requirement in &after.requirements {
        let Some(old) = before.requirement(&requirement.id) else {
            continue;
        };
        for (kind, had, has) in [
            ("verification", old.verification_review, requirement.verification_review),
            ("realization", old.realization_review, requirement.realization_review),
        ] {
            if had && !has {
                lowered.push(Lowered { kind, subject: "requirement", id: requirement.id.clone() });
            }
        }
    }
    for check in &after.checks {
        if check.reuse.is_none() && before.check(&check.id).is_some_and(|old| old.reuse.is_some()) {
            lowered.push(Lowered { kind: "reuse", subject: "check", id: check.id.clone() });
        }
    }
    lowered
}

/// The class of one changed file comes from where it is, and a doubt counts as the judge.
fn file_class(path: &str, marked_files: &BTreeSet<&str>, before: &Side, after: &Side) -> char {
    let in_model = path.starts_with(&format!("{MODEL_FOLDER}/"));
    let name = path.rsplit('/').next().unwrap_or(path);
    if path == PROJECT_FILE || (in_model && name == "specification.md") || is_installed_skill(path) {
        return 'D';
    }
    let in_a_suite = [before.project, after.project]
        .into_iter()
        .flatten()
        .any(|project| project.suites.iter().any(|suite| suite.contains(path)));
    if (in_model && name == "verification.md") || marked_files.contains(path) || in_a_suite {
        return 'C';
    }
    if in_model && name == "realization.md" {
        return 'B';
    }
    'A'
}

fn is_installed_skill(path: &str) -> bool {
    skills::INSTALLED_PATHS.iter().any(|installed| path == *installed || path.starts_with(&format!("{installed}/")))
}

/// Text compared without the blank lines at its ends, which carry no meaning.
fn content(lines: &[String]) -> &[String] {
    let start = lines.iter().position(|line| !line.trim().is_empty()).unwrap_or(lines.len());
    let end = lines.iter().rposition(|line| !line.trim().is_empty()).map_or(start, |end| end + 1);
    &lines[start..end]
}

fn requirement_lists(before: &Model, after: &Model) -> Lists {
    let mut lists = Lists::default();
    for requirement in &after.requirements {
        match before.requirement(&requirement.id) {
            None => lists.added.push(requirement.id.clone()),
            Some(old) => {
                let same = content(&old.text) == content(&requirement.text)
                    && old.module == requirement.module
                    && old.verification_review == requirement.verification_review
                    && old.realization_review == requirement.realization_review;
                if !same {
                    lists.changed.push(requirement.id.clone());
                }
            }
        }
    }
    for requirement in &before.requirements {
        if after.requirement(&requirement.id).is_none() {
            lists.removed.push(requirement.id.clone());
        }
    }
    lists
}

fn scenarios_of(model: &Model) -> BTreeMap<String, &[String]> {
    let mut scenarios = BTreeMap::new();
    for requirement in &model.requirements {
        for scenario in &requirement.scenarios {
            scenarios.insert(format!("{}/{}", requirement.id, scenario.id), content(&scenario.text));
        }
    }
    scenarios
}

fn scenario_lists(before: &Model, after: &Model) -> Lists {
    let (old, new) = (scenarios_of(before), scenarios_of(after));
    let mut lists = Lists::default();
    for (id, text) in &new {
        match old.get(id) {
            None => lists.added.push(id.clone()),
            Some(old_text) if old_text != text => lists.changed.push(id.clone()),
            Some(_) => {}
        }
    }
    lists.removed = old.keys().filter(|id| !new.contains_key(*id)).cloned().collect();
    lists
}

fn check_lists(before: &Model, after: &Model) -> Lists {
    let mut lists = Lists::default();
    for check in &after.checks {
        match before.check(&check.id) {
            None => lists.added.push(check.id.clone()),
            Some(old) => {
                let same = old.claim == check.claim
                    && old.requirement == check.requirement
                    && old.scenario == check.scenario
                    && old.reuse.as_ref().map(|reuse| (reuse.value, &reuse.reason)) == check.reuse.as_ref().map(|reuse| (reuse.value, &reuse.reason));
                if !same {
                    lists.changed.push(check.id.clone());
                }
            }
        }
    }
    lists.removed = before.checks.iter().filter(|check| after.check(&check.id).is_none()).map(|check| check.id.clone()).collect();
    lists
}

/// The marker lines that the Checks still declared lost against the target. A removed Check is
/// listed as removed instead.
fn markers_removed(before: &Side, after: &Side) -> usize {
    let count = |markers: &[Marker], id: &str| markers.iter().filter(|marker| marker.check == id).count();
    after
        .model
        .checks
        .iter()
        .map(|check| count(before.markers, &check.id).saturating_sub(count(after.markers, &check.id)))
        .sum()
}
