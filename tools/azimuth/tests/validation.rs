use azimuth::design::parse_design;
use azimuth::model::{Artifact, CheckImplementation, Model, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::validation::{
    challenge_plan_relevant_to_selection, resolve_challenge_plan, validate, CandidateDisposition,
    DecisionKind, FindingKind, RelationKind,
};
use azimuth::verification::{
    parse_standards, parse_verification, ChallengeDomain, ClaimJudgmentVerdict,
    QualificationVerdict, Selector,
};
use std::collections::BTreeSet;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPEC: &str = "# Spec: alpha\n\n\
## Requirement: behavior\n\
Criticality: standard\n\n\
The system SHALL work.\n\n\
### Scenario: works\n\
WHEN invoked\n\
THEN it works\n";
const STANDARDS: &str = "# Decision policies and Challenge schedule\n\n\
## Decision Policy: credible\n\
Required challenge: mutation\n\n\
The implementation must be challenged.\n\n\
## Challenge Schedule: current\n\
Gate challenge: mutation\n\n\
Mutation is required at the gate.\n";
const VERIFICATION: &str = "# Verification: alpha\n\n\
## Check: alpha/works\n\
Method: invoke the behavior\n\
Terminal: the behavior works\n\n\
This is one atomic terminal result.\n\n\
## Evidence Binding: alpha/works-edge\n\
Check: alpha/works\n\
Claim: alpha#works\n\
Proposition: the result directly exercises the Claim\n\
Scope: unit\n\
Quantification: example\n\
Oracle: direct\n\
Context: {}\n\
Challenge domain: [\"realization\",\"mechanism\"]\n\
Policy: credible\n\n\
The edge is independently reviewable.\n\n\
## Qualification: alpha/works-edge\n\
Verdict: qualified\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Qualified: 2026-08-21\n\
Qualifier: owner@example\n\n\
The Check and oracle are credible.\n\n\
## Claim Judgment: alpha#works\n\
Verdict: accepted\n\
Policy: credible\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Judged: 2026-08-21\n\
Judge: owner@example\n\
Basis: the qualified Check directly exercises the case\n\
Residual risk: none identified\n\n\
The total composition is acceptable.\n\n\
## Challenger: mutation/perturb\n\
Form: mutation\n\
Searches for: a change the Check cannot detect\n\
Required scope: [\"binding\"]\n\n\
Survivors are objections.\n\n\
## Challenge Plan: alpha/credibility\n\
Challenger: mutation/perturb\n\
Select: qualification from binding alpha/works-edge\n\
Select: claim-judgment from claim alpha#works\n\n\
The plan targets both reviewed decisions.\n";

fn source(address: &str) -> SourceIdentity {
    SourceIdentity {
        area: "core".into(),
        kind: "rust-item".into(),
        address: address.into(),
        mount: "code".into(),
    }
}

fn model(criticality: &str) -> Model {
    let spec = parse_spec("spec.md", &SPEC.replace("standard", criticality)).unwrap();
    let mut model = Model {
        specs: vec![spec],
        realizes: vec![Site {
            spec: "alpha".into(),
            scenario: "works".into(),
            site: "alpha::works".into(),
            file: "src/alpha.rs".into(),
            lang: "rust".into(),
            source: Some(source("alpha::works")),
            source_fingerprint: SHA.into(),
        }],
        check_implementations: vec![CheckImplementation {
            check: "alpha/works".into(),
            site: "tests::works".into(),
            file: "tests/works.rs".into(),
            lang: "rust".into(),
            source: Some(source("tests::works")),
            source_fingerprint: SHA.into(),
        }],
        decision_standards: Some(parse_standards("standards.md", STANDARDS).unwrap()),
        verifications: vec![parse_verification("verification.md", VERIFICATION).unwrap()],
        ..Default::default()
    };
    refresh_decisions(&mut model);
    model
}

fn refresh_decisions(model: &mut Model) {
    for index in 0..model.verifications[0].bindings.len() {
        let expected = model
            .expected_qualification_fingerprint(&model.verifications[0].bindings[index])
            .unwrap();
        let id = model.verifications[0].bindings[index].id.clone();
        model.verifications[0]
            .qualifications
            .iter_mut()
            .find(|qualification| qualification.id == id)
            .unwrap()
            .fingerprint = expected;
    }
    for index in 0..model.verifications[0].claim_judgments.len() {
        if let Some(expected) = model
            .expected_claim_judgment_fingerprint(&model.verifications[0].claim_judgments[index])
        {
            model.verifications[0].claim_judgments[index].fingerprint = expected;
        }
    }
}

fn kinds(model: &Model) -> Vec<FindingKind> {
    validate(model)
        .into_iter()
        .map(|finding| finding.kind)
        .collect()
}

fn disposition(model: &Model, target: DecisionKind) -> CandidateDisposition {
    resolve_challenge_plan(model, &model.verifications[0].challenge_plans[0])
        .candidates
        .into_iter()
        .find(|candidate| candidate.selector.target == target)
        .unwrap()
        .disposition
}

#[test]
fn complete_non_routine_graph_is_clean_and_routine_without_verification_is_valid() {
    assert!(validate(&model("standard")).is_empty());
    let mut routine = model("routine");
    routine.verifications.clear();
    routine.realizes.clear();
    routine.check_implementations.clear();
    assert!(validate(&routine).is_empty());
}

#[test]
fn reports_qualification_and_judgment_precedence_without_double_reporting() {
    let mut value = model("standard");
    value.verifications[0].qualifications.clear();
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::MissingQualification));
    assert!(found.contains(&FindingKind::InvalidClaimJudgment));
    assert_eq!(
        disposition(&value, DecisionKind::Qualification),
        CandidateDisposition::MissingDecision
    );

    value = model("standard");
    value.verifications[0].bindings[0].check = "missing/check".into();
    assert_eq!(
        disposition(&value, DecisionKind::Qualification),
        CandidateDisposition::InvalidDecision
    );

    value = model("standard");
    value.verifications[0].bindings[0]
        .proposition
        .push_str(" changed");
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::StaleQualification));
    assert!(!found.contains(&FindingKind::RejectedQualification));
    assert_eq!(
        disposition(&value, DecisionKind::Qualification),
        CandidateDisposition::StaleDecision
    );

    value = model("standard");
    value.verifications[0].qualifications[0].verdict = QualificationVerdict::Rejected;
    refresh_decisions(&mut value);
    assert!(kinds(&value).contains(&FindingKind::RejectedQualification));
    assert_eq!(
        disposition(&value, DecisionKind::Qualification),
        CandidateDisposition::RejectedDecision
    );

    value = model("standard");
    value.verifications[0].claim_judgments[0].verdict = ClaimJudgmentVerdict::Rejected;
    refresh_decisions(&mut value);
    assert!(kinds(&value).contains(&FindingKind::RejectedClaimJudgment));
    assert_eq!(
        disposition(&value, DecisionKind::ClaimJudgment),
        CandidateDisposition::RejectedDecision
    );
}

