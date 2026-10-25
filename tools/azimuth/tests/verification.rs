use azimuth::account_verification::parse_account_verification;
use azimuth::fingerprint::{
    canonical_json, canonical_sha256, check_fingerprint, execution_context_fingerprint,
    verification_challenge_fingerprint,
};
use azimuth::json;
use azimuth::model::{CheckImplementation, Model, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::verification::{
    execution_context_json, Check, SemanticScopeKind, VerificationChallenge,
};
use std::collections::BTreeMap;

const SPEC: &str = "# Spec: payments/recovery\n\n\
## Requirement: recover\n\
Verification: reviewed\n\n\
The system SHALL replay accepted writes.\n\n\
### Scenario: accepted-write-replayed\n\
Event: the broker recovers\n\
Required: the accepted write is replayed exactly once\n";

fn verification(
    owner_path: &str,
    check: &str,
) -> azimuth::account_verification::AccountVerification {
    parse_account_verification(
        owner_path,
        &format!(
            "# Verification: payments/recovery\n\n## Requirement verification: recover\n\n\
             ### Scenario verification: accepted-write-replayed\n\n#### Check: {check}\n\
             - Evidence bindings:\n  - Contribution: Replay after injected loss exercises recovery.\n\n\
             Inject broker loss and observe replay after recovery.\n"
        ),
    )
    .unwrap()
}

#[test]
fn execution_context_fingerprint_uses_a_versioned_canonical_envelope() {
    let context = BTreeMap::from([
        ("storage".to_string(), "postgres-17".to_string()),
        ("platform".to_string(), "linux-x86_64".to_string()),
    ]);
    let digest = execution_context_fingerprint(&context);

    assert_eq!(
        canonical_json(&execution_context_json(&context)),
        concat!(
            "{\n",
            "  \"format\": \"azimuth-execution-context-fingerprint\",\n",
            "  \"required_execution_context\": {\n",
            "    \"platform\": \"linux-x86_64\",\n",
            "    \"storage\": \"postgres-17\"\n",
            "  },\n",
            "  \"version\": 2\n",
            "}\n",
        )
    );
    assert_eq!(
        digest,
        "sha256:4e876db5e0d830400e9771c879b132d9423aa52afd1d3ca70a41e7db03939813"
    );
    assert_ne!(
        digest,
        canonical_sha256(&json::Json::Obj(
            context
                .iter()
                .map(|(key, value)| (key.clone(), json::Json::str(value)))
                .collect(),
        ))
    );
    let version_three = json::Json::obj(vec![
        (
            "format",
            json::Json::str("azimuth-execution-context-fingerprint"),
        ),
        ("version", json::Json::Num(3.0)),
        (
            "required_execution_context",
            json::Json::Obj(
                context
                    .iter()
                    .map(|(key, value)| (key.clone(), json::Json::str(value)))
                    .collect(),
            ),
        ),
    ]);
    assert_ne!(digest, canonical_sha256(&version_three));
}

#[test]
fn verification_challenge_matches_the_frozen_vector_and_ignores_prose_and_location() {
    let challenge = VerificationChallenge {
        targets: Vec::new(),
        requirements: Vec::new(),
        mechanisms: Vec::new(),
        checks: Vec::new(),
        id: "mutation/implementation-perturbation".into(),
        form: "implementation-perturbation".into(),
        searches_for: "an implementation change that leaves the bound Check satisfied".into(),
        required_scope: vec![
            SemanticScopeKind::Realization,
            SemanticScopeKind::CheckImplementation,
        ],
        rationale: "Surviving changes object to credibility.".into(),
        path: "verification.md".into(),
        line: 1,
    };
    let digest = verification_challenge_fingerprint(&challenge);
    assert_eq!(
        digest,
        "sha256:f5af895b2e4261f5bad8deca851f96c1c8f675651b8933791fd6eba18839c0a1"
    );
    let mut relocated = challenge.clone();
    relocated.rationale = "Other prose.".into();
    relocated.path = "elsewhere/verification.md".into();
    relocated.line = 40;
    relocated.required_scope.reverse();
    assert_eq!(digest, verification_challenge_fingerprint(&relocated));
    let mut retargeted = challenge;
    retargeted.targets = vec!["implementation".into()];
    assert_ne!(digest, verification_challenge_fingerprint(&retargeted));
}

#[test]
fn check_fingerprint_ignores_locators_and_follows_semantic_identity() {
    let check = Check {
        id: "recovery-under-loss".into(),
        methods: vec![
            "inject broker loss".into(),
            "observe replay after recovery".into(),
        ],
        terminal: "the accepted write is replayed exactly once".into(),
        rationale: "The methods describe one atomic terminal result.".into(),
        path: "verification.md".into(),
        line: 3,
    };
    let implementation = CheckImplementation {
        check: check.id.clone(),
        site: "ignored locator".into(),
        file: "ignored/path.rs".into(),
        lang: "rust".into(),
        source: Some(SourceIdentity {
            area: "payments".into(),
            kind: "rust-item".into(),
            address: "recovery::replay".into(),
            mount: "ignored-mount".into(),
        }),
        source_fingerprint: "sha256:source".into(),
    };
    let digest = check_fingerprint(&check, std::slice::from_ref(&implementation));
    let mut moved = implementation.clone();
    moved.file = "moved/path.rs".into();
    moved.source.as_mut().unwrap().mount = "new-mount".into();
    assert_eq!(digest, check_fingerprint(&check, &[moved]));
    let mut readdressed = implementation.clone();
    readdressed.source.as_mut().unwrap().address = "recovery::other".into();
    assert_ne!(digest, check_fingerprint(&check, &[readdressed]));
    let mut changed = implementation;
    changed.source_fingerprint = "sha256:changed".into();
    assert_ne!(digest, check_fingerprint(&check, &[changed]));
}

#[test]
fn model_enforces_project_check_cardinality_and_exports_only_current_root_keys() {
    let mut model = Model {
        specs: vec![parse_spec("spec.md", SPEC).unwrap()],
        account_verifications: vec![
            verification("verification.md", "recovery-under-loss"),
            verification("other/verification.md", "recovery-under-loss"),
        ],
        ..Default::default()
    };
    assert!(model
        .account_check_issues()
        .iter()
        .any(|issue| issue.message.contains("already has a definition")));

    model.account_verifications.pop();
    assert!(model.account_check_issues().is_empty());
    let exported = model.to_json(&[]);
    let json::Json::Obj(fields) = &exported else {
        panic!("export root is an object");
    };
    assert_eq!(
        fields
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        [
            "version",
            "extensions",
            "specs",
            "realizes",
            "workspace",
            "mechanism_implementations",
            "account_checks",
            "check_implementations",
            "class_members",
            "enumerations",
            "artifacts",
            "mechanisms",
            "checks",
            "verification_challenges",
            "findings",
        ]
    );
    assert_eq!(
        exported.get("version").and_then(|value| value.as_num()),
        Some(8.0)
    );
    let requirement = &exported.get("specs").unwrap().as_array().unwrap()[0]
        .get("requirements")
        .unwrap()
        .as_array()
        .unwrap()[0];
    assert_eq!(
        requirement.get("verification").and_then(json::Json::as_str),
        Some("reviewed")
    );
    assert_eq!(requirement.get("realization"), Some(&json::Json::Null));
    assert!(requirement.get("criticality").is_none());
    assert_eq!(
        exported
            .get("checks")
            .and_then(json::Json::as_array)
            .map(<[json::Json]>::len),
        Some(1)
    );
}
