# Design: assurance-project-snapshots

## Three different fingerprints

The project model fingerprint identifies one complete accepted Azimuth account. The claim-contract
fingerprint identifies the stable semantics against which an evidence definition is qualified. The
evidence-definition fingerprint identifies the executable proposition, form, oracle, inputs,
lifecycle stage and execution context.

The claim contract includes claim identity and predicate, criticality, effective verification
requirements, and relevant surface or area realization obligations. It excludes realization and
evidence source bodies, enumerated members, observations and judgments. Those belong either to the
repository agent tier or to the exact execution subject. This prevents every implementation edit
from forcing semantic requalification while still expiring it when the required assurance account
changes.

## Repository authority and service projection

`azimuth assurance export` runs the ordinary loader and checks before writing a snapshot. A
snapshot contains its own content-derived id, accepted model fingerprint and canonical claim
contracts. It is repository-authored authority transferred into the service; the service neither
parses specs nor recomputes workspace membership.

The service stores a snapshot immutably. Definitions use structured `{ spec, claim,
contractFingerprint }` references and are accepted only when that exact contract exists in a
registered snapshot. Observations and gate requests use an exact registered snapshot id.

## Applicability and reuse

At gate evaluation, the current definition must reference a contract present in the requested
snapshot. Otherwise the gate closes before execution selection. A definition may remain qualified
across two snapshots when both contain the same contract fingerprint. Exact subject equality still
requires a distinct observation for the new snapshot and revision.

An observation is rejected at ingestion when its snapshot does not exist or does not contain the
definition's contract. This keeps impossible provenance out of immutable history rather than merely
closing a later gate.

## Diagnostic metadata

Contracts carry surface id and obligated areas only as repository-derived diagnostic metadata.
The service displays them to explain qualification drift. It does not derive route populations,
area membership, realization completeness or evidence obligations.

## Compatibility boundary

This phase has one consumer and intentionally changes the reference protocol in place. Existing
seed data and tests are updated together; no backward-compatible unregistered-snapshot mode is
retained.
