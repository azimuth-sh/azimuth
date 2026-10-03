# Account-delta contract

A change may author `deltas/<module-id>/{spec,design,verification,judgments}.md`. Each facet begins with `# Intent delta: <module-id>`, `# Design delta: <module-id>`, `# Verification delta: <module-id>` or `# Judgments delta: <module-id>`. The declared module id must match its path. The target is `azimuth/model/<module-id>/<facet>.md` or the configured model root. All `spec.md` transitions, including new modules, belong to the separate [intent-delta contract](intent-delta.md) and are skipped by account projection. Intent deltas may live in either the older `specs/` tree or `deltas/<module-id>/spec.md`; a change containing both trees fails closed. The intent projector is the only writer of accepted specs. It projects `## Add term:` vocabulary as well as Claim and Case transitions.

For a **new account facet**, supported declarations use `Add` headings. The account projector changes the first heading to `# Design:`, `# Verification:` or `# Judgments:` and removes `Add ` from recognized declaration headings outside Markdown fences. All other bytes remain unchanged. The caller rejects an existing target facet before using full-module projection.

| Facet | Full-module Add declarations |
| --- | --- |
| Design | `## Add claim design:`, `### Add case design:`, `##` through `###### Add section:`, `###` through `###### Add element:`, `###` through `###### Add mechanism:` |
| Verification | `## Add claim verification:`, `### Add case verification:`, `##` or `### Add section:`, `###` or `#### Add element:`, `###` or `#### Add check:` |
| Judgments | `## Add claim review:`, `## Add section:` |

For an **existing design, verification or judgments facet**, an operation declares one accepted block. Supported headings are `Add`, `Replace` and `Remove` followed by a declaration kind and identity at the heading levels above. The operation must give the full Markdown parent path. The module itself is the parent of an H2 declaration; each nested parent adds ` > <canonical kind> <identity>`. For example, a section nested in a Claim design may declare `Parent: auth/cli > Claim design refresh-rotates-credentials-and-revokes-reuse`. A mechanism nested in that section adds ` > Section refresh-elements`. The exact parent selects the block even when local section names repeat in different Claims.

```md
### Replace section: refresh-elements
Parent: auth/cli > Claim design refresh-rotates-credentials-and-revokes-reuse
From: sha256:<exact-accepted-block-digest>
Because: the revised boundary separates refresh and replay

The complete replacement section body follows. Nested accepted-format headings may follow.

### Remove section: obsolete-path
Parent: auth/cli > Claim design refresh-rotates-credentials-and-revokes-reuse
From: sha256:<exact-accepted-block-digest>
Because: the responsibility moved to another section

### Add section: new-path
Parent: auth/cli > Claim design refresh-rotates-credentials-and-revokes-reuse

The new section body follows.
```

`From:` hashes the exact accepted block bytes, starting at its heading and ending immediately before the next Markdown heading at the same or higher level outside a fence. This includes nested headings, prose, blank lines and trailing separators. `azimuth change account-capture` fills only an empty `From: ` value; it never overwrites a different existing digest. A stale digest, missing parent, duplicate block identity, unsupported operation, overlapping edit or target collision fails with a source path and line. A block already equal to its complete Add or Replace target is recognized as applied; an absent Remove target under an existing parent is recognized as applied. Replacement supplies the complete target body; the projector restores the accepted heading and Markdown blank-line separators. Removal removes that block and its descendants. Add inserts at the end of the declared parent. Other accepted text is copied byte-for-byte. Adding to a parent that is replaced or removed in the same delta is an overlapping edit; replace the parent as one complete block instead.

`azimuth change account-capture <change> [--model <dir>]` fills empty accepted-block fingerprints in existing-facet deltas without changing accepted content. `azimuth change account-preview <change> [--model <dir>] [--out <file>]` projects design, verification and judgments facets. `azimuth change intent-preview` projects the spec facets from the same `deltas/` authoring tree. It reports exact before and target text. A preview file must be outside the accepted model and cannot be overwritten with different content. `azimuth change account-check <change> [--model <dir>] [--workspace <file>] [--manifest <file>...] [--support <file>...]` assembles the prospective model in a temporary directory. Parser errors, duplicate identities, unknown references and invalid Areas are hard errors. Missing production realization, Check contributions, verification-element source support and pending Claim reviews remain visible findings; they do not prevent applying the authored account, but they prevent a positive assurance conclusion.

`azimuth change account-apply <change> --preview <file> [--model <dir>]` requires an active change with `Status: active` or `Status: accepted and complete`, byte-for-byte equality with a freshly computed preview, matching accepted block fingerprints and a structurally valid prospective account. It displays unresolved findings before writing. It stages target facets and never silently replaces a newly created target; an existing target is re-read before replacement. Multi-file replacement is not a filesystem transaction: a later write failure may leave earlier facets applied and require deliberate repair.

This projector preserves prose and fences but does not infer design sufficiency or verification adequacy. Source extraction, method review, execution and Assurance State remain distinct stages. Existing spec replacements/removals continue through the intent projector; this account grammar does not redefine their behavior.

`azimuth change check` reports each account facet as planned or applied and uses the same single-tree spec resolver as `intent-preview`, `intent-capture` and `intent-apply`. Finalize and archive require every account target to be applied, in addition to the existing intent gates.

Claim design, Claim verification and Claim review declaration identities are local to the facet's declared module. Case design identities are local to their enclosing Claim design. Existing-facet `Parent:` paths retain those authored local identities, for example `payments/recovery > Claim design accepted-write > Case design broker-loss`. Full-module additions and guarded Add, Replace and Remove operations support Case design blocks. The projected design parser resolves local identities and Mechanism Case references before account inspection; projection retains their authored spelling. Section prose and explicit Case design declarations remain distinct.

Case verification identities are local to their enclosing Claim verification. Full-module additions and guarded Add, Replace and Remove operations support these blocks, for example `Parent: payments/recovery > Claim verification accepted-write`. Projection preserves authored spelling; account parsing normalizes Case references and derives required Element ownership before inspection.

Verification Check blocks support full-module additions and guarded Add, Replace and Remove operations at their permitted nesting levels. A Case-owned Check parent is `payments/recovery > Claim verification accepted-write > Case verification broker-loss`; a Section-owned Check parent substitutes its Section. Existing accepted-block fingerprints retain exact Method, Terminal, reference and rationale bytes.

Ordinary verification captions carry no Add declaration or identity. Projection retains their Markdown as prose; nested Check declarations remain explicit Add operations at levels three through six. Guarded parent paths preserve authored reading ancestry, but captions do not create an entity locator or replace a Claim/Case owner.
