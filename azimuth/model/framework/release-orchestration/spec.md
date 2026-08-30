# Spec: framework/release-orchestration

## Claim: qualification-lanes-converge
Criticality: routine

The first-alpha release SHALL qualify independent artifact lanes before one complete candidate
account can pass.

### Case: candidate-verification-reuses-retained-results
- Event: the canonical pull request or main candidate verification runs
- Required outcome: source, Assurance and candidate production execute in isolated parallel jobs
- Additional condition or outcome: downstream qualification consumes exact retained package and image artifacts without rebuilding them

### Case: selected-lanes-are-independent
- Event: first-alpha candidates are qualified
- Required outcome: packages, native binaries and assurance images execute in separate failure boundaries
- Additional condition or outcome: a successful lane retains its immutable outputs when another lane fails
- Additional condition or outcome: one final verification job observes every required source, deployment, qualification and candidate-account outcome

### Case: complete-account-needs-every-lane
- Event: release qualification converges
- Required outcome: the selected population is derived from the release catalog
- Additional condition or outcome: no missing, duplicate or unexpected candidate is accepted

## Claim: tagged-candidates-are-verifiable
Criticality: routine

Every retained first-alpha candidate SHALL remain attributable to one tagged source revision until
public verification completes.

*(revised)* Attribution may be direct or may chain a retained candidate to a separately attested
publication operation when immutable provider state forces an orchestration repair.

### Case: tag-catalog-and-revision-agree
- Event: a release candidate account is assembled
- Required outcome: its tag and version match the release catalog
- Additional condition or outcome: its full source revision is the commit named by that tag

### Case: retained-downloads-have-checksums
- Event: a downloadable candidate enters the retained account
- Required outcome: its filename, byte size and SHA-256 checksum are recorded
- Additional condition or outcome: later use rejects any byte change

### Case: executable-subjects-have-provenance
- Event: a downloadable candidate or container image is published
- Required outcome: GitHub build provenance identifies its exact checksum or image digest
- Additional condition or outcome: direct provenance or a retained-to-published provenance chain names the tagged source revision

## Claim: qualified-candidates-compose
Criticality: routine

Every selected first-alpha candidate SHALL expose its declared public entry point from the retained
distribution shape.

### Case: packed-packages-install
- Event: the five retained package candidates are given to disposable ecosystem consumers
- Required outcome: each consumer installs its candidate without a source-tree dependency
- Additional condition or outcome: exercises the package's declared public entry point

### Case: native-binaries-run
- Event: a retained native archive is exercised on its selected operating-system runner
- Required outcome: its Azimuth CLI starts and reports the selected release version

### Case: selected-image-platforms-start
- Event: each retained assurance image is exercised for a selected platform
- Required outcome: the image starts through its declared private-deployment entry point
- Additional condition or outcome: every catalog-selected image and platform is represented

## Claim: partial-publication-resumes-safely
Criticality: routine

The first-alpha publication SHALL preserve successful immutable targets and resume only targets
that are absent from retrieved registry state.

*(revised)* Public image retrieval is anonymous; organization or workflow credentials cannot
contribute to completion.

### Case: exact-existing-target-is-preserved
- Context: a registry target whose retrieved identity and digest match the retained candidate
- Event: publication is resumed
- Required outcome: that target is not overwritten or republished

### Case: absent-target-is-selected
- Context: a selected registry target that does not exist
- Event: publication is resumed
- Required outcome: only that absent target is selected for publication

### Case: conflicting-target-fails
- Context: a registry target whose immutable identity exists with different retained content
- Event: publication is resumed
- Required outcome: orchestration fails before any remaining target is published

### Case: completion-needs-public-retrieval
- Event: first-alpha publication is evaluated for completion
- Required outcome: every selected package, native archive and image index is retrieved publicly
- Additional condition or outcome: each image index is retrieved without registry credentials
- Additional condition or outcome: each retrieved target matches the retained tag, identity, checksum, provenance and platforms
- Additional condition or outcome: each npm prerelease channel selects its version
- Additional condition or outcome: `latest` does not select that prerelease when the package has any stable version
- Additional condition or outcome: *(revised)* `latest` selects that prerelease while the package has no stable version
