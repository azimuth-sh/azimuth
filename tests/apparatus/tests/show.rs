//! Group "`azimuth requirement show` and `azimuth check show`" of docs/increments/1.md.

use azimuth_tests::*;

const OWED_REASONS: [&str; 9] = [
    "an input changed",
    "never run",
    "deferred by the lane",
    "cannot run here",
    "failed on these inputs",
    "inconclusive",
    "contradicted",
    "the start-up guard is owed",
    "expired",
];

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
        assert!(text.contains(expected), "requirement show lacks {expected:?}:\n{text}");
    }
    repo.azimuth(&["requirement", "show", "no-such-requirement"]).expect_status(2);
}

// behavior: `azimuth check show <id>` prints the Check's state and the reason
#[test]
fn check_show_prints_state_and_reason() {
    let repo = known_fail();
    let output = repo.azimuth(&["check", "show", LATE_RETURN]);
    output.expect_status(0);
    assert!(output.stdout.contains("owed") && output.stdout.contains("failed on these inputs"), "{}", output.stdout);
    repo.azimuth(&["check", "show", "no-such-check"]).expect_status(2);
}

// behavior: The document of `azimuth requirement show` is `requirement-1`: one Requirement with the fields it has in `status-1`
#[test]
fn requirement_document_matches_its_status_entry() {
    let repo = known_fail();
    let output = repo.azimuth(&["requirement", "show", LATE_FINE, "--json"]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "requirement-1");
    let status = repo.status_json();
    let entry = requirement_in(&status, LATE_FINE).unwrap();
    for field in ["id", "module", "state", "verification", "realization", "approval", "scenarios"] {
        let shown = document.get(field).or_else(|| document["requirement"].get(field));
        assert_eq!(shown, entry.get(field), "`{field}` differs from status-1:\n{document:#}\n{entry:#}");
    }
}

// behavior: The document of `azimuth check show` is `check-1` with these fields
#[test]
fn check_document_has_its_fields() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let document = repo.check_show(SIXTH_LOAN);
    for field in [
        "id", "requirement", "scenario", "suite", "claim", "reuse", "state", "reason", "parts_changed", "files_changed",
        "failing_tests", "result",
    ] {
        assert!(document.get(field).is_some(), "no `{field}`: {document:#}");
    }
    for field in ["where", "commit", "at"] {
        assert!(document["result"].get(field).is_some(), "no `result.{field}`: {document:#}");
    }
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
    assert!(!result["where"].as_str().unwrap_or_default().is_empty(), "{document:#}");
    let commit = result["commit"].as_str().unwrap_or_default();
    assert!(commit.len() >= 7 && head.starts_with(commit), "{document:#}");
    assert!(!result["at"].as_str().unwrap_or_default().is_empty(), "{document:#}");
    let text = repo.azimuth(&["check", "show", SIXTH_LOAN]).stdout;
    assert!(text.contains(&head[..7]), "{text}");
    assert!(text.contains("ago"), "{text}");
}

// behavior: A failed Check is shown with its failing tests
#[test]
fn a_failed_check_is_shown_with_its_failing_tests() {
    let repo = known_fail();
    let document = repo.check_show(LATE_RETURN);
    assert_eq!(strings_of(&document["failing_tests"]), vec!["late-fine"], "{document:#}");
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
    for (repo, check) in [(&changed, SIXTH_LOAN), (&failing, LATE_RETURN)] {
        let document = repo.check_show(check);
        let reason = document["reason"].as_str().unwrap_or_default();
        assert!(OWED_REASONS.iter().any(|named| reason.starts_with(named)), "{reason:?} is not a named reason");
    }
}

// behavior: When the files part changed, `azimuth check show` names the files that differ
#[test]
fn check_show_names_the_files_that_differ() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    let document = repo.check_show(ON_TIME_RETURN);
    let mut files = files_changed(&document);
    files.sort();
    assert_eq!(files, vec![PRODUCT_FILE.to_string(), "desk/shelves.txt".to_string()], "{document:#}");
    let text = repo.azimuth(&["check", "show", ON_TIME_RETURN]).stdout;
    assert!(text.contains(PRODUCT_FILE) && text.contains("desk/shelves.txt"), "{text}");
}

// behavior: The reason "an input changed" names the part that changed
#[test]
fn an_input_changed_names_its_part() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    let document = repo.check_show(SIXTH_LOAN);
    assert!(document["reason"].as_str().unwrap_or_default().starts_with("an input changed"), "{document:#}");
    let parts = parts_changed(&document);
    assert!(mentions(&parts, "files") && !mentions(&parts, "claim"), "{parts:?}");
    let text = repo.azimuth(&["check", "show", SIXTH_LOAN]).stdout;
    assert!(text.contains("files"), "{text}");
}

// behavior: A pass and a fail on one fingerprint are shown as contradicted
#[test]
fn a_pass_and_a_fail_on_one_fingerprint_are_contradicted() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let output = repo.azimuth_with_env(&["check", "run", LATE_RETURN, "--json"], &[("TEXT_ADAPTER_FAIL", "late-fine")]);
    output.expect_status(1);
    let document = repo.check_show(LATE_RETURN);
    let words = format!("{} {}", document["state"], document["reason"]);
    assert!(words.contains("contradicted"), "{document:#}");
    let status = repo.status_json();
    assert!(status_checks(&status)[LATE_RETURN].to_string().contains("contradicted"), "{status:#}");
}
