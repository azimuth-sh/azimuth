//! Spec parser tests.
//!
//! Fixtures here are synthetic by decision (D2). The moment this suite asserts against real demo
//! specs, the tool and the fixture are welded together and neither can move independently.

use azimuth::model::{Criticality, StepKind};
use azimuth::spec::parse_spec;

fn err(source: &str) -> String {
    match parse_spec("t.md", source) {
        Ok(_) => panic!("expected a parse error, got a spec"),
        Err(diags) => diags
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

const MINIMAL: &str = "\
# Spec: alpha/beta

Prose that claims nothing.

## Requirement: thing-holds
Criticality: standard

The system SHALL hold the thing.

### Scenario: thing-held
GIVEN a thing
WHEN it is examined
THEN it is held
AND nothing else changed
";

#[test]
fn parses_a_minimal_spec() {
    let spec = parse_spec("t.md", MINIMAL).expect("parses");
    assert_eq!(spec.id, "alpha/beta");
    assert_eq!(spec.requirements.len(), 1);

    let r = &spec.requirements[0];
    assert_eq!(r.id, "thing-holds");
    assert_eq!(r.criticality, Some(Criticality::Standard));
    assert_eq!(r.statement, "The system SHALL hold the thing.");
    assert_eq!(r.scenarios.len(), 1);

    let s = &r.scenarios[0];
    assert_eq!(s.id, "thing-held");
    assert_eq!(s.steps.len(), 4);
    assert_eq!(s.steps[0].kind, StepKind::Given);
    assert_eq!(s.steps[1].kind, StepKind::When);
    assert_eq!(s.steps[3].kind, StepKind::And);
    assert_eq!(s.steps[2].text, "it is held");
}

#[test]
fn prose_before_the_first_requirement_is_not_a_statement() {
    let spec = parse_spec("t.md", MINIMAL).unwrap();
    assert!(!spec.requirements[0].statement.contains("claims nothing"));
}

/// D6.2 vs D11: a missing *declaration* is a hole, an unrecognized *construct* is a parse error.
/// Conflating them would either let syntax through as findings or hide a semantic gap.
#[test]
fn missing_criticality_parses_and_becomes_a_hole_not_an_error() {
    let source = MINIMAL.replace("Criticality: standard\n", "");
    let spec = parse_spec("t.md", &source).expect("missing criticality still parses");
    assert_eq!(spec.requirements[0].criticality, None);
}

#[test]
fn unknown_criticality_is_a_parse_error() {
    let source = MINIMAL.replace("standard", "quite-important");
    let message = err(&source);
    assert!(message.contains("unknown criticality"), "{message}");
    assert!(
        message.contains("critical, standard or routine"),
        "{message}"
    );
}

#[test]
fn diagnostics_carry_file_and_line() {
    let source = MINIMAL.replace("standard", "quite-important");
    let message = err(&source);
    assert!(message.starts_with("t.md:6:"), "{message}");
}

#[test]
fn ids_are_lowercase_kebab_case() {
    let message = err(&MINIMAL.replace("thing-held", "Thing_Held"));
    assert!(message.contains("invalid scenario id"), "{message}");
}

#[test]
fn slash_is_only_allowed_in_spec_ids() {
    let message = err(&MINIMAL.replace("## Requirement: thing-holds", "## Requirement: a/b"));
    assert!(message.contains("only allowed in spec ids"), "{message}");
}

#[test]
fn a_scenario_needs_a_when_and_a_then() {
    let message = err(&MINIMAL.replace("WHEN it is examined\n", ""));
    assert!(message.contains("has no WHEN"), "{message}");

    let message = err(&MINIMAL.replace("THEN it is held\n", ""));
    assert!(message.contains("has no THEN"), "{message}");
}

#[test]
fn steps_must_be_ordered() {
    let source = MINIMAL.replace("AND nothing else changed", "GIVEN a late precondition");
    let message = err(&source);
    assert!(
        message.contains("`GIVEN` after a WHEN or THEN"),
        "{message}"
    );
}

#[test]
fn unrecognized_lines_in_a_scenario_fail_loudly() {
    let source = MINIMAL.replace("AND nothing else changed", "BUT something else did");
    let message = err(&source);
    assert!(message.contains("unrecognized line"), "{message}");
    assert!(message.contains("GIVEN, WHEN, THEN or AND"), "{message}");
}

#[test]
fn a_requirement_needs_at_least_one_scenario() {
    let source = "\
# Spec: alpha

## Requirement: lonely
Criticality: standard

The system SHALL do something unfalsifiable.
";
    let message = err(source);
    assert!(message.contains("has no scenarios"), "{message}");
    assert!(message.contains("unit of coverage"), "{message}");
}

#[test]
fn a_requirement_needs_a_statement() {
    let source = "\
# Spec: alpha

## Requirement: silent
Criticality: standard

### Scenario: something
WHEN a thing happens
THEN another thing happens
";
    let message = err(source);
    assert!(message.contains("has no statement"), "{message}");
}

#[test]
fn a_file_declares_exactly_one_spec() {
    let source = format!("{MINIMAL}\n# Spec: gamma\n");
    let message = err(&source);
    assert!(message.contains("exactly one spec"), "{message}");
}

#[test]
fn a_file_without_a_spec_heading_is_an_error() {
    let message = err("## Requirement: orphan\nCriticality: standard\n\nA SHALL.\n");
    assert!(message.contains("no spec declared"), "{message}");
}

#[test]
fn unknown_headings_fail_loudly() {
    let message = err(&MINIMAL.replace("## Requirement: thing-holds", "## Rule: thing-holds"));
    assert!(message.contains("unrecognized heading"), "{message}");
    assert!(message.contains("`## Requirement:"), "{message}");
}

#[test]
fn unknown_labels_fail_loudly() {
    let source = MINIMAL.replace(
        "Criticality: standard",
        "Criticality: standard\nScope: unit",
    );
    let message = err(&source);
    assert!(message.contains("unknown label `Scope:`"), "{message}");
}

/// Scope and quantification are evidence judgments and live in the verification plan (D5). A spec
/// that carries them is a spec doing the plan's job, and the parser says so.
#[test]
fn a_spec_cannot_carry_a_required_form() {
    let source = MINIMAL.replace(
        "Criticality: standard",
        "Criticality: standard\nQuantification: universal",
    );
    assert!(err(&source).contains("unknown label `Quantification:`"));
}

/// Scenario ids are unique per spec, not per requirement — that is what lets a requirement split
/// without touching a tag.
#[test]
fn scenario_ids_are_unique_across_the_whole_spec() {
    let source = "\
# Spec: alpha

## Requirement: first
Criticality: standard

A SHALL.

### Scenario: shared
WHEN a thing happens
THEN another thing happens

## Requirement: second
Criticality: standard

Another SHALL.

### Scenario: shared
WHEN a thing happens
THEN another thing happens
";
    let message = err(source);
    assert!(message.contains("not unique within this spec"), "{message}");
    assert!(message.contains("survive a requirement split"), "{message}");
}

#[test]
fn requirement_ids_are_unique() {
    let source = format!(
        "{MINIMAL}\n## Requirement: thing-holds\nCriticality: standard\n\nA SHALL.\n\n\
         ### Scenario: other\nWHEN a thing happens\nTHEN another thing happens\n"
    );
    assert!(err(&source).contains("declared twice"));
}

/// A diagram either illustrates and claims nothing, or it is the source of claims and nothing
/// restates it. Fenced blocks take the first reading and are never parsed.
#[test]
fn fenced_blocks_are_not_parsed() {
    let source = "\
# Spec: alpha

```
# Spec: not-a-spec
## Requirement: not-a-requirement
```

## Requirement: real
Criticality: routine

A SHALL.

### Scenario: real-scenario
WHEN a thing happens
THEN another thing happens
";
    let spec = parse_spec("t.md", source).expect("parses");
    assert_eq!(spec.id, "alpha");
    assert_eq!(spec.requirements.len(), 1);
    assert_eq!(spec.requirements[0].criticality, Some(Criticality::Routine));
}

#[test]
fn blockquotes_are_prose() {
    let source = MINIMAL.replace(
        "Prose that claims nothing.",
        "> A note about a concern held as prose.\n> More of it.",
    );
    assert!(parse_spec("t.md", &source).is_ok());
}

#[test]
fn multiple_errors_are_reported_together() {
    let source = "\
# Spec: alpha

## Requirement: Bad_Id
Criticality: enormous

A SHALL.

### Scenario: also-bad
THEN an outcome with no trigger
";
    let message = err(source);
    assert!(message.contains("invalid requirement id"), "{message}");
    assert!(message.contains("unknown criticality"), "{message}");
    assert!(message.contains("has no WHEN"), "{message}");
}
