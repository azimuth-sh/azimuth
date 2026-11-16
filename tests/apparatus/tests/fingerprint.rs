//! Group "The fingerprint" of docs/increments/1.md.

use std::collections::BTreeSet;
use std::fs;

use azimuth_tests::*;

fn all_kept(document: &serde_json::Value) {
    for check in ALL_CHECKS {
        assert_eq!(state_of(document, check), "kept", "{document:#}");
    }
}

fn kept_without_a_launch(repo: &Repo) {
    repo.clear_log();
    all_kept(&repo.run_json(&[], 0));
    assert!(repo.adapter_runs().is_empty(), "the adapter ran although every Check is kept");
}

fn all_owed_in_status(repo: &Repo) {
    let status = repo.status_json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "owed", "{status:#}");
    }
}

// behavior: With no input provider, every file of the repository is an input of every Check
#[test]
fn an_edit_anywhere_makes_every_check_owed() {
    for file in ["desk/returns.txt", "README.md", "dotnet/LibraryDesk/Desk.cs"] {
        let repo = Repo::sample();
        repo.run_json(&[], 0);
        repo.append(file, "\none more line\n");
        all_owed_in_status(&repo);
        repo.clear_log();
        let document = repo.run_json(&[], 0);
        for check in ALL_CHECKS {
            assert_eq!(state_of(&document, check), "passed", "after an edit of {file}: {document:#}");
        }
        assert_eq!(repo.requested_checks(), ALL_CHECKS.iter().map(|c| c.to_string()).collect::<BTreeSet<_>>());
    }
}

// behavior: Declaring a smaller area changes no Check's inputs
#[test]
fn a_smaller_area_does_not_narrow_the_inputs() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "mounts = [\".\"]", "mounts = [\"azimuth\", \"suites\"]");
    repo.commit_all("narrow the area");
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    all_owed_in_status(&repo);
}

// behavior: An edit to a tracked file changes the fingerprint before it is staged or committed
#[test]
fn an_unstaged_edit_moves_the_fingerprint() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    all_owed_in_status(&repo);
    let check = repo.check_show(SIXTH_LOAN);
    assert_eq!(check["state"], "owed");
    assert_eq!(parts_changed(&check), vec!["files"], "{check:#}");
    assert_eq!(files_changed(&check), vec![PRODUCT_FILE.to_string()], "{check:#}");
}

// behavior: A new file that git does not ignore is an input before any `git add`
#[test]
fn a_new_file_is_an_input_before_it_is_added() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    all_owed_in_status(&repo);
    let check = repo.check_show(LATE_RETURN);
    assert_eq!(files_changed(&check), vec!["desk/shelves.txt".to_string()], "{check:#}");
}

// behavior: Staging a change does not move the fingerprint
#[test]
fn staging_keeps_the_fingerprint() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    repo.run_json(&[], 0);
    repo.git(&["add", "-A"]);
    kept_without_a_launch(&repo);
    repo.git(&["commit", "--quiet", "-m", "renewals"]);
    kept_without_a_launch(&repo);
}

// behavior: An edit to a file that git ignores moves no fingerprint
#[test]
fn an_ignored_file_moves_no_fingerprint() {
    let repo = Repo::sample();
    repo.write("desk/settings.local", "loan limit = 6\n");
    repo.run_json(&[], 0);
    repo.write("desk/settings.local", "loan limit = 7\n");
    repo.write("desk/more.local", "anything\n");
    kept_without_a_launch(&repo);
}

/// A clone of the repository with the local store copied into it, so results made in one
/// checkout can be read in another.
fn clone_with_store(repo: &Repo, name: &str, extra: &[&str]) -> std::path::PathBuf {
    let target = repo.beside(name);
    let mut args = vec!["clone", "--quiet"];
    args.extend_from_slice(extra);
    let source = repo.root().to_string_lossy().to_string();
    let destination = target.to_string_lossy().to_string();
    args.push(&source);
    args.push(&destination);
    repo.git_in(&repo.beside(""), &args);
    copy_tree(&repo.store(), &target.join(".git/azimuth"));
    fs::create_dir_all(target.join(".azimuth")).unwrap();
    fs::write(target.join(".azimuth/.gitignore"), "*\n").unwrap();
    target
}

