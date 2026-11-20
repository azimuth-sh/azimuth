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
    let shown: Vec<_> = output
        .stdout
        .lines()
        .skip_while(|line| *line != "    files  changed")
        .skip(1)
        .map_while(|line| line.strip_prefix("      "))
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

const CLASS_RETURN: &str = "suites/desk-checks/class-return.test";
const METHOD_RETURN: &str = "suites/desk-checks/method-return.test";
const RETURN_TESTS: [(&str, &str); 3] = [
    ("class-return.charges-late-return", "failed"),
    ("class-return.accepts-exact-fine", "passed"),
    ("method-return.issues-receipt", "passed"),
];

fn return_parts() -> Repo {
    let repo = Repo::sample();
    repo.remove(LATE_FINE_TESTS);
    // The text-suite adapter gives a file marker the same meaning as a class marker.
    repo.write(
        CLASS_RETURN,
        include_str!("fixtures/check-parts/class-return.test"),
    );
    repo.write(
        METHOD_RETURN,
        include_str!("fixtures/check-parts/method-return.test"),
    );
    repo
}

fn return_parts_result() -> Repo {
    let repo = return_parts();
    repo.run_json(&[], 1);
    let ran = repo.tests_ran();
    for (name, _) in RETURN_TESTS {
        assert!(
            ran.contains(name),
            "the fixture did not run {name}: {ran:?}"
        );
    }
    repo
}

fn return_parts_text(repo: &Repo) -> String {
    let output = repo.azimuth(&["check", "show", LATE_RETURN]);
    output.expect_status(0);
    output.stdout
}

fn marker_lines<'a>(text: &'a str, file: &str, line: u64) -> Vec<&'a str> {
    text.lines()
        .filter(|shown| {
            shown.contains(file)
                && shown
                    .split(|c: char| !c.is_ascii_digit())
                    .any(|number| number == line.to_string())
        })
        .collect()
}

fn assert_test_outcome(text: &str, name: &str, outcome: &str) {
    assert!(
        text.lines()
            .any(|line| line.contains(name) && line.contains(outcome)),
        "the tests of the last result do not show {name} with outcome {outcome}:\n{text}"
    );
}

fn list_length(document: &serde_json::Value, field: &str) -> usize {
    document[field]
        .as_array()
        .unwrap_or_else(|| panic!("check-1 lacks the list {field}: {document:#}"))
        .len()
}

fn mentions_result_time(text: &str) -> bool {
    text.split_whitespace().any(|word| {
        let word = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        let lower = word.to_lowercase();
        if ["ago", "age", "old", "seconds", "minutes", "hours", "days"].contains(&lower.as_str()) {
            return true;
        }
        let bytes = word.as_bytes();
        let date = bytes.len() >= 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[..4].iter().all(u8::is_ascii_digit)
            && bytes[5..7].iter().all(u8::is_ascii_digit)
            && bytes[8..10].iter().all(u8::is_ascii_digit);
        let time = word.split_once(':').is_some_and(|(hour, rest)| {
            hour.len() == 2
                && hour.bytes().all(|c| c.is_ascii_digit())
                && rest.len() >= 2
                && rest.as_bytes()[..2].iter().all(u8::is_ascii_digit)
        });
        let short_age = ["ms", "s", "m", "h", "d"].iter().any(|unit| {
            word.strip_suffix(unit)
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()))
        });
        date || time || short_age
    }) || text.to_lowercase().contains("just now")
}

// behavior-2-follow-up: `azimuth check show <id>` prints the marker lines of the Check, each with its file and line
#[test]
fn check_show_prints_each_marker_file_and_line() {
    let repo = return_parts_result();
    let text = return_parts_text(&repo);
    for file in [CLASS_RETURN, METHOD_RETURN] {
        let line = repo.line_of(file, &format!("# Check: {LATE_RETURN}"));
        assert_eq!(
            marker_lines(&text, file, line).len(),
            1,
            "check show must print the marker at {file}:{line} once:\n{text}"
        );
    }
}

// behavior-2-follow-up: A marker on a class is printed as one line, and no test is named beside it
#[test]
fn check_show_keeps_a_class_marker_as_one_line_without_test_names() {
    let repo = return_parts_result();
    let text = return_parts_text(&repo);
    let lines = marker_lines(&text, CLASS_RETURN, repo.line_of(CLASS_RETURN, "# Check:"));
    assert_eq!(
        lines.len(),
        1,
        "the class marker is missing or expanded into tests:\n{text}"
    );
    assert_eq!(
        text.lines()
            .filter(|line| line.contains(CLASS_RETURN))
            .count(),
        1,
        "the class marker must occupy only one line:\n{text}"
    );
    for (name, _) in RETURN_TESTS {
        assert!(
            !lines[0].contains(name),
            "a class marker names a test beside it: {}",
            lines[0]
        );
        assert!(
            !lines[0].contains(name.split('.').next_back().unwrap()),
            "a class marker names a short test name beside it: {}",
            lines[0]
        );
    }
}

