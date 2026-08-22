use azimuth::adapter::{
    self, AdapterContent, AdapterEnvironment, AdapterLimits, Capability, CapabilityClass,
    ConfiguredAdapter, ConfiguredFile,
};
use azimuth::json::Json;
use azimuth::run::*;
use azimuth::run_plan::{self, PlanRequest, RequestedChallenge, RequestedCheck, RunOperation};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-adapter-cli-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn azimuth(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_azimuth"))
        .args(arguments)
        .output()
        .unwrap()
}

fn fp(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn empty_configuration(path: &Path) {
    fs::write(
        path,
        "{\"format\":\"azimuth-adapter-configuration\",\"version\":1,\"adapters\":[]}",
    )
    .unwrap();
}

fn configuration(root: &Path) -> PathBuf {
    configuration_with_script(root, "#!/bin/sh\nexit 1\n").0
}

fn configuration_with_script(root: &Path, script: &str) -> (PathBuf, ConfiguredAdapter) {
    let executable = root.join("adapter");
    fs::write(&executable, script).unwrap();
    let executable = executable.canonicalize().unwrap();
    let executable_identity = adapter::identify_input("adapter", &executable).unwrap();
    let mut configured = ConfiguredAdapter {
        id: "synthetic".into(),
        provider_family: "synthetic/test".into(),
        protocol_version: 1,
        adapter_version: "alpha.2-test".into(),
        build: "cli-fixture".into(),
        content: AdapterContent {
            executable: ConfiguredFile {
                locator: executable.display().to_string(),
                resolved: executable.clone(),
                digest: executable_identity.digest,
            },
            resources: Vec::new(),
        },
        semantic_settings: BTreeMap::new(),
        environment: AdapterEnvironment {
            literals: BTreeMap::new(),
        },
        limits: AdapterLimits {
            timeout_ms: 5_000,
            stdout_bytes: 1_000_000,
            stderr_bytes: 1_024,
        },
        capabilities: vec![
            Capability {
                id: "challenges".into(),
                classes: vec![
                    CapabilityClass::ChallengeExecute,
                    CapabilityClass::ChallengeImport,
                ],
                challenge_forms: vec!["mutation".into()],
                semantic_settings: BTreeMap::new(),
                fingerprint: String::new(),
            },
            Capability {
                id: "checks".into(),
                classes: vec![CapabilityClass::CheckExecute, CapabilityClass::CheckImport],
                challenge_forms: Vec::new(),
                semantic_settings: BTreeMap::new(),
                fingerprint: String::new(),
            },
        ],
        adapter_fingerprint: String::new(),
        descriptor_fingerprint: String::new(),
        configuration_fingerprint: String::new(),
    };
    configured.adapter_fingerprint = adapter::adapter_fingerprint(&configured);
    for capability in &mut configured.capabilities {
        capability.fingerprint =
            adapter::capability_fingerprint(&configured.adapter_fingerprint, capability);
    }
    configured.descriptor_fingerprint =
        adapter::descriptor_fingerprint(&configured.expected_description());
    configured.configuration_fingerprint = adapter::configuration_fingerprint(&configured);

    let json = Json::obj(vec![
        ("format", Json::str("azimuth-adapter-configuration")),
        ("version", Json::Num(1.0)),
        (
            "adapters",
            Json::Arr(vec![Json::obj(vec![
                ("id", Json::str(&configured.id)),
                ("provider_family", Json::str(&configured.provider_family)),
                ("protocol_version", Json::Num(1.0)),
                ("adapter_version", Json::str(&configured.adapter_version)),
                ("build", Json::str(&configured.build)),
                (
                    "content",
                    Json::obj(vec![
                        (
                            "executable",
                            Json::obj(vec![
                                ("locator", Json::str(&configured.content.executable.locator)),
                                ("digest", Json::str(&configured.content.executable.digest)),
                            ]),
                        ),
                        ("resources", Json::Arr(Vec::new())),
                    ]),
                ),
                ("semantic_settings", Json::Obj(Vec::new())),
                (
                    "environment",
                    Json::obj(vec![("literals", Json::Obj(Vec::new()))]),
                ),
                (
                    "limits",
                    Json::obj(vec![
                        ("timeout_ms", Json::Num(5_000.0)),
                        ("stdout_bytes", Json::Num(1_000_000.0)),
                        ("stderr_bytes", Json::Num(1_024.0)),
                    ]),
                ),
                (
                    "capabilities",
                    Json::Arr(
                        configured
                            .capabilities
                            .iter()
                            .map(|capability| {
                                Json::obj(vec![
                                    ("id", Json::str(&capability.id)),
                                    (
                                        "classes",
                                        Json::Arr(
                                            capability
                                                .classes
                                                .iter()
                                                .map(|class| Json::str(class.name()))
                                                .collect(),
                                        ),
                                    ),
                                    (
                                        "challenge_forms",
                                        Json::Arr(
                                            capability
                                                .challenge_forms
                                                .iter()
                                                .map(Json::str)
                                                .collect(),
                                        ),
                                    ),
                                    ("semantic_settings", Json::Obj(Vec::new())),
                                    ("fingerprint", Json::str(&capability.fingerprint)),
                                ])
                            })
                            .collect(),
                    ),
                ),
                (
                    "adapter_fingerprint",
                    Json::str(&configured.adapter_fingerprint),
                ),
                (
                    "descriptor_fingerprint",
                    Json::str(&configured.descriptor_fingerprint),
                ),
                (
                    "configuration_fingerprint",
                    Json::str(&configured.configuration_fingerprint),
                ),
            ])]),
        ),
    ]);
    let path = root.join("adapters.json");
    fs::write(&path, json.to_string_pretty()).unwrap();
    adapter::load_configuration(&path).unwrap();
    (path, configured)
}

fn model(root: &Path, checks: &[&str]) -> (PathBuf, PathBuf, PathBuf) {
    let model = root.join("model");
    fs::create_dir(&model).unwrap();
    let package = model.join("demo");
    fs::create_dir(&package).unwrap();
    fs::write(
        package.join("spec.md"),
        "# Spec: demo\n\n## Requirement: works\nCriticality: routine\n\n\
         The demo SHALL work.\n\n### Scenario: works\nWHEN invoked\nTHEN it works\n",
    )
    .unwrap();
    let declarations = checks
        .iter()
        .map(|id| format!("## Check: {id}\nMethod: invoke\nTerminal: works\n\nOne result.\n"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        package.join("verification.md"),
        format!("# Verification: demo\n\n{declarations}"),
    )
    .unwrap();
    let workspace = root.join("workspace.json");
    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":1,\
         \"areas\":[{\"id\":\"core\",\"mounts\":[{\"id\":\"code\",\"path\":\"src\"}]}],\
         \"surfaces\":[],\"realization_obligations\":[]}",
    )
    .unwrap();
    let manifest = root.join("manifest.json");
    let implementations = checks
        .iter()
        .enumerate()
        .map(|(index, id)| {
            format!(
                "{{\"check\":\"{id}\",\"site\":\"checks::{index}\",\
                 \"file\":\"src/check-{index}.rs\",\"lang\":\"rust-symbol\",\
                 \"source_fingerprint\":\"{}\"}}",
                fp((b'a' + index as u8) as char)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    fs::write(
        &manifest,
        format!(
            "{{\"check_implementations\":[{implementations}],\"artifacts\":[],\
             \"class_members\":[],\"enumerations\":[]}}"
        ),
    )
    .unwrap();
    (model, workspace, manifest)
}

fn challenge_model(root: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let (model, workspace, manifest) = model(root, &["demo/check"]);
    fs::write(
        model.join("demo/spec.md"),
        "# Spec: demo\n\n## Requirement: works\nCriticality: standard\n\n\
         The demo SHALL work.\n\n### Scenario: works\nWHEN invoked\nTHEN it works\n",
    )
    .unwrap();
    let standards = root.join("standards.md");
    fs::write(
        &standards,
        "# Decision policies and Challenge schedule\n\n\
         ## Decision Policy: credible\nRequired challenge: mutation\n\n\
         The decision must be challenged.\n\n\
         ## Challenge Schedule: current\nGate challenge: mutation\n\nMutation is gate work.\n",
    )
    .unwrap();
    let verification = model.join("demo/verification.md");
    let render = |fingerprint: &str, verdict: &str, form: &str, selector: &str| {
        format!(
            "# Verification: demo\n\n\
             ## Check: demo/check\nMethod: invoke\nTerminal: works\n\nOne result.\n\n\
             ## Evidence Binding: demo/edge\nCheck: demo/check\nClaim: demo#works\n\
             Proposition: direct\nScope: unit\nQuantification: example\nOracle: direct\n\
             Context: {{\"platform\":\"linux\"}}\nChallenge domain: [\"realization\",\"mechanism\"]\n\
             Policy: credible\n\nReviewable.\n\n\
             ## Qualification: demo/edge\nVerdict: {verdict}\nFingerprint: {fingerprint}\n\
             Qualified: 2026-08-22\nQualifier: owner\n\nQualified.\n\n\
             ## Challenger: mutation/search\nForm: {form}\nSearches for: an undetected change\n\
             Required scope: [\"binding\",\"check-implementation\"]\n\nSearches exact semantics.\n\n\
             ## Challenge Plan: demo/plan\nChallenger: mutation/search\nSelect: {selector}\n\n\
             Targets the current decision.\n"
        )
    };
    fs::write(
        &verification,
        render(
            &fp('0'),
            "qualified",
            "mutation",
            "qualification from binding demo/edge",
        ),
    )
    .unwrap();
    let loaded = azimuth::load(
        &model,
        &standards,
        &workspace,
        std::slice::from_ref(&manifest),
        &[],
    )
    .unwrap();
    let fingerprint = loaded
        .model
        .expected_qualification_fingerprint(&loaded.model.verifications[0].bindings[0])
        .unwrap();
    fs::write(
        &verification,
        render(
            &fingerprint,
            "qualified",
            "mutation",
            "qualification from binding demo/edge",
        ),
    )
    .unwrap();
    (model, standards, workspace, manifest)
}

fn request(path: &Path, operation: RunOperation, check: &str) {
    let request = PlanRequest {
        operation,
        planned_at_ms: 1_787_300_000_000,
        subject: Subject::Artifact {
            artifacts: vec![ArtifactState {
                id: "candidate".into(),
                digest: fp('8'),
            }],
        },
        required_context: BTreeMap::new(),
        checks: vec![RequestedCheck {
            id: check.into(),
            capability: "synthetic/checks".into(),
            units: vec![WorkUnit {
                id: "whole".into(),
                parameters: BTreeMap::new(),
            }],
        }],
        challenges: Vec::new(),
    };
    fs::write(
        path,
        run_plan::plan_request_to_json(&request).to_string_pretty(),
    )
    .unwrap();
}

fn challenge_request(
    path: &Path,
    operation: RunOperation,
    checks: bool,
    context: &str,
    capability: &str,
    max_candidates: u64,
) {
    let request = PlanRequest {
        operation,
        planned_at_ms: 1_787_300_000_000,
        subject: Subject::Artifact {
            artifacts: vec![ArtifactState {
                id: "candidate".into(),
                digest: fp('8'),
            }],
        },
        required_context: BTreeMap::from([("platform".into(), context.into())]),
        checks: if checks {
            vec![RequestedCheck {
                id: "demo/check".into(),
                capability: "synthetic/checks".into(),
                units: vec![WorkUnit {
                    id: "whole".into(),
                    parameters: BTreeMap::new(),
                }],
            }]
        } else {
            Vec::new()
        },
        challenges: vec![RequestedChallenge {
            id: "demo/plan".into(),
            capability: capability.into(),
            max_candidates,
            units: vec![WorkUnit {
                id: "whole".into(),
                parameters: BTreeMap::new(),
            }],
        }],
    };
    fs::write(
        path,
        run_plan::plan_request_to_json(&request).to_string_pretty(),
    )
    .unwrap();
}

fn planning_arguments<'a>(
    request: &'a Path,
    model: &'a Path,
    workspace: &'a Path,
    manifest: &'a Path,
    config: &'a Path,
) -> Vec<&'a str> {
    vec![
        "run",
        "plan",
        "--request",
        request.to_str().unwrap(),
        "--model",
        model.to_str().unwrap(),
        "--workspace",
        workspace.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
    ]
}

fn challenge_planning_arguments<'a>(
    request: &'a Path,
    model: &'a Path,
    standards: &'a Path,
    workspace: &'a Path,
    manifest: &'a Path,
    config: &'a Path,
) -> Vec<&'a str> {
    let mut arguments = planning_arguments(request, model, workspace, manifest, config);
    arguments.extend(["--standards", standards.to_str().unwrap()]);
    arguments
}

