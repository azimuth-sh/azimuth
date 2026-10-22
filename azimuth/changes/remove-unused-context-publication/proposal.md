# Change: Remove the unused collector context publication path

Status: accepted and complete

Intent delta: none
Because: the change deletes collector code that no longer runs; no accepted Requirement of the framework changes.

## Problem

Deployment context captures reach ASS as target observations through `target_outbox`. The earlier `context_outbox` table, `contexts.stage` and `contexts.publish` still existed: nothing staged into the table, so `contexts.publish` polled an always-empty table every cycle, and the capacity check counted it.

## Outcome

The collector has one path for context captures. `contexts.initialize`, `contexts.stage`, `contexts.publish`, the `context_outbox` table definition and its use in the pending-capacity count are gone.

## Scope

In scope: `services/collector/contexts.py`, `services/collector/collector.py` and the collector README sentence describing the outboxes. Excluded: the `outbox` and `target_outbox` paths, the invalidation trigger and the reconciler interval.

## Completion conditions

- [x] No collector code references `context_outbox`.
- [x] A collector state volume that already holds an empty `context_outbox` table starts and runs unchanged; the table is no longer created or read.
