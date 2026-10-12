# Intent delta: framework/traceability-challenge-planning

## Replace claim: claim-judgments-bind-total-composition
From: sha256:1115e66cc1f79b27417ed8fb449201a7639c5c90442aeaed61c40f238eed54e2
Because: Definition review and exact Subject health replace the mandatory total-composition Judgment.
Criticality: routine
Remove cases: routine-claims-reject-judgments

Azimuth SHALL identify each Claim Review by its complete relevant definition composition and each Claim Health by that composition, its exact Subject, current review and execution basis. Historical record names SHALL NOT create current assessment status.

### Case: accepted-judgment-binds-composition

Given a current accepted Claim Review, when its identity is assessed, then its fingerprint pins the Claim, Cases, design, realizations, mechanisms, Checks, contributions, independent inputs and selected Challenger declarations and implementation configuration. Acceptance assesses the definition rather than asserting observed product health.

### Case: relevant-drift-stales-dependent-judgment

Given a retained Claim Review or Claim Health, when one of its relevant fingerprinted inputs changes, then it becomes stale. Unrelated assessments retain their identity; Health also becomes stale if its selected current review or active evidence basis changes.

### Case: routine-claims-have-no-mandatory-native-review

Given a routine Claim, when the standard workflow derives required assurance work, then it does not demand Claim Review or Claim Health solely because that Claim exists. This does not convert historical inapplicable declarations into positive native assessments.

## Replace claim: challenge-resolution-preserves-every-candidate
From: sha256:89e83ba938cf33a732db4313439ad3022bf8660ccc58a61f0cc5796e39d2c0f5
Because: Assessment target and source-scope resolution retain explicit gaps without relying on per-binding challenge domains.
Criticality: routine

Azimuth SHALL resolve machine Challenger targets and scope without hiding missing, stale, rejected, invalid or unresolved assessment candidates.

### Case: successful-sibling-does-not-hide-gap

Given a requested selection containing several targets, when only some are current and eligible, then every selected and non-selected disposition remains visible. Eligible siblings do not make unresolved targets disappear.

### Case: overlapping-selectors-deduplicate-exact-targets

Given several selectors or requested plans reaching the same Challenger and assessment fingerprint, when selection is resolved, then it creates one semantic selection and retains its declared required scope.

### Case: relation-domain-exclusion-does-not-widen-selection

Given declared Claim, Mechanism or Check scope that cannot resolve a required target, when scope is prepared, then it reports the unresolved mapping and does not fall back to a path, glob or whole suite. The removed per-binding challenge-domain field is not required to express this boundary.

### Case: required-scope-cannot-be-faked-by-form

Given a machine Challenger declaring required semantic scope, when its scope report omits that scope or cannot map it, then the Challenger cannot claim complete coverage merely by naming mutation or source analysis.

## Replace claim: challenge-planning-binds-current-decisions
From: sha256:5642a3b38aeafd2b594753f3f2242271c1323cd86d319fbcd5aa827390f9ab94
Because: New machine targets are exact native Claim Review or Claim Health records and do not share an implicit Run context.
Criticality: routine

Azimuth SHALL expand authored machine Challenger declarations and requests from the complete current model into exact current eligible assessment targets, bounded work units and pinned implementation configuration.

### Case: decision-context-must-match-run

Given a selected assessment and an execution Subject, when a Challenge is planned, then planning validates the exact target and records the execution Subject separately. A Claim Review has no execution Subject to match, and a mutant execution Subject does not overwrite a Claim Health target's Subject.

### Case: challenge-only-and-mixed-plans-are-valid

Given a request containing only Challengers or Checks and Challengers, when planning succeeds, then it derives one nonempty provider-neutral semantic plan.

### Case: challenger-form-binds-explicit-capability

Given a machine Challenger selected for execution, when its plan is expanded, then planning pins its current declaration, implementation configuration and supported machine form and requires an explicit capability with the corresponding operation class and form.

### Case: target-cap-fails-before-truncation

Given target expansion exceeding a nonzero request cap, when planning checks the limit, then it fails without silently truncating the semantic selection.

## Replace claim: adapters-receive-frozen-semantic-scope
From: sha256:1672968becb21c65a9e9bb22d144d31c6b17d70f877b21fe8c0d10d8f50d85f1
Because: Standard realization and Mechanism markers plus pinned explicit additions resolve bounded machine source scope.
Criticality: routine

Azimuth SHALL freeze selected assessment identity, semantic source scope, source-resolution report and accountable launch inputs before an adapter translates a machine Challenger into provider-native work. Build dependencies SHALL remain distinct from challenged targets.

### Case: mutation-receives-realization-and-check-scope

Given a mutation Challenger selecting a Claim Review or Claim Health through a Claim's realization, when scope is prepared, then it includes the exact selected production sites, explicitly selected Check implementations and pinned configured additions. Compilation of another dependency does not mean that dependency was mutated.

### Case: fault-injection-receives-mechanism-and-check-scope

