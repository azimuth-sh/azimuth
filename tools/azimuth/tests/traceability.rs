use azimuth::model::{CheckImplementation, Criticality, Model, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::traceability::{project, project_decision_impacts, DecisionReference, ImpactNodeKind};
use azimuth::validation::{CandidateDisposition, DecisionKind};
use azimuth::verification::{
    parse_standards, parse_verification, ClaimJudgmentVerdict, QualificationVerdict,
};

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const STANDARDS: &str = "# Decision policies and Challenge schedule\n\n\
## Decision Policy: credible\n\
Required challenge: mutation\n\n\
The decision must be challenged.\n\n\
## Challenge Schedule: current\n\
Gate challenge: mutation\n\n\
Mutation is gate work.\n";
const VERIFICATION: &str = "# Verification: alpha\n\n\
## Check: alpha/check\n\
Method: invoke\n\
Terminal: works\n\n\
Atomic.\n\n\
## Evidence Binding: alpha/edge\n\
Check: alpha/check\n\
Claim: alpha#observed\n\
Proposition: direct\n\
Scope: unit\n\
Quantification: example\n\
Oracle: direct\n\
Context: {}\n\
Challenge domain: [\"realization\"]\n\
Policy: credible\n\n\
Reviewable.\n\n\
## Qualification: alpha/edge\n\
Verdict: qualified\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Qualified: 2026-08-21\n\
Qualifier: owner\n\n\
Qualified.\n\n\
## Claim Judgment: alpha#observed\n\
Verdict: accepted\n\
Policy: credible\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Judged: 2026-08-21\n\
Judge: owner\n\
Basis: the Check directly exercises the case\n\
Residual risk: none identified\n\n\
Accepted.\n\n\
## Challenger: mutation/search\n\
Form: mutation\n\
Searches for: an undetected change\n\
Required scope: [\"binding\"]\n\n\
Searches the exact edge.\n\n\
## Challenge Plan: alpha/decisions\n\
Challenger: mutation/search\n\
Select: qualification from binding alpha/edge\n\
Select: claim-judgment from claim alpha#observed\n\n\
Targets both decisions.\n";

fn source(address: &str) -> SourceIdentity {
    SourceIdentity {
        area: "core".into(),
        kind: "rust-item".into(),
        address: address.into(),
        mount: "code".into(),
    }
}

fn check_implementation(check: &str, address: &str) -> CheckImplementation {
    CheckImplementation {
        check: check.into(),
        site: format!("tests::{address}"),
        file: format!("tests/{address}.rs"),
        lang: "rust".into(),
        source: Some(source(&format!("tests::{address}"))),
        source_fingerprint: SHA.into(),
    }
}

fn model() -> Model {
    let spec = parse_spec(
        "spec.md",
        "# Spec: alpha\n\n## Requirement: works\nCriticality: standard\n\nA SHALL work.\n\n\
         ### Scenario: observed\nWHEN invoked\nTHEN it works\n",
    )
    .unwrap();
    let mut model = Model {
        specs: vec![spec],
        realizes: vec![
            Site {
                spec: "alpha".into(),
                scenario: "observed".into(),
                site: "z".into(),
                file: "z.rs".into(),
                lang: "rust".into(),
                source: Some(source("z")),
                source_fingerprint: SHA.into(),
            },
            Site {
                spec: "alpha".into(),
                scenario: "observed".into(),
                site: "a".into(),
                file: "a.rs".into(),
                lang: "rust".into(),
                source: Some(source("a")),
                source_fingerprint: SHA.into(),
            },
        ],
        decision_standards: Some(parse_standards("standards.md", STANDARDS).unwrap()),
        verifications: vec![parse_verification("verification.md", VERIFICATION).unwrap()],
        check_implementations: vec![check_implementation("alpha/check", "check")],
        ..Default::default()
    };
    refresh(&mut model);
    model
}

fn refresh(model: &mut Model) {
    for index in 0..model.verifications[0].bindings.len() {
        let id = model.verifications[0].bindings[index].id.clone();
        let expected = model
            .expected_qualification_fingerprint(&model.verifications[0].bindings[index])
            .unwrap();
        model.verifications[0]
            .qualifications
            .iter_mut()
            .find(|qualification| qualification.id == id)
            .unwrap()
            .fingerprint = expected;
    }
    let expected = model
        .expected_claim_judgment_fingerprint(&model.verifications[0].claim_judgments[0])
        .unwrap();
    model.verifications[0].claim_judgments[0].fingerprint = expected;
}

#[test]
fn projects_sorted_realization_verification_and_current_judgment_relationships() {
    let report = project(&model());
    let claim = &report.claims[0];
    assert_eq!(claim.id, "alpha#observed");
    assert_eq!(claim.realizations, ["core|rust-item|a", "core|rust-item|z"]);
    assert_eq!(claim.verification[0].binding, "alpha/edge");
    assert!(claim.verification[0].current);
    assert_eq!(claim.verification[0].verdict.as_deref(), Some("qualified"));
    assert!(claim.judgment.applicable);
    assert!(claim.judgment.current);
    assert_eq!(claim.judgment.verdict.as_deref(), Some("accepted"));
    assert_eq!(claim.judgment.policy.as_deref(), Some("credible"));
}

#[test]
fn traceability_exposes_the_same_strict_candidate_account() {
    let report = project(&model());
    assert_eq!(report.challenge_resolutions.len(), 1);
    assert_eq!(report.challenge_resolutions[0].candidates.len(), 2);
    assert!(report.challenge_resolutions[0]
        .candidates
        .iter()
        .all(|candidate| candidate.disposition == CandidateDisposition::Selected));
    let json = report.to_json().to_string_pretty();
    assert!(json.contains("\"challenge_resolutions\""));
    assert!(json.contains("\"authored_fingerprint\""));
    assert!(json.contains("\"decision_impacts\""));
}

#[test]
fn challenge_resolutions_sort_independently_of_authority_order() {
    let mut value = model();
    let mut later = value.verifications[0].challenge_plans[0].clone();
    later.id = "zeta/decisions".into();
    let earlier = value.verifications[0].challenge_plans[0].clone();
    value.verifications[0].challenge_plans = vec![later, earlier];

    let report = project(&value);
    assert_eq!(
        report
            .challenge_resolutions
            .iter()
            .map(|resolution| resolution.plan.as_str())
            .collect::<Vec<_>>(),
        ["alpha/decisions", "zeta/decisions"]
    );
}

#[test]
fn impact_graph_deduplicates_one_dependent_judgment_across_qualifications() {
    let mut value = model();
    let mut check = value.verifications[0].checks[0].clone();
    check.id = "alpha/alternate".into();
    let mut binding = value.verifications[0].bindings[0].clone();
    binding.id = "alpha/alternate-edge".into();
    binding.check = check.id.clone();
    let mut qualification = value.verifications[0].qualifications[0].clone();
    qualification.id = binding.id.clone();
    let implementation = check_implementation(&check.id, "alternate");
    value.verifications[0].checks.push(check);
    value.verifications[0].bindings.push(binding);
    value.verifications[0].qualifications.push(qualification);
    value.check_implementations.push(implementation);
    value.verifications[0].claim_judgments[0].verdict = ClaimJudgmentVerdict::Rejected;
    refresh(&mut value);

    let targets = value.verifications[0]
        .qualifications
        .iter()
        .map(|qualification| DecisionReference {
            kind: DecisionKind::Qualification,
            id: qualification.id.clone(),
            fingerprint: qualification.fingerprint.clone(),
        })
        .collect::<Vec<_>>();
    let impact = project_decision_impacts(&value, &targets);
    assert_eq!(
        impact
            .nodes
            .iter()
            .filter(|node| node.kind == ImpactNodeKind::ClaimJudgment)
            .count(),
        1
    );
    assert_eq!(impact.nodes.len(), 6);
    assert_eq!(impact.edges.len(), 5);
}

#[test]
fn direct_judgment_impact_reaches_only_its_claim_without_manufactured_results() {
    let value = model();
    let judgment = &value.verifications[0].claim_judgments[0];
    let impact = project_decision_impacts(
        &value,
        &[DecisionReference {
            kind: DecisionKind::ClaimJudgment,
            id: judgment.id.clone(),
            fingerprint: judgment.fingerprint.clone(),
        }],
    );
    assert_eq!(impact.nodes.len(), 2);
    assert_eq!(impact.edges.len(), 1);
    assert_eq!(impact.edges[0].from.kind, ImpactNodeKind::ClaimJudgment);
    assert_eq!(impact.edges[0].to.kind, ImpactNodeKind::Claim);
    let json = impact.to_json().to_string_pretty();
    assert!(json.contains("\"format\": \"azimuth-decision-impact-projection\""));
    assert!(json.contains("\"version\": 1"));
    assert!(!json.contains("result"));
    assert!(!json.contains("observation"));
    assert!(!json.contains("state"));
}

#[test]
fn stale_decisions_are_not_presented_as_current_or_projected_as_impact() {
    let mut value = model();
    value.verifications[0].bindings[0]
        .proposition
        .push_str(" changed");
    let report = project(&value);
    assert!(!report.claims[0].verification[0].current);
    assert!(!report.claims[0].judgment.current);
    assert!(report.decision_impacts.nodes.is_empty());
    assert!(report.decision_impacts.edges.is_empty());
}

#[test]
fn routine_relationships_are_inapplicable_and_never_current() {
    let mut value = model();
    value.specs[0].requirements[0].criticality = Some(Criticality::Routine);
    let report = project(&value);
    assert!(report.claims[0].verification.iter().all(|relationship| {
        !relationship.applicable && !relationship.current && relationship.qualification.is_none()
    }));
    assert!(!report.claims[0].judgment.applicable);
    assert!(!report.claims[0].judgment.current);
}

#[test]
fn report_is_deterministic_and_creates_no_execution_or_authority_fields() {
    let left = project(&model()).to_json().to_string_pretty();
    let right = project(&model()).to_json().to_string_pretty();
    assert_eq!(left, right);
    assert!(left.contains("\"version\": 2"));
    assert!(!left.contains("observations"));
    assert!(!left.contains("challenge_results"));
    assert!(!left.contains("assurance_state"));
    assert!(!left.contains("rationale"));
    assert!(!left.contains("tests/"));
}

#[test]
fn current_negative_qualification_remains_current_traceability_not_selected_resolution() {
    let mut value = model();
    value.verifications[0].qualifications[0].verdict = QualificationVerdict::Rejected;
    refresh(&mut value);
    let report = project(&value);
    assert!(report.claims[0].verification[0].current);
    assert_eq!(
        report.claims[0].verification[0].verdict.as_deref(),
        Some("rejected")
    );
    let candidate = report.challenge_resolutions[0]
        .candidates
        .iter()
        .find(|candidate| candidate.selector.target == DecisionKind::Qualification)
        .unwrap();
    assert_eq!(
        candidate.disposition,
        CandidateDisposition::RejectedDecision
    );
}
