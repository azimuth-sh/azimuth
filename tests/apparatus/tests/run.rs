//! Group "`azimuth run`" of docs/increments/1.md.

use std::collections::BTreeSet;
use std::fs;

use azimuth_tests::*;

fn ids(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|id| id.to_string()).collect()
}

fn owed_everywhere(document: &serde_json::Value) {
    for check in ALL_CHECKS {
        assert_eq!(state_of(document, check), "owed", "{document:#}");
    }
}

fn no_results(repo: &Repo) {
    for check in ALL_CHECKS {
        assert!(repo.records_naming(check).is_empty(), "a result was written for {check}");
    }
}

fn reuse_never(repo: &Repo) {
    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never, because the desk reads today's date\n",
    );
}

// behavior: It starts only the adapter command named in the project file and downloads nothing
#[test]
fn run_starts_the_named_adapter_command() {
    let mut repo = Repo::sample();
    let bin = repo.beside("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::copy(binaries_dir().join("text-suite-adapter"), bin.join("desk-runner")).unwrap();
    let path = format!("{}:{}", bin.display(), path_without("dotnet"));
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
    assert!(output.text().contains("desk-adapter-that-is-not-installed"), "{}", output.text());
    assert!(repo.adapter_starts().is_empty());
    no_results(&repo);
    let document = repo.azimuth(&["run", "--json"]).json();
    owed_everywhere(&document);
}

// behavior: It computes each Check's fingerprint before it starts anything
#[test]
fn an_input_changed_by_the_proof_gives_no_results() {
    let repo = Repo::sample();
    let target = repo.path("desk/returns.txt").display().to_string();
    repo.need_facility(&format!("edit {target}"));
    let document = repo.azimuth(&["run", "--json"]).json();
    assert_eq!(repo.probe_starts().len(), 1, "the proof ran once");
    for check in ALL_CHECKS {
        assert_ne!(state_of(&document, check), "passed", "{document:#}");
    }
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
        assert!(reason_of(&document, check).starts_with("cannot run here"), "{document:#}");
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
    // The design names a time limit and no value. The harness gives a command 120 seconds.
    let repo = Repo::sample();
    repo.need_facility("hang");
    cannot_run_here(&repo);
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
    let document = repo.azimuth(&["run", "--json"]).json();
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    assert!(repo.records_naming(LATE_RETURN).is_empty(), "a result was written");
}

// behavior: A marker in source that matches no listed test is a finding, and that Check gets no result
#[test]
fn a_marker_on_a_helper_gives_no_result() {
    let repo = Repo::sample();
    repo.write(
        "suites/desk-checks/fine-calendar.test",
        "kind: helper\n# A helper the fine tests read. It is not a test, and the marker on it is a mistake.\n# Check: late-return-is-fined\n",
    );
    let document = repo.azimuth(&["run", "--json"]).json();
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    for check in [SIXTH_LOAN, RETURNED_BOOK, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
    assert!(repo.records_naming(LATE_RETURN).is_empty(), "a result was written");
    let commit = repo.azimuth(&["run", "--lane", "commit", "--json"]);
    commit.expect_status(1);
    assert!(count(&commit.json()["findings"]) >= 1, "the commit lane shows no finding");
}

// behavior: A listed test whose Check id appears nowhere in source is a finding, and that Check gets no result
#[test]
fn a_listed_test_without_a_marker_in_source_gives_no_result() {
    let repo = Repo::sample();
    listed_but_not_in_source(&repo);
    let document = repo.azimuth(&["run", "--json"]).json();
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    assert!(repo.records_naming(LATE_RETURN).is_empty());
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
    let document = repo.azimuth(&["run", "--json"]).json();
    owed_everywhere(&document);
    assert_ne!(document["suites"][0]["launch"], "ok", "{document:#}");
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
        let document = repo.azimuth(&["run", "--json"]).json();
        assert_eq!(state_of(&document, ON_TIME_RETURN), "owed", "{outcome}: {document:#}");
        assert!(repo.records_naming(ON_TIME_RETURN).is_empty(), "{outcome}: a result was written");
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
    let document = mixed.azimuth(&["run", "--json"]).json();
    assert_ne!(state_of(&document, ON_TIME_RETURN), "passed", "one test passed and one was skipped: {document:#}");
}

// behavior: It hashes the inputs again after the tests ran
#[test]
fn an_edit_during_the_tests_is_noticed() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: edits README.md\n");
    let document = repo.azimuth(&["run", "--json"]).json();
    owed_everywhere(&document);
}

// behavior: When an input changed during the launch, no result is written
#[test]
fn an_input_changed_during_the_launch_gives_no_results() {
    let repo = Repo::sample();
    repo.append(LATE_FINE_TESTS, "adapter: edits desk/lending.txt\n");
    let document = repo.azimuth(&["run", "--json"]).json();
    owed_everywhere(&document);
    no_results(&repo);
}

// behavior: When the launch crashes, no result is written
#[test]
fn a_crashed_launch_gives_no_results() {
    for directive in ["adapter: crash", "adapter: garbage"] {
        let repo = Repo::sample();
        repo.append(LATE_FINE_TESTS, &format!("{directive}\n"));
        let document = repo.azimuth(&["run", "--json"]).json();
        owed_everywhere(&document);
        assert_ne!(document["suites"][0]["launch"], "ok", "{directive}: {document:#}");
        no_results(&repo);
    }
}

// behavior: Each result stores the names of the tests it enrolled
#[test]
fn a_result_stores_its_test_names() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let strings: BTreeSet<String> = repo.records_naming(ON_TIME_RETURN).iter().flat_map(|(_, value)| all_strings(value)).collect();
    assert!(strings.contains("free-return.returned-on-due-date"), "{strings:?}");
    assert!(strings.contains("free-return.returned-early"), "{strings:?}");
}

