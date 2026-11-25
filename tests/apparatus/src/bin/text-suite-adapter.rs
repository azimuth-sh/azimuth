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
//! The directives of a launch with contexts. `when <key>=<value>` or `when none` may stand
//! before any directive below or above, and the directive then applies only to a launch whose
//! request condition states that value, or states nothing. Without `when` it applies to every launch.
//!
//! - `adapter: fails <test>` makes that test, by its full name, fail in the launch.
//! - `adapter: runner` starts a child process that records the `AZIMUTH_CONTEXT` it inherited,
//!   as a runner inherits it from its adapter.
//! - `adapter: components applied` writes `components` in the report: for each component of the
//!   request, a `start` read and a `stop` read of a report as a faithful library would give it, with
//!   every key of the launch's condition applied by Azimuth. A report with no such directive and no
//!   `component` directive has no `components`, as the report of an adapter that predates them.
//! - `adapter: maps <component> <key>[=<configured>]...` sets the keys the component maps. A key
//!   the condition states is applied, any other is read from configuration, before Azimuth, with
//!   the text given, or absent when none is given. Without `maps` a component maps the condition's keys.
//! - `adapter: component <name> none` leaves the component out of `components`.
//! - `adapter: component <name> answered` gives it the entry `error: "something else answers there"`
//!   with no reads, and reports every test as `not_run`.
//! - `adapter: component <name> phases <phase>...` sets the phases of the component's reads, in
//!   order, in place of `start stop`. A host inside the test process gives `built disposing`, and
//!   several hosts give `built disposing built disposing`. Each read is as faithful as a `start`
//!   read, and the first of each phase is dated like the first read of a pair.
//! - `adapter: component <name> <selector> <change>` changes what a read gives. The selector is a
//!   phase (`start`, `stop`, `built`, `disposing`, all the reads of that phase), `both` (the reads
//!   of `start` and `stop`), `all` (every read), or `read <n>` (the n-th read of the sequence,
//!   counted from 1). A change is
//!   `skip` (no read of that phase), `error <text>`, `malformed` (the document is a text),
//!   `incomplete` (no `context`), `applying not-possible`, `name <other>`, `read-time <text>`,
//!   `value <key> <text>`, `absent <key>` (null value and null source), `drop <key>` (no such key),
//!   or `source <key> <before-azimuth|azimuth|after-azimuth> <source name>`.
//!
//! Five environment variables reach what the files cannot hold without moving the fingerprint:
//! `TEXT_ADAPTER_CONFIGURED` is a list `<key>=<text>,...` of what configuration gives each key that
//! the condition does not state and a component maps, as the machine's own configuration would,
//! `TEXT_ADAPTER_READ_TIME` is the time every read of a component gives, as another moment of the
//! same run would, `TEXT_ADAPTER_VERSION` overrides the version the adapter reports, `TEXT_ADAPTER_FAIL` names
//! tests, separated by commas, that fail whatever their file says, as an unstable test or a fault
//! of the machine would, and `APPARATUS_LOG` names a file outside the repository where each start
//! is recorded for the tests to inspect, with the value of `APPARATUS_MARK`, so a test can see
//! that the core passed its environment on.

use std::collections::{BTreeMap, BTreeSet};
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

#[derive(Default, Clone)]
struct ReadPlan {
    skip: bool,
    error: Option<String>,
    malformed: bool,
    incomplete: bool,
    applying_off: bool,
    name: Option<String>,
    read_time: Option<String>,
    values: BTreeMap<String, String>,
    absent: BTreeSet<String>,
    dropped: BTreeSet<String>,
    sources: BTreeMap<String, (String, String)>,
}

#[derive(Default, Clone)]
struct ComponentPlan {
    maps: Option<Vec<(String, Option<String>)>>,
    omitted: bool,
    answered: bool,
    phases: Option<Vec<String>>,
    changes: Vec<(String, Vec<String>)>,
}

