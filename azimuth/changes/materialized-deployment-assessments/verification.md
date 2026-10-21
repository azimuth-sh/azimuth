# Verification plan

- Compare materialized native results with complete direct assessment using real PostgreSQL and authenticated HTTP.
- Verify selective invalidation for Runs/revisions, attribution, captures, contracts, reviews, release/deployment registration and target resolution; unrelated Claims retain results.
- Verify generation races, atomic totals/Claim/Case revision publication, durable restart recovery and failures.
- Verify time-driven monitor/context/reconciliation expiry with no new writes; fast reads never expose expired supported as current.
- Verify additive upgrade preserves existing facts and contains no JSON/document storage columns.
- Verify synthetic UI, pending/freshness rendering, compact summary usage, and native framework validation.
- Independently review implementation and saved production rollout plan; verify actual backup, schema, readiness, fresh statuses and dashboard endpoint timings after deployment.
