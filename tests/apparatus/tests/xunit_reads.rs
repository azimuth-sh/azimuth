//! The rows of the xUnit adapter in `docs/increments/contexts-in-process.md`: the folder for reads
//! it gives the code of the tests, `AZIMUTH_READS`, `components.json`, and how it gathers the read
//! files into the report's `components`.
//!
//! These tests start the real adapter against the sample's xUnit project `payouts/api-tests`, whose
//! tests write read files into the folder `AZIMUTH_READS` names, using what they find in
//! `components.json`. That code stands for a team's fixture; no row is about it. Each test of the
//! project is one scenario with its own Check id, and a test of Azimuth asks for one id at a time.
//! They need the .NET SDK and run only with `cargo test --features dotnet`.
//!
//! The design gives the fields of a read (`name`, `phase`, and `report` or `error`), the place of
//! the entries (`components` of the report) and the words "a file that is not a read ... names the
//! file". It does not give the layout of `components.json`, the stream the file's name goes to, how
//! reads of one component are ordered, or what a component with no read looks like in the report, so
//! a test asserts only what the design gives and looks for a name in the whole text.

use std::collections::BTreeSet;
use std::net::TcpListener;

use azimuth_tests::*;
use serde_json::Value;

#[path = "support/xunit_adapter.rs"]
mod xunit_adapter;
use xunit_adapter::*;

const API: &str = "api";
const WEBSITE: &str = "website";
const API_PLACE: &str = "/internal/azimuth";
const WEBSITE_PLACE: &str = "/azimuth/context";
const BOTH: [(&str, &str); 2] = [(API, API_PLACE), (WEBSITE, WEBSITE_PLACE)];

const PAIR: &str = "host-gives-a-pair";
const ERROR: &str = "host-gives-an-error";
const SEVERAL: &str = "hosts-give-several-pairs";
const UNLISTED: &str = "host-of-an-unlisted-component";
const PROCESS: &str = "website-is-a-separate-process";
const FOLDER: &str = "tests-look-at-their-folder";
const NOTHING: &str = "tests-write-no-read";
const NOT_JSON: &str = "file-is-not-json";
const NO_NAME: &str = "file-has-no-name";
const NO_PHASE: &str = "file-has-no-phase";
const NAME_NOT_TEXT: &str = "name-is-not-text";
const PHASE_NOT_TEXT: &str = "phase-is-not-text";

/// The report of one launch of one scenario. The adapter must answer with a report.
fn launch(repo: &Repo, check: &str) -> Value {
    launch_with(repo, check, &[])
}

fn launch_with(repo: &Repo, check: &str, env: &[(&str, &str)]) -> Value {
    let output = start_adapter(repo, &run_request(&[check], &BOTH), env);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "adapter-report-1", "{document:#}\n{}", output.stderr);
    document
}

/// A launch that must not give a report: the adapter ends with a status other than 0, as a
/// launch that crashed does. Returns what it said, on both streams.
fn crashed(repo: &Repo, check: &str) -> String {
    let output = start_adapter(repo, &run_request(&[check], &BOTH), &[]);
    assert_ne!(output.status, 0, "the launch did not crash:\n{}\n{}", output.stdout, output.stderr);
    output.text()
}

/// Every read of the component `name`, over all the entries that carry that name.
fn reads_of(report: &Value, name: &str) -> Vec<Value> {
    report["components"]
        .as_array()
        .unwrap_or_else(|| panic!("the report has no `components`: {report:#}"))
        .iter()
        .filter(|entry| entry["name"] == name)
        .flat_map(|entry| entry["reads"].as_array().cloned().unwrap_or_default())
        .collect()
}

fn all_reads(report: &Value) -> Vec<Value> {
    report["components"]
        .as_array()
        .map(|entries| entries.iter().flat_map(|entry| entry["reads"].as_array().cloned().unwrap_or_default()).collect())
        .unwrap_or_default()
}

fn phases_of(reads: &[Value]) -> Vec<String> {
    let mut phases: Vec<String> = reads.iter().map(|read| read["phase"].as_str().unwrap_or_default().to_string()).collect();
    phases.sort();
    phases
}

