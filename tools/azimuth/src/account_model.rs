//! Structural assembly of Requirement-first account documents and available source linkage.
//!
//! A finding records an unresolved relationship. This module does not judge a Requirement, qualify a
//! method, interpret a Run, or establish deployment Assurance State.

use crate::account_realization::{self, AccountRealization, Declaration, DeclarationKind};
use crate::account_support::AccountSupport;
use crate::account_verification::{self, AccountVerification};
use crate::diag::Diag;
use crate::manifest::Manifest;
use crate::model::{Domain, Spec};
use crate::spec;
use crate::workspace::Workspace;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountFindingKind {
    UnknownRealizationRequirement,
    UnknownRealizationScenario,
    UnknownMechanismScenario,
    UnknownVerificationRequirement,
    UnknownVerificationScenario,
    DuplicateFacet,
    MissingRequirementVerification,
    UnknownArea,
    MissingAreaRealization,
    VerificationElementSupportUnresolved,
    ScenarioCheckContributionUnresolved,
    CheckWithoutScenarioContribution,
    MechanismSupportUnresolved,
    CheckImplementationUnresolved,
}

impl AccountFindingKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::UnknownRealizationRequirement => "unknown-realization-requirement",
            Self::UnknownRealizationScenario => "unknown-realization-scenario",
            Self::UnknownMechanismScenario => "unknown-mechanism-scenario",
            Self::UnknownVerificationRequirement => "unknown-verification-requirement",
            Self::UnknownVerificationScenario => "unknown-verification-scenario",
            Self::DuplicateFacet => "duplicate-account-facet",
            Self::MissingRequirementVerification => "missing-requirement-verification",
            Self::UnknownArea => "unknown-realization-area",
            Self::MissingAreaRealization => "missing-area-realization",
            Self::VerificationElementSupportUnresolved => "verification-element-support-unresolved",
            Self::ScenarioCheckContributionUnresolved => "scenario-check-contribution-unresolved",
            Self::CheckWithoutScenarioContribution => "check-without-scenario-contribution",
            Self::MechanismSupportUnresolved => "mechanism-support-unresolved",
            Self::CheckImplementationUnresolved => "check-implementation-unresolved",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountFinding {
    pub kind: AccountFindingKind,
    pub path: String,
    pub line: usize,
    pub requirement: Option<String>,
    pub detail: String,
}

