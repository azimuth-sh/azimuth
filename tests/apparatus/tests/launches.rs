//! Launches on the sample: `docs/increments/contexts-launches.md`. The design gives the words of
//! some causes and not of others; a test asserts the words where the design gives them and the
//! names (component, key, declared place, source) where it only says what a cause names.
//!
//! The order of launches of one suite is not stated, so no test asserts which runs first. A test
//! about "the next launch still runs" lets every launch of the suite fail in the same way and
//! asserts that every launch started, which holds whatever the order.

use std::collections::BTreeSet;

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
#[path = "support/contexts_launches.rs"]
mod contexts_launches;
use contexts_conditions::*;
use contexts_launches::*;

const WRONG_START: &str = "component api both source payouts.instant before-azimuth configuration";
const NEVER_REACHED: &str = "the context never reached the component";

fn instant_true() -> Vec<(&'static str, &'static str)> {
    vec![("payouts.instant", "true")]
}

fn instant_false() -> Vec<(&'static str, &'static str)> {
    vec![("payouts.instant", "false")]
}

fn set_outcome(repo: &Repo, outcome: &str) {
    repo.replace(PAYOUTS_TESTS, "# Check: payout-is-sent-at-once\noutcome: passed", &format!("# Check: payout-is-sent-at-once\noutcome: {outcome}"));
}

fn status_reason(repo: &Repo, check: &str) -> String {
    let status = repo.status_json();
    status_checks(&status)[check]["reason"].as_str().unwrap_or_default().to_string()
}

fn every_test_of_the_sample() -> Vec<&'static str> {
    vec![
        SENDS_TEST,
        HOLDS_TEST,
        REGION_TEST,
        "loan-limit.refuses-sixth-loan",
        "loan-limit.lends-after-a-return",
        "late-fine",
        "free-return.returned-on-due-date",
        "free-return.returned-early",
    ]
}

fn payout_and_lending_checks() -> Vec<&'static str> {
    PAYOUT_SCENARIOS.iter().copied().chain(ALL_CHECKS).collect()
}

// behavior-contexts-launches: Requested Checks of a suite with different effective conditions get different launches
#[test]
fn checks_with_different_conditions_get_different_launches() {
    let repo = two_conditions();
    applied(&repo);
    let document = repo.run_json(&[], 0);
    let all = launches(&repo);
    let on = launch_of(&all, &instant_true());
    let off = launch_of(&all, &instant_false());
    assert!(on.requests(INSTANT_CHECK) && !on.requests(REVIEW_CHECK), "{all:#?}");
    assert!(off.requests(REVIEW_CHECK) && !off.requests(INSTANT_CHECK), "{all:#?}");
    proven(&repo, &document, INSTANT_CHECK);
    proven(&repo, &document, REVIEW_CHECK);
}

// behavior-contexts-launches: Requested Checks of a suite with the same effective condition share one launch
#[test]
fn checks_with_the_same_condition_share_one_launch() {
    let repo = launches_with(&[INSTANT_TRUE_LINE], &[INSTANT_TRUE_LINE], &[]);
    applied(&repo);
    let document = repo.run_json(&[], 0);
    let all = launches(&repo);
    let shared = launch_of(&all, &instant_true());
    assert!(shared.requests(INSTANT_CHECK) && shared.requests(REVIEW_CHECK), "{all:#?}");
    proven(&repo, &document, INSTANT_CHECK);
    proven(&repo, &document, REVIEW_CHECK);
}

// behavior-contexts-launches: Only the keys a condition states decide the group
#[test]
fn only_the_stated_keys_decide_the_group() {
    // The second Scenario states the same key and value in another spelling, and the third adds a key.
    let repo = launches_with(&[INSTANT_TRUE_LINE], &["Context: payouts.instant = TRUE"], &[INSTANT_TRUE_LINE, REGION_SE_LINE]);
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    let plain = launch_of(&all, &instant_true());
    assert!(plain.requests(INSTANT_CHECK) && plain.requests(REVIEW_CHECK) && !plain.requests(REGION_CHECK), "{all:#?}");
    let both = launch_of(&all, &[("payouts.instant", "true"), ("payouts.region", "se")]);
    assert_eq!(both.checks(), ids(&[REGION_CHECK]), "{all:#?}");
    for launch in &all {
        let keys: Vec<&String> = launch.condition().as_object().unwrap().keys().collect();
        assert!(keys.iter().all(|key| *key == "payouts.instant" || *key == "payouts.region"), "an unstated key is in {launch:#?}");
    }
}

// behavior-contexts-launches: Checks of a suite with no condition form a group of their own, with its own launch
#[test]
fn checks_with_no_condition_form_a_group_with_its_own_launch() {
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    let free = launch_of(&all, &[]);
    assert_eq!(free.checks(), lending_checks(), "{all:#?}");
    for launch in all.iter().filter(|launch| *launch.condition() != json!({})) {
        assert!(launch.checks().is_disjoint(&lending_checks()), "a Check with no condition joined {launch:#?}");
    }
}

// behavior-contexts-launches: Launches of one suite run one after another
#[test]
fn launches_of_one_suite_run_one_after_another() {
    let repo = four_launches();
    applied(&repo);
    give(&repo, &["sleeps 300"]);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(all.len(), 4, "{all:#?}");
    for pair in all.windows(2) {
        let ended = pair[0].ended.expect("a launch that wrote its report ended before the next began");
        assert!(ended <= pair[1].started, "two launches overlapped: {pair:#?}");
    }
}

// behavior-contexts-launches: Each launch of a suite has its own facility proof
#[test]
fn each_launch_of_a_suite_has_its_own_facility_proof() {
    let repo = four_launches();
    repo.need_facility("present");
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(all.len(), 4, "{all:#?}");
    assert_eq!(repo.probe_starts().len(), all.len(), "{:#?}", repo.probe_starts());
}

// behavior-contexts-launches: A facility proof runs before each launch
#[test]
fn a_facility_proof_runs_before_each_launch() {
    let repo = four_launches();
    repo.need_facility("present");
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    let proofs: Vec<u64> = repo.probe_starts().iter().map(|probe| probe["time_ns"].as_u64().unwrap()).collect();
    for (index, launch) in all.iter().enumerate() {
        let after = if index == 0 { 0 } else { all[index - 1].ended.unwrap() };
        assert!(
            proofs.iter().any(|time| *time > after && *time < launch.started),
            "no proof stands between the end of the launch before and the start of launch {index}: proofs {proofs:?}, launches {all:#?}"
        );
    }
}

