//! `azimuth lint`: is the description consistent with itself and with the code?

use serde_json::json;

use crate::source::Disk;
use crate::lint::{self, Finding};
use crate::markers;
use crate::output::{self, status};
use crate::store::WORKTREE_FOLDER;
use crate::tree::Tree;

pub fn run(json: bool) -> i32 {
    let workspace = match output::open_workspace(json) {
        Ok(workspace) => workspace,
        Err(status) => return status,
    };
    output::warn_on_other_version(&workspace);
    let git = &workspace.git;
    let tree = match Tree::current(git) {
        Ok(tree) => tree,
        Err(message) => return output::could_not(&message),
    };
    let tracked = match git.tracked_under(WORKTREE_FOLDER) {
        Ok(tracked) => tracked,
        Err(message) => return output::could_not(&message),
    };
    let markers = markers::scan(&Disk { root: git.root() }, &workspace.project, &tree.files);
    let findings = lint::findings(&workspace.model, &markers, &tracked);

    if json {
        output::document(&json!({ "format": "lint-1", "findings": findings.iter().map(Finding::to_json).collect::<Vec<_>>() }));
    } else if findings.is_empty() {
        println!("lint: no finding");
    } else {
        for finding in &findings {
            println!("{}", finding.to_text());
        }
        println!("lint: {} finding{}", findings.len(), if findings.len() == 1 { "" } else { "s" });
    }
    if findings.is_empty() {
        status::FINE
    } else {
        status::NOT_FINE
    }
}
