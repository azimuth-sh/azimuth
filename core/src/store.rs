//! The local records: the outbox in the worktree, which every session may write, and the store in
//! the clone's shared git directory, which `azimuth sync` fills. Both are plain files named by
//! the digest of their own bytes, and reading uses both.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::canonical::canonical_json;
use crate::digest::sha256_hex;
use crate::result::StoredResult;

/// The folder of the worktree that ignores itself.
pub const WORKTREE_FOLDER: &str = ".azimuth";

pub struct Store {
    worktree_folder: PathBuf,
    outbox: PathBuf,
    store: PathBuf,
}

/// The folder holds a `.gitignore` that contains `*`, so no record can be committed by mistake.
/// Returns whether the ignore file was written now.
pub fn ensure_worktree_folder(folder: &Path) -> Result<bool, String> {
    fs::create_dir_all(folder).map_err(|error| format!("create {}: {error}", folder.display()))?;
    let ignore = folder.join(".gitignore");
    if ignore.exists() {
        return Ok(false);
    }
    fs::write(&ignore, "*\n").map_err(|error| format!("write {}: {error}", ignore.display()))?;
    Ok(true)
}

pub struct Records {
    pub results: Vec<StoredResult>,
    /// Files this version did not use: a format it cannot read, or bytes that do not match the name.
    pub skipped: usize,
}

impl Records {
    pub fn of_check<'r>(&'r self, check: &'r str) -> impl Iterator<Item = &'r StoredResult> + 'r {
        self.results.iter().filter(move |result| result.check == check)
    }
}

pub struct SyncReport {
    pub waiting: usize,
}

impl Store {
    pub fn new(root: &Path, common_dir: &Path) -> Store {
        let worktree_folder = root.join(WORKTREE_FOLDER);
        Store { outbox: worktree_folder.join("outbox"), worktree_folder, store: common_dir.join("azimuth") }
    }

    /// Writes a record into the outbox under the digest of its canonical bytes.
    pub fn write_to_outbox(&self, record: &Value) -> Result<(), String> {
        ensure_worktree_folder(&self.worktree_folder)?;
        let bytes = canonical_json(record)?;
        fs::create_dir_all(&self.outbox).map_err(|error| format!("create {}: {error}", self.outbox.display()))?;
        let path = self.outbox.join(record_name(bytes.as_bytes()));
        fs::write(&path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
    }

    /// The local step of sync: every record of the outbox moves into the store under the same
    /// name. A record that cannot be moved, because the session may not write the store, waits.
    pub fn sync(&self) -> SyncReport {
        let mut waiting = 0;
        for file in json_files(&self.outbox) {
            if self.move_into_store(&file).is_err() {
                waiting += 1;
            }
        }
        SyncReport { waiting }
    }

    fn move_into_store(&self, file: &Path) -> Result<(), ()> {
        let bytes = fs::read(file).map_err(|_| ())?;
        let name = file.file_name().ok_or(())?.to_string_lossy().to_string();
        if name != record_name(&bytes) {
            return Err(());
        }
        let record: Value = serde_json::from_slice(&bytes).map_err(|_| ())?;
        let folder = match record["subjects"][0]["name"].as_str() {
            Some(check) if crate::model::is_valid_id(check) => self.store.join("cache/results").join(check),
            _ => self.store.join("cache/records"),
        };
        fs::create_dir_all(&folder).map_err(|_| ())?;
        let target = folder.join(&name);
        if !target.exists() {
            let partial = folder.join(format!("{name}.partial"));
            fs::write(&partial, &bytes).map_err(|_| ())?;
            fs::rename(&partial, &target).map_err(|_| ())?;
        }
        fs::remove_file(file).map_err(|_| ())
    }

    /// Every result in the store and the outbox. A file counts once, wherever it lies.
    pub fn read(&self) -> Records {
        let mut seen = BTreeSet::new();
        let mut results = Vec::new();
        let mut skipped = 0;
        for file in json_files(&self.store).into_iter().chain(json_files(&self.outbox)) {
            let name = file.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
            if !seen.insert(name.clone()) {
                continue;
            }
            let Ok(bytes) = fs::read(&file) else {
                skipped += 1;
                continue;
            };
            let result = (name == record_name(&bytes))
                .then(|| serde_json::from_slice::<Value>(&bytes).ok())
                .flatten()
                .and_then(|record| StoredResult::from_record(&name, &record));
            match result {
                Some(result) => results.push(result),
                None => skipped += 1,
            }
        }
        Records { results, skipped }
    }
}

/// `record-1`: the file name is the SHA-256 of the record's bytes, followed by `.json`.
fn record_name(bytes: &[u8]) -> String {
    format!("{}.json", sha256_hex(bytes))
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_json(dir, &mut files);
    files.sort();
    files
}

fn collect_json(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            collect_json(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
}
