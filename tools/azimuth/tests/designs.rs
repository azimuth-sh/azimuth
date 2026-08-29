//! Design mechanism and enforcement tests. Synthetic fixtures only.

use azimuth::design::{parse_design, Enforcement, Target};
use azimuth::model::{MechanismImplementation, Model};

const DESIGN: &str = "# Design: alpha\n\n\
## Claim: matters\n\
Mechanism: concurrent-insert-constraint\n\
Enforcement: constraint\n\
Binding: schema:index:ux_alpha\n\
Expect: unique=true\n\
Expect: columns=account_id,request_id\n\
Expect: predicate=active\n\n\
The storage constraint makes duplicate insertion unrepresentable.\n";

fn error(source: &str) -> String {
    parse_design("design.md", source)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn parses_implementation_binding_and_enforcement_expectations() {
    let design = parse_design("design.md", DESIGN).unwrap();
    let entry = &design.entries[0];
    assert_eq!(entry.target, Target::Claim("matters".into()));
    let mechanism = &entry.mechanisms[0];
    assert_eq!(mechanism.id, "concurrent-insert-constraint");
    assert_eq!(mechanism.kind, Enforcement::Constraint);
    assert_eq!(mechanism.binding.as_deref(), Some("schema:index:ux_alpha"));
    assert_eq!(mechanism.expected_unique, Some(true));
    assert_eq!(mechanism.expected_columns, ["account_id", "request_id"]);
    assert_eq!(mechanism.expected_predicate.as_deref(), Some("active"));
}

#[test]
fn enforcement_ladder_remains_stable() {
    assert_eq!(Enforcement::Type.rung(), 1);
    assert_eq!(Enforcement::Schema.rung(), 1);
    assert_eq!(Enforcement::Constraint.rung(), 2);
    assert_eq!(Enforcement::ChokePoint.rung(), 2);
    assert_eq!(Enforcement::Middleware.rung(), 3);
    assert_eq!(Enforcement::Guard.rung(), 4);
    assert!(Enforcement::Constraint.is_proof_capable());
    assert!(!Enforcement::Guard.is_proof_capable());
}

#[test]
fn rejects_unknown_enforcement_and_duplicate_mechanism_ids() {
    let unknown = DESIGN.replace("Enforcement: constraint", "Enforcement: vibes");
    assert!(error(&unknown).contains("unknown enforcement `vibes`"));

    let duplicate = format!(
        "{DESIGN}\n## Claim: other\nMechanism: concurrent-insert-constraint\n\
         Enforcement: guard\nBinding: rust:alpha::guard\n\nA reason.\n"
    );
    assert!(error(&duplicate).contains("declared twice"));
}

#[test]
fn a_requirement_may_carry_several_mechanisms() {
    let source = "# Design: alpha\n\n\
## Claim: matters\n\
Mechanism: transition-writer\n\
Enforcement: choke-point\n\
Binding: rust:alpha::transition\n\
Mechanism: current-state-constraint\n\
Enforcement: constraint\n\
Binding: schema:index:current-state\n\n\
The choke point alone does not survive concurrency.\n";
    let design = parse_design("design.md", source).unwrap();
    let mechanisms = &design.entries[0].mechanisms;
    assert_eq!(mechanisms.len(), 2);
    assert_eq!(mechanisms[0].kind, Enforcement::ChokePoint);
    assert_eq!(mechanisms[1].kind, Enforcement::Constraint);
}

#[test]
fn every_mechanism_needs_an_enforcement() {
    let source = DESIGN.replace("Enforcement: constraint\n", "");
    assert!(error(&source).contains("has no enforcement"));
}

#[test]
fn a_binding_must_follow_enforcement() {
    let source = DESIGN.replace(
        "Enforcement: constraint\nBinding: schema:index:ux_alpha",
        "Binding: schema:index:ux_alpha\nEnforcement: constraint",
    );
    assert!(error(&source).contains("with no enforcement"));
}

#[test]
fn an_entry_needs_review_rationale() {
    let source = DESIGN.replace(
        "\nThe storage constraint makes duplicate insertion unrepresentable.\n",
        "\n",
    );
    assert!(error(&source).contains("gives no reason"));
}

#[test]
fn residue_is_not_parsed_as_a_design_entry() {
    let source = format!(
        "{DESIGN}\n## Residue\n\nEnforcement: this is prose.\nBinding: this is also prose.\n"
    );
    let design = parse_design("design.md", &source).unwrap();
    assert_eq!(design.entries.len(), 1);
    assert!(design.residue.contains("this is prose"));
}

#[test]
fn strength_is_never_written_in_a_design_entry() {
    let source = DESIGN.replace(
        "Mechanism: concurrent-insert-constraint",
        "Strength: proof\nMechanism: concurrent-insert-constraint",
    );
    assert!(error(&source).contains("unrecognized line"));
}

#[test]
fn a_code_mechanism_may_derive_its_binding_from_an_implementation_marker() {
    let source = DESIGN.replace("Binding: schema:index:ux_alpha\n", "");
    let model = Model {
        designs: vec![parse_design("design.md", &source).unwrap()],
        mechanism_implementations: vec![MechanismImplementation {
            spec: "alpha".into(),
            mechanism: "concurrent-insert-constraint".into(),
            site: "alpha::insert".into(),
            binding: "rust-symbol:alpha::insert".into(),
            file: "src/alpha.rs".into(),
            lang: "rust".into(),
            source: None,
            source_fingerprint: "sha256:source".into(),
        }],
        ..Default::default()
    };
    let mechanism = &model.designs[0].entries[0].mechanisms[0];
    assert_eq!(
        model.mechanism_bindings("alpha", mechanism),
        ["rust-symbol:alpha::insert"]
    );
}
