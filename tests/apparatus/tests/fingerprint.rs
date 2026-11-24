//! Group "The fingerprint" of docs/increments/1.md.

use std::collections::BTreeSet;
use std::fs;

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
use contexts_conditions::*;

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

// Conditions in the model: `docs/increments/contexts-conditions.md`. A stored result of a Check
// has the claim part the Check had before it carried a condition. Each test rewrites that part as
// the design computes it for a condition (`claim_is`) and reads whether `azimuth check show`
// names the claim among the changed parts. Every edit of the model also moves the files part,
// because with no provider every file is an input, so `files` is expected beside it.

const INSTANT_TRUE: (&str, &str) = ("payouts.instant", "true");

fn with_conditions(on_requirement_lines: &[&str], on_reaches_lines: &[&str]) -> Repo {
    let repo = payouts_with_results();
    if !on_requirement_lines.is_empty() {
        on_requirement(&repo, on_requirement_lines);
    }
    if !on_reaches_lines.is_empty() {
        on_reaches(&repo, on_reaches_lines);
    }
    repo
}

/// A one-of or number key under the Requirement, in the spelling written, reads as the normalized
/// value `normalized` and as no other.
fn requirement_condition_reads_as(key: &str, written: &str, normalized: &str, other: &str) {
    let repo = with_conditions(&[&format!("Context: {key}={written}")], &[]);
    lint_is_clean(&repo);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[(key, normalized)]);
    claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[(key, other)]);
    claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[]);
}

// behavior-contexts-conditions: A Scenario takes its Requirement's condition
#[test]
fn a_scenario_takes_its_requirements_condition() {
    let repo = with_conditions(&["Context: payouts.instant=true"], &[]);
    for (check, claim) in [(REACHES_THE_CARD, REACHES_CLAIM), (WAITS_FOR_REVIEW, WAITS_CLAIM)] {
        claim_is(&repo, check, claim, &[INSTANT_TRUE]);
        claim_is_not(&repo, check, claim, &[]);
    }
}

// behavior-contexts-conditions: A Scenario may add a key to its Requirement's condition
#[test]
fn a_scenario_may_add_a_key() {
    let repo = with_conditions(&["Context: payouts.instant=true"], &["Context: payouts.region=\"no\""]);
    lint_is_clean(&repo);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[INSTANT_TRUE, ("payouts.region", "\"no\"")]);
    claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[INSTANT_TRUE]);
    // The sibling Scenario does not take what its sibling added.
    claim_is(&repo, WAITS_FOR_REVIEW, WAITS_CLAIM, &[INSTANT_TRUE]);
}

// behavior-contexts-conditions: A Scenario with no `Context:` of its own keeps its Requirement's condition
#[test]
fn a_scenario_with_no_context_of_its_own_keeps_its_requirements_condition() {
    let repo = with_conditions(&["Context: payouts.region=\"se\""], &["Context: payouts.limit=5"]);
    claim_is(&repo, WAITS_FOR_REVIEW, WAITS_CLAIM, &[("payouts.region", "\"se\"")]);
    claim_is_not(&repo, WAITS_FOR_REVIEW, WAITS_CLAIM, &[("payouts.region", "\"se\""), ("payouts.limit", "5")]);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.region", "\"se\""), ("payouts.limit", "5")]);
}

// behavior-contexts-conditions: Repeating an inherited value changes nothing
#[test]
fn repeating_an_inherited_value_changes_nothing() {
    for repeated in ["Context: payouts.instant=true", "Context: payouts.instant=TRUE", "Context: payouts.instant =\ttrue"] {
        let repo = with_conditions(&["Context: payouts.instant=true"], &[repeated]);
        lint_is_clean(&repo);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[INSTANT_TRUE]);
        claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.instant=true\nContext: payouts.instant=true"));
    }
}

