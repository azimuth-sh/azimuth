//! The derived model.
//!
//! `requirement = (domain, predicate)`. Scenarios are normative constituents of that predicate; they are
//! addressable without becoming independent assurance centres.

use crate::json::Json;
use std::collections::BTreeMap;

/// The review rigor a Requirement declares on one axis. Absence means the axis is not declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rigor {
    Reviewed,
}

impl Rigor {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "reviewed" => Some(Rigor::Reviewed),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Rigor::Reviewed => "reviewed",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafType {
    String,
    Boolean,
    Integer,
}

impl LeafType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "string" => Some(Self::String),
            "boolean" => Some(Self::Boolean),
            "integer" => Some(Self::Integer),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
}

impl ConditionValue {
    pub fn leaf_type(&self) -> LeafType {
        match self {
            Self::Boolean(_) => LeafType::Boolean,
            Self::Integer(_) => LeafType::Integer,
            Self::Text(_) => LeafType::String,
        }
    }

    pub fn render(&self) -> String {
        match self {
            Self::Boolean(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Text(value) => format!("\"{value}\""),
        }
    }

    fn to_json(&self) -> Json {
        match self {
            Self::Boolean(value) => Json::Bool(*value),
            Self::Integer(value) => Json::Num(*value as f64),
            Self::Text(value) => Json::str(value),
        }
    }
}

/// One `Context: <component>: <leaf> = <value>` line. All conditions of a Requirement and of a
/// Scenario hold together; the line locates diagnostics and takes no part in identity.
#[derive(Debug, Clone)]
pub struct Condition {
    pub component: String,
    pub leaf: String,
    pub value: ConditionValue,
    pub line: usize,
}

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component && self.leaf == other.leaf && self.value == other.value
    }
}

impl Eq for Condition {}

impl Condition {
    pub const LABEL: &'static str = "Context:";
    pub const EXPECTED: &'static str = "`Context: <component-id>: <leaf> = <value>` with a value of true, false, an integer or a double-quoted string";

    pub fn parse(text: &str, line: usize) -> Result<Self, String> {
        let (component, rest) = text
            .split_once(':')
            .ok_or_else(|| "missing `:` after the component id".to_string())?;
        let component = component.trim();
        crate::diag::validate_id(component, false)
            .map_err(|why| format!("invalid component id: {why}"))?;
        let (leaf, value) = rest
            .split_once('=')
            .ok_or_else(|| "missing `=` after the leaf".to_string())?;
        let leaf = leaf.trim();
        if !is_leaf_path(leaf) {
            return Err(format!(
                "invalid leaf `{leaf}`; a leaf is dot-separated names of letters, digits and `_`"
            ));
        }
        Ok(Self {
            component: component.to_string(),
            leaf: leaf.to_string(),
            value: parse_condition_value(value.trim())?,
            line,
        })
    }

    pub fn render(&self) -> String {
        format!(
            "{}: {} = {}",
            self.component,
            self.leaf,
            self.value.render()
        )
    }

    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("component", Json::str(&self.component)),
            ("leaf", Json::str(&self.leaf)),
            ("value", self.value.to_json()),
        ])
    }
}

pub fn is_leaf_path(leaf: &str) -> bool {
    !leaf.is_empty()
        && leaf.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
}

fn parse_condition_value(value: &str) -> Result<ConditionValue, String> {
    match value {
        "true" => return Ok(ConditionValue::Boolean(true)),
        "false" => return Ok(ConditionValue::Boolean(false)),
        _ => {}
    }
    if let Some(inner) = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        if inner.contains('"') {
            return Err("a string value contains an unescaped `\"`".into());
        }
        return Ok(ConditionValue::Text(inner.to_string()));
    }
    value
        .parse::<i64>()
        .map(ConditionValue::Integer)
        .map_err(|_| format!("invalid value `{value}`"))
}

fn conditions_json(conditions: &[Condition]) -> Json {
    Json::Arr(conditions.iter().map(Condition::to_json).collect())
}

/// A runtime unit that serves one execution-context document. Distinct from an area, which is a
/// code ownership namespace.
#[derive(Debug, Clone)]
pub struct Component {
    pub id: String,
    pub repository: String,
    pub areas: Vec<String>,
    pub shared_areas: Vec<String>,
    /// `None` when the component declares no execution context.
    pub leaves: Option<BTreeMap<String, LeafType>>,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub id: String,
    pub conditions: Vec<Condition>,
    /// Authoritative free-form Markdown. Core preserves and fingerprints it without interpreting
    /// its natural-language or diagram semantics.
    pub statement: String,
    pub line: usize,
}

