//! The rows of `docs/increments/contexts-launches-through-playwright.md`: how the Playwright
//! adapter takes the addresses of components from the runner configuration, looks at them before it
//! starts anything, reads each component's report right after the servers are up and again before
//! they stop, wraps the suite's own setup and teardown, carries what could not be read as a cause,
//! and passes `AZIMUTH_CONTEXT` to the runner.
//!
//! These tests start the real adapter against the sample's suite `playwright/components`, with the
//! real runner and the locally packed adapter. The suite's components are one small server started
//! once per `webServer` entry, whose behavior the configuration sets (see `components.config.cjs`
//! in the sample); what the servers and the suite's own setup, test and teardown did is read from a
//! journal they all append to, so the order of events is the order of its lines. That code stands
//! for a team's suite; no row is about it. Each scenario below is launched once and several tests
//! read its outcome, because the rows that share a launch are shown by the same operation.
//!
//! The design gives the fields of a read (`phase`, and `report` or `error`), the place of the
//! entries (`components` of the report, each with the expected `name` and `reads`), the two error
//! texts "something else answers there" and "no address in the runner configuration", and the text
//! of a stop read that was not made. It does not give the words of any other error, whether a
//! component with no reads has an empty `reads` or none, the order of the reads of one component, or
//! what the report's tests say when the runner was killed, so a test asserts only what the design
//! gives and looks for what an error says (a status, a component's name) in its whole text.
//!
//! They need the real runner from npm, and run only with `cargo test --features playwright`.

use std::net::TcpListener;
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use azimuth_tests::*;
use serde_json::Value;

#[allow(dead_code)]
#[path = "support/http.rs"]
mod http;
#[path = "support/playwright_components.rs"]
mod host;
use host::PACKAGE;

const SUITE: &str = "playwright/components";
const CHECK: &str = "payout-is-sent-at-once";
const API: &str = "api";
const WEBSITE: &str = "website";
const SEARCH: &str = "search";
const API_PLACE: &str = "/internal/azimuth";
const WEBSITE_PLACE: &str = "/azimuth/context";
const SEARCH_PLACE: &str = "/search/report";
const ANSWERS: &str = "something else answers there";
const NO_ADDRESS: &str = "no address in the runner configuration";
const NOT_MADE: &str = "the stop read was not made";

/// The outcome of one launch of the adapter and the journal the suite's processes left.
struct Launched {
    output: Output,
    journal: Vec<Value>,
    seconds: u64,
    ports: Vec<u16>,
}

fn free_ports(count: usize) -> Vec<u16> {
    let held: Vec<TcpListener> = (0..count).map(|_| TcpListener::bind(("127.0.0.1", 0)).unwrap()).collect();
    held.iter().map(|listener| listener.local_addr().unwrap().port()).collect()
}

/// An entry the configuration turns into a `webServer` entry: with `url` when it has one, with
/// `port` alone otherwise, and with no `name` when it has none.
fn entry(name: Option<&str>, port: u16, url: Option<String>, report: &str, mode: &str) -> Value {
    json!({ "name": name, "port": port, "url": url, "report": report, "mode": mode })
}

fn at_url(name: &str, port: u16, report: &str) -> Value {
    entry(Some(name), port, Some(format!("http://127.0.0.1:{port}")), report, "ok")
}

fn listed(names: &[(&str, &str)]) -> Vec<(String, String)> {
    names.iter().map(|(name, place)| (name.to_string(), place.to_string())).collect()
}

fn launch(entries: Vec<Value>, later: Option<Vec<Value>>, components: &[(&str, &str)], condition: &Value, env: &[(&str, &str)]) -> Launched {
    launch_config("components.config.cjs", entries, later, components, condition, env)
}

fn launch_config(config: &str, entries: Vec<Value>, later: Option<Vec<Value>>, components: &[(&str, &str)], condition: &Value, env: &[(&str, &str)]) -> Launched {
    let mut repo = host::sample();
    let journal_path = repo.beside("components-journal.jsonl");
    let evaluations = repo.beside("evaluations.txt");
    let request = json!({
        "format": "adapter-request-1",
        "verb": "run",
        "suite": SUITE,
        "marker_key": "Check",
        "checks": [CHECK],
        "condition": condition,
        "components": listed(components).iter().map(|(name, report)| json!({ "name": name, "report": report })).collect::<Vec<_>>(),
    });
    let mut variables = vec![
        ("SUITE_JOURNAL".to_string(), journal_path.to_str().unwrap().to_string()),
        ("EVALUATION_FILE".to_string(), evaluations.to_str().unwrap().to_string()),
        ("COMPONENT_SERVERS".to_string(), Value::Array(entries.clone()).to_string()),
        ("AZIMUTH_CONTEXT".to_string(), condition.to_string()),
    ];
    if let Some(later) = later {
        variables.push(("COMPONENT_SERVERS_LATER".to_string(), Value::Array(later).to_string()));
    }
    variables.extend(env.iter().map(|(name, value)| (name.to_string(), value.to_string())));
    for (name, value) in &variables {
        repo.set_env(name, value);
    }
    let command = repo.azimuth_command(&[]);
    let mut adapter = Command::new("npx");
    adapter
        .env_clear()
        .envs(command.get_envs().filter_map(|(name, value)| value.map(|value| (name, value))))
        .args(["--no", PACKAGE, "--", "--config", config])
        .current_dir(repo.path(SUITE));
    let started = Instant::now();
    let output = run_with_time_limit(adapter, Duration::from_secs(120), Some(&request.to_string()));
    let seconds = started.elapsed().as_secs();
    let journal = std::fs::read_to_string(&journal_path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|error| panic!("the journal has a line that is not JSON ({error}): {line}")))
        .collect();
    let ports = entries.iter().filter_map(|entry| entry["port"].as_u64()).map(|port| port as u16).collect();
    Launched { output, journal, seconds, ports }
}