Given a request using a machine form outside the implemented mutation and source-analysis forms, when it is planned, then it is rejected rather than interpreted as one of those supported forms. Future fault-injection support requires its own explicit package and provider contract.

### Case: broad-analysis-receives-claim-composition

Given source analysis targeting a Claim assessment, when scope is prepared, then it receives the exact selected Claim composition and bounded concrete targets, with additions, exclusions, unresolved mappings and tool-induced expansion visible in its scope report.

### Case: scope-substitution-fails-publication

Given an adapter adding, removing or changing semantic scope or accountable launch inputs, when its response is validated, then the response is rejected before publishing a Run bundle.

### Case: tagged-sites-do-not-prove-scope-completeness

Given relevant helpers outside tagged sites, when scope is prepared, then explicit pinned configuration can include them without assigning misleading realization or Mechanism tags. Claim Review examines the resolved scope and its limitations; automatic dependency traversal is not required or silently assumed.

## Replace claim: challenge-outcomes-remain-negative-search-facts
From: sha256:681e1652c01f13ff21240960aa922e2d2787a79747469cdb1b14bc12597f8d28
Because: Machine objection searches never fabricate native assessments or product Observations.
Criticality: routine

Azimuth SHALL keep machine Challenge outcomes, scheduling deferral and selection mismatch as different states.

### Case: clean-is-not-product-evidence

Given a clean Challenge Result, when it is retained, then it records only that no objection was found in the declared search. It creates no product Observation, accepted Claim Review or supported Claim Health.

### Case: selected-inconclusive-work-remains-non-clean

Given selected Challenger work that cannot establish clean or findings, when its result is retained, then its outcome is inconclusive.

### Case: deferred-work-has-no-result

Given planned Challenger work omitted from an allowed incomplete Run, when the Run is published, then no Challenge Result is fabricated. The planned target remains outstanding and a diagnostic names the selection and deferral reason.

### Case: selection-mismatch-is-not-an-outcome

Given actual target, fingerprint, execution Subject, units or scope differing from the plan, when the exchange is validated, then it is rejected rather than published as a Challenge outcome.

## Replace claim: challenge-policy-is-form-and-cost-aware
From: sha256:c54c0b1d2a50096c53d14aa18bb712a27a55381a53a1a0d818c2fcab069767ef
Because: Required declared Challenger work and explicit resource bounds replace mandatory Decision Policies.
Criticality: routine

Azimuth SHALL distinguish declared machine objection searches and their execution requirements from negative search outcomes. Resource limits and selected scope SHALL remain accountable without requiring a Decision Policy record.

### Case: required-form-coverage-is-declared

Given a compact account with machine Challenger declarations, when its structural relationships are validated, then their Claim, Mechanism and Check references, supported forms and source implementation configuration are checked. Required workflow work is derived from explicit current declarations; a historical authority lacking declarations is not interpreted as an explicitly empty Challenger set.

### Case: scheduling-lane-does-not-stale-decision

Given the same pinned Challenger declaration and configuration, when execution is deferred without changing that definition, then deferral alone does not rewrite an assessment fingerprint or imply a clean search.

### Case: scheduled-omission-is-not-clean

Given selected Challenger work that does not execute, when current assurance is inspected, then it is not counted as a clean result.

### Case: temporal-reuse-remains-deferred

Given prior Challenge facts for another execution Subject or time, when current assurance is assessed, then this change supplies no implicit cache-validity or cross-Subject reuse inference.

## Replace claim: challenged-decision-impact-does-not-duplicate-results
From: sha256:351e37e15212da272d5a837b44bc9d85a80360e44792b766640ec3dc876157df
Because: Exact native assessment targets replace method/applicability fan-out without duplicating machine objections.
Criticality: routine
Remove cases: method-finding-fans-out-through-dependent-edges, applicability-finding-remains-local

Azimuth SHALL retain each machine objection as one exact Challenge Result and derive affected native assessment status without manufacturing additional results.

### Case: adverse-review-result-expires-dependent-health

Given an adverse Challenge Result against a current Claim Review, when dependent Claim Health is inspected, then it cannot remain supported through that review. The original target and result remain visible; no synthetic Claim Health Challenge Result is created.

### Case: direct-judgment-challenge-remains-distinct

Given an explicit Claim Health target, when its Challenger completes, then its result targets that exact Claim Health fingerprint rather than an inferred Claim Review or sibling health assessment.

### Case: several-edge-findings-deduplicate-claim-impact

Given several machine objections against one assessment, when assurance is inspected, then it exposes one impacted assessment with all distinct retained results. Their coexistence neither duplicates product Observations nor manufactures reviewer decisions.

### Case: reassessment-pins-inspected-objections

Given a reviewer reassessing after correction, when the new record is compiled, then it identifies its predecessor and exact inspected Challenge facts. Changed Run heads or result fingerprints make a dependent reassessment stale; historical objections remain inspectable without pretending they target the new fingerprint.