#[test]
fn missing_stale_and_invalid_judgments_are_findings() {
    let mut value = model("standard");
    value.verifications[0].claim_judgments.clear();
    assert!(kinds(&value).contains(&FindingKind::MissingClaimJudgment));

    value = model("standard");
    value.verifications[0].claim_judgments[0].basis[0].push_str(" changed");
    assert!(kinds(&value).contains(&FindingKind::StaleClaimJudgment));

    value = model("standard");
    value.verifications[0].claim_judgments[0].policy = "missing".into();
    assert!(kinds(&value).contains(&FindingKind::InvalidClaimJudgment));
}

#[test]
fn all_seven_selectors_preserve_exact_relations_and_deduplicate_only_identical_records() {
    let mut value = model("standard");
    value.designs.push(
        parse_design(
            "design.md",
            "# Design: alpha\n\n## Claim: works\nMechanism: guard\n\
             Enforcement: guard\nBinding: artifact:guard\n\nA reason.\n",
        )
        .unwrap(),
    );
    value.artifacts.push(Artifact {
        id: "artifact:guard".into(),
        kind: "rust-method".into(),
        file: "src/alpha.rs".into(),
        unique: None,
        columns: Vec::new(),
        predicate: None,
        source: Some(source("alpha::guard")),
    });
    refresh_decisions(&mut value);
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::QualificationFromBinding("alpha/works-edge".into()),
        Selector::QualificationFromCheck("alpha/works".into()),
        Selector::QualificationFromRealization("core|rust-item|alpha::works".into()),
        Selector::QualificationFromMechanism("alpha#guard".into()),
        Selector::ClaimJudgmentFromClaim("alpha#works".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
        Selector::ClaimJudgmentFromMechanism("alpha#guard".into()),
        Selector::QualificationFromBinding("alpha/works-edge".into()),
    ];

    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.candidates.len(), 7);
    assert!(resolution
        .candidates
        .iter()
        .all(|candidate| candidate.disposition == CandidateDisposition::Selected));
    assert_eq!(
        resolution
            .candidates
            .iter()
            .filter(|candidate| candidate.selector.target == DecisionKind::Qualification)
            .map(|candidate| candidate.relation.kind)
            .collect::<Vec<_>>(),
        [
            RelationKind::Binding,
            RelationKind::Binding,
            RelationKind::Binding,
            RelationKind::Binding,
        ]
    );
    let json = resolution.to_json().to_string_pretty();
    assert!(json.contains("\"format\": \"azimuth-challenge-resolution\""));
    assert!(json.contains("\"version\": 1"));
    assert!(json.contains("\"candidates\""));
}

