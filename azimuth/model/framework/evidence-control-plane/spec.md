# Spec: framework/evidence-control-plane

## Requirement: claims-own-governance-and-cases-own-addressability
Title: Requirements own governance and scenarios own addressability

Azimuth SHALL govern one product proposition as a Requirement while retaining each normative
condition/outcome clause as an addressable Scenario within that Requirement.

### Scenario: observable-cases-remain-addressable
- Event: a Requirement needs several materially different conditions
- Required outcome: each condition remains an addressable Scenario within the Requirement
- Additional condition or outcome: only the Requirement owns criticality, production realization and total Judgment

## Requirement: normative-text-remains-author-owned
Title: Normative text remains author owned

Requirement and Scenario bodies remain authored normative content whose meaning belongs to human and agent
review rather than interpretation by Azimuth core.

### Scenario: free-form-markdown-is-preserved

Azimuth accepts any non-empty free-form Markdown body, preserves its internal line structure and
includes it in the addressed Requirement or Scenario fingerprint.

### Scenario: notation-has-no-implicit-semantics

Natural-language keywords, translations, tables, diagrams, code fences and domain notations do not
gain parser-defined meaning merely by appearing in a normative body.

## Requirement: checks-bind-atomic-outcomes-to-cases
Title: Checks bind atomic outcomes to scenarios

Azimuth SHALL treat a Check as a deliberately enrolled verification method whose atomic outcome
bears on Scenarios only through explicit Evidence Bindings.

### Scenario: one-check-supports-several-cases
- Event: one atomic Check outcome honestly bears on several Scenarios
- Required outcome: each Check-to-Scenario relationship is declared as a separate Evidence Binding

### Scenario: independent-outcomes-remain-separate
- Event: assertions can vary independently even though one native command executes them
- Required outcome: Azimuth represents them as separate Checks

## Requirement: credibility-separates-method-from-applicability
Title: Credibility separates method from applicability

Azimuth SHALL qualify shared Check-method credibility independently from the applicability of that
method to each Evidence Binding.

### Scenario: shared-method-does-not-repeat-review
- Event: one Check and common method context serve more than one Scenario
- Required outcome: one Method Qualification may support several Evidence Bindings
- Additional condition or outcome: every binding retains its own Applicability Decision

## Requirement: runs-contain-subject-bound-execution
Title: Runs contain verification target bound execution

Azimuth SHALL represent a Run as a bounded execution envelope over an exact Verification Target that can
contain Check executions, Verification Challenge executions or both.

### Scenario: check-execution-produces-observation
- Event: a Check reaches terminal outcomes within a Run
- Required outcome: the Run contains one satisfied, violated or inconclusive Check Result for every selected Scenario

### Scenario: challenger-execution-produces-challenge-result
- Event: a Verification Challenge reaches a terminal outcome within a Run
- Required outcome: the Run contains a Challenge Result rather than product evidence

## Requirement: challenges-target-semantic-decisions
Title: Challenges target semantic decisions

Azimuth SHALL target each Challenge Result at one exact Method Qualification, Applicability
Decision or Requirement Judgment and SHALL propagate its impact through traceability without duplicating
the result.

### Scenario: direct-product-failure-remains-observation
- Event: an execution directly falsifies a product or operational Requirement
- Required outcome: Azimuth records a violated Check Result instead of reclassifying it as a Challenge Result

## Requirement: core-owns-semantics-and-adapters-own-providers
Title: Core owns semantics and adapters own providers

Azimuth SHALL keep model traversal, semantic target selection and normalized-result validation in
core while configured adapters own provider-native selection, execution and report import.

### Scenario: adapter-reports-actual-selection
- Event: an adapter executes or imports a provider-native operation
- Required outcome: it reports actual selection with normalized outcomes
- Additional condition or outcome: it does not interpret the repository model independently
