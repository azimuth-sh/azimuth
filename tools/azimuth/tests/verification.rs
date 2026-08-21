use azimuth::fingerprint::{
    binding_fingerprint, canonical_json, canonical_sha256, check_fingerprint, context_fingerprint,
    policy_fingerprint, qualification_fingerprint,
};
use azimuth::json;
use azimuth::model::{CheckImplementation, Model, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::verification::{
    context_json, parse_policies, parse_selector, parse_verification, ChallengeDomain, Selector,
};
use std::collections::BTreeMap;

const DECLARATIONS: &str = r#"# Verification: payments/recovery

## Check: payments/recovery-under-loss
Method: inject broker loss
Method: observe replay after recovery
Terminal: the accepted write is replayed exactly once

The methods describe one atomic terminal result.

## Evidence Binding: payments/recovery-edge
Check: payments/recovery-under-loss
Claim: payments/recovery#accepted-write-replayed
Proposition: replay exercises the recovery predicate
Scope: component
Quantification: example
Oracle: relational
Context: {"storage":"postgres-17","platform":"linux-x86_64"}
Challenge domain: ["oracle","realization","mechanism","context","check-implementation"]
Qualification policy: credible-executable

The edge is narrower than the whole suite.

## Qualification: payments/recovery-edge
Verdict: qualified
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Qualified: 2026-08-21
Qualifier: evidence-owner@example

The implementation and oracle make the edge credible.

## Challenger: mutation/implementation-perturbation
Form: implementation-perturbation
Searches for: a change that leaves the Check satisfied

Surviving changes object to credibility.

## Challenge Plan: payments/recovery-credibility
Challenger: mutation/implementation-perturbation
Select: qualification from binding payments/recovery-edge
Select: qualification from check payments/recovery-under-loss
Select: qualification from realization payments|rust-item|recovery::replay
Select: qualification from mechanism payments/recovery#transactional-outbox
Select: claim-judgment from claim payments/recovery#accepted-write-replayed
Select: claim-judgment from realization payments|rust-item|recovery::replay
Select: claim-judgment from mechanism payments/recovery#transactional-outbox

The Run will freeze the exact selected fingerprints.
"#;

const POLICIES: &str = "# Qualification policies\n\n\
## Policy: credible-executable\n\
Required challenge: oracle-perturbation\n\
Required challenge: implementation-perturbation\n\n\
Both objections must be searched for.\n";

const SPEC: &str = "# Spec: payments/recovery\n\n\
## Requirement: recover\n\
Criticality: standard\n\n\
The system SHALL replay accepted writes.\n\n\
### Scenario: accepted-write-replayed\n\
WHEN the broker recovers\n\
THEN the accepted write is replayed exactly once\n";

#[test]
fn parses_the_complete_strict_declaration_graph() {
    let declarations = parse_verification("verification.md", DECLARATIONS).unwrap();
    assert_eq!(declarations.owner, "payments/recovery");
    assert_eq!(declarations.checks[0].methods.len(), 2);
    assert_eq!(
        declarations.bindings[0].context.keys().collect::<Vec<_>>(),
        ["platform", "storage"]
    );
    assert_eq!(
        declarations.bindings[0].challenge_domain,
        [
            ChallengeDomain::Realization,
            ChallengeDomain::Mechanism,
            ChallengeDomain::CheckImplementation,
            ChallengeDomain::Oracle,
            ChallengeDomain::Context,
        ]
    );
    assert_eq!(declarations.challenge_plans[0].selectors.len(), 7);
    assert!(matches!(
        declarations.challenge_plans[0].selectors[6],
        Selector::ClaimJudgmentFromMechanism(_)
    ));
}

#[test]
fn rejects_retired_headings_and_non_exact_context() {
    let retired =
        "# Verification: alpha\n\n## Claim: retired\nStrength: demonstration\n\nRetired.\n";
    let errors = parse_verification("verification.md", retired).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("unrecognized heading")));

    let invalid = DECLARATIONS.replace(
        "{\"storage\":\"postgres-17\",\"platform\":\"linux-x86_64\"}",
        "{\"storage\":17}",
    );
    let errors = parse_verification("verification.md", &invalid).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("not a string")));
}

