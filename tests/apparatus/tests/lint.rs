//! Group "`azimuth lint`" of docs/increments/1.md.

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
#[path = "support/contexts_keys.rs"]
mod contexts_keys;
use contexts_conditions::*;
use contexts_keys::*;

fn unknown_marker(repo: &Repo) -> u64 {
    repo.replace(
        LOAN_LIMIT_TESTS,
        "# Check: returned-book-frees-a-slot\n",
        "# Check: returned-book-frees-a-slot\n# Check: renewal-counts-as-one-loan\n",
    );
    repo.line_of(LOAN_LIMIT_TESTS, "renewal-counts-as-one-loan")
}

// behavior: It reports reading errors and findings, and ends with failure on either
#[test]
fn lint_fails_on_a_finding() {
    let repo = Repo::sample();
    unknown_marker(&repo);
    let findings = repo.lint_findings(1);
    assert!(!findings.is_empty());
}

// behavior: It reports reading errors and findings, and ends with failure on either
#[test]
fn lint_fails_on_a_reading_error() {
    let repo = Repo::sample();
    repo.replace(LENDING_SPEC, "### Requirement: loan-limit-holds", "### Requirement: loan_limit_holds");
    let output = repo.azimuth(&["lint", "--json"]);
    output.expect_status(2);
    let document = output.json();
    assert_eq!(document["format"], "error-1", "{document:#}");
    assert_eq!(document["errors"][0]["line"].as_u64(), Some(repo.line_of(LENDING_SPEC, "loan_limit_holds")));
}

// behavior: With no finding it ends with status 0, with a finding with 1, and on a reading error with 2
#[test]
fn lint_ends_with_one_of_three_statuses() {
    let clean = Repo::sample();
    assert!(clean.lint_findings(0).is_empty());

    let finding = Repo::sample();
    unknown_marker(&finding);
    finding.lint_findings(1);

    let broken = Repo::sample();
    broken.replace(LENDING_SPEC, "### Requirement: loan-limit-holds", "### Requirement:");
    broken.azimuth(&["lint"]).expect_status(2);
}

// behavior: Its document is `lint-1` with these fields
#[test]
fn lint_document_has_its_fields() {
    let repo = Repo::sample();
    let line = unknown_marker(&repo);
    let findings = repo.lint_findings(1);
    let at_marker = findings_at(&findings, LOAN_LIMIT_TESTS, Some(line));
    assert_eq!(at_marker.len(), 1, "{findings:#?}");
    let finding = at_marker[0];
    assert!(finding["kind"].as_str().is_some_and(|kind| !kind.is_empty()), "{finding:#}");
    assert!(finding["message"].as_str().is_some_and(|message| !message.is_empty()), "{finding:#}");
}

// behavior: It runs no test and proves no facility
#[test]
fn lint_runs_no_test_and_no_proof() {
    let repo = Repo::sample();
    repo.need_facility("present");
    unknown_marker(&repo);
    repo.lint_findings(1);
    assert!(repo.adapter_starts().is_empty(), "lint started the adapter");
    assert!(repo.probe_starts().is_empty(), "lint proved a facility");
}

// behavior: It works with no adapter installed and without building anything
#[test]
fn lint_needs_no_adapter() {
    let mut repo = Repo::sample();
    repo.replace(PROJECT_FILE, "adapter = \"text-suite-adapter\"", "adapter = \"desk-adapter-that-is-not-installed\"");
    repo.set_env("PATH", &repo.path_without("dotnet"));
    assert!(repo.lint_findings(0).is_empty());
    let before = repo.snapshot();
    repo.lint_findings(0);
    assert_eq!(repo.snapshot(), before, "lint wrote files into the tree");
}

// behavior: It finds markers by scanning each suite's files for that suite's marker form
#[test]
fn lint_reads_markers_by_the_suite_marker_form() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "marker = \"# Check: <id>\"", "marker = \"// Check: <id>\"");
    let findings = repo.lint_findings(1);
    for (file, check) in [
        (LENDING_VERIFICATION, SIXTH_LOAN),
        (LENDING_VERIFICATION, RETURNED_BOOK),
        (RETURNS_VERIFICATION, LATE_RETURN),
        (RETURNS_VERIFICATION, ON_TIME_RETURN),
    ] {
        let line = repo.line_of(file, &format!("- Check: {check}"));
        assert_eq!(findings_at(&findings, file, Some(line)).len(), 1, "{check} has no marker in the new form: {findings:#?}");
    }

    for file in [LOAN_LIMIT_TESTS, LATE_FINE_TESTS, FREE_RETURN_TESTS] {
        let text = repo.read(file).replace("# Check: ", "// Check: ");
        repo.write(file, &text);
    }
    assert!(repo.lint_findings(0).is_empty());
}