// behavior: Each result is written with its fingerprint and part digests
#[test]
fn a_result_is_written_with_its_digests() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        let digests: BTreeSet<String> = repo
            .records_naming(check)
            .iter()
            .flat_map(|(_, value)| all_strings(value))
            .filter(|text| is_digest(text))
            .collect();
        assert!(digests.iter().any(|digest| digest.starts_with("parts-1:")), "{check}: {digests:?}");
        assert!(digests.iter().any(|digest| digest.starts_with("text-1:")), "{check}: {digests:?}");
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
    let first = repo.records_naming(SIXTH_LOAN).len();
    repo.azimuth(&["check", "run", SIXTH_LOAN, "--json"]).expect_status(0);
    let second = repo.records_naming(SIXTH_LOAN).len();
    assert_eq!(second, first + 1, "the second result did not join the first");
}

// behavior: The duration of each launch is stored
#[test]
fn durations_are_reported_and_stored() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: passed\nduration: 250");
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert!(checks_by_id(&document)[check]["duration_ms"].is_u64(), "{document:#}");
    }
    let keys: Vec<String> = repo.records_naming(LATE_RETURN).iter().flat_map(|(_, value)| all_keys(value)).collect();
    assert!(keys.iter().any(|key| key.contains("duration")), "no stored duration: {keys:?}");
}

// behavior: It writes results to the outbox of its own worktree and then runs the local step of sync
#[test]
fn results_pass_through_the_outbox_into_the_store() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    assert!(repo.outbox_files().is_empty(), "records stayed in the outbox: {:?}", repo.outbox_files());
    for check in ALL_CHECKS {
        let in_store = repo.records_naming(check).iter().any(|(path, _)| path.starts_with(repo.store()));
        assert!(in_store, "no record of {check} in the store");
    }
}

// behavior: Where it may not write the store, the results stay in the outbox and it says how many records wait
#[test]
fn an_unwritable_store_leaves_records_in_the_outbox() {
    let repo = Repo::sample();
    let store = repo.store();
    fs::create_dir_all(&store).unwrap();
    set_mode(&store, 0o555);
    let output = repo.azimuth(&["run"]);
    set_mode(&store, 0o755);
    output.expect_status(0);
    let waiting = repo.outbox_files().len();
    assert!(waiting >= 4, "the results are not in the outbox: {:?}", repo.outbox_files());
    assert!(files_under(&store).is_empty(), "the store was written");
    assert!(output.text().contains(&waiting.to_string()), "the output does not say {waiting} records wait:\n{}", output.text());
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
    assert_eq!(count(&document["findings"]), 0, "{document:#}");
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
    other.replace(PROJECT_FILE, &format!("cli = \"{PINNED_CLI}\""), &format!("cli = \"{OTHER_CLI}\""));
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
    let suites = document["suites"].as_array().expect("suites");
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0]["name"], "desk-checks");
    assert!(suites[0].get("launch").is_some(), "{document:#}");
    let checks = checks_by_id(&document);
    assert_eq!(checks.len(), 4);
    for (_, check) in checks {
        for field in ["id", "state"] {
            assert!(check.get(field).is_some(), "{check:#}");
        }
    }
    assert!(checks_by_id(&document)[LATE_RETURN].get("reason").is_some());

    let fresh = Repo::sample();
    fresh.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let document = fresh.run_json(&[], 1);
    let failed = &checks_by_id(&document)[LATE_RETURN];
    assert_eq!(strings_of(&failed["failing_tests"]), vec!["late-fine"]);
    assert!(failed["duration_ms"].is_u64(), "{failed:#}");
    assert_eq!(document["suites"][0]["launch"], "ok");
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
    assert!(error["file"].as_str().unwrap_or_default().ends_with(RETURNS_VERIFICATION), "{error:#}");
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

fn second_suite_stopped_by(prepare: impl FnOnce(&Repo)) {
    let repo = Repo::sample();
    prepare(&repo);
    let document = repo.azimuth(&["run", "--json"]).json();
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
    });
}

// behavior: A suite that does not build stops only its own Checks
#[test]
fn a_crashed_launch_stops_only_its_suite() {
    second_suite_stopped_by(|repo| {
        repo.split_returns_suite("");
        repo.append("suites/returns-checks/late-fine.test", "adapter: crash\n");
    });
}

// behavior: A suite that does not build stops only its own Checks
#[test]
fn a_missing_facility_stops_only_its_suite() {
    second_suite_stopped_by(|repo| {
        repo.split_returns_suite("\"shelf-scanner\"");
        repo.append(PROJECT_FILE, "\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe absent\"\n");
    });
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
        "  After one return the desk lends again to a borrower who held five loans.\n  Reuse: never, because the desk reads today's date\n",
    );
    let document = repo.run_json(&[], 0);
    assert_eq!(document["outcome"], "success");
    assert_eq!(state_of(&document, SIXTH_LOAN), "passed");
    assert_eq!(reason_of(&document, RETURNED_BOOK), "runs in CI");
    assert!(reason_of(&document, LATE_RETURN).starts_with("cannot run here"));
    assert!(reason_of(&document, ON_TIME_RETURN).starts_with("cannot run here"));
}
