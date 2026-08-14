use azimuth::assurance::{contracts, snapshot};
use azimuth::model::Model;
use azimuth::{plan, spec, workspace};

fn model(workspace_source: &str) -> Model {
    Model {
        specs: vec![spec::parse_spec(
            "model/trips/view/spec.md",
            "# Spec: trips/view\n\n## Invariant: live-only\nCriticality: critical\nOver: trips/view\n\nOnly live trips are exposed.\n",
        )
        .unwrap()],
        standards: Some(
            plan::parse_standards(
                "standards.md",
                "# Verification standards\nDefault scope: unit\n\n## Level: critical\nStrength: demonstration\nQuantification: universal\nResidual: required\n\n## Level: standard\nStrength: demonstration\nQuantification: example\nResidual: optional\n\n## Level: routine\nStrength: none\nResidual: optional\n",
            )
            .unwrap(),
        ),
        workspace: workspace::parse(
            "workspace.json",
            &azimuth::json::parse(workspace_source).unwrap(),
        )
        .unwrap(),
        ..Default::default()
    }
}

const WORKSPACE: &str = r#"{
  "format":"azimuth-workspace","version":1,
  "areas":[
    {"id":"web","mounts":[{"id":"code","path":"web"}]},
    {"id":"backend","mounts":[{"id":"code","path":"backend"}]}
  ],
  "surfaces":[{"id":"trips/view","contributions":[
    {"area":"web","mount":"code","enumerator":"next-routes"}
  ]}],
  "realization_obligations":[{
    "spec":"trips/view","claim":"live-only","areas":["backend","web"]
  }]
}"#;

#[test]
fn contract_carries_surface_areas_and_effective_verification() {
    let contract = contracts(&model(WORKSPACE)).pop().unwrap();

    assert_eq!(contract.identity(), "trips/view#live-only");
    assert_eq!(contract.verification.scope, "unit");
    assert_eq!(
        contract.verification.quantification.as_deref(),
        Some("universal")
    );
    assert!(contract.verification.residual_required);
    assert_eq!(
        contract.surface.as_ref().map(|item| item.id.as_str()),
        Some("trips/view")
    );
    assert_eq!(
        contract
            .obligated_areas
            .iter()
            .map(|area| area.id.as_str())
            .collect::<Vec<_>>(),
        vec!["backend", "web"]
    );
}

#[test]
fn set_reordering_does_not_change_the_contract() {
    let reordered = WORKSPACE
        .replace(
            r#"{"id":"web","mounts":[{"id":"code","path":"web"}]},
    {"id":"backend","mounts":[{"id":"code","path":"backend"}]}"#,
            r#"{"id":"backend","mounts":[{"id":"code","path":"backend"}]},
    {"id":"web","mounts":[{"id":"code","path":"web"}]}"#,
        )
        .replace(r#"["backend","web"]"#, r#"["web","backend"]"#);
    let left = model(WORKSPACE);
    let right = model(&reordered);

    assert_eq!(
        contracts(&left)[0].fingerprint(),
        contracts(&right)[0].fingerprint()
    );
}

#[test]
fn architectural_or_verification_drift_changes_the_contract() {
    let original = model(WORKSPACE);
    let changed_workspace = model(&WORKSPACE.replace(r#"["backend","web"]"#, r#"["backend"]"#));
    let mut changed_verification = model(WORKSPACE);
    changed_verification
        .standards
        .as_mut()
        .unwrap()
        .default_scope = azimuth::model::Scope::E2e;

    assert_ne!(
        contracts(&original)[0].fingerprint(),
        contracts(&changed_workspace)[0].fingerprint()
    );
    assert_ne!(
        contracts(&original)[0].fingerprint(),
        contracts(&changed_verification)[0].fingerprint()
    );
}

#[test]
fn realization_source_changes_only_the_exact_snapshot() {
    let mut before = model(WORKSPACE);
    let mut after = model(WORKSPACE);
    before.realizes.push(site("before"));
    after.realizes.push(site("after"));

    assert_eq!(
        contracts(&before)[0].fingerprint(),
        contracts(&after)[0].fingerprint()
    );
    assert_ne!(
        snapshot(&before, &[], "rides").id,
        snapshot(&after, &[], "rides").id
    );
}

fn site(source_fingerprint: &str) -> azimuth::model::Site {
    azimuth::model::Site {
        spec: "trips/view".into(),
        scenario: "live-only".into(),
        site: "View".into(),
        file: "backend/View.cs".into(),
        lang: "csharp".into(),
        source: None,
        source_fingerprint: source_fingerprint.into(),
        evidence_kind: None,
        evidence_outcome: None,
        observed_at: None,
        expires_at: None,
        scope: None,
        quantification: None,
        oracle: None,
    }
}