// behavior-contexts-conditions: Repeating a key with the same normalized value on one heading changes nothing
#[test]
fn repeating_a_key_with_the_same_normalized_value_changes_nothing() {
    let repo = with_conditions(
        &[
            "Context: payouts.limit=1",
            "Context: payouts.limit=1.0",
            "Context: payouts.instant=true",
            "Context: payouts.instant=True",
            "Context: payouts.region=\"\u{e9}\"",
            "Context: payouts.region=\"e\u{301}\"",
        ],
        &[],
    );
    lint_is_clean(&repo);
    let condition = [("payouts.limit", "1"), INSTANT_TRUE, ("payouts.region", "\"\u{e9}\"")];
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &condition);
    let scenario = with_conditions(&[], &["Context: payouts.limit=1", "Context: payouts.limit=1.0", "Context: payouts.limit=1e0"]);
    lint_is_clean(&scenario);
    claim_is(&scenario, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.limit", "1")]);
    claim_is(&scenario, WAITS_FOR_REVIEW, WAITS_CLAIM, &[]);
}

// behavior-contexts-conditions: A Scenario with no condition of its own or its Requirement's has no condition
#[test]
fn a_scenario_with_no_condition_anywhere_has_no_condition() {
    let repo = payouts_with_results();
    for (check, claim) in [(REACHES_THE_CARD, REACHES_CLAIM), (WAITS_FOR_REVIEW, WAITS_CLAIM)] {
        assert_eq!(result_claim(&repo, check), text_1_digest(claim), "{check}");
    }
    // A condition on one Scenario does not reach the Requirement's other Scenario or its Checks.
    let one = with_conditions(&[], &["Context: payouts.instant=true"]);
    claim_is(&one, REACHES_THE_CARD, REACHES_CLAIM, &[INSTANT_TRUE]);
    claim_is(&one, WAITS_FOR_REVIEW, WAITS_CLAIM, &[]);
    // A Check with no condition is promised everywhere: it runs, and passes, with the others owed.
    let document = one.run_json(&[], 1);
    assert_eq!(state_of(&document, WAITS_FOR_REVIEW), "passed", "{document:#}");
}

// behavior-contexts-conditions: Boolean text is accepted without regard to ASCII case
#[test]
fn boolean_text_is_accepted_in_any_ascii_case() {
    for written in ["TRUE", "True", "tRuE", "true"] {
        let repo = with_conditions(&[&format!("Context: payouts.instant={written}")], &[]);
        lint_is_clean(&repo);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[INSTANT_TRUE]);
    }
    for written in ["FALSE", "False", "fAlSe"] {
        let repo = with_conditions(&[&format!("Context: payouts.instant={written}")], &[]);
        lint_is_clean(&repo);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.instant", "false")]);
        claim_is_not(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[INSTANT_TRUE]);
    }
}

// behavior-contexts-conditions: Boolean text normalizes to lowercase
#[test]
fn boolean_text_normalizes_to_lowercase() {
    let repo = with_conditions(&["Context: payouts.instant=TRUE"], &[]);
    claim_is_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.instant=true"));
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.instant=TRUE"));
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.instant=True"));
}

// behavior-contexts-conditions: A one-of value is decoded from its JSON string
#[test]
fn a_one_of_value_is_decoded_from_its_json_string() {
    for written in ["\"\\u0073e\"", "\"s\\u0065\"", "\"\\u0073\\u0065\"", "\"se\""] {
        requirement_condition_reads_as("payouts.region", written, "\"se\"", "\"no\"");
    }
    requirement_condition_reads_as("payouts.region", "\"\\u00e9\"", "\"\u{e9}\"", "\"\u{c9}\"");
    // The quotes are the string's, not part of its value: the digest spells them again.
    let repo = with_conditions(&["Context: payouts.region=\"se\""], &[]);
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.region=se"));
}

