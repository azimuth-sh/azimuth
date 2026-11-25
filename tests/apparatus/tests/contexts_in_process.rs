//! The rows of the core in `docs/increments/contexts-in-process.md`: how the core reads the reads of a
//! host inside the test process, tells a host from a separate process by the phases of its reads,
//! compares the reads with the condition, and shows them in `run-1` and in results.
//!
//! The text-suite adapter stands in for a runner. Its directive `component <name> phases <phase>...`
//! gives a component the reads of a host (`built disposing`, or several pairs), and
//! `component <name> read <n> <change>` or `component <name> built <change>` changes one read or
//! all reads of a phase. The sample's `payouts.instant` is listed for `api` and `website`, so each
//! launch of it expects reads of both; a test makes `api` the host and leaves `website` a separate
//! process unless it says otherwise (`instant_true_with` and `two_checks_reading` give both faithful reads first).
//!
//! The design gives the cause "a report missing or unreadable" in those words, and says what a
//! launch's Checks become ("context not applied"); it gives no layout for a detail, so a test
//! asserts the words the design gives and the names it says a detail carries.

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
#[path = "support/contexts_launches.rs"]
mod contexts_launches;
use contexts_conditions::*;
use contexts_launches::*;

const HOST: &str = "component api phases built disposing";
const TWO_HOSTS: &str = "component api phases built disposing built disposing";
const MISSING: &str = "a report missing or unreadable";
const NEVER_REACHED: &str = "the context never reached the component";
const BEFORE_AZIMUTH: &str = "source payouts.instant before-azimuth configuration";

/// One launch of `payouts.instant=true` that holds two Checks, so a test can see that a wrong read
/// takes every Check of the launch and not only one.
fn two_checks_reading(directives: &[&str]) -> Repo {
    let repo = launches_with(&[INSTANT_TRUE_LINE], &[], &[INSTANT_TRUE_LINE]);
    applied(&repo);
    give(&repo, directives);
    repo
}

fn assert_owed_with_cause(repo: &Repo, cause: &str) {
    let document = repo.run_json(&[], 1);
    let detail = not_applied(repo, &document, INSTANT_CHECK);
    assert!(detail.contains(cause), "the detail does not give the cause {cause:?}: {detail}");
}

fn assert_owed(repo: &Repo) {
    let document = repo.run_json(&[], 1);
    owed_not_applied(repo, &document, INSTANT_CHECK);
}

/// A launch whose `api` is a host with a faithful pair is accepted. A test of a refusal starts here, so
/// it cannot pass only because the core owes every launch whose reads are not `start` and `stop`.
fn host_is_accepted() {
    assert_accepted(&instant_true_with(&[HOST]));
}

fn assert_accepted(repo: &Repo) {
    proven(repo, &repo.run_json(&[], 0), INSTANT_CHECK);
}

fn launch_entry(document: &serde_json::Value, pairs: &[(&str, &str)]) -> serde_json::Value {
    let wanted = condition_object(pairs);
    let entries = document["suites"].as_array().unwrap_or_else(|| panic!("no `suites`: {document:#}"));
    let matching: Vec<&serde_json::Value> = entries.iter().filter(|entry| entry["condition"] == wanted).collect();
    assert_eq!(matching.len(), 1, "{} entries have the condition {wanted}: {document:#}", matching.len());
    matching[0].clone()
}

fn reads_of(components: &serde_json::Value, name: &str) -> Vec<serde_json::Value> {
    components
        .as_array()
        .and_then(|list| list.iter().find(|component| component["name"] == name))
        .and_then(|component| component["reads"].as_array().cloned())
        .unwrap_or_else(|| panic!("no component {name} with reads: {components:#}"))
}

fn phases_of(reads: &[serde_json::Value]) -> Vec<String> {
    reads.iter().map(|read| read["phase"].as_str().unwrap_or_default().to_string()).collect()
}