#[test]
fn successful_sibling_does_not_hide_missing_or_domain_excluded_binding() {
    let mut value = model("standard");
    let mut second_check = value.verifications[0].checks[0].clone();
    second_check.id = "alpha/alternate".into();
    let mut second_binding = value.verifications[0].bindings[0].clone();
    second_binding.id = "alpha/alternate-edge".into();
    second_binding.check = second_check.id.clone();
    second_binding.challenge_domain = vec![ChallengeDomain::Mechanism];
    let mut second_impl = value.check_implementations[0].clone();
    second_impl.check = second_check.id.clone();
    second_impl.source.as_mut().unwrap().address = "tests::alternate".into();
    value.verifications[0].checks.push(second_check);
    value.verifications[0].bindings.push(second_binding);
    value.check_implementations.push(second_impl);
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::QualificationFromRealization(
            "core|rust-item|alpha::works".into(),
        )];

    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.candidates.len(), 2);
    assert!(resolution
        .candidates
        .iter()
        .any(|candidate| candidate.disposition == CandidateDisposition::Selected));
    assert!(resolution
        .candidates
        .iter()
        .any(|candidate| candidate.disposition == CandidateDisposition::Inapplicable));
}

#[test]
fn unresolved_relations_retain_the_exact_direct_or_traversal_anchor() {
    let mut value = model("standard");
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::QualificationFromBinding("missing/binding".into()),
        Selector::QualificationFromCheck("missing/check".into()),
        Selector::QualificationFromRealization("core|rust-item|missing".into()),
        Selector::QualificationFromMechanism("alpha#missing".into()),
        Selector::ClaimJudgmentFromClaim("alpha#missing".into()),
    ];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.candidates.len(), 5);
    assert!(resolution.candidates.iter().all(|candidate| {
        candidate.disposition == CandidateDisposition::UnresolvedRelation
            && candidate.target.is_none()
            && candidate.relation.id == candidate.selector.id
    }));
}

#[test]
fn dangling_realization_does_not_invent_a_claim_relation() {
    let mut value = model("standard");
    value.realizes.push(Site {
        spec: "alpha".into(),
        scenario: "missing".into(),
        site: "alpha::dangling".into(),
        file: "src/dangling.rs".into(),
        lang: "rust".into(),
        source: Some(source("alpha::dangling")),
        source_fingerprint: SHA.into(),
    });
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::QualificationFromRealization("core|rust-item|alpha::dangling".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::dangling".into()),
    ];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(resolution.candidates.iter().all(|candidate| {
        candidate.disposition == CandidateDisposition::UnresolvedRelation
            && candidate.relation.kind == RelationKind::Realization
            && candidate.relation.id == "core|rust-item|alpha::dangling"
    }));
}

#[test]
fn qualification_unbound_claim_is_unresolved_but_judgment_traversal_reaches_the_claim() {
    let mut value = model("standard");
    value.verifications[0].bindings.clear();
    value.verifications[0].qualifications.clear();
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::QualificationFromRealization("core|rust-item|alpha::works".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
    ];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    let qualification = resolution
        .candidates
        .iter()
        .find(|candidate| candidate.selector.target == DecisionKind::Qualification)
        .unwrap();
    assert_eq!(qualification.relation.kind, RelationKind::Claim);
    assert_eq!(
        qualification.disposition,
        CandidateDisposition::UnresolvedRelation
    );
    let judgment = resolution
        .candidates
        .iter()
        .find(|candidate| candidate.selector.target == DecisionKind::ClaimJudgment)
        .unwrap();
    assert!(judgment.target.is_some());
    assert_eq!(judgment.disposition, CandidateDisposition::InvalidDecision);
}

#[test]
fn routine_precedes_missing_and_invalid_decisions() {
    let mut value = model("routine");
    value.verifications[0].qualifications.clear();
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(resolution
        .candidates
        .iter()
        .all(|candidate| candidate.disposition == CandidateDisposition::Inapplicable));
}

