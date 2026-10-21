# Spec: framework/traceability-challenge-planning

## Requirement: claim-judgments-bind-total-composition
Title: Claim judgments bind total composition
Criticality: routine

Azimuth SHALL identify one current Claim Judgment by the complete applicable assurance composition
for one non-routine parent Claim.

### Scenario: accepted-judgment-binds-composition
- Context: a standard or critical Claim has an accepted Claim Judgment
- Event: its exact semantic composition is unchanged
- Required outcome: the Judgment fingerprint identifies its Claim, obligations, realizations and mechanisms
- Additional condition or outcome: it identifies every Case, binding, Method Qualification, Applicability Decision, policy, basis and residual risk

### Scenario: relevant-drift-stales-dependent-judgment
- Event: a fingerprinted input to one Claim's composition changes
- Required outcome: its current Claim Judgment becomes stale
- Additional condition or outcome: unrelated Claim Judgments retain their identity

### Scenario: routine-claims-reject-judgments
- Event: a Claim has routine criticality
- Required outcome: a Claim Judgment declaration for it is inapplicable

## Requirement: challenge-resolution-preserves-every-candidate
Title: Challenge resolution preserves every candidate
Criticality: routine

Azimuth SHALL resolve Challenge Plan selectors without hiding missing, stale, rejected,
invalid, inapplicable or unresolved decision candidates.

### Scenario: successful-sibling-does-not-hide-gap
- Context: one selector reaches several candidate decisions
- Event: only some candidates are current and accepted
- Required outcome: Azimuth reports every selected and non-selected disposition

### Scenario: overlapping-selectors-deduplicate-exact-targets
- Event: several selectors or requested plans reach the same Challenger and decision fingerprint
- Required outcome: Azimuth creates one semantic Challenge selection

### Scenario: relation-domain-exclusion-does-not-widen-selection
- Event: realization or mechanism traversal is outside a binding's declared challenge domain
- Required outcome: Azimuth reports an inapplicable candidate
- Additional condition or outcome: it does not fall back to a path, glob or whole suite

### Scenario: required-scope-cannot-be-faked-by-form
- Context: a Challenger declares required semantic scope kinds
- Event: a selector resolves its decision without those kinds
- Required outcome: the Plan does not cover the Challenger's required form

## Requirement: challenge-planning-binds-current-decisions
Title: Challenge planning binds current decisions
Criticality: routine

Azimuth SHALL expand authored Challenge Plans from the complete current model into exact accepted
decision targets under one Run context.

### Scenario: decision-context-must-match-run
- Event: a selected decision's exact required context differs from the Run context
- Required outcome: planning fails without emitting a launch plan

### Scenario: challenge-only-and-mixed-plans-are-valid
- Event: a planning request contains only Challenges or a mixture of Checks and Challenges
- Required outcome: Azimuth derives one non-empty provider-neutral semantic Plan

### Scenario: challenger-form-binds-explicit-capability
- Event: core expands a Challenge Plan
- Required outcome: it derives the current Challenger fingerprint and open form
- Additional condition or outcome: it requires an explicitly named capability with the matching operation class and form

### Scenario: target-cap-fails-before-truncation
- Event: selector expansion exceeds the request's nonzero candidate cap
- Required outcome: planning fails
- Additional condition or outcome: it does not silently truncate the semantic selection

## Requirement: adapters-receive-frozen-semantic-scope
Title: Adapters receive frozen semantic scope
Criticality: routine

Azimuth SHALL freeze traceability-derived semantic scope and accountable launch inputs before an
adapter translates a Challenge into provider-native work.

### Scenario: mutation-receives-realization-and-check-scope
- Event: a mutation Challenger selects a Method Qualification through a realization
- Required outcome: its scope contains the exact realization and bound Check implementations

### Scenario: fault-injection-receives-mechanism-and-check-scope
- Event: a fault-injection Challenger selects a Method Qualification through a mechanism
- Required outcome: its scope contains the exact mechanism and bound Check implementations

