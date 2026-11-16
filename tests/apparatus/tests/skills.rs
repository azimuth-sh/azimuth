//! Group "`azimuth skills install`" of docs/increments/1.md.

use std::fs;

use azimuth_tests::*;

fn install(repo: &Repo, harness: &str) -> Output {
    let output = repo.azimuth(&["skills", "install", harness]);
    output.expect_status(0);
    output
}

/// Asserts that a file ends with a digest line that is correct for the rest of the file, and
/// returns the rest.
fn digest_line_is_correct(repo: &Repo, relative: &str) -> String {
    let file = repo.read(relative);
    let (text, last) = split_last_line(&file);
    assert_eq!(last, skill_digest_line(&text), "{relative} does not end with the digest line of its text:\n{file}");
    text
}

/// A skill file as an earlier CLI would have left it: other text, and the correct digest line for
/// that text.
fn earlier_shipped_file() -> String {
    let text = "# azimuth-change\n\nAn earlier version of the default process.\n\n1. Proposal. Stop for a person.\n";
    format!("{text}{}\n", skill_digest_line(text))
}

/// Paths the install wrote outside git's knowledge, every one of them inside the skill folders.
fn written(repo: &Repo) -> Vec<String> {
    let paths = repo.untracked();
    for path in &paths {
        assert!(
            path.starts_with(&format!("{SKILL_FOLDER}/")) || path == CLAUDE_SKILL_LINK,
            "skills install wrote {path}, outside the skill folders"
        );
    }
    paths
}

// behavior: It writes the skill as tracked files, in one copy
#[test]
fn skills_install_writes_one_visible_copy() {
    let repo = Repo::sample();
    install(&repo, "claude");
    let paths = written(&repo);
    assert!(paths.contains(&SKILL_FILE.to_string()), "{paths:?}");
    for path in paths.iter().filter(|path| path.starts_with(SKILL_FOLDER)) {
        assert!(fs::symlink_metadata(repo.path(path)).unwrap().is_file(), "{path} is not a plain file");
    }
    let ignored = repo.git(&["status", "--porcelain", "--ignored"]);
    assert!(
        !ignored.lines().any(|line| line.starts_with("!!") && !line.contains(".azimuth/")),
        "installed files are ignored by git:\n{ignored}"
    );
    // A file reached through a link is the same copy, so paths through a link are left out.
    let through_a_link = |file: &std::path::Path| {
        file.ancestors()
            .take_while(|dir| *dir != repo.root())
            .any(|dir| fs::symlink_metadata(dir).is_ok_and(|meta| meta.file_type().is_symlink()))
    };
    let copies: Vec<_> = files_under(repo.root())
        .into_iter()
        .filter(|file| !file.starts_with(repo.path(".git")) && !through_a_link(file))
        .filter(|file| file.file_name().is_some_and(|name| name == "SKILL.md"))
        .collect();
    assert_eq!(copies, vec![repo.path(SKILL_FILE)], "more than one copy of the text");
}

// behavior: For a second harness it writes a link to that copy
#[test]
fn a_second_harness_gets_a_link_and_no_copy() {
    let repo = Repo::sample();
    install(&repo, "codex");
    let text = repo.read(SKILL_FILE);
    install(&repo, "claude");
    assert_eq!(repo.read(SKILL_FILE), text, "the second harness rewrote the copy");
    let link = repo.path(CLAUDE_SKILL_LINK);
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink(), "{CLAUDE_SKILL_LINK} is not a link");
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(repo.path(SKILL_FOLDER)).unwrap());
    install(&repo, "codex");
    assert_eq!(repo.read(SKILL_FILE), text);
}

// behavior: Each installed file carries one line with the digest of the shipped text
#[test]
fn each_installed_file_names_its_shipped_digest() {
    let repo = Repo::sample();
    install(&repo, "claude");
    for path in written(&repo).iter().filter(|path| path.starts_with(SKILL_FOLDER)) {
        let text = repo.read(path);
        let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("<!-- azimuth text-1:")).collect();
        assert_eq!(lines.len(), 1, "{path} has {} digest lines:\n{text}", lines.len());
    }
}

// behavior: The digest line of an installed file is the `text-1` digest of the file's text without that line
#[test]
fn the_digest_line_is_the_text_1_digest_of_the_rest() {
    let repo = Repo::sample();
    install(&repo, "codex");
    let text = digest_line_is_correct(&repo, SKILL_FILE);
    assert!(!text.contains("<!-- azimuth"), "the digest line is not the only one:\n{text}");
}

