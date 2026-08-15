# Spec: framework/release-orchestration

## Requirement: qualification-lanes-converge
Criticality: standard

The first-alpha release SHALL qualify independent artifact lanes before one complete candidate
account can pass.

### Scenario: ordinary-ci-excludes-release-only-matrix
WHEN the canonical pull request or main repository check runs
THEN it executes the complete ordinary repository gate below its hosted-job limit
AND it does not build the release-only multi-platform image matrix

### Scenario: selected-lanes-are-independent
WHEN first-alpha candidates are qualified
THEN packages, native binaries and assurance images execute in separate failure boundaries
AND a successful lane retains its immutable outputs when another lane fails

### Scenario: complete-account-needs-every-lane
WHEN release qualification converges
THEN the selected population is derived from the release catalog
AND no missing, duplicate or unexpected candidate is accepted

## Requirement: tagged-candidates-are-verifiable
Criticality: critical

Every retained first-alpha candidate SHALL remain attributable to one tagged source revision until
public verification completes.

### Scenario: tag-catalog-and-revision-agree
WHEN a release candidate account is assembled
THEN its tag and version match the release catalog
AND its full source revision is the commit named by that tag

### Scenario: retained-downloads-have-checksums
WHEN a downloadable candidate enters the retained account
THEN its filename, byte size and SHA-256 checksum are recorded
AND later use rejects any byte change

### Scenario: executable-subjects-have-provenance
WHEN a downloadable candidate or container image is published
THEN GitHub build provenance identifies its exact checksum or image digest
AND the provenance names the tagged source revision

## Requirement: qualified-candidates-compose
Criticality: standard

Every selected first-alpha candidate SHALL expose its declared public entry point from the retained
distribution shape.

### Scenario: packed-packages-install
WHEN the five retained package candidates are given to disposable ecosystem consumers
THEN each consumer installs its candidate without a source-tree dependency
AND exercises the package's declared public entry point

### Scenario: native-binaries-run
WHEN a retained native archive is exercised on its selected operating-system runner
THEN its Azimuth CLI starts and reports the selected release version

### Scenario: selected-image-platforms-start
WHEN each retained assurance image is exercised for a selected platform
THEN the image starts through its declared private-deployment entry point
AND every catalog-selected image and platform is represented

## Requirement: partial-publication-resumes-safely
Criticality: critical

The first-alpha publication SHALL preserve successful immutable targets and resume only targets
that are absent from the retained release account.

### Scenario: exact-existing-target-is-preserved
GIVEN a registry target whose retrieved identity and digest match the retained candidate
WHEN publication is resumed
THEN that target is not overwritten or republished

### Scenario: absent-target-is-selected
GIVEN a selected registry target that does not exist
WHEN publication is resumed
THEN only that absent target is selected for publication

### Scenario: conflicting-target-fails
GIVEN a registry target whose immutable identity exists with different retained content
WHEN publication is resumed
THEN orchestration fails before any remaining target is published

### Scenario: completion-needs-public-retrieval
WHEN first-alpha publication is evaluated for completion
THEN every selected package, native archive and image index is retrieved publicly
AND each retrieved target matches the retained tag, identity, checksum, provenance and platforms