// behavior-contexts-launches: Each launch of a suite has its own time limit
#[test]
fn each_launch_of_a_suite_has_its_own_time_limit() {
    let repo = two_conditions();
    repo.replace(PROJECT_FILE, "needs = []", "needs = []\ntime_limit_minutes = 1");
    applied(&repo);
    // Two launches of 35 s each take longer than the limit of one minute together, and neither is stopped.
    give(&repo, &["when payouts.instant=true sleeps 35000", "when payouts.instant=false sleeps 35000"]);
    let document = repo.run_json(&[], 0);
    for check in payout_and_lending_checks() {
        proven(&repo, &document, check);
    }
    let all = launches(&repo);
    let slow: u64 = all.iter().filter(|launch| *launch.condition() != json!({})).map(|launch| launch.ended.unwrap() - launch.started).sum();
    assert!(slow > 60_000_000_000, "the launches did not together outlast the limit: {all:#?}");
}

// behavior-contexts-launches: A launch that timed out gives no result for its Checks, and the next launch of the suite still runs
#[test]
fn a_launch_that_timed_out_gives_no_result_and_the_next_launch_still_runs() {
    let repo = launches_with(&[INSTANT_TRUE_LINE], &[INSTANT_FALSE_LINE], &[INSTANT_TRUE_LINE]);
    repo.replace(PROJECT_FILE, "needs = []", "needs = []\ntime_limit_minutes = 1");
    applied(&repo);
    give(&repo, &["when payouts.instant=true sleeps 70000"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        no_result(&repo, &document, check);
    }
    for check in [REVIEW_CHECK].into_iter().chain(ALL_CHECKS) {
        proven(&repo, &document, check);
    }
    let all = launches(&repo);
    assert_eq!(all.len(), 3, "{all:#?}");
    assert!(launch_of(&all, &instant_true()).ended.is_none(), "the launch that timed out ended by itself: {all:#?}");
    assert!(!launch_of(&all, &instant_false()).ran.is_empty(), "the other launch ran no test: {all:#?}");
}

// behavior-contexts-launches: A test marked for Checks of different groups runs in each group's launch
#[test]
fn a_test_marked_for_checks_of_different_groups_runs_in_each_launch() {
    let repo = two_conditions();
    journey_for(&repo, &[INSTANT_CHECK, REVIEW_CHECK]);
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    assert!(launch_of(&all, &instant_true()).ran.contains(JOURNEY_TEST), "{all:#?}");
    assert!(launch_of(&all, &instant_false()).ran.contains(JOURNEY_TEST), "{all:#?}");
}

// behavior-contexts-launches: The outcome of such a test counts only for the Check ids requested in that launch
#[test]
fn the_outcome_of_such_a_test_counts_only_for_the_checks_requested_in_that_launch() {
    let repo = two_conditions();
    journey_for(&repo, &[INSTANT_CHECK, REVIEW_CHECK]);
    applied(&repo);
    give(&repo, &[&format!("fails {JOURNEY_TEST}")]);
    // Only the Check of one group is requested: the failure of the journey is its alone.
    let output = repo.azimuth(&["check", "run", REVIEW_CHECK, "--json"]);
    output.expect_status(1);
    let document = output.json();
    assert_eq!(state_of(&document, REVIEW_CHECK), "failed", "{document:#}");
    assert_eq!(repo.results_of(REVIEW_CHECK).len(), 1);
    assert!(repo.results_of(INSTANT_CHECK).is_empty(), "the other Check of the journey got a result: {:#?}", repo.results_of(INSTANT_CHECK));
    let all = launches(&repo);
    assert_eq!(all.len(), 1, "{all:#?}");
    assert_eq!(all[0].checks(), ids(&[REVIEW_CHECK]));
}

// behavior-contexts-launches: A journey marked for `payout-is-sent-at-once` and `payout-waits-for-review` runs twice and counts for one Check each time
#[test]
fn a_journey_marked_for_two_checks_runs_twice_and_counts_for_one_check_each_time() {
    let repo = two_conditions();
    journey_for(&repo, &[INSTANT_CHECK, REVIEW_CHECK]);
    applied(&repo);
    let document = repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(all.iter().filter(|launch| launch.ran.contains(JOURNEY_TEST)).count(), 2, "{all:#?}");
    for check in [INSTANT_CHECK, REVIEW_CHECK] {
        proven(&repo, &document, check);
        let tests = repo.check_show(check)["tests"].clone();
        assert!(tests.as_array().unwrap().iter().any(|test| test["name"] == JOURNEY_TEST), "{check}: {tests:#}");
    }
}

// behavior-contexts-launches: A failure of the test in one launch does not fail the Check of the other launch
#[test]
fn a_failure_of_the_test_in_one_launch_does_not_fail_the_check_of_the_other_launch() {
    let repo = two_conditions();
    journey_for(&repo, &[INSTANT_CHECK, REVIEW_CHECK]);
    applied(&repo);
    give(&repo, &[&format!("when payouts.instant=true fails {JOURNEY_TEST}")]);
    let document = repo.run_json(&[], 1);
    assert_eq!(state_of(&document, INSTANT_CHECK), "failed", "{document:#}");
    proven(&repo, &document, REVIEW_CHECK);
    assert_eq!(repo.result_of(REVIEW_CHECK)["body"]["outcome"], "passed");
}

// behavior-contexts-launches: A launch with a condition gets `AZIMUTH_CONTEXT` as one JSON object of product keys to normalized text values
#[test]
fn a_launch_with_a_condition_gets_azimuth_context_as_one_json_object_of_normalized_text() {
    let repo = launches_with(&["Context: payouts.instant=TRUE", "Context: payouts.limit=1e3", REGION_SE_LINE], &[], &[]);
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    let launch = all.iter().find(|launch| launch.requests(INSTANT_CHECK)).unwrap();
    let text = launch.context.clone().expect("the adapter got AZIMUTH_CONTEXT");
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_else(|error| panic!("{text:?} is not one JSON document: {error}"));
    assert_eq!(parsed, json!({"payouts.instant": "true", "payouts.limit": "1000", "payouts.region": "se"}), "{text}");
}

// behavior-contexts-launches: A launch of `payouts.instant=true` gets `AZIMUTH_CONTEXT={"payouts.instant":"true"}`
#[test]
fn a_launch_of_instant_true_gets_the_context_the_design_shows() {
    let repo = two_conditions();
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(launch_of(&all, &instant_true()).context.as_deref(), Some("{\"payouts.instant\":\"true\"}"), "{all:#?}");
    assert_eq!(launch_of(&all, &instant_false()).context.as_deref(), Some("{\"payouts.instant\":\"false\"}"), "{all:#?}");
}

// behavior-contexts-launches: A launch with no condition gets `{}`
#[test]
fn a_launch_with_no_condition_gets_an_empty_object() {
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(launch_of(&all, &[]).context.as_deref(), Some("{}"), "{all:#?}");
}

// behavior-contexts-launches: A value of `AZIMUTH_CONTEXT` inherited from the environment of the core is replaced by the launch's condition
#[test]
fn an_inherited_azimuth_context_is_replaced_by_the_launchs_condition() {
    let repo = two_conditions();
    applied(&repo);
    let inherited = "{\"payouts.instant\":\"false\",\"payouts.region\":\"no\"}";
    let output = repo.azimuth_with_env(&["run", "--json"], &[("AZIMUTH_CONTEXT", inherited)]);
    output.expect_status(0);
    let all = launches(&repo);
    assert_eq!(launch_of(&all, &instant_true()).context.as_deref(), Some("{\"payouts.instant\":\"true\"}"), "{all:#?}");
    assert_eq!(launch_of(&all, &instant_false()).context.as_deref(), Some("{\"payouts.instant\":\"false\"}"), "{all:#?}");
}

// behavior-contexts-launches: An inherited `AZIMUTH_CONTEXT` is replaced by `{}` in a launch with no condition
#[test]
fn an_inherited_azimuth_context_is_replaced_by_an_empty_object_in_a_launch_with_no_condition() {
    let repo = two_conditions();
    applied(&repo);
    let output = repo.azimuth_with_env(&["run", "--json"], &[("AZIMUTH_CONTEXT", "{\"payouts.instant\":\"true\"}")]);
    output.expect_status(0);
    let all = launches(&repo);
    assert_eq!(launch_of(&all, &[]).context.as_deref(), Some("{}"), "{all:#?}");
}

// behavior-contexts-launches: A launch with no condition expects no component report
#[test]
fn a_launch_with_no_condition_expects_no_component_report() {
    let repo = launches_with(&[], &[], &[]);
    give(
        &repo,
        &[
            "component api both applying not-possible",
            "component website both error nothing answers there",
            "component api both name billing",
        ],
    );
    let document = repo.run_json(&[], 0);
    for check in payout_and_lending_checks() {
        proven(&repo, &document, check);
    }
    // The same reports under a condition lose the Checks, so the reports were read when they mattered.
    let stated = launches_with(&[INSTANT_TRUE_LINE], &[], &[]);
    give(&stated, &["components applied", "component api both applying not-possible"]);
    let document = stated.run_json(&[], 1);
    owed_not_applied(&stated, &document, INSTANT_CHECK);
}

// behavior-contexts-launches: The adapter passes the variable to the runner
#[test]
fn the_adapter_passes_the_variable_to_the_runner() {
    let repo = two_conditions();
    applied(&repo);
    give(&repo, &["runner"]);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    let mut runner: Vec<String> = repo
        .log_entries()
        .iter()
        .filter(|entry| entry["program"] == "text-suite-runner")
        .map(|entry| entry["azimuth_context"].as_str().unwrap_or("<unset>").to_string())
        .collect();
    let mut expected: Vec<String> = all.iter().map(|launch| launch.context.clone().unwrap()).collect();
    runner.sort();
    expected.sort();
    assert_eq!(runner, expected, "{all:#?}");
    assert!(expected.contains(&"{\"payouts.instant\":\"true\"}".to_string()), "{expected:?}");
}

// behavior-contexts-launches: The core sets no key that the condition does not state
#[test]
fn the_core_sets_no_key_that_the_condition_does_not_state() {
    // The keys file declares four keys. A launch states some, and its variable holds exactly those.
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(all.len(), 4, "{all:#?}");
    for launch in &all {
        let parsed: serde_json::Value = serde_json::from_str(launch.context.as_deref().unwrap()).unwrap();
        assert_eq!(&parsed, launch.condition(), "{launch:#?}");
    }
    let only_instant = launch_of(&all, &instant_true());
    assert_eq!(only_instant.context.as_deref(), Some("{\"payouts.instant\":\"true\"}"));
    assert_eq!(launch_of(&all, &[]).context.as_deref(), Some("{}"));
}

// behavior-contexts-launches: A facility proof gets the environment unchanged
#[test]
fn a_facility_proof_gets_the_environment_unchanged() {
    let repo = two_conditions();
    repo.need_facility("present");
    applied(&repo);
    let inherited = "{\"payouts.instant\":\"false\",\"payouts.region\":\"no\"}";
    repo.azimuth_with_env(&["run", "--json"], &[("AZIMUTH_CONTEXT", inherited)]).expect_status(0);
    let proofs = repo.probe_starts();
    assert_eq!(proofs.len(), launches(&repo).len(), "{proofs:#?}");
    for proof in proofs {
        assert_eq!(proof["azimuth_context"], inherited, "{proof:#}");
    }
}

// behavior-contexts-launches: The request keeps the number `adapter-request-1`
#[test]
fn the_request_keeps_the_number_adapter_request_1() {
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    for launch in launches(&repo) {
        assert_eq!(launch.request["format"], "adapter-request-1", "{launch:#?}");
    }
}

// behavior-contexts-launches: The run request carries `condition`, the same object as `AZIMUTH_CONTEXT`
#[test]
fn the_run_request_carries_the_condition_that_azimuth_context_holds() {
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    for launch in &all {
        let variable: serde_json::Value = serde_json::from_str(launch.context.as_deref().unwrap()).unwrap();
        assert_eq!(&variable, launch.condition(), "{launch:#?}");
    }
    assert_eq!(*launch_of(&all, &instant_true()).condition(), json!({"payouts.instant": "true"}));
}

// behavior-contexts-launches: The request of a launch with no condition carries `{}` as its condition
#[test]
fn the_request_of_a_launch_with_no_condition_carries_an_empty_condition() {
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    let free = launch_of(&all, &[]);
    assert!(free.request.get("condition").is_some_and(|condition| condition.is_object()), "{free:#?}");
}

// behavior-contexts-launches: The run request carries `components`, one entry with `name` and `report` path for each component in the suite's `starts`
#[test]
fn the_run_request_lists_each_started_component_with_its_report_path() {
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    for launch in launches(&repo) {
        let components = launch.request["components"].as_array().unwrap_or_else(|| panic!("no components in {launch:#?}"));
        let found: std::collections::BTreeMap<String, String> = components
            .iter()
            .map(|entry| (entry["name"].as_str().unwrap().to_string(), entry["report"].as_str().unwrap().to_string()))
            .collect();
        assert_eq!(components.len(), 2, "{launch:#?}");
        assert_eq!(found[API], API_PLACE, "{launch:#?}");
        assert_eq!(found[WEBSITE], WEBSITE_PLACE, "{launch:#?}");
    }
}

// behavior-contexts-launches: The request lists the components of `starts` even when the condition needs none of their keys
#[test]
fn the_request_lists_components_even_when_the_condition_needs_none_of_their_keys() {
    // `payouts.region` is read by `api` alone.
    let repo = launches_with(&[REGION_SE_LINE], &[], &[]);
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    let launch = all.iter().find(|launch| launch.requests(INSTANT_CHECK)).unwrap();
    let names = strings_of(&json!(launch.request["components"].as_array().unwrap().iter().map(|entry| entry["name"].clone()).collect::<Vec<_>>()));
    assert!(names.contains(&WEBSITE.to_string()), "website is not listed: {launch:#?}");
    let free = launch_of(&all, &[]);
    assert_eq!(free.request["components"].as_array().unwrap().len(), 2, "{free:#?}");
}

// behavior-contexts-launches: An adapter that predates the new fields still proves a launch with `{}` as its condition
#[test]
fn an_adapter_that_predates_the_fields_still_proves_a_launch_with_no_condition() {
    // The test adapter writes no `components` unless told to, and ignores the new request fields.
    let repo = launches_with(&[], &[], &[]);
    let document = repo.run_json(&[], 0);
    for check in payout_and_lending_checks() {
        proven(&repo, &document, check);
    }
    let all = launches(&repo);
    assert_eq!(all.len(), 1, "{all:#?}");
    assert_eq!(*all[0].condition(), json!({}));
}

// behavior-contexts-launches: The report keeps the number `adapter-report-1`
#[test]
fn the_report_keeps_the_number_adapter_report_1() {
    // The adapter writes `components` in a report of the same number, and the core accepts it.
    let repo = two_conditions();
    applied(&repo);
    let document = repo.run_json(&[], 0);
    proven(&repo, &document, INSTANT_CHECK);
    proven(&repo, &document, REVIEW_CHECK);
}

// behavior-contexts-launches: The report gains `components`, a list with the expected `name` and `reads` for each component collected
#[test]
fn the_report_gains_components_with_the_expected_name_and_reads() {
    let repo = instant_true_with(&[]);
    let document = repo.run_json(&[], 0);
    proven(&repo, &document, INSTANT_CHECK);
    // The core reads that list: leaving an expected component out of it loses the Check.
    let without = instant_true_with(&["component api none"]);
    let document = without.run_json(&[], 1);
    owed_not_applied(&without, &document, INSTANT_CHECK);
}

// behavior-contexts-launches: Each read has `phase` and either `report`, the document as read, or `error`, the cause it could not be read
#[test]
fn each_read_has_a_phase_and_either_a_report_or_an_error() {
    let repo = instant_true_with(&[]);
    proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
    let error = "connection refused on the report path";
    let failed = instant_true_with(&[&format!("component api both error {error}")]);
    let document = failed.run_json(&[], 1);
    let detail = not_applied(&failed, &document, INSTANT_CHECK);
    assert!(detail.contains(API) && detail.contains(API_PLACE), "{detail}");
    assert!(detail.contains(error), "the cause of the read is left out: {detail}");
}

// behavior-contexts-launches: The phases `start` and `stop` mark the two reads of a component in its own process
#[test]
fn the_phases_start_and_stop_mark_the_two_reads_of_a_component_in_its_own_process() {
    let both = instant_true_with(&[]);
    proven(&both, &both.run_json(&[], 0), INSTANT_CHECK);
    for missing in ["component api start skip", "component api stop skip"] {
        let repo = instant_true_with(&[missing]);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(detail.contains(API), "{missing}: {detail}");
    }
}

// behavior-contexts-launches: A component that answered before start has `error: "something else answers there"` and no reads
#[test]
fn a_component_that_answered_before_start_has_an_error_and_no_reads() {
    let control = instant_true_with(&[]);
    proven(&control, &control.run_json(&[], 0), INSTANT_CHECK);
    let repo = instant_true_with(&["component api answered"]);
    let document = repo.run_json(&[], 1);
    owed_not_applied(&repo, &document, INSTANT_CHECK);
    // The adapter reports the tests as not run in a launch with no condition too, so no Check of it passes.
    let free = launches_with(&[], &[], &[]);
    give(&free, &["components applied", "component api answered"]);
    let document = free.run_json(&[], 1);
    for check in payout_and_lending_checks() {
        assert_eq!(state_of(&document, check), "owed", "{check}: {document:#}");
        assert!(free.results_of(check).is_empty(), "{check} has a result");
    }
}

// behavior-contexts-launches: Only components needed by a nonempty condition are required to have reads
#[test]
fn only_components_needed_by_a_nonempty_condition_are_required_to_have_reads() {
    // `payouts.region` is read by `api` alone, so `website` needs no reads.
    let repo = launches_with(&[REGION_SE_LINE], &[], &[]);
    applied(&repo);
    give(&repo, &["component website none"]);
    proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
    let empty = launches_with(&[REGION_SE_LINE], &[], &[]);
    applied(&empty);
    give(&empty, &["component website both skip"]);
    proven(&empty, &empty.run_json(&[], 0), INSTANT_CHECK);
    let needed = launches_with(&[REGION_SE_LINE], &[], &[]);
    applied(&needed);
    give(&needed, &["component api none"]);
    let document = needed.run_json(&[], 1);
    owed_not_applied(&needed, &document, INSTANT_CHECK);
}

// behavior-contexts-launches: A malformed component report is a component read error, not a new outcome of a test
#[test]
fn a_malformed_component_report_is_a_read_error_and_not_an_outcome_of_a_test() {
    for malformed in ["malformed", "incomplete"] {
        let repo = instant_true_with(&[&format!("component api both {malformed}")]);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(!detail.trim().is_empty(), "{malformed}: the detail names no cause");
        for check in payout_and_lending_checks() {
            assert_ne!(state_of(&document, check), "failed", "{malformed}: {check}: {document:#}");
        }
        assert!(launches(&repo).iter().any(|launch| launch.ran.contains(SENDS_TEST)), "{malformed}: the test did not run");
    }
}

// behavior-contexts-launches: A report with no `components` leaves Checks with a condition owed with "context not applied: the adapter reports no components"
#[test]
fn a_report_with_no_components_leaves_checks_with_a_condition_owed() {
    // The test adapter writes no `components` unless told to.
    let repo = two_conditions();
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REVIEW_CHECK] {
        not_applied(&repo, &document, check);
        let shown = shown_reason(&entry_of(&document, check));
        assert!(shown.contains("context not applied: the adapter reports no components"), "{check}: {shown}");
    }
}

