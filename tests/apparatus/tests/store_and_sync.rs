//! Group "The local store and `azimuth sync`" of docs/increments/1.md.

use std::collections::BTreeSet;
use std::fs;

use azimuth_tests::*;

fn all_kept_in(repo: &Repo, dir: &std::path::Path) {
    let output = repo.azimuth_in(dir, &["status", "--json"], &[]);
    output.expect_status(0);
    let status = output.json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "kept", "{status:#}");
    }
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

// behavior: The store is the folder `azimuth/` in the clone's shared git directory
#[test]
fn the_store_is_in_the_shared_git_directory() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    assert_eq!(repo.store(), repo.root().join(".git/azimuth"));
    for check in ALL_CHECKS {
        assert!(
            repo.records_naming(check).iter().any(|(path, _)| path.starts_with(repo.root().join(".git/azimuth"))),
            "no record of {check} in .git/azimuth"
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
    let files = repo.store_files();
    assert!(!files.is_empty());
    for file in &files {
        let bytes = fs::read(file).unwrap();
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        if name.ends_with(".json") {
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_else(|e| panic!("{name} is not JSON: {e}"));
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
    let waiting: BTreeSet<String> = repo
        .outbox_files()
        .iter()
        .map(|file| file.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    let tree_before = repo.git(&["status", "--porcelain"]);
    let closed = "http://127.0.0.1:9";
    let output = repo.azimuth_with_env(&["sync"], &[("https_proxy", closed), ("HTTPS_PROXY", closed), ("ALL_PROXY", closed)]);
    output.expect_status(0);
    assert!(repo.outbox_files().is_empty(), "records stayed in the outbox");
    let stored: BTreeSet<String> = repo
        .store_files()
        .iter()
        .map(|file| file.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(waiting.is_subset(&stored), "the records did not arrive in the store under their names");
    assert_eq!(repo.git(&["status", "--porcelain"]), tree_before);
    assert!(repo.adapter_starts().len() == 1, "sync started the adapter");
    assert!(files_under(&repo.beside("home")).is_empty(), "sync wrote into the home folder");
    all_kept_in(&repo, repo.root());
}

// behavior: A record in a format this version cannot read counts as absent, and its Check is owed
#[test]
fn an_unreadable_record_counts_as_absent() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.rewrite_records(bump_formats);
    let status = repo.status_json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "owed", "{status:#}");
    }
    assert!(count(&status["skipped_records"]) >= 4, "{status:#}");
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
    let status = repo.status_json();
    let checks = status_checks(&status);
    for check in ALL_CHECKS {
        assert_eq!(checks[check]["state"], "owed", "{status:#}");
    }
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
