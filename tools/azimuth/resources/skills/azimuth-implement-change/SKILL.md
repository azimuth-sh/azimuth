---
name: azimuth-implement-change
description: Implement an authorized Azimuth change or bounded correction against reviewed delta targets, including faithful Check/Challenge capture and durable outputs. Use for implement-change work.
---

# Implement a change

Read the repository instructions, original request, captured handoff, reviewed proposal/deltas/previews and engineering plan. Confirm actual implementation authorization. Inspect the dirty worktree and preserve unrelated edits. Use the composed candidate as the target; never derive normative intent from code or tests.

Implement production behavior, interfaces/configuration, fixtures, independent expectations, Checks, Challenges, instrumentation and native capture together where the reviewed account requires them. Preserve stable entity identities. Move or remove every code `Realizes` site and support entry that a `removed-identity-referenced` finding from `change validate` still lists; `intent-apply` refuses the removal until none remains. Keep implementation inside scope and use repository-native conventions and permitted verification commands.

When assigned a package, edit only its declared paths. Report integration changes needed outside ownership rather than taking those files. One coordinator owns shared workflow state and current-model application. Workers do not finalize, archive or commit implicitly.

## Correct against actual feedback

Read every supplied review issue and failed Check/Challenge report, its exact assessed basis and currentness diagnostics. Inspect the current source directly; the coordinator's summary and your own prior report are not proof. For each stable issue record changed locations and actual verification, a justified disagreement, or an explicit unresolved limitation. Address the complete available issue set rather than fixing only the first rejection.

Return material intent/realization/verification changes to their owning deltas, decisions and independent reviews. Do not weaken the required behavior, fabricate a clean Challenge, hide adverse history, or mark unfinished capture as a legitimate scope boundary.

## Execute through the run pipeline

Load `azimuth reference show verification` before executing Checks or Challenges. Any execution that a later task, review or acceptance relies on goes through the run pipeline; selection is tooling output, never a hand-picked list of test classes:

1. Capture the baseline and the candidate with `azimuth run snapshot --project-id <id> --verification-target <file> [--unit-rule <file> --checkout <dir>]`. Combine them, with the obligations still pending from earlier iterations, into an `azimuth-affected-execution-request` and run `azimuth run affected --input <file> --out <file>`.
2. Resolve eligible Checks, capabilities and units with `azimuth run select --policy <file> --out <file>`. A required full-suite policy may broaden the affected selection; nothing narrows it.
3. Plan the selected work from the complete model with `azimuth run plan --selection <file>` or `--request <file>` (`contracts/run-launch-plan.md`).
4. Run it with `azimuth run execute --plan <file>` or `azimuth run import --plan <file> --input <id>=<file>`. Pass earlier bundles through `--predecessor` only where the unit-reuse contract allows it.
5. Check the bundles with `azimuth run verify --bundle <file>...` and retain them.

Start using the pipeline as soon as the first Checks exist, so capture and adapter gaps surface during implementation rather than at candidate preparation. Native test commands remain fine for quick engineering feedback, but their results are not evidence and do not decide what the pipeline must run. An `incomplete` affected report, a planning failure or a missing adapter is a prerequisite to report, not a reason to fall back to a manual selection.

## Retain and hand off

Prepare fresh source support and exact execution inputs required by candidate preparation. Required engineering execution must use capture that produces its declared observations and native records; a passing process without required capture is incomplete. Checks run before implementation/Check-directed Challenges, followed by independent Verification Review; Realization Review judges the code at the realization sites. Do not guess which obligations to skip or rerun the entire project without the required policy basis.

Retain the implementation result, per-issue dispositions and necessary output/dependency closure outside scratch-only storage. Report missing prerequisites honestly. Return the compact result; neither implementation completion nor CI success is human acceptance, deployment confirmation or permission to publish.

If Dima requests correction after an archived change, create an explicitly related correction change against the current model and findings. Preserve the original archive unchanged; refresh affected reviews and acceptance through the owning contracts. CI only reports diagnostics and never creates or assigns that correction automatically.
