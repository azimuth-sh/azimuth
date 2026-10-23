---
name: azimuth-review-requirement
description: Independently assess an Azimuth Requirement, its design and implemented verification before execution or after corrections. Use for review-requirement work and successor Requirement Reviews; deployment Requirement Status is computed separately from applicable evidence.
---

# Review a Requirement

## Establish the review basis

Read the repository instructions, the captured workflow handoff when supplied, and `azimuth reference show assurance-reviews` for native record and authoring contracts. Resolve the selected Requirement and exact candidate authority, Scenarios, design, verification definitions, source-support manifests, implementation/configuration inputs and prepared Verification Challenge scopes. Read the original request and relevant owning-domain contracts. Compare current source with the retained basis; report unavailable or changed inputs rather than claiming they were reviewed.

The reviewer must be independent of the author of the work under assessment. A coordinator's correction report is navigation, not proof. Read predecessor reviews and inspect each finding against the corrected source. Preserve adverse history and link a successor assessment to its predecessor.

## Acceptance requires both conditions

1. **Adequate verification approach.** The declared Checks, Scenario contributions, independently established expectations, positive/negative controls and oracles can establish the Requirement's required behavior within a justified scope. Assess failure paths, relevant interfaces and trust boundaries, assumptions, and the validity of representative coverage. A design must credibly explain how the implementation provides that behavior.
2. **Faithful implementation.** The production mechanisms and implemented verification match that account. Check code, configuration, fixtures, instrumentation, selection and normalization can actually perform the declared examination and produce its required observations and controls. A tag, passing compilation, source inspection or empty response cannot substitute for an oracle required by a Check.

Accept only when both hold. Reject when either has a material gap. An accurate list of unfinished work does not make an incomplete verification implementation acceptable. Missing capture, control correlation or normalization is an implementation gap when it prevents the declared contribution from being produced.

Execution may remain pending. Review acceptance establishes readiness and credibility of the reviewed account, not successful Checks, supported Requirement Status, deployment readiness or permission to publish. Do not require execution merely to approve a definition review. Existing results may expose defects, but neither a passing test suite nor a clean Verification Challenge search replaces this assessment.

## Inspect the account through its implementations

Follow each required Scenario through its declared contributions to the concrete enforcing mechanisms and verification sites. Assess substantive coverage, not the number of tags, files or tests. Reuse owning-contract controls and shared enforcement where the relationship is justified; do not demand every business operation be repeated with every credential or input variant.

For each contribution, determine whether:

- selection accounts for required operations, transports and relevant variants, with independent expectations;
- fixtures make the required state and protected effect observable, and successful controls distinguish real denial from an invalid request, nonexistent resource, disabled feature or broken environment;
- observations measure the stated property at the relevant boundary; a failure status alone does not establish ordering, identity provenance or absence of a protected effect;
- effect and disclosure oracles are independently fixed and sufficiently complete for the proposition;
- the producer retains the required measurements, request/control relationships and exact inputs, and the normalizer preserves missing, failed or inconclusive measurements;
- Verification Challenge definitions, selected files/sites, helpers, exclusions, tool filters and configuration match their advertised search. Mapping completeness or a clean bounded search does not prove Requirement adequacy.

A legitimate scope boundary states what the method examines and has a justified relationship to the Requirement. Unfinished implementation omits something the method requires. For example, a bounded static scan may intentionally cover named rules and files; a Check promising transport-specific ordering cannot omit ordering observations for one required transport. Pending deployment access is an execution prerequisite when the method is implemented; a collector incapable of producing required replica observations is an implementation defect.

Assess this distinction explicitly. Do not weaken a Requirement or rewrite a Check during review to make its implementation fit. Identify any required scope decision for the accountable author or human.

## Author and retain the decision

Use `azimuth assurance review-input` against the exact candidate and retained history, then `review-draft` to scaffold the authored record. Use the command/reference contracts for model options, packet fingerprints and change-relative paths. Generated packets belong in the change's ignored `.azimuth/`; reviewer-authored Markdown belongs in its review artifact. Parallel reviewers use separate output files.

Record the actual reviewer, UTC review time, exact input packet and substantive rationale. Do not write `Agent` or `Model` fields: compilation rejects them, and the agent, model and harness that performed the review are recorded in the change's Work Provenance ledger (`azimuth reference show work-provenance`). State findings with the required proposition, concrete source location, why the present implementation cannot establish it and the bounded correction needed. Explain disposition of predecessor findings and separate blocking defects from legitimate limits and pending execution. Acceptance needs positive reasoning for both criteria, not merely an absence of objections.

Use `Reassesses` for successor records and retain the predecessor in compilation/history. Compile with `azimuth assurance review-compile`; compilation checks identity, dependency and record consistency, not semantic correctness. Do not fabricate timestamps, independence, observations or decisions to satisfy tooling.

Return a compact result with `work: review-requirement`, the selected Requirement target, `outcome: accepted | rejected`, a concise report and a `requirement-review` output selecting the compiled `record` ID. Output references are relative to the workflow directory unless a repository is explicit. The coordinator records the result using the original captured handoff and current coordination revision; a delegated reviewer does not advance the workflow unless assigned that responsibility.

Stop after retaining the review. Findings return to correction and independent reassessment. Review work alone authorizes no source fixes, test or Verification Challenge execution, AAS publication, accepted-authority selection, deployment, commit or archive.