fn add_second_challenge_candidate(
    model: &Path,
    standards: &Path,
    workspace: &Path,
    manifest: &Path,
) {
    let manifest_source = fs::read_to_string(manifest).unwrap();
    fs::write(
        manifest,
        manifest_source.replacen(
            '{',
            &format!(
                "{{\"realizes\":[{{\"spec\":\"demo\",\"scenario\":\"works\",\
                 \"site\":\"demo::works\",\"file\":\"src/demo.rs\",\
                 \"lang\":\"rust-symbol\",\"source_fingerprint\":\"{}\"}}],",
                fp('2')
            ),
            1,
        ),
    )
    .unwrap();
    let verification = model.join("demo/verification.md");
    let source = fs::read_to_string(&verification).unwrap();
    let source = source
        .replace(
            "## Challenger: mutation/search",
            &format!(
                "## Claim Judgment: demo#works\nVerdict: accepted\nPolicy: credible\nFingerprint: {}\n\
                 Judged: 2026-08-22\nJudge: owner\nBasis: the exact composition is accepted\n\
                 Residual risk: none identified\n\nAccepted.\n\n\
                 ## Challenger: mutation/search",
                fp('1')
            ),
        )
        .replace(
            "Select: qualification from binding demo/edge",
            "Select: qualification from binding demo/edge\n\
             Select: claim-judgment from claim demo#works",
        );
    fs::write(&verification, source).unwrap();
    let loaded = azimuth::load(
        model,
        standards,
        workspace,
        std::slice::from_ref(&manifest.to_path_buf()),
        &[],
    )
    .unwrap();
    let fingerprint = loaded
        .model
        .expected_claim_judgment_fingerprint(&loaded.model.verifications[0].claim_judgments[0])
        .unwrap();
    let source = fs::read_to_string(&verification)
        .unwrap()
        .replace(&fp('1'), &fingerprint);
    fs::write(verification, source).unwrap();
}

