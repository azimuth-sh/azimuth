use azimuth_tests::*;
use std::fs;
use std::time::Duration;
#[allow(dead_code)]
mod support;
use support::*;

const WATCHDOG: Duration = Duration::from_secs(180);
fn gated(two: bool) -> Repo {
    let mut repo = Repo::sample();
    if two {
        repo.split_returns_suite("");
    }
    let folder = repo.beside("output-gates");
    repo.set_env("TEXT_ADAPTER_GATE_DIR", folder.to_str().unwrap());
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        &format!("adapter = \"text-suite-adapter {}\"", folder.display()),
    );
    if two {
        repo.replace(
            PROJECT_FILE,
            "adapter = \"text-suite-adapter\"",
            &format!("adapter = \"text-suite-adapter {}\"", folder.display()),
        );
    }
    repo
}
fn payload(suite: &str, phase: &str) -> String {
    format!("AZIMUTH-OUTPUT-{suite}-{phase}-731")
}
fn gate(repo: &Repo, suite: &str, phase: &str) -> std::path::PathBuf {
    repo.beside("output-gates").join(suite).join(phase)
}
fn release(repo: &Repo, suite: &str, phase: &str) {
    fs::write(gate(repo, suite, &format!("{phase}.release")), "").unwrap();
}
fn no_payload(output: &Output, suite: &str) {
    for phase in ["first", "second"] {
        let text = payload(suite, phase);
        assert!(
            !output.stdout.contains(&text) && !output.stderr.contains(&text),
            "runner output escaped: {}",
            output.text()
        );
    }
}
fn held(repo: &Repo, text: &str) -> Vec<std::path::PathBuf> {
    files_under(&repo.path(".azimuth"))
        .into_iter()
        .filter(|file| fs::read_to_string(file).unwrap_or_default().contains(text))
        .collect()
}
fn checkpoint(repo: &Repo, run: &mut LiveRun, suite: &str, phase: &str) {
    run.wait_until(WATCHDOG, |_| {
        gate(repo, suite, &format!("{phase}.ready")).exists()
    });
    let pid: u32 = fs::read_to_string(gate(repo, suite, &format!("{phase}.ready")))
        .unwrap()
        .parse()
        .unwrap();
    let table = azimuth_tests::process::processes().unwrap_or_else(|error| panic!("{error}"));
    let adapter = table
        .get(&pid)
        .unwrap_or_else(|| panic!("gated adapter {pid} exited before observation"));
    assert!(adapter.running());
    assert!(adapter
        .command
        .split_whitespace()
        .any(|arg| arg == repo.beside("output-gates").to_str().unwrap()));
    // Keep the writer blocked while a consumer scheduled after the ready file can read its pipe.
    let observe = std::time::Instant::now();
    loop {
        no_payload(&run.text(), suite);
        if observe.elapsed() >= Duration::from_secs(1) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let table = azimuth_tests::process::processes().unwrap_or_else(|error| panic!("{error}"));
    assert!(
        table.get(&pid).is_some_and(|process| process.running()),
        "adapter exited while its gate was closed"
    );
}
fn drive(repo: &Repo, run: &mut LiveRun, suite: &str) {
    for phase in ["first", "second", "exit"] {
        checkpoint(repo, run, suite, phase);
        release(repo, suite, phase);
    }
}
fn drive_all(repo: &Repo, run: &mut LiveRun, suites: &[&str]) {
    let phases = ["first", "second", "exit"];
    let mut next = vec![0; suites.len()];
    while next.iter().any(|phase| *phase < 3) {
        run.wait_until(WATCHDOG, |_| {
            suites.iter().enumerate().any(|(index, suite)| {
                next[index] < 3
                    && gate(repo, suite, &format!("{}.ready", phases[next[index]])).exists()
            })
        });
        for (index, suite) in suites.iter().enumerate() {
            if next[index] < 3
                && gate(repo, suite, &format!("{}.ready", phases[next[index]])).exists()
            {
                checkpoint(repo, run, suite, phases[next[index]]);
                release(repo, suite, phases[next[index]]);
                next[index] += 1;
            }
        }
    }
}

fn expected(suite: &str, kind: &str) -> String {
    let prefix = format!(
        "{}\n{}\n",
        payload(suite, "first"),
        payload(suite, "second")
    );
    let body=match kind {
        "crashed"=>"text-suite-adapter: crashing on request of the suite\n".to_string(),
        "build failed"=>"text-suite-adapter: the suite does not build, on request of the suite\n".to_string(),
        _ if suite=="desk-checks"=>"text-suite-adapter: free-return.returned-on-due-date passed\ntext-suite-adapter: free-return.returned-early passed\ntext-suite-adapter: late-fine failed\ntext-suite-adapter: loan-limit.refuses-sixth-loan passed\ntext-suite-adapter: loan-limit.lends-after-a-return passed\n".to_string(),
        _=>"text-suite-adapter: free-return.returned-on-due-date passed\ntext-suite-adapter: free-return.returned-early passed\ntext-suite-adapter: late-fine failed\n".to_string(),
    };
    format!("{prefix}{body}")
}
fn trouble(directive: Option<&str>, kind: &str) {
    for json in [false, true] {
        let repo = gated(false);
        if let Some(directive) = directive {
            repo.append(LATE_FINE_TESTS, &format!("adapter: {directive}\n"));
        } else {
            repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
        }
        let args = if json {
            vec!["run", "--json"]
        } else {
            vec!["run"]
        };
        let mut run = LiveRun::start(&repo, &args, "trouble");
        drive(&repo, &mut run, "desk-checks");
        let output = run.finish(WATCHDOG);
        output.expect_status(1);
        if json {
            assert_eq!(output.json()["suites"][0]["launch"], kind);
        }
        let block = expected("desk-checks", kind);
        assert_eq!(
            output.stderr.matches(&block).count(),
            1,
            "whole output missing or split:\n{}",
            output.stderr
        );
        for phase in ["first", "second"] {
            assert_eq!(
                output
                    .stderr
                    .matches(&payload("desk-checks", phase))
                    .count(),
                1
            );
            assert!(!output.stdout.contains(&payload("desk-checks", phase)));
        }
    }
}
// behavior-2: A launch where every test passed prints none of the runner's output
// behavior-2: With `--json` standard output holds only the one document, and the held output goes to standard error under the same rule
#[test]
fn a_passing_launch_keeps_all_runner_output_off_both_streams() {
    for json in [false, true] {
        let repo = gated(false);
        let args = if json {
            vec!["run", "--json"]
        } else {
            vec!["run"]
        };
        let mut run = LiveRun::start(&repo, &args, "passing");
        drive(&repo, &mut run, "desk-checks");
        let output = run.finish(WATCHDOG);
        output.expect_status(0);
        no_payload(&output, "desk-checks");
        if json {
            assert_eq!(output.json()["format"], "run-1");
        }
    }
}
// behavior-2: A launch that crashed prints the runner's whole output on standard error after it ended
#[test]
fn a_crashed_launch_releases_the_whole_output_after_exit() {
    trouble(Some("crash"), "crashed");
}
// behavior-2: A launch whose build failed prints the output
#[test]
fn a_build_failure_releases_the_whole_output() {
    trouble(Some("build-fails"), "build failed");
}
// behavior-2: A launch with a failed test prints the output, also when other tests passed
#[test]
fn one_failed_test_releases_output_from_passing_tests_too() {
    trouble(None, "ok");
}
// behavior-2: The output of a launch is printed in one piece, after the launch ended, and not mixed with another launch
#[test]
fn two_troubled_launches_release_separate_unbroken_blocks() {
    for json in [false, true] {
        let repo = gated(true);
        repo.replace(LOAN_LIMIT_TESTS, "outcome: passed", "outcome: failed");
        repo.replace(
            "suites/returns-checks/late-fine.test",
            "outcome: passed",
            "outcome: failed",
        );
        let args = if json {
            vec!["run", "--json"]
        } else {
            vec!["run"]
        };
        let mut run = LiveRun::start(&repo, &args, "two-troubled");
        drive_all(&repo, &mut run, &["desk-checks", "returns-checks"]);
        let output = run.finish(WATCHDOG);
        output.expect_status(1);
        if json {
            output.json();
        }
        let a=format!("{}\n{}\ntext-suite-adapter: loan-limit.refuses-sixth-loan failed\ntext-suite-adapter: loan-limit.lends-after-a-return passed\n",payload("desk-checks","first"),payload("desk-checks","second"));
        for block in [a, expected("returns-checks", "ok")] {
            assert_eq!(
                output.stderr.matches(&block).count(),
                1,
                "output split or duplicated:\n{}",
                output.stderr
            );
        }
    }
}
#[test]
fn a_passing_suite_stays_silent_beside_a_troubled_suite() {
    for json in [false, true] {
        let repo = gated(true);
        repo.replace(
            "suites/returns-checks/late-fine.test",
            "outcome: passed",
            "outcome: failed",
        );
        let args = if json {
            vec!["run", "--json"]
        } else {
            vec!["run"]
        };
        let mut run = LiveRun::start(&repo, &args, "mixed-output");
        drive_all(&repo, &mut run, &["desk-checks", "returns-checks"]);
        let output = run.finish(WATCHDOG);
        output.expect_status(1);
        if json {
            output.json();
        }
        no_payload(&output, "desk-checks");
        assert_eq!(
            output
                .stderr
                .matches(&expected("returns-checks", "ok"))
                .count(),
            1
        );
    }
}
// behavior-2: The core prints one line when a launch starts and one when it ends
#[test]
fn progress_has_a_start_line_before_exit_and_an_end_line_after_it() {
    for json in [false, true] {
        let repo = gated(true);
        let args = if json {
            vec!["run", "--json"]
        } else {
            vec!["run"]
        };
        let mut run = LiveRun::start(&repo, &args, "progress");
        run.wait_until(WATCHDOG, |_| {
            ["desk-checks", "returns-checks"]
                .iter()
                .any(|suite| gate(&repo, suite, "first.ready").exists())
        });
        let snapshot = run.text();
        let progress = if json {
            snapshot.stderr
        } else {
            snapshot.text()
        };
        let started = repo.adapter_starts().len();
        assert!(
            progress
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
                >= started,
            "launches have no nonempty start progress: {progress}"
        );
        drive_all(&repo, &mut run, &["desk-checks", "returns-checks"]);
        let output = run.finish(WATCHDOG);
        output.expect_status(0);
        no_payload(&output, "desk-checks");
        no_payload(&output, "returns-checks");
        if json {
            output.json();
            assert_eq!(
                output
                    .stderr
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .count(),
                4
            );
        } else {
            let progress: Vec<_> = output
                .text()
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter(|line| !ALL_CHECKS.iter().any(|id| line.contains(id)))
                .map(str::to_string)
                .collect();
            assert!(
                progress.len() >= 4,
                "two launches need four nonempty progress lines: {progress:#?}"
            );
        }
    }
}

// behavior-2: While a launch runs, its output is in a file under `.azimuth/`
// behavior-2: After a launch with no trouble that file is gone
#[test]
fn held_output_is_on_disk_during_the_launch_and_removed_after_success() {
    let repo = gated(false);
    let mut run = LiveRun::start(&repo, &["run", "--json"], "held-file");
    checkpoint(&repo, &mut run, "desk-checks", "first");
    run.wait_until(WATCHDOG, |_| {
        held(&repo, &payload("desk-checks", "first")).len() == 1
    });
    let files = held(&repo, &payload("desk-checks", "first"));
    let file = &files[0];
    let initial = fs::metadata(file).unwrap().len();
    release(&repo, "desk-checks", "first");
    checkpoint(&repo, &mut run, "desk-checks", "second");
    run.wait_until(WATCHDOG, |_| {
        fs::read_to_string(file)
            .unwrap()
            .contains(&payload("desk-checks", "second"))
    });
    assert!(fs::metadata(file).unwrap().len() > initial);
    release(&repo, "desk-checks", "second");
    checkpoint(&repo, &mut run, "desk-checks", "exit");
    release(&repo, "desk-checks", "exit");
    run.finish(WATCHDOG).expect_status(0);
    assert!(!file.exists());
    assert!(held(&repo, &payload("desk-checks", "first")).is_empty());
    assert!(held(&repo, &payload("desk-checks", "second")).is_empty());
}

#[test]
fn concurrent_clis_keep_their_held_output_separate() {
    for json in [false, true] {
        let repo = gated(true);
        repo.replace(
            "suites/returns-checks/late-fine.test",
            "outcome: passed",
            "outcome: failed",
        );
        let mut pass_args = vec!["check", "run", SIXTH_LOAN];
        let mut trouble_args = vec!["check", "run", LATE_RETURN];
        if json {
            pass_args.push("--json");
            trouble_args.push("--json");
        }
        let mut passing = LiveRun::start(&repo, &pass_args, "concurrent-pass");
        let mut troubled = LiveRun::start(&repo, &trouble_args, "concurrent-trouble");
        // Both files must coexist before either command is allowed to finish.
        checkpoint(&repo, &mut passing, "desk-checks", "first");
        checkpoint(&repo, &mut troubled, "returns-checks", "first");
        let pass_files = held(&repo, &payload("desk-checks", "first"));
        let trouble_files = held(&repo, &payload("returns-checks", "first"));
        assert_eq!(pass_files.len(), 1);
        assert_eq!(trouble_files.len(), 1);
        assert_ne!(pass_files, trouble_files);
        release(&repo, "desk-checks", "first");
        release(&repo, "returns-checks", "first");
        for phase in ["second", "exit"] {
            checkpoint(&repo, &mut passing, "desk-checks", phase);
            checkpoint(&repo, &mut troubled, "returns-checks", phase);
            release(&repo, "desk-checks", phase);
            release(&repo, "returns-checks", phase);
        }
        let passing = passing.finish(WATCHDOG);
        let troubled = troubled.finish(WATCHDOG);
        passing.expect_status(0);
        troubled.expect_status(1);
        for suite in ["desk-checks", "returns-checks"] {
            no_payload(&passing, suite);
        }
        assert!(!passing.text().contains("text-suite-adapter:"));
        no_payload(&troubled, "desk-checks");
        assert!(!troubled.text().contains("text-suite-adapter: loan-limit."));
        assert!(!troubled.stdout.contains("text-suite-adapter:"));
        let expected = format!(
            "{}\n{}\ntext-suite-adapter: late-fine failed\n",
            payload("returns-checks", "first"),
            payload("returns-checks", "second")
        );
        assert_eq!(
            troubled.stderr.matches(&expected).count(),
            1,
            "concurrent trouble output was lost or split:\n{}",
            troubled.stderr
        );
        for phase in ["first", "second"] {
            assert_eq!(
                troubled
                    .stderr
                    .matches(&payload("returns-checks", phase))
                    .count(),
                1
            );
            assert!(!troubled.stdout.contains(&payload("returns-checks", phase)));
        }
        if json {
            passing.json();
            troubled.json();
        }
    }
}

#[test]
fn an_unwritable_capture_directory_is_a_command_error_before_launch() {
    let repo = Repo::sample();
    make_unwritable(&repo.path(".azimuth"));
    let output = repo.azimuth(&["run", "--json"]);
    set_mode(&repo.path(".azimuth"), 0o755);
    output.expect_status(2);
    assert!(
        !output.stderr.trim().is_empty(),
        "command error has no standard-error message"
    );
    assert!(
        repo.adapter_starts().is_empty(),
        "capture failure started an adapter"
    );
    assert!(repo.store_files().is_empty() && repo.outbox_files().is_empty());
    if !output.stdout.trim().is_empty() {
        let doc = output.json();
        assert!(
            doc.get("suites").is_none(),
            "capture failure reported a launch: {doc:#}"
        );
        assert_ne!(doc["format"], "run-1");
    }
}

#[test]
fn end_progress_is_a_separate_line_after_runner_output_without_a_newline() {
    let mut repo = Repo::sample();
    let tail = "FINAL-RUNNER-BYTES-WITHOUT-NEWLINE-8451";
    repo.set_env("TEXT_ADAPTER_TAIL", tail);
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let output = repo.azimuth(&["run", "--json"]);
    output.expect_status(1);
    assert_eq!(output.json()["suites"][0]["launch"], "ok");
    assert!(
        output.stderr.lines().any(|line| line == tail),
        "runner tail and launch end must occupy separate lines:\n{}",
        output.stderr
    );
    let progress: Vec<_> = output
        .stderr
        .lines()
        .filter(|line| {
            !line.is_empty() && *line != tail && !line.starts_with("text-suite-adapter:")
        })
        .collect();
    assert_eq!(
        progress.len(),
        2,
        "launch needs separate start and end lines: {progress:#?}"
    );
}

#[test]
fn launch_duration_excludes_slow_replay_of_large_held_output() {
    use std::io::{Read, Write};
    use std::process::Stdio;
    use std::time::Instant;
    let mut repo = Repo::sample();
    repo.set_env("TEXT_ADAPTER_BULK_BYTES", "8388608");
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    let stdout = repo.beside("slow-reader.stdout");
    let stderr = repo.beside("slow-reader.stderr");
    let mut child = repo
        .azimuth_command(&["run", "--json"])
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut pipe = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut file = fs::File::create(stderr).unwrap();
        let mut buffer = [0; 65536];
        let mut pending = 0;
        let mut payload_bytes = 0;
        let mut replay_start = None;
        let mut replay_end = 0;
        loop {
            let count = pipe.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            let payload = buffer[..count].iter().filter(|byte| **byte == b'X').count();
            if payload != 0 {
                let now = azimuth_tests::process::monotonic_ns();
                replay_start.get_or_insert(now);
                replay_end = now;
            }
            payload_bytes += payload;
            file.write_all(&buffer[..count]).unwrap();
            pending += count;
            if pending >= buffer.len() {
                pending -= buffer.len();
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        (payload_bytes, replay_start.unwrap(), replay_end)
    });
    let began = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if began.elapsed() > Duration::from_secs(90) {
            child.kill().unwrap();
            child.wait().unwrap();
            reader.join().unwrap();
            panic!("CLI did not finish replay within the independent reader watchdog");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let returned = azimuth_tests::process::monotonic_ns();
    let (payload_bytes, replay_start, replay_end) = reader.join().unwrap();
    assert_eq!(payload_bytes, 8 * 1024 * 1024);
    assert_eq!(status.code(), Some(1));
    let doc: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(stdout).unwrap()).unwrap();
    assert_eq!(doc["suites"][0]["launch"], "ok");
    let starts = repo.adapter_starts();
    assert_eq!(starts.len(), 1);
    let ends: Vec<_> = repo
        .log_entries()
        .into_iter()
        .filter(|entry| entry["program"] == "text-suite-adapter" && entry["event"] == "exited")
        .collect();
    assert_eq!(ends.len(), 1);
    let exited = ends[0]["time_ns"].as_u64().unwrap();
    let adapter_ms = (exited - starts[0]["time_ns"].as_u64().unwrap()) / 1_000_000;
    assert!(
        returned - exited >= 4_000_000_000,
        "slow consumer did not impose a measurable replay interval"
    );
    let reported_ms = doc["suites"][0]["duration_ms"].as_u64().unwrap();
    // Half the observed replay interval allows scheduling gaps on a busy host. A duration
    // charged for the whole replay still exceeds that allowance, however slow the reader is.
    let replay_ms = (replay_end - replay_start) / 1_000_000;
    assert!(
        replay_ms >= 4000,
        "reader did not impose four seconds of replay"
    );
    assert!(reported_ms.abs_diff(adapter_ms) <= replay_ms / 2,
        "duration_ms includes output replay: reported {reported_ms} ms, adapter journal {adapter_ms} ms");
}

#[test]
fn late_stdout_does_not_change_the_report_written_before_adapter_exit() {
    assert_gated_late_write("stdout");
}

#[test]
fn late_stderr_does_not_change_the_held_output_from_its_first_byte() {
    assert_gated_late_write("stderr");
}

fn assert_gated_late_write(stream: &str) {
    use azimuth_tests::process::processes;
    let mut repo = Repo::sample();
    let token = repo.beside("tree-token");
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        &format!("adapter = \"process-tree adapter {}\"", token.display()),
    );
    repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
    repo.set_env("TREE_MODE", "late-output");
    repo.set_env("TREE_LOG", repo.beside("tree.jsonl").to_str().unwrap());
    repo.set_env(
        "TREE_REQUEST",
        repo.beside("request.json").to_str().unwrap(),
    );
    repo.set_env("TREE_LATE_STREAM", stream);
    let mut run = LiveRun::start_with_replay_gate(&repo, &["run", "--json"], "late-write");
    run.wait_until(Duration::from_secs(30), |_| {
        repo.beside("replay.ready").exists()
    });
    let exited = run
        .events()
        .into_iter()
        .find(|event| event["role"] == "adapter" && event["event"] == "exited")
        .unwrap();
    run.wait_until(Duration::from_secs(30), |_| {
        let table = processes().unwrap_or_else(|error| panic!("{error}"));
        !table
            .get(&(exited["pid"].as_u64().unwrap() as u32))
            .is_some_and(|process| process.running())
    });
    // Backpressure keeps the replay in progress; the write happens only after actual adapter exit.
    fs::write(token.with_extension("late-write"), "").unwrap();
    run.wait_until(Duration::from_secs(30), |run| {
        run.events()
            .iter()
            .any(|event| event["event"] == "late-written")
    });
    fs::write(token.with_extension("stop"), "").unwrap();
    fs::write(repo.beside("replay.release"), "").unwrap();
    let output = run.finish_cli(Duration::from_secs(60));
    output.expect_status(1);
    let doc = output.json();
    assert_eq!(doc["suites"][0]["launch"], "ok");
    for check in ALL_CHECKS {
        let expected = if check == LATE_RETURN {
            "failed"
        } else {
            "passed"
        };
        assert_eq!(state_of(&doc, check), expected);
        assert_eq!(repo.result_of(check)["body"]["outcome"], expected);
    }
    for (check, outcome, names) in [
        (SIXTH_LOAN, "passed", vec!["loan-limit.refuses-sixth-loan"]),
        (
            RETURNED_BOOK,
            "passed",
            vec!["loan-limit.lends-after-a-return"],
        ),
        (LATE_RETURN, "failed", vec!["late-fine"]),
        (
            ON_TIME_RETURN,
            "passed",
            vec![
                "free-return.returned-early",
                "free-return.returned-on-due-date",
            ],
        ),
    ] {
        let record = repo.result_of(check);
        let mut actual = record["body"]["tests"].as_array().unwrap().clone();
        actual.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        let mut expected: Vec<_> = names
            .iter()
            .map(|name| serde_json::json!({"name":name,"outcome":outcome,"duration_ms":1}))
            .collect();
        expected.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        assert_eq!(
            actual, expected,
            "late writer changed the report's enrolled tests for {check}"
        );
        assert_eq!(
            record["body"]["method"]["adapter"],
            serde_json::json!({"name":"text-suite-adapter","version":"1.0.0"})
        );
        assert_eq!(checks_by_id(&doc)[check]["duration_ms"], names.len());
    }
    assert_eq!(repo.records().len(), 4);
    assert!(!output.text().contains("LATE-STDOUT-CORRUPTION"));
    assert!(!output.text().contains("LATE-STDERR-CORRUPTION"));
    let first = "TREE-HELD-FIRST-728\n";
    let last = "\nTREE-HELD-LAST-728\n";
    assert_eq!(output.stderr.matches(first).count(), 1);
    assert_eq!(output.stderr.matches(last).count(), 1);
    let start = output.stderr.find(first).unwrap();
    let end = output.stderr.find(last).unwrap() + last.len();
    let mut expected = first.as_bytes().to_vec();
    expected.extend(vec![b'X'; 8 * 1024 * 1024]);
    expected.extend_from_slice(last.as_bytes());
    assert_eq!(
        sha256_hex(&output.stderr.as_bytes()[start..end]),
        sha256_hex(&expected),
        "held output differs from the complete bytes written before adapter exit ({stream})"
    );
}

