//! Group "The model files" of docs/increments/1.md.

use azimuth_tests::*;

fn status(repo: &Repo) -> serde_json::Value {
    let output = repo.azimuth(&["status", "--json"]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "status-1", "{document:#}");
    document
}

fn scenario_ids(status: &serde_json::Value, requirement: &str) -> Vec<String> {
    let requirement = requirement_in(status, requirement)
        .unwrap_or_else(|| panic!("status does not list {requirement}: {status:#}"));
    requirement["scenarios"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|scenario| scenario["id"].as_str().unwrap_or_default().to_string())
        .collect()
}

fn lint_is_clean(repo: &Repo) {
    let output = repo.azimuth(&["lint", "--json"]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "lint-1");
    assert_eq!(count(&document["findings"]), 0, "{document:#}");
}

// behavior: The model is read from module folders at any depth under `azimuth/model/`
#[test]
fn modules_are_read_at_any_depth() {
    let repo = Repo::sample();
    let document = status(&repo);
    let lending = requirement_in(&document, LOAN_LIMIT).expect("the Requirement of azimuth/model/lending");
    let returns = requirement_in(&document, LATE_FINE).expect("the Requirement of azimuth/model/desk/returns");
    assert!(lending["module"].as_str().unwrap_or_default().contains("lending"), "{lending:#}");
    assert!(returns["module"].as_str().unwrap_or_default().contains("returns"), "{returns:#}");
}

// behavior: A module folder may lack any of its files, and that is not a finding
#[test]
fn a_module_folder_without_files_is_no_finding() {
    // `azimuth/model/desk/` holds only a child folder, and no module of the sample has a README
    // or a realization file. One more module holds only its README.
    let repo = Repo::sample();
    repo.write("azimuth/model/catalog/README.md", "# Catalog\n\nThe list of books the desk can lend.\n");
    lint_is_clean(&repo);
    status(&repo);
}

// behavior: A Requirement or a Scenario is recognized by a heading that holds its structure word and an id
#[test]
fn requirements_and_scenarios_come_from_headings() {
    let repo = Repo::sample();
    let document = status(&repo);
    assert_eq!(scenario_ids(&document, LOAN_LIMIT), vec![SIXTH_LOAN, RETURNED_BOOK]);
    assert_eq!(scenario_ids(&document, LATE_FINE), vec![LATE_RETURN, ON_TIME_RETURN]);
    assert_eq!(document["requirements"].as_array().unwrap().len(), 2, "{document:#}");
}

// behavior: A heading is recognized at any heading level
#[test]
fn any_heading_level_holds_a_structure_word() {
    let repo = Repo::sample();
    for file in [LENDING_SPEC, LENDING_VERIFICATION] {
        repo.replace(file, "### Requirement: loan-limit-holds", "# Requirement: loan-limit-holds");
        repo.replace(file, "#### Scenario: sixth-loan-is-refused", "###### Scenario: sixth-loan-is-refused");
        repo.replace(file, "#### Scenario: returned-book-frees-a-slot", "## Scenario: returned-book-frees-a-slot");
    }
    lint_is_clean(&repo);
    let document = status(&repo);
    assert_eq!(scenario_ids(&document, LOAN_LIMIT), vec![SIXTH_LOAN, RETURNED_BOOK]);
}

// behavior: A line that starts with `Context`, `Realization`, `Verification` or `Reuse` and a colon is read as a declaration, wherever it stands under its heading
#[test]
fn a_declaration_is_read_in_the_middle_of_prose() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.\n",
        "A borrower holds at most five loans at a time.\nVerification: reviewed\nThe limit counts books, not visits.\n",
    );
    repo.replace(
        LENDING_VERIFICATION,
        "  The desk refuses a sixth loan to a borrower who holds five.\n",
        "  The desk refuses a sixth loan\n  Reuse: never, because the desk reads today's date\n  to a borrower who holds five.\n",
    );
    let document = status(&repo);
    let requirement = requirement_in(&document, LOAN_LIMIT).unwrap();
    assert!(
        requirement["verification"].as_str().unwrap_or_default().contains("this version records no reviews"),
        "the declaration in the middle of the prose was not read: {requirement:#}"
    );
    let check = repo.azimuth(&["check", "show", SIXTH_LOAN, "--json"]);
    check.expect_status(0);
    assert!(all_strings(&check.json()["reuse"]).iter().any(|value| value.contains("never")));
}

