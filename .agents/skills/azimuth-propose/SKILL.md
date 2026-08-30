---
name: azimuth-propose
description: Create or revise one bounded, approval-ready Azimuth change from a clear request or approved exploration. Use to establish singular authority, author current Claim/Case intent deltas, define scope and solution decisions, plan compatibility and migration, add safe work packages when useful, and validate a comprehensive proposal before implementation.
---

# Propose one change

Create the smallest semantic transition that can be reviewed, implemented and accepted independently. A proposal is fully contained when a reviewer can decide it without reconstructing hidden product, architecture, compatibility, migration or completion assumptions from conversation history.

Stop after presenting the authored proposal for explicit approval. Approval of an originating exploration authorizes proposal creation; it does not approve the resulting change or authorize implementation.

## Establish readiness and authority

1. Read the target repository's applicable `AGENTS.md` files and its change guidance. Respect its authority order, verification policy and dirty worktree.
2. Confirm the request is sufficiently decided for one change. Use an approved exploration for an uncertain multi-change initiative. Read the actual exploration files, require their explicit approved status, and carry only confirmed decisions assigned to this change.
3. Resolve any material contradiction or owner decision that would change the outcome, public contract, data authority, migration strategy or scope before authoring. Do not hide it as an implementation detail.
4. In a federated project, locate the repository with singular authority for the change. Read the project catalog and workset when present; do not create competing local authority.
5. Inspect the current accepted model, relevant archived decision records, implementation, data model, migrations, APIs, configuration and ordinary engineering checks. Treat current behavior as a discoverable fact, not as intended behavior unless accepted intent says so.
6. Check `azimuth --version` and repository package/tool pins. Read the current [`contracts/spec.md`](../../../contracts/spec.md) and parser contracts for every artifact being authored. Do not copy syntax from archived changes, which may preserve an older version.
7. Run the repository's normal `azimuth validate` invocation to distinguish a valid baseline from pre-existing Findings. If the current account cannot be loaded, report the blocker instead of manufacturing a valid proposal account.

## Create singular change authority

1. Run `azimuth change list`. Check active and archived entries; a stable id is not reused after archival.
2. Choose a lower-kebab id that names the transition rather than an implementation task.
3. Run:

   ```text
   azimuth change create <id> --title "<title>"
   ```

4. Keep `proposal.md` at `Status: proposed` throughout authoring. Do not mark it active merely because its exploration was approved.

## Bound the transition

Author the proposal from evidence gathered in the repository and any approved exploration.

- **Problem:** Describe the present condition, who or what it harms, and why the existing model or mechanism cannot support the desired outcome. Separate facts from inference.
- **Outcome:** State the accepted end state in observable terms. Avoid promising later-change behavior merely because this change creates an extension point for it.
- **In scope:** Enumerate the behavior, data, contracts, components, migration work and documentation this change owns.
- **Out of scope:** Name adjacent behavior, later changes, operational rollout, destructive cleanup and attractive refactors that are intentionally excluded.
- **Affected claims:** List exact `<spec-id>#<claim-id>` identities added or deliberately affected. Do not use file paths or informal headings as identity.
- **Originating decisions:** When applicable, cite the exploration and the exact decision or change-map entries carried by this proposal. Record any necessary refinement and why it does not contradict the approved direction.
- **Completion conditions:** Make every condition inspectable after implementation. Cover observable behavior, persisted invariants, compatibility or migration results, required current-facet updates, documentation and permitted engineering checks. Do not use vague conditions such as "works" or "tests pass."

Split the work when two outcomes can be accepted independently, when later work can begin without the first outcome, or when one proposal would require unrelated rollback decisions. A foundation change must still deliver usable invariants; an empty schema or speculative abstraction is not a semantic transition.

## Author current intent deltas

Add an intent delta only for an observable obligation that will become accepted intent. Organize specs by domain area, not service topology. Each file under `specs/` starts with one declared spec id:

```markdown
# Intent delta: <spec-id>

## Add claim: <falsifiable-claim-id>
Criticality: routine

Non-empty free-form normative Markdown stating this Claim.

### Add case: <standalone-case-id>
Non-empty free-form normative Markdown stating this Case.
```

Follow these rules:

- Use `## Add claim:` and `### Add case:` exactly. Never use the removed `Requirement`/`Scenario` vocabulary.
- Give every added Claim `Criticality: routine` during the current fast-moving alpha.
- Give each Claim one or more Cases. A Claim owns governance and criticality; Cases express distinct normative conditions within its predicate.
- Write one independently governable, falsifiable proposition per Claim. Split conjunctions that carry different consequences, owners or acceptance decisions.
- Make Claim and Case ids lower-kebab, proposition-like and visibly distinct. Cases must stand alone in traceability output.
- Put universal meaning in the Claim and materially distinct normative conditions in Cases. Cases describe behavior, never test mechanics, source paths or implementation structure.
- Claim and Case bodies are non-empty free-form normative Markdown in any human language. Core preserves and fingerprints their prose, tables, diagrams and code fences without interpreting natural-language keywords, translations or notation. Reserve `# Spec:`, `## Claim:` or `## Invariant:` and `### Case:` for structure; use level-four headings or fenced content inside a body. Keep orientation, rationale and generated duplicate views outside normative bodies.
- Do not add package `verification.md`, Checks, Evidence Bindings, Qualifications or Claim Judgments for routine Claims. Ordinary engineering checks remain outside the Azimuth evidence graph.
- The current delta parser machine-projects whole Claim additions and criticality changes. Do not invent modify, remove, rename or add-Case-to-existing-Claim operations. If the transition requires one, surface the parser/model limitation and resolve it explicitly before proceeding.

