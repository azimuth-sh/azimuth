# Plan

1. Persist typed deployment/Claim/Case projections and dependency/invalidation state with additive migration.
2. Evaluate affected Claims with native semantics in a durable background worker. Preserve global validation closure and generation consistency; fail closed on dirty/expired projections.
3. Serve compact overview projections, retaining detailed evidence on demand and visible pending/freshness states.
4. Verify event coverage, expiry without writes, concurrent ingestion, restart recovery, reconstruction parity and performance; obtain independent review.
5. Publish gated images, back up production, apply reviewed rollout and measure live performance/status correctness. Keep acceptance separate.
