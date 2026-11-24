//! Group "`azimuth run`" of docs/increments/1.md.

use std::collections::BTreeSet;
use std::fs;

use std::time::{Duration, Instant};

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
use contexts_conditions::*;
#[path = "support/contexts_launches.rs"]
mod contexts_launches;
use contexts_launches::{all_launch_checks, applied, conditions_of, four_launches, instant_true_with, launches, proven, INSTANT_CHECK};

fn ids(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|id| id.to_string()).collect()
}

fn owed_everywhere(document: &serde_json::Value) {
    for check in ALL_CHECKS {
        assert_eq!(state_of(document, check), "owed", "{document:#}");
    }
}

fn no_results(repo: &Repo) {
    assert!(repo.records().is_empty(), "results were written: {:#?}", repo.records());
}

/// Every Check of the document is owed with the reason `no result` and a `detail` that names a
/// cause. Returns the details, so a test can compare the causes of two launches.
fn no_result_everywhere(document: &serde_json::Value) -> BTreeSet<String> {
    let mut details = BTreeSet::new();
    for check in ALL_CHECKS {
        let entry = &checks_by_id(document)[check];
        assert_eq!(entry["state"], "owed", "{document:#}");
        assert_eq!(entry["reason"], "no result", "{document:#}");
        let detail = detail_of_entry(entry);
        assert!(!detail.trim().is_empty(), "the detail names no cause: {document:#}");
        details.insert(detail);
    }
    details
}

fn reuse_never(repo: &Repo) {
    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never, the desk reads today's date\n",
    );
}

// behavior: It starts only the adapter command named in the project file and downloads nothing
#[test]
fn run_starts_the_named_adapter_command() {
    let mut repo = Repo::sample();
    let bin = repo.beside("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::copy(binaries_dir().join("text-suite-adapter"), bin.join("desk-runner")).unwrap();
    let path = format!("{}:{}", bin.display(), repo.path_without("dotnet"));
    repo.set_env("PATH", &path);
    repo.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"desk-runner\"");
    let closed = "http://127.0.0.1:9";
    let output = repo.azimuth_with_env(&["run", "--json"], &[("https_proxy", closed), ("HTTPS_PROXY", closed), ("ALL_PROXY", closed)]);
    output.expect_status(0);
    let starts = repo.adapter_starts();
    assert!(!starts.is_empty());
    for start in starts {
        assert!(start["argv0"].as_str().unwrap_or_default().ends_with("desk-runner"), "{start:#}");
    }
    assert!(files_under(&repo.beside("home")).is_empty(), "run wrote into the home folder");
}

// behavior: With the adapter missing, it runs nothing and says which command is missing and how to install it
#[test]
fn run_names_a_missing_adapter() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"desk-adapter-that-is-not-installed\"");
    let output = repo.azimuth(&["run"]);
    output.expect_status(1);
    assert!(output.text().contains("desk-adapter-that-is-not-installed"), "{}", output.text());
    assert!(repo.adapter_starts().is_empty());
    no_results(&repo);
    let document = repo.run_json(&[], 1);
    owed_everywhere(&document);
}

// behavior: The message about a missing adapter at a launch prints the same line, with the CLI's own version
#[test]
fn a_missing_dotnet_adapter_is_named_with_the_versioned_line() {
    let mut repo = Repo::sample();
    repo.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"dotnet azimuth-xunit\"");
    repo.set_env("PATH", &repo.path_without("dotnet"));
    let output = repo.azimuth(&["run"]);
    output.expect_status(1);
    let text = output.text();
    let expected = format!("{XUNIT_INSTALL} --version {}", cli_version());
    let lines = xunit_install_lines(&text);
    assert!(!lines.is_empty(), "the launch prints no install line of the .NET adapter:\n{text}");
    for line in lines {
        assert_eq!(line, expected, "the launch prints another install line:\n{text}");
    }
}

// behavior: It computes each Check's fingerprint before it starts anything
#[test]
fn an_input_changed_by_the_proof_gives_no_results() {
    let repo = Repo::sample();
    let target = repo.path("desk/returns.txt").display().to_string();
    repo.need_facility(&format!("edit {target}"));
    let document = repo.run_json(&[], 1);
    assert_eq!(repo.probe_starts().len(), 1, "the proof ran once");
    no_result_everywhere(&document);
    no_results(&repo);
}

// behavior: It proves each declared facility of a suite before the launch
#[test]
fn the_facility_is_proven_before_the_launch() {
    let repo = Repo::sample();
    repo.need_facility("present");
    repo.run_json(&[], 0);
    let order: Vec<String> = repo.log_entries().iter().map(|entry| entry["program"].as_str().unwrap().to_string()).collect();
    assert_eq!(order.first().map(String::as_str), Some("facility-probe"), "{order:?}");
    assert!(order.iter().any(|program| program == "text-suite-adapter"), "{order:?}");
}

// behavior: It proves a facility by running the proof command that the project file defines for it
#[test]
fn the_proof_is_the_command_of_the_definition() {
    let repo = Repo::sample();
    repo.need_facility("present");
    repo.run_json(&[], 0);
    let probes = repo.probe_starts();
    assert_eq!(probes.len(), 1, "{probes:#?}");
    assert_eq!(probes[0]["args"], json!(["present"]));
}

fn cannot_run_here(repo: &Repo) {
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "owed", "{document:#}");
        assert_eq!(reason_of(&document, check), "cannot run here", "{document:#}");
    }
    assert!(repo.adapter_runs().is_empty(), "the suite was launched without its facility");
}

// behavior: A proof that fails, hangs past its time limit or has no definition means "cannot run here"
#[test]
fn a_failing_proof_means_cannot_run_here() {
    let repo = Repo::sample();
    repo.need_facility("absent");
    cannot_run_here(&repo);
}

// behavior: A proof that fails, hangs past its time limit or has no definition means "cannot run here"
#[test]
fn a_hanging_proof_means_cannot_run_here() {
    let repo = Repo::sample();
    repo.need_facility("hang");
    let started = Instant::now();
    cannot_run_here(&repo);
    assert!(started.elapsed() < Duration::from_secs(60), "the run waited {:?} for a hung proof", started.elapsed());
}

// behavior: A proof that fails, hangs past its time limit or has no definition means "cannot run here"
#[test]
fn a_facility_without_a_definition_means_cannot_run_here() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "needs = []", "needs = [\"shelf-scanner\"]");
    cannot_run_here(&repo);
}

// behavior: No flag and no setting marks a facility as present
#[test]
fn no_flag_marks_a_facility_present() {
    let repo = Repo::sample();
    repo.need_facility("absent");
    for flag in [&["run", "--facility", "shelf-scanner"][..], &["run", "--present", "shelf-scanner"][..], &["run", "--assume-facilities"][..]] {
        repo.azimuth(flag).expect_status(2);
    }
    assert!(repo.adapter_runs().is_empty());
}

// behavior: No flag and no setting marks a facility as present
#[test]
fn no_setting_marks_a_facility_present() {
    let repo = Repo::sample();
    repo.need_facility("absent");
    repo.append(PROJECT_FILE, "present = true\n");
    repo.expect_reading_error(Some(PROJECT_FILE), Some(repo.line_of(PROJECT_FILE, "present = true")));
}

