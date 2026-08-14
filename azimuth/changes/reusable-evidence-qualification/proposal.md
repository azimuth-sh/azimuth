# Change: reusable-evidence-qualification

Status: implemented, pending acceptance

Exploration: continuous-assurance-service
Carries decisions: E2, E3, E5

## Problem

Current judgments fingerprint every assurance observation. This is correct for one-time evidence,
but a recurring CI or production execution changes observation identity even when the qualified
test, claim binding, oracle and execution conditions have not changed. Treating every successful
rerun as new semantic evidence would require agent re-judgment or repository churn and make
continuous assurance impractical.

## Outcome

A provider-neutral prototype separates a stable evidence definition and its agent qualification
from immutable execution observations. The evaluator derives lifecycle gates for exact subjects.
Repeated applicable successes reuse the qualification; definition drift, failure, expiry,
challenge findings and context mismatch create precise work instead of silently inheriting trust.

## Scope

In scope:

- a framework-independent protocol prototype for definitions, qualifications, subjects,
  observations, gates and feedback;
- one CI and one production lifecycle stage;
- two successful CI executions over different revisions, a production execution, expiry,
  violation, definition drift and context mismatch;
- measurement of judgment and repository churn relative to current observation fingerprinting;
- a decision on whether the result justifies the reference assurance service.

Out of scope:

- an HTTP server, database, web interface, authentication or tenancy;
- changing D39 or the accepted Rust manifest before the experiment settles the distinction;
- native CI, TestOps, deployment or monitoring integrations;
- accepting an unexecuted critical assertion as satisfied.

## Affected claims

None in the ride-hailing model. This change tests the Azimuth protocol against a synthetic
assurance lifecycle.

## Completion conditions

- One stable qualification is reused by at least two successful observations without becoming
  stale and without changing a repository artifact.
- A successful observation gates only the exact revision or artifact and lifecycle stage to which
  it applies.
- An expired or violated observation closes its gate with a deterministic reason.
- A changed definition invalidates its qualification before any observation can open a gate.
- A challenge finding or incompatible execution context creates agent work rather than a false
  pass.
- The experiment is executable, deterministic and fails if a falsifier is reintroduced.
- The outcome records whether to proceed with the service and which protocol boundary survived.