#[test]
fn a_missing_temporary_directory_leaves_no_launch_log_after_run() {
    let repo = Repo::sample();
    let missing = repo.beside("missing-temporary-directory");
    assert!(!missing.exists());
    let output =
        repo.azimuth_with_env(&["run", "--json"], &[("TMPDIR", missing.to_str().unwrap())]);
    assert!(
        matches!(output.status, 0 | 2),
        "local preparation must work or be a command error: {}",
        output.text()
    );
    if output.status == 0 {
        assert_eq!(output.json()["suites"][0]["launch"], "ok");
        for check in ALL_CHECKS {
            assert_eq!(state_of(&output.json(), check), "passed");
        }
    } else {
        assert!(!output.stderr.trim().is_empty());
        assert!(repo.adapter_starts().is_empty());
    }
    let logs: Vec<_> = files_under(&repo.path(".azimuth"))
        .into_iter()
        .filter(|file| {
            let name = file.file_name().unwrap().to_str().unwrap();
            name.starts_with("launch-") && name.ends_with(".log")
        })
        .collect();
    assert!(
        logs.is_empty(),
        "preparation failure left launch logs: {logs:#?}"
    );
}

#[test]
fn unterminated_trouble_output_does_not_join_the_next_launch_line() {
    for json in [false, true] {
        let mut repo = Repo::sample();
        repo.split_returns_suite("");
        repo.replace(LOAN_LIMIT_TESTS, "outcome: passed", "outcome: failed");
        repo.replace(
            "suites/returns-checks/late-fine.test",
            "outcome: passed",
            "outcome: failed",
        );
        let tail = "LAST-RUNNER-BYTE-BEFORE-NEXT-LAUNCH-9572";
        repo.set_env("TEXT_ADAPTER_TAIL", tail);
        let args = if json {
            vec!["run", "--json"]
        } else {
            vec!["run"]
        };
        let output = repo.azimuth(&args);
        output.expect_status(1);
        if json {
            assert_eq!(output.json()["suites"].as_array().unwrap().len(), 2);
        }
        assert_eq!(output.stderr.matches(tail).count(), 2);
        let desk = format!("text-suite-adapter: loan-limit.refuses-sixth-loan failed\ntext-suite-adapter: loan-limit.lends-after-a-return passed\n{tail}");
        let returns = format!("text-suite-adapter: free-return.returned-on-due-date passed\ntext-suite-adapter: free-return.returned-early passed\ntext-suite-adapter: late-fine failed\n{tail}");
        let mut progress = output.stderr.clone();
        for block in [desk, returns] {
            assert_eq!(output.stderr.matches(&block).count(), 1);
            progress = progress.replace(&block, "");
        }
        let core_lines: Vec<_> = progress
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        if json {
            assert_eq!(core_lines.len(), 4);
        }
        // Learn the messages by removing our exact fixture bytes, without fixing their wording
        // or the plain form's stream; each message must already have its own line in the raw stream.
        for line in core_lines {
            assert_eq!(
                output
                    .stderr
                    .lines()
                    .filter(|actual| *actual == line)
                    .count(),
                1,
                "core progress is not a line of its own after unterminated output: {line}"
            );
        }
    }
}