// behavior: With a facility missing, the suite's Checks are owed with the reason "cannot run here" and none is failed
#[test]
fn a_missing_facility_is_never_a_fail() {
    let repo = Repo::sample();
    repo.need_facility("absent");
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    cannot_run_here(&repo);
}

// behavior: It builds and starts the test process through the adapter
#[test]
fn the_launch_goes_through_the_adapter() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let runs = repo.adapter_runs();
    assert_eq!(runs.len(), 1, "one launch for one suite: {runs:#?}");
    assert_eq!(runs[0]["request"]["suite"], "suites/desk-checks");
    assert_eq!(runs[0]["request"]["format"], "adapter-request-1");
    assert_eq!(repo.tests_ran().len(), 5, "{:?}", repo.tests_ran());
}

// behavior: It asks the runner only for the tests of owed Checks
#[test]
fn the_launch_asks_only_for_owed_checks() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.forget(LATE_RETURN);
    repo.clear_log();
    let document = repo.run_json(&[], 0);
    assert_eq!(repo.requested_checks(), ids(&[LATE_RETURN]));
    assert_eq!(state_of(&document, LATE_RETURN), "passed");
}

// behavior: It never starts a test that has no marker
#[test]
fn an_unmarked_test_never_runs() {
    let repo = Repo::sample();
    repo.write("suites/desk-checks/catalog.test", "# A test nobody marked. It fails so that a leak would show.\noutcome: failed\n");
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
    assert!(!repo.tests_ran().contains("catalog"), "{:?}", repo.tests_ran());
    let requested = repo.requested_checks();
    assert!(requested.is_subset(&ids(&ALL_CHECKS)), "{requested:?}");
}

// behavior: A Check with a valid result is not run
#[test]
fn a_kept_check_is_not_run() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.forget(SIXTH_LOAN);
    repo.clear_log();
    let document = repo.run_json(&[], 0);
    for check in [RETURNED_BOOK, LATE_RETURN, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, check), "kept", "{document:#}");
        assert!(!repo.requested_checks().contains(check));
    }
}

// behavior: A second run on an unchanged tree runs nothing
#[test]
fn a_second_run_runs_nothing() {
    let repo = Repo::sample();
    let first = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&first, check), "passed", "{first:#}");
    }
    repo.clear_log();
    let second = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&second, check), "kept", "{second:#}");
    }
    assert!(repo.adapter_starts().is_empty(), "the adapter was started on an unchanged tree");
}

fn known_fail() -> Repo {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let first = repo.run_json(&[], 1);
    assert_eq!(state_of(&first, LATE_RETURN), "failed");
    repo.clear_log();
    repo
}

// behavior: A Check that failed on the current fingerprint is not run again on a developer's machine
#[test]
fn a_known_fail_is_not_run_again_locally() {
    let repo = known_fail();
    let second = repo.run_json(&[], 1);
    assert_eq!(state_of(&second, LATE_RETURN), "owed", "{second:#}");
    assert_eq!(reason_of(&second, LATE_RETURN), "failed on these inputs", "{second:#}");
    assert!(repo.adapter_runs().is_empty());
}

// behavior: A Check with `Reuse: never` is not run on a developer's machine and is shown as "runs in CI"
#[test]
fn a_never_reused_check_runs_in_ci_only() {
    let repo = Repo::sample();
    reuse_never(&repo);
    let document = repo.run_json(&[], 0);
    assert_eq!(state_of(&document, ON_TIME_RETURN), "owed", "{document:#}");
    assert_eq!(reason_of(&document, ON_TIME_RETURN), "runs in CI", "{document:#}");
    assert!(!repo.requested_checks().contains(ON_TIME_RETURN));
    assert!(!repo.tests_ran().iter().any(|test| test.starts_with("free-return")));
}

// behavior: A marker on a class enrolls every test of the class
#[test]
fn a_file_level_marker_enrolls_every_test_of_the_file() {
    let repo = Repo::sample();
    let passing = repo.run_json(&[], 0);
    assert_eq!(state_of(&passing, ON_TIME_RETURN), "passed");
    assert!(repo.tests_ran().contains("free-return.returned-on-due-date"));
    assert!(repo.tests_ran().contains("free-return.returned-early"));

    repo.replace(FREE_RETURN_TESTS, "test: returned-early\noutcome: passed", "test: returned-early\noutcome: failed");
    let failing = repo.run_json(&[], 1);
    assert_eq!(state_of(&failing, ON_TIME_RETURN), "failed", "{failing:#}");
    assert_eq!(strings_of(&checks_by_id(&failing)[ON_TIME_RETURN]["failing_tests"]), vec!["free-return.returned-early"]);
}

/// The marker of `late-return-is-fined` disappears from source, and the adapter still lists a
/// test that carries the id, as a trait built from a constant would.
fn listed_but_not_in_source(repo: &Repo) {
    repo.write(LATE_FINE_TESTS, "# The marker is built at run time, so a scan of source cannot see it.\nadapter: lists late-fine-from-constant late-return-is-fined\noutcome: passed\n");
}

// behavior: It gives no result when the tests run differ from the tests listed
#[test]
fn a_report_that_differs_from_source_gives_no_result() {
    let repo = Repo::sample();
    listed_but_not_in_source(&repo);
    let document = repo.run_json(&[], 1);
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    assert_eq!(reason_of(&document, LATE_RETURN), "no result", "{document:#}");
    assert!(repo.results_of(LATE_RETURN).is_empty(), "a result was written");
}

// behavior: A marker in source that matches no listed test is a finding, and that Check gets no result
#[test]
fn a_marker_on_a_helper_gives_no_result() {
    let repo = Repo::sample();
    repo.write(
        "suites/desk-checks/fine-calendar.test",
        "kind: helper\n# A helper the fine tests read. It is not a test, and the marker on it is a mistake.\n# Check: late-return-is-fined\n",
    );
    let document = repo.run_json(&[], 1);
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    assert_eq!(reason_of(&document, LATE_RETURN), "no result", "{document:#}");
    for check in [SIXTH_LOAN, RETURNED_BOOK, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
    assert!(repo.results_of(LATE_RETURN).is_empty(), "a result was written");
    let commit = repo.azimuth(&["run", "--lane", "commit", "--json"]);
    commit.expect_status(1);
    assert!(count(&commit.json()["findings"]) >= 1, "the commit lane shows no finding");
}

// behavior: A listed test whose Check id appears nowhere in source is a finding, and that Check gets no result
#[test]
fn a_listed_test_without_a_marker_in_source_gives_no_result() {
    let repo = Repo::sample();
    listed_but_not_in_source(&repo);
    let document = repo.run_json(&[], 1);
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    assert_eq!(reason_of(&document, LATE_RETURN), "no result", "{document:#}");
    assert!(repo.results_of(LATE_RETURN).is_empty());
    let commit = repo.azimuth(&["run", "--lane", "commit", "--json"]);
    commit.expect_status(1);
    let findings = commit.json()["findings"].clone();
    assert!(findings.to_string().contains(LATE_RETURN), "{findings:#}");
}

// behavior: When the suite does not build, no Check of the suite gets a result
#[test]
fn a_suite_that_does_not_build_gives_no_results() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: build-fails\n");
    let document = repo.run_json(&[], 1);
    no_result_everywhere(&document);
    assert_eq!(document["suites"][0]["launch"], "build failed", "{document:#}");
    no_results(&repo);
}

