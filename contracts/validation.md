# Accepted and prospective validation

```text
azimuth validate [local model options] [--scope structure|acceptance] [--out <report.json>]
azimuth validate --project <project.json> --workset <workset.json> [--scope structure|acceptance] [--out <report.json>]
azimuth change validate <change> [local model options] [--scope structure|acceptance] [--out <report.json>]
azimuth change validate <change> --project <project.json> --workset <workset.json> [--scope structure|acceptance] [--out <report.json>]
azimuth change finalize <change> [local model options] --scope acceptance --assurance-snapshot <snapshot.json>
azimuth change archive <change> --date <YYYY-MM-DD> [local model options] --scope acceptance --assurance-snapshot <snapshot.json>
```

The project and workset options are required together. They select a complete federated account; local model, workspace, manifest or partial selection options cannot redefine its authority. Local validation accepts explicit model, workspace, manifest and support inputs. A change assessment requires the complete account, not `--only` selection.

The catalog owns repository topology, model sources, packages and Area mounts. The workset pins required checkouts, revisions and manifest digests. Core derives source identities from repository-specific mounts and preserves exact supplied qualified identities. Missing or corrupt required repository facts are not replaced by local fallback.

Change validation stages the prospective intent and account facets before loading their source support. Candidate producer declarations can therefore resolve even when their entities are not accepted yet. It does not mutate accepted facets or turn candidate authority into accepted intent. Collision, stale delta, malformed declaration and unresolved structural reference remain errors.

## Validation scopes

`--scope structure` is the default for both validation commands. It checks declaration and reference structure, source-link resolution and package contracts. Its report contains `format: "azimuth-validation"`, `version: 3`, `scope: "structure"`, `structural_validity` and categorized structural `findings`. It does not report evidence completeness, independent review obligations, acceptance readiness or unfinished plan items. Structural errors cause exit one; valid structure causes exit zero.

