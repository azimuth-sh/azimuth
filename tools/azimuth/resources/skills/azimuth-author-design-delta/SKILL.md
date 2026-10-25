---
name: azimuth-author-design-delta
description: Author a guarded Azimuth design delta realizing the reviewed spec delta, including material architecture and failure decisions. Use for author-design-delta work.
---

# Author a design delta

If this change leaves the design facet unchanged, load `azimuth reference show facet-decisions` and retain a `facet-unchanged` JSON result instead of inventing delta operations. Pin the actual accepted facet bytes, or its verified absence, and explain why the requested correction needs no design change. Independent review still examines the composed account and may require a real delta.

Read the original request, reviewed proposal/spec delta and exact projected spec, accepted design, captured handoff and `azimuth reference show design`. Use the installed account-delta grammar; do not invent block operations, heading ownership or metadata.

Author `deltas/<module-id>/design.md` as a guarded transition. Relate the composed design to every affected Requirement/Scenario and explain the enforcing mechanisms, state ownership, interfaces and relevant failure effects. Preserve stable identities when moving content; nesting supplies relationships, not identity or evidence.

Resolve material authority, identity, concurrency, retry/idempotency, security, observability, migration and rollback decisions where relevant. State why the chosen mechanism provides the obligation and what alternatives were rejected. Avoid speculative machinery that no scoped obligation requires.

Capture existing-block bases with `account-capture` and generate the exact `account-preview`. Inspect preserved content, changed relationships and the complete prospective spec/design account. A meaningful uncertainty requires a decision, not an implementer's assumption.

Validate structure and retain the delta, baseline, preview and necessary decision inputs. Return for independent review; the workflow serializes design review before verification delta authoring. Do not apply facets, implement code or author your own acceptance.