// behavior: A Check fails when any of its marked tests failed
#[test]
fn one_failed_test_fails_its_check() {
    let repo = Repo::sample();
    repo.replace(LOAN_LIMIT_TESTS, "# Check: sixth-loan-is-refused\noutcome: passed", "# Check: sixth-loan-is-refused\noutcome: failed");
    let document = repo.run_json(&[], 1);
    assert_eq!(state_of(&document, SIXTH_LOAN), "failed");
    assert_eq!(strings_of(&checks_by_id(&document)[SIXTH_LOAN]["failing_tests"]), vec!["loan-limit.refuses-sixth-loan"]);
    assert_eq!(state_of(&document, RETURNED_BOOK), "passed");
}

// behavior: A Check gets no result when a marked test was skipped or not run and none failed
#[test]
fn a_skipped_test_gives_no_result() {
    for outcome in ["skipped", "not_run"] {
        let repo = Repo::sample();
        repo.replace(FREE_RETURN_TESTS, "test: returned-early\noutcome: passed", &format!("test: returned-early\noutcome: {outcome}"));
        let document = repo.run_json(&[], 1);
        assert_eq!(state_of(&document, ON_TIME_RETURN), "owed", "{outcome}: {document:#}");
        assert_eq!(reason_of(&document, ON_TIME_RETURN), "no result", "{outcome}: {document:#}");
        assert!(repo.results_of(ON_TIME_RETURN).is_empty(), "{outcome}: a result was written");
    }
}

// behavior: A Check passes only when every marked test ran and passed
#[test]
fn every_marked_test_passing_passes_the_check() {
    let repo = Repo::sample();
    let document = repo.run_json(&[], 0);
    assert_eq!(state_of(&document, ON_TIME_RETURN), "passed");
    assert_eq!(count(&checks_by_id(&document)[ON_TIME_RETURN]["failing_tests"]), 0);

    let mixed = Repo::sample();
    mixed.replace(FREE_RETURN_TESTS, "test: returned-early\noutcome: passed", "test: returned-early\noutcome: skipped");
    let document = mixed.run_json(&[], 1);
    assert_eq!(state_of(&document, ON_TIME_RETURN), "owed", "one test passed and one was skipped: {document:#}");
}

// behavior: It hashes the inputs again after the tests ran
#[test]
fn an_edit_during_the_tests_is_noticed() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: edits README.md\n");
    let document = repo.run_json(&[], 1);
    no_result_everywhere(&document);
}

// behavior: When an input changed during the launch, no result is written
#[test]
fn an_input_changed_during_the_launch_gives_no_results() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: edits desk/lending.txt\n");
    let document = repo.run_json(&[], 1);
    no_result_everywhere(&document);
    no_results(&repo);
}

// behavior: When the launch crashes, no result is written
#[test]
fn a_crashed_launch_gives_no_results() {
    for directive in ["adapter: crash", "adapter: garbage"] {
        let repo = Repo::sample();
        repo.append(LATE_FINE_TESTS, &format!("{directive}\n"));
        let document = repo.run_json(&[], 1);
        no_result_everywhere(&document);
        assert_eq!(document["suites"][0]["launch"], "crashed", "{directive}: {document:#}");
        no_results(&repo);
    }
}

// behavior: Each result stores the names of the tests it enrolled
#[test]
fn a_result_stores_its_test_names() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let record = repo.result_of(ON_TIME_RETURN);
    let names: BTreeSet<String> = record["body"]["tests"]
        .as_array()
        .unwrap_or_else(|| panic!("no tests: {record:#}"))
        .iter()
        .map(|test| test["name"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(names, set_of(&["free-return.returned-on-due-date", "free-return.returned-early"]), "{record:#}");
}

// behavior: Each result is written with its fingerprint and part digests
#[test]
fn a_result_is_written_with_its_digests() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        let body = repo.result_of(check)["body"].clone();
        assert!(body["fingerprint"].as_str().is_some_and(is_digest), "{check}: {body:#}");
        for part in PARTS {
            assert!(body["parts"][part].as_str().is_some_and(is_digest), "{check}: no digest of {part}: {body:#}");
        }
    }
}

// behavior: No command accepts a result from a file or a test report
#[test]
fn no_command_imports_a_result() {
    let repo = Repo::sample();
    repo.write("report.json", "{\"format\":\"adapter-report-1\",\"build\":\"ok\",\"tests\":[]}\n");
    for args in [
        &["run", "--report", "report.json"][..],
        &["run", "--from", "report.json"][..],
        &["check", "run", SIXTH_LOAN, "--report", "report.json"][..],
        &["check", "import", "report.json"][..],
        &["import", "report.json"][..],
        &["sync", "report.json"][..],
    ] {
        let output = repo.azimuth(args);
        assert_eq!(output.status, 2, "azimuth {args:?} accepted a report:\n{}", output.text());
    }
    no_results(&repo);
}

// behavior: A fail on one fingerprint leaves a pass on another fingerprint in place
#[test]
fn a_fail_keeps_the_pass_of_other_content() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let before: BTreeSet<_> = repo.store_files().into_iter().collect();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.run_json(&[], 1);
    let after: BTreeSet<_> = repo.store_files().into_iter().collect();
    assert!(before.is_subset(&after), "a record of the pass disappeared");
    repo.replace(LATE_FINE_TESTS, "outcome: failed", "outcome: passed");
    let status = repo.status_json();
    assert_eq!(status_checks(&status)[LATE_RETURN]["state"], "kept", "{status:#}");
}

// behavior: A later result on one fingerprint is added beside the earlier one
#[test]
fn a_second_result_on_one_fingerprint_is_added() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let first = repo.result_of(SIXTH_LOAN);
    repo.azimuth(&["check", "run", SIXTH_LOAN, "--json"]).expect_status(0);
    let results = repo.results_of(SIXTH_LOAN);
    assert_eq!(results.len(), 2, "the second result did not join the first: {results:#?}");
    assert!(results.iter().any(|(_, record)| record == &first), "the first result changed");
    assert_eq!(results[0].1["body"]["fingerprint"], results[1].1["body"]["fingerprint"]);
}

// behavior: The duration of each test is stored in its result
#[test]
fn durations_are_reported_and_stored() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: passed\nduration: 250");
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert!(checks_by_id(&document)[check]["duration_ms"].is_u64(), "{document:#}");
    }
    let record = repo.result_of(LATE_RETURN);
    assert_eq!(record["body"]["tests"][0]["duration_ms"], 250, "{record:#}");
}

// behavior: Each entry of `suites` in `run-1` holds `duration_ms`, the wall time of its launch
#[test]
fn each_launched_suite_holds_the_wall_time_of_its_launch() {
    // The adapter of the returns suite waits 1.5 seconds, and every test reports 1 ms or less, so
    // only the wall time of the launch can reach 1500.
    let repo = Repo::sample();
    repo.split_returns_suite("");
    repo.append("suites/returns-checks/late-fine.test", "adapter: sleeps 1500\n");
    let document = repo.run_json(&[], 0);
    let suites = document["suites"].as_array().unwrap_or_else(|| panic!("no `suites`: {document:#}"));
    assert_eq!(suites.len(), 2, "{document:#}");
    for suite in suites {
        let duration = suite["duration_ms"].as_u64().unwrap_or_else(|| panic!("no whole `duration_ms`: {suite:#}"));
        if suite["name"] == "returns-checks" {
            assert!(duration >= 1500, "the launch took at least 1500 ms: {suite:#}");
        }
    }

    // A launch is not stored: in a record only a test carries a duration.
    for (path, record) in repo.records() {
        let mut found = Vec::new();
        durations_in(&record, String::new(), &mut found);
        for at in found {
            assert!(at.starts_with("body.tests."), "{} holds a duration at {at}: {record:#}", path.display());
        }
    }

    let missing = Repo::sample();
    missing.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"desk-adapter-that-is-not-installed\"");
    let document = missing.run_json(&[], 1);
    assert_eq!(document["suites"][0]["launch"], "not started", "{document:#}");
    assert!(document["suites"][0].get("duration_ms").is_none(), "an adapter that did not start has no launch time: {document:#}");
}