fn notes_of(reads: &[Value]) -> BTreeSet<String> {
    reads.iter().map(|read| read["report"]["note"].as_str().unwrap_or_default().to_string()).collect()
}

fn strs(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

fn outcome_of(report: &Value, test: &str) -> String {
    report["tests"]
        .as_array()
        .and_then(|tests| tests.iter().find(|entry| entry["name"].as_str().is_some_and(|name| name.contains(test))))
        .and_then(|entry| entry["outcome"].as_str())
        .unwrap_or_else(|| panic!("no test {test} in {report:#}"))
        .to_string()
}

/// What the tests recorded about their folder, from the `built` read of `api` in the scenario that looks.
fn found_by_the_tests(report: &Value) -> Value {
    let reads = reads_of(report, API);
    let built = reads.iter().find(|read| read["phase"] == "built").unwrap_or_else(|| panic!("no built read of api: {report:#}"));
    built["report"].clone()
}

/// Whether some object or list in the JSON holds both strings among its direct members, or one of
/// them as a key whose value is the other. The design does not give the layout of `components.json`.
fn pairs_up(node: &Value, first: &str, second: &str) -> bool {
    let direct = |members: Vec<&Value>| {
        let strings: Vec<&str> = members.iter().filter_map(|member| member.as_str()).collect();
        strings.contains(&first) && strings.contains(&second)
    };
    match node {
        Value::Object(map) => {
            direct(map.values().collect())
                || map.get(first).and_then(Value::as_str) == Some(second)
                || map.values().any(|child| pairs_up(child, first, second))
        }
        Value::Array(items) => direct(items.iter().collect()) || items.iter().any(|child| pairs_up(child, first, second)),
        _ => false,
    }
}

fn strings_in(node: &Value) -> Vec<String> {
    match node {
        Value::String(text) => vec![text.clone()],
        Value::Array(items) => items.iter().flat_map(strings_in).collect(),
        Value::Object(map) => map.iter().flat_map(|(key, value)| std::iter::once(key.clone()).chain(strings_in(value))).collect(),
        _ => Vec::new(),
    }
}

// behavior-contexts-in-process: The reads of a host reach the report's `components` under the name of its component
#[test]
fn the_reads_of_a_host_reach_the_reports_components_under_the_name_of_its_component() {
    let repo = sample_with_adapter();
    let report = launch(&repo, PAIR);
    let reads = reads_of(&report, API);
    assert_eq!(phases_of(&reads), strs(&["built", "disposing"]), "{report:#}");
    assert!(reads_of(&report, WEBSITE).is_empty(), "the website gave no read: {report:#}");
}

// behavior-contexts-in-process: A read holds either a `report` or an `error`
#[test]
fn a_read_holds_either_a_report_or_an_error() {
    let repo = sample_with_adapter();
    let report = launch(&repo, ERROR);
    let reads = reads_of(&report, API);
    assert_eq!(reads.len(), 2, "{report:#}");
    for read in &reads {
        assert_ne!(read.get("report").is_some(), read.get("error").is_some(), "a read holds one of them: {read:#}");
    }
    assert!(reads.iter().any(|read| read.get("report").is_some()), "{report:#}");
    assert!(reads.iter().any(|read| read.get("error").is_some()), "{report:#}");
}

// behavior-contexts-in-process: The report carries several pairs of `built` and `disposing` reads for one component
#[test]
fn the_report_carries_several_pairs_of_built_and_disposing_reads_for_one_component() {
    let repo = sample_with_adapter();
    let report = launch(&repo, SEVERAL);
    let reads = reads_of(&report, API);
    assert_eq!(phases_of(&reads), strs(&["built", "built", "disposing", "disposing"]), "{report:#}");
    assert_eq!(notes_of(&reads), ["built 1", "built 2", "disposing 1", "disposing 2"].iter().map(|note| note.to_string()).collect(), "{report:#}");
}

// behavior-contexts-in-process: The adapter gives the code of the tests a folder for reads
#[test]
fn the_adapter_gives_the_code_of_the_tests_a_folder_for_reads() {
    let repo = sample_with_adapter();
    let report = launch(&repo, FOLDER);
    assert_eq!(outcome_of(&report, "The_tests_record_what_they_find_in_the_folder"), "passed", "{report:#}");
    let found = found_by_the_tests(&report);
    assert_eq!(found["exists"], true, "{found:#}");
    assert!(found["folder"].as_str().is_some_and(|folder| !folder.is_empty()), "{found:#}");
}

// behavior-contexts-in-process: For each launch the adapter creates a folder that is empty
#[test]
fn for_each_launch_the_adapter_creates_a_folder_that_is_empty() {
    let repo = sample_with_adapter();
    // Reads of an earlier launch are in the way if the folder is not new. Writing them first makes
    // the listing of the second launch show them.
    launch(&repo, PAIR);
    let report = launch(&repo, FOLDER);
    let found = found_by_the_tests(&report);
    // Only the file the adapter writes itself is there before the tests write any read.
    assert_eq!(found["listing"], json!(["components.json"]), "{found:#}");
}

// behavior-contexts-in-process: The folder of a launch is named in `AZIMUTH_READS`
#[test]
fn the_folder_of_a_launch_is_named_in_azimuth_reads() {
    let repo = sample_with_adapter();
    // The tests find the folder only through `AZIMUTH_READS`, and it exists and holds the file the
    // adapter wrote when they look.
    let found = found_by_the_tests(&launch(&repo, FOLDER));
    let folder = found["folder"].as_str().unwrap();
    assert!(std::path::Path::new(folder).is_absolute(), "{found:#}");
    assert_eq!(found["exists"], true, "{found:#}");
    assert!(found["listing"].as_array().unwrap().iter().any(|file| file == "components.json"), "{found:#}");
}

// behavior-contexts-in-process: Two launches of one suite get different folders
#[test]
fn two_launches_of_one_suite_get_different_folders() {
    let repo = sample_with_adapter();
    let first = found_by_the_tests(&launch(&repo, FOLDER))["folder"].clone();
    let second = found_by_the_tests(&launch(&repo, FOLDER))["folder"].clone();
    assert!(first.is_string() && second.is_string(), "{first} {second}");
    assert_ne!(first, second);
}

// behavior-contexts-in-process: The adapter writes `components.json` in the folder before the tests run
#[test]
fn the_adapter_writes_components_json_in_the_folder_before_the_tests_run() {
    let repo = sample_with_adapter();
    let found = found_by_the_tests(&launch(&repo, FOLDER));
    let text = found["components_json"].as_str().unwrap_or_else(|| panic!("the tests found no components.json: {found:#}"));
    serde_json::from_str::<Value>(text).unwrap_or_else(|error| panic!("components.json is not JSON ({error}): {text}"));
}

// behavior-contexts-in-process: `components.json` lists every component of the request with its name and report path
#[test]
fn components_json_lists_every_component_of_the_request_with_its_name_and_report_path() {
    let repo = sample_with_adapter();
    let found = found_by_the_tests(&launch(&repo, FOLDER));
    let listed: Value = serde_json::from_str(found["components_json"].as_str().unwrap()).unwrap();
    for (name, place) in BOTH {
        assert!(pairs_up(&listed, name, place), "{name} with {place} is not listed: {listed:#}");
    }
    // The tests read the names back from the file, so the file is one a team's code can use.
    assert_eq!(found["names"], json!([API, WEBSITE]), "{found:#}");
}

// behavior-contexts-in-process: `components.json` lists every component of the request with its name and report path
#[test]
fn components_json_is_an_empty_list_when_the_request_has_no_component() {
    let repo = sample_with_adapter();
    let output = start_adapter(&repo, &run_request(&[FOLDER], &[]), &[]);
    output.expect_status(0);
    let report = output.json();
    let found = found_by_the_tests(&report);
    let listed: Value = serde_json::from_str(found["components_json"].as_str().unwrap_or_else(|| panic!("{found:#}"))).unwrap();
    assert!(listed.is_array() || listed.is_object(), "{listed}");
    assert_eq!(strings_in(&listed), Vec::<String>::new(), "the file names a component of a request that has none: {listed}");
    assert_eq!(found["names"], json!([]), "{found:#}");
}

// behavior-contexts-in-process: The adapter takes a read file with `name`, `phase` and `report` into the report
#[test]
fn the_adapter_takes_a_read_file_with_name_phase_and_report_into_the_report() {
    let repo = sample_with_adapter();
    let report = launch(&repo, PAIR);
    let reads = reads_of(&report, API);
    for phase in ["built", "disposing"] {
        let read = reads.iter().find(|read| read["phase"] == phase).unwrap_or_else(|| panic!("no {phase} read: {report:#}"));
        assert_eq!(read["report"], json!({ "format": 1, "component": "api", "applying": "possible", "note": phase }), "{read:#}");
        assert!(read.get("error").is_none(), "{read:#}");
    }
}

// behavior-contexts-in-process: The adapter takes a read file with `name`, `phase` and `error` into the report
#[test]
fn the_adapter_takes_a_read_file_with_name_phase_and_error_into_the_report() {
    let repo = sample_with_adapter();
    let report = launch(&repo, ERROR);
    let reads = reads_of(&report, API);
    let read = reads.iter().find(|read| read["phase"] == "disposing").unwrap_or_else(|| panic!("no disposing read: {report:#}"));
    assert_eq!(read["error"], "the host was disposed before it could be read", "{read:#}");
    assert!(read.get("report").is_none(), "{read:#}");
}

// behavior-contexts-in-process: Every read file the adapter finds after the runner ends is in the report
#[test]
fn every_read_file_the_adapter_finds_after_the_runner_ends_is_in_the_report() {
    let repo = sample_with_adapter();
    let report = launch(&repo, SEVERAL);
    // Four files of `api` and two of `website`.
    assert_eq!(all_reads(&report).len(), 6, "{report:#}");
    assert_eq!(reads_of(&report, WEBSITE).len(), 2, "{report:#}");
}

// behavior-contexts-in-process: A read whose `name` is no component of the request is kept in the report
#[test]
fn a_read_whose_name_is_no_component_of_the_request_is_kept_in_the_report() {
    let repo = sample_with_adapter();
    let report = launch(&repo, UNLISTED);
    let reads = reads_of(&report, "billing");
    assert_eq!(phases_of(&reads), strs(&["built", "disposing"]), "{report:#}");
    assert!(reads.iter().all(|read| read.get("report").is_some()), "{report:#}");
}

// behavior-contexts-in-process: A read file of an earlier launch is not read by a later launch
#[test]
fn a_read_file_of_an_earlier_launch_is_not_read_by_a_later_launch() {
    let repo = sample_with_adapter();
    let earlier = launch(&repo, PAIR);
    assert_eq!(all_reads(&earlier).len(), 2, "{earlier:#}");
    let later = launch(&repo, NOTHING);
    assert_eq!(all_reads(&later).len(), 0, "the later launch carries a read of the earlier one: {later:#}");
}

// behavior-contexts-in-process: A file in the folder that is not JSON makes the launch `crashed`
#[test]
fn a_file_in_the_folder_that_is_not_json_makes_the_launch_crashed() {
    let repo = sample_with_adapter();
    crashed(&repo, NOT_JSON);
    // The same suite without the stray file gives a report, so the crash is the file's doing.
    launch(&repo, PAIR);
}

// behavior-contexts-in-process: A file in the folder with no `name` makes the launch `crashed`
#[test]
fn a_file_in_the_folder_with_no_name_makes_the_launch_crashed() {
    let repo = sample_with_adapter();
    crashed(&repo, NO_NAME);
}

// behavior-contexts-in-process: A file in the folder with no `name` makes the launch `crashed`
#[test]
fn a_file_in_the_folder_whose_name_is_not_text_counts_as_having_no_name() {
    let repo = sample_with_adapter();
    crashed(&repo, NAME_NOT_TEXT);
}

// behavior-contexts-in-process: A file in the folder with no `phase` makes the launch `crashed`
#[test]
fn a_file_in_the_folder_with_no_phase_makes_the_launch_crashed() {
    let repo = sample_with_adapter();
    crashed(&repo, NO_PHASE);
}

// behavior-contexts-in-process: A file in the folder with no `phase` makes the launch `crashed`
#[test]
fn a_file_in_the_folder_whose_phase_is_not_text_counts_as_having_no_phase() {
    let repo = sample_with_adapter();
    crashed(&repo, PHASE_NOT_TEXT);
}

// behavior-contexts-in-process: The adapter names the file that is not a read
#[test]
fn the_adapter_names_the_file_that_is_not_a_read() {
    let repo = sample_with_adapter();
    for (scenario, file) in [
        (NOT_JSON, "notes.txt"),
        (NO_NAME, "nameless.json"),
        (NO_PHASE, "phaseless.json"),
        (NAME_NOT_TEXT, "numbered.json"),
        (PHASE_NOT_TEXT, "unphased.json"),
    ] {
        let said = crashed(&repo, scenario);
        assert!(said.contains(file), "{scenario}: the adapter does not name {file}:\n{said}");
    }
}

// behavior-contexts-in-process: A `start` and a `stop` read that the code of the tests writes in the folder for a separate process reach the report under its component's name
#[test]
fn a_start_and_a_stop_read_of_a_separate_process_reach_the_report_under_its_components_name() {
    let repo = sample_with_adapter();
    let report = launch(&repo, PROCESS);
    let reads = reads_of(&report, WEBSITE);
    assert_eq!(phases_of(&reads), strs(&["start", "stop"]), "{report:#}");
    assert_eq!(notes_of(&reads), ["start on port none", "stop on port none"].iter().map(|note| note.to_string()).collect(), "{report:#}");
    assert!(reads_of(&report, API).is_empty(), "{report:#}");
}

// behavior-contexts-in-process: The adapter does not look at the address of a process that the code of the tests starts
#[test]
fn the_adapter_does_not_look_at_the_address_of_a_process_that_the_code_of_the_tests_starts() {
    // The website of the tests is served at a port that something already holds. The adapter has no
    // address for it, because no runner configuration names one, so it must run the tests as usual
    // and touch nothing there.
    let holder = TcpListener::bind("127.0.0.1:0").unwrap();
    holder.set_nonblocking(true).unwrap();
    let port = holder.local_addr().unwrap().port().to_string();
    let repo = sample_with_adapter();
    let report = launch_with(&repo, PROCESS, &[("PAYOUTS_WEBSITE_PORT", &port)]);
    assert_eq!(outcome_of(&report, "A_website_started_by_the_tests_gives_a_start_and_a_stop_read"), "passed", "{report:#}");
    for entry in report["components"].as_array().cloned().unwrap_or_default() {
        assert!(entry.get("error").is_none(), "a component has an error: {entry:#}");
    }
    assert_eq!(reads_of(&report, WEBSITE).len(), 2, "{report:#}");
    let mut visits = 0;
    while holder.accept().is_ok() {
        visits += 1;
    }
    assert_eq!(visits, 0, "something connected to the held port during the launch");
}

// behavior-contexts-in-process: The xUnit adapter reports no "something else answers there" for a host inside the test process
#[test]
fn the_xunit_adapter_reports_no_something_else_answers_there_for_a_host_inside_the_test_process() {
    let repo = sample_with_adapter();
    let output = start_adapter(&repo, &run_request(&[PAIR], &BOTH), &[]);
    output.expect_status(0);
    let report = output.json();
    assert!(!report.to_string().contains("something else answers there"), "{report:#}");
    assert!(!output.stderr.contains("something else answers there"), "{}", output.stderr);
    assert_eq!(outcome_of(&report, "A_host_of_api_gives_a_built_and_a_disposing_read"), "passed", "{report:#}");
    assert_eq!(phases_of(&reads_of(&report, API)), strs(&["built", "disposing"]), "{report:#}");
}
