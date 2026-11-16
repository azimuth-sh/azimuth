//! Group "The .NET test adapter" of docs/increments/1.md, and the rows of `azimuth init` that
//! need an installed adapter. These tests start the real adapter against the sample's real xUnit
//! project, so they need the .NET SDK and run only with `cargo test --features dotnet`.
//!
//! The adapter's code is in `adapters/xunit/` of this repository. The tests pack it once into a
//! local folder feed and pin it in each test's own copy of the sample with a tool manifest, as a
//! team pins it, so no registry is involved. Until that folder holds the adapter, every test here
//! fails at the pack.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use azimuth_tests::*;
use serde_json::Value;

/// The package id and the command, which the design makes one name.
const ADAPTER: &str = "azimuth-xunit";
const DOTNET_SUITE: &str = "dotnet/LibraryDesk.Tests";
const SIXTH_LOAN_CHECK: &str = "desk-refuses-a-sixth-loan";
const ON_TIME_CHECK: &str = "desk-charges-nothing-on-time";
const LATE_DAYS_CHECK: &str = "desk-fines-each-late-day";
const LOST_BOOK_CHECK: &str = "desk-charges-for-a-lost-book";
const DOTNET_CHECKS: [&str; 4] = [SIXTH_LOAN_CHECK, ON_TIME_CHECK, LATE_DAYS_CHECK, LOST_BOOK_CHECK];

struct Feed {
    folder: PathBuf,
    package: PathBuf,
    version: String,
}

fn dotnet(dir: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("dotnet");
    command.args(args).current_dir(dir);
    run_with_time_limit(command, Duration::from_secs(600), None)
}

/// Packs `adapters/xunit/` once per test binary. The version gets a suffix of its own for each
/// pack, because the machine's package cache keeps the first package of a version for good, and
/// a repacked adapter under an old version would never reach the tests.
fn feed() -> &'static Feed {
    static FEED: OnceLock<Feed> = OnceLock::new();
    FEED.get_or_init(|| {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../adapters/xunit");
        assert!(source.is_dir(), "the adapter's code is not in {} yet", source.display());
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let version = format!("0.0.0-local.{stamp}");
        let folder = binaries_dir().join(format!("azimuth-xunit-feed-{stamp}"));
        std::fs::create_dir_all(&folder).unwrap();
        let output = dotnet(&source, &["pack", "--configuration", "Release", "--output", folder.to_str().unwrap(), &format!("-p:Version={version}")]);
        output.expect_status(0);
        let package = folder.join(format!("{ADAPTER}.{version}.nupkg"));
        assert!(package.is_file(), "the pack wrote no package {}:\n{}", package.display(), output.text());
        Feed { folder, package, version }
    })
}

/// Pins the packed adapter in the repository with a tool manifest, commits it as a team would,
/// and restores it from the local feed only.
fn pin_adapter(repo: &Repo) {
    let feed = feed();
    let manifest = json!({
        "version": 1,
        "isRoot": true,
        "tools": { ADAPTER: { "version": feed.version, "commands": [ADAPTER], "rollForward": false } }
    });
    repo.write(".config/dotnet-tools.json", &format!("{manifest:#}\n"));
    repo.commit_all("pin the test adapter");
    let config = repo.beside("nuget.config");
    std::fs::write(
        &config,
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<configuration>\n  <packageSources>\n    <clear />\n    <add key=\"local\" value=\"{}\" />\n  </packageSources>\n</configuration>\n",
            feed.folder.display()
        ),
    )
    .unwrap();
    dotnet(repo.root(), &["tool", "restore", "--configfile", config.to_str().unwrap()]).expect_status(0);
}

/// The sample with the adapter pinned.
fn sample() -> Repo {
    let repo = Repo::sample();
    pin_adapter(&repo);
    repo
}

