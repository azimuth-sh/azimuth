# Outcome: assurance-service-reference

Status: accepted

## Result

The qualification/observation split survived both persistence and network boundaries. The pure
evaluator is now a reusable serialized Rust crate. An Axum API stores project-scoped definition
versions, qualifications, observations and producer-scoped challenge streams in PostgreSQL;
derives exact-subject lifecycle gates; preserves ordered decision history; and exposes current
gates, focused work and a portable project snapshot. A Next.js client renders only those
projections. Docker Compose runs the complete local application.

The staged validation succeeded, so implementation proceeded from the initial pure experiment to
the complete reference slice authorized by the proposal.

## Evidence executed

- Eleven current pure lifecycle cases passed, including qualification reuse, exact-subject and
  temporal confinement, expiry, violation, definition and claim-contract drift, context mismatch,
  challenge findings and unknown-snapshot registration work.
- The promoted domain wire-contract regression passed.
- One HTTP component lifecycle passed against a real PostgreSQL Testcontainer. It exercised
  idempotent replay and identity conflict, two exact CI revisions, subject and context mismatch,
  failure, challenge finding and resolution, canary expiry, definition drift, decision order,
  current work and reconstruction from persisted state.
- The Rust workspace compiled across all targets with no warnings.
- The Next.js client passed strict type-checking and a production build; its project route is
  server-rendered and consumes the API as its only decision source.
- Both production Docker images built. The isolated Compose stack migrated, reported a healthy API,
  accepted the replay-safe seed twice, returned the seeded open gate and rendered the same state
  through the Next.js route. Its containers, network and database volume were removed afterward.
- The canonical accepted model contained 2 routine claims in 1 spec with zero holes, errors or
  warnings. `azimuth change check` reported zero incomplete plan items and zero accepted-state
  findings.

The earlier 9-case evaluator count and 90-claim repository check remain implementation provenance
from the source repository before extraction. Acceptance on 2026-08-14 reran the current protocol,
HTTP/PostgreSQL component and Compose runbook from the standalone canonical checkout.

## Departures

The original change map described a local ledger before a dashboard. After the persistent protocol
passed, the user explicitly authorized proceeding to the full service. The completed reference
slice therefore includes the diagnostic web client and Compose packaging, but still excludes the
production-hardening change from that map.

Gate history uses a database sequence in addition to UUID identity. Wall-clock seconds alone were
not a sufficient order because several decisions may be evaluated within one second.

## Residual decisions

- Authenticate producers and readers; isolate organizations and projects.
- Bind semantic inputs and execution receipts to verifiable source and artifact provenance.
- Define report-object storage, retention, privacy, backup and recovery.
- Add rate limits, service metrics, traces, alerting, SLOs and availability design.
- Build one real CI adapter and one real production-observation adapter, then test incomplete and
  dishonest subject metadata.
- Decide whether repository feedback is an issue, a draft Azimuth change or a CLI worklist import;
  the reference service only exposes work through its API.

## Framework result

D40 remains supported. Stable semantic qualification can be reused while execution facts change,
without weakening exact applicability or moving claim authority into the service. The result is a
convincing reference implementation of the protocol, not evidence of production readiness or
organizational adoption.
