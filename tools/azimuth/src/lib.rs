//! Azimuth core model loading and selection.

pub mod adapter;
pub mod adapter_host;
pub mod assurance;
pub mod change;
pub mod design;
pub mod diag;
pub mod federation;
pub mod fingerprint;
pub mod json;
pub mod labels;
pub mod manifest;
pub mod model;
pub mod run;
pub mod run_plan;
pub mod spec;
pub mod traceability;
pub mod validation;
pub mod verification;
pub mod workflow;
pub mod workspace;

use crate::diag::Diag;
use crate::model::{Criticality, Model, SourceIdentity};
use crate::verification::{ChallengeDomain, Selector, Verification};
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
    standards_path: &Path,
    workspace_path: &Path,
    manifests: &[PathBuf],
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
    if standards_path.exists() {
        match verification::load_policies(standards_path) {
            Ok(policies) => model.qualification_policies = Some(policies),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    match design::load_designs(model_dir) {
        Ok(designs) => model.designs = designs,
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    match load_verifications(model_dir) {
        Ok(verifications) => model.verifications = verifications,
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    reject_retired_judgment_facets(model_dir, &mut errors);

    for path in manifests {
        match manifest::load(path) {
            Ok(manifest) => append_manifest(&mut model, &manifest),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    normalize_local_sources(&mut model);
    errors.extend(verification_owner_issues(&model));
    errors.extend(model.verification_declaration_issues());
    errors.extend(merged_manifest_issues(&model));
    if !errors.is_empty() {
        return Err(errors);
    }

    if model.qualification_policies.is_none() && needs_policies(&model) {
        warnings.push(Diag::file(
            &standards_path.display().to_string(),
            "no qualification policies file; non-routine Evidence Bindings cannot be resolved",
        ));
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
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

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
        match design::load_designs(root) {
            Ok(items) => extend_unique_facets(
                &mut model.designs,
                items,
                |item| &item.spec,
                |item| &item.path,
                "design",
                &mut errors,
            ),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        match load_verifications(root) {
            Ok(items) => extend_unique_facets(
                &mut model.verifications,
                items,
                |item| &item.owner,
                |item| &item.path,
                "verification authority",
                &mut errors,
            ),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        reject_retired_judgment_facets(root, &mut errors);
    }
    if let Some(path) = &assembly.standards_path {
        match verification::load_policies(path) {
            Ok(policies) => model.qualification_policies = Some(policies),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    for manifest in &assembly.manifests {
        append_manifest(&mut model, manifest);
    }
    errors.extend(verification_owner_issues(&model));
    errors.extend(model.verification_declaration_issues());
    errors.extend(merged_manifest_issues(&model));
    if !errors.is_empty() {
        return Err(errors);
    }

    if model.qualification_policies.is_none() && needs_policies(&model) {
        warnings.push(Diag::file(
            "project",
            "qualification policies are outside this assembly; non-routine bindings are incomplete",
        ));
    }
    warnings.extend(package_location_warnings(&model));
    apply_selection(&mut model, only);
    Ok(Loaded { model, warnings })
}

fn load_verifications(root: &Path) -> Result<Vec<Verification>, Vec<Diag>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    collect_named(root, "verification.md", &mut paths).map_err(|error| {
        vec![Diag::file(
            &root.display().to_string(),
            format!("cannot discover verification authorities: {error}"),
        )]
    })?;
    paths.sort();
    let mut declarations = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match verification::load_verification(&path) {
            Ok(value) => {
                if let Some(previous) = declarations
                    .iter()
                    .find(|previous: &&Verification| previous.owner == value.owner)
                {
                    errors.push(Diag::at(
                        &value.path,
                        1,
                        format!(
                            "verification authority `{}` is already declared by {}",
                            value.owner, previous.path
                        ),
                    ));
                } else {
                    declarations.push(value);
                }
            }
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    if errors.is_empty() {
        Ok(declarations)
    } else {
        Err(errors)
    }
}

fn reject_retired_judgment_facets(root: &Path, errors: &mut Vec<Diag>) {
    if !root.exists() {
        return;
    }
    let mut paths = Vec::new();
    if let Err(error) = collect_named(root, "judgments.md", &mut paths) {
        errors.push(Diag::file(
            &root.display().to_string(),
            format!("cannot discover retired judgment facets: {error}"),
        ));
        return;
    }
    paths.sort();
    errors.extend(paths.into_iter().map(|path| {
        Diag::at(
            &path.display().to_string(),
            1,
            "alpha 1 `judgments.md` is retired; no Claim Judgment format exists in alpha 2",
        )
    }));
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

fn needs_policies(model: &Model) -> bool {
    model.evidence_bindings().any(|binding| {
        model
            .claims()
            .find(|claim| claim.id() == binding.claim)
            .and_then(|claim| claim.requirement.criticality)
            .is_some_and(|criticality| {
                matches!(criticality, Criticality::Standard | Criticality::Critical)
            })
    })
}

fn verification_owner_issues(model: &Model) -> Vec<Diag> {
    let owners = model
        .specs
        .iter()
        .map(|spec| spec.id.as_str())
        .collect::<BTreeSet<_>>();
    model
        .verifications
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

fn merged_manifest_issues(model: &Model) -> Vec<Diag> {
    let mut issues = Vec::new();
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

    let mut artifacts = BTreeMap::<&str, &str>::new();
    for artifact in &model.artifacts {
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
    let selected_claims = model
        .claims()
        .filter(|claim| only.iter().any(|pattern| selects(pattern, &claim.spec.id)))
        .map(|claim| claim.id())
        .collect::<BTreeSet<_>>();
    let selected_bindings = model
        .evidence_bindings()
        .filter(|binding| selected_claims.contains(&binding.claim))
        .map(|binding| binding.id.clone())
        .collect::<BTreeSet<_>>();
    let selected_checks = model
        .evidence_bindings()
        .filter(|binding| selected_bindings.contains(&binding.id))
        .map(|binding| binding.check.clone())
        .collect::<BTreeSet<_>>();
    let selected_surfaces = model
        .claims()
        .filter(|claim| selected_claims.contains(&claim.id()))
        .filter_map(|claim| claim.requirement.over.clone())
        .collect::<BTreeSet<_>>();
    let mut selected_artifacts = BTreeSet::new();
    for design in model
        .designs
        .iter()
        .filter(|design| only.iter().any(|pattern| selects(pattern, &design.spec)))
    {
        for mechanism in design.entries.iter().flat_map(|entry| &entry.mechanisms) {
            selected_artifacts.extend(
                model
                    .mechanism_bindings(&design.spec, mechanism)
                    .into_iter()
                    .map(str::to_string),
            );
        }
    }
    let selected_plans = model
        .challenge_plans()
        .filter_map(|plan| {
            let selectors = plan
                .selectors
                .iter()
                .filter(|selector| {
                    selector_relevant(
                        model,
                        selector,
                        &selected_claims,
                        &selected_bindings,
                        &selected_checks,
                    )
                })
                .map(Selector::canonical)
                .collect::<BTreeSet<_>>();
            (!selectors.is_empty()).then(|| (plan.id.clone(), selectors))
        })
        .collect::<Vec<_>>();
    let selected_plan_ids = selected_plans
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let selected_challengers = model
        .challenge_plans()
        .filter(|plan| selected_plan_ids.contains(&plan.id))
        .map(|plan| plan.challenger.clone())
        .collect::<BTreeSet<_>>();
    let retained_sources = model
        .realizes
        .iter()
        .filter(|site| only.iter().any(|pattern| selects(pattern, &site.spec)))
        .filter_map(|item| item.source.as_ref())
        .chain(
            model
                .mechanism_implementations
                .iter()
                .filter(|item| only.iter().any(|pattern| selects(pattern, &item.spec)))
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
        .workspace
        .realization_obligations
        .iter()
        .filter(|obligation| {
            only.iter()
                .any(|pattern| selects(pattern, &obligation.spec))
        })
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

    model
        .specs
        .retain(|spec| only.iter().any(|pattern| selects(pattern, &spec.id)));
    model
        .designs
        .retain(|design| only.iter().any(|pattern| selects(pattern, &design.spec)));
    model
        .realizes
        .retain(|site| only.iter().any(|pattern| selects(pattern, &site.spec)));
    model.mechanism_implementations.retain(|implementation| {
        only.iter()
            .any(|pattern| selects(pattern, &implementation.spec))
    });
    model
        .workspace
        .realization_obligations
        .retain(|obligation| {
            only.iter()
                .any(|pattern| selects(pattern, &obligation.spec))
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
    for file in &mut model.verifications {
        file.checks
            .retain(|check| selected_checks.contains(&check.id));
        file.bindings
            .retain(|binding| selected_bindings.contains(&binding.id));
        file.qualifications
            .retain(|qualification| selected_bindings.contains(&qualification.id));
        file.challengers
            .retain(|challenger| selected_challengers.contains(&challenger.id));
        file.challenge_plans
            .retain(|plan| selected_plan_ids.contains(&plan.id));
        for plan in &mut file.challenge_plans {
            let retained = selected_plans
                .iter()
                .find(|(id, _)| id == &plan.id)
                .map(|(_, selectors)| selectors)
                .expect("selected plan has retained selectors");
            plan.selectors
                .retain(|selector| retained.contains(&selector.canonical()));
        }
    }
    model.verifications.retain(|file| {
        !file.checks.is_empty()
            || !file.bindings.is_empty()
            || !file.qualifications.is_empty()
            || !file.challengers.is_empty()
            || !file.challenge_plans.is_empty()
    });
}

fn selector_relevant(
    model: &Model,
    selector: &Selector,
    claims: &BTreeSet<String>,
    bindings: &BTreeSet<String>,
    checks: &BTreeSet<String>,
) -> bool {
    match selector {
        Selector::QualificationFromBinding(id) => bindings.contains(id),
        Selector::QualificationFromCheck(id) => checks.contains(id),
        Selector::QualificationFromRealization(identity) => model.realizes.iter().any(|site| {
            site.source
                .as_ref()
                .is_some_and(|source| source.key() == *identity)
                && claims.contains(&format!("{}#{}", site.spec, site.scenario))
                && model.evidence_bindings().any(|binding| {
                    binding.claim == format!("{}#{}", site.spec, site.scenario)
                        && binding
                            .challenge_domain
                            .contains(&ChallengeDomain::Realization)
                })
        }),
        Selector::QualificationFromMechanism(identity) => {
            let mechanism_claims = selected_mechanism_claims(model, identity);
            model.evidence_bindings().any(|binding| {
                bindings.contains(&binding.id)
                    && mechanism_claims.contains(&binding.claim)
                    && binding
                        .challenge_domain
                        .contains(&ChallengeDomain::Mechanism)
            })
        }
        Selector::ClaimJudgmentFromClaim(id) => claims.contains(id),
        Selector::ClaimJudgmentFromRealization(identity) => model.realizes.iter().any(|site| {
            site.source
                .as_ref()
                .is_some_and(|source| source.key() == *identity)
                && claims.contains(&format!("{}#{}", site.spec, site.scenario))
        }),
        Selector::ClaimJudgmentFromMechanism(identity) => {
            selected_mechanism_claims(model, identity)
                .iter()
                .any(|claim| claims.contains(claim))
        }
    }
}

fn selected_mechanism_claims(model: &Model, identity: &str) -> BTreeSet<String> {
    let Some((spec_id, mechanism_id)) = identity.split_once('#') else {
        return BTreeSet::new();
    };
    let Some(design) = model.design_for(spec_id) else {
        return BTreeSet::new();
    };
    model
        .claims()
        .filter(|claim| claim.spec.id == spec_id)
        .filter(|claim| {
            design
                .for_scenario(&claim.scenario.id)
                .into_iter()
                .chain(design.for_requirement(&claim.requirement.id))
                .any(|entry| entry.mechanisms.iter().any(|item| item.id == mechanism_id))
        })
        .map(|claim| claim.id())
        .collect()
}

fn normalize_local_sources(model: &mut Model) {
    let workspace = &model.workspace;
    for item in &mut model.realizes {
        item.source = local_source(
            workspace,
            &item.file,
            address_kind(&item.lang, &item.file, &item.site),
            address_value(&item.lang, &item.file, &item.site),
        );
    }
    for item in &mut model.check_implementations {
        item.source = local_source(
            workspace,
            &item.file,
            address_kind(&item.lang, &item.file, &item.site),
            address_value(&item.lang, &item.file, &item.site),
        );
    }
    for item in &mut model.mechanism_implementations {
        let (kind, address) = split_typed_binding(&item.binding, &item.lang);
        item.source = local_source(workspace, &item.file, kind, address);
    }
    for item in &mut model.class_members {
        item.source = local_source(
            workspace,
            &item.file,
            "class-member".into(),
            format!("{}#{}", item.class, item.site),
        );
    }
    for item in &mut model.enumerations {
        item.identity = local_source(
            workspace,
            &item.source,
            "enumerator".into(),
            format!("{}#{}", item.class, item.kind),
        );
    }
    for item in &mut model.artifacts {
        item.source = local_source(workspace, &item.file, item.kind.clone(), item.id.clone());
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

fn split_typed_binding(binding: &str, language: &str) -> (String, String) {
    binding
        .split_once(':')
        .map(|(kind, address)| (kind.to_string(), address.to_string()))
        .unwrap_or_else(|| (format!("{language}-symbol"), binding.to_string()))
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
    for design in &model.designs {
        warn_if_not_sibling(model, &design.spec, &design.path, "design", &mut warnings);
    }
    for verification in &model.verifications {
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
