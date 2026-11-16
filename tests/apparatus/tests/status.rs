//! Group "`azimuth status`" of docs/increments/1.md.

use std::collections::BTreeSet;

use azimuth_tests::*;

const QUIET_SUBJECT: &str = "2 Requirements";

fn non_empty_lines(text: &str) -> Vec<String> {
    text.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

fn proven() -> Repo {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo
}

fn known_fail() -> Repo {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.run_json(&[], 1);
    repo
}

fn declare_reviews(repo: &Repo) {
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.\n",
        "A borrower holds at most five loans at a time.\n\nVerification: reviewed\nRealization: reviewed\n",
    );
}

// behavior: It runs nothing and needs no network
#[test]
fn status_runs_nothing() {
    let repo = Repo::sample();
    repo.need_facility("present");
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    let closed = "http://127.0.0.1:9";
    let output = repo.azimuth_with_env(&["status", "--json"], &[("https_proxy", closed), ("HTTPS_PROXY", closed), ("ALL_PROXY", closed)]);
    output.expect_status(0);
    assert!(repo.adapter_starts().is_empty(), "status started the adapter");
    assert!(repo.probe_starts().is_empty(), "status proved a facility");
    assert!(repo.store_files().is_empty() && repo.outbox_files().is_empty(), "status wrote records");
}

// behavior: It lists every Requirement that has less than full evidence, with the reason
#[test]
fn status_lists_incomplete_requirements_with_their_reason() {
    let repo = known_fail();
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(output.stdout.contains(LATE_FINE), "{}", output.stdout);
    assert!(!output.stdout.contains(LOAN_LIMIT), "a proven Requirement was listed:\n{}", output.stdout);
    assert!(output.stdout.contains("failed on these inputs"), "{}", output.stdout);
    let status = repo.status_json();
    assert_ne!(requirement_in(&status, LATE_FINE).unwrap()["state"], "proven");
    assert_eq!(requirement_in(&status, LOAN_LIMIT).unwrap()["state"], "proven");
}

// behavior: With everything proven it prints one line that names the number of Requirements
#[test]
fn a_proven_tree_prints_one_line() {
    let repo = proven();
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    let lines = non_empty_lines(&output.stdout);
    assert_eq!(lines.len(), 1, "{}", output.stdout);
    assert!(lines[0].contains(QUIET_SUBJECT), "{}", lines[0]);
}

// behavior: With no server named in the project file, the quiet line says "proven on this machine" and no pass is marked "local result only"
#[test]
fn without_a_server_the_quiet_line_says_this_machine() {
    let repo = proven();
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(output.stdout.contains("proven on this machine"), "{}", output.stdout);
    for args in [&["status"][..], &["status", "--json"][..], &["requirement", "show", LOAN_LIMIT][..], &["check", "show", SIXTH_LOAN][..]] {
        assert!(!repo.azimuth(args).text().contains("local result only"), "azimuth {args:?} marks a local pass");
    }
}

// behavior: With an empty model it prints one line
#[test]
fn an_empty_model_prints_one_line() {
    let repo = Repo::sample();
    repo.remove("azimuth/model");
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert_eq!(non_empty_lines(&output.stdout).len(), 1, "{}", output.stdout);
}

