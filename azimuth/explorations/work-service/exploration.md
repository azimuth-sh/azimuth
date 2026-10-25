# Exploration: Azimuth Work Service

Id: work-service
Created: 2026-10-25
Status: approved

## Objective

Build Azimuth Work Service: a separate Rust service with its own PostgreSQL database that accepts work from people, CI failures and adverse Azimuth Assurance Service (AAS) assessments, assigns whole requests to external workers, and coordinates their outer lifecycle through an explicit completion boundary.

For a production correction, delivery of a patch is not completion. The objective remains open through local engineering, CI, release, deployment and applicable AAS confirmation that the Requirement is Satisfied again. Explorations and other requests may have shorter declared completion boundaries. Independent requests can share a delivery batch without sharing a single completion verdict.

The first implementation proves the backend and protocol through reference worker and adapter clients. It does not require an LLM, production provider integration, UI or cloud deployment.

## Boundaries

- Separate service and owned database; no shared AAS tables. Follow applicable AAS Rust/PostgreSQL conventions while keeping the HTTP protocol language independent.
- The service owns work coordination, not agent processes, local engineering task execution, assurance interpretation or delivery pipelines.
- Each worker claims an entire request and runs the local Azimuth workflow in its own workspace. Independent reviewers can be separate agents within that harness; a single claimant does not establish review independence.
- Intermediate artifacts remain local until needed for an ownership-release checkpoint, required retained output or external handoff. Another worker must be able to restore a released request from its retained continuation state.
- First-version interfaces are HTTP JSON with polling. Projects, pools, credentials, policies and lifecycle definitions are operator-managed files, validated at startup.
- First-version storage is PostgreSQL for metadata and a service-managed persistent filesystem for bundles, accessed through the API. No AWS infrastructure dependency is introduced.
- First-version delivery batches are explicitly created through an API. Automatic scheduling can later call the same interface.
- No UI, embedded agent runtime, task-level worker assignment, streaming transport, production GitHub/Mergify/AAS adapter, automatic batch scheduler or cloud infrastructure provisioning enters the initial scope.
- Simulation is explicitly identified and cannot become real assurance evidence.
- This exploration is non-normative. It approves no model change, implementation, commit, publication, deployment or archival. The actual files still require approval before this anchor becomes approved.

## Existing context

Repository inspection on 2026-10-25 established the following facts. Paths refer to this repository unless a consumer is named.

- **F1 — AAS computes assurance, not worker assignment.** `contracts/deployment-requirement-status.md` defines computed deployment Requirement Status, applicability, currentness and freshness. `contracts/run-ledger.md` defines retained Runs, reviews and publication. A failed execution is not automatically a violated Requirement.
- **F2 — The intended architecture anticipates externally triggered correction.** `docs/intended-architecture.md`, especially Production maintenance loop and Resolution history, places dispatch and recovery outside AAS and requires evidence for the corrected deployment before resolution. This is derived direction, not an implemented work-assignment contract.
- **F3 — A local workflow engine already exists.** `contracts/workflow.md` and `tools/azimuth/src/workflow_state.rs` define pinned finite graphs, eligible handoffs, result dependencies, correction paths and revision-controlled updates. They do not establish a distributed whole-request claim/lease service or this outer lifecycle.
- **F4 — Process provenance already has its own boundary.** `contracts/work-provenance.md` records workflow attempts without participating in assurance assessment. Work Service activity must not silently redefine that contract or substitute for review independence.
- **F5 — The backend ecosystem exists.** `services/assurance/server/Cargo.toml` uses Rust, Axum, Tokio, SQLx and PostgreSQL. Reusing conventions does not imply sharing tables or selecting exact Work Service dependencies without proposal review.
- **F6 — Execution scope and evidence reuse are distinct.** `contracts/run-selection.md` excludes general affected-only optimization. `contracts/deployment-requirement-status.md` separately defines bounded hermetic unit reuse. Neither is a generic correction scheduler.
- **F7 — Publication and deployment have distinct authority.** `contracts/release-publication.md` defines explicit release inputs and publication; publication creates neither deployment observations nor agent permission.
- **F8 — The inspected account baseline is structurally clean.** The installed `azimuth validate` invocation reported 50 Requirements in 12 Specs, no Findings, zero errors and zero warnings. This is structural validation, not execution or proof of this proposed service.

### Inferences

- **I1 — One assignment protocol can serve several origins.** New requests, CI failures and adverse AAS situations share ownership and recovery mechanics while retaining different inputs and completion conditions.
- **I2 — Request lifetime must exceed worker lifetime.** CI, decisions, batches and production confirmation can wait without occupying an agent. Durable continuation is therefore necessary at ownership-release boundaries.
- **I3 — External action delivery needs reconciliation, not an exactly-once claim.** A lost acknowledgement can follow a real effect. Stable operation identity and provider reconciliation are necessary before retries.
- **I4 — Existing workflow machinery is a reuse candidate, not a proven fit.** Outer lifecycle activities differ from local authoring tasks; inspect the engine before choosing an extension or a separate interpreter.

