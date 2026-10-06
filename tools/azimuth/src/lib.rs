//! Azimuth core model loading and selection.

pub mod account_delta;
pub mod account_design;
pub mod account_model;
pub mod account_support;
pub mod account_verification;
pub mod adapter;
pub mod adapter_host;
pub mod assurance;
pub mod assurance_review;
pub mod change;
pub mod design;
pub mod diag;
pub mod federation;
pub mod fingerprint;
pub mod installation;
pub mod intent;
pub mod json;
pub mod labels;
pub mod manifest;
pub mod model;
pub mod resources;
pub mod run;
pub mod run_plan;
pub mod run_selection;
pub mod spec;
pub mod traceability;
pub mod validation;
pub mod verification;
pub mod verification_packages;
pub mod workflow;
pub mod workspace;

use crate::diag::Diag;
use crate::model::{Criticality, Model, SourceIdentity};
use crate::verification::{Selector, Verification};
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
    load_with_support(
        model_dir,
        standards_path,
        workspace_path,
        manifests,
        &[],
        only,
    )
}

pub fn load_with_support(
    model_dir: &Path,
    standards_path: &Path,
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
    if standards_path.exists() {
        match verification::load_standards(standards_path) {
            Ok(standards) => model.decision_standards = Some(standards),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    match load_design_facets(model_dir) {
        Ok((legacy, accounts)) => {
            model.designs = legacy;
            model.account_designs = accounts;
        }
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    match load_verification_facets(model_dir) {
        Ok((legacy, accounts)) => {
            model.verifications = legacy;
            model.account_verifications = accounts;
        }
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    match load_review_facets(model_dir) {
        Ok(reviews) => model.account_reviews = reviews,
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }

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
    errors.extend(verification_owner_issues(&model));
    check_facet_owners(
        model
            .designs
            .iter()
            .map(|item| (&item.spec, &item.path))
            .chain(
                model
                    .account_designs
                    .iter()
                    .map(|item| (&item.spec, &item.path)),
            ),
        "design",
        &mut errors,
    );
    check_facet_owners(
        model
            .verifications
            .iter()
            .map(|item| (&item.owner, &item.path))
            .chain(
                model
                    .account_verifications
                    .iter()
                    .map(|item| (&item.owner, &item.path)),
            ),
        "verification",
        &mut errors,
    );
    check_facet_owners(
        model
            .account_reviews
            .iter()
            .map(|item| (&item.owner, &item.path)),
        "review",
        &mut errors,
    );
    errors.extend(model.entity_declaration_issues());
    errors.extend(model.verification_declaration_issues());
    errors.extend(model.account_check_issues());
    errors.extend(verification_packages::validate_model(&model));
    errors.extend(merged_manifest_issues(&model));
    errors.extend(mechanism_route_issues(&model));
    if !errors.is_empty() {
        return Err(errors);
    }

    if model.decision_standards.is_none() && needs_standards(&model) {
        warnings.push(Diag::file(
            &standards_path.display().to_string(),
            "no Decision Standards file; non-routine decisions cannot be resolved",
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
        match load_design_facets(root) {
            Ok((legacy, accounts)) => {
                extend_unique_facets(
                    &mut model.designs,
                    legacy,
                    |item| &item.spec,
                    |item| &item.path,
                    "design",
                    &mut errors,
                );
                extend_unique_facets(
                    &mut model.account_designs,
                    accounts,
                    |item| &item.spec,
                    |item| &item.path,
                    "account design",
                    &mut errors,
                );
            }
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        match load_verification_facets(root) {
            Ok((legacy, accounts)) => {
                extend_unique_facets(
                    &mut model.verifications,
                    legacy,
                    |item| &item.owner,
                    |item| &item.path,
                    "verification authority",
                    &mut errors,
                );
                extend_unique_facets(
                    &mut model.account_verifications,
                    accounts,
                    |item| &item.owner,
                    |item| &item.path,
                    "account verification",
                    &mut errors,
                );
            }
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        match load_review_facets(root) {
            Ok(reviews) => extend_unique_facets(
                &mut model.account_reviews,
                reviews,
                |item| &item.owner,
                |item| &item.path,
                "account review",
                &mut errors,
            ),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    if let Some(path) = &assembly.standards_path {
        match verification::load_standards(path) {
            Ok(standards) => model.decision_standards = Some(standards),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
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
    errors.extend(verification_owner_issues(&model));
    check_facet_owners(
        model
            .designs
            .iter()
            .map(|item| (&item.spec, &item.path))
            .chain(
                model
                    .account_designs
                    .iter()
                    .map(|item| (&item.spec, &item.path)),
            ),
        "design",
        &mut errors,
    );
    check_facet_owners(
        model
            .verifications
            .iter()
            .map(|item| (&item.owner, &item.path))
            .chain(
                model
                    .account_verifications
                    .iter()
                    .map(|item| (&item.owner, &item.path)),
            ),
        "verification",
        &mut errors,
    );
    check_facet_owners(
        model
            .account_reviews
            .iter()
            .map(|item| (&item.owner, &item.path)),
        "review",
        &mut errors,
    );
    errors.extend(model.entity_declaration_issues());
    errors.extend(model.verification_declaration_issues());
    errors.extend(model.account_check_issues());
    errors.extend(verification_packages::validate_model(&model));
    errors.extend(merged_manifest_issues(&model));
    errors.extend(mechanism_route_issues(&model));
    if !errors.is_empty() {
        return Err(errors);
    }

    if model.decision_standards.is_none() && needs_standards(&model) {
        warnings.push(Diag::file(
            "project",
            "Decision Standards are outside this assembly; non-routine decisions are incomplete",
        ));
    }
    warnings.extend(package_location_warnings(&model));
    apply_selection(&mut model, only);
    Ok(Loaded { model, warnings })
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

fn load_design_facets(
    root: &Path,
) -> Result<(Vec<design::Design>, Vec<account_design::AccountDesign>), Vec<Diag>> {
    let paths = named_paths(root, "design.md")?;
    let (mut legacy, mut accounts, mut errors) = (Vec::new(), Vec::new(), Vec::new());
    for path in paths {
        let display = path.display().to_string();
        match fs::read_to_string(&path) {
            Ok(source)
                if source
                    .lines()
                    .any(|line| line.starts_with("## Claim design: ")) =>
            {
                match account_design::parse_account_design(&display, &source) {
                    Ok(item) => accounts.push(item),
                    Err(mut diagnostics) => errors.append(&mut diagnostics),
                }
            }
            Ok(source) => match design::parse_design(&display, &source) {
                Ok(item) => legacy.push(item),
                Err(mut diagnostics) => errors.append(&mut diagnostics),
            },
            Err(error) => errors.push(Diag::file(&display, format!("cannot read design: {error}"))),
        }
    }
    check_facet_owners(
        legacy
            .iter()
            .map(|item| (&item.spec, &item.path))
            .chain(accounts.iter().map(|item| (&item.spec, &item.path))),
        "design",
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
) -> Result<
    (
        Vec<Verification>,
        Vec<account_verification::AccountVerification>,
    ),
    Vec<Diag>,
> {
    let paths = named_paths(root, "verification.md")?;
    let (mut legacy, mut accounts, mut errors) = (Vec::new(), Vec::new(), Vec::new());
    for path in paths {
        let display = path.display().to_string();
        match fs::read_to_string(&path) {
            Ok(source)
                if source
                    .lines()
                    .any(|line| line.starts_with("## Claim verification: ")) =>
            {
                match account_verification::parse_account_verification(&display, &source) {
                    Ok(item) => accounts.push(item),
                    Err(mut diagnostics) => errors.append(&mut diagnostics),
                }
            }
            Ok(_) => match verification::load_verification(&path) {
                Ok(item) => legacy.push(item),
                Err(mut diagnostics) => errors.append(&mut diagnostics),
            },
            Err(error) => errors.push(Diag::file(
                &display,
                format!("cannot read verification: {error}"),
            )),
        }
    }
    check_facet_owners(
        legacy
            .iter()
            .map(|item| (&item.owner, &item.path))
            .chain(accounts.iter().map(|item| (&item.owner, &item.path))),
        "verification",
        &mut errors,
    );
    if errors.is_empty() {
        Ok((legacy, accounts))
    } else {
        Err(errors)
    }
}

fn load_review_facets(root: &Path) -> Result<Vec<account_model::ReviewFacet>, Vec<Diag>> {
    let paths = named_paths(root, "judgments.md")?;
    let mut reviews = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match account_model::load_reviews(&path) {
            Ok(review) => reviews.push(review),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    if errors.is_empty() {
        Ok(reviews)
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

fn needs_standards(model: &Model) -> bool {
    model.cases().any(|claim| {
        matches!(
            claim.claim.criticality,
            Some(Criticality::Standard | Criticality::Critical)
        )
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
    let mut realizations = BTreeMap::<(String, String), (&str, &str)>::new();
    for realization in &model.realizes {
        let Some(source) = realization.source.as_ref() else {
            continue;
        };
        let key = (realization.claim.clone(), source.key());
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
    let mut selected_claims = model
        .cases()
        .filter(|claim| only.iter().any(|pattern| selects(pattern, &claim.spec.id)))
        .map(|claim| claim.id())
        .collect::<BTreeSet<_>>();
    let initial_bindings = model
        .evidence_bindings()
        .filter(|binding| selected_claims.contains(&binding.case))
        .map(|binding| binding.id.clone())
        .collect::<BTreeSet<_>>();
    let initial_checks = model
        .evidence_bindings()
        .filter(|binding| initial_bindings.contains(&binding.id))
        .map(|binding| binding.check.clone())
        .collect::<BTreeSet<_>>();
    let selected_plan_ids = model
        .challenge_plans()
        .filter(|plan| {
            crate::validation::challenge_plan_relevant_to_selection(
                model,
                plan,
                &selected_claims,
                &initial_bindings,
                &initial_checks,
            )
        })
        .map(|plan| plan.id.clone())
        .collect::<BTreeSet<_>>();
    let selected_plans = model
        .challenge_plans()
        .filter(|plan| selected_plan_ids.contains(&plan.id))
        .cloned()
        .collect::<Vec<_>>();
    let mut directly_selected_checks = BTreeSet::new();
    let mut anchored_realizations = BTreeSet::new();
    let mut anchored_mechanisms = BTreeSet::new();
    for plan in &selected_plans {
        for selector in &plan.selectors {
            selected_claims.extend(selector_claims(model, selector));
            match selector {
                Selector::MethodQualificationFromCheck(id)
                | Selector::ApplicabilityDecisionFromCheck(id) => {
                    directly_selected_checks.insert(id.clone());
                }
                Selector::MethodQualificationFromRealization(identity)
                | Selector::ApplicabilityDecisionFromRealization(identity)
                | Selector::ClaimJudgmentFromRealization(identity) => {
                    anchored_realizations.insert(identity.clone());
                }
                Selector::MethodQualificationFromMechanism(identity)
                | Selector::ApplicabilityDecisionFromMechanism(identity)
                | Selector::ClaimJudgmentFromMechanism(identity) => {
                    anchored_mechanisms.insert(identity.clone());
                }
                Selector::MethodQualificationFromMethodQualification(_)
                | Selector::ApplicabilityDecisionFromBinding(_)
                | Selector::ApplicabilityDecisionFromCase(_)
                | Selector::ClaimJudgmentFromClaim(_) => {}
            }
        }
    }
    let selected_parent_claims = selected_claims
        .iter()
        .filter_map(|case| model.find_case(case).map(|view| view.claim.id.clone()))
        .collect::<BTreeSet<_>>();
    // Retaining the parent composition prevents partial selection from inventing a narrower assurance account.
    selected_claims.extend(
        model
            .cases()
            .filter(|case| selected_parent_claims.contains(&case.claim.id.clone()))
            .map(|case| case.id()),
    );
    let selected_bindings = model
        .evidence_bindings()
        .filter(|binding| selected_claims.contains(&binding.case))
        .map(|binding| binding.id.clone())
        .collect::<BTreeSet<_>>();
    let mut selected_checks = model
        .evidence_bindings()
        .filter(|binding| selected_bindings.contains(&binding.id))
        .map(|binding| binding.check.clone())
        .collect::<BTreeSet<_>>();
    selected_checks.extend(directly_selected_checks);
    let selected_specs = model
        .cases()
        .filter(|claim| selected_claims.contains(&claim.id()))
        .map(|claim| claim.spec.id.clone())
        .collect::<BTreeSet<_>>();
    let selected_surfaces = model
        .cases()
        .filter(|claim| selected_claims.contains(&claim.id()))
        .filter_map(|claim| claim.claim.over.clone())
        .collect::<BTreeSet<_>>();
    let mut retained_mechanisms = anchored_mechanisms;
    for claim in model
        .cases()
        .filter(|claim| selected_claims.contains(&claim.id()))
    {
        if let Some(design) = model.design_for(&claim.spec.id) {
            for entry in &design.entries {
                let attached = entry.target.id() == claim.claim.id;
                if attached {
                    retained_mechanisms.extend(
                        entry
                            .mechanisms
                            .iter()
                            .filter(|mechanism| {
                                mechanism.cases.is_empty()
                                    || mechanism.cases.contains(&claim.case.id)
                            })
                            .map(|mechanism| mechanism.id.clone()),
                    );
                }
            }
        }
    }
    let mut selected_artifacts = BTreeSet::new();
    for design in &model.designs {
        for mechanism in design.entries.iter().flat_map(|entry| &entry.mechanisms) {
            let identity = mechanism.id.clone();
            if retained_mechanisms.contains(&identity) {
                selected_artifacts.extend(
                    model
                        .mechanism_bindings(&design.spec, mechanism)
                        .into_iter()
                        .map(str::to_string),
                );
            }
        }
    }
    let selected_challengers = model
        .challenge_plans()
        .filter(|plan| selected_plan_ids.contains(&plan.id))
        .map(|plan| plan.challenger.clone())
        .collect::<BTreeSet<_>>();
    let retained_sources = model
        .realizes
        .iter()
        .filter(|site| {
            selected_parent_claims.contains(&site.claim.clone())
                || selected_surfaces.contains(&site.spec)
                || site
                    .source
                    .as_ref()
                    .is_some_and(|source| anchored_realizations.contains(&source.key()))
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
        .filter(|obligation| selected_parent_claims.contains(&obligation.claim.clone()))
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
        spec.claims.retain_mut(|claim| {
            claim
                .cases
                .retain(|case| selected_claims.contains(&case.id.clone()));
            !claim.cases.is_empty()
        });
        !spec.claims.is_empty()
    });
    model
        .account_designs
        .retain(|design| selected_specs.contains(&design.spec));
    model
        .account_verifications
        .retain(|item| selected_specs.contains(&item.owner));
    model
        .account_reviews
        .retain(|item| selected_specs.contains(&item.owner));
    model.designs.retain_mut(|design| {
        design.entries.retain_mut(|entry| {
            entry
                .mechanisms
                .retain(|mechanism| retained_mechanisms.contains(&mechanism.id.clone()));
            !entry.mechanisms.is_empty()
        });
        !design.entries.is_empty()
    });
    model.realizes.retain(|site| {
        selected_parent_claims.contains(&site.claim.clone())
            || selected_surfaces.contains(&site.spec)
            || site
                .source
                .as_ref()
                .is_some_and(|source| anchored_realizations.contains(&source.key()))
    });
    model
        .mechanism_implementations
        .retain(|implementation| retained_mechanisms.contains(&implementation.mechanism));
    model
        .workspace
        .realization_obligations
        .retain(|obligation| selected_parent_claims.contains(&obligation.claim.clone()));
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
        let selected_method_qualifications = file
            .bindings
            .iter()
            .map(|binding| binding.method_qualification.clone())
            .collect::<BTreeSet<_>>();
        file.method_qualifications
            .retain(|qualification| selected_method_qualifications.contains(&qualification.id));
        file.applicability_decisions
            .retain(|decision| selected_bindings.contains(&decision.id));
        file.claim_judgments
            .retain(|judgment| selected_parent_claims.contains(&judgment.id));
        file.challengers
            .retain(|challenger| selected_challengers.contains(&challenger.id));
        file.challenge_plans
            .retain(|plan| selected_plan_ids.contains(&plan.id));
    }
    model.verifications.retain(|file| {
        !file.checks.is_empty()
            || !file.bindings.is_empty()
            || !file.method_qualifications.is_empty()
            || !file.applicability_decisions.is_empty()
            || !file.claim_judgments.is_empty()
            || !file.challengers.is_empty()
            || !file.challenge_plans.is_empty()
    });
}

fn selector_claims(model: &Model, selector: &Selector) -> BTreeSet<String> {
    match selector {
        Selector::ApplicabilityDecisionFromBinding(id) => model
            .evidence_bindings()
            .filter(|binding| binding.id == *id)
            .map(|binding| binding.case.clone())
            .collect(),
        Selector::MethodQualificationFromMethodQualification(id) => model
            .evidence_bindings()
            .filter(|binding| binding.method_qualification == *id)
            .map(|binding| binding.case.clone())
            .collect(),
        Selector::MethodQualificationFromCheck(id)
        | Selector::ApplicabilityDecisionFromCheck(id) => model
            .evidence_bindings()
            .filter(|binding| binding.check == *id)
            .map(|binding| binding.case.clone())
            .collect(),
        Selector::MethodQualificationFromRealization(identity)
        | Selector::ApplicabilityDecisionFromRealization(identity)
        | Selector::ClaimJudgmentFromRealization(identity) => model
            .realizes
            .iter()
            .filter(|site| {
                site.source
                    .as_ref()
                    .is_some_and(|source| source.key() == *identity)
            })
            .filter(|site| model.has_claim(&site.claim))
            .flat_map(|site| {
                model
                    .find_claim(&site.claim)
                    .into_iter()
                    .flat_map(move |claim| {
                        claim.claim.cases.iter().map(move |case| case.id.clone())
                    })
            })
            .collect(),
        Selector::MethodQualificationFromMechanism(identity)
        | Selector::ApplicabilityDecisionFromMechanism(identity)
        | Selector::ClaimJudgmentFromMechanism(identity) => {
            selected_mechanism_claims(model, identity)
        }
        Selector::ApplicabilityDecisionFromCase(id) => model
            .find_case(id)
            .map(|case| BTreeSet::from([case.id()]))
            .unwrap_or_default(),
        Selector::ClaimJudgmentFromClaim(id) => model
            .claims()
            .find(|claim| claim.id() == *id)
            .map(|claim| {
                claim
                    .claim
                    .cases
                    .iter()
                    .map(|case| case.id.clone())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn selected_mechanism_claims(model: &Model, identity: &str) -> BTreeSet<String> {
    let mechanism_id = identity;
    let Some(design) = model.designs.iter().find(|design| {
        design.entries.iter().any(|entry| {
            entry
                .mechanisms
                .iter()
                .any(|mechanism| mechanism.id == identity)
        })
    }) else {
        return BTreeSet::new();
    };
    model
        .cases()
        .filter(|claim| {
            design
                .entries
                .iter()
                .filter(|entry| entry.target.id() == claim.claim.id)
                .any(|entry| {
                    entry.mechanisms.iter().any(|item| {
                        item.id == mechanism_id
                            && (item.cases.is_empty() || item.cases.contains(&claim.case.id))
                    })
                })
        })
        .map(|claim| claim.id())
        .collect()
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
        for design in &model.designs {
            for mechanism in design.entries.iter().flat_map(|entry| &entry.mechanisms) {
                if mechanism
                    .binding
                    .as_ref()
                    .is_some_and(|binding| binding == &raw || binding == &assembled)
                {
                    issues.push(Diag::file(
                        &design.path,
                        format!(
                            "explicit mechanism `{}#{}` may not bind marker-only Artifact `{}`",
                            design.spec,
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
    mechanism: &'a crate::design::Mechanism,
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

#[cfg(test)]
mod tests {
    use super::{needs_standards, selected_mechanism_claims};
    use crate::design::{Design, DesignEntry, Enforcement, Mechanism, Target};
    use crate::model::Model;

    fn mechanism(id: &str) -> Mechanism {
        Mechanism {
            id: id.into(),
            kind: Enforcement::Guard,
            cases: Vec::new(),
            binding: Some(format!("{id}-artifact")),
            expected_unique: None,
            expected_columns: Vec::new(),
            expected_predicate: None,
            line: 1,
        }
    }

    #[test]
    fn mechanism_selection_closure_reads_every_matching_design_entry() {
        let spec = crate::spec::parse_spec(
            "spec.md",
            "# Spec: example\n\n## Claim: works\nCriticality: standard\n\nThe example \
             SHALL work.\n\n### Case: succeeds\nIt succeeds when invoked.\n",
        )
        .unwrap();
        let model = Model {
            specs: vec![spec],
            designs: vec![Design {
                spec: "example".into(),
                path: "design.md".into(),
                entries: vec![
                    DesignEntry {
                        target: Target::Claim("works".into()),
                        mechanisms: vec![mechanism("first")],
                        line: 1,
                    },
                    DesignEntry {
                        target: Target::Claim("works".into()),
                        mechanisms: vec![mechanism("second")],
                        line: 2,
                    },
                ],
                residue: String::new(),
            }],
            ..Default::default()
        };

        assert_eq!(
            selected_mechanism_claims(&model, "example#second"),
            ["example#works/succeeds".to_string()].into_iter().collect()
        );
    }

    #[test]
    fn every_nonroutine_claim_needs_decision_standards_even_without_declarations() {
        let spec = crate::spec::parse_spec(
            "spec.md",
            "# Spec: example\n\n## Claim: works\nCriticality: standard\n\nThe example \
             SHALL work.\n\n### Case: succeeds\nIt succeeds when invoked.\n",
        )
        .unwrap();
        assert!(needs_standards(&Model {
            specs: vec![spec],
            ..Default::default()
        }));
    }
}