#[derive(Default)]
struct Directives {
    components_applied: bool,
    plans: BTreeMap<String, ComponentPlan>,
    runner: bool,
    fails: BTreeSet<String>,
    crash: bool,
    garbage: bool,
    two_reports: bool,
    build_fails: bool,
    extra_listed: Vec<(String, String)>,
    edits: Vec<String>,
    sleep_ms: u64,
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--runner") {
        log(json!({ "program": "text-suite-runner", "event": "started" }));
        return;
    }
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
    log(json!({"program": NAME, "event": "exited"}));
}

fn run(request: &Value, adapter: Value, args: Vec<String>) {
    // Written first, so that a launch which is stopped before it reports still shows its request.
    log(json!({ "program": NAME, "event": "requested", "asked": request }));
    let key = request["marker_key"].as_str().unwrap_or("Check");
    let asked: BTreeSet<String> = request["checks"]
        .as_array()
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let condition: BTreeMap<String, String> = request["condition"]
        .as_object()
        .map(|entries| {
            entries
                .iter()
                .map(|(key, value)| (key.clone(), value.as_str().unwrap_or_default().to_string()))
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
        read_directives(&text, &mut directives, &condition);
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

    let mut forced_failures: BTreeSet<String> = std::env::var("TEXT_ADAPTER_FAIL")
        .unwrap_or_default()
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect();
    forced_failures.extend(directives.fails.iter().cloned());
    let answered = directives.plans.values().any(|plan| plan.answered);
    if answered {
        for test in &mut tests {
            test.outcome = "not_run".to_string();
        }
    }
    if directives.runner {
        std::process::Command::new(std::env::current_exe().expect("the adapter knows its path"))
            .arg("--runner")
            .status()
            .expect("the runner starts");
    }
    for test in &mut tests {
        if forced_failures.contains(&test.name) {
            test.outcome = "failed".to_string();
        }
    }

    let mut ran = Vec::new();
    let mut reported = Vec::new();
    for test in tests.iter().filter(|test| !test.checks.is_empty()) {
        let wanted = !answered && test.checks.iter().any(|check| asked.contains(check));
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
    if let Ok(bytes) = std::env::var("TEXT_ADAPTER_BULK_BYTES") {
        let mut remaining: usize = bytes.parse().unwrap();
        let block = vec![b'X'; 65536];
        while remaining != 0 {
            let count = remaining.min(block.len());
            std::io::stderr().write_all(&block[..count]).unwrap();
            remaining -= count;
        }
    }
    if let Ok(tail) = std::env::var("TEXT_ADAPTER_TAIL") {
        std::io::stderr().write_all(tail.as_bytes()).unwrap();
    }
    std::io::stderr().flush().unwrap();
    let mut report = json!({ "format": "adapter-report-1", "adapter": adapter, "build": "ok", "tests": reported });
    if let Some(components) = component_entries(request, &directives, &condition) {
        report["components"] = Value::Array(components);
    }
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

fn read_directives(text: &str, directives: &mut Directives, condition: &BTreeMap<String, String>) {
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("adapter:") else {
            continue;
        };
        let mut words: Vec<&str> = rest.split_whitespace().collect();
        if words.first() == Some(&"when") {
            let selector = words.get(1).copied().unwrap_or_default();
            words.drain(..2.min(words.len()));
            let matches = match selector.split_once('=') {
                Some((key, value)) => condition.get(key).is_some_and(|stated| stated == value),
                None => selector == "none" && condition.is_empty(),
            };
            if !matches {
                continue;
            }
        }
        match words.as_slice() {
            ["crash"] => directives.crash = true,
            ["garbage"] => directives.garbage = true,
            ["two-reports"] => directives.two_reports = true,
            ["build-fails"] => directives.build_fails = true,
            ["runner"] => directives.runner = true,
            ["fails", name] => {
                directives.fails.insert(name.to_string());
            }
            ["components", "applied"] => directives.components_applied = true,
            ["maps", component, keys @ ..] => {
                let maps = keys
                    .iter()
                    .map(|key| match key.split_once('=') {
                        Some((key, text)) => (key.to_string(), Some(text.to_string())),
                        None => (key.to_string(), None),
                    })
                    .collect();
                directives.plans.entry(component.to_string()).or_default().maps = Some(maps);
            }
            ["component", name, "none"] => {
                directives.plans.entry(name.to_string()).or_default().omitted = true
            }
            ["component", name, "answered"] => {
                directives.plans.entry(name.to_string()).or_default().answered = true
            }
            ["component", name, "phases", phases @ ..] => {
                directives.plans.entry(name.to_string()).or_default().phases =
                    Some(phases.iter().map(|phase| phase.to_string()).collect())
            }
            ["component", name, "read", number, change @ ..] => directives
                .plans
                .entry(name.to_string())
                .or_default()
                .changes
                .push((format!("read {number}"), change.iter().map(|word| word.to_string()).collect())),
            ["component", name, selector @ ("start" | "stop" | "built" | "disposing" | "both" | "all"), change @ ..] => {
                directives
                    .plans
                    .entry(name.to_string())
                    .or_default()
                    .changes
                    .push((selector.to_string(), change.iter().map(|word| word.to_string()).collect()))
            }
            ["lists", name, check] => directives
                .extra_listed
                .push((name.to_string(), check.to_string())),
            ["edits", path] => directives.edits.push(path.to_string()),
            ["sleeps", ms] => {
                directives.sleep_ms += ms.parse::<u64>().expect("sleeps names whole milliseconds")
            }
            _ => eprintln!("{NAME}: unknown directive {line:?}"),
        }
    }
}

fn change_read(read: &mut ReadPlan, change: &[&str]) {
    match change {
        ["skip"] => read.skip = true,
        ["error", text @ ..] => read.error = Some(text.join(" ")),
        ["malformed"] => read.malformed = true,
        ["incomplete"] => read.incomplete = true,
        ["applying", "not-possible"] => read.applying_off = true,
        ["name", other] => read.name = Some(other.to_string()),
        ["read-time", time] => read.read_time = Some(time.to_string()),
        ["value", key, text] => {
            read.values.insert(key.to_string(), text.to_string());
        }
        ["absent", key] => {
            read.absent.insert(key.to_string());
        }
        ["drop", key] => {
            read.dropped.insert(key.to_string());
        }
        ["source", key, precedence, name] => {
            read.sources
                .insert(key.to_string(), (precedence.replace('-', " "), name.to_string()));
        }
        _ => eprintln!("{NAME}: unknown change of a read {change:?}"),
    }
}

fn component_entries(
    request: &Value,
    directives: &Directives,
    condition: &BTreeMap<String, String>,
) -> Option<Vec<Value>> {
    if !directives.components_applied && directives.plans.is_empty() {
        return None;
    }
    let mut entries = Vec::new();
    for component in request["components"].as_array().cloned().unwrap_or_default() {
        let name = component["name"].as_str().unwrap_or_default().to_string();
        let plan = directives.plans.get(&name).cloned().unwrap_or_default();
        if plan.omitted {
            continue;
        }
        if plan.answered {
            entries.push(json!({ "name": name, "error": "something else answers there" }));
            continue;
        }
        let mut reads = Vec::new();
        let phases = plan.phases.clone().unwrap_or_else(|| vec!["start".to_string(), "stop".to_string()]);
        for (index, phase) in phases.iter().enumerate() {
            let closing = matches!(phase.as_str(), "stop" | "disposing");
            let time = if closing { "2026-11-24T10:16:30Z" } else { "2026-11-24T10:15:00Z" };
            let mut read = ReadPlan::default();
            for (selector, change) in &plan.changes {
                let selected = match selector.as_str() {
                    "all" => true,
                    "both" => matches!(phase.as_str(), "start" | "stop"),
                    other => other == phase || other == format!("read {}", index + 1),
                };
                if selected {
                    let words: Vec<&str> = change.iter().map(String::as_str).collect();
                    change_read(&mut read, &words);
                }
            }
            if read.skip {
                continue;
            }
            if let Some(error) = &read.error {
                reads.push(json!({ "phase": phase, "error": error }));
                continue;
            }
            reads.push(json!({ "phase": phase, "report": component_report(&name, &plan, &read, time, condition) }));
        }
        entries.push(json!({ "name": name, "reads": reads }));
    }
    Some(entries)
}

fn component_report(
    name: &str,
    plan: &ComponentPlan,
    read: &ReadPlan,
    default_time: &str,
    condition: &BTreeMap<String, String>,
) -> Value {
    if read.malformed {
        return json!("this is not the report of a component");
    }
    let time = read
        .read_time
        .clone()
        .or_else(|| std::env::var("TEXT_ADAPTER_READ_TIME").ok())
        .unwrap_or_else(|| default_time.to_string());
    let machine: BTreeMap<String, String> = std::env::var("TEXT_ADAPTER_CONFIGURED")
        .unwrap_or_default()
        .split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, text)| (key.to_string(), text.to_string()))
        .collect();
    let applied = |text: &str| {
        json!({ "value": text, "source": { "name": "azimuth", "precedence": "azimuth" }, "read": time })
    };
    let mut context = serde_json::Map::new();
    match &plan.maps {
        Some(keys) => {
            for (key, configured) in keys {
                let configured = configured.clone().or_else(|| machine.get(key).cloned());
                let entry = match (condition.get(key), &configured) {
                    (Some(stated), _) => applied(stated),
                    (None, Some(text)) => json!({
                        "value": text,
                        "source": { "name": "configuration", "precedence": "before azimuth" },
                        "read": time,
                    }),
                    (None, None) => json!({ "value": null, "source": null, "read": time }),
                };
                context.insert(key.clone(), entry);
            }
        }
        None => {
            for (key, stated) in condition {
                context.insert(key.clone(), applied(stated));
            }
        }
    }
    for (key, text) in &read.values {
        let source = context
            .get(key)
            .map(|entry| entry["source"].clone())
            .filter(|source| !source.is_null())
            .unwrap_or_else(|| json!({ "name": "azimuth", "precedence": "azimuth" }));
        context.insert(key.clone(), json!({ "value": text, "source": source, "read": time }));
    }
    for (key, (precedence, source)) in &read.sources {
        let value = context.get(key).map(|entry| entry["value"].clone()).unwrap_or(Value::Null);
        context.insert(
            key.clone(),
            json!({ "value": value, "source": { "name": source, "precedence": precedence }, "read": time }),
        );
    }
    for key in &read.absent {
        context.insert(key.clone(), json!({ "value": null, "source": null, "read": time }));
    }
    for key in &read.dropped {
        context.remove(key);
    }
    let mut document = json!({
        "format": 1,
        "component": read.name.clone().unwrap_or_else(|| name.to_string()),
        "applying": if read.applying_off { "not possible" } else { "possible" },
    });
    if !read.incomplete {
        document["context"] = Value::Object(context);
    }
    document
}

fn read_tests(relative: &Path, text: &str, key: &str) -> Vec<TextTest> {
    let marker_prefix = format!("# {key}:");
    let stem = relative
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/");

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
    entry["azimuth_context"] = std::env::var("AZIMUTH_CONTEXT")
        .map(Value::String)
        .unwrap_or(Value::Null);
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
        let tag = std::env::var("TEXT_ADAPTER_OUTPUT_TAG").unwrap_or_else(|_| "731".to_string());
        eprintln!("AZIMUTH-OUTPUT-{suite}-{phase}-{tag}");
        std::io::stderr().flush().unwrap();
    }
    let ready = folder.join(format!("{phase}.ready"));
    let temporary = ready.with_extension("tmp");
    fs::write(&temporary, std::process::id().to_string()).unwrap();
    fs::rename(temporary, ready).unwrap();
    while !folder.join(format!("{phase}.release")).exists() {
        if std::env::args()
            .nth(1)
            .is_some_and(|token| Path::new(&token).with_extension("stop").exists())
        {
            std::process::exit(0);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
