# Outcome: run-bundle-protocol

Status: accepted

## Result

Azimuth now has one strict, provider-neutral Run bundle version 1. One bundle binds an exact typed
Subject to an explicit semantic plan, actual selection, physical activities, normalized attempts,
one terminal Observation per selected Check and one exact Challenge Result per selected
Challenger target. Negative, partial and cancelled execution facts remain valid protocol accounts.

Canonical RFC 8785 JSON and SHA-256 envelopes identify Subjects, plans, selections, results,
bundles and immutable correction chains. `azimuth run verify` validates one bundle or a correction
set, while `azimuth run inspect` emits the same deterministic account as text or JSON. Neither
command invokes a provider, resolves current repository decisions, ingests a ledger record or
projects Assurance State.

## Acceptance checks

- The complete Rust CLI suite passed 176 tests, including 24 Run-kernel cases and seven CLI cases
  for strict schema handling, Subjects, fingerprints, selection, reduction, references,
  correction histories, output parity and exit classes.
- The standalone Run conformance gate passed its six Subject forms, imported provenance,
  multi-Run aggregation, retry and shard reduction, dual-role activity, partial execution,
  correction, mismatch and malformed-input cases.
- The canonical root gate passed all extractors, the polyglot and assurance-extension experiments,
  the new Run gate, the Assurance Service and web build, deployment and lifecycle tests, all 66
  release tests and the composed model validation.
- Release isolation discovered 11 experiment roots and 46 executable inputs, including the three
  Run-conformance files and its root-gate invocation.
- Five release manifests validated 68 Claims across eight specs with zero Findings. Traceability
  selected all 15 new case Claims, and the selected export was version 2 with no retired evidence
  collections.
- Every one of the 26 current framework requirements is routine. The model contains no
  `verification.md`, Qualification or Claim Judgment facet created for ordinary engineering tests.
- The hosted ordinary root gate passed for pull request 13 at revision
  `f649620e9a96620e94f183b6c5a1b2065ee28f6b`. The release rehearsal completed its package, three
  native and assurance-web jobs; its assurance-api multi-architecture image job remained in
  progress when this outcome was recorded and is not change acceptance authority.
- Formatting, work-package dependency, public-link, skill-frontmatter, current-command,
  all-routine, no-compatibility and prohibited-name audits passed.

## Departures

- Contract review replaced an underspecified canonicalization rule with exact RFC 8785 byte
  serialization and literal versioned fingerprint envelopes.
- A selected Check originally could name only a subset of its planned implementations. The final
  contract requires the complete implementation set, preventing partial execution from being
  reduced to a satisfied Observation.
- Challenge cardinality now makes both plan-local ids and semantic Challenger-target tuples unique,
  while result identity retains the plan-local id needed to distinguish the selected execution.
- Kernel review found that a sparse maximum-safe correction revision could cause an effectively
  unbounded gap loop. Correction validation now walks only adjacent present revisions.
- Claim Judgment target ids, semantic implementation identities, empty string-map values, integral
  JSON number spellings and programmatically constructed unsafe integers received explicit
  validation after adversarial review.
- Adding the experiment changed the root gate and release-isolation account. All stale active
  hosted receipts were retired; immutable historical copies remain in their accepted archives.

## Residual decisions

- Adapter capability discovery, plan generation, provider invocation, native selector translation,
  timeout handling and normalized import belong to the dependent adapter-protocol change.
- Resolving authored Challenge Plans to current Qualification or Claim Judgment fingerprints and
  matching policy forms to provider capabilities remain dependent challenge-planning work.
- Durable ingest, authorization, idempotency, correction acceptance, revocation, retention and
  Assurance State require the atomic Assurance Service ledger replacement.
- Event gateways and long-running monitoring ingress remain separate from the adapter process
  protocol and the ledger.
- Package versions and release authority intentionally remain at alpha 1. The coordinated
  `alpha2-release` change owns the one-time version transition, fresh hosted execution accounts and
  publication; this change does not publish altered artifacts under the immutable alpha 1 version.
