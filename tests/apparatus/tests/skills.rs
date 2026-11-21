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

/// The skill text as the released `0.2.0-alpha.2` shipped it, without the digest line.
const ALPHA_2_TEXT: &str = include_str!("fixtures/azimuth-change-0.2.0-alpha.2.md");

/// The text of one numbered step, from its heading to the next heading.
fn step_text(text: &str, number: u32) -> String {
    let heading = format!("\n## {number}.");
    let start = text.find(&heading).unwrap_or_else(|| panic!("the skill has no step {number}:\n{text}")) + 1;
    let rest = &text[start..];
    let end = rest[1..].find("\n## ").map_or(rest.len(), |index| index + 2);
    rest[..end].to_string()
}

/// The paragraph, between blank lines, that holds the given position.
fn paragraph_at(text: &str, at: usize) -> &str {
    let start = text[..at].rfind("\n\n").map_or(0, |index| index + 2);
    let end = text[at..].find("\n\n").map_or(text.len(), |index| at + index);
    &text[start..end]
}

/// Paths the install wrote outside git's knowledge, every one of them inside the skill folders.
fn written(repo: &Repo) -> Vec<String> {
    let paths = repo.untracked();
    for path in &paths {
        assert!(
            path.starts_with(&format!("{SKILL_FOLDER}/"))
                || path == CLAUDE_SKILL_LINK
                || path.starts_with(&format!("{INITIATIVE_SKILL_FOLDER}/"))
                || path == CLAUDE_INITIATIVE_SKILL_LINK
                || path.starts_with(&format!("{EXPLORATION_SKILL_FOLDER}/"))
                || path == CLAUDE_EXPLORATION_SKILL_LINK,
            "skills install wrote {path}, outside the skill folders"
        );
    }
    paths
}

// Rows of prose in the lists are shown by a person's reading; this test keeps the shipped
// text equal to what was read.
#[test]
fn installed_skill_matches_the_text_a_person_accepted() {
    // A person accepted the skill text on 2026-11-20 and its changes on 2026-11-21.
    const ACCEPTED_SKILL_SHA256: &str =
        "d525e128cda650911483d8fd062ab7ad9eceedb769f93e65e629dec674c5a4ca";

    let repo = Repo::sample();
    install(&repo, "codex");
    let (text, _) = split_last_line(&repo.read(SKILL_FILE));
    assert_eq!(
        sha256_hex(text.as_bytes()),
        ACCEPTED_SKILL_SHA256,
        "The skill text differs from the text that a person accepted. A person reads the new \
         text or its diff; then a session that did not write the text sets this value. \
         The author of a text never does."
    );
}

#[test]
fn installed_initiative_skill_matches_the_text_a_person_accepted() {
    // A person accepted the whole skill text on 2026-11-21.
    const ACCEPTED_SKILL_SHA256: &str =
        "f9c896cf335fb472fd5b6ad42bef406139d13e6ecd3af7031a1a184639a1b62d";

    let repo = Repo::sample();
    install(&repo, "codex");
    let (text, _) = split_last_line(&repo.read(INITIATIVE_SKILL_FILE));
    assert_eq!(
        sha256_hex(text.as_bytes()),
        ACCEPTED_SKILL_SHA256,
        "The skill text differs from the text that a person accepted. A person reads the new \
         text or its diff; then a session that did not write the text sets this value. \
         The author of a text never does."
    );
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
    let mut expected = vec![repo.path(SKILL_FILE)];
    // The initiative skill has its own copy; this test still checks the change skill on a
    // CLI that has not added the initiative skill yet.
    if repo.path(INITIATIVE_SKILL_FILE).is_file() {
        expected.push(repo.path(INITIATIVE_SKILL_FILE));
    }
    if repo.path(EXPLORATION_SKILL_FILE).is_file() {
        expected.push(repo.path(EXPLORATION_SKILL_FILE));
    }
    let mut copies = copies;
    copies.sort();
    expected.sort();
    assert_eq!(copies, expected, "more than one copy of a skill's text");
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
// behavior-2-initiative-skill: For either harness the command still writes the change skill to `.agents/skills/azimuth-change/SKILL.md`
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

// behavior-2-exploration-skill: For either harness the command still writes the change skill and the initiative skill
#[test]
fn the_change_and_initiative_texts_are_still_installed_for_either_harness() {
    let claude = Repo::sample();
    install(&claude, "claude");
    let codex = Repo::sample();
    install(&codex, "codex");
    for file in [SKILL_FILE, INITIATIVE_SKILL_FILE] {
        assert!(fs::symlink_metadata(claude.path(file)).unwrap().is_file(), "{file}");
        assert!(fs::symlink_metadata(codex.path(file)).unwrap().is_file(), "{file}");
        let text = claude.read(file);
        assert!(!split_last_line(&text).0.trim().is_empty(), "{file} has no text");
        assert_eq!(text, codex.read(file), "the harnesses got different texts for {file}");
    }
}

// behavior-2-initiative-skill: For either harness the command writes the text of the initiative skill to `.agents/skills/azimuth-initiative/SKILL.md`
#[test]
fn the_initiative_text_is_in_the_agents_folder_for_either_harness() {
    let mut texts = Vec::new();
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(),
            "the initiative skill is not installed for {harness}");
        assert!(fs::symlink_metadata(repo.path(INITIATIVE_SKILL_FILE)).unwrap().is_file());
        let text = repo.read(INITIATIVE_SKILL_FILE);
        assert!(!split_last_line(&text).0.trim().is_empty(), "the initiative skill has no text");
        texts.push(text);
        if harness == "codex" {
            assert!(!repo.path(".claude").exists(), "codex got a link for claude");
        }
    }
    assert_eq!(texts[0], texts[1], "the harnesses got different initiative texts");
}

