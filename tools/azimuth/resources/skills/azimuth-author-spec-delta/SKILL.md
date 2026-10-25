---
name: azimuth-author-spec-delta
description: Author or revise a guarded spec delta and its exact projected target. Use for author-spec-delta work against new or existing Azimuth modules.
---

# Author a spec delta

If this change leaves the spec facet unchanged, load `azimuth reference show facet-decisions` and retain a `facet-unchanged` JSON result instead of inventing delta operations. Pin the actual accepted facet bytes, or its verified absence, and explain why the requested correction needs no spec change. Independent review still examines the composed account and may require a real delta.

Read the original request, reviewed proposal, captured handoff and accepted spec. Load `azimuth reference show intent-delta` and the repository's installed account-delta guidance. Spec, design and verification are symmetric change facets: author the transition and inspect the resulting account, not a replacement document detached from its baseline.

Use only supported operations and one authoring tree. Prefer `deltas/<module-id>/spec.md` when the change uses the symmetric layout. Keep Requirement and Scenario IDs stable and project-wide in their typed namespaces; module paths do not create identity. Replacements/removals must pin exact accepted blocks through `azimuth change intent-capture` and explicitly account for removed Scenarios.

Give each Requirement one falsifiable, independently governable proposition. Its id should compress what is true. Put universal meaning in the Requirement and give each Scenario an independently understandable condition and observable outcome. Include material failure/boundary conditions; do not turn test mechanics, rationale or implementation choices into intent unless they are themselves the required observable contract. Given/When/Then is optional authoring guidance, not parser syntax.

Generate `intent-preview` into a distinct reviewable file. Inspect the exact target and preserved surrounding text, references and removed blocks. Validate the prospective complete account with `change validate`; parser success cannot establish semantic adequacy.

Retain the delta, captured baseline and exact preview required by the handoff. Return the result for independent review. Do not apply current specs or author design/verification ahead of the workflow's accepted spec review.
