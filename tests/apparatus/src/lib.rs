//! The harness for Azimuth's end-to-end tests.
//!
//! A test copies `tests/sample/` to a temporary folder, makes it a git repository with one
//! commit, runs the real `azimuth` binary there and inspects the exit status, the output, the
//! files in the tree and the files of the local store. Nothing in Azimuth is replaced by a stand-in:
//! the only programs besides `azimuth` and git are the text-suite adapter and the facility probe,
//! which are the test runner and the proof command of the sample.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

pub mod process;

pub use serde_json;
pub use serde_json::json;

/// The four Checks of the sample, in the order the model declares them.
pub const SIXTH_LOAN: &str = "sixth-loan-is-refused";
pub const RETURNED_BOOK: &str = "returned-book-frees-a-slot";
pub const LATE_RETURN: &str = "late-return-is-fined";
pub const ON_TIME_RETURN: &str = "on-time-return-is-free";
pub const ALL_CHECKS: [&str; 4] = [SIXTH_LOAN, RETURNED_BOOK, LATE_RETURN, ON_TIME_RETURN];

pub const LOAN_LIMIT: &str = "loan-limit-holds";
pub const LATE_FINE: &str = "late-return-owes-a-fine";

pub const PROJECT_FILE: &str = "azimuth/project.toml";
pub const LENDING_SPEC: &str = "azimuth/model/lending/specification.md";
pub const LENDING_VERIFICATION: &str = "azimuth/model/lending/verification.md";
pub const RETURNS_SPEC: &str = "azimuth/model/desk/returns/specification.md";
pub const RETURNS_VERIFICATION: &str = "azimuth/model/desk/returns/verification.md";
pub const LOAN_LIMIT_TESTS: &str = "suites/desk-checks/loan-limit.test";
pub const LATE_FINE_TESTS: &str = "suites/desk-checks/late-fine.test";
pub const FREE_RETURN_TESTS: &str = "suites/desk-checks/free-return.test";
pub const PRODUCT_FILE: &str = "desk/lending.txt";

/// Where `azimuth skills install` puts the one copy of the text, and the link it adds for `claude`.
pub const SKILL_FILE: &str = ".agents/skills/azimuth-change/SKILL.md";
pub const SKILL_FOLDER: &str = ".agents/skills/azimuth-change";
pub const CLAUDE_SKILL_LINK: &str = ".claude/skills/azimuth-change";

/// The reasons for owed, as the design writes them. A `reason` field holds exactly one of them.
pub const OWED_REASONS: [&str; 11] = [
    "an input changed",
    "never run",
    "deferred by the lane",
    "cannot run here",
    "failed on these inputs",
    "inconclusive",
    "contradicted",
    "the start-up guard is owed",
    "expired",
    "no result",
    "runs in CI",
];

/// The parts of a fingerprint without an input provider.
pub const PARTS: [&str; 4] = ["claim", "files", "environment", "method"];

/// A version that is not the CLI's, for the tests of a pin that differs on purpose. The sample's
/// own pin is the version of the CLI under test (see `Repo::build`).
pub const OTHER_CLI: &str = "0.2.0-alpha.99";

const TIME_LIMIT: Duration = Duration::from_secs(120);

pub fn sample_source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample")
}

pub fn docs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs")
}

/// Folder that holds the binaries of the workspace. Integration tests run from `target/<profile>/deps`.
///
/// `cargo test` builds the binaries of this package for its integration tests, but not the
/// `azimuth` binary of the core, which no test target of the core needs. A test would then run
/// whatever `azimuth` an earlier build left behind. So before the first use in each test binary
/// the harness asks cargo to build every binary the tests start, in the same target folder and
/// profile. Cargo builds only what changed, so a fresh tree costs one check.
pub fn binaries_dir() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(build_binaries).clone()
}

fn build_binaries() -> PathBuf {
    let exe = std::env::current_exe().expect("the test binary knows its path");
    let dir = exe
        .parent()
        .and_then(Path::parent)
        .expect("the test binary sits in target/<profile>/deps")
        .to_path_buf();
    let profile_folder = dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a profile folder")
        .to_string();
    // Cargo calls the folder of the `dev` profile `debug`; every other profile has its own name.
    let profile = if profile_folder == "debug" {
        "dev".to_string()
    } else {
        profile_folder
    };
    let target_dir = dir
        .parent()
        .expect("the profile folder sits in the target folder");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut command = Command::new(cargo);
    command
        .current_dir(&workspace)
        .args([
            "build",
            "--quiet",
            "--profile",
            &profile,
            "--package",
            "azimuth",
            "--bin",
            "azimuth",
        ])
        .args([
            "--package",
            "azimuth-tests",
            "--bin",
            "text-suite-adapter",
            "--bin",
            "facility-probe",
            "--bin",
            "stdin-recorder",
            "--bin",
            "process-tree",
            "--bin",
            "git-recorder",
        ])
        .env("CARGO_TARGET_DIR", target_dir);
    if cfg!(feature = "dotnet") {
        // The same features as the running tests, so cargo does not rebuild this package twice.
        command.args(["--features", "azimuth-tests/dotnet"]);
    }
    let status = command.status().expect("cargo starts");
    assert!(
        status.success(),
        "cargo could not build the binaries the tests start ({status})"
    );
    for name in [
        "azimuth",
        "text-suite-adapter",
        "facility-probe",
        "stdin-recorder",
        "process-tree",
        "git-recorder",
    ] {
        assert!(
            dir.join(name).exists(),
            "{name} is not in {} after the build",
            dir.display()
        );
    }
    dir
}

#[derive(Debug)]
pub struct Output {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    /// The one JSON document on standard output. Fails when standard output holds anything else.
    pub fn json(&self) -> Value {
        let mut documents = serde_json::Deserializer::from_str(&self.stdout).into_iter::<Value>();
        let first = match documents.next() {
            Some(Ok(value)) => value,
            other => panic!(
                "standard output is not one JSON document ({other:?})\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
                self.status, self.stdout, self.stderr
            ),
        };
        assert!(
            documents.next().is_none(),
            "standard output holds more than one JSON document:\n{}",
            self.stdout
        );
        first
    }

    /// Standard output and standard error together, for assertions about what a person reads.
    pub fn text(&self) -> String {
        format!("{}\n{}", self.stdout, self.stderr)
    }

