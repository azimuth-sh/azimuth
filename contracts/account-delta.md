# Account-delta contract

A change may author `deltas/<module-id>/{spec,design,verification,judgments}.md`. Each facet begins with `# Intent delta: <module-id>`, `# Design delta: <module-id>`, `# Verification delta: <module-id>` or `# Judgments delta: <module-id>`. The declared module id must match its path. The target is `azimuth/model/<module-id>/<facet>.md` or the configured model root. All `spec.md` transitions, including new modules, belong to the separate [intent-delta contract](intent-delta.md) and are skipped by account projection. Intent deltas may live in either the older `specs/` tree or `deltas/<module-id>/spec.md`; a change containing both trees fails closed. The intent projector is the only writer of accepted specs. It projects `## Add term:` vocabulary as well as Requirement and Scenario transitions.

For a **new account facet**, supported declarations use `Add` headings. The account projector changes the first heading to `# Design:`, `# Verification:` or `# Judgments:` and removes `Add ` from recognized declaration headings outside Markdown fences. All other bytes remain unchanged. The caller rejects an existing target facet before using full-module projection.

| Facet | Full-module Add declarations |
| --- | --- |
| Design | `## Add requirement design:`, `### Add scenario design:`, `##` through `###### Add section:`, `###` through `###### Add element:`, `###` through `###### Add mechanism:` |
| Verification | `## Add requirement verification:`, `### Add scenario verification:`, `##` or `### Add section:`, `###` or `#### Add element:`, `###` or `#### Add check:` |
| Judgments | `## Add requirement review:`, `## Add section:` |

For an **existing design, verification or judgments facet**, an operation declares one accepted block. Supported headings are `Add`, `Replace` and `Remove` followed by a declaration kind and identity at the heading levels above. The operation must give the full Markdown parent path. The module itself is the parent of an H2 declaration; each nested parent adds ` > <canonical kind> <identity>`. For example, a section nested in a Requirement design may declare `Parent: auth/cli > Requirement design refresh-rotates-credentials-and-revokes-reuse`. A mechanism nested in that section adds ` > Section refresh-elements`. The exact parent selects the block even when local section names repeat in different Requirements.

```md
### Replace section: refresh-elements
Parent: auth/cli > Requirement design refresh-rotates-credentials-and-revokes-reuse
From: sha256:<exact-accepted-block-digest>
Because: the revised boundary separates refresh and replay

The complete replacement section body follows. Nested accepted-format headings may follow.

### Remove section: obsolete-path
Parent: auth/cli > Requirement design refresh-rotates-credentials-and-revokes-reuse
From: sha256:<exact-accepted-block-digest>
Because: the responsibility moved to another section

### Add section: new-path
Parent: auth/cli > Requirement design refresh-rotates-credentials-and-revokes-reuse

The new section body follows.
```

`From:` hashes the exact accepted block bytes, starting at its heading and ending immediately before the next Markdown heading at the same or higher level outside a fence. This includes nested headings, prose, blank lines and trailing separators. `azimuth change account-capture` fills only an empty `From: ` value; it never overwrites a different existing digest. A stale digest, missing parent, duplicate block identity, unsupported operation, overlapping edit or target collision fails with a source path and line. A block already equal to its complete Add or Replace target is recognized as applied; an absent Remove target under an existing parent is recognized as applied. Replacement supplies the complete target body; the projector restores the accepted heading and Markdown blank-line separators. Removal removes that block and its descendants. Add inserts at the end of the declared parent. Other accepted text is copied byte-for-byte. Adding to a parent that is replaced or removed in the same delta is an overlapping edit; replace the parent as one complete block instead.