fn kept_in(repo: &Repo, dir: &std::path::Path) {
    let output = repo.azimuth_in(dir, &["status", "--json"], &[]);
    output.expect_status(0);
    let status = output.json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "kept", "{status:#}");
    }
}

// behavior: A clean checkout of a commit gives the digests of claim and files that the working tree had when it held exactly that content
#[test]
fn a_clean_checkout_matches_the_working_tree_it_was_committed_from() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    repo.run_json(&[], 0);
    repo.commit_all("renewals and shelves");
    let clone = clone_with_store(&repo, "clean-checkout", &[]);
    kept_in(&repo, &clone);
}

// behavior: A file is hashed in the form git stores, so a checkout with other line endings gives an equal fingerprint
#[test]
fn a_checkout_with_windows_line_endings_has_an_equal_fingerprint() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let clone = clone_with_store(&repo, "crlf-checkout", &["-c", "core.autocrlf=true"]);
    let on_disk = fs::read(clone.join(PRODUCT_FILE)).unwrap();
    assert!(on_disk.windows(2).any(|pair| pair == b"\r\n"), "the clone did not convert line endings");
    kept_in(&repo, &clone);
}

fn claim_parts_after(prepare: &str, edit: &str) -> BTreeSet<String> {
    let repo = Repo::sample();
    repo.replace(LENDING_VERIFICATION, "  The desk refuses a sixth loan to a borrower who holds five.\n", prepare);
    repo.commit_all("claim");
    repo.run_json(&[], 0);
    let text = repo.read(LENDING_VERIFICATION).replace(prepare, edit);
    repo.write(LENDING_VERIFICATION, &text);
    parts_changed_set(&repo.check_show(SIXTH_LOAN))
}

// behavior: A claim that differs only in Unicode form, in line endings, or in spaces and tabs at line ends has an equal digest
#[test]
fn a_claim_in_another_unicode_form_has_an_equal_digest() {
    // "é" composed in the first claim, "e" with a combining accent in the second.
    let parts = claim_parts_after(
        "  The desk refuses a sixth loan at the caf\u{e9} counter.\n",
        "  The desk refuses a sixth loan at the cafe\u{301} counter.\n",
    );
    assert_eq!(parts, set_of(&["files"]));
}

// behavior: A claim that differs only in Unicode form, in line endings, or in spaces and tabs at line ends has an equal digest
#[test]
fn a_claim_with_trailing_spaces_and_windows_line_endings_has_an_equal_digest() {
    let parts = claim_parts_after(
        "  The desk refuses a sixth loan\n  to a borrower who holds five.\n",
        "  The desk refuses a sixth loan   \r\n  to a borrower who holds five.  \r\n",
    );
    assert_eq!(parts, set_of(&["files"]));
}

// behavior: A claim that differs only in Unicode form, in line endings, or in spaces and tabs at line ends has an equal digest
#[test]
fn a_claim_with_tabs_at_line_ends_has_an_equal_digest() {
    let parts = claim_parts_after(
        "  The desk refuses a sixth loan\n  to a borrower who holds five.\n",
        "  The desk refuses a sixth loan\t\n  to a borrower who holds five. \t \n",
    );
    assert_eq!(parts, set_of(&["files"]));
}

// behavior: A claim that is re-wrapped or restyled has a different digest
#[test]
fn a_rewrapped_claim_has_a_different_digest() {
    let parts = claim_parts_after(
        "  The desk refuses a sixth loan\n  to a borrower who holds five.\n",
        "  The desk refuses a sixth loan to a borrower\n  who holds five.\n",
    );
    assert_eq!(parts, set_of(&["claim", "files"]));
}

