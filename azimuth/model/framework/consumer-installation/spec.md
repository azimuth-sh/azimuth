# Spec: framework/consumer-installation

## Claim: self-contained-installation
Criticality: routine

An initialized repository SHALL contain a release-matched Azimuth account and every selected agent workflow needed to use that account without a canonical-source checkout or network documentation.

### Case: integration-selection-is-explicit
WHEN a repository initializes Azimuth
THEN it explicitly selects Codex, Claude Code, both integrations or no agent integration
AND the selection is recorded in `azimuth/installation.json`

### Case: consumer-resources-have-independent-authority
WHEN the CLI installs an agent workflow
THEN its bytes come from the release-owned consumer resource bundle
AND no installed workflow requires the canonical Azimuth checkout or contributor-only skill tree

### Case: installed-files-are-accounted
WHEN initialization succeeds
THEN every CLI-managed repository file has an identity, repository-relative path and SHA-256 digest in `azimuth/installation.json`
AND project-owned policy remains outside that managed file set

### Case: alias-adoption-is-bounded
WHEN initialization explicitly adopts an existing integration alias
THEN the CLI records a relative repository-internal alias to the exact supported managed skill location
AND the CLI does not create a symlink

## Claim: version-matched-guidance
Criticality: routine

The installed CLI SHALL be the complete offline source for its artifact grammar and stage choreography.

### Case: stages-live-in-skills
WHEN a selected agent discovers an Azimuth stage
THEN its installed skill defines the sequence, gates, commands, stopping conditions and prohibited actions

### Case: references-live-in-the-cli
WHEN an author requests a supported artifact reference
THEN `azimuth reference list|show` returns the running release's parser-sensitive descriptor and bundled prose
AND JSON output states the release, migration line and artifact format version

### Case: contextual-handoff-is-not-a-reference
WHEN an eligible work package is delegated
THEN `azimuth change brief` renders its change-specific handoff
AND the retired `azimuth change instructions` operation is rejected

## Claim: coherent-installation-maintenance
Criticality: routine

Azimuth SHALL synchronize one complete managed-resource cohort without taking ownership of ecosystem package installation or user-authored account meaning.

### Case: components-are-explicitly-registered
WHEN an adopted annotation library or emitter is registered
THEN its supported identity, exact running release and repository-contained native manifest are validated before registration
AND Azimuth does not edit the native manifest or invoke its package manager

### Case: update-fails-before-conflicting-writes
WHEN a managed resource was modified, an alias drifted or a registered component is not pinned to the running release
THEN `azimuth update` reports the conflict and writes no cohort member

### Case: update-is-offline-and-bounded
WHEN `azimuth update` applies an eligible cohort
THEN it replaces only CLI-managed resources and the installation account with the running release's cohort
AND it performs no network discovery, CLI self-upgrade, package-manager operation or semantic account rewrite

## Claim: reviewed-account-migration
Criticality: routine

An incompatible account transition SHALL be planned and applied separately from managed-resource synchronization.

### Case: historical-reading-is-isolated
WHEN migration inspects a supported older account
THEN only a dedicated migration reader accepts the historical form
AND normal validation continues to reject retired syntax

### Case: plan-is-content-addressed
WHEN `azimuth migrate plan` completes
THEN it records the migration line, exact releases, installation digest, deterministic edits, findings, disposition and canonical fingerprint

### Case: semantic-review-blocks-application
WHEN retired syntax or another meaning-bearing transition requires review
THEN the plan disposition is review-required and apply writes nothing

### Case: apply-refuses-drift-and-partial-work
WHEN `azimuth migrate apply` receives a plan
THEN it accepts only an automatic plan whose fingerprint and input digest still match
AND it never inserts placeholders or applies an unrecognized edit set