#[test]
fn concurrent_troubled_clis_of_the_same_suite_keep_distinct_complete_output() {
    for json in [false, true] {
        let mut repo = gated(false);
        repo.replace(LATE_FINE_TESTS, "outcome: passed", "outcome: failed");
        let roots = [
            repo.beside("output-gates").join("one"),
            repo.beside("output-gates").join("two"),
        ];
        let tags = ["OWNED-BY-FIRST-CLI-481", "OWNED-BY-SECOND-CLI-926"];
        let args = if json {
            vec!["run", "--json"]
        } else {
            vec!["run"]
        };
        repo.set_env("TEXT_ADAPTER_GATE_DIR", roots[0].to_str().unwrap());
        repo.set_env("TEXT_ADAPTER_OUTPUT_TAG", tags[0]);
        let mut first = LiveRun::start(&repo, &args, "same-suite-first");
        repo.set_env("TEXT_ADAPTER_GATE_DIR", roots[1].to_str().unwrap());
        repo.set_env("TEXT_ADAPTER_OUTPUT_TAG", tags[1]);
        let mut second = LiveRun::start(&repo, &args, "same-suite-second");
        let runs = [&mut first, &mut second];
        for phase in ["first", "second", "exit"] {
            for index in 0..2 {
                let folder = roots[index].join("desk-checks");
                runs[index]
                    .wait_until(WATCHDOG, |_| folder.join(format!("{phase}.ready")).exists());
                for tag in tags {
                    assert!(!runs[index].text().text().contains(tag));
                }
            }
            if phase == "first" {
                let files: Vec<_> = tags
                    .iter()
                    .map(|tag| {
                        let files = held(&repo, tag);
                        assert_eq!(files.len(), 1);
                        files[0].clone()
                    })
                    .collect();
                assert_ne!(
                    files[0], files[1],
                    "same-suite commands shared one held file"
                );
            }
            for root in &roots {
                fs::write(
                    root.join("desk-checks").join(format!("{phase}.release")),
                    "",
                )
                .unwrap();
            }
        }
        let outputs = [runs[0].finish(WATCHDOG), runs[1].finish(WATCHDOG)];
        for (index, output) in outputs.iter().enumerate() {
            output.expect_status(1);
            if json {
                assert_eq!(output.json()["suites"][0]["launch"], "ok");
            }
            let block = expected("desk-checks", "ok").replace("731", tags[index]);
            assert_eq!(
                output.stderr.matches(&block).count(),
                1,
                "same-suite output was truncated or split:\n{}",
                output.stderr
            );
            assert!(!output.stdout.contains(tags[index]));
            assert!(
                !output.text().contains(tags[1 - index]),
                "one CLI printed the other's held output"
            );
            for phase in ["first", "second"] {
                assert_eq!(
                    output
                        .stderr
                        .matches(&format!(
                            "AZIMUTH-OUTPUT-desk-checks-{phase}-{}",
                            tags[index]
                        ))
                        .count(),
                    1
                );
            }
        }
    }
}