// behavior: A claim that is re-wrapped or restyled has a different digest
#[test]
fn a_restyled_claim_has_a_different_digest() {
    let parts = claim_parts_after(
        "  The desk refuses a sixth loan to a borrower who holds five.\n",
        "  The desk refuses a **sixth** loan to a borrower who holds five.\n",
    );
    assert_eq!(parts, set_of(&["claim", "files"]));
}

// behavior: Without a provider the fingerprint has the parts claim, files, environment and method
#[test]
fn the_fingerprint_has_four_parts() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        let parts = repo.result_of(check)["body"]["parts"].clone();
        let names: BTreeSet<String> = parts.as_object().unwrap_or_else(|| panic!("no parts: {parts:#}")).keys().cloned().collect();
        assert_eq!(names, set_of(&PARTS), "{check}");
    }
    repo.replace(LENDING_VERIFICATION, "The desk refuses a sixth loan", "The desk turns down a sixth loan");
    let changed = parts_changed_set(&repo.check_show(SIXTH_LOAN));
    assert!(changed.is_subset(&set_of(&PARTS)), "{changed:?}");
    assert_eq!(changed, set_of(&["claim", "files"]));
}

// behavior: For a tree that is exactly a commit, the store keeps the id of that commit's tree and no list
#[test]
fn a_clean_tree_is_stored_as_its_tree_id() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
    for check in ALL_CHECKS {
        assert_eq!(repo.result_of(check)["body"]["tree"]["base"], tree.as_str(), "{check}");
    }
    let strings = repo.record_strings();
    let tracked: BTreeSet<String> = repo.tracked_paths().into_iter().collect();
    let listed: Vec<&String> = strings.iter().filter(|text| tracked.contains(*text)).collect();
    assert!(listed.is_empty(), "the store keeps a list of files for a clean tree: {listed:?}");
}

// behavior: For a commit with local edits, the store keeps that tree id plus one entry per edited, new or deleted path
#[test]
fn a_tree_with_local_edits_is_stored_as_the_tree_id_and_its_differences() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    repo.remove("desk/returns.txt");
    repo.run_json(&[], 0);
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
    for check in ALL_CHECKS {
        let stored = repo.result_of(check)["body"]["tree"].clone();
        assert_eq!(stored["base"], tree.as_str(), "{check}: {stored:#}");
        let paths: BTreeSet<String> = stored["edits"]
            .as_array()
            .unwrap_or_else(|| panic!("no edits: {stored:#}"))
            .iter()
            .map(|edit| edit["path"].as_str().unwrap_or_default().to_string())
            .collect();
        assert_eq!(paths, set_of(&[PRODUCT_FILE, "desk/shelves.txt", "desk/returns.txt"]), "{check}: {stored:#}");
    }
}

/// A result stored for the tree of a commit that git then forgets, with the working tree holding
/// the same content again as uncommitted edits on `main`.
fn result_on_a_forgotten_tree() -> Repo {
    let repo = Repo::sample();
    repo.git(&["checkout", "--quiet", "-b", "side"]);
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.commit_all("renewals");
    let content = repo.read(PRODUCT_FILE);
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
    repo.run_json(&[], 0);
    repo.git(&["checkout", "--quiet", "main"]);
    repo.write(PRODUCT_FILE, &content);
    repo.git(&["branch", "--quiet", "-D", "side"]);
    repo.git(&["reflog", "expire", "--expire=now", "--all"]);
    repo.git(&["gc", "--quiet", "--prune=now"]);
    let gone = repo.git_output_in(repo.root(), &["cat-file", "-e", &tree]);
    assert_ne!(gone.status, 0, "git still has the tree, so the test proves nothing");
    repo
}

// behavior: Kept or owed is decided without reading a list of files
#[test]
fn kept_is_decided_when_git_no_longer_has_the_stored_tree() {
    let repo = result_on_a_forgotten_tree();
    let status = repo.status_json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "kept", "{status:#}");
    }
}

