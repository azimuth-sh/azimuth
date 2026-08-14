//! Azimuth core.
//!
//! Reads claims, linkage tags and (later) evidence; derives a model; runs checks over it. The
//! model is a first-class artifact (D10): checks, dashboards, PR annotations and the agent tier
//! are all consumers of the export, and nothing else re-parses specs.
//!
//! No dependencies, by decision (D17).

pub mod assurance;
pub mod change;
pub mod check;
pub mod design;
pub mod diag;
pub mod federation;
pub mod fingerprint;
pub mod json;
pub mod judgment;
pub mod labels;
pub mod manifest;
pub mod model;
pub mod plan;
pub mod spec;
pub mod workflow;
pub mod workspace;

use diag::Diag;
use model::Model;
use std::path::Path;

pub struct Loaded {
    pub model: Model,
    pub warnings: Vec<Diag>,
}

/// Selection operates on **ids**, not paths — the hierarchy lives in the id string, so scoping
/// keeps working if the folders are reorganized tomorrow.
///
/// `billing/**` matches every spec whose id starts with `billing/`; anything else matches exactly.
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
    manifests: &[std::path::PathBuf],
    only: &[String],
) -> Result<Loaded, Vec<Diag>> {
    let loaded = spec::load_specs(model_dir)?;
    let mut model = Model {
        specs: loaded.specs,
        ..Default::default()
    };
    let mut warnings = loaded.warnings;

    if !only.is_empty() {
        model
            .specs
            .retain(|s| only.iter().any(|p| selects(p, &s.id)));
    }

    let mut errors = Vec::new();

    match workspace::load(workspace_path) {
        Ok(workspace) => model.workspace = workspace,
        Err(mut diagnostics) => errors.append(&mut diagnostics),
    }
    if !only.is_empty() {
        model
            .workspace
            .realization_obligations
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
    }

    // Routine is intrinsically intent-only. Other levels need the project mapping before a clean
    // result can mean that the required evidence form was checked.
    if standards_path.exists() {
        match plan::load_standards(standards_path) {
            Ok(s) => model.standards = Some(s),
            Err(mut d) => errors.append(&mut d),
        }
    }

    match judgment::load(model_dir) {
        Ok(js) => {
            model.judgments = js;
            if !only.is_empty() {
                model
                    .judgments
                    .retain(|j| only.iter().any(|p| selects(p, &j.spec)));
            }
        }
        Err(mut d) => errors.append(&mut d),
    }

    match design::load_designs(model_dir) {
        Ok(designs) => {
            model.designs = designs;
            if !only.is_empty() {
                model
                    .designs
                    .retain(|d| only.iter().any(|pat| selects(pat, &d.spec)));
            }
        }
        Err(mut d) => errors.append(&mut d),
    }

    match plan::load_plans(model_dir) {
        Ok(plans) => {
            model.plans = plans;
            if !only.is_empty() {
                model
                    .plans
                    .retain(|p| only.iter().any(|pat| selects(pat, &p.spec)));
            }
        }
        Err(mut d) => errors.append(&mut d),
    }

    for path in manifests {
        match manifest::load(path) {
            Ok(m) => {
                model.realizes.extend(m.realizes);
                model.covers.extend(m.covers);
                model
                    .mechanism_implementations
                    .extend(m.mechanism_implementations);
                model.mechanism_covers.extend(m.mechanism_covers);
                model.class_members.extend(m.class_members);
                model.enumerations.extend(m.enumerations);
                model.artifacts.extend(m.artifacts);
                model
                    .covers
                    .extend(m.observations.iter().flat_map(|item| item.evidence_sites()));
                model.observations.extend(m.observations);
            }
            Err(mut d) => errors.append(&mut d),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    if !only.is_empty() {
        model
            .realizes
            .retain(|s| only.iter().any(|p| selects(p, &s.spec)));
        model
            .covers
            .retain(|s| only.iter().any(|p| selects(p, &s.spec)));
        model
            .mechanism_implementations
            .retain(|s| only.iter().any(|p| selects(p, &s.spec)));
        model
            .mechanism_covers
            .retain(|s| only.iter().any(|p| selects(p, &s.spec)));
        for observation in &mut model.observations {
            observation
                .bindings
                .retain(|binding| only.iter().any(|p| selects(p, &binding.spec)));
        }
        model
            .observations
            .retain(|observation| !observation.bindings.is_empty());
    }

    if model.standards.is_none() && needs_standards(&model) {
        warnings.push(Diag::file(
            &standards_path.display().to_string(),
            "no standards file; evidence-form checks are incomplete for non-routine claims",
        ));
    }

    warnings.extend(package_location_warnings(&model));

    Ok(Loaded { model, warnings })
}

/// Loads the model assembled from a multi-repository workset. Model sources remain independently
/// owned directories; concatenating them into a synthetic tree would hide duplicate ownership and
/// make physical checkout layout part of identity.
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
                for spec in loaded.specs {
                    if let Some(previous) = model.specs.iter().find(|item| item.id == spec.id) {
                        errors.push(Diag::at(
                            &spec.path,
                            1,
                            format!(
                                "model-source-ownership-conflict: spec `{}` is already declared by {}",
                                spec.id, previous.path
                            ),
                        ));
                    } else {
                        model.specs.push(spec);
                    }
                }
            }
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
        match judgment::load(root) {
            Ok(items) => extend_unique_facets(
                &mut model.judgments,
                items,
                |item| &item.spec,
                |item| &item.path,
                "judgments",
                &mut errors,
            ),
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
        match plan::load_plans(root) {
            Ok(items) => extend_unique_facets(
                &mut model.plans,
                items,
                |item| &item.spec,
                |item| &item.path,
                "verification",
                &mut errors,
            ),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }

    if let Some(path) = &assembly.standards_path {
        match plan::load_standards(path) {
            Ok(standards) => model.standards = Some(standards),
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }

    for manifest in &assembly.manifests {
        append_manifest(&mut model, manifest);
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    if !only.is_empty() {
        model
            .specs
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.id)));
        model
            .judgments
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
        model
            .designs
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
        model
            .plans
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
        model
            .realizes
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
        model
            .covers
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
        model
            .mechanism_implementations
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
        model
            .mechanism_covers
            .retain(|item| only.iter().any(|pattern| selects(pattern, &item.spec)));
        for observation in &mut model.observations {
            observation
                .bindings
                .retain(|binding| only.iter().any(|pattern| selects(pattern, &binding.spec)));
        }
        model
            .observations
            .retain(|observation| !observation.bindings.is_empty());
    }
    if model.standards.is_none() && needs_standards(&model) {
        warnings.push(Diag::file(
            "project",
            "standards input is outside this local assembly; evidence-form checks are incomplete for non-routine claims",
        ));
    }
    warnings.extend(package_location_warnings(&model));
    Ok(Loaded { model, warnings })
}

