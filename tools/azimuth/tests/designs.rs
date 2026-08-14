//! Design artifact tests. Synthetic fixtures only (D2).

use azimuth::check::{rtm, HoleKind};
use azimuth::design::{parse_design, Enforcement, Target};
use azimuth::model::{Artifact, MechanismCover, MechanismImplementation, Model};
use azimuth::plan::{parse_plan, parse_standards};
use azimuth::spec::parse_spec;

const STANDARDS: &str = "\
# Verification standards
Default scope: unit

## Level: critical
Strength: demonstration
Quantification: universal
Residual: required

## Level: standard
Strength: demonstration
Quantification: example
Residual: optional

## Level: routine
Strength: none
Residual: optional
";

const SPEC: &str = "\
# Spec: alpha

## Requirement: matters
Criticality: critical

A SHALL.

### Scenario: concurrent-thing
WHEN things happen concurrently
THEN exactly one of them wins

## Requirement: lesser
Criticality: standard

A lesser SHALL.

### Scenario: ordinary-thing
WHEN a thing happens
THEN it works
";

fn design_err(source: &str) -> String {
    match parse_design("d.md", source) {
        Ok(_) => panic!("expected a parse error"),
        Err(d) => d
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn model(design_source: &str, plan_source: &str) -> Model {
    let spec = parse_spec("alpha.md", SPEC).expect("spec parses");
    let standards = parse_standards("s.md", STANDARDS).expect("standards parse");
    let mut m = Model {
        specs: vec![spec],
        standards: Some(standards),
        ..Default::default()
    };
    if !design_source.is_empty() {
        m.designs = vec![parse_design("d.md", design_source).expect("design parses")];
        m.artifacts = m.designs[0]
            .entries
            .iter()
            .flat_map(|entry| &entry.mechanisms)
            .filter_map(|mechanism| {
                mechanism.binding.as_ref().map(|binding| Artifact {
                    id: binding.clone(),
                    kind: match mechanism.kind {
                        Enforcement::Type => "dotnet-type",
                        Enforcement::Constraint => "database-index",
                        _ => "dotnet-method",
                    }
                    .into(),
                    file: "subject.cs".into(),
                    unique: (mechanism.kind == Enforcement::Constraint).then_some(true),
                    columns: vec![],
                    predicate: None,
                    source: None,
                })
            })
            .collect();
    }
    if !plan_source.is_empty() {
        m.plans = vec![parse_plan("p.md", plan_source).expect("plan parses")];
    }
    m
}

fn kinds(m: &Model) -> Vec<(HoleKind, String)> {
    rtm(m)
        .into_iter()
        .map(|h| (h.kind, h.claim.unwrap_or_default()))
        .collect()
}

const DESIGN: &str = "\
# Design: alpha

## Requirement: matters
Mechanism: concurrent-insert-constraint
Enforcement: constraint
Binding: `ux_alpha_thing` — partial unique index on `things(alpha_id)`

Compare-and-set at the storage layer, not a check-then-write in the handler.

## Residue

Offers are fanned out optimistically and withdrawn afterwards. Deliberate.
";

#[test]
fn parses_a_design() {
    let d = parse_design("d.md", DESIGN).expect("parses");
    assert_eq!(d.spec, "alpha");
    assert_eq!(d.entries.len(), 1);
    let entry = &d.entries[0];
    assert_eq!(entry.target, Target::Requirement("matters".into()));
    assert_eq!(entry.mechanisms.len(), 1);
    assert_eq!(entry.mechanisms[0].id, "concurrent-insert-constraint");
    assert_eq!(entry.mechanisms[0].kind, Enforcement::Constraint);
    assert!(entry.mechanisms[0]
        .binding
        .as_deref()
        .unwrap()
        .contains("ux_alpha_thing"));
    assert!(d.residue.contains("optimistically"));
}

#[test]
fn parses_expected_schema_properties_separately_from_the_binding() {
    let source = DESIGN.replace(
        "Binding: `ux_alpha_thing` — partial unique index on `things(alpha_id)`",
        "Binding: postgres-index:things.ux_alpha_thing\n\
Expect: unique=true; columns=alpha_id; predicate=active",
    );
    let design = parse_design("d.md", &source).expect("parses");
    let mechanism = &design.entries[0].mechanisms[0];
    assert_eq!(
        mechanism.binding.as_deref(),
        Some("postgres-index:things.ux_alpha_thing")
    );
    assert_eq!(mechanism.expected_unique, Some(true));
    assert_eq!(mechanism.expected_columns, ["alpha_id"]);
    assert_eq!(mechanism.expected_predicate.as_deref(), Some("active"));
}

/// C2 in the concern catalog: one rule, a choke point *and* a representation constraint. A single
/// mechanism field would have conflated them.
#[test]
fn a_requirement_may_carry_several_mechanisms() {
    let source = "\
# Design: alpha

## Requirement: matters
Mechanism: transition-writer
Enforcement: choke-point
Binding: `Alpha.Transition` is the only writer
Mechanism: current-state-constraint
Enforcement: constraint
Binding: conditional update predicated on current state

The choke point alone does not survive concurrency.
";
    let d = parse_design("d.md", source).expect("parses");
    let m = &d.entries[0].mechanisms;
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].kind, Enforcement::ChokePoint);
    assert_eq!(m[1].kind, Enforcement::Constraint);
}

