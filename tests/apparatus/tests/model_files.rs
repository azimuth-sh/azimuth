//! Group "The model files" of docs/increments/1.md.

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
#[path = "support/contexts_keys.rs"]
mod contexts_keys;
use contexts_conditions::*;
use contexts_keys::*;

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
    assert_eq!(lending["module"], "lending", "{lending:#}");
    assert_eq!(returns["module"], "desk/returns", "{returns:#}");
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
        "  The desk refuses a sixth loan\n  Reuse: never, the desk reads today's date\n  to a borrower who holds five.\n",
    );
    let document = status(&repo);
    let requirement = requirement_in(&document, LOAN_LIMIT).unwrap();
    assert_eq!(
        requirement["verification"]["state"], "owed",
        "the declaration in the middle of the prose was not read: {requirement:#}"
    );
    assert_eq!(repo.check_show(SIXTH_LOAN)["reuse"]["value"], "never");
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
    assert_eq!(tight_requirement["verification"]["state"], "owed", "{tight_requirement:#}");
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
    assert_eq!(requirement["verification"], json!({ "state": "not reviewed" }), "{requirement:#}");
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
    assert!(plain.get("reuse").is_none(), "{plain:#}");

    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never, the desk reads today's date\n",
    );
    let strict = repo.azimuth(&["check", "show", ON_TIME_RETURN, "--json"]);
    strict.expect_status(0);
    let strict = strict.json();
    assert_eq!(strict["scenario"], ON_TIME_RETURN);
    assert_eq!(strict["reuse"]["value"], "never", "{strict:#}");
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

fn reuse_line(line: &str) -> Repo {
    let repo = Repo::sample();
    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        &format!("  A return on the due date or earlier records no fine.\n  {line}\n"),
    );
    repo
}

fn reuse_of(line: &str) -> serde_json::Value {
    reuse_line(line).check_show(ON_TIME_RETURN)["reuse"].clone()
}

fn reuse_reason(reuse: &serde_json::Value) -> String {
    reuse["reason"].as_str().unwrap_or_else(|| panic!("no reason: {reuse:#}")).to_string()
}

fn expect_reuse_reading_error(line: &str) {
    let repo = reuse_line(line);
    let number = repo.line_of(RETURNS_VERIFICATION, line);
    repo.expect_reading_error(Some(RETURNS_VERIFICATION), Some(number));
}

// behavior: A review is declared as `Verification: reviewed` or `Realization: reviewed`
#[test]
fn reviews_are_declared_with_the_word_reviewed() {
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "A borrower holds at most five loans at a time.\n", "A borrower holds at most five loans at a time.\nVerification: reviewed\n");
    repo.replace(RETURNS_SPEC, "A book returned after its due date owes a fine for each day it is late.\n", "A book returned after its due date owes a fine for each day it is late.\nRealization: reviewed\n");
    lint_is_clean(&repo);
    let document = status(&repo);
    let lending = requirement_in(&document, LOAN_LIMIT).unwrap();
    let returns = requirement_in(&document, LATE_FINE).unwrap();
    assert_eq!(lending["verification"]["state"], "owed", "{lending:#}");
    assert_eq!(lending["realization"]["state"], "not reviewed", "{lending:#}");
    assert_eq!(returns["realization"]["state"], "owed", "{returns:#}");
    assert_eq!(returns["verification"]["state"], "not reviewed", "{returns:#}");
}

// behavior: `Reuse:` is written on one line as a value, a comma and a reason
#[test]
fn reuse_is_a_value_a_comma_and_a_reason_on_one_line() {
    let reuse = reuse_of("Reuse: never, the desk reads today's date");
    assert_eq!(reuse["value"], "never", "{reuse:#}");
    assert_eq!(reuse_reason(&reuse), "the desk reads today's date");

    // The next line is claim text again, so a reason never continues past its line.
    let repo = reuse_line("Reuse: never, the desk reads today's date\n  and the clock of the machine");
    let check = repo.check_show(ON_TIME_RETURN);
    assert_eq!(reuse_reason(&check["reuse"]), "the desk reads today's date", "{check:#}");
}

// behavior: The reason is the text after the first comma, without the spaces and tabs around it, and may be in any language
#[test]
fn the_reason_is_the_text_after_the_first_comma_in_any_language() {
    let reuse = reuse_of("Reuse: never, стойка читает сегодняшнюю дату, а не дату выдачи");
    assert_eq!(reuse["value"], "never", "{reuse:#}");
    assert_eq!(reuse_reason(&reuse), "стойка читает сегодняшнюю дату, а не дату выдачи");
}

// behavior: The reason is the text after the first comma, without the spaces and tabs around it, and may be in any language
#[test]
fn the_spaces_around_the_reason_are_not_part_of_it() {
    for line in [
        "Reuse: never,the desk reads today's date",
        "Reuse: never,    the desk reads today's date",
        "Reuse: never, the desk reads today's date   ",
        "Reuse: never,\tthe desk reads today's date",
        "Reuse: never, \t the desk reads today's date\t",
    ] {
        let reuse = reuse_of(line);
        assert_eq!(reuse_reason(&reuse), "the desk reads today's date", "{line:?}: {reuse:#}");
    }
    // Spaces and tabs inside the reason are its own text.
    assert_eq!(reuse_reason(&reuse_of("Reuse: never, the desk  reads today's date")), "the desk  reads today's date");
    assert_eq!(reuse_reason(&reuse_of("Reuse: never, the desk\treads today's date")), "the desk\treads today's date");
}

// behavior: `never` is the one value this version knows
#[test]
fn never_is_the_one_known_value() {
    assert_eq!(reuse_of("Reuse: never, the desk reads today's date")["value"], "never");
    for line in ["Reuse: for 3 days, the shelf list changes weekly", "Reuse: always, nothing outside the tree is read"] {
        expect_reuse_reading_error(line);
    }
}

// behavior: A `Reuse:` line with no comma, with nothing but spaces and tabs after the comma, or with an unknown value is a reading error
#[test]
fn a_malformed_reuse_line_is_a_reading_error() {
    for line in ["Reuse: never the desk reads today's date", "Reuse: never,", "Reuse: never,   ", "Reuse: never,\t", "Reuse: never, \t ", "Reuse: seldom, the desk reads today's date"] {
        expect_reuse_reading_error(line);
    }
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
    let files: Vec<String> = document["errors"].as_array().unwrap().iter().map(|error| error["file"].as_str().unwrap_or_default().to_string()).collect();
    let text = document.to_string();
    assert!(files.iter().any(|file| file == LENDING_SPEC || file == RETURNS_SPEC), "{document:#}");
    assert!(text.contains(LENDING_SPEC) && text.contains(RETURNS_SPEC), "the error names both modules: {document:#}");
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
    // The emptied heading reads like the start of every other Scenario heading, so its line is
    // taken while it still carries its id.
    let line = repo.line_of(RETURNS_SPEC, "#### Scenario: on-time-return-is-free");
    repo.replace(RETURNS_SPEC, "#### Scenario: on-time-return-is-free", "#### Scenario:");
    repo.expect_reading_error(Some(RETURNS_SPEC), Some(line));
}