    pub fn expect_status(&self, status: i32) -> &Self {
        assert_eq!(
            self.status, status,
            "unexpected exit status\nstdout:\n{}\nstderr:\n{}",
            self.stdout, self.stderr
        );
        self
    }
}

/// A copy of the sample in a temporary folder, with the places around it that a test needs:
/// a home folder, a git configuration, and a log of every start of the apparatus programs.
pub struct Repo {
    temp: TempDir,
    root: PathBuf,
    extra_env: Vec<(String, String)>,
}

impl Repo {
    /// The sample as a repository with one commit on `main`, in the state `azimuth init` leaves.
    pub fn sample() -> Repo {
        Repo::build("main", |_| {})
    }

    /// The sample with `azimuth/` and `.azimuth/` removed, as a repository that has never met Azimuth.
    pub fn sample_before_init(branch: &str) -> Repo {
        Repo::build(branch, |root| {
            fs::remove_dir_all(root.join("azimuth")).expect("the sample has azimuth/");
            fs::remove_dir_all(root.join(".azimuth")).expect("the harness wrote .azimuth/");
        })
    }

    /// The sample with an edit applied before the one commit is made.
    pub fn sample_with(prepare: impl FnOnce(&Path)) -> Repo {
        Repo::build("main", prepare)
    }

    fn build(branch: &str, prepare: impl FnOnce(&Path)) -> Repo {
        let temp = tempfile::Builder::new()
            .prefix("azimuth-sample-")
            .tempdir()
            .expect("a temporary folder");
        let root = temp.path().join("library-desk");
        copy_git_worktree_files(&sample_source(), &root);
        pin_to_the_cli(&root);
        // `.azimuth/` ignores itself, so the sample cannot carry it in Azimuth's own repository.
        // It is written here as `azimuth init` would have left it.
        fs::create_dir_all(root.join(".azimuth")).unwrap();
        fs::write(root.join(".azimuth/.gitignore"), "*\n").unwrap();
        fs::create_dir_all(temp.path().join("home")).unwrap();
        fs::write(
            temp.path().join("gitconfig"),
            "[user]\n\tname = Sample Author\n\temail = author@library-desk.invalid\n[init]\n\tdefaultBranch = main\n[advice]\n\tdetachedHead = false\n[protocol \"file\"]\n\tallow = always\n",
        )
        .unwrap();
        let repo = Repo { root, temp, extra_env: Vec::new() };
        prepare(&repo.root);
        repo.git(&["init", "--quiet", "-b", branch]);
        repo.commit_all("sample");
        repo
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// A path in the temporary folder beside the repository, for clones and worktrees.
    pub fn beside(&self, name: &str) -> PathBuf {
        self.temp.path().join(name)
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn log_path(&self) -> PathBuf {
        self.temp.path().join("apparatus.log")
    }

    /// Adds an environment variable to every later start of `azimuth`.
    pub fn set_env(&mut self, name: &str, value: &str) {
        self.extra_env.retain(|(key, _)| key != name);
        self.extra_env.push((name.to_string(), value.to_string()));
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).unwrap_or_else(|e| panic!("read {relative}: {e}"))
    }

    pub fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap_or_else(|e| panic!("write {relative}: {e}"));
    }

    pub fn append(&self, relative: &str, text: &str) {
        let mut current = self.read(relative);
        current.push_str(text);
        self.write(relative, &current);
    }

    /// Replaces text in a file and fails when the text is not there, so a test never edits nothing.
    pub fn replace(&self, relative: &str, from: &str, to: &str) {
        let current = self.read(relative);
        assert!(current.contains(from), "{relative} does not contain {from:?}:\n{current}");
        self.write(relative, &current.replacen(from, to, 1));
    }

