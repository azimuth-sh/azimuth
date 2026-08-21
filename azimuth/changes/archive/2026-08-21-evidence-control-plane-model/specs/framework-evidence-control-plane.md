# Intent delta: framework/evidence-control-plane

## Add requirement: claims-separate-normative-and-observable-intent
Criticality: routine

Azimuth SHALL distinguish a normative requirement-level Claim from its observable case-level
Claims without transferring criticality away from the normative proposition.

### Add scenario: observable-cases-remain-addressable
WHEN a requirement needs several independently observable conditions
THEN each condition remains an addressable case-level Claim beneath the requirement-level Claim

## Add requirement: checks-bind-atomic-outcomes-to-claims
Criticality: routine

Azimuth SHALL treat a Check as a deliberately enrolled verification method whose atomic outcome
bears on Claims only through explicit Evidence Bindings.

### Add scenario: one-check-supports-several-claims
WHEN one atomic Check outcome honestly bears on several Claim aspects
THEN each Check-to-Claim relationship is declared as a separate Evidence Binding

### Add scenario: independent-outcomes-remain-separate
WHEN assertions can vary independently even though one native command executes them
THEN Azimuth represents them as separate Checks

## Add requirement: qualifications-belong-to-evidence-bindings
Criticality: routine

Azimuth SHALL qualify each Evidence Binding independently from the whole-Claim assurance
composition and from recurring execution outcomes.

### Add scenario: shared-check-has-binding-specific-credibility
WHEN one Check binds to more than one Claim
THEN each Evidence Binding has its own Qualification

## Add requirement: runs-contain-subject-bound-execution
Criticality: routine

Azimuth SHALL represent a Run as a bounded execution envelope over an exact Subject that can
contain Check executions, Challenger executions or both.

### Add scenario: check-execution-produces-observation
WHEN a Check reaches a terminal outcome within a Run
THEN the Run contains one satisfied, violated or inconclusive Observation for that Check

### Add scenario: challenger-execution-produces-challenge-result
WHEN a Challenger reaches a terminal outcome within a Run
THEN the Run contains a Challenge Result rather than product evidence

## Add requirement: challenges-target-semantic-decisions
Criticality: routine

Azimuth SHALL target each Challenge Result at one exact Qualification or Claim Judgment and SHALL
propagate its impact through traceability without duplicating the result.

### Add scenario: direct-product-failure-remains-observation
WHEN an execution directly falsifies a product or operational Claim
THEN Azimuth records a violated Observation instead of reclassifying it as a Challenge Result

## Add requirement: core-owns-semantics-and-adapters-own-providers
Criticality: routine

Azimuth SHALL keep model traversal, semantic target selection and normalized-result validation in
core while configured adapters own provider-native selection, execution and report import.

### Add scenario: adapter-reports-actual-selection
WHEN an adapter executes or imports a provider-native operation
THEN it reports actual selection with normalized outcomes
AND it does not interpret the repository model independently
