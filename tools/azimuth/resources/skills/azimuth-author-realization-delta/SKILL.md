---
name: azimuth-author-realization-delta
description: Author a guarded Azimuth realization delta stating how the reviewed spec delta is realized, including material architecture and failure decisions. Use for author-realization-delta work.
---

# Author a realization delta

If this change leaves the realization facet unchanged, load `azimuth reference show facet-decisions` and retain a `facet-unchanged` JSON result instead of inventing delta operations. Pin the actual accepted facet bytes, or its verified absence, and explain why the requested correction needs no realization change. Independent delta review still examines the composed account and may require a real delta.

A realization.md is optional for any Requirement. Author one where the behaviour is not self-evident from the code at its `Realizes` sites, so that a later Realization Review can tell how each affected Scenario is provided. Do not add realization prose that only restates the spec.

Read the original request, reviewed proposal/spec delta and exact projected spec, accepted realization, captured handoff and `azimuth reference show realization`. Use the installed account-delta grammar; do not invent block operations, heading ownership or metadata.

Author `deltas/<module-id>/realization.md` as a guarded transition. Relate the composed realization to every affected Requirement/Scenario through `## Requirement realization:` and `### Scenario realization:` blocks and explain the enforcing mechanisms, state ownership, interfaces and relevant failure effects. Mechanisms are named `### Mechanism:` entries inside the realization facet; production sites implementing one carry `ImplementsMechanism`. Preserve stable identities when moving content; nesting supplies relationships, not identity or evidence.

Resolve material authority, identity, concurrency, retry/idempotency, security, observability, migration and rollback decisions where relevant. State why the chosen mechanism provides the obligation. Free-form alternatives, rejected options and migration order belong in the optional change-root `design.md`, which is never applied to the model. Avoid speculative machinery that no scoped obligation requires.

Capture existing-block bases with `account-capture` and generate the exact `account-preview`. Inspect preserved content, changed relationships and the complete prospective spec/realization account. A meaningful uncertainty requires a decision, not an implementer's assumption.

Validate structure and retain the delta, baseline, preview and necessary decision inputs. Return for independent delta review; the workflow serializes the realization delta review before verification delta authoring. Do not apply facets, implement code or author your own acceptance.
