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
