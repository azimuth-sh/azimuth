//! What a run and a result record of a launch: `docs/increments/contexts-launch-records.md`. The
//! design gives the added fields and says that text shows a launch's suite, condition, duration and
//! requested Checks; it does not give the layout of a line. A test about text therefore looks for
//! those facts on the lines that name the launch's suite and condition, and never for a layout.
//!
//! The order of launches of one suite is not stated, so no test asserts which runs first. A test
//! about a crash lets each launch crash in turn, so it holds whichever comes first.

use std::collections::BTreeSet;

use azimuth_tests::*;

#[path = "support/contexts_conditions.rs"]
mod contexts_conditions;
#[path = "support/contexts_launches.rs"]
mod contexts_launches;
use contexts_conditions::*;
use contexts_launches::*;

const SUITE: &str = "desk-checks";
const WRONG_START: &str = "component api both source payouts.instant before-azimuth configuration";
const START_TIME: &str = "2026-11-24T10:15:00Z";
const STOP_TIME: &str = "2026-11-24T10:16:30Z";
const DIFFERS: &str = "the reported value differs from the stated value";

fn suites_of(document: &serde_json::Value) -> Vec<serde_json::Value> {
    document["suites"].as_array().unwrap_or_else(|| panic!("no `suites`: {document:#}")).clone()
}

fn entry_with<'a>(entries: &'a [serde_json::Value], pairs: &[(&str, &str)]) -> &'a serde_json::Value {
    let wanted = condition_object(pairs);
    let matching: Vec<&serde_json::Value> = entries.iter().filter(|entry| entry["condition"] == wanted).collect();
    assert_eq!(matching.len(), 1, "{} entries have the condition {wanted}: {entries:#?}", matching.len());
    matching[0]
}

fn checks_of(entry: &serde_json::Value) -> BTreeSet<String> {
    strings_of(&entry["checks"]).into_iter().collect()
}

/// The reads the adapter collects for one component when it maps only the condition's keys, each
/// applied by Azimuth.
fn faithful_reads(component: &str, condition: &[(&str, &str)]) -> serde_json::Value {
    let mut context = serde_json::Map::new();
    for (key, value) in condition {
        context.insert(
            key.to_string(),
            json!({ "value": value, "source": { "name": "azimuth", "precedence": "azimuth" }, "read": START_TIME }),
        );
    }
    let read = |phase: &str, time: &str| {
        let mut context = context.clone();
        for entry in context.values_mut() {
            entry["read"] = json!(time);
        }
        json!({ "phase": phase, "report": { "format": 1, "component": component, "applying": "possible", "context": context } })
    };
    json!({ "name": component, "reads": [read("start", START_TIME), read("stop", STOP_TIME)] })
}

fn expected_components(condition: &[(&str, &str)]) -> BTreeSet<String> {
    [API, WEBSITE].iter().map(|component| faithful_reads(component, condition).to_string()).collect()
}

fn as_strings(components: &serde_json::Value) -> BTreeSet<String> {
    components
        .as_array()
        .unwrap_or_else(|| panic!("`components` is no list: {components:#}"))
        .iter()
        .map(|component| component.to_string())
        .collect()
}

fn component_named<'a>(components: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    components
        .as_array()
        .and_then(|list| list.iter().find(|component| component["name"] == name))
        .unwrap_or_else(|| panic!("no component {name}: {components:#}"))
}

fn read_of<'a>(component: &'a serde_json::Value, phase: &str) -> &'a serde_json::Value {
    component["reads"]
        .as_array()
        .and_then(|reads| reads.iter().find(|read| read["phase"] == phase))
        .unwrap_or_else(|| panic!("no {phase} read: {component:#}"))
}

/// The lines of a text that name the suite and the stated condition of one launch.
fn launch_lines(text: &str, condition_text: &str) -> Vec<String> {
    text.lines().filter(|line| line.contains(SUITE) && line.contains(condition_text)).map(str::to_string).collect()
}

fn has_duration(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    words.windows(2).any(|pair| pair[0].parse::<u64>().is_ok() && pair[1].trim_end_matches(|c: char| !c.is_alphanumeric()) == "ms")
        || words.iter().any(|word| word.strip_suffix("ms").is_some_and(|number| number.parse::<u64>().is_ok()))
}