fn durations_in(value: &serde_json::Value, at: String, found: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, inner) in map {
                let here = if at.is_empty() { key.clone() } else { format!("{at}.{key}") };
                if key == "duration_ms" {
                    found.push(here.clone());
                }
                durations_in(inner, here, found);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, inner) in items.iter().enumerate() {
                durations_in(inner, format!("{at}.{index}"), found);
            }
        }
        _ => {}
    }
}

// behavior: It writes results to the outbox of its own worktree and then runs the local step of sync
#[test]
fn results_pass_through_the_outbox_into_the_store() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    assert!(repo.outbox_files().is_empty(), "records stayed in the outbox: {:?}", repo.outbox_files());
    for check in ALL_CHECKS {
        let in_store = repo.results_of(check).iter().any(|(path, _)| path.starts_with(repo.store()));
        assert!(in_store, "no record of {check} in the store");
    }
}

// behavior: Where it may not write the store, the results stay in the outbox and it says how many records wait
#[test]
fn an_unwritable_store_leaves_records_in_the_outbox() {
    let repo = Repo::sample();
    let store = repo.store();
    fs::create_dir_all(&store).unwrap();
    make_unwritable(&store);
    let output = repo.azimuth(&["run"]);
    set_mode(&store, 0o755);
    output.expect_status(0);
    assert_eq!(repo.outbox_files().len(), 4, "the results are not in the outbox: {:?}", repo.outbox_files());
    assert!(files_under(&store).is_empty(), "the store was written");
    assert!(output.text().contains('4'), "the output does not say that 4 records wait:\n{}", output.text());
}

// behavior: It writes nothing into a tracked file
#[test]
fn run_writes_no_tracked_file() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    assert_eq!(repo.git(&["status", "--porcelain"]), "", "run changed the tree");
}

// behavior: The local lane runs no lint and reports no finding
#[test]
fn the_local_lane_does_not_lint() {
    let repo = Repo::sample();
    repo.replace(LOAN_LIMIT_TESTS, "# Check: returned-book-frees-a-slot\n", "# Check: returned-book-frees-a-slot\n# Check: renewal-counts-as-one-loan\n");
    repo.write("desk/notes.txt", "Verificaton: a near miss outside the model is nothing.\n");
    repo.replace(LENDING_SPEC, "A borrower holds at most five loans at a time.\n", "A borrower holds at most five loans at a time.\nVerificaton: reviewed\n");
    let document = repo.run_json(&[], 0);
    assert!(document.get("findings").is_none(), "{document:#}");
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed");
    }
}

// behavior: A lane that ends with failure ends with status 1, and one that ends with success with 0
#[test]
fn the_lane_outcome_decides_the_status() {
    let good = Repo::sample();
    let document = good.run_json(&[], 0);
    assert_eq!(document["outcome"], "success", "{document:#}");
    let bad = known_fail();
    let document = bad.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
}

// behavior: On a reading error, a wrong argument or a CLI version other than the pinned one it ends with status 2
#[test]
fn run_ends_with_two_when_it_cannot_do_its_job() {
    let broken = Repo::sample();
    broken.replace(LENDING_SPEC, "### Requirement: loan-limit-holds", "### Requirement:");
    broken.azimuth(&["run"]).expect_status(2);

    let repo = Repo::sample();
    repo.azimuth(&["run", "--lane", "weekend"]).expect_status(2);
    repo.azimuth(&["run", "--no-such-flag"]).expect_status(2);
    repo.azimuth(&["run", "sixth-loan-is-refused"]).expect_status(2);

    let other = Repo::sample();
    other.replace(PROJECT_FILE, &format!("cli = \"{}\"", cli_version()), &format!("cli = \"{OTHER_CLI}\""));
    other.azimuth(&["run"]).expect_status(2);
    for repo in [&broken, &repo, &other] {
        assert!(repo.adapter_starts().is_empty());
    }
}

// behavior: Its document is `run-1` with these fields
#[test]
fn run_document_has_its_fields() {
    let repo = known_fail();
    let output = repo.azimuth(&["run", "--json"]);
    let document = output.json();
    assert_eq!(document["format"], "run-1");
    assert_eq!(document["lane"], "local");
    assert!(document["outcome"].is_string());
    // Every Check is kept or a known fail here, so no adapter starts and no suite is listed.
    assert_eq!(document["suites"], json!([]), "{document:#}");
    let checks = checks_by_id(&document);
    assert_eq!(checks.len(), 4);
    for (_, check) in checks {
        for field in ["id", "state"] {
            assert!(has(&check, field), "{check:#}");
        }
    }
    let known_fail = &checks_by_id(&document)[LATE_RETURN];
    assert_eq!(known_fail["reason"], "failed on these inputs", "{document:#}");
    assert!(has(known_fail, "detail"), "no `detail`: {document:#}");

    let fresh = Repo::sample();
    fresh.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let document = fresh.run_json(&[], 1);
    let failed = &checks_by_id(&document)[LATE_RETURN];
    assert_eq!(strings_of(&failed["failing_tests"]), vec!["late-fine"]);
    assert!(failed["duration_ms"].is_u64(), "{failed:#}");
    let suites = document["suites"].as_array().expect("suites");
    assert_eq!(suites.len(), 1, "{document:#}");
    assert_eq!(suites[0]["name"], "desk-checks");
    assert_eq!(suites[0]["launch"], "ok");
    assert!(suites[0]["duration_ms"].is_u64(), "a suite has no whole `duration_ms`: {document:#}");
}

fn suite_names(document: &serde_json::Value) -> Vec<String> {
    let suites = document["suites"].as_array().unwrap_or_else(|| panic!("no `suites`: {document:#}"));
    let mut names: Vec<String> = suites.iter().map(|suite| suite["name"].as_str().unwrap_or_default().to_string()).collect();
    names.sort();
    names
}

// behavior: `suites` lists only the suites whose adapter the core started or tried to start
#[test]
fn suites_lists_the_launched_suites_only() {
    let repo = Repo::sample();
    repo.split_returns_suite("");
    let first = repo.run_json(&[], 0);
    assert_eq!(suite_names(&first), vec!["desk-checks", "returns-checks"]);

    // Any edit of the tree would make every Check owed, so one Check's results are dropped instead.
    repo.forget(LATE_RETURN);
    repo.clear_log();
    let second = repo.run_json(&[], 0);
    assert_eq!(suite_names(&second), vec!["returns-checks"], "{second:#}");
    assert_eq!(repo.adapter_runs().len(), 1, "{:#?}", repo.adapter_runs());

    // An adapter that cannot be started was still tried, so its suite is listed.
    let missing = Repo::sample();
    missing.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"desk-adapter-that-is-not-installed\"");
    let document = missing.run_json(&[], 1);
    assert_eq!(suite_names(&document), vec!["desk-checks"], "{document:#}");
    assert_eq!(document["suites"][0]["launch"], "not started", "{document:#}");
}

