//! The derived model.
//!
//! `claim = (domain, predicate)` (D13). The steel thread exercises only the behavioural domain,
//! which scenarios take implicitly and never name — so `domain` is not represented yet. When a
//! second domain arrives it becomes a field here, not a second artifact type.

use crate::json::Json;

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

    /// D6.5: the level gates which artifacts are required at all. A `routine` claim needs a spec
    /// entry and nothing else, so it cannot be `uncovered` — no evidence was ever required of it.
    pub fn requires_evidence(self) -> bool {
        self != Criticality::Routine
    }
}

/// Ladders. A stronger form on any axis satisfies a requirement for a weaker one, so the derived
/// `Ord` is the comparison `wrong-form` uses.
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

/// How far evidence reaches (D4.1). A ladder: stronger satisfies weaker.
///
/// `Proof` is deliberately narrower than the formal-methods sense — established by construction
/// over all executions, with no obligation discharged and no semantics checked. A unique index
/// counts because violation is unrepresentable, not because anything was proved (see the glossary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Strength {
    Detection,
    Demonstration,
    Proof,
}

impl Strength {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "proof" => Some(Strength::Proof),
            "demonstration" => Some(Strength::Demonstration),
            "detection" => Some(Strength::Detection),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Strength::Proof => "proof",
            Strength::Demonstration => "demonstration",
            Strength::Detection => "detection",
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

/// What a claim ranges over (D13). The behavioural domain is implicit and never written; a second
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
    /// `None` is the `unclassified` hole (D6.2), not a parse error: a missing *declaration* is a
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
/// a different realization or expire a judgment. Legacy single-repository manifests omit this
/// value and retain their file/site identity until they are emitted through a repository manifest.
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

/// A tag site, from a manifest. `realizes` and `covers` differ only in the form fields, which are
/// absent on code: form is how a test checks, not a property of code.
#[derive(Debug, Clone)]
pub struct Site {
    pub spec: String,
    pub scenario: String,
    pub site: String,
    pub file: String,
    pub lang: String,
    pub source: Option<SourceIdentity>,
    /// Hash of the compiler-resolved source site. An empty value means the extractor could not
    /// isolate the site, so judgment freshness falls back to the whole file.
    pub source_fingerprint: String,
    /// Present for evidence imported from a system of record rather than extracted from code.
    pub evidence_kind: Option<String>,
    pub evidence_outcome: Option<String>,
    pub observed_at: Option<String>,
    pub expires_at: Option<u64>,
    pub scope: Option<Scope>,
    pub quantification: Option<Quantification>,
    pub oracle: Option<Oracle>,
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
    pub binding: String,
    pub file: String,
    pub lang: String,
    pub source: Option<SourceIdentity>,
    pub source_fingerprint: String,
}

/// Evidence about a mechanism's own contract. It is deliberately not claim evidence: whether a
/// mechanism establishes a particular business claim is a separate composition judgment.
#[derive(Debug, Clone)]
pub struct MechanismCover {
    pub spec: String,
    pub mechanism: String,
    pub site: String,
    pub file: String,
    pub lang: String,
    pub source: Option<SourceIdentity>,
    pub source_fingerprint: String,
    pub scope: Option<Scope>,
    pub quantification: Option<Quantification>,
    pub oracle: Option<Oracle>,
}

/// A member of a class, enumerated by the project's extractor from what the build produced —
/// a route table, a container, a manifest — rather than from a tag.
///
/// This exists because deriving membership from tags cannot see a site nobody tagged, which is the
/// failure D13.1 names: an enumerator that misses a member reports green over the gap. Identity is
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationRole {
    Evidence,
    Challenge,
}

impl ObservationRole {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "evidence" => Some(Self::Evidence),
            "challenge" => Some(Self::Challenge),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Challenge => "challenge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationSubjectRelation {
    Realization,
    Evidence,
    Mechanism,
}

impl ObservationSubjectRelation {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "realization" => Some(Self::Realization),
            "evidence" => Some(Self::Evidence),
            "mechanism" => Some(Self::Mechanism),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Realization => "realization",
            Self::Evidence => "evidence",
            Self::Mechanism => "mechanism",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObservationSubject {
    pub relation: ObservationSubjectRelation,
    pub identity: String,
}

#[derive(Debug, Clone)]
pub struct ObservationBinding {
    pub role: ObservationRole,
    pub spec: String,
    pub scenario: String,
    pub assertion: String,
    pub outcome: String,
    pub subjects: Vec<ObservationSubject>,
    pub scope: Option<Scope>,
    pub quantification: Option<Quantification>,
    pub oracle: Option<Oracle>,
}

/// One immutable execution may answer several claim-specific questions. Tool-specific meaning
/// remains in the fingerprinted report and opaque payload; the core knows only whether each
/// binding contributes evidence or challenges an existing assurance account.
#[derive(Debug, Clone)]
pub struct Observation {
    pub id: String,
    pub kind: String,
    pub tool: String,
    pub tool_version: String,
    pub report: String,
    pub inputs: Vec<String>,
    pub observed_at: Option<String>,
    pub expires_at: Option<u64>,
    pub source_fingerprint: String,
    pub source: Option<SourceIdentity>,
    pub bindings: Vec<ObservationBinding>,
    pub payload: crate::json::Json,
}

impl Observation {
    pub fn evidence_sites(&self) -> impl Iterator<Item = Site> + '_ {
        self.bindings
            .iter()
            .enumerate()
            .filter(|(_, binding)| binding.role == ObservationRole::Evidence)
            .map(|(index, binding)| Site {
                spec: binding.spec.clone(),
                scenario: binding.scenario.clone(),
                site: format!("{}:{}", self.id, index + 1),
                file: self.report.clone(),
                lang: self.tool.clone(),
                source: self.source.clone(),
                source_fingerprint: self.source_fingerprint.clone(),
                evidence_kind: Some(self.kind.clone()),
                evidence_outcome: Some(if binding.outcome == "satisfied" {
                    "passed".into()
                } else {
                    "failed".into()
                }),
                observed_at: self.observed_at.clone(),
                expires_at: self.expires_at,
                scope: binding.scope,
                quantification: binding.quantification,
                oracle: binding.oracle,
            })
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub specs: Vec<Spec>,
    pub realizes: Vec<Site>,
    pub covers: Vec<Site>,
    pub mechanism_implementations: Vec<MechanismImplementation>,
    pub mechanism_covers: Vec<MechanismCover>,
    /// Class members enumerated by an extractor. Empty when no project emits them, in which case
    /// a class is only as wide as its tags.
    pub class_members: Vec<ClassMember>,
    pub enumerations: Vec<Enumeration>,
    pub artifacts: Vec<Artifact>,
    pub observations: Vec<Observation>,
    /// Absent until a standards file is read. Without it no evidence standard is known, so
    /// `wrong-form` cannot fire and `uncovered` falls back to "has any evidence at all".
    pub standards: Option<crate::plan::Standards>,
    pub plans: Vec<crate::plan::Plan>,
    pub designs: Vec<crate::design::Design>,
    pub judgments: Vec<crate::judgment::Judgments>,
    pub workspace: crate::workspace::Workspace,
}

/// The evidence standard for one claim: the project mapping, overridden by a plan entry.
#[derive(Debug, Clone, Copy)]
pub struct Required {
    /// `None` means no evidence is required — D6.5's `routine`.
    pub strength: Option<Strength>,
    pub quantification: Option<Quantification>,
    pub scope: Scope,
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

