# Spec: framework/change-lifecycle

## Claim: framework-only-completion
Criticality: routine

A framework-only change SHALL distinguish a deliberate absence of accepted-intent change from an
omitted intent delta.

### Case: explicit-no-delta-is-reviewable
- Event: a change with no supported intent delta reaches completion
- Additional condition or outcome: its proposal explicitly declares no intent delta with a rationale
- Required outcome: completion evaluates it without requiring an unrelated intent transition

### Case: missing-no-delta-declaration-is-rejected
- Event: a change with no supported intent delta reaches completion without the explicit declaration
- Required outcome: completion rejects the change as incomplete
