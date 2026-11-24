#![allow(dead_code)]
//! Launches on the sample, with the invented payouts product. The sample's suite starts `api` and
//! `website`, and the text-suite adapter stands in for a runner: its directives (see the header of
//! `src/bin/text-suite-adapter.rs`) say what component reports it writes, per launch.
//!
//! Conditions are given to the three payout Scenarios, so a fixture has up to four launches: one
//! for each distinct condition and one for the Checks of the lending desk, which state none.

use std::collections::BTreeSet;

use azimuth_tests::*;

use crate::contexts_conditions::*;

pub const INSTANT_CHECK: &str = "payout-is-sent-at-once";
pub const REVIEW_CHECK: &str = "payout-waits-for-review";
pub const REGION_CHECK: &str = "payout-stays-in-region";
pub const PAYOUT_SCENARIOS: [&str; 3] = [INSTANT_CHECK, REVIEW_CHECK, REGION_CHECK];

pub const INSTANT_TRUE_LINE: &str = "Context: payouts.instant=true";
pub const INSTANT_FALSE_LINE: &str = "Context: payouts.instant=false";
pub const REGION_SE_LINE: &str = "Context: payouts.region=\"se\"";

pub const DIRECTIVES_FILE: &str = "suites/desk-checks/directives.test";
pub const JOURNEY_FILE: &str = "suites/desk-checks/journey.test";
pub const JOURNEY: &str = "journey";

/// The declared place of each component's report.
pub const API_PLACE: &str = "/internal/azimuth";
pub const WEBSITE_PLACE: &str = "/azimuth/context";

const SPEC: &str = "# Payouts\n\nMoney leaves the product only when a payout is approved.\n\n### Requirement: payouts-follow-their-rules\nA payout is paid out by the rules of its recipient.\n\n#### Scenario: payout-is-sent-at-once\nAn approved payout is sent to the card at once.\n\n#### Scenario: payout-waits-for-review\nAn approved payout of a recipient under review is held.\n\n#### Scenario: payout-stays-in-region\nA payout stays in the region of its recipient.\n";
const VERIFICATION: &str = "### Requirement: payouts-follow-their-rules\n#### Scenario: payout-is-sent-at-once\n- Check: payout-is-sent-at-once\n  An approved payout reaches the card at once.\n\n#### Scenario: payout-waits-for-review\n- Check: payout-waits-for-review\n  A payout under review is held until the review ends.\n\n#### Scenario: payout-stays-in-region\n- Check: payout-stays-in-region\n  A payout never leaves the region of its recipient.\n";
const TESTS: &str = "test: sends-at-once\n# Check: payout-is-sent-at-once\noutcome: passed\n\ntest: holds-for-review\n# Check: payout-waits-for-review\noutcome: passed\n\ntest: stays-in-region\n# Check: payout-stays-in-region\noutcome: passed\n";

pub const SENDS_TEST: &str = "payouts.sends-at-once";
pub const HOLDS_TEST: &str = "payouts.holds-for-review";
pub const REGION_TEST: &str = "payouts.stays-in-region";
pub const JOURNEY_TEST: &str = "journey";

/// The sample with the payouts module of three Scenarios, its keys, both components declared with
/// reports at different places, and both started by the suite. Each list is the `Context:` lines of
/// one Scenario. Committed.
pub fn launches_with(instant: &[&str], waits: &[&str], region: &[&str]) -> Repo {
    let repo = Repo::sample();
    repo.write(PAYOUTS_SPEC, SPEC);
    repo.write(PAYOUTS_VERIFICATION, VERIFICATION);
    repo.write(PAYOUTS_TESTS, TESTS);
    repo.write(PAYOUT_KEY_FILE, &payout_keys());
    repo.append(PROJECT_FILE, &component_table(API, &["all"], API_PLACE));
    repo.append(PROJECT_FILE, &component_table(WEBSITE, &["all"], WEBSITE_PLACE));
    start_components(&repo, &[API, WEBSITE]);
    for (scenario, lines) in PAYOUT_SCENARIOS.iter().zip([instant, waits, region]) {
        if !lines.is_empty() {
            under(&repo, PAYOUTS_SPEC, &format!("#### Scenario: {scenario}\n"), lines);
        }
    }
    repo.commit_all("payouts with launches");
    repo
}