### Scenario: broad-analysis-receives-claim-composition
- Event: broad static analysis directly challenges a Claim Judgment
- Required outcome: its scope contains the exact selected Claim composition

### Scenario: scope-substitution-fails-publication
- Event: an adapter adds, removes or changes a semantic scope or accountable launch input
- Required outcome: Azimuth rejects the response before publishing a Run bundle

## Requirement: challenge-outcomes-remain-negative-search-facts
Title: Challenge outcomes remain negative search facts
Criticality: routine

Azimuth SHALL keep Challenge search outcomes, scheduling deferral and selection mismatch as
different states.

### Scenario: clean-is-not-product-evidence
- Event: a Challenge Result is clean
- Required outcome: it records only that no objection was found in the declared search
- Additional condition or outcome: it creates no Observation, Method Qualification, Applicability Decision or Claim Judgment

### Scenario: selected-inconclusive-work-remains-non-clean
- Event: selected Challenger work cannot establish clean or findings
- Required outcome: its Challenge Result is inconclusive

### Scenario: deferred-work-has-no-result
- Event: planned Challenge work is omitted from an allowed incomplete Run
- Required outcome: no Challenge Result is fabricated
- Additional condition or outcome: the planned target remains visibly outstanding
- Additional condition or outcome: one diagnostic names the planned Challenge selection and deferral reason

### Scenario: selection-mismatch-is-not-an-outcome
- Event: actual target, fingerprint, context, units or scope adds to or differs from the Plan
- Required outcome: Azimuth rejects the exchange
- Additional condition or outcome: it does not publish a Challenge Result

## Requirement: challenge-policy-is-form-and-cost-aware
Title: Challenge policy is form and cost aware
Criticality: routine

Azimuth SHALL distinguish required semantic challenge forms from their gate or scheduled execution
lane without treating omitted expensive work as success.

### Scenario: required-form-coverage-is-declared
- Context: a current accepted decision uses a policy with required challenge forms
- Event: the repository model is validated
- Required outcome: every required form has a current Challenger and authored Plan that resolves the decision

### Scenario: scheduling-lane-does-not-stale-decision
- Event: a required form moves between gate and scheduled lanes
- Required outcome: its Method Qualification, Applicability Decision or Claim Judgment fingerprint remains unchanged

### Scenario: scheduled-omission-is-not-clean
- Event: a scheduled Challenge does not execute
- Required outcome: Azimuth does not count it as a clean required form

### Scenario: temporal-reuse-remains-deferred
- Event: prior Challenge facts exist for another Subject or time
- Required outcome: this change defines no cache-validity or cross-Subject reuse inference

## Requirement: challenged-decision-impact-does-not-duplicate-results
Title: Challenged decision impact does not duplicate results
Criticality: routine

Azimuth SHALL propagate challenged-decision impact through the semantic graph without manufacturing
additional Challenge Results.

### Scenario: method-finding-fans-out-through-dependent-edges
- Event: one Method Qualification receives a findings Challenge Result
- Required outcome: the impact graph reaches every dependent applicability edge, parent Claim and current Claim Judgment
- Additional condition or outcome: no direct Claim Judgment Challenge Result is created

### Scenario: applicability-finding-remains-local
- Event: one Applicability Decision receives a findings Challenge Result
- Required outcome: the impact graph reaches only its Case, parent Claim and current Claim Judgment
- Additional condition or outcome: it does not manufacture sibling applicability results

### Scenario: direct-judgment-challenge-remains-distinct
- Event: an authored Claim Judgment selector is planned explicitly
- Required outcome: its Challenge Result targets the exact current Claim Judgment fingerprint

### Scenario: several-edge-findings-deduplicate-claim-impact
- Event: several Applicability Decisions of one Claim receive findings
- Required outcome: the graph exposes one impacted Claim Judgment node
- Additional condition or outcome: it retains every distinct Applicability Decision result
