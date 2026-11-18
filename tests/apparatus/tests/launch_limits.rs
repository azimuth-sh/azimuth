use azimuth_tests::*;
use std::time::Duration;
#[allow(dead_code)]
mod support;
use support::*;

fn limited(repo: &Repo, value: &str) {
    repo.append(PROJECT_FILE, &format!("time_limit_minutes = {value}\n"));
}
fn tree(mode: &str, minutes: Option<u32>) -> Repo {
    let mut repo = Repo::sample();
    if let Some(minutes) = minutes {
        limited(&repo, &minutes.to_string());
    }
    let token = repo.beside("tree-token").display().to_string();
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        &format!("adapter = \"process-tree adapter {token}\""),
    );
    repo.set_env("TREE_MODE", mode);
    repo.set_env("TREE_LOG", repo.beside("tree.jsonl").to_str().unwrap());
    repo.set_env(
        "TREE_REQUEST",
        repo.beside("request.json").to_str().unwrap(),
    );
    repo
}
fn no_results(repo: &Repo, out: &Output) {
    out.expect_status(1);
    let doc = out.json();
    assert_eq!(doc["format"], "run-1");
    assert_eq!(doc["suites"][0]["launch"], "timed out");
    for check in ALL_CHECKS {
        assert_eq!(state_of(&doc, check), "owed");
        assert_eq!(reason_of(&doc, check), "no result");
    }
    assert!(
        repo.store_files().is_empty() && repo.outbox_files().is_empty(),
        "timed-out launch wrote record files"
    );
}
fn stamp(run: &LiveRun, role: &str, event: &str) -> u64 {
    let entries: Vec<_> = run
        .events()
        .into_iter()
        .filter(|e| e["role"] == role && e["event"] == event)
        .collect();
    assert_eq!(entries.len(), 1, "{entries:#?}");
    entries[0]["time_ns"].as_u64().unwrap()
}
fn assert_tree_stopped(run: &LiveRun, root: &str) {
    let entries = run.events();
    let starts: Vec<_> = entries.iter().filter(|e| e["event"] == "started").collect();
    assert_eq!(starts.len(), 3);
    let group = starts.iter().find(|e| e["role"] == root).unwrap()["group"].clone();
    assert_ne!(group, json!(run.cli_group));
    for entry in starts {
        assert_eq!(entry["group"], group);
        assert!(
            entries
                .iter()
                .any(|e| e["pid"] == entry["pid"] && e["event"] == "observed-exit"),
            "surviving process: {entry:#}"
        );
    }
}
fn first_term(run: &LiveRun) -> u64 {
    run.events()
        .iter()
        .filter(|event| event["event"] == "SIGTERM")
        .map(|event| event["time_ns"].as_u64().unwrap())
        .min()
        .expect("the group received SIGTERM")
}
fn limit_bounds(run: &LiveRun, role: &str, seconds: u64) {
    let elapsed = first_term(run) - stamp(run, role, "entered");
    // Entry excludes helper setup, stdin and child creation. Allow 750 ms for exec-to-entry
    // scheduling, less than the one-second early limit this test must reject. Dispatch can
    // run late under load, so the other side allows three seconds. The preceding four-second
    // proof still cannot be charged to the adapter's clock.
    assert!(
        elapsed >= seconds * 1_000_000_000 - 750_000_000
            && elapsed <= (seconds + 3) * 1_000_000_000,
        "limit after {elapsed} ns from main entry"
    );
}
fn grace_bounds(run: &LiveRun, role: &str) {
    // A group mate's receipt cannot prove that this resistant process got the polite stop.
    let term = stamp(run, role, "SIGTERM");
    let alive = stamp(run, role, "observed-alive") - term;
    // A sample begun after nine seconds proves an eight-second grace is wrong; a delayed ps
    // response cannot manufacture that witness. Allow exit detection until 11.5 seconds for
    // polling and process-inspection latency, still short of the twelve-second wrong grace.
    assert!(
        alive >= 9_000_000_000,
        "resistant process was not witnessed alive during the grace"
    );
    let elapsed = stamp(run, role, "observed-exit") - term;
    assert!(
        (9_000_000_000..=11_500_000_000).contains(&elapsed),
        "{role} grace after {elapsed} ns from its own SIGTERM receipt"
    );
}
// behavior-2: A launch of an adapter has a time limit
// behavior-2: At the limit the core sends SIGTERM to the whole group, not to the adapter alone
// behavior-2: The suite of a launch that was stopped at the limit has the word `timed out` in `launch`
// behavior-2: Every Check of that launch is owed with the reason `no result`
// behavior-2: A run with a launch that timed out ends with status 1
// behavior-2: The clock starts when the adapter starts and stops when it exits, so the build counts
// behavior-2: A launch that timed out prints what the runner wrote up to the stop
#[test]
fn a_build_that_hangs_loses_the_whole_process_tree_at_one_minute() {
    std::thread::scope(|scope| {
        for json in [false, true] {
            scope.spawn(move || {
                let repo = tree("orphan", Some(1));
                repo.need_facility("sleep 4");
                let args = if json {
                    vec!["run", "--json"]
                } else {
                    vec!["run"]
                };
                let mut run = LiveRun::start(&repo, &args, "timeout");
                run.wait_until(Duration::from_secs(150), |run| {
                    run.events()
                        .iter()
                        .filter(|e| e["event"] == "started")
                        .count()
                        == 3
                });
                assert_tree_is_separate_and_alive(&run, "adapter");
                let during = run.text();
                let payload = format!("TREE-PAYLOAD-{}", repo.beside("tree-token").display());
                run.wait_until(Duration::from_secs(180), |run| {
                    let stopped = || {
                        run.events()
                            .iter()
                            .any(|event| event["role"] == "adapter" && event["event"] == "SIGTERM")
                    };
                    if stopped() {
                        return true;
                    }
                    let output = run.text();
                    if stopped() {
                        return true;
                    }
                    assert!(
                        !output.stdout.contains(&payload) && !output.stderr.contains(&payload),
                        "timeout output escaped while the adapter was alive"
                    );
                    false
                });
                let output = run.finish(Duration::from_secs(240));
                output.expect_status(1);
                if json {
                    no_results(&repo, &output);
                }
                let payload = format!("TREE-PAYLOAD-{}", repo.beside("tree-token").display());
                assert!(
                    !during.stdout.contains(&payload) && !during.stderr.contains(&payload),
                    "timed launch output escaped before exit"
                );
                assert_eq!(output.stderr.matches(&payload).count(), 1);
                assert!(!output.stdout.contains(&payload));
                assert_tree_stopped(&run, "adapter");
                limit_bounds(&run, "adapter", 60);
                for role in ["adapter", "runner", "grandchild"] {
                    stamp(&run, role, "SIGTERM");
                }
                let proof = repo.probe_starts();
                assert_eq!(proof.len(), 1);
                assert!(
                    stamp(&run, "adapter", "entered") - proof[0]["time_ns"].as_u64().unwrap()
                        >= 4_000_000_000
                );
            });
        }
    });
}

