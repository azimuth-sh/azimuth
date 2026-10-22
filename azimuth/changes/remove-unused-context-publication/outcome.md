# Outcome: Remove the unused collector context publication path

Status: accepted

## Delivered

`contexts.initialize`, `contexts.stage`, `contexts.publish`, the `context_outbox` table definition and its entry in the pending-capacity count are removed. The collector loop no longer calls `contexts.publish`.

## Departures

None.

## Verification performed

`contexts.py` and `collector.py` compile, and a search finds no remaining reference to `context_outbox` outside archived changes and this change. Per repository rules no tests were written or run. Existing state volumes keep an empty `context_outbox` table that nothing reads; it is not dropped.

## Residual decisions

None.