impl Launched {
    /// The adapter's report. Its absence is what the test is about, so it is an assertion.
    fn report(&self) -> Value {
        let report = serde_json::from_str::<Value>(&self.output.stdout)
            .ok()
            .filter(|report| report["format"] == "adapter-report-1");
        report.unwrap_or_else(|| panic!("the adapter wrote no report (status {}):\n{}\njournal: {:#?}", self.output.status, self.output.text(), self.journal))
    }

    fn text(&self) -> String {
        self.output.text()
    }

    fn lines(&self, source: &str, event: &str) -> Vec<(usize, &Value)> {
        self.journal.iter().enumerate().filter(|(_, line)| line["source"] == source && line["event"] == event).collect()
    }

    fn server_reads(&self, name: &str) -> Vec<(usize, &Value)> {
        self.lines("server", "read").into_iter().filter(|(_, line)| line["name"] == name).collect()
    }

    fn first(&self, source: &str, event: &str) -> usize {
        self.lines(source, event).first().map(|(index, _)| *index).unwrap_or_else(|| panic!("the journal has no {source} {event}:\n{:#?}", self.journal))
    }

    fn last_of(&self, source: &str, event: &str) -> usize {
        self.lines(source, event).last().map(|(index, _)| *index).unwrap_or_else(|| panic!("the journal has no {source} {event}:\n{:#?}", self.journal))
    }

    /// The servers and the suite's own setup, test and teardown that ran.
    fn started_anything(&self) -> bool {
        self.journal.iter().any(|line| line["source"] == "server" || ["setup", "test", "teardown"].contains(&line["event"].as_str().unwrap_or("")))
    }

    fn assert_nothing_started(&self) {
        assert!(!self.started_anything(), "the adapter started something:\n{:#?}", self.journal);
    }
}

fn entry_of(report: &Value, name: &str) -> Value {
    let entries = report["components"].as_array().unwrap_or_else(|| panic!("the report has no `components`: {report:#}"));
    let found: Vec<&Value> = entries.iter().filter(|entry| entry["name"] == name).collect();
    assert_eq!(found.len(), 1, "the report does not list {name} once: {report:#}");
    found[0].clone()
}

fn reads_in(entry: &Value) -> Vec<Value> {
    entry["reads"].as_array().cloned().unwrap_or_default()
}

fn read_in_phase(entry: &Value, phase: &str) -> Value {
    let found: Vec<Value> = reads_in(entry).into_iter().filter(|read| read["phase"] == phase).collect();
    assert_eq!(found.len(), 1, "{} has not one read of the phase {phase}: {entry:#}", entry["name"]);
    found[0].clone()
}

fn names_of(report: &Value) -> Vec<String> {
    let mut names: Vec<String> = report["components"].as_array().unwrap_or(&Vec::new()).iter().filter_map(|entry| entry["name"].as_str()).map(String::from).collect();
    names.sort();
    names
}

fn host_port(line: &Value) -> String {
    line["host"].as_str().unwrap_or_default().to_string()
}

fn idle() -> Value {
    json!({})
}

fn instant() -> Value {
    json!({ "payouts.instant": "true" })
}

fn both_listed() -> [(&'static str, &'static str); 2] {
    [(API, API_PLACE), (WEBSITE, WEBSITE_PLACE)]
}

/// Two components that read normally, a third entry the request does not list, no condition. `api`
/// gives a `url` with a path and a query, `website` gives only `port`.
fn ordinary() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(3);
        let entries = vec![
            entry(Some(API), ports[0], Some(format!("http://127.0.0.1:{}/health?probe=1", ports[0])), API_PLACE, "ok"),
            entry(Some(WEBSITE), ports[1], None, WEBSITE_PLACE, "ok"),
            at_url(SEARCH, ports[2], SEARCH_PLACE),
        ];
        launch(entries, None, &both_listed(), &idle(), &[])
    })
}

/// The same two components with a condition, which the runner receives as `AZIMUTH_CONTEXT`.
fn with_condition() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        launch(vec![at_url(API, ports[0], API_PLACE), at_url(WEBSITE, ports[1], WEBSITE_PLACE)], None, &both_listed(), &instant(), &[])
    })
}

/// Four components whose reads fail in four ways: a status other than 200, a connection the
/// component closes, a body that is not JSON, and no answer at all.
fn failing_reads() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(4);
        let entries = vec![
            entry(Some("ledger"), ports[0], Some(format!("http://127.0.0.1:{}", ports[0])), "/ledger/report", "status"),
            entry(Some("vault"), ports[1], Some(format!("http://127.0.0.1:{}", ports[1])), "/vault/report", "closed"),
            entry(Some("mailer"), ports[2], Some(format!("http://127.0.0.1:{}", ports[2])), "/mailer/report", "not-json"),
            entry(Some("audit"), ports[3], Some(format!("http://127.0.0.1:{}", ports[3])), "/audit/report", "silent"),
        ];
        let components = [("ledger", "/ledger/report"), ("vault", "/vault/report"), ("mailer", "/mailer/report"), ("audit", "/audit/report")];
        launch(entries, None, &components, &idle(), &[])
    })
}

/// A listener that something else started answers at `api`'s address; `website`'s is free.
fn api_answered() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        let _other = TcpListener::bind(("127.0.0.1", ports[0])).expect("the port was free a moment ago");
        launch(vec![at_url(API, ports[0], API_PLACE), at_url(WEBSITE, ports[1], WEBSITE_PLACE)], None, &both_listed(), &idle(), &[])
    })
}

/// Listeners answer at the addresses of both components.
fn both_answered() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        let _first = TcpListener::bind(("127.0.0.1", ports[0])).expect("the port was free a moment ago");
        let _second = TcpListener::bind(("127.0.0.1", ports[1])).expect("the port was free a moment ago");
        launch(vec![at_url(API, ports[0], API_PLACE), at_url(WEBSITE, ports[1], WEBSITE_PLACE)], None, &both_listed(), &idle(), &[])
    })
}

