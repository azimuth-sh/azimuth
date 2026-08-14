# Design delta: framework/release-orchestration

## Add requirement: qualification-lanes-converge
Mechanism: retained-candidate-dag
Enforcement: guard
Binding: release-rehearsal-account

The hosted release workflow derives package, native and image lanes from the catalog, uploads each
lane's immutable outputs and lets a final account job accept only the exact selected population.
Ordinary CI retains its one canonical root command but excludes the release-only image matrix.
Removing the convergence guard would let a partial lane population look like a complete release.

## Add requirement: tagged-candidates-are-verifiable
Mechanism: tagged-subject-manifest
Enforcement: choke-point
Binding: release-candidate-manifest

One deterministic manifest binds the catalog tag and version, full source revision and every
retained subject's identity, platform population, byte size and SHA-256 checksum. Attestation steps
bind each lane's retained outputs before upload; convergence accepts only those exact downloaded
filenames and hashes their bytes. Removing the manifest choke point would allow publication to
rebuild or substitute bytes after qualification.

## Add requirement: qualified-candidates-compose
Mechanism: disposable-candidate-consumers
Enforcement: guard
Binding: release-consumer-rehearsal

Disposable Cargo, NuGet and npm consumers install retained archives rather than workspace source;
native runners execute retained CLI archives; emulated image checks start each selected platform.
Removing these checks would leave packer success as the only evidence that distributions compose.

## Add requirement: partial-publication-resumes-safely
Mechanism: registry-state-planner
Enforcement: choke-point
Binding: release-publication-plan

Before any write, one planner compares the complete selected population with retrieved registry
state and classifies every target as absent, exact or conflicting. Publication jobs consume only
the resulting absent set; any conflict prevents a plan. Completion rejects an incomplete state,
but the publication operation must still bind each registry adapter to a fresh public read.
Removing the planner would make partial recovery depend on mutable operator memory.

## Residue

The rehearsal cannot establish that future credentials are valid or that a registry will be
available during publication. GitHub build provenance does not supply complete SBOM or
cross-ecosystem signing coverage. Public completion evidence is rollout-dependent and cannot exist
until `v0.1.0-alpha.1` is actually published; the release operation must retain that condition.
