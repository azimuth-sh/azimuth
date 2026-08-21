#[path = "../src/adapter.rs"]
mod adapter;

use adapter::{
    adapter_fingerprint, capability_fingerprint, configuration_fingerprint,
    describe_request_fingerprint, descriptor_fingerprint, parse_configuration, parse_description,
    run_request_fingerprint, stage_content, stage_file, validate_description, AdapterConfiguration,
    AdapterContent, AdapterEnvironment, AdapterLimits, AdapterOperation, Capability,
    CapabilityClass, ConfiguredAdapter, ConfiguredFile, ConfiguredResource, InputIdentity,
    PredecessorIdentity,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    config_path: PathBuf,
    adapter: ConfiguredAdapter,
}

impl Fixture {
    fn new() -> Self {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "azimuth-adapter-kernel-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let executable = directory.join("adapter.bin");
        let resource = directory.join("rules.json");
        fs::write(&executable, b"synthetic adapter\n").unwrap();
        fs::write(&resource, b"{\"dialect\":\"v1\"}\n").unwrap();
        let executable_digest = adapter::identify_input("executable", &executable)
            .unwrap()
            .digest;
        let resource_digest = adapter::identify_input("resource", &resource)
            .unwrap()
            .digest;

        let mut configured = ConfiguredAdapter {
            id: "demo".into(),
            provider_family: "synthetic/demo".into(),
            protocol_version: 1,
            adapter_version: "1".into(),
            build: "b1".into(),
            content: AdapterContent {
                executable: ConfiguredFile {
                    locator: "adapter.bin".into(),
                    resolved: executable.canonicalize().unwrap(),
                    digest: executable_digest,
                },
                resources: vec![ConfiguredResource {
                    id: "rules".into(),
                    locator: "rules.json".into(),
                    resolved: resource.canonicalize().unwrap(),
                    digest: resource_digest,
                }],
            },
            semantic_settings: BTreeMap::from([("dialect".into(), "v1".into())]),
            environment: AdapterEnvironment {
                literals: BTreeMap::from([("LANG".into(), "C".into())]),
            },
            limits: AdapterLimits {
                timeout_ms: 1000,
                stdout_bytes: 4096,
                stderr_bytes: 1024,
            },
            capabilities: vec![],
            adapter_fingerprint: String::new(),
            descriptor_fingerprint: String::new(),
            configuration_fingerprint: String::new(),
        };
        configured.adapter_fingerprint = adapter_fingerprint(&configured);
        configured.capabilities = vec![
            capability(
                &configured.adapter_fingerprint,
                "all",
                vec![
                    CapabilityClass::ChallengeExecute,
                    CapabilityClass::ChallengeImport,
                    CapabilityClass::CheckExecute,
                    CapabilityClass::CheckImport,
                    CapabilityClass::ModelExtract,
                ],
                vec!["implementation-perturbation".into()],
            ),
            capability(
                &configured.adapter_fingerprint,
                "checks",
                vec![CapabilityClass::CheckExecute],
                vec![],
            ),
        ];
        configured.descriptor_fingerprint =
            descriptor_fingerprint(&configured.expected_description());
        configured.configuration_fingerprint = configuration_fingerprint(&configured);
        let config_path = directory.join("adapters.json");
        Self {
            directory,
            config_path,
            adapter: configured,
        }
    }

    fn configuration_json(&self) -> String {
        configuration_json(&self.adapter)
    }

    fn parse(&self) -> AdapterConfiguration {
        fs::write(&self.config_path, self.configuration_json()).unwrap();
        adapter::load_configuration(&self.config_path).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn capability(
    adapter_fingerprint_value: &str,
    id: &str,
    classes: Vec<CapabilityClass>,
    challenge_forms: Vec<String>,
) -> Capability {
    let mut capability = Capability {
        id: id.into(),
        classes,
        challenge_forms,
        semantic_settings: BTreeMap::from([("mode".into(), "strict".into())]),
        fingerprint: String::new(),
    };
    capability.fingerprint = capability_fingerprint(adapter_fingerprint_value, &capability);
    capability
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character => out.push(character),
        }
    }
    out.push('"');
    out
}

fn string_map_json(map: &BTreeMap<String, String>) -> String {
    let fields = map
        .iter()
        .map(|(key, value)| format!("{}:{}", json_string(key), json_string(value)))
        .collect::<Vec<_>>();
    format!("{{{}}}", fields.join(","))
}