/// What a requirement ranges over. The behavioural domain is implicit and never written; a second
/// domain arrived only when the demo produced evidence that the first could not carry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    /// Executions of a behaviour described by one or more authored Scenarios.
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
pub struct Requirement {
    pub id: String,
    pub title: Option<String>,
    /// Declared by `Realization:`. Absent means no realization review is owed.
    pub realization: Option<Rigor>,
    /// Declared by `Verification:`. Absent means no Check or verification review is owed.
    pub verification: Option<Rigor>,
    pub conditions: Vec<Condition>,
    pub statement: String,
    pub scenarios: Vec<Scenario>,
    pub line: usize,
    pub domain: Domain,
    /// For `Domain::Sites`: the declared surface whose derived members form the domain.
    pub over: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Term {
    pub id: String,
    /// Authoritative vocabulary definition. Every definition in the module participates in
    /// Requirement and Scenario semantic digests because prose use cannot be resolved reliably by core.
    pub definition: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub id: String,
    pub path: String,
    pub requirements: Vec<Requirement>,
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
    pub requirement: String,
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

/// A compiler-resolved site that implements a realization-owned mechanism identity.
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
/// carry only facts the extractor can derive; semantic requirements remain in the realization prose.
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

#[derive(Debug, Default)]
pub struct Model {
    pub package_producers: Vec<crate::verification_packages::Producer>,
    pub specs: Vec<Spec>,
    pub account_realizations: Vec<crate::account_realization::AccountRealization>,
    pub account_verifications: Vec<crate::account_verification::AccountVerification>,
    pub account_supports: Vec<crate::account_support::AccountSupport>,
    pub review_heads: Vec<crate::review_heads::Head>,
    pub realizes: Vec<Site>,
    pub mechanism_implementations: Vec<MechanismImplementation>,
    pub check_implementations: Vec<CheckImplementation>,
    /// Class members enumerated by an extractor. Empty when no project emits them, in which case
    /// a class is only as wide as its tags.
    pub class_members: Vec<ClassMember>,
    pub enumerations: Vec<Enumeration>,
    pub artifacts: Vec<Artifact>,
    pub realizations: Vec<crate::realization::Realization>,
    pub workspace: crate::workspace::Workspace,
    /// Empty when the project catalog declares no `components`.
    pub components: Vec<Component>,
}

/// A normative Scenario plus its independently governed parent Requirement.
pub struct ScenarioView<'a> {
    pub spec: &'a Spec,
    pub requirement: &'a Requirement,
    pub scenario: &'a Scenario,
}

impl<'a> ScenarioView<'a> {
    pub fn id(&self) -> String {
        self.scenario.id.clone()
    }
}

/// An independently governable product proposition.
pub struct RequirementView<'a> {
    pub spec: &'a Spec,
    pub requirement: &'a Requirement,
}

impl RequirementView<'_> {
    pub fn id(&self) -> String {
        self.requirement.id.clone()
    }
}

impl Model {
    pub fn component(&self, id: &str) -> Option<&Component> {
        self.components.iter().find(|component| component.id == id)
    }