## Decisions

### D1 — External workers claim whole requests

Workers claim rather than receive inbound execution calls. The service never launches agent processes. One claimant coordinates the request's local workflow, including independent reviews. This avoids service-mediated artifact exchange between every local task. Claims are finite attempts, not permanent ownership of the request.

### D2 — Common intake, separate service, Rust and PostgreSQL

People, CI and AAS use a common intake envelope with source-specific context. A submitted desired behavior is a work request, not accepted Requirement intent. The service is separate from AAS, owns its database and uses Rust/PostgreSQL conventions. First-version interfaces require no UI.

### D3 — Outer lifecycle and local workflow remain distinct

The worker owns its local Azimuth workflow. The service tracks request state, worker attempts, decisions and external milestones; it does not reproduce every local authoring/review transition. It can require retained results without interpreting free-form engineering prose.

Each request pins a validated, versioned outer lifecycle definition describing agent work, external actions, waiting conditions, gates, correction paths and completion. Definition upgrades never silently change active requests. Initial definitions are a small shipped set; arbitrary script execution or agent interpretation of prose is not the orchestration mechanism. Inspect existing engine reuse in C1.

### D4 — Project-scoped credentials and named worker pools

Bearer credentials identify submitters, workers and owners with explicit permissions. Submitters submit and inspect permitted requests; workers claim, renew, report and retain work; owners manage policy decisions, cancellation and recovery. One credential can hold several permission groups.

Owners configure worker membership in named pools. Requests name a pool, initially `default`. Workers cannot acquire pool access through self-declared capabilities. The service atomically selects the oldest eligible request across the worker's requested permitted pools; exact ordering and tie-breaking are contract design work.

### D5 — Lease expiry requires owner-controlled recovery

Attempts have explicit identities, renewable leases and ownership generations. Mutations require current ownership. Lease expiry revokes the attempt's mutation authority and marks the request recovery-required; it does not automatically reassign work or assert that the old process stopped.

An authorized caller reconciles repository branches, PRs, checkpoints and external operations before making continuation claimable. Another worker may then restore the retained state. Stale attempts cannot overwrite newer state.

### D6 — Generic human decisions suspend work with a checkpoint

A worker submits the exact question, options, basis and continuation checkpoint. Only after the checkpoint is durably accepted does suspension release ownership. An authorized owner responds through the API; the response retains identity, choice, rationale and exact basis. The resumed worker records that decision in the local workflow.

The service does not duplicate typed proposal/review approval rules. A generic response neither authorizes unrelated actions nor remains valid after a material basis change. The relevant local workflow and action contracts enforce that relationship.

### D7 — Filesystem bundles and PostgreSQL metadata

Workers upload through an API. The service stages and verifies bundle bytes on a persistent filesystem and records manifests, digests and references in PostgreSQL. Interrupted uploads never appear as usable checkpoints. A storage interface permits a future object-storage backend without requiring it now.

A reconstructible checkpoint includes exact repository revisions, required source availability, uncommitted changes, local workflow state, required artifacts and continuation context. A transient merge-queue branch name alone is insufficient. Bundles exclude credentials, which remain supplied by execution environments. Restore validates digests, dependency closure and extraction path containment.

Uploads must complete before ownership release whenever continuation needs local work, including waits for external automation. Required final outputs and historical inputs also receive explicit retention contracts. Temporary artifacts need not all be uploaded. Cleanup cannot silently break retained references; backup and restoration coordinate database metadata and filesystem content.

### D8 — End-to-end objectives and independent batches

Production correction objectives remain open through CI, release, deployment and exact AAS confirmation. Workers release leases while deterministic automation or evidence is pending. `unresolved` requires diagnosis of missing evidence, freshness or review gaps; it is not a command to modify production code. A Requirement violation does not authorize weakening its intent.

Other request kinds declare their own completion boundary. Worker completion of an agent stage is not necessarily completion of the request. AAS owns the authoritative assessment and assurance-resolution facts; Work Service tracks their relationship to the objective without inventing an assessment.

Requests remain independent and can join a shared release/deployment batch. An authorized caller creates and dispatches a batch through explicit APIs. Validate readiness, compatible delivery inputs/targets and authorization; seal membership and inputs at dispatch. Outcomes and confirmation are evaluated per request, including partial batch success. Subsequent changes form another batch. Automatic batch formation is deferred.

### D9 — Generic milestone adapters and durable external actions

