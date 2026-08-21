use azimuth::fingerprint::sha256;
use azimuth::run::*;
use std::collections::BTreeMap;

fn fp(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn map(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn valid_bundle() -> RunBundle {
    let subject = Subject::Workspace {
        repositories: vec![RepositoryState {
            id: "root".into(),
            revision: "revision-a".into(),
            content_fingerprint: fp('a'),
        }],
    };
    let context = map(&[("platform", "linux")]);
    let check = CheckSelection {
        id: "payments/recovery".into(),
        fingerprint: fp('b'),
        implementations: vec![Implementation {
            identity: "payments|rust-symbol|recovery::replay".into(),
            source_fingerprint: fp('c'),
        }],
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::new(),
        }],
    };
    let challenge = ChallengeSelection {
        id: "recovery-credibility".into(),
        challenger: ChallengerRef {
            id: "mutation/perturbation".into(),
            fingerprint: fp('d'),
        },
        target: ChallengeTarget {
            kind: ChallengeTargetKind::Qualification,
            id: "payments/recovery-edge".into(),
            fingerprint: fp('e'),
        },
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::new(),
        }],
    };
    let mut bundle = RunBundle {
        run_id: fp('0'),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: fp('0'),
        subject,
        subject_fingerprint: fp('0'),
        planned_at_ms: 1,
        started_at_ms: 2,
        finished_at_ms: 4,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: fp('f'),
            required_context: context.clone(),
            checks: vec![check.clone()],
            challenges: vec![challenge.clone()],
            fingerprint: fp('0'),
        },
        actual_selection: ActualSelection {
            context,
            plan_fingerprint: fp('0'),
            checks: vec![check],
            challenges: vec![challenge.clone()],
            fingerprint: fp('0'),
        },
        provenance: Provenance {
            mode: ProvenanceMode::Execute,
            source: SourceProvenance {
                system: "local-runner".into(),
                execution: "native-17".into(),
                uri: None,
            },
            normalizer: Normalizer {
                id: "azimuth/local".into(),
                version: "alpha.2".into(),
                build_fingerprint: Some(fp('1')),
            },
            generated_at_ms: 5,
            principal: Some("ci/principal".into()),
            attributes: None,
        },
        artifacts: vec![Artifact {
            id: "native-report".into(),
            kind: "test-report".into(),
            media_type: "application/json".into(),
            digest: fp('2'),
            size_bytes: 12,
            locator: ArtifactLocator {
                kind: LocatorKind::BundleRelative,
                value: "reports/native.json".into(),
            },
        }],
        diagnostics: vec![Diagnostic {
            id: "mutation/survivor".into(),
            class: DiagnosticClass::Objection,
            severity: Severity::Error,
            code: "mutation/survived".into(),
            message: "A mutation survived.".into(),
            scope: DiagnosticScope::ChallengerExecution {
                challenger_fingerprint: challenge.challenger.fingerprint.clone(),
                target_fingerprint: challenge.target.fingerprint.clone(),
            },
            artifacts: vec!["native-report".into()],
            details: BTreeMap::new(),
        }],
        activities: vec![Activity {
            id: "fault-probe".into(),
            status: ActivityStatus::Completed,
            started_at_ms: 2,
            finished_at_ms: 3,
            artifacts: vec!["native-report".into()],
            diagnostics: vec!["mutation/survivor".into()],
            attributes: BTreeMap::new(),
        }],
        check_executions: vec![CheckExecution {
            check: CheckRef {
                id: "payments/recovery".into(),
                fingerprint: fp('b'),
            },
            units: vec![CheckExecutionUnit {
                id: "whole".into(),
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "fault-probe".into(),
                    outcome: ObservationOutcome::Satisfied,
                }],
            }],
            observation: Observation {
                outcome: ObservationOutcome::Satisfied,
                observed_at_ms: 3,
                fingerprint: fp('0'),
                artifacts: vec!["native-report".into()],
                diagnostics: vec![],
            },
        }],
        challenger_executions: vec![ChallengerExecution {
            challenge: "recovery-credibility".into(),
            challenger: challenge.challenger,
            target: challenge.target,
            units: vec![ChallengeExecutionUnit {
                id: "whole".into(),
                attempts: vec![ChallengeAttempt {
                    ordinal: 1,
                    activity: "fault-probe".into(),
                    outcome: ChallengeOutcome::Findings,
                }],
            }],
            result: ChallengeResult {
                outcome: ChallengeOutcome::Findings,
                observed_at_ms: 3,
                fingerprint: fp('0'),
                objections: vec!["mutation/survivor".into()],
                artifacts: vec!["native-report".into()],
                diagnostics: vec![],
            },
        }],
    };
    refresh(&mut bundle);
    bundle
}