// behavior: A suite with a missing facility or with nothing to run is not in `suites`
#[test]
fn a_suite_without_its_facility_or_work_is_not_listed() {
    let repo = Repo::sample();
    repo.split_returns_suite("\"shelf-scanner\"");
    repo.append(PROJECT_FILE, "\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe absent\"\n");
    let document = repo.run_json(&[], 0);
    assert_eq!(suite_names(&document), vec!["desk-checks"], "{document:#}");
    assert_eq!(reason_of(&document, LATE_RETURN), "cannot run here", "its Checks say why: {document:#}");

    let idle = Repo::sample();
    idle.run_json(&[], 0);
    let document = idle.run_json(&[], 0);
    assert_eq!(document["suites"], json!([]), "nothing was owed: {document:#}");
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "kept", "{document:#}");
    }
}

// behavior: When an input changed during a launch, `launch` is `ok` and the Checks have the reason `no result`
#[test]
fn an_input_changed_during_the_launch_is_an_ok_launch_without_results() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: edits desk/lending.txt\n");
    let document = repo.run_json(&[], 1);
    assert_eq!(suite_names(&document), vec!["desk-checks"], "{document:#}");
    assert_eq!(document["suites"][0]["launch"], "ok", "{document:#}");
    no_result_everywhere(&document);
}

// behavior: A Check's `duration_ms` is the sum of the durations of its tests in this run, and is absent when the Check did not run
#[test]
fn a_checks_duration_is_the_sum_of_its_tests() {
    let repo = Repo::sample();
    repo.replace(FREE_RETURN_TESTS, "test: returned-on-due-date\noutcome: passed", "test: returned-on-due-date\noutcome: passed\nduration: 30");
    repo.replace(FREE_RETURN_TESTS, "test: returned-early\noutcome: passed", "test: returned-early\noutcome: passed\nduration: 40");
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: passed\nduration: 250");
    let document = repo.run_json(&[], 0);
    assert_eq!(checks_by_id(&document)[ON_TIME_RETURN]["duration_ms"], 70, "{document:#}");
    assert_eq!(checks_by_id(&document)[LATE_RETURN]["duration_ms"], 250, "{document:#}");

    repo.forget(LATE_RETURN);
    let second = repo.run_json(&[], 0);
    assert_eq!(checks_by_id(&second)[LATE_RETURN]["duration_ms"], 250, "only this run counts: {second:#}");
    for check in [ON_TIME_RETURN, SIXTH_LOAN, RETURNED_BOOK] {
        assert_eq!(state_of(&second, check), "kept", "{second:#}");
        assert!(checks_by_id(&second)[check].get("duration_ms").is_none(), "{check} did not run: {second:#}");
    }

    let unproven = Repo::sample();
    unproven.need_facility("absent");
    let document = unproven.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert!(checks_by_id(&document)[check].get("duration_ms").is_none(), "{check} did not run: {document:#}");
    }
}

// behavior: A Check that ran in this command is `passed` or `failed`, and one that did not is `kept` or `owed`
#[test]
fn only_a_check_that_ran_is_passed_or_failed() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let first = repo.run_json(&[], 1);
    assert_eq!(state_of(&first, LATE_RETURN), "failed");
    assert_eq!(state_of(&first, SIXTH_LOAN), "passed");
    let second = repo.run_json(&[], 1);
    assert_eq!(state_of(&second, LATE_RETURN), "owed");
    assert_eq!(state_of(&second, SIXTH_LOAN), "kept");
}

// behavior: On a reading error the document is `error-1` with these fields
#[test]
fn run_on_a_reading_error_prints_error_1() {
    let repo = Repo::sample();
    repo.replace(RETURNS_VERIFICATION, "- Check: late-return-is-fined", "- Check: late return is fined");
    let output = repo.azimuth(&["run", "--json"]);
    output.expect_status(2);
    let document = output.json();
    assert_eq!(document["format"], "error-1");
    let error = &document["errors"][0];
    assert!(error["kind"].is_string(), "{error:#}");
    assert_eq!(error["file"], RETURNS_VERIFICATION, "{error:#}");
    assert_eq!(error["line"].as_u64(), Some(repo.line_of(RETURNS_VERIFICATION, "late return is fined")));
    assert!(!error["message"].as_str().unwrap_or_default().is_empty());
}

// behavior: With no flag it runs the local lane
#[test]
fn the_default_lane_is_local() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let document = repo.run_json(&[], 0);
    assert_eq!(document["lane"], "local");
    assert!(repo.adapter_runs().len() == 1, "the default lane read no stored result");
}

// behavior: It reads no environment variable to choose a lane
#[test]
fn no_environment_variable_chooses_the_lane() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.clear_log();
    let output = repo.azimuth_with_env(
        &["run", "--json"],
        &[("CI", "true"), ("GITHUB_ACTIONS", "true"), ("GITLAB_CI", "true"), ("BUILDKITE", "true"), ("AZIMUTH_LANE", "commit")],
    );
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["lane"], "local");
    assert!(repo.adapter_runs().is_empty(), "the run behaved as the commit lane");
}

// behavior: It goes on after a failed test and reports every Check with its state
#[test]
fn a_failure_does_not_stop_the_run() {
    let repo = Repo::sample();
    repo.replace(LOAN_LIMIT_TESTS, "# Check: sixth-loan-is-refused\noutcome: passed", "# Check: sixth-loan-is-refused\noutcome: failed");
    let document = repo.run_json(&[], 1);
    assert_eq!(state_of(&document, SIXTH_LOAN), "failed");
    for check in [RETURNED_BOOK, LATE_RETURN, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, check), "passed");
    }
}

fn second_suite_stopped_by(prepare: impl FnOnce(&Repo), status: i32) {
    let repo = Repo::sample();
    prepare(&repo);
    let document = repo.run_json(&[], status);
    assert_eq!(state_of(&document, SIXTH_LOAN), "passed", "{document:#}");
    assert_eq!(state_of(&document, RETURNED_BOOK), "passed", "{document:#}");
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    assert_eq!(state_of(&document, ON_TIME_RETURN), "owed", "{document:#}");
}

// behavior: A suite that does not build stops only its own Checks
#[test]
fn a_suite_that_does_not_build_stops_only_itself() {
    second_suite_stopped_by(|repo| {
        repo.split_returns_suite("");
        repo.append("suites/returns-checks/late-fine.test", "adapter: build-fails\n");
    }, 1);
}

// behavior: A suite that does not build stops only its own Checks
#[test]
fn a_crashed_launch_stops_only_its_suite() {
    second_suite_stopped_by(|repo| {
        repo.split_returns_suite("");
        repo.append("suites/returns-checks/late-fine.test", "adapter: crash\n");
    }, 1);
}

// behavior: A suite that does not build stops only its own Checks
#[test]
fn a_missing_facility_stops_only_its_suite() {
    second_suite_stopped_by(|repo| {
        repo.split_returns_suite("\"shelf-scanner\"");
        repo.append(PROJECT_FILE, "\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe absent\"\n");
    }, 0);
}

