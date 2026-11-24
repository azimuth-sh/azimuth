//! The behaviors of the .NET context library, `docs/increments/contexts-library-dotnet.md`.
//!
//! The fixture is `api` of the invented payouts product, `tests/sample/payouts/api/`, written the way
//! the design's example shows. Each test starts it (or inspects the library through it) in a process
//! of its own and drives it with `AZIMUTH_CONTEXT`, `PAYOUTS_TEST_STACK` and the fixture's own
//! `FIXTURE_*` variables. It builds against the library by project path, so until
//! `libraries/dotnet/Azimuth.Context/` exists every test here fails at the build. The tests need the
//! .NET SDK and run only with `cargo test --features dotnet`.

use azimuth_tests::*;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const STACK: (&str, &str) = ("PAYOUTS_TEST_STACK", "1");
const INSTANT: &str = "payouts.instant";
const REGION: &str = "payouts.region";
const INSTANT_PATH: &str = "Features:Payouts:Instant";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn fresh_folder(name: &str) -> PathBuf {
    static COUNTER: OnceLock<Mutex<u32>> = OnceLock::new();
    let mut counter = COUNTER.get_or_init(|| Mutex::new(0)).lock().unwrap();
    *counter += 1;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let folder = binaries_dir().join(format!("context-library-{name}-{stamp}-{counter}"));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// A restore that starts from an empty package folder and a home of its own, so no package an
/// earlier run left can stand in for one the library should have declared.
fn dotnet_build_command(folder: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("dotnet");
    command
        .args(args)
        .current_dir(repo_root())
        .env("NUGET_PACKAGES", folder.join("nuget-packages"))
        .env("DOTNET_CLI_HOME", folder.join("dotnet-home"))
        .env("DOTNET_NOLOGO", "1");
    command
}

/// The built fixture, once per test binary.
fn fixture_dll() -> &'static PathBuf {
    static DLL: OnceLock<PathBuf> = OnceLock::new();
    DLL.get_or_init(|| {
        let folder = fresh_folder("build");
        let output = run_with_time_limit(
            dotnet_build_command(
                &folder,
                &[
                    "build",
                    "tests/sample/payouts/api/Api.csproj",
                    "--configuration",
                    "Release",
                    "--artifacts-path",
                    folder.join("artifacts").to_str().unwrap(),
                ],
            ),
            Duration::from_secs(900),
            None,
        );
        output.expect_status(0);
        let dll = folder.join("artifacts/bin/Api/release/Api.dll");
        assert!(dll.is_file(), "the build wrote no {}\n{}", dll.display(), output.text());
        dll
    })
}

/// The library packed once per test binary.
fn library_package() -> &'static PathBuf {
    static PACKAGE: OnceLock<PathBuf> = OnceLock::new();
    PACKAGE.get_or_init(|| {
        let folder = fresh_folder("pack");
        let output = run_with_time_limit(
            dotnet_build_command(
                &folder,
                &[
                    "pack",
                    "libraries/dotnet/Azimuth.Context/Azimuth.Context.csproj",
                    "--configuration",
                    "Release",
                    "--output",
                    folder.join("feed").to_str().unwrap(),
                ],
            ),
            Duration::from_secs(900),
            None,
        );
        output.expect_status(0);
        let packages: Vec<PathBuf> = std::fs::read_dir(folder.join("feed"))
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "nupkg"))
            .collect();
        assert_eq!(packages.len(), 1, "the pack did not write one package: {packages:?}");
        packages[0].clone()
    })
}

