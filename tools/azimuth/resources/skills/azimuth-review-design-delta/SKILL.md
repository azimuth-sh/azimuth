---
name: azimuth-review-design-delta
description: Independently assess a guarded design delta against the reviewed spec and its composed account, including failure and trust boundaries. Use for review-design-delta.
---

# Review a design delta

Load `azimuth reference show authoring-reviews` for the exact result, stable issue and correction-disposition contracts.

A `facet-unchanged` result is an explicit no-change decision. Load `azimuth reference show facet-decisions`, verify its exact accepted baseline or verified absence, and independently assess the rationale against the request and composed account. Reject it when the design requires correction; an unchanged declaration does not waive this review.

Read the original request, reviewed spec delta/preview, design delta, guarded baseline, account preview and captured handoff. Load `azimuth reference show design` and installed account-delta guidance. Inspect the complete composed account, not just added prose.

Follow each affected Requirement and Scenario to its enforcing mechanism and relevant state/interface boundaries. Assess feasibility, ownership, concurrency, retries, idempotency, security, failure behavior, operational prerequisites, migration and rollback only where they matter. Check that shared mechanisms justify their claimed consumers and that preserved design still composes.

Identify missing decisions, mechanisms that cannot provide the required effect, unsupported assumptions and contradictions with current architecture. Check changed or removed identities and dependent references. Structural validity or familiar patterns are not sufficient acceptance reasons.

Finish the assigned review and retain all material issues with stable identities, exact basis/locations, consequence and bounded correction. Independently reassess predecessor issues against corrected artifacts. Do not implement the proposed correction during review.

Return accepted/rejected with substantive reasoning. Acceptance unlocks verification delta authoring; material design changes discovered there return to design correction and affected reassessment.
