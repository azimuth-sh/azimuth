//! One launch of a suite: the proofs of its facilities, the adapter started in the suite's
//! folder with one request on standard input, and the report it writes on standard output.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::project::{Project, Suite};
use crate::runners;

/// A proof that runs longer counts as hung. The number is fixed in the first increment.
const PROOF_TIME_LIMIT: Duration = Duration::from_secs(10);

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
    let mut child = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("the proof of `{facility}` could not start: `{program}`: {error}"))?;
    match wait_with_limit(&mut child, PROOF_TIME_LIMIT) {
        Some(status) if status.success() => Ok(()),
        Some(_) => Err(format!("the proof of `{facility}` failed: `{}`", definition.proof)),
        None => Err(format!("the proof of `{facility}` did not end within {} seconds", PROOF_TIME_LIMIT.as_secs())),
    }
}

fn wait_with_limit(child: &mut Child, limit: Duration) -> Option<std::process::ExitStatus> {
    let started = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Some(status);
        }
        if started.elapsed() >= limit {
            let _ = child.kill();
            let _ = child.wait();
            return None;
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
    BuildFailed,
    Ran(Report),
}

impl Launch {
    /// The word of `launch` in `run-1`.
    pub fn word(&self) -> &'static str {
        match self {
            Launch::NotStarted { .. } => "not started",
            Launch::Crashed { .. } => "crashed",
            Launch::BuildFailed => "build failed",
            Launch::Ran(_) => "ok",
        }
    }
}

/// Starts the suite's adapter in the suite's folder with no added arguments, asks it to run the
/// tests of the given Checks, and reads its one report.
pub fn run_suite(root: &Path, suite: &Suite, checks: &[String]) -> Launch {
    let request = json!({
        "format": REQUEST_FORMAT,
        "verb": "run",
        "suite": suite.path,
        "marker_key": suite.marker_key(),
        "checks": checks,
    });
    match start_adapter(&root.join(&suite.path), &suite.adapter, &request) {
        Ok(stdout) => parse_report(&stdout),
        Err(Started::No(why)) => {
            let mut detail = format!("the adapter of the suite `{}` could not start: {why}", suite.name);
            match runners::install_line(&suite.adapter) {
                Some(line) => detail.push_str(&format!(". Install it: {line}")),
                None => detail.push_str(". Install the adapter with its stack's package manager"),
            }
            Launch::NotStarted { detail }
        }
        Err(Started::Crashed(detail)) => Launch::Crashed { detail },
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
    let words = split_command(command);
    let Some((program, args)) = words.split_first() else {
        return Err(Started::No("the command is empty".to_string()));
    };
    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| Started::No(format!("`{program}`: {error}")))?;
    let mut stdin = child.stdin.take().expect("stdin is piped");
    let request = request.to_string();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(request.as_bytes());
    });
    let mut stdout = String::new();
    let read = child.stdout.take().expect("stdout is piped").read_to_string(&mut stdout);
    let _ = writer.join();
    match (read, child.wait()) {
        (Ok(_), Ok(status)) if status.success() => Ok(stdout),
        (_, Ok(status)) => Err(Started::Crashed(format!("the adapter ended with {status} and gave no report"))),
        (_, Err(error)) => Err(Started::Crashed(format!("the adapter could not be waited for: {error}"))),
    }
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
