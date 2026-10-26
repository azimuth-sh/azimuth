---
name: azimuth-review-verification
description: Independently perform the Verification Review of an Azimuth Requirement that declares `Verification:`, after required local execution or after corrections. Use for review-requirement-verification work and successor Verification Reviews; deployment Requirement Status is computed separately from applicable evidence.
---

# Verification Review

The Verification Review judges whether the verification of one Requirement is credible: its Checks, oracles, coverage, independence and Challenge declarations. Every Requirement that declares `Verification: reviewed` owes one. Whether the product code does what the Scenarios say is the separate Realization Review (`azimuth-review-realization`); this review does not depend on the product realization sites.

Load `azimuth reference show retained-inputs` before assembling or relying on the final durable evidence package.

## Establish the review basis

Read the repository instructions, the captured workflow handoff when supplied, and `azimuth reference show assurance-reviews` for native record and authoring contracts. Resolve the selected Requirement and exact candidate authority: the Requirement text including its rigor declarations, Scenarios, verification definitions and Evidence Bindings, Challenge declarations, the execution units each Check runs, required local Check results, Check-directed Challenge reports, proposed issue dispositions and resolved source scopes. Read the original request and relevant owning-domain contracts. Compare current source with the retained basis; report unavailable or changed inputs rather than claiming they were reviewed. Missing material review inputs prevent acceptance; distinguish them from an unavailable environment needed only for later execution.

The reviewer must be independent of the author of the work under assessment. A coordinator's correction report is navigation, not proof. Read predecessor reviews and inspect each finding against the corrected source. Preserve adverse history and link a successor assessment to its predecessor.

## Acceptance requires both conditions

1. **Adequate verification approach.** First assess whether the Requirement and Scenarios express a complete, unambiguous and falsifiable obligation against the original request and owning contracts. Then assess whether the declared Checks, Scenario contributions, independently established expectations, positive/negative controls and oracles can establish the Requirement's required behavior within a justified scope. Assess failure paths, relevant interfaces and trust boundaries, assumptions, and the validity of representative coverage.
2. **Faithful Check implementation.** The executed Check units can actually perform the declared examination and produce its required observations and controls. Check code, configuration, fixtures, instrumentation, selection and normalization must match the declared method. A tag, passing compilation, source inspection or empty response cannot substitute for an oracle required by a Check.

Accept only when both hold. Reject when either has a material gap. An accurate list of unfinished work does not make an incomplete verification implementation acceptable. Missing capture, control correlation or normalization is an implementation gap when it prevents the declared contribution from being produced.

In the ordinary change workflow, required local Checks and declared Challenges precede Verification Review. Failed Checks, execution gaps and confirmed defects return to correction before review; a claimed equivalence or irrelevance disposition may proceed for independent judgment. Deployment-only execution may remain pending. Review acceptance establishes credibility of the reviewed verification, not supported deployment Requirement Status or permission to publish. Neither passing tests nor a clean Verification Challenge search replaces this assessment. Follow an explicitly different pinned review task according to its installed contract rather than fabricating execution facts.

## Judge the Challenge declarations

Challenges are optional and declared by the verification author per Check or Requirement. Judge whether the declared Challenges, or their absence, are adequate for the risk the Requirement carries. When a Challenge is declared, assess whether its definition, selected files/sites, helpers, exclusions, tool filters and configuration match its advertised search. When a security-sensitive Requirement declares no Challenge, record in the review why that absence is acceptable or raise it as an issue. A clean bounded search does not prove Requirement adequacy.

## Inspect the verification through its Checks

Start from the Requirement itself: Scenarios must not silently narrow its universal meaning or leave required propositions uncovered. Follow each required Scenario through its declared contributions to the Checks and the execution units they run. Assess substantive coverage, not the number of tags, files or tests. Reuse owning-contract controls where the relationship is justified; do not demand every business operation be repeated with every credential or input variant.