/// Writes a declaration line after the given text and returns the one `error-1` entry that names
/// the line, with its file and line number.
fn misplaced(file: &str, after: &str, line: &str) -> serde_json::Value {
    let repo = Repo::sample();
    repo.replace(file, after, &format!("{after}{line}\n"));
    let number = repo.line_of(file, line);
    let document = repo.expect_reading_error(Some(file), Some(number));
    let entries: Vec<serde_json::Value> = document["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|error| error["file"] == file && error["line"].as_u64() == Some(number))
        .cloned()
        .collect();
    assert_eq!(entries.len(), 1, "{line:?} in {file}: {document:#}");
    entries[0].clone()
}

// behavior: A `Reuse:` line in `specification.md`, or in `verification.md` under no Check, is a reading error
#[test]
fn a_reuse_line_outside_a_check_is_a_reading_error() {
    misplaced(
        RETURNS_SPEC,
        "A book returned after its due date owes a fine for each day it is late.\n",
        "Reuse: never, the fine table changes each year",
    );
    misplaced(RETURNS_SPEC, "The desk records a fine of three days.\n", "Reuse: never, the desk reads today's date");
    misplaced(RETURNS_VERIFICATION, "#### Scenario: on-time-return-is-free\n", "Reuse: never, the calendar of the desk is read");
    misplaced(RETURNS_VERIFICATION, "### Requirement: late-return-owes-a-fine\n", "Reuse: never, the fines are counted by the clock");
}

// behavior: A `Verification:` or `Realization:` line in `verification.md` is a reading error
#[test]
fn a_review_declaration_in_verification_md_is_a_reading_error() {
    misplaced(RETURNS_VERIFICATION, "### Requirement: late-return-owes-a-fine\n", "Verification: reviewed");
    misplaced(RETURNS_VERIFICATION, "  A return three days late records a fine of three days.\n", "  Realization: reviewed");
    misplaced(RETURNS_VERIFICATION, "#### Scenario: on-time-return-is-free\n", "Verification: reviewed");
}

// behavior: A `Context:` line under a Requirement in `verification.md` is a reading error
#[test]
fn a_context_line_under_a_requirement_is_a_reading_error() {
    misplaced(RETURNS_VERIFICATION, "### Requirement: late-return-owes-a-fine\n", "Context: the desk is open on weekdays");
    misplaced(RETURNS_VERIFICATION, "  A return on the due date or earlier records no fine.\n", "  Context: the due date is printed on the slip");
}

// behavior: The message of a misplaced declaration names the file and the place where the line belongs
#[test]
fn a_misplaced_declaration_names_where_it_belongs() {
    // The wording is free. The message must name the file the line belongs in and the word for
    // the place in that file.
    for (file, after, line, belongs_in, place) in [
        (RETURNS_SPEC, "The desk records a fine of three days.\n", "Reuse: never, the desk reads today's date", "verification.md", "Check"),
        (RETURNS_VERIFICATION, "#### Scenario: on-time-return-is-free\n", "Reuse: never, the calendar of the desk is read", "verification.md", "Check"),
        (RETURNS_VERIFICATION, "### Requirement: late-return-owes-a-fine\n", "Verification: reviewed", "specification.md", "Requirement"),
        (RETURNS_VERIFICATION, "  A return three days late records a fine of three days.\n", "  Realization: reviewed", "specification.md", "Requirement"),
    ] {
        let error = misplaced(file, after, line);
        let message = error["message"].as_str().unwrap_or_default();
        assert!(message.contains(belongs_in), "{line:?} in {file}: the message does not name {belongs_in}: {message}");
        assert!(message.contains(place), "{line:?} in {file}: the message does not name the {place}: {message}");
    }
}

// behavior: A `Reuse:` line in `specification.md`, or in `verification.md` under no Check, is a reading error
#[test]
fn declarations_above_the_first_requirement_stay_prose() {
    // Text above the first Requirement of a file is prose, whatever it looks like, so the same
    // lines that are errors below a heading are read as nothing here.
    let lines = "Reuse: never, the desk reads today's date\nVerification: reviewed\nRealization: reviewed\nContext: the desk is open on weekdays\n\n";
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "### Requirement: loan-limit-holds\n", &format!("{lines}### Requirement: loan-limit-holds\n"));
    repo.replace(RETURNS_VERIFICATION, "### Requirement: late-return-owes-a-fine\n", &format!("{lines}### Requirement: late-return-owes-a-fine\n"));
    lint_is_clean(&repo);
    let document = status(&repo);
    for requirement in [LOAN_LIMIT, LATE_FINE] {
        let entry = requirement_in(&document, requirement).unwrap();
        assert_eq!(entry["verification"], json!({ "state": "not reviewed" }), "{entry:#}");
        assert_eq!(entry["realization"], json!({ "state": "not reviewed" }), "{entry:#}");
    }
    for check in [LATE_RETURN, ON_TIME_RETURN] {
        assert!(repo.check_show(check).get("reuse").is_none(), "{check} took a Reuse line from above the Requirement");
    }
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
    &["change", "diff", "--json"],
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

    let check = repo.check_show(LATE_RETURN);
    assert_eq!(parts_changed(&check), vec!["files"], "the move changed only files: {check:#}");
}

fn key_declaration(name: &str, fields: &str) -> String {
    format!("[{name}]\n{fields}\n")
}

fn key_reading_error(text: &str) {
    let repo = Repo::sample();
    repo.write(KEY_FILE, text);
    // The design gives no wording or exact source line for these diagnostics.
    repo.expect_reading_error(Some(KEY_FILE), None);
}

fn readable_key_with_control(text: &str, table: &str) {
    let repo = Repo::sample();
    repo.write(KEY_FILE, text);
    status(&repo);
    // There is no declaration-list output. An invalid field in this very table proves the
    // reader did not accept the valid form merely by ignoring the whole file.
    repo.replace(KEY_FILE, table, &format!("{table}\nunknown = true"));
    repo.expect_reading_error(Some(KEY_FILE), None);
}

// behavior-contexts-keys: The reader reads key declarations from `azimuth/execution-context.toml`, beside the project file
#[test]
fn key_declarations_are_read_beside_the_project_file() {
    let repo = Repo::sample();
    repo.write("execution-context.toml", "this is not TOML");
    repo.write(KEY_FILE, INSTANT);
    status(&repo);
    repo.write(KEY_FILE, "this is not TOML");
    repo.expect_reading_error(Some(KEY_FILE), None);
}

// behavior-contexts-keys: Each key is a table at its dotted name
#[test]
fn dotted_tables_declare_independent_keys() {
    let text = format!("[payouts]\n\n{INSTANT}\n[payouts.region]\ntype = \"one of\"\nvalues = [\"no\", \"se\"]\ncomponents = [\"api\"]\n");
    for table in ["[payouts.instant]", "[payouts.region]"] {
        readable_key_with_control(&text, table);
    }
}

// behavior-contexts-keys: A key name is lowercase segments joined by dots
#[test]
fn key_names_are_lowercase_dotted_segments() {
    readable_key_with_control(
        &key_declaration("payouts.region.enabled", "type = \"boolean\"\ncomponents = [\"api\"]"),
        "[payouts.region.enabled]",
    );
    for name in ["Payouts.instant", "payouts.Instant", "payouts_instant"] {
        key_reading_error(&key_declaration(name, "type = \"boolean\"\ncomponents = [\"api\"]"));
    }
}

