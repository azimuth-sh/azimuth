//! Pure derived Claim, realization, and verification traceability projection.

use crate::json::Json;
use crate::model::{Criticality, Model, Site, StepKind};
use crate::validation::{resolve_challenge_plan, ChallengeResolution, DecisionKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityReport {
    pub claims: Vec<TraceabilityClaim>,
    pub challenge_resolutions: Vec<ChallengeResolution>,
    pub decision_impacts: DecisionImpactProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityClaim {
    pub id: String,
    pub parent_requirement: String,
    pub criticality: Option<Criticality>,
    pub statement: String,
    pub steps: Vec<TraceabilityStep>,
    pub realizations: Vec<String>,
    pub verification: Vec<TraceabilityVerification>,
    pub judgment: TraceabilityJudgment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityStep {
    pub kind: StepKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceabilityVerification {
    pub check: String,
    pub binding: String,
    pub applicable: bool,
    pub current: bool,
    pub qualification: Option<String>,
    pub verdict: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityJudgment {
    pub applicable: bool,
    pub current: bool,
    pub fingerprint: Option<String>,
    pub verdict: Option<String>,
    pub policy: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactNodeKind {
    Binding,
    Claim,
    ClaimJudgment,
    Qualification,
}

impl ImpactNodeKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::Claim => "claim",
            Self::ClaimJudgment => "claim-judgment",
            Self::Qualification => "qualification",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionImpactNode {
    pub kind: ImpactNodeKind,
    pub id: String,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionImpactEdge {
    pub from: DecisionImpactNode,
    pub to: DecisionImpactNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionImpactProjection {
    pub nodes: Vec<DecisionImpactNode>,
    pub edges: Vec<DecisionImpactEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionReference {
    pub kind: DecisionKind,
    pub id: String,
    pub fingerprint: String,
}

pub fn project(model: &Model) -> TraceabilityReport {
    let mut realizations = BTreeMap::<(String, String), BTreeSet<String>>::new();
    for site in &model.realizes {
        realizations
            .entry((site.spec.clone(), site.scenario.clone()))
            .or_default()
            .extend(realization_identity(site));
    }

    let mut claims = BTreeMap::<String, TraceabilityClaim>::new();
    for claim in model.claims() {
        let id = claim.id();
        let relation_key = (claim.spec.id.clone(), claim.scenario.id.clone());
        let mut verification = model
            .evidence_bindings()
            .filter(|binding| binding.claim == id)
            .map(|binding| {
                let applicable = matches!(
                    claim.requirement.criticality,
                    Some(Criticality::Standard | Criticality::Critical)
                );
                let qualification = model
                    .qualifications()
                    .find(|qualification| qualification.id == binding.id);
                let current_qualification = qualification.filter(|qualification| {
                    applicable
                        && model
                            .expected_qualification_fingerprint(binding)
                            .is_some_and(|expected| qualification.fingerprint == expected)
                });
                let current = current_qualification.is_some();
                TraceabilityVerification {
                    check: binding.check.clone(),
                    binding: binding.id.clone(),
                    applicable,
                    current,
                    qualification: current_qualification
                        .map(|qualification| qualification.fingerprint.clone()),
                    verdict: current_qualification
                        .map(|qualification| qualification.verdict.name().to_string()),
                }
            })
            .collect::<Vec<_>>();
        verification.sort_by(|left, right| {
            (
                &left.binding,
                &left.check,
                left.applicable,
                left.current,
                &left.qualification,
                &left.verdict,
            )
                .cmp(&(
                    &right.binding,
                    &right.check,
                    right.applicable,
                    right.current,
                    &right.qualification,
                    &right.verdict,
                ))
        });
        verification.dedup();
        claims.insert(
            id.clone(),
            TraceabilityClaim {
                id: id.clone(),
                parent_requirement: claim.requirement.id.clone(),
                criticality: claim.requirement.criticality,
                statement: claim.requirement.statement.clone(),
                steps: claim
                    .scenario
                    .steps
                    .iter()
                    .map(|step| TraceabilityStep {
                        kind: step.kind,
                        text: step.text.clone(),
                    })
                    .collect(),
                realizations: realizations
                    .remove(&relation_key)
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                verification,
                judgment: traceability_judgment(model, &id, claim.requirement.criticality),
            },
        );
    }
    let mut challenge_resolutions = model
        .challenge_plans()
        .map(|plan| resolve_challenge_plan(model, plan))
        .collect::<Vec<_>>();
    challenge_resolutions.sort_by(|left, right| {
        (&left.plan, &left.challenger).cmp(&(&right.plan, &right.challenger))
    });
    TraceabilityReport {
        claims: claims.into_values().collect(),
        challenge_resolutions,
        decision_impacts: project_current_decision_impacts(model),
    }
}

fn traceability_judgment(
    model: &Model,
    claim_id: &str,
    criticality: Option<Criticality>,
) -> TraceabilityJudgment {
    let applicable = matches!(
        criticality,
        Some(Criticality::Standard | Criticality::Critical)
    );
    let judgment = model
        .claim_judgments()
        .find(|judgment| judgment.id == claim_id);
    let current_judgment = judgment.filter(|judgment| {
        applicable
            && model
                .expected_claim_judgment_fingerprint(judgment)
                .is_some_and(|expected| expected == judgment.fingerprint)
    });
    TraceabilityJudgment {
        applicable,
        current: current_judgment.is_some(),
        fingerprint: current_judgment.map(|judgment| judgment.fingerprint.clone()),
        verdict: current_judgment.map(|judgment| judgment.verdict.name().to_string()),
        policy: current_judgment.map(|judgment| judgment.policy.clone()),
    }
}

/// Projects only semantic dependency nodes and edges. It never creates a Challenge Result or
/// mutates repository decisions.
pub fn project_decision_impacts(
    model: &Model,
    targets: &[DecisionReference],
) -> DecisionImpactProjection {
    let mut nodes = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for target in targets {
        match target.kind {
            DecisionKind::Qualification => {
                let Some(binding) = model
                    .evidence_bindings()
                    .find(|binding| binding.id == target.id)
                else {
                    continue;
                };
                let Some(qualification) = model
                    .qualifications()
                    .find(|qualification| qualification.id == target.id)
                else {
                    continue;
                };
                let Some(expected) = model.expected_qualification_fingerprint(binding) else {
                    continue;
                };
                if expected != target.fingerprint || qualification.fingerprint != target.fingerprint
                {
                    continue;
                }
                let source = decision_node(target);
                let binding_node = impact_node(ImpactNodeKind::Binding, &binding.id, None);
                let claim_node = impact_node(ImpactNodeKind::Claim, &binding.claim, None);
                insert_edge(&mut nodes, &mut edges, source, binding_node.clone());
                insert_edge(&mut nodes, &mut edges, binding_node, claim_node.clone());
                if let Some(judgment_node) = current_judgment_node(model, &binding.claim) {
                    insert_edge(&mut nodes, &mut edges, claim_node, judgment_node);
                }
            }
            DecisionKind::ClaimJudgment => {
                let Some(judgment) = model
                    .claim_judgments()
                    .find(|judgment| judgment.id == target.id)
                else {
                    continue;
                };
                let Some(expected) = model.expected_claim_judgment_fingerprint(judgment) else {
                    continue;
                };
                if expected != target.fingerprint || judgment.fingerprint != target.fingerprint {
                    continue;
                }
                insert_edge(
                    &mut nodes,
                    &mut edges,
                    decision_node(target),
                    impact_node(ImpactNodeKind::Claim, &target.id, None),
                );
            }
        }
    }
    DecisionImpactProjection {
        nodes: nodes.into_iter().collect(),
        edges: edges.into_iter().collect(),
    }
}

pub fn project_current_decision_impacts(model: &Model) -> DecisionImpactProjection {
    let mut targets = Vec::new();
    for qualification in model.qualifications() {
        let Some(binding) = model
            .evidence_bindings()
            .find(|binding| binding.id == qualification.id)
        else {
            continue;
        };
        if model
            .expected_qualification_fingerprint(binding)
            .is_some_and(|expected| expected == qualification.fingerprint)
        {
            targets.push(DecisionReference {
                kind: DecisionKind::Qualification,
                id: qualification.id.clone(),
                fingerprint: qualification.fingerprint.clone(),
            });
        }
    }
    for judgment in model.claim_judgments() {
        if model
            .expected_claim_judgment_fingerprint(judgment)
            .is_some_and(|expected| expected == judgment.fingerprint)
        {
            targets.push(DecisionReference {
                kind: DecisionKind::ClaimJudgment,
                id: judgment.id.clone(),
                fingerprint: judgment.fingerprint.clone(),
            });
        }
    }
    targets.sort();
    targets.dedup();
    project_decision_impacts(model, &targets)
}

fn current_judgment_node(model: &Model, claim_id: &str) -> Option<DecisionImpactNode> {
    let judgment = model
        .claim_judgments()
        .find(|judgment| judgment.id == claim_id)?;
    let expected = model.expected_claim_judgment_fingerprint(judgment)?;
    (expected == judgment.fingerprint).then(|| {
        impact_node(
            ImpactNodeKind::ClaimJudgment,
            &judgment.id,
            Some(judgment.fingerprint.clone()),
        )
    })
}

fn decision_node(target: &DecisionReference) -> DecisionImpactNode {
    impact_node(
        match target.kind {
            DecisionKind::Qualification => ImpactNodeKind::Qualification,
            DecisionKind::ClaimJudgment => ImpactNodeKind::ClaimJudgment,
        },
        &target.id,
        Some(target.fingerprint.clone()),
    )
}

fn impact_node(kind: ImpactNodeKind, id: &str, fingerprint: Option<String>) -> DecisionImpactNode {
    DecisionImpactNode {
        kind,
        id: id.to_string(),
        fingerprint,
    }
}

fn insert_edge(
    nodes: &mut BTreeSet<DecisionImpactNode>,
    edges: &mut BTreeSet<DecisionImpactEdge>,
    from: DecisionImpactNode,
    to: DecisionImpactNode,
) {
    nodes.insert(from.clone());
    nodes.insert(to.clone());
    edges.insert(DecisionImpactEdge { from, to });
}

impl TraceabilityReport {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("version", Json::Num(2.0)),
            (
                "claims",
                Json::Arr(self.claims.iter().map(TraceabilityClaim::to_json).collect()),
            ),
            (
                "challenge_resolutions",
                Json::Arr(
                    self.challenge_resolutions
                        .iter()
                        .map(ChallengeResolution::to_json)
                        .collect(),
                ),
            ),
            ("decision_impacts", self.decision_impacts.to_json()),
        ])
    }
}

impl TraceabilityClaim {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("id", Json::str(&self.id)),
            ("parent_requirement", Json::str(&self.parent_requirement)),
            (
                "criticality",
                self.criticality
                    .map(|criticality| Json::str(criticality.name()))
                    .unwrap_or(Json::Null),
            ),
            ("statement", Json::str(&self.statement)),
            (
                "steps",
                Json::Arr(self.steps.iter().map(TraceabilityStep::to_json).collect()),
            ),
            (
                "realizations",
                Json::Arr(self.realizations.iter().map(Json::str).collect()),
            ),
            (
                "verification",
                Json::Arr(
                    self.verification
                        .iter()
                        .map(TraceabilityVerification::to_json)
                        .collect(),
                ),
            ),
            ("judgment", self.judgment.to_json()),
        ])
    }
}

impl TraceabilityJudgment {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("applicable", Json::Bool(self.applicable)),
            ("current", Json::Bool(self.current)),
            (
                "fingerprint",
                self.fingerprint
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "verdict",
                self.verdict.as_ref().map(Json::str).unwrap_or(Json::Null),
            ),
            (
                "policy",
                self.policy.as_ref().map(Json::str).unwrap_or(Json::Null),
            ),
        ])
    }
}