#[test]
fn every_mechanism_needs_an_enforcement() {
    let source = DESIGN.replace("Enforcement: constraint\n", "");
    let text = design_err(&source);
    assert!(text.contains("has no enforcement"), "{text}");
}

#[test]
fn a_binding_needs_an_enforcement() {
    let source = DESIGN.replace("Enforcement: constraint\n", "");
    let text = design_err(&source);
    assert!(text.contains("with no enforcement"), "{text}");
}

#[test]
fn a_binding_that_no_extractor_emitted_is_a_hole() {
    let mut m = model(DESIGN, "");
    m.artifacts.clear();
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::UnresolvedDesignBinding, "alpha#matters".into())),
        "{holes:?}"
    );
}

#[test]
fn a_non_unique_index_cannot_back_constraint_enforcement() {
    let mut m = model(DESIGN, "");
    m.artifacts[0].unique = Some(false);
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::EnforcementMismatch, "alpha#matters".into())),
        "{holes:?}"
    );
}

#[test]
fn the_enforcement_set_is_closed() {
    let text = design_err(&DESIGN.replace("constraint", "vibes"));
    assert!(text.contains("unknown enforcement `vibes`"), "{text}");
    assert!(text.contains("choke-point"), "{text}");
}

/// "An entry that states a mechanism without a reason records a fact the code already knows."
#[test]
fn an_entry_needs_a_reason() {
    let source = "\
# Design: alpha

## Requirement: matters
Mechanism: constraint
Enforcement: constraint
Binding: an index
";
    assert!(design_err(source).contains("gives no reason"));
}

/// Strength is derived from the enforcement kind (D7) and writing it would duplicate a derivable
/// fact. The label is not in the closed set, so it fails loudly rather than being ignored.
#[test]
fn strength_is_never_written_in_a_design_entry() {
    let source = DESIGN.replace(
        "Mechanism: concurrent-insert-constraint",
        "Strength: proof\nMechanism: concurrent-insert-constraint",
    );
    let text = design_err(&source);
    assert!(text.contains("unrecognized line"), "{text}");
}