/// The entry names `localhost`; the listener is on `127.0.0.1` or on `::1`, never both.
fn localhost_answered(on_ipv6: bool) -> Option<&'static Launched> {
    static ON_IPV4: OnceLock<Launched> = OnceLock::new();
    static ON_IPV6: OnceLock<Option<Launched>> = OnceLock::new();
    let named = |port: u16| entry(Some(API), port, Some(format!("http://localhost:{port}")), API_PLACE, "ok");
    if on_ipv6 {
        ON_IPV6
            .get_or_init(|| {
                let port = free_ports(1)[0];
                let Ok(_other) = TcpListener::bind(("::1", port)) else {
                    eprintln!("::1 is not available on this machine; the row of ::1 is not shown");
                    return None;
                };
                Some(launch(vec![named(port)], None, &[(API, API_PLACE)], &idle(), &[]))
            })
            .as_ref()
    } else {
        Some(ON_IPV4.get_or_init(|| {
            let port = free_ports(1)[0];
            let _other = TcpListener::bind(("127.0.0.1", port)).expect("the port was free a moment ago");
            launch(vec![named(port)], None, &[(API, API_PLACE)], &idle(), &[])
        }))
    }
}

/// The entry of `api` gives neither `url` nor `port`: the free port is chosen by the server after
/// the start.
fn learned_after_start() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let mut chosen_later = entry(Some(API), 0, None, API_PLACE, "ok");
        chosen_later["port"] = Value::Null;
        launch(vec![chosen_later], None, &[(API, API_PLACE)], &idle(), &[])
    })
}

/// Something answers at an address that only the environment names, not the configuration.
fn unnamed_address() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        let other = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let elsewhere = format!("http://127.0.0.1:{}", other.local_addr().unwrap().port());
        launch(vec![at_url(API, ports[0], API_PLACE)], None, &[(API, API_PLACE)], &idle(), &[("API_ADDRESS", &elsewhere), ("API_URL", &elsewhere)])
    })
}

/// Two entries carry the name of one component.
fn two_entries_named_alike() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        launch(vec![at_url(API, ports[0], API_PLACE), at_url(API, ports[1], API_PLACE)], None, &[(API, API_PLACE)], &idle(), &[])
    })
}

/// The configuration gives `api` another port the second time it is evaluated.
fn address_changes() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        launch(vec![at_url(API, ports[0], API_PLACE)], Some(vec![at_url(API, ports[1], API_PLACE)]), &[(API, API_PLACE)], &idle(), &[])
    })
}

/// `api` reads normally, `website` answers with a status other than 200, `search` has no entry, and
/// the runner is killed by the only test, before its teardown.
fn every_cause() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        let entries = vec![
            at_url(API, ports[0], API_PLACE),
            entry(Some(WEBSITE), ports[1], Some(format!("http://127.0.0.1:{}", ports[1])), WEBSITE_PLACE, "status"),
        ];
        launch(entries, None, &[(API, API_PLACE), (WEBSITE, WEBSITE_PLACE), (SEARCH, SEARCH_PLACE)], &idle(), &[("KILL_RUNNER", "yes")])
    })
}

/// The suite's own global setup throws.
fn setup_fails() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(1);
        launch(vec![at_url(API, ports[0], API_PLACE)], None, &[(API, API_PLACE)], &idle(), &[("FAIL_SETUP", "yes")])
    })
}

fn port_of_host(line: &Value) -> u16 {
    host_port(line).rsplit(':').next().and_then(|port| port.parse().ok()).unwrap_or(0)
}

fn answer_check(launched: &Launched, name: &str) -> Value {
    let entry = entry_of(&launched.report(), name);
    assert_eq!(entry["error"], ANSWERS, "{entry:#}");
    entry
}

// ---- Where the adapter takes what it needs ----

// behavior-contexts-launches-through-playwright: The address of a component comes from the runner configuration
// behavior-contexts-launches-through-playwright: The name of a component comes from the request
// behavior-contexts-launches-through-playwright: The report path of a component comes from the request
#[test]
fn a_component_is_read_at_the_configurations_address_under_the_requests_name_and_path() {
    let launched = ordinary();
    let report = launched.report();
    let api_reads = launched.server_reads(API);
    let website_reads = launched.server_reads(WEBSITE);
    assert_eq!(api_reads.len(), 2, "{:#?}", launched.journal);
    assert_eq!(website_reads.len(), 2, "{:#?}", launched.journal);
    assert!(api_reads.iter().all(|(_, line)| port_of_host(line) == launched.ports[0] && line["path"] == API_PLACE), "{:#?}", launched.journal);
    assert!(website_reads.iter().all(|(_, line)| port_of_host(line) == launched.ports[1] && line["path"] == WEBSITE_PLACE), "{:#?}", launched.journal);
    assert_eq!(reads_in(&entry_of(&report, API)).len(), 2, "{report:#}");
    assert_eq!(reads_in(&entry_of(&report, WEBSITE)).len(), 2, "{report:#}");
}

// behavior-contexts-launches-through-playwright: The adapter finds the addresses of components in the `webServer` entries of the config
#[test]
fn the_addresses_are_the_ones_of_the_web_server_entries() {
    let launched = ordinary();
    let report = launched.report();
    let ports: Vec<u16> = [API, WEBSITE].iter().flat_map(|name| launched.server_reads(name)).map(|(_, line)| port_of_host(line)).collect();
    assert_eq!(ports.len(), 4, "{:#?}", launched.journal);
    assert!(ports.iter().all(|port| launched.ports[..2].contains(port)), "{:#?}", launched.journal);
    assert!(read_in_phase(&entry_of(&report, API), "start")["report"].is_object(), "{report:#}");
}