// behavior-contexts-conditions: A one-of value compares equal to its declared string in another Unicode form after NFC normalization
#[test]
fn a_one_of_value_in_another_unicode_form_equals_the_declared_string() {
    let composed = "\"\u{e9}\"";
    // Declared composed, written decomposed.
    let repo = with_conditions(&["Context: payouts.region=\"e\u{301}\""], &[]);
    lint_is_clean(&repo);
    claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.region", composed)]);
    // Declared decomposed, written composed and decomposed.
    for written in ["\"\u{e9}\"", "\"e\u{301}\"", "\"e\\u0301\""] {
        let repo = payouts();
        repo.write(
            PAYOUT_KEY_FILE,
            &payout_keys().replace("values = [\"no\", \"se\", \"\u{e9}\", \"\u{c9}\"]", "values = [\"no\", \"se\", \"e\\u0301\"]"),
        );
        repo.commit_all("keys with a decomposed value");
        repo.run_json(&[], 0);
        on_requirement(&repo, &[&format!("Context: payouts.region={written}")]);
        lint_is_clean(&repo);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.region", composed)]);
    }
}

// behavior-contexts-conditions: A one-of value with another case is not folded to a declared string
#[test]
fn a_one_of_value_with_another_case_is_not_folded() {
    for written in ["\"NO\"", "\"No\"", "\"SE\""] {
        let repo = with_conditions(&[&format!("Context: payouts.region={written}")], &[]);
        lint_names_the_specification(&repo);
    }
    // `é` and `É` are two declared values, and each keeps its own claim part.
    requirement_condition_reads_as("payouts.region", "\"\u{c9}\"", "\"\u{c9}\"", "\"\u{e9}\"");
    requirement_condition_reads_as("payouts.region", "\"\u{e9}\"", "\"\u{e9}\"", "\"\u{c9}\"");
}

// behavior-contexts-conditions: A number is read in JSON decimal notation
#[test]
fn a_number_is_read_in_json_decimal_notation() {
    for (written, normalized) in [
        ("12.5", "12.5"),
        ("-3", "-3"),
        ("0", "0"),
        ("1e3", "1000"),
        ("1E+3", "1000"),
        ("2.5e-1", "0.25"),
        ("0.25", "0.25"),
        ("-1.25E2", "-125"),
    ] {
        requirement_condition_reads_as("payouts.limit", written, normalized, "7");
    }
}

// behavior-contexts-conditions: Numbers compare as exact decimals
#[test]
fn numbers_compare_as_exact_decimals() {
    // Each pair is one number in a floating-point approximation and two numbers exactly.
    for (written, other) in [
        ("0.30000000000000004", "0.3"),
        ("9007199254740993", "9007199254740992"),
        ("0.1000000000000000055511151231257827", "0.1"),
        ("1.0000000000000000000000001", "1"),
        ("123456789012345678901234567890", "123456789012345678901234567891"),
    ] {
        requirement_condition_reads_as("payouts.limit", written, written, other);
    }
    // And equal exactly in any spelling.
    for (written, equal) in [("0.10", "0.1"), ("1.0", "1"), ("1e2", "100"), ("100e-2", "1"), ("1e30", "1000000000000000000000000000000")] {
        requirement_condition_reads_as("payouts.limit", written, equal, "2");
    }
}

// behavior-contexts-conditions: A normalized number has no exponent
#[test]
fn a_normalized_number_has_no_exponent() {
    for (written, normalized) in [("1.5e2", "150"), ("1e-3", "0.001"), ("12E+1", "120"), ("5e0", "5"), ("-2.5e-2", "-0.025"), ("1e-30", "0.000000000000000000000000000001")] {
        requirement_condition_reads_as("payouts.limit", written, normalized, "9");
        let repo = with_conditions(&[&format!("Context: payouts.limit={written}")], &[]);
        claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.limit={written}"));
    }
}