// behavior-2-exploration-skill: For either harness the command writes the text of the exploration skill to `.agents/skills/azimuth-exploration/SKILL.md`
#[test]
fn the_exploration_text_is_in_the_agents_folder_for_either_harness() {
    let mut texts = Vec::new();
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(),
            "the exploration skill is not installed for {harness}");
        assert!(fs::symlink_metadata(repo.path(EXPLORATION_SKILL_FILE)).unwrap().is_file());
        let text = repo.read(EXPLORATION_SKILL_FILE);
        assert!(!split_last_line(&text).0.trim().is_empty(), "the exploration skill has no text");
        texts.push(text);
        if harness == "codex" {
            assert!(!repo.path(".claude").exists(), "codex got a link for claude");
        }
    }
    assert_eq!(texts[0], texts[1], "the harnesses got different exploration texts");
}

// behavior-2-initiative-skill: For `claude` it adds a link at `.claude/skills/azimuth-initiative`
#[test]
fn claude_gets_a_link_to_the_initiative_skill_folder() {
    let repo = Repo::sample();
    install(&repo, "claude");
    assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(), "the initiative skill is not installed");
    let link = repo.path(CLAUDE_INITIATIVE_SKILL_LINK);
    assert!(fs::symlink_metadata(&link).expect("the initiative skill link").file_type().is_symlink());
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(repo.path(INITIATIVE_SKILL_FOLDER)).unwrap());
    assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), repo.read(INITIATIVE_SKILL_FILE));
}

// behavior-2-exploration-skill: For `claude` it adds a link at `.claude/skills/azimuth-exploration`
#[test]
fn claude_gets_a_link_to_the_exploration_skill_folder() {
    let repo = Repo::sample();
    install(&repo, "claude");
    assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed");
    let link = repo.path(CLAUDE_EXPLORATION_SKILL_LINK);
    assert!(fs::symlink_metadata(&link).expect("the exploration skill link").file_type().is_symlink());
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(repo.path(EXPLORATION_SKILL_FOLDER)).unwrap());
    assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), repo.read(EXPLORATION_SKILL_FILE));
}

// behavior-2-initiative-skill: The link is relative, and the skill is still found through it after the clone is moved
#[test]
fn the_initiative_skill_link_survives_a_move_of_the_clone() {
    let repo = Repo::sample();
    install(&repo, "claude");
    assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(), "the initiative skill is not installed");
    let original = repo.read(INITIATIVE_SKILL_FILE);
    let target = fs::read_link(repo.path(CLAUDE_INITIATIVE_SKILL_LINK)).unwrap();
    assert!(target.is_relative(), "the initiative skill link is absolute: {target:?}");
    repo.commit_all("install the two skills");
    let cloned = repo.beside("clone");
    repo.git(&["clone", "--quiet", repo.root().to_str().unwrap(), cloned.to_str().unwrap()]);
    let moved = repo.beside("moved-clone");
    fs::rename(&cloned, &moved).unwrap();
    assert!(!cloned.exists(), "the original clone location remains");
    let link = moved.join(CLAUDE_INITIATIVE_SKILL_LINK);
    assert!(fs::read_link(&link).unwrap().is_relative());
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(moved.join(INITIATIVE_SKILL_FOLDER)).unwrap());
    assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), original);
}

// behavior-2-exploration-skill: The link is relative, and the skill is still found through it after the clone is moved
#[test]
fn the_exploration_skill_link_survives_a_move_of_the_clone() {
    let repo = Repo::sample();
    install(&repo, "claude");
    assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed");
    let original = repo.read(EXPLORATION_SKILL_FILE);
    let target = fs::read_link(repo.path(CLAUDE_EXPLORATION_SKILL_LINK)).unwrap();
    assert!(target.is_relative(), "the exploration skill link is absolute: {target:?}");
    repo.commit_all("install the three skills");
    let cloned = repo.beside("clone");
    repo.git(&["clone", "--quiet", repo.root().to_str().unwrap(), cloned.to_str().unwrap()]);
    let moved = repo.beside("moved-clone");
    fs::rename(&cloned, &moved).unwrap();
    assert!(!cloned.exists(), "the original clone location remains");
    let link = moved.join(CLAUDE_EXPLORATION_SKILL_LINK);
    assert!(fs::read_link(&link).unwrap().is_relative());
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(moved.join(EXPLORATION_SKILL_FOLDER)).unwrap());
    assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), original);
}

// behavior-2-initiative-skill: The last line of the initiative skill file is its own digest line, `<!-- azimuth text-1:<hex> -->` over the file's text without that line
#[test]
fn the_initiative_skill_ends_with_its_own_text_1_digest() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(), "the initiative skill is not installed for {harness}");
        let text = digest_line_is_correct(&repo, INITIATIVE_SKILL_FILE);
        assert!(!text.contains("<!-- azimuth text-1:"), "the initiative skill has another digest line");
    }
}

// behavior-2-exploration-skill: The last line of the exploration skill file is its own digest line, `<!-- azimuth text-1:<hex> -->` over the file's text without that line
#[test]
fn the_exploration_skill_ends_with_its_own_text_1_digest() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed for {harness}");
        let text = digest_line_is_correct(&repo, EXPLORATION_SKILL_FILE);
        assert!(!text.contains("<!-- azimuth text-1:"), "the exploration skill has another digest line");
    }
}