/// Starts the adapter as the core does: in the suite's folder for `run`, at the root for
/// `discover`, with one request on standard input.
fn start_adapter(repo: &Repo, request: &Value) -> Output {
    let dir = match request["suite"].as_str() {
        Some(suite) => repo.path(suite),
        None => repo.root().to_path_buf(),
    };
    let mut command = Command::new("dotnet");
    command.arg(ADAPTER).current_dir(dir);
    // A build needs the machine's own SDK and package cache, so the environment is kept.
    run_with_time_limit(command, Duration::from_secs(600), Some(&request.to_string()))
}

fn run_request(checks: &[&str]) -> Value {
    json!({ "format": "adapter-request-1", "verb": "run", "suite": DOTNET_SUITE, "marker_key": "Check", "checks": checks })
}

fn report(repo: &Repo, request: &Value) -> Value {
    let output = start_adapter(repo, request);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "adapter-report-1", "{document:#}");
    document
}

fn tests_of(report: &Value) -> Vec<Value> {
    report["tests"].as_array().cloned().unwrap_or_default()
}

fn test_named<'a>(tests: &'a [Value], name: &str) -> &'a Value {
    let found: Vec<&Value> = tests.iter().filter(|test| test["name"].as_str().unwrap_or_default().contains(name)).collect();
    assert_eq!(found.len(), 1, "{name} is listed {} times: {tests:#?}", found.len());
    found[0]
}

// behavior: It is a separate command, installed as a local dotnet tool
#[test]
fn the_adapter_is_a_local_dotnet_tool() {
    let repo = sample();
    let output = dotnet(repo.root(), &["tool", "list", "--local"]);
    output.expect_status(0);
    assert!(output.stdout.contains(ADAPTER), "the sample pins no local tool {ADAPTER}:\n{}", output.stdout);
}

// behavior: Its package id and its command are `azimuth-xunit`
#[test]
fn the_package_id_and_the_command_are_one_name() {
    let feed = feed();
    let name = feed.package.file_name().unwrap().to_string_lossy().to_string();
    assert_eq!(name, format!("{ADAPTER}.{}.nupkg", feed.version), "the package id is not {ADAPTER}");
    let repo = sample();
    let output = dotnet(repo.root(), &["tool", "list", "--local"]);
    output.expect_status(0);
    let row = output.stdout.lines().find(|line| line.starts_with(ADAPTER)).unwrap_or_else(|| panic!("{}", output.stdout));
    // The columns are the package id, the version and the commands.
    let words: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(words.first(), Some(&ADAPTER), "package id: {row}");
    assert_eq!(words.get(2), Some(&ADAPTER), "command: {row}");
}

// behavior: It lists the tests of a suite with the Check ids they carry
#[test]
fn the_adapter_lists_tests_with_their_check_ids() {
    let repo = sample();
    let tests = tests_of(&report(&repo, &run_request(&[])));
    for (name, check) in [
        ("LoanLimitTests.Refuses_a_sixth_loan", SIXTH_LOAN_CHECK),
        ("LoanLimitTests.Lends_again_after_a_return", SIXTH_LOAN_CHECK),
        ("FineTests.Charges_nothing_on_the_due_date", ON_TIME_CHECK),
        ("FineTests.Fines_each_late_day", LATE_DAYS_CHECK),
        ("FineTests.Charges_for_a_lost_book", LOST_BOOK_CHECK),
    ] {
        assert_eq!(strings_of(&test_named(&tests, name)["checks"]), vec![check.to_string()], "{name}");
    }
    assert!(!tests.iter().any(|test| test.to_string().contains("Unmarked_failing_test")), "an unmarked test is listed");
}

