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
fn raising_the_adapter_pin_makes_every_check_owed() {
    // A stack pins its adapter in a tracked file. The text adapter has no package manager, so a
    // plain file in the tree stands for the pin, and its version is what the adapter reports.
    let mut repo = Repo::sample_with(|root| {
        std::fs::write(root.join("adapter-pin.txt"), "text-suite-adapter 1.0.0\n").unwrap();
    });
    repo.run_json(&[], 0);
    repo.write("adapter-pin.txt", "text-suite-adapter 1.1.0\n");
    repo.set_env("TEXT_ADAPTER_VERSION", "1.1.0");
    for check in ALL_CHECKS {
        let document = repo.check_show(check);
        assert_eq!(document["state"], "owed", "{document:#}");
        assert_eq!(parts_changed(&document), vec!["files"], "{document:#}");
        assert_eq!(files_changed(&document), vec!["adapter-pin.txt".to_string()], "{document:#}");
    }
    repo.clear_log();
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
        let versions: Vec<serde_json::Value> =
            repo.results_of(check).iter().map(|(_, record)| record["body"]["method"]["adapter"]["version"].clone()).collect();
        assert!(versions.contains(&json!("1.1.0")), "{check}: {versions:?}");
    }
}

// behavior: A suite's `marker_key` is sent to the adapter in the request, and without the key the request says `Check`
#[test]
fn the_suite_marker_key_reaches_the_adapter() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "marker = \"# Check: <id>\"", "marker = \"# Proof: <id>\"\nmarker_key = \"Proof\"");
    for file in [LOAN_LIMIT_TESTS, LATE_FINE_TESTS, FREE_RETURN_TESTS] {
        let text = repo.read(file).replace("# Check: ", "# Proof: ");
        repo.write(file, &text);
    }
    let document = repo.run_json(&[], 0);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "passed", "{document:#}");
    }
    let runs = repo.adapter_runs();
    assert_eq!(runs.len(), 1, "{runs:#?}");
    assert_eq!(runs[0]["request"]["marker_key"], "Proof", "{:#}", runs[0]);

    let plain = Repo::sample();
    plain.run_json(&[], 0);
    assert_eq!(plain.adapter_runs()[0]["request"]["marker_key"], "Check");
}

// behavior: A `marker_key` that the marker form does not use leaves the Checks of the suite with no result at the launch
#[test]
fn a_marker_key_the_form_does_not_use_gives_no_result() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, "marker = \"# Check: <id>\"", "marker = \"# Check: <id>\"\nmarker_key = \"Proof\"");
    let findings = repo.lint_findings(0);
    assert!(findings.is_empty(), "lint cannot see a wrong pair: {findings:#?}");
    let document = repo.run_json(&[], 1);
    for check in ALL_CHECKS {
        assert_eq!(state_of(&document, check), "owed", "{document:#}");
        assert_eq!(reason_of(&document, check), "no result", "{document:#}");
    }
    assert!(repo.records().is_empty(), "results were written: {:#?}", repo.records());
}

// behavior: The core treats any other status, or output that is not one report, as a crashed launch
#[test]
fn a_bad_status_or_a_bad_report_is_a_crashed_launch() {
    for directive in ["adapter: crash", "adapter: garbage", "adapter: two-reports"] {
        let repo = Repo::sample();
        repo.append(LATE_FINE_TESTS, &format!("{directive}\n"));
        let document = repo.run_json(&[], 1);
        assert_eq!(document["suites"][0]["launch"], "crashed", "{directive}: {document:#}");
        for check in ALL_CHECKS {
            assert_eq!(state_of(&document, check), "owed", "{directive}: {document:#}");
            assert_eq!(reason_of(&document, check), "no result", "{directive}: {document:#}");
            assert!(repo.results_of(check).is_empty(), "{directive}: a result was written for {check}");
        }
    }
}
