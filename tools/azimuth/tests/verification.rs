use azimuth::design::{Design, DesignEntry, Enforcement, Mechanism, Target};
use azimuth::fingerprint::{
    binding_fingerprint, canonical_json, canonical_sha256, check_fingerprint,
    execution_context_fingerprint, method_qualification_fingerprint, policy_fingerprint,
    schedule_fingerprint,
};
use azimuth::json;
use azimuth::model::{Artifact, CheckImplementation, Model, Site, SourceIdentity};
use azimuth::spec::parse_spec;
use azimuth::verification::{
    execution_context_json, parse_selector, parse_standards, parse_verification, ChallengeDomain,
    EvidenceBinding, Selector,
};
use std::collections::BTreeMap;

const DECLARATIONS: &str = r#"# Verification: payments/recovery

## Check: recovery-under-loss
Method: inject broker loss
Method: observe replay after recovery
Terminal: the accepted write is replayed exactly once

The methods describe one atomic terminal result.

## Evidence Binding: payments/recovery-edge
Check: recovery-under-loss
Scenario: accepted-write-replayed
Method qualification: payments/recovery-method
Proposition: replay exercises the recovery predicate
Execution Context: {"storage":"postgres-17","platform":"linux-x86_64"}
Challenge domain: ["oracle","realization","mechanism","context","check-implementation"]
Policy: credible-executable

The edge is narrower than the whole suite.

## Method Qualification: payments/recovery-method
Check: recovery-under-loss
Scope: component
Quantification: example
Oracle: relational
Execution Context: {"storage":"postgres-17","platform":"linux-x86_64"}
Challenge domain: ["oracle","context","check-implementation"]
Policy: credible-executable
Verdict: qualified
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Qualified: 2026-08-21
Qualifier: evidence-owner@example

The implementation and oracle make the edge credible.

## Applicability Decision: payments/recovery-edge
Verdict: applicable
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Decided: 2026-08-21
Decider: evidence-owner@example

The qualified method applies to this exact edge.

## Requirement Judgment: recover
Verdict: accepted
Policy: credible-executable
Fingerprint: sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
Judged: 2026-08-21
Judge: assurance-owner@example
Basis: the realized behavior and qualified edge compose the Requirement
Residual risk: none identified

The reviewed total composition is sufficient.

## Verification Challenge: mutation/implementation-perturbation
Form: implementation-perturbation
Searches for: an implementation change that leaves the bound Check satisfied
Required scope: ["check-implementation","realization"]

Surviving changes object to credibility.

## Challenge Plan: payments/recovery-credibility
Verification Challenge: mutation/implementation-perturbation
Select: applicability-decision from binding payments/recovery-edge
Select: method-qualification from check recovery-under-loss
Select: method-qualification from realization payments|rust-item|recovery::replay
Select: method-qualification from mechanism transactional-outbox
Select: requirement-judgment from requirement recover
Select: requirement-judgment from realization payments|rust-item|recovery::replay
Select: requirement-judgment from mechanism transactional-outbox

The Run will freeze the exact selected fingerprints.
"#;

const STANDARDS: &str = "# Decision policies and Challenge schedule\n\n\
## Decision Policy: credible-executable\n\
Required challenge: oracle-perturbation\n\
Required challenge: implementation-perturbation\n\n\
Both objections must be searched for.\n\n\
## Challenge Schedule: current\n\
Gate challenge: implementation-perturbation\n\
Scheduled challenge: oracle-perturbation\n\n\
The expensive objection remains scheduled.\n";

const SPEC: &str = "# Spec: payments/recovery\n\n\
## Requirement: recover\n\
Criticality: standard\n\n\
The system SHALL replay accepted writes.\n\n\
### Scenario: accepted-write-replayed\n\
Event: the broker recovers\n\
Required: the accepted write is replayed exactly once\n";

