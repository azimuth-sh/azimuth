//! The change comparison behaviors of docs/increments/1.md and docs/increments/2.md.

use azimuth_tests::*;

fn change_diff(repo: &Repo, args: &[&str]) -> serde_json::Value {
    let mut all = vec!["change", "diff", "--json"];
    all.extend_from_slice(args);
    let output = repo.azimuth(&all);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "change-1", "{document:#}");
    document
}

/// One list of a `change-1` document. Lists hold ids, and a Scenario as `requirement/scenario`.
fn listed(document: &serde_json::Value, kind: &str, which: &str) -> Vec<String> {
    let list = &document[kind][which];
    assert!(
        list.is_array(),
        "`{kind}.{which}` is not a list: {document:#}"
    );
    strings_of(list)
}

fn names(document: &serde_json::Value, kind: &str, which: &str, id: &str) -> bool {
    listed(document, kind, which)
        .iter()
        .any(|entry| entry == id)
}

fn class(document: &serde_json::Value) -> String {
    document["class"].as_str().unwrap_or_default().to_string()
}

fn on_branch(repo: &Repo) {
    repo.git(&["checkout", "--quiet", "-b", "change"]);
}

/// Declares a new Check under an existing Scenario and marks a new test for it.
fn add_check(repo: &Repo, id: &str) {
    repo.replace(
        LENDING_VERIFICATION,
        "  After one return the desk lends again to a borrower who held five loans.\n",
        &format!("  After one return the desk lends again to a borrower who held five loans.\n\n- Check: {id}\n  A claim of the new Check {id}.\n"),
    );
    repo.write(
        &format!("suites/desk-checks/{id}.test"),
        &format!("# Check: {id}\noutcome: passed\n"),
    );
}

fn remove_returned_book_check(repo: &Repo) {
    repo.replace(
        LENDING_VERIFICATION,
        "- Check: returned-book-frees-a-slot\n  After one return the desk lends again to a borrower who held five loans.\n",
        "",
    );
    repo.replace(
        LOAN_LIMIT_TESTS,
        "# Check: returned-book-frees-a-slot\n",
        "",
    );
}

// behavior: By default it compares the branch with its target
#[test]
fn change_diff_compares_the_branch_with_main() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    repo.commit_all("add a Check");
    let document = change_diff(&repo, &[]);
    assert_eq!(document["target"], "main", "{document:#}");
    assert!(
        names(&document, "checks", "added", "renewal-keeps-the-count"),
        "{document:#}"
    );
}

// behavior: It lists added, changed and removed Requirements, Scenarios and Checks
#[test]
fn change_diff_lists_what_the_change_did_to_the_model() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    repo.append(
        LENDING_SPEC,
        "\n### Requirement: renewals-are-limited\nA loan is renewed at most twice.\n",
    );
    repo.replace(
        LENDING_SPEC,
        "A borrower holds at most five loans at a time.",
        "A borrower holds at most five loans at once.",
    );
    repo.replace(
        LENDING_SPEC,
        "#### Scenario: returned-book-frees-a-slot\nA borrower who holds five loans returns one book and asks for another.\nThe desk lends it.\n",
        "",
    );
    repo.replace(
        LENDING_VERIFICATION,
        "#### Scenario: returned-book-frees-a-slot\n",
        "",
    );
    remove_returned_book_check(&repo);
    repo.replace(
        RETURNS_SPEC,
        "The desk records a fine of three days.",
        "The desk records a fine for each of the three days.",
    );
    repo.replace(
        RETURNS_VERIFICATION,
        "A return three days late records a fine of three days.",
        "A return three days late records three days of fine.",
    );
    repo.commit_all("rework lending");
    let document = change_diff(&repo, &[]);
    assert!(
        names(&document, "requirements", "added", "renewals-are-limited"),
        "{document:#}"
    );
    assert!(
        names(&document, "requirements", "changed", LOAN_LIMIT),
        "{document:#}"
    );
    assert!(
        names(
            &document,
            "scenarios",
            "removed",
            &format!("{LOAN_LIMIT}/{RETURNED_BOOK}")
        ),
        "{document:#}"
    );
    assert!(
        names(
            &document,
            "scenarios",
            "changed",
            &format!("{LATE_FINE}/{LATE_RETURN}")
        ),
        "{document:#}"
    );
    assert!(
        names(&document, "checks", "removed", RETURNED_BOOK),
        "{document:#}"
    );
    assert!(
        names(&document, "checks", "changed", LATE_RETURN),
        "{document:#}"
    );
    assert!(
        names(&document, "checks", "added", "renewal-keeps-the-count"),
        "{document:#}"
    );
}

// behavior: It stores nothing
#[test]
fn change_diff_stores_nothing() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let before = repo.snapshot();
    change_diff(&repo, &[]);
    repo.azimuth(&["change", "diff"]).expect_status(0);
    assert_eq!(
        repo.snapshot(),
        before,
        "change diff wrote into the worktree"
    );
    assert!(repo.store_files().is_empty(), "change diff wrote the store");
}

// behavior: It reads the name of the target branch from the project file
#[test]
fn change_diff_uses_the_target_of_the_project_file() {
    let repo = Repo::sample_with(|root| {
        let project = std::fs::read_to_string(root.join(PROJECT_FILE)).unwrap();
        std::fs::write(
            root.join(PROJECT_FILE),
            project.replace("target = \"main\"", "target = \"release\""),
        )
        .unwrap();
    });
    repo.git(&["checkout", "--quiet", "-b", "release"]);
    add_check(&repo, "release-only-check");
    repo.commit_all("a Check on the release branch");
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_diff(&repo, &[]);
    assert_eq!(document["target"], "release");
    assert!(
        names(&document, "checks", "added", "renewal-keeps-the-count"),
        "{document:#}"
    );
    assert!(
        !names(&document, "checks", "added", "release-only-check"),
        "the change was compared with main: {document:#}"
    );
}