fn assert_tree_is_separate_and_alive(run: &LiveRun, root: &str) {
    let events = run.events();
    assert!(!events.iter().any(|event| event["event"] == "SIGTERM"));
    let starts: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "started")
        .collect();
    assert_eq!(starts.len(), 3);
    let group = starts.iter().find(|event| event["role"] == root).unwrap()["group"]
        .as_i64()
        .unwrap() as i32;
    let table = azimuth_tests::process::processes().unwrap_or_else(|error| panic!("{error}"));
    for event in starts {
        let process = &table[&(event["pid"].as_u64().unwrap() as u32)];
        assert!(
            process.running(),
            "{} was not alive before the limit",
            event["role"]
        );
        assert_eq!(process.group, group);
        assert_ne!(
            process.group, run.cli_group,
            "{} shares the CLI's group before the limit",
            event["role"]
        );
    }
}

#[test]
fn a_timeout_finishes_even_when_an_escaped_process_retains_stdout() {
    let repo = tree("escaped-output", Some(1));
    let mut run = LiveRun::start(&repo, &["run", "--json"], "escaped-output");
    run.wait_until(Duration::from_secs(30), |run| {
        run.events()
            .iter()
            .any(|event| event["role"] == "holder" && event["event"] == "started")
    });
    assert_ne!(stamp_group(&run, "holder"), stamp_group(&run, "adapter"));
    // The escaped process is explicitly outside the core's reach; waiting for its pipe is not grace.
    let output = run.finish_cli(Duration::from_secs(80));
    no_results(&repo, &output);
    assert_holder_alive(&run);
    // LiveRun's guard verifies executable, unique token, birth and command before killing the holder.
}