    pub fn judgments_for(&self, spec: &str) -> Option<&crate::judgment::Judgments> {
        self.judgments.iter().find(|j| j.spec == spec)
    }

    /// Everything a judgment about this claim would have had to look at.
    pub fn evidence_files(&self, spec: &str, scenario: &str) -> Vec<String> {
        self.covers
            .iter()
            .filter(|s| s.spec == spec && s.scenario == scenario)
            .map(|s| s.file.clone())
            .collect()
    }

    /// Every source the agent-tier rubric requires for a verdict. Compiler-resolved evidence and
    /// realization sites carry their own fingerprints; prose and explicitly bound non-code
    /// mechanisms remain deliberately file-scoped.
    pub fn judgment_inputs(
        &self,
        spec: &str,
        scenario: &str,
    ) -> Vec<crate::judgment::FingerprintInput> {
        let mut inputs: Vec<crate::judgment::FingerprintInput> = self
            .realizes
            .iter()
            .filter(|site| site.spec == spec && site.scenario == scenario)
            .map(crate::judgment::FingerprintInput::realization)
            .collect();
        inputs.extend(
            self.covers
                .iter()
                .filter(|site| site.spec == spec && site.scenario == scenario)
                .map(crate::judgment::FingerprintInput::evidence),
        );
        for observation in &self.observations {
            for binding in observation
                .bindings
                .iter()
                .filter(|binding| binding.spec == spec && binding.scenario == scenario)
            {
                inputs.push(match binding.role {
                    ObservationRole::Evidence => {
                        crate::judgment::FingerprintInput::observed_evidence(observation, binding)
                    }
                    ObservationRole::Challenge => {
                        crate::judgment::FingerprintInput::challenge(observation, binding)
                    }
                });
                inputs.extend(
                    observation
                        .inputs
                        .iter()
                        .map(|path| crate::judgment::FingerprintInput::file(path)),
                );
            }
        }
        let Some(claim) = self.find_claim(spec, scenario) else {
            return inputs;
        };
        if let Some(standards) = &self.standards {
            inputs.push(crate::judgment::FingerprintInput::model_artifact(
                "verification",
                "standards",
                &standards.path,
            ));
        }
        if let Some(plan) = self.plan_for(spec) {
            inputs.push(crate::judgment::FingerprintInput::model_artifact(
                "verification",
                spec,
                &plan.path,
            ));
            if let Some(evidence) = plan
                .entry(scenario)
                .and_then(|entry| entry.evidence.as_ref())
            {
                for binding in evidence.bindings.iter().chain(&evidence.detector_bindings) {
                    if let Some(artifact) = self.artifacts.iter().find(|item| item.id == *binding) {
                        if !artifact.file.is_empty() {
                            inputs.push(crate::judgment::FingerprintInput::file(&artifact.file));
                        }
                    }
                }
            }
        }
        if let Some(design) = self.design_for(spec) {
            let entries = design
                .for_scenario(scenario)
                .into_iter()
                .chain(design.for_requirement(&claim.requirement.id));
            let mut has_entry = false;
            for entry in entries {
                has_entry = true;
                for mechanism in &entry.mechanisms {
                    if let Some(binding) = mechanism.binding.as_deref() {
                        if let Some(artifact) = self
                            .artifacts
                            .iter()
                            .find(|artifact| artifact.id == binding)
                        {
                            if !artifact.file.is_empty() {
                                inputs
                                    .push(crate::judgment::FingerprintInput::file(&artifact.file));
                            }
                        }
                    }
                    inputs.extend(
                        self.mechanism_implementations
                            .iter()
                            .filter(|implementation| {
                                implementation.spec == spec
                                    && implementation.mechanism == mechanism.id
                            })
                            .map(crate::judgment::FingerprintInput::mechanism),
                    );
                }
            }
            if has_entry {
                inputs.push(crate::judgment::FingerprintInput::model_artifact(
                    "design",
                    spec,
                    &design.path,
                ));
            }
        }
        if let Some(obligation) = self.workspace.obligation(spec, scenario) {
            let mut area_declarations = Vec::new();
            for area_id in &obligation.areas {
                if let Some(area) = self.workspace.areas.iter().find(|area| &area.id == area_id) {
                    let mut mounts = Vec::new();
                    for mount in &area.mounts {
                        mounts.push(format!("{}={}", mount.id, mount.path));
                    }
                    mounts.sort();
                    area_declarations.push(format!("{}|{}", area.id, mounts.join("|")));
                }
            }
            area_declarations.sort();
            inputs.push(crate::judgment::FingerprintInput::declaration(
                "realization-obligation",
                &format!("{spec}#{scenario}"),
                &area_declarations.join(","),
            ));
        }
        if claim.requirement.domain == Domain::Sites {
            if let Some(surface_id) = claim.requirement.over.as_deref() {
                if let Some(surface) = self.workspace.surface(surface_id) {
                    let mut contributions = surface
                        .contributions
                        .iter()
                        .map(|contribution| {
                            let path = self
                                .workspace
                                .areas
                                .iter()
                                .find(|area| area.id == contribution.area)
                                .and_then(|area| {
                                    area.mounts
                                        .iter()
                                        .find(|mount| mount.id == contribution.mount)
                                })
                                .map(|mount| mount.path.as_str())
                                .unwrap_or_default();
                            format!(
                                "{}|{}|{}|{}",
                                contribution.area,
                                contribution.mount,
                                contribution.enumerator,
                                path
                            )
                        })
                        .collect::<Vec<_>>();
                    contributions.sort();
                    inputs.push(crate::judgment::FingerprintInput::declaration(
                        "surface",
                        surface_id,
                        &contributions.join(","),
                    ));
                    for enumeration in self
                        .enumerations
                        .iter()
                        .filter(|enumeration| enumeration.class == surface_id)
                    {
                        inputs.push(crate::judgment::FingerprintInput::declaration(
                            "surface-enumeration",
                            &format!(
                                "{}|{}|{}",
                                surface_id,
                                enumeration.kind,
                                enumeration
                                    .identity
                                    .as_ref()
                                    .map(|identity| identity.key())
                                    .unwrap_or_else(|| enumeration.source.clone())
                            ),
                            &enumeration.source_fingerprint,
                        ));
                    }
                }
            }
        }
        inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        inputs.dedup_by(|a, b| a.identity == b.identity);
        inputs
    }