// behavior: A blank line between a heading and a declaration changes nothing
#[test]
fn a_blank_line_before_a_declaration_changes_nothing() {
    let tight = Repo::sample();
    tight.replace(
        LENDING_SPEC,
        "### Requirement: loan-limit-holds\n",
        "### Requirement: loan-limit-holds\nVerification: reviewed\n",
    );
    let loose = Repo::sample();
    loose.replace(
        LENDING_SPEC,
        "### Requirement: loan-limit-holds\n",
        "### Requirement: loan-limit-holds\n\n\nVerification: reviewed\n",
    );
    let tight_requirement = requirement_in(&status(&tight), LOAN_LIMIT).cloned().unwrap();
    let loose_requirement = requirement_in(&status(&loose), LOAN_LIMIT).cloned().unwrap();
    assert!(tight_requirement["verification"].as_str().unwrap_or_default().contains("this version records no reviews"));
    assert_eq!(tight_requirement["verification"], loose_requirement["verification"]);
}

// behavior: A line that starts with any other word and a colon is prose
#[test]
fn another_word_with_a_colon_is_prose() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.\n",
        "A borrower holds at most five loans at a time.\nNote: the desk counts renewals as one loan.\nGiven: a borrower card.\n",
    );
    lint_is_clean(&repo);
    let requirement = requirement_in(&status(&repo), LOAN_LIMIT).cloned().unwrap();
    assert_eq!(requirement["verification"], "not reviewed", "{requirement:#}");
}

// behavior: A key or a table in the project file that the CLI does not know is a reading error
#[test]
fn an_unknown_key_in_the_project_file_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "[repository]\n", "colour = \"blue\"\n\n[repository]\n");
    let line = repo.line_of(PROJECT_FILE, "colour");
    repo.expect_reading_error(Some(PROJECT_FILE), Some(line));
}

// behavior: A key or a table in the project file that the CLI does not know is a reading error
#[test]
fn an_unknown_table_in_the_project_file_is_a_reading_error() {
    let repo = Repo::sample();
    repo.append(PROJECT_FILE, "\n[shelf]\nname = \"north\"\n");
    repo.expect_reading_error(Some(PROJECT_FILE), None);
}

// behavior: A key or a table in the project file that the CLI does not know is a reading error
#[test]
fn an_unknown_key_inside_a_suite_is_a_reading_error() {
    // A mistyped key must not be ignored, because the suite would then run without its proof.
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "needs = []", "neds = [\"shelf-scanner\"]");
    let line = repo.line_of(PROJECT_FILE, "neds");
    repo.expect_reading_error(Some(PROJECT_FILE), Some(line));
}

// behavior: The text under a heading is accepted in any language and is never judged by lint
#[test]
fn prose_in_any_language_is_accepted() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.\n",
        "Читатель держит не больше пяти книг одновременно.\nОқырман бір мезгілде бестен артық кітап ұстамайды.\n",
    );
    repo.replace(
        LENDING_VERIFICATION,
        "  The desk refuses a sixth loan to a borrower who holds five.\n",
        "  Стойка отказывает в шестой книге.\n",
    );
    lint_is_clean(&repo);
    status(&repo);
}

// behavior: No Scenario is refused for lacking Given, When, Then
#[test]
fn a_scenario_without_given_when_then_is_accepted() {
    let repo = Repo::sample();
    assert!(!repo.read(LENDING_SPEC).contains("Given"));
    repo.replace(
        LENDING_SPEC,
        "The desk lends it.\n",
        "The desk lends it.\n\n#### Scenario: renewal-keeps-the-count\nA renewal does not add a loan.\n",
    );
    lint_is_clean(&repo);
    let document = status(&repo);
    assert!(scenario_ids(&document, LOAN_LIMIT).contains(&"renewal-keeps-the-count".to_string()));
}