// behavior-contexts-conditions: A normalized number has no unnecessary leading zeros
#[test]
fn a_normalized_number_has_no_unnecessary_leading_zeros() {
    for (written, normalized) in [("0.0001e2", "0.01"), ("0.0100e2", "1"), ("0.5e1", "5"), ("0.00123e3", "1.23"), ("-0.002e1", "-0.02")] {
        requirement_condition_reads_as("payouts.limit", written, normalized, "0.1");
        let repo = with_conditions(&[&format!("Context: payouts.limit={written}")], &[]);
        claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.limit={written}"));
    }
    // The zero before the decimal point of a fraction is necessary and stays.
    let repo = with_conditions(&["Context: payouts.limit=0.5"], &[]);
    claim_is_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.limit=0.5"));
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.limit=.5"));
}

// behavior-contexts-conditions: A normalized number has no fractional trailing zeros
#[test]
fn a_normalized_number_has_no_fractional_trailing_zeros() {
    for (written, normalized) in [("2.50", "2.5"), ("0.100", "0.1"), ("1.2300", "1.23"), ("-7.10", "-7.1"), ("1.2300e0", "1.23")] {
        requirement_condition_reads_as("payouts.limit", written, normalized, "2.51");
        let repo = with_conditions(&[&format!("Context: payouts.limit={written}")], &[]);
        claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.limit={written}"));
    }
}

// behavior-contexts-conditions: A normalized whole number has no decimal point
#[test]
fn a_normalized_whole_number_has_no_decimal_point() {
    for (written, normalized) in [("3.0", "3"), ("10.00", "10"), ("1e1", "10"), ("5.0e0", "5"), ("-4.0", "-4"), ("100.000", "100")] {
        requirement_condition_reads_as("payouts.limit", written, normalized, "4");
        let repo = with_conditions(&[&format!("Context: payouts.limit={written}")], &[]);
        claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.limit={written}"));
    }
    // Trailing zeros of a whole number are digits, and stay.
    requirement_condition_reads_as("payouts.limit", "100", "100", "1");
}

// behavior-contexts-conditions: Negative zero normalizes to `0`
#[test]
fn negative_zero_normalizes_to_zero() {
    for written in ["-0", "-0.0", "-0e5", "-0.000E-3", "0", "0.0", "0e0"] {
        requirement_condition_reads_as("payouts.limit", written, "0", "1");
        let repo = with_conditions(&[&format!("Context: payouts.limit={written}")], &[]);
        claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.limit=-0"));
    }
}

// behavior-contexts-conditions: The claim part is the `text-1` digest of the claim text followed by its effective condition
#[test]
fn the_claim_part_is_the_digest_of_the_claim_and_its_condition() {
    let repo = with_conditions(&["Context: payouts.instant=true"], &["Context: payouts.region=\"no\""]);
    claim_is_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.instant=true\nContext: payouts.region=\"no\""));
    // The digest is of the claim text and the lines together, as one `text-1` digest.
    claim_is_not_raw(&repo, REACHES_THE_CARD, REACHES_CLAIM);
    claim_is_not_raw(&repo, REACHES_THE_CARD, "Context: payouts.instant=true\nContext: payouts.region=\"no\"");
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("Context: payouts.instant=true\nContext: payouts.region=\"no\"\n{REACHES_CLAIM}"));
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\n\nContext: payouts.instant=true\nContext: payouts.region=\"no\""));
    // After a run the stored claim part of a Check with no condition is the digest of the claim alone.
    let plain = payouts_with_results();
    assert_eq!(result_claim(&plain, REACHES_THE_CARD), text_1_digest(REACHES_CLAIM));
}

// behavior-contexts-conditions: The claim text is normalized as model text before the condition is appended
#[test]
fn the_claim_text_is_normalized_before_the_condition_is_appended() {
    let repo = payouts_with_results();
    // The claim is written decomposed and with trailing spaces and a tab.
    repo.replace(
        PAYOUTS_VERIFICATION,
        "  An approved payout reaches the card at once.\n",
        "  An approved payout reaches the card at cafe\u{301}.  \t\n",
    );
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    claim_is(&repo, REACHES_THE_CARD, "An approved payout reaches the card at caf\u{e9}.", &[INSTANT_TRUE]);
    claim_is_not(&repo, REACHES_THE_CARD, "An approved payout reaches the card at cafe.", &[INSTANT_TRUE]);
}