// behavior: It ends with failure when any Check is failed on the current content
#[test]
fn a_failed_check_fails_the_local_lane() {
    let repo = known_fail();
    let document = repo.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure");
}

// behavior: A second run on unchanged content after a failure runs nothing and ends with failure again
#[test]
fn a_known_fail_fails_again_without_running() {
    let repo = known_fail();
    for _ in 0..2 {
        let document = repo.run_json(&[], 1);
        assert_eq!(state_of(&document, LATE_RETURN), "owed");
        assert_eq!(reason_of(&document, LATE_RETURN), "failed on these inputs");
    }
    assert!(repo.adapter_starts().is_empty());
}

// behavior: It ends with success when the only owed Checks cannot run here or run in CI
#[test]
fn owed_checks_this_machine_cannot_fix_do_not_fail_the_local_lane() {
    let repo = Repo::sample();
    repo.split_returns_suite("\"shelf-scanner\"");
    repo.append(PROJECT_FILE, "\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe absent\"\n");
    repo.replace(
        LENDING_VERIFICATION,
        "  After one return the desk lends again to a borrower who held five loans.\n",
        "  After one return the desk lends again to a borrower who held five loans.\n  Reuse: never, the desk reads today's date\n",
    );
    let document = repo.run_json(&[], 0);
    assert_eq!(document["outcome"], "success");
    assert_eq!(state_of(&document, SIXTH_LOAN), "passed");
    assert_eq!(reason_of(&document, RETURNED_BOOK), "runs in CI");
    assert_eq!(reason_of(&document, LATE_RETURN), "cannot run here");
    assert_eq!(reason_of(&document, ON_TIME_RETURN), "cannot run here");
}

// behavior: A Check whose launch gave no result is `owed` with the reason `no result`, and `detail` names the cause
#[test]
fn a_launch_without_results_names_its_cause_in_the_detail() {
    let unbuilt = Repo::sample();
    unbuilt.append(LATE_FINE_TESTS, "adapter: build-fails\n");
    let build_details = no_result_everywhere(&unbuilt.run_json(&[], 1));

    let crashed = Repo::sample();
    crashed.append(LATE_FINE_TESTS, "adapter: crash\n");
    let crash_details = no_result_everywhere(&crashed.run_json(&[], 1));

    assert!(build_details.is_disjoint(&crash_details), "two causes have one detail: {build_details:?} {crash_details:?}");
}

// behavior: After a launch with no result, `azimuth status` shows the Check with the reason it had before the run
#[test]
fn a_launch_without_results_leaves_the_earlier_reason() {
    let fresh = Repo::sample();
    fresh.append(LATE_FINE_TESTS, "adapter: crash\n");
    fresh.run_json(&[], 1);
    let status = fresh.status_json();
    for check in ALL_CHECKS {
        assert_eq!(status_checks(&status)[check]["reason"], "never run", "{status:#}");
    }

    let changed = Repo::sample();
    changed.run_json(&[], 0);
    changed.append(LATE_FINE_TESTS, "adapter: build-fails\n");
    changed.run_json(&[], 1);
    let status = changed.status_json();
    for check in ALL_CHECKS {
        assert_eq!(status_checks(&status)[check]["state"], "owed", "{status:#}");
        assert_eq!(status_checks(&status)[check]["reason"], "an input changed", "{status:#}");
    }
}

// behavior: A Check with `Reuse: never` is `owed` with the reason `runs in CI` on a developer's machine
#[test]
fn a_never_reused_check_is_owed_with_runs_in_ci() {
    let repo = Repo::sample();
    reuse_never(&repo);
    let document = repo.run_json(&[], 0);
    let entry = &checks_by_id(&document)[ON_TIME_RETURN];
    assert_eq!((entry["state"].as_str(), entry["reason"].as_str()), (Some("owed"), Some("runs in CI")), "{document:#}");
    let status = repo.status_json();
    assert_eq!(status_checks(&status)[ON_TIME_RETURN]["reason"], "runs in CI", "{status:#}");
}

// behavior: `outcome` is `success` or `failure`, and `launch` is `ok`, `build failed`, `crashed` or `not started`
#[test]
fn outcome_and_launch_are_fixed_words() {
    let good = Repo::sample();
    let document = good.run_json(&[], 0);
    assert_eq!((document["outcome"].as_str(), document["suites"][0]["launch"].as_str()), (Some("success"), Some("ok")));

    let failing = Repo::sample();
    failing.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let document = failing.run_json(&[], 1);
    assert_eq!((document["outcome"].as_str(), document["suites"][0]["launch"].as_str()), (Some("failure"), Some("ok")));

    let unbuilt = Repo::sample();
    unbuilt.append(LATE_FINE_TESTS, "adapter: build-fails\n");
    assert_eq!(unbuilt.run_json(&[], 1)["suites"][0]["launch"], "build failed");

    let crashed = Repo::sample();
    crashed.append(LATE_FINE_TESTS, "adapter: two-reports\n");
    assert_eq!(crashed.run_json(&[], 1)["suites"][0]["launch"], "crashed");

    let missing = Repo::sample();
    missing.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"desk-adapter-that-is-not-installed\"");
    let document = missing.run_json(&[], 1);
    assert_eq!((document["outcome"].as_str(), document["suites"][0]["launch"].as_str()), (Some("failure"), Some("not started")));
}

// behavior: `findings` is absent in the local lane
#[test]
fn the_local_lane_document_has_no_findings() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "# Check: late-return-is-fined\n", "# Check: late-return-is-fined\n# Check: renewal-counts-as-one-loan\n");
    let document = repo.run_json(&[], 0);
    assert!(document.get("findings").is_none(), "{document:#}");
    let commit = repo.run_json(&["--lane", "commit"], 1);
    assert!(has(&commit, "findings"), "{commit:#}");
}

// behavior: A proof and an adapter are started with no shell, from a command line split on spaces
#[test]
fn a_proof_is_split_on_spaces_and_given_to_no_shell() {
    // A shell would drop the quotes, expand the variable and run the second command after `||`.
    let repo = Repo::sample();
    repo.need_facility("absent 'two words' $HOME || facility-probe present");
    let document = repo.run_json(&[], 0);
    assert_eq!(reason_of(&document, SIXTH_LOAN), "cannot run here", "{document:#}");
    let probes = repo.probe_starts();
    assert_eq!(probes.len(), 1, "{probes:#?}");
    assert_eq!(probes[0]["args"], json!(["absent", "'two", "words'", "$HOME", "||", "facility-probe", "present"]));
}

// behavior: A proof and an adapter are started with no shell, from a command line split on spaces
#[test]
fn an_adapter_command_is_split_on_spaces_and_given_to_no_shell() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"text-suite-adapter --suite-of 'the desk' $HOME\"");
    repo.run_json(&[], 0);
    let starts = repo.adapter_starts();
    assert_eq!(starts.len(), 1, "{starts:#?}");
    assert_eq!(starts[0]["args"], json!(["--suite-of", "'the", "desk'", "$HOME"]));
}

fn canonical_cwd(start: &serde_json::Value) -> std::path::PathBuf {
    std::fs::canonicalize(start["cwd"].as_str().unwrap_or_else(|| panic!("no cwd: {start:#}"))).unwrap()
}

