//! Group "`azimuth requirement show` and `azimuth check show`" of docs/increments/1.md.

use azimuth_tests::*;

fn known_fail() -> Repo {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.run_json(&[], 1);
    repo
}

// behavior: `azimuth requirement show <id>` prints the Scenarios, the Checks under each with their state, the reviews and the approval
#[test]
fn requirement_show_prints_the_whole_requirement() {
    let repo = known_fail();
    let output = repo.azimuth(&["requirement", "show", LATE_FINE]);
    output.expect_status(0);
    let text = output.stdout;
    for expected in [
        LATE_FINE,
        "late-return-is-fined",
        "on-time-return-is-free",
        "kept",
        "owed",
        "verification: not reviewed",
        "realization: not reviewed",
        "approval: not recorded",
    ] {
        assert!(
            text.contains(expected),
            "requirement show lacks {expected:?}:\n{text}"
        );
    }
    repo.azimuth(&["requirement", "show", "no-such-requirement"])
        .expect_status(2);
}

// behavior: `azimuth check show <id>` prints the Check's state and the reason
#[test]
fn check_show_prints_state_and_reason() {
    let repo = known_fail();
    let output = repo.azimuth(&["check", "show", LATE_RETURN]);
    output.expect_status(0);
    assert!(
        output.stdout.contains("owed") && output.stdout.contains("failed on these inputs"),
        "{}",
        output.stdout
    );
    repo.azimuth(&["check", "show", "no-such-check"])
        .expect_status(2);
}

// behavior: The document of `azimuth requirement show` is `requirement-1`: `format`, and beside it the fields that one Requirement entry has in `status-1`
#[test]
fn requirement_document_matches_its_status_entry() {
    let repo = known_fail();
    let output = repo.azimuth(&["requirement", "show", LATE_FINE, "--json"]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "requirement-1");
    let status = repo.status_json();
    let entry = requirement_in(&status, LATE_FINE).unwrap();
    let mut flat = document.as_object().unwrap().clone();
    flat.remove("format");
    assert_eq!(
        serde_json::Value::Object(flat),
        *entry,
        "requirement-1 is not the status-1 entry beside `format`:\n{document:#}\n{entry:#}"
    );
}

// behavior: The document of `azimuth check show` is `check-1` with these fields
#[test]
fn check_document_has_its_fields() {
    // `reuse` is absent for a Check without a `Reuse:` line, so it is looked for on another Check.
    let repo = Repo::sample();
    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never, the desk reads today's date\n",
    );
    repo.run_json(&[], 0);
    assert!(has(&repo.check_show(ON_TIME_RETURN), "reuse"));
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    let document = repo.check_show(SIXTH_LOAN);
    for field in [
        "id",
        "requirement",
        "scenario",
        "suite",
        "claim",
        "state",
        "reason",
        "detail",
        "parts_changed",
        "files_changed",
        "failing_tests",
        "result",
    ] {
        assert!(has(&document, field), "no `{field}`: {document:#}");
    }
    for field in ["where", "commit", "edited_paths", "at"] {
        assert!(
            has(&document["result"], field),
            "no `result.{field}`: {document:#}"
        );
    }
    assert_eq!(document["id"], SIXTH_LOAN);
    assert_eq!(document["requirement"], LOAN_LIMIT);
    assert_eq!(document["scenario"], SIXTH_LOAN);
    assert_eq!(document["suite"], "desk-checks");
}

// behavior: A kept pass is shown with who produced it, at which commit and how long ago
#[test]
fn a_kept_pass_is_shown_with_its_origin() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let head = repo.head();
    let document = repo.check_show(SIXTH_LOAN);
    assert_eq!(document["state"], "kept");
    let result = &document["result"];
    assert_eq!(result["where"], "this machine", "{document:#}");
    assert_eq!(result["commit"], head.as_str(), "{document:#}");
    assert!(
        !result["at"].as_str().unwrap_or_default().is_empty(),
        "{document:#}"
    );
    let text = repo.azimuth(&["check", "show", SIXTH_LOAN]).stdout;
    assert!(text.contains(&head[..7]), "{text}");
    assert!(text.contains("ago"), "{text}");
}

