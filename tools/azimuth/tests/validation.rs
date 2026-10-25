use azimuth::account_verification::parse_account_verification;
use azimuth::model::{CheckImplementation, Model, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::validation::{validate, FindingKind};
use std::collections::BTreeSet;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPEC: &str = "# Spec: alpha\n\n\
## Requirement: behavior\n\
Verification: reviewed\n\n\
The system SHALL work.\n\n\
### Scenario: works\n\
Event: invoked\n\
Required: it works\n";
const VERIFICATION: &str = "# Verification: alpha\n\n\
## Requirement verification: behavior\n\n\
### Scenario verification: works\n\n\
#### Check: alpha-works\n\
- Evidence bindings:\n  \
- Contribution: Observes the behavior working.\n\n\
Invoke the behavior and observe the terminal result.\n";

fn source(address: &str) -> SourceIdentity {
    SourceIdentity {
        area: "core".into(),
        kind: "rust-item".into(),
        address: address.into(),
        mount: "code".into(),
    }
}

fn site(requirement: &str, address: &str) -> Site {
    Site {
        spec: "alpha".into(),
        requirement: requirement.into(),
        site: address.into(),
        file: "src/alpha.rs".into(),
        lang: "rust".into(),
        source: Some(source(address)),
        source_fingerprint: SHA.into(),
    }
}

fn model() -> Model {
    Model {
        specs: vec![parse_spec("spec.md", SPEC).unwrap()],
        account_verifications: vec![
            parse_account_verification("verification.md", VERIFICATION).unwrap()
        ],
        realizes: vec![site("behavior", "alpha::works")],
        check_implementations: vec![CheckImplementation {
            check: "alpha-works".into(),
            site: "tests::works".into(),
            file: "tests/works.rs".into(),
            lang: "rust".into(),
            source: Some(source("tests::works")),
            source_fingerprint: SHA.into(),
        }],
        ..Default::default()
    }
}

fn routine_model() -> Model {
    Model {
        specs: vec![parse_spec("spec.md", &SPEC.replace("Verification: reviewed\n", "")).unwrap()],
        realizes: vec![site("behavior", "alpha::works")],
        ..Default::default()
    }
}

fn kinds(model: &Model) -> Vec<FindingKind> {
    validate(model)
        .into_iter()
        .map(|finding| finding.kind)
        .collect()
}

#[test]
fn complete_reviewed_graph_awaits_only_its_review_and_routine_without_verification_is_valid() {
    assert_eq!(
        kinds(&model()),
        vec![FindingKind::MissingRequirementAssurance]
    );
    assert!(kinds(&routine_model()).is_empty());
}

#[test]
fn reviewed_realization_without_a_site_is_unrealized_and_awaits_its_review() {
    let mut value = Model {
        specs: vec![parse_spec(
            "spec.md",
            &SPEC.replace("Verification: reviewed", "Realization: reviewed"),
        )
        .unwrap()],
        ..Default::default()
    };
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::Unrealized), "{found:?}");
    assert!(
        found.contains(&FindingKind::MissingRealizationReview),
        "{found:?}"
    );
    value.realizes.push(site("behavior", "alpha::works"));
    let found = kinds(&value);
    assert!(!found.contains(&FindingKind::Unrealized), "{found:?}");
    assert!(
        found.contains(&FindingKind::MissingRealizationReview),
        "{found:?}"
    );
}

#[test]
fn check_without_binding_and_unimplemented_check_are_findings() {
    let mut value = model();
    value.account_verifications[0].checks[0].bindings.clear();
    assert!(kinds(&value).contains(&FindingKind::CheckWithoutBinding));
    let mut value = model();
    value.check_implementations.clear();
    assert!(kinds(&value).contains(&FindingKind::UnimplementedCheck));
}

#[test]
fn scenario_of_a_verified_requirement_without_a_binding_is_unbound() {
    let mut value = model();
    value.account_verifications[0].checks[0].bindings.clear();
    let found = kinds(&value);
    assert!(found.contains(&FindingKind::UnboundScenario), "{found:?}");
}

#[test]
fn check_implementation_marker_for_an_undeclared_check_dangles() {
    let mut value = model();
    value.check_implementations.push(CheckImplementation {
        check: "missing".into(),
        site: "tests::missing".into(),
        file: "tests/missing.rs".into(),
        lang: "rust".into(),
        source: Some(source("tests::missing")),
        source_fingerprint: SHA.into(),
    });
    assert!(kinds(&value).contains(&FindingKind::DanglingCheckImplementation));
}

#[test]
fn dangling_realization_names_the_site_and_its_absent_requirement() {
    let mut value = routine_model();
    value.realizes.push(site("missing", "alpha::dangling"));
    let findings = validate(&value);
    let dangling: Vec<_> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::DanglingRealization)
        .collect();
    assert_eq!(dangling.len(), 1, "{findings:?}");
    assert!(dangling[0].detail.contains("alpha::dangling"));
    assert_eq!(dangling[0].requirement.as_deref(), Some("missing"));
}

#[test]
fn finding_registry_is_exhaustive_and_has_guidance() {
    assert_eq!(FindingKind::ALL.len(), 29);
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
        FindingKind::MissingRequirementAssurance.category().name(),
        "assurance"
    );
    assert_eq!(
        FindingKind::UnboundScenario.category().name(),
        "verification"
    );
}

#[test]
fn finding_records_name_the_model_entity_kind() {
    let finding = azimuth::validation::Finding {
        kind: FindingKind::UnboundScenario,
        severity: azimuth::validation::Severity::Error,
        requirement: Some("visible".into()),
        entity_kind: Some(azimuth::diag::EntityKind::Scenario),
        path: "spec.md".into(),
        line: 1,
        detail: "unbound".into(),
    };
    let json = finding.to_json();
    assert_eq!(
        json.get("entity_kind").and_then(|value| value.as_str()),
        Some("scenario")
    );
    assert!(json.get("verification_target_kind").is_none());
    assert!(json.get("subject_kind").is_none());
}
