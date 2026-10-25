//! Realization mechanism and enforcement tests. Synthetic fixtures only.

use azimuth::model::{MechanismImplementation, Model};
use azimuth::realization::{parse_realization, Enforcement, Target};

const REALIZATION: &str = "# Realization: alpha\n\n\
## Requirement: matters\n\
Mechanism: concurrent-insert-constraint\n\
Enforcement: constraint\n\
Binding: schema:index:ux_alpha\n\
Expect: unique=true\n\
Expect: columns=account_id,request_id\n\
Expect: predicate=active\n\n\
The storage constraint makes duplicate insertion unrepresentable.\n";

fn error(source: &str) -> String {
    parse_realization("realization.md", source)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn parses_implementation_binding_and_enforcement_expectations() {
    let realization = parse_realization("realization.md", REALIZATION).unwrap();
    let entry = &realization.entries[0];
    assert_eq!(entry.target, Target::Requirement("matters".into()));
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
    let unknown = REALIZATION.replace("Enforcement: constraint", "Enforcement: vibes");
    assert!(error(&unknown).contains("unknown enforcement `vibes`"));

    let duplicate = format!(
        "{REALIZATION}\n## Requirement: other\nMechanism: concurrent-insert-constraint\n\
         Enforcement: guard\nBinding: rust:alpha::guard\n\nA reason.\n"
    );
    assert!(error(&duplicate).contains("declared twice"));
}

#[test]
fn a_requirement_may_carry_several_mechanisms() {
    let source = "# Realization: alpha\n\n\
## Requirement: matters\n\
Mechanism: transition-writer\n\
Enforcement: choke-point\n\
Binding: rust:alpha::transition\n\
Mechanism: current-state-constraint\n\
Enforcement: constraint\n\
Binding: schema:index:current-state\n\n\
The choke point alone does not survive concurrency.\n";
    let realization = parse_realization("realization.md", source).unwrap();
    let mechanisms = &realization.entries[0].mechanisms;
    assert_eq!(mechanisms.len(), 2);
    assert_eq!(mechanisms[0].kind, Enforcement::ChokePoint);
    assert_eq!(mechanisms[1].kind, Enforcement::Constraint);
}

#[test]
fn every_mechanism_needs_an_enforcement() {
    let source = REALIZATION.replace("Enforcement: constraint\n", "");
    assert!(error(&source).contains("has no enforcement"));
}

#[test]
fn a_binding_must_follow_enforcement() {
    let source = REALIZATION.replace(
        "Enforcement: constraint\nBinding: schema:index:ux_alpha",
        "Binding: schema:index:ux_alpha\nEnforcement: constraint",
    );
    assert!(error(&source).contains("with no enforcement"));
}

#[test]
fn an_entry_needs_review_rationale() {
    let source = REALIZATION.replace(
        "\nThe storage constraint makes duplicate insertion unrepresentable.\n",
        "\n",
    );
    assert!(error(&source).contains("gives no reason"));
}

#[test]
fn residue_is_not_parsed_as_a_realization_entry() {
    let source = format!(
        "{REALIZATION}\n## Residue\n\nEnforcement: this is prose.\nBinding: this is also prose.\n"
    );
    let realization = parse_realization("realization.md", &source).unwrap();
    assert_eq!(realization.entries.len(), 1);
    assert!(realization.residue.contains("this is prose"));
}

#[test]
fn strength_is_never_written_in_a_realization_entry() {
    let source = REALIZATION.replace(
        "Mechanism: concurrent-insert-constraint",
        "Strength: proof\nMechanism: concurrent-insert-constraint",
    );
    assert!(error(&source).contains("unrecognized line"));
}

#[test]
fn a_code_mechanism_may_derive_its_binding_from_an_implementation_marker() {
    let source = REALIZATION.replace("Binding: schema:index:ux_alpha\n", "");
    let model = Model {
        realizations: vec![parse_realization("realization.md", &source).unwrap()],
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
    let mechanism = &model.realizations[0].entries[0].mechanisms[0];
    assert_eq!(
        model.mechanism_bindings("alpha", mechanism),
        ["rust-symbol:alpha::insert"]
    );
}

#[test]
fn local_marker_relocation_preserves_mechanism_identity_and_changes_only_locators() {
    const A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let root = std::env::temp_dir().join(format!(
        "azimuth-marker-relocation-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let model_dir = root.join("model");
    std::fs::create_dir_all(model_dir.join("alpha")).unwrap();
    std::fs::write(
        model_dir.join("alpha/spec.md"),
        "# Spec: alpha\n\n## Requirement: behavior\nRealization: reviewed\n\n\
         The system SHALL work.\n\n### Scenario: works\nEvent: invoked\nRequired: it works\n",
    )
    .unwrap();
    std::fs::write(
        model_dir.join("alpha/realization.md"),
        "# Realization: alpha\n\n## Requirement: behavior\nMechanism: guard\nEnforcement: guard\nScenarios: [\"works\"]\n\n\
         The qualified implementation establishes the guard.\n",
    )
    .unwrap();
    let workspace = root.join("workspace.json");
    std::fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":2,\
         \"areas\":[{\"id\":\"core\",\"mounts\":[{\"id\":\"code\",\"path\":\"src\"}]}],\
         \"surfaces\":[],\"realization_obligations\":[]}",
    )
    .unwrap();
    let manifest = root.join("manifest.json");
    let load_at = |file: &str| {
        std::fs::write(
            &manifest,
            format!(
                "{{\"realizes\":[{{\"requirement\":\"behavior\",\
                 \"site\":\"alpha::works\",\"file\":\"src/alpha.rs\",\"lang\":\"rust\",\
                 \"source_fingerprint\":\"{A}\"}}],\
                 \"mechanism_implementations\":[{{\"mechanism\":\"guard\",\
                 \"site\":\"alpha::Guard::apply\",\
                 \"binding\":\"rust-symbol:alpha::Guard::apply\",\"file\":\"{file}\",\
                 \"lang\":\"rust\",\"source_fingerprint\":\"{B}\"}}],\
                 \"artifacts\":[{{\"id\":\"rust-symbol:alpha::Guard::apply\",\
                 \"kind\":\"rust-symbol\",\"file\":\"{file}\",\"unique\":false,\
                 \"columns\":[\"key\"],\"predicate\":\"active\"}}]}}"
            ),
        )
        .unwrap();
        azimuth::load(&model_dir, &workspace, &[manifest.clone()], &[])
            .unwrap()
            .model
    };
    let before = load_at("src/guard.rs");
    let after = load_at("src/moved/guard.rs");
    let implementation_before = &before.mechanism_implementations[0];
    let implementation_after = &after.mechanism_implementations[0];
    assert_eq!(
        implementation_before.binding,
        "core|rust-symbol|alpha::Guard::apply"
    );
    assert_eq!(implementation_before.binding, implementation_after.binding);
    assert_eq!(
        implementation_before.source_fingerprint,
        implementation_after.source_fingerprint
    );
    assert_ne!(implementation_before.file, implementation_after.file);
    assert_eq!(before.artifacts[0].id, implementation_before.binding);
    assert_eq!(before.artifacts[0].columns, ["key"]);
    assert_eq!(before.artifacts[0].unique, Some(false));
    assert_ne!(
        azimuth::fingerprint::model_digest(&before, &[]),
        azimuth::fingerprint::model_digest(&after, &[])
    );
    for forbidden in [
        "rust-symbol:alpha::Guard::apply",
        "core|rust-symbol|alpha::Guard::apply",
    ] {
        std::fs::write(
            model_dir.join("alpha/realization.md"),
            format!(
                "# Realization: alpha\n\n## Requirement: behavior\nMechanism: explicit-guard\n\
                 Enforcement: guard\nBinding: {forbidden}\n\n\
                 A marker companion cannot cross into the explicit route.\n"
            ),
        )
        .unwrap();
        let errors = azimuth::load(&model_dir, &workspace, &[manifest.clone()], &[]).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.message.contains("may not bind marker-only Artifact")));
    }
    std::fs::remove_dir_all(root).unwrap();
}