// behavior-contexts-launches-through-playwright: The entry of a component is the `webServer` entry whose `name` is the component's name
#[test]
fn a_component_is_read_at_the_entry_that_carries_its_name() {
    let launched = ordinary();
    let report = launched.report();
    for (name, port) in [(API, launched.ports[0]), (WEBSITE, launched.ports[1])] {
        let reads = launched.server_reads(name);
        assert_eq!(reads.len(), 2, "{name}: {:#?}", launched.journal);
        assert!(reads.iter().all(|(_, line)| port_of_host(line) == port), "{name}: {:#?}", launched.journal);
        assert_eq!(read_in_phase(&entry_of(&report, name), "start")["report"]["component"], name, "{report:#}");
    }
    assert!(launched.server_reads(SEARCH).is_empty(), "an entry of another name was read: {:#?}", launched.journal);
}

// behavior-contexts-launches-through-playwright: The address of a component is the origin of the `url` of its entry
#[test]
fn the_address_is_the_origin_of_the_entrys_url() {
    let launched = ordinary();
    launched.report();
    let reads = launched.server_reads(API);
    assert_eq!(reads.len(), 2, "{:#?}", launched.journal);
    for (_, line) in reads {
        assert_eq!(line["host"], format!("127.0.0.1:{}", launched.ports[0]), "{line:#}");
        assert_eq!(line["path"], API_PLACE, "the report path was put after the url's own path or query: {line:#}");
    }
}

// behavior-contexts-launches-through-playwright: An entry that gives only `port` has the address `http://localhost:<port>`
#[test]
fn an_entry_with_only_a_port_is_read_at_localhost() {
    let launched = ordinary();
    launched.report();
    let reads = launched.server_reads(WEBSITE);
    assert_eq!(reads.len(), 2, "{:#?}", launched.journal);
    for (_, line) in reads {
        assert_eq!(line["host"], format!("localhost:{}", launched.ports[1]), "{line:#}");
    }
}

// behavior-contexts-launches-through-playwright: A config in which two entries have the name of one component is refused, and the adapter runs nothing
#[test]
fn two_entries_of_one_name_run_nothing() {
    let launched = two_entries_named_alike();
    launched.assert_nothing_started();
}

// behavior-contexts-launches-through-playwright: The adapter says why it refused such a config
#[test]
fn the_refusal_of_two_entries_of_one_name_says_why() {
    let launched = two_entries_named_alike();
    let said = launched.text();
    assert!(said.contains(API), "the refusal does not name the component:\n{said}");
    assert!(!said.trim().is_empty(), "the adapter said nothing");
}

// behavior-contexts-launches-through-playwright: The Checks of a launch with such a config get no result
#[test]
fn two_entries_of_one_name_give_no_result() {
    let launched = two_entries_named_alike();
    assert_ne!(launched.output.status, 0, "{}", launched.text());
    assert!(!launched.output.stdout.contains("adapter-report-1"), "the adapter wrote a report:\n{}", launched.output.stdout);
}

// behavior-contexts-launches-through-playwright: An address that the runner configuration does not name is not looked at
#[test]
fn an_address_only_the_environment_names_is_not_looked_at() {
    let launched = unnamed_address();
    let report = launched.report();
    let api = entry_of(&report, API);
    assert!(api.get("error").is_none(), "{api:#}");
    assert_eq!(reads_in(&api).len(), 2, "{api:#}");
    assert_eq!(launched.lines("suite", "test").len(), 1, "{:#?}", launched.journal);
}

// behavior-contexts-launches-through-playwright: An address must be the same at every evaluation of the config
#[test]
fn an_address_that_changes_between_evaluations_is_refused() {
    let launched = address_changes();
    launched.assert_nothing_started();
    assert!(!launched.output.stdout.contains("adapter-report-1"), "the adapter wrote a report:\n{}", launched.output.stdout);
}

// behavior-contexts-launches-through-playwright: The adapter evaluates the resolved config twice, each time in a fresh process
#[test]
fn the_config_is_evaluated_twice_in_fresh_processes_before_anything_starts() {
    let launched = ordinary();
    launched.report();
    let first_start = launched.first("server", "listening");
    let before: Vec<&Value> = launched.lines("suite", "config").into_iter().filter(|(index, _)| *index < first_start).map(|(_, line)| line).collect();
    assert!(before.len() >= 2, "fewer than two evaluations before the start: {:#?}", launched.journal);
    assert!(before.iter().take(2).all(|line| !line["argv1"].as_str().unwrap_or_default().contains("azimuth-playwright")), "an evaluation ran in the adapter's own process: {before:#?}");
    assert_ne!(before[0]["pid"], before[1]["pid"], "{before:#?}");
}

// behavior-contexts-launches-through-playwright: A config that gives a component a different address the second time is refused, and the adapter runs nothing
#[test]
fn a_config_that_gives_another_address_the_second_time_runs_nothing() {
    let launched = address_changes();
    launched.assert_nothing_started();
    assert_ne!(launched.output.status, 0, "{}", launched.text());
    assert!(launched.text().contains(API), "the refusal does not name the component:\n{}", launched.text());
}

// ---- The look before the start ----

// behavior-contexts-launches-through-playwright: The adapter looks at each address of a component in `starts` before it starts anything
#[test]
fn nothing_is_started_where_an_address_answers() {
    let launched = api_answered();
    launched.report();
    launched.assert_nothing_started();
}

// behavior-contexts-launches-through-playwright: The look happens in a launch with no condition too
#[test]
fn the_look_happens_in_a_launch_with_no_condition() {
    let launched = api_answered();
    answer_check(launched, API);
}

// behavior-contexts-launches-through-playwright: An address answers when a TCP connection to its host and port is accepted
#[test]
fn an_address_that_accepts_a_connection_but_speaks_no_http_answers() {
    // The listener of the scenario accepts connections in the kernel and never says a word.
    answer_check(api_answered(), API);
}

