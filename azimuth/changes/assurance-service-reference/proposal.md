# Change: assurance-service-reference

Status: implemented, pending acceptance

Exploration: continuous-assurance-service
Carries decisions: E1, E2, E3, E4, E5, E7; D40

## Problem

The successful lifecycle experiment has no durable or interoperable boundary. It cannot yet accept
records from a CI runner or monitoring adapter, preserve immutable execution history, evaluate an
exact lifecycle target after process restart, or show an engineer why a gate is closed. Freezing a
larger platform before proving those mechanics over real persistence would repeat the risk that the
in-memory experiment removed.

## Outcome

An open reference application exposes the validated qualification/observation protocol through an
Axum HTTP API backed by PostgreSQL. It stores immutable, idempotent records; derives and preserves
gate decisions and focused work items for exact subjects; exports a project snapshot; and provides
a small Next.js diagnostic interface. It runs locally as independent backend and frontend
processes and remains optional to the repository-only Azimuth workflow.

## Scope

In scope:

- a reusable Rust domain crate containing the deterministic evaluator;
- project, definition, qualification, observation and challenge ingestion;
- append-only record versions, idempotent replay and conflicting-identity refusal;
- current gate evaluation plus immutable decision history and derived worklist;
- portable project snapshot and a read-only diagnostic web interface;
- Docker Compose packaging and component evidence through HTTP with real PostgreSQL;
- merge and canary lifecycle examples that reproduce the experiment's surviving boundary.

Out of scope:

- authentication, organizational tenancy and hosted-service billing;
- a CI runner, deployment orchestrator, telemetry store or report-blob store;
- automatic repository edits, issue creation or acceptance decisions;
- cryptographic signing, retention policy, high availability and production SLOs;
- changing the current Azimuth manifest or making the service mandatory for routine work.

## Affected claims

None in the ride-hailing reference model. This change validates optional framework infrastructure
against a synthetic assurance project.

## Completion conditions

- The service reproduces qualification reuse, exact-subject confinement, expiry, violation,
  definition drift, context mismatch and challenge behavior over HTTP and PostgreSQL.
- Replaying an identical immutable record succeeds without duplication; reusing its id for
  different content is refused.
- A gate decision records the definition fingerprint, qualification and observation that justify
  it, or every deterministic reason and focused work item that closes it.
- Process-independent reads expose immutable history, current gate state, current work and a
  portable project snapshot.
- The web interface displays state and reasons without introducing a second evaluator.
- Local startup and verification are documented and executable; the standalone CLI remains
  unaffected when the service is absent.
