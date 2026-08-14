# Outcome: reusable-evidence-qualification

Status: implemented, pending acceptance

## Result

The pre-registered lifecycle experiment passed all eight deterministic cases. A ninth temporal
regression was added when the evaluator became a reusable service domain crate. One qualification
opened merge gates for two different exact revisions without another semantic judgment or a
repository result commit. Production evidence remained confined to its artifact, deployment,
cohort and stage. Expiry, violation, definition drift, context mismatch and challenge findings
each closed the gate with focused work.

The result supports proceeding to the reference assurance service. Stable evidence definition and
qualification records form the semantic input; immutable observations form the execution input;
gate decisions and work items are derived outputs.

## Measurements

- pre-registered executable cases: 8 passed, 0 failed;
- promoted-domain temporal regression cases: 1 passed, 0 failed;
- semantic qualifications for two successful CI revisions: 1;
- repository writes caused by those execution results: 0;
- false cross-subject or cross-stage passes: 0;
- sleeps or wall-clock dependence: 0.

## Departures

None. The experiment remained a pure library with no persistence or network boundary.

## Residue carried into the service

- Persistent records need idempotent identity and immutable history.
- When several qualifications or challenges exist, the evaluator must select the latest applicable
  record deterministically rather than depend on collection order.
- The service must prove the same protocol through HTTP and real PostgreSQL without becoming
  authority for claim meaning.
