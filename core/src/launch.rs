//! One launch of a suite: the proofs of its facilities, the adapter started in the suite's
//! folder with one request on standard input, and the report it writes on standard output.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::FileExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::model::Condition;
use crate::project::{Project, Suite};
use crate::runners;
use crate::store;

/// A proof that runs longer counts as hung. The number is fixed in the first increment.
const PROOF_TIME_LIMIT: Duration = Duration::from_secs(10);
const STOP_GRACE: Duration = Duration::from_secs(10);
static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn request_stop(_: i32) {
    // Shutdown uses normal control flow; a signal handler must not touch process or file state.
    STOP_REQUESTED.store(true, Ordering::Relaxed);
}

pub fn install_stop_handlers() -> std::io::Result<()> {
    for number in [2, 15] {
        if unsafe { signal(number, request_stop) } == usize::MAX {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

pub fn exit_if_stopped() {
    if STOP_REQUESTED.load(Ordering::Relaxed) {
        std::process::exit(1);
    }
}

unsafe extern "C" {
    fn signal(number: i32, handler: extern "C" fn(i32)) -> usize;
    fn kill(pid: i32, signal: i32) -> i32;
    fn fcntl(fd: i32, command: i32, ...) -> i32;
    fn waitid(kind: i32, pid: u32, info: *mut std::ffi::c_void, options: i32) -> i32;
}

fn signal_group(group: u32, signal: i32) -> std::io::Result<()> {
    // A negative pid reaches descendants while the unreaped leader reserves the group id.
    if unsafe { kill(-(group as i32), signal) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn exited(child: &Child) -> std::io::Result<bool> {
    // WNOWAIT preserves the leader's identity until every possible group signal is finished.
    #[cfg(target_os = "macos")]
    let no_reap = 0x20;
    #[cfg(not(target_os = "macos"))]
    let no_reap = 0x01000000;
    loop {
        let mut info = [0u64; 16];
        let result = unsafe { waitid(1, child.id(), info.as_mut_ptr().cast(), 1 | 4 | no_reap) };
        if result == 0 {
            return Ok(info[0] != 0);
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

struct Snapshot {
    duration_ms: u64,
    lengths: Vec<u64>,
}

impl Snapshot {
    fn take(started: Instant, files: &[&File]) -> std::io::Result<Snapshot> {
        let duration_ms = started.elapsed().as_millis() as u64;
        let lengths = files.iter().map(|file| file.metadata().map(|metadata| metadata.len())).collect::<std::io::Result<_>>()?;
        Ok(Snapshot { duration_ms, lengths })
    }
}

struct Completed {
    status: std::process::ExitStatus,
    snapshot: Snapshot,
}

enum Waited {
    Exited(Completed),
    TimedOut(Snapshot),
    Failed { error: std::io::Error, snapshot: Snapshot },
}

fn stop_group(child: &Child, started: Instant, files: &[&File]) -> std::io::Result<Snapshot> {
    let group = child.id();
    let signal = |number| {
        if let Err(error) = signal_group(group, number) {
            if error.raw_os_error() != Some(3) {
                eprintln!("azimuth: signal {number} to process group {group}: {error}");
            }
        }
    };
    signal(2);
    let stopped = Instant::now();
    let mut snapshot = None;
    // Even an empty group can retain an unreaped leader; waiting the grace avoids reusing its id.
    while stopped.elapsed() < STOP_GRACE {
        if snapshot.is_none() && exited(child).unwrap_or(false) {
            snapshot = Some(Snapshot::take(started, files));
        }
        std::thread::sleep(Duration::from_millis(20).min(STOP_GRACE.saturating_sub(stopped.elapsed())));
    }
    signal(9);
    snapshot.unwrap_or_else(|| Snapshot::take(started, files))
}

fn reap(mut child: Child) {
    if !matches!(child.try_wait(), Ok(Some(_))) {
        // A leader that escaped its group must not hold the command open either.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

pub const REQUEST_FORMAT: &str = "adapter-request-1";
pub const REPORT_FORMAT: &str = "adapter-report-1";

/// A command line from the project file, split on spaces. No shell sees it.
fn split_command(line: &str) -> Vec<String> {
    line.split(' ').filter(|word| !word.is_empty()).map(str::to_string).collect()
}

/// Proves one facility by its proof command, started at the root of the repository. The error
/// says why the suite cannot run here.
pub fn prove(project: &Project, facility: &str, root: &Path) -> Result<(), String> {
    exit_if_stopped();
    let Some(definition) = project.facility(facility) else {
        return Err(format!("the facility `{facility}` has no definition in the project file"));
    };
    let words = split_command(&definition.proof);
    let Some((program, args)) = words.split_first() else {
        return Err(format!("the facility `{facility}` has an empty proof"));
    };
    let started = Instant::now();
    let child = Command::new(program)
        .args(args)
        .current_dir(root)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("the proof of `{facility}` could not start: `{program}`: {error}"))?;
    let running = Running { child, started, input: None };
    match running.wait_with_limit(PROOF_TIME_LIMIT, &[]) {
        Ok(Waited::Exited(completed)) if completed.status.success() => Ok(()),
        Ok(Waited::Exited(_)) => Err(format!("the proof of `{facility}` failed: `{}`", definition.proof)),
        Ok(Waited::TimedOut(_)) => Err(format!("the proof of `{facility}` did not end within {} seconds", PROOF_TIME_LIMIT.as_secs())),
        Ok(Waited::Failed { error, .. }) => Err(format!("wait for the proof of `{facility}`: {error}")),
        Err(error) => Err(format!("wait for the proof of `{facility}`: {error}")),
    }
}

struct Running<'a> {
    child: Child,
    started: Instant,
    input: Option<(ChildStdin, &'a [u8])>,
}

impl Running<'_> {
    fn poll_exit(&mut self, files: &[&File]) -> std::io::Result<Option<Completed>> {
        if exited(&self.child)? {
            let snapshot = Snapshot::take(self.started, files)?;
            let status = self.child.wait()?;
            return Ok(Some(Completed { status, snapshot }));
        }
        Ok(None)
    }

    fn deliver_request(&mut self) {
        if let Some((pipe, remaining)) = self.input.as_mut() {
            match pipe.write(remaining) {
                Ok(count) => *remaining = &remaining[count..],
                Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
                Err(_) => *remaining = &[],
            }
            if remaining.is_empty() {
                self.input.take();
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    fn stop(mut self, files: &[&File]) -> std::io::Result<Snapshot> {
        self.input.take();
        let snapshot = stop_group(&self.child, self.started, files);
        reap(self.child);
        snapshot
    }

    fn wait_with_limit(mut self, limit: Duration, files: &[&File]) -> std::io::Result<Waited> {
        loop {
            if STOP_REQUESTED.load(Ordering::Relaxed) {
                let _ = self.stop(files);
                std::process::exit(1);
            }
            match self.poll_exit(files) {
                Ok(Some(completed)) => return Ok(Waited::Exited(completed)),
                Ok(None) => {}
                Err(error) => {
                    return self.stop(files).map(|snapshot| Waited::Failed { error, snapshot });
                }
            }
            if self.started.elapsed() >= limit {
                return self.stop(files).map(Waited::TimedOut);
            }
            self.deliver_request();
        }
    }

    fn wait_until_exit(mut self, files: &[&File]) -> std::io::Result<Completed> {
        loop {
            if STOP_REQUESTED.load(Ordering::Relaxed) {
                let _ = self.stop(files);
                std::process::exit(1);
            }
            match self.poll_exit(files) {
                Ok(Some(completed)) => return Ok(completed),
                Ok(None) => self.deliver_request(),
                Err(error) => {
                    let _ = self.stop(files);
                    return Err(error);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ReportedTest {
    pub name: String,
    pub checks: Vec<String>,
    pub outcome: String,
    pub duration_ms: u64,
}

/// One read of a component's report: the document as read, or the cause it could not be read.
#[derive(Debug)]
pub struct ComponentRead {
    pub phase: String,
    pub result: Result<Value, String>,
}

impl ComponentRead {
    fn to_json(&self) -> Value {
        match &self.result {
            Ok(report) => json!({ "phase": self.phase, "report": report }),
            Err(error) => json!({ "phase": self.phase, "error": error }),
        }
    }
}

#[derive(Debug)]
pub struct ComponentEntry {
    pub name: String,
    /// A cause for the whole component, as when something answered at its address before the start.
    pub error: Option<String>,
    pub reads: Vec<ComponentRead>,
}

impl ComponentEntry {
    pub fn to_json(&self) -> Value {
        let mut entry = json!({ "name": self.name, "reads": self.reads.iter().map(ComponentRead::to_json).collect::<Vec<_>>() });
        if let Some(error) = &self.error {
            entry["error"] = json!(error);
        }
        entry
    }
}

#[derive(Debug)]
pub struct Report {
    pub adapter_name: String,
    pub adapter_version: String,
    pub tests: Vec<ReportedTest>,
    /// `None` when the report has no `components`, as the report of an adapter that predates them.
    pub components: Option<Vec<ComponentEntry>>,
}

#[derive(Debug)]
pub enum Launch {
    NotStarted { detail: String },
    Crashed { detail: String },
    TimedOut,
    BuildFailed,
    Ran(Report),
}

impl Launch {
    /// The reads the launch collected, as `run-1` and `result-1` keep them. A launch that gave no
    /// report, or an adapter that reports no reads, keeps an empty list.
    pub fn components_json(&self) -> Value {
        match self {
            Launch::Ran(Report { components: Some(entries), .. }) => entries.iter().map(ComponentEntry::to_json).collect(),
            _ => json!([]),
        }
    }

    /// The word of `launch` in `run-1`.
    pub fn word(&self) -> &'static str {
        match self {
            Launch::NotStarted { .. } => "not started",
            Launch::Crashed { .. } => "crashed",
            Launch::TimedOut => "timed out",
            Launch::BuildFailed => "build failed",
            Launch::Ran(_) => "ok",
        }
    }
}

/// The variable and the request field that carry a launch's condition: one object of product keys
/// to normalized text values.
fn condition_object(condition: &Condition) -> Value {
    json!(condition.texts)
}

/// Starts the suite's adapter in the suite's folder with no added arguments, asks it to run the
/// tests of the given Checks under the condition of this launch, and reads its one report.
pub fn run_suite(root: &Path, project: &Project, suite: &Suite, condition: &Condition, checks: &[String]) -> Result<(Launch, Option<u64>), String> {
    let components: Vec<Value> = suite
        .starts
        .iter()
        .filter_map(|name| project.component(name))
        .map(|component| json!({ "name": component.name, "report": component.report }))
        .collect();
    let condition_of_request = condition;
    let condition = condition_object(condition);
    let request = json!({
        "format": REQUEST_FORMAT,
        "verb": "run",
        "suite": suite.path,
        "marker_key": suite.marker_key(),
        "checks": checks,
        "condition": condition,
        "components": components,
    });
    let mut output = held_output(root).map_err(|error| format!("the runner's output could not be held: {error}"))?;
    let report = temporary_file().map_err(|error| format!("prepare the adapter report: {error}"))?;
    let stderr = output.file.try_clone().map_err(|error| format!("hold the runner's output: {error}"))?;
    let stdout = report.try_clone().map_err(|error| format!("hold the adapter report: {error}"))?;
    let request = request.to_string();
    let limit = Duration::from_secs(suite.time_limit_minutes.saturating_mul(60));
    let named = crate::contexts::condition_text(&condition_of_request);
    eprintln!("suite {} ({named}): launch starting", suite.name);
    let waited =
        spawn_adapter(&root.join(&suite.path), &suite.adapter, request.as_bytes(), stdout, Stdio::from(stderr), Some(&condition.to_string())).and_then(|running| {
            running
                .wait_with_limit(limit, &[&report, &output.file])
                .map_err(|error| Started::Crashed(format!("wait for the adapter: {error}")))
        });
    let mut output_length = 0;
    let (launch, duration_ms) = match waited {
        Ok(Waited::Exited(completed)) => {
            output_length = completed.snapshot.lengths[1];
            let launch = if completed.status.success() {
                read_report(&report, completed.snapshot.lengths[0])
            } else {
                Launch::Crashed { detail: format!("the adapter ended with {} and gave no report", completed.status) }
            };
            (launch, Some(completed.snapshot.duration_ms))
        }
        Ok(Waited::TimedOut(snapshot)) => {
            output_length = snapshot.lengths[1];
            (Launch::TimedOut, Some(snapshot.duration_ms))
        }
        Ok(Waited::Failed { error, snapshot }) => {
            output_length = snapshot.lengths[1];
            (Launch::Crashed { detail: format!("wait for the adapter: {error}") }, Some(snapshot.duration_ms))
        }
        Err(Started::No(why)) => {
            let mut detail = format!("the adapter of the suite `{}` could not start: {why}", suite.name);
            match runners::install_line(&suite.adapter) {
                Some(line) => detail.push_str(&format!(". Install it: {line}")),
                None => detail.push_str(". Install the adapter with its stack's package manager"),
            }
            (Launch::NotStarted { detail }, None)
        }
        Err(Started::Crashed(detail)) => (Launch::Crashed { detail }, None),
        Err(Started::Preparation(detail)) => return Err(detail),
    };
    let duration = duration_ms.map(|ms| format!(" in {ms} ms")).unwrap_or_default();
    eprintln!("suite {} ({named}): launch {}{duration}", suite.name, launch.word());
    let trouble = match &launch {
        Launch::Crashed { .. } | Launch::TimedOut | Launch::BuildFailed => true,
        Launch::Ran(report) => report.tests.iter().any(|test| test.outcome == "failed"),
        Launch::NotStarted { .. } => false,
    };
    if trouble {
        output.retain = true;
        if let Err(error) = replay(&output.file, output_length) {
            eprintln!("azimuth: read the held runner output: {error}");
        }
    }
    Ok((launch, duration_ms))
}

struct Captured<'a> {
    file: &'a File,
    position: u64,
    length: u64,
}

impl Read for Captured<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = buffer.len().min(self.length.saturating_sub(self.position) as usize);
        let count = self.file.read_at(&mut buffer[..count], self.position)?;
        self.position += count as u64;
        Ok(count)
    }
}

fn replay(file: &File, length: u64) -> std::io::Result<()> {
    let mut input = Captured { file, position: 0, length };
    let mut stderr = std::io::stderr().lock();
    std::io::copy(&mut input, &mut stderr)?;
    if length > 0 {
        let mut last = [0];
        if file.read_at(&mut last, length - 1)? == 1 && last != *b"\n" {
            stderr.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn read_report(report: &File, length: u64) -> Launch {
    match report_text(report, length) {
        Ok(stdout) => parse_report(&stdout),
        Err(error) => Launch::Crashed { detail: format!("read the adapter report: {error}") },
    }
}

fn report_text(report: &File, length: u64) -> std::io::Result<String> {
    let mut text = String::new();
    Captured { file: report, position: 0, length }.read_to_string(&mut text)?;
    Ok(text)
}

struct HeldOutput {
    path: std::path::PathBuf,
    file: File,
    retain: bool,
}

impl Drop for HeldOutput {
    fn drop(&mut self) {
        if !self.retain {
            if let Err(error) = std::fs::remove_file(&self.path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("azimuth: remove the held runner output: {error}");
                }
            }
        }
    }
}

fn temporary_file() -> std::io::Result<File> {
    // An inherited standard-I/O handle must not make exit depend on a descendant closing a pipe.
    let (path, file) = unique_file(&std::env::temp_dir())?;
    std::fs::remove_file(path)?;
    Ok(file)
}

fn held_output(root: &Path) -> std::io::Result<HeldOutput> {
    let folder = root.join(store::WORKTREE_FOLDER);
    store::ensure_worktree_folder(&folder).map_err(std::io::Error::other)?;
    let (path, file) = unique_file(&folder)?;
    Ok(HeldOutput { path, file, retain: false })
}

fn unique_file(folder: &Path) -> std::io::Result<(std::path::PathBuf, File)> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let path = folder.join(format!("launch-{}-{}.log", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        match OpenOptions::new().read(true).write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

pub enum Started {
    /// The command could not be started at all.
    No(String),
    /// A local file could not be prepared, so the command cannot launch the adapter.
    Preparation(String),
    /// The adapter started but did not complete with a usable report.
    Crashed(String),
}

/// Starts an adapter command with no added arguments and one request on standard input, and
/// returns what it wrote on standard output when it ended with status 0. The runner's output
/// goes to standard error.
pub fn start_adapter(dir: &Path, command: &str, request: &Value) -> Result<String, Started> {
    let report = temporary_file().map_err(|error| Started::Preparation(format!("prepare the adapter report: {error}")))?;
    let stdout = report.try_clone().map_err(|error| Started::Preparation(format!("hold the adapter report: {error}")))?;
    let request = request.to_string();
    let running = spawn_adapter(dir, command, request.as_bytes(), stdout, Stdio::inherit(), None)?;
    let completed = running.wait_until_exit(&[&report]).map_err(|error| Started::Crashed(format!("wait for the adapter: {error}")))?;
    if completed.status.success() {
        report_text(&report, completed.snapshot.lengths[0]).map_err(|error| Started::Crashed(format!("read the adapter report: {error}")))
    } else {
        Err(Started::Crashed(format!("the adapter ended with {} and gave no report", completed.status)))
    }
}

/// `context` is the launch's `AZIMUTH_CONTEXT`. It replaces any value the core inherited, and
/// without one the adapter gets the environment unchanged.
fn spawn_adapter<'a>(
    dir: &Path,
    command: &str,
    request: &'a [u8],
    stdout: File,
    stderr: Stdio,
    context: Option<&str>,
) -> Result<Running<'a>, Started> {
    exit_if_stopped();
    let words = split_command(command);
    let Some((program, args)) = words.split_first() else {
        return Err(Started::No("the command is empty".to_string()));
    };
    let started = Instant::now();
    let mut command = Command::new(program);
    command.args(args).current_dir(dir).process_group(0).stdin(Stdio::piped()).stdout(Stdio::from(stdout)).stderr(stderr);
    if let Some(context) = context {
        command.env("AZIMUTH_CONTEXT", context);
    }
    let mut child = command.spawn().map_err(|error| Started::No(format!("`{program}`: {error}")))?;
    let input = child.stdin.take().expect("stdin is piped");
    #[cfg(target_os = "macos")]
    let nonblocking = 4;
    #[cfg(not(target_os = "macos"))]
    let nonblocking = 0x800;
    let flags = unsafe { fcntl(input.as_raw_fd(), 3) };
    if flags < 0 || unsafe { fcntl(input.as_raw_fd(), 4, flags | nonblocking) } < 0 {
        let error = std::io::Error::last_os_error();
        drop(input);
        let _ = stop_group(&child, started, &[]);
        reap(child);
        return Err(Started::Crashed(format!("prepare adapter input: {error}")));
    }
    Ok(Running { child, started, input: Some((input, request)) })
}

/// A report is one JSON document of the format `adapter-report-1`; anything else is a crash.
fn parse_report(stdout: &str) -> Launch {
    let crashed = |why: &str| Launch::Crashed { detail: format!("the adapter wrote {why}") };
    let mut documents = serde_json::Deserializer::from_str(stdout).into_iter::<Value>();
    let report = match documents.next() {
        Some(Ok(report)) => report,
        _ => return crashed("no report"),
    };
    if documents.next().is_some() {
        return crashed("more than one report");
    }
    if report["format"] != REPORT_FORMAT {
        return crashed("a document that is not an adapter-report-1");
    }
    if report["build"] == "failed" {
        return Launch::BuildFailed;
    }
    if report["build"] != "ok" {
        return crashed("a report without `build`");
    }
    let (Some(name), Some(version)) = (report["adapter"]["name"].as_str(), report["adapter"]["version"].as_str()) else {
        return crashed("a report that does not name the adapter and its version");
    };
    let Some(items) = report["tests"].as_array() else {
        return crashed("a report without `tests`");
    };
    let mut tests = Vec::new();
    for item in items {
        let test = (|| {
            Some(ReportedTest {
                name: item["name"].as_str()?.to_string(),
                checks: item["checks"].as_array()?.iter().map(|check| check.as_str().map(str::to_string)).collect::<Option<_>>()?,
                outcome: item["outcome"].as_str()?.to_string(),
                duration_ms: item["duration_ms"].as_u64()?,
            })
        })();
        match test {
            Some(test) => tests.push(test),
            None => return crashed("a test entry without `name`, `checks`, `outcome` and `duration_ms`"),
        }
    }
    let components = match report.get("components") {
        None => None,
        Some(Value::Array(items)) => Some(items.iter().filter_map(read_component).collect()),
        Some(_) => return crashed("a report whose `components` is not a list"),
    };
    Launch::Ran(Report { adapter_name: name.to_string(), adapter_version: version.to_string(), tests, components })
}

/// An entry that names no component is dropped, so the component it was meant for has no read.
fn read_component(item: &Value) -> Option<ComponentEntry> {
    let name = item["name"].as_str()?.to_string();
    let reads = item["reads"]
        .as_array()
        .map(|reads| {
            reads
                .iter()
                .filter_map(|read| {
                    let phase = read["phase"].as_str()?.to_string();
                    let result = match (read.get("report"), read["error"].as_str()) {
                        (Some(report), _) => Ok(report.clone()),
                        (None, Some(error)) => Err(error.to_string()),
                        (None, None) => Err("the read holds neither a report nor an error".to_string()),
                    };
                    Some(ComponentRead { phase, result })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(ComponentEntry { name, error: item["error"].as_str().map(str::to_string), reads })
}

/// The Check rule over the marked tests of one Check.
#[derive(Debug)]
pub enum Verdict<'r> {
    Passed(Vec<&'r ReportedTest>),
    Failed(Vec<&'r ReportedTest>),
    NoResult(String),
    /// The report and the markers in source disagree. The Check gets no result, and the
    /// disagreement is a finding.
    Mismatch(String),
}

/// Applies the Check rule to the asked Checks of a report, after the report is compared with the
/// markers in source. A listed test whose Check id appears nowhere in source leaves that Check
/// with no result, asked or not. The core knows no language, so it cannot tell which test a
/// marker sits on; it compares counts instead: a marker enrolls at least one test, so more marker
/// lines than listed tests for a Check means some marker matches no listed test.
pub fn judge<'r>(
    report: &'r Report,
    asked: &[String],
    markers_in_suite: &BTreeMap<String, usize>,
    marked_anywhere: &BTreeSet<String>,
) -> BTreeMap<String, Verdict<'r>> {
    let mut verdicts = BTreeMap::new();
    for test in &report.tests {
        for check in &test.checks {
            if !marked_anywhere.contains(check) {
                let detail = format!("the listed test {} carries this Check id, and no marker in source shows it", test.name);
                verdicts.insert(check.clone(), Verdict::Mismatch(detail));
            }
        }
    }
    for check in asked {
        if verdicts.contains_key(check) {
            continue;
        }
        let tests: Vec<&ReportedTest> = report.tests.iter().filter(|test| test.checks.contains(check)).collect();
        let markers = markers_in_suite.get(check).copied().unwrap_or(0);
        let verdict = if markers > tests.len() {
            Verdict::Mismatch(format!(
                "a marker in source matches no listed test: {markers} marker lines and {} listed tests carry this Check",
                tests.len()
            ))
        } else if tests.iter().any(|test| test.outcome == "failed") {
            Verdict::Failed(tests)
        } else if tests.iter().all(|test| test.outcome == "passed") {
            Verdict::Passed(tests)
        } else {
            Verdict::NoResult("a marked test was skipped or not run".to_string())
        };
        verdicts.insert(check.clone(), verdict);
    }
    verdicts
}