impl AccountFinding {
    fn new(
        kind: AccountFindingKind,
        path: &str,
        line: usize,
        requirement: Option<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            path: path.to_string(),
            line,
            requirement,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct AccountReport {
    pub specs: usize,
    pub requirements: usize,
    pub scenarios: usize,
    pub realization_facets: usize,
    pub verification_facets: usize,
    pub realizations: usize,
    pub support_artifacts: usize,
    pub support_checks: usize,
    pub scenario_contributions: usize,
    pub findings: Vec<AccountFinding>,
}

impl AccountReport {
    pub fn render(&self) -> String {
        let mut output = format!(
            "Account inspection: {} specs, {} requirements, {} scenarios, {} realization facets, {} verification facets, {} realizations, {} support artifacts, {} Checks, {} Scenario contributions, {} unresolved findings\n",
            self.specs, self.requirements, self.scenarios, self.realization_facets,
            self.verification_facets, self.realizations, self.support_artifacts, self.support_checks, self.scenario_contributions, self.findings.len()
        );
        for finding in &self.findings {
            let location = if finding.line == 0 {
                finding.path.clone()
            } else {
                format!("{}:{}", finding.path, finding.line)
            };
            let _ = writeln!(
                output,
                "{}: {}: {}",
                location,
                finding.kind.name(),
                finding.detail
            );
        }
        output
    }
}

/// Inspect a complete or projected model root. The caller supplies the corresponding workspace
/// and extracted manifests; no positive verification or Assurance State is inferred from them.
pub fn inspect(
    model_root: &Path,
    workspace: &Workspace,
    manifests: &[Manifest],
) -> Result<AccountReport, Vec<Diag>> {
    inspect_with_support(model_root, workspace, manifests, &[])
}

pub fn inspect_with_support(
    model_root: &Path,
    workspace: &Workspace,
    manifests: &[Manifest],
    supports: &[AccountSupport],
) -> Result<AccountReport, Vec<Diag>> {
    let loaded = spec::load_specs(model_root)?;
    let mut errors = Vec::new();
    let mut files = Vec::new();
    collect_facets(model_root, &mut files).map_err(|error| {
        vec![Diag::file(
            &model_root.display().to_string(),
            format!("cannot enumerate account facets: {error}"),
        )]
    })?;
    files.sort();
    let mut realizations = BTreeMap::<String, AccountRealization>::new();
    let mut verifications = BTreeMap::<String, AccountVerification>::new();
    let mut report = AccountReport {
        specs: loaded.specs.len(),
        requirements: loaded
            .specs
            .iter()
            .map(|spec| spec.requirements.len())
            .sum(),
        scenarios: loaded
            .specs
            .iter()
            .flat_map(|spec| &spec.requirements)
            .map(|requirement| requirement.scenarios.len())
            .sum(),
        realizations: manifests
            .iter()
            .map(|manifest| manifest.realizes.len())
            .sum(),
        support_artifacts: supports.iter().map(|support| support.artifacts.len()).sum(),
        support_checks: supports.iter().map(|support| support.checks.len()).sum(),
        scenario_contributions: supports
            .iter()
            .map(|support| support.contributions.len())
            .sum(),
        ..Default::default()
    };
    for path in files {
        let display = path.display().to_string();
        match path.file_name().and_then(|name| name.to_str()) {
            Some("realization.md") if uses_account_format(&path, "## Requirement realization")? => {
                match account_realization::load_account_realization(&path) {
                    Ok(realization) => {
                        if let Some(previous) =
                            realizations.insert(realization.spec.clone(), realization)
                        {
                            report.findings.push(AccountFinding::new(
                                AccountFindingKind::DuplicateFacet,
                                &display,
                                1,
                                None,
                                format!("realization module also declared by {}", previous.path),
                            ));
                        }
                    }
                    Err(mut diagnostics) => errors.append(&mut diagnostics),
                }
            }
            Some("verification.md") => {
                match account_verification::load_account_verification(&path) {
                    Ok(verification) => {
                        if let Some(previous) =
                            verifications.insert(verification.owner.clone(), verification)
                        {
                            report.findings.push(AccountFinding::new(
                                AccountFindingKind::DuplicateFacet,
                                &display,
                                1,
                                None,
                                format!("verification module also declared by {}", previous.path),
                            ));
                        }
                    }
                    Err(mut diagnostics) => errors.append(&mut diagnostics),
                }
            }
            _ => {}
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    report.realization_facets = realizations.len();
    report.verification_facets = verifications.len();
    let (legacy_realizations, _) = crate::load_realization_facets(model_root)?;
    let known_mechanisms = mechanism_ids(realizations.values(), &legacy_realizations);
    let check_errors = authored_check_issues(
        verifications.values(),
        &loaded.specs,
        supports,
        &known_mechanisms,
    );
    if !check_errors.is_empty() {
        return Err(check_errors);
    }
    report.support_checks += verifications
        .values()
        .map(|file| file.checks.len())
        .sum::<usize>();
    report.scenario_contributions += verifications
        .values()
        .flat_map(|file| &file.checks)
        .map(|check| check.scenarios.len())
        .sum::<usize>();
    let definitions = crate::model::Model {
        specs: loaded.specs.clone(),
        account_realizations: realizations.values().cloned().collect(),
        account_verifications: verifications.values().cloned().collect(),
        ..Default::default()
    };
    let identity_issues = definitions.entity_declaration_issues();
    if !identity_issues.is_empty() {
        return Err(identity_issues);
    }
    validate_supports(supports, &loaded.specs, &verifications)?;
    for support in supports {
        for check in &support.checks {
            if !supports
                .iter()
                .flat_map(|source| &source.contributions)
                .any(|contribution| contribution.check == check.id)
            {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::CheckWithoutScenarioContribution,
                    &support.path,
                    0,
                    None,
                    format!("Check `{}` has no explicit Scenario contribution", check.id),
                ));
            }
        }
    }
    for realization in realizations.values() {
        inspect_realization(
            realization,
            &loaded.specs,
            workspace,
            manifests,
            &mut report,
        );
    }
    let package_accounts = verifications.values().cloned().collect::<Vec<_>>();
    let mut package_producers = manifests
        .iter()
        .flat_map(|manifest| manifest.extensions.iter().cloned())
        .collect::<Vec<_>>();
    for producer in &mut package_producers {
        if producer.source.is_none() {
            producer.source = crate::local_source(
                workspace,
                &producer.file,
                crate::address_kind(&producer.lang, &producer.file, &producer.site),
                crate::address_value(&producer.lang, &producer.file, &producer.site),
            );
        }
    }
    errors.extend(crate::verification_packages::validate_accounts(
        &package_accounts,
        workspace,
        &package_producers,
    ));
    for account in &package_accounts {
        for entity in &account.package_entities {
            let contract = match entity.kind.as_str() {
                "Surface" => "surface-enumerator",
                "Expectation set" => "surface-expectations",
                _ => "probe-implementation",
            };
            if !package_producers.iter().any(|producer| {
                producer.package == entity.package()
                    && producer.contract == contract
                    && producer.entity == entity.id
                    && producer.source.is_some()
            }) {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::CheckImplementationUnresolved,
                    &account.path,
                    entity.line,
                    None,
                    format!(
                        "{} `{}` has no source-linked producer",
                        entity.kind, entity.id
                    ),
                ));
            }
        }
    }
    for account in verifications.values() {
        for verification_challenge in &account.verification_challenges {
            if !workspace
                .packages
                .iter()
                .any(|package| package == "azimuth.challenge")
            {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::CheckImplementationUnresolved,
                    &account.path,
                    verification_challenge.line,
                    Some(verification_challenge.id.clone()),
                    "Verification Challenge package azimuth.challenge is not enabled",
                ));
            }
            let implementations = package_producers
                .iter()
                .filter(|producer| {
                    producer.package == "azimuth.challenge"
                        && producer.contract == "verification-challenge-implementation"
                        && producer.entity == verification_challenge.id
                })
                .collect::<Vec<_>>();
            if implementations.len() != 1
                || implementations
                    .iter()
                    .any(|producer| producer.source.is_none())
            {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::CheckImplementationUnresolved,
                    &account.path,
                    verification_challenge.line,
                    Some(verification_challenge.id.clone()),
                    "Verification Challenge must resolve exactly one qualified package implementation",
                ));
            }
        }
    }
    let authored_checks = verifications
        .values()
        .flat_map(|file| &file.checks)
        .collect::<Vec<_>>();
    let normalized_checks = manifests
        .iter()
        .flat_map(|manifest| &manifest.check_implementations)
        .cloned()
        .map(|mut item| {
            item.source = crate::local_source(
                workspace,
                &item.file,
                crate::address_kind(&item.lang, &item.file, &item.site),
                crate::address_value(&item.lang, &item.file, &item.site),
            );
            item
        })
        .collect::<Vec<_>>();
    for verification in verifications.values() {
        inspect_verification(
            verification,
            &loaded.specs,
            supports,
            &authored_checks,
            &normalized_checks,
            &mut report,
        );
    }
    for spec in &loaded.specs {
        if !realizations.contains_key(&spec.id) && !verifications.contains_key(&spec.id) {
            continue;
        }
        for requirement in &spec.requirements {
            if requirement.verification.is_none() {
                continue;
            }
            let identity = requirement.id.clone();
            if !verifications.get(&spec.id).is_some_and(|verification| {
                verification
                    .requirements
                    .iter()
                    .any(|scope| scope.requirement == identity)
            }) {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::MissingRequirementVerification,
                    &spec.path,
                    requirement.line,
                    Some(identity.clone()),
                    format!("Requirement `{identity}` declares `Verification:` but has no Requirement verification"),
                ));
            }
        }
    }
    report.findings.sort_by(|left, right| {
        (&left.path, left.line, left.kind.name(), &left.detail).cmp(&(
            &right.path,
            right.line,
            right.kind.name(),
            &right.detail,
        ))
    });
    let structural = report
        .findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.kind,
                AccountFindingKind::UnknownRealizationRequirement
                    | AccountFindingKind::UnknownRealizationScenario
                    | AccountFindingKind::UnknownMechanismScenario
                    | AccountFindingKind::UnknownVerificationRequirement
                    | AccountFindingKind::UnknownVerificationScenario
                    | AccountFindingKind::DuplicateFacet
                    | AccountFindingKind::UnknownArea
            )
        })
        .map(|finding| Diag::at(&finding.path, finding.line, finding.detail.clone()))
        .collect::<Vec<_>>();
    if !structural.is_empty() {
        return Err(structural);
    }
    Ok(report)
}