// behavior-contexts-launches-through-playwright: When an address answers, no test runs
#[test]
fn no_test_runs_where_an_address_answers() {
    let launched = api_answered();
    launched.report();
    assert!(launched.lines("suite", "test").is_empty(), "{:#?}", launched.journal);
}

// behavior-contexts-launches-through-playwright: When an address answers, the report lists the tests as `not_run`
#[test]
fn the_tests_are_not_run_where_an_address_answers() {
    let report = api_answered().report();
    let tests = report["tests"].as_array().unwrap_or_else(|| panic!("the report has no tests: {report:#}"));
    assert!(!tests.is_empty(), "{report:#}");
    assert!(tests.iter().all(|test| test["outcome"] == "not_run"), "{report:#}");
}

// behavior-contexts-launches-through-playwright: When an address answers, the report gives "something else answers there" for that component
#[test]
fn the_report_says_something_else_answers_there() {
    let report = api_answered().report();
    assert!(report.to_string().contains(ANSWERS), "{report:#}");
}

// behavior-contexts-launches-through-playwright: The component that answered has `error: "something else answers there"` and no reads
#[test]
fn the_component_that_answered_has_that_error_and_no_reads() {
    let entry = answer_check(api_answered(), API);
    assert!(reads_in(&entry).is_empty(), "{entry:#}");
}

// behavior-contexts-launches-through-playwright: The look covers every address before the adapter decides
// behavior-contexts-launches-through-playwright: Each component whose address answers has the error "something else answers there"
#[test]
fn every_component_whose_address_answers_has_the_error() {
    let launched = both_answered();
    answer_check(launched, API);
    answer_check(launched, WEBSITE);
}

// behavior-contexts-launches-through-playwright: When an address answers, the adapter starts nothing
#[test]
fn the_adapter_starts_nothing_where_an_address_answers() {
    let launched = both_answered();
    launched.report();
    launched.assert_nothing_started();
}

// behavior-contexts-launches-through-playwright: Every component whose address does not answer is listed with no reads
#[test]
fn a_component_whose_address_is_free_is_listed_with_no_reads() {
    let report = api_answered().report();
    let website = entry_of(&report, WEBSITE);
    assert!(reads_in(&website).is_empty(), "{website:#}");
}

// behavior-contexts-launches-through-playwright: For `localhost` an answer on `127.0.0.1` is found
#[test]
fn localhost_is_found_answering_on_ipv4() {
    answer_check(localhost_answered(false).unwrap(), API);
}

// behavior-contexts-launches-through-playwright: For `localhost` an answer on `::1` is found
#[test]
fn localhost_is_found_answering_on_ipv6() {
    if let Some(launched) = localhost_answered(true) {
        answer_check(launched, API);
    }
}

// behavior-contexts-launches-through-playwright: An address learned only after the start, such as a free port chosen then, is not looked at
#[test]
fn an_address_learned_after_the_start_is_not_looked_at() {
    let launched = learned_after_start();
    assert_eq!(launched.lines("suite", "test").len(), 1, "the tests did not run:\n{:#?}\n{}", launched.journal, launched.text());
    let report = launched.report();
    assert_ne!(entry_of(&report, API)["error"], ANSWERS, "{report:#}");
}

// ---- The two reads ----

// behavior-contexts-launches-through-playwright: The adapter reads a component's report after the servers start and before the tests
#[test]
fn the_start_read_comes_after_the_servers_start_and_before_the_tests() {
    let launched = ordinary();
    launched.report();
    for name in [API, WEBSITE] {
        let started = launched.lines("server", "listening").into_iter().find(|(_, line)| line["name"] == name).map(|(index, _)| index).unwrap();
        let first_read = launched.server_reads(name).first().map(|(index, _)| *index).expect("the component was never read, so there is no read to place");
        assert!(started < first_read && first_read < launched.first("suite", "test"), "{name}: {:#?}", launched.journal);
    }
}

// behavior-contexts-launches-through-playwright: The adapter reads a component's report after the tests and before the servers stop
#[test]
fn the_stop_read_comes_after_the_tests_and_before_the_servers_stop() {
    let launched = ordinary();
    launched.report();
    for name in [API, WEBSITE] {
        let last_read = launched.server_reads(name).last().map(|(index, _)| *index).expect("the component was never read, so there is no read to place");
        let stopped = launched.lines("server", "stopped").into_iter().find(|(_, line)| line["name"] == name).map(|(index, _)| index).expect("the server did not stop");
        assert!(launched.last_of("suite", "test") < last_read && last_read < stopped, "{name}: {:#?}", launched.journal);
    }
}

// behavior-contexts-launches-through-playwright: A component in its own process gets a read with the phase `start`
#[test]
fn a_component_in_its_own_process_gets_a_start_read() {
    let report = ordinary().report();
    for name in [API, WEBSITE] {
        assert!(read_in_phase(&entry_of(&report, name), "start")["report"].is_object(), "{report:#}");
    }
}

// behavior-contexts-launches-through-playwright: A component in its own process gets a read with the phase `stop`
#[test]
fn a_component_in_its_own_process_gets_a_stop_read() {
    let report = ordinary().report();
    for name in [API, WEBSITE] {
        assert!(read_in_phase(&entry_of(&report, name), "stop")["report"].is_object(), "{report:#}");
    }
}

// behavior-contexts-launches-through-playwright: A read is one HTTP GET of the component's `report` path at its address
#[test]
fn a_read_is_one_get_of_the_report_path() {
    let launched = ordinary();
    let report = launched.report();
    for (name, place) in both_listed() {
        let reads = launched.server_reads(name);
        assert_eq!(reads.len(), reads_in(&entry_of(&report, name)).len(), "one request for each read of {name}: {:#?}", launched.journal);
        assert!(!reads.is_empty(), "{:#?}", launched.journal);
        assert!(reads.iter().all(|(_, line)| line["method"] == "GET" && line["path"] == place), "{name}: {:#?}", launched.journal);
    }
}

