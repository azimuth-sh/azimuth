# Change: Deployment Claim Status from applicable evidence
Status: active

Implementation authorization: Dima approved the deployment-focused design and explicitly requested implementation on 2026-10-20. Dima subsequently explicitly authorized ASS deployment and restarting the drim.dev Azimuth process with fresh evidence. Do not accept releases or archive this change implicitly.

## Problem
The current assessor requires independently authored exact-Subject health conclusions and rejects cross-Subject Observations. CI engineering gates retain native reports without generating fresh Runs. Context maps have no effective-configuration observation contract. Deployment freshness is separate from verification freshness. This leaves deployed Claims unresolved even when their required facts can be established across source, artifact and deployment stages.

## Outcome
ASS deterministically explains deployment Claim Status using a current accepted independent Claim Review, current applicable Check observations, explicit release lineage, effective verification contexts and Monitor freshness. Source-only status is not a new assessment product. A public xUnit integration and Playwright integration capture execution facts and context; separate adapters construct native Runs and a separate publisher delivers them. Adopt the complete path in drim.dev, locally and in CI, including a verified data-preserving production rollout.

## Scope
Canonical protocol/core/CLI; public context and test integrations; shared normalization and publisher; ASS relational persistence/API/UI; collector/reconciler context collection; consumer application, test fixtures, CI and deployment configuration. Preserve exact historical records, adverse outcomes, correction heads and release ownership. Use synthetic canonical fixtures only. Existing Claim IDs remain stable. Concrete intent deltas will pin the affected canonical framework Claims before current intent is applied; do not infer accepted intent from this implementation authorization.

## Completion
Trace a synthetic source Check through a release to a deployment; demonstrate mismatched source/artifact/context, missing/overdue monitoring, corrections and adverse outcomes. No arbitrary failure becomes a violation. Real xUnit and Playwright tests emit typed facts and captures; adapters generate protocol-verified Runs and a separate publisher retries without rerunning tests. Consumer CI exports fresh Runs even on failure where feasible. New status endpoint/UI identify reasons, dependencies and evaluated time. Real PostgreSQL upgrade preserves existing records. Permitted tests pass. After verified rollout, restart the consumer Azimuth workflow and publish fresh actual CI/production evidence; retain unresolved outcomes honestly.