// behavior: It compares with the common ancestor of the current commit and the target branch
#[test]
fn change_diff_compares_with_the_common_ancestor() {
    let repo = Repo::sample();
    let start = repo.head();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    repo.commit_all("the change");
    repo.git(&["checkout", "--quiet", "main"]);
    add_check(&repo, "landed-on-main-later");
    repo.commit_all("work that landed on main after the branch started");
    repo.git(&["checkout", "--quiet", "change"]);
    let document = change_diff(&repo, &[]);
    assert_eq!(
        document["base"]
            .as_str()
            .map(|base| start.starts_with(base)),
        Some(true),
        "{document:#}"
    );
    assert!(
        !names(&document, "checks", "removed", "landed-on-main-later"),
        "{document:#}"
    );
    assert_eq!(class(&document), "C", "{document:#}");
}

// behavior: It uses the remote-tracking copy of the target, or the local one when no remote exists, and fetches nothing
#[test]
fn change_diff_uses_the_last_fetched_target_and_never_fetches() {
    let origin = Repo::sample();
    let clone = origin.beside("clone");
    origin.git_in(
        &origin.beside(""),
        &[
            "clone",
            "--quiet",
            origin.root().to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let fetched = origin.git_in(&clone, &["rev-parse", "origin/main"]);

    // Local main moves ahead of origin/main, so the two copies of the target disagree.
    let in_clone = |args: &[&str]| origin.git_in(&clone, args);
    let write = |relative: &str, text: &str| std::fs::write(clone.join(relative), text).unwrap();
    let verification = std::fs::read_to_string(clone.join(LENDING_VERIFICATION)).unwrap();
    write(
        LENDING_VERIFICATION,
        &format!(
            "{verification}\n- Check: local-main-only\n  A Check committed to local main only.\n"
        ),
    );
    write(
        "suites/desk-checks/local-main-only.test",
        "# Check: local-main-only\noutcome: passed\n",
    );
    in_clone(&["add", "-A"]);
    in_clone(&["commit", "--quiet", "-m", "local main moves"]);
    in_clone(&["checkout", "--quiet", "-b", "change"]);

    origin.append(
        PRODUCT_FILE,
        "a commit on the remote that was never fetched\n",
    );
    origin.commit_all("remote moves");

    let output = origin.azimuth_in(&clone, &["change", "diff", "--json"], &[]);
    output.expect_status(0);
    let document = output.json();
    assert!(
        names(&document, "checks", "added", "local-main-only"),
        "the local copy was used: {document:#}"
    );
    assert_eq!(
        origin.git_in(&clone, &["rev-parse", "origin/main"]),
        fetched,
        "change diff fetched"
    );
}

// behavior: On the target branch itself it compares the working tree with the last commit
#[test]
fn on_the_target_branch_the_working_tree_is_the_change() {
    let repo = Repo::sample();
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_diff(&repo, &[]);
    assert!(
        names(&document, "checks", "added", "renewal-keeps-the-count"),
        "{document:#}"
    );
    assert_eq!(
        document["base"]
            .as_str()
            .map(|base| repo.head().starts_with(base)),
        Some(true),
        "{document:#}"
    );
    repo.commit_all("commit the Check");
    let committed = change_diff(&repo, &[]);
    assert!(
        listed(&committed, "checks", "added").is_empty(),
        "{committed:#}"
    );
}

// behavior: `--target <ref>` compares with the common ancestor of another ref
#[test]
fn another_target_can_be_named() {
    let repo = Repo::sample();
    repo.git(&["checkout", "--quiet", "-b", "first"]);
    add_check(&repo, "first-change-check");
    repo.commit_all("first change");
    repo.git(&["checkout", "--quiet", "-b", "second"]);
    add_check(&repo, "second-change-check");
    repo.commit_all("second change");
    let stacked = change_diff(&repo, &["--target", "first"]);
    assert!(
        names(&stacked, "checks", "added", "second-change-check"),
        "{stacked:#}"
    );
    assert!(
        !names(&stacked, "checks", "added", "first-change-check"),
        "{stacked:#}"
    );
    let whole = change_diff(&repo, &[]);
    assert!(
        names(&whole, "checks", "added", "first-change-check")
            && names(&whole, "checks", "added", "second-change-check")
    );
}

fn shallow_clone_without_ancestor() -> (Repo, std::path::PathBuf) {
    let origin = Repo::sample();
    on_branch(&origin);
    add_check(&origin, "renewal-keeps-the-count");
    origin.commit_all("change 1");
    origin.append(PRODUCT_FILE, "renewals do not count\n");
    origin.commit_all("change 2");
    origin.git(&["checkout", "--quiet", "main"]);
    origin.append("desk/returns.txt", "fines are rounded up\n");
    origin.commit_all("main moves");
    let shallow = origin.beside("shallow");
    let url = format!("file://{}", origin.root().display());
    origin.git_in(
        &origin.beside(""),
        &[
            "clone",
            "--quiet",
            "--depth",
            "1",
            "--no-single-branch",
            &url,
            shallow.to_str().unwrap(),
        ],
    );
    origin.git_in(&shallow, &["checkout", "--quiet", "change"]);
    (origin, shallow)
}

// behavior: When the common ancestor is not in the clone it ends with status 2
#[test]
fn a_missing_ancestor_ends_with_two() {
    let (origin, shallow) = shallow_clone_without_ancestor();
    origin
        .azimuth_in(&shallow, &["change", "diff"], &[])
        .expect_status(2);
    origin
        .azimuth_in(&shallow, &["change", "diff", "--json"], &[])
        .expect_status(2);
}

// behavior: When the common ancestor is not in the clone it says so and prints no class
#[test]
fn a_missing_ancestor_is_named_and_no_class_is_guessed() {
    let (origin, shallow) = shallow_clone_without_ancestor();
    let output = origin.azimuth_in(&shallow, &["change", "diff"], &[]);
    assert!(
        output
            .text()
            .contains("the common commit is not in this clone"),
        "{}",
        output.text()
    );
    let lower = output.text().to_lowercase();
    assert!(
        !lower.contains("class"),
        "a class was printed:\n{}",
        output.text()
    );
    let json = origin.azimuth_in(&shallow, &["change", "diff", "--json"], &[]);
    assert!(!json.stdout.contains("\"class\""), "{}", json.stdout);
}

// behavior: Its document is `change-1` with these fields
#[test]
fn change_document_has_its_fields() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_diff(&repo, &[]);
    for field in [
        "target",
        "base",
        "class",
        "requirements",
        "scenarios",
        "checks",
        "markers_removed",
        "not_compared",
        "rigor_lowered",
    ] {
        assert!(has(&document, field), "no `{field}`: {document:#}");
    }
    for kind in ["requirements", "scenarios", "checks"] {
        for which in ["added", "changed", "removed"] {
            assert!(
                has(&document[kind], which),
                "no `{kind}.{which}`: {document:#}"
            );
        }
    }
}

// behavior: It can compare the working tree before a commit exists
#[test]
fn the_preview_reads_the_working_tree() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_diff(&repo, &[]);
    assert!(
        names(&document, "checks", "added", "renewal-keeps-the-count"),
        "{document:#}"
    );
    assert_eq!(class(&document), "C");
}

