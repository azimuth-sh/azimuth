//! The behaviors of the Node context library, `docs/increments/contexts-library-node.md`.
//!
//! The fixture is `website` of the invented payouts product, `tests/sample/payouts/website/`, written
//! the way the design's example shows. Each test copies it into a folder of its own, installs the
//! library into that copy from `libraries/node` by local path with an npm cache of its own, and
//! starts the fixture in processes of its own, driven by `AZIMUTH_CONTEXT`, `PAYOUTS_TEST_STACK` and
//! the fixture's `FIXTURE_*` variables. Until `libraries/node/` exists every test here fails at the
//! install. The tests need Node and npm and run only with `cargo test --features playwright`.
//!
//! The choices the design's example leaves open: the package is ESM and is imported by name,
//! `read` and `apply` are synchronous, `report()` returns a string, and a malformed `AZIMUTH_CONTEXT`
//! throws from `registerAzimuthContext` with a message that names the variable.

use azimuth_tests::*;
use serde_json::Value;
use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

const STACK: (&str, &str) = ("PAYOUTS_TEST_STACK", "1");
const INSTANT: &str = "payouts.instant";
const REGION: &str = "payouts.region";
const INSTANT_NAME: &str = "FEATURE_INSTANT_PAYOUTS";
const LAUNCH_INSTANT: (&str, &str) = ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

/// The fixture and the library installed into it, in a folder of this test's own.
struct Work {
    dir: TempDir,
}

impl Work {
    fn entry(&self) -> PathBuf {
        self.dir.path().join("website/server.js")
    }

    fn library_folder(&self) -> PathBuf {
        self.dir.path().join("website/node_modules/@azimuth-sh/context").canonicalize().unwrap_or_else(|error| {
            panic!("the install left no @azimuth-sh/context: {error}")
        })
    }
}

fn npm_install(dir: &Path, cache: &Path) -> Output {
    let home = cache.parent().unwrap();
    std::fs::write(home.join("user.npmrc"), "").unwrap();
    std::fs::write(home.join("global.npmrc"), "").unwrap();
    let mut command = Command::new("npm");
    command
        .current_dir(dir)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("npm_config_userconfig", home.join("user.npmrc"))
        .env("npm_config_globalconfig", home.join("global.npmrc"))
        .env("npm_config_update_notifier", "false")
        .env("npm_config_cache", cache)
        .env("npm_config_fetch_retries", "0")
        .env("npm_config_audit", "false")
        .env("npm_config_fund", "false")
        .args(["install", "--no-package-lock"]);
    run_with_time_limit(command, Duration::from_secs(180), None)
}

fn install_fixture() -> Work {
    let root = repo_root();
    let dir = tempfile::Builder::new().prefix("context-library-node-").tempdir().unwrap();
    let website = dir.path().join("website");
    std::fs::create_dir_all(&website).unwrap();
    let sample = root.join("tests/sample/payouts/website");
    std::fs::copy(sample.join("server.js"), website.join("server.js")).unwrap();
    let mut manifest: Value = serde_json::from_str(&std::fs::read_to_string(sample.join("package.json")).unwrap()).unwrap();
    manifest["dependencies"]["@azimuth-sh/context"] = Value::String(format!("file:{}", root.join("libraries/node").display()));
    std::fs::write(website.join("package.json"), serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    let cache = dir.path().join("npm-home/cache");
    std::fs::create_dir_all(&cache).unwrap();
    let output = npm_install(&website, &cache);
    output.expect_status(0);
    let installed = website.join("node_modules/@azimuth-sh/context/package.json");
    assert!(installed.is_file(), "the install left no @azimuth-sh/context from libraries/node\n{}", output.text());
    Work { dir }
}

thread_local! {
    // A test runs on a thread of its own, so each test installs once and owns its npm cache.
    static WORK: RefCell<Option<Rc>> = const { RefCell::new(None) };
}

type Rc = std::rc::Rc<Work>;

fn work() -> Rc {
    WORK.with(|slot| slot.borrow_mut().get_or_insert_with(|| std::rc::Rc::new(install_fixture())).clone())
}

fn scratch_folder() -> TempDir {
    tempfile::Builder::new().prefix("context-library-node-run-").tempdir().unwrap()
}

/// A command for the fixture with the variables of an earlier test taken out of its environment.
fn fixture_command(env: &[(&str, &str)], dir: &Path) -> Command {
    let mut command = Command::new("node");
    command.arg(work().entry()).current_dir(dir);
    for (name, _) in std::env::vars() {
        if name.starts_with("FIXTURE_")
            || name.starts_with("FEATURE_")
            || name == "AZIMUTH_CONTEXT"
            || name == "PAYOUTS_TEST_STACK"
            || name == "WEBSITE_TEST_STACK"
        {
            command.env_remove(name);
        }
    }
    for (name, value) in env {
        command.env(name, value);
    }
    command
}

fn run_to_end(env: &[(&str, &str)]) -> Output {
    let dir = scratch_folder();
    run_with_time_limit(fixture_command(env, dir.path()), Duration::from_secs(60), None)
}

fn inspect() -> Value {
    let output = run_to_end(&[("FIXTURE_MODE", "inspect")]);
    output.expect_status(0);
    let line = output.stdout.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("");
    serde_json::from_str(line).unwrap_or_else(|error| panic!("inspection gave no JSON ({error}):\n{}", output.text()))
}

fn strings(value: &Value) -> Vec<String> {
    value.as_array().unwrap().iter().map(|item| item.as_str().unwrap().to_string()).collect()
}

struct Website {
    child: Child,
    port: u16,
    done: Arc<AtomicBool>,
    _scratch: TempDir,
}

impl Drop for Website {
    fn drop(&mut self) {
        self.done.store(true, Ordering::SeqCst);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Starts `website` and waits until it says where it listens. The fixture listens on a port of its
/// own choosing on the loopback, which is the fixture's door and not the library's.
fn start(env: &[(&str, &str)]) -> Website {
    start_in(env, None)
}

fn start_in(env: &[(&str, &str)], dir: Option<&Path>) -> Website {
    let scratch = scratch_folder();
    let mut command = fixture_command(env, dir.unwrap_or(scratch.path()));
    command.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
    let mut child = command.spawn().expect("start the fixture");
    let stderr = Arc::new(Mutex::new(String::new()));
    let mut child_stderr = child.stderr.take().unwrap();
    let collected = stderr.clone();
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = child_stderr.read_to_string(&mut text);
        collected.lock().unwrap().push_str(&text);
    });
    let done = Arc::new(AtomicBool::new(false));
    let pid = child.id().to_string();
    let watched = done.clone();
    std::thread::spawn(move || {
        for _ in 0..120 {
            std::thread::sleep(Duration::from_millis(500));
            if watched.load(Ordering::SeqCst) {
                return;
            }
        }
        let _ = Command::new("kill").arg(&pid).status();
    });
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut seen = String::new();
    let port = loop {
        let mut line = String::new();
        let read = stdout.read_line(&mut line).unwrap();
        if read == 0 {
            let _ = child.wait();
            panic!("the fixture ended before it listened.\nstdout:\n{seen}\nstderr:\n{}", stderr.lock().unwrap());
        }
        seen.push_str(&line);
        if let Some(url) = line.trim().strip_prefix("listening ") {
            break url.rsplit(':').next().unwrap().trim_end_matches('/').parse::<u16>().unwrap();
        }
    };
    std::thread::spawn(move || {
        let mut sink = String::new();
        let _ = stdout.read_to_string(&mut sink);
    });
    Website { child, port, done, _scratch: scratch }
}

fn decode_chunked(body: &str) -> String {
    let mut rest = body;
    let mut decoded = String::new();
    while let Some((size, after)) = rest.split_once("\r\n") {
        let size = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
        if size == 0 {
            break;
        }
        decoded.push_str(&after[..size]);
        rest = &after[size + 2..];
    }
    decoded
}

impl Website {
    fn get(&self, path: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
        write!(stream, "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let mut raw = String::new();
        stream.read_to_string(&mut raw).unwrap();
        let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
        let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
        let body = if head.to_lowercase().contains("transfer-encoding: chunked") {
            decode_chunked(body)
        } else {
            body.to_string()
        };
        (status, body)
    }

    fn ok(&self, path: &str) -> Value {
        let (status, body) = self.get(path);
        assert_eq!(status, 200, "GET {path}: {body}");
        serde_json::from_str(&body).unwrap_or_else(|error| panic!("GET {path} gave no JSON ({error}): {body}"))
    }

    /// The report through the callback.
    fn report(&self) -> Value {
        self.ok("/fixture/report")
    }

    /// The report through the component's declared `report` path.
    fn declared_report(&self) -> Value {
        self.ok("/internal/azimuth")
    }

    /// The text of a name in `process.env` of the running component.
    fn setting(&self, name: &str) -> Option<String> {
        self.ok(&format!("/fixture/envvar?name={name}"))["value"].as_str().map(str::to_string)
    }

    /// Writes a name in `process.env` of the running component, as a later writer would.
    fn write_setting(&self, name: &str, value: &str) {
        self.ok(&format!("/fixture/env?name={name}&value={value}"));
    }

    fn state(&self) -> Value {
        self.ok("/fixture/state")
    }

    fn region(&self) -> Value {
        self.ok("/fixture/region")
    }
}

fn key<'a>(report: &'a Value, name: &str) -> &'a Value {
    assert!(report["context"].get(name).is_some(), "the report has no key {name}: {report}");
    &report["context"][name]
}

