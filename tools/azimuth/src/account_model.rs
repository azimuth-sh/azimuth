//! Structural assembly of Claim-first account documents and available source linkage.
//!
//! A finding records an unresolved relationship. This module does not judge a Claim, qualify a
//! method, interpret a Run, or establish deployment Assurance State.

use crate::account_design::{self, AccountDesign, Declaration, DeclarationKind};
use crate::account_support::AccountSupport;
use crate::account_verification::{self, AccountVerification};
use crate::diag::Diag;
use crate::manifest::Manifest;
use crate::model::{Criticality, Domain, Spec};
use crate::spec;
use crate::workspace::Workspace;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountFindingKind {
    UnknownDesignClaim,
    UnknownDesignCase,
    UnknownMechanismCase,
    UnknownVerificationClaim,
    UnknownVerificationCase,
    DuplicateFacet,
    MissingClaimDesign,
    MissingClaimVerification,
    MissingClaimReview,
    PendingClaimReview,
    InvalidClaimReview,
    UnknownArea,
    MissingAreaRealization,
    VerificationElementSupportUnresolved,
    CaseCheckContributionUnresolved,
    CheckWithoutCaseContribution,
    MechanismSupportUnresolved,
    CheckImplementationUnresolved,
}

impl AccountFindingKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::UnknownDesignClaim => "unknown-design-claim",
            Self::UnknownDesignCase => "unknown-design-case",
            Self::UnknownMechanismCase => "unknown-mechanism-case",
            Self::UnknownVerificationClaim => "unknown-verification-claim",
            Self::UnknownVerificationCase => "unknown-verification-case",
            Self::DuplicateFacet => "duplicate-account-facet",
            Self::MissingClaimDesign => "missing-claim-design",
            Self::MissingClaimVerification => "missing-claim-verification",
            Self::MissingClaimReview => "missing-claim-review",
            Self::PendingClaimReview => "pending-claim-review",
            Self::InvalidClaimReview => "invalid-claim-review",
            Self::UnknownArea => "unknown-design-area",
            Self::MissingAreaRealization => "missing-area-realization",
            Self::VerificationElementSupportUnresolved => "verification-element-support-unresolved",
            Self::CaseCheckContributionUnresolved => "case-check-contribution-unresolved",
            Self::CheckWithoutCaseContribution => "check-without-case-contribution",
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
    pub claim: Option<String>,
    pub detail: String,
}

