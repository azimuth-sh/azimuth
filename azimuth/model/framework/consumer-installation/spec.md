# Spec: framework/consumer-installation

## Claim: self-contained-installation
Criticality: routine

An initialized repository SHALL contain a release-matched Azimuth account and every selected agent workflow needed to use that account without a canonical-source checkout or network documentation.

### Case: integration-selection-is-explicit
- Event: a repository initializes Azimuth
- Required outcome: it explicitly selects Codex, Claude Code, both integrations or no agent integration
- Additional condition or outcome: the selection is recorded in `azimuth/installation.json`

### Case: consumer-resources-have-independent-authority
- Event: the CLI installs an agent workflow
- Required outcome: its bytes come from the release-owned consumer resource bundle
- Additional condition or outcome: no installed workflow requires the canonical Azimuth checkout or contributor-only skill tree

### Case: installed-files-are-accounted
- Event: initialization succeeds
- Required outcome: every CLI-managed repository file has an identity, repository-relative path and SHA-256 digest in `azimuth/installation.json`
- Additional condition or outcome: project-owned policy remains outside that managed file set

### Case: alias-adoption-is-bounded
- Event: initialization explicitly adopts an existing integration alias
- Required outcome: the CLI records a relative repository-internal alias to the exact supported managed skill location
- Additional condition or outcome: the CLI does not create a symlink

## Claim: version-matched-guidance
Criticality: routine

The installed CLI SHALL be the complete offline source for its artifact grammar and stage choreography.

### Case: stages-live-in-skills
- Event: a selected agent discovers an Azimuth stage
- Required outcome: its installed skill defines the sequence, gates, commands, stopping conditions and prohibited actions

### Case: references-live-in-the-cli
- Event: an author requests a supported artifact reference
- Required outcome: `azimuth reference list|show` returns the running release's parser-sensitive descriptor and bundled prose
- Additional condition or outcome: JSON output states the release, migration line and artifact format version

### Case: contextual-handoff-is-not-a-reference
- Event: an eligible work package is delegated
- Required outcome: `azimuth change brief` renders its change-specific handoff
- Additional condition or outcome: the retired `azimuth change instructions` operation is rejected

## Claim: coherent-installation-maintenance
Criticality: routine

Azimuth SHALL synchronize one complete managed-resource cohort without taking ownership of ecosystem package installation or user-authored account meaning.

### Case: components-are-explicitly-registered
- Event: an adopted annotation library or emitter is registered
- Required outcome: its supported identity, exact running release and repository-contained native manifest are validated before registration
- Additional condition or outcome: Azimuth does not edit the native manifest or invoke its package manager

### Case: update-fails-before-conflicting-writes
- Event: a managed resource was modified, an alias drifted or a registered component is not pinned to the running release
- Required outcome: `azimuth update` reports the conflict and writes no cohort member

### Case: update-is-offline-and-bounded
- Event: `azimuth update` applies an eligible cohort
- Required outcome: it replaces only CLI-managed resources and the installation account with the running release's cohort
- Additional condition or outcome: it performs no network discovery, CLI self-upgrade, package-manager operation or semantic account rewrite

## Claim: reviewed-account-migration
Criticality: routine

An incompatible account transition SHALL be planned and applied separately from managed-resource synchronization.

### Case: historical-reading-is-isolated
- Event: migration inspects a supported older account
- Required outcome: only a dedicated migration reader accepts the historical form
- Additional condition or outcome: normal validation continues to reject retired syntax

### Case: plan-is-content-addressed
- Event: `azimuth migrate plan` completes
- Required outcome: it records the migration line, exact releases, installation digest, deterministic edits, findings, disposition and canonical fingerprint

### Case: semantic-review-blocks-application
- Event: retired syntax or another meaning-bearing transition requires review
- Required outcome: the plan disposition is review-required and apply writes nothing

### Case: apply-refuses-drift-and-partial-work
- Event: `azimuth migrate apply` receives a plan
- Required outcome: it accepts only an automatic plan whose fingerprint and input digest still match
- Additional condition or outcome: it never inserts placeholders or applies an unrecognized edit set
