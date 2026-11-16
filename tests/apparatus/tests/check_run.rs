//! Group "`azimuth check run`" of docs/increments/1.md.

use azimuth_tests::*;

// behavior: `azimuth check run <id>` runs one named Check, also when its result is kept
#[test]
fn check_run_runs_a_kept_check() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.clear_log();
    let output = repo.azimuth(&["check", "run", SIXTH_LOAN, "--json"]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "run-1");
    assert_eq!(state_of(&document, SIXTH_LOAN), "passed", "{document:#}");
    let requested = repo.requested_checks();
    assert_eq!(requested.into_iter().collect::<Vec<_>>(), vec![SIXTH_LOAN.to_string()]);
}

// behavior: It runs a Check that failed on the current fingerprint again on a developer's machine
#[test]
fn check_run_runs_a_known_fail_again() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.run_json(&[], 1);
    repo.clear_log();
    let output = repo.azimuth(&["check", "run", LATE_RETURN, "--json"]);
    output.expect_status(1);
    assert_eq!(state_of(&output.json(), LATE_RETURN), "failed");
    assert_eq!(repo.adapter_runs().len(), 1, "the known fail was not run again");
}

// behavior: With no id it runs nothing and reports an error
#[test]
fn check_run_without_an_id_is_an_error() {
    let repo = Repo::sample();
    let output = repo.azimuth(&["check", "run"]);
    output.expect_status(2);
    assert!(!output.stderr.trim().is_empty(), "no error was reported");
    assert!(repo.adapter_starts().is_empty());
    let unknown = repo.azimuth(&["check", "run", "no-such-check"]);
    unknown.expect_status(2);
    assert!(repo.adapter_starts().is_empty());
}

// behavior: It follows the same launch as `azimuth run`, so its result is verified
#[test]
fn a_result_of_check_run_counts_like_any_other() {
    let repo = Repo::sample();
    repo.azimuth(&["check", "run", ON_TIME_RETURN, "--json"]).expect_status(0);
    let status = repo.status_json();
    let checks = status_checks(&status);
    assert_eq!(checks[ON_TIME_RETURN]["state"], "kept", "{status:#}");
    assert_eq!(checks[SIXTH_LOAN]["state"], "owed", "{status:#}");
    repo.clear_log();
    let document = repo.run_json(&[], 0);
    assert_eq!(state_of(&document, ON_TIME_RETURN), "kept");
    assert!(!repo.requested_checks().contains(ON_TIME_RETURN));

    let crashed = Repo::sample();
    crashed.append(FREE_RETURN_TESTS, "adapter: crash\n");
    crashed.azimuth(&["check", "run", ON_TIME_RETURN, "--json"]).expect_status(1);
    assert!(crashed.results_of(ON_TIME_RETURN).is_empty(), "a crashed launch of check run wrote a result");
}