// behavior: It finds markers by scanning each suite's files for that suite's marker form
#[test]
fn lint_ignores_marker_text_outside_every_suite() {
    let repo = Repo::sample();
    repo.write("desk/notes.txt", "# Check: renewal-counts-as-one-loan\n");
    assert!(repo.lint_findings(0).is_empty(), "a marker outside every suite folder was read");
}

// behavior: A project may declare a marker key other than `Check`
#[test]
fn a_project_may_use_another_marker_key() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "marker = \"# Check: <id>\"", "marker = \"# Proof: <id>\"\nmarker_key = \"Proof\"");
    for file in [LOAN_LIMIT_TESTS, LATE_FINE_TESTS, FREE_RETURN_TESTS] {
        let text = repo.read(file).replace("# Check: ", "# Proof: ");
        repo.write(file, &text);
    }
    assert!(repo.lint_findings(0).is_empty());
    repo.replace(LATE_FINE_TESTS, "# Proof: late-return-is-fined", "# Check: late-return-is-fined");
    let findings = repo.lint_findings(1);
    let line = repo.line_of(RETURNS_VERIFICATION, "- Check: late-return-is-fined");
    assert_eq!(findings_at(&findings, RETURNS_VERIFICATION, Some(line)).len(), 1, "{findings:#?}");
}

fn expect_near_miss(file: &str, after: &str, line: &str) {
    let repo = Repo::sample();
    repo.replace(file, after, &format!("{after}{line}\n"));
    let number = repo.line_of(file, line);
    let findings = repo.lint_findings(1);
    assert_eq!(findings_at(&findings, file, Some(number)).len(), 1, "{line:?} gave no finding: {findings:#?}");
}

// behavior: It reports a line whose first word is one or two letters away from a declaration word
#[test]
fn lint_reports_a_near_miss_of_verification() {
    expect_near_miss(LENDING_SPEC, "A borrower holds at most five loans at a time.\n", "Verificaton: reviewed");
}

// behavior: It reports a line whose first word is one or two letters away from a declaration word
#[test]
fn lint_reports_a_near_miss_of_realization_two_letters_away() {
    expect_near_miss(LENDING_SPEC, "A borrower holds at most five loans at a time.\n", "Realisaton: reviewed");
}

// behavior: It reports a line whose first word is one or two letters away from a declaration word
#[test]
fn lint_reports_a_near_miss_of_context() {
    expect_near_miss(RETURNS_SPEC, "The desk records no fine.\n", "Contex: the desk is open");
}

// behavior: It reports a line whose first word is one or two letters away from a declaration word
#[test]
fn lint_reports_a_near_miss_of_reuse() {
    expect_near_miss(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  Reuze: never, the desk reads today's date",
    );
}

// behavior: It reports a line whose first word is one or two letters away from a declaration word
#[test]
fn lint_does_not_report_a_word_far_from_every_declaration_word() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.\n",
        "A borrower holds at most five loans at a time.\nVerify: the count after each loan.\n",
    );
    assert!(repo.lint_findings(0).is_empty());
}

// behavior: It reports a marker that names an unknown Check
#[test]
fn lint_reports_a_marker_of_an_unknown_check() {
    let repo = Repo::sample();
    let line = unknown_marker(&repo);
    let findings = repo.lint_findings(1);
    let found = findings_at(&findings, LOAN_LIMIT_TESTS, Some(line));
    assert_eq!(found.len(), 1, "{findings:#?}");
    assert!(found[0]["message"].as_str().unwrap().contains("renewal-counts-as-one-loan"));
    assert_eq!(findings.len(), 1, "only the marker disagrees: {findings:#?}");
}

// behavior: It reports a Check that no marker names
#[test]
fn lint_reports_a_check_without_a_marker() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "# Check: late-return-is-fined\n", "");
    let line = repo.line_of(RETURNS_VERIFICATION, "- Check: late-return-is-fined");
    let findings = repo.lint_findings(1);
    let found = findings_at(&findings, RETURNS_VERIFICATION, Some(line));
    assert_eq!(found.len(), 1, "{findings:#?}");
    assert!(found[0]["message"].as_str().unwrap().contains(LATE_RETURN));
    assert_eq!(findings.len(), 1, "{findings:#?}");
}

