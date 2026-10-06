//! The derived model.
//!
//! `claim = (domain, predicate)`. Cases are normative constituents of that predicate; they are
//! addressable without becoming independent assurance centres.

use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Criticality {
    Routine,
    Standard,
    Critical,
}

impl Criticality {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(Criticality::Critical),
            "standard" => Some(Criticality::Standard),
            "routine" => Some(Criticality::Routine),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Criticality::Critical => "critical",
            Criticality::Standard => "standard",
            Criticality::Routine => "routine",
        }
    }
}

/// The execution reach declared by an Evidence Binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Unit,
    Component,
    E2e,
}

impl Scope {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "unit" => Some(Scope::Unit),
            "component" => Some(Scope::Component),
            "e2e" => Some(Scope::E2e),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Scope::Unit => "unit",
            Scope::Component => "component",
            Scope::E2e => "e2e",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quantification {
    Example,
    Universal,
}

impl Quantification {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "example" => Some(Quantification::Example),
            "universal" => Some(Quantification::Universal),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Quantification::Example => "example",
            Quantification::Universal => "universal",
        }
    }
}

/// How evidence obtains its expected result. Oracle kinds are descriptive categories rather than
/// a strength ladder, but keeping the vocabulary closed prevents stale emitters from inventing a
/// category the model and its judges do not understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oracle {
    Direct,
    Golden,
    Relational,
    Metamorphic,
    ModelBased,
    Contract,
}

impl Oracle {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "direct" => Some(Oracle::Direct),
            "golden" => Some(Oracle::Golden),
            "relational" => Some(Oracle::Relational),
            "metamorphic" => Some(Oracle::Metamorphic),
            "model-based" => Some(Oracle::ModelBased),
            "contract" => Some(Oracle::Contract),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Oracle::Direct => "direct",
            Oracle::Golden => "golden",
            Oracle::Relational => "relational",
            Oracle::Metamorphic => "metamorphic",
            Oracle::ModelBased => "model-based",
            Oracle::Contract => "contract",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub id: String,
    /// Authoritative free-form Markdown. Core preserves and fingerprints it without interpreting
    /// its natural-language or diagram semantics.
    pub statement: String,
    pub line: usize,
}

/// What a claim ranges over. The behavioural domain is implicit and never written; a second
/// domain arrived only when the demo produced evidence that the first could not carry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    /// Executions of a behaviour described by one or more authored Cases.
    Behaviour,
    /// A set of sites. Membership is derived from what the code built, so a new site joins the
    /// class without anyone declaring it.
    Sites,
}

impl Domain {
    pub fn name(self) -> &'static str {
        match self {
            Self::Behaviour => "behaviour",
            Self::Sites => "sites",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Claim {
    pub id: String,
    /// `None` is the `unclassified` finding, not a parse error: a missing *declaration* is a
    /// semantic gap, while an unrecognized *construct* fails the parse.
    pub criticality: Option<Criticality>,
    pub statement: String,
    pub cases: Vec<Case>,
    pub line: usize,
    pub domain: Domain,
    /// For `Domain::Sites`: the declared surface whose derived members form the domain.
    pub over: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Term {
    pub id: String,
    /// Authoritative vocabulary definition. Every definition in the module participates in
    /// Claim and Case semantic digests because prose use cannot be resolved reliably by core.
    pub definition: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub id: String,
    pub path: String,
    pub claims: Vec<Claim>,
    pub terms: Vec<Term>,
}

fn vocabulary_json(spec: &Spec) -> Json {
    let mut terms = spec.terms.iter().collect::<Vec<_>>();
    terms.sort_by(|left, right| left.id.cmp(&right.id));
    Json::Arr(
        terms
            .into_iter()
            .map(|term| {
                Json::obj(vec![
                    ("id", Json::str(&term.id)),
                    ("definition", Json::str(&term.definition)),
                ])
            })
            .collect(),
    )
}

/// Stable identity of a compiler/schema source inside a federated Azimuth project.
///
/// `area + kind + address` is semantic identity. `mount` and the relation's existing `file`
/// field are locators: moving an unchanged area or changing a checkout layout must not manufacture
/// a different realization. Extractor manifests omit this value; local or federated assembly
/// derives it from the declared workspace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceIdentity {
    pub area: String,
    pub kind: String,
    pub address: String,
    pub mount: String,
}

impl SourceIdentity {
    pub fn key(&self) -> String {
        format!("{}|{}|{}", self.area, self.kind, self.address)
    }
}

/// A compiler-resolved production realization site.
#[derive(Debug, Clone)]
pub struct Site {
    pub spec: String,
    pub claim: String,
    pub site: String,
    pub file: String,
    pub lang: String,
    pub source: Option<SourceIdentity>,
    /// Hash of the exact compiler-resolved enclosing site.
    pub source_fingerprint: String,
}

impl Site {
    pub fn subject_identities(&self) -> [String; 2] {
        [
            self.source
                .as_ref()
                .map(SourceIdentity::key)
                .unwrap_or_default(),
            format!("{}#{}|{}", self.file, self.site, self.lang),
        ]
    }
}

/// A compiler-resolved site that implements a design-owned mechanism identity.
#[derive(Debug, Clone)]
pub struct MechanismImplementation {
    pub spec: String,
    pub mechanism: String,
    pub site: String,
    pub binding: String,
    pub file: String,
    pub lang: String,
    pub source: Option<SourceIdentity>,
    pub source_fingerprint: String,
}

/// A compiler-resolved source site that implements one project-global Check.
#[derive(Debug, Clone)]
pub struct CheckImplementation {
    pub check: String,
    pub site: String,
    pub file: String,
    pub lang: String,
    pub source: Option<SourceIdentity>,
    pub source_fingerprint: String,
}

impl CheckImplementation {
    pub fn semantic_identity(&self) -> String {
        self.source
            .as_ref()
            .map(SourceIdentity::key)
            .unwrap_or_else(|| format!("|{}|{}", self.lang, self.site))
    }
}

/// A member of a class, enumerated by the project's extractor from what the build produced —
/// a route table, a container, a manifest — rather than from a tag.
///
/// This exists because deriving membership from tags cannot see a site nobody tagged, which is the
/// enumerator failure: an enumerator that misses a member reports green over the gap. Identity is
/// the **file**: the member is the file, and a discharge anywhere in it discharges the member.
#[derive(Debug, Clone)]
pub struct ClassMember {
    pub class: String,
    pub site: String,
    pub file: String,
    pub lang: String,
    pub source: Option<SourceIdentity>,
}

/// Evidence that a class was enumerated from a system-produced source rather than reconstructed
/// from the declarations whose omissions the enumeration exists to find.
#[derive(Debug, Clone)]
pub struct Enumeration {
    pub class: String,
    pub kind: String,
    pub source: String,
    pub source_fingerprint: String,
    pub identity: Option<SourceIdentity>,
}

/// A machine-addressable artifact emitted from a compiler or schema model. Optional properties
/// carry only facts the extractor can derive; semantic claims remain in the design prose.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: String,
    pub kind: String,
    pub file: String,
    pub unique: Option<bool>,
    pub columns: Vec<String>,
    pub predicate: Option<String>,
    pub source: Option<SourceIdentity>,
}

/// One provider-neutral semantic-scope item plus the optional source account needed to build
/// launch inputs. The model owns semantic identity; Run protocol types remain a downstream
/// projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticScopeComponent {
    pub kind: crate::verification::SemanticScopeKind,
    pub id: String,
    pub fingerprint: String,
    pub locator: Option<SemanticScopeLocator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticScopeLocator {
    Source {
        file: String,
        language: String,
        site: String,
    },
    Artifact {
        file: String,
        artifact_kind: String,
        identity: String,
        unique: Option<bool>,
        columns: Vec<String>,
        predicate: Option<String>,
    },
    Enumeration {
        file: String,
        enumerator_kind: String,
        identity: String,
    },
    EnumeratedSurfaceMember {
        file: String,
        language: String,
        site: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticChallengeScope {
    pub anchors: Vec<SemanticScopeComponent>,
    pub inputs: Vec<SemanticScopeComponent>,
}

impl SemanticChallengeScope {
    /// Unions selector projections with the format's ordering and conflict rules. The same
    /// exact item may
    /// remain in both arrays because authored origin and decision composition are distinct roles.
    pub fn merge(scopes: impl IntoIterator<Item = Self>) -> Option<Self> {
        let mut anchors = Vec::new();
        let mut inputs = Vec::new();
        for scope in scopes {
            anchors.extend(scope.anchors);
            inputs.extend(scope.inputs);
        }
        let anchors = normalize_scope_components(anchors)?;
        let inputs = normalize_scope_components(inputs)?;
        for anchor in &anchors {
            if let Some(input) = inputs
                .iter()
                .find(|input| input.kind == anchor.kind && input.id == anchor.id)
            {
                if input != anchor {
                    return None;
                }
            }
        }
        Some(Self { anchors, inputs })
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub package_producers: Vec<crate::verification_packages::Producer>,
    pub specs: Vec<Spec>,
    pub account_designs: Vec<crate::account_design::AccountDesign>,
    pub account_verifications: Vec<crate::account_verification::AccountVerification>,
    pub account_supports: Vec<crate::account_support::AccountSupport>,
    pub account_reviews: Vec<crate::account_model::ReviewFacet>,
    pub realizes: Vec<Site>,
    pub mechanism_implementations: Vec<MechanismImplementation>,
    pub check_implementations: Vec<CheckImplementation>,
    /// Class members enumerated by an extractor. Empty when no project emits them, in which case
    /// a class is only as wide as its tags.
    pub class_members: Vec<ClassMember>,
    pub enumerations: Vec<Enumeration>,
    pub artifacts: Vec<Artifact>,
    pub decision_standards: Option<crate::verification::DecisionStandards>,
    pub verifications: Vec<crate::verification::Verification>,
    pub designs: Vec<crate::design::Design>,
    pub workspace: crate::workspace::Workspace,
}

/// A normative Case plus its independently governed parent Claim.
pub struct CaseView<'a> {
    pub spec: &'a Spec,
    pub claim: &'a Claim,
    pub case: &'a Case,
}

impl<'a> CaseView<'a> {
    pub fn id(&self) -> String {
        self.case.id.clone()
    }
}

/// An independently governable product proposition.
pub struct ClaimView<'a> {
    pub spec: &'a Spec,
    pub claim: &'a Claim,
}

impl ClaimView<'_> {
    pub fn id(&self) -> String {
        self.claim.id.clone()
    }
}

impl Model {
    pub fn entity_declaration_issues(&self) -> Vec<crate::diag::Diag> {
        let mut issues = Vec::new();
        let mut identities = BTreeMap::<(String, String), String>::new();
        fn record(
            kind: &str,
            id: &str,
            path: &str,
            line: usize,
            identities: &mut BTreeMap<(String, String), String>,
            issues: &mut Vec<crate::diag::Diag>,
        ) {
            if let Some(previous) =
                identities.insert((kind.to_string(), id.to_string()), path.to_string())
            {
                issues.push(crate::diag::Diag::at(
                    path,
                    line,
                    format!(
                        "duplicate project-wide {kind} ID `{id}` (first declared in {previous})"
                    ),
                ));
            }
        }
        fn mechanisms(
            item: &crate::account_design::Declaration,
            path: &str,
            identities: &mut BTreeMap<(String, String), String>,
            issues: &mut Vec<crate::diag::Diag>,
        ) {
            if item.kind == crate::account_design::DeclarationKind::Mechanism {
                record("Mechanism", &item.id, path, item.line, identities, issues);
            }
            for child in &item.children {
                mechanisms(child, path, identities, issues);
            }
        }
        for spec in &self.specs {
            for claim in &spec.claims {
                record(
                    "Claim",
                    &claim.id,
                    &spec.path,
                    claim.line,
                    &mut identities,
                    &mut issues,
                );
                for case in &claim.cases {
                    record(
                        "Case",
                        &case.id,
                        &spec.path,
                        case.line,
                        &mut identities,
                        &mut issues,
                    );
                }
            }
        }
        for design in &self.designs {
            for entry in &design.entries {
                for mechanism in &entry.mechanisms {
                    record(
                        "Mechanism",
                        &mechanism.id,
                        &design.path,
                        mechanism.line,
                        &mut identities,
                        &mut issues,
                    );
                }
            }
        }
        for design in &self.account_designs {
            for item in &design.declarations {
                mechanisms(item, &design.path, &mut identities, &mut issues);
            }
        }
        for document in &self.verifications {
            for check in &document.checks {
                record(
                    "Check",
                    &check.id,
                    &document.path,
                    check.line,
                    &mut identities,
                    &mut issues,
                );
            }
        }
        for document in &self.account_verifications {
            for check in &document.checks {
                record(
                    "Check",
                    &check.definition.id,
                    &document.path,
                    check.definition.line,
                    &mut identities,
                    &mut issues,
                );
            }
        }
        issues
    }

