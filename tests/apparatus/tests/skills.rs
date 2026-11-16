//! Group "`azimuth skills install`" of docs/increments/1.md.

use std::collections::BTreeSet;
use std::fs;

use azimuth_tests::*;

fn install(repo: &Repo, harness: &str) -> Output {
    let output = repo.azimuth(&["skills", "install", harness]);
    output.expect_status(0);
    output
}

fn new_paths(repo: &Repo, before: &BTreeSet<String>) -> Vec<String> {
    repo.untracked().into_iter().filter(|path| !before.contains(path)).collect()
}

fn installed_text(repo: &Repo) -> String {
    let before = BTreeSet::new();
    install(repo, "claude");
    new_paths(repo, &before).iter().map(|path| repo.read(path)).collect::<Vec<_>>().join("\n")
}

// behavior: It writes the skill as tracked files, in one copy
#[test]
fn skills_install_writes_visible_files() {
    let repo = Repo::sample();
    install(&repo, "claude");
    let written = repo.untracked();
    assert!(!written.is_empty(), "nothing was written, or only ignored files");
    let mut contents = BTreeSet::new();
    for path in &written {
        let metadata = fs::symlink_metadata(repo.path(path)).unwrap();
        assert!(metadata.is_file(), "{path} is not a plain file");
        assert!(contents.insert(repo.read(path)), "{path} repeats the text of another installed file");
    }
    let ignored = repo.git(&["status", "--porcelain", "--ignored"]);
    assert!(
        !ignored.lines().any(|line| line.starts_with("!!") && !line.contains(".azimuth/")),
        "installed files are ignored by git:\n{ignored}"
    );
}

// behavior: For a second harness it writes a link to that copy
#[test]
fn a_second_harness_gets_a_link() {
    let repo = Repo::sample();
    install(&repo, "claude");
    let first: BTreeSet<String> = repo.untracked().into_iter().collect();
    install(&repo, "codex");
    let second = new_paths(&repo, &first);
    assert!(!second.is_empty(), "nothing was written for the second harness");
    for path in &second {
        let metadata = fs::symlink_metadata(repo.path(path)).unwrap();
        assert!(metadata.file_type().is_symlink(), "{path} is a second copy, not a link");
        let target = fs::canonicalize(repo.path(path)).unwrap();
        let inside_first_copy = first.iter().any(|copy| {
            let copy = fs::canonicalize(repo.path(copy)).unwrap();
            copy == target || copy.starts_with(&target)
        });
        assert!(inside_first_copy, "{path} points at {}, which is not the first copy", target.display());
    }
}

// behavior: Each installed file carries one line with the digest of the shipped text
#[test]
fn each_installed_file_names_its_shipped_digest() {
    let repo = Repo::sample();
    install(&repo, "claude");
    for path in repo.untracked() {
        let text = repo.read(&path);
        let lines: Vec<&str> = text
            .lines()
            .filter(|line| {
                line.split(|c: char| c.is_whitespace() || "\"'`()[]<>,;".contains(c)).any(is_digest)
            })
            .collect();
        assert_eq!(lines.len(), 1, "{path} has {} digest lines:\n{text}", lines.len());
    }
}

// behavior: Run again by a newer CLI, it leaves an edited file alone and names it
#[test]
fn a_second_install_keeps_an_edited_file_and_names_it() {
    let repo = Repo::sample();
    install(&repo, "claude");
    let installed = repo.untracked();
    let edited = installed[0].clone();
    repo.append(&edited, "\nOur team also stops after the build.\n");
    let kept = repo.read(&edited);
    let untouched: Vec<(String, String)> = installed[1..].iter().map(|path| (path.clone(), repo.read(path))).collect();
    let output = install(&repo, "claude");
    assert_eq!(repo.read(&edited), kept, "the edited file was replaced");
    assert!(output.text().contains(&edited), "the edited file is not named:\n{}", output.text());
    for (path, text) in untouched {
        assert_eq!(repo.read(&path), text, "{path} changed although the CLI did not");
    }
}

// behavior: The installed skill has four steps and stops for a person after the proposal, after the intent and at the hand over
#[test]
fn the_skill_names_its_steps_and_stops() {
    // Whether the prose says the route well is for a person to read. The test asserts that every
    // command the route runs is named and that the skill speaks of stopping.
    let repo = Repo::sample();
    let text = installed_text(&repo);
    for command in ["azimuth lint", "azimuth change show", "azimuth run", "azimuth run --lane commit", "azimuth status"] {
        assert!(text.contains(command), "the skill does not name {command:?}");
    }
    let lower = text.to_lowercase();
    for step in ["proposal", "intent", "build", "hand over"] {
        assert!(lower.contains(step), "the skill does not name the step {step:?}");
    }
    assert!(lower.matches("stop").count() >= 3, "the skill names fewer than three stops");
}