    pub fn remove(&self, relative: &str) {
        let path = self.path(relative);
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        } else {
            fs::remove_file(path).unwrap_or_else(|e| panic!("remove {relative}: {e}"));
        }
    }

    /// Runs git in the repository and returns its standard output. Fails when git fails.
    pub fn git(&self, args: &[&str]) -> String {
        self.git_in(&self.root, args)
    }

    pub fn git_in(&self, dir: &Path, args: &[&str]) -> String {
        let output = self.git_output_in(dir, args);
        assert_eq!(output.status, 0, "git {args:?} failed:\n{}\n{}", output.stdout, output.stderr);
        output.stdout
    }

    pub fn git_output_in(&self, dir: &Path, args: &[&str]) -> Output {
        let mut command = Command::new("git");
        command.args(args).current_dir(dir);
        self.base_env(&mut command);
        run_with_limit(command)
    }

    pub fn commit_all(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "--quiet", "--allow-empty", "-m", message]);
    }

    pub fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"]).trim().to_string()
    }

    /// Starts the real `azimuth` binary in the repository.
    pub fn azimuth(&self, args: &[&str]) -> Output {
        self.azimuth_in(&self.root, args, &[])
    }

    pub fn azimuth_with_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        self.azimuth_in(&self.root, args, env)
    }

    pub fn azimuth_in(&self, dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut command = Command::new(binaries_dir().join("azimuth"));
        command.args(args).current_dir(dir);
        self.base_env(&mut command);
        for (name, value) in env {
            command.env(name, value);
        }
        run_with_limit(command)
    }

    pub fn azimuth_command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(binaries_dir().join("azimuth"));
        command.args(args).current_dir(&self.root);
        self.base_env(&mut command);
        command
    }

    /// The environment of every command a test starts. It is cleared first, so that nothing on
    /// the machine running the tests, such as a CI variable, reaches the command by accident.
    fn base_env(&self, command: &mut Command) {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs = vec![binaries_dir()];
        dirs.extend(std::env::split_paths(&path));
        command
            .env_clear()
            .env("PATH", std::env::join_paths(dirs).unwrap())
            .env("HOME", self.temp.path().join("home"))
            .env("TMPDIR", std::env::temp_dir())
            .env("GIT_CONFIG_GLOBAL", self.temp.path().join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("APPARATUS_LOG", self.log_path());
        for (name, value) in &self.extra_env {
            command.env(name, value);
        }
    }

    /// Every start of the text-suite adapter and the facility probe, in order.
    pub fn log_entries(&self) -> Vec<Value> {
        let Ok(text) = fs::read_to_string(self.log_path()) else {
            return Vec::new();
        };
        text.lines().map(|line| serde_json::from_str(line).expect("a log line is JSON")).collect()
    }

    pub fn clear_log(&self) {
        let _ = fs::remove_file(self.log_path());
    }

    /// The starts of the text-suite adapter with the verb `run`.
    pub fn adapter_runs(&self) -> Vec<Value> {
        self.log_entries()
            .into_iter()
            .filter(|entry| entry["program"] == "text-suite-adapter" && entry["request"]["verb"] == "run")
            .collect()
    }

    pub fn adapter_starts(&self) -> Vec<Value> {
        let entries = self.log_entries();
        entries
            .iter()
            .filter(|e| e["program"] == "text-suite-adapter" && e["event"] == "started")
            .map(|start| {
                let mut entry = start.clone();
                if let Some(end) = entries.iter().rev().find(|e| {
                    e["program"] == "text-suite-adapter"
                        && e["pid"] == start["pid"]
                        && e.get("event").is_none()
                }) {
                    entry
                        .as_object_mut()
                        .unwrap()
                        .extend(end.as_object().unwrap().clone());
                    entry["event"] = json!("started");
                    entry["time_ns"] = start["time_ns"].clone();
                }
                entry
            })
            .collect()
    }

    pub fn probe_starts(&self) -> Vec<Value> {
        self.log_entries()
            .into_iter()
            .filter(|entry| entry["program"] == "facility-probe")
            .collect()
    }

    /// The Check ids the core asked the adapter for, over every launch since the log was cleared.
    pub fn requested_checks(&self) -> BTreeSet<String> {
        self.adapter_runs()
            .iter()
            .flat_map(|entry| strings_of(&entry["request"]["checks"]))
            .collect()
    }

    /// The tests the adapter ran, over every launch since the log was cleared.
    pub fn tests_ran(&self) -> BTreeSet<String> {
        self.adapter_runs().iter().flat_map(|entry| strings_of(&entry["ran"])).collect()
    }

    pub fn git_common_dir(&self) -> PathBuf {
        let dir = self.git(&["rev-parse", "--git-common-dir"]).trim().to_string();
        let dir = PathBuf::from(dir);
        if dir.is_absolute() {
            dir
        } else {
            self.root.join(dir)
        }
    }

    /// The local store: `azimuth/` in the clone's shared git directory.
    pub fn store(&self) -> PathBuf {
        self.git_common_dir().join("azimuth")
    }

    /// The root of the outbox. The design fixes only the two roots, the outbox in the worktree and
    /// the store in the git directory, so nothing below either is assumed.
    pub fn outbox(&self) -> PathBuf {
        self.path(".azimuth")
    }

    /// The record files of the store: every `.json` file at any depth below it.
    pub fn store_files(&self) -> Vec<PathBuf> {
        json_files_under(&self.store())
    }

    /// The record files of the outbox: every `.json` file at any depth below it.
    pub fn outbox_files(&self) -> Vec<PathBuf> {
        json_files_under(&self.outbox())
    }

    /// Every record in the store and the outbox, parsed. A file that is not JSON is left out.
    pub fn records(&self) -> Vec<(PathBuf, Value)> {
        let mut records = Vec::new();
        for file in self.store_files().into_iter().chain(self.outbox_files()) {
            if let Ok(value) = serde_json::from_slice::<Value>(&fs::read(&file).unwrap_or_default()) {
                records.push((file, value));
            }
        }
        records
    }

    /// The `result-1` records of one Check: the ones whose one subject names it.
    pub fn results_of(&self, check: &str) -> Vec<(PathBuf, Value)> {
        self.records()
            .into_iter()
            .filter(|(_, value)| value["format"] == "result-1" && value["subjects"][0]["name"] == check)
            .collect()
    }

    /// The one `result-1` record of a Check. Fails when there is none or more than one.
    pub fn result_of(&self, check: &str) -> Value {
        let results = self.results_of(check);
        assert_eq!(results.len(), 1, "{check} has {} results: {results:#?}", results.len());
        results[0].1.clone()
    }

    /// Rewrites every record in the store and the outbox, as a person or a later CLI could. The new
    /// record is written in the canonical form under the SHA-256 of its bytes, as `record-1` names
    /// a record, so it is a well-formed record and only its content differs.
    pub fn rewrite_records(&self, rewrite: impl Fn(&mut Value)) {
        for file in self.store_files().into_iter().chain(self.outbox_files()) {
            let mut value: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
            let before = value.clone();
            rewrite(&mut value);
            if value == before {
                continue;
            }
            fs::remove_file(&file).unwrap();
            write_record(file.parent().unwrap(), &value);
        }
    }

    /// Every file of the working tree outside `.git`, ignored ones included, with its bytes.
    pub fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        snapshot_dir(&self.root)
    }

    pub fn untracked(&self) -> Vec<String> {
        self.git(&["ls-files", "--others", "--exclude-standard"])
            .lines()
            .map(str::to_string)
            .collect()
    }
}

pub fn run_with_limit(command: Command) -> Output {
    run_with_time_limit(command, TIME_LIMIT, None)
}

/// Runs a command with an optional standard input and a time limit, and kills it at the limit.
pub fn run_with_time_limit(mut command: Command, limit: Duration, input: Option<&str>) -> Output {
    command
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap_or_else(|e| panic!("start {command:?}: {e}"));
    if let Some(input) = input {
        use std::io::Write;
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(input.as_bytes()).unwrap();
    }
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        text
    });
    let err = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > limit {
            let _ = child.kill();
            panic!("{command:?} did not end within {limit:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    Output {
        // A command killed by a signal has no code. It counts as none of the three statuses.
        status: status.code().unwrap_or(-1),
        stdout: out.join().unwrap(),
        stderr: err.join().unwrap(),
    }
}

fn copy_git_worktree_files(from: &Path, to: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let listing = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            ".",
        ])
        .current_dir(from)
        .output()
        .unwrap();
    assert!(
        listing.status.success(),
        "git could not list {}: {}",
        from.display(),
        String::from_utf8_lossy(&listing.stderr)
    );
    fs::create_dir_all(to).unwrap();
    for name in listing
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let relative = Path::new(std::ffi::OsStr::from_bytes(name));
        let source = from.join(relative);
        let destination = to.join(relative);
        let metadata = match fs::symlink_metadata(&source) {
            Ok(metadata) => metadata,
            // An uncommitted sample deletion must survive copying.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("read {}: {error}", source.display()),
        };
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        if metadata.file_type().is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(&source).unwrap(), destination).unwrap();
        } else {
            fs::copy(source, destination).unwrap();
        }
    }
}

