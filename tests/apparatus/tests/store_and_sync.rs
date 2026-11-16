//! Group "The local store and `azimuth sync`" of docs/increments/1.md.

use std::collections::BTreeSet;
use std::fs;

use azimuth_tests::*;
use serde_json::Value;

fn all_kept_in(repo: &Repo, dir: &std::path::Path) {
    let output = repo.azimuth_in(dir, &["status", "--json"], &[]);
    output.expect_status(0);
    let status = output.json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "kept", "{status:#}");
    }
}

fn all_owed(repo: &Repo) -> Value {
    let status = repo.status_json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "owed", "{status:#}");
    }
    status
}

/// Runs the sample with a store the session may not write, so the results stay in the outbox.
fn results_in_the_outbox() -> Repo {
    let repo = Repo::sample();
    let store = repo.store();
    fs::create_dir_all(&store).unwrap();
    set_mode(&store, 0o555);
    let output = repo.azimuth(&["run", "--json"]);
    set_mode(&store, 0o755);
    output.expect_status(0);
    assert!(!repo.outbox_files().is_empty(), "the run left nothing in the outbox");
    assert!(files_under(&store).is_empty());
    repo
}

fn names(files: &[std::path::PathBuf]) -> BTreeSet<String> {
    files.iter().map(|file| file.file_name().unwrap().to_string_lossy().to_string()).collect()
}

/// The sample after one run in which `late-return-is-fined` failed and the other Checks passed.
fn one_fail_and_three_passes() -> Repo {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed\nduration: 250");
    repo.replace(FREE_RETURN_TESTS, "test: returned-early\noutcome: passed", "test: returned-early\noutcome: passed\nduration: 40");
    repo.run_json(&[], 1);
    repo
}

// behavior: The store is the folder `azimuth/` in the clone's shared git directory
#[test]
fn the_store_is_in_the_shared_git_directory() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    assert_eq!(repo.store(), repo.root().join(".git/azimuth"));
    for check in ALL_CHECKS {
        assert!(
            repo.results_of(check).iter().any(|(path, _)| path.starts_with(repo.root().join(".git/azimuth"))),
            "no result of {check} in .git/azimuth"
        );
    }
}

// behavior: Two worktrees of one clone see the same results
#[test]
fn a_second_worktree_sees_the_results() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let worktree = repo.beside("second-worktree");
    repo.git(&["worktree", "add", "--quiet", "-b", "second", worktree.to_str().unwrap()]);
    all_kept_in(&repo, &worktree);
    let output = repo.azimuth_in(&worktree, &["run", "--json"], &[]);
    output.expect_status(0);
    assert!(files_under(&worktree.join(".git/azimuth")).is_empty(), "a linked worktree got a store of its own");
}

// behavior: The store is plain files with no index and no database
#[test]
fn the_store_is_plain_json_files() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let files = files_under(&repo.store());
    assert!(!files.is_empty());
    for file in &files {
        let bytes = fs::read(file).unwrap();
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        if name.ends_with(".json") {
            serde_json::from_slice::<Value>(&bytes).unwrap_or_else(|e| panic!("{name} is not JSON: {e}"));
        } else {
            assert!(String::from_utf8(bytes).is_ok(), "{name} is not plain text");
        }
        for word in ["index", ".db", ".sqlite", ".lock"] {
            assert!(!name.contains(word), "the store holds {name}");
        }
    }
}

// behavior: A record still in the outbox is read like a record in the store
#[test]
fn a_record_in_the_outbox_counts() {
    let repo = results_in_the_outbox();
    all_kept_in(&repo, repo.root());
}

