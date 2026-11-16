//! The tree a command sees: a commit's tree plus the paths that differ from it. Without an input
//! provider every file of it is an input of every Check.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::digest;
use crate::git::{Entry, Git, WorkingChange};

/// A path that differs from the base tree, as a result stores it.
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Present { path: String, entry: Entry },
    Deleted { path: String },
}

impl Edit {
    pub fn path(&self) -> &str {
        match self {
            Edit::Present { path, .. } | Edit::Deleted { path } => path,
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            Edit::Present { path, entry } => json!({ "path": path, "mode": entry.mode, "id": digest::file_digest(&entry.id) }),
            Edit::Deleted { path } => json!({ "path": path, "deleted": true }),
        }
    }

    /// An edit line of a stored result, or `None` when it is not one this version wrote.
    pub fn from_json(value: &Value) -> Option<Edit> {
        let path = value["path"].as_str()?.to_string();
        if value["deleted"] == Value::Bool(true) {
            return Some(Edit::Deleted { path });
        }
        let id = value["id"].as_str()?.strip_prefix(&format!("{}:", digest::FILE_SCHEME))?.to_string();
        Some(Edit::Present { path, entry: Entry { mode: value["mode"].as_str()?.to_string(), id } })
    }
}

pub struct Tree {
    pub base: String,
    pub edits: Vec<Edit>,
    pub files: BTreeMap<String, Entry>,
}

impl Tree {
    /// What git would commit if everything were staged now.
    pub fn current(git: &Git) -> Result<Tree, String> {
        let base = git.head_tree()?;
        let mut files = git.tree_entries(&base).ok_or_else(|| format!("git does not have the tree {base}"))?;
        let changes = git.working_changes()?;

        let mut present: Vec<(String, String)> = Vec::new();
        let mut deleted: Vec<String> = Vec::new();
        for change in changes {
            match change {
                WorkingChange::Present { path, mode } => present.push((path, mode)),
                WorkingChange::Deleted { path } => deleted.push(path),
            }
        }
        let (links, regular): (Vec<_>, Vec<_>) = present.into_iter().partition(|(_, mode)| mode == "120000");
        let regular_paths: Vec<String> = regular.iter().map(|(path, _)| path.clone()).collect();
        let ids = git.hash_files(&regular_paths)?;
        let mut hashed: Vec<(String, Entry)> =
            regular.into_iter().zip(ids).map(|((path, mode), id)| (path, Entry { mode, id })).collect();
        for (path, mode) in links {
            let id = git.hash_link(&path)?;
            hashed.push((path, Entry { mode, id }));
        }

        let mut edits = Vec::new();
        for (path, entry) in hashed {
            if files.get(&path) == Some(&entry) {
                continue;
            }
            files.insert(path.clone(), entry.clone());
            edits.push(Edit::Present { path, entry });
        }
        for path in deleted {
            if files.remove(&path).is_some() {
                edits.push(Edit::Deleted { path });
            }
        }
        edits.sort_by(|a, b| a.path().cmp(b.path()));
        Ok(Tree { base, edits, files })
    }

    pub fn files_digest(&self) -> String {
        files_digest(&self.files)
    }
}

/// The digest of the files part under `parts-1`: every path with its mode and object id, as
/// git's own tree lists them, so a clean checkout of a commit and a working tree that holds
/// exactly that content give one digest.
pub fn files_digest(files: &BTreeMap<String, Entry>) -> String {
    let mut bytes = b"files\n".to_vec();
    for (path, entry) in files {
        bytes.extend_from_slice(format!("{} {} {}", entry.mode, entry.id, path).as_bytes());
        bytes.push(0);
    }
    digest::parts_digest(&bytes)
}

/// The files of a stored tree, rebuilt from git, or `None` when git no longer has its base.
pub fn stored_files(git: &Git, base: &str, edits: &[Edit]) -> Option<BTreeMap<String, Entry>> {
    let mut files = git.tree_entries(base)?;
    for edit in edits {
        match edit {
            Edit::Present { path, entry } => {
                files.insert(path.clone(), entry.clone());
            }
            Edit::Deleted { path } => {
                files.remove(path);
            }
        }
    }
    Some(files)
}

/// The paths whose mode or content differs between two lists, or that only one of them holds.
pub fn differing_paths(before: &BTreeMap<String, Entry>, now: &BTreeMap<String, Entry>) -> Vec<String> {
    let mut paths: Vec<String> = before
        .iter()
        .filter(|(path, entry)| now.get(*path) != Some(*entry))
        .map(|(path, _)| path.clone())
        .collect();
    paths.extend(now.keys().filter(|path| !before.contains_key(*path)).cloned());
    paths.sort();
    paths
}