`--scope acceptance` additionally reports declared evidence gaps, required independent reviews and change completion obligations. Its report adds `evidence_completeness`, `acceptance_readiness` and `acceptance_blockers`; there is no `review_heads` key. Review obligations are Findings ([assurance-reviews.md](assurance-reviews.md#review-heads-in-the-model)): validation raises `missing-requirement-assurance` for every Requirement that declares `Verification:` and `missing-realization-review` for every Requirement that declares `Realization:`, then recomputes ratifications from the sealed reviews of active and archived changes plus the model's review heads (latest by review time wins, assessed against the merged model) and removes the Finding of each Requirement and kind with a current accepted review. A head whose Requirement is absent or no longer declares that review is `unknown-review-head`. Code drift never creates a Finding. Any declared finding or acceptance blocker prevents a positive readiness result and causes exit one. Parser or input errors cause exit two in either scope.

`--assurance-snapshot <file>` is an optional input to acceptance validation. It contains the exact selected `{project, model_revision, authority, reviews, bundles}`. Core checks complete authority against the inspected model and uses its bundles to resolve the exact Verification Targets, active Run heads and Challenge facts on which Verification Review heads rely. A snapshot never supplies a review or a head; only a ratifying review from the change reviews or the model heads clears a missing-review finding. The snapshot does not clear unrelated structural, source-support or completion findings. Supplying it in structural scope is an input error; validation never invents approval or execution facts.

The same snapshot can be supplied at explicit `change finalize` and `change archive` boundaries with `--scope acceptance`. Each command reassesses it against the complete loaded current model and preserves existing completion, applied-facet, finding and finalization-currentness gates. Partial `--only` selection is rejected. These local commands do not perform full federated finalization, select AAS authority or publish records. A snapshot does not authorize intent/account application or archive an incomplete change. Finalize writes its ordinary model finalization receipt; archive additionally verifies that receipt against current model findings before moving the completed change. A Requirement that declares a review remains blocked at either boundary until a current accepted review of that kind ratifies it.

A change directory may carry `acceptance.json`, the owner's recorded acceptance of the intent of Requirements the change adds. Every Requirement the change adds that declares `Realization:` or `Verification:` must be named in it before finalization (see the ratification gate below):

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

The record is strict: unknown fields, an empty `requirements` list, duplicates and a non-identifier `accepted_by` are errors. `change finalize` and `change archive` accept only Requirements that the change itself adds. The record supplies no review, head, Run or Status and satisfies no finding. The finalization receipt lists each accepting change, decider, date and Requirement set, so removing or editing the record makes the receipt stale. Archived records keep applying to plain `azimuth validate` at acceptance scope and to federated project finalization, because they are part of the immutable decision record. Azimuth does not contact an Azimuth Assurance Service to accept a change; the basis is the decider's statement.

## Ratification gate at finalization

A change is applied to the model and archived when it merges to main, so finalization is where a model change is ratified. Ratification evidence lives in git and moves with the change:

- Sealed native `verification-review` and `realization-review` records produced by `review-compile` are committed under the change (`reviews/*.json`) and move with it into the archive. The exact packets, Markdown section sources, relied-on Run proofs and required history are preserved through the portable retained-input package (`retained/manifest.json` and `files/`); ignored `.azimuth/` copies may remain scratch only. See [retained-inputs.md](retained-inputs.md).
- `acceptance.json` names every Requirement the change adds that declares a review, as above.

Finalization proceeds in this order:

1. Collect the sealed `reviews/*.json` records of every active and archived change and the records carried by the model's review heads. For every Requirement that declares `Verification:` or `Realization:`, select the latest record of that kind by review time and assess it against the merged model, supplying actual verified Run proofs from active and archived changes' retained-input manifests. Only an `accepted` assessment ratifies. A declared `challenge_basis` whose exact Run, result, target or correction history is absent remains stale or missing; a retained proof never fabricates an accepted review.
2. Every Requirement that this change's `acceptance.json` accepts needs a ratification for each label it declares; a missing one is a completion issue (`acceptance.json accepts <requirement>, which has no accepted <Label> Review that is current against the merged model`).
3. Remove the `missing-requirement-assurance` and `missing-realization-review` Findings that ratifications discharge. Any remaining error or warning — including a missing review for a Requirement the change did not touch whose blocking dependencies the change altered, or an `unknown-review-head` — fails finalization.
4. Write a head ([assurance-reviews.md](assurance-reviews.md#review-heads-in-the-model)) for each ratification the model does not already carry with the same record fingerprint, replacing that Requirement's previous head of the kind, and write the finalization receipt. No other command writes heads.

Whole-authority fingerprint equality between a review and the merged model is not required; each review is judged against its own blocking dependencies, so an unrelated Requirement edit does not stale it. A federated account applies the identical ratification inside its validation. The finalization receipt carries `ratifying_reviews`, one `{requirement, kind, review}` entry per ratification, where `review` is the record fingerprint, next to the accepting changes it lists.

The gate establishes that the records are present, current and accepted. Whether the commit came from a pull request with an owner's approving review is not known to the CLI: trusted CI verifies it and publishes the verdict ([release-publication.md](release-publication.md)).

Reports keep the structural conclusion separate from evidence and acceptance conclusions. No-declared-gaps or no-declared-blockers is bounded by the inspected account; neither establishes a supported Requirement or an exact-target Assurance State. The structural workflow task uses `azimuth change validate --scope structure`; acceptance assessment belongs to the later explicit boundary. Validation never executes native Checks, generates independent decisions, commits or archives.

The current alpha command has no `change check` or `change account-check` alias. Account preview/apply and intent preview/apply remain explicit authoring and acceptance operations.

## Explicit correction scope

A manually requested correction is a new change against the current model. `correction.json` version 1 has exact fields `format: "azimuth-change-correction"`, `version: 1`, `source: {path, fingerprint}`, `requirements`, `failed_revision`, `issues`. The source path is `archive/<dated-change-id>` below the same changes directory and must name a finalized archived change. Its fingerprint is canonical SHA-256 over the sorted relative file/digest inventory (`azimuth-archived-change-basis`, version 1), excluding ignored `.azimuth/` runtime state. Symlinks and nonregular files fail. The original archive is never edited.

`requirements` is a unique nonempty explicit set drawn from that source change's declared Requirement delta, sealed reviews or acceptance scope. Each must exist in the correction's prospective account or be added by it. `failed_revision` is null or a full lowercase 40/64-character Git revision; `issues` contains unique nonempty retained issue references. No field grants permission or changes an existing review. Unknown fields, a changed source fingerprint or unrelated Requirement scope fail closed. The record is immutable once authored; an identical write is idempotent.

An active correction may record actual human acceptance for these verified existing Requirements as well as its own additions. It still requires ratifying current accepted reviews and all other completion obligations. There is no blanket waiver for a problem label or arbitrary preexisting Requirement. Acceptance basis participates in the finalization receipt. Before merge the correction may join the same PR; after merge it needs a new PR. CI reports diagnostics only; it neither creates nor reports work packages, generates correction changes nor dispatches agents.

## Finalization closure

The local workflow's combined `finalize-change` action applies any still-pending reviewed facets through their projectors, invokes finalization, then archives the completed account. The command operations remain individually inspectable/recoverable. Native completion validation requires a verified retained-input package for this workflow cohort and rejects a sealed review omitted from its native dependency closure.

Work Provenance's committed shaping ledger excludes the consuming `finalize-change` action so recording that action does not recursively stale its own input. The actual finalization result is retained separately under `provenance/finalization-result.json`; it is not fed back into the semantic input manifest or ledger. Finalization and archive validate the same complete candidate and real acceptance. Generated records and relocation cannot excuse behaviorally changed source; affected native dependencies still require reassessment.

Finalization receipts are version 3. Alongside the validation model fingerprint they retain `authority_model_fingerprint`, the `sha256:`-prefixed authority model digest independent of project label and staging paths. Workflow completion compares this exact value with the candidate authority's `model_fingerprint`, requires zero errors/warnings, and validates retained evidence and acceptance separately. It never compares whole authority fingerprints belonging to different project labels.

## Factual outcome and separate change acceptance

New outcome authoring uses `Status: complete`. Human approval does not modify that file after the decision, because it was part of the reviewed input basis. A new workflow completion and any factual complete outcome require durable `change-acceptance.json` with exact fields `format: "azimuth-change-acceptance-decision"`, `version: 1`, `actor`, `decision`, `reviewed_at_ms`, `candidate_model_fingerprint`, `outcome_fingerprint`, `retained_inputs_fingerprint`, `report`. Only an actual `approved` decision qualifies. Time is a positive safe UTC millisecond integer; actor/report are substantive nonempty strings.

The candidate fingerprint must equal current authority's model digest; the outcome fingerprint hashes the exact `outcome.md` bytes; the retained-input fingerprint names the currently verified package. Changed basis blocks completion. The decision itself is outside its own input manifest to avoid self-reference. The workflow additionally requires the exact current approved human predecessor before finalization. Changes that add no Requirement declaring a review, and unchanged-intent changes, use this generic decision without fabricating a nonempty Requirement list in `acceptance.json`; a change that adds a Requirement declaring a review still requires the scoped acceptance and current accepted heads.

Historical `Status: accepted` outcomes remain recognizable outside the new finalize-change workflow; they do not let a new workflow bypass the separate decision gate. CI validates the submitted bytes and reports stale approval as a blocker; it never manufactures replacement permission.
