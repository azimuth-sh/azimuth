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

fn rule_in_step(number: u32, rule: &str, holds: impl Fn(&str) -> bool) {
    let step = installed_step(number);
    assert!(
        holds(&prose(&step)),
        "step {number} does not say {rule}:\n{step}"
    );
}

fn rule_in_paragraph(number: u32, rule: &str, holds: impl Fn(&str) -> bool) {
    let step = installed_step(number);
    assert!(
        step.split("\n\n").any(|paragraph| holds(&prose(paragraph))),
        "step {number} does not say {rule}:\n{step}"
    );
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
    rule_in_step(1, "the moved folder is named <date>-<name>", |s| {
        s.contains("<date>-<name>") && s.contains("azimuth/changes/finished/")
    });
}

// behavior-2-follow-up: It says which day the date is: the day on which the folder arrived on the target branch
#[test]
fn step_1_uses_the_day_the_folder_arrived_on_the_target() {
    rule_in_paragraph(
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
/// only YYYY-MM-DD on stdout. The two executable tests use that same block with TZ=Etc/GMT-14.
// behavior-2-follow-up: It gives one git command that prints that day in UTC, and tells the agent not to type the date
#[test]
fn step_1_reads_the_utc_day_from_git_instead_of_typing_it() {
    let step = installed_step(1);
    arrival_command(&step);
    let lower = prose(&step);
    assert!(lower.contains("utc"), "step 1 does not name UTC:\n{step}");
    assert!(
        contains_any(
            &lower,
            &[
                "do not type",
                "never type",
                "nobody types",
                "don't type",
                "do not enter",
                "never enter"
            ]
        ),
        "step 1 does not forbid typing the date:\n{step}"
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

// behavior-2-follow-up: Step 2 says that a Requirement and its Scenarios speak about the system as somebody outside it meets it
#[test]
fn step_2_keeps_requirements_and_scenarios_outside_the_system() {
    rule_in_paragraph(
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
    rule_in_paragraph(
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
    rule_in_paragraph(2, "every part carries the marker of its Check", |s| {
        s.contains("part")
            && s.contains("marker")
            && s.contains("check")
            && contains_any(s, &["every", "each", "all"])
    });
}

// behavior-2-follow-up: It says that a Check is never made only of tests of one part in isolation
#[test]
fn step_2_forbids_a_check_made_only_of_isolated_tests() {
    rule_in_paragraph(2, "a Check is never made only of isolated tests", |s| {
        s.contains("check")
            && contains_any(s, &["isolat", "unit test"])
            && contains_any(s, &["never", "must not", "cannot", "can't", "do not"])
            && contains_any(s, &["only", "alone"])
    });
}

// behavior-2-follow-up: It gives the order of the claim of such a Check: the door, what each part shows, and last what is not shown
#[test]
fn step_2_orders_the_claim_door_parts_then_limits() {
    rule_in_paragraph(
        2,
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
}

// behavior-2-follow-up: Step 2 says that the change needs a plan when a Requirement that it adds or changes declares `Realization:`
#[test]
fn step_2_requires_a_plan_for_an_added_or_changed_realization_requirement() {
    rule_in_paragraph(
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
    rule_in_paragraph(2, "no such Requirement means no plan", |s| {
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
    });
}

// behavior-2-follow-up: It puts the writing of the plan after the Scenarios and the Checks, and before the one stop of the step
#[test]
fn step_2_writes_the_plan_after_scenarios_and_checks_before_its_only_stop() {
    let step = installed_step(2);
    let lower = prose(&step);
    let plan_at = step
        .split("\n\n")
        .find(|p| {
            let p = prose(p);
            p.contains("plan") && contains_any(&p, &["write", "draft"])
        })
        .map(|p| step.find(p).unwrap())
        .expect("step 2 never asks the agent to write the plan");
    let scenario_at = lower.find("scenario").expect("step 2 names no Scenario");
    let check_at = lower.find("check").expect("step 2 names no Check");
    let plan_paragraph = prose(paragraph_at(&step, plan_at));
    let plan_in_lower = lower.find(&plan_paragraph).unwrap();
    let explicit_after = plan_paragraph.contains("after")
        && plan_paragraph.contains("scenario")
        && plan_paragraph.contains("check");
    assert!(
        explicit_after || (scenario_at < plan_in_lower && check_at < plan_in_lower),
        "the plan is not written after the Scenarios and Checks:\n{step}"
    );
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
    rule_in_paragraph(2, "the plan at the stop holds proposed decisions, units to add, change or delete, and migration steps", |s| {
        s.contains("plan") && s.contains("proposed") && s.contains("decision")
            && s.contains("unit") && s.contains("add") && s.contains("chang")
            && contains_any(s, &["delet", "remov"]) && s.contains("migration") && s.contains("step")
    });
}

// behavior-2-follow-up: It says that what somebody outside the system can observe is a Scenario and no line of the plan, and that the addresses and the answers between the team's own components are in the plan
#[test]
fn step_2_puts_observations_in_scenarios_and_internal_contracts_in_the_plan() {
    rule_in_step(2, "outside observations belong to Scenarios, while internal addresses and answers belong to the plan", |s| {
        s.contains("scenario") && s.contains("plan") && contains_any(s, &["outside", "external"])
            && contains_any(s, &["observ", "see", "meet"])
            && contains_any(s, &["not in the plan", "out of the plan", "only in", "belongs in", "belong in", "is a scenario", "are scenarios"])
            && s.contains("address") && contains_any(s, &["answer", "response"])
            && s.contains("component") && contains_any(s, &["own", "internal"])
    });
}

// behavior-2-follow-up: It asks about data and the release before
#[test]
fn step_2_asks_about_data_and_the_release_before() {
    let step = installed_step(2)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        step.contains(
            "What data is added or changed, and how does the release before it run beside it?"
        ),
        "step 2 lacks the exact question about data and the release before:\n{step}"
    );
}

// behavior-2-follow-up: It asks about the promises between components
#[test]
fn step_2_asks_about_promises_between_components() {
    let step = installed_step(2)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        step.contains("What do the team's own components promise each other?"),
        "step 2 lacks the exact question about promises between components:\n{step}"
    );
}

// behavior-2-follow-up: It asks who may do it
#[test]
fn step_2_asks_who_may_do_it() {
    let step = installed_step(2)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        step.contains("Who may do it, and where is that enforced?"),
        "step 2 lacks the exact question about who may do it:\n{step}"
    );
}

// behavior-2-follow-up: It asks how a refusal reaches the caller
#[test]
fn step_2_asks_how_a_refusal_reaches_the_caller() {
    let step = installed_step(2)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        step.contains("How does a refusal reach the caller?"),
        "step 2 lacks the exact question about how a refusal reaches the caller:\n{step}"
    );
}

// behavior-2-follow-up: It asks about two requests that meet
#[test]
fn step_2_asks_about_two_requests_that_meet() {
    let step = installed_step(2)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        step.contains("What happens when two requests meet?"),
        "step 2 lacks the exact question about two requests that meet:\n{step}"
    );
}

// behavior-2-follow-up: It asks what can no longer be deleted
#[test]
fn step_2_asks_what_can_no_longer_be_deleted() {
    let step = installed_step(2)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        step.contains("What can no longer be deleted?"),
        "step 2 lacks the exact question about what can no longer be deleted:\n{step}"
    );
}

// behavior-2-follow-up: It says that a question with an answer becomes a proposed decision, and that a question with no answer leaves no line
#[test]
fn step_2_turns_answers_into_proposed_decisions_and_leaves_no_empty_line() {
    rule_in_step(2, "an answered question becomes a proposed decision and an unanswered question leaves no line", |s| {
        s.contains("question") && s.contains("answer") && s.contains("plan")
            && s.contains("proposed") && s.contains("decision")
            && contains_any(s, &["no answer", "unanswered", "without an answer"])
            && contains_any(s, &["no line", "omit", "leave nothing", "write nothing"])
    });
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
}

// behavior-2-follow-up: Step 3 tells the agent to move a proposed decision into `realization.md` only when the finished code follows it
#[test]
fn step_3_keeps_only_decisions_the_finished_code_follows() {
    rule_in_paragraph(
        3,
        "a proposed decision moves to realization.md only when finished code follows it",
        |s| {
            s.contains("proposed")
                && s.contains("decision")
                && s.contains("realization.md")
                && s.contains("code")
                && contains_any(s, &["finished", "built", "completed"])
                && contains_any(s, &["only if", "only when"])
                && contains_any(s, &["follow", "implement", "match"])
        },
    );
}

// behavior-2-follow-up: Step 4 tells the agent to name every proposed decision that the finished code does not follow, and to say why
#[test]
fn step_4_names_every_departure_from_the_plan_and_why() {
    rule_in_paragraph(
        4,
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
}