fn value_of(report: &Value, name: &str) -> Option<String> {
    key(report, name)["value"].as_str().map(str::to_string)
}

fn precedence_of(report: &Value, name: &str) -> String {
    key(report, name)["source"]["precedence"].as_str().unwrap_or_else(|| panic!("no source precedence: {report}")).to_string()
}

fn context_keys(report: &Value) -> Vec<String> {
    let mut keys: Vec<String> = report["context"].as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

/// The report without the times it was read, which differ between two calls.
fn without_times(report: &Value) -> Value {
    let mut copy = report.clone();
    for (_, entry) in copy["context"].as_object_mut().unwrap().iter_mut() {
        entry.as_object_mut().unwrap().remove("read");
    }
    copy
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Seconds since the epoch of an ISO 8601 time given in UTC as `...Z` or `...+00:00`.
fn utc_seconds(text: &str) -> f64 {
    let body = text
        .strip_suffix('Z')
        .or_else(|| text.strip_suffix("+00:00"))
        .unwrap_or_else(|| panic!("{text} is not marked as UTC"));
    let (date, time) = body.split_once('T').unwrap_or_else(|| panic!("{text} has no time"));
    let mut date = date.split('-').map(|part| part.parse::<i64>().unwrap());
    let (year, month, day) = (date.next().unwrap(), date.next().unwrap(), date.next().unwrap());
    let mut time = time.split(':');
    let hours: f64 = time.next().unwrap().parse().unwrap();
    let minutes: f64 = time.next().unwrap().parse().unwrap();
    let seconds: f64 = time.next().unwrap().parse().unwrap();
    days_from_civil(year, month, day) as f64 * 86_400.0 + hours * 3600.0 + minutes * 60.0 + seconds
}

fn now_seconds() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()
}

/// The text of every script the installed library ships.
fn library_sources() -> Vec<(PathBuf, String)> {
    fn walk(folder: &Path, found: &mut Vec<(PathBuf, String)>) {
        for entry in std::fs::read_dir(folder).unwrap().flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if name != "node_modules" {
                    walk(&path, found);
                }
            } else if ["js", "mjs", "cjs"].iter().any(|extension| path.extension().is_some_and(|actual| actual == *extension)) {
                found.push((path.clone(), std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    let mut found = Vec::new();
    walk(&work().library_folder(), &mut found);
    assert!(!found.is_empty(), "the installed library ships no script");
    found
}

/// The module names the library's scripts import.
fn imported_modules() -> Vec<String> {
    let mut modules = Vec::new();
    for (_, text) in library_sources() {
        for marker in ["from ", "require(", "import(", "import "] {
            let mut rest = text.as_str();
            while let Some(at) = rest.find(marker) {
                rest = &rest[at + marker.len()..];
                let rest_trimmed = rest.trim_start();
                if let Some(quote) = rest_trimmed.chars().next().filter(|c| matches!(c, '"' | '\'' | '`')) {
                    if let Some(end) = rest_trimmed[1..].find(quote) {
                        modules.push(rest_trimmed[1..1 + end].to_string());
                    }
                }
            }
        }
    }
    modules
}

fn library_manifest() -> Value {
    serde_json::from_str(&std::fs::read_to_string(work().library_folder().join("package.json")).unwrap()).unwrap()
}

fn declared_dependencies() -> Vec<String> {
    let manifest = library_manifest();
    ["dependencies", "peerDependencies", "optionalDependencies"]
        .iter()
        .filter_map(|section| manifest.get(*section).and_then(Value::as_object))
        .flat_map(|section| section.keys().cloned())
        .collect()
}

fn network_modules() -> Vec<String> {
    let network = ["http", "https", "http2", "net", "tls", "dgram", "dns", "child_process", "cluster", "worker_threads", "undici", "node-fetch", "axios", "got", "ws"];
    let mut found: Vec<String> = imported_modules()
        .into_iter()
        .chain(declared_dependencies())
        .filter(|name| {
            let bare = name.strip_prefix("node:").unwrap_or(name);
            let bare = bare.split('/').next().unwrap_or(bare);
            network.contains(&bare)
        })
        .collect();
    for (path, text) in library_sources() {
        for call in ["fetch(", "XMLHttpRequest", "WebSocket", "EventSource"] {
            if text.contains(call) {
                found.push(format!("{call} in {}", path.display()));
            }
        }
    }
    found
}

/// A component that registered and builds its report in a process with no web server, and the
/// sockets that process holds while the library is registered and has reported.
fn sockets_of_a_registered_component() -> String {
    let dir = scratch_folder();
    let mut command = fixture_command(&[STACK, ("FIXTURE_MODE", "console"), LAUNCH_INSTANT], dir.path());
    command.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::piped());
    let mut child = command.spawn().expect("start the fixture");
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    stdout.read_line(&mut first).unwrap();
    assert!(first.contains("\"component\""), "the fixture gave no report: {first}");
    let listing = Command::new("lsof")
        .args(["-nP", "-a", "-p", &child.id().to_string(), "-i"])
        .output()
        .expect("lsof lists the sockets of the process");
    drop(child.stdin.take());
    let _ = child.wait();
    String::from_utf8_lossy(&listing.stdout).to_string()
}

/// A folder the component starts in that holds the files a project of the invented payouts product
/// would, none of which the library may read.
fn decoy_project() -> TempDir {
    let dir = scratch_folder();
    std::fs::create_dir_all(dir.path().join("azimuth")).unwrap();
    std::fs::write(
        dir.path().join("azimuth/project.toml"),
        "cli = \"0.2.0-alpha.2\"\n\n[repository]\nname = \"payouts\"\ntarget = \"main\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("azimuth/execution-context.toml"),
        "[payouts.declared]\ntype = \"text\"\ncomponents = [\"website\"]\n\n[payouts.instant]\ntype = \"text\"\ncomponents = [\"website\"]\n",
    )
    .unwrap();
    std::fs::write(dir.path().join(".env"), "FEATURE_DECLARED_PAYOUTS=from-dotenv\nFEATURE_INSTANT_PAYOUTS=from-dotenv\n").unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"name":"decoy","azimuth":{"map":{"payouts.declared":"FEATURE_DECLARED_PAYOUTS"}}}"#,
    )
    .unwrap();
    dir
}

// ---------------------------------------------------------------- The package

// behavior-contexts-library-node: The library is named `@azimuth-sh/context` for Node
#[test]
fn the_library_is_named_azimuth_sh_context() {
    assert_eq!(library_manifest()["name"], "@azimuth-sh/context");
    let inspection = inspect();
    assert!(strings(&inspection["exports"]).contains(&"registerAzimuthContext".to_string()), "{inspection}");
}

// behavior-contexts-library-node: The library speaks of keys, sources, values, applying and reports, and names no testing
#[test]
fn the_library_speaks_of_keys_sources_values_applying_and_reports_and_names_no_testing() {
    let inspection = inspect();
    let exports = strings(&inspection["exports"]);
    let properties = strings(&inspection["contextProperties"]);
    assert!(exports.contains(&"registerAzimuthContext".to_string()), "{exports:?}");
    assert!(properties.contains(&"report".to_string()), "the context has no report: {properties:?}");
    assert_eq!(inspection["reportType"], "string");
    let testing = ["test", "mock", "fake", "stub", "jest", "mocha", "vitest", "ava", "playwright", "fixture"];
    let mut spoken = exports;
    spoken.extend(properties);
    spoken.extend(imported_modules());
    spoken.extend(declared_dependencies());
    let spoken_of_testing: Vec<&String> = spoken
        .iter()
        .filter(|name| name.as_str() != "constructor" && testing.iter().any(|word| name.to_lowercase().contains(word)))
        .collect();
    assert!(spoken_of_testing.is_empty(), "the library names testing: {spoken_of_testing:?}");
}

// behavior-contexts-library-node: The library opens no port
#[test]
fn the_library_opens_no_port() {
    let listing = sockets_of_a_registered_component();
    assert!(listing.trim().is_empty(), "a registered component with no web server holds sockets:\n{listing}");
    let referencing = network_modules();
    assert!(referencing.is_empty(), "the library reaches networking: {referencing:?}");
}

// behavior-contexts-library-node: The library makes no connection out
#[test]
fn the_library_makes_no_connection_out() {
    let listing = sockets_of_a_registered_component();
    assert!(!listing.contains("ESTABLISHED") && !listing.contains("->"), "the process holds a connection:\n{listing}");
    let referencing = network_modules();
    assert!(referencing.is_empty(), "the library reaches networking: {referencing:?}");
}

// behavior-contexts-library-node: The library exposes the report at no path of its own; the component does, at its declared `report` path
#[test]
fn the_library_exposes_the_report_at_no_path_of_its_own() {
    let silent = start(&[STACK, ("FIXTURE_NO_REPORT_PATH", "1")]);
    for path in ["/", "/internal/azimuth", "/azimuth", "/report", "/azimuth/report", "/.well-known/azimuth", "/internal/azimuth/report", "/_azimuth", "/context"] {
        assert_eq!(silent.get(path).0, 404, "{path} answers though the component declares no path");
    }
    let declaring = start(&[STACK]);
    let (status, body) = declaring.get("/internal/azimuth");
    assert_eq!(status, 200);
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["component"], "website");
}

