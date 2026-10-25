#[allow(dead_code)]
#[path = "../src/adapter.rs"]
mod adapter;
mod assurance_review {
    pub use azimuth::assurance_review::*;
}
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
mod run_selection {
    pub use azimuth::run_selection::{execution_from_inputs, verification_target_kind};
}
mod validation {
    pub use azimuth::validation::*;
}

#[allow(dead_code)]
#[path = "../src/run_plan.rs"]
mod run_plan;

use adapter::{
    AdapterConfiguration, AdapterContent, AdapterEnvironment, AdapterLimits, Capability,
    CapabilityClass, ConfiguredAdapter, ConfiguredFile,
};
use azimuth::account_verification::{parse_account_verification, AccountVerification};
use model::{CheckImplementation, Model, Site, SourceIdentity};
use run::{ArtifactState, Plan, VerificationTarget, WorkUnit};
use run_plan::*;
use spec::parse_spec;
use std::collections::BTreeMap;
use std::path::PathBuf;

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

fn verification(path: &str, checks: &[&str]) -> AccountVerification {
    let declarations = checks
        .iter()
        .map(|id| {
            format!(
                "#### Check: {id}\n- Evidence bindings:\n  - Contribution: Observes the subject working.\n\n\
                 Evaluate the synthetic predicate.\n"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    parse_account_verification(
        path,
        &format!(
            "# Verification: synthetic\n\n## Requirement verification: works\n\n\
             ### Scenario verification: works\n\n{declarations}"
        ),
    )
    .unwrap()
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
        "# Spec: synthetic\n\n## Requirement: works\nVerification: reviewed\n\nThe synthetic subject SHALL work.\n\n### Scenario: works\nEvent: checked\nRequired: it works\n",
    )
    .unwrap();
    Model {
        account_verifications: vec![verification("verification.md", checks)],
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
        "# Spec: alpha\n\n## Requirement: behavior\nVerification: reviewed\n\nThe system SHALL work.\n\n### Scenario: works\nEvent: invoked\nRequired: it works\n",
    )
    .unwrap();
    let verification = parse_account_verification(
        "alpha/verification.md",
        "# Verification: alpha\n\n## Requirement verification: behavior\n\n\
         ### Verification Challenge: search\n- Form: `mutation`\n- Targets: `implementation`\n\
         - Checks: `check`\n\nSearch for a change the Check cannot detect.\n\n\
         ### Scenario verification: works\n\n#### Check: check\n- Evidence bindings:\n  \
         - Contribution: Observes the behavior working.\n\nInvoke the behavior.\n",
    )
    .unwrap();
    Model {
        specs: vec![spec],
        account_verifications: vec![verification],
        check_implementations: vec![implementation("check", "checks::alpha")],
        realizes: vec![Site {
            spec: "alpha".into(),
            requirement: "behavior".into(),
            site: "alpha::works".into(),
            file: "src/alpha.rs".into(),
            lang: "rust".into(),
            source: Some(SourceIdentity {
                area: "core".into(),
                kind: "rust-item".into(),
                address: "alpha::works".into(),
                mount: "root".into(),
            }),
            source_fingerprint: fp('b'),
        }],
        ..Default::default()
    }
}

/// Mirrors the prepared source scope that is produced outside the planner for the declared
/// Verification Challenge `search`.
fn scope_report(sites: Vec<(&str, &str, &str, &str, String)>) -> json::Json {
    use json::Json;
    let payload = Json::obj(vec![
        ("format", Json::str("azimuth-challenge-source-scope")),
        ("version", Json::Num(2.0)),
        ("status", Json::str("complete")),
        ("verification_challenge", Json::str("search")),
        (
            "request",
            Json::obj(vec![
                ("requirements", Json::Arr(vec![Json::str("behavior")])),
                ("mechanisms", Json::Arr(Vec::new())),
                ("checks", Json::Arr(vec![Json::str("check")])),
            ]),
        ),
        (
            "sites",
            Json::Arr(
                sites
                    .into_iter()
                    .map(|(relationship, entity, site, file, fingerprint)| {
                        Json::obj(vec![
                            ("relationship", Json::str(relationship)),
                            ("entity", Json::str(entity)),
                            ("site", Json::str(site)),
                            ("file", Json::str(file)),
                            ("source_fingerprint", Json::str(&fingerprint)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    let fingerprint = run::canonical_fingerprint(&payload).unwrap();
    let Json::Obj(mut fields) = payload else {
        unreachable!()
    };
    fields.push(("fingerprint".into(), Json::str(&fingerprint)));
    Json::Obj(fields)
}

fn check_scope() -> json::Json {
    scope_report(vec![(
        "check",
        "check",
        "checks::alpha",
        "src/check.rs",
        fp('a'),
    )])
}

fn challenge_request_with(operation: RunOperation, scope: json::Json) -> PlanRequest {
    let scope_fingerprint = scope
        .get("fingerprint")
        .and_then(json::Json::as_str)
        .unwrap()
        .to_string();
    let mut request = request(&[], operation, "demo/alpha");
    request.challenges = vec![RequestedChallenge {
        assessment: None,
        assessment_history: Vec::new(),
        run_history: Vec::new(),
        scope_report: Some(scope),
        id: "search".into(),
        capability: "demo/challenge".into(),
        max_candidates: 1,
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::from([("scope_fingerprint".into(), scope_fingerprint)]),
        }],
    }];
    request
}

fn challenge_request(operation: RunOperation) -> PlanRequest {
    challenge_request_with(operation, check_scope())
}

fn request(ids: &[&str], operation: RunOperation, capability: &str) -> PlanRequest {
    PlanRequest {
        operation,
        planned_at_ms: 1_787_300_000_000,
        verification_target: VerificationTarget::Artifact {
            artifacts: vec![ArtifactState {
                id: "image".into(),
                digest: fp('8'),
            }],
        },
        required_execution_context: BTreeMap::from([("platform".into(), "linux".into())]),
        checks: ids
            .iter()
            .map(|id| RequestedCheck {
                id: (*id).into(),
                capability: capability.into(),
                scenarios: vec!["works".into()],
                units: vec![unit("whole")],
            })
            .collect(),
        challenges: Vec::new(),
    }
}

#[test]
fn request_parser_preserves_exact_verification_target_time_context_operation_and_units() {
    let source = plan_request_to_json(&request(&["alpha"], RunOperation::Import, "demo/reports"))
        .to_string_pretty();
    let parsed = parse_plan_request("request.json", &source).unwrap();
    assert_eq!(parsed.operation, RunOperation::Import);
    assert_eq!(parsed.planned_at_ms, 1_787_300_000_000);
    assert_eq!(parsed.required_execution_context["platform"], "linux");
    assert_eq!(
        parsed.verification_target,
        request(&["alpha"], RunOperation::Import, "demo/reports").verification_target
    );
    assert_eq!(parsed.checks[0].units, vec![unit("whole")]);

    let source = plan_request_to_json(&challenge_request(RunOperation::Execute)).to_string_pretty();
    let parsed = parse_plan_request("request.json", &source).unwrap();
    assert!(parsed.checks.is_empty());
    assert_eq!(parsed.challenges[0].id, "search");
    assert!(parsed.challenges[0].scope_report.is_some());
    assert_eq!(parsed.challenges[0].max_candidates, 1);
}

#[test]
fn request_parser_rejects_unknown_duplicate_and_noncanonical_members() {
    let valid = plan_request_to_json(&request(&["alpha"], RunOperation::Execute, "demo/alpha"))
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

    let mut unsorted = request(&["beta", "alpha"], RunOperation::Execute, "demo/alpha");
    assert!(plan(&model(&["alpha", "beta"]), &configuration(), &unsorted).is_err());
    unsorted.checks[0].id = "alpha".into();
    assert!(plan(&model(&["alpha"]), &configuration(), &unsorted).is_err());
}

#[test]
fn planner_uses_the_complete_model_before_selecting_checks() {
    let first = model(&["alpha"]);
    let second = model(&["alpha", "unselected"]);
    let request = request(&["alpha"], RunOperation::Execute, "demo/alpha");
    let first = plan(&first, &configuration(), &request).unwrap();
    let second = plan(&second, &configuration(), &request).unwrap();
    assert_eq!(first.plan.checks, second.plan.checks);
    assert_ne!(first.plan.model_fingerprint, second.plan.model_fingerprint);
    assert_ne!(first.plan.fingerprint, second.plan.fingerprint);
}

#[test]
fn unknown_and_duplicate_model_checks_fail_closed() {
    let unknown = request(&["missing"], RunOperation::Execute, "demo/alpha");
    assert!(
        plan(&model(&["alpha"]), &configuration(), &unknown).unwrap_err()[0]
            .detail
            .contains("unknown Check")
    );

    let mut duplicate = model(&["alpha"]);
    duplicate
        .account_verifications
        .push(verification("second/verification.md", &["alpha"]));
    assert!(plan(
        &duplicate,
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap_err()[0]
        .detail
        .contains("declared more than once"));
}

#[test]
fn unimplemented_and_unstable_checks_fail_closed() {
    let mut unimplemented = model(&["alpha"]);
    unimplemented.check_implementations.clear();
    let request = request(&["alpha"], RunOperation::Execute, "demo/alpha");
    assert!(
        plan(&unimplemented, &configuration(), &request).unwrap_err()[0]
            .detail
            .contains("no implementation")
    );

    let mut unstable = model(&["alpha"]);
    unstable.check_implementations[0].source = None;
    assert!(plan(&unstable, &configuration(), &request).unwrap_err()[0]
        .detail
        .contains("stable SourceIdentity"));
}

#[test]
fn planner_includes_the_complete_sorted_implementation_closure() {
    let mut model = model(&["alpha"]);
    model.check_implementations = vec![
        implementation("alpha", "checks::zeta"),
        implementation("alpha", "checks::alpha"),
    ];
    let launch = plan(
        &model,
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
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
    let model = model(&["alpha"]);
    let import_through_execute = request(&["alpha"], RunOperation::Import, "demo/alpha");
    assert!(
        plan(&model, &configuration(), &import_through_execute).unwrap_err()[0]
            .detail
            .contains("check.import")
    );
    let unknown = request(&["alpha"], RunOperation::Execute, "other/alpha");
    assert!(plan(&model, &configuration(), &unknown).unwrap_err()[0]
        .detail
        .contains("unknown configured capability"));
}

#[test]
fn one_launch_routes_checks_through_explicitly_configured_adapters() {
    let mut config = configuration();
    let mut other = config.adapters[0].clone();
    other.id = "other".into();
    other.adapter_fingerprint = fp('a');
    other.descriptor_fingerprint = fp('b');
    other.configuration_fingerprint = fp('c');
    config.adapters.push(other);
    let mut request = request(&["alpha", "beta"], RunOperation::Execute, "demo/alpha");
    request.checks[1].capability = "other/alpha".into();
    let launch = plan(&model(&["alpha", "beta"]), &config, &request).unwrap();
    assert_eq!(launch.adapters.len(), 2);
    assert_eq!(launch.adapter.id, "azimuth-coordinator");
    assert!(validate_launch_configuration(&launch, &config).is_empty());
    let encoded = run::canonical_json(&launch_plan_to_json(&launch)).unwrap();
    assert_eq!(parse_launch_plan("launch.json", &encoded).unwrap(), launch);
}

#[test]
fn capability_substitution_changes_launch_identity_but_not_semantic_plan() {
    let model = model(&["alpha"]);
    let alpha = plan(
        &model,
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();
    let beta = plan(
        &model,
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/beta"),
    )
    .unwrap();
    assert_eq!(alpha.plan, beta.plan);
    assert_ne!(alpha.fingerprint, beta.fingerprint);
}

#[test]
fn exact_verification_target_context_units_time_and_operation_are_identity_bearing() {
    let model = model(&["alpha"]);
    let config = configuration();
    let base_request = request(&["alpha"], RunOperation::Execute, "demo/alpha");
    let base = plan(&model, &config, &base_request).unwrap();

    let mut context = base_request.clone();
    context
        .required_execution_context
        .insert("platform".into(), "macos".into());
    let context = plan(&model, &config, &context).unwrap();
    assert_ne!(base.plan.fingerprint, context.plan.fingerprint);

    let mut units = base_request.clone();
    units.checks[0].units[0]
        .parameters
        .insert("shard".into(), "one".into());
    let units = plan(&model, &config, &units).unwrap();
    assert_ne!(base.plan.fingerprint, units.plan.fingerprint);

    let mut verification_target = base_request.clone();
    verification_target.verification_target = VerificationTarget::Artifact {
        artifacts: vec![ArtifactState {
            id: "image".into(),
            digest: fp('9'),
        }],
    };
    let verification_target = plan(&model, &config, &verification_target).unwrap();
    assert_ne!(
        base.verification_target_fingerprint,
        verification_target.verification_target_fingerprint
    );
    assert_ne!(base.fingerprint, verification_target.fingerprint);

    let mut planned_time = base_request.clone();
    planned_time.planned_at_ms += 1;
    let planned_time = plan(&model, &config, &planned_time).unwrap();
    assert_eq!(base.plan, planned_time.plan);
    assert_ne!(base.fingerprint, planned_time.fingerprint);

    let imported = plan(
        &model,
        &config,
        &request(&["alpha"], RunOperation::Import, "demo/reports"),
    )
    .unwrap();
    assert_eq!(base.plan, imported.plan);
    assert_ne!(base.fingerprint, imported.fingerprint);
}

#[test]
fn several_bindings_still_produce_one_check_and_no_challenges() {
    let mut model = model(&["alpha"]);
    let mut other_scenario = model.specs[0].requirements[0].scenarios[0].clone();
    other_scenario.id = "other".into();
    model.specs[0].requirements[0]
        .scenarios
        .push(other_scenario);
    let mut second = model.account_verifications[0].checks[0].bindings[0].clone();
    second.scenario = "other".into();
    model.account_verifications[0].checks[0]
        .bindings
        .push(second);
    let launch = plan(
        &model,
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();
    assert_eq!(launch.plan.checks.len(), 1);
    assert!(launch.plan.challenges.is_empty());
}

#[test]
fn route_order_cardinality_and_configured_capability_identity_fail_closed() {
    let model = model(&["alpha", "beta"]);
    let config = configuration();
    let launch = plan(
        &model,
        &config,
        &request(&["alpha", "beta"], RunOperation::Execute, "demo/alpha"),
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
        &model(&["alpha"]),
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
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
    let source = plan_request_to_json(&request(&["alpha"], RunOperation::Execute, "demo/alpha"))
        .to_string_pretty();
    let unsafe_number = source.replacen("1787300000000", "9007199254740992", 1);
    assert!(parse_plan_request("request.json", &unsafe_number).is_err());
    let fractional = source.replacen("1787300000000", "1787300000000.5", 1);
    assert!(parse_plan_request("request.json", &fractional).is_err());
}

#[test]
fn typed_launch_validation_never_hashes_unsafe_numbers() {
    let valid = plan(
        &model(&["alpha"]),
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
    )
    .unwrap();

    let mut unsafe_time = valid.clone();
    unsafe_time.planned_at_ms = 9_007_199_254_740_992;
    let errors = std::panic::catch_unwind(|| validate_launch_plan(&unsafe_time)).unwrap();
    assert!(errors
        .iter()
        .any(|error| error.contains("planned_at_ms exceeds")));

    let mut unsafe_verification_target = valid;
    unsafe_verification_target.verification_target = VerificationTarget::MonitoringWindow {
        environment: "staging".into(),
        services: vec![run::ServiceState {
            service: "api".into(),
            deployment: "candidate".into(),
            deployment_fingerprint: fp('f'),
        }],
        window_start_ms: 1,
        window_end_ms: 9_007_199_254_740_992,
    };
    let errors =
        std::panic::catch_unwind(|| validate_launch_plan(&unsafe_verification_target)).unwrap();
    assert!(errors
        .iter()
        .any(|error| error.contains("maximum safe integer")));
}

#[test]
fn launch_round_trip_is_deterministic_and_rejects_substitution() {
    let launch = plan(
        &model(&["alpha"]),
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
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
    let verification_target = VerificationTarget::Artifact {
        artifacts: vec![ArtifactState {
            id: "image".into(),
            digest: fp('4'),
        }],
    };
    let mut launch = LaunchPlan {
        adapters: Vec::new(),
        operation: RunOperation::Execute,
        planned_at_ms: 1_787_300_000_000,
        verification_target,
        verification_target_fingerprint:
            "sha256:339156da3a55df33136e71859f1a3482bb39ee9aa45c11a30780334ee9e63759".into(),
        plan: Plan {
            model_fingerprint: fp('8'),
            required_execution_context: BTreeMap::new(),
            checks: vec![run::CheckSelection {
                id: "check".into(),
                fingerprint: fp('6'),
                scenarios: vec!["works".into()],
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
                id: "check".into(),
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
    launch.adapters = vec![launch.adapter.clone()];
    assert_eq!(
        launch_fingerprint(&launch),
        "sha256:efe730885be229f75122c7e69a69fb931d6abfe7587336ad47f790fd03320179"
    );
}

#[test]
fn planning_and_finalization_share_the_exact_model_fingerprint() {
    let model = model(&["alpha"]);
    let findings = validation::validate(&model);
    let expected = fingerprint::model_digest(&model, &findings);
    let (finalized, _) = azimuth::change::finalization(&model, &findings, &[], &[]);
    let launch = plan(
        &model,
        &configuration(),
        &request(&["alpha"], RunOperation::Execute, "demo/alpha"),
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
        id: "check".into(),
        capability: "demo/alpha".into(),
        scenarios: vec!["works".into()],
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
fn import_challenges_route_through_the_import_class() {
    let launch = plan(
        &challenge_model(),
        &configuration(),
        &challenge_request(RunOperation::Import),
    )
    .unwrap();
    assert_eq!(launch.plan.challenges.len(), 1);
    assert_eq!(
        launch.routes[0].capability.class,
        run::RouteCapabilityClass::ChallengeImport
    );
}

#[test]
fn declared_challenge_planning_fails_closed() {
    let config = configuration();
    let unknown = {
        let mut request = challenge_request(RunOperation::Execute);
        request.challenges[0].id = "missing".into();
        request
    };
    let mut review_only = challenge_model();
    review_only.account_verifications[0].verification_challenges[0].targets =
        vec!["verification-review".into()];
    let mut moved = challenge_model();
    moved.check_implementations[0].site = "checks::moved".into();
    let mut drifted_scope = check_scope();
    if let json::Json::Obj(fields) = &mut drifted_scope {
        for (key, value) in fields.iter_mut() {
            if key == "request" {
                *value = json::Json::obj(vec![
                    (
                        "requirements",
                        json::Json::Arr(vec![json::Json::str("behavior")]),
                    ),
                    ("mechanisms", json::Json::Arr(Vec::new())),
                    ("checks", json::Json::Arr(Vec::new())),
                ]);
            }
        }
    }
    let mut unpinned = challenge_request(RunOperation::Execute);
    unpinned.challenges[0].units[0].parameters.clear();
    for (model, request, expected) in [
        (
            challenge_model(),
            unknown,
            "does not select one compact machine Verification Challenge",
        ),
        (
            review_only,
            challenge_request(RunOperation::Execute),
            "must declare Targets: implementation",
        ),
        (
            moved,
            challenge_request(RunOperation::Execute),
            "one current qualified identity",
        ),
        (
            challenge_model(),
            challenge_request_with(RunOperation::Execute, drifted_scope),
            "scope report fingerprint mismatch",
        ),
        (
            challenge_model(),
            unpinned,
            "must pin the prepared scope_fingerprint",
        ),
    ] {
        let errors = plan(&model, &config, &request).unwrap_err();
        assert!(
            errors.iter().any(|error| error.detail.contains(expected)),
            "{expected}: {errors:?}"
        );
    }
}

#[test]
fn realization_relocation_changes_only_launch_accounting() {
    let scope = |site: &str, file: &str| {
        scope_report(vec![
            ("check", "check", "checks::alpha", "src/check.rs", fp('a')),
            ("requirement", "behavior", site, file, fp('b')),
        ])
    };
    let model = challenge_model();
    let first = plan(
        &model,
        &configuration(),
        &challenge_request_with(RunOperation::Execute, scope("alpha::works", "src/alpha.rs")),
    )
    .unwrap();
    assert!(first.plan.challenges[0]
        .scope
        .inputs
        .iter()
        .any(|item| item.kind == run::ChallengeScopeKind::Realization));

    let mut relocated_model = challenge_model();
    relocated_model.realizes[0].file = "moved/alpha.rs".into();
    relocated_model.realizes[0].site = "moved::alpha::works".into();
    let relocated = plan(
        &relocated_model,
        &configuration(),
        &challenge_request_with(
            RunOperation::Execute,
            scope("moved::alpha::works", "moved/alpha.rs"),
        ),
    )
    .unwrap();
    assert_eq!(
        first.plan.challenges[0].scope.inputs,
        relocated.plan.challenges[0].scope.inputs
    );
    assert_ne!(first.routes[0].inputs, relocated.routes[0].inputs);
    assert!(relocated.routes[0].inputs.iter().any(|input| matches!(
        &input.source,
        run::LaunchInputSource::Source { file, site, .. }
            if file == "moved/alpha.rs" && site == "moved::alpha::works"
    )));
    assert_ne!(first.fingerprint, relocated.fingerprint);
}