#[test]
fn required_form_coverage_checks_declared_scope_per_plan_and_target() {
    let mut value = model("standard");
    value.verifications[0].challengers[0].required_scope = vec![
        azimuth::verification::SemanticScopeKind::Binding,
        azimuth::verification::SemanticScopeKind::Realization,
    ];
    assert!(kinds(&value).contains(&FindingKind::InsufficientChallengeScope));

    value.verifications[0].challenge_plans[0].selectors.push(
        Selector::QualificationFromRealization("core|rust-item|alpha::works".into()),
    );
    assert!(!kinds(&value).contains(&FindingKind::InsufficientChallengeScope));

    value.decision_standards.as_mut().unwrap().policies[0]
        .required_challenges
        .push("static-analysis".into());
    value
        .decision_standards
        .as_mut()
        .unwrap()
        .schedule
        .gate_challenges
        .push("static-analysis".into());
    refresh_decisions(&mut value);
    assert!(kinds(&value).contains(&FindingKind::MissingRequiredChallenge));
}

#[test]
fn adverse_sibling_makes_a_plan_unrunnable_for_policy_coverage() {
    let mut value = model("standard");
    value.verifications[0].challenge_plans[0]
        .selectors
        .push(Selector::QualificationFromBinding("missing/binding".into()));
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(!resolution.is_runnable());
    assert!(kinds(&value).contains(&FindingKind::MissingRequiredChallenge));
}

#[test]
fn unstable_inputs_and_unprojectable_mechanisms_cannot_fake_scope_coverage() {
    let mut value = model("standard");
    value.verifications[0].challengers[0].required_scope =
        vec![azimuth::verification::SemanticScopeKind::CheckImplementation];
    value.check_implementations[0].source = None;
    refresh_decisions(&mut value);
    assert!(kinds(&value).contains(&FindingKind::InsufficientChallengeScope));

    value = model("standard");
    value.designs.push(
        parse_design(
            "design.md",
            "# Design: alpha\n\n## Claim: works\nMechanism: guard\n\
             Enforcement: guard\nBinding: artifact:missing\n\nA reason.\n",
        )
        .unwrap(),
    );
    value.verifications[0].challengers[0].required_scope =
        vec![azimuth::verification::SemanticScopeKind::Mechanism];
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::QualificationFromMechanism("alpha#guard".into())];
    assert_eq!(
        disposition(&value, DecisionKind::Qualification),
        CandidateDisposition::Selected
    );
    assert!(kinds(&value).contains(&FindingKind::InsufficientChallengeScope));
}

#[test]
fn duplicate_candidate_identity_is_an_explicit_resolution_failure() {
    let mut value = model("standard");
    let duplicate = value.verifications[0].bindings[0].clone();
    value.verifications[0].bindings.push(duplicate);
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::QualificationFromCheck("alpha/works".into())];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.issues.len(), 1);
    assert!(!resolution.is_runnable());
    assert!(kinds(&value).contains(&FindingKind::InvalidChallengeResolution));
}

#[test]
fn relevant_plans_are_retained_atomically_without_success_filtering() {
    let mut value = model("standard");
    value.verifications[0].challenge_plans[0]
        .selectors
        .push(Selector::QualificationFromBinding("missing/binding".into()));
    let claims = BTreeSet::from(["alpha#works".to_string()]);
    let bindings = BTreeSet::from(["alpha/works-edge".to_string()]);
    let checks = BTreeSet::from(["alpha/works".to_string()]);
    assert!(challenge_plan_relevant_to_selection(
        &value,
        &value.verifications[0].challenge_plans[0],
        &claims,
        &bindings,
        &checks,
    ));
    assert_eq!(value.verifications[0].challenge_plans[0].selectors.len(), 3);
}

#[test]
fn finding_registry_is_exhaustive_and_has_guidance() {
    assert_eq!(FindingKind::ALL.len(), 42);
    let mut names = BTreeSet::new();
    for kind in FindingKind::ALL {
        assert!(!kind.name().is_empty());
        assert!(
            names.insert(kind.name()),
            "duplicate kind `{}`",
            kind.name()
        );
        assert!(!kind.category().name().is_empty());
        assert!(!kind.help().is_empty());
    }
    assert_eq!(
        FindingKind::MissingClaimJudgment.category().name(),
        "judgment"
    );
    assert_eq!(
        FindingKind::UnresolvedChallengeRelation.category().name(),
        "verification"
    );
}