// behavior-contexts-launches-through-playwright: The body of a 200 response is the document as read
#[test]
fn the_body_of_a_200_response_is_the_document_as_read() {
    let launched = ordinary();
    let report = launched.report();
    for name in [API, WEBSITE] {
        let served: Vec<Value> = launched.server_reads(name).iter().map(|(_, line)| serde_json::from_str(line["body"].as_str().unwrap()).unwrap()).collect();
        let entry = entry_of(&report, name);
        for (phase, document) in [("start", &served[0]), ("stop", &served[1])] {
            assert_eq!(&read_in_phase(&entry, phase)["report"], document, "{name} {phase}: {report:#}");
        }
    }
}

// behavior-contexts-launches-through-playwright: A read that succeeded has `report`, the document as read
#[test]
fn a_read_that_succeeded_has_a_report_and_no_error() {
    let report = ordinary().report();
    for name in [API, WEBSITE] {
        let reads = reads_in(&entry_of(&report, name));
        assert_eq!(reads.len(), 2, "{report:#}");
        assert!(reads.iter().all(|read| read["report"].is_object() && read.get("error").is_none()), "{report:#}");
    }
}

fn failed(name: &str) -> Vec<Value> {
    let launched = failing_reads();
    let reads = reads_in(&entry_of(&launched.report(), name));
    assert!(!reads.is_empty(), "{name} has no reads: {:#}", launched.report());
    assert!(reads.iter().all(|read| read["error"].as_str().is_some_and(|error| !error.is_empty()) && read.get("report").is_none()), "{name}: {reads:#?}");
    reads
}

// behavior-contexts-launches-through-playwright: A response with a status other than 200 is a failed read whose `error` says which
#[test]
fn a_status_other_than_200_is_a_failed_read_that_says_which() {
    for read in failed("ledger") {
        assert!(read["error"].as_str().unwrap().contains("503"), "{read:#}");
    }
}

// behavior-contexts-launches-through-playwright: A failed connection is a failed read whose `error` says which
#[test]
fn a_failed_connection_is_a_failed_read_that_says_which() {
    let reads = failed("vault");
    let other = failed("ledger");
    assert_ne!(reads[0]["error"], other[0]["error"], "the error does not say which kind of failure it was");
}

// behavior-contexts-launches-through-playwright: A body that is not JSON is a failed read whose `error` says which
#[test]
fn a_body_that_is_not_json_is_a_failed_read_that_says_which() {
    let reads = failed("mailer");
    let other = failed("vault");
    assert_ne!(reads[0]["error"], other[0]["error"], "the error does not say which kind of failure it was");
}

// behavior-contexts-launches-through-playwright: No response within 10 seconds is a failed read whose `error` says which
#[test]
fn no_response_within_ten_seconds_is_a_failed_read_that_says_which() {
    let launched = failing_reads();
    let reads = failed("audit");
    assert_ne!(reads[0]["error"], failed("mailer")[0]["error"], "the error does not say which kind of failure it was");
    assert!(launched.seconds >= 10, "the silent component was not waited for ({} seconds)", launched.seconds);
}

// behavior-contexts-launches-through-playwright: A read that failed has `error`, the cause it could not be read
#[test]
fn a_read_that_failed_has_an_error_and_no_report() {
    for name in ["ledger", "vault", "mailer", "audit"] {
        failed(name);
    }
}

// behavior-contexts-launches-through-playwright: The adapter reads every component that the request lists
#[test]
fn every_component_the_request_lists_is_read() {
    let launched = with_condition();
    let report = launched.report();
    for (name, _) in both_listed() {
        assert_eq!(reads_in(&entry_of(&report, name)).len(), 2, "{report:#}");
    }
}

// behavior-contexts-launches-through-playwright: It reads them in a launch with no condition too
#[test]
fn the_components_are_read_in_a_launch_with_no_condition() {
    let launched = ordinary();
    let report = launched.report();
    for (name, _) in both_listed() {
        assert_eq!(reads_in(&entry_of(&report, name)).len(), 2, "{report:#}");
    }
}

// behavior-contexts-launches-through-playwright: The report lists each component collected under the name that the request gave
#[test]
fn the_report_lists_each_component_under_the_requests_name() {
    let report = ordinary().report();
    assert_eq!(names_of(&report), vec![API.to_string(), WEBSITE.to_string()], "{report:#}");
}

// behavior-contexts-launches-through-playwright: A component report that is malformed is a read error, not a new outcome of a test
#[test]
fn a_malformed_component_report_is_a_read_error_and_changes_no_outcome() {
    let launched = failing_reads();
    let report = launched.report();
    let mailer = entry_of(&report, "mailer");
    assert!(reads_in(&mailer).iter().all(|read| read["error"].is_string()), "{mailer:#}");
    let tests = report["tests"].as_array().unwrap_or_else(|| panic!("the report has no tests: {report:#}"));
    assert!(!tests.is_empty() && tests.iter().all(|test| test["outcome"] == "passed"), "{report:#}");
}

// ---- Wrapping the suite's setup and teardown ----

// behavior-contexts-launches-through-playwright: The adapter wraps the runner's global setup and teardown
#[test]
fn the_reads_are_the_outermost_around_the_setup_the_tests_and_the_teardown() {
    let launched = ordinary();
    launched.report();
    for name in [API, WEBSITE] {
        let reads = launched.server_reads(name);
        assert!(!reads.is_empty(), "the component was never read: {:#?}", launched.journal);
        let (first, last) = (reads.first().unwrap().0, reads.last().unwrap().0);
        assert!(first < launched.first("suite", "setup"), "{name}: {:#?}", launched.journal);
        assert!(launched.last_of("suite", "teardown") < last, "{name}: {:#?}", launched.journal);
    }
}