// behavior: With no server address, `azimuth sync` moves records from the outbox into the store and does nothing else
#[test]
fn sync_moves_the_outbox_into_the_store() {
    let repo = results_in_the_outbox();
    let waiting = names(&repo.outbox_files());
    let tree_before = repo.git(&["status", "--porcelain"]);
    let closed = "http://127.0.0.1:9";
    let output = repo.azimuth_with_env(&["sync"], &[("https_proxy", closed), ("HTTPS_PROXY", closed), ("ALL_PROXY", closed)]);
    output.expect_status(0);
    assert!(repo.outbox_files().is_empty(), "records stayed in the outbox");
    assert!(waiting.is_subset(&names(&repo.store_files())), "the records did not arrive in the store under their names");
    assert_eq!(repo.git(&["status", "--porcelain"]), tree_before);
    assert!(repo.adapter_starts().len() == 1, "sync started the adapter");
    assert!(files_under(&repo.beside("home")).is_empty(), "sync wrote into the home folder");
    all_kept_in(&repo, repo.root());
}

// behavior: A record in a format this version cannot read counts as absent, and its Check is owed
#[test]
fn a_record_of_the_next_format_number_counts_as_absent() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.rewrite_records(next_format);
    let status = all_owed(&repo);
    assert_eq!(status["skipped_records"], 4, "{status:#}");
    let check = repo.check_show(SIXTH_LOAN);
    assert_eq!(check["reason"], "never run", "{check:#}");
}

// behavior: `azimuth run` writes each result as a record whose `format` is `result-1`
#[test]
fn each_result_is_a_result_1_record() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let records = repo.records();
    assert_eq!(records.len(), 4, "one record per Check: {records:#?}");
    let subjects: BTreeSet<String> =
        records.iter().map(|(_, record)| record["subjects"][0]["name"].as_str().unwrap_or_default().to_string()).collect();
    assert_eq!(subjects, set_of(&ALL_CHECKS));
    for (path, record) in &records {
        assert_eq!(record["format"], "result-1", "{}: {record:#}", path.display());
    }
}

// behavior: A record's `body.outcome` is `passed` or `failed`, and a crash or a skipped test writes no record
#[test]
fn a_record_holds_passed_or_failed_and_nothing_else() {
    let repo = one_fail_and_three_passes();
    assert_eq!(repo.result_of(LATE_RETURN)["body"]["outcome"], "failed");
    for check in [SIXTH_LOAN, RETURNED_BOOK, ON_TIME_RETURN] {
        assert_eq!(repo.result_of(check)["body"]["outcome"], "passed");
    }

    let crashed = Repo::sample();
    crashed.append(LATE_FINE_TESTS, "adapter: crash\n");
    crashed.run_json(&[], 1);
    assert!(crashed.records().is_empty(), "a crashed launch wrote records: {:#?}", crashed.records());

    let skipped = Repo::sample();
    skipped.replace(FREE_RETURN_TESTS, "test: returned-early\noutcome: passed", "test: returned-early\noutcome: skipped");
    skipped.run_json(&[], 1);
    assert!(skipped.results_of(ON_TIME_RETURN).is_empty(), "a skipped test wrote a record");
    assert_eq!(skipped.records().len(), 3, "{:#?}", skipped.records());
}

// behavior: A record's body holds `fingerprint` and, under `parts`, the digests of `claim`, `files`, `environment` and `method`
#[test]
fn a_record_holds_its_fingerprint_and_four_part_digests() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        let body = &repo.result_of(check)["body"];
        let fingerprint = body["fingerprint"].as_str().unwrap_or_default();
        assert!(fingerprint.starts_with("parts-1:") && is_digest(fingerprint), "{check}: {body:#}");
        let parts = body["parts"].as_object().unwrap_or_else(|| panic!("{check}: no `parts`: {body:#}"));
        let names: BTreeSet<String> = parts.keys().cloned().collect();
        assert_eq!(names, set_of(&PARTS), "{check}: {body:#}");
        for part in PARTS {
            assert!(is_digest(parts[part].as_str().unwrap_or_default()), "{check}: {part} is no digest: {body:#}");
        }
        assert!(parts["claim"].as_str().unwrap().starts_with("text-1:"), "{check}: {body:#}");
    }
}

// behavior: A record of a run on exactly a commit holds that commit's tree id as `body.tree.base` and no `edits`
#[test]
fn a_run_on_a_commit_stores_its_tree_id_and_no_edits() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
    for check in ALL_CHECKS {
        let stored = &repo.result_of(check)["body"]["tree"];
        assert_eq!(stored["base"], tree.as_str(), "{check}: {stored:#}");
        let edits = &stored["edits"];
        assert!(edits.is_null() || edits.as_array().is_some_and(|items| items.is_empty()), "{check}: {stored:#}");
    }
}

