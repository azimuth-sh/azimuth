//! Where the tree stands: each Check's fingerprint on the current tree and its state from the
//! local records. Reading commands and the planning of a run both start here.

use std::collections::BTreeMap;

use crate::source::Disk;
use crate::fingerprint::{self, Fingerprint};
use crate::markers::{self, Marker};
use crate::state::{Evaluation, Evaluator};
use crate::store::{Records, Store};
use crate::tree::Tree;
use crate::workspace::Workspace;

pub struct Survey<'w> {
    pub workspace: &'w Workspace,
    pub tree: Tree,
    pub object_format: String,
    pub store: Store,
    pub records: Records,
    pub markers: Vec<Marker>,
    pub fingerprints: BTreeMap<String, Fingerprint>,
    pub evaluations: BTreeMap<String, Evaluation>,
}

impl<'w> Survey<'w> {
    pub fn take(workspace: &'w Workspace) -> Result<Survey<'w>, String> {
        let git = &workspace.git;
        let tree = Tree::current(git)?;
        let object_format = git.object_format()?;
        let store = Store::new(git.root(), &git.common_dir()?);
        let records = store.read();
        let markers = markers::scan(&Disk { root: git.root() }, &workspace.project, &tree.files);
        let files = tree.files_digest();
        let fingerprints: BTreeMap<String, Fingerprint> = workspace
            .model
            .checks
            .iter()
            .map(|check| (check.id.clone(), fingerprint::of_check(&check.claim, &workspace.model.condition_of(check), &files, &object_format)))
            .collect();
        let evaluator = Evaluator::new(git, &tree);
        let evaluations = workspace
            .model
            .checks
            .iter()
            .map(|check| {
                let results: Vec<_> = records.of_check(&check.id).collect();
                (check.id.clone(), evaluator.evaluate(check, &fingerprints[&check.id], &results))
            })
            .collect();
        Ok(Survey { workspace, tree, object_format, store, records, markers, fingerprints, evaluations })
    }

    pub fn suite_of(&self, check: &str) -> Option<&str> {
        markers::suite_of(&self.markers, &self.workspace.project, check)
    }
}
