# Workflow implementation plan

Status: implementation authorized directly by Dima; no Azimuth change/proposal process for this work.

## Scope

Implement the local change-development workflow, focused skills, symmetric delta authoring, pre-review Check/Challenge execution, structured correction findings, deterministic affected execution planning, durable retained inputs, combined completion/archival, and deterministic consumer CI/publication integration. Synchronize the completed canonical skill cohort into drim-dev before consumer agents use it.

Azimuth Work Service and its assignment/API/DB/checkpoint protocol are deferred. Its exploration is approved as a design record, not implementation authorization. CI will retain ordinary diagnostics and actual execution records only. CI must not generate or report work packages, create work requests, assign agents, or initiate correction changes. Dima sees CI failures and starts correction work manually.

## Parallel ownership

| Lane | Exclusive implementation ownership | Outputs |
| --- | --- | --- |
| A: affected execution | `tools/azimuth/src/run_selection.rs`, `unit_inputs.rs`, new `execution_impact.rs`; `contracts/run-selection.md`, `unit-inputs.md` | Strict deterministic baseline/candidate/pending selection, conservative widening, explanations and exact targets; no agent and no blanket evidence reuse |
| B: Challenge/review records | `tools/azimuth/src/run.rs`, `run_plan.rs`, `assurance_review.rs`, `review_authoring.rs`, `verification.rs`, `account_verification.rs`; relevant Python/TypeScript Challenge producers; matching Run/review contracts | Implementation/Check-directed Challenge targets executable before first review; retained Check/Challenge input basis; stable review issues/dispositions |
| C: focused skills | `tools/azimuth/resources/skills/` and the assigned proposal/delta/design/verification/workflow/review/outcome references | Cohesive canonical task skills and metadata; no registry or consumer edits |
| Coordinator | Shared CLI/module wiring, workflow engine/template, resource registries/installer, candidate/finalization/retention/correction integration, AAS integration, remaining contracts, consumer CI/publication and managed skill synchronization | Integrated graph, complete native outcome gates, durable closure, correction lineage, consumer rollout |

Workers do not edit another lane's files, shared registries, current model facets, or Git state. Changes to public interfaces are communicated before dependent edits. No worker commits, finalizes, archives or publishes this implementation. Additional paths require explicit reassignment by the coordinator. Reviewer independence is preserved; parallel implementation is not independent review.

## Sequence and integration barriers

1. Freeze strict affected-planner and implementation-Challenge interfaces. Preserve existing symmetric intent/account projectors; do not replace them with a competing delta grammar. Keep model Findings distinct from authored review issues.
2. Run A, B and C concurrently. Coordinator integrates the outer graph and completion contracts against the agreed interfaces. Incomplete intermediate work is not shipped to consumer agents.
3. Integrate Challenge/review outcomes with workflow eligibility. Failed/violated/inconclusive required Checks block Challenges and review; Challenge execution gaps and confirmed defects block review. Candidate equivalence/irrelevance dispositions remain independently reviewable rather than requiring a fabricated clean Run.
4. Compose spec, design and verification deltas against an explicit baseline; preserve guards and exact previews. Serialize authoring/review: proposal, spec delta, design delta, verification delta, engineering plan and real implementation approval. Design/verification review inspects the composed account.
5. Retain the exact inputs required by review/Run records at their producing task. Validate the portable closure before successful completion and again in finalization/CI. Finalization consumes actual Challenge proofs; no ignored-local-only required dependency or stale-positive fallback.
6. Add outcome authoring, human change acceptance and one finalization task including archival. Resolve provenance closure without including a result recursively in its own digest. Remove `verify-change` and agent-authored publication preparation from the standard graph.
7. Support explicit correction changes linked to an earlier archived change, including corrections to existing Requirements. Preserve immutable earlier records and refresh affected reviews/acceptance. A pre-merge correction joins the existing PR; a post-merge correction uses a new PR. No automatic CI work creation.
8. Wire the deterministic affected planner into the CLI and CI. Derive complete inventories from actual account/source inputs. Track cumulative outstanding obligations; missing dependencies widen or fail rather than silently select nothing. Reuse existing hermetic unit contracts only where they authorize evidence reuse.
9. Assemble publication inputs in trusted CI from the complete retained account, actual revisions, native latest reviews and evidence. Do not filter accepted records before selecting the latest assessment. PR validation remains unprivileged and never authors reviews or infers deployment success.
10. Update drim-dev candidate validation to include active and newly archived changes, and expose one stable `azimuth-ready` gate. Mergify temporary PRs must run it. Keep current remote Mergify configuration until the complete replacement gate is integrated; do not overwrite newer remote files from the stale local checkout.
11. Register all completed canonical skills and metadata in the embedded CLI resources/manifest. Retire broad propose/coordinate/apply discovery only after their responsibilities and callers are replaced. Build the exact CLI and use its managed update path to synchronize both Codex and Claude resources into drim-dev, including installation digests. Verify canonical and installed contents match; preserve custom project skills and integration configuration.
12. Inspect integrated diffs and use repository-permitted product commands/build checks. No commits, pushes or deployments are implicit. Repository restrictions on test execution remain in force unless Dima explicitly authorizes tests.

## Intended local workflow

Scope → proposal/review → spec delta/review → design delta/review → verification delta/review → engineering plan → implementation approval → implementation → capture/validate/prepare candidate → Checks → Challenges → Requirement Review → outcome → change acceptance → finalize and archive.

Corrections repeat affected work only. Native outcomes, required unit coverage and current dependencies determine gates; successful command execution is insufficient. Routine Requirements retain their existing independent-review exemption. Hosted-only Checks remain explicit later obligations. An empty eligible set is not completion when required inputs or obligations remain missing.

## Completion evidence

- Standard definition has serial symmetric author/review tasks, pre-review execution and explicit completion tasks, with resolvable skill guidance.
- Required adverse/incomplete execution cannot unlock downstream work; raw Challenge findings have an independent disposition path.
- Affected planning is deterministic, explains every inclusion/exclusion and conservatively handles unknown, shared, removed and cumulative dependencies.
- Review input retention and finalization preserve exact proof dependencies and newer adverse assessments.
- Linked correction changes can reassess affected existing Requirements without modifying the original archive.
- Consumer CI validates the combined candidate, publishes no assignments and assembles exact publication inputs through existing authorized transports.
- All new skills and metadata are present in canonical resources and in drim-dev's managed Codex/Claude installations before agents use them.
- Build/product validation reports actual outcomes and remaining integration limits. Hosted CI variables or unreleased tooling pins are reported explicitly rather than invented.
