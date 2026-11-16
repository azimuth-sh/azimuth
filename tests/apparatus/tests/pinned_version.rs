//! Group "The pinned CLI version" of docs/increments/1.md.

use azimuth_tests::*;

fn pin(repo: &Repo, value: &str) {
    repo.replace(PROJECT_FILE, &format!("cli = \"{PINNED_CLI}\""), &format!("cli = \"{value}\""));
}

// behavior: The project file holds one exact CLI version, and a range is refused
#[test]
fn a_version_range_is_a_reading_error() {
    for range in [">=0.2.0", "^0.2.0-alpha.1", "0.2.*", "~0.2"] {
        let repo = Repo::sample();
        pin(&repo, range);
        let line = repo.line_of(PROJECT_FILE, "cli =");
        repo.expect_reading_error(Some(PROJECT_FILE), Some(line));
    }
}

// behavior: The project file holds one exact CLI version, and a range is refused
#[test]
fn the_exact_pinned_version_works_without_a_warning() {
    let repo = Repo::sample();
    let output = repo.azimuth(&["status", "--json"]);
    output.expect_status(0);
    assert!(!output.stderr.contains(PINNED_CLI), "a warning about the version on the pinned version:\n{}", output.stderr);
}

// behavior: With another version, lint and the reading commands work and print a warning
#[test]
fn lint_and_reading_commands_warn_under_another_version() {
    let repo = Repo::sample();
    pin(&repo, OTHER_CLI);
    for args in [
        &["lint", "--json"][..],
        &["status", "--json"][..],
        &["requirement", "show", LOAN_LIMIT, "--json"][..],
        &["check", "show", SIXTH_LOAN, "--json"][..],
        &["change", "show", "--json"][..],
    ] {
        let output = repo.azimuth(args);
        output.expect_status(0);
        output.json();
        assert!(
            output.stderr.contains(OTHER_CLI),
            "azimuth {args:?} gives no warning that names the pinned version:\n{}",
            output.stderr
        );
    }
}

// behavior: With another version, `azimuth run` runs nothing and prints the command that installs the pinned version
#[test]
fn run_refuses_under_another_version() {
    let repo = Repo::sample();
    pin(&repo, OTHER_CLI);
    let output = repo.azimuth(&["run"]);
    output.expect_status(2);
    assert!(output.text().contains(OTHER_CLI), "run does not name the version to install:\n{}", output.text());
    assert!(repo.adapter_starts().is_empty(), "run started the adapter under another version");
    assert!(repo.store_files().is_empty() && repo.outbox_files().is_empty(), "run wrote results");
}

// behavior: Changing only the CLI version changes no fingerprint
#[test]
fn the_cli_version_is_not_a_part_of_the_fingerprint() {
    // The pin is a line of a tracked file, so moving it changes the files part. What must not
    // move is the method part, where the version would sit if it were an input.
    let repo = Repo::sample();
    repo.azimuth(&["run", "--json"]).expect_status(0);
    pin(&repo, OTHER_CLI);
    let output = repo.azimuth(&["check", "show", SIXTH_LOAN, "--json"]);
    output.expect_status(0);
    let parts = parts_changed(&output.json());
    assert!(mentions(&parts, "files"), "{parts:?}");
    assert!(!mentions(&parts, "method"), "the CLI version moved the method part: {parts:?}");
    assert!(!mentions(&parts, "environment"), "the CLI version moved the environment part: {parts:?}");
}
