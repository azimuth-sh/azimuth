fn fp(c: char) -> String {
    format!("sha256:{}", c.to_string().repeat(64))
}
fn native(value: &Value) -> azimuth::json::Json {
    azimuth::json::parse(&value.to_string()).unwrap()
}
fn value(value: &azimuth::json::Json) -> Value {
    serde_json::from_str(&value.to_string_pretty()).unwrap()
}
fn contract() -> Value {
    let mut result = json!({"name":"settings","fields":{"enabled":"boolean","revision":"integer","mode":"string"}});
    result["fingerprint"] = json!(azimuth::run::canonical_fingerprint(&native(&result)).unwrap());
    result
}
fn publication() -> Value {
    publication_policy(
        json!({"subject":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}),
    )
}
fn publication_policy(policy: Value) -> Value {
    publication_policy_with_criticality(policy,"routine")
}
fn publication_policy_with_criticality(policy: Value, criticality:&str) -> Value {
    let deployment_id = if policy["monitor"].is_null() {
        "deployment"
    } else {
        "monitor-deployment"
    };
    let model=azimuth::model::Model {
        specs:vec![azimuth::spec::parse_spec("spec.md",&"# Spec: sample\n\n## Claim: restricted\nCriticality: routine\n\nAccess SHALL be restricted.\n\n### Case: works\nEvent: request\nRequired: restricted access\n".replace("Criticality: routine",&format!("Criticality: {criticality}"))).unwrap()],
        account_verifications:vec![azimuth::account_verification::parse_account_verification("verification.md",&format!("# Verification: sample\n\n## Claim verification: restricted\n\n### Check: recovery\n- Deployment: {policy}\n- Evidence bindings:\n  - Case: `works`\n    - Contribution: Establishes access restrictions.\n\nExercise access restrictions.\n")).unwrap()],
        ..Default::default()
    };
    let authority = value(&azimuth::assurance_review::authority(&model, "project").unwrap());
    let subject = json!({"kind":"deployment","environment":"production","deployment":deployment_id,"deployment_fingerprint":fp('c'),"artifacts":[{"id":"api","digest":fp('e')}]});
    let subject_fp = azimuth::run::subject_fingerprint(
        &azimuth::run::subject_from_json(&native(&subject)).unwrap(),
    );
    let deployment = json!({"id":deployment_id,"environment":"production","configuration_fingerprint":fp('c'),"artifacts":[fp('e')],"subject":subject,"subject_fingerprint":subject_fp});
    value(&azimuth::release::prepare(&native(&json!({
        "format":"azimuth-release-request","version":1,"project":"project","label":"Synthetic release",
        "baseline_model_revision":authority["fingerprint"],"changes":[{"id":"restricted","revision":fp('a')}],
        "sources":[{"repository":"root","commit":"a".repeat(40),"content_fingerprint":fp('a')}],
        "builds":[{"id":"build-api","sources":["root"],"toolchain":{"compiler":"1"},"configuration_fingerprint":fp('b'),"artifacts":[fp('e')]}],
        "artifacts":[{"digest":fp('e'),"kind":"container"}],"deployments":[deployment],"components":[{"id":"api","digest":fp('e')}],
        "snapshot":{"project":"project","model_revision":authority["fingerprint"],"authority":authority,"reviews":[],"bundles":[]},"evidence":[]
    }))).unwrap())
}

fn context() -> BTreeMap<String, String> {
    BTreeMap::new()
}
fn valid_bundle() -> RunBundle {
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
    let mut bundle = RunBundle {
        contributions: Vec::new(),
        run_id: fp('0'),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: fp('0'),
        subject: Subject::Workspace {
            repositories: vec![RepositoryState {
                id: "root".into(),
                revision: "revision-a".into(),
                content_fingerprint: fp('a'),
            }],
        },
        subject_fingerprint: fp('0'),
        planned_at_ms: 1,
        started_at_ms: 2,
        finished_at_ms: 4,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: fp('d'),
            required_context: context(),
            checks: vec![check.clone()],
            challenges: vec![],
            fingerprint: fp('0'),
        },
        actual_selection: ActualSelection {
            context: context(),
            plan_fingerprint: fp('0'),
            checks: vec![check],
            challenges: vec![],
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
                build_fingerprint: fp('e'),
            },
            adapter: AdapterProvenance {
                id: "synthetic".into(),
                adapter_version: "alpha.2".into(),
                adapter_fingerprint: fp('e'),
                descriptor_fingerprint: fp('f'),
                configuration_fingerprint: fp('7'),
                launch_fingerprint: fp('8'),
                routes: vec![LaunchRoute {
                    selection: RouteSelection {
                        kind: RouteSelectionKind::Check,
                        id: "recovery".into(),
                    },
                    capability: RouteCapability {
                        address: "synthetic/checks".into(),
                        class: RouteCapabilityClass::CheckExecute,
                        challenge_form: None,
                        fingerprint: fp('9'),
                    },
                    inputs: Vec::new(),
                }],
                import_inputs: vec![],
            },
            generated_at_ms: 5,
            principal: None,
            attributes: None,
        },
        artifacts: vec![],
        diagnostics: vec![],
        activities: vec![Activity {
            id: "check-attempt".into(),
            status: ActivityStatus::Completed,
            started_at_ms: 2,
            finished_at_ms: 3,
            artifacts: vec![],
            diagnostics: vec![],
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
                    activity: "check-attempt".into(),
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
                artifacts: vec![],
                diagnostics: vec![],
            }],
        }],
        challenger_executions: vec![],
    };
    refresh(&mut bundle);
    bundle
}
fn refresh(bundle: &mut RunBundle) {
    bundle.subject_fingerprint = subject_fingerprint(&bundle.subject);
    bundle.plan.fingerprint = plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    bundle.provenance.adapter.launch_fingerprint = launch_fingerprint(
        bundle.provenance.mode,
        bundle.planned_at_ms,
        &bundle.subject,
        &bundle.subject_fingerprint,
        &bundle.plan,
        &LaunchAdapterIdentity {
            id: bundle.provenance.adapter.id.clone(),
            adapter_version: bundle.provenance.adapter.adapter_version.clone(),
            adapter_fingerprint: bundle.provenance.adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: bundle.provenance.adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: bundle.provenance.adapter.configuration_fingerprint.clone(),
        },
        &bundle.provenance.adapter.routes,
    );
    bundle.run_id = run_id(bundle);
    for index in 0..bundle.check_executions.len() {
        for observation_index in 0..bundle.check_executions[index].observations.len() {
            let fingerprint = observation_fingerprint(
                bundle,
                &bundle.check_executions[index],
                &bundle.check_executions[index].observations[observation_index],
            );
            bundle.check_executions[index].observations[observation_index].fingerprint =
                fingerprint;
        }
    }
    for index in 0..bundle.challenger_executions.len() {
        bundle.challenger_executions[index].result.fingerprint =
            challenge_result_fingerprint(bundle, &bundle.challenger_executions[index]);
    }
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}

