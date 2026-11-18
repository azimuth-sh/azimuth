//! Where the readers take file content from: the working tree on disk, or a tree that git holds.
//! `azimuth change diff` reads the model and the markers of two trees with the same readers.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::git::Entry;

pub trait Source {
    fn read(&self, path: &str) -> Option<Vec<u8>>;

    /// The names of the folders directly inside a folder, or `None` when the folder is absent.
    fn folders(&self, folder: &str) -> Option<Vec<String>>;

    fn read_text(&self, path: &str) -> Option<String> {
        self.read(path).and_then(|bytes| String::from_utf8(bytes).ok())
    }
}

pub struct Disk<'a> {
    pub root: &'a Path,
}

impl Source for Disk<'_> {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        std::fs::read(self.root.join(path)).ok()
    }

    fn folders(&self, folder: &str) -> Option<Vec<String>> {
        let entries = std::fs::read_dir(self.root.join(folder)).ok()?;
        Some(
            entries
                .flatten()
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect(),
        )
    }
}

/// The files of a tree as git holds them.
pub struct Stored<'a> {
    pub blobs: BTreeMap<String, Vec<u8>>,
    pub files: &'a BTreeMap<String, Entry>,
}

impl Source for Stored<'_> {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let entry = self.files.get(path)?;
        if entry.mode == "120000" || entry.mode == "160000" {
            return None;
        }
        self.blobs.get(&entry.id).cloned()
    }

    fn folders(&self, folder: &str) -> Option<Vec<String>> {
        let prefix = format!("{folder}/");
        let mut found = false;
        let mut folders = BTreeSet::new();
        for path in self.files.keys().filter_map(|path| path.strip_prefix(&prefix)) {
            found = true;
            if let Some((name, _)) = path.split_once('/') {
                folders.insert(name.to_string());
            }
        }
        found.then(|| folders.into_iter().collect())
    }
}