// behavior-contexts-conditions: A nonempty condition is appended after LF as one `Context: <key>=<normalized value>` line per key, sorted by key, with no last LF
#[test]
fn a_condition_is_appended_as_sorted_lines_without_a_last_lf() {
    let repo = with_conditions(
        &["Context: payouts.region=\"se\"", "Context: payouts.limit=5", "Context: payouts.instant=true", "Context: payouts.label=\"a/b\""],
        &[],
    );
    lint_is_clean(&repo);
    let lines = "Context: payouts.instant=true\nContext: payouts.label=\"a/b\"\nContext: payouts.limit=5\nContext: payouts.region=\"se\"";
    claim_is_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\n{lines}"));
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\n{lines}\n"));
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}{lines}"));
    // The order the lines were written in does not decide the order of the digest.
    claim_is_not_raw(
        &repo,
        REACHES_THE_CARD,
        &format!("{REACHES_CLAIM}\nContext: payouts.region=\"se\"\nContext: payouts.limit=5\nContext: payouts.instant=true\nContext: payouts.label=\"a/b\""),
    );
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\n{}", lines.replace("Context: ", "")));
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\n{}", lines.replace("=", " = ")));
}

// Not a row of the list: the order of lines, which the design settles by sorting the lines of the claim part.
#[test]
fn the_order_of_context_lines_changes_nothing() {
    let lines = ["Context: payouts.instant=true", "Context: payouts.limit=5", "Context: payouts.region=\"se\""];
    let condition = [INSTANT_TRUE, ("payouts.limit", "5"), ("payouts.region", "\"se\"")];
    for order in [[0, 1, 2], [2, 1, 0], [1, 2, 0], [2, 0, 1]] {
        let repo = with_conditions(&order.map(|index| lines[index]), &[]);
        lint_is_clean(&repo);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &condition);
    }
    // An inherited line and an own line of a Scenario give the same claim part in either place.
    let split = with_conditions(&[lines[2]], &[lines[1], lines[0]]);
    claim_is(&split, REACHES_THE_CARD, REACHES_CLAIM, &condition);
}

// behavior-contexts-conditions: One-of values in the claim part use the JSON string spelling of RFC 8785
#[test]
fn one_of_values_in_the_claim_part_use_the_rfc_8785_spelling() {
    // How an author may write each value, which is not how RFC 8785 writes it.
    let written = [
        ("a = b", "\"a \\u003d b\""),
        ("a \"quoted\" b", "\"a \\u0022quoted\\u0022 b\""),
        ("back\\slash", "\"back\\u005cslash\""),
        ("tab\there", "\"tab\\u0009here\""),
        ("line\nbreak", "\"line\\u000abreak\""),
        ("\u{1f}", "\"\\u001F\""),
        ("a/b", "\"a\\/b\""),
        ("\u{e9}", "\"\\u00e9\""),
        ("\u{e9}", "\"e\\u0301\""),
    ];
    for (label, spelling) in written {
        let repo = with_conditions(&[&format!("Context: payouts.label={spelling}")], &[]);
        lint_is_clean(&repo);
        let value = json_string(label);
        claim_is(&repo, REACHES_THE_CARD, REACHES_CLAIM, &[("payouts.label", &value)]);
        // The spelling the author wrote is not the one the digest holds, unless they are the same.
        if spelling != value {
            claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.label={spelling}"));
        }
    }
    // RFC 8785 writes the short escapes for these and lowercase hexadecimal for the others.
    for (label, rfc_8785) in [("tab\there", "\"tab\\there\""), ("line\nbreak", "\"line\\nbreak\""), ("\u{1f}", "\"\\u001f\""), ("a/b", "\"a/b\""), ("\u{e9}", "\"\u{e9}\"")] {
        assert_eq!(json_string(label), rfc_8785);
        let repo = with_conditions(&[&format!("Context: payouts.label={}", json_string(label))], &[]);
        claim_is_raw(&repo, REACHES_THE_CARD, &format!("{REACHES_CLAIM}\nContext: payouts.label={rfc_8785}"));
    }
}