fn collect_facets(root: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_facets(&path, paths)?;
        } else if matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("realization.md" | "verification.md")
        ) {
            paths.push(path);
        }
    }
    Ok(())
}

fn requirement<'a>(
    specs: &'a [Spec],
    identity: &str,
) -> Option<(&'a Spec, &'a crate::model::Requirement)> {
    specs.iter().find_map(|spec| {
        spec.requirements
            .iter()
            .find(|requirement| requirement.id == identity)
            .map(|requirement| (spec, requirement))
    })
}

fn find_scenario_by_id<'a>(
    specs: &'a [Spec],
    identity: &str,
) -> Option<(
    &'a Spec,
    &'a crate::model::Requirement,
    &'a crate::model::Scenario,
)> {
    specs.iter().find_map(|spec| {
        spec.requirements.iter().find_map(|requirement| {
            requirement
                .scenarios
                .iter()
                .find(|scenario| scenario.id == identity)
                .map(|scenario| (spec, requirement, scenario))
        })
    })
}

fn inspect_realization(
    realization: &AccountRealization,
    specs: &[Spec],
    workspace: &Workspace,
    manifests: &[Manifest],
    report: &mut AccountReport,
) {
    for item in &realization.declarations {
        inspect_realization_item(realization, item, specs, workspace, manifests, None, report);
    }
}