#[test]
fn rejects_retired_or_unknown_label_like_lines_without_rejecting_wrapped_values() {
    let retired = DECLARATIONS.replace(
        "Method: inject broker loss",
        "Method: inject broker loss\nStrength: demonstration",
    );
    let errors = parse_verification("verification.md", &retired).unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("unrecognized label-like line `Strength: demonstration`")
    }));

    let unknown = DECLARATIONS.replace(
        "Qualifier: evidence-owner@example",
        "Qualifier: evidence-owner@example\nDetector field: retired-value",
    );
    let errors = parse_verification("verification.md", &unknown).unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("unrecognized label-like line `Detector field: retired-value`")
    }));

    let lower = DECLARATIONS.replace(
        "Method: inject broker loss",
        "Method: inject broker loss\nstrength: demonstration",
    );
    let errors = parse_verification("verification.md", &lower).unwrap_err();
    assert!(errors.iter().any(|error| error
        .message
        .contains("unrecognized label-like line `strength: demonstration`")));

    let wrapped = DECLARATIONS.replace(
        "Method: inject broker loss",
        "Method: inject broker loss\nafter the accepted write is durable",
    );
    let parsed = parse_verification("verification.md", &wrapped).unwrap();
    assert_eq!(
        parsed.checks[0].methods[0],
        "inject broker loss after the accepted write is durable"
    );
}

#[test]
fn rejects_calendar_impossible_qualification_dates() {
    let invalid = DECLARATIONS.replace("Qualified: 2026-08-21", "Qualified: 2026-02-29");
    let errors = parse_verification("verification.md", &invalid).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("invalid Qualification date")));

    let leap = DECLARATIONS.replace("Qualified: 2026-08-21", "Qualified: 2028-02-29");
    assert!(parse_verification("verification.md", &leap).is_ok());
}

#[test]
fn realization_selectors_reject_locator_and_pattern_shaped_identities() {
    for identity in [
        "payments|file|recovery.rs",
        "payments|path|src/recovery.rs",
        "payments|line|42",
        "payments|rust-item|src/recovery.rs",
        "payments|rust-item|src\\recovery.rs",
        "payments|rust-item|recovery.rs",
        "payments|rust-item|recovery.rs:42",
        "payments|rust-item|recovery.rs:42:7",
        "payments|rust-item|recovery.rs#L42",
        "payments|next-route|src/recovery.rs",
        "payments|next-route|GET recovery",
        "payments|rust.item|recovery::replay",
        "payments|rust-item|recovery::*",
    ] {
        let mut errors = Vec::new();
        let selector = format!("qualification from realization {identity}");
        parse_selector("verification.md", 1, &selector, &mut errors);
        assert!(!errors.is_empty(), "accepted `{identity}`");
    }

    for identity in [
        "payments|rust-symbol|recovery::replay",
        "payments|dotnet-symbol|Payments.Recovery.Replay",
        "payments|jvm-symbol|payments.Recovery#replay",
        "payments|go-symbol|Recovery.Replay",
        "payments|typescript-symbol|Recovery.replay",
        "payments|next-route|GET /payments/[id]",
    ] {
        let mut errors = Vec::new();
        let selector = format!("qualification from realization {identity}");
        let parsed = parse_selector("verification.md", 1, &selector, &mut errors);
        assert!(errors.is_empty(), "rejected `{identity}`: {errors:?}");
        assert_eq!(
            parsed,
            Some(Selector::QualificationFromRealization(identity.to_string()))
        );
    }
}

