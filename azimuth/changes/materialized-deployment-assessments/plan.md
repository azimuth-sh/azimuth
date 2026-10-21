# Plan

1. Persist typed deployment/Claim/Case projections and dependency/invalidation state with additive migration.
2. Evaluate affected Claims with native semantics in a durable background worker. Preserve global validation closure and generation consistency; fail closed on dirty/expired projections.
3. Serve compact overview projections, retaining detailed evidence on demand and visible pending/freshness states.
4. Verify event coverage, expiry without writes, concurrent ingestion, restart recovery, reconstruction parity and performance; obtain independent review.
5. Publish gated images, back up production, apply reviewed rollout and measure live performance/status correctness. Keep acceptance separate.

## Local completion

Implementation and independent frozen source review are complete. All 20 server tests pass with no warnings, failures or ignored tests. UI rendering, typecheck and production build pass; overview and model regressions cover missing/expired freshness and explicit history loading. Native framework and change validation pass with 50 Claims and no Findings. CI and production rollout remain pending; no live schema or service change has occurred yet.
