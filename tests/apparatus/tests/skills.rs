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
