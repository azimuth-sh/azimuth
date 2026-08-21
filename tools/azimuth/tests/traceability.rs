use azimuth::model::{CheckImplementation, Criticality, Model, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::traceability::project;
use azimuth::verification::{parse_policies, parse_verification};

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn model() -> Model {
    let spec = parse_spec(
        "spec.md",
        "# Spec: alpha\n\n## Requirement: works\nCriticality: standard\n\nA SHALL work.\n\n\
         ### Scenario: observed\nWHEN invoked\nTHEN it works\n",
    )
    .unwrap();
    let verification = parse_verification(
        "verification.md",
        "# Verification: alpha\n\n## Check: z/check\nMethod: invoke\nTerminal: works\n\nAtomic.\n\n\
         ## Check: a/check\nMethod: invoke\nTerminal: works\n\nAtomic.\n\n\
         ## Evidence Binding: z/edge\nCheck: z/check\nClaim: alpha#observed\n\
         Proposition: direct\nScope: unit\nQuantification: example\nOracle: direct\nContext: {}\n\
         Challenge domain: [\"context\"]\nQualification policy: credible\n\nReviewable.\n\n\
         ## Evidence Binding: a/edge\nCheck: a/check\nClaim: alpha#observed\nProposition: direct\n\
         Scope: unit\nQuantification: example\nOracle: direct\nContext: {}\n\
         Challenge domain: [\"context\"]\nQualification policy: credible\n\nReviewable.\n\n\
         ## Qualification: z/edge\nVerdict: rejected\n\
         Fingerprint: sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n\
         Qualified: 2026-08-21\nQualifier: owner\n\nRejected.\n\n\
         ## Qualification: a/edge\nVerdict: qualified\n\
         Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
         Qualified: 2026-08-21\nQualifier: owner\n\nQualified.\n",
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
                source: Some(SourceIdentity {
                    area: "core".into(),
                    kind: "rust-item".into(),
                    address: "z".into(),
                    mount: "code".into(),
                }),
                source_fingerprint: String::new(),
            },
            Site {
                spec: "alpha".into(),
                scenario: "observed".into(),
                site: "a".into(),
                file: "a.rs".into(),
                lang: "rust".into(),
                source: Some(SourceIdentity {
                    area: "core".into(),
                    kind: "rust-item".into(),
                    address: "a".into(),
                    mount: "code".into(),
                }),
                source_fingerprint: String::new(),
            },
        ],
        qualification_policies: Some(
            parse_policies(
                "standards.md",
                "# Qualification policies\n\n## Policy: credible\nRequired challenge: mutation\n\n\
                 A reason.\n",
            )
            .unwrap(),
        ),
        verifications: vec![verification],
        check_implementations: vec![
            check_implementation("a/check", "a"),
            check_implementation("z/check", "z"),
        ],
        ..Default::default()
    };
    for index in 0..model.verifications[0].bindings.len() {
        let expected = model
            .expected_qualification_fingerprint(&model.verifications[0].bindings[index])
            .unwrap();
        model.verifications[0].qualifications[index].fingerprint = expected;
    }
    model
}

fn check_implementation(check: &str, address: &str) -> CheckImplementation {
    CheckImplementation {
        check: check.into(),
        site: format!("tests::{address}"),
        file: format!("tests/{address}.rs"),
        lang: "rust".into(),
        source: Some(SourceIdentity {
            area: "core".into(),
            kind: "rust-item".into(),
            address: format!("tests::{address}"),
            mount: "tests".into(),
        }),
        source_fingerprint: SHA.into(),
    }
}

#[test]
fn projects_sorted_realization_and_verification_relationships() {
    let report = project(&model());
    let claim = &report.claims[0];
    assert_eq!(claim.id, "alpha#observed");
    assert_eq!(claim.realizations, ["core|rust-item|a", "core|rust-item|z"]);
    assert_eq!(claim.verification[0].binding, "a/edge");
    assert_eq!(claim.verification[0].check, "a/check");
    assert!(claim.verification[0].applicable);
    assert!(claim.verification[0].current);
    assert_eq!(claim.verification[0].verdict.as_deref(), Some("qualified"));
    assert_eq!(claim.verification[1].binding, "z/edge");
    assert!(claim.verification[1].current);
    assert_eq!(claim.verification[1].verdict.as_deref(), Some("rejected"));
}

#[test]
fn report_is_deterministic_and_creates_no_execution_or_authority_fields() {
    let left = project(&model()).to_json().to_string_pretty();
    let right = project(&model()).to_json().to_string_pretty();
    assert_eq!(left, right);
    assert!(left.contains("\"version\": 2"));
    assert!(!left.contains("observations"));
    assert!(!left.contains("covers"));
    assert!(!left.contains("methods"));
    assert!(!left.contains("terminal"));
    assert!(!left.contains("context"));
    assert!(!left.contains("rationale"));
    assert!(!left.contains("proposition"));
    assert!(!left.contains("qualification_policy"));
    assert!(!left.contains("challenge_plans"));
    assert!(!left.contains("tests/"));
}

#[test]
fn missing_qualification_remains_an_explicit_empty_relationship() {
    let mut value = model();
    value.verifications[0].qualifications.clear();
    let report = project(&value);
    assert!(report.claims[0]
        .verification
        .iter()
        .all(|relationship| relationship.qualification.is_none()));
    assert!(report.claims[0]
        .verification
        .iter()
        .all(|relationship| !relationship.current));
}

#[test]
fn stale_qualification_is_never_presented_as_current() {
    let mut value = model();
    value.verifications[0].bindings[0]
        .proposition
        .push_str(" changed");

    let report = project(&value);
    let relationship = report.claims[0]
        .verification
        .iter()
        .find(|relationship| relationship.binding == "z/edge")
        .unwrap();
    assert!(relationship.applicable);
    assert!(!relationship.current);
    assert!(relationship.qualification.is_none());
    assert!(relationship.verdict.is_none());
}

#[test]
fn routine_verification_relationships_are_inapplicable_and_never_current() {
    let mut value = model();
    value.specs[0].requirements[0].criticality = Some(Criticality::Routine);

    let report = project(&value);
    assert!(report.claims[0].verification.iter().all(|relationship| {
        !relationship.applicable
            && !relationship.current
            && relationship.qualification.is_none()
            && relationship.verdict.is_none()
    }));
}

#[test]
fn realizations_without_stable_source_identity_are_omitted() {
    let mut value = model();
    value.realizes[0].source = None;

    let report = project(&value);
    assert_eq!(report.claims[0].realizations, ["core|rust-item|a"]);
    assert!(!report.to_json().to_string_pretty().contains("z.rs"));
}