fn crashed_when(value: &str) -> Repo {
    let repo = two_conditions();
    applied(&repo);
    give(&repo, &[&format!("when payouts.instant={value} crash")]);
    repo
}

// behavior-contexts-launch-records: `suites` in `run-1` has one entry per launch, so a suite's name can occur more than once
#[test]
fn suites_has_one_entry_per_launch_so_a_name_occurs_more_than_once() {
    let repo = two_conditions();
    applied(&repo);
    let document = repo.run_json(&[], 0);
    let entries = suites_of(&document);
    assert_eq!(entries.len(), launches(&repo).len(), "{document:#}");
    assert_eq!(entries.len(), 3, "{document:#}");
    assert!(entries.iter().all(|entry| entry["name"] == SUITE), "{document:#}");
    let conditions: BTreeSet<String> = entries.iter().map(|entry| entry["condition"].to_string()).collect();
    assert_eq!(conditions.len(), 3, "the entries do not tell the launches apart: {document:#}");
}

// behavior-contexts-launch-records: Each `suites` entry gains `condition`
#[test]
fn each_entry_gains_the_condition_of_its_launch() {
    let repo = two_conditions();
    applied(&repo);
    let entries = suites_of(&repo.run_json(&[], 0));
    let all = launches(&repo);
    for launch in &all {
        let wanted = launch.condition();
        assert_eq!(entries.iter().filter(|entry| entry["condition"] == *wanted).count(), 1, "{wanted}: {entries:#?}");
    }
    assert_eq!(entry_with(&entries, &instant_pairs("true"))["condition"], json!({ "payouts.instant": "true" }));
    // A launch with no condition carries `{}` as its condition, so its entry holds `{}`.
    assert_eq!(entry_with(&entries, &[])["condition"], json!({}), "{entries:#?}");
}

fn instant_pairs(value: &str) -> Vec<(&'static str, &'static str)> {
    vec![("payouts.instant", if value == "true" { "true" } else { "false" })]
}

// behavior-contexts-launch-records: Each `suites` entry gains `checks`, the ids requested for that launch
#[test]
fn each_entry_gains_the_checks_requested_for_its_launch() {
    let repo = two_conditions();
    applied(&repo);
    let entries = suites_of(&repo.run_json(&[], 0));
    assert_eq!(checks_of(entry_with(&entries, &instant_pairs("true"))), ids(&[INSTANT_CHECK]), "{entries:#?}");
    assert_eq!(checks_of(entry_with(&entries, &instant_pairs("false"))), ids(&[REVIEW_CHECK]), "{entries:#?}");
    assert_eq!(checks_of(entry_with(&entries, &[])), lending_checks(), "{entries:#?}");
    for launch in launches(&repo) {
        assert_eq!(checks_of(entry_with(&entries, &launch_pairs(&launch))), launch.checks(), "{entries:#?}");
    }
}

fn launch_pairs(launch: &Launch) -> Vec<(&'static str, &'static str)> {
    match launch.condition()["payouts.instant"].as_str() {
        Some("true") => instant_pairs("true"),
        Some("false") => instant_pairs("false"),
        _ => Vec::new(),
    }
}

// behavior-contexts-launch-records: Each `suites` entry gains `components`, the collected reads
#[test]
fn each_entry_gains_the_reads_its_launch_collected() {
    let repo = two_conditions();
    applied(&repo);
    let entries = suites_of(&repo.run_json(&[], 0));
    let on = entry_with(&entries, &instant_pairs("true"));
    assert_eq!(as_strings(&on["components"]), expected_components(&[("payouts.instant", "true")]), "{on:#}");
    let off = entry_with(&entries, &instant_pairs("false"));
    assert_eq!(as_strings(&off["components"]), expected_components(&[("payouts.instant", "false")]), "{off:#}");

    // A launch with no condition has the reads collected for it, and none when the adapter collects none.
    assert_eq!(as_strings(&entry_with(&entries, &[])["components"]), expected_components(&[]), "{entries:#?}");
    let bare = Repo::sample();
    let document = bare.run_json(&[], 0);
    for entry in suites_of(&document) {
        assert_eq!(entry["condition"], json!({}), "{entry:#}");
        assert_eq!(entry["components"], json!([]), "the adapter collected nothing: {entry:#}");
    }
}