// behavior-2-follow-up: It prints the tests of the last result of the Check, each with its outcome
#[test]
fn check_show_prints_the_tests_of_the_last_result_with_outcomes() {
    let repo = return_parts_result();
    // The last result differs from the first, so this cannot pass by reading any old result.
    repo.azimuth_with_env(
        &["check", "run", LATE_RETURN, "--json"],
        &[("TEXT_ADAPTER_FAIL", "method-return.issues-receipt")],
    )
    .expect_status(1);
    let text = return_parts_text(&repo);
    for (name, outcome) in RETURN_TESTS {
        let last_outcome = if name == "method-return.issues-receipt" {
            "failed"
        } else {
            outcome
        };
        assert_test_outcome(&text, name, last_outcome);
        let other = if last_outcome == "passed" {
            "failed"
        } else {
            "passed"
        };
        assert!(
            !text
                .lines()
                .any(|line| line.contains(name) && line.contains(other)),
            "check show also gives {name} an outcome from an earlier result:\n{text}"
        );
    }
}

// behavior-2-follow-up: The second list also holds the tests that passed
#[test]
fn check_show_includes_passed_tests_beside_failed_tests() {
    let repo = return_parts_result();
    let text = return_parts_text(&repo);
    for (name, outcome) in RETURN_TESTS
        .into_iter()
        .filter(|(_, outcome)| *outcome == "passed")
    {
        assert_test_outcome(&text, name, outcome);
    }
}

// behavior-2-follow-up: It says how old that result is
#[test]
fn check_show_dates_the_last_result_even_when_it_is_owed() {
    let repo = return_parts_result();
    for edited in [false, true] {
        if edited {
            repo.append(PRODUCT_FILE, "a return also issues a receipt\n");
        }
        let text = return_parts_text(&repo);
        assert!(mentions_result_time(&text),
            "check show gives neither the time nor the age of the last result (edited={edited}):\n{text}");
    }
}

// behavior-2-follow-up: It starts no launch, also for a Check that is owed
#[test]
fn check_show_starts_no_launch_for_kept_or_owed_checks() {
    let repo = return_parts_result();
    repo.clear_log();
    for check in [LATE_RETURN, SIXTH_LOAN] {
        for json in [false, true] {
            let args = if json {
                vec!["check", "show", check, "--json"]
            } else {
                vec!["check", "show", check]
            };
            repo.azimuth(&args).expect_status(0);
        }
    }
    repo.append(PRODUCT_FILE, "a return also issues a receipt\n");
    for json in [false, true] {
        let args = if json {
            vec!["check", "show", SIXTH_LOAN, "--json"]
        } else {
            vec!["check", "show", SIXTH_LOAN]
        };
        repo.azimuth(&args).expect_status(0);
    }
    assert!(
        repo.adapter_starts().is_empty(),
        "check show started an adapter: {:?}",
        repo.log_entries()
    );
    assert!(
        repo.probe_starts().is_empty(),
        "check show started a facility proof"
    );
}

// behavior-2-follow-up: After an edit that adds a marked test, the first list shows the new marker and the second list is unchanged
#[test]
fn check_show_refreshes_markers_without_changing_the_last_tests() {
    let repo = return_parts_result();
    let before_text = return_parts_text(&repo);
    let before = repo.check_show(LATE_RETURN);
    repo.append(
        METHOD_RETURN,
        "\ntest: keeps-receipt\n# Check: late-return-is-fined\noutcome: passed\n",
    );
    let new_line = repo
        .read(METHOD_RETURN)
        .lines()
        .enumerate()
        .filter(|(_, line)| *line == "# Check: late-return-is-fined")
        .map(|(index, _)| index)
        .last()
        .unwrap()
        + 1;
    repo.clear_log();
    let after_text = return_parts_text(&repo);
    assert_eq!(
        marker_lines(&after_text, METHOD_RETURN, new_line as u64).len(),
        1,
        "the new marker is not visible before the next run:\n{after_text}"
    );
    for (name, outcome) in RETURN_TESTS {
        assert_test_outcome(&before_text, name, outcome);
        assert_test_outcome(&after_text, name, outcome);
    }
    assert!(
        !after_text.contains("method-return.keeps-receipt"),
        "the new test has never run but is shown in the last result:\n{after_text}"
    );
    let after = repo.check_show(LATE_RETURN);
    assert_eq!(list_length(&before, "markers"), 2);
    assert_eq!(list_length(&after, "markers"), 3);
    assert_eq!(list_length(&before, "tests"), 3);
    assert_eq!(
        after["tests"], before["tests"],
        "the second list changed without a launch"
    );
    assert!(
        repo.adapter_starts().is_empty(),
        "show launched a runner after the edit"
    );
}

