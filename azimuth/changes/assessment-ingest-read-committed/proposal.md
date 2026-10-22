# Change: Ingest deployment context captures without snapshot isolation

Status: accepted and complete

Intent delta: none
Because: the change adjusts transaction isolation and statement batching inside the Assurance State Service; no accepted Requirement of the framework changes.

## Problem

Production produced 152 `40001 could not serialize access due to concurrent update` errors in twelve minutes while a release was published. Each came from the deferred `invalidate_assessment_event` trigger, which locks every deployment assessment of the project with `SELECT ... FOR UPDATE` at commit of a context-capture ingest. That ingest ran at REPEATABLE READ, so the lock failed whenever the assessment worker had published since the transaction's first statement, and the collector's capture was rejected. The worker's publication transaction also issued one statement per dependency and per Scenario result, holding its row locks for seconds.

## Outcome

The capture ingest runs at READ COMMITTED, the isolation the trigger already handles by finishing invalidation immediately. Publication writes each Requirement's dependencies and Scenario results in one statement each, keeping the same results as sequential upserts.

## Scope

In scope: the isolation level of `context-captures` ingest and the batched dependency and Scenario writes in `materialized::publish`. Excluded: the trigger's lock scope, the reconciler interval, the unused collector `contexts.publish` path.

## Completion conditions

- No ingest transaction raises `40001` from the invalidation trigger.
- Publication results are unchanged for duplicate bindings (the last result wins) and unchanged rows are not rewritten.
