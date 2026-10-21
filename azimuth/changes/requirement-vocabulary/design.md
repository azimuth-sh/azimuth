# Design

This change renames domain terms across contracts. It adds no mechanism, so there are no Claim-bound mechanisms to declare. The decisions below fix identity, wire, storage and rollout semantics that implementation would otherwise decide.

## Principles

- Presentation, serialized names, canonical bytes and schemas are classified separately; a presentation rename never changes the identity of a Requirement, Scenario, Check, Run, Result or Verification Target.
- Wire formats whose key set or `format` string changes take a new version. A renamed format continues its version counter (no restart at 1), so a frozen vector can never match two meanings.
- No aliases, fallback parsers, dual writes or legacy endpoints. Old producers fail closed against new consumers and vice versa; the rollout stops producers before the reset.
- `Observation` stays for runtime facts. Classification is by meaning, not by file or type name.
- Stable Claim and Case identifiers are preserved.

## Terminology matrix

| Boundary | Old | New |
|---|---|---|
| Authored spec | `## Claim:`, `### Case:` | `## Requirement:`, `### Scenario:` |
| Intent delta | `Add/Replace/Remove claim`, `Add/Remove case`, `Remove cases:` | `Add/Replace/Remove requirement`, `Add/Remove scenario`, `Remove scenarios:` |
| Account facets | Claim design, Case design, Claim verification, Case verification, Claim review | Requirement design, Scenario design, Requirement verification, Scenario verification, Requirement review |
| Verification metadata | `Claims:` (on Challengers), `Case` bindings | `Requirements:`, `Scenario` bindings |
| Authored Challenger | `### Challenger:`, `Add challenger` | `### Verification Challenge:`, `Add verification challenge` |
| Run | `subject`, `subject_fingerprint`, `observations[]{case,...}`, `challenger_executions`, `required_context`, `context` | `verification_target`, `verification_target_fingerprint`, `results[]{scenario,...}`, `verification_challenge_executions`, `required_execution_context`, `execution_context` |
| Review kinds | `claim-review`, `claim-health`, `claim-judgment` | `requirement-review`, `requirement-status`, `requirement-judgment` |
| Review fields | `claim`, `claim_fingerprint`, `claim_review_fingerprint` | `requirement`, `requirement_fingerprint`, `requirement_review_fingerprint` |
| Deployment status | `claims[]` per-claim `claim`/`status` | `requirements[]` with `requirement`/`status` |
| Export | `claims`, `cases`, `challengers`, `claim_judgments` | `requirements`, `scenarios`, `verification_challenges`, `requirement_judgments` |
| Extractor manifests | `realizes[].claim`, `elements[].claim`, `contributions[].case` | `realizes[].requirement`, `elements[].requirement`, `contributions[].scenario` |
| Annotations | `Realizes(claim)`, `SupportsVerificationElement(claim, ...)`, `ContributesCheckToCase`, `implementsChallenger` | parameters named `requirement`; `ContributesCheckToScenario`, `implementsVerificationChallenge` (and per-language equivalents); `Realizes`, `ImplementsCheck`, `ImplementsMechanism` keep their names |
| Execution Context report | `azimuth-verification-context` v1, `/internal/verification-context` | `azimuth-execution-context` v2, `/internal/execution-context` (decided with Dima) |
| Fingerprint formats | `azimuth-subject-fingerprint`, `azimuth-observation-fingerprint`, `azimuth-case-digest`, `azimuth-claim-digest`, `azimuth-claim-judgment-fingerprint`, `azimuth-challenger-fingerprint`, `azimuth-context-fingerprint` | `azimuth-verification-target-fingerprint`, `azimuth-check-result-fingerprint`, `azimuth-scenario-digest`, `azimuth-requirement-digest`, `azimuth-requirement-judgment-fingerprint`, `azimuth-verification-challenge-fingerprint`, `azimuth-execution-context-fingerprint`; each version incremented |
| CLI | `change account-*` flags and `--subjects`; workflow tasks `review-claim`, `assess-claim-health`, `judge-claim`, `run-challenger(s)` | `--verification-targets`; `review-requirement`, `assess-requirement-status`, `judge-requirement`, `run-verification-challenge(s)` |
| Skills / references | `azimuth-review-claim`, reference `deployment-claim-status` | `azimuth-review-requirement`, `deployment-requirement-status` |
| Migration line | `alpha-claim-case` | successor line with a migration edge that rewrites authored syntax |
| ASS routes | `/claim-reviews`, `/subjects`, `/model-revisions/{r}/subjects/{s}/state` | `/requirement-reviews`, `/verification-targets`, `.../verification-targets/{t}/state` |
| ASS legacy gate API | `/observations`, `/challenges`, `/gates/*`, `/work-items`, `/definitions`, `/qualifications`, `/model-snapshots`, `ClaimContract` in `lib.rs` and the domain crate | deleted with Dima's explicit permission (not built into any image, superseded by the run-ledger API); the shared domain types the run ledger still uses are renamed |
| ASS storage | kinds `assessment_claim`, `subject`, `observation`; tables `assessment_claims`, `assessment_cases`, `definition_claims`; columns `claim`, `case_id`, `subject`, `subject_fingerprint`; dependency kinds `claim-review` | `assessment_requirement`, `verification_target`, `check_result`; `assessment_requirements`, `assessment_scenarios`, `definition_requirements`; `requirement`, `scenario_id`, `verification_target`, `verification_target_fingerprint`; `requirement-review` |
| UI | Claims, Cases, Subjects; "Claim Status/Review"; Confirmed / Violated / Unresolved; routes `/claims`, `/subjects/...`, `?claim=` | Requirements, Scenarios, Verification Targets; "Requirement Status/Review"; Satisfied / Not satisfied / Unresolved; `/requirements`, `/verification-targets/...`, `?requirement=` |
| drim.dev | `RealizesAttribute.Claim`, `VerificationContext`, `VerificationContext:Port`, `azimuth/verification-contexts/` | `Requirement`, `ExecutionContext`, `ExecutionContext:Port`, `azimuth/execution-contexts/` |