/// The sample after a run that saw one edited, one new and one deleted path, with those edits
/// then committed. The content is the same as at the run, so the results stay kept, and the tree
/// at `check show` is a clean commit: what is shown about the edits can only come from the record.
fn kept_from_an_edited_tree() -> (Repo, String) {
    let repo = Repo::sample();
    let commit_of_the_run = repo.head();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    repo.remove("desk/returns.txt");
    repo.run_json(&[], 0);
    repo.commit_all("the edits of the run");
    assert_eq!(
        repo.git(&["status", "--porcelain"]),
        "",
        "the tree is not clean, so the test proves nothing"
    );
    assert_ne!(repo.head(), commit_of_the_run);
    (repo, commit_of_the_run)
}

// behavior: For a result of a run that saw edited, new or deleted paths, the text of `azimuth check show` says `commit <7 characters> with uncommitted edits`
#[test]
fn a_result_from_an_edited_tree_says_so() {
    let (repo, commit) = kept_from_an_edited_tree();
    let output = repo.azimuth(&["check", "show", SIXTH_LOAN]);
    output.expect_status(0);
    let phrase = format!("commit {} with uncommitted edits", &commit[..7]);
    assert!(
        output.stdout.contains(&phrase),
        "check show does not say {phrase:?}:\n{}",
        output.stdout
    );
}

// behavior: `result.edited_paths` is the number of edited, new and deleted paths the run saw, and 0 for a run on exactly the commit, where the text names the commit alone
#[test]
fn edited_paths_counts_what_the_run_saw() {
    let (repo, commit) = kept_from_an_edited_tree();
    let document = repo.check_show(SIXTH_LOAN);
    assert_eq!(document["state"], "kept", "{document:#}");
    assert_eq!(
        document["result"]["commit"],
        commit.as_str(),
        "{document:#}"
    );
    assert_eq!(document["result"]["edited_paths"], 3, "{document:#}");

    let clean = Repo::sample();
    clean.run_json(&[], 0);
    let head = clean.head();
    let document = clean.check_show(SIXTH_LOAN);
    assert!(document["result"]["edited_paths"].is_u64(), "{document:#}");
    assert_eq!(document["result"]["edited_paths"], 0, "{document:#}");
    let text = clean.azimuth(&["check", "show", SIXTH_LOAN]).stdout;
    assert!(
        text.contains(&format!("commit {}", &head[..7])),
        "the commit is not named:\n{text}"
    );
    assert!(
        !text.contains("uncommitted"),
        "a run on exactly the commit is shown with edits:\n{text}"
    );
}

// behavior: A failed Check is shown with its failing tests
#[test]
fn a_failed_check_is_shown_with_its_failing_tests() {
    let repo = known_fail();
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(
        strings_of(&document["failing_tests"]),
        vec!["late-fine"],
        "{document:#}"
    );
    let text = repo.azimuth(&["check", "show", LATE_RETURN]).stdout;
    assert!(text.contains("late-fine"), "{text}");
}

// behavior: An owed Check is shown with one of the named reasons
#[test]
fn an_owed_check_has_a_named_reason() {
    let fresh = Repo::sample();
    let document = fresh.check_show(SIXTH_LOAN);
    assert_eq!(document["state"], "owed");
    assert_eq!(document["reason"], "never run", "{document:#}");

    let changed = Repo::sample();
    changed.run_json(&[], 0);
    changed.append(PRODUCT_FILE, "renewals do not count\n");
    let failing = known_fail();
    for (repo, check, reason) in [
        (&changed, SIXTH_LOAN, "an input changed"),
        (&failing, LATE_RETURN, "failed on these inputs"),
    ] {
        let document = repo.check_show(check);
        assert!(
            OWED_REASONS.contains(&document["reason"].as_str().unwrap_or_default()),
            "{document:#}"
        );
        assert_eq!(document["reason"], reason, "{document:#}");
    }
}

