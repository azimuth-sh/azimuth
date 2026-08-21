use azimuth::design::parse_design;
use azimuth::model::{Artifact, CheckImplementation, Model, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::validation::{resolve_challenge_plan, validate, FindingKind};
use azimuth::verification::{
    parse_policies, parse_verification, ChallengeDomain, QualificationVerdict, Selector,
    Verification,
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
const POLICIES: &str = "# Qualification policies\n\n\
## Policy: credible\n\
Required challenge: implementation-perturbation\n\n\
The implementation must be challenged.\n";
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
Qualification policy: credible\n\n\
The edge is independently reviewable.\n\n\
## Qualification: alpha/works-edge\n\
Verdict: qualified\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Qualified: 2026-08-21\n\
Qualifier: owner@example\n\n\
The Check and oracle are credible.\n\n\
## Challenger: mutation/perturb\n\
Form: implementation-perturbation\n\
Searches for: a change the Check cannot detect\n\n\
Survivors are objections.\n\n\
## Challenge Plan: alpha/credibility\n\
Challenger: mutation/perturb\n\
Select: qualification from binding alpha/works-edge\n\n\
The plan targets the reviewed edge.\n";

fn model(criticality: &str, with_verification: bool) -> Model {
    let spec = parse_spec("spec.md", &SPEC.replace("standard", criticality)).unwrap();
    let mut model = Model {
        specs: vec![spec],
        realizes: vec![Site {
            spec: "alpha".into(),
            scenario: "works".into(),
            site: "alpha::works".into(),
            file: "src/alpha.rs".into(),
            lang: "rust".into(),
            source: Some(SourceIdentity {
                area: "core".into(),
                kind: "rust-item".into(),
                address: "alpha::works".into(),
                mount: "code".into(),
            }),
            source_fingerprint: SHA.into(),
        }],
        check_implementations: vec![CheckImplementation {
            check: "alpha/works".into(),
            site: "tests::works".into(),
            file: "tests/works.rs".into(),
            lang: "rust".into(),
            source: Some(SourceIdentity {
                area: "core".into(),
                kind: "rust-item".into(),
                address: "tests::works".into(),
                mount: "tests".into(),
            }),
            source_fingerprint: SHA.into(),
        }],
        qualification_policies: Some(parse_policies("standards.md", POLICIES).unwrap()),
        verifications: with_verification
            .then(|| parse_verification("verification.md", VERIFICATION).unwrap())
            .into_iter()
            .collect(),
        ..Default::default()
    };
    if with_verification {
        let expected = model
            .expected_qualification_fingerprint(&model.verifications[0].bindings[0])
            .unwrap();
        model.verifications[0].qualifications[0].fingerprint = expected;
    }
    model
}

fn kinds(model: &Model) -> Vec<FindingKind> {
    validate(model)
        .into_iter()
        .map(|finding| finding.kind)
        .collect()
}

#[test]
fn complete_non_routine_graph_is_clean_and_routine_without_verification_is_valid() {
    assert!(validate(&model("standard", true)).is_empty());
    let mut routine = model("routine", false);
    routine.realizes.clear();
    routine.check_implementations.clear();
    assert!(validate(&routine).is_empty());
}

#[test]
fn reports_binding_check_and_qualification_failures() {
    let mut value = model("standard", false);
    assert!(kinds(&value).contains(&FindingKind::UnboundClaim));

    value = model("standard", true);
    value.verifications[0].bindings.clear();
    value.verifications[0].qualifications.clear();
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::CheckWithoutBinding));

    value = model("standard", true);
    value.verifications[0].bindings[0].check = "missing/check".into();
    assert!(kinds(&value).contains(&FindingKind::BindingMissingCheck));

    value = model("standard", true);
    value.verifications[0].bindings[0].claim = "alpha#missing".into();
    assert!(kinds(&value).contains(&FindingKind::BindingMissingClaim));

    value = model("standard", true);
    value.verifications[0].bindings[0].qualification_policy = "missing".into();
    assert!(kinds(&value).contains(&FindingKind::BindingMissingPolicy));

    value = model("standard", true);
    value.verifications[0].qualifications.clear();
    assert!(kinds(&value).contains(&FindingKind::MissingQualification));

    value = model("standard", true);
    value.verifications[0].bindings.clear();
    assert!(kinds(&value).contains(&FindingKind::DanglingQualification));

    value = model("standard", true);
    value.verifications[0].qualifications[0].verdict = QualificationVerdict::Rejected;
    assert!(kinds(&value).contains(&FindingKind::RejectedQualification));

    value = model("standard", true);
    value.verifications[0].bindings[0]
        .proposition
        .push_str(" changed");
    assert!(kinds(&value).contains(&FindingKind::StaleQualification));
}

