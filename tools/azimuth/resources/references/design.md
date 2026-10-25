# Design-delta reference

Design is a change facet symmetric with spec and verification. Author `deltas/<module-id>/design.md` against the accepted baseline using the installed account-delta grammar, then inspect the complete prospective account. Do not author an unguarded replacement of the current design as a substitute for a delta.

A design delta starts with `# Design delta: <module-id>`. New facets use supported Add declaration headings; existing facets use supported guarded Add, Replace and Remove blocks. Existing-block operations carry the exact parent path, `From:` fingerprint and required rationale from the account contract. Use `azimuth change account-capture`, `account-preview` and `change validate`; those commands capture or project without authorizing application.

The canonical structural grammar is `contracts/account-delta.md` and `contracts/account-design.md` in the Azimuth repository. Installed commands and references define the release-matched syntax. IDs and relationships remain typed and stable; paths, nesting and module moves do not invent entity identity.

Relate each affected Requirement and Scenario to the mechanism that provides its behavior. Explain the relevant ownership, state/interface boundary, trust assumption and protected effect; a familiar pattern name is not a causal account. Address durable identity, authority, concurrency, retry/idempotency, failure behavior, diagnostics, scale, compatibility, migration, rollback and rejected alternatives where material. Avoid machinery with no scoped obligation.

Author and review the spec delta first, then the design delta, then verification. Review the exact projected result, including preserved content and repaired references. Verification can reveal a design defect; return material changes to design correction and independent reassessment rather than silently diverging.

Current model application is an explicit tooling operation against the reviewed preview. A stale guarded baseline or changed preview requires reconciliation. Never derive normative wording from implementation merely to make an existing implementation acceptable.


## Unchanged design facet

When the requested implementation correction needs no design transition, retain the explicit `facet-unchanged` decision described by `azimuth reference show facet-decisions`. Pin the actual accepted facet or verified absence. Do not create empty or artificial delta operations. The independent design review still evaluates the decision and complete composed account.