fn inspect_realization_item(
    realization: &AccountRealization,
    item: &Declaration,
    specs: &[Spec],
    workspace: &Workspace,
    manifests: &[Manifest],
    enclosing_requirement: Option<&str>,
    report: &mut AccountReport,
) {
    match item.kind {
        DeclarationKind::Requirement => {
            let identity = item.id.clone();
            match requirement(specs, &identity) {
                None => report.findings.push(AccountFinding::new(
                    AccountFindingKind::UnknownRealizationRequirement,
                    &realization.path,
                    item.line,
                    Some(identity.clone()),
                    format!("Requirement realization `{identity}` names no spec Requirement"),
                )),
                Some((_, target)) if target.domain != Domain::Behaviour => {
                    report.findings.push(AccountFinding::new(AccountFindingKind::UnknownRealizationRequirement,
                        &realization.path, item.line, Some(identity.clone()),
                        format!("Requirement realization `{identity}` does not target a behavioral Requirement")));
                }
                Some(_) => {}
            }
            for area in &item.areas {
                if !workspace.areas.iter().any(|declared| declared.id == *area) {
                    report.findings.push(AccountFinding::new(
                        AccountFindingKind::UnknownArea,
                        &realization.path,
                        item.line,
                        Some(identity.clone()),
                        format!(
                            "Requirement realization `{identity}` names undeclared Area `{area}`"
                        ),
                    ));
                    continue;
                }
                let requirement_id = identity.as_str();
                let realized = manifests
                    .iter()
                    .flat_map(|manifest| &manifest.realizes)
                    .any(|site| {
                        site.requirement == requirement_id
                            && site
                                .source
                                .as_ref()
                                .map(|source| source.area.as_str())
                                .or_else(|| {
                                    workspace
                                        .area_for_file(&site.file)
                                        .map(|found| found.id.as_str())
                                })
                                == Some(area.as_str())
                    });
                if !realized {
                    report.findings.push(AccountFinding::new(AccountFindingKind::MissingAreaRealization,
                        &realization.path, item.line, Some(identity.clone()),
                        format!("Requirement `{identity}` has no extracted Realizes site in required Area `{area}`")));
                }
            }
        }
        DeclarationKind::Scenario => {
            if !find_scenario_by_id(specs, &item.id).is_some_and(|(_, requirement, _)| {
                enclosing_requirement == Some(requirement.id.as_str())
            }) {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::UnknownRealizationScenario,
                    &realization.path,
                    item.line,
                    None,
                    format!("Scenario realization `{}` names no spec Scenario", item.id),
                ));
            }
        }
        DeclarationKind::Element => {
            for area in &item.areas {
                if !workspace.areas.iter().any(|declared| declared.id == *area) {
                    report.findings.push(AccountFinding::new(
                        AccountFindingKind::UnknownArea,
                        &realization.path,
                        item.line,
                        None,
                        format!("Element `{}` names undeclared Area `{area}`", item.id),
                    ));
                }
            }
        }
        DeclarationKind::Mechanism => {
            for scenario in &item.scenarios {
                if find_scenario_by_id(specs, scenario).is_none() {
                    report.findings.push(AccountFinding::new(
                        AccountFindingKind::UnknownMechanismScenario,
                        &realization.path,
                        item.line,
                        None,
                        format!(
                            "Mechanism `{}` names unknown Scenario `{scenario}`",
                            item.id
                        ),
                    ));
                }
            }
            let local_id = item.id.as_str();
            if !manifests
                .iter()
                .flat_map(|manifest| &manifest.mechanism_implementations)
                .any(|implementation| implementation.mechanism == local_id)
            {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::MechanismSupportUnresolved,
                    &realization.path,
                    item.line,
                    None,
                    format!(
                        "selected Mechanism `{}` has no extracted implementation",
                        item.id
                    ),
                ));
            }
        }
        DeclarationKind::Section => {}
    }
    for child in &item.children {
        let owner = if item.kind == DeclarationKind::Requirement {
            Some(item.id.as_str())
        } else {
            enclosing_requirement
        };
        inspect_realization_item(
            realization,
            child,
            specs,
            workspace,
            manifests,
            owner,
            report,
        );
    }
}