// behavior: An adapter starts in its suite's folder
#[test]
fn each_adapter_starts_in_its_suites_folder() {
    let repo = Repo::sample();
    repo.split_returns_suite("");
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
    let mut folders: Vec<(String, std::path::PathBuf)> = repo
        .adapter_runs()
        .iter()
        .map(|start| (start["request"]["suite"].as_str().unwrap_or_default().to_string(), canonical_cwd(start)))
        .collect();
    folders.sort();
    let in_folder = |suite: &str| std::fs::canonicalize(repo.path(suite)).unwrap();
    assert_eq!(
        folders,
        vec![
            ("suites/desk-checks".to_string(), in_folder("suites/desk-checks")),
            ("suites/returns-checks".to_string(), in_folder("suites/returns-checks")),
        ],
        "the request names the suite from the root, and the adapter starts inside it"
    );
}

// behavior: A proof starts at the root of the repository
#[test]
fn a_proof_starts_at_the_root() {
    let repo = Repo::sample();
    repo.need_facility("present");
    repo.run_json(&[], 0);
    let probes = repo.probe_starts();
    assert_eq!(probes.len(), 1, "{probes:#?}");
    assert_eq!(canonical_cwd(&probes[0]), std::fs::canonicalize(repo.root()).unwrap(), "{:#}", probes[0]);
}

// behavior: A proof and an adapter get the environment of the CLI
#[test]
fn a_proof_and_an_adapter_get_the_environment() {
    let repo = Repo::sample();
    repo.need_facility("present");
    let output = repo.azimuth_with_env(&["run", "--json"], &[("APPARATUS_MARK", "passed on by the core")]);
    output.expect_status(0);
    let starts: Vec<serde_json::Value> = repo.probe_starts().into_iter().chain(repo.adapter_starts()).collect();
    assert_eq!(starts.len(), 2, "{starts:#?}");
    for start in starts {
        assert_eq!(start["mark"], "passed on by the core", "{start:#}");
    }
}

// behavior: The core adds no argument to an adapter's command line and passes on the arguments written in the project file
// behavior-2: Arguments on the adapter's command line in the project file reach the adapter as written
#[test]
fn the_adapter_gets_exactly_the_written_arguments() {
    let plain = Repo::sample();
    plain.run_json(&[], 0);
    let starts = plain.adapter_starts();
    assert_eq!(starts.len(), 1, "{starts:#?}");
    assert_eq!(starts[0]["args"], json!([]), "the core added arguments: {:#}", starts[0]);

    let written = Repo::sample();
    written.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"text-suite-adapter --config ../desk.config --verbose\"");
    written.run_json(&[], 0);
    let starts = written.adapter_starts();
    assert_eq!(starts.len(), 1, "{starts:#?}");
    assert_eq!(starts[0]["args"], json!(["--config", "../desk.config", "--verbose"]), "{:#}", starts[0]);
}

// behavior: A proof that runs longer than 10 seconds counts as hung, and its suite cannot run here
#[test]
fn a_proof_slower_than_ten_seconds_is_hung() {
    let slow = Repo::sample();
    slow.need_facility("sleep 12");
    let started = Instant::now();
    cannot_run_here(&slow);
    assert!(started.elapsed() >= Duration::from_secs(10), "the proof was given only {:?}", started.elapsed());

    let patient = Repo::sample();
    patient.need_facility("sleep 3");
    let document = patient.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "a proof of 3 seconds was cut short: {document:#}");
    }
}

// behavior: An unknown command, flag or lane, and an unknown id of a Check or a Requirement, end with status 2
#[test]
fn whatever_the_cli_does_not_know_ends_with_two() {
    let repo = Repo::sample();
    for args in [
        &["shelve"][..],
        &["status", "--colour"][..],
        &["run", "--lane", "weekend"][..],
        &["check", "show", "no-such-check"][..],
        &["check", "run", "no-such-check"][..],
        &["requirement", "show", "no-such-requirement"][..],
        &["requirement", "show", SIXTH_LOAN][..],
        &["check", "show", LOAN_LIMIT][..],
    ] {
        let output = repo.azimuth(args);
        assert_eq!(output.status, 2, "azimuth {args:?}:\n{}", output.text());
    }
    assert!(repo.adapter_starts().is_empty());
}

// behavior: It ends with status 1 when a suite did not build
#[test]
fn a_suite_that_does_not_build_ends_with_one() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: build-fails\n");
    let document = repo.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
}

// behavior: It ends with status 1 when a launch crashed
#[test]
fn a_crashed_launch_ends_with_one() {
    for directive in ["adapter: crash", "adapter: garbage", "adapter: two-reports"] {
        let repo = Repo::sample();
        repo.append(LATE_FINE_TESTS, &format!("{directive}\n"));
        let document = repo.run_json(&[], 1);
        assert_eq!(document["outcome"], "failure", "{directive}: {document:#}");
    }
}

// behavior: It ends with status 1 when the adapter did not start, and the Checks of that suite are owed
#[test]
fn an_adapter_that_does_not_start_ends_with_one() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"desk-adapter-that-is-not-installed\"");
    let document = repo.run_json(&[], 1);
    no_result_everywhere(&document);
    assert_eq!(document["outcome"], "failure", "{document:#}");
}

// behavior: It ends with status 1 when an input changed during the launch
#[test]
fn an_input_changed_during_the_launch_ends_with_one() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: edits desk/returns.txt\n");
    let document = repo.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
}

// behavior: It ends with status 1 when a marker in source matched no listed test
#[test]
fn a_marker_that_matches_no_listed_test_ends_with_one() {
    let repo = Repo::sample();
    repo.write("suites/desk-checks/fine-calendar.test", "kind: helper\n# Check: late-return-is-fined\n");
    let document = repo.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
    assert_eq!(reason_of(&document, LATE_RETURN), "no result", "{document:#}");
}

// behavior: It ends with status 1 when a marked test was skipped or not run
#[test]
fn a_skipped_marked_test_ends_with_one() {
    for outcome in ["skipped", "not_run"] {
        let repo = Repo::sample();
        repo.replace(LATE_FINE_TESTS, "outcome: passed", &format!("outcome: {outcome}"));
        let document = repo.run_json(&[], 1);
        assert_eq!(document["outcome"], "failure", "{outcome}: {document:#}");
        assert_eq!(reason_of(&document, LATE_RETURN), "no result", "{outcome}: {document:#}");
    }
}

// behavior: More marker lines for a Check id than listed tests that carry it leave the Check with no result
#[test]
fn more_marker_lines_than_listed_tests_give_no_result() {
    // The second marker for the late fine sits in a helper file, which the text adapter lists no
    // test from, as a runner lists no test for a marker on a helper method. So the id has two
    // marker lines in source and one listed test.
    let repo = Repo::sample();
    repo.write("suites/desk-checks/fine-calendar.test", "kind: helper\n# Check: late-return-is-fined\n");
    assert!(repo.lint_findings(0).is_empty(), "lint sees no fault in a marker on a helper");
    let document = repo.run_json(&[], 1);
    let late = &checks_by_id(&document)[LATE_RETURN];
    assert_eq!(late["state"], "owed", "{document:#}");
    assert_eq!(late["reason"], "no result", "{document:#}");
    assert!(repo.results_of(LATE_RETURN).is_empty(), "a result was written for the Check with a stray marker");

    // One marker line for each listed test, and one marker line that enrolls two tests: neither
    // has more marker lines than tests, so both get their results.
    for check in [SIXTH_LOAN, RETURNED_BOOK, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
        assert_eq!(repo.results_of(check).len(), 1, "{check}");
    }
}

