//! Group "`azimuth status`" of docs/increments/1.md.

use std::collections::BTreeSet;

use azimuth_tests::*;

/// The quiet line names the number of Requirements. The design gives its wording only by an
/// example, so the test asks for the number and the word.
fn is_quiet_line_for_two(line: &str) -> bool {
    line.contains('2') && line.contains("Requirement") && line.contains("proven on this machine")
}

fn non_empty_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
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
    let output = repo.azimuth_with_env(
        &["status", "--json"],
        &[
            ("https_proxy", closed),
            ("HTTPS_PROXY", closed),
            ("ALL_PROXY", closed),
        ],
    );
    output.expect_status(0);
    assert!(
        repo.adapter_starts().is_empty(),
        "status started the adapter"
    );
    assert!(repo.probe_starts().is_empty(), "status proved a facility");
    assert!(
        repo.store_files().is_empty() && repo.outbox_files().is_empty(),
        "status wrote records"
    );
}

// behavior: It lists every Requirement that has less than full evidence, with the reason
#[test]
fn status_lists_incomplete_requirements_with_their_reason() {
    let repo = known_fail();
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(output.stdout.contains(LATE_FINE), "{}", output.stdout);
    assert!(
        !output.stdout.contains(LOAN_LIMIT),
        "a proven Requirement was listed:\n{}",
        output.stdout
    );
    assert!(
        output.stdout.contains("failed on these inputs"),
        "{}",
        output.stdout
    );
    let status = repo.status_json();
    assert_ne!(
        requirement_in(&status, LATE_FINE).unwrap()["state"],
        "proven"
    );
    assert_eq!(
        requirement_in(&status, LOAN_LIMIT).unwrap()["state"],
        "proven"
    );
}

// behavior: With everything proven it prints one line that names the number of Requirements
#[test]
fn a_proven_tree_prints_one_line() {
    let repo = proven();
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    let lines = non_empty_lines(&output.stdout);
    assert_eq!(lines.len(), 1, "{}", output.stdout);
    assert!(is_quiet_line_for_two(&lines[0]), "{}", lines[0]);
}

// behavior: With no server named in the project file, the quiet line says "proven on this machine" and no pass is marked "local result only"
#[test]
fn without_a_server_the_quiet_line_says_this_machine() {
    let repo = proven();
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(
        output.stdout.contains("proven on this machine"),
        "{}",
        output.stdout
    );
    for args in [
        &["status"][..],
        &["status", "--json"][..],
        &["requirement", "show", LOAN_LIMIT][..],
        &["check", "show", SIXTH_LOAN][..],
    ] {
        assert!(
            !repo.azimuth(args).text().contains("local result only"),
            "azimuth {args:?} marks a local pass"
        );
    }
}

// behavior: With an empty model it prints one line
#[test]
fn an_empty_model_prints_one_line() {
    let repo = Repo::sample();
    repo.remove("azimuth/model");
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert_eq!(
        non_empty_lines(&output.stdout).len(),
        1,
        "{}",
        output.stdout
    );
}

// behavior: It names a Scenario that has no Check
#[test]
fn status_names_a_scenario_without_a_check() {
    let repo = Repo::sample();
    repo.append(
        RETURNS_SPEC,
        "\n#### Scenario: lost-book-is-charged\nA book reported lost is charged at its price.\n",
    );
    repo.run_json(&[], 0);
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(
        output.stdout.contains("lost-book-is-charged"),
        "{}",
        output.stdout
    );
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
    repo.write(
        "suites/desk-checks/catalog.test",
        "test: lists-titles\noutcome: failed\n\ntest: lists-authors\noutcome: passed\n",
    );
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
    assert!(
        !keys
            .iter()
            .any(|key| key.contains("unmarked") || key.contains("coverage")),
        "{keys:?}"
    );
}

// behavior: A failing test without a marker changes nothing in its output
#[test]
fn an_unmarked_failing_test_leaves_status_quiet() {
    let repo = with_an_unmarked_failing_test();
    let output = repo.azimuth(&["status"]);
    let lines = non_empty_lines(&output.stdout);
    assert_eq!(lines.len(), 1, "{}", output.stdout);
    assert!(is_quiet_line_for_two(&lines[0]), "{}", lines[0]);
}