// behavior: It names a Scenario that has no Check
#[test]
fn status_names_a_scenario_without_a_check() {
    let repo = Repo::sample();
    repo.append(RETURNS_SPEC, "\n#### Scenario: lost-book-is-charged\nA book reported lost is charged at its price.\n");
    repo.run_json(&[], 0);
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(output.stdout.contains("lost-book-is-charged"), "{}", output.stdout);
    let status = repo.status_json();
    let requirement = requirement_in(&status, LATE_FINE).unwrap();
    assert_ne!(requirement["state"], "proven", "{requirement:#}");
    let scenario = requirement["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scenario| scenario["id"] == "lost-book-is-charged")
        .expect("the Scenario is listed");
    assert_eq!(count(&scenario["checks"]), 0);
}

fn with_an_unmarked_failing_test() -> Repo {
    let repo = Repo::sample();
    repo.write("suites/desk-checks/catalog.test", "test: lists-titles\noutcome: failed\n\ntest: lists-authors\noutcome: passed\n");
    repo.run_json(&[], 0);
    repo
}

// behavior: It prints no count of unmarked tests and no coverage figure
#[test]
fn status_counts_no_unmarked_tests() {
    let repo = with_an_unmarked_failing_test();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.run_json(&[], 1);
    let text = repo.azimuth(&["status"]).stdout.to_lowercase();
    for word in ["unmarked", "coverage", "%", "catalog"] {
        assert!(!text.contains(word), "status mentions {word:?}:\n{text}");
    }
    let keys = all_keys(&repo.status_json());
    assert!(!keys.iter().any(|key| key.contains("unmarked") || key.contains("coverage")), "{keys:?}");
}

// behavior: A failing test without a marker changes nothing in its output
#[test]
fn an_unmarked_failing_test_leaves_status_quiet() {
    let repo = with_an_unmarked_failing_test();
    let output = repo.azimuth(&["status"]);
    let lines = non_empty_lines(&output.stdout);
    assert_eq!(lines.len(), 1, "{}", output.stdout);
    assert!(lines[0].contains(QUIET_SUBJECT));
}

// behavior: For a Requirement that declares no Verification Review it says "verification: not reviewed"
#[test]
fn no_declared_verification_review_reads_not_reviewed() {
    let repo = Repo::sample();
    let status = repo.status_json();
    assert_eq!(requirement_in(&status, LOAN_LIMIT).unwrap()["verification"], "not reviewed");
    let text = repo.azimuth(&["requirement", "show", LOAN_LIMIT]).stdout;
    assert!(text.contains("verification: not reviewed"), "{text}");
}

// behavior: For a Requirement that declares no Realization Review it says "realization: not reviewed"
#[test]
fn no_declared_realization_review_reads_not_reviewed() {
    let repo = Repo::sample();
    let status = repo.status_json();
    assert_eq!(requirement_in(&status, LATE_FINE).unwrap()["realization"], "not reviewed");
    let text = repo.azimuth(&["requirement", "show", LATE_FINE]).stdout;
    assert!(text.contains("realization: not reviewed"), "{text}");
}

// behavior: For a Requirement that declares a review it shows that review as owed, with the reason "this version records no reviews"
#[test]
fn a_declared_review_is_owed() {
    let repo = Repo::sample();
    declare_reviews(&repo);
    let status = repo.status_json();
    let requirement = requirement_in(&status, LOAN_LIMIT).unwrap();
    for field in ["verification", "realization"] {
        let value = requirement[field].as_str().unwrap_or_default();
        assert!(value.contains("owed") && value.contains("this version records no reviews"), "{field}: {requirement:#}");
    }
    let text = repo.azimuth(&["status"]).stdout;
    assert!(text.contains("this version records no reviews"), "{text}");
}

// behavior: A Requirement with an owed review is not counted in the quiet line
#[test]
fn an_owed_review_keeps_status_from_being_quiet() {
    let repo = Repo::sample();
    declare_reviews(&repo);
    repo.run_json(&[], 0);
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(output.stdout.contains(LOAN_LIMIT), "{}", output.stdout);
    assert!(!output.stdout.contains(QUIET_SUBJECT), "{}", output.stdout);
    let status = repo.status_json();
    assert_ne!(requirement_in(&status, LOAN_LIMIT).unwrap()["state"], "proven");
    assert_eq!(requirement_in(&status, LATE_FINE).unwrap()["state"], "proven");
}

// behavior: It says in one line how many stored records it skipped as unreadable
#[test]
fn status_counts_skipped_records() {
    let repo = proven();
    let before = repo.status_json();
    assert_eq!(before["skipped_records"], 0, "{before:#}");
    repo.rewrite_records(bump_formats);
    let status = repo.status_json();
    let skipped = status["skipped_records"].as_u64().expect("a number of skipped records");
    assert!(skipped >= 4, "{status:#}");
    let text = repo.azimuth(&["status"]).stdout;
    let lines: Vec<&str> = text.lines().filter(|line| line.contains("skipped")).collect();
    assert_eq!(lines.len(), 1, "{text}");
    assert!(lines[0].contains(&skipped.to_string()), "{text}");
}

// behavior: It says "approval: not recorded"
#[test]
fn approval_is_not_recorded() {
    let repo = proven();
    let status = repo.status_json();
    for id in [LOAN_LIMIT, LATE_FINE] {
        assert_eq!(requirement_in(&status, id).unwrap()["approval"], "not recorded");
    }
    let text = repo.azimuth(&["requirement", "show", LOAN_LIMIT]).stdout;
    assert!(text.contains("approval: not recorded"), "{text}");
}

// behavior: It says "the process here differs from the default" when an installed skill file was edited
#[test]
fn an_edited_skill_shows_a_process_of_its_own() {
    let repo = Repo::sample();
    repo.azimuth(&["skills", "install", "claude"]).expect_status(0);
    let untouched = repo.status_json();
    assert!(!untouched["process"].to_string().contains("differs"), "{untouched:#}");
    let installed = repo.untracked();
    assert!(!installed.is_empty(), "skills install wrote nothing");
    repo.append(&installed[0], "\nStop after the build as well.\n");
    let status = repo.status_json();
    assert!(status["process"].to_string().contains("differs"), "{status:#}");
    let text = repo.azimuth(&["status"]).stdout;
    assert!(text.contains("the process here differs from the default"), "{text}");
}

// behavior: With `--json`, standard output holds exactly one JSON document and progress goes to standard error
#[test]
fn json_output_is_one_document() {
    let repo = Repo::sample();
    for args in [&["status", "--json"][..], &["run", "--json"][..], &["run", "--lane", "commit", "--json"][..], &["lint", "--json"][..]] {
        let output = repo.azimuth(args);
        assert!(output.status == 0, "azimuth {args:?}:\n{}", output.text());
        output.json();
    }
}

// behavior: Without `--json` the output is text, also when it is piped
#[test]
fn piped_output_without_the_flag_is_text() {
    let repo = known_fail();
    for args in [&["status"][..], &["run"][..], &["lint"][..], &["check", "show", LATE_RETURN][..], &["requirement", "show", LATE_FINE][..]] {
        let output = repo.azimuth(args);
        assert!(!output.stdout.trim().is_empty(), "azimuth {args:?} printed nothing");
        assert!(
            serde_json::from_str::<serde_json::Value>(&output.stdout).is_err(),
            "azimuth {args:?} printed JSON into a pipe:\n{}",
            output.stdout
        );
    }
}

// behavior: The document names its format with a word and a number
#[test]
fn every_document_names_its_format() {
    let repo = known_fail();
    for (args, format) in [
        (&["status", "--json"][..], "status-1"),
        (&["requirement", "show", LATE_FINE, "--json"][..], "requirement-1"),
        (&["check", "show", LATE_RETURN, "--json"][..], "check-1"),
        (&["change", "show", "--json"][..], "change-1"),
        (&["lint", "--json"][..], "lint-1"),
        (&["run", "--json"][..], "run-1"),
    ] {
        assert_eq!(repo.azimuth(args).json()["format"], format, "azimuth {args:?}");
    }
}

// behavior: The document uses the same words for states and reasons as the text output
#[test]
fn json_and_text_share_their_words() {
    let repo = known_fail();
    let status = repo.status_json();
    let check = &status_checks(&status)[LATE_RETURN];
    let state = check["state"].as_str().unwrap();
    let reason = check["reason"].as_str().unwrap();
    assert_eq!((state, reason), ("owed", "failed on these inputs"));
    for args in [&["status"][..], &["check", "show", LATE_RETURN][..], &["requirement", "show", LATE_FINE][..]] {
        let text = repo.azimuth(args).stdout;
        assert!(text.contains(state) && text.contains(reason), "azimuth {args:?}:\n{text}");
    }
}

// behavior: On a reading error with `--json`, the output is a document of the format `error-1` with the file, the line and the message
#[test]
fn a_reading_error_is_an_error_1_document() {
    let repo = Repo::sample();
    repo.replace(RETURNS_SPEC, "#### Scenario: late-return-is-fined", "#### Scenario: late-return-is-fined\n#### Scenario: late-return-is-fined");
    let document = repo.expect_reading_error(Some(RETURNS_SPEC), None);
    assert!(document["errors"][0]["line"].as_u64().is_some(), "{document:#}");
}

// behavior: `azimuth lint`, `azimuth run` and the other reading commands take `--json` under the same rules
#[test]
fn every_command_of_the_increment_takes_json() {
    let repo = known_fail();
    for args in [
        &["lint", "--json"][..],
        &["run", "--json"][..],
        &["check", "run", SIXTH_LOAN, "--json"][..],
        &["status", "--json"][..],
        &["requirement", "show", LATE_FINE, "--json"][..],
        &["check", "show", LATE_RETURN, "--json"][..],
        &["change", "show", "--json"][..],
    ] {
        let output = repo.azimuth(args);
        assert!(output.status == 0 || output.status == 1, "azimuth {args:?}:\n{}", output.text());
        let format = output.json()["format"].as_str().unwrap_or_default().to_string();
        assert!(format.ends_with("-1"), "azimuth {args:?} names format {format:?}");
    }
}

// behavior: It ends with status 0 whenever it could answer, also when a Check is failed or owed
#[test]
fn reading_commands_answer_with_zero() {
    let repo = known_fail();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    for args in [&["status"][..], &["requirement", "show", LATE_FINE][..], &["check", "show", LATE_RETURN][..], &["change", "show"][..]] {
        repo.azimuth(args).expect_status(0);
    }
}

// behavior: Its document is `status-1` with these fields
#[test]
fn status_document_has_its_fields() {
    let repo = known_fail();
    let status = repo.status_json();
    let requirements = status["requirements"].as_array().expect("requirements");
    assert_eq!(requirements.len(), 2);
    for requirement in requirements {
        for field in ["id", "module", "state", "verification", "realization", "approval", "scenarios"] {
            assert!(requirement.get(field).is_some(), "no `{field}`: {requirement:#}");
        }
        for scenario in requirement["scenarios"].as_array().unwrap() {
            assert!(scenario.get("id").is_some() && scenario.get("checks").is_some(), "{scenario:#}");
            for check in scenario["checks"].as_array().unwrap() {
                for field in ["id", "state", "reason"] {
                    assert!(check.get(field).is_some(), "no `{field}`: {check:#}");
                }
            }
        }
    }
    assert!(status["skipped_records"].is_u64(), "{status:#}");
    assert!(status.get("process").is_some(), "{status:#}");
}

// behavior: A Check with a stored fail on the current fingerprint is `owed` with the reason `failed on these inputs`, and no Check is `passed` or `failed`
#[test]
fn status_shows_a_known_fail_as_owed() {
    let repo = known_fail();
    let status = repo.status_json();
    let checks = status_checks(&status);
    assert_eq!(checks[LATE_RETURN]["state"], "owed");
    assert_eq!(checks[LATE_RETURN]["reason"], "failed on these inputs");
    let states: BTreeSet<String> = checks.values().map(|check| check["state"].as_str().unwrap().to_string()).collect();
    assert!(!states.contains("passed") && !states.contains("failed"), "{states:?}");
}

// behavior: It has a structured form with the same content
#[test]
fn the_text_and_the_document_say_the_same() {
    let repo = known_fail();
    repo.append(RETURNS_SPEC, "\n#### Scenario: lost-book-is-charged\nA book reported lost is charged at its price.\n");
    let text = repo.azimuth(&["status"]).stdout;
    let status = repo.status_json();
    for requirement in status["requirements"].as_array().unwrap() {
        let id = requirement["id"].as_str().unwrap();
        let listed = text.contains(id);
        assert_eq!(listed, requirement["state"] != "proven", "{id} in text: {listed}\n{text}\n{status:#}");
        if listed {
            for (check_id, check) in status_checks(&serde_json::json!({ "requirements": [requirement] })) {
                assert!(text.contains(&check_id), "{check_id} is missing from the text:\n{text}");
                assert!(text.contains(check["state"].as_str().unwrap()), "{text}");
            }
        }
    }
}