// behavior: It reports one Check id marked in two suites
#[test]
fn lint_reports_a_check_marked_in_two_suites() {
    let repo = Repo::sample();
    repo.add_text_suite("returns-checks", "");
    repo.write("suites/returns-checks/second-limit.test", "# Check: sixth-loan-is-refused\noutcome: passed\n");
    let findings = repo.lint_findings(1);
    assert!(
        findings.iter().any(|finding| finding["message"].as_str().unwrap_or_default().contains(SIXTH_LOAN)),
        "{findings:#?}"
    );
}

// behavior: It reports a heading in `verification.md` whose id exists nowhere in `specification.md`
#[test]
fn lint_reports_a_verification_heading_with_an_unknown_requirement() {
    let repo = Repo::sample();
    repo.append(
        LENDING_VERIFICATION,
        "\n### Requirement: renewals-are-limited\n#### Scenario: third-renewal-is-refused\n- Check: third-renewal-is-refused\n  The desk refuses a third renewal.\n",
    );
    repo.append(LATE_FINE_TESTS, "# Check: third-renewal-is-refused\n");
    let line = repo.line_of(LENDING_VERIFICATION, "Requirement: renewals-are-limited");
    let findings = repo.lint_findings(1);
    assert_eq!(findings_at(&findings, LENDING_VERIFICATION, Some(line)).len(), 1, "{findings:#?}");
}

// behavior: It reports a heading in `verification.md` whose id exists nowhere in `specification.md`
#[test]
fn lint_reports_a_verification_heading_with_an_unknown_scenario() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_VERIFICATION,
        "#### Scenario: returned-book-frees-a-slot",
        "#### Scenario: returned-book-frees-two-slots",
    );
    let line = repo.line_of(LENDING_VERIFICATION, "returned-book-frees-two-slots");
    let findings = repo.lint_findings(1);
    assert_eq!(findings_at(&findings, LENDING_VERIFICATION, Some(line)).len(), 1, "{findings:#?}");
}

// behavior: Text above the first Requirement in `specification.md` is prose, also under a heading that says `Term`, and gives no finding
#[test]
fn text_above_the_first_requirement_is_prose() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_SPEC,
        "### Requirement: loan-limit-holds\n",
        "## Term: renewal\nA renewal moves the due date of a loan and does not add one.\n\nTerm: overdue\nA loan past its due date.\n\n### Requirement: loan-limit-holds\n",
    );
    assert!(repo.lint_findings(0).is_empty());
    repo.azimuth(&["status", "--json"]).expect_status(0);
}

const UNREADABLE_REALIZATION: &str = "### Requirement: Loan-Limit-Holds\nRealization: kind slice\nVerificaton: reviewed\n\n### Requirement: loan-limit-holds\n### Requirement: loan-limit-holds\n#### Scenario:\n- Root: the shelf module, which does not exist\n";

// behavior: It reads nothing inside `realization.md` and reports nothing about it
#[test]
fn lint_reads_nothing_inside_realization() {
    let repo = Repo::sample();
    repo.write("azimuth/model/lending/realization.md", UNREADABLE_REALIZATION);
    assert!(repo.lint_findings(0).is_empty());
    repo.azimuth(&["status", "--json"]).expect_status(0);
}

// behavior: It resolves no structure unit and reads no profile
#[test]
fn lint_resolves_no_root_and_needs_no_profile() {
    let repo = Repo::sample();
    repo.write(
        "azimuth/model/desk/returns/realization.md",
        "### Requirement: late-return-owes-a-fine\nRoot: slice ReturnBook\nRoot: kind endpoint\nRoot: desk/no-such-file.txt\n",
    );
    assert!(repo.lint_findings(0).is_empty());
}

// behavior: It does not report a Scenario that has no Check
#[test]
fn lint_does_not_report_a_scenario_without_a_check() {
    let repo = Repo::sample();
    repo.append(RETURNS_SPEC, "\n#### Scenario: lost-book-is-charged\nA book reported lost is charged at its price.\n");
    assert!(repo.lint_findings(0).is_empty());
}

// behavior: It does not report an owed Check or a missing result
#[test]
fn lint_does_not_report_owed_checks() {
    let repo = Repo::sample();
    assert!(repo.lint_findings(0).is_empty(), "lint reported Checks that never ran");
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.azimuth(&["run", "--json"]).expect_status(1);
    assert!(repo.lint_findings(0).is_empty(), "lint reported a known fail");
}