// behavior-contexts-keys: A segment starts with an ASCII lowercase letter and continues with ASCII lowercase letters, digits or hyphens
#[test]
fn key_segments_use_the_ascii_letter_digit_hyphen_grammar() {
    for name in ["payouts.a", "payouts.instant2", "payouts.instant-mode", "payouts.a-"] {
        readable_key_with_control(
            &key_declaration(name, "type = \"boolean\"\ncomponents = [\"api\"]"),
            &format!("[{name}]"),
        );
    }
    for segment in ["2instant", "-instant", "instant_mode", "Instant", "é", "aé", "а", "a２"] {
        key_reading_error(&key_declaration(
            &format!("payouts.\"{segment}\""),
            "type = \"boolean\"\ncomponents = [\"api\"]",
        ));
    }
}

// behavior-contexts-keys: The reader reads no component from a key's name
#[test]
fn payouts_in_a_key_name_does_not_declare_or_require_a_component() {
    let repo = Repo::sample();
    declare_payout_components(&repo);
    repo.write(KEY_FILE, INSTANT);
    lint_is_clean(&repo);
    status(&repo);
}

// behavior-contexts-keys: A key's `type` may be `boolean`
#[test]
fn a_boolean_key_is_read() {
    readable_key_with_control(INSTANT, "[payouts.instant]");
}

// behavior-contexts-keys: A key's `type` may be `one of`
#[test]
fn a_one_of_key_is_read() {
    readable_key_with_control(
        "[payouts.region]\ntype = \"one of\"\nvalues = [\"no\", \"se\"]\ncomponents = [\"api\"]\n",
        "[payouts.region]",
    );
}

// behavior-contexts-keys: A key's `type` may be `number`
#[test]
fn a_number_key_is_read() {
    readable_key_with_control(&INSTANT.replace("boolean", "number"), "[payouts.instant]");
}

// behavior-contexts-keys: A one-of key's `values` declares its allowed strings
#[test]
fn one_of_values_are_strings_in_the_key_table() {
    let repo = Repo::sample();
    repo.write(KEY_FILE, "[payouts.region]\ntype = \"one of\"\nvalues = [\"no\", \"se\", \"\", \"é\", \"É\"]\ncomponents = [\"api\"]\n");
    status(&repo);
    // Conditions belong to the next list. Here the observable is reading the string list.
    repo.replace(KEY_FILE, "\"se\"", "42");
    repo.expect_reading_error(Some(KEY_FILE), None);
}

// behavior-contexts-keys: A key's `components` lists the components that must read it
#[test]
fn components_are_an_explicit_list_on_the_key() {
    let repo = Repo::sample();
    declare_payout_components(&repo);
    repo.write(KEY_FILE, INSTANT);
    lint_is_clean(&repo);
    repo.replace(KEY_FILE, "[\"api\", \"website\"]", "[\"website\"]");
    lint_is_clean(&repo);
    repo.replace(KEY_FILE, "[\"website\"]", "\"website\"");
    repo.expect_reading_error(Some(KEY_FILE), None);
}

// behavior-contexts-keys: An absent `execution-context.toml` declares no keys
#[test]
fn absent_key_declarations_need_no_components() {
    let repo = Repo::sample();
    assert!(!repo.path(KEY_FILE).exists());
    assert!(!parse_toml(&repo.read(PROJECT_FILE)).contains_key("component"));
    lint_is_clean(&repo);
    status(&repo);
}

// behavior-contexts-keys: An empty `execution-context.toml` declares no keys
#[test]
fn empty_key_declarations_are_the_same_answer_as_absent_declarations() {
    let repo = Repo::sample();
    let absent = status(&repo);
    repo.write(KEY_FILE, "");
    lint_is_clean(&repo);
    assert_eq!(status(&repo), absent);
}

// behavior-contexts-keys: Malformed TOML in `execution-context.toml` is a reading error
#[test]
fn malformed_key_toml_is_a_reading_error() {
    key_reading_error("[payouts.instant\ntype = \"boolean\"\ncomponents = [\"api\"]\n");
}

// behavior-contexts-keys: An unknown field in a key table is a reading error
#[test]
fn an_unknown_key_table_field_is_a_reading_error() {
    key_reading_error(&format!("{INSTANT}setting = \"instant\"\n"));
}

// behavior-contexts-keys: An invalid dotted key is a reading error
#[test]
fn empty_key_segments_are_reading_errors() {
    for name in ["\"\"", "payouts.\"\"", "\"\".instant", "payouts.\"\".instant"] {
        key_reading_error(&key_declaration(name, "type = \"boolean\"\ncomponents = [\"api\"]"));
    }
}

// behavior-contexts-keys: A key table with no `type` is a reading error
#[test]
fn a_key_without_type_is_a_reading_error() {
    key_reading_error("[payouts.instant]\ncomponents = [\"api\"]\n");
}

// behavior-contexts-keys: A key table with no `components` is a reading error
#[test]
fn a_key_without_components_is_a_reading_error() {
    key_reading_error("[payouts.instant]\ntype = \"boolean\"\n");
}

// behavior-contexts-keys: An unknown `type` is a reading error
#[test]
fn an_unknown_key_type_is_a_reading_error() {
    for kind in ["string", "Boolean", "one-of", ""] {
        key_reading_error(&INSTANT.replace("boolean", kind));
    }
}

// behavior-contexts-keys: A one-of key with no `values` is a reading error
#[test]
fn a_one_of_key_without_values_is_a_reading_error() {
    key_reading_error("[payouts.region]\ntype = \"one of\"\ncomponents = [\"api\"]\n");
}

// behavior-contexts-keys: A boolean or number key with `values` is a reading error
#[test]
fn values_on_boolean_and_number_keys_are_reading_errors() {
    for kind in ["boolean", "number"] {
        key_reading_error(&format!("{}values = [\"no\", \"se\"]\n", INSTANT.replace("boolean", kind)));
    }
}

// behavior-contexts-keys: An empty one-of list is a reading error
#[test]
fn an_empty_one_of_list_is_a_reading_error() {
    key_reading_error("[payouts.region]\ntype = \"one of\"\nvalues = []\ncomponents = [\"api\"]\n");
}

// behavior-contexts-keys: A one-of list with a repeated value after NFC normalization is a reading error
#[test]
fn repeated_one_of_values_include_nfc_equivalents() {
    for values in ["[\"no\", \"no\"]", "[\"caf\u{e9}\", \"cafe\u{301}\"]"] {
        key_reading_error(&format!("[payouts.region]\ntype = \"one of\"\nvalues = {values}\ncomponents = [\"api\"]\n"));
    }
}