// behavior: The command is `azimuth skills install <harness>`, and it knows `claude` and `codex`
#[test]
fn the_two_known_harnesses_install_and_another_is_refused() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(SKILL_FILE).is_file(), "{harness}");
    }
    let repo = Repo::sample();
    for args in [&["skills", "install", "cursor"][..], &["skills", "install"][..], &["skills", "add", "claude"][..]] {
        repo.azimuth(args).expect_status(2);
    }
    assert!(repo.untracked().is_empty(), "a refused install wrote {:?}", repo.untracked());
}

// behavior: The text is written to `.agents/skills/azimuth-change/SKILL.md` for either harness
#[test]
fn the_text_is_in_the_agents_folder_for_either_harness() {
    let claude = Repo::sample();
    install(&claude, "claude");
    let codex = Repo::sample();
    install(&codex, "codex");
    assert!(fs::symlink_metadata(claude.path(SKILL_FILE)).unwrap().is_file());
    assert_eq!(claude.read(SKILL_FILE), codex.read(SKILL_FILE));
    assert!(!codex.path(".claude").exists(), "codex got a link for claude");
}

// behavior: For `claude` it adds a link at `.claude/skills/azimuth-change`
#[test]
fn claude_gets_a_link_to_the_skill_folder() {
    let repo = Repo::sample();
    install(&repo, "claude");
    let link = repo.path(CLAUDE_SKILL_LINK);
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink(), "{CLAUDE_SKILL_LINK} is not a link");
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(repo.path(SKILL_FOLDER)).unwrap());
    assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), repo.read(SKILL_FILE));
}

// behavior: The digest line is the last line of the file, written `<!-- azimuth text-1:<hex> -->`
#[test]
fn the_digest_line_is_the_last_line_in_its_written_form() {
    let repo = Repo::sample();
    install(&repo, "claude");
    let file = repo.read(SKILL_FILE);
    let last = file.lines().last().unwrap_or_default();
    let hex = last
        .strip_prefix("<!-- azimuth text-1:")
        .and_then(|rest| rest.strip_suffix(" -->"))
        .unwrap_or_else(|| panic!("the last line is not a digest line: {last:?}"));
    assert_eq!(hex.len(), 64, "{last}");
    assert!(hex.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)), "{last}");
}

// behavior: Run again by a newer CLI, it replaces a file nobody edited
#[test]
fn a_second_install_replaces_a_file_an_earlier_version_shipped() {
    let repo = Repo::sample();
    repo.write(SKILL_FILE, &earlier_shipped_file());
    repo.commit_all("the default process of an earlier version");
    let output = install(&repo, "codex");
    let fresh = Repo::sample();
    install(&fresh, "codex");
    assert_eq!(repo.read(SKILL_FILE), fresh.read(SKILL_FILE), "the unedited file was not replaced:\n{}", output.text());
    digest_line_is_correct(&repo, SKILL_FILE);
}

// behavior: Run again by a newer CLI, it leaves an edited file alone and names it
#[test]
fn a_second_install_keeps_an_edited_file_and_names_it() {
    let repo = Repo::sample();
    let edited = earlier_shipped_file().replacen("Stop for a person.", "Stop for a person, and for the team lead.", 1);
    repo.write(SKILL_FILE, &edited);
    repo.commit_all("the default process of an earlier version, edited by the team");
    let output = install(&repo, "claude");
    assert_eq!(repo.read(SKILL_FILE), edited, "the edited file was replaced");
    assert!(output.text().contains(SKILL_FILE), "the edited file is not named:\n{}", output.text());
}

// behavior: The installed skill has four steps and stops for a person after the proposal, after the intent and at the hand over
#[test]
fn the_skill_names_its_steps_and_stops() {
    // Whether the prose says the route well is for a person to read. The test asserts that every
    // command the route runs is named and that the skill speaks of stopping.
    let repo = Repo::sample();
    install(&repo, "claude");
    let text = repo.read(SKILL_FILE);
    for command in ["azimuth lint", "azimuth change show", "azimuth run", "azimuth run --lane commit", "azimuth status"] {
        assert!(text.contains(command), "the skill does not name {command:?}");
    }
    let lower = text.to_lowercase();
    for step in ["proposal", "intent", "build", "hand over"] {
        assert!(lower.contains(step), "the skill does not name the step {step:?}");
    }
    assert!(lower.matches("stop").count() >= 3, "the skill names fewer than three stops");
}

// behavior: The installed skill tells the agent to use `~/.azimuth/bin/azimuth` when `azimuth` is not found
#[test]
fn the_skill_names_the_installed_binary_for_when_azimuth_is_not_found() {
    // The words around the path are for the agent to read and are not asserted.
    let repo = Repo::sample();
    install(&repo, "codex");
    let text = digest_line_is_correct(&repo, SKILL_FILE);
    assert!(text.contains("~/.azimuth/bin/azimuth"), "the skill does not name ~/.azimuth/bin/azimuth:\n{text}");
}
