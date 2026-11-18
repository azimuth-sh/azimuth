//! Everything the core asks git. Git is the authority on content: a file's digest is its object
//! id, and the tree is what git would commit if everything were staged now.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Git {
    root: PathBuf,
}

/// A file as git's tree holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub mode: String,
    pub id: String,
}

/// One path of the working tree that differs from the commit it stands on.
#[derive(Debug)]
pub enum WorkingChange {
    /// Edited or new, with the mode git would store.
    Present { path: String, mode: String },
    Deleted { path: String },
}

impl Git {
    /// The repository that holds the current folder, opened at its root.
    pub fn open_here() -> Result<Git, String> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .stderr(Stdio::null())
            .output()
            .map_err(|error| format!("git could not be started: {error}"))?;
        if !output.status.success() {
            return Err("this folder is not inside a git repository".to_string());
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Git { root: PathBuf::from(root) })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new("git");
        // Reading commands must not rewrite the index, which a sandbox may forbid.
        command.arg("--no-optional-locks").args(args).current_dir(&self.root);
        command
    }

    fn output(&self, args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let mut command = self.command(args);
        command.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|error| format!("git could not be started: {error}"))?;
        if let Some(input) = input {
            let mut stdin = child.stdin.take().expect("stdin is piped");
            let input = input.to_vec();
            std::thread::spawn(move || {
                let _ = stdin.write_all(&input);
            });
        }
        let output = child.wait_with_output().map_err(|error| format!("git did not finish: {error}"))?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(format!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim()))
        }
    }

    fn text(&self, args: &[&str]) -> Result<String, String> {
        self.output(args, None).map(|bytes| String::from_utf8_lossy(&bytes).trim().to_string())
    }

    /// The clone's shared git directory, which every worktree of the clone sees.
    pub fn common_dir(&self) -> Result<PathBuf, String> {
        self.text(&["rev-parse", "--path-format=absolute", "--git-common-dir"]).map(PathBuf::from)
    }

    /// `sha1` or `sha256`.
    pub fn object_format(&self) -> Result<String, String> {
        self.text(&["rev-parse", "--show-object-format"])
    }

    pub fn head_commit(&self) -> Option<String> {
        self.text(&["rev-parse", "--verify", "--quiet", "HEAD"]).ok().filter(|id| !id.is_empty())
    }

    /// The tree of the current commit, or the empty tree before the first commit.
    pub fn head_tree(&self) -> Result<String, String> {
        match self.text(&["rev-parse", "--verify", "--quiet", "HEAD^{tree}"]) {
            Ok(tree) if !tree.is_empty() => Ok(tree),
            _ => self.output(&["hash-object", "-t", "tree", "--stdin"], Some(b"")).map(|bytes| String::from_utf8_lossy(&bytes).trim().to_string()),
        }
    }

    /// The short name of the checked out branch, or `None` on a detached head.
    pub fn current_branch(&self) -> Option<String> {
        self.text(&["symbolic-ref", "--quiet", "--short", "HEAD"]).ok().filter(|name| !name.is_empty())
    }

    pub fn remotes(&self) -> Vec<String> {
        self.text(&["remote"]).map(|text| text.lines().map(str::to_string).collect()).unwrap_or_default()
    }

    /// The commit a ref names, or `None` when this clone has no such commit.
    pub fn commit_of(&self, reference: &str) -> Option<String> {
        self.text(&["rev-parse", "--verify", "--quiet", &format!("{reference}^{{commit}}")]).ok().filter(|id| !id.is_empty())
    }

    pub fn merge_base(&self, one: &str, other: &str) -> Option<String> {
        self.text(&["merge-base", one, other]).ok().filter(|id| !id.is_empty())
    }

    pub fn tree_of(&self, commit: &str) -> Option<String> {
        self.text(&["rev-parse", "--verify", "--quiet", &format!("{commit}^{{tree}}")]).ok().filter(|id| !id.is_empty())
    }

    /// The default branch of a remote as this clone last saw it, from `refs/remotes/<remote>/HEAD`.
    pub fn remote_default_branch(&self, remote: &str) -> Option<String> {
        let name = self.text(&["symbolic-ref", "--quiet", "--short", &format!("refs/remotes/{remote}/HEAD")]).ok()?;
        name.strip_prefix(&format!("{remote}/")).map(str::to_string)
    }

    pub fn remote_address(&self, remote: &str) -> Option<String> {
        self.text(&["remote", "get-url", remote]).ok()
    }

    pub fn blobs(&self) -> Result<Blobs, String> {
        let mut child = self
            .command(&["cat-file", "--batch"])
            .env("GIT_NO_LAZY_FETCH", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("start git cat-file: {error}"))?;
        let input = child.stdin.take();
        let output = child.stdout.take().map(BufReader::new);
        Ok(Blobs { child, input, output })
    }

    /// Every file of a tree, or `None` when git no longer has that tree.
    pub fn tree_entries(&self, tree: &str) -> Option<BTreeMap<String, Entry>> {
        let bytes = self.output(&["ls-tree", "-r", "-z", "--full-tree", tree], None).ok()?;
        let mut entries = BTreeMap::new();
        for record in bytes.split(|byte| *byte == 0).filter(|record| !record.is_empty()) {
            let record = String::from_utf8_lossy(record);
            let Some((meta, path)) = record.split_once('\t') else {
                continue;
            };
            let fields: Vec<&str> = meta.split(' ').collect();
            if let [mode, _kind, id] = fields.as_slice() {
                entries.insert(path.to_string(), Entry { mode: mode.to_string(), id: id.to_string() });
            }
        }
        Some(entries)
    }

    /// The paths of the working tree that differ from the current commit: tracked files edited,
    /// staged or not, new files that git does not ignore, and deleted files.
    pub fn working_changes(&self) -> Result<Vec<WorkingChange>, String> {
        let bytes = self.output(&["status", "--porcelain=v2", "-z", "--untracked-files=all", "--no-renames", "--ignored=no"], None)?;
        let mut changes = Vec::new();
        for record in bytes.split(|byte| *byte == 0).filter(|record| !record.is_empty()) {
            let record = String::from_utf8_lossy(record).to_string();
            if let Some(path) = record.strip_prefix("? ") {
                // An untracked folder that is a repository of its own is not content of this one.
                if !path.ends_with('/') {
                    let mode = working_mode(&self.root.join(path));
                    changes.push(WorkingChange::Present { path: path.to_string(), mode });
                }
                continue;
            }
            let (kind, rest) = record.split_at(1);
            let fields: Vec<&str> = rest.trim_start().splitn(match kind { "1" => 8, "u" => 10, _ => 0 }, ' ').collect();
            let (head_mode, worktree_mode, path) = match (kind, fields.as_slice()) {
                ("1", [_xy, _sub, head, _index, worktree, _hh, _hi, path]) => (*head, *worktree, *path),
                ("u", [_xy, _sub, _m1, _m2, _m3, worktree, _h1, _h2, _h3, path]) => ("100644", *worktree, *path),
                _ => continue,
            };
            if worktree_mode == "000000" {
                if head_mode != "000000" {
                    changes.push(WorkingChange::Deleted { path: path.to_string() });
                }
            } else {
                changes.push(WorkingChange::Present { path: path.to_string(), mode: worktree_mode.to_string() });
            }
        }
        Ok(changes)
    }

    /// The object ids git gives the content of working-tree files, in the form git would store,
    /// computed without writing anything into git's object store.
    pub fn hash_files(&self, paths: &[String]) -> Result<Vec<String>, String> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let input: String = paths.iter().map(|path| format!("{path}\n")).collect();
        let bytes = self.output(&["hash-object", "--stdin-paths"], Some(input.as_bytes()))?;
        let ids: Vec<String> = String::from_utf8_lossy(&bytes).lines().map(str::to_string).collect();
        if ids.len() != paths.len() {
            return Err("git hash-object gave fewer ids than it was given paths".to_string());
        }
        Ok(ids)
    }

    /// The object id of a symbolic link: git stores the text of its target.
    pub fn hash_link(&self, path: &str) -> Result<String, String> {
        let target = std::fs::read_link(self.root.join(path)).map_err(|error| format!("read the link {path}: {error}"))?;
        let bytes = self.output(&["hash-object", "--no-filters", "--stdin"], Some(target.to_string_lossy().as_bytes()))?;
        Ok(String::from_utf8_lossy(&bytes).trim().to_string())
    }

    /// Tracked paths at or below a folder of the working tree.
    pub fn tracked_under(&self, folder: &str) -> Result<Vec<String>, String> {
        let bytes = self.output(&["ls-files", "-z", "--", folder], None)?;
        Ok(bytes
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| String::from_utf8_lossy(path).to_string())
            .collect())
    }
}