// behavior: A record of a run with an edited, a new and a deleted file holds one line per such path under `body.tree.edits`
#[test]
fn a_run_with_local_edits_stores_one_line_per_path() {
    let repo = Repo::sample();
    repo.append(PRODUCT_FILE, "renewals do not count\n");
    repo.write("desk/shelves.txt", "north shelf holds novels\n");
    repo.remove("desk/returns.txt");
    repo.run_json(&[], 0);
    let edited = repo.git(&["hash-object", PRODUCT_FILE]).trim().to_string();
    let new = repo.git(&["hash-object", "desk/shelves.txt"]).trim().to_string();
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]).trim().to_string();
    let stored = repo.result_of(SIXTH_LOAN)["body"]["tree"].clone();
    assert_eq!(stored["base"], tree.as_str(), "{stored:#}");
    let mut edits = stored["edits"].as_array().cloned().unwrap_or_else(|| panic!("no `edits`: {stored:#}"));
    edits.sort_by_key(|edit| edit["path"].as_str().unwrap_or_default().to_string());
    assert_eq!(
        edits,
        vec![
            json!({ "path": PRODUCT_FILE, "mode": "100644", "id": format!("file-1:{edited}") }),
            json!({ "path": "desk/returns.txt", "deleted": true }),
            json!({ "path": "desk/shelves.txt", "mode": "100644", "id": format!("file-1:{new}") }),
        ]
    );
}

// behavior: A record's body holds, under `method`, the scheme names, the object format and the adapter's name and version
#[test]
fn a_record_names_its_method() {
    let mut repo = Repo::sample();
    repo.set_env("TEXT_ADAPTER_VERSION", "1.4.2");
    repo.run_json(&[], 0);
    let method = repo.result_of(LATE_RETURN)["body"]["method"].clone();
    assert_eq!(method["object_format"], "sha1", "{method:#}");
    assert_eq!(method["adapter"], json!({ "name": "text-suite-adapter", "version": "1.4.2" }), "{method:#}");
}

// behavior: `method.schemes` holds `file-1`, `parts-1` and `text-1`, sorted by name, and not `record-1`
#[test]
fn the_schemes_are_the_three_of_the_fingerprint_in_order() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        let record = repo.result_of(check);
        assert_eq!(record["body"]["method"]["schemes"], json!(["file-1", "parts-1", "text-1"]), "{record:#}");
    }
}

// behavior: A record's body holds every enrolled test with its name, outcome and duration
#[test]
fn a_record_lists_its_enrolled_tests() {
    let repo = one_fail_and_three_passes();
    assert_eq!(
        repo.result_of(LATE_RETURN)["body"]["tests"],
        json!([{ "name": "late-fine", "outcome": "failed", "duration_ms": 250 }])
    );
    let mut tests = repo.result_of(ON_TIME_RETURN)["body"]["tests"].as_array().cloned().unwrap_or_default();
    tests.sort_by_key(|test| test["name"].as_str().unwrap_or_default().to_string());
    assert_eq!(
        tests,
        vec![
            json!({ "name": "free-return.returned-early", "outcome": "passed", "duration_ms": 40 }),
            json!({ "name": "free-return.returned-on-due-date", "outcome": "passed", "duration_ms": 1 }),
        ]
    );
}

// behavior: A record has the outer fields `format`, `actor`, `at`, `subjects` and `body`
#[test]
fn a_record_has_the_five_outer_fields() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for (path, record) in repo.records() {
        let fields: BTreeSet<String> = record.as_object().unwrap().keys().cloned().collect();
        assert_eq!(fields, set_of(&["format", "actor", "at", "subjects", "body"]), "{}: {record:#}", path.display());
        assert!(!record["at"].as_str().unwrap_or_default().is_empty(), "{record:#}");
    }
}

// behavior: A result's `actor` is `this machine`
#[test]
fn a_result_is_made_by_this_machine() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(repo.result_of(check)["actor"], "this machine");
    }
}

