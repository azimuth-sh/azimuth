//! The derived model.
//!
//! `claim = (domain, predicate)`. The current corpus exercises only the behavioural domain,
//! which scenarios take implicitly and never name — so `domain` is not represented yet. When a
//! second domain arrives it becomes a field here, not a second artifact type.

use crate::json::Json;
use std::collections::BTreeSet;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    Given,
    When,
    Then,
    And,
}

impl StepKind {
    pub fn name(self) -> &'static str {
        match self {
            StepKind::Given => "given",
            StepKind::When => "when",
            StepKind::Then => "then",
            StepKind::And => "and",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    pub kind: StepKind,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub id: String,
    pub steps: Vec<Step>,
    pub line: usize,
}

/// What a claim ranges over. The behavioural domain is implicit and never written; a second
/// domain arrived only when the demo produced evidence that the first could not carry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    /// Executions of a behaviour — inputs matching the WHEN.
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
    /// `None` is the `unclassified` finding, not a parse error: a missing *declaration* is a
    /// semantic gap, while an unrecognized *construct* fails the parse.
    pub criticality: Option<Criticality>,
    pub statement: String,
    pub scenarios: Vec<Scenario>,
    pub line: usize,
    pub domain: Domain,
    /// For `Domain::Sites`: the declared surface whose derived members form the domain.
    pub over: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub id: String,
    pub path: String,
    pub requirements: Vec<Requirement>,
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
    pub scenario: String,
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
    pub specs: Vec<Spec>,
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

/// A scenario plus the context needed to report on it.
pub struct ClaimView<'a> {
    pub spec: &'a Spec,
    pub requirement: &'a Requirement,
    pub scenario: &'a Scenario,
}

impl<'a> ClaimView<'a> {
    pub fn id(&self) -> String {
        format!("{}#{}", self.spec.id, self.scenario.id)
    }
}

