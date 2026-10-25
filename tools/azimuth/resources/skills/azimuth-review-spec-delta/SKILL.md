---
name: azimuth-review-spec-delta
description: Independently assess an Azimuth spec delta, its guarded baseline and projected account for completeness and precise behavior. Use for review-spec-delta and reassessment.
---

# Review a spec delta

Load `azimuth reference show authoring-reviews` for the exact result, stable issue and correction-disposition contracts.

A `facet-unchanged` result is an explicit no-change decision. Load `azimuth reference show facet-decisions`, verify its exact accepted baseline or verified absence, and independently assess the rationale against the request and composed account. Reject it when the spec requires correction; an unchanged declaration does not waive this review.

Read the original request, reviewed proposal, captured handoff, delta, captured baseline and exact intent preview. Load `azimuth reference show intent-delta` and applicable installed account guidance. Assess both the transition and its composed result, including unchanged neighboring obligations and repaired references.

Check that each Requirement is one falsifiable proposition with deliberate `Realization:`/`Verification:` declarations (or deliberately none), and that Scenarios supply meaningful independent conditions/results without narrowing the universal obligation. Examine denial effects, failure paths, boundaries, removals and compatibility where observable. A concrete example does not prove the full Requirement.

Reject silent Scenario loss, duplicated or incompatible obligations, unjustified scope changes and test procedures disguised as intent. Confirm that guarded replacement/removal targets the intended accepted blocks. Do not accept merely because projection parses or all included examples sound plausible.

Finish the complete assigned review rather than stopping at the first blocker. Retain stable review issue identities, exact locations/basis, the required proposition, consequence and bounded correction through the installed issue contract. Assess predecessor issues directly against current artifacts and preserve their history.

Return an independently authored accepted/rejected result. Acceptance permits realization delta authoring, not implementation or intent application. Do not edit the author's delta, invent approval or weaken intent to match implementation.
