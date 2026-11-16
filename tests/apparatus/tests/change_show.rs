//! Group "`azimuth change show`" of docs/increments/1.md.

use azimuth_tests::*;

fn change_show(repo: &Repo, args: &[&str]) -> serde_json::Value {
    let mut all = vec!["change", "show", "--json"];
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
    assert!(list.is_array(), "`{kind}.{which}` is not a list: {document:#}");
    strings_of(list)
}

fn names(document: &serde_json::Value, kind: &str, which: &str, id: &str) -> bool {
    listed(document, kind, which).iter().any(|entry| entry == id)
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
    repo.write(&format!("suites/desk-checks/{id}.test"), &format!("# Check: {id}\noutcome: passed\n"));
}

fn remove_returned_book_check(repo: &Repo) {
    repo.replace(
        LENDING_VERIFICATION,
        "- Check: returned-book-frees-a-slot\n  After one return the desk lends again to a borrower who held five loans.\n",
        "",
    );
    repo.replace(LOAN_LIMIT_TESTS, "# Check: returned-book-frees-a-slot\n", "");
}

// behavior: By default it compares the branch with its target
#[test]
fn change_show_compares_the_branch_with_main() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    repo.commit_all("add a Check");
    let document = change_show(&repo, &[]);
    assert_eq!(document["target"], "main", "{document:#}");
    assert!(names(&document, "checks", "added", "renewal-keeps-the-count"), "{document:#}");
}

// behavior: It lists added, changed and removed Requirements, Scenarios and Checks
#[test]
fn change_show_lists_what_the_change_did_to_the_model() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    repo.append(LENDING_SPEC, "\n### Requirement: renewals-are-limited\nA loan is renewed at most twice.\n");
    repo.replace(LENDING_SPEC, "A borrower holds at most five loans at a time.", "A borrower holds at most five loans at once.");
    repo.replace(
        LENDING_SPEC,
        "#### Scenario: returned-book-frees-a-slot\nA borrower who holds five loans returns one book and asks for another.\nThe desk lends it.\n",
        "",
    );
    repo.replace(LENDING_VERIFICATION, "#### Scenario: returned-book-frees-a-slot\n", "");
    remove_returned_book_check(&repo);
    repo.replace(RETURNS_SPEC, "The desk records a fine of three days.", "The desk records a fine for each of the three days.");
    repo.replace(RETURNS_VERIFICATION, "A return three days late records a fine of three days.", "A return three days late records three days of fine.");
    repo.commit_all("rework lending");
    let document = change_show(&repo, &[]);
    assert!(names(&document, "requirements", "added", "renewals-are-limited"), "{document:#}");
    assert!(names(&document, "requirements", "changed", LOAN_LIMIT), "{document:#}");
    assert!(names(&document, "scenarios", "removed", &format!("{LOAN_LIMIT}/{RETURNED_BOOK}")), "{document:#}");
    assert!(names(&document, "scenarios", "changed", &format!("{LATE_FINE}/{LATE_RETURN}")), "{document:#}");
    assert!(names(&document, "checks", "removed", RETURNED_BOOK), "{document:#}");
    assert!(names(&document, "checks", "changed", LATE_RETURN), "{document:#}");
    assert!(names(&document, "checks", "added", "renewal-keeps-the-count"), "{document:#}");
}

// behavior: It stores nothing
#[test]
fn change_show_stores_nothing() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let before = repo.snapshot();
    change_show(&repo, &[]);
    repo.azimuth(&["change", "show"]).expect_status(0);
    assert_eq!(repo.snapshot(), before, "change show wrote into the worktree");
    assert!(repo.store_files().is_empty(), "change show wrote the store");
}

// behavior: It reads the name of the target branch from the project file
#[test]
fn change_show_uses_the_target_of_the_project_file() {
    let repo = Repo::sample_with(|root| {
        let project = std::fs::read_to_string(root.join(PROJECT_FILE)).unwrap();
        std::fs::write(root.join(PROJECT_FILE), project.replace("target = \"main\"", "target = \"release\"")).unwrap();
    });
    repo.git(&["checkout", "--quiet", "-b", "release"]);
    add_check(&repo, "release-only-check");
    repo.commit_all("a Check on the release branch");
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_show(&repo, &[]);
    assert_eq!(document["target"], "release");
    assert!(names(&document, "checks", "added", "renewal-keeps-the-count"), "{document:#}");
    assert!(!names(&document, "checks", "added", "release-only-check"), "the change was compared with main: {document:#}");
}

