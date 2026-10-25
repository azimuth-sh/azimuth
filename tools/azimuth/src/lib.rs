//! Azimuth core model loading and selection.

pub mod account_delta;
pub mod account_model;
pub mod account_realization;
pub mod account_support;
pub mod account_syntax_migration;
pub mod account_verification;
pub mod adapter;
pub mod adapter_host;
pub mod assurance;
pub mod assurance_publication;
pub mod assurance_review;
pub mod authoring_review;
pub mod candidate_execution;
pub mod change;
pub mod deployment_status;
pub mod diag;
pub mod execution_impact;
pub mod federation;
pub mod fingerprint;
pub mod harness_capture;
pub mod installation;
pub mod intent;
pub mod json;
pub mod labels;
pub mod manifest;
pub mod model;
pub mod provenance;
pub mod realization;
pub mod release;
pub mod resources;
pub mod retained_inputs;
pub mod review_authoring;
pub mod review_heads;
pub mod run;
pub mod run_plan;
pub mod run_selection;
pub mod spec;
pub mod traceability;
pub mod unit_inputs;
pub mod validation;
pub mod verification;
pub mod verification_packages;
pub mod workflow;
pub mod workflow_references;
pub mod workflow_state;
pub mod workspace;

use crate::diag::Diag;
use crate::model::{Model, SourceIdentity};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Loaded {
    pub model: Model,
    pub warnings: Vec<Diag>,
}

/// `billing/**` selects every spec below that id; all other patterns are exact spec ids.
pub fn selects(pattern: &str, spec_id: &str) -> bool {
    match pattern.strip_suffix("/**") {
        Some(prefix) => spec_id == prefix || spec_id.starts_with(&format!("{prefix}/")),
        None => pattern == spec_id,
    }
}

pub fn load(
    model_dir: &Path,
    workspace_path: &Path,
    manifests: &[PathBuf],
    only: &[String],
) -> Result<Loaded, Vec<Diag>> {
    load_with_support(model_dir, workspace_path, manifests, &[], only)
}

