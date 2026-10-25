# Candidate change map: Azimuth Work Service

Candidates only. Each requires its own bounded proposal and approval. This map does not authorize implementation, commits or deployment.

```text
C1 work-service-protocol
          |
          v
C2 work-service-assignment
          |
          v
C3 work-service-continuation
          |
          v
C4 work-service-lifecycle
          |
          v
C5 work-service-delivery-batches
          |
          v
C6 work-service-protocol-demonstration
```

The arrows show proposal/implementation dependency. Reference clients and focused protocol demonstrations develop alongside C2–C5; C6 integrates them rather than deferring all observable verification until the end.

## C1 — work-service-protocol

Depends on: none in this graph; inspect current workflow, assurance, provenance and delivery contracts.

Decisions: D1–D14; resolves the shared structural parts of U1–U7.

Define the service boundary and strict contracts for requests, worker attempts, decisions, checkpoints, grants, actions, milestones, batches and completion. Define identity, replay/conflict behavior, authoritative sources, simulation isolation, failure classification and correction relationships. Decide how the outer lifecycle reuses existing Azimuth machinery without centralizing the local workflow. Select a small initial set of pinned lifecycle definitions, including a shorter objective and an end-to-end production correction.

Retain explicit authorization for sources/actions, versioning, lifecycle input and outcome contracts, and the state invariants subsequent slices implement. Do not create placeholders claiming implemented behavior or automatic provider integration.

Completion boundary: an approved, coherent protocol/design basis with observable completion conditions for each following slice. Unresolved material runtime questions cannot be silently delegated to implementers.

## C2 — work-service-assignment

Depends on: C1.

Decisions: D1–D5, D10–D13.

Implement the separate Rust backend and owned PostgreSQL persistence. Add validated operator configuration, scoped bearer permissions, common intake, explicit deduplication/problem occurrences, owner-configured pools, atomic oldest-eligible claims, lease renewal, ownership generations, progress and recovery-required state. Define the minimal initial read APIs and retained attempt history. No automatic reassignment or embedded agent runtime.

Completion boundary: a reference client can submit and replay a request, claim it from permitted pools, renew/report with current ownership, and observe expiry blocking reassignment until authorized recovery. Restart preserves accepted state; stale workers cannot overwrite newer ownership. Early client functionality is retained for C6.

## C3 — work-service-continuation

Depends on: C2.

Decisions: D5–D7, D10, D13–D14.

Implement staged filesystem bundle upload, manifests/digests and PostgreSQL references, retrieval and safe restoration. Add generic decision exchange, authenticated responses, basis pinning and checkpoint-before-release suspension. Continuation can be claimed by a different permitted worker. Cover rejected/incomplete uploads, missing dependencies and retained final outputs. Document retention, orphan handling and coordinated backup/restore.

Completion boundary: worker A suspends with a verified reconstructible checkpoint, an owner answers through the API, and worker B restores the local continuation and resumes without access to A's original workspace or credentials. A byte-valid but incomplete checkpoint cannot be accepted as reconstructible.

## C4 — work-service-lifecycle

Depends on: C2 and C3.

Decisions: D3, D8–D14.

Implement pinned outer lifecycle progression using the engine boundary chosen in C1. Add transactional outbox/inbox, external-action polling, authenticated milestone ingestion, source/target correlation, deduplication, operation reconciliation and waits that release agent ownership. Enforce project policy/request grants before dispatch, bounded attempts, cooperative cancellation and recovery for uncertain effects. Retain explicit distinctions among action receipt, execution success, failed/inconclusive work and confirmed objective completion.

Completion boundary: reference adapters demonstrate deterministic progression, duplicate delivery, a lost acknowledgement with reconciliation, out-of-order/late events, restart, cancellation, budget exhaustion and resumption. Simulated events cannot establish real AAS support. Live GitHub/AAS delivery is excluded.

## C5 — work-service-delivery-batches

Depends on: C4.

Decisions: D8–D12.

Add explicit authorized APIs to assemble ready requests into compatible release/deployment batches and pin exact delivery inputs and target. Freeze membership at dispatch. Correlate shared external operations while retaining per-request completion and confirmation. Handle partial outcomes, member cancellation, later corrections and recurrence without rewriting a dispatched batch or closing every request from one aggregate success.

Completion boundary: two requests share a simulated delivery, one is confirmed and the other remains unresolved, and only the appropriate request continues correction. Nonmatching targets, insufficient authority and incompatible inputs do not dispatch. Automatic batch scheduling remains deferred.

## C6 — work-service-protocol-demonstration

Depends on: C2–C5; reference clients are incrementally available earlier.

Decisions: D14; verifies the integrated boundaries of D1–D13.

Complete reference worker/adapter clients and reproducible local operating instructions for the real API, PostgreSQL database and filesystem storage. Demonstrate human/CI/AAS-origin envelopes with explicitly simulated domain operations, whole-request claims, independent local workflow ownership, decision suspension/restoration, owner-controlled expiry recovery, action/event replay, lost acknowledgements, bounded retries, cancellation, batch partial results and exact-target confirmation. Demonstrate persistence over service restart and document backup/restore and required external privileges for future adapters.

Completion boundary: an operator can reconstruct each demonstrated progression and retained basis without a UI, agent vendor, live GitHub credentials, real AAS submission or cloud deployment. Reports clearly distinguish executed protocol behavior from simulated provider outcomes and unproven live-integration assumptions. Use repository-permitted verification; this map does not authorize otherwise prohibited test work.

## Subsequent candidates outside this graph

- Live GitHub/Mergify CI intake and correction-report adapters.
- Live AAS situation intake and exact deployment-confirmation adapter.
- Real agent-harness workers and process/isolation management in those workers.
- Provider release/deployment adapters with explicit authority and reconciliation.
- Deterministic automatic batch formation and optional richer worker scheduling.
- Object storage, configuration management APIs, OIDC, streaming notification and a UI where justified.
- The separately discussed local workflow, symmetric delta, focused skill, structured finding and affected execution selection changes. Coordinate dependencies through proposals rather than absorbing them into this service graph.