fn refresh(bundle: &mut RunBundle) {
    bundle.subject_fingerprint = subject_fingerprint(&bundle.subject);
    bundle.plan.fingerprint = plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    bundle.run_id = run_id(bundle);
    for index in 0..bundle.check_executions.len() {
        let fingerprint = observation_fingerprint(bundle, &bundle.check_executions[index]);
        bundle.check_executions[index].observation.fingerprint = fingerprint;
    }
    for index in 0..bundle.challenger_executions.len() {
        let fingerprint =
            challenge_result_fingerprint(bundle, &bundle.challenger_executions[index]);
        bundle.challenger_executions[index].result.fingerprint = fingerprint;
    }
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}

fn has(findings: &[Finding], code: &str) -> bool {
    findings.iter().any(|finding| finding.code == code)
}

fn terminal_without_selection(status: RunStatus) -> RunBundle {
    let mut bundle = valid_bundle();
    bundle.status = status;
    bundle.actual_selection.checks.clear();
    bundle.actual_selection.challenges.clear();
    bundle.artifacts.clear();
    bundle.diagnostics.clear();
    bundle.activities.clear();
    bundle.check_executions.clear();
    bundle.challenger_executions.clear();
    refresh(&mut bundle);
    bundle
}

#[test]
fn valid_dual_role_bundle_round_trips_and_verifies() {
    let bundle = valid_bundle();
    assert!(verify(&bundle).is_empty());
    let source = to_json(&bundle).to_string_pretty();
    let parsed = parse("run.json", &source).unwrap();
    assert_eq!(parsed, bundle);
    assert!(verify_set(&[parsed]).is_empty());
}

#[test]
fn subject_fingerprint_uses_the_literal_jcs_envelope() {
    let subject = Subject::Artifact {
        artifacts: vec![ArtifactState {
            id: "api".into(),
            digest: fp('a'),
        }],
    };
    let literal = format!(
        "{{\"format\":\"azimuth-subject-fingerprint\",\"subject\":{{\"artifacts\":[{{\"digest\":\"{}\",\"id\":\"api\"}}],\"kind\":\"artifact\"}},\"version\":1}}",
        fp('a')
    );
    assert_eq!(
        subject_fingerprint(&subject),
        format!("sha256:{}", sha256(literal.as_bytes()))
    );
}

#[test]
fn every_subject_variant_is_structurally_round_trippable() {
    let repository = RepositoryState {
        id: "root".into(),
        revision: "revision".into(),
        content_fingerprint: fp('a'),
    };
    let artifact = ArtifactState {
        id: "api".into(),
        digest: fp('b'),
    };
    let subjects = vec![
        Subject::Workspace {
            repositories: vec![repository.clone()],
        },
        Subject::CiCandidate {
            repositories: vec![repository],
        },
        Subject::Artifact {
            artifacts: vec![artifact.clone()],
        },
        Subject::Deployment {
            environment: "production".into(),
            deployment: "orders/release".into(),
            deployment_fingerprint: fp('c'),
            artifacts: vec![artifact],
        },
        Subject::Service {
            environment: "production".into(),
            service: "orders/api".into(),
            deployment: "orders/release".into(),
            deployment_fingerprint: fp('c'),
        },
        Subject::MonitoringWindow {
            environment: "production".into(),
            services: vec![ServiceState {
                service: "orders/api".into(),
                deployment: "orders/release".into(),
                deployment_fingerprint: fp('c'),
            }],
            window_start_ms: 2,
            window_end_ms: 3,
        },
    ];
    for subject in subjects {
        let mut bundle = valid_bundle();
        bundle.subject = subject;
        refresh(&mut bundle);
        let parsed = parse("subject.json", &to_json(&bundle).to_string_pretty()).unwrap();
        assert_eq!(parsed.subject, bundle.subject);
        assert!(verify(&parsed).is_empty());
    }
}

