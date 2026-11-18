//! The adapter for the sample's text-file suite. It speaks the adapter contract of the design:
//! one `adapter-request-1` document on standard input, one `adapter-report-1` document on
//! standard output, and the status 0 whenever a report was written.
//!
//! The core starts an adapter in its suite's folder, so the adapter reads its tests from its
//! working directory, as a real runner finds its project there. The request's `suite`, the path
//! from the repository root, is only logged. Started anywhere else, the adapter reads the wrong
//! files, and the tests of the core notice.
//!
//! The syntax of a text test file is test apparatus, not product. A file in the suite folder is
//! read line by line:
//!
//! - `# <key>: <id>` is a marker, where `<key>` is the request's `marker_key`. A marker before the
//!   first `test:` line belongs to the whole file, like a marker on a test class. A marker after a
//!   `test:` line belongs to that test only.
//! - `test: <name>` starts a named test. A file with no `test:` line is one test.
//! - `outcome: passed|failed|skipped|not_run` is the outcome of the current test when it runs.
//! - `duration: <ms>` is the duration the report gives the current test.
//! - `kind: helper` says the file is not a test. Its markers stay in source, but the adapter lists
//!   nothing from it, as a runner lists no test for a marker on a helper method.
//! - Any other line starting with `#` is a comment.
//!
//! A line starting with `adapter:` makes the adapter misbehave, so that tests can reach the paths
//! of the core that a well-behaved runner never takes:
//!
//! - `adapter: crash` ends with status 3 and writes no report.
//! - `adapter: garbage` ends with status 0 and writes text that is not a report.
//! - `adapter: two-reports` ends with status 0 and writes two reports.
//! - `adapter: build-fails` writes a report with `"build": "failed"` and no tests.
//! - `adapter: lists <name> <check-id>` adds a listed test carrying a Check id that no marker in
//!   source shows, as a trait built from a constant would.
//! - `adapter: sleeps <ms>` waits that long before it writes its report, so a test knows a lower
//!   bound for the wall time of the launch.
//! - `adapter: edits <path>` appends a line to that file, a path from the repository root,
//!   while the launch runs. The root is the nearest folder above that holds `.git`.
//!
//! Three environment variables reach what the files cannot hold without moving the fingerprint:
//! `TEXT_ADAPTER_VERSION` overrides the version the adapter reports, `TEXT_ADAPTER_FAIL` names
//! tests, separated by commas, that fail whatever their file says, as an unstable test or a fault
//! of the machine would, and `APPARATUS_LOG` names a file outside the repository where each start
//! is recorded for the tests to inspect, with the value of `APPARATUS_MARK`, so a test can see
//! that the core passed its environment on.

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

const NAME: &str = "text-suite-adapter";

struct TextTest {
    name: String,
    checks: Vec<String>,
    outcome: String,
    duration_ms: u64,
}

#[derive(Default)]
struct Directives {
    crash: bool,
    garbage: bool,
    two_reports: bool,
    build_fails: bool,
    extra_listed: Vec<(String, String)>,
    edits: Vec<String>,
    sleep_ms: u64,
}

fn main() {
    log(
        json!({"program":NAME,"event":"started","args":std::env::args().skip(1).collect::<Vec<_>>() }),
    );
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("standard input is readable");
    let args: Vec<String> = std::env::args().skip(1).collect();

    let request: Value = match serde_json::from_str(&input) {
        Ok(value) => value,
        Err(error) => {
            log(
                json!({ "program": NAME, "args": args, "stdin": input, "error": error.to_string() }),
            );
            eprintln!("{NAME}: standard input is not one JSON document: {error}");
            std::process::exit(64);
        }
    };

    let version = std::env::var("TEXT_ADAPTER_VERSION").unwrap_or_else(|_| "1.0.0".to_string());
    let adapter = json!({ "name": NAME, "version": version });
    let verb = request["verb"].as_str().unwrap_or_default().to_string();

    match verb.as_str() {
        "discover" => {
            log(json!({ "program": NAME, "args": args, "request": request, "ran": [] }));
            emit(&json!({ "format": "adapter-report-1", "adapter": adapter, "projects": [] }));
        }
        "run" => run(&request, adapter, args),
        other => {
            log(
                json!({ "program": NAME, "args": args, "request": request, "error": "unknown verb" }),
            );
            eprintln!("{NAME}: unknown verb {other:?}");
            std::process::exit(64);
        }
    }
}

