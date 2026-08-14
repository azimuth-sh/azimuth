use azimuth_assurance_domain::{
    ClaimContract, ContractStep, ContractVerification, ProjectModelSnapshot,
    PROJECT_SNAPSHOT_FORMAT, PROJECT_SNAPSHOT_VERSION,
};

fn main() {
    let mut contract = ClaimContract {
        contract_fingerprint: String::new(),
        spec: "checkout/performance".into(),
        claim: "latency-objective".into(),
        requirement: "performance".into(),
        criticality: "standard".into(),
        statement: "Checkout latency remains bounded.".into(),
        steps: vec![ContractStep {
            kind: "then".into(),
            text: "p95 latency stays below the qualified threshold".into(),
        }],
        domain: "behaviour".into(),
        verification: ContractVerification {
            strength: Some("demonstration".into()),
            scope: "e2e".into(),
            quantification: Some("example".into()),
            oracle: Some("direct".into()),
            residual_required: false,
            residual: None,
            residual_acceptance: None,
        },
        surface: None,
        obligated_areas: vec![],
    };
    contract.contract_fingerprint = contract.fingerprint();
    let mut snapshot = ProjectModelSnapshot {
        format: PROJECT_SNAPSHOT_FORMAT.into(),
        version: PROJECT_SNAPSHOT_VERSION,
        id: String::new(),
        project: "checkout".into(),
        model_fingerprint: "synthetic-checkout-model-v1".into(),
        claims: vec![contract],
    };
    snapshot.id = snapshot.fingerprint();
    println!("{}", serde_json::to_string(&snapshot).unwrap());
}
