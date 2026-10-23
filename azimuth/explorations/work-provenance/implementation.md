# Work Provenance implementation plan

Status: in progress, 2026-10-24. Dima directed direct implementation without the Azimuth change workflow ("Azimuth is not yet mature enough to be used for Azimuth development"), with tests where necessary, and authorized implementation, commits, merges, image publication and the Assurance Service redeploy and reset through Terraform in `drim-dev-infra`.

This file records the implementation plan and the decisions taken after the exploration was approved. The exploration itself stays as approved.

## Decisions after approval

- **Ledger location (U2).** The committed ledger lives in an explicit `<change>/provenance/` folder, beside `reviews/`, never under the ignored `.azimuth/`.
- **Normalized storage.** Work Provenance is stored in hand-written relational tables with typed columns and no JSON or `Schema::Document` text. The 60-table cap test on generated schema is removed.
- **Reviews keep special semantics; provenance leaves them.** Requirement Reviews remain assurance records (assessment input, currentness, `reassesses`, reviewer credential binding). `provenance.agent` and `provenance.model` are removed; "who reviewed" is the generating attempt in the ledger. `reassesses` becomes a top-level field.
- **Review record normalization.** `dependencies` is replaced by `dependencies_fingerprint` (currentness only ever compared its canonical hash; the packet stays recoverable from the stored authority of the linked model revision). `challenge_basis` (now required, possibly empty) and `artifacts` are stored as child tables; Requirement Review storage holds no JSON text.
- **The authored `requirement-status` record kind is removed** everywhere: native records, Markdown compilation, workflow actions and templates that require it, Verification Challenge target kind, AAS ingestion and storage. Deployment Requirement Status remains computed. Historical review kinds (Method Qualification, Applicability Decision, Requirement Judgment) are no longer stored by AAS either.
- **Production reviews** are re-encoded and republished after the reset (schema v9 needs a fresh database).
- **Intent fingerprints.** The authority projection gains top-level `requirement_intents`, outside every review dependency packet.

## Work packages

Each package works in its own branch from `work-provenance` and edits only its owned paths. Formats are fixed by `contracts/work-provenance.md` and the review shape below.

| Package | Owns | Depends on |
|---|---|---|
| WP1 core-provenance | `tools/azimuth/src/provenance.rs`; provenance parts of `workflow_state.rs` (descriptor, timestamps, history sibling, version 3, predecessor, abandon); `main.rs` dispatch for `provenance` and `report provenance`; `change.rs` finalize/archive ledger; `installation.rs` skip list; `resources/references/workflow.md` and a new `work-provenance` reference; skills `azimuth-apply`, `azimuth-coordinate`, `azimuth-propose`, `azimuth-archive`; `contracts/workflow.md`; tests `tools/azimuth/tests/work_provenance.rs` | contract |
| WP2 core-reviews | `assurance_review.rs`, `review_authoring.rs`, `assurance_publication.rs`, `account_verification.rs`, `run_plan.rs`, `account_syntax_migration.rs`, review and `requirement-status` parts of `workflow_state.rs` and `resources/workflows/`, `contracts/assurance-reviews.md`, `contracts/run-bundle.md`, references `assurance-reviews.md` and `verification.md`, skill `azimuth-review-requirement`; review tests | contract |
| WP3 service | `services/assurance/server/**` (schema v9, review storage, migration `0009_work_provenance.sql`, `run_ledger/provenance.rs`, routes, tests), `services/assurance/README.md`, `contracts/run-ledger.md` | WP1, WP2 at integration |
| WP4 web | `services/assurance/web/**` | WP3 API contract |
| WP5 harness and transport | `tools/harness/claude-code/` hook and its tests, `scripts/publish-assurance.py` | WP1 contract |

Integration (lead): merge packages, `docs/` (glossary, intended architecture, change process), full build and tests, pull request and merge, image publication, CLI pin bump and hook installation in drim-dev, backfill of `private-api-service-ingress`, Terraform redeploy with reset, republication, verification.

## Review record shape (WP2, consumed by WP3 and WP4)

```json
{
  "kind": "requirement-review",
  "reviewer": "…",
  "reviewed_at_ms": 0,
  "rationale": "…",
  "artifacts": [{"id": "…", "digest": "sha256:…", "locator": "…"}],
  "requirement": "…",
  "requirement_fingerprint": "sha256:…",
  "dependencies_fingerprint": "sha256:…",
  "decision": "accepted",
  "reassesses": null,
  "challenge_basis": [],
  "fingerprint": "sha256:…"
}
```

The compiled bundle entry is `{id, input, reassesses, record, assessment}`. Markdown `Agent:` and `Model:` fields and `## Requirement status:` headings are rejected with a message naming Work Provenance.

## Residual items

- `release.rs` change-metadata `contributions[].agent/model` is a separate provenance site; it is left unchanged and recorded as a follow-up.
- Codex harness capture is not implemented; descriptors from Codex sessions are `declared` or `unknown`.
