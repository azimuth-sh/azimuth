//! Independent reviews pin definitions rather than inheriting authored confidence.
use azimuth::account_verification::parse_account_verification;
use azimuth::assurance_review::{self, assess_record};
use azimuth::json::{self, Json};
use azimuth::model::Model;
use azimuth::spec::parse_spec;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn model() -> Model {
    Model {
        specs: vec![parse_spec("spec.md", "# Spec: catalog\n\n## Claim: restricted-access\nCriticality: critical\n\nAccess SHALL be restricted.\n\n### Case: rejected-access\nEvent: rejected caller requests data\nRequired: no data is returned\n").unwrap()],
        account_verifications: vec![parse_account_verification("verification.md", "# Verification: catalog\n\n## Claim verification: restricted-access\n\n### Check: rejects-access\n- Evidence bindings:\n  - Case: `rejected-access`\n    - Contribution: Establishes rejection.\n\nExercise rejection and a successful control.\n").unwrap()],
        ..Model::default()
    }
}
fn qualification(authority: &Json, at: u64) -> Json {
    let check_fp = authority.get("checks").unwrap().as_array().unwrap()[0]
        .get("fingerprint")
        .unwrap()
        .clone();
    let mut record = Json::obj(vec![
        ("kind", Json::str("method-qualification")),
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
        ("check", Json::str("rejects-access")),
        ("check_fingerprint", check_fp),
        ("decision", Json::str("accepted")),
    ]);
    let fp = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    record
}
#[test]
fn changing_check_method_stales_prior_independent_qualification() {
    let mut model = model();
    let initial = assurance_review::authority(&model, "sample-project").unwrap();
    let review = qualification(&initial, 100);
    assert_eq!(
        assess_record(&review, &initial, &[], &[]).status,
        "accepted"
    );
    model.account_verifications[0].checks[0].definition.methods[0].push_str(" A different oracle.");
    let changed = assurance_review::authority(&model, "sample-project").unwrap();
    assert_eq!(assess_record(&review, &changed, &[], &[]).status, "stale");
}
#[test]
fn conflicting_latest_reviews_do_not_fall_back_to_earlier_acceptance() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let earlier = qualification(&authority, 50);
    let later = qualification(&authority, 100);
    let mut conflicting = later.clone();
    if let Json::Obj(fields) = &mut conflicting {
        fields.iter_mut().find(|(k, _)| k == "decision").unwrap().1 = Json::str("rejected");
        fields.retain(|(k, _)| k != "fingerprint");
    }
    let fp = assurance_review::record_fingerprint(&conflicting).unwrap();
    if let Json::Obj(fields) = &mut conflicting {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    let target = assurance_review::review_target(&later).unwrap();
    assert!(assurance_review::latest_review(&[earlier, later, conflicting], &target).is_err());
}
#[test]
fn editing_review_content_without_recomputing_fingerprint_is_invalid() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let mut record = qualification(&authority, 100);
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