// behavior-contexts-library-node: The same report is used in a launch and in a deployment
#[test]
fn the_same_report_is_used_in_a_launch_and_in_a_deployment() {
    let launch = start(&[STACK, ("AZIMUTH_CONTEXT", "{}")]).report();
    let deployment = start(&[STACK]).report();
    assert_eq!(without_times(&launch), without_times(&deployment));
    let launch_with_value = start(&[STACK, LAUNCH_INSTANT]).report();
    assert_eq!(
        launch_with_value.as_object().unwrap().keys().collect::<Vec<_>>(),
        deployment.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    assert_eq!(context_keys(&launch_with_value), context_keys(&deployment));
}

// ---------------------------------------------------------------- Registration

// behavior-contexts-library-node: A component registers its name
#[test]
fn a_component_registers_its_name() {
    let website = start(&[STACK, ("FIXTURE_COMPONENT", "payouts-website")]);
    assert_eq!(website.declared_report()["component"], "payouts-website");
}

// behavior-contexts-library-node: A component registers the condition under which applying is possible
#[test]
fn a_component_registers_the_condition_under_which_applying_is_possible() {
    let holding = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(holding.state()["conditionCalls"], 1);
    assert_eq!(holding.report()["applying"], "possible");
    let not_holding = start(&[LAUNCH_INSTANT]);
    assert_eq!(not_holding.state()["conditionCalls"], 1);
    assert_eq!(not_holding.report()["applying"], "not possible");
}

// behavior-contexts-library-node: A component registers an explicit mapping of every key it uses to its own setting
#[test]
fn a_component_registers_an_explicit_mapping_of_every_key_it_uses_to_its_own_setting() {
    let website = start(&[STACK, ("FIXTURE_KEYS", "wide")]);
    assert_eq!(
        context_keys(&website.report()),
        ["payouts.instant", "payouts.region", "payouts.second", "payouts.side", "payouts.sideteam"]
    );
    let only_instant = start(&[STACK, ("FIXTURE_KEYS", "instant")]);
    assert_eq!(context_keys(&only_instant.report()), ["payouts.instant"]);
}

// behavior-contexts-library-node: A mapping reads the key's own setting
#[test]
fn a_mapping_reads_the_keys_own_setting() {
    let website = start(&[("FIXTURE_INSTANT_NAME", "FEATURE_OTHER"), ("FEATURE_OTHER", "other"), (INSTANT_NAME, "default")]);
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("other"));
}

