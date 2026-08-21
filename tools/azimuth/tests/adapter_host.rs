use azimuth::{diag, fingerprint, json, model, run, validation};
#[path = "../src/adapter.rs"]
mod adapter;
#[path = "../src/adapter_host.rs"]
mod adapter_host;
#[path = "../src/run_plan.rs"]
mod run_plan;

use adapter::{
    AdapterConfiguration, AdapterContent, AdapterEnvironment, AdapterLimits, AdapterOperation,
    Capability, CapabilityClass, ConfiguredAdapter, ConfiguredFile, ConfiguredResource,
    InputIdentity,
};
use adapter_host::{HostErrorClass, ImportInput};
use json::Json;
use run::*;
use run_plan::{LaunchAdapter, LaunchPlan, RunOperation};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn fp(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

struct Fixture {
    root: PathBuf,
    response: PathBuf,
    capture: PathBuf,
    mode: PathBuf,
    count: PathBuf,
    descendant_pid: PathBuf,
    external_target: PathBuf,
    configuration: AdapterConfiguration,
    launch: LaunchPlan,
    input: Option<ImportInput>,
    input_identities: Vec<InputIdentity>,
}

impl Fixture {
    fn new(operation: RunOperation, timeout_ms: u64, stdout_bytes: u64, stderr_bytes: u64) -> Self {
        let root = temporary_directory();
        let response = root.join("response.json");
        let capture = root.join("request.json");
        let mode = root.join("mode");
        let count = root.join("count");
        let descendant_pid = root.join("descendant-pid");
        let external_target = root.join("external-target");
        let executable = root.join("adapter");
        let resource = root.join("rules.json");
        fs::write(&resource, b"{\"rule\":\"strict\"}\n").unwrap();
        fs::write(&mode, b"response\n").unwrap();
        fs::write(&response, b"{}\n").unwrap();
        fs::write(&external_target, b"outside-stage\n").unwrap();
        let script = format!(
            concat!(
                "#!/bin/sh\n",
                "/bin/cat > '{}'\n",
                "if [ \"${{ONLY_LITERAL-}}\" != \"yes\" ] || [ \"${{HOME+x}}\" = \"x\" ]; then exit 31; fi\n",
                "mode=$(/bin/cat '{}')\n",
                "printf x >> '{}'\n",
                "case \"$mode\" in\n",
                "response) /bin/cat '{}' ;;\n",
                "nonzero) exit 7 ;;\n",
                "timeout) /bin/sleep 2 ;;\n",
                "stdout-overflow) while :; do printf 1234567890; done ;;\n",
                "stderr-overflow) while :; do printf 1234567890 >&2; done ;;\n",
                "both-overflow) while :; do printf 1234567890; printf 1234567890 >&2; done ;;\n",
                "malformed) printf '{{' ;;\n",
                "extra) /bin/cat '{}'; printf '{{}}' ;;\n",
                "failed) /bin/cat '{}' ;;\n",
                "descendant) ( /bin/sleep 20 ) & child=$!; printf '%s' \"$child\" > '{}'; /bin/cat '{}' ;;\n",
                "stderr-exact) printf 1234567812345678123456781234567812345678123456781234567812345678 >&2; /bin/cat '{}' ;;\n",
                "hostile-cleanup) /bin/mkdir nested; printf locked > nested/file; /bin/ln -s '{}' outside-link; /bin/chmod 000 nested; /bin/chmod 000 .; /bin/cat '{}' ;;\n",
                "esac\n"
            ),
            capture.display(),
            mode.display(),
            count.display(),
            response.display(),
            response.display(),
            response.display(),
            descendant_pid.display(),
            response.display(),
            response.display(),
            external_target.display(),
            response.display(),
        );
        fs::write(&executable, script).unwrap();

        let executable_identity = adapter::identify_input("adapter", &executable).unwrap();
        let resource_identity = adapter::identify_input("rules", &resource).unwrap();
        let mut configured = ConfiguredAdapter {
            id: "synthetic".into(),
            provider_family: "synthetic/test".into(),
            protocol_version: 1,
            adapter_version: "alpha.2-test".into(),
            build: "host-fixture".into(),
            content: AdapterContent {
                executable: ConfiguredFile {
                    locator: executable.display().to_string(),
                    resolved: executable,
                    digest: executable_identity.digest,
                },
                resources: vec![ConfiguredResource {
                    id: "rules".into(),
                    locator: resource.display().to_string(),
                    resolved: resource,
                    digest: resource_identity.digest,
                }],
            },
            semantic_settings: map(&[("dialect", "test")]),
            environment: AdapterEnvironment {
                literals: map(&[("ONLY_LITERAL", "yes")]),
            },
            limits: AdapterLimits {
                timeout_ms,
                stdout_bytes,
                stderr_bytes,
            },
            capabilities: vec![
                Capability {
                    id: "checks".into(),
                    classes: vec![match operation {
                        RunOperation::Execute => CapabilityClass::CheckExecute,
                        RunOperation::Import => CapabilityClass::CheckImport,
                    }],
                    challenge_forms: Vec::new(),
                    semantic_settings: map(&[("mode", "strict")]),
                    fingerprint: String::new(),
                },
                Capability {
                    id: "extractor".into(),
                    classes: vec![CapabilityClass::ModelExtract],
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

        let subject = Subject::Artifact {
            artifacts: vec![ArtifactState {
                id: "candidate".into(),
                digest: fp('a'),
            }],
        };
        let subject_fingerprint = run::subject_fingerprint(&subject);
        let check = CheckSelection {
            id: "demo/check".into(),
            fingerprint: fp('b'),
            implementations: vec![Implementation {
                identity: "demo|rust-symbol|demo::check".into(),
                source_fingerprint: fp('c'),
            }],
            units: vec![WorkUnit {
                id: "whole".into(),
                parameters: BTreeMap::new(),
            }],
        };
        let plan = run::construct_plan(
            &subject_fingerprint,
            fp('d'),
            map(&[("platform", "test")]),
            vec![check],
            Vec::new(),
        )
        .unwrap();
        let class = match operation {
            RunOperation::Execute => RouteCapabilityClass::CheckExecute,
            RunOperation::Import => RouteCapabilityClass::CheckImport,
        };
        let route = LaunchRoute {
            selection: RouteSelection {
                kind: RouteSelectionKind::Check,
                id: "demo/check".into(),
            },
            capability: RouteCapability {
                address: "synthetic/checks".into(),
                class,
                challenge_form: None,
                fingerprint: configured.capabilities[0].fingerprint.clone(),
            },
        };
        let mut launch = LaunchPlan {
            operation,
            planned_at_ms: 10,
            subject,
            subject_fingerprint,
            plan,
            adapter: LaunchAdapter {
                id: configured.id.clone(),
                adapter_version: configured.adapter_version.clone(),
                adapter_fingerprint: configured.adapter_fingerprint.clone(),
                descriptor_fingerprint: configured.descriptor_fingerprint.clone(),
                configuration_fingerprint: configured.configuration_fingerprint.clone(),
            },
            routes: vec![route],
            fingerprint: fp('0'),
        };
        launch.fingerprint = run_plan::launch_fingerprint(&launch);

        let input = if operation == RunOperation::Import {
            let path = root.join("native-report.json");
            fs::write(&path, b"{\"native\":true}\n").unwrap();
            Some(ImportInput {
                id: "native-report".into(),
                path,
            })
        } else {
            None
        };
        let input_identities = input
            .as_ref()
            .map(|input| vec![adapter::identify_input(&input.id, &input.path).unwrap()])
            .unwrap_or_default();
        let configuration = AdapterConfiguration {
            path: root.join("adapters.json"),
            directory: root.clone(),
            adapters: vec![configured],
        };
        Self {
            root,
            response,
            capture,
            mode,
            count,
            descendant_pid,
            external_target,
            configuration,
            launch,
            input,
            input_identities,
        }
    }

    fn adapter(&self) -> &ConfiguredAdapter {
        &self.configuration.adapters[0]
    }

    fn set_mode(&self, mode: &str) {
        fs::write(&self.mode, format!("{mode}\n")).unwrap();
    }

    fn set_description_response(&self) {
        let request_id = adapter::describe_request_fingerprint(
            &self.adapter().id,
            &self.adapter().configuration_fingerprint,
        )
        .unwrap();
        self.write_response(response_json(
            &request_id,
            "describe",
            self.adapter(),
            None,
            None,
            false,
        ));
    }

    fn set_bundle_response(&self, bundle: &RunBundle, predecessors: &[RunBundle]) {
        let identities = predecessors
            .iter()
            .map(|bundle| adapter::PredecessorIdentity {
                bundle_revision: bundle.bundle_revision,
                bundle_fingerprint: bundle.bundle_fingerprint.clone(),
            })
            .collect::<Vec<_>>();
        let request_id = adapter::run_request_fingerprint(
            self.launch.operation.adapter_operation(),
            &self.launch.fingerprint,
            &self.current_input_identities(),
            &identities,
        )
        .unwrap();
        self.write_response(response_json(
            &request_id,
            self.launch.operation.name(),
            self.adapter(),
            Some(&self.launch.fingerprint),
            Some(bundle),
            false,
        ));
    }

    fn current_input_identities(&self) -> Vec<InputIdentity> {
        self.input
            .as_ref()
            .map(|input| vec![adapter::identify_input(&input.id, &input.path).unwrap()])
            .unwrap_or_default()
    }

    fn set_failed_response(&self) {
        let request_id = adapter::run_request_fingerprint(
            self.launch.operation.adapter_operation(),
            &self.launch.fingerprint,
            &self.input_identities,
            &[],
        )
        .unwrap();
        self.write_response(response_json(
            &request_id,
            self.launch.operation.name(),
            self.adapter(),
            Some(&self.launch.fingerprint),
            None,
            true,
        ));
    }

    fn write_response(&self, value: Json) {
        fs::write(&self.response, run::canonical_json(&value).unwrap()).unwrap();
    }

    fn bundle(&self) -> RunBundle {
        let import_inputs = self
            .input_identities
            .iter()
            .map(|input| ImportInputIdentity {
                id: input.id.clone(),
                digest: input.digest.clone(),
                size_bytes: input.size_bytes,
            })
            .collect();
        let mode = match self.launch.operation {
            RunOperation::Execute => ProvenanceMode::Execute,
            RunOperation::Import => ProvenanceMode::Import,
        };
        let mut bundle = RunBundle {
            run_id: fp('0'),
            bundle_revision: 0,
            corrects: None,
            correction_reason: None,
            bundle_fingerprint: fp('0'),
            subject: self.launch.subject.clone(),
            subject_fingerprint: self.launch.subject_fingerprint.clone(),
            planned_at_ms: self.launch.planned_at_ms,
            started_at_ms: 11,
            finished_at_ms: 12,
            status: RunStatus::Partial,
            plan: self.launch.plan.clone(),
            actual_selection: ActualSelection {
                context: self.launch.plan.required_context.clone(),
                plan_fingerprint: self.launch.plan.fingerprint.clone(),
                checks: Vec::new(),
                challenges: Vec::new(),
                fingerprint: fp('0'),
            },
            provenance: Provenance {
                mode,
                source: SourceProvenance {
                    system: "synthetic/host".into(),
                    execution: "native-1".into(),
                    uri: None,
                },
                normalizer: Normalizer {
                    id: format!("adapter/{}", self.adapter().id),
                    version: self.adapter().adapter_version.clone(),
                    build_fingerprint: self.adapter().adapter_fingerprint.clone(),
                },
                adapter: AdapterProvenance {
                    id: self.adapter().id.clone(),
                    adapter_version: self.adapter().adapter_version.clone(),
                    adapter_fingerprint: self.adapter().adapter_fingerprint.clone(),
                    descriptor_fingerprint: self.adapter().descriptor_fingerprint.clone(),
                    configuration_fingerprint: self.adapter().configuration_fingerprint.clone(),
                    launch_fingerprint: self.launch.fingerprint.clone(),
                    routes: self.launch.routes.clone(),
                    import_inputs,
                },
                generated_at_ms: 13,
                principal: None,
                attributes: None,
            },
            artifacts: Vec::new(),
            diagnostics: Vec::new(),
            activities: Vec::new(),
            check_executions: Vec::new(),
            challenger_executions: Vec::new(),
        };
        refresh(&mut bundle);
        assert!(
            run::verify(&bundle).is_empty(),
            "{:?}",
            run::verify(&bundle)
        );
        bundle
    }

    fn violated_bundle(&self) -> RunBundle {
        let mut bundle = self.bundle();
        bundle.status = RunStatus::Complete;
        bundle.actual_selection.checks = bundle.plan.checks.clone();
        bundle.activities = vec![Activity {
            id: "native-check".into(),
            status: ActivityStatus::Completed,
            started_at_ms: 11,
            finished_at_ms: 12,
            artifacts: Vec::new(),
            diagnostics: Vec::new(),
            attributes: BTreeMap::new(),
        }];
        bundle.check_executions = vec![CheckExecution {
            check: CheckRef {
                id: bundle.plan.checks[0].id.clone(),
                fingerprint: bundle.plan.checks[0].fingerprint.clone(),
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
                observed_at_ms: 12,
                fingerprint: fp('0'),
                artifacts: Vec::new(),
                diagnostics: Vec::new(),
            },
        }];
        refresh(&mut bundle);
        assert!(
            run::verify(&bundle).is_empty(),
            "{:?}",
            run::verify(&bundle)
        );
        bundle
    }

    fn invoke(
        &self,
        predecessors: &[RunBundle],
    ) -> Result<adapter_host::HostedBundle, adapter_host::HostError> {
        match &self.input {
            None => adapter_host::execute(&self.configuration, &self.launch, predecessors),
            Some(input) => adapter_host::import(
                &self.configuration,
                &self.launch,
                std::slice::from_ref(input),
                predecessors,
            ),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn temporary_directory() -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "azimuth-adapter-host-test-{}-{epoch}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn map(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn refresh(bundle: &mut RunBundle) {
    bundle.subject_fingerprint = run::subject_fingerprint(&bundle.subject);
    bundle.plan.fingerprint = run::plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = run::selection_fingerprint(&bundle.actual_selection);
    bundle.run_id = run::run_id(bundle);
    for index in 0..bundle.check_executions.len() {
        let fingerprint = run::observation_fingerprint(bundle, &bundle.check_executions[index]);
        bundle.check_executions[index].observation.fingerprint = fingerprint;
    }
    for index in 0..bundle.challenger_executions.len() {
        let fingerprint =
            run::challenge_result_fingerprint(bundle, &bundle.challenger_executions[index]);
        bundle.challenger_executions[index].result.fingerprint = fingerprint;
    }
    bundle.bundle_fingerprint = run::bundle_fingerprint(bundle);
}

fn description_json(adapter: &ConfiguredAdapter) -> Json {
    let description = adapter.expected_description();
    Json::obj(vec![
        ("format", Json::str("azimuth-adapter-description")),
        ("version", Json::Num(1.0)),
        ("protocol_version", Json::Num(1.0)),
        ("id", Json::str(description.id)),
        ("provider_family", Json::str(description.provider_family)),
        ("adapter_version", Json::str(description.adapter_version)),
        ("build", Json::str(description.build)),
        (
            "content",
            Json::obj(vec![
                (
                    "executable_digest",
                    Json::str(description.content.executable_digest),
                ),
                (
                    "resources",
                    Json::Arr(
                        description
                            .content
                            .resources
                            .iter()
                            .map(|resource| {
                                Json::obj(vec![
                                    ("id", Json::str(&resource.id)),
                                    ("digest", Json::str(&resource.digest)),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        (
            "adapter_fingerprint",
            Json::str(description.adapter_fingerprint),
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
                            (
                                "semantic_settings",
                                Json::Obj(
                                    capability
                                        .semantic_settings
                                        .iter()
                                        .map(|(key, value)| (key.clone(), Json::str(value)))
                                        .collect(),
                                ),
                            ),
                            ("fingerprint", Json::str(&capability.fingerprint)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "descriptor_fingerprint",
            Json::str(description.descriptor_fingerprint),
        ),
    ])
}

fn response_json(
    request_id: &str,
    operation: &str,
    adapter: &ConfiguredAdapter,
    launch_fingerprint: Option<&str>,
    bundle: Option<&RunBundle>,
    failed: bool,
) -> Json {
    let mut fields = vec![
        ("format".into(), Json::str("azimuth-adapter-response")),
        ("version".into(), Json::Num(1.0)),
        ("request_id".into(), Json::str(request_id)),
        ("operation".into(), Json::str(operation)),
        (
            "status".into(),
            Json::str(if failed { "failed" } else { "ok" }),
        ),
        ("description".into(), description_json(adapter)),
    ];
    if let Some(fingerprint) = launch_fingerprint {
        fields.push(("launch_fingerprint".into(), Json::str(fingerprint)));
    }
    if let Some(bundle) = bundle {
        fields.push(("bundle".into(), run::to_json(bundle)));
    }
    if failed {
        fields.push((
            "failure".into(),
            Json::obj(vec![
                ("code", Json::str("native/failure")),
                ("message", Json::str("provider rejected the request")),
                ("details", Json::Obj(Vec::new())),
            ]),
        ));
    }
    Json::Obj(fields)
}

fn assert_no_invocation_stage_leaked() {
    let prefix = format!("azimuth-adapter-invocation-{}-", std::process::id());
    let leaked = fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
        .collect::<Vec<_>>();
    assert!(leaked.is_empty(), "leaked invocation stages: {leaked:?}");
}

#[cfg(unix)]
fn assert_process_disappears(path: &PathBuf) {
    let pid = fs::read_to_string(path).unwrap().parse::<i32>().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while unix_process_exists(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !unix_process_exists(pid),
        "adapter descendant {pid} survived"
    );
}

#[cfg(unix)]
fn unix_process_exists(pid: i32) -> bool {
    unsafe { test_kill(pid, 0) == 0 }
}

#[cfg(unix)]
unsafe extern "C" {
    #[link_name = "kill"]
    fn test_kill(pid: i32, signal: i32) -> i32;
}

#[test]
fn description_and_partial_execute_succeed_with_exact_staged_request() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    fixture.set_description_response();
    let description = adapter_host::verify_adapter(fixture.adapter()).unwrap();
    assert_eq!(description, fixture.adapter().expected_description());
    let describe_request = run::strict_json(
        "describe request",
        &fs::read_to_string(&fixture.capture).unwrap(),
    )
    .unwrap();
    assert_eq!(
        describe_request
            .get("configuration")
            .unwrap()
            .get("capabilities")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_no_invocation_stage_leaked();

    let bundle = fixture.bundle();
    fixture.set_bundle_response(&bundle, &[]);
    let hosted = fixture.invoke(&[]).unwrap();
    assert_eq!(hosted.bundle, bundle);
    assert_eq!(
        hosted.canonical_json,
        run::canonical_json(&run::to_json(&bundle)).unwrap()
    );
    let request = fs::read_to_string(&fixture.capture).unwrap();
    let parsed_request = run::strict_json("execute request", &request).unwrap();
    let selected = parsed_request
        .get("configuration")
        .unwrap()
        .get("capabilities")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].get("address").unwrap().as_str(),
        Some("synthetic/checks")
    );
    assert!(request.contains("adapter-resource-0000"));
    assert!(request.contains("\"terminal_predecessor\":null"));
    assert!(!request.contains(&fixture.adapter().content.resources[0].locator));
    assert_eq!(fs::read_to_string(&fixture.count).unwrap(), "xx");
    assert_no_invocation_stage_leaked();
}

#[test]
fn violated_cancelled_and_timed_out_runs_remain_successful_exchanges() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    let violated = fixture.violated_bundle();
    fixture.set_bundle_response(&violated, &[]);
    assert_eq!(
        fixture.invoke(&[]).unwrap().bundle.check_executions[0]
            .observation
            .outcome,
        ObservationOutcome::Violated
    );

    for status in [RunStatus::Cancelled, RunStatus::TimedOut] {
        let mut bundle = fixture.bundle();
        bundle.status = status;
        refresh(&mut bundle);
        fixture.set_bundle_response(&bundle, &[]);
        assert_eq!(fixture.invoke(&[]).unwrap().bundle.status, status);
    }
    assert_no_invocation_stage_leaked();
}

#[test]
fn import_stages_and_binds_the_exact_input_bytes() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Import, 1_000, 1_000_000, 1_000_000);
    let bundle = fixture.bundle();
    fixture.set_bundle_response(&bundle, &[]);
    let hosted = fixture.invoke(&[]).unwrap();
    assert_eq!(hosted.bundle.provenance.adapter.import_inputs.len(), 1);
    let request = fs::read_to_string(&fixture.capture).unwrap();
    assert!(request.contains("adapter-input-0000"));
    assert!(request.contains(&fixture.input_identities[0].digest));
    assert!(!request.contains(&fixture.input.as_ref().unwrap().path.display().to_string()));

    fs::write(
        &fixture.input.as_ref().unwrap().path,
        b"{\"native\":\"changed\"}\n",
    )
    .unwrap();
    assert_eq!(
        fixture.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn import_correction_accepts_changed_bytes_while_preserving_fixed_anchors() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Import, 1_000, 1_000_000, 1_000_000);
    let revision_zero = fixture.bundle();
    fixture.set_bundle_response(&revision_zero, &[]);
    fixture.invoke(&[]).unwrap();

    fs::write(
        &fixture.input.as_ref().unwrap().path,
        b"{\"native\":\"completed\"}\n",
    )
    .unwrap();
    let changed = fixture.current_input_identities();
    let mut revision_one = revision_zero.clone();
    revision_one.bundle_revision = 1;
    revision_one.corrects = Some(revision_zero.bundle_fingerprint.clone());
    revision_one.correction_reason = Some("the native report completed".into());
    revision_one.provenance.adapter.import_inputs = changed
        .iter()
        .map(|input| ImportInputIdentity {
            id: input.id.clone(),
            digest: input.digest.clone(),
            size_bytes: input.size_bytes,
        })
        .collect();
    refresh(&mut revision_one);
    fixture.set_bundle_response(&revision_one, std::slice::from_ref(&revision_zero));
    let hosted = fixture
        .invoke(std::slice::from_ref(&revision_zero))
        .unwrap();
    assert_ne!(
        hosted.bundle.provenance.adapter.import_inputs,
        revision_zero.provenance.adapter.import_inputs
    );
    assert_eq!(hosted.bundle.started_at_ms, revision_zero.started_at_ms);
    assert_eq!(
        hosted.bundle.provenance.source,
        revision_zero.provenance.source
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn transport_failures_are_class_one_bounded_and_never_retried() {
    let _guard = test_lock();
    for mode in [
        "nonzero",
        "timeout",
        "stdout-overflow",
        "stderr-overflow",
        "both-overflow",
    ] {
        let fixture = Fixture::new(RunOperation::Execute, 1_000, 64, 64);
        fixture.set_mode(mode);
        let started = Instant::now();
        let error = fixture.invoke(&[]).unwrap_err();
        assert_eq!(error.class, HostErrorClass::Semantic, "mode {mode}");
        assert!(
            started.elapsed() < Duration::from_millis(1_800),
            "mode {mode}"
        );
        assert_eq!(
            fs::read_to_string(&fixture.count).unwrap_or_default(),
            "x",
            "mode {mode}"
        );
        assert_no_invocation_stage_leaked();
    }
}

#[test]
fn successful_exchange_kills_and_reaps_leftover_descendants() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    fixture.set_bundle_response(&fixture.bundle(), &[]);
    fixture.set_mode("descendant");
    let started = Instant::now();
    assert!(fixture.invoke(&[]).is_ok());
    assert!(started.elapsed() < Duration::from_millis(900));
    #[cfg(unix)]
    assert_process_disappears(&fixture.descendant_pid);
    assert_no_invocation_stage_leaked();
}

#[test]
fn exact_stream_cap_is_inclusive_and_one_extra_byte_overflows() {
    let _guard = test_lock();
    let exact = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 64);
    exact.set_bundle_response(&exact.bundle(), &[]);
    exact.set_mode("stderr-exact");
    assert!(exact.invoke(&[]).is_ok());

    let overflow = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 63);
    overflow.set_bundle_response(&overflow.bundle(), &[]);
    overflow.set_mode("stderr-exact");
    assert_eq!(
        overflow.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn cleanup_recovers_hostile_permissions_without_following_symlinks() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    fixture.set_bundle_response(&fixture.bundle(), &[]);
    fixture.set_mode("hostile-cleanup");
    assert!(fixture.invoke(&[]).is_ok());
    assert_eq!(
        fs::read(&fixture.external_target).unwrap(),
        b"outside-stage\n"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_ne!(
            fs::metadata(&fixture.external_target)
                .unwrap()
                .permissions()
                .mode()
                & 0o700,
            0
        );
    }
    assert_no_invocation_stage_leaked();
}

#[test]
fn response_framing_preserves_schema_and_transport_exit_classes() {
    let _guard = test_lock();
    let malformed = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    malformed.set_mode("malformed");
    assert_eq!(
        malformed.invoke(&[]).unwrap_err().class,
        HostErrorClass::Schema
    );

    let extra = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    extra.set_bundle_response(&extra.bundle(), &[]);
    extra.set_mode("extra");
    assert_eq!(
        extra.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );

    let failed = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    failed.set_failed_response();
    failed.set_mode("failed");
    assert_eq!(
        failed.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn response_request_operation_and_launch_substitution_are_semantic_failures() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    let bundle = fixture.bundle();
    let request_id = adapter::run_request_fingerprint(
        AdapterOperation::Execute,
        &fixture.launch.fingerprint,
        &[],
        &[],
    )
    .unwrap();

    fixture.write_response(response_json(
        &fp('9'),
        "execute",
        fixture.adapter(),
        Some(&fixture.launch.fingerprint),
        Some(&bundle),
        false,
    ));
    assert_eq!(
        fixture.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );

    fixture.write_response(response_json(
        &request_id,
        "import",
        fixture.adapter(),
        Some(&fixture.launch.fingerprint),
        Some(&bundle),
        false,
    ));
    assert_eq!(
        fixture.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );

    fixture.write_response(response_json(
        &request_id,
        "execute",
        fixture.adapter(),
        Some(&fp('8')),
        Some(&bundle),
        false,
    ));
    assert_eq!(
        fixture.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn description_launch_plan_and_provenance_substitution_fail_closed() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    let mut response = fixture.bundle();
    response.provenance.normalizer.version = "substituted".into();
    refresh(&mut response);
    fixture.set_bundle_response(&response, &[]);
    assert_eq!(
        fixture.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );

    let mut response = fixture.bundle();
    response.plan.model_fingerprint = fp('e');
    refresh(&mut response);
    fixture.set_bundle_response(&response, &[]);
    assert_eq!(
        fixture.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );

    let mut launch = fixture.launch.clone();
    launch.routes[0].capability.challenge_form = Some("invalid/form".into());
    launch.fingerprint = run_plan::launch_fingerprint(&launch);
    let error = adapter_host::execute(&fixture.configuration, &launch, &[]).unwrap_err();
    assert_eq!(error.class, HostErrorClass::Semantic);

    let other = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    let wrong_description = description_json(other.adapter());
    let request_id = adapter::describe_request_fingerprint(
        &fixture.adapter().id,
        &fixture.adapter().configuration_fingerprint,
    )
    .unwrap();
    let mut response = response_json(
        &request_id,
        "describe",
        fixture.adapter(),
        None,
        None,
        false,
    );
    if let Json::Obj(fields) = &mut response {
        fields
            .iter_mut()
            .find(|(key, _)| key == "description")
            .unwrap()
            .1 = wrong_description;
    }
    fixture.write_response(response);
    assert_eq!(
        adapter_host::verify_adapter(fixture.adapter())
            .unwrap_err()
            .class,
        HostErrorClass::Semantic
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn staged_content_drift_and_spawn_failure_cleanup_without_invocation() {
    let _guard = test_lock();
    let drift = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    fs::write(&drift.adapter().content.executable.resolved, b"changed\n").unwrap();
    assert_eq!(
        drift.invoke(&[]).unwrap_err().class,
        HostErrorClass::Semantic
    );
    assert!(!drift.count.exists());
    assert_no_invocation_stage_leaked();

    let spawn = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    fs::write(
        &spawn.adapter().content.executable.resolved,
        b"not-an-executable\n",
    )
    .unwrap();
    let identity =
        adapter::identify_input("adapter", &spawn.adapter().content.executable.resolved).unwrap();
    let mut configuration = spawn.configuration.clone();
    configuration.adapters[0].content.executable.digest = identity.digest;
    configuration.adapters[0].adapter_fingerprint =
        adapter::adapter_fingerprint(&configuration.adapters[0]);
    let adapter_fingerprint = configuration.adapters[0].adapter_fingerprint.clone();
    for capability in &mut configuration.adapters[0].capabilities {
        capability.fingerprint = adapter::capability_fingerprint(&adapter_fingerprint, capability);
    }
    configuration.adapters[0].descriptor_fingerprint =
        adapter::descriptor_fingerprint(&configuration.adapters[0].expected_description());
    configuration.adapters[0].configuration_fingerprint =
        adapter::configuration_fingerprint(&configuration.adapters[0]);
    let mut launch = spawn.launch.clone();
    launch.adapter = LaunchAdapter {
        id: configuration.adapters[0].id.clone(),
        adapter_version: configuration.adapters[0].adapter_version.clone(),
        adapter_fingerprint: configuration.adapters[0].adapter_fingerprint.clone(),
        descriptor_fingerprint: configuration.adapters[0].descriptor_fingerprint.clone(),
        configuration_fingerprint: configuration.adapters[0].configuration_fingerprint.clone(),
    };
    launch.routes[0].capability.fingerprint = configuration.adapters[0].capabilities[0]
        .fingerprint
        .clone();
    launch.fingerprint = run_plan::launch_fingerprint(&launch);
    assert_eq!(
        adapter_host::execute(&configuration, &launch, &[])
            .unwrap_err()
            .class,
        HostErrorClass::Semantic
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn predecessor_chain_accepts_unordered_replay_and_requires_exact_successor() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    let revision_zero = fixture.bundle();
    let mut revision_one = revision_zero.clone();
    revision_one.bundle_revision = 1;
    revision_one.corrects = Some(revision_zero.bundle_fingerprint.clone());
    revision_one.correction_reason = Some("late native facts arrived".into());
    refresh(&mut revision_one);
    let mut revision_two = revision_one.clone();
    revision_two.bundle_revision = 2;
    revision_two.corrects = Some(revision_one.bundle_fingerprint.clone());
    revision_two.correction_reason = Some("normalization completed".into());
    refresh(&mut revision_two);

    fixture.set_bundle_response(
        &revision_two,
        &[revision_zero.clone(), revision_one.clone()],
    );
    let hosted = fixture
        .invoke(&[
            revision_one.clone(),
            revision_zero.clone(),
            revision_one.clone(),
        ])
        .unwrap();
    assert_eq!(hosted.bundle.bundle_revision, 2);
    let request = fs::read_to_string(&fixture.capture).unwrap();
    assert!(request.contains("terminal_predecessor"));
    assert!(request.contains(&revision_one.bundle_fingerprint));

    let count_before = fs::read_to_string(&fixture.count).unwrap();
    let mut tampered_terminal = revision_one.clone();
    tampered_terminal.started_at_ms = 99;
    assert_eq!(
        fixture
            .invoke(&[revision_zero.clone(), tampered_terminal])
            .unwrap_err()
            .class,
        HostErrorClass::Semantic
    );
    assert_eq!(fs::read_to_string(&fixture.count).unwrap(), count_before);

    fixture.set_bundle_response(
        &revision_one,
        &[revision_zero.clone(), revision_one.clone()],
    );
    assert_eq!(
        fixture
            .invoke(&[revision_zero.clone(), revision_one.clone()])
            .unwrap_err()
            .class,
        HostErrorClass::Semantic
    );

    let mut nonterminal = revision_two.clone();
    nonterminal.corrects = Some(revision_zero.bundle_fingerprint.clone());
    refresh(&mut nonterminal);
    fixture.set_bundle_response(&nonterminal, &[revision_zero.clone(), revision_one.clone()]);
    assert_eq!(
        fixture
            .invoke(&[revision_zero.clone(), revision_one.clone()])
            .unwrap_err()
            .class,
        HostErrorClass::Semantic
    );

    let mut anchor_drift = revision_two.clone();
    anchor_drift.started_at_ms += 1;
    refresh(&mut anchor_drift);
    fixture.set_bundle_response(
        &anchor_drift,
        &[revision_zero.clone(), revision_one.clone()],
    );
    assert_eq!(
        fixture
            .invoke(&[revision_zero, revision_one])
            .unwrap_err()
            .class,
        HostErrorClass::Semantic
    );
    assert_no_invocation_stage_leaked();
}

#[test]
fn predecessor_validation_rejects_two_runs_before_spawn() {
    let _guard = test_lock();
    let fixture = Fixture::new(RunOperation::Execute, 1_000, 1_000_000, 1_000_000);
    let first = fixture.bundle();
    let mut second = first.clone();
    second.provenance.source.execution = "native-2".into();
    refresh(&mut second);
    let error = fixture.invoke(&[first, second]).unwrap_err();
    assert_eq!(error.class, HostErrorClass::Semantic);
    assert!(!fixture.count.exists());
    assert_no_invocation_stage_leaked();
}