fn needs_standards(model: &Model) -> bool {
    model
        .claims()
        .any(|claim| claim.requirement.criticality != Some(model::Criticality::Routine))
}

fn append_manifest(model: &mut Model, manifest: &manifest::Manifest) {
    model.realizes.extend(manifest.realizes.clone());
    model.covers.extend(manifest.covers.clone());
    model
        .mechanism_implementations
        .extend(manifest.mechanism_implementations.clone());
    model
        .mechanism_covers
        .extend(manifest.mechanism_covers.clone());
    model.class_members.extend(manifest.class_members.clone());
    model.enumerations.extend(manifest.enumerations.clone());
    model.artifacts.extend(manifest.artifacts.clone());
    model.covers.extend(
        manifest
            .observations
            .iter()
            .flat_map(|item| item.evidence_sites()),
    );
    model.observations.extend(manifest.observations.clone());
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
                    "model-source-ownership-conflict: {kind} for `{}` is already declared by {}",
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
    for plan in &model.plans {
        warn_if_not_sibling(
            model,
            &plan.spec,
            &plan.path,
            "verification plan",
            &mut warnings,
        );
    }
    for judgments in &model.judgments {
        warn_if_not_sibling(
            model,
            &judgments.spec,
            &judgments.path,
            "judgments",
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