pub fn load_with_support(
    model_dir: &Path,
    workspace_path: &Path,
    manifests: &[PathBuf],
    supports: &[PathBuf],
    only: &[String],
) -> Result<Loaded, Vec<Diag>> {
    let loaded = spec::load_specs(model_dir)?;
    let mut model = Model {
        specs: loaded.specs,
        ..Default::default()
    };
    let mut warnings = loaded.warnings;
    let mut errors = Vec::new();

    match workspace::load(workspace_path) {
        Ok(workspace) => model.workspace = workspace,
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    match load_realization_facets(model_dir) {
        Ok((legacy, accounts)) => {
            model.realizations = legacy;
            model.account_realizations = accounts;
        }
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    match load_verification_facets(model_dir) {
        Ok(accounts) => model.account_verifications = accounts,
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    load_review_heads(model_dir, &mut model, &mut errors);

    for path in manifests {
        match manifest::load(path) {
            Ok(manifest) => append_manifest(&mut model, &manifest),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    for path in supports {
        match account_support::load(path) {
            Ok(support) => model.account_supports.push(support),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    normalize_local_sources(&mut model, &mut errors);
    model.resolve_source_membership();
    check_facet_owners(
        model
            .realizations
            .iter()
            .map(|item| (&item.spec, &item.path))
            .chain(
                model
                    .account_realizations
                    .iter()
                    .map(|item| (&item.spec, &item.path)),
            ),
        "realization",
        &mut errors,
    );
    check_facet_owners(
        model
            .account_verifications
            .iter()
            .map(|item| (&item.owner, &item.path)),
        "verification",
        &mut errors,
    );
    errors.extend(verification_owner_issues(&model));
    errors.extend(model.entity_declaration_issues());
    errors.extend(model.verification_declaration_issues());
    errors.extend(model.account_check_issues());
    errors.extend(verification_packages::validate_model(&model));
    errors.extend(merged_manifest_issues(&model));
    errors.extend(mechanism_route_issues(&model));
    if !errors.is_empty() {
        return Err(errors);
    }

    warnings.extend(package_location_warnings(&model));
    apply_selection(&mut model, only);
    Ok(Loaded { model, warnings })
}

/// Loads a complete multi-repository authority account before deriving any selected view.
pub fn load_assembly(
    assembly: &federation::Assembly,
    only: &[String],
) -> Result<Loaded, Vec<Diag>> {
    let mut model = Model::default();
    let mut errors = Vec::new();
    match federation::load_component_registry(assembly) {
        Ok(components) => model.components = components,
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    model.workspace.packages = assembly.project.packages.clone();
    model.workspace.areas = assembly
        .project
        .areas
        .iter()
        .map(|area| workspace::Area {
            id: area.id.clone(),
            mounts: area
                .mounts
                .iter()
                .map(|mount| workspace::Mount {
                    id: mount.id.clone(),
                    path: mount.path.clone(),
                })
                .collect(),
        })
        .collect();

    let mut warnings = Vec::new();

    for root in &assembly.model_roots {
        match spec::load_specs(root) {
            Ok(loaded) => {
                warnings.extend(loaded.warnings);
                extend_unique_facets(
                    &mut model.specs,
                    loaded.specs,
                    |item| &item.id,
                    |item| &item.path,
                    "spec",
                    &mut errors,
                );
            }
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        match load_realization_facets(root) {
            Ok((legacy, accounts)) => {
                extend_unique_facets(
                    &mut model.realizations,
                    legacy,
                    |item| &item.spec,
                    |item| &item.path,
                    "realization",
                    &mut errors,
                );
                extend_unique_facets(
                    &mut model.account_realizations,
                    accounts,
                    |item| &item.spec,
                    |item| &item.path,
                    "account realization",
                    &mut errors,
                );
            }
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        match load_verification_facets(root) {
            Ok(accounts) => extend_unique_facets(
                &mut model.account_verifications,
                accounts,
                |item| &item.owner,
                |item| &item.path,
                "account verification",
                &mut errors,
            ),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        load_review_heads(root, &mut model, &mut errors);
    }
    for manifest in &assembly.manifests {
        append_manifest(&mut model, manifest);
    }
    for item in &assembly.account_support {
        let mut support = item.support.clone();
        let qualify = |id: &str| format!("{}/{}", item.repository, id);
        for artifact in &mut support.artifacts {
            artifact.id = qualify(&artifact.id);
        }
        for element in &mut support.elements {
            element.artifacts = element.artifacts.iter().map(|id| qualify(id)).collect();
        }
        for check in &mut support.checks {
            check.artifacts = check.artifacts.iter().map(|id| qualify(id)).collect();
        }
        model.account_supports.push(support);
    }
    model.resolve_source_membership();
    check_facet_owners(
        model
            .realizations
            .iter()
            .map(|item| (&item.spec, &item.path))
            .chain(
                model
                    .account_realizations
                    .iter()
                    .map(|item| (&item.spec, &item.path)),
            ),
        "realization",
        &mut errors,
    );
    check_facet_owners(
        model
            .account_verifications
            .iter()
            .map(|item| (&item.owner, &item.path)),
        "verification",
        &mut errors,
    );
    errors.extend(verification_owner_issues(&model));
    errors.extend(model.entity_declaration_issues());
    errors.extend(model.verification_declaration_issues());
    errors.extend(model.account_check_issues());
    errors.extend(verification_packages::validate_model(&model));
    errors.extend(merged_manifest_issues(&model));
    errors.extend(mechanism_route_issues(&model));
    if !errors.is_empty() {
        return Err(errors);
    }

    warnings.extend(package_location_warnings(&model));
    apply_selection(&mut model, only);
    Ok(Loaded { model, warnings })
}

fn load_review_heads(root: &Path, model: &mut Model, errors: &mut Vec<Diag>) {
    for kind in review_heads::HeadKind::ALL {
        match named_paths(root, kind.file_name()) {
            Ok(paths) => {
                for path in paths {
                    match review_heads::load(&path, kind) {
                        Ok(mut heads) => model.review_heads.append(&mut heads),
                        Err(mut diagnostics) => errors.append(&mut diagnostics),
                    }
                }
            }
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
}

fn check_facet_owners<'a>(
    facets: impl Iterator<Item = (&'a String, &'a String)>,
    kind: &str,
    errors: &mut Vec<Diag>,
) {
    let mut seen = BTreeMap::<&str, &str>::new();
    for (owner, path) in facets {
        if let Some(previous) = seen.insert(owner, path) {
            errors.push(Diag::at(path, 1, format!(
                "model-source-ownership-conflict: {kind} `{owner}` is already declared by {previous}"
            )));
        }
    }
}

fn verification_owner_issues(model: &Model) -> Vec<Diag> {
    let owners = model
        .specs
        .iter()
        .map(|spec| spec.id.as_str())
        .collect::<BTreeSet<_>>();
    model
        .account_verifications
        .iter()
        .filter(|verification| !owners.contains(verification.owner.as_str()))
        .map(|verification| {
            Diag::at(
                &verification.path,
                1,
                format!(
                    "verification authority `{}` has no current owning spec",
                    verification.owner
                ),
            )
        })
        .collect()
}

fn uses_compact_facet(source: &str, heading: &str) -> bool {
    let mut fenced = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if !fenced && (trimmed.starts_with(heading) || trimmed.starts_with("## Section: ")) {
            return true;
        }
    }
    false
}

fn load_realization_facets(
    root: &Path,
) -> Result<
    (
        Vec<realization::Realization>,
        Vec<account_realization::AccountRealization>,
    ),
    Vec<Diag>,
> {
    let paths = named_paths(root, "realization.md")?;
    let (mut legacy, mut accounts, mut errors) = (Vec::new(), Vec::new(), Vec::new());
    for path in paths {
        let display = path.display().to_string();
        match fs::read_to_string(&path) {
            Ok(source) if uses_compact_facet(&source, "## Requirement realization: ") => {
                match account_realization::parse_account_realization(&display, &source) {
                    Ok(item) => accounts.push(item),
                    Err(mut diagnostics) => errors.append(&mut diagnostics),
                }
            }
            Ok(source) => match realization::parse_realization(&display, &source) {
                Ok(item) => legacy.push(item),
                Err(mut diagnostics) => errors.append(&mut diagnostics),
            },
            Err(error) => errors.push(Diag::file(
                &display,
                format!("cannot read realization: {error}"),
            )),
        }
    }
    check_facet_owners(
        legacy
            .iter()
            .map(|item| (&item.spec, &item.path))
            .chain(accounts.iter().map(|item| (&item.spec, &item.path))),
        "realization",
        &mut errors,
    );
    if errors.is_empty() {
        Ok((legacy, accounts))
    } else {
        Err(errors)
    }
}

fn load_verification_facets(
    root: &Path,
) -> Result<Vec<account_verification::AccountVerification>, Vec<Diag>> {
    let paths = named_paths(root, "verification.md")?;
    let (mut accounts, mut errors) = (Vec::new(), Vec::new());
    for path in paths {
        let display = path.display().to_string();
        match fs::read_to_string(&path) {
            Ok(source) => match account_verification::parse_account_verification(&display, &source)
            {
                Ok(item) => accounts.push(item),
                Err(mut diagnostics) => errors.append(&mut diagnostics),
            },
            Err(error) => errors.push(Diag::file(
                &display,
                format!("cannot read verification: {error}"),
            )),
        }
    }
    check_facet_owners(
        accounts.iter().map(|item| (&item.owner, &item.path)),
        "verification",
        &mut errors,
    );
    if errors.is_empty() {
        Ok(accounts)
    } else {
        Err(errors)
    }
}

fn named_paths(root: &Path, name: &str) -> Result<Vec<PathBuf>, Vec<Diag>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    collect_named(root, name, &mut paths).map_err(|error| {
        vec![Diag::file(
            &root.display().to_string(),
            format!("cannot discover {name}: {error}"),
        )]
    })?;
    paths.sort();
    Ok(paths)
}

fn collect_named(dir: &Path, name: &str, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_named(&path, name, out)?;
        } else if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            out.push(path);
        }
    }
    Ok(())
}

fn merged_manifest_issues(model: &Model) -> Vec<Diag> {
    let mut issues = Vec::new();
    let mut realizations = BTreeMap::<(String, String), (&str, &str)>::new();
    for realization in &model.realizes {
        let Some(source) = realization.source.as_ref() else {
            continue;
        };
        let key = (realization.requirement.clone(), source.key());
        if let Some((previous_file, previous_fingerprint)) = realizations.get(&key) {
            issues.push(Diag::file(
                &realization.file,
                format!(
                    "duplicate realization `{}|{}` across manifests (first at `{previous_file}` \
                     with fingerprint `{previous_fingerprint}`)",
                    key.0, key.1
                ),
            ));
        } else {
            realizations.insert(key, (&realization.file, &realization.source_fingerprint));
        }
    }

    let mut implementations = BTreeMap::<(String, String), (&str, &str)>::new();
    for implementation in &model.check_implementations {
        let Some(source) = implementation.source.as_ref() else {
            continue;
        };
        let key = (implementation.check.clone(), source.key());
        if let Some((previous_file, previous_fingerprint)) = implementations.get(&key) {
            issues.push(Diag::file(
                &implementation.file,
                format!(
                    "duplicate Check implementation `{}|{}` across manifests (first at `{previous_file}` with fingerprint `{previous_fingerprint}`)",
                    key.0, key.1
                ),
            ));
        } else {
            implementations.insert(
                key,
                (&implementation.file, &implementation.source_fingerprint),
            );
        }
    }

    let mut mechanism_sources = BTreeMap::<String, ((String, String), &str)>::new();
    for implementation in &model.mechanism_implementations {
        let key = (
            implementation.spec.clone(),
            implementation.mechanism.clone(),
        );
        if let Some(source) = &implementation.source {
            let source_key = source.key();
            if let Some((previous_target, previous_file)) = mechanism_sources.get(&source_key) {
                issues.push(Diag::file(
                    &implementation.file,
                    format!(
                        "mechanism source identity `{source_key}` has multiple marker targets \
                         (first `{}#{}` at `{previous_file}`)",
                        previous_target.0, previous_target.1
                    ),
                ));
            } else {
                mechanism_sources.insert(source_key, (key, &implementation.file));
            }
        }
    }

    let mut enumeration_witnesses =
        BTreeMap::<(String, String, String, String), (&str, &str)>::new();
    for enumeration in &model.enumerations {
        let Some(identity) = enumeration.identity.as_ref() else {
            continue;
        };
        let key = (
            enumeration.class.clone(),
            identity.area.clone(),
            identity.mount.clone(),
            enumeration.kind.clone(),
        );
        if let Some((previous_source, previous_fingerprint)) = enumeration_witnesses.get(&key) {
            issues.push(Diag::file(
                &enumeration.source,
                format!(
                    "multiple enumeration witnesses for contribution `{}|{}|{}|{}` across \
                     manifests (first at `{previous_source}` with fingerprint \
                     `{previous_fingerprint}`)",
                    key.0, key.1, key.2, key.3
                ),
            ));
        } else {
            enumeration_witnesses
                .insert(key, (&enumeration.source, &enumeration.source_fingerprint));
        }
    }

    let mut surface_members = BTreeMap::<(String, String), (&str, &str)>::new();
    for member in &model.class_members {
        let key = (member.class.clone(), member.file.clone());
        if let Some((previous_site, previous_language)) = surface_members.get(&key) {
            issues.push(Diag::file(
                &member.file,
                format!(
                    "duplicate surface member `{}|{}` across manifests (first site \
                     `{previous_site}` in `{previous_language}`)",
                    key.0, key.1
                ),
            ));
        } else {
            surface_members.insert(key, (&member.site, &member.lang));
        }
    }

    let mut artifacts = BTreeMap::<&str, &str>::new();
    let reserved_marker_ids = model
        .mechanism_implementations
        .iter()
        .filter_map(|implementation| {
            manifest::mechanism_address_kind(&implementation.lang)
                .map(|kind| format!("{kind}:{}", implementation.site))
        })
        .collect::<BTreeSet<_>>();
    for artifact in &model.artifacts {
        if reserved_marker_ids.contains(&artifact.id) {
            issues.push(Diag::file(
                &artifact.file,
                format!(
                    "marker companion Artifact `{}` collides with an ordinary Artifact",
                    artifact.id
                ),
            ));
        }
        if let Some(previous_file) = artifacts.insert(&artifact.id, &artifact.file) {
            issues.push(Diag::file(
                &artifact.file,
                format!(
                    "duplicate artifact id `{}` across manifests (first at `{previous_file}`)",
                    artifact.id
                ),
            ));
        }
    }
    issues
}

fn append_manifest(model: &mut Model, manifest: &manifest::Manifest) {
    model.package_producers.extend(manifest.extensions.clone());
    model.realizes.extend(manifest.realizes.clone());
    model
        .check_implementations
        .extend(manifest.check_implementations.clone());
    model
        .mechanism_implementations
        .extend(manifest.mechanism_implementations.clone());
    model.class_members.extend(manifest.class_members.clone());
    model.enumerations.extend(manifest.enumerations.clone());
    model.artifacts.extend(manifest.artifacts.clone());
}

fn apply_selection(model: &mut Model, only: &[String]) {
    if only.is_empty() {
        return;
    }
    let mut selected_requirements = model
        .scenarios()
        .filter(|requirement| {
            only.iter()
                .any(|pattern| selects(pattern, &requirement.spec.id))
        })
        .map(|requirement| requirement.id())
        .collect::<BTreeSet<_>>();
    let selected_parent_requirements = selected_requirements
        .iter()
        .filter_map(|scenario| {
            model
                .find_scenario(scenario)
                .map(|view| view.requirement.id.clone())
        })
        .collect::<BTreeSet<_>>();
    // Retaining the parent composition prevents partial selection from inventing a narrower assurance account.
    selected_requirements.extend(
        model
            .scenarios()
            .filter(|scenario| {
                selected_parent_requirements.contains(&scenario.requirement.id.clone())
            })
            .map(|scenario| scenario.id()),
    );
    let selected_checks = model
        .account_verifications
        .iter()
        .flat_map(|file| &file.checks)
        .filter(|check| selected_parent_requirements.contains(&check.requirement))
        .map(|check| check.definition.id.clone())
        .collect::<BTreeSet<_>>();
    let selected_specs = model
        .scenarios()
        .filter(|requirement| selected_requirements.contains(&requirement.id()))
        .map(|requirement| requirement.spec.id.clone())
        .collect::<BTreeSet<_>>();
    let selected_surfaces = model
        .scenarios()
        .filter(|requirement| selected_requirements.contains(&requirement.id()))
        .filter_map(|requirement| requirement.requirement.over.clone())
        .collect::<BTreeSet<_>>();
    let mut retained_mechanisms = BTreeSet::new();
    for requirement in model
        .scenarios()
        .filter(|requirement| selected_requirements.contains(&requirement.id()))
    {
        if let Some(realization) = model.realization_for(&requirement.spec.id) {
            for entry in &realization.entries {
                let attached = entry.target.id() == requirement.requirement.id;
                if attached {
                    retained_mechanisms.extend(
                        entry
                            .mechanisms
                            .iter()
                            .filter(|mechanism| {
                                mechanism.scenarios.is_empty()
                                    || mechanism.scenarios.contains(&requirement.scenario.id)
                            })
                            .map(|mechanism| mechanism.id.clone()),
                    );
                }
            }
        }
    }
    let mut selected_artifacts = BTreeSet::new();
    for realization in &model.realizations {
        for mechanism in realization
            .entries
            .iter()
            .flat_map(|entry| &entry.mechanisms)
        {
            let identity = mechanism.id.clone();
            if retained_mechanisms.contains(&identity) {
                selected_artifacts.extend(
                    model
                        .mechanism_bindings(&realization.spec, mechanism)
                        .into_iter()
                        .map(str::to_string),
                );
            }
        }
    }
    let retained_sources = model
        .realizes
        .iter()
        .filter(|site| {
            selected_parent_requirements.contains(&site.requirement.clone())
                || selected_surfaces.contains(&site.spec)
        })
        .filter_map(|item| item.source.as_ref())
        .chain(
            model
                .mechanism_implementations
                .iter()
                .filter(|item| retained_mechanisms.contains(&item.mechanism.clone()))
                .filter_map(|item| item.source.as_ref()),
        )
        .chain(
            model
                .check_implementations
                .iter()
                .filter(|item| selected_checks.contains(&item.check))
                .filter_map(|item| item.source.as_ref()),
        )
        .chain(
            model
                .class_members
                .iter()
                .filter(|item| selected_surfaces.contains(&item.class))
                .filter_map(|item| item.source.as_ref()),
        )
        .chain(
            model
                .enumerations
                .iter()
                .filter(|item| selected_surfaces.contains(&item.class))
                .filter_map(|item| item.identity.as_ref()),
        )
        .chain(
            model
                .artifacts
                .iter()
                .filter(|item| selected_artifacts.contains(&item.id))
                .filter_map(|item| item.source.as_ref()),
        )
        .map(|source| (source.area.clone(), source.mount.clone()))
        .collect::<BTreeSet<_>>();
    let obligation_areas = model
        .realization_obligations()
        .iter()
        .filter(|obligation| selected_parent_requirements.contains(&obligation.requirement.clone()))
        .flat_map(|obligation| obligation.areas.iter().cloned())
        .collect::<BTreeSet<_>>();
    let surface_mounts = model
        .workspace
        .surfaces
        .iter()
        .filter(|surface| selected_surfaces.contains(&surface.id))
        .flat_map(|surface| {
            surface
                .contributions
                .iter()
                .map(|item| (item.area.clone(), item.mount.clone()))
        })
        .collect::<BTreeSet<_>>();

    model.specs.retain_mut(|spec| {
        if selected_surfaces.contains(&spec.id) {
            return true;
        }
        if !selected_specs.contains(&spec.id) {
            return false;
        }
        spec.requirements.retain_mut(|requirement| {
            requirement
                .scenarios
                .retain(|scenario| selected_requirements.contains(&scenario.id.clone()));
            !requirement.scenarios.is_empty()
        });
        !spec.requirements.is_empty()
    });
    model
        .account_realizations
        .retain(|realization| selected_specs.contains(&realization.spec));
    model
        .account_verifications
        .retain(|item| selected_specs.contains(&item.owner));
    model.realizations.retain_mut(|realization| {
        realization.entries.retain_mut(|entry| {
            entry
                .mechanisms
                .retain(|mechanism| retained_mechanisms.contains(&mechanism.id.clone()));
            !entry.mechanisms.is_empty()
        });
        !realization.entries.is_empty()
    });
    model.realizes.retain(|site| {
        selected_parent_requirements.contains(&site.requirement.clone())
            || selected_surfaces.contains(&site.spec)
    });
    model
        .mechanism_implementations
        .retain(|implementation| retained_mechanisms.contains(&implementation.mechanism));
    model
        .workspace
        .realization_obligations
        .retain(|obligation| {
            selected_parent_requirements.contains(&obligation.requirement.clone())
        });
    model
        .check_implementations
        .retain(|implementation| selected_checks.contains(&implementation.check));
    model
        .class_members
        .retain(|member| selected_surfaces.contains(&member.class));
    model
        .enumerations
        .retain(|enumeration| selected_surfaces.contains(&enumeration.class));
    model
        .artifacts
        .retain(|artifact| selected_artifacts.contains(&artifact.id));
    model
        .workspace
        .surfaces
        .retain(|surface| selected_surfaces.contains(&surface.id));
    model.workspace.areas.retain_mut(|area| {
        let retain_all_mounts = obligation_areas.contains(&area.id);
        area.mounts.retain(|mount| {
            retain_all_mounts
                || retained_sources.contains(&(area.id.clone(), mount.id.clone()))
                || surface_mounts.contains(&(area.id.clone(), mount.id.clone()))
        });
        retain_all_mounts || !area.mounts.is_empty()
    });
}

fn normalize_local_sources(model: &mut Model, errors: &mut Vec<Diag>) {
    let workspace = model.workspace.clone();
    for item in &mut model.package_producers {
        if let Some(source) = &item.source {
            validate_supplied_source(&workspace, source, &item.file, errors);
            continue;
        }
        item.source = local_source(
            &workspace,
            &item.file,
            address_kind(&item.lang, &item.file, &item.site),
            address_value(&item.lang, &item.file, &item.site),
        );
    }
    for item in &mut model.realizes {
        if let Some(source) = &item.source {
            validate_supplied_source(&workspace, source, &item.file, errors);
            continue;
        }
        item.source = local_source(
            &workspace,
            &item.file,
            address_kind(&item.lang, &item.file, &item.site),
            address_value(&item.lang, &item.file, &item.site),
        );
    }
    for item in &mut model.check_implementations {
        if let Some(source) = &item.source {
            validate_supplied_source(&workspace, source, &item.file, errors);
            continue;
        }
        item.source = local_source(
            &workspace,
            &item.file,
            address_kind(&item.lang, &item.file, &item.site),
            address_value(&item.lang, &item.file, &item.site),
        );
    }
    let mut rewrites = Vec::new();
    let mut paired_artifacts = BTreeSet::new();
    let mut reserved_raw_ids = BTreeSet::new();
    for (implementation_index, item) in model.mechanism_implementations.iter().enumerate() {
        if let Some(source) = &item.source {
            validate_supplied_source(&workspace, source, &item.file, errors);
            continue;
        }
        let Some(kind) = manifest::mechanism_address_kind(&item.lang) else {
            continue;
        };
        let raw_binding = format!("{kind}:{}", item.site);
        reserved_raw_ids.insert(raw_binding.clone());
        let matches = model
            .artifacts
            .iter()
            .enumerate()
            .filter(|(_, artifact)| {
                artifact.id == raw_binding
                    && artifact.kind == kind
                    && artifact.file == item.file
                    && artifact.source.is_none()
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let [artifact_index] = matches.as_slice() else {
            errors.push(Diag::file(
                &item.file,
                format!(
                    "marker implementation `{}#{}` has no unique exact companion Artifact",
                    item.spec, item.mechanism
                ),
            ));
            continue;
        };
        if !paired_artifacts.insert(*artifact_index) {
            errors.push(Diag::file(
                &item.file,
                format!("marker companion Artifact `{raw_binding}` has multiple targets"),
            ));
            continue;
        }
        let Some(source) = local_source(&workspace, &item.file, kind, item.site.clone()) else {
            errors.push(Diag::file(
                &item.file,
                "marker implementation belongs to no declared area mount",
            ));
            continue;
        };
        rewrites.push((implementation_index, *artifact_index, source));
    }
    for (artifact_index, artifact) in model.artifacts.iter().enumerate() {
        if !paired_artifacts.contains(&artifact_index) && reserved_raw_ids.contains(&artifact.id) {
            errors.push(Diag::file(
                &artifact.file,
                format!(
                    "marker companion Artifact `{}` collides with an ordinary Artifact",
                    artifact.id
                ),
            ));
        }
    }
    for (implementation_index, artifact_index, source) in rewrites {
        let key = source.key();
        let implementation = &mut model.mechanism_implementations[implementation_index];
        implementation.binding = key.clone();
        implementation.source = Some(source.clone());
        let artifact = &mut model.artifacts[artifact_index];
        artifact.id = key;
        artifact.source = Some(source);
    }
    for item in &mut model.class_members {
        if let Some(source) = &item.source {
            validate_supplied_source(&workspace, source, &item.file, errors);
            continue;
        }
        item.source = local_source(
            &workspace,
            &item.file,
            "class-member".into(),
            format!("{}#{}", item.class, item.site),
        );
    }
    for item in &mut model.enumerations {
        if let Some(source) = &item.identity {
            validate_supplied_source(&workspace, source, &item.source, errors);
            continue;
        }
        item.identity = local_source(
            &workspace,
            &item.source,
            "enumerator".into(),
            format!("{}#{}", item.class, item.kind),
        );
    }
    for item in &mut model.artifacts {
        if let Some(source) = &item.source {
            validate_supplied_source(&workspace, source, &item.file, errors);
        } else {
            item.source = local_source(&workspace, &item.file, item.kind.clone(), item.id.clone());
        }
    }
}

fn validate_supplied_source(
    workspace: &workspace::Workspace,
    source: &SourceIdentity,
    file: &str,
    errors: &mut Vec<Diag>,
) {
    let Some(area) = workspace.areas.iter().find(|area| area.id == source.area) else {
        errors.push(Diag::file(
            file,
            format!("qualified source names unknown Area `{}`", source.area),
        ));
        return;
    };
    if !area.mounts.is_empty() && !area.mounts.iter().any(|mount| mount.id == source.mount) {
        errors.push(Diag::file(
            file,
            format!(
                "qualified source names unknown mount `{}` in Area `{}`",
                source.mount, source.area
            ),
        ));
    }
}

fn mechanism_route_issues(model: &Model) -> Vec<Diag> {
    let mut issues = Vec::new();
    for implementation in &model.mechanism_implementations {
        let Some(source) = &implementation.source else {
            continue;
        };
        let raw = format!("{}:{}", source.kind, implementation.site);
        let assembled = source.key();
        for realization in &model.realizations {
            for mechanism in realization
                .entries
                .iter()
                .flat_map(|entry| &entry.mechanisms)
            {
                if mechanism
                    .binding
                    .as_ref()
                    .is_some_and(|binding| binding == &raw || binding == &assembled)
                {
                    issues.push(Diag::file(
                        &realization.path,
                        format!(
                            "explicit mechanism `{}#{}` may not bind marker-only Artifact `{}`",
                            realization.spec,
                            mechanism.id,
                            binding_label(&raw, &assembled, mechanism)
                        ),
                    ));
                }
            }
        }
    }
    issues
}

fn binding_label<'a>(
    raw: &'a str,
    assembled: &'a str,
    mechanism: &'a crate::realization::Mechanism,
) -> &'a str {
    match mechanism.binding.as_deref() {
        Some(binding) if binding == raw => raw,
        _ => assembled,
    }
}

fn local_source(
    workspace: &workspace::Workspace,
    file: &str,
    kind: String,
    address: String,
) -> Option<SourceIdentity> {
    let Some((area, mount)) = local_mount(workspace, file) else {
        return None;
    };
    Some(SourceIdentity {
        area: area.id.clone(),
        mount: mount.id.clone(),
        kind,
        address,
    })
}

fn local_mount<'a>(
    workspace: &'a workspace::Workspace,
    file: &str,
) -> Option<(&'a workspace::Area, &'a workspace::Mount)> {
    let mut matches = workspace
        .areas
        .iter()
        .flat_map(|area| area.mounts.iter().map(move |mount| (area, mount)))
        .filter(|(_, mount)| is_within(file, &mount.path))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.1.path.len().cmp(&left.1.path.len()));
    matches.into_iter().next()
}

fn is_within(file: &str, root: &str) -> bool {
    let file = file.replace('\\', "/");
    let root = root.trim_matches('/');
    file == root
        || file
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn address_kind(language: &str, file: &str, site: &str) -> String {
    if language == "csharp" {
        "dotnet-symbol".into()
    } else if language == "prometheus" && file.ends_with(".rules.test.yml") {
        "prometheus-rule-test".into()
    } else if language == "prometheus" {
        "prometheus-alert".into()
    } else if language == "typescript"
        && file.replace('\\', "/").contains("/app/")
        && file.ends_with("/route.ts")
        && matches!(site, "GET" | "POST" | "PUT" | "PATCH" | "DELETE")
    {
        "next-route".into()
    } else {
        format!("{language}-symbol")
    }
}

fn address_value(language: &str, file: &str, site: &str) -> String {
    if address_kind(language, file, site) != "next-route" {
        return site.to_string();
    }
    let normalized = file.replace('\\', "/");
    let route = normalized
        .split("/app/")
        .nth(1)
        .unwrap_or(&normalized)
        .trim_end_matches("/route.ts");
    format!("{site} /{route}")
}

fn extend_unique_facets<T>(
    target: &mut Vec<T>,
    incoming: Vec<T>,
    id: impl Fn(&T) -> &String,
    path: impl Fn(&T) -> &String,
    kind: &str,
    errors: &mut Vec<Diag>,
) {
    for item in incoming {
        if let Some(previous) = target.iter().find(|previous| id(previous) == id(&item)) {
            errors.push(Diag::at(
                path(&item),
                1,
                format!(
                    "model-source-ownership-conflict: {kind} `{}` is already declared by {}",
                    id(&item),
                    path(previous)
                ),
            ));
        } else {
            target.push(item);
        }
    }
}

fn package_location_warnings(model: &Model) -> Vec<Diag> {
    let mut warnings = Vec::new();
    for realization in &model.realizations {
        warn_if_not_sibling(
            model,
            &realization.spec,
            &realization.path,
            "realization",
            &mut warnings,
        );
    }
    for verification in &model.account_verifications {
        warn_if_not_sibling(
            model,
            &verification.owner,
            &verification.path,
            "verification authority",
            &mut warnings,
        );
    }
    warnings
}

fn warn_if_not_sibling(
    model: &Model,
    spec_id: &str,
    artifact_path: &str,
    artifact_kind: &str,
    warnings: &mut Vec<Diag>,
) {
    let Some(spec) = model.specs.iter().find(|spec| spec.id == spec_id) else {
        return;
    };
    if Path::new(&spec.path).parent() == Path::new(artifact_path).parent() {
        return;
    }
    warnings.push(Diag::at(
        artifact_path,
        1,
        format!(
            "{artifact_kind} for `{spec_id}` is not beside {}; ids are path-independent, so this \
             is a navigation hint only",
            spec.path
        ),
    ));
}
