use azimuth_tests::*;
use std::time::Duration;
#[allow(dead_code)]
mod support;
use support::LiveRun;

// behavior-2: A suite with no such key has a limit of 30 minutes
#[test]
fn an_omitted_limit_stops_a_launch_at_thirty_minutes() {
    let mut repo = Repo::sample();
    let token = repo.beside("tree-token").display().to_string();
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        &format!("adapter = \"process-tree adapter {token}\""),
    );
    repo.set_env("TREE_MODE", "orphan");
    repo.set_env("TREE_LOG", repo.beside("tree.jsonl").to_str().unwrap());
    repo.set_env(
        "TREE_REQUEST",
        repo.beside("request.json").to_str().unwrap(),
    );
    let mut run = LiveRun::start(&repo, &["run", "--json"], "default-limit");
    let output = run.finish(Duration::from_secs(2100));
    output.expect_status(1);
    let doc = output.json();
    assert_eq!(doc["suites"][0]["launch"], "timed out");
    for check in ALL_CHECKS {
        assert_eq!(state_of(&doc, check), "owed");
        assert_eq!(reason_of(&doc, check), "no result");
    }
    assert!(repo.store_files().is_empty() && repo.outbox_files().is_empty());
    let events = run.events();
    let stamp = |kind: &str| {
        events
            .iter()
            .find(|e| e["role"] == "adapter" && e["event"] == kind)
            .unwrap()["time_ns"]
            .as_u64()
            .unwrap()
    };
    let elapsed = stamp("SIGTERM") - stamp("started");
    assert!(
        (1_799_500_000_000..=1_803_000_000_000).contains(&elapsed),
        "default after {elapsed} ns"
    );
    let starts: Vec<_> = events.iter().filter(|e| e["event"] == "started").collect();
    assert_eq!(starts.len(), 3);
    let group = starts[0]["group"].clone();
    assert_ne!(group, json!(run.cli_group));
    for start in starts {
        assert_eq!(start["group"], group);
        assert!(events
            .iter()
            .any(|e| e["event"] == "observed-exit" && e["pid"] == start["pid"]));
    }
}