// behavior: When git no longer has the stored tree, the reason says "files changed" and names no file, and no state changes
#[test]
fn a_forgotten_tree_shrinks_the_explanation_only() {
    let repo = result_on_a_forgotten_tree();
    repo.append("desk/returns.txt", "fines are rounded up\n");
    let check = repo.check_show(LATE_RETURN);
    assert_eq!(check["state"], "owed", "{check:#}");
    assert_eq!(check["reason"], "an input changed", "{check:#}");
    assert_eq!(check["detail"], "files changed", "{check:#}");
    assert_eq!(check["files_changed"], json!([]), "{check:#}");
    assert_eq!(parts_changed(&check), vec!["files"], "{check:#}");
    let status = repo.status_json();
    assert_eq!(status_checks(&status)[LATE_RETURN]["state"], "owed");
}

// behavior: On a developer's machine the environment part is one fixed value
#[test]
fn the_environment_of_the_machine_is_not_measured() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.clear_log();
    let output = repo.azimuth_with_env(
        &["run", "--json"],
        &[("LANG", "kk_KZ.UTF-8"), ("TZ", "Asia/Almaty"), ("USER", "someone-else"), ("DOTNET_ROOT", "/opt/elsewhere")],
    );
    output.expect_status(0);
    all_kept(&output.json());
    assert!(repo.adapter_runs().is_empty());
}

// behavior: A `text-1` digest is the SHA-256 of the normalized text, written as lowercase hex
#[test]
fn a_text_1_digest_is_lowercase_sha_256() {
    // A claim's exact text is the reader's business, so the full computation is checked on the
    // digest line of an installed skill, whose text is the whole file without that line.
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        let claim = repo.result_of(check)["body"]["parts"]["claim"].as_str().unwrap_or_default().to_string();
        let hex = claim.strip_prefix("text-1:").unwrap_or_else(|| panic!("{check}: {claim}"));
        assert!(hex.len() == 64 && hex.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)), "{check}: {claim}");
    }
    repo.azimuth(&["skills", "install", "codex"]).expect_status(0);
    let (text, last) = split_last_line(&repo.read(SKILL_FILE));
    assert_eq!(last, skill_digest_line(&text));
}

// behavior: The method part holds the scheme names and the object format of the repository
#[test]
fn the_method_names_the_schemes_and_the_object_format() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        let method = repo.result_of(check)["body"]["method"].clone();
        assert_eq!(method["schemes"], json!(["file-1", "parts-1", "text-1"]), "{check}: {method:#}");
        assert_eq!(method["object_format"], "sha1", "{check}: {method:#}");
    }
}

// behavior: The method part holds the scheme names and the object format of the repository
#[test]
fn the_object_format_of_a_sha_256_repository_is_recorded() {
    let repo = Repo::sample();
    let other = repo.beside("sha256-desk");
    copy_tree(&sample_source(), &other);
    pin_to_the_cli(&other);
    std::fs::create_dir_all(other.join(".azimuth")).unwrap();
    std::fs::write(other.join(".azimuth/.gitignore"), "*\n").unwrap();
    let init = repo.git_output_in(&other, &["init", "--quiet", "--object-format=sha256", "-b", "main"]);
    assert_eq!(init.status, 0, "this git cannot make a SHA-256 repository: {}", init.stderr);
    repo.git_in(&other, &["add", "-A"]);
    repo.git_in(&other, &["commit", "--quiet", "-m", "sample"]);
    repo.azimuth_in(&other, &["run", "--json"], &[]).expect_status(0);
    let records = json_files_under(&other.join(".git/azimuth"));
    assert_eq!(records.len(), 4, "{records:?}");
    for file in records {
        let record: serde_json::Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        assert_eq!(record["body"]["method"]["object_format"], "sha256", "{record:#}");
    }
}

// behavior: The digest of a file is the object id git gives its content
#[test]
fn a_file_digest_is_the_git_object_id() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.run_json(&[], 0);
    let id = repo.git(&["hash-object", PRODUCT_FILE]).trim().to_string();
    let edits = repo.result_of(SIXTH_LOAN)["body"]["tree"]["edits"].clone();
    assert_eq!(edits, json!([{ "path": PRODUCT_FILE, "mode": "100644", "id": format!("file-1:{id}") }]));
}