// behavior: It compares with the common ancestor of the current commit and the target branch
#[test]
fn change_show_compares_with_the_common_ancestor() {
    let repo = Repo::sample();
    let start = repo.head();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    repo.commit_all("the change");
    repo.git(&["checkout", "--quiet", "main"]);
    add_check(&repo, "landed-on-main-later");
    repo.commit_all("work that landed on main after the branch started");
    repo.git(&["checkout", "--quiet", "change"]);
    let document = change_show(&repo, &[]);
    assert_eq!(document["base"].as_str().map(|base| start.starts_with(base)), Some(true), "{document:#}");
    assert!(!names(&document, "checks", "removed", "landed-on-main-later"), "{document:#}");
    assert_eq!(class(&document), "C", "{document:#}");
}

// behavior: It uses the remote-tracking copy of the target, or the local one when no remote exists, and fetches nothing
#[test]
fn change_show_uses_the_last_fetched_target_and_never_fetches() {
    let origin = Repo::sample();
    let clone = origin.beside("clone");
    origin.git_in(&origin.beside(""), &["clone", "--quiet", origin.root().to_str().unwrap(), clone.to_str().unwrap()]);
    let fetched = origin.git_in(&clone, &["rev-parse", "origin/main"]);

    // Local main moves ahead of origin/main, so the two copies of the target disagree.
    let in_clone = |args: &[&str]| origin.git_in(&clone, args);
    let write = |relative: &str, text: &str| std::fs::write(clone.join(relative), text).unwrap();
    let verification = std::fs::read_to_string(clone.join(LENDING_VERIFICATION)).unwrap();
    write(LENDING_VERIFICATION, &format!("{verification}\n- Check: local-main-only\n  A Check committed to local main only.\n"));
    write("suites/desk-checks/local-main-only.test", "# Check: local-main-only\noutcome: passed\n");
    in_clone(&["add", "-A"]);
    in_clone(&["commit", "--quiet", "-m", "local main moves"]);
    in_clone(&["checkout", "--quiet", "-b", "change"]);

    origin.append(PRODUCT_FILE, "a commit on the remote that was never fetched\n");
    origin.commit_all("remote moves");

    let output = origin.azimuth_in(&clone, &["change", "show", "--json"], &[]);
    output.expect_status(0);
    let document = output.json();
    assert!(names(&document, "checks", "added", "local-main-only"), "the local copy was used: {document:#}");
    assert_eq!(origin.git_in(&clone, &["rev-parse", "origin/main"]), fetched, "change show fetched");
}

// behavior: On the target branch itself it compares the working tree with the last commit
#[test]
fn on_the_target_branch_the_working_tree_is_the_change() {
    let repo = Repo::sample();
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_show(&repo, &[]);
    assert!(names(&document, "checks", "added", "renewal-keeps-the-count"), "{document:#}");
    assert_eq!(document["base"].as_str().map(|base| repo.head().starts_with(base)), Some(true), "{document:#}");
    repo.commit_all("commit the Check");
    let committed = change_show(&repo, &[]);
    assert!(listed(&committed, "checks", "added").is_empty(), "{committed:#}");
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
    let stacked = change_show(&repo, &["--target", "first"]);
    assert!(names(&stacked, "checks", "added", "second-change-check"), "{stacked:#}");
    assert!(!names(&stacked, "checks", "added", "first-change-check"), "{stacked:#}");
    let whole = change_show(&repo, &[]);
    assert!(names(&whole, "checks", "added", "first-change-check") && names(&whole, "checks", "added", "second-change-check"));
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
    origin.git_in(&origin.beside(""), &["clone", "--quiet", "--depth", "1", "--no-single-branch", &url, shallow.to_str().unwrap()]);
    origin.git_in(&shallow, &["checkout", "--quiet", "change"]);
    (origin, shallow)
}

// behavior: When the common ancestor is not in the clone it ends with status 2
#[test]
fn a_missing_ancestor_ends_with_two() {
    let (origin, shallow) = shallow_clone_without_ancestor();
    origin.azimuth_in(&shallow, &["change", "show"], &[]).expect_status(2);
    origin.azimuth_in(&shallow, &["change", "show", "--json"], &[]).expect_status(2);
}