// behavior-contexts-library-node: A mapping writes the key's own setting
#[test]
fn a_mapping_writes_the_keys_own_setting() {
    let website = start(&[STACK, ("FIXTURE_INSTANT_NAME", "FEATURE_OTHER"), LAUNCH_INSTANT]);
    assert_eq!(website.setting("FEATURE_OTHER").as_deref(), Some("true"));
    assert_eq!(website.setting(INSTANT_NAME), None);
}

// behavior-contexts-library-node: No rule derives a setting from a key; a key with no mapping has no setting
#[test]
fn no_rule_derives_a_setting_from_a_key_and_a_key_with_no_mapping_has_no_setting() {
    let website = start(&[
        STACK,
        ("AZIMUTH_CONTEXT", r#"{"payouts.refunds":"on","payouts.instant":"true"}"#),
        ("FEATURE_REFUNDS_PAYOUTS", "earlier"),
    ]);
    for derived in ["FEATURE_REFUNDS_PAYOUTS", "FEATURE_PAYOUTS_REFUNDS", "PAYOUTS_REFUNDS", "payouts.refunds", "FEATURE_REFUNDS", "REFUNDS"] {
        let expected = if derived == "FEATURE_REFUNDS_PAYOUTS" { Some("earlier") } else { None };
        assert_eq!(website.setting(derived).as_deref(), expected, "setting {derived}");
    }
    assert_eq!(context_keys(&website.report()), ["payouts.instant", "payouts.region"]);
}

// behavior-contexts-library-node: The library knows no types
#[test]
fn the_library_knows_no_types() {
    let website = start(&[
        STACK,
        (INSTANT_NAME, "true"),
        ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"eu-west","payouts.region":"TRUE"}"#),
    ]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("eu-west"));
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("eu-west"));
    assert_eq!(precedence_of(&report, INSTANT), "azimuth");
    assert_eq!(value_of(&report, REGION).as_deref(), Some("TRUE"));
}

// behavior-contexts-library-node: The library reads no project file
#[test]
fn the_library_reads_no_project_file() {
    let project = decoy_project();
    let website = start_in(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.declared":"x","payouts.instant":"true"}"#)], Some(project.path()));
    let report = website.report();
    assert_eq!(context_keys(&report), ["payouts.instant", "payouts.region"]);
    assert_eq!(website.setting("FEATURE_DECLARED_PAYOUTS"), None);
    assert_eq!(website.setting("payouts.declared"), None);
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    let reading = start_in(&[], Some(project.path()));
    assert_eq!(value_of(&reading.report(), INSTANT), None, "a file of the project's folder supplied a value");
}

// behavior-contexts-library-node: The library gives text values and interprets none; the core and server interpret them by `execution-context.toml`
#[test]
fn the_library_gives_text_values_and_interprets_none() {
    for text in ["TRUE", "false", "null", "0", "[1]", "{}", "1e3", "eu-west"] {
        let launch = serde_json::json!({ "payouts.instant": text, "payouts.region": text }).to_string();
        let website = start(&[STACK, ("AZIMUTH_CONTEXT", &launch)]);
        let report = website.report();
        assert_eq!(key(&report, INSTANT)["value"], text, "reported {text}");
        assert_eq!(key(&report, REGION)["value"], text, "reported {text} for a team's source");
        assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some(text), "applied {text}");
        assert_eq!(website.region()["lastApplied"], text, "handed {text} to a team's source");
    }
}

// ---------------------------------------------------------------- The condition and applying

// behavior-contexts-library-node: The component's own code gives the condition
#[test]
fn the_components_own_code_gives_the_condition() {
    assert_eq!(start(&[STACK, LAUNCH_INSTANT]).report()["applying"], "possible");
    assert_eq!(start(&[LAUNCH_INSTANT]).report()["applying"], "not possible");
    let other = start(&[("FIXTURE_CONDITION_VARIABLE", "WEBSITE_TEST_STACK"), ("WEBSITE_TEST_STACK", "1"), LAUNCH_INSTANT]);
    assert_eq!(other.report()["applying"], "possible");
}

// behavior-contexts-library-node: The library never guesses applying from an environment name
#[test]
fn the_library_never_guesses_applying_from_an_environment_name() {
    for name in ["development", "test", "testing", "staging", "ci"] {
        let website = start(&[("NODE_ENV", name), ("CI", "true"), LAUNCH_INSTANT]);
        assert_eq!(website.report()["applying"], "not possible", "environment {name}");
        assert_eq!(website.setting(INSTANT_NAME), None, "environment {name}");
    }
    let production = start(&[STACK, ("NODE_ENV", "production"), LAUNCH_INSTANT]);
    assert_eq!(production.report()["applying"], "possible");
}

// behavior-contexts-library-node: The condition is evaluated once, at start
#[test]
fn the_condition_is_evaluated_once_at_start() {
    let website = start(&[STACK, LAUNCH_INSTANT]);
    website.report();
    website.declared_report();
    website.report();
    assert_eq!(website.state()["conditionCalls"], 1);
}

// behavior-contexts-library-node: The condition is evaluated before applying
#[test]
fn the_condition_is_evaluated_before_applying() {
    let website = start(&[STACK, (INSTANT_NAME, "earlier"), LAUNCH_INSTANT]);
    assert_eq!(website.state()["seenAtCondition"], "earlier");
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
}

// behavior-contexts-library-node: A later change to the condition leaves `applying` as it was
#[test]
fn a_later_change_to_the_condition_leaves_applying_as_it_was() {
    let holding = start(&[STACK, LAUNCH_INSTANT]);
    holding.ok("/fixture/condition?value=0");
    assert_eq!(holding.report()["applying"], "possible");
    let not_holding = start(&[LAUNCH_INSTANT]);
    not_holding.ok("/fixture/condition?value=1");
    assert_eq!(not_holding.report()["applying"], "not possible");
}

// behavior-contexts-library-node: A later change to the condition leaves the applied values as they were
#[test]
fn a_later_change_to_the_condition_leaves_the_applied_values_as_they_were() {
    let launch = ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west"}"#);
    let holding = start(&[STACK, launch]);
    holding.ok("/fixture/condition?value=0");
    let report = holding.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu-west"));
    assert_eq!(holding.setting(INSTANT_NAME).as_deref(), Some("true"));
    let not_holding = start(&[launch]);
    not_holding.ok("/fixture/condition?value=1");
    let report = not_holding.report();
    assert_eq!(value_of(&report, INSTANT), None);
    assert_eq!(not_holding.setting(INSTANT_NAME), None);
    assert_eq!(not_holding.region()["applies"], 0);
}

// behavior-contexts-library-node: `applying` is `possible` when the condition held at start
#[test]
fn applying_is_possible_when_the_condition_held_at_start() {
    assert_eq!(start(&[STACK]).report()["applying"], "possible");
}

// behavior-contexts-library-node: `applying` is `not possible` when the condition did not hold at start
#[test]
fn applying_is_not_possible_when_the_condition_did_not_hold_at_start() {
    assert_eq!(start(&[]).report()["applying"], "not possible");
}