// behavior: It reports a tracked file under `.azimuth/`
#[test]
fn lint_reports_a_tracked_file_under_the_ignored_folder() {
    let repo = Repo::sample();
    repo.write(".azimuth/fake-pass.json", "{}\n");
    repo.git(&["add", "-f", ".azimuth/fake-pass.json"]);
    let findings = repo.lint_findings(1);
    let about_the_file: Vec<&serde_json::Value> = findings.iter().filter(|finding| finding["file"] == ".azimuth/fake-pass.json").collect();
    assert_eq!(about_the_file.len(), 1, "{findings:#?}");
    // The finding is about the whole file, so it has no line to name, and the field is absent.
    assert!(about_the_file[0].get("line").is_none(), "{:#}", about_the_file[0]);
}

// behavior: Every finding says what is wrong and what to do
#[test]
fn every_finding_carries_a_message_that_names_its_subject() {
    // Whether a message says what to do is a judgement of words. The test asserts what it can:
    // every finding has a message of more than a few words that names the thing it is about.
    let repo = Repo::sample();
    unknown_marker(&repo);
    repo.replace(LATE_FINE_TESTS, "# Check: late-return-is-fined\n", "");
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.\n",
        "A borrower holds at most five loans at a time.\nVerificaton: reviewed\n",
    );
    let findings = repo.lint_findings(1);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    for (subject, finding) in [
        ("renewal-counts-as-one-loan", findings_at(&findings, LOAN_LIMIT_TESTS, None)),
        (LATE_RETURN, findings_at(&findings, RETURNS_VERIFICATION, None)),
        ("Verificaton", findings_at(&findings, LENDING_SPEC, None)),
    ] {
        assert_eq!(finding.len(), 1, "{findings:#?}");
        let message = finding[0]["message"].as_str().unwrap();
        assert!(message.contains(subject), "the message does not name {subject}: {message}");
        assert!(message.split_whitespace().count() >= 5, "the message is too short to say what to do: {message}");
    }
}

// behavior: It ends with a failure status when there is a finding
#[test]
fn lint_ends_with_status_one_on_a_finding() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "# Check: late-return-is-fined\n", "");
    let output = repo.azimuth(&["lint"]);
    output.expect_status(1);
    assert!(output.text().contains(LATE_RETURN), "{}", output.text());
}

fn key_finding_repo(overlap: bool) -> Repo {
    let repo = Repo::sample();
    declare_payout_components(&repo);
    let declarations = if overlap {
        format!("[payouts]\ntype = \"boolean\"\ncomponents = [\"api\"]\n\n{INSTANT}")
    } else {
        INSTANT.replace("[\"api\", \"website\"]", "[]")
    };
    repo.write(KEY_FILE, &declarations);
    repo
}

fn key_findings_are_reported(findings: &[serde_json::Value]) {
    assert!(
        !findings_at(findings, KEY_FILE, None).is_empty(),
        "no finding names the key file: {findings:#?}"
    );
    // No diagnostic kind, exact message, or exact line is specified for key findings.
    for finding in findings {
        entry_has_kind_file_line_message(finding);
    }
}

// behavior-contexts-keys: Lint reports a key that begins another at a segment boundary
#[test]
fn lint_reports_segment_boundary_prefixes_but_not_plain_text_prefixes() {
    let repo = Repo::sample();
    declare_payout_components(&repo);
    repo.write(KEY_FILE, &format!("{INSTANT}\n[payouts.instantly]\ntype = \"boolean\"\ncomponents = [\"api\"]\n"));
    assert!(repo.lint_findings(0).is_empty());
    repo.append(KEY_FILE, "\n[payouts]\ntype = \"boolean\"\ncomponents = [\"api\"]\n");
    key_findings_are_reported(&repo.lint_findings(1));
}

// behavior-contexts-keys: Lint reports a key whose `components` list is empty
#[test]
fn lint_reports_a_key_listed_for_no_component() {
    let repo = Repo::sample();
    declare_payout_components(&repo);
    assert!(repo.lint_findings(0).is_empty());
    repo.write(KEY_FILE, &INSTANT.replace("[\"api\", \"website\"]", "[]"));
    key_findings_are_reported(&repo.lint_findings(1));
}

// behavior-contexts-keys: A key-declaration finding fails lint
#[test]
fn either_key_declaration_finding_fails_lint_with_status_one() {
    for overlap in [false, true] {
        let repo = key_finding_repo(overlap);
        key_findings_are_reported(&repo.lint_findings(1));
        repo.azimuth(&["lint"]).expect_status(1);
    }
}