// behavior-contexts-keys: A field of the wrong TOML type is a reading error
#[test]
fn key_fields_reject_wrong_toml_types() {
    for fields in [
        "type = true\ncomponents = [\"api\"]",
        "type = 1\ncomponents = [\"api\"]",
        "type = [\"boolean\"]\ncomponents = [\"api\"]",
        "type = { name = \"boolean\" }\ncomponents = [\"api\"]",
        "type = \"boolean\"\ncomponents = \"api\"",
        "type = \"boolean\"\ncomponents = [1]",
        "type = \"boolean\"\ncomponents = [\"api\", false]",
        "type = \"one of\"\nvalues = \"no\"\ncomponents = [\"api\"]",
        "type = \"one of\"\nvalues = [\"no\", 1]\ncomponents = [\"api\"]",
    ] {
        key_reading_error(&key_declaration("payouts.region", fields));
    }
}

// behavior-contexts-keys: An intermediate table holding anything but nested key tables is a reading error
#[test]
fn intermediate_key_tables_reject_scalar_and_list_fields() {
    for field in ["note = \"product\"", "enabled = true", "names = [\"instant\"]"] {
        key_reading_error(&format!("[payouts]\n{field}\n\n{INSTANT}"));
    }
}

fn broken_key_declaration() -> Repo {
    let repo = Repo::sample();
    repo.write(KEY_FILE, "[payouts.instant]\ntype = \"unknown\"\ncomponents = [\"api\"]\n");
    repo
}

// behavior-contexts-keys: A key-declaration reading error has the same words in every command
#[test]
fn key_reading_errors_have_the_same_words_in_each_command() {
    let repo = broken_key_declaration();
    let expected = repo.expect_reading_error(Some(KEY_FILE), None)["errors"].clone();
    let mut commands = COMMANDS_THAT_READ.to_vec();
    commands.push(&["run", "--lane", "commit", "--json"]);
    for args in commands {
        let output = repo.azimuth(args);
        output.expect_status(2);
        assert_eq!(output.json()["errors"], expected, "{args:?}");
        let plain: Vec<_> = args.iter().copied().filter(|arg| *arg != "--json").collect();
        let output = repo.azimuth(&plain);
        output.expect_status(2);
        for error in expected.as_array().unwrap() {
            let message = error["message"].as_str().unwrap();
            assert!(output.text().contains(message), "{plain:?} omits {message:?}: {}", output.text());
        }
    }
}

// behavior-contexts-keys: On a key-declaration reading error, `azimuth run`, `azimuth check run` and the reading commands do nothing else
#[test]
fn key_reading_errors_stop_commands_before_any_other_work() {
    let repo = broken_key_declaration();
    repo.need_facility("present");
    let before = repo.snapshot();
    for args in COMMANDS_THAT_READ {
        let output = repo.azimuth(args);
        assert_ne!(output.status, 0, "{args:?} succeeded");
        let document = output.json();
        assert_eq!(document["format"], "error-1", "{args:?}: {document:#}");
        assert!(!document["errors"].as_array().unwrap().is_empty(), "{document:#}");
        for field in ["requirements", "checks", "suites", "class", "findings"] {
            assert!(document.get(field).is_none(), "{args:?} answered despite the error: {document:#}");
        }
        assert_eq!(repo.snapshot(), before, "{args:?} changed the worktree");
        assert!(repo.adapter_starts().is_empty(), "{args:?} started an adapter");
        assert!(repo.probe_starts().is_empty(), "{args:?} proved a facility");
        assert!(repo.store_files().is_empty(), "{args:?} wrote the store");
        assert!(repo.outbox_files().is_empty(), "{args:?} wrote the outbox");
    }
}

// behavior-contexts-keys: On a key-declaration reading error, commands end with status 2
#[test]
fn key_reading_errors_end_with_status_two() {
    let repo = broken_key_declaration();
    let mut commands = COMMANDS_THAT_READ.to_vec();
    commands.push(&["run", "--lane", "commit", "--json"]);
    for args in commands {
        repo.azimuth(args).expect_status(2);
    }
}

// Conditions in the model: `docs/increments/contexts-conditions.md`.

/// A `Context:` line stands where it belongs and reads as a condition: the declared key is
/// accepted, and a key that no declaration holds is found by lint at the same place.
fn context_is_read_at(place: impl Fn(&Repo, &[&str])) {
    let repo = started_payouts();
    place(&repo, &["Context: payouts.instant=true"]);
    lint_is_clean(&repo);
    status(&repo);
    let control = started_payouts();
    place(&control, &["Context: payouts.speed=true"]);
    lint_names_the_specification(&control);
}

// behavior-contexts-conditions: The reader accepts `Context:` under a Requirement in `specification.md`
#[test]
fn the_reader_accepts_context_under_a_requirement() {
    context_is_read_at(on_requirement);
    context_is_read_at(|repo, lines| under(repo, PAYOUTS_SPEC, REQUIREMENT_PROSE, lines));
    let repo = payouts();
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    assert_eq!(scenario_ids(&status(&repo), SENT_AT_ONCE), vec![REACHES_THE_CARD, WAITS_FOR_REVIEW]);
}

// behavior-contexts-conditions: The reader accepts `Context:` under a Scenario in `specification.md`
#[test]
fn the_reader_accepts_context_under_a_scenario() {
    context_is_read_at(on_reaches);
    context_is_read_at(|repo, lines| under(repo, PAYOUTS_SPEC, REACHES_PROSE, lines));
    context_is_read_at(on_waits);
    let repo = payouts();
    on_reaches(&repo, &["Context: payouts.instant=true"]);
    assert_eq!(scenario_ids(&status(&repo), SENT_AT_ONCE), vec![REACHES_THE_CARD, WAITS_FOR_REVIEW]);
}

/// Writes a `Context:` line after the given text of a file of the payouts module and returns the
/// one `error-1` entry that names the line.
fn misplaced_context(file: &str, after: &str, line: &str) -> serde_json::Value {
    let repo = payouts();
    repo.replace(file, after, &format!("{after}{line}\n"));
    let number = repo.line_of(file, "Context:");
    let document = repo.expect_reading_error(Some(file), Some(number));
    let entries: Vec<serde_json::Value> = document["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|error| error["file"] == file && error["line"].as_u64() == Some(number))
        .cloned()
        .collect();
    assert_eq!(entries.len(), 1, "{line:?} in {file}: {document:#}");
    entries[0].clone()
}

const CONTEXT_IN_VERIFICATION: [(&str, &str); 5] = [
    ("### Requirement: payout-is-sent-at-once\n", "Context: payouts.instant=true"),
    ("#### Scenario: payout-reaches-the-card\n", "Context: payouts.instant=true"),
    ("- Check: payout-reaches-the-card\n", "  Context: payouts.instant=true"),
    ("  An approved payout reaches the card at once.\n", "  Context: payouts.instant=true"),
    ("  A payout under review is held until the review ends.\n", "Context: payouts.instant=true"),
];

// behavior-contexts-conditions: A `Context:` under a Check in `verification.md` is a reading error
#[test]
fn a_context_under_a_check_is_a_reading_error() {
    for (after, line) in &CONTEXT_IN_VERIFICATION[2..] {
        misplaced_context(PAYOUTS_VERIFICATION, after, line);
    }
}

// behavior-contexts-conditions: A `Context:` under a Requirement or Scenario in `verification.md` is a reading error
#[test]
fn a_context_under_a_requirement_or_scenario_in_verification_md_is_a_reading_error() {
    for (after, line) in &CONTEXT_IN_VERIFICATION[..2] {
        misplaced_context(PAYOUTS_VERIFICATION, after, line);
    }
    // The same two lines in the sample's own module, which has no keys, stay errors as well.
    misplaced(RETURNS_VERIFICATION, "### Requirement: late-return-owes-a-fine\n", "Context: payouts.instant=true");
    misplaced(RETURNS_VERIFICATION, "#### Scenario: on-time-return-is-free\n", "Context: payouts.instant=true");
}

// behavior-contexts-conditions: A `Context:` above the first Requirement stays prose
#[test]
fn a_context_above_the_first_requirement_stays_prose() {
    let lines = "Context: payouts.instant=true\nContext: payouts.speed=true\nContext: payouts.instant=true payouts.region=\"no\"\nContext:\n\n";
    let repo = started_payouts_with_results();
    repo.replace(PAYOUTS_SPEC, REQUIREMENT_HEADING, &format!("{lines}{REQUIREMENT_HEADING}"));
    repo.replace(PAYOUTS_VERIFICATION, REQUIREMENT_HEADING, &format!("{lines}{REQUIREMENT_HEADING}"));
    lint_is_clean(&repo);
    status(&repo);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[]);
    claim_is(&repo, WAITS_FOR_REVIEW, WAITS_CLAIM, &[]);
}