// behavior: For a Requirement that declares no Verification Review it says "verification: not reviewed"
#[test]
fn no_declared_verification_review_reads_not_reviewed() {
    let repo = Repo::sample();
    let status = repo.status_json();
    assert_eq!(
        requirement_in(&status, LOAN_LIMIT).unwrap()["verification"],
        json!({ "state": "not reviewed" })
    );
    let text = repo.azimuth(&["requirement", "show", LOAN_LIMIT]).stdout;
    assert!(text.contains("verification: not reviewed"), "{text}");
}

// behavior: For a Requirement that declares no Realization Review it says "realization: not reviewed"
#[test]
fn no_declared_realization_review_reads_not_reviewed() {
    let repo = Repo::sample();
    let status = repo.status_json();
    assert_eq!(
        requirement_in(&status, LATE_FINE).unwrap()["realization"],
        json!({ "state": "not reviewed" })
    );
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
        assert_eq!(
            requirement[field],
            json!({ "state": "owed", "reason": "this version records no reviews" }),
            "{field}: {requirement:#}"
        );
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
    assert!(
        !output.stdout.contains("proven on this machine"),
        "{}",
        output.stdout
    );
    let status = repo.status_json();
    assert_ne!(
        requirement_in(&status, LOAN_LIMIT).unwrap()["state"],
        "proven"
    );
    assert_eq!(
        requirement_in(&status, LATE_FINE).unwrap()["state"],
        "proven"
    );
}

// behavior: It says in one line how many stored records it skipped as unreadable
#[test]
fn status_counts_skipped_records() {
    let repo = proven();
    let before = repo.status_json();
    assert_eq!(before["skipped_records"], 0, "{before:#}");
    repo.rewrite_records(next_format);
    let status = repo.status_json();
    let skipped = status["skipped_records"]
        .as_u64()
        .expect("a number of skipped records");
    assert_eq!(skipped, 4, "{status:#}");
    // The wording of that line is text for a person, so the test looks only for the number.
    let text = repo.azimuth(&["status"]).stdout;
    let number = skipped.to_string();
    assert!(
        text.lines().any(|line| line
            .split(|c: char| !c.is_ascii_digit())
            .any(|word| word == number)),
        "no line names the {skipped} skipped records:\n{text}"
    );
}

// behavior: It says "approval: not recorded"
#[test]
fn approval_is_not_recorded() {
    let repo = proven();
    let status = repo.status_json();
    for id in [LOAN_LIMIT, LATE_FINE] {
        assert_eq!(
            requirement_in(&status, id).unwrap()["approval"],
            "not recorded"
        );
    }
    let text = repo.azimuth(&["requirement", "show", LOAN_LIMIT]).stdout;
    assert!(text.contains("approval: not recorded"), "{text}");
}

// behavior: It says "the process here differs from the default" when an installed skill file was edited
#[test]
fn an_edited_skill_shows_a_process_of_its_own() {
    let repo = Repo::sample();
    repo.azimuth(&["skills", "install", "claude"])
        .expect_status(0);
    let untouched = repo.status_json();
    assert_eq!(untouched["process"], "default", "{untouched:#}");
    repo.replace(SKILL_FILE, "\n", "\nStop after the build as well.\n");
    let status = repo.status_json();
    assert_eq!(status["process"], "differs", "{status:#}");
    let text = repo.azimuth(&["status"]).stdout;
    assert!(
        text.contains("the process here differs from the default"),
        "{text}"
    );
}