// behavior-contexts-keys: A key-declaration finding fails the commit lane
#[test]
fn key_findings_fail_the_commit_lane_even_when_every_check_passes() {
    for overlap in [false, true] {
        let repo = key_finding_repo(overlap);
        let document = repo.run_json(&["--lane", "commit"], 1);
        assert_eq!(document["outcome"], "failure", "{document:#}");
        key_findings_are_reported(document["findings"].as_array().unwrap());
        for check in ALL_CHECKS {
            assert_eq!(state_of(&document, check), "passed", "{document:#}");
            assert!(repo.requested_checks().contains(check), "the lane skipped {check}");
        }
    }
}

// behavior-contexts-keys: A key-declaration finding does not stop the reading commands
#[test]
fn reading_commands_still_answer_with_either_key_finding() {
    for overlap in [false, true] {
        let repo = key_finding_repo(overlap);
        key_findings_are_reported(&repo.lint_findings(1));
        for (args, format) in [
            (vec!["status", "--json"], "status-1"),
            (vec!["requirement", "show", LOAN_LIMIT, "--json"], "requirement-1"),
            (vec!["check", "show", SIXTH_LOAN, "--json"], "check-1"),
            (vec!["change", "diff", "--json"], "change-1"),
        ] {
            let output = repo.azimuth(&args);
            output.expect_status(0);
            let document = output.json();
            assert_eq!(document["format"], format, "{document:#}");
            if format == "status-1" {
                for check in ALL_CHECKS {
                    assert_eq!(status_checks(&document)[check]["state"], "owed", "{document:#}");
                }
            } else if format == "check-1" {
                assert_eq!(document["state"], "owed", "{document:#}");
            }
        }
        assert!(repo.adapter_starts().is_empty());
        assert!(repo.probe_starts().is_empty());
    }
}

// Conditions in the model: `docs/increments/contexts-conditions.md`.

fn with_condition(lines: &[&str]) -> Repo {
    let repo = started_payouts();
    lint_is_clean(&repo);
    on_requirement(&repo, lines);
    repo
}

fn condition_is_a_finding(lines: &[&str]) {
    let repo = with_condition(lines);
    lint_names_the_specification(&repo);
    // Lint does not stop the commands that read: the condition is readable.
    repo.status_json();
}

// behavior-contexts-conditions: Lint reports a `Context:` that names an undeclared key
#[test]
fn lint_reports_a_condition_on_an_undeclared_key() {
    condition_is_a_finding(&["Context: payouts.speed=true"]);
    condition_is_a_finding(&["Context: payouts.instant-mode=true"]);
    let on_scenario = started_payouts();
    on_waits(&on_scenario, &["Context: payouts.speed=true"]);
    lint_names_the_specification(&on_scenario);
    let without_keys = started_payouts();
    without_keys.remove(PAYOUT_KEY_FILE);
    lint_is_clean(&without_keys);
    on_requirement(&without_keys, &["Context: payouts.instant=true"]);
    lint_names_the_specification(&without_keys);
}

// behavior-contexts-conditions: Lint reports a well-formed value of the wrong declared type
#[test]
fn lint_reports_a_value_of_the_wrong_type() {
    for line in [
        "Context: payouts.instant=\"true\"",
        "Context: payouts.instant=1",
        "Context: payouts.instant=\"no\"",
        "Context: payouts.region=true",
        "Context: payouts.region=5",
        "Context: payouts.limit=true",
        "Context: payouts.limit=\"5\"",
    ] {
        condition_is_a_finding(&[line]);
    }
    let on_scenario = started_payouts();
    on_reaches(&on_scenario, &["Context: payouts.instant=\"true\""]);
    lint_names_the_specification(&on_scenario);
}

// behavior-contexts-conditions: Lint reports a one-of value not in the declared list
#[test]
fn lint_reports_a_one_of_value_that_is_not_listed() {
    for value in ["\"xx\"", "\"NO\"", "\"No\"", "\"se \"", "\" se\"", "\"\"", "\"\\u0073\""] {
        condition_is_a_finding(&[&format!("Context: payouts.region={value}")]);
    }
    for value in ["\"no\"", "\"se\"", "\"\u{e9}\"", "\"\u{c9}\"", "\"\\u0073e\""] {
        let repo = with_condition(&[&format!("Context: payouts.region={value}")]);
        lint_is_clean(&repo);
    }
}