fn inspect_verification(
    verification: &AccountVerification,
    specs: &[Spec],
    supports: &[AccountSupport],
    authored_checks: &[&crate::account_verification::AuthoredCheck],
    implementations: &[crate::model::CheckImplementation],
    report: &mut AccountReport,
) {
    for check in &verification.checks {
        if !check_has_implementation(check, implementations.iter()) {
            report.findings.push(AccountFinding::new(
                AccountFindingKind::CheckImplementationUnresolved,
                &verification.path,
                check.definition.line,
                Some(check.requirement.clone()),
                format!(
                    "document-authored Check `{}` has no stable extracted implementation",
                    check.definition.id
                ),
            ));
        }
    }
    for scope in &verification.requirements {
        let identity = &scope.requirement;
        let Some((_, target)) = requirement(specs, identity) else {
            report.findings.push(AccountFinding::new(
                AccountFindingKind::UnknownVerificationRequirement,
                &verification.path,
                scope.line,
                Some(identity.clone()),
                format!("Requirement verification `{identity}` names no spec Requirement"),
            ));
            continue;
        };
        if target.domain != Domain::Behaviour || target.verification.is_none() {
            report.findings.push(AccountFinding::new(AccountFindingKind::UnknownVerificationRequirement,
                &verification.path, scope.line, Some(identity.clone()),
                format!("Requirement verification `{identity}` does not target a behavioral Requirement that declares `Verification:`")));
            continue;
        }
        for scenario in verification
            .scenarios
            .iter()
            .filter(|scenario| scenario.requirement == *identity)
        {
            let local = scenario.scenario.as_str();
            if !target.scenarios.iter().any(|item| item.id == local) {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::UnknownVerificationScenario,
                    &verification.path,
                    scenario.line,
                    Some(identity.clone()),
                    format!(
                        "Scenario verification `{}` names no spec Scenario",
                        scenario.scenario
                    ),
                ));
            }
        }
        for element in &scope.required_elements {
            let qualified = format!("{}#{element}", verification.owner);
            let supported = supports
                .iter()
                .flat_map(|support| &support.elements)
                .any(|support| support.requirement == *identity && support.element == qualified);
            if !supported {
                report.findings.push(AccountFinding::new(AccountFindingKind::VerificationElementSupportUnresolved,
                    &verification.path, scope.line, Some(identity.clone()),
                    format!("required verification Element `{qualified}` has no extracted source/configuration support for `{identity}`")));
            }
        }
        for scenario in &target.scenarios {
            let scenario_id = scenario.id.clone();
            let contributed = authored_checks.iter().any(|check| {
                check.scenarios.contains(&scenario_id)
                    && check_has_implementation(check, implementations.iter())
            }) || supports
                .iter()
                .flat_map(|support| &support.contributions)
                .any(|contribution| contribution.scenario == scenario_id);
            if !contributed {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::ScenarioCheckContributionUnresolved,
                    &verification.path,
                    scope.line,
                    Some(identity.clone()),
                    format!("Scenario `{scenario_id}` has no extracted Check contribution"),
                ));
            }
        }
    }
}

