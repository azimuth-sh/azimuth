# Change: assurance-project-snapshots

Status: implemented, pending acceptance

## Problem

The assurance service accepts `projectSnapshot` as an unchecked string and identifies an evidence
definition's claim with another unchecked string. Its qualification fingerprint therefore cannot
distinguish a stable execution definition from a changed claim contract such as a new realization
obligation or surface assignment. A producer can submit observations for an unknown snapshot, and
a prior qualification can remain apparently current after the repository changes the architectural
account that made that evidence adequate.

## Outcome

The Azimuth CLI exports an immutable assurance project snapshot containing the accepted model
fingerprint and per-claim assurance contracts. The service registers those snapshots idempotently,
requires evidence definitions and execution subjects to reference registered contracts and
snapshots, and closes a gate when its definition is not applicable to the requested snapshot.

Claim contracts include durable claim and verification semantics plus relevant surface and area
obligations. They exclude realization source bodies and enumerated members, so a source revision
requires a new observation but does not manufacture semantic requalification when the definition
and contract are unchanged.

## Scope

In scope:

- deterministic local snapshot export from a hole-free accepted Azimuth model;
- structured claim identity and a canonical per-claim assurance-contract fingerprint;
- immutable project-snapshot ingestion and discovery in the assurance service;
- validation that definitions name a claim contract in at least one registered snapshot;
- exact snapshot applicability during observation ingestion and gate evaluation;
- diagnostic display of model identity, claim contract, surface and obligated areas;
- pure and PostgreSQL-backed evidence for drift, unknown snapshots and unchanged-contract reuse.

Out of scope:

- moving workspace parsing, enumerators or realization checks into the service;
- cryptographic producer authentication or signed provenance;
- automatic network publication by `azimuth check`;
- federated snapshot conversion in this first local slice;
- per-area evidence definitions or one test per obligated area;
- changing ride-hailing product intent.

## Affected claims

None in the ride-hailing model. This change strengthens the optional assurance protocol using
synthetic framework and service accounts.

## Completion conditions

- A hole-free model exports a deterministic snapshot with one contract for every non-routine claim.
- Claim-contract fingerprints change when claim, verification, surface or area obligations change,
  but not when set-like declarations are reordered or realization source bodies change.
- The service rejects definitions for unknown contracts and observations for unknown or
  inapplicable snapshots.
- A gate for a snapshot where the definition's contract is inapplicable closes with focused
  definition-revision and requalification work before considering an otherwise successful
  observation.
- The same qualified definition may serve another registered snapshot when its claim contract and
  definition semantics are unchanged, while the new exact subject still requires its own
  observation.
- The browser exposes imported snapshot and contract provenance without evaluating it independently.
- CLI, domain, HTTP/PostgreSQL, web and complete repository checks pass.