// behavior: A result has one subject: the Check, with its kind, its id as the name, and the digest of its claim
#[test]
fn a_result_is_about_one_check_and_its_claim() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let mut kinds = BTreeSet::new();
    let mut claims = BTreeSet::new();
    for check in ALL_CHECKS {
        let record = repo.result_of(check);
        let subjects = record["subjects"].as_array().unwrap_or_else(|| panic!("no subjects: {record:#}"));
        assert_eq!(subjects.len(), 1, "{record:#}");
        let fields: BTreeSet<String> = subjects[0].as_object().unwrap().keys().cloned().collect();
        assert_eq!(fields, set_of(&["kind", "name", "digest"]), "{record:#}");
        assert_eq!(subjects[0]["name"], check);
        assert_eq!(subjects[0]["digest"], record["body"]["parts"]["claim"], "{record:#}");
        kinds.insert(subjects[0]["kind"].as_str().unwrap_or_default().to_string());
        claims.insert(subjects[0]["digest"].as_str().unwrap_or_default().to_string());
    }
    assert_eq!(kinds, set_of(&["check"]), "{kinds:?}");
    assert_eq!(claims.len(), 4, "four claims have four digests: {claims:?}");
}

fn every_record_is_canonical_and_named_by_its_bytes(repo: &Repo) {
    let files: Vec<_> = repo.store_files().into_iter().chain(repo.outbox_files()).collect();
    assert!(!files.is_empty(), "no record was written");
    for file in files {
        let bytes = fs::read(&file).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(String::from_utf8(bytes.clone()).unwrap(), canonical_json(&value), "{} is not canonical", file.display());
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(name, format!("{}.json", sha256_hex(&bytes)), "{} is not named by the SHA-256 of its bytes", file.display());
    }
}

// behavior: A record's file holds the canonical form of RFC 8785, and its name is the SHA-256 of those bytes
#[test]
fn records_are_canonical_json_named_by_sha_256() {
    let repo = one_fail_and_three_passes();
    every_record_is_canonical_and_named_by_its_bytes(&repo);
}

// behavior: A record holds no fractional number
#[test]
fn a_record_holds_only_whole_numbers() {
    fn walk(value: &Value, numbers: &mut usize) {
        match value {
            Value::Number(number) => {
                assert!(number.is_u64() || number.is_i64(), "a fractional number {number}");
                *numbers += 1;
            }
            Value::Array(items) => items.iter().for_each(|item| walk(item, numbers)),
            Value::Object(map) => map.values().for_each(|item| walk(item, numbers)),
            _ => {}
        }
    }
    let repo = one_fail_and_three_passes();
    let mut numbers = 0;
    for (_, record) in repo.records() {
        walk(&record, &mut numbers);
    }
    assert!(numbers > 0, "no record holds a number, so nothing was checked");
}

// behavior: A record's file is named by the digest of its own bytes
#[test]
fn a_record_keeps_its_name_from_the_outbox_into_the_store() {
    let repo = results_in_the_outbox();
    every_record_is_canonical_and_named_by_its_bytes(&repo);
    let waiting = names(&repo.outbox_files());
    repo.azimuth(&["sync"]).expect_status(0);
    assert_eq!(names(&repo.store_files()), waiting);
    every_record_is_canonical_and_named_by_its_bytes(&repo);
}

// behavior: The `kind` of the subject is the word `check`
#[test]
fn the_subject_kind_is_check() {
    let repo = one_fail_and_three_passes();
    for (path, record) in repo.records() {
        assert_eq!(record["subjects"][0]["kind"], "check", "{}: {record:#}", path.display());
    }
}

// behavior: A record's file name is 64 lowercase hex characters followed by `.json`
#[test]
fn a_record_file_name_is_lowercase_hex_and_json() {
    let repo = one_fail_and_three_passes();
    let files: Vec<_> = repo.store_files().into_iter().chain(repo.outbox_files()).collect();
    assert!(!files.is_empty(), "no record was written");
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let hex = name.strip_suffix(".json").unwrap_or_else(|| panic!("{name} does not end with .json"));
        assert_eq!(hex.len(), 64, "{name}");
        assert!(hex.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)), "{name} is not lowercase hex");
    }
}

