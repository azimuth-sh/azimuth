# Intent delta: framework/traceability-challenge-planning

## Add requirement: claim-judgments-bind-total-composition
Criticality: routine

Azimuth SHALL identify one current Claim Judgment by the complete applicable assurance composition
for one non-routine case Claim.

### Add scenario: accepted-judgment-binds-composition
GIVEN a standard or critical case Claim has an accepted Claim Judgment
WHEN its exact semantic composition is unchanged
THEN the Judgment fingerprint identifies its Claim, obligations, realizations and mechanisms
AND it identifies its bindings, Qualifications, policy, basis and residual risk

### Add scenario: relevant-drift-stales-dependent-judgment
WHEN a fingerprinted input to one Claim's composition changes
THEN its current Claim Judgment becomes stale
AND unrelated Claim Judgments retain their identity

### Add scenario: routine-claims-reject-judgments
WHEN a Claim has routine criticality
THEN a Claim Judgment declaration for it is inapplicable

## Add requirement: challenge-resolution-preserves-every-candidate
Criticality: routine

Azimuth SHALL resolve Challenge Plan selectors without hiding missing, stale, rejected,
invalid, inapplicable or unresolved decision candidates.

### Add scenario: successful-sibling-does-not-hide-gap
GIVEN one selector reaches several candidate decisions
WHEN only some candidates are current and accepted
THEN Azimuth reports every selected and non-selected disposition

### Add scenario: overlapping-selectors-deduplicate-exact-targets
WHEN several selectors or requested plans reach the same Challenger and decision fingerprint
THEN Azimuth creates one semantic Challenge selection

### Add scenario: relation-domain-exclusion-does-not-widen-selection
WHEN realization or mechanism traversal is outside a binding's declared challenge domain
THEN Azimuth reports an inapplicable candidate
AND it does not fall back to a path, glob or whole suite

### Add scenario: required-scope-cannot-be-faked-by-form
GIVEN a Challenger declares required semantic scope kinds
WHEN a selector resolves its decision without those kinds
THEN the Plan does not cover the Challenger's required form

## Add requirement: challenge-planning-binds-current-decisions
Criticality: routine

Azimuth SHALL expand authored Challenge Plans from the complete current model into exact accepted
decision targets under one Run context.

### Add scenario: qualification-context-must-match-run
WHEN a selected Qualification's exact binding context differs from the Run context
THEN planning fails without emitting a launch plan

### Add scenario: challenge-only-and-mixed-plans-are-valid
WHEN a planning request contains only Challenges or a mixture of Checks and Challenges
THEN Azimuth derives one non-empty provider-neutral semantic Plan

### Add scenario: challenger-form-binds-explicit-capability
WHEN core expands a Challenge Plan
THEN it derives the current Challenger fingerprint and open form
AND it requires an explicitly named capability with the matching operation class and form

### Add scenario: target-cap-fails-before-truncation
WHEN selector expansion exceeds the request's nonzero candidate cap
THEN planning fails
AND it does not silently truncate the semantic selection

## Add requirement: adapters-receive-frozen-semantic-scope
Criticality: routine

Azimuth SHALL freeze traceability-derived semantic scope and accountable launch inputs before an
adapter translates a Challenge into provider-native work.

### Add scenario: mutation-receives-realization-and-check-scope
WHEN a mutation Challenger selects a Qualification through a realization
THEN its scope contains the exact realization and bound Check implementations

### Add scenario: fault-injection-receives-mechanism-and-check-scope
WHEN a fault-injection Challenger selects a Qualification through a mechanism
THEN its scope contains the exact mechanism and bound Check implementations

### Add scenario: broad-analysis-receives-claim-composition
WHEN broad static analysis directly challenges a Claim Judgment
THEN its scope contains the exact selected Claim composition

### Add scenario: scope-substitution-fails-publication
WHEN an adapter adds, removes or changes a semantic scope or accountable launch input
THEN Azimuth rejects the response before publishing a Run bundle

## Add requirement: challenge-outcomes-remain-negative-search-facts
Criticality: routine

Azimuth SHALL keep Challenge search outcomes, scheduling deferral and selection mismatch as
different states.

### Add scenario: clean-is-not-product-evidence
WHEN a Challenge Result is clean
THEN it records only that no objection was found in the declared search
AND it creates no Observation, Qualification or Claim Judgment

### Add scenario: selected-inconclusive-work-remains-non-clean
WHEN selected Challenger work cannot establish clean or findings
THEN its Challenge Result is inconclusive

### Add scenario: deferred-work-has-no-result
WHEN planned Challenge work is omitted from an allowed incomplete Run
THEN no Challenge Result is fabricated
AND the planned target remains visibly outstanding
AND one diagnostic names the planned Challenge selection and deferral reason

### Add scenario: selection-mismatch-is-not-an-outcome
WHEN actual target, fingerprint, context, units or scope adds to or differs from the Plan
THEN Azimuth rejects the exchange
AND it does not publish a Challenge Result

## Add requirement: challenge-policy-is-form-and-cost-aware
Criticality: routine

Azimuth SHALL distinguish required semantic challenge forms from their gate or scheduled execution
lane without treating omitted expensive work as success.

### Add scenario: required-form-coverage-is-declared
GIVEN a current accepted decision uses a policy with required challenge forms
WHEN the repository model is validated
THEN every required form has a current Challenger and authored Plan that resolves the decision

### Add scenario: scheduling-lane-does-not-stale-decision
WHEN a required form moves between gate and scheduled lanes
THEN its Qualification or Claim Judgment fingerprint remains unchanged

### Add scenario: scheduled-omission-is-not-clean
WHEN a scheduled Challenge does not execute
THEN Azimuth does not count it as a clean required form

### Add scenario: temporal-reuse-remains-deferred
WHEN prior Challenge facts exist for another Subject or time
THEN this change defines no cache-validity or cross-Subject reuse inference

## Add requirement: qualification-impact-does-not-duplicate-judgment-results
Criticality: routine

Azimuth SHALL propagate challenged-decision impact through the semantic graph without manufacturing
additional Challenge Results.

### Add scenario: qualification-finding-impacts-dependent-judgment
WHEN one Qualification receives a findings Challenge Result
THEN the impact graph reaches its owning Claim and current Claim Judgment
AND no direct Claim Judgment Challenge Result is created

### Add scenario: direct-judgment-challenge-remains-distinct
WHEN an authored Claim Judgment selector is planned explicitly
THEN its Challenge Result targets the exact current Claim Judgment fingerprint

### Add scenario: several-edge-findings-deduplicate-claim-impact
WHEN several Qualifications of one Claim receive findings
THEN the graph exposes one impacted Claim Judgment node
AND it retains every distinct Qualification result
