//! Bounded process host for the D47 adapter protocol.
//!
//! The host owns one process exchange and complete response validation. It deliberately does not
//! own CLI parsing or output-file publication: callers receive canonical bytes only after the
//! returned Run has passed every configured, launch and correction-chain check.

use crate::adapter::{
    self, AdapterDescription, AdapterOperation, AdapterResponseStatus, ConfiguredAdapter,
    InputIdentity, PredecessorIdentity, StagedAdapterContent,
};
use crate::json::Json;
use crate::run::{self, ImportInputIdentity, ProvenanceMode, RunBundle};
use crate::run_plan::{self, LaunchPlan, RunOperation};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const REQUEST_FORMAT: &str = "azimuth-adapter-request";
const VERSION: u64 = 1;
static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostErrorClass {
    Semantic,
    Schema,
}

impl HostErrorClass {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Semantic => 1,
            Self::Schema => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostError {
    pub class: HostErrorClass,
    pub detail: String,
    pub stderr: String,
}

impl HostError {
    fn semantic(detail: impl Into<String>) -> Self {
        Self {
            class: HostErrorClass::Semantic,
            detail: detail.into(),
            stderr: String::new(),
        }
    }

    fn schema(detail: impl Into<String>) -> Self {
        Self {
            class: HostErrorClass::Schema,
            detail: detail.into(),
            stderr: String::new(),
        }
    }