// behavior: A Check is read from `verification.md` under its Scenario, as an id, a claim and an optional `Reuse:`
#[test]
fn a_check_is_read_with_its_claim_and_its_reuse() {
    let repo = Repo::sample();
    let plain = repo.azimuth(&["check", "show", SIXTH_LOAN, "--json"]);
    plain.expect_status(0);
    let plain = plain.json();
    assert_eq!(plain["format"], "check-1");
    assert_eq!(plain["id"], SIXTH_LOAN);
    assert_eq!(plain["requirement"], LOAN_LIMIT);
    assert_eq!(plain["scenario"], SIXTH_LOAN);
    assert!(plain["claim"].as_str().unwrap_or_default().contains("The desk refuses a sixth loan"), "{plain:#}");
    assert!(!all_strings(&plain["reuse"]).iter().any(|value| value.contains("never")), "{plain:#}");

    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never, because the desk reads today's date\n",
    );
    let strict = repo.azimuth(&["check", "show", ON_TIME_RETURN, "--json"]);
    strict.expect_status(0);
    let strict = strict.json();
    assert_eq!(strict["scenario"], ON_TIME_RETURN);
    assert!(all_strings(&strict["reuse"]).iter().any(|value| value.contains("never")), "{strict:#}");
}

// behavior: A `Reuse:` field without a reason is a reading error
#[test]
fn reuse_without_a_reason_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never\n",
    );
    let line = repo.line_of(RETURNS_VERIFICATION, "Reuse: never");
    repo.expect_reading_error(Some(RETURNS_VERIFICATION), Some(line));
}

// behavior: A Check declared without `Reuse:` is reused by equal fingerprint
#[test]
fn a_check_without_reuse_is_kept_on_an_equal_fingerprint() {
    let repo = Repo::sample();
    repo.azimuth(&["run", "--json"]).expect_status(0);
    repo.clear_log();
    let second = repo.azimuth(&["run", "--json"]);
    second.expect_status(0);
    let document = second.json();
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "kept", "{document:#}");
    }
    assert!(repo.adapter_runs().is_empty(), "the adapter ran for kept Checks");
}

// behavior: Two Checks with one id anywhere in the project are a reading error
#[test]
fn one_check_id_in_two_modules_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(
        RETURNS_VERIFICATION,
        "- Check: on-time-return-is-free\n",
        "- Check: sixth-loan-is-refused\n  The same id as a Check of the lending module.\n\n- Check: on-time-return-is-free\n",
    );
    repo.expect_reading_error(None, None);
}

// behavior: Two Checks with one id anywhere in the project are a reading error
#[test]
fn one_check_id_twice_in_one_file_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_VERIFICATION,
        "- Check: returned-book-frees-a-slot\n",
        "- Check: sixth-loan-is-refused\n  A second declaration of the same id.\n\n- Check: returned-book-frees-a-slot\n",
    );
    repo.expect_reading_error(Some(LENDING_VERIFICATION), None);
}

// behavior: Two Scenarios with one id in one Requirement are a reading error
#[test]
fn one_scenario_id_twice_in_a_requirement_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "#### Scenario: returned-book-frees-a-slot",
        "#### Scenario: sixth-loan-is-refused",
    );
    repo.expect_reading_error(Some(LENDING_SPEC), None);
}

// behavior: Two Scenarios with one id in different Requirements are valid
#[test]
fn one_scenario_id_in_two_requirements_is_valid() {
    let repo = Repo::sample();
    repo.append(
        RETURNS_SPEC,
        "\n#### Scenario: sixth-loan-is-refused\nA borrower with five loans returns a late book; the fine does not depend on the loans.\n",
    );
    lint_is_clean(&repo);
    let document = status(&repo);
    assert!(scenario_ids(&document, LOAN_LIMIT).contains(&SIXTH_LOAN.to_string()));
    assert!(scenario_ids(&document, LATE_FINE).contains(&SIXTH_LOAN.to_string()));
}

// behavior: A Scenario heading with no Requirement heading above it is a reading error, also in `verification.md`
#[test]
fn a_scenario_above_every_requirement_is_a_reading_error_in_the_specification() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "### Requirement: loan-limit-holds\n",
        "#### Scenario: stray-scenario\nA Scenario that belongs to no Requirement.\n\n### Requirement: loan-limit-holds\n",
    );
    let line = repo.line_of(LENDING_SPEC, "stray-scenario");
    repo.expect_reading_error(Some(LENDING_SPEC), Some(line));
}