fn uses_account_format(path: &Path, heading: &str) -> Result<bool, Vec<Diag>> {
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diag::file(
            &path.display().to_string(),
            format!("cannot read facet: {error}"),
        )]
    })?;
    let mut fenced = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if !fenced
            && (trimmed.starts_with(&format!("{heading}: ")) || trimmed.starts_with("## Section: "))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_supports(
    supports: &[AccountSupport],
    specs: &[Spec],
    verifications: &BTreeMap<String, AccountVerification>,
) -> Result<(), Vec<Diag>> {
    let mut errors = Vec::new();
    let mut artifact_ids = BTreeMap::<&str, &str>::new();
    let mut check_ids = BTreeMap::<&str, &str>::new();
    let mut contribution_pairs = BTreeMap::<(String, String), &str>::new();
    for support in supports {
        for artifact in &support.artifacts {
            if let Some(previous) = artifact_ids.insert(&artifact.id, &support.path) {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Artifact `{}` is already declared by {previous}",
                        artifact.id
                    ),
                ));
            }
        }
        for check in &support.checks {
            if let Some(previous) = check_ids.insert(&check.id, &support.path) {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!("Check `{}` is already declared by {previous}", check.id),
                ));
            }
        }
        for contribution in &support.contributions {
            let pair = (contribution.check.clone(), contribution.scenario.clone());
            if let Some(previous) = contribution_pairs.insert(pair.clone(), &support.path) {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Check-to-Scenario contribution `{} -> {}` is already declared by {previous}",
                        pair.0, pair.1
                    ),
                ));
            }
        }
    }
    for support in supports {
        for element in &support.elements {
            let Some((module, local)) = element.element.split_once('#') else {
                continue;
            };
            let declared = verifications.get(module).is_some_and(|verification| {
                verification.elements.iter().any(|item| item.id == local)
            });
            if !declared {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Element support names undeclared verification Element `{}`",
                        element.element
                    ),
                ));
            }
            if !requirement(specs, &element.requirement).is_some_and(|(spec, _)| spec.id == module)
            {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Element `{}` is not owned by Requirement `{}` module",
                        element.element, element.requirement
                    ),
                ));
            }
            let scoped = verifications.get(module).is_some_and(|verification| {
                verification.requirements.iter().any(|scope| {
                    scope.requirement == element.requirement
                        && scope
                            .required_elements
                            .iter()
                            .any(|required| required == local)
                })
            });
            if !scoped {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Element `{}` is not required by Requirement `{}` verification scope",
                        element.element, element.requirement
                    ),
                ));
            }
            for dependency in &element.depends_on {
                let Some((owner, local)) = dependency.split_once('#') else {
                    continue;
                };
                if !verifications.get(owner).is_some_and(|verification| {
                    verification.elements.iter().any(|item| item.id == local)
                }) {
                    errors.push(Diag::at(
                        &support.path,
                        0,
                        format!(
                            "Element `{}` depends on undeclared Element `{dependency}`",
                            element.element
                        ),
                    ));
                }
            }
        }
        for check in &support.checks {
            for element in &check.elements {
                let Some((module, local)) = element.split_once('#') else {
                    continue;
                };
                if !verifications.get(module).is_some_and(|verification| {
                    verification.elements.iter().any(|item| item.id == local)
                }) {
                    errors.push(Diag::at(
                        &support.path,
                        0,
                        format!(
                            "Check `{}` names undeclared verification Element `{element}`",
                            check.id
                        ),
                    ));
                }
            }
        }
        for contribution in &support.contributions {
            let scenario = find_scenario_by_id(specs, &contribution.scenario);
            let valid_scenario = scenario.is_some();
            let requirement_id = scenario
                .map(|(_, requirement, _)| requirement.id.as_str())
                .unwrap_or_default();
            if !valid_scenario {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Check `{}` contributes to unknown Scenario `{}`",
                        contribution.check, contribution.scenario
                    ),
                ));
            }
            if !support.elements.iter().any(|element| {
                element.requirement == requirement_id && element.element == contribution.element
            }) {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Check `{}` contribution to `{}` lacks Element `{}` support for its Requirement",
                        contribution.check, contribution.scenario, contribution.element
                    ),
                ));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn check_has_implementation<'a>(
    check: &crate::account_verification::AuthoredCheck,
    implementations: impl Iterator<Item = &'a crate::model::CheckImplementation>,
) -> bool {
    implementations
        .filter(|item| item.check == check.definition.id)
        .any(|item| {
            item.source.is_some()
                && item
                    .source_fingerprint
                    .strip_prefix("sha256:")
                    .is_some_and(|hex| {
                        hex.len() == 64
                            && hex
                                .bytes()
                                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    })
        })
}

