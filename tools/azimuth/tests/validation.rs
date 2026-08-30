use azimuth::design::parse_design;
use azimuth::model::{Artifact, CheckImplementation, Model, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::validation::{
    challenge_plan_relevant_to_selection, resolve_challenge_plan, validate, CandidateDisposition,
    DecisionKind, FindingKind, RelationKind,
};
use azimuth::verification::{
    parse_standards, parse_verification, ChallengeDomain, ClaimJudgmentVerdict,
    MethodQualificationVerdict, Selector,
};
use std::collections::BTreeSet;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPEC: &str = "# Spec: alpha\n\n\
## Claim: behavior\n\
Criticality: standard\n\n\
The system SHALL work.\n\n\
### Case: works\n\
Event: invoked\n\
Required: it works\n";
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
Case: alpha#behavior/works\n\
Method qualification: alpha/method\n\
Proposition: the result directly exercises the Claim\n\
Context: {}\n\
Challenge domain: [\"realization\",\"mechanism\"]\n\
Policy: credible\n\n\
The edge is independently reviewable.\n\n\
## Method Qualification: alpha/method\n\
Check: alpha/works\n\
Scope: unit\n\
Quantification: example\n\
Oracle: direct\n\
Context: {}\n\
Challenge domain: [\"realization\",\"mechanism\"]\n\
Policy: credible\n\
Verdict: qualified\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Qualified: 2026-08-21\n\
Qualifier: owner@example\n\n\
The Check and oracle are credible.\n\n\
## Applicability Decision: alpha/works-edge\n\
Verdict: applicable\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Decided: 2026-08-21\n\
Decider: owner@example\n\n\
The qualified method applies to this edge.\n\n\
## Claim Judgment: alpha#behavior\n\
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
Required scope: [\"context\"]\n\n\
Survivors are objections.\n\n\
## Challenge Plan: alpha/credibility\n\
Challenger: mutation/perturb\n\
Select: applicability-decision from binding alpha/works-edge\n\
Select: method-qualification from method-qualification alpha/method\n\
Select: claim-judgment from claim alpha#behavior\n\n\
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
            claim: "behavior".into(),
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
    for index in 0..model.verifications[0].method_qualifications.len() {
        let expected = model
            .expected_method_qualification_fingerprint(
                &model.verifications[0].method_qualifications[index],
            )
            .unwrap();
        model.verifications[0].method_qualifications[index].fingerprint = expected;
    }
    for index in 0..model.verifications[0].applicability_decisions.len() {
        if let Some(expected) =
            model.expected_applicability_fingerprint(&model.verifications[0].bindings[index])
        {
            model.verifications[0].applicability_decisions[index].fingerprint = expected;
        }
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
    let findings = validate(&model("standard"));
    assert!(findings.is_empty(), "{findings:?}");
    let mut routine = model("routine");
    routine.verifications.clear();
    routine.realizes.clear();
    routine.check_implementations.clear();
    assert!(validate(&routine).is_empty());
}

#[test]
fn reports_qualification_and_judgment_precedence_without_double_reporting() {
    let mut value = model("standard");
    value.verifications[0].method_qualifications.clear();
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::MissingMethodQualification));
    assert!(found.contains(&FindingKind::InvalidClaimJudgment));
    assert_eq!(
        disposition(&value, DecisionKind::MethodQualification),
        CandidateDisposition::UnresolvedRelation
    );

    value = model("standard");
    value.verifications[0].method_qualifications[0].check = "missing/check".into();
    assert_eq!(
        disposition(&value, DecisionKind::MethodQualification),
        CandidateDisposition::InvalidDecision
    );

    value = model("standard");
    value.verifications[0].method_qualifications[0]
        .context
        .insert("platform".into(), "changed".into());
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::StaleMethodQualification));
    assert!(!found.contains(&FindingKind::RejectedMethodQualification));
    assert_eq!(
        disposition(&value, DecisionKind::MethodQualification),
        CandidateDisposition::StaleDecision
    );

    value = model("standard");
    value.verifications[0].method_qualifications[0].verdict = MethodQualificationVerdict::Rejected;
    refresh_decisions(&mut value);
    assert!(kinds(&value).contains(&FindingKind::RejectedMethodQualification));
    assert_eq!(
        disposition(&value, DecisionKind::MethodQualification),
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
            "# Design: alpha\n\n## Claim: behavior\nMechanism: guard\n\
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
        Selector::ApplicabilityDecisionFromBinding("alpha/works-edge".into()),
        Selector::MethodQualificationFromCheck("alpha/works".into()),
        Selector::MethodQualificationFromRealization("core|rust-item|alpha::works".into()),
        Selector::MethodQualificationFromMechanism("alpha#guard".into()),
        Selector::ClaimJudgmentFromClaim("alpha#behavior".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
        Selector::ClaimJudgmentFromMechanism("alpha#guard".into()),
        Selector::ApplicabilityDecisionFromBinding("alpha/works-edge".into()),
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
            .filter(|candidate| candidate.selector.target == DecisionKind::MethodQualification)
            .map(|candidate| candidate.relation.kind)
            .collect::<Vec<_>>(),
        [
            RelationKind::MethodQualification,
            RelationKind::MethodQualification,
            RelationKind::MethodQualification,
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
    second_binding.method_qualification = "alpha/alternate-method".into();
    second_binding.challenge_domain = vec![ChallengeDomain::Mechanism];
    let mut second_impl = value.check_implementations[0].clone();
    second_impl.check = second_check.id.clone();
    second_impl.source.as_mut().unwrap().address = "tests::alternate".into();
    value.verifications[0].checks.push(second_check);
    value.verifications[0].bindings.push(second_binding);
    value.check_implementations.push(second_impl);
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::MethodQualificationFromRealization(
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
        .any(|candidate| candidate.disposition != CandidateDisposition::Selected));
}

#[test]
fn unresolved_relations_retain_the_exact_direct_or_traversal_anchor() {
    let mut value = model("standard");
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::ApplicabilityDecisionFromBinding("missing/binding".into()),
        Selector::MethodQualificationFromCheck("missing/check".into()),
        Selector::MethodQualificationFromRealization("core|rust-item|missing".into()),
        Selector::MethodQualificationFromMechanism("alpha#missing".into()),
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
        claim: "missing".into(),
        site: "alpha::dangling".into(),
        file: "src/dangling.rs".into(),
        lang: "rust".into(),
        source: Some(source("alpha::dangling")),
        source_fingerprint: SHA.into(),
    });
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::MethodQualificationFromRealization("core|rust-item|alpha::dangling".into()),
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
    value.verifications[0].method_qualifications.clear();
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::MethodQualificationFromRealization("core|rust-item|alpha::works".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
    ];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    let qualification = resolution
        .candidates
        .iter()
        .find(|candidate| candidate.selector.target == DecisionKind::MethodQualification)
        .unwrap();
    assert_eq!(qualification.relation.kind, RelationKind::Realization);
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
    value.verifications[0].method_qualifications[0].fingerprint = "invalid".into();
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(resolution
        .candidates
        .iter()
        .all(|candidate| candidate.disposition == CandidateDisposition::Inapplicable));
}

