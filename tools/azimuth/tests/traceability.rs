use azimuth::account_verification::parse_account_verification;
use azimuth::model::{CheckImplementation, Model, Rigor, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::traceability::project;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const VERIFICATION: &str = "# Verification: alpha\n\n\
## Requirement verification: works\n\n\
### Scenario verification: observed\n\n\
#### Check: zeta-check\n\
- Evidence bindings:\n  \
- Contribution: Observes the late path.\n\n\
Invoke the late path.\n\n\
#### Check: alpha-check\n\
- Evidence bindings:\n  \
- Contribution: Observes the early path.\n\n\
Invoke the early path.\n";

fn source(address: &str) -> SourceIdentity {
    SourceIdentity {
        area: "core".into(),
        kind: "rust-item".into(),
        address: address.into(),
        mount: "code".into(),
    }
}

fn site(address: &str) -> Site {
    Site {
        spec: "alpha".into(),
        requirement: "works".into(),
        site: address.into(),
        file: format!("{address}.rs"),
        lang: "rust".into(),
        source: Some(source(address)),
        source_fingerprint: SHA.into(),
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

fn spec(labels: &str) -> azimuth::model::Spec {
    parse_spec(
        "spec.md",
        &format!(
            "# Spec: alpha\n\n## Requirement: works\n{labels}\nA SHALL work.\n\n\
             ### Scenario: observed\nEvent: invoked\nRequired: it works\n\n\
             ### Scenario: unobserved\nEvent: skipped\nRequired: nothing happens\n"
        ),
    )
    .unwrap()
}

fn model() -> Model {
    Model {
        specs: vec![spec("Realization: reviewed\nVerification: reviewed\n")],
        realizes: vec![site("z"), site("a")],
        account_verifications: vec![
            parse_account_verification("verification.md", VERIFICATION).unwrap()
        ],
        check_implementations: vec![
            check_implementation("alpha-check", "alpha"),
            check_implementation("zeta-check", "zeta"),
        ],
        ..Default::default()
    }
}

#[test]
fn projects_sorted_realization_and_verification_relationships_per_scenario() {
    let report = project(&model());
    assert_eq!(
        report
            .scenarios
            .iter()
            .map(|scenario| scenario.id.as_str())
            .collect::<Vec<_>>(),
        ["observed", "unobserved"]
    );
    let observed = &report.scenarios[0];
    assert_eq!(observed.parent_requirement, "works");
    assert_eq!(
        observed.realizations,
        ["core|rust-item|a", "core|rust-item|z"]
    );
    assert_eq!(
        observed
            .verification
            .iter()
            .map(|relationship| (
                relationship.check.as_str(),
                relationship.contribution.as_str()
            ))
            .collect::<Vec<_>>(),
        [
            ("alpha-check", "Observes the early path."),
            ("zeta-check", "Observes the late path.")
        ]
    );
    let unobserved = &report.scenarios[1];
    assert_eq!(unobserved.realizations, observed.realizations);
    assert!(unobserved.verification.is_empty());
}

#[test]
fn rigor_labels_are_projected_and_absent_labels_are_null() {
    let report = project(&model());
    assert_eq!(report.scenarios[0].realization_rigor, Some(Rigor::Reviewed));
    assert_eq!(
        report.scenarios[0].verification_rigor,
        Some(Rigor::Reviewed)
    );

    let routine = Model {
        specs: vec![spec("")],
        realizes: vec![site("a")],
        ..Default::default()
    };
    let report = project(&routine);
    assert_eq!(report.scenarios[0].realization_rigor, None);
    assert_eq!(report.scenarios[0].verification_rigor, None);
    assert!(report.scenarios[0].verification.is_empty());
    let json = report.to_json().to_string_pretty();
    assert!(json.contains("\"realization_rigor\": null"), "{json}");
    assert!(json.contains("\"verification_rigor\": null"), "{json}");
}

#[test]
fn report_is_deterministic_and_creates_no_execution_or_authority_fields() {
    let left = project(&model()).to_json().to_string_pretty();
    let right = project(&model()).to_json().to_string_pretty();
    assert_eq!(left, right);
    assert!(left.contains("\"version\": 6"));
    for absent in [
        "observations",
        "challenge_results",
        "assurance_state",
        "rationale",
        "tests/",
        "judgment",
        "decision_impacts",
        "challenge_resolutions",
        "criticality",
    ] {
        assert!(!left.contains(absent), "{absent}: {left}");
    }
}