// behavior-contexts-conditions: Lint reports contradicting `Context:` lines on one heading
#[test]
fn lint_reports_contradicting_lines_on_one_heading() {
    for lines in [
        ["Context: payouts.instant=true", "Context: payouts.instant=false"],
        ["Context: payouts.region=\"no\"", "Context: payouts.region=\"se\""],
        ["Context: payouts.limit=1", "Context: payouts.limit=2"],
        ["Context: payouts.limit=9007199254740992", "Context: payouts.limit=9007199254740993"],
        ["Context: payouts.region=\"\u{e9}\"", "Context: payouts.region=\"\u{c9}\""],
    ] {
        condition_is_a_finding(&lines);
        let on_scenario = started_payouts();
        on_waits(&on_scenario, &lines);
        lint_names_the_specification(&on_scenario);
    }
    // Lines that normalize to one value contradict nothing.
    for lines in [
        ["Context: payouts.instant=true", "Context: payouts.instant=TRUE"],
        ["Context: payouts.limit=1", "Context: payouts.limit=1.00"],
        ["Context: payouts.limit=100", "Context: payouts.limit=1e2"],
        ["Context: payouts.region=\"\u{e9}\"", "Context: payouts.region=\"e\u{301}\""],
    ] {
        lint_is_clean(&with_condition(&lines));
    }
}

// behavior-contexts-conditions: Lint reports a Scenario that gives an inherited key another value
#[test]
fn lint_reports_a_scenario_that_overrides_an_inherited_key() {
    let repo = started_payouts();
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    lint_is_clean(&repo);
    on_reaches(&repo, &["Context: payouts.instant=false"]);
    lint_names_the_specification(&repo);
    // Another key, and the same value in any spelling, are no override.
    let other_key = started_payouts();
    on_requirement(&other_key, &["Context: payouts.instant=true"]);
    on_reaches(&other_key, &["Context: payouts.region=\"no\""]);
    lint_is_clean(&other_key);
    let same_value = started_payouts();
    on_requirement(&same_value, &["Context: payouts.limit=100"]);
    on_reaches(&same_value, &["Context: payouts.limit=1E2"]);
    lint_is_clean(&same_value);
    // Another value for an inherited one-of key is an override as well.
    let other_value = started_payouts();
    on_requirement(&other_value, &["Context: payouts.region=\"se\""]);
    on_reaches(&other_value, &["Context: payouts.region=\"no\""]);
    lint_names_the_specification(&other_value);
}

// behavior-contexts-conditions: A condition finding fails lint
#[test]
fn a_condition_finding_fails_lint_with_status_one() {
    for lines in [
        ["Context: payouts.speed=true", "Context: payouts.instant=true"],
        ["Context: payouts.instant=true", "Context: payouts.instant=false"],
        ["Context: payouts.instant=\"true\"", "Context: payouts.limit=1"],
    ] {
        let repo = with_condition(&lines);
        lint_names_the_specification(&repo);
        let output = repo.azimuth(&["lint"]);
        output.expect_status(1);
        assert!(output.text().contains(PAYOUTS_SPEC), "the text output does not name the file:\n{}", output.text());
    }
}

// behavior-contexts-conditions: A range is not accepted as a condition value
#[test]
fn a_range_is_not_accepted_as_a_condition_value() {
    // The design does not say whether a range is a reading error or a finding, so either ends
    // lint with a failure, and a value that states one number is clean.
    for value in ["1..5", "1-5", ">=1", ">1", "<5", "[1,5]", "(1, 5)", "1 to 5", "1:5", "\"1..5\"", "\"1-5\""] {
        let repo = with_condition(&[&format!("Context: payouts.limit={value}")]);
        let output = repo.azimuth(&["lint", "--json"]);
        assert!(matches!(output.status, 1 | 2), "the range {value} was accepted:\n{}", output.text());
    }
    for value in ["\"se\"..\"no\"", "\"se\"-\"no\"", "\"no\" \"se\""] {
        let repo = with_condition(&[&format!("Context: payouts.region={value}")]);
        let output = repo.azimuth(&["lint", "--json"]);
        assert!(matches!(output.status, 1 | 2), "the range {value} was accepted:\n{}", output.text());
    }
    lint_is_clean(&with_condition(&["Context: payouts.limit=5"]));
}

// Started components in the model: `docs/increments/contexts-components.md`. The design says
// that lint names a Check, and for the two declaration findings which declarations disagree;
// it gives no kind, message or line, so the tests assert the Check named, or a file of the pair.