    pub fn claim_text(&self, claim: &ClaimView<'_>) -> String {
        let steps: Vec<String> = claim
            .scenario
            .steps
            .iter()
            .map(|s| format!("{} {}", s.kind.name(), s.text))
            .collect();
        format!(
            "{}|{}|{}|{}",
            claim
                .requirement
                .criticality
                .map(|criticality| criticality.name())
                .unwrap_or("unclassified"),
            claim.requirement.statement,
            claim.scenario.id,
            steps.join("|")
        )
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

    pub fn plan_for(&self, spec: &str) -> Option<&crate::plan::Plan> {
        self.plans.iter().find(|p| p.spec == spec)
    }

    /// Resolves the standard for a claim. Returns `None` when no standards file was read, or when
    /// the requirement declares no criticality — in both cases nothing is known to require.
    pub fn required_for(&self, claim: &ClaimView<'_>) -> Option<Required> {
        let standards = self.standards.as_ref()?;
        let level = standards.for_level(claim.requirement.criticality?)?;
        let entry = self
            .plan_for(&claim.spec.id)
            .and_then(|p| p.entry(&claim.scenario.id));
        Some(Required {
            strength: level.strength,
            quantification: entry
                .and_then(|e| e.quantification)
                .or(level.quantification),
            // D15: scope is not derived from criticality. Default, raised per claim where truth
            // depends on something real.
            scope: entry
                .and_then(|e| e.scope)
                .unwrap_or(standards.default_scope),
        })
    }

    /// D10: the export is the extension seam. Checks, dashboards and PR annotations are all
    /// consumers of this; nothing else re-parses specs.
    pub fn to_json(&self, holes: &[crate::check::Hole]) -> Json {
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
            ("version", Json::Num(1.0)),
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
            (
                "covers",
                Json::Arr(
                    self.covers
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
                "mechanism_covers",
                Json::Arr(
                    self.mechanism_covers
                        .iter()
                        .map(mechanism_cover_json)
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
            (
                "observations",
                Json::Arr(self.observations.iter().map(observation_json).collect()),
            ),
            ("mechanisms", Json::Arr(self.mechanism_json())),
            (
                "holes",
                Json::Arr(holes.iter().map(|h| h.to_json()).collect()),
            ),
        ])
    }
}

fn observation_json(item: &Observation) -> Json {
    let mut fields = vec![
        ("id".to_string(), Json::str(&item.id)),
        ("kind".to_string(), Json::str(&item.kind)),
        ("tool".to_string(), Json::str(&item.tool)),
        ("tool_version".to_string(), Json::str(&item.tool_version)),
        ("report".to_string(), Json::str(&item.report)),
        (
            "inputs".to_string(),
            Json::Arr(item.inputs.iter().map(Json::str).collect()),
        ),
        (
            "bindings".to_string(),
            Json::Arr(item.bindings.iter().map(observation_binding_json).collect()),
        ),
        (
            "source_fingerprint".to_string(),
            Json::str(&item.source_fingerprint),
        ),
        ("payload".to_string(), item.payload.clone()),
    ];
    if let Some(observed_at) = &item.observed_at {
        fields.push(("observed_at".to_string(), Json::str(observed_at)));
    }
    if let Some(expires_at) = item.expires_at {
        fields.push(("expires_at".to_string(), Json::Num(expires_at as f64)));
    }
    append_source(&mut fields, item.source.as_ref());
    Json::Obj(fields)
}

fn observation_binding_json(binding: &ObservationBinding) -> Json {
    let mut fields = vec![
        ("role".to_string(), Json::str(binding.role.name())),
        ("spec".to_string(), Json::str(&binding.spec)),
        ("scenario".to_string(), Json::str(&binding.scenario)),
        ("assertion".to_string(), Json::str(&binding.assertion)),
        ("outcome".to_string(), Json::str(&binding.outcome)),
        (
            "subjects".to_string(),
            Json::Arr(
                binding
                    .subjects
                    .iter()
                    .map(|subject| {
                        Json::obj(vec![
                            ("relation", Json::str(subject.relation.name())),
                            ("identity", Json::str(&subject.identity)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ];
    if let Some(scope) = binding.scope {
        fields.push(("scope".to_string(), Json::str(scope.name())));
    }
    if let Some(quantification) = binding.quantification {
        fields.push((
            "quantification".to_string(),
            Json::str(quantification.name()),
        ));
    }
    if let Some(oracle) = binding.oracle {
        fields.push(("oracle".to_string(), Json::str(oracle.name())));
    }
    Json::Obj(fields)
}

impl Model {
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
    if let Some(value) = &s.evidence_kind {
        pairs.push(("evidence_kind".to_string(), Json::str(value)));
    }
    if let Some(value) = &s.evidence_outcome {
        pairs.push(("evidence_outcome".to_string(), Json::str(value)));
    }
    if let Some(value) = &s.observed_at {
        pairs.push(("observed_at".to_string(), Json::str(value)));
    }
    if let Some(value) = s.expires_at {
        pairs.push(("expires_at".to_string(), Json::Num(value as f64)));
    }
    if let Some(scope) = s.scope {
        pairs.push(("scope".to_string(), Json::str(scope.name())));
    }
    if let Some(q) = s.quantification {
        pairs.push(("quantification".to_string(), Json::str(q.name())));
    }
    if let Some(o) = s.oracle {
        pairs.push(("oracle".to_string(), Json::str(o.name())));
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

fn mechanism_cover_json(item: &MechanismCover) -> Json {
    let mut fields = vec![
        ("spec".to_string(), Json::str(&item.spec)),
        ("mechanism".to_string(), Json::str(&item.mechanism)),
        ("site".to_string(), Json::str(&item.site)),
        ("file".to_string(), Json::str(&item.file)),
        ("lang".to_string(), Json::str(&item.lang)),
        (
            "source_fingerprint".to_string(),
            Json::str(&item.source_fingerprint),
        ),
    ];
    if let Some(scope) = item.scope {
        fields.push(("scope".to_string(), Json::str(scope.name())));
    }
    if let Some(quantification) = item.quantification {
        fields.push((
            "quantification".to_string(),
            Json::str(quantification.name()),
        ));
    }
    if let Some(oracle) = item.oracle {
        fields.push(("oracle".to_string(), Json::str(oracle.name())));
    }
    append_source(&mut fields, item.source.as_ref());
    Json::Obj(fields)
}

fn append_source(fields: &mut Vec<(String, Json)>, source: Option<&SourceIdentity>) {
    if let Some(source) = source {
        fields.push(("area".to_string(), Json::str(&source.area)));
        fields.push(("address_kind".to_string(), Json::str(&source.kind)));
        fields.push(("address".to_string(), Json::str(&source.address)));
        fields.push(("mount".to_string(), Json::str(&source.mount)));
    }
}
