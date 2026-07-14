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

### Requirement: Ingest a language-neutral manifest

The generator SHALL read `realizes` and `covers` tags from a `*.manifest.json` a codebase emits,
equivalently to scanning source comments, so the method is polyglot.

_req-id: ingest-manifest_

#### Scenario: A manifest's realizes and covers entries enter the matrix

_scenario-id: manifest-entries-ingested • scope: unit • quant: example_

- **WHEN** a manifest lists a `realizes` entry and a `covers` entry for a scenario
- **THEN** `rtm` builds the matrix from them as it would from scanned comment tags

### Requirement: Flag a cross-cutting invariant breach

The generator SHALL flag every code site that realizes an `exposes: C` scenario but does not also
realize an `upholds: I` scenario for an invariant `I over: C` — the leak the per-scenario matrix
cannot see, since the guard scenario is realized at some other site and the class looks covered.

_req-id: flag-invariant-breach_

#### Scenario: An exposure site without a guard breaches the invariant

_scenario-id: exposure-without-guard-breaches • scope: unit • quant: example_

- **WHEN** a site realizes an `exposes: C` scenario but no `upholds: I` scenario for `I over: C`
- **THEN** it is reported as an invariant-breach hole naming the invariant and the site

### Requirement: Flag a dangling invariant

The generator SHALL flag an invariant whose surface class has no exposure sites.

_req-id: flag-dangling-invariant_

#### Scenario: An invariant over an empty class is flagged dangling

_scenario-id: invariant-over-empty-class-dangles • scope: unit • quant: example_

- **WHEN** an invariant is declared `over: C` but no realized scenario exposes class `C`
- **THEN** it is reported as a dangling-invariant hole

### Requirement: Flag a dangling upholds

The generator SHALL flag a scenario that `upholds` an invariant no spec declares.

_req-id: flag-dangling-upholds_

#### Scenario: A scenario upholding an undeclared invariant is flagged dangling

_scenario-id: upholds-undeclared-invariant-dangles • scope: unit • quant: example_

- **WHEN** a scenario carries `upholds: I` but no spec declares an invariant `I`
- **THEN** it is reported as a dangling-upholds hole

### Requirement: Flag an untraced test

The generator SHALL flag every test a manifest reports as untraced — a test in a class that
participates in tracing (has ≥1 `covers`) yet declares no scenario and is not explicitly opted out
— so a test that exercises behavior the spec never named cannot stay invisible. This is the dual of
an uncovered scenario: the RTM asks whether every scenario has a test; this asks whether every test
in a tracing class declares a scenario.

_req-id: flag-untraced-test_

#### Scenario: A test in a traced class with no scenario is flagged untraced

_scenario-id: traced-test-without-scenario-untraced • scope: unit • quant: example_

- **WHEN** a manifest reports an `untraced_tests` entry (a test in a tracing class carrying neither a `covers` nor an opt-out)
- **THEN** it is reported as an untraced-test hole naming the test site

### Requirement: Scope a run to requested specs

The generator SHALL, given a `--only` set of spec-ids, narrow the matrix to those specs plus the
transitive `references` closure of their invariants, so an invariant's reached surfaces stay in
scope while unrelated capabilities drop out.

_req-id: scope-to-requested-specs_

#### Scenario: The references closure pulls in a referenced capability

_scenario-id: references-closure-pulls-referenced • scope: unit • quant: example_

- **WHEN** a requested spec declares an invariant that `references` another capability
- **THEN** that capability's spec enters scope so its exposure sites join the invariant's class
