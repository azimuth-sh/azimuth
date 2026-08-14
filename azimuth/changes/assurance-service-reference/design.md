# Design: assurance-service-reference

## Authority boundary

The service receives repository-authored evidence definitions and agent qualifications as signed-or-
otherwise-authenticated inputs in a future production deployment. This reference implementation
does not authenticate them, so it never claims to be production authority. It preserves their
fingerprints and derives state without editing their meaning. Observations and challenges are
immutable execution facts. Gate decisions and work items are reproducible projections.

## Record identity and history

Each submitted record has a producer-supplied id scoped to a project. The service computes a
canonical content fingerprint. Replaying the same id and content is idempotent; submitting changed
content under that id returns a conflict. Evidence definitions are versioned by semantic
fingerprint so a changed definition leaves history intact and becomes the current head.

Qualifications and challenges carry observation time. The evaluator chooses the latest applicable
record deterministically, avoiding collection-order semantics. Observations remain confined by
definition fingerprint, lifecycle stage and the complete execution subject.

## Derived state

`POST .../gates/evaluate` loads one consistent project account, invokes the pure evaluator and
stores the decision with its request and evaluation time. Reads expose decision history, the latest
decision for each target and work derived from closed decisions. The browser consumes these APIs;
it contains no independent gate rules.

## Deployment shape

The reference package has three replaceable parts: a pure Rust domain crate, an Axum/PostgreSQL
server, and a Next.js diagnostic client. Docker Compose supplies PostgreSQL and both processes for
local evaluation. Provider adapters can call the HTTP protocol without linking Rust.

## Residual production design

Authentication, tenant isolation, signatures and provenance, retention, report-object storage,
backup, rate limits, observability and availability objectives remain explicit hardening work.
Their absence is acceptable only for the local reference application.