#[test]
fn strict_schema_rejects_unknown_duplicate_and_conditional_fields() {
    let source = to_json(&valid_bundle()).to_string_pretty();
    let unknown = source.replacen("\"run_id\"", "\"unknown\": 1,\n  \"run_id\"", 1);
    assert!(parse("unknown.json", &unknown).unwrap_err()[0]
        .detail
        .contains("unknown field"));
    let duplicate = source.replacen(
        "\"run_id\"",
        &format!("\"run_id\": \"{}\",\n  \"run_id\"", fp('0')),
        1,
    );
    assert!(parse("duplicate.json", &duplicate).unwrap_err()[0]
        .detail
        .contains("duplicate field"));
    let correction = source.replacen("\"bundle_revision\": 0", "\"bundle_revision\": 1", 1);
    assert!(parse("correction.json", &correction).unwrap_err()[0]
        .detail
        .contains("requires"));
    let missing_activity_attributes = source.replacen(",\n      \"attributes\": {}", "", 1);
    assert!(parse("activity.json", &missing_activity_attributes).is_err());
}

#[test]
fn strict_json_handles_surrogates_and_rejects_lexical_gaps() {
    let source = to_json(&valid_bundle()).to_string_pretty();
    let pair = source.replace("ci/principal", "ci/\\ud83d\\ude00");
    assert_eq!(
        parse("pair.json", &pair)
            .unwrap()
            .provenance
            .principal
            .as_deref(),
        Some("ci/😀")
    );
    for integral_spelling in [
        source.replacen("\"version\": 1", "\"version\": 1.0", 1),
        source.replacen("\"version\": 1", "\"version\": 1e0", 1),
    ] {
        assert!(parse("integral.json", &integral_spelling).is_ok());
    }
    for malformed in [
        source.replace("ci/principal", "ci/\\ud83d"),
        source.replace("ci/principal", "ci/\\ude00"),
        source.replace("ci/principal", "ci/\u{1}"),
        source.replacen("\"version\": 1", "\"version\": 01", 1),
        source.replacen("\"planned_at_ms\": 1", "\"planned_at_ms\": 1.5", 1),
        source.replacen("\"version\": 1", "\"version\": -1", 1),
        source.replacen(
            "\"planned_at_ms\": 1",
            "\"planned_at_ms\": 9007199254740992",
            1,
        ),
    ] {
        assert!(
            parse("malformed.json", &malformed).is_err(),
            "accepted {malformed}"
        );
    }
}

#[test]
fn claim_judgment_targets_and_semantic_implementation_identities_are_strict() {
    let mut claim_target = valid_bundle();
    for challenge in [
        &mut claim_target.plan.challenges[0],
        &mut claim_target.actual_selection.challenges[0],
    ] {
        challenge.target.kind = ChallengeTargetKind::ClaimJudgment;
        challenge.target.id = "payments/recovery#accepted-write".into();
    }
    claim_target.challenger_executions[0].target.kind = ChallengeTargetKind::ClaimJudgment;
    claim_target.challenger_executions[0].target.id = "payments/recovery#accepted-write".into();
    refresh(&mut claim_target);
    assert!(parse(
        "claim-target.json",
        &to_json(&claim_target).to_string_pretty()
    )
    .is_ok());

    let source = to_json(&valid_bundle()).to_string_pretty();
    for invalid in [
        "payments|file|src/lib.rs",
        "payments|rust-symbol|src/lib.rs",
        "payments|rust-symbol|recovery:*",
        "payments|rust-symbol|recovery:17",
    ] {
        let malformed = source.replace("payments|rust-symbol|recovery::replay", invalid);
        assert!(
            parse("identity.json", &malformed).is_err(),
            "accepted {invalid}"
        );
    }

    let route = source.replace(
        "payments|rust-symbol|recovery::replay",
        "payments|next-route|GET /orders/[id]",
    );
    assert!(parse("route.json", &route).is_ok());
}