For each contribution, determine whether:

- selection accounts for required operations, transports and relevant variants, with independent expectations; justify representative coverage through shared enforcement or another explicit relationship to the omitted variants;
- fixtures make the required state and protected effect observable, and successful controls distinguish real denial from an invalid request, nonexistent resource, disabled feature or broken environment;
- observations measure the stated property at the relevant boundary; a failure status alone does not establish ordering, identity provenance or absence of a protected effect;
- expected outcomes come from the required contract or independently authored expectation data, rather than being inferred from the implementation or observations under review; effect and disclosure oracles are sufficiently complete for the proposition;
- the producer retains the required measurements, request/control relationships and exact inputs, and the normalizer preserves missing, failed or inconclusive measurements.

A legitimate scope boundary states what the method examines and has a justified relationship to the Requirement. Unfinished implementation omits something the method requires. For example, a bounded static scan may intentionally cover named rules and files; a Check promising transport-specific ordering cannot omit ordering observations for one required transport. Pending deployment access is an execution prerequisite when the method is implemented; a collector incapable of producing required replica observations is an implementation defect.

A bounded Check need not establish the whole Requirement alone, but the combined contributions must address the required propositions; an acknowledged limitation cannot excuse missing required coverage. Assess this distinction explicitly. Do not weaken a Requirement or rewrite a Check during review to make its implementation fit. Identify any required scope decision for the accountable author or human.

## Author and retain the decision

Use `azimuth assurance review-input` against the exact candidate and retained history, then `review-draft` to scaffold the authored record. Use the command/reference contracts for model options, packet fingerprints and change-relative paths. Scratch packets may be generated in ignored `.azimuth/`, but the final relied-on input packet and required dependency closure must be retained in declared durable locations accessible to CI; reviewer-authored Markdown belongs in its review artifact. Parallel reviewers use separate output files.

Record the actual reviewer, UTC review time, exact input packet and substantive rationale. Do not write `Agent` or `Model` fields: compilation rejects them, and the agent, model and harness that performed the review are recorded in the change's Work Provenance ledger (`azimuth reference show work-provenance`). Complete the assigned assessment after finding the first blocker. State all material review issues with stable identities from the installed contract, the required proposition, exact assessed basis/source location, why the verification cannot establish it and the bounded correction needed. Authored review issues are distinct from deterministic model Findings. Explain disposition of predecessor findings and separate blocking defects from legitimate limits and pending execution. Acceptance needs positive reasoning for both criteria, not merely an absence of objections.

Use `Reassesses` for successor records of the same Requirement and retain the predecessor in compilation/history. Inspect supplied Challenge findings against their exact examined inputs and assess proposed equivalence or irrelevance dispositions independently; retain the exact proof dependencies through the prepared packet. A clean search does not resolve a substantive finding by itself.

Compile with `azimuth assurance review-compile`; compilation checks identity, dependency and record consistency, not semantic correctness. Inspect the selected entry's assessment status and diagnostics before presenting it as current. Retain the reviewer-authored Markdown and the sealed native JSON written beside it in the change's `reviews/`. Finalization writes the latest ratifying review into the model's `verification-review.md`; never write or edit that file by hand, because a hand edit breaks its seal. Preserve earlier records when adding a successor rather than overwriting their sealed files. Do not fabricate timestamps, independence, observations or decisions to satisfy tooling.

Return a compact result with `work: review-requirement-verification`, the selected Requirement target, `outcome: accepted | rejected`, a concise report and a `verification-review` output selecting the compiled `record` ID. Output references are relative to the workflow directory unless a repository is explicit. The coordinator records the result using the original captured handoff and current coordination revision; a delegated reviewer does not advance the workflow unless assigned that responsibility.

Stop after retaining the review. Findings return to correction and independent reassessment. Review work alone authorizes no source fixes, test or Verification Challenge execution, AAS publication, accepted-authority selection, deployment, commit or archive.