// behavior-contexts-launches-through-playwright: The suite's own global setup still runs
#[test]
fn the_suites_own_global_setup_still_runs_once() {
    let launched = ordinary();
    launched.report();
    assert_eq!(launched.lines("suite", "setup").len(), 1, "{:#?}", launched.journal);
    assert_eq!(launched.server_reads(API).len(), 2, "the reads that wrap the setup were not made: {:#?}", launched.journal);
}

// behavior-contexts-launches-through-playwright: The suite's own global teardown still runs
#[test]
fn the_suites_own_global_teardown_still_runs_once() {
    let launched = ordinary();
    launched.report();
    assert_eq!(launched.lines("suite", "teardown").len(), 1, "{:#?}", launched.journal);
    assert_eq!(launched.server_reads(API).len(), 2, "the reads that wrap the teardown were not made: {:#?}", launched.journal);
}

// behavior-contexts-launches-through-playwright: The start read comes before the suite's own global setup
#[test]
fn the_start_read_comes_before_the_suites_global_setup() {
    let launched = ordinary();
    launched.report();
    for name in [API, WEBSITE] {
        let first = launched.server_reads(name).first().map(|(index, _)| *index).expect("the component was never read, so there is no read to place");
        assert!(first < launched.first("suite", "setup"), "{name}: {:#?}", launched.journal);
    }
}

// behavior-contexts-launches-through-playwright: The stop read comes after the suite's own global teardown
#[test]
fn the_stop_read_comes_after_the_suites_global_teardown() {
    let launched = ordinary();
    launched.report();
    for name in [API, WEBSITE] {
        let last = launched.server_reads(name).last().map(|(index, _)| *index).expect("the component was never read, so there is no read to place");
        assert!(launched.last_of("suite", "teardown") < last, "{name}: {:#?}", launched.journal);
    }
}

// behavior-contexts-launches-through-playwright: A failing global setup of the suite still gets the stop read
#[test]
fn a_failing_global_setup_still_gets_the_stop_read() {
    let launched = setup_fails();
    let reads = launched.server_reads(API);
    assert_eq!(reads.len(), 2, "{:#?}\n{}", launched.journal, launched.text());
    assert!(reads[0].0 < launched.first("suite", "setup") && launched.first("suite", "setup") < reads[1].0, "{:#?}", launched.journal);
}

// ---- A missing address, a failed read, a missing second read ----

// behavior-contexts-launches-through-playwright: A component in `starts` with no `webServer` entry of its name has a missing address
#[test]
fn a_component_with_no_entry_of_its_name_has_a_missing_address() {
    let report = every_cause().report();
    let search = entry_of(&report, SEARCH);
    assert!(search["error"].is_string(), "{search:#}");
}

// behavior-contexts-launches-through-playwright: A component with no address has `error: "no address in the runner configuration"` and no reads
#[test]
fn a_component_with_no_address_has_that_error_and_no_reads() {
    let report = every_cause().report();
    let search = entry_of(&report, SEARCH);
    assert_eq!(search["error"], NO_ADDRESS, "{search:#}");
    assert!(reads_in(&search).is_empty(), "{search:#}");
}

// behavior-contexts-launches-through-playwright: A stop read that was never made is a read with phase `stop` and `error: "the stop read was not made"`
// behavior-contexts-launches-through-playwright: A stop read is not made when the runner ended before its teardown
#[test]
fn a_runner_that_ended_before_its_teardown_leaves_a_stop_read_that_was_not_made() {
    let launched = every_cause();
    let report = launched.report();
    assert!(launched.lines("suite", "teardown").is_empty(), "the runner was not ended before its teardown: {:#?}", launched.journal);
    let api = entry_of(&report, API);
    assert_eq!(read_in_phase(&api, "start")["report"]["component"], API, "{api:#}");
    assert_eq!(read_in_phase(&api, "stop")["error"], NOT_MADE, "{api:#}");
}

// behavior-contexts-launches-through-playwright: A missing address, a failed read and a missing second read are not left out of the report
#[test]
fn a_missing_address_a_failed_read_and_a_missing_second_read_are_all_in_the_report() {
    let report = every_cause().report();
    assert_eq!(entry_of(&report, SEARCH)["error"], NO_ADDRESS, "{report:#}");
    let website = entry_of(&report, WEBSITE);
    let start = read_in_phase(&website, "start");
    assert!(start["error"].as_str().is_some_and(|error| error.contains("503")), "{website:#}");
    assert_eq!(read_in_phase(&website, "stop")["error"], NOT_MADE, "{website:#}");
    assert_eq!(read_in_phase(&entry_of(&report, API), "stop")["error"], NOT_MADE, "{report:#}");
}

// ---- `AZIMUTH_CONTEXT` through the adapter ----

fn context_seen_by(launched: &Launched, event: &str) -> Value {
    let line = launched.lines("suite", event).first().map(|(_, line)| (*line).clone()).unwrap_or_else(|| panic!("the journal has no {event}:\n{:#?}\n{}", launched.journal, launched.text()));
    serde_json::from_str(line["context"].as_str().unwrap_or("null")).unwrap_or_else(|error| panic!("the runner saw a context that is not JSON ({error}): {line:#}"))
}

// behavior-contexts-launches-through-playwright: The runner gets the `AZIMUTH_CONTEXT` that the adapter got
#[test]
fn the_runner_gets_the_context_the_adapter_got() {
    let launched = with_condition();
    let report = launched.report();
    assert_eq!(context_seen_by(launched, "setup"), instant(), "{:#?}", launched.journal);
    assert_eq!(context_seen_by(launched, "test"), instant(), "{:#?}", launched.journal);
    assert_eq!(read_in_phase(&entry_of(&report, API), "start")["report"]["applied"], instant(), "{report:#}");
}