// behavior-2-exploration-skill: `azimuth status` says that the process here differs from the default when only the exploration skill was edited
#[test]
fn only_an_edited_exploration_skill_shows_a_process_of_its_own() {
    let repo = Repo::sample();
    repo.azimuth(&["skills", "install", "claude"]).expect_status(0);
    assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed");
    let change_text = repo.read(SKILL_FILE);
    let untouched = repo.status_json();
    assert_eq!(untouched["process"], "default", "{untouched:#}");
    repo.replace(EXPLORATION_SKILL_FILE, "\n", "\nA team edit to the exploration form.\n");
    assert_eq!(repo.read(SKILL_FILE), change_text, "the change skill was also edited");
    let status = repo.status_json();
    assert_eq!(status["process"], "differs", "{status:#}");
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    assert!(output.stdout.contains("the process here differs from the default"), "{}", output.text());
}

// behavior: With `--json`, standard output holds exactly one JSON document and progress goes to standard error
#[test]
fn json_output_is_one_document() {
    let repo = Repo::sample();
    for args in [
        &["status", "--json"][..],
        &["run", "--json"][..],
        &["run", "--lane", "commit", "--json"][..],
        &["lint", "--json"][..],
    ] {
        let output = repo.azimuth(args);
        assert!(output.status == 0, "azimuth {args:?}:\n{}", output.text());
        output.json();
    }
}