// behavior-contexts-launch-records: `launch` and `duration_ms` of an entry describe that launch
#[test]
fn launch_and_duration_describe_that_launch() {
    let repo = two_conditions();
    applied(&repo);
    give(&repo, &["when payouts.instant=true sleeps 1500", "when payouts.instant=false crash"]);
    let document = repo.run_json(&[], 1);
    let entries = suites_of(&document);
    let slow = entry_with(&entries, &instant_pairs("true"));
    assert_eq!(slow["launch"], "ok", "{document:#}");
    assert!(slow["duration_ms"].as_u64().unwrap_or(0) >= 1500, "{document:#}");
    let crashed = entry_with(&entries, &instant_pairs("false"));
    assert_eq!(crashed["launch"], "crashed", "{document:#}");
    assert!(crashed["duration_ms"].as_u64().is_some_and(|ms| ms < 1500), "the crash took the time of the other launch: {document:#}");
    let free = entry_with(&entries, &[]);
    assert_eq!(free["launch"], "ok", "{document:#}");
    assert!(free["duration_ms"].as_u64().is_some_and(|ms| ms < 1500), "{document:#}");
}

// behavior-contexts-launch-records: The top-level `checks` holds each Check's state once
#[test]
fn the_top_level_checks_hold_each_check_once() {
    let repo = four_launches();
    applied(&repo);
    let document = repo.run_json(&[], 0);
    let listed: Vec<String> = document["checks"]
        .as_array()
        .unwrap_or_else(|| panic!("{document:#}"))
        .iter()
        .map(|check| check["id"].as_str().unwrap_or_default().to_string())
        .collect();
    let unique: BTreeSet<&String> = listed.iter().collect();
    assert_eq!(listed.len(), unique.len(), "a Check is listed twice: {listed:?}");
    let wanted: BTreeSet<String> = all_launch_checks().iter().map(|id| id.to_string()).collect();
    assert_eq!(unique.into_iter().cloned().collect::<BTreeSet<_>>(), wanted, "{document:#}");
}

// behavior-contexts-launch-records: `run-1` keeps its number
#[test]
fn run_keeps_its_number() {
    let repo = two_conditions();
    applied(&repo);
    assert_eq!(repo.run_json(&[], 0)["format"], "run-1");
    let output = repo.azimuth(&["check", "run", INSTANT_CHECK, "--json"]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "run-1", "{document:#}");
    assert_eq!(suites_of(&document).len(), 1, "{document:#}");
}

// behavior-contexts-launch-records: A crash of the first launch does not remove the second from the plan
#[test]
fn a_crash_of_one_launch_leaves_the_other_in_the_plan() {
    for (crashing, other, other_check) in [("true", "false", REVIEW_CHECK), ("false", "true", INSTANT_CHECK)] {
        let repo = crashed_when(crashing);
        let document = repo.run_json(&[], 1);
        let entries = suites_of(&document);
        assert_eq!(entries.len(), 3, "{crashing}: {document:#}");
        assert_eq!(entry_with(&entries, &instant_pairs(crashing))["launch"], "crashed", "{document:#}");
        assert_eq!(entry_with(&entries, &instant_pairs(other))["launch"], "ok", "{crashing}: {document:#}");
        assert_eq!(entry_with(&entries, &[])["launch"], "ok", "{crashing}: {document:#}");
        assert_eq!(launches(&repo).len(), 3, "every launch started: {:#?}", launches(&repo));
        proven(&repo, &document, other_check);
    }
}

