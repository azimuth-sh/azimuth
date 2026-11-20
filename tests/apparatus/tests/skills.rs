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

fn installed_step(number: u32) -> String {
    let repo = Repo::sample();
    install(&repo, "codex");
    step_text(&digest_line_is_correct(&repo, SKILL_FILE), number)
}

fn prose(text: &str) -> String {
    text.replace(['`', '*'], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn contains_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

fn sentences(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    for paragraph in text.split("\n\n") {
        let normalized = prose(paragraph);
        let mut start = 0;
        for (at, c) in normalized.char_indices() {
            // A period inside a filename must not hide the condition after that filename.
            if matches!(c, '.' | '?' | '!')
                && normalized[at + 1..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
            {
                result.push(normalized[start..at].trim().to_string());
                start = at + 1;
            }
        }
        if !normalized[start..].trim().is_empty() {
            result.push(normalized[start..].trim().to_string());
        }
    }
    result
}

fn rule_sentence(text: &str, rule: &str, holds: impl Fn(&str) -> bool) -> String {
    sentences(text)
        .into_iter()
        .find(|s| holds(s))
        .unwrap_or_else(|| panic!("no sentence says {rule}:\n{text}"))
}

fn assert_affirmative(sentence: &str) {
    let words: Vec<_> = sentence
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .collect();
    for reversal in [
        "never", "not", "except", "unless", "no", "don't", "cannot", "can't", "without",
    ] {
        assert!(
            !words.contains(&reversal),
            "{reversal:?} reverses the rule: {sentence}"
        );
    }
}

fn positive_rule_in_sentence(number: u32, rule: &str, holds: impl Fn(&str) -> bool) {
    let sentence = rule_sentence(&installed_step(number), rule, holds);
    assert_affirmative(&sentence);
}

fn assert_plan_question_is_asked(question: &str) {
    let step = installed_step(2).split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(step.contains(question), "step 2 lacks the exact question {question:?}:\n{step}");
    let instruction = rule_sentence(&step, "ask the plan questions", |s| {
        contains_any(s, &["ask:", "ask the", "ask these", "ask each"])
    });
    assert_affirmative(&instruction);
    let normalized = prose(question);
    let sentence = rule_sentence(&step, question, |s| s.contains(normalized.trim_end_matches('?')));
    assert_affirmative(&sentence.replace("no longer", ""));
}

fn arrival_command(step: &str) -> String {
    let blocks: Vec<_> = step
        .split("```sh\n")
        .skip(1)
        .map(|rest| rest.split_once("```").expect("the command fence closes").0)
        .filter(|block| block.contains("<target-branch>") && block.contains("<name>"))
        .collect();
    assert_eq!(blocks.len(), 1,
        "step 1 needs one sh fenced block with <target-branch> and <name> that prints the arrival day:\n{step}");
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

fn folder_merged_into_target() -> Repo {
    let mut repo = Repo::sample();
    repo.git(&["checkout", "--quiet", "-b", "prepared-change"]);
    repo.write(
        "azimuth/changes/return-receipt/proposal.md",
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

fn printed_arrival_day(repo: &Repo, command: &str) -> String {
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
    output.stdout.trim().to_string()
}

// behavior-2-follow-up: Step 1 names the moved folder `<date>-<name>`
#[test]
fn step_1_names_finished_folders_with_date_and_name() {
    positive_rule_in_sentence(1, "the moved folder is named <date>-<name>", |s| {
        s.contains("<date>-<name>") && s.contains("azimuth/changes/finished/")
    });
}

// behavior-2-follow-up: It says which day the date is: the day on which the folder arrived on the target branch
#[test]
fn step_1_uses_the_day_the_folder_arrived_on_the_target() {
    positive_rule_in_sentence(
        1,
        "the date is the day the folder arrived on the target branch",
        |s| {
            s.contains("date")
                && s.contains("day")
                && s.contains("target branch")
                && contains_any(s, &["arriv", "reach", "merge"])
        },
    );
}

/// The command needs a stable place so these tests run the skill's command, not a copy.
/// Step 1 has exactly one `sh` fenced block containing both `<target-branch>` and `<name>`.
/// Its body runs as `sh -eu` from the repository root after literal substitution. It prints
/// only YYYY-MM-DD on stdout. The executable tests use that same block with TZ=Etc/GMT-14.
// behavior-2-follow-up: It gives one git command that prints that day in UTC, and tells the agent not to type the date
#[test]
fn step_1_reads_the_utc_day_from_git_instead_of_typing_it() {
    let step = installed_step(1);
    arrival_command(&step);
    let reading = rule_sentence(
        &step,
        "read the date from git in UTC and do not type it",
        |s| {
            s.contains("git")
                && s.contains("utc")
                && s.contains("date")
                && contains_any(s, &["read", "use"])
                && contains_any(
                    s,
                    &[
                        "do not type",
                        "never type",
                        "nobody types",
                        "don't type",
                        "do not enter",
                        "never enter",
                    ],
                )
        },
    );
    assert_affirmative(
        &reading
            .replace("do not type", "")
            .replace("never type", "")
            .replace("nobody types", "")
            .replace("don't type", "")
            .replace("do not enter", "")
            .replace("never enter", ""),
    );
}

// behavior-2-follow-up: For a folder that arrived by a merge, the command of the skill prints the day of the merge on the target branch, and not the day of the commit on the change branch
#[test]
fn the_skill_command_prints_the_target_merge_day_in_utc() {
    let repo = folder_merged_into_target();
    repo.git(&["checkout", "--quiet", "-b", "next-change"]);
    let command = arrival_command(&installed_step(1));
    assert_eq!(printed_arrival_day(&repo, &command), "2031-03-08",
        "the skill command must print the target merge day in UTC, not the change commit day, the local merge day or the target tip day");
}

// behavior-2-follow-up: Run on two branches that start from the same target branch, the command prints the same day
#[test]
fn the_skill_command_prints_the_same_day_on_two_change_branches() {
    let mut repo = folder_merged_into_target();
    repo.git(&["checkout", "--quiet", "-b", "next-change-a", "main"]);
    date_for_git(&mut repo, "2031-03-12T12:00:00+1400");
    repo.commit_all("start the first change");
    repo.git(&["checkout", "--quiet", "-b", "next-change-b", "main"]);
    date_for_git(&mut repo, "2031-03-15T12:00:00+1400");
    repo.commit_all("start the second change");
    let command = arrival_command(&installed_step(1));
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
        printed_arrival_day(&repo, &arrival_command(&installed_step(1))),
        "2031-03-04",
        "use the first folder commit in UTC, even when arrival left no merge commit"
    );
}

// behavior-2-follow-up: Step 1 tells the agent to read the day in a clone that has the whole history of the target branch
#[test]
fn step_1_requires_the_whole_target_history_for_the_day() {
    positive_rule_in_sentence(
        1,
        "read the day in a clone with the whole target branch history",
        |s| {
            s.contains("clone")
                && s.contains("target branch")
                && s.contains("history")
                && contains_any(s, &["whole", "full", "complete"])
                && contains_any(s, &["day", "date"])
                && contains_any(s, &["read", "use"])
        },
    );
}

// behavior-2-follow-up: Step 1 says where the target branch is named: in `azimuth/project.toml`
#[test]
fn step_1_names_the_project_file_as_the_source_of_the_target_branch() {
    rule_sentence(&installed_step(1), "the target branch is named in azimuth/project.toml", |s| {
        s.contains("target branch") && s.contains("azimuth/project.toml")
            && contains_any(s, &["named", "declared", "read", "find", "configured", "set", "specified"])
    });
}

// behavior-2-follow-up: Step 1 says that the change gets a name that no change used before
#[test]
fn step_1_requires_a_change_name_that_has_never_been_used() {
    rule_sentence(&installed_step(1), "give the change a name no change used before", |s| {
        s.contains("change") && s.contains("name")
            && contains_any(s, &["never used", "never been used", "no change used", "no earlier change",
                "unused", "unique", "not used before", "not been used", "never reuse", "do not reuse"])
    });
}

// The five definitions may share a sentence or a table row group. Their own negative
// phrases describe the content, rather than reversing an instruction about a part.
fn proposal_part_sentence(rule: &str, holds: impl Fn(&str) -> bool) {
    let sentence = rule_sentence(&installed_step(1), rule, holds);
    assert_affirmative(&sentence
        .replace("cannot now", "").replace("can't now", "")
        .replace("could not before", "").replace("couldn't before", "")
        .replace("not possible now", "").replace("does not include", ""));
}

// behavior-2-follow-up: Step 1 says what the Outcome holds: what somebody can do or observe after the change that they cannot now, in their words
#[test]
fn step_1_defines_the_outcome_in_the_persons_words() {
    proposal_part_sentence("Outcome holds the new ability or observation in their words", |s| {
        s.contains("outcome")
            && contains_any(s, &["somebody", "someone", "person", "user", "they"])
            && contains_any(s, &["do", "observ", "see"])
            && s.contains("after") && s.contains("change")
            && contains_any(s, &["cannot now", "can't now", "could not before", "couldn't before", "not possible now"])
            && contains_any(s, &["their words", "their own words", "person's words", "user's words"])
    });
}

// behavior-2-follow-up: It says what the Sources hold: the sources and decision notes that the proposal relies on, and which part of each
#[test]
fn step_1_defines_the_sources_and_the_relied_on_parts() {
    proposal_part_sentence("Sources names the sources, decision notes and the part of each relied on", |s| {
        s.contains("sources") && s.contains("decision notes")
            && contains_any(s, &["relies", "rely", "uses", "used", "draws", "based on"])
            && contains_any(s, &["part", "section", "passage"])
            && contains_any(s, &["each", "every"])
    });
}

// behavior-2-follow-up: It says what the Boundary holds: what the change leaves out, each item with the name of its map row
#[test]
fn step_1_defines_the_boundary_with_a_map_row_for_each_item() {
    proposal_part_sentence("Boundary names what is left out and each item's map row", |s| {
        s.contains("boundary")
            && contains_any(s, &["leaves out", "left out", "exclude", "out of scope", "does not include"])
            && s.contains("map") && s.contains("row") && s.contains("name")
            && contains_any(s, &["each", "every"])
    });
}

// behavior-2-follow-up: It says what the Decisions hold: what this change settles, each with the alternative it rejects, one line each
#[test]
fn step_1_defines_the_decisions_with_rejected_alternatives_one_line_each() {
    proposal_part_sentence("Decisions gives what the change settles and the rejected alternative, one line each", |s| {
        s.contains("decisions") && s.contains("change")
            && contains_any(s, &["settle", "decide", "resolve"])
            && s.contains("alternative") && contains_any(s, &["reject", "discard"])
            && contains_any(s, &["each", "every", "per decision"])
            && contains_any(s, &["one line", "single line"])
    });
}

// behavior-2-follow-up: It says what the Approver is: who is expected to approve the Scenarios
#[test]
fn step_1_defines_the_approver_of_the_scenarios() {
    proposal_part_sentence("Approver names who is expected to approve the Scenarios", |s| {
        s.contains("approver") && s.contains("scenario")
            && contains_any(s, &["who", "person"])
            && contains_any(s, &["approve", "approval"])
    });
}

// behavior-2-follow-up: It says that a proposal holds no Scenarios, no plan and no completion conditions
#[test]
fn step_1_excludes_scenarios_plans_and_completion_conditions_from_the_proposal() {
    let step = installed_step(1);
    for content in ["scenario", "plan", "completion condition"] {
        let sentence = rule_sentence(&step, &format!("the proposal excludes {content}"), |s| {
            contains_any(s, &["proposal", "it holds", "it contains"])
                && s.contains(content)
                && contains_any(s, &["no ", "not hold", "not contain", "not include", "exclude", "leave out", "keep out", "never include"])
        });
        assert_affirmative(&sentence
            .replace("no ", "").replace("not hold", "").replace("not contain", "")
            .replace("not include", "").replace("never include", ""));
    }
}

// behavior-2-follow-up: It says that a proposal that does not fit on one page, or has two outcomes, is several changes
#[test]
fn step_1_splits_a_proposal_that_exceeds_one_page_or_has_two_outcomes() {
    let step = installed_step(1);
    for trigger in ["page", "outcomes"] {
        let sentence = rule_sentence(&step, &format!("a proposal with excess {trigger} is several changes"), |s| {
            s.contains("proposal")
                && contains_any(s, &["several changes", "multiple changes", "split", "separate changes"])
                && if trigger == "page" {
                    contains_any(s, &["one page", "single page"])
                        && contains_any(s, &["does not fit", "doesn't fit", "cannot fit", "can't fit", "more than", "exceed", "longer than", "beyond"])
                } else {
                    contains_any(s, &["two outcomes", "multiple outcomes", "more than one outcome"])
                }
        });
        assert_affirmative(&sentence
            .replace("does not fit", "").replace("doesn't fit", "")
            .replace("cannot fit", "").replace("can't fit", ""));
    }
}

// behavior-2-follow-up: It says that a proposal is not edited after approval
#[test]
fn step_1_keeps_the_proposal_unchanged_after_approval() {
    let sentence = rule_sentence(&installed_step(1), "the proposal is not edited after approval", |s| {
        s.contains("proposal") && contains_any(s, &["after approval", "after it is approved", "once approved", "approved proposal"])
            && (contains_any(s, &["unchanged", "immutable", "frozen"])
                || (contains_any(s, &["edit", "rewrite", "alter", "change"])
                    && contains_any(s, &["not", "never", "cannot", "can't", "don't"])))
    });
    assert_affirmative(&sentence
        .replace("cannot", "").replace("can't", "").replace("don't", "")
        .replace("not", "").replace("never", ""));
}

// behavior-2-follow-up: It says that a change of an exploration is written from its row of the map, and takes its name from the row
#[test]
fn step_1_uses_the_exploration_map_row_for_the_proposal_and_change_name() {
    let step = installed_step(1);
    let source = rule_sentence(&step, "a change of an exploration is written from its map row", |s| {
        s.contains("exploration") && s.contains("map") && s.contains("row")
            && contains_any(s, &["change", "proposal"])
            && contains_any(s, &["from", "read", "use", "follow", "based on"])
    });
    let name = rule_sentence(&step, "the map row supplies the change name", |s| {
        s.contains("row") && s.contains("name")
            && contains_any(s, &["change its name", "change's name", "change’s name", "change name",
                "name of the change", "name the change", "name from", "name comes from",
                "name in", "name on", "name for", "row's name", "row’s name"])
            && contains_any(s, &["gives", "supplies", "provides", "take", "use", "from", "named"])
    });
    // Taking the map's name and requiring an unused name may be said together.
    for sentence in [&source, &name] {
        assert_affirmative(&sentence
            .replace("never been used", "").replace("never used", "")
            .replace("no change used", "").replace("no earlier change", "")
            .replace("not been used", "").replace("not used before", "")
            .replace("never reuse", "").replace("do not reuse", "")
            .replace("not the whole folder", "").replace("not the whole exploration folder", ""));
    }
    // A row reference must not also send the agent to read the exploration wholesale.
    for sentence in sentences(&step).iter().filter(|s| {
        s.contains("read") && s.contains("folder")
            && contains_any(s, &["whole", "entire", "all of"])
    }) {
        assert!(contains_any(sentence, &["do not read", "does not read", "never read", "don't read",
            "not the whole", "not the entire", "not whole", "not entire", "instead of reading", "rather than reading"]),
            "step 1 tells the agent to read the whole exploration folder: {sentence}");
    }
}

// behavior-2-follow-up: It says that a change with no exploration is written from the request of the person, and from `sources/` in its own folder when it has one
#[test]
fn step_1_uses_the_persons_request_and_own_sources_without_an_exploration() {
    let step = installed_step(1);
    let request = rule_sentence(&step, "without an exploration use the person's request", |s| {
        s.contains("exploration")
            && contains_any(s, &["no exploration", "without an exploration", "has no exploration", "not part of an exploration", "outside an exploration"])
            && s.contains("request") && contains_any(s, &["person", "user"])
            && contains_any(s, &["from", "use", "based on"])
    });
    assert_affirmative(&request
        .replace("no exploration", "").replace("without an exploration", "")
        .replace("not part of an exploration", ""));
    let sources = rule_sentence(&step, "use sources/ in the change's own folder when present", |s| {
        s.contains("sources/") && s.contains("folder")
            && contains_any(s, &["own", "change's", "change’s", "change folder", "folder of the change"])
            && contains_any(s, &["when", "if", "where present", "where available"])
            && contains_any(s, &["from", "read", "use", "based on"])
    });
    assert_affirmative(&sources
        .replace("no exploration", "").replace("without an exploration", "")
        .replace("not part of an exploration", ""));
}

// behavior-2-follow-up: It says that the agent asks the person for what it cannot find there, and does not invent it
#[test]
fn step_1_asks_the_person_for_missing_information_and_does_not_invent_it() {
    let step = installed_step(1);
    let asking = rule_sentence(&step, "ask the person for what cannot be found in the inputs", |s| {
        s.contains("ask") && contains_any(s, &["person", "user"])
            && contains_any(s, &["cannot find", "can't find", "not find", "missing", "absent", "unavailable"])
    });
    assert_affirmative(&asking
        .replace("cannot find", "").replace("can't find", "").replace("not find", "")
        .replace("does not invent", "").replace("do not invent", "")
        .replace("never invent", "").replace("don't invent", ""));
    let invention = rule_sentence(&step, "do not invent missing information", |s| {
        contains_any(s, &["does not invent", "do not invent", "never invent", "don't invent"])
    });
    assert_affirmative(&invention
        .replace("does not invent", "").replace("do not invent", "")
        .replace("never invent", "").replace("don't invent", "")
        .replace("cannot find", "").replace("can't find", "").replace("not find", ""));
}

// behavior-2-follow-up: Step 2 says that a Requirement and its Scenarios speak about the system as somebody outside it meets it
#[test]
fn step_2_keeps_requirements_and_scenarios_outside_the_system() {
    positive_rule_in_sentence(
        2,
        "Requirements and Scenarios speak from outside the system",
        |s| {
            s.contains("requirement")
                && s.contains("scenario")
                && s.contains("system")
                && contains_any(s, &["outside", "external"])
                && contains_any(s, &["meet", "observ", "see", "experience"])
        },
    );
}

// behavior-2-follow-up: Step 2 says that a Check may be made of a test through a door of the system and of tests of one part in isolation
#[test]
fn step_2_allows_a_door_test_and_isolated_tests_in_one_check() {
    positive_rule_in_sentence(
        2,
        "one Check may combine a door test and tests of a part in isolation",
        |s| {
            s.contains("check")
                && s.contains("door")
                && s.contains("part")
                && contains_any(s, &["isolat", "unit test"])
                && contains_any(s, &["may", "can", "combine"])
        },
    );
}

// behavior-2-follow-up: It says that every part carries the marker of the Check
#[test]
fn step_2_marks_every_part_for_the_check() {
    let sentence = rule_sentence(&installed_step(2), "every part carries the marker of its Check", |s| {
        s.contains("marker") && s.contains("check")
            && ((s.contains("part")
                && contains_any(s, &["carries", "carry", "marked with"])
                && contains_any(s, &["every", "each", "all"]))
                || (s.contains("no part") && s.contains("omit")))
    });
    assert_affirmative(&sentence.replace("no part", ""));
}

// behavior-2-follow-up: It says that a Check is never made only of tests of one part in isolation
#[test]
fn step_2_forbids_a_check_made_only_of_isolated_tests() {
    let sentence = rule_sentence(
        &installed_step(2),
        "a Check is never made only of isolated tests",
        |s| {
            s.contains("check")
                && contains_any(s, &["isolat", "unit test"])
                && contains_any(s, &["never", "must not", "cannot", "can't", "do not"])
                && contains_any(s, &["only", "alone"])
        },
    );
    assert_affirmative(
        &sentence
            .replace("never", "")
            .replace("must not", "")
            .replace("cannot", "")
            .replace("can't", "")
            .replace("do not", ""),
    );
}

// behavior-2-follow-up: It says that a test through a door is needed once for each place where the part is wired
#[test]
fn step_2_requires_a_door_test_for_each_wiring_place() {
    let sentence = rule_sentence(
        &installed_step(2),
        "a door test is needed once per place the part is wired",
        |s| {
            s.contains("door")
                && s.contains("test")
                && s.contains("part")
                && s.contains("wired")
                && contains_any(s, &["each place", "every place"])
                && contains_any(s, &["once", "one test"])
                && contains_any(s, &["need", "require", "write"])
        },
    );
    assert_affirmative(&sentence.replace("not once for a requirement", ""));
}

// behavior-2-follow-up: It gives the order of the claim of such a Check: the door, what each part shows, and last what is not shown
#[test]
fn step_2_orders_the_claim_door_parts_then_limits() {
    let sentence = rule_sentence(
        &installed_step(2),
        "the claim orders the door, what each part shows, and last what is not shown",
        |s| {
            let door = s.find("door");
            let parts = s.find("each part").or_else(|| s.find("every part"));
            let limit = s.find("not shown").or_else(|| s.find("not show"));
            s.contains("claim")
                && contains_any(s, &["order", "first"])
                && contains_any(s, &["last", "finally"])
                && matches!((door, parts, limit), (Some(d), Some(p), Some(l)) if d < p && p < l)
        },
    );
    assert_affirmative(&sentence.replace("not shown", "").replace("not show", ""));
}

// behavior-2-follow-up: It says that a journey, one test through several real operations, may be a part of several Checks and carries the marker of each Check that relies on it
#[test]
fn step_2_allows_a_journey_to_carry_each_checks_marker() {
    let step = installed_step(2);
    let definition = rule_sentence(
        &step,
        "a journey is one test through several real operations",
        |s| {
            s.contains("journey")
                && contains_any(s, &["one test", "single test"])
                && contains_any(
                    s,
                    &[
                        "several real operations",
                        "multiple real operations",
                        "more than one real operation",
                    ],
                )
        },
    );
    assert_affirmative(&definition);
    let sharing = rule_sentence(&step, "a journey may be part of several Checks", |s| {
        s.contains("part")
            && contains_any(
                s,
                &["several checks", "multiple checks", "more than one check"],
            )
            && contains_any(s, &["may", "can"])
    });
    assert_affirmative(&sharing);
    let markers = rule_sentence(
        &step,
        "every Check that relies on the journey carries its marker",
        |s| {
            s.contains("marker")
                && contains_any(s, &["every check", "each check", "all checks"])
                && contains_any(s, &["relies on", "rely on", "depends on"])
        },
    );
    assert_affirmative(&markers);
    assert!(
        !markers.contains("whose scenario"),
        "the marker still depends on the Scenario: {markers}"
    );
}

// behavior-2-follow-up: It says that the claim of each such Check names the stretch of the journey that it relies on
#[test]
fn step_2_names_the_relied_on_stretch_in_each_journey_checks_claim() {
    positive_rule_in_sentence(
        2,
        "each Check's claim names the stretch of the journey it relies on",
        |s| {
            s.contains("journey")
                && s.contains("claim")
                && contains_any(
                    s,
                    &[
                        "each check",
                        "every check",
                        "each such check",
                        "every such check",
                    ],
                )
                && contains_any(s, &["stretch", "portion", "segment"])
                && contains_any(s, &["relies on", "rely on", "depends on"])
                && contains_any(s, &["name", "state", "say", "describe"])
        },
    );
}

// behavior-2-follow-up: It says that a Check also has a test of one operation alone, where that operation can be tested alone
#[test]
fn step_2_adds_an_operation_test_where_it_can_run_alone() {
    positive_rule_in_sentence(
        2,
        "where an operation can be tested alone, the Check also has a test of that operation alone",
        |s| {
            s.contains("check")
                && s.contains("operation")
                && s.contains("test")
                && contains_any(s, &["alone", "isolat"])
                && contains_any(s, &["where", "when", "if"])
                && contains_any(s, &["can be tested", "can test", "testable"])
                && contains_any(
                    s,
                    &[
                        "also has",
                        "also needs",
                        "also add",
                        "also write",
                        "must also",
                        "add a test",
                        "include a test",
                    ],
                )
        },
    );
}

// behavior-2-follow-up: It says that when the change removes a Check, the marker leaves, and its test leaves only when the test carries no other marker
#[test]
fn removing_a_check_keeps_a_test_that_carries_another_marker() {
    let intent = installed_step(2);
    let build = installed_step(3);
    let steps = format!("{intent}\n\n{build}");
    let marker = rule_sentence(&steps, "remove the Check's marker in the Build", |s| {
        s.contains("marker")
            && contains_any(s, &["remov", "delet", "leaves", "leave"])
            && (s.contains("build") || sentences(&build).iter().any(|b| b == s))
    });
    assert_affirmative(&marker.replace("no other", "").replace("no remaining", ""));
    let guarded = |s: &str| {
        s.contains("test")
            && contains_any(s, &["remov", "delet", "leaves", "leave"])
            && contains_any(s, &["only when", "only if"])
            && contains_any(
                s,
                &["no other marker", "no remaining marker", "carries no other"],
            )
    };
    let test = rule_sentence(
        &steps,
        "remove the test only when it carries no other marker",
        guarded,
    );
    assert_affirmative(&test.replace("no other", "").replace("no remaining", ""));
    for sentence in sentences(&steps).iter().filter(|s| {
        contains_any(
            s,
            &[
                "remove its test",
                "remove the test",
                "remove a test",
                "remove tests",
                "delete its test",
                "delete the test",
                "delete a test",
                "delete tests",
                "test leaves",
                "tests leave",
                "test is removed",
                "tests are removed",
                "test is deleted",
                "tests are deleted",
                "test is written or removed",
            ],
        )
    }) {
        assert!(guarded(sentence), "unconditional test removal: {sentence}");
    }
}

// behavior-2-follow-up: It says what a Check is and where it stands: one claim about its Scenario, under that Scenario in `verification.md`
#[test]
fn step_2_defines_a_check_and_places_it_under_its_scenario() {
    let step = installed_step(2);
    rule_sentence(&step, "a Check states one claim about its Scenario", |s| {
        s.contains("check") && s.contains("scenario")
            && contains_any(s, &["one claim", "a single claim"])
    });
    rule_sentence(&step, "declare the Check under its Scenario in verification.md", |s| {
        s.contains("check") && s.contains("scenario") && s.contains("verification.md")
            && contains_any(s, &["under", "beneath", "below"])
    });
}

// behavior-2-follow-up: It says that a Check belongs to one suite, and that evidence from another suite is another Check under the Scenario
#[test]
fn step_2_uses_one_check_per_suite_under_the_scenario() {
    let step = installed_step(2);
    rule_sentence(&step, "a Check belongs to one suite", |s| {
        s.contains("check") && contains_any(s, &["one suite", "a single suite"])
    });
    rule_sentence(&step, "evidence from another suite is another Check under the Scenario", |s| {
        s.contains("scenario") && s.contains("suite")
            && contains_any(s, &["another check", "separate check", "two checks", "different check"])
            && contains_any(s, &["another suite", "two suites", "different suite", "each suite"])
    });
}

// behavior-2-follow-up: It names the file of the plan, `azimuth/changes/<name>/plan.md`
#[test]
fn step_2_names_the_plan_file_in_the_change_folder() {
    positive_rule_in_sentence(2, "the plan is azimuth/changes/<name>/plan.md", |s| {
        s.contains("plan") && s.contains("azimuth/changes/<name>/plan.md")
    });
}

// behavior-2-follow-up: Step 2 says that the change needs a plan when a Requirement that it adds or changes declares `Realization:`
#[test]
fn step_2_requires_a_plan_for_an_added_or_changed_realization_requirement() {
    positive_rule_in_sentence(
        2,
        "a plan is needed when an added or changed Requirement declares Realization:",
        |s| {
            s.contains("plan")
                && s.contains("requirement")
                && s.contains("realization:")
                && s.contains("add")
                && contains_any(s, &["chang", "alter"])
                && contains_any(s, &["need", "require", "write"])
        },
    );
}

// behavior-2-follow-up: It says that a change with no such Requirement needs no plan
#[test]
fn step_2_needs_no_plan_without_such_a_requirement() {
    let sentence = rule_sentence(
        &installed_step(2),
        "no such Requirement means no plan",
        |s| {
            s.contains("plan")
                && s.contains("requirement")
                && contains_any(s, &["no such", "without", "none"])
                && contains_any(
                    s,
                    &[
                        "no plan",
                        "needs no",
                        "need no",
                        "not need",
                        "not required",
                        "skip",
                    ],
                )
        },
    );
    assert_affirmative(
        &sentence
            .replace("no such", "")
            .replace("without", "")
            .replace("none", "")
            .replace("no plan", "")
            .replace("needs no", "")
            .replace("need no", "")
            .replace("not need", "")
            .replace("not required", ""),
    );
}

// behavior-2-follow-up: It says that the plan covers the whole change, also what is built for a Requirement that declares no `Realization:`
#[test]
fn step_2_plans_the_whole_change_including_requirements_without_realization() {
    let sentence = rule_sentence(
        &installed_step(2),
        "the whole change is planned, including Requirements without Realization:",
        |s| {
            s.contains("plan")
                && s.contains("change")
                && s.contains("requirement")
                && s.contains("realization:")
                && contains_any(s, &["whole", "entire", "all of"])
                && contains_any(s, &["also", "includ", "even"])
                && contains_any(
                    s,
                    &[
                        "no realization:",
                        "without realization:",
                        "does not declare",
                    ],
                )
        },
    );
    assert_affirmative(
        &sentence
            .replace("no realization:", "")
            .replace("without realization:", "")
            .replace("does not declare", ""),
    );
}

// behavior-2-follow-up: It puts the writing of the plan after the Scenarios and the Checks, and before the one stop of the step
#[test]
fn step_2_writes_the_plan_after_scenarios_and_checks_before_its_only_stop() {
    let step = installed_step(2);
    let order = rule_sentence(
        &step,
        "write the plan after the Scenarios and Checks",
        |s| {
            s.contains("plan")
                && contains_any(s, &["write", "draft"])
                && s.contains("scenario")
                && s.contains("check")
                && s.contains("after")
        },
    );
    assert_affirmative(&order);
    let after = order.find("after").unwrap();
    let scenario = order.find("scenario").unwrap();
    let check = order.find("check").unwrap();
    let plan = order.find("plan").unwrap();
    assert!(
        after < scenario && after < check && ((scenario < plan && check < plan) || plan < after),
        "the plan precedes the Scenarios or Checks: {order}"
    );
    let plan_at = step
        .split("\n\n")
        .find(|p| sentences(p).contains(&order))
        .map(|p| step.find(p).unwrap())
        .unwrap();
    let stops: Vec<_> = step.match_indices("Stop.").collect();
    assert_eq!(
        stops.len(),
        1,
        "the plan must share the one Intent stop:\n{step}"
    );
    assert!(
        plan_at < stops[0].0,
        "the plan is written after the Intent stop:\n{step}"
    );
    let stop = prose(paragraph_at(&step, stops[0].0));
    assert!(
        stop.contains("scenario") && stop.contains("check") && stop.contains("plan"),
        "the one Intent stop does not offer all three to the person:\n{step}"
    );
}

// behavior-2-follow-up: It names what the plan holds at the stop: the proposed decisions, the units to add, change or delete, and the migration steps
#[test]
fn step_2_names_the_plan_contents_at_the_stop() {
    positive_rule_in_sentence(2, "the plan at the stop holds proposed decisions, units to add, change or delete, and migration steps", |s| {
        s.contains("plan") && s.contains("proposed") && s.contains("decision")
            && s.contains("unit") && s.contains("add") && s.contains("chang")
            && contains_any(s, &["delet", "remov"]) && s.contains("migration") && s.contains("step")
    });
}

// behavior-2-follow-up: It says that what somebody outside the system can observe is a Scenario and no line of the plan, and that the addresses and the answers between the team's own components are in the plan
#[test]
fn step_2_puts_observations_in_scenarios_and_internal_contracts_in_the_plan() {
    let step = installed_step(2);
    let outside = rule_sentence(
        &step,
        "outside observations belong in a Scenario, outside the plan",
        |s| {
            contains_any(s, &["outside", "external"])
                && contains_any(s, &["observ", "see", "meet"])
                && s.contains("scenario")
                && s.contains("plan")
        },
    );
    let observation = outside
        .find("observ")
        .or_else(|| outside.find("see"))
        .or_else(|| outside.find("meet"))
        .unwrap();
    let scenario = outside.find("scenario").unwrap();
    assert!(
        observation < scenario,
        "the observation and its home are reversed: {outside}"
    );
    assert_affirmative(
        &outside
            .replace("not in the plan", "")
            .replace("no line of the plan", ""),
    );
    let exclusion = outside.contains("not in the plan")
        || outside.contains("no line of the plan")
        || sentences(&step)
            .iter()
            .any(|s| s.contains("observ") && s.contains("not in the plan"));
    assert!(
        exclusion,
        "outside observations are not excluded from the plan: {outside}"
    );
    let internal = rule_sentence(
        &step,
        "internal addresses and answers belong in the plan",
        |s| {
            s.contains("address")
                && contains_any(s, &["answer", "response"])
                && s.contains("component")
                && contains_any(s, &["own", "internal"])
                && s.contains("plan")
        },
    );
    assert_affirmative(&internal);
    assert!(
        !internal.contains("scenario"),
        "internal contracts have the wrong home: {internal}"
    );
}

// behavior-2-follow-up: It asks about data and the release before
#[test]
fn step_2_asks_about_data_and_the_release_before() {
    assert_plan_question_is_asked("What data is added or changed, and how does the release before it run beside it?");
}

// behavior-2-follow-up: It asks about the promises between components
#[test]
fn step_2_asks_about_promises_between_components() {
    assert_plan_question_is_asked("What do the team's own components promise each other?");
}

// behavior-2-follow-up: It asks who may do it
#[test]
fn step_2_asks_who_may_do_it() {
    assert_plan_question_is_asked("Who may do it, and where is that enforced?");
}

// behavior-2-follow-up: It asks how a refusal reaches the caller
#[test]
fn step_2_asks_how_a_refusal_reaches_the_caller() {
    assert_plan_question_is_asked("How does a refusal reach the caller?");
}

// behavior-2-follow-up: It asks about two requests that meet
#[test]
fn step_2_asks_about_two_requests_that_meet() {
    assert_plan_question_is_asked("What happens when two requests meet?");
}

// behavior-2-follow-up: It asks what can no longer be deleted
#[test]
fn step_2_asks_what_can_no_longer_be_deleted() {
    assert_plan_question_is_asked("What can no longer be deleted?");
}

// behavior-2-follow-up: It says that a question with an answer becomes a proposed decision, and that a question with no answer leaves no line
#[test]
fn step_2_turns_answers_into_proposed_decisions_and_leaves_no_empty_line() {
    positive_rule_in_sentence(
        2,
        "an answered question becomes a proposed decision in the plan",
        |s| {
            s.contains("question")
                && s.contains("answer")
                && s.contains("plan")
                && s.contains("proposed")
                && s.contains("decision")
                && contains_any(s, &["become", "turn"])
        },
    );
    let sentence = rule_sentence(
        &installed_step(2),
        "an unanswered question leaves no line",
        |s| {
            s.contains("question")
                && contains_any(s, &["no answer", "unanswered", "without an answer"])
                && contains_any(s, &["no line", "omit", "leave nothing", "write nothing"])
        },
    );
    assert_affirmative(
        &sentence
            .replace("no answer", "")
            .replace("without an answer", "")
            .replace("no line", ""),
    );
}

// behavior-2-follow-up: The stop of step 2 names the plan among what the person reads
#[test]
fn step_2_offers_the_plan_to_the_person_at_the_stop() {
    let step = installed_step(2);
    let stop_at = step.find("Stop.").expect("step 2 has no stop");
    let stop = prose(paragraph_at(&step, stop_at));
    assert!(
        stop.contains("plan")
            && stop.contains("person")
            && contains_any(&stop, &["read", "review"]),
        "the Intent stop does not ask the person to read the plan:\n{step}"
    );
    assert_affirmative(&stop);
}

// behavior-2-follow-up: Step 3 says what a marker is and where it stands: it enrolls a test in a Check, on a test or on a class, in the form that the suite declares in `azimuth/project.toml`
#[test]
fn step_3_defines_markers_their_placement_and_the_suite_declared_form() {
    let step = installed_step(3);
    rule_sentence(&step, "a marker enrolls a test in a Check", |s| {
        s.contains("marker") && s.contains("test") && s.contains("check")
            && contains_any(s, &["enroll", "associate", "register", "join", "link"])
    });
    rule_sentence(&step, "put a marker on a test or on a class", |s| {
        s.contains("marker") && s.contains("test") && s.contains("class")
            && contains_any(s, &["on", "attach", "place"])
    });
    rule_sentence(&step, "use the suite's marker form from azimuth/project.toml", |s| {
        s.contains("marker") && s.contains("suite") && s.contains("azimuth/project.toml")
            && contains_any(s, &["form", "pattern", "syntax"])
    });
}

// behavior-2-follow-up: Step 3 tells the agent to move a proposed decision into `realization.md` only when the finished code follows it
#[test]
fn step_3_keeps_only_decisions_the_finished_code_follows() {
    positive_rule_in_sentence(
        3,
        "a proposed decision moves to realization.md only when finished code follows it",
        |s| {
            s.contains("proposed") && contains_any(s, &["move", "keep"])
                && s.contains("decision")
                && s.contains("realization.md")
                && s.contains("code")
                && contains_any(s, &["finished", "built", "completed"])
                && contains_any(s, &["only if", "only when"])
                && contains_any(s, &["follow", "implement", "match"])
        },
    );
}

// behavior-2-follow-up: Step 3 puts the tests through a door before the code, and the tests of one part in isolation after it, written from the Scenario and not from the code
#[test]
fn step_3_writes_door_tests_before_code_and_isolated_tests_after_from_the_scenario() {
    let step = installed_step(3);
    let door = rule_sentence(&step, "write tests through a door before the code", |s| {
        s.contains("test") && s.contains("door") && s.contains("writ")
            && s.contains("before") && s.contains("code")
    });
    // The source of isolated tests may share the sentence about the writing order.
    assert_affirmative(&door.replacen("not from the code", "", 1));
    let before = door.find("before").unwrap();
    let code = door.find("code").unwrap();
    let test = door.find("test").unwrap();
    assert!(
        before < code && (test < before || code < test),
        "the code precedes the door tests: {door}"
    );

    let isolated = rule_sentence(
        &step,
        "write tests of one part in isolation after the part, from the Scenario and not from the code",
        |s| {
            s.contains("test")
                && contains_any(s, &["isolat", "unit test"])
                && s.contains("writ")
                && s.contains("after")
                && s.contains("from the scenario")
                && s.contains("not from the code")
        },
    );
    // Remove only the rule's own exclusion; any other reversal must still fail.
    assert_affirmative(&isolated.replacen("not from the code", "", 1));
    let after = isolated.find("after").unwrap();
    let follows = isolated[after + "after".len()..].trim_start();
    assert!(
        ["the code", "writing the code", "the part", "writing the part", "it"]
            .iter()
            .any(|object| follows.strip_prefix(object).is_some_and(|rest| {
                rest.chars().next().is_none_or(|c| !c.is_alphanumeric())
            })),
        "the isolated tests must follow the code or the part: {isolated}"
    );
    assert!(
        isolated.find("from the scenario").unwrap()
            < isolated.find("not from the code").unwrap(),
        "the isolated tests have the wrong source: {isolated}"
    );
}

// behavior-2-follow-up: Step 3 puts the writing of `realization.md` before the last run
#[test]
fn step_3_writes_realization_before_the_last_run() {
    let sentence = rule_sentence(
        &installed_step(3),
        "write realization.md before the last run",
        |s| {
            s.contains("realization.md")
                && contains_any(s, &["writ", "move"])
                && s.contains("before")
                && contains_any(s, &["last run", "final run", "last azimuth run", "final azimuth run"])
        },
    );
    assert_affirmative(&sentence);
    let before = sentence.find("before").unwrap();
    let run = sentence.find("run").unwrap();
    let realization = sentence.find("realization.md").unwrap();
    assert!(
        before < run && (realization < before || run < realization),
        "the last run precedes realization.md: {sentence}"
    );
}

// behavior-2-follow-up: Step 3 says where `realization.md` stands: beside `specification.md`, in the folder of the module
#[test]
fn step_3_places_realization_beside_specification_in_the_module_folder() {
    rule_sentence(&installed_step(3), "realization.md stands beside specification.md in the module folder", |s| {
        s.contains("realization.md") && s.contains("specification.md") && s.contains("module")
            && contains_any(s, &["folder", "directory", "azimuth/model/<module>/"])
            && contains_any(s, &["beside", "alongside", "same folder", "same directory", "together"])
    });
}

// behavior-2-follow-up: Step 3 says what `realization.md` holds: the heading of the Requirement again, its roots, and its decisions with their reasons, where a root marks where the realization starts
#[test]
fn step_3_describes_the_requirement_heading_roots_and_reasoned_decisions() {
    let step = installed_step(3);
    rule_sentence(&step, "repeat the Requirement heading in realization.md", |s| {
        s.contains("requirement") && s.contains("heading")
            && contains_any(s, &["repeat", "again", "same heading", "copy"])
    });
    rule_sentence(&step, "realization.md holds roots and decisions with their reasons", |s| {
        s.contains("root") && s.contains("decision") && contains_any(s, &["reason", "why"])
    });
    rule_sentence(&step, "a root marks where the realization of a Requirement starts", |s| {
        s.contains("root") && s.contains("realization") && s.contains("requirement")
            && contains_any(s, &["start", "begin", "entry"])
    });
}

// behavior-2-follow-up: Step 3 ends when `azimuth run` succeeds and `azimuth status` names nothing that a run could settle
#[test]
fn step_3_ends_when_a_successful_run_can_settle_nothing_more() {
    let step = installed_step(3);
    let sentence = rule_sentence(
        &step,
        "azimuth run succeeds and status names nothing a run could settle",
        |s| {
            s.contains("azimuth run")
                && s.contains("succeed")
                && s.contains("azimuth status")
                && contains_any(s, &["nothing", "no remaining", "no work"])
                && contains_any(s, &["settle", "resolve"])
                && contains_any(s, &["until", "ends when", "end when"])
        },
    );
    assert_affirmative(&sentence.replace("no remaining", "").replace("no work", ""));
    assert!(
        !prose(&step).contains("until azimuth status is quiet")
            && !prose(&step).contains("azimuth status is quiet"),
        "Build still demands quiet status: {step}"
    );
}

// behavior-2-follow-up: It says what may be left: a review, which this version does not record, a Check that runs only in CI, and a Check that cannot run here
#[test]
fn step_3_allows_unrecorded_reviews_ci_checks_and_checks_that_cannot_run_here_to_remain() {
    let step = installed_step(3);
    let sentence = rule_sentence(
        &step,
        "status may still name an unrecorded review or a CI-only Check",
        |s| {
            s.contains("review")
                && s.contains("check")
                && s.contains("ci")
                && contains_any(s, &["may", "can"])
                && contains_any(s, &["still", "remain", "left"])
                && s.contains("version")
                && contains_any(s, &["does not record", "records no reviews"])
                && contains_any(s, &["only in ci", "ci only", "ci-only"])
        },
    );
    assert_affirmative(
        &sentence
            .replace("does not record", "")
            .replace("records no reviews", "")
            .replace("cannot run here", "")
            .replace("can't run here", "")
            .replace("cannot run locally", "")
            .replace("cannot run on this machine", ""),
    );
    rule_sentence(&step, "a Check that cannot run here may remain", |s| {
        s.contains("check")
            && contains_any(s, &["cannot run here", "can't run here", "cannot run locally", "cannot run on this machine"])
            && contains_any(s, &["may", "can remain", "can still", "allowed"])
            && contains_any(s, &["still", "remain", "left", "stay"])
    });
}

// behavior-2-follow-up: It says that a Check that a run here could settle is work of the Build
#[test]
fn step_3_keeps_locally_settleable_checks_in_the_build() {
    rule_sentence(&installed_step(3), "a Check that a run here could settle is work of the Build", |s| {
        s.contains("check") && s.contains("run") && s.contains("build")
            && contains_any(s, &["here", "locally", "this machine"])
            && contains_any(s, &["settle", "resolve"])
            && contains_any(s, &["work", "finish", "complete", "remain", "belongs"])
    });
}

// behavior-2-follow-up: Step 4 tells the agent to name every proposed decision that the finished code does not follow, and to say why
#[test]
fn step_4_names_every_departure_from_the_plan_and_why() {
    let sentence = rule_sentence(
        &installed_step(4),
        "every proposed decision the finished code does not follow is named with its reason",
        |s| {
            s.contains("proposed")
                && s.contains("decision")
                && s.contains("code")
                && contains_any(s, &["every", "each", "all"])
                && contains_any(s, &["name", "list"])
                && contains_any(
                    s,
                    &[
                        "does not follow",
                        "did not follow",
                        "doesn't follow",
                        "depart",
                        "not implement",
                        "not match",
                    ],
                )
                && contains_any(s, &["why", "reason"])
        },
    );
    assert_affirmative(
        &sentence
            .replace("does not follow", "")
            .replace("did not follow", "")
            .replace("doesn't follow", "")
            .replace("not implement", "")
            .replace("not match", ""),
    );
}

// behavior-2-follow-up: Step 4 tells the agent to name what `azimuth status` still names
#[test]
fn step_4_names_the_work_status_still_reports() {
    positive_rule_in_sentence(
        4,
        "name what azimuth status still names at hand over",
        |s| {
            s.contains("azimuth status")
                && contains_any(s, &["still", "remain", "left"])
                && contains_any(s, &["name", "list", "report"])
        },
    );
}

// behavior-2-follow-up: Step 4 says that a declared review is still owed and is done outside Azimuth
#[test]
fn step_4_keeps_declared_reviews_owed_outside_azimuth() {
    let step = installed_step(4);
    rule_sentence(&step, "a declared review is still owed", |s| {
        s.contains("review") && contains_any(s, &["declared", "required", "declares"])
            && contains_any(s, &["owed", "due", "outstanding"])
    });
    rule_sentence(&step, "the review is done outside Azimuth", |s| {
        s.contains("review") && s.contains("azimuth")
            && contains_any(s, &["outside", "externally"])
    });
}

// behavior-2-follow-up: Step 4 says what follows a commit lane that fails: after status 1 the agent repairs the code or the tests, after 2 the model, and it runs the lane again
#[test]
fn step_4_repairs_commit_lane_failures_by_status_and_runs_it_again() {
    let step = installed_step(4);
    let fixes = |s: &str| contains_any(s, &["fix", "repair", "correct"]);
    rule_sentence(&step, "after status 1 fix the code or the tests", |s| {
        s.split(|c: char| !c.is_alphanumeric()).any(|word| word == "1")
            && s.contains("code") && s.contains("test") && fixes(s)
    });
    rule_sentence(&step, "after status 2 fix the model", |s| {
        s.split(|c: char| !c.is_alphanumeric()).any(|word| word == "2")
            && s.contains("model") && fixes(s)
    });
    rule_sentence(&step, "run the commit lane again after repairs", |s| {
        contains_any(s, &["again", "rerun", "re-run", "retry", "repeat"])
            && contains_any(s, &["commit lane", "lane", "azimuth run --lane commit"])
    });
}

// behavior-2-follow-up: Step 4 tells the agent to name a proposed decision that served a Requirement with no `Realization:` as archived with the plan
#[test]
fn step_4_names_decisions_without_realization_as_archived_with_the_plan() {
    rule_sentence(&installed_step(4), "name a proposed decision for a Requirement with no Realization: as archived with the plan", |s| {
        s.contains("proposed") && s.contains("decision") && s.contains("requirement")
            && s.contains("realization:") && s.contains("plan") && s.contains("archiv")
            && contains_any(s, &["no realization:", "without realization:", "does not declare", "declares no"])
            && contains_any(s, &["name", "report", "list", "say", "state"])
    });
}