impl AccountFinding {
    fn new(
        kind: AccountFindingKind,
        path: &str,
        line: usize,
        claim: Option<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            path: path.to_string(),
            line,
            claim,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct AccountReport {
    pub specs: usize,
    pub claims: usize,
    pub cases: usize,
    pub design_facets: usize,
    pub verification_facets: usize,
    pub review_facets: usize,
    pub realizations: usize,
    pub support_artifacts: usize,
    pub support_checks: usize,
    pub case_contributions: usize,
    pub findings: Vec<AccountFinding>,
}

impl AccountReport {
    pub fn render(&self) -> String {
        let mut output = format!(
            "Account inspection: {} specs, {} claims, {} cases, {} design facets, {} verification facets, {} review facets, {} realizations, {} support artifacts, {} Checks, {} Case contributions, {} unresolved findings\n",
            self.specs, self.claims, self.cases, self.design_facets,
            self.verification_facets, self.review_facets, self.realizations, self.support_artifacts, self.support_checks, self.case_contributions, self.findings.len()
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
    let mut designs = BTreeMap::<String, AccountDesign>::new();
    let mut verifications = BTreeMap::<String, AccountVerification>::new();
    let mut reviews = BTreeMap::<String, ReviewFacet>::new();
    let mut report = AccountReport {
        specs: loaded.specs.len(),
        claims: loaded.specs.iter().map(|spec| spec.claims.len()).sum(),
        cases: loaded
            .specs
            .iter()
            .flat_map(|spec| &spec.claims)
            .map(|claim| claim.cases.len())
            .sum(),
        realizations: manifests
            .iter()
            .map(|manifest| manifest.realizes.len())
            .sum(),
        support_artifacts: supports.iter().map(|support| support.artifacts.len()).sum(),
        support_checks: supports.iter().map(|support| support.checks.len()).sum(),
        case_contributions: supports
            .iter()
            .map(|support| support.contributions.len())
            .sum(),
        ..Default::default()
    };
    for path in files {
        let display = path.display().to_string();
        match path.file_name().and_then(|name| name.to_str()) {
            Some("design.md") if uses_account_format(&path, "## Claim design")? => {
                match account_design::load_account_design(&path) {
                    Ok(design) => {
                        if let Some(previous) = designs.insert(design.spec.clone(), design) {
                            report.findings.push(AccountFinding::new(
                                AccountFindingKind::DuplicateFacet,
                                &display,
                                1,
                                None,
                                format!("design module also declared by {}", previous.path),
                            ));
                        }
                    }
                    Err(mut diagnostics) => errors.append(&mut diagnostics),
                }
            }
            Some("verification.md") if uses_account_format(&path, "## Claim verification")? => {
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
            Some("judgments.md") => match load_reviews(&path) {
                Ok(review) => {
                    if let Some(previous) = reviews.insert(review.owner.clone(), review) {
                        errors.push(Diag::at(
                            &display,
                            1,
                            format!("review module also declared by {}", previous.path),
                        ));
                    }
                }
                Err(mut diagnostics) => errors.append(&mut diagnostics),
            },
            _ => {}
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    report.design_facets = designs.len();
    report.verification_facets = verifications.len();
    report.review_facets = reviews.len();
    let (legacy_verifications, _) = crate::load_verification_facets(model_root)?;
    let (legacy_designs, _) = crate::load_design_facets(model_root)?;
    let known_mechanisms = mechanism_ids(designs.values(), &legacy_designs);
    let check_errors = authored_check_issues(
        verifications.values(),
        &loaded.specs,
        legacy_verifications.iter().flat_map(|file| &file.checks),
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
    report.case_contributions += verifications
        .values()
        .flat_map(|file| &file.checks)
        .map(|check| check.cases.len())
        .sum::<usize>();
    let definitions = crate::model::Model {
        specs: loaded.specs.clone(),
        account_designs: designs.values().cloned().collect(),
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
                    AccountFindingKind::CheckWithoutCaseContribution,
                    &support.path,
                    0,
                    None,
                    format!("Check `{}` has no explicit Case contribution", check.id),
                ));
            }
        }
    }
    for design in designs.values() {
        inspect_design(design, &loaded.specs, workspace, manifests, &mut report);
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
    for review in reviews.values() {
        inspect_reviews(review, &loaded.specs, &designs, &verifications, &mut report);
    }
    for spec in &loaded.specs {
        if !designs.contains_key(&spec.id)
            && !verifications.contains_key(&spec.id)
            && !reviews.contains_key(&spec.id)
        {
            continue;
        }
        for claim in &spec.claims {
            if !matches!(
                claim.criticality,
                Some(Criticality::Standard | Criticality::Critical)
            ) {
                continue;
            }
            let identity = claim.id.clone();
            if !designs
                .get(&spec.id)
                .is_some_and(|design| has_claim_design(design, &identity))
            {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::MissingClaimDesign,
                    &spec.path,
                    claim.line,
                    Some(identity.clone()),
                    format!("non-routine Claim `{identity}` has no Claim design"),
                ));
            }
            if !verifications.get(&spec.id).is_some_and(|verification| {
                verification
                    .claims
                    .iter()
                    .any(|scope| scope.claim == identity)
            }) {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::MissingClaimVerification,
                    &spec.path,
                    claim.line,
                    Some(identity.clone()),
                    format!("non-routine Claim `{identity}` has no Claim verification"),
                ));
            }
            if !reviews
                .get(&spec.id)
                .is_some_and(|review| review.claims.iter().any(|entry| entry.claim == identity))
            {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::MissingClaimReview,
                    &spec.path,
                    claim.line,
                    Some(identity.clone()),
                    format!("non-routine Claim `{identity}` has no Claim review"),
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
                AccountFindingKind::UnknownDesignClaim
                    | AccountFindingKind::UnknownDesignCase
                    | AccountFindingKind::UnknownMechanismCase
                    | AccountFindingKind::UnknownVerificationClaim
                    | AccountFindingKind::UnknownVerificationCase
                    | AccountFindingKind::DuplicateFacet
                    | AccountFindingKind::UnknownArea
                    | AccountFindingKind::InvalidClaimReview
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
            Some("design.md" | "verification.md" | "judgments.md")
        ) {
            paths.push(path);
        }
    }
    Ok(())
}

fn claim<'a>(specs: &'a [Spec], identity: &str) -> Option<(&'a Spec, &'a crate::model::Claim)> {
    specs.iter().find_map(|spec| {
        spec.claims
            .iter()
            .find(|claim| claim.id == identity)
            .map(|claim| (spec, claim))
    })
}

fn find_case_by_id<'a>(
    specs: &'a [Spec],
    identity: &str,
) -> Option<(&'a Spec, &'a crate::model::Claim, &'a crate::model::Case)> {
    specs.iter().find_map(|spec| {
        spec.claims.iter().find_map(|claim| {
            claim
                .cases
                .iter()
                .find(|case| case.id == identity)
                .map(|case| (spec, claim, case))
        })
    })
}

