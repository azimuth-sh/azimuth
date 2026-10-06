# Accepted and prospective validation

```text
azimuth validate [local model options] [--out <report.json>]
azimuth validate --project <project.json> --workset <workset.json> [--out <report.json>]
azimuth change validate <change> [local model options] [--out <report.json>]
azimuth change validate <change> --project <project.json> --workset <workset.json> [--out <report.json>]
```

The project and workset options are required together. They select a complete federated account; local model, standards, workspace, manifest or partial selection options cannot redefine its authority. Local validation continues to accept explicit model, standards, workspace, manifest and support inputs. A change assessment requires the complete account, not `--only` selection.

The catalog owns repository topology, model sources, standards, packages and Area mounts. The workset pins required checkouts, revisions and manifest digests. Core derives source identities from repository-specific mounts and preserves exact supplied qualified identities. Missing or corrupt required observations are not replaced by local fallback.

Change validation stages the prospective intent and account facets before loading their source support. Candidate producer declarations can therefore resolve even when their entities are not accepted yet. It does not mutate accepted facets or turn candidate authority into accepted intent. Collision, stale delta, malformed declaration and unresolved structural reference remain errors.

Reports separate:

- Structural validity: whether declarations and references are well formed.
- Evidence completeness: missing source support and declared review obligations.
- Acceptance readiness: declared findings and uncompleted change obligations.

`--out` writes an `azimuth-validation` version 1 JSON report with `structural_validity`, `evidence_completeness`, `acceptance_readiness`, categorized `findings` and `acceptance_blockers`. No-declared-gaps or no-declared-blockers is bounded by the inspected account; neither is a supported Claim conclusion. Runtime independent reviews and exact-Subject Assurance State remain separate.

Structural errors cause nonzero validation. Evidence gaps and pending review obligations remain visible without being reclassified as parser failures. A separate explicit finalization/acceptance boundary retains its completeness requirements. Validation never executes native Checks, generates independent decisions, commits or archives.

The current alpha command has no `change check` or `change account-check` alias. Account preview/apply and intent preview/apply remain explicit authoring and acceptance operations.