/// Four launches: `payouts.instant=true`, `payouts.instant=false`, both `payouts.instant=true`
/// and `payouts.region="se"`, and none for the four Checks of the lending desk.
pub fn four_launches() -> Repo {
    launches_with(&[INSTANT_TRUE_LINE], &[INSTANT_FALSE_LINE], &[INSTANT_TRUE_LINE, REGION_SE_LINE])
}

/// Two launches that state different values of one key, and the lending desk's own.
pub fn two_conditions() -> Repo {
    launches_with(&[INSTANT_TRUE_LINE], &[INSTANT_FALSE_LINE], &[])
}

/// A journey marked for two Checks stands in place of the tests of those Checks.
pub fn journey_for(repo: &Repo, checks: &[&str]) {
    let markers: String = checks.iter().map(|check| format!("# Check: {check}\n")).collect();
    repo.write(JOURNEY_FILE, &format!("{markers}outcome: passed\n"));
    let mut tests = String::new();
    for (test, check) in [("sends-at-once", INSTANT_CHECK), ("holds-for-review", REVIEW_CHECK), ("stays-in-region", REGION_CHECK)] {
        if !checks.contains(&check) {
            tests.push_str(&format!("test: {test}\n# Check: {check}\noutcome: passed\n\n"));
        }
    }
    repo.write(PAYOUTS_TESTS, &tests);
}

/// Appends directives for the text-suite adapter, each of them one line without the prefix.
pub fn give(repo: &Repo, directives: &[&str]) {
    let lines: String = directives.iter().map(|line| format!("adapter: {line}\n")).collect();
    let before = if repo.path(DIRECTIVES_FILE).exists() { repo.read(DIRECTIVES_FILE) } else { String::new() };
    repo.write(DIRECTIVES_FILE, &format!("{before}{lines}"));
}

/// Components that report what a faithful library gives, in every launch.
pub fn applied(repo: &Repo) {
    give(repo, &["components applied"]);
}

pub fn condition_object(pairs: &[(&str, &str)]) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    for (key, value) in pairs {
        object.insert(key.to_string(), json!(value));
    }
    serde_json::Value::Object(object)
}

/// One launch as the apparatus saw it: the request the adapter got, the `AZIMUTH_CONTEXT` it
/// was started with, the tests it ran, and when its process started and ended.
#[derive(Debug, Clone)]
pub struct Launch {
    pub request: serde_json::Value,
    pub context: Option<String>,
    pub ran: BTreeSet<String>,
    pub started: u64,
    pub ended: Option<u64>,
}

impl Launch {
    pub fn condition(&self) -> &serde_json::Value {
        &self.request["condition"]
    }

    pub fn checks(&self) -> BTreeSet<String> {
        strings_of(&self.request["checks"]).into_iter().collect()
    }

    pub fn requests(&self, check: &str) -> bool {
        self.checks().contains(check)
    }
}

/// Every launch since the log was cleared, in the order the adapters started. A launch that was
/// stopped before it reported is listed too, with no end.
pub fn launches(repo: &Repo) -> Vec<Launch> {
    let entries = repo.log_entries();
    let finished = repo.adapter_starts();
    let mut found: Vec<Launch> = Vec::new();
    for asked in entries.iter().filter(|entry| entry["program"] == "text-suite-adapter" && entry["event"] == "requested") {
        let pid = &asked["pid"];
        let start = finished.iter().find(|start| start["pid"] == *pid).expect("an adapter that asked has started");
        found.push(Launch {
            request: asked["asked"].clone(),
            context: asked["azimuth_context"].as_str().map(str::to_string),
            ran: strings_of(&start["ran"]).into_iter().collect(),
            started: start["time_ns"].as_u64().unwrap(),
            ended: entries.iter().find(|entry| entry["event"] == "exited" && entry["pid"] == *pid).and_then(|entry| entry["time_ns"].as_u64()),
        });
    }
    found.sort_by_key(|launch| launch.started);
    found
}

