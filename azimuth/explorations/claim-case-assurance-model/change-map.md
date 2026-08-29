# Candidate change map: Claim/Case assurance model

Exploration: claim-case-assurance-model
Status: candidate dependency graph; no change authorized

## Dependency graph

```text
C1 Claim/Case ontology
 |-- C2 Claim-level realization and Mechanism composition
 `-- C3 Case-directed Evidence Bindings
       `-- C4 Split credibility decisions
             `-- C5 Challenger policies, targets and impact
                   `-- C6 Run selection and result projection

C1 + C2 + C3 + C4 + C5 + C6
 `-- C7 Fail-closed brownfield migration

All settled changes
 `-- C8 Derived framework guidance and worked examples
```

## C1 - Claim/Case ontology

Define Claim identity independently of OpenSpec Requirement and Scenario. Define normative nested
Cases, stable addresses, digest contribution and the Case-to-Claim promotion boundary.

This is the root dependency because every later relation needs the new target identities.

## C2 - Claim-level realization and Mechanism composition

Move production realization and total Mechanism composition to Claim level while retaining exact
Case-relevance information where it contributes to review. Preserve sparse implementation markers
and standalone reusable Mechanism identity.

Depends on C1. It may proceed independently of C3 after the Claim/Case identity is fixed.

## C3 - Case-directed Evidence Bindings

Make Cases evidence and Run-selection targets. Preserve many-to-many Check relationships,
project-global Check identity and Check-only implementation markers.

Depends on C1. It establishes the edge consumed by C4.

## C4 - Split credibility decisions

Introduce Method Qualification and per-binding Applicability Decision. Define their verdicts,
identity preimages, context partition, staleness and contribution to total Claim Judgment.

Depends on C3 and on the Case digest account from C1.

## C5 - Challenger policies, targets and impact

Define Decision Policies, required forms, selectors, semantic scopes and impact propagation for the
two credibility propositions. Preserve exact adverse targets and forbid inferred escalation or
fabricated sibling results.

Depends on C4. Mechanism- and realization-based traversal also consumes C2.

## C6 - Run selection and result projection

Deduplicate shared Check execution while preserving Case coverage, exact Observations, adverse
provenance, diagnostics and parent Claim impact. Decide the protocol account for one result serving
several Cases.

Depends on C3, C4 and C5.

## C7 - Fail-closed brownfield migration

Provide a one-way transition that reuses authored rationale and provenance, generates no accepted
decision, preserves historical fingerprints and Results, and reports every ambiguous context or
grouping instead of guessing.

Depends on the settled identities and semantics of C1 through C6.

## C8 - Derived framework guidance and worked examples

Update derived framework prose and add synthetic framework-owned examples after the behavioral
changes settle. External domain fixtures remain outside this repository and never become executable
Azimuth dependencies.

Depends on every change whose behavior it describes.

## Prior exploration disposition

The preserved `scenario-bound-mechanisms` exploration is not edited by this candidate graph. Likely
downstream dispositions are:

- revise its scenario-only assurance target to Claim-level realization and Mechanism composition
  plus Case-directed evidence;
- retain reusable Mechanism identity, relation-specific role and sparse Mechanism implementation
  linkage;
- retain rejection of a separate Mechanism Qualification;
- revise atomic Mechanism Binding to terminate at Claim rather than scenario while carrying exact
  Case relevance where necessary; and
- replace complete per-binding Qualification with the split decisions from C4.

Those are candidate consequences. Each future proposal must name the exact earlier decisions it
supersedes and preserve the earlier exploration as reasoning history.