// behavior-contexts-conditions: Inherited and repeated lines contribute each key once to the claim part
#[test]
fn inherited_and_repeated_lines_contribute_each_key_once() {
    let repo = with_conditions(
        &["Context: payouts.instant=true", "Context: payouts.instant=TRUE", "Context: payouts.limit=1", "Context: payouts.limit=1.0"],
        &["Context: payouts.instant=true", "Context: payouts.limit=1.00", "Context: payouts.region=\"no\""],
    );
    lint_is_clean(&repo);
    let once = format!("{REACHES_CLAIM}\nContext: payouts.instant=true\nContext: payouts.limit=1\nContext: payouts.region=\"no\"");
    claim_is_raw(&repo, REACHES_THE_CARD, &once);
    claim_is_not_raw(&repo, REACHES_THE_CARD, &format!("{once}\nContext: payouts.instant=true"));
    claim_is_not_raw(
        &repo,
        REACHES_THE_CARD,
        &format!("{REACHES_CLAIM}\nContext: payouts.instant=true\nContext: payouts.instant=true\nContext: payouts.limit=1\nContext: payouts.region=\"no\""),
    );
    claim_is_raw(&repo, WAITS_FOR_REVIEW, &format!("{WAITS_CLAIM}\nContext: payouts.instant=true\nContext: payouts.limit=1"));
}

const SAMPLE_CLAIMS: [(&str, &str); 4] = [
    (SIXTH_LOAN, "The desk refuses a sixth loan to a borrower who holds five."),
    (RETURNED_BOOK, "After one return the desk lends again to a borrower who held five loans."),
    (LATE_RETURN, "A return three days late records a fine of three days."),
    (ON_TIME_RETURN, "A return on the due date or earlier records no fine."),
];

// behavior-contexts-conditions: A Check with no condition keeps the digest of its claim text alone
#[test]
fn a_check_with_no_condition_keeps_the_digest_of_its_claim_text() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for (check, claim) in SAMPLE_CLAIMS {
        assert_eq!(result_claim(&repo, check), text_1_digest(claim), "{check}");
        assert_eq!(repo.result_of(check)["subjects"][0]["digest"], text_1_digest(claim), "{check}");
    }
    // Keys and conditions elsewhere in the model do not touch it: only the files part moves.
    let payouts = payouts_with_results();
    on_requirement(&payouts, &["Context: payouts.instant=true"]);
    for (check, claim) in SAMPLE_CLAIMS {
        claim_is(&payouts, check, claim, &[]);
        assert_eq!(parts_changed(&payouts.check_show(check)), vec!["files"], "{check}");
    }
}

// behavior-contexts-conditions: Changing a condition makes the Check owed with "claim changed"
#[test]
fn changing_a_condition_makes_the_check_owed_with_a_changed_claim() {
    let repo = payouts_with_results();
    on_requirement(&repo, &["Context: payouts.region=\"no\""]);
    // The result the Check would have had under the first condition.
    store_with_claim(&repo, REACHES_THE_CARD, &claim_digest(REACHES_CLAIM, &[("payouts.region", "\"no\"")]));
    assert_eq!(parts_changed(&repo.check_show(REACHES_THE_CARD)), vec!["files"]);
    repo.replace(PAYOUTS_SPEC, "Context: payouts.region=\"no\"", "Context: payouts.region=\"se\"");
    let check = repo.check_show(REACHES_THE_CARD);
    assert_eq!(check["state"], "owed", "{check:#}");
    assert_eq!(parts_changed_set(&check), set_of(&["claim", "files"]), "{check:#}");
    let explanation = format!("{} {}", check["reason"].as_str().unwrap_or_default(), check["detail"].as_str().unwrap_or_default());
    assert!(explanation.contains("claim"), "the explanation does not name the claim: {check:#}");
    let status = repo.status_json();
    assert_eq!(status_checks(&status)[REACHES_THE_CARD]["state"], "owed", "{status:#}");
    // Adding and removing a condition changes the claim as well.
    let added = payouts_with_results();
    on_requirement(&added, &["Context: payouts.instant=true"]);
    let check = added.check_show(WAITS_FOR_REVIEW);
    assert_eq!(parts_changed_set(&check), set_of(&["claim", "files"]), "{check:#}");
}

