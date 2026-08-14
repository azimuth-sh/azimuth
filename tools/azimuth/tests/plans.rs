//! Verification plan and standards tests. Synthetic fixtures only (D2).

use azimuth::check::{rtm, HoleKind};
use azimuth::json;
use azimuth::manifest;
use azimuth::model::{Model, Oracle, Quantification, Scope, Strength};
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

### Scenario: typed-thing
WHEN an amount is produced
THEN it is an integer count of minor units
";

fn model(plan_source: &str, manifest_json: &str) -> Model {
    let spec = parse_spec("alpha.md", SPEC).expect("spec parses");
    let standards = parse_standards("standards.md", STANDARDS).expect("standards parse");
    let mut m = Model {
        specs: vec![spec],
        standards: Some(standards),
        ..Default::default()
    };
    if !plan_source.is_empty() {
        m.plans = vec![parse_plan("alpha-plan.md", plan_source).expect("plan parses")];
    }
    if !manifest_json.is_empty() {
        let root = json::parse(manifest_json).expect("valid json");
        let parsed = manifest::parse("m.json", &root).expect("manifest parses");
        m.covers = parsed.covers;
        m.realizes = parsed.realizes;
    }
    m
}

fn plan_err(source: &str) -> String {
    match parse_plan("p.md", source) {
        Ok(_) => panic!("expected a parse error"),
        Err(d) => d
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[test]
fn relational_oracle_parses() {
    let plan = parse_plan(
        "p.md",
        "# Verification: alpha\n\n## Claim: typed-thing\nOracle: relational\n\nA reason.\n",
    )
    .expect("plan parses");

    assert_eq!(plan.entries[0].oracle, Some(Oracle::Relational));
}

#[test]
fn unknown_oracle_is_rejected_in_a_plan() {
    let error = plan_err(
        "# Verification: alpha\n\n## Claim: typed-thing\nOracle: differential\n\nA reason.\n",
    );

    assert!(error.contains("unknown oracle `differential`"), "{error}");
}

fn kinds(m: &Model) -> Vec<(HoleKind, String)> {
    rtm(m)
        .into_iter()
        .map(|h| (h.kind, h.claim.unwrap_or_default()))
        .collect()
}

fn covers(scenario: &str, scope: &str, quantification: &str) -> String {
    format!(
        r#"{{"covers":[{{"spec":"alpha","scenario":"{scenario}","site":"T","file":"t.cs",
           "lang":"csharp","scope":"{scope}","quantification":"{quantification}"}}]}}"#
    )
}

fn manual_receipt(outcome: &str, expires_at: u64) -> String {
    format!(
        r#"{{"covers":[{{"spec":"alpha","scenario":"typed-thing",
           "site":"testrail:case-42","file":"https://tracker.example/runs/7#42",
           "lang":"external","source_fingerprint":"abc123","evidence_kind":"manual-test",
           "evidence_outcome":"{outcome}","observed_at":"2026-08-08T01:00:00Z",
           "expires_at":{expires_at},"scope":"unit","quantification":"universal"}}]}}"#
    )
}

#[test]
fn standards_parse() {
    let s = parse_standards("standards.md", STANDARDS).unwrap();
    assert_eq!(s.default_scope, Scope::Unit);
    let critical = s.for_level(azimuth::model::Criticality::Critical).unwrap();
    assert_eq!(critical.strength, Some(Strength::Demonstration));
    assert_eq!(critical.quantification, Some(Quantification::Universal));
    assert!(critical.residual_required);
    let routine = s.for_level(azimuth::model::Criticality::Routine).unwrap();
    assert_eq!(routine.strength, None);
}

/// The level set is closed (D6.4). A standards file that forgets one leaves claims at that level
/// with no standard at all, which would read as a clean run.
#[test]
fn standards_must_cover_every_level() {
    let source = STANDARDS.replace(
        "## Level: routine\nStrength: none\nResidual: optional\n",
        "",
    );
    let errors = parse_standards("s.md", &source).unwrap_err();
    let text = errors
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("no standard for `routine`"), "{text}");
}

const RAISED_SCOPE: &str = "\
# Verification: alpha

## Claim: concurrent-thing
Scope: component
Quantification: universal

An in-memory repository serializes writes and cannot exhibit the race.
";

#[test]
fn a_plan_entry_raises_the_required_scope() {
    let m = model(
        RAISED_SCOPE,
        &covers("concurrent-thing", "unit", "universal"),
    );
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::WrongForm, "alpha#concurrent-thing".into())),
        "{holes:?}"
    );
}

#[test]
fn evidence_at_the_required_form_satisfies() {
    let m = model(
        RAISED_SCOPE,
        &covers("concurrent-thing", "component", "universal"),
    );
    assert!(!kinds(&m).iter().any(|(k, _)| *k == HoleKind::WrongForm));
}

#[test]
fn a_current_passed_manual_receipt_satisfies_the_evidence_floor() {
    let m = model("", &manual_receipt("passed", 9_999_999_999));
    let holes = kinds(&m);
    assert!(!holes.contains(&(HoleKind::Uncovered, "alpha#typed-thing".into())));
    assert!(!holes.contains(&(HoleKind::WrongForm, "alpha#typed-thing".into())));
}

