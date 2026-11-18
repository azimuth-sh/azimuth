//! One launch of a suite: the proofs of its facilities, the adapter started in the suite's
//! folder with one request on standard input, and the report it writes on standard output.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::project::{Project, Suite};
use crate::runners;
use crate::store;

/// A proof that runs longer counts as hung. The number is fixed in the first increment.
const PROOF_TIME_LIMIT: Duration = Duration::from_secs(10);
const STOP_GRACE: Duration = Duration::from_secs(10);
unsafe extern "C" {
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
    let mut info = [0u64; 16];
    let result = unsafe { waitid(1, child.id(), info.as_mut_ptr().cast(), 1 | 4 | no_reap) };
    if result == 0 {
        Ok(info[0] != 0)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn stop_group(child: &Child, started: Instant) -> u64 {
    let group = child.id();
    let signal = |number| {
        if let Err(error) = signal_group(group, number) {
            if error.raw_os_error() != Some(3) {
                eprintln!("azimuth: signal {number} to process group {group}: {error}");
            }
        }
    };
    signal(15);
    let stopped = Instant::now();
    let mut duration_ms = None;
    // Even an empty group can retain an unreaped leader; waiting the grace avoids reusing its id.
    while stopped.elapsed() < STOP_GRACE {
        if duration_ms.is_none() && exited(child).unwrap_or(false) {
            duration_ms = Some(started.elapsed().as_millis() as u64);
        }
        std::thread::sleep(Duration::from_millis(20).min(STOP_GRACE.saturating_sub(stopped.elapsed())));
    }
    signal(9);
    duration_ms.unwrap_or_else(|| started.elapsed().as_millis() as u64)
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
    match wait_with_limit(child, started, Some(PROOF_TIME_LIMIT), None) {
        Ok((Some(status), _)) if status.success() => Ok(()),
        Ok((Some(_), _)) => Err(format!("the proof of `{facility}` failed: `{}`", definition.proof)),
        Ok((None, _)) => Err(format!("the proof of `{facility}` did not end within {} seconds", PROOF_TIME_LIMIT.as_secs())),
        Err(error) => Err(format!("wait for the proof of `{facility}`: {error}")),
    }
}

fn wait_with_limit(
    mut child: Child,
    started: Instant,
    limit: Option<Duration>,
    mut input: Option<(ChildStdin, &[u8])>,
) -> std::io::Result<(Option<std::process::ExitStatus>, u64)> {
    loop {
        if exited(&child)? {
            let duration_ms = started.elapsed().as_millis() as u64;
            return child.wait().map(|status| (Some(status), duration_ms));
        }
        if limit.is_some_and(|limit| started.elapsed() >= limit) {
            let duration_ms = stop_group(&child, started);
            reap(child);
            return Ok((None, duration_ms));
        }
        if let Some((pipe, remaining)) = input.as_mut() {
            match pipe.write(remaining) {
                Ok(count) => *remaining = &remaining[count..],
                Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
                Err(_) => *remaining = &[],
            }
            if remaining.is_empty() {
                input.take();
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[derive(Debug)]
pub struct ReportedTest {
    pub name: String,
    pub checks: Vec<String>,
    pub outcome: String,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct Report {
    pub adapter_name: String,
    pub adapter_version: String,
    pub tests: Vec<ReportedTest>,
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

/// Starts the suite's adapter in the suite's folder with no added arguments, asks it to run the
/// tests of the given Checks, and reads its one report.
pub fn run_suite(root: &Path, suite: &Suite, checks: &[String]) -> Result<(Launch, Option<u64>), String> {
    let request = json!({
        "format": REQUEST_FORMAT,
        "verb": "run",
        "suite": suite.path,
        "marker_key": suite.marker_key(),
        "checks": checks,
    });
    let (path, mut output) = held_output(root).map_err(|error| format!("the runner's output could not be held: {error}"))?;
    let mut report = temporary_file().map_err(|error| format!("prepare the adapter report: {error}"))?;
    let limit = Duration::from_secs(suite.time_limit_minutes.saturating_mul(60));
    let stderr = output.try_clone().map_err(|error| format!("hold the runner's output: {error}"))?;
    eprintln!("suite {}: launch starting", suite.name);
    let (launch, duration_ms) = match adapter(&root.join(&suite.path), &suite.adapter, &request, &mut report, Some(limit), Stdio::from(stderr)) {
        Ok((Some(status), duration_ms)) if status.success() => (read_report(&mut report), Some(duration_ms)),
        Ok((Some(status), duration_ms)) => (Launch::Crashed { detail: format!("the adapter ended with {status} and gave no report") }, Some(duration_ms)),
        Ok((None, duration_ms)) => (Launch::TimedOut, Some(duration_ms)),
        Err(Started::No(why)) => {
            let mut detail = format!("the adapter of the suite `{}` could not start: {why}", suite.name);
            match runners::install_line(&suite.adapter) {
                Some(line) => detail.push_str(&format!(". Install it: {line}")),
                None => detail.push_str(". Install the adapter with its stack's package manager"),
            }
            (Launch::NotStarted { detail }, None)
        }
        Err(Started::Crashed(detail)) => (Launch::Crashed { detail }, None),
    };
    let duration = duration_ms.map(|ms| format!(" in {ms} ms")).unwrap_or_default();
    eprintln!("suite {}: launch {}{duration}", suite.name, launch.word());
    let trouble = match &launch {
        Launch::Crashed { .. } | Launch::TimedOut | Launch::BuildFailed => true,
        Launch::Ran(report) => report.tests.iter().any(|test| test.outcome == "failed"),
        Launch::NotStarted { .. } => false,
    };
    if trouble {
        if let Err(error) = output.rewind().and_then(|_| {
            let length = output.metadata()?.len();
            std::io::copy(&mut output.take(length), &mut std::io::stderr().lock())
        }) {
            eprintln!("azimuth: read the held runner output: {error}");
        }
    } else if let Err(error) = std::fs::remove_file(&path) {
        eprintln!("azimuth: remove the held runner output: {error}");
    }
    Ok((launch, duration_ms))
}

fn read_report(report: &mut File) -> Launch {
    match report_text(report) {
        Ok(stdout) => parse_report(&stdout),
        Err(error) => Launch::Crashed { detail: format!("read the adapter report: {error}") },
    }
}

fn report_text(report: &mut File) -> std::io::Result<String> {
    report.rewind()?;
    let length = report.metadata()?.len();
    let mut text = String::new();
    report.take(length).read_to_string(&mut text)?;
    Ok(text)
}

fn temporary_file() -> std::io::Result<File> {
    // An inherited standard-I/O handle must not make exit depend on a descendant closing a pipe.
    let (path, file) = unique_file(&std::env::temp_dir())?;
    std::fs::remove_file(path)?;
    Ok(file)
}

fn held_output(root: &Path) -> std::io::Result<(std::path::PathBuf, File)> {
    let folder = root.join(store::WORKTREE_FOLDER);
    store::ensure_worktree_folder(&folder).map_err(std::io::Error::other)?;
    unique_file(&folder)
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
    /// It started and ended without a report.
    Crashed(String),
}

/// Starts an adapter command with no added arguments and one request on standard input, and
/// returns what it wrote on standard output when it ended with status 0. The runner's output
/// goes to standard error.
pub fn start_adapter(dir: &Path, command: &str, request: &Value) -> Result<String, Started> {
    let mut report = temporary_file().map_err(|error| Started::Crashed(format!("prepare the adapter report: {error}")))?;
    let (status, _) = adapter(dir, command, request, &mut report, None, Stdio::inherit())?;
    match status {
        Some(status) if status.success() => report_text(&mut report).map_err(|error| Started::Crashed(format!("read the adapter report: {error}"))),
        Some(status) => Err(Started::Crashed(format!("the adapter ended with {status} and gave no report"))),
        None => unreachable!("discovery has no time limit"),
    }
}

fn adapter(
    dir: &Path,
    command: &str,
    request: &Value,
    report: &mut File,
    limit: Option<Duration>,
    stderr: Stdio,
) -> Result<(Option<std::process::ExitStatus>, u64), Started> {
    let words = split_command(command);
    let Some((program, args)) = words.split_first() else {
        return Err(Started::No("the command is empty".to_string()));
    };
    let stdout = report.try_clone().map_err(|error| Started::Crashed(format!("hold the adapter report: {error}")))?;
    let request = request.to_string();
    let started = Instant::now();
    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(stdout))
        .stderr(stderr)
        .spawn()
        .map_err(|error| Started::No(format!("`{program}`: {error}")))?;
    let input = child.stdin.take().expect("stdin is piped");
    #[cfg(target_os = "macos")]
    let nonblocking = 4;
    #[cfg(not(target_os = "macos"))]
    let nonblocking = 0x800;
    let flags = unsafe { fcntl(input.as_raw_fd(), 3) };
    if flags < 0 || unsafe { fcntl(input.as_raw_fd(), 4, flags | nonblocking) } < 0 {
        let error = std::io::Error::last_os_error();
        stop_group(&child, started);
        reap(child);
        return Err(Started::Crashed(format!("prepare adapter input: {error}")));
    }
    wait_with_limit(child, started, limit, Some((input, request.as_bytes()))).map_err(|error| Started::Crashed(format!("wait for the adapter: {error}")))
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
    Launch::Ran(Report { adapter_name: name.to_string(), adapter_version: version.to_string(), tests })
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
            Verdict::Mismatch(format!("a marker in source matches no listed test: {markers} marker lines and {} listed tests carry this Check", tests.len()))
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