#[test]
fn required_form_coverage_checks_declared_scope_per_plan_and_target() {
    let mut value = model("standard");
    value.verifications[0].challengers[0].required_scope =
        vec![azimuth::verification::SemanticScopeKind::Realization];
    assert!(kinds(&value).contains(&FindingKind::InsufficientChallengeScope));

    value.verifications[0].challenge_plans[0].selectors.extend([
        Selector::MethodQualificationFromRealization("core|rust-item|alpha::works".into()),
        Selector::ApplicabilityDecisionFromRealization("core|rust-item|alpha::works".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
    ]);
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
    value.verifications[0].challenge_plans[0].selectors.push(
        Selector::ApplicabilityDecisionFromBinding("missing/binding".into()),
    );
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
            "# Design: alpha\n\n## Claim: behavior\nMechanism: guard\n\
             Enforcement: guard\nBinding: artifact:missing\n\nA reason.\n",
        )
        .unwrap(),
    );
    value.verifications[0].challengers[0].required_scope =
        vec![azimuth::verification::SemanticScopeKind::Mechanism];
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::MethodQualificationFromMechanism(
            "alpha#guard".into(),
        )];
    assert_eq!(
        disposition(&value, DecisionKind::MethodQualification),
        CandidateDisposition::Selected
    );
    assert!(kinds(&value).contains(&FindingKind::InsufficientChallengeScope));
}

#[test]
fn duplicate_candidate_identity_is_an_explicit_resolution_failure() {
    let mut value = model("standard");
    let duplicate = value.verifications[0].method_qualifications[0].clone();
    value.verifications[0].method_qualifications.push(duplicate);
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::MethodQualificationFromCheck("alpha/works".into())];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.issues.len(), 1);
    assert!(!resolution.is_runnable());
    assert!(kinds(&value).contains(&FindingKind::InvalidChallengeResolution));
}

#[test]
fn relevant_plans_are_retained_atomically_without_success_filtering() {
    let mut value = model("standard");
    value.verifications[0].challenge_plans[0].selectors.push(
        Selector::ApplicabilityDecisionFromBinding("missing/binding".into()),
    );
    let claims = BTreeSet::from(["alpha#behavior".to_string()]);
    let bindings = BTreeSet::from(["alpha/works-edge".to_string()]);
    let checks = BTreeSet::from(["alpha/works".to_string()]);
    assert!(challenge_plan_relevant_to_selection(
        &value,
        &value.verifications[0].challenge_plans[0],
        &claims,
        &bindings,
        &checks,
    ));
    assert_eq!(value.verifications[0].challenge_plans[0].selectors.len(), 4);
}

#[test]
fn finding_registry_is_exhaustive_and_has_guidance() {
    assert_eq!(FindingKind::ALL.len(), 46);
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