#[test]
fn a_failed_manual_receipt_is_adverse_evidence_not_coverage() {
    let m = model("", &manual_receipt("failed", 9_999_999_999));
    let holes = kinds(&m);
    assert!(holes.contains(&(HoleKind::FailedEvidence, "alpha#typed-thing".into())));
    assert!(holes.contains(&(HoleKind::Uncovered, "alpha#typed-thing".into())));
}

#[test]
fn an_expired_manual_receipt_is_named_and_no_longer_covers() {
    let m = model("", &manual_receipt("passed", 100));
    let receipt_holes = azimuth::check::receipt_holes_at(&m, 100);
    assert_eq!(receipt_holes.len(), 1);
    assert_eq!(receipt_holes[0].kind, HoleKind::ExpiredEvidence);

    let holes = kinds(&m);
    assert!(holes.contains(&(HoleKind::ExpiredEvidence, "alpha#typed-thing".into())));
    assert!(holes.contains(&(HoleKind::Uncovered, "alpha#typed-thing".into())));
}

#[test]
fn a_manual_receipt_without_freshness_is_rejected() {
    let source = manual_receipt("passed", 100).replace("\"expires_at\":100,", "");
    let root = json::parse(&source).unwrap();
    let errors = manifest::parse("manual.json", &root).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.to_string().contains("no expiry")),
        "{errors:?}"
    );
}

/// Ladders: a stronger form on any axis satisfies a requirement for a weaker one.
#[test]
fn a_stronger_form_satisfies_a_weaker_requirement() {
    let m = model(
        RAISED_SCOPE,
        &covers("concurrent-thing", "e2e", "universal"),
    );
    assert!(!kinds(&m).iter().any(|(k, _)| *k == HoleKind::WrongForm));
}

/// A completeness rule checked by one happy-path example is a hole coverage calls green. This is
/// the case the whole framework exists for.
#[test]
fn an_example_does_not_satisfy_a_universal_requirement() {
    let m = model("", &covers("typed-thing", "unit", "example"));
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::WrongForm, "alpha#typed-thing".into())),
        "{holes:?}"
    );
}

/// D7's identity: strong enforcement is self-evidencing. The alpha's model reported the better
/// design as a violation; this is the test that it no longer does.
#[test]
fn proof_strength_evidence_satisfies_without_a_test() {
    let plan = "\
# Verification: alpha

## Claim: typed-thing
Strength: proof
Evidence: Money is an integer-backed type with no floating-point constructor

Violation is unrepresentable rather than untested.
";
    let m = model(plan, "");
    let holes = kinds(&m);
    assert!(
        !holes.contains(&(HoleKind::Uncovered, "alpha#typed-thing".into())),
        "{holes:?}"
    );
    assert!(
        !holes.contains(&(HoleKind::WrongForm, "alpha#typed-thing".into())),
        "{holes:?}"
    );
    // Criticality gates evidence, not implementation.
    assert!(holes.contains(&(HoleKind::Unrealized, "alpha#typed-thing".into())));
}

/// "We'll alert on it" is the most common way a hard requirement is quietly downgraded. Detection
/// is a claim about the detector, never about the property, and it sits below demonstration.
#[test]
fn detection_does_not_satisfy_a_demonstration_requirement() {
    let plan = "\
# Verification: alpha

## Claim: typed-thing
Strength: detection
Evidence: a nightly scan for non-integer amounts
Binding: synthetic:nightly-scan
Re-established: continuously
Dies silently: the query drifts after a schema change and fires zero times
Detector test: ScanTests.FlagsPlantedRow
Detector binding: synthetic:scan-test

Recorded to show that a monitor is not a substitute here.
";
    let m = model(plan, "");
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::WrongForm, "alpha#typed-thing".into())),
        "{holes:?}"
    );
    assert!(
        holes.contains(&(
            HoleKind::UnresolvedEvidenceBinding,
            "alpha#typed-thing".into()
        )),
        "{holes:?}"
    );
    assert!(
        holes.contains(&(
            HoleKind::UnresolvedDetectorBinding,
            "alpha#typed-thing".into()
        )),
        "{holes:?}"
    );
}

/// D4.3: a monitor that can no longer fire is worse than no monitor, because it is carried on the
/// books as evidence.
#[test]
fn detection_evidence_requires_a_detector_test() {
    let source = "\
# Verification: alpha

## Claim: typed-thing
Strength: detection
Evidence: a nightly scan

A reason.
";
    let text = plan_err(source);
    assert!(text.contains("Detector test:"), "{text}");
    assert!(text.contains("Binding:"), "{text}");
    assert!(text.contains("Detector binding:"), "{text}");
    assert!(text.contains("Re-established:"), "{text}");
    assert!(text.contains("Dies silently:"), "{text}");
}

/// `Strength` qualifies a provided item; `Scope` and `Quantification` state what is required. On
/// its own, `Strength` reads as either.
#[test]
fn strength_without_evidence_is_an_error() {
    let source = "\
# Verification: alpha

## Claim: typed-thing
Strength: proof

A reason.
";
    let text = plan_err(source);
    assert!(text.contains("without evidence"), "{text}");
}

