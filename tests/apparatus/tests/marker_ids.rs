//! Punctuation around a marker belongs to the source language, not to the Check id.

use azimuth_tests::*;

fn rewrite_markers(repo: &Repo, form: &str, source: impl Fn(&str) -> String) {
    repo.replace(
        PROJECT_FILE,
        "marker = \"# Check: <id>\"",
        &format!("marker = {}", toml::Value::String(form.into())),
    );
    for file in [LOAN_LIMIT_TESTS, LATE_FINE_TESTS, FREE_RETURN_TESTS] {
        let mut text = repo.read(file);
        for id in ALL_CHECKS {
            text = text.replace(&format!("# Check: {id}"), &source(id));
        }
        repo.write(file, &text);
    }
}

// behavior-2: With a marker form that ends with `<id>`, the id ends at the first character that is not a letter, a digit or a hyphen
#[test]
fn punctuation_after_an_id_is_not_part_of_the_marker() {
    for (form, quotes) in [("@Check:<id>", true), ("# Check: <id>", false)] {
        let repo = Repo::sample();
        rewrite_markers(&repo, form, |id| {
            if quotes {
                format!("tag: '@Check:{id}',")
            } else {
                format!("# Check: {id}.")
            }
        });
        assert!(repo.lint_findings(0).is_empty());
        if quotes {
            repo.replace(
                LATE_FINE_TESTS,
                &format!("'@Check:{LATE_RETURN}',"),
                &format!("\"@Check:{LATE_RETURN}\",next"),
            );
            assert!(repo.lint_findings(0).is_empty());
        }
    }
}

// behavior-2: With a marker form that ends with `<id>`, the id ends at the first character that is not a letter, a digit or a hyphen
#[test]
fn a_full_stop_after_a_comment_marker_ends_the_id() {
    let repo = Repo::sample();
    rewrite_markers(&repo, "# Check: <id>", |id| format!("# Check: {id}."));
    assert!(repo.lint_findings(0).is_empty());
}

// behavior-2: With a marker form that ends with `<id>`, the id ends at the first character that is not a letter, a digit or a hyphen
#[test]
fn a_digit_belongs_to_the_id_and_an_underscore_ends_it() {
    let repo = Repo::sample();
    repo.replace(
        LENDING_VERIFICATION,
        &format!("- Check: {SIXTH_LOAN}"),
        "- Check: loan-limit-2",
    );
    repo.replace(
        LOAN_LIMIT_TESTS,
        &format!("# Check: {SIXTH_LOAN}"),
        "# Check: loan-limit-2_tail",
    );
    assert!(repo.lint_findings(0).is_empty());
}

// behavior-2: With a text after `<id>` in the form, the id ends where that text starts
#[test]
fn a_suffix_in_the_form_delimits_the_whole_id() {
    let repo = Repo::sample();
    rewrite_markers(&repo, "# Check: <id>!", |id| format!("# Check: {id}!"));
    assert!(repo.lint_findings(0).is_empty());
    repo.replace(
        LOAN_LIMIT_TESTS,
        &format!("# Check: {SIXTH_LOAN}!"),
        &format!("# Check: {SIXTH_LOAN}.note!"),
    );
    let findings = repo.lint_findings(1);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(
        findings_at(
            &findings,
            LOAN_LIMIT_TESTS,
            Some(repo.line_of(LOAN_LIMIT_TESTS, ".note!"))
        )
        .len(),
        1,
        "{findings:#?}"
    );
    assert_eq!(
        findings_at(
            &findings,
            LENDING_VERIFICATION,
            Some(repo.line_of(LENDING_VERIFICATION, &format!("- Check: {SIXTH_LOAN}")))
        )
        .len(),
        1,
        "{findings:#?}"
    );
}

// behavior-2: With a text after `<id>` in the form, the id ends where that text starts
#[test]
fn two_suffixed_markers_on_one_line_both_assign_their_checks() {
    let repo = Repo::sample();
    rewrite_markers(&repo, "# Check: <id>!", |id| format!("# Check: {id}!"));
    repo.write(
        LOAN_LIMIT_TESTS,
        &format!("# Check: {SIXTH_LOAN}! # Check: {RETURNED_BOOK}!\n"),
    );
    assert!(repo.lint_findings(0).is_empty());
}

// behavior-2: A marker whose id holds a capital letter is found, and lint reports that it names no Check
#[test]
fn an_uppercase_id_is_found_and_reported_as_unknown() {
    let repo = Repo::sample();
    repo.replace(
        LOAN_LIMIT_TESTS,
        &format!("# Check: {SIXTH_LOAN}"),
        "# Check: Sixth-loan-is-refused.",
    );
    let findings = repo.lint_findings(1);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(
        findings_at(
            &findings,
            LOAN_LIMIT_TESTS,
            Some(repo.line_of(LOAN_LIMIT_TESTS, "Sixth-loan-is-refused"))
        )
        .len(),
        1,
        "{findings:#?}"
    );
    assert_eq!(
        findings_at(
            &findings,
            LENDING_VERIFICATION,
            Some(repo.line_of(LENDING_VERIFICATION, &format!("- Check: {SIXTH_LOAN}")))
        )
        .len(),
        1,
        "{findings:#?}"
    );
}

// behavior-2: A marker whose id holds a capital letter is found, and lint reports that it names no Check
#[test]
fn the_unknown_check_finding_names_the_complete_id_without_the_full_stop() {
    let repo = Repo::sample();
    repo.replace(
        LOAN_LIMIT_TESTS,
        &format!("# Check: {SIXTH_LOAN}"),
        "# Check: Sixth-loan-is-refused.",
    );
    let findings = repo.lint_findings(1);
    let at_marker = findings_at(
        &findings,
        LOAN_LIMIT_TESTS,
        Some(repo.line_of(LOAN_LIMIT_TESTS, "Sixth-loan-is-refused")),
    );
    assert_eq!(at_marker.len(), 1, "{findings:#?}");
    let finding = at_marker[0];
    assert_eq!(finding["kind"], "unknown check", "{finding:#}");
    let message = finding["message"].as_str().unwrap();
    // The diagnostic may explain the repair freely; only the named identifier is compared.
    let named = ['`', '\'', '"']
        .into_iter()
        .find_map(|quote| {
            message
                .split(quote)
                .enumerate()
                .find(|(index, part)| index % 2 == 1 && part.contains("Sixth-loan-is-refused"))
                .map(|(_, part)| part)
        })
        .expect("the unknown-Check diagnostic must delimit the identifier it names");
    assert_eq!(named, "Sixth-loan-is-refused", "{finding:#}");
}
