#[allow(dead_code)]
#[path = "../src/adapter.rs"]
mod adapter;
mod diag {
    pub use azimuth::diag::*;
}
mod fingerprint {
    pub use azimuth::fingerprint::*;
}
mod json {
    pub use azimuth::json::*;
}
mod model {
    pub use azimuth::model::*;
}
mod run {
    pub use azimuth::run::*;
}
mod validation {
    pub use azimuth::validation::*;
}
mod verification {
    pub use azimuth::verification::*;
}

#[allow(dead_code)]
#[path = "../src/run_plan.rs"]
mod run_plan;

use adapter::{
    AdapterConfiguration, AdapterContent, AdapterEnvironment, AdapterLimits, Capability,
    CapabilityClass, ConfiguredAdapter, ConfiguredFile,
};
use azimuth::design::{Design, DesignEntry, Enforcement, Mechanism, Target};
use azimuth::workspace::{Area, Mount, Surface, SurfaceContribution, Workspace};
use model::{
    Artifact, CheckImplementation, ClassMember, Enumeration, MechanismImplementation, Model, Site,
    SourceIdentity,
};
use run::{ArtifactState, Plan, Subject, WorkUnit};
use run_plan::*;
use spec::parse_spec;
use std::collections::BTreeMap;
use std::path::PathBuf;
use verification::{
    parse_standards, parse_verification, Check, EvidenceBinding, MethodQualification,
    MethodQualificationVerdict, Selector, SemanticScopeKind, Verification,
};

mod spec {
    pub use azimuth::spec::*;
}