fn strs(phases: &[&str]) -> Vec<String> {
    phases.iter().map(|phase| phase.to_string()).collect()
}

// behavior-contexts-in-process: The core accepts reads with the phases `built` and `disposing` as the reads of a host
#[test]
fn the_core_accepts_reads_with_the_phases_built_and_disposing_as_the_reads_of_a_host() {
    let repo = instant_true_with(&[HOST]);
    assert_accepted(&repo);
    let on = launches(&repo).into_iter().find(|launch| launch.requests(INSTANT_CHECK)).unwrap();
    assert!(on.ran.contains(SENDS_TEST), "{on:#?}");
}

// behavior-contexts-in-process: A component whose reads have the phase `built` is read as a host
#[test]
fn a_component_whose_reads_have_the_phase_built_is_read_as_a_host() {
    // Both components are hosts, and neither gave a `start` or a `stop` read.
    let repo = instant_true_with(&[HOST, "component website phases built disposing"]);
    assert_accepted(&repo);
}

// behavior-contexts-in-process: A component whose reads have the phase `disposing` is read as a host
#[test]
fn a_component_whose_reads_have_the_phase_disposing_is_read_as_a_host() {
    // One host beside a separate process in the same launch: each is judged by its own phases.
    let repo = instant_true_with(&[HOST]);
    assert_accepted(&repo);
    let document = repo.run_json(&[], 0);
    let entry = launch_entry(&document, &[("payouts.instant", "true")]);
    assert_eq!(phases_of(&reads_of(&entry["components"], API)), strs(&["built", "disposing"]), "{entry:#}");
    assert_eq!(phases_of(&reads_of(&entry["components"], WEBSITE)), strs(&["start", "stop"]), "{entry:#}");
}

// behavior-contexts-in-process: A component whose reads have only `start` and `stop` is read as a separate process
#[test]
fn a_component_whose_reads_have_only_start_and_stop_is_read_as_a_separate_process() {
    let repo = instant_true_with(&[]);
    assert_accepted(&repo);
    // A separate process is not given a host's reads: it still needs both of its own.
    let without_stop = instant_true_with(&["component api stop skip"]);
    assert_owed(&without_stop);
}

// behavior-contexts-in-process: A host of `api` with one `built` and one `disposing` read is accepted
#[test]
fn a_host_of_api_with_one_built_and_one_disposing_read_is_accepted() {
    let repo = instant_true_with(&[HOST]);
    assert_accepted(&repo);
    let document = repo.run_json(&[], 0);
    let entry = launch_entry(&document, &[("payouts.instant", "true")]);
    assert_eq!(phases_of(&reads_of(&entry["components"], API)), strs(&["built", "disposing"]), "{entry:#}");
}