fn capability_json(capability: &Capability) -> String {
    let classes = capability
        .classes
        .iter()
        .map(|class| json_string(class.name()))
        .collect::<Vec<_>>()
        .join(",");
    let forms = capability
        .challenge_forms
        .iter()
        .map(|form| json_string(form))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\"id\":{},\"classes\":[{}],\"challenge_forms\":[{}],",
            "\"semantic_settings\":{},\"fingerprint\":{}}}"
        ),
        json_string(&capability.id),
        classes,
        forms,
        string_map_json(&capability.semantic_settings),
        json_string(&capability.fingerprint)
    )
}

fn configuration_json(adapter: &ConfiguredAdapter) -> String {
    let resources = adapter
        .content
        .resources
        .iter()
        .map(|resource| {
            format!(
                "{{\"id\":{},\"locator\":{},\"digest\":{}}}",
                json_string(&resource.id),
                json_string(&resource.locator),
                json_string(&resource.digest)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let capabilities = adapter
        .capabilities
        .iter()
        .map(capability_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\"format\":\"azimuth-adapter-configuration\",\"version\":1,\"adapters\":[{{",
            "\"id\":{},\"provider_family\":{},\"protocol_version\":1,",
            "\"adapter_version\":{},\"build\":{},",
            "\"content\":{{\"executable\":{{\"locator\":{},\"digest\":{}}},",
            "\"resources\":[{}]}},",
            "\"semantic_settings\":{},",
            "\"environment\":{{\"literals\":{}}},",
            "\"limits\":{{\"timeout_ms\":{},\"stdout_bytes\":{},\"stderr_bytes\":{}}},",
            "\"capabilities\":[{}],\"adapter_fingerprint\":{},",
            "\"descriptor_fingerprint\":{},\"configuration_fingerprint\":{}",
            "}}]}}"
        ),
        json_string(&adapter.id),
        json_string(&adapter.provider_family),
        json_string(&adapter.adapter_version),
        json_string(&adapter.build),
        json_string(&adapter.content.executable.locator),
        json_string(&adapter.content.executable.digest),
        resources,
        string_map_json(&adapter.semantic_settings),
        string_map_json(&adapter.environment.literals),
        adapter.limits.timeout_ms,
        adapter.limits.stdout_bytes,
        adapter.limits.stderr_bytes,
        capabilities,
        json_string(&adapter.adapter_fingerprint),
        json_string(&adapter.descriptor_fingerprint),
        json_string(&adapter.configuration_fingerprint),
    )
}

fn description_json(adapter: &ConfiguredAdapter) -> String {
    let resources = adapter
        .content
        .resources
        .iter()
        .map(|resource| {
            format!(
                "{{\"id\":{},\"digest\":{}}}",
                json_string(&resource.id),
                json_string(&resource.digest)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let capabilities = adapter
        .capabilities
        .iter()
        .map(capability_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\"format\":\"azimuth-adapter-description\",\"version\":1,",
            "\"protocol_version\":1,\"id\":{},\"provider_family\":{},",
            "\"adapter_version\":{},\"build\":{},",
            "\"content\":{{\"executable_digest\":{},\"resources\":[{}]}},",
            "\"adapter_fingerprint\":{},\"capabilities\":[{}],",
            "\"descriptor_fingerprint\":{} }}"
        ),
        json_string(&adapter.id),
        json_string(&adapter.provider_family),
        json_string(&adapter.adapter_version),
        json_string(&adapter.build),
        json_string(&adapter.content.executable.digest),
        resources,
        json_string(&adapter.adapter_fingerprint),
        capabilities,
        json_string(&adapter.descriptor_fingerprint),
    )
}

fn parse_error(path: &Path, source: &str) -> String {
    parse_configuration(path, source).unwrap_err()[0]
        .detail
        .clone()
}

#[test]
fn matches_all_published_adapter_identity_vectors() {
    let zero = format!("sha256:{}", "0".repeat(64));
    let mut configured = ConfiguredAdapter {
        id: "demo".into(),
        provider_family: "synthetic/demo".into(),
        protocol_version: 1,
        adapter_version: "1".into(),
        build: "b1".into(),
        content: AdapterContent {
            executable: ConfiguredFile {
                locator: "/relocatable/adapter".into(),
                resolved: PathBuf::from("/relocatable/adapter"),
                digest: zero,
            },
            resources: vec![],
        },
        semantic_settings: BTreeMap::from([("dialect".into(), "v1".into())]),
        environment: AdapterEnvironment {
            literals: BTreeMap::from([("LANG".into(), "C".into())]),
        },
        limits: AdapterLimits {
            timeout_ms: 1000,
            stdout_bytes: 4096,
            stderr_bytes: 1024,
        },
        capabilities: vec![],
        adapter_fingerprint: String::new(),
        descriptor_fingerprint: String::new(),
        configuration_fingerprint: String::new(),
    };
    configured.adapter_fingerprint = adapter_fingerprint(&configured);
    assert_eq!(
        configured.adapter_fingerprint,
        "sha256:5274f56569ecfbe3cf6a1d8657ff431f78e99b0b97bf2365ea3d6714f950fa2a"
    );
    configured.capabilities = vec![capability(
        &configured.adapter_fingerprint,
        "check",
        vec![CapabilityClass::CheckExecute],
        vec![],
    )];
    assert_eq!(
        configured.capabilities[0].fingerprint,
        "sha256:41d224fdbb6fd9c43e067993ff30beb27eb5fc9793c32c9a7701d8678d3a397f"
    );
    configured.descriptor_fingerprint = descriptor_fingerprint(&configured.expected_description());
    assert_eq!(
        configured.descriptor_fingerprint,
        "sha256:f94a0c51a0050bbadfd0d0cb9b34fd6a696f4b7c06246c890b60310bbcb18670"
    );
    configured.configuration_fingerprint = configuration_fingerprint(&configured);
    assert_eq!(
        configured.configuration_fingerprint,
        "sha256:8b554d29e9bf8cdaee20699d1d10f64493acba3f2d1466c7523c078922c4f6e1"
    );
    assert_eq!(
        describe_request_fingerprint("demo", &format!("sha256:{}", "1".repeat(64)),),
        "sha256:4247bd475c6d87a35d495dc1b83f0125c2072d0453db1cd6353406603df18edf"
    );
    let launch = format!("sha256:{}", "9".repeat(64));
    assert_eq!(
        run_request_fingerprint(AdapterOperation::Execute, &launch, &[], &[]).unwrap(),
        "sha256:17730bd1fa89859bb3c4562bc305a9316e079e0daa11756f432afa374e9d19f4"
    );
    assert_eq!(
        run_request_fingerprint(
            AdapterOperation::Import,
            &launch,
            &[InputIdentity {
                id: "native-report".into(),
                digest: format!("sha256:{}", "a".repeat(64)),
                size_bytes: 12,
            }],
            &[PredecessorIdentity {
                bundle_revision: 0,
                bundle_fingerprint: format!("sha256:{}", "b".repeat(64)),
            }],
        )
        .unwrap(),
        "sha256:d2ca3469eff8a9bea75a43863fe23103a3b6c137b144b48b581772924a79427d"
    );
}

#[test]
fn parses_pinned_content_all_classes_addresses_and_cleared_environment() {
    let fixture = Fixture::new();
    let configuration = fixture.parse();
    let configured = configuration.adapter("demo").unwrap();
    let all = configured.capability("demo/all").unwrap();
    assert_eq!(all.address("demo"), "demo/all");
    assert_eq!(all.classes.len(), 5);
    assert!(CapabilityClass::ALL
        .into_iter()
        .all(|class| all.supports(class)));
    assert!(configuration.capability("demo/checks").is_some());
    assert!(configuration.capability("other/checks").is_none());
    assert!(configuration.capability("demo/all/extra").is_none());

    assert_eq!(
        configured.child_environment(),
        BTreeMap::from([("LANG".into(), "C".into())])
    );
    assert!(configured.content.executable.resolved.is_absolute());
    assert!(configured
        .content
        .resources
        .iter()
        .all(|resource| resource.resolved.is_absolute()));
}

#[test]
fn rejects_unknown_duplicate_unsafe_and_noncanonical_configuration() {
    let fixture = Fixture::new();
    let valid = fixture.configuration_json();
    let unknown = valid.replacen("\"version\":1", "\"version\":1,\"unexpected\":true", 1);
    assert!(parse_error(&fixture.config_path, &unknown).contains("unknown field"));

    let duplicate = valid.replacen(
        "\"format\":\"azimuth-adapter-configuration\"",
        concat!(
            "\"format\":\"azimuth-adapter-configuration\",",
            "\"format\":\"azimuth-adapter-configuration\""
        ),
        1,
    );
    assert!(parse_error(&fixture.config_path, &duplicate).contains("duplicate field"));

    let unsafe_number = valid.replacen("\"timeout_ms\":1000", "\"timeout_ms\":9007199254740992", 1);
    assert!(parse_error(&fixture.config_path, &unsafe_number).contains("safe-integer"));

    let mut reversed = fixture.adapter.clone();
    reversed.capabilities.reverse();
    let noncanonical = configuration_json(&reversed);
    assert!(parse_error(&fixture.config_path, &noncanonical).contains("sorted canonically"));

    let no_classes = valid.replacen(
        concat!(
            "\"classes\":[\"challenge.execute\",\"challenge.import\",",
            "\"check.execute\",\"check.import\",\"model.extract\"]"
        ),
        "\"classes\":[]",
        1,
    );
    assert!(parse_error(&fixture.config_path, &no_classes).contains("must not be empty"));

    let inherited = valid.replacen(
        "\"environment\":{\"literals\":{\"LANG\":\"C\"}}",
        "\"environment\":{\"literals\":{\"LANG\":\"C\"},\"inherit\":[\"TMPDIR\"]}",
        1,
    );
    assert!(parse_error(&fixture.config_path, &inherited).contains("unknown field `inherit`"));
}

#[test]
fn stages_the_exact_single_open_byte_stream_and_fails_closed_on_drift() {
    let fixture = Fixture::new();
    let stage = fixture.directory.join("stage");
    fs::create_dir(&stage).unwrap();
    let staged = stage_content(&fixture.adapter, &stage).unwrap();
    assert_eq!(
        fs::read(&staged.executable.path).unwrap(),
        fs::read(&fixture.adapter.content.executable.resolved).unwrap()
    );
    assert_eq!(staged.resources.len(), 1);
    assert_eq!(staged.resources[0].id, "rules");
    assert_eq!(
        fs::read(&staged.resources[0].path).unwrap(),
        fs::read(&fixture.adapter.content.resources[0].resolved).unwrap()
    );
    assert!(fs::metadata(&staged.resources[0].path)
        .unwrap()
        .permissions()
        .readonly());
    assert!(stage_content(&fixture.adapter, &stage)
        .unwrap_err()
        .contains("cannot be created"));

    fs::write(&fixture.adapter.content.executable.resolved, b"substituted").unwrap();
    let drift_stage = fixture.directory.join("drift-stage");
    fs::create_dir(&drift_stage).unwrap();
    assert!(stage_content(&fixture.adapter, &drift_stage)
        .unwrap_err()
        .contains("digest mismatch"));
    assert!(!drift_stage.join("adapter-executable").exists());

    let import_stage = fixture.directory.join("import-stage");
    fs::create_dir(&import_stage).unwrap();
    let imported = stage_file(
        &fixture.adapter.content.resources[0].resolved,
        &import_stage.join("native-report"),
        None,
        None,
        false,
    )
    .unwrap();
    assert_eq!(
        imported.size_bytes,
        fs::metadata(&fixture.adapter.content.resources[0].resolved)
            .unwrap()
            .len()
    );
    assert_eq!(imported.digest, fixture.adapter.content.resources[0].digest);
    assert_eq!(
        imported.input_identity("native-report").unwrap().size_bytes,
        imported.size_bytes
    );
}

#[test]
fn rejects_content_drift_relative_escape_and_symlink_escape() {
    let fixture = Fixture::new();
    let valid = fixture.configuration_json();
    fs::write(fixture.directory.join("rules.json"), b"changed\n").unwrap();
    let parsed = parse_configuration(&fixture.config_path, &valid).unwrap();
    let stage = fixture.directory.join("content-drift-stage");
    fs::create_dir(&stage).unwrap();
    assert!(stage_content(&parsed.adapters[0], &stage)
        .unwrap_err()
        .contains("digest mismatch"));
    assert!(!stage.join("adapter-executable").exists());

    let escaped = valid.replacen(
        "\"locator\":\"adapter.bin\"",
        "\"locator\":\"../adapter.bin\"",
        1,
    );
    assert!(parse_error(&fixture.config_path, &escaped).contains("normalized relative path"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = std::env::temp_dir().join(format!("adapter-outside-{}", std::process::id()));
        fs::write(&outside, b"outside").unwrap();
        let link = fixture.directory.join("link.bin");
        symlink(&outside, &link).unwrap();
        let digest = adapter::identify_input("outside", &outside).unwrap().digest;
        let linked = valid
            .replacen("\"locator\":\"adapter.bin\"", "\"locator\":\"link.bin\"", 1)
            .replacen(
                &format!(
                    "\"digest\":{}",
                    json_string(&fixture.adapter.content.executable.digest)
                ),
                &format!("\"digest\":{}", json_string(&digest)),
                1,
            );
        assert!(parse_error(&fixture.config_path, &linked).contains("escapes"));
        let _ = fs::remove_file(outside);
    }
}

#[test]
fn locator_relocation_does_not_change_semantic_fingerprints() {
    let fixture = Fixture::new();
    let relocated_directory = fixture.directory.join("relocated");
    fs::create_dir_all(&relocated_directory).unwrap();
    fs::copy(
        fixture.directory.join("adapter.bin"),
        relocated_directory.join("renamed.bin"),
    )
    .unwrap();
    fs::copy(
        fixture.directory.join("rules.json"),
        relocated_directory.join("renamed.json"),
    )
    .unwrap();
    let mut relocated = fixture.adapter.clone();
    relocated.content.executable.locator = "relocated/renamed.bin".into();
    relocated.content.executable.resolved = relocated_directory.join("renamed.bin");
    relocated.content.resources[0].locator = "relocated/renamed.json".into();
    relocated.content.resources[0].resolved = relocated_directory.join("renamed.json");
    assert_eq!(
        adapter_fingerprint(&relocated),
        fixture.adapter.adapter_fingerprint
    );
    assert_eq!(
        configuration_fingerprint(&relocated),
        fixture.adapter.configuration_fingerprint
    );
    let parsed =
        parse_configuration(&fixture.config_path, &configuration_json(&relocated)).unwrap();
    assert_eq!(
        parsed.adapters[0].configuration_fingerprint,
        fixture.adapter.configuration_fingerprint
    );
}

#[test]
fn parses_exact_description_and_fails_closed_on_drift() {
    let fixture = Fixture::new();
    let parsed = parse_description(&description_json(&fixture.adapter)).unwrap();
    validate_description(&fixture.adapter, &parsed).unwrap();

    let mut drifted = parsed.clone();
    drifted.build = "substituted".into();
    assert!(validate_description(&fixture.adapter, &drifted)
        .unwrap_err()
        .contains("differs"));

    let internally_invalid = description_json(&fixture.adapter).replacen(
        "\"adapter_version\":\"1\"",
        "\"adapter_version\":\"2\"",
        1,
    );
    assert!(parse_description(&internally_invalid).is_err());
}

#[test]
fn request_identity_excludes_locators_and_enforces_operation_cardinality() {
    let launch = format!("sha256:{}", "3".repeat(64));
    let digest = format!("sha256:{}", "4".repeat(64));
    let inputs = vec![InputIdentity {
        id: "native/report".into(),
        digest,
        size_bytes: 42,
    }];
    let first = run_request_fingerprint(AdapterOperation::Import, &launch, &inputs, &[]).unwrap();
    let second = run_request_fingerprint(AdapterOperation::Import, &launch, &inputs, &[]).unwrap();
    assert_eq!(first, second);
    assert!(run_request_fingerprint(AdapterOperation::Execute, &launch, &inputs, &[]).is_err());
    assert!(run_request_fingerprint(AdapterOperation::Import, &launch, &[], &[]).is_err());
    assert!(run_request_fingerprint(AdapterOperation::Describe, &launch, &[], &[]).is_err());
    assert!(run_request_fingerprint(
        AdapterOperation::Import,
        &launch,
        &inputs,
        &[PredecessorIdentity {
            bundle_revision: 1,
            bundle_fingerprint: format!("sha256:{}", "5".repeat(64)),
        }],
    )
    .is_err());
}

#[test]
fn strict_json_accepts_integral_forms_and_rejects_nonintegral_and_invalid_unicode() {
    let fixture = Fixture::new();
    assert!(parse_configuration(
        &fixture.config_path,
        "{\"format\":\"azimuth-adapter-configuration\",\"version\":1.0,\"adapters\":[]}"
    )
    .is_ok());
    assert!(parse_configuration(
        &fixture.config_path,
        "{\"format\":\"azimuth-adapter-configuration\",\"version\":1e0,\"adapters\":[]}"
    )
    .is_ok());
    assert!(parse_configuration(
        &fixture.config_path,
        "{\"format\":\"azimuth-adapter-configuration\",\"version\":0.1e1,\"adapters\":[]}"
    )
    .is_ok());
    assert!(parse_configuration(
        &fixture.config_path,
        "{\"format\":\"azimuth-adapter-configuration\",\"version\":1.1,\"adapters\":[]}"
    )
    .is_err());
    assert!(parse_configuration(
        &fixture.config_path,
        concat!(
            "{\"format\":\"azimuth-adapter-configuration\",\"version\":1,",
            "\"adapters\":[],\"x\":\"\\ud800\"}"
        )
    )
    .is_err());
}
