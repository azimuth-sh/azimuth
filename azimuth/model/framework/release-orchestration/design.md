# Design: framework/release-orchestration

## Claim: qualification-lanes-converge
Mechanism: same-run-release-dag
Enforcement: guard
Binding: release-build-dag

One reusable continuous-integration graph derives package, native and image lanes from the catalog
and runs source and Assurance checks in parallel. Package qualification downloads the retained
package archives. Deployment qualification validates and imports the retained amd64 image
fragments, disables Compose builds and exercises those exact images. Multi-platform assembly
independently consumes the same fragments. One final check observes every required source,
artifact, deployment and qualification outcome. Pull requests, main pushes and tag-triggered
releases call the same graph; the tag run publishes only its own outputs. Removing the convergence
guard would let a partial lane population look like a complete release; rebuilding in a consumer
would let the tested bytes differ from the selected artifact.

## Claim: tagged-candidates-are-verifiable
Mechanism: catalog-bound-artifact-validation
Enforcement: choke-point
Binding: release-artifact-validator

The tag-triggered workflow derives the expected subject population directly from the catalog,
requires the annotated tag and version to agree, requires the tag revision to belong to main, and
accepts exactly one same-run file for every selected subject. It hashes the selected bytes into
`SHA256SUMS`; no candidate manifest is persisted or transferred. Attestation steps bind each lane's
outputs before upload, and publication consumes those same-run artifacts. Removing the validation
choke point would allow a missing, duplicate or substituted artifact to enter publication.

## Claim: qualified-candidates-compose
Mechanism: disposable-candidate-consumers
Enforcement: guard
Binding: release-consumer-rehearsal

Disposable Cargo, NuGet and npm consumers install retained archives rather than workspace source;
native runners execute retained CLI archives; emulated image checks start each selected platform.
Removing these checks would leave packer success as the only evidence that distributions compose.

## Claim: partial-publication-resumes-safely
Mechanism: registry-state-planner
Enforcement: choke-point
Binding: release-publication-plan

Before any write, one planner compares the complete selected population with retrieved registry
state and classifies every target as absent, exact or conflicting. Publication jobs consume only
the resulting absent set; any conflict prevents a plan. npm distribution tags and the stable-version
population are mutable metadata, so exact npm tarballs with channel drift enter a separate
normalization set without leaving the immutable preserve set. The tag-triggered publication jobs
wait for the reusable build graph and consume its same-run outputs. Provider adapters retrieve package bytes, npm distribution
tags, GitHub Release assets and GHCR index manifests into the closed-world state. GHCR inspection
uses `skopeo --no-creds`, and the completion job configures no registry login, because an
authenticated organization read does not establish public availability. A credential gate checks
presence plus provider-supported non-mutating identity before the first write. npm
publication derives an explicit distribution tag from the release's prerelease channel; stable
versions use `latest`. The npm registry package model requires `latest`, so a package whose only
versions are prereleases may expose the selected prerelease at both its explicit channel and
`latest`. Completion accepts that provider-required alias only while the retrieved package has no
stable version. If a stable version exists, a prerelease at `latest` remains a blocking ambiguity:
the adapter cannot derive which stable version the owner intends to select. NuGet retrieval requires
a valid repository signature and compares
a signature-independent payload digest because NuGet.org adds `.signature.p7s` during ingestion.
Completion performs a new retrieval after image-index provenance is attached and tag metadata is
normalized. Removing the planner would make partial recovery depend on mutable operator memory.

## Residue

The adapters cannot establish a narrowly scoped crates.io token's identity without requiring the
broader legacy scope, or establish permission to create an unused NuGet, GitHub Release or GHCR
identity without the first registry write. A provider may accept a write before its public read
surface exposes the immutable target; recovery requires a later observation rather than treating
an immediate absence as proof that no write occurred. A NuGet public archive's raw checksum differs
from the retained candidate after repository signing; the state account therefore records both and
uses non-signature payload identity for preservation. GitHub build provenance does not supply
complete SBOM or cross-ecosystem signing coverage. Public completion evidence is rollout-dependent
and cannot exist until `v0.1.0-alpha.1` is actually published; the release operation retains that
condition.

*(revised)* The `v0.1.0-alpha.1` publication discharged that condition and produced the
first public completion evidence. The condition itself is retained rather than removed: it
now binds each subsequent immutable version, which owes its own completion evidence and cannot
inherit the first alpha's.
