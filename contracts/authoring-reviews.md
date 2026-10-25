# Authoring review issues and correction dispositions

Authoring reviews independently assess a proposal or a spec, design or verification delta. Their issues are authored judgments, not deterministic model Findings from `findings.md`. Existing sealed historical records containing prose finding strings remain readable with their original meaning; newly authored rejected reviews retain stable structured issue objects.

## Issue account

An issue has exactly:

```json
{
  "id": "missing-denial-effect",
  "proposition": "The denial Scenario must state the protected effect.",
  "location": "deltas/access/spec.md:18",
  "reason": "The current Scenario states a response but omits the required effect boundary.",
  "correction": "State the protected effect or absence of it in this Scenario.",
  "blocking": true
}
```

`id` is a lower-kebab identity, unique inside the review. `proposition`, `location`, `reason` and `correction` are nonempty strings; `blocking` is boolean. A rejected review requires at least one issue. An accepted review cannot retain a blocking issue. Location is navigation context, not the identity: moving the source does not rename the issue. IDs describe the proposition and should be preserved when an issue is carried into a successor review.

An exact issue reference is `<review-fingerprint>#<issue-id>`. The native review or tool-retained review decision fingerprint pins the historical review; caller-provided labels and current source locations cannot substitute for it. Native review identity, exact dependency validation, author/reviewer separation and permission are governed by the workflow contract.

## Correction dispositions

Each required issue receives exactly one disposition:

```json
{
  "issue": "sha256:<review-fingerprint>#missing-denial-effect",
  "outcome": "corrected",
  "rationale": "The Scenario now names the absence of the protected effect; the corresponding verification contribution was updated."
}
```

All three fields are required, and unknown or duplicate fields are rejected. `issue` must belong to the exact required correction basis. Duplicate, unknown and missing references fail validation. Outcomes are `corrected | equivalent | irrelevant | confirmed-defect | unresolved`; rationale is nonempty and explains the actual disposition, including performed verification or explicit remaining limits where applicable.

Complete disposition coverage is distinct from resolution. `confirmed-defect` and `unresolved` preserve adverse facts and do not close the issue or justify acceptance. Independent reassessment decides whether a claimed correction or disagreement resolves the proposition. An implementer's `corrected` report alone is not an accepted review.

The tooling exposes `validate_issues`, `validate_dispositions`, `dispositions_resolved` and `issue_reference` in `authoring_review`. Disposition closure is used only after exact shape and coverage validation; an empty array establishes closure only when the required issue basis is itself empty.

## History and scope

Predecessor reviews and their issues remain retained. A successor uses exact references to disposition the issues it assesses. Changed source is inspected directly; unchanged prose or a coordinator summary is not proof of correction. Missing input history remains a gap rather than implicit acceptance. Review currentness follows the workflow's exact dependency basis.

These records guide local correction work and independent reassessment. CI may retain diagnostics and native records, but does not generate work packages, create assignments or launch agents. Dima initiates CI corrections manually.