fn run(request: &Value, adapter: Value, args: Vec<String>) {
    let key = request["marker_key"].as_str().unwrap_or("Check");
    let asked: BTreeSet<String> = request["checks"]
        .as_array()
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let suite_dir = PathBuf::from(".");
    let mut files = Vec::new();
    collect_files(&suite_dir, &mut files);
    files.sort();

    let mut directives = Directives::default();
    let mut tests = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default();
        read_directives(&text, &mut directives);
        let relative = file.strip_prefix(&suite_dir).unwrap_or(file);
        tests.extend(read_tests(relative, &text, key));
    }

    if output_folder().is_some() {
        output_checkpoint("first", true);
        output_checkpoint("second", true);
    }

    if directives.crash {
        log(
            json!({ "program": NAME, "args": args, "request": request, "ran": [], "crashed": true }),
        );
        eprintln!("{NAME}: crashing on request of the suite");
        output_checkpoint("exit", false);
        std::process::exit(3);
    }

    for path in &directives.edits {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(repository_root().join(path))
            .expect("the file to edit during the launch is writable");
        writeln!(file, "edited while the launch ran").expect("the edit is written");
    }

    std::thread::sleep(std::time::Duration::from_millis(directives.sleep_ms));

    if directives.garbage {
        log(json!({ "program": NAME, "args": args, "request": request, "ran": [] }));
        println!("this is not an adapter report");
        output_checkpoint("exit", false);
        return;
    }

    if directives.build_fails {
        log(
            json!({ "program": NAME, "args": args, "request": request, "ran": [], "build": "failed" }),
        );
        eprintln!("{NAME}: the suite does not build, on request of the suite");
        emit(
            &json!({ "format": "adapter-report-1", "adapter": adapter, "build": "failed", "tests": [] }),
        );
        return;
    }

    for (name, check) in &directives.extra_listed {
        tests.push(TextTest {
            name: name.clone(),
            checks: vec![check.clone()],
            outcome: "passed".to_string(),
            duration_ms: 1,
        });
    }

    let forced_failures: BTreeSet<String> = std::env::var("TEXT_ADAPTER_FAIL")
        .unwrap_or_default()
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect();
    for test in &mut tests {
        if forced_failures.contains(&test.name) {
            test.outcome = "failed".to_string();
        }
    }

    let mut ran = Vec::new();
    let mut reported = Vec::new();
    for test in tests.iter().filter(|test| !test.checks.is_empty()) {
        let wanted = test.checks.iter().any(|check| asked.contains(check));
        let (outcome, duration_ms) = if wanted {
            ran.push(test.name.clone());
            eprintln!("{NAME}: {} {}", test.name, test.outcome);
            (test.outcome.clone(), test.duration_ms)
        } else {
            ("not_run".to_string(), 0)
        };
        reported.push(json!({
            "name": test.name,
            "checks": test.checks,
            "outcome": outcome,
            "duration_ms": duration_ms,
        }));
    }

    log(json!({ "program": NAME, "args": args, "request": request, "ran": ran }));
    let report = json!({ "format": "adapter-report-1", "adapter": adapter, "build": "ok", "tests": reported });
    emit(&report);
    if directives.two_reports {
        emit(&report);
    }
}

fn repository_root() -> PathBuf {
    let here = std::env::current_dir().expect("the working directory is readable");
    here.ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
        .expect("the adapter runs inside a git repository")
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn read_directives(text: &str, directives: &mut Directives) {
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("adapter:") else {
            continue;
        };
        let words: Vec<&str> = rest.split_whitespace().collect();
        match words.as_slice() {
            ["crash"] => directives.crash = true,
            ["garbage"] => directives.garbage = true,
            ["two-reports"] => directives.two_reports = true,
            ["build-fails"] => directives.build_fails = true,
            ["lists", name, check] => directives.extra_listed.push((name.to_string(), check.to_string())),
            ["edits", path] => directives.edits.push(path.to_string()),
            ["sleeps", ms] => directives.sleep_ms += ms.parse::<u64>().expect("sleeps names whole milliseconds"),
            _ => eprintln!("{NAME}: unknown directive {line:?}"),
        }
    }
}