// behavior: Without `--json` the output is text, also when it is piped
#[test]
fn piped_output_without_the_flag_is_text() {
    let repo = known_fail();
    for args in [
        &["status"][..],
        &["run"][..],
        &["lint"][..],
        &["check", "show", LATE_RETURN][..],
        &["requirement", "show", LATE_FINE][..],
    ] {
        let output = repo.azimuth(args);
        assert!(
            !output.stdout.trim().is_empty(),
            "azimuth {args:?} printed nothing"
        );
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
        (
            &["requirement", "show", LATE_FINE, "--json"][..],
            "requirement-1",
        ),
        (&["check", "show", LATE_RETURN, "--json"][..], "check-1"),
        (&["change", "diff", "--json"][..], "change-1"),
        (&["lint", "--json"][..], "lint-1"),
        (&["run", "--json"][..], "run-1"),
    ] {
        assert_eq!(
            repo.azimuth(args).json()["format"],
            format,
            "azimuth {args:?}"
        );
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
    for args in [
        &["status"][..],
        &["check", "show", LATE_RETURN][..],
        &["requirement", "show", LATE_FINE][..],
    ] {
        let text = repo.azimuth(args).stdout;
        assert!(
            text.contains(state) && text.contains(reason),
            "azimuth {args:?}:\n{text}"
        );
    }
}

// behavior: On a reading error with `--json`, the output is a document of the format `error-1` with the file, the line and the message
#[test]
fn a_reading_error_is_an_error_1_document() {
    let repo = Repo::sample();
    repo.replace(
        RETURNS_SPEC,
        "#### Scenario: late-return-is-fined",
        "#### Scenario: late-return-is-fined\n#### Scenario: late-return-is-fined",
    );
    let document = repo.expect_reading_error(Some(RETURNS_SPEC), None);
    assert!(
        document["errors"][0]["line"].as_u64().is_some(),
        "{document:#}"
    );
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
        &["change", "diff", "--json"][..],
    ] {
        let output = repo.azimuth(args);
        assert!(
            output.status == 0 || output.status == 1,
            "azimuth {args:?}:\n{}",
            output.text()
        );
        let format = output.json()["format"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        assert!(
            format.ends_with("-1"),
            "azimuth {args:?} names format {format:?}"
        );
    }
}

// behavior: It ends with status 0 whenever it could answer, also when a Check is failed or owed
#[test]
fn reading_commands_answer_with_zero() {
    let repo = known_fail();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    for args in [
        &["status"][..],
        &["requirement", "show", LATE_FINE][..],
        &["check", "show", LATE_RETURN][..],
        &["change", "diff"][..],
    ] {
        repo.azimuth(args).expect_status(0);
    }
}

// behavior: Its document is `status-1` with these fields
#[test]
fn status_document_has_its_fields() {
    let repo = known_fail();
    let status = repo.status_json();
    let mut states = BTreeSet::new();
    let requirements = status["requirements"].as_array().expect("requirements");
    assert_eq!(requirements.len(), 2);
    for requirement in requirements {
        for field in [
            "id",
            "module",
            "state",
            "verification",
            "realization",
            "approval",
            "scenarios",
        ] {
            assert!(has(requirement, field), "no `{field}`: {requirement:#}");
        }
        for scenario in requirement["scenarios"].as_array().unwrap() {
            assert!(
                has(scenario, "id") && has(scenario, "checks"),
                "{scenario:#}"
            );
            for check in scenario["checks"].as_array().unwrap() {
                assert!(has(check, "id") && has(check, "state"), "{check:#}");
                // A field with nothing to say is absent: a kept Check has no reason and no detail.
                match check["state"].as_str().unwrap_or_default() {
                    "owed" => assert!(
                        has(check, "reason"),
                        "an owed Check has no reason: {check:#}"
                    ),
                    "kept" => assert!(
                        check.get("reason").is_none() && check.get("detail").is_none(),
                        "a kept Check carries a reason or a detail: {check:#}"
                    ),
                    other => panic!("status shows a Check as {other:?}: {check:#}"),
                }
                states.insert(check["state"].as_str().unwrap_or_default().to_string());
            }
        }
    }
    assert_eq!(
        states,
        set_of(&["kept", "owed"]),
        "the sample shows both states: {status:#}"
    );
    assert!(status["skipped_records"].is_u64(), "{status:#}");
    assert!(has(&status, "process"), "{status:#}");
}

// behavior: A Check with a stored fail on the current fingerprint is `owed` with the reason `failed on these inputs`, and no Check is `passed` or `failed`
#[test]
fn status_shows_a_known_fail_as_owed() {
    let repo = known_fail();
    let status = repo.status_json();
    let checks = status_checks(&status);
    assert_eq!(checks[LATE_RETURN]["state"], "owed");
    assert_eq!(checks[LATE_RETURN]["reason"], "failed on these inputs");
    let states: BTreeSet<String> = checks
        .values()
        .map(|check| check["state"].as_str().unwrap().to_string())
        .collect();
    assert!(
        !states.contains("passed") && !states.contains("failed"),
        "{states:?}"
    );
}

// behavior: It has a structured form with the same content
#[test]
fn the_text_and_the_document_say_the_same() {
    let repo = known_fail();
    repo.append(
        RETURNS_SPEC,
        "\n#### Scenario: lost-book-is-charged\nA book reported lost is charged at its price.\n",
    );
    let text = repo.azimuth(&["status"]).stdout;
    let status = repo.status_json();
    for requirement in status["requirements"].as_array().unwrap() {
        let id = requirement["id"].as_str().unwrap();
        let listed = text.contains(id);
        assert_eq!(
            listed,
            requirement["state"] != "proven",
            "{id} in text: {listed}\n{text}\n{status:#}"
        );
        if listed {
            for (check_id, check) in
                status_checks(&serde_json::json!({ "requirements": [requirement] }))
            {
                assert!(
                    text.contains(&check_id),
                    "{check_id} is missing from the text:\n{text}"
                );
                assert!(text.contains(check["state"].as_str().unwrap()), "{text}");
            }
        }
    }
}

// behavior: In every document `reason` is exactly one of the design's reasons for owed, and anything more is in `detail`
#[test]
fn every_reason_is_one_of_the_named_reasons() {
    let repo = Repo::sample();
    repo.split_returns_suite("\"shelf-scanner\"");
    repo.append(
        PROJECT_FILE,
        "\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe absent\"\n",
    );
    repo.replace(
        LENDING_VERIFICATION,
        "  After one return the desk lends again to a borrower who held five loans.\n",
        "  After one return the desk lends again to a borrower who held five loans.\n  Reuse: never, the desk reads today's date\n",
    );
    let fresh = repo.status_json();
    let run = repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    let changed = repo.status_json();
    let mut documents: Vec<serde_json::Value> = vec![run];
    for status in [&fresh, &changed] {
        documents
            .push(json!({ "checks": status_checks(status).into_values().collect::<Vec<_>>() }));
    }
    for check in ALL_CHECKS {
        documents.push(json!({ "checks": [repo.check_show(check)] }));
    }
    let mut seen = BTreeSet::new();
    for document in &documents {
        for entry in document["checks"].as_array().unwrap() {
            if entry["state"] == "owed" {
                let reason = entry["reason"].as_str().unwrap_or_default();
                assert!(
                    OWED_REASONS.contains(&reason),
                    "{reason:?} is not a named reason: {entry:#}"
                );
                seen.insert(reason.to_string());
            }
        }
    }
    assert_eq!(
        seen,
        set_of(&[
            "never run",
            "cannot run here",
            "runs in CI",
            "an input changed"
        ]),
        "{documents:#?}"
    );
    let check = repo.check_show(SIXTH_LOAN);
    assert_eq!(
        check["reason"], "an input changed",
        "the part goes into the detail and the lists: {check:#}"
    );
}

// behavior: The `status-1` document lists every Requirement, also the proven ones, and the text lists only the incomplete ones
#[test]
fn the_document_lists_proven_requirements_and_the_text_does_not() {
    let repo = known_fail();
    let status = repo.status_json();
    let ids: Vec<&str> = status["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 2, "{status:#}");
    assert!(
        ids.contains(&LOAN_LIMIT) && ids.contains(&LATE_FINE),
        "{status:#}"
    );
    assert_eq!(
        requirement_in(&status, LOAN_LIMIT).unwrap()["state"],
        "proven"
    );
    let text = repo.azimuth(&["status"]).stdout;
    assert!(
        text.contains(LATE_FINE) && !text.contains(LOAN_LIMIT),
        "{text}"
    );

    let proven = proven();
    assert_eq!(count(&proven.status_json()["requirements"]), 2);
}

// behavior: `module` is the folder path under `azimuth/model`
#[test]
fn module_is_the_folder_path_under_the_model() {
    let repo = Repo::sample();
    let status = repo.status_json();
    assert_eq!(
        requirement_in(&status, LOAN_LIMIT).unwrap()["module"],
        "lending"
    );
    assert_eq!(
        requirement_in(&status, LATE_FINE).unwrap()["module"],
        "desk/returns"
    );
}

// behavior: `verification` and `realization` hold a `state` of `not reviewed` or `owed`, with a `reason` when owed
#[test]
fn reviews_hold_a_state_and_a_reason_when_owed() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.\n",
        "A borrower holds at most five loans at a time.\nVerification: reviewed\n",
    );
    let status = repo.status_json();
    let lending = requirement_in(&status, LOAN_LIMIT).unwrap();
    assert_eq!(
        lending["verification"],
        json!({ "state": "owed", "reason": "this version records no reviews" })
    );
    assert_eq!(lending["realization"], json!({ "state": "not reviewed" }));
    let returns = requirement_in(&status, LATE_FINE).unwrap();
    assert_eq!(returns["verification"], json!({ "state": "not reviewed" }));
}

// behavior: `approval` is `not recorded`, and `process` is `default` or `differs`
#[test]
fn approval_and_process_are_fixed_words() {
    let repo = proven();
    let status = repo.status_json();
    for requirement in status["requirements"].as_array().unwrap() {
        assert_eq!(requirement["approval"], "not recorded", "{requirement:#}");
    }
    assert_eq!(status["process"], "default", "{status:#}");
    repo.azimuth(&["skills", "install", "codex"])
        .expect_status(0);
    assert_eq!(repo.status_json()["process"], "default");
    repo.replace(SKILL_FILE, "\n", "\nOur team stops after the build too.\n");
    assert_eq!(repo.status_json()["process"], "differs");
}

// behavior: In `error-1`, `file` is a path from the repository root and `line` counts from 1, and for the project file it is the line of the key
#[test]
fn an_error_names_the_file_from_the_root_and_the_line_from_one() {
    let model = Repo::sample();
    model.replace(
        RETURNS_VERIFICATION,
        "- Check: late-return-is-fined",
        "- Check: Late-return-is-fined",
    );
    let document = model.expect_reading_error(None, None);
    assert_eq!(
        document["errors"][0]["file"], RETURNS_VERIFICATION,
        "{document:#}"
    );
    assert_eq!(document["errors"][0]["line"], 3, "{document:#}");

    let first_line = Repo::sample();
    first_line.replace(LENDING_SPEC, "# Lending\n", "### Requirement: Lending\n");
    let document = first_line.expect_reading_error(None, None);
    assert_eq!(document["errors"][0]["file"], LENDING_SPEC, "{document:#}");
    assert_eq!(document["errors"][0]["line"], 1, "{document:#}");

    let project = Repo::sample();
    project.replace(PROJECT_FILE, "needs = []", "needs = []\nretries = 2");
    let document = project.expect_reading_error(None, None);
    assert_eq!(document["errors"][0]["file"], PROJECT_FILE, "{document:#}");
    assert_eq!(
        document["errors"][0]["line"],
        project.line_of(PROJECT_FILE, "retries = 2"),
        "{document:#}"
    );
}

// behavior: The `reason` of an owed review is exactly `this version records no reviews`
#[test]
fn an_owed_review_has_its_own_reason() {
    assert!(
        !OWED_REASONS.contains(&"this version records no reviews"),
        "a review's reason is not a Check's"
    );
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "A borrower holds at most five loans at a time.\n", "A borrower holds at most five loans at a time.\nVerification: reviewed\nRealization: reviewed\n");
    let status = repo.status_json();
    let requirement = requirement_in(&status, LOAN_LIMIT).unwrap();
    for field in ["verification", "realization"] {
        assert_eq!(
            requirement[field]["state"], "owed",
            "{field}: {requirement:#}"
        );
        assert_eq!(
            requirement[field]["reason"], "this version records no reviews",
            "{field}: {requirement:#}"
        );
    }
    let output = repo.azimuth(&["requirement", "show", LOAN_LIMIT, "--json"]);
    output.expect_status(0);
    let shown = output.json();
    assert_eq!(shown["format"], "requirement-1", "{shown:#}");
    for field in ["verification", "realization"] {
        assert_eq!(
            shown[field],
            json!({ "state": "owed", "reason": "this version records no reviews" }),
            "{field}: {shown:#}"
        );
    }
}

