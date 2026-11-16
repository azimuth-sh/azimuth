//! Group "`azimuth run --lane commit`" of docs/increments/1.md.

use azimuth_tests::*;

fn commit_lane(repo: &Repo, status: i32) -> serde_json::Value {
    let document = repo.run_json(&["--lane", "commit"], status);
    assert_eq!(document["lane"], "commit", "{document:#}");
    document
}

fn all_requested(repo: &Repo) {
    let requested = repo.requested_checks();
    for check in ALL_CHECKS {
        assert!(requested.contains(check), "{check} was not run: {requested:?}");
    }
}

// behavior: With no server it reads no stored result and runs every Check
#[test]
fn the_commit_lane_runs_every_check() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.clear_log();
    let document = commit_lane(&repo, 0);
    all_requested(&repo);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
}

// behavior: A result file placed in the store beforehand has no effect
#[test]
fn a_planted_pass_has_no_effect() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.run_json(&[], 1);
    // Forge the stored fail into a pass, as code under test could in a CI job. The forged record
    // is well formed, which the local lane shows by keeping it.
    repo.rewrite_records(|record| {
        if record["body"]["outcome"] == "failed" {
            record["body"]["outcome"] = json!("passed");
            record["body"]["tests"][0]["outcome"] = json!("passed");
        }
    });
    let status = repo.status_json();
    assert_eq!(status_checks(&status)[LATE_RETURN]["state"], "kept", "the forged pass is not a readable record: {status:#}");
    repo.clear_log();
    let document = commit_lane(&repo, 1);
    all_requested(&repo);
    assert_eq!(state_of(&document, LATE_RETURN), "failed", "{document:#}");
}

// behavior: It runs a Check with `Reuse: never`
#[test]
fn the_commit_lane_runs_a_never_reused_check() {
    let repo = Repo::sample();
    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never, the desk reads today's date\n",
    );
    let document = commit_lane(&repo, 0);
    assert_eq!(state_of(&document, ON_TIME_RETURN), "passed", "{document:#}");
    assert!(repo.requested_checks().contains(ON_TIME_RETURN));
}

// behavior: It runs lint first, and its output names lint as its first step
#[test]
fn the_commit_lane_names_lint_first() {
    let repo = Repo::sample();
    let output = repo.azimuth(&["run", "--lane", "commit"]);
    output.expect_status(0);
    // Progress may go to either stream, so the first line of each is looked at.
    let first_line = |text: &str| text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or_default().to_lowercase();
    assert!(
        first_line(&output.stdout).contains("lint") || first_line(&output.stderr).contains("lint"),
        "the first line of the lane is not about lint:\n{}",
        output.text()
    );
    let document = commit_lane(&repo, 0);
    assert!(has(&document, "findings"), "the commit lane document has no findings: {document:#}");
}

fn with_a_finding() -> Repo {
    let repo = Repo::sample();
    repo.replace(
        LOAN_LIMIT_TESTS,
        "# Check: returned-book-frees-a-slot\n",
        "# Check: returned-book-frees-a-slot\n# Check: renewal-counts-as-one-loan\n",
    );
    repo
}

// behavior: It ends with failure on a finding
#[test]
fn the_commit_lane_fails_on_a_finding() {
    let repo = with_a_finding();
    let document = commit_lane(&repo, 1);
    assert_eq!(document["outcome"], "failure");
    assert_eq!(count(&document["findings"]), 1, "{document:#}");
    for finding in document["findings"].as_array().unwrap() {
        entry_has_kind_file_line_message(finding);
    }
}

// behavior: It ends with failure when any Check is failed or owed
#[test]
fn the_commit_lane_fails_on_a_failed_check() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let document = commit_lane(&repo, 1);
    assert_eq!(state_of(&document, LATE_RETURN), "failed");
}

// behavior: It ends with failure when any Check is failed or owed
#[test]
fn the_commit_lane_fails_on_an_owed_check() {
    let repo = Repo::sample();
    repo.need_facility("absent");
    let document = commit_lane(&repo, 1);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "owed", "{document:#}");
    }
    let local = repo.run_json(&[], 0);
    assert_eq!(local["outcome"], "success", "the same state passes the local lane");
}

// behavior: With a finding it still runs every Check, and ends with failure
#[test]
fn a_finding_does_not_stop_the_checks_of_the_commit_lane() {
    let repo = with_a_finding();
    let document = commit_lane(&repo, 1);
    all_requested(&repo);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
}

// behavior: On a reading error it stops before any Check runs
#[test]
fn the_commit_lane_stops_on_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "#### Scenario: sixth-loan-is-refused", "#### Scenario:");
    let output = repo.azimuth(&["run", "--lane", "commit", "--json"]);
    output.expect_status(2);
    assert_eq!(output.json()["format"], "error-1");
    assert!(repo.adapter_starts().is_empty());
    assert!(repo.probe_starts().is_empty());
}
