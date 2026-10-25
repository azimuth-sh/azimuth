# Realization-delta reference

Realization is a change facet symmetric with spec and verification: the three facets are intent (`spec.md`), realization (`realization.md`) and verification (`verification.md`). A module's `realization.md` states how the implementation makes its Requirements hold. It is optional for any Requirement; no review declaration requires one. When it exists, the Realization Review checks that the code at the realization sites is consistent with it.

Author `deltas/<module-id>/realization.md` against the accepted baseline using the installed account-delta grammar, then inspect the complete prospective account. Do not author an unguarded replacement of the current realization as a substitute for a delta.

A realization delta starts with `# Realization delta: <module-id>`; the accepted file starts with `# Realization: <module-id>`. Requirement and Scenario entries use `## Requirement realization: <id>` and `### Scenario realization: <id>`; `## Section:`, `### Mechanism:` and `#### Element:` entries keep their headings. New facets use supported Add declaration headings; existing facets use supported guarded Add, Replace and Remove blocks. Existing-block operations carry the exact parent path, `From:` fingerprint and required rationale from the account contract. Use `azimuth change account-capture`, `account-preview` and `change validate`; those commands capture or project without authorizing application.

The canonical structural grammar is `contracts/account-delta.md` and `contracts/account-realization.md` in the Azimuth repository. Installed commands and references define the release-matched syntax. IDs and relationships remain typed and stable; paths, nesting and module moves do not invent entity identity. A Mechanism is a named entry inside the realization facet; production sites implementing it carry `ImplementsMechanism(<mechanism-id>)`, and sites realizing a Requirement carry `Realizes(<requirement-id>)`.

Relate each affected Requirement and Scenario to the mechanism that provides its behavior. Explain the relevant ownership, state/interface boundary, trust assumption and protected effect; a familiar pattern name is not a causal account. Address durable identity, authority, concurrency, retry/idempotency, failure behavior, diagnostics, scale, compatibility, migration and rollback where material. Free-form alternatives and migration order belong in the optional change-root `design.md`, which is non-normative and never applied to the model. Avoid machinery with no scoped obligation.

Author and review the spec delta first, then the realization delta, then verification. Each delta review examines the exact projected result, including preserved content and repaired references. Verification can reveal a realization defect; return material changes to realization correction and independent reassessment rather than silently diverging.

Current model application is an explicit tooling operation against the reviewed preview. A stale guarded baseline or changed preview requires reconciliation. Never derive normative wording from implementation merely to make an existing implementation acceptable.


## Unchanged realization facet

When the requested implementation correction needs no realization transition, retain the explicit `facet-unchanged` decision described by `azimuth reference show facet-decisions`. Pin the actual accepted facet or verified absence. Do not create empty or artificial delta operations. The independent realization delta review still evaluates the decision and complete composed account.