fn stamp_group(run: &LiveRun, role: &str) -> serde_json::Value {
    run.events()
        .into_iter()
        .find(|event| event["role"] == role && event["event"] == "started")
        .unwrap()["group"]
        .clone()
}
fn assert_holder_alive(run: &LiveRun) {
    let event = run
        .events()
        .into_iter()
        .find(|event| event["role"] == "holder" && event["event"] == "started")
        .unwrap();
    let table = azimuth_tests::process::processes().unwrap_or_else(|error| panic!("{error}"));
    let process = &table[&(event["pid"].as_u64().unwrap() as u32)];
    assert!(process.running());
    assert!(process
        .command
        .split_whitespace()
        .any(|arg| Some(arg) == event["token"].as_str()));
}

#[test]
fn a_report_is_used_without_waiting_for_a_descendants_stdout_to_close() {
    let repo = tree("inherited-output", Some(1));
    let mut run = LiveRun::start(&repo, &["run", "--json"], "inherited-output");
    let output = run.finish_cli(Duration::from_secs(15));
    output.expect_status(0);
    let doc = output.json();
    assert_eq!(doc["suites"][0]["launch"], "ok");
    for check in ALL_CHECKS {
        assert_eq!(state_of(&doc, check), "passed");
        assert_eq!(repo.result_of(check)["body"]["outcome"], "passed");
    }
    assert_eq!(repo.records().len(), 4);
    assert!(stamp(&run, "holder", "started") < stamp(&run, "adapter", "exited"));
}

#[test]
fn the_commit_lane_stops_a_launch_at_its_suite_limit() {
    let repo = tree("orphan", Some(1));
    let mut run = LiveRun::start(
        &repo,
        &["run", "--lane", "commit", "--json"],
        "commit-timeout",
    );
    let output = run.finish(Duration::from_secs(90));
    no_results(&repo, &output);
    assert_eq!(output.json()["lane"], "commit");
    limit_bounds(&run, "adapter", 60);
    assert_tree_stopped(&run, "adapter");
}

#[test]
fn check_run_stops_a_launch_at_its_suite_limit() {
    let repo = tree("orphan", Some(1));
    let mut run = LiveRun::start(
        &repo,
        &["check", "run", SIXTH_LOAN, "--json"],
        "check-timeout",
    );
    let output = run.finish(Duration::from_secs(90));
    output.expect_status(1);
    let doc = output.json();
    assert_eq!(doc["format"], "run-1");
    assert_eq!(doc["suites"].as_array().unwrap().len(), 1);
    assert_eq!(doc["suites"][0]["launch"], "timed out");
    assert_eq!(doc["checks"].as_array().unwrap().len(), 1);
    assert_eq!(state_of(&doc, SIXTH_LOAN), "owed");
    assert_eq!(reason_of(&doc, SIXTH_LOAN), "no result");
    assert!(repo.store_files().is_empty() && repo.outbox_files().is_empty());
    limit_bounds(&run, "adapter", 60);
    assert_tree_stopped(&run, "adapter");
}

