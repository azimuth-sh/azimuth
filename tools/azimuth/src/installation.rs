//! Consumer installation ownership, component registration, resource synchronization and migration.

use crate::fingerprint::{canonical_sha256, sha256};
use crate::json::{self, Json};
use crate::resources;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component as PathComponent, Path, PathBuf};

const INSTALLATION_FORMAT: &str = "azimuth-installation";
const INSTALLATION_SCHEMA: u32 = 1;

#[derive(Clone, Debug)]
pub struct ManagedResource {
    pub id: String,
    pub path: String,
    pub digest: String,
}

#[derive(Clone, Debug)]
pub struct RegisteredComponent {
    pub id: String,
    pub manifest: String,
    pub version: String,
}

#[derive(Clone, Debug)]
pub struct AdoptedAlias {
    pub integration: String,
    pub path: String,
    pub target: String,
}

#[derive(Clone, Debug)]
pub struct Installation {
    pub release_version: String,
    pub migration_line: String,
    pub agents: Vec<String>,
    pub components: Vec<RegisteredComponent>,
    pub resources: Vec<ManagedResource>,
    pub aliases: Vec<AdoptedAlias>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateMode {
    Check,
    DryRun,
    Apply,
}

#[derive(Clone, Debug)]
struct DesiredResource {
    id: String,
    path: String,
    content: &'static str,
}

struct TransactionState {
    path: PathBuf,
    stage: Option<PathBuf>,
    backup: Option<PathBuf>,
    published: bool,
}

pub fn initialize(
    root: &Path,
    agents: &[String],
    adopt_alias: bool,
) -> Result<Vec<PathBuf>, String> {
    let agents = normalize_agents(agents)?;
    let repository = repository_root(root)?;
    let manifest_path = root.join("installation.json");
    if manifest_path.exists() {
        return Err(format!(
            "{} already exists; use `azimuth update` or `azimuth agent`",
            manifest_path.display()
        ));
    }
    let aliases = discover_aliases(&repository, &agents, adopt_alias)?;
    let desired = desired_resources(&agents, &aliases);
    preflight_new(&repository, &desired)?;

    let mut created = crate::workflow::initialize(root)?;
    let mut managed = Vec::new();
    for item in desired {
        let destination = repository.join(&item.path);
        if destination.exists() || fs::symlink_metadata(&destination).is_ok() {
            return Err(format!(
                "managed resource destination already exists: {}",
                destination.display()
            ));
        }
        write_new(&destination, item.content.as_bytes())?;
        created.push(destination);
        managed.push(ManagedResource {
            id: item.id,
            path: item.path,
            digest: sha256(item.content.as_bytes()),
        });
    }
    let installation = Installation {
        release_version: env!("CARGO_PKG_VERSION").into(),
        migration_line: resources::MIGRATION_LINE.into(),
        agents,
        components: Vec::new(),
        resources: managed,
        aliases,
    };
    write_installation(root, &installation)?;
    created.push(manifest_path);
    Ok(created)
}

pub fn load(root: &Path) -> Result<Installation, String> {
    let path = root.join("installation.json");
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let value = json::parse(&source).map_err(|error| format!("{}: {error}", path.display()))?;
    require_string(&value, "format", &path).and_then(|value| {
        if value == INSTALLATION_FORMAT {
            Ok(())
        } else {
            Err(format!("{}: unsupported format `{value}`", path.display()))
        }
    })?;
    let schema = require_number(&value, "schemaVersion", &path)?;
    if schema != INSTALLATION_SCHEMA {
        return Err(format!(
            "{}: unsupported schemaVersion `{schema}`",
            path.display()
        ));
    }
    let agents = string_array(&value, "agents", &path)?;
    let components = object_array(&value, "components", &path)?
        .iter()
        .map(|item| {
            let manifest = require_string(item, "manifest", &path)?;
            require_safe_relative(manifest, "component manifest", &path)?;
            Ok(RegisteredComponent {
                id: require_string(item, "id", &path)?.into(),
                manifest: manifest.into(),
                version: require_string(item, "version", &path)?.into(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let managed = object_array(&value, "resources", &path)?
        .iter()
        .map(|item| {
            let managed_path = require_string(item, "path", &path)?;
            require_safe_relative(managed_path, "managed resource path", &path)?;
            Ok(ManagedResource {
                id: require_string(item, "id", &path)?.into(),
                path: managed_path.into(),
                digest: require_string(item, "sha256", &path)?.into(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let aliases = object_array(&value, "aliases", &path)?
        .iter()
        .map(|item| {
            let alias_path = require_string(item, "path", &path)?;
            require_safe_relative(alias_path, "alias path", &path)?;
            Ok(AdoptedAlias {
                integration: require_string(item, "integration", &path)?.into(),
                path: alias_path.into(),
                target: require_string(item, "target", &path)?.into(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let installation = Installation {
        release_version: require_string(&value, "releaseVersion", &path)?.into(),
        migration_line: require_string(&value, "migrationLine", &path)?.into(),
        agents,
        components,
        resources: managed,
        aliases,
    };
    validate_installation(&installation, &path)?;
    Ok(installation)
}

pub fn agent_add(root: &Path, agent: &str) -> Result<Vec<PathBuf>, String> {
    if agent == "none" {
        return Err("`none` is only valid for explicit initialization".into());
    }
    let normalized = normalize_agents(&[agent.to_string()])?;
    let agent = &normalized[0];
    let mut installation = load(root)?;
    require_running_release(&installation)?;
    if installation.agents.iter().any(|item| item == agent) {
        return Err(format!("agent integration `{agent}` is already installed"));
    }
    verify_managed(root, &installation)?;
    verify_aliases(root, &installation)?;
    let repository = repository_root(root)?;
    let desired = desired_resources(&[agent.clone()], &[])
        .into_iter()
        .filter(|item| item.id != "orientation")
        .collect::<Vec<_>>();
    preflight_new(&repository, &desired)?;
    let mut created = Vec::new();
    let mut changes = Vec::new();
    for item in desired {
        let destination = repository.join(&item.path);
        created.push(destination.clone());
        changes.push((destination, Some(item.content.as_bytes().to_vec())));
        installation.resources.push(ManagedResource {
            id: item.id,
            path: item.path,
            digest: sha256(item.content.as_bytes()),
        });
    }
    installation.agents.push(agent.clone());
    installation.agents.sort();
    installation
        .resources
        .sort_by(|left, right| left.path.cmp(&right.path));
    changes.push((
        root.join("installation.json"),
        Some(installation_json(&installation).into_bytes()),
    ));
    apply_transaction(changes)?;
    Ok(created)
}

pub fn agent_remove(root: &Path, agent: &str) -> Result<Vec<PathBuf>, String> {
    if agent == "none" {
        return Err("`none` is only valid for explicit initialization".into());
    }
    let normalized = normalize_agents(&[agent.to_string()])?;
    let agent = &normalized[0];
    let mut installation = load(root)?;
    require_running_release(&installation)?;
    if !installation.agents.iter().any(|item| item == agent) {
        return Err(format!("agent integration `{agent}` is not installed"));
    }
    if agent == "codex"
        && installation
            .aliases
            .iter()
            .any(|item| item.integration == "claude")
    {
        return Err(
            "cannot remove codex while the adopted claude alias targets its skills; remove claude first"
                .into(),
        );
    }
    verify_managed(root, &installation)?;
    verify_aliases(root, &installation)?;
    let prefix = format!("skill:{agent}:");
    let metadata_prefix = format!("skill-metadata:{agent}:");
    let repository = repository_root(root)?;
    let removed = installation
        .resources
        .iter()
        .filter(|item| item.id.starts_with(&prefix) || item.id.starts_with(&metadata_prefix))
        .map(|item| repository.join(&item.path))
        .collect::<Vec<_>>();
    installation
        .resources
        .retain(|item| !item.id.starts_with(&prefix) && !item.id.starts_with(&metadata_prefix));
    installation.agents.retain(|item| item != agent);
    installation
        .aliases
        .retain(|item| item.integration != *agent);
    let mut changes = removed
        .iter()
        .cloned()
        .map(|path| (path, None))
        .collect::<Vec<_>>();
    changes.push((
        root.join("installation.json"),
        Some(installation_json(&installation).into_bytes()),
    ));
    apply_transaction(changes)?;
    Ok(removed)
}

pub fn component_add(root: &Path, id: &str, manifest: &Path) -> Result<(), String> {
    let identity = component_identity(id).ok_or_else(|| format!("unsupported component `{id}`"))?;
    let mut installation = load(root)?;
    require_running_release(&installation)?;
    verify_managed(root, &installation)?;
    verify_aliases(root, &installation)?;
    if installation.components.iter().any(|item| item.id == id) {
        return Err(format!("component `{id}` is already registered"));
    }
    verify_component_manifest(id, identity, manifest)?;
    let repository = repository_root(root)?;
    let manifest = repository_relative(&repository, manifest)?;
    installation.components.push(RegisteredComponent {
        id: id.into(),
        manifest,
        version: env!("CARGO_PKG_VERSION").into(),
    });
    installation
        .components
        .sort_by(|left, right| left.id.cmp(&right.id));
    write_installation(root, &installation)
}

pub fn component_remove(root: &Path, id: &str) -> Result<(), String> {
    let mut installation = load(root)?;
    require_running_release(&installation)?;
    verify_managed(root, &installation)?;
    verify_aliases(root, &installation)?;
    let previous = installation.components.len();
    installation.components.retain(|item| item.id != id);
    if previous == installation.components.len() {
        return Err(format!("component `{id}` is not registered"));
    }
    write_installation(root, &installation)
}

pub fn update(root: &Path, mode: UpdateMode) -> Result<Vec<String>, String> {
    let mut installation = load(root)?;
    verify_managed(root, &installation)?;
    verify_aliases(root, &installation)?;
    verify_components(root, &installation)?;
    if installation.migration_line != resources::MIGRATION_LINE {
        return Err(format!(
            "installation migration line `{}` is incompatible with running CLI `{}`; use `azimuth migrate`",
            installation.migration_line,
            resources::MIGRATION_LINE
        ));
    }
    let repository = repository_root(root)?;
    let desired = desired_resources(&installation.agents, &installation.aliases);
    let old_by_path = installation
        .resources
        .iter()
        .map(|item| (item.path.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    let desired_paths = desired
        .iter()
        .map(|item| item.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut actions = Vec::new();
    if installation.release_version != env!("CARGO_PKG_VERSION") {
        actions.push(format!(
            "record release {} -> {}",
            installation.release_version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    for component in &installation.components {
        if component.version != env!("CARGO_PKG_VERSION") {
            actions.push(format!(
                "record component {} release {} -> {}",
                component.id,
                component.version,
                env!("CARGO_PKG_VERSION")
            ));
        }
    }
    for item in &desired {
        let digest = sha256(item.content.as_bytes());
        match old_by_path.get(item.path.as_str()) {
            Some(old) if old.digest == digest => {}
            Some(_) => actions.push(format!("update {}", item.path)),
            None => {
                let path = repository.join(&item.path);
                if fs::symlink_metadata(&path).is_ok() {
                    return Err(format!(
                        "unmanaged destination blocks update: {}",
                        path.display()
                    ));
                }
                actions.push(format!("create {}", item.path));
            }
        }
    }
    for old in &installation.resources {
        if !desired_paths.contains(old.path.as_str()) {
            actions.push(format!("remove {}", old.path));
        }
    }
    if mode == UpdateMode::Apply && !actions.is_empty() {
        let mut changes = installation
            .resources
            .iter()
            .filter(|item| !desired_paths.contains(item.path.as_str()))
            .map(|item| (repository.join(&item.path), None))
            .collect::<Vec<_>>();
        changes.extend(desired.iter().map(|item| {
            (
                repository.join(&item.path),
                Some(item.content.as_bytes().to_vec()),
            )
        }));
        installation.release_version = env!("CARGO_PKG_VERSION").into();
        installation.migration_line = resources::MIGRATION_LINE.into();
        for component in &mut installation.components {
            component.version = env!("CARGO_PKG_VERSION").into();
        }
        installation.resources = desired
            .into_iter()
            .map(|item| ManagedResource {
                id: item.id,
                path: item.path,
                digest: sha256(item.content.as_bytes()),
            })
            .collect();
        changes.push((
            root.join("installation.json"),
            Some(installation_json(&installation).into_bytes()),
        ));
        apply_transaction(changes)?;
    }
    Ok(actions)
}

pub fn migration_plan(root: &Path) -> Result<Json, String> {
    let installation_path = root.join("installation.json");
    let source = fs::read_to_string(&installation_path)
        .map_err(|error| format!("cannot read {}: {error}", installation_path.display()))?;
    let installation = load(root)?;
    let repository = repository_root(root)?;
    let installation_relative = repository_relative(&repository, &installation_path)?;
    let findings = retired_syntax_findings(&repository)?;
    let supported =
        resources::migration_reference(&installation.release_version, env!("CARGO_PKG_VERSION"))
            .is_some()
            || installation.release_version == env!("CARGO_PKG_VERSION");
    let disposition = if !supported {
        "unsupported"
    } else if findings.is_empty() {
        "automatic"
    } else {
        "review-required"
    };
    let base = Json::obj(vec![
        ("format", Json::str("azimuth-migration-plan")),
        ("schemaVersion", Json::Num(1.0)),
        ("migrationLine", Json::str(resources::MIGRATION_LINE)),
        ("fromRelease", Json::str(&installation.release_version)),
        ("toRelease", Json::str(env!("CARGO_PKG_VERSION"))),
        ("installation", Json::str(installation_relative)),
        ("installationSha256", Json::str(sha256(source.as_bytes()))),
        ("disposition", Json::str(disposition)),
        ("edits", Json::Arr(Vec::new())),
        (
            "findings",
            Json::Arr(findings.into_iter().map(Json::str).collect()),
        ),
    ]);
    let fingerprint = canonical_sha256(&base);
    let Json::Obj(mut fields) = base else {
        unreachable!()
    };
    fields.push(("fingerprint".into(), Json::str(fingerprint)));
    Ok(Json::Obj(fields))
}

pub fn migration_apply(root: &Path, plan_path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(plan_path)
        .map_err(|error| format!("cannot read {}: {error}", plan_path.display()))?;
    let plan = json::parse(&source).map_err(|error| format!("{}: {error}", plan_path.display()))?;
    if require_string(&plan, "format", plan_path)? != "azimuth-migration-plan" {
        return Err(format!(
            "{}: expected azimuth-migration-plan",
            plan_path.display()
        ));
    }
    if require_number(&plan, "schemaVersion", plan_path)? != 1 {
        return Err(format!(
            "{}: unsupported migration plan schemaVersion",
            plan_path.display()
        ));
    }
    if require_string(&plan, "migrationLine", plan_path)? != resources::MIGRATION_LINE {
        return Err("migration plan belongs to a different migration line".into());
    }
    if require_string(&plan, "toRelease", plan_path)? != env!("CARGO_PKG_VERSION") {
        return Err("migration plan targets a different CLI release".into());
    }
    if require_string(&plan, "disposition", plan_path)? != "automatic" {
        return Err("migration plan is not automatic; resolve findings and replan".into());
    }
    let expected = require_string(&plan, "fingerprint", plan_path)?;
    let Json::Obj(fields) = &plan else {
        unreachable!()
    };
    let unsigned = Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "fingerprint")
            .cloned()
            .collect(),
    );
    if canonical_sha256(&unsigned) != expected {
        return Err("migration plan fingerprint does not match its content".into());
    }
    let installation_path = root.join("installation.json");
    let loaded = load(root)?;
    let repository = repository_root(root)?;
    if require_string(&plan, "installation", plan_path)?
        != repository_relative(&repository, &installation_path)?
    {
        return Err("migration plan names a different installation account".into());
    }
    if require_string(&plan, "fromRelease", plan_path)? != loaded.release_version {
        return Err("migration plan source release differs from the installation".into());
    }
    let installation = fs::read(&installation_path)
        .map_err(|error| format!("cannot read {}: {error}", installation_path.display()))?;
    if sha256(&installation) != require_string(&plan, "installationSha256", plan_path)? {
        return Err("installation changed after migration planning; replan".into());
    }
    if !object_array(&plan, "edits", plan_path)?.is_empty() {
        return Err("this CLI cannot apply the plan's edit schema".into());
    }
    Ok(())
}

fn desired_resources(agents: &[String], aliases: &[AdoptedAlias]) -> Vec<DesiredResource> {
    let mut desired = vec![DesiredResource {
        id: "orientation".into(),
        path: "azimuth/README.md".into(),
        content: resources::PROJECT_README,
    }];
    for agent in agents {
        if aliases.iter().any(|item| item.integration == *agent) {
            continue;
        }
        let directory = match agent.as_str() {
            "codex" => ".agents/skills",
            "claude" => ".claude/skills",
            _ => continue,
        };
        for skill in resources::SKILLS {
            desired.push(DesiredResource {
                id: format!("skill:{agent}:{}", skill.id),
                path: format!("{directory}/{}/SKILL.md", skill.id),
                content: skill.source,
            });
            if agent == "codex" {
                desired.push(DesiredResource {
                    id: format!("skill-metadata:{agent}:{}", skill.id),
                    path: format!("{directory}/{}/agents/openai.yaml", skill.id),
                    content: skill.openai,
                });
            }
        }
    }
    desired.sort_by(|left, right| left.path.cmp(&right.path));
    desired
}

fn normalize_agents(agents: &[String]) -> Result<Vec<String>, String> {
    if agents.is_empty() {
        return Err(
            "explicit agent selection is required; use `--agents codex,claude` or `--agents none`"
                .into(),
        );
    }
    let mut normalized = BTreeSet::new();
    for agent in agents {
        match agent.as_str() {
            "none" if agents.len() == 1 => return Ok(Vec::new()),
            "none" => return Err("`none` cannot be combined with another agent".into()),
            "codex" | "claude" => {
                normalized.insert(agent.clone());
            }
            _ => {
                return Err(format!(
                    "unsupported agent integration `{agent}`; expected codex, claude or none"
                ))
            }
        }
    }
    Ok(normalized.into_iter().collect())
}

fn require_running_release(installation: &Installation) -> Result<(), String> {
    if installation.release_version == env!("CARGO_PKG_VERSION")
        && installation.migration_line == resources::MIGRATION_LINE
    {
        Ok(())
    } else {
        Err(format!(
            "installation is release `{}` on migration line `{}`; run `azimuth update` with CLI `{}` before changing registrations",
            installation.release_version,
            installation.migration_line,
            env!("CARGO_PKG_VERSION")
        ))
    }
}

fn validate_installation(installation: &Installation, path: &Path) -> Result<(), String> {
    if installation.release_version.is_empty() || installation.migration_line.is_empty() {
        return Err(format!(
            "{}: releaseVersion and migrationLine must be non-empty",
            path.display()
        ));
    }
    let normalized = if installation.agents.is_empty() {
        Vec::new()
    } else {
        normalize_agents(&installation.agents)?
    };
    if normalized != installation.agents {
        return Err(format!(
            "{}: agents must be sorted and unique",
            path.display()
        ));
    }
    unique_fields(
        installation.components.iter().map(|item| item.id.as_str()),
        "component id",
        path,
    )?;
    unique_fields(
        installation
            .components
            .iter()
            .map(|item| item.manifest.as_str()),
        "component manifest",
        path,
    )?;
    for component in &installation.components {
        if component_identity(&component.id).is_none() || component.version.is_empty() {
            return Err(format!(
                "{}: unsupported or unversioned component `{}`",
                path.display(),
                component.id
            ));
        }
    }
    unique_fields(
        installation.resources.iter().map(|item| item.id.as_str()),
        "resource id",
        path,
    )?;
    unique_fields(
        installation.resources.iter().map(|item| item.path.as_str()),
        "resource path",
        path,
    )?;
    for resource in &installation.resources {
        if resource.digest.len() != 64
            || !resource
                .digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(format!(
                "{}: resource `{}` has invalid SHA-256",
                path.display(),
                resource.id
            ));
        }
    }
    unique_fields(
        installation.aliases.iter().map(|item| item.path.as_str()),
        "alias path",
        path,
    )?;
    unique_fields(
        installation
            .aliases
            .iter()
            .map(|item| item.integration.as_str()),
        "alias integration",
        path,
    )?;
    for alias in &installation.aliases {
        if alias.integration != "claude"
            || !installation
                .agents
                .iter()
                .any(|agent| agent == &alias.integration)
            || Path::new(&alias.target).is_absolute()
        {
            return Err(format!(
                "{}: unsupported alias for integration `{}`",
                path.display(),
                alias.integration
            ));
        }
    }
    Ok(())
}

fn unique_fields<'a>(
    values: impl Iterator<Item = &'a str>,
    field: &str,
    path: &Path,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.is_empty() || !seen.insert(value) {
            return Err(format!(
                "{}: `{field}` values must be non-empty and unique",
                path.display()
            ));
        }
    }
    Ok(())
}

fn verify_managed(root: &Path, installation: &Installation) -> Result<(), String> {
    let repository = repository_root(root)?;
    let mut conflicts = Vec::new();
    for item in &installation.resources {
        let path = repository.join(&item.path);
        match fs::read(&path) {
            Ok(content) if sha256(&content) == item.digest => {}
            Ok(_) => conflicts.push(format!("modified managed resource: {}", path.display())),
            Err(error) => conflicts.push(format!(
                "cannot read managed resource {}: {error}",
                path.display()
            )),
        }
    }
    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(conflicts.join("\n"))
    }
}

fn verify_aliases(root: &Path, installation: &Installation) -> Result<(), String> {
    let repository = repository_root(root)?;
    for alias in &installation.aliases {
        let path = repository.join(&alias.path);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("cannot inspect adopted alias {}: {error}", path.display()))?;
        if !metadata.file_type().is_symlink() {
            return Err(format!(
                "adopted alias is no longer a symlink: {}",
                path.display()
            ));
        }
        let target = fs::read_link(&path)
            .map_err(|error| format!("cannot read adopted alias {}: {error}", path.display()))?;
        if target.is_absolute() || target.to_string_lossy() != alias.target {
            return Err(format!("adopted alias target drifted: {}", path.display()));
        }
        let resolved = normalize_path(&path.parent().unwrap_or(&repository).join(target));
        if !resolved.starts_with(normalize_path(&repository)) {
            return Err(format!(
                "adopted alias escapes repository: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn verify_components(root: &Path, installation: &Installation) -> Result<(), String> {
    let repository = repository_root(root)?;
    for component in &installation.components {
        let identity = component_identity(&component.id)
            .ok_or_else(|| format!("unsupported registered component `{}`", component.id))?;
        verify_component_manifest(
            &component.id,
            identity,
            &repository.join(&component.manifest),
        )?;
    }
    Ok(())
}

fn component_identity(id: &str) -> Option<&'static str> {
    match id {
        "typescript-annotations" => Some("@azimuth-sh/annotations"),
        "typescript-emitter" => Some("@azimuth-sh/emit"),
        "dotnet-annotations" => Some("Azimuth.Annotations"),
        "dotnet-emitter" => Some("Azimuth.Emit"),
        _ => None,
    }
}

fn verify_component_manifest(id: &str, identity: &str, manifest: &Path) -> Result<(), String> {
    let source = fs::read_to_string(manifest).map_err(|error| {
        format!(
            "cannot read component manifest {}: {error}",
            manifest.display()
        )
    })?;
    let exact = match manifest.extension().and_then(|value| value.to_str()) {
        Some("json") => json::parse(&source)
            .ok()
            .is_some_and(|value| json_declares_component(&value, identity)),
        Some("csproj") => {
            let start = format!("<PackageReference Include=\"{identity}\"");
            source.find(&start).is_some_and(|offset| {
                let declaration = &source[offset..];
                let end = declaration
                    .find("</PackageReference>")
                    .map(|index| index + "</PackageReference>".len())
                    .or_else(|| declaration.find("/>").map(|index| index + 2))
                    .unwrap_or(0);
                end > 0 && declaration[..end].contains(env!("CARGO_PKG_VERSION"))
            })
        }
        _ => false,
    };
    if !exact {
        return Err(format!(
            "{} does not pin component `{id}` identity `{identity}` to exact release `{}`",
            manifest.display(),
            env!("CARGO_PKG_VERSION")
        ));
    }
    Ok(())
}

fn json_declares_component(value: &Json, identity: &str) -> bool {
    for collection in ["dependencies", "devDependencies", "peerDependencies"] {
        if value
            .get(collection)
            .and_then(|item| item.get(identity))
            .and_then(Json::as_str)
            == Some(env!("CARGO_PKG_VERSION"))
        {
            return true;
        }
    }
    value
        .get("tools")
        .and_then(|item| {
            item.get(identity)
                .or_else(|| item.get(&identity.to_ascii_lowercase()))
        })
        .and_then(|item| item.get("version"))
        .and_then(Json::as_str)
        == Some(env!("CARGO_PKG_VERSION"))
}

fn retired_syntax_findings(repository: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    collect_markdown(&repository.join("azimuth/model"), &mut files)?;
    collect_active_change_markdown(&repository.join("azimuth/changes"), &mut files)?;
    files.sort();
    let mut findings = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file)
            .map_err(|error| format!("cannot read {}: {error}", file.display()))?;
        for (index, line) in source.lines().enumerate() {
            if line.starts_with("## Requirement:") || line.starts_with("### Scenario:") {
                findings.push(format!(
                    "{}:{}: retired Requirement/Scenario syntax requires semantic review",
                    file.display(),
                    index + 1
                ));
            }
        }
    }
    Ok(findings)
}

fn collect_active_change_markdown(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("cannot read {}: {error}", root.display())),
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", root.display()))?;
        if entry.file_name() == "archive" {
            continue;
        }
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?
            .is_dir()
        {
            collect_markdown(&path, files)?;
        }
    }
    Ok(())
}

fn collect_markdown(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("cannot read {}: {error}", root.display())),
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", root.display()))?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
        if kind.is_dir() {
            collect_markdown(&path, files)?;
        } else if kind.is_file() && path.extension().and_then(|value| value.to_str()) == Some("md")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn repository_root(root: &Path) -> Result<PathBuf, String> {
    let root = if root.as_os_str().is_empty() {
        Path::new("azimuth")
    } else {
        root
    };
    root.parent()
        .map(|path| {
            if path.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                path.to_path_buf()
            }
        })
        .ok_or_else(|| format!("Azimuth root {} has no repository parent", root.display()))
}

fn repository_relative(repository: &Path, path: &Path) -> Result<String, String> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(path)
    };
    let repository = fs::canonicalize(repository).map_err(|error| {
        format!(
            "cannot resolve repository {}: {error}",
            repository.display()
        )
    })?;
    let path = fs::canonicalize(&path)
        .map_err(|error| format!("cannot resolve manifest {}: {error}", path.display()))?;
    let relative = path.strip_prefix(&repository).map_err(|_| {
        format!(
            "manifest {} is outside repository {}",
            path.display(),
            repository.display()
        )
    })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn preflight_new(repository: &Path, desired: &[DesiredResource]) -> Result<(), String> {
    let mut conflicts = Vec::new();
    for item in desired {
        let path = repository.join(&item.path);
        if fs::symlink_metadata(&path).is_ok() {
            conflicts.push(path.display().to_string());
        }
    }
    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "initialization would overwrite existing paths:\n{}",
            conflicts.join("\n")
        ))
    }
}

fn discover_aliases(
    repository: &Path,
    agents: &[String],
    adopt_alias: bool,
) -> Result<Vec<AdoptedAlias>, String> {
    if !adopt_alias {
        return Ok(Vec::new());
    }
    if !agents.iter().any(|item| item == "claude") || !agents.iter().any(|item| item == "codex") {
        return Err("`--adopt-alias` requires both codex and claude integrations".into());
    }
    let path = repository.join(".claude/skills");
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("cannot inspect proposed alias {}: {error}", path.display()))?;
    if !metadata.file_type().is_symlink() {
        return Err(format!(
            "proposed alias is not a symlink: {}",
            path.display()
        ));
    }
    let target = fs::read_link(&path)
        .map_err(|error| format!("cannot read proposed alias {}: {error}", path.display()))?;
    if target.is_absolute() {
        return Err(format!(
            "proposed alias must be relative: {}",
            path.display()
        ));
    }
    let expected = normalize_path(&repository.join(".agents/skills"));
    let resolved = normalize_path(&path.parent().unwrap_or(repository).join(&target));
    if resolved != expected || !resolved.starts_with(normalize_path(repository)) {
        return Err(format!(
            "proposed alias {} must target the repository-internal .agents/skills directory",
            path.display()
        ));
    }
    Ok(vec![AdoptedAlias {
        integration: "claude".into(),
        path: ".claude/skills".into(),
        target: target.to_string_lossy().into_owned(),
    }])
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            PathComponent::ParentDir => {
                result.pop();
            }
            PathComponent::CurDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    result
}

fn write_installation(root: &Path, installation: &Installation) -> Result<(), String> {
    write_replace(
        &root.join("installation.json"),
        installation_json(installation).as_bytes(),
    )
}

fn installation_json(installation: &Installation) -> String {
    let value = Json::obj(vec![
        ("format", Json::str(INSTALLATION_FORMAT)),
        ("schemaVersion", Json::Num(INSTALLATION_SCHEMA as f64)),
        ("releaseVersion", Json::str(&installation.release_version)),
        ("migrationLine", Json::str(&installation.migration_line)),
        (
            "agents",
            Json::Arr(installation.agents.iter().map(Json::str).collect()),
        ),
        (
            "components",
            Json::Arr(
                installation
                    .components
                    .iter()
                    .map(|item| {
                        Json::obj(vec![
                            ("id", Json::str(&item.id)),
                            ("manifest", Json::str(&item.manifest)),
                            ("version", Json::str(&item.version)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "resources",
            Json::Arr(
                installation
                    .resources
                    .iter()
                    .map(|item| {
                        Json::obj(vec![
                            ("id", Json::str(&item.id)),
                            ("path", Json::str(&item.path)),
                            ("sha256", Json::str(&item.digest)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "aliases",
            Json::Arr(
                installation
                    .aliases
                    .iter()
                    .map(|item| {
                        Json::obj(vec![
                            ("integration", Json::str(&item.integration)),
                            ("path", Json::str(&item.path)),
                            ("target", Json::str(&item.target)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    value.to_string_pretty()
}

fn apply_transaction(changes: Vec<(PathBuf, Option<Vec<u8>>)>) -> Result<(), String> {
    let mut paths = BTreeSet::new();
    for (path, _) in &changes {
        if !paths.insert(path.clone()) {
            return Err(format!(
                "transaction contains duplicate path {}",
                path.display()
            ));
        }
    }
    let mut entries: Vec<TransactionState> = Vec::new();
    for (index, (path, content)) in changes.into_iter().enumerate() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        let stage = match content {
            Some(content) => {
                let stage = transaction_sidecar(&path, index, "stage")?;
                let mut options = fs::OpenOptions::new();
                options.write(true).create_new(true);
                use std::io::Write;
                let mut file = match options.open(&stage) {
                    Ok(file) => file,
                    Err(error) => {
                        for entry in &entries {
                            if let Some(stage) = &entry.stage {
                                let _ = fs::remove_file(stage);
                            }
                        }
                        return Err(format!("cannot stage {}: {error}", path.display()));
                    }
                };
                if let Err(error) = file.write_all(&content) {
                    let _ = fs::remove_file(&stage);
                    for entry in &entries {
                        if let Some(stage) = &entry.stage {
                            let _ = fs::remove_file(stage);
                        }
                    }
                    return Err(format!("cannot stage {}: {error}", path.display()));
                }
                Some(stage)
            }
            None => None,
        };
        entries.push(TransactionState {
            path,
            stage,
            backup: None,
            published: false,
        });
    }
    for index in 0..entries.len() {
        if fs::symlink_metadata(&entries[index].path).is_ok() {
            let backup = transaction_sidecar(&entries[index].path, index, "backup")?;
            if fs::symlink_metadata(&backup).is_ok() {
                let detail = rollback_transaction(&mut entries);
                return Err(format!(
                    "transaction backup path is occupied: {}{detail}",
                    backup.display()
                ));
            }
            if let Err(error) = fs::rename(&entries[index].path, &backup) {
                let detail = rollback_transaction(&mut entries);
                return Err(format!(
                    "cannot prepare replacement of {}: {error}{detail}",
                    entries[index].path.display()
                ));
            }
            entries[index].backup = Some(backup);
        }
    }
    for index in 0..entries.len() {
        if let Some(stage) = entries[index].stage.as_ref() {
            if let Err(error) = fs::rename(stage, &entries[index].path) {
                let path = entries[index].path.display().to_string();
                let detail = rollback_transaction(&mut entries);
                return Err(format!("cannot publish {path}: {error}{detail}"));
            }
            entries[index].published = true;
        }
    }
    let mut cleanup = Vec::new();
    for entry in &entries {
        if let Some(backup) = &entry.backup {
            if let Err(error) = fs::remove_file(backup) {
                cleanup.push(format!("{}: {error}", backup.display()));
            }
        }
    }
    if cleanup.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "cohort was published, but backup cleanup failed:\n{}",
            cleanup.join("\n")
        ))
    }
}

fn rollback_transaction(entries: &mut [TransactionState]) -> String {
    let mut errors = Vec::new();
    for entry in entries.iter_mut().rev() {
        if entry.published {
            if let Err(error) = fs::remove_file(&entry.path) {
                errors.push(format!("cannot remove {}: {error}", entry.path.display()));
            }
        }
        if let Some(backup) = &entry.backup {
            if let Err(error) = fs::rename(backup, &entry.path) {
                errors.push(format!("cannot restore {}: {error}", entry.path.display()));
            }
        }
        if let Some(stage) = &entry.stage {
            let _ = fs::remove_file(stage);
        }
    }
    if errors.is_empty() {
        "; transaction rolled back".into()
    } else {
        format!("; rollback also failed: {}", errors.join("; "))
    }
}

fn transaction_sidecar(path: &Path, index: usize, role: &str) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("managed path {} has no file name", path.display()))?
        .to_string_lossy();
    Ok(path.with_file_name(format!(
        ".{name}.azimuth-{}-{index}.{role}",
        std::process::id()
    )))
}

fn write_new(path: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    use std::io::Write;
    let mut file = options
        .open(path)
        .map_err(|error| format!("cannot create {}: {error}", path.display()))?;
    file.write_all(content)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn write_replace(path: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let temporary = transaction_sidecar(path, 0, "single-stage")?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    use std::io::Write;
    let mut file = options
        .open(&temporary)
        .map_err(|error| format!("cannot stage {}: {error}", path.display()))?;
    if let Err(error) = file.write_all(content) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("cannot write {}: {error}", path.display()));
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("cannot replace {}: {error}", path.display()));
    }
    Ok(())
}

fn require_string<'a>(value: &'a Json, field: &str, path: &Path) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Json::as_str)
        .ok_or_else(|| format!("{}: expected string `{field}`", path.display()))
}

fn require_number(value: &Json, field: &str, path: &Path) -> Result<u32, String> {
    let number = value
        .get(field)
        .and_then(Json::as_num)
        .ok_or_else(|| format!("{}: expected number `{field}`", path.display()))?;
    if number.fract() != 0.0 || number < 0.0 || number > u32::MAX as f64 {
        return Err(format!("{}: invalid integer `{field}`", path.display()));
    }
    Ok(number as u32)
}

fn string_array(value: &Json, field: &str, path: &Path) -> Result<Vec<String>, String> {
    value
        .get(field)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{}: expected array `{field}`", path.display()))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{}: `{field}` entries must be strings", path.display()))
        })
        .collect()
}

fn object_array<'a>(value: &'a Json, field: &str, path: &Path) -> Result<&'a [Json], String> {
    let values = value
        .get(field)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{}: expected array `{field}`", path.display()))?;
    if values.iter().any(|item| !matches!(item, Json::Obj(_))) {
        return Err(format!(
            "{}: `{field}` entries must be objects",
            path.display()
        ));
    }
    Ok(values)
}

fn require_safe_relative(value: &str, field: &str, path: &Path) -> Result<(), String> {
    let candidate = Path::new(value);
    if candidate.as_os_str().is_empty()
        || candidate.is_absolute()
        || candidate.components().any(|component| {
            matches!(
                component,
                PathComponent::ParentDir | PathComponent::RootDir | PathComponent::Prefix(_)
            )
        })
    {
        return Err(format!(
            "{}: `{field}` must be a non-escaping repository-relative path",
            path.display()
        ));
    }
    Ok(())
}
