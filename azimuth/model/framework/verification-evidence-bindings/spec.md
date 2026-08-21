# Spec: framework/verification-evidence-bindings

## Requirement: checks-bind-through-explicit-edges
Criticality: routine

Azimuth SHALL represent a Check as one atomic terminal proposition with sparse explicit Evidence
Bindings to case-level Claims.

### Scenario: one-check-binds-several-claims
GIVEN one Check whose terminal result bears on several product propositions
WHEN the repository declares an Evidence Binding for each relevant case-level Claim
THEN the Check has one identity and each Claim edge remains independently qualified

### Scenario: one-claim-receives-several-checks
GIVEN several Checks bear on different observable aspects of one case-level Claim
WHEN the repository declares one Evidence Binding for each Check-to-Claim pair
THEN the Claim composes the independent edges without merging their outcomes

### Scenario: implementation-does-not-declare-evidence
WHEN source code implements an enrolled Check
THEN its marker names only the Check identity
AND Claim, form and Qualification meaning remain repository declarations

## Requirement: qualifications-are-exact-binding-decisions
Criticality: routine

Azimuth SHALL assign exactly one context-bound Qualification to each applicable Evidence Binding and
SHALL expire that decision when any credibility input changes.

### Scenario: one-qualification-per-binding
GIVEN an Evidence Binding names one Check and one case-level Claim
WHEN the binding is applicable
THEN one Qualification with the binding identity records its verdict and fingerprint

### Scenario: semantic-change-expires-qualification
GIVEN a current Qualification
WHEN a Check, Claim, binding, policy or required-context credibility input changes
THEN validation reports the Qualification as stale

### Scenario: routine-claim-rejects-verification
GIVEN a case-level Claim whose requirement is routine
WHEN a verification declaration targets that Claim
THEN validation reports inapplicable verification instead of granting optional evidence status

## Requirement: challenge-plans-resolve-through-traceability
Criticality: routine

Azimuth SHALL resolve Challenge Plan selectors through stable model relations to exact current
Qualification or Claim Judgment fingerprints.

### Scenario: selectors-compose-deterministically
GIVEN selectors that reach overlapping decisions through several stable relations
WHEN Azimuth resolves the Challenge Plan
THEN it returns the sorted deduplicated union of exact decision fingerprints

### Scenario: empty-selection-fails-closed
WHEN a Challenge Plan selector resolves no current decision
THEN validation reports the unresolved selector
AND execution planning does not substitute a broad or whole-suite target

### Scenario: paths-are-not-semantic-selectors
WHEN a Challenge Plan attempts to select by source path, line or glob
THEN parsing rejects the selector

## Requirement: check-linkage-is-provider-neutral
Criticality: routine

Azimuth SHALL normalize Check implementation linkage independently of providers and SHALL reject
every alpha 1 evidence input.

### Scenario: several-sites-compose-one-check
GIVEN one Check implemented by several source sites
WHEN extractors emit their stable semantic identities and source fingerprints
THEN Azimuth derives one ordered implementation set for the Check

### Scenario: ordinary-tests-remain-outside-azimuth
WHEN a native test has no ImplementsCheck marker
THEN extractors emit no Check implementation
AND Azimuth assigns it no assurance meaning

### Scenario: alpha-one-input-fails
WHEN a repository or manifest input uses an alpha 1 evidence construct
THEN the responsible parser rejects it without translation