fn earlier_initiative_shipped_file() -> String {
    let text = "# azimuth-initiative\n\nAn earlier version of the initiative form.\n";
    format!("{text}{}\n", skill_digest_line(text))
}

fn earlier_exploration_shipped_file() -> String {
    let text = "# azimuth-exploration\n\nAn earlier version of the exploration method.\n";
    format!("{text}{}\n", skill_digest_line(text))
}

// behavior-2-initiative-skill: Run again, the command replaces an initiative skill file that carries a correct digest line of another text
#[test]
fn a_second_install_replaces_an_unedited_earlier_initiative_skill() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(), "the initiative skill is not installed for {harness}");
        let expected = repo.read(INITIATIVE_SKILL_FILE);
        let earlier = earlier_initiative_shipped_file();
        assert_ne!(expected, earlier);
        repo.write(INITIATIVE_SKILL_FILE, &earlier);
        digest_line_is_correct(&repo, INITIATIVE_SKILL_FILE);
        repo.commit_all("an unedited initiative skill from an earlier version");
        install(&repo, harness);
        assert_eq!(repo.read(INITIATIVE_SKILL_FILE), expected, "the unedited initiative skill was not replaced for {harness}");
        digest_line_is_correct(&repo, INITIATIVE_SKILL_FILE);
    }
}

// behavior-2-exploration-skill: Run again, the command replaces an exploration skill file that carries a correct digest line of another text
#[test]
fn a_second_install_replaces_an_unedited_earlier_exploration_skill() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed for {harness}");
        let expected = repo.read(EXPLORATION_SKILL_FILE);
        let earlier = earlier_exploration_shipped_file();
        assert_ne!(expected, earlier);
        repo.write(EXPLORATION_SKILL_FILE, &earlier);
        digest_line_is_correct(&repo, EXPLORATION_SKILL_FILE);
        repo.commit_all("an unedited exploration skill from an earlier version");
        install(&repo, harness);
        assert_eq!(repo.read(EXPLORATION_SKILL_FILE), expected, "the unedited exploration skill was not replaced for {harness}");
        digest_line_is_correct(&repo, EXPLORATION_SKILL_FILE);
    }
}

// behavior-2-initiative-skill: Run again, it leaves an edited initiative skill file alone and names it, and still replaces an unedited change skill
#[test]
fn an_edited_initiative_skill_does_not_prevent_a_change_skill_upgrade() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(), "the initiative skill is not installed for {harness}");
        let expected_change = repo.read(SKILL_FILE);
        let edited = earlier_initiative_shipped_file().replacen("\n", "\nA team edit.\n", 1);
        repo.write(INITIATIVE_SKILL_FILE, &edited);
        repo.write(SKILL_FILE, &earlier_shipped_file());
        assert_ne!(repo.read(SKILL_FILE), expected_change);
        repo.commit_all("edit the initiative skill and leave the earlier change skill unedited");
        let output = install(&repo, harness);
        assert_eq!(repo.read(INITIATIVE_SKILL_FILE), edited, "the edited initiative skill was replaced");
        assert!(output.text().contains(INITIATIVE_SKILL_FILE), "the edited initiative skill is not named:\n{}", output.text());
        assert_eq!(repo.read(SKILL_FILE), expected_change, "the unedited change skill was not upgraded");
        digest_line_is_correct(&repo, SKILL_FILE);
    }
}

// behavior-2-exploration-skill: Run again, it leaves an edited exploration skill file alone and names it, and still replaces an unedited change skill and an unedited initiative skill
#[test]
fn an_edited_exploration_skill_does_not_prevent_the_other_two_upgrades() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed for {harness}");
        let expected_change = repo.read(SKILL_FILE);
        let expected_initiative = repo.read(INITIATIVE_SKILL_FILE);
        let edited = earlier_exploration_shipped_file().replacen("\n", "\nA team edit.\n", 1);
        repo.write(EXPLORATION_SKILL_FILE, &edited);
        repo.write(SKILL_FILE, &earlier_shipped_file());
        repo.write(INITIATIVE_SKILL_FILE, &earlier_initiative_shipped_file());
        assert_ne!(repo.read(SKILL_FILE), expected_change);
        assert_ne!(repo.read(INITIATIVE_SKILL_FILE), expected_initiative);
        repo.commit_all("edit the exploration skill and leave the other two earlier skills unedited");
        let output = install(&repo, harness);
        assert_eq!(repo.read(EXPLORATION_SKILL_FILE), edited, "the edited exploration skill was replaced");
        assert!(output.text().contains(EXPLORATION_SKILL_FILE), "the edited exploration skill is not named:\n{}", output.text());
        assert_eq!(repo.read(SKILL_FILE), expected_change, "the unedited change skill was not upgraded");
        assert_eq!(repo.read(INITIATIVE_SKILL_FILE), expected_initiative, "the unedited initiative skill was not upgraded");
        digest_line_is_correct(&repo, SKILL_FILE);
        digest_line_is_correct(&repo, INITIATIVE_SKILL_FILE);
    }
}

