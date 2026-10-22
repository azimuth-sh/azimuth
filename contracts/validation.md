# Accepted and prospective validation

```text
azimuth validate [local model options] [--scope structure|acceptance] [--out <report.json>]
azimuth validate --project <project.json> --workset <workset.json> [--scope structure|acceptance] [--out <report.json>]
azimuth change validate <change> [local model options] [--scope structure|acceptance] [--out <report.json>]
azimuth change validate <change> --project <project.json> --workset <workset.json> [--scope structure|acceptance] [--out <report.json>]
azimuth change finalize <change> [local model options] --scope acceptance --assurance-snapshot <snapshot.json>
azimuth change archive <change> --date <YYYY-MM-DD> [local model options] --scope acceptance --assurance-snapshot <snapshot.json>
```

The project and workset options are required together. They select a complete federated account; local model, standards, workspace, manifest or partial selection options cannot redefine its authority. Local validation continues to accept explicit model, standards, workspace, manifest and support inputs. A change assessment requires the complete account, not `--only` selection.

The catalog owns repository topology, model sources, standards, packages and Area mounts. The workset pins required checkouts, revisions and manifest digests. Core derives source identities from repository-specific mounts and preserves exact supplied qualified identities. Missing or corrupt required repository facts are not replaced by local fallback.

Change validation stages the prospective intent and account facets before loading their source support. Candidate producer declarations can therefore resolve even when their entities are not accepted yet. It does not mutate accepted facets or turn candidate authority into accepted intent. Collision, stale delta, malformed declaration and unresolved structural reference remain errors.

## Validation scopes

`--scope structure` is the default for both validation commands. It checks declaration and reference structure, source-link resolution and package contracts. Its report contains `format: "azimuth-validation"`, `version: 2`, `scope: "structure"`, `structural_validity` and categorized structural `findings`. It does not report evidence completeness, independent review obligations, acceptance readiness or unfinished plan items. Structural errors cause exit one; valid structure causes exit zero.

`--scope acceptance` additionally reports declared evidence gaps, required independent assessments and change completion obligations. Its report adds `evidence_completeness`, `acceptance_readiness` and `acceptance_blockers`. Non-routine compact Requirements require current accepted native Requirement Review and supported exact-target Requirement Status; an absent snapshot leaves `missing-requirement-assurance` findings. Explicitly authored historical declarations retain their own diagnostics, without satisfying current native assessment requirements. Any declared finding or acceptance blocker prevents a positive readiness result and causes exit one. Parser or input errors cause exit two in either scope.

`--assurance-snapshot <file>` is an optional input to acceptance validation. It contains the exact selected `{project, model_revision, authority, reviews, bundles}`. Core checks complete authority against the inspected model and resolves native review dependencies, retained assessment history, exact Verification Targets, active Run heads and relevant Challenge facts. Only a current accepted Requirement Review together with supported Requirement Status clears the corresponding missing-assurance finding. Rejected, stale, ambiguous, unsupported or absent assessments do not clear it. The snapshot does not clear unrelated structural, source-support or completion findings. Supplying it in structural scope is an input error; validation never invents approval or execution facts.

The same snapshot can be supplied at explicit `change finalize` and `change archive` boundaries with `--scope acceptance`. Each command reassesses it against the complete loaded current model and preserves existing completion, applied-facet, finding and finalization-currentness gates. Partial `--only` selection is rejected. These local commands do not perform full federated finalization, select ASS authority or publish records. A snapshot does not authorize intent/account application or archive an incomplete change. Finalize writes its ordinary model finalization receipt; archive additionally verifies that receipt against current model findings before moving the completed change. Without native evidence, a compact non-routine Requirement remains blocked at either boundary.

A change directory may carry `acceptance.json`, a recorded human decision that accepts the non-routine Requirements the change adds without a locally authored Requirement Status:

```json
{
  "format": "azimuth-change-acceptance",
  "version": 1,
  "accepted_by": "<plain identifier>",
  "accepted_on": "YYYY-MM-DD",
  "requirements": ["<requirement-id>"],
  "basis": "What the decider relied on, for example an Assurance State status read for a named release."
}
```

The record is strict: unknown fields, an empty `requirements` list, duplicates and a non-identifier `accepted_by` are errors. `change finalize` and `change archive` accept only Requirements that the change itself adds, and the record then removes exactly the `missing-requirement-assurance` findings for those Requirements. It supplies no review, Run or Status and satisfies no other finding. The finalization receipt lists each accepting change, decider, date and Requirement set, so removing or editing the record makes the receipt stale. Archived records keep applying to plain `azimuth validate` at acceptance scope and to federated project finalization, because they are part of the immutable decision record. Azimuth does not contact an Assurance State Service to accept a change; the basis is the decider's statement.

Reports keep the structural conclusion separate from evidence and acceptance conclusions. No-declared-gaps or no-declared-blockers is bounded by the inspected account; neither establishes a supported Requirement or an exact-target Assurance State. The structural workflow task uses `azimuth change validate --scope structure`; acceptance assessment belongs to the later explicit boundary. Validation never executes native Checks, generates independent decisions, commits or archives.

The current alpha command has no `change check` or `change account-check` alias. Account preview/apply and intent preview/apply remain explicit authoring and acceptance operations.