    pub fn resolve_source_membership(&mut self) {
        let claims = self
            .claims()
            .map(|view| (view.claim.id.clone(), view.spec.id.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut mechanisms = self
            .designs
            .iter()
            .flat_map(|design| {
                design.entries.iter().flat_map(move |entry| {
                    entry
                        .mechanisms
                        .iter()
                        .map(move |mechanism| (mechanism.id.clone(), design.spec.clone()))
                })
            })
            .collect::<BTreeMap<_, _>>();
        fn collect(
            item: &crate::account_design::Declaration,
            module: &str,
            targets: &mut BTreeMap<String, String>,
        ) {
            if item.kind == crate::account_design::DeclarationKind::Mechanism {
                targets.insert(item.id.clone(), module.to_string());
            }
            for child in &item.children {
                collect(child, module, targets);
            }
        }
        for design in &self.account_designs {
            for item in &design.declarations {
                collect(item, &design.spec, &mut mechanisms);
            }
        }
        for site in &mut self.realizes {
            site.spec = claims.get(&site.claim).cloned().unwrap_or_default();
        }
        for implementation in &mut self.mechanism_implementations {
            implementation.spec = mechanisms
                .get(&implementation.mechanism)
                .cloned()
                .unwrap_or_default();
        }
    }

    pub fn account_design_for_claim(
        &self,
        spec: &str,
        claim: &str,
    ) -> Option<(
        &crate::account_design::AccountDesign,
        &crate::account_design::Declaration,
    )> {
        let id = claim.to_string();
        self.account_designs.iter().find_map(|design| {
            (design.spec == spec)
                .then(|| {
                    design
                        .declarations
                        .iter()
                        .find(|declaration| {
                            declaration.kind == crate::account_design::DeclarationKind::Claim
                                && declaration.id == id
                        })
                        .map(|declaration| (design, declaration))
                })
                .flatten()
        })
    }

    pub fn realization_obligation(
        &self,
        spec: &str,
        claim: &str,
    ) -> Option<crate::workspace::RealizationObligation> {
        if let Some((_, declaration)) = self.account_design_for_claim(spec, claim) {
            return Some(crate::workspace::RealizationObligation {
                spec: spec.to_string(),
                claim: claim.to_string(),
                areas: declaration.areas.clone(),
            });
        }
        self.workspace.obligation(spec, claim).cloned()
    }

    pub fn realization_obligations(&self) -> Vec<crate::workspace::RealizationObligation> {
        let mut obligations = self.workspace.realization_obligations.clone();
        for design in &self.account_designs {
            for declaration in &design.declarations {
                if declaration.kind != crate::account_design::DeclarationKind::Claim {
                    continue;
                }
                {
                    obligations.retain(|item| item.claim != declaration.id);
                    obligations.push(crate::workspace::RealizationObligation {
                        spec: design.spec.clone(),
                        claim: declaration.id.clone(),
                        areas: declaration.areas.clone(),
                    });
                }
            }
        }
        obligations
    }

    pub(crate) fn account_design_digest(&self, spec: &str, claim: &str) -> Option<String> {
        let (design, declaration) = self.account_design_for_claim(spec, claim)?;
        fn relationships(item: &crate::account_design::Declaration) -> Json {
            Json::obj(vec![
                ("kind", Json::str(&format!("{:?}", item.kind))),
                ("id", Json::str(&item.id)),
                (
                    "cases",
                    Json::Arr(item.cases.iter().map(Json::str).collect()),
                ),
                (
                    "children",
                    Json::Arr(item.children.iter().map(relationships).collect()),
                ),
            ])
        }
        Some(crate::fingerprint::canonical_sha256(&Json::obj(vec![
            (
                "relationships",
                Json::Arr(
                    design
                        .declarations
                        .iter()
                        .filter(|item| {
                            item.kind != crate::account_design::DeclarationKind::Claim
                                || item.id == declaration.id
                        })
                        .map(relationships)
                        .collect(),
                ),
            ),
            ("introduction", Json::str(&design.introduction)),
            (
                "shared",
                Json::Arr(
                    design
                        .declarations
                        .iter()
                        .filter(|item| item.kind != crate::account_design::DeclarationKind::Claim)
                        .map(|item| Json::str(&item.source))
                        .collect(),
                ),
            ),
            ("claim", Json::str(&declaration.source)),
        ])))
    }

    pub(crate) fn account_verification_digest(&self, spec: &str, claim: &str) -> Option<String> {
        let id = claim.to_string();
        let document = self
            .account_verifications
            .iter()
            .find(|item| item.owner == spec)?;
        let scope = document.claims.iter().find(|item| item.claim == id)?;
        let sections = document
            .sections
            .iter()
            .filter(|section| section.claim.as_deref().is_none_or(|owner| owner == id))
            .map(|section| {
                Json::obj(vec![
                    ("id", Json::str(&section.id)),
                    (
                        "claim",
                        section.claim.as_ref().map_or(Json::Null, Json::str),
                    ),
                    ("prose", Json::str(&section.prose)),
                ])
            })
            .collect();
        let elements = document
            .elements
            .iter()
            .filter(|element| element.claim.as_deref().is_none_or(|owner| owner == id))
            .map(|element| {
                Json::obj(vec![
                    ("id", Json::str(&element.id)),
                    (
                        "claim",
                        element.claim.as_ref().map_or(Json::Null, Json::str),
                    ),
                    ("kind", Json::str(&element.kind)),
                    ("case", element.case.as_ref().map_or(Json::Null, Json::str)),
                    (
                        "section",
                        element.section.as_ref().map_or(Json::Null, Json::str),
                    ),
                    ("prose", Json::str(&element.prose)),
                ])
            })
            .collect();
        Some(crate::fingerprint::canonical_sha256(&Json::obj(vec![
            ("introduction", Json::str(&document.introduction)),
            ("scope", Json::str(&scope.prose)),
            (
                "cases",
                Json::Arr(
                    document
                        .cases
                        .iter()
                        .filter(|case| case.claim == id)
                        .map(|case| {
                            Json::obj(vec![
                                ("case", Json::str(&case.case)),
                                ("prose", Json::str(&case.prose)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "packages",
                crate::verification_packages::declarations_json(&document.package_entities),
            ),
            ("sections", Json::Arr(sections)),
            ("elements", Json::Arr(elements)),
            (
                "checks",
                Json::Arr(
                    self.account_verifications
                        .iter()
                        .flat_map(|file| &file.checks)
                        .filter(|check| {
                            check.claim == id
                                || check.cases.iter().any(|case| {
                                    self.find_case(case).is_some_and(|view| view.claim.id == id)
                                })
                        })
                        .map(|check| {
                            Json::obj(vec![
                                ("id", Json::str(&check.definition.id)),
                                ("rationale", Json::str(&check.definition.rationale)),
                                (
                                    "bindings",
                                    crate::verification_packages::bindings_json(&check.bindings),
                                ),
                                (
                                    "inputs",
                                    crate::verification_packages::map_json(&check.inputs),
                                ),
                                (
                                    "selectors",
                                    crate::verification_packages::map_json(&check.selectors),
                                ),
                                (
                                    "definition",
                                    Json::str(self.check_execution_fingerprint(&check.definition)),
                                ),
                                (
                                    "cases",
                                    Json::Arr(check.cases.iter().map(Json::str).collect()),
                                ),
                                (
                                    "mechanisms",
                                    Json::Arr(check.mechanisms.iter().map(Json::str).collect()),
                                ),
                                (
                                    "proposition",
                                    check.proposition.as_ref().map_or(Json::Null, Json::str),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "required_elements",
                Json::Arr(scope.required_elements.iter().map(Json::str).collect()),
            ),
        ])))
    }

    fn account_support_digest(&self, _spec: &str, claim: &str) -> Option<String> {
        let id = claim.to_string();
        let case_ids = self
            .find_claim(&id)
            .map(|view| {
                view.claim
                    .cases
                    .iter()
                    .map(|case| case.id.as_str())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let mut records = Vec::new();
        for support in &self.account_supports {
            let elements = support
                .elements
                .iter()
                .filter(|item| item.claim == id)
                .map(|item| {
                    Json::obj(vec![
                        ("element", Json::str(&item.element)),
                        (
                            "artifacts",
                            Json::Arr(item.artifacts.iter().map(Json::str).collect()),
                        ),
                        (
                            "depends_on",
                            Json::Arr(item.depends_on.iter().map(Json::str).collect()),
                        ),
                    ])
                })
                .collect::<Vec<_>>();
            let contributions = support
                .contributions
                .iter()
                .filter(|item| case_ids.contains(item.case.as_str()))
                .map(|item| {
                    Json::obj(vec![
                        ("check", Json::str(&item.check)),
                        ("case", Json::str(&item.case)),
                        ("element", Json::str(&item.element)),
                        ("proposition", Json::str(&item.proposition)),
                    ])
                })
                .collect::<Vec<_>>();
            if elements.is_empty() && contributions.is_empty() {
                continue;
            }
            records.push(Json::obj(vec![
                ("elements", Json::Arr(elements)),
                ("contributions", Json::Arr(contributions)),
                (
                    "checks",
                    Json::Arr(
                        support
                            .checks
                            .iter()
                            .map(|item| {
                                Json::obj(vec![
                                    ("id", Json::str(&item.id)),
                                    ("terminal", Json::str(&item.terminal)),
                                    (
                                        "artifacts",
                                        Json::Arr(item.artifacts.iter().map(Json::str).collect()),
                                    ),
                                    (
                                        "elements",
                                        Json::Arr(item.elements.iter().map(Json::str).collect()),
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
                (
                    "artifacts",
                    Json::Arr(
                        support
                            .artifacts
                            .iter()
                            .map(|item| {
                                Json::obj(vec![
                                    ("id", Json::str(&item.id)),
                                    ("kind", Json::str(&item.kind)),
                                    ("file", Json::str(&item.file)),
                                    ("fingerprint", Json::str(&item.fingerprint)),
                                    (
                                        "site",
                                        item.site.as_ref().map(Json::str).unwrap_or(Json::Null),
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]));
        }
        (!records.is_empty()).then(|| crate::fingerprint::canonical_sha256(&Json::Arr(records)))
    }

    pub fn cases(&self) -> impl Iterator<Item = CaseView<'_>> {
        self.specs.iter().flat_map(|spec| {
            spec.claims.iter().flat_map(move |claim| {
                claim
                    .cases
                    .iter()
                    .map(move |case| CaseView { spec, claim, case })
            })
        })
    }

    pub fn claims(&self) -> impl Iterator<Item = ClaimView<'_>> {
        self.specs.iter().flat_map(|spec| {
            spec.claims
                .iter()
                .map(move |claim| ClaimView { spec, claim })
        })
    }

    pub fn has_claim(&self, claim: &str) -> bool {
        self.specs
            .iter()
            .any(|s| s.claims.iter().any(|candidate| candidate.id == claim))
    }

    pub fn case_count(&self) -> usize {
        self.cases().count()
    }

    pub fn claim_count(&self) -> usize {
        self.claims().count()
    }

    pub fn find_claim(&self, claim: &str) -> Option<ClaimView<'_>> {
        self.claims().find(|candidate| candidate.claim.id == claim)
    }

    pub fn find_case(&self, id: &str) -> Option<CaseView<'_>> {
        self.cases().find(|case| case.id() == id)
    }

    pub fn checks(&self) -> impl Iterator<Item = &crate::verification::Check> {
        self.verifications
            .iter()
            .flat_map(|file| &file.checks)
            .chain(
                self.account_verifications
                    .iter()
                    .flat_map(|file| file.checks.iter().map(|check| &check.definition)),
            )
    }

    pub fn check_binding_count(&self, check: &str, case: &str) -> usize {
        self.evidence_bindings()
            .filter(|binding| binding.check == check && binding.case == case)
            .count()
            + self
                .account_verifications
                .iter()
                .flat_map(|file| &file.checks)
                .filter(|item| item.definition.id == check)
                .flat_map(|item| &item.bindings)
                .filter(|binding| binding.case == case)
                .count()
    }

    pub fn check_execution_fingerprint(&self, check: &crate::verification::Check) -> String {
        let base = crate::fingerprint::check_fingerprint(check, &self.check_implementations);
        if let Some(authored) = self
            .account_verifications
            .iter()
            .flat_map(|file| &file.checks)
            .find(|item| item.definition.id == check.id)
        {
            let mut dependencies = self
                .package_producers
                .iter()
                .filter(|producer| {
                    authored
                        .inputs
                        .values()
                        .any(|value| value == &producer.entity)
                })
                .collect::<Vec<_>>();
            dependencies.sort_by(|left, right| {
                (&left.package, &left.contract, &left.entity, &left.site).cmp(&(
                    &right.package,
                    &right.contract,
                    &right.entity,
                    &right.site,
                ))
            });
            let dependencies = dependencies
                .into_iter()
                .map(|producer| {
                    Json::obj(vec![
                        ("package", Json::str(&producer.package)),
                        ("contract", Json::str(&producer.contract)),
                        ("entity", Json::str(&producer.entity)),
                        ("site", Json::str(&producer.site)),
                        (
                            "source",
                            producer
                                .source
                                .as_ref()
                                .map_or(Json::Null, |source| Json::str(source.key())),
                        ),
                        (
                            "source_fingerprint",
                            Json::str(&producer.source_fingerprint),
                        ),
                        ("schema", producer.schema.clone().unwrap_or(Json::Null)),
                        ("inputs", producer.inputs_json()),
                    ])
                })
                .collect();
            crate::fingerprint::canonical_sha256(&Json::obj(vec![
                ("definition", Json::str(base)),
                (
                    "inputs",
                    crate::verification_packages::map_json(&authored.inputs),
                ),
                (
                    "selectors",
                    crate::verification_packages::map_json(&authored.selectors),
                ),
                ("producers", Json::Arr(dependencies)),
            ]))
        } else {
            base
        }
    }

    pub fn evidence_bindings(&self) -> impl Iterator<Item = &crate::verification::EvidenceBinding> {
        self.verifications.iter().flat_map(|file| &file.bindings)
    }

    pub fn method_qualifications(
        &self,
    ) -> impl Iterator<Item = &crate::verification::MethodQualification> {
        self.verifications
            .iter()
            .flat_map(|file| &file.method_qualifications)
    }

    pub fn applicability_decisions(
        &self,
    ) -> impl Iterator<Item = &crate::verification::ApplicabilityDecision> {
        self.verifications
            .iter()
            .flat_map(|file| &file.applicability_decisions)
    }

    pub fn claim_judgments(&self) -> impl Iterator<Item = &crate::verification::ClaimJudgment> {
        self.verifications
            .iter()
            .flat_map(|file| &file.claim_judgments)
    }

    pub fn challengers(&self) -> impl Iterator<Item = &crate::verification::Challenger> {
        self.verifications.iter().flat_map(|file| &file.challengers)
    }

    pub fn challenge_plans(&self) -> impl Iterator<Item = &crate::verification::ChallengePlan> {
        self.verifications
            .iter()
            .flat_map(|file| &file.challenge_plans)
    }

    /// Project-wide identity and cardinality checks run only after every authority is loaded.
    pub fn account_check_issues(&self) -> Vec<crate::diag::Diag> {
        crate::account_model::authored_check_issues(
            self.account_verifications.iter(),
            &self.specs,
            self.verifications.iter().flat_map(|file| &file.checks),
            &self.account_supports,
            &crate::account_model::mechanism_ids(self.account_designs.iter(), &self.designs),
        )
    }

    pub fn verification_declaration_issues(&self) -> Vec<crate::diag::Diag> {
        use crate::diag::Diag;
        use std::collections::{BTreeMap, BTreeSet};

        let mut issues = Vec::new();
        let mut check_ids = BTreeMap::new();
        let mut binding_ids = BTreeMap::new();
        let mut qualification_ids = BTreeMap::new();
        let mut applicability_ids = BTreeMap::new();
        let mut judgment_ids = BTreeMap::new();
        let mut challenger_ids = BTreeMap::new();
        let mut plan_ids = BTreeMap::new();
        let mut binding_pairs = BTreeSet::new();

        for file in &self.verifications {
            for check in &file.checks {
                record_global_id(
                    &mut check_ids,
                    "Check",
                    &check.id,
                    &check.path,
                    check.line,
                    &mut issues,
                );
            }
            for binding in &file.bindings {
                record_global_id(
                    &mut binding_ids,
                    "Evidence Binding",
                    &binding.id,
                    &binding.path,
                    binding.line,
                    &mut issues,
                );
                if !binding_pairs.insert((binding.check.clone(), binding.case.clone())) {
                    issues.push(Diag::at(
                        &binding.path,
                        binding.line,
                        format!(
                            "Check `{}` is already bound to Case `{}`",
                            binding.check, binding.case
                        ),
                    ));
                }
            }
            for qualification in &file.method_qualifications {
                record_global_id(
                    &mut qualification_ids,
                    "Method Qualification",
                    &qualification.id,
                    &qualification.path,
                    qualification.line,
                    &mut issues,
                );
            }
            for decision in &file.applicability_decisions {
                record_global_id(
                    &mut applicability_ids,
                    "Applicability Decision",
                    &decision.id,
                    &decision.path,
                    decision.line,
                    &mut issues,
                );
            }
            for judgment in &file.claim_judgments {
                record_global_id(
                    &mut judgment_ids,
                    "Claim Judgment",
                    &judgment.id,
                    &judgment.path,
                    judgment.line,
                    &mut issues,
                );
                match self.claims().find(|claim| claim.id() == judgment.id) {
                    None => issues.push(Diag::at(
                        &judgment.path,
                        judgment.line,
                        format!("Claim Judgment `{}` names no current Claim", judgment.id),
                    )),
                    Some(claim) if claim.claim.criticality == Some(Criticality::Routine) => {
                        issues.push(Diag::at(
                            &judgment.path,
                            judgment.line,
                            format!(
                                "routine Claim `{}` rejects a Claim Judgment declaration",
                                judgment.id
                            ),
                        ));
                    }
                    Some(_) => {}
                }
            }
            for challenger in &file.challengers {
                record_global_id(
                    &mut challenger_ids,
                    "Challenger",
                    &challenger.id,
                    &challenger.path,
                    challenger.line,
                    &mut issues,
                );
            }
            for plan in &file.challenge_plans {
                record_global_id(
                    &mut plan_ids,
                    "Challenge Plan",
                    &plan.id,
                    &plan.path,
                    plan.line,
                    &mut issues,
                );
            }
        }

        if let Some(standards) = &self.decision_standards {
            let scheduled = standards
                .schedule
                .gate_challenges
                .iter()
                .chain(&standards.schedule.scheduled_challenges)
                .cloned()
                .collect::<BTreeSet<_>>();
            let declared = self
                .challengers()
                .map(|challenger| challenger.form.clone())
                .chain(
                    standards
                        .policies
                        .iter()
                        .flat_map(|policy| policy.required_challenges.iter().cloned()),
                )
                .collect::<BTreeSet<_>>();
            for form in scheduled.difference(&declared) {
                issues.push(Diag::at(
                    &standards.schedule.path,
                    standards.schedule.line,
                    format!(
                        "scheduled Challenge form `{form}` has no Decision Policy or current \
                         Challenger"
                    ),
                ));
            }
            for challenger in self.challengers() {
                if !scheduled.contains(&challenger.form) {
                    issues.push(Diag::at(
                        &challenger.path,
                        challenger.line,
                        format!(
                            "Challenger form `{}` has no current scheduling lane",
                            challenger.form
                        ),
                    ));
                }
            }
        }

        issues
    }

    /// Semantic Case digest for Evidence Binding identity. Criticality and locations are omitted.
    pub fn case_digest(&self, case_id: &str) -> Option<String> {
        let case = self.find_case(case_id)?;
        Some(crate::fingerprint::canonical_sha256(&Json::obj(vec![
            ("format", Json::str("azimuth-case-digest")),
            ("version", Json::Num(2.0)),
            ("id", Json::str(case_id)),
            ("parent_claim", Json::str(&case.claim.id)),
            ("terms", vocabulary_json(case.spec)),
            ("claim", Json::str(&case.claim.statement)),
            ("domain", Json::str(case.claim.domain.name())),
            (
                "over",
                case.claim
                    .over
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            ("statement", Json::str(&case.case.statement)),
        ])))
    }

    pub fn claim_digest(&self, claim_id: &str) -> Option<String> {
        let claim = self.claims().find(|claim| claim.id() == claim_id)?;
        Some(crate::fingerprint::canonical_sha256(&Json::obj(vec![
            ("format", Json::str("azimuth-claim-digest")),
            ("version", Json::Num(2.0)),
            ("id", Json::str(claim_id)),
            ("terms", vocabulary_json(claim.spec)),
            ("predicate", Json::str(&claim.claim.statement)),
            ("domain", Json::str(claim.claim.domain.name())),
            (
                "over",
                claim
                    .claim
                    .over
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "cases",
                Json::Arr(
                    claim
                        .claim
                        .cases
                        .iter()
                        .map(|case| {
                            let id = case.id.clone();
                            Json::obj(vec![
                                ("id", Json::str(&id)),
                                (
                                    "semantic_digest",
                                    Json::str(self.case_digest(&id).unwrap_or_default()),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])))
    }

    pub fn expected_method_qualification_fingerprint(
        &self,
        qualification: &crate::verification::MethodQualification,
    ) -> Option<String> {
        let check = self
            .checks()
            .find(|check| check.id == qualification.check)?;
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == qualification.policy)?;
        Some(crate::fingerprint::method_qualification_fingerprint(
            qualification,
            &self.check_execution_fingerprint(check),
            &crate::fingerprint::policy_fingerprint(policy),
        ))
    }

    pub fn expected_applicability_fingerprint(
        &self,
        binding: &crate::verification::EvidenceBinding,
    ) -> Option<String> {
        let qualification = self
            .method_qualifications()
            .find(|qualification| qualification.id == binding.method_qualification)?;
        if qualification.check != binding.check {
            return None;
        }
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == binding.policy)?;
        let binding_fingerprint = crate::fingerprint::binding_fingerprint(
            binding,
            &self.case_digest(&binding.case)?,
            &self.expected_method_qualification_fingerprint(qualification)?,
            &crate::fingerprint::policy_fingerprint(policy),
        );
        Some(crate::fingerprint::applicability_decision_fingerprint(
            &binding_fingerprint,
        ))
    }

    /// Builds the exact total-composition preimage for one authored Claim Judgment.
    /// `None` means at least one required semantic dependency is absent or ambiguous.
    pub fn claim_judgment_preimage(
        &self,
        judgment: &crate::verification::ClaimJudgment,
    ) -> Option<Json> {
        let claim = self.claims().find(|claim| claim.id() == judgment.id)?;
        let criticality = claim.claim.criticality?;
        if criticality == Criticality::Routine {
            return None;
        }
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == judgment.policy)?;

        let mut obligation_areas = self
            .realization_obligation(&claim.spec.id, &claim.claim.id)
            .map(|obligation| obligation.areas.clone())
            .unwrap_or_default();
        obligation_areas.sort();
        if has_duplicates(&obligation_areas) {
            return None;
        }

        let surface = match &claim.claim.over {
            Some(id) => self.surface_account(id),
            None => Some(Json::Null),
        }?;

        let mut realization_sites = self
            .realizes
            .iter()
            .filter(|site| site.claim == claim.claim.id)
            .collect::<Vec<_>>();
        if realization_sites.is_empty() {
            return None;
        }
        realization_sites.sort_by_key(|site| site.source.as_ref().map(SourceIdentity::key));
        let mut realization_identities = BTreeSet::new();
        let mut realization_areas = BTreeSet::new();
        let mut realizations = Vec::new();
        for site in realization_sites {
            let source = site.source.as_ref()?;
            let identity = source.key();
            if site.source_fingerprint.is_empty()
                || !realization_identities.insert(identity.clone())
            {
                return None;
            }
            realization_areas.insert(source.area.clone());
            realizations.push(Json::obj(vec![
                ("identity", Json::str(identity)),
                ("source_fingerprint", Json::str(&site.source_fingerprint)),
            ]));
        }
        if obligation_areas
            .iter()
            .any(|area| !realization_areas.contains(area))
        {
            return None;
        }

        let mechanisms = self.mechanism_records(&claim)?;

        let case_ids = claim
            .claim
            .cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<BTreeSet<_>>();
        let mut bindings = self
            .evidence_bindings()
            .filter(|binding| case_ids.contains(binding.case.as_str()))
            .collect::<Vec<_>>();
        bindings.sort_by(|left, right| left.id.cmp(&right.id));
        if bindings.is_empty() || has_duplicates_by(&bindings, |binding| binding.id.clone()) {
            return None;
        }
        let mut binding_records = Vec::new();
        let mut method_qualification_records = BTreeSet::new();
        let mut applicability_records = Vec::new();
        for binding in bindings {
            let method_qualifications = self
                .method_qualifications()
                .filter(|qualification| qualification.id == binding.method_qualification)
                .collect::<Vec<_>>();
            let [method_qualification] = method_qualifications.as_slice() else {
                return None;
            };
            let binding_policy = self
                .decision_standards
                .as_ref()?
                .policies
                .iter()
                .find(|policy| policy.id == binding.policy)?;
            let binding_fingerprint = crate::fingerprint::binding_fingerprint(
                binding,
                &self.case_digest(&binding.case)?,
                &self.expected_method_qualification_fingerprint(method_qualification)?,
                &crate::fingerprint::policy_fingerprint(binding_policy),
            );
            binding_records.push(Json::obj(vec![
                ("id", Json::str(&binding.id)),
                ("fingerprint", Json::str(binding_fingerprint)),
            ]));
            method_qualification_records.insert((
                method_qualification.id.clone(),
                self.expected_method_qualification_fingerprint(method_qualification)?,
                method_qualification.verdict.name().to_string(),
            ));
            let decisions = self
                .applicability_decisions()
                .filter(|decision| decision.id == binding.id)
                .collect::<Vec<_>>();
            let [decision] = decisions.as_slice() else {
                return None;
            };
            applicability_records.push(Json::obj(vec![
                ("id", Json::str(&decision.id)),
                (
                    "expected_fingerprint",
                    Json::str(self.expected_applicability_fingerprint(binding)?),
                ),
                ("verdict", Json::str(decision.verdict.name())),
            ]));
        }
        let method_qualification_records = method_qualification_records
            .into_iter()
            .map(|(id, expected_fingerprint, verdict)| {
                Json::obj(vec![
                    ("id", Json::str(id)),
                    ("expected_fingerprint", Json::str(expected_fingerprint)),
                    ("verdict", Json::str(verdict)),
                ])
            })
            .collect();

        let mut claim_fields = vec![
            ("id", Json::str(&judgment.id)),
            (
                "semantic_digest",
                Json::str(self.claim_digest(&judgment.id)?),
            ),
            ("criticality", Json::str(criticality.name())),
            (
                "realization_obligation_areas",
                Json::Arr(obligation_areas.iter().map(Json::str).collect()),
            ),
            ("surface", surface),
        ];
        if let Some(digest) = self.account_design_digest(&claim.spec.id, &claim.claim.id) {
            claim_fields.push(("design_digest", Json::str(digest)));
        }
        if let Some(digest) = self.account_verification_digest(&claim.spec.id, &claim.claim.id) {
            claim_fields.push(("verification_digest", Json::str(digest)));
        }
        if let Some(digest) = self.account_support_digest(&claim.spec.id, &claim.claim.id) {
            claim_fields.push(("support_digest", Json::str(digest)));
        }
        Some(Json::obj(vec![
            ("format", Json::str("azimuth-claim-judgment-fingerprint")),
            ("version", Json::Num(1.0)),
            ("claim", Json::obj(claim_fields)),
            ("realizations", Json::Arr(realizations)),
            ("mechanisms", Json::Arr(mechanisms)),
            ("bindings", Json::Arr(binding_records)),
            (
                "method_qualifications",
                Json::Arr(method_qualification_records),
            ),
            ("applicability_decisions", Json::Arr(applicability_records)),
            (
                "policy_digest",
                Json::str(crate::fingerprint::policy_fingerprint(policy)),
            ),
            ("verdict", Json::str(judgment.verdict.name())),
            (
                "basis",
                Json::Arr(judgment.basis.iter().map(Json::str).collect()),
            ),
            (
                "residual_risks",
                Json::Arr(judgment.residual_risks.iter().map(Json::str).collect()),
            ),
        ]))
    }

    pub fn expected_claim_judgment_fingerprint(
        &self,
        judgment: &crate::verification::ClaimJudgment,
    ) -> Option<String> {
        Some(crate::fingerprint::claim_judgment_fingerprint(
            &self.claim_judgment_preimage(judgment)?,
        ))
    }

    /// Expands one selected resolution candidate into the exact model-semantic scope. The
    /// caller may union several projections before constructing a Run-bundle scope.
    pub fn challenge_candidate_scope(
        &self,
        candidate: &crate::validation::ChallengeCandidate,
    ) -> Option<SemanticChallengeScope> {
        use crate::validation::{CandidateDisposition, DecisionKind, RelationKind};

        if candidate.disposition != CandidateDisposition::Selected {
            return None;
        }
        let mut anchors = Vec::new();
        let mut inputs = Vec::new();
        match candidate.selector.from {
            RelationKind::Binding => {
                let binding = self
                    .evidence_bindings()
                    .find(|binding| binding.id == candidate.selector.id)?;
                anchors.push(self.binding_scope_component(binding)?);
            }
            RelationKind::Case => {
                anchors.push(self.case_scope_component(&candidate.selector.id)?);
            }
            RelationKind::Check => {
                anchors.push(self.check_scope_component(&candidate.selector.id)?);
            }
            RelationKind::Claim => {
                anchors.push(self.claim_scope_component(&candidate.selector.id)?);
            }
            RelationKind::MethodQualification => {
                let qualification = self
                    .method_qualifications()
                    .find(|qualification| qualification.id == candidate.selector.id)?;
                anchors.push(self.method_qualification_scope_component(qualification)?);
            }
            RelationKind::Mechanism => {
                let (mechanism, related_inputs) =
                    self.mechanism_scope_components(&candidate.selector.id)?;
                anchors.push(mechanism);
                inputs.extend(related_inputs);
            }
            RelationKind::Realization => {
                anchors.push(self.realization_scope_component(&candidate.selector.id)?);
            }
        }

        let target = candidate.target.as_ref()?;
        match target.kind {
            DecisionKind::ApplicabilityDecision => {
                let binding = self
                    .evidence_bindings()
                    .find(|binding| binding.id == target.id)?;
                let expected = self.expected_applicability_fingerprint(binding)?;
                if target.expected_fingerprint.as_deref() != Some(expected.as_str())
                    || target.authored_fingerprint.as_deref() != Some(expected.as_str())
                {
                    return None;
                }
                inputs.extend(self.applicability_scope_components(binding)?);
            }
            DecisionKind::MethodQualification => {
                let qualification = self
                    .method_qualifications()
                    .find(|qualification| qualification.id == target.id)?;
                let expected = self.expected_method_qualification_fingerprint(qualification)?;
                if target.expected_fingerprint.as_deref() != Some(expected.as_str())
                    || target.authored_fingerprint.as_deref() != Some(expected.as_str())
                {
                    return None;
                }
                inputs.extend(self.method_qualification_scope_components(qualification)?);
            }
            DecisionKind::ClaimJudgment => {
                let judgment = self
                    .claim_judgments()
                    .find(|judgment| judgment.id == target.id)?;
                let expected = self.expected_claim_judgment_fingerprint(judgment)?;
                if judgment.fingerprint != expected
                    || target.expected_fingerprint.as_deref() != Some(expected.as_str())
                    || target.authored_fingerprint.as_deref() != Some(expected.as_str())
                {
                    return None;
                }
                inputs.extend(self.claim_judgment_scope_components(&target.id)?);
            }
        }
        SemanticChallengeScope::merge([SemanticChallengeScope { anchors, inputs }])
    }

    fn claim_scope_component(&self, id: &str) -> Option<SemanticScopeComponent> {
        Some(scope_component(
            crate::verification::SemanticScopeKind::Claim,
            id,
            self.claim_digest(id)?,
        ))
    }

    fn case_scope_component(&self, id: &str) -> Option<SemanticScopeComponent> {
        Some(scope_component(
            crate::verification::SemanticScopeKind::Case,
            id,
            self.case_digest(id)?,
        ))
    }

    fn binding_scope_component(
        &self,
        binding: &crate::verification::EvidenceBinding,
    ) -> Option<SemanticScopeComponent> {
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == binding.policy)?;
        let qualification = self
            .method_qualifications()
            .find(|qualification| qualification.id == binding.method_qualification)?;
        Some(scope_component(
            crate::verification::SemanticScopeKind::Binding,
            &binding.id,
            crate::fingerprint::binding_fingerprint(
                binding,
                &self.case_digest(&binding.case)?,
                &self.expected_method_qualification_fingerprint(qualification)?,
                &crate::fingerprint::policy_fingerprint(policy),
            ),
        ))
    }

    fn check_scope_component(&self, id: &str) -> Option<SemanticScopeComponent> {
        let checks = self
            .checks()
            .filter(|check| check.id == id)
            .collect::<Vec<_>>();
        let [check] = checks.as_slice() else {
            return None;
        };
        Some(scope_component(
            crate::verification::SemanticScopeKind::Check,
            id,
            self.check_execution_fingerprint(check),
        ))
    }

    fn realization_scope_component(&self, id: &str) -> Option<SemanticScopeComponent> {
        let sites = self
            .realizes
            .iter()
            .filter(|site| {
                site.source
                    .as_ref()
                    .is_some_and(|source| source.key() == id)
            })
            .collect::<Vec<_>>();
        let site = *sites.first()?;
        if sites.iter().any(|candidate| {
            candidate.source_fingerprint != site.source_fingerprint
                || candidate.file != site.file
                || candidate.lang != site.lang
                || candidate.site != site.site
        }) {
            return None;
        }
        source_scope_component(
            crate::verification::SemanticScopeKind::Realization,
            id,
            &site.source_fingerprint,
            &site.file,
            &site.lang,
            &site.site,
        )
    }

    fn method_qualification_scope_component(
        &self,
        qualification: &crate::verification::MethodQualification,
    ) -> Option<SemanticScopeComponent> {
        Some(scope_component(
            crate::verification::SemanticScopeKind::MethodQualification,
            &qualification.id,
            self.expected_method_qualification_fingerprint(qualification)?,
        ))
    }

    fn method_qualification_scope_components(
        &self,
        qualification: &crate::verification::MethodQualification,
    ) -> Option<Vec<SemanticScopeComponent>> {
        use crate::verification::SemanticScopeKind;
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == qualification.policy)?;
        let mut components = vec![
            self.method_qualification_scope_component(qualification)?,
            self.check_scope_component(&qualification.check)?,
            scope_component(
                SemanticScopeKind::Context,
                &qualification.id,
                crate::fingerprint::context_fingerprint(&qualification.context),
            ),
            scope_component(
                SemanticScopeKind::Policy,
                &policy.id,
                crate::fingerprint::policy_fingerprint(policy),
            ),
        ];
        for implementation in self
            .check_implementations
            .iter()
            .filter(|implementation| implementation.check == qualification.check)
        {
            let identity = implementation.source.as_ref()?.key();
            components.push(source_scope_component(
                SemanticScopeKind::CheckImplementation,
                &identity,
                &implementation.source_fingerprint,
                &implementation.file,
                &implementation.lang,
                &implementation.site,
            )?);
        }
        normalize_scope_components(components)
    }

    fn applicability_scope_components(
        &self,
        binding: &crate::verification::EvidenceBinding,
    ) -> Option<Vec<SemanticScopeComponent>> {
        use crate::verification::SemanticScopeKind;

        let qualifications = self
            .method_qualifications()
            .filter(|qualification| qualification.id == binding.method_qualification)
            .collect::<Vec<_>>();
        let [qualification] = qualifications.as_slice() else {
            return None;
        };
        let decisions = self
            .applicability_decisions()
            .filter(|decision| decision.id == binding.id)
            .collect::<Vec<_>>();
        let [_decision] = decisions.as_slice() else {
            return None;
        };
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == binding.policy)?;
        let mut components = vec![
            scope_component(
                SemanticScopeKind::ApplicabilityDecision,
                &binding.id,
                self.expected_applicability_fingerprint(binding)?,
            ),
            self.binding_scope_component(binding)?,
            self.case_scope_component(&binding.case)?,
            self.claim_scope_component(&self.find_case(&binding.case)?.claim.id)?,
            scope_component(
                SemanticScopeKind::Context,
                &binding.id,
                crate::fingerprint::context_fingerprint(&binding.context),
            ),
            scope_component(
                SemanticScopeKind::Policy,
                &policy.id,
                crate::fingerprint::policy_fingerprint(policy),
            ),
        ];
        components.extend(self.method_qualification_scope_components(qualification)?);
        normalize_scope_components(components)
    }

    fn claim_judgment_scope_components(&self, id: &str) -> Option<Vec<SemanticScopeComponent>> {
        use crate::verification::SemanticScopeKind;

        let judgments = self
            .claim_judgments()
            .filter(|judgment| judgment.id == id)
            .collect::<Vec<_>>();
        let [judgment] = judgments.as_slice() else {
            return None;
        };
        self.claim_judgment_preimage(judgment)?;
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == judgment.policy)?;
        let claim = self.claims().find(|claim| claim.id() == id)?;
        let mut components = vec![
            scope_component(
                SemanticScopeKind::ClaimJudgment,
                id,
                judgment.fingerprint.clone(),
            ),
            self.claim_scope_component(id)?,
            scope_component(
                SemanticScopeKind::Policy,
                &policy.id,
                crate::fingerprint::policy_fingerprint(policy),
            ),
        ];
        for site in self
            .realizes
            .iter()
            .filter(|site| site.claim == claim.claim.id)
        {
            let identity = site.source.as_ref()?.key();
            components.push(source_scope_component(
                SemanticScopeKind::Realization,
                &identity,
                &site.source_fingerprint,
                &site.file,
                &site.lang,
                &site.site,
            )?);
        }
        if let Some(design) = self.design_for(&claim.spec.id) {
            for entry in design
                .entries
                .iter()
                .filter(|entry| entry.target.id() == claim.claim.id)
            {
                for mechanism in &entry.mechanisms {
                    let identity = mechanism.id.clone();
                    let (mechanism, related_inputs) = self.mechanism_scope_components(&identity)?;
                    components.push(mechanism);
                    components.extend(related_inputs);
                }
            }
        }
        for case in &claim.claim.cases {
            components.push(self.case_scope_component(&case.id)?);
        }
        let case_ids = self
            .find_claim(&id)
            .map(|view| {
                view.claim
                    .cases
                    .iter()
                    .map(|case| case.id.as_str())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        for binding in self
            .evidence_bindings()
            .filter(|binding| case_ids.contains(binding.case.as_str()))
        {
            components.extend(self.applicability_scope_components(binding)?);
        }
        if let Some(surface_id) = &claim.claim.over {
            components.extend(self.surface_scope_components(surface_id)?);
        }
        if let Some(obligation) = self.realization_obligation(&claim.spec.id, &claim.claim.id) {
            let mut areas = obligation.areas.clone();
            areas.sort();
            if has_duplicates(&areas) {
                return None;
            }
            components.push(scope_component(
                SemanticScopeKind::RealizationObligation,
                id,
                crate::fingerprint::realization_obligation_digest(id, &areas),
            ));
            components.extend(areas.iter().map(|area| {
                scope_component(
                    SemanticScopeKind::Area,
                    area,
                    crate::fingerprint::area_digest(area),
                )
            }));
        }
        normalize_scope_components(components)
    }

    fn mechanism_scope_components(
        &self,
        identity: &str,
    ) -> Option<(SemanticScopeComponent, Vec<SemanticScopeComponent>)> {
        use crate::verification::SemanticScopeKind;

        let mechanism_id = identity;
        let design = self.designs.iter().find(|design| {
            design.entries.iter().any(|entry| {
                entry
                    .mechanisms
                    .iter()
                    .any(|mechanism| mechanism.id == identity)
            })
        })?;
        let matches = design
            .entries
            .iter()
            .flat_map(|entry| {
                entry
                    .mechanisms
                    .iter()
                    .filter(move |mechanism| mechanism.id == mechanism_id)
                    .map(move |mechanism| (entry, mechanism))
            })
            .collect::<Vec<_>>();
        let [(entry, mechanism)] = matches.as_slice() else {
            return None;
        };
        let record = self.mechanism_record(design, entry, mechanism)?;
        let mut inputs = Vec::new();
        for artifact_account in record.get("artifacts")?.as_array()? {
            let artifact_id = artifact_account.get("id")?.as_str()?;
            let artifacts = self
                .artifacts
                .iter()
                .filter(|artifact| artifact.id == artifact_id)
                .collect::<Vec<_>>();
            let [artifact] = artifacts.as_slice() else {
                return None;
            };
            inputs.push(SemanticScopeComponent {
                kind: SemanticScopeKind::Artifact,
                id: artifact.id.clone(),
                fingerprint: crate::fingerprint::artifact_property_digest(artifact_account),
                locator: Some(SemanticScopeLocator::Artifact {
                    file: artifact.file.clone(),
                    artifact_kind: artifact.kind.clone(),
                    identity: artifact.source.as_ref()?.key(),
                    unique: artifact.unique,
                    columns: artifact.columns.clone(),
                    predicate: artifact.predicate.clone(),
                }),
            });
        }
        if mechanism.binding.is_none() {
            for implementation in self
                .mechanism_implementations
                .iter()
                .filter(|implementation| implementation.mechanism == mechanism_id)
            {
                let source = implementation.source.as_ref()?;
                inputs.push(source_scope_component(
                    SemanticScopeKind::MechanismImplementation,
                    &source.key(),
                    &implementation.source_fingerprint,
                    &implementation.file,
                    &implementation.lang,
                    &implementation.site,
                )?);
            }
        }
        Some((
            scope_component(
                SemanticScopeKind::Mechanism,
                identity,
                crate::fingerprint::mechanism_record_digest(&record),
            ),
            inputs,
        ))
    }

    fn surface_scope_components(&self, id: &str) -> Option<Vec<SemanticScopeComponent>> {
        use crate::verification::SemanticScopeKind;

        let account = self.surface_account(id)?;
        let surface = self.workspace.surface(id)?;
        let mut components = vec![scope_component(
            SemanticScopeKind::Surface,
            id,
            crate::fingerprint::surface_account_digest(&account),
        )];
        for contribution in &surface.contributions {
            components.push(scope_component(
                SemanticScopeKind::Area,
                &contribution.area,
                crate::fingerprint::area_digest(&contribution.area),
            ));
            let witnesses = self
                .enumerations
                .iter()
                .filter(|enumeration| {
                    enumeration.class == surface.id
                        && enumeration.kind == contribution.enumerator
                        && enumeration.identity.as_ref().is_some_and(|identity| {
                            identity.area == contribution.area
                                && identity.mount == contribution.mount
                        })
                })
                .collect::<Vec<_>>();
            let [witness] = witnesses.as_slice() else {
                return None;
            };
            let identity = witness.identity.as_ref()?.key();
            components.push(SemanticScopeComponent {
                kind: SemanticScopeKind::Enumeration,
                id: format!(
                    "{}|{}|{}|{}|{}",
                    id, contribution.area, contribution.mount, contribution.enumerator, identity
                ),
                fingerprint: witness.source_fingerprint.clone(),
                locator: Some(SemanticScopeLocator::Enumeration {
                    file: witness.source.clone(),
                    enumerator_kind: witness.kind.clone(),
                    identity,
                }),
            });
        }
        let behavioural = self
            .specs
            .iter()
            .find(|spec| spec.id == id)
            .into_iter()
            .flat_map(|spec| &spec.claims)
            .filter(|claim| claim.domain == Domain::Behaviour)
            .map(|claim| &claim.id)
            .collect::<BTreeSet<_>>();
        for site in self
            .realizes
            .iter()
            .filter(|site| site.spec == id && behavioural.contains(&site.claim))
        {
            let identity = site.source.as_ref()?.key();
            components.push(source_scope_component(
                SemanticScopeKind::SurfaceMember,
                &format!("{id}|tagged|{identity}"),
                &site.source_fingerprint,
                &site.file,
                &site.lang,
                &site.site,
            )?);
        }
        for member in self
            .class_members
            .iter()
            .filter(|member| member.class == id)
        {
            components.push(SemanticScopeComponent {
                kind: SemanticScopeKind::SurfaceMember,
                id: format!("{id}|enumerated|{}", member.file),
                fingerprint: crate::fingerprint::enumerated_surface_member_digest(id, &member.file),
                locator: Some(SemanticScopeLocator::EnumeratedSurfaceMember {
                    file: member.file.clone(),
                    language: member.lang.clone(),
                    site: member.site.clone(),
                }),
            });
        }
        normalize_scope_components(components)
    }

    fn mechanism_records(&self, claim: &ClaimView<'_>) -> Option<Vec<Json>> {
        let Some(design) = self.design_for(&claim.spec.id) else {
            return Some(Vec::new());
        };
        let mut attached = design
            .entries
            .iter()
            .filter(|entry| entry.target.id() == claim.claim.id)
            .flat_map(|entry| {
                entry
                    .mechanisms
                    .iter()
                    .map(move |mechanism| (entry, mechanism))
            })
            .collect::<Vec<_>>();
        attached.sort_by(|left, right| left.1.id.cmp(&right.1.id));
        if has_duplicates_by(&attached, |(_, mechanism)| mechanism.id.clone()) {
            return None;
        }
        attached
            .into_iter()
            .map(|(entry, mechanism)| self.mechanism_record(design, entry, mechanism))
            .collect()
    }

    fn mechanism_record(
        &self,
        _design: &crate::design::Design,
        entry: &crate::design::DesignEntry,
        mechanism: &crate::design::Mechanism,
    ) -> Option<Json> {
        let claim = self.find_claim(entry.target.id())?;
        if mechanism.cases.iter().any(|case| {
            !claim
                .claim
                .cases
                .iter()
                .any(|candidate| candidate.id == *case)
        }) {
            return None;
        }
        let implementations = self
            .mechanism_implementations
            .iter()
            .filter(|implementation| implementation.mechanism == mechanism.id)
            .collect::<Vec<_>>();
        let (artifact_ids, implementation_records) = match &mechanism.binding {
            Some(binding) if implementations.is_empty() => (vec![binding.as_str()], Vec::new()),
            Some(_) => return None,
            None if !implementations.is_empty() => {
                let mut records = Vec::new();
                for implementation in &implementations {
                    let source = implementation.source.as_ref()?;
                    if implementation.source_fingerprint.is_empty() {
                        return None;
                    }
                    records.push(Json::obj(vec![
                        ("identity", Json::str(source.key())),
                        (
                            "source_fingerprint",
                            Json::str(&implementation.source_fingerprint),
                        ),
                        ("artifact", Json::str(&implementation.binding)),
                    ]));
                }
                (
                    implementations
                        .iter()
                        .map(|item| item.binding.as_str())
                        .collect(),
                    records,
                )
            }
            None => return None,
        };
        let mut artifact_records = Vec::new();
        for artifact_id in artifact_ids {
            let artifacts = self
                .artifacts
                .iter()
                .filter(|artifact| artifact.id == artifact_id)
                .collect::<Vec<_>>();
            let [artifact] = artifacts.as_slice() else {
                return None;
            };
            let artifact_identity = artifact.source.as_ref()?.key();
            artifact_records.push(Json::obj(vec![
                ("id", Json::str(&artifact.id)),
                ("kind", Json::str(&artifact.kind)),
                ("identity", Json::str(artifact_identity)),
                (
                    "unique",
                    artifact.unique.map(Json::Bool).unwrap_or(Json::Null),
                ),
                (
                    "columns",
                    Json::Arr(artifact.columns.iter().map(Json::str).collect()),
                ),
                (
                    "predicate",
                    artifact
                        .predicate
                        .as_ref()
                        .map(Json::str)
                        .unwrap_or(Json::Null),
                ),
            ]));
        }
        artifact_records.sort_by_key(|record| {
            record
                .get("identity")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_string()
        });
        let mut implementation_records = implementation_records;
        implementation_records.sort_by_key(|record| {
            record
                .get("identity")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_string()
        });
        let attachment_kind = "claim";
        Some(Json::obj(vec![
            ("id", Json::str(mechanism.id.clone())),
            (
                "attachment",
                Json::obj(vec![
                    ("target_kind", Json::str(attachment_kind)),
                    ("target_id", Json::str(entry.target.id())),
                    (
                        "cases",
                        Json::Arr(mechanism.cases.iter().map(Json::str).collect()),
                    ),
                ]),
            ),
            ("enforcement", Json::str(mechanism.kind.name())),
            (
                "expect",
                Json::obj(vec![
                    (
                        "unique",
                        mechanism
                            .expected_unique
                            .map(Json::Bool)
                            .unwrap_or(Json::Null),
                    ),
                    (
                        "columns",
                        Json::Arr(mechanism.expected_columns.iter().map(Json::str).collect()),
                    ),
                    (
                        "predicate",
                        mechanism
                            .expected_predicate
                            .as_ref()
                            .map(Json::str)
                            .unwrap_or(Json::Null),
                    ),
                ]),
            ),
            ("artifacts", Json::Arr(artifact_records)),
            ("implementations", Json::Arr(implementation_records)),
        ]))
    }

    fn surface_account(&self, id: &str) -> Option<Json> {
        let surface = self.workspace.surface(id)?;
        let mut contributions = Vec::new();
        let mut contribution_keys = BTreeSet::new();
        for contribution in &surface.contributions {
            let witnesses = self
                .enumerations
                .iter()
                .filter(|enumeration| {
                    enumeration.class == surface.id
                        && enumeration.kind == contribution.enumerator
                        && enumeration.identity.as_ref().is_some_and(|identity| {
                            identity.area == contribution.area
                                && identity.mount == contribution.mount
                        })
                })
                .collect::<Vec<_>>();
            let [witness] = witnesses.as_slice() else {
                return None;
            };
            let identity = witness.identity.as_ref()?.key();
            if witness.source_fingerprint.is_empty() {
                return None;
            }
            let key = (
                contribution.area.clone(),
                contribution.mount.clone(),
                contribution.enumerator.clone(),
                identity.clone(),
            );
            if !contribution_keys.insert(key.clone()) {
                return None;
            }
            contributions.push((
                key,
                Json::obj(vec![
                    ("area", Json::str(&contribution.area)),
                    ("mount", Json::str(&contribution.mount)),
                    ("enumerator", Json::str(&contribution.enumerator)),
                    (
                        "witness",
                        Json::obj(vec![
                            ("kind", Json::str(&witness.kind)),
                            ("identity", Json::str(identity)),
                            ("source_fingerprint", Json::str(&witness.source_fingerprint)),
                        ]),
                    ),
                ]),
            ));
        }
        contributions.sort_by(|left, right| left.0.cmp(&right.0));
        let contributions = contributions
            .into_iter()
            .map(|(_, contribution)| contribution)
            .collect();

        let class_spec = self.specs.iter().find(|spec| spec.id == surface.id);
        let behavioural = class_spec
            .into_iter()
            .flat_map(|spec| &spec.claims)
            .filter(|claim| claim.domain == Domain::Behaviour)
            .map(|claim| &claim.id)
            .collect::<BTreeSet<_>>();
        let mut members = Vec::new();
        let mut member_keys = BTreeSet::new();
        for site in self
            .realizes
            .iter()
            .filter(|site| site.spec == surface.id && behavioural.contains(&site.claim))
        {
            let identity = site.source.as_ref()?.key();
            if site.source_fingerprint.is_empty()
                || !member_keys.insert(("tagged", identity.clone()))
            {
                return None;
            }
            members.push(Json::obj(vec![
                ("kind", Json::str("tagged")),
                ("identity", Json::str(identity)),
                ("source_fingerprint", Json::str(&site.source_fingerprint)),
            ]));
        }
        for member in self
            .class_members
            .iter()
            .filter(|member| member.class == surface.id)
        {
            if !member_keys.insert(("enumerated", member.file.clone())) {
                return None;
            }
            members.push(Json::obj(vec![
                ("kind", Json::str("enumerated")),
                ("file", Json::str(&member.file)),
            ]));
        }
        members.sort_by(|left, right| {
            let key = |item: &Json| {
                let kind = item.get("kind").and_then(Json::as_str).unwrap_or_default();
                let rank = if kind == "tagged" { 0 } else { 1 };
                let identity = item
                    .get("identity")
                    .or_else(|| item.get("file"))
                    .and_then(Json::as_str)
                    .unwrap_or_default();
                (rank, identity.to_string())
            };
            key(left).cmp(&key(right))
        });
        Some(Json::obj(vec![
            ("id", Json::str(&surface.id)),
            ("contributions", Json::Arr(contributions)),
            ("members", Json::Arr(members)),
        ]))
    }

    pub fn design_for(&self, spec: &str) -> Option<&crate::design::Design> {
        self.designs.iter().find(|d| d.spec == spec)
    }

    pub fn mechanism_bindings<'a>(
        &'a self,
        _spec: &str,
        mechanism: &'a crate::design::Mechanism,
    ) -> Vec<&'a str> {
        let mut bindings = Vec::new();
        if let Some(binding) = mechanism.binding.as_deref() {
            bindings.push(binding);
        }
        bindings.extend(
            self.mechanism_implementations
                .iter()
                .filter(|implementation| implementation.mechanism == mechanism.id)
                .map(|implementation| implementation.binding.as_str()),
        );
        bindings
    }

    /// The export is the extension seam. Validation, dashboards and PR annotations
    /// consume this model; nothing else re-parses specs.
    pub fn to_json(&self, findings: &[crate::validation::Finding]) -> Json {
        let specs = self
            .specs
            .iter()
            .map(|spec| {
                let claims = spec
                    .claims
                    .iter()
                    .map(|r| {
                        let cases = r
                            .cases
                            .iter()
                            .map(|sc| {
                                Json::obj(vec![
                                    ("id", Json::str(&sc.id)),
                                    ("line", Json::Num(sc.line as f64)),
                                    ("statement", Json::str(&sc.statement)),
                                ])
                            })
                            .collect();
                        Json::obj(vec![
                            ("id", Json::str(&r.id)),
                            (
                                "criticality",
                                match r.criticality {
                                    Some(c) => Json::str(c.name()),
                                    None => Json::Null,
                                },
                            ),
                            ("statement", Json::str(&r.statement)),
                            ("line", Json::Num(r.line as f64)),
                            ("domain", Json::str(r.domain.name())),
                            ("over", r.over.as_ref().map(Json::str).unwrap_or(Json::Null)),
                            ("cases", Json::Arr(cases)),
                        ])
                    })
                    .collect();
                Json::obj(vec![
                    ("id", Json::str(&spec.id)),
                    ("path", Json::str(&spec.path)),
                    ("terms", vocabulary_json(spec)),
                    ("claims", Json::Arr(claims)),
                ])
            })
            .collect();

        Json::obj(vec![
            ("version", Json::Num(5.0)),
            (
                "extensions",
                Json::Arr(
                    self.package_producers
                        .iter()
                        .map(crate::verification_packages::Producer::to_json)
                        .collect(),
                ),
            ),
            ("specs", Json::Arr(specs)),
            (
                "realizes",
                Json::Arr(
                    self.realizes
                        .iter()
                        .map(|site| {
                            site_json(
                                site,
                                self.workspace
                                    .area_for_file(&site.file)
                                    .map(|area| area.id.as_str()),
                            )
                        })
                        .collect(),
                ),
            ),
            ("workspace", workspace_json(self)),
            (
                "mechanism_implementations",
                Json::Arr(
                    self.mechanism_implementations
                        .iter()
                        .map(mechanism_implementation_json)
                        .collect(),
                ),
            ),
            (
                "account_checks",
                Json::Arr(
                    self.account_verifications
                        .iter()
                        .flat_map(|file| &file.checks)
                        .map(|check| {
                            Json::obj(vec![
                                ("definition", check_json(self, &check.definition)),
                                ("rationale", Json::str(&check.definition.rationale)),
                                (
                                    "bindings",
                                    crate::verification_packages::bindings_json(&check.bindings),
                                ),
                                (
                                    "inputs",
                                    crate::verification_packages::map_json(&check.inputs),
                                ),
                                (
                                    "selectors",
                                    crate::verification_packages::map_json(&check.selectors),
                                ),
                                ("claim", Json::str(&check.claim)),
                                (
                                    "cases",
                                    Json::Arr(check.cases.iter().map(Json::str).collect()),
                                ),
                                (
                                    "mechanisms",
                                    Json::Arr(check.mechanisms.iter().map(Json::str).collect()),
                                ),
                                (
                                    "proposition",
                                    check.proposition.as_ref().map_or(Json::Null, Json::str),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "check_implementations",
                Json::Arr(
                    self.check_implementations
                        .iter()
                        .map(check_implementation_json)
                        .collect(),
                ),
            ),
            (
                "class_members",
                Json::Arr(
                    self.class_members
                        .iter()
                        .map(|member| {
                            let mut fields = vec![
                                ("class".to_string(), Json::str(&member.class)),
                                ("site".to_string(), Json::str(&member.site)),
                                ("file".to_string(), Json::str(&member.file)),
                                ("lang".to_string(), Json::str(&member.lang)),
                            ];
                            append_source(&mut fields, member.source.as_ref());
                            Json::Obj(fields)
                        })
                        .collect(),
                ),
            ),
            (
                "enumerations",
                Json::Arr(
                    self.enumerations
                        .iter()
                        .map(|e| {
                            let mut fields = vec![
                                ("class".to_string(), Json::str(&e.class)),
                                ("kind".to_string(), Json::str(&e.kind)),
                                ("source".to_string(), Json::str(&e.source)),
                                (
                                    "source_fingerprint".to_string(),
                                    Json::str(&e.source_fingerprint),
                                ),
                            ];
                            append_source(&mut fields, e.identity.as_ref());
                            Json::Obj(fields)
                        })
                        .collect(),
                ),
            ),
            (
                "artifacts",
                Json::Arr(
                    self.artifacts
                        .iter()
                        .map(|artifact| {
                            let mut fields = vec![
                                ("id".to_string(), Json::str(&artifact.id)),
                                ("kind".to_string(), Json::str(&artifact.kind)),
                                ("file".to_string(), Json::str(&artifact.file)),
                            ];
                            if let Some(unique) = artifact.unique {
                                fields.push(("unique".to_string(), Json::Bool(unique)));
                            }
                            if !artifact.columns.is_empty() {
                                fields.push((
                                    "columns".to_string(),
                                    Json::Arr(artifact.columns.iter().map(Json::str).collect()),
                                ));
                            }
                            if let Some(predicate) = &artifact.predicate {
                                fields.push(("predicate".to_string(), Json::str(predicate)));
                            }
                            append_source(&mut fields, artifact.source.as_ref());
                            Json::Obj(fields)
                        })
                        .collect(),
                ),
            ),
            ("mechanisms", Json::Arr(self.mechanism_json())),
            (
                "checks",
                Json::Arr(self.checks().map(|check| check_json(self, check)).collect()),
            ),
            (
                "evidence_bindings",
                Json::Arr(
                    self.evidence_bindings()
                        .map(|binding| binding_json(self, binding))
                        .collect(),
                ),
            ),
            (
                "method_qualifications",
                Json::Arr(
                    self.method_qualifications()
                        .map(|item| method_qualification_json(self, item))
                        .collect(),
                ),
            ),
            (
                "applicability_decisions",
                Json::Arr(
                    self.applicability_decisions()
                        .map(|decision| applicability_decision_json(self, decision))
                        .collect(),
                ),
            ),
            (
                "claim_judgments",
                Json::Arr(
                    self.claim_judgments()
                        .map(|judgment| claim_judgment_json(self, judgment))
                        .collect(),
                ),
            ),
            (
                "decision_policies",
                Json::Arr(
                    self.decision_standards
                        .as_ref()
                        .into_iter()
                        .flat_map(|standards| &standards.policies)
                        .map(decision_policy_json)
                        .collect(),
                ),
            ),
            (
                "challenge_schedule",
                self.decision_standards
                    .as_ref()
                    .map(|standards| challenge_schedule_json(&standards.schedule))
                    .unwrap_or(Json::Null),
            ),
            (
                "challengers",
                Json::Arr(self.challengers().map(challenger_json).collect()),
            ),
            (
                "challenge_plans",
                Json::Arr(self.challenge_plans().map(challenge_plan_json).collect()),
            ),
            (
                "challenge_resolutions",
                Json::Arr(self.challenge_resolution_json()),
            ),
            (
                "findings",
                Json::Arr(findings.iter().map(|h| h.to_json()).collect()),
            ),
        ])
    }
}

impl Model {
    fn challenge_resolution_json(&self) -> Vec<Json> {
        let mut resolutions = self
            .challenge_plans()
            .map(|plan| crate::validation::resolve_challenge_plan(self, plan))
            .collect::<Vec<_>>();
        resolutions.sort_by(|left, right| {
            (&left.plan, &left.challenger).cmp(&(&right.plan, &right.challenger))
        });
        resolutions
            .iter()
            .map(crate::validation::ChallengeResolution::to_json)
            .collect()
    }

    fn mechanism_json(&self) -> Vec<Json> {
        let mut out = Vec::new();
        for design in &self.designs {
            for entry in &design.entries {
                for m in &entry.mechanisms {
                    let mut bindings = self.mechanism_bindings(&design.spec, m);
                    bindings.sort_unstable();
                    out.push(Json::obj(vec![
                        ("spec", Json::str(&design.spec)),
                        ("target_kind", Json::str("claim")),
                        ("target", Json::str(entry.target.id())),
                        ("id", Json::str(&m.id)),
                        ("cases", Json::Arr(m.cases.iter().map(Json::str).collect())),
                        ("enforcement", Json::str(m.kind.name())),
                        ("rung", Json::Num(m.kind.rung() as f64)),
                        (
                            "binding",
                            if bindings.len() == 1 {
                                Json::str(bindings[0])
                            } else {
                                Json::Null
                            },
                        ),
                        (
                            "bindings",
                            Json::Arr(bindings.iter().map(|binding| Json::str(*binding)).collect()),
                        ),
                        (
                            "expected_unique",
                            m.expected_unique.map(Json::Bool).unwrap_or(Json::Null),
                        ),
                        (
                            "expected_columns",
                            Json::Arr(m.expected_columns.iter().map(Json::str).collect()),
                        ),
                        (
                            "expected_predicate",
                            m.expected_predicate
                                .as_ref()
                                .map(Json::str)
                                .unwrap_or(Json::Null),
                        ),
                    ]));
                }
            }
        }
        out
    }
}

fn site_json(s: &Site, derived_area: Option<&str>) -> Json {
    let mut pairs = vec![
        ("claim".to_string(), Json::str(&s.claim)),
        ("site".to_string(), Json::str(&s.site)),
        ("file".to_string(), Json::str(&s.file)),
        ("lang".to_string(), Json::str(&s.lang)),
    ];
    if !s.source_fingerprint.is_empty() {
        pairs.push((
            "source_fingerprint".to_string(),
            Json::str(&s.source_fingerprint),
        ));
    }
    if let Some(source) = &s.source {
        pairs.push(("area".to_string(), Json::str(&source.area)));
        pairs.push(("address_kind".to_string(), Json::str(&source.kind)));
        pairs.push(("address".to_string(), Json::str(&source.address)));
        pairs.push(("mount".to_string(), Json::str(&source.mount)));
    } else if let Some(area) = derived_area {
        pairs.push(("derived_area".to_string(), Json::str(area)));
    }
    Json::Obj(pairs)
}

fn workspace_json(model: &Model) -> Json {
    let workspace = &model.workspace;
    Json::Obj(vec![
        ("path".into(), Json::str(&workspace.path)),
        (
            "packages".into(),
            Json::Arr(workspace.packages.iter().map(Json::str).collect()),
        ),
        (
            "areas".into(),
            Json::Arr(
                workspace
                    .areas
                    .iter()
                    .map(|area| {
                        Json::Obj(vec![
                            ("id".into(), Json::str(&area.id)),
                            (
                                "mounts".into(),
                                Json::Arr(
                                    area.mounts
                                        .iter()
                                        .map(|mount| {
                                            Json::Obj(vec![
                                                ("id".into(), Json::str(&mount.id)),
                                                ("path".into(), Json::str(&mount.path)),
                                            ])
                                        })
                                        .collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "surfaces".into(),
            Json::Arr(
                workspace
                    .surfaces
                    .iter()
                    .map(|surface| {
                        Json::Obj(vec![
                            ("id".into(), Json::str(&surface.id)),
                            (
                                "contributions".into(),
                                Json::Arr(
                                    surface
                                        .contributions
                                        .iter()
                                        .map(|item| {
                                            Json::Obj(vec![
                                                ("area".into(), Json::str(&item.area)),
                                                ("mount".into(), Json::str(&item.mount)),
                                                ("enumerator".into(), Json::str(&item.enumerator)),
                                            ])
                                        })
                                        .collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "realization_obligations".into(),
            Json::Arr(
                model
                    .realization_obligations()
                    .iter()
                    .map(|item| {
                        Json::Obj(vec![
                            ("spec".into(), Json::str(&item.spec)),
                            ("claim".into(), Json::str(&item.claim)),
                            (
                                "areas".into(),
                                Json::Arr(item.areas.iter().map(Json::str).collect()),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn mechanism_implementation_json(item: &MechanismImplementation) -> Json {
    let mut fields = vec![
        ("mechanism".to_string(), Json::str(&item.mechanism)),
        ("site".to_string(), Json::str(&item.site)),
        ("binding".to_string(), Json::str(&item.binding)),
        ("file".to_string(), Json::str(&item.file)),
        ("lang".to_string(), Json::str(&item.lang)),
        (
            "source_fingerprint".to_string(),
            Json::str(&item.source_fingerprint),
        ),
    ];
    append_source(&mut fields, item.source.as_ref());
    Json::Obj(fields)
}

fn check_implementation_json(item: &CheckImplementation) -> Json {
    let mut fields = vec![
        ("check".to_string(), Json::str(&item.check)),
        ("site".to_string(), Json::str(&item.site)),
        ("file".to_string(), Json::str(&item.file)),
        ("lang".to_string(), Json::str(&item.lang)),
        (
            "source_fingerprint".to_string(),
            Json::str(&item.source_fingerprint),
        ),
    ];
    append_source(&mut fields, item.source.as_ref());
    Json::Obj(fields)
}

fn check_json(model: &Model, item: &crate::verification::Check) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        (
            "methods",
            Json::Arr(item.methods.iter().map(Json::str).collect()),
        ),
        (
            "terminal",
            if model
                .account_verifications
                .iter()
                .flat_map(|file| &file.checks)
                .any(|check| check.definition.id == item.id && !check.bindings.is_empty())
            {
                Json::Null
            } else {
                Json::str(&item.terminal)
            },
        ),
        (
            "fingerprint",
            Json::str(model.check_execution_fingerprint(item)),
        ),
    ])
}

fn binding_json(model: &Model, item: &crate::verification::EvidenceBinding) -> Json {
    let mut fields = vec![
        ("id".to_string(), Json::str(&item.id)),
        ("check".to_string(), Json::str(&item.check)),
        ("case".to_string(), Json::str(&item.case)),
        (
            "method_qualification".to_string(),
            Json::str(&item.method_qualification),
        ),
        ("proposition".to_string(), Json::str(&item.proposition)),
        (
            "context".to_string(),
            crate::verification::context_json(&item.context),
        ),
        (
            "challenge_domain".to_string(),
            Json::Arr(
                item.challenge_domain
                    .iter()
                    .map(|domain| Json::str(domain.name()))
                    .collect(),
            ),
        ),
        ("policy".to_string(), Json::str(&item.policy)),
        (
            "context_fingerprint".to_string(),
            Json::str(crate::fingerprint::context_fingerprint(&item.context)),
        ),
    ];
    if let Some(expected) = model.expected_applicability_fingerprint(item) {
        fields.push(("applicability_fingerprint".to_string(), Json::str(expected)));
    }
    Json::Obj(fields)
}

fn method_qualification_json(
    model: &Model,
    item: &crate::verification::MethodQualification,
) -> Json {
    let mut fields = vec![
        ("id", Json::str(&item.id)),
        ("check", Json::str(&item.check)),
        ("scope", Json::str(item.scope.name())),
        ("quantification", Json::str(item.quantification.name())),
        ("oracle", Json::str(item.oracle.name())),
        ("context", crate::verification::context_json(&item.context)),
        (
            "challenge_domain",
            Json::Arr(
                item.challenge_domain
                    .iter()
                    .map(|domain| Json::str(domain.name()))
                    .collect(),
            ),
        ),
        ("policy", Json::str(&item.policy)),
        ("verdict", Json::str(item.verdict.name())),
        ("fingerprint", Json::str(&item.fingerprint)),
        ("qualified", Json::str(&item.qualified)),
        ("qualifier", Json::str(&item.qualifier)),
    ];
    if let Some(expected) = model.expected_method_qualification_fingerprint(item) {
        fields.push(("expected_fingerprint", Json::str(expected)));
    }
    Json::obj(fields)
}

fn applicability_decision_json(
    model: &Model,
    item: &crate::verification::ApplicabilityDecision,
) -> Json {
    let binding = model
        .evidence_bindings()
        .find(|binding| binding.id == item.id);
    let mut fields = vec![
        ("id".to_string(), Json::str(&item.id)),
        ("verdict".to_string(), Json::str(item.verdict.name())),
        ("fingerprint".to_string(), Json::str(&item.fingerprint)),
        ("decided".to_string(), Json::str(&item.decided)),
        ("decider".to_string(), Json::str(&item.decider)),
    ];
    if let Some(expected) =
        binding.and_then(|binding| model.expected_applicability_fingerprint(binding))
    {
        fields.push(("expected_fingerprint".to_string(), Json::str(expected)));
    }
    Json::Obj(fields)
}

fn claim_judgment_json(model: &Model, item: &crate::verification::ClaimJudgment) -> Json {
    let mut fields = vec![
        ("id".to_string(), Json::str(&item.id)),
        ("verdict".to_string(), Json::str(item.verdict.name())),
        ("policy".to_string(), Json::str(&item.policy)),
        ("fingerprint".to_string(), Json::str(&item.fingerprint)),
        ("judged".to_string(), Json::str(&item.judged)),
        ("judge".to_string(), Json::str(&item.judge)),
        (
            "basis".to_string(),
            Json::Arr(item.basis.iter().map(Json::str).collect()),
        ),
        (
            "residual_risks".to_string(),
            Json::Arr(item.residual_risks.iter().map(Json::str).collect()),
        ),
    ];
    if let Some(expected) = model.expected_claim_judgment_fingerprint(item) {
        fields.push(("expected_fingerprint".to_string(), Json::str(expected)));
    }
    Json::Obj(fields)
}

fn decision_policy_json(item: &crate::verification::DecisionPolicy) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        (
            "required_challenges",
            Json::Arr(item.required_challenges.iter().map(Json::str).collect()),
        ),
        (
            "digest",
            Json::str(crate::fingerprint::policy_fingerprint(item)),
        ),
    ])
}

fn challenge_schedule_json(item: &crate::verification::ChallengeSchedule) -> Json {
    Json::obj(vec![
        (
            "gate_challenges",
            Json::Arr(item.gate_challenges.iter().map(Json::str).collect()),
        ),
        (
            "scheduled_challenges",
            Json::Arr(item.scheduled_challenges.iter().map(Json::str).collect()),
        ),
        (
            "digest",
            Json::str(crate::fingerprint::schedule_fingerprint(item)),
        ),
    ])
}

fn challenger_json(item: &crate::verification::Challenger) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("form", Json::str(&item.form)),
        ("searches_for", Json::str(&item.searches_for)),
        (
            "required_scope",
            Json::Arr(
                item.required_scope
                    .iter()
                    .map(|kind| Json::str(kind.name()))
                    .collect(),
            ),
        ),
        (
            "fingerprint",
            Json::str(crate::fingerprint::challenger_fingerprint(item)),
        ),
    ])
}

fn challenge_plan_json(item: &crate::verification::ChallengePlan) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("challenger", Json::str(&item.challenger)),
        (
            "selectors",
            Json::Arr(
                item.selectors
                    .iter()
                    .map(|selector| Json::str(selector.canonical()))
                    .collect(),
            ),
        ),
    ])
}

fn append_source(fields: &mut Vec<(String, Json)>, source: Option<&SourceIdentity>) {
    if let Some(source) = source {
        fields.push(("area".to_string(), Json::str(&source.area)));
        fields.push(("address_kind".to_string(), Json::str(&source.kind)));
        fields.push(("address".to_string(), Json::str(&source.address)));
        fields.push(("mount".to_string(), Json::str(&source.mount)));
    }
}

fn scope_component(
    kind: crate::verification::SemanticScopeKind,
    id: &str,
    fingerprint: String,
) -> SemanticScopeComponent {
    SemanticScopeComponent {
        kind,
        id: id.to_string(),
        fingerprint,
        locator: None,
    }
}

fn source_scope_component(
    kind: crate::verification::SemanticScopeKind,
    id: &str,
    fingerprint: &str,
    file: &str,
    language: &str,
    site: &str,
) -> Option<SemanticScopeComponent> {
    if id.is_empty()
        || fingerprint.is_empty()
        || file.is_empty()
        || language.is_empty()
        || site.is_empty()
    {
        return None;
    }
    Some(SemanticScopeComponent {
        kind,
        id: id.to_string(),
        fingerprint: fingerprint.to_string(),
        locator: Some(SemanticScopeLocator::Source {
            file: file.to_string(),
            language: language.to_string(),
            site: site.to_string(),
        }),
    })
}

fn normalize_scope_components(
    components: Vec<SemanticScopeComponent>,
) -> Option<Vec<SemanticScopeComponent>> {
    use std::collections::BTreeMap;

    let mut normalized = BTreeMap::new();
    for component in components {
        if component.id.is_empty() || component.fingerprint.is_empty() {
            return None;
        }
        let key = (component.kind, component.id.clone());
        match normalized.get(&key) {
            Some(previous) if previous != &component => return None,
            Some(_) => {}
            None => {
                normalized.insert(key, component);
            }
        }
    }
    Some(normalized.into_values().collect())
}

fn record_global_id(
    seen: &mut std::collections::BTreeMap<String, String>,
    kind: &str,
    id: &str,
    path: &str,
    line: usize,
    issues: &mut Vec<crate::diag::Diag>,
) {
    if let Some(first_path) = seen.insert(id.to_string(), path.to_string()) {
        issues.push(crate::diag::Diag::at(
            path,
            line,
            format!("{kind} `{id}` is already declared by {first_path}"),
        ));
    }
}

fn has_duplicates<T: PartialEq>(items: &[T]) -> bool {
    items.windows(2).any(|pair| pair[0] == pair[1])
}

fn has_duplicates_by<T, K: PartialEq>(items: &[T], key: impl Fn(&T) -> K) -> bool {
    items.windows(2).any(|pair| key(&pair[0]) == key(&pair[1]))
}