// behavior-contexts-launches: Checks with no condition keep their results when the report has no `components`
#[test]
fn checks_with_no_condition_keep_their_results_when_the_report_has_no_components() {
    let repo = two_conditions();
    let document = repo.run_json(&[], 1);
    for check in ALL_CHECKS.iter().copied().chain([REGION_CHECK]) {
        proven(&repo, &document, check);
    }
}

// behavior-contexts-launches: The core compares the component reports with the launch's condition before it applies the Check rule
#[test]
fn the_core_compares_the_reports_with_the_condition_before_it_applies_the_check_rule() {
    // A failing test is a fail under a correct report, and no result under a wrong one.
    let right = instant_true_with(&[]);
    set_outcome(&right, "failed");
    let document = right.run_json(&[], 1);
    assert_eq!(state_of(&document, INSTANT_CHECK), "failed", "{document:#}");
    let wrong = instant_true_with(&[WRONG_START]);
    set_outcome(&wrong, "failed");
    let document = wrong.run_json(&[], 1);
    owed_not_applied(&wrong, &document, INSTANT_CHECK);
}

// behavior-contexts-launches: Only stated keys are compared
#[test]
fn only_stated_keys_are_compared() {
    // `payouts.region` and `payouts.limit` are mapped and not stated: whatever the reports hold passes.
    let repo = instant_true_with(&[
        "maps api payouts.instant payouts.region=no payouts.limit",
        "maps website payouts.instant",
        "component api both source payouts.region after-azimuth flag-service",
        "component api both source payouts.limit before-azimuth configuration",
    ]);
    proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
}