/// The one launch whose request condition is exactly these pairs.
pub fn launch_of<'a>(all: &'a [Launch], pairs: &[(&str, &str)]) -> &'a Launch {
    let wanted = condition_object(pairs);
    let matching: Vec<&Launch> = all.iter().filter(|launch| *launch.condition() == wanted).collect();
    assert_eq!(matching.len(), 1, "{} launches have the condition {wanted}: {all:#?}", matching.len());
    matching[0]
}

pub fn conditions_of(all: &[Launch]) -> Vec<serde_json::Value> {
    all.iter().map(|launch| launch.condition().clone()).collect()
}

pub fn ids(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|id| id.to_string()).collect()
}

pub fn lending_checks() -> BTreeSet<String> {
    ids(&ALL_CHECKS)
}

pub fn entry_of(document: &serde_json::Value, check: &str) -> serde_json::Value {
    checks_by_id(document)[check].clone()
}

/// The words a person reads for an owed Check: the detail, led by the reason unless the detail
/// already starts with it. The design quotes such a message as a whole and does not say which
/// field carries the first words.
pub fn shown_reason(entry: &serde_json::Value) -> String {
    let reason = entry["reason"].as_str().unwrap_or_default();
    let detail = detail_of_entry(entry);
    if detail.starts_with(reason) {
        detail
    } else {
        format!("{reason}: {detail}")
    }
}

/// The Check is owed as "context not applied" and has no result. Returns its detail.
pub fn not_applied(repo: &Repo, document: &serde_json::Value, check: &str) -> String {
    owed_not_applied(repo, document, check);
    detail_of_entry(&entry_of(document, check))
}

/// The Check is owed with the reason `no result` and has no result.
pub fn no_result(repo: &Repo, document: &serde_json::Value, check: &str) {
    assert_eq!(state_of(document, check), "owed", "{check}: {document:#}");
    assert_eq!(reason_of(document, check), "no result", "{check}: {document:#}");
    assert!(repo.results_of(check).is_empty(), "{check} has a result");
}

/// The Check passed in this run and has exactly one result.
pub fn proven(repo: &Repo, document: &serde_json::Value, check: &str) {
    assert_eq!(state_of(document, check), "passed", "{check}: {document:#}");
    assert_eq!(repo.results_of(check).len(), 1, "{check} has no single result");
}

pub fn all_launch_checks() -> Vec<&'static str> {
    PAYOUT_SCENARIOS.iter().copied().chain(ALL_CHECKS).collect()
}

/// Every payout Check of the document is owed as "context not applied".
pub fn payouts_not_applied(repo: &Repo, document: &serde_json::Value, checks: &[&str]) -> Vec<String> {
    checks.iter().map(|check| not_applied(repo, document, check)).collect()
}

/// A repository whose one launch of `payouts.instant=true` reads `api` as these directives say.
pub fn instant_true_with(directives: &[&str]) -> Repo {
    let repo = launches_with(&[INSTANT_TRUE_LINE], &[], &[]);
    applied(&repo);
    give(&repo, directives);
    repo
}

/// The same for `payouts.instant=false`, and the Check `payout-is-sent-at-once` is not run.
pub fn instant_false_with(directives: &[&str]) -> Repo {
    let repo = launches_with(&[INSTANT_FALSE_LINE], &[], &[]);
    applied(&repo);
    give(&repo, directives);
    repo
}

/// Runs the repository and asserts that `payout-is-sent-at-once` ends as expected, with the other
/// Checks of the sample passing. Returns the document.
pub fn run_expecting_owed(repo: &Repo) -> serde_json::Value {
    let document = repo.run_json(&[], 1);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{check}: {document:#}");
    }
    document
}
