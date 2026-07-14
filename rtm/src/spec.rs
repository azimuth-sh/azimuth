//! Parse an openspec spec into its scenarios and invariant declarations. The ids, forms, and
//! invariant metadata live in lines openspec preserves: `_spec-id: X_`, an id-only `_req-id: Y_`
//! under each requirement, a bullet-list scenario line
//! `_scenario-id: Z • scope: S • quant: Q_` (optionally `• oracle: O • exposes: C • upholds: I`),
//! and a `## Invariant: <name>` block carrying `_invariant-id: I • over: C_` and optional
//! `_references: cap…_`.

use crate::{Form, Invariant, Key, Oracle, ParsedSpec, Quantification, Scenario, Scope};

pub fn parse_spec(text: &str) -> ParsedSpec {
    let mut spec_id = String::new();
    let mut req_id = String::new();
    let mut scenario_name = String::new();
    let mut invariant_name = String::new();
    let mut parsed = ParsedSpec::default();

    for raw in text.lines() {
        let line = raw.trim();

        if let Some(rest) = line.strip_prefix("#### Scenario:") {
            scenario_name = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("## Invariant:") {
            invariant_name = rest.trim().to_string();
        } else if let Some(value) = meta(line, "_spec-id:") {
            spec_id = value;
        } else if let Some(value) = meta(line, "_req-id:") {
            req_id = value;
        } else if line.starts_with("_scenario-id:") {
            if let Some(scenario) = parse_scenario_line(line, &spec_id, &req_id, &scenario_name) {
                parsed.scenarios.push(scenario);
            }
        } else if line.starts_with("_invariant-id:") {
            if let Some(invariant) = parse_invariant_line(line, &spec_id, &invariant_name) {
                parsed.invariants.push(invariant);
            }
        } else if let Some(value) = meta(line, "_references:") {
            // The references line follows the invariant it belongs to.
            if let Some(last) = parsed.invariants.last_mut() {
                last.references = value
                    .split([',', ' '])
                    .filter(|r| !r.is_empty())
                    .map(String::from)
                    .collect();
            }
        }
    }

    parsed
}

fn parse_scenario_line(line: &str, spec_id: &str, req_id: &str, name: &str) -> Option<Scenario> {
    let fields = bullet_fields(line);
    let scenario_id = field(&fields, "scenario-id")?;
    let scope = Scope::parse(field(&fields, "scope")?)?;
    let quantification = Quantification::parse(field(&fields, "quant")?)?;
    let _oracle: Option<Oracle> = field(&fields, "oracle").and_then(Oracle::parse);

    Some(Scenario {
        key: Key {
            spec_id: spec_id.to_string(),
            req_id: req_id.to_string(),
            scenario_id: scenario_id.to_string(),
        },
        required_form: Form::new(scope, quantification),
        name: name.to_string(),
        exposes: field(&fields, "exposes").map(String::from),
        upholds: field(&fields, "upholds").map(String::from),
    })
}

fn parse_invariant_line(line: &str, spec_id: &str, name: &str) -> Option<Invariant> {
    let fields = bullet_fields(line);
    Some(Invariant {
        id: field(&fields, "invariant-id")?.to_string(),
        over: field(&fields, "over")?.to_string(),
        references: Vec::new(),
        name: name.to_string(),
        spec_id: spec_id.to_string(),
    })
}

/// Split a `_key: value • key: value_` metadata line into its `(key, value)` pairs.
pub(crate) fn bullet_fields(line: &str) -> Vec<(String, String)> {
    line.trim_start_matches('_')
        .trim_end_matches('_')
        .split('•')
        .filter_map(|part| {
            part.split_once(':')
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

pub(crate) fn field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

/// The value of a `_marker: value_` line, or None if the line is not that marker.
fn meta(line: &str, prefix: &str) -> Option<String> {
    if line.starts_with(prefix) {
        Some(
            line.trim_start_matches(prefix)
                .trim()
                .trim_end_matches('_')
                .trim()
                .to_string(),
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scenario_ids_and_forms_scoped_to_their_requirement() {
        let text = "\
_spec-id: demo_
### Requirement: Do a thing
_req-id: do-thing_
#### Scenario: It works
_scenario-id: it-works • scope: unit • quant: example_
#### Scenario: It never leaks
_scenario-id: it-complete • scope: component • quant: invariant_
";
        let scenarios = parse_spec(text).scenarios;
        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0].key.spec_id, "demo");
        assert_eq!(scenarios[0].key.req_id, "do-thing");
        assert_eq!(scenarios[0].key.scenario_id, "it-works");
        assert_eq!(
            scenarios[0].required_form,
            Form::new(Scope::Unit, Quantification::Example)
        );
        assert_eq!(
            scenarios[1].required_form,
            Form::new(Scope::Component, Quantification::Invariant)
        );
    }

    #[test]
    fn parses_an_optional_oracle_note_without_gating_on_it() {
        let text = "\
_spec-id: demo_
### Requirement: Rank things
_req-id: rank_
#### Scenario: Stable under permutation
_scenario-id: stable • scope: component • quant: invariant • oracle: metamorphic_
";
        let scenarios = parse_spec(text).scenarios;
        assert_eq!(scenarios.len(), 1);
        assert_eq!(
            scenarios[0].required_form,
            Form::new(Scope::Component, Quantification::Invariant)
        );
    }

    #[test]
    fn parses_scenario_exposes_and_upholds_attributes() {
        let text = "\
_spec-id: certs_
### Requirement: Public detail
_req-id: detail_
#### Scenario: Valid detail is shown
_scenario-id: detail-valid • scope: component • quant: example • exposes: public-cert_
#### Scenario: Revoked is void
_scenario-id: detail-revoked • scope: component • quant: invariant • upholds: revoked-hidden_
";
        let scenarios = parse_spec(text).scenarios;
        assert_eq!(scenarios[0].exposes.as_deref(), Some("public-cert"));
        assert_eq!(scenarios[0].upholds, None);
        assert_eq!(scenarios[1].upholds.as_deref(), Some("revoked-hidden"));
    }

    #[test]
    fn parses_an_invariant_declaration_with_over_and_references() {
        let text = "\
_spec-id: certs_
## Invariant: revoked-hidden
_invariant-id: revoked-hidden • over: public-cert_
_references: seo, search-index_

A revoked certificate SHALL NOT be shown as valid on any public surface.
";
        let invariants = parse_spec(text).invariants;
        assert_eq!(invariants.len(), 1);
        assert_eq!(invariants[0].id, "revoked-hidden");
        assert_eq!(invariants[0].over, "public-cert");
        assert_eq!(invariants[0].spec_id, "certs");
        assert_eq!(invariants[0].references, vec!["seo", "search-index"]);
    }
}