// behavior: A record whose bytes do not match its file name is counted with the skipped records
#[test]
fn a_damaged_record_is_counted_as_skipped() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    assert_eq!(repo.status_json()["skipped_records"], 0);
    let store = repo.store_files();
    let mut damaged_in_store = 0;
    for (file, mut record) in repo.results_of(LATE_RETURN) {
        record["at"] = Value::String(format!("{} ", record["at"].as_str().unwrap()));
        fs::write(&file, canonical_json(&record)).unwrap();
        damaged_in_store += usize::from(store.contains(&file));
    }
    assert!(damaged_in_store > 0, "the result is not in the store");
    let status = repo.status_json();
    assert_eq!(status["skipped_records"], damaged_in_store, "{status:#}");
    assert_eq!(status_checks(&status)[LATE_RETURN]["state"], "owed", "{status:#}");
}

// behavior: A record whose bytes do not match its file name counts as absent
#[test]
fn a_damaged_record_counts_as_absent() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    for (file, mut record) in repo.records() {
        // The new bytes are canonical and readable, and only the name no longer matches them.
        record["at"] = Value::String(format!("{} ", record["at"].as_str().unwrap()));
        fs::write(&file, canonical_json(&record)).unwrap();
    }
    all_owed(&repo);
    repo.clear_log();
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
}

// behavior: The command that first creates `.azimuth/` writes its `.gitignore`
#[test]
fn the_first_command_to_create_the_folder_writes_its_ignore_file() {
    let repo = Repo::sample();
    repo.remove(".azimuth");
    repo.run_json(&[], 0);
    assert!(repo.outbox().is_dir(), "the run created no .azimuth/");
    let ignore = repo.read(".azimuth/.gitignore");
    assert!(ignore.lines().any(|line| line.trim() == "*"), "{ignore:?}");
    assert_eq!(repo.git(&["status", "--porcelain"]), "", "the folder shows in git status");
}

// behavior: No command deletes records from the store
#[test]
fn no_command_removes_a_record() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let mut seen: BTreeSet<_> = repo.store_files().into_iter().collect();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    for args in [
        &["run", "--json"][..],
        &["check", "run", LATE_RETURN][..],
        &["run", "--lane", "commit"][..],
        &["sync"][..],
        &["status"][..],
        &["lint"][..],
        &["check", "show", LATE_RETURN][..],
        &["requirement", "show", LATE_FINE][..],
        &["change", "show"][..],
    ] {
        repo.azimuth(args);
        let now: BTreeSet<_> = repo.store_files().into_iter().collect();
        assert!(seen.is_subset(&now), "azimuth {args:?} removed records");
        seen = now;
    }
    for args in [&["clean"][..], &["store", "clear"][..], &["sync", "--prune"][..]] {
        repo.azimuth(args).expect_status(2);
    }
}

// behavior: Deleting the store loses no correctness: every Check is then owed
#[test]
fn deleting_the_store_makes_every_check_owed() {
    let repo = Repo::sample();
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.run_json(&[], 1);
    fs::remove_dir_all(repo.store()).unwrap();
    all_owed(&repo);
    let document = repo.run_json(&[], 1);
    assert_eq!(state_of(&document, LATE_RETURN), "failed", "an emptied store hid the failure");
}

// behavior: No setting anywhere names a rung
#[test]
fn no_setting_names_a_rung() {
    for (table, line) in [("", "rung = \"core\"\n"), ("[repository]\n", "rung = \"core\"\n")] {
        let repo = Repo::sample();
        if table.is_empty() {
            repo.replace(PROJECT_FILE, "[repository]\n", &format!("{line}\n[repository]\n"));
        } else {
            repo.replace(PROJECT_FILE, table, &format!("{table}{line}"));
        }
        repo.expect_reading_error(Some(PROJECT_FILE), Some(repo.line_of(PROJECT_FILE, "rung =")));
    }
    let repo = Repo::sample();
    repo.azimuth(&["run", "--rung", "core"]).expect_status(2);
}