// Not a row of the list: the two sides of the claim digest, as a person edits a condition.
#[test]
fn a_respelled_condition_leaves_the_claim_unchanged() {
    for (first, second) in [
        ("Context: payouts.instant=true", "Context: payouts.instant=TRUE"),
        ("Context: payouts.limit=1", "Context: payouts.limit=1.00"),
        ("Context: payouts.limit=100", "Context: payouts.limit=1e2"),
        ("Context: payouts.limit=0", "Context: payouts.limit=-0"),
        ("Context: payouts.region=\"\u{e9}\"", "Context: payouts.region=\"e\u{301}\""),
        ("Context: payouts.region=\"se\"", "Context: payouts.region=\"\\u0073e\""),
    ] {
        let repo = payouts_with_results();
        on_requirement(&repo, &[first]);
        // The first spelling of each pair is already the normalized one.
        let (key, value) = first.trim_start_matches("Context: ").split_once('=').unwrap();
        store_with_claim(&repo, REACHES_THE_CARD, &claim_digest(REACHES_CLAIM, &[(key, value)]));
        repo.replace(PAYOUTS_SPEC, first, second);
        let check = repo.check_show(REACHES_THE_CARD);
        assert_eq!(parts_changed(&check), vec!["files"], "{first} became {second}: {check:#}");
    }
    // The control: another value of the same key is another claim.
    let repo = payouts_with_results();
    on_requirement(&repo, &["Context: payouts.limit=1"]);
    store_with_claim(&repo, REACHES_THE_CARD, &claim_digest(REACHES_CLAIM, &[("payouts.limit", "1")]));
    repo.replace(PAYOUTS_SPEC, "Context: payouts.limit=1", "Context: payouts.limit=1.001");
    assert_eq!(parts_changed_set(&repo.check_show(REACHES_THE_CARD)), set_of(&["claim", "files"]));
}

// behavior-contexts-conditions: The fingerprint gains no extra context part
#[test]
fn the_fingerprint_has_no_context_part() {
    let repo = payouts_with_results();
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    // A Check with no condition, in a model that has conditions, is written with the four parts.
    for (check, claim) in SAMPLE_CLAIMS {
        let record = repo.result_of(check);
        let parts: BTreeSet<String> = record["body"]["parts"].as_object().unwrap().keys().cloned().collect();
        assert_eq!(parts, PARTS.iter().map(|part| part.to_string()).collect::<BTreeSet<_>>(), "{check}: {record:#}");
        claim_is(&repo, check, claim, &[]);
    }
    // A changed condition names no part but the claim, besides the files the edit touched.
    store_with_claim(&repo, REACHES_THE_CARD, &claim_digest(REACHES_CLAIM, &[INSTANT_TRUE]));
    repo.replace(PAYOUTS_SPEC, "Context: payouts.instant=true", "Context: payouts.instant=false");
    let parts = parts_changed_set(&repo.check_show(REACHES_THE_CARD));
    assert_eq!(parts, set_of(&["claim", "files"]));
    for part in parts {
        assert!(PARTS.contains(&part.as_str()), "{part} is not a part of the fingerprint");
    }
}