#[test]
fn ordering_duplicates_and_semantic_challenge_duplicates_are_findings() {
    let mut bundle = valid_bundle();
    let mut second = bundle.plan.challenges[0].clone();
    second.id = "second".into();
    bundle.plan.challenges.push(second);
    refresh(&mut bundle);
    let parsed = parse("duplicates.json", &to_json(&bundle).to_string_pretty()).unwrap();
    let findings = verify(&parsed);
    assert!(has(&findings, "run/duplicate-challenge-target"));

    let mut reversed = valid_bundle();
    reversed.artifacts.push(Artifact {
        id: "aaa".into(),
        ..reversed.artifacts[0].clone()
    });
    refresh(&mut reversed);
    let parsed = parse("unsorted.json", &to_json(&reversed).to_string_pretty()).unwrap();
    assert!(has(&verify(&parsed), "run/non-canonical-array"));
}

#[test]
fn optional_provenance_attributes_preserve_presence_and_empty_map_values() {
    let absent = valid_bundle();
    let mut present = absent.clone();
    present.provenance.attributes = Some(map(&[("provider-ref", "")]));
    refresh(&mut present);
    let parsed = parse("attributes.json", &to_json(&present).to_string_pretty()).unwrap();
    assert_eq!(parsed, present);
    assert!(verify(&parsed).is_empty());
    assert_ne!(absent.bundle_fingerprint, present.bundle_fingerprint);
}

#[test]
fn result_fingerprints_bind_context_check_and_plan_local_challenge_id() {
    let bundle = valid_bundle();
    let observation = observation_fingerprint(&bundle, &bundle.check_executions[0]);
    let mut changed_context = bundle.clone();
    changed_context
        .actual_selection
        .context
        .insert("platform".into(), "other".into());
    assert_ne!(
        observation,
        observation_fingerprint(&changed_context, &changed_context.check_executions[0])
    );

    let mut changed_check = bundle.clone();
    changed_check.check_executions[0].check.fingerprint = fp('8');
    assert_ne!(
        observation,
        observation_fingerprint(&changed_check, &changed_check.check_executions[0])
    );

    let result = challenge_result_fingerprint(&bundle, &bundle.challenger_executions[0]);
    let mut changed_challenge = bundle.clone();
    changed_challenge.challenger_executions[0].challenge = "other-address".into();
    assert_ne!(
        result,
        challenge_result_fingerprint(
            &changed_challenge,
            &changed_challenge.challenger_executions[0]
        )
    );
}

#[test]
fn lexical_fingerprints_parse_but_mismatches_are_semantic_findings() {
    let mut bundle = valid_bundle();
    bundle.subject_fingerprint = fp('7');
    let parsed = parse("mismatch.json", &to_json(&bundle).to_string_pretty()).unwrap();
    assert!(has(&verify(&parsed), "run/subject-fingerprint"));
}

#[test]
fn actual_checks_repeat_implementations_and_subset_only_units() {
    let mut bundle = valid_bundle();
    bundle.actual_selection.checks[0].implementations.clear();
    refresh(&mut bundle);
    let findings = verify(&bundle);
    assert!(has(&findings, "run/check-implementation-substitution"));
    assert!(has(&findings, "run/observation-reduction"));

    let mut extra = valid_bundle();
    extra.actual_selection.checks[0].units.push(WorkUnit {
        id: "unplanned".into(),
        parameters: BTreeMap::new(),
    });
    refresh(&mut extra);
    assert!(has(&verify(&extra), "run/check-unit-substitution"));
}

#[test]
fn retry_reduction_recovers_technical_inconclusion_but_preserves_violation() {
    let mut recovered = valid_bundle();
    recovered.activities.insert(
        0,
        Activity {
            id: "earlier".into(),
            status: ActivityStatus::TimedOut,
            started_at_ms: 2,
            finished_at_ms: 2,
            artifacts: vec![],
            diagnostics: vec![],
            attributes: BTreeMap::new(),
        },
    );
    recovered.check_executions[0].units[0].attempts.insert(
        0,
        CheckAttempt {
            ordinal: 1,
            activity: "earlier".into(),
            outcome: ObservationOutcome::Inconclusive,
        },
    );
    recovered.check_executions[0].units[0].attempts[1].ordinal = 2;
    refresh(&mut recovered);
    assert!(verify(&recovered).is_empty());

    recovered.check_executions[0].units[0].attempts[0].outcome = ObservationOutcome::Violated;
    refresh(&mut recovered);
    assert!(has(&verify(&recovered), "run/observation-reduction"));
}

#[test]
fn violated_observations_are_valid_terminal_facts() {
    let mut bundle = valid_bundle();
    bundle.check_executions[0].units[0].attempts[0].outcome = ObservationOutcome::Violated;
    bundle.check_executions[0].observation.outcome = ObservationOutcome::Violated;
    refresh(&mut bundle);
    assert!(verify(&bundle).is_empty());
}