// behavior: It builds the suite, starts the runner and reads the runner's report
#[test]
fn the_adapter_runs_the_real_tests() {
    let repo = sample();
    let document = report(&repo, &run_request(&DOTNET_CHECKS));
    assert_eq!(document["build"], "ok");
    let tests = tests_of(&document);
    for (name, outcome) in [
        ("Refuses_a_sixth_loan", "passed"),
        ("Lends_again_after_a_return", "passed"),
        ("Charges_nothing_on_the_due_date", "passed"),
        ("Fines_each_late_day", "failed"),
        ("Charges_for_a_lost_book", "skipped"),
    ] {
        assert_eq!(test_named(&tests, name)["outcome"], outcome, "{name}");
    }
}

// behavior: It passes the ids of the owed Checks to the runner's own filter
#[test]
fn the_adapter_runs_only_the_asked_checks() {
    let repo = sample();
    let tests = tests_of(&report(&repo, &run_request(&[ON_TIME_CHECK])));
    assert_eq!(test_named(&tests, "Charges_nothing_on_the_due_date")["outcome"], "passed");
    for name in ["Refuses_a_sixth_loan", "Lends_again_after_a_return", "Fines_each_late_day", "Charges_for_a_lost_book"] {
        assert_eq!(test_named(&tests, name)["outcome"], "not_run", "{name}");
    }
}

// behavior: It reports its own version
#[test]
fn the_adapter_reports_its_version() {
    let repo = sample();
    let document = report(&repo, &run_request(&[]));
    assert!(!document["adapter"]["name"].as_str().unwrap_or_default().is_empty(), "{document:#}");
    assert!(!document["adapter"]["version"].as_str().unwrap_or_default().is_empty(), "{document:#}");
}

// behavior: A test project references nothing from Azimuth
#[test]
fn the_test_project_stays_free_of_azimuth() {
    let repo = sample();
    report(&repo, &run_request(&DOTNET_CHECKS));
    assert_eq!(repo.git(&["status", "--porcelain", "--untracked-files=no"]), "", "the adapter edited tracked files");
    for file in files_under(&repo.path("dotnet")) {
        let relative = file.strip_prefix(repo.root()).unwrap().to_string_lossy().to_string();
        if relative.contains("/bin/") || relative.contains("/obj/") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap_or_default().to_lowercase();
        assert!(!text.contains("azimuth"), "{relative} references Azimuth");
    }
}

// behavior: The adapter writes one report document to standard output, and the runner's own output goes to standard error
#[test]
fn the_report_is_alone_on_standard_output() {
    let repo = sample();
    let output = start_adapter(&repo, &run_request(&DOTNET_CHECKS));
    output.expect_status(0);
    output.json();
    assert!(!output.stderr.trim().is_empty(), "the runner said nothing on standard error");
}

// behavior: The request is `adapter-request-1` and the report `adapter-report-1`
#[test]
fn the_adapter_answers_with_adapter_report_1() {
    let repo = sample();
    assert_eq!(report(&repo, &run_request(&[]))["format"], "adapter-report-1");
}

fn discover_request() -> Value {
    json!({ "format": "adapter-request-1", "verb": "discover", "folder": "." })
}

fn discovered_paths(report: &Value) -> Vec<String> {
    report["projects"]
        .as_array()
        .unwrap_or_else(|| panic!("no `projects`: {report:#}"))
        .iter()
        .map(|project| project["path"].as_str().unwrap_or_else(|| panic!("a project has no path: {report:#}")).trim_end_matches('/').to_string())
        .collect()
}

// behavior: The verb `discover` reports the test projects under a folder
#[test]
fn discover_finds_the_xunit_project() {
    let repo = sample();
    repo.write("checks/Shelf.Tests/Shelf.Tests.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />\n");
    let document = report(&repo, &discover_request());
    assert_eq!(discovered_paths(&document), vec![DOTNET_SUITE.to_string()], "{document:#}");
}

// behavior: The `discover` request holds `format`, `verb` and `folder`
#[test]
fn the_adapter_answers_a_discover_request_of_three_fields() {
    let repo = sample();
    let request = discover_request();
    assert_eq!(request.as_object().unwrap().len(), 3);
    report(&repo, &request);
}

