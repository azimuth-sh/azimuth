use azimuth::run::{
    CheckAttempt, CheckExecutionUnit, CheckResultOutcome, ExecutedTest, ExecutedTestOutcome,
    WorkUnit,
};
use azimuth::unit_inputs::{
    assign_units, compute_inputs, parse_rule, verify_reported_unit, SourceFile, UnitRule,
};
use std::collections::BTreeMap;

fn rule(extra_unit_paths: &str) -> UnitRule {
    parse_rule(
        &format!(
            r#"{{"format":"azimuth-unit-rule","version":1,"units":[
              {{"id":"admission","selection":"ingress-admission","hermetic":true,
                "private_paths":["tests/Admission/**"{extra_unit_paths}]}},
              {{"id":"credentials","selection":"ingress-credentials",
                "private_paths":["tests/Credentials/**"]}}
            ]}}"#
        ),
        None,
    )
    .unwrap()
}

fn file(path: &str, content: &str) -> SourceFile {
    SourceFile {
        path: path.into(),
        digest: format!("sha256:{content}"),
    }
}

fn checkout(admission: &str, credentials: &str, shared: &str) -> Vec<SourceFile> {
    vec![
        file("tests/Admission/A.cs", admission),
        file("tests/Credentials/C.cs", credentials),
        file("src/Program.cs", shared),
    ]
}

fn fingerprints(rule: &UnitRule, files: &[SourceFile]) -> BTreeMap<String, String> {
    compute_inputs(rule, files)
        .units
        .into_iter()
        .map(|unit| (unit.id, unit.input_fingerprint))
        .collect()
}

#[test]
fn fingerprint_ignores_file_order_and_is_stable() {
    let rule = rule("");
    let mut files = checkout("a", "c", "s");
    let first = fingerprints(&rule, &files);
    files.reverse();
    assert_eq!(first, fingerprints(&rule, &files));
}

#[test]
fn fingerprint_changes_only_for_units_whose_inputs_changed() {
    let rule = rule("");
    let base = fingerprints(&rule, &checkout("a", "c", "s"));

    let own_change = fingerprints(&rule, &checkout("a2", "c", "s"));
    assert_ne!(base["admission"], own_change["admission"]);
    assert_eq!(base["credentials"], own_change["credentials"]);

    let shared_change = fingerprints(&rule, &checkout("a", "c", "s2"));
    assert_ne!(base["admission"], shared_change["admission"]);
    assert_ne!(base["credentials"], shared_change["credentials"]);
}

#[test]
fn fingerprint_changes_when_the_rule_changes() {
    let files = checkout("a", "c", "s");
    let narrow = fingerprints(&rule(""), &files);
    let widened = fingerprints(&rule(r#","tests/Other/**""#), &files);
    assert_ne!(narrow["credentials"], widened["credentials"]);
}

#[test]
fn overlapping_private_paths_become_shared_and_are_recorded() {
    let overlap = parse_rule(
        r#"{"format":"azimuth-unit-rule","version":1,"units":[
          {"id":"admission","selection":"s","private_paths":["tests/**"]},
          {"id":"credentials","selection":"s","private_paths":["tests/Credentials/**"]}
        ]}"#,
        None,
    )
    .unwrap();
    let files = checkout("a", "c", "s");
    let before = compute_inputs(&overlap, &files);
    assert_eq!(before.widened_paths.len(), 1);
    assert_eq!(before.widened_paths[0].path, "tests/Credentials/C.cs");
    assert_eq!(
        before.widened_paths[0].claimed_by,
        ["admission", "credentials"]
    );

    let after = compute_inputs(&overlap, &checkout("a", "c2", "s"));
    assert_ne!(
        before.units[0].input_fingerprint,
        after.units[0].input_fingerprint
    );
}