#[test]
fn attempts_cannot_repeat_activity_and_noncompleted_activity_is_inconclusive() {
    let mut bundle = valid_bundle();
    bundle.check_executions[0].units[0]
        .attempts
        .push(CheckAttempt {
            ordinal: 2,
            activity: "fault-probe".into(),
            outcome: ObservationOutcome::Satisfied,
        });
    bundle.activities[0].status = ActivityStatus::Failed;
    refresh(&mut bundle);
    let findings = verify(&bundle);
    assert!(has(&findings, "run/repeated-attempt-activity"));
    assert!(has(&findings, "run/activity-outcome-mismatch"));
}

#[test]
fn cardinality_references_and_objections_fail_closed() {
    let mut bundle = valid_bundle();
    bundle.check_executions.clear();
    bundle.activities[0].artifacts = vec!["missing".into()];
    bundle.challenger_executions[0].units[0].attempts[0].outcome = ChallengeOutcome::Clean;
    bundle.challenger_executions[0].result.outcome = ChallengeOutcome::Clean;
    refresh(&mut bundle);
    let findings = verify(&bundle);
    assert!(has(&findings, "run/check-execution-cardinality"));
    assert!(has(&findings, "run/unresolved-reference"));
    assert!(has(&findings, "run/unexpected-objection"));
}

#[test]
fn partial_cancelled_and_timed_out_runs_may_report_an_empty_actual_subset() {
    for status in [
        RunStatus::Partial,
        RunStatus::Cancelled,
        RunStatus::TimedOut,
    ] {
        let bundle = terminal_without_selection(status);
        assert!(verify(&bundle).is_empty(), "{}", status.name());
    }
}

#[test]
fn complete_runs_require_the_entire_plan_selection() {
    let bundle = terminal_without_selection(RunStatus::Complete);
    assert!(has(&verify(&bundle), "run/incomplete-complete-selection"));
}

#[test]
fn malformed_bundle_relative_locators_are_schema_errors() {
    let source = to_json(&valid_bundle()).to_string_pretty();
    for locator in [
        "/report.json",
        "reports//report.json",
        "reports/../report.json",
    ] {
        let malformed = source.replace("reports/native.json", locator);
        assert!(
            parse("locator.json", &malformed).is_err(),
            "accepted {locator}"
        );
    }
}

#[test]
fn timestamp_relations_and_monitoring_closure_are_findings() {
    let mut bundle = valid_bundle();
    bundle.started_at_ms = 5;
    bundle.provenance.generated_at_ms = 3;
    refresh(&mut bundle);
    let findings = verify(&bundle);
    assert!(has(&findings, "run/time-order"));
    assert!(has(&findings, "run/provenance-time"));

    let mut monitoring = valid_bundle();
    monitoring.subject = Subject::MonitoringWindow {
        environment: "production".into(),
        services: vec![ServiceState {
            service: "orders/api".into(),
            deployment: "orders/release".into(),
            deployment_fingerprint: fp('3'),
        }],
        window_start_ms: 3,
        window_end_ms: 5,
    };
    refresh(&mut monitoring);
    assert!(has(&verify(&monitoring), "run/monitoring-window"));
}

#[test]
fn every_in_memory_number_is_guarded_before_lossy_serialization() {
    let unsafe_value = 9_007_199_254_740_992;
    let mut variants = Vec::new();

    let mut bundle = valid_bundle();
    bundle.bundle_revision = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.planned_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.started_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.finished_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.provenance.generated_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.artifacts[0].size_bytes = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.activities[0].started_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.activities[0].finished_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.check_executions[0].units[0].attempts[0].ordinal = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.check_executions[0].observation.observed_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.challenger_executions[0].units[0].attempts[0].ordinal = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.challenger_executions[0].result.observed_at_ms = unsafe_value;
    variants.push(bundle);
    let mut bundle = valid_bundle();
    bundle.subject = Subject::MonitoringWindow {
        environment: "production".into(),
        services: vec![ServiceState {
            service: "orders/api".into(),
            deployment: "orders/release".into(),
            deployment_fingerprint: fp('5'),
        }],
        window_start_ms: unsafe_value,
        window_end_ms: unsafe_value,
    };
    variants.push(bundle);

    for bundle in variants {
        assert!(has(&verify(&bundle), "run/unsafe-number"));
        assert!(has(&verify_set(&[bundle]), "run/unsafe-number"));
    }
}