pub fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("read {}: {e}", from.display())) {
        let entry = entry.unwrap();
        let name = entry.file_name();
        // Intermediate output is never source; a `bin` folder can hold a package's commands.
        if name == "obj" || name == ".DS_Store" {
            continue;
        }
        let target = to.join(&name);
        if entry.file_type().unwrap().is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(entry.path()).unwrap(), &target).unwrap();
        } else if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

pub fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn snapshot_dir(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut map = BTreeMap::new();
    for file in files_under(root) {
        let relative = file.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
        if relative.starts_with(".git/") {
            continue;
        }
        map.insert(relative, fs::read(&file).unwrap());
    }
    map
}

/// Every string value inside a JSON value, at any depth.
pub fn all_strings(value: &Value) -> Vec<String> {
    let mut found = Vec::new();
    collect_strings(value, &mut found);
    found
}

fn collect_strings(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::String(text) => found.push(text.clone()),
        Value::Array(items) => items.iter().for_each(|item| collect_strings(item, found)),
        Value::Object(map) => map.values().for_each(|item| collect_strings(item, found)),
        _ => {}
    }
}

/// Every key inside a JSON value, at any depth.
pub fn all_keys(value: &Value) -> Vec<String> {
    let mut found = Vec::new();
    fn walk(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::Array(items) => items.iter().for_each(|item| walk(item, found)),
            Value::Object(map) => {
                for (key, item) in map {
                    found.push(key.clone());
                    walk(item, found);
                }
            }
            _ => {}
        }
    }
    walk(value, &mut found);
    found
}

pub fn strings_of(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .unwrap_or_else(|| panic!("list contains a non-string element: {item:#}"))
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The entries of a document's `checks` list, by id.
pub fn checks_by_id(document: &Value) -> BTreeMap<String, Value> {
    document["checks"]
        .as_array()
        .unwrap_or_else(|| panic!("the document has no `checks` list:\n{document:#}"))
        .iter()
        .map(|check| (check["id"].as_str().expect("a Check has an id").to_string(), check.clone()))
        .collect()
}

pub fn state_of(document: &Value, check: &str) -> String {
    let checks = checks_by_id(document);
    let entry = checks
        .get(check)
        .unwrap_or_else(|| panic!("the document does not list Check {check}:\n{document:#}"));
    entry["state"].as_str().unwrap_or_default().to_string()
}

pub fn reason_of(document: &Value, check: &str) -> String {
    checks_by_id(document)[check]["reason"].as_str().unwrap_or_default().to_string()
}

/// The Checks of a `status-1` or `requirement-1` document, by id, from inside its Requirements.
pub fn status_checks(document: &Value) -> BTreeMap<String, Value> {
    let mut checks = BTreeMap::new();
    let requirements = match document.get("requirements") {
        Some(Value::Array(items)) => items.clone(),
        _ => vec![document.get("requirement").cloned().unwrap_or_else(|| document.clone())],
    };
    for requirement in requirements {
        for scenario in requirement["scenarios"].as_array().cloned().unwrap_or_default() {
            for check in scenario["checks"].as_array().cloned().unwrap_or_default() {
                checks.insert(check["id"].as_str().unwrap_or_default().to_string(), check);
            }
        }
    }
    checks
}

pub fn requirement_in<'a>(status: &'a Value, id: &str) -> Option<&'a Value> {
    status["requirements"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["id"] == id))
}

/// Counts a field that the design names but whose type it does not fix: a number is its own
/// count, a list counts its entries, `true` counts one.
pub fn count(value: &Value) -> u64 {
    match value {
        Value::Number(number) => number.as_u64().unwrap_or(0),
        Value::Array(items) => items.len() as u64,
        Value::Bool(true) => 1,
        Value::String(text) if !text.is_empty() => 1,
        _ => 0,
    }
}

pub fn json_files_under(dir: &Path) -> Vec<PathBuf> {
    files_under(dir)
        .into_iter()
        .filter(|file| file.extension().is_some_and(|ext| ext == "json"))
        .collect()
}

/// SHA-256 of the bytes, as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Model text as `text-1` normalizes it before hashing: NFC, every line ending as LF, spaces and
/// tabs at line ends removed, and nothing else folded.
pub fn text_1_normalize(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let composed: String = text.nfc().collect();
    let unified = composed.replace("\r\n", "\n").replace('\r', "\n");
    unified
        .split('\n')
        .map(|line| line.trim_end_matches([' ', '\t']))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The `text-1` digest of model text: the scheme name, a colon and the SHA-256 of the normalized
/// text in lowercase hex.
pub fn text_1_digest(text: &str) -> String {
    format!("text-1:{}", sha256_hex(text_1_normalize(text).as_bytes()))
}

/// The digest line `azimuth skills install` writes as the last line of a file, for the given
/// text of the file without that line.
pub fn skill_digest_line(text: &str) -> String {
    format!("<!-- azimuth {} -->", text_1_digest(text))
}

/// Splits an installed file into its text without the last line, and the last line. The text
/// keeps the line ending of the line before the digest line, because that ending is part of it.
pub fn split_last_line(file: &str) -> (String, String) {
    let trimmed = file.strip_suffix('\n').unwrap_or(file);
    match trimmed.rfind('\n') {
        Some(index) => (file[..index + 1].to_string(), trimmed[index + 1..].to_string()),
        None => (String::new(), trimmed.to_string()),
    }
}

/// The canonical form of RFC 8785 for a JSON value: no whitespace, object members sorted by the
/// UTF-16 code units of their names, strings escaped as ECMAScript does. A record holds only
/// whole numbers, so a fractional number is refused rather than formatted.
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => {
            if let Some(whole) = number.as_i64() {
                out.push_str(&whole.to_string());
            } else if let Some(whole) = number.as_u64() {
                out.push_str(&whole.to_string());
            } else {
                panic!("a record holds the fractional number {number}");
            }
        }
        Value::String(text) => write_canonical_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical_string(key, out);
                out.push(':');
                write_canonical(&map[*key], out);
            }
            out.push('}');
        }
    }
}

