//! The rows of group "The .NET test adapter" in docs/increments/1.md that state what the core
//! does with any adapter. They run against the text-suite adapter, so they need no .NET SDK.

use azimuth_tests::*;

// behavior: The core starts the adapter's command with no arguments and writes one request document to its standard input
#[test]
fn the_core_starts_the_adapter_without_arguments() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    let starts = repo.adapter_starts();
    assert_eq!(starts.len(), 1, "{starts:#?}");
    assert_eq!(starts[0]["args"], json!([]), "the adapter got arguments: {:#}", starts[0]);
    assert!(starts[0].get("error").is_none(), "standard input was not one JSON document: {:#}", starts[0]);
    let request = &starts[0]["request"];
    assert_eq!(request["verb"], "run");
    assert_eq!(request["suite"], "suites/desk-checks");
    assert_eq!(request["marker_key"], "Check");
    let mut checks = strings_of(&request["checks"]);
    checks.sort();
    let mut expected: Vec<String> = ALL_CHECKS.iter().map(|id| id.to_string()).collect();
    expected.sort();
    assert_eq!(checks, expected);
}

// behavior: The request is `adapter-request-1` and the report `adapter-report-1`
#[test]
fn the_core_sends_an_adapter_request_1() {
    let repo = Repo::sample();
    repo.run_json(&[], 0);
    assert_eq!(repo.adapter_runs()[0]["request"]["format"], "adapter-request-1");
}

// behavior: An upgrade of the adapter makes every Check owed
#[test]
fn a_new_adapter_version_makes_the_checks_owed() {
    let mut repo = Repo::sample();
    repo.run_json(&[], 0);
    repo.set_env("TEXT_ADAPTER_VERSION", "1.1.0");
    // The core learns an adapter's version from its report, so one launch must happen first.
    repo.azimuth(&["check", "run", SIXTH_LOAN, "--json"]).expect_status(0);
    let status = repo.status_json();
    let checks = status_checks(&status);
    for check in [RETURNED_BOOK, LATE_RETURN, ON_TIME_RETURN] {
        assert_eq!(checks[check]["state"], "owed", "{status:#}");
    }
    repo.clear_log();
    let document = repo.run_json(&[], 0);
    for check in [RETURNED_BOOK, LATE_RETURN, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
}

// behavior: The core treats any other status, or output that is not one report, as a crashed launch
#[test]
fn a_bad_status_or_a_bad_report_is_a_crashed_launch() {
    for directive in ["adapter: crash", "adapter: garbage", "adapter: two-reports"] {
        let repo = Repo::sample();
        repo.append(LATE_FINE_TESTS, &format!("{directive}\n"));
        let document = repo.azimuth(&["run", "--json"]).json();
        assert_ne!(document["suites"][0]["launch"], "ok", "{directive}: {document:#}");
        for check in ALL_CHECKS {
            assert_eq!(state_of(&document, check), "owed", "{directive}: {document:#}");
            assert!(repo.records_naming(check).is_empty(), "{directive}: a result was written for {check}");
        }
    }
}