#[test]
fn evidence_without_strength_is_an_error() {
    let source = "\
# Verification: alpha

## Claim: typed-thing
Evidence: something

A reason.
";
    assert!(plan_err(source).contains("declares no strength"));
}

#[test]
fn an_entry_needs_a_reason() {
    let source = "\
# Verification: alpha

## Claim: typed-thing
Scope: component
";
    let text = plan_err(source);
    assert!(text.contains("gives no reason"), "{text}");
}

/// "Silent weakening is not available." A plan may require less than the standard, but only with a
/// recorded, accepted residual (D6.3 applied to evidence).
#[test]
fn weakening_below_the_standard_needs_an_accepted_residual() {
    let plan = "\
# Verification: alpha

## Claim: typed-thing
Quantification: example

Property testing this is awkward.
";
    let m = model(plan, &covers("typed-thing", "unit", "example"));
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::UnacceptedWeakening, "alpha#typed-thing".into())),
        "{holes:?}"
    );
}

#[test]
fn an_accepted_weakening_is_not_a_hole() {
    let plan = "\
# Verification: alpha

## Claim: typed-thing
Quantification: example
Residual: not checked across all currencies
Accepted: single-currency market until the second one launches; revisit then
";
    let m = model(plan, &covers("typed-thing", "unit", "example"));
    assert!(!kinds(&m)
        .iter()
        .any(|(k, _)| *k == HoleKind::UnacceptedWeakening));
}

#[test]
fn a_residual_must_be_accepted() {
    let source = "\
# Verification: alpha

## Residual: something

Prose about what is not covered.
";
    let text = plan_err(source);
    assert!(text.contains("is not accepted"), "{text}");
    assert!(text.contains("D6.3"), "{text}");
}

/// The uniform grammar: labels first, then a blank line, then prose. Prose before a label would
/// otherwise be silently swallowed.
#[test]
fn prose_before_a_label_is_rejected() {
    let source = "\
# Verification: alpha

## Residual: something
Prose that arrived too early.
Accepted: a reason
";
    let text = plan_err(source);
    assert!(text.contains("unrecognized line"), "{text}");
    assert!(text.contains("`Accepted:` first"), "{text}");
}

#[test]
fn a_plan_entry_for_no_claim_is_dangling() {
    let plan = "\
# Verification: alpha

## Claim: ghost
Scope: component

A reason.
";
    let m = model(plan, "");
    assert!(kinds(&m).contains(&(HoleKind::DanglingPlanEntry, "alpha#ghost".into())));
}

#[test]
fn a_plan_for_no_spec_is_dangling() {
    let spec = parse_spec("alpha.md", SPEC).unwrap();
    let standards = parse_standards("s.md", STANDARDS).unwrap();
    let plan = parse_plan(
        "p.md",
        "# Verification: beta\n\n## Claim: whatever\nScope: unit\n\nA reason.\n",
    )
    .unwrap();
    let m = Model {
        specs: vec![spec],
        standards: Some(standards),
        plans: vec![plan],
        ..Default::default()
    };
    assert!(kinds(&m).contains(&(HoleKind::DanglingPlanEntry, "beta".into())));
}

/// Values wrap: prose wraps at 100 columns everywhere in this repo, and a format that forbade
/// wrapped values would push authors toward long lines or toward saying less.
#[test]
fn label_values_wrap() {
    let source = "\
# Verification: alpha

## Claim: typed-thing
Evidence: a very long description of a mechanism that does not fit on one line and therefore
continues onto the next
Strength: proof

A reason.
";
    let plan = parse_plan("p.md", source).expect("parses");
    let evidence = plan.entries[0].evidence.as_ref().unwrap();
    assert!(
        evidence.description.ends_with("continues onto the next"),
        "{evidence:?}"
    );
}

/// A tag that omits a form is read at the weakest rung. An unstated claim should never satisfy a
/// requirement.
#[test]
fn an_undeclared_form_is_read_as_weakest() {
    let m = model(
        "",
        r#"{"covers":[{"spec":"alpha","scenario":"typed-thing","site":"T","file":"t.cs",
           "lang":"csharp"}]}"#,
    );
    let holes = kinds(&m);
    assert!(
        holes.contains(&(HoleKind::WrongForm, "alpha#typed-thing".into())),
        "{holes:?}"
    );
}

/// Without a standards file nothing is known to require, so wrong-form cannot fire. It must not
/// silently pass either.
#[test]
fn without_standards_wrong_form_cannot_fire() {
    let spec = parse_spec("alpha.md", SPEC).unwrap();
    let root = json::parse(&covers("typed-thing", "unit", "example")).unwrap();
    let parsed = manifest::parse("m.json", &root).unwrap();
    let m = Model {
        specs: vec![spec],
        covers: parsed.covers,
        ..Default::default()
    };
    assert!(!kinds(&m).iter().any(|(k, _)| *k == HoleKind::WrongForm));
}
