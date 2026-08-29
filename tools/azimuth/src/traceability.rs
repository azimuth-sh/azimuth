//! Pure derived Claim, realization, and verification traceability projection.

use crate::json::Json;
use crate::model::{Criticality, Model, Site, StepKind};
use crate::validation::{resolve_challenge_plan, ChallengeResolution, DecisionKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityReport {
    pub cases: Vec<TraceabilityCase>,
    pub challenge_resolutions: Vec<ChallengeResolution>,
    pub decision_impacts: DecisionImpactProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityCase {
    pub id: String,
    pub parent_claim: String,
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
    pub method_qualification: Option<String>,
    pub method_verdict: Option<String>,
    pub applicability_decision: Option<String>,
    pub applicability_verdict: Option<String>,
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
    ApplicabilityDecision,
    Binding,
    Case,
    Claim,
    ClaimJudgment,
    MethodQualification,
}

impl ImpactNodeKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::ApplicabilityDecision => "applicability-decision",
            Self::Binding => "binding",
            Self::Case => "case",
            Self::Claim => "claim",
            Self::ClaimJudgment => "claim-judgment",
            Self::MethodQualification => "method-qualification",
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
            .entry((site.spec.clone(), site.claim.clone()))
            .or_default()
            .extend(realization_identity(site));
    }

    let mut cases = BTreeMap::<String, TraceabilityCase>::new();
    for claim in model.cases() {
        let id = claim.id();
        let relation_key = (claim.spec.id.clone(), claim.claim.id.clone());
        let mut verification = model
            .evidence_bindings()
            .filter(|binding| binding.case == id)
            .map(|binding| {
                let applicable = matches!(
                    claim.claim.criticality,
                    Some(Criticality::Standard | Criticality::Critical)
                );
                let qualification = model
                    .method_qualifications()
                    .find(|qualification| qualification.id == binding.method_qualification);
                let current_qualification = qualification.filter(|qualification| {
                    applicable
                        && model
                            .expected_method_qualification_fingerprint(qualification)
                            .is_some_and(|expected| qualification.fingerprint == expected)
                });
                let decision = model
                    .applicability_decisions()
                    .find(|decision| decision.id == binding.id);
                let current_decision = decision.filter(|decision| {
                    applicable
                        && model
                            .expected_applicability_fingerprint(binding)
                            .is_some_and(|expected| decision.fingerprint == expected)
                });
                let current = current_qualification.is_some() && current_decision.is_some();
                TraceabilityVerification {
                    check: binding.check.clone(),
                    binding: binding.id.clone(),
                    applicable,
                    current,
                    method_qualification: current_qualification
                        .map(|qualification| qualification.fingerprint.clone()),
                    method_verdict: current_qualification
                        .map(|qualification| qualification.verdict.name().to_string()),
                    applicability_decision: current_decision
                        .map(|decision| decision.fingerprint.clone()),
                    applicability_verdict: current_decision
                        .map(|decision| decision.verdict.name().to_string()),
                }
            })
            .collect::<Vec<_>>();
        verification.sort_by(|left, right| {
            (
                &left.binding,
                &left.check,
                left.applicable,
                left.current,
                &left.method_qualification,
                &left.method_verdict,
                &left.applicability_decision,
                &left.applicability_verdict,
            )
                .cmp(&(
                    &right.binding,
                    &right.check,
                    right.applicable,
                    right.current,
                    &right.method_qualification,
                    &right.method_verdict,
                    &right.applicability_decision,
                    &right.applicability_verdict,
                ))
        });
        verification.dedup();
        cases.insert(
            id.clone(),
            TraceabilityCase {
                id: id.clone(),
                parent_claim: format!("{}#{}", claim.spec.id, claim.claim.id),
                criticality: claim.claim.criticality,
                statement: claim.claim.statement.clone(),
                steps: claim
                    .case
                    .steps
                    .iter()
                    .map(|step| TraceabilityStep {
                        kind: step.kind,
                        text: step.text.clone(),
                    })
                    .collect(),
                realizations: realizations
                    .get(&relation_key)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                verification,
                judgment: traceability_judgment(
                    model,
                    &format!("{}#{}", claim.spec.id, claim.claim.id),
                    claim.claim.criticality,
                ),
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
        cases: cases.into_values().collect(),
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
            DecisionKind::ApplicabilityDecision => {
                let Some(binding) = model
                    .evidence_bindings()
                    .find(|binding| binding.id == target.id)
                else {
                    continue;
                };
                let Some(decision) = model
                    .applicability_decisions()
                    .find(|decision| decision.id == target.id)
                else {
                    continue;
                };
                let Some(expected) = model.expected_applicability_fingerprint(binding) else {
                    continue;
                };
                if expected != target.fingerprint || decision.fingerprint != target.fingerprint {
                    continue;
                }
                let source = decision_node(target);
                let binding_node = impact_node(ImpactNodeKind::Binding, &binding.id, None);
                let case_node = impact_node(ImpactNodeKind::Case, &binding.case, None);
                let Some((claim_id, _)) = binding.case.rsplit_once('/') else {
                    continue;
                };
                let claim_node = impact_node(ImpactNodeKind::Claim, claim_id, None);
                insert_edge(&mut nodes, &mut edges, source, binding_node.clone());
                insert_edge(&mut nodes, &mut edges, binding_node, case_node.clone());
                insert_edge(&mut nodes, &mut edges, case_node, claim_node.clone());
                if let Some(judgment_node) = current_judgment_node(model, claim_id) {
                    insert_edge(&mut nodes, &mut edges, claim_node, judgment_node);
                }
            }
            DecisionKind::MethodQualification => {
                let Some(qualification) = model
                    .method_qualifications()
                    .find(|qualification| qualification.id == target.id)
                else {
                    continue;
                };
                let Some(expected) = model.expected_method_qualification_fingerprint(qualification)
                else {
                    continue;
                };
                if expected != target.fingerprint || qualification.fingerprint != target.fingerprint
                {
                    continue;
                }
                let source = decision_node(target);
                for binding in model
                    .evidence_bindings()
                    .filter(|binding| binding.method_qualification == target.id)
                {
                    let binding_node = impact_node(ImpactNodeKind::Binding, &binding.id, None);
                    let case_node = impact_node(ImpactNodeKind::Case, &binding.case, None);
                    let Some((claim_id, _)) = binding.case.rsplit_once('/') else {
                        continue;
                    };
                    let claim_node = impact_node(ImpactNodeKind::Claim, claim_id, None);
                    let current_applicability = model
                        .applicability_decisions()
                        .find(|decision| decision.id == binding.id)
                        .and_then(|decision| {
                            model
                                .expected_applicability_fingerprint(binding)
                                .filter(|expected| *expected == decision.fingerprint)
                                .map(|_| {
                                    impact_node(
                                        ImpactNodeKind::ApplicabilityDecision,
                                        &decision.id,
                                        Some(decision.fingerprint.clone()),
                                    )
                                })
                        });
                    if let Some(applicability_node) = current_applicability {
                        insert_edge(
                            &mut nodes,
                            &mut edges,
                            source.clone(),
                            applicability_node.clone(),
                        );
                        insert_edge(
                            &mut nodes,
                            &mut edges,
                            applicability_node,
                            binding_node.clone(),
                        );
                    } else {
                        insert_edge(&mut nodes, &mut edges, source.clone(), binding_node.clone());
                    }
                    insert_edge(&mut nodes, &mut edges, binding_node, case_node.clone());
                    insert_edge(&mut nodes, &mut edges, case_node, claim_node.clone());
                    if let Some(judgment_node) = current_judgment_node(model, claim_id) {
                        insert_edge(&mut nodes, &mut edges, claim_node, judgment_node);
                    }
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
    for qualification in model.method_qualifications() {
        if model
            .expected_method_qualification_fingerprint(qualification)
            .is_some_and(|expected| expected == qualification.fingerprint)
        {
            targets.push(DecisionReference {
                kind: DecisionKind::MethodQualification,
                id: qualification.id.clone(),
                fingerprint: qualification.fingerprint.clone(),
            });
        }
    }
    for decision in model.applicability_decisions() {
        let Some(binding) = model
            .evidence_bindings()
            .find(|binding| binding.id == decision.id)
        else {
            continue;
        };
        if model
            .expected_applicability_fingerprint(binding)
            .is_some_and(|expected| expected == decision.fingerprint)
        {
            targets.push(DecisionReference {
                kind: DecisionKind::ApplicabilityDecision,
                id: decision.id.clone(),
                fingerprint: decision.fingerprint.clone(),
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
            DecisionKind::ApplicabilityDecision => ImpactNodeKind::ApplicabilityDecision,
            DecisionKind::ClaimJudgment => ImpactNodeKind::ClaimJudgment,
            DecisionKind::MethodQualification => ImpactNodeKind::MethodQualification,
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
            ("version", Json::Num(3.0)),
            (
                "cases",
                Json::Arr(self.cases.iter().map(TraceabilityCase::to_json).collect()),
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

impl TraceabilityCase {
    fn to_json(&self) -> Json {
        Json::obj(vec![
            ("id", Json::str(&self.id)),
            ("parent_claim", Json::str(&self.parent_claim)),
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
                "method_qualification",
                self.method_qualification
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "method_verdict",
                self.method_verdict
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "applicability_decision",
                self.applicability_decision
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
            (
                "applicability_verdict",
                self.applicability_verdict
                    .as_ref()
                    .map(Json::str)
                    .unwrap_or(Json::Null),
            ),
        ])
    }
}

fn realization_identity(site: &Site) -> Option<String> {
    site.source.as_ref().map(|source| source.key())
}
