# Spec: framework/traceability-challenge-planning

## Claim: claim-judgments-bind-total-composition
Criticality: routine

Azimuth SHALL identify one current Claim Judgment by the complete applicable assurance composition
for one non-routine parent Claim.

### Case: accepted-judgment-binds-composition
GIVEN a standard or critical Claim has an accepted Claim Judgment
WHEN its exact semantic composition is unchanged
THEN the Judgment fingerprint identifies its Claim, obligations, realizations and mechanisms
AND it identifies every Case, binding, Method Qualification, Applicability Decision, policy, basis and residual risk

### Case: relevant-drift-stales-dependent-judgment
WHEN a fingerprinted input to one Claim's composition changes
THEN its current Claim Judgment becomes stale
AND unrelated Claim Judgments retain their identity

### Case: routine-claims-reject-judgments
WHEN a Claim has routine criticality
THEN a Claim Judgment declaration for it is inapplicable

## Claim: challenge-resolution-preserves-every-candidate
Criticality: routine

Azimuth SHALL resolve Challenge Plan selectors without hiding missing, stale, rejected,
invalid, inapplicable or unresolved decision candidates.

### Case: successful-sibling-does-not-hide-gap
GIVEN one selector reaches several candidate decisions
WHEN only some candidates are current and accepted
THEN Azimuth reports every selected and non-selected disposition

### Case: overlapping-selectors-deduplicate-exact-targets
WHEN several selectors or requested plans reach the same Challenger and decision fingerprint
THEN Azimuth creates one semantic Challenge selection

### Case: relation-domain-exclusion-does-not-widen-selection
WHEN realization or mechanism traversal is outside a binding's declared challenge domain
THEN Azimuth reports an inapplicable candidate
AND it does not fall back to a path, glob or whole suite

### Case: required-scope-cannot-be-faked-by-form
GIVEN a Challenger declares required semantic scope kinds
WHEN a selector resolves its decision without those kinds
THEN the Plan does not cover the Challenger's required form

## Claim: challenge-planning-binds-current-decisions
Criticality: routine

Azimuth SHALL expand authored Challenge Plans from the complete current model into exact accepted
decision targets under one Run context.

### Case: decision-context-must-match-run
WHEN a selected decision's exact required context differs from the Run context
THEN planning fails without emitting a launch plan

### Case: challenge-only-and-mixed-plans-are-valid
WHEN a planning request contains only Challenges or a mixture of Checks and Challenges
THEN Azimuth derives one non-empty provider-neutral semantic Plan

### Case: challenger-form-binds-explicit-capability
WHEN core expands a Challenge Plan
THEN it derives the current Challenger fingerprint and open form
AND it requires an explicitly named capability with the matching operation class and form

### Case: target-cap-fails-before-truncation
WHEN selector expansion exceeds the request's nonzero candidate cap
THEN planning fails
AND it does not silently truncate the semantic selection

## Claim: adapters-receive-frozen-semantic-scope
Criticality: routine

Azimuth SHALL freeze traceability-derived semantic scope and accountable launch inputs before an
adapter translates a Challenge into provider-native work.

### Case: mutation-receives-realization-and-check-scope
WHEN a mutation Challenger selects a Method Qualification through a realization
THEN its scope contains the exact realization and bound Check implementations

### Case: fault-injection-receives-mechanism-and-check-scope
WHEN a fault-injection Challenger selects a Method Qualification through a mechanism
THEN its scope contains the exact mechanism and bound Check implementations

### Case: broad-analysis-receives-claim-composition
WHEN broad static analysis directly challenges a Claim Judgment
THEN its scope contains the exact selected Claim composition

### Case: scope-substitution-fails-publication
WHEN an adapter adds, removes or changes a semantic scope or accountable launch input
THEN Azimuth rejects the response before publishing a Run bundle

## Claim: challenge-outcomes-remain-negative-search-facts
Criticality: routine

Azimuth SHALL keep Challenge search outcomes, scheduling deferral and selection mismatch as
different states.

### Case: clean-is-not-product-evidence
WHEN a Challenge Result is clean
THEN it records only that no objection was found in the declared search
AND it creates no Observation, Method Qualification, Applicability Decision or Claim Judgment

### Case: selected-inconclusive-work-remains-non-clean
WHEN selected Challenger work cannot establish clean or findings
THEN its Challenge Result is inconclusive

### Case: deferred-work-has-no-result
WHEN planned Challenge work is omitted from an allowed incomplete Run
THEN no Challenge Result is fabricated
AND the planned target remains visibly outstanding
AND one diagnostic names the planned Challenge selection and deferral reason

### Case: selection-mismatch-is-not-an-outcome
WHEN actual target, fingerprint, context, units or scope adds to or differs from the Plan
THEN Azimuth rejects the exchange
AND it does not publish a Challenge Result

## Claim: challenge-policy-is-form-and-cost-aware
Criticality: routine

Azimuth SHALL distinguish required semantic challenge forms from their gate or scheduled execution
lane without treating omitted expensive work as success.

### Case: required-form-coverage-is-declared
GIVEN a current accepted decision uses a policy with required challenge forms
WHEN the repository model is validated
THEN every required form has a current Challenger and authored Plan that resolves the decision

### Case: scheduling-lane-does-not-stale-decision
WHEN a required form moves between gate and scheduled lanes
THEN its Method Qualification, Applicability Decision or Claim Judgment fingerprint remains unchanged

### Case: scheduled-omission-is-not-clean
WHEN a scheduled Challenge does not execute
THEN Azimuth does not count it as a clean required form

### Case: temporal-reuse-remains-deferred
WHEN prior Challenge facts exist for another Subject or time
THEN this change defines no cache-validity or cross-Subject reuse inference

## Claim: challenged-decision-impact-does-not-duplicate-results
Criticality: routine

Azimuth SHALL propagate challenged-decision impact through the semantic graph without manufacturing
additional Challenge Results.

### Case: method-finding-fans-out-through-dependent-edges
WHEN one Method Qualification receives a findings Challenge Result
THEN the impact graph reaches every dependent applicability edge, parent Claim and current Claim Judgment
AND no direct Claim Judgment Challenge Result is created

### Case: applicability-finding-remains-local
WHEN one Applicability Decision receives a findings Challenge Result
THEN the impact graph reaches only its Case, parent Claim and current Claim Judgment
AND it does not manufacture sibling applicability results

### Case: direct-judgment-challenge-remains-distinct
WHEN an authored Claim Judgment selector is planned explicitly
THEN its Challenge Result targets the exact current Claim Judgment fingerprint

### Case: several-edge-findings-deduplicate-claim-impact
WHEN several Applicability Decisions of one Claim receive findings
THEN the graph exposes one impacted Claim Judgment node
AND it retains every distinct Applicability Decision result