// behavior-contexts-launch-records: `run-1` exposes the reads, so a launch that gives no result can be inspected in the output of the command
#[test]
fn run_exposes_the_reads_of_a_launch_that_gave_no_result() {
    let repo = instant_true_with(&[WRONG_START]);
    let document = repo.run_json(&[], 1);
    owed_not_applied(&repo, &document, INSTANT_CHECK);
    assert!(repo.results_of(INSTANT_CHECK).is_empty());
    let entries = suites_of(&document);
    let entry = entry_with(&entries, &instant_pairs("true"));
    let api = component_named(&entry["components"], API);
    for phase in ["start", "stop"] {
        let context = &read_of(api, phase)["report"]["context"]["payouts.instant"];
        assert_eq!(context["value"], "true", "{phase}: {entry:#}");
        assert_eq!(context["source"]["name"], "configuration", "{phase}: {entry:#}");
        assert_eq!(context["source"]["precedence"], "before azimuth", "{phase}: {entry:#}");
    }
}

fn text_of_run(repo: &Repo, status: i32) -> String {
    let output = repo.azimuth(&["run"]);
    output.expect_status(status);
    output.text()
}

fn assert_launch_shown(text: &str, condition_text: &str, requested: &[&str]) {
    let lines = launch_lines(text, condition_text);
    assert!(!lines.is_empty(), "no line names {SUITE} with {condition_text}:\n{text}");
    let joined = lines.join("\n");
    for check in requested {
        assert!(joined.contains(check), "{condition_text}: no requested Check {check} on its lines:\n{text}");
    }
    assert!(has_duration(&joined), "{condition_text}: no duration on its lines:\n{text}");
}

// behavior-contexts-launch-records: The text shows the suite, condition, duration and requested Checks for each launch
#[test]
fn the_text_shows_suite_condition_duration_and_checks_of_each_launch() {
    let repo = two_conditions();
    applied(&repo);
    give(&repo, &["sleeps 200"]);
    let text = text_of_run(&repo, 0);
    assert_launch_shown(&text, "payouts.instant=true", &[INSTANT_CHECK]);
    assert_launch_shown(&text, "payouts.instant=false", &[REVIEW_CHECK]);
    let true_lines = launch_lines(&text, "payouts.instant=true").join("\n");
    assert!(!true_lines.contains(REVIEW_CHECK), "a Check of the other launch is on these lines:\n{text}");
}

// behavior-contexts-launch-records: The text shows a launch that failed before the next one ran
#[test]
fn the_text_shows_a_launch_that_failed() {
    for crashing in ["true", "false"] {
        let repo = crashed_when(crashing);
        let text = text_of_run(&repo, 1);
        let (check, other_check) = if crashing == "true" { (INSTANT_CHECK, REVIEW_CHECK) } else { (REVIEW_CHECK, INSTANT_CHECK) };
        let other = if crashing == "true" { "false" } else { "true" };
        assert_launch_shown(&text, &format!("payouts.instant={crashing}"), &[check]);
        assert_launch_shown(&text, &format!("payouts.instant={other}"), &[other_check]);
    }
}

// behavior-contexts-launch-records: The start and end lines of a launch name its condition
#[test]
fn the_start_and_end_lines_name_the_condition() {
    let repo = two_conditions();
    applied(&repo);
    let output = repo.azimuth(&["run", "--json"]);
    output.expect_status(0);
    let progress: Vec<&str> = output.stderr.lines().filter(|line| !line.trim().is_empty()).collect();
    for condition in ["payouts.instant=true", "payouts.instant=false"] {
        let naming = progress.iter().filter(|line| line.contains(condition)).count();
        assert_eq!(naming, 2, "{condition} is named by {naming} progress lines, not a start and an end:\n{}", output.stderr);
    }
}

fn result_body(repo: &Repo, check: &str) -> serde_json::Value {
    repo.result_of(check)["body"].clone()
}

// behavior-contexts-launch-records: `result-1` gains `condition` and `components` in its body
#[test]
fn a_result_gains_condition_and_components_in_its_body() {
    let repo = two_conditions();
    applied(&repo);
    repo.run_json(&[], 0);
    for check in [INSTANT_CHECK, REVIEW_CHECK] {
        let record = repo.result_of(check);
        assert_eq!(record["format"], "result-1", "{record:#}");
        assert!(record["body"].get("condition").is_some(), "{check}: {record:#}");
        assert!(record["body"].get("components").is_some(), "{check}: {record:#}");
    }
}