// behavior-contexts-in-process: A host with a `built` read and no `disposing` read leaves the Checks of its launch owed
#[test]
fn a_host_with_a_built_read_and_no_disposing_read_leaves_the_checks_of_its_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&["component api phases built"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: A host with a `disposing` read and no `built` read leaves the Checks of its launch owed
#[test]
fn a_host_with_a_disposing_read_and_no_built_read_leaves_the_checks_of_its_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&["component api phases disposing"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: Two `built` reads of `api` and one `disposing` read leave the Checks of the launch owed
#[test]
fn two_built_reads_of_api_and_one_disposing_read_leave_the_checks_of_the_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&["component api phases built built disposing"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: Two `built` reads and two `disposing` reads of `api` are accepted
#[test]
fn two_built_reads_and_two_disposing_reads_of_api_are_accepted() {
    assert_accepted(&instant_true_with(&[TWO_HOSTS]));
    // The order of the reads is not stated, so a count that holds in any order is accepted too.
    assert_accepted(&instant_true_with(&["component api phases built built disposing disposing"]));
}

// behavior-contexts-in-process: A separate process with a `start` read and no `stop` read leaves the Checks of its launch owed
#[test]
fn a_separate_process_with_a_start_read_and_no_stop_read_leaves_the_checks_of_its_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&["component api stop skip"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: A separate process with a `stop` read and no `start` read leaves the Checks of its launch owed
#[test]
fn a_separate_process_with_a_stop_read_and_no_start_read_leaves_the_checks_of_its_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&["component api start skip"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: More `disposing` reads of `api` than `built` reads leave the Checks of the launch owed
#[test]
fn more_disposing_reads_of_api_than_built_reads_leave_the_checks_of_the_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&["component api phases built disposing disposing"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: A component with a `start` read and a `built` read in one launch is a report the core cannot read
#[test]
fn a_component_with_a_start_read_and_a_built_read_in_one_launch_is_a_report_the_core_cannot_read() {
    host_is_accepted();
    assert_owed(&instant_true_with(&["component api phases start built"]));
    // Reads that are complete in each kind do not make the mix readable.
    assert_owed(&instant_true_with(&["component api phases start stop built disposing"]));
}

// behavior-contexts-in-process: A component with a `stop` read and a `disposing` read in one launch is a report the core cannot read
#[test]
fn a_component_with_a_stop_read_and_a_disposing_read_in_one_launch_is_a_report_the_core_cannot_read() {
    host_is_accepted();
    assert_owed(&instant_true_with(&["component api phases stop disposing"]));
    assert_owed(&instant_true_with(&["component api phases built disposing start stop"]));
}

// behavior-contexts-in-process: A component with reads of both kinds leaves the Checks of its launch owed
#[test]
fn a_component_with_reads_of_both_kinds_leaves_the_checks_of_its_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&["component api phases start stop built disposing"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: The `built` read of a host must show the stated value applied by Azimuth
#[test]
fn the_built_read_of_a_host_must_show_the_stated_value_applied_by_azimuth() {
    assert_accepted(&instant_true_with(&[HOST]));
    let wrong_value = instant_true_with(&[HOST, "component api built value payouts.instant false"]);
    assert_owed(&wrong_value);
    let not_by_azimuth = instant_true_with(&[HOST, &format!("component api built {BEFORE_AZIMUTH}")]);
    assert_owed(&not_by_azimuth);
}

// behavior-contexts-in-process: The `disposing` read of a host must show the stated value applied by Azimuth
#[test]
fn the_disposing_read_of_a_host_must_show_the_stated_value_applied_by_azimuth() {
    host_is_accepted();
    let wrong_value = instant_true_with(&[HOST, "component api disposing value payouts.instant false"]);
    assert_owed(&wrong_value);
    let not_by_azimuth = instant_true_with(&[HOST, &format!("component api disposing {BEFORE_AZIMUTH}")]);
    assert_owed(&not_by_azimuth);
}

// behavior-contexts-in-process: A launch of `payouts.instant=true` with an `api` host whose `built` and `disposing` reads both show `true` applied by Azimuth runs the Check `payout-is-sent-at-once`
#[test]
fn a_launch_with_an_api_host_whose_reads_both_show_true_runs_the_check() {
    let repo = instant_true_with(&[HOST]);
    let document = repo.run_json(&[], 0);
    proven(&repo, &document, INSTANT_CHECK);
    let all = launches(&repo);
    let launch = launch_of(&all, &[("payouts.instant", "true")]);
    assert!(launch.requests(INSTANT_CHECK), "{launch:#?}");
    assert!(launch.ran.contains(SENDS_TEST), "the test of the Check did not run: {launch:#?}");
}

// behavior-contexts-in-process: A launch where only one host of two reports a wrong value leaves every Check of that launch owed
#[test]
fn a_launch_where_only_one_host_of_two_reports_a_wrong_value_leaves_every_check_of_that_launch_owed() {
    // The first pair is right and the second host's `built` read is wrong, then the same with `disposing`.
    for wrong in ["component api read 3 value payouts.instant false", "component api read 4 value payouts.instant false"] {
        let repo = two_checks_reading(&[TWO_HOSTS, wrong]);
        let document = repo.run_json(&[], 1);
        for check in [INSTANT_CHECK, REGION_CHECK] {
            owed_not_applied(&repo, &document, check);
        }
    }
    let control = two_checks_reading(&[TWO_HOSTS]);
    let document = control.run_json(&[], 0);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        proven(&control, &document, check);
    }
}

// behavior-contexts-in-process: A launch where the `built` read of a host is wrong leaves every Check of that launch owed
#[test]
fn a_launch_where_the_built_read_of_a_host_is_wrong_leaves_every_check_of_that_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&[HOST, "component api built value payouts.instant false"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: A launch where the `disposing` read of a host is wrong leaves every Check of that launch owed
#[test]
fn a_launch_where_the_disposing_read_of_a_host_is_wrong_leaves_every_check_of_that_launch_owed() {
    host_is_accepted();
    let repo = two_checks_reading(&[HOST, "component api disposing value payouts.instant false"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-in-process: The state of those Checks is `owed` and the reason is `context not applied`
#[test]
fn the_state_of_those_checks_is_owed_and_the_reason_is_context_not_applied() {
    host_is_accepted();
    let repo = two_checks_reading(&[HOST, "component api built value payouts.instant false"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        let entry = entry_of(&document, check);
        assert_eq!(entry["state"], "owed", "{check}: {document:#}");
        assert_eq!(entry["reason"], "context not applied", "{check}: {document:#}");
    }
}

// behavior-contexts-in-process: The detail of such a result names the component, key, stated and reported values, and cause
#[test]
fn the_detail_of_such_a_result_names_the_component_key_stated_and_reported_values_and_cause() {
    host_is_accepted();
    let repo = instant_true_with(&[HOST, &format!("component api built {BEFORE_AZIMUTH}"), "component api built value payouts.instant false"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains("api, "), "the component is not named first: {detail}");
    assert!(detail.contains("payouts.instant stated true, reported false"), "{detail}");
    assert!(detail.contains(NEVER_REACHED), "{detail}");
}

// behavior-contexts-in-process: A read of a host with an `error` instead of a report gives the cause of a report missing or unreadable, naming its declared place
#[test]
fn a_read_of_a_host_with_an_error_instead_of_a_report_gives_the_cause_naming_its_declared_place() {
    host_is_accepted();
    for directive in ["component api built error connection refused", "component api disposing error connection refused"] {
        let repo = instant_true_with(&[HOST, directive]);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(detail.contains(MISSING), "{directive}: {detail}");
        assert!(detail.contains(API) && detail.contains(API_PLACE), "{directive}: {detail}");
    }
}

// behavior-contexts-in-process: A host with a `built` read and no `disposing` read gives the cause of a report missing or unreadable
#[test]
fn a_host_with_a_built_read_and_no_disposing_read_gives_the_cause_of_a_report_missing_or_unreadable() {
    host_is_accepted();
    assert_owed_with_cause(&instant_true_with(&["component api phases built"]), MISSING);
}

// behavior-contexts-in-process: A separate process with a `start` read and no `stop` read gives the cause of a report missing or unreadable
#[test]
fn a_separate_process_with_a_start_read_and_no_stop_read_gives_the_cause_of_a_report_missing_or_unreadable() {
    host_is_accepted();
    assert_owed_with_cause(&instant_true_with(&["component api stop skip"]), MISSING);
}

// behavior-contexts-in-process: A separate process with a `stop` read and no `start` read gives the cause of a report missing or unreadable
#[test]
fn a_separate_process_with_a_stop_read_and_no_start_read_gives_the_cause_of_a_report_missing_or_unreadable() {
    host_is_accepted();
    assert_owed_with_cause(&instant_true_with(&["component api start skip"]), MISSING);
}

// behavior-contexts-in-process: An unequal count of `built` and `disposing` reads gives the cause of a report missing or unreadable
#[test]
fn an_unequal_count_of_built_and_disposing_reads_gives_the_cause_of_a_report_missing_or_unreadable() {
    host_is_accepted();
    assert_owed_with_cause(&instant_true_with(&["component api phases built built disposing"]), MISSING);
}

// behavior-contexts-in-process: More `disposing` reads than `built` reads give the cause of a report missing or unreadable
#[test]
fn more_disposing_reads_than_built_reads_give_the_cause_of_a_report_missing_or_unreadable() {
    host_is_accepted();
    assert_owed_with_cause(&instant_true_with(&["component api phases built disposing disposing"]), MISSING);
}

// behavior-contexts-in-process: A component with reads of both kinds gives the cause of a report missing or unreadable
#[test]
fn a_component_with_reads_of_both_kinds_gives_the_cause_of_a_report_missing_or_unreadable() {
    host_is_accepted();
    assert_owed_with_cause(&instant_true_with(&["component api phases start stop built disposing"]), MISSING);
}

// behavior-contexts-in-process: A launch with no condition needs no reads of a host
#[test]
fn a_launch_with_no_condition_needs_no_reads_of_a_host() {
    // A launch that states no condition has Checks whatever its components read, even a host that
    // gave a `built` read and no `disposing` read, or no read at all.
    for directives in [
        vec!["component api phases built", "component website phases built built disposing"],
        vec!["component api phases start stop built disposing", "component website none"],
        vec!["component api phases built disposing disposing", "component website both skip"],
    ] {
        let repo = launches_with(&[], &[], &[]);
        applied(&repo);
        give(&repo, &directives);
        let document = repo.run_json(&[], 0);
        for check in all_launch_checks() {
            proven(&repo, &document, check);
        }
    }
}

// behavior-contexts-in-process: A malformed read of a host is a component read error, not a new outcome of a test
#[test]
fn a_malformed_read_of_a_host_is_a_component_read_error_and_not_an_outcome_of_a_test() {
    host_is_accepted();
    for directive in [
        "component api built malformed",
        "component api disposing malformed",
        "component api built incomplete",
        "component api disposing incomplete",
    ] {
        let repo = instant_true_with(&[HOST, directive]);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(!detail.trim().is_empty(), "{directive}: the detail names no cause");
        for check in all_launch_checks() {
            assert_ne!(state_of(&document, check), "failed", "{directive}: {check}: {document:#}");
        }
        assert!(launches(&repo).iter().any(|launch| launch.ran.contains(SENDS_TEST)), "{directive}: the test did not run");
    }
}

// behavior-contexts-in-process: The collected reads of a host appear in `run-1` and in each Check result of the launch
#[test]
fn the_collected_reads_of_a_host_appear_in_run_and_in_each_check_result_of_the_launch() {
    let repo = two_checks_reading(&[TWO_HOSTS]);
    let document = repo.run_json(&[], 0);
    let entry = launch_entry(&document, &[("payouts.instant", "true")]);
    let in_run = reads_of(&entry["components"], API);
    assert_eq!(phases_of(&in_run), strs(&["built", "disposing", "built", "disposing"]), "{entry:#}");
    for check in [INSTANT_CHECK, REGION_CHECK] {
        let body = repo.result_of(check)["body"].clone();
        assert_eq!(reads_of(&body["components"], API), in_run, "{check}: {body:#}");
        assert_eq!(reads_of(&body["components"], WEBSITE), reads_of(&entry["components"], WEBSITE), "{check}");
    }
    // A launch that gave no result still shows the reads of its host in `run-1`.
    let wrong = instant_true_with(&[HOST, "component api built value payouts.instant false"]);
    let document = wrong.run_json(&[], 1);
    let entry = launch_entry(&document, &[("payouts.instant", "true")]);
    let reads = reads_of(&entry["components"], API);
    assert_eq!(phases_of(&reads), strs(&["built", "disposing"]), "{entry:#}");
    assert_eq!(reads[0]["report"]["context"]["payouts.instant"]["value"], "false", "{entry:#}");
}
