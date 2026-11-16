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
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

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

/// The version the stub and the sample both carry. A test that needs "another version" writes
/// this one into the pin and changes it.
pub const PINNED_CLI: &str = "0.2.0-alpha.1";
pub const OTHER_CLI: &str = "0.2.0-alpha.99";

const TIME_LIMIT: Duration = Duration::from_secs(120);

pub fn sample_source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample")
}

pub fn docs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs")
}

/// Folder that holds the binaries of the workspace. Integration tests run from `target/<profile>/deps`.
pub fn binaries_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("the test binary knows its path");
    let dir = exe
        .parent()
        .and_then(Path::parent)
        .expect("the test binary sits in target/<profile>/deps")
        .to_path_buf();
    for name in ["azimuth", "text-suite-adapter", "facility-probe"] {
        assert!(
            dir.join(name).exists(),
            "{name} is not built in {}. Run `cargo test` from the workspace root, so every member is built.",
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
        copy_tree(&sample_source(), &root);
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
        self.log_entries()
            .into_iter()
            .filter(|entry| entry["program"] == "text-suite-adapter")
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

    pub fn outbox(&self) -> PathBuf {
        self.path(".azimuth/outbox")
    }

    pub fn store_files(&self) -> Vec<PathBuf> {
        files_under(&self.store())
    }

    pub fn outbox_files(&self) -> Vec<PathBuf> {
        files_under(&self.outbox())
    }

    /// Every JSON record in the store and the outbox, parsed.
    pub fn records(&self) -> Vec<(PathBuf, Value)> {
        let mut records = Vec::new();
        for file in self.store_files().into_iter().chain(self.outbox_files()) {
            if file.extension().is_some_and(|ext| ext == "json") {
                if let Ok(value) = serde_json::from_str::<Value>(&fs::read_to_string(&file).unwrap_or_default()) {
                    records.push((file, value));
                }
            }
        }
        records
    }

    /// The records whose content names a Check id. The record format is not fixed by the design,
    /// so a record is found by the id it holds and not by a field name.
    pub fn records_naming(&self, check: &str) -> Vec<(PathBuf, Value)> {
        self.records()
            .into_iter()
            .filter(|(_, value)| all_strings(value).iter().any(|text| text == check))
            .collect()
    }

    /// Rewrites every record in the store and the outbox, and keeps a file name that was the
    /// SHA-256 of the old bytes equal to the SHA-256 of the new bytes, as the design names records.
    pub fn rewrite_records(&self, rewrite: impl Fn(&str) -> String) {
        for file in self.store_files().into_iter().chain(self.outbox_files()) {
            if !file.extension().is_some_and(|ext| ext == "json") {
                continue;
            }
            let old = fs::read_to_string(&file).unwrap();
            let new = rewrite(&old);
            if new == old {
                continue;
            }
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            if stem == sha256_hex(old.as_bytes()) {
                fs::remove_file(&file).unwrap();
                fs::write(file.with_file_name(format!("{}.json", sha256_hex(new.as_bytes()))), new).unwrap();
            } else {
                fs::write(&file, new).unwrap();
            }
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

pub fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("read {}: {e}", from.display())) {
        let entry = entry.unwrap();
        let name = entry.file_name();
        // Build output of the xUnit project may sit in the source folder after a local `dotnet test`.
        if name == "bin" || name == "obj" || name == ".DS_Store" {
            continue;
        }
        let target = to.join(&name);
        if entry.file_type().unwrap().is_dir() {
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
        .map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect())
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

pub fn sha256_hex(bytes: &[u8]) -> String {
    for (program, args) in [("sha256sum", vec![]), ("shasum", vec!["-a", "256"])] {
        let child = Command::new(program)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = child else { continue };
        use std::io::Write;
        child.stdin.take().unwrap().write_all(bytes).unwrap();
        let output = child.wait_with_output().unwrap();
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).split_whitespace().next().unwrap().to_string();
        }
    }
    panic!("neither sha256sum nor shasum is available");
}

/// Rows of `docs/increments/1.md`: (section heading, behavior, quote). The section "Left for
/// increment 4" is not part of the increment and is left out.
pub fn increment_rows() -> Vec<(String, String, String)> {
    let text = fs::read_to_string(docs_dir().join("increments/1.md")).expect("docs/increments/1.md");
    let mut rows = Vec::new();
    let mut section = String::new();
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            section = heading.trim().to_string();
            continue;
        }
        if section.starts_with("Left for increment 4") || !line.starts_with("| ") {
            continue;
        }
        if line.starts_with("| ---") || line.starts_with("| Behavior |") {
            continue;
        }
        let body = &line[2..];
        let split = body.find(" | ").unwrap_or_else(|| panic!("a row has two cells: {line}"));
        let behavior = body[..split].trim().to_string();
        let quote = body[split + 3..].trim_end().trim_end_matches('|').trim_end().to_string();
        rows.push((section.clone(), behavior, quote));
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
    /// document. When a file is given, one error must name it, and the line when one is given.
    pub fn expect_reading_error(&self, file: Option<&str>, line: Option<u64>) -> Value {
        let output = self.azimuth(&["status", "--json"]);
        output.expect_status(2);
        let document = output.json();
        assert_eq!(document["format"], "error-1", "{document:#}");
        let errors = document["errors"].as_array().cloned().unwrap_or_default();
        assert!(!errors.is_empty(), "error-1 lists no error: {document:#}");
        for error in &errors {
            for field in ["kind", "file", "line", "message"] {
                assert!(error.get(field).is_some(), "an error has no `{field}`: {error:#}");
            }
        }
        if let Some(file) = file {
            let named: Vec<&Value> = errors
                .iter()
                .filter(|error| error["file"].as_str().is_some_and(|path| path.ends_with(file)))
                .collect();
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

/// The names of the parts in a `check-1` document's `parts_changed`, lowercased. The design fixes
/// the field and the part names, not its shape, so every string inside the field counts.
pub fn parts_changed(check: &Value) -> Vec<String> {
    all_strings(&check["parts_changed"]).iter().map(|part| part.to_lowercase()).collect()
}

pub fn mentions(parts: &[String], part: &str) -> bool {
    parts.iter().any(|name| name.contains(part))
}

/// The PATH of the machine without the folders that hold a program, so a test can stand on a
/// machine where that program, and every adapter it would start, is absent.
pub fn path_without(program: &str) -> String {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs = vec![binaries_dir()];
    dirs.extend(std::env::split_paths(&path).filter(|dir| !dir.join(program).exists()));
    std::env::join_paths(dirs).unwrap().to_string_lossy().to_string()
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
            for field in ["kind", "file", "line", "message"] {
                assert!(finding.get(field).is_some(), "a finding has no `{field}`: {finding:#}");
            }
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

    /// Every string in every record, so a test can look for a digest without a field name.
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

pub fn files_changed(check: &Value) -> Vec<String> {
    all_strings(&check["files_changed"])
}

impl Repo {
    /// Deletes every record that names a Check, as a person may delete anything in the store.
    pub fn forget(&self, check: &str) {
        let records = self.records_naming(check);
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

/// Raises the number of every format name in a record, as a later CLI version would write it.
pub fn bump_formats(text: &str) -> String {
    let mut value: serde_json::Value = serde_json::from_str(text).unwrap();
    fn walk(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(text) => {
                if let Some((word, number)) = text.rsplit_once('-') {
                    let is_format = !word.is_empty()
                        && word.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                        && !number.is_empty()
                        && number.chars().all(|c| c.is_ascii_digit());
                    if is_format {
                        *text = format!("{word}-999");
                    }
                }
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(walk),
            serde_json::Value::Object(map) => {
                for (key, item) in map.iter_mut() {
                    if (key.contains("schema") || key.contains("version")) && item.is_u64() {
                        *item = serde_json::Value::from(999);
                    } else {
                        walk(item);
                    }
                }
            }
            _ => {}
        }
    }
    walk(&mut value);
    value.to_string()
}

