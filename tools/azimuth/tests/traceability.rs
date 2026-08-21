//! D44 traceability projection tests.
//!
//! The command-router package owns `lib.rs`, so this test imports the source module directly until
//! that package exposes it. The small re-exports preserve the module's normal `crate` paths.

pub mod json {
    pub use azimuth::json::*;
}

pub mod model {
    pub use azimuth::model::*;
}

#[path = "../src/traceability.rs"]
mod traceability;

use azimuth::json::Json;
use azimuth::manifest;
use azimuth::model::Model;
use azimuth::spec::parse_spec;
use traceability::project;

const SPEC: &str = "\
# Spec: sample

## Requirement: later-parent
Criticality: standard

The system SHALL preserve the later condition.

### Scenario: zeta
WHEN the later condition is exercised
THEN it remains preserved

## Requirement: earlier-parent
Criticality: routine

The system SHALL expose the earlier condition.

### Scenario: alpha
GIVEN an observable precondition
WHEN the earlier condition is exercised
THEN it is exposed
";

fn model_with(manifest_json: &str) -> Model {
    let spec = parse_spec("sample.md", SPEC).expect("spec parses");
    let mut model = Model {
        specs: vec![spec],
        ..Default::default()
    };
    if !manifest_json.is_empty() {
        let root = azimuth::json::parse(manifest_json).expect("manifest json parses");
        let parsed = manifest::parse("manifest.json", &root).expect("manifest parses");
        model.realizes = parsed.realizes;
        model.covers = parsed.covers;
        model.observations = parsed.observations;
    }
    model
}

fn field<'a>(value: &'a Json, key: &str) -> &'a Json {
    value.get(key).unwrap_or_else(|| panic!("missing `{key}`"))
}

#[test]
fn claims_and_realizations_are_ordered_by_stable_identity() {
    let model = model_with(
        r#"{
          "realizes": [
            {"spec":"sample","scenario":"zeta","site":"Legacy.Z","file":"z.rs",
             "lang":"rust"},
            {"spec":"sample","scenario":"alpha","site":"Alpha.B","file":"b.rs",
             "lang":"rust","area":"web","address_kind":"rust.item",
             "address":"sample::beta","mount":"source"},
            {"spec":"sample","scenario":"alpha","site":"Alpha.A","file":"a.rs",
             "lang":"rust","area":"api","address_kind":"rust.item",
             "address":"sample::alpha","mount":"source"},
            {"spec":"sample","scenario":"alpha","site":"Alpha.B","file":"b.rs",
             "lang":"rust","area":"web","address_kind":"rust.item",
             "address":"sample::beta","mount":"source"}
          ]
        }"#,
    );

    let report = project(&model);
    let ids = report
        .claims
        .iter()
        .map(|claim| claim.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["sample#alpha", "sample#zeta"]);
    assert_eq!(
        report.claims[0].realizations,
        vec![
            "api|rust.item|sample::alpha".to_string(),
            "web|rust.item|sample::beta".to_string(),
        ]
    );
    assert_eq!(
        report.claims[1].realizations,
        vec!["z.rs#Legacy.Z|rust".to_string()]
    );
}

#[test]
fn projection_uses_only_claims_present_in_the_selected_model() {
    let mut model = model_with("");
    model.specs[0]
        .requirements
        .retain(|requirement| requirement.id == "earlier-parent");

    let report = project(&model);
    assert_eq!(report.claims.len(), 1);
    assert_eq!(report.claims[0].id, "sample#alpha");
    assert_eq!(report.claims[0].parent_requirement, "earlier-parent");
}

#[test]
fn json_is_a_derived_view_without_authority_or_alpha_one_evidence() {
    let model = model_with(
        r#"{
          "realizes": [
            {"spec":"sample","scenario":"alpha","site":"Alpha.A","file":"a.rs",
             "lang":"rust"}
          ],
          "covers": [
            {"spec":"sample","scenario":"alpha","site":"Checks.Alpha","file":"test.rs",
             "lang":"rust","scope":"unit","quantification":"example","oracle":"direct"}
          ]
        }"#,
    );

    let json = project(&model).to_json();
    let claims = field(&json, "claims").as_array().expect("claims array");
    let alpha = claims
        .iter()
        .find(|claim| field(claim, "id").as_str() == Some("sample#alpha"))
        .expect("alpha claim");

    assert_eq!(
        field(alpha, "parent_requirement").as_str(),
        Some("earlier-parent")
    );
    assert_eq!(field(alpha, "criticality").as_str(), Some("routine"));
    assert_eq!(
        field(alpha, "statement").as_str(),
        Some("The system SHALL expose the earlier condition.")
    );
    assert_eq!(field(alpha, "steps").as_array().unwrap().len(), 3);

    let text = json.to_string_pretty();
    for forbidden in [
        "covers",
        "observations",
        "evidence",
        "path",
        "line",
        "source_fingerprint",
    ] {
        assert!(
            !text.contains(forbidden),
            "unexpected authority `{forbidden}` in {text}"
        );
    }
}

#[test]
fn input_traversal_order_does_not_change_json() {
    let mut first = model_with(
        r#"{"realizes":[
          {"spec":"sample","scenario":"alpha","site":"Alpha.B","file":"b.rs","lang":"rust"},
          {"spec":"sample","scenario":"alpha","site":"Alpha.A","file":"a.rs","lang":"rust"}
        ]}"#,
    );
    let expected = project(&first).to_json().to_string_pretty();

    first.specs[0].requirements.reverse();
    first.realizes.reverse();
    let actual = project(&first).to_json().to_string_pretty();

    assert_eq!(actual, expected);
}