// behavior-contexts-launches-through-playwright: The adapter sets no key that the condition does not state
#[test]
fn the_adapter_adds_no_key_the_condition_does_not_state() {
    let launched = with_condition();
    let report = launched.report();
    for event in ["setup", "test"] {
        let seen = context_seen_by(launched, event);
        assert_eq!(seen.as_object().unwrap().keys().collect::<Vec<_>>(), ["payouts.instant"], "{seen:#}");
    }
    for name in [API, WEBSITE] {
        for read in reads_in(&entry_of(&report, name)) {
            assert_eq!(read["report"]["applied"].as_object().unwrap().keys().collect::<Vec<_>>(), ["payouts.instant"], "{read:#}");
        }
    }
}

// ---- A config that is not CommonJS, an entry with no address, a start read not made ----

const START_NOT_MADE: &str = "the start read was not made";

/// The suite configured in TypeScript, which the runner compiles to CommonJS.
fn typescript_config() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(2);
        launch_config("components.typescript.config.ts", vec![at_url(API, ports[0], API_PLACE), at_url(WEBSITE, ports[1], WEBSITE_PLACE)], None, &both_listed(), &idle(), &[])
    })
}

/// The suite configured as an ES module, with a component listed.
fn module_config() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(1);
        launch_config("components.module.config.mjs", vec![at_url(API, ports[0], API_PLACE)], None, &[(API, API_PLACE)], &idle(), &[])
    })
}

/// A second entry ends the runner while the servers start, before the suite's global setup.
fn runner_ends_before_its_setup() -> &'static Launched {
    static LAUNCHED: OnceLock<Launched> = OnceLock::new();
    LAUNCHED.get_or_init(|| {
        let ports = free_ports(1);
        launch(vec![at_url(API, ports[0], API_PLACE), json!({ "kill_runner": true })], None, &[(API, API_PLACE)], &idle(), &[])
    })
}

// behavior-contexts-launches-through-playwright: The adapter wraps a config that the runner loads as CommonJS
#[test]
fn a_commonjs_config_is_wrapped() {
    let launched = ordinary();
    let report = launched.report();
    for name in [API, WEBSITE] {
        let reads = launched.server_reads(name);
        assert_eq!(reads.len(), 2, "{name}: {:#?}", launched.journal);
        assert!(reads[0].0 < launched.first("suite", "setup") && launched.last_of("suite", "teardown") < reads[1].0, "{name}: {:#?}", launched.journal);
        assert_eq!(reads_in(&entry_of(&report, name)).len(), 2, "{report:#}");
    }
}

// behavior-contexts-launches-through-playwright: The adapter wraps a TypeScript config that the runner compiles to CommonJS
#[test]
fn a_typescript_config_is_wrapped() {
    let launched = typescript_config();
    let report = launched.report();
    for name in [API, WEBSITE] {
        let reads = launched.server_reads(name);
        assert_eq!(reads.len(), 2, "{name}: {:#?}\n{}", launched.journal, launched.text());
        assert!(reads[0].0 < launched.first("suite", "setup") && launched.last_of("suite", "teardown") < reads[1].0, "{name}: {:#?}", launched.journal);
        let entry = entry_of(&report, name);
        assert!(read_in_phase(&entry, "start")["report"].is_object() && read_in_phase(&entry, "stop")["report"].is_object(), "{report:#}");
    }
    assert_eq!(launched.lines("suite", "setup").len(), 1, "{:#?}", launched.journal);
    assert_eq!(launched.lines("suite", "teardown").len(), 1, "{:#?}", launched.journal);
}

// behavior-contexts-launches-through-playwright: When the request lists a component, a config that the runner loads as an ES module ends the launch with a failure that says so
#[test]
fn an_es_module_config_with_a_component_ends_with_a_failure_that_says_so() {
    let launched = module_config();
    assert_ne!(launched.output.status, 0, "{}", launched.text());
    let said = launched.text().to_lowercase();
    assert!(said.contains("module"), "the failure does not say the config is an ES module:\n{said}");
    assert!(!launched.output.stdout.contains("adapter-report-1"), "{}", launched.output.stdout);
}

// behavior-contexts-launches-through-playwright: The Checks of a launch whose config loads as an ES module get no result
#[test]
fn an_es_module_config_with_a_component_gives_no_result() {
    let launched = module_config();
    assert_ne!(launched.output.status, 0, "{}", launched.text());
    assert!(!launched.output.stdout.contains("adapter-report-1"), "the adapter wrote a report:\n{}", launched.output.stdout);
}

// behavior-contexts-launches-through-playwright: A component whose entry gives neither `url` nor `port` has a missing address
#[test]
fn an_entry_with_neither_url_nor_port_has_a_missing_address() {
    let report = learned_after_start().report();
    let api = entry_of(&report, API);
    assert_eq!(api["error"], NO_ADDRESS, "{api:#}");
    assert!(reads_in(&api).is_empty(), "{api:#}");
}

// behavior-contexts-launches-through-playwright: A start read that was never made is a read with phase `start` and `error: "the start read was not made"`
#[test]
fn a_start_read_that_was_not_made_is_a_read_with_that_error() {
    let launched = runner_ends_before_its_setup();
    let report = launched.report();
    let api = entry_of(&report, API);
    assert_eq!(read_in_phase(&api, "start")["error"], START_NOT_MADE, "{api:#}");
}

// behavior-contexts-launches-through-playwright: A start read is not made when the runner ended before its global setup
#[test]
fn a_start_read_is_not_made_when_the_runner_ended_before_its_global_setup() {
    let launched = runner_ends_before_its_setup();
    let report = launched.report();
    assert!(launched.lines("suite", "setup").is_empty(), "the runner got as far as the suite's setup: {:#?}", launched.journal);
    assert!(launched.server_reads(API).is_empty(), "{:#?}", launched.journal);
    assert_eq!(read_in_phase(&entry_of(&report, API), "start")["error"], START_NOT_MADE, "{report:#}");
}
