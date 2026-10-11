---
name: azimuth-apply
description: Implement one approved Azimuth change, update current facets, perform repository-permitted engineering verification and leave the change ready for finalization. Use after proposal approval or when resuming active implementation; do not archive.
---

# Apply an approved change

## Establish the boundary

1. Read the target repository's applicable instructions and the complete approved change.
2. Confirm proposal approval or explicit implementation authorization. Resolve any material contradiction rather than inventing semantics.
3. Inspect the dirty worktree and preserve unrelated user changes.
4. Run `azimuth change show <id>` and the repository's normal `azimuth validate` to establish the baseline.
5. When work packages exist, use `azimuth change work-packages <id>` and `azimuth change brief <id> --package <package>` or invoke `azimuth-coordinate`.

## Implement

Follow the dependency order in `plan.md`. Keep changes inside scope, preserve exact accepted identities and use the target repository's native tooling. Update implementation, migrations, interfaces and documentation together when the approved design requires them. Record actual plan progress without manufacturing verification or rollout facts.

Update current `design.md` and other facets only to the state actually established by the implementation. Preserve archived decisions and unrelated accepted intent. Keep the intent delta proposed while implementation is unfinished. Near acceptance, recompute `azimuth change intent-preview <id> --out <file>` and review drift from the approved preview; revise and reapprove material intent changes. Apply the reviewed target with `azimuth change intent-apply <id> --preview <file>` only once the implementation supports it. The command, not manual transcription, updates current `spec.md`.

An accepted behavioral spec uses `# Spec: <spec-id>`, `## Claim: <claim-id>`, one `Criticality: critical | standard | routine`, a non-empty free-form normative Markdown Claim body, and one or more `### Case: <case-id>` blocks with non-empty free-form normative Markdown bodies. Bodies may use any human language, EARS, unrestricted prose, tables, diagrams or code fences. Core preserves and fingerprints them without interpreting keywords, translations or notation. The top three structural headings are reserved for Spec, Claim or Invariant, and Case declarations; use level-four headings or fenced content inside a body. Keep rationale and generated duplicate views outside normative bodies, preserve exact accepted ids, and never derive normative wording from implementation or tests.

## Verify and stop

Perform only engineering checks permitted by repository instructions. Run `azimuth change validate <id>`, the normal `azimuth validate`, and relevant reports or exports. Inspect the final diff and record residuals, unperformed checks and rollout-dependent conditions honestly. Leave the change ready for finalization and ask for the next action. Do not finalize, archive or commit implicitly.

## Explicit workflow handoffs

When a workflow instance is configured, use `azimuth workflow next --instance <file>` or `--change <directory>` and read `azimuth reference show workflow`. Select work from `eligible`; the harness owns scheduling and may execute eligible work in parallel with appropriate isolation. Read its full guidance and pinned inputs. Perform the work, retain its actual records, then call `workflow update` with the result and expected revision. Querying never advances the instance. Do not infer completion, approval or independent assurance from file presence.

Delegate required authoring reviews to another agent or an independent human when the runtime permits it. The reviewer reads the original request, artifact and dependencies directly, records exact input references and findings, and remains distinct from the recorded author. Rejected reviews require correction and reassessment; changed dependencies require fresh assessment. Local actor labels express harness assertions rather than authenticated identity. Service review ingestion must match the actual reviewer credential.

Use existing Method Qualification, Applicability Decision and Claim Judgment contracts for concrete assurance. Submit unchanged reviewer records to the explicit candidate revision in the existing ASS project, fetch an authenticated revision snapshot, and retain it as an accountable workflow input. Registering a candidate never replaces accepted authority or authorizes deployment. Agents may run locally while CI supplies Runs. Never manufacture reviewed-at times, independent reviewer decisions, Run facts or permission records to satisfy a workflow gate.

## Standard change workflow selection

Select `azimuth workflow init --workflow change-development --inputs <file> --instance <file> --repositories <file>` for ordinary change coordination. Inspect `azimuth workflow definition --workflow change-development` and `azimuth reference show workflow` for the exact input and inventory contracts. Supply explicit project/change identities, original request, optional referenced task instructions and declared module inventory; do not manually compose a process DAG or invent context-specific action names. Instruction overrides can direct retained-artifact copying while preserving approval and independent review gates.

Read `workflow next` using the same explicit repository map (or selected `--project`); handoffs carry portable references plus complete local execution locations. Act on `action` and typed `target`, and record results against the returned concrete `id`, basis and current revision. The harness chooses workers and obtains real permission. A local approval label grants no permission itself.

After source support, candidate validation and real same-project registration, supply the strict assurance-target inventory through the eligible inventory action. Core validates the native revision snapshot and expands qualification, execution, applicability and non-routine Claim Judgment targets. Unknown or unavailable inputs remain failed inventories or explicit blockers; file presence creates no process state. Corrective work registering another candidate adds a fresh generation and retains prior tasks as superseded. Preserve every supporting record and use independent reassessment; never invent assurance facts to advance a gate.
