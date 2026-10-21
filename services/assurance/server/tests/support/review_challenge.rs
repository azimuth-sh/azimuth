fn challenge_bundle() -> RunBundle {
    let subject = Subject::Workspace {
        repositories: vec![RepositoryState {
            id: "root".into(),
            revision: "revision-a".into(),
            content_fingerprint: fp('a'),
        }],
    };
    let context = BTreeMap::from([("platform".into(), "linux".into())]);
    let check = CheckSelection {
        id: "recovery".into(),
        fingerprint: fp('b'),
        cases: vec!["works".into()],
        implementations: vec![Implementation {
            identity: "payments|rust-symbol|recovery::replay".into(),
            source_fingerprint: fp('c'),
        }],
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::new(),
        }],
    };
    let scope = construct_challenge_scope(
        vec![ChallengeScopeItem {
            kind: ChallengeScopeKind::Realization,
            id: "payments|rust-symbol|recovery::replay".into(),
            fingerprint: fp('c'),
        }],
        vec![ChallengeScopeItem {
            kind: ChallengeScopeKind::CheckImplementation,
            id: "payments|rust-symbol|recovery::replay".into(),
            fingerprint: fp('c'),
        }],
    )
    .unwrap();
    let mut challenge = ChallengeSelection {
        id: "challenge/placeholder".into(),
        challenger: ChallengerRef {
            id: "mutation/perturbation".into(),
            fingerprint: fp('d'),
        },
        target: ChallengeTarget {
            kind: ChallengeTargetKind::MethodQualification,
            id: "payments/recovery-edge".into(),
            fingerprint: fp('e'),
        },
        lane: ChallengeLane::Gate,
        scope,
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::new(),
        }],
    };
    challenge.id = challenge_selection_id(
        &challenge.challenger.fingerprint,
        challenge.target.kind,
        &challenge.target.fingerprint,
    );
    let mut bundle = RunBundle {
        contributions: Vec::new(),
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
                id: "adapter/synthetic".into(),
                version: "alpha.2".into(),
                build_fingerprint: fp('1'),
            },
            adapter: AdapterProvenance {
                id: "synthetic".into(),
                adapter_version: "alpha.2".into(),
                adapter_fingerprint: fp('1'),
                descriptor_fingerprint: fp('3'),
                configuration_fingerprint: fp('4'),
                launch_fingerprint: fp('5'),
                routes: vec![
                    LaunchRoute {
                        selection: RouteSelection {
                            kind: RouteSelectionKind::Check,
                            id: "recovery".into(),
                        },
                        capability: RouteCapability {
                            address: "synthetic/checks".into(),
                            class: RouteCapabilityClass::CheckExecute,
                            challenge_form: None,
                            fingerprint: fp('6'),
                        },
                        inputs: vec![],
                    },
                    LaunchRoute {
                        selection: RouteSelection {
                            kind: RouteSelectionKind::Challenge,
                            id: challenge.id.clone(),
                        },
                        capability: RouteCapability {
                            address: "synthetic/challenges".into(),
                            class: RouteCapabilityClass::ChallengeExecute,
                            challenge_form: Some("implementation-perturbation".into()),
                            fingerprint: fp('7'),
                        },
                        inputs: vec![
                            LaunchInput {
                                kind: LaunchInputKind::CheckImplementation,
                                id: "payments|rust-symbol|recovery::replay".into(),
                                fingerprint: fp('c'),
                                source: LaunchInputSource::Source {
                                    file: "tests/recovery.rs".into(),
                                    language: "rust".into(),
                                    site: "recovery::replay".into(),
                                },
                            },
                            LaunchInput {
                                kind: LaunchInputKind::Realization,
                                id: "payments|rust-symbol|recovery::replay".into(),
                                fingerprint: fp('c'),
                                source: LaunchInputSource::Source {
                                    file: "src/recovery.rs".into(),
                                    language: "rust".into(),
                                    site: "recovery::replay".into(),
                                },
                            },
                        ],
                    },
                ],
                import_inputs: vec![],
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
                id: "recovery".into(),
                fingerprint: fp('b'),
            },
            units: vec![CheckExecutionUnit {
                id: "whole".into(),
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "fault-probe".into(),
                    outcomes: [("works".into(), ObservationOutcome::Satisfied)]
                        .into_iter()
                        .collect(),
                }],
            }],
            observations: vec![Observation {
                case: "works".into(),
                outcome: ObservationOutcome::Satisfied,
                observed_at_ms: 3,
                fingerprint: fp('0'),
                artifacts: vec!["native-report".into()],
                diagnostics: vec![],
            }],
        }],
        challenger_executions: vec![ChallengerExecution {
            challenge: challenge.id.clone(),
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