pub struct Blobs {
    child: Child,
    input: Option<ChildStdin>,
    output: Option<BufReader<ChildStdout>>,
}

impl Blobs {
    pub fn read(&mut self, id: &str) -> Result<Vec<u8>, String> {
        self.read_object(id).map_err(|error| format!("git object {id}: {error}"))
    }

    fn read_object(&mut self, id: &str) -> std::io::Result<Vec<u8>> {
        let invalid = |message| std::io::Error::new(std::io::ErrorKind::InvalidData, message);
        let input = self.input.as_mut().expect("git input is open");
        writeln!(input, "{id}")?;
        input.flush()?;
        let output = self.output.as_mut().expect("git output is open");
        let mut header = String::new();
        output.read_line(&mut header)?;
        let fields: Vec<&str> = header.split_whitespace().collect();
        let size = match fields.as_slice() {
            [found, "blob", size] if *found == id => size.parse::<usize>().map_err(|_| invalid("invalid object size"))?,
            [found, "missing"] if *found == id => return Err(invalid("missing object")),
            _ => return Err(invalid("unexpected object id or type in the batch answer")),
        };
        let mut bytes = vec![0; size];
        output.read_exact(&mut bytes)?;
        let mut separator = [0];
        output.read_exact(&mut separator)?;
        if separator != *b"\n" {
            return Err(invalid("invalid object separator"));
        }
        Ok(bytes)
    }
}

impl Drop for Blobs {
    fn drop(&mut self) {
        self.input.take();
        self.output.take();
        let _ = self.child.wait();
    }
}

/// The mode git would store for a new file of the working tree.
#[cfg(unix)]
fn working_mode(path: &Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => "120000".to_string(),
        Ok(meta) if meta.permissions().mode() & 0o100 != 0 => "100755".to_string(),
        _ => "100644".to_string(),
    }
}

#[cfg(not(unix))]
fn working_mode(path: &Path) -> String {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => "120000".to_string(),
        _ => "100644".to_string(),
    }
}