fn undeclared_in_a_key() -> Repo {
    let repo = started_payouts();
    repo.append(PAYOUT_KEY_FILE, "\n[payouts.ledger]\ntype = \"boolean\"\ncomponents = [\"ledger\"]\n");
    repo
}

fn undeclared_in_starts() -> Repo {
    let repo = started_payouts();
    start_components(&repo, &[API, WEBSITE, "ledger"]);
    repo
}

/// The two Checks of the payouts Requirement, one of them under a condition that no started
/// component is listed for.
fn check_not_proved() -> Repo {
    let repo = started_payouts();
    on_reaches(&repo, &["Context: payouts.instant=true"]);
    start_components(&repo, &[]);
    repo
}

// behavior-contexts-components: Lint reports an undeclared component in a key's `components`
#[test]
fn lint_reports_an_undeclared_component_in_a_key() {
    let repo = undeclared_in_a_key();
    lint_names_a_declaration_file(&repo);
    let control = started_payouts();
    control.append(PAYOUT_KEY_FILE, "\n[payouts.ledger]\ntype = \"boolean\"\ncomponents = [\"api\"]\n");
    lint_is_clean(&control);
    let sample = Repo::sample();
    sample.write(KEY_FILE, INSTANT);
    lint_names_a_declaration_file(&sample);
}

// behavior-contexts-components: Lint reports an undeclared component in a suite's `starts`
#[test]
fn lint_reports_an_undeclared_component_in_starts() {
    let repo = undeclared_in_starts();
    lint_names_a_declaration_file(&repo);
    lint_is_clean(&started_payouts());
}

fn condition_with_starts(requirement: &[&str], scenario: &[&str], starts: &[&str]) -> Repo {
    let repo = started_payouts();
    on_requirement(&repo, requirement);
    if !scenario.is_empty() {
        on_reaches(&repo, scenario);
    }
    start_components(&repo, starts);
    repo
}

const INSTANT_TRUE: &str = "Context: payouts.instant=true";
const REGION_NO: &str = "Context: payouts.region=\"no\"";

// behavior-contexts-components: Lint reports a Check whose condition states a key that no component its suite starts is listed for
#[test]
fn lint_reports_a_check_whose_condition_no_started_component_serves() {
    // The Requirement's condition is every Scenario's, so both Checks are named.
    lint_names_checks(&condition_with_starts(&[INSTANT_TRUE], &[], &[]), &PAYOUT_CHECKS);
    // A Scenario's own condition names only its Check.
    lint_names_only(&condition_with_starts(&[], &[INSTANT_TRUE], &[]), &[REACHES_THE_CARD], &[WAITS_FOR_REVIEW]);
    // `payouts.region` is listed for `api` alone, and a Scenario adds it to the Requirement's condition.
    lint_names_only(
        &condition_with_starts(&[INSTANT_TRUE], &[REGION_NO], &[WEBSITE]),
        &[REACHES_THE_CARD],
        &[WAITS_FOR_REVIEW],
    );
    // A Check with no condition is promised everywhere, and needs no started component.
    lint_is_clean(&condition_with_starts(&[], &[], &[]));
}

// behavior-contexts-components: Lint reports such a Check when its suite has no `starts`
#[test]
fn lint_reports_a_check_when_its_suite_has_no_starts() {
    let repo = started_payouts();
    on_requirement(&repo, &[INSTANT_TRUE]);
    omit_starts(&repo);
    assert!(!repo.read(PROJECT_FILE).contains("starts"));
    lint_names_checks(&repo, &PAYOUT_CHECKS);
}

// behavior-contexts-components: Lint reports such a Check when its suite starts only components the key is not listed for
#[test]
fn lint_reports_a_check_when_the_suite_starts_only_unlisted_components() {
    // `payouts.region` is listed for `api`, and the suite starts `website`.
    lint_names_checks(&condition_with_starts(&["Context: payouts.region=\"se\""], &[], &[WEBSITE]), &PAYOUT_CHECKS);
    // Starting a component that is listed for another key does not help either.
    lint_names_only(&condition_with_starts(&[], &[REGION_NO], &[WEBSITE]), &[REACHES_THE_CARD], &[WAITS_FOR_REVIEW]);
    lint_is_clean(&condition_with_starts(&["Context: payouts.region=\"se\""], &[], &[API]));
}