// behavior-2-initiative-skill: Run again, it leaves an edited change skill file alone and names it, and still replaces an unedited initiative skill
#[test]
fn an_edited_change_skill_does_not_prevent_an_initiative_skill_upgrade() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(), "the initiative skill is not installed for {harness}");
        let expected_initiative = repo.read(INITIATIVE_SKILL_FILE);
        let edited = earlier_shipped_file().replacen("\n", "\nA team edit.\n", 1);
        repo.write(SKILL_FILE, &edited);
        repo.write(INITIATIVE_SKILL_FILE, &earlier_initiative_shipped_file());
        assert_ne!(repo.read(INITIATIVE_SKILL_FILE), expected_initiative);
        repo.commit_all("edit the change skill and leave the earlier initiative skill unedited");
        let output = install(&repo, harness);
        assert_eq!(repo.read(SKILL_FILE), edited, "the edited change skill was replaced");
        assert!(output.text().contains(SKILL_FILE), "the edited change skill is not named:\n{}", output.text());
        assert_eq!(repo.read(INITIATIVE_SKILL_FILE), expected_initiative, "the unedited initiative skill was not upgraded");
        digest_line_is_correct(&repo, INITIATIVE_SKILL_FILE);
    }
}

// behavior-2-exploration-skill: Run again, it leaves an edited change skill file alone and names it, and still replaces an unedited exploration skill
#[test]
fn an_edited_change_skill_does_not_prevent_an_exploration_skill_upgrade() {
    for harness in ["claude", "codex"] {
        let repo = Repo::sample();
        install(&repo, harness);
        assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed for {harness}");
        let expected_exploration = repo.read(EXPLORATION_SKILL_FILE);
        let edited = earlier_shipped_file().replacen("\n", "\nA team edit.\n", 1);
        repo.write(SKILL_FILE, &edited);
        repo.write(EXPLORATION_SKILL_FILE, &earlier_exploration_shipped_file());
        assert_ne!(repo.read(EXPLORATION_SKILL_FILE), expected_exploration);
        repo.commit_all("edit the change skill and leave the earlier exploration skill unedited");
        let output = install(&repo, harness);
        assert_eq!(repo.read(SKILL_FILE), edited, "the edited change skill was replaced");
        assert!(output.text().contains(SKILL_FILE), "the edited change skill is not named:\n{}", output.text());
        assert_eq!(repo.read(EXPLORATION_SKILL_FILE), expected_exploration, "the unedited exploration skill was not upgraded");
        digest_line_is_correct(&repo, EXPLORATION_SKILL_FILE);
    }
}

// behavior-2-initiative-skill: In a repository that has only the change skill of the version before, the command writes the initiative skill and its link, and replaces the change skill
#[test]
fn an_install_with_only_the_earlier_change_skill_gets_both_current_skills() {
    let repo = Repo::sample();
    let earlier = earlier_shipped_file();
    repo.write(SKILL_FILE, &earlier);
    fs::create_dir_all(repo.path(".claude/skills")).unwrap();
    std::os::unix::fs::symlink("../../.agents/skills/azimuth-change", repo.path(CLAUDE_SKILL_LINK)).unwrap();
    repo.commit_all("only the unedited change skill of an earlier version");
    assert!(!repo.path(INITIATIVE_SKILL_FOLDER).exists());
    install(&repo, "claude");
    assert!(repo.path(INITIATIVE_SKILL_FILE).is_file(), "the initiative skill is not installed");
    let fresh = Repo::sample();
    install(&fresh, "claude");
    assert_ne!(repo.read(SKILL_FILE), earlier, "the earlier change skill was not replaced");
    assert_eq!(repo.read(SKILL_FILE), fresh.read(SKILL_FILE));
    assert_eq!(repo.read(INITIATIVE_SKILL_FILE), fresh.read(INITIATIVE_SKILL_FILE));
    let link = repo.path(CLAUDE_INITIATIVE_SKILL_LINK);
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(repo.path(INITIATIVE_SKILL_FOLDER)).unwrap());
    assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), repo.read(INITIATIVE_SKILL_FILE));
    digest_line_is_correct(&repo, SKILL_FILE);
    digest_line_is_correct(&repo, INITIATIVE_SKILL_FILE);
}

// behavior-2-exploration-skill: In a repository that has only the change skill and the initiative skill of the version before, the command writes the exploration skill and its link, and replaces the other two
#[test]
fn an_install_with_only_the_two_earlier_skills_gets_all_three_current_skills() {
    let repo = Repo::sample();
    let earlier_change = earlier_shipped_file();
    let earlier_initiative = earlier_initiative_shipped_file();
    repo.write(SKILL_FILE, &earlier_change);
    repo.write(INITIATIVE_SKILL_FILE, &earlier_initiative);
    fs::create_dir_all(repo.path(".claude/skills")).unwrap();
    std::os::unix::fs::symlink("../../.agents/skills/azimuth-change", repo.path(CLAUDE_SKILL_LINK)).unwrap();
    std::os::unix::fs::symlink("../../.agents/skills/azimuth-initiative", repo.path(CLAUDE_INITIATIVE_SKILL_LINK)).unwrap();
    repo.commit_all("only the two unedited skills of an earlier version");
    assert!(!repo.path(EXPLORATION_SKILL_FOLDER).exists());
    assert!(fs::symlink_metadata(repo.path(CLAUDE_EXPLORATION_SKILL_LINK)).is_err());
    install(&repo, "claude");
    assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed");
    let fresh = Repo::sample();
    install(&fresh, "claude");
    assert_ne!(repo.read(SKILL_FILE), earlier_change, "the earlier change skill was not replaced");
    assert_ne!(repo.read(INITIATIVE_SKILL_FILE), earlier_initiative, "the earlier initiative skill was not replaced");
    for file in [SKILL_FILE, INITIATIVE_SKILL_FILE, EXPLORATION_SKILL_FILE] {
        assert_eq!(repo.read(file), fresh.read(file), "{file} differs from a fresh install");
        digest_line_is_correct(&repo, file);
    }
    let link = repo.path(CLAUDE_EXPLORATION_SKILL_LINK);
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert!(fs::read_link(&link).unwrap().is_relative());
    assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(repo.path(EXPLORATION_SKILL_FOLDER)).unwrap());
    assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), repo.read(EXPLORATION_SKILL_FILE));
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