// behavior-2-follow-up: With `--json` the document holds the fields `markers` and `tests`
#[test]
fn check_show_json_has_marker_and_test_lists_with_different_counts() {
    let repo = return_parts_result();
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(
        list_length(&document, "markers"),
        2,
        "one entry per marker line"
    );
    assert_eq!(
        list_length(&document, "tests"),
        3,
        "one entry per test of the last result"
    );
}

// behavior-2-follow-up: `failing_tests` is still there and holds the failed tests of the second list
#[test]
fn check_show_failing_tests_keeps_only_the_failed_names() {
    let repo = return_parts_result();
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(
        strings_of(&document["failing_tests"]),
        vec!["class-return.charges-late-return"],
        "failing_tests must keep the failed name and omit the two passed names: {document:#}"
    );
}

// behavior-2-follow-up: The document still names its format `check-1`
#[test]
fn check_show_keeps_the_check_1_format_name() {
    let repo = return_parts_result();
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(
        document["format"], "check-1",
        "adding lists must not raise the format number"
    );
}

// behavior-2-follow-up: With `--json` an entry of `markers` holds `file`, the path from the root of the repository, and `line`
#[test]
fn check_show_json_names_each_marker_by_repository_path_and_line() {
    let repo = return_parts_result();
    let document = repo.check_show(LATE_RETURN);
    let markers = document["markers"]
        .as_array()
        .unwrap_or_else(|| panic!("check-1 lacks markers with file and line: {document:#}"));
    let mut actual: Vec<_> = markers
        .iter()
        .map(|marker| {
            let file = marker["file"].as_str().unwrap_or_else(|| {
                panic!("a marker needs its repository-relative file: {marker:#}")
            });
            let line = marker["line"]
                .as_u64()
                .unwrap_or_else(|| panic!("a marker needs its line number: {marker:#}"));
            (file.to_string(), line)
        })
        .collect();
    let mut expected: Vec<_> = [CLASS_RETURN, METHOD_RETURN]
        .into_iter()
        .map(|file| {
            (
                file.to_string(),
                repo.line_of(file, &format!("# Check: {LATE_RETURN}")),
            )
        })
        .collect();
    actual.sort();
    expected.sort();
    assert_eq!(
        actual, expected,
        "markers must identify the two source lines, not the three tests"
    );
}

// behavior-2-follow-up: An entry of `tests` holds `name`, `outcome` and `duration_ms`
#[test]
fn check_show_json_keeps_each_test_name_outcome_and_duration() {
    let repo = return_parts_result();
    let document = repo.check_show(LATE_RETURN);
    let tests = document["tests"].as_array().unwrap_or_else(|| {
        panic!("check-1 lacks tests with name, outcome and duration_ms: {document:#}")
    });
    let mut actual: Vec<_> = tests
        .iter()
        .map(|test| {
            let name = test["name"]
                .as_str()
                .unwrap_or_else(|| panic!("a test needs its name: {test:#}"));
            let outcome = test["outcome"]
                .as_str()
                .unwrap_or_else(|| panic!("a test needs its outcome: {test:#}"));
            let duration = test["duration_ms"]
                .as_u64()
                .unwrap_or_else(|| panic!("a test needs its duration_ms: {test:#}"));
            (name, outcome, duration)
        })
        .collect();
    // Different durations expose a total or a made-up default in place of a test's duration.
    let mut expected: Vec<_> = RETURN_TESTS
        .into_iter()
        .zip([7, 11, 23])
        .map(|((name, outcome), duration)| (name, outcome, duration))
        .collect();
    actual.sort();
    expected.sort();
    assert_eq!(
        actual, expected,
        "tests must retain the names, outcomes and durations of all three tests"
    );
}

// behavior-2-follow-up: For a Check with no result, `tests` is an empty list and the text says that no result exists yet
#[test]
fn check_show_without_a_result_has_no_tests_and_says_so() {
    let repo = return_parts();
    assert!(
        repo.results_of(LATE_RETURN).is_empty(),
        "the fixture already has a result"
    );
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(
        document["tests"],
        json!([]),
        "a Check with no result needs an empty tests list: {document:#}"
    );
    let text = return_parts_text(&repo);
    let lower = text.to_lowercase();
    assert!(
        lower.contains("no result") || lower.contains("not yet run") || lower.contains("never run"),
        "check show must say that no result exists yet:\n{text}"
    );
}

// behavior-2-follow-up: For a Check with no marker, `markers` is an empty list
#[test]
fn check_show_without_a_marker_has_an_empty_marker_list() {
    let repo = return_parts_result();
    // A saved result must not keep a marker alive after it is removed from source.
    for file in [CLASS_RETURN, METHOD_RETURN] {
        repo.replace(file, &format!("# Check: {LATE_RETURN}\n"), "");
    }
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(
        document["markers"],
        json!([]),
        "a Check with no source marker needs an empty markers list: {document:#}"
    );
}
