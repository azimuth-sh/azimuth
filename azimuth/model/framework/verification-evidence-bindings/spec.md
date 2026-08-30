# Spec: framework/verification-evidence-bindings

## Claim: checks-bind-through-explicit-edges
Criticality: routine

Azimuth SHALL represent a Check as one atomic terminal proposition with sparse explicit Evidence
Bindings to normative Cases.

### Case: one-check-binds-several-cases
- Context: one Check whose terminal result bears on several product propositions
- Event: the repository declares an Evidence Binding for each relevant Case
- Required outcome: the Check has one identity and each Case edge remains independently addressable

### Case: one-case-receives-several-checks
- Context: several Checks bear on different observable aspects of one Case
- Event: the repository declares one Evidence Binding for each Check-to-Case pair
- Required outcome: the parent Claim composes the independent edges without merging their outcomes

### Case: implementation-does-not-declare-evidence
- Event: source code implements an enrolled Check
- Required outcome: its marker names only the Check identity
- Additional condition or outcome: Case, form and decision meaning remain repository declarations

## Claim: method-and-applicability-decisions-have-exact-boundaries
Criticality: routine

Azimuth SHALL assign one exact Method Qualification to shared method inputs and one exact
Applicability Decision to every Evidence Binding, expiring only decisions whose inputs change.

### Case: one-applicability-decision-per-binding
- Context: an Evidence Binding names one Check, one Case and one Method Qualification
- Event: the binding is applicable
- Required outcome: one Applicability Decision with the binding identity records its verdict and fingerprint

### Case: semantic-change-expires-dependent-decisions
- Context: current method and applicability decisions
- Event: one fingerprinted method or edge input changes
- Required outcome: validation reports each affected decision as stale
- Additional condition or outcome: does not stale an unrelated applicability edge

### Case: routine-claim-rejects-verification
- Context: a Case whose parent Claim is routine
- Event: a verification declaration targets that Case or Claim
- Required outcome: validation reports inapplicable verification instead of granting optional evidence status

## Claim: challenge-plans-resolve-through-traceability
Criticality: routine

Azimuth SHALL resolve Challenge Plan selectors through stable model relations to exact current
Method Qualification, Applicability Decision or Claim Judgment fingerprints.

### Case: selectors-compose-deterministically
- Context: selectors that reach overlapping decisions through several stable relations
- Event: Azimuth resolves the Challenge Plan
- Required outcome: it returns the sorted deduplicated union of exact decision fingerprints

### Case: empty-selection-fails-closed
- Event: a Challenge Plan selector resolves no current decision
- Required outcome: validation reports the unresolved selector
- Additional condition or outcome: execution planning does not substitute a broad or whole-suite target

### Case: paths-are-not-semantic-selectors
- Event: a Challenge Plan attempts to select by source path, line or glob
- Required outcome: parsing rejects the selector

## Claim: check-linkage-is-provider-neutral
Criticality: routine

Azimuth SHALL normalize Check implementation linkage independently of providers and SHALL reject
every alpha 1 evidence input.

### Case: several-sites-compose-one-check
- Context: one Check implemented by several source sites
- Event: extractors emit their stable semantic identities and source fingerprints
- Required outcome: Azimuth derives one ordered implementation set for the Check

### Case: ordinary-tests-remain-outside-azimuth
- Event: a native test has no ImplementsCheck marker
- Required outcome: extractors emit no Check implementation
- Additional condition or outcome: Azimuth assigns it no assurance meaning

### Case: alpha-one-input-fails
- Event: a repository or manifest input uses an alpha 1 evidence construct
- Required outcome: the responsible parser rejects it without translation