// behavior-contexts-launch-records: The `condition` of a result is the request's condition
#[test]
fn the_condition_of_a_result_is_the_requests_condition() {
    let repo = four_launches();
    applied(&repo);
    repo.run_json(&[], 0);
    let all = launches(&repo);
    for check in all_launch_checks() {
        let asked: Vec<&Launch> = all.iter().filter(|launch| launch.requests(check)).collect();
        assert_eq!(asked.len(), 1, "{check}: {all:#?}");
        assert_eq!(result_body(&repo, check)["condition"], *asked[0].condition(), "{check}");
    }
    // The condition of a launch with no condition is `{}`.
    assert_eq!(result_body(&repo, ALL_CHECKS[0])["condition"], json!({}));
}

// behavior-contexts-launch-records: The `components` of a result are the adapter report's collected reads
#[test]
fn the_components_of_a_result_are_the_reports_collected_reads() {
    let repo = two_conditions();
    applied(&repo);
    repo.run_json(&[], 0);
    assert_eq!(as_strings(&result_body(&repo, INSTANT_CHECK)["components"]), expected_components(&[("payouts.instant", "true")]));
    assert_eq!(as_strings(&result_body(&repo, REVIEW_CHECK)["components"]), expected_components(&[("payouts.instant", "false")]));
    assert_eq!(as_strings(&result_body(&repo, ALL_CHECKS[0])["components"]), expected_components(&[]));
    // An adapter that collects nothing leaves a Check with no condition with no reads.
    let bare = Repo::sample();
    bare.run_json(&[], 0);
    assert_eq!(result_body(&bare, ALL_CHECKS[0])["components"], json!([]));
    assert_eq!(bare.check_show(ALL_CHECKS[0])["components"], json!([]));
}

// behavior-contexts-launch-records: `azimuth check show` exposes the same added fields of `check-1`
#[test]
fn check_show_exposes_the_same_added_fields() {
    let repo = two_conditions();
    applied(&repo);
    repo.run_json(&[], 0);
    for check in [INSTANT_CHECK, REVIEW_CHECK, ALL_CHECKS[0]] {
        let shown = repo.check_show(check);
        let body = result_body(&repo, check);
        assert!(body["condition"].is_object() && body["components"].is_array(), "{check}: {body:#}");
        assert_eq!(shown["condition"], body["condition"], "{check}: {shown:#}");
        assert_eq!(shown["components"], body["components"], "{check}: {shown:#}");
        assert_eq!(shown["format"], "check-1");
    }
}

// behavior-contexts-launch-records: The text of `azimuth check show` lists the mapped values, their sources and read times
#[test]
fn the_text_of_check_show_lists_values_sources_and_read_times() {
    let repo = instant_true_with(&["maps api payouts.instant payouts.region=no"]);
    repo.run_json(&[], 0);
    let output = repo.azimuth(&["check", "show", INSTANT_CHECK]);
    output.expect_status(0);
    let text = output.text();
    for word in ["payouts.instant", "payouts.region", "no", "azimuth", "configuration", START_TIME, STOP_TIME] {
        assert!(text.contains(word), "the text does not show {word}:\n{text}");
    }
    assert!(text.contains(API) && text.contains(WEBSITE), "{text}");
}

// behavior-contexts-launch-records: The kept reads include unstated keys
#[test]
fn the_kept_reads_include_unstated_keys() {
    let repo = instant_true_with(&["maps api payouts.instant payouts.region=no"]);
    repo.run_json(&[], 0);
    let record = repo.result_of(INSTANT_CHECK);
    assert_eq!(record["body"]["condition"], json!({ "payouts.instant": "true" }));
    let api = component_named(&record["body"]["components"], API).clone();
    for phase in ["start", "stop"] {
        let region = &read_of(&api, phase)["report"]["context"]["payouts.region"];
        assert_eq!(region["value"], "no", "{phase}: {api:#}");
        assert_eq!(region["source"]["name"], "configuration", "{phase}: {api:#}");
    }
    assert!(record["body"]["components"].is_array(), "{record:#}");
    assert_eq!(repo.check_show(INSTANT_CHECK)["components"], record["body"]["components"]);
}