#[test]
fn resolves_relationships_across_verification_authorities() {
    let mut value = model("standard", true);
    let declaration = value.verifications.remove(0);
    value.verifications = vec![
        Verification {
            owner: "checks".into(),
            path: "checks/verification.md".into(),
            checks: declaration.checks,
            bindings: Vec::new(),
            qualifications: Vec::new(),
            challengers: Vec::new(),
            challenge_plans: Vec::new(),
        },
        Verification {
            owner: "bindings".into(),
            path: "bindings/verification.md".into(),
            checks: Vec::new(),
            bindings: declaration.bindings,
            qualifications: Vec::new(),
            challengers: Vec::new(),
            challenge_plans: Vec::new(),
        },
        Verification {
            owner: "decisions".into(),
            path: "decisions/verification.md".into(),
            checks: Vec::new(),
            bindings: Vec::new(),
            qualifications: declaration.qualifications,
            challengers: declaration.challengers,
            challenge_plans: declaration.challenge_plans,
        },
    ];

    assert!(validate(&value).is_empty());
}

#[test]
fn duplicate_authorities_are_derivation_errors_not_findings() {
    let mut value = model("standard", true);
    let duplicate = value.verifications[0].checks[0].clone();
    value.verifications.push(Verification {
        owner: "other".into(),
        path: "other/verification.md".into(),
        checks: vec![duplicate],
        bindings: Vec::new(),
        qualifications: Vec::new(),
        challengers: Vec::new(),
        challenge_plans: Vec::new(),
    });

    assert!(!value.verification_declaration_issues().is_empty());
    assert!(validate(&value).is_empty());
}

#[test]
fn reports_implementation_and_routine_applicability_failures() {
    let mut value = model("standard", true);
    value.check_implementations.clear();
    assert!(kinds(&value).contains(&FindingKind::UnimplementedCheck));

    value = model("standard", true);
    value.check_implementations[0].check = "missing/check".into();
    assert!(kinds(&value).contains(&FindingKind::DanglingCheckImplementation));

    value = model("standard", true);
    value.check_implementations[0].source = None;
    assert!(kinds(&value).contains(&FindingKind::UnstableCheckImplementation));

    value = model("standard", true);
    value.check_implementations[0].source_fingerprint = "not-a-fingerprint".into();
    assert!(kinds(&value).contains(&FindingKind::UnstableCheckImplementation));

    value = model("routine", true);
    assert!(kinds(&value).contains(&FindingKind::InapplicableVerification));
}

#[test]
fn routine_bindings_suppress_qualification_cascades_but_plans_still_resolve_zero() {
    let expected = [
        FindingKind::InapplicableVerification,
        FindingKind::UnresolvedChallengePlan,
        FindingKind::UnresolvedChallengeSelector,
    ];
    let mut value = model("routine", true);
    value.check_implementations.clear();
    value.verifications[0].qualifications.clear();
    assert_eq!(kinds(&value), expected);

    value = model("routine", true);
    value.verifications[0].qualifications[0].verdict = QualificationVerdict::Rejected;
    assert_eq!(kinds(&value), expected);

    value = model("routine", true);
    value.verifications[0].bindings[0]
        .proposition
        .push_str(" changed");
    assert_eq!(kinds(&value), expected);

    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(resolution.qualifications.is_empty());
    assert_eq!(
        resolution.unresolved_selectors,
        ["qualification from binding alpha/works-edge"]
    );
}

#[test]
fn a_check_shared_with_an_applicable_binding_still_requires_an_implementation() {
    let mut value = model("standard", true);
    value.specs.push(
        parse_spec(
            "routine.md",
            "# Spec: routine\n\n## Requirement: behavior\nCriticality: routine\n\n\
             The system SHALL work.\n\n### Scenario: works\nWHEN invoked\nTHEN it works\n",
        )
        .unwrap(),
    );
    let mut routine_binding = value.verifications[0].bindings[0].clone();
    routine_binding.id = "alpha/routine-edge".into();
    routine_binding.claim = "routine#works".into();
    value.verifications[0].bindings.push(routine_binding);
    value.check_implementations.clear();

    let found = kinds(&value);
    assert!(found.contains(&FindingKind::InapplicableVerification));
    assert!(found.contains(&FindingKind::UnimplementedCheck));
    assert!(!found.contains(&FindingKind::MissingQualification));
}

#[test]
fn reports_missing_challenger_and_zero_resolution() {
    let mut value = model("standard", true);
    value.verifications[0].challenge_plans[0].challenger = "missing/challenger".into();
    assert!(kinds(&value).contains(&FindingKind::MissingChallenger));

    value = model("standard", true);
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::ClaimJudgmentFromClaim("alpha#works".into()),
        Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
        Selector::ClaimJudgmentFromMechanism("alpha#guard".into()),
    ];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(resolution.qualifications.is_empty());
    assert_eq!(resolution.unresolved_selectors.len(), 3);
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::UnresolvedChallengeSelector));
    assert!(found.contains(&FindingKind::UnresolvedChallengePlan));
}

