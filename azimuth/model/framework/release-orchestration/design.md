# Design: framework/release-orchestration

## Claim: qualification-lanes-converge
Mechanism: retained-candidate-dag
Enforcement: guard
Binding: release-rehearsal-account

One hosted candidate-verification graph derives package, native and image lanes from the catalog
and runs source and Assurance checks in parallel. Package qualification downloads the retained
package archives. Deployment qualification validates and imports the retained amd64 image
fragments, disables Compose builds and exercises those exact images. Multi-platform assembly
independently consumes the same fragments, and the final account accepts only the exact selected
population. One final check observes every required source, deployment, qualification and account
outcome. Removing the convergence guard would let a partial lane population look like a complete
release; rebuilding in a consumer would let the tested bytes differ from the retained candidate.

## Claim: tagged-candidates-are-verifiable
Mechanism: tagged-subject-manifest
Enforcement: choke-point
Binding: release-candidate-manifest

One deterministic manifest binds the catalog tag and version, full source revision and every
retained subject's identity, platform population, byte size and SHA-256 checksum. Attestation steps
bind each lane's retained outputs before upload; convergence accepts only those exact downloaded
filenames and hashes their bytes. Removing the manifest choke point would allow publication to
rebuild or substitute bytes after qualification.

Publication normally runs at that tagged revision. When an immutable provider exposes a defect in
the publication adapter after some targets exist, a reviewed repair revision may execute the
operation against the unchanged tag-bound account. The completion receipt records both revisions.
For a published image digest, the chain requires retained-archive provenance at the candidate
revision, deterministic archive-to-index identity and registry-digest provenance at the repair
revision. This distinguishes artifact source from the later operation that made it public.

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
normalization set without leaving the immutable preserve set. The owner-dispatched publication
workflow downloads the tag-bound rehearsal
outputs rather than rebuilding them. Provider adapters retrieve package bytes, npm distribution
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
now binds each subsequent immutable version, so `v0.1.0-alpha.4` owes its own completion
evidence and cannot inherit the first alpha's.