When accepted intent truly does not change, add this metadata directly after `Status: proposed` and before the first section instead of creating an empty or cosmetic delta:

```markdown
Intent delta: none
Because: <non-empty reason accepted intent remains unchanged>
```

Never combine `Intent delta: none` with a supported delta under `specs/`.

## Make solution decisions reviewable

Add `design.md` whenever implementation would otherwise have to choose behavior or architecture that could change the proposal's meaning. Include only relevant sections, but inspect each of these questions:

- What owns each durable identity, namespace and lifecycle?
- Which state is authoritative before, during and after the transition?
- What invariants must schema constraints, domain logic and transaction boundaries preserve?
- What are the read, write and event flows, including retries, concurrency and idempotency?
- How do authorization, privacy, secrets and trust boundaries change?
- What fails closed, what remains retriable, and what user-visible or operational failure results?
- What telemetry or diagnostics are required to operate and troubleshoot the mechanism?
- What scale, latency, storage or cost bounds affect the design?
- Which existing APIs, data, content, clients and deployments remain compatible?
- Does migration use expand/migrate/contract, backfill, dual read or dual write? Name the sole authority in every phase, cutover criteria, rollback boundary and destructive cleanup owner.
- Which alternatives were rejected, and for what concrete trade-off?
- Which extension points are intentionally reserved for later changes without implementing their semantics now?

Do not claim reversibility when deployed writes or destructive migrations make it false. Do not use a feature flag to avoid deciding data authority. For additive foundations, state explicitly what begins using the foundation in this change and what remains unchanged until a later cutover.

Keep design decisions traceable to a scoped outcome, Claim, Case or completion condition. Remove speculative machinery that none of them requires.

## Write an executable implementation plan

Replace the generated placeholders in `plan.md` with dependency-ordered, checkable implementation stages. The plan is an implementation sequence, not a verification artifact or exhaustive test inventory. Include, when applicable:

1. contract and accepted-intent changes that later steps depend on;
2. data/domain types, invariants and persistence changes;
3. compatibility adapters, backfill and authority cutover;
4. application behavior, interfaces and integrations;
5. ordinary engineering checks permitted by repository instructions;
6. emitted manifests and `azimuth validate` for the resulting current account;
7. current `spec.md` and `design.md` facet updates;
8. documentation, operational notes and removal of temporary compatibility paths owned by this change;
9. outcome recording and readiness for finalization, without finalizing or archiving.

Each item should name a concrete result and be markable complete without interpreting broad verbs such as "handle," "support" or "finish." Keep later-change work out of the plan even when the design names its extension boundary.

## Add work packages only when useful

Use `work-packages.md` only when independent, path-isolated execution will materially help. Freeze shared contracts before dependent packages, give shared state to the coordinator, and ensure workers do not edit overlapping paths, proposal state, outcome or archive location.

```markdown
# Work packages: <change-id>

## Work package: <id>
Status: pending
Depends on: none
Owns: path/one, path/two
Objective: one bounded result
Evidence: exact permitted engineering commands or inspections
```

Declare every dependency, use checkout-relative owned paths, and make each package independently reportable. Then run `azimuth change work-packages <id>` and resolve invalid status, missing objective, escaping or overlapping paths, unknown dependencies and cycles before any delegation.

## Preserve current framework boundaries

- For a site-domain invariant, identify the semantic population before implementation. Reuse an exact declared surface or propose area-mount enumerator contributions; a path is not semantic identity.
- For extractor changes, preserve the two-argument marker and ecosystem-semantic, path-free site identity. Heavy analysis remains in the ecosystem extractor and fails closed on ambiguity.
- For adapter or Run changes, read the current adapter, launch-plan and Run-bundle contracts. Preserve core authority over exact semantic selection, bounded shell-free exchanges, complete validation and atomic output. Do not invent daemons, ingest, retention, cache validity or Assurance State authority.
- A clean Challenge Result remains only a negative search fact. Do not turn execution success into repository acceptance or product evidence.

## Validate completeness and hand off

1. Run `azimuth change check <id>` and resolve every parser or projection error.
2. Run `azimuth change work-packages <id>` when `work-packages.md` exists.
3. Run the repository's normal `azimuth validate` command again. Distinguish newly introduced Findings from an explicitly recorded pre-existing baseline.
4. Run `azimuth change show <id>` and inspect the complete rendered account, then inspect the working diff for omissions, accidental files and conflicts with user-owned changes.
5. Audit the proposal against this question: can an implementer proceed without inventing product behavior, identity, ownership, compatibility, migration, failure or completion decisions? If not, resolve the gap or identify it as a blocking owner decision.
6. Present the change id, authored artifacts, intent Claims/Cases, central solution decisions, scope exclusions, validation results and any residual questions. Ask for explicit approval of the actual proposal files.

Do not implement, mark the change active, finalize, archive, commit or claim engineering results in the proposal-authoring turn unless the user separately and explicitly authorizes the applicable next action.