#[test]
fn challenge_resolution_unions_sorts_and_deduplicates_exact_qualifications() {
    let mut value = model("standard", true);
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::QualificationFromCheck("alpha/works".into()),
        Selector::QualificationFromBinding("alpha/works-edge".into()),
        Selector::QualificationFromRealization("core|rust-item|alpha::works".into()),
    ];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.qualifications.len(), 1);
    assert!(resolution.unresolved_selectors.is_empty());

    value.verifications[0].bindings[0].challenge_domain = vec![ChallengeDomain::Mechanism];
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::QualificationFromRealization(
            "core|rust-item|alpha::works".into(),
        )];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(resolution.qualifications.is_empty());
    assert_eq!(resolution.unresolved_selectors.len(), 1);
}

#[test]
fn many_to_many_bindings_resolve_as_independent_qualifications() {
    let mut value = model("standard", true);
    let second = parse_spec(
        "second.md",
        "# Spec: second\n\n## Requirement: behavior\nCriticality: standard\n\n\
         The system SHALL work.\n\n### Scenario: also-works\nWHEN invoked again\n\
         THEN it also works\n",
    )
    .unwrap();
    value.specs[0].requirements[0]
        .scenarios
        .push(second.requirements[0].scenarios[0].clone());
    let mut second_site = value.realizes[0].clone();
    second_site.scenario = "also-works".into();
    second_site.site = "alpha::also_works".into();
    second_site.source.as_mut().unwrap().address = "alpha::also_works".into();
    value.realizes.push(second_site);

    let mut second_claim_binding = value.verifications[0].bindings[0].clone();
    second_claim_binding.id = "alpha/also-works-edge".into();
    second_claim_binding.claim = "alpha#also-works".into();
    let mut second_claim_qualification = value.verifications[0].qualifications[0].clone();
    second_claim_qualification.id = second_claim_binding.id.clone();

    let mut second_check = value.verifications[0].checks[0].clone();
    second_check.id = "alpha/alternate".into();
    let mut second_check_binding = value.verifications[0].bindings[0].clone();
    second_check_binding.id = "alpha/alternate-edge".into();
    second_check_binding.check = second_check.id.clone();
    let mut second_check_qualification = value.verifications[0].qualifications[0].clone();
    second_check_qualification.id = second_check_binding.id.clone();
    let mut second_implementation = value.check_implementations[0].clone();
    second_implementation.check = second_check.id.clone();
    second_implementation.site = "tests::alternate".into();
    second_implementation.source.as_mut().unwrap().address = "tests::alternate".into();

    value.verifications[0].checks.push(second_check);
    value.verifications[0]
        .bindings
        .extend([second_claim_binding, second_check_binding]);
    value.verifications[0]
        .qualifications
        .extend([second_claim_qualification, second_check_qualification]);
    value.check_implementations.push(second_implementation);
    for index in 0..value.verifications[0].bindings.len() {
        let expected = value
            .expected_qualification_fingerprint(&value.verifications[0].bindings[index])
            .unwrap();
        value.verifications[0].qualifications[index].fingerprint = expected;
    }

    assert!(validate(&value).is_empty());
    value.verifications[0].challenge_plans[0].selectors = vec![
        Selector::QualificationFromCheck("alpha/works".into()),
        Selector::QualificationFromBinding("alpha/alternate-edge".into()),
        Selector::QualificationFromBinding("alpha/works-edge".into()),
    ];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.qualifications.len(), 3);
    assert!(resolution
        .qualifications
        .windows(2)
        .all(|pair| pair[0].fingerprint < pair[1].fingerprint));
}

#[test]
fn mechanism_traversal_requires_the_binding_domain() {
    let mut value = model("standard", true);
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
        source: None,
    });
    value.verifications[0].challenge_plans[0].selectors =
        vec![Selector::QualificationFromMechanism("alpha#guard".into())];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert_eq!(resolution.qualifications.len(), 1);

    value.verifications[0].bindings[0].challenge_domain = vec![ChallengeDomain::Realization];
    let resolution = resolve_challenge_plan(&value, &value.verifications[0].challenge_plans[0]);
    assert!(resolution.qualifications.is_empty());
}

#[test]
fn finding_registry_is_exhaustive_and_has_guidance() {
    assert_eq!(FindingKind::ALL.len(), 30);
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
        FindingKind::MissingQualification.category().name(),
        "judgment"
    );
    assert_eq!(
        FindingKind::UnresolvedChallengeSelector.category().name(),
        "verification"
    );
}