fn judgment_model() -> Model {
    Model {
        specs: vec![parse_spec("spec.md", SPEC).unwrap()],
        realizes: vec![Site {
            spec: "payments/recovery".into(),
            requirement: "recover".into(),
            site: "recovery::replay".into(),
            file: "src/recovery.rs".into(),
            lang: "rust".into(),
            source: Some(SourceIdentity {
                area: "payments".into(),
                kind: "rust-item".into(),
                address: "recovery::replay".into(),
                mount: "code".into(),
            }),
            source_fingerprint:
                "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        }],
        check_implementations: vec![CheckImplementation {
            check: "recovery-under-loss".into(),
            site: "tests::replay".into(),
            file: "tests/recovery.rs".into(),
            lang: "rust".into(),
            source: Some(SourceIdentity {
                area: "payments".into(),
                kind: "rust-item".into(),
                address: "tests::replay".into(),
                mount: "tests".into(),
            }),
            source_fingerprint:
                "sha256:2222222222222222222222222222222222222222222222222222222222222222".into(),
        }],
        decision_standards: Some(parse_standards("standards.md", STANDARDS).unwrap()),
        verifications: vec![parse_verification("verification.md", DECLARATIONS).unwrap()],
        ..Default::default()
    }
}

#[test]
fn parses_the_complete_strict_declaration_graph() {
    let declarations = parse_verification("verification.md", DECLARATIONS).unwrap();
    assert_eq!(declarations.owner, "payments/recovery");
    assert_eq!(declarations.checks[0].methods.len(), 2);
    assert_eq!(
        declarations.bindings[0]
            .execution_context
            .keys()
            .collect::<Vec<_>>(),
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
    assert_eq!(declarations.requirement_judgments.len(), 1);
    assert_eq!(
        declarations.verification_challenges[0].required_scope.len(),
        2
    );
    assert!(matches!(
        declarations.challenge_plans[0].selectors[6],
        Selector::RequirementJudgmentFromMechanism(_)
    ));
}

#[test]
fn rejects_retired_headings_and_non_exact_context() {
    let retired =
        "# Verification: alpha\n\n## Requirement: retired\nStrength: demonstration\n\nRetired.\n";
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
fn rejects_pre_f_policy_grammar_and_requires_one_current_schedule() {
    let old_binding = DECLARATIONS.replace(
        "Policy: credible-executable",
        "Qualification policy: credible-executable",
    );
    assert!(parse_verification("verification.md", &old_binding).is_err());

    for source in [
        "# Qualification policies\n\n## Policy: credible\nRequired challenge: mutation\n\nOld.\n",
        "# Decision policies and Challenge schedule\n\n## Decision Policy: credible\n\
         Required challenge: mutation\n\nCurrent.\n",
    ] {
        assert!(parse_standards("standards.md", source).is_err());
    }

    let duplicate = format!(
        "{STANDARDS}\n## Challenge Schedule: current\nGate challenge: another\n\nDuplicate.\n"
    );
    assert!(parse_standards("standards.md", &duplicate).is_err());
}

#[test]
fn rejects_invalid_requirement_judgments_and_required_scope() {
    let routine_target = DECLARATIONS.replace(
        "## Requirement Judgment: recover",
        "## Requirement Judgment: not_a_requirement",
    );
    assert!(parse_verification("verification.md", &routine_target).is_err());

    let empty_scope = DECLARATIONS.replace(
        "Required scope: [\"check-implementation\",\"realization\"]",
        "Required scope: []",
    );
    assert!(parse_verification("verification.md", &empty_scope).is_err());

    let old_heading = DECLARATIONS.replace("## Requirement Judgment:", "## Judgment:");
    assert!(parse_verification("verification.md", &old_heading).is_err());
}

#[test]
fn missing_judgments_remain_observable_while_authored_cardinality_is_strict() {
    let mut missing = judgment_model();
    missing.verifications[0].requirement_judgments.clear();
    assert!(missing.verification_declaration_issues().is_empty());
    assert_eq!(missing.requirement_judgments().count(), 0);
    assert_eq!(
        missing
            .requirements()
            .filter(|requirement| matches!(
                requirement.requirement.criticality,
                Some(azimuth::model::Criticality::Standard | azimuth::model::Criticality::Critical)
            ))
            .count(),
        1
    );

    let mut duplicate = judgment_model();
    let mut second = duplicate.verifications[0].requirement_judgments[0].clone();
    second.path = "other/verification.md".into();
    duplicate.verifications[0]
        .requirement_judgments
        .push(second);
    let duplicate_issues = duplicate.verification_declaration_issues();
    let duplicate_messages = duplicate_issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(duplicate_messages.contains("already declared"));

    let mut routine = judgment_model();
    routine.specs[0].requirements[0].criticality = Some(azimuth::model::Criticality::Routine);
    assert!(routine
        .verification_declaration_issues()
        .iter()
        .any(|issue| issue.message.contains("routine Requirement")
            && issue.message.contains("rejects a Requirement Judgment")));
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
        .any(|error| error.message.contains("invalid Method Qualification date")));

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
        let selector = format!("method-qualification from realization {identity}");
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
        let selector = format!("method-qualification from realization {identity}");
        let parsed = parse_selector("verification.md", 1, &selector, &mut errors);
        assert!(errors.is_empty(), "rejected `{identity}`: {errors:?}");
        assert_eq!(
            parsed,
            Some(Selector::MethodQualificationFromRealization(
                identity.to_string()
            ))
        );
    }
}

#[test]
fn execution_context_fingerprint_uses_a_versioned_canonical_envelope() {
    let declarations = parse_verification("verification.md", DECLARATIONS).unwrap();
    let context = &declarations.bindings[0].execution_context;
    let digest = execution_context_fingerprint(context);

    assert_eq!(
        canonical_json(&execution_context_json(context)),
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
    assert_eq!(digest, canonical_sha256(&execution_context_json(context)));
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
        canonical_sha256(&execution_context_json(context)),
        canonical_sha256(&execution_context_json(&reverse_inserted))
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
fn decision_policy_schedule_and_verification_challenge_match_frozen_vectors() {
    let declarations = parse_verification("verification.md", DECLARATIONS).unwrap();
    let standards = parse_standards("standards.md", STANDARDS).unwrap();
    assert_eq!(
        policy_fingerprint(&standards.policies[0]),
        "sha256:852f3fdc2d9f376403c41e215e3a06304e667df9d1e4a49eae9af53300433b06"
    );
    assert_eq!(
        schedule_fingerprint(&standards.schedule),
        "sha256:ce320ac98fed500eff1ef1032817884ca0d7dba4c2160fa22641ed0c8b058ae1"
    );
    assert_eq!(
        azimuth::fingerprint::verification_challenge_fingerprint(
            &declarations.verification_challenges[0]
        ),
        "sha256:f5af895b2e4261f5bad8deca851f96c1c8f675651b8933791fd6eba18839c0a1"
    );
}

#[test]
fn evidence_binding_matches_the_frozen_vector() {
    let binding = EvidenceBinding {
        id: "demo/binding".into(),
        check: "demo/check".into(),
        scenario: "demo/spec#requirement/scenario".into(),
        method_qualification: "demo/method".into(),
        proposition: "the Check directly exercises the scenario".into(),
        execution_context: BTreeMap::new(),
        challenge_domain: vec![ChallengeDomain::Realization],
        policy: "credible-executable".into(),
        rationale: "Reviewable.".into(),
        path: "verification.md".into(),
        line: 1,
    };
    assert_eq!(
        binding_fingerprint(
            &binding,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ),
        "sha256:17266ceea1b8b383ed06e207085bb2be104fb503ebd32ae1b5e03a79855ab650"
    );
}

#[test]
fn requirement_judgment_fingerprint_matches_the_frozen_minimal_vector() {
    let preimage = json::parse(
        r#"{
          "basis": ["the bound Check directly exercises the scenario"],
          "bindings": [{"fingerprint": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                        "id": "demo/binding"}],
          "requirement": {
            "criticality": "standard",
            "id": "demo/spec#scenario",
            "realization_obligation_areas": [],
            "semantic_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "surface": null
          },
          "format": "azimuth-requirement-judgment-fingerprint",
          "mechanisms": [],
          "policy_digest": "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
          "qualifications": [{
            "expected_fingerprint": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "id": "demo/binding",
            "verdict": "qualified"
          }],
          "realizations": [{
            "identity": "demo|rust-item|demo::subject",
            "source_fingerprint": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
          }],
          "residual_risks": ["none identified"],
          "verdict": "accepted",
          "version": 2
        }"#,
    )
    .unwrap();
    assert_eq!(
        azimuth::fingerprint::requirement_judgment_fingerprint(&preimage),
        "sha256:64456e959891fbf3f68654fc1e07e69879db4bd296cfc35570d216416fb2e701"
    );
}

#[test]
fn fingerprints_stale_only_for_their_semantic_inputs() {
    let declarations = parse_verification("verification.md", DECLARATIONS).unwrap();
    let standards = parse_standards("standards.md", STANDARDS).unwrap();
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

    let policy_digest = policy_fingerprint(&standards.policies[0]);
    let qualification = &declarations.method_qualifications[0];
    let qualification_digest =
        method_qualification_fingerprint(qualification, &check_digest, &policy_digest);
    let binding_digest = binding_fingerprint(
        binding,
        "sha256:scenario",
        &qualification_digest,
        &policy_digest,
    );
    let mut reordered_domain = binding.clone();
    reordered_domain.challenge_domain.reverse();
    reordered_domain
        .challenge_domain
        .push(ChallengeDomain::Mechanism);
    assert_eq!(
        binding_digest,
        binding_fingerprint(
            &reordered_domain,
            "sha256:scenario",
            &qualification_digest,
            &policy_digest,
        )
    );
    let context_digest = execution_context_fingerprint(&qualification.execution_context);
    let mut context_changed = binding.clone();
    context_changed
        .execution_context
        .insert("platform".into(), "macos".into());
    assert_ne!(
        binding_digest,
        binding_fingerprint(
            &context_changed,
            "sha256:scenario",
            &qualification_digest,
            &policy_digest,
        )
    );
    assert_eq!(
        context_digest,
        execution_context_fingerprint(&qualification.execution_context)
    );
    assert_eq!(
        qualification_digest,
        method_qualification_fingerprint(qualification, &check_digest, &policy_digest)
    );
}

#[test]
fn requirement_judgment_identity_uses_recomputed_total_composition() {
    let mut model = judgment_model();
    let judgment = model.requirement_judgments().next().unwrap().clone();
    let original = model
        .expected_requirement_judgment_fingerprint(&judgment)
        .unwrap();

    let original_schedule =
        schedule_fingerprint(&model.decision_standards.as_ref().unwrap().schedule);
    model
        .decision_standards
        .as_mut()
        .unwrap()
        .schedule
        .gate_challenges = vec!["oracle-perturbation".into()];
    model
        .decision_standards
        .as_mut()
        .unwrap()
        .schedule
        .scheduled_challenges = vec!["implementation-perturbation".into()];
    assert_ne!(
        original_schedule,
        schedule_fingerprint(&model.decision_standards.as_ref().unwrap().schedule)
    );
    assert_eq!(
        original,
        model
            .expected_requirement_judgment_fingerprint(&judgment)
            .unwrap()
    );

    model.check_implementations[0].file = "moved/tests.rs".into();
    assert_eq!(
        original,
        model
            .expected_requirement_judgment_fingerprint(&judgment)
            .unwrap()
    );
    model.check_implementations[0].source_fingerprint =
        "sha256:3333333333333333333333333333333333333333333333333333333333333333".into();
    assert_ne!(
        original,
        model
            .expected_requirement_judgment_fingerprint(&judgment)
            .unwrap()
    );

    model.check_implementations[0].source_fingerprint =
        "sha256:2222222222222222222222222222222222222222222222222222222222222222".into();
    model.verifications[0].method_qualifications[0].fingerprint =
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into();
    assert_eq!(
        original,
        model
            .expected_requirement_judgment_fingerprint(&judgment)
            .unwrap()
    );
    model.verifications[0].method_qualifications[0].verdict =
        azimuth::verification::MethodQualificationVerdict::Rejected;
    assert_ne!(
        original,
        model
            .expected_requirement_judgment_fingerprint(&judgment)
            .unwrap()
    );

    let mut accountable_only = judgment.clone();
    accountable_only.judged = "2026-08-22".into();
    accountable_only.judge = "another-owner".into();
    accountable_only.rationale = "Different explanatory prose.".into();
    model.verifications[0].method_qualifications[0].verdict =
        azimuth::verification::MethodQualificationVerdict::Qualified;
    assert_eq!(
        original,
        model
            .expected_requirement_judgment_fingerprint(&accountable_only)
            .unwrap()
    );
}

#[test]
fn requirement_judgment_requires_a_realization_in_every_obligated_area() {
    let mut model = judgment_model();
    let judgment = model.requirement_judgments().next().unwrap().clone();
    model.workspace.realization_obligations = vec![azimuth::workspace::RealizationObligation {
        spec: "payments/recovery".into(),
        requirement: "recover".into(),
        areas: vec!["payments".into(), "secondary".into()],
    }];
    assert!(model.requirement_judgment_preimage(&judgment).is_none());

    let mut secondary = model.realizes[0].clone();
    secondary.site = "secondary::replay".into();
    secondary.file = "secondary/recovery.rs".into();
    secondary.source = Some(SourceIdentity {
        area: "secondary".into(),
        kind: "rust-item".into(),
        address: "secondary::replay".into(),
        mount: "code".into(),
    });
    secondary.source_fingerprint =
        "sha256:4444444444444444444444444444444444444444444444444444444444444444".into();
    model.realizes.push(secondary);
    assert!(model.requirement_judgment_preimage(&judgment).is_some());

    model.realizes.retain(|site| {
        site.source
            .as_ref()
            .is_some_and(|source| source.area == "secondary")
    });
    assert!(model.requirement_judgment_preimage(&judgment).is_none());
}

#[test]
fn requirement_judgment_collects_every_matching_design_entry() {
    let mut model = judgment_model();
    model.designs = vec![Design {
        spec: "payments/recovery".into(),
        path: "design.md".into(),
        entries: vec![
            DesignEntry {
                target: Target::Requirement("recover".into()),
                mechanisms: vec![Mechanism {
                    id: "first-guard".into(),
                    kind: Enforcement::Guard,
                    scenarios: vec!["accepted-write-replayed".into()],
                    binding: Some("first-artifact".into()),
                    expected_unique: None,
                    expected_columns: Vec::new(),
                    expected_predicate: None,
                    line: 3,
                }],
                line: 2,
            },
            DesignEntry {
                target: Target::Requirement("recover".into()),
                mechanisms: vec![Mechanism {
                    id: "second-guard".into(),
                    kind: Enforcement::Guard,
                    scenarios: vec!["accepted-write-replayed".into()],
                    binding: Some("second-artifact".into()),
                    expected_unique: None,
                    expected_columns: Vec::new(),
                    expected_predicate: None,
                    line: 8,
                }],
                line: 7,
            },
        ],
        residue: String::new(),
    }];
    model.artifacts = ["first-artifact", "second-artifact"]
        .into_iter()
        .map(|id| Artifact {
            id: id.into(),
            kind: "guard".into(),
            file: format!("src/{id}.rs"),
            unique: None,
            columns: Vec::new(),
            predicate: None,
            source: Some(SourceIdentity {
                area: "payments".into(),
                kind: "rust-item".into(),
                address: format!("recovery::{id}"),
                mount: "code".into(),
            }),
        })
        .collect();
    let judgment = model.requirement_judgments().next().unwrap().clone();
    let preimage = model.requirement_judgment_preimage(&judgment).unwrap();
    let mechanisms = preimage
        .get("mechanisms")
        .and_then(json::Json::as_array)
        .unwrap();
    assert_eq!(mechanisms.len(), 2);
    assert_eq!(
        mechanisms
            .iter()
            .map(|mechanism| mechanism.get("id").unwrap().as_str().unwrap())
            .collect::<Vec<_>>(),
        ["first-guard", "second-guard"]
    );
}

#[test]
fn surface_contributions_use_the_authoritative_tuple_order() {
    let mut model = judgment_model();
    model.specs[0].requirements[0].domain = azimuth::model::Domain::Sites;
    model.specs[0].requirements[0].over = Some("payments/recovery".into());
    model.workspace.surfaces = vec![azimuth::workspace::Surface {
        id: "payments/recovery".into(),
        contributions: vec![
            azimuth::workspace::SurfaceContribution {
                area: "shared".into(),
                mount: "z-mount".into(),
                enumerator: "a-enumerator".into(),
            },
            azimuth::workspace::SurfaceContribution {
                area: "shared".into(),
                mount: "a-mount".into(),
                enumerator: "z-enumerator".into(),
            },
        ],
    }];
    model.enumerations = vec![
        azimuth::model::Enumeration {
            class: "payments/recovery".into(),
            kind: "a-enumerator".into(),
            source: "generated/z.json".into(),
            source_fingerprint:
                "sha256:5555555555555555555555555555555555555555555555555555555555555555".into(),
            identity: Some(SourceIdentity {
                area: "shared".into(),
                kind: "enumerator".into(),
                address: "surface::z".into(),
                mount: "z-mount".into(),
            }),
        },
        azimuth::model::Enumeration {
            class: "payments/recovery".into(),
            kind: "z-enumerator".into(),
            source: "generated/a.json".into(),
            source_fingerprint:
                "sha256:6666666666666666666666666666666666666666666666666666666666666666".into(),
            identity: Some(SourceIdentity {
                area: "shared".into(),
                kind: "enumerator".into(),
                address: "surface::a".into(),
                mount: "a-mount".into(),
            }),
        },
    ];
    let judgment = model.requirement_judgments().next().unwrap().clone();
    let preimage = model.requirement_judgment_preimage(&judgment).unwrap();
    let contributions = preimage
        .get("requirement")
        .unwrap()
        .get("surface")
        .unwrap()
        .get("contributions")
        .and_then(json::Json::as_array)
        .unwrap();
    assert_eq!(
        contributions[0].get("mount").and_then(json::Json::as_str),
        Some("a-mount")
    );
    let fingerprint = azimuth::fingerprint::requirement_judgment_fingerprint(&preimage);

    model.workspace.surfaces[0].contributions.reverse();
    model.enumerations.reverse();
    let reordered = model.requirement_judgment_preimage(&judgment).unwrap();
    assert_eq!(preimage, reordered);
    assert_eq!(
        fingerprint,
        azimuth::fingerprint::requirement_judgment_fingerprint(&reordered)
    );
    assert_eq!(
        fingerprint,
        "sha256:1e61f5f2cfb4791ca838602273023d6b3086f64405e58546ae1122f1bc845d2f"
    );
}

#[test]
fn model_enforces_project_cardinality_and_exports_only_current_fields() {
    let mut second = parse_verification("other/verification.md", DECLARATIONS).unwrap();
    second.bindings.clear();
    second.method_qualifications.clear();
    second.applicability_decisions.clear();
    second.verification_challenges.clear();
    second.challenge_plans.clear();
    let mut model = Model {
        specs: vec![parse_spec("spec.md", SPEC).unwrap()],
        decision_standards: Some(parse_standards("standards.md", STANDARDS).unwrap()),
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
        Some(7.0)
    );
    for key in ["covers", "mechanism_covers", "observations"] {
        assert!(exported.get(key).is_none(), "retired export key `{key}`");
    }
    for key in [
        "checks",
        "evidence_bindings",
        "method_qualifications",
        "applicability_decisions",
        "requirement_judgments",
        "decision_policies",
        "challenge_schedule",
        "verification_challenges",
        "challenge_plans",
        "check_implementations",
    ] {
        assert!(exported.get(key).is_some(), "missing v2 export key `{key}`");
    }
    let binding = &exported
        .get("evidence_bindings")
        .unwrap()
        .as_array()
        .unwrap()[0];
    assert!(binding.get("execution_context").is_some());
    assert!(binding.get("execution_context_fingerprint").is_some());
    assert!(binding.get("context").is_none() && binding.get("context_fingerprint").is_none());
    let qualification = &exported
        .get("method_qualifications")
        .unwrap()
        .as_array()
        .unwrap()[0];
    assert!(qualification.get("execution_context").is_some());
    assert!(qualification.get("context").is_none());
    let serialized = exported.to_string_pretty();
    assert!(!serialized.contains("qualification_policy"));
    assert!(!serialized.contains("Qualification Policy"));
    assert!(json::parse(&serialized).is_ok());
}