// behavior-contexts-library-node: The library applies a value only when its condition held at start
#[test]
fn the_library_applies_a_value_only_when_its_condition_held_at_start() {
    let website = start(&[(INSTANT_NAME, "earlier"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west"}"#)]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("earlier"));
    assert_eq!(website.region()["applies"], 0);
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("earlier"));
}

// behavior-contexts-library-node: At start, when its condition holds, the library reads `AZIMUTH_CONTEXT`
#[test]
fn at_start_when_its_condition_holds_the_library_reads_azimuth_context() {
    let website = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
    website.ok("/fixture/env?name=AZIMUTH_CONTEXT&value=%7B%22payouts.instant%22%3A%22changed%22%7D");
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("true"));
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
}

// behavior-contexts-library-node: The library applies the value of each mapped key that `AZIMUTH_CONTEXT` names, and no other key
#[test]
fn the_library_applies_the_value_of_each_mapped_key_that_azimuth_context_names_and_no_other_key() {
    let website = start(&[
        STACK,
        ("FIXTURE_KEYS", "wide"),
        ("FEATURE_SECOND_PAYOUTS", "kept"),
        ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.refunds":"on"}"#),
    ]);
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(value_of(&report, "payouts.second").as_deref(), Some("kept"));
    assert_eq!(website.region()["applies"], 0);
    assert!(website.state()["sideTeamApplied"].is_null());
    assert!(report["context"].get("payouts.refunds").is_none());
    assert_eq!(website.setting("FEATURE_REFUNDS_PAYOUTS"), None);
}

// behavior-contexts-library-node: Without `AZIMUTH_CONTEXT` the library applies nothing
#[test]
fn without_azimuth_context_the_library_applies_nothing() {
    let website = start(&[STACK, (INSTANT_NAME, "earlier"), ("FIXTURE_REGION_VALUE", "eu")]);
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("earlier"));
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu"));
    assert_eq!(website.region()["applies"], 0);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("earlier"));
}

// behavior-contexts-library-node: An `AZIMUTH_CONTEXT` that is not one JSON object of text values stops the start with an error that names the variable
#[test]
fn an_azimuth_context_that_is_not_one_json_object_of_text_values_stops_the_start_with_an_error_that_names_the_variable() {
    for bad in ["not json", "[]", r#""text""#, r#"{"payouts.instant":1}"#, r#"{"payouts.instant":true}"#, r#"{"payouts.instant":{"a":"b"}}"#, r#"{"payouts.instant":"true"} {}"#] {
        let output = run_to_end(&[STACK, ("AZIMUTH_CONTEXT", bad)]);
        assert_ne!(output.status, 0, "{bad} did not stop the start:\n{}", output.text());
        assert!(!output.stdout.contains("listening"), "{bad}: the component started\n{}", output.text());
        assert!(output.stderr.contains("registration failed:"), "{bad}: registration did not throw\n{}", output.text());
        let message = output.stderr.split("registration failed:").nth(1).unwrap();
        assert!(message.contains("AZIMUTH_CONTEXT"), "{bad}: the error does not name the variable:\n{}", output.text());
    }
}

// behavior-contexts-library-node: Applying is possible only under the component's own explicit condition
#[test]
fn applying_is_possible_only_under_the_components_own_explicit_condition() {
    let website = start(&[STACK, ("FIXTURE_CONDITION_VARIABLE", "WEBSITE_TEST_STACK"), LAUNCH_INSTANT]);
    let report = website.report();
    assert_eq!(report["applying"], "not possible");
    assert_eq!(value_of(&report, INSTANT), None);
    assert_eq!(website.setting(INSTANT_NAME), None);
}

// behavior-contexts-library-node: The values to apply come from `AZIMUTH_CONTEXT`, one JSON object of product keys to text values
#[test]
fn the_values_to_apply_come_from_azimuth_context() {
    let website = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west é \"q\""}"#)]);
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu-west é \"q\""));
}