// behavior-2: The text of `azimuth check show` names the files
// behavior: When the files part changed, `azimuth check show` names the files that differ
#[test]
fn check_show_names_the_files_that_differ() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf\n");
    let expected = vec![PRODUCT_FILE.to_string(), "desk/shelves.txt".to_string()];
    assert_eq!(files_changed(&repo.check_show(ON_TIME_RETURN)), expected);
    let output = repo.azimuth(&["check", "show", ON_TIME_RETURN]);
    output.expect_status(0);
    for path in &expected {
        let base = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(
            output.stdout.matches(base).count(),
            1,
            "changed file {path} must be named exactly once:\n{}",
            output.stdout
        );
    }
    let mut candidates = repo.tracked_paths();
    candidates.push("desk/shelves.txt".into());
    candidates.sort();
    let shown: Vec<_> = candidates
        .into_iter()
        .filter(|path| {
            output.stdout.contains(path)
                || output.stdout.contains(
                    std::path::Path::new(path)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
        })
        .collect();
    assert_eq!(
        shown, expected,
        "check show names more than the differing files:\n{}",
        output.stdout
    );
}

// behavior: The reason "an input changed" names the part that changed
#[test]
fn an_input_changed_names_its_part() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    let document = repo.check_show(SIXTH_LOAN);
    assert_eq!(document["reason"], "an input changed", "{document:#}");
    assert_eq!(parts_changed(&document), vec!["files"], "{document:#}");
    let text = repo.azimuth(&["check", "show", SIXTH_LOAN]).stdout;
    assert!(text.contains("files"), "{text}");
}

// behavior: A pass and a fail on one fingerprint are shown as contradicted
#[test]
fn a_pass_and_a_fail_on_one_fingerprint_are_contradicted() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let output = repo.azimuth_with_env(
        &["check", "run", LATE_RETURN, "--json"],
        &[("TEXT_ADAPTER_FAIL", "late-fine")],
    );
    output.expect_status(1);
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(document["reason"], "contradicted", "{document:#}");
    let status = repo.status_json();
    assert_eq!(
        status_checks(&status)[LATE_RETURN]["reason"],
        "contradicted",
        "{status:#}"
    );
    let text = repo.azimuth(&["check", "show", LATE_RETURN]).stdout;
    assert!(text.contains("contradicted"), "{text}");
}

fn contradicted() -> Repo {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.azimuth_with_env(
        &["check", "run", LATE_RETURN, "--json"],
        &[("TEXT_ADAPTER_FAIL", "late-fine")],
    )
    .expect_status(1);
    repo
}

// behavior: A contradicted Check has the state `owed` and the reason `contradicted`
#[test]
fn a_contradicted_check_is_owed() {
    let repo = contradicted();
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(
        (document["state"].as_str(), document["reason"].as_str()),
        (Some("owed"), Some("contradicted")),
        "{document:#}"
    );
    let status = repo.status_json();
    let entry = &status_checks(&status)[LATE_RETURN];
    assert_eq!(
        (entry["state"].as_str(), entry["reason"].as_str()),
        (Some("owed"), Some("contradicted")),
        "{status:#}"
    );
}

// behavior: `parts_changed` is a list of part names and `files_changed` a list of paths from the repository root
#[test]
fn the_changed_parts_and_files_are_plain_lists() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.replace(
        LENDING_VERIFICATION,
        "The desk refuses a sixth loan",
        "The desk turns down a sixth loan",
    );
    repo.write("desk/shelves/north.txt", "novels\n");
    let document = repo.check_show(SIXTH_LOAN);
    assert_eq!(
        parts_changed_set(&document),
        set_of(&["claim", "files"]),
        "{document:#}"
    );
    assert_eq!(
        files_changed(&document),
        vec![
            LENDING_VERIFICATION.to_string(),
            "desk/shelves/north.txt".to_string()
        ],
        "{document:#}"
    );
    let untouched = repo.check_show(RETURNED_BOOK);
    assert_eq!(parts_changed(&untouched), vec!["files"], "{untouched:#}");
}

// behavior: `result.commit` is the full commit id, and the text shows 7 characters of it
#[test]
fn the_commit_is_full_in_the_document_and_short_in_the_text() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let head = repo.head();
    assert_eq!(
        repo.check_show(SIXTH_LOAN)["result"]["commit"],
        head.as_str()
    );
    let text = repo.azimuth(&["check", "show", SIXTH_LOAN]).stdout;
    assert!(text.contains(&head[..7]), "{text}");
    assert!(
        !text.contains(&head[..8]),
        "the text shows more than 7 characters of the commit:\n{text}"
    );
}