fn has_claim_design(design: &AccountDesign, identity: &str) -> bool {
    design
        .declarations
        .iter()
        .any(|item| item.kind == DeclarationKind::Claim && item.id == identity)
}

fn inspect_design(
    design: &AccountDesign,
    specs: &[Spec],
    workspace: &Workspace,
    manifests: &[Manifest],
    report: &mut AccountReport,
) {
    for item in &design.declarations {
        inspect_design_item(design, item, specs, workspace, manifests, None, report);
    }
}

fn inspect_design_item(
    design: &AccountDesign,
    item: &Declaration,
    specs: &[Spec],
    workspace: &Workspace,
    manifests: &[Manifest],
    enclosing_claim: Option<&str>,
    report: &mut AccountReport,
) {
    match item.kind {
        DeclarationKind::Claim => {
            let identity = item.id.clone();
            match claim(specs, &identity) {
                None => report.findings.push(AccountFinding::new(
                    AccountFindingKind::UnknownDesignClaim,
                    &design.path,
                    item.line,
                    Some(identity.clone()),
                    format!("Claim design `{identity}` names no spec Claim"),
                )),
                Some((_, target))
                    if target.domain != Domain::Behaviour
                        || !matches!(
                            target.criticality,
                            Some(Criticality::Standard | Criticality::Critical)
                        ) =>
                {
                    report.findings.push(AccountFinding::new(AccountFindingKind::UnknownDesignClaim,
                        &design.path, item.line, Some(identity.clone()),
                        format!("Claim design `{identity}` does not target a non-routine behavioral Claim")));
                }
                Some(_) => {}
            }
            for area in &item.areas {
                if !workspace.areas.iter().any(|declared| declared.id == *area) {
                    report.findings.push(AccountFinding::new(
                        AccountFindingKind::UnknownArea,
                        &design.path,
                        item.line,
                        Some(identity.clone()),
                        format!("Claim design `{identity}` names undeclared Area `{area}`"),
                    ));
                    continue;
                }
                let claim_id = identity.as_str();
                let realized = manifests
                    .iter()
                    .flat_map(|manifest| &manifest.realizes)
                    .any(|site| {
                        site.claim == claim_id
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
                        &design.path, item.line, Some(identity.clone()),
                        format!("Claim `{identity}` has no extracted Realizes site in required Area `{area}`")));
                }
            }
        }
        DeclarationKind::Case => {
            if !find_case_by_id(specs, &item.id)
                .is_some_and(|(_, claim, _)| enclosing_claim == Some(claim.id.as_str()))
            {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::UnknownDesignCase,
                    &design.path,
                    item.line,
                    None,
                    format!("Case design `{}` names no spec Case", item.id),
                ));
            }
        }
        DeclarationKind::Element => {
            for area in &item.areas {
                if !workspace.areas.iter().any(|declared| declared.id == *area) {
                    report.findings.push(AccountFinding::new(
                        AccountFindingKind::UnknownArea,
                        &design.path,
                        item.line,
                        None,
                        format!("Element `{}` names undeclared Area `{area}`", item.id),
                    ));
                }
            }
        }
        DeclarationKind::Mechanism => {
            for case in &item.cases {
                if find_case_by_id(specs, case).is_none() {
                    report.findings.push(AccountFinding::new(
                        AccountFindingKind::UnknownMechanismCase,
                        &design.path,
                        item.line,
                        None,
                        format!("Mechanism `{}` names unknown Case `{case}`", item.id),
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
                    &design.path,
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
        let owner = if item.kind == DeclarationKind::Claim {
            Some(item.id.as_str())
        } else {
            enclosing_claim
        };
        inspect_design_item(design, child, specs, workspace, manifests, owner, report);
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
                Some(check.claim.clone()),
                format!(
                    "document-authored Check `{}` has no stable extracted implementation",
                    check.definition.id
                ),
            ));
        }
    }
    for scope in &verification.claims {
        let identity = &scope.claim;
        let Some((_, target)) = claim(specs, identity) else {
            report.findings.push(AccountFinding::new(
                AccountFindingKind::UnknownVerificationClaim,
                &verification.path,
                scope.line,
                Some(identity.clone()),
                format!("Claim verification `{identity}` names no spec Claim"),
            ));
            continue;
        };
        if target.domain != Domain::Behaviour
            || !matches!(
                target.criticality,
                Some(Criticality::Standard | Criticality::Critical)
            )
        {
            report.findings.push(AccountFinding::new(AccountFindingKind::UnknownVerificationClaim,
                &verification.path, scope.line, Some(identity.clone()),
                format!("Claim verification `{identity}` does not target a non-routine behavioral Claim")));
            continue;
        }
        for case in verification
            .cases
            .iter()
            .filter(|case| case.claim == *identity)
        {
            let local = case.case.as_str();
            if !target.cases.iter().any(|item| item.id == local) {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::UnknownVerificationCase,
                    &verification.path,
                    case.line,
                    Some(identity.clone()),
                    format!("Case verification `{}` names no spec Case", case.case),
                ));
            }
        }
        for element in &scope.required_elements {
            let qualified = format!("{}#{element}", verification.owner);
            let supported = supports
                .iter()
                .flat_map(|support| &support.elements)
                .any(|support| support.claim == *identity && support.element == qualified);
            if !supported {
                report.findings.push(AccountFinding::new(AccountFindingKind::VerificationElementSupportUnresolved,
                    &verification.path, scope.line, Some(identity.clone()),
                    format!("required verification Element `{qualified}` has no extracted source/configuration support for `{identity}`")));
            }
        }
        for case in &target.cases {
            let case_id = case.id.clone();
            let contributed = authored_checks.iter().any(|check| {
                check.cases.contains(&case_id)
                    && check_has_implementation(check, implementations.iter())
            }) || supports
                .iter()
                .flat_map(|support| &support.contributions)
                .any(|contribution| contribution.case == case_id);
            if !contributed {
                report.findings.push(AccountFinding::new(
                    AccountFindingKind::CaseCheckContributionUnresolved,
                    &verification.path,
                    scope.line,
                    Some(identity.clone()),
                    format!("Case `{case_id}` has no extracted Check contribution"),
                ));
            }
        }
    }
}

