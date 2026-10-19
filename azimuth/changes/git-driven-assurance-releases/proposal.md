# Change: Git-driven assurance releases and deployment reconciliation

Status: proposed
Intent delta: none
Because: This transition implements the already authorized composed-release lifecycle and observation transport without rewriting accepted framework Claims. New protocol obligations are specified in the release and deployment observation contracts; acceptance and archival remain explicit.

## Problem
Local workflow registration currently couples authoring to ASS. Consumer publication requires manual dispatch and uploaded generated records. Deployment observation has no durable reconciler or freshness signal, and the Web lacks an authenticated release acceptance control.

## Outcome
Git carries compact prepared inputs into a configured release repository. Trusted automation publishes named exact compositions and generated deployment values. A separate read-only reconciler observes actual rollout and reliably publishes deployment history and freshness. Owners accept a release explicitly in ASS.

## Scope
Canonical tooling, strict portable contracts, ASS API and themed UI, standalone reconciler and image publication; consumer Git-driven integration; GitOps environment bindings; disabled Terraform reconciler deployment and environment wiring.

## Affected claims
Existing framework/release-orchestration and framework/assurance-deployment Claims retain their accepted intent. No accepted Claim is replaced or removed. No automatic acceptance, object storage, fictional service or runtime deployment is included.

## Authorization
Dima explicitly authorized the discussed implementation on 2026-10-19. This proposal remains proposed until independently assessed; implementation authorization does not manufacture proposal or Claim Review acceptance.

## Completion conditions
Strict Git request and observation validation, offline local handoff, safe trusted CI, owner-only acceptance UI, idempotent ordered deployment observations and stale UI, reconciler Docker publication and disabled Terraform wiring are implemented. Native validation/compiler/type checks are recorded with pre-existing Findings and runtime limitations. No tests, deployment, credentials or acceptance facts are fabricated.