// behavior-contexts-library-node: The launch's value reaches each mapped setting when applying is possible
#[test]
fn the_launchs_value_reaches_each_mapped_setting_when_applying_is_possible() {
    let website = start(&[
        STACK,
        ("FIXTURE_KEYS", "wide"),
        ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.second":"2","payouts.region":"eu-west","payouts.sideteam":"s"}"#),
    ]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
    assert_eq!(website.setting("FEATURE_SECOND_PAYOUTS").as_deref(), Some("2"));
    assert_eq!(website.region()["lastApplied"], "eu-west");
    assert_eq!(website.state()["sideTeamApplied"], "s");
}

// ---------------------------------------------------------------- Sources

// behavior-contexts-library-node: A source reads a key's current value and where it came from
#[test]
fn a_source_reads_a_keys_current_value_and_where_it_came_from() {
    let website = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_NAME", "region-flags")]);
    let report = website.report();
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu"));
    assert_eq!(key(&report, REGION)["source"]["name"], "region-flags");
}

// behavior-contexts-library-node: A source applies a value in this process
#[test]
fn a_source_applies_a_value_in_this_process() {
    let website = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    assert_eq!(website.region()["applies"], 1);
    assert_eq!(website.region()["lastApplied"], "eu-west");
    assert_eq!(value_of(&website.report(), REGION).as_deref(), Some("eu-west"));
}

// behavior-contexts-library-node: The library ships one source over the stack's configuration
#[test]
fn the_library_ships_one_source_over_the_stacks_configuration() {
    let website = start(&[STACK, ("FIXTURE_KEYS", "instant"), (INSTANT_NAME, "earlier")]);
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("earlier"));
    let applying = start(&[STACK, ("FIXTURE_KEYS", "instant"), (INSTANT_NAME, "earlier"), LAUNCH_INSTANT]);
    assert_eq!(applying.setting(INSTANT_NAME).as_deref(), Some("true"));
    assert_eq!(value_of(&applying.report(), INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-node: The shipped source is used for each key with no other registered source
#[test]
fn the_shipped_source_is_used_for_each_key_with_no_other_registered_source() {
    let website = start(&[
        ("FIXTURE_KEYS", "wide"),
        (INSTANT_NAME, "a"),
        ("FEATURE_SECOND_PAYOUTS", "b"),
        ("FIXTURE_REGION_VALUE", "eu"),
        ("FIXTURE_REGION_NAME", "region-flags"),
    ]);
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("a"));
    assert_eq!(value_of(&report, "payouts.second").as_deref(), Some("b"));
    assert_eq!(key(&report, REGION)["source"]["name"], "region-flags");
    assert_eq!(website.region()["reads"], 1);
    assert_eq!(website.state()["conditionCalls"], 1);
}

// behavior-contexts-library-node: The source that applies Azimuth's values is added last among the configuration's sources
#[test]
fn the_source_that_applies_azimuths_values_is_added_last_among_the_configurations_sources() {
    // `process.env` is one store and has no list of sources, so the order is observed in time: the
    // library applies at registration, over what was there before it, and under what a later writer
    // puts there after it.
    let website = start(&[STACK, (INSTANT_NAME, "earlier"), LAUNCH_INSTANT]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
    website.write_setting(INSTANT_NAME, "later");
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("later"));
    assert_eq!(precedence_of(&website.report(), INSTANT), "after azimuth");
}

// behavior-contexts-library-node: Azimuth's applied value wins over the configuration's other sources
#[test]
fn azimuths_applied_value_wins_over_the_configurations_other_sources() {
    let website = start(&[STACK, (INSTANT_NAME, "earlier"), LAUNCH_INSTANT]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(precedence_of(&report, INSTANT), "azimuth");
}

// behavior-contexts-library-node: A team registers its own source for the keys that a flag service, a database or another store serves
#[test]
fn a_team_registers_its_own_source_for_the_keys_that_a_flag_service_a_database_or_another_store_serves() {
    let website = start(&[STACK, ("FIXTURE_KEYS", "region"), ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    assert_eq!(context_keys(&website.report()), ["payouts.region"]);
    assert_eq!(value_of(&website.report(), REGION).as_deref(), Some("eu-west"));
    assert_eq!(website.setting("payouts.region"), None);
    assert_eq!(website.setting("FEATURE_REGION_PAYOUTS"), None);
}

// behavior-contexts-library-node: A team's own source has a read operation and an apply operation
#[test]
fn a_teams_own_source_has_a_read_operation_and_an_apply_operation() {
    let website = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    website.report();
    let region = website.region();
    assert!(region["reads"].as_u64().unwrap() >= 1, "read was not called: {region}");
    assert_eq!(region["applies"], 1);
}

// behavior-contexts-library-node: A key with a team's own source is read and applied through it
#[test]
fn a_key_with_a_teams_own_source_is_read_and_applied_through_it() {
    let website = start(&[STACK, ("FIXTURE_KEYS", "wide"), ("FIXTURE_SIDE", "before"), ("AZIMUTH_CONTEXT", r#"{"payouts.sideteam":"after"}"#)]);
    assert_eq!(website.state()["sideTeamApplied"], "after");
    let report = website.report();
    assert_eq!(value_of(&report, "payouts.sideteam").as_deref(), Some("after"));
    assert_eq!(key(&report, "payouts.sideteam")["source"]["name"], "side-store");
    assert_eq!(website.setting("FEATURE_SIDE_PAYOUTS"), None);
}

// behavior-contexts-library-node: A later source that wins is reported as an override
#[test]
fn a_later_source_that_wins_is_reported_as_an_override() {
    let website = start(&[STACK, ("FIXTURE_LATER", "FEATURE_INSTANT_PAYOUTS=false"), LAUNCH_INSTANT]);
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("false"));
    assert_eq!(precedence_of(&report, INSTANT), "after azimuth");
}

// behavior-contexts-library-node: The library ships no flag-service integration
#[test]
fn the_library_ships_no_flag_service_integration() {
    let flag_words = ["openfeature", "launchdarkly", "unleash", "flagsmith", "split.io", "splitsoftware", "featuremanagement", "growthbook", "configcat", "statsig"];
    let inspection = inspect();
    let mut spoken = strings(&inspection["exports"]);
    spoken.extend(strings(&inspection["contextProperties"]));
    spoken.extend(imported_modules());
    spoken.extend(declared_dependencies());
    let found: Vec<&String> = spoken.iter().filter(|name| flag_words.iter().any(|word| name.to_lowercase().contains(word))).collect();
    assert!(found.is_empty(), "the library integrates a flag service: {found:?}");
}

// behavior-contexts-library-node: The library requires no OpenFeature integration
#[test]
fn the_library_requires_no_openfeature_integration() {
    let dependencies = declared_dependencies();
    assert!(dependencies.iter().all(|name| !name.to_lowercase().contains("openfeature")), "{dependencies:?}");
    assert!(imported_modules().iter().all(|name| !name.to_lowercase().contains("openfeature")));
    let website = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("true"));
}

// ---------------------------------------------------------------- The shipped source over `process.env`

// behavior-contexts-library-node: The shipped source reads the mapped name from `process.env`
#[test]
fn the_shipped_source_reads_the_mapped_name_from_process_env() {
    let website = start(&[(INSTANT_NAME, "fromenv")]);
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("fromenv"));
    website.write_setting(INSTANT_NAME, "written");
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("written"));
}

// behavior-contexts-library-node: The shipped source applies by writing the mapped name to `process.env`
#[test]
fn the_shipped_source_applies_by_writing_the_mapped_name_to_process_env() {
    let website = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
}

// behavior-contexts-library-node: `website` mapped to `FEATURE_INSTANT_PAYOUTS` reads and applies `payouts.instant` under that name
#[test]
fn website_mapped_to_feature_instant_payouts_reads_and_applies_payouts_instant_under_that_name() {
    let reading = start(&[(INSTANT_NAME, "false")]);
    assert_eq!(reading.report()["component"], "website");
    assert_eq!(value_of(&reading.report(), INSTANT).as_deref(), Some("false"));
    let applying = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(applying.setting("FEATURE_INSTANT_PAYOUTS").as_deref(), Some("true"));
    assert_eq!(value_of(&applying.report(), INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-node: The source keeps the origin of its applied text
#[test]
fn the_source_keeps_the_origin_of_its_applied_text() {
    // The origin is observed by returning to the applied text: after a later writer changed the
    // name and wrote the applied text back, the report is again the applied value's.
    let website = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(precedence_of(&website.report(), INSTANT), "azimuth");
    website.write_setting(INSTANT_NAME, "false");
    assert_eq!(precedence_of(&website.report(), INSTANT), "after azimuth");
    website.write_setting(INSTANT_NAME, "true");
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(precedence_of(&report, INSTANT), "azimuth");
}

// behavior-contexts-library-node: When the current text of the name differs from the applied text, the environment is reported as a later source
#[test]
fn when_the_current_text_of_the_name_differs_from_the_applied_text_the_environment_is_reported_as_a_later_source() {
    let website = start(&[STACK, LAUNCH_INSTANT]);
    website.write_setting(INSTANT_NAME, "false");
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("false"));
    assert_eq!(precedence_of(&report, INSTANT), "after azimuth");
    assert!(key(&report, INSTANT)["source"]["name"].as_str().is_some_and(|name| !name.is_empty()), "{report}");
}

// behavior-contexts-library-node: A later writer of the same text cannot be told from the applied value in `process.env`
#[test]
fn a_later_writer_of_the_same_text_cannot_be_told_from_the_applied_value_in_process_env() {
    let website = start(&[STACK, LAUNCH_INSTANT]);
    let before = website.report();
    website.write_setting(INSTANT_NAME, "true");
    let after = website.report();
    assert_eq!(without_times(&before), without_times(&after));
    assert_eq!(precedence_of(&after, INSTANT), "azimuth");
}

// behavior-contexts-library-node: Another configuration store of the application needs a registered source of its own
#[test]
fn another_configuration_store_of_the_application_needs_a_registered_source_of_its_own() {
    let website = start(&[("FIXTURE_KEYS", "wide"), ("FIXTURE_SIDE", "x")]);
    let report = website.report();
    assert_eq!(value_of(&report, "payouts.side"), None);
    assert_eq!(value_of(&report, "payouts.sideteam").as_deref(), Some("x"));
}

// ---------------------------------------------------------------- The callback and the report

// behavior-contexts-library-node: The callback returns the report
#[test]
fn the_callback_returns_the_report() {
    let report = start(&[STACK]).report();
    assert_eq!(report["format"], 1);
    assert_eq!(report["component"], "website");
    assert!(report["context"].is_object());
}

// behavior-contexts-library-node: The library reads every mapped key again at each call of the callback
#[test]
fn the_library_reads_every_mapped_key_again_at_each_call_of_the_callback() {
    let website = start(&[("FIXTURE_REGION_VALUE", "eu")]);
    let first = website.report();
    let reads_after_first = website.region()["reads"].as_u64().unwrap();
    let second = website.report();
    let reads_after_second = website.region()["reads"].as_u64().unwrap();
    assert!(reads_after_first >= 1);
    assert!(reads_after_second > reads_after_first, "a second call did not read a team's source again");
    std::thread::sleep(Duration::from_millis(1100));
    let third = website.report();
    for name in [INSTANT, REGION] {
        let read = |report: &Value| utc_seconds(key(report, name)["read"].as_str().unwrap());
        assert!(read(&third) > read(&second), "{name} was not read again");
        assert!(read(&second) >= read(&first));
    }
}

// behavior-contexts-library-node: A value that changed while the component ran is read again at the next call
#[test]
fn a_value_that_changed_while_the_component_ran_is_read_again_at_the_next_call() {
    let website = start(&[("FIXTURE_REGION_VALUE", "eu")]);
    assert_eq!(value_of(&website.report(), REGION).as_deref(), Some("eu"));
    website.ok("/fixture/region?value=us");
    assert_eq!(value_of(&website.report(), REGION).as_deref(), Some("us"));
}

// behavior-contexts-library-node: The report has format `1`
#[test]
fn the_report_has_format_1() {
    assert_eq!(start(&[]).report()["format"], 1);
}

// behavior-contexts-library-node: The report reports exactly its mapped keys
#[test]
fn the_report_reports_exactly_its_mapped_keys() {
    for (keys, expected) in [
        ("default", vec!["payouts.instant", "payouts.region"]),
        ("instant", vec!["payouts.instant"]),
        ("region", vec!["payouts.region"]),
        ("wide", vec!["payouts.instant", "payouts.region", "payouts.second", "payouts.side", "payouts.sideteam"]),
    ] {
        let website = start(&[STACK, ("FIXTURE_KEYS", keys), ("FEATURE_UNMAPPED_PAYOUTS", "u")]);
        assert_eq!(context_keys(&website.report()), expected, "mapping {keys}");
    }
}

// behavior-contexts-library-node: The report carries `component`, `applying` and `context`
#[test]
fn the_report_carries_component_applying_and_context() {
    let report = start(&[STACK]).report();
    let mut fields: Vec<&String> = report.as_object().unwrap().keys().collect();
    fields.sort();
    assert_eq!(fields, ["applying", "component", "context", "format"]);
}

// behavior-contexts-library-node: The report carries no identity of its instance and no mark of a launch
#[test]
fn the_report_carries_no_identity_of_its_instance_and_no_mark_of_a_launch() {
    let launch = r#"{"payouts.instant":"true"}"#;
    let first = start(&[STACK, ("AZIMUTH_CONTEXT", launch), ("AZIMUTH_LAUNCH", "launch-7d3f")]).report();
    let second = start(&[STACK, ("AZIMUTH_CONTEXT", launch), ("AZIMUTH_LAUNCH", "launch-91ac")]).report();
    assert_eq!(without_times(&first), without_times(&second));
    assert!(!first.to_string().contains("launch-7d3f"));
    for (_, entry) in first["context"].as_object().unwrap() {
        let mut fields: Vec<&String> = entry.as_object().unwrap().keys().collect();
        fields.sort();
        assert_eq!(fields, ["read", "source", "value"]);
    }
}

// behavior-contexts-library-node: `component` is the name from registration
#[test]
fn component_is_the_name_from_registration() {
    assert_eq!(start(&[]).report()["component"], "website");
    assert_eq!(start(&[("FIXTURE_COMPONENT", "api")]).report()["component"], "api");
}

// behavior-contexts-library-node: `context` is an object of mapped product keys
#[test]
fn context_is_an_object_of_mapped_product_keys() {
    let report = start(&[]).report();
    assert!(report["context"].is_object());
    assert_eq!(context_keys(&report), ["payouts.instant", "payouts.region"]);
    assert!(report["context"].get(INSTANT_NAME).is_none(), "the report names a setting and not a product key");
}

// behavior-contexts-library-node: A key's `value` is text, or `null` when absent
#[test]
fn a_keys_value_is_text_or_null_when_absent() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu")]).report();
    assert!(key(&report, REGION)["value"].is_string());
    assert!(key(&report, INSTANT)["value"].is_null());
}

// behavior-contexts-library-node: A key's `source` is `null` when absent
#[test]
fn a_keys_source_is_null_when_absent() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_SOURCE", "none")]).report();
    assert_eq!(key(&report, REGION)["value"], "eu");
    assert!(key(&report, REGION).get("source").is_some_and(Value::is_null), "{report}");
}

// behavior-contexts-library-node: A key's `read` is the UTC time it was read
#[test]
fn a_keys_read_is_the_utc_time_it_was_read() {
    let website = start(&[]);
    let before = now_seconds();
    let report = website.report();
    let after = now_seconds();
    for name in [INSTANT, REGION] {
        let read = utc_seconds(key(&report, name)["read"].as_str().unwrap());
        assert!(read >= before - 2.0 && read <= after + 2.0, "{name} read at {read}, between {before} and {after}");
    }
}

// behavior-contexts-library-node: A source has a `name` and a `precedence`
#[test]
fn a_source_has_a_name_and_a_precedence() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_NAME", "region-flags"), ("FIXTURE_REGION_PRECEDENCE", "before azimuth")]).report();
    let source = key(&report, REGION)["source"].as_object().unwrap();
    let mut fields: Vec<&String> = source.keys().collect();
    fields.sort();
    assert_eq!(fields, ["name", "precedence"]);
    assert_eq!(source["name"], "region-flags");
    assert_eq!(source["precedence"], "before azimuth");
}

// behavior-contexts-library-node: `before azimuth` says that Azimuth's value would win
#[test]
fn before_azimuth_says_that_azimuths_value_would_win() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_PRECEDENCE", "before azimuth")]).report();
    assert_eq!(precedence_of(&report, REGION), "before azimuth");
    let shipped = start(&[(INSTANT_NAME, "earlier")]).report();
    assert_eq!(precedence_of(&shipped, INSTANT), "before azimuth");
}

// behavior-contexts-library-node: `azimuth` says that this is Azimuth's applied value
#[test]
fn azimuth_says_that_this_is_azimuths_applied_value() {
    let website = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west"}"#)]);
    let report = website.report();
    assert_eq!(precedence_of(&report, INSTANT), "azimuth");
    assert_eq!(precedence_of(&report, REGION), "azimuth");
}

// behavior-contexts-library-node: `after azimuth` says that this source wins over Azimuth's value
#[test]
fn after_azimuth_says_that_this_source_wins_over_azimuths_value() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_PRECEDENCE", "after azimuth")]).report();
    assert_eq!(precedence_of(&report, REGION), "after azimuth");
    let later = start(&[STACK, ("FIXTURE_LATER", "FEATURE_INSTANT_PAYOUTS=false"), LAUNCH_INSTANT]).report();
    assert_eq!(precedence_of(&later, INSTANT), "after azimuth");
}

// behavior-contexts-library-node: An absent value or source is reported as absent
#[test]
fn an_absent_value_or_source_is_reported_as_absent() {
    let no_value = start(&[("FIXTURE_REGION_NAME", "region-flags")]).report();
    assert!(key(&no_value, REGION)["value"].is_null());
    let no_source = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_SOURCE", "none")]).report();
    assert!(key(&no_source, REGION)["source"].is_null());
    assert_eq!(key(&no_source, REGION)["value"], "eu");
}

// behavior-contexts-library-node: The library reports its mappings, not what it thinks the declaration ought to hold
#[test]
fn the_library_reports_its_mappings_not_what_it_thinks_the_declaration_ought_to_hold() {
    let project = decoy_project();
    let website = start_in(&[STACK, ("FIXTURE_KEYS", "region")], Some(project.path()));
    assert_eq!(context_keys(&website.report()), ["payouts.region"]);
}

// ---------------------------------------------------------------- Cases of the conformance kit for the report of a component

// behavior-contexts-library-node: A mapping reads a value through the callback, for read
#[test]
fn a_mapping_reads_a_value_through_the_callback_for_read() {
    let website = start(&[(INSTANT_NAME, "false")]);
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("false"));
}