fn write_canonical_string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Writes a record as `record-1` names it: its canonical bytes, in a file named by their SHA-256.
pub fn write_record(dir: &Path, value: &Value) -> PathBuf {
    let bytes = canonical_json(value);
    fs::create_dir_all(dir).unwrap();
    let path = dir.join(format!("{}.json", sha256_hex(bytes.as_bytes())));
    fs::write(&path, bytes).unwrap();
    path
}

/// The lists of behaviors the tests are written from, under `docs/increments/`.
pub const BEHAVIOR_LISTS: [&str; 4] = [
    "increments/1.md",
    "increments/2.md",
    "increments/2-follow-up.md",
    "increments/release.md",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    Behavior,
    Prose,
}

/// One row of a behavior table in a list, including rows of prose.
#[derive(Debug, Clone)]
pub struct BehaviorRow {
    pub list: String,
    pub section: String,
    pub behavior: String,
    pub quote: String,
    pub kind: RowKind,
    /// False for prose and in the sections whose rows have no test in this repository:
    /// "Left for increment 4" of list 1, and "Proven by the first tag" of the release list.
    pub needs_test: bool,
}

/// The rows of every behavior table of every list. A table counts when its header starts with
/// `Behavior`; other tables, such as the open questions, hold no behaviors.
pub fn behavior_rows() -> Vec<BehaviorRow> {
    let mut rows = Vec::new();
    for list in BEHAVIOR_LISTS {
        let text = fs::read_to_string(docs_dir().join(list)).unwrap_or_else(|_| panic!("docs/{list}"));
        rows.extend(behavior_rows_in(list, &text));
    }
    rows
}

fn behavior_rows_in(list: &str, text: &str) -> Vec<BehaviorRow> {
    let mut rows = Vec::new();
    let mut section = String::new();
    let mut kind = RowKind::Behavior;
    let mut first_paragraph = false;
    let mut in_behavior_table = false;
    let mut previous_was_table = false;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            section = heading.trim().to_string();
            kind = RowKind::Behavior;
            first_paragraph = true;
        } else if first_paragraph && !line.trim().is_empty() {
            if line.trim_start().starts_with("Rows of prose:") {
                kind = RowKind::Prose;
            }
            first_paragraph = false;
        }
        let is_table = line.starts_with('|');
        if is_table && !previous_was_table {
            in_behavior_table = line.starts_with("| Behavior |");
            previous_was_table = true;
            continue;
        }
        previous_was_table = is_table;
        if !is_table || !in_behavior_table || line.starts_with("| ---") {
            continue;
        }
        let body = &line[2..];
        let split = body.find(" | ").unwrap_or_else(|| panic!("a row has two cells: {line}"));
        let behavior = body[..split].trim().to_string();
        let quote = body[split + 3..].trim_end().trim_end_matches('|').trim_end().to_string();
        let needs_test = kind == RowKind::Behavior
            && !(section.starts_with("Left for increment 4") || section.starts_with("Proven by the first tag"));
        rows.push(BehaviorRow { list: list.to_string(), section: section.clone(), behavior, quote, kind, needs_test });
    }
    rows
}

impl Repo {
    /// The 1-based number of the first line of a file that contains the text.
    pub fn line_of(&self, relative: &str, needle: &str) -> u64 {
        let text = self.read(relative);
        let index = text
            .lines()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("{relative} has no line with {needle:?}"));
        index as u64 + 1
    }

    /// Asserts that `azimuth status --json` stops on a reading error and returns the `error-1`
    /// document. When a file is given, one error must name it as a path from the repository root,
    /// and the line, counted from 1, when one is given.
    pub fn expect_reading_error(&self, file: Option<&str>, line: Option<u64>) -> Value {
        let output = self.azimuth(&["status", "--json"]);
        output.expect_status(2);
        let document = output.json();
        assert_eq!(document["format"], "error-1", "{document:#}");
        let errors = document["errors"].as_array().cloned().unwrap_or_default();
        assert!(!errors.is_empty(), "error-1 lists no error: {document:#}");
        for error in &errors {
            entry_has_kind_file_line_message(error);
        }
        if let Some(file) = file {
            let named: Vec<&Value> = errors.iter().filter(|error| error["file"] == file).collect();
            assert!(!named.is_empty(), "no error names {file}: {document:#}");
            if let Some(line) = line {
                assert!(
                    named.iter().any(|error| error["line"].as_u64() == Some(line)),
                    "no error names line {line} of {file}: {document:#}"
                );
            }
        }
        document
    }
}

/// A field is there when it is present with a value. A `null` never counts as a value, because
/// the design writes a field with nothing to say by leaving it out.
pub fn has(value: &Value, field: &str) -> bool {
    value.get(field).is_some_and(|inner| !inner.is_null())
}

/// An entry of `lint-1` or `error-1`: `kind`, `file` and `message` are always there. `line` is a
/// line number counted from 1, and is absent for a finding that belongs to no one line.
pub fn entry_has_kind_file_line_message(entry: &Value) {
    for field in ["kind", "file", "message"] {
        assert!(has(entry, field), "no `{field}`: {entry:#}");
    }
    if let Some(line) = entry.get("line") {
        assert!(line.as_u64().is_some_and(|number| number >= 1), "`line` is not a line number: {entry:#}");
    }
}

/// Asserts that no `null` appears at any depth of a document or a record.
pub fn assert_no_null(value: &Value, origin: &str) {
    fn walk(value: &Value, path: &mut Vec<String>, origin: &str, whole: &Value) {
        match value {
            Value::Null => panic!("{origin} holds null at {}:\n{whole:#}", if path.is_empty() { "the top".to_string() } else { path.join(".") }),
            Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    path.push(index.to_string());
                    walk(item, path, origin, whole);
                    path.pop();
                }
            }
            Value::Object(map) => {
                for (key, item) in map {
                    path.push(key.clone());
                    walk(item, path, origin, whole);
                    path.pop();
                }
            }
            _ => {}
        }
    }
    walk(value, &mut Vec::new(), origin, value);
}

/// The `parts_changed` list of a `check-1` document: part names, in the order given.
pub fn parts_changed(check: &Value) -> Vec<String> {
    let list = check["parts_changed"]
        .as_array()
        .unwrap_or_else(|| panic!("`parts_changed` is not a list: {check:#}"));
    list.iter()
        .map(|part| part.as_str().unwrap_or_else(|| panic!("a part name is not text: {check:#}")).to_string())
        .collect()
}