// behavior-contexts-conditions: A `Context:` line states one key and one exact value as `Context: <key>=<value>`
#[test]
fn a_context_line_states_one_key_and_one_exact_value() {
    let repo = started_payouts_with_results();
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    lint_is_clean(&repo);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.instant", "true")]);
    claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.instant", "false")]);
    claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[]);
    let region = started_payouts_with_results();
    on_requirement(&region, &["Context: payouts.region=\"se\""]);
    claim_is(&region, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.region", "\"se\"")]);
    claim_is_not(&region, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.region", "\"no\"")]);
}

// behavior-contexts-conditions: Spaces and tabs around `=` change nothing
#[test]
fn spaces_and_tabs_around_the_equals_sign_change_nothing() {
    for line in [
        "Context: payouts.instant=true",
        "Context: payouts.instant =true",
        "Context: payouts.instant= true",
        "Context: payouts.instant = true",
        "Context: payouts.instant\t=\ttrue",
        "Context: payouts.instant \t = \t true",
    ] {
        let repo = started_payouts_with_results();
        on_requirement(&repo, &[line]);
        lint_is_clean(&repo);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.instant", "true")]);
        claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.instant", "false")]);
    }
    let region = started_payouts_with_results();
    on_requirement(&region, &["Context: payouts.region\t= \"se\""]);
    lint_is_clean(&region);
    claim_is(&region, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.region", "\"se\"")]);
}

// behavior-contexts-conditions: Several keys are read from several lines under the same heading
#[test]
fn several_keys_are_read_from_several_lines() {
    let repo = started_payouts_with_results();
    on_requirement(&repo, &["Context: payouts.instant=true", "Context: payouts.region=\"se\"", "Context: payouts.limit=100"]);
    lint_is_clean(&repo);
    let condition = [("payouts.instant", "true"), ("payouts.region", "\"se\""), ("payouts.limit", "100")];
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &condition);
    claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &condition[..2]);
    claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &condition[1..]);
    let scenario = started_payouts_with_results();
    on_reaches(&scenario, &["Context: payouts.instant=true", "Context: payouts.region=\"se\""]);
    lint_is_clean(&scenario);
    claim_is(&scenario, REACHES_THE_CARD, REACHES_CLAIM, &condition[..2]);
}

// behavior-contexts-conditions: Spaces and `=` inside a one-of value's JSON string are part of its value
#[test]
fn spaces_and_equals_signs_inside_a_one_of_value_belong_to_it() {
    for label in ["a = b", "x=y", "  two  spaces"] {
        let repo = started_payouts_with_results();
        let value = json_string(label);
        on_requirement(&repo, &[&format!("Context: payouts.label = {value}")]);
        lint_is_clean(&repo);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.label", &value)]);
        claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.label", &json_string("a"))]);
    }
}

// behavior-contexts-conditions: Boolean values are unquoted
#[test]
fn boolean_values_are_unquoted() {
    let repo = started_payouts_with_results();
    on_requirement(&repo, &["Context: payouts.instant=false"]);
    lint_is_clean(&repo);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.instant", "false")]);
    let quoted = started_payouts();
    on_requirement(&quoted, &["Context: payouts.instant=\"false\""]);
    status(&quoted);
    lint_names_the_specification(&quoted);
}

// behavior-contexts-conditions: Number values are unquoted
#[test]
fn number_values_are_unquoted() {
    let repo = started_payouts_with_results();
    on_requirement(&repo, &["Context: payouts.limit=100"]);
    lint_is_clean(&repo);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.limit", "100")]);
    let quoted = started_payouts();
    on_requirement(&quoted, &["Context: payouts.limit=\"100\""]);
    status(&quoted);
    lint_names_the_specification(&quoted);
}

/// A line that cannot be read as a condition stops every command at its place in the file,
/// where the same line with a readable shape does not.
fn condition_reading_error(line: &str) -> serde_json::Value {
    let repo = payouts();
    on_requirement(&repo, &[line]);
    let number = repo.line_of(PAYOUTS_SPEC, "Context:");
    let document = repo.expect_reading_error(Some(PAYOUTS_SPEC), Some(number));
    assert_eq!(document["errors"].as_array().unwrap().len(), 1, "{line:?}: {document:#}");
    document
}

fn condition_is_readable(line: &str) {
    let repo = payouts();
    on_requirement(&repo, &[line]);
    status(&repo);
}

// behavior-contexts-conditions: A `Context:` without a `key=value` assignment is a reading error
#[test]
fn a_context_without_an_assignment_is_a_reading_error() {
    for line in ["Context:", "Context: payouts.instant", "Context: true", "Context:    "] {
        condition_reading_error(line);
    }
    condition_is_readable("Context: payouts.instant=true");
}

// behavior-contexts-conditions: A `Context:` with more than one assignment is a reading error
#[test]
fn a_context_with_two_assignments_is_a_reading_error() {
    for line in [
        "Context: payouts.instant=true payouts.region=\"no\"",
        "Context: payouts.instant=true, payouts.limit=1",
        "Context: payouts.instant=true payouts.instant=false",
    ] {
        condition_reading_error(line);
    }
    condition_is_readable("Context: payouts.instant=true");
}

// behavior-contexts-conditions: A `Context:` with an invalid key spelling is a reading error
#[test]
fn a_context_with_an_invalid_key_is_a_reading_error() {
    for key in [
        "Payouts.instant",
        "payouts.Instant",
        "payouts.2instant",
        "payouts.-instant",
        "payouts.instant_mode",
        "payouts..instant",
        ".instant",
        "payouts.instant.",
        "payouts.\u{e9}",
        "payouts.a\u{ff12}",
    ] {
        condition_reading_error(&format!("Context: {key}=true"));
    }
    // A key spelled well but declared nowhere is read, and lint finds it.
    let repo = started_payouts();
    on_requirement(&repo, &["Context: payouts.instant-mode=true"]);
    status(&repo);
    lint_names_the_specification(&repo);
}