fn description_json(adapter: &ConfiguredAdapter) -> Json {
    let description = adapter.expected_description();
    Json::obj(vec![
        ("format", Json::str("azimuth-adapter-description")),
        ("version", Json::Num(1.0)),
        ("protocol_version", Json::Num(1.0)),
        ("id", Json::str(&description.id)),
        ("provider_family", Json::str(&description.provider_family)),
        ("adapter_version", Json::str(&description.adapter_version)),
        ("build", Json::str(&description.build)),
        (
            "content",
            Json::obj(vec![
                (
                    "executable_digest",
                    Json::str(&description.content.executable_digest),
                ),
                ("resources", Json::Arr(Vec::new())),
            ]),
        ),
        (
            "adapter_fingerprint",
            Json::str(&description.adapter_fingerprint),
        ),
        (
            "capabilities",
            Json::Arr(
                description
                    .capabilities
                    .iter()
                    .map(|capability| {
                        Json::obj(vec![
                            ("id", Json::str(&capability.id)),
                            (
                                "classes",
                                Json::Arr(
                                    capability
                                        .classes
                                        .iter()
                                        .map(|class| Json::str(class.name()))
                                        .collect(),
                                ),
                            ),
                            (
                                "challenge_forms",
                                Json::Arr(
                                    capability.challenge_forms.iter().map(Json::str).collect(),
                                ),
                            ),
                            ("semantic_settings", Json::Obj(Vec::new())),
                            ("fingerprint", Json::str(&capability.fingerprint)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "descriptor_fingerprint",
            Json::str(&description.descriptor_fingerprint),
        ),
    ])
}

fn response_template(
    operation: &str,
    adapter: &ConfiguredAdapter,
    launch_fingerprint: Option<&str>,
    bundle: Option<&RunBundle>,
) -> String {
    let mut fields = vec![
        ("format".into(), Json::str("azimuth-adapter-response")),
        ("version".into(), Json::Num(1.0)),
        ("request_id".into(), Json::str("REQUEST_ID")),
        ("operation".into(), Json::str(operation)),
        ("status".into(), Json::str("ok")),
        ("description".into(), description_json(adapter)),
    ];
    if let Some(fingerprint) = launch_fingerprint {
        fields.push(("launch_fingerprint".into(), Json::str(fingerprint)));
    }
    if let Some(bundle) = bundle {
        fields.push(("bundle".into(), to_json(bundle)));
    }
    Json::Obj(fields).to_string_pretty()
}

fn refresh_bundle(bundle: &mut RunBundle) {
    bundle.subject_fingerprint = subject_fingerprint(&bundle.subject);
    bundle.plan.fingerprint = plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    bundle.run_id = run_id(bundle);
    for index in 0..bundle.check_executions.len() {
        bundle.check_executions[index].observation.fingerprint =
            observation_fingerprint(bundle, &bundle.check_executions[index]);
    }
    for index in 0..bundle.challenger_executions.len() {
        bundle.challenger_executions[index].result.fingerprint =
            challenge_result_fingerprint(bundle, &bundle.challenger_executions[index]);
    }
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}

fn adapter_bundle(
    launch: &run_plan::LaunchPlan,
    adapter: &ConfiguredAdapter,
    inputs: Vec<ImportInputIdentity>,
    adverse: bool,
) -> RunBundle {
    let mode = match launch.operation {
        RunOperation::Execute => ProvenanceMode::Execute,
        RunOperation::Import => ProvenanceMode::Import,
    };
    let mut bundle = RunBundle {
        run_id: fp('0'),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: fp('0'),
        subject: launch.subject.clone(),
        subject_fingerprint: launch.subject_fingerprint.clone(),
        planned_at_ms: launch.planned_at_ms,
        started_at_ms: launch.planned_at_ms + 1,
        finished_at_ms: launch.planned_at_ms + 2,
        status: if adverse {
            RunStatus::Complete
        } else {
            RunStatus::Partial
        },
        plan: launch.plan.clone(),
        actual_selection: ActualSelection {
            context: launch.plan.required_context.clone(),
            plan_fingerprint: launch.plan.fingerprint.clone(),
            checks: if adverse {
                launch.plan.checks.clone()
            } else {
                Vec::new()
            },
            challenges: Vec::new(),
            fingerprint: fp('0'),
        },
        provenance: Provenance {
            mode,
            source: SourceProvenance {
                system: "synthetic/cli".into(),
                execution: "native-1".into(),
                uri: None,
            },
            normalizer: Normalizer {
                id: format!("adapter/{}", adapter.id),
                version: adapter.adapter_version.clone(),
                build_fingerprint: adapter.adapter_fingerprint.clone(),
            },
            adapter: AdapterProvenance {
                id: adapter.id.clone(),
                adapter_version: adapter.adapter_version.clone(),
                adapter_fingerprint: adapter.adapter_fingerprint.clone(),
                descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
                configuration_fingerprint: adapter.configuration_fingerprint.clone(),
                launch_fingerprint: launch.fingerprint.clone(),
                routes: launch.routes.clone(),
                import_inputs: inputs,
            },
            generated_at_ms: launch.planned_at_ms + 3,
            principal: None,
            attributes: None,
        },
        artifacts: Vec::new(),
        diagnostics: Vec::new(),
        activities: Vec::new(),
        check_executions: Vec::new(),
        challenger_executions: Vec::new(),
    };
    if adverse {
        bundle.activities.push(Activity {
            id: "native-check".into(),
            status: ActivityStatus::Completed,
            started_at_ms: launch.planned_at_ms + 1,
            finished_at_ms: launch.planned_at_ms + 2,
            artifacts: Vec::new(),
            diagnostics: Vec::new(),
            attributes: BTreeMap::new(),
        });
        bundle.check_executions.push(CheckExecution {
            check: CheckRef {
                id: launch.plan.checks[0].id.clone(),
                fingerprint: launch.plan.checks[0].fingerprint.clone(),
            },
            units: vec![CheckExecutionUnit {
                id: "whole".into(),
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "native-check".into(),
                    outcome: ObservationOutcome::Violated,
                }],
            }],
            observation: Observation {
                outcome: ObservationOutcome::Violated,
                observed_at_ms: launch.planned_at_ms + 2,
                fingerprint: fp('0'),
                artifacts: Vec::new(),
                diagnostics: Vec::new(),
            },
        });
    }
    refresh_bundle(&mut bundle);
    assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
    bundle
}

fn challenge_adapter_bundle(
    launch: &run_plan::LaunchPlan,
    adapter: &ConfiguredAdapter,
    inputs: Vec<ImportInputIdentity>,
    outcome: Option<ChallengeOutcome>,
) -> RunBundle {
    let selection = launch.plan.challenges[0].clone();
    let mut diagnostics = Vec::new();
    let mut activities = Vec::new();
    let mut executions = Vec::new();
    let actual_challenges = if let Some(outcome) = outcome {
        let objections = if outcome == ChallengeOutcome::Findings {
            diagnostics.push(Diagnostic {
                id: "mutation/finding".into(),
                class: DiagnosticClass::Objection,
                severity: Severity::Error,
                code: "mutation/survived".into(),
                message: "A mutation survived.".into(),
                scope: DiagnosticScope::ChallengerExecution {
                    challenger_fingerprint: selection.challenger.fingerprint.clone(),
                    target_fingerprint: selection.target.fingerprint.clone(),
                },
                artifacts: Vec::new(),
                details: BTreeMap::new(),
            });
            vec!["mutation/finding".into()]
        } else {
            Vec::new()
        };
        activities.push(Activity {
            id: "challenge-attempt".into(),
            status: if outcome == ChallengeOutcome::Inconclusive {
                ActivityStatus::Failed
            } else {
                ActivityStatus::Completed
            },
            started_at_ms: launch.planned_at_ms + 1,
            finished_at_ms: launch.planned_at_ms + 2,
            artifacts: Vec::new(),
            diagnostics: objections.clone(),
            attributes: BTreeMap::new(),
        });
        executions.push(ChallengerExecution {
            challenge: selection.id.clone(),
            challenger: selection.challenger.clone(),
            target: selection.target.clone(),
            units: vec![ChallengeExecutionUnit {
                id: "whole".into(),
                attempts: vec![ChallengeAttempt {
                    ordinal: 1,
                    activity: "challenge-attempt".into(),
                    outcome,
                }],
            }],
            result: ChallengeResult {
                outcome,
                observed_at_ms: launch.planned_at_ms + 2,
                fingerprint: fp('0'),
                objections,
                artifacts: Vec::new(),
                diagnostics: Vec::new(),
            },
        });
        vec![selection.clone()]
    } else {
        diagnostics.push(Diagnostic {
            id: "challenge/deferred".into(),
            class: DiagnosticClass::Execution,
            severity: Severity::Warning,
            code: "challenge/deferred".into(),
            message: "The scheduled Challenge did not execute.".into(),
            scope: DiagnosticScope::ChallengeSelection(selection.id.clone()),
            artifacts: Vec::new(),
            details: BTreeMap::new(),
        });
        Vec::new()
    };
    let mode = match launch.operation {
        RunOperation::Execute => ProvenanceMode::Execute,
        RunOperation::Import => ProvenanceMode::Import,
    };
    let mut bundle = RunBundle {
        run_id: fp('0'),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: fp('0'),
        subject: launch.subject.clone(),
        subject_fingerprint: launch.subject_fingerprint.clone(),
        planned_at_ms: launch.planned_at_ms,
        started_at_ms: launch.planned_at_ms + 1,
        finished_at_ms: launch.planned_at_ms + 2,
        status: if outcome.is_some() {
            RunStatus::Complete
        } else {
            RunStatus::Partial
        },
        plan: launch.plan.clone(),
        actual_selection: ActualSelection {
            context: launch.plan.required_context.clone(),
            plan_fingerprint: launch.plan.fingerprint.clone(),
            checks: Vec::new(),
            challenges: actual_challenges,
            fingerprint: fp('0'),
        },
        provenance: Provenance {
            mode,
            source: SourceProvenance {
                system: "synthetic/cli".into(),
                execution: "native-challenge".into(),
                uri: None,
            },
            normalizer: Normalizer {
                id: format!("adapter/{}", adapter.id),
                version: adapter.adapter_version.clone(),
                build_fingerprint: adapter.adapter_fingerprint.clone(),
            },
            adapter: AdapterProvenance {
                id: adapter.id.clone(),
                adapter_version: adapter.adapter_version.clone(),
                adapter_fingerprint: adapter.adapter_fingerprint.clone(),
                descriptor_fingerprint: adapter.descriptor_fingerprint.clone(),
                configuration_fingerprint: adapter.configuration_fingerprint.clone(),
                launch_fingerprint: launch.fingerprint.clone(),
                routes: launch.routes.clone(),
                import_inputs: inputs,
            },
            generated_at_ms: launch.planned_at_ms + 3,
            principal: None,
            attributes: None,
        },
        artifacts: Vec::new(),
        diagnostics,
        activities,
        check_executions: Vec::new(),
        challenger_executions: executions,
    };
    refresh_bundle(&mut bundle);
    assert!(verify(&bundle).is_empty(), "{:?}", verify(&bundle));
    bundle
}

struct ProtocolFixture {
    config: PathBuf,
    adapter: ConfiguredAdapter,
    response: PathBuf,
    mode: PathBuf,
}

impl ProtocolFixture {
    fn new(root: &Path) -> Self {
        let response = root.join("adapter-response.json");
        let mode = root.join("adapter-mode");
        fs::write(&response, "{}").unwrap();
        fs::write(&mode, "response\n").unwrap();
        let script = format!(
            concat!(
                "#!/bin/sh\n",
                "request=$(/bin/cat)\n",
                "mode=$(/bin/cat '{}')\n",
                "case \"$mode\" in\n",
                "response) request_id=$(printf '%s' \"$request\" | /usr/bin/sed -n ",
                "'s/.*\"request_id\":\"\\([^\"]*\\)\".*/\\1/p'); ",
                "/usr/bin/sed \"s|REQUEST_ID|$request_id|g\" '{}' ;;\n",
                "malformed) printf '{{' ;;\n",
                "nonzero) exit 7 ;;\n",
                "esac\n"
            ),
            mode.display(),
            response.display()
        );
        let (config, adapter) = configuration_with_script(root, &script);
        Self {
            config,
            adapter,
            response,
            mode,
        }
    }

    fn respond(&self, source: String) {
        fs::write(&self.response, source).unwrap();
        fs::write(&self.mode, "response\n").unwrap();
    }

    fn mode(&self, mode: &str) {
        fs::write(&self.mode, format!("{mode}\n")).unwrap();
    }
}

#[test]
fn adapter_verify_is_silent_for_an_empty_configuration() {
    let root = root();
    let config = root.join("empty.json");
    empty_configuration(&config);
    for arguments in [
        vec!["adapter", "--help"],
        vec!["adapter", "verify", "--help"],
    ] {
        let output = azimuth(&arguments);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .contains("azimuth adapter verify"));
    }
    let output = azimuth(&["adapter", "verify", "--config", config.to_str().unwrap()]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nonempty_verify_and_execute_import_routes_publish_valid_bundles() {
    let root = root();
    let fixture = ProtocolFixture::new(&root);
    fixture.respond(response_template("describe", &fixture.adapter, None, None));
    let verified = azimuth(&[
        "adapter",
        "verify",
        "--config",
        fixture.config.to_str().unwrap(),
    ]);
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    assert!(verified.stdout.is_empty());

    let (model, workspace, manifest) = model(&root, &["demo/selected"]);
    let request_path = root.join("request.json");
    request(&request_path, RunOperation::Execute, "demo/selected");
    let planned = azimuth(&planning_arguments(
        &request_path,
        &model,
        &workspace,
        &manifest,
        &fixture.config,
    ));
    assert!(planned.status.success());
    let launch = run_plan::parse_launch_plan(
        "execute launch",
        std::str::from_utf8(&planned.stdout).unwrap(),
    )
    .unwrap();
    let launch_path = root.join("execute-launch.json");
    fs::write(&launch_path, planned.stdout).unwrap();
    let adverse = adapter_bundle(&launch, &fixture.adapter, Vec::new(), true);
    fixture.respond(response_template(
        "execute",
        &fixture.adapter,
        Some(&launch.fingerprint),
        Some(&adverse),
    ));
    let execute_arguments = [
        "run",
        "execute",
        "--plan",
        launch_path.to_str().unwrap(),
        "--config",
        fixture.config.to_str().unwrap(),
    ];
    let execute = azimuth(&execute_arguments);
    assert!(
        execute.status.success(),
        "{}",
        String::from_utf8_lossy(&execute.stderr)
    );
    assert_eq!(
        parse(
            "execute bundle",
            std::str::from_utf8(&execute.stdout).unwrap()
        )
        .unwrap(),
        adverse
    );
    let execute_out = root.join("execute-bundle.json");
    let mut execute_file_arguments = execute_arguments.to_vec();
    execute_file_arguments.extend(["--out", execute_out.to_str().unwrap()]);
    let execute_file = azimuth(&execute_file_arguments);
    assert!(execute_file.status.success());
    assert!(execute_file.stdout.is_empty());
    assert_eq!(execute.stdout, fs::read(&execute_out).unwrap());

    let predecessor_path = root.join("predecessor.json");
    fs::write(&predecessor_path, &execute.stdout).unwrap();
    let mut correction = adverse.clone();
    correction.bundle_revision = 1;
    correction.corrects = Some(adverse.bundle_fingerprint.clone());
    correction.correction_reason = Some("late-provider-account".into());
    correction.provenance.generated_at_ms += 1;
    refresh_bundle(&mut correction);
    assert!(verify_set(&[adverse.clone(), correction.clone()]).is_empty());
    fixture.respond(response_template(
        "execute",
        &fixture.adapter,
        Some(&launch.fingerprint),
        Some(&correction),
    ));
    let corrected = azimuth(&[
        "run",
        "execute",
        "--plan",
        launch_path.to_str().unwrap(),
        "--predecessor",
        predecessor_path.to_str().unwrap(),
        "--config",
        fixture.config.to_str().unwrap(),
    ]);
    assert!(
        corrected.status.success(),
        "{}",
        String::from_utf8_lossy(&corrected.stderr)
    );
    assert_eq!(
        parse(
            "corrected bundle",
            std::str::from_utf8(&corrected.stdout).unwrap()
        )
        .unwrap(),
        correction
    );

    request(&request_path, RunOperation::Import, "demo/selected");
    let import_plan = azimuth(&planning_arguments(
        &request_path,
        &model,
        &workspace,
        &manifest,
        &fixture.config,
    ));
    assert!(import_plan.status.success());
    let import_launch = run_plan::parse_launch_plan(
        "import launch",
        std::str::from_utf8(&import_plan.stdout).unwrap(),
    )
    .unwrap();
    let import_launch_path = root.join("import-launch.json");
    fs::write(&import_launch_path, import_plan.stdout).unwrap();
    let native = root.join("native=report.json");
    fs::write(&native, "{\"native\":true}\n").unwrap();
    let identity = adapter::identify_input("native-report", &native).unwrap();
    let imported = adapter_bundle(
        &import_launch,
        &fixture.adapter,
        vec![ImportInputIdentity {
            id: identity.id,
            digest: identity.digest,
            size_bytes: identity.size_bytes,
        }],
        false,
    );
    fixture.respond(response_template(
        "import",
        &fixture.adapter,
        Some(&import_launch.fingerprint),
        Some(&imported),
    ));
    let imported_output = azimuth(&[
        "run",
        "import",
        "--plan",
        import_launch_path.to_str().unwrap(),
        "--input",
        &format!("native-report={}", native.display()),
        "--config",
        fixture.config.to_str().unwrap(),
    ]);
    assert!(
        imported_output.status.success(),
        "{}",
        String::from_utf8_lossy(&imported_output.stderr)
    );
    assert_eq!(
        parse(
            "imported bundle",
            std::str::from_utf8(&imported_output.stdout).unwrap()
        )
        .unwrap(),
        imported
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn challenge_execute_and_import_accept_terminal_and_scheduled_incomplete_facts() {
    let root = root();
    let fixture = ProtocolFixture::new(&root);
    let (model, standards, workspace, manifest) = challenge_model(&root);
    let request_path = root.join("challenge-request.json");
    challenge_request(
        &request_path,
        RunOperation::Execute,
        false,
        "linux",
        "synthetic/challenges",
        1,
    );
    let planned = azimuth(&challenge_planning_arguments(
        &request_path,
        &model,
        &standards,
        &workspace,
        &manifest,
        &fixture.config,
    ));
    assert!(
        planned.status.success(),
        "{}",
        String::from_utf8_lossy(&planned.stderr)
    );
    let launch = run_plan::parse_launch_plan(
        "challenge execute launch",
        std::str::from_utf8(&planned.stdout).unwrap(),
    )
    .unwrap();
    let launch_path = root.join("challenge-execute.json");
    fs::write(&launch_path, &planned.stdout).unwrap();
    for (index, outcome) in [
        ChallengeOutcome::Clean,
        ChallengeOutcome::Findings,
        ChallengeOutcome::Inconclusive,
    ]
    .into_iter()
    .enumerate()
    {
        let bundle = challenge_adapter_bundle(&launch, &fixture.adapter, Vec::new(), Some(outcome));
        fixture.respond(response_template(
            "execute",
            &fixture.adapter,
            Some(&launch.fingerprint),
            Some(&bundle),
        ));
        let output = azimuth(&[
            "run",
            "execute",
            "--plan",
            launch_path.to_str().unwrap(),
            "--config",
            fixture.config.to_str().unwrap(),
        ]);
        assert!(
            output.status.success(),
            "{index}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            parse(
                "challenge execute",
                std::str::from_utf8(&output.stdout).unwrap()
            )
            .unwrap(),
            bundle
        );
        if index == 0 {
            let output_path = root.join("challenge-execute-bundle.json");
            let written = azimuth(&[
                "run",
                "execute",
                "--plan",
                launch_path.to_str().unwrap(),
                "--config",
                fixture.config.to_str().unwrap(),
                "--out",
                output_path.to_str().unwrap(),
            ]);
            assert!(written.status.success());
            assert!(written.stdout.is_empty());
            assert_eq!(output.stdout, fs::read(output_path).unwrap());
        }
        let bundle_path = root.join(format!("challenge-{index}.json"));
        fs::write(&bundle_path, &output.stdout).unwrap();
        assert!(
            azimuth(&["run", "verify", "--bundle", bundle_path.to_str().unwrap()])
                .status
                .success()
        );
        assert!(
            azimuth(&["run", "inspect", "--bundle", bundle_path.to_str().unwrap()])
                .status
                .success()
        );
    }

    let schedule = fs::read_to_string(&standards)
        .unwrap()
        .replace("Gate challenge: mutation", "Scheduled challenge: mutation");
    fs::write(&standards, schedule).unwrap();
    challenge_request(
        &request_path,
        RunOperation::Import,
        false,
        "linux",
        "synthetic/challenges",
        1,
    );
    let planned = azimuth(&challenge_planning_arguments(
        &request_path,
        &model,
        &standards,
        &workspace,
        &manifest,
        &fixture.config,
    ));
    assert!(planned.status.success());
    let launch = run_plan::parse_launch_plan(
        "challenge import launch",
        std::str::from_utf8(&planned.stdout).unwrap(),
    )
    .unwrap();
    assert_eq!(launch.plan.challenges[0].lane, ChallengeLane::Scheduled);
    let launch_path = root.join("challenge-import.json");
    fs::write(&launch_path, &planned.stdout).unwrap();
    let native = root.join("native-challenge.json");
    fs::write(&native, "{\"native\":true}\n").unwrap();
    let identity = adapter::identify_input("native-challenge", &native).unwrap();
    let bundle = challenge_adapter_bundle(
        &launch,
        &fixture.adapter,
        vec![ImportInputIdentity {
            id: identity.id,
            digest: identity.digest,
            size_bytes: identity.size_bytes,
        }],
        None,
    );
    fixture.respond(response_template(
        "import",
        &fixture.adapter,
        Some(&launch.fingerprint),
        Some(&bundle),
    ));
    let output = azimuth(&[
        "run",
        "import",
        "--plan",
        launch_path.to_str().unwrap(),
        "--input",
        &format!("native-challenge={}", native.display()),
        "--config",
        fixture.config.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        parse(
            "challenge import",
            std::str::from_utf8(&output.stdout).unwrap()
        )
        .unwrap(),
        bundle
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invoke_exit_classes_preserve_sentinel_and_leave_no_temporary_output() {
    let root = root();
    let fixture = ProtocolFixture::new(&root);
    let (model, workspace, manifest) = model(&root, &["demo/selected"]);
    let request_path = root.join("request.json");
    request(&request_path, RunOperation::Execute, "demo/selected");
    let planned = azimuth(&planning_arguments(
        &request_path,
        &model,
        &workspace,
        &manifest,
        &fixture.config,
    ));
    assert!(planned.status.success());
    let launch =
        run_plan::parse_launch_plan("launch", std::str::from_utf8(&planned.stdout).unwrap())
            .unwrap();
    let launch_path = root.join("launch.json");
    fs::write(&launch_path, planned.stdout).unwrap();
    let output = root.join("bundle.json");
    fs::write(&output, "sentinel").unwrap();
    let invoke = || {
        azimuth(&[
            "run",
            "execute",
            "--plan",
            launch_path.to_str().unwrap(),
            "--config",
            fixture.config.to_str().unwrap(),
            "--out",
            output.to_str().unwrap(),
        ])
    };

    let mut substituted = adapter_bundle(&launch, &fixture.adapter, Vec::new(), false);
    substituted.subject_fingerprint = fp('9');
    fixture.respond(response_template(
        "execute",
        &fixture.adapter,
        Some(&launch.fingerprint),
        Some(&substituted),
    ));
    assert_eq!(invoke().status.code(), Some(1));
    assert_eq!(fs::read_to_string(&output).unwrap(), "sentinel");

    fixture.mode("nonzero");
    assert_eq!(invoke().status.code(), Some(1));
    assert_eq!(fs::read_to_string(&output).unwrap(), "sentinel");

    fixture.mode("malformed");
    assert_eq!(invoke().status.code(), Some(2));
    assert_eq!(fs::read_to_string(&output).unwrap(), "sentinel");
    assert!(!fs::read_dir(&root)
        .unwrap()
        .flatten()
        .any(|entry| entry.file_name().to_string_lossy().ends_with(".tmp")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn plan_loads_the_complete_model_and_has_exact_stdout_file_parity() {
    let root = root();
    let config = configuration(&root);
    let (model, workspace, manifest) = model(&root, &["demo/selected", "demo/unselected"]);
    let request_path = root.join("request.json");
    request(&request_path, RunOperation::Execute, "demo/selected");
    let arguments = planning_arguments(&request_path, &model, &workspace, &manifest, &config);
    let stdout = azimuth(&arguments);
    assert!(
        stdout.status.success(),
        "{}",
        String::from_utf8_lossy(&stdout.stderr)
    );
    let launch =
        run_plan::parse_launch_plan("stdout", std::str::from_utf8(&stdout.stdout).unwrap())
            .unwrap();
    assert_eq!(launch.plan.checks.len(), 1);
    assert_eq!(launch.plan.checks[0].id, "demo/selected");

    let output = root.join("launch.json");
    let mut file_arguments = arguments;
    file_arguments.extend(["--out", output.to_str().unwrap()]);
    let written = azimuth(&file_arguments);
    assert!(written.status.success());
    assert!(written.stdout.is_empty());
    assert_eq!(stdout.stdout, fs::read(&output).unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn challenge_only_and_mixed_plans_have_exact_stdout_file_parity() {
    let root = root();
    let config = configuration(&root);
    let (model, standards, workspace, manifest) = challenge_model(&root);
    for (index, checks) in [false, true].into_iter().enumerate() {
        let request_path = root.join(format!("challenge-request-{index}.json"));
        challenge_request(
            &request_path,
            RunOperation::Execute,
            checks,
            "linux",
            "synthetic/challenges",
            1,
        );
        let arguments = challenge_planning_arguments(
            &request_path,
            &model,
            &standards,
            &workspace,
            &manifest,
            &config,
        );
        let stdout = azimuth(&arguments);
        assert!(
            stdout.status.success(),
            "{}",
            String::from_utf8_lossy(&stdout.stderr)
        );
        let launch = run_plan::parse_launch_plan(
            "challenge launch",
            std::str::from_utf8(&stdout.stdout).unwrap(),
        )
        .unwrap();
        assert_eq!(launch.plan.checks.len(), usize::from(checks));
        assert_eq!(launch.plan.challenges.len(), 1);
        assert_eq!(launch.routes.len(), usize::from(checks) + 1);
        let output = root.join(format!("challenge-launch-{index}.json"));
        let mut file_arguments = arguments;
        file_arguments.extend(["--out", output.to_str().unwrap()]);
        let written = azimuth(&file_arguments);
        assert!(written.status.success());
        assert!(written.stdout.is_empty());
        assert_eq!(stdout.stdout, fs::read(output).unwrap());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn challenge_request_schema_is_strict_and_preserves_sentinel_outputs() {
    let root = root();
    let config = configuration(&root);
    let (model, standards, workspace, manifest) = challenge_model(&root);
    let request_path = root.join("request.json");
    challenge_request(
        &request_path,
        RunOperation::Execute,
        false,
        "linux",
        "synthetic/challenges",
        1,
    );
    let valid = azimuth::json::parse(&fs::read_to_string(&request_path).unwrap()).unwrap();
    for (name, mutate) in [("missing", 0_u8), ("unknown", 1_u8)] {
        let Json::Obj(mut fields) = valid.clone() else {
            unreachable!()
        };
        if mutate == 0 {
            fields.retain(|(key, _)| key != "challenges");
        } else {
            fields.push(("challengez".into(), Json::Arr(Vec::new())));
        }
        fs::write(&request_path, Json::Obj(fields).to_string_pretty()).unwrap();
        let output_path = root.join(format!("{name}.json"));
        fs::write(&output_path, "sentinel").unwrap();
        let mut arguments = challenge_planning_arguments(
            &request_path,
            &model,
            &standards,
            &workspace,
            &manifest,
            &config,
        );
        arguments.extend(["--out", output_path.to_str().unwrap()]);
        let output = azimuth(&arguments);
        assert_eq!(output.status.code(), Some(2), "{name}");
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read_to_string(output_path).unwrap(), "sentinel");
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn challenge_semantic_failures_exit_one_without_publishing() {
    enum Failure {
        Adverse,
        Context,
        Cap,
        Form,
        Capability,
        UnknownPlan,
    }
    for (index, failure) in [
        Failure::Adverse,
        Failure::Context,
        Failure::Cap,
        Failure::Form,
        Failure::Capability,
        Failure::UnknownPlan,
    ]
    .into_iter()
    .enumerate()
    {
        let root = root();
        let config = configuration(&root);
        let (model, standards, workspace, manifest) = challenge_model(&root);
        let request_path = root.join("request.json");
        let mut context = "linux";
        let mut capability = "synthetic/challenges";
        if matches!(failure, Failure::Adverse) {
            let path = model.join("demo/verification.md");
            let source = fs::read_to_string(&path)
                .unwrap()
                .replace("Verdict: qualified", "Verdict: rejected");
            fs::write(path, source).unwrap();
        }
        if matches!(failure, Failure::Context) {
            context = "windows";
        }
        if matches!(failure, Failure::Cap) {
            add_second_challenge_candidate(&model, &standards, &workspace, &manifest);
        }
        if matches!(failure, Failure::Form) {
            let path = model.join("demo/verification.md");
            let source = fs::read_to_string(&path)
                .unwrap()
                .replace("Form: mutation", "Form: destructive");
            fs::write(path, source).unwrap();
            let source = fs::read_to_string(&standards).unwrap().replace(
                "Gate challenge: mutation",
                "Gate challenge: mutation\nScheduled challenge: destructive",
            );
            fs::write(&standards, source).unwrap();
        }
        if matches!(failure, Failure::Capability) {
            capability = "synthetic/missing";
        }
        challenge_request(
            &request_path,
            RunOperation::Execute,
            false,
            context,
            capability,
            1,
        );
        if matches!(failure, Failure::UnknownPlan) {
            let source = fs::read_to_string(&request_path)
                .unwrap()
                .replace("demo/plan", "demo/missing");
            fs::write(&request_path, source).unwrap();
        }
        let output_path = root.join(format!("failure-{index}.json"));
        fs::write(&output_path, "sentinel").unwrap();
        let mut arguments = challenge_planning_arguments(
            &request_path,
            &model,
            &standards,
            &workspace,
            &manifest,
            &config,
        );
        arguments.extend(["--out", output_path.to_str().unwrap()]);
        let output = azimuth(&arguments);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{index}: {stderr}",);
        let expected = match failure {
            Failure::Adverse => "not runnable",
            Failure::Context => "context must equal",
            Failure::Cap => "exceeding max_candidates 1",
            Failure::Form => "Challenge form `destructive`",
            Failure::Capability => "unknown configured capability",
            Failure::UnknownPlan => "unknown Challenge Plan",
        };
        assert!(stderr.contains(expected), "{index}: {stderr}");
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read_to_string(&output_path).unwrap(), "sentinel");
        assert!(!fs::read_dir(&root)
            .unwrap()
            .flatten()
            .any(|entry| entry.file_name().to_string_lossy().ends_with(".tmp")));
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn plan_rejects_partial_selection_and_duplicate_singleton_options() {
    let root = root();
    let request = root.join("request.json");
    fs::write(&request, "{}").unwrap();
    for forbidden in ["--only", "--project", "--workset", "--local"] {
        let output = azimuth(&[
            "run",
            "plan",
            "--request",
            request.to_str().unwrap(),
            forbidden,
            "value",
        ]);
        assert_eq!(output.status.code(), Some(2), "{forbidden}");
        assert!(output.stdout.is_empty());
    }
    let duplicate = azimuth(&[
        "run",
        "plan",
        "--request",
        request.to_str().unwrap(),
        "--request",
        request.to_str().unwrap(),
    ]);
    assert_eq!(duplicate.status.code(), Some(2));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn planning_schema_and_resolution_failures_preserve_output_and_clean_temps() {
    let root = root();
    let config = configuration(&root);
    let (model, workspace, manifest) = model(&root, &["demo/selected"]);
    let request_path = root.join("request.json");
    let output = root.join("launch.json");
    fs::write(&output, b"sentinel").unwrap();

    fs::write(&request_path, "{").unwrap();
    let mut arguments = planning_arguments(&request_path, &model, &workspace, &manifest, &config);
    arguments.extend(["--out", output.to_str().unwrap()]);
    let schema = azimuth(&arguments);
    assert_eq!(schema.status.code(), Some(2));
    assert_eq!(fs::read(&output).unwrap(), b"sentinel");

    request(&request_path, RunOperation::Execute, "demo/missing");
    let resolution = azimuth(&arguments);
    assert_eq!(resolution.status.code(), Some(1));
    assert_eq!(fs::read(&output).unwrap(), b"sentinel");
    assert!(!fs::read_dir(&root)
        .unwrap()
        .flatten()
        .any(|entry| entry.file_name().to_string_lossy().ends_with(".tmp")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invoke_parsing_sorts_inputs_and_rejects_duplicates_and_operation_mismatch() {
    let root = root();
    let config = configuration(&root);
    let (model, workspace, manifest) = model(&root, &["demo/selected"]);
    let request_path = root.join("request.json");
    request(&request_path, RunOperation::Import, "demo/selected");
    let planned = azimuth(&planning_arguments(
        &request_path,
        &model,
        &workspace,
        &manifest,
        &config,
    ));
    assert!(planned.status.success());
    let launch = root.join("launch=import.json");
    fs::write(&launch, planned.stdout).unwrap();

    let mismatch = azimuth(&[
        "run",
        "execute",
        "--plan",
        launch.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
    ]);
    assert_eq!(mismatch.status.code(), Some(1));

    let native = root.join("native=report.json");
    fs::write(&native, "{}").unwrap();
    let duplicate = azimuth(&[
        "run",
        "import",
        "--plan",
        launch.to_str().unwrap(),
        "--input",
        &format!("native-report={}", native.display()),
        "--input",
        &format!("native-report={}", native.display()),
    ]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(String::from_utf8(duplicate.stderr)
        .unwrap()
        .contains("duplicate import input id"));

    let malformed_predecessor = root.join("predecessor.json");
    fs::write(&malformed_predecessor, "{").unwrap();
    let predecessor = azimuth(&[
        "run",
        "import",
        "--plan",
        launch.to_str().unwrap(),
        "--input",
        &format!("native-report={}", native.display()),
        "--predecessor",
        malformed_predecessor.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
    ]);
    assert_eq!(predecessor.status.code(), Some(2));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn every_output_path_rejects_lexical_equality_with_an_input() {
    let root = root();
    let request = root.join("request.json");
    fs::write(&request, "{}").unwrap();
    let plan = azimuth(&[
        "run",
        "plan",
        "--request",
        request.to_str().unwrap(),
        "--out",
        request.to_str().unwrap(),
    ]);
    assert_eq!(plan.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&request).unwrap(), "{}");

    let invoke = azimuth(&[
        "run",
        "execute",
        "--plan",
        request.to_str().unwrap(),
        "--out",
        request.to_str().unwrap(),
    ]);
    assert_eq!(invoke.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&request).unwrap(), "{}");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn output_aliases_symlinks_and_model_descendants_are_rejected() {
    use std::os::unix::fs::symlink;

    let root = root();
    let request = root.join("request.json");
    fs::write(&request, "{}").unwrap();
    let alias = root.join("request-alias.json");
    symlink(&request, &alias).unwrap();
    let symlink_output = azimuth(&[
        "run",
        "plan",
        "--request",
        request.to_str().unwrap(),
        "--out",
        alias.to_str().unwrap(),
    ]);
    assert_eq!(symlink_output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&request).unwrap(), "{}");

    let subdirectory = root.join("subdirectory");
    fs::create_dir(&subdirectory).unwrap();
    let normalized_alias = subdirectory.join("..").join("request.json");
    let normalized = azimuth(&[
        "run",
        "plan",
        "--request",
        request.to_str().unwrap(),
        "--out",
        normalized_alias.to_str().unwrap(),
    ]);
    assert_eq!(normalized.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&request).unwrap(), "{}");

    let model = root.join("model");
    fs::create_dir(&model).unwrap();
    let model_output = model.join("generated-launch.json");
    let descendant = azimuth(&[
        "run",
        "plan",
        "--request",
        request.to_str().unwrap(),
        "--model",
        model.to_str().unwrap(),
        "--out",
        model_output.to_str().unwrap(),
    ]);
    assert_eq!(descendant.status.code(), Some(2));
    assert!(!model_output.exists());
    fs::remove_dir_all(root).unwrap();
}
