# Spec: framework/change-lifecycle

## Requirement: framework-only-completion
Criticality: routine

A framework-only change SHALL distinguish a deliberate absence of accepted-intent change from an
omitted intent delta.

### Scenario: explicit-no-delta-is-reviewable
WHEN a change with no supported intent delta reaches completion
AND its proposal explicitly declares no intent delta with a rationale
THEN completion evaluates it without requiring an unrelated intent transition

### Scenario: missing-no-delta-declaration-is-rejected
WHEN a change with no supported intent delta reaches completion without the explicit declaration
THEN completion rejects the change as incomplete