#[test]
fn context_fingerprint_uses_a_versioned_canonical_envelope() {
    let declarations = parse_verification("verification.md", DECLARATIONS).unwrap();
    let context = &declarations.bindings[0].context;
    let digest = context_fingerprint(&declarations.bindings[0]);

    assert_eq!(
        canonical_json(&context_json(context)),
        concat!(
            "{\n",
            "  \"format\": \"azimuth-context-fingerprint\",\n",
            "  \"required_context\": {\n",
            "    \"platform\": \"linux-x86_64\",\n",
            "    \"storage\": \"postgres-17\"\n",
            "  },\n",
            "  \"version\": 1\n",
            "}\n",
        )
    );
    assert_eq!(
        digest,
        "sha256:69f80f51c5fc8bd40d590d110108cb67f82c634c17c6d40ae1472a68d2fa793d"
    );
    assert_eq!(digest, canonical_sha256(&context_json(context)));
    assert_ne!(
        digest,
        canonical_sha256(&json::Json::Obj(
            context
                .iter()
                .map(|(key, value)| (key.clone(), json::Json::str(value)))
                .collect(),
        ))
    );

    let mut reverse_inserted = BTreeMap::new();
    reverse_inserted.insert("storage".to_string(), "postgres-17".to_string());
    reverse_inserted.insert("platform".to_string(), "linux-x86_64".to_string());
    assert_eq!(
        canonical_sha256(&context_json(context)),
        canonical_sha256(&context_json(&reverse_inserted))
    );

    let version_two = json::Json::obj(vec![
        ("format", json::Json::str("azimuth-context-fingerprint")),
        ("version", json::Json::Num(2.0)),
        (
            "required_context",
            json::Json::Obj(
                context
                    .iter()
                    .map(|(key, value)| (key.clone(), json::Json::str(value)))
                    .collect(),
            ),
        ),
    ]);
    assert_ne!(digest, canonical_sha256(&version_two));
}

#[test]
fn fingerprints_stale_only_for_their_semantic_inputs() {
    let declarations = parse_verification("verification.md", DECLARATIONS).unwrap();
    let policies = parse_policies("standards.md", POLICIES).unwrap();
    let check = &declarations.checks[0];
    let binding = &declarations.bindings[0];
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
    let check_digest = check_fingerprint(check, &[implementation.clone()]);
    let mut moved = implementation;
    moved.file = "moved/path.rs".into();
    moved.source.as_mut().unwrap().mount = "new-mount".into();
    assert_eq!(check_digest, check_fingerprint(check, &[moved]));

    let policy_digest = policy_fingerprint(&policies.policies[0]);
    let binding_digest = binding_fingerprint(binding, "sha256:claim", &policy_digest);
    let mut reordered_domain = binding.clone();
    reordered_domain.challenge_domain.reverse();
    reordered_domain
        .challenge_domain
        .push(ChallengeDomain::Mechanism);
    assert_eq!(
        binding_digest,
        binding_fingerprint(&reordered_domain, "sha256:claim", &policy_digest)
    );
    let context_digest = context_fingerprint(binding);
    let qualification = qualification_fingerprint(&check_digest, &binding_digest, &context_digest);
    let mut context_changed = binding.clone();
    context_changed
        .context
        .insert("platform".into(), "macos".into());
    assert_eq!(
        binding_digest,
        binding_fingerprint(&context_changed, "sha256:claim", &policy_digest)
    );
    assert_ne!(context_digest, context_fingerprint(&context_changed));
    assert_ne!(
        qualification,
        qualification_fingerprint(
            &check_digest,
            &binding_digest,
            &context_fingerprint(&context_changed)
        )
    );
}

#[test]
fn model_enforces_project_cardinality_and_exports_only_version_two_fields() {
    let mut second = parse_verification("other/verification.md", DECLARATIONS).unwrap();
    second.bindings.clear();
    second.qualifications.clear();
    second.challengers.clear();
    second.challenge_plans.clear();
    let mut model = Model {
        specs: vec![parse_spec("spec.md", SPEC).unwrap()],
        qualification_policies: Some(parse_policies("standards.md", POLICIES).unwrap()),
        verifications: vec![
            parse_verification("verification.md", DECLARATIONS).unwrap(),
            second,
        ],
        ..Default::default()
    };
    assert!(model
        .verification_declaration_issues()
        .iter()
        .any(|issue| issue.message.contains("already declared")));

    model.verifications.pop();
    let exported = model.to_json(&[]);
    assert_eq!(
        exported.get("version").and_then(|value| value.as_num()),
        Some(2.0)
    );
    for key in ["covers", "mechanism_covers", "observations"] {
        assert!(exported.get(key).is_none(), "retired export key `{key}`");
    }
    for key in [
        "checks",
        "evidence_bindings",
        "qualifications",
        "challengers",
        "challenge_plans",
        "check_implementations",
    ] {
        assert!(exported.get(key).is_some(), "missing v2 export key `{key}`");
    }
    assert!(json::parse(&exported.to_string_pretty()).is_ok());
}