// behavior: A Scenario heading with no Requirement heading above it is a reading error, also in `verification.md`
#[test]
fn a_scenario_without_its_requirement_heading_is_a_reading_error_in_verification() {
    let repo = Repo::sample();
    repo.replace(LENDING_VERIFICATION, "### Requirement: loan-limit-holds\n", "");
    let line = repo.line_of(LENDING_VERIFICATION, "Scenario: sixth-loan-is-refused");
    repo.expect_reading_error(Some(LENDING_VERIFICATION), Some(line));
}

// behavior: Two Requirements with one id in different modules are a reading error
#[test]
fn one_requirement_id_in_two_modules_is_a_reading_error() {
    let repo = Repo::sample();
    for file in [RETURNS_SPEC, RETURNS_VERIFICATION] {
        repo.replace(file, "Requirement: late-return-owes-a-fine", "Requirement: loan-limit-holds");
    }
    let document = repo.expect_reading_error(None, None);
    let text = document.to_string();
    assert!(text.contains("lending"), "the error names the lending module: {document:#}");
    assert!(text.contains("returns"), "the error names the returns module: {document:#}");
}

fn expect_bad_requirement_id(id: &str) {
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "### Requirement: loan-limit-holds", &format!("### Requirement: {id}"));
    let line = repo.line_of(LENDING_SPEC, "### Requirement:");
    repo.expect_reading_error(Some(LENDING_SPEC), Some(line));
}

// behavior: An id with a capital letter, a space or a letter outside the Latin alphabet is a reading error
#[test]
fn an_id_with_a_capital_letter_is_a_reading_error() {
    expect_bad_requirement_id("Loan-limit-holds");
}

// behavior: An id with a capital letter, a space or a letter outside the Latin alphabet is a reading error
#[test]
fn an_id_with_a_space_is_a_reading_error() {
    expect_bad_requirement_id("loan limit holds");
}

// behavior: An id with a capital letter, a space or a letter outside the Latin alphabet is a reading error
#[test]
fn an_id_with_a_cyrillic_letter_is_a_reading_error() {
    // The "о" below is Cyrillic. It looks like the Latin letter and must not pass as one.
    expect_bad_requirement_id("loan-limit-h\u{043e}lds");
}

// behavior: An id with a capital letter, a space or a letter outside the Latin alphabet is a reading error
#[test]
fn a_check_id_with_a_capital_letter_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(LENDING_VERIFICATION, "- Check: sixth-loan-is-refused", "- Check: Sixth-loan-is-refused");
    let line = repo.line_of(LENDING_VERIFICATION, "Sixth-loan-is-refused");
    repo.expect_reading_error(Some(LENDING_VERIFICATION), Some(line));
}

// behavior: A heading with a structure word and no id is a reading error
#[test]
fn a_requirement_heading_without_an_id_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "### Requirement: loan-limit-holds", "### Requirement:");
    let line = repo.line_of(LENDING_SPEC, "### Requirement:");
    repo.expect_reading_error(Some(LENDING_SPEC), Some(line));
}

// behavior: A heading with a structure word and no id is a reading error
#[test]
fn a_scenario_heading_without_an_id_is_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(RETURNS_SPEC, "#### Scenario: on-time-return-is-free", "#### Scenario:");
    let line = repo.line_of(RETURNS_SPEC, "#### Scenario:");
    repo.expect_reading_error(Some(RETURNS_SPEC), Some(line));
}

fn with_reading_error() -> Repo {
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "### Requirement: loan-limit-holds", "### Requirement: Loan-limit-holds");
    repo
}

const COMMANDS_THAT_READ: [&[&str]; 7] = [
    &["run", "--json"],
    &["check", "run", SIXTH_LOAN, "--json"],
    &["status", "--json"],
    &["requirement", "show", LATE_FINE, "--json"],
    &["check", "show", LATE_RETURN, "--json"],
    &["change", "show", "--json"],
    &["lint", "--json"],
];