#[test]
fn a_code_mechanism_may_derive_its_binding_from_an_implementation_tag() {
    let source = DESIGN.replace(
        "Binding: `ux_alpha_thing` — partial unique index on `things(alpha_id)`\n",
        "",
    );
    let mut m = model(&source, "");
    m.mechanism_implementations.push(MechanismImplementation {
        spec: "alpha".into(),
        mechanism: "concurrent-insert-constraint".into(),
        binding: "dotnet-symbol:Alpha.Insert".into(),
        file: "alpha.cs".into(),
        lang: "csharp".into(),
        source_fingerprint: "abc".into(),
        source: None,
    });
    m.artifacts.push(Artifact {
        id: "dotnet-symbol:Alpha.Insert".into(),
        kind: "dotnet-method".into(),
        file: "alpha.cs".into(),
        unique: None,
        columns: vec![],
        predicate: None,
        source: None,
    });

    assert!(!kinds(&m)
        .iter()
        .any(|(kind, _)| *kind == HoleKind::UnresolvedDesignBinding));

    m.mechanism_implementations.clear();
    assert!(kinds(&m).contains(&(HoleKind::UnresolvedDesignBinding, "alpha#matters".into())));
}

#[test]
fn a_mechanism_must_resolve_to_one_atomic_implementation() {
    let mut m = model(DESIGN, "");
    m.mechanism_implementations.push(MechanismImplementation {
        spec: "alpha".into(),
        mechanism: "concurrent-insert-constraint".into(),
        binding: "dotnet-symbol:Alpha.Insert".into(),
        file: "alpha.cs".into(),
        lang: "csharp".into(),
        source_fingerprint: "abc".into(),
        source: None,
    });
    let hole = rtm(&m)
        .into_iter()
        .find(|hole| hole.kind == HoleKind::UnresolvedDesignBinding)
        .expect("explicit plus derived is ambiguous");
    assert!(
        hole.detail.contains("resolves to 2 bindings"),
        "{}",
        hole.detail
    );
}

#[test]
fn mechanism_ids_are_unique_across_the_design() {
    let source = format!(
        "{DESIGN}\n## Requirement: lesser\nMechanism: concurrent-insert-constraint\n\
         Enforcement: guard\nBinding: another guard\n\nA reason.\n"
    );
    let text = design_err(&source);
    assert!(text.contains("declared twice"), "{text}");
}

#[test]
fn mechanism_links_to_an_unknown_design_identity_are_dangling() {
    let mut m = model(DESIGN, "");
    m.mechanism_implementations.push(MechanismImplementation {
        spec: "alpha".into(),
        mechanism: "ghost".into(),
        binding: "dotnet-symbol:Alpha.Ghost".into(),
        file: "alpha.cs".into(),
        lang: "csharp".into(),
        source_fingerprint: "abc".into(),
        source: None,
    });
    m.mechanism_covers.push(MechanismCover {
        spec: "alpha".into(),
        mechanism: "ghost".into(),
        site: "ghost test".into(),
        file: "alpha.test.cs".into(),
        lang: "csharp".into(),
        source_fingerprint: "def".into(),
        source: None,
        scope: None,
        quantification: None,
        oracle: None,
    });
    let found = kinds(&m);
    assert!(found.contains(&(
        HoleKind::DanglingMechanismImplementation,
        "alpha#ghost".into()
    )));
    assert!(found.contains(&(HoleKind::DanglingMechanismCover, "alpha#ghost".into())));
}

/// The residue attaches to no claim, participates in no check, and is the one part the machine
/// must never pretend to understand.
#[test]
fn the_residue_is_never_parsed() {
    let source = "\
# Design: alpha

## Requirement: matters
Mechanism: per-handler-check
Enforcement: guard
Binding: every handler checks

A reason.

## Residue

Enforcement: this line looks like a label and is prose.
Binding: so is this one.
";
    let d = parse_design("d.md", source).expect("parses");
    assert_eq!(d.entries.len(), 1);
    assert!(d.residue.contains("looks like a label"));
}