// behavior: When the common ancestor is not in the clone it says so and prints no class
#[test]
fn a_missing_ancestor_is_named_and_no_class_is_guessed() {
    let (origin, shallow) = shallow_clone_without_ancestor();
    let output = origin.azimuth_in(&shallow, &["change", "show"], &[]);
    assert!(output.text().contains("the common commit is not in this clone"), "{}", output.text());
    let lower = output.text().to_lowercase();
    assert!(!lower.contains("class"), "a class was printed:\n{}", output.text());
    let json = origin.azimuth_in(&shallow, &["change", "show", "--json"], &[]);
    assert!(!json.stdout.contains("\"class\""), "{}", json.stdout);
}

// behavior: Its document is `change-1` with these fields
#[test]
fn change_document_has_its_fields() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_show(&repo, &[]);
    for field in ["target", "base", "class", "requirements", "scenarios", "checks", "markers_removed", "not_compared", "rigor_lowered"] {
        assert!(has(&document, field), "no `{field}`: {document:#}");
    }
    for kind in ["requirements", "scenarios", "checks"] {
        for which in ["added", "changed", "removed"] {
            assert!(has(&document[kind], which), "no `{kind}.{which}`: {document:#}");
        }
    }
}

// behavior: It can compare the working tree before a commit exists
#[test]
fn the_preview_reads_the_working_tree() {
    let repo = Repo::sample();
    on_branch(&repo);
    add_check(&repo, "renewal-keeps-the-count");
    let document = change_show(&repo, &[]);
    assert!(names(&document, "checks", "added", "renewal-keeps-the-count"), "{document:#}");
    assert_eq!(class(&document), "C");
}