fn nuspec() -> String {
    let package = library_package();
    let output = Command::new("unzip")
        .args(["-p", package.to_str().unwrap(), "*.nuspec"])
        .output()
        .expect("unzip reads the package");
    assert!(output.status.success(), "unzip: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
}

fn nuspec_dependency_ids() -> Vec<String> {
    nuspec()
        .split("<dependency ")
        .skip(1)
        .filter_map(|part| {
            let start = part.find("id=\"")? + 4;
            let end = part[start..].find('"')? + start;
            Some(part[start..end].to_string())
        })
        .collect()
}

/// A command for the fixture with the variables of an earlier test taken out of its environment.
fn fixture_command(env: &[(&str, &str)], dir: &Path) -> Command {
    let mut command = Command::new("dotnet");
    command.arg(fixture_dll()).current_dir(dir);
    for (name, _) in std::env::vars() {
        if name.starts_with("FIXTURE_")
            || name.starts_with("Features__")
            || name == "AZIMUTH_CONTEXT"
            || name == "PAYOUTS_TEST_STACK"
            || name == "WEBSITE_TEST_STACK"
        {
            command.env_remove(name);
        }
    }
    command
        .env("DOTNET_NOLOGO", "1")
        .env("DOTNET_EnableDiagnostics", "0")
        .env("Logging__LogLevel__Default", "Warning");
    for (name, value) in env {
        command.env(name, value);
    }
    command
}

fn run_to_end(env: &[(&str, &str)]) -> Output {
    let dir = fresh_folder("run");
    run_with_time_limit(fixture_command(env, &dir), Duration::from_secs(120), None)
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

struct Api {
    child: Child,
    port: u16,
    done: Arc<AtomicBool>,
}

impl Drop for Api {
    fn drop(&mut self) {
        self.done.store(true, Ordering::SeqCst);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Starts `api` and waits until it says where it listens. The fixture listens on a port of its
/// own choosing on the loopback, which is the fixture's door and not the library's.
fn start(env: &[(&str, &str)]) -> Api {
    start_in(env, &fresh_folder("run"))
}

fn start_in(env: &[(&str, &str)], dir: &Path) -> Api {
    let mut command = fixture_command(env, dir);
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
        for _ in 0..240 {
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
    Api { child, port, done }
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

impl Api {
    fn get(&self, path: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
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

    fn setting(&self, path: &str) -> Option<String> {
        self.ok(&format!("/fixture/setting?path={path}"))["value"].as_str().map(str::to_string)
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

fn network_references(inspection: &Value) -> Vec<String> {
    strings(&inspection["references"])
        .into_iter()
        .filter(|name| {
            ["System.Net.Sockets", "System.Net.Http", "System.Net.Security", "System.Net.WebSockets", "System.Net.Requests", "System.Net.Quic", "System.Net.HttpListener", "System.Net.NameResolution", "System.Net.Ping", "System.Net.Mail", "Microsoft.AspNetCore", "Microsoft.Extensions.Http"]
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
        .collect()
}

/// A component that registered and builds its report in a process with no web host, and the
/// sockets that process holds while the library is registered and has reported.
fn sockets_of_a_registered_component() -> String {
    let dir = fresh_folder("run");
    let mut command = fixture_command(
        &[STACK, ("FIXTURE_MODE", "console"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)],
        &dir,
    );
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

fn decoy_project() -> PathBuf {
    let dir = fresh_folder("decoy");
    std::fs::create_dir_all(dir.join("azimuth")).unwrap();
    std::fs::write(
        dir.join("azimuth/project.toml"),
        "cli = \"0.2.0-alpha.2\"\n\n[repository]\nname = \"payouts\"\ntarget = \"main\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("azimuth/execution-context.toml"),
        "[payouts.declared]\ntype = \"text\"\ncomponents = [\"api\"]\n\n[payouts.instant]\ntype = \"text\"\ncomponents = [\"api\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("Decoy.csproj"),
        "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><AzimuthMap>payouts.declared=Features:Payouts:Declared</AzimuthMap></PropertyGroup></Project>\n",
    )
    .unwrap();
    dir
}

// ---------------------------------------------------------------- The package

// behavior-contexts-library-dotnet: The library is named `Azimuth.Context` for .NET
#[test]
fn the_library_is_named_azimuth_context() {
    assert!(nuspec().contains("<id>Azimuth.Context</id>"), "the package id is not Azimuth.Context:\n{}", nuspec());
    assert_eq!(inspect()["assembly"], "Azimuth.Context");
}

// behavior-contexts-library-dotnet: The library speaks of keys, sources, values, applying and reports, and names no testing
#[test]
fn the_library_speaks_of_keys_sources_values_applying_and_reports_and_names_no_testing() {
    let inspection = inspect();
    let names = strings(&inspection["names"]);
    for spoken in ["Key", "Read", "Apply", "Report", "Value", "Source", "AddAzimuthContext"] {
        assert!(names.iter().any(|name| name == spoken), "no public member or type {spoken}: {names:?}");
    }
    let testing = ["test", "mock", "fake", "stub", "xunit", "nunit", "mstest", "fixture"];
    let references = strings(&inspection["references"]);
    let spoken_of_testing: Vec<&String> = names
        .iter()
        .chain(references.iter())
        .filter(|name| testing.iter().any(|word| name.to_lowercase().contains(word)))
        .collect();
    assert!(spoken_of_testing.is_empty(), "the library names testing: {spoken_of_testing:?}");
    let dependencies = nuspec_dependency_ids();
    let testing_dependencies: Vec<&String> = dependencies
        .iter()
        .filter(|name| testing.iter().any(|word| name.to_lowercase().contains(word)))
        .collect();
    assert!(testing_dependencies.is_empty(), "the package depends on testing: {testing_dependencies:?}");
}

// behavior-contexts-library-dotnet: The library opens no port
#[test]
fn the_library_opens_no_port() {
    let listing = sockets_of_a_registered_component();
    assert!(listing.trim().is_empty(), "a registered component with no web host holds sockets:\n{listing}");
    let referencing = network_references(&inspect());
    assert!(referencing.is_empty(), "the library references networking assemblies: {referencing:?}");
}

// behavior-contexts-library-dotnet: The library makes no connection out
#[test]
fn the_library_makes_no_connection_out() {
    let listing = sockets_of_a_registered_component();
    assert!(!listing.contains("ESTABLISHED") && !listing.contains("->"), "the process holds a connection:\n{listing}");
    let referencing = network_references(&inspect());
    assert!(referencing.is_empty(), "the library references networking assemblies: {referencing:?}");
}

// behavior-contexts-library-dotnet: The library exposes the report at no path of its own; the component does, at its declared `report` path
#[test]
fn the_library_exposes_the_report_at_no_path_of_its_own() {
    let silent = start(&[STACK, ("FIXTURE_NO_REPORT_PATH", "1")]);
    for path in ["/", "/internal/azimuth", "/azimuth", "/report", "/azimuth/report", "/.well-known/azimuth", "/internal/azimuth/report", "/_azimuth", "/context"] {
        assert_eq!(silent.get(path).0, 404, "{path} answers though the component declares no path");
    }
    let declaring = start(&[STACK]);
    let (status, body) = declaring.get("/internal/azimuth");
    assert_eq!(status, 200);
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["component"], "api");
}

// behavior-contexts-library-dotnet: The same report is used in a launch and in a deployment
#[test]
fn the_same_report_is_used_in_a_launch_and_in_a_deployment() {
    let launch = start(&[STACK, ("AZIMUTH_CONTEXT", "{}")]).report();
    let deployment = start(&[STACK]).report();
    assert_eq!(without_times(&launch), without_times(&deployment));
    let launch_with_value = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]).report();
    assert_eq!(launch_with_value.as_object().unwrap().keys().collect::<Vec<_>>(), deployment.as_object().unwrap().keys().collect::<Vec<_>>());
    assert_eq!(context_keys(&launch_with_value), context_keys(&deployment));
}

// ---------------------------------------------------------------- Registration

// behavior-contexts-library-dotnet: A component registers its name
#[test]
fn a_component_registers_its_name() {
    let api = start(&[STACK, ("FIXTURE_COMPONENT", "payouts-api")]);
    assert_eq!(api.declared_report()["component"], "payouts-api");
}

// behavior-contexts-library-dotnet: A component registers the condition under which applying is possible
#[test]
fn a_component_registers_the_condition_under_which_applying_is_possible() {
    let launch = ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#);
    let holding = start(&[STACK, launch]);
    assert_eq!(holding.state()["conditionCalls"], 1);
    assert_eq!(holding.report()["applying"], "possible");
    let not_holding = start(&[launch]);
    assert_eq!(not_holding.state()["conditionCalls"], 1);
    assert_eq!(not_holding.report()["applying"], "not possible");
}

// behavior-contexts-library-dotnet: A component registers an explicit mapping of every key it uses to its own setting
#[test]
fn a_component_registers_an_explicit_mapping_of_every_key_it_uses_to_its_own_setting() {
    let api = start(&[STACK, ("FIXTURE_KEYS", "wide")]);
    assert_eq!(
        context_keys(&api.report()),
        ["payouts.instant", "payouts.region", "payouts.second", "payouts.side", "payouts.sideteam"]
    );
    let only_instant = start(&[STACK, ("FIXTURE_KEYS", "instant")]);
    assert_eq!(context_keys(&only_instant.report()), ["payouts.instant"]);
}

// behavior-contexts-library-dotnet: A mapping reads the key's own setting
#[test]
fn a_mapping_reads_the_keys_own_setting() {
    let api = start(&[("FIXTURE_INSTANT_PATH", "Features:Payouts:Other"), ("FIXTURE_EARLIER", "Features:Payouts:Other=other;Features:Payouts:Instant=default")]);
    let report = api.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("other"));
}

// behavior-contexts-library-dotnet: A mapping writes the key's own setting
#[test]
fn a_mapping_writes_the_keys_own_setting() {
    let api = start(&[STACK, ("FIXTURE_INSTANT_PATH", "Features:Payouts:Other"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(api.setting("Features:Payouts:Other").as_deref(), Some("true"));
    assert_eq!(api.setting(INSTANT_PATH), None);
}

// behavior-contexts-library-dotnet: No rule derives a setting from a key; a key with no mapping has no setting
#[test]
fn no_rule_derives_a_setting_from_a_key_and_a_key_with_no_mapping_has_no_setting() {
    let api = start(&[
        STACK,
        ("AZIMUTH_CONTEXT", r#"{"payouts.refunds":"on","payouts.instant":"true"}"#),
        ("FIXTURE_EARLIER", "Features:Payouts:Refunds=earlier"),
    ]);
    for derived in ["Features:Payouts:Refunds", "payouts.refunds", "Payouts:Refunds", "payouts:refunds", "Payouts"] {
        let expected = if derived == "Features:Payouts:Refunds" { Some("earlier") } else { None };
        assert_eq!(api.setting(derived).as_deref(), expected, "setting {derived}");
    }
    assert_eq!(context_keys(&api.report()), ["payouts.instant", "payouts.region"]);
}

// behavior-contexts-library-dotnet: The library knows no types
#[test]
fn the_library_knows_no_types() {
    let inspection = inspect();
    let generic = strings(&inspection["genericMembers"]);
    assert!(generic.is_empty(), "the library offers members that take a type of the application: {generic:?}");
    let names = strings(&inspection["names"]);
    let typed: Vec<&String> = names.iter().filter(|name| ["Options", "Binder", "Bind", "Typed", "Deserialize"].iter().any(|word| name.contains(word))).collect();
    assert!(typed.is_empty(), "the library speaks of types: {typed:?}");
}

// behavior-contexts-library-dotnet: The library reads no project file
#[test]
fn the_library_reads_no_project_file() {
    let project = decoy_project();
    let api = start_in(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.declared":"x","payouts.instant":"true"}"#)], &project);
    let report = api.report();
    assert_eq!(context_keys(&report), ["payouts.instant", "payouts.region"]);
    assert_eq!(api.setting("Features:Payouts:Declared"), None);
    assert_eq!(api.setting("payouts.declared"), None);
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-dotnet: The library gives text values and interprets none; the core and server interpret them by `execution-context.toml`
#[test]
fn the_library_gives_text_values_and_interprets_none() {
    for text in ["TRUE", "false", "null", "0", "[1]", "{}", "1e3", "eu-west"] {
        let launch = serde_json::json!({ "payouts.instant": text, "payouts.region": text }).to_string();
        let api = start(&[STACK, ("AZIMUTH_CONTEXT", &launch)]);
        let report = api.report();
        assert_eq!(key(&report, INSTANT)["value"], text, "reported {text}");
        assert_eq!(key(&report, REGION)["value"], text, "reported {text} for a team's source");
        assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some(text), "applied {text}");
        assert_eq!(api.region()["lastApplied"], text, "handed {text} to a team's source");
    }
}

// ---------------------------------------------------------------- The condition and applying

// behavior-contexts-library-dotnet: The component's own code gives the condition
#[test]
fn the_components_own_code_gives_the_condition() {
    let launch = ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#);
    assert_eq!(start(&[STACK, launch]).report()["applying"], "possible");
    assert_eq!(start(&[launch]).report()["applying"], "not possible");
    let other = start(&[("FIXTURE_CONDITION_VARIABLE", "WEBSITE_TEST_STACK"), ("WEBSITE_TEST_STACK", "1"), launch]);
    assert_eq!(other.report()["applying"], "possible");
}

// behavior-contexts-library-dotnet: The library never guesses applying from an environment name
#[test]
fn the_library_never_guesses_applying_from_an_environment_name() {
    let launch = ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#);
    for name in ["Development", "Testing", "Test", "Staging", "CI"] {
        let api = start(&[("ASPNETCORE_ENVIRONMENT", name), ("DOTNET_ENVIRONMENT", name), launch]);
        let report = api.report();
        assert_eq!(report["applying"], "not possible", "environment {name}");
        assert_eq!(api.setting(INSTANT_PATH), None, "environment {name}");
    }
    let production = start(&[STACK, ("ASPNETCORE_ENVIRONMENT", "Production"), ("DOTNET_ENVIRONMENT", "Production"), launch]);
    assert_eq!(production.report()["applying"], "possible");
}

// behavior-contexts-library-dotnet: The condition is evaluated once, at start
#[test]
fn the_condition_is_evaluated_once_at_start() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    api.report();
    api.declared_report();
    api.report();
    assert_eq!(api.state()["conditionCalls"], 1);
}

// behavior-contexts-library-dotnet: The condition is evaluated before applying
#[test]
fn the_condition_is_evaluated_before_applying() {
    let api = start(&[STACK, ("FIXTURE_EARLIER", "Features:Payouts:Instant=earlier"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(api.state()["seenAtCondition"], "earlier");
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("true"));
}

// behavior-contexts-library-dotnet: A later change to the condition leaves `applying` as it was
#[test]
fn a_later_change_to_the_condition_leaves_applying_as_it_was() {
    let launch = ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#);
    let holding = start(&[STACK, launch]);
    holding.ok("/fixture/condition?value=0");
    assert_eq!(holding.report()["applying"], "possible");
    let not_holding = start(&[launch]);
    not_holding.ok("/fixture/condition?value=1");
    assert_eq!(not_holding.report()["applying"], "not possible");
}

// behavior-contexts-library-dotnet: A later change to the condition leaves the applied values as they were
#[test]
fn a_later_change_to_the_condition_leaves_the_applied_values_as_they_were() {
    let launch = ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west"}"#);
    let holding = start(&[STACK, launch]);
    holding.ok("/fixture/condition?value=0");
    let report = holding.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu-west"));
    assert_eq!(holding.setting(INSTANT_PATH).as_deref(), Some("true"));
    let not_holding = start(&[launch]);
    not_holding.ok("/fixture/condition?value=1");
    let report = not_holding.report();
    assert_eq!(value_of(&report, INSTANT), None);
    assert_eq!(not_holding.setting(INSTANT_PATH), None);
    assert_eq!(not_holding.region()["applies"], 0);
}

// behavior-contexts-library-dotnet: `applying` is `possible` when the condition held at start
#[test]
fn applying_is_possible_when_the_condition_held_at_start() {
    assert_eq!(start(&[STACK]).report()["applying"], "possible");
}

// behavior-contexts-library-dotnet: `applying` is `not possible` when the condition did not hold at start
#[test]
fn applying_is_not_possible_when_the_condition_did_not_hold_at_start() {
    assert_eq!(start(&[]).report()["applying"], "not possible");
}

// behavior-contexts-library-dotnet: The library applies a value only when its condition held at start
#[test]
fn the_library_applies_a_value_only_when_its_condition_held_at_start() {
    let api = start(&[("FIXTURE_EARLIER", "Features:Payouts:Instant=earlier"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west"}"#)]);
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("earlier"));
    assert_eq!(api.region()["applies"], 0);
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("earlier"));
}

// behavior-contexts-library-dotnet: At start, when its condition holds, the library reads `AZIMUTH_CONTEXT`
#[test]
fn at_start_when_its_condition_holds_the_library_reads_azimuth_context() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("true"));
    api.ok(&format!("/fixture/env?name=AZIMUTH_CONTEXT&value={}", "%7B%22payouts.instant%22%3A%22changed%22%7D"));
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("true"));
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("true"));
}

// behavior-contexts-library-dotnet: The library applies the value of each mapped key that `AZIMUTH_CONTEXT` names, and no other key
#[test]
fn the_library_applies_the_value_of_each_mapped_key_that_azimuth_context_names_and_no_other_key() {
    let api = start(&[
        STACK,
        ("FIXTURE_KEYS", "wide"),
        ("FIXTURE_EARLIER", "Features:Payouts:Second=kept"),
        ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.refunds":"on"}"#),
    ]);
    let report = api.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(value_of(&report, "payouts.second").as_deref(), Some("kept"));
    assert_eq!(api.region()["applies"], 0);
    assert!(api.state()["sideTeamApplied"].is_null());
    assert!(report["context"].get("payouts.refunds").is_none());
    assert_eq!(api.setting("Features:Payouts:Refunds"), None);
}

// behavior-contexts-library-dotnet: Without `AZIMUTH_CONTEXT` the library applies nothing
#[test]
fn without_azimuth_context_the_library_applies_nothing() {
    let api = start(&[STACK, ("FIXTURE_EARLIER", "Features:Payouts:Instant=earlier"), ("FIXTURE_REGION_VALUE", "eu")]);
    let report = api.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("earlier"));
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu"));
    assert_eq!(api.region()["applies"], 0);
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("earlier"));
}

// behavior-contexts-library-dotnet: An `AZIMUTH_CONTEXT` that is not one JSON object of text values stops the start with an error that names the variable
#[test]
fn an_azimuth_context_that_is_not_one_json_object_of_text_values_stops_the_start_with_an_error_that_names_the_variable() {
    for bad in ["not json", "[]", r#""text""#, r#"{"payouts.instant":1}"#, r#"{"payouts.instant":true}"#, r#"{"payouts.instant":{"a":"b"}}"#, r#"{"payouts.instant":"true"} {}"#] {
        let output = run_to_end(&[STACK, ("AZIMUTH_CONTEXT", bad)]);
        assert_ne!(output.status, 0, "{bad} did not stop the start:\n{}", output.text());
        assert!(!output.stdout.contains("listening"), "{bad}: the component started\n{}", output.text());
        assert!(output.text().contains("AZIMUTH_CONTEXT"), "{bad}: the error does not name the variable:\n{}", output.text());
    }
}

// behavior-contexts-library-dotnet: Applying is possible only under the component's own explicit condition
#[test]
fn applying_is_possible_only_under_the_components_own_explicit_condition() {
    let api = start(&[
        STACK,
        ("FIXTURE_COMPONENT", "website"),
        ("FIXTURE_CONDITION_VARIABLE", "WEBSITE_TEST_STACK"),
        ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#),
    ]);
    let report = api.report();
    assert_eq!(report["applying"], "not possible");
    assert_eq!(value_of(&report, INSTANT), None);
    assert_eq!(api.setting(INSTANT_PATH), None);
}

// behavior-contexts-library-dotnet: The values to apply come from `AZIMUTH_CONTEXT`, one JSON object of product keys to text values
#[test]
fn the_values_to_apply_come_from_azimuth_context() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west é \"q\""}"#)]);
    let report = api.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("true"));
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu-west é \"q\""));
}

// behavior-contexts-library-dotnet: The launch's value reaches each mapped setting when applying is possible
#[test]
fn the_launchs_value_reaches_each_mapped_setting_when_applying_is_possible() {
    let api = start(&[STACK, ("FIXTURE_KEYS", "wide"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.second":"2","payouts.region":"eu-west","payouts.sideteam":"s"}"#)]);
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("true"));
    assert_eq!(api.setting("Features:Payouts:Second").as_deref(), Some("2"));
    assert_eq!(api.region()["lastApplied"], "eu-west");
    assert_eq!(api.state()["sideTeamApplied"], "s");
}

// ---------------------------------------------------------------- Sources

// behavior-contexts-library-dotnet: A source reads a key's current value and where it came from
#[test]
fn a_source_reads_a_keys_current_value_and_where_it_came_from() {
    let api = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_NAME", "region-flags")]);
    let report = api.report();
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu"));
    assert_eq!(key(&report, REGION)["source"]["name"], "region-flags");
}

// behavior-contexts-library-dotnet: A source applies a value in this process
#[test]
fn a_source_applies_a_value_in_this_process() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    assert_eq!(api.region()["applies"], 1);
    assert_eq!(api.region()["lastApplied"], "eu-west");
    assert_eq!(value_of(&api.report(), REGION).as_deref(), Some("eu-west"));
}

// behavior-contexts-library-dotnet: The library ships one source over the stack's configuration
#[test]
fn the_library_ships_one_source_over_the_stacks_configuration() {
    let api = start(&[STACK, ("FIXTURE_KEYS", "instant"), ("FIXTURE_EARLIER", "Features:Payouts:Instant=earlier"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    let state = api.state();
    assert_eq!(state["sourcesAfterRegistration"].as_array().unwrap().len(), state["sourcesBefore"].as_array().unwrap().len() + 1);
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-dotnet: The shipped source is used for each key with no other registered source
#[test]
fn the_shipped_source_is_used_for_each_key_with_no_other_registered_source() {
    let api = start(&[("FIXTURE_KEYS", "wide"), ("FIXTURE_EARLIER", "Features:Payouts:Instant=a;Features:Payouts:Second=b"), ("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_NAME", "region-flags")]);
    let report = api.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("a"));
    assert_eq!(value_of(&report, "payouts.second").as_deref(), Some("b"));
    assert_eq!(key(&report, REGION)["source"]["name"], "region-flags");
    assert_eq!(api.region()["reads"], 1);
    assert_eq!(api.state()["conditionCalls"], 1);
}

// behavior-contexts-library-dotnet: The source that applies Azimuth's values is added last among the configuration's sources
#[test]
fn the_source_that_applies_azimuths_values_is_added_last_among_the_configurations_sources() {
    let api = start(&[("FIXTURE_EARLIER", "Features:Payouts:Instant=earlier")]);
    let state = api.state();
    let before = strings(&state["sourcesBefore"]);
    let after = strings(&state["sourcesAfterRegistration"]);
    assert!(before.len() >= 2, "the fixture has too few sources to tell an order: {before:?}");
    assert_eq!(after.len(), before.len() + 1, "{after:?}");
    assert_eq!(after[..before.len()], before[..], "an existing source moved or was replaced");
}

// behavior-contexts-library-dotnet: Azimuth's applied value wins over the configuration's other sources
#[test]
fn azimuths_applied_value_wins_over_the_configurations_other_sources() {
    let api = start(&[STACK, ("FIXTURE_EARLIER", "Features:Payouts:Instant=earlier"), ("Features__Payouts__Instant", "fromenv"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("true"));
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-dotnet: A team registers its own source for the keys that a flag service, a database or another store serves
#[test]
fn a_team_registers_its_own_source_for_the_keys_that_a_flag_service_a_database_or_another_store_serves() {
    let api = start(&[STACK, ("FIXTURE_KEYS", "region"), ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    assert_eq!(context_keys(&api.report()), ["payouts.region"]);
    assert_eq!(value_of(&api.report(), REGION).as_deref(), Some("eu-west"));
    assert_eq!(api.setting("payouts.region"), None);
    assert_eq!(api.setting("Features:Payouts:Region"), None);
}

// behavior-contexts-library-dotnet: A team's own source has a read operation and an apply operation
#[test]
fn a_teams_own_source_has_a_read_operation_and_an_apply_operation() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.region":"eu-west"}"#)]);
    api.report();
    let region = api.region();
    assert!(region["reads"].as_u64().unwrap() >= 1, "Read was not called: {region}");
    assert_eq!(region["applies"], 1);
}

// behavior-contexts-library-dotnet: A key with a team's own source is read and applied through it
#[test]
fn a_key_with_a_teams_own_source_is_read_and_applied_through_it() {
    let api = start(&[STACK, ("FIXTURE_KEYS", "wide"), ("FIXTURE_SIDE", "before"), ("AZIMUTH_CONTEXT", r#"{"payouts.sideteam":"after"}"#)]);
    assert_eq!(api.state()["sideTeamApplied"], "after");
    let report = api.report();
    assert_eq!(value_of(&report, "payouts.sideteam").as_deref(), Some("after"));
    assert_eq!(key(&report, "payouts.sideteam")["source"]["name"], "side-store");
    assert_eq!(api.setting("Features:Payouts:Side"), None);
}

// behavior-contexts-library-dotnet: A later source that wins is reported as an override
#[test]
fn a_later_source_that_wins_is_reported_as_an_override() {
    let api = start(&[STACK, ("FIXTURE_LATER", "Features:Payouts:Instant=false"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    let report = api.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("false"));
    assert_eq!(precedence_of(&report, INSTANT), "after azimuth");
}

// behavior-contexts-library-dotnet: The library ships no flag-service integration
#[test]
fn the_library_ships_no_flag_service_integration() {
    let inspection = inspect();
    let flag_words = ["openfeature", "launchdarkly", "unleash", "flagsmith", "split.io", "featuremanagement", "growthbook", "configcat"];
    let mut spoken: Vec<String> = strings(&inspection["names"]);
    spoken.extend(strings(&inspection["references"]));
    spoken.extend(nuspec_dependency_ids());
    let found: Vec<&String> = spoken.iter().filter(|name| flag_words.iter().any(|word| name.to_lowercase().contains(word))).collect();
    assert!(found.is_empty(), "the library integrates a flag service: {found:?}");
}

// behavior-contexts-library-dotnet: The library requires no OpenFeature integration
#[test]
fn the_library_requires_no_openfeature_integration() {
    let dependencies = nuspec_dependency_ids();
    assert!(dependencies.iter().all(|name| !name.to_lowercase().contains("openfeature")), "{dependencies:?}");
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("true"));
}

// ---------------------------------------------------------------- The shipped source over .NET configuration

// behavior-contexts-library-dotnet: The shipped source reads the mapped configuration path through a configuration provider
#[test]
fn the_shipped_source_reads_the_mapped_configuration_path_through_a_configuration_provider() {
    let api = start(&[("Features__Payouts__Instant", "fromenv")]);
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("fromenv"));
    api.ok("/fixture/set?path=Features:Payouts:Instant&value=written");
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("written"));
}

// behavior-contexts-library-dotnet: The shipped source writes the mapped configuration path through a configuration provider
#[test]
fn the_shipped_source_writes_the_mapped_configuration_path_through_a_configuration_provider() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    let state = api.state();
    assert_eq!(state["sourcesAfterRegistration"].as_array().unwrap().len(), state["sourcesBefore"].as_array().unwrap().len() + 1);
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("true"));
    assert!(api.ok("/fixture/envvar?name=Features__Payouts__Instant")["value"].is_null(), "the value was written to the environment");
}

// behavior-contexts-library-dotnet: `api` mapped to `Features:Payouts:Instant` reads and applies `payouts.instant` at that configuration path
#[test]
fn api_mapped_to_features_payouts_instant_reads_and_applies_payouts_instant_at_that_configuration_path() {
    let reading = start(&[("FIXTURE_EARLIER", "Features:Payouts:Instant=false")]);
    assert_eq!(reading.report()["component"], "api");
    assert_eq!(value_of(&reading.report(), INSTANT).as_deref(), Some("false"));
    let applying = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(applying.setting("Features:Payouts:Instant").as_deref(), Some("true"));
    assert_eq!(value_of(&applying.report(), INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-dotnet: Another configuration store of the application needs a registered source of its own
#[test]
fn another_configuration_store_of_the_application_needs_a_registered_source_of_its_own() {
    let api = start(&[("FIXTURE_KEYS", "wide"), ("FIXTURE_SIDE", "x")]);
    let report = api.report();
    assert_eq!(value_of(&report, "payouts.side"), None);
    assert_eq!(value_of(&report, "payouts.sideteam").as_deref(), Some("x"));
}

// ---------------------------------------------------------------- The callback and the report

// behavior-contexts-library-dotnet: The callback returns the report
#[test]
fn the_callback_returns_the_report() {
    let report = start(&[STACK]).report();
    assert_eq!(report["format"], 1);
    assert_eq!(report["component"], "api");
    assert!(report["context"].is_object());
}

// behavior-contexts-library-dotnet: The library reads every mapped key again at each call of the callback
#[test]
fn the_library_reads_every_mapped_key_again_at_each_call_of_the_callback() {
    let api = start(&[("FIXTURE_REGION_VALUE", "eu")]);
    let first = api.report();
    let reads_after_first = api.region()["reads"].as_u64().unwrap();
    let second = api.report();
    let reads_after_second = api.region()["reads"].as_u64().unwrap();
    assert!(reads_after_first >= 1);
    assert!(reads_after_second > reads_after_first, "a second call did not read a team's source again");
    std::thread::sleep(Duration::from_millis(1100));
    let third = api.report();
    for name in [INSTANT, REGION] {
        let read = |report: &Value| utc_seconds(key(report, name)["read"].as_str().unwrap());
        assert!(read(&third) > read(&second), "{name} was not read again");
        assert!(read(&second) >= read(&first));
    }
}

// behavior-contexts-library-dotnet: A value that changed while the component ran is read again at the next call
#[test]
fn a_value_that_changed_while_the_component_ran_is_read_again_at_the_next_call() {
    let api = start(&[("FIXTURE_REGION_VALUE", "eu")]);
    assert_eq!(value_of(&api.report(), REGION).as_deref(), Some("eu"));
    api.ok("/fixture/region?value=us");
    assert_eq!(value_of(&api.report(), REGION).as_deref(), Some("us"));
}

// behavior-contexts-library-dotnet: The report has format `1`
#[test]
fn the_report_has_format_1() {
    assert_eq!(start(&[]).report()["format"], 1);
}

// behavior-contexts-library-dotnet: The report reports exactly its mapped keys
#[test]
fn the_report_reports_exactly_its_mapped_keys() {
    for (keys, expected) in [
        ("default", vec!["payouts.instant", "payouts.region"]),
        ("instant", vec!["payouts.instant"]),
        ("region", vec!["payouts.region"]),
        ("wide", vec!["payouts.instant", "payouts.region", "payouts.second", "payouts.side", "payouts.sideteam"]),
    ] {
        let api = start(&[STACK, ("FIXTURE_KEYS", keys), ("FIXTURE_EARLIER", "Features:Payouts:Unmapped=u")]);
        assert_eq!(context_keys(&api.report()), expected, "mapping {keys}");
    }
}

// behavior-contexts-library-dotnet: The report carries `component`, `applying` and `context`
#[test]
fn the_report_carries_component_applying_and_context() {
    let report = start(&[STACK]).report();
    let mut fields: Vec<&String> = report.as_object().unwrap().keys().collect();
    fields.sort();
    assert_eq!(fields, ["applying", "component", "context", "format"]);
}

// behavior-contexts-library-dotnet: The report carries no identity of its instance and no mark of a launch
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

// behavior-contexts-library-dotnet: `component` is the name from registration
#[test]
fn component_is_the_name_from_registration() {
    assert_eq!(start(&[]).report()["component"], "api");
    assert_eq!(start(&[("FIXTURE_COMPONENT", "website")]).report()["component"], "website");
}

// behavior-contexts-library-dotnet: `context` is an object of mapped product keys
#[test]
fn context_is_an_object_of_mapped_product_keys() {
    let report = start(&[]).report();
    assert!(report["context"].is_object());
    assert_eq!(context_keys(&report), ["payouts.instant", "payouts.region"]);
    assert!(report["context"].get(INSTANT_PATH).is_none(), "the report names a setting and not a product key");
}

// behavior-contexts-library-dotnet: A key's `value` is text, or `null` when absent
#[test]
fn a_keys_value_is_text_or_null_when_absent() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu")]).report();
    assert!(key(&report, REGION)["value"].is_string());
    assert!(key(&report, INSTANT)["value"].is_null());
}

// behavior-contexts-library-dotnet: A key's `source` is `null` when absent
#[test]
fn a_keys_source_is_null_when_absent() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_SOURCE", "none")]).report();
    assert_eq!(key(&report, REGION)["value"], "eu");
    assert!(key(&report, REGION).get("source").is_some_and(Value::is_null), "{report}");
}

// behavior-contexts-library-dotnet: A key's `read` is the UTC time it was read
#[test]
fn a_keys_read_is_the_utc_time_it_was_read() {
    let api = start(&[]);
    let before = now_seconds();
    let report = api.report();
    let after = now_seconds();
    for name in [INSTANT, REGION] {
        let read = utc_seconds(key(&report, name)["read"].as_str().unwrap());
        assert!(read >= before - 2.0 && read <= after + 2.0, "{name} read at {read}, between {before} and {after}");
    }
}

// behavior-contexts-library-dotnet: A source has a `name` and a `precedence`
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

// behavior-contexts-library-dotnet: `before azimuth` says that Azimuth's value would win
#[test]
fn before_azimuth_says_that_azimuths_value_would_win() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_PRECEDENCE", "before azimuth")]).report();
    assert_eq!(precedence_of(&report, REGION), "before azimuth");
}

// behavior-contexts-library-dotnet: `azimuth` says that this is Azimuth's applied value
#[test]
fn azimuth_says_that_this_is_azimuths_applied_value() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true","payouts.region":"eu-west"}"#)]);
    let report = api.report();
    assert_eq!(precedence_of(&report, INSTANT), "azimuth");
    assert_eq!(precedence_of(&report, REGION), "azimuth");
}

// behavior-contexts-library-dotnet: `after azimuth` says that this source wins over Azimuth's value
#[test]
fn after_azimuth_says_that_this_source_wins_over_azimuths_value() {
    let report = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_PRECEDENCE", "after azimuth")]).report();
    assert_eq!(precedence_of(&report, REGION), "after azimuth");
    let later = start(&[STACK, ("FIXTURE_LATER", "Features:Payouts:Instant=false"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]).report();
    assert_eq!(precedence_of(&later, INSTANT), "after azimuth");
}

// behavior-contexts-library-dotnet: An absent value or source is reported as absent
#[test]
fn an_absent_value_or_source_is_reported_as_absent() {
    let no_value = start(&[("FIXTURE_REGION_NAME", "region-flags")]).report();
    assert!(key(&no_value, REGION)["value"].is_null());
    let no_source = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_SOURCE", "none")]).report();
    assert!(key(&no_source, REGION)["source"].is_null());
    assert_eq!(key(&no_source, REGION)["value"], "eu");
}

// behavior-contexts-library-dotnet: The library reports its mappings, not what it thinks the declaration ought to hold
#[test]
fn the_library_reports_its_mappings_not_what_it_thinks_the_declaration_ought_to_hold() {
    let project = decoy_project();
    let api = start_in(&[STACK, ("FIXTURE_KEYS", "region")], &project);
    assert_eq!(context_keys(&api.report()), ["payouts.region"]);
}

// ---------------------------------------------------------------- Cases of the conformance kit for the report of a component

// behavior-contexts-library-dotnet: A mapping reads a value through the callback, for read
#[test]
fn a_mapping_reads_a_value_through_the_callback_for_read() {
    let api = start(&[("FIXTURE_EARLIER", "Features:Payouts:Instant=false")]);
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("false"));
}

// behavior-contexts-library-dotnet: A mapping applies a value, for apply
#[test]
fn a_mapping_applies_a_value_for_apply() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(api.setting(INSTANT_PATH).as_deref(), Some("true"));
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("true"));
}

// behavior-contexts-library-dotnet: The callback gives a report when applying is possible
#[test]
fn the_callback_gives_a_report_when_applying_is_possible() {
    let report = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]).report();
    assert_eq!(report["applying"], "possible");
    assert_eq!(report["component"], "api");
}

// behavior-contexts-library-dotnet: The callback gives a report when applying is not possible
#[test]
fn the_callback_gives_a_report_when_applying_is_not_possible() {
    let report = start(&[("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]).report();
    assert_eq!(report["applying"], "not possible");
    assert_eq!(report["component"], "api");
    assert_eq!(context_keys(&report), ["payouts.instant", "payouts.region"]);
}

// behavior-contexts-library-dotnet: An absent value is reported as absent
#[test]
fn an_absent_value_is_reported_as_absent() {
    let report = start(&[STACK]).report();
    assert!(key(&report, INSTANT)["value"].is_null());
    assert!(key(&report, REGION)["value"].is_null());
}

// behavior-contexts-library-dotnet: A key the launch does not state keeps its source
#[test]
fn a_key_the_launch_does_not_state_keeps_its_source() {
    let api = start(&[STACK, ("FIXTURE_KEYS", "wide"), ("FIXTURE_EARLIER", "Features:Payouts:Second=kept"), ("FIXTURE_REGION_VALUE", "eu"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    let report = api.report();
    assert_eq!(value_of(&report, "payouts.second").as_deref(), Some("kept"));
    assert_ne!(precedence_of(&report, "payouts.second"), "azimuth");
    assert_eq!(value_of(&report, REGION).as_deref(), Some("eu"));
    assert_eq!(api.region()["applies"], 0);
    assert_eq!(api.setting("Features:Payouts:Second").as_deref(), Some("kept"));
}

// behavior-contexts-library-dotnet: A source added later that gives another value is reported with precedence `after azimuth`
#[test]
fn a_source_added_later_that_gives_another_value_is_reported_with_precedence_after_azimuth() {
    let api = start(&[STACK, ("FIXTURE_KEYS", "instant"), ("FIXTURE_LATER", "Features:Payouts:Instant=false"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    let report = api.report();
    assert_eq!(value_of(&report, INSTANT).as_deref(), Some("false"));
    assert_eq!(precedence_of(&report, INSTANT), "after azimuth");
    assert!(key(&report, INSTANT)["source"]["name"].as_str().is_some_and(|name| !name.is_empty()));
}

// behavior-contexts-library-dotnet: A value that changed is read again at the next call
#[test]
fn a_value_that_changed_is_read_again_at_the_next_call() {
    let api = start(&[STACK, ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("true"));
    api.ok("/fixture/set?path=Features:Payouts:Instant&value=false");
    assert_eq!(value_of(&api.report(), INSTANT).as_deref(), Some("false"));
}

// behavior-contexts-library-dotnet: The component name and the report format are checked
#[test]
fn the_component_name_and_the_report_format_are_checked() {
    let report = start(&[STACK]).declared_report();
    assert_eq!(report["component"], "api");
    assert_eq!(report["format"], 1);
}

// behavior-contexts-library-dotnet: The document of the callback is the same document through the fixture's declared path
#[test]
fn the_document_of_the_callback_is_the_same_document_through_the_fixtures_declared_path() {
    let api = start(&[STACK, ("FIXTURE_REGION_VALUE", "eu"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    let (status, _) = api.get("/internal/azimuth");
    assert_eq!(status, 200);
    assert_eq!(without_times(&api.report()), without_times(&api.declared_report()));
}

// behavior-contexts-library-dotnet: A team's own source passes these cases too
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

    let unstated = start(&[STACK, ("FIXTURE_REGION_VALUE", "eu"), ("AZIMUTH_CONTEXT", r#"{"payouts.instant":"true"}"#)]);
    assert_eq!(value_of(&unstated.report(), REGION).as_deref(), Some("eu"));
    assert_eq!(unstated.region()["applies"], 0);

    let changing = start(&[("FIXTURE_REGION_VALUE", "eu")]);
    assert_eq!(value_of(&changing.report(), REGION).as_deref(), Some("eu"));
    changing.ok("/fixture/region?value=us");
    assert_eq!(value_of(&changing.report(), REGION).as_deref(), Some("us"));

    let later = start(&[("FIXTURE_REGION_VALUE", "eu"), ("FIXTURE_REGION_PRECEDENCE", "after azimuth")]);
    assert_eq!(precedence_of(&later.report(), REGION), "after azimuth");
}