/// D6.5: a design entry is required for `critical`, optional for `standard`.
///
/// Reported only once the artifact is in use: a spec covered by some other design file, with none
/// of its own.
#[test]
fn a_critical_requirement_without_a_design_entry_is_a_hole() {
    let elsewhere =
        "# Design: beta\n\n## Requirement: other\nMechanism: guard\nEnforcement: guard\nBinding: x\n\nA reason.\n";
    let mut m = model("", "");
    m.designs = vec![azimuth::design::parse_design("beta.md", elsewhere).unwrap()];
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::UndeclaredMechanism, "alpha#matters".into())),
        "{holes:?}"
    );
    assert!(!holes.contains(&(HoleKind::UndeclaredMechanism, "alpha#lesser".into())));
}

#[test]
fn a_design_entry_closes_it() {
    let m = model(DESIGN, "");
    assert!(!kinds(&m)
        .iter()
        .any(|(k, _)| *k == HoleKind::UndeclaredMechanism));
}

/// A facet attaches at the coarsest level where its statement is true, but an entry may key on a
/// scenario where the mechanism genuinely differs.
#[test]
fn a_scenario_keyed_entry_also_satisfies_the_requirement() {
    let source = "\
# Design: alpha

## Claim: concurrent-thing
Mechanism: concurrent-index
Enforcement: constraint
Binding: an index

A reason.
";
    let m = model(source, "");
    assert!(!kinds(&m)
        .iter()
        .any(|(k, _)| *k == HoleKind::UndeclaredMechanism));
}

#[test]
fn a_design_entry_for_no_requirement_is_dangling() {
    let source = "\
# Design: alpha

## Requirement: ghost
Mechanism: ghost-index
Enforcement: constraint
Binding: an index

A reason.
";
    let m = model(source, "");
    assert!(kinds(&m).contains(&(HoleKind::DanglingDesignEntry, "alpha#ghost".into())));
}

/// The three-artifact check. Proof comes from a mechanism at the top of the enforcement ladder,
/// and the developer owns which mechanism that is — so a plan claiming proof with nothing behind
/// it is asserting the strongest available result out of thin air.
#[test]
fn proof_evidence_needs_a_proof_capable_mechanism() {
    let plan = "\
# Verification: alpha

## Claim: concurrent-thing
Strength: proof
Evidence: it is simply impossible

Asserted without a mechanism.
";
    let weak_design = "\
# Design: alpha

## Requirement: matters
Mechanism: per-handler-guard
Enforcement: guard
Binding: every handler checks

Guards everywhere, because a choke point was not feasible.
";
    let m = model(weak_design, plan);
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::UnbackedProof, "alpha#concurrent-thing".into())),
        "{holes:?}"
    );

    let m = model(DESIGN, plan);
    assert!(!kinds(&m).iter().any(|(k, _)| *k == HoleKind::UnbackedProof));
}

#[test]
fn the_ladder_ranks_enforcement() {
    assert_eq!(Enforcement::Type.rung(), 1);
    assert_eq!(Enforcement::Schema.rung(), 1);
    assert_eq!(Enforcement::Constraint.rung(), 2);
    assert_eq!(Enforcement::ChokePoint.rung(), 2);
    assert_eq!(Enforcement::Middleware.rung(), 3);
    assert_eq!(Enforcement::Guard.rung(), 4);

    assert!(Enforcement::Type.is_proof_capable());
    assert!(Enforcement::ChokePoint.is_proof_capable());
    assert!(!Enforcement::Middleware.is_proof_capable());
    assert!(!Enforcement::Guard.is_proof_capable());
}

/// D8.1: each mechanism is usable alone. A project that has not adopted the design artifact must
/// not be told that every critical requirement is a hole.
#[test]
fn without_any_design_artifact_the_mechanism_check_is_silent() {
    let m = model("", "");
    assert!(m.designs.is_empty());
    assert!(!kinds(&m)
        .iter()
        .any(|(k, _)| *k == HoleKind::UndeclaredMechanism));
}