/// `azimuth init` with `dotnet` reached through a recorder, so the discover request the core wrote
/// and the folder it started the adapter in can be read back. Returns the recorded starts that
/// carried a `discover` request.
fn init_through_a_recorder() -> (Repo, Vec<Value>) {
    let repo = Repo::sample_before_init("main");
    pin_adapter(&repo);
    let real_dotnet = String::from_utf8(Command::new("which").arg("dotnet").output().unwrap().stdout).unwrap().trim().to_string();
    let front = repo.beside("recorder");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::copy(binaries_dir().join("stdin-recorder"), front.join("dotnet")).unwrap();
    let home = std::env::var("HOME").unwrap_or_default();
    let path = format!("{}:{}:{}", front.display(), binaries_dir().display(), std::env::var("PATH").unwrap_or_default());
    let output = repo.azimuth_with_env(&["init"], &[("HOME", &home), ("PATH", &path), ("RECORDER_TARGET", &real_dotnet)]);
    output.expect_status(0);
    let starts: Vec<Value> = repo
        .log_entries()
        .into_iter()
        .filter(|entry| entry["program"] == "stdin-recorder")
        .filter(|entry| {
            serde_json::from_str::<Value>(entry["stdin"].as_str().unwrap_or_default()).is_ok_and(|request| request["verb"] == "discover")
        })
        .collect();
    assert_eq!(starts.len(), 1, "{starts:#?}");
    (repo, starts)
}

// behavior: The `discover` request holds `format`, `verb` and `folder`
#[test]
fn init_sends_a_discover_request_of_three_fields() {
    let (_, starts) = init_through_a_recorder();
    let request: Value = serde_json::from_str(starts[0]["stdin"].as_str().unwrap()).unwrap();
    let fields: std::collections::BTreeSet<String> = request.as_object().unwrap().keys().cloned().collect();
    assert_eq!(fields, set_of(&["format", "verb", "folder"]), "{request:#}");
    assert_eq!(request["format"], "adapter-request-1");
}

// behavior: For `discover` it starts the adapter at the root of the repository
#[test]
fn init_starts_discover_at_the_root() {
    let (repo, starts) = init_through_a_recorder();
    let cwd = std::fs::canonicalize(starts[0]["cwd"].as_str().unwrap()).unwrap();
    assert_eq!(cwd, std::fs::canonicalize(repo.root()).unwrap(), "{:#}", starts[0]);
}

// behavior: The `discover` report holds `format`, `adapter` and `projects`, each with a `path`
#[test]
fn the_discover_report_lists_projects_with_paths() {
    let repo = sample();
    let document = report(&repo, &discover_request());
    for field in ["format", "adapter", "projects"] {
        assert!(document.get(field).is_some(), "no `{field}`: {document:#}");
    }
    assert!(!document["adapter"]["name"].as_str().unwrap_or_default().is_empty(), "{document:#}");
    assert!(discovered_paths(&document).contains(&DOTNET_SUITE.to_string()), "{document:#}");
}

// behavior: The verb `run` reports every marked test of the suite, also the ones not asked for
#[test]
fn run_reports_the_tests_not_asked_for() {
    let repo = sample();
    let tests = tests_of(&report(&repo, &run_request(&[SIXTH_LOAN_CHECK])));
    assert_eq!(tests.len(), 5, "{tests:#?}");
}

// behavior: A test with many data rows is one entry, and it failed if any row failed
#[test]
fn a_theory_is_one_failed_entry() {
    let repo = sample();
    let tests = tests_of(&report(&repo, &run_request(&[LATE_DAYS_CHECK])));
    assert_eq!(test_named(&tests, "Fines_each_late_day")["outcome"], "failed");
}