`azimuth change account-capture <change> [--model <dir>]` fills empty accepted-block fingerprints in existing-facet deltas without changing accepted content. `azimuth change account-preview <change> [--model <dir>] [--out <file>]` projects design, verification and judgments facets. `azimuth change intent-preview` projects the spec facets from the same `deltas/` authoring tree. It reports exact before and target text. A preview file must be outside the accepted model and cannot be overwritten with different content. `azimuth change validate <change> [--model <dir>] [--workspace <file>] [--manifest <file>...] [--support <file>...]` projects both intent and account facets into a temporary prospective model. Project catalog and complete workset inputs derive source qualification internally; local model/workspace/manifest inputs remain available for a single account. Parser errors, duplicate identities, unknown references and invalid Areas are hard errors. In acceptance scope, missing production realization, Check contributions, verification-element source support and pending Requirement reviews remain visible findings; they do not prevent applying the authored account, but they prevent a positive assurance conclusion.

`azimuth change account-apply <change> --preview <file> [--model <dir>]` requires an active change with `Status: active` or `Status: accepted and complete`, byte-for-byte equality with a freshly computed preview, matching accepted block fingerprints and a structurally valid prospective account. It displays unresolved findings before writing. It stages target facets and never silently replaces a newly created target; an existing target is re-read before replacement. Multi-file replacement is not a filesystem transaction: a later write failure may leave earlier facets applied and require deliberate repair.

This projector preserves prose and fences but does not infer design sufficiency or verification adequacy. Source extraction, method review, execution and Assurance State remain distinct stages. Existing spec replacements/removals continue through the intent projector; this account grammar does not redefine their behavior.

`azimuth change validate` reports each account facet as planned or applied and uses the same single-tree spec resolver as `intent-preview`, `intent-capture` and `intent-apply`. Finalize and archive require every account target to be applied, in addition to the existing intent gates.

Requirement design, Requirement verification and Requirement review declaration identities are local to the facet's declared module. Scenario design identities are local to their enclosing Requirement design. Existing-facet `Parent:` paths retain those authored local identities, for example `payments/recovery > Requirement design accepted-write > Scenario design broker-loss`. Full-module additions and guarded Add, Replace and Remove operations support Scenario design blocks. The projected design parser resolves local identities and Mechanism Scenario references before account inspection; projection retains their authored spelling. Section prose and explicit Scenario design declarations remain distinct.

Scenario verification identities are local to their enclosing Requirement verification. Full-module additions and guarded Add, Replace and Remove operations support these blocks, for example `Parent: payments/recovery > Requirement verification accepted-write`. Projection preserves authored spelling; account parsing normalizes Scenario references and derives required Element ownership before inspection.

Verification Check blocks support full-module additions and guarded Add, Replace and Remove operations at their permitted nesting levels. A Scenario-owned Check parent is `payments/recovery > Requirement verification accepted-write > Scenario verification broker-loss`; a Section-owned Check parent substitutes its Section. Existing accepted-block fingerprints retain exact Method, Terminal, reference and rationale bytes.

Ordinary verification captions carry no Add declaration or identity. Projection retains their Markdown as prose; nested Check declarations remain explicit Add operations at levels three through six. Guarded parent paths preserve authored reading ancestry, but captions do not create an entity locator or replace a Requirement/Scenario owner.

## Unified validation: 2026-10-06

`change validate` replaces the separate change check and account-check commands in the current alpha interface. There are no compatibility aliases. It validates prospective intent and account relationships together, defaults to structural scope, with evidence gaps and acceptance blockers reported only by explicit `--scope acceptance`, and does not apply facets or approve independent reviews. Pending reviews do not make otherwise valid account syntax invalid. See [validation.md](validation.md).

## Corrections after completion

A correction requested after CI/PR feedback is an explicitly related new change against the current model, which already includes any applied original delta. The `correction.json` scope/basis contract is defined in [validation.md](validation.md#explicit-correction-scope). It preserves the original archived change unchanged. Reconcile any changed accepted-block guards, author the correction's own transition and retain successor assessments for affected definitions/inputs. Do not reapply or silently mutate the original delta. Current semantic reviews, native execution and actual human acceptance still gate completion.