// behavior: On a reading error `azimuth run`, `azimuth check run` and the reading commands print the message, do nothing else and end with failure
#[test]
fn a_reading_error_stops_every_command() {
    let repo = with_reading_error();
    for args in COMMANDS_THAT_READ {
        let output = repo.azimuth(args);
        assert_eq!(output.status, 2, "azimuth {args:?} did not stop on the reading error:\n{}", output.text());
        let document = output.json();
        assert_eq!(document["format"], "error-1", "azimuth {args:?}: {document:#}");
        assert!(!document.as_object().unwrap().contains_key("requirements"), "azimuth {args:?} answered anyway");
        assert!(!document.as_object().unwrap().contains_key("checks"), "azimuth {args:?} answered anyway");
    }
    assert!(repo.adapter_starts().is_empty(), "a command started the adapter despite the reading error");
    assert!(repo.store_files().is_empty(), "a command wrote the store despite the reading error");
    assert!(repo.outbox_files().is_empty(), "a command wrote the outbox despite the reading error");
}

// behavior: On a reading error `azimuth run`, `azimuth check run` and the reading commands print the message, do nothing else and end with failure
#[test]
fn a_reading_error_is_printed_as_text_without_the_flag() {
    let repo = with_reading_error();
    let output = repo.azimuth(&["status"]);
    output.expect_status(2);
    assert!(output.text().contains("Loan-limit-holds"), "the message names the wrong id:\n{}", output.text());
    assert!(output.text().contains(LENDING_SPEC), "the message names the file:\n{}", output.text());
}

// behavior: A reading error has the same words in every command
#[test]
fn a_reading_error_has_the_same_words_in_every_command() {
    let repo = with_reading_error();
    let mut seen: Vec<(String, serde_json::Value)> = Vec::new();
    for args in COMMANDS_THAT_READ {
        let output = repo.azimuth(args);
        output.expect_status(2);
        seen.push((format!("{args:?}"), output.json()["errors"].clone()));
    }
    let (first_command, first_errors) = &seen[0];
    for (command, errors) in &seen[1..] {
        assert_eq!(errors, first_errors, "{command} words the error differently from {first_command}");
    }
    let plain: Vec<String> = [&["status"][..], &["run"][..], &["lint"][..]]
        .iter()
        .map(|args| repo.azimuth(args).text())
        .collect();
    let message = first_errors[0]["message"].as_str().unwrap().to_string();
    for text in plain {
        assert!(text.contains(&message), "the text output does not carry the message {message:?}:\n{text}");
    }
}

// behavior: With a finding that is not a reading error, `azimuth run` and the reading commands work
#[test]
fn a_finding_does_not_stop_run_or_the_reading_commands() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "# Check: late-return-is-fined\n", "");
    let run = repo.azimuth(&["run", "--json"]);
    assert_ne!(run.status, 2, "run stopped on a finding:\n{}", run.text());
    let document = run.json();
    assert_eq!(document["format"], "run-1");
    assert_eq!(state_of(&document, LATE_RETURN), "owed", "{document:#}");
    for check in [SIXTH_LOAN, RETURNED_BOOK, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
    for args in [
        &["status", "--json"][..],
        &["requirement", "show", LATE_FINE, "--json"][..],
        &["check", "show", LATE_RETURN, "--json"][..],
    ] {
        repo.azimuth(args).expect_status(0);
    }
    let status = status(&repo);
    assert_eq!(status_checks(&status)[LATE_RETURN]["state"], "owed", "{status:#}");
}

// behavior: Moving a module folder changes no Requirement id and no fingerprint of a claim
#[test]
fn moving_a_module_keeps_ids_and_claim_digests() {
    let repo = Repo::sample();
    repo.azimuth(&["run", "--json"]).expect_status(0);
    std::fs::rename(repo.path("azimuth/model/desk/returns"), repo.path("azimuth/model/returns")).unwrap();

    let document = status(&repo);
    let moved = requirement_in(&document, LATE_FINE).expect("the moved Requirement keeps its id");
    assert!(!moved["module"].as_str().unwrap_or_default().contains("desk"), "{moved:#}");

    let check = repo.azimuth(&["check", "show", LATE_RETURN, "--json"]);
    check.expect_status(0);
    let parts = parts_changed(&check.json());
    assert!(mentions(&parts, "files"), "the move changed files: {parts:?}");
    assert!(!mentions(&parts, "claim"), "the move changed the claim's digest: {parts:?}");
}