fn unresolved_judgment(authority: &Json) -> Json {
    let claim = &authority.get("claims").unwrap().as_array().unwrap()[0];
    let cases = authority.get("cases").unwrap().as_array().unwrap();
    let bindings = authority.get("bindings").unwrap().as_array().unwrap();
    let mut record = qualification(authority, 100);
    if let Json::Obj(fields) = &mut record {
        fields.retain(|(name, _)| {
            !matches!(
                name.as_str(),
                "check" | "check_fingerprint" | "decision" | "fingerprint"
            )
        });
        fields
            .iter_mut()
            .find(|(name, _)| name == "kind")
            .unwrap()
            .1 = Json::str("claim-judgment");
        fields.extend([
            ("claim".into(), claim.get("id").unwrap().clone()),
            (
                "claim_fingerprint".into(),
                claim.get("fingerprint").unwrap().clone(),
            ),
            ("subject_fingerprint".into(), Json::str(SHA)),
            (
                "cases".into(),
                Json::Arr(
                    cases
                        .iter()
                        .map(|case| {
                            Json::obj(vec![
                                ("id", case.get("id").unwrap().clone()),
                                ("fingerprint", case.get("fingerprint").unwrap().clone()),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "mechanisms".into(),
                claim.get("mechanisms").unwrap().clone(),
            ),
            (
                "bindings".into(),
                Json::Arr(
                    bindings
                        .iter()
                        .map(|binding| {
                            Json::obj(vec![
                                ("check", binding.get("check").unwrap().clone()),
                                ("case", binding.get("case").unwrap().clone()),
                                ("fingerprint", binding.get("fingerprint").unwrap().clone()),
                                ("applicability_fingerprint", Json::Null),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("conclusion".into(), Json::str("unresolved")),
            (
                "residual_risks".into(),
                Json::Arr(vec![Json::str("Applicable evidence is missing.")]),
            ),
        ]);
    }
    let fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fingerprint)));
    }
    record
}

#[test]
fn judgment_dependencies_survive_json_object_reordering_but_not_fingerprint_change() {
    let mut model = model();
    model.account_designs.push(azimuth::account_design::parse_account_design(
        "design.md",
        "# Design: catalog\n\n## Claim design: restricted-access\n- Areas:\n  - `api`\n\n### Mechanism: admission-guard\n- Cases: `rejected-access`\n\nThe guard precedes protected data delivery.\n",
    ).unwrap());
    let mut authority = assurance_review::authority(&model, "sample-project").unwrap();
    let mut record = unresolved_judgment(&authority);
    assert_eq!(
        assess_record(&record, &authority, &[], &[]).status,
        "accepted"
    );
    let original_fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    reverse_object_fields(&mut record);
    reverse_object_fields(&mut authority);
    assert_eq!(
        assurance_review::record_fingerprint(&record).unwrap(),
        original_fingerprint
    );
    assert_eq!(
        assess_record(&record, &authority, &[], &[]).status,
        "accepted"
    );
    if let Json::Obj(fields) = &mut record {
        let mechanisms = &mut fields
            .iter_mut()
            .find(|(name, _)| name == "mechanisms")
            .unwrap()
            .1;
        let Json::Arr(mechanisms) = mechanisms else {
            panic!("mechanisms must be an array")
        };
        let Json::Obj(dependency) = &mut mechanisms[0] else {
            panic!("dependency must be an object")
        };
        dependency
            .iter_mut()
            .find(|(name, _)| name == "fingerprint")
            .unwrap()
            .1 =
            Json::str("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        fields.retain(|(name, _)| name != "fingerprint");
    }
    let fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fingerprint)));
    }
    assert_eq!(assess_record(&record, &authority, &[], &[]).status, "stale");
}

#[test]
fn latest_review_target_is_independent_of_json_object_key_order() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let earlier = qualification(&authority, 50);
    let later = qualification(&authority, 100);
    let mut target = assurance_review::review_target(&later).unwrap();
    reverse_object_fields(&mut target);
    let reviews = vec![earlier, later];
    assert_eq!(
        assurance_review::latest_review(&reviews, &target).unwrap(),
        Some(&reviews[1])
    );
}

#[test]
fn method_review_input_pins_definition_without_manufacturing_a_review() {
    let input = assurance_review::review_input(
        &model(),
        "sample-project",
        &json::parse(r#"{"kind":"method-qualification","check":"rejects-access"}"#).unwrap(),
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
        target.get("check").and_then(Json::as_str),
        Some("rejects-access")
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
fn applicability_review_input_exposes_missing_exact_subject_evidence() {
    let input = assurance_review::review_input(&model(), "sample-project", &json::parse(&format!(
        r#"{{"kind":"applicability-decision","check":"rejects-access","case":"rejected-access","subject_fingerprint":"{SHA}"}}"#
    )).unwrap(), &[], &[]).unwrap();
    assert!(input
        .get("observations")
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty());
    assert!(input
        .get("gaps")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .any(|gap| gap.get("case").and_then(Json::as_str) == Some("rejected-access")));
    let binding = &input
        .get("dependencies")
        .unwrap()
        .get("bindings")
        .unwrap()
        .as_array()
        .unwrap()[0];
    for field in ["check_fingerprint", "case_fingerprint", "fingerprint"] {
        assert!(binding.get(field).is_some());
    }
}

#[test]
fn review_input_rejects_unknown_targets_and_authored_decisions() {
    for request in [
        r#"{"kind":"method-qualification","check":"unknown-check"}"#,
        r#"{"kind":"method-qualification","check":"rejects-access","decision":"accepted"}"#,
        r#"{"kind":"applicability-decision","check":"rejects-access","case":"unknown-case","subject_fingerprint":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        r#"{"kind":"claim-judgment","claim":"restricted-access"}"#,
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
fn claim_review_input_includes_all_current_dependencies_and_visible_gaps() {
    let input = assurance_review::review_input(&model(), "sample-project", &json::parse(&format!(
        r#"{{"kind":"claim-judgment","claim":"restricted-access","subject_fingerprint":"{SHA}"}}"#
    )).unwrap(), &[], &[]).unwrap();
    let dependencies = input.get("dependencies").unwrap();
    for field in ["claims", "cases", "checks", "bindings"] {
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
        .any(|gap| gap.get("status").and_then(Json::as_str) == Some("structural-gap")));
}

#[test]
fn unrelated_authority_changes_do_not_require_method_requalification() {
    let mut model = model();
    let initial = assurance_review::authority(&model, "sample-project").unwrap();
    let review = qualification(&initial, 100);
    model.specs.push(parse_spec("other/spec.md", "# Spec: other\n\n## Claim: other-restriction\nCriticality: routine\n\nOther access is restricted.\n\n### Case: other-rejected\nOther rejection occurs.\n").unwrap());
    let changed = assurance_review::authority(&model, "sample-project").unwrap();
    assert_ne!(initial.get("fingerprint"), changed.get("fingerprint"));
    assert_eq!(
        assess_record(&review, &changed, &[], &[]).status,
        "accepted"
    );
}

#[test]
fn evidence_binding_edits_do_not_requalify_an_unchanged_method() {
    let mut model = model();
    let initial = assurance_review::authority(&model, "sample-project").unwrap();
    let review = qualification(&initial, 100);
    model.account_verifications[0].checks[0].bindings[0]
        .contribution
        .push_str(" A clarified contribution.");
    let changed = assurance_review::authority(&model, "sample-project").unwrap();
    assert_ne!(initial.get("bindings"), changed.get("bindings"));
    assert_eq!(
        assess_record(&review, &changed, &[], &[]).status,
        "accepted"
    );
}

fn observed_bundle(
    authority: &Json,
    check_id: &str,
    case_id: &str,
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
        cases: vec![case_id.into()],
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
        subject: Subject::Workspace {
            repositories: vec![RepositoryState {
                id: "root".into(),
                revision: "review-fixture".into(),
                content_fingerprint: SHA.into(),
            }],
        },
        subject_fingerprint: SHA.into(),
        planned_at_ms: time - 2,
        started_at_ms: time - 1,
        finished_at_ms: time + 1,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: SHA.into(),
            required_context: BTreeMap::new(),
            checks: vec![selection.clone()],
            challenges: vec![],
            fingerprint: SHA.into(),
        },
        actual_selection: ActualSelection {
            context: BTreeMap::new(),
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
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "exercise".into(),
                    outcomes: [(case_id.into(), ObservationOutcome::Satisfied)]
                        .into_iter()
                        .collect(),
                }],
            }],
            observations: vec![Observation {
                case: case_id.into(),
                outcome: ObservationOutcome::Satisfied,
                observed_at_ms: time,
                fingerprint: SHA.into(),
                artifacts: vec!["native-report".into()],
                diagnostics: vec![],
            }],
        }],
        challenger_executions: vec![],
        contributions: vec![],
    };
    refresh_observed_bundle(&mut bundle);
    assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
    bundle
}
fn refresh_observed_bundle(bundle: &mut azimuth::run::RunBundle) {
    use azimuth::run::*;
    bundle.subject_fingerprint = subject_fingerprint(&bundle.subject);
    bundle.plan.fingerprint = plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    let adapter = &bundle.provenance.adapter;
    bundle.provenance.adapter.launch_fingerprint = launch_fingerprint(
        bundle.provenance.mode,
        bundle.planned_at_ms,
        &bundle.subject,
        &bundle.subject_fingerprint,
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
    bundle.check_executions[0].observations[0].fingerprint = observation_fingerprint(
        bundle,
        &bundle.check_executions[0],
        &bundle.check_executions[0].observations[0],
    );
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}
fn applicability(authority: &Json, qualification: &Json, bundle: &azimuth::run::RunBundle) -> Json {
    let binding = &authority.get("bindings").unwrap().as_array().unwrap()[0];
    let mut record = qualification.clone();
    if let Json::Obj(fields) = &mut record {
        fields.retain(|(name, _)| name != "fingerprint");
        fields
            .iter_mut()
            .find(|(name, _)| name == "kind")
            .unwrap()
            .1 = Json::str("applicability-decision");
        fields.extend([
            ("case".into(), binding.get("case").unwrap().clone()),
            (
                "case_fingerprint".into(),
                binding.get("case_fingerprint").unwrap().clone(),
            ),
            (
                "binding_fingerprint".into(),
                binding.get("fingerprint").unwrap().clone(),
            ),
            (
                "subject_fingerprint".into(),
                Json::str(&bundle.subject_fingerprint),
            ),
            (
                "method_qualification_fingerprint".into(),
                qualification.get("fingerprint").unwrap().clone(),
            ),
            (
                "basis".into(),
                Json::Arr(vec![Json::obj(vec![
                    ("run", Json::str(&bundle.run_id)),
                    ("bundle_fingerprint", Json::str(&bundle.bundle_fingerprint)),
                    (
                        "observation",
                        Json::str(&bundle.check_executions[0].observations[0].fingerprint),
                    ),
                ])]),
            ),
            ("limitations".into(), Json::Arr(vec![])),
        ]);
    }
    let fp = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    record
}
#[test]
fn fresh_observations_stale_only_relevant_applicability_not_qualified_methods() {
    let mut model = model();
    model.specs.push(parse_spec("other/spec.md", "# Spec: other\n\n## Claim: other-restriction\nCriticality: routine\n\nOther access is restricted.\n\n### Case: other-rejected\nOther rejection occurs.\n").unwrap());
    model.account_verifications.push(parse_account_verification("other/verification.md", "# Verification: other\n\n## Claim verification: other-restriction\n\n### Check: rejects-other\n- Evidence bindings:\n  - Case: `other-rejected`\n    - Contribution: Establishes other rejection.\n\nExercise other rejection.\n").unwrap());
    let authority = assurance_review::authority(&model, "sample-project").unwrap();
    let qualification = qualification(&authority, 100);
    let old = observed_bundle(&authority, "rejects-access", "rejected-access", 10);
    let application = applicability(&authority, &qualification, &old);
    let unrelated = observed_bundle(&authority, "rejects-other", "other-rejected", 20);
    let reviews = vec![qualification.clone(), application.clone()];
    assert_eq!(
        assess_record(
            &application,
            &authority,
            &reviews,
            &[old.clone(), unrelated.clone()]
        )
        .status,
        "accepted"
    );
    let relevant = observed_bundle(&authority, "rejects-access", "rejected-access", 30);
    assert_eq!(
        assess_record(
            &application,
            &authority,
            &reviews,
            &[old.clone(), unrelated, relevant.clone()]
        )
        .status,
        "stale"
    );
    assert_eq!(
        assess_record(&qualification, &authority, &reviews, &[old, relevant]).status,
        "accepted"
    );
}
#[test]
fn review_packet_keeps_exact_observations_and_native_bundle_artifacts() {
    let model = model();
    let authority = assurance_review::authority(&model, "sample-project").unwrap();
    let bundle = observed_bundle(&authority, "rejects-access", "rejected-access", 10);
    let request=json::parse(&format!(r#"{{"kind":"applicability-decision","check":"rejects-access","case":"rejected-access","subject_fingerprint":"{}"}}"#,bundle.subject_fingerprint)).unwrap();
    let packet =
        assurance_review::review_input(&model, "sample-project", &request, &[], &[bundle.clone()])
            .unwrap();
    assert_eq!(
        packet.get("evidence_bundles").unwrap().as_array().unwrap()[0],
        azimuth::run::to_json(&bundle)
    );
    assert_eq!(
        packet.get("observations").unwrap().as_array().unwrap()[0]
            .get("observation")
            .and_then(Json::as_str),
        Some(
            bundle.check_executions[0].observations[0]
                .fingerprint
                .as_str()
        )
    );
}

#[test]
fn unresolved_judgment_becomes_stale_when_its_applicability_basis_is_superseded() {
    let model = model();
    let authority = assurance_review::authority(&model, "sample-project").unwrap();
    let qualification = qualification(&authority, 100);
    let old = observed_bundle(&authority, "rejects-access", "rejected-access", 10);
    let application = applicability(&authority, &qualification, &old);
    let mut judgment = unresolved_judgment(&authority);
    if let Json::Obj(fields) = &mut judgment {
        fields.retain(|(name, _)| name != "fingerprint");
        fields
            .iter_mut()
            .find(|(name, _)| name == "subject_fingerprint")
            .unwrap()
            .1 = Json::str(&old.subject_fingerprint);
        if let Json::Arr(bindings) = &mut fields
            .iter_mut()
            .find(|(name, _)| name == "bindings")
            .unwrap()
            .1
        {
            if let Json::Obj(binding) = &mut bindings[0] {
                binding
                    .iter_mut()
                    .find(|(name, _)| name == "applicability_fingerprint")
                    .unwrap()
                    .1 = application.get("fingerprint").unwrap().clone();
            }
        }
    }
    let fp = assurance_review::record_fingerprint(&judgment).unwrap();
    if let Json::Obj(fields) = &mut judgment {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    let reviews = vec![qualification, application, judgment.clone()];
    assert_eq!(
        assess_record(&judgment, &authority, &reviews, &[old.clone()]).status,
        "accepted"
    );
    let new = observed_bundle(&authority, "rejects-access", "rejected-access", 20);
    assert_eq!(
        assess_record(&judgment, &authority, &reviews, &[old, new]).status,
        "stale"
    );
}

#[test]
fn review_input_retains_existing_review_without_rewriting_its_decision() {
    let model = model();
    let authority = assurance_review::authority(&model, "sample-project").unwrap();
    let review = qualification(&authority, 100);
    let packet = assurance_review::review_input(
        &model,
        "sample-project",
        &json::parse(r#"{"kind":"method-qualification","check":"rejects-access"}"#).unwrap(),
        &[review.clone()],
        &[],
    )
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
