//! Parse an openspec spec into scenarios. The ids and forms live in metadata lines openspec
//! preserves: `_spec-id: X_`, an id-only `_req-id: Y_` under each requirement, and
//! `_scenario-id: Z • form: F_` under each scenario. The parser walks spec → requirement (for the
//! req-id in scope) → scenario.

use crate::{Form, Key, Scenario};

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
            let body = line
                .trim_start_matches("_scenario-id:")
                .trim_end_matches('_');
            if let Some((id_part, form_part)) = body.split_once("• form:") {
                if let Some(form) = Form::parse(form_part) {
                    scenarios.push(Scenario {
                        key: Key {
                            spec_id: spec_id.clone(),
                            req_id: req_id.clone(),
                            scenario_id: id_part.trim().to_string(),
                        },
                        required_form: form,
                        name: scenario_name.clone(),
                    });
                }
            }
        }
    }

    scenarios
}

/// The value of a `_marker: value_` line, or None if the line is not that marker. `_scenario-id:`
/// is handled separately (it carries a form), so this guards against `_req-id:` matching it.
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
_scenario-id: it-works • form: example_
#### Scenario: It is complete
_scenario-id: it-complete • form: completeness_
";
        let scenarios = parse_spec(text);
        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0].key.spec_id, "demo");
        assert_eq!(scenarios[0].key.req_id, "do-thing");
        assert_eq!(scenarios[0].key.scenario_id, "it-works");
        assert_eq!(scenarios[0].required_form, Form::Example);
        assert_eq!(scenarios[1].required_form, Form::Completeness);
    }
}