/// The `parts_changed` list of a `check-1` document as a set, for comparing with an exact set.
pub fn parts_changed_set(check: &Value) -> BTreeSet<String> {
    parts_changed(check).into_iter().collect()
}

pub fn set_of(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|item| item.to_string()).collect()
}

/// The PATH of the machine with one program hidden, so a test can stand on a machine where that
/// program, and every adapter it would start, is absent. A folder of PATH that holds the program
/// is replaced by a folder in `under` with a link to each of its other entries, so `git`, `sh`
/// and the rest stay reachable by name. Dropping the whole folder would also hide them where the
/// program sits in a shared folder, as `/usr/bin/dotnet` does on a Linux runner. Nothing outside
/// `under` is written.
pub fn path_hiding(program: &str, path: &std::ffi::OsStr, under: &Path) -> String {
    let mut dirs = Vec::new();
    for (index, dir) in std::env::split_paths(path).enumerate() {
        // A broken link named like the program counts as the program too.
        if fs::symlink_metadata(dir.join(program)).is_err() {
            dirs.push(dir);
            continue;
        }
        let stand_in = under.join(format!("{index}"));
        fs::create_dir_all(&stand_in).unwrap();
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            if entry.file_name() != program {
                std::os::unix::fs::symlink(entry.path(), stand_in.join(entry.file_name())).unwrap();
            }
        }
        dirs.push(stand_in);
    }
    std::env::join_paths(dirs).unwrap().to_string_lossy().to_string()
}

impl Repo {
    /// Init asks every known host, so a test of absent adapters must hide both hosts even on a
    /// machine that happens to have Node installed.
    pub fn path_without_programs(&self, programs: &[&str]) -> String {
        let under = self.beside(&format!("path-without-{}", programs.join("-")));
        let _ = fs::remove_dir_all(&under);
        let mut dirs = vec![binaries_dir()];
        dirs.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let mut path = std::env::join_paths(dirs).unwrap();
        for (index, program) in programs.iter().enumerate() {
            path = path_hiding(program, &path, &under.join(index.to_string())).into();
        }
        path.to_string_lossy().to_string()
    }

    /// The machine's PATH, with the apparatus first and one program hidden; the stand-in
    /// folders live in this repository's temporary folder.
    pub fn path_without(&self, program: &str) -> String {
        self.path_without_programs(&[program])
    }
}

/// Writes the version of the CLI under test into the pin of a copy of the sample, so raising the
/// version touches only the code. The literal in `tests/sample/` is overwritten in every copy.
pub fn pin_to_the_cli(root: &Path) {
    let project = root.join("azimuth/project.toml");
    let text = fs::read_to_string(&project).unwrap();
    assert!(text.lines().any(|line| line.starts_with("cli = ")), "the sample has no pin");
    let pinned: String = text
        .lines()
        .map(|line| if line.starts_with("cli = ") { format!("cli = \"{}\"\n", cli_version()) } else { format!("{line}\n") })
        .collect();
    fs::write(&project, pinned).unwrap();
}

/// The version the CLI under test prints for `--version`.
pub fn cli_version() -> String {
    static VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    VERSION
        .get_or_init(|| {
            let output = Command::new(binaries_dir().join("azimuth")).arg("--version").output().unwrap();
            assert!(output.status.success(), "azimuth --version failed");
            String::from_utf8(output.stdout).unwrap().trim().to_string()
        })
        .clone()
}

/// The install line of the .NET adapter up to its version, as the design writes it.
pub const XUNIT_INSTALL: &str =
    "dotnet tool install --local --create-manifest-if-needed Azimuth.Xunit";

/// Each install line of the .NET adapter in a printed text, as a person would copy it. The words
/// around it are free text: the line ends at the end of its printed line, at `)`, at `;`, or at a
/// `.` that ends a sentence, so `… --version 1.2.0-alpha.3.` gives the line without the last
/// dot. A `.` inside a version is followed by no space and stays.
pub fn xunit_install_lines(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let rest = &line[line.find(XUNIT_INSTALL)?..];
            let end = [rest.find(')'), rest.find(';'), rest.find(". ")].into_iter().flatten().min().unwrap_or(rest.len());
            Some(rest[..end].trim_end().trim_end_matches('.').to_string())
        })
        .collect()
}

pub fn parse_toml(text: &str) -> toml::Table {
    text.parse::<toml::Table>().unwrap_or_else(|e| panic!("not TOML ({e}):\n{text}"))
}

impl Repo {
    /// Runs `azimuth lint --json`, asserts its status and format, and returns its findings.
    pub fn lint_findings(&self, status: i32) -> Vec<Value> {
        let output = self.azimuth(&["lint", "--json"]);
        output.expect_status(status);
        let document = output.json();
        assert_eq!(document["format"], "lint-1", "{document:#}");
        let findings = document["findings"].as_array().cloned().unwrap_or_else(|| panic!("no findings list: {document:#}"));
        for finding in &findings {
            entry_has_kind_file_line_message(finding);
        }
        findings
    }

    /// Declares a second suite of the text adapter in `suites/<name>/`.
    pub fn add_text_suite(&self, name: &str, needs: &str) {
        self.append(
            PROJECT_FILE,
            &format!(
                "\n[[suite]]\nname = \"{name}\"\narea = \"all\"\npath = \"suites/{name}\"\nadapter = \"text-suite-adapter\"\nmarker = \"# Check: <id>\"\nneeds = [{needs}]\n"
            ),
        );
    }

    /// Makes the sample's suite need a facility whose proof is the facility probe in a given mode.
    pub fn need_facility(&self, mode: &str) {
        self.replace(PROJECT_FILE, "needs = []", "needs = [\"shelf-scanner\"]");
        self.append(
            PROJECT_FILE,
            &format!("\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe {mode}\"\n"),
        );
    }
}

/// Findings that name a file, and a line when one is given.
pub fn findings_at<'a>(findings: &'a [Value], file: &str, line: Option<u64>) -> Vec<&'a Value> {
    findings
        .iter()
        .filter(|finding| finding["file"].as_str().is_some_and(|path| path.ends_with(file)))
        .filter(|finding| line.is_none() || finding["line"].as_u64() == line)
        .collect()
}