pub fn mechanism_ids<'a>(
    realizations: impl Iterator<Item = &'a AccountRealization>,
    legacy: &[crate::realization::Realization],
) -> BTreeSet<String> {
    fn collect(item: &Declaration, ids: &mut BTreeSet<String>) {
        if item.kind == DeclarationKind::Mechanism {
            ids.insert(item.id.clone());
        }
        for child in &item.children {
            collect(child, ids);
        }
    }
    let mut ids = BTreeSet::new();
    for realization in realizations {
        for item in &realization.declarations {
            collect(item, &mut ids);
        }
    }
    for realization in legacy {
        for entry in &realization.entries {
            for mechanism in &entry.mechanisms {
                ids.insert(mechanism.id.clone());
            }
        }
    }
    ids
}

pub fn authored_check_issues<'a>(
    verifications: impl Iterator<Item = &'a AccountVerification>,
    specs: &[Spec],
    supports: &[AccountSupport],
    mechanisms: &BTreeSet<String>,
) -> Vec<Diag> {
    let mut errors = Vec::new();
    let mut ids = BTreeMap::<String, String>::new();
    for support in supports {
        for check in &support.checks {
            if let Some(previous) = ids.insert(check.id.clone(), support.path.clone()) {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Check `{}` already has a definition in {previous}",
                        check.id
                    ),
                ));
            }
        }
    }
    let documents = verifications.collect::<Vec<_>>();
    for document in documents {
        for check in &document.checks {
            let id = &check.definition.id;
            let line = check.definition.line;
            if let Some(previous) = ids.insert(id.clone(), document.path.clone()) {
                errors.push(Diag::at(
                    &document.path,
                    line,
                    format!("Check `{id}` already has a definition in {previous}"),
                ));
            }
            if supports.iter().any(|support| {
                support.checks.iter().any(|item| item.id == *id)
                    || support.contributions.iter().any(|item| item.check == *id)
            }) {
                errors.push(Diag::at(&document.path, line,
                    format!("document-authored Check `{id}` also has source-authored semantics; retain only ImplementsCheck linkage")));
            }
            for scenario in &check.scenarios {
                if !find_scenario_by_id(specs, scenario)
                    .is_some_and(|(_, target, _)| target.verification.is_some())
                {
                    errors.push(Diag::at(
                        &document.path,
                        line,
                        format!("Check `{id}` references an unknown Scenario or one of a Requirement without `Verification:` `{scenario}`"),
                    ));
                }
            }
            for mechanism in &check.mechanisms {
                if !mechanisms.contains(mechanism) {
                    errors.push(Diag::at(
                        &document.path,
                        line,
                        format!("Check `{id}` depends on unknown Mechanism `{mechanism}`"),
                    ));
                }
            }
        }
    }
    errors
}
