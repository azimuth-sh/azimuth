use azimuth::{account_verification::parse_account_verification, assurance_review, model::Model};

fn document(policy: &str) -> String {
    format!("# Verification: sample\n\n## Requirement verification: restricted\n\n### Check: admission\n- Deployment: {policy}\n- Evidence bindings:\n  - Scenario: denied\n    - Contribution: Exercises denied access.\n\nExercise denied access.\n")
}

#[test]
fn deployment_policy_is_structured_review_authority() {
    let policy = r#"{"verification_target":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}"#;
    let account = parse_account_verification("verification.md", &document(policy)).unwrap();
    assert!(account.checks[0].inputs.contains_key("deployment-policy"));
    let authority = assurance_review::authority(&Model { specs: vec![azimuth::spec::parse_spec("spec.md", "# Spec: sample\n\n## Requirement: restricted\n\nAccess SHALL be restricted.\n\n### Scenario: denied\nEvent: denied request\nRequired: denied access\n").unwrap()], account_verifications: vec![account], ..Default::default() }, "sample").unwrap();
    let deployment = authority.get("checks").unwrap().as_array().unwrap()[0]
        .get("deployment")
        .unwrap();
    assert_eq!(
        deployment.get("verification_target").unwrap().as_str(),
        Some("source")
    );
}

#[test]
fn invalid_or_duplicate_deployment_policy_is_rejected() {
    let invalid = r#"{"verification_target":"source","components":["api"],"contexts":[],"contribution":"both","monitor":{"id":"exposure","interval_seconds":30,"freshness_seconds":60}}"#;
    assert!(parse_account_verification("verification.md", &document(invalid)).is_err());
    let valid = r#"{"verification_target":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}"#;
    let duplicate = document(valid).replace(
        "- Evidence bindings:",
        &format!("- Deployment: {valid}\n- Evidence bindings:"),
    );
    assert!(parse_account_verification("verification.md", &duplicate).is_err());
}
