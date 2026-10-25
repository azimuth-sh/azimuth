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
    let mut result = json!({"name":"api","fields":{"enabled":"boolean","revision":"integer","mode":"string"}});
    result["fingerprint"] = json!(azimuth::run::canonical_fingerprint(&native(&result)).unwrap());
    result
}
fn publication() -> Value {
    publication_policy(
        json!({"verification_target":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}),
    )
}
fn publication_policy(policy: Value) -> Value {
    publication_policy_with_header(policy, "")
}
// The Requirement owns an enrolled Check, so it declares `Verification:`; `extra_header` adds lines
// such as `Context:` conditions beneath that declaration.
fn publication_policy_with_header(policy: Value, extra_header:&str) -> Value {
    let deployment_id = if policy["monitor"].is_null() {
        "deployment"
    } else {
        "monitor-deployment"
    };
    let model=azimuth::model::Model {
        specs:vec![azimuth::spec::parse_spec("spec.md",&format!("# Spec: sample\n\n## Requirement: restricted\nVerification: reviewed{extra_header}\n\nAccess SHALL be restricted.\n\n### Scenario: works\nEvent: request\nRequired: restricted access\n")).unwrap()],
        check_implementations:vec![azimuth::model::CheckImplementation {
            check:"recovery".into(), site:"recovery::replay".into(), file:"src/recovery.rs".into(), lang:"rust".into(),
            source:Some(azimuth::model::SourceIdentity { area:"payments".into(), kind:"rust-symbol".into(), address:"recovery::replay".into(), mount:"code".into() }),
            source_fingerprint:fp('c'),
        }],
        account_verifications:vec![azimuth::account_verification::parse_account_verification("verification.md",&format!("# Verification: sample\n\n## Requirement verification: restricted\n\n### Check: recovery\n- Deployment: {policy}\n- Evidence bindings:\n  - Scenario: `works`\n    - Contribution: Establishes access restrictions.\n\nExercise access restrictions.\n")).unwrap()],
        ..Default::default()
    };
    let authority = value(&azimuth::assurance_review::authority(&model, "project").unwrap());
    let review = accepted_verification_review(&authority);
    let verification_target = json!({"kind":"deployment","environment":"production","deployment":deployment_id,"deployment_fingerprint":fp('c'),"artifacts":[{"id":"api","digest":fp('e')}]});
    let verification_target_fp = azimuth::run::verification_target_fingerprint(
        &azimuth::run::verification_target_from_json(&native(&verification_target)).unwrap(),
    );
    let deployment = json!({"id":deployment_id,"environment":"production","configuration_fingerprint":fp('c'),"artifacts":[fp('e')],"verification_target":verification_target,"verification_target_fingerprint":verification_target_fp});
    value(&azimuth::release::prepare(&native(&json!({
        "format":"azimuth-release-request","version":2,"project":"project","label":"Synthetic release",
        "changes":[{"id":"restricted","revision":fp('a')}],
        "sources":[{"repository":"root","commit":"a".repeat(40),"content_fingerprint":fp('a')}],
        "builds":[{"id":"build-api","sources":["root"],"toolchain":{"compiler":"1"},"configuration_fingerprint":fp('b'),"artifacts":[fp('e')]}],
        "artifacts":[{"digest":fp('e'),"kind":"container"}],"deployments":[deployment],"components":[{"id":"api","digest":fp('e')}],
        "snapshot":{"project":"project","model_revision":authority["fingerprint"],"authority":authority,"reviews":[review],"bundles":[]},"evidence":[]
    }))).unwrap())
}

// Satisfied requires an accepted current Verification Review, so the fixture publishes one.
fn accepted_verification_review(authority: &Value) -> Value {
    let mut review = json!({"kind":"verification-review","requirement":"restricted","requirement_fingerprint":authority["requirements"][0]["fingerprint"],
        "dependencies_fingerprint":azimuth::assurance_review::requirement_closure_fingerprint(&native(authority),"restricted").unwrap(),
        "decision":"accepted","reassesses":null,"challenge_basis":[],"reviewer":"reviewer","reviewed_at_ms":100,"rationale":"Reviewed the exact declared account.",
        "artifacts":[{"id":"notes","digest":format!("sha256:{}","a".repeat(64)),"locator":"reviews/notes.md"}]});
    review["fingerprint"] = json!(azimuth::assurance_review::record_fingerprint(&native(&review)).unwrap());
    review
}
fn context() -> BTreeMap<String, String> {
    BTreeMap::new()
}
fn valid_bundle() -> RunBundle {
    let check = CheckSelection {
        id: "recovery".into(),
        fingerprint: fp('b'),
        scenarios: vec!["works".into()],
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
        verification_target: VerificationTarget::Workspace {
            repositories: vec![RepositoryState {
                id: "root".into(),
                revision: "revision-a".into(),
                content_fingerprint: fp('a'),
            }],
        },
        verification_target_fingerprint: fp('0'),
        planned_at_ms: 1,
        started_at_ms: 2,
        finished_at_ms: 4,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: fp('d'),
            required_execution_context: context(),
            checks: vec![check.clone()],
            challenges: vec![],
            fingerprint: fp('0'),
        },
        actual_selection: ActualSelection {
            execution_context: context(),
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
                input_fingerprint: None,
                executed_tests: Vec::new(),
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "check-attempt".into(),
                    outcomes: [("works".into(), CheckResultOutcome::Satisfied)]
                        .into_iter()
                        .collect(),
                }],
            }],
            results: vec![CheckResult {
                scenario: "works".into(),
                outcome: CheckResultOutcome::Satisfied,
                observed_at_ms: 3,
                fingerprint: fp('0'),
                artifacts: vec![],
                diagnostics: vec![],
            }],
        }],
        verification_challenge_executions: vec![],
    };
    refresh(&mut bundle);
    bundle
}
fn refresh(bundle: &mut RunBundle) {
    bundle.verification_target_fingerprint = verification_target_fingerprint(&bundle.verification_target);
    bundle.plan.fingerprint = plan_fingerprint(&bundle.verification_target_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    bundle.provenance.adapter.launch_fingerprint = launch_fingerprint(
        bundle.provenance.mode,
        bundle.planned_at_ms,
        &bundle.verification_target,
        &bundle.verification_target_fingerprint,
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
        for result_index in 0..bundle.check_executions[index].results.len() {
            let fingerprint = check_result_fingerprint(
                bundle,
                &bundle.check_executions[index],
                &bundle.check_executions[index].results[result_index],
            );
            bundle.check_executions[index].results[result_index].fingerprint =
                fingerprint;
        }
    }
    for index in 0..bundle.verification_challenge_executions.len() {
        bundle.verification_challenge_executions[index].result.fingerprint =
            challenge_result_fingerprint(bundle, &bundle.verification_challenge_executions[index]);
    }
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}