    pub fn component_leaf_type(&self, component: &str, leaf: &str) -> Option<LeafType> {
        self.component(component)?
            .leaves
            .as_ref()?
            .get(leaf)
            .copied()
    }

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
            item: &crate::account_realization::Declaration,
            path: &str,
            identities: &mut BTreeMap<(String, String), String>,
            issues: &mut Vec<crate::diag::Diag>,
        ) {
            if item.kind == crate::account_realization::DeclarationKind::Mechanism {
                record("Mechanism", &item.id, path, item.line, identities, issues);
            }
            for child in &item.children {
                mechanisms(child, path, identities, issues);
            }
        }
        for spec in &self.specs {
            for requirement in &spec.requirements {
                record(
                    "Requirement",
                    &requirement.id,
                    &spec.path,
                    requirement.line,
                    &mut identities,
                    &mut issues,
                );
                for scenario in &requirement.scenarios {
                    record(
                        "Scenario",
                        &scenario.id,
                        &spec.path,
                        scenario.line,
                        &mut identities,
                        &mut issues,
                    );
                }
            }
        }
        for realization in &self.realizations {
            for entry in &realization.entries {
                for mechanism in &entry.mechanisms {
                    record(
                        "Mechanism",
                        &mechanism.id,
                        &realization.path,
                        mechanism.line,
                        &mut identities,
                        &mut issues,
                    );
                }
            }
        }
        for realization in &self.account_realizations {
            for item in &realization.declarations {
                mechanisms(item, &realization.path, &mut identities, &mut issues);
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
        let requirements = self
            .requirements()
            .map(|view| (view.requirement.id.clone(), view.spec.id.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut mechanisms = self
            .realizations
            .iter()
            .flat_map(|realization| {
                realization.entries.iter().flat_map(move |entry| {
                    entry
                        .mechanisms
                        .iter()
                        .map(move |mechanism| (mechanism.id.clone(), realization.spec.clone()))
                })
            })
            .collect::<BTreeMap<_, _>>();
        fn collect(
            item: &crate::account_realization::Declaration,
            module: &str,
            targets: &mut BTreeMap<String, String>,
        ) {
            if item.kind == crate::account_realization::DeclarationKind::Mechanism {
                targets.insert(item.id.clone(), module.to_string());
            }
            for child in &item.children {
                collect(child, module, targets);
            }
        }
        for realization in &self.account_realizations {
            for item in &realization.declarations {
                collect(item, &realization.spec, &mut mechanisms);
            }
        }
        for site in &mut self.realizes {
            site.spec = requirements
                .get(&site.requirement)
                .cloned()
                .unwrap_or_default();
        }
        for implementation in &mut self.mechanism_implementations {
            implementation.spec = mechanisms
                .get(&implementation.mechanism)
                .cloned()
                .unwrap_or_default();
        }
    }

    pub fn account_realization_for_requirement(
        &self,
        spec: &str,
        requirement: &str,
    ) -> Option<(
        &crate::account_realization::AccountRealization,
        &crate::account_realization::Declaration,
    )> {
        let id = requirement.to_string();
        self.account_realizations.iter().find_map(|realization| {
            (realization.spec == spec)
                .then(|| {
                    realization
                        .declarations
                        .iter()
                        .find(|declaration| {
                            declaration.kind
                                == crate::account_realization::DeclarationKind::Requirement
                                && declaration.id == id
                        })
                        .map(|declaration| (realization, declaration))
                })
                .flatten()
        })
    }

    pub fn realization_obligation(
        &self,
        spec: &str,
        requirement: &str,
    ) -> Option<crate::workspace::RealizationObligation> {
        if let Some((_, declaration)) = self.account_realization_for_requirement(spec, requirement)
        {
            return Some(crate::workspace::RealizationObligation {
                spec: spec.to_string(),
                requirement: requirement.to_string(),
                areas: declaration.areas.clone(),
            });
        }
        self.workspace.obligation(spec, requirement).cloned()
    }

    pub fn realization_obligations(&self) -> Vec<crate::workspace::RealizationObligation> {
        let mut obligations = self.workspace.realization_obligations.clone();
        for realization in &self.account_realizations {
            for declaration in &realization.declarations {
                if declaration.kind != crate::account_realization::DeclarationKind::Requirement {
                    continue;
                }
                {
                    obligations.retain(|item| item.requirement != declaration.id);
                    obligations.push(crate::workspace::RealizationObligation {
                        spec: realization.spec.clone(),
                        requirement: declaration.id.clone(),
                        areas: declaration.areas.clone(),
                    });
                }
            }
        }
        obligations
    }

    pub(crate) fn account_realization_digest(
        &self,
        spec: &str,
        requirement: &str,
    ) -> Option<String> {
        let (realization, declaration) =
            self.account_realization_for_requirement(spec, requirement)?;
        fn relationships(item: &crate::account_realization::Declaration) -> Json {
            Json::obj(vec![
                ("kind", Json::str(&format!("{:?}", item.kind))),
                ("id", Json::str(&item.id)),
                (
                    "scenarios",
                    Json::Arr(item.scenarios.iter().map(Json::str).collect()),
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
                    realization
                        .declarations
                        .iter()
                        .filter(|item| {
                            item.kind != crate::account_realization::DeclarationKind::Requirement
                                || item.id == declaration.id
                        })
                        .map(relationships)
                        .collect(),
                ),
            ),
            ("introduction", Json::str(&realization.introduction)),
            (
                "shared",
                Json::Arr(
                    realization
                        .declarations
                        .iter()
                        .filter(|item| {
                            item.kind != crate::account_realization::DeclarationKind::Requirement
                        })
                        .map(|item| Json::str(&item.source))
                        .collect(),
                ),
            ),
            ("requirement", Json::str(&declaration.source)),
        ])))
    }

    pub(crate) fn account_verification_digest(
        &self,
        spec: &str,
        requirement: &str,
    ) -> Option<String> {
        let id = requirement.to_string();
        let document = self
            .account_verifications
            .iter()
            .find(|item| item.owner == spec)?;
        let scope = document
            .requirements
            .iter()
            .find(|item| item.requirement == id)?;
        let sections = document
            .sections
            .iter()
            .filter(|section| {
                section
                    .requirement
                    .as_deref()
                    .is_none_or(|owner| owner == id)
            })
            .map(|section| {
                Json::obj(vec![
                    ("id", Json::str(&section.id)),
                    (
                        "requirement",
                        section.requirement.as_ref().map_or(Json::Null, Json::str),
                    ),
                    ("prose", Json::str(&section.prose)),
                ])
            })
            .collect();
        let elements = document
            .elements
            .iter()
            .filter(|element| {
                element
                    .requirement
                    .as_deref()
                    .is_none_or(|owner| owner == id)
            })
            .map(|element| {
                Json::obj(vec![
                    ("id", Json::str(&element.id)),
                    (
                        "requirement",
                        element.requirement.as_ref().map_or(Json::Null, Json::str),
                    ),
                    ("kind", Json::str(&element.kind)),
                    (
                        "scenario",
                        element.scenario.as_ref().map_or(Json::Null, Json::str),
                    ),
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
                "scenarios",
                Json::Arr(
                    document
                        .scenarios
                        .iter()
                        .filter(|scenario| scenario.requirement == id)
                        .map(|scenario| {
                            Json::obj(vec![
                                ("scenario", Json::str(&scenario.scenario)),
                                ("prose", Json::str(&scenario.prose)),
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
                            check.requirement == id
                                || check.scenarios.iter().any(|scenario| {
                                    self.find_scenario(scenario)
                                        .is_some_and(|view| view.requirement.id == id)
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
                                    "scenarios",
                                    Json::Arr(check.scenarios.iter().map(Json::str).collect()),
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

    pub fn scenarios(&self) -> impl Iterator<Item = ScenarioView<'_>> {
        self.specs.iter().flat_map(|spec| {
            spec.requirements.iter().flat_map(move |requirement| {
                requirement
                    .scenarios
                    .iter()
                    .map(move |scenario| ScenarioView {
                        spec,
                        requirement,
                        scenario,
                    })
            })
        })
    }

    pub fn requirements(&self) -> impl Iterator<Item = RequirementView<'_>> {
        self.specs.iter().flat_map(|spec| {
            spec.requirements
                .iter()
                .map(move |requirement| RequirementView { spec, requirement })
        })
    }

    pub fn has_requirement(&self, requirement: &str) -> bool {
        self.specs.iter().any(|s| {
            s.requirements
                .iter()
                .any(|candidate| candidate.id == requirement)
        })
    }

    pub fn scenario_count(&self) -> usize {
        self.scenarios().count()
    }

    pub fn requirement_count(&self) -> usize {
        self.requirements().count()
    }

    pub fn find_requirement(&self, requirement: &str) -> Option<RequirementView<'_>> {
        self.requirements()
            .find(|candidate| candidate.requirement.id == requirement)
    }

    pub fn find_scenario(&self, id: &str) -> Option<ScenarioView<'_>> {
        self.scenarios().find(|scenario| scenario.id() == id)
    }

    pub fn checks(&self) -> impl Iterator<Item = &crate::verification::Check> {
        self.account_verifications
            .iter()
            .flat_map(|file| file.checks.iter().map(|check| &check.definition))
    }

    pub fn check_binding_count(&self, check: &str, scenario: &str) -> usize {
        self.account_verifications
            .iter()
            .flat_map(|file| &file.checks)
            .filter(|item| item.definition.id == check)
            .flat_map(|item| &item.bindings)
            .filter(|binding| binding.scenario == scenario)
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

    pub fn verification_challenges(
        &self,
    ) -> impl Iterator<Item = &crate::verification::VerificationChallenge> {
        self.account_verifications
            .iter()
            .flat_map(|file| &file.verification_challenges)
    }

    /// Project-wide identity and cardinality checks run only after every authority is loaded.
    pub fn account_check_issues(&self) -> Vec<crate::diag::Diag> {
        crate::account_model::authored_check_issues(
            self.account_verifications.iter(),
            &self.specs,
            &self.account_supports,
            &crate::account_model::mechanism_ids(
                self.account_realizations.iter(),
                &self.realizations,
            ),
        )
    }

    pub fn verification_declaration_issues(&self) -> Vec<crate::diag::Diag> {
        use crate::diag::Diag;
        use std::collections::BTreeMap;

        let mut issues = Vec::new();
        let mut verification_challenge_ids = BTreeMap::new();

        for file in &self.account_verifications {
            for verification_challenge in &file.verification_challenges {
                record_global_id(
                    &mut verification_challenge_ids,
                    "Verification Challenge",
                    &verification_challenge.id,
                    &verification_challenge.path,
                    verification_challenge.line,
                    &mut issues,
                );
                if !self
                    .workspace
                    .packages
                    .iter()
                    .any(|package| package == "azimuth.challenge")
                {
                    issues.push(Diag::at(
                        &verification_challenge.path,
                        verification_challenge.line,
                        "Verification Challenge requires enabled azimuth.challenge package",
                    ));
                }
                for id in &verification_challenge.requirements {
                    if !self.requirements().any(|view| view.requirement.id == *id) {
                        issues.push(Diag::at(
                            &verification_challenge.path,
                            verification_challenge.line,
                            format!("Verification Challenge selects unknown Requirement `{id}`"),
                        ));
                    }
                }
                for id in &verification_challenge.mechanisms {
                    if !crate::account_model::mechanism_ids(
                        self.account_realizations.iter(),
                        &self.realizations,
                    )
                    .contains(id)
                    {
                        issues.push(Diag::at(
                            &verification_challenge.path,
                            verification_challenge.line,
                            format!("Verification Challenge selects unknown Mechanism `{id}`"),
                        ));
                    }
                }
                for id in &verification_challenge.checks {
                    if !self.checks().any(|item| item.id == *id) {
                        issues.push(Diag::at(
                            &verification_challenge.path,
                            verification_challenge.line,
                            format!("Verification Challenge selects unknown Check `{id}`"),
                        ));
                    }
                }
            }
        }
        issues
    }

    /// Semantic Scenario digest for Evidence Binding identity. Rigor declarations and locations are omitted.
    pub fn scenario_digest(&self, scenario_id: &str) -> Option<String> {
        let scenario = self.find_scenario(scenario_id)?;
        let mut fields = vec![
            ("format", Json::str("azimuth-scenario-digest")),
            ("version", Json::Num(3.0)),
            ("id", Json::str(scenario_id)),
            ("parent_requirement", Json::str(&scenario.requirement.id)),
            ("terms", vocabulary_json(scenario.spec)),
            ("requirement", Json::str(&scenario.requirement.statement)),
            ("domain", Json::str(scenario.requirement.domain.name())),
            (
                "over",
                scenario
                    .requirement
                    .over
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            ("statement", Json::str(&scenario.scenario.statement)),
        ];
        if !scenario.requirement.conditions.is_empty() {
            fields.push((
                "requirement_conditions",
                conditions_json(&scenario.requirement.conditions),
            ));
        }
        if !scenario.scenario.conditions.is_empty() {
            fields.push(("conditions", conditions_json(&scenario.scenario.conditions)));
        }
        Some(crate::fingerprint::canonical_sha256(&Json::obj(fields)))
    }

    pub fn requirement_digest(&self, requirement_id: &str) -> Option<String> {
        let requirement = self
            .requirements()
            .find(|requirement| requirement.id() == requirement_id)?;
        let mut fields = vec![
            ("format", Json::str("azimuth-requirement-digest")),
            ("version", Json::Num(3.0)),
            ("id", Json::str(requirement_id)),
            ("terms", vocabulary_json(requirement.spec)),
            ("predicate", Json::str(&requirement.requirement.statement)),
            ("domain", Json::str(requirement.requirement.domain.name())),
            (
                "over",
                requirement
                    .requirement
                    .over
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "scenarios",
                Json::Arr(
                    requirement
                        .requirement
                        .scenarios
                        .iter()
                        .map(|scenario| {
                            let id = scenario.id.clone();
                            Json::obj(vec![
                                ("id", Json::str(&id)),
                                (
                                    "semantic_digest",
                                    Json::str(self.scenario_digest(&id).unwrap_or_default()),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
        ];
        if !requirement.requirement.conditions.is_empty() {
            fields.push((
                "conditions",
                conditions_json(&requirement.requirement.conditions),
            ));
        }
        Some(crate::fingerprint::canonical_sha256(&Json::obj(fields)))
    }

    pub fn realization_for(&self, spec: &str) -> Option<&crate::realization::Realization> {
        self.realizations.iter().find(|d| d.spec == spec)
    }

    pub fn mechanism_bindings<'a>(
        &'a self,
        _spec: &str,
        mechanism: &'a crate::realization::Mechanism,
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
                let requirements = spec
                    .requirements
                    .iter()
                    .map(|r| {
                        let scenarios = r
                            .scenarios
                            .iter()
                            .map(|sc| {
                                let mut fields = vec![
                                    ("id", Json::str(&sc.id)),
                                    ("line", Json::Num(sc.line as f64)),
                                    ("statement", Json::str(&sc.statement)),
                                ];
                                if !sc.conditions.is_empty() {
                                    fields.push(("conditions", conditions_json(&sc.conditions)));
                                }
                                Json::obj(fields)
                            })
                            .collect();
                        let mut fields = vec![
                            ("id", Json::str(&r.id)),
                            (
                                "realization",
                                match r.realization {
                                    Some(c) => Json::str(c.name()),
                                    None => Json::Null,
                                },
                            ),
                            (
                                "verification",
                                match r.verification {
                                    Some(c) => Json::str(c.name()),
                                    None => Json::Null,
                                },
                            ),
                            ("statement", Json::str(&r.statement)),
                            ("line", Json::Num(r.line as f64)),
                            ("domain", Json::str(r.domain.name())),
                            ("over", r.over.as_ref().map(Json::str).unwrap_or(Json::Null)),
                            ("scenarios", Json::Arr(scenarios)),
                        ];
                        if let Some(title) = &r.title {
                            fields.push(("title", Json::str(title)));
                        }
                        if !r.conditions.is_empty() {
                            fields.push(("conditions", conditions_json(&r.conditions)));
                        }
                        Json::obj(fields)
                    })
                    .collect();
                Json::obj(vec![
                    ("id", Json::str(&spec.id)),
                    ("path", Json::str(&spec.path)),
                    ("terms", vocabulary_json(spec)),
                    ("requirements", Json::Arr(requirements)),
                ])
            })
            .collect();

        Json::obj(vec![
            ("version", Json::Num(8.0)),
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
                                ("requirement", Json::str(&check.requirement)),
                                (
                                    "scenarios",
                                    Json::Arr(check.scenarios.iter().map(Json::str).collect()),
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
                "verification_challenges",
                Json::Arr(
                    self.verification_challenges()
                        .map(verification_challenge_json)
                        .collect(),
                ),
            ),
            (
                "findings",
                Json::Arr(findings.iter().map(|h| h.to_json()).collect()),
            ),
        ])
    }
}

impl Model {
    fn mechanism_json(&self) -> Vec<Json> {
        let mut out = Vec::new();
        for realization in &self.realizations {
            for entry in &realization.entries {
                for m in &entry.mechanisms {
                    let mut bindings = self.mechanism_bindings(&realization.spec, m);
                    bindings.sort_unstable();
                    out.push(Json::obj(vec![
                        ("spec", Json::str(&realization.spec)),
                        ("target_kind", Json::str("requirement")),
                        ("target", Json::str(entry.target.id())),
                        ("id", Json::str(&m.id)),
                        (
                            "scenarios",
                            Json::Arr(m.scenarios.iter().map(Json::str).collect()),
                        ),
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
        ("requirement".to_string(), Json::str(&s.requirement)),
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
                            ("requirement".into(), Json::str(&item.requirement)),
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

fn verification_challenge_json(item: &crate::verification::VerificationChallenge) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("form", Json::str(&item.form)),
        ("searches_for", Json::str(&item.searches_for)),
        (
            "targets",
            Json::Arr(item.targets.iter().map(Json::str).collect()),
        ),
        (
            "requirements",
            Json::Arr(item.requirements.iter().map(Json::str).collect()),
        ),
        (
            "mechanisms",
            Json::Arr(item.mechanisms.iter().map(Json::str).collect()),
        ),
        (
            "checks",
            Json::Arr(item.checks.iter().map(Json::str).collect()),
        ),
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
            Json::str(crate::fingerprint::verification_challenge_fingerprint(item)),
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