Integrations report authenticated milestones with source/event identity, exact targets, correlated operation and retained results. Deterministic lifecycle rules decide to wait, continue agent work or complete. Duplicate/late events remain attributable to their original operation and cannot advance unrelated attempts. Reconciliation recovers missed notifications. First-version reference clients exercise these APIs without live providers.

The service durably requests authorized CI, release and deployment operations through adapters rather than embedding pipelines or recalling an agent just to trigger automation. Requests carry exact inputs and stable idempotency identities. Adapters poll an outbox, invoke external systems, reconcile uncertain execution and report results. Acceptance for execution is not successful execution. Accept operations initiated externally as well.

PostgreSQL transactions record state and outbox effects together. Delivery can repeat; adapters deduplicate or reconcile before repeating effects. Incoming milestones have a durable deduplicated inbox. Lost acknowledgements never justify blind retry of an irreversible operation.

### D10 — Project policy and request-specific grants

Project policy bounds permitted actions; request grants can narrow authority or record explicitly approved exceptions. Claiming or submitting work grants no implicit merge/deployment permission. Actions outside existing authority require a recorded owner decision. Recheck current authorization before claiming or dispatching, including after policy revocation/restart; adapters enforce their own credentials and infrastructure permissions.

Persist the decision/grant basis used by an action. Keep secrets out of request/checkpoint payloads. Authentication proves the submitting principal, not semantic truth; lifecycle contracts name authoritative sources for milestones. Simulated principals/results cannot satisfy real assurance conditions.

### D11 — Explicit submission and problem identities

Submitters provide an idempotency key for an API operation and a separate stable correlation key for repeated observations of the same problem. Identical replay returns the recorded result; key reuse with different content fails. New observations are retained without silently rewriting an active assignment's basis. A recurrence after confirmed resolution creates a new linked occurrence.

Do not infer semantic equivalence from similar text or merge unrelated CI/AAS failures automatically. Exact namespace, occurrence and conflict rules are C1 contract work.

### D12 — Cooperative cancellation and bounded attempts

Cancellation stops new dispatches, notifies active participants through polling and allows a bounded interval to stop and report effects. Unknown outcomes require recovery. Cancellation does not assert an operation was stopped or undone. Dispatched shared batches retain their membership: cancelling one request does not automatically cancel every member's delivery, and actual results remain recorded.

Project policy separately bounds agent correction attempts, infrastructure/adapter retries and release/deployment attempts. Polls and lease renewals consume no attempt budget. Replayed deliveries are not new deployments. Exhaustion suspends for an owner decision; it neither resolves the problem nor automatically permits more attempts. Owners can authorize continuation, revise scope or cancel.

### D13 — HTTP polling and operator-managed configuration

Use HTTP JSON endpoints for intake, claims, renewal, progress, decisions, checkpoints, outcomes, external-action polling and milestone reporting. Correctness depends on persisted state and identities rather than connection lifetime. Streaming is deferred.

Validate operator-managed project, pool, credential, policy and definition files at startup. Configuration changes use explicit restart/reload behavior defined in proposals. Pin definition content and retain relied-on policy/decision bases; evaluate current authorization before new effects. Owner APIs record decisions, recovery and request grants, not a general configuration administration subsystem.

### D14 — Reference protocol clients prove the first implementation

Implement real persistence, storage, API, lifecycle transitions and delivery/reconciliation mechanics. Reference worker and adapter clients use explicitly simulated engineering/provider outcomes to demonstrate the protocol. They do not impersonate real reviews, Runs, CI facts or AAS assessments.

Required demonstrations cover request intake/claim; suspension/decision/restoration on another worker; expiry/recovery; duplicate input/events/actions; lost acknowledgements; shared batches with differing outcomes; cancellation; budget exhaustion; exact-target confirmation; and restart persistence. Clients grow with C2–C5; C6 finishes integrated demonstrations and operational documentation.

## Responsibility and authority

| Component | Owns |
| --- | --- |
| Work Service | Requests, outer lifecycle progression, attempts, leases, decisions, checkpoints, actions, batches and retained history |
| External worker/harness | Agent processes, workspace, local workflow, permissions and independent reviewer execution |
| Local Azimuth tooling | Local eligibility, artifact contracts, output validation and review/correction relationships |
| Action adapter | Authorized provider invocation, deduplication/reconciliation and result reporting |
| CI/delivery system | Actual execution and exact source/artifact/release/deployment identities |
| AAS | Evidence applicability, computed Requirement Status and authoritative assurance-resolution facts |
| Authorized owner | Policy, grants, human decisions, recovery and escalation dispositions |

A successful upload establishes retained bytes, not a credible review. A worker report establishes an assertion, not an AAS assessment. A lifecycle transition cannot change accepted repository intent. Actual product decisions and permission remain with their owning contracts.