// behavior: For an edited or a new file the id is that of the content in the working tree, and nothing is written into git's object store
#[test]
fn hashing_the_working_tree_writes_no_git_object() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    let edited = repo.git(&["hash-object", PRODUCT_FILE]).trim().to_string();
    let new = repo.git(&["hash-object", "desk/shelves.txt"]).trim().to_string();
    let objects_before = repo.git(&["count-objects", "-v"]);
    repo.run_json(&[], 0);
    repo.azimuth(&["status", "--json"]).expect_status(0);
    assert_eq!(repo.git(&["count-objects", "-v"]), objects_before, "a command wrote into git's object store");
    let ids: BTreeSet<String> = repo.result_of(SIXTH_LOAN)["body"]["tree"]["edits"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|edit| edit["id"].as_str().unwrap_or_default().to_string())
        .collect();
    for id in [&edited, &new] {
        assert_ne!(repo.git_output_in(repo.root(), &["cat-file", "-e", id]).status, 0, "git stores {id}");
        assert!(ids.contains(&format!("file-1:{id}")), "no edit holds {id}: {ids:?}");
    }
}

// behavior: A change of only a file's mode changes the files part
#[test]
fn a_mode_change_moves_the_files_part() {
    use std::os::unix::fs::PermissionsExt;
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    fs::set_permissions(repo.path(PRODUCT_FILE), fs::Permissions::from_mode(0o755)).unwrap();
    let check = repo.check_show(SIXTH_LOAN);
    assert_eq!(check["state"], "owed", "{check:#}");
    assert_eq!(parts_changed(&check), vec!["files"], "{check:#}");
    assert_eq!(files_changed(&check), vec![PRODUCT_FILE.to_string()], "{check:#}");
}

// behavior: A digest is written as its scheme name, a colon and the value
#[test]
fn digests_are_written_with_their_scheme() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.run_json(&[], 0);
    let record = repo.result_of(SIXTH_LOAN);
    let body = &record["body"];
    let mut digests = vec![record["subjects"][0]["digest"].clone(), body["fingerprint"].clone(), body["tree"]["edits"][0]["id"].clone()];
    digests.extend(PARTS.iter().map(|part| body["parts"][*part].clone()));
    for digest in digests {
        assert!(digest.as_str().is_some_and(is_digest), "{digest} is not a scheme, a colon and a value: {record:#}");
    }
}

// behavior: The digest of model text names `text-1`, of a file `file-1`, and a fingerprint `parts-1`
#[test]
fn the_three_schemes_name_their_digests() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.run_json(&[], 0);
    let body = repo.result_of(SIXTH_LOAN)["body"].clone();
    for (digest, scheme) in [
        (&body["parts"]["claim"], "text-1:"),
        (&body["tree"]["edits"][0]["id"], "file-1:"),
        (&body["fingerprint"], "parts-1:"),
    ] {
        assert!(digest.as_str().is_some_and(|text| text.starts_with(scheme) && is_digest(text)), "{digest} is not {scheme}: {body:#}");
    }
}

/// The sample after one run, with every result rewritten as a CLI of the scheme `parts-2` would
/// have written it.
fn result_under_parts_2() -> Repo {
    fn walk(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(text) if text.starts_with("parts-1:") || text == "parts-1" => {
                *text = text.replacen("parts-1", "parts-2", 1);
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(walk),
            serde_json::Value::Object(map) => map.values_mut().for_each(walk),
            _ => {}
        }
    }
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.rewrite_records(|record| walk(&mut record["body"]));
    assert!(repo.result_of(SIXTH_LOAN)["body"]["fingerprint"].as_str().unwrap().starts_with("parts-2:"));
    repo
}

// behavior: A stored result whose digests name another scheme number is not compared, and its Check is owed
#[test]
fn a_result_under_another_scheme_number_is_owed() {
    let repo = result_under_parts_2();
    all_owed_in_status(&repo);
}

