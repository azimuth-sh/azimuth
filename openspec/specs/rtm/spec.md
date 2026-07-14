# rtm Specification

## Purpose
The traceability-matrix generator, specified in its own format so it validates itself: `rtm` run
on this spec and its own tagged source reports no holes.

_spec-id: azimuth-rtm_

## Requirements

### Requirement: Generate the traceability matrix

The generator SHALL read a spec's scenarios, the `covers` tags on tests, and the `realizes` tags
on code, and produce, for each scenario, its realizing code sites and covering tests, keyed by
the (spec-id, req-id, scenario-id) triple.

_req-id: generate_

#### Scenario: A covered and realized scenario lists both columns

_scenario-id: covered-and-realized-lists-both • scope: unit • quant: example_

- **WHEN** a scenario has both a `realizes` tag and a `covers` tag for its triple
- **THEN** the matrix lists that scenario with its code site and its test, and reports no hole for it

### Requirement: Flag an uncovered scenario

The generator SHALL flag a scenario that no `covers` tag covers.

_req-id: flag-uncovered_

#### Scenario: A scenario with no covering test is flagged uncovered

_scenario-id: second-scenario-uncovered • scope: unit • quant: example_

- **WHEN** a scenario has no `covers` tag
- **THEN** it is reported as an uncovered hole

### Requirement: Flag an unrealized scenario

The generator SHALL flag a scenario that no `realizes` tag realizes.

_req-id: flag-unrealized_

#### Scenario: A scenario with no realizing code is flagged unrealized

_scenario-id: tested-but-unrealized • scope: unit • quant: example_

- **WHEN** a scenario has a `covers` tag but no `realizes` tag
- **THEN** it is reported as an unrealized hole

### Requirement: Flag a scenario missing its required form

The generator SHALL flag a scenario no covering tag reaches on both form axes (scope and
quantification), where a stronger covering form on either axis still satisfies the requirement.

_req-id: flag-wrong-form_

#### Scenario: A scenario under-proven on either form axis is flagged

_scenario-id: under-proven-on-either-axis • scope: unit • quant: example_

- **WHEN** a scenario requires a component invariant but its only covering tag is a unit example
- **THEN** it is reported as a wrong-form hole

### Requirement: Flag a dangling tag

The generator SHALL flag a `covers` tag whose triple matches no scenario.

_req-id: flag-dangling_

#### Scenario: A tag for an unknown scenario is flagged dangling

_scenario-id: unknown-scenario-dangling • scope: unit • quant: example_

- **WHEN** a `covers` tag references a triple that no scenario declares
- **THEN** it is reported as a dangling tag