fn fp(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn unit(id: &str) -> WorkUnit {
    WorkUnit {
        id: id.into(),
        parameters: BTreeMap::new(),
    }
}

fn check(id: &str) -> Check {
    Check {
        id: id.into(),
        methods: vec!["synthetic".into()],
        terminal: "the synthetic predicate is true".into(),
        rationale: String::new(),
        path: "verification.md".into(),
        line: 1,
    }
}

fn implementation(id: &str, address: &str) -> CheckImplementation {
    CheckImplementation {
        check: id.into(),
        site: address.into(),
        file: "src/check.rs".into(),
        lang: "rust-symbol".into(),
        source: Some(SourceIdentity {
            area: "core".into(),
            kind: "rust-symbol".into(),
            address: address.into(),
            mount: "root".into(),
        }),
        source_fingerprint: fp('a'),
    }
}

fn model(checks: &[&str]) -> Model {
    let spec = parse_spec(
        "synthetic/spec.md",
        "# Spec: synthetic\n\n## Claim: works\nCriticality: standard\n\nThe synthetic subject SHALL work.\n\n### Case: works\nWHEN checked\nTHEN it works\n",
    )
    .unwrap();
    Model {
        verifications: vec![Verification {
            owner: "root".into(),
            path: "verification.md".into(),
            checks: checks.iter().map(|id| check(id)).collect(),
            bindings: checks
                .iter()
                .map(|id| EvidenceBinding {
                    id: format!("bindings/{}", id.rsplit('/').next().unwrap()),
                    check: (*id).into(),
                    case: "synthetic#works/works".into(),
                    method_qualification: format!(
                        "qualifications/{}",
                        id.rsplit('/').next().unwrap()
                    ),
                    proposition: "the Check bears on the Case".into(),
                    context: BTreeMap::new(),
                    challenge_domain: Vec::new(),
                    policy: "credible".into(),
                    rationale: String::new(),
                    path: "verification.md".into(),
                    line: 1,
                })
                .collect(),
            method_qualifications: Vec::new(),
            applicability_decisions: Vec::new(),
            claim_judgments: Vec::new(),
            challengers: Vec::new(),
            challenge_plans: Vec::new(),
        }],
        check_implementations: checks
            .iter()
            .map(|id| implementation(id, &format!("checks::{}", id.rsplit('/').next().unwrap())))
            .collect(),
        specs: vec![spec],
        ..Default::default()
    }
}

fn capability(id: &str, classes: Vec<CapabilityClass>, seed: char) -> Capability {
    Capability {
        id: id.into(),
        classes,
        challenge_forms: Vec::new(),
        semantic_settings: BTreeMap::new(),
        fingerprint: fp(seed),
    }
}

fn configuration() -> AdapterConfiguration {
    AdapterConfiguration {
        path: PathBuf::from("azimuth/adapters.json"),
        directory: PathBuf::from("azimuth"),
        adapters: vec![ConfiguredAdapter {
            id: "demo".into(),
            provider_family: "synthetic/demo".into(),
            protocol_version: 1,
            adapter_version: "1".into(),
            build: "b1".into(),
            content: AdapterContent {
                executable: ConfiguredFile {
                    locator: "adapter".into(),
                    resolved: PathBuf::from("adapter"),
                    digest: fp('1'),
                },
                resources: Vec::new(),
            },
            semantic_settings: BTreeMap::new(),
            environment: AdapterEnvironment {
                literals: BTreeMap::new(),
            },
            limits: AdapterLimits {
                timeout_ms: 1000,
                stdout_bytes: 4096,
                stderr_bytes: 1024,
            },
            capabilities: vec![
                capability("alpha", vec![CapabilityClass::CheckExecute], '2'),
                capability("beta", vec![CapabilityClass::CheckExecute], '3'),
                capability("reports", vec![CapabilityClass::CheckImport], '4'),
                Capability {
                    id: "challenge".into(),
                    classes: vec![
                        CapabilityClass::ChallengeExecute,
                        CapabilityClass::ChallengeImport,
                    ],
                    challenge_forms: vec!["mutation".into()],
                    semantic_settings: BTreeMap::new(),
                    fingerprint: fp('9'),
                },
            ],
            adapter_fingerprint: fp('5'),
            descriptor_fingerprint: fp('6'),
            configuration_fingerprint: fp('7'),
        }],
    }
}

fn challenge_model() -> Model {
    let spec = parse_spec(
        "alpha/spec.md",
        "# Spec: alpha\n\n## Claim: behavior\nCriticality: standard\n\nThe system SHALL work.\n\n### Case: works\nWHEN invoked\nTHEN it works\n",
    )
    .unwrap();
    let verification = parse_verification(
        "alpha/verification.md",
        "# Verification: alpha\n\n## Check: alpha/check\nMethod: invoke\nTerminal: the behavior works\n\nAtomic.\n\n## Evidence Binding: alpha/edge\nCheck: alpha/check\nCase: alpha#behavior/works\nMethod qualification: alpha/method\nProposition: direct\nContext: {\"platform\":\"linux\"}\nChallenge domain: [\"check-implementation\"]\nPolicy: credible\n\nReviewable.\n\n## Method Qualification: alpha/method\nCheck: alpha/check\nScope: unit\nQuantification: example\nOracle: direct\nContext: {\"platform\":\"linux\"}\nChallenge domain: [\"check-implementation\"]\nPolicy: credible\nVerdict: qualified\nFingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nQualified: 2026-08-22\nQualifier: owner\n\nQualified.\n\n## Challenger: mutation/search\nForm: mutation\nSearches for: an undetected change\nRequired scope: [\"check-implementation\"]\n\nSearches exact semantics.\n\n## Challenge Plan: alpha/plan\nChallenger: mutation/search\nSelect: method-qualification from method-qualification alpha/method\n\nTargets the qualification.\n",
    )
    .unwrap();
    let standards = parse_standards(
        "standards.md",
        "# Decision policies and Challenge schedule\n\n## Decision Policy: credible\nRequired challenge: mutation\n\nThe composition must be challenged.\n\n## Challenge Schedule: current\nGate challenge: mutation\n\nMutation is gate work.\n",
    )
    .unwrap();
    let mut model = Model {
        specs: vec![spec],
        verifications: vec![verification],
        decision_standards: Some(standards),
        check_implementations: vec![implementation("alpha/check", "checks::alpha")],
        ..Default::default()
    };
    let expected = model
        .expected_method_qualification_fingerprint(&model.verifications[0].method_qualifications[0])
        .unwrap();
    model.verifications[0].method_qualifications[0].fingerprint = expected;
    model
}

fn rich_challenge_model() -> Model {
    let alpha = parse_spec(
        "alpha/spec.md",
        "# Spec: alpha\n\n## Claim: behavior\nCriticality: standard\n\nThe system SHALL work.\n\n### Case: works\nWHEN invoked\nTHEN it works\n",
    )
    .unwrap();
    let surface = parse_spec(
        "surface/spec.md",
        "# Spec: surface\n\n## Claim: routes\nCriticality: routine\n\nEvery route SHALL exist.\n\n### Case: tagged\nWHEN built\nTHEN it exists\n",
    )
    .unwrap();
    let verification = parse_verification(
        "alpha/verification.md",
        "# Verification: alpha\n\n## Check: alpha/check\nMethod: invoke\nTerminal: the behavior works\n\nAtomic.\n\n## Evidence Binding: alpha/edge\nCheck: alpha/check\nCase: alpha#behavior/works\nMethod qualification: alpha/method\nProposition: direct\nContext: {\"platform\":\"linux\"}\nChallenge domain: [\"realization\",\"mechanism\"]\nPolicy: credible\n\nReviewable.\n\n## Method Qualification: alpha/method\nCheck: alpha/check\nScope: unit\nQuantification: example\nOracle: direct\nContext: {\"platform\":\"linux\"}\nChallenge domain: [\"realization\",\"mechanism\"]\nPolicy: credible\nVerdict: qualified\nFingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nQualified: 2026-08-22\nQualifier: owner\n\nQualified.\n\n## Applicability Decision: alpha/edge\nVerdict: applicable\nFingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nDecided: 2026-08-22\nDecider: owner\n\nApplicable.\n\n## Claim Judgment: alpha#behavior\nVerdict: accepted\nPolicy: credible\nFingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nJudged: 2026-08-22\nJudge: owner\nBasis: the exact composition is accepted\nResidual risk: none identified\n\nAccepted.\n\n## Challenger: mutation/search\nForm: mutation\nSearches for: an undetected change\nRequired scope: [\"claim\"]\n\nSearches exact semantics.\n\n## Challenge Plan: alpha/plan\nChallenger: mutation/search\nSelect: claim-judgment from claim alpha#behavior\n\nTargets the total decision.\n",
    )
    .unwrap();
    let standards = parse_standards(
        "standards.md",
        "# Decision policies and Challenge schedule\n\n## Decision Policy: credible\nRequired challenge: mutation\n\nThe composition must be challenged.\n\n## Challenge Schedule: current\nGate challenge: mutation\n\nMutation is gate work.\n",
    )
    .unwrap();
    let source = |area: &str, mount: &str, kind: &str, address: &str| SourceIdentity {
        area: area.into(),
        mount: mount.into(),
        kind: kind.into(),
        address: address.into(),
    };
    let mut model = Model {
        specs: vec![alpha, surface],
        realizes: vec![
            Site {
                spec: "alpha".into(),
                claim: "behavior".into(),
                site: "alpha::works".into(),
                file: "src/alpha.rs".into(),
                lang: "rust".into(),
                source: Some(source("core", "code", "rust-item", "alpha::works")),
                source_fingerprint: fp('a'),
            },
            Site {
                spec: "surface".into(),
                claim: "routes".into(),
                site: "GET /tagged".into(),
                file: "app/tagged.ts".into(),
                lang: "typescript".into(),
                source: Some(source("web", "app", "route", "routes::tagged")),
                source_fingerprint: fp('b'),
            },
        ],
        mechanism_implementations: vec![MechanismImplementation {
            spec: "alpha".into(),
            mechanism: "worker".into(),
            binding: "artifact:worker".into(),
            site: "alpha::worker".into(),
            file: "src/worker.rs".into(),
            lang: "rust".into(),
            source: Some(source("core", "code", "rust-item", "alpha::worker")),
            source_fingerprint: fp('d'),
        }],
        check_implementations: vec![CheckImplementation {
            check: "alpha/check".into(),
            site: "checks::alpha".into(),
            file: "tests/alpha.rs".into(),
            lang: "rust".into(),
            source: Some(source("core", "code", "rust-item", "checks::alpha")),
            source_fingerprint: fp('c'),
        }],
        class_members: vec![ClassMember {
            class: "surface".into(),
            site: "GET /enumerated".into(),
            file: "app/enumerated.ts".into(),
            lang: "typescript".into(),
            source: Some(source(
                "web",
                "app",
                "class-member",
                "surface#GET /enumerated",
            )),
        }],
        enumerations: vec![Enumeration {
            class: "surface".into(),
            kind: "routes".into(),
            source: "generated/routes.json".into(),
            source_fingerprint: fp('e'),
            identity: Some(source("web", "app", "enumerator", "surface#routes")),
        }],
        artifacts: vec![
            Artifact {
                id: "artifact:guard".into(),
                kind: "rust-method".into(),
                file: "src/guard.rs".into(),
                unique: None,
                columns: vec!["key".into()],
                predicate: Some("active".into()),
                source: Some(source("core", "code", "rust-method", "alpha::guard")),
            },
            Artifact {
                id: "artifact:worker".into(),
                kind: "rust-method".into(),
                file: "src/worker.rs".into(),
                unique: None,
                columns: Vec::new(),
                predicate: None,
                source: Some(source("core", "code", "rust-method", "alpha::worker")),
            },
        ],
        decision_standards: Some(standards),
        verifications: vec![verification],
        designs: vec![Design {
            spec: "alpha".into(),
            path: "alpha/design.md".into(),
            entries: vec![DesignEntry {
                target: Target::Claim("behavior".into()),
                mechanisms: vec![
                    Mechanism {
                        id: "guard".into(),
                        kind: Enforcement::Guard,
                        cases: vec!["works".into()],
                        binding: Some("artifact:guard".into()),
                        expected_unique: None,
                        expected_columns: vec!["key".into()],
                        expected_predicate: Some("active".into()),
                        line: 1,
                    },
                    Mechanism {
                        id: "worker".into(),
                        kind: Enforcement::ChokePoint,
                        cases: vec!["works".into()],
                        binding: None,
                        expected_unique: None,
                        expected_columns: Vec::new(),
                        expected_predicate: None,
                        line: 2,
                    },
                ],
                line: 1,
            }],
            residue: String::new(),
        }],
        workspace: Workspace {
            path: "azimuth/workspace.json".into(),
            areas: vec![
                Area {
                    id: "core".into(),
                    mounts: vec![Mount {
                        id: "code".into(),
                        path: "".into(),
                    }],
                },
                Area {
                    id: "web".into(),
                    mounts: vec![Mount {
                        id: "app".into(),
                        path: "app".into(),
                    }],
                },
            ],
            surfaces: vec![Surface {
                id: "surface".into(),
                contributions: vec![SurfaceContribution {
                    area: "web".into(),
                    mount: "app".into(),
                    enumerator: "routes".into(),
                }],
            }],
            realization_obligations: Vec::new(),
        },
    };
    model.specs[0].claims[0].over = Some("surface".into());
    refresh_decisions(&mut model);
    model
}

fn refresh_decisions(model: &mut Model) {
    for index in 0..model.verifications[0].method_qualifications.len() {
        let fingerprint = model
            .expected_method_qualification_fingerprint(
                &model.verifications[0].method_qualifications[index],
            )
            .unwrap();
        model.verifications[0].method_qualifications[index].fingerprint = fingerprint;
    }
    for index in 0..model.verifications[0].applicability_decisions.len() {
        let fingerprint = model
            .expected_applicability_fingerprint(&model.verifications[0].bindings[index])
            .unwrap();
        model.verifications[0].applicability_decisions[index].fingerprint = fingerprint;
    }
    for index in 0..model.verifications[0].claim_judgments.len() {
        let fingerprint = model
            .expected_claim_judgment_fingerprint(&model.verifications[0].claim_judgments[index])
            .unwrap();
        model.verifications[0].claim_judgments[index].fingerprint = fingerprint;
    }
}

fn challenge_request(operation: RunOperation) -> PlanRequest {
    let mut request = request(&[], operation, "demo/alpha");
    request.challenges = vec![RequestedChallenge {
        id: "alpha/plan".into(),
        capability: "demo/challenge".into(),
        max_candidates: 1,
        units: vec![unit("whole")],
    }];
    request
}

fn request(ids: &[&str], operation: RunOperation, capability: &str) -> PlanRequest {
    PlanRequest {
        operation,
        planned_at_ms: 1_787_300_000_000,
        subject: Subject::Artifact {
            artifacts: vec![ArtifactState {
                id: "image".into(),
                digest: fp('8'),
            }],
        },
        required_context: BTreeMap::from([("platform".into(), "linux".into())]),
        checks: ids
            .iter()
            .map(|id| RequestedCheck {
                id: (*id).into(),
                capability: capability.into(),
                cases: vec!["synthetic#works/works".into()],
                units: vec![unit("whole")],
            })
            .collect(),
        challenges: Vec::new(),
    }
}

#[test]
fn request_parser_preserves_exact_subject_time_context_operation_and_units() {
    let source = plan_request_to_json(&request(
        &["checks/alpha"],
        RunOperation::Import,
        "demo/reports",
    ))
    .to_string_pretty();
    let parsed = parse_plan_request("request.json", &source).unwrap();
    assert_eq!(parsed.operation, RunOperation::Import);
    assert_eq!(parsed.planned_at_ms, 1_787_300_000_000);
    assert_eq!(parsed.required_context["platform"], "linux");
    assert_eq!(
        parsed.subject,
        request(&["checks/alpha"], RunOperation::Import, "demo/reports").subject
    );
    assert_eq!(parsed.checks[0].units, vec![unit("whole")]);

    let source = plan_request_to_json(&challenge_request(RunOperation::Execute)).to_string_pretty();
    let parsed = parse_plan_request("request.json", &source).unwrap();
    assert!(parsed.checks.is_empty());
    assert_eq!(parsed.challenges[0].id, "alpha/plan");
    assert_eq!(parsed.challenges[0].max_candidates, 1);
}

#[test]
fn request_parser_rejects_unknown_duplicate_and_noncanonical_members() {
    let valid = plan_request_to_json(&request(
        &["checks/alpha"],
        RunOperation::Execute,
        "demo/alpha",
    ))
    .to_string_pretty();
    assert!(parse_plan_request(
        "request.json",
        &valid.replacen("\"checks\":", "\"extra\":true,\n  \"checks\":", 1),
    )
    .is_err());
    let duplicate = valid.replacen(
        "\"format\": \"azimuth-run-plan-request\"",
        "\"format\": \"azimuth-run-plan-request\",\n  \"format\": \"azimuth-run-plan-request\"",
        1,
    );
    assert!(parse_plan_request("request.json", &duplicate).is_err());

    let mut unsorted = request(
        &["checks/beta", "checks/alpha"],
        RunOperation::Execute,
        "demo/alpha",
    );
    assert!(plan(
        &model(&["checks/alpha", "checks/beta"]),
        &configuration(),
        &unsorted
    )
    .is_err());
    unsorted.checks[0].id = "checks/alpha".into();
    assert!(plan(&model(&["checks/alpha"]), &configuration(), &unsorted).is_err());
}

#[test]
fn planner_uses_the_complete_model_before_selecting_checks() {
    let first = model(&["checks/alpha"]);
    let second = model(&["checks/alpha", "checks/unselected"]);
    let request = request(&["checks/alpha"], RunOperation::Execute, "demo/alpha");
    let first = plan(&first, &configuration(), &request).unwrap();
    let second = plan(&second, &configuration(), &request).unwrap();
    assert_eq!(first.plan.checks, second.plan.checks);
    assert_ne!(first.plan.model_fingerprint, second.plan.model_fingerprint);
    assert_ne!(first.plan.fingerprint, second.plan.fingerprint);
}

#[test]
fn unknown_and_duplicate_model_checks_fail_closed() {
    let unknown = request(&["checks/missing"], RunOperation::Execute, "demo/alpha");
    assert!(
        plan(&model(&["checks/alpha"]), &configuration(), &unknown).unwrap_err()[0]
            .detail
            .contains("unknown Check")
    );

    let mut duplicate = model(&["checks/alpha"]);
    duplicate.verifications.push(Verification {
        owner: "second".into(),
        path: "second/verification.md".into(),
        checks: vec![check("checks/alpha")],
        bindings: Vec::new(),
        method_qualifications: Vec::new(),
        applicability_decisions: Vec::new(),
        claim_judgments: Vec::new(),
        challengers: Vec::new(),
        challenge_plans: Vec::new(),
    });
    assert!(plan(
        &duplicate,
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap_err()[0]
        .detail
        .contains("declared more than once"));
}

#[test]
fn unimplemented_and_unstable_checks_fail_closed() {
    let mut unimplemented = model(&["checks/alpha"]);
    unimplemented.check_implementations.clear();
    let request = request(&["checks/alpha"], RunOperation::Execute, "demo/alpha");
    assert!(
        plan(&unimplemented, &configuration(), &request).unwrap_err()[0]
            .detail
            .contains("no implementation")
    );

    let mut unstable = model(&["checks/alpha"]);
    unstable.check_implementations[0].source = None;
    assert!(plan(&unstable, &configuration(), &request).unwrap_err()[0]
        .detail
        .contains("stable SourceIdentity"));
}

#[test]
fn planner_includes_the_complete_sorted_implementation_closure() {
    let mut model = model(&["checks/alpha"]);
    model.check_implementations = vec![
        implementation("checks/alpha", "checks::zeta"),
        implementation("checks/alpha", "checks::alpha"),
    ];
    let launch = plan(
        &model,
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();
    assert_eq!(
        launch.plan.checks[0]
            .implementations
            .iter()
            .map(|item| item.identity.as_str())
            .collect::<Vec<_>>(),
        vec![
            "core|rust-symbol|checks::alpha",
            "core|rust-symbol|checks::zeta"
        ]
    );
}

#[test]
fn capability_class_and_adapter_address_are_exact() {
    let model = model(&["checks/alpha"]);
    let import_through_execute = request(&["checks/alpha"], RunOperation::Import, "demo/alpha");
    assert!(
        plan(&model, &configuration(), &import_through_execute).unwrap_err()[0]
            .detail
            .contains("check.import")
    );
    let unknown = request(&["checks/alpha"], RunOperation::Execute, "other/alpha");
    assert!(plan(&model, &configuration(), &unknown).unwrap_err()[0]
        .detail
        .contains("unknown configured capability"));
}

#[test]
fn one_launch_cannot_route_checks_through_several_configured_adapters() {
    let mut config = configuration();
    let mut other = config.adapters[0].clone();
    other.id = "other".into();
    other.adapter_fingerprint = fp('a');
    other.descriptor_fingerprint = fp('b');
    other.configuration_fingerprint = fp('c');
    config.adapters.push(other);
    let mut request = request(
        &["checks/alpha", "checks/beta"],
        RunOperation::Execute,
        "demo/alpha",
    );
    request.checks[1].capability = "other/alpha".into();
    let errors = plan(&model(&["checks/alpha", "checks/beta"]), &config, &request).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.detail.contains("several adapters")));
}

#[test]
fn capability_substitution_changes_launch_identity_but_not_semantic_plan() {
    let model = model(&["checks/alpha"]);
    let alpha = plan(
        &model,
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();
    let beta = plan(
        &model,
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/beta"),
    )
    .unwrap();
    assert_eq!(alpha.plan, beta.plan);
    assert_ne!(alpha.fingerprint, beta.fingerprint);
}

#[test]
fn exact_subject_context_units_time_and_operation_are_identity_bearing() {
    let model = model(&["checks/alpha"]);
    let config = configuration();
    let base_request = request(&["checks/alpha"], RunOperation::Execute, "demo/alpha");
    let base = plan(&model, &config, &base_request).unwrap();

    let mut context = base_request.clone();
    context
        .required_context
        .insert("platform".into(), "macos".into());
    let context = plan(&model, &config, &context).unwrap();
    assert_ne!(base.plan.fingerprint, context.plan.fingerprint);

    let mut units = base_request.clone();
    units.checks[0].units[0]
        .parameters
        .insert("shard".into(), "one".into());
    let units = plan(&model, &config, &units).unwrap();
    assert_ne!(base.plan.fingerprint, units.plan.fingerprint);

    let mut subject = base_request.clone();
    subject.subject = Subject::Artifact {
        artifacts: vec![ArtifactState {
            id: "image".into(),
            digest: fp('9'),
        }],
    };
    let subject = plan(&model, &config, &subject).unwrap();
    assert_ne!(base.subject_fingerprint, subject.subject_fingerprint);
    assert_ne!(base.fingerprint, subject.fingerprint);

    let mut planned_time = base_request.clone();
    planned_time.planned_at_ms += 1;
    let planned_time = plan(&model, &config, &planned_time).unwrap();
    assert_eq!(base.plan, planned_time.plan);
    assert_ne!(base.fingerprint, planned_time.fingerprint);

    let imported = plan(
        &model,
        &config,
        &request(&["checks/alpha"], RunOperation::Import, "demo/reports"),
    )
    .unwrap();
    assert_eq!(base.plan, imported.plan);
    assert_ne!(base.fingerprint, imported.fingerprint);
}

#[test]
fn several_bindings_still_produce_one_check_and_no_challenges_or_qualification_gate() {
    let mut model = model(&["checks/alpha"]);
    let mut other_case = model.specs[0].claims[0].cases[0].clone();
    other_case.id = "other".into();
    model.specs[0].claims[0].cases.push(other_case);
    let mut second = binding("binding/two");
    second.case = "synthetic#works/other".into();
    model.verifications[0].bindings.push(second);
    assert!(model.verifications[0].method_qualifications.is_empty());
    let launch = plan(
        &model,
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();
    assert_eq!(launch.plan.checks.len(), 1);
    assert!(launch.plan.challenges.is_empty());
}

#[test]
fn rejected_qualification_and_binding_context_mismatch_do_not_gate_planning() {
    let mut model = model(&["checks/alpha"]);
    let mut binding = binding("binding/one");
    binding.context.insert("platform".into(), "linux".into());
    model.verifications[0].bindings = vec![binding];
    model.verifications[0].method_qualifications = vec![MethodQualification {
        id: "binding/one".into(),
        check: "checks/alpha".into(),
        scope: model::Scope::Unit,
        quantification: model::Quantification::Example,
        oracle: model::Oracle::Direct,
        context: BTreeMap::new(),
        challenge_domain: Vec::new(),
        policy: "credible".into(),
        verdict: MethodQualificationVerdict::Rejected,
        fingerprint: fp('d'),
        qualified: "2026-08-21".into(),
        qualifier: "reviewer".into(),
        rationale: "The candidate is deliberately rejected.".into(),
        path: "verification.md".into(),
        line: 2,
    }];
    let mut request = request(&["checks/alpha"], RunOperation::Execute, "demo/alpha");
    request
        .required_context
        .insert("platform".into(), "windows".into());
    let launch = plan(&model, &configuration(), &request).unwrap();
    assert_eq!(launch.plan.checks.len(), 1);
    assert!(launch.plan.challenges.is_empty());
}

#[test]
fn route_order_cardinality_and_configured_capability_identity_fail_closed() {
    let model = model(&["checks/alpha", "checks/beta"]);
    let config = configuration();
    let launch = plan(
        &model,
        &config,
        &request(
            &["checks/alpha", "checks/beta"],
            RunOperation::Execute,
            "demo/alpha",
        ),
    )
    .unwrap();

    let mut reversed = launch.clone();
    reversed.routes.reverse();
    reversed.fingerprint = launch_fingerprint(&reversed);
    assert!(validate_launch_plan(&reversed)
        .iter()
        .any(|error| error.contains("canonical Plan selection")));

    let mut missing = launch.clone();
    missing.routes.pop();
    missing.fingerprint = launch_fingerprint(&missing);
    assert!(validate_launch_plan(&missing)
        .iter()
        .any(|error| error.contains("exactly one entry")));

    let mut fingerprint_drift = launch.clone();
    fingerprint_drift.routes[0].capability.fingerprint = fp('e');
    fingerprint_drift.fingerprint = launch_fingerprint(&fingerprint_drift);
    assert!(validate_launch_configuration(&fingerprint_drift, &config)
        .iter()
        .any(|error| error.contains("fingerprint differs")));

    let mut class_drift = launch;
    class_drift.routes[0].capability.class = run::RouteCapabilityClass::CheckImport;
    class_drift.fingerprint = launch_fingerprint(&class_drift);
    assert!(validate_launch_configuration(&class_drift, &config)
        .iter()
        .any(|error| error.contains("does not support `check.import`")));
}

#[test]
fn typed_routes_enforce_address_and_challenge_form_shape() {
    let launch = plan(
        &model(&["checks/alpha"]),
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();

    let mut malformed_address = launch.clone();
    malformed_address.routes[0].capability.address = "demo/alpha/extra".into();
    malformed_address.fingerprint = launch_fingerprint(&malformed_address);
    assert!(validate_launch_plan(&malformed_address)
        .iter()
        .any(|error| error.contains("exactly two lower-kebab segments")));

    let mut check_form = launch.clone();
    check_form.routes[0].capability.challenge_form = Some("mutation/perturbation".into());
    check_form.fingerprint = launch_fingerprint(&check_form);
    assert!(validate_launch_plan(&check_form)
        .iter()
        .any(|error| error.contains("forbidden for a Check route")));

    let mut missing_challenge_form = launch.clone();
    missing_challenge_form.routes[0].selection.kind = run::RouteSelectionKind::Challenge;
    missing_challenge_form.fingerprint = launch_fingerprint(&missing_challenge_form);
    assert!(validate_launch_plan(&missing_challenge_form)
        .iter()
        .any(|error| error.contains("required for a Challenge route")));

    let mut invalid_challenge_form = missing_challenge_form;
    invalid_challenge_form.routes[0].capability.challenge_form = Some("Bad/Form".into());
    invalid_challenge_form.fingerprint = launch_fingerprint(&invalid_challenge_form);
    assert!(validate_launch_plan(&invalid_challenge_form)
        .iter()
        .any(|error| error.contains("must be a lower-kebab path id")));
}

#[test]
fn unsafe_and_fractional_request_numbers_are_schema_errors() {
    let source = plan_request_to_json(&request(
        &["checks/alpha"],
        RunOperation::Execute,
        "demo/alpha",
    ))
    .to_string_pretty();
    let unsafe_number = source.replacen("1787300000000", "9007199254740992", 1);
    assert!(parse_plan_request("request.json", &unsafe_number).is_err());
    let fractional = source.replacen("1787300000000", "1787300000000.5", 1);
    assert!(parse_plan_request("request.json", &fractional).is_err());
}

#[test]
fn typed_launch_validation_never_hashes_unsafe_numbers() {
    let valid = plan(
        &model(&["checks/alpha"]),
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();

    let mut unsafe_time = valid.clone();
    unsafe_time.planned_at_ms = 9_007_199_254_740_992;
    let errors = std::panic::catch_unwind(|| validate_launch_plan(&unsafe_time)).unwrap();
    assert!(errors
        .iter()
        .any(|error| error.contains("planned_at_ms exceeds")));

    let mut unsafe_subject = valid;
    unsafe_subject.subject = Subject::MonitoringWindow {
        environment: "staging".into(),
        services: vec![run::ServiceState {
            service: "api".into(),
            deployment: "candidate".into(),
            deployment_fingerprint: fp('f'),
        }],
        window_start_ms: 1,
        window_end_ms: 9_007_199_254_740_992,
    };
    let errors = std::panic::catch_unwind(|| validate_launch_plan(&unsafe_subject)).unwrap();
    assert!(errors
        .iter()
        .any(|error| error.contains("maximum safe integer")));
}

#[test]
fn launch_round_trip_is_deterministic_and_rejects_substitution() {
    let launch = plan(
        &model(&["checks/alpha"]),
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();
    let first = launch_plan_to_json(&launch).to_string_pretty();
    let parsed = parse_launch_plan("launch.json", &first).unwrap();
    assert_eq!(launch, parsed);
    assert_eq!(first, launch_plan_to_json(&parsed).to_string_pretty());

    let substituted = first.replacen("demo/alpha", "demo/beta", 1);
    assert!(parse_launch_plan("launch.json", &substituted).is_err());
}

#[test]
fn canonical_launch_vector_matches_the_frozen_vector() {
    let subject = Subject::Artifact {
        artifacts: vec![ArtifactState {
            id: "image".into(),
            digest: fp('4'),
        }],
    };
    let launch = LaunchPlan {
        operation: RunOperation::Execute,
        planned_at_ms: 1_787_300_000_000,
        subject,
        subject_fingerprint:
            "sha256:22478698e6731ce5984658e366386e466fe173216bc7cb721168e1638d2dee02".into(),
        plan: Plan {
            model_fingerprint: fp('8'),
            required_context: BTreeMap::new(),
            checks: vec![run::CheckSelection {
                id: "demo/check".into(),
                fingerprint: fp('6'),
                cases: vec!["demo#works/works".into()],
                implementations: vec![run::Implementation {
                    identity: "demo|rust-symbol|demo::check".into(),
                    source_fingerprint: fp('7'),
                }],
                units: vec![unit("whole")],
            }],
            challenges: Vec::new(),
            fingerprint: "sha256:b75606956b9c1857f8b401d9bad207253b90f6948efddb5532a769b9f488fbfb"
                .into(),
        },
        adapter: LaunchAdapter {
            id: "demo".into(),
            adapter_version: "1".into(),
            adapter_fingerprint: fp('0'),
            configuration_fingerprint: fp('1'),
            descriptor_fingerprint: fp('2'),
        },
        routes: vec![run::LaunchRoute {
            selection: run::RouteSelection {
                kind: run::RouteSelectionKind::Check,
                id: "demo/check".into(),
            },
            capability: run::RouteCapability {
                address: "demo/check".into(),
                class: run::RouteCapabilityClass::CheckExecute,
                challenge_form: None,
                fingerprint: fp('3'),
            },
            inputs: Vec::new(),
        }],
        fingerprint: fp('0'),
    };
    assert_eq!(
        launch_fingerprint(&launch),
        "sha256:7043a3051227f7f36561e2076fd681f0567c745e1a0475df8983c8eabde866f6"
    );
}

#[test]
fn planning_and_finalization_share_the_exact_model_fingerprint() {
    let model = model(&["checks/alpha"]);
    let findings = validation::validate(&model);
    let expected = fingerprint::model_digest(&model, &findings);
    let (finalized, _) = azimuth::change::finalization(&model, &findings);
    let launch = plan(
        &model,
        &configuration(),
        &request(&["checks/alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();
    assert_eq!(finalized, expected);
    assert_eq!(finalized.len(), 64);
    assert!(finalized
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    assert_eq!(launch.plan.model_fingerprint, format!("sha256:{finalized}"));
}

#[test]
fn challenge_only_and_mixed_plans_derive_exact_semantics_and_accountable_inputs() {
    let model = challenge_model();
    let config = configuration();
    let challenge = plan(&model, &config, &challenge_request(RunOperation::Execute)).unwrap();
    assert!(challenge.plan.checks.is_empty());
    assert_eq!(challenge.plan.challenges.len(), 1);
    assert_eq!(challenge.plan.challenges[0].lane, run::ChallengeLane::Gate);
    assert_eq!(challenge.routes.len(), 1);
    assert_eq!(challenge.routes[0].inputs.len(), 1);
    assert_eq!(
        challenge.routes[0].inputs[0].kind,
        run::LaunchInputKind::CheckImplementation
    );
    assert_eq!(
        challenge.routes[0].inputs[0].source,
        run::LaunchInputSource::Source {
            file: "src/check.rs".into(),
            language: "rust-symbol".into(),
            site: "checks::alpha".into(),
        }
    );
    assert_eq!(
        challenge.routes[0].capability.challenge_form.as_deref(),
        Some("mutation")
    );

    let mut mixed = challenge_request(RunOperation::Execute);
    mixed.checks.push(RequestedCheck {
        id: "alpha/check".into(),
        capability: "demo/alpha".into(),
        cases: vec!["alpha#behavior/works".into()],
        units: vec![unit("whole")],
    });
    let mixed = plan(&model, &config, &mixed).unwrap();
    assert_eq!(mixed.plan.checks.len(), 1);
    assert_eq!(mixed.plan.challenges.len(), 1);
    assert_eq!(
        mixed.routes[0].selection.kind,
        run::RouteSelectionKind::Check
    );
    assert!(mixed.routes[0].inputs.is_empty());
    assert_eq!(
        mixed.routes[1].selection.kind,
        run::RouteSelectionKind::Challenge
    );
}

#[test]
fn challenge_planning_fails_closed_on_caps_context_forms_and_empty_selection() {
    let model = challenge_model();
    let config = configuration();

    let mut capped = challenge_request(RunOperation::Execute);
    capped.challenges[0].max_candidates = 0;
    assert!(plan(&model, &config, &capped).unwrap_err()[0]
        .detail
        .contains("max_candidates"));

    let mut context = challenge_request(RunOperation::Execute);
    context
        .required_context
        .insert("platform".into(), "windows".into());
    assert!(plan(&model, &config, &context)
        .unwrap_err()
        .iter()
        .any(|error| error.detail.contains("context must equal")));

    let mut form_config = config.clone();
    form_config.adapters[0].capabilities[3]
        .challenge_forms
        .clear();
    assert!(plan(
        &model,
        &form_config,
        &challenge_request(RunOperation::Execute)
    )
    .unwrap_err()
    .iter()
    .any(|error| error.detail.contains("Challenge form")));

    let mut empty = challenge_request(RunOperation::Execute);
    empty.challenges.clear();
    assert!(plan(&model, &config, &empty).unwrap_err()[0]
        .detail
        .contains("must not both be empty"));

    let mut no_targets = challenge_model();
    no_targets.verifications[0].challenge_plans[0]
        .selectors
        .clear();
    assert!(plan(
        &no_targets,
        &config,
        &challenge_request(RunOperation::Execute)
    )
    .unwrap_err()
    .iter()
    .any(|error| error.detail.contains("resolves no targets")));

    let mut stale = challenge_model();
    stale.verifications[0].method_qualifications[0].fingerprint = fp('f');
    assert!(
        plan(&stale, &config, &challenge_request(RunOperation::Execute))
            .unwrap_err()
            .iter()
            .any(|error| error.detail.contains("not runnable"))
    );

    let mut insufficient = challenge_model();
    insufficient.verifications[0].challengers[0]
        .required_scope
        .push(SemanticScopeKind::Mechanism);
    assert!(plan(
        &insufficient,
        &config,
        &challenge_request(RunOperation::Execute)
    )
    .unwrap_err()
    .iter()
    .any(|error| error.detail.contains("required scope")));
}

#[test]
fn max_candidates_counts_the_resolved_plan_before_cross_plan_deduplication() {
    let mut model = challenge_model();
    model.verifications[0].challenge_plans[0].selectors =
        vec![Selector::MethodQualificationFromCheck("alpha/check".into())];
    let mut binding = model.verifications[0].bindings[0].clone();
    binding.id = "alpha/edge-two".into();
    model.verifications[0].bindings.push(binding);
    let mut qualification = model.verifications[0].method_qualifications[0].clone();
    qualification.id = "alpha/edge-two".into();
    qualification.fingerprint = model
        .expected_method_qualification_fingerprint(&qualification)
        .unwrap();
    model.verifications[0]
        .method_qualifications
        .push(qualification);
    assert!(plan(
        &model,
        &configuration(),
        &challenge_request(RunOperation::Execute)
    )
    .unwrap_err()
    .iter()
    .any(|error| error.detail.contains("exceeding max_candidates 1")));
}

#[test]
fn challenge_route_validation_detects_scope_and_capability_drift() {
    let model = challenge_model();
    let config = configuration();
    let launch = plan(&model, &config, &challenge_request(RunOperation::Import)).unwrap();
    assert_eq!(
        launch.routes[0].capability.class,
        run::RouteCapabilityClass::ChallengeImport
    );

    let mut missing_input = launch.clone();
    missing_input.routes[0].inputs.clear();
    missing_input.fingerprint = launch_fingerprint(&missing_input);
    assert!(validate_launch_plan(&missing_input)
        .iter()
        .any(|error| error.contains("input")));

    let mut form_drift = launch.clone();
    form_drift.routes[0].capability.challenge_form = Some("different".into());
    form_drift.fingerprint = launch_fingerprint(&form_drift);
    assert!(validate_launch_configuration(&form_drift, &config)
        .iter()
        .any(|error| error.contains("does not support Challenge form")));
}

#[test]
fn overlapping_plans_deduplicate_exact_targets_and_reject_route_conflicts() {
    let mut model = challenge_model();
    let mut second = model.verifications[0].challenge_plans[0].clone();
    second.id = "alpha/plan-two".into();
    model.verifications[0].challenge_plans.push(second);
    let mut request = challenge_request(RunOperation::Execute);
    request.challenges.push(RequestedChallenge {
        id: "alpha/plan-two".into(),
        capability: "demo/challenge".into(),
        max_candidates: 1,
        units: vec![unit("whole")],
    });
    let launch = plan(&model, &configuration(), &request).unwrap();
    assert_eq!(launch.plan.challenges.len(), 1);
    assert_eq!(launch.routes.len(), 1);

    request.challenges[1].units[0]
        .parameters
        .insert("shard".into(), "other".into());
    assert!(plan(&model, &configuration(), &request)
        .unwrap_err()
        .iter()
        .any(|error| error.detail.contains("conflicting capability or units")));
}

#[test]
fn challenge_planning_uses_complete_model_identity_and_required_form_union() {
    let model = challenge_model();
    let base = plan(
        &model,
        &configuration(),
        &challenge_request(RunOperation::Execute),
    )
    .unwrap();
    let mut expanded = challenge_model();
    expanded.specs.push(
        parse_spec(
            "extra/spec.md",
            "# Spec: extra\n\n## Claim: behavior\nCriticality: routine\n\nThe system SHALL remain explicit.\n\n### Case: stable\nWHEN inspected\nTHEN it remains explicit\n",
        )
        .unwrap(),
    );
    let expanded = plan(
        &expanded,
        &configuration(),
        &challenge_request(RunOperation::Execute),
    )
    .unwrap();
    assert_eq!(base.plan.challenges, expanded.plan.challenges);
    assert_ne!(base.plan.model_fingerprint, expanded.plan.model_fingerprint);

    let mut missing_form = model;
    missing_form.decision_standards.as_mut().unwrap().policies[0]
        .required_challenges
        .push("destructive".into());
    missing_form.verifications[0].method_qualifications[0].fingerprint = missing_form
        .expected_method_qualification_fingerprint(
            &missing_form.verifications[0].method_qualifications[0],
        )
        .unwrap();
    let errors = plan(
        &missing_form,
        &configuration(),
        &challenge_request(RunOperation::Execute),
    )
    .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.detail.contains("missing required Challenge form")),
        "{errors:?}"
    );
}

#[test]
fn planner_executes_all_seven_selector_forms_through_current_decisions() {
    let cases = [
        (
            Selector::ApplicabilityDecisionFromBinding("alpha/edge".into()),
            run::ChallengeTargetKind::ApplicabilityDecision,
            SemanticScopeKind::Binding,
        ),
        (
            Selector::MethodQualificationFromCheck("alpha/check".into()),
            run::ChallengeTargetKind::MethodQualification,
            SemanticScopeKind::Check,
        ),
        (
            Selector::MethodQualificationFromRealization("core|rust-item|alpha::works".into()),
            run::ChallengeTargetKind::MethodQualification,
            SemanticScopeKind::Realization,
        ),
        (
            Selector::MethodQualificationFromMechanism("alpha#guard".into()),
            run::ChallengeTargetKind::MethodQualification,
            SemanticScopeKind::Mechanism,
        ),
        (
            Selector::ClaimJudgmentFromClaim("alpha#behavior".into()),
            run::ChallengeTargetKind::ClaimJudgment,
            SemanticScopeKind::Claim,
        ),
        (
            Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
            run::ChallengeTargetKind::ClaimJudgment,
            SemanticScopeKind::Realization,
        ),
        (
            Selector::ClaimJudgmentFromMechanism("alpha#guard".into()),
            run::ChallengeTargetKind::ClaimJudgment,
            SemanticScopeKind::Mechanism,
        ),
    ];
    for (selector, expected_kind, required_scope) in cases {
        let mut model = rich_challenge_model();
        model.verifications[0].challenge_plans[0].selectors = vec![selector];
        model.verifications[0].challengers[0].required_scope = vec![required_scope];
        let launch = plan(
            &model,
            &configuration(),
            &challenge_request(RunOperation::Execute),
        )
        .unwrap();
        assert_eq!(launch.plan.challenges.len(), 1);
        assert_eq!(launch.plan.challenges[0].target.kind, expected_kind);
        assert_eq!(launch.routes.len(), 1);
    }
}

#[test]
fn scheduled_lane_and_every_launch_input_locator_variant_are_derived() {
    let mut model = rich_challenge_model();
    let schedule = &mut model.decision_standards.as_mut().unwrap().schedule;
    schedule.gate_challenges.clear();
    schedule.scheduled_challenges = vec!["mutation".into()];
    let launch = plan(
        &model,
        &configuration(),
        &challenge_request(RunOperation::Execute),
    )
    .unwrap();
    assert_eq!(
        launch.plan.challenges[0].lane,
        run::ChallengeLane::Scheduled
    );
    let inputs = &launch.routes[0].inputs;
    for kind in [
        run::LaunchInputKind::CheckImplementation,
        run::LaunchInputKind::Realization,
        run::LaunchInputKind::MechanismImplementation,
        run::LaunchInputKind::Artifact,
        run::LaunchInputKind::Enumeration,
        run::LaunchInputKind::SurfaceMember,
    ] {
        assert!(inputs.iter().any(|input| input.kind == kind), "{kind:?}");
    }
    assert!(inputs
        .iter()
        .any(|input| matches!(input.source, run::LaunchInputSource::Artifact { .. })));
    assert!(inputs
        .iter()
        .any(|input| matches!(input.source, run::LaunchInputSource::Enumeration { .. })));
    assert!(inputs
        .iter()
        .any(|input| matches!(input.source, run::LaunchInputSource::SurfaceMember { .. })));
    assert!(inputs.iter().any(|input| {
        input.kind == run::LaunchInputKind::SurfaceMember
            && matches!(input.source, run::LaunchInputSource::Source { .. })
    }));
}

fn add_second_binding(model: &mut Model, context: &str, stale: bool) {
    let mut binding = model.verifications[0].bindings[0].clone();
    binding.id = "alpha/edge-two".into();
    binding.method_qualification = "alpha/method-two".into();
    binding.context.insert("platform".into(), context.into());
    model.verifications[0].bindings.push(binding);
    let mut qualification = model.verifications[0].method_qualifications[0].clone();
    qualification.id = "alpha/method-two".into();
    qualification
        .context
        .insert("platform".into(), context.into());
    model.verifications[0]
        .method_qualifications
        .push(qualification);
    let fingerprint = model
        .expected_method_qualification_fingerprint(&model.verifications[0].method_qualifications[1])
        .unwrap();
    model.verifications[0].method_qualifications[1].fingerprint =
        if stale { fp('f') } else { fingerprint };
}

#[test]
fn multi_target_context_and_adverse_siblings_fail_before_any_launch() {
    let mut contexts = rich_challenge_model();
    contexts.verifications[0].challenge_plans[0].selectors =
        vec![Selector::MethodQualificationFromCheck("alpha/check".into())];
    contexts.verifications[0].challengers[0].required_scope = vec![SemanticScopeKind::Check];
    add_second_binding(&mut contexts, "windows", false);
    let mut request = challenge_request(RunOperation::Execute);
    request.challenges[0].max_candidates = 2;
    let context_errors = plan(&contexts, &configuration(), &request).unwrap_err();
    assert!(
        context_errors
            .iter()
            .any(|error| error.detail.contains("context must equal")),
        "{context_errors:?}"
    );

    let mut adverse = rich_challenge_model();
    adverse.verifications[0].challenge_plans[0].selectors =
        vec![Selector::MethodQualificationFromCheck("alpha/check".into())];
    adverse.verifications[0].challengers[0].required_scope = vec![SemanticScopeKind::Check];
    add_second_binding(&mut adverse, "linux", true);
    let cap_errors = plan(
        &adverse,
        &configuration(),
        &challenge_request(RunOperation::Execute),
    )
    .unwrap_err();
    assert!(cap_errors
        .iter()
        .any(|error| error.detail.contains("exceeding max_candidates 1")));
    request.challenges[0].max_candidates = 2;
    let adverse_errors = plan(&adverse, &configuration(), &request).unwrap_err();
    assert!(adverse_errors
        .iter()
        .any(|error| error.detail.contains("selected") && error.detail.contains("stale-decision")));
}

#[test]
fn mixed_check_challenge_routes_enforce_one_adapter_and_support_import() {
    let model = rich_challenge_model();
    let mut config = configuration();
    let mut other = config.adapters[0].clone();
    other.id = "other".into();
    other.adapter_fingerprint = fp('a');
    other.descriptor_fingerprint = fp('b');
    other.configuration_fingerprint = fp('c');
    config.adapters.push(other);
    let mut cross = challenge_request(RunOperation::Execute);
    cross.checks.push(RequestedCheck {
        id: "alpha/check".into(),
        capability: "demo/alpha".into(),
        cases: vec!["alpha#behavior/works".into()],
        units: vec![unit("whole")],
    });
    cross.challenges[0].capability = "other/challenge".into();
    assert!(plan(&model, &config, &cross)
        .unwrap_err()
        .iter()
        .any(|error| error.detail.contains("several adapters")));

    let mut import = challenge_request(RunOperation::Import);
    import.checks.push(RequestedCheck {
        id: "alpha/check".into(),
        capability: "demo/reports".into(),
        cases: vec!["alpha#behavior/works".into()],
        units: vec![unit("whole")],
    });
    let launch = plan(&model, &configuration(), &import).unwrap();
    assert_eq!(launch.plan.checks.len(), 1);
    assert_eq!(launch.plan.challenges.len(), 1);
    assert_eq!(
        launch.routes[0].capability.class,
        run::RouteCapabilityClass::CheckImport
    );
    assert_eq!(
        launch.routes[1].capability.class,
        run::RouteCapabilityClass::ChallengeImport
    );
}

#[test]
fn two_required_forms_use_the_fixed_requested_plan_union() {
    let mut model = rich_challenge_model();
    let mut challenger = model.verifications[0].challengers[0].clone();
    challenger.id = "destructive/search".into();
    challenger.form = "destructive".into();
    model.verifications[0].challengers.push(challenger);
    let mut destructive = model.verifications[0].challenge_plans[0].clone();
    destructive.id = "alpha/destructive".into();
    destructive.challenger = "destructive/search".into();
    model.verifications[0].challenge_plans.push(destructive);
    let standards = model.decision_standards.as_mut().unwrap();
    standards.policies[0]
        .required_challenges
        .push("destructive".into());
    standards
        .schedule
        .scheduled_challenges
        .push("destructive".into());
    refresh_decisions(&mut model);

    let mut config = configuration();
    config.adapters[0].capabilities[3]
        .challenge_forms
        .push("destructive".into());
    let missing = plan(&model, &config, &challenge_request(RunOperation::Execute)).unwrap_err();
    assert!(missing.iter().any(|error| error
        .detail
        .contains("missing required Challenge form `destructive`")));

    let mut request = challenge_request(RunOperation::Execute);
    request.challenges.insert(
        0,
        RequestedChallenge {
            id: "alpha/destructive".into(),
            capability: "demo/challenge".into(),
            max_candidates: 1,
            units: vec![unit("whole")],
        },
    );
    let launch = plan(&model, &config, &request).unwrap();
    assert_eq!(launch.plan.challenges.len(), 2);
}

#[test]
fn selector_order_is_stable_and_relocation_changes_only_launch_accounting() {
    let selectors = vec![
        Selector::ClaimJudgmentFromClaim("alpha#behavior".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
        Selector::ClaimJudgmentFromMechanism("alpha#guard".into()),
    ];
    let mut first_model = rich_challenge_model();
    first_model.verifications[0].challenge_plans[0].selectors = selectors.clone();
    let mut request = challenge_request(RunOperation::Execute);
    request.challenges[0].max_candidates = 3;
    let first = plan(&first_model, &configuration(), &request).unwrap();
    let mut reordered_model = rich_challenge_model();
    reordered_model.verifications[0].challenge_plans[0].selectors =
        selectors.into_iter().rev().collect();
    let reordered = plan(&reordered_model, &configuration(), &request).unwrap();
    assert_eq!(first.plan.challenges, reordered.plan.challenges);
    assert_eq!(first.routes, reordered.routes);

    let mut relocated_model = first_model;
    relocated_model.realizes[0].file = "moved/alpha.rs".into();
    relocated_model.realizes[0].site = "moved::alpha::works".into();
    let relocated = plan(&relocated_model, &configuration(), &request).unwrap();
    assert_eq!(
        first.plan.challenges[0].target,
        relocated.plan.challenges[0].target
    );
    assert_eq!(
        first.plan.challenges[0].scope,
        relocated.plan.challenges[0].scope
    );
    assert_ne!(first.routes[0].inputs, relocated.routes[0].inputs);
    assert!(relocated.routes[0].inputs.iter().any(|input| matches!(
        &input.source,
        run::LaunchInputSource::Source { file, site, .. }
            if file == "moved/alpha.rs" && site == "moved::alpha::works"
    )));
    assert_ne!(first.fingerprint, relocated.fingerprint);
}

fn binding(id: &str) -> EvidenceBinding {
    EvidenceBinding {
        id: id.into(),
        check: "checks/alpha".into(),
        case: "synthetic#works/works".into(),
        method_qualification: format!("qualifications/{}", id.rsplit('/').next().unwrap()),
        proposition: "the Check bears on the Claim".into(),
        context: BTreeMap::new(),
        challenge_domain: Vec::new(),
        policy: "routine".into(),
        rationale: String::new(),
        path: "verification.md".into(),
        line: 1,
    }
}