impl Repo {
    /// `azimuth run --json` that must end with the given status; returns the `run-1` document.
    pub fn run_json(&self, args: &[&str], status: i32) -> Value {
        let mut all = vec!["run", "--json"];
        all.extend_from_slice(args);
        let output = self.azimuth(&all);
        output.expect_status(status);
        let document = output.json();
        assert_eq!(document["format"], "run-1", "{document:#}");
        document
    }

    /// `azimuth check show <id> --json`, which must answer; returns the `check-1` document.
    pub fn check_show(&self, check: &str) -> Value {
        let output = self.azimuth(&["check", "show", check, "--json"]);
        output.expect_status(0);
        let document = output.json();
        assert_eq!(document["format"], "check-1", "{document:#}");
        document
    }

    /// `azimuth status --json`, which must answer; returns the `status-1` document.
    pub fn status_json(&self) -> Value {
        let output = self.azimuth(&["status", "--json"]);
        output.expect_status(0);
        let document = output.json();
        assert_eq!(document["format"], "status-1", "{document:#}");
        document
    }

    /// Every string in every record, so a test can look for a bare hash wherever it stands.
    pub fn record_strings(&self) -> Vec<String> {
        self.records().iter().flat_map(|(_, value)| all_strings(value)).collect()
    }

    pub fn tracked_paths(&self) -> Vec<String> {
        self.git(&["ls-files"]).lines().map(str::to_string).collect()
    }
}

/// Strings of the form `<word>-<number>:<hex>`, the written form of a digest.
pub fn is_digest(text: &str) -> bool {
    let Some((scheme, value)) = text.split_once(':') else {
        return false;
    };
    let Some((word, number)) = scheme.rsplit_once('-') else {
        return false;
    };
    !word.is_empty()
        && word.chars().all(|c| c.is_ascii_lowercase())
        && !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit())
        && value.len() >= 7
        && value.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn is_bare_hash(text: &str) -> bool {
    (text.len() == 40 || text.len() == 64) && text.chars().all(|c| c.is_ascii_hexdigit())
}

/// The `files_changed` list of a `check-1` document, sorted: paths from the repository root.
pub fn files_changed(check: &Value) -> Vec<String> {
    let mut files: Vec<String> = check["files_changed"]
        .as_array()
        .unwrap_or_else(|| panic!("`files_changed` is not a list: {check:#}"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("changed path is not text: {entry:#}"))
                .to_string()
        })
        .collect();
    files.sort();
    files
}

impl Repo {
    /// Deletes every result of a Check, as a person may delete anything in the store.
    pub fn forget(&self, check: &str) {
        let records = self.results_of(check);
        assert!(!records.is_empty(), "no record names {check}");
        for (file, _) in records {
            fs::remove_file(file).unwrap();
        }
    }

    /// Moves two text tests into a second suite, so a test can break one suite and watch the other.
    pub fn split_returns_suite(&self, needs: &str) {
        self.add_text_suite("returns-checks", needs);
        for name in ["late-fine.test", "free-return.test"] {
            fs::create_dir_all(self.path("suites/returns-checks")).unwrap();
            fs::rename(self.path(&format!("suites/desk-checks/{name}")), self.path(&format!("suites/returns-checks/{name}"))).unwrap();
        }
    }
}

pub fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

/// Makes a folder one the session may not write, with mode 0o555, and proves that it holds:
/// root, or a process with a capability that overrides file modes, writes the folder anyway.
/// There is no way without a mount or root's own tools to refuse such a process the same
/// permission error, so a test that needs the folder unwritable fails here with that reason
/// instead of failing later in a way that points at the product.
pub fn make_unwritable(dir: &Path) {
    set_mode(dir, 0o555);
    let probe = dir.join(".apparatus-write-probe");
    if fs::write(&probe, b"").is_ok() {
        let _ = fs::remove_file(&probe);
        set_mode(dir, 0o755);
        panic!(
            "this test cannot run as root: it makes {} unwritable with mode 0o555, and this user writes it anyway, so what the product does when it may not write there cannot be observed",
            dir.display()
        );
    }
}

/// Gives a record the next number of its format, as a later CLI version would write it.
pub fn next_format(record: &mut Value) {
    assert_eq!(record["format"], "result-1", "{record:#}");
    record["format"] = Value::String("result-2".to_string());
}

/// The `detail` of a Check entry as text, or an empty text when it has none.
pub fn detail_of_entry(check: &Value) -> String {
    check["detail"].as_str().unwrap_or_default().to_string()
}

pub fn assert_owed_file_count(text: &str, id: &str, expected: usize) {
    let lines: Vec<_> = text
        .lines()
        .filter(|line| {
            line.split_whitespace().any(|word| {
                word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-') == id
            }) && line
                .split(|c: char| !c.is_ascii_alphanumeric())
                .any(|word| word == "owed")
        })
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "{id} has no single owed Check line:\n{text}"
    );
    let words: Vec<_> = lines[0]
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let counts: Vec<usize> = words
        .iter()
        .enumerate()
        .filter(|(_, word)| **word == "file" || **word == "files")
        .flat_map(|(index, _)| {
            words[index.saturating_sub(2)..=(index + 2).min(words.len() - 1)]
                .iter()
                .filter_map(|word| word.parse().ok())
        })
        .collect();
    assert_eq!(
        counts,
        vec![expected],
        "{id} does not associate its file count with files: {}",
        lines[0]
    );
}
pub fn assert_no_file_names(text: &str, files: &[&str]) {
    for file in files {
        let base = std::path::Path::new(file)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            !text.contains(file) && !text.contains(base),
            "wide view names {file}: {text}"
        );
    }
}

#[cfg(test)]
mod apparatus_checks {
    use super::*;