// behavior-contexts-launch-records: The same launch's reports appear in each of its Check results
#[test]
fn the_reports_of_a_launch_appear_in_each_of_its_check_results() {
    let repo = launches_with(&[INSTANT_TRUE_LINE], &[INSTANT_TRUE_LINE], &[]);
    applied(&repo);
    repo.run_json(&[], 0);
    let first = result_body(&repo, INSTANT_CHECK);
    let second = result_body(&repo, REVIEW_CHECK);
    assert!(!first["components"].as_array().map_or(true, |list| list.is_empty()), "{first:#}");
    assert_eq!(first["components"], second["components"]);
    assert_eq!(first["condition"], second["condition"]);
}

/// The sample on another machine: the same committed tree, run with that machine's configuration of
/// a key the condition does not state. The fixture is the same files, so no input of a Check moves.
fn on_a_machine_where(region: &str, read_time: Option<&str>) -> Repo {
    let repo = instant_true_with(&["maps api payouts.instant payouts.region"]);
    let mut env = vec![("TEXT_ADAPTER_CONFIGURED", format!("payouts.region={region}"))];
    if let Some(time) = read_time {
        env.push(("TEXT_ADAPTER_READ_TIME", time.to_string()));
    }
    let env: Vec<(&str, &str)> = env.iter().map(|(name, value)| (*name, value.as_str())).collect();
    let output = repo.azimuth_with_env(&["run", "--json"], &env);
    output.expect_status(0);
    let document = output.json();
    proven(&repo, &document, INSTANT_CHECK);
    repo
}

// behavior-contexts-launch-records: A result of a Check on a machine whose unstated key differs has the same fingerprint
#[test]
fn an_unstated_key_that_differs_between_machines_keeps_the_fingerprint() {
    let north = on_a_machine_where("no", None);
    let south = on_a_machine_where("se", None);
    let (a, b) = (result_body(&north, INSTANT_CHECK), result_body(&south, INSTANT_CHECK));
    assert_eq!(a["fingerprint"], b["fingerprint"], "{a:#}\n{b:#}");
    let value_of = |body: &serde_json::Value| {
        read_of(component_named(&body["components"], API), "start")["report"]["context"]["payouts.region"].clone()
    };
    assert_eq!(value_of(&a)["value"], "no", "{a:#}");
    assert_eq!(value_of(&b)["value"], "se", "the reports keep the value visible: {b:#}");
    assert_eq!(value_of(&b)["source"]["name"], "configuration", "{b:#}");
}

// behavior-contexts-launch-records: The collected reports are not an input of the fingerprint
#[test]
fn the_collected_reports_are_not_an_input_of_the_fingerprint() {
    let first = on_a_machine_where("no", Some("2026-11-24T08:00:00Z"));
    let second = on_a_machine_where("se", Some("2026-11-25T09:30:00Z"));
    let (a, b) = (result_body(&first, INSTANT_CHECK), result_body(&second, INSTANT_CHECK));
    assert!(a["components"].is_array() && b["components"].is_array(), "{a:#}\n{b:#}");
    assert_ne!(a["components"], b["components"], "the reports differ in values and times: {a:#}\n{b:#}");
    let parts: BTreeSet<String> = a["parts"].as_object().unwrap().keys().cloned().collect();
    assert_eq!(parts, ["claim", "files", "environment", "method"].iter().map(|part| part.to_string()).collect(), "an extra part holds context: {a:#}");
    assert_eq!(a["parts"], b["parts"], "a part moved with the reports");
    assert_eq!(a["fingerprint"], b["fingerprint"]);
}

// behavior-contexts-launch-records: A report with precedence `azimuth` and a value other than the stated one gives the cause "the reported value differs from the stated value"
#[test]
fn a_value_applied_by_azimuth_that_differs_is_a_cause_of_its_own() {
    let repo = instant_true_with(&["component api both value payouts.instant false"]);
    let document = repo.run_json(&[], 1);
    let detail = not_applied(&repo, &document, INSTANT_CHECK);
    assert!(detail.contains(DIFFERS), "{detail}");
    assert!(detail.contains(API) && detail.contains("payouts.instant"), "{detail}");
}
