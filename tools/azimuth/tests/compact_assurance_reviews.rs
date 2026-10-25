//! Independent reviews pin definitions rather than inheriting authored confidence.
use azimuth::account_verification::parse_account_verification;
use azimuth::assurance_review::{self, assess_record};
use azimuth::json::{self, Json};
use azimuth::model::Model;
use azimuth::spec::parse_spec;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn model() -> Model {
    Model {
        specs: vec![parse_spec("spec.md", "# Spec: catalog\n\n## Requirement: restricted-access\nVerification: reviewed\n\nAccess SHALL be restricted.\n\n### Scenario: rejected-access\nEvent: rejected caller requests data\nRequired: no data is returned\n").unwrap()],
        account_verifications: vec![parse_account_verification("verification.md", "# Verification: catalog\n\n## Requirement verification: restricted-access\n\n### Check: rejects-access\n- Evidence bindings:\n  - Scenario: `rejected-access`\n    - Contribution: Establishes rejection.\n\nExercise rejection and a successful control.\n").unwrap()],
        ..Model::default()
    }
}
fn seal(mut record: Json) -> Json {
    if let Json::Obj(fields) = &mut record {
        fields.retain(|(name, _)| name != "fingerprint");
    }
    let fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fingerprint)));
    }
    record
}
fn verification_review(authority: &Json, at: u64) -> Json {
    let requirement = authority.get("requirements").unwrap().as_array().unwrap()[0].clone();
    seal(Json::obj(vec![
        ("kind", Json::str("verification-review")),
        ("reviewer", Json::str("independent-reviewer")),
        ("reviewed_at_ms", Json::Num(at as f64)),
        (
            "rationale",
            Json::str("The rejection oracle detects unauthorized data with a successful control."),
        ),
        (
            "artifacts",
            json::parse(&format!(
                r#"[{{"id":"review-notes","digest":"{SHA}","locator":"reviews/notes.md"}}]"#
            ))
            .unwrap(),
        ),
        ("requirement", Json::str("restricted-access")),
        (
            "requirement_fingerprint",
            requirement.get("fingerprint").unwrap().clone(),
        ),
        (
            "dependencies_fingerprint",
            Json::str(
                assurance_review::requirement_closure_fingerprint(authority, "restricted-access")
                    .unwrap(),
            ),
        ),
        ("decision", Json::str("accepted")),
        ("reassesses", Json::Null),
        ("challenge_basis", Json::Arr(vec![])),
    ]))
}
#[test]
fn conflicting_latest_reviews_do_not_fall_back_to_earlier_acceptance() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let earlier = verification_review(&authority, 50);
    let later = verification_review(&authority, 100);
    let mut conflicting = later.clone();
    if let Json::Obj(fields) = &mut conflicting {
        fields.iter_mut().find(|(k, _)| k == "decision").unwrap().1 = Json::str("rejected");
    }
    let conflicting = seal(conflicting);
    let target = assurance_review::review_target(&later).unwrap();
    assert!(assurance_review::latest_review(&[earlier, later, conflicting], &target).is_err());
}
#[test]
fn editing_review_content_without_recomputing_fingerprint_is_invalid() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let mut record = verification_review(&authority, 100);
    assert_eq!(
        assess_record(&record, &authority, &[], &[]).status,
        "accepted"
    );
    if let Json::Obj(fields) = &mut record {
        fields.iter_mut().find(|(k, _)| k == "rationale").unwrap().1 =
            Json::str("Substituted rationale.");
    }
    assert_eq!(
        assess_record(&record, &authority, &[], &[]).status,
        "invalid"
    );
}

fn reverse_object_fields(value: &mut Json) {
    match value {
        Json::Obj(fields) => {
            fields.reverse();
            for (_, child) in fields {
                reverse_object_fields(child);
            }
        }
        Json::Arr(values) => {
            for child in values {
                reverse_object_fields(child);
            }
        }
        _ => {}
    }
}

