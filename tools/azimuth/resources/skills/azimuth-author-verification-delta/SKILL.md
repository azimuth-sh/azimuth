---
name: azimuth-author-verification-delta
description: Author a guarded verification delta with credible Checks, independent expectations, controls and implementation-directed Challenges. Use after reviewed spec and design deltas.
---

# Author a verification delta

If this change leaves the verification facet unchanged, load `azimuth reference show facet-decisions` and retain a `facet-unchanged` JSON result instead of inventing delta operations. Pin the actual accepted facet bytes, or its verified absence, and explain why the requested correction needs no verification change. Independent review still examines the composed account and may require a real delta.

Read the original request, reviewed spec/design deltas and their composed targets, accepted verification, captured handoff and `azimuth reference show verification`. Use installed account-delta/package grammar for `deltas/<module-id>/verification.md`; Checks and Challenges are authored methods, not conclusions supplied by source tags.

For each required proposition define the observation, independent expectation/oracle, fixture, controls, relevant boundary, selection scope, retained measurements and inconclusive conditions. Relate Check contributions to Scenarios without claiming a narrow observation establishes the whole Requirement. Justify representative coverage through shared enforcement or another inspectable relationship.

Define Challenges against exact production implementation and Check inputs. Distinguish mutation targets, build dependencies and exercised Checks. Their results inform subsequent Requirement Review; a clean search is not proof of adequacy. Use only target and disposition forms provided by the installed release.

Specify execution contexts, adapters, required units, source/expectation fingerprints and retention outputs so engineering execution produces usable native records on its first run. Do not add evidence review obligations to routine Requirements merely because engineering tests exist.

Capture accepted-block bases, generate the account preview and inspect the full spec/design/verification composition. If verification exposes a design or intent defect, return it to the owning delta and relevant review instead of working around it in test prose.

Validate and retain the delta, preview and pinned supporting inputs. Return for independent verification review. Do not fabricate execution results, apply facets or advance to implementation without the applicable gate.