// Conditions in the model: `docs/increments/contexts-conditions.md`. No component is started in
// the payouts product, so a condition can never be applied.

fn every_check_is_owed_as_not_applied(repo: &Repo, document: &serde_json::Value) {
    for check in all_checks() {
        owed_not_applied(repo, document, check);
    }
}

// behavior-contexts-conditions: `azimuth run` leaves a Check with a condition owed with "context not applied" when no started component can apply it
#[test]
fn run_leaves_a_conditioned_check_owed_as_not_applied() {
    let repo = every_check_under_a_condition();
    let document = repo.run_json(&[], 1);
    every_check_is_owed_as_not_applied(&repo, &document);
    let scenario = payouts();
    on_reaches(&scenario, &["Context: payouts.instant=true", "Context: payouts.region=\"se\""]);
    let document = scenario.run_json(&[], 1);
    owed_not_applied(&scenario, &document, REACHES_THE_CARD);
}

// behavior-contexts-conditions: `azimuth run` leaves such a Check owed also in a suite with Checks that have no condition
#[test]
fn run_leaves_a_conditioned_check_owed_beside_checks_with_no_condition() {
    let repo = payouts();
    on_reaches(&repo, &["Context: payouts.instant=true"]);
    let document = repo.run_json(&[], 1);
    owed_not_applied(&repo, &document, REACHES_THE_CARD);
    for check in ALL_CHECKS.iter().copied().chain([WAITS_FOR_REVIEW]) {
        assert_eq!(state_of(&document, check), "passed", "{check}: {document:#}");
        assert_eq!(repo.results_of(check).len(), 1, "{check} has no result");
    }
}

// behavior-contexts-conditions: `azimuth run` leaves Checks with a faulty readable condition owed with "context not applied"
#[test]
fn run_leaves_checks_with_a_faulty_condition_owed_as_not_applied() {
    for (requirement, scenario) in FAULTY_CONDITIONS {
        let repo = faulty_payouts(requirement, scenario);
        let document = repo.run_json(&[], 1);
        owed_not_applied(&repo, &document, REACHES_THE_CARD);
        if scenario.is_empty() {
            owed_not_applied(&repo, &document, WAITS_FOR_REVIEW);
        }
    }
}

// behavior-contexts-conditions: A Check whose condition was not applied has no result, whatever its tests did
#[test]
fn a_check_whose_condition_was_not_applied_has_no_result_whatever_its_tests_did() {
    for outcome in ["passed", "failed"] {
        let repo = every_check_under_a_condition();
        repo.write(PAYOUTS_TESTS, &format!("test: sends-at-once\n# Check: payout-reaches-the-card\noutcome: {outcome}\n\ntest: holds-for-review\n# Check: payout-waits-for-review\noutcome: {outcome}\n"));
        let document = repo.run_json(&[], 1);
        every_check_is_owed_as_not_applied(&repo, &document);
        no_results(&repo);
        let check_run = repo.azimuth(&["check", "run", REACHES_THE_CARD, "--json"]);
        check_run.expect_status(1);
        owed_not_applied(&repo, &check_run.json(), REACHES_THE_CARD);
        no_results(&repo);
        let status = repo.status_json();
        for check in all_checks() {
            assert_eq!(status_checks(&status)[check]["state"], "owed", "{status:#}");
        }
    }
}

// behavior-contexts-conditions: `azimuth run` can still run Checks without the faulty condition in the same suite
#[test]
fn run_still_runs_the_checks_without_the_faulty_condition() {
    for (requirement, scenario) in FAULTY_CONDITIONS {
        let repo = faulty_payouts(requirement, scenario);
        let document = repo.run_json(&[], 1);
        owed_not_applied(&repo, &document, REACHES_THE_CARD);
        let free: Vec<&str> = ALL_CHECKS.iter().copied().chain(if requirement.is_empty() { Some(WAITS_FOR_REVIEW) } else { None }).collect();
        if !requirement.is_empty() {
            owed_not_applied(&repo, &document, WAITS_FOR_REVIEW);
        }
        let requested = repo.requested_checks();
        let ran = repo.tests_ran();
        for check in free {
            assert_eq!(state_of(&document, check), "passed", "{check}: {document:#}");
            assert!(requested.contains(check), "{check} was not requested: {requested:?}");
            assert_eq!(repo.results_of(check).len(), 1, "{check} has no result");
        }
        assert!(ran.iter().any(|name| name.contains("loan")), "the tests of the free Checks did not run: {ran:?}");
    }
}

// behavior-contexts-conditions: The local lane ends with status 1 for "context not applied"
#[test]
fn the_local_lane_ends_with_status_one_for_a_condition_not_applied() {
    let repo = payouts();
    on_reaches(&repo, &["Context: payouts.instant=true"]);
    let document = repo.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
    let output = repo.azimuth(&["run"]);
    output.expect_status(1);
    // Nothing else is wrong: the same model without the condition ends with status 0.
    let control = payouts();
    let document = control.run_json(&[], 0);
    assert_eq!(document["outcome"], "success", "{document:#}");
    // `check run` of the Check ends the same way.
    repo.azimuth(&["check", "run", REACHES_THE_CARD]).expect_status(1);
}

// Launches on the sample: `docs/increments/contexts-launches.md`.

// behavior-contexts-launches: `azimuth run` makes one launch per group
#[test]
fn run_makes_one_launch_per_group() {
    let repo = four_launches();
    applied(&repo);
    let document = repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(all.len(), 4, "{all:#?}");
    let conditions = conditions_of(&all);
    for expected in [json!({}), json!({"payouts.instant": "true"}), json!({"payouts.instant": "false"}), json!({"payouts.instant": "true", "payouts.region": "se"})] {
        assert_eq!(conditions.iter().filter(|condition| **condition == expected).count(), 1, "{expected}: {all:#?}");
    }
    let requested: Vec<String> = all.iter().flat_map(|launch| launch.checks()).collect();
    assert_eq!(requested.len(), 7, "a Check was requested in two launches or none: {requested:?}");
    for check in all_launch_checks() {
        proven(&repo, &document, check);
    }
}

// behavior-contexts-launches: The local lane ends with status 1
#[test]
fn the_local_lane_ends_with_status_one_when_a_context_was_not_applied() {
    let repo = instant_true_with(&["component api both source payouts.instant before-azimuth configuration"]);
    repo.azimuth(&["run"]).expect_status(1);
    repo.azimuth(&["check", "run", INSTANT_CHECK]).expect_status(1);
    // Nothing else is wrong: the reports of the same launch, applied, end with status 0.
    instant_true_with(&[]).azimuth(&["run"]).expect_status(0);
}

// behavior-contexts-launches: The run ends with failure while a Check is owed because its launch gave no result
#[test]
fn the_run_ends_with_failure_while_a_check_is_owed_because_its_launch_gave_no_result() {
    let repo = instant_true_with(&["component api both source payouts.instant before-azimuth configuration"]);
    let document = repo.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
    owed_not_applied(&repo, &document, INSTANT_CHECK);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{check}: {document:#}");
    }
    let crashed = instant_true_with(&["when payouts.instant=true crash"]);
    let document = crashed.run_json(&[], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
    let control = instant_true_with(&[]);
    assert_eq!(control.run_json(&[], 0)["outcome"], "success");
}