#[test]
fn correction_sets_are_order_independent_and_exact_replays_are_idempotent() {
    let initial = valid_bundle();
    let mut correction = initial.clone();
    correction.bundle_revision = 1;
    correction.corrects = Some(initial.bundle_fingerprint.clone());
    correction.correction_reason = Some("late report".into());
    correction.finished_at_ms = 5;
    correction.provenance.generated_at_ms = 6;
    correction.bundle_fingerprint = bundle_fingerprint(&correction);
    assert!(verify_set(&[correction.clone(), initial.clone(), initial]).is_empty());
}

#[test]
fn correction_sets_reject_gaps_forks_conflicts_and_anchor_changes() {
    let initial = valid_bundle();
    let mut second = initial.clone();
    second.bundle_revision = 2;
    second.corrects = Some(initial.bundle_fingerprint.clone());
    second.correction_reason = Some("gap".into());
    second.bundle_fingerprint = bundle_fingerprint(&second);
    let findings = verify_set(&[initial.clone(), second]);
    assert!(has(&findings, "run/history-gap"));

    let mut anchor = initial.clone();
    anchor.bundle_revision = 1;
    anchor.corrects = Some(initial.bundle_fingerprint.clone());
    anchor.correction_reason = Some("changed anchor".into());
    anchor.started_at_ms = 3;
    anchor.bundle_fingerprint = bundle_fingerprint(&anchor);
    assert!(has(
        &verify_set(&[initial, anchor]),
        "run/history-anchor-change"
    ));
}

#[test]
fn correction_sets_reject_missing_initial_conflicts_forks_and_cycles() {
    let initial = valid_bundle();

    let mut missing = initial.clone();
    missing.bundle_revision = 1;
    missing.corrects = Some(initial.bundle_fingerprint.clone());
    missing.correction_reason = Some("missing initial".into());
    missing.bundle_fingerprint = bundle_fingerprint(&missing);
    assert!(has(&verify_set(&[missing]), "run/history-missing-initial"));

    let mut conflicting = initial.clone();
    conflicting.finished_at_ms = 5;
    conflicting.provenance.generated_at_ms = 5;
    conflicting.bundle_fingerprint = bundle_fingerprint(&conflicting);
    let forward = verify_set(&[initial.clone(), conflicting.clone()]);
    let reverse = verify_set(&[conflicting, initial.clone()]);
    assert_eq!(forward, reverse);
    assert!(has(&forward, "run/history-conflict"));

    let mut first = initial.clone();
    first.bundle_revision = 1;
    first.corrects = Some(initial.bundle_fingerprint.clone());
    first.correction_reason = Some("first branch".into());
    first.bundle_fingerprint = bundle_fingerprint(&first);
    let mut second = initial.clone();
    second.bundle_revision = 2;
    second.corrects = Some(initial.bundle_fingerprint.clone());
    second.correction_reason = Some("second branch".into());
    second.bundle_fingerprint = bundle_fingerprint(&second);
    let fork_findings = verify_set(&[initial.clone(), first, second]);
    assert!(has(&fork_findings, "run/history-fork"));
    assert!(has(&fork_findings, "run/history-predecessor"));

    let mut cycle = initial.clone();
    cycle.bundle_revision = 1;
    cycle.corrects = Some(fp('9'));
    cycle.correction_reason = Some("cycle".into());
    cycle.bundle_fingerprint = fp('9');
    assert!(has(&verify_set(&[initial, cycle]), "run/history-cycle"));
}

#[test]
fn sparse_revision_checks_do_not_enumerate_the_revision_range() {
    let initial = valid_bundle();
    let mut distant = initial.clone();
    distant.bundle_revision = 9_007_199_254_740_991;
    distant.corrects = Some(initial.bundle_fingerprint.clone());
    distant.correction_reason = Some("distant revision".into());
    distant.bundle_fingerprint = bundle_fingerprint(&distant);
    let findings = verify_set(&[initial, distant]);
    assert!(has(&findings, "run/history-gap"));
    assert!(has(&findings, "run/history-missing-predecessor"));
}