#[derive(Debug)]
pub struct ReviewFacet {
    pub owner: String,
    pub path: String,
    pub claims: Vec<ClaimReview>,
}

#[derive(Debug)]
pub struct ClaimReview {
    pub claim: String,
    pub cases: Vec<String>,
    pub design: String,
    pub verification: String,
    pub line: usize,
}

pub fn load_reviews(path: &Path) -> Result<ReviewFacet, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diag::file(
            &display,
            format!("cannot read reviews: {error}"),
        )]
    })?;
    parse_reviews(&display, &source)
}

fn parse_reviews(path: &str, source: &str) -> Result<ReviewFacet, Vec<Diag>> {
    let lines = source.lines().collect::<Vec<_>>();
    let owner = lines
        .first()
        .and_then(|line| line.strip_prefix("# Judgments: "))
        .unwrap_or("");
    let mut errors = Vec::new();
    if let Err(reason) = crate::diag::validate_id(owner, true) {
        errors.push(Diag::expecting(
            path,
            1,
            format!("invalid review module id: {reason}"),
            "# Judgments: <module-id>",
        ));
    }
    let mut claims = Vec::new();
    let mut index = 1;
    let mut fenced = false;
    while index < lines.len() {
        let text = lines[index].trim();
        if text.starts_with("```") || text.starts_with("~~~") {
            fenced = !fenced;
            index += 1;
            continue;
        }
        if fenced {
            index += 1;
            continue;
        }
        if let Some(id) = text.strip_prefix("## Claim review: ") {
            let line = index + 1;
            if let Err(reason) = crate::diag::validate_id(id, false) {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("invalid local Claim review identity `{id}`: {reason}"),
                    "## Claim review: <claim-id> within the declared module",
                ));
            }
            let identity = id.to_string();
            let mut end = index + 1;
            let mut inside_fence = false;
            while end < lines.len() {
                let next = lines[end].trim();
                if next.starts_with("```") || next.starts_with("~~~") {
                    inside_fence = !inside_fence;
                }
                if !inside_fence && next.starts_with("## ") {
                    break;
                }
                end += 1;
            }
            let mut state = None;
            let mut cases = Vec::new();
            let mut design = None;
            let mut verification = None;
            let mut cursor = index + 1;
            while cursor < end && lines[cursor].trim().is_empty() {
                cursor += 1;
            }
            while cursor < end && lines[cursor].starts_with("- ") {
                let field_line = cursor + 1;
                let field = &lines[cursor][2..];
                let Some((name, value)) = field.split_once(':') else {
                    errors.push(Diag::expecting(
                        path,
                        field_line,
                        "malformed review field",
                        "- State:, - Cases:, - Design:, or - Verification:",
                    ));
                    cursor += 1;
                    continue;
                };
                match name {
                    "State" => {
                        if state.is_some() {
                            errors.push(Diag::at(path, field_line, "duplicate State"));
                        }
                        state = review_value(path, field_line, value, &mut errors);
                    }
                    "Design" => {
                        if design.is_some() {
                            errors.push(Diag::at(path, field_line, "duplicate Design"));
                        }
                        design = review_value(path, field_line, value, &mut errors);
                    }
                    "Verification" => {
                        if verification.is_some() {
                            errors.push(Diag::at(path, field_line, "duplicate Verification"));
                        }
                        verification = review_value(path, field_line, value, &mut errors);
                    }
                    "Cases" => {
                        if !cases.is_empty() {
                            errors.push(Diag::at(path, field_line, "duplicate Cases"));
                        }
                        if !value.trim().is_empty() {
                            errors.push(Diag::expecting(
                                path,
                                field_line,
                                "Cases must be a list",
                                "- Cases: followed by indented values",
                            ));
                        }
                        cursor += 1;
                        while cursor < end && lines[cursor].starts_with("  - ") {
                            if let Some(value) =
                                review_value(path, cursor + 1, &lines[cursor][4..], &mut errors)
                            {
                                cases.push(value);
                            }
                            cursor += 1;
                        }
                        continue;
                    }
                    _ => errors.push(Diag::expecting(
                        path,
                        field_line,
                        format!("unknown review field `{name}`"),
                        "State, Cases, Design, Verification",
                    )),
                }
                cursor += 1;
            }
            if state.as_deref() != Some("pending") {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!(
                        "Claim review `{id}` has no pending state or claims an accepted verdict"
                    ),
                    "- State: `pending`; accepted verdicts require a separate reviewed transition",
                ));
            }
            if cases.is_empty() {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("Claim review `{id}` names no Cases"),
                    "- Cases: with every current Case",
                ));
            }
            if design.is_none() || verification.is_none() {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("Claim review `{id}` lacks design or verification reference"),
                    "- Design: and - Verification: naming this Claim",
                ));
            }
            if claims
                .iter()
                .any(|existing: &ClaimReview| existing.claim == identity)
            {
                errors.push(Diag::at(
                    path,
                    line,
                    format!("duplicate Claim review `{id}`"),
                ));
            }
            claims.push(ClaimReview {
                claim: identity,
                cases,
                design: design.unwrap_or_default(),
                verification: verification.unwrap_or_default(),
                line,
            });
            index = end;
            continue;
        }
        if text.starts_with("## ") && !text.starts_with("## Section: ") {
            errors.push(Diag::expecting(
                path,
                index + 1,
                format!("unknown review declaration `{text}`"),
                "## Section: or ## Claim review:",
            ));
        }
        index += 1;
    }
    if claims.is_empty() {
        errors.push(Diag::expecting(
            path,
            1,
            "review has no Claim accounts",
            "at least one ## Claim review:",
        ));
    }
    if errors.is_empty() {
        Ok(ReviewFacet {
            owner: owner.to_string(),
            path: path.to_string(),
            claims,
        })
    } else {
        Err(errors)
    }
}