// behavior-contexts-conditions: A `Context:` with an unfinished JSON string is a reading error
#[test]
fn a_context_with_an_unfinished_json_string_is_a_reading_error() {
    for line in ["Context: payouts.region=\"no", "Context: payouts.region=\"", "Context: payouts.region=\"no\\\"", "Context: payouts.region=\"n\\"] {
        condition_reading_error(line);
    }
    condition_is_readable("Context: payouts.region=\"no\"");
}

// behavior-contexts-conditions: A `Context:` with a malformed number is a reading error
#[test]
fn a_context_with_a_malformed_number_is_a_reading_error() {
    for number in ["1.", ".5", "+5", "1e", "1.2.3", "--1", "1e+", "-"] {
        condition_reading_error(&format!("Context: payouts.limit={number}"));
    }
    condition_is_readable("Context: payouts.limit=1.5");
}

// behavior-contexts-conditions: A `Context:` number that would need more than 1000 added zeros without an exponent is a reading error
#[test]
fn a_context_number_with_more_than_a_thousand_added_zeros_is_a_reading_error() {
    condition_is_readable("Context: payouts.limit=1e1000");
    condition_reading_error("Context: payouts.limit=1e1001");
    condition_is_readable("Context: payouts.limit=1e-1001");
    condition_reading_error("Context: payouts.limit=1e-1002");
}

fn with_condition_reading_error() -> Repo {
    let repo = payouts();
    on_requirement(&repo, &["Context: payouts.instant=true payouts.region=\"no\""]);
    repo
}

fn commands_that_read() -> Vec<&'static [&'static str]> {
    let mut commands = COMMANDS_THAT_READ.to_vec();
    commands.push(&["run", "--lane", "commit", "--json"]);
    commands.push(&["check", "run", REACHES_THE_CARD, "--json"]);
    commands.push(&["check", "show", WAITS_FOR_REVIEW, "--json"]);
    commands.push(&["requirement", "show", SENT_AT_ONCE, "--json"]);
    commands
}

// behavior-contexts-conditions: A condition reading error has the same words in every command
#[test]
fn a_condition_reading_error_has_the_same_words_in_every_command() {
    let repo = with_condition_reading_error();
    let number = repo.line_of(PAYOUTS_SPEC, "Context:");
    let expected = repo.expect_reading_error(Some(PAYOUTS_SPEC), Some(number))["errors"].clone();
    for args in commands_that_read() {
        let output = repo.azimuth(args);
        output.expect_status(2);
        assert_eq!(output.json()["errors"], expected, "{args:?}");
        let plain: Vec<_> = args.iter().copied().filter(|arg| *arg != "--json").collect();
        let output = repo.azimuth(&plain);
        output.expect_status(2);
        for error in expected.as_array().unwrap() {
            let message = error["message"].as_str().unwrap();
            assert!(output.text().contains(message), "{plain:?} omits {message:?}: {}", output.text());
        }
    }
}

// behavior-contexts-conditions: On a condition reading error, `azimuth run`, `azimuth check run` and the reading commands do nothing else
#[test]
fn a_condition_reading_error_stops_commands_before_any_other_work() {
    let repo = with_condition_reading_error();
    repo.need_facility("present");
    let before = repo.snapshot();
    for args in commands_that_read() {
        let output = repo.azimuth(args);
        assert_ne!(output.status, 0, "{args:?} succeeded");
        let document = output.json();
        assert_eq!(document["format"], "error-1", "{args:?}: {document:#}");
        for field in ["requirements", "checks", "suites", "class", "findings"] {
            assert!(document.get(field).is_none(), "{args:?} answered despite the error: {document:#}");
        }
        assert_eq!(repo.snapshot(), before, "{args:?} changed the worktree");
        assert!(repo.adapter_starts().is_empty(), "{args:?} started an adapter");
        assert!(repo.probe_starts().is_empty(), "{args:?} proved a facility");
        assert!(repo.store_files().is_empty(), "{args:?} wrote the store");
        assert!(repo.outbox_files().is_empty(), "{args:?} wrote the outbox");
    }
}

// behavior-contexts-conditions: On a condition reading error, commands end with status 2
#[test]
fn a_condition_reading_error_ends_commands_with_status_two() {
    let repo = with_condition_reading_error();
    for args in commands_that_read() {
        repo.azimuth(args).expect_status(2);
    }
}

// behavior-contexts-conditions: A misplaced `Context:` message names the file
#[test]
fn a_misplaced_context_message_names_the_file() {
    for (after, line) in CONTEXT_IN_VERIFICATION {
        let error = misplaced_context(PAYOUTS_VERIFICATION, after, line);
        let message = error["message"].as_str().unwrap_or_default();
        assert!(message.contains("specification.md"), "{line:?} after {after:?}: the message does not name the file the line belongs in: {message}");
    }
}

// behavior-contexts-conditions: A misplaced `Context:` message names the place where the line belongs
#[test]
fn a_misplaced_context_message_names_the_place() {
    // The wording is free. A `Context:` belongs under a Requirement or a Scenario, so the
    // message names both words.
    for (after, line) in CONTEXT_IN_VERIFICATION {
        let error = misplaced_context(PAYOUTS_VERIFICATION, after, line);
        let message = error["message"].as_str().unwrap_or_default();
        for place in ["Requirement", "Scenario"] {
            assert!(message.contains(place), "{line:?} after {after:?}: the message does not name the {place}: {message}");
        }
    }
}

// behavior-contexts-conditions: A condition finding does not stop the reading commands
#[test]
fn reading_commands_still_answer_with_a_condition_finding() {
    for lines in [
        vec!["Context: payouts.speed=true"],
        vec!["Context: payouts.instant=\"true\""],
        vec!["Context: payouts.instant=true", "Context: payouts.instant=false"],
    ] {
        let repo = started_payouts();
        on_requirement(&repo, &lines);
        lint_names_the_specification(&repo);
        for (args, format) in [
            (vec!["status", "--json"], "status-1"),
            (vec!["requirement", "show", SENT_AT_ONCE, "--json"], "requirement-1"),
            (vec!["check", "show", REACHES_THE_CARD, "--json"], "check-1"),
            (vec!["change", "diff", "--json"], "change-1"),
        ] {
            let output = repo.azimuth(&args);
            output.expect_status(0);
            let document = output.json();
            assert_eq!(document["format"], format, "{lines:?}: {document:#}");
            if format == "status-1" {
                let checks = status_checks(&document);
                for check in PAYOUT_CHECKS {
                    assert_eq!(checks[check]["state"], "owed", "{lines:?}: {document:#}");
                }
            } else if format == "check-1" {
                assert_eq!(document["state"], "owed", "{lines:?}: {document:#}");
            }
        }
        assert!(repo.adapter_starts().is_empty());
        assert!(repo.probe_starts().is_empty());
    }
}

// Started components in the model: `docs/increments/contexts-components.md`. A component is
// observed through what refers to it by name: lint finds a key's `components` and a suite's
// `starts` that name no declared component, and a reading error stops every command.

fn lint_finds_an_undeclared_component(repo: &Repo) {
    lint_names_a_declaration_file(repo);
}