// behavior: `reuse` is absent for a Check with no `Reuse:` line, and otherwise holds `value` and `reason`
#[test]
fn reuse_is_absent_or_a_value_and_a_reason() {
    let repo = Repo::sample();
    repo.replace(
        RETURNS_VERIFICATION,
        "  A return on the due date or earlier records no fine.\n",
        "  A return on the due date or earlier records no fine.\n  Reuse: never, the desk reads today's date\n",
    );
    let plain = repo.check_show(LATE_RETURN);
    assert!(plain.get("reuse").is_none(), "{plain:#}");
    let strict = repo.check_show(ON_TIME_RETURN);
    let reuse = strict["reuse"]
        .as_object()
        .unwrap_or_else(|| panic!("no `reuse`: {strict:#}"));
    assert_eq!(reuse.len(), 2, "{strict:#}");
    assert_eq!(reuse["value"], "never");
    assert_eq!(
        reuse["reason"].as_str().map(str::trim),
        Some("the desk reads today's date")
    );
}

// behavior-2: The text of `azimuth requirement show` does the same
#[test]
fn requirement_show_counts_files_on_each_owed_check_line() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    for count in [1, 3] {
        if count == 3 {
            repo.write("desk/shelves.txt", "north\n");
            repo.write("desk/catalog.txt", "books\n");
        }
        let output = repo.azimuth(&["requirement", "show", LOAN_LIMIT]);
        output.expect_status(0);
        for check in [SIXTH_LOAN, RETURNED_BOOK] {
            assert_owed_file_count(&output.stdout, check, count);
        }
        assert_no_file_names(
            &output.stdout,
            &[PRODUCT_FILE, "desk/shelves.txt", "desk/catalog.txt"],
        );
    }
    repo.azimuth(&["check", "run", RETURNED_BOOK, "--json"])
        .expect_status(0);
    repo.write("desk/loans.txt", "one more changed file\n");
    let output = repo.azimuth(&["requirement", "show", LOAN_LIMIT]);
    output.expect_status(0);
    for (check, expected) in [(SIXTH_LOAN, 4), (RETURNED_BOOK, 1)] {
        assert_owed_file_count(&output.stdout, check, expected);
    }
    assert_no_file_names(
        &output.stdout,
        &[
            PRODUCT_FILE,
            "desk/shelves.txt",
            "desk/catalog.txt",
            "desk/loans.txt",
        ],
    );
}

// behavior-2: With `--json` all three still hold `files_changed` for every Check
#[test]
fn reading_documents_keep_the_exact_file_list_for_each_check() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.azimuth(&["check", "run", RETURNED_BOOK, "--json"])
        .expect_status(0);
    repo.write("desk/shelves.txt", "north\n");
    repo.azimuth(&["check", "run", ON_TIME_RETURN, "--json"])
        .expect_status(0);
    let expectations = std::collections::BTreeMap::from([
        (
            SIXTH_LOAN,
            (
                "owed",
                vec![PRODUCT_FILE.to_string(), "desk/shelves.txt".to_string()],
            ),
        ),
        (
            RETURNED_BOOK,
            ("owed", vec!["desk/shelves.txt".to_string()]),
        ),
        (
            LATE_RETURN,
            (
                "owed",
                vec![PRODUCT_FILE.to_string(), "desk/shelves.txt".to_string()],
            ),
        ),
        (ON_TIME_RETURN, ("kept", vec![])),
    ]);
    let status = repo.status_json();
    assert_eq!(status_checks(&status).len(), 4);
    for requirement in [LOAN_LIMIT, LATE_FINE] {
        let output = repo.azimuth(&["requirement", "show", requirement, "--json"]);
        output.expect_status(0);
        let shown = output.json();
        assert_eq!(shown["format"], "requirement-1");
        let checks = status_checks(&json!({"requirements":[shown]}));
        assert_eq!(checks.len(), 2);
        for (id, entry) in checks {
            let (state, expected) = &expectations[id.as_str()];
            for document in [&entry, &status_checks(&status)[&id], &repo.check_show(&id)] {
                assert_eq!(document["state"], *state);
                assert_eq!(&files_changed(document), expected, "{id}: {document:#}");
            }
        }
    }
}
