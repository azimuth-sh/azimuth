use azimuth::{
    deployment_status,
    json::{self, Json},
    run::*,
};
use std::collections::BTreeMap;
fn fp(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
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
fn set(v: &mut Json, key: &str, value: Json) {
    let Json::Obj(fields) = v else {
        panic!("object expected")
    };
    if let Some((_, v)) = fields.iter_mut().find(|(k, _)| k == key) {
        *v = value;
    } else {
        fields.push((key.into(), value));
    }
}
fn policy(subject: &str) -> Json {
    Json::obj(vec![
        ("subject", Json::str(subject)),
        ("components", Json::Arr(vec![Json::str("api")])),
        ("contexts", Json::Arr(vec![])),
        ("contribution", Json::str("both")),
        ("monitor", Json::Null),
    ])
}
fn request(policy: Json) -> Json {
    request_with_criticality(policy, "routine")
}
fn request_with_criticality(policy: Json, criticality: &str) -> Json {
    let model=azimuth::model::Model{
  specs:vec![azimuth::spec::parse_spec("spec.md",&"# Spec: sample\n\n## Claim: restricted-access\nCriticality: routine\n\nAccess SHALL be restricted.\n\n### Case: works\nEvent: a request arrives\nRequired: access is restricted\n".replace("Criticality: routine", &format!("Criticality: {criticality}"))).unwrap()],
  account_verifications:vec![azimuth::account_verification::parse_account_verification("verification.md",&format!("# Verification: sample\n\n## Claim verification: restricted-access\n\n### Check: recovery\n- Deployment: {}\n- Evidence bindings:\n  - Case: `works`\n    - Contribution: Establishes restricted access.\n\nExercise restricted access.\n",policy.to_string_pretty().replace('\n'," "))).unwrap()],
  check_implementations:vec![azimuth::model::CheckImplementation {check:"recovery".into(),site:"access::check".into(),file:"tests/access.rs".into(),lang:"rust".into(),source:Some(azimuth::model::SourceIdentity {area:"sample".into(),kind:"rust-symbol".into(),address:"access::check".into(),mount:"root".into()}),source_fingerprint:fp('c')}],
  realizes:vec![azimuth::model::Site {spec:"sample".into(),claim:"restricted-access".into(),site:"access::enforce".into(),file:"src/access.rs".into(),lang:"rust".into(),source:Some(azimuth::model::SourceIdentity {area:"sample".into(),kind:"rust-symbol".into(),address:"access::enforce".into(),mount:"root".into()}),source_fingerprint:fp('d')}],
  workspace:azimuth::workspace::Workspace {realization_obligations:vec![azimuth::workspace::RealizationObligation{spec:"sample".into(),claim:"restricted-access".into(),areas:vec!["sample".into()]}],..Default::default()},
  ..Default::default()
 };
    let authority = azimuth::assurance_review::authority(&model, "sample-project").unwrap();
    assert!(authority.get("checks").unwrap().as_array().unwrap()[0]
        .get("deployment")
        .is_some());
    let snapshot = Json::obj(vec![
        ("project", Json::str("sample-project")),
        (
            "model_revision",
            authority.get("fingerprint").unwrap().clone(),
        ),
        ("authority", authority),
        ("reviews", Json::Arr(vec![])),
        ("bundles", Json::Arr(vec![])),
    ]);
    let release=azimuth::release::prepare(&Json::obj(vec![
  ("format",Json::str("azimuth-release-request")),("version",Json::Num(1.0)),("project",Json::str("sample-project")),("label",Json::str("Synthetic release")),("baseline_model_revision",Json::str(fp('a'))),
  ("changes",json::parse(&format!(r#"[{{"id":"restricted-access","revision":"{}"}}]"#,fp('a'))).unwrap()),
  ("sources",json::parse(&format!(r#"[{{"repository":"root","commit":"{}","content_fingerprint":"{}"}}]"#,"a".repeat(40),fp('a'))).unwrap()),
  ("builds",json::parse(&format!(r#"[{{"id":"build-api","sources":["root"],"toolchain":{{"compiler":"1"}},"configuration_fingerprint":"{}","artifacts":["{}"]}}]"#,fp('b'),fp('e'))).unwrap()),
  ("artifacts",json::parse(&format!(r#"[{{"digest":"{}","kind":"container"}}]"#,fp('e'))).unwrap()),
  ("deployments",Json::Arr(vec![])),("components",json::parse(&format!(r#"[{{"id":"api","digest":"{}"}}]"#,fp('e'))).unwrap()),("snapshot",snapshot),("evidence",Json::Arr(vec![]))
 ])).unwrap();
    let subject = Subject::Deployment {
        environment: "production".into(),
        deployment: "deploy-1".into(),
        deployment_fingerprint: fp('c'),
        artifacts: vec![ArtifactState {
            id: "api".into(),
            digest: fp('e'),
        }],
    };
    let subject_fp = subject_fingerprint(&subject);
    let deployment = Json::obj(vec![
        ("id", Json::str("deploy-1")),
        ("environment", Json::str("production")),
        ("configuration_fingerprint", Json::str(fp('c'))),
        ("artifacts", Json::Arr(vec![Json::str(fp('e'))])),
        ("subject", subject_to_json(&subject)),
        ("subject_fingerprint", Json::str(&subject_fp)),
    ]);
    let observation = Json::obj(vec![
        ("subject_fingerprint", Json::str(subject_fp)),
        ("observed_at_ms", Json::Num(10_000.0)),
        ("valid_for_seconds", Json::Num(300.0)),
        ("state", Json::str("complete")),
        (
            "components",
            json::parse(&format!(
                r#"[{{"id":"api","digest":"{}","instances":["process-1"]}}]"#,
                fp('e')
            ))
            .unwrap(),
        ),
        ("trusted", Json::Bool(true)),
    ]);
    Json::obj(vec![
        ("format", Json::str("azimuth-deployment-assessment")),
        ("version", Json::Num(1.0)),
        ("publication", release),
        ("deployment", deployment),
        ("deployment_observation", observation),
        ("context_contracts", Json::Arr(vec![])),
        ("captures", Json::Arr(vec![])),
        ("bundles", Json::Arr(vec![])),
        ("reviews", Json::Arr(vec![])),
        ("evaluated_at_ms", Json::Num(10_000.0)),
    ])
}
fn bundle_for(
    request: &Json,
    subject: Subject,
    time: u64,
    outcome: ObservationOutcome,
) -> RunBundle {
    let authority = request
        .get("publication")
        .unwrap()
        .get("snapshot")
        .unwrap()
        .get("authority")
        .unwrap();
    let check_fp = authority.get("checks").unwrap().as_array().unwrap()[0]
        .get("fingerprint")
        .unwrap()
        .as_str()
        .unwrap();
    let mut bundle = valid_bundle();
    bundle.subject = subject;
    bundle.plan.model_fingerprint = authority
        .get("model_fingerprint")
        .unwrap()
        .as_str()
        .unwrap()
        .into();
    bundle.plan.checks[0].fingerprint = check_fp.into();
    bundle.actual_selection.checks[0].fingerprint = check_fp.into();
    bundle.plan.checks[0].implementations[0].identity = "sample|rust-symbol|access::check".into();
    bundle.actual_selection.checks[0].implementations =
        bundle.plan.checks[0].implementations.clone();
    bundle.check_executions[0].check.fingerprint = check_fp.into();
    bundle.planned_at_ms = time - 2;
    bundle.started_at_ms = time - 1;
    bundle.finished_at_ms = time + 1;
    bundle.provenance.generated_at_ms = time + 2;
    bundle.provenance.source.execution = format!("execution-{time}");
    bundle.activities[0].started_at_ms = time - 1;
    bundle.activities[0].finished_at_ms = time;
    bundle.check_executions[0].observations[0].observed_at_ms = time;
    bundle.check_executions[0].observations[0].outcome = outcome;
    bundle.check_executions[0].units[0].attempts[0]
        .outcomes
        .insert("works".into(), outcome);
    refresh(&mut bundle);
    assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
    bundle
}
fn source() -> Subject {
    Subject::Source {
        repositories: vec![RepositoryState {
            id: "root".into(),
            revision: "a".repeat(40),
            content_fingerprint: fp('a'),
        }],
    }
}
fn status(request: &Json) -> String {
    let out = deployment_status::assess(request).unwrap();
    out.get("claims").unwrap().as_array().unwrap()[0]
        .get("status")
        .unwrap()
        .as_str()
        .unwrap()
        .into()
}
fn attach(request: &mut Json, bundles: &[RunBundle]) {
    set(
        request,
        "bundles",
        Json::Arr(bundles.iter().map(to_json).collect()),
    );
}
#[test]
fn source_evidence_supports_exact_deployment_without_source_status() {
    let mut request = request(policy("source"));
    assert_eq!(status(&request), "unresolved");
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "supported");
    let output = deployment_status::assess(&request).unwrap();
    let explanation = output.get("claims").unwrap().as_array().unwrap()[0]
        .get("explanation")
        .unwrap();
    assert!(
        explanation
            .get("coverage")
            .unwrap()
            .as_array()
            .unwrap()
            .len()
            == 1
    );
    assert_eq!(
        explanation
            .get("review")
            .unwrap()
            .get("status")
            .unwrap()
            .as_str(),
        Some("not-required")
    );
}
#[test]
fn wrong_source_or_artifact_cannot_support_deployment() {
    for kind in ["source", "artifact"] {
        let mut request = request(policy(kind));
        let subject = if kind == "source" {
            Subject::Source {
                repositories: vec![RepositoryState {
                    id: "root".into(),
                    revision: "b".repeat(40),
                    content_fingerprint: fp('a'),
                }],
            }
        } else {
            Subject::Artifact {
                artifacts: vec![ArtifactState {
                    id: "api".into(),
                    digest: fp('f'),
                }],
            }
        };
        let bundle = bundle_for(&request, subject, 100, ObservationOutcome::Satisfied);
        attach(&mut request, &[bundle]);
        assert_eq!(status(&request), "unresolved");
    }
}
#[test]
fn monitor_latest_inconclusive_and_equal_time_conflict_never_fall_back() {
    let mut p = policy("deployment");
    set(
        &mut p,
        "monitor",
        json::parse(r#"{"id":"access","interval_seconds":5,"freshness_seconds":15}"#).unwrap(),
    );
    let mut request = request(p);
    let subject =
        subject_from_json(request.get("deployment").unwrap().get("subject").unwrap()).unwrap();
    let passed = bundle_for(
        &request,
        subject.clone(),
        9_000,
        ObservationOutcome::Satisfied,
    );
    let newer = bundle_for(
        &request,
        subject.clone(),
        9_500,
        ObservationOutcome::Inconclusive,
    );
    attach(&mut request, &[passed.clone()]);
    assert_eq!(status(&request), "supported");
    attach(&mut request, &[passed, newer]);
    assert_eq!(status(&request), "unresolved");
    let left = bundle_for(
        &request,
        subject.clone(),
        9_600,
        ObservationOutcome::Satisfied,
    );
    let mut right = bundle_for(&request, subject, 9_600, ObservationOutcome::Violated);
    right.provenance.source.execution = "independent-execution".into();
    refresh(&mut right);
    attach(&mut request, &[left, right]);
    assert_eq!(status(&request), "unresolved");
}
#[test]
fn expired_monitor_and_fresh_deployment_do_not_confirm() {
    let mut p = policy("deployment");
    set(
        &mut p,
        "monitor",
        json::parse(r#"{"id":"access","interval_seconds":1,"freshness_seconds":2}"#).unwrap(),
    );
    let mut request = request(p);
    let subject =
        subject_from_json(request.get("deployment").unwrap().get("subject").unwrap()).unwrap();
    let bundle = bundle_for(&request, subject, 100, ObservationOutcome::Satisfied);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "unresolved");
}
#[test]
fn counterexample_is_visible_without_review_but_failure_is_not_automatically_violation() {
    let mut p = policy("source");
    set(&mut p, "contribution", Json::str("support"));
    let mut request = request(p);
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Violated);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "unresolved");
    let output = deployment_status::assess(&request).unwrap();
    assert_eq!(
        output.get("claims").unwrap().as_array().unwrap()[0]
            .get("explanation")
            .unwrap()
            .get("evidence")
            .unwrap()
            .get("violations")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn strict_contract_and_policy_validation_rejects_unknowns_and_secret_fields() {
    let mut p = policy("source");
    set(&mut p, "extra", Json::Bool(true));
    assert!(deployment_status::validate_policy(&p).is_err());
    let body = json::parse(r#"{"name":"settings","fields":{"password":"string"}}"#).unwrap();
    let mut contract = body.clone();
    set(
        &mut contract,
        "fingerprint",
        Json::str(canonical_fingerprint(&body).unwrap()),
    );
    assert!(deployment_status::validate_context_contract(&contract).is_err());
}

fn rehash_release(request: &mut Json) {
    let publication = match request {
        Json::Obj(fields) => {
            &mut fields
                .iter_mut()
                .find(|(k, _)| k == "publication")
                .unwrap()
                .1
        }
        _ => unreachable!(),
    };
    let release = match publication {
        Json::Obj(fields) => &mut fields.iter_mut().find(|(k, _)| k == "release").unwrap().1,
        _ => unreachable!(),
    };
    let Json::Obj(fields) = release else {
        unreachable!()
    };
    let body = Json::Obj(
        fields
            .iter()
            .filter(|(k, _)| k != "fingerprint")
            .cloned()
            .collect(),
    );
    set(
        release,
        "fingerprint",
        Json::str(canonical_fingerprint(&body).unwrap()),
    );
}
fn mutate_release(request: &mut Json, f: impl FnOnce(&mut Json)) {
    let Json::Obj(fields) = request else {
        unreachable!()
    };
    let Json::Obj(publication) = &mut fields
        .iter_mut()
        .find(|(k, _)| k == "publication")
        .unwrap()
        .1
    else {
        unreachable!()
    };
    f(&mut publication
        .iter_mut()
        .find(|(k, _)| k == "release")
        .unwrap()
        .1);
    rehash_release(request);
}
#[test]
fn source_scope_ignores_unrelated_release_sources_but_requires_all_build_inputs() {
    let mut request = request(policy("source"));
    mutate_release(&mut request, |release| {
        let mut sources = release.get("sources").unwrap().as_array().unwrap().to_vec();
        sources.push(
            json::parse(&format!(
                r#"{{"repository":"other","commit":"{}","content_fingerprint":"{}"}}"#,
                "b".repeat(40),
                fp('b')
            ))
            .unwrap(),
        );
        set(release, "sources", Json::Arr(sources));
    });
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "supported");
    mutate_release(&mut request, |release| {
        let mut builds = release.get("builds").unwrap().as_array().unwrap().to_vec();
        set(
            &mut builds[0],
            "sources",
            Json::Arr(vec![Json::str("root"), Json::str("other")]),
        );
        set(release, "builds", Json::Arr(builds));
    });
    assert_eq!(status(&request), "unresolved");
}
#[test]
fn conflicting_producers_do_not_establish_source_lineage() {
    let mut request = request(policy("source"));
    mutate_release(&mut request, |release| {
        let mut sources = release.get("sources").unwrap().as_array().unwrap().to_vec();
        sources.push(
            json::parse(&format!(
                r#"{{"repository":"other","commit":"{}","content_fingerprint":"{}"}}"#,
                "b".repeat(40),
                fp('b')
            ))
            .unwrap(),
        );
        set(release, "sources", Json::Arr(sources));
        let mut builds = release.get("builds").unwrap().as_array().unwrap().to_vec();
        let mut other = builds[0].clone();
        set(&mut other, "id", Json::str("other-build"));
        set(&mut other, "sources", Json::Arr(vec![Json::str("other")]));
        builds.push(other);
        set(release, "builds", Json::Arr(builds));
    });
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "unresolved");
}
#[test]
fn corrections_replace_support_and_stale_deployment_observation_does_not_renew_it() {
    let mut request = request(policy("source"));
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    let mut corrected = bundle.clone();
    corrected.bundle_revision = 1;
    corrected.corrects = Some(bundle.bundle_fingerprint.clone());
    corrected.correction_reason = Some("Correct report classification".into());
    corrected.check_executions[0].units[0].attempts[0]
        .outcomes
        .insert("works".into(), ObservationOutcome::Inconclusive);
    corrected.check_executions[0].observations[0].outcome = ObservationOutcome::Inconclusive;
    refresh(&mut corrected);
    attach(&mut request, &[bundle, corrected]);
    assert_eq!(status(&request), "unresolved");
    let mut stale = request.get("deployment_observation").unwrap().clone();
    set(&mut stale, "observed_at_ms", Json::Num(0.0));
    set(&mut stale, "valid_for_seconds", Json::Num(1.0));
    set(&mut request, "deployment_observation", stale);
    assert_eq!(status(&request), "unresolved");
}
#[test]
fn missing_review_blocks_support_without_hiding_declared_counterexample() {
    let mut request = request_with_criticality(policy("source"), "critical");
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "unresolved");
    let failed = bundle_for(&request, source(), 101, ObservationOutcome::Violated);
    attach(&mut request, &[failed]);
    assert_eq!(status(&request), "unresolved");
    let out = deployment_status::assess(&request).unwrap();
    assert_eq!(
        out.get("claims").unwrap().as_array().unwrap()[0]
            .get("explanation")
            .unwrap()
            .get("evidence")
            .unwrap()
            .get("violations")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        out.get("claims").unwrap().as_array().unwrap()[0]
            .get("review")
            .unwrap()
            .as_str(),
        Some("missing")
    );
}
fn context_contract() -> Json {
    let mut contract =
        json::parse(r#"{"name":"authentication","fields":{"enabled":"boolean"}}"#).unwrap();
    let fingerprint = canonical_fingerprint(&contract).unwrap();
    set(&mut contract, "fingerprint", Json::str(fingerprint));
    contract
}
fn context_report(contract: &Json, enabled: bool) -> Json {
    Json::obj(vec![
        ("format", Json::str("azimuth-verification-context")),
        ("version", Json::Num(1.0)),
        (
            "contexts",
            Json::obj(vec![(
                "authentication",
                Json::obj(vec![
                    (
                        "contract_revision",
                        contract.get("fingerprint").unwrap().clone(),
                    ),
                    ("values", Json::obj(vec![("enabled", Json::Bool(enabled))])),
                ]),
            )]),
        ),
    ])
}
fn capture(
    request: &Json,
    bundle: &RunBundle,
    contract: &Json,
    execution: bool,
    instance: &str,
    enabled: bool,
) -> Json {
    let (start, finish) = if execution {
        (bundle.started_at_ms - 1, bundle.finished_at_ms + 1)
    } else {
        (9_999, 10_000)
    };
    let snapshot = |time| {
        Json::obj(vec![
            ("captured_at_ms", Json::Num(time as f64)),
            ("instance", Json::str(instance)),
            ("report", context_report(contract, enabled)),
        ])
    };
    Json::obj(vec![
        (
            "kind",
            Json::str(if execution { "execution" } else { "deployment" }),
        ),
        (
            "run",
            if execution {
                Json::str(&bundle.run_id)
            } else {
                Json::Null
            },
        ),
        (
            "bundle_fingerprint",
            if execution {
                Json::str(&bundle.bundle_fingerprint)
            } else {
                Json::Null
            },
        ),
        (
            "subject_fingerprint",
            if execution {
                Json::str(&bundle.subject_fingerprint)
            } else {
                request
                    .get("deployment")
                    .unwrap()
                    .get("subject_fingerprint")
                    .unwrap()
                    .clone()
            },
        ),
        ("activity", Json::Null),
        ("component", Json::str("api")),
        ("instance", Json::str(instance)),
        ("started_at_ms", Json::Num(start as f64)),
        ("finished_at_ms", Json::Num(finish as f64)),
        ("before", snapshot(start)),
        ("after", snapshot(finish)),
        ("trusted", Json::Bool(true)),
    ])
}
fn context_fixture() -> (Json, RunBundle, Json, Vec<Json>) {
    let contract = context_contract();
    let mut p = policy("source");
    set(
        &mut p,
        "contexts",
        Json::Arr(vec![contract.get("fingerprint").unwrap().clone()]),
    );
    let mut request = request(p);
    set(
        &mut request,
        "context_contracts",
        Json::Arr(vec![contract.clone()]),
    );
    let mut bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    let report = context_report(&contract, true);
    let context = report
        .get("contexts")
        .unwrap()
        .get("authentication")
        .unwrap();
    bundle.plan.required_context.insert(
        format!(
            "context/{}",
            contract.get("fingerprint").unwrap().as_str().unwrap()
        ),
        canonical_fingerprint(context).unwrap(),
    );
    bundle.actual_selection.context = bundle.plan.required_context.clone();
    refresh(&mut bundle);
    attach(&mut request, &[bundle.clone()]);
    let captures = vec![
        capture(&request, &bundle, &contract, true, "test-process", true),
        capture(&request, &bundle, &contract, false, "process-1", true),
    ];
    set(&mut request, "captures", Json::Arr(captures.clone()));
    (request, bundle, contract, captures)
}
#[test]
fn effective_context_matches_exact_run_and_every_production_process() {
    let (mut request, bundle, contract, mut captures) = context_fixture();
    assert_eq!(status(&request), "supported");
    let mut observation = request.get("deployment_observation").unwrap().clone();
    set(
        &mut observation,
        "components",
        json::parse(&format!(
            r#"[{{"id":"api","digest":"{}","instances":["process-1","process-2"]}}]"#,
            fp('e')
        ))
        .unwrap(),
    );
    set(&mut request, "deployment_observation", observation);
    assert_eq!(status(&request), "unresolved");
    captures.push(capture(
        &request,
        &bundle,
        &contract,
        false,
        "process-2",
        false,
    ));
    set(&mut request, "captures", Json::Arr(captures.clone()));
    assert_eq!(status(&request), "unresolved");
    captures.pop();
    captures.push(capture(
        &request,
        &bundle,
        &contract,
        false,
        "process-2",
        true,
    ));
    set(&mut request, "captures", Json::Arr(captures));
    assert_eq!(status(&request), "supported");
}
#[test]
fn untrusted_changed_restarted_and_wrong_run_contexts_cannot_support() {
    let (request, _, contract, captures) = context_fixture();
    for variant in 0..4 {
        let mut request = request.clone();
        let mut captures = captures.clone();
        match variant {
            0 => set(&mut captures[1], "trusted", Json::Bool(false)),
            1 => {
                let mut after = captures[0].get("after").unwrap().clone();
                set(&mut after, "report", context_report(&contract, false));
                set(&mut captures[0], "after", after);
            }
            2 => set(&mut captures[0], "bundle_fingerprint", Json::str(fp('0'))),
            _ => {
                let mut after = captures[0].get("after").unwrap().clone();
                set(&mut after, "instance", Json::str("restarted-process"));
                set(&mut captures[0], "after", after);
            }
        }
        set(&mut request, "captures", Json::Arr(captures));
        if variant == 3 {
            assert!(deployment_status::assess(&request).is_err());
        } else {
            assert_eq!(status(&request), "unresolved");
        }
    }
}

fn accepted_review(request: &Json) -> Json {
    let authority = request
        .get("publication")
        .unwrap()
        .get("snapshot")
        .unwrap()
        .get("authority")
        .unwrap();
    let claim = &authority.get("claims").unwrap().as_array().unwrap()[0];
    let mut record = Json::obj(vec![
        ("kind", Json::str("claim-review")),
        ("reviewer", Json::str("independent-reviewer")),
        ("reviewed_at_ms", Json::Num(50.0)),
        (
            "rationale",
            Json::str(
                "The synthetic verification establishes the Case and its component/context scope.",
            ),
        ),
        (
            "artifacts",
            json::parse(&format!(
                r#"[{{"id":"review-notes","digest":"{}","locator":"reviews/notes.md"}}]"#,
                fp('a')
            ))
            .unwrap(),
        ),
        ("claim", claim.get("id").unwrap().clone()),
        (
            "claim_fingerprint",
            claim.get("fingerprint").unwrap().clone(),
        ),
        (
            "dependencies",
            Json::obj(vec![
                ("claims", authority.get("claims").unwrap().clone()),
                ("checks", authority.get("checks").unwrap().clone()),
                ("cases", authority.get("cases").unwrap().clone()),
                ("bindings", authority.get("bindings").unwrap().clone()),
            ]),
        ),
        ("decision", Json::str("accepted")),
    ]);
    let fingerprint = azimuth::assurance_review::record_fingerprint(&record).unwrap();
    set(&mut record, "fingerprint", Json::str(fingerprint));
    azimuth::assurance_review::validate_record(&record).unwrap();
    record
}
#[test]
fn current_independent_review_enables_confirmation_and_policy_change_stales_it() {
    let mut request = request_with_criticality(policy("source"), "critical");
    let review = accepted_review(&request);
    set(&mut request, "reviews", Json::Arr(vec![review.clone()]));
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "supported");
    let mut changed_policy = policy("source");
    set(&mut changed_policy, "contribution", Json::str("support"));
    let mut changed = request_with_criticality(changed_policy, "critical");
    set(&mut changed, "reviews", Json::Arr(vec![review]));
    let bundle = bundle_for(&changed, source(), 100, ObservationOutcome::Satisfied);
    attach(&mut changed, &[bundle]);
    assert_eq!(status(&changed), "unresolved");
    let out = deployment_status::assess(&changed).unwrap();
    assert_eq!(
        out.get("claims").unwrap().as_array().unwrap()[0]
            .get("review")
            .unwrap()
            .as_str(),
        Some("stale")
    );
}
#[test]
fn incomplete_new_monitor_execution_does_not_leave_previous_green_result_current() {
    let mut p = policy("deployment");
    set(
        &mut p,
        "monitor",
        json::parse(r#"{"id":"access","interval_seconds":5,"freshness_seconds":15}"#).unwrap(),
    );
    let mut request = request(p);
    let subject =
        subject_from_json(request.get("deployment").unwrap().get("subject").unwrap()).unwrap();
    let previous = bundle_for(
        &request,
        subject.clone(),
        9_000,
        ObservationOutcome::Satisfied,
    );
    let mut failed = bundle_for(&request, subject, 9_500, ObservationOutcome::Inconclusive);
    failed.status = RunStatus::Partial;
    refresh(&mut failed);
    attach(&mut request, &[previous, failed]);
    assert_eq!(status(&request), "unresolved");
}

#[test]
fn startup_fixed_context_and_deployment_observer_need_not_collect_simultaneously() {
    let (mut request, _, _, mut captures) = context_fixture();
    set(&mut captures[1], "started_at_ms", Json::Num(9_990.0));
    set(&mut captures[1], "finished_at_ms", Json::Num(9_991.0));
    for (key, time) in [("before", 9_990.0), ("after", 9_991.0)] {
        let mut snapshot = captures[1].get(key).unwrap().clone();
        set(&mut snapshot, "captured_at_ms", Json::Num(time));
        set(&mut captures[1], key, snapshot);
    }
    set(&mut request, "captures", Json::Arr(captures));
    assert_eq!(status(&request), "supported");
}

#[test]
fn unrelated_model_revision_does_not_invalidate_current_check_evidence() {
    let mut request = request(policy("source"));
    let mut bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    bundle.plan.model_fingerprint = fp('9');
    refresh(&mut bundle);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "supported");
}
#[test]
fn monitor_identity_cannot_be_shared_by_different_checks() {
    let mut p = policy("deployment");
    set(
        &mut p,
        "monitor",
        json::parse(r#"{"id":"access","interval_seconds":5,"freshness_seconds":15}"#).unwrap(),
    );
    let authority = Json::obj(vec![(
        "checks",
        Json::Arr(vec![
            Json::obj(vec![("id", Json::str("one")), ("deployment", p.clone())]),
            Json::obj(vec![("id", Json::str("two")), ("deployment", p)]),
        ]),
    )]);
    assert!(deployment_status::validate_policies(&authority).is_err());
}

#[test]
fn unrelated_context_sidecars_do_not_invalidate_required_context() {
    let (mut request, _, contract, mut captures) = context_fixture();
    let mut unrelated =
        json::parse(r#"{"name":"features","fields":{"enabled":"boolean"}}"#).unwrap();
    let revision = canonical_fingerprint(&unrelated).unwrap();
    set(&mut unrelated, "fingerprint", Json::str(&revision));
    set(
        &mut request,
        "context_contracts",
        Json::Arr(vec![contract, unrelated]),
    );
    let report = json::parse(&format!(r#"{{"format":"azimuth-verification-context","version":1,"contexts":{{"features":{{"contract_revision":"{}","values":{{"enabled":true}}}}}}}}"#, revision)).unwrap();
    let mut sidecar = captures[0].clone();
    for phase in ["before", "after"] {
        let mut snapshot = sidecar.get(phase).unwrap().clone();
        set(&mut snapshot, "report", report.clone());
        set(&mut sidecar, phase, snapshot);
    }
    captures.push(sidecar);
    set(&mut request, "captures", Json::Arr(captures));
    assert_eq!(status(&request), "supported");
}

#[test]
fn per_activity_captures_cover_distinct_fixture_lifetimes_and_ignore_baselines() {
    let (mut request, mut bundle, contract, _) = context_fixture();
    let mut second = bundle.activities[0].clone();
    second.id = "second-fixture".into();
    second.started_at_ms = 101;
    second.finished_at_ms = 102;
    bundle.activities.push(second);
    let mut unit = bundle.check_executions[0].units[0].clone();
    unit.id = "second".into();
    unit.attempts[0].activity = "second-fixture".into();
    bundle.check_executions[0].units.push(unit);
    let mut planned = bundle.plan.checks[0].units[0].clone();
    planned.id = "second".into();
    bundle.plan.checks[0].units.push(planned);
    bundle.plan.checks[0].units.sort_by(|a, b| a.id.cmp(&b.id));
    bundle.check_executions[0]
        .units
        .sort_by(|a, b| a.id.cmp(&b.id));
    bundle.actual_selection.checks = bundle.plan.checks.clone();
    bundle.finished_at_ms = 103;
    bundle.provenance.generated_at_ms = 104;
    bundle.check_executions[0].observations[0].observed_at_ms = 102;
    let value = bundle
        .plan
        .required_context
        .values()
        .next()
        .unwrap()
        .clone();
    bundle.plan.required_context.clear();
    for activity in &bundle.activities {
        bundle.plan.required_context.insert(
            format!(
                "context/{}/activity/{}",
                contract.get("fingerprint").unwrap().as_str().unwrap(),
                activity.id
            ),
            value.clone(),
        );
    }
    bundle.actual_selection.context = bundle.plan.required_context.clone();
    refresh(&mut bundle);
    assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
    attach(&mut request, &[bundle.clone()]);
    let mut captures = vec![capture(
        &request,
        &bundle,
        &contract,
        false,
        "process-1",
        true,
    )];
    for activity in &bundle.activities {
        let mut current = capture(&request, &bundle, &contract, true, &activity.id, true);
        set(&mut current, "activity", Json::str(&activity.id));
        for (key, time) in [
            ("before", activity.started_at_ms),
            ("after", activity.finished_at_ms),
        ] {
            let mut snapshot = current.get(key).unwrap().clone();
            set(&mut snapshot, "captured_at_ms", Json::Num(time as f64));
            set(&mut current, key, snapshot);
        }
        set(
            &mut current,
            "started_at_ms",
            Json::Num(activity.started_at_ms as f64),
        );
        set(
            &mut current,
            "finished_at_ms",
            Json::Num(activity.finished_at_ms as f64),
        );
        captures.push(current);
    }
    set(&mut request, "captures", Json::Arr(captures.clone()));
    assert_eq!(status(&request), "supported");
    let mut baseline = capture(
        &request,
        &bundle,
        &contract,
        true,
        "unrelated-baseline",
        false,
    );
    set(&mut baseline, "activity", Json::str("unused-baseline"));
    captures.push(baseline);
    set(&mut request, "captures", Json::Arr(captures.clone()));
    assert_eq!(status(&request), "supported");
    captures.remove(1);
    set(&mut request, "captures", Json::Arr(captures));
    assert_eq!(status(&request), "unresolved");
}

#[test]
fn reviewed_controls_require_complete_units_and_cannot_alone_support() {
    let contract = context_contract();
    let fp = contract.get("fingerprint").unwrap().as_str().unwrap();
    let mut p = policy("source");
    set(&mut p, "contexts", Json::Arr(vec![Json::str(fp)]));
    set(
        &mut p,
        "units",
        Json::Arr(vec![Json::str("control"), Json::str("whole")]),
    );
    set(
        &mut p,
        "controls",
        Json::Arr(vec![Json::obj(vec![
            ("unit", Json::str("control")),
            ("contexts", Json::Arr(vec![Json::str(fp)])),
        ])]),
    );
    let mut request = request(p);
    set(
        &mut request,
        "context_contracts",
        Json::Arr(vec![contract.clone()]),
    );
    let mut bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    let mut control = bundle.check_executions[0].units[0].clone();
    control.id = "control".into();
    control.attempts[0].activity = "control-activity".into();
    bundle.check_executions[0].units.insert(0, control);
    let mut planned = bundle.plan.checks[0].units[0].clone();
    planned.id = "control".into();
    bundle.plan.checks[0].units.insert(0, planned);
    bundle.actual_selection.checks = bundle.plan.checks.clone();
    let mut activity = bundle.activities[0].clone();
    activity.id = "control-activity".into();
    bundle.activities.push(activity);
    bundle.plan.required_context.clear();
    for (activity, enabled) in [("check-attempt", true), ("control-activity", false)] {
        let report = context_report(&contract, enabled);
        let value = report
            .get("contexts")
            .unwrap()
            .get("authentication")
            .unwrap();
        bundle.plan.required_context.insert(
            format!("context/{fp}/activity/{activity}"),
            canonical_fingerprint(value).unwrap(),
        );
    }
    bundle.actual_selection.context = bundle.plan.required_context.clone();
    refresh(&mut bundle);
    assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
    attach(&mut request, &[bundle.clone()]);
    let mut captures = vec![capture(
        &request,
        &bundle,
        &contract,
        false,
        "process-1",
        true,
    )];
    for (activity, enabled) in [("check-attempt", true), ("control-activity", false)] {
        let mut current = capture(&request, &bundle, &contract, true, activity, enabled);
        set(&mut current, "activity", Json::str(activity));
        captures.push(current);
    }
    set(&mut request, "captures", Json::Arr(captures));
    assert_eq!(status(&request), "supported");
    bundle.check_executions[0].units.pop();
    bundle.plan.checks[0].units.pop();
    bundle.actual_selection.checks = bundle.plan.checks.clone();
    refresh(&mut bundle);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "unresolved");
}

#[test]
fn configuration_sources_require_exact_released_repository_versions() {
    let mut p = policy("source");
    set(
        &mut p,
        "configuration_sources",
        Json::Arr(vec![Json::str("gitops")]),
    );
    let mut request = request(p);
    mutate_release(&mut request, |release| {
        let mut sources = release.get("sources").unwrap().as_array().unwrap().to_vec();
        sources.push(
            json::parse(&format!(
                r#"{{"repository":"gitops","commit":"{}","content_fingerprint":"{}"}}"#,
                "b".repeat(40),
                fp('b')
            ))
            .unwrap(),
        );
        set(release, "sources", Json::Arr(sources));
    });
    let bundle = bundle_for(&request, source(), 100, ObservationOutcome::Satisfied);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "unresolved");
    let Subject::Source { mut repositories } = source() else {
        unreachable!()
    };
    repositories.push(RepositoryState {
        id: "gitops".into(),
        revision: "b".repeat(40),
        content_fingerprint: fp('b'),
    });
    repositories.sort_by(|a, b| a.id.cmp(&b.id));
    let mut bundle = bundle_for(
        &request,
        Subject::Source { repositories },
        101,
        ObservationOutcome::Satisfied,
    );
    attach(&mut request, &[bundle.clone()]);
    assert_eq!(status(&request), "supported");
    let Subject::Source { repositories } = &mut bundle.subject else {
        unreachable!()
    };
    repositories[0].revision = "c".repeat(40);
    refresh(&mut bundle);
    attach(&mut request, &[bundle]);
    assert_eq!(status(&request), "unresolved");
}
