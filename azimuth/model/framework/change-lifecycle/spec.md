# Spec: framework/change-lifecycle

## Requirement: framework-only-completion
Title: Framework only completion

A framework-only change SHALL distinguish a deliberate absence of accepted-intent change from an
omitted intent delta.

### Scenario: explicit-no-delta-is-reviewable
- Event: a change with no supported intent delta reaches completion
- Additional condition or outcome: its proposal explicitly declares no intent delta with a rationale
- Required outcome: completion evaluates it without requiring an unrelated intent transition

### Scenario: missing-no-delta-declaration-is-rejected
- Event: a change with no supported intent delta reaches completion without the explicit declaration
- Required outcome: completion rejects the change as incomplete