#[test]
fn rule_rejects_whole_unit_and_invalid_patterns() {
    for unit in [
        r#"{"id":"whole","selection":"s","private_paths":["a/**"]}"#,
        r#"{"id":"x","selection":"s","private_paths":["/abs/**"]}"#,
        r#"{"id":"x","selection":"s","private_paths":["a/../b"]}"#,
        r#"{"id":"x","selection":"s","private_paths":[]}"#,
    ] {
        let source = format!(r#"{{"format":"azimuth-unit-rule","version":1,"units":[{unit}]}}"#);
        assert!(parse_rule(&source, None).is_err(), "{unit}");
    }
}

fn planned_unit(id: &str, files: &[SourceFile]) -> (WorkUnit, UnitRule) {
    let rule = rule("");
    let inputs = compute_inputs(&rule, files);
    let mut requested = vec![azimuth::run_plan::RequestedCheck {
        id: "ingress".into(),
        capability: "synthetic/checks".into(),
        scenarios: Vec::new(),
        units: azimuth::run_plan::whole_units(),
    }];
    assign_units(&mut requested, &rule, &inputs);
    let unit = requested[0]
        .units
        .iter()
        .find(|unit| unit.id == id)
        .unwrap()
        .clone();
    (unit, rule)
}

fn reported(
    planned: &WorkUnit,
    tests: Vec<(&str, &str, ExecutedTestOutcome)>,
    outcome: CheckResultOutcome,
) -> CheckExecutionUnit {
    CheckExecutionUnit {
        id: planned.id.clone(),
        input_fingerprint: planned.parameters.get("input_fingerprint").cloned(),
        executed_tests: tests
            .into_iter()
            .map(|(native_id, file, outcome)| ExecutedTest {
                native_id: native_id.into(),
                file: file.into(),
                outcome,
            })
            .collect(),
        attempts: vec![CheckAttempt {
            ordinal: 1,
            activity: "run".into(),
            outcomes: BTreeMap::from([("works".to_string(), outcome)]),
        }],
    }
}

fn codes(planned: &WorkUnit, reported: &CheckExecutionUnit) -> Vec<String> {
    let mut codes = Vec::new();
    verify_reported_unit("ingress", planned, reported, &mut |code, _| {
        codes.push(code.to_string())
    });
    codes
}

#[test]
fn bundle_accepts_tests_inside_the_units_inputs() {
    let (planned, _) = planned_unit("admission", &checkout("a", "c", "s"));
    let report = reported(
        &planned,
        vec![
            ("A.One", "tests/Admission/A.cs", ExecutedTestOutcome::Passed),
            ("S.Shared", "src/Program.cs", ExecutedTestOutcome::Passed),
        ],
        CheckResultOutcome::Satisfied,
    );
    assert_eq!(codes(&planned, &report), Vec::<String>::new());
}

#[test]
fn bundle_rejects_tests_defined_in_another_units_private_paths() {
    let (planned, _) = planned_unit("admission", &checkout("a", "c", "s"));
    let report = reported(
        &planned,
        vec![(
            "C.One",
            "tests/Credentials/C.cs",
            ExecutedTestOutcome::Passed,
        )],
        CheckResultOutcome::Satisfied,
    );
    assert_eq!(codes(&planned, &report), ["run/test-outside-unit-inputs"]);
}

#[test]
fn bundle_rejects_unsafe_files_duplicates_and_missing_inventory() {
    let (planned, _) = planned_unit("admission", &checkout("a", "c", "s"));
    let unsafe_file = reported(
        &planned,
        vec![("A.One", "../escape.cs", ExecutedTestOutcome::Passed)],
        CheckResultOutcome::Satisfied,
    );
    assert_eq!(codes(&planned, &unsafe_file), ["run/test-file-invalid"]);

    let duplicate = reported(
        &planned,
        vec![
            ("A.One", "tests/Admission/A.cs", ExecutedTestOutcome::Passed),
            ("A.One", "tests/Admission/A.cs", ExecutedTestOutcome::Passed),
        ],
        CheckResultOutcome::Satisfied,
    );
    assert_eq!(codes(&planned, &duplicate), ["run/test-duplicate"]);

    let unreported = reported(&planned, Vec::new(), CheckResultOutcome::Satisfied);
    assert_eq!(codes(&planned, &unreported), ["run/unit-tests-unreported"]);
}

#[test]
fn bundle_rejects_fingerprint_drift_and_support_from_skipped_tests() {
    let (planned, _) = planned_unit("admission", &checkout("a", "c", "s"));
    let mut drifted = reported(
        &planned,
        vec![("A.One", "tests/Admission/A.cs", ExecutedTestOutcome::Passed)],
        CheckResultOutcome::Satisfied,
    );
    drifted.input_fingerprint = Some(format!("sha256:{}", "0".repeat(64)));
    assert_eq!(codes(&planned, &drifted), ["run/unit-fingerprint-mismatch"]);

    let mut missing = drifted.clone();
    missing.input_fingerprint = None;
    assert_eq!(codes(&planned, &missing), ["run/unit-fingerprint-mismatch"]);

    let skipped = reported(
        &planned,
        vec![(
            "A.One",
            "tests/Admission/A.cs",
            ExecutedTestOutcome::Skipped,
        )],
        CheckResultOutcome::Satisfied,
    );
    assert_eq!(codes(&planned, &skipped), ["run/skipped-test-support"]);

    let inconclusive = reported(
        &planned,
        vec![(
            "A.One",
            "tests/Admission/A.cs",
            ExecutedTestOutcome::Skipped,
        )],
        CheckResultOutcome::Inconclusive,
    );
    assert_eq!(codes(&planned, &inconclusive), Vec::<String>::new());
}

#[test]
fn bundle_rejects_a_satisfied_outcome_beside_a_failed_test() {
    let (planned, _) = planned_unit("admission", &checkout("a", "c", "s"));
    let failed = reported(
        &planned,
        vec![
            ("A.One", "tests/Admission/A.cs", ExecutedTestOutcome::Passed),
            ("A.Two", "tests/Admission/A.cs", ExecutedTestOutcome::Failed),
        ],
        CheckResultOutcome::Satisfied,
    );
    assert_eq!(codes(&planned, &failed), ["run/failed-test-support"]);

    let violated = reported(
        &planned,
        vec![("A.Two", "tests/Admission/A.cs", ExecutedTestOutcome::Failed)],
        CheckResultOutcome::Violated,
    );
    assert_eq!(codes(&planned, &violated), Vec::<String>::new());
}
