# Spec: framework/verification-evidence-bindings

## Claim: checks-bind-through-explicit-edges
Criticality: routine

Azimuth SHALL represent a Check as one atomic terminal proposition with sparse explicit Evidence
Bindings to normative Cases.

### Case: one-check-binds-several-cases
GIVEN one Check whose terminal result bears on several product propositions
WHEN the repository declares an Evidence Binding for each relevant Case
THEN the Check has one identity and each Case edge remains independently addressable

### Case: one-case-receives-several-checks
GIVEN several Checks bear on different observable aspects of one Case
WHEN the repository declares one Evidence Binding for each Check-to-Case pair
THEN the parent Claim composes the independent edges without merging their outcomes

### Case: implementation-does-not-declare-evidence
WHEN source code implements an enrolled Check
THEN its marker names only the Check identity
AND Case, form and decision meaning remain repository declarations

## Claim: method-and-applicability-decisions-have-exact-boundaries
Criticality: routine

Azimuth SHALL assign one exact Method Qualification to shared method inputs and one exact
Applicability Decision to every Evidence Binding, expiring only decisions whose inputs change.

### Case: one-applicability-decision-per-binding
GIVEN an Evidence Binding names one Check, one Case and one Method Qualification
WHEN the binding is applicable
THEN one Applicability Decision with the binding identity records its verdict and fingerprint

### Case: semantic-change-expires-dependent-decisions
GIVEN current method and applicability decisions
WHEN one fingerprinted method or edge input changes
THEN validation reports each affected decision as stale
AND does not stale an unrelated applicability edge

### Case: routine-claim-rejects-verification
GIVEN a Case whose parent Claim is routine
WHEN a verification declaration targets that Case or Claim
THEN validation reports inapplicable verification instead of granting optional evidence status

## Claim: challenge-plans-resolve-through-traceability
Criticality: routine

Azimuth SHALL resolve Challenge Plan selectors through stable model relations to exact current
Method Qualification, Applicability Decision or Claim Judgment fingerprints.

### Case: selectors-compose-deterministically
GIVEN selectors that reach overlapping decisions through several stable relations
WHEN Azimuth resolves the Challenge Plan
THEN it returns the sorted deduplicated union of exact decision fingerprints

### Case: empty-selection-fails-closed
WHEN a Challenge Plan selector resolves no current decision
THEN validation reports the unresolved selector
AND execution planning does not substitute a broad or whole-suite target

### Case: paths-are-not-semantic-selectors
WHEN a Challenge Plan attempts to select by source path, line or glob
THEN parsing rejects the selector

## Claim: check-linkage-is-provider-neutral
Criticality: routine

Azimuth SHALL normalize Check implementation linkage independently of providers and SHALL reject
every alpha 1 evidence input.

### Case: several-sites-compose-one-check
GIVEN one Check implemented by several source sites
WHEN extractors emit their stable semantic identities and source fingerprints
THEN Azimuth derives one ordered implementation set for the Check

### Case: ordinary-tests-remain-outside-azimuth
WHEN a native test has no ImplementsCheck marker
THEN extractors emit no Check implementation
AND Azimuth assigns it no assurance meaning

### Case: alpha-one-input-fails
WHEN a repository or manifest input uses an alpha 1 evidence construct
THEN the responsible parser rejects it without translation
