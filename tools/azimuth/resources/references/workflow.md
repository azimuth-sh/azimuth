# Local change workflow

Azimuth computes eligibility and validates retained results; the harness executes or delegates the eligible work and enforces permissions. Workflow completion is not model authority, authenticated reviewer independence, deployment Requirement Status or permission to publish.

The `azimuth-workflow` skill operates the process. Task skills explain how to author/review the assigned artifacts. There is no additional coordination task between ordinary tasks.

## Initialize and retain exact handoffs

```text
azimuth workflow definition --workflow change-development [--out <file>]
azimuth workflow init --workflow change-development --inputs <file> --instance <file> --repositories <file> [--predecessor <file>]
azimuth workflow next --instance <file> --repositories <file> [--out <file>]
azimuth workflow update --instance <file> --repositories <file> --handoff <file> --actor <id> --result <file> --expected-revision <number> [--descriptor <file>] [--out <file>]
azimuth workflow verify --instance <file> --repositories <file> [--out <file>]
azimuth workflow abandon --instance <file> --expected-revision <number> --reason <text> [--out <file>]
```

Inspect the installed definition for exact action names, version, input schema, outcomes and output contracts. A custom strict definition uses the supported `--definition` interface. Existing instances pin their definitions; an installation update never silently changes an in-flight instance. Preserve abandoned/replaced state and its supporting records. Use explicit restart/migration contracts rather than deleting history.

`--change <directory>` is an alternative to `--instance`, resolving that directory's workflow instance; it is not a search across repositories. `--repositories` supplies an explicit ignored machine-local map from declared repository IDs to canonical checkout paths. A selected `--project` can supply the installed default map location. Portable record references retain repository identity and relative locator, not machine-specific roots. Workflow-local references resolve beside the instance. Relocation preserves their fingerprints and history; missing records remain blockers.

Read `workflow next` before executing work and capture its handoff. `eligible` contains complete work descriptions, typed targets, guidance, input/dependency basis, corrections and required output. Ordering is not priority. The engine never chooses a worker. Missing required inputs and unsatisfied obligations remain visible; an empty eligible set alone is not completion.

Perform only work whose actual permissions and prerequisites hold. Return one compact authored result using the handoff's exact schema and native output contract. Do not hand-author the engine's derived identity/fingerprint/dependency fields. Supply the original captured handoff and current coordination revision to update. Querying again after work does not recreate its starting basis.

Update validates exact current prerequisites and records atomically. Expected revision protects competing coordinators. An exact idempotent replay returns its recorded transition; key reuse with different content fails. Interrupted locks require inspection rather than automatic deletion. One coordinator owns shared progression when agents execute in parallel.

## Delta authoring and independent review

The ordinary local sequence is scope, proposal/review, spec delta/review, design delta/review, verification delta/review, engineering plan, actual implementation authorization, implementation, exact candidate preparation, required Checks, Challenges, Requirement Review, outcome, actual change acceptance, and combined finalization/archive. The installed definition is the executable authority; do not invent absent action names when resuming a pinned older instance.

Spec, design and verification are symmetric guarded transitions. Inspect both each delta and the composed result against its explicit baseline. Serial authoring reduces design/verification mismatch: a downstream discovery of a material upstream defect returns to that delta and invalidates affected downstream work.

Independent authoring reviews assess the original request, artifact, baseline/preview and dependencies directly. Reviewers remain distinct from authors. Finish the assigned scope even after the first blocker, retaining all material issues together. Stable authored review issue identities and dispositions follow the installed review contract; they are not deterministic model Findings. A later review must independently explain predecessor disposition.

Accepted proposal review permits delta authoring, not implementation. Implementation and final change acceptance require their actual human decisions against exact bases. Local actor labels and approval files never create permission on their own. Missing material inputs prevent acceptance.

Each agent task explicitly names its necessary skills/references in `guidance`. Resolve all of them before work and report missing resources. No one-skill-per-command rule is required: related tasks may share guidance and deterministic tool operations need executable contracts rather than an agent.

## Implementation, execution and correction

