//! The state of one Check on the current tree, from the results stored for it. A result is valid
//! when it is a verified pass with a fingerprint equal to the current one.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::digest::{self, FINGERPRINT_SCHEMES};
use crate::fingerprint::{Fingerprint, PART_NAMES};
use crate::git::{Entry, Git};
use crate::model::Check;
use crate::result::{Outcome, StoredResult};
use crate::tree::{self, Tree};

/// The reasons for owed, as the design names them. A `reason` field holds exactly one of them.
pub mod reason {
    pub const INPUT_CHANGED: &str = "an input changed";
    pub const NEVER_RUN: &str = "never run";
    pub const CANNOT_RUN_HERE: &str = "cannot run here";
    pub const FAILED_ON_THESE_INPUTS: &str = "failed on these inputs";
    pub const CONTRADICTED: &str = "contradicted";
    pub const NO_RESULT: &str = "no result";
    pub const CONTEXT_NOT_APPLIED: &str = "context not applied";
    pub const RUNS_IN_CI: &str = "runs in CI";
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Kept,
    Owed,
}

impl State {
    pub fn as_str(&self) -> &'static str {
        match self {
            State::Kept => "kept",
            State::Owed => "owed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    pub state: State,
    pub reason: Option<&'static str>,
    pub detail: Option<String>,
    pub parts_changed: Vec<String>,
    pub files_changed: Vec<String>,
    pub failing_tests: Vec<String>,
    /// The result the state is explained by: the kept pass, the known fail, or the last result
    /// before an input changed.
    pub reference: Option<StoredResult>,
}

impl Evaluation {
    fn owed(reason: &'static str, detail: Option<String>, reference: Option<StoredResult>) -> Evaluation {
        Evaluation {
            state: State::Owed,
            reason: Some(reason),
            detail,
            parts_changed: Vec::new(),
            files_changed: Vec::new(),
            failing_tests: Vec::new(),
            reference,
        }
    }
}

pub struct Evaluator<'a> {
    git: &'a Git,
    tree: &'a Tree,
    /// Stored trees rebuilt from git, by base and edits, so each is listed once per command.
    stored_trees: RefCell<HashMap<String, Option<BTreeMap<String, Entry>>>>,
}

impl<'a> Evaluator<'a> {
    pub fn new(git: &'a Git, tree: &'a Tree) -> Evaluator<'a> {
        Evaluator { git, tree, stored_trees: RefCell::new(HashMap::new()) }
    }

    /// The state of a Check on a developer's machine.
    pub fn evaluate(&self, check: &Check, current: &Fingerprint, results: &[&StoredResult]) -> Evaluation {
        let mut ordered: Vec<&StoredResult> = results.to_vec();
        // `at` is fixed-width UTC text, so its order is the order of time.
        ordered.sort_by(|a, b| (&a.at, &a.file_name).cmp(&(&b.at, &b.file_name)));

        if let Some(reuse) = &check.reuse {
            // A `never` result is bound to its run, so no stored result counts.
            let detail = format!("Reuse: {}, {}", reuse.value.as_str(), reuse.reason);
            return Evaluation::owed(reason::RUNS_IN_CI, Some(detail), ordered.last().map(|result| (*result).clone()));
        }

        let on_these_inputs: Vec<&StoredResult> = ordered.iter().copied().filter(|result| result.fingerprint == current.value).collect();
        let last_pass = on_these_inputs.iter().rev().find(|result| result.outcome == Outcome::Passed);
        let last_fail = on_these_inputs.iter().rev().find(|result| result.outcome == Outcome::Failed);
        match (last_pass, last_fail) {
            (Some(_), Some(fail)) => {
                let mut evaluation = Evaluation::owed(
                    reason::CONTRADICTED,
                    Some("a pass and a fail were stored on these inputs".to_string()),
                    Some((*fail).clone()),
                );
                evaluation.failing_tests = fail.failing_tests();
                return evaluation;
            }
            (Some(pass), None) => {
                return Evaluation {
                    state: State::Kept,
                    reason: None,
                    detail: None,
                    parts_changed: Vec::new(),
                    files_changed: Vec::new(),
                    failing_tests: Vec::new(),
                    reference: Some((*pass).clone()),
                };
            }
            (None, Some(fail)) => {
                let failing = fail.failing_tests();
                let detail = format!("failing tests: {}", failing.join(", "));
                let mut evaluation = Evaluation::owed(reason::FAILED_ON_THESE_INPUTS, Some(detail), Some((*fail).clone()));
                evaluation.failing_tests = failing;
                return evaluation;
            }
            (None, None) => {}
        }

        let Some(last) = ordered.last() else {
            return Evaluation::owed(reason::NEVER_RUN, None, None);
        };
        self.input_changed(last, current)
    }

    /// "An input changed" names the part, and for files the files that differ, from the part
    /// digests stored with the last result.
    fn input_changed(&self, last: &StoredResult, current: &Fingerprint) -> Evaluation {
        let scheme_changes = scheme_changes(last);
        if !scheme_changes.is_empty() {
            let detail = format!("method changed: {}", scheme_changes.join(", "));
            let mut evaluation = Evaluation::owed(reason::INPUT_CHANGED, Some(detail), Some(last.clone()));
            evaluation.parts_changed = vec!["method".to_string()];
            return evaluation;
        }
        let parts_changed: Vec<String> = PART_NAMES
            .iter()
            .filter(|part| last.parts.get(part) != current.parts.get(part))
            .map(|part| part.to_string())
            .collect();
        let files_changed = if parts_changed.iter().any(|part| part == "files") {
            match self.stored_files(last) {
                Some(stored) => tree::differing_paths(&stored, &self.tree.files),
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let mut detail = format!("{} changed", parts_changed.join(", "));
        if !files_changed.is_empty() {
            detail.push_str(&format!(": {}", files_changed.join(", ")));
        }
        let mut evaluation = Evaluation::owed(reason::INPUT_CHANGED, Some(detail), Some(last.clone()));
        evaluation.parts_changed = parts_changed;
        evaluation.files_changed = files_changed;
        evaluation
    }

    fn stored_files(&self, result: &StoredResult) -> Option<BTreeMap<String, Entry>> {
        let key = format!("{} {:?}", result.base, result.edits);
        self.stored_trees
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| tree::stored_files(self.git, &result.base, &result.edits))
            .clone()
    }
}

/// The schemes of a stored result that differ from the ones this version computes, written as
/// "`old` became `new`". A result under another number is not comparable.
fn scheme_changes(result: &StoredResult) -> Vec<String> {
    let mut stored: BTreeSet<&str> = result.schemes.iter().map(String::as_str).collect();
    for written in PART_NAMES.iter().map(|part| result.parts.get(part)).chain([result.fingerprint.as_str()]) {
        if let Some(scheme) = digest::scheme_of(written) {
            stored.insert(scheme);
        }
    }
    let mut changes = Vec::new();
    for current in FINGERPRINT_SCHEMES {
        for old in &stored {
            if *old != current && digest::scheme_word(old) == digest::scheme_word(current) {
                changes.push(format!("`{old}` became `{current}`"));
            }
        }
    }
    changes
}