Kept unchanged: `deployment_observations`, `azimuth-deployment-observation`, `azimuth-target-observation` (raw transport envelope; its payload `subject` fields become `verification_target`, and a `kind:check` payload is normalized into a Run whose Result is a Check Result), `assurance-target-observations`, context captures and `/context-contracts`, `/context-captures`, Challenge Schedule, Plan, Selection and Result, `azimuth-run-bundle`, `azimuth-run-launch-plan` names (version bumped), release provenance and attestation `subject`, OIDC `sub`, Rust/Go/Java/C#/shell `case`, `Requirements:` in Check `Execution:` metadata and `required_receipts[].requirement` (existing unrelated meanings; each collision is resolved explicitly during apply and recorded), `ClaimContract.requirement` string value `behavior`.

## Status and enum semantics

Requirement Status is the computed deployment conclusion. The surviving exact-target record kind `claim-health` becomes `requirement-status` and is described as a Requirement Status record for one Verification Target. Internal assessment values `supported | violated | unresolved`, Check Result outcomes `satisfied | violated | inconclusive`, freshness `ready | pending | expired`, review assessment `accepted | stale | rejected | missing` and binding/coverage statuses keep their serialized values; only labels change. Because the new "Satisfied" label would collide with the Check Result outcome `satisfied`, the UI labels Check Result outcomes "Passed / Failed / Inconclusive". A passing Check or accepted Review alone never implies a satisfied Requirement.

## Requirement assurance record

"Claim assurance account" has no first-class aggregate. The Claim-first account is the per-Claim cluster of authored module documents (`spec.md`, `design.md`, `verification.md`, `judgments.md`) assembled by `account_model` (definition-time; no Verification Target or Observation). The ASS holds Reviews, Runs, Results and derived assessments separately. "Requirement assurance record" names that logical collection only; no new table, API resource or lifecycle is introduced. Where "account" names a broader scope (installation account, project account, repository account), the scope is preserved and described accurately.

## Contract and identity strategy

| Change | Class |
|---|---|
| Docs, labels, CSS class names, UI routes and anchors | presentation; no identity effect |
| JSON keys, `format` strings, `kind` tags, closed vocabularies, CLI flags, routes | serialized/wire; coordinated producer and consumer upgrade; version bump |
| Run subject/observation/challenge keys and fingerprint format strings | change canonical bytes: `verification_target_fingerprint`, `run_id`, launch, selection, result and challenge-selection identities all change |
| Claim/Case digest keys, authority model digest, export JSON, review record fingerprints | change canonical bytes; every Requirement, Scenario, binding, judgment, review and authority fingerprint changes |
| ASS registry signature | changes automatically; schema version 5 → 6 |
| Embedded skills, references and templates | change `azimuth/installation.json` resource digests; need a migration edge |