    #[test]
    fn the_list_reader_distinguishes_prose_from_behavior() {
        let rows = behavior_rows_in("list.md", "\
## The text

Rows of prose: a person reads the text.

| Behavior | Quote from the design |
| --- | --- |
| The text explains the rule | The rule is explained. |

## The command

A test runs the command.

Rows of prose: this later paragraph does not mark the section.

| Behavior | Quote from the design |
| --- | --- |
| The command prints the result | The result is printed. |
");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].kind, RowKind::Prose);
        assert!(!rows[0].needs_test);
        assert_eq!(rows[0].behavior, "The text explains the rule");
        assert_eq!(rows[0].quote, "The rule is explained.");
        assert_eq!(rows[1].kind, RowKind::Behavior);
        assert!(rows[1].needs_test);
        assert_eq!(rows[1].behavior, "The command prints the result");
    }

    #[test]
    fn a_sample_copy_keeps_git_files_and_omits_ignored_output() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("source");
        let sample = root.join("sample");
        fs::create_dir_all(&sample).unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init", "--quiet"]);
        git(&["config", "core.excludesFile", "/dev/null"]);
        let write = |path: &str, text: &str| {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        write(".gitignore", "bin/\nobj/\n*.local\n.DS_Store\n");
        write("sample/tracked.txt", "tracked source");
        write("sample/commands/bin/tool", "tracked command");
        git(&[
            "add",
            "--force",
            "--",
            "sample/tracked.txt",
            "sample/commands/bin/tool",
        ]);
        write("sample/.gitignore", "nested-cache/\n");
        write("sample/draft.txt", "untracked source");
        write("sample/name with space.txt", "untracked with spaces");
        std::os::unix::fs::symlink("commands/bin/tool", sample.join("shortcut")).unwrap();
        for path in [
            "sample/project/bin/build.dll",
            "sample/project/obj/build.cache",
            "sample/config.local",
            "sample/.DS_Store",
            "sample/nested-cache/data",
        ] {
            write(path, "ignored output");
        }
        let destination = temp.path().join("copy");
        copy_git_worktree_files(&sample, &destination);
        let paths: Vec<_> = files_under(&destination)
            .into_iter()
            .map(|path| {
                path.strip_prefix(&destination)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            paths,
            vec![
                ".gitignore",
                "commands/bin/tool",
                "draft.txt",
                "name with space.txt",
                "shortcut",
                "tracked.txt"
            ]
        );
        assert_eq!(
            fs::read_to_string(destination.join("commands/bin/tool")).unwrap(),
            "tracked command"
        );
        assert_eq!(
            fs::read_to_string(destination.join("draft.txt")).unwrap(),
            "untracked source"
        );
        assert_eq!(
            fs::read_link(destination.join("shortcut")).unwrap(),
            Path::new("commands/bin/tool")
        );
    }

    // A folder that holds the hidden program and another tool, as `/usr/bin` holds `dotnet` and
    // `git` on a Linux runner: the tool stays reachable by name, the program does not, and the
    // folders of PATH are left as they were.
    #[test]
    fn path_hiding_hides_one_program_and_keeps_its_neighbours() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let shared = temp.path().join("shared");
        let own = temp.path().join("own");
        for dir in [&shared, &own] {
            fs::create_dir_all(dir).unwrap();
        }
        let tool = |dir: &Path, name: &str| {
            fs::write(dir.join(name), format!("#!/bin/sh\necho {name}\n")).unwrap();
            fs::set_permissions(dir.join(name), fs::Permissions::from_mode(0o755)).unwrap();
        };
        tool(&shared, "hidden");
        tool(&shared, "neighbour");
        tool(&own, "other");
        let before: Vec<_> = [&shared, &own].iter().map(|dir| files_under(dir)).collect();
        let path = std::env::join_paths([&shared, &own]).unwrap();
        let under = temp.path().join("stand-ins");
        let hidden = path_hiding("hidden", &path, &under);
        let found = |name: &str| {
            Command::new("/bin/sh").args(["-c", &format!("command -v {name} && {name}")]).env_clear().env("PATH", &hidden).output().unwrap()
        };
        for name in ["neighbour", "other"] {
            let output = found(name);
            assert!(output.status.success(), "{name} is not reachable through {hidden}");
            assert!(String::from_utf8_lossy(&output.stdout).ends_with(&format!("{name}\n")));
        }
        assert!(!found("hidden").status.success(), "the hidden program is reachable through {hidden}");
        assert_eq!([&shared, &own].iter().map(|dir| files_under(dir)).collect::<Vec<_>>(), before);
        let written: Vec<_> = std::env::split_paths(&hidden).filter(|dir| dir != &own).collect();
        assert!(written.iter().all(|dir| dir.starts_with(&under)), "{written:?} is not in the folder given");
    }

    // The sorting example of RFC 8785, section 3.2.3, where UTF-16 order differs from UTF-8 order.
    #[test]
    fn canonical_json_sorts_members_by_utf16_code_units() {
        let value: Value = serde_json::from_str(
            r#"{"€":"Euro Sign","\r":"Carriage Return","דּ":"Hebrew Letter Dalet With Dagesh","1":"One","😀":"Emoji: Grinning Face","\u0080":"Control","ö":"Latin Small Letter O With Diaeresis"}"#,
        )
        .unwrap();
        let names: Vec<String> = canonical_json(&value)
            .split("\",\"")
            .map(|member| member.trim_start_matches("{\"").split("\":\"").next().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["\\r", "1", "\u{80}", "\u{f6}", "\u{20ac}", "\u{1f600}", "\u{fb33}"]);
    }

    #[test]
    fn canonical_json_escapes_as_ecmascript_and_drops_whitespace() {
        let value = json!({ "b": "line\nend \u{f} \"q\" \\ é", "a": [1, -2, true, null] });
        assert_eq!(canonical_json(&value), "{\"a\":[1,-2,true,null],\"b\":\"line\\nend \\u000f \\\"q\\\" \\\\ é\"}");
    }

    #[test]
    fn text_1_folds_only_form_line_endings_and_blanks_at_line_ends() {
        assert_eq!(text_1_normalize("caf\u{65}\u{301}  \r\nnext\t \rlast "), "caf\u{e9}\nnext\nlast");
        assert_eq!(text_1_normalize("\ta\tb\t\n"), "\ta\tb\n");
        assert_eq!(text_1_normalize("a\u{a0}\n"), "a\u{a0}\n");
        assert_eq!(text_1_digest("abc"), "text-1:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn the_last_line_splits_off_with_the_text_before_it_intact() {
        assert_eq!(split_last_line("a\nb\n<!-- x -->\n"), ("a\nb\n".to_string(), "<!-- x -->".to_string()));
        assert_eq!(split_last_line("a\n<!-- x -->"), ("a\n".to_string(), "<!-- x -->".to_string()));
    }
}