fn payouts_with_keys() -> Repo {
    let repo = Repo::sample();
    repo.write(KEY_FILE, INSTANT);
    repo
}

// behavior-contexts-components: The reader reads each `[[component]]` table of the project file
#[test]
fn each_component_table_is_read() {
    let repo = payouts_with_keys();
    declare_payout_components(&repo);
    lint_is_clean(&repo);
    status(&repo);
    for name in [API, WEBSITE] {
        let without = payouts_with_keys();
        declare_payout_components(&without);
        without.replace(PROJECT_FILE, &component_table(name, &["all"], "/internal/azimuth"), "");
        status(&without);
        lint_finds_an_undeclared_component(&without);
    }
}

// behavior-contexts-components: A component has a `name`, unique in the project
#[test]
fn a_component_is_referred_to_by_its_exact_name() {
    let repo = payouts_with_keys();
    declare_payout_components(&repo);
    lint_is_clean(&repo);
    repo.replace(PROJECT_FILE, "name = \"api\"", "name = \"API\"");
    status(&repo);
    lint_finds_an_undeclared_component(&repo);
    let renamed = payouts_with_keys();
    declare_payout_components(&renamed);
    renamed.replace(PROJECT_FILE, "name = \"api\"", "name = \"api-server\"");
    status(&renamed);
    lint_finds_an_undeclared_component(&renamed);
}

// behavior-contexts-components: A component's `areas` names the areas it is built from
#[test]
fn a_components_areas_name_declared_areas() {
    let repo = payouts_with_keys();
    split_areas(&repo);
    repo.append(PROJECT_FILE, &component_table(API, &["api"], "/internal/azimuth"));
    repo.append(PROJECT_FILE, &component_table(WEBSITE, &["website"], "/internal/azimuth"));
    status(&repo);
    lint_is_clean(&repo);
    repo.replace(PROJECT_FILE, "areas = [\"website\"]", "areas = [\"web\"]");
    repo.expect_reading_error(Some(PROJECT_FILE), None);
}

// behavior-contexts-components: A component is made from one area
#[test]
fn a_component_is_made_from_one_area() {
    let repo = payouts_with_keys();
    declare_payout_components(&repo);
    status(&repo);
    lint_is_clean(&repo);
}

// behavior-contexts-components: A component is made from several areas
#[test]
fn a_component_is_made_from_several_areas() {
    let repo = Repo::sample();
    split_areas(&repo);
    repo.write(KEY_FILE, "[payouts.instant]\ntype = \"boolean\"\ncomponents = [\"shop\"]\n");
    repo.append(PROJECT_FILE, &component_table("shop", &["api", "website"], "/internal/azimuth"));
    status(&repo);
    lint_is_clean(&repo);
}

// behavior-contexts-components: A component's `report` is the absolute path where its report is exposed
#[test]
fn a_report_is_an_absolute_path() {
    for report in ["/internal/azimuth", "/", "/internal/azimuth/report", "/a-b/c_d"] {
        let repo = payouts_with_keys();
        repo.append(PROJECT_FILE, &component_table(API, &["all"], report));
        repo.append(PROJECT_FILE, &component_table(WEBSITE, &["all"], report));
        status(&repo);
        lint_is_clean(&repo);
    }
}

// behavior-contexts-components: A project file with no component declares none
#[test]
fn a_project_with_no_component_declares_none() {
    let repo = Repo::sample();
    assert!(!parse_toml(&repo.read(PROJECT_FILE)).contains_key("component"));
    lint_is_clean(&repo);
    repo.write(KEY_FILE, INSTANT);
    status(&repo);
    lint_finds_an_undeclared_component(&repo);
}

/// A key's component and a started component that the project declares, with a Check under the
/// condition: lint is clean exactly while the suite starts a component the key is listed for.
fn condition_on_instant(starts: &[&str]) -> Repo {
    let repo = payouts();
    repo.write(PAYOUT_KEY_FILE, "[payouts.instant]\ntype = \"boolean\"\ncomponents = [\"website\"]\n");
    declare_payout_components(&repo);
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    start_components(&repo, starts);
    repo
}

// behavior-contexts-components: A suite's `starts` lists the names of the components it starts
#[test]
fn starts_lists_component_names() {
    lint_is_clean(&condition_on_instant(&[WEBSITE]));
    lint_is_clean(&condition_on_instant(&[API, WEBSITE]));
    let other = condition_on_instant(&[API]);
    status(&other);
    lint_names_checks(&other, &PAYOUT_CHECKS);
}

/// A suite in the `api` area, a component of the `website` area and one of the suite's own area.
fn two_area_payouts(key_components: &str, starts: &[&str]) -> Repo {
    let repo = payouts();
    split_areas(&repo);
    repo.write(PAYOUT_KEY_FILE, &format!("[payouts.instant]\ntype = \"boolean\"\ncomponents = [{key_components}]\n"));
    repo.append(PROJECT_FILE, &component_table(WEBSITE, &["website"], "/internal/azimuth"));
    repo.append(PROJECT_FILE, &component_table("desk-host", &["api"], "/internal/azimuth"));
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    start_components(&repo, starts);
    repo
}

// behavior-contexts-components: A component in its own process counts as started
#[test]
fn a_component_in_its_own_process_counts_as_started() {
    // `website` is built from an area that does not hold the suite.
    lint_is_clean(&two_area_payouts("\"website\"", &[WEBSITE]));
    lint_names_checks(&two_area_payouts("\"website\"", &[]), &PAYOUT_CHECKS);
}

// behavior-contexts-components: A host built inside the test process counts as started
#[test]
fn a_host_built_inside_the_test_process_counts_as_started() {
    // `desk-host` is built from the area that holds the suite itself.
    lint_is_clean(&two_area_payouts("\"desk-host\"", &["desk-host"]));
    lint_names_checks(&two_area_payouts("\"desk-host\"", &[]), &PAYOUT_CHECKS);
}

// behavior-contexts-components: A suite with no `starts` starts no component
#[test]
fn a_suite_with_no_starts_starts_no_component() {
    let repo = condition_on_instant(&[API, WEBSITE]);
    lint_is_clean(&repo);
    omit_starts(&repo);
    status(&repo);
    lint_names_checks(&repo, &PAYOUT_CHECKS);
}

// behavior-contexts-components: A suite with `starts = []` starts no component
#[test]
fn a_suite_with_empty_starts_starts_no_component() {
    let repo = condition_on_instant(&[]);
    assert!(repo.read(PROJECT_FILE).contains("starts = []"));
    status(&repo);
    lint_names_checks(&repo, &PAYOUT_CHECKS);
}

fn broken_project(from: &str, to: &str) -> Repo {
    let repo = Repo::sample();
    declare_payout_components(&repo);
    start_components(&repo, &[API, WEBSITE]);
    status(&repo);
    let project = repo.read(PROJECT_FILE);
    assert!(project.contains(from), "{from:?} is not in the project file:\n{project}");
    repo.write(PROJECT_FILE, &project.replacen(from, to, 1));
    repo
}

fn project_reading_error(from: &str, to: &str) {
    broken_project(from, to).expect_reading_error(Some(PROJECT_FILE), None);
}

