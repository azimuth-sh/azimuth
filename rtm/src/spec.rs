//! Parse an openspec spec into scenarios. The ids and forms live in metadata lines openspec
//! preserves: `_spec-id: X_`, an id-only `_req-id: Y_` under each requirement, and a bullet-list
//! scenario line `_scenario-id: Z • scope: S • quant: Q_` (optionally `• oracle: O`). The parser
//! walks spec → requirement (for the req-id in scope) → scenario.

use crate::{Form, Key, Oracle, Quantification, Scenario, Scope};

pub fn parse_spec(text: &str) -> Vec<Scenario> {
    let mut spec_id = String::new();
    let mut req_id = String::new();
    let mut scenario_name = String::new();
    let mut scenarios = Vec::new();

    for raw in text.lines() {
        let line = raw.trim();

        if let Some(rest) = line.strip_prefix("#### Scenario:") {
            scenario_name = rest.trim().to_string();
        } else if let Some(value) = meta(line, "_spec-id:") {
            spec_id = value;
        } else if let Some(value) = meta(line, "_req-id:") {
            req_id = value;
        } else if line.starts_with("_scenario-id:") {
            if let Some(scenario) = parse_scenario_line(line, &spec_id, &req_id, &scenario_name) {
                scenarios.push(scenario);
            }
        }
    }

    scenarios
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
        let scenarios = parse_spec(text);
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
        let scenarios = parse_spec(text);
        assert_eq!(scenarios.len(), 1);
        assert_eq!(
            scenarios[0].required_form,
            Form::new(Scope::Component, Quantification::Invariant)
        );
    }
}