/// Prints one JSON document with the given command and adds it to the walk.
fn printed(repo: &Repo, args: &[&str], documents: &mut Vec<(String, serde_json::Value)>) {
    let output = repo.azimuth(args);
    documents.push((format!("azimuth {}", args.join(" ")), output.json()));
}

// behavior: No document and no stored record holds `null`
#[test]
fn no_document_and_no_record_holds_null() {
    let mut documents = Vec::new();

    // Kept Checks, Checks that cannot run here, a stored fail and an input change, on a branch.
    let repo = Repo::sample();
    repo.split_returns_suite("\"shelf-scanner\"");
    repo.append(
        PROJECT_FILE,
        "\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe absent\"\n",
    );
    repo.replace(
        LOAN_LIMIT_TESTS,
        "test: lends-after-a-return\n# Check: returned-book-frees-a-slot\noutcome: passed",
        "test: lends-after-a-return\n# Check: returned-book-frees-a-slot\noutcome: failed",
    );
    repo.commit_all("a suite that cannot run here and a failing test");
    printed(&repo, &["run", "--json"], &mut documents);
    printed(&repo, &["run", "--json"], &mut documents);
    printed(
        &repo,
        &["check", "run", SIXTH_LOAN, "--json"],
        &mut documents,
    );
    repo.git(&["checkout", "--quiet", "-b", "change"]);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    for args in [
        &["status", "--json"][..],
        &["requirement", "show", LOAN_LIMIT, "--json"][..],
        &["requirement", "show", LATE_FINE, "--json"][..],
        &["change", "diff", "--json"][..],
        &["lint", "--json"][..],
        &["run", "--lane", "commit", "--json"][..],
    ] {
        printed(&repo, args, &mut documents);
    }
    for check in ALL_CHECKS {
        printed(&repo, &["check", "show", check, "--json"], &mut documents);
    }
    let mut records: Vec<(String, serde_json::Value)> = repo
        .records()
        .into_iter()
        .map(|(path, record)| (path.display().to_string(), record))
        .collect();
    for start in repo.adapter_starts() {
        records.push(("an adapter request".to_string(), start["request"].clone()));
    }

    // Findings, among them one that belongs to no single line, and the commit lane on them.
    let findings = Repo::sample();
    findings.replace(
        LOAN_LIMIT_TESTS,
        "# Check: returned-book-frees-a-slot\n",
        "# Check: returned-book-frees-a-slot\n# Check: renewal-counts-as-one-loan\n",
    );
    findings.add_text_suite("returns-checks", "");
    findings.write(
        "suites/returns-checks/second-limit.test",
        "# Check: sixth-loan-is-refused\noutcome: passed\n",
    );
    printed(&findings, &["lint", "--json"], &mut documents);
    printed(
        &findings,
        &["run", "--lane", "commit", "--json"],
        &mut documents,
    );

    // A reading error.
    let broken = Repo::sample();
    broken.replace(
        LENDING_SPEC,
        "### Requirement: loan-limit-holds",
        "### Requirement: Loan-limit-holds",
    );
    printed(&broken, &["status", "--json"], &mut documents);
    printed(&broken, &["run", "--json"], &mut documents);

    let formats: BTreeSet<String> = documents
        .iter()
        .map(|(_, document)| document["format"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        formats,
        set_of(&[
            "run-1",
            "status-1",
            "requirement-1",
            "check-1",
            "change-1",
            "lint-1",
            "error-1"
        ]),
        "the walk misses a document"
    );
    assert!(!records.is_empty(), "the walk found no record");
    for (origin, value) in documents.iter().chain(records.iter()) {
        assert_no_null(value, origin);
    }
}

fn printed_json(repo: &Repo, args: &[&str]) -> serde_json::Value {
    repo.azimuth(args).json()
}

fn empty_list(document: &serde_json::Value, path: &[&str]) {
    let mut value = document;
    for key in path {
        value = value
            .get(*key)
            .unwrap_or_else(|| panic!("`{}` is absent: {document:#}", path.join(".")));
    }
    assert_eq!(
        *value,
        json!([]),
        "`{}` is not an empty list: {document:#}",
        path.join(".")
    );
}

// behavior: A list in a printed document is present when it is empty
#[test]
fn an_empty_list_is_printed() {
    let repo = Repo::sample();
    empty_list(&printed_json(&repo, &["lint", "--json"]), &["findings"]);
    repo.run_json(&[], 0);
    empty_list(&repo.run_json(&[], 0), &["suites"]);
    let commit = printed_json(&repo, &["run", "--lane", "commit", "--json"]);
    assert_eq!(commit["lane"], "commit", "{commit:#}");
    empty_list(&commit, &["findings"]);
    let kept = repo.check_show(SIXTH_LOAN);
    assert_eq!(kept["state"], "kept", "{kept:#}");
    for list in ["parts_changed", "files_changed", "failing_tests"] {
        empty_list(&kept, &[list]);
    }

    repo.git(&["checkout", "--quiet", "-b", "change"]);
    let change = printed_json(&repo, &["change", "diff", "--json"]);
    assert_eq!(change["format"], "change-1", "{change:#}");
    for kind in ["requirements", "scenarios", "checks"] {
        for which in ["added", "changed", "removed"] {
            empty_list(&change, &[kind, which]);
        }
    }
    empty_list(&change, &["not_compared"]);
    empty_list(&change, &["rigor_lowered"]);

    // A Scenario that no Check proves, and a Requirement with no Scenario, are gaps and no error.
    let gaps = Repo::sample();
    gaps.append(
        RETURNS_SPEC,
        "\n#### Scenario: lost-book-is-charged\nA book reported lost is charged at its price.\n",
    );
    gaps.append(
        LENDING_SPEC,
        "\n### Requirement: renewals-are-limited\nA loan is renewed at most twice.\n",
    );
    let status = gaps.status_json();
    let returns = requirement_in(&status, LATE_FINE).unwrap();
    let lost = returns["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scenario| scenario["id"] == "lost-book-is-charged")
        .cloned();
    empty_list(
        &lost.unwrap_or_else(|| panic!("the Scenario is not listed: {status:#}")),
        &["checks"],
    );
    empty_list(
        requirement_in(&status, "renewals-are-limited").unwrap_or_else(|| panic!("{status:#}")),
        &["scenarios"],
    );
}

// behavior: A finding or a reading error about a whole file or a folder has no `line`
#[test]
fn a_finding_about_a_file_or_a_folder_has_no_line() {
    let repo = Repo::sample();
    repo.write(".azimuth/fake-pass.json", "{}\n");
    repo.git(&["add", "-f", ".azimuth/fake-pass.json"]);
    // A marker that names no Check points at its line, so it keeps one.
    repo.replace(
        LOAN_LIMIT_TESTS,
        "# Check: returned-book-frees-a-slot\n",
        "# Check: returned-book-frees-a-slot\n# Check: renewal-counts-as-one-loan\n",
    );
    let findings = repo.lint_findings(1);
    let whole_file: Vec<&serde_json::Value> = findings
        .iter()
        .filter(|finding| finding["file"] == ".azimuth/fake-pass.json")
        .collect();
    assert_eq!(whole_file.len(), 1, "{findings:#?}");
    assert!(whole_file[0].get("line").is_none(), "{:#}", whole_file[0]);
    let marker_line = repo.line_of(LOAN_LIMIT_TESTS, "renewal-counts-as-one-loan");
    assert_eq!(
        findings_at(&findings, LOAN_LIMIT_TESTS, Some(marker_line)).len(),
        1,
        "{findings:#?}"
    );

    // A listed test whose Check id no Check declares is a finding of the launch about the suite
    // folder, because no line of source shows the id.
    let listed = Repo::sample();
    listed.append(
        LATE_FINE_TESTS,
        "adapter: lists late-fine-from-constant renewal-counts-as-one-loan\n",
    );
    let commit = listed.run_json(&["--lane", "commit"], 1);
    let findings = commit["findings"].as_array().cloned().unwrap_or_default();
    let on_folder: Vec<&serde_json::Value> = findings
        .iter()
        .filter(|finding| finding["file"] == "suites/desk-checks")
        .collect();
    assert_eq!(on_folder.len(), 1, "{commit:#}");
    assert!(on_folder[0].get("line").is_none(), "{:#}", on_folder[0]);
}

// behavior-2: In the text of `azimuth status` a Check that is owed because files changed says how many files differ and names none
#[test]
fn status_counts_changed_files_on_each_owed_check_line() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    for count in [1, 3] {
        if count == 3 {
            repo.write("desk/shelves.txt", "north\n");
            repo.write("desk/catalog.txt", "books\n");
        }
        let output = repo.azimuth(&["status"]);
        output.expect_status(0);
        for check in ALL_CHECKS {
            assert_owed_file_count(&output.stdout, check, count);
        }
        assert_no_file_names(
            &output.stdout,
            &[PRODUCT_FILE, "desk/shelves.txt", "desk/catalog.txt"],
        );
    }
    repo.azimuth(&["check", "run", RETURNED_BOOK, "--json"])
        .expect_status(0);
    repo.write("desk/loans.txt", "one more changed file\n");
    let output = repo.azimuth(&["status"]);
    output.expect_status(0);
    for (check, expected) in [
        (SIXTH_LOAN, 4),
        (RETURNED_BOOK, 1),
        (LATE_RETURN, 4),
        (ON_TIME_RETURN, 4),
    ] {
        assert_owed_file_count(&output.stdout, check, expected);
    }
    assert_no_file_names(
        &output.stdout,
        &[
            PRODUCT_FILE,
            "desk/shelves.txt",
            "desk/catalog.txt",
            "desk/loans.txt",
        ],
    );
}
