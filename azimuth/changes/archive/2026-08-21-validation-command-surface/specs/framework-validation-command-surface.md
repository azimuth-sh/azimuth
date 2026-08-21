# Intent delta: framework/validation-command-surface

## Add requirement: validation-is-distinct-from-check-execution
Criticality: routine

Azimuth SHALL expose deterministic model validation separately from enrolled Check execution and
SHALL present traceability only as a derived view.

### Add scenario: validation-reports-complete-findings
WHEN a consumer invokes `azimuth validate`
THEN every applicable Finding has a stable kind, category, severity and corrective guidance

### Add scenario: removed-identities-fail-closed
WHEN a consumer invokes the removed top-level Check command or passes a validator identity
THEN the CLI rejects the invocation without redirecting it to validation

### Add scenario: traceability-is-derived
WHEN a consumer invokes `azimuth report traceability`
THEN the CLI derives a deterministic view from selected current Claims and realizations
AND the view creates no repository authority or execution fact

### Add scenario: initialization-points-to-validation
WHEN a consumer initializes an Azimuth model
THEN the CLI identifies `azimuth validate` as the next deterministic command