fn class_of(edit: impl FnOnce(&Repo)) -> (String, serde_json::Value, Output) {
    let repo = Repo::sample();
    on_branch(&repo);
    edit(&repo);
    let document = change_show(&repo, &[]);
    let text = repo.azimuth(&["change", "show"]);
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
    repo.azimuth(&["skills", "install", "codex"]).expect_status(0);
    repo.commit_all("install the default process");
    on_branch(&repo);
    repo.replace(SKILL_FILE, "\n", "\nThe person also reads the build log.\n");
    let document = change_show(&repo, &[]);
    assert_eq!(class(&document), "D", "{document:#}");
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_specification_change_is_class_d() {
    let (class, document, _) = class_of(|repo| repo.replace(LENDING_SPEC, "at most five loans", "at most six loans"));
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_prose_only_specification_change_is_class_d() {
    let (class, document, _) = class_of(|repo| repo.replace(LENDING_SPEC, "A loan is one book", "A loan is a single book"));
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_project_file_change_is_class_d() {
    let (class, document, _) = class_of(|repo| repo.append(PROJECT_FILE, "# a comment is a change too\n"));
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to `specification.md`, a rigor declaration, the project file or an installed skill file makes the class D
#[test]
fn a_rigor_declaration_change_is_class_d() {
    let repo = Repo::sample_with(|root| {
        let path = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{text}  Reuse: never, the desk reads today's date\n")).unwrap();
    });
    on_branch(&repo);
    repo.replace(RETURNS_VERIFICATION, "  Reuse: never, the desk reads today's date\n", "");
    assert_eq!(class(&change_show(&repo, &[])), "D");
}

// behavior: A change to `verification.md`, a file with a marker or any file inside a suite's folder makes the class at least C
#[test]
fn a_verification_change_is_class_c() {
    let (class, document, _) = class_of(|repo| repo.replace(LENDING_VERIFICATION, "The desk refuses a sixth loan", "The desk turns down a sixth loan"));
    assert_eq!(class, "C", "{document:#}");
}

// behavior: A change to `verification.md`, a file with a marker or any file inside a suite's folder makes the class at least C
#[test]
fn a_change_to_a_marked_file_is_class_c() {
    let (class, document, _) = class_of(|repo| repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: passed\nduration: 3"));
    assert_eq!(class, "C", "{document:#}");
}

// behavior: A change to `verification.md`, a file with a marker or any file inside a suite's folder makes the class at least C
#[test]
fn a_change_to_an_unmarked_file_in_a_suite_folder_is_class_c() {
    let (class, document, _) = class_of(|repo| repo.write("suites/desk-checks/calendar.txt", "the desk is closed on Sundays\n"));
    assert_eq!(class, "C", "{document:#}");
}

// behavior: A change to `realization.md` makes the class at least B
#[test]
fn a_realization_change_is_class_b() {
    let (class, document, _) = class_of(|repo| repo.write("azimuth/model/lending/realization.md", "The limit is counted in desk/lending.txt.\n"));
    assert_eq!(class, "B", "{document:#}");
    let (class, document, _) = class_of(|repo| {
        repo.write("azimuth/model/lending/realization.md", "The limit is counted in desk/lending.txt.\n");
        repo.replace(LENDING_VERIFICATION, "The desk refuses a sixth loan", "The desk turns down a sixth loan");
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
    assert!(names(&document, "checks", "removed", RETURNED_BOOK), "{document:#}");
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
    let document = change_show(&repo, &[]);
    assert_eq!(class(&document), "D", "{document:#}");
    assert_eq!(document["markers_removed"], 1, "{document:#}");
    let text = repo.azimuth(&["change", "show"]).stdout;
    assert!(text.contains("1 marker removed"), "{text}");
}

// behavior: For every changed file that holds a marker it prints "tests in this file were not compared"
#[test]
fn a_changed_marked_file_is_named_as_not_compared() {
    let (_, document, text) = class_of(|repo| {
        repo.replace(FREE_RETURN_TESTS, "test: returned-early\noutcome: passed\n", "");
        repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: passed\nduration: 3");
        repo.append(PRODUCT_FILE, "renewals do not count\n");
    });
    let not_compared = all_strings(&document["not_compared"]);
    for file in [FREE_RETURN_TESTS, LATE_FINE_TESTS] {
        assert!(not_compared.iter().any(|entry| entry.ends_with(file)), "{document:#}");
    }
    assert!(!not_compared.iter().any(|entry| entry.ends_with(PRODUCT_FILE)), "{document:#}");
    assert_eq!(text.stdout.matches("tests in this file were not compared").count(), 2, "{}", text.stdout);
}

// behavior: It builds nothing and starts no adapter
#[test]
fn change_show_starts_no_adapter() {
    let repo = Repo::sample();
    on_branch(&repo);
    repo.need_facility("present");
    add_check(&repo, "renewal-keeps-the-count");
    change_show(&repo, &[]);
    assert!(repo.adapter_starts().is_empty() && repo.probe_starts().is_empty());
}

// behavior: A change that adds a test or a Check, or edits a test, stays class C
#[test]
fn adding_or_editing_tests_stays_class_c() {
    let (added_test, document, _) = class_of(|repo| {
        repo.write("suites/desk-checks/late-fine-weekend.test", "# Check: late-return-is-fined\noutcome: passed\n");
    });
    assert_eq!(added_test, "C", "{document:#}");
    let (added_check, document, _) = class_of(|repo| add_check(repo, "renewal-keeps-the-count"));
    assert_eq!(added_check, "C", "{document:#}");
    let (edited_test, document, _) = class_of(|repo| repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed"));
    assert_eq!(edited_test, "C", "{document:#}");
}

// behavior: A change that removes or loosens `Reuse:` is class D and prints "rigor lowered"
#[test]
fn loosening_reuse_lowers_rigor() {
    let repo = Repo::sample_with(|root| {
        let path = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{text}  Reuse: never, the desk reads today's date\n")).unwrap();
    });
    on_branch(&repo);
    repo.replace(RETURNS_VERIFICATION, "  Reuse: never, the desk reads today's date\n", "");
    let document = change_show(&repo, &[]);
    assert_eq!(class(&document), "D");
    assert_eq!(document["rigor_lowered"], json!([{ "kind": "reuse", "check": ON_TIME_RETURN }]), "{document:#}");
    let text = repo.azimuth(&["change", "show"]).stdout;
    assert!(text.contains("rigor lowered") && text.contains(ON_TIME_RETURN), "{text}");
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
        let document = change_show(&repo, &[]);
        assert_eq!(class(&document), "D", "{line:?}: {document:#}");
        assert_eq!(count(&document["rigor_lowered"]), 1, "{line:?}: {document:#}");
        let text = repo.azimuth(&["change", "show"]).stdout;
        assert!(text.contains("rigor lowered") && text.contains(LOAN_LIMIT), "{line:?}:\n{text}");
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
        std::fs::write(&verification, format!("{text}  Reuse: never, the desk reads today's date\n")).unwrap();
    });
    on_branch(&repo);
    repo.replace(LENDING_SPEC, "Verification: reviewed\nRealization: reviewed\n", "");
    repo.replace(RETURNS_VERIFICATION, "  Reuse: never, the desk reads today's date\n", "");
    let document = change_show(&repo, &[]);
    let mut entries = document["rigor_lowered"].as_array().cloned().unwrap_or_else(|| panic!("not a list: {document:#}"));
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
        std::fs::write(&verification, format!("{text}  Reuse: never, the desk reads today's date\n")).unwrap();
    });
    on_branch(&repo);
    repo.remove("azimuth/model/desk");
    for file in [LATE_FINE_TESTS, FREE_RETURN_TESTS] {
        repo.remove(file);
    }
    let document = change_show(&repo, &[]);
    assert!(names(&document, "requirements", "removed", LATE_FINE), "{document:#}");
    assert!(names(&document, "checks", "removed", ON_TIME_RETURN), "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");

    // A Check removed alone, with its Reuse line, is not lowered either.
    let check = Repo::sample_with(|root| {
        let verification = root.join(RETURNS_VERIFICATION);
        let text = std::fs::read_to_string(&verification).unwrap();
        std::fs::write(&verification, format!("{text}  Reuse: never, the desk reads today's date\n")).unwrap();
    });
    on_branch(&check);
    check.replace(
        RETURNS_VERIFICATION,
        "- Check: on-time-return-is-free\n  A return on the due date or earlier records no fine.\n  Reuse: never, the desk reads today's date\n",
        "",
    );
    check.remove(FREE_RETURN_TESTS);
    let document = change_show(&check, &[]);
    assert!(names(&document, "checks", "removed", ON_TIME_RETURN), "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
}

// behavior: A change that makes `Reuse:` stricter is not class D for that reason
#[test]
fn tightening_reuse_is_not_class_d() {
    let (class, document, text) = class_of(|repo| {
        repo.append(RETURNS_VERIFICATION, "  Reuse: never, the desk reads today's date\n");
    });
    assert_eq!(class, "C", "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
    assert!(!text.stdout.contains("rigor lowered"));
}

// behavior: A change to the pinned CLI version is class D
#[test]
fn raising_the_pin_is_class_d() {
    let (class, document, _) = class_of(|repo| {
        repo.replace(PROJECT_FILE, &format!("cli = \"{}\"", cli_version()), &format!("cli = \"{OTHER_CLI}\""));
    });
    assert_eq!(class, "D", "{document:#}");
}

// behavior: A change to an installed skill file is class D
#[test]
fn editing_an_installed_skill_is_class_d() {
    let repo = Repo::sample();
    repo.azimuth(&["skills", "install", "claude"]).expect_status(0);
    repo.commit_all("install the default process");
    on_branch(&repo);
    repo.replace(SKILL_FILE, "\n", "\nNo stop after the intent.\n");
    assert_eq!(class(&change_show(&repo, &[])), "D");
}

// behavior: It prints nothing when the change touches no model file and no test
#[test]
fn a_change_outside_model_and_tests_prints_nothing() {
    let (_, _, text) = class_of(|repo| repo.append(PRODUCT_FILE, "renewals do not count\n"));
    text.expect_status(0);
    assert!(text.stdout.trim().is_empty(), "{}", text.stdout);
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
    repo.write("suites/desk-checks/lost-book.test", "# Check: lost-book-is-charged\noutcome: passed\n");
    let document = change_show(&repo, &[]);
    assert_eq!(document["class"], "D", "{document:#}");
    assert_eq!(listed(&document, "scenarios", "added"), vec![format!("{LATE_FINE}/lost-book-is-charged")], "{document:#}");
    assert_eq!(listed(&document, "checks", "added"), vec!["lost-book-is-charged".to_string()], "{document:#}");
    assert_eq!(document["markers_removed"], 0, "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
}

// behavior: With a change that touches no model file and no test, the text is empty and the document says class `A` with empty lists
#[test]
fn a_change_outside_model_and_tests_is_an_empty_class_a_document() {
    let (class, document, text) = class_of(|repo| repo.append(PRODUCT_FILE, "renewals do not count\n"));
    text.expect_status(0);
    assert_eq!(text.stdout, "", "the text is not empty");
    assert_eq!(class, "A", "{document:#}");
    for kind in ["requirements", "scenarios", "checks"] {
        for which in ["added", "changed", "removed"] {
            assert_eq!(document[kind][which], json!([]), "`{kind}.{which}`: {document:#}");
        }
    }
    assert_eq!(document["markers_removed"], 0, "{document:#}");
    assert_eq!(document["rigor_lowered"], json!([]), "{document:#}");
}
