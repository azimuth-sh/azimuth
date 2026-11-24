//! Group "`azimuth lint`" of docs/increments/1.md.

use azimuth_tests::*;

#[path = "support/contexts_keys.rs"]
mod contexts_keys;
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
    let declarations = if overlap {
        declare_components(&repo);
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
    declare_components(&repo);
    repo.write(KEY_FILE, &format!("{INSTANT}\n[payouts.instantly]\ntype = \"boolean\"\ncomponents = [\"api\"]\n"));
    assert!(repo.lint_findings(0).is_empty());
    repo.append(KEY_FILE, "\n[payouts]\ntype = \"boolean\"\ncomponents = [\"api\"]\n");
    key_findings_are_reported(&repo.lint_findings(1));
}

// behavior-contexts-keys: Lint reports a key whose `components` list is empty
#[test]
fn lint_reports_a_key_listed_for_no_component() {
    let repo = Repo::sample();
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