impl Model {
    pub fn claims(&self) -> impl Iterator<Item = ClaimView<'_>> {
        self.specs.iter().flat_map(|spec| {
            spec.requirements.iter().flat_map(move |requirement| {
                requirement.scenarios.iter().map(move |scenario| ClaimView {
                    spec,
                    requirement,
                    scenario,
                })
            })
        })
    }

    pub fn has_claim(&self, spec: &str, scenario: &str) -> bool {
        self.specs.iter().any(|s| {
            s.id == spec
                && s.requirements
                    .iter()
                    .any(|r| r.scenarios.iter().any(|sc| sc.id == scenario))
        })
    }

    pub fn scenario_count(&self) -> usize {
        self.claims().count()
    }

    pub fn find_claim(&self, spec: &str, scenario: &str) -> Option<ClaimView<'_>> {
        self.claims()
            .find(|c| c.spec.id == spec && c.scenario.id == scenario)
    }

    pub fn checks(&self) -> impl Iterator<Item = &crate::verification::Check> {
        self.verifications.iter().flat_map(|file| &file.checks)
    }

    pub fn evidence_bindings(&self) -> impl Iterator<Item = &crate::verification::EvidenceBinding> {
        self.verifications.iter().flat_map(|file| &file.bindings)
    }

    pub fn qualifications(&self) -> impl Iterator<Item = &crate::verification::Qualification> {
        self.verifications
            .iter()
            .flat_map(|file| &file.qualifications)
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
    pub fn verification_declaration_issues(&self) -> Vec<crate::diag::Diag> {
        use crate::diag::Diag;
        use std::collections::{BTreeMap, BTreeSet};

        let mut issues = Vec::new();
        let mut check_ids = BTreeMap::new();
        let mut binding_ids = BTreeMap::new();
        let mut qualification_ids = BTreeMap::new();
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
                if !binding_pairs.insert((binding.check.clone(), binding.claim.clone())) {
                    issues.push(Diag::at(
                        &binding.path,
                        binding.line,
                        format!(
                            "Check `{}` is already bound to Claim `{}`",
                            binding.check, binding.claim
                        ),
                    ));
                }
            }
            for qualification in &file.qualifications {
                record_global_id(
                    &mut qualification_ids,
                    "Qualification",
                    &qualification.id,
                    &qualification.path,
                    qualification.line,
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
                        format!(
                            "Claim Judgment `{}` names no current case Claim",
                            judgment.id
                        ),
                    )),
                    Some(claim) if claim.requirement.criticality == Some(Criticality::Routine) => {
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

    /// Semantic Claim digest for Evidence Binding identity. Criticality and locations are omitted.
    pub fn claim_digest(&self, claim_id: &str) -> Option<String> {
        let claim = self.claims().find(|claim| claim.id() == claim_id)?;
        Some(crate::fingerprint::canonical_sha256(&Json::obj(vec![
            ("format", Json::str("azimuth-case-claim-digest")),
            ("version", Json::Num(1.0)),
            ("id", Json::str(claim_id)),
            ("requirement", Json::str(&claim.requirement.statement)),
            ("domain", Json::str(claim.requirement.domain.name())),
            (
                "over",
                claim
                    .requirement
                    .over
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "steps",
                Json::Arr(
                    claim
                        .scenario
                        .steps
                        .iter()
                        .map(|step| {
                            Json::obj(vec![
                                ("kind", Json::str(step.kind.name())),
                                ("text", Json::str(&step.text)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])))
    }

    pub fn expected_qualification_fingerprint(
        &self,
        binding: &crate::verification::EvidenceBinding,
    ) -> Option<String> {
        let check = self.checks().find(|check| check.id == binding.check)?;
        let policy = self
            .decision_standards
            .as_ref()?
            .policies
            .iter()
            .find(|policy| policy.id == binding.policy)?;
        let check_fingerprint =
            crate::fingerprint::check_fingerprint(check, &self.check_implementations);
        let binding_fingerprint = crate::fingerprint::binding_fingerprint(
            binding,
            &self.claim_digest(&binding.claim)?,
            &crate::fingerprint::policy_fingerprint(policy),
        );
        Some(crate::fingerprint::qualification_fingerprint(
            &check_fingerprint,
            &binding_fingerprint,
            &crate::fingerprint::context_fingerprint(binding),
        ))
    }

    /// Builds the exact total-composition preimage for one authored Claim Judgment.
    /// `None` means at least one required semantic dependency is absent or ambiguous.
    pub fn claim_judgment_preimage(
        &self,
        judgment: &crate::verification::ClaimJudgment,
    ) -> Option<Json> {
        let claim = self.claims().find(|claim| claim.id() == judgment.id)?;
        let criticality = claim.requirement.criticality?;
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
            .workspace
            .obligation(&claim.spec.id, &claim.scenario.id)
            .map(|obligation| obligation.areas.clone())
            .unwrap_or_default();
        obligation_areas.sort();
        if has_duplicates(&obligation_areas) {
            return None;
        }

        let surface = match &claim.requirement.over {
            Some(id) => self.surface_account(id),
            None => Some(Json::Null),
        }?;

        let mut realization_sites = self
            .realizes
            .iter()
            .filter(|site| site.spec == claim.spec.id && site.scenario == claim.scenario.id)
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

        let mut bindings = self
            .evidence_bindings()
            .filter(|binding| binding.claim == judgment.id)
            .collect::<Vec<_>>();
        bindings.sort_by(|left, right| left.id.cmp(&right.id));
        if bindings.is_empty() || has_duplicates_by(&bindings, |binding| binding.id.clone()) {
            return None;
        }
        let mut binding_records = Vec::new();
        let mut qualification_records = Vec::new();
        for binding in bindings {
            let binding_policy = self
                .decision_standards
                .as_ref()?
                .policies
                .iter()
                .find(|policy| policy.id == binding.policy)?;
            let binding_fingerprint = crate::fingerprint::binding_fingerprint(
                binding,
                &self.claim_digest(&binding.claim)?,
                &crate::fingerprint::policy_fingerprint(binding_policy),
            );
            binding_records.push(Json::obj(vec![
                ("id", Json::str(&binding.id)),
                ("fingerprint", Json::str(binding_fingerprint)),
            ]));
            let qualifications = self
                .qualifications()
                .filter(|qualification| qualification.id == binding.id)
                .collect::<Vec<_>>();
            let [qualification] = qualifications.as_slice() else {
                return None;
            };
            qualification_records.push(Json::obj(vec![
                ("id", Json::str(&qualification.id)),
                (
                    "expected_fingerprint",
                    Json::str(self.expected_qualification_fingerprint(binding)?),
                ),
                ("verdict", Json::str(qualification.verdict.name())),
            ]));
        }

        Some(Json::obj(vec![
            ("format", Json::str("azimuth-claim-judgment-fingerprint")),
            ("version", Json::Num(1.0)),
            (
                "claim",
                Json::obj(vec![
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
                ]),
            ),
            ("realizations", Json::Arr(realizations)),
            ("mechanisms", Json::Arr(mechanisms)),
            ("bindings", Json::Arr(binding_records)),
            ("qualifications", Json::Arr(qualification_records)),
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
            RelationKind::Check => {
                anchors.push(self.check_scope_component(&candidate.selector.id)?);
            }
            RelationKind::Claim => {
                anchors.push(self.claim_scope_component(&candidate.selector.id)?);
            }
            RelationKind::Mechanism => {
                let (mechanism, artifact, implementation) =
                    self.mechanism_scope_components(&candidate.selector.id)?;
                anchors.push(mechanism);
                inputs.push(artifact);
                inputs.extend(implementation);
            }
            RelationKind::Realization => {
                anchors.push(self.realization_scope_component(&candidate.selector.id)?);
            }
        }

        let target = candidate.target.as_ref()?;
        match target.kind {
            DecisionKind::Qualification => {
                let binding = self
                    .evidence_bindings()
                    .find(|binding| binding.id == target.id)?;
                let expected = self.expected_qualification_fingerprint(binding)?;
                if target.expected_fingerprint.as_deref() != Some(expected.as_str())
                    || target.authored_fingerprint.as_deref() != Some(expected.as_str())
                {
                    return None;
                }
                inputs.extend(self.qualification_scope_components(binding)?);
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
        Some(scope_component(
            crate::verification::SemanticScopeKind::Binding,
            &binding.id,
            crate::fingerprint::binding_fingerprint(
                binding,
                &self.claim_digest(&binding.claim)?,
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
            crate::fingerprint::check_fingerprint(check, &self.check_implementations),
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

    fn qualification_scope_components(
        &self,
        binding: &crate::verification::EvidenceBinding,
    ) -> Option<Vec<SemanticScopeComponent>> {
        use crate::verification::SemanticScopeKind;

        let qualifications = self
            .qualifications()
            .filter(|qualification| qualification.id == binding.id)
            .collect::<Vec<_>>();
        let [_qualification] = qualifications.as_slice() else {
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
                SemanticScopeKind::Qualification,
                &binding.id,
                self.expected_qualification_fingerprint(binding)?,
            ),
            self.binding_scope_component(binding)?,
            self.claim_scope_component(&binding.claim)?,
            self.check_scope_component(&binding.check)?,
            scope_component(
                SemanticScopeKind::Context,
                &binding.id,
                crate::fingerprint::context_fingerprint(binding),
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
            .filter(|implementation| implementation.check == binding.check)
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
            .filter(|site| site.spec == claim.spec.id && site.scenario == claim.scenario.id)
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
            for entry in design.entries.iter().filter(|entry| match &entry.target {
                crate::design::Target::Requirement(target) => target == &claim.requirement.id,
                crate::design::Target::Scenario(target) => target == &claim.scenario.id,
            }) {
                for mechanism in &entry.mechanisms {
                    let identity = format!("{}#{}", claim.spec.id, mechanism.id);
                    let (mechanism, artifact, implementation) =
                        self.mechanism_scope_components(&identity)?;
                    components.extend([mechanism, artifact]);
                    components.extend(implementation);
                }
            }
        }
        for binding in self
            .evidence_bindings()
            .filter(|binding| binding.claim == id)
        {
            components.extend(self.qualification_scope_components(binding)?);
        }
        if let Some(surface_id) = &claim.requirement.over {
            components.extend(self.surface_scope_components(surface_id)?);
        }
        if let Some(obligation) = self
            .workspace
            .obligation(&claim.spec.id, &claim.scenario.id)
        {
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
    ) -> Option<(
        SemanticScopeComponent,
        SemanticScopeComponent,
        Option<SemanticScopeComponent>,
    )> {
        use crate::verification::SemanticScopeKind;

        let (spec_id, mechanism_id) = identity.split_once('#')?;
        let design = self.design_for(spec_id)?;
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
        let artifact_account = record.get("artifact")?.clone();
        let artifact_id = artifact_account.get("id")?.as_str()?;
        let artifacts = self
            .artifacts
            .iter()
            .filter(|artifact| artifact.id == artifact_id)
            .collect::<Vec<_>>();
        let [artifact] = artifacts.as_slice() else {
            return None;
        };
        let artifact_identity = artifact.source.as_ref()?.key();
        let artifact_component = SemanticScopeComponent {
            kind: SemanticScopeKind::Artifact,
            id: artifact.id.clone(),
            fingerprint: crate::fingerprint::artifact_property_digest(&artifact_account),
            locator: Some(SemanticScopeLocator::Artifact {
                file: artifact.file.clone(),
                artifact_kind: artifact.kind.clone(),
                identity: artifact_identity,
                unique: artifact.unique,
                columns: artifact.columns.clone(),
                predicate: artifact.predicate.clone(),
            }),
        };
        let implementation = if mechanism.binding.is_none() {
            let implementations = self
                .mechanism_implementations
                .iter()
                .filter(|implementation| {
                    implementation.spec == spec_id && implementation.mechanism == mechanism_id
                })
                .collect::<Vec<_>>();
            let [implementation] = implementations.as_slice() else {
                return None;
            };
            let source = implementation.source.as_ref()?;
            Some(source_scope_component(
                SemanticScopeKind::MechanismImplementation,
                &source.key(),
                &implementation.source_fingerprint,
                &implementation.file,
                &implementation.lang,
                &implementation.site,
            )?)
        } else {
            None
        };
        Some((
            scope_component(
                SemanticScopeKind::Mechanism,
                identity,
                crate::fingerprint::mechanism_record_digest(&record),
            ),
            artifact_component,
            implementation,
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
            .flat_map(|spec| &spec.requirements)
            .filter(|requirement| requirement.domain == Domain::Behaviour)
            .flat_map(|requirement| requirement.scenarios.iter().map(|scenario| &scenario.id))
            .collect::<BTreeSet<_>>();
        for site in self
            .realizes
            .iter()
            .filter(|site| site.spec == id && behavioural.contains(&site.scenario))
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
            .filter(|entry| match &entry.target {
                crate::design::Target::Requirement(id) => id == &claim.requirement.id,
                crate::design::Target::Scenario(id) => id == &claim.scenario.id,
            })
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
        design: &crate::design::Design,
        entry: &crate::design::DesignEntry,
        mechanism: &crate::design::Mechanism,
    ) -> Option<Json> {
        let implementations = self
            .mechanism_implementations
            .iter()
            .filter(|implementation| {
                implementation.spec == design.spec && implementation.mechanism == mechanism.id
            })
            .collect::<Vec<_>>();
        let (artifact_id, implementation) = match &mechanism.binding {
            Some(binding) if implementations.is_empty() => (binding.as_str(), Json::Null),
            Some(_) => return None,
            None => {
                let [implementation] = implementations.as_slice() else {
                    return None;
                };
                let source = implementation.source.as_ref()?;
                if implementation.source_fingerprint.is_empty() {
                    return None;
                }
                (
                    implementation.binding.as_str(),
                    Json::obj(vec![
                        ("identity", Json::str(source.key())),
                        (
                            "source_fingerprint",
                            Json::str(&implementation.source_fingerprint),
                        ),
                        ("artifact", Json::str(&implementation.binding)),
                    ]),
                )
            }
        };
        let artifacts = self
            .artifacts
            .iter()
            .filter(|artifact| artifact.id == artifact_id)
            .collect::<Vec<_>>();
        let [artifact] = artifacts.as_slice() else {
            return None;
        };
        let artifact_identity = artifact.source.as_ref()?.key();
        let attachment_kind = match entry.target {
            crate::design::Target::Requirement(_) => "requirement",
            crate::design::Target::Scenario(_) => "scenario",
        };
        Some(Json::obj(vec![
            ("id", Json::str(format!("{}#{}", design.spec, mechanism.id))),
            (
                "attachment",
                Json::obj(vec![
                    ("target_kind", Json::str(attachment_kind)),
                    ("target_id", Json::str(entry.target.id())),
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
            (
                "artifact",
                Json::obj(vec![
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
                ]),
            ),
            ("implementation", implementation),
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
            .flat_map(|spec| &spec.requirements)
            .filter(|requirement| requirement.domain == Domain::Behaviour)
            .flat_map(|requirement| requirement.scenarios.iter().map(|scenario| &scenario.id))
            .collect::<BTreeSet<_>>();
        let mut members = Vec::new();
        let mut member_keys = BTreeSet::new();
        for site in self
            .realizes
            .iter()
            .filter(|site| site.spec == surface.id && behavioural.contains(&site.scenario))
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
        spec: &str,
        mechanism: &'a crate::design::Mechanism,
    ) -> Vec<&'a str> {
        let mut bindings = Vec::new();
        if let Some(binding) = mechanism.binding.as_deref() {
            bindings.push(binding);
        }
        bindings.extend(
            self.mechanism_implementations
                .iter()
                .filter(|implementation| {
                    implementation.spec == spec && implementation.mechanism == mechanism.id
                })
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
                let reqs = spec
                    .requirements
                    .iter()
                    .map(|r| {
                        let scenarios = r
                            .scenarios
                            .iter()
                            .map(|sc| {
                                let steps = sc
                                    .steps
                                    .iter()
                                    .map(|st| {
                                        Json::obj(vec![
                                            ("kind", Json::str(st.kind.name())),
                                            ("text", Json::str(&st.text)),
                                        ])
                                    })
                                    .collect();
                                Json::obj(vec![
                                    ("id", Json::str(&sc.id)),
                                    ("line", Json::Num(sc.line as f64)),
                                    ("steps", Json::Arr(steps)),
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
                            ("scenarios", Json::Arr(scenarios)),
                        ])
                    })
                    .collect();
                Json::obj(vec![
                    ("id", Json::str(&spec.id)),
                    ("path", Json::str(&spec.path)),
                    ("requirements", Json::Arr(reqs)),
                ])
            })
            .collect();

        Json::obj(vec![
            ("version", Json::Num(2.0)),
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
            ("workspace", workspace_json(&self.workspace)),
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
                "qualifications",
                Json::Arr(self.qualifications().map(qualification_json).collect()),
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
                    let bindings = self.mechanism_bindings(&design.spec, m);
                    out.push(Json::obj(vec![
                        ("spec", Json::str(&design.spec)),
                        (
                            "target_kind",
                            Json::str(match entry.target {
                                crate::design::Target::Requirement(_) => "requirement",
                                crate::design::Target::Scenario(_) => "scenario",
                            }),
                        ),
                        ("target", Json::str(entry.target.id())),
                        ("id", Json::str(&m.id)),
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
        ("spec".to_string(), Json::str(&s.spec)),
        ("scenario".to_string(), Json::str(&s.scenario)),
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

fn workspace_json(workspace: &crate::workspace::Workspace) -> Json {
    Json::Obj(vec![
        ("path".into(), Json::str(&workspace.path)),
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
                workspace
                    .realization_obligations
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
        ("spec".to_string(), Json::str(&item.spec)),
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
        ("terminal", Json::str(&item.terminal)),
        (
            "fingerprint",
            Json::str(crate::fingerprint::check_fingerprint(
                item,
                &model.check_implementations,
            )),
        ),
    ])
}

fn binding_json(model: &Model, item: &crate::verification::EvidenceBinding) -> Json {
    let mut fields = vec![
        ("id".to_string(), Json::str(&item.id)),
        ("check".to_string(), Json::str(&item.check)),
        ("claim".to_string(), Json::str(&item.claim)),
        ("proposition".to_string(), Json::str(&item.proposition)),
        ("scope".to_string(), Json::str(item.scope.name())),
        (
            "quantification".to_string(),
            Json::str(item.quantification.name()),
        ),
        ("oracle".to_string(), Json::str(item.oracle.name())),
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
            Json::str(crate::fingerprint::context_fingerprint(item)),
        ),
    ];
    if let Some(expected) = model.expected_qualification_fingerprint(item) {
        fields.push(("qualification_fingerprint".to_string(), Json::str(expected)));
    }
    Json::Obj(fields)
}

fn qualification_json(item: &crate::verification::Qualification) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("verdict", Json::str(item.verdict.name())),
        ("fingerprint", Json::str(&item.fingerprint)),
        ("qualified", Json::str(&item.qualified)),
        ("qualifier", Json::str(&item.qualifier)),
    ])
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