#[test]
fn an_omitted_limit_allows_a_launch_longer_than_one_minute() {
    let repo = Repo::sample();
    assert!(!repo.read(PROJECT_FILE).contains("time_limit_minutes"));
    repo.append(LATE_FINE_TESTS, "adapter: sleeps 70000\n");
    let mut run = LiveRun::start(&repo, &["run", "--json"], "default-over-one-minute");
    let output = run.finish(Duration::from_secs(100));
    output.expect_status(0);
    let doc = output.json();
    assert_eq!(doc["suites"][0]["launch"], "ok");
    assert!(doc["suites"][0]["duration_ms"].as_u64().unwrap() >= 70_000);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&doc, check), "passed");
        assert_eq!(repo.result_of(check)["body"]["outcome"], "passed");
    }
    assert_eq!(repo.records().len(), 4);
}
// behavior-2: A process of the group that is still alive after the grace is killed with SIGKILL, together with the rest of the group
// behavior-2: The grace is 10 seconds and no setting changes it
#[test]
fn a_group_ignoring_sigterm_is_killed_after_the_fixed_grace() {
    let repo = tree("resists", Some(1));
    let mut run = LiveRun::start(&repo, &["run", "--json"], "resistant");
    let output = run.finish(Duration::from_secs(240));
    no_results(&repo, &output);
    assert_tree_stopped(&run, "adapter");
    limit_bounds(&run, "adapter", 60);
    for role in ["adapter", "runner"] {
        grace_bounds(&run, role);
    }
}
#[test]
fn escalation_reaches_a_resistant_grandchild_after_the_adapter_exits() {
    let repo = tree("orphan-resists", Some(1));
    let mut run = LiveRun::start(&repo, &["run", "--json"], "orphan-resistant");
    let output = run.finish(Duration::from_secs(240));
    no_results(&repo, &output);
    assert_tree_stopped(&run, "adapter");
    assert!(stamp(&run, "adapter", "exited") < stamp(&run, "grandchild", "observed-exit"));
    grace_bounds(&run, "grandchild");
}
// behavior-2: A test that had finished before the limit gets no result either, and nothing the adapter wrote is read
#[test]
fn a_complete_report_written_before_a_hang_is_discarded_at_the_limit() {
    let repo = tree("partial", Some(1));
    let mut run = LiveRun::start(&repo, &["run", "--json"], "partial");
    let output = run.finish(Duration::from_secs(240));
    no_results(&repo, &output);
    stamp(&run, "adapter", "finished-tests");
    assert_tree_stopped(&run, "adapter");
}
// behavior-2: The core starts each adapter and each proof in a process group of its own
// behavior-2: A proof still counts as hung after 10 seconds, whatever the suite's limit is
#[test]
fn proofs_have_their_own_group_and_ten_second_limit() {
    let mut repo = tree("orphan", Some(2));
    // Make the old late-start measurement fail deterministically, without changing the CLI clock.
    repo.set_env("TREE_SETUP_DELAY_MS", "1500");
    repo.need_facility("present");
    let token = repo.beside("tree-token").display().to_string();
    repo.replace(
        PROJECT_FILE,
        "proof = \"facility-probe present\"",
        &format!("proof = \"process-tree proof {token}\""),
    );
    let mut run = LiveRun::start(&repo, &["run", "--json"], "proof");
    let output = run.finish(Duration::from_secs(150));
    output.expect_status(0);
    for check in ALL_CHECKS {
        assert_eq!(reason_of(&output.json(), check), "cannot run here");
    }
    assert_tree_stopped(&run, "proof");
    limit_bounds(&run, "proof", 10);
    assert!(stamp(&run, "proof", "started") - stamp(&run, "proof", "entered") >= 1_500_000_000);
    assert!(!run.events().iter().any(|e| e["role"] == "adapter"));
}
#[test]
fn suites_keep_their_own_limits_and_results_when_one_times_out() {
    let mut repo = tree("orphan", Some(1));
    repo.split_returns_suite("");
    repo.append(PROJECT_FILE, "time_limit_minutes = 2\n");
    repo.append(
        "suites/returns-checks/late-fine.test",
        "adapter: sleeps 70000\n",
    );
    repo.set_env("TREE_MODE", "orphan");
    let mut run = LiveRun::start(&repo, &["run", "--json"], "mixed");
    let output = run.finish(Duration::from_secs(300));
    output.expect_status(1);
    let doc = output.json();
    let suites = doc["suites"].as_array().unwrap();
    assert_eq!(suites.len(), 2);
    for (name, launch) in [("desk-checks", "timed out"), ("returns-checks", "ok")] {
        let s: Vec<_> = suites.iter().filter(|s| s["name"] == name).collect();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0]["launch"], launch);
    }
    for check in [SIXTH_LOAN, RETURNED_BOOK] {
        assert_eq!(state_of(&doc, check), "owed");
        assert_eq!(reason_of(&doc, check), "no result");
        assert!(repo.results_of(check).is_empty());
    }
    for check in [LATE_RETURN, ON_TIME_RETURN] {
        assert_eq!(state_of(&doc, check), "passed");
        assert_eq!(repo.result_of(check)["body"]["outcome"], "passed");
    }
    assert_eq!(repo.records().len(), 2);
    assert_eq!(repo.store_files().len() + repo.outbox_files().len(), 2);
    limit_bounds(&run, "adapter", 60);
    assert!(
        suites
            .iter()
            .find(|s| s["name"] == "returns-checks")
            .unwrap()["duration_ms"]
            .as_u64()
            .unwrap()
            >= 70_000
    );
    assert_tree_stopped(&run, "adapter");
}
// behavior-2: The limit of a suite is the key `time_limit_minutes` in the project file, a whole number
// behavior-2: The key is optional and belongs to a suite, not to the project
#[test]
fn the_optional_limit_belongs_to_each_suite() {
    let repo = Repo::sample();
    repo.split_returns_suite("");
    limited(&repo, "2");
    repo.replace(
        PROJECT_FILE,
        "needs = []",
        "needs = []\ntime_limit_minutes = 1",
    );
    repo.run_json(&[], 0);
    let top = Repo::sample();
    top.replace(
        PROJECT_FILE,
        "[repository]",
        "time_limit_minutes = 1\n\n[repository]",
    );
    let error = top.expect_reading_error(
        Some(PROJECT_FILE),
        Some(top.line_of(PROJECT_FILE, "time_limit_minutes")),
    );
    assert_eq!(error["format"], "error-1");
}