fn class_of(edit: impl FnOnce(&Repo)) -> (String, serde_json::Value, Output) {
    let repo = Repo::sample();
    on_branch(&repo);
    edit(&repo);
    let document = change_diff(&repo, &[]);
    let text = repo.azimuth(&["change", "diff"]);
    (class(&document), document, text)
}

// behavior: It prints the class, computed from the files that differ
#[test]
fn the_class_is_printed() {
    let (class, _, text) = class_of(|repo| add_check(repo, "renewal-keeps-the-count"));
    assert_eq!(class, "C");
    assert!(text.stdout.contains("class C"), "{}", text.stdout);
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn an_installed_skill_change_is_class_d() {
    let repo = Repo::sample();
    repo.azimuth(&["skills", "install", "codex"])
        .expect_status(0);
    repo.commit_all("install the default process");
    on_branch(&repo);
    repo.replace(SKILL_FILE, "\n", "\nThe person also reads the build log.\n");
    let document = change_diff(&repo, &[]);
    assert_eq!(class(&document), "D", "{document:#}");
}

// behavior-2-exploration-skill: A change that edits `.agents/skills/azimuth-exploration/SKILL.md` is of class D in `azimuth change diff`
#[test]
fn an_installed_exploration_skill_change_is_class_d() {
    let repo = Repo::sample();
    repo.azimuth(&["skills", "install", "codex"]).expect_status(0);
    assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed");
    repo.commit_all("install the two skills");
    on_branch(&repo);
    repo.replace(EXPLORATION_SKILL_FILE, "\n", "\nA team edit to the exploration form.\n");
    let document = change_diff(&repo, &[]);
    assert_eq!(class(&document), "D", "{document:#}");
}

// behavior-2-exploration-skill: A change that touches the link `.claude/skills/azimuth-exploration` is of class D in `azimuth change diff`
#[test]
fn an_exploration_skill_link_change_is_class_d() {
    for retarget in [false, true] {
        let repo = Repo::sample();
        repo.azimuth(&["skills", "install", "claude"]).expect_status(0);
        assert!(repo.path(EXPLORATION_SKILL_FILE).is_file(), "the exploration skill is not installed");
        let link = repo.path(CLAUDE_EXPLORATION_SKILL_LINK);
        assert!(std::fs::symlink_metadata(&link).expect("the exploration skill link").file_type().is_symlink());
        let text = repo.read(EXPLORATION_SKILL_FILE);
        repo.commit_all("install the two skills and their links");
        on_branch(&repo);
        std::fs::remove_file(&link).unwrap();
        if retarget {
            std::os::unix::fs::symlink("../../.agents/skills/azimuth-change", &link).unwrap();
        }
        assert_eq!(repo.read(EXPLORATION_SKILL_FILE), text, "the skill text was also edited");
        let document = change_diff(&repo, &[]);
        assert_eq!(class(&document), "D", "retarget={retarget}: {document:#}");
    }
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_specification_change_is_class_d() {
    let (class, document, _) =
        class_of(|repo| repo.replace(LENDING_SPEC, "at most five loans", "at most six loans"));
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_prose_only_specification_change_is_class_d() {
    let (class, document, _) = class_of(|repo| {
        repo.replace(
            LENDING_SPEC,
            "A loan is one book",
            "A loan is a single book",
        )
    });
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_project_file_change_is_class_d() {
    let (class, document, _) =
        class_of(|repo| repo.append(PROJECT_FILE, "# a comment is a change too\n"));
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_rigor_declaration_change_is_class_d() {
    let repo = Repo::sample_with(|root| {
        let path = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            format!("{text}  Reuse: never, the desk reads today's date\n"),
        )
        .unwrap();
    });
    on_branch(&repo);
    repo.replace(
        RETURNS_VERIFICATION,
        "  Reuse: never, the desk reads today's date\n",
        "",
    );
    assert_eq!(class(&change_diff(&repo, &[])), "D");
}

// behavior: A change to `verification.md`, a file with a marker or any file inside a suite's folder makes the class at least C
#[test]
fn a_verification_change_is_class_c() {
    let (class, document, _) = class_of(|repo| {
        repo.replace(
            LENDING_VERIFICATION,
            "The desk refuses a sixth loan",
            "The desk turns down a sixth loan",
        )
    });
    assert_eq!(class, "C", "{document:#}");
}

// behavior: A change to `verification.md`, a file with a marker or any file inside a suite's folder makes the class at least C
#[test]
fn a_change_to_a_marked_file_is_class_c() {
    let (class, document, _) = class_of(|repo| {
        repo.replace(
            LATE_FINE_TESTS,
            "outcome: passed",
            "outcome: passed\nduration: 3",
        )
    });
    assert_eq!(class, "C", "{document:#}");
}

// behavior: A change to `verification.md`, a file with a marker or any file inside a suite's folder makes the class at least C
#[test]
fn a_change_to_an_unmarked_file_in_a_suite_folder_is_class_c() {
    let (class, document, _) = class_of(|repo| {
        repo.write(
            "suites/desk-checks/calendar.txt",
            "the desk is closed on Sundays\n",
        )
    });
    assert_eq!(class, "C", "{document:#}");
}

// behavior: A change to `realization.md` makes the class at least B
#[test]
fn a_realization_change_is_class_b() {
    let (class, document, _) = class_of(|repo| {
        repo.write(
            "azimuth/model/lending/realization.md",
            "The limit is counted in desk/lending.txt.\n",
        )
    });
    assert_eq!(class, "B", "{document:#}");
    let (class, document, _) = class_of(|repo| {
        repo.write(
            "azimuth/model/lending/realization.md",
            "The limit is counted in desk/lending.txt.\n",
        );
        repo.replace(
            LENDING_VERIFICATION,
            "The desk refuses a sixth loan",
            "The desk turns down a sixth loan",
        );
    });
    assert_eq!(class, "C", "{document:#}");
}

// behavior: A change to other files only is class A
#[test]
fn a_product_change_is_class_a() {
    let (class, document, _) = class_of(|repo| {
        repo.append(PRODUCT_FILE, "renewals do not count\n");
        repo.write("desk/shelves.txt", "north shelf holds novels\n");
        repo.append("README.md", "A note.\n");
    });
    assert_eq!(class, "A", "{document:#}");
}

// behavior: A change that removes a Check is class D and the removal is listed
#[test]
fn removing_a_check_is_class_d() {
    let (class, document, text) = class_of(remove_returned_book_check);
    assert_eq!(class, "D", "{document:#}");
    assert!(
        names(&document, "checks", "removed", RETURNED_BOOK),
        "{document:#}"
    );
    assert!(text.stdout.contains(RETURNED_BOOK), "{}", text.stdout);
}

// behavior: A change that leaves a Check with fewer marker lines than on the target is class D and prints the number of markers removed
#[test]
fn removing_a_marker_line_is_class_d() {
    let repo = Repo::sample_with(|root| {
        std::fs::write(
            root.join("suites/desk-checks/free-return-weekend.test"),
            "# Check: on-time-return-is-free\n# A return on a weekend counts as on time.\noutcome: passed\n",
        )
        .unwrap();
    });
    on_branch(&repo);
    repo.remove("suites/desk-checks/free-return-weekend.test");
    let document = change_diff(&repo, &[]);
    assert_eq!(class(&document), "D", "{document:#}");
    assert_eq!(document["markers_removed"], 1, "{document:#}");
    let text = repo.azimuth(&["change", "diff"]).stdout;
    assert!(text.contains("1 marker removed"), "{text}");
}

// behavior: For every changed file that holds a marker it prints "tests in this file were not compared"
#[test]
fn a_changed_marked_file_is_named_as_not_compared() {
    let (_, document, text) = class_of(|repo| {
        repo.replace(
            FREE_RETURN_TESTS,
            "test: returned-early\noutcome: passed\n",
            "",
        );
        repo.replace(
            LATE_FINE_TESTS,
            "outcome: passed",
            "outcome: passed\nduration: 3",
        );
        repo.append(PRODUCT_FILE, "renewals do not count\n");
    });
    let not_compared = all_strings(&document["not_compared"]);
    for file in [FREE_RETURN_TESTS, LATE_FINE_TESTS] {
        assert!(
            not_compared.iter().any(|entry| entry.ends_with(file)),
            "{document:#}"
        );
    }
    assert!(
        !not_compared
            .iter()
            .any(|entry| entry.ends_with(PRODUCT_FILE)),
        "{document:#}"
    );
    assert_eq!(
        text.stdout
            .matches("tests in this file were not compared")
            .count(),
        2,
        "{}",
        text.stdout
    );
}

// behavior: It builds nothing and starts no adapter
#[test]
fn change_diff_starts_no_adapter() {
    let repo = Repo::sample();
    on_branch(&repo);
    repo.need_facility("present");
    add_check(&repo, "renewal-keeps-the-count");
    change_diff(&repo, &[]);
    assert!(repo.adapter_starts().is_empty() && repo.probe_starts().is_empty());
}

// behavior: A change that adds a test or a Check, or edits a test, stays class C
#[test]
fn adding_or_editing_tests_stays_class_c() {
    let (added_test, document, _) = class_of(|repo| {
        repo.write(
            "suites/desk-checks/late-fine-weekend.test",
            "# Check: late-return-is-fined\noutcome: passed\n",
        );
    });
    assert_eq!(added_test, "C", "{document:#}");
    let (added_check, document, _) = class_of(|repo| add_check(repo, "renewal-keeps-the-count"));
    assert_eq!(added_check, "C", "{document:#}");
    let (edited_test, document, _) =
        class_of(|repo| repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed"));
    assert_eq!(edited_test, "C", "{document:#}");
}

// behavior: A change that removes or loosens `Reuse:` is class D and prints "rigor lowered"
#[test]
fn loosening_reuse_lowers_rigor() {
    let repo = Repo::sample_with(|root| {
        let path = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            format!("{text}  Reuse: never, the desk reads today's date\n"),
        )
        .unwrap();
    });
    on_branch(&repo);
    repo.replace(
        RETURNS_VERIFICATION,
        "  Reuse: never, the desk reads today's date\n",
        "",
    );
    let document = change_diff(&repo, &[]);
    assert_eq!(class(&document), "D");
    assert_eq!(
        document["rigor_lowered"],
        json!([{ "kind": "reuse", "check": ON_TIME_RETURN }]),
        "{document:#}"
    );
    let text = repo.azimuth(&["change", "diff"]).stdout;
    assert!(
        text.contains("rigor lowered") && text.contains(ON_TIME_RETURN),
        "{text}"
    );
}

/// The sample with both reviews declared on the lending Requirement, on a branch.
fn with_reviews_declared() -> Repo {
    let repo = Repo::sample_with(|root| {
        let path = root.join(LENDING_SPEC);
        let text = std::fs::read_to_string(&path).unwrap().replace(
            "A borrower holds at most five loans at a time.\n",
            "A borrower holds at most five loans at a time.\nVerification: reviewed\nRealization: reviewed\n",
        );
        std::fs::write(&path, text).unwrap();
    });
    on_branch(&repo);
    repo
}

// behavior: A change that removes a review declaration from a Requirement prints "rigor lowered"
#[test]
fn removing_a_review_declaration_lowers_rigor() {
    for line in ["Verification: reviewed\n", "Realization: reviewed\n"] {
        let repo = with_reviews_declared();
        repo.replace(LENDING_SPEC, line, "");
        let document = change_diff(&repo, &[]);
        assert_eq!(class(&document), "D", "{line:?}: {document:#}");
        assert_eq!(
            count(&document["rigor_lowered"]),
            1,
            "{line:?}: {document:#}"
        );
        let text = repo.azimuth(&["change", "diff"]).stdout;
        assert!(
            text.contains("rigor lowered") && text.contains(LOAN_LIMIT),
            "{line:?}:\n{text}"
        );
    }
}

// behavior: An entry of `rigor_lowered` holds the kind `reuse`, `verification` or `realization`, and the Check or the Requirement that lost the declaration
#[test]
fn each_lowered_declaration_is_an_entry_with_its_kind_and_owner() {
    let repo = Repo::sample_with(|root| {
        let spec = root.join(LENDING_SPEC);
        let text = std::fs::read_to_string(&spec).unwrap().replace(
            "A borrower holds at most five loans at a time.\n",
            "A borrower holds at most five loans at a time.\nVerification: reviewed\nRealization: reviewed\n",
        );
        std::fs::write(&spec, text).unwrap();
        let verification = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&verification).unwrap();
        std::fs::write(
            &verification,
            format!("{text}  Reuse: never, the desk reads today's date\n"),
        )
        .unwrap();
    });
    on_branch(&repo);
    repo.replace(
        LENDING_SPEC,
        "Verification: reviewed\nRealization: reviewed\n",
        "",
    );
    repo.replace(
        RETURNS_VERIFICATION,
        "  Reuse: never, the desk reads today's date\n",
        "",
    );
    let document = change_diff(&repo, &[]);
    let mut entries = document["rigor_lowered"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| panic!("not a list: {document:#}"));
    entries.sort_by_key(|entry| entry["kind"].as_str().unwrap_or_default().to_string());
    assert_eq!(
        entries,
        vec![
            json!({ "kind": "realization", "requirement": LOAN_LIMIT }),
            json!({ "kind": "reuse", "check": ON_TIME_RETURN }),
            json!({ "kind": "verification", "requirement": LOAN_LIMIT }),
        ],
        "{document:#}"
    );
}

// behavior: A Requirement or a Check that the change removes whole is not listed in `rigor_lowered`
#[test]
fn a_removed_requirement_or_check_is_not_listed_as_lowered() {
    // The returns module goes whole, with its reviews and the Reuse line of one of its Checks.
    let repo = Repo::sample_with(|root| {
        let spec = root.join(RETURNS_SPEC);
        let text = std::fs::read_to_string(&spec).unwrap().replace(
            "A book returned after its due date owes a fine for each day it is late.\n",
            "A book returned after its due date owes a fine for each day it is late.\nVerification: reviewed\n",
        );
        std::fs::write(&spec, text).unwrap();
        let verification = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&verification).unwrap();
        std::fs::write(
            &verification,
            format!("{text}  Reuse: never, the desk reads today's date\n"),
        )
        .unwrap();
    });
    on_branch(&repo);
    repo.remove("azimuth/model/desk");
    for file in [LATE_FINE_TESTS, FREE_RETURN_TESTS] {
        repo.remove(file);
    }
    let document = change_diff(&repo, &[]);
    assert!(
        names(&document, "requirements", "removed", LATE_FINE),
        "{document:#}"
    );
    assert!(
        names(&document, "checks", "removed", ON_TIME_RETURN),
        "{document:#}"
    );
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");

    // A Check removed alone, with its Reuse line, is not lowered either.
    let check = Repo::sample_with(|root| {
        let verification = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&verification).unwrap();
        std::fs::write(
            &verification,
            format!("{text}  Reuse: never, the desk reads today's date\n"),
        )
        .unwrap();
    });
    on_branch(&check);
    check.replace(
        RETURNS_VERIFICATION,
        "- Check: on-time-return-is-free\n  A return on the due date or earlier records no fine.\n  Reuse: never, the desk reads today's date\n",
        "",
    );
    check.remove(FREE_RETURN_TESTS);
    let document = change_diff(&check, &[]);
    assert!(
        names(&document, "checks", "removed", ON_TIME_RETURN),
        "{document:#}"
    );
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
}

// behavior: A change that makes `Reuse:` stricter is not class D for that reason
#[test]
fn tightening_reuse_is_not_class_d() {
    let (class, document, text) = class_of(|repo| {
        repo.append(
            RETURNS_VERIFICATION,
            "  Reuse: never, the desk reads today's date\n",
        );
    });
    assert_eq!(class, "C", "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
    assert!(!text.stdout.contains("rigor lowered"));
}

// behavior: A change to the pinned CLI version is class D
#[test]
fn raising_the_pin_is_class_d() {
    let (class, document, _) = class_of(|repo| {
        repo.replace(
            PROJECT_FILE,
            &format!("cli = \"{}\"", cli_version()),
            &format!("cli = \"{OTHER_CLI}\""),
        );
    });
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to an installed skill file is class D
#[test]
fn editing_an_installed_skill_is_class_d() {
    let repo = Repo::sample();
    repo.azimuth(&["skills", "install", "claude"])
        .expect_status(0);
    repo.commit_all("install the default process");
    on_branch(&repo);
    repo.replace(SKILL_FILE, "\n", "\nNo stop after the intent.\n");
    assert_eq!(class(&change_diff(&repo, &[])), "D");
}

// behavior-2: For a change that touches no model file and no test the command prints one line
// behavior: It prints one line when the change touches no model file and no test
#[test]
fn a_change_outside_model_and_tests_prints_one_line() {
    for product_edit in [false, true] {
        let (_, _, text) = class_of(|repo| {
            if product_edit {
                repo.append(PRODUCT_FILE, "renewals do not count\n");
            }
        });
        text.expect_status(0);
        assert_eq!(text.stdout.lines().count(), 1, "{}", text.stdout);
        assert!(
            !text.stdout.trim().is_empty(),
            "the change prints no summary line"
        );
        quiet_class_is_a(&text.stdout);
    }
}

// behavior: In `change-1`, `class` is one letter, a Scenario is written `requirement/scenario`, `markers_removed` is a number and `rigor_lowered` is a list
#[test]
fn change_document_values_have_their_fixed_forms() {
    let repo = Repo::sample();
    on_branch(&repo);
    repo.append(
        RETURNS_SPEC,
        "\n#### Scenario: lost-book-is-charged\nA book reported lost is charged at its price.\n",
    );
    repo.append(
        RETURNS_VERIFICATION,
        "\n#### Scenario: lost-book-is-charged\n- Check: lost-book-is-charged\n  A lost book is charged at its price.\n",
    );
    repo.write(
        "suites/desk-checks/lost-book.test",
        "# Check: lost-book-is-charged\noutcome: passed\n",
    );
    let document = change_diff(&repo, &[]);
    assert_eq!(document["class"], "D", "{document:#}");
    assert_eq!(
        listed(&document, "scenarios", "added"),
        vec![format!("{LATE_FINE}/lost-book-is-charged")],
        "{document:#}"
    );
    assert_eq!(
        listed(&document, "checks", "added"),
        vec!["lost-book-is-charged".to_string()],
        "{document:#}"
    );
    assert_eq!(document["markers_removed"], 0, "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
}

// behavior: With a change that touches no model file and no test, the text is one line and the document says class `A` with empty lists
#[test]
fn a_change_outside_model_and_tests_is_a_class_a_document_with_one_text_line() {
    let (class, document, text) =
        class_of(|repo| repo.append(PRODUCT_FILE, "renewals do not count\n"));
    text.expect_status(0);
    assert_eq!(text.stdout.lines().count(), 1, "{}", text.stdout);
    assert!(
        !text.stdout.trim().is_empty(),
        "the change prints no summary line"
    );
    quiet_class_is_a(&text.stdout);
    assert_eq!(class, "A", "{document:#}");
    for kind in ["requirements", "scenarios", "checks"] {
        for which in ["added", "changed", "removed"] {
            assert_eq!(
                document[kind][which],
                json!([]),
                "`{kind}.{which}`: {document:#}"
            );
        }
    }
    assert_eq!(document["markers_removed"], 0, "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
}

// behavior-2: The command that compares a branch with its target is `azimuth change diff`
#[test]
fn change_diff_takes_no_id() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    assert_eq!(change_diff(&repo, &[])["class"], "A");
    repo.azimuth(&["change", "diff", LOAN_LIMIT])
        .expect_status(2);
}

// behavior-2: `azimuth change show` is an unknown command
#[test]
fn change_show_is_an_unknown_command() {
    let repo = Repo::sample();
    for args in [&["change", "show"][..], &["change", "show", "--json"][..]] {
        let output = repo.azimuth(args);
        output.expect_status(2);
    }
}

// behavior-2: A changed file with a marker that the target does not have gets no line "tests in this file were not compared"
// behavior-2: Such a file is absent from `not_compared` in the structured form
// behavior-2: A changed file with a marker that the target has still gets the line
#[test]
fn only_marked_files_present_in_the_target_are_not_compared() {
    let repo = Repo::sample();
    let gaining = "suites/desk-checks/unmarked.test";
    let new = "suites/desk-checks/renewal.test";
    repo.write(gaining, "outcome: passed\n");
    repo.commit_all("an unmarked base file");
    on_branch(&repo);
    repo.write(
        new,
        "# Check: returned-book-frees-a-slot\noutcome: passed\n",
    );
    repo.write(gaining, "# Check: sixth-loan-is-refused\noutcome: passed\n");
    repo.replace(FREE_RETURN_TESTS, "# Check: on-time-return-is-free\n", "");
    repo.commit_all("add and remove markers on a branch");
    let doc = change_diff(&repo, &[]);
    let mut paths: Vec<_> = doc["not_compared"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    paths.sort();
    let mut expected = vec![FREE_RETURN_TESTS, gaining];
    expected.sort();
    assert_eq!(paths, expected);
    assert_eq!(doc["markers_removed"], 1);
    let out = repo.azimuth(&["change", "diff"]);
    out.expect_status(0);
    let lines: Vec<_> = out
        .stdout
        .lines()
        .filter(|line| line.contains("tests in this file were not compared"))
        .collect();
    assert_eq!(lines.len(), 2);
    for file in expected {
        assert_eq!(lines.iter().filter(|line| line.contains(file)).count(), 1);
    }
    assert!(
        !out.stdout.contains(new),
        "committed new file was named as not compared: {}",
        out.stdout
    );
}

// behavior-2: In the plain form each added, changed or removed id stands on a line of its own under its kind
#[test]
fn plain_diff_gives_every_model_id_its_own_line_under_its_kind() {
    let repo = Repo::sample();
    let declaration = |id: &str| {
        format!("\n### Requirement: {id}\nThe shelf keeps its order.\n\n#### Scenario: {id}-scenario\nThe shelf is read in order.\n")
    };
    let verification = |id: &str| {
        format!("\n### Requirement: {id}\n#### Scenario: {id}-scenario\n- Check: {id}-check\n  The shelf is read in order.\n")
    };
    for id in ["catalog-order-holds", "shelf-order-holds"] {
        repo.append(LENDING_SPEC, &declaration(id));
        repo.append(LENDING_VERIFICATION, &verification(id));
    }
    repo.commit_all("two removable declarations");
    for id in ["catalog-order-holds", "shelf-order-holds"] {
        repo.replace(LENDING_SPEC, &declaration(id), "");
        repo.replace(LENDING_VERIFICATION, &verification(id), "");
    }
    for id in ["renewal-order-holds", "return-order-holds"] {
        repo.append(LENDING_SPEC, &declaration(id));
        repo.append(LENDING_VERIFICATION, &verification(id));
    }
    repo.replace(
        LENDING_SPEC,
        "at most five loans at a time",
        "at most five concurrent loans",
    );
    repo.replace(
        RETURNS_SPEC,
        "owes a fine for each day it is late",
        "owes one fine for each day it is late",
    );
    repo.replace(
        LENDING_SPEC,
        "The desk refuses the loan and the borrower keeps five.",
        "The desk refuses and the borrower keeps five loans.",
    );
    repo.replace(
        RETURNS_SPEC,
        "The desk records a fine of three days.",
        "The desk records three days of fines.",
    );
    repo.replace(
        LENDING_VERIFICATION,
        "The desk refuses a sixth loan",
        "The desk turns down a sixth loan",
    );
    repo.replace(
        RETURNS_VERIFICATION,
        "A return three days late records a fine of three days.",
        "A return three days late records three days of fines.",
    );
    let output = repo.azimuth(&["change", "diff"]);
    output.expect_status(0);
    let mut expected = std::collections::BTreeMap::new();
    for (kind, operation, mut ids) in [
        (
            "requirements",
            "added",
            vec![
                "renewal-order-holds".to_string(),
                "return-order-holds".to_string(),
            ],
        ),
        (
            "requirements",
            "removed",
            vec![
                "catalog-order-holds".to_string(),
                "shelf-order-holds".to_string(),
            ],
        ),
        (
            "requirements",
            "changed",
            vec![LOAN_LIMIT.to_string(), LATE_FINE.to_string()],
        ),
        (
            "scenarios",
            "added",
            vec![
                "renewal-order-holds/renewal-order-holds-scenario".to_string(),
                "return-order-holds/return-order-holds-scenario".to_string(),
            ],
        ),
        (
            "scenarios",
            "removed",
            vec![
                "catalog-order-holds/catalog-order-holds-scenario".to_string(),
                "shelf-order-holds/shelf-order-holds-scenario".to_string(),
            ],
        ),
        (
            "scenarios",
            "changed",
            vec![
                format!("{LOAN_LIMIT}/{SIXTH_LOAN}"),
                format!("{LATE_FINE}/{LATE_RETURN}"),
            ],
        ),
        (
            "checks",
            "added",
            vec![
                "renewal-order-holds-check".to_string(),
                "return-order-holds-check".to_string(),
            ],
        ),
        (
            "checks",
            "removed",
            vec![
                "catalog-order-holds-check".to_string(),
                "shelf-order-holds-check".to_string(),
            ],
        ),
        (
            "checks",
            "changed",
            vec![SIXTH_LOAN.to_string(), LATE_RETURN.to_string()],
        ),
    ] {
        ids.sort();
        expected.insert((kind.to_string(), operation.to_string()), ids);
    }
    let mut actual: std::collections::BTreeMap<(String, String), Vec<String>> =
        std::collections::BTreeMap::new();
    let mut kind = None;
    let mut operation = None;
    for line in output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let stripped = line
            .trim_start_matches(['-', '+', '*', '•'])
            .trim()
            .trim_matches('`');
        let id_line = stripped
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '/');
        let words: Vec<_> = line
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|word| !word.is_empty())
            .map(str::to_lowercase)
            .collect();
        let found_kind = words
            .iter()
            .find_map(|word| match word.trim_end_matches('s') {
                "requirement" => Some("requirements"),
                "scenario" => Some("scenarios"),
                "check" => Some("checks"),
                _ => None,
            });
        let found_operation = words.iter().find_map(|word| {
            if word.starts_with("add") || word == "new" {
                Some("added")
            } else if word.starts_with("chang")
                || word.starts_with("modif")
                || word.starts_with("edit")
            {
                Some("changed")
            } else if word.starts_with("remov") || word.starts_with("delet") {
                Some("removed")
            } else {
                None
            }
        });
        if id_line
            && (stripped.contains('-')
                || stripped.contains('/')
                || (found_kind.is_none() && found_operation.is_none()))
        {
            let key = (
                kind.clone().expect("id has no kind"),
                operation
                    .clone()
                    .expect("id has no operation under its kind"),
            );
            actual.entry(key).or_default().push(stripped.to_string());
            continue;
        }
        if let Some(found) = found_kind {
            kind = Some(found.to_string());
            operation = None;
        }
        if let Some(found) = found_operation {
            operation = Some(found.to_string());
        }
    }
    for ids in actual.values_mut() {
        ids.sort();
    }
    assert_eq!(
        actual, expected,
        "complete plain groups differ:\n{}",
        output.stdout
    );
}

#[test]
fn the_base_tree_contents_are_read_by_one_git_process() {
    let measure = |extra_files: usize| {
        let repo = Repo::sample();
        let marked: Vec<_> = (0..3)
            .map(|index| format!("suites/desk-checks/base-proof-{index}.test"))
            .collect();
        for file in &marked {
            repo.write(file, "# Check: sixth-loan-is-refused\noutcome: passed\n");
        }
        for index in 0..extra_files {
            repo.write(
                &format!("suites/desk-checks/shelf-{index}.txt"),
                "base contents without markers\n",
            );
        }
        repo.commit_all("several marked base files");
        on_branch(&repo);
        for file in &marked {
            repo.replace(file, "# Check: sixth-loan-is-refused\n", "");
        }
        repo.commit_all("remove three base markers");
        let (doc, invocations, _) = recorded_base_reads(&repo);
        assert_eq!(doc["markers_removed"], 3);
        assert_eq!(doc["class"], "D");
        assert_eq!(
            content_readers(&invocations).len(),
            1,
            "exactly one git process must read base contents: {invocations:#?}"
        );
        invocations.len()
    };
    let small = measure(0);
    let large = measure(40);
    assert_eq!(
        large, small,
        "git process count grew with the base tree: {small} -> {large}"
    );
}

fn quiet_class_is_a(text: &str) {
    let words: Vec<_> = text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    for pair in words.windows(2) {
        if pair[0].eq_ignore_ascii_case("class") {
            assert!(
                pair[1].eq_ignore_ascii_case("a"),
                "quiet change names another class: {text}"
            );
        }
    }
}

fn content_readers(invocations: &[Vec<String>]) -> Vec<&Vec<String>> {
    invocations
        .iter()
        .filter(|args| {
            (args.iter().any(|arg| arg == "cat-file")
                && !args.iter().any(|arg| arg.starts_with("--batch-check")))
                || args.iter().any(|arg| arg == "archive")
                || (args.iter().any(|arg| arg == "show")
                    && args.iter().any(|arg| arg.contains(':')))
        })
        .collect()
}
fn recorded_base_reads(repo: &Repo) -> (serde_json::Value, Vec<Vec<String>>, Vec<String>) {
    let git = std::process::Command::new("/usr/bin/which")
        .arg("git")
        .output()
        .unwrap();
    assert!(git.status.success());
    let git = String::from_utf8(git.stdout).unwrap().trim().to_string();
    let front = repo.beside("git-front");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::copy(binaries_dir().join("git-recorder"), front.join("git")).unwrap();
    let path = format!(
        "{}:{}",
        front.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let log = repo.beside("git-processes.jsonl");
    let objects = repo.beside("git-objects.jsonl");
    let output = repo.azimuth_with_env(
        &["change", "diff", "--json"],
        &[
            ("PATH", &path),
            ("REAL_GIT", &git),
            ("GIT_PROCESS_LOG", log.to_str().unwrap()),
            ("GIT_OBJECT_LOG", objects.to_str().unwrap()),
        ],
    );
    output.expect_status(0);
    let invocations = std::fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let objects = std::fs::read_to_string(objects)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (output.json(), invocations, objects)
}
#[test]
fn base_contents_outside_the_model_and_suites_are_not_requested() {
    let repo = Repo::sample();
    let irrelevant = "assets/unrelated-large.bin";
    std::fs::create_dir_all(repo.path("assets")).unwrap();
    std::fs::write(repo.path(irrelevant), vec![b'z'; 8 * 1024 * 1024]).unwrap();
    let mut needed = Vec::new();
    for index in 0..3 {
        let path = format!("suites/desk-checks/base-marker-{index}.test");
        repo.write(
            &path,
            &format!("# Check: sixth-loan-is-refused\n# distinct base {index}\noutcome: passed\n"),
        );
        needed.push(path);
    }
    repo.commit_all("base markers and an unrelated asset");
    let irrelevant_oid = repo
        .git(&["rev-parse", &format!("main:{irrelevant}")])
        .trim()
        .to_string();
    let needed_oids: Vec<_> = needed
        .iter()
        .map(|path| {
            repo.git(&["rev-parse", &format!("main:{path}")])
                .trim()
                .to_string()
        })
        .collect();
    on_branch(&repo);
    for path in &needed {
        repo.replace(path, "# Check: sixth-loan-is-refused\n", "");
    }
    repo.commit_all("remove three distinct base markers");
    let (doc, invocations, requests) = recorded_base_reads(&repo);
    assert_eq!(doc["markers_removed"], 3);
    assert_eq!(content_readers(&invocations).len(), 1);
    let requested: Vec<_> = requests
        .iter()
        .flat_map(|request| request.split_whitespace())
        .chain(
            content_readers(&invocations)
                .into_iter()
                .flat_map(|args| args.iter().map(String::as_str)),
        )
        .collect();
    assert!(
        !requested.iter().any(|object| *object == irrelevant_oid
            || object.ends_with(&format!(":{irrelevant}"))
            || *object == "--batch-all-objects"),
        "unneeded base object {irrelevant_oid} ({irrelevant}) was requested: {requests:#?}"
    );
    // Distinct removed markers prove this recorder saw base reads, rather than an unused wrapper.
    for oid in needed_oids {
        assert!(
            base_object_was_requested(&repo, &requested, &oid),
            "needed base object {oid} was not observed by id or tree:path"
        );
    }
}
fn base_object_was_requested(repo: &Repo, requests: &[&str], oid: &str) -> bool {
    requests.iter().any(|request| {
        if *request == oid {
            return true;
        }
        if request.starts_with('-') || !request.contains(':') {
            return false;
        }
        let resolved = repo.git_output_in(repo.root(), &["rev-parse", "--verify", request]);
        resolved.status == 0 && resolved.stdout.trim() == oid
    })
}

#[test]
fn a_needed_missing_base_blob_names_its_path_and_ends_with_status_two() {
    let repo = Repo::sample();
    repo.git(&["config", "gc.auto", "0"]);
    let path = "suites/desk-checks/missing-base-marker.test";
    repo.write(
        path,
        "# Check: sixth-loan-is-refused\n# a unique base blob to remove by hand\noutcome: passed\n",
    );
    repo.commit_all("a marked file in the base tree");
    let oid = repo
        .git(&["rev-parse", &format!("main:{path}")])
        .trim()
        .to_string();
    on_branch(&repo);
    repo.replace(path, "# Check: sixth-loan-is-refused\n", "");
    repo.commit_all("remove the base marker");
    let object = repo.root().join(
        repo.git(&[
            "rev-parse",
            "--git-path",
            &format!("objects/{}/{}", &oid[..2], &oid[2..]),
        ])
        .trim(),
    );
    assert!(
        object.is_file(),
        "fixture object is not loose: {}",
        object.display()
    );
    std::fs::remove_file(object).unwrap();
    let output = repo.azimuth(&["change", "diff", "--json"]);
    output.expect_status(2);
    assert!(
        output.text().contains(path),
        "missing base blob error must name {path}:\n{}",
        output.text()
    );
}
#[test]
fn a_scenario_with_one_blank_body_line_does_not_crash_the_diff() {
    let repo = Repo::sample_with(|root| {
        let path = root.join(LENDING_SPEC);
        let source = std::fs::read_to_string(&path).unwrap();
        let body = "A borrower who holds five loans asks for a sixth book.\nThe desk refuses the loan and the borrower keeps five.\n";
        assert!(source.contains(body));
        let blank = source.replace(body, "");
        assert!(blank.contains(
            "#### Scenario: sixth-loan-is-refused\n\n#### Scenario: returned-book-frees-a-slot"
        ));
        std::fs::write(path, blank).unwrap();
    });
    on_branch(&repo);
    repo.append(PRODUCT_FILE, "an unrelated product edit\n");
    for json in [false, true] {
        let args = if json {
            vec!["change", "diff", "--json"]
        } else {
            vec!["change", "diff"]
        };
        let output = repo.azimuth(&args);
        output.expect_status(0);
        if json {
            let doc = output.json();
            assert_eq!(doc["format"], "change-1");
            assert_eq!(doc["class"], "A");
        } else {
            assert_eq!(output.stdout.lines().count(), 1);
            assert!(!output.stdout.trim().is_empty());
        }
    }
}