fn read_tests(relative: &Path, text: &str, key: &str) -> Vec<TextTest> {
    let marker_prefix = format!("# {key}:");
    let stem = relative.with_extension("").to_string_lossy().replace('\\', "/");

    let mut is_helper = false;
    let mut file_checks = Vec::new();
    let mut file_outcome = None;
    let mut file_duration = None;
    let mut named: Vec<TextTest> = Vec::new();

    for line in text.lines().map(str::trim) {
        if let Some(id) = line.strip_prefix(&marker_prefix) {
            match named.last_mut() {
                Some(test) => test.checks.push(id.trim().to_string()),
                None => file_checks.push(id.trim().to_string()),
            }
        } else if let Some(name) = line.strip_prefix("test:") {
            named.push(TextTest {
                name: format!("{stem}.{}", name.trim()),
                checks: Vec::new(),
                outcome: "failed".to_string(),
                duration_ms: 1,
            });
        } else if let Some(outcome) = line.strip_prefix("outcome:") {
            match named.last_mut() {
                Some(test) => test.outcome = outcome.trim().to_string(),
                None => file_outcome = Some(outcome.trim().to_string()),
            }
        } else if let Some(duration) = line.strip_prefix("duration:") {
            let duration = duration.trim().parse().unwrap_or(1);
            match named.last_mut() {
                Some(test) => test.duration_ms = duration,
                None => file_duration = Some(duration),
            }
        } else if line == "kind: helper" {
            is_helper = true;
        }
    }

    if is_helper {
        return Vec::new();
    }
    if named.is_empty() {
        // A missing outcome line reads as a failure, so a malformed test can never pass silently.
        return vec![TextTest {
            name: stem,
            checks: file_checks,
            outcome: file_outcome.unwrap_or_else(|| "failed".to_string()),
            duration_ms: file_duration.unwrap_or(1),
        }];
    }
    for test in &mut named {
        let mut checks = file_checks.clone();
        checks.append(&mut test.checks);
        test.checks = checks;
    }
    named
}

fn emit(report: &Value) {
    println!("{report}");
    std::io::stdout().flush().unwrap();
    output_checkpoint("exit", false);
}

fn log(entry: Value) {
    let Ok(path) = std::env::var("APPARATUS_LOG") else {
        return;
    };
    let mut entry = entry;
    entry["pid"] = json!(std::process::id());
    entry["time_ns"] = json!(azimuth_tests::process::monotonic_ns());
    entry["argv0"] = Value::String(std::env::args().next().unwrap_or_default());
    entry["cwd"] = Value::String(
        std::env::current_dir()
            .map(|dir| dir.display().to_string())
            .unwrap_or_default(),
    );
    entry["mark"] = std::env::var("APPARATUS_MARK")
        .map(Value::String)
        .unwrap_or(Value::Null);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("the apparatus log is writable");
    file.write_all(format!("{entry}\n").as_bytes())
        .expect("the apparatus log entry is written");
}

fn output_folder() -> Option<PathBuf> {
    std::env::var_os("TEXT_ADAPTER_GATE_DIR")
        .map(|root| PathBuf::from(root).join(std::env::current_dir().unwrap().file_name().unwrap()))
}
fn output_checkpoint(phase: &str, payload: bool) {
    let Some(folder) = output_folder() else {
        return;
    };
    fs::create_dir_all(&folder).unwrap();
    if payload {
        let suite = folder.file_name().unwrap().to_str().unwrap();
        eprintln!("AZIMUTH-OUTPUT-{suite}-{phase}-731");
        std::io::stderr().flush().unwrap();
    }
    let ready = folder.join(format!("{phase}.ready"));
    let temporary = ready.with_extension("tmp");
    fs::write(&temporary, std::process::id().to_string()).unwrap();
    fs::rename(temporary, ready).unwrap();
    while !folder.join(format!("{phase}.release")).exists() {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
