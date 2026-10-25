use azimuth::unit_inputs::{RuleUnit, UnitRule};
use azimuth::{
    account_verification::parse_account_verification, assurance_review, candidate_execution,
    json::Json, model::Model, spec::parse_spec,
};
const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn model() -> Model {
    Model {
        specs: vec![parse_spec("spec.md", "# Spec: catalog\n\n## Requirement: restricted-access\nCriticality: critical\n\nAccess SHALL be restricted.\n\n### Scenario: rejected-access\nEvent: rejected caller requests data\nRequired: no data is returned\n").unwrap()],
        account_verifications: vec![parse_account_verification("verification.md", "# Verification: catalog\n\n## Requirement verification: restricted-access\n\n### Check: rejects-access\n- Evidence bindings:\n  - Scenario: `rejected-access`\n    - Contribution: Establishes rejection.\n\nExercise rejection and a successful control.\n").unwrap()],
        ..Model::default()
    }
}
fn observed_bundle(
    authority: &Json,
    check_id: &str,
    scenario_id: &str,
    time: u64,
) -> azimuth::run::RunBundle {
    use azimuth::run::*;
    use std::collections::BTreeMap;
    let check_fp = authority
        .get("checks")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check.get("id").and_then(Json::as_str) == Some(check_id))
        .unwrap()
        .get("fingerprint")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let selection = CheckSelection {
        id: check_id.into(),
        fingerprint: check_fp.clone(),
        scenarios: vec![scenario_id.into()],
        implementations: vec![Implementation {
            identity: "catalog|rust-symbol|test::check".into(),
            source_fingerprint: SHA.into(),
        }],
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::new(),
        }],
    };
    let mut bundle = RunBundle {
        run_id: SHA.into(),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: SHA.into(),
        verification_target: VerificationTarget::Workspace {
            repositories: vec![RepositoryState {
                id: "root".into(),
                revision: "review-fixture".into(),
                content_fingerprint: SHA.into(),
            }],
        },
        verification_target_fingerprint: SHA.into(),
        planned_at_ms: time - 2,
        started_at_ms: time - 1,
        finished_at_ms: time + 1,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: SHA.into(),
            required_execution_context: BTreeMap::new(),
            checks: vec![selection.clone()],
            challenges: vec![],
            fingerprint: SHA.into(),
        },
        actual_selection: ActualSelection {
            execution_context: BTreeMap::new(),
            plan_fingerprint: SHA.into(),
            checks: vec![selection],
            challenges: vec![],
            fingerprint: SHA.into(),
        },
        provenance: Provenance {
            mode: ProvenanceMode::Execute,
            source: SourceProvenance {
                system: "fixture-runner".into(),
                execution: format!("execution-{time}"),
                uri: None,
            },
            normalizer: Normalizer {
                id: "adapter/synthetic".into(),
                version: "alpha.6".into(),
                build_fingerprint: SHA.into(),
            },
            adapter: AdapterProvenance {
                id: "synthetic".into(),
                adapter_version: "alpha.6".into(),
                adapter_fingerprint: SHA.into(),
                descriptor_fingerprint: SHA.into(),
                configuration_fingerprint: SHA.into(),
                launch_fingerprint: SHA.into(),
                routes: vec![LaunchRoute {
                    selection: RouteSelection {
                        kind: RouteSelectionKind::Check,
                        id: check_id.into(),
                    },
                    capability: RouteCapability {
                        address: "synthetic/checks".into(),
                        class: RouteCapabilityClass::CheckExecute,
                        challenge_form: None,
                        fingerprint: SHA.into(),
                    },
                    inputs: vec![],
                }],
                import_inputs: vec![],
            },
            generated_at_ms: time + 2,
            principal: None,
            attributes: None,
        },
        artifacts: vec![Artifact {
            id: "native-report".into(),
            kind: "test-report".into(),
            media_type: "application/json".into(),
            digest: SHA.into(),
            size_bytes: 12,
            locator: ArtifactLocator {
                kind: LocatorKind::BundleRelative,
                value: "reports/native.json".into(),
            },
        }],
        diagnostics: vec![],
        activities: vec![Activity {
            id: "exercise".into(),
            status: ActivityStatus::Completed,
            started_at_ms: time - 1,
            finished_at_ms: time,
            artifacts: vec!["native-report".into()],
            diagnostics: vec![],
            attributes: BTreeMap::new(),
        }],
        check_executions: vec![CheckExecution {
            check: CheckRef {
                id: check_id.into(),
                fingerprint: check_fp,
            },
            units: vec![CheckExecutionUnit {
                id: "whole".into(),
                input_fingerprint: None,
                executed_tests: Vec::new(),
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "exercise".into(),
                    outcomes: [(scenario_id.into(), CheckResultOutcome::Satisfied)]
                        .into_iter()
                        .collect(),
                }],
            }],
            results: vec![CheckResult {
                scenario: scenario_id.into(),
                outcome: CheckResultOutcome::Satisfied,
                observed_at_ms: time,
                fingerprint: SHA.into(),
                artifacts: vec!["native-report".into()],
                diagnostics: vec![],
            }],
        }],
        verification_challenge_executions: vec![],
        contributions: vec![],
    };
    refresh_observed_bundle(&mut bundle);
    assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
    bundle
}
fn refresh_observed_bundle(bundle: &mut azimuth::run::RunBundle) {
    use azimuth::run::*;
    bundle.verification_target_fingerprint =
        verification_target_fingerprint(&bundle.verification_target);
    bundle.plan.fingerprint =
        plan_fingerprint(&bundle.verification_target_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    let adapter = &bundle.provenance.adapter;
    bundle.provenance.adapter.launch_fingerprint = launch_fingerprint(
        bundle.provenance.mode,
        bundle.planned_at_ms,
        &bundle.verification_target,
        &bundle.verification_target_fingerprint,
        &bundle.plan,
        &LaunchAdapterIdentity {
            id: adapter.id.clone(),
            adapter_version: adapter.adapter_version.clone(),
            adapter_fingerprint: adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: adapter.configuration_fingerprint.clone(),
        },
        &adapter.routes,
    );
    bundle.run_id = run_id(bundle);
    bundle.check_executions[0].results[0].fingerprint = check_result_fingerprint(
        bundle,
        &bundle.check_executions[0],
        &bundle.check_executions[0].results[0],
    );
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}

fn authority(declared: &[&str]) -> Json {
    let mut model = model();
    if !declared.is_empty() {
        let units = azimuth::fingerprint::canonical_json(&Json::Arr(
            declared.iter().map(|id| Json::str(*id)).collect(),
        ))
        .lines()
        .collect::<String>();
        let document = format!("# Verification: catalog\n\n## Requirement verification: restricted-access\n\n### Check: rejects-access\n- Deployment: {{\"verification_target\":\"source\",\"components\":[\"root\"],\"contexts\":[],\"contribution\":\"both\",\"monitor\":null,\"units\":{units}}}\n- Evidence bindings:\n  - Scenario: `rejected-access`\n    - Contribution: Establishes rejection.\n\nExercise rejection.\n");
        model.account_verifications =
            vec![parse_account_verification("verification.md", &document).unwrap()];
    }
    assurance_review::authority(&model, "sample-project").unwrap()
}
fn rule(units: &[&str], checks: &[&str]) -> UnitRule {
    UnitRule {
        fingerprint: SHA.into(),
        units: units
            .iter()
            .map(|id| RuleUnit {
                id: (*id).into(),
                selection: (*id).into(),
                private_paths: vec![],
                hermetic: false,
                checks: checks.iter().map(|check| (*check).into()).collect(),
            })
            .collect(),
    }
}
fn execution(authority: &Json, units: &[&str]) -> azimuth::run::RunBundle {
    let mut bundle = observed_bundle(authority, "rejects-access", "rejected-access", 10);
    let selection = bundle.plan.checks[0].units[0].clone();
    let measured = bundle.check_executions[0].units[0].clone();
    let activity = bundle.activities[0].clone();
    bundle.plan.checks[0].units = units
        .iter()
        .map(|id| {
            let mut unit = selection.clone();
            unit.id = (*id).into();
            unit
        })
        .collect();
    bundle.actual_selection.checks = bundle.plan.checks.clone();
    bundle.activities = units
        .iter()
        .map(|id| {
            let mut value = activity.clone();
            value.id = format!("exercise-{id}");
            value
        })
        .collect();
    bundle.check_executions[0].units = units
        .iter()
        .map(|id| {
            let mut unit = measured.clone();
            unit.id = (*id).into();
            unit.attempts[0].activity = format!("exercise-{id}");
            unit
        })
        .collect();
    refresh_observed_bundle(&mut bundle);
    assert!(
        azimuth::run::verify(&bundle).is_empty(),
        "{:?}",
        azimuth::run::verify(&bundle)
    );
    bundle
}
fn status(authority: &Json, units: &[&str], rule: Option<&UnitRule>) -> String {
    candidate_execution::verify_with_unit_rule(authority, &execution(authority, units), rule)
        .unwrap()
        .get("status")
        .unwrap()
        .as_str()
        .unwrap()
        .into()
}
#[test]
fn declared_units_require_every_obligation_and_override_partition_rules() {
    let authority = authority(&["allow", "deny"]);
    let narrower = rule(&["allow"], &[]);
    assert_eq!(status(&authority, &["allow", "deny"], None), "passed");
    assert_eq!(status(&authority, &["allow"], None), "incomplete");
    assert_eq!(status(&authority, &["whole"], None), "incomplete");
    assert_eq!(
        status(&authority, &["allow"], Some(&narrower)),
        "incomplete"
    );
    assert_eq!(
        status(&authority, &["allow", "deny"], Some(&narrower)),
        "passed"
    );
}
#[test]
fn partitioned_execution_requires_complete_applicable_unit_rule() {
    let authority = authority(&[]);
    let partition = rule(&["allow", "deny"], &["rejects-access"]);
    assert_eq!(status(&authority, &["whole"], None), "passed");
    assert_eq!(status(&authority, &["allow", "deny"], None), "incomplete");
    assert_eq!(
        status(&authority, &["allow", "deny"], Some(&partition)),
        "passed"
    );
    assert_eq!(
        status(&authority, &["allow"], Some(&partition)),
        "incomplete"
    );
    assert_eq!(
        status(&authority, &["whole"], Some(&partition)),
        "incomplete"
    );
    let unrelated = rule(&["allow", "deny"], &["another-check"]);
    assert_eq!(
        status(&authority, &["allow", "deny"], Some(&unrelated)),
        "incomplete"
    );
    let universal = rule(&["allow", "deny"], &[]);
    assert_eq!(
        status(&authority, &["allow", "deny"], Some(&universal)),
        "passed"
    );
}