fn review_value(path: &str, line: usize, value: &str, errors: &mut Vec<Diag>) -> Option<String> {
    let value = value.trim();
    let Some(inner) = value
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
    else {
        errors.push(Diag::expecting(
            path,
            line,
            format!("invalid review value `{value}`"),
            "one backtick-delimited value",
        ));
        return None;
    };
    if inner.is_empty() || inner.contains('`') {
        errors.push(Diag::expecting(
            path,
            line,
            "empty or malformed review value",
            "one nonempty backtick-delimited value",
        ));
        None
    } else {
        Some(inner.to_string())
    }
}

fn inspect_reviews(
    review: &ReviewFacet,
    specs: &[Spec],
    designs: &BTreeMap<String, AccountDesign>,
    verifications: &BTreeMap<String, AccountVerification>,
    report: &mut AccountReport,
) {
    for entry in &review.claims {
        let identity = &entry.claim;
        let Some((_, target)) = claim(specs, identity) else {
            report.findings.push(AccountFinding::new(
                AccountFindingKind::InvalidClaimReview,
                &review.path,
                entry.line,
                Some(identity.clone()),
                format!("Claim review `{identity}` names no spec Claim"),
            ));
            continue;
        };
        let expected = target
            .cases
            .iter()
            .map(|case| case.id.clone())
            .collect::<BTreeSet<_>>();
        let actual = entry.cases.iter().cloned().collect::<BTreeSet<_>>();
        if expected != actual || entry.cases.len() != actual.len() {
            report.findings.push(AccountFinding::new(
                AccountFindingKind::InvalidClaimReview,
                &review.path,
                entry.line,
                Some(identity.clone()),
                format!("Claim review `{identity}` does not name exactly its current Cases"),
            ));
        }
        if entry.design != *identity
            || !designs
                .get(&review.owner)
                .is_some_and(|design| has_claim_design(design, identity))
        {
            report.findings.push(AccountFinding::new(
                AccountFindingKind::InvalidClaimReview,
                &review.path,
                entry.line,
                Some(identity.clone()),
                format!("Claim review `{identity}` has no matching Design reference"),
            ));
        }
        if entry.verification != *identity
            || !verifications
                .get(&review.owner)
                .is_some_and(|verification| {
                    verification
                        .claims
                        .iter()
                        .any(|scope| scope.claim == *identity)
                })
        {
            report.findings.push(AccountFinding::new(
                AccountFindingKind::InvalidClaimReview,
                &review.path,
                entry.line,
                Some(identity.clone()),
                format!("Claim review `{identity}` has no matching Verification reference"),
            ));
        }
        report.findings.push(AccountFinding::new(
            AccountFindingKind::PendingClaimReview,
            &review.path,
            entry.line,
            Some(identity.clone()),
            format!("Claim review `{identity}` remains pending; no verdict is inferred"),
        ));
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
        if !fenced && trimmed.starts_with(heading) {
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
            let pair = (contribution.check.clone(), contribution.case.clone());
            if let Some(previous) = contribution_pairs.insert(pair.clone(), &support.path) {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Check-to-Case contribution `{} -> {}` is already declared by {previous}",
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
            if !claim(specs, &element.claim).is_some_and(|(spec, _)| spec.id == module) {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Element `{}` is not owned by Claim `{}` module",
                        element.element, element.claim
                    ),
                ));
            }
            let scoped = verifications.get(module).is_some_and(|verification| {
                verification.claims.iter().any(|scope| {
                    scope.claim == element.claim
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
                        "Element `{}` is not required by Claim `{}` verification scope",
                        element.element, element.claim
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
            let case = find_case_by_id(specs, &contribution.case);
            let valid_case = case.is_some();
            let claim_id = case
                .map(|(_, claim, _)| claim.id.as_str())
                .unwrap_or_default();
            if !valid_case {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Check `{}` contributes to unknown Case `{}`",
                        contribution.check, contribution.case
                    ),
                ));
            }
            if !support
                .elements
                .iter()
                .any(|element| element.claim == claim_id && element.element == contribution.element)
            {
                errors.push(Diag::at(
                    &support.path,
                    0,
                    format!(
                        "Check `{}` contribution to `{}` lacks Element `{}` support for its Claim",
                        contribution.check, contribution.case, contribution.element
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
    designs: impl Iterator<Item = &'a AccountDesign>,
    legacy: &[crate::design::Design],
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
    for design in designs {
        for item in &design.declarations {
            collect(item, &mut ids);
        }
    }
    for design in legacy {
        for entry in &design.entries {
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
    legacy: impl Iterator<Item = &'a crate::verification::Check>,
    supports: &[AccountSupport],
    mechanisms: &BTreeSet<String>,
) -> Vec<Diag> {
    let mut errors = Vec::new();
    let mut ids = BTreeMap::<String, String>::new();
    for check in legacy {
        if let Some(previous) = ids.insert(check.id.clone(), check.path.clone()) {
            errors.push(Diag::at(
                &check.path,
                check.line,
                format!(
                    "Check `{}` already has a definition in {previous}",
                    check.id
                ),
            ));
        }
    }
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
            for case in &check.cases {
                if !find_case_by_id(specs, case).is_some_and(|(_, target, _)| {
                    matches!(
                        target.criticality,
                        Some(Criticality::Standard | Criticality::Critical)
                    )
                }) {
                    errors.push(Diag::at(
                        &document.path,
                        line,
                        format!("Check `{id}` references unknown or routine Case `{case}`"),
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
