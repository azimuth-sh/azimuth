# Design: framework/release-orchestration

## Requirement: qualification-lanes-converge
Mechanism: retained-candidate-dag
Enforcement: guard
Binding: release-rehearsal-account

The hosted release workflow derives package, native and image lanes from the catalog, uploads each
lane's immutable outputs and lets a final account job accept only the exact selected population.
Ordinary CI retains its one canonical root command but excludes the release-only image matrix.
Removing the convergence guard would let a partial lane population look like a complete release.

## Requirement: tagged-candidates-are-verifiable
Mechanism: tagged-subject-manifest
Enforcement: choke-point
Binding: release-candidate-manifest

One deterministic manifest binds the catalog tag and version, full source revision and every
retained subject's identity, platform population, byte size and SHA-256 checksum. Attestation steps
bind each lane's retained outputs before upload; convergence accepts only those exact downloaded
filenames and hashes their bytes. Removing the manifest choke point would allow publication to
rebuild or substitute bytes after qualification.

## Requirement: qualified-candidates-compose
Mechanism: disposable-candidate-consumers
Enforcement: guard
Binding: release-consumer-rehearsal

Disposable Cargo, NuGet and npm consumers install retained archives rather than workspace source;
native runners execute retained CLI archives; emulated image checks start each selected platform.
Removing these checks would leave packer success as the only evidence that distributions compose.

## Requirement: partial-publication-resumes-safely
Mechanism: registry-state-planner
Enforcement: choke-point
Binding: release-publication-plan

Before any write, one planner compares the complete selected population with retrieved registry
state and classifies every target as absent, exact or conflicting. Publication jobs consume only
the resulting absent set; any conflict prevents a plan. The owner-dispatched publication workflow
downloads the tag-bound rehearsal outputs rather than rebuilding them. Provider adapters retrieve
package bytes, GitHub Release assets and GHCR index manifests into the closed-world state. A
credential gate checks presence plus provider-supported non-mutating identity before the first
write, and completion performs a new retrieval after image-index provenance is attached. Removing
the planner would make partial recovery depend on mutable operator memory.

## Residue

The adapters cannot establish a narrowly scoped crates.io token's identity without requiring the
broader legacy scope, or establish permission to create an unused NuGet, GitHub Release or GHCR
identity without the first registry write. GitHub build provenance does not supply complete SBOM
or cross-ecosystem signing coverage. Public completion evidence is rollout-dependent and cannot
exist until `v0.1.0-alpha.1` is actually published; the release operation retains that condition.
