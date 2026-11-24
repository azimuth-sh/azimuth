//! Group "`azimuth check run`" of docs/increments/1.md.

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
use contexts_conditions::*;

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

// Conditions in the model: `docs/increments/contexts-conditions.md`. No component is started in
// the payouts product, so a condition can never be applied.

// behavior-contexts-conditions: `azimuth check run` leaves a Check with a condition owed with "context not applied" when no started component can apply it
#[test]
fn check_run_leaves_a_conditioned_check_owed_as_not_applied() {
    let repo = every_check_under_a_condition();
    for check in all_checks() {
        let output = repo.azimuth(&["check", "run", check, "--json"]);
        output.expect_status(1);
        let document = output.json();
        assert_eq!(document["format"], "run-1", "{document:#}");
        owed_not_applied(&repo, &document, check);
    }
    let scenario = payouts();
    on_reaches(&scenario, &["Context: payouts.instant=true", "Context: payouts.region=\"se\""]);
    let output = scenario.azimuth(&["check", "run", REACHES_THE_CARD, "--json"]);
    output.expect_status(1);
    owed_not_applied(&scenario, &output.json(), REACHES_THE_CARD);
}

// behavior-contexts-conditions: `azimuth check run` leaves such a Check owed also in a suite with Checks that have no condition
#[test]
fn check_run_leaves_a_conditioned_check_owed_beside_checks_with_no_condition() {
    let repo = payouts();
    on_reaches(&repo, &["Context: payouts.instant=true"]);
    let output = repo.azimuth(&["check", "run", REACHES_THE_CARD, "--json"]);
    output.expect_status(1);
    owed_not_applied(&repo, &output.json(), REACHES_THE_CARD);
    let requested = repo.requested_checks();
    assert!(requested.iter().all(|check| check == REACHES_THE_CARD), "another Check was run: {requested:?}");
    let free = repo.azimuth(&["check", "run", WAITS_FOR_REVIEW, "--json"]);
    free.expect_status(0);
    assert_eq!(state_of(&free.json(), WAITS_FOR_REVIEW), "passed");
}

// behavior-contexts-conditions: `azimuth check run` leaves Checks with a faulty readable condition owed with "context not applied"
#[test]
fn check_run_leaves_checks_with_a_faulty_condition_owed_as_not_applied() {
    for (requirement, scenario) in FAULTY_CONDITIONS {
        let repo = faulty_payouts(requirement, scenario);
        let output = repo.azimuth(&["check", "run", REACHES_THE_CARD, "--json"]);
        output.expect_status(1);
        owed_not_applied(&repo, &output.json(), REACHES_THE_CARD);
        if scenario.is_empty() {
            let output = repo.azimuth(&["check", "run", WAITS_FOR_REVIEW, "--json"]);
            output.expect_status(1);
            owed_not_applied(&repo, &output.json(), WAITS_FOR_REVIEW);
        }
    }
}

// behavior-contexts-conditions: `azimuth check run` can still run a named Check without the faulty condition in the same suite
#[test]
fn check_run_still_runs_a_named_check_without_the_faulty_condition() {
    for (requirement, scenario) in FAULTY_CONDITIONS {
        let repo = faulty_payouts(requirement, scenario);
        for check in ALL_CHECKS {
            repo.clear_log();
            let output = repo.azimuth(&["check", "run", check, "--json"]);
            output.expect_status(0);
            let document = output.json();
            assert_eq!(state_of(&document, check), "passed", "{check}: {document:#}");
            assert_eq!(repo.requested_checks().into_iter().collect::<Vec<_>>(), vec![check.to_string()]);
            assert_eq!(repo.results_of(check).len(), 1, "{check} has no result");
        }
    }
}