// behavior-contexts-library-node: A mapping applies a value, for apply
#[test]
fn a_mapping_applies_a_value_for_apply() {
    let website = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(website.setting(INSTANT_NAME).as_deref(), Some("true"));
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-node: The callback gives a report when applying is possible
#[test]
fn the_callback_gives_a_report_when_applying_is_possible() {
    let report = start(&[STACK, LAUNCH_INSTANT]).report();
    assert_eq!(report["applying"], "possible");
    assert_eq!(report["component"], "website");
}

// behavior-contexts-library-node: The callback gives a report when applying is not possible
#[test]
fn the_callback_gives_a_report_when_applying_is_not_possible() {
    let report = start(&[LAUNCH_INSTANT]).report();
    assert_eq!(report["applying"], "not possible");
    assert_eq!(report["component"], "website");
    assert_eq!(context_keys(&report), ["payouts.instant", "payouts.region"]);
}

// behavior-contexts-library-node: An absent value is reported as absent
#[test]
fn an_absent_value_is_reported_as_absent() {
    let report = start(&[STACK]).report();
    assert!(key(&report, INSTANT)["value"].is_null());
    assert!(key(&report, REGION)["value"].is_null());
}

// behavior-contexts-library-node: A key the launch does not state keeps its source
#[test]
fn a_key_the_launch_does_not_state_keeps_its_source() {
    let website = start(&[STACK, ("FIXTURE_KEYS", "wide"), ("FEATURE_SECOND_PAYOUTS", "kept"), ("FIXTURE_REGION_VALUE", "eu"), LAUNCH_INSTANT]);
    let report = website.report();
    assert_eq!(value_of(&report, "payouts.second").as_deref(), Some("kept"));
    assert_eq!(precedence_of(&report, "payouts.second"), "before azimuth");
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu"));
    assert_eq!(website.region()["applies"], 0);
    assert_eq!(website.setting("FEATURE_SECOND_PAYOUTS").as_deref(), Some("kept"));
}

// behavior-contexts-library-node: A source added later that gives another value is reported with precedence `after azimuth`
#[test]
fn a_source_added_later_that_gives_another_value_is_reported_with_precedence_after_azimuth() {
    let website = start(&[STACK, ("FIXTURE_KEYS", "instant"), ("FIXTURE_LATER", "FEATURE_INSTANT_PAYOUTS=false"), LAUNCH_INSTANT]);
    let report = website.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("false"));
    assert_eq!(precedence_of(&report, INSTANT), "after azimuth");
    assert!(key(&report, INSTANT)["source"]["name"].as_str().is_some_and(|name| !name.is_empty()));
}

