//! Group "The .NET test adapter" of docs/increments/1.md, and the rows of `azimuth init` that
//! need an installed adapter. These tests start the real adapter against the sample's real xUnit
//! project, so they need the .NET SDK and run only with `cargo test --features dotnet`.
//!
//! The design names the adapter's command only by a placeholder, `dotnet azimuth-xunit`. The
//! tests use that command unless `AZIMUTH_DOTNET_ADAPTER` names another one, as a command line
//! split on spaces.

use std::process::Command;
use std::time::Duration;

use azimuth_tests::*;
use serde_json::Value;

const DOTNET_SUITE: &str = "dotnet/LibraryDesk.Tests";
const SIXTH_LOAN_CHECK: &str = "desk-refuses-a-sixth-loan";
const ON_TIME_CHECK: &str = "desk-charges-nothing-on-time";
const LATE_DAYS_CHECK: &str = "desk-fines-each-late-day";
const LOST_BOOK_CHECK: &str = "desk-charges-for-a-lost-book";
const DOTNET_CHECKS: [&str; 4] = [SIXTH_LOAN_CHECK, ON_TIME_CHECK, LATE_DAYS_CHECK, LOST_BOOK_CHECK];

fn adapter_command() -> Vec<String> {
    std::env::var("AZIMUTH_DOTNET_ADAPTER")
        .unwrap_or_else(|_| "dotnet azimuth-xunit".to_string())
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Starts the adapter in the sample with one request on standard input, as the core does.
fn start_adapter(repo: &Repo, request: &Value) -> Output {
    let command_line = adapter_command();
    let mut command = Command::new(&command_line[0]);
    command.args(&command_line[1..]).current_dir(repo.root());
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
    let command_line = adapter_command();
    assert_eq!(command_line[0], "dotnet", "the adapter is not started through dotnet: {command_line:?}");
    let repo = Repo::sample();
    let mut command = Command::new("dotnet");
    command.args(["tool", "list", "--local"]).current_dir(repo.root());
    let output = run_with_time_limit(command, Duration::from_secs(120), None);
    output.expect_status(0);
    assert!(output.stdout.contains(&command_line[1]), "the sample pins no local tool {}:\n{}", command_line[1], output.stdout);
}

// behavior: It lists the tests of a suite with the Check ids they carry
#[test]
fn the_adapter_lists_tests_with_their_check_ids() {
    let repo = Repo::sample();
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
    let repo = Repo::sample();
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
    let repo = Repo::sample();
    let tests = tests_of(&report(&repo, &run_request(&[ON_TIME_CHECK])));
    assert_eq!(test_named(&tests, "Charges_nothing_on_the_due_date")["outcome"], "passed");
    for name in ["Refuses_a_sixth_loan", "Lends_again_after_a_return", "Fines_each_late_day", "Charges_for_a_lost_book"] {
        assert_eq!(test_named(&tests, name)["outcome"], "not_run", "{name}");
    }
}

// behavior: It reports its own version
#[test]
fn the_adapter_reports_its_version() {
    let repo = Repo::sample();
    let document = report(&repo, &run_request(&[]));
    assert!(!document["adapter"]["name"].as_str().unwrap_or_default().is_empty(), "{document:#}");
    assert!(!document["adapter"]["version"].as_str().unwrap_or_default().is_empty(), "{document:#}");
}

// behavior: A test project references nothing from Azimuth
#[test]
fn the_test_project_stays_free_of_azimuth() {
    let repo = Repo::sample();
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
    let repo = Repo::sample();
    let output = start_adapter(&repo, &run_request(&DOTNET_CHECKS));
    output.expect_status(0);
    output.json();
    assert!(!output.stderr.trim().is_empty(), "the runner said nothing on standard error");
}

// behavior: The request is `adapter-request-1` and the report `adapter-report-1`
#[test]
fn the_adapter_answers_with_adapter_report_1() {
    let repo = Repo::sample();
    assert_eq!(report(&repo, &run_request(&[]))["format"], "adapter-report-1");
}

// behavior: The verb `discover` reports the test projects under a folder
#[test]
fn discover_finds_the_xunit_project() {
    let repo = Repo::sample();
    repo.write("checks/Shelf.Tests/Shelf.Tests.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />\n");
    let document = report(&repo, &json!({ "format": "adapter-request-1", "verb": "discover", "folder": "." }));
    let strings = all_strings(&document);
    assert!(strings.iter().any(|text| text.trim_end_matches('/') == DOTNET_SUITE), "{document:#}");
    assert!(!strings.iter().any(|text| text.trim_end_matches('/') == "dotnet/LibraryDesk"), "a product project was listed: {document:#}");
    assert!(!strings.iter().any(|text| text.contains("Shelf.Tests")), "a project without xUnit was listed: {document:#}");
}

// behavior: The verb `run` reports every marked test of the suite, also the ones not asked for
#[test]
fn run_reports_the_tests_not_asked_for() {
    let repo = Repo::sample();
    let tests = tests_of(&report(&repo, &run_request(&[SIXTH_LOAN_CHECK])));
    assert_eq!(tests.len(), 5, "{tests:#?}");
}

// behavior: A test with many data rows is one entry, and it failed if any row failed
#[test]
fn a_theory_is_one_failed_entry() {
    let repo = Repo::sample();
    let tests = tests_of(&report(&repo, &run_request(&[LATE_DAYS_CHECK])));
    assert_eq!(test_named(&tests, "Fines_each_late_day")["outcome"], "failed");
}

// behavior: Each test has the outcome `passed`, `failed`, `skipped` or `not_run`, and a duration
#[test]
fn every_test_has_an_outcome_and_a_duration() {
    let repo = Repo::sample();
    for test in tests_of(&report(&repo, &run_request(&[SIXTH_LOAN_CHECK, LATE_DAYS_CHECK, LOST_BOOK_CHECK]))) {
        let outcome = test["outcome"].as_str().unwrap_or_default();
        assert!(["passed", "failed", "skipped", "not_run"].contains(&outcome), "{test:#}");
        assert!(test["duration_ms"].is_u64(), "{test:#}");
    }
}

// behavior: A build that fails gives a report with `"build": "failed"` and no tests
#[test]
fn a_failed_build_is_a_report() {
    let repo = Repo::sample();
    repo.append("dotnet/LibraryDesk/Desk.cs", "\nthis line does not compile\n");
    let document = report(&repo, &run_request(&DOTNET_CHECKS));
    assert_eq!(document["build"], "failed");
    assert_eq!(count(&document["tests"]), 0);
}

// behavior: The adapter ends with status 0 when it wrote a report, also when tests failed
#[test]
fn failed_tests_still_end_with_zero() {
    let repo = Repo::sample();
    let output = start_adapter(&repo, &run_request(&[LATE_DAYS_CHECK]));
    output.expect_status(0);
    let tests = tests_of(&output.json());
    assert_eq!(test_named(&tests, "Fines_each_late_day")["outcome"], "failed");
}

/// `azimuth init` on a machine where the .NET adapter is installed.
fn init_with_adapter() -> (Repo, toml::Table) {
    let repo = Repo::sample_before_init("main");
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

// behavior: It never lists a suite that no adapter reported
#[test]
fn init_lists_only_what_the_adapter_reported() {
    let (_, project) = init_with_adapter();
    for suite in suites(&project) {
        assert!(!suite["path"].as_str().unwrap_or_default().contains("Shelf"), "{project:?}");
        assert!(!suite["path"].as_str().unwrap_or_default().contains("suites/desk-checks"), "{project:?}");
    }
}