// behavior: The owed reason names the scheme that changed
#[test]
fn the_owed_reason_names_the_changed_scheme() {
    let repo = result_under_parts_2();
    let check = repo.check_show(SIXTH_LOAN);
    assert_eq!(check["state"], "owed", "{check:#}");
    assert_eq!(check["reason"], "an input changed", "{check:#}");
    let detail = detail_of_entry(&check);
    assert!(detail.contains("parts-1") && detail.contains("parts-2"), "{check:#}");
}

// behavior: No setting and no flag chooses a scheme
#[test]
fn no_flag_chooses_a_scheme() {
    let repo = Repo::sample();
    repo.azimuth(&["run", "--scheme", "parts-2"]).expect_status(2);
    repo.azimuth(&["status", "--scheme", "parts-2"]).expect_status(2);
    assert!(repo.adapter_starts().is_empty());
}

// behavior: No setting and no flag chooses a scheme
#[test]
fn no_setting_chooses_a_scheme() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "[repository]\n", "scheme = \"parts-2\"\n\n[repository]\n");
    repo.expect_reading_error(Some(PROJECT_FILE), None);
}

// behavior: Every stored digest names its scheme
#[test]
fn no_record_holds_a_bare_hash() {
    // A commit id is information, not a digest, and a tree id is git's own name for a tree; both
    // may stand bare. Any other bare hash would be a digest without its scheme.
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.run_json(&[], 0);
    let commit = repo.head();
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
    let bare: Vec<String> = repo
        .record_strings()
        .into_iter()
        .filter(|text| is_bare_hash(text) && text != &commit && text != &tree)
        .collect();
    assert!(bare.is_empty(), "digests without a scheme: {bare:?}");
}

// behavior: A result stores the digest of each part of its fingerprint
#[test]
fn a_result_stores_a_digest_per_part() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let parts = repo.result_of(SIXTH_LOAN)["body"]["parts"].clone();
    for part in PARTS {
        assert!(parts[part].as_str().is_some_and(is_digest), "no digest of {part}: {parts:#}");
    }
}

// behavior: The adapter's reported version is stored with the result and moves no fingerprint
#[test]
fn a_new_adapter_version_is_recorded_and_moves_no_fingerprint() {
    let mut repo = Repo::sample();
    repo.run_json(&[], 0);
    let before = repo.result_of(RETURNED_BOOK);
    assert_eq!(before["body"]["method"]["adapter"]["version"], "1.0.0", "{before:#}");

    repo.set_env("TEXT_ADAPTER_VERSION", "2.0.0");
    kept_without_a_launch(&repo);
    repo.azimuth(&["check", "run", SIXTH_LOAN, "--json"]).expect_status(0);
    let versions: BTreeSet<String> = repo
        .results_of(SIXTH_LOAN)
        .iter()
        .map(|(_, record)| record["body"]["method"]["adapter"]["version"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(versions, set_of(&["1.0.0", "2.0.0"]));
    let fingerprints: BTreeSet<String> = repo
        .results_of(SIXTH_LOAN)
        .iter()
        .map(|(_, record)| record["body"]["fingerprint"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(fingerprints.len(), 1, "the adapter's version moved the fingerprint: {fingerprints:?}");
    kept_without_a_launch(&repo);
}

// behavior: Restoring earlier content makes the earlier pass valid again
#[test]
fn restoring_content_brings_the_pass_back() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let failed = repo.run_json(&[], 1);
    assert_eq!(state_of(&failed, LATE_RETURN), "failed");
    repo.replace(LATE_FINE_TESTS, "outcome: failed", "outcome: passed");
    kept_without_a_launch(&repo);
}

// behavior: No record names a commit as its identity
#[test]
fn a_rewritten_commit_keeps_its_results() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let before = repo.head();
    repo.git(&["commit", "--quiet", "--amend", "-m", "the same tree under another commit"]);
    assert_ne!(repo.head(), before);
    kept_without_a_launch(&repo);
    repo.git(&["checkout", "--quiet", "--orphan", "fresh-history"]);
    repo.git(&["commit", "--quiet", "-m", "the same tree with no parent"]);
    kept_without_a_launch(&repo);
}
