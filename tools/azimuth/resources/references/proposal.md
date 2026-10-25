# Proposal reference

`proposal.md` begins with `# Change: <title>` and `Status: proposed`. It states the present problem, observable outcome, included and excluded work, exact affected stable Requirement identities and project context, originating exploration decisions and inspectable completion conditions.

When accepted intent does not change, declare `Intent delta: none` and a non-empty `Because:` reason after status. Do not combine that declaration with intent delta files. Proposal review permits subsequent delta authoring; exploration approval, implementation authorization and actual change acceptance are distinct decisions.

A proposal is one bounded transition, not the aggregate contents of every facet. Spec, realization and verification deltas own their proposed transitions; exact previews show their composed targets. Preserve the original request and baseline so reviewers assess intended effects and preserved account content directly.

A manually requested correction identifies its original change, failed candidate or execution, and relevant issues. It authors a new bounded transition against the current model; it does not rewrite an archived record or apply the original delta a second time. CI reports diagnostics and does not create correction proposals or work packages.


## Proposal content identity

Workflow references of kind `proposal-content` fingerprint the proposal after normalizing its top header `Status:` metadata to the workflow-owned status marker. This permits the actual lifecycle status to advance without invalidating an otherwise unchanged proposal review. Only that header status is workflow-owned: all remaining content is exact, including problem, scope, rationale, completion conditions and any `Status:` text inside body sections. Changing substantive proposal content requires affected reassessment; do not use the status field to hold normative content. Other file-reference kinds retain their ordinary exact-byte identity.