Frozen-vector tests in `tests/run.rs`, `run_plan.rs`, `verification.rs`, `decision_scope.rs` and `adapter.rs` are regenerated from the renamed contracts, never edited to match old bytes. The Python protocol mirror (`azimuth_run`) and the services' hand-mirrored shapes are verified byte-identical to Rust. Export contract version is reconciled between `model.rs`, `contracts/export.md` and `release/artifacts.json` (currently 5 versus 4); the intended value is determined during apply and recorded.

Historical material is never rewritten or re-fingerprinted: archived changes, `contracts/history`, `azimuth/changes/*/history`, numbered reviews, retained release records and input envelopes in the GitOps repository, `azimuth/releases/accepted.json`, `baseline-selection.json` and the gitignored `.azimuth/` runtime artifacts. The retired-syntax check is replaced by rejection of the old vocabulary in active artifacts; archived paths are excluded from validation. `release_pipeline.py resume` and `records` must never be fed pre-rename `.azimuth/` outputs. Stale `accepted.json`/`baseline-selection.json` are not re-derived; a fresh baseline is registered through the owner-authorized workflow, and any owner-only baseline decision is surfaced, not invented.

Authored artifacts in consuming repositories migrate through `azimuth migrate plan/apply` with a new migration line; review-required items are resolved by hand and replanned.

## Fresh ASS and reset boundary

Schema version becomes 6. The version 1–4 upgrade branches and their hard-coded signatures (dead on a fresh database) are deleted. Migrations `0001–0003` and `assessment_triggers.sql` are rewritten in place with the renamed identifiers because only fresh initialization is supported; the unreferenced `migrations/0001–0008` document-storage generation is left as is unless a reference is found. The server refuses any database that is not fresh or whose signature differs, as it does today.

Reset boundary (confirmed again against live state immediately before execution): database `assurance_state_relational` only. The legacy `assurance_state` database is inspected for data first and is reset only if it is ASS-owned and in use; it is otherwise left alone. PVC contents of `assurance-collector` (`/state/collector.db` outboxes and checkpoints), `deployment-reconciler` (`/state/reconciler.sqlite`) and the `private-api-probe` volume are cleared by emptying contents; the PVCs themselves (Terraform `prevent_destroy`) are not deleted. Collector outboxes retry rejected rows forever and would block capture at `max_pending`, so they are cleared in lock-step. Excluded: the drim.dev business and Umami databases, the `postgres-cluster` cluster, backups and volumes, all credentials Secrets, Terraform state, Tailscale, TLS, kubeconfig. If a concrete operation exceeds this boundary, work stops for approval. No backup is taken, by Dima's decision: ASS starts empty, and all ASS data is deleted with no baseline carried over; the fresh baseline is registered through the authorized workflow. Release acceptance stays pending. Recovery is re-initialization and republication from source plus a re-run of the pipeline, not image rollback.

## Rollout order

1. Core and contracts, then packages and extractors, then ASS, Collector and Reconciler, then release tooling; images and packages published from CI.
2. Update drim.dev (annotations, `ExecutionContext` endpoint, migrated artifacts, tooling pin `tooling_revision` and `AZIMUTH_RELEASE_REVISION`), GitOps and infra pins.
3. Stop producers (collector, reconciler, probe, CI publication), reset the boundary above, apply the reviewed Terraform plan with the new image digests, start ASS and confirm fresh initialization, start producers.
4. Execute Checks, publish Runs and Results, create and deploy the new drim.dev release, verify reconciliation, Execution Context capture, Monitor delivery, Reviews and Requirement Status in production ASS. Report API and browser timings separately.

An old collector polling the renamed app endpoint receives 404, so the app, collector and test packages move together in step 2–3.

## Rejected alternatives

- Compatibility aliases or dual-read parsers: rejected by repository policy and unnecessary for a fresh ASS.
- Renaming stable identifiers that contain old words: breaks identity continuity and is a remove-and-add decision.
- Pure global text replacement: would corrupt unrelated `case`, `subject`, `context` and Observation uses and silently change digests.
- Keeping Health as a separate term: rejected by Dima.
- Prefixing the whole Challenge family: rejected by Dima.
