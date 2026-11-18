//! `azimuth change diff`: what did this change do to the model? It compares the working tree
//! with the common ancestor of the current commit and the target branch, and never fetches.

use serde_json::{json, Value};

use crate::change::{self, Change, Lists, Side};
use crate::git::Git;
use crate::markers;
use crate::model;
use crate::output::{self, status};
use crate::project;
use crate::source::{Disk, Stored};
use crate::tree::Tree;

pub fn run(target: Option<String>, json: bool) -> i32 {
    let workspace = match output::open_workspace(json) {
        Ok(workspace) => workspace,
        Err(status) => return status,
    };
    output::warn_on_other_version(&workspace);
    let git = &workspace.git;
    let target = target.unwrap_or_else(|| workspace.project.repository.target.clone());
    let base = match common_commit(git, &target) {
        Ok(base) => base,
        Err(message) => return output::could_not(&message),
    };
    let Some(base_files) = git.tree_of(&base).and_then(|tree| git.tree_entries(&tree)) else {
        return output::could_not(&format!("git does not have the tree of the commit {base}"));
    };
    let tree = match Tree::current(git) {
        Ok(tree) => tree,
        Err(message) => return output::could_not(&message),
    };

    let stored = match Stored::new(git, &base_files) {
        Ok(stored) => stored,
        Err(message) => return output::could_not(&message),
    };
    let base_project = project::read(&stored).ok();
    // A model that the target cannot read compares as what could be read of it; the working
    // tree, which this change is about, reads without errors or the command stopped above.
    let (base_model, _) = model::read(&stored);
    let base_markers = markers::scan(&stored, base_project.as_ref().unwrap_or(&workspace.project), &base_files);
    if let Some(message) = stored.error() {
        return output::could_not(&message);
    }
    let markers_now = markers::scan(&Disk { root: git.root() }, &workspace.project, &tree.files);

    let change = change::compare(
        &Side { files: &base_files, project: base_project.as_ref(), model: &base_model, markers: &base_markers },
        &Side { files: &tree.files, project: Some(&workspace.project), model: &workspace.model, markers: &markers_now },
    );

    if json {
        output::document(&document(&target, &base, &change));
    } else {
        print!("{}", text(&target, &base, &change));
    }
    status::FINE
}

/// The commit to compare with. On the target branch itself the change is the working tree
/// against the last commit. Elsewhere it is the common ancestor with the remote-tracking copy of
/// the target as last fetched, or with the local one when no remote has it.
fn common_commit(git: &Git, target: &str) -> Result<String, String> {
    let head = git.head_commit().ok_or("the branch has no commit yet, so there is no change to compare")?;
    if git.current_branch().as_deref() == Some(target) {
        return Ok(head);
    }
    let mut remotes = git.remotes();
    remotes.sort_by_key(|remote| remote != "origin");
    let reference = remotes
        .iter()
        .map(|remote| format!("refs/remotes/{remote}/{target}"))
        .find(|reference| git.commit_of(reference).is_some())
        .unwrap_or_else(|| target.to_string());
    if git.commit_of(&reference).is_none() {
        return Err(format!("the target `{target}` is not in this clone. Name another with `--target <ref>`"));
    }
    git.merge_base(&head, &reference).ok_or_else(|| {
        format!("the common commit is not in this clone: git holds no commit shared by HEAD and `{target}`. Fetch the missing history, for example with `git fetch --unshallow`")
    })
}

fn lists_json(lists: &Lists) -> Value {
    json!({ "added": lists.added, "changed": lists.changed, "removed": lists.removed })
}

fn document(target: &str, base: &str, change: &Change) -> Value {
    json!({
        "format": "change-1",
        "target": target,
        "base": base,
        "class": change.class.to_string(),
        "requirements": lists_json(&change.requirements),
        "scenarios": lists_json(&change.scenarios),
        "checks": lists_json(&change.checks),
        "markers_removed": change.markers_removed,
        "not_compared": change.not_compared,
        "rigor_lowered": change.rigor_lowered.iter().map(|lowered| json!({ "kind": lowered.kind, lowered.subject: lowered.id })).collect::<Vec<_>>(),
    })
}

fn text(target: &str, base: &str, change: &Change) -> String {
    let mut text = format!("class {}, compared with {target} at {}\n", change.class, &base[..base.len().min(7)]);
    for (noun, lists) in [("Requirements", &change.requirements), ("Scenarios", &change.scenarios), ("Checks", &change.checks)] {
        for (which, ids) in [("added", &lists.added), ("changed", &lists.changed), ("removed", &lists.removed)] {
            if !ids.is_empty() {
                text.push_str(&format!("{noun} {which}:\n"));
                for id in ids {
                    text.push_str(&format!("  {id}\n"));
                }
            }
        }
    }
    if change.markers_removed > 0 {
        let noun = if change.markers_removed == 1 { "marker" } else { "markers" };
        text.push_str(&format!("{} {noun} removed\n", change.markers_removed));
    }
    for lowered in &change.rigor_lowered {
        let (noun, line) = match lowered.kind {
            "verification" => ("Requirement", "`Verification: reviewed`"),
            "realization" => ("Requirement", "`Realization: reviewed`"),
            _ => ("Check", "`Reuse:` line"),
        };
        text.push_str(&format!("rigor lowered: the {noun} {} lost its {line}\n", lowered.id));
    }
    for path in &change.not_compared {
        text.push_str(&format!("{path}: tests in this file were not compared\n"));
    }
    text
}