The implementation task follows the reviewed plan and complete candidate. Candidate preparation captures current source support, exact execution Verification Targets and necessary inputs, validates the prospective account and derives obligations. Directory presence cannot declare scope or completion. Whole-project authority is loaded even when execution selects a bounded affected subset.

Checks execute first, using native capture that retains exact targets, required observations, controls, context and unit coverage. Failed required Checks or incomplete execution block Challenges and Requirement Review. Required ordinary engineering gates remain obligations even when no non-routine Requirement enrolls them as Checks.

Challenges examine implementation and Check quality against exact examined definitions/source/configuration. They do not require an accepted Requirement Review merely to execute. Clean means bounded no-findings, never positive assurance. Execution gaps or confirmed defects return to diagnosis/correction. Proposed equivalence or irrelevance dispositions can proceed to independent Requirement Review; they do not relabel native findings as clean.

Affected selection is deterministic local/CI tooling, based on explicit baseline, cumulative candidate changes, dependency manifests, policy and outstanding obligations. Unknown impact broadens execution or blocks planning. Skipped work and reused evidence are distinct decisions. Existing hermetic-unit reuse follows its native inputs/context/applicability contract; no broad cross-target reuse is inferred.

Requirement Review assesses the complete implemented account, adequate verification, faithful realization, required local execution and proposed Challenge dispositions. Hosted prerequisites remain explicitly pending; a deployment-only obligation is not invented as a local result. Review rejection returns bounded issues to implementation, or to an owning delta if product/design decisions changed. Rerun affected execution before independent reassessment; retain unrelated current records under native rules.

Correction handoffs retain exact prior review/result basis, readable issues, stable issue identities, source locations, previous dispositions and currentness diagnostics. Currentness of retained inputs is not proof that the current checkout matches them. Implementers inspect sources and address the complete available issue set. Their reports identify each issue's changes and actual verification, justified disagreement or unresolved limitation. Do not add a mandatory correction-plan stage or silently weaken an obligation.

## Local packages, retention and finalization

Use local path-isolated work packages only for independently implementable engineering slices. Validate ownership/dependencies and generate briefs through the CLI. Reserve proposal/current-model integration, outcome, archival and shared files for the coordinator. Workers never take undeclared paths or finalize/archive independently.

Task output contracts define durable artifact locations and required dependency closure. Retain authored Markdown, sealed reviews, relied-on Runs, scopes, captures, expectations, acceptance and necessary history where tooling/CI can retrieve exact bytes. Scratch drafts can remain in ignored `.azimuth/`; depended-on inputs cannot exist there exclusively. Missing retention prevents task completion. Publish only the current relied-on package and relevant adverse history, not every exploratory attempt.

Outcome authoring records actual results and residual conditions. Finalization applies reviewed intent/account deltas when necessary, checks real acceptance and current reviews/evidence, emits the installed provenance/finalization records and archives the exact complete account. Final artifacts and relocated references need validation without semantic self-invalidation. There is no extra verify-change task.

CI validates the actual submitted or Mergify integration candidate, recomputes/validates affected selection and deterministically assembles publication inputs from retained records and actual identities. No agent prepare-publication-inputs task, local AAS credentials or fabricated upload receipt is required. Trusted publication preserves reviewer identity and native records; it neither authorizes deployment nor manufactures support.

CI reports and retains diagnostics only. It does not generate/report work packages, create correction assignments, dispatch agents or implement a Work Service protocol. Dima initiates correction explicitly. A correction preserves the original archived change and authors a related change against the current model, reusing the same PR before merge where appropriate. Conflicting baselines require reconciliation, never double application.

## Provenance and assurance boundary

`--actor` names one execution instance, not a role, attempt number or model. Follow `azimuth reference show work-provenance` for hook capture and optional descriptor behavior. Never invent harness values, reviewer identities, timestamps, permission or execution facts. Provenance describes work and is not an assurance input.

AAS computes deployment Requirement Status after authorized rollout from exact lineage, observed composition, applicable evidence and compatible fresh context. Neither workflow completion, passing CI, accepted review nor clean bounded Challenge alone establishes deployed behavior. Historical records preserve their original meaning and are not rewritten into new approvals.
