use super::*;
use azimuth::adapter::*;
use std::{path::PathBuf, sync::OnceLock};

pub(super) fn adapter() -> Result<&'static ConfiguredAdapter, LedgerError> {
    static ADAPTER: OnceLock<Result<ConfiguredAdapter, String>> = OnceLock::new();
    ADAPTER
        .get_or_init(build)
        .as_ref()
        .map_err(|error| LedgerError::invalid(error.clone()))
}
fn build() -> Result<ConfiguredAdapter, String> {
    let executable = std::env::current_exe()
        .map_err(|e| format!("normalizer executable identity unavailable: {e}"))?;
    let content = std::fs::read(&executable)
        .map_err(|e| format!("normalizer executable content unavailable: {e}"))?;
    let mut configured = ConfiguredAdapter {
        id: "assurance-target-observations".into(),
        provider_family: "azimuth/assurance".into(),
        protocol_version: 1,
        adapter_version: "1".into(),
        build: "in-process-target-resolution".into(),
        content: AdapterContent {
            executable: ConfiguredFile {
                locator: "in-process:run-ledger".into(),
                resolved: executable,
                digest: format!("sha256:{}", sha256(&content)),
            },
            resources: vec![
                resource(
                    "native-adapter",
                    include_bytes!("../../../../../tools/azimuth/src/adapter.rs"),
                ),
                resource(
                    "native-run",
                    include_bytes!("../../../../../tools/azimuth/src/run.rs"),
                ),
                resource("resolver", include_bytes!("target_observation.rs")),
                resource("resolver-identity", include_bytes!("target_normalizer.rs")),
            ],
        },
        semantic_settings: BTreeMap::from([
            (
                "applicability".into(),
                "independent-historical-bracket".into(),
            ),
            ("outcomes".into(), "violated,inconclusive".into()),
        ]),
        environment: AdapterEnvironment {
            literals: BTreeMap::new(),
        },
        limits: AdapterLimits {
            timeout_ms: 30000,
            stdout_bytes: 262144,
            stderr_bytes: 65536,
        },
        capabilities: vec![Capability {
            id: "checks".into(),
            classes: vec![CapabilityClass::CheckImport],
            challenge_forms: vec![],
            semantic_settings: BTreeMap::new(),
            fingerprint: String::new(),
        }],
        adapter_fingerprint: String::new(),
        descriptor_fingerprint: String::new(),
        configuration_fingerprint: String::new(),
    };
    configured.adapter_fingerprint = adapter_fingerprint(&configured);
    configured.capabilities[0].fingerprint =
        capability_fingerprint(&configured.adapter_fingerprint, &configured.capabilities[0]);
    configured.descriptor_fingerprint = descriptor_fingerprint(&configured.expected_description());
    configured.configuration_fingerprint = configuration_fingerprint(&configured);
    let description = record(&configured)["description"].clone();
    parse_description(&description.to_string()).map_err(|errors| {
        errors
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    Ok(configured)
}
fn resource(id: &str, bytes: &[u8]) -> ConfiguredResource {
    ConfiguredResource {
        id: id.into(),
        locator: format!("compiled-source:{id}"),
        resolved: PathBuf::new(),
        digest: format!("sha256:{}", sha256(bytes)),
    }
}
pub(super) fn record(adapter: &ConfiguredAdapter) -> Value {
    let capabilities=adapter.capabilities.iter().map(|c|json!({"id":c.id,"classes":c.classes.iter().map(|class|class.name()).collect::<Vec<_>>(),"challenge_forms":c.challenge_forms,"semantic_settings":c.semantic_settings,"fingerprint":c.fingerprint})).collect::<Vec<_>>();
    json!({"description":{"format":"azimuth-adapter-description","version":1,"protocol_version":adapter.protocol_version,"id":adapter.id,"provider_family":adapter.provider_family,"adapter_version":adapter.adapter_version,"build":adapter.build,"content":{"executable_digest":adapter.content.executable.digest,"resources":adapter.content.resources.iter().map(|r|json!({"id":r.id,"digest":r.digest})).collect::<Vec<_>>()},"adapter_fingerprint":adapter.adapter_fingerprint,"capabilities":capabilities,"descriptor_fingerprint":adapter.descriptor_fingerprint},"configuration":{"fingerprint":adapter.configuration_fingerprint,"semantic_settings":adapter.semantic_settings,"environment":{"literals":adapter.environment.literals},"limits":{"timeout_ms":adapter.limits.timeout_ms,"stdout_bytes":adapter.limits.stdout_bytes,"stderr_bytes":adapter.limits.stderr_bytes},"capabilities":capabilities}})
}
pub(super) async fn list(
    State(state): State<LedgerState>,
    Path(project): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, LedgerError> {
    state.authorize(&project, &headers, None)?;
    let records:Vec<Value>=sqlx::query_scalar("SELECT assurance_record('target_normalizer',project,record_key) FROM assurance_records WHERE kind='target_normalizer' AND project=$1 ORDER BY record_key LIMIT 501").bind(project).fetch_all(&state.pool).await?;
    if records.len() > 500 {
        return Err(LedgerError::invalid(
            "normalizer history exceeds bounded page",
        ));
    }
    Ok(Json(json!({"records":records})))
}