// behavior-contexts-launches: For a stated key the core expects every component that is in `starts` and listed for that key
#[test]
fn for_a_stated_key_every_started_component_listed_for_it_is_expected() {
    // `payouts.instant` is listed for both started components.
    let right = instant_true_with(&[]);
    proven(&right, &right.run_json(&[], 0), INSTANT_CHECK);
    for (wrong, name) in [("component api both source payouts.instant before-azimuth configuration", API), ("component website both source payouts.instant before-azimuth configuration", WEBSITE)] {
        let repo = instant_true_with(&[wrong]);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(detail.contains(name), "{name} is not named: {detail}");
    }
}

// behavior-contexts-launches: A component listed for the key but not in `starts` is not expected
#[test]
fn a_component_listed_for_the_key_but_not_in_starts_is_not_expected() {
    let repo = launches_with(&[INSTANT_TRUE_LINE], &[], &[]);
    start_components(&repo, &[API]);
    applied(&repo);
    let document = repo.run_json(&[], 0);
    proven(&repo, &document, INSTANT_CHECK);
    let all = launches(&repo);
    let launch = all.iter().find(|launch| launch.requests(INSTANT_CHECK)).unwrap();
    let names: Vec<&str> = launch.request["components"].as_array().unwrap().iter().map(|entry| entry["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec![API], "{launch:#?}");
}

// behavior-contexts-launches: A component in `starts` but not listed for the key is not expected
#[test]
fn a_component_in_starts_but_not_listed_for_the_key_is_not_expected() {
    // `payouts.region` is read by `api` alone, and `website` reports nonsense for it.
    let repo = launches_with(&[REGION_SE_LINE], &[], &[]);
    applied(&repo);
    give(
        &repo,
        &[
            "component website both value payouts.region zz",
            "component website both applying not-possible",
            "component website both source payouts.region after-azimuth flag-service",
        ],
    );
    proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
}

// behavior-contexts-launches: Each expected report must give the stated typed value
#[test]
fn each_expected_report_must_give_the_stated_typed_value() {
    let both = ["api", "website"];
    // (condition line, key, reported text, components that report it, accepted)
    let cases: [(&str, &str, &str, &[&str], bool); 9] = [
        ("Context: payouts.instant=true", "payouts.instant", "TRUE", &both, true),
        ("Context: payouts.instant=true", "payouts.instant", "false", &both, false),
        ("Context: payouts.limit=1e3", "payouts.limit", "1000.0", &["api"], true),
        ("Context: payouts.limit=1e3", "payouts.limit", "1e3", &["api"], true),
        ("Context: payouts.limit=1e3", "payouts.limit", "1001", &["api"], false),
        ("Context: payouts.region=\"\u{e9}\"", "payouts.region", "e\u{301}", &["api"], true),
        ("Context: payouts.region=\"\u{e9}\"", "payouts.region", "\u{c9}", &["api"], false),
        ("Context: payouts.region=\"se\"", "payouts.region", "SE", &["api"], false),
        ("Context: payouts.region=\"se\"", "payouts.region", "se", &["api"], true),
    ];
    for (line, key, reported, components, accepted) in cases {
        let repo = launches_with(&[line], &[], &[]);
        applied(&repo);
        for component in components {
            give(&repo, &[&format!("component {component} both value {key} {reported}")]);
        }
        if accepted {
            proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
        } else {
            let document = repo.run_json(&[], 1);
            owed_not_applied(&repo, &document, INSTANT_CHECK);
        }
    }
}

// behavior-contexts-launches: A value reported with source precedence `azimuth` and applying `possible` is accepted
#[test]
fn a_value_with_source_precedence_azimuth_and_applying_possible_is_accepted() {
    // The name of the source is the source's own, and the precedence is what the launch reads.
    let repo = instant_true_with(&[
        "component api both source payouts.instant azimuth flag-service",
        "component website both source payouts.instant azimuth flag-service",
    ]);
    proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
}

// behavior-contexts-launches: A correct value from configuration alone is not accepted
#[test]
fn a_correct_value_from_configuration_alone_is_not_accepted() {
    let repo = instant_true_with(&[WRONG_START]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains(NEVER_REACHED), "{detail}");
}

// behavior-contexts-launches: The condition must hold in the `start` read
#[test]
fn the_condition_must_hold_in_the_start_read() {
    let control = instant_true_with(&[]);
    proven(&control, &control.run_json(&[], 0), INSTANT_CHECK);
    let repo = instant_true_with(&["component api start source payouts.instant before-azimuth configuration"]);
    let document = repo.run_json(&[], 1);
    owed_not_applied(&repo, &document, INSTANT_CHECK);
}

// behavior-contexts-launches: The condition must hold in the `stop` read
#[test]
fn the_condition_must_hold_in_the_stop_read() {
    let control = instant_true_with(&[]);
    proven(&control, &control.run_json(&[], 0), INSTANT_CHECK);
    let repo = instant_true_with(&["component api stop source payouts.instant before-azimuth configuration"]);
    let document = repo.run_json(&[], 1);
    owed_not_applied(&repo, &document, INSTANT_CHECK);
}

// behavior-contexts-launches: A difference in an unstated key changes nothing
#[test]
fn a_difference_in_an_unstated_key_changes_nothing() {
    // The unstated key reads `no` at start and `se` at stop.
    let repo = instant_true_with(&[
        "maps api payouts.instant payouts.region=no",
        "component api stop value payouts.region se",
        "component api stop source payouts.region after-azimuth flag-service",
    ]);
    proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
}

// behavior-contexts-launches: A difference in a read's time changes nothing
#[test]
fn a_difference_in_a_reads_time_changes_nothing() {
    let repo = instant_true_with(&["component api start read-time 2026-11-24T12:00:00Z", "component api stop read-time 2020-01-01T00:00:00Z"]);
    proven(&repo, &repo.run_json(&[], 0), INSTANT_CHECK);
}

// behavior-contexts-launches: A launch whose context was not applied gives no result for any Check of that launch
#[test]
fn a_launch_whose_context_was_not_applied_gives_no_result_for_any_check_of_that_launch() {
    let repo = launches_with(&[INSTANT_TRUE_LINE], &[], &[INSTANT_TRUE_LINE]);
    applied(&repo);
    give(&repo, &[WRONG_START]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
    assert_eq!(launches(&repo).iter().filter(|launch| launch.requests(INSTANT_CHECK)).count(), 1);
}

// behavior-contexts-launches: A wrong report loses the result of a Check that states no key of that component
#[test]
fn a_wrong_report_loses_the_result_of_a_check_that_states_no_key_of_that_component() {
    // `website` reads `payouts.instant` and not `payouts.region`. Every Check of the launch states
    // both keys, so the design's case of a Check that states no key of the wrong component cannot
    // arise inside one launch; the Checks here are lost although one of the stated keys is not read by `website`.
    let control = launches_with(&[INSTANT_TRUE_LINE, REGION_SE_LINE], &[], &[INSTANT_TRUE_LINE, REGION_SE_LINE]);
    applied(&control);
    let document = control.run_json(&[], 0);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        proven(&control, &document, check);
    }
    let repo = launches_with(&[INSTANT_TRUE_LINE, REGION_SE_LINE], &[], &[INSTANT_TRUE_LINE, REGION_SE_LINE]);
    applied(&repo);
    give(&repo, &["component website both source payouts.instant before-azimuth configuration"]);
    let document = repo.run_json(&[], 1);
    for check in [INSTANT_CHECK, REGION_CHECK] {
        owed_not_applied(&repo, &document, check);
    }
}

// behavior-contexts-launches: The state is `owed` and the reason is `context not applied`
#[test]
fn the_state_is_owed_and_the_reason_is_context_not_applied() {
    let control = instant_true_with(&[]);
    proven(&control, &control.run_json(&[], 0), INSTANT_CHECK);
    let repo = instant_true_with(&[WRONG_START]);
    let document = repo.run_json(&[], 1);
    let entry = entry_of(&document, INSTANT_CHECK);
    assert_eq!(entry["state"], "owed", "{document:#}");
    assert_eq!(entry["reason"], "context not applied", "{document:#}");
}

// behavior-contexts-launches: The detail names the component, key, stated and reported values, and cause
#[test]
fn the_detail_names_the_component_key_stated_and_reported_values_and_cause() {
    let repo = instant_true_with(&[WRONG_START, "component api both value payouts.instant false"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains("api, "), "the component is not named first: {detail}");
    assert!(detail.contains("payouts.instant stated true, reported false"), "{detail}");
    assert!(detail.contains(NEVER_REACHED), "{detail}");
}

// behavior-contexts-launches: Cause: applying is off
#[test]
fn cause_applying_is_off() {
    let repo = instant_true_with(&["component api both applying not-possible"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains("applying is off") && detail.contains(API), "{detail}");
}

// behavior-contexts-launches: Cause: the context never reached the component
#[test]
fn cause_the_context_never_reached_the_component() {
    let repo = instant_true_with(&["component api both source payouts.instant before-azimuth configuration", "component api both value payouts.instant false"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains(NEVER_REACHED) && detail.contains(API), "{detail}");
}

// behavior-contexts-launches: Cause: an overriding source, which the detail names
#[test]
fn cause_an_overriding_source_which_the_detail_names() {
    for directives in [
        vec!["component api both value payouts.instant false", "component api both source payouts.instant after-azimuth flag-service"],
        vec!["component api both source payouts.instant after-azimuth flag-service"],
    ] {
        let repo = instant_true_with(&directives);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(detail.contains("flag-service"), "{directives:?}: {detail}");
        assert!(!detail.contains(NEVER_REACHED), "{directives:?}: {detail}");
    }
}

// behavior-contexts-launches: Cause: a missing key or value, reported as absent
#[test]
fn cause_a_missing_key_or_value_reported_as_absent() {
    for directive in ["component api both absent payouts.instant", "component api both drop payouts.instant"] {
        let repo = instant_true_with(&[directive]);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(detail.contains("absent") && detail.contains("payouts.instant") && detail.contains(API), "{directive}: {detail}");
    }
}

// behavior-contexts-launches: Cause: a value that cannot be read by its type
#[test]
fn cause_a_value_that_cannot_be_read_by_its_type() {
    let repo = instant_true_with(&["component api both value payouts.instant maybe"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains("a value that cannot be read by its type"), "{detail}");
    assert!(detail.contains("maybe") && detail.contains(API), "{detail}");
}

// behavior-contexts-launches: Cause: a wrong component name, naming both the expected and the reported component
#[test]
fn cause_a_wrong_component_name_naming_both_the_expected_and_the_reported_component() {
    let repo = instant_true_with(&["component api both name billing"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains(API) && detail.contains("billing"), "{detail}");
}

// behavior-contexts-launches: Cause: an address already answering
#[test]
fn cause_an_address_already_answering() {
    let repo = instant_true_with(&["component api answered"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains("an address already answering") && detail.contains(API), "{detail}");
}

// behavior-contexts-launches: Cause: a report missing or unreadable, naming its declared place
#[test]
fn cause_a_report_missing_or_unreadable_naming_its_declared_place() {
    for directive in [
        "component website none",
        "component website both error connection refused",
        "component website both malformed",
        "component website both incomplete",
    ] {
        let repo = instant_true_with(&[directive]);
        let document = repo.run_json(&[], 1);
        let detail = not_applied(&repo, &document, INSTANT_CHECK);
        assert!(detail.contains(WEBSITE) && detail.contains(WEBSITE_PLACE), "{directive}: {detail}");
    }
}

// behavior-contexts-launches: Cause: no started component reading a stated key
#[test]
fn cause_no_started_component_reading_a_stated_key() {
    // `payouts.region` is read by `api` alone, and the suite starts `website`.
    let repo = launches_with(&[REGION_SE_LINE], &[], &[]);
    start_components(&repo, &[WEBSITE]);
    applied(&repo);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains("no started component reading a stated key"), "{detail}");
    assert!(detail.contains("payouts.region"), "{detail}");
}

// behavior-contexts-launches: A condition that no started component can apply gives "context not applied" in both commands, also in a suite with Checks that have no condition
#[test]
fn a_condition_that_no_started_component_can_apply_is_owed_in_both_commands_beside_checks_with_no_condition() {
    let repo = launches_with(&[], &[], &[REGION_SE_LINE]);
    start_components(&repo, &[WEBSITE]);
    applied(&repo);
    let document = repo.run_json(&[], 1);
    owed_not_applied(&repo, &document, REGION_CHECK);
    for check in [INSTANT_CHECK, REVIEW_CHECK].into_iter().chain(ALL_CHECKS) {
        proven(&repo, &document, check);
    }
    let output = repo.azimuth(&["check", "run", REGION_CHECK, "--json"]);
    output.expect_status(1);
    owed_not_applied(&repo, &output.json(), REGION_CHECK);
}

// behavior-contexts-launches: A launch whose context was not applied gives no result even when a test of a Check failed
#[test]
fn a_launch_whose_context_was_not_applied_gives_no_result_even_when_a_test_failed() {
    let control = instant_true_with(&[]);
    set_outcome(&control, "failed");
    assert_eq!(state_of(&control.run_json(&[], 1), INSTANT_CHECK), "failed");
    let repo = instant_true_with(&[WRONG_START]);
    set_outcome(&repo, "failed");
    let document = repo.run_json(&[], 1);
    owed_not_applied(&repo, &document, INSTANT_CHECK);
    assert_ne!(state_of(&document, INSTANT_CHECK), "failed");
}

// behavior-contexts-launches: A Check whose condition was not applied has no result, whatever its tests did
#[test]
fn a_check_whose_condition_was_not_applied_has_no_result_whatever_its_tests_did() {
    let control = instant_true_with(&[]);
    proven(&control, &control.run_json(&[], 0), INSTANT_CHECK);
    for outcome in ["passed", "failed", "skipped", "not_run"] {
        let repo = instant_true_with(&[WRONG_START]);
        set_outcome(&repo, outcome);
        let document = repo.run_json(&[], 1);
        owed_not_applied(&repo, &document, INSTANT_CHECK);
    }
}

// behavior-contexts-launches: A launch whose context was applied has its Checks decided by the Check rule
#[test]
fn a_launch_whose_context_was_applied_has_its_checks_decided_by_the_check_rule() {
    let passed = instant_true_with(&[]);
    proven(&passed, &passed.run_json(&[], 0), INSTANT_CHECK);
    let failed = instant_true_with(&[]);
    set_outcome(&failed, "failed");
    let document = failed.run_json(&[], 1);
    assert_eq!(state_of(&document, INSTANT_CHECK), "failed", "{document:#}");
    assert!(strings_of(&entry_of(&document, INSTANT_CHECK)["failing_tests"]).contains(&SENDS_TEST.to_string()), "{document:#}");
    for outcome in ["skipped", "not_run"] {
        let repo = instant_true_with(&[]);
        set_outcome(&repo, outcome);
        let document = repo.run_json(&[], 1);
        no_result(&repo, &document, INSTANT_CHECK);
    }
}

// behavior-contexts-launches: A Check whose launch gave no result for this cause is `owed` in `run-1`, with the reason `context not applied`
#[test]
fn a_check_whose_launch_gave_no_result_is_owed_with_the_reason_that_names_the_fault() {
    let wrong = instant_true_with(&[WRONG_START]);
    let document = wrong.run_json(&[], 1);
    assert_eq!(reason_of(&document, INSTANT_CHECK), "context not applied", "{document:#}");
    assert!(!detail_of_entry(&entry_of(&document, INSTANT_CHECK)).trim().is_empty(), "{document:#}");
    // Another fault of the launch keeps the reason `no result`, and its detail names its cause.
    let crashed = instant_true_with(&["when payouts.instant=true crash"]);
    let document = crashed.run_json(&[], 1);
    no_result(&crashed, &document, INSTANT_CHECK);
    assert!(!detail_of_entry(&entry_of(&document, INSTANT_CHECK)).trim().is_empty(), "{document:#}");
}

// behavior-contexts-launches: A launch with no result stores no result record
#[test]
fn a_launch_with_no_result_stores_no_result_record() {
    let control = instant_true_with(&[]);
    control.run_json(&[], 0);
    assert_eq!(control.results_of(INSTANT_CHECK).len(), 1, "a launch that applied its context stores its result");
    let repo = instant_true_with(&[WRONG_START]);
    repo.run_json(&[], 1);
    assert!(repo.results_of(INSTANT_CHECK).is_empty());
    assert!(!repo.record_strings().iter().any(|text| text.contains(INSTANT_CHECK)), "a record names {INSTANT_CHECK}");
    for check in ALL_CHECKS {
        assert_eq!(repo.results_of(check).len(), 1, "the launch of {check} gave a result, so the store works");
    }
}

// behavior-contexts-launches: A later `azimuth status` shows the reason the Check had before the run
#[test]
fn a_later_status_shows_the_reason_the_check_had_before_the_run() {
    let never_run = instant_true_with(&[WRONG_START]);
    let before = status_reason(&never_run, INSTANT_CHECK);
    assert!(!before.is_empty());
    never_run.run_json(&[], 1);
    assert_eq!(status_reason(&never_run, INSTANT_CHECK), before);

    let proven_once = instant_true_with(&[]);
    proven_once.run_json(&[], 0);
    give(&proven_once, &[WRONG_START]);
    let before = status_reason(&proven_once, INSTANT_CHECK);
    assert_ne!(before, "context not applied", "the reason before the run is not one the run can give");
    proven_once.run_json(&[], 1);
    assert_eq!(status_reason(&proven_once, INSTANT_CHECK), before);
}

// behavior-contexts-launches: Example: `api` reports `payouts.instant=true` where the launch states `false`, and the Checks are owed with "context not applied: api, the context never reached the component; payouts.instant stated false, reported true"
#[test]
fn example_api_reports_true_where_the_launch_states_false() {
    let repo = launches_with(&[], &[INSTANT_FALSE_LINE], &[]);
    applied(&repo);
    give(&repo, &["component api both value payouts.instant true", "component api both source payouts.instant before-azimuth configuration"]);
    let document = repo.run_json(&[], 1);
    not_applied(&repo, &document, REVIEW_CHECK);
    let shown = shown_reason(&entry_of(&document, REVIEW_CHECK));
    assert!(
        shown.contains("context not applied: api, the context never reached the component; payouts.instant stated false, reported true"),
        "{shown}"
    );
    for check in [INSTANT_CHECK, REGION_CHECK].into_iter().chain(ALL_CHECKS) {
        proven(&repo, &document, check);
    }
}

// behavior-contexts-launches: A launch with a failed Check does not stop the next launch of its suite
#[test]
fn a_launch_with_a_failed_check_does_not_stop_the_next_launch_of_its_suite() {
    let repo = four_launches();
    applied(&repo);
    let failing: Vec<String> = every_test_of_the_sample().iter().map(|test| format!("fails {test}")).collect();
    give(&repo, &failing.iter().map(String::as_str).collect::<Vec<_>>());
    let document = repo.run_json(&[], 1);
    assert_eq!(launches(&repo).len(), 4, "a launch did not run");
    for check in payout_and_lending_checks() {
        assert_eq!(state_of(&document, check), "failed", "{check}: {document:#}");
    }
}

// behavior-contexts-launches: A launch whose context was not applied does not stop the next launch of its suite
#[test]
fn a_launch_whose_context_was_not_applied_does_not_stop_the_next_launch_of_its_suite() {
    let repo = four_launches();
    applied(&repo);
    give(&repo, &[WRONG_START]);
    let document = repo.run_json(&[], 1);
    assert_eq!(launches(&repo).len(), 4, "a launch did not run");
    for check in PAYOUT_SCENARIOS {
        owed_not_applied(&repo, &document, check);
    }
    for check in ALL_CHECKS {
        proven(&repo, &document, check);
    }
}

// behavior-contexts-launches: A launch that crashes gives no results and does not stop the next launch of its suite
#[test]
fn a_launch_that_crashes_gives_no_results_and_does_not_stop_the_next_launch_of_its_suite() {
    let repo = four_launches();
    applied(&repo);
    give(&repo, &["crash"]);
    let document = repo.run_json(&[], 1);
    assert_eq!(launches(&repo).len(), 4, "a launch did not run");
    for check in payout_and_lending_checks() {
        no_result(&repo, &document, check);
    }
}

// behavior-contexts-launches: A launch whose build failed does not stop the next launch of its suite
#[test]
fn a_launch_whose_build_failed_does_not_stop_the_next_launch_of_its_suite() {
    let repo = four_launches();
    applied(&repo);
    give(&repo, &["build-fails"]);
    let document = repo.run_json(&[], 1);
    assert_eq!(launches(&repo).len(), 4, "a launch did not run");
    for check in payout_and_lending_checks() {
        no_result(&repo, &document, check);
    }
}

// behavior-contexts-launches: Azimuth's own tests prove launches with a small script as an adapter
#[test]
fn launches_are_proven_with_a_small_script_as_the_adapter() {
    let repo = four_launches();
    assert!(repo.read(PROJECT_FILE).contains("adapter = \"text-suite-adapter\""));
    applied(&repo);
    let document = repo.run_json(&[], 0);
    let all = launches(&repo);
    assert_eq!(all.len(), 4, "{all:#?}");
    for check in payout_and_lending_checks() {
        proven(&repo, &document, check);
    }
    let adapters: BTreeSet<String> = repo.adapter_starts().iter().map(|start| start["argv0"].as_str().unwrap().rsplit('/').next().unwrap().to_string()).collect();
    assert_eq!(adapters, ids(&["text-suite-adapter"]));
}
