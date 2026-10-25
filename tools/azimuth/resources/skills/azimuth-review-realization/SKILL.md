---
name: azimuth-review-realization
description: Independently perform the Realization Review of an Azimuth Requirement that declares `Realization:`, judging whether the code at its realization sites does what each Scenario says. Use for Realization Review work and successor Realization Reviews.
---

# Realization Review

The Realization Review judges whether the production code at a Requirement's realization sites does what each of its Scenarios says, consistent with the module's realization.md where one exists. Every Requirement that declares `Realization: reviewed` owes one. Whether the verification is credible is the separate Verification Review (`azimuth-review-verification`).

A Realization Review is an opinion about a code snapshot, not evidence. It never makes a Requirement Satisfied, and it supplies no Check Result or deployment Requirement Status.

Load `azimuth reference show assurance-reviews` for native record and authoring contracts and `azimuth reference show realization` for the realization facet.

## Establish the review basis

Read the repository instructions, the captured workflow handoff when supplied and the original request. Resolve the selected Requirement and read, from the exact candidate:

- the Requirement text, including its rigor declarations, and all of its Scenarios;
- the `## Requirement realization:` block and its `### Scenario realization:` and `### Mechanism:` entries in the module's realization.md, if present;
- the code at every `Realizes(<requirement>)` site and, where realization.md names mechanisms, every `ImplementsMechanism` site, with enough surrounding code to follow the behaviour each Scenario depends on.

Record the exact commit whose code you read. Report unavailable sites or inputs rather than claiming they were reviewed; missing material inputs prevent acceptance.

The reviewer must be independent of the author of the work under assessment. A coordinator's correction report is navigation, not proof. Read predecessor reviews, inspect each finding against the corrected code and link a successor assessment to its predecessor.

## Judge the realization

For each Scenario, follow its condition through the realization sites to its result and decide whether the code produces that result, including the failure and denial paths the Scenario states. Where realization.md exists, decide whether the code is consistent with it: the named mechanisms exist at their sites, own the state and boundaries it claims and provide the effect it attributes to them. A contradiction between code and realization.md is an issue even when the Scenario appears to hold.

Reject when the behaviour a Scenario depends on is not self-evident from the code and nothing states how it is realized: a reader must be able to see why the Scenario holds, either from the code itself or from realization.md. Reject when realization sites are missing for a Scenario, when the code at a site does not provide the stated result, or when it depends on an unstated assumption the Requirement does not permit.

Do not judge test adequacy here and do not treat passing Checks as proof that the code is right. Do not rewrite the Requirement, realization.md or code during review to make them fit.

## Author and retain the decision

Prepare and compile the record through the installed `azimuth assurance` review commands for the `realization-review` kind; use generated drafts instead of inventing metadata. Record the actual reviewer, UTC review time, reviewed commit and substantive rationale. Do not write `Agent` or `Model` fields; Work Provenance records the executing agent. Complete the assigned assessment after the first blocker. State every material issue with a stable identity, the Scenario or realization entry concerned, the exact code location, why the code does not realize it and the bounded correction. Explain disposition of predecessor issues. Acceptance needs positive reasoning for each Scenario, not merely an absence of objections.

The blocking dependencies of this review are the Requirement text, its Scenarios and realization.md. Later changes at the realization sites are shown as code drift since the reviewed commit; they do not stale the review, so state precisely what you read.

Retain the reviewer-authored Markdown and the sealed native JSON beside it in the change's `reviews/`. Finalization writes the latest ratifying review into the model's `realization-review.md`; never write or edit that file by hand, because a hand edit breaks its seal. Preserve earlier records when adding a successor.

Return the compact result the handoff requires, with `outcome: accepted | rejected`, a concise report and a `realization-review` output selecting the compiled record. Stop after retaining the review. Findings return to correction and independent reassessment; review work alone authorizes no source fixes, execution, publication, deployment, commit or archive.
