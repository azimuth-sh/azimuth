use azimuth::assurance::{
    ClaimContract, ContractArea, ContractContribution, ContractMount, ContractStep,
    ContractSurface, ContractVerification, ProjectSnapshot, FORMAT, VERSION,
};

fn contract(spec: &str, claim: &str) -> ClaimContract {
    ClaimContract {
        spec: spec.into(),
        claim: claim.into(),
        requirement: "behavior".into(),
        criticality: "standard".into(),
        statement: "The system SHALL work.".into(),
        steps: vec![ContractStep {
            kind: "then".into(),
            text: "it works".into(),
        }],
        domain: "behaviour".into(),
        verification: ContractVerification {
            strength: Some("demonstration".into()),
            scope: "component".into(),
            quantification: Some("example".into()),
            oracle: Some("direct".into()),
            residual_required: false,
            residual: None,
            residual_acceptance: None,
        },
        surface: Some(ContractSurface {
            id: "surface".into(),
            contributions: vec![
                ContractContribution {
                    area: "zeta".into(),
                    mount: "code".into(),
                    path: "zeta".into(),
                    enumerator: "routes".into(),
                },
                ContractContribution {
                    area: "alpha".into(),
                    mount: "code".into(),
                    path: "alpha".into(),
                    enumerator: "routes".into(),
                },
            ],
        }),
        obligated_areas: vec![
            ContractArea {
                id: "zeta".into(),
                mounts: vec![
                    ContractMount {
                        id: "tests".into(),
                        path: "zeta/tests".into(),
                    },
                    ContractMount {
                        id: "code".into(),
                        path: "zeta/src".into(),
                    },
                ],
            },
            ContractArea {
                id: "alpha".into(),
                mounts: vec![ContractMount {
                    id: "code".into(),
                    path: "alpha/src".into(),
                }],
            },
        ],
    }
}

#[test]
fn alpha_one_wire_types_keep_deterministic_contract_identity() {
    let left = contract("zeta", "works");
    let mut reordered = left.clone();
    reordered.surface.as_mut().unwrap().contributions.reverse();
    reordered.obligated_areas.reverse();
    for area in &mut reordered.obligated_areas {
        area.mounts.reverse();
    }

    assert_eq!(left.identity(), "zeta#works");
    assert_eq!(left.fingerprint(), reordered.fingerprint());
}

#[test]
fn snapshot_wire_shape_and_fingerprint_remain_service_compatible() {
    let first = contract("zeta", "works");
    let second = contract("alpha", "works");
    let left = ProjectSnapshot::derive(
        "project",
        "model-fingerprint",
        vec![first.clone(), second.clone()],
    );
    let right = ProjectSnapshot::derive("project", "model-fingerprint", vec![second, first]);

    assert_eq!(left, right);
    assert_eq!(left.id, left.fingerprint());
    assert_eq!(left.claims[0].identity(), "alpha#works");
    assert_eq!(FORMAT, "azimuth-assurance-project-snapshot");
    assert_eq!(VERSION, 1);

    let json = left.to_json();
    assert_eq!(
        json.get("version").and_then(|value| value.as_num()),
        Some(1.0)
    );
    assert!(json.get("modelFingerprint").is_some());
    assert!(json.get("claims").is_some());
    assert!(json.get("bindings").is_none());
    let rendered = json.to_string_pretty();
    assert!(rendered.contains("contractFingerprint"));
    assert!(!rendered.contains("method_qualification_fingerprint"));
}

#[test]
fn semantic_wire_changes_alter_contract_and_snapshot_fingerprints() {
    let original = contract("alpha", "works");
    let mut changed = original.clone();
    changed.statement.push_str(" Always.");
    assert_ne!(original.fingerprint(), changed.fingerprint());

    let left = ProjectSnapshot::derive("project", "model-a", vec![original]);
    let right = ProjectSnapshot::derive("project", "model-b", vec![changed]);
    assert_ne!(left.id, right.id);
}