// behavior: Run again by a newer CLI, it replaces a file nobody edited
#[test]
fn a_file_that_0_2_0_alpha_2_shipped_gets_the_current_text() {
    // Repositories that installed 0.2.0-alpha.2 hold these exact bytes. Their step 1 does not
    // move finished change folders, so the install must replace them with a text that differs.
    let shipped = format!("{ALPHA_2_TEXT}{}\n", skill_digest_line(ALPHA_2_TEXT));
    let repo = Repo::sample();
    repo.write(SKILL_FILE, &shipped);
    repo.commit_all("the default process of 0.2.0-alpha.2");
    let output = install(&repo, "claude");
    let fresh = Repo::sample();
    install(&fresh, "claude");
    assert_eq!(repo.read(SKILL_FILE), fresh.read(SKILL_FILE), "the file 0.2.0-alpha.2 wrote was not replaced:\n{}", output.text());
    assert_ne!(
        repo.read(SKILL_FILE),
        shipped,
        "the shipped text is still the text of 0.2.0-alpha.2, so a repository on that version gets nothing new"
    );
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
    for command in [
        "azimuth lint",
        "azimuth change diff",
        "azimuth run",
        "azimuth run --lane commit",
        "azimuth status",
    ] {
        assert!(
            text.contains(command),
            "the skill does not name {command:?}"
        );
    }
    let lower = text.to_lowercase();
    for step in ["proposal", "intent", "build", "hand over"] {
        assert!(
            lower.contains(step),
            "the skill does not name the step {step:?}"
        );
    }
    assert!(
        lower.matches("stop").count() >= 3,
        "the skill names fewer than three stops"
    );
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

// behavior: The installed skill tells the agent, in step 1 before it writes the proposal, to move every change folder that already exists on the target branch into `azimuth/changes/finished/`
#[test]
fn step_1_moves_the_finished_change_folders_before_the_proposal() {
    // The wording is for the agent to read and is not asserted. The test asserts the folder, the
    // branch whose folders move, the move itself, and that it comes before the proposal is
    // written and before the stop, inside step 1 and not as a step of its own.
    let repo = Repo::sample();
    install(&repo, "claude");
    let text = digest_line_is_correct(&repo, SKILL_FILE);
    let headings: Vec<&str> = text.lines().filter(|line| line.starts_with("## ")).collect();
    assert_eq!(headings.len(), 4, "the skill does not have four steps: {headings:?}");
    let step = step_text(&text, 1);
    let folder = "azimuth/changes/finished/";
    let move_at = step.find(folder).unwrap_or_else(|| panic!("step 1 does not name {folder}:\n{step}"));
    let paragraph = paragraph_at(&step, move_at).to_lowercase();
    assert!(paragraph.contains("move"), "step 1 names {folder} and does not say to move folders there:\n{step}");
    assert!(paragraph.contains("target branch"), "step 1 does not say that the folders on the target branch move:\n{step}");
    let write_at = step
        .find("azimuth/changes/<name>/proposal.md")
        .unwrap_or_else(|| panic!("step 1 no longer says where the proposal is written:\n{step}"));
    let says_before = paragraph.contains("before") && paragraph.contains("proposal") && !paragraph.contains("after");
    assert!(move_at < write_at || says_before, "step 1 does not move the folders before the proposal is written:\n{step}");
    let stop_at = step.find("Stop.").unwrap_or_else(|| panic!("step 1 does not stop:\n{step}"));
    assert!(move_at < stop_at, "step 1 moves the folders after its stop:\n{step}");
}

// behavior-2: The shipped skill names the command `azimuth change diff` at both stops
#[test]
fn both_stops_name_change_diff_and_neither_names_change_show() {
    let repo = Repo::sample();
    install(&repo, "codex");
    let text = digest_line_is_correct(&repo, SKILL_FILE);
    for number in [2, 4] {
        assert!(
            step_text(&text, number).contains("azimuth change diff"),
            "step {number} does not name azimuth change diff:\n{text}"
        );
    }
    assert!(
        !text.contains("azimuth change show"),
        "the skill names an unknown command:\n{text}"
    );
}

// behavior-2: Step 2 of the shipped skill no longer says to run lint until it reports nothing
#[test]
fn intent_does_not_wait_for_lint_to_report_nothing() {
    let repo = Repo::sample();
    install(&repo, "codex");
    let step = step_text(&digest_line_is_correct(&repo, SKILL_FILE), 2);
    assert!(step.contains("azimuth lint"));
    let lower = step
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    for instruction in lower.split(['.', ';']) {
        let demands_empty = instruction.contains("until")
            && ["nothing", "zero", "no findings", "none", "clean", "empty"]
                .iter()
                .any(|word| instruction.contains(word));
        let prohibits = instruction.contains("do not")
            || instruction.contains("don't")
            || instruction.contains("never")
            || instruction.contains("not required");
        assert!(
            !demands_empty || prohibits,
            "Intent still instructs an empty lint result: {instruction}"
        );
    }
}

// behavior-2: It names the first finding that may remain: a Check that this change adds has no marked test
// behavior-2: It names the second: a marker names a Check that this change removes
// behavior-2: It says that every other finding and every reading error is gone before the stop
// behavior-2: It tells the agent to list the findings that remain when it stops
#[test]
fn intent_names_exactly_the_two_findings_that_may_remain_at_the_stop() {
    let repo = Repo::sample();
    install(&repo, "codex");
    let step = step_text(&digest_line_is_correct(&repo, SKILL_FILE), 2);
    let lower = step
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let permission = [
        "may remain",
        "may still report",
        "may report",
        "can still report",
        "can remain",
        "are allowed",
        "is allowed",
        "permitted",
        "acceptable",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase));
    let exactly_two = (lower.contains("two") || lower.split_whitespace().any(|word| word == "2"))
        && ["only", "no other", "no others", "exactly"]
            .iter()
            .any(|word| lower.contains(word));
    assert!(
        permission && exactly_two,
        "Intent does not allow exactly two findings: {step}"
    );
    let clauses: Vec<_> = lower
        .split(['.', ';'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let added = |text: &str| {
        text.contains("check")
            && text.contains("add")
            && text.contains("change")
            && (((text.contains("no")
                || text.contains("without")
                || text.contains("lack")
                || text.contains("missing"))
                && text.contains("mark")
                && text.contains("test"))
                || text.contains("without a marked test")
                || text.contains("missing a marked test"))
    };
    let removed = |text: &str| {
        text.contains("marker")
            && text.contains("check")
            && (text.contains("remov") || text.contains("delet"))
            && text.contains("change")
    };
    let normalized = step.lines().fold(String::new(), |mut text, line| {
        if line.trim_start().starts_with(['-', '*']) {
            text.push(';');
        }
        text.push_str(line);
        text.push(' ');
        text
    });
    let statements: Vec<_> = normalized
        .split(['.', ';'])
        .map(|part| {
            part.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        })
        .collect();
    for describes in [added as fn(&str) -> bool, removed as fn(&str) -> bool] {
        assert!(
            statements.iter().any(|text| describes(text)
                && ![
                    "do not allow",
                    "not allowed",
                    "must fix",
                    "fix a check",
                    "fix the marker"
                ]
                .iter()
                .any(|negative| text.contains(negative))),
            "Intent does not permit one of the two findings: {step}"
        );
    }
    for statement in &statements {
        let permits = [
            "may remain",
            "may still report",
            "may report",
            "can still report",
            "can remain",
            "allowed",
            "permitted",
            "acceptable",
        ]
        .iter()
        .any(|word| statement.contains(word));
        let negated = ["not allowed", "do not allow", "no other", "no others"]
            .iter()
            .any(|word| statement.contains(word));
        if permits && !negated {
            assert!(
                !statement.contains("third") && !statement.contains("three findings"),
                "third permitted finding: {statement}"
            );
            let finding = statement.contains("check") || statement.contains("marker");
            if finding {
                assert!(
                    added(statement) || removed(statement),
                    "another finding is permitted: {statement}"
                );
            }
        }
    }
    let bullets: Vec<_> = step
        .lines()
        .filter(|line| line.trim_start().starts_with(['-', '*']))
        .map(|line| line.to_lowercase())
        .filter(|line| {
            line.contains("finding") || line.contains("check") || line.contains("marker")
        })
        .collect();
    if !bullets.is_empty() {
        assert_eq!(
            bullets.len(),
            2,
            "Intent permits a third listed finding: {step}"
        );
        for bullet in &bullets {
            assert!(
                added(bullet) || removed(bullet),
                "third allowed finding: {bullet}"
            );
        }
    }
    assert!(
        statements.iter().any(
            |text| (text.contains("other") || text.contains("remaining"))
                && (text.contains("finding") && text.contains("reading error"))
                && ["fix", "clear", "gone", "resolve"]
                    .iter()
                    .any(|word| text.contains(word))
                && text.contains("before")
                && text.contains("stop")
        ),
        "Intent does not clear other findings and reading errors before the stop: {step}"
    );
    assert!(
        clauses.iter().any(|text| ["list", "enumerate", "show"]
            .iter()
            .any(|word| text.contains(word))
            && (text.contains("remain") || text.contains("remaining"))
            && text.contains("stop")
            && !["do not", "don't", "never", "not list", "not show"]
                .iter()
                .any(|word| text.contains(word))),
        "Intent does not tell the agent to list what remains at the stop: {step}"
    );
}

// behavior-2: `azimuth lint` reports the same findings as before, with no option for a step
#[test]
fn lint_still_reports_both_intent_findings_and_accepts_no_step_option() {
    let repo = Repo::sample();
    repo.replace(LENDING_VERIFICATION, "- Check: returned-book-frees-a-slot\n  After one return the desk lends again to a borrower who held five loans.\n", "");
    repo.append(
        LENDING_VERIFICATION,
        "\n- Check: renewal-keeps-the-count\n  A renewal keeps the count.\n",
    );
    let findings = repo.lint_findings(1);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(
        findings_at(
            &findings,
            LOAN_LIMIT_TESTS,
            Some(repo.line_of(LOAN_LIMIT_TESTS, RETURNED_BOOK))
        )
        .len(),
        1
    );
    assert_eq!(
        findings_at(
            &findings,
            LENDING_VERIFICATION,
            Some(repo.line_of(LENDING_VERIFICATION, "renewal-keeps-the-count"))
        )
        .len(),
        1
    );
    for (file, id) in [
        (LOAN_LIMIT_TESTS, RETURNED_BOOK),
        (LENDING_VERIFICATION, "renewal-keeps-the-count"),
    ] {
        let entries = findings_at(&findings, file, None);
        assert_eq!(entries.len(), 1);
        assert!(
            entries[0]["message"].as_str().unwrap().contains(id),
            "finding names another Check: {entries:#?}"
        );
    }
    let output = repo.azimuth(&["lint", "--step", "intent", "--json"]);
    output.expect_status(2);
}

fn installed_step(number: u32) -> String {
    let repo = Repo::sample();
    install(&repo, "codex");
    step_text(&digest_line_is_correct(&repo, SKILL_FILE), number)
}

fn arrival_command(step: &str, folder: &str) -> String {
    let path = format!("azimuth/{folder}/<name>");
    let blocks: Vec<_> = step
        .split("```sh\n")
        .skip(1)
        .map(|rest| rest.split_once("```").expect("the command fence closes").0)
        .filter(|block| block.contains(&path))
        .collect();
    assert!(!blocks.is_empty(),
        "the {} command is missing from step 1: no sh block names {path}\n{step}",
        if folder == "initiatives" { "initiative" } else { "change" });
    assert_eq!(blocks.len(), 1,
        "step 1 needs exactly one sh fenced block naming {path}:\n{step}");
    assert!(blocks[0].contains("<target-branch>") && blocks[0].contains("<name>"),
        "the arrival command for {path} needs <target-branch> and <name>:\n{}", blocks[0]);
    assert!(
        blocks[0].contains("git "),
        "the arrival command does not read git: {}",
        blocks[0]
    );
    blocks[0].to_string()
}

fn date_for_git(repo: &mut Repo, date: &str) {
    repo.set_env("GIT_AUTHOR_DATE", date);
    repo.set_env("GIT_COMMITTER_DATE", date);
}

fn folder_merged_into_target(folder_file: &str) -> Repo {
    let mut repo = Repo::sample();
    repo.git(&["checkout", "--quiet", "-b", "prepared-change"]);
    repo.write(
        folder_file,
        "A receipt for a returned book.\n",
    );
    date_for_git(&mut repo, "2031-03-05T00:10:00+1400");
    repo.commit_all("prepare the receipt change");
    repo.git(&["checkout", "--quiet", "main"]);
    date_for_git(&mut repo, "2031-03-09T00:15:00+1400");
    repo.git(&[
        "merge",
        "--quiet",
        "--no-ff",
        "prepared-change",
        "-m",
        "merge the receipt change",
    ]);
    assert_eq!(
        repo.git(&["show", "-s", "--format=%cI", "HEAD"]).trim(),
        "2031-03-09T00:15:00+14:00"
    );
    // The target tip is later than the arrival, so reading only its date cannot pass.
    date_for_git(&mut repo, "2031-03-11T12:00:00+1400");
    repo.commit_all("continue work on the target");
    repo
}

fn run_arrival_command(repo: &Repo, command: &str) -> Output {
    let script = command
        .replace("<target-branch>", "main")
        .replace("<name>", "return-receipt");
    let mut shell = std::process::Command::new("sh");
    shell
        .args(["-eu", "-c", &script])
        .current_dir(repo.root())
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("LC_ALL", "C")
        .env("TZ", "Etc/GMT-14");
    let output = run_with_time_limit(shell, std::time::Duration::from_secs(10), None);
    output.expect_status(0);
    output
}

fn printed_arrival_day(repo: &Repo, command: &str) -> String {
    let output = run_arrival_command(repo, command);
    output.stdout.trim().to_string()
}

// behavior-2-follow-up: For a folder that arrived by a merge, the command of the skill prints the day of the merge on the target branch, and not the day of the commit on the change branch
#[test]
fn the_skill_command_prints_the_target_merge_day_in_utc() {
    let repo = folder_merged_into_target("azimuth/changes/return-receipt/proposal.md");
    repo.git(&["checkout", "--quiet", "-b", "next-change"]);
    let command = arrival_command(&installed_step(1), "changes");
    assert_eq!(printed_arrival_day(&repo, &command), "2031-03-08",
        "the skill command must print the target merge day in UTC, not the change commit day, the local merge day or the target tip day");
}

// behavior-2-follow-up: Run on two branches that start from the same target branch, the command prints the same day
#[test]
fn the_skill_command_prints_the_same_day_on_two_change_branches() {
    let mut repo = folder_merged_into_target("azimuth/changes/return-receipt/proposal.md");
    repo.git(&["checkout", "--quiet", "-b", "next-change-a", "main"]);
    date_for_git(&mut repo, "2031-03-12T12:00:00+1400");
    repo.commit_all("start the first change");
    repo.git(&["checkout", "--quiet", "-b", "next-change-b", "main"]);
    date_for_git(&mut repo, "2031-03-15T12:00:00+1400");
    repo.commit_all("start the second change");
    let command = arrival_command(&installed_step(1), "changes");
    let second = printed_arrival_day(&repo, &command);
    repo.git(&["checkout", "--quiet", "next-change-a"]);
    let first = printed_arrival_day(&repo, &command);
    assert_eq!(
        first, second,
        "branches from the same target disagree about the arrival day"
    );
    assert_eq!(
        first, "2031-03-08",
        "both branches must print the UTC arrival day"
    );
}

// behavior-2-follow-up: For a folder that arrived by a fast-forward, the command prints the day of the commit that first brought the folder
#[test]
fn the_skill_command_prints_the_first_folder_commit_day_after_fast_forward() {
    let mut repo = Repo::sample();
    repo.git(&["checkout", "--quiet", "-b", "prepared-change"]);
    let folder = "azimuth/changes/return-receipt/proposal.md";
    repo.write(folder, "A receipt for a returned book.\n");
    date_for_git(&mut repo, "2031-03-05T00:10:00+1400");
    repo.commit_all("prepare the receipt change");
    // A later edit inside the folder separates first introduction from its latest change.
    repo.append(folder, "Keep the receipt at the desk.\n");
    date_for_git(&mut repo, "2031-03-07T12:00:00+1400");
    repo.commit_all("complete the receipt proposal");
    let change_tip = repo.head();
    repo.git(&["checkout", "--quiet", "main"]);
    date_for_git(&mut repo, "2031-03-09T12:00:00+1400");
    repo.git(&["merge", "--quiet", "--ff-only", "prepared-change"]);
    assert_eq!(
        repo.head(),
        change_tip,
        "the arrival must be a fast-forward"
    );
    date_for_git(&mut repo, "2031-03-11T12:00:00+1400");
    repo.commit_all("continue work on the target");
    repo.git(&["checkout", "--quiet", "-b", "next-change"]);
    assert_eq!(
        printed_arrival_day(&repo, &arrival_command(&installed_step(1), "changes")),
        "2031-03-04",
        "use the first folder commit in UTC, even when arrival left no merge commit"
    );
}

// behavior-2-initiative-skill: For an initiative folder that arrived by a merge, the command prints the day of the merge on the target branch, and not the day of a commit on the change branch
#[test]
fn the_initiative_command_prints_the_target_merge_day_in_utc() {
    let mut repo = folder_merged_into_target("azimuth/initiatives/return-receipt/map.md");
    // A same-named change arrived on a different day, so the wrong folder cannot pass.
    repo.write("azimuth/changes/return-receipt/proposal.md", "A receipt change.\n");
    date_for_git(&mut repo, "2031-03-13T12:00:00+1400");
    repo.commit_all("a separate change with the initiative's name");
    repo.git(&["checkout", "--quiet", "-b", "next-change"]);
    let command = arrival_command(&installed_step(1), "initiatives");
    assert_eq!(printed_arrival_day(&repo, &command), "2031-03-08",
        "use the initiative's target merge day in UTC, not its branch commit, local merge day, target tip or same-named change");
}

// behavior-2-initiative-skill: For an initiative folder that arrived by a fast-forward, the command prints the day of the commit that first brought the folder
#[test]
fn the_initiative_command_prints_the_first_folder_commit_day_after_fast_forward() {
    let mut repo = Repo::sample();
    repo.git(&["checkout", "--quiet", "-b", "prepared-initiative"]);
    let folder = "azimuth/initiatives/return-receipt/map.md";
    repo.write(folder, "A map of receipt work.\n");
    date_for_git(&mut repo, "2031-03-05T00:10:00+1400");
    repo.commit_all("prepare the receipt initiative");
    // A later edit separates first introduction from the folder's most recent commit.
    repo.append(folder, "Another row of receipt work.\n");
    date_for_git(&mut repo, "2031-03-07T12:00:00+1400");
    repo.commit_all("extend the receipt initiative");
    let initiative_tip = repo.head();
    repo.git(&["checkout", "--quiet", "main"]);
    date_for_git(&mut repo, "2031-03-09T12:00:00+1400");
    repo.git(&["merge", "--quiet", "--ff-only", "prepared-initiative"]);
    assert_eq!(repo.head(), initiative_tip, "the arrival must be a fast-forward");
    repo.write("azimuth/changes/return-receipt/proposal.md", "A separate receipt change.\n");
    date_for_git(&mut repo, "2031-03-11T12:00:00+1400");
    repo.commit_all("continue work on the target with a same-named change");
    repo.git(&["checkout", "--quiet", "-b", "next-change"]);
    let command = arrival_command(&installed_step(1), "initiatives");
    assert_eq!(printed_arrival_day(&repo, &command), "2031-03-04",
        "use the initiative's first folder commit in UTC, even when arrival left no merge commit");
}

// behavior-2-initiative-skill: For an initiative folder that is not on the target branch, the command prints nothing
#[test]
fn the_initiative_command_prints_nothing_for_a_folder_absent_from_the_target() {
    let mut repo = Repo::sample();
    repo.write("azimuth/changes/return-receipt/proposal.md", "A separate receipt change.\n");
    date_for_git(&mut repo, "2031-03-05T00:10:00+1400");
    repo.commit_all("a same-named change on the target");
    repo.git(&["checkout", "--quiet", "-b", "open-initiative"]);
    repo.write("azimuth/initiatives/return-receipt/map.md", "An initiative only on this branch.\n");
    date_for_git(&mut repo, "2031-03-07T12:00:00+1400");
    repo.commit_all("an initiative not yet on the target");
    let command = arrival_command(&installed_step(1), "initiatives");
    let output = run_arrival_command(&repo, &command);
    output.expect_status(0);
    assert!(output.stdout.is_empty(), "an initiative absent from the target printed output: {:?}", output.stdout);
    assert!(output.stderr.is_empty(), "an initiative absent from the target printed errors: {:?}", output.stderr);
}
