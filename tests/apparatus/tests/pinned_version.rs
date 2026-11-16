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

const READING_COMMANDS: [&[&str]; 5] = [
    &["lint", "--json"],
    &["status", "--json"],
    &["requirement", "show", LOAN_LIMIT, "--json"],
    &["check", "show", SIXTH_LOAN, "--json"],
    &["change", "show", "--json"],
];

// behavior: With another version, lint and the reading commands work and print a warning
#[test]
fn lint_and_reading_commands_warn_under_another_version() {
    let repo = Repo::sample();
    pin(&repo, OTHER_CLI);
    for args in READING_COMMANDS {
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

// behavior: Under another version, lint and the reading commands print one warning line on standard error that names the pinned version
#[test]
fn the_warning_is_one_line_on_standard_error() {
    let repo = Repo::sample();
    pin(&repo, OTHER_CLI);
    let without_json: Vec<Vec<&str>> =
        READING_COMMANDS.iter().map(|args| args.iter().copied().filter(|arg| *arg != "--json").collect()).collect();
    for args in READING_COMMANDS.iter().map(|args| args.to_vec()).chain(without_json) {
        let output = repo.azimuth(&args);
        output.expect_status(0);
        let warnings: Vec<&str> = output.stderr.lines().filter(|line| line.contains(OTHER_CLI)).collect();
        assert_eq!(warnings.len(), 1, "azimuth {args:?} warns on {} lines:\n{}", warnings.len(), output.stderr);
        assert!(!output.stdout.contains(OTHER_CLI), "azimuth {args:?} warns on standard output:\n{}", output.stdout);
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

// behavior: Changing only the pinned version in the project file moves the files part and no other part
#[test]
fn raising_the_pin_moves_only_the_files_part() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    pin(&repo, OTHER_CLI);
    for check in ALL_CHECKS {
        let document = repo.check_show(check);
        assert_eq!(document["state"], "owed", "{document:#}");
        assert_eq!(document["reason"], "an input changed", "{document:#}");
        assert_eq!(parts_changed(&document), vec!["files"], "{document:#}");
        assert_eq!(files_changed(&document), vec![PROJECT_FILE.to_string()], "{document:#}");
    }
}

const INSTALL_LINE: &str = "curl -fsSL https://azimuth.sh/install | sh";

// behavior: The command it prints is the one install line, which names no version
#[test]
fn run_prints_the_one_install_line() {
    let repo = Repo::sample();
    pin(&repo, OTHER_CLI);
    let output = repo.azimuth(&["run"]);
    output.expect_status(2);
    let text = output.text();
    let lines: Vec<&str> = text.lines().filter(|line| line.contains("azimuth.sh/install")).collect();
    assert_eq!(lines.len(), 1, "the install line is not printed once:\n{text}");
    // The words before the command may name the pinned version; the command itself is the
    // install line with nothing after it, so no version rides on it.
    let at = lines[0].find(INSTALL_LINE).unwrap_or_else(|| panic!("the printed command is not the install line: {:?}", lines[0]));
    assert_eq!(lines[0][at + INSTALL_LINE.len()..].trim(), "", "something follows `| sh`: {:?}", lines[0]);
    assert!(text.contains(OTHER_CLI), "the refusal does not name the pinned version:\n{text}");
}
