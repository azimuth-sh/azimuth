//! The traceability matrix, derived from a spec's scenarios and the linkage tags on code and
//! tests. Pure core (`build`) plus a spec parser and a source scanner, all std-only so the tool
//! builds offline. The port of the drim-dev C# machine tier, made language-agnostic: the spec is
//! markdown and the tags are a line-comment convention any language can carry.

use std::fmt;

pub mod scan;
pub mod spec;

/// The honest kind of check a scenario demands. A completeness scenario covered only by an
/// example is a hole, not a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Example,
    Component,
    Integration,
    E2e,
    Completeness,
    Invariant,
}

impl Form {
    pub fn parse(text: &str) -> Option<Form> {
        match text.trim().to_ascii_lowercase().as_str() {
            "example" => Some(Form::Example),
            "component" => Some(Form::Component),
            "integration" => Some(Form::Integration),
            "e2e" => Some(Form::E2e),
            "completeness" => Some(Form::Completeness),
            "invariant" => Some(Form::Invariant),
            _ => None,
        }
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_ascii_lowercase())
    }
}

/// The (spec-id, req-id, scenario-id) triple — the stable key that survives display-name renames.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub spec_id: String,
    pub req_id: String,
    pub scenario_id: String,
}

/// A specified behavior: the coverage unit.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub key: Key,
    pub required_form: Form,
    pub name: String,
}

/// A `covers` tag on a test: this test verifies that scenario, at this form.
#[derive(Debug, Clone)]
pub struct Tag {
    pub key: Key,
    pub form: Form,
    pub site: String,
}

/// A `realizes` tag on production code: this site is on that scenario's path. No form.
#[derive(Debug, Clone)]
pub struct Realization {
    pub key: Key,
    pub site: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoleKind {
    Uncovered,
    Unrealized,
    WrongForm,
    Dangling,
    DanglingRealization,
}

#[derive(Debug, Clone)]
pub struct Hole {
    pub kind: HoleKind,
    pub key: Key,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub scenario: Scenario,
    pub realizations: Vec<Realization>,
    pub covering_tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub struct Matrix {
    pub rows: Vec<Row>,
    pub holes: Vec<Hole>,
}

impl Matrix {
    pub fn is_whole(&self) -> bool {
        self.holes.is_empty()
    }
}

/// The pure core: scenarios + realizations (code) + tags (tests) → the matrix and its holes.
/// Two independent axes (realized? covered?); a hole per empty axis, so the cross-states fall out
/// without double-reporting.
// realizes: azimuth-rtm generate covered-and-realized-lists-both
// realizes: azimuth-rtm flag-uncovered second-scenario-uncovered
// realizes: azimuth-rtm flag-unrealized tested-but-unrealized
// realizes: azimuth-rtm flag-wrong-form completeness-only-example
// realizes: azimuth-rtm flag-dangling unknown-scenario-dangling
pub fn build(scenarios: &[Scenario], tags: &[Tag], realizations: &[Realization]) -> Matrix {
    let mut rows = Vec::new();
    let mut holes = Vec::new();

    for scenario in scenarios {
        let realizing: Vec<Realization> = realizations
            .iter()
            .filter(|realization| realization.key == scenario.key)
            .cloned()
            .collect();
        let covering: Vec<Tag> = tags
            .iter()
            .filter(|tag| tag.key == scenario.key)
            .cloned()
            .collect();

        if realizing.is_empty() {
            holes.push(Hole {
                kind: HoleKind::Unrealized,
                key: scenario.key.clone(),
                detail: format!("'{}' has no code realizing it", scenario.name),
            });
        }

        if covering.is_empty() {
            holes.push(Hole {
                kind: HoleKind::Uncovered,
                key: scenario.key.clone(),
                detail: format!("'{}' has no covering test", scenario.name),
            });
        } else if covering.iter().all(|tag| tag.form != scenario.required_form) {
            holes.push(Hole {
                kind: HoleKind::WrongForm,
                key: scenario.key.clone(),
                detail: format!(
                    "'{}' requires {} but is covered only by {}",
                    scenario.name,
                    scenario.required_form,
                    covering
                        .iter()
                        .map(|tag| tag.form.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        rows.push(Row {
            scenario: scenario.clone(),
            realizations: realizing,
            covering_tags: covering,
        });
    }

    let declared: std::collections::HashSet<&Key> =
        scenarios.iter().map(|scenario| &scenario.key).collect();

    for tag in tags {
        if !declared.contains(&tag.key) {
            holes.push(Hole {
                kind: HoleKind::Dangling,
                key: tag.key.clone(),
                detail: format!(
                    "'{}' covers ({}, {}, {}) which no scenario declares",
                    tag.site, tag.key.spec_id, tag.key.req_id, tag.key.scenario_id
                ),
            });
        }
    }

    for realization in realizations {
        if !declared.contains(&realization.key) {
            holes.push(Hole {
                kind: HoleKind::DanglingRealization,
                key: realization.key.clone(),
                detail: format!(
                    "'{}' realizes ({}, {}, {}) which no scenario declares",
                    realization.site,
                    realization.key.spec_id,
                    realization.key.req_id,
                    realization.key.scenario_id
                ),
            });
        }
    }

    Matrix { rows, holes }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(scenario_id: &str) -> Key {
        Key {
            spec_id: "spec".into(),
            req_id: "req".into(),
            scenario_id: scenario_id.into(),
        }
    }

    fn scenario(scenario_id: &str, form: Form) -> Scenario {
        Scenario {
            key: key(scenario_id),
            required_form: form,
            name: scenario_id.into(),
        }
    }

    fn tag(scenario_id: &str, form: Form) -> Tag {
        Tag {
            key: key(scenario_id),
            form,
            site: "T".into(),
        }
    }

    fn realization(scenario_id: &str) -> Realization {
        Realization {
            key: key(scenario_id),
            site: "C".into(),
        }
    }

    #[test]
    // covers: azimuth-rtm generate covered-and-realized-lists-both example
    fn a_covered_and_realized_scenario_leaves_no_hole() {
        let matrix = build(
            &[scenario("runs", Form::Example)],
            &[tag("runs", Form::Example)],
            &[realization("runs")],
        );
        assert!(matrix.is_whole());
        assert_eq!(matrix.rows[0].covering_tags.len(), 1);
        assert_eq!(matrix.rows[0].realizations.len(), 1);
    }

    #[test]
    // covers: azimuth-rtm flag-uncovered second-scenario-uncovered example
    fn an_uncovered_scenario_is_flagged() {
        let matrix = build(
            &[scenario("runs", Form::Example)],
            &[],
            &[realization("runs")],
        );
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::Uncovered);
    }

    #[test]
    // covers: azimuth-rtm flag-unrealized tested-but-unrealized example
    fn an_unrealized_scenario_is_flagged() {
        let matrix = build(&[scenario("runs", Form::Example)], &[tag("runs", Form::Example)], &[]);
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::Unrealized);
    }

    #[test]
    // covers: azimuth-rtm flag-wrong-form completeness-only-example example
    fn a_scenario_missing_its_required_form_is_flagged() {
        let matrix = build(
            &[scenario("complete", Form::Completeness)],
            &[tag("complete", Form::Example)],
            &[realization("complete")],
        );
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::WrongForm);
    }

    #[test]
    // covers: azimuth-rtm flag-dangling unknown-scenario-dangling example
    fn a_tag_for_an_unknown_scenario_is_dangling() {
        let matrix = build(
            &[scenario("runs", Form::Example)],
            &[tag("runs", Form::Example), tag("ghost", Form::Example)],
            &[realization("runs")],
        );
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::Dangling);
    }
}
