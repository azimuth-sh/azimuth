use azimuth::account_verification::parse_account_verification;
use azimuth::adapter::*;
use azimuth::model::{CheckImplementation, Model, SourceIdentity};
use azimuth::run_selection::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
fn fp() -> String {
    format!("sha256:{}", "a".repeat(64))
}
fn model() -> Model {
    let document="# Verification: catalog\n\n## Claim verification: admission\n\n### Case verification: rejected\n\n#### Check: alpha\n- Execution:\n  - Subjects: artifact, ci-candidate, workspace\n  - Family: application\n  - Requirements: database\n- Evidence bindings:\n  - Contribution: Rejects invalid requests.\n\nExercise requests.\n\n#### Check: beta\n- Execution:\n  - Subjects: artifact, workspace\n  - Family: application\n- Evidence bindings:\n  - Contribution: Examines registrations.\n\nInspect registrations.\n";
    let account = parse_account_verification("verification.md", document).unwrap();
    let spec=azimuth::spec::parse_spec("spec.md","# Spec: catalog\n\n## Claim: admission\nCriticality: standard\n\nThe API SHALL reject invalid callers.\n\n### Case: rejected\nGiven invalid authority, when invoked, then deny.\n").unwrap();
    Model {
        account_verifications: vec![account],
        specs: vec![spec],
        check_implementations: ["alpha", "beta"]
            .iter()
            .map(|id| CheckImplementation {
                check: (*id).into(),
                site: format!("Catalog::{id}"),
                file: "checks.rs".into(),
                lang: "rust-symbol".into(),
                source: Some(SourceIdentity {
                    area: "api".into(),
                    kind: "rust-symbol".into(),
                    address: format!("Catalog::{id}"),
                    mount: "checks".into(),
                }),
                source_fingerprint: fp(),
            })
            .collect(),
        ..Default::default()
    }
}
fn configuration() -> AdapterConfiguration {
    AdapterConfiguration {
        path: PathBuf::from("adapters.json"),
        directory: PathBuf::from("."),
        adapters: vec![ConfiguredAdapter {
            id: "native".into(),
            provider_family: "synthetic".into(),
            protocol_version: 1,
            adapter_version: "1".into(),
            build: "test".into(),
            content: AdapterContent {
                executable: ConfiguredFile {
                    locator: "adapter".into(),
                    resolved: PathBuf::from("adapter"),
                    digest: fp(),
                },
                resources: vec![],
            },
            semantic_settings: BTreeMap::new(),
            environment: AdapterEnvironment {
                literals: BTreeMap::new(),
            },
            limits: AdapterLimits {
                timeout_ms: 1000,
                stdout_bytes: 10000,
                stderr_bytes: 10000,
            },
            capabilities: vec![Capability {
                id: "checks".into(),
                classes: vec![CapabilityClass::CheckExecute],
                challenge_forms: vec![],
                semantic_settings: BTreeMap::new(),
                fingerprint: fp(),
            }],
            adapter_fingerprint: fp(),
            descriptor_fingerprint: fp(),
            configuration_fingerprint: fp(),
        }],
    }
}
fn policy() -> SelectionPolicy {
    parse_policy("policy.json",&format!(r#"{{"format":"azimuth-run-selection-policy","version":1,"operation":"execute","planned_at_ms":10,"subject":{{"kind":"artifact","artifacts":[{{"id":"candidate","digest":"{}"}}]}},"required_context":{{}},"families":["application"],"available_requirements":["database"],"routes":[{{"family":"application","execution-capability":"native/checks"}}]}}"#,fp())).unwrap()
}
#[test]
fn selects_all_eligible_checks_without_an_id_list_and_derives_cases_and_units() {
    let m = model();
    let c = configuration();
    let report = select(&m, &c, &policy());
    assert!(report.findings.is_empty());
    let request = report.request.as_ref().unwrap();
    assert_eq!(request.checks.len(), 2);
    assert!(request.checks.iter().all(|c| c.cases.is_empty()));
    let launch = azimuth::run_plan::plan(&m, &c, request).unwrap();
    assert!(launch
        .plan
        .checks
        .iter()
        .all(|c| c.cases == vec!["rejected"] && c.units[0].id == "whole"));
    let source = azimuth::run::canonical_json(&to_json(&report)).unwrap();
    assert_eq!(
        request_from_selection(&m, &c, "selection.json", &source).unwrap(),
        *request
    );
}
#[test]
fn missing_routes_and_requirements_are_explicit_findings() {
    let mut p = policy();
    p.routes.clear();
    let r = select(&model(), &configuration(), &p);
    assert_eq!(r.findings.len(), 2);
    assert!(r.request.is_none());
    let mut p = policy();
    p.available_requirements.clear();
    let r = select(&model(), &configuration(), &p);
    assert!(r
        .findings
        .iter()
        .any(|f| f.check == "alpha" && f.code == "missing-execution-requirements"));
    assert_eq!(r.request.unwrap().checks.len(), 1);
}
#[test]
fn undeclared_execution_and_incompatible_subjects_are_not_silently_excluded() {
    let mut m = model();
    m.account_verifications[0].checks[0].inputs.clear();
    let r = select(&m, &configuration(), &policy());
    assert!(r.findings.iter().any(|f| f.code == "undeclared-execution"));
    let mut p = policy();
    p.subject = azimuth::run::Subject::Service {
        environment: "hosted".into(),
        service: "api".into(),
        deployment: "deployment-a".into(),
        deployment_fingerprint: fp(),
    };
    let r = select(&model(), &configuration(), &p);
    assert_eq!(r.findings.len(), 2);
    assert!(r.request.is_none());
}
#[test]
fn stale_or_edited_selection_reports_cannot_launch() {
    let m = model();
    let c = configuration();
    let report = select(&m, &c, &policy());
    let source = azimuth::run::canonical_json(&to_json(&report)).unwrap();
    let mut changed = model();
    changed.account_verifications[0].checks[0]
        .definition
        .methods
        .push("different oracle".into());
    assert!(request_from_selection(&changed, &c, "selection.json", &source).is_err());
    let edited = source.replace("native/checks", "native/other");
    assert!(request_from_selection(&m, &c, "selection.json", &edited).is_err());
    let mut incompatible = policy();
    incompatible.routes.clear();
    let source = azimuth::run::canonical_json(&to_json(&select(&m, &c, &incompatible))).unwrap();
    assert!(request_from_selection(&m, &c, "selection.json", &source).is_err());
}
#[test]
fn outside_policy_families_are_visible_and_do_not_claim_execution() {
    let mut p = policy();
    p.families = vec!["hosted".into()];
    p.routes.clear();
    let r = select(&model(), &configuration(), &p);
    assert!(r.findings.is_empty());
    assert!(r.request.is_none());
    assert!(r.coverage.iter().all(|e| e.status == "outside-policy"));
}
#[test]
fn execution_maps_and_policy_routes_fail_closed_on_ambiguity() {
    let m = model();
    assert_eq!(
        execution_from_inputs(&m.account_verifications[0].checks[0].inputs)
            .unwrap()
            .unwrap()
            .requirements,
        vec!["database"]
    );
    let doc="# Verification: catalog\n\n## Claim verification: admission\n\n### Case verification: rejected\n\n#### Check: alpha\n- Execution:\n  - Subjects: nonexistent\n  - Family: application\n- Evidence bindings:\n  - Contribution: Examines request.\n\nRun.\n";
    assert!(parse_account_verification("verification.md", doc).is_err());
    let json = azimuth::run::canonical_json(&policy_to_json(&policy())).unwrap();
    let duplicate = json.replace(
        "\"routes\":[{",
        "\"routes\":[{\"family\":\"application\",\"execution-capability\":\"native/checks\"},{",
    );
    assert!(parse_policy("policy.json", &duplicate).is_err());
    assert!(parse_policy(
        "policy.json",
        &json.replace("execution-capability", "capability")
    )
    .is_err());
}
