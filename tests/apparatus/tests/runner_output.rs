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
        loop {
            let count = pipe.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            payload_bytes += buffer[..count].iter().filter(|byte| **byte == b'X').count();
            file.write_all(&buffer[..count]).unwrap();
            pending += count;
            if pending >= buffer.len() {
                pending -= buffer.len();
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        payload_bytes
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
    assert_eq!(reader.join().unwrap(), 8 * 1024 * 1024);
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
    // Journal entry/exit exclude exec and reaping; 1.5 s tolerates those scheduling gaps,
    // while the synchronized slow consumer imposes at least four seconds after adapter exit.
    assert!(reported_ms.abs_diff(adapter_ms) <= 1500,
        "duration_ms includes output replay: reported {reported_ms} ms, adapter journal {adapter_ms} ms");
}
