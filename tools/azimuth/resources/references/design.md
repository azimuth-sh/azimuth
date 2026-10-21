# Design reference

Use `design.md` when implementation would otherwise decide material behavior or architecture. Relate each mechanism to an accepted Requirement, declare its enforcement kind and exact binding, and explain why removing the mechanism would break the Requirement.

Address durable identity, authority, state transitions, data invariants, concurrency, retries, idempotency, trust boundaries, failure behavior, diagnostics, scale, compatibility, migration, rollback and rejected alternatives only where relevant. Do not add speculative machinery with no scoped outcome or Requirement.
