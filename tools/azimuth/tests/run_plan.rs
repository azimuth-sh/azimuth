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
use model::{CheckImplementation, Model, SourceIdentity};
use run::{ArtifactState, Plan, Subject, WorkUnit};
use run_plan::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use verification::{Check, EvidenceBinding, Qualification, QualificationVerdict, Verification};

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
    Model {
        verifications: vec![Verification {
            owner: "root".into(),
            path: "verification.md".into(),
            checks: checks.iter().map(|id| check(id)).collect(),
            bindings: Vec::new(),
            qualifications: Vec::new(),
            challengers: Vec::new(),
            challenge_plans: Vec::new(),
        }],
        check_implementations: checks
            .iter()
            .map(|id| implementation(id, &format!("checks::{}", id.rsplit('/').next().unwrap())))
            .collect(),
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
            ],
            adapter_fingerprint: fp('5'),
            descriptor_fingerprint: fp('6'),
            configuration_fingerprint: fp('7'),
        }],
    }
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
                units: vec![unit("whole")],
            })
            .collect(),
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
        qualifications: Vec::new(),
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
    model.verifications[0].bindings = vec![binding("binding/one"), binding("binding/two")];
    assert!(model.verifications[0].qualifications.is_empty());
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
    model.verifications[0].qualifications = vec![Qualification {
        id: "binding/one".into(),
        verdict: QualificationVerdict::Rejected,
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
fn canonical_launch_vector_matches_d47() {
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
        }],
        fingerprint: fp('0'),
    };
    assert_eq!(
        launch_fingerprint(&launch),
        "sha256:980dc9e544f41414e3a2735e84a6d9733aee85b2961899bb538f1f34c4347237"
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

fn binding(id: &str) -> EvidenceBinding {
    EvidenceBinding {
        id: id.into(),
        check: "checks/alpha".into(),
        claim: format!("claims/example#{}", id.rsplit('/').next().unwrap()),
        proposition: "the Check bears on the Claim".into(),
        scope: model::Scope::Unit,
        quantification: model::Quantification::Example,
        oracle: model::Oracle::Direct,
        context: BTreeMap::new(),
        challenge_domain: Vec::new(),
        qualification_policy: "routine".into(),
        rationale: String::new(),
        path: "verification.md".into(),
        line: 1,
    }
}