// behavior-contexts-library-node: A value that changed is read again at the next call
#[test]
fn a_value_that_changed_is_read_again_at_the_next_call() {
    let website = start(&[STACK, LAUNCH_INSTANT]);
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("true"));
    website.write_setting(INSTANT_NAME, "false");
    assert_eq!(value_of(&website.report(), INSTANT).as_deref(), Some("false"));
}

// behavior-contexts-library-node: The component name and the report format are checked
#[test]
fn the_component_name_and_the_report_format_are_checked() {
    let report = start(&[STACK]).declared_report();
    assert_eq!(report["component"], "website");
    assert_eq!(report["format"], 1);
}

// behavior-contexts-library-node: The document of the callback is the same document through the fixture's declared path
#[test]
fn the_document_of_the_callback_is_the_same_document_through_the_fixtures_declared_path() {
    let website = start(&[STACK, ("FIXTURE_REGION_VALUE", "eu"), LAUNCH_INSTANT]);
    let (status, _) = website.get("/internal/azimuth");
    assert_eq!(status, 200);
    assert_eq!(without_times(&website.report()), without_times(&website.declared_report()));
}

// behavior-contexts-library-node: A team's own source passes these cases too
#[test]
fn a_teams_own_source_passes_these_cases_too() {
    let applied = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    let report = applied.report();
    assert_eq!(report["applying"], "possible");
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu-west"));
    assert_eq!(precedence_of(&report, REGION), "azimuth");

    let not_possible = start(&[("FIXTURE_REGION_VALUE", "eu"), ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    let report = not_possible.report();
    assert_eq!(report["applying"], "not possible");
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu"));
    assert_eq!(not_possible.region()["applies"], 0);

    let absent = start(&[STACK]);
    assert!(key(&absent.report(), REGION)["value"].is_null());

    let unstated = start(&[STACK, ("FIXTURE_REGION_VALUE", "eu"), LAUNCH_INSTANT]);
    assert_eq!(value_of(&unstated.report(), REGION).as_deref(), Some("eu"));
    assert_eq!(unstated.region()["applies"], 0);

    let changing = start(&[("FIXTURE_REGION_VALUE", "eu")]);
    assert_eq!(value_of(&changing.report(), REGION).as_deref(), Some("eu"));
    changing.ok("/fixture/region?value=us");
    assert_eq!(value_of(&changing.report(), REGION).as_deref(), Some("us"));

    let later = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_PRECEDENCE", "after azimuth")]);
    assert_eq!(precedence_of(&later.report(), REGION), "after azimuth");
}