## Rejected alternatives

- Embedded agent processes, inbound worker launching or a persistent WebSocket session as a correctness requirement: adds runtime/session infrastructure before the assignment protocol is proven.
- Per-task assignment or configurable task groups initially: requires shared intermediate handoffs and partial-work ownership. Whole-request assignment is the approved boundary.
- One hard-coded lifecycle or progression inferred from prose: prevents explicit, versioned deterministic handling of different objectives.
- Automatic reassignment after expiry: cannot safely assume the old worker stopped or all local effects were retained.
- Closing a production correction when the worker delivers a PR: leaves release/deployment and exact confirmation outside the objective.
- One aggregate completion verdict per batch: hides independent request outcomes.
- Blind external-action retry or claiming exactly-once effects: confuses transport acknowledgement with provider execution.
- Semantic duplicate inference: can merge independent problems. Use explicit keys and retained occurrences.
- Unlimited autonomous attempts or mandatory approval on every routine retry: either unbounded resource consumption or unnecessary human interruption.
- A generic configuration UI/API, OIDC, capability matching, priority scheduling, automatic batch formation and live integrations initially: useful extensions, not necessary for the approved proof.

## Residual risks

| Risk | Required treatment |
| --- | --- |
| Old worker continues after lease loss | Reject stale mutations; check authority before consequential effects; reconcile uncertain operations |
| Provider executes but acknowledgement is lost | Stable identity and reconciliation before retry |
| Checkpoint bytes exist but restoration is incomplete | Validate dependency closure and demonstrate another worker restoring the workspace |
| Filesystem and database recover inconsistently | Coordinated backup/restore plus missing/orphaned bundle detection |
| New facts contradict a claimed basis | Retain and reevaluate continuation/completion without rewriting the old attempt |
| Shared delivery partially succeeds | Record operation/member outcomes; retry only necessary authorized work |
| Reference simulation hides provider assumptions | Explicit failure/recovery demonstrations; independently proposed live adapters |
| Self-reported outcomes are mistaken for assurance | Authoritative-source rules, authentication and explicit simulation separation |
| Outer and local workflow engines duplicate semantics | Resolve reuse boundary in C1; no assumption that current finite-task grammar already suffices |

## Open design work

No unresolved owner-level choice was identified at synthesis approval. The following are bounded proposal decisions, not established formats or commands:

- **U1:** exact workflow-engine reuse and the shipped outer lifecycle definitions, including request/attempt stage state and suspended continuations;
- **U2:** strict HTTP/record schemas, concurrency preconditions, time/lease boundaries, resource limits, idempotency namespaces, event ordering and occurrence correlation;
- **U3:** authentication-file/policy formats, grant narrowing and revocation semantics, permitted read scopes and simulation isolation;
- **U4:** checkpoint manifest/restoration contract, upload transactions, retention and coordinated backup/restore;
- **U5:** provider-operation reconciliation interface, incomplete failure-packet handling, cancellation races and shared-batch authorization;
- **U6:** exact confirmation freshness and target-correlation contracts; changed objectives require explicit decisions rather than silent mutation;
- **U7:** service paths and schema layout following applicable repository conventions, without borrowing consumer implementation dependencies.

Proposals must surface a material architectural contradiction for decision rather than silently overturn this exploration. Source inspection and product commands remain the permitted verification basis unless separate authorization expands it; persistence of this exploration authorizes no test execution.

## Relationship to other initiatives

The preceding workflow discussion identified symmetric spec/design/verification deltas, serialized authoring/review, focused task skills, Checks and implementation-directed Challenges before Requirement Review, structured findings, deterministic affected execution selection and durable publication inputs. Those local workflow/tooling changes remain a related initiative, not hidden additions to the six Work Service candidates.

CI/GitHub/Mergify, actual harness and AAS integrations are later candidates. Mergify is the selected consumer queue direction; no Enterprise dependency or live adapter is needed for this proof. The Work Service interface remains provider independent. Before-merge CI corrections may add an explicitly related correction change to the same PR; post-merge correction uses a new change/PR. Exact local change-record semantics belong to the related workflow initiative.

Any existing authority or provenance contracts revised by these candidates require explicit deltas. This exploration does not hand-edit accepted model files or derived architecture documents.

## Result

Dima approved the objective/boundaries, responsibility model, lifecycle/recovery behavior, protocol/durability contract, exclusions/risks and six-change graph in conversation on 2026-10-25, then authorized persistence. The persisted package remains `Status: exploring` pending review and approval of these actual files.

`change-map.md` contains the six candidate changes and their proposal boundaries. No change has been created or approved, no implementation started and no commit, publication, deployment or archival authorized by this package.
