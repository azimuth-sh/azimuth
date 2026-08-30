use azimuth::design::{Design, DesignEntry, Enforcement, Mechanism, Target};
use azimuth::json::Json;
use azimuth::model::{
    Artifact, CheckImplementation, ClassMember, Enumeration, MechanismImplementation, Model,
    SemanticChallengeScope, SemanticScopeLocator, Site, SourceIdentity,
};
use azimuth::spec::parse_spec;
use azimuth::validation::{resolve_challenge_plan, CandidateDisposition};
use azimuth::verification::{parse_standards, parse_verification, Selector, SemanticScopeKind};
use azimuth::workspace::{
    Area, Mount, RealizationObligation, Surface, SurfaceContribution, Workspace,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
static NEXT_TEMPORARY_ROOT: AtomicU64 = AtomicU64::new(0);
const STANDARDS: &str = "# Decision policies and Challenge schedule\n\n\
## Decision Policy: credible\n\
Required challenge: mutation\n\n\
The composition must be challenged.\n\n\
## Challenge Schedule: current\n\
Gate challenge: mutation\n\n\
Mutation is gate work.\n";
const VERIFICATION: &str = "# Verification: alpha\n\n\
## Check: alpha/check\n\
Method: invoke\n\
Terminal: the behavior works\n\n\
Atomic.\n\n\
## Evidence Binding: alpha/edge\n\
Check: alpha/check\n\
Case: alpha#behavior/works\n\
Method qualification: alpha/method\n\
Proposition: direct\n\
Context: {}\n\
Challenge domain: [\"realization\",\"mechanism\"]\n\
Policy: credible\n\n\
Reviewable.\n\n\
## Method Qualification: alpha/method\n\
Check: alpha/check\n\
Scope: unit\n\
Quantification: example\n\
Oracle: direct\n\
Context: {}\n\
Challenge domain: [\"realization\",\"mechanism\"]\n\
Policy: credible\n\
Verdict: qualified\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Qualified: 2026-08-22\n\
Qualifier: owner\n\n\
Qualified.\n\n\
## Applicability Decision: alpha/edge\n\
Verdict: applicable\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Decided: 2026-08-22\n\
Decider: owner\n\n\
Applicable.\n\n\
## Claim Judgment: alpha#behavior\n\
Verdict: accepted\n\
Policy: credible\n\
Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
Judged: 2026-08-22\n\
Judge: owner\n\
Basis: the exact composition is accepted\n\
Residual risk: none identified\n\n\
Accepted.\n\n\
## Challenger: mutation/search\n\
Form: mutation\n\
Searches for: an undetected change\n\
Required scope: [\"mechanism\"]\n\n\
Searches exact semantics.\n\n\
## Challenge Plan: alpha/plan\n\
Challenger: mutation/search\n\
Select: claim-judgment from mechanism alpha#guard\n\n\
Targets the total decision.\n";

fn source(area: &str, mount: &str, kind: &str, address: &str) -> SourceIdentity {
    SourceIdentity {
        area: area.into(),
        mount: mount.into(),
        kind: kind.into(),
        address: address.into(),
    }
}

fn model() -> Model {
    let alpha = parse_spec(
        "alpha/spec.md",
        "# Spec: alpha\n\n## Claim: behavior\nCriticality: standard\n\n\
         The system SHALL work.\n\n### Case: works\nEvent: invoked\nRequired: it works\n",
    )
    .unwrap();
    let surface = parse_spec(
        "surface/spec.md",
        "# Spec: surface\n\n## Claim: routes\nCriticality: routine\n\n\
         Every route SHALL exist.\n\n### Case: tagged\nEvent: built\nRequired: it exists\n",
    )
    .unwrap();
    let mut model = Model {
        specs: vec![alpha, surface],
        realizes: vec![
            Site {
                spec: "alpha".into(),
                claim: "behavior".into(),
                site: "alpha::works".into(),
                file: "src/alpha.rs".into(),
                lang: "rust".into(),
                source: Some(source("core", "code", "rust-item", "alpha::works")),
                source_fingerprint: A.into(),
            },
            Site {
                spec: "surface".into(),
                claim: "routes".into(),
                site: "GET /tagged".into(),
                file: "app/tagged.ts".into(),
                lang: "typescript".into(),
                source: Some(source("web", "app", "route", "GET /tagged")),
                source_fingerprint: B.into(),
            },
        ],
        check_implementations: vec![CheckImplementation {
            check: "alpha/check".into(),
            site: "checks::alpha".into(),
            file: "tests/alpha.rs".into(),
            lang: "rust".into(),
            source: Some(source("core", "code", "rust-item", "checks::alpha")),
            source_fingerprint: C.into(),
        }],
        class_members: vec![ClassMember {
            class: "surface".into(),
            site: "GET /enumerated".into(),
            file: "app/enumerated.ts".into(),
            lang: "typescript".into(),
            source: Some(source(
                "web",
                "app",
                "class-member",
                "surface#GET /enumerated",
            )),
        }],
        enumerations: vec![Enumeration {
            class: "surface".into(),
            kind: "routes".into(),
            source: "generated/routes.json".into(),
            source_fingerprint: C.into(),
            identity: Some(source("web", "app", "enumerator", "surface#routes")),
        }],
        artifacts: vec![Artifact {
            id: "artifact:guard".into(),
            kind: "rust-method".into(),
            file: "src/guard.rs".into(),
            unique: None,
            columns: vec!["key".into()],
            predicate: Some("active".into()),
            source: Some(source("core", "code", "rust-item", "alpha::guard")),
        }],
        decision_standards: Some(parse_standards("standards.md", STANDARDS).unwrap()),
        verifications: vec![parse_verification("verification.md", VERIFICATION).unwrap()],
        designs: vec![Design {
            spec: "alpha".into(),
            path: "alpha/design.md".into(),
            entries: vec![DesignEntry {
                target: Target::Claim("behavior".into()),
                mechanisms: vec![Mechanism {
                    id: "guard".into(),
                    kind: Enforcement::Guard,
                    cases: vec!["works".into()],
                    binding: Some("artifact:guard".into()),
                    expected_unique: None,
                    expected_columns: vec!["key".into()],
                    expected_predicate: Some("active".into()),
                    line: 1,
                }],
                line: 1,
            }],
            residue: String::new(),
        }],
        workspace: Workspace {
            path: "azimuth/workspace.json".into(),
            areas: vec![
                Area {
                    id: "core".into(),
                    mounts: vec![Mount {
                        id: "code".into(),
                        path: "".into(),
                    }],
                },
                Area {
                    id: "web".into(),
                    mounts: vec![Mount {
                        id: "app".into(),
                        path: "app".into(),
                    }],
                },
            ],
            surfaces: vec![Surface {
                id: "surface".into(),
                contributions: vec![SurfaceContribution {
                    area: "web".into(),
                    mount: "app".into(),
                    enumerator: "routes".into(),
                }],
            }],
            realization_obligations: vec![RealizationObligation {
                spec: "alpha".into(),
                claim: "behavior".into(),
                areas: vec!["core".into()],
            }],
        },
        ..Default::default()
    };
    model.specs[0].claims[0].over = Some("surface".into());
    refresh(&mut model);
    model
}

fn refresh(model: &mut Model) {
    let qualification = model
        .expected_method_qualification_fingerprint(&model.verifications[0].method_qualifications[0])
        .unwrap();
    model.verifications[0].method_qualifications[0].fingerprint = qualification;
    let applicability = model
        .expected_applicability_fingerprint(&model.verifications[0].bindings[0])
        .unwrap();
    model.verifications[0].applicability_decisions[0].fingerprint = applicability;
    let judgment = model
        .expected_claim_judgment_fingerprint(&model.verifications[0].claim_judgments[0])
        .unwrap();
    model.verifications[0].claim_judgments[0].fingerprint = judgment;
}

#[test]
fn export_resolutions_match_validation_order_and_an_independent_golden() {
    let mut model = model();
    let mut earlier = model.verifications[0].challenge_plans[0].clone();
    earlier.id = "alpha/earlier".into();
    model.verifications[0].challenge_plans.insert(0, earlier);
    let mut expected = model
        .challenge_plans()
        .map(|plan| resolve_challenge_plan(&model, plan))
        .collect::<Vec<_>>();
    expected.sort_by(|left, right| {
        (&left.plan, &left.challenger).cmp(&(&right.plan, &right.challenger))
    });
    let expected = Json::Arr(expected.iter().map(|item| item.to_json()).collect());
    let exported = model.to_json(&[]);
    assert_eq!(exported.get("challenge_resolutions"), Some(&expected));
    assert_eq!(
        exported
            .get("challenge_resolutions")
            .unwrap()
            .to_string_pretty(),
        r#"[
  {
    "format": "azimuth-challenge-resolution",
    "version": 1,
    "plan": "alpha/earlier",
    "challenger": "mutation/search",
    "candidates": [
      {
        "selector": {
          "target": "claim-judgment",
          "from": "mechanism",
          "id": "alpha#guard"
        },
        "relation": {
          "kind": "claim",
          "id": "alpha#behavior"
        },
        "target": {
          "kind": "claim-judgment",
          "id": "alpha#behavior",
          "expected_fingerprint": "sha256:92cefc23047b1c5a5171550e0386a93f55ee3f645285d9d2e5e1ffc0601c7d1e",
          "authored_fingerprint": "sha256:92cefc23047b1c5a5171550e0386a93f55ee3f645285d9d2e5e1ffc0601c7d1e"
        },
        "disposition": "selected"
      }
    ],
    "issues": []
  },
  {
    "format": "azimuth-challenge-resolution",
    "version": 1,
    "plan": "alpha/plan",
    "challenger": "mutation/search",
    "candidates": [
      {
        "selector": {
          "target": "claim-judgment",
          "from": "mechanism",
          "id": "alpha#guard"
        },
        "relation": {
          "kind": "claim",
          "id": "alpha#behavior"
        },
        "target": {
          "kind": "claim-judgment",
          "id": "alpha#behavior",
          "expected_fingerprint": "sha256:92cefc23047b1c5a5171550e0386a93f55ee3f645285d9d2e5e1ffc0601c7d1e",
          "authored_fingerprint": "sha256:92cefc23047b1c5a5171550e0386a93f55ee3f645285d9d2e5e1ffc0601c7d1e"
        },
        "disposition": "selected"
      }
    ],
    "issues": []
  }
]
"#
    );
}

#[test]
fn direct_mechanism_and_total_judgment_scope_preserve_every_component_variant() {
    let model = model();
    let resolution = resolve_challenge_plan(&model, &model.verifications[0].challenge_plans[0]);
    let candidate = resolution
        .candidates
        .iter()
        .find(|candidate| candidate.disposition == CandidateDisposition::Selected)
        .unwrap();
    let scope = model.challenge_candidate_scope(candidate).unwrap();

    assert_eq!(scope.anchors.len(), 1);
    assert_eq!(scope.anchors[0].kind, SemanticScopeKind::Mechanism);
    assert_eq!(scope.anchors[0].id, "alpha#guard");
    assert!(scope
        .inputs
        .iter()
        .any(|item| item.kind == SemanticScopeKind::ClaimJudgment));
    assert_eq!(
        scope
            .inputs
            .iter()
            .filter(|item| item.kind == SemanticScopeKind::Artifact)
            .count(),
        1
    );
    assert!(!scope
        .inputs
        .iter()
        .any(|item| item.kind == SemanticScopeKind::MechanismImplementation));
    assert!(scope.inputs.iter().any(|item| {
        item.kind == SemanticScopeKind::SurfaceMember && item.id.contains("|tagged|")
    }));
    assert!(scope.inputs.iter().any(|item| {
        item.kind == SemanticScopeKind::SurfaceMember && item.id.contains("|enumerated|")
    }));
    let enumerated = scope
        .inputs
        .iter()
        .find(|item| item.id == "surface|enumerated|app/enumerated.ts")
        .unwrap();
    assert!(matches!(
        enumerated.locator,
        Some(SemanticScopeLocator::EnumeratedSurfaceMember { .. })
    ));
    assert_eq!(
        azimuth::fingerprint::area_digest("core"),
        "sha256:c1e41a8cf0a5666a3817b2c2835d6c2fcda9c65d8ea1383255c7c16be21aee51"
    );
    assert_eq!(
        azimuth::fingerprint::realization_obligation_digest("alpha#behavior", &["core".into()]),
        "sha256:d853b01504c755d039d1950e436b619032edc9bf759f40d4d45e12f5ec1325bb"
    );
    assert_eq!(
        azimuth::fingerprint::enumerated_surface_member_digest("surface", "app/enumerated.ts"),
        "sha256:46339368007a1c8635547d2f9eeed90d77f4c59b48b78e1fa42c5cc0c01a4813"
    );
}

#[test]
fn marker_derived_mechanism_adds_one_source_implementation() {
    let mut model = model();
    model.designs[0].entries[0].mechanisms[0].binding = None;
    model.artifacts[0].id = "core|rust-symbol|alpha::guard".into();
    model.artifacts[0].kind = "rust-symbol".into();
    model.artifacts[0].source = Some(source("core", "code", "rust-symbol", "alpha::guard"));
    model.mechanism_implementations = vec![MechanismImplementation {
        spec: "alpha".into(),
        mechanism: "guard".into(),
        site: "alpha::guard".into(),
        binding: "core|rust-symbol|alpha::guard".into(),
        file: "src/guard.rs".into(),
        lang: "rust".into(),
        source: Some(source("core", "code", "rust-symbol", "alpha::guard")),
        source_fingerprint: B.into(),
    }];
    refresh(&mut model);
    let resolution = resolve_challenge_plan(&model, &model.verifications[0].challenge_plans[0]);
    let scope = model
        .challenge_candidate_scope(&resolution.candidates[0])
        .unwrap();
    let implementations = scope
        .inputs
        .iter()
        .filter(|item| item.kind == SemanticScopeKind::MechanismImplementation)
        .collect::<Vec<_>>();
    assert_eq!(implementations.len(), 1);
    assert!(matches!(
        implementations[0].locator,
        Some(SemanticScopeLocator::Source { .. })
    ));
}

#[test]
fn every_selector_projects_its_exact_authored_anchor() {
    let cases = [
        (
            Selector::ApplicabilityDecisionFromBinding("alpha/edge".into()),
            SemanticScopeKind::Binding,
            "alpha/edge",
            SemanticScopeKind::MethodQualification,
        ),
        (
            Selector::MethodQualificationFromCheck("alpha/check".into()),
            SemanticScopeKind::Check,
            "alpha/check",
            SemanticScopeKind::MethodQualification,
        ),
        (
            Selector::MethodQualificationFromRealization("core|rust-item|alpha::works".into()),
            SemanticScopeKind::Realization,
            "core|rust-item|alpha::works",
            SemanticScopeKind::MethodQualification,
        ),
        (
            Selector::MethodQualificationFromMechanism("alpha#guard".into()),
            SemanticScopeKind::Mechanism,
            "alpha#guard",
            SemanticScopeKind::MethodQualification,
        ),
        (
            Selector::ClaimJudgmentFromClaim("alpha#behavior".into()),
            SemanticScopeKind::Claim,
            "alpha#behavior",
            SemanticScopeKind::ClaimJudgment,
        ),
        (
            Selector::ClaimJudgmentFromRealization("core|rust-item|alpha::works".into()),
            SemanticScopeKind::Realization,
            "core|rust-item|alpha::works",
            SemanticScopeKind::ClaimJudgment,
        ),
        (
            Selector::ClaimJudgmentFromMechanism("alpha#guard".into()),
            SemanticScopeKind::Mechanism,
            "alpha#guard",
            SemanticScopeKind::ClaimJudgment,
        ),
    ];

    for (selector, anchor_kind, anchor_id, decision_kind) in cases {
        let mut model = model();
        model.verifications[0].challenge_plans[0].selectors = vec![selector];
        let resolution = resolve_challenge_plan(&model, &model.verifications[0].challenge_plans[0]);
        assert_eq!(resolution.candidates.len(), 1);
        let scope = model
            .challenge_candidate_scope(&resolution.candidates[0])
            .unwrap();
        assert_eq!(scope.anchors.len(), 1);
        assert_eq!(scope.anchors[0].kind, anchor_kind);
        assert_eq!(scope.anchors[0].id, anchor_id);
        assert!(scope.inputs.iter().any(|input| input.kind == decision_kind));
        if anchor_kind == SemanticScopeKind::Mechanism {
            assert!(scope
                .inputs
                .iter()
                .any(|input| input.kind == SemanticScopeKind::Artifact));
        }
    }
}

#[test]
fn selector_scopes_merge_exactly_and_reject_component_conflicts() {
    let mut model = model();
    let plan = &mut model.verifications[0].challenge_plans[0];
    plan.selectors = vec![Selector::ClaimJudgmentFromRealization(
        "core|rust-item|alpha::works".into(),
    )];
    let realization = resolve_challenge_plan(&model, model.challenge_plans().next().unwrap());
    let realization_scope = model
        .challenge_candidate_scope(&realization.candidates[0])
        .unwrap();

    model.verifications[0].challenge_plans[0].selectors =
        vec![Selector::ClaimJudgmentFromMechanism("alpha#guard".into())];
    let mechanism = resolve_challenge_plan(&model, model.challenge_plans().next().unwrap());
    let mechanism_scope = model
        .challenge_candidate_scope(&mechanism.candidates[0])
        .unwrap();
    let merged =
        SemanticChallengeScope::merge([realization_scope.clone(), mechanism_scope.clone()])
            .unwrap();
    assert_eq!(merged.anchors.len(), 2);
    assert!(merged
        .inputs
        .iter()
        .any(|item| item.kind == SemanticScopeKind::Artifact));
    assert_eq!(
        merged
            .inputs
            .iter()
            .filter(|item| item.kind == SemanticScopeKind::ClaimJudgment)
            .count(),
        1
    );

    let mut conflict = mechanism_scope;
    conflict
        .inputs
        .iter_mut()
        .find(|item| item.kind == SemanticScopeKind::ClaimJudgment)
        .unwrap()
        .fingerprint = B.into();
    assert!(SemanticChallengeScope::merge([realization_scope, conflict]).is_none());
}

#[test]
fn selected_view_retains_exact_plan_dependencies_and_resolution_bytes() {
    let root = temporary_root();
    let model_dir = root.join("model");
    fs::create_dir_all(model_dir.join("alpha")).unwrap();
    fs::create_dir_all(model_dir.join("beta")).unwrap();
    fs::write(
        model_dir.join("alpha/spec.md"),
        "# Spec: alpha\n\n## Claim: works\nCriticality: standard\n\nA SHALL work.\n\n\
         ### Case: yes\nEvent: invoked\nRequired: it works\n",
    )
    .unwrap();
    fs::write(
        model_dir.join("beta/spec.md"),
        "# Spec: beta\n\n## Claim: works\nCriticality: standard\n\nB SHALL work.\n\n\
         ### Case: yes\nEvent: invoked\nRequired: it works\n\n\
         ### Case: unrelated\nEvent: invoked elsewhere\nRequired: the sibling works\n",
    )
    .unwrap();
    fs::write(
        model_dir.join("beta/design.md"),
        "# Design: beta\n\n## Claim: works\nMechanism: selected-guard\nEnforcement: guard\nCases: [\"yes\"]\n\
         Binding: artifact:beta-selected\n\nThe selected guard establishes this Claim.\n\n\
         Mechanism: sibling-guard\nEnforcement: guard\nCases: [\"unrelated\"]\n\
         Binding: artifact:beta-sibling\n\nThe sibling guard establishes only its Claim.\n",
    )
    .unwrap();
    fs::write(
        model_dir.join("alpha/verification.md"),
        "# Verification: alpha\n\n\
         ## Check: alpha/check\nMethod: invoke\nTerminal: alpha works\n\nAtomic.\n\n\
         ## Check: beta/check\nMethod: invoke\nTerminal: beta works\n\nAtomic.\n\n\
         ## Evidence Binding: alpha/edge\nCheck: alpha/check\nCase: alpha#works/yes\n\
         Method qualification: alpha/method\nProposition: direct\nContext: {}\n\
         Challenge domain: [\"realization\"]\nPolicy: credible\n\nReviewable.\n\n\
         ## Evidence Binding: beta/edge\nCheck: beta/check\nCase: beta#works/yes\n\
         Method qualification: beta/method\nProposition: direct\nContext: {}\n\
         Challenge domain: [\"mechanism\"]\nPolicy: credible\n\nReviewable.\n\n\
         ## Challenger: mutation/search\nForm: mutation\nSearches for: an undetected change\n\
         Required scope: [\"binding\"]\n\nSearches edges.\n\n\
         ## Challenge Plan: alpha/plan\nChallenger: mutation/search\n\
         Select: applicability-decision from binding alpha/edge\n\
         Select: applicability-decision from binding beta/edge\n\nRetain both.\n",
    )
    .unwrap();
    let standards = root.join("standards.md");
    fs::write(&standards, STANDARDS).unwrap();
    let workspace = root.join("workspace.json");
    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":1,\
         \"areas\":[{\"id\":\"core\",\"mounts\":[{\"id\":\"code\",\"path\":\"src\"}]}],\
         \"surfaces\":[],\"realization_obligations\":[]}",
    )
    .unwrap();
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        format!(
            "{{\"realizes\":[\
             {{\"spec\":\"beta\",\"claim\":\"works\",\"site\":\"beta::yes\",\
             \"file\":\"src/beta.rs\",\"lang\":\"rust\",\"source_fingerprint\":\"{A}\"}},\
             {{\"spec\":\"beta\",\"claim\":\"works\",\"site\":\"beta::sibling\",\
             \"file\":\"src/sibling.rs\",\"lang\":\"rust\",\"source_fingerprint\":\"{B}\"}}],\
             \"artifacts\":[\
             {{\"id\":\"artifact:beta-selected\",\"kind\":\"rust-item\",\
             \"file\":\"src/beta.rs\"}},\
             {{\"id\":\"artifact:beta-sibling\",\"kind\":\"rust-item\",\
             \"file\":\"src/sibling.rs\"}}]}}"
        ),
    )
    .unwrap();
    let complete = azimuth::load(&model_dir, &standards, &workspace, &[manifest.clone()], &[])
        .unwrap()
        .model;
    let before = resolve_challenge_plan(&complete, complete.challenge_plans().next().unwrap())
        .to_json()
        .to_string_pretty();
    let selected = azimuth::load(
        &model_dir,
        &standards,
        &workspace,
        &[manifest],
        &["alpha".into()],
    )
    .unwrap()
    .model;
    let plan = selected.challenge_plans().next().unwrap();
    let after = resolve_challenge_plan(&selected, plan)
        .to_json()
        .to_string_pretty();
    assert_eq!(before, after);
    assert_eq!(plan.selectors.len(), 2);
    let beta = selected
        .specs
        .iter()
        .find(|spec| spec.id == "beta")
        .unwrap();
    assert_eq!(beta.claims[0].statement, "B SHALL work.");
    assert_eq!(
        beta.claims[0]
            .cases
            .iter()
            .map(|scenario| scenario.id.as_str())
            .collect::<Vec<_>>(),
        ["yes", "unrelated"]
    );
    assert_eq!(
        selected
            .realizes
            .iter()
            .filter(|site| site.spec == "beta")
            .map(|site| site.claim.as_str())
            .collect::<Vec<_>>(),
        ["works", "works"]
    );
    assert_eq!(
        selected.designs[0].entries[0].mechanisms[0].id,
        "selected-guard"
    );
    assert!(!selected
        .designs
        .iter()
        .flat_map(|design| &design.entries)
        .flat_map(|entry| &entry.mechanisms)
        .any(|mechanism| mechanism.id == "sibling-guard"));
    assert_eq!(
        selected
            .artifacts
            .iter()
            .map(|artifact| artifact.id.as_str())
            .collect::<Vec<_>>(),
        ["artifact:beta-selected"]
    );
    assert_eq!(
        resolve_challenge_plan(&selected, plan)
            .candidates
            .iter()
            .filter(|candidate| candidate.disposition == CandidateDisposition::MissingDecision)
            .count(),
        2
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_marker_relocation_preserves_semantics_and_changes_only_locator_accounts() {
    let root = temporary_root();
    let model_dir = root.join("model");
    fs::create_dir_all(model_dir.join("alpha")).unwrap();
    fs::write(
        model_dir.join("alpha/spec.md"),
        "# Spec: alpha\n\n## Claim: behavior\nCriticality: standard\n\n\
         The system SHALL work.\n\n### Case: works\nEvent: invoked\nRequired: it works\n",
    )
    .unwrap();
    fs::write(
        model_dir.join("alpha/design.md"),
        "# Design: alpha\n\n## Claim: behavior\nMechanism: guard\nEnforcement: guard\nCases: [\"works\"]\n\n\
         The qualified implementation establishes the guard.\n",
    )
    .unwrap();
    fs::write(model_dir.join("alpha/verification.md"), VERIFICATION).unwrap();
    let standards = root.join("standards.md");
    fs::write(&standards, STANDARDS).unwrap();
    let workspace = root.join("workspace.json");
    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":1,\
         \"areas\":[{\"id\":\"core\",\"mounts\":[{\"id\":\"code\",\"path\":\"src\"}]}],\
         \"surfaces\":[],\"realization_obligations\":[]}",
    )
    .unwrap();
    let manifest = root.join("manifest.json");
    let load_at = |file: &str| {
        fs::write(
            &manifest,
            format!(
                "{{\"realizes\":[{{\"spec\":\"alpha\",\"claim\":\"behavior\",\
                 \"site\":\"alpha::works\",\"file\":\"src/alpha.rs\",\"lang\":\"rust\",\
                 \"source_fingerprint\":\"{A}\"}}],\
                 \"check_implementations\":[{{\"check\":\"alpha/check\",\
                 \"site\":\"checks::alpha\",\"file\":\"src/check.rs\",\"lang\":\"rust\",\
                 \"source_fingerprint\":\"{C}\"}}],\
                 \"mechanism_implementations\":[{{\"spec\":\"alpha\",\"mechanism\":\"guard\",\
                 \"site\":\"alpha::Guard::apply\",\
                 \"binding\":\"rust-symbol:alpha::Guard::apply\",\"file\":\"{file}\",\
                 \"lang\":\"rust\",\"source_fingerprint\":\"{B}\"}}],\
                 \"artifacts\":[{{\"id\":\"rust-symbol:alpha::Guard::apply\",\
                 \"kind\":\"rust-symbol\",\"file\":\"{file}\",\"unique\":false,\
                 \"columns\":[\"key\"],\"predicate\":\"active\"}}]}}"
            ),
        )
        .unwrap();
        azimuth::load(&model_dir, &standards, &workspace, &[manifest.clone()], &[])
            .unwrap()
            .model
    };
    let mut before = load_at("src/guard.rs");
    let mut after = load_at("src/moved/guard.rs");
    let implementation_before = &before.mechanism_implementations[0];
    let implementation_after = &after.mechanism_implementations[0];
    assert_eq!(
        implementation_before.binding,
        "core|rust-symbol|alpha::Guard::apply"
    );
    assert_eq!(implementation_before.binding, implementation_after.binding);
    assert_eq!(before.artifacts[0].id, implementation_before.binding);
    assert_eq!(before.artifacts[0].columns, ["key"]);
    assert_eq!(before.artifacts[0].unique, Some(false));
    let judgment_before = before
        .expected_claim_judgment_fingerprint(&before.verifications[0].claim_judgments[0])
        .unwrap();
    let judgment_after = after
        .expected_claim_judgment_fingerprint(&after.verifications[0].claim_judgments[0])
        .unwrap();
    assert_eq!(judgment_before, judgment_after);
    before.verifications[0].claim_judgments[0].fingerprint = judgment_before;
    after.verifications[0].claim_judgments[0].fingerprint = judgment_after;
    let scope_for = |model: &Model| {
        let resolution = resolve_challenge_plan(model, model.challenge_plans().next().unwrap());
        model
            .challenge_candidate_scope(&resolution.candidates[0])
            .unwrap()
    };
    let scope_before = scope_for(&before);
    let scope_after = scope_for(&after);
    let marker_before = scope_before
        .inputs
        .iter()
        .find(|item| item.kind == SemanticScopeKind::MechanismImplementation)
        .unwrap();
    let marker_after = scope_after
        .inputs
        .iter()
        .find(|item| item.kind == SemanticScopeKind::MechanismImplementation)
        .unwrap();
    assert_eq!(marker_before.id, marker_after.id);
    assert_eq!(marker_before.fingerprint, marker_after.fingerprint);
    assert_ne!(marker_before.locator, marker_after.locator);
    assert!(matches!(
        &marker_after.locator,
        Some(SemanticScopeLocator::Source { file, site, .. })
            if file == "src/moved/guard.rs" && site == "alpha::Guard::apply"
    ));
    assert_ne!(
        azimuth::fingerprint::model_digest(&before, &[]),
        azimuth::fingerprint::model_digest(&after, &[])
    );
    for forbidden in [
        "rust-symbol:alpha::Guard::apply",
        "core|rust-symbol|alpha::Guard::apply",
    ] {
        fs::write(
            model_dir.join("alpha/design.md"),
            format!(
                "# Design: alpha\n\n## Claim: works\nMechanism: explicit-guard\n\
                 Enforcement: guard\nBinding: {forbidden}\n\n\
                 A marker companion cannot cross into the explicit route.\n"
            ),
        )
        .unwrap();
        let errors = azimuth::load(&model_dir, &standards, &workspace, &[manifest.clone()], &[])
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.message.contains("may not bind marker-only Artifact")));
    }
    fs::remove_dir_all(root).unwrap();
}

fn temporary_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    temporary_root_with_nonce(nonce)
}

fn temporary_root_with_nonce(nonce: u128) -> PathBuf {
    let sequence = NEXT_TEMPORARY_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "azimuth-decision-scope-{nonce}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn temporary_roots_remain_distinct_when_clock_nonce_matches() {
    let first = temporary_root_with_nonce(0);
    let second = temporary_root_with_nonce(0);

    assert_ne!(first, second);
    fs::remove_dir_all(first).unwrap();
    fs::remove_dir_all(second).unwrap();
}