#[test]
fn latest_review_target_is_independent_of_json_object_key_order() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let earlier = verification_review(&authority, 50);
    let later = verification_review(&authority, 100);
    let mut target = assurance_review::review_target(&later).unwrap();
    reverse_object_fields(&mut target);
    let reviews = vec![earlier, later];
    assert_eq!(
        assurance_review::latest_review(&reviews, &target).unwrap(),
        Some(&reviews[1])
    );
}

#[test]
fn requirement_review_input_pins_definition_without_manufacturing_a_review() {
    let input = assurance_review::review_input(
        &model(),
        "sample-project",
        &json::parse(r#"{"kind":"verification-review","requirement":"restricted-access"}"#)
            .unwrap(),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        input.get("format").and_then(Json::as_str),
        Some("azimuth-review-input")
    );
    let target = input.get("target").unwrap();
    assert_eq!(
        target.get("requirement").and_then(Json::as_str),
        Some("restricted-access")
    );
    for field in ["decision", "reviewer", "conclusion", "reviewed_at_ms"] {
        assert!(input.get(field).is_none());
    }
    assert!(!input.get("gaps").unwrap().as_array().unwrap().is_empty());
    assert_eq!(
        input.get("fingerprint").and_then(Json::as_str),
        Some(
            assurance_review::record_fingerprint(&input)
                .unwrap()
                .as_str()
        )
    );
    let checks = input
        .get("definitions")
        .unwrap()
        .get("checks")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(checks.len(), 1);
    assert!(checks[0].get("methods").unwrap().as_array().unwrap()[0]
        .as_str()
        .unwrap()
        .contains("successful control"));
}

#[test]
fn requirement_status_input_is_rejected_because_status_is_computed() {
    let error = assurance_review::review_input(
        &model(),
        "sample-project",
        &json::parse(&format!(
            r#"{{"kind":"requirement-status","requirement":"restricted-access","verification_target_fingerprint":"{SHA}"}}"#
        ))
        .unwrap(),
        &[],
        &[],
    )
    .unwrap_err();
    assert!(error.contains("no longer authored"), "{error}");
}

#[test]
fn review_input_rejects_unknown_targets_and_authored_decisions() {
    for request in [
        r#"{"kind":"verification-review","requirement":"unknown-requirement"}"#,
        r#"{"kind":"verification-review","requirement":"restricted-access","decision":"accepted"}"#,
        r#"{"kind":"requirement-status","requirement":"unknown-requirement","verification_target_fingerprint":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        r#"{"kind":"requirement-status","requirement":"restricted-access"}"#,
        r#"{"kind":"method-qualification","check":"unknown-check"}"#,
        r#"{"kind":"method-qualification","check":"rejects-access","decision":"accepted"}"#,
        r#"{"kind":"applicability-decision","check":"rejects-access","scenario":"unknown-scenario","verification_target_fingerprint":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        r#"{"kind":"requirement-judgment","requirement":"restricted-access"}"#,
    ] {
        assert!(assurance_review::review_input(
            &model(),
            "sample-project",
            &json::parse(request).unwrap(),
            &[],
            &[]
        )
        .is_err());
    }
}

#[test]
fn requirement_review_input_includes_all_current_dependencies_and_visible_gaps() {
    let input = assurance_review::review_input(
        &model(),
        "sample-project",
        &json::parse(r#"{"kind":"verification-review","requirement":"restricted-access"}"#)
            .unwrap(),
        &[],
        &[],
    )
    .unwrap();
    let dependencies = input.get("dependencies").unwrap();
    for field in ["requirements", "scenarios", "checks", "bindings"] {
        assert_eq!(
            dependencies.get(field).unwrap().as_array().unwrap().len(),
            1
        );
    }
    assert!(input
        .get("gaps")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .any(|gap| gap.get("record_status").and_then(Json::as_str) == Some("missing")));
    assert!(!dependencies
        .get("requirements")
        .unwrap()
        .as_array()
        .unwrap()[0]
        .get("gaps")
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn unrelated_authority_changes_do_not_stale_a_verification_review() {
    let mut model = model();
    let initial = assurance_review::authority(&model, "sample-project").unwrap();
    let review = verification_review(&initial, 100);
    model.specs.push(parse_spec("other/spec.md", "# Spec: other\n\n## Requirement: other-restriction\n\nOther access is restricted.\n\n### Scenario: other-rejected\nOther rejection occurs.\n").unwrap());
    let changed = assurance_review::authority(&model, "sample-project").unwrap();
    assert_ne!(initial.get("fingerprint"), changed.get("fingerprint"));
    assert_eq!(
        assess_record(&review, &changed, &[], &[]).status,
        "accepted"
    );
}

#[test]
fn evidence_binding_edits_stale_a_verification_review() {
    let mut model = model();
    let initial = assurance_review::authority(&model, "sample-project").unwrap();
    let review = verification_review(&initial, 100);
    model.account_verifications[0].checks[0].bindings[0]
        .contribution
        .push_str(" A clarified contribution.");
    let changed = assurance_review::authority(&model, "sample-project").unwrap();
    assert_ne!(initial.get("bindings"), changed.get("bindings"));
    assert_eq!(assess_record(&review, &changed, &[], &[]).status, "stale");
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
#[test]
fn requirement_review_packet_retains_relevant_native_execution_evidence() {
    let mut model = model();
    model.specs.push(parse_spec("other/spec.md", "# Spec: other\n\n## Requirement: other-restriction\n\nOther access is restricted.\n\n### Scenario: other-rejected\nOther rejection occurs.\n").unwrap());
    model.account_verifications.push(parse_account_verification("other/verification.md", "# Verification: other\n\n## Requirement verification: other-restriction\n\n### Check: rejects-other\n- Evidence bindings:\n  - Scenario: `other-rejected`\n    - Contribution: Establishes other rejection.\n\nExercise other rejection.\n").unwrap());
    let authority = assurance_review::authority(&model, "sample-project").unwrap();
    let bundle = observed_bundle(&authority, "rejects-access", "rejected-access", 10);
    let request =
        json::parse(r#"{"kind":"verification-review","requirement":"restricted-access"}"#).unwrap();
    let unrelated = observed_bundle(&authority, "rejects-other", "other-rejected", 20);
    let packet = assurance_review::review_input(
        &model,
        "sample-project",
        &request,
        &[],
        &[bundle.clone(), unrelated],
    )
    .unwrap();
    assert_eq!(
        packet.get("evidence_bundles").unwrap().as_array().unwrap(),
        &[azimuth::run::to_json(&bundle)]
    );
    assert_eq!(
        packet
            .get("verification_targets")
            .unwrap()
            .as_array()
            .unwrap(),
        &[azimuth::run::to_json(&bundle)
            .get("verification_target")
            .unwrap()
            .clone()]
    );
    let results = packet.get("check_results").unwrap().as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].get("run"), Some(&Json::str(&bundle.run_id)));
    assert_eq!(
        results[0].get("bundle_fingerprint"),
        Some(&Json::str(&bundle.bundle_fingerprint))
    );
    assert_eq!(
        results[0].get("execution"),
        azimuth::run::to_json(&bundle)
            .get("check_executions")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
    );
    assert!(packet.get("decision").is_none());
}

#[test]
fn review_input_retains_existing_review_without_rewriting_its_decision() {
    let model = model();
    let authority = assurance_review::authority(&model, "sample-project").unwrap();
    let request =
        json::parse(r#"{"kind":"verification-review","requirement":"restricted-access"}"#).unwrap();
    let input =
        assurance_review::review_input(&model, "sample-project", &request, &[], &[]).unwrap();
    let review = verification_review(&authority, 100);
    assert_eq!(
        review
            .get("dependencies_fingerprint")
            .and_then(Json::as_str),
        Some(azimuth::fingerprint::canonical_sha256(input.get("dependencies").unwrap()).as_str())
    );
    let packet =
        assurance_review::review_input(&model, "sample-project", &request, &[review.clone()], &[])
            .unwrap();
    assert_eq!(
        packet.get("existing_reviews").unwrap().as_array().unwrap(),
        &[review]
    );
    assert!(packet.get("decision").is_none());
    assert_eq!(
        packet.get("review_states").unwrap().as_array().unwrap()[0]
            .get("record_status")
            .and_then(Json::as_str),
        Some("accepted")
    );
}
