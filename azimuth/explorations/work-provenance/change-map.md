# Candidate change map: Work Provenance ledger

Candidates only. Each change requires its own `azimuth-propose` proposal and approval.

```text
X1 probe harnesses ──┐
                     ▼
              C1 workflow-attempt-provenance
              ┌──────┼──────────────┐
              ▼      ▼              ▼
   C2 review-   C3 committed-   C7 harness-
   provenance-  provenance-     provenance-
   descriptor   ledger          capture
              └──────┬──────┘
                     ▼
              C4 assurance-work-provenance ──► X2 backfill private-api-service-ingress
                     ▼
              C5 requirement-provenance-history
                     ▼
              C6 failure-attribution
```

C1 with C3 is the minimal slice: it closes F4 by making history durable and enriched with no service work.

## C1 — workflow-attempt-provenance

Depends on: `workflow-coordination`; informed by X1.

`workflow update` accepts the agent descriptor (D3) with a basis for each field (D6). The contract narrows `actor` to the execution instance (D4). The tool stamps `handoff_captured_at` and `recorded_at`. Restarted instances record an explicit predecessor link, abandoned changes an `abandoned` disposition (D7), and an attempt may carry an optional `trace` reference. Provenance is excluded from attempt identity.

## C2 — review-provenance-descriptor

Depends on: C1.

Requirement Review and Requirement Status `provenance` adopt the descriptor in place of free `agent`/`model` labels across the contract, review compilation and AAS validation.

## C3 — committed-provenance-ledger

Depends on: C1.

`change finalize` and `archive` write the committed ledger into the change folder, defining its format and resolving U2.

## C7 — harness-provenance-capture

Depends on: X1, C1.

A shipped Claude Code integration, and Codex where X1 permits, supplies descriptor values with `harness` basis. Installed and maintained through `azimuth-maintain`.

## C4 — assurance-work-provenance

Depends on: C2, C3.

The Work Provenance domain in AAS: its own schema and API, the non-assessment invariant (D2), ingestion under the publisher credential, and transport in the publication script. Revises `docs/intended-architecture.md:132` and `:136` and the glossary to separate provenance from telemetry.

## C5 — requirement-provenance-history

Depends on: C4.

Service-computed fingerprint-closure attribution labeled `intent` or `dependency` (D5); `azimuth report provenance --requirement <id>` with JSONL output, reading AAS exports through `--assurance <file>`; and the history panel on the Requirement page.

## C6 — failure-attribution

Depends on: C5.

The append-only Failure Attribution record (D8) with caused and allowed-through roles, rationale, confidence and investigator descriptor, its authoring path and its UI. U1 is reconsidered here.

## Outside the change graph

- On persistence, `agent-tier-representation` receives a note that its E10 is resolved by this exploration.
- X2 runs after C4, against the backfilled drim-dev instance.
