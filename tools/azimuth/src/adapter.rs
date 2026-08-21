//! Strict adapter configuration and identity kernel (D47).
//!
//! This module deliberately owns no process lifecycle. It turns one explicit configuration file
//! into content-verified, typed adapter descriptions that the planner and bounded host can use.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

pub const CONFIGURATION_FORMAT: &str = "azimuth-adapter-configuration";
pub const DESCRIPTION_FORMAT: &str = "azimuth-adapter-description";
pub const PROTOCOL_VERSION: u64 = 1;
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    pub path: String,
    pub detail: String,
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConfiguration {
    pub path: PathBuf,
    pub directory: PathBuf,
    pub adapters: Vec<ConfiguredAdapter>,
}

impl AdapterConfiguration {
    pub fn adapter(&self, id: &str) -> Option<&ConfiguredAdapter> {
        self.adapters.iter().find(|adapter| adapter.id == id)
    }

    pub fn capability(&self, address: &str) -> Option<(&ConfiguredAdapter, &Capability)> {
        let (adapter_id, capability_id) = parse_address(address)?;
        let adapter = self.adapter(adapter_id)?;
        adapter
            .capabilities
            .iter()
            .find(|capability| capability.id == capability_id)
            .map(|capability| (adapter, capability))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredAdapter {
    pub id: String,
    pub provider_family: String,
    pub protocol_version: u64,
    pub adapter_version: String,
    pub build: String,
    pub content: AdapterContent,
    pub semantic_settings: BTreeMap<String, String>,
    pub environment: AdapterEnvironment,
    pub limits: AdapterLimits,
    pub capabilities: Vec<Capability>,
    pub adapter_fingerprint: String,
    pub descriptor_fingerprint: String,
    pub configuration_fingerprint: String,
}

impl ConfiguredAdapter {
    pub fn capability(&self, address: &str) -> Option<&Capability> {
        let (adapter_id, capability_id) = parse_address(address)?;
        if adapter_id != self.id {
            return None;
        }
        self.capabilities
            .iter()
            .find(|capability| capability.id == capability_id)
    }

    pub fn expected_description(&self) -> AdapterDescription {
        AdapterDescription {
            protocol_version: self.protocol_version,
            id: self.id.clone(),
            provider_family: self.provider_family.clone(),
            adapter_version: self.adapter_version.clone(),
            build: self.build.clone(),
            content: DescriptionContent {
                executable_digest: self.content.executable.digest.clone(),
                resources: self
                    .content
                    .resources
                    .iter()
                    .map(|resource| ContentIdentity {
                        id: resource.id.clone(),
                        digest: resource.digest.clone(),
                    })
                    .collect(),
            },
            adapter_fingerprint: self.adapter_fingerprint.clone(),
            capabilities: self.capabilities.clone(),
            descriptor_fingerprint: self.descriptor_fingerprint.clone(),
        }
    }

    /// Returns the complete environment for a child whose ambient environment was cleared.
    pub fn child_environment(&self) -> BTreeMap<String, String> {
        self.environment.literals.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterContent {
    pub executable: ConfiguredFile,
    pub resources: Vec<ConfiguredResource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredFile {
    pub locator: String,
    pub resolved: PathBuf,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredResource {
    pub id: String,
    pub locator: String,
    pub resolved: PathBuf,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterEnvironment {
    pub literals: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedAdapterContent {
    pub executable: StagedContentFile,
    pub resources: Vec<StagedResource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedContentFile {
    pub path: PathBuf,
    pub digest: String,
    pub size_bytes: u64,
}

impl StagedContentFile {
    pub fn input_identity(&self, id: &str) -> Result<InputIdentity, String> {
        if !valid_path_id(id) {
            return Err("input id is not a lower-kebab path id".into());
        }
        Ok(InputIdentity {
            id: id.to_string(),
            digest: self.digest.clone(),
            size_bytes: self.size_bytes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedResource {
    pub id: String,
    pub path: PathBuf,
    pub digest: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterLimits {
    pub timeout_ms: u64,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pub id: String,
    pub classes: Vec<CapabilityClass>,
    pub challenge_forms: Vec<String>,
    pub semantic_settings: BTreeMap<String, String>,
    pub fingerprint: String,
}

impl Capability {
    pub fn address(&self, adapter_id: &str) -> String {
        format!("{adapter_id}/{}", self.id)
    }

    pub fn supports(&self, class: CapabilityClass) -> bool {
        self.classes.contains(&class)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityClass {
    ModelExtract,
    CheckExecute,
    CheckImport,
    ChallengeExecute,
    ChallengeImport,
}

impl CapabilityClass {
    pub const ALL: [Self; 5] = [
        Self::ModelExtract,
        Self::CheckExecute,
        Self::CheckImport,
        Self::ChallengeExecute,
        Self::ChallengeImport,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::ModelExtract => "model.extract",
            Self::CheckExecute => "check.execute",
            Self::CheckImport => "check.import",
            Self::ChallengeExecute => "challenge.execute",
            Self::ChallengeImport => "challenge.import",
        }
    }

    pub fn is_challenge(self) -> bool {
        matches!(self, Self::ChallengeExecute | Self::ChallengeImport)
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|class| class.name() == value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterDescription {
    pub protocol_version: u64,
    pub id: String,
    pub provider_family: String,
    pub adapter_version: String,
    pub build: String,
    pub content: DescriptionContent,
    pub adapter_fingerprint: String,
    pub capabilities: Vec<Capability>,
    pub descriptor_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptionContent {
    pub executable_digest: String,
    pub resources: Vec<ContentIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentIdentity {
    pub id: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterOperation {
    Describe,
    Execute,
    Import,
}

impl AdapterOperation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Describe => "describe",
            Self::Execute => "execute",
            Self::Import => "import",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputIdentity {
    pub id: String,
    pub digest: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredecessorIdentity {
    pub bundle_revision: u64,
    pub bundle_fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterResponseStatus {
    Ok,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterFailure {
    pub code: String,
    pub message: String,
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterResponse {
    pub request_id: String,
    pub operation: AdapterOperation,
    pub status: AdapterResponseStatus,
    pub description: AdapterDescription,
    pub description_json: String,
    pub launch_fingerprint: Option<String>,
    pub bundle_json: Option<String>,
    pub failure: Option<AdapterFailure>,
}

pub fn load_configuration(path: &Path) -> Result<AdapterConfiguration, Vec<SchemaError>> {
    let source = fs::read_to_string(path).map_err(|error| {
        vec![SchemaError {
            path: path.display().to_string(),
            detail: error.to_string(),
        }]
    })?;
    parse_configuration(path, &source)
}

pub fn parse_configuration(
    path: &Path,
    source: &str,
) -> Result<AdapterConfiguration, Vec<SchemaError>> {
    parse_configuration_inner(path, source).map_err(|detail| {
        vec![SchemaError {
            path: path.display().to_string(),
            detail,
        }]
    })
}

fn parse_configuration_inner(path: &Path, source: &str) -> Result<AdapterConfiguration, String> {
    let root = StrictJson::parse(source)?;
    reject_duplicate_keys(&root, "$".into())?;
    let fields = object(&root, "$", &["format", "version", "adapters"])?;
    exact_string(fields, "format", "$", CONFIGURATION_FORMAT)?;
    exact_integer(fields, "version", "$", PROTOCOL_VERSION)?;

    let configured_directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let directory = configured_directory.canonicalize().map_err(|error| {
        format!(
            "configuration directory `{}` cannot be resolved: {error}",
            configured_directory.display()
        )
    })?;
    let items = array(required(fields, "adapters", "$")?, "$.adapters")?;
    let mut adapters = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        adapters.push(parse_adapter(
            item,
            &format!("$.adapters[{index}]"),
            &directory,
        )?);
    }
    ensure_sorted_unique(&adapters, |adapter| adapter.id.clone(), "$.adapters")?;
    Ok(AdapterConfiguration {
        path: path.to_path_buf(),
        directory,
        adapters,
    })
}

fn parse_adapter(
    value: &Json,
    where_: &str,
    directory: &Path,
) -> Result<ConfiguredAdapter, String> {
    let fields = object(
        value,
        where_,
        &[
            "id",
            "provider_family",
            "protocol_version",
            "adapter_version",
            "build",
            "content",
            "semantic_settings",
            "environment",
            "limits",
            "capabilities",
            "adapter_fingerprint",
            "descriptor_fingerprint",
            "configuration_fingerprint",
        ],
    )?;
    let id = segment(fields, "id", where_)?;
    let provider_family = path_id(fields, "provider_family", where_)?;
    let protocol_version = integer(fields, "protocol_version", where_)?;
    if protocol_version != PROTOCOL_VERSION {
        return Err(format!("{where_}.protocol_version must be `1`"));
    }
    let adapter_version = nonempty(fields, "adapter_version", where_)?;
    let build = nonempty(fields, "build", where_)?;
    let content = parse_content(
        required(fields, "content", where_)?,
        &format!("{where_}.content"),
        directory,
    )?;
    let semantic_settings = string_map(
        required(fields, "semantic_settings", where_)?,
        &format!("{where_}.semantic_settings"),
        false,
    )?;
    let environment = parse_environment(
        required(fields, "environment", where_)?,
        &format!("{where_}.environment"),
    )?;
    let limits = parse_limits(
        required(fields, "limits", where_)?,
        &format!("{where_}.limits"),
    )?;

    let capability_values = array(
        required(fields, "capabilities", where_)?,
        &format!("{where_}.capabilities"),
    )?;
    let supplied_adapter_fingerprint = fingerprint(fields, "adapter_fingerprint", where_)?;
    let mut capabilities = Vec::with_capacity(capability_values.len());
    for (index, capability) in capability_values.iter().enumerate() {
        capabilities.push(parse_capability(
            capability,
            &format!("{where_}.capabilities[{index}]"),
            &supplied_adapter_fingerprint,
        )?);
    }
    ensure_sorted_unique(
        &capabilities,
        |capability| capability.id.clone(),
        &format!("{where_}.capabilities"),
    )?;

    let adapter_fingerprint = adapter_fingerprint_from(
        &id,
        &provider_family,
        protocol_version,
        &adapter_version,
        &build,
        &content,
    );
    require_equal_fingerprint(
        &supplied_adapter_fingerprint,
        &adapter_fingerprint,
        &format!("{where_}.adapter_fingerprint"),
    )?;

    // Capability fingerprints were provisionally checked using the supplied adapter fingerprint;
    // the equality above closes the chain to verified content.
    let supplied_descriptor_fingerprint = fingerprint(fields, "descriptor_fingerprint", where_)?;
    let description_without_fingerprint = AdapterDescription {
        protocol_version,
        id: id.clone(),
        provider_family: provider_family.clone(),
        adapter_version: adapter_version.clone(),
        build: build.clone(),
        content: DescriptionContent {
            executable_digest: content.executable.digest.clone(),
            resources: content
                .resources
                .iter()
                .map(|resource| ContentIdentity {
                    id: resource.id.clone(),
                    digest: resource.digest.clone(),
                })
                .collect(),
        },
        adapter_fingerprint: adapter_fingerprint.clone(),
        capabilities: capabilities.clone(),
        descriptor_fingerprint: String::new(),
    };
    let descriptor_fingerprint = descriptor_fingerprint_from(&description_without_fingerprint);
    require_equal_fingerprint(
        &supplied_descriptor_fingerprint,
        &descriptor_fingerprint,
        &format!("{where_}.descriptor_fingerprint"),
    )?;

    let supplied_configuration_fingerprint =
        fingerprint(fields, "configuration_fingerprint", where_)?;
    let configuration_fingerprint = configuration_fingerprint_from(
        &adapter_fingerprint,
        &descriptor_fingerprint,
        &semantic_settings,
        &environment,
        &limits,
        &capabilities,
    );
    require_equal_fingerprint(
        &supplied_configuration_fingerprint,
        &configuration_fingerprint,
        &format!("{where_}.configuration_fingerprint"),
    )?;

    Ok(ConfiguredAdapter {
        id,
        provider_family,
        protocol_version,
        adapter_version,
        build,
        content,
        semantic_settings,
        environment,
        limits,
        capabilities,
        adapter_fingerprint,
        descriptor_fingerprint,
        configuration_fingerprint,
    })
}

fn parse_content(value: &Json, where_: &str, directory: &Path) -> Result<AdapterContent, String> {
    let fields = object(value, where_, &["executable", "resources"])?;
    let executable_where = format!("{where_}.executable");
    let executable_fields = object(
        required(fields, "executable", where_)?,
        &executable_where,
        &["locator", "digest"],
    )?;
    let executable_locator = nonempty(executable_fields, "locator", &executable_where)?;
    let executable_digest = fingerprint(executable_fields, "digest", &executable_where)?;
    let executable_resolved = resolve_content(
        directory,
        &executable_locator,
        &format!("{executable_where}.locator"),
    )?;

    let resources_where = format!("{where_}.resources");
    let resource_values = array(required(fields, "resources", where_)?, &resources_where)?;
    let mut resources = Vec::with_capacity(resource_values.len());
    for (index, resource) in resource_values.iter().enumerate() {
        let item_where = format!("{resources_where}[{index}]");
        let resource_fields = object(resource, &item_where, &["id", "locator", "digest"])?;
        let locator = nonempty(resource_fields, "locator", &item_where)?;
        let digest = fingerprint(resource_fields, "digest", &item_where)?;
        let resolved = resolve_content(directory, &locator, &format!("{item_where}.locator"))?;
        resources.push(ConfiguredResource {
            id: path_id(resource_fields, "id", &item_where)?,
            locator,
            resolved,
            digest,
        });
    }
    ensure_sorted_unique(&resources, |resource| resource.id.clone(), &resources_where)?;
    Ok(AdapterContent {
        executable: ConfiguredFile {
            locator: executable_locator,
            resolved: executable_resolved,
            digest: executable_digest,
        },
        resources,
    })
}

fn parse_environment(value: &Json, where_: &str) -> Result<AdapterEnvironment, String> {
    let fields = object(value, where_, &["literals"])?;
    let literals = string_map(
        required(fields, "literals", where_)?,
        &format!("{where_}.literals"),
        true,
    )?;
    Ok(AdapterEnvironment { literals })
}

fn parse_limits(value: &Json, where_: &str) -> Result<AdapterLimits, String> {
    let fields = object(
        value,
        where_,
        &["timeout_ms", "stdout_bytes", "stderr_bytes"],
    )?;
    let timeout_ms = positive_integer(fields, "timeout_ms", where_)?;
    let stdout_bytes = positive_integer(fields, "stdout_bytes", where_)?;
    let stderr_bytes = positive_integer(fields, "stderr_bytes", where_)?;
    Ok(AdapterLimits {
        timeout_ms,
        stdout_bytes,
        stderr_bytes,
    })
}

fn parse_capability(
    value: &Json,
    where_: &str,
    adapter_fingerprint: &str,
) -> Result<Capability, String> {
    let fields = object(
        value,
        where_,
        &[
            "id",
            "classes",
            "challenge_forms",
            "semantic_settings",
            "fingerprint",
        ],
    )?;
    let id = segment(fields, "id", where_)?;
    let class_values = array(
        required(fields, "classes", where_)?,
        &format!("{where_}.classes"),
    )?;
    if class_values.is_empty() {
        return Err(format!("{where_}.classes must not be empty"));
    }
    let mut classes = Vec::with_capacity(class_values.len());
    for (index, value) in class_values.iter().enumerate() {
        let value = value
            .as_str()
            .ok_or_else(|| format!("{where_}.classes[{index}] must be a string"))?;
        classes.push(
            CapabilityClass::parse(value).ok_or_else(|| {
                format!("{where_}.classes[{index}] has unsupported value `{value}`")
            })?,
        );
    }
    ensure_sorted_unique(&classes, |class| class.name(), &format!("{where_}.classes"))?;

    let form_values = array(
        required(fields, "challenge_forms", where_)?,
        &format!("{where_}.challenge_forms"),
    )?;
    let mut challenge_forms = Vec::with_capacity(form_values.len());
    for (index, value) in form_values.iter().enumerate() {
        let value = value
            .as_str()
            .ok_or_else(|| format!("{where_}.challenge_forms[{index}] must be a string"))?;
        if !valid_path_id(value) {
            return Err(format!(
                "{where_}.challenge_forms[{index}] is not a lower-kebab path id"
            ));
        }
        challenge_forms.push(value.to_string());
    }
    ensure_sorted_unique(
        &challenge_forms,
        Clone::clone,
        &format!("{where_}.challenge_forms"),
    )?;
    let has_challenge = classes.iter().any(|class| class.is_challenge());
    if has_challenge != !challenge_forms.is_empty() {
        return Err(format!(
            "{where_}.challenge_forms must be non-empty exactly when a challenge class is present"
        ));
    }
    let semantic_settings = string_map(
        required(fields, "semantic_settings", where_)?,
        &format!("{where_}.semantic_settings"),
        false,
    )?;
    let supplied_fingerprint = fingerprint(fields, "fingerprint", where_)?;
    let derived_fingerprint = capability_fingerprint_from(
        adapter_fingerprint,
        &id,
        &classes,
        &challenge_forms,
        &semantic_settings,
    );
    require_equal_fingerprint(
        &supplied_fingerprint,
        &derived_fingerprint,
        &format!("{where_}.fingerprint"),
    )?;
    Ok(Capability {
        id,
        classes,
        challenge_forms,
        semantic_settings,
        fingerprint: derived_fingerprint,
    })
}

pub fn parse_description(source: &str) -> Result<AdapterDescription, Vec<SchemaError>> {
    parse_description_inner(source).map_err(|detail| {
        vec![SchemaError {
            path: "adapter response description".into(),
            detail,
        }]
    })
}

fn parse_description_inner(source: &str) -> Result<AdapterDescription, String> {
    let root = StrictJson::parse(source)?;
    reject_duplicate_keys(&root, "$".into())?;
    parse_description_value(&root, "$")
}

fn parse_description_value(value: &Json, where_: &str) -> Result<AdapterDescription, String> {
    let fields = object(
        value,
        where_,
        &[
            "format",
            "version",
            "protocol_version",
            "id",
            "provider_family",
            "adapter_version",
            "build",
            "content",
            "adapter_fingerprint",
            "capabilities",
            "descriptor_fingerprint",
        ],
    )?;
    exact_string(fields, "format", where_, DESCRIPTION_FORMAT)?;
    exact_integer(fields, "version", where_, PROTOCOL_VERSION)?;
    exact_integer(fields, "protocol_version", where_, PROTOCOL_VERSION)?;
    let id = segment(fields, "id", where_)?;
    let provider_family = path_id(fields, "provider_family", where_)?;
    let adapter_version = nonempty(fields, "adapter_version", where_)?;
    let build = nonempty(fields, "build", where_)?;
    let content_where = format!("{where_}.content");
    let content_fields = object(
        required(fields, "content", where_)?,
        &content_where,
        &["executable_digest", "resources"],
    )?;
    let executable_digest = fingerprint(content_fields, "executable_digest", &content_where)?;
    let resource_values = array(
        required(content_fields, "resources", &content_where)?,
        &format!("{content_where}.resources"),
    )?;
    let mut resources = Vec::with_capacity(resource_values.len());
    for (index, resource) in resource_values.iter().enumerate() {
        let item_where = format!("{content_where}.resources[{index}]");
        let item = object(resource, &item_where, &["id", "digest"])?;
        resources.push(ContentIdentity {
            id: path_id(item, "id", &item_where)?,
            digest: fingerprint(item, "digest", &item_where)?,
        });
    }
    ensure_sorted_unique(
        &resources,
        |item| item.id.clone(),
        &format!("{content_where}.resources"),
    )?;
    let adapter_fingerprint = fingerprint(fields, "adapter_fingerprint", where_)?;
    let capability_values = array(
        required(fields, "capabilities", where_)?,
        &format!("{where_}.capabilities"),
    )?;
    let mut capabilities = Vec::with_capacity(capability_values.len());
    for (index, capability) in capability_values.iter().enumerate() {
        capabilities.push(parse_capability(
            capability,
            &format!("{where_}.capabilities[{index}]"),
            &adapter_fingerprint,
        )?);
    }
    ensure_sorted_unique(
        &capabilities,
        |capability| capability.id.clone(),
        &format!("{where_}.capabilities"),
    )?;

    let derived_adapter_fingerprint = adapter_fingerprint_from_identities(
        &id,
        &provider_family,
        PROTOCOL_VERSION,
        &adapter_version,
        &build,
        &executable_digest,
        &resources,
    );
    require_equal_fingerprint(
        &adapter_fingerprint,
        &derived_adapter_fingerprint,
        &format!("{where_}.adapter_fingerprint"),
    )?;
    let supplied_descriptor = fingerprint(fields, "descriptor_fingerprint", where_)?;
    let mut description = AdapterDescription {
        protocol_version: PROTOCOL_VERSION,
        id,
        provider_family,
        adapter_version,
        build,
        content: DescriptionContent {
            executable_digest,
            resources,
        },
        adapter_fingerprint,
        capabilities,
        descriptor_fingerprint: String::new(),
    };
    let derived_descriptor = descriptor_fingerprint_from(&description);
    require_equal_fingerprint(
        &supplied_descriptor,
        &derived_descriptor,
        &format!("{where_}.descriptor_fingerprint"),
    )?;
    description.descriptor_fingerprint = derived_descriptor;
    Ok(description)
}

pub fn validate_description(
    adapter: &ConfiguredAdapter,
    actual: &AdapterDescription,
) -> Result<(), String> {
    let expected = adapter.expected_description();
    if &expected == actual {
        Ok(())
    } else {
        Err(format!(
            "adapter `{}` description differs from configured descriptor {}",
            adapter.id, adapter.descriptor_fingerprint
        ))
    }
}

/// Parses the strict transport envelope without interpreting provider-neutral Run semantics.
///
/// `bundle_json`, when present, is RFC 8785 JSON for handoff to the Run parser. The bounded host
/// remains responsible for request/launch equality and complete bundle validation.
pub fn parse_response(source: &str) -> Result<AdapterResponse, Vec<SchemaError>> {
    parse_response_inner(source).map_err(|detail| {
        vec![SchemaError {
            path: "adapter response".into(),
            detail,
        }]
    })
}

fn parse_response_inner(source: &str) -> Result<AdapterResponse, String> {
    let root = StrictJson::parse(source)?;
    reject_duplicate_keys(&root, "$".into())?;
    let fields = object(
        &root,
        "$",
        &[
            "format",
            "version",
            "request_id",
            "operation",
            "status",
            "description",
            "launch_fingerprint",
            "bundle",
            "failure",
        ],
    )?;
    exact_string(fields, "format", "$", "azimuth-adapter-response")?;
    exact_integer(fields, "version", "$", PROTOCOL_VERSION)?;
    let request_id = fingerprint(fields, "request_id", "$")?;
    let operation = match string(fields, "operation", "$")? {
        "describe" => AdapterOperation::Describe,
        "execute" => AdapterOperation::Execute,
        "import" => AdapterOperation::Import,
        other => return Err(format!("$.operation has unsupported value `{other}`")),
    };
    let status = match string(fields, "status", "$")? {
        "ok" => AdapterResponseStatus::Ok,
        "failed" => AdapterResponseStatus::Failed,
        other => return Err(format!("$.status has unsupported value `{other}`")),
    };
    let description_value = required(fields, "description", "$")?;
    let description = parse_description_value(description_value, "$.description")?;
    let mut description_json = String::new();
    write_jcs(&description_json_value(&description), &mut description_json);

    let launch_fingerprint = match fields.iter().find(|(key, _)| key == "launch_fingerprint") {
        None => None,
        Some((_, Json::Str(value))) if valid_fingerprint(value) => Some(value.clone()),
        Some(_) => return Err("$.launch_fingerprint has invalid shape".into()),
    };
    if operation == AdapterOperation::Describe && launch_fingerprint.is_some() {
        return Err("$.launch_fingerprint is forbidden for describe".into());
    }
    if operation != AdapterOperation::Describe && launch_fingerprint.is_none() {
        return Err("$.launch_fingerprint is required for execute and import".into());
    }

    let bundle_value = fields
        .iter()
        .find(|(key, _)| key == "bundle")
        .map(|(_, value)| value);
    let failure_value = fields
        .iter()
        .find(|(key, _)| key == "failure")
        .map(|(_, value)| value);
    let (bundle_json, failure) = match status {
        AdapterResponseStatus::Ok if operation == AdapterOperation::Describe => {
            if bundle_value.is_some() || failure_value.is_some() {
                return Err("successful describe forbids bundle and failure".into());
            }
            (None, None)
        }
        AdapterResponseStatus::Ok => {
            if failure_value.is_some() {
                return Err("successful execute or import forbids failure".into());
            }
            let bundle = bundle_value
                .ok_or_else(|| "successful execute or import requires a bundle".to_string())?;
            if !matches!(bundle, Json::Obj(_)) {
                return Err("$.bundle must be an object".into());
            }
            let mut canonical = String::new();
            write_jcs(bundle, &mut canonical);
            (Some(canonical), None)
        }
        AdapterResponseStatus::Failed => {
            if bundle_value.is_some() {
                return Err("failed response forbids bundle".into());
            }
            let failure_value =
                failure_value.ok_or_else(|| "failed response requires failure".to_string())?;
            let failure_where = "$.failure";
            let failure_fields = object(
                failure_value,
                failure_where,
                &["code", "message", "details"],
            )?;
            let failure = AdapterFailure {
                code: path_id(failure_fields, "code", failure_where)?,
                message: nonempty(failure_fields, "message", failure_where)?,
                details: string_map(
                    required(failure_fields, "details", failure_where)?,
                    "$.failure.details",
                    false,
                )?,
            };
            (None, Some(failure))
        }
    };
    Ok(AdapterResponse {
        request_id,
        operation,
        status,
        description,
        description_json,
        launch_fingerprint,
        bundle_json,
        failure,
    })
}

fn description_json_value(description: &AdapterDescription) -> Json {
    description_json(description, true)
}

pub fn describe_request_fingerprint(
    adapter_id: &str,
    configuration_fingerprint: &str,
) -> Result<String, String> {
    if !valid_segment(adapter_id) {
        return Err("adapter id is not a lower-kebab segment".into());
    }
    if !valid_fingerprint(configuration_fingerprint) {
        return Err("configuration fingerprint has invalid shape".into());
    }
    Ok(jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-adapter-request-fingerprint")),
        ("version", Json::Num(PROTOCOL_VERSION)),
        ("operation", Json::str("describe")),
        (
            "adapter",
            Json::obj(vec![
                ("id", Json::str(adapter_id)),
                (
                    "configuration_fingerprint",
                    Json::str(configuration_fingerprint),
                ),
            ]),
        ),
    ])))
}

pub fn run_request_fingerprint(
    operation: AdapterOperation,
    launch_fingerprint: &str,
    inputs: &[InputIdentity],
    predecessors: &[PredecessorIdentity],
) -> Result<String, String> {
    if operation == AdapterOperation::Describe {
        return Err("a Run request operation must be `execute` or `import`".into());
    }
    if !valid_fingerprint(launch_fingerprint) {
        return Err("launch fingerprint has invalid shape".into());
    }
    ensure_sorted_unique(inputs, |input| input.id.clone(), "inputs")?;
    if operation == AdapterOperation::Execute && !inputs.is_empty() {
        return Err("execute request inputs must be empty".into());
    }
    if operation == AdapterOperation::Import && inputs.is_empty() {
        return Err("import request inputs must be non-empty".into());
    }
    for (index, input) in inputs.iter().enumerate() {
        if !valid_path_id(&input.id) {
            return Err(format!("inputs[{index}].id is not a lower-kebab path id"));
        }
        if !valid_fingerprint(&input.digest) {
            return Err(format!("inputs[{index}].digest has invalid shape"));
        }
        if input.size_bytes > MAX_SAFE_INTEGER {
            return Err(format!(
                "inputs[{index}].size_bytes exceeds the safe-integer limit"
            ));
        }
    }
    for (index, predecessor) in predecessors.iter().enumerate() {
        if predecessor.bundle_revision != index as u64 {
            return Err(format!(
                "predecessors[{index}].bundle_revision must be `{index}`"
            ));
        }
        if !valid_fingerprint(&predecessor.bundle_fingerprint) {
            return Err(format!(
                "predecessors[{index}].bundle_fingerprint has invalid shape"
            ));
        }
    }
    Ok(jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-adapter-request-fingerprint")),
        ("version", Json::Num(PROTOCOL_VERSION)),
        ("operation", Json::str(operation.name())),
        ("launch_fingerprint", Json::str(launch_fingerprint)),
        (
            "inputs",
            Json::Arr(
                inputs
                    .iter()
                    .map(|input| {
                        Json::obj(vec![
                            ("id", Json::str(&input.id)),
                            ("digest", Json::str(&input.digest)),
                            ("size_bytes", Json::Num(input.size_bytes)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "predecessors",
            Json::Arr(
                predecessors
                    .iter()
                    .map(|predecessor| {
                        Json::obj(vec![
                            ("bundle_revision", Json::Num(predecessor.bundle_revision)),
                            (
                                "bundle_fingerprint",
                                Json::str(&predecessor.bundle_fingerprint),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])))
}

/// Computes a point-in-time input identity from one open handle.
///
/// Invocation code must use [`stage_file`] instead so the bytes made available to an adapter are
/// the same bytes that were identified.
pub fn identify_input(id: &str, path: &Path) -> Result<InputIdentity, String> {
    if !valid_path_id(id) {
        return Err("input id is not a lower-kebab path id".into());
    }
    let mut file = fs::File::open(path)
        .map_err(|error| format!("input `{}` cannot be read: {error}", path.display()))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("input `{}` cannot be inspected: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("input `{}` is not a regular file", path.display()));
    }
    let mut hasher = Sha256::new();
    let mut size_bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("input `{}` cannot be read: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        size_bytes = size_bytes
            .checked_add(count as u64)
            .ok_or_else(|| "input byte size overflowed".to_string())?;
        if size_bytes > MAX_SAFE_INTEGER {
            return Err(format!(
                "input `{}` exceeds the safe-integer byte-size limit",
                path.display()
            ));
        }
        hasher.update(&buffer[..count]);
    }
    Ok(InputIdentity {
        id: id.to_string(),
        digest: format!("sha256:{}", hasher.finish()),
        size_bytes,
    })
}

/// Copies configured content into a caller-owned private directory while hashing the exact bytes
/// copied from each single-open source handle.
///
/// The returned paths, rather than the configured locators, are the only paths a bounded host
/// should execute or expose to the adapter. Destination creation is exclusive, and staged files
/// become read-only after their configured digest has been verified. This provides integrity
/// isolation for invoked bytes; it is not a process or filesystem sandbox.
pub fn stage_content(
    adapter: &ConfiguredAdapter,
    staging_directory: &Path,
) -> Result<StagedAdapterContent, String> {
    let metadata = fs::metadata(staging_directory).map_err(|error| {
        format!(
            "staging directory `{}` cannot be inspected: {error}",
            staging_directory.display()
        )
    })?;
    if !metadata.is_dir() {
        return Err(format!(
            "staging directory `{}` is not a directory",
            staging_directory.display()
        ));
    }
    let directory = staging_directory.canonicalize().map_err(|error| {
        format!(
            "staging directory `{}` cannot be resolved: {error}",
            staging_directory.display()
        )
    })?;
    let executable_path = directory.join("adapter-executable");
    let mut created = Vec::new();
    let result = (|| {
        let executable = stage_file(
            &adapter.content.executable.resolved,
            &executable_path,
            Some(&adapter.content.executable.digest),
            None,
            true,
        )?;
        created.push(executable_path.clone());
        let mut resources = Vec::with_capacity(adapter.content.resources.len());
        for (index, resource) in adapter.content.resources.iter().enumerate() {
            let path = directory.join(format!("adapter-resource-{index:04}"));
            let staged = stage_file(
                &resource.resolved,
                &path,
                Some(&resource.digest),
                None,
                false,
            )?;
            created.push(path.clone());
            resources.push(StagedResource {
                id: resource.id.clone(),
                path: staged.path,
                digest: staged.digest,
                size_bytes: staged.size_bytes,
            });
        }
        Ok(StagedAdapterContent {
            executable,
            resources,
        })
    })();
    if result.is_err() {
        for path in created {
            let _ = fs::remove_file(path);
        }
    }
    result
}

/// Copies one regular file from a single-open source handle while deriving the digest and size of
/// the exact staged byte stream. Configured content supplies an expected digest; import callers may
/// omit expectations and use the returned identity in the adapter request.
pub fn stage_file(
    source: &Path,
    destination: &Path,
    expected_digest: Option<&str>,
    expected_size: Option<u64>,
    executable: bool,
) -> Result<StagedContentFile, String> {
    if expected_digest.is_some_and(|digest| !valid_fingerprint(digest)) {
        return Err("expected staged-content digest has invalid shape".into());
    }
    if expected_size.is_some_and(|size| size > MAX_SAFE_INTEGER) {
        return Err("expected staged-content size exceeds the safe-integer limit".into());
    }
    let mut input = fs::File::open(source).map_err(|error| {
        format!(
            "configured content `{}` cannot be opened: {error}",
            source.display()
        )
    })?;
    if !input
        .metadata()
        .map_err(|error| {
            format!(
                "configured content `{}` cannot be inspected: {error}",
                source.display()
            )
        })?
        .is_file()
    {
        return Err(format!(
            "configured content `{}` is not a regular file",
            source.display()
        ));
    }
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| {
            format!(
                "staged content `{}` cannot be created: {error}",
                destination.display()
            )
        })?;
    let copied = (|| -> Result<(String, u64), String> {
        let mut hasher = Sha256::new();
        let mut size_bytes = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = input.read(&mut buffer).map_err(|error| {
                format!(
                    "configured content `{}` cannot be read: {error}",
                    source.display()
                )
            })?;
            if count == 0 {
                break;
            }
            size_bytes = size_bytes
                .checked_add(count as u64)
                .ok_or_else(|| "staged content byte size overflowed".to_string())?;
            if size_bytes > MAX_SAFE_INTEGER {
                return Err(format!(
                    "configured content `{}` exceeds the safe-integer byte-size limit",
                    source.display()
                ));
            }
            hasher.update(&buffer[..count]);
            output.write_all(&buffer[..count]).map_err(|error| {
                format!(
                    "staged content `{}` cannot be written: {error}",
                    destination.display()
                )
            })?;
        }
        output.flush().map_err(|error| {
            format!(
                "staged content `{}` cannot be flushed: {error}",
                destination.display()
            )
        })?;
        Ok((format!("sha256:{}", hasher.finish()), size_bytes))
    })();
    drop(output);
    let (actual, size_bytes) = match copied {
        Ok(copied) => copied,
        Err(error) => {
            let _ = fs::remove_file(destination);
            return Err(error);
        }
    };
    if let Some(expected) = expected_digest {
        if actual != expected {
            let _ = fs::remove_file(destination);
            return Err(format!(
                concat!(
                    "configured content `{}` digest mismatch: supplied `{}`, ",
                    "derived `{}`"
                ),
                source.display(),
                expected,
                actual
            ));
        }
    }
    if let Some(expected) = expected_size {
        if expected != size_bytes {
            let _ = fs::remove_file(destination);
            return Err(format!(
                concat!(
                    "configured content `{}` size mismatch: supplied `{}`, ",
                    "derived `{}`"
                ),
                source.display(),
                expected,
                size_bytes
            ));
        }
    }
    if let Err(error) = set_staged_permissions(destination, executable) {
        let _ = fs::remove_file(destination);
        return Err(error);
    }
    Ok(StagedContentFile {
        path: destination.to_path_buf(),
        digest: actual,
        size_bytes,
    })
}

#[cfg(unix)]
fn set_staged_permissions(path: &Path, executable: bool) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if executable { 0o500 } else { 0o400 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|error| {
        format!(
            "staged content `{}` permissions cannot be restricted: {error}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
fn set_staged_permissions(path: &Path, _executable: bool) -> Result<(), String> {
    let mut permissions = fs::metadata(path)
        .map_err(|error| {
            format!(
                "staged content `{}` cannot be inspected: {error}",
                path.display()
            )
        })?
        .permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions).map_err(|error| {
        format!(
            "staged content `{}` permissions cannot be restricted: {error}",
            path.display()
        )
    })
}

fn resolve_content(directory: &Path, locator: &str, where_: &str) -> Result<PathBuf, String> {
    let configured = Path::new(locator);
    let candidate = if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        if locator.contains('\\')
            || configured.components().any(|component| {
                matches!(
                    component,
                    Component::CurDir | Component::ParentDir | Component::RootDir
                )
            })
            || locator.split('/').any(str::is_empty)
        {
            return Err(format!("{where_} is not a normalized relative path"));
        }
        directory.join(configured)
    };
    let resolved = candidate
        .canonicalize()
        .map_err(|error| format!("{where_} cannot be resolved: {error}"))?;
    if !configured.is_absolute() && !resolved.starts_with(directory) {
        return Err(format!("{where_} escapes the configuration directory"));
    }
    let metadata = fs::metadata(&resolved)
        .map_err(|error| format!("{where_} cannot be inspected: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{where_} must resolve to a regular file"));
    }
    Ok(resolved)
}

pub fn adapter_fingerprint(adapter: &ConfiguredAdapter) -> String {
    adapter_fingerprint_from(
        &adapter.id,
        &adapter.provider_family,
        adapter.protocol_version,
        &adapter.adapter_version,
        &adapter.build,
        &adapter.content,
    )
}

fn adapter_fingerprint_from(
    id: &str,
    provider_family: &str,
    protocol_version: u64,
    adapter_version: &str,
    build: &str,
    content: &AdapterContent,
) -> String {
    let resources = content
        .resources
        .iter()
        .map(|resource| ContentIdentity {
            id: resource.id.clone(),
            digest: resource.digest.clone(),
        })
        .collect::<Vec<_>>();
    adapter_fingerprint_from_identities(
        id,
        provider_family,
        protocol_version,
        adapter_version,
        build,
        &content.executable.digest,
        &resources,
    )
}

fn adapter_fingerprint_from_identities(
    id: &str,
    provider_family: &str,
    protocol_version: u64,
    adapter_version: &str,
    build: &str,
    executable_digest: &str,
    resources: &[ContentIdentity],
) -> String {
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-adapter-fingerprint")),
        ("version", Json::Num(PROTOCOL_VERSION)),
        ("protocol_version", Json::Num(protocol_version)),
        ("id", Json::str(id)),
        ("provider_family", Json::str(provider_family)),
        ("adapter_version", Json::str(adapter_version)),
        ("build", Json::str(build)),
        (
            "content",
            Json::obj(vec![
                ("executable_digest", Json::str(executable_digest)),
                (
                    "resources",
                    Json::Arr(
                        resources
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
    ]))
}

pub fn capability_fingerprint(adapter_fingerprint: &str, capability: &Capability) -> String {
    capability_fingerprint_from(
        adapter_fingerprint,
        &capability.id,
        &capability.classes,
        &capability.challenge_forms,
        &capability.semantic_settings,
    )
}

fn capability_fingerprint_from(
    adapter_fingerprint: &str,
    id: &str,
    classes: &[CapabilityClass],
    challenge_forms: &[String],
    semantic_settings: &BTreeMap<String, String>,
) -> String {
    jcs_sha256(&Json::obj(vec![
        (
            "format",
            Json::str("azimuth-adapter-capability-fingerprint"),
        ),
        ("version", Json::Num(PROTOCOL_VERSION)),
        ("adapter_fingerprint", Json::str(adapter_fingerprint)),
        ("id", Json::str(id)),
        (
            "classes",
            Json::Arr(
                classes
                    .iter()
                    .map(|class| Json::str(class.name()))
                    .collect(),
            ),
        ),
        (
            "challenge_forms",
            Json::Arr(challenge_forms.iter().map(Json::str).collect()),
        ),
        ("semantic_settings", map_json(semantic_settings)),
    ]))
}

pub fn descriptor_fingerprint(description: &AdapterDescription) -> String {
    descriptor_fingerprint_from(description)
}

fn descriptor_fingerprint_from(description: &AdapterDescription) -> String {
    jcs_sha256(&Json::obj(vec![
        (
            "format",
            Json::str("azimuth-adapter-descriptor-fingerprint"),
        ),
        ("version", Json::Num(PROTOCOL_VERSION)),
        ("descriptor", description_json(description, false)),
    ]))
}

pub fn configuration_fingerprint(adapter: &ConfiguredAdapter) -> String {
    configuration_fingerprint_from(
        &adapter.adapter_fingerprint,
        &adapter.descriptor_fingerprint,
        &adapter.semantic_settings,
        &adapter.environment,
        &adapter.limits,
        &adapter.capabilities,
    )
}

fn configuration_fingerprint_from(
    adapter_fingerprint: &str,
    descriptor_fingerprint: &str,
    semantic_settings: &BTreeMap<String, String>,
    environment: &AdapterEnvironment,
    limits: &AdapterLimits,
    capabilities: &[Capability],
) -> String {
    jcs_sha256(&Json::obj(vec![
        (
            "format",
            Json::str("azimuth-adapter-configuration-fingerprint"),
        ),
        ("version", Json::Num(PROTOCOL_VERSION)),
        ("adapter_fingerprint", Json::str(adapter_fingerprint)),
        ("descriptor_fingerprint", Json::str(descriptor_fingerprint)),
        ("semantic_settings", map_json(semantic_settings)),
        (
            "environment",
            Json::obj(vec![("literals", map_json(&environment.literals))]),
        ),
        (
            "limits",
            Json::obj(vec![
                ("timeout_ms", Json::Num(limits.timeout_ms)),
                ("stdout_bytes", Json::Num(limits.stdout_bytes)),
                ("stderr_bytes", Json::Num(limits.stderr_bytes)),
            ]),
        ),
        (
            "capabilities",
            Json::Arr(capabilities.iter().map(capability_json).collect()),
        ),
    ]))
}

fn description_json(description: &AdapterDescription, include_fingerprint: bool) -> Json {
    let mut fields = vec![
        ("format", Json::str(DESCRIPTION_FORMAT)),
        ("version", Json::Num(PROTOCOL_VERSION)),
        ("protocol_version", Json::Num(description.protocol_version)),
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
            Json::str(&description.adapter_fingerprint),
        ),
        (
            "capabilities",
            Json::Arr(
                description
                    .capabilities
                    .iter()
                    .map(capability_json)
                    .collect(),
            ),
        ),
    ];
    if include_fingerprint {
        fields.push((
            "descriptor_fingerprint",
            Json::str(&description.descriptor_fingerprint),
        ));
    }
    Json::obj(fields)
}

fn capability_json(capability: &Capability) -> Json {
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
            Json::Arr(capability.challenge_forms.iter().map(Json::str).collect()),
        ),
        ("semantic_settings", map_json(&capability.semantic_settings)),
        ("fingerprint", Json::str(&capability.fingerprint)),
    ])
}

fn map_json(map: &BTreeMap<String, String>) -> Json {
    Json::Obj(
        map.iter()
            .map(|(key, value)| (key.clone(), Json::str(value)))
            .collect(),
    )
}

fn require_equal_fingerprint(supplied: &str, derived: &str, where_: &str) -> Result<(), String> {
    if supplied == derived {
        Ok(())
    } else {
        Err(format!(
            "{where_} mismatch: supplied `{supplied}`, derived `{derived}`"
        ))
    }
}

fn parse_address(value: &str) -> Option<(&str, &str)> {
    let (adapter, capability) = value.split_once('/')?;
    if capability.contains('/') || !valid_segment(adapter) || !valid_segment(capability) {
        return None;
    }
    Some((adapter, capability))
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value.as_bytes()[0] != b'-'
        && value.as_bytes()[value.len() - 1] != b'-'
        && !value.contains("--")
}

fn valid_path_id(value: &str) -> bool {
    !value.is_empty() && value.split('/').all(valid_segment)
}

fn valid_environment_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(
        bytes.next(),
        Some(b'_') | Some(b'A'..=b'Z') | Some(b'a'..=b'z')
    ) && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn object<'a>(
    value: &'a Json,
    where_: &str,
    allowed: &[&str],
) -> Result<&'a [(String, Json)], String> {
    let fields = match value {
        Json::Obj(fields) => fields.as_slice(),
        _ => return Err(format!("{where_} must be an object")),
    };
    for (key, _) in fields {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{where_} has unknown field `{key}`"));
        }
    }
    Ok(fields)
}

fn required<'a>(
    fields: &'a [(String, Json)],
    field: &str,
    where_: &str,
) -> Result<&'a Json, String> {
    fields
        .iter()
        .find(|(key, _)| key == field)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("{where_} is missing `{field}`"))
}

fn array<'a>(value: &'a Json, where_: &str) -> Result<&'a [Json], String> {
    match value {
        Json::Arr(items) => Ok(items),
        _ => Err(format!("{where_} must be an array")),
    }
}

fn string<'a>(fields: &'a [(String, Json)], field: &str, where_: &str) -> Result<&'a str, String> {
    required(fields, field, where_)?
        .as_str()
        .ok_or_else(|| format!("{where_}.{field} must be a string"))
}

fn exact_string(
    fields: &[(String, Json)],
    field: &str,
    where_: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = string(fields, field, where_)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{where_}.{field} must be `{expected}`"))
    }
}

fn nonempty(fields: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = string(fields, field, where_)?;
    if value.is_empty() {
        Err(format!("{where_}.{field} must not be empty"))
    } else {
        Ok(value.to_string())
    }
}

fn segment(fields: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = nonempty(fields, field, where_)?;
    if valid_segment(&value) {
        Ok(value)
    } else {
        Err(format!("{where_}.{field} is not a lower-kebab segment"))
    }
}

fn path_id(fields: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = nonempty(fields, field, where_)?;
    if valid_path_id(&value) {
        Ok(value)
    } else {
        Err(format!("{where_}.{field} is not a lower-kebab path id"))
    }
}

fn fingerprint(fields: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = nonempty(fields, field, where_)?;
    if valid_fingerprint(&value) {
        Ok(value)
    } else {
        Err(format!(
            "{where_}.{field} must be `sha256:` followed by 64 lowercase hex digits"
        ))
    }
}

fn integer(fields: &[(String, Json)], field: &str, where_: &str) -> Result<u64, String> {
    match required(fields, field, where_)? {
        Json::Num(value) if *value <= MAX_SAFE_INTEGER => Ok(*value),
        _ => Err(format!(
            "{where_}.{field} must be a non-negative safe integer"
        )),
    }
}

fn positive_integer(fields: &[(String, Json)], field: &str, where_: &str) -> Result<u64, String> {
    let value = integer(fields, field, where_)?;
    if value == 0 {
        Err(format!("{where_}.{field} must be positive"))
    } else {
        Ok(value)
    }
}

fn exact_integer(
    fields: &[(String, Json)],
    field: &str,
    where_: &str,
    expected: u64,
) -> Result<(), String> {
    let actual = integer(fields, field, where_)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{where_}.{field} must be `{expected}`"))
    }
}

fn string_map(
    value: &Json,
    where_: &str,
    environment_names: bool,
) -> Result<BTreeMap<String, String>, String> {
    let fields = match value {
        Json::Obj(fields) => fields,
        _ => return Err(format!("{where_} must be an object")),
    };
    let mut map = BTreeMap::new();
    for (key, value) in fields {
        if key.is_empty() {
            return Err(format!("{where_} contains an empty key"));
        }
        if environment_names && !valid_environment_name(key) {
            return Err(format!(
                "{where_} contains invalid environment name `{key}`"
            ));
        }
        let value = value
            .as_str()
            .ok_or_else(|| format!("{where_}.{key} must be a string"))?;
        map.insert(key.clone(), value.to_string());
    }
    Ok(map)
}

fn ensure_sorted_unique<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> K,
    where_: &str,
) -> Result<(), String> {
    for pair in values.windows(2) {
        match key(&pair[0]).cmp(&key(&pair[1])) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => {
                return Err(format!("{where_} contains a duplicate identity"))
            }
            std::cmp::Ordering::Greater => {
                return Err(format!("{where_} must be sorted canonically"))
            }
        }
    }
    Ok(())
}

fn reject_duplicate_keys(value: &Json, where_: String) -> Result<(), String> {
    match value {
        Json::Obj(fields) => {
            let mut seen = BTreeSet::new();
            for (key, value) in fields {
                if !seen.insert(key) {
                    return Err(format!("{where_} contains duplicate field `{key}`"));
                }
                reject_duplicate_keys(value, format!("{where_}.{key}"))?;
            }
        }
        Json::Arr(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_duplicate_keys(item, format!("{where_}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Json {
    Null,
    Bool(bool),
    Num(u64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn obj(fields: Vec<(&str, Json)>) -> Self {
        Self::Obj(
            fields
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        )
    }

    fn str(value: impl Into<String>) -> Self {
        Self::Str(value.into())
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value),
            _ => None,
        }
    }
}

struct StrictJson<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> StrictJson<'a> {
    fn parse(source: &'a str) -> Result<Json, String> {
        let mut parser = Self {
            bytes: source.as_bytes(),
            position: 0,
        };
        parser.skip_whitespace();
        let value = parser.value()?;
        parser.skip_whitespace();
        if parser.position != parser.bytes.len() {
            return Err(parser.error("trailing content after the top-level value"));
        }
        Ok(value)
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'\"') => self.string().map(Json::Str),
            Some(b't') => self.literal(b"true", Json::Bool(true)),
            Some(b'f') => self.literal(b"false", Json::Bool(false)),
            Some(b'n') => self.literal(b"null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number().map(Json::Num),
            Some(byte) => Err(self.error(&format!("unexpected `{}`", byte as char))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect(b'{')?;
        let mut fields = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Json::Obj(fields));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'\"') {
                return Err(self.error("object keys must be strings"));
            }
            let key = self.string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.value()?;
            fields.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Json::Obj(fields));
                }
                _ => return Err(self.error("expected `,` or `}`")),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b']') => {
                    self.position += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(self.error("expected `,` or `]`")),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'\"')?;
        let mut value = String::new();
        while let Some(byte) = self.peek() {
            match byte {
                b'\"' => {
                    self.position += 1;
                    return Ok(value);
                }
                b'\\' => {
                    self.position += 1;
                    let escape = self
                        .peek()
                        .ok_or_else(|| self.error("unterminated string escape"))?;
                    self.position += 1;
                    match escape {
                        b'\"' => value.push('\"'),
                        b'\\' => value.push('\\'),
                        b'/' => value.push('/'),
                        b'b' => value.push('\u{08}'),
                        b'f' => value.push('\u{0c}'),
                        b'n' => value.push('\n'),
                        b'r' => value.push('\r'),
                        b't' => value.push('\t'),
                        b'u' => {
                            let first = self.hex_code_unit()?;
                            if (0xd800..=0xdbff).contains(&first) {
                                if self.peek() != Some(b'\\')
                                    || self.bytes.get(self.position + 1) != Some(&b'u')
                                {
                                    return Err(
                                        self.error("high surrogate requires a low surrogate")
                                    );
                                }
                                self.position += 2;
                                let second = self.hex_code_unit()?;
                                if !(0xdc00..=0xdfff).contains(&second) {
                                    return Err(
                                        self.error("high surrogate requires a low surrogate")
                                    );
                                }
                                let scalar = 0x10000
                                    + (((first as u32) - 0xd800) << 10)
                                    + ((second as u32) - 0xdc00);
                                value.push(char::from_u32(scalar).unwrap());
                            } else if (0xdc00..=0xdfff).contains(&first) {
                                return Err(self.error("unpaired low surrogate"));
                            } else {
                                value.push(char::from_u32(first as u32).unwrap());
                            }
                        }
                        _ => return Err(self.error("invalid string escape")),
                    }
                }
                0x00..=0x1f => return Err(self.error("unescaped control character in string")),
                _ => {
                    let rest = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| self.error("invalid UTF-8 in string"))?;
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.error("unterminated string"))?;
                    value.push(character);
                    self.position += character.len_utf8();
                }
            }
        }
        Err(self.error("unterminated string"))
    }

    fn hex_code_unit(&mut self) -> Result<u16, String> {
        let bytes = self
            .bytes
            .get(self.position..self.position + 4)
            .ok_or_else(|| self.error("truncated Unicode escape"))?;
        let text = std::str::from_utf8(bytes).map_err(|_| self.error("invalid Unicode escape"))?;
        let value =
            u16::from_str_radix(text, 16).map_err(|_| self.error("invalid Unicode escape"))?;
        self.position += 4;
        Ok(value)
    }

    fn number(&mut self) -> Result<u64, String> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            return Err(self.error("numbers must be non-negative safe integers"));
        }
        if self.peek() == Some(b'0') {
            self.position += 1;
            if matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("numbers cannot contain leading zeroes"));
            }
        } else {
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            let fraction_start = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == fraction_start {
                return Err(self.error("number fraction requires digits"));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if self.peek() == Some(b'-') {
                self.position += 1;
            } else {
                if self.peek() == Some(b'+') {
                    self.position += 1;
                }
            }
            let exponent_start = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == exponent_start {
                return Err(self.error("number exponent requires digits"));
            }
        }
        let number = std::str::from_utf8(&self.bytes[start..self.position]).unwrap();
        decimal_safe_integer(number).map_err(|detail| self.error(detail))
    }

    fn literal(&mut self, expected: &[u8], value: Json) -> Result<Json, String> {
        if self.bytes[self.position..].starts_with(expected) {
            self.position += expected.len();
            Ok(value)
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(self.error(&format!("expected `{}`", expected as char)))
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }

    fn error(&self, detail: &str) -> String {
        let line = 1 + self.bytes[..self.position.min(self.bytes.len())]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        format!("line {line}: {detail}")
    }
}

fn decimal_safe_integer(number: &str) -> Result<u64, &'static str> {
    let (mantissa, exponent_text) = number
        .split_once(['e', 'E'])
        .map_or((number, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    let (whole, fraction) = mantissa
        .split_once('.')
        .map_or((mantissa, ""), |(whole, fraction)| (whole, fraction));
    let digits = format!("{whole}{fraction}");
    if digits.bytes().all(|byte| byte == b'0') {
        return Ok(0);
    }
    let exponent = match exponent_text {
        None => 0i64,
        Some(text) => text
            .parse::<i64>()
            .map_err(|_| "number exceeds the safe-integer limit")?,
    };
    let scale = exponent - fraction.len() as i64;
    let significant = digits.trim_start_matches('0');
    let integer_digits = if scale >= 0 {
        if significant.len() as i64 + scale > 16 {
            return Err("number exceeds the safe-integer limit");
        }
        format!("{significant}{}", "0".repeat(scale as usize))
    } else {
        let removed = (-scale) as usize;
        if removed > digits.len() {
            return Err("numbers must be integral");
        }
        let split = digits.len() - removed;
        if digits.as_bytes()[split..].iter().any(|byte| *byte != b'0') {
            return Err("numbers must be integral");
        }
        let retained = digits[..split].trim_start_matches('0');
        if retained.is_empty() {
            "0".to_string()
        } else {
            retained.to_string()
        }
    };
    let value = integer_digits
        .parse::<u64>()
        .map_err(|_| "number exceeds the safe-integer limit")?;
    if value > MAX_SAFE_INTEGER {
        Err("number exceeds the safe-integer limit")
    } else {
        Ok(value)
    }
}

fn jcs_sha256(value: &Json) -> String {
    let mut canonical = String::new();
    write_jcs(value, &mut canonical);
    format!("sha256:{}", sha256(canonical.as_bytes()))
}

fn write_jcs(value: &Json, output: &mut String) {
    match value {
        Json::Null => output.push_str("null"),
        Json::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Json::Num(value) => output.push_str(&value.to_string()),
        Json::Str(value) => write_jcs_string(value, output),
        Json::Arr(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_jcs(value, output);
            }
            output.push(']');
        }
        Json::Obj(fields) => {
            let mut fields = fields.iter().collect::<Vec<_>>();
            fields.sort_by(|left, right| left.0.encode_utf16().cmp(right.0.encode_utf16()));
            output.push('{');
            for (index, (key, value)) in fields.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_jcs_string(key, output);
                output.push(':');
                write_jcs(value, output);
            }
            output.push('}');
        }
    }
}

fn write_jcs_string(value: &str, output: &mut String) {
    use std::fmt::Write as _;
    output.push('\"');
    for character in value.chars() {
        match character {
            '\"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{09}' => output.push_str("\\t"),
            '\u{0a}' => output.push_str("\\n"),
            '\u{0c}' => output.push_str("\\f"),
            '\u{0d}' => output.push_str("\\r"),
            character if character <= '\u{1f}' => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
    output.push('\"');
}

const SHA256_INITIAL: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];
const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

struct Sha256 {
    state: [u32; 8],
    tail: Vec<u8>,
    byte_length: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: SHA256_INITIAL,
            tail: Vec::new(),
            byte_length: 0,
        }
    }

    fn update(&mut self, input: &[u8]) {
        self.byte_length = self.byte_length.wrapping_add(input.len() as u64);
        self.tail.extend_from_slice(input);
        let complete_length = self.tail.len() / 64 * 64;
        for block in self.tail[..complete_length].chunks_exact(64) {
            sha256_block(&mut self.state, block);
        }
        self.tail.drain(..complete_length);
    }

    fn finish(mut self) -> String {
        let bit_length = self.byte_length.wrapping_mul(8);
        self.tail.push(0x80);
        while self.tail.len() % 64 != 56 {
            self.tail.push(0);
        }
        self.tail.extend_from_slice(&bit_length.to_be_bytes());
        for block in self.tail.chunks_exact(64) {
            sha256_block(&mut self.state, block);
        }
        self.state
            .iter()
            .map(|word| format!("{word:08x}"))
            .collect()
    }
}

fn sha256_block(state: &mut [u32; 8], block: &[u8]) {
    let mut words = [0u32; 64];
    for (index, bytes) in block.chunks_exact(4).enumerate() {
        words[index] = u32::from_be_bytes(bytes.try_into().unwrap());
    }
    for index in 16..64 {
        let s0 = words[index - 15].rotate_right(7)
            ^ words[index - 15].rotate_right(18)
            ^ (words[index - 15] >> 3);
        let s1 = words[index - 2].rotate_right(17)
            ^ words[index - 2].rotate_right(19)
            ^ (words[index - 2] >> 10);
        words[index] = words[index - 16]
            .wrapping_add(s0)
            .wrapping_add(words[index - 7])
            .wrapping_add(s1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for index in 0..64 {
        let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ (!e & g);
        let first = h
            .wrapping_add(sum1)
            .wrapping_add(choose)
            .wrapping_add(SHA256_K[index])
            .wrapping_add(words[index]);
        let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let second = sum0.wrapping_add(majority);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(first);
        d = c;
        c = b;
        b = a;
        a = first.wrapping_add(second);
    }
    for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *slot = slot.wrapping_add(value);
    }
}

// Dependency-free SHA-256 keeps adapter identity available in isolated test harnesses and avoids
// making provider invocation depend on a platform binary.
fn sha256(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finish()
}
