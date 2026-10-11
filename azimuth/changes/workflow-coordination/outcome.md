# Implementation outcome

The bounded initial increment is implemented in source. The change remains active; its intent has not been applied to the accepted model, and it has not been finalized or archived.

## Delivered

- Explicit workflow initialization, read-only eligible handoffs, conditional result updates and state verification.
- Independent intent/design/verification authoring review records, dependency freshness and correction/reassessment loops.
- Native Method Qualification, Applicability Decision and Claim Judgment integration using exact candidate snapshots and Subjects.
- Immutable candidate model revisions and revision-scoped authenticated reviews and assessments within an existing ASS project, preserving accepted authority.
- CLI reference, skills and generic contracts. No consumer-specific content was added.

## Engineering evidence

The CLI binary builds with `cargo build --locked --manifest-path tools/azimuth/Cargo.toml --bin azimuth`. The service compiles with `cargo check --locked --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server --bin run-ledger`. `git diff --check` is clean.

Product-command inspection confirmed workflow reference registration and a local initialization → author handoff → recorded draft → independent review handoff sequence. The example is held outside the repository and establishes command behavior only; it is not a genuine authoring review or assurance record.

`change validate workflow-coordination` and repository `validate` report structurally valid accounts. Nine existing unresolved design-binding evidence gaps remain because source manifests were not supplied. This is not full acceptance. Tests were neither written nor run under the canonical repository instruction.

## Remaining boundaries

The live service and database were not changed. Service-backed exercise requires publishing/deploying the updated service and applying its additive migration. Candidate browsing in the Web UI is not implemented. The harness supplies authenticated service transport and actual human/agent identities; local labels do not authenticate them.

Task graphs are resolved from explicitly selected reusable definitions and declared inventories. Source capture execution, operational-data exclusion from source identity, archive handoff, federation finalization, promotion/deployment gates and production scheduling remain outside the coordinator. Workflow completion alone cannot accept the account.

## Portable execution locations

Workflow instances retain no checkout paths. Commands accept a local repository map through `--repositories`, or resolve `azimuth/local-repositories.json` beside an explicitly selected `--project` file. Repository-free references resolve within the instance directory. Resolved repository locations and workflow directory accompany each eligible handoff without affecting its basis. The prior alpha root-bearing shape is replaced; no compatibility aliases were added.

Product-command inspection recorded a local authored artifact at revision 1, copied the complete instance directory, and supplied another checkout containing identical referenced bytes. The relocated query retained revision 1, the current authored result and the same review basis. Default project-relative repository-file selection also worked. This was command inspection only, not an actual product review or assurance submission. No tests were written or run.

## Standard declarative change workflow

The CLI now ships a reusable version-1 `change-development` template, revision 1, in the embedded workflow registry. `workflow definition --workflow change-development` exposes the exact template. `workflow init --workflow change-development --inputs <file> --instance <file> --repositories <file>` pins its source, canonical fingerprint, portable input parameters and concrete action/target expansion. A package may supply another strict template through `--definition <file> --inputs <file>` using the same engine. Local repository paths remain external.

The standard process owns scope/proposal, per-module spec authoring and independent review, accepted-intent design/verification authoring and independent reviews, engineering planning, actual implementation approval, implementation, source support, candidate validation/registration and validated assurance inventory. Later inventories validate native same-project candidate snapshots and declared Check–Case/Claim relationships before instantiating qualification, exact-Subject execution/applicability and non-routine Claim Judgment tasks. Context-specific instructions are referenced hashed inputs; they do not invent task names or bypass a gate. Bounded declared outcomes and current-success dependencies govern progression. Qualification and authoring review remain separate record classes.

Concrete identities are structured `{action, target}` references, not target digests. The `next` response omits `next`, `blocked`, `current` and `superseded` entirely; eligibility and currentness remain internal assessments. Correction registering another immutable candidate appends assurance tasks while retaining earlier definitions/results as superseded history. Missing instructions/snapshots make affected tasks ineligible, and malformed/unknown inventories fail without progression. The alpha spec-action spellings replace the prior intent-action spellings without compatibility aliases. No service API was changed by this extension.

Permitted evidence: `cargo build --locked --manifest-path tools/azimuth/Cargo.toml --bin azimuth` succeeded. Synthetic local product-command inspection instantiated two modules (21 initial tasks), retained a rejected spec review and fresh author/reviewer correction, reached candidate inventory registration, expanded four assurance tasks, retained a native synthetic rejected Method Qualification, reopened implementation, and added another candidate generation at revision 29 while preserving the previous four tasks as superseded. An exact inventory-result retry returned its original revision 23 without progression; repeated `next` preserved instance bytes. An unknown outcome failed with definition/input locations, and missing referenced instructions produced blockers with no eligible tasks. These synthetic records establish command behavior only, not real review, permission, registration or assurance. No tests were written or run.

`change validate workflow-coordination` remains structurally valid, with the same nine unresolved source-binding evidence gaps and explicit acceptance blockers. Accepted model integration, acceptance and archival have not occurred. `change work-packages workflow-coordination` accepts the declared ownership, and `git diff --check` is clean.

## Departures

The initial finite-graph-only increment now includes the explicitly requested reusable declarative templates and shipped standard workflow. Module targets are declared before initialization and confirmed in the scope record; assurance targets expand later only through validated inventories. There is no directory-based process discovery, scheduler, transition script or executable plugin boundary.

## Residual decisions

The harness must establish module scope completeness, obtain real permission, launch independent actors and authenticate service snapshots/registration. Current candidate target inventories and their snapshot references remain fixed per revision; snapshot content may be refreshed, while a different registered candidate adds a new generation. Core is not an identity provider or adversarial state store. Ordinary source/support authoring and candidate validation outputs retain accountable content but do not acquire a fabricated semantic validator. Live service rollout remains separately authorized work.


The readable-reference adjustment replaces all public concrete work references in definitions, handoffs, result/review/approval records and history with `{action, target}`. Existing hashed-reference instances are preserved rather than automatically migrated. Fresh instances require reassessment; no historical result or basis is rewritten. Artifact, basis and definition content fingerprints remain unchanged in purpose. The CLI was built and inspected through fresh synthetic product commands; no tests were written or run.
