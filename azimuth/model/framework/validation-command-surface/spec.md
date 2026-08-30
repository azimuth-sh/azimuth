# Spec: framework/validation-command-surface

## Claim: validation-is-distinct-from-check-execution
Criticality: routine

Azimuth SHALL expose deterministic model validation separately from enrolled Check execution and
SHALL present traceability only as a derived view.

### Case: validation-reports-complete-findings
- Event: a consumer invokes `azimuth validate`
- Required outcome: every applicable Finding has a stable kind, category, severity and corrective guidance

### Case: removed-identities-fail-closed
- Event: a consumer invokes the removed top-level Check command or passes a validator identity
- Required outcome: the CLI rejects the invocation without redirecting it to validation

### Case: traceability-is-derived
- Event: a consumer invokes `azimuth report traceability`
- Required outcome: the CLI derives a deterministic view from selected current Claims and realizations
- Additional condition or outcome: the view creates no repository authority or execution fact

### Case: initialization-points-to-validation
- Event: a consumer initializes an Azimuth model
- Required outcome: the CLI identifies `azimuth validate` as the next deterministic command