// behavior-2: A `time_limit_minutes` of zero, a negative one, or one that is not a whole number is a reading error
#[test]
fn invalid_limits_are_reading_errors_before_any_launch() {
    for value in ["0", "-1", "1.5", "1.0", "\"1\"", "true"] {
        let repo = Repo::sample();
        limited(&repo, value);
        let error = repo.expect_reading_error(
            Some(PROJECT_FILE),
            Some(repo.line_of(PROJECT_FILE, "time_limit_minutes")),
        );
        assert_eq!(error["format"], "error-1");
        repo.azimuth(&["run", "--json"]).expect_status(2);
        assert!(repo.adapter_starts().is_empty());
        assert!(repo.store_files().is_empty() && repo.outbox_files().is_empty());
    }
}

// behavior-2: The request holds no time limit
// behavior-2: The formats `adapter-request-1` and `adapter-report-1` keep their numbers
#[test]
fn the_limit_never_reaches_the_adapter_contract() {
    let repo = Repo::sample();
    limited(&repo, "1");
    repo.run_json(&[], 0);
    let starts = repo.adapter_runs();
    assert_eq!(starts.len(), 1);
    let mut ids = ALL_CHECKS.to_vec();
    ids.sort();
    let mut request = starts[0]["request"].clone();
    let mut checks = strings_of(&request["checks"]);
    checks.sort();
    assert_eq!(checks, ids);
    request["checks"] = json!(checks);
    assert_eq!(
        request,
        json!({"format":"adapter-request-1", "verb":"run", "suite":"suites/desk-checks", "marker_key":"Check", "checks": ids})
    );
    let records = repo.records();
    assert_eq!(records.len(), ALL_CHECKS.len());
    assert_eq!(
        repo.store_files().len() + repo.outbox_files().len(),
        ALL_CHECKS.len()
    );
    for (check, claim) in [
        (
            SIXTH_LOAN,
            "The desk refuses a sixth loan to a borrower who holds five.",
        ),
        (
            RETURNED_BOOK,
            "After one return the desk lends again to a borrower who held five loans.",
        ),
        (
            LATE_RETURN,
            "A return three days late records a fine of three days.",
        ),
        (
            ON_TIME_RETURN,
            "A return on the due date or earlier records no fine.",
        ),
    ] {
        let record = repo.result_of(check);
        assert_eq!(record["format"], "result-1");
        assert_eq!(
            record["subjects"],
            json!([{"kind":"check", "name":check, "digest":text_1_digest(claim)}])
        );
        assert_eq!(record["body"]["outcome"], "passed");
    }
}

// behavior-2: A change of the number shows as class D
#[test]
fn raising_the_limit_changes_intent() {
    let repo = Repo::sample();
    limited(&repo, "1");
    repo.commit_all("limit the sample suite");
    repo.replace(
        PROJECT_FILE,
        "time_limit_minutes = 1",
        "time_limit_minutes = 2",
    );
    let output = repo.azimuth(&["change", "diff", "--json"]);
    output.expect_status(0);
    assert_eq!(output.json()["class"], "D");
}

#[test]
fn no_setting_changes_the_grace() {
    for key in ["time_limit_grace_seconds", "grace_seconds"] {
        let repo = Repo::sample();
        repo.append(PROJECT_FILE, &format!("{key} = 1\n"));
        repo.expect_reading_error(Some(PROJECT_FILE), Some(repo.line_of(PROJECT_FILE, key)));
    }
}
