---
name: azimuth-workflow
description: Start or resume a local Azimuth change workflow, dispatch eligible work through its declared guidance, and retain results, corrections and real human decisions. Use for preparing or implementing a change across its authorized stages.
---

# Run a change workflow

Load `azimuth reference show retained-inputs` before assembling or relying on the final durable evidence package.

Read the repository instructions and `azimuth reference show workflow`. This skill operates the process; the workflow definition owns scheduling and gates, and task skills own artifact quality. Do not insert a coordination task between ordinary tasks.

## Start or resume

- Resolve the original request, explicit project/change identities, singular change authority, participating repositories and declared module scope. Explore unresolved product or architectural decisions before committing to a solution.
- For a new instance, inspect `azimuth workflow definition --workflow change-development`, prepare its exact input contract, and initialize it with an explicit repository map. Do not discover authority or manufacture scope by scanning folders.
- For an existing instance, inspect its pinned definition and history. Installing updated skills does not migrate that instance. Preserve prior instances and use only the installed migration/restart procedure.
- Read `workflow next` and retain its original handoff before starting work. Select from `eligible`; an empty set does not prove completion when obligations or required inputs remain outstanding.

## Dispatch and record

Load every skill/reference named by the handoff. Report missing guidance rather than guessing it. Supply the worker the complete handoff, original request, exact inputs and readable correction feedback. Use separate authors and reviewers; role labels or different model names do not establish independence.

One coordinator owns shared workflow updates and integration. Delegate path-isolated engineering packages only after validating their ownership and dependencies with `azimuth change work-packages`; generate each package brief through the tool. Do not treat local engineering packages as CI assignments. Workers retain distinct reports and edit only their declared paths.

Use the tool's gates: serialized spec, realization and verification delta authoring/review; implementation; candidate preparation; required Checks; implementation/Check-directed Challenges; independent Realization Review (`review-requirement-realization`, after implementation and candidate preparation) and Verification Review (`review-requirement-verification`, after execution), each only for Requirements that declare it; outcome; actual human acceptance; finalization and archival. Follow the installed definition when an existing instance differs; do not silently rewrite its history to this sequence.

Failed Checks, execution gaps and confirmed Challenge defects return to correction before review. Equivalence or irrelevance justifications remain proposals for independent review, never fabricated clean results. Rerun affected work against changed inputs and retain still-current unrelated results according to native contracts.

Record a compact result using the captured handoff, actual execution actor and current revision. The actor names one execution instance, not a role or model. Follow installed Work Provenance hook/descriptor guidance; never guess provenance or approval. Retry an update idempotently rather than manufacturing another attempt after losing its receipt.

## Stop at the boundary

Retain each task's required output/dependency closure where tooling and CI can retrieve it. Ignored local files may hold scratch data, but cannot be the only copy of relied-on records. Report incomplete retention instead of claiming completion.

The local workflow does not include a catch-all verify-change task or an agent publication-inputs task. CI assembles publication inputs deterministically from retained records. CI emits diagnostics; Dima initiates any correction manually. Do not create CI work packages, issue assignments, launch a Work Service, or infer permission to commit, merge, publish or deploy.

For a manually requested post-finalization correction, create an explicitly related correction change against the current model, retain the original archive unchanged, and reassess affected dependencies. Before merge it may join the existing PR; after merge it needs a new change/PR. Never apply the original delta twice.