impl DecisionImpactProjection {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("format", Json::str("azimuth-decision-impact-projection")),
            ("version", Json::Num(1.0)),
            (
                "nodes",
                Json::Arr(self.nodes.iter().map(DecisionImpactNode::to_json).collect()),
            ),
            (
                "edges",
                Json::Arr(self.edges.iter().map(DecisionImpactEdge::to_json).collect()),
            ),
        ])
    }
}

impl DecisionImpactNode {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("kind", Json::str(self.kind.name())),
            ("id", Json::str(&self.id)),
            (
                "fingerprint",
                self.fingerprint
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
        ])
    }
}

impl DecisionImpactEdge {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("from", self.from.to_json()),
            ("to", self.to.to_json()),
        ])
    }
}

impl TraceabilityStep {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("kind", Json::str(self.kind.name())),
            ("text", Json::str(&self.text)),
        ])
    }
}

impl TraceabilityVerification {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("check", Json::str(&self.check)),
            ("binding", Json::str(&self.binding)),
            ("applicable", Json::Bool(self.applicable)),
            ("current", Json::Bool(self.current)),
            (
                "qualification",
                self.qualification
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "verdict",
                self.verdict.as_ref().map(Json::str).unwrap_or(Json::Null),
            ),
        ])
    }
}

fn realization_identity(site: &Site) -> Option<String> {
    site.source.as_ref().map(|source| source.key())
}