// behavior: Each test has the outcome `passed`, `failed`, `skipped` or `not_run`, and a duration
#[test]
fn every_test_has_an_outcome_and_a_duration() {
    let repo = sample();
    for test in tests_of(&report(&repo, &run_request(&[SIXTH_LOAN_CHECK, LATE_DAYS_CHECK, LOST_BOOK_CHECK]))) {
        let outcome = test["outcome"].as_str().unwrap_or_default();
        assert!(["passed", "failed", "skipped", "not_run"].contains(&outcome), "{test:#}");
        assert!(test["duration_ms"].is_u64(), "{test:#}");
    }
}

// behavior: A build that fails gives a report with `"build": "failed"` and no tests
#[test]
fn a_failed_build_is_a_report() {
    let repo = sample();
    repo.append("dotnet/LibraryDesk/Desk.cs", "\nthis line does not compile\n");
    let document = report(&repo, &run_request(&DOTNET_CHECKS));
    assert_eq!(document["build"], "failed");
    assert_eq!(count(&document["tests"]), 0);
}

// behavior: The adapter ends with status 0 when it wrote a report, also when tests failed
#[test]
fn failed_tests_still_end_with_zero() {
    let repo = sample();
    let output = start_adapter(&repo, &run_request(&[LATE_DAYS_CHECK]));
    output.expect_status(0);
    let tests = tests_of(&output.json());
    assert_eq!(test_named(&tests, "Fines_each_late_day")["outcome"], "failed");
}

/// `azimuth init` on a machine where the .NET adapter is installed.
fn init_with_adapter() -> (Repo, toml::Table) {
    let repo = Repo::sample_before_init("main");
    pin_adapter(&repo);
    repo.write("checks/Shelf.Tests/Shelf.Tests.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />\n");
    repo.commit_all("a project that only looks like a test project");
    let home = std::env::var("HOME").unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_default();
    let output = repo.azimuth_with_env(&["init"], &[("HOME", &home), ("PATH", &format!("{}:{path}", binaries_dir().display()))]);
    output.expect_status(0);
    let project = parse_toml(&repo.read(PROJECT_FILE));
    (repo, project)
}

fn suites(project: &toml::Table) -> Vec<toml::Table> {
    project
        .get("suite")
        .and_then(|value| value.as_array())
        .map(|items| items.iter().filter_map(|item| item.as_table().cloned()).collect())
        .unwrap_or_default()
}

// behavior: It lists as suites the test projects that the adapter finds
#[test]
fn init_lists_the_xunit_project_as_a_suite() {
    let (_, project) = init_with_adapter();
    let suites = suites(&project);
    assert_eq!(suites.len(), 1, "{project:?}");
    assert_eq!(suites[0]["path"].as_str().map(|path| path.trim_end_matches('/')), Some(DOTNET_SUITE));
    assert_eq!(suites[0]["area"].as_str(), project["area"][0]["name"].as_str());
}

// behavior: It writes the marker form of each suite whose runner it knows
#[test]
fn init_writes_the_xunit_marker_form() {
    let (_, project) = init_with_adapter();
    let suites = suites(&project);
    assert_eq!(suites[0]["marker"].as_str(), Some("[Trait(\"Check\", \"<id>\")]"), "{project:?}");
}

// behavior: `azimuth init` writes no `marker_key`
#[test]
fn init_writes_no_marker_key_for_the_xunit_suite() {
    let (repo, project) = init_with_adapter();
    for suite in suites(&project) {
        assert!(!suite.contains_key("marker_key"), "{project:?}");
    }
    assert!(!repo.read(PROJECT_FILE).contains("marker_key"));
}

// behavior: It never lists a suite that no adapter reported
#[test]
fn init_lists_only_what_the_adapter_reported() {
    let (_, project) = init_with_adapter();
    for suite in suites(&project) {
        assert!(!suite["path"].as_str().unwrap_or_default().contains("Shelf"), "{project:?}");
        assert!(!suite["path"].as_str().unwrap_or_default().contains("suites/desk-checks"), "{project:?}");
    }
}