// behavior-contexts-components: Lint reports nothing for a Check whose suite starts one component the key is listed for
#[test]
fn lint_reports_nothing_when_the_suite_starts_one_listed_component() {
    // `payouts.instant` is listed for both components.
    for starts in [[API], [WEBSITE]] {
        lint_is_clean(&condition_with_starts(&[INSTANT_TRUE], &[], &starts));
        lint_is_clean(&condition_with_starts(&[], &[INSTANT_TRUE], &starts));
    }
    // The condition a Scenario inherits and adds to is served by `api`, which both keys list.
    lint_is_clean(&condition_with_starts(&[INSTANT_TRUE], &[REGION_NO], &[API]));
}

// behavior-contexts-components: Lint reports nothing for a Check whose suite starts every component the key is listed for
#[test]
fn lint_reports_nothing_when_the_suite_starts_every_listed_component() {
    lint_is_clean(&condition_with_starts(&[INSTANT_TRUE], &[], &[API, WEBSITE]));
    lint_is_clean(&condition_with_starts(&[INSTANT_TRUE], &[REGION_NO], &[API, WEBSITE]));
    // A component the key is not listed for may be started as well.
    let repo = condition_with_starts(&[INSTANT_TRUE], &[], &[API, WEBSITE, "desk-host"]);
    repo.append(PROJECT_FILE, &component_table("desk-host", &["all"], "/internal/azimuth"));
    lint_is_clean(&repo);
}

// behavior-contexts-components: A component that exists but is not started by the suite does not satisfy the finding
#[test]
fn a_declared_component_that_the_suite_does_not_start_does_not_satisfy_lint() {
    // Both components are declared and listed for the key; another suite starts `api`.
    let repo = started_payouts();
    let tests = repo.read(PAYOUTS_TESTS);
    repo.remove(PAYOUTS_TESTS);
    repo.write("suites/payout-checks/payouts.test", &tests);
    repo.add_text_suite("payout-checks", "");
    on_requirement(&repo, &[INSTANT_TRUE]);
    lint_names_checks(&repo, &PAYOUT_CHECKS);
    repo.append(PROJECT_FILE, "starts = [\"website\"]\n");
    lint_is_clean(&repo);
}

fn component_findings() -> Vec<Repo> {
    vec![undeclared_in_a_key(), undeclared_in_starts(), check_not_proved()]
}

// behavior-contexts-components: A component finding fails lint
#[test]
fn a_component_finding_fails_lint_with_status_one() {
    for repo in component_findings() {
        let findings = repo.lint_findings(1);
        assert!(!findings.is_empty());
        repo.azimuth(&["lint"]).expect_status(1);
    }
}

// behavior-contexts-components: A component finding fails the commit lane
#[test]
fn a_component_finding_fails_the_commit_lane() {
    for repo in [undeclared_in_a_key(), undeclared_in_starts()] {
        let document = repo.run_json(&["--lane", "commit"], 1);
        assert_eq!(document["outcome"], "failure", "{document:#}");
        assert!(!document["findings"].as_array().unwrap().is_empty(), "{document:#}");
        for check in all_checks() {
            assert_eq!(state_of(&document, check), "passed", "{document:#}");
        }
    }
    let document = check_not_proved().run_json(&["--lane", "commit"], 1);
    assert_eq!(document["outcome"], "failure", "{document:#}");
    assert!(findings_naming(document["findings"].as_array().unwrap(), REACHES_THE_CARD) > 0, "{document:#}");
}

// behavior-contexts-components: A component finding does not stop the reading commands
#[test]
fn reading_commands_still_answer_with_a_component_finding() {
    for repo in component_findings() {
        repo.lint_findings(1);
        for (args, format) in [
            (vec!["status", "--json"], "status-1"),
            (vec!["requirement", "show", SENT_AT_ONCE, "--json"], "requirement-1"),
            (vec!["check", "show", REACHES_THE_CARD, "--json"], "check-1"),
            (vec!["change", "diff", "--json"], "change-1"),
        ] {
            let output = repo.azimuth(&args);
            output.expect_status(0);
            let document = output.json();
            assert_eq!(document["format"], format, "{document:#}");
            if format == "status-1" {
                for check in all_checks() {
                    assert_eq!(status_checks(&document)[check]["state"], "owed", "{document:#}");
                }
            } else if format == "check-1" {
                assert_eq!(document["state"], "owed", "{document:#}");
            }
        }
        assert!(repo.adapter_starts().is_empty());
        assert!(repo.probe_starts().is_empty());
    }
}