const NAME: &str = "name = \"api\"";
const AREAS: &str = "areas = [\"all\"]";
const REPORT: &str = "report = \"/internal/azimuth\"";
const STARTS: &str = "starts = [\"api\", \"website\"]";

// behavior-contexts-components: A component with no `name` is a reading error
#[test]
fn a_component_with_no_name_is_a_reading_error() {
    project_reading_error(&format!("{NAME}\n"), "");
}

// behavior-contexts-components: A component with no `areas` is a reading error
#[test]
fn a_component_with_no_areas_is_a_reading_error() {
    project_reading_error(&format!("{AREAS}\n"), "");
}

// behavior-contexts-components: A component with no `report` is a reading error
#[test]
fn a_component_with_no_report_is_a_reading_error() {
    project_reading_error(&format!("{REPORT}\n"), "");
}

// behavior-contexts-components: A component whose `areas` names an undeclared area is a reading error
#[test]
fn a_component_built_from_an_undeclared_area_is_a_reading_error() {
    for areas in ["areas = [\"nowhere\"]", "areas = [\"all\", \"nowhere\"]", "areas = [\"All\"]"] {
        project_reading_error(AREAS, areas);
    }
}

// behavior-contexts-components: A component with an empty `areas` is a reading error
#[test]
fn a_component_with_empty_areas_is_a_reading_error() {
    let repo = broken_project(AREAS, "areas = []");
    let line = repo.line_of(PROJECT_FILE, "areas = []");
    let expected = repo.expect_reading_error(Some(PROJECT_FILE), Some(line))["errors"].clone();
    let mut commands = COMMANDS_THAT_READ.to_vec();
    commands.push(&["run", "--lane", "commit", "--json"]);
    for args in &commands {
        let output = repo.azimuth(args);
        output.expect_status(2);
        assert_eq!(output.json()["errors"], expected, "{args:?}");
    }
}

// behavior-contexts-components: A `report` path that does not start with `/` is a reading error
#[test]
fn a_report_path_without_a_leading_slash_is_a_reading_error() {
    for report in ["internal/azimuth", "", "./azimuth", " /azimuth", "http://localhost/azimuth"] {
        project_reading_error(REPORT, &format!("report = \"{report}\""));
    }
}

// behavior-contexts-components: A component field of the wrong TOML type is a reading error
#[test]
fn a_component_field_of_the_wrong_type_is_a_reading_error() {
    for (from, to) in [
        (NAME, "name = 5"),
        (NAME, "name = [\"api\"]"),
        (NAME, "name = true"),
        (AREAS, "areas = \"all\""),
        (AREAS, "areas = [1]"),
        (AREAS, "areas = [\"all\", false]"),
        (AREAS, "areas = { all = true }"),
        (REPORT, "report = 5"),
        (REPORT, "report = [\"/internal/azimuth\"]"),
        (REPORT, "report = true"),
    ] {
        project_reading_error(from, to);
    }
}

// behavior-contexts-components: A `starts` that is not a list of names is a reading error
#[test]
fn starts_that_is_not_a_list_of_names_is_a_reading_error() {
    for starts in [
        "starts = \"api\"",
        "starts = [1]",
        "starts = [\"api\", 2]",
        "starts = { api = true }",
        "starts = true",
        "starts = [[\"api\"]]",
    ] {
        project_reading_error(STARTS, starts);
    }
}

// behavior-contexts-components: Two components with the same name are a reading error
#[test]
fn two_components_with_one_name_are_a_reading_error() {
    project_reading_error("name = \"website\"", "name = \"api\"");
}

// behavior-contexts-components: A field of a component table that the CLI does not know is a reading error
#[test]
fn an_unknown_component_field_is_a_reading_error() {
    project_reading_error(REPORT, &format!("{REPORT}\nshape = \"round\""));
}

// behavior-contexts-components: A component reading error names the line of the key in the project file
#[test]
fn a_component_reading_error_names_the_line_of_the_key() {
    for (from, to, written) in [
        (NAME, "name = 5", "name = 5"),
        (AREAS, "areas = [\"nowhere\"]", "areas = [\"nowhere\"]"),
        (REPORT, "report = 5", "report = 5"),
        (REPORT, "report = \"internal/azimuth\"", "report = \"internal/azimuth\""),
        (REPORT, &format!("{REPORT}\nshape = \"round\""), "shape = \"round\""),
        (STARTS, "starts = \"api\"", "starts = \"api\""),
    ] {
        let repo = broken_project(from, to);
        let line = repo.line_of(PROJECT_FILE, written);
        repo.expect_reading_error(Some(PROJECT_FILE), Some(line));
    }
}

fn component_reading_errors() -> Vec<Repo> {
    vec![broken_project(REPORT, "report = 5"), broken_project(STARTS, "starts = \"api\"")]
}

// behavior-contexts-components: A reading error of components or `starts` has the same words in every command
#[test]
fn component_reading_errors_have_the_same_words_in_each_command() {
    let mut commands = COMMANDS_THAT_READ.to_vec();
    commands.push(&["run", "--lane", "commit", "--json"]);
    for repo in component_reading_errors() {
        let expected = repo.expect_reading_error(Some(PROJECT_FILE), None)["errors"].clone();
        for args in &commands {
            let output = repo.azimuth(args);
            output.expect_status(2);
            assert_eq!(output.json()["errors"], expected, "{args:?}");
            let plain: Vec<_> = args.iter().copied().filter(|arg| *arg != "--json").collect();
            let output = repo.azimuth(&plain);
            output.expect_status(2);
            for error in expected.as_array().unwrap() {
                let message = error["message"].as_str().unwrap();
                assert!(output.text().contains(message), "{plain:?} omits {message:?}: {}", output.text());
            }
        }
    }
}

// behavior-contexts-components: On such a reading error, `azimuth run`, `azimuth check run` and the reading commands do nothing else
#[test]
fn component_reading_errors_stop_commands_before_any_other_work() {
    for repo in component_reading_errors() {
        repo.need_facility("present");
        let before = repo.snapshot();
        for args in COMMANDS_THAT_READ {
            let output = repo.azimuth(args);
            assert_ne!(output.status, 0, "{args:?} succeeded");
            let document = output.json();
            assert_eq!(document["format"], "error-1", "{args:?}: {document:#}");
            for field in ["requirements", "checks", "suites", "class", "findings"] {
                assert!(document.get(field).is_none(), "{args:?} answered despite the error: {document:#}");
            }
            assert_eq!(repo.snapshot(), before, "{args:?} changed the worktree");
            assert!(repo.adapter_starts().is_empty(), "{args:?} started an adapter");
            assert!(repo.probe_starts().is_empty(), "{args:?} proved a facility");
            assert!(repo.store_files().is_empty(), "{args:?} wrote the store");
            assert!(repo.outbox_files().is_empty(), "{args:?} wrote the outbox");
        }
    }
}

// behavior-contexts-components: On such a reading error, commands end with status 2
#[test]
fn component_reading_errors_end_with_status_two() {
    let mut commands = COMMANDS_THAT_READ.to_vec();
    commands.push(&["run", "--lane", "commit", "--json"]);
    for repo in component_reading_errors() {
        for args in &commands {
            repo.azimuth(args).expect_status(2);
        }
    }
}