    fn with_stderr(mut self, stderr: &[u8]) -> Self {
        self.stderr = String::from_utf8_lossy(stderr).into_owned();
        self
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportInput {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedBundle {
    pub bundle: RunBundle,
    pub canonical_json: String,
}

/// Performs the strict description exchange for one already parsed configuration entry.
pub fn verify_adapter(adapter: &ConfiguredAdapter) -> Result<AdapterDescription, HostError> {
    validate_configured_adapter(adapter)?;
    let mut stage = InvocationStage::new()?;
    let content = adapter::stage_content(adapter, stage.path()).map_err(HostError::semantic)?;
    let request_id =
        adapter::describe_request_fingerprint(&adapter.id, &adapter.configuration_fingerprint)
            .map_err(HostError::semantic)?;
    let request = describe_request_json(adapter, &content, &request_id);
    let request = run::canonical_json(&request).map_err(HostError::schema)?;
    let exchange = exchange(adapter, &content.executable.path, stage.path(), request)?;
    let response =
        parse_response(&exchange.stdout).map_err(|error| error.with_stderr(&exchange.stderr))?;

    if response.operation != AdapterOperation::Describe {
        return Err(
            HostError::semantic("adapter response operation differs from `describe`")
                .with_stderr(&exchange.stderr),
        );
    }
    if response.request_id != request_id {
        return Err(
            HostError::semantic("adapter response request identity differs from request")
                .with_stderr(&exchange.stderr),
        );
    }
    adapter::validate_description(adapter, &response.description)
        .map_err(HostError::semantic)
        .map_err(|error| error.with_stderr(&exchange.stderr))?;
    if response.status != AdapterResponseStatus::Ok {
        return Err(explicit_failure(&response).with_stderr(&exchange.stderr));
    }
    stage.cleanup()?;
    Ok(response.description)
}

/// Executes one launch through its configured adapter and returns only a validated candidate.
pub fn execute(
    configuration: &adapter::AdapterConfiguration,
    launch: &LaunchPlan,
    predecessors: &[RunBundle],
) -> Result<HostedBundle, HostError> {
    invoke_run(
        configuration,
        launch,
        &[],
        predecessors,
        RunOperation::Execute,
    )
}

/// Imports exact native files through one launch and returns only a validated candidate.
pub fn import(
    configuration: &adapter::AdapterConfiguration,
    launch: &LaunchPlan,
    inputs: &[ImportInput],
    predecessors: &[RunBundle],
) -> Result<HostedBundle, HostError> {
    invoke_run(
        configuration,
        launch,
        inputs,
        predecessors,
        RunOperation::Import,
    )
}

fn invoke_run(
    configuration: &adapter::AdapterConfiguration,
    launch: &LaunchPlan,
    inputs: &[ImportInput],
    predecessors: &[RunBundle],
    operation: RunOperation,
) -> Result<HostedBundle, HostError> {
    if launch.operation != operation {
        return Err(HostError::semantic(format!(
            "launch operation `{}` cannot be used for `{}`",
            launch.operation.name(),
            operation.name()
        )));
    }
    let adapter = configured_launch_adapter(configuration, launch)?;
    let predecessor_chain = prepare_predecessors(launch, predecessors)?;
    validate_input_arguments(operation, inputs)?;

    let mut stage = InvocationStage::new()?;
    let content = adapter::stage_content(adapter, stage.path()).map_err(HostError::semantic)?;
    let staged_inputs = stage_inputs(stage.path(), inputs)?;
    let input_identities = staged_inputs
        .iter()
        .map(|input| input.identity.clone())
        .collect::<Vec<_>>();
    let predecessor_identities = predecessor_chain
        .iter()
        .map(|bundle| PredecessorIdentity {
            bundle_revision: bundle.bundle_revision,
            bundle_fingerprint: bundle.bundle_fingerprint.clone(),
        })
        .collect::<Vec<_>>();
    let adapter_operation = operation.adapter_operation();
    let request_id = adapter::run_request_fingerprint(
        adapter_operation,
        &launch.fingerprint,
        &input_identities,
        &predecessor_identities,
    )
    .map_err(HostError::semantic)?;
    let request = run_request_json(
        adapter,
        &content,
        launch,
        &staged_inputs,
        &predecessor_chain,
        &predecessor_identities,
        &request_id,
    )?;
    let request = run::canonical_json(&request).map_err(HostError::schema)?;
    let exchange = exchange(adapter, &content.executable.path, stage.path(), request)?;
    let response =
        parse_response(&exchange.stdout).map_err(|error| error.with_stderr(&exchange.stderr))?;

    if response.operation != adapter_operation {
        return Err(
            HostError::semantic("adapter response operation differs from request")
                .with_stderr(&exchange.stderr),
        );
    }
    if response.request_id != request_id {
        return Err(
            HostError::semantic("adapter response request identity differs from request")
                .with_stderr(&exchange.stderr),
        );
    }
    if response.launch_fingerprint.as_deref() != Some(launch.fingerprint.as_str()) {
        return Err(
            HostError::semantic("adapter response launch identity differs from request")
                .with_stderr(&exchange.stderr),
        );
    }
    adapter::validate_description(adapter, &response.description)
        .map_err(HostError::semantic)
        .map_err(|error| error.with_stderr(&exchange.stderr))?;
    if response.status != AdapterResponseStatus::Ok {
        return Err(explicit_failure(&response).with_stderr(&exchange.stderr));
    }

    let bundle_source = response
        .bundle_json
        .as_deref()
        .ok_or_else(|| HostError::schema("successful adapter response omitted its bundle"))?;
    let bundle = run::parse("adapter response bundle", bundle_source)
        .map_err(|errors| HostError::schema(join_schema_errors(&errors)))?;
    validate_returned_bundle(
        adapter,
        launch,
        &input_identities,
        &predecessor_chain,
        &bundle,
    )?;
    let canonical_json = run::canonical_json(&run::to_json(&bundle)).map_err(HostError::schema)?;
    stage.cleanup()?;
    Ok(HostedBundle {
        bundle,
        canonical_json,
    })
}

fn configured_launch_adapter<'a>(
    configuration: &'a adapter::AdapterConfiguration,
    launch: &LaunchPlan,
) -> Result<&'a ConfiguredAdapter, HostError> {
    let canonical =
        run::canonical_json(&run_plan::launch_plan_to_json(launch)).map_err(HostError::semantic)?;
    let reparsed =
        run_plan::parse_launch_plan("host launch plan", &canonical).map_err(|errors| {
            HostError::semantic(format!(
                "launch plan cannot pass the strict parser: {}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ))
        })?;
    if reparsed != *launch {
        return Err(HostError::semantic(
            "launch plan typed value does not equal its strict canonical form",
        ));
    }
    let launch_errors = run_plan::validate_launch_plan(launch);
    if !launch_errors.is_empty() {
        return Err(HostError::semantic(format!(
            "launch plan is invalid: {}",
            launch_errors.join("; ")
        )));
    }
    let configuration_errors = run_plan::validate_launch_configuration(launch, configuration);
    if !configuration_errors.is_empty() {
        return Err(HostError::semantic(format!(
            "launch configuration mismatch: {}",
            configuration_errors.join("; ")
        )));
    }
    let adapter = configuration.adapter(&launch.adapter.id).ok_or_else(|| {
        HostError::semantic(format!(
            "launch adapter `{}` is not configured",
            launch.adapter.id
        ))
    })?;
    validate_configured_adapter(adapter)?;
    Ok(adapter)
}

fn validate_configured_adapter(adapter: &ConfiguredAdapter) -> Result<(), HostError> {
    if adapter.protocol_version != VERSION {
        return Err(HostError::semantic(
            "configured adapter protocol version differs from host protocol",
        ));
    }
    if adapter::adapter_fingerprint(adapter) != adapter.adapter_fingerprint {
        return Err(HostError::semantic(
            "configured adapter fingerprint differs from its content identity",
        ));
    }
    for capability in &adapter.capabilities {
        if adapter::capability_fingerprint(&adapter.adapter_fingerprint, capability)
            != capability.fingerprint
        {
            return Err(HostError::semantic(format!(
                "configured capability `{}` fingerprint differs from its declaration",
                capability.id
            )));
        }
    }
    if adapter::descriptor_fingerprint(&adapter.expected_description())
        != adapter.descriptor_fingerprint
    {
        return Err(HostError::semantic(
            "configured descriptor fingerprint differs from its description",
        ));
    }
    if adapter::configuration_fingerprint(adapter) != adapter.configuration_fingerprint {
        return Err(HostError::semantic(
            "configured invocation fingerprint differs from its settings and limits",
        ));
    }
    Ok(())
}

fn validate_input_arguments(
    operation: RunOperation,
    inputs: &[ImportInput],
) -> Result<(), HostError> {
    if operation == RunOperation::Execute && !inputs.is_empty() {
        return Err(HostError::schema("execute does not accept import inputs"));
    }
    if operation == RunOperation::Import && inputs.is_empty() {
        return Err(HostError::schema("import requires at least one input"));
    }
    let mut previous = None;
    for input in inputs {
        if input.id.is_empty() || input.id.split('/').any(|part| !valid_kebab_segment(part)) {
            return Err(HostError::schema(format!(
                "import input id `{}` is not a lower-kebab path id",
                input.id
            )));
        }
        if previous.is_some_and(|value: &str| value >= input.id.as_str()) {
            return Err(HostError::schema(
                "import inputs must be sorted by unique id",
            ));
        }
        previous = Some(input.id.as_str());
    }
    Ok(())
}

#[derive(Debug)]
struct StagedInput {
    identity: InputIdentity,
    path: PathBuf,
}

fn stage_inputs(stage: &Path, inputs: &[ImportInput]) -> Result<Vec<StagedInput>, HostError> {
    let mut staged = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.iter().enumerate() {
        let destination = stage.join(format!("adapter-input-{index:04}"));
        let file = adapter::stage_file(&input.path, &destination, None, None, false)
            .map_err(HostError::semantic)?;
        let identity = file.input_identity(&input.id).map_err(HostError::schema)?;
        staged.push(StagedInput {
            identity,
            path: file.path,
        });
    }
    Ok(staged)
}

fn prepare_predecessors(
    launch: &LaunchPlan,
    supplied: &[RunBundle],
) -> Result<Vec<RunBundle>, HostError> {
    let mut unique = Vec::<RunBundle>::new();
    for bundle in supplied {
        if !unique.contains(bundle) {
            unique.push(bundle.clone());
        }
    }
    unique.sort_by(|left, right| {
        left.bundle_revision
            .cmp(&right.bundle_revision)
            .then_with(|| left.bundle_fingerprint.cmp(&right.bundle_fingerprint))
    });
    if unique.is_empty() {
        return Ok(unique);
    }
    let run_ids = unique
        .iter()
        .map(|bundle| bundle.run_id.as_str())
        .collect::<BTreeSet<_>>();
    if run_ids.len() != 1 {
        return Err(HostError::semantic(
            "predecessors must contain exactly one Run correction chain",
        ));
    }
    let findings = run::verify_set(&unique);
    if !findings.is_empty() {
        return Err(HostError::semantic(format!(
            "predecessor correction chain is invalid: {}",
            join_findings(&findings)
        )));
    }
    for (index, bundle) in unique.iter().enumerate() {
        if bundle.bundle_revision != index as u64 {
            return Err(HostError::semantic(format!(
                "predecessor revision {} is not contiguous at position {index}",
                bundle.bundle_revision
            )));
        }
        validate_bundle_launch_identity(launch, bundle, "predecessor")?;
    }
    Ok(unique)
}

fn validate_returned_bundle(
    adapter: &ConfiguredAdapter,
    launch: &LaunchPlan,
    inputs: &[InputIdentity],
    predecessors: &[RunBundle],
    bundle: &RunBundle,
) -> Result<(), HostError> {
    validate_bundle_launch_identity(launch, bundle, "returned bundle")?;
    let expected_mode = match launch.operation {
        RunOperation::Execute => ProvenanceMode::Execute,
        RunOperation::Import => ProvenanceMode::Import,
    };
    if bundle.provenance.mode != expected_mode {
        return Err(HostError::semantic(
            "returned bundle provenance mode differs from launch operation",
        ));
    }
    let expected_inputs = inputs
        .iter()
        .map(|input| ImportInputIdentity {
            id: input.id.clone(),
            digest: input.digest.clone(),
            size_bytes: input.size_bytes,
        })
        .collect::<Vec<_>>();
    let provenance = &bundle.provenance;
    let returned_adapter = &provenance.adapter;
    let expected_normalizer_id = format!("adapter/{}", adapter.id);
    if provenance.normalizer.id != expected_normalizer_id
        || provenance.normalizer.version != adapter.adapter_version
        || provenance.normalizer.build_fingerprint != adapter.adapter_fingerprint
    {
        return Err(HostError::semantic(
            "returned bundle normalizer differs from configured adapter identity",
        ));
    }
    if returned_adapter.id != adapter.id
        || returned_adapter.adapter_version != adapter.adapter_version
        || returned_adapter.adapter_fingerprint != adapter.adapter_fingerprint
        || returned_adapter.descriptor_fingerprint != adapter.descriptor_fingerprint
        || returned_adapter.configuration_fingerprint != adapter.configuration_fingerprint
        || returned_adapter.launch_fingerprint != launch.fingerprint
        || returned_adapter.routes != launch.routes
        || returned_adapter.import_inputs != expected_inputs
    {
        return Err(HostError::semantic(
            "returned bundle adapter provenance differs from launch and request",
        ));
    }

    let findings = run::verify(bundle);
    if !findings.is_empty() {
        return Err(HostError::semantic(format!(
            "returned Run bundle is invalid: {}",
            join_findings(&findings)
        )));
    }
    match predecessors.last() {
        None => {
            if bundle.bundle_revision != 0
                || bundle.corrects.is_some()
                || bundle.correction_reason.is_some()
            {
                return Err(HostError::semantic(
                    "a Run without predecessors must return revision zero",
                ));
            }
        }
        Some(terminal) => {
            if bundle.bundle_revision != terminal.bundle_revision + 1
                || bundle.corrects.as_deref() != Some(terminal.bundle_fingerprint.as_str())
            {
                return Err(HostError::semantic(
                    "returned correction is not the exact successor of the terminal predecessor",
                ));
            }
            if bundle.run_id != terminal.run_id {
                return Err(HostError::semantic(
                    "returned correction belongs to a different Run",
                ));
            }
            let mut combined = predecessors.to_vec();
            combined.push(bundle.clone());
            if combined
                .iter()
                .map(|item| item.run_id.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                != 1
            {
                return Err(HostError::semantic(
                    "combined correction account contains more than one Run",
                ));
            }
            let findings = run::verify_set(&combined);
            if !findings.is_empty() {
                return Err(HostError::semantic(format!(
                    "returned correction chain is invalid: {}",
                    join_findings(&findings)
                )));
            }
        }
    }
    Ok(())
}

fn validate_bundle_launch_identity(
    launch: &LaunchPlan,
    bundle: &RunBundle,
    label: &str,
) -> Result<(), HostError> {
    let adapter = &bundle.provenance.adapter;
    let expected_mode = match launch.operation {
        RunOperation::Execute => ProvenanceMode::Execute,
        RunOperation::Import => ProvenanceMode::Import,
    };
    if bundle.subject != launch.subject
        || bundle.subject_fingerprint != launch.subject_fingerprint
        || bundle.planned_at_ms != launch.planned_at_ms
        || bundle.plan != launch.plan
        || bundle.provenance.mode != expected_mode
        || adapter.id != launch.adapter.id
        || adapter.adapter_version != launch.adapter.adapter_version
        || adapter.adapter_fingerprint != launch.adapter.adapter_fingerprint
        || adapter.descriptor_fingerprint != launch.adapter.descriptor_fingerprint
        || adapter.configuration_fingerprint != launch.adapter.configuration_fingerprint
        || adapter.launch_fingerprint != launch.fingerprint
        || adapter.routes != launch.routes
        || bundle.provenance.normalizer.id != format!("adapter/{}", launch.adapter.id)
        || bundle.provenance.normalizer.version != launch.adapter.adapter_version
        || bundle.provenance.normalizer.build_fingerprint != launch.adapter.adapter_fingerprint
    {
        return Err(HostError::semantic(format!(
            "{label} differs from the launch Subject, time, Plan or adapter route"
        )));
    }
    Ok(())
}

fn describe_request_json(
    adapter: &ConfiguredAdapter,
    content: &StagedAdapterContent,
    request_id: &str,
) -> Json {
    Json::obj(vec![
        ("format", Json::str(REQUEST_FORMAT)),
        ("version", Json::Num(VERSION as f64)),
        ("request_id", Json::str(request_id)),
        ("operation", Json::str("describe")),
        (
            "adapter",
            Json::obj(vec![
                ("id", Json::str(&adapter.id)),
                (
                    "configuration_fingerprint",
                    Json::str(&adapter.configuration_fingerprint),
                ),
            ]),
        ),
        ("configuration", configuration_json(adapter, content, None)),
    ])
}

fn run_request_json(
    adapter: &ConfiguredAdapter,
    content: &StagedAdapterContent,
    launch: &LaunchPlan,
    inputs: &[StagedInput],
    predecessors: &[RunBundle],
    predecessor_identities: &[PredecessorIdentity],
    request_id: &str,
) -> Result<Json, HostError> {
    let selected = launch
        .routes
        .iter()
        .map(|route| route.capability.address.as_str())
        .collect::<BTreeSet<_>>();
    let terminal = predecessors.last().map(run::to_json).unwrap_or(Json::Null);
    Ok(Json::obj(vec![
        ("format", Json::str(REQUEST_FORMAT)),
        ("version", Json::Num(VERSION as f64)),
        ("request_id", Json::str(request_id)),
        ("operation", Json::str(launch.operation.name())),
        ("launch_plan", run_plan::launch_plan_to_json(launch)),
        (
            "configuration",
            configuration_json(adapter, content, Some(&selected)),
        ),
        (
            "inputs",
            Json::Arr(
                inputs
                    .iter()
                    .map(|input| {
                        Json::obj(vec![
                            ("id", Json::str(&input.identity.id)),
                            ("digest", Json::str(&input.identity.digest)),
                            ("size_bytes", Json::Num(input.identity.size_bytes as f64)),
                            (
                                "locator",
                                Json::str(input.path.to_string_lossy().into_owned()),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "predecessors",
            Json::Arr(
                predecessor_identities
                    .iter()
                    .map(|identity| {
                        Json::obj(vec![
                            (
                                "bundle_revision",
                                Json::Num(identity.bundle_revision as f64),
                            ),
                            (
                                "bundle_fingerprint",
                                Json::str(&identity.bundle_fingerprint),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("terminal_predecessor", terminal),
    ]))
}

fn configuration_json(
    adapter: &ConfiguredAdapter,
    content: &StagedAdapterContent,
    selected: Option<&BTreeSet<&str>>,
) -> Json {
    let resource_paths = content
        .resources
        .iter()
        .map(|resource| (resource.id.as_str(), resource.path.as_path()))
        .collect::<BTreeMap<_, _>>();
    let capabilities = adapter
        .capabilities
        .iter()
        .filter(|capability| {
            selected.is_none_or(|addresses| {
                let address = capability.address(&adapter.id);
                addresses.contains(address.as_str())
            })
        })
        .map(|capability| {
            Json::obj(vec![
                ("address", Json::str(capability.address(&adapter.id))),
                ("fingerprint", Json::str(&capability.fingerprint)),
                (
                    "semantic_settings",
                    string_map_json(&capability.semantic_settings),
                ),
            ])
        })
        .collect();
    Json::obj(vec![
        ("fingerprint", Json::str(&adapter.configuration_fingerprint)),
        (
            "semantic_settings",
            string_map_json(&adapter.semantic_settings),
        ),
        (
            "resources",
            Json::Arr(
                adapter
                    .content
                    .resources
                    .iter()
                    .map(|resource| {
                        let path = resource_paths
                            .get(resource.id.as_str())
                            .expect("staged resources match configured resources");
                        Json::obj(vec![
                            ("id", Json::str(&resource.id)),
                            ("digest", Json::str(&resource.digest)),
                            ("locator", Json::str(path.to_string_lossy().into_owned())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("capabilities", Json::Arr(capabilities)),
    ])
}

fn string_map_json(values: &BTreeMap<String, String>) -> Json {
    Json::Obj(
        values
            .iter()
            .map(|(key, value)| (key.clone(), Json::str(value)))
            .collect(),
    )
}

struct InvocationStage {
    path: Option<PathBuf>,
}

impl InvocationStage {
    fn new() -> Result<Self, HostError> {
        let base = std::env::temp_dir();
        for _ in 0..64 {
            let sequence = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = base.join(format!(
                "azimuth-adapter-invocation-{}-{epoch}-{sequence}",
                std::process::id()
            ));
            match create_private_directory(&path) {
                Ok(()) => return Ok(Self { path: Some(path) }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(HostError::semantic(format!(
                        "private adapter staging directory cannot be created: {error}"
                    )))
                }
            }
        }
        Err(HostError::semantic(
            "private adapter staging directory could not be allocated",
        ))
    }

    fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("an active invocation stage has a path")
    }

    fn cleanup(&mut self) -> Result<(), HostError> {
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };
        prepare_tree_for_removal(path).map_err(|error| {
            HostError::semantic(format!(
                "adapter staging directory could not be prepared for cleanup: {error}"
            ))
        })?;
        fs::remove_dir_all(path).map_err(|error| {
            HostError::semantic(format!(
                "adapter staging directory could not be removed: {error}"
            ))
        })?;
        self.path = None;
        Ok(())
    }
}

impl Drop for InvocationStage {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = prepare_tree_for_removal(&path);
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn prepare_tree_for_removal(path: &Path) -> std::io::Result<()> {
    let root_metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if !root_metadata.is_dir() {
        return Ok(());
    }
    make_directory_traversable(path, root_metadata.permissions())?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let metadata = fs::symlink_metadata(&child)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            make_directory_traversable(&child, metadata.permissions())?;
            prepare_tree_for_removal(&child)?;
        }
        make_removable(&child, metadata.permissions())?;
    }
    Ok(())
}

#[cfg(unix)]
fn make_directory_traversable(path: &Path, _permissions: fs::Permissions) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn make_directory_traversable(
    path: &Path,
    mut permissions: fs::Permissions,
) -> std::io::Result<()> {
    if permissions.readonly() {
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[cfg(unix)]
fn make_removable(_path: &Path, _permissions: fs::Permissions) -> std::io::Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn make_removable(path: &Path, mut permissions: fs::Permissions) -> std::io::Result<()> {
    if permissions.readonly() {
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700).create(path)
}

#[cfg(not(unix))]
fn create_private_directory(path: &Path) -> std::io::Result<()> {
    fs::create_dir(path)
}

struct ExchangeOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn exchange(
    adapter: &ConfiguredAdapter,
    executable: &Path,
    working_directory: &Path,
    mut request: String,
) -> Result<ExchangeOutput, HostError> {
    request.push('\n');
    let mut command = Command::new(executable);
    configure_process_tree(&mut command)?;
    command
        .current_dir(working_directory)
        .env_clear()
        .envs(adapter.child_environment())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| {
        HostError::semantic(format!("adapter process could not start: {error}"))
    })?;
    let process_tree = ProcessTree::new(&child)?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| HostError::semantic("adapter standard input was unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| HostError::semantic("adapter standard output was unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| HostError::semantic("adapter standard error was unavailable"))?;

    let (writer_tx, writer_rx) = mpsc::channel();
    let writer = thread::spawn(move || {
        let result = write_request(stdin, request.as_bytes());
        let _ = writer_tx.send(result);
    });
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let stdout_limit = adapter.limits.stdout_bytes;
    let stdout_reader = thread::spawn(move || {
        let result = read_capped(stdout, stdout_limit, "standard output");
        let _ = stdout_tx.send(result);
    });
    let (stderr_tx, stderr_rx) = mpsc::channel();
    let stderr_limit = adapter.limits.stderr_bytes;
    let stderr_reader = thread::spawn(move || {
        let result = read_capped(stderr, stderr_limit, "standard error");
        let _ = stderr_tx.send(result);
    });

    let started = Instant::now();
    let timeout = Duration::from_millis(adapter.limits.timeout_ms);
    let mut writer_result = None;
    let mut stdout_result = None;
    let mut stderr_result = None;
    let mut status = None;
    let mut early_error = None;
    let mut termination_requested = false;
    loop {
        if writer_result.is_none() {
            writer_result = writer_rx.try_recv().ok();
        }
        if stdout_result.is_none() {
            stdout_result = stdout_rx.try_recv().ok();
        }
        if stderr_result.is_none() {
            stderr_result = stderr_rx.try_recv().ok();
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(Some(actual)) => status = Some(actual),
                Ok(None) => {}
                Err(error) => {
                    early_error.get_or_insert_with(|| {
                        format!("adapter process status could not be read: {error}")
                    });
                }
            }
        }

        if let Some(Err(error)) = &writer_result {
            early_error.get_or_insert_with(|| error.clone());
        }
        if let Some(Err(error)) = &stdout_result {
            early_error.get_or_insert_with(|| error.clone());
        }
        if let Some(Err(error)) = &stderr_result {
            early_error.get_or_insert_with(|| error.clone());
        }
        if status.is_some() && !termination_requested {
            process_tree.terminate_descendants();
            termination_requested = true;
        }
        if early_error.is_some() && !termination_requested {
            process_tree.terminate(&mut child);
            termination_requested = true;
        }

        let complete = writer_result.is_some()
            && stdout_result.is_some()
            && stderr_result.is_some()
            && status.is_some();
        if complete {
            break;
        }
        if started.elapsed() >= timeout {
            early_error.get_or_insert_with(|| {
                "adapter exchange exceeded its configured timeout before all streams closed".into()
            });
            if status.is_none() {
                process_tree.terminate(&mut child);
                status = child.wait().ok();
            } else {
                process_tree.terminate_descendants();
            }
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }

    // A channel result means the worker has completed its blocking operation; joins cannot extend
    // the protocol deadline in the fully successful case. In deadline failures unfinished workers
    // are detached so descendant-held pipe handles cannot stall the host.
    if writer_result.is_some() {
        let _ = writer.join();
    }
    if stdout_result.is_some() {
        let _ = stdout_reader.join();
    }
    if stderr_result.is_some() {
        let _ = stderr_reader.join();
    }
    if status.is_some() {
        if let Ok(reaped) = child.wait() {
            status = Some(reaped);
        }
    }
    let stderr = stderr_result
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    if let Some(error) = early_error {
        return Err(HostError::semantic(error).with_stderr(&stderr));
    }
    let writer_result = writer_result
        .ok_or_else(|| HostError::semantic("adapter request writer did not complete"))?;
    if let Err(error) = writer_result {
        return Err(HostError::semantic(error).with_stderr(&stderr));
    }
    let stdout_result = stdout_result
        .ok_or_else(|| HostError::semantic("adapter standard-output reader did not complete"))?;
    let stdout = stdout_result.map_err(|error| HostError::semantic(error).with_stderr(&stderr))?;
    let stderr_result = stderr_result
        .ok_or_else(|| HostError::semantic("adapter standard-error reader did not complete"))?;
    stderr_result.map_err(|error| HostError::semantic(error).with_stderr(&stderr))?;
    let status = status.ok_or_else(|| {
        HostError::semantic("adapter process status was unavailable").with_stderr(&stderr)
    })?;
    if !status.success() {
        return Err(nonzero_status(status).with_stderr(&stderr));
    }
    Ok(ExchangeOutput { stdout, stderr })
}

fn write_request(mut stdin: impl Write, bytes: &[u8]) -> Result<(), String> {
    stdin
        .write_all(bytes)
        .map_err(|error| format!("adapter request could not be written: {error}"))?;
    stdin
        .flush()
        .map_err(|error| format!("adapter request could not be flushed: {error}"))?;
    drop(stdin);
    Ok(())
}

fn read_capped(mut input: impl Read, limit: u64, name: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|error| format!("adapter {name} could not be read: {error}"))?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len() as u64 + count as u64 > limit {
            return Err(format!("adapter {name} exceeded its configured byte limit"));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

#[cfg(unix)]
fn configure_process_tree(command: &mut Command) -> Result<(), HostError> {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
    Ok(())
}

#[cfg(not(unix))]
fn configure_process_tree(_command: &mut Command) -> Result<(), HostError> {
    Err(HostError::semantic(
        "bounded adapter process-tree containment is unavailable on this platform",
    ))
}

struct ProcessTree {
    #[cfg(unix)]
    group: i32,
}

impl ProcessTree {
    fn new(child: &std::process::Child) -> Result<Self, HostError> {
        #[cfg(unix)]
        {
            let group = i32::try_from(child.id()).map_err(|_| {
                HostError::semantic("adapter process id exceeds the Unix pid range")
            })?;
            Ok(Self { group })
        }
        #[cfg(not(unix))]
        {
            let _ = child;
            Err(HostError::semantic(
                "bounded adapter process-tree containment is unavailable on this platform",
            ))
        }
    }

    fn terminate(&self, child: &mut std::process::Child) {
        self.terminate_descendants();
        let _ = child.kill();
    }

    fn terminate_descendants(&self) {
        #[cfg(unix)]
        unsafe {
            // Negative pid addresses the process group created immediately before spawn.
            let _ = kill(-self.group, SIGKILL);
        }
    }
}

#[cfg(unix)]
const SIGKILL: i32 = 9;

#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}

fn nonzero_status(status: ExitStatus) -> HostError {
    HostError::semantic(format!("adapter process exited with status {status}"))
}

fn parse_response(bytes: &[u8]) -> Result<adapter::AdapterResponse, HostError> {
    let source = std::str::from_utf8(bytes)
        .map_err(|_| HostError::schema("adapter response is not valid UTF-8"))?;
    adapter::parse_response(source).map_err(|errors| {
        let detail = errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        if errors
            .iter()
            .any(|error| error.detail.contains("trailing content"))
        {
            HostError::semantic(format!("adapter wrote extra response content: {detail}"))
        } else {
            HostError::schema(format!("adapter response schema is invalid: {detail}"))
        }
    })
}

fn explicit_failure(response: &adapter::AdapterResponse) -> HostError {
    match &response.failure {
        Some(failure) => HostError::semantic(format!(
            "adapter reported `{}`: {}",
            failure.code, failure.message
        )),
        None => HostError::semantic("adapter reported failure without details"),
    }
}

fn join_findings(findings: &[run::Finding]) -> String {
    findings
        .iter()
        .map(|finding| format!("{}: {}", finding.code, finding.detail))
        .collect::<Vec<_>>()
        .join("; ")
}

fn join_schema_errors(errors: &[run::SchemaError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

fn valid_kebab_segment(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
