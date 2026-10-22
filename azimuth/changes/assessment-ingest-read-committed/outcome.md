# Outcome: Ingest deployment context captures without snapshot isolation

Status: accepted

## Delivered

`context-captures` ingest no longer sets REPEATABLE READ. `materialized::publish` writes dependencies and Scenario results with one `UNNEST` statement each per Requirement.

## Departures

None.

## Verification performed

The server crate builds. The two batched statements were run against a throwaway Postgres 17 with the migration tables: duplicate dependencies collapse, an unchanged Scenario row keeps its generation, a changed one takes the new generation and a repeated binding keeps its last result. Per repository rules no tests were written or run. Production effect is not yet measured.

## Residual decisions

The trigger still takes a project-wide advisory lock and row lock at every ingest commit, so ingests still queue behind a long publication. Narrowing it to the affected deployments is separate work. The publish transaction's remaining per-Requirement statements (`relational::store`) are unchanged; whether they dominate its duration is not measured. After deployment, compare the `publish_ms` in the API log and the count of `could not serialize` errors against 152 per twelve minutes.
